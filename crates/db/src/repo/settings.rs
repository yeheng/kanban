use crate::error::DbError;
use sqlx::SqlitePool;

/// Effective global utilization thresholds (design §3.3.8a). Per-team overrides live
/// in `team_overrides` and are resolved by the app-layer resolver, not here.
#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    pub overload: f64,
    pub underload: f64,
    pub green: f64,
    pub yellow: f64,
}

pub struct SettingsRepo;
impl SettingsRepo {
    pub async fn thresholds(pool: &SqlitePool) -> Result<Thresholds, DbError> {
        let (overload, underload, green, yellow): (Option<f64>, Option<f64>, Option<f64>, Option<f64>) =
            sqlx::query_as(
                "SELECT overload_threshold, underload_threshold, utilization_green, utilization_yellow \
                 FROM settings WHERE id = 1")
            .fetch_one(pool).await?;
        Ok(Thresholds {
            overload: overload.unwrap_or(1.10),
            underload: underload.unwrap_or(0.50),
            green: green.unwrap_or(0.70),
            yellow: yellow.unwrap_or(1.00),
        })
    }
}
