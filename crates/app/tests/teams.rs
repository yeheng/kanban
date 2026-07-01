use app::service::teams::TeamsService;
use app::service::projects::ProjectsService;
use db::models::TeamOverride;
use db::pool::connect;
use db::ResourcesRepo;

#[tokio::test]
async fn team_members_and_override() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let _ = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap(); // satisfy FK-free env

    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();
    let tid = TeamsService::create(&pool, "Platform", Some("core")).await.unwrap();
    TeamsService::add_member(&pool, tid, rid, Some("lead")).await.unwrap();
    assert_eq!(TeamsService::members(&pool, tid).await.unwrap().len(), 1);

    TeamsService::set_override(&pool, TeamOverride {
        team_id: tid, pd_hours: Some(8.0), pm_workdays: Some(20.0),
        overload_threshold: Some(1.1), underload_threshold: None,
        utilization_green: Some(0.7), utilization_yellow: Some(0.9),
    }).await.unwrap();
    let o = TeamsService::get_override(&pool, tid).await.unwrap().unwrap();
    assert!((o.utilization_green.unwrap() - 0.7).abs() < 1e-9);
}

#[tokio::test]
async fn remove_member_drops_membership_and_404s_when_absent() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();
    let tid = TeamsService::create(&pool, "Platform", None).await.unwrap();
    TeamsService::add_member(&pool, tid, rid, Some("lead")).await.unwrap();
    assert_eq!(TeamsService::members(&pool, tid).await.unwrap().len(), 1);

    // Happy path: membership is gone afterwards.
    TeamsService::remove_member(&pool, tid, rid).await.unwrap();
    assert_eq!(TeamsService::members(&pool, tid).await.unwrap().len(), 0);

    // Removing an absent membership is a 404, not a silent success.
    let err = TeamsService::remove_member(&pool, tid, rid).await.unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
}

#[tokio::test]
async fn bad_override_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let tid = TeamsService::create(&pool, "T", None).await.unwrap();
    let err = TeamsService::set_override(&pool, TeamOverride {
        team_id: tid, pd_hours: None, pm_workdays: None,
        overload_threshold: None, underload_threshold: None,
        utilization_green: Some(1.5), utilization_yellow: None,
    }).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}