use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use db::models::AllocationView;
use db::AllocationsRepo;
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/allocations", post(create_allocation))
        .route("/api/allocations/{id}", axum::routing::put(update_allocation))
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

async fn create_allocation(
    State(state): State<AppState>,
    Json(body): Json<CreateAllocation>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    if !(body.percent > 0.0 && body.percent <= 1.0) {
        return Err(domain::DomainError::InvalidRatio(body.percent).into());
    }
    let id = AllocationsRepo::create(&state.pool, body.resource_id, body.task_id, &body.start, &body.end, body.percent).await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

async fn list_allocations(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<AllocationView>>, HttpError> {
    Ok(Json(AllocationsRepo::list_by_project(&state.pool, project_id).await?))
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
async fn update_allocation(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateAllocation>,
) -> Result<axum::http::StatusCode, HttpError> {
    if !(body.percent > 0.0 && body.percent <= 1.0) {
        return Err(domain::DomainError::InvalidRatio(body.percent).into());
    }
    if body.end.as_str() < body.start.as_str() {
        return Err(domain::DomainError::InvalidDateWindow.into());
    }
    AllocationsRepo::update(&state.pool, id, &body.start, &body.end, body.percent).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
