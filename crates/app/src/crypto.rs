use crate::error::AppError;
use crate::state::AppState;

/// Open (or create) the encrypted application database (design §6.8).
/// `passphrase` is obtained by the frontend first-run flow (Phase 1b); here we
/// only wire the mechanism. Migrations run on first open.
pub async fn open_encrypted(db_path: &str, passphrase: &str) -> Result<AppState, AppError> {
    // connect_with_key already sets create_if_missing(true); no ?mode=rwc needed
    // (sqlx parses unknown query params as PRAGMAs, which fails on "mode=rwc").
    let url = format!("sqlite:{}", db_path);
    let pool = db::pool::connect_with_key(&url, Some(passphrase)).await?;
    sqlx::migrate!("../db/migrations")
        .run(&pool)
        .await
        .map_err(db::DbError::from)?;
    Ok(AppState { pool })
}