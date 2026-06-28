use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

impl AppState {
    pub async fn open(url: &str) -> Self {
        let pool = db::pool::connect(url).await.expect("db connect");
        sqlx::migrate!("../db/migrations")
            .run(&pool)
            .await
            .expect("migrate");
        Self { pool }
    }
}
