use crate::error::DbError;
use crate::models::Tag;
use sqlx::SqlitePool;
use tracing;

pub struct TagsRepo;

impl TagsRepo {
    #[tracing::instrument(skip_all, level = "debug", fields(name))]
    pub async fn ensure(pool: &SqlitePool, name: &str, color: Option<&str>) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO tags (name, color) VALUES (?, ?) \
             ON CONFLICT(name) DO UPDATE SET color = COALESCE(excluded.color, tags.color) \
             RETURNING id")
            .bind(name).bind(color).fetch_one(pool).await?;
        Ok(id)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Tag>, DbError> {
        Ok(sqlx::query_as::<_, Tag>("SELECT id, name, color FROM tags ORDER BY name").fetch_all(pool).await?)
    }
}