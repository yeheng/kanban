use crate::error::AppError;
use chrono::NaiveDate;
use db::models::{Holiday, TimeOff, WeekTemplate};
use db::{HolidayRepo, ProjectsRepo, ResourcesRepo, TimeOffRepo, WeekTemplateRepo};
use sqlx::SqlitePool;

pub struct CalendarService;

impl CalendarService {
    pub async fn work_weeks(pool: &SqlitePool) -> Result<Vec<WeekTemplate>, AppError> {
        Ok(WeekTemplateRepo::list(pool).await?)
    }

    pub async fn set_global_work_week(pool: &SqlitePool, week: Vec<f64>) -> Result<(), AppError> {
        if week.len() != 7 {
            return Err(domain::DomainError::InvalidInput("work week must have exactly 7 day fractions").into());
        }
        validate_week(&week)?;
        let arr = [
            week[0], week[1], week[2], week[3], week[4], week[5], week[6],
        ];
        WeekTemplateRepo::upsert_global(pool, arr).await?;
        Ok(())
    }

    pub async fn holidays(pool: &SqlitePool) -> Result<Vec<Holiday>, AppError> {
        Ok(HolidayRepo::list(pool).await?)
    }

    pub async fn add_holiday(
        pool: &SqlitePool,
        project_id: Option<i64>,
        day: &str,
        fraction: Option<f64>,
        name: Option<&str>,
    ) -> Result<i64, AppError> {
        validate_date(day)?;
        let fraction = fraction.unwrap_or(1.0);
        validate_positive_fraction(fraction)?;
        if let Some(project_id) = project_id {
            ProjectsRepo::get(pool, project_id).await?;
        }
        Ok(HolidayRepo::add(pool, project_id, day, fraction, name).await?)
    }

    pub async fn add_time_off(
        pool: &SqlitePool,
        resource_id: i64,
        day: &str,
        fraction: Option<f64>,
        reason: Option<&str>,
    ) -> Result<i64, AppError> {
        validate_date(day)?;
        let fraction = fraction.unwrap_or(1.0);
        validate_positive_fraction(fraction)?;
        ResourcesRepo::get(pool, resource_id).await?;
        Ok(TimeOffRepo::add(pool, resource_id, day, fraction, reason).await?)
    }

    pub async fn delete_holiday(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        Ok(HolidayRepo::delete(pool, id).await?)
    }

    pub async fn delete_time_off(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        Ok(TimeOffRepo::delete(pool, id).await?)
    }

    pub async fn time_off_all(pool: &SqlitePool) -> Result<Vec<TimeOff>, AppError> {
        Ok(TimeOffRepo::list_all(pool).await?)
    }
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
