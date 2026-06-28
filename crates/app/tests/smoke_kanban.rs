use app::service::catalog::CatalogService;
use app::service::projects::ProjectsService;
use app::service::tasks::TasksService;
use app::service::teams::TeamsService;
use db::pool::connect;
use db::ResourcesRepo;
use db::AllocationsRepo;

#[tokio::test]
async fn end_to_end_kanban_with_assignee() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    let pid = ProjectsService::create(&pool, "Atlas", None, Some("2026-06-01"), Some("2026-06-30"), 2, 60.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let hot = CatalogService::ensure_tag(&pool, "hot", Some("#f00")).await.unwrap();
    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();
    let team = TeamsService::create(&pool, "Eng", None).await.unwrap();
    TeamsService::add_member(&pool, team, rid, Some("lead")).await.unwrap();

    let tid = TasksService::create(
        &pool, pid, "Implement core", None, 5.0, Some("2026-06-05"), Some("2026-06-15"),
        false, 0, &[(rust, 4, true, 1.0)], &[hot]).await.unwrap();

    // assign Alice (within task window & project window)
    AllocationsRepo::create(&pool, rid, tid, "2026-06-08", "2026-06-12", 0.5).await.unwrap();

    let kb = TasksService::kanban(&pool, pid).await.unwrap();
    assert_eq!(kb.len(), 1);
    assert_eq!(kb[0].assignee.as_deref(), Some("Alice"));
    assert_eq!(kb[0].skill_count, 1);
    assert_eq!(kb[0].status, "todo");
}