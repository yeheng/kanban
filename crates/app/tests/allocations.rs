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

#[tokio::test]
async fn overlapping_allocations_over_capacity_are_blocked() {
    let (pool, task_id) = seeded().await;
    let task2 = TasksService::create(
        &pool,
        1,
        "T2",
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
    AllocationsService::create(&pool, 1, task_id, "2026-06-01", "2026-06-05", 0.7)
        .await
        .unwrap();
    let err = AllocationsService::create(&pool, 1, task2, "2026-06-03", "2026-06-06", 0.4)
        .await
        .unwrap_err();
    assert_eq!(err.code, "DOMAIN");
}

#[tokio::test]
async fn weekend_only_overlap_does_not_count_as_capacity_conflict() {
    let (pool, task_id) = seeded().await;
    let task2 = TasksService::create(
        &pool,
        1,
        "T2",
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
    AllocationsService::create(&pool, 1, task_id, "2026-06-05", "2026-06-07", 0.8)
        .await
        .unwrap();
    AllocationsService::create(&pool, 1, task2, "2026-06-07", "2026-06-08", 0.8)
        .await
        .unwrap();
}

#[tokio::test]
async fn dependency_order_is_blocked_for_allocations() {
    let (pool, predecessor) = seeded().await;
    let successor = TasksService::create(
        &pool,
        1,
        "T2",
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
    AllocationsService::create(&pool, 1, predecessor, "2026-06-01", "2026-06-05", 0.5)
        .await
        .unwrap();
    TasksService::add_dependency(&pool, successor, predecessor, 0)
        .await
        .unwrap();
    let err = AllocationsService::create(&pool, 1, successor, "2026-06-04", "2026-06-10", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "DOMAIN");
}
