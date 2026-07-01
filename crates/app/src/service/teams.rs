use crate::error::AppError;
use db::models::{Team, TeamMember, TeamOverride};
use db::{TeamMembersRepo, TeamOverridesRepo, TeamsRepo};
use sqlx::SqlitePool;

pub struct TeamsService;

impl TeamsService {
    #[tracing::instrument(skip(pool), fields(name = %name))]
    pub async fn create(pool: &SqlitePool, name: &str, description: Option<&str>) -> Result<i64, AppError> {
        let id = TeamsRepo::create(pool, name, description).await?;
        tracing::info!(team_id = id, name = %name, "created team");
        Ok(id)
    }

    #[tracing::instrument(skip(pool))]
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Team>, AppError> {
        Ok(TeamsRepo::list_active(pool).await?)
    }

    #[tracing::instrument(skip(pool), fields(team_id = id))]
    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        TeamsRepo::soft_delete(pool, id).await?;
        tracing::info!(team_id = id, "deleted team");
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(team_id = team_id, resource_id = resource_id))]
    pub async fn add_member(pool: &SqlitePool, team_id: i64, resource_id: i64, role: Option<&str>) -> Result<(), AppError> {
        TeamMembersRepo::add(pool, team_id, resource_id, role).await?;
        tracing::info!(team_id = team_id, resource_id = resource_id, role = ?role, "added team member");
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(team_id = team_id, resource_id = resource_id))]
    pub async fn remove_member(pool: &SqlitePool, team_id: i64, resource_id: i64) -> Result<(), AppError> {
        TeamMembersRepo::remove(pool, team_id, resource_id).await?;
        tracing::info!(team_id = team_id, resource_id = resource_id, "removed team member");
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(team_id = team_id))]
    pub async fn members(pool: &SqlitePool, team_id: i64) -> Result<Vec<TeamMember>, AppError> {
        Ok(TeamMembersRepo::list_members(pool, team_id).await?)
    }

    #[tracing::instrument(skip(pool), fields(team_id = o.team_id))]
    pub async fn set_override(pool: &SqlitePool, o: TeamOverride) -> Result<(), AppError> {
        validate_override(&o)?;
        TeamOverridesRepo::upsert(pool, &o).await?;
        tracing::info!(team_id = o.team_id, "set team override");
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(team_id = team_id))]
    pub async fn get_override(pool: &SqlitePool, team_id: i64) -> Result<Option<TeamOverride>, AppError> {
        Ok(TeamOverridesRepo::get(pool, team_id).await?)
    }
}

#[tracing::instrument(fields(team_id = o.team_id))]
fn validate_override(o: &TeamOverride) -> Result<(), AppError> {
    if let Some(g) = o.utilization_green { if !(0.0..=1.0).contains(&g) {
        tracing::warn!(team_id = o.team_id, utilization_green = g, "invalid utilization green");
        return Err(domain::DomainError::InvalidRatio(g).into());
    } }
    if let Some(y) = o.utilization_yellow { if !(0.0..=1.0).contains(&y) {
        tracing::warn!(team_id = o.team_id, utilization_yellow = y, "invalid utilization yellow");
        return Err(domain::DomainError::InvalidRatio(y).into());
    } }
    if let Some(p) = o.pd_hours { if p <= 0.0 {
        tracing::warn!(team_id = o.team_id, pd_hours = p, "invalid pd_hours");
        return Err(domain::DomainError::InvalidValue { field: "pd_hours", value: p }.into());
    } }
    if let Some(p) = o.pm_workdays { if p <= 0.0 {
        tracing::warn!(team_id = o.team_id, pm_workdays = p, "invalid pm_workdays");
        return Err(domain::DomainError::InvalidValue { field: "pm_workdays", value: p }.into());
    } }
    if let Some(t) = o.overload_threshold { if t <= 0.0 {
        tracing::warn!(team_id = o.team_id, overload_threshold = t, "invalid overload threshold");
        return Err(domain::DomainError::InvalidValue { field: "overload_threshold", value: t }.into());
    } }
    if let Some(t) = o.underload_threshold { if t < 0.0 {
        tracing::warn!(team_id = o.team_id, underload_threshold = t, "invalid underload threshold");
        return Err(domain::DomainError::InvalidValue { field: "underload_threshold", value: t }.into());
    } }
    Ok(())
}
