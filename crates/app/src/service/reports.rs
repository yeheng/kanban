use crate::error::AppError;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ReportKind {
    ResourceUtilization,
    ProjectBurn,
    AiDecisions,
    Cost,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportTable {
    pub title: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct ReportService;
impl ReportService {
    pub async fn build(
        pool: &SqlitePool, kind: ReportKind, project_id: Option<i64>, start: &str, end: &str,
    ) -> Result<ReportTable, AppError> {
        match kind {
            ReportKind::ResourceUtilization => Self::resource_utilization(pool, start, end).await,
            ReportKind::ProjectBurn => Self::project_burn(pool).await,
            ReportKind::AiDecisions => Self::ai_decisions(pool).await,
            ReportKind::Cost => Self::cost(pool, project_id).await,
        }
    }

    async fn resource_utilization(pool: &SqlitePool, start: &str, end: &str) -> Result<ReportTable, AppError> {
        let mut rows = Vec::new();
        for r in db::ResourcesRepo::list_active(pool).await? {
            let s = crate::service::workload::WorkloadService::resource_summary(pool, r.id, start, end).await?;
            rows.push(vec![
                r.name, fmt(s.capacity_pd), fmt(s.workload_pd), fmt(s.utilization),
                s.overloaded.to_string(),
            ]);
        }
        Ok(ReportTable {
            title: "Resource Utilization".into(),
            columns: cols(["resource", "capacity_pd", "workload_pd", "utilization", "overloaded"]),
            rows,
        })
    }

    async fn project_burn(pool: &SqlitePool) -> Result<ReportTable, AppError> {
        let mut rows = Vec::new();
        for p in db::ProjectsRepo::list_active(pool).await? {
            let b = crate::service::workload::WorkloadService::project_burn(pool, p.id).await?;
            rows.push(vec![p.name, fmt(b.budget_pd), fmt(b.allocated_pd), fmt(b.usage)]);
        }
        Ok(ReportTable {
            title: "Project Budget Burn".into(),
            columns: cols(["project", "budget_pd", "allocated_pd", "usage"]),
            rows,
        })
    }

    /// R4: structured AI decision records only (no LLM prompt/response — confirmed #40).
    async fn ai_decisions(pool: &SqlitePool) -> Result<ReportTable, AppError> {
        let rows: Vec<(i64, String, i64, Option<f64>, String)> = sqlx::query_as(
            "SELECT id, status, applied, score_overall, created_at \
             FROM ai_optimization_runs ORDER BY id DESC LIMIT 200")
            .fetch_all(pool).await?;
        Ok(ReportTable {
            title: "AI Decision Records".into(),
            columns: cols(["run_id", "status", "applied", "score_overall", "created_at"]),
            rows: rows.into_iter()
                .map(|(id, st, ap, sc, ts)| {
                    vec![id.to_string(), st, ap.to_string(), sc.map(fmt).unwrap_or_default(), ts]
                })
                .collect(),
        })
    }

    /// R7: cost = Σ allocated_pd × effective_daily_rate(resource, project).
    ///
    /// allocated_pd is computed DYNAMICALLY via the Phase 0 alloc_pd over each allocation's
    /// full span (calendar-aware) — the `allocations.allocated_pd` cache column is never
    /// populated (defaults 0), so SUM(allocated_pd) would always be 0. effective_daily_rate =
    /// resource_project_rates (latest valid_from) → resources.daily_rate_pd → 0.
    async fn cost(pool: &SqlitePool, project_id: Option<i64>) -> Result<ReportTable, AppError> {
        let cal = db::repo::calendar::hydrate(pool).await?;
        // rows: resource_id, resource_name, project_id, project_name, start, end, percent
        let q = match project_id {
            Some(pid) => sqlx::query_as::<_, (i64, String, i64, String, chrono::NaiveDate, chrono::NaiveDate, f64)>(
                "SELECT r.id, r.name, p.id, p.name, a.start_date, a.end_date, a.percent \
                 FROM allocations a JOIN resources r ON r.id=a.resource_id \
                 JOIN tasks t ON t.id=a.task_id JOIN projects p ON p.id=t.project_id \
                 WHERE p.id=? AND a.deleted_at IS NULL").bind(pid),
            None => sqlx::query_as::<_, (i64, String, i64, String, chrono::NaiveDate, chrono::NaiveDate, f64)>(
                "SELECT r.id, r.name, p.id, p.name, a.start_date, a.end_date, a.percent \
                 FROM allocations a JOIN resources r ON r.id=a.resource_id \
                 JOIN tasks t ON t.id=a.task_id JOIN projects p ON p.id=t.project_id \
                 WHERE a.deleted_at IS NULL"),
        };
        let allocs = q.fetch_all(pool).await?;

        // Aggregate PD per (resource_id, project_id), then resolve rate per (resource, project).
        use std::collections::HashMap;
        let mut pd: HashMap<(i64, i64), f64> = HashMap::new();
        let mut names: HashMap<(i64, i64), (String, String)> = HashMap::new();
        for (rid, rname, pid, pname, start, end, percent) in &allocs {
            let a = domain::Allocation { id: 0, resource_id: *rid, project_id: *pid, start: *start, end: *end, percent: *percent };
            let span = domain::Window { start: *start, end: *end };
            *pd.entry((*rid, *pid)).or_insert(0.0) += domain::alloc_pd(&cal, &a, span);
            names.entry((*rid, *pid)).or_insert((rname.clone(), pname.clone()));
        }

        let mut out = Vec::new();
        let mut total = 0.0;
        for ((rid, pid), pdays) in &pd {
            let (rname, pname) = names.get(&(*rid, *pid)).cloned().unwrap_or_default();
            let rate = effective_daily_rate(pool, *rid, *pid).await?;
            let cost = pdays * rate;
            total += cost;
            out.push(vec![rname, pname, fmt(*pdays), fmt(rate), fmt(cost)]);
        }
        out.push(vec!["TOTAL".into(), "".into(), "".into(), "".into(), fmt(total)]);
        Ok(ReportTable {
            title: "Cost".into(),
            columns: cols(["resource", "project", "allocated_pd", "daily_rate_pd", "cost"]),
            rows: out,
        })
    }

    /// Workforce snapshot (JSON) — current utilization of all resources over a window.
    pub async fn snapshot_json(pool: &SqlitePool, start: &str, end: &str) -> Result<String, AppError> {
        let mut entries = Vec::new();
        for r in db::ResourcesRepo::list_active(pool).await? {
            let s = crate::service::workload::WorkloadService::resource_summary(pool, r.id, start, end).await?;
            entries.push(serde_json::json!({
                "resource": r.name, "capacity_pd": s.capacity_pd, "workload_pd": s.workload_pd,
                "utilization": s.utilization, "overloaded": s.overloaded,
            }));
        }
        serde_json::to_string_pretty(&serde_json::json!({
            "window": { "start": start, "end": end }, "resources": entries,
        })).map_err(|e| AppError::internal(e.to_string()))
    }
}

/// effective_daily_rate: latest resource_project_rates row → resources.daily_rate_pd → 0.
async fn effective_daily_rate(pool: &SqlitePool, resource_id: i64, project_id: i64) -> Result<f64, AppError> {
    let row: Option<(f64,)> = sqlx::query_as(
        "SELECT daily_rate_pd FROM resource_project_rates \
         WHERE resource_id=? AND project_id=? ORDER BY valid_from DESC LIMIT 1")
        .bind(resource_id).bind(project_id).fetch_optional(pool).await?;
    if let Some((r,)) = row { return Ok(r); }
    let (rate,): (Option<f64>,) = sqlx::query_as("SELECT daily_rate_pd FROM resources WHERE id=?")
        .bind(resource_id).fetch_one(pool).await?;
    Ok(rate.unwrap_or(0.0))
}

fn fmt(v: f64) -> String { format!("{:.2}", v) }
fn cols<const N: usize>(arr: [&str; N]) -> Vec<String> { arr.iter().map(|s| s.to_string()).collect() }
