use crate::error::DbError;
use sqlx::SqlitePool;
use tracing;

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
    pub embed_provider: String,
    pub embed_base_url: Option<String>,
    pub embed_api_key_enc: Option<String>,
    pub embed_model: String,
    pub embed_dim: i64,
    pub solver_backend: String,
    pub solver_timeout_ms: i64,
    pub locale: String,
    pub use_semantic_scorer: i64,
    pub use_llm_explainer: i64,
    pub ai_explanation_prompt: String,
    pub ai_explanation_preamble: String,
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
    pub embed_provider: Option<String>,
    pub embed_base_url: Option<Option<String>>,
    pub embed_api_key_enc: Option<Option<String>>,
    pub embed_model: Option<String>,
    pub embed_dim: Option<i64>,
    pub solver_backend: Option<String>,
    pub solver_timeout_ms: Option<i64>,
    pub locale: Option<String>,
    pub use_semantic_scorer: Option<i64>,
    pub use_llm_explainer: Option<i64>,
    pub ai_explanation_prompt: Option<String>,
    pub ai_explanation_preamble: Option<String>,
    pub overload_threshold: Option<f64>,
    pub underload_threshold: Option<f64>,
    pub utilization_green: Option<f64>,
    pub utilization_yellow: Option<f64>,
}

pub struct SettingsRepo;
impl SettingsRepo {
    /// Load the full global settings row.
    #[tracing::instrument(skip_all, level = "debug")]
    pub async fn get(pool: &SqlitePool) -> Result<SettingsRow, DbError> {
        Ok(sqlx::query_as(
            "SELECT id, default_unit, pd_hours, pm_workdays, ai_provider, ai_base_url, \
             ai_api_key_enc, secret_store, ai_chat_model, embed_provider, embed_base_url, \
             embed_api_key_enc, embed_model, embed_dim, solver_backend, solver_timeout_ms, locale, \
             use_semantic_scorer, use_llm_explainer, ai_explanation_prompt, ai_explanation_preamble, \
             overload_threshold, underload_threshold, utilization_green, utilization_yellow \
             FROM settings WHERE id = 1",
        )
        .fetch_one(pool)
        .await?)
    }

    /// Update the global settings row. Only fields set in `update` are written.
    #[tracing::instrument(skip_all, level = "debug")]
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
        if update.embed_provider.is_some() { sets.push("embed_provider = ?"); }
        if update.embed_base_url.is_some() { sets.push("embed_base_url = ?"); }
        if update.embed_api_key_enc.is_some() { sets.push("embed_api_key_enc = ?"); }
        if update.embed_model.is_some() { sets.push("embed_model = ?"); }
        if update.embed_dim.is_some() { sets.push("embed_dim = ?"); }
        if update.solver_backend.is_some() { sets.push("solver_backend = ?"); }
        if update.solver_timeout_ms.is_some() { sets.push("solver_timeout_ms = ?"); }
        if update.locale.is_some() { sets.push("locale = ?"); }
        if update.use_semantic_scorer.is_some() { sets.push("use_semantic_scorer = ?"); }
        if update.use_llm_explainer.is_some() { sets.push("use_llm_explainer = ?"); }
        if update.ai_explanation_prompt.is_some() { sets.push("ai_explanation_prompt = ?"); }
        if update.ai_explanation_preamble.is_some() { sets.push("ai_explanation_preamble = ?"); }
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
        if let Some(v) = &update.embed_provider { q = q.bind(v); }
        if let Some(v) = &update.embed_base_url { q = q.bind(v); }
        if let Some(v) = &update.embed_api_key_enc { q = q.bind(v); }
        if let Some(v) = &update.embed_model { q = q.bind(v); }
        if let Some(v) = &update.embed_dim { q = q.bind(v); }
        if let Some(v) = &update.solver_backend { q = q.bind(v); }
        if let Some(v) = &update.solver_timeout_ms { q = q.bind(v); }
        if let Some(v) = &update.locale { q = q.bind(v); }
        if let Some(v) = &update.use_semantic_scorer { q = q.bind(v); }
        if let Some(v) = &update.use_llm_explainer { q = q.bind(v); }
        if let Some(v) = &update.ai_explanation_prompt { q = q.bind(v); }
        if let Some(v) = &update.ai_explanation_preamble { q = q.bind(v); }
        if let Some(v) = &update.overload_threshold { q = q.bind(v); }
        if let Some(v) = &update.underload_threshold { q = q.bind(v); }
        if let Some(v) = &update.utilization_green { q = q.bind(v); }
        if let Some(v) = &update.utilization_yellow { q = q.bind(v); }

