use crate::error::AppError;
use ai_engine::explainer::TemplateExplainer;
use ai_engine::scorer::FallbackScorer;
use ai_engine::solver::GreedySolver;
use ai_engine::types::*;
use ai_engine::OptimizationEngine;
use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct RunResult {
    pub run_id: i64,
    pub plan: ai_engine::OptimizedPlan,
}

pub struct OptimizationService;

impl OptimizationService {
    /// Build the problem from DB (all active resources + a project's todo/in_progress tasks),
    /// run the engine, persist a reproducible run (status='proposed'), return the plan.
    /// `weights` overrides the default balanced objective (design §5; confirmed #6 UI-tunable).
    pub async fn run_for_project(
        pool: &SqlitePool, project_id: i64, weights: Option<ObjectiveWeights>,
    ) -> Result<RunResult, AppError> {
        let mut problem = build_problem(pool, project_id).await?;
        if let Some(w) = weights { problem.weights = w; }

        // Default offline pipeline; swap in SemanticScorer/MilpSolver/LlmExplainer when Ollama is up.
        // Scorer weights mirror the problem's objective weights: jaccard weight = skill_fit,
        // proficiency weight = balance (normalized so they sum to 1).
        let total = (problem.weights.skill_fit + problem.weights.balance).max(0.001);
        let engine = OptimizationEngine {
            scorer: Arc::new(FallbackScorer {
                w_jaccard: problem.weights.skill_fit / total,
                w_proficiency: problem.weights.balance / total,
            }),
            solver: Arc::new(GreedySolver),
            explainer: Arc::new(TemplateExplainer),
        };
        let started = chrono::Utc::now();
        let mut plan = engine.optimize(&problem).await;
        let finished = chrono::Utc::now();
        let duration_ms = (finished - started).num_milliseconds();

        let cfg = serde_json::to_string(&problem.config).unwrap_or_default();
        let cons = serde_json::to_string(&problem.flags).unwrap_or_default();
        let wts = serde_json::to_string(&problem.weights).unwrap_or_default();

        // Insert with empty JSON snapshots to get the autoincrement id first (avoids the
        // MAX(id)+1 race — two concurrent runs could both predict the same run_id, baking a
        // stale value into the persisted JSON). scope='full' (this is a full project re-plan;
        // 'incremental' is reserved for when the solver respects existing allocations).
        let (run_id,): (i64,) = sqlx::query_as(
            "INSERT INTO ai_optimization_runs (seed, scope, scope_project_ids, config_json, constraints_json, \
                weights_json, input_snapshot_json, output_plan_json, score_overall, score_skill_fit, \
                score_utilization, score_fairness, explanation_md, provider, chat_model, embed_model, \
                solver_backend, solver_status, status, started_at, finished_at, duration_ms) \
             VALUES (?,?,?,?,?,?, '','',?,?,?,?,?, 'ollama','qwen2.5:7b','nomic-embed-text', 'greedy','feasible','proposed', ?,?,?) RETURNING id")
            .bind(problem.config.seed as i64).bind("full").bind(format!("[{}]", project_id))
            .bind(cfg).bind(cons).bind(wts)
            .bind(plan.solution.metrics.overall).bind(plan.solution.metrics.skill_fit)
            .bind(plan.solution.metrics.utilization).bind(plan.solution.metrics.fairness)
            .bind(&plan.explanation_md)
            .bind(started.to_rfc3339()).bind(finished.to_rfc3339()).bind(duration_ms)
            .fetch_one(pool).await?;

        // Now stamp the real run_id into the problem + solution, serialize, and backfill the
        // snapshot/plan JSON so the persisted row is self-consistent for reproducibility.
        problem.run_id = run_id;
        plan.solution.run_id = run_id;
        let snap = serde_json::to_string(&problem).unwrap_or_default();
        let out = serde_json::to_string(&plan.solution.assignments).unwrap_or_default();
        sqlx::query("UPDATE ai_optimization_runs SET input_snapshot_json=?, output_plan_json=? WHERE id=?")
            .bind(snap).bind(out).bind(run_id)
            .execute(pool).await?;

        Ok(RunResult { run_id, plan })
    }

