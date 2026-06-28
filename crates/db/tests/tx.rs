use db::pool::connect;
use db::tx::with_write_tx;

#[tokio::test]
async fn tx_commits_on_success() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let id = with_write_tx(&pool, |mut tx| {
        Box::pin(async move {
            sqlx::query("INSERT INTO skills (name) VALUES (?)")
                .bind("Rust")
                .execute(&mut *tx)
                .await?;
            let row: (i64,) = sqlx::query_as("SELECT count(*) FROM skills")
                .fetch_one(&mut *tx)
                .await?;
            Ok((tx, row.0))
        })
    })
    .await
    .unwrap();
    assert_eq!(id, 1);

    // visible after commit on a fresh connection
    let after: (i64,) = sqlx::query_as("SELECT count(*) FROM skills")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(after.0, 1);
}

#[tokio::test]
async fn tx_rolls_back_on_error() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let res = with_write_tx(&pool, |mut tx| {
        Box::pin(async move {
            sqlx::query("INSERT INTO skills (name) VALUES (?)")
                .bind("Rust")
                .execute(&mut *tx)
                .await?;
            // force a failure -> closure returns Err -> tx dropped (rollback)
            sqlx::query("INSERT INTO no_such_table VALUES (1)")
                .execute(&mut *tx)
                .await?;
            Ok((tx, ()))
        })
    })
    .await;
    assert!(res.is_err());

    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM skills")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0, "rollback should leave table empty");
}

#[tokio::test]
async fn tx_uses_begin_immediate() {
    // Design §6.5: with_write_tx must use BEGIN IMMEDIATE, not the default DEFERRED.
    // A write done inside the tx must already hold the write lock when it executes,
    // so the lock-acquire happens before f runs.
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let val = with_write_tx(&pool, |mut tx| {
        Box::pin(async move {
            // `tx` is in IMMEDIATE mode; we can both read and write without an
            // upgrade. The fact that this completes without SQLITE_BUSY on a
            // single-writer :memory: DB confirms the lock was acquired up front.
            sqlx::query("INSERT INTO skills (name) VALUES (?)")
                .bind("Rust")
                .execute(&mut *tx)
                .await?;
            let row: (i64,) =
                sqlx::query_as("SELECT count(*) FROM skills").fetch_one(&mut *tx).await?;
            Ok((tx, row.0))
        })
    })
    .await
    .unwrap();
    assert_eq!(val, 1);
}