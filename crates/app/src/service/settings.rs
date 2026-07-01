use crate::error::AppError;
use db::repo::settings::{SettingsRow, SettingsUpdate};
use db::{SettingsRepo, SqlitePool};

/// Frontend-facing global settings DTO. Mirrors `settings` columns except `id`/`updated_at`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SettingsDto {
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
    pub use_semantic_scorer: bool,
    pub use_llm_explainer: bool,
    pub use_llm_advisor: bool,
    pub ai_explanation_prompt: String,
    pub ai_explanation_preamble: String,
    pub overload_threshold: f64,
    pub underload_threshold: f64,
    pub utilization_green: f64,
    pub utilization_yellow: f64,
}

impl From<SettingsRow> for SettingsDto {
    fn from(row: SettingsRow) -> Self {
        Self {
            default_unit: row.default_unit,
            pd_hours: row.pd_hours,
            pm_workdays: row.pm_workdays,
            ai_provider: row.ai_provider,
            ai_base_url: row.ai_base_url,
            ai_api_key_enc: row.ai_api_key_enc,
            secret_store: row.secret_store,
            ai_chat_model: row.ai_chat_model,
            embed_provider: row.embed_provider,
            embed_base_url: row.embed_base_url,
            embed_api_key_enc: row.embed_api_key_enc,
            embed_model: row.embed_model,
            embed_dim: row.embed_dim,
            solver_backend: row.solver_backend,
            solver_timeout_ms: row.solver_timeout_ms,
            locale: row.locale,
            use_semantic_scorer: row.use_semantic_scorer != 0,
            use_llm_explainer: row.use_llm_explainer != 0,
            use_llm_advisor: row.use_llm_advisor != 0,
            ai_explanation_prompt: row.ai_explanation_prompt,
            ai_explanation_preamble: row.ai_explanation_preamble,
            overload_threshold: row.overload_threshold.unwrap_or(1.10),
            underload_threshold: row.underload_threshold.unwrap_or(0.50),
            utilization_green: row.utilization_green.unwrap_or(0.70),
            utilization_yellow: row.utilization_yellow.unwrap_or(1.00),
        }
    }
}

pub struct SettingsService;

impl SettingsService {
    #[tracing::instrument(skip(pool))]
    pub async fn get(pool: &SqlitePool) -> Result<SettingsDto, AppError> {
        let row = SettingsRepo::get(pool).await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(pool))]
    pub async fn update(pool: &SqlitePool, dto: SettingsDto) -> Result<(), AppError> {
        validate(&dto)?;
        SettingsRepo::update(
            pool,
            &SettingsUpdate {
                default_unit: Some(dto.default_unit),
                pd_hours: Some(dto.pd_hours),
                pm_workdays: Some(dto.pm_workdays),
                ai_provider: Some(dto.ai_provider),
                ai_base_url: Some(dto.ai_base_url),
                ai_api_key_enc: Some(dto.ai_api_key_enc),
                secret_store: Some(dto.secret_store),
                ai_chat_model: Some(dto.ai_chat_model),
                embed_provider: Some(dto.embed_provider),
                embed_base_url: Some(dto.embed_base_url),
                embed_api_key_enc: Some(dto.embed_api_key_enc),
                embed_model: Some(dto.embed_model),
                embed_dim: Some(dto.embed_dim),
                solver_backend: Some(dto.solver_backend),
                solver_timeout_ms: Some(dto.solver_timeout_ms),
                locale: Some(dto.locale),
                use_semantic_scorer: Some(i64::from(dto.use_semantic_scorer)),
                use_llm_explainer: Some(i64::from(dto.use_llm_explainer)),
                use_llm_advisor: Some(i64::from(dto.use_llm_advisor)),
                ai_explanation_prompt: Some(dto.ai_explanation_prompt),
                ai_explanation_preamble: Some(dto.ai_explanation_preamble),
                overload_threshold: Some(dto.overload_threshold),
                underload_threshold: Some(dto.underload_threshold),
                utilization_green: Some(dto.utilization_green),
                utilization_yellow: Some(dto.utilization_yellow),
            },
        )
        .await?;
        tracing::info!("updated settings");
        Ok(())
    }
}

