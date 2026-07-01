use crate::error::DbError;
use crate::models::{Resource, ResourceSkill, ResourceTag};
use crate::tx::with_write_tx;
use sqlx::SqlitePool;
use tracing;

pub struct ResourcesRepo;

impl ResourcesRepo {
    #[tracing::instrument(skip_all, level = "debug", fields(name))]
    pub async fn create(pool: &SqlitePool, name: &str, email: Option<&str>) -> Result<i64, DbError> {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO resources (name, email) VALUES (?, ?) RETURNING id"
        )
        .bind(name)
        .bind(email)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Resource>, DbError> {
        let rows = sqlx::query_as::<_, Resource>(
            "SELECT id, name, email, available_from, available_to, status, \
             daily_capacity_pd, daily_rate_pd, max_parallel_tasks_per_day, metadata \
             FROM resources WHERE deleted_at IS NULL ORDER BY name"
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    #[tracing::instrument(skip_all, level = "debug", fields(id))]
    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Resource, DbError> {
        sqlx::query_as::<_, Resource>(
            "SELECT id, name, email, available_from, available_to, status, \
             daily_capacity_pd, daily_rate_pd, max_parallel_tasks_per_day, metadata \
             FROM resources WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(DbError::NotFound)
    }

    #[tracing::instrument(skip_all, level = "debug", fields(id))]
    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
        let n = sqlx::query(
            "UPDATE resources SET deleted_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", fields(id))]
    pub async fn update(
        pool: &SqlitePool, id: i64, name: &str, email: Option<&str>,
        available_from: Option<&str>, available_to: Option<&str>,
        daily_capacity_pd: Option<f64>, daily_rate_pd: Option<f64>,
    ) -> Result<(), DbError> {
        // `daily_capacity_pd` is NOT NULL in the schema, so a `None` here means "field left
        // untouched in the edit form" — keep the existing value via COALESCE rather than
        // clobbering it to a magic 1.0 (which silently corrupts every utilization calc).
        let n = sqlx::query(
            "UPDATE resources SET name=?, email=?, available_from=?, available_to=?, \
                    daily_capacity_pd=COALESCE(?, daily_capacity_pd), daily_rate_pd=?, \
                    updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') \
            WHERE id=? AND deleted_at IS NULL")
            .bind(name).bind(email).bind(available_from).bind(available_to)
            .bind(daily_capacity_pd).bind(daily_rate_pd).bind(id)
            .execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    /// List a resource's skills with the skill name resolved (design §3.3.5).
    #[tracing::instrument(skip_all, level = "debug", fields(resource_id))]
    pub async fn list_skills(pool: &SqlitePool, resource_id: i64) -> Result<Vec<ResourceSkill>, DbError> {
        Ok(sqlx::query_as::<_, ResourceSkill>(
            "SELECT rs.resource_id, rs.skill_id, s.name AS skill_name, rs.proficiency, rs.evidence \
             FROM resource_skills rs JOIN skills s ON s.id = rs.skill_id \
             WHERE rs.resource_id = ? ORDER BY s.name",
        )
        .bind(resource_id)
        .fetch_all(pool)
        .await?)
    }

    /// Replace a resource's skills atomically: (skill_id, proficiency). Caller validates
    /// proficiency ∈ 1..=5 and that skill ids exist. Uses a write tx so a partial failure
    /// rolls back (design §3.7 single-source-of-truth write paths).
    #[tracing::instrument(skip_all, level = "debug", fields(resource_id))]
    pub async fn set_skills(
        pool: &SqlitePool, resource_id: i64, skills: &[(i64, i64)],
    ) -> Result<(), DbError> {
        with_write_tx(pool, move |mut tx| {
            Box::pin(async move {
                sqlx::query("DELETE FROM resource_skills WHERE resource_id = ?")
                    .bind(resource_id)
                    .execute(&mut *tx).await?;
                for &(sid, prof) in skills {
                    sqlx::query(
                        "INSERT INTO resource_skills (resource_id, skill_id, proficiency) VALUES (?,?,?)")
                        .bind(resource_id).bind(sid).bind(prof)
                        .execute(&mut *tx).await?;
                }
                Ok((tx, ()))
            })
        }).await
    }

    /// List a resource's tags with the tag name + color resolved (design §3.3.6).
    #[tracing::instrument(skip_all, level = "debug", fields(resource_id))]
    pub async fn list_tags(pool: &SqlitePool, resource_id: i64) -> Result<Vec<ResourceTag>, DbError> {
        Ok(sqlx::query_as::<_, ResourceTag>(
            "SELECT rt.resource_id, rt.tag_id, t.name AS tag_name, t.color \
             FROM resource_tags rt JOIN tags t ON t.id = rt.tag_id \
             WHERE rt.resource_id = ? ORDER BY t.name",
        )
        .bind(resource_id)
        .fetch_all(pool)
        .await?)
    }

    /// Replace a resource's tags atomically: tag_ids. Caller validates tag ids exist.
    #[tracing::instrument(skip_all, level = "debug", fields(resource_id))]
    pub async fn set_tags(
        pool: &SqlitePool, resource_id: i64, tag_ids: &[i64],
    ) -> Result<(), DbError> {
        with_write_tx(pool, move |mut tx| {
            Box::pin(async move {
                sqlx::query("DELETE FROM resource_tags WHERE resource_id = ?")
                    .bind(resource_id)
                    .execute(&mut *tx).await?;
                for &tid in tag_ids {
                    sqlx::query("INSERT INTO resource_tags (resource_id, tag_id) VALUES (?,?)")
                        .bind(resource_id).bind(tid)
                        .execute(&mut *tx).await?;
                }
                Ok((tx, ()))
            })
        }).await
    }
}