    /// Accept a proposed run: write its assignments as allocations (source='ai', run_id) and
    /// mark applied. The trg_allocation_validate_insert trigger enforces task/resource windows;
    /// an out-of-window proposal ABORTs the tx (surfaced as AppError).
    pub async fn apply(pool: &SqlitePool, run_id: i64) -> Result<i64, AppError> {
        // Fetch the plan JSON up front to confirm the run exists + is unapplied; carry only the
        // (cheap, Clone) String + count into the tx closure so the FnMut retry is move-safe.
        let (plan_json,): (Option<String>,) = sqlx::query_as(
            "SELECT output_plan_json FROM ai_optimization_runs WHERE id=? AND applied=0")
            .bind(run_id).fetch_optional(pool).await?
            .ok_or_else(|| domain::DomainError::NotFound(format!("run {}", run_id)))?;
        let plan_json = plan_json.unwrap_or_default();
        let plan_json: &str = &plan_json; // shared borrow, cheap to capture across retries
        let count = serde_json::from_str::<Vec<ai_engine::ScoredAssignment>>(plan_json)
            .map(|v| v.len() as i64).unwrap_or(0);

        db::tx::with_write_tx(pool, |mut tx| {
            Box::pin(async move {
                // plan_json is a shared borrow captured by reference; re-deserialize each retry
                // so no owned Vec is moved across FnMut invocations.
                let assignments: Vec<ai_engine::ScoredAssignment> =
                    serde_json::from_str(plan_json).unwrap_or_default();
                for a in &assignments {
                    sqlx::query(
                        "INSERT INTO allocations (resource_id, task_id, start_date, end_date, percent, source, run_id) \
                         VALUES (?,?,?,?,?,?,?)")
                        .bind(a.resource_id).bind(a.task_id).bind(a.start).bind(a.end)
                        .bind(a.percent).bind("ai").bind(run_id)
                        .execute(&mut *tx).await?;
                }
                sqlx::query("UPDATE ai_optimization_runs SET applied=1, status='accepted' WHERE id=?")
                    .bind(run_id).execute(&mut *tx).await?;
                Ok((tx, ()))
            })
        }).await?;
        Ok(count)
    }

    pub async fn reject(pool: &SqlitePool, run_id: i64) -> Result<(), AppError> {
        sqlx::query("UPDATE ai_optimization_runs SET status='rejected' WHERE id=?")
            .bind(run_id).execute(pool).await?;
        Ok(())
    }

