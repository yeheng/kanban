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

#[tracing::instrument(skip(state))]
async fn list_skills(State(state): State<AppState>) -> Result<Json<Vec<db::models::Skill>>, HttpError> {
    tracing::debug!("listing skills");
    Ok(Json(app::service::catalog::CatalogService::list_skills(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct EnsureSkill { name: String }

#[tracing::instrument(skip(state), fields(name = %body.name))]
async fn ensure_skill(
    State(state): State<AppState>,
    Json(body): Json<EnsureSkill>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::catalog::CatalogService::ensure_skill(&state.pool, &body.name).await?;
    tracing::info!(skill_id = id, name = %body.name, "ensured skill");
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

#[tracing::instrument(skip(state))]
async fn list_tags(State(state): State<AppState>) -> Result<Json<Vec<db::models::Tag>>, HttpError> {
    tracing::debug!("listing tags");
    Ok(Json(app::service::catalog::CatalogService::list_tags(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct EnsureTag { name: String, color: Option<String> }

#[tracing::instrument(skip(state), fields(name = %body.name))]
async fn ensure_tag(
    State(state): State<AppState>,
    Json(body): Json<EnsureTag>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::catalog::CatalogService::ensure_tag(&state.pool, &body.name, body.color.as_deref()).await?;
    tracing::info!(tag_id = id, name = %body.name, "ensured tag");
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}
