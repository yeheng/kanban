use crate::error::HttpError;
use crate::state::AppState;
use app::error::AppError;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use db::models::{Holiday, WeekTemplate};
use db::{HolidayRepo, ProjectsRepo, ResourcesRepo, TimeOffRepo, WeekTemplateRepo};
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
        // time off
        .route("/api/calendar/time-off", post(add_time_off))
}

async fn list_work_weeks(
    State(state): State<AppState>,
) -> Result<Json<Vec<WeekTemplate>>, HttpError> {
    Ok(Json(WeekTemplateRepo::list(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
struct SetGlobalWorkWeek {
    week: Vec<f64>,
}

async fn set_global_work_week(
    State(state): State<AppState>,
    Json(body): Json<SetGlobalWorkWeek>,
) -> Result<axum::http::StatusCode, HttpError> {
    if body.week.len() != 7 {
        return Err(domain::DomainError::InvalidRatio(body.week.len() as f64).into());
    }
    validate_week(&body.week)?;
    let arr: [f64; 7] = [
        body.week[0],
        body.week[1],
        body.week[2],
        body.week[3],
        body.week[4],
        body.week[5],
        body.week[6],
    ];
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
    validate_date(&body.day)?;
    let fraction = body.fraction.unwrap_or(1.0);
    validate_positive_fraction(fraction)?;
    if let Some(project_id) = body.project_id {
        ProjectsRepo::get(&state.pool, project_id).await?;
    }
    let id = HolidayRepo::add(
        &state.pool,
        body.project_id,
        &body.day,
        fraction,
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
    validate_date(&body.day)?;
    let fraction = body.fraction.unwrap_or(1.0);
    validate_positive_fraction(fraction)?;
    ResourcesRepo::get(&state.pool, body.resource_id).await?;
    let id = TimeOffRepo::add(
        &state.pool,
        body.resource_id,
        &body.day,
        fraction,
        body.reason.as_deref(),
    )
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(id)))
}

fn validate_week(week: &[f64]) -> Result<(), AppError> {
    for &fraction in week {
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(domain::DomainError::InvalidRatio(fraction).into());
        }
    }
    Ok(())
}

fn validate_positive_fraction(fraction: f64) -> Result<(), AppError> {
    if fraction.is_finite() && fraction > 0.0 && fraction <= 1.0 {
        Ok(())
    } else {
        Err(domain::DomainError::InvalidRatio(fraction).into())
    }
}

fn validate_date(day: &str) -> Result<(), AppError> {
    NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| domain::DomainError::InvalidDateWindow.into())
}

#[cfg(test)]
mod tests {
    use super::{validate_date, validate_positive_fraction, validate_week};

    #[test]
    fn validates_calendar_input_ranges() {
        assert!(validate_week(&[1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0]).is_ok());
        assert_eq!(validate_week(&[1.2]).unwrap_err().code, "VALIDATION");
        assert!(validate_positive_fraction(1.0).is_ok());
        assert_eq!(
            validate_positive_fraction(0.0).unwrap_err().code,
            "VALIDATION"
        );
    }

    #[test]
    fn validates_calendar_dates() {
        assert!(validate_date("2026-07-01").is_ok());
        assert_eq!(validate_date("2026-99-01").unwrap_err().code, "VALIDATION");
    }
}
