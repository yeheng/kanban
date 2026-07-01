use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use db::models::{DepEdge, GanttBar};
use db::{GanttRepo, TaskDepsRepo};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/gantt/projects/{id}", get(gantt_by_project))
        .route("/api/gantt/resources/{id}", get(gantt_by_resource))
        .route("/api/projects/{id}/dependencies", get(dependencies))
        .route("/api/tasks/{id}/dependencies", post(add_dependency))
}

#[tracing::instrument(skip(state), fields(project_id = project_id))]
async fn gantt_by_project(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<GanttBar>>, HttpError> {
    tracing::debug!("gantt by project");
    Ok(Json(GanttRepo::by_project(&state.pool, project_id).await?))
}

#[tracing::instrument(skip(state), fields(resource_id = resource_id))]
async fn gantt_by_resource(
    State(state): State<AppState>,
    Path(resource_id): Path<i64>,
) -> Result<Json<Vec<GanttBar>>, HttpError> {
    tracing::debug!("gantt by resource");
    Ok(Json(GanttRepo::by_resource(&state.pool, resource_id).await?))
}

#[tracing::instrument(skip(state), fields(project_id = project_id))]
async fn dependencies(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<DepEdge>>, HttpError> {
    tracing::debug!("listing dependencies");
    Ok(Json(TaskDepsRepo::for_project(&state.pool, project_id).await?))
}

#[derive(Debug, Deserialize)]
struct AddDependency {
    predecessor_id: i64,
    lag_days: Option<i64>,
    dep_type: Option<String>,
}

#[tracing::instrument(skip(state), fields(task_id = task_id, predecessor_id = body.predecessor_id))]
async fn add_dependency(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    Json(body): Json<AddDependency>,
) -> Result<axum::http::StatusCode, HttpError> {
    // Default to finish-to-start; the service validates/normalizes to the FS/FF/SS/SF codes.
    let dep_type = body.dep_type.as_deref().unwrap_or("FS");
    app::service::tasks::TasksService::add_dependency(
        &state.pool, task_id, body.predecessor_id, body.lag_days.unwrap_or(0), dep_type,
    ).await?;
    tracing::info!(task_id = task_id, predecessor_id = body.predecessor_id, dep_type, "added dependency");
    Ok(axum::http::StatusCode::CREATED)
}