#[tracing::instrument]
fn validate(dto: &SettingsDto) -> Result<(), AppError> {
    if !["PD", "PM"].contains(&dto.default_unit.as_str()) {
        tracing::warn!(default_unit = %dto.default_unit, "invalid default_unit");
        return Err(AppError::validation(format!(
            "invalid default_unit: {}",
            dto.default_unit
        )));
    }
    if dto.pd_hours <= 0.0 {
        tracing::warn!(pd_hours = dto.pd_hours, "invalid pd_hours");
        return Err(domain::DomainError::InvalidValue {
            field: "pd_hours",
            value: dto.pd_hours,
        }
        .into());
    }
    if dto.pm_workdays <= 0.0 {
        tracing::warn!(pm_workdays = dto.pm_workdays, "invalid pm_workdays");
        return Err(domain::DomainError::InvalidValue {
            field: "pm_workdays",
            value: dto.pm_workdays,
        }
        .into());
    }
    if dto.overload_threshold <= 0.0 {
        tracing::warn!(overload_threshold = dto.overload_threshold, "invalid overload_threshold");
        return Err(domain::DomainError::InvalidValue {
            field: "overload_threshold",
            value: dto.overload_threshold,
        }
        .into());
    }
    if dto.underload_threshold < 0.0 {
        tracing::warn!(underload_threshold = dto.underload_threshold, "invalid underload_threshold");
        return Err(domain::DomainError::InvalidValue {
            field: "underload_threshold",
            value: dto.underload_threshold,
        }
        .into());
    }
    if !(0.0..=1.0).contains(&dto.utilization_green) {
        tracing::warn!(utilization_green = dto.utilization_green, "invalid utilization_green");
        return Err(domain::DomainError::InvalidRatio(dto.utilization_green).into());
    }
    if !(0.0..=1.0).contains(&dto.utilization_yellow) {
        tracing::warn!(utilization_yellow = dto.utilization_yellow, "invalid utilization_yellow");
        return Err(domain::DomainError::InvalidRatio(dto.utilization_yellow).into());
    }
    if dto.solver_timeout_ms <= 0 {
        tracing::warn!(solver_timeout_ms = dto.solver_timeout_ms, "invalid solver_timeout_ms");
        return Err(AppError::validation(format!(
            "solver_timeout_ms must be positive: {}",
            dto.solver_timeout_ms
        )));
    }
    if dto.embed_dim <= 0 {
        tracing::warn!(embed_dim = dto.embed_dim, "invalid embed_dim");
        return Err(AppError::validation(format!(
            "embed_dim must be positive: {}",
            dto.embed_dim
        )));
    }
    if !["ollama", "openai", "anthropic", "deepseek"].contains(&dto.ai_provider.as_str()) {
        tracing::warn!(ai_provider = %dto.ai_provider, "invalid ai_provider");
        return Err(AppError::validation(format!(
            "invalid ai_provider: {}",
            dto.ai_provider
        )));
    }
    if !["ollama", "openai", "anthropic", "deepseek"].contains(&dto.embed_provider.as_str()) {
        tracing::warn!(embed_provider = %dto.embed_provider, "invalid embed_provider");
        return Err(AppError::validation(format!(
            "invalid embed_provider: {}",
            dto.embed_provider
        )));
    }
    if !["keychain", "encrypted_file"].contains(&dto.secret_store.as_str()) {
        tracing::warn!(secret_store = %dto.secret_store, "invalid secret_store");
        return Err(AppError::validation(format!(
            "invalid secret_store: {}",
            dto.secret_store
        )));
    }
    if !["good_lp", "greedy", "hungarian"].contains(&dto.solver_backend.as_str()) {
        tracing::warn!(solver_backend = %dto.solver_backend, "invalid solver_backend");
        return Err(AppError::validation(format!(
            "invalid solver_backend: {}",
            dto.solver_backend
        )));
    }
    Ok(())
}
