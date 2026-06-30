use crate::error::AppError;
use db::models::Resource;
use db::ResourcesRepo;
use sqlx::SqlitePool;

pub struct ResourcesService;

impl ResourcesService {
    pub async fn create(pool: &SqlitePool, name: &str, email: Option<&str>) -> Result<i64, AppError> {
        Ok(ResourcesRepo::create(pool, name, email).await?)
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Resource>, AppError> {
        Ok(ResourcesRepo::list_active(pool).await?)
    }

    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        Ok(ResourcesRepo::soft_delete(pool, id).await?)
    }

    pub async fn update(
        pool: &SqlitePool, id: i64, name: &str, email: Option<&str>,
        available_from: Option<&str>, available_to: Option<&str>,
        daily_capacity_pd: Option<f64>, daily_rate_pd: Option<f64>,
    ) -> Result<(), AppError> {
        if let (Some(s), Some(e)) = (available_from, available_to) {
            if e < s { return Err(domain::DomainError::InvalidDateWindow.into()); }
        }
        if let Some(capacity) = daily_capacity_pd {
            if !capacity.is_finite() || capacity <= 0.0 {
                return Err(domain::DomainError::InvalidRatio(capacity).into());
            }
        }
        if let Some(rate) = daily_rate_pd {
            if !rate.is_finite() || rate < 0.0 {
                return Err(domain::DomainError::InvalidRatio(rate).into());
            }
        }
        Ok(ResourcesRepo::update(pool, id, name, email, available_from, available_to,
            daily_capacity_pd, daily_rate_pd).await?)
    }
}
