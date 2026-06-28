use crate::error::AppError;
use db::{SettingsRepo, TeamMembersRepo, TeamOverridesRepo};
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
