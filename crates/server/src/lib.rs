pub mod error;
pub mod routes;
pub mod state;

use state::AppState;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

pub async fn run_server(pool: sqlx::SqlitePool, port: u16) {
    let state = AppState { pool };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = routes::api_router()
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    tracing::info!("API server listening on http://{}", addr);
    axum::serve(listener, app).await.expect("serve");
}
