use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{post, put};
use axum::{Json, Router};
use db::models::TeamOverride;
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/teams", post(create_team))
        .route("/api/teams/{id}/members", post(add_member))
        .route("/api/teams/overrides", put(set_override))
}

#[derive(Debug, Deserialize)]
struct CreateTeam { name: String, description: Option<String> }

async fn create_team(
    State(state): State<AppState>,
    Json(body): Json<CreateTeam>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = app::service::teams::TeamsService::create(&state.pool, &body.name, body.description.as_deref()).await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

#[derive(Debug, Deserialize)]
struct AddMember { resource_id: i64, role: Option<String> }

async fn add_member(
    State(state): State<AppState>,
    Path(team_id): Path<i64>,
    Json(body): Json<AddMember>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::teams::TeamsService::add_member(&state.pool, team_id, body.resource_id, body.role.as_deref()).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn set_override(
    State(state): State<AppState>,
    Json(body): Json<TeamOverride>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::teams::TeamsService::set_override(&state.pool, body).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
