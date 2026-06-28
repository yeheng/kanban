use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/resources", get(list_resources).post(create_resource))
}

async fn list_resources(State(state): State<AppState>) -> Result<Json<Vec<db::models::Resource>>, HttpError> {
    Ok(Json(db::ResourcesRepo::list_active(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct CreateResource { name: String, email: Option<String> }

async fn create_resource(
    State(state): State<AppState>,
    Json(body): Json<CreateResource>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = db::ResourcesRepo::create(&state.pool, &body.name, body.email.as_deref()).await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}
