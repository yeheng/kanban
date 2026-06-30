use crate::error::AppError;
use db::models::TeamOverride;
use db::repo::settings::Thresholds;
use db::{SettingsRepo, TeamMembersRepo, TeamOverridesRepo};
use domain::UnitConfig;
use sqlx::SqlitePool;
use std::collections::HashMap;

/// The team override that applies to a resource — its team's override row, if any.
///
/// Single source of truth for the `team_of_resource` → `team_overrides` lookup, so every
/// effective-* resolver (thresholds, units) and the batch loops share one code path instead of
/// re-implementing the same two queries (design §3.3.8a).
pub async fn team_override_for(
    pool: &SqlitePool,
    resource_id: i64,
) -> Result<Option<TeamOverride>, AppError> {
    if let Some(team_id) = TeamMembersRepo::team_of_resource(pool, resource_id).await? {
        return Ok(TeamOverridesRepo::get(pool, team_id).await?);
    }
    Ok(None)
}

/// Resolve the four utilization thresholds: each field prefers the team override, falling back
/// to the global setting (design §3.3.8a). Pure — feed it a pre-loaded override for batch loops.
pub fn resolve_thresholds(global: Thresholds, o: Option<&TeamOverride>) -> Thresholds {
    match o {
        None => global,
        Some(o) => Thresholds {
            overload: o.overload_threshold.unwrap_or(global.overload),
            underload: o.underload_threshold.unwrap_or(global.underload),
            green: o.utilization_green.unwrap_or(global.green),
            yellow: o.utilization_yellow.unwrap_or(global.yellow),
        },
    }
}

/// Effective utilization thresholds for a resource (team override → global settings).
pub async fn effective_thresholds(
    pool: &SqlitePool,
    resource_id: i64,
) -> Result<Thresholds, AppError> {
    let global = SettingsRepo::thresholds(pool).await?;
    Ok(resolve_thresholds(global, team_override_for(pool, resource_id).await?.as_ref()))
}

/// Effective overload threshold for a resource (back-compat single-value accessor over
/// `effective_thresholds`).
pub async fn effective_overload(pool: &SqlitePool, resource_id: i64) -> Result<f64, AppError> {
    Ok(effective_thresholds(pool, resource_id).await?.overload)
}

/// Global PD/PM unit constants from `settings` (design §2.9: N fixed, default 20).
pub async fn global_unit_config(pool: &SqlitePool) -> Result<UnitConfig, AppError> {
    let u = SettingsRepo::unit_config(pool).await?;
    Ok(UnitConfig { hours_per_pd: u.pd_hours, pd_per_pm: u.pm_workdays })
}

/// Resolve PD/PM unit constants: each field prefers the team override, else global. Pure.
pub fn resolve_unit_config(global: UnitConfig, o: Option<&TeamOverride>) -> UnitConfig {
    match o {
        None => global,
        Some(o) => UnitConfig {
            hours_per_pd: o.pd_hours.unwrap_or(global.hours_per_pd),
            pd_per_pm: o.pm_workdays.unwrap_or(global.pd_per_pm),
        },
    }
}

/// Effective PD/PM unit constants for a resource: team override (pd_hours/pm_workdays) → global
/// settings → `UnitConfig::DEFAULT` (design §3.3.8a team-level override; §2.9 global default).
pub async fn effective_unit_config(
    pool: &SqlitePool,
    resource_id: i64,
) -> Result<UnitConfig, AppError> {
    let global = global_unit_config(pool).await?;
    Ok(resolve_unit_config(global, team_override_for(pool, resource_id).await?.as_ref()))
}

/// Utilization color band for a value against effective thresholds (design §4.9 Dashboard
/// bands). This is the SERVER-SIDE single source of truth: the frontend renders the returned
/// string directly instead of re-deriving the band from GLOBAL thresholds, so per-team
/// overrides actually take effect. Ordering matches the legacy frontend `band()` exactly.
pub fn band(util: f64, t: &Thresholds) -> &'static str {
    if util >= t.overload {
        "red"
    } else if util >= t.yellow {
        "yellow"
    } else if util >= t.green {
        "green"
    } else {
        "under"
    }
}

/// Effective thresholds for many resources in 3 queries total — NOT N+1 (design §4.9 <5ms
/// target). Loads global settings + every team membership + every team override once, then
/// resolves per resource. A resource with no team (or whose team has no override) maps to the
/// global thresholds. The membership tie-break (`lead` first, newest `joined_at`) mirrors
/// `TeamMembersRepo::team_of_resource`, so batch and single-resource resolution agree.
pub async fn effective_thresholds_map(
    pool: &SqlitePool,
    resource_ids: &[i64],
) -> Result<HashMap<i64, Thresholds>, AppError> {
    let global = SettingsRepo::thresholds(pool).await?;
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT resource_id, team_id FROM team_members \
         ORDER BY resource_id, (role = 'lead') DESC, joined_at DESC",
    )
    .fetch_all(pool)
    .await?;
    // Keep the FIRST (highest-priority) team per resource — matches team_of_resource's LIMIT 1.
    let mut team_of: HashMap<i64, i64> = HashMap::new();
    for (rid, tid) in rows {
        team_of.entry(rid).or_insert(tid);
    }
    let overrides: Vec<TeamOverride> = sqlx::query_as(
        "SELECT team_id, pd_hours, pm_workdays, overload_threshold, underload_threshold, \
         utilization_green, utilization_yellow FROM team_overrides",
    )
    .fetch_all(pool)
    .await?;
    let ov_of: HashMap<i64, TeamOverride> = overrides.into_iter().map(|o| (o.team_id, o)).collect();

    let mut out = HashMap::with_capacity(resource_ids.len());
    for &rid in resource_ids {
        let ov = team_of.get(&rid).and_then(|tid| ov_of.get(tid));
        out.insert(rid, resolve_thresholds(global, ov));
    }
    Ok(out)
}
