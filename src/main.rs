use std::net::SocketAddr;

use tracing::info;

mod app;
mod handlers;
mod models;
mod service;
mod state;

use app::build_router;
use state::AppState;

// Bootstraps routes for static client + HTTP-only signaling endpoints.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info,tower_http=info")
        .init();

    let state = AppState::default();
    let app = build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
