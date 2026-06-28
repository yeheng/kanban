use crate::error::DbError;
use sqlx::SqlitePool;
use std::future::Future;
use std::pin::Pin;

/// Run `f` inside a write transaction with `BEGIN IMMEDIATE` + busy-retry
/// (design §6.5 / §3.7).
///
/// Design invariant: every write path goes through this helper. It uses
/// `BEGIN IMMEDIATE` (acquires the write lock up front, so the closure never
/// upgrades from a read lock mid-tx and eats `SQLITE_BUSY` after doing work),
/// combined with the pool-level `busy_timeout = 5000ms` and an application-level
/// backoff retry for locks held longer than that.
///
/// `f` receives an owned `Transaction` (sqlx 0.8's `begin_with` yields
/// `Transaction<'static>`, so no pool lifetime tie), does its work with
/// `&mut *tx`, and hands the transaction back alongside the result so
/// `with_write_tx` can `commit()` on `Ok` or drop (rollback) on `Err`.
///
/// On `SQLITE_BUSY` (code "5") the whole tx is retried with backoff up to 3
/// times. This hand-back shape compiles cleanly on stable Rust; the naive
/// `for<'c> FnOnce(&'c mut Transaction<'c, _>) -> Pin<Box<Future + 'c>>` does
/// not (the borrow outlives the commit call). `FnMut` allows re-invocation on
/// retry.
#[allow(clippy::type_complexity)]
pub fn with_write_tx<'p, F, T>(
    pool: &'p SqlitePool,
    mut f: F,
) -> Pin<Box<dyn Future<Output = Result<T, DbError>> + Send + 'p>>
where
    F: FnMut(
            sqlx::Transaction<'static, sqlx::Sqlite>,
        ) -> Pin<Box<dyn Future<Output = Result<(sqlx::Transaction<'static, sqlx::Sqlite>, T), DbError>> + Send>>
        + Send
        + 'p,
    T: Send + 'p,
{
    Box::pin(async move {
        const BACKOFF_MS: [u64; 3] = [50, 100, 200];
        let mut last: Option<DbError> = None;
        for &ms in BACKOFF_MS.iter() {
            // BEGIN IMMEDIATE — acquire the write lock before any work, so the
            // closure never upgrades a read lock and loses prior reads on busy.
            let tx = match pool.begin_with("BEGIN IMMEDIATE").await {
                Ok(tx) => tx,
                Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("5") => {
                    // SQLITE_BUSY acquiring the write lock; retry after backoff.
                    last = Some(DbError::Sqlx(sqlx::Error::Database(e)));
                    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                    continue;
                }
                Err(e) => return Err(DbError::Sqlx(e)),
            };
            match f(tx).await {
                Ok((tx, val)) => {
                    tx.commit().await?;
                    return Ok(val);
                }
                Err(DbError::Sqlx(sqlx::Error::Database(e)))
                    if e.code().as_deref() == Some("5") =>
                {
                    // SQLITE_BUSY mid-tx; tx already dropped by f. Retry.
                    last = Some(DbError::Sqlx(sqlx::Error::Database(e)));
                    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                }
                Err(e) => return Err(e), // tx already dropped by f on the Err path
            }
        }
        Err(last.unwrap_or(DbError::NotFound))
    })
}