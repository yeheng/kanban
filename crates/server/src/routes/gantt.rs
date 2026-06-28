use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use db::models::{DepEdge, GanttBar};
use db::{GanttRepo, TaskDepsRepo};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/gantt/projects/{id}", get(gantt_by_project))
        .route("/api/gantt/resources/{id}", get(gantt_by_resource))
        .route("/api/projects/{id}/dependencies", get(dependencies))
}

async fn gantt_by_project(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<GanttBar>>, HttpError> {
    Ok(Json(GanttRepo::by_project(&state.pool, project_id).await?))
}

async fn gantt_by_resource(
    State(state): State<AppState>,
    Path(resource_id): Path<i64>,
) -> Result<Json<Vec<GanttBar>>, HttpError> {
    Ok(Json(GanttRepo::by_resource(&state.pool, resource_id).await?))
}

async fn dependencies(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<DepEdge>>, HttpError> {
    Ok(Json(TaskDepsRepo::for_project(&state.pool, project_id).await?))
}
