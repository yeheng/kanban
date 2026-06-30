use app::service::thresholds::{effective_overload, effective_unit_config, global_unit_config};
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

#[tokio::test]
async fn unit_config_global_default_and_team_override() {
    let pool = db::pool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    // Global default: 8h/PD, 20 PD/PM (design §2.9).
    let g = global_unit_config(&pool).await.unwrap();
    assert!((g.hours_per_pd - 8.0).abs() < 1e-9);
    assert!((g.pd_per_pm - 20.0).abs() < 1e-9);

    // Resource without a team -> global default.
    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();
    let u = effective_unit_config(&pool, rid).await.unwrap();
    assert!((u.pd_per_pm - 20.0).abs() < 1e-9, "no team: global 20");

    // Put Alice in a team with pm_workdays=21 override -> effective becomes 21.
    let tid = TeamsService::create(&pool, "Eng", None).await.unwrap();
    TeamsService::add_member(&pool, tid, rid, Some("lead")).await.unwrap();
    TeamsService::set_override(&pool, TeamOverride {
        team_id: tid, pd_hours: Some(7.5), pm_workdays: Some(21.0),
        overload_threshold: None, underload_threshold: None,
        utilization_green: None, utilization_yellow: None,
    }).await.unwrap();
    let u = effective_unit_config(&pool, rid).await.unwrap();
    assert!((u.pd_per_pm - 21.0).abs() < 1e-9, "team override: 21");
    assert!((u.hours_per_pd - 7.5).abs() < 1e-9, "team pd_hours override: 7.5");

    // PM conversion uses the effective N: 42 PD / 21 = 2 PM.
    assert!((u.pd_to_pm(42.0) - 2.0).abs() < 1e-9);

    // A resource in a different team without overrides still gets the global default.
    let bob = ResourcesRepo::create(&pool, "Bob", None).await.unwrap();
    let t2 = TeamsService::create(&pool, "Ops", None).await.unwrap();
    TeamsService::add_member(&pool, t2, bob, None).await.unwrap();
    let ub = effective_unit_config(&pool, bob).await.unwrap();
    assert!((ub.pd_per_pm - 20.0).abs() < 1e-9, "other team no override: global 20");
}
