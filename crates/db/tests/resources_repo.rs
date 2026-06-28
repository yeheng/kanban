use db::pool::connect;
use db::repo::ResourcesRepo;

#[tokio::test]
async fn create_list_get_softdelete() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let id = ResourcesRepo::create(&pool, "Alice", Some("a@x.com")).await.unwrap();
    assert!(id > 0);

    let list = ResourcesRepo::list_active(&pool).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Alice");

    let got = ResourcesRepo::get(&pool, id).await.unwrap();
    assert_eq!(got.email.as_deref(), Some("a@x.com"));
    assert!((got.daily_capacity_pd - 1.0).abs() < 1e-9); // default 1.0

    ResourcesRepo::soft_delete(&pool, id).await.unwrap();
    assert!(ResourcesRepo::get(&pool, id).await.is_err());
    let after = ResourcesRepo::list_active(&pool).await.unwrap();
    assert_eq!(after.len(), 0);
}