use crate::error::AppError;
use db::models::{Resource, ResourceSkill, ResourceTag};
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

    /// Read a resource's skills (with skill name resolved) — design §3.3.5.
    pub async fn list_skills(pool: &SqlitePool, id: i64) -> Result<Vec<ResourceSkill>, AppError> {
        Ok(ResourcesRepo::list_skills(pool, id).await?)
    }

    /// Replace a resource's skills. Each entry is (skill_id, proficiency); proficiency must
    /// be 1..=5 (design §3.3.5 CHECK). Skill ids are validated against the `skills` table.
    pub async fn set_skills(
        pool: &SqlitePool, id: i64, skills: &[(i64, i64)],
    ) -> Result<(), AppError> {
        for &(sid, prof) in skills {
            if !(1..=5).contains(&prof) {
                return Err(domain::DomainError::InvalidRatio(prof as f64).into());
            }
            let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM skills WHERE id=?")
                .bind(sid).fetch_optional(pool).await?;
            if exists.is_none() {
                return Err(domain::DomainError::NotFound(format!("skill {}", sid)).into());
            }
        }
        Ok(ResourcesRepo::set_skills(pool, id, skills).await?)
    }

    /// Read a resource's tags (with tag name + color resolved) — design §3.3.6.
    pub async fn list_tags(pool: &SqlitePool, id: i64) -> Result<Vec<ResourceTag>, AppError> {
        Ok(ResourcesRepo::list_tags(pool, id).await?)
    }

    /// Replace a resource's tags. Tag ids are validated against the `tags` table.
    pub async fn set_tags(pool: &SqlitePool, id: i64, tag_ids: &[i64]) -> Result<(), AppError> {
        for &tid in tag_ids {
            let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM tags WHERE id=?")
                .bind(tid).fetch_optional(pool).await?;
            if exists.is_none() {
                return Err(domain::DomainError::NotFound(format!("tag {}", tid)).into());
            }
        }
        Ok(ResourcesRepo::set_tags(pool, id, tag_ids).await?)
    }
}
