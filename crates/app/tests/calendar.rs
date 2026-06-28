use app::service::calendar::CalendarService;
use db::ResourcesRepo;

async fn fresh() -> sqlx::SqlitePool {
    let pool = db::pool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn rejects_invalid_calendar_inputs() {
    let pool = fresh().await;

    let err = CalendarService::set_global_work_week(&pool, vec![1.0, 1.2, 1.0, 1.0, 1.0, 0.0, 0.0])
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");

    let err = CalendarService::add_holiday(&pool, None, "2026-99-01", Some(1.0), None)
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");

    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();
    let err = CalendarService::add_time_off(&pool, rid, "2026-07-01", Some(0.0), None)
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn rejects_missing_calendar_foreign_keys() {
    let pool = fresh().await;

    let err = CalendarService::add_holiday(&pool, Some(999), "2026-07-01", Some(1.0), None)
        .await
        .unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");

    let err = CalendarService::add_time_off(&pool, 999, "2026-07-01", Some(1.0), None)
        .await
        .unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
}
