use sqlx::SqlitePool;

/// Shared Tauri state. Holds the single SQLite pool for the local app.
pub struct AppState {
    pub pool: SqlitePool,
}