use app::service::tasks::TasksService;
use app::service::projects::ProjectsService;
use db::pool::connect;
use db::repo::tasks::TaskDepsRepo;

#[tokio::test]
async fn deps_for_project_returns_edges() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, false, None, None, 0, &[], &[]).await.unwrap();
    let b = TasksService::create(&pool, pid, "B", None, 1.0, None, None, false, None, None, 1, &[], &[]).await.unwrap();
    TasksService::add_dependency(&pool, b, a, 0).await.unwrap();

    let edges = TaskDepsRepo::for_project(&pool, pid).await.unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].task_id, b);
    assert_eq!(edges[0].predecessor_id, a);
    assert_eq!(edges[0].dep_type, "FS"); // schema default
}

#[tokio::test]
async fn dependency_then_cycle_blocked() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, false, None, None, 0, &[], &[]).await.unwrap();
    let b = TasksService::create(&pool, pid, "B", None, 1.0, None, None, false, None, None, 1, &[], &[]).await.unwrap();
    let c = TasksService::create(&pool, pid, "C", None, 1.0, None, None, false, None, None, 2, &[], &[]).await.unwrap();

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
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, false, None, None, 0, &[], &[]).await.unwrap();
    let err = TasksService::add_dependency(&pool, a, a, 0).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}