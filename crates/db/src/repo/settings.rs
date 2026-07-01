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

/// Full global settings row (design §3.3.1). There is always exactly one row with id = 1.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SettingsRow {
    pub id: i64,
    pub default_unit: String,
    pub pd_hours: f64,
    pub pm_workdays: f64,
    pub ai_provider: String,
    pub ai_base_url: Option<String>,
    pub ai_api_key_enc: Option<String>,
    pub secret_store: String,
    pub ai_chat_model: String,
    pub ai_embed_model: String,
    pub ai_embed_dim: i64,
    pub solver_backend: String,
    pub solver_timeout_ms: i64,
    pub locale: String,
    pub overload_threshold: Option<f64>,
    pub underload_threshold: Option<f64>,
    pub utilization_green: Option<f64>,
    pub utilization_yellow: Option<f64>,
}

/// Editable subset of `settings`. All fields are optional so callers can send only what
/// changed; `SettingsRepo::update` writes only the provided fields.
#[derive(Debug, Clone, Default)]
pub struct SettingsUpdate {
    pub default_unit: Option<String>,
    pub pd_hours: Option<f64>,
    pub pm_workdays: Option<f64>,
    pub ai_provider: Option<String>,
    pub ai_base_url: Option<Option<String>>,
    pub ai_api_key_enc: Option<Option<String>>,
    pub secret_store: Option<String>,
    pub ai_chat_model: Option<String>,
    pub ai_embed_model: Option<String>,
    pub ai_embed_dim: Option<i64>,
    pub solver_backend: Option<String>,
    pub solver_timeout_ms: Option<i64>,
    pub locale: Option<String>,
    pub overload_threshold: Option<f64>,
    pub underload_threshold: Option<f64>,
    pub utilization_green: Option<f64>,
    pub utilization_yellow: Option<f64>,
}

pub struct SettingsRepo;
impl SettingsRepo {
    /// Load the full global settings row.
    pub async fn get(pool: &SqlitePool) -> Result<SettingsRow, DbError> {
        Ok(sqlx::query_as(
            "SELECT id, default_unit, pd_hours, pm_workdays, ai_provider, ai_base_url, \
             ai_api_key_enc, secret_store, ai_chat_model, ai_embed_model, ai_embed_dim, \
             solver_backend, solver_timeout_ms, locale, overload_threshold, underload_threshold, \
             utilization_green, utilization_yellow FROM settings WHERE id = 1",
        )
        .fetch_one(pool)
        .await?)
    }

    /// Update the global settings row. Only fields set in `update` are written.
    pub async fn update(pool: &SqlitePool, update: &SettingsUpdate) -> Result<(), DbError> {
        let mut sets: Vec<&'static str> = Vec::new();
        if update.default_unit.is_some() { sets.push("default_unit = ?"); }
        if update.pd_hours.is_some() { sets.push("pd_hours = ?"); }
        if update.pm_workdays.is_some() { sets.push("pm_workdays = ?"); }
        if update.ai_provider.is_some() { sets.push("ai_provider = ?"); }
        if update.ai_base_url.is_some() { sets.push("ai_base_url = ?"); }
        if update.ai_api_key_enc.is_some() { sets.push("ai_api_key_enc = ?"); }
        if update.secret_store.is_some() { sets.push("secret_store = ?"); }
        if update.ai_chat_model.is_some() { sets.push("ai_chat_model = ?"); }
        if update.ai_embed_model.is_some() { sets.push("ai_embed_model = ?"); }
        if update.ai_embed_dim.is_some() { sets.push("ai_embed_dim = ?"); }
        if update.solver_backend.is_some() { sets.push("solver_backend = ?"); }
        if update.solver_timeout_ms.is_some() { sets.push("solver_timeout_ms = ?"); }
        if update.locale.is_some() { sets.push("locale = ?"); }
        if update.overload_threshold.is_some() { sets.push("overload_threshold = ?"); }
        if update.underload_threshold.is_some() { sets.push("underload_threshold = ?"); }
        if update.utilization_green.is_some() { sets.push("utilization_green = ?"); }
        if update.utilization_yellow.is_some() { sets.push("utilization_yellow = ?"); }

        if sets.is_empty() {
            return Ok(());
        }

        let sql = format!(
            "UPDATE settings SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = 1",
            sets.join(", ")
        );

        let mut q = sqlx::query(&sql);
        if let Some(v) = &update.default_unit { q = q.bind(v); }
        if let Some(v) = &update.pd_hours { q = q.bind(v); }
        if let Some(v) = &update.pm_workdays { q = q.bind(v); }
        if let Some(v) = &update.ai_provider { q = q.bind(v); }
        if let Some(v) = &update.ai_base_url { q = q.bind(v); }
        if let Some(v) = &update.ai_api_key_enc { q = q.bind(v); }
        if let Some(v) = &update.secret_store { q = q.bind(v); }
        if let Some(v) = &update.ai_chat_model { q = q.bind(v); }
        if let Some(v) = &update.ai_embed_model { q = q.bind(v); }
        if let Some(v) = &update.ai_embed_dim { q = q.bind(v); }
        if let Some(v) = &update.solver_backend { q = q.bind(v); }
        if let Some(v) = &update.solver_timeout_ms { q = q.bind(v); }
        if let Some(v) = &update.locale { q = q.bind(v); }
        if let Some(v) = &update.overload_threshold { q = q.bind(v); }
        if let Some(v) = &update.underload_threshold { q = q.bind(v); }
        if let Some(v) = &update.utilization_green { q = q.bind(v); }
        if let Some(v) = &update.utilization_yellow { q = q.bind(v); }

        q.execute(pool).await?;
        Ok(())
    }

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
