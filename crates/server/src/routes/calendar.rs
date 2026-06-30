use crate::error::HttpError;
use crate::state::AppState;
use app::service::calendar::CalendarService;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use db::models::{Holiday, TimeOff, WeekTemplate};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        // work-week template (single global row in MVP)
        .route(
            "/api/calendar/work-week",
            get(list_work_weeks).post(set_global_work_week),
        )
        // holidays
        .route(
            "/api/calendar/holidays",
            get(list_holidays).post(add_holiday),
        )
        .route("/api/calendar/holidays/{id}", delete(delete_holiday))
        // time off
        .route(
            "/api/calendar/time-off",
            get(list_time_off).post(add_time_off),
        )
        .route("/api/calendar/time-off/{id}", delete(delete_time_off))
}

async fn list_work_weeks(
    State(state): State<AppState>,
) -> Result<Json<Vec<WeekTemplate>>, HttpError> {
    Ok(Json(CalendarService::work_weeks(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct SetGlobalWorkWeek {
    week: Vec<f64>,
}

async fn set_global_work_week(
    State(state): State<AppState>,
    Json(body): Json<SetGlobalWorkWeek>,
) -> Result<axum::http::StatusCode, HttpError> {
    CalendarService::set_global_work_week(&state.pool, body.week).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn list_holidays(State(state): State<AppState>) -> Result<Json<Vec<Holiday>>, HttpError> {
    Ok(Json(CalendarService::holidays(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct AddHoliday {
    project_id: Option<i64>,
    day: String,
    fraction: Option<f64>,
    name: Option<String>,
}

async fn add_holiday(
    State(state): State<AppState>,
    Json(body): Json<AddHoliday>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = CalendarService::add_holiday(
        &state.pool,
        body.project_id,
        &body.day,
        body.fraction,
        body.name.as_deref(),
    )
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

#[derive(Debug, Deserialize)]
struct AddTimeOff {
    resource_id: i64,
    day: String,
    fraction: Option<f64>,
    reason: Option<String>,
}

async fn add_time_off(
    State(state): State<AppState>,
    Json(body): Json<AddTimeOff>,
) -> Result<(axum::http::StatusCode, Json<i64>), HttpError> {
    let id = CalendarService::add_time_off(
        &state.pool,
        body.resource_id,
        &body.day,
        body.fraction,
        body.reason.as_deref(),
    )
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

async fn delete_holiday(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, HttpError> {
    CalendarService::delete_holiday(&state.pool, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn delete_time_off(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, HttpError> {
    CalendarService::delete_time_off(&state.pool, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn list_time_off(State(state): State<AppState>) -> Result<Json<Vec<TimeOff>>, HttpError> {
    Ok(Json(CalendarService::time_off_all(&state.pool).await?))
}
