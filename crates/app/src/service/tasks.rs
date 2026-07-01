use crate::error::AppError;
use db::models::{KanbanTask, Task, TaskSkillRequirement};
use db::repo::tasks::TaskCreate;
use db::{TaskDepsRepo, TasksRepo};
use sqlx::SqlitePool;

pub struct TasksService;

impl TasksService {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(pool, skill_reqs, tag_ids), fields(project_id = project_id, title = %title, estimate_pd = estimate_pd))]
    pub async fn create(
        pool: &SqlitePool,
        project_id: i64, title: &str, description: Option<&str>,
        estimate_pd: f64, start: Option<&str>, end: Option<&str>,
        is_long_term: bool, parent_task_id: Option<i64>, segment_kind: Option<&str>,
        sort_order: i64,
        skill_reqs: &[(i64, i64, bool, f64)], tag_ids: &[i64],
    ) -> Result<i64, AppError> {
        if estimate_pd < 0.0 {
            tracing::warn!(estimate_pd = estimate_pd, "invalid task estimate");
            return Err(domain::DomainError::InvalidValue { field: "estimate_pd", value: estimate_pd }.into());
        }
        if let (Some(s), Some(e)) = (start, end) {
            if e < s {
                tracing::warn!(project_id = project_id, start = %s, end = %e, "invalid task date window");
                return Err(domain::DomainError::InvalidDateWindow.into());
            }
        }
        for &(_, min_prof, _, _) in skill_reqs {
            if !(1..=5).contains(&min_prof) {
                tracing::warn!(min_proficiency = min_prof, "invalid skill requirement proficiency");
                return Err(domain::DomainError::InvalidValue { field: "min_proficiency", value: min_prof as f64 }.into());
            }
        }
        validate_segment(parent_task_id, segment_kind)?;
        // A segment must reference an existing, non-deleted parent task in the same project.
        if let Some(pid) = parent_task_id {
            let row: Option<(i64,)> = sqlx::query_as(
                "SELECT 1 FROM tasks WHERE id=? AND project_id=? AND deleted_at IS NULL")
                .bind(pid).bind(project_id).fetch_optional(pool).await?;
            if row.is_none() {
                tracing::warn!(project_id = project_id, parent_task_id = pid, "parent task not found");
                return Err(domain::DomainError::NotFound(format!("parent task {}", pid)).into());
            }
        }
        let input = TaskCreate {
            project_id, title, description, estimate_pd, start, end, is_long_term,
            parent_task_id, segment_kind, sort_order, skill_reqs, tag_ids,
        };
        let id = TasksRepo::create(pool, input).await?;
        tracing::info!(task_id = id, project_id = project_id, "created task");
        Ok(id)
    }

    #[tracing::instrument(skip(pool), fields(project_id = project_id))]
    pub async fn list_by_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<Task>, AppError> {
        Ok(TasksRepo::list_by_project(pool, project_id).await?)
    }

    #[tracing::instrument(skip(pool), fields(task_id = id, status = %status))]
    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), AppError> {
        match status {
            "todo" | "in_progress" | "blocked" | "review" | "done" | "cancelled" => {}
            _ => {
                tracing::warn!(task_id = id, status = %status, "invalid task status");
                return Err(domain::DomainError::InvalidStatus(status.into()).into());
            }
        }
        TasksRepo::set_status(pool, id, status).await?;
        tracing::info!(task_id = id, status = %status, "set task status");
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(task_id = id, title = %title, estimate_pd = estimate_pd))]
    pub async fn update(
        pool: &SqlitePool, id: i64, title: &str, description: Option<&str>,
        estimate_pd: f64, start: Option<&str>, end: Option<&str>,
        is_long_term: bool, parent_task_id: Option<i64>, segment_kind: Option<&str>,
    ) -> Result<(), AppError> {
        if estimate_pd < 0.0 {
            tracing::warn!(task_id = id, estimate_pd = estimate_pd, "invalid task estimate");
            return Err(domain::DomainError::InvalidValue { field: "estimate_pd", value: estimate_pd }.into());
        }
        if let (Some(s), Some(e)) = (start, end) {
            if e < s {
                tracing::warn!(task_id = id, start = %s, end = %e, "invalid task date window");
                return Err(domain::DomainError::InvalidDateWindow.into());
            }
        }
        validate_segment(parent_task_id, segment_kind)?;
        // A segment cannot be its own parent, and the parent chain must not cycle.
        if let Some(pid) = parent_task_id {
            if pid == id {
                tracing::warn!(task_id = id, "task cannot be its own parent");
                return Err(domain::DomainError::InvalidInput("task cannot be its own parent").into());
            }
            // Walk up the parent chain to ensure setting this parent doesn't create a cycle.
            let mut cur = pid;
            for _ in 0..1000 {
                let row: Option<(Option<i64>,)> = sqlx::query_as(
                    "SELECT parent_task_id FROM tasks WHERE id=? AND deleted_at IS NULL")
                    .bind(cur).fetch_optional(pool).await?;
                match row {
                    None => {
                        tracing::warn!(task_id = id, parent_task_id = cur, "parent task not found");
                        return Err(domain::DomainError::NotFound(format!("parent task {}", cur)).into());
                    }
                    Some((Some(p),)) if p == id => {
                        tracing::warn!(task_id = id, parent_task_id = pid, "task parent cycle detected");
                        return Err(domain::DomainError::DependencyCycle(id).into());
                    }
                    Some((Some(p),)) => cur = p,
                    Some((None,)) => break, // reached a top-level parent
                }
            }
        }
        TasksRepo::update(pool, id, title, description, estimate_pd, start, end,
            is_long_term, parent_task_id, segment_kind).await?;
        tracing::info!(task_id = id, "updated task");
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(task_id = id))]
    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        TasksRepo::soft_delete(pool, id).await?;
        tracing::info!(task_id = id, "deleted task");
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(task_id = task_id))]
    pub async fn skill_reqs(pool: &SqlitePool, task_id: i64) -> Result<Vec<TaskSkillRequirement>, AppError> {
        Ok(TasksRepo::list_skill_reqs(pool, task_id).await?)
    }

    #[tracing::instrument(skip(pool), fields(project_id = project_id))]
    pub async fn kanban(pool: &SqlitePool, project_id: i64) -> Result<Vec<KanbanTask>, AppError> {
        Ok(TasksRepo::list_kanban(pool, project_id).await?)
    }
}

