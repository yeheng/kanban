use crate::error::AppError;
use db::{SettingsRepo, TeamMembersRepo, TeamOverridesRepo};
use domain::UnitConfig;
use sqlx::SqlitePool;

/// Effective overload threshold for a resource:
/// team_overrides.overload_threshold (if the resource's team has one) → settings → 1.10.
/// (design §3.3.8a; confirmed #3: thresholds configurable per team/role.)
pub async fn effective_overload(pool: &SqlitePool, resource_id: i64) -> Result<f64, AppError> {
    if let Some(team_id) = TeamMembersRepo::team_of_resource(pool, resource_id).await? {
        if let Some(o) = TeamOverridesRepo::get(pool, team_id).await? {
            if let Some(t) = o.overload_threshold { return Ok(t); }
        }
    }
    Ok(SettingsRepo::thresholds(pool).await?.overload)
}

/// Like `effective_overload` but avoids re-querying global settings when called in a loop.
/// `global_threshold` should be `SettingsRepo::thresholds(pool).await?.overload` loaded once
/// before the loop.
pub async fn effective_overload_cached(
    pool: &SqlitePool,
    resource_id: i64,
    global_threshold: f64,
) -> Result<f64, AppError> {
    if let Some(team_id) = TeamMembersRepo::team_of_resource(pool, resource_id).await? {
        if let Some(o) = TeamOverridesRepo::get(pool, team_id).await? {
            if let Some(t) = o.overload_threshold { return Ok(t); }
        }
    }
    Ok(global_threshold)
}

/// Global PD/PM unit constants from `settings` (design §2.9: N fixed, default 20).
pub async fn global_unit_config(pool: &SqlitePool) -> Result<UnitConfig, AppError> {
    let u = SettingsRepo::unit_config(pool).await?;
    Ok(UnitConfig { hours_per_pd: u.pd_hours, pd_per_pm: u.pm_workdays })
}

/// Effective PD/PM unit constants for a resource: team_overrides.pd_hours/pm_workdays
/// (if the resource's team has them) → settings global → UnitConfig::DEFAULT
/// (design §3.3.8a team-level override; §2.9 global default).
pub async fn effective_unit_config(pool: &SqlitePool, resource_id: i64) -> Result<UnitConfig, AppError> {
    let global = SettingsRepo::unit_config(pool).await?;
    let (mut pd_hours, mut pm_workdays) = (global.pd_hours, global.pm_workdays);
    if let Some(team_id) = TeamMembersRepo::team_of_resource(pool, resource_id).await? {
        if let Some(o) = TeamOverridesRepo::get(pool, team_id).await? {
            if let Some(v) = o.pd_hours { pd_hours = v; }
            if let Some(v) = o.pm_workdays { pm_workdays = v; }
        }
    }
    Ok(UnitConfig { hours_per_pd: pd_hours, pd_per_pm: pm_workdays })
}

/// Like `effective_unit_config` but avoids re-querying global settings when called in a loop.
pub async fn effective_unit_config_cached(
    pool: &SqlitePool,
    resource_id: i64,
    global: UnitConfig,
) -> Result<UnitConfig, AppError> {
    if let Some(team_id) = TeamMembersRepo::team_of_resource(pool, resource_id).await? {
        if let Some(o) = TeamOverridesRepo::get(pool, team_id).await? {
            let pd_hours = o.pd_hours.unwrap_or(global.hours_per_pd);
            let pm_workdays = o.pm_workdays.unwrap_or(global.pd_per_pm);
            return Ok(UnitConfig { hours_per_pd: pd_hours, pd_per_pm: pm_workdays });
        }
    }
    Ok(global)
}
