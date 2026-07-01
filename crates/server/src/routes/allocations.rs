use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use db::models::AllocationView;
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/allocations", post(create_allocation))
        .route("/api/allocations/{id}", axum::routing::put(update_allocation).delete(delete_allocation))
        .route("/api/projects/{id}/allocations", get(list_allocations))
}

#[derive(Debug, Deserialize)]
struct CreateAllocation {
    resource_id: i64,
    task_id: i64,
    start: String,
    end: String,
    percent: f64,
}

#[tracing::instrument(skip(state), fields(resource_id = body.resource_id, task_id = body.task_id))]
async fn create_allocation(
    State(state): State<AppState>,
    Json(body): Json<CreateAllocation>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::allocations::AllocationsService::create(
        &state.pool,
        body.resource_id,
        body.task_id,
        &body.start,
        &body.end,
        body.percent,
    )
    .await?;
    tracing::info!(allocation_id = id, resource_id = body.resource_id, task_id = body.task_id, "created allocation");
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

#[tracing::instrument(skip(state), fields(project_id = project_id))]
async fn list_allocations(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<AllocationView>>, HttpError> {
    tracing::debug!("listing allocations");
    Ok(Json(db::AllocationsRepo::list_by_project(&state.pool, project_id).await?))
}

#[derive(Debug, Deserialize)]
struct UpdateAllocation {
    start: String,
    end: String,
    percent: f64,
}

/// Update an allocation's window/percent (Gantt drag move/resize).
/// `start`/`end` are ISO `YYYY-MM-DD`; lexicographic order == chronological for that
/// format, so the window check is a plain string compare. The DB trigger additionally
/// enforces the task/resource window intersection.
#[tracing::instrument(skip(state), fields(allocation_id = id))]
async fn update_allocation(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateAllocation>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::allocations::AllocationsService::update(
        &state.pool,
        id,
        &body.start,
        &body.end,
        body.percent,
    )
    .await?;
    tracing::info!(allocation_id = id, "updated allocation");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state), fields(allocation_id = id))]
async fn delete_allocation(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::allocations::AllocationsService::soft_delete(&state.pool, id).await?;
    tracing::info!(allocation_id = id, "deleted allocation");
    Ok(axum::http::StatusCode::NO_CONTENT)
}