impl TasksService {
    /// Add a dependency edge after checking it creates no cycle (design §3.3.12).
    ///
    /// `dep_type` is normalized to the schema's 4-code enum (`FS`/`FF`/`SS`/`SF`). The edge
    /// read + cycle check + insert run inside ONE `with_write_tx` (BEGIN IMMEDIATE + busy-retry,
    /// design §3.7) so a concurrent writer can't slip a cycle-forming edge between the check and
    /// the insert. The cycle rejection is carried out of the tx as the `T` value (no insert, the
    /// tx commits read-only) and re-raised as a DOMAIN error.
    #[tracing::instrument(skip(pool), fields(task_id = task_id, predecessor_id = predecessor_id, dep_type = %dep_type))]
    pub async fn add_dependency(
        pool: &SqlitePool,
        task_id: i64,
        predecessor_id: i64,
        lag_days: i64,
        dep_type: &str,
    ) -> Result<(), AppError> {
        if task_id == predecessor_id {
            tracing::warn!("task cannot depend on itself");
            return Err(domain::DomainError::InvalidInput("a task cannot depend on itself").into());
        }
        let dep_type = normalize_dep_type(dep_type)?;

        let outcome: Result<(), domain::DomainError> = db::tx::with_write_tx(pool, move |mut tx| {
            Box::pin(async move {
                // Project-internal semantics (design §3.3.12, tasks.md T1): a dependency edge
                // must connect two tasks in the SAME project. Cross-project scheduling order is
                // not modeled by the solver, so reject it at write time rather than silently
                // dropping the edge inside build_problem.
                let (p1,): (Option<i64>,) = sqlx::query_as("SELECT project_id FROM tasks WHERE id=?")
                    .bind(task_id).fetch_one(&mut *tx).await?;
                let (p2,): (Option<i64>,) = sqlx::query_as("SELECT project_id FROM tasks WHERE id=?")
                    .bind(predecessor_id).fetch_one(&mut *tx).await?;
                if p1 != p2 {
                    tracing::warn!(task_id = task_id, predecessor_id = predecessor_id, "cross-project dependency rejected");
                    return Ok((tx, Err(domain::DomainError::DependencyViolation {
                        task_id,
                        related_task_id: predecessor_id,
                    })));
                }
                let mut edges: Vec<(i64, i64)> =
                    sqlx::query_as("SELECT task_id, predecessor_id FROM task_dependencies")
                        .fetch_all(&mut *tx)
                        .await?;
                edges.push((task_id, predecessor_id));
                if has_cycle(&edges) {
                    tracing::warn!(task_id = task_id, predecessor_id = predecessor_id, "dependency cycle detected");
                    // Hand the tx back unmodified; carry the rejection as the success value.
                    return Ok((tx, Err(domain::DomainError::DependencyCycle(task_id))));
                }
                TaskDepsRepo::upsert_tx(&mut tx, task_id, predecessor_id, lag_days, dep_type)
                    .await?;
                Ok((tx, Ok::<(), domain::DomainError>(())))
            })
        })
        .await?;
        outcome.map_err(Into::into)
    }
}

