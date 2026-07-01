use crate::error::HttpError;
use crate::state::AppState;
use app::service::occupancy::CalendarOccupancyService;
use app::service::workload::{ProjectBurn, ResourceSummary, TeamSummary, WorkloadService};
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use db::models::DayOccupancy;
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        // workload (read-only summaries; window passed as query params)
        .route("/api/workload/resources/{id}", get(resource_summary))
        .route("/api/workload/teams/{id}", get(team_summary))
        .route("/api/workload/overloads", get(overloads))
        .route("/api/projects/{id}/burn", get(project_burn))
        // calendar daily occupancy grid
        .route("/api/occupancy", get(daily_occupancy))
        // Dashboard color bands (global thresholds)
        .route("/api/thresholds", get(get_thresholds))
        // Global PD/PM unit constants (design §2.9; used by frontend PM display)
        .route("/api/config/units", get(get_unit_config))
}

/// Effective global thresholds for Dashboard color bands.
#[derive(Debug, Serialize)]
pub struct ThresholdsDto {
    pub overload: f64,
    pub underload: f64,
    pub green: f64,
    pub yellow: f64,
}

#[tracing::instrument(skip(state))]
async fn get_thresholds(State(state): State<AppState>) -> Result<Json<ThresholdsDto>, HttpError> {
    tracing::debug!("getting thresholds");
    let t = db::SettingsRepo::thresholds(&state.pool).await?;
    Ok(Json(ThresholdsDto { overload: t.overload, underload: t.underload, green: t.green, yellow: t.yellow }))
}

/// Global PD/PM constants for frontend PM display (design §2.9). Per-team overrides are
/// applied client-side via the team override API.
#[derive(Debug, Serialize)]
pub struct UnitConfigDto {
    pub pd_hours: f64,
    pub pm_workdays: f64,
}

#[tracing::instrument(skip(state))]
async fn get_unit_config(State(state): State<AppState>) -> Result<Json<UnitConfigDto>, HttpError> {
    tracing::debug!("getting unit config");
    let u = app::service::thresholds::global_unit_config(&state.pool).await?;
    Ok(Json(UnitConfigDto { pd_hours: u.hours_per_pd, pm_workdays: u.pd_per_pm }))
}

#[derive(Debug, Deserialize)]
struct WindowQuery { start: String, end: String }

#[tracing::instrument(skip(state), fields(resource_id = resource_id))]
async fn resource_summary(
    State(state): State<AppState>,
    Path(resource_id): Path<i64>,
    Query(q): Query<WindowQuery>,
) -> Result<Json<ResourceSummary>, HttpError> {
    tracing::debug!("resource summary");
    Ok(Json(WorkloadService::resource_summary(&state.pool, resource_id, &q.start, &q.end).await?))
}

#[tracing::instrument(skip(state), fields(team_id = team_id))]
async fn team_summary(
    State(state): State<AppState>,
    Path(team_id): Path<i64>,
    Query(q): Query<WindowQuery>,
) -> Result<Json<TeamSummary>, HttpError> {
    tracing::debug!("team summary");
    Ok(Json(WorkloadService::team_summary(&state.pool, team_id, &q.start, &q.end).await?))
}

#[tracing::instrument(skip(state))]
async fn overloads(
    State(state): State<AppState>,
    Query(q): Query<WindowQuery>,
) -> Result<Json<Vec<ResourceSummary>>, HttpError> {
    tracing::debug!("listing overloads");
    Ok(Json(WorkloadService::overloads(&state.pool, &q.start, &q.end).await?))
}

#[tracing::instrument(skip(state), fields(project_id = project_id))]
async fn project_burn(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<ProjectBurn>, HttpError> {
    tracing::debug!("project burn");
    Ok(Json(WorkloadService::project_burn(&state.pool, project_id).await?))
}

#[tracing::instrument(skip(state))]
async fn daily_occupancy(
    State(state): State<AppState>,
    Query(q): Query<WindowQuery>,
) -> Result<Json<Vec<DayOccupancy>>, HttpError> {
    tracing::debug!("daily occupancy");
    Ok(Json(CalendarOccupancyService::range(&state.pool, &q.start, &q.end).await?))
}
