use std::env;

use tracing::info;

mod app;
mod handlers;
mod input_injector;
mod media_bridge;
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

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_owned());
    let bind_addr = format!("{host}:{port}");
    info!("listening on http://{bind_addr}");

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind listener");
    axum::serve(listener, app)
        .await
        .expect("server exited with error");
}
