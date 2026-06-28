use app::service::tasks::TasksService;
use app::service::projects::ProjectsService;
use db::pool::connect;

#[tokio::test]
async fn dependency_then_cycle_blocked() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, false, 0, &[], &[]).await.unwrap();
    let b = TasksService::create(&pool, pid, "B", None, 1.0, None, None, false, 1, &[], &[]).await.unwrap();
    let c = TasksService::create(&pool, pid, "C", None, 1.0, None, None, false, 2, &[], &[]).await.unwrap();

    // B depends on A; C depends on B  (A -> B -> C)
    TasksService::add_dependency(&pool, b, a, 0).await.unwrap();
    TasksService::add_dependency(&pool, c, b, 0).await.unwrap();

    // A depending on C would close the cycle A->B->C->A -> must be rejected
    let err = TasksService::add_dependency(&pool, a, c, 0).await.unwrap_err();
    assert_eq!(err.code, "DOMAIN");
    assert!(err.detail.contains("cycle"));
}

#[tokio::test]
async fn self_dependency_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, false, 0, &[], &[]).await.unwrap();
    let err = TasksService::add_dependency(&pool, a, a, 0).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}