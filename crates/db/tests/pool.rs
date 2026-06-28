use db::pool::connect;

#[tokio::test]
async fn connect_sets_pragmas_and_foreign_keys() {
    let pool = connect("sqlite::memory:").await.unwrap();
    // foreign_keys ON
    let fk: (i64,) = sqlx::query_as("PRAGMA foreign_keys").fetch_one(&pool).await.unwrap();
    assert_eq!(fk.0, 1);
    // synchronous NORMAL (= 1)
    let syn: (i64,) = sqlx::query_as("PRAGMA synchronous").fetch_one(&pool).await.unwrap();
    assert_eq!(syn.0, 1);
    // migrations still runnable on the pool
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
}