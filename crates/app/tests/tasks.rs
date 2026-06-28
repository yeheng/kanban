use app::service::tasks::TasksService;
use app::service::projects::ProjectsService;
use app::service::catalog::CatalogService;
use db::pool::connect;

async fn fresh() -> sqlx::SqlitePool {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn create_task_with_skills_and_tags_atomic() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let urgent = CatalogService::ensure_tag(&pool, "urgent", None).await.unwrap();

    let tid = TasksService::create(
        &pool, pid, "Build API", None, 5.0, Some("2026-06-01"), Some("2026-06-10"),
        false, 0, &[(rust, 3, true, 1.0)], &[urgent]).await.unwrap();

    let reqs = TasksService::skill_reqs(&pool, tid).await.unwrap();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].skill_id, rust);
    assert_eq!(reqs[0].min_proficiency, 3);

    let kb = TasksService::kanban(&pool, pid).await.unwrap();
    assert_eq!(kb.len(), 1);
    assert_eq!(kb[0].title, "Build API");
    assert_eq!(kb[0].skill_count, 1);
    assert_eq!(kb[0].assignee, None); // no allocation yet
}

#[tokio::test]
async fn invalid_proficiency_rejected() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let err = TasksService::create(
        &pool, pid, "T", None, 1.0, None, None, false, 0, &[(1, 9, true, 1.0)], &[]).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn set_status_transitions() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let tid = TasksService::create(&pool, pid, "T", None, 1.0, None, None, false, 0, &[], &[]).await.unwrap();
    TasksService::set_status(&pool, tid, "in_progress").await.unwrap();
    let kb = TasksService::kanban(&pool, pid).await.unwrap();
    assert_eq!(kb[0].status, "in_progress");
    let err = TasksService::set_status(&pool, tid, "bogus").await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}