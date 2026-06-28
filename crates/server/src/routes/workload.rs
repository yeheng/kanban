use crate::error::HttpError;
use crate::state::AppState;
use app::service::workload::{ProjectBurn, ResourceSummary, TeamSummary, WorkloadService};
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        // workload (read-only summaries; window passed as query params)
        .route("/api/workload/resources/{id}", get(resource_summary))
        .route("/api/workload/teams/{id}", get(team_summary))
        .route("/api/workload/overloads", get(overloads))
        .route("/api/projects/{id}/burn", get(project_burn))
        // Dashboard color bands (global thresholds)
        .route("/api/thresholds", get(get_thresholds))
}

/// Effective global thresholds for Dashboard color bands.
#[derive(Debug, Serialize)]
pub struct ThresholdsDto {
    pub overload: f64,
    pub underload: f64,
    pub green: f64,
    pub yellow: f64,
}

async fn get_thresholds(State(state): State<AppState>) -> Result<Json<ThresholdsDto>, HttpError> {
    let t = db::SettingsRepo::thresholds(&state.pool).await?;
    Ok(Json(ThresholdsDto { overload: t.overload, underload: t.underload, green: t.green, yellow: t.yellow }))
}

#[derive(Debug, Deserialize)]
struct WindowQuery { start: String, end: String }

async fn resource_summary(
    State(state): State<AppState>,
    Path(resource_id): Path<i64>,
    Query(q): Query<WindowQuery>,
) -> Result<Json<ResourceSummary>, HttpError> {
    Ok(Json(WorkloadService::resource_summary(&state.pool, resource_id, &q.start, &q.end).await?))
}

async fn team_summary(
    State(state): State<AppState>,
    Path(team_id): Path<i64>,
    Query(q): Query<WindowQuery>,
) -> Result<Json<TeamSummary>, HttpError> {
    Ok(Json(WorkloadService::team_summary(&state.pool, team_id, &q.start, &q.end).await?))
}

async fn overloads(
    State(state): State<AppState>,
    Query(q): Query<WindowQuery>,
) -> Result<Json<Vec<ResourceSummary>>, HttpError> {
    Ok(Json(WorkloadService::overloads(&state.pool, &q.start, &q.end).await?))
}

async fn project_burn(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<ProjectBurn>, HttpError> {
    Ok(Json(WorkloadService::project_burn(&state.pool, project_id).await?))
}
