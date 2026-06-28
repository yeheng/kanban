use app::service::thresholds::effective_overload;
use app::service::teams::TeamsService;
use db::models::TeamOverride;
use db::ResourcesRepo;

#[tokio::test]
async fn resolves_team_override_then_settings_default() {
    let pool = db::pool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    // resource without a team -> settings default 1.10
    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();
    assert!((effective_overload(&pool, rid).await.unwrap() - 1.10).abs() < 1e-9);

    // put Alice in a team with override 1.30 -> wins
    let tid = TeamsService::create(&pool, "Eng", None).await.unwrap();
    TeamsService::add_member(&pool, tid, rid, Some("lead")).await.unwrap();
    TeamsService::set_override(&pool, TeamOverride {
        team_id: tid, pd_hours: None, pm_workdays: None,
        overload_threshold: Some(1.30), underload_threshold: None,
        utilization_green: None, utilization_yellow: None,
    }).await.unwrap();
    assert!((effective_overload(&pool, rid).await.unwrap() - 1.30).abs() < 1e-9);
}
