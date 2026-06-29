use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, patch};
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/resources", get(list_resources).post(create_resource))
        .route("/api/resources/{id}", delete(delete_resource).patch(update_resource))
}

async fn list_resources(State(state): State<AppState>) -> Result<Json<Vec<db::models::Resource>>, HttpError> {
    Ok(Json(app::service::resources::ResourcesService::list(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct CreateResource { name: String, email: Option<String> }

async fn create_resource(
    State(state): State<AppState>,
    Json(body): Json<CreateResource>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::resources::ResourcesService::create(&state.pool, &body.name, body.email.as_deref()).await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

async fn delete_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::resources::ResourcesService::soft_delete(&state.pool, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct UpdateResource {
    name: String,
    email: Option<String>,
}

async fn update_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateResource>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::resources::ResourcesService::update(
        &state.pool, id, &body.name, body.email.as_deref(),
    ).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
