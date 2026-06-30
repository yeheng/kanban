use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use db::models::{Team, TeamMember, TeamOverride};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/teams", get(list_teams).post(create_team))
        .route("/api/teams/{id}", delete(delete_team))
        .route("/api/teams/{id}/members", get(list_team_members).post(add_member))
        .route("/api/teams/{id}/members/{resource_id}", delete(remove_member))
        .route("/api/teams/{id}/override", get(get_override))
        .route("/api/teams/overrides", put(set_override))
}

async fn list_teams(State(state): State<AppState>) -> Result<Json<Vec<Team>>, HttpError> {
    Ok(Json(app::service::teams::TeamsService::list(&state.pool).await?))
}

async fn list_team_members(
    State(state): State<AppState>,
    Path(team_id): Path<i64>,
) -> Result<Json<Vec<TeamMember>>, HttpError> {
    Ok(Json(app::service::teams::TeamsService::members(&state.pool, team_id).await?))
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

async fn remove_member(
    State(state): State<AppState>,
    Path((team_id, resource_id)): Path<(i64, i64)>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::teams::TeamsService::remove_member(&state.pool, team_id, resource_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn set_override(
    State(state): State<AppState>,
    Json(body): Json<TeamOverride>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::teams::TeamsService::set_override(&state.pool, body).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn get_override(
    State(state): State<AppState>,
    Path(team_id): Path<i64>,
) -> Result<Json<Option<TeamOverride>>, HttpError> {
    Ok(Json(app::service::teams::TeamsService::get_override(&state.pool, team_id).await?))
}

async fn delete_team(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, HttpError> {
    app::service::teams::TeamsService::soft_delete(&state.pool, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
