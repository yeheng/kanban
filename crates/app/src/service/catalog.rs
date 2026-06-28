use crate::error::AppError;
use db::models::{Skill, Tag};
use db::{SkillsRepo, TagsRepo};
use sqlx::SqlitePool;

pub struct CatalogService;

impl CatalogService {
    pub async fn ensure_skill(pool: &SqlitePool, name: &str) -> Result<i64, AppError> {
        Ok(SkillsRepo::ensure(pool, name).await?)
    }
    pub async fn list_skills(pool: &SqlitePool) -> Result<Vec<Skill>, AppError> {
        Ok(SkillsRepo::list(pool).await?)
    }
    pub async fn ensure_tag(pool: &SqlitePool, name: &str, color: Option<&str>) -> Result<i64, AppError> {
        Ok(TagsRepo::ensure(pool, name, color).await?)
    }
    pub async fn list_tags(pool: &SqlitePool) -> Result<Vec<Tag>, AppError> {
        Ok(TagsRepo::list(pool).await?)
    }
}