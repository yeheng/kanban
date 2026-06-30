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

/// Global PD/PM unit constants (design §3.3.1 `settings`). Per-team overrides live in
/// `team_overrides` and are resolved by the app-layer resolver, not here.
#[derive(Debug, Clone, Copy)]
pub struct UnitRow {
    pub pd_hours: f64,
    pub pm_workdays: f64,
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

    /// Global PD/PM constants from `settings` (design §2.9 / §3.3.1). Defaults 8h/PD, 20 PD/PM.
    pub async fn unit_config(pool: &SqlitePool) -> Result<UnitRow, DbError> {
        let (pd_hours, pm_workdays): (Option<f64>, Option<f64>) = sqlx::query_as(
            "SELECT pd_hours, pm_workdays FROM settings WHERE id = 1",
        )
        .fetch_one(pool)
        .await?;
        Ok(UnitRow {
            pd_hours: pd_hours.unwrap_or(8.0),
            pm_workdays: pm_workdays.unwrap_or(20.0),
        })
    }

    /// AI provider/model + solver backend configuration (design §3.3.1 `settings`).
    /// Used by the optimization pipeline to pick the scorer/explainer and to persist the
    /// actual provider/backend used in each run row (instead of hardcoded literals).
    pub async fn ai_settings(pool: &SqlitePool) -> Result<AiSettings, DbError> {
        let (provider, base_url, chat_model, embed_model, solver_backend): (
            Option<String>, Option<String>, Option<String>, Option<String>, Option<String>,
        ) = sqlx::query_as(
            "SELECT ai_provider, ai_base_url, ai_chat_model, ai_embed_model, solver_backend \
             FROM settings WHERE id = 1",
        )
        .fetch_one(pool)
        .await?;
        Ok(AiSettings {
            provider: provider.unwrap_or_else(|| "ollama".into()),
            base_url,
            chat_model: chat_model.unwrap_or_else(|| "qwen2.5:7b".into()),
            embed_model: embed_model.unwrap_or_else(|| "nomic-embed-text".into()),
            solver_backend: solver_backend.unwrap_or_else(|| "greedy".into()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct AiSettings {
    pub provider: String,
    pub base_url: Option<String>,
    pub chat_model: String,
    pub embed_model: String,
    pub solver_backend: String,
}
