use crate::error::AppError;
use ai_engine::explainer::TemplateExplainer;
use ai_engine::scorer::FallbackScorer;
use ai_engine::solver::GreedySolver;
use ai_engine::types::*;
use ai_engine::OptimizationEngine;
use chrono::NaiveDate;
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
    pub async fn run_for_project(pool: &SqlitePool, project_id: i64) -> Result<RunResult, AppError> {
        let problem = build_problem(pool, project_id).await?;

        // Default offline pipeline; swap in SemanticScorer/MilpSolver/LlmExplainer when Ollama is up.
        let engine = OptimizationEngine {
            scorer: Arc::new(FallbackScorer),
            solver: Arc::new(GreedySolver),
            explainer: Arc::new(TemplateExplainer),
        };
        let started = chrono::Utc::now();
        let plan = engine.optimize(&problem).await;
        let finished = chrono::Utc::now();
        let duration_ms = (finished - started).num_milliseconds();

        let cfg = serde_json::to_string(&problem.config).unwrap_or_default();
        let cons = serde_json::to_string(&problem.flags).unwrap_or_default();
        let wts = serde_json::to_string(&problem.weights).unwrap_or_default();
        let snap = serde_json::to_string(&problem).unwrap_or_default();
        let out = serde_json::to_string(&plan.solution.assignments).unwrap_or_default();

        let (run_id,): (i64,) = sqlx::query_as(
            "INSERT INTO ai_optimization_runs (seed, scope, scope_project_ids, config_json, constraints_json, \
                weights_json, input_snapshot_json, output_plan_json, score_overall, score_skill_fit, \
                score_utilization, score_fairness, explanation_md, provider, chat_model, embed_model, \
                solver_backend, solver_status, status, started_at, finished_at, duration_ms) \
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?, 'ollama','qwen2.5:7b','nomic-embed-text', 'greedy','feasible','proposed', ?,?,?) RETURNING id")
            .bind(problem.config.seed as i64).bind("incremental").bind(format!("[{}]", project_id))
            .bind(cfg).bind(cons).bind(wts).bind(snap).bind(out)
            .bind(plan.solution.metrics.overall).bind(plan.solution.metrics.skill_fit)
            .bind(plan.solution.metrics.utilization).bind(plan.solution.metrics.fairness)
            .bind(&plan.explanation_md)
            .bind(started.to_rfc3339()).bind(finished.to_rfc3339()).bind(duration_ms)
            .fetch_one(pool).await?;

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
    // resources + skills
    let resources: Vec<(i64, String, f64)> = sqlx::query_as(
        "SELECT id, name, daily_capacity_pd FROM resources WHERE deleted_at IS NULL AND status='active'")
        .fetch_all(pool).await?;
    let mut cand = Vec::new();
    for (id, name, cap) in resources {
        let skills: HashMap<i64, i64> = sqlx::query_as::<_, (i64, i64)>(
            "SELECT skill_id, proficiency FROM resource_skills WHERE resource_id=?")
            .bind(id).fetch_all(pool).await?
            .into_iter().collect();
        cand.push(CandidateResource { id, name, skills, tags: vec![], daily_capacity_pd: cap });
    }
    // tasks + skill reqs for the project (todo/in_progress only). Priority lives on the
    // project (design §3.3.3), not per-task, so join projects.priority.
    type TaskRow = (i64, String, f64, Option<NaiveDate>, Option<NaiveDate>, i64);
    let tasks: Vec<TaskRow> = sqlx::query_as(
        "SELECT t.id, t.title, t.estimate_pd, t.start_date, t.end_date, p.priority \
         FROM tasks t JOIN projects p ON p.id = t.project_id \
         WHERE t.project_id=? AND t.deleted_at IS NULL AND t.status IN ('todo','in_progress')")
        .bind(project_id).fetch_all(pool).await?;
    let mut cand_tasks = Vec::new();
    for (id, title, est, start, end, pri) in tasks {
        let reqs: Vec<SkillReq> = sqlx::query_as::<_, (i64, i64, i64, f64)>(
            "SELECT skill_id, min_proficiency, is_mandatory, weight FROM task_skill_requirements WHERE task_id=?")
            .bind(id).fetch_all(pool).await?
            .into_iter().map(|(s, p, m, w)| SkillReq { skill_id: s, min_proficiency: p, is_mandatory: m != 0, weight: w }).collect();
        cand_tasks.push(CandidateTask {
            id, project_id, title, estimate_pd: est,
            start: start.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            end: end.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
            priority: pri, skill_reqs: reqs,
        });
    }

    let (run_id,): (i64,) = sqlx::query_as("SELECT COALESCE(MAX(id),0)+1 FROM ai_optimization_runs")
        .fetch_one(pool).await?;
    Ok(AllocationProblem {
        run_id, resources: cand, tasks: cand_tasks, existing: vec![],
        weights: ObjectiveWeights::default(), flags: ConstraintFlags::default(),
        config: SolverConfig::default(),
    })
}
