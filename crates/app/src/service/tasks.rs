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