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
}
