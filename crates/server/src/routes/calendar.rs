use crate::error::HttpError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use db::models::{Holiday, WeekTemplate};
use db::{HolidayRepo, TimeOffRepo, WeekTemplateRepo};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        // work-week template (single global row in MVP)
        .route("/api/calendar/work-week", get(list_work_weeks).post(set_global_work_week))
        // holidays
        .route("/api/calendar/holidays", get(list_holidays).post(add_holiday))
        // time off
        .route("/api/calendar/time-off", post(add_time_off))
}

async fn list_work_weeks(State(state): State<AppState>) -> Result<Json<Vec<WeekTemplate>>, HttpError> {
    Ok(Json(WeekTemplateRepo::list(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct SetGlobalWorkWeek { week: Vec<f64> }

async fn set_global_work_week(
    State(state): State<AppState>,
    Json(body): Json<SetGlobalWorkWeek>,
) -> Result<axum::http::StatusCode, HttpError> {
    if body.week.len() != 7 {
        return Err(domain::DomainError::InvalidRatio(body.week.len() as f64).into());
    }
    let arr: [f64; 7] = [body.week[0], body.week[1], body.week[2], body.week[3],
                         body.week[4], body.week[5], body.week[6]];
    WeekTemplateRepo::upsert_global(&state.pool, arr).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn list_holidays(State(state): State<AppState>) -> Result<Json<Vec<Holiday>>, HttpError> {
    Ok(Json(HolidayRepo::list(&state.pool).await?))
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
    let id = HolidayRepo::add(
        &state.pool, body.project_id, &body.day, body.fraction.unwrap_or(1.0), body.name.as_deref(),
    ).await?;
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
    let id = TimeOffRepo::add(
        &state.pool, body.resource_id, &body.day, body.fraction.unwrap_or(1.0), body.reason.as_deref(),
    ).await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}