        q.execute(pool).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug")]
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
    #[tracing::instrument(skip_all, level = "debug")]
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
    #[tracing::instrument(skip_all, level = "debug")]
    pub async fn ai_settings(pool: &SqlitePool) -> Result<AiSettings, DbError> {
        let row: AiSettingsRow = sqlx::query_as(
            "SELECT ai_provider, ai_base_url, ai_api_key_enc, ai_chat_model, embed_provider, \
             embed_base_url, embed_api_key_enc, embed_model, embed_dim, solver_backend, \
             solver_timeout_ms, use_semantic_scorer, use_llm_explainer, use_llm_advisor, \
             ai_explanation_prompt, ai_explanation_preamble FROM settings WHERE id = 1",
        )
        .fetch_one(pool)
        .await?;
        Ok(AiSettings {
            chat: ChatLlmConfig {
                provider: row.ai_provider.unwrap_or_else(|| "ollama".into()),
                base_url: row.ai_base_url,
                api_key_enc: row.ai_api_key_enc,
                model: row.ai_chat_model.unwrap_or_else(|| "qwen2.5:7b".into()),
            },
            embed: EmbedLlmConfig {
                provider: row.embed_provider.unwrap_or_else(|| "ollama".into()),
                base_url: row.embed_base_url,
                api_key_enc: row.embed_api_key_enc,
                model: row.embed_model.unwrap_or_else(|| "nomic-embed-text".into()),
                dim: row.embed_dim.unwrap_or(768).max(0) as usize,
            },
            solver_backend: row.solver_backend.unwrap_or_else(|| "greedy".into()),
            solver_timeout_ms: row.solver_timeout_ms.unwrap_or(5000).max(0) as u64,
            use_semantic_scorer: row.use_semantic_scorer.unwrap_or(1) != 0,
            use_llm_explainer: row.use_llm_explainer.unwrap_or(1) != 0,
            use_llm_advisor: row.use_llm_advisor.unwrap_or(0) != 0,  // 默认关（0），与 use_llm_explainer 默认 1 相反
            explanation_prompt: row.ai_explanation_prompt,
            explanation_preamble: row.ai_explanation_preamble,
        })
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct AiSettingsRow {
    ai_provider: Option<String>,
    ai_base_url: Option<String>,
    ai_api_key_enc: Option<String>,
    ai_chat_model: Option<String>,
    embed_provider: Option<String>,
    embed_base_url: Option<String>,
    embed_api_key_enc: Option<String>,
    embed_model: Option<String>,
    embed_dim: Option<i64>,
    solver_backend: Option<String>,
    solver_timeout_ms: Option<i64>,
    use_semantic_scorer: Option<i64>,
    use_llm_explainer: Option<i64>,
    use_llm_advisor: Option<i64>,
    ai_explanation_prompt: Option<String>,
    ai_explanation_preamble: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AiSettings {
    pub chat: ChatLlmConfig,
    pub embed: EmbedLlmConfig,
    pub solver_backend: String,
    /// HiGHS time budget (ms) for the MILP solver; also the outer tokio::time::timeout budget.
    pub solver_timeout_ms: u64,
    /// Whether to use the LLM-based semantic scorer (embedding similarity) for resource-task fit.
    pub use_semantic_scorer: bool,
    /// Whether to use the LLM-based explainer for optimization result summaries.
    pub use_llm_explainer: bool,
    /// Whether to use the LLM-based advisor for structured optimization suggestions.
    pub use_llm_advisor: bool,
    /// User-defined prompt template for the LLM explainer.
    pub explanation_prompt: Option<String>,
    /// User-defined system preamble for the LLM explainer.
    pub explanation_preamble: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChatLlmConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key_enc: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct EmbedLlmConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key_enc: Option<String>,
    pub model: String,
    pub dim: usize,
}
