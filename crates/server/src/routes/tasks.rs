use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use db::models::Task;
use db::TasksRepo;
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tasks", post(create_task))
        .route("/api/tasks/{id}/status", patch(set_status))
        .route("/api/projects/{id}/tasks", get(list_tasks))
}

async fn list_tasks(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<Task>>, HttpError> {
    Ok(Json(TasksRepo::list_by_project(&state.pool, project_id).await?))
}

#[derive(Debug, Deserialize)]
struct CreateTask {
    project_id: i64,
    title: String,
    description: Option<String>,
    estimate_pd: f64,
    start: Option<String>,
    end: Option<String>,
    is_long_term: Option<bool>,
    sort_order: Option<i64>,
    skill_reqs: Vec<(i64, i64, bool, f64)>,
    tag_ids: Vec<i64>,
}

async fn create_task(
    State(state): State<AppState>,
    Json(body): Json<CreateTask>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::tasks::TasksService::create(
        &state.pool,
        body.project_id,
        &body.title,
        body.description.as_deref(),
        body.estimate_pd,
        body.start.as_deref(),
        body.end.as_deref(),
        body.is_long_term.unwrap_or(false),
        body.sort_order.unwrap_or(0),
        &body.skill_reqs,
        &body.tag_ids,
    )
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

#[derive(Debug, Deserialize)]
struct SetStatus { status: String }

async fn set_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<SetStatus>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::tasks::TasksService::set_status(&state.pool, id, &body.status).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
