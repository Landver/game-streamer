use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{
    handlers::{
        answer_handler, health, ice_candidate_handler, join_handler, leave_handler, offer_handler,
        poll_handler,
    },
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/signal/join", post(join_handler))
        .route("/signal/leave", post(leave_handler))
        .route("/signal/offer", post(offer_handler))
        .route("/signal/answer", post(answer_handler))
        .route("/signal/ice_candidate", post(ice_candidate_handler))
        .route("/signal/poll", get(poll_handler))
        .fallback_service(ServeDir::new("public"))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
