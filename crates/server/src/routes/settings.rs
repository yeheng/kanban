use crate::error::HttpError;
use crate::state::AppState;
use app::service::settings::{SettingsDto, SettingsService};
use axum::routing::get;
use axum::{extract::State, Json, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/settings", get(get_settings).put(update_settings))
}

async fn get_settings(State(state): State<AppState>) -> Result<Json<SettingsDto>, HttpError> {
    Ok(Json(SettingsService::get(&state.pool).await?))
}

async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<SettingsDto>,
) -> Result<axum::http::StatusCode, HttpError> {
    SettingsService::update(&state.pool, body).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
