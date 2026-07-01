use crate::error::DbError;
use crate::models::Skill;
use sqlx::SqlitePool;
use tracing;

pub struct SkillsRepo;

impl SkillsRepo {
    /// Upsert by name; returns the skill id. Used for catalog management + LLM normalization.
    #[tracing::instrument(skip_all, level = "debug", fields(name))]
    pub async fn ensure(pool: &SqlitePool, name: &str) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO skills (name) VALUES (?) \
             ON CONFLICT(name) DO UPDATE SET name=excluded.name \
             RETURNING id")
            .bind(name).fetch_one(pool).await?;
        Ok(id)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Skill>, DbError> {
        Ok(sqlx::query_as::<_, Skill>("SELECT id, name FROM skills ORDER BY name").fetch_all(pool).await?)
    }
}