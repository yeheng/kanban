use crate::error::HttpError;
use crate::state::AppState;
use app::service::optimization::{OptimizationService, RunList, RunResult};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post, patch};
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/optimization/run/{project_id}", post(run_optimization))
        .route("/api/optimization/runs", get(list_runs))
        .route("/api/optimization/runs/{id}", get(get_run))
        .route("/api/optimization/runs/{id}/apply", post(apply_solution))
        .route("/api/optimization/runs/{id}/reject", post(reject_solution))
        .route("/api/optimization/runs/{id}/suggestions", get(list_suggestions))
        .route("/api/optimization/runs/{id}/rerun", post(rerun_run))
        .route("/api/optimization/suggestions/{id}", patch(set_suggestion_status))
}

/// Optional multi-objective weights override (design §5; confirmed #6). Omitted body ⇒ balanced
/// default. Takes the flat ObjectiveWeights directly (no flatten wrapper) — matches what the
/// frontend sends and avoids a serde foot-gun on partial bodies.
#[tracing::instrument(skip(state), fields(project_id = project_id))]
async fn run_optimization(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    body: Option<Json<ai_engine::ObjectiveWeights>>,
) -> Result<Json<RunResult>, HttpError> {
    let weights = body.map(|Json(w)| w);
    let result = OptimizationService::run_for_project(&state.pool, project_id, weights).await?;
    tracing::info!(
        run_id = result.run_id,
        overall = result.plan.solution.metrics.overall,
        assignments = result.plan.solution.assignments.len(),
        unscheduled = result.plan.solution.unscheduled.len(),
        "optimization run completed"
    );
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
struct PageQuery { offset: Option<i64>, limit: Option<i64> }

#[tracing::instrument(skip(state))]
async fn list_runs(
    State(state): State<AppState>,
    Query(q): Query<PageQuery>,
) -> Result<Json<RunList>, HttpError> {
    let offset = q.offset.unwrap_or(0).max(0);
    let limit = q.limit.unwrap_or(10).max(1);
    tracing::debug!(offset = offset, limit = limit, "listing optimization runs");
    Ok(Json(OptimizationService::list_recent(&state.pool, offset, limit).await?))
}

#[tracing::instrument(skip(state), fields(run_id = run_id))]
async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<i64>,
) -> Result<Json<RunResult>, HttpError> {
    tracing::debug!(run_id = run_id, "fetching optimization run");
    Ok(Json(OptimizationService::get_run(&state.pool, run_id).await?))
}

#[tracing::instrument(skip(state), fields(run_id = run_id))]
async fn apply_solution(
    State(state): State<AppState>,
    Path(run_id): Path<i64>,
) -> Result<Json<i64>, HttpError> {
    let count = OptimizationService::apply(&state.pool, run_id).await?;
    tracing::info!(run_id = run_id, allocations_written = count, "applied optimization solution");
    Ok(Json(count))
}

#[tracing::instrument(skip(state), fields(run_id = run_id))]
async fn reject_solution(
    State(state): State<AppState>,
    Path(run_id): Path<i64>,
) -> Result<axum::http::StatusCode, HttpError> {
    OptimizationService::reject(&state.pool, run_id).await?;
    tracing::info!(run_id = run_id, "rejected optimization solution");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state), fields(run_id = run_id))]
async fn list_suggestions(
    State(state): State<AppState>,
    Path(run_id): Path<i64>,
) -> Result<Json<Vec<ai_engine::types::SuggestionItem>>, HttpError> {
    Ok(Json(OptimizationService::list_suggestions(&state.pool, run_id).await?))
}

#[derive(Debug, Deserialize)]
struct RerunBody { suggestion_ids: Vec<i64> }

#[tracing::instrument(skip(state), fields(run_id = run_id))]
async fn rerun_run(
    State(state): State<AppState>,
    Path(run_id): Path<i64>,
    Json(RerunBody { suggestion_ids }): Json<RerunBody>,
) -> Result<Json<RunResult>, HttpError> {
    Ok(Json(OptimizationService::rerun(&state.pool, run_id, suggestion_ids).await?))
}

#[derive(Debug, Deserialize)]
struct SuggestionStatusBody { status: String }

#[tracing::instrument(skip(state), fields(suggestion_id = suggestion_id))]
async fn set_suggestion_status(
    State(state): State<AppState>,
    Path(suggestion_id): Path<i64>,
    Json(SuggestionStatusBody { status }): Json<SuggestionStatusBody>,
) -> Result<axum::http::StatusCode, HttpError> {
    OptimizationService::set_suggestion_status(&state.pool, suggestion_id, &status).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
