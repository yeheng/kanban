use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, patch};
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/projects", get(list_projects).post(create_project))
        .route("/api/projects/{id}", delete(delete_project).patch(update_project))
        .route("/api/projects/{id}/status", patch(set_project_status))
        .route("/api/projects/{id}/kanban", get(kanban_tasks))
}

#[tracing::instrument(skip(state))]
async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<db::models::Project>>, HttpError> {
    tracing::debug!("listing projects");
    Ok(Json(app::service::projects::ProjectsService::list(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct CreateProject {
    name: String,
    description: Option<String>,
    start: Option<String>,
    end: Option<String>,
    priority: i64,
    budget_pd: Option<f64>,
}

#[tracing::instrument(skip(state), fields(name = %body.name, priority = body.priority))]
async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProject>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::projects::ProjectsService::create(
        &state.pool,
        &body.name,
        body.description.as_deref(),
        body.start.as_deref(),
        body.end.as_deref(),
        body.priority,
        body.budget_pd.unwrap_or(0.0),
    )
    .await?;
    tracing::info!(project_id = id, "created project");
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

#[tracing::instrument(skip(state), fields(project_id = id))]
async fn kanban_tasks(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<db::models::KanbanTask>>, HttpError> {
    tracing::debug!("kanban tasks");
    Ok(Json(app::service::tasks::TasksService::kanban(&state.pool, id).await?))
}

#[tracing::instrument(skip(state), fields(project_id = id))]
async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::projects::ProjectsService::soft_delete(&state.pool, id).await?;
    tracing::info!(project_id = id, "deleted project");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct UpdateProject {
    name: String,
    description: Option<String>,
    start: Option<String>,
    end: Option<String>,
    priority: i64,
    budget_pd: f64,
}

#[tracing::instrument(skip(state), fields(project_id = id, name = %body.name, priority = body.priority))]
async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateProject>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::projects::ProjectsService::update(
        &state.pool, id, &body.name, body.description.as_deref(),
        body.start.as_deref(), body.end.as_deref(), body.priority, body.budget_pd,
    ).await?;
    tracing::info!(project_id = id, "updated project");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct SetProjectStatus {
    status: String,
}

#[tracing::instrument(skip(state), fields(project_id = id, status = %body.status))]
async fn set_project_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<SetProjectStatus>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::projects::ProjectsService::set_status(&state.pool, id, &body.status).await?;
    tracing::info!(project_id = id, status = %body.status, "set project status");
    Ok(axum::http::StatusCode::NO_CONTENT)
}
