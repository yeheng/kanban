use crate::error::DbError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Connect to a SQLite file (or `:memory:`), unencrypted (design §3.1 PRAGMAs).
/// WAL is skipped for in-memory DBs (unsupported). Delegates to `connect_with_key`
/// with `None`.
pub async fn connect(url: &str) -> Result<SqlitePool, DbError> {
    connect_with_key(url, None).await
}

/// Connect with optional SQLCipher encryption. When `key` is `Some`, `PRAGMA key`
/// is issued **first** (SQLCipher requires the key before any other op) — honoring
/// decision #55 (encryption default-on). NOTE: SQLCipher itself is **not** wired up
/// yet — `Cargo.toml` pulls plain `libsqlite3-sys` (`bundled` only), so `PRAGMA key`
/// is currently a harmless no-op. Enabling the `sqlcipher` feature is deferred to
/// Phase 1 (design §3.1 / §6). Headless `:memory:` tests pass `None` and are unaffected.
pub async fn connect_with_key(url: &str, key: Option<&str>) -> Result<SqlitePool, DbError> {
    let mut opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_millis(5000));
    if let Some(k) = key {
        opts = opts.pragma("key", k.to_owned()); // SQLCipher key — MUST be the first pragma
    }
    opts = opts
        .pragma("synchronous", "NORMAL")
        .pragma("temp_store", "MEMORY");
    if !url.contains(":memory:") {
        opts = opts.pragma("journal_mode", "WAL").pragma("mmap_size", "268435456");
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await?;
    Ok(pool)
}