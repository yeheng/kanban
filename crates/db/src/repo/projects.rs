use crate::error::DbError;
use crate::models::Project;
use sqlx::SqlitePool;

pub struct ProjectsRepo;

impl ProjectsRepo {
    pub async fn create(
        pool: &SqlitePool, name: &str, description: Option<&str>,
        start: Option<&str>, end: Option<&str>, priority: i64, budget_pd: f64,
    ) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO projects (name, description, start_date, end_date, priority, budget_pd) \
             VALUES (?,?,?,?,?,?) RETURNING id")
            .bind(name).bind(description).bind(start).bind(end).bind(priority).bind(budget_pd)
            .fetch_one(pool).await?;
        Ok(id)
    }

    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Project>, DbError> {
        let rows = sqlx::query_as::<_, Project>(
            "SELECT id, name, description, start_date, end_date, priority, budget_pd, \
                    max_parallel_tasks_per_day, status \
             FROM projects WHERE deleted_at IS NULL ORDER BY priority, name")
            .fetch_all(pool).await?;
        Ok(rows)
    }

    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Project, DbError> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, start_date, end_date, priority, budget_pd, \
                    max_parallel_tasks_per_day, status \
             FROM projects WHERE id = ? AND deleted_at IS NULL")
            .bind(id).fetch_optional(pool).await?.ok_or(DbError::NotFound)
    }

    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), DbError> {
        let n = sqlx::query("UPDATE projects SET status = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(status).bind(id).execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
        let n = sqlx::query(
            "UPDATE projects SET deleted_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') \
             WHERE id = ? AND deleted_at IS NULL")
            .bind(id).execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }
}