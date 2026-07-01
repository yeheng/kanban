use crate::error::AppError;
use db::models::{Skill, Tag};
use db::{SkillsRepo, TagsRepo};
use sqlx::SqlitePool;

pub struct CatalogService;

impl CatalogService {
    #[tracing::instrument(skip(pool), fields(name = %name))]
    pub async fn ensure_skill(pool: &SqlitePool, name: &str) -> Result<i64, AppError> {
        let id = SkillsRepo::ensure(pool, name).await?;
        tracing::info!(skill_id = id, name = %name, "ensured skill");
        Ok(id)
    }

    #[tracing::instrument(skip(pool))]
    pub async fn list_skills(pool: &SqlitePool) -> Result<Vec<Skill>, AppError> {
        Ok(SkillsRepo::list(pool).await?)
    }

    #[tracing::instrument(skip(pool), fields(name = %name))]
    pub async fn ensure_tag(pool: &SqlitePool, name: &str, color: Option<&str>) -> Result<i64, AppError> {
        let id = TagsRepo::ensure(pool, name, color).await?;
        tracing::info!(tag_id = id, name = %name, "ensured tag");
        Ok(id)
    }

    #[tracing::instrument(skip(pool))]
    pub async fn list_tags(pool: &SqlitePool) -> Result<Vec<Tag>, AppError> {
        Ok(TagsRepo::list(pool).await?)
    }
}
