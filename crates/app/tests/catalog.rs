use app::service::catalog::CatalogService;
use db::pool::connect;

#[tokio::test]
async fn ensure_is_idempotent() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let id1 = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let id2 = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    assert_eq!(id1, id2);
    assert_eq!(CatalogService::list_skills(&pool).await.unwrap().len(), 1);
}

#[tokio::test]
async fn tag_with_color() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let _ = CatalogService::ensure_tag(&pool, "urgent", Some("#f00")).await.unwrap();
    let tags = CatalogService::list_tags(&pool).await.unwrap();
    assert_eq!(tags[0].color.as_deref(), Some("#f00"));
}