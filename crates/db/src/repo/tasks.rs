use crate::error::DbError;
use crate::models::{KanbanTask, Task, TaskSkillRequirement};
use sqlx::SqlitePool;

pub struct TasksRepo;

/// Input for creating a task with its skill requirements and tags (design §3.3.11/13/14).
pub struct TaskCreate<'a> {
    pub project_id: i64,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub estimate_pd: f64,
    pub start: Option<&'a str>,
    pub end: Option<&'a str>,
    pub is_long_term: bool,
    pub sort_order: i64,
    pub skill_reqs: &'a [(i64 /*skill_id*/, i64 /*min_prof*/, bool /*mandatory*/, f64 /*weight*/)],
    pub tag_ids: &'a [i64],
}

impl TasksRepo {
    /// Atomic create: task + skill requirements + tags in one transaction.
    ///
    /// Uses the hand-back `with_write_tx` contract: the closure owns the `Transaction`,
    /// does its work with `&mut *tx`, and returns `(tx, id)` so `with_write_tx` commits.
    pub async fn create(pool: &SqlitePool, input: TaskCreate<'_>) -> Result<i64, DbError> {
        crate::tx::with_write_tx(pool, |mut tx| Box::pin(async move {
            let (id,): (i64,) = sqlx::query_as(
                "INSERT INTO tasks (project_id, title, description, estimate_pd, start_date, end_date, \
                 is_long_term, sort_order) VALUES (?,?,?,?,?,?,?,?) RETURNING id")
                .bind(input.project_id).bind(input.title).bind(input.description)
                .bind(input.estimate_pd).bind(input.start).bind(input.end)
                .bind(input.is_long_term as i64).bind(input.sort_order)
                .fetch_one(&mut *tx).await?;
            for &(skill_id, min_prof, mandatory, weight) in input.skill_reqs {
                sqlx::query(
                    "INSERT INTO task_skill_requirements (task_id, skill_id, min_proficiency, is_mandatory, weight) \
                     VALUES (?,?,?,?,?)")
                    .bind(id).bind(skill_id).bind(min_prof).bind(mandatory as i64).bind(weight)
                    .execute(&mut *tx).await?;
            }
            for &tag_id in input.tag_ids {
                sqlx::query("INSERT INTO task_tags (task_id, tag_id) VALUES (?,?)")
                    .bind(id).bind(tag_id).execute(&mut *tx).await?;
            }
            Ok((tx, id))
        })).await
    }

    pub async fn list_by_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<Task>, DbError> {
        Ok(sqlx::query_as::<_, Task>(
            "SELECT id, project_id, parent_task_id, title, description, estimate_pd, start_date, end_date, \
                    is_long_term, segment_kind, status, sort_order \
             FROM tasks WHERE project_id = ? AND deleted_at IS NULL ORDER BY sort_order, id")
            .bind(project_id).fetch_all(pool).await?)
    }

    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), DbError> {
        let n = sqlx::query("UPDATE tasks SET status = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(status).bind(id).execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    pub async fn list_skill_reqs(pool: &SqlitePool, task_id: i64) -> Result<Vec<TaskSkillRequirement>, DbError> {
        Ok(sqlx::query_as::<_, TaskSkillRequirement>(
            "SELECT task_id, skill_id, min_proficiency, is_mandatory, weight \
             FROM task_skill_requirements WHERE task_id = ?")
            .bind(task_id).fetch_all(pool).await?)
    }

    /// Kanban-shaped read: task + first assignee name + skill count (design §7 Kanban card).
    pub async fn list_kanban(pool: &SqlitePool, project_id: i64) -> Result<Vec<KanbanTask>, DbError> {
        Ok(sqlx::query_as::<_, KanbanTask>(
            "SELECT t.id, t.project_id, t.title, t.status, t.sort_order, t.estimate_pd, \
                    (SELECT r.name FROM allocations a JOIN resources r ON r.id = a.resource_id \
                     WHERE a.task_id = t.id AND a.deleted_at IS NULL LIMIT 1) AS assignee, \
                    (SELECT count(*) FROM task_skill_requirements sr WHERE sr.task_id = t.id) AS skill_count \
             FROM tasks t WHERE t.project_id = ? AND t.deleted_at IS NULL \
             ORDER BY t.sort_order, t.id")
            .bind(project_id).fetch_all(pool).await?)
    }
}

pub struct TaskDepsRepo;

impl TaskDepsRepo {
    pub async fn add(pool: &SqlitePool, task_id: i64, predecessor_id: i64, lag_days: i64) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO task_dependencies (task_id, predecessor_id, lag_days) VALUES (?,?,?) \
             ON CONFLICT(task_id, predecessor_id) DO UPDATE SET lag_days = excluded.lag_days")
            .bind(task_id).bind(predecessor_id).bind(lag_days)
            .execute(pool).await?;
        Ok(())
    }

    /// All (task_id, predecessor_id) edges, for in-memory cycle detection.
    pub async fn all_edges(pool: &SqlitePool) -> Result<Vec<(i64, i64)>, DbError> {
        Ok(sqlx::query_as("SELECT task_id, predecessor_id FROM task_dependencies")
            .fetch_all(pool).await?)
    }

    /// Direct predecessors of a task (for the Kanban/Gantt dependency display).
    pub async fn predecessors(pool: &SqlitePool, task_id: i64) -> Result<Vec<i64>, DbError> {
        let rows: Vec<(i64,)> = sqlx::query_as(
            "SELECT predecessor_id FROM task_dependencies WHERE task_id = ?")
            .bind(task_id).fetch_all(pool).await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Dependency edges among tasks of one project (for Gantt arrows).
    pub async fn for_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<crate::models::DepEdge>, DbError> {
        Ok(sqlx::query_as::<_, crate::models::DepEdge>(
            "SELECT d.task_id, d.predecessor_id, d.lag_days, d.dep_type \
             FROM task_dependencies d JOIN tasks t ON t.id = d.task_id \
             WHERE t.project_id = ?")
            .bind(project_id).fetch_all(pool).await?)
    }
}