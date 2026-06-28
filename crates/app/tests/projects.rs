use app::service::projects::ProjectsService;
use db::pool::connect;

#[tokio::test]
async fn create_list_get_project() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    let id = ProjectsService::create(&pool, "Atlas", Some("desc"), Some("2026-06-01"), Some("2026-07-01"), 3, 40.0).await.unwrap();
    let list = ProjectsService::list(&pool).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Atlas");
    let got = ProjectsService::get(&pool, id).await.unwrap();
    assert_eq!(got.priority, 3);
    assert!((got.budget_pd - 40.0).abs() < 1e-9);
}

#[tokio::test]
async fn invalid_priority_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let err = ProjectsService::create(&pool, "X", None, None, None, 99, 0.0).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn bad_date_window_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let err = ProjectsService::create(&pool, "X", None, Some("2026-07-01"), Some("2026-06-01"), 5, 0.0).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION"); // InvalidDateWindow -> VALIDATION
}