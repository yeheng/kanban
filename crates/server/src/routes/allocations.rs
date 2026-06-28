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
