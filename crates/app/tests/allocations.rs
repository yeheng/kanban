use app::service::allocations::AllocationsService;
use app::service::projects::ProjectsService;
use app::service::tasks::TasksService;
use db::pool::connect;

async fn seeded() -> (sqlx::SqlitePool, i64) {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0)
        .await
        .unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')")
        .execute(&pool)
        .await
        .unwrap();
    let task_id = TasksService::create(
        &pool,
        pid,
        "T",
        None,
        1.0,
        Some("2026-06-01"),
        Some("2026-06-10"),
        false,
        0,
        &[],
        &[],
    )
    .await
    .unwrap();
    (pool, task_id)
}

#[tokio::test]
async fn create_rejects_malformed_dates() {
    let (pool, task_id) = seeded().await;
    let err = AllocationsService::create(&pool, 1, task_id, "2026-06-99", "2026-06-10", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");

    let err = AllocationsService::create(&pool, 1, task_id, "2026-06-1", "2026-06-10", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn update_rejects_bad_window_before_repo() {
    let (pool, task_id) = seeded().await;
    let id = AllocationsService::create(&pool, 1, task_id, "2026-06-01", "2026-06-05", 0.5)
        .await
        .unwrap();
    let err = AllocationsService::update(&pool, id, "2026-06-06", "2026-06-05", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}
