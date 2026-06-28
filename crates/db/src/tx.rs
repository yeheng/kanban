use crate::error::DbError;
use sqlx::SqlitePool;
use std::future::Future;
use std::pin::Pin;

/// Run `f` inside a write transaction (design §6.5).
///
/// `f` receives an owned `Transaction`, does its work with `&mut *tx`, and hands
/// the transaction back alongside the result so `with_write_tx` can `commit()` on
/// `Ok` or drop (rollback) on `Err`.
///
/// This hand-back shape compiles cleanly on stable Rust + sqlx 0.8. The naive
/// `for<'c> FnOnce(&'c mut Transaction<'c, _>) -> Pin<Box<Future + 'c>>` signature
/// fights the borrow checker (the `&mut` borrow outlives the commit call) and does
/// not compile. The cost of hand-back is one extra tuple element per call site.
///
/// Single-shot (no retry) in MVP: SQLite in WAL mode has a single writer and the
/// pool already sets `busy_timeout = 5000ms`, so `SQLITE_BUSY` is handled at the
/// SQLite layer. If explicit app-level retry becomes necessary, extend `f` to `FnMut`.
pub fn with_write_tx<'p, F, T>(
    pool: &'p SqlitePool,
    f: F,
) -> Pin<Box<dyn Future<Output = Result<T, DbError>> + Send + 'p>>
where
    F: FnOnce(
            sqlx::Transaction<'p, sqlx::Sqlite>,
        ) -> Pin<Box<dyn Future<Output = Result<(sqlx::Transaction<'p, sqlx::Sqlite>, T), DbError>> + Send + 'p>>
        + Send
        + 'p,
    T: Send + 'p,
{
    Box::pin(async move {
        let tx = pool.begin().await?;
        match f(tx).await {
            Ok((tx, val)) => {
                tx.commit().await?;
                Ok(val)
            }
            Err(e) => Err(e), // tx already dropped inside f on the Err path
        }
    })
}