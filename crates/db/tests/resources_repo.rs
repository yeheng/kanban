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

#[tokio::test]
async fn update_with_null_capacity_preserves_existing() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let id = ResourcesRepo::create(&pool, "Bob", None).await.unwrap();
    // Give the resource a non-default capacity.
    ResourcesRepo::update(&pool, id, "Bob", None, None, None, Some(3.0), None).await.unwrap();
    assert!((ResourcesRepo::get(&pool, id).await.unwrap().daily_capacity_pd - 3.0).abs() < 1e-9);
    // Update with capacity=None (field left untouched in the edit form) must NOT clobber to 1.0.
    ResourcesRepo::update(&pool, id, "Bobby", None, None, None, None, None).await.unwrap();
    let got = ResourcesRepo::get(&pool, id).await.unwrap();
    assert_eq!(got.name, "Bobby");
    assert!(
        (got.daily_capacity_pd - 3.0).abs() < 1e-9,
        "capacity must be preserved on null update, got {}",
        got.daily_capacity_pd
    );
}