    pub async fn list_recent(pool: &SqlitePool, limit: i64) -> Result<Vec<RunRow>, AppError> {
        Ok(sqlx::query_as::<_, RunRow>(
            "SELECT id, objective, status, applied, score_overall, created_at FROM ai_optimization_runs \
             ORDER BY created_at DESC LIMIT ?")
            .bind(limit).fetch_all(pool).await?)
    }
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct RunRow {
    pub id: i64,
    pub objective: String,
    pub status: String,
    pub applied: i64,
    pub score_overall: Option<f64>,
    pub created_at: String,
}

async fn build_problem(pool: &SqlitePool, project_id: i64) -> Result<AllocationProblem, AppError> {
    use std::collections::HashMap;
    let (budget_pd,): (f64,) =
        sqlx::query_as("SELECT budget_pd FROM projects WHERE id=? AND deleted_at IS NULL")
            .bind(project_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| domain::DomainError::NotFound(format!("project {}", project_id)))?;
    // resources + skills + availability window (gated in the solver to mirror the trigger).
    type ResRow = (i64, String, f64, Option<NaiveDate>, Option<NaiveDate>);
    let resources: Vec<ResRow> = sqlx::query_as(
        "SELECT id, name, daily_capacity_pd, available_from, available_to \
         FROM resources WHERE deleted_at IS NULL AND status='active'")
        .fetch_all(pool).await?;

    // Batch-load all resource skills (1 query instead of N).
    let skill_rows: Vec<(i64, i64, i64)> = sqlx::query_as(
        "SELECT resource_id, skill_id, proficiency FROM resource_skills")
        .fetch_all(pool).await?;
    let mut skills_by_res: HashMap<i64, HashMap<i64, i64>> = HashMap::new();
    for (rid, sid, prof) in skill_rows {
        skills_by_res.entry(rid).or_default().insert(sid, prof);
    }

    let mut cand = Vec::new();
    for (id, name, cap, avail_from, avail_to) in resources {
        let skills = skills_by_res.remove(&id).unwrap_or_default();
        cand.push(CandidateResource {
            id, name, skills, tags: vec![], daily_capacity_pd: cap,
            available_from: avail_from, available_to: avail_to,
        });
    }
    // tasks + skill reqs for the project (todo/in_progress and not already allocated).
    // Priority lives on the project (design §3.3.3), not per-task, so join projects.priority.
    type TaskRow = (i64, String, f64, Option<NaiveDate>, Option<NaiveDate>, i64);
    let tasks: Vec<TaskRow> = sqlx::query_as(
        "SELECT t.id, t.title, t.estimate_pd, t.start_date, t.end_date, p.priority \
         FROM tasks t JOIN projects p ON p.id = t.project_id \
         WHERE t.project_id=? AND t.deleted_at IS NULL AND t.status IN ('todo','in_progress') \
           AND NOT EXISTS ( \
             SELECT 1 FROM allocations a \
             WHERE a.task_id=t.id AND a.deleted_at IS NULL AND a.status <> 'cancelled' \
           )")
        .bind(project_id).fetch_all(pool).await?;

    // Batch-load all task skill requirements for this project's tasks (1 query instead of M).
    let task_ids: Vec<i64> = tasks.iter().map(|t| t.0).collect();
    let req_rows: Vec<(i64, i64, i64, i64, f64)> = if task_ids.is_empty() {
        Vec::new()
    } else {
        // Build a parameterized IN clause dynamically. sqlx supports binding a slice/vec
        // only for specific drivers; for SQLite we build the placeholders string and bind
        // each id individually via a raw query.
        let placeholders: String = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT task_id, skill_id, min_proficiency, is_mandatory, weight \
             FROM task_skill_requirements WHERE task_id IN ({})", placeholders);
        let mut query = sqlx::query_as::<_, (i64, i64, i64, i64, f64)>(&sql);
        for id in &task_ids {
            query = query.bind(id);
        }
        query.fetch_all(pool).await?
    };
    let mut reqs_by_task: HashMap<i64, Vec<SkillReq>> = HashMap::new();
    for (tid, sid, prof, mandatory, weight) in req_rows {
        reqs_by_task.entry(tid).or_default().push(SkillReq {
            skill_id: sid, min_proficiency: prof, is_mandatory: mandatory != 0, weight,
        });
    }

    let mut cand_tasks = Vec::new();
    for (id, title, est, start, end, pri) in tasks {
        let reqs = reqs_by_task.remove(&id).unwrap_or_default();
        cand_tasks.push(CandidateTask {
            id, project_id, title, estimate_pd: est,
            start: start.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            end: end.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
            priority: pri, skill_reqs: reqs,
        });
    }

    let (existing, daily_capacity) =
        if let (Some(h_start), Some(h_end)) = (horizon_start(&cand_tasks), horizon_end(&cand_tasks)) {
            let rows: Vec<(i64, NaiveDate, NaiveDate, f64)> = sqlx::query_as(
                "SELECT a.resource_id, a.start_date, a.end_date, a.percent \
                 FROM allocations a \
                 JOIN tasks t ON t.id=a.task_id AND t.deleted_at IS NULL \
                 WHERE a.deleted_at IS NULL AND a.status <> 'cancelled' \
                   AND a.start_date <= ? AND a.end_date >= ?",
            )
            .bind(h_end)
            .bind(h_start)
            .fetch_all(pool)
            .await?;
            let existing = rows
                .into_iter()
                .map(|(resource_id, start, end, percent)| ExistingAlloc {
                    resource_id,
                    start,
                    end,
                    percent,
                })
                .collect();

            let cal = db::repo::calendar::hydrate(pool).await?;
            let mut caps = Vec::new();
            for r in &cand {
                let mut day = h_start;
                while day <= h_end {
                    caps.push(DailyCapacity {
                        resource_id: r.id,
                        day,
                        factor: cal.day_factor(project_id, r.id, day),
                    });
                    day = day.checked_add_days(Days::new(1)).unwrap();
                }
            }
            (existing, caps)
        } else {
            (Vec::new(), Vec::new())
        };

    // Placeholder run_id (0) — the real run_id is assigned by INSERT RETURNING id in
    // run_for_project and stamped back into the problem struct before snapshot serialization.
    Ok(AllocationProblem {
        run_id: 0, resources: cand, tasks: cand_tasks, existing, daily_capacity,
        budget_pd: Some(budget_pd),
        weights: ObjectiveWeights::default(), flags: ConstraintFlags::default(),
        config: SolverConfig::default(),
    })
}

fn horizon_start(tasks: &[CandidateTask]) -> Option<NaiveDate> {
    tasks.iter().map(|t| t.start).min()
}

fn horizon_end(tasks: &[CandidateTask]) -> Option<NaiveDate> {
    tasks.iter().map(|t| t.end).max()
}
