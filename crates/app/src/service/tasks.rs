use crate::error::AppError;
use db::models::{KanbanTask, Task, TaskSkillRequirement};
use db::repo::tasks::TaskCreate;
use db::TasksRepo;
use sqlx::SqlitePool;

pub struct TasksService;

impl TasksService {
    pub async fn create(
        pool: &SqlitePool,
        project_id: i64, title: &str, description: Option<&str>,
        estimate_pd: f64, start: Option<&str>, end: Option<&str>,
        is_long_term: bool, sort_order: i64,
        skill_reqs: &[(i64, i64, bool, f64)], tag_ids: &[i64],
    ) -> Result<i64, AppError> {
        if estimate_pd < 0.0 { return Err(domain::DomainError::InvalidRatio(estimate_pd).into()); }
        if let (Some(s), Some(e)) = (start, end) {
            if e < s { return Err(domain::DomainError::InvalidDateWindow.into()); }
        }
        for &(_, min_prof, _, _) in skill_reqs {
            if !(1..=5).contains(&min_prof) {
                return Err(domain::DomainError::InvalidRatio(min_prof as f64).into());
            }
        }
        let input = TaskCreate {
            project_id, title, description, estimate_pd, start, end, is_long_term, sort_order,
            skill_reqs, tag_ids,
        };
        Ok(TasksRepo::create(pool, input).await?)
    }

    pub async fn list_by_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<Task>, AppError> {
        Ok(TasksRepo::list_by_project(pool, project_id).await?)
    }

    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), AppError> {
        match status {
            "todo" | "in_progress" | "blocked" | "review" | "done" | "cancelled" => {}
            _ => return Err(domain::DomainError::InvalidRatio(0.0).into()),
        }
        Ok(TasksRepo::set_status(pool, id, status).await?)
    }

    pub async fn skill_reqs(pool: &SqlitePool, task_id: i64) -> Result<Vec<TaskSkillRequirement>, AppError> {
        Ok(TasksRepo::list_skill_reqs(pool, task_id).await?)
    }

    pub async fn kanban(pool: &SqlitePool, project_id: i64) -> Result<Vec<KanbanTask>, AppError> {
        Ok(TasksRepo::list_kanban(pool, project_id).await?)
    }
}

use db::TaskDepsRepo;

impl TasksService {
    /// Add a dependency edge after checking it creates no cycle (design §3.3.12).
    pub async fn add_dependency(
        pool: &SqlitePool, task_id: i64, predecessor_id: i64, lag_days: i64,
    ) -> Result<(), AppError> {
        if task_id == predecessor_id {
            return Err(domain::DomainError::InvalidRatio(0.0).into()); // self-dep invalid
        }
        // Tentative new edge set: existing edges + the proposed one.
        let mut edges = TaskDepsRepo::all_edges(pool).await?;
        edges.push((task_id, predecessor_id));
        if has_cycle(&edges) {
            return Err(domain::DomainError::DependencyCycle(task_id).into());
        }
        TaskDepsRepo::add(pool, task_id, predecessor_id, lag_days).await?;
        Ok(())
    }
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
                if white.contains(&nb) { stack.push(nb); descended = true; break; }
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