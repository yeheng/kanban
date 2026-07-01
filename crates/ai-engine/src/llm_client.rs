//! Multi-provider LLM client helpers used by the semantic scorer and LLM explainer.
//!
//! Currently supported providers:
//! - Embeddings: ollama, openai
//! - Chat completions: ollama, openai, anthropic, deepseek
//!
//! The `provider` field is matched case-insensitively; any unsupported provider or
//! configuration error degrades gracefully (returns `None`) so the engine never
//! panics when LLM settings are misconfigured.

use rig::client::{CompletionClient, EmbeddingsClient};
use rig::completion::Prompt;
use rig::embeddings::EmbeddingModel;
use rig::providers::{anthropic, deepseek, ollama, openai};

/// Runtime configuration for one LLM provider/model pair.
#[derive(Debug, Clone)]
pub struct LlmClientConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
}

/// Embed a single piece of text. Returns `None` on any provider error so the caller
/// can degrade gracefully (e.g. leave the score matrix entry empty).
pub async fn embed_text(cfg: &LlmClientConfig, text: &str) -> Option<Vec<f64>> {
    match cfg.provider.to_lowercase().as_str() {
        "ollama" => embed_ollama(cfg, text).await,
        "openai" => embed_openai(cfg, text).await,
        _ => None,
    }
}

/// Send a one-shot chat prompt and return the model's text response. Returns `None`
/// on any provider error so the caller can fall back to a template explanation.
pub async fn completion_prompt(cfg: &LlmClientConfig, preamble: &str, prompt: &str) -> Option<String> {
    match cfg.provider.to_lowercase().as_str() {
        "ollama" => prompt_ollama(cfg, preamble, prompt).await,
        "openai" => prompt_openai(cfg, preamble, prompt).await,
        "anthropic" => prompt_anthropic(cfg, preamble, prompt).await,
        "deepseek" => prompt_deepseek(cfg, preamble, prompt).await,
        _ => None,
    }
}

async fn embed_ollama(cfg: &LlmClientConfig, text: &str) -> Option<Vec<f64>> {
    let mut builder = ollama::Client::builder().api_key(cfg.api_key.as_deref().unwrap_or(""));
    if let Some(url) = &cfg.base_url {
        builder = builder.base_url(url);
    }
    tracing::debug!(provider = "ollama", model = %cfg.model, input = %text, "embed request");
    let client = builder.build().ok()?;
    let resp = client.embedding_model(&cfg.model).embed_text(text).await.ok().map(|e| e.vec);
    match &resp {
        Some(v) => tracing::debug!(provider = "ollama", dim = v.len(), "embed response"),
        None => tracing::debug!(provider = "ollama", "embed response: None (degraded)"),
    }
    resp
}

async fn embed_openai(cfg: &LlmClientConfig, text: &str) -> Option<Vec<f64>> {
    let key = cfg.api_key.as_deref()?;
    let mut builder = openai::Client::builder().api_key(key);
    if let Some(url) = &cfg.base_url {
        builder = builder.base_url(url);
    }
    tracing::debug!(provider = "openai", model = %cfg.model, input = %text, "embed request");
    let client = builder.build().ok()?;
    let resp = client.embedding_model(&cfg.model).embed_text(text).await.ok().map(|e| e.vec);
    match &resp {
        Some(v) => tracing::debug!(provider = "openai", dim = v.len(), "embed response"),
        None => tracing::debug!(provider = "openai", "embed response: None (degraded)"),
    }
    resp
}

async fn prompt_ollama(cfg: &LlmClientConfig, preamble: &str, prompt: &str) -> Option<String> {
    let mut builder = ollama::Client::builder().api_key(cfg.api_key.as_deref().unwrap_or(""));
    if let Some(url) = &cfg.base_url {
        builder = builder.base_url(url);
    }
    tracing::debug!(provider = "ollama", model = %cfg.model, %preamble, prompt = %prompt, "chat request");
    let client = builder.build().ok()?;
    let agent = rig::agent::AgentBuilder::new(client.completion_model(&cfg.model))
        .preamble(preamble)
        .build();
    let resp = agent.prompt(prompt).await.ok().map(|s| s.trim().to_owned());
    match &resp {
        Some(s) => tracing::debug!(provider = "ollama", response_len = s.len(), response = %s, "chat response"),
        None => tracing::debug!(provider = "ollama", "chat response: None (degraded)"),
    }
    resp
}

async fn prompt_openai(cfg: &LlmClientConfig, preamble: &str, prompt: &str) -> Option<String> {
    let key = cfg.api_key.as_deref()?;
    let mut builder = openai::Client::builder().api_key(key);
    if let Some(url) = &cfg.base_url {
        builder = builder.base_url(url);
    }
    tracing::debug!(provider = "openai", model = %cfg.model, %preamble, prompt = %prompt, "chat request");
    let client = builder.build().ok()?;
    let agent = rig::agent::AgentBuilder::new(client.completion_model(&cfg.model))
        .preamble(preamble)
        .build();
    let resp = agent.prompt(prompt).await.ok().map(|s| s.trim().to_owned());
    match &resp {
        Some(s) => tracing::debug!(provider = "openai", response_len = s.len(), response = %s, "chat response"),
        None => tracing::debug!(provider = "openai", "chat response: None (degraded)"),
    }
    resp
}

async fn prompt_anthropic(cfg: &LlmClientConfig, preamble: &str, prompt: &str) -> Option<String> {
    let key = cfg.api_key.as_deref()?;
    let mut builder = anthropic::Client::builder().api_key(key);
    if let Some(url) = &cfg.base_url {
        builder = builder.base_url(url);
    }
    tracing::debug!(provider = "anthropic", model = %cfg.model, %preamble, prompt = %prompt, "chat request");
    let client = builder.build().ok()?;
    let agent = rig::agent::AgentBuilder::new(client.completion_model(&cfg.model))
        .preamble(preamble)
        .build();
    let resp = agent.prompt(prompt).await.ok().map(|s| s.trim().to_owned());
    match &resp {
        Some(s) => tracing::debug!(provider = "anthropic", response_len = s.len(), response = %s, "chat response"),
        None => tracing::debug!(provider = "anthropic", "chat response: None (degraded)"),
    }
    resp
}

async fn prompt_deepseek(cfg: &LlmClientConfig, preamble: &str, prompt: &str) -> Option<String> {
    let key = cfg.api_key.as_deref()?;
    let mut builder = deepseek::Client::builder().api_key(key);
    if let Some(url) = &cfg.base_url {
        builder = builder.base_url(url);
    }
    tracing::debug!(provider = "deepseek", model = %cfg.model, %preamble, prompt = %prompt, "chat request");
    let client = builder.build().ok()?;
    let agent = rig::agent::AgentBuilder::new(client.completion_model(&cfg.model))
        .preamble(preamble)
        .build();
    let resp = agent.prompt(prompt).await.ok().map(|s| s.trim().to_owned());
    match &resp {
        Some(s) => tracing::debug!(provider = "deepseek", response_len = s.len(), response = %s, "chat response"),
        None => tracing::debug!(provider = "deepseek", "chat response: None (degraded)"),
    }
    resp
}
