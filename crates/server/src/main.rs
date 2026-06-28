use std::env;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("HOME env var");
        let dir = std::path::Path::new(&home).join("Library/Application Support/com.hrkanban.app");
        std::fs::create_dir_all(&dir).ok();
        format!("sqlite://{}/hrk.db?mode=rwc", dir.to_string_lossy())
    });

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let state = server::state::AppState::open(&db_url).await;
    server::run_server(state.pool, port).await;
}
