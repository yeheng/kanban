use crate::error::DbError;
use crate::models::AllocationRow;
use sqlx::SqlitePool;

pub struct AllocationsRepo;

impl AllocationsRepo {
    /// Insert an allocation. Caller guarantees the (task,resource,window) validity —
    /// the DB trigger (trg_allocation_validate_insert) is the schema-level backstop.
    pub async fn create(
        pool: &SqlitePool,
        resource_id: i64,
        task_id: i64,
        start: &str,
        end: &str,
        percent: f64,
    ) -> Result<i64, DbError> {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO allocations (resource_id, task_id, start_date, end_date, percent) \
             VALUES (?, ?, ?, ?, ?) RETURNING id"
        )
        .bind(resource_id)
        .bind(task_id)
        .bind(start)
        .bind(end)
        .bind(percent)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    /// All active allocations for a resource overlapping [start, end], joined to the
    /// task's project so they can be bridged to `domain::Allocation` via `to_domain()`.
    pub async fn list_for_resource(
        pool: &SqlitePool,
        resource_id: i64,
        start: &str,
        end: &str,
    ) -> Result<Vec<AllocationRow>, DbError> {
        // Column order matches AllocationRow field order (project_id joined in).
        let rows: Vec<AllocationRow> = sqlx::query_as(
            "SELECT a.id, a.resource_id, a.task_id, t.project_id, \
                    a.start_date, a.end_date, a.percent, a.status, a.source, a.run_id \
             FROM allocations a JOIN tasks t ON t.id = a.task_id \
             WHERE a.resource_id = ? AND a.deleted_at IS NULL \
               AND a.start_date <= ? AND a.end_date >= ? \
             ORDER BY a.start_date"
        )
        .bind(resource_id)
        .bind(end)   // a.start_date <= window.end
        .bind(start) // a.end_date   >= window.start  => overlap
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// All active allocations on a project, joined with resource name + task title.
    /// Column order matches `AllocationView` field order.
    pub async fn list_by_project(
        pool: &SqlitePool,
        project_id: i64,
    ) -> Result<Vec<crate::models::AllocationView>, DbError> {
        let rows: Vec<crate::models::AllocationView> = sqlx::query_as(
            "SELECT a.id, a.resource_id, r.name AS resource_name, a.task_id, t.title AS task_title, \
                    t.project_id, a.start_date, a.end_date, a.percent, a.status, a.source \
             FROM allocations a \
             JOIN resources r ON r.id = a.resource_id \
             JOIN tasks t ON t.id = a.task_id \
             WHERE t.project_id = ? AND a.deleted_at IS NULL \
             ORDER BY a.start_date"
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}