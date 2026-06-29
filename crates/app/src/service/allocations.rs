use crate::error::AppError;
use crate::service::dates;
use db::models::AllocationView;
use db::AllocationsRepo;
use sqlx::SqlitePool;

pub struct AllocationsService;

impl AllocationsService {
    pub async fn create(
        pool: &SqlitePool,
        resource_id: i64,
        task_id: i64,
        start: &str,
        end: &str,
        percent: f64,
    ) -> Result<i64, AppError> {
        validate_percent(percent)?;
        let window = dates::required_window(start, end)?;
        Ok(AllocationsRepo::create(
            pool,
            resource_id,
            task_id,
            &window.start,
            &window.end,
            percent,
        )
        .await?)
    }

    pub async fn list_by_project(
        pool: &SqlitePool,
        project_id: i64,
    ) -> Result<Vec<AllocationView>, AppError> {
        Ok(AllocationsRepo::list_by_project(pool, project_id).await?)
    }

    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        start: &str,
        end: &str,
        percent: f64,
    ) -> Result<(), AppError> {
        validate_percent(percent)?;
        let window = dates::required_window(start, end)?;
        Ok(AllocationsRepo::update(pool, id, &window.start, &window.end, percent).await?)
    }
}

fn validate_percent(percent: f64) -> Result<(), AppError> {
    if percent.is_finite() && percent > 0.0 && percent <= 1.0 {
        Ok(())
    } else {
        Err(domain::DomainError::InvalidRatio(percent).into())
    }
}
