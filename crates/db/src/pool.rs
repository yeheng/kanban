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

/// Connect with optional SQLCipher encryption (design §6.8 / decision #55).
///
/// When `key` is `Some`, `PRAGMA key = '...'` is issued so SQLCipher unlocks the
/// file. sqlx interpolates pragma values verbatim (no auto-quoting), so the
/// passphrase is single-quoted here with `'` doubled per the SQL string-literal
/// rule. The non-key PRAGMAs (synchronous/journal_mode/...) run after `PRAGMA key`
/// on an already-unlocked DB, which is safe. With a WRONG key the non-key PRAGMAs
/// touch the encrypted header, so the connection itself fails with code 26
/// ("file is not a database") — callers see a connection error, which is the
/// desired reject-wrong-passphrase behavior.
pub async fn connect_with_key(url: &str, key: Option<&str>) -> Result<SqlitePool, DbError> {
    let mut opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_millis(5000));
    if let Some(k) = key {
        let escaped = k.replace('\'', "''");
        opts = opts.pragma("key", format!("'{}'", escaped));
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