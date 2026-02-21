use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    models::{
        ApiResponse, IceCandidatePayload, PollQuery, SdpPayload, SessionPeerQuery, SignalMessage,
    },
    service::{join_session, leave_session, route_signal_message},
    state::AppState,
};

pub async fn health() -> &'static str {
    "ok"
}

fn empty_poll() -> Json<Vec<SignalMessage>> {
    Json(Vec::new())
}

async fn route_payload<P, F>(
    state: AppState,
    query: SessionPeerQuery,
    payload: P,
    builder: F,
) -> (StatusCode, Json<ApiResponse>)
where
    F: FnOnce(P) -> SignalMessage,
{
    route_signal_message(state, query, builder(payload)).await
}

// Registers peer in a session and notifies existing peers via inbox queues.
pub async fn join_handler(
    State(state): State<AppState>,
    Query(query): Query<SessionPeerQuery>,
) -> impl IntoResponse {
    join_session(state, query).await
}

// Removes peer from session and informs remaining peers about disconnect.
pub async fn leave_handler(
    State(state): State<AppState>,
    Query(query): Query<SessionPeerQuery>,
) -> impl IntoResponse {
    leave_session(state, query).await
}

// Receives one offer and routes it to the target peer inbox.
pub async fn offer_handler(
    State(state): State<AppState>,
    Query(query): Query<SessionPeerQuery>,
    Json(payload): Json<SdpPayload>,
) -> impl IntoResponse {
    route_payload(state, query, payload, |payload| SignalMessage::Offer {
        from: payload.from,
        to: payload.to,
        sdp: payload.sdp,
    })
    .await
}

// Receives one answer and routes it to the target peer inbox.
pub async fn answer_handler(
    State(state): State<AppState>,
    Query(query): Query<SessionPeerQuery>,
    Json(payload): Json<SdpPayload>,
) -> impl IntoResponse {
    route_payload(state, query, payload, |payload| SignalMessage::Answer {
        from: payload.from,
        to: payload.to,
        sdp: payload.sdp,
    })
    .await
}

// Receives one ICE candidate and routes it to the target peer inbox.
pub async fn ice_candidate_handler(
    State(state): State<AppState>,
    Query(query): Query<SessionPeerQuery>,
    Json(payload): Json<IceCandidatePayload>,
) -> impl IntoResponse {
    route_payload(state, query, payload, |payload| SignalMessage::IceCandidate {
        from: payload.from,
        to: payload.to,
        candidate: payload.candidate,
    })
    .await
}

// Poll endpoint returns and drains all queued messages for a peer.
pub async fn poll_handler(
    State(state): State<AppState>,
    Query(query): Query<PollQuery>,
) -> impl IntoResponse {
    let mut sessions = state.sessions.write().await;
    let Some(session) = sessions.get_mut(&query.session_id) else {
        return empty_poll();
    };
    let Some(inbox) = session.inboxes.get_mut(&query.peer_id) else {
        return empty_poll();
    };

    let drained: Vec<_> = inbox.drain(..).collect();
    Json(drained)
}
