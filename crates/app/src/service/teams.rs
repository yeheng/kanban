use crate::error::AppError;
use db::models::{Team, TeamMember, TeamOverride};
use db::{TeamMembersRepo, TeamOverridesRepo, TeamsRepo};
use sqlx::SqlitePool;

pub struct TeamsService;

impl TeamsService {
    pub async fn create(pool: &SqlitePool, name: &str, description: Option<&str>) -> Result<i64, AppError> {
        Ok(TeamsRepo::create(pool, name, description).await?)
    }
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Team>, AppError> {
        Ok(TeamsRepo::list_active(pool).await?)
    }
    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        Ok(TeamsRepo::soft_delete(pool, id).await?)
    }
    pub async fn add_member(pool: &SqlitePool, team_id: i64, resource_id: i64, role: Option<&str>) -> Result<(), AppError> {
        Ok(TeamMembersRepo::add(pool, team_id, resource_id, role).await?)
    }
    pub async fn members(pool: &SqlitePool, team_id: i64) -> Result<Vec<TeamMember>, AppError> {
        Ok(TeamMembersRepo::list_members(pool, team_id).await?)
    }
    pub async fn set_override(pool: &SqlitePool, o: TeamOverride) -> Result<(), AppError> {
        validate_override(&o)?;
        Ok(TeamOverridesRepo::upsert(pool, &o).await?)
    }
    pub async fn get_override(pool: &SqlitePool, team_id: i64) -> Result<Option<TeamOverride>, AppError> {
        Ok(TeamOverridesRepo::get(pool, team_id).await?)
    }
}

fn validate_override(o: &TeamOverride) -> Result<(), AppError> {
    if let Some(g) = o.utilization_green { if !(0.0..=1.0).contains(&g) { return Err(domain::DomainError::InvalidRatio(g).into()); } }
    if let Some(y) = o.utilization_yellow { if !(0.0..=1.0).contains(&y) { return Err(domain::DomainError::InvalidRatio(y).into()); } }
    if let Some(p) = o.pd_hours { if p <= 0.0 { return Err(domain::DomainError::InvalidRatio(p).into()); } }
    if let Some(p) = o.pm_workdays { if p <= 0.0 { return Err(domain::DomainError::InvalidRatio(p).into()); } }
    if let Some(t) = o.overload_threshold { if t <= 0.0 { return Err(domain::DomainError::InvalidRatio(t).into()); } }
    if let Some(t) = o.underload_threshold { if t < 0.0 { return Err(domain::DomainError::InvalidRatio(t).into()); } }
    Ok(())
}
