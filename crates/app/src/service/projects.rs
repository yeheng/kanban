use crate::error::AppError;
use db::models::Project;
use db::ProjectsRepo;
use sqlx::SqlitePool;

pub struct ProjectsService;

impl ProjectsService {
    pub async fn create(
        pool: &SqlitePool, name: &str, description: Option<&str>,
        start: Option<&str>, end: Option<&str>, priority: i64, budget_pd: f64,
    ) -> Result<i64, AppError> {
        validate_priority(priority)?;
        if let (Some(s), Some(e)) = (start, end) {
            if e < s { return Err(domain::DomainError::InvalidDateWindow.into()); }
        }
        Ok(ProjectsRepo::create(pool, name, description, start, end, priority, budget_pd).await?)
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Project>, AppError> {
        Ok(ProjectsRepo::list_active(pool).await?)
    }

    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Project, AppError> {
        Ok(ProjectsRepo::get(pool, id).await?)
    }

    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        Ok(ProjectsRepo::soft_delete(pool, id).await?)
    }

    pub async fn update(
        pool: &SqlitePool, id: i64, name: &str, description: Option<&str>,
        start: Option<&str>, end: Option<&str>, priority: i64, budget_pd: f64,
    ) -> Result<(), AppError> {
        validate_priority(priority)?;
        if let (Some(s), Some(e)) = (start, end) {
            if e < s { return Err(domain::DomainError::InvalidDateWindow.into()); }
        }
        Ok(ProjectsRepo::update(pool, id, name, description, start, end, priority, budget_pd).await?)
    }

    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), AppError> {
        match status {
            "planning" | "active" | "on_hold" | "done" | "cancelled" => {}
            _ => return Err(domain::DomainError::InvalidRatio(0.0).into()),
        }
        Ok(ProjectsRepo::set_status(pool, id, status).await?)
    }
}

fn validate_priority(p: i64) -> Result<(), AppError> {
    if !(1..=9).contains(&p) {
        return Err(domain::DomainError::InvalidRatio(p as f64).into());
    }
    Ok(())
}
