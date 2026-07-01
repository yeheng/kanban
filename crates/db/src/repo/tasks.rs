use crate::error::DbError;
use crate::models::{KanbanTask, Task, TaskSkillRequirement};
use sqlx::SqlitePool;
use tracing;

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
    pub parent_task_id: Option<i64>,
    pub segment_kind: Option<&'a str>,
    pub sort_order: i64,
    pub skill_reqs: &'a [(i64 /*skill_id*/, i64 /*min_prof*/, bool /*mandatory*/, f64 /*weight*/)],
    pub tag_ids: &'a [i64],
}

impl TasksRepo {
    /// Atomic create: task + skill requirements + tags in one transaction.
    ///
    /// Uses the hand-back `with_write_tx` contract: the closure owns the `Transaction`,
    /// does its work with `&mut *tx`, and returns `(tx, id)` so `with_write_tx` commits.
    #[tracing::instrument(skip_all, level = "debug", fields(project_id = input.project_id))]
    pub async fn create(pool: &SqlitePool, input: TaskCreate<'_>) -> Result<i64, DbError> {
        crate::tx::with_write_tx(pool, |mut tx| Box::pin(async move {
            let (id,): (i64,) = sqlx::query_as(
                "INSERT INTO tasks (project_id, parent_task_id, title, description, estimate_pd, start_date, end_date, \
                 is_long_term, segment_kind, sort_order) VALUES (?,?,?,?,?,?,?,?,?,?) RETURNING id")
                .bind(input.project_id).bind(input.parent_task_id).bind(input.title).bind(input.description)
                .bind(input.estimate_pd).bind(input.start).bind(input.end)
                .bind(input.is_long_term as i64).bind(input.segment_kind).bind(input.sort_order)
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

    #[tracing::instrument(skip_all, level = "debug", fields(project_id))]
    pub async fn list_by_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<Task>, DbError> {
        Ok(sqlx::query_as::<_, Task>(
            "SELECT id, project_id, parent_task_id, title, description, estimate_pd, start_date, end_date, \
                    is_long_term, segment_kind, status, sort_order \
             FROM tasks WHERE project_id = ? AND deleted_at IS NULL ORDER BY sort_order, id")
            .bind(project_id).fetch_all(pool).await?)
    }

    #[tracing::instrument(skip_all, level = "debug", fields(id, status))]
    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), DbError> {
        let n = sqlx::query("UPDATE tasks SET status = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(status).bind(id).execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", fields(id))]
    pub async fn update(
        pool: &SqlitePool, id: i64, title: &str, description: Option<&str>,
        estimate_pd: f64, start: Option<&str>, end: Option<&str>,
        is_long_term: bool, parent_task_id: Option<i64>, segment_kind: Option<&str>,
    ) -> Result<(), DbError> {
        let n = sqlx::query(
            "UPDATE tasks SET title=?, description=?, estimate_pd=?, start_date=?, end_date=?, \
                    is_long_term=?, parent_task_id=?, segment_kind=?, \
                    updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') \
             WHERE id=? AND deleted_at IS NULL")
            .bind(title).bind(description).bind(estimate_pd).bind(start).bind(end)
            .bind(is_long_term as i64).bind(parent_task_id).bind(segment_kind).bind(id)
            .execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", fields(id))]
    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
        let n = sqlx::query(
            "UPDATE tasks SET deleted_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') \
             WHERE id = ? AND deleted_at IS NULL")
            .bind(id).execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", fields(task_id))]
    pub async fn list_skill_reqs(pool: &SqlitePool, task_id: i64) -> Result<Vec<TaskSkillRequirement>, DbError> {
        Ok(sqlx::query_as::<_, TaskSkillRequirement>(
            "SELECT task_id, skill_id, min_proficiency, is_mandatory, weight \
             FROM task_skill_requirements WHERE task_id = ?")
            .bind(task_id).fetch_all(pool).await?)
    }

    /// Kanban-shaped read: task + first assignee name + skill count (design §7 Kanban card).
    #[tracing::instrument(skip_all, level = "debug", fields(project_id))]
    pub async fn list_kanban(pool: &SqlitePool, project_id: i64) -> Result<Vec<KanbanTask>, DbError> {
        Ok(sqlx::query_as::<_, KanbanTask>(
            "SELECT t.id, t.project_id, t.parent_task_id, t.title, t.description, t.is_long_term, t.segment_kind, \
                    t.status, t.sort_order, t.estimate_pd, t.start_date, t.end_date, \
                    (SELECT r.name FROM allocations a JOIN resources r ON r.id = a.resource_id \
                     WHERE a.task_id = t.id AND a.deleted_at IS NULL AND r.deleted_at IS NULL LIMIT 1) AS assignee, \
                    (SELECT count(*) FROM task_skill_requirements sr WHERE sr.task_id = t.id) AS skill_count \
             FROM tasks t WHERE t.project_id = ? AND t.deleted_at IS NULL \
             ORDER BY t.sort_order, t.id")
            .bind(project_id).fetch_all(pool).await?)
    }
}

pub struct TaskDepsRepo;

impl TaskDepsRepo {
    /// Insert (or update the lag/type of) a dependency edge inside an existing write tx.
    /// Cycle-freedom is enforced by the caller (the service reads all edges in the SAME tx
    /// before calling this), so this is purely the write step. `dep_type` must already be one
    /// of the schema's `FS`/`FF`/`SS`/`SF` codes (the service normalizes it).
    #[tracing::instrument(skip_all, level = "debug", fields(task_id, predecessor_id, lag_days, dep_type))]
    pub async fn upsert_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        task_id: i64,
        predecessor_id: i64,
        lag_days: i64,
        dep_type: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO task_dependencies (task_id, predecessor_id, lag_days, dep_type) \
             VALUES (?,?,?,?) \
             ON CONFLICT(task_id, predecessor_id) DO UPDATE SET \
             lag_days = excluded.lag_days, dep_type = excluded.dep_type",
        )
        .bind(task_id)
        .bind(predecessor_id)
        .bind(lag_days)
        .bind(dep_type)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// All (task_id, predecessor_id) edges for live tasks, for in-memory cycle detection.
    #[tracing::instrument(skip_all, level = "debug")]
    pub async fn all_edges(pool: &SqlitePool) -> Result<Vec<(i64, i64)>, DbError> {
        Ok(sqlx::query_as(
            "SELECT d.task_id, d.predecessor_id \
             FROM task_dependencies d \
             JOIN tasks t1 ON t1.id = d.task_id AND t1.deleted_at IS NULL \
             JOIN tasks t2 ON t2.id = d.predecessor_id AND t2.deleted_at IS NULL")
            .fetch_all(pool).await?)
    }

    /// Direct predecessors of a task (for the Kanban/Gantt dependency display).
    #[tracing::instrument(skip_all, level = "debug", fields(task_id))]
    pub async fn predecessors(pool: &SqlitePool, task_id: i64) -> Result<Vec<i64>, DbError> {
        let rows: Vec<(i64,)> = sqlx::query_as(
            "SELECT predecessor_id FROM task_dependencies WHERE task_id = ?")
            .bind(task_id).fetch_all(pool).await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Dependency edges among tasks of one project (for Gantt arrows).
    /// Filters both the dependent task and the predecessor for soft-deletes.
    #[tracing::instrument(skip_all, level = "debug", fields(project_id))]
    pub async fn for_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<crate::models::DepEdge>, DbError> {
        Ok(sqlx::query_as::<_, crate::models::DepEdge>(
            "SELECT d.task_id, d.predecessor_id, d.lag_days, d.dep_type \
             FROM task_dependencies d \
             JOIN tasks t ON t.id = d.task_id AND t.deleted_at IS NULL \
             JOIN tasks tp ON tp.id = d.predecessor_id AND tp.deleted_at IS NULL \
             WHERE t.project_id = ?")
            .bind(project_id).fetch_all(pool).await?)
    }
}
