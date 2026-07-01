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
            overload_threshold: row.overload_threshold.unwrap_or(1.10),
            underload_threshold: row.underload_threshold.unwrap_or(0.50),
            utilization_green: row.utilization_green.unwrap_or(0.70),
            utilization_yellow: row.utilization_yellow.unwrap_or(1.00),
        }
    }
}

pub struct SettingsService;

impl SettingsService {
    pub async fn get(pool: &SqlitePool) -> Result<SettingsDto, AppError> {
        let row = SettingsRepo::get(pool).await?;
        Ok(row.into())
    }

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
                overload_threshold: Some(dto.overload_threshold),
                underload_threshold: Some(dto.underload_threshold),
                utilization_green: Some(dto.utilization_green),
                utilization_yellow: Some(dto.utilization_yellow),
            },
        )
        .await?;
        Ok(())
    }
}

fn validate(dto: &SettingsDto) -> Result<(), AppError> {
    if !["PD", "PM"].contains(&dto.default_unit.as_str()) {
        return Err(AppError::validation(format!(
            "invalid default_unit: {}",
            dto.default_unit
        )));
    }
    if dto.pd_hours <= 0.0 {
        return Err(domain::DomainError::InvalidValue {
            field: "pd_hours",
            value: dto.pd_hours,
        }
        .into());
    }
    if dto.pm_workdays <= 0.0 {
        return Err(domain::DomainError::InvalidValue {
            field: "pm_workdays",
            value: dto.pm_workdays,
        }
        .into());
    }
    if dto.overload_threshold <= 0.0 {
        return Err(domain::DomainError::InvalidValue {
            field: "overload_threshold",
            value: dto.overload_threshold,
        }
        .into());
    }
    if dto.underload_threshold < 0.0 {
        return Err(domain::DomainError::InvalidValue {
            field: "underload_threshold",
            value: dto.underload_threshold,
        }
        .into());
    }
    if !(0.0..=1.0).contains(&dto.utilization_green) {
        return Err(domain::DomainError::InvalidRatio(dto.utilization_green).into());
    }
    if !(0.0..=1.0).contains(&dto.utilization_yellow) {
        return Err(domain::DomainError::InvalidRatio(dto.utilization_yellow).into());
    }
    if dto.solver_timeout_ms <= 0 {
        return Err(AppError::validation(format!(
            "solver_timeout_ms must be positive: {}",
            dto.solver_timeout_ms
        )));
    }
    if dto.embed_dim <= 0 {
        return Err(AppError::validation(format!(
            "embed_dim must be positive: {}",
            dto.embed_dim
        )));
    }
    if !["ollama", "openai", "anthropic", "deepseek"].contains(&dto.ai_provider.as_str()) {
        return Err(AppError::validation(format!(
            "invalid ai_provider: {}",
            dto.ai_provider
        )));
    }
    if !["ollama", "openai", "anthropic", "deepseek"].contains(&dto.embed_provider.as_str()) {
        return Err(AppError::validation(format!(
            "invalid embed_provider: {}",
            dto.embed_provider
        )));
    }
    if !["keychain", "encrypted_file"].contains(&dto.secret_store.as_str()) {
        return Err(AppError::validation(format!(
            "invalid secret_store: {}",
            dto.secret_store
        )));
    }
    if !["good_lp", "greedy", "hungarian"].contains(&dto.solver_backend.as_str()) {
        return Err(AppError::validation(format!(
            "invalid solver_backend: {}",
            dto.solver_backend
        )));
    }
    Ok(())
}
