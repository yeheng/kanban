use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/skills", get(list_skills).post(ensure_skill))
        .route("/api/tags", get(list_tags).post(ensure_tag))
}

async fn list_skills(State(state): State<AppState>) -> Result<Json<Vec<db::models::Skill>>, HttpError> {
    Ok(Json(app::service::catalog::CatalogService::list_skills(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct EnsureSkill { name: String }

async fn ensure_skill(
    State(state): State<AppState>,
    Json(body): Json<EnsureSkill>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::catalog::CatalogService::ensure_skill(&state.pool, &body.name).await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

async fn list_tags(State(state): State<AppState>) -> Result<Json<Vec<db::models::Tag>>, HttpError> {
    Ok(Json(app::service::catalog::CatalogService::list_tags(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct EnsureTag { name: String, color: Option<String> }

async fn ensure_tag(
    State(state): State<AppState>,
    Json(body): Json<EnsureTag>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::catalog::CatalogService::ensure_tag(&state.pool, &body.name, body.color.as_deref()).await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}
