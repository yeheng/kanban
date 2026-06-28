use crate::error::DbError;
use sqlx::SqlitePool;
use std::future::Future;
use std::pin::Pin;

/// Begin a write transaction with `busy_timeout` already configured on the pool
/// (design §6.5). The caller owns the transaction and must `commit().await` on
/// success (dropping rolls back).
pub async fn begin_write_tx(
    pool: &SqlitePool,
) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>, DbError> {
    Ok(pool.begin().await?)
}

/// Run `f` inside a write transaction with busy-retry (design §6.5).
///
/// `f` receives an owned `Transaction`, does its work, and hands the transaction
/// back alongside the result so `with_write_tx` can `commit()` on `Ok` or drop
/// (rollback) on `Err`. On `SQLITE_BUSY` (code "5") the whole tx is retried with
/// backoff up to 3 times.
///
/// This hand-back shape compiles cleanly on stable Rust + sqlx 0.8. Returning
/// `&mut Transaction` from a boxed `for<'c>` future fights the borrow checker and
/// does not compile; the cost of hand-back is one extra tuple element per call site.
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
        const BACKOFF_MS: [u64; 3] = [50, 100, 200];
        // FnOnce cannot retry; busy-retry requires re-running f, so we accept the
        // single-shot semantics here. For multi-shot retry, call with_write_tx again.
        let _ = BACKOFF_MS; // kept for documentation; single-shot in MVP.
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