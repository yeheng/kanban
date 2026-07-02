use app::service::settings::{SettingsDto, SettingsService};
use app::service::thresholds::{global_unit_config, effective_thresholds_map};
use db::pool::connect;
use db::ResourcesRepo;

#[tokio::test]
async fn get_settings_returns_defaults() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    let s = SettingsService::get(&pool).await.unwrap();
    assert_eq!(s.default_unit, "PD");
    assert!((s.pd_hours - 8.0).abs() < 1e-9);
    assert!((s.pm_workdays - 20.0).abs() < 1e-9);
    assert_eq!(s.ai_provider, "ollama");
    assert_eq!(s.secret_store, "keychain");
    assert_eq!(s.solver_backend, "good_lp");
    assert_eq!(s.locale, "zh-CN");
    assert!(s.use_semantic_scorer);
    assert!(s.use_llm_explainer);
    assert!(!s.ai_explanation_prompt.is_empty());
    assert!(!s.ai_explanation_preamble.is_empty());
    assert!((s.overload_threshold - 1.10).abs() < 1e-9);
    assert!((s.underload_threshold - 0.50).abs() < 1e-9);
    assert!((s.utilization_green - 0.70).abs() < 1e-9);
    assert!((s.utilization_yellow - 1.00).abs() < 1e-9);
}

#[tokio::test]
async fn update_settings_persists_and_affects_readers() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();

    let updated = SettingsDto {
        default_unit: "PM".into(),
        pd_hours: 7.5,
        pm_workdays: 22.0,
        ai_provider: "openai".into(),
        ai_base_url: Some("https://api.openai.com".into()),
        ai_api_key_enc: Some("enc".into()),
        secret_store: "encrypted_file".into(),
        ai_chat_model: "gpt-4".into(),
        embed_provider: "openai".into(),
        embed_base_url: Some("https://api.openai.com".into()),
        embed_api_key_enc: Some("enc-embed".into()),
        embed_model: "text-embedding-3".into(),
        embed_dim: 1536,
        solver_backend: "greedy".into(),
        solver_timeout_ms: 10000,
        locale: "en-US".into(),
        use_semantic_scorer: false,
        use_llm_explainer: false,
        ai_explanation_prompt: "custom prompt {resource_count}".into(),
        ai_explanation_preamble: "custom preamble".into(),
        overload_threshold: 1.20,
        underload_threshold: 0.60,
        utilization_green: 0.75,
        utilization_yellow: 0.95,
        use_llm_advisor: true
    };

    SettingsService::update(&pool, updated.clone()).await.unwrap();

    let s = SettingsService::get(&pool).await.unwrap();
    assert_eq!(s.default_unit, "PM");
    assert!((s.pd_hours - 7.5).abs() < 1e-9);
    assert!((s.pm_workdays - 22.0).abs() < 1e-9);
    assert_eq!(s.ai_provider, "openai");
    assert_eq!(s.solver_backend, "greedy");
    assert_eq!(s.locale, "en-US");

    // Unit config reader sees new values.
    let u = global_unit_config(&pool).await.unwrap();
    assert!((u.hours_per_pd - 7.5).abs() < 1e-9);
    assert!((u.pd_per_pm - 22.0).abs() < 1e-9);

    // Threshold reader sees new values.
    let mut map = effective_thresholds_map(&pool, &[rid]).await.unwrap();
    let t = map.remove(&rid).unwrap();
    assert!((t.overload - 1.20).abs() < 1e-9);
    assert!((t.underload - 0.60).abs() < 1e-9);
    assert!((t.green - 0.75).abs() < 1e-9);
    assert!((t.yellow - 0.95).abs() < 1e-9);
}

#[tokio::test]
async fn update_settings_rejects_invalid_values() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    let base = SettingsService::get(&pool).await.unwrap();

    let mut invalid = base.clone();
    invalid.pd_hours = 0.0;
    assert!(SettingsService::update(&pool, invalid).await.is_err());

    let mut invalid = base.clone();
    invalid.utilization_green = 1.5;
    assert!(SettingsService::update(&pool, invalid).await.is_err());

    let mut invalid = base.clone();
    invalid.ai_provider = "unknown".into();
    assert!(SettingsService::update(&pool, invalid).await.is_err());
}
