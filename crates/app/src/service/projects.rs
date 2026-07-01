use crate::error::AppError;
use db::models::Project;
use db::ProjectsRepo;
use sqlx::SqlitePool;

pub struct ProjectsService;

impl ProjectsService {
    #[tracing::instrument(skip(pool), fields(name = %name, priority = priority))]
    pub async fn create(
        pool: &SqlitePool, name: &str, description: Option<&str>,
        start: Option<&str>, end: Option<&str>, priority: i64, budget_pd: f64,
    ) -> Result<i64, AppError> {
        validate_priority(priority)?;
        if let (Some(s), Some(e)) = (start, end) {
            if e < s {
                tracing::warn!(start = %s, end = %e, "invalid project date window");
                return Err(domain::DomainError::InvalidDateWindow.into());
            }
        }
        let id = ProjectsRepo::create(pool, name, description, start, end, priority, budget_pd).await?;
        tracing::info!(project_id = id, "created project");
        Ok(id)
    }

    #[tracing::instrument(skip(pool))]
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Project>, AppError> {
        Ok(ProjectsRepo::list_active(pool).await?)
    }

    #[tracing::instrument(skip(pool), fields(project_id = id))]
    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Project, AppError> {
        Ok(ProjectsRepo::get(pool, id).await?)
    }

    #[tracing::instrument(skip(pool), fields(project_id = id))]
    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        ProjectsRepo::soft_delete(pool, id).await?;
        tracing::info!(project_id = id, "deleted project");
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(project_id = id, name = %name, priority = priority))]
    pub async fn update(
        pool: &SqlitePool, id: i64, name: &str, description: Option<&str>,
        start: Option<&str>, end: Option<&str>, priority: i64, budget_pd: f64,
    ) -> Result<(), AppError> {
        validate_priority(priority)?;
        if let (Some(s), Some(e)) = (start, end) {
            if e < s {
                tracing::warn!(project_id = id, start = %s, end = %e, "invalid project date window");
                return Err(domain::DomainError::InvalidDateWindow.into());
            }
        }
        ProjectsRepo::update(pool, id, name, description, start, end, priority, budget_pd).await?;
        tracing::info!(project_id = id, "updated project");
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(project_id = id, status = %status))]
    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), AppError> {
        match status {
            "planning" | "active" | "on_hold" | "done" | "cancelled" => {}
            _ => {
                tracing::warn!(project_id = id, status = %status, "invalid project status");
                return Err(domain::DomainError::InvalidStatus(status.into()).into());
            }
        }
        ProjectsRepo::set_status(pool, id, status).await?;
        tracing::info!(project_id = id, status = %status, "set project status");
        Ok(())
    }
}

#[tracing::instrument(fields(priority = p))]
fn validate_priority(p: i64) -> Result<(), AppError> {
    if !(1..=9).contains(&p) {
        tracing::warn!(priority = p, "invalid project priority");
        return Err(domain::DomainError::InvalidValue { field: "priority", value: p as f64 }.into());
    }
    Ok(())
}
