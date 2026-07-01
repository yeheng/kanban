use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/resources", get(list_resources).post(create_resource))
        .route("/api/resources/{id}", delete(delete_resource).patch(update_resource))
        .route(
            "/api/resources/{id}/skills",
            get(list_resource_skills).put(set_resource_skills),
        )
        .route(
            "/api/resources/{id}/tags",
            get(list_resource_tags).put(set_resource_tags),
        )
}

#[tracing::instrument(skip(state))]
async fn list_resources(State(state): State<AppState>) -> Result<Json<Vec<db::models::Resource>>, HttpError> {
    tracing::debug!("listing resources");
    Ok(Json(app::service::resources::ResourcesService::list(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct CreateResource { name: String, email: Option<String> }

#[tracing::instrument(skip(state), fields(name = %body.name))]
async fn create_resource(
    State(state): State<AppState>,
    Json(body): Json<CreateResource>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::resources::ResourcesService::create(&state.pool, &body.name, body.email.as_deref()).await?;
    tracing::info!(resource_id = id, "created resource");
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

#[tracing::instrument(skip(state), fields(resource_id = id))]
async fn delete_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::resources::ResourcesService::soft_delete(&state.pool, id).await?;
    tracing::info!(resource_id = id, "deleted resource");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct UpdateResource {
    name: String,
    email: Option<String>,
    available_from: Option<String>,
    available_to: Option<String>,
    daily_capacity_pd: Option<f64>,
    daily_rate_pd: Option<f64>,
}

#[tracing::instrument(skip(state), fields(resource_id = id, name = %body.name))]
async fn update_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateResource>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::resources::ResourcesService::update(
        &state.pool, id, &body.name, body.email.as_deref(),
        body.available_from.as_deref(), body.available_to.as_deref(),
        body.daily_capacity_pd, body.daily_rate_pd,
    ).await?;
    tracing::info!(resource_id = id, "updated resource");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state), fields(resource_id = id))]
async fn list_resource_skills(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<db::models::ResourceSkill>>, HttpError> {
    tracing::debug!("listing resource skills");
    Ok(Json(app::service::resources::ResourcesService::list_skills(&state.pool, id).await?))
}

#[derive(Debug, Deserialize)]
struct SetResourceSkills { skills: Vec<(i64, i64)> } // (skill_id, proficiency)

#[tracing::instrument(skip(state), fields(resource_id = id, skill_count = body.skills.len()))]
async fn set_resource_skills(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<SetResourceSkills>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::resources::ResourcesService::set_skills(&state.pool, id, &body.skills).await?;
    tracing::info!(resource_id = id, skill_count = body.skills.len(), "set resource skills");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state), fields(resource_id = id))]
async fn list_resource_tags(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<db::models::ResourceTag>>, HttpError> {
    tracing::debug!("listing resource tags");
    Ok(Json(app::service::resources::ResourcesService::list_tags(&state.pool, id).await?))
}

#[derive(Debug, Deserialize)]
struct SetResourceTags { tag_ids: Vec<i64> }

#[tracing::instrument(skip(state), fields(resource_id = id, tag_count = body.tag_ids.len()))]
async fn set_resource_tags(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<SetResourceTags>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::resources::ResourcesService::set_tags(&state.pool, id, &body.tag_ids).await?;
    tracing::info!(resource_id = id, tag_count = body.tag_ids.len(), "set resource tags");
    Ok(axum::http::StatusCode::NO_CONTENT)
}