/// Normalize a dependency-type input to the schema's 4-code enum (design §3.3.12). Accepts the
/// codes directly (case-insensitive) plus the common long forms; anything else is a VALIDATION
/// error rather than a CHECK-constraint 500 at INSERT time.
#[tracing::instrument(fields(dep_type = %s))]
fn normalize_dep_type(s: &str) -> Result<&'static str, AppError> {
    let code = match s.trim().to_ascii_uppercase().as_str() {
        "FS" | "FINISH_TO_START" | "FINISH-TO-START" => "FS",
        "FF" | "FINISH_TO_FINISH" | "FINISH-TO-FINISH" => "FF",
        "SS" | "START_TO_START" | "START-TO-START" => "SS",
        "SF" | "START_TO_FINISH" | "START-TO-FINISH" => "SF",
        _ => {
            tracing::warn!(dep_type = %s, "invalid dependency type");
            return Err(domain::DomainError::InvalidStatus(s.to_string()).into());
        }
    };
    Ok(code)
}

/// Validate long-term / segment fields (design §3.3.11, §3.4):
/// - `segment_kind` must be one of milestone|phase|segment (or None).
/// - A segment (non-None segment_kind) MUST have a parent_task_id; a plain long-term task
///   (is_long_term with no segment_kind) has parent NULL.
#[tracing::instrument(fields(parent_task_id = ?parent_task_id, segment_kind = ?segment_kind))]
fn validate_segment(parent_task_id: Option<i64>, segment_kind: Option<&str>) -> Result<(), AppError> {
    if let Some(kind) = segment_kind {
        if !matches!(kind, "milestone" | "phase" | "segment") {
            tracing::warn!(segment_kind = %kind, "invalid segment kind");
            return Err(domain::DomainError::InvalidInput("segment_kind must be milestone, phase, or segment").into());
        }
        if parent_task_id.is_none() {
            tracing::warn!("segment missing parent task");
            return Err(domain::DomainError::InvalidInput("a segment must reference a parent task").into());
        }
    }
    Ok(())
}

/// Edge direction: task depends on predecessor (task must come after predecessor).
/// A cycle in this graph means impossible ordering.
fn has_cycle(edges: &[(i64, i64)]) -> bool {
    use std::collections::{HashMap, HashSet};
    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
    for &(t, p) in edges { adj.entry(p).or_default().push(t); } // p -> t
    let nodes: HashSet<i64> = edges.iter().flat_map(|(t, p)| [*t, *p]).collect();
    let mut white = nodes.clone();
    while let Some(&start) = white.iter().next() {
        let mut stack = vec![start];
        let mut on_path = HashSet::new();
        while let Some(&n) = stack.last() {
            if !white.contains(&n) { stack.pop(); continue; }
            white.remove(&n);
            on_path.insert(n);
            let neighbors = adj.get(&n).cloned().unwrap_or_default();
            let mut descended = false;
            for nb in neighbors {
                if on_path.contains(&nb) { return true; } // back edge -> cycle
                if white.contains(&nb) {
                    stack.push(nb);
                    descended = true;
                    break;
                }
            }
            if !descended { on_path.remove(&n); stack.pop(); }
        }
    }
    false
}

#[cfg(test)]
mod cycle_tests {
    use super::has_cycle;
    #[test]
    fn acyclic_ok() { assert!(!has_cycle(&[(1, 2), (2, 3)])); }
    #[test]
    fn cycle_detected() { assert!(has_cycle(&[(1, 2), (2, 3), (3, 1)])); }
}
