use crate::error::DbError;
use crate::models::GanttBar;
use sqlx::SqlitePool;

pub struct GanttRepo;
impl GanttRepo {
    /// All allocation bars in a project (project Gantt view).
    pub async fn by_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<GanttBar>, DbError> {
        Ok(sqlx::query_as::<_, GanttBar>(
            "SELECT a.id AS allocation_id, a.resource_id, r.name AS resource_name, \
                    a.task_id, t.title AS task_title, t.project_id, p.name AS project_name, \
                    a.start_date, a.end_date, a.percent, a.status, a.source \
             FROM allocations a \
             JOIN resources r ON r.id = a.resource_id \
             JOIN tasks t ON t.id = a.task_id \
             JOIN projects p ON p.id = t.project_id \
             WHERE t.project_id = ? AND a.deleted_at IS NULL \
             ORDER BY r.name, a.start_date")
            .bind(project_id).fetch_all(pool).await?)
    }

    /// A resource's allocation bars across ALL projects (cross-project resource Gantt view).
    pub async fn by_resource(pool: &SqlitePool, resource_id: i64) -> Result<Vec<GanttBar>, DbError> {
        Ok(sqlx::query_as::<_, GanttBar>(
            "SELECT a.id AS allocation_id, a.resource_id, r.name AS resource_name, \
                    a.task_id, t.title AS task_title, t.project_id, p.name AS project_name, \
                    a.start_date, a.end_date, a.percent, a.status, a.source \
             FROM allocations a \
             JOIN resources r ON r.id = a.resource_id \
             JOIN tasks t ON t.id = a.task_id \
             JOIN projects p ON p.id = t.project_id \
             WHERE a.resource_id = ? AND a.deleted_at IS NULL \
             ORDER BY a.start_date")
            .bind(resource_id).fetch_all(pool).await?)
    }
}
