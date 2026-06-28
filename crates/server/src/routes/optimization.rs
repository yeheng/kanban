use crate::error::HttpError;
use crate::state::AppState;
use app::service::optimization::{OptimizationService, RunResult, RunRow};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/optimization/run/{project_id}", post(run_optimization))
        .route("/api/optimization/runs", get(list_runs))
        .route("/api/optimization/runs/{id}/apply", post(apply_solution))
        .route("/api/optimization/runs/{id}/reject", post(reject_solution))
}

/// Optional multi-objective weights override (design §5; confirmed #6). Omitted body ⇒ balanced
/// default. Takes the flat ObjectiveWeights directly (no flatten wrapper) — matches what the
/// frontend sends and avoids a serde foot-gun on partial bodies.
async fn run_optimization(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    body: Option<Json<ai_engine::ObjectiveWeights>>,
) -> Result<Json<RunResult>, HttpError> {
    let weights = body.map(|Json(w)| w);
    Ok(Json(OptimizationService::run_for_project(&state.pool, project_id, weights).await?))
}

#[derive(Debug, Deserialize)]
struct LimitQuery { limit: Option<i64> }

async fn list_runs(
    State(state): State<AppState>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<RunRow>>, HttpError> {
    Ok(Json(OptimizationService::list_recent(&state.pool, q.limit.unwrap_or(20)).await?))
}

async fn apply_solution(
    State(state): State<AppState>,
    Path(run_id): Path<i64>,
) -> Result<Json<i64>, HttpError> {
    Ok(Json(OptimizationService::apply(&state.pool, run_id).await?))
}

async fn reject_solution(
    State(state): State<AppState>,
    Path(run_id): Path<i64>,
) -> Result<axum::http::StatusCode, HttpError> {
    OptimizationService::reject(&state.pool, run_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
