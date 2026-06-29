use crate::error::DbError;
use crate::models::Resource;
use sqlx::SqlitePool;

pub struct ResourcesRepo;

impl ResourcesRepo {
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

    pub async fn update(
        pool: &SqlitePool, id: i64, name: &str, email: Option<&str>,
    ) -> Result<(), DbError> {
        let n = sqlx::query(
            "UPDATE resources SET name=?, email=?, \
                    updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') \
             WHERE id=? AND deleted_at IS NULL")
            .bind(name).bind(email).bind(id)
            .execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }
}