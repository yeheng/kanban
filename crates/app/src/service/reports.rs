use crate::error::AppError;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ReportKind {
    ResourceUtilization,
    TeamUtilization,
    ProjectBurn,
    AiDecisions,
    Cost,
}

/// A catalog entry describing a report kind: id, display title, the export formats
/// available for it, and whether it accepts a project_id filter. Surfaced via
/// `report_catalog()` so the frontend can render the design's report roadmap and hide
/// unavailable formats instead of failing on export (design §8 / G5).
#[derive(Debug, Clone, Serialize)]
pub struct ReportCatalogEntry {
    pub kind: String,
    pub title: String,
    pub description: String,
    pub formats: Vec<String>,
    pub accepts_project_id: bool,
    pub mvp: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportTable {
    pub title: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct ReportService;
impl ReportService {
    #[tracing::instrument(skip(pool), fields(kind = ?kind, project_id = project_id, start = %start, end = %end))]
    pub async fn build(
        pool: &SqlitePool, kind: ReportKind, project_id: Option<i64>, start: &str, end: &str,
    ) -> Result<ReportTable, AppError> {
        match kind {
            ReportKind::ResourceUtilization => Self::resource_utilization(pool, start, end).await,
            ReportKind::TeamUtilization => Self::team_utilization(pool, start, end).await,
            ReportKind::ProjectBurn => Self::project_burn(pool).await,
            ReportKind::AiDecisions => Self::ai_decisions(pool).await,
            ReportKind::Cost => Self::cost(pool, project_id).await,
        }
    }

    #[tracing::instrument(skip(pool), fields(start = %start, end = %end))]
    async fn resource_utilization(pool: &SqlitePool, start: &str, end: &str) -> Result<ReportTable, AppError> {
        let cal = db::repo::calendar::hydrate(pool).await?;
        // Global PM conversion for report-level PM columns (design §2.9; cross-resource
        // aggregation uses the global N, not per-team overrides).
        let unit = crate::service::thresholds::global_unit_config(pool).await?;
        let mut rows = Vec::new();
        for r in db::ResourcesRepo::list_active(pool).await? {
            let s = crate::service::workload::WorkloadService::resource_summary_with_cal(pool, &cal, r.id, start, end).await?;
            rows.push(vec![
                r.name, fmt(s.capacity_pd), fmt(s.workload_pd), fmt(s.utilization),
                s.overloaded.to_string(),
                fmt(unit.pd_to_pm(s.capacity_pd)), fmt(unit.pd_to_pm(s.workload_pd)),
            ]);
        }
        Ok(ReportTable {
            title: "Resource Utilization".into(),
            columns: cols(["resource", "capacity_pd", "workload_pd", "utilization", "overloaded", "capacity_pm", "workload_pm"]),
            rows,
        })
    }

    /// Team utilization report (design §8 / G5 "by team" aggregation dimension).
    /// Per active team: Σ workload / Σ capacity + overloaded-member count.
    #[tracing::instrument(skip(pool), fields(start = %start, end = %end))]
    async fn team_utilization(pool: &SqlitePool, start: &str, end: &str) -> Result<ReportTable, AppError> {
        let unit = crate::service::thresholds::global_unit_config(pool).await?;
        let mut rows = Vec::new();
        for t in db::TeamsRepo::list_active(pool).await? {
            let s = crate::service::workload::WorkloadService::team_summary(pool, t.id, start, end).await?;
            rows.push(vec![
                t.name, s.overloaded_members.len().to_string(),
                fmt(s.capacity_pd), fmt(s.workload_pd), fmt(s.utilization),
                fmt(unit.pd_to_pm(s.capacity_pd)), fmt(unit.pd_to_pm(s.workload_pd)),
            ]);
        }
        Ok(ReportTable {
            title: "Team Utilization".into(),
            columns: cols(["team", "overloaded_members", "capacity_pd", "workload_pd", "utilization", "capacity_pm", "workload_pm"]),
            rows,
        })
    }

    #[tracing::instrument(skip(pool))]
    async fn project_burn(pool: &SqlitePool) -> Result<ReportTable, AppError> {
        let unit = crate::service::thresholds::global_unit_config(pool).await?;
        let mut rows = Vec::new();
        for p in db::ProjectsRepo::list_active(pool).await? {
            let b = crate::service::workload::WorkloadService::project_burn(pool, p.id).await?;
            rows.push(vec![
                p.name, fmt(b.budget_pd), fmt(b.allocated_pd), fmt(b.usage),
                fmt(unit.pd_to_pm(b.budget_pd)), fmt(unit.pd_to_pm(b.allocated_pd)),
            ]);
        }
        Ok(ReportTable {
            title: "Project Budget Burn".into(),
            columns: cols(["project", "budget_pd", "allocated_pd", "usage", "budget_pm", "allocated_pm"]),
            rows,
        })
    }

    /// R4: structured AI decision records only (no LLM prompt/response — confirmed #40).
    #[tracing::instrument(skip(pool))]
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
    /// full span (calendar-aware). There is no cached PD column — the dead
    /// `allocations.allocated_pd` (always 0) was dropped in migration 0004. effective_daily_rate =
    /// resource_project_rates (latest valid_from) → resources.daily_rate_pd → 0.
    #[tracing::instrument(skip(pool), fields(project_id = project_id))]
    async fn cost(pool: &SqlitePool, project_id: Option<i64>) -> Result<ReportTable, AppError> {
        let cal = db::repo::calendar::hydrate(pool).await?;
        // rows: resource_id, resource_name, daily_capacity_pd, project_id, project_name, start, end, percent
        let q = match project_id {
            Some(pid) => sqlx::query_as::<_, (i64, String, f64, i64, String, chrono::NaiveDate, chrono::NaiveDate, f64)>(
                "SELECT r.id, r.name, r.daily_capacity_pd, p.id, p.name, a.start_date, a.end_date, a.percent \
                 FROM allocations a JOIN resources r ON r.id=a.resource_id \
                 JOIN tasks t ON t.id=a.task_id JOIN projects p ON p.id=t.project_id \
                 WHERE p.id=? AND a.deleted_at IS NULL AND r.deleted_at IS NULL AND t.deleted_at IS NULL AND p.deleted_at IS NULL").bind(pid),
            None => sqlx::query_as::<_, (i64, String, f64, i64, String, chrono::NaiveDate, chrono::NaiveDate, f64)>(
                "SELECT r.id, r.name, r.daily_capacity_pd, p.id, p.name, a.start_date, a.end_date, a.percent \
                 FROM allocations a JOIN resources r ON r.id=a.resource_id \
                 JOIN tasks t ON t.id=a.task_id JOIN projects p ON p.id=t.project_id \
                 WHERE a.deleted_at IS NULL AND r.deleted_at IS NULL AND t.deleted_at IS NULL AND p.deleted_at IS NULL"),
        };
        let allocs = q.fetch_all(pool).await?;

        // Aggregate PD per (resource_id, project_id), then resolve rate per (resource, project).
        // BTreeMap (not HashMap) so exported rows are in a stable, reproducible order.
        use std::collections::BTreeMap;
        let mut pd: BTreeMap<(i64, i64), f64> = BTreeMap::new();
        let mut names: BTreeMap<(i64, i64), (String, String)> = BTreeMap::new();
        for (rid, rname, daily_capacity_pd, pid, pname, start, end, percent) in &allocs {
            let a = domain::Allocation {
                id: 0,
                resource_id: *rid,
                project_id: *pid,
                daily_capacity_pd: *daily_capacity_pd,
                start: *start,
                end: *end,
                percent: *percent,
            };
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
    #[tracing::instrument(skip(pool), fields(start = %start, end = %end))]
    pub async fn snapshot_json(pool: &SqlitePool, start: &str, end: &str) -> Result<String, AppError> {
        let cal = db::repo::calendar::hydrate(pool).await?;
        let mut entries = Vec::new();
        for r in db::ResourcesRepo::list_active(pool).await? {
            let s = crate::service::workload::WorkloadService::resource_summary_with_cal(pool, &cal, r.id, start, end).await?;
            entries.push(serde_json::json!({
                "resource": r.name, "capacity_pd": s.capacity_pd, "workload_pd": s.workload_pd,
                "utilization": s.utilization, "overloaded": s.overloaded,
            }));
        }
        serde_json::to_string_pretty(&serde_json::json!({
            "window": { "start": start, "end": end }, "resources": entries,
        })).map_err(|e| AppError::internal(e.to_string()))
    }

    /// Report catalog mapped to the design roadmap (design §8 / G5). Each entry lists the
    /// formats actually available (csv/xlsx always; pdf only when the `app/pdf` feature is
    /// compiled in) so the frontend can hide unavailable formats rather than fail on export.
    pub fn report_catalog() -> Vec<ReportCatalogEntry> {
        let csv_xlsx = vec!["csv".into(), "xlsx".into()];
        #[cfg(feature = "pdf")]
        let formats = { let mut f = csv_xlsx.clone(); f.push("pdf".into()); f };
        #[cfg(not(feature = "pdf"))]
        let formats = csv_xlsx;
        vec![
            ReportCatalogEntry {
                kind: "ResourceUtilization".into(), title: "资源利用率".into(),
                description: "按资源：容量/工作负载/利用率/过载 (MVP)".into(),
                formats: formats.clone(), accepts_project_id: false, mvp: true,
            },
            ReportCatalogEntry {
                kind: "TeamUtilization".into(), title: "团队利用率".into(),
                description: "按团队聚合：Σ工作负载/Σ容量 + 过载成员 (MVP)".into(),
                formats: formats.clone(), accepts_project_id: false, mvp: true,
            },
            ReportCatalogEntry {
                kind: "ProjectBurn".into(), title: "项目预算消耗".into(),
                description: "按项目：预算/已分配/消耗比 (MVP)".into(),
                formats: formats.clone(), accepts_project_id: false, mvp: true,
            },
            ReportCatalogEntry {
                kind: "Cost".into(), title: "成本".into(),
                description: "按资源×项目：PD×费率 = 成本 (MVP)".into(),
                formats: formats.clone(), accepts_project_id: true, mvp: true,
            },
            ReportCatalogEntry {
                kind: "AiDecisions".into(), title: "AI 决策记录".into(),
                description: "结构化 AI 运行记录 (MVP)".into(),
                formats, accepts_project_id: false, mvp: true,
            },
        ]
    }
}

#[tracing::instrument(skip(pool), fields(resource_id = resource_id, project_id = project_id))]
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

/// Format generators (CSV / Excel / PDF). Pure: ReportTable → bytes.
impl ReportService {
    pub fn to_csv(t: &ReportTable) -> Result<Vec<u8>, AppError> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        wtr.write_record(&t.columns).map_err(|e| AppError::internal(e.to_string()))?;
        for row in &t.rows {
            wtr.write_record(row).map_err(|e| AppError::internal(e.to_string()))?;
        }
        let mut bytes = wtr.into_inner().map_err(|e| AppError::internal(e.to_string()))?;
        // Prepend a UTF-8 BOM so Excel for Windows reads non-ASCII names correctly
        // (the csv crate writes valid UTF-8 but no BOM; Excel defaults to ANSI without one).
        let mut out = Vec::with_capacity(bytes.len() + 3);
        out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        out.append(&mut bytes);
        Ok(out)
    }

    pub fn to_xlsx(t: &ReportTable) -> Result<Vec<u8>, AppError> {
        use rust_xlsxwriter::{Workbook, XlsxError};
        let map = |e: XlsxError| AppError::internal(e.to_string());
        let mut wb = Workbook::new();
        let sheet = wb.add_worksheet().set_name(&t.title).map_err(map)?;
        for (c, col) in t.columns.iter().enumerate() {
            sheet.write_string(0, c as u16, col).map_err(map)?;
        }
        for (r, row) in t.rows.iter().enumerate() {
            for (c, val) in row.iter().enumerate() {
                sheet.write_string((r + 1) as u32, c as u16, val).map_err(map)?;
            }
        }
        wb.save_to_buffer().map_err(map)
    }
}

/// PDF generator via pure-Rust `printpdf` (no Chromium; HTML→PDF remains optional #41).
/// Feature-gated (`pdf`); simple paginated text table — multi-page when y drops below margin.
#[cfg(feature = "pdf")]
impl ReportService {
    pub fn to_pdf(t: &ReportTable) -> Result<Vec<u8>, AppError> {
        use printpdf::{BuiltinFont, Mm, PdfDocument};
        let (doc, page_idx, layer_idx) =
            PdfDocument::new("report", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| AppError::internal(e.to_string()))?;
        let mut y = 280.0_f32;
        let mut layer = doc.get_page(page_idx).get_layer(layer_idx);
        layer.use_text(&t.title, 14.0, Mm(15.0), Mm(y), &font);
        y -= 10.0;
        layer.use_text(&t.columns.join("   |   "), 9.0, Mm(15.0), Mm(y), &font);
        y -= 8.0;
        for row in &t.rows {
            if y < 20.0 {
                let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                layer = doc.get_page(p).get_layer(l);
                y = 280.0;
            }
            layer.use_text(&row.join("   |   "), 9.0, Mm(15.0), Mm(y), &font);
            y -= 8.0;
        }
        doc.save_to_bytes().map_err(|e| AppError::internal(e.to_string()))
    }
}
