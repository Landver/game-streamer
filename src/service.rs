use axum::{http::StatusCode, Json};
use tracing::{info, warn};

use crate::{
    media_bridge::MediaBridge,
    models::{ApiResponse, SessionPeerQuery, SignalMessage},
    state::{AppState, SessionState},
};

pub async fn join_session(
    state: AppState,
    query: SessionPeerQuery,
) -> (StatusCode, Json<ApiResponse>) {
    let mut sessions = state.sessions.write().await;
    let session = sessions.entry(query.session_id.clone()).or_default();
    session.peers.insert(query.peer_id.clone());
    session.inboxes.entry(query.peer_id.clone()).or_default();

    let join_msg = SignalMessage::Join {
        peer_id: query.peer_id.clone(),
    };
    enqueue_to_others(session, &query.peer_id, join_msg);
    if query.peer_id != "ffmpeg-bot" {
        if let Some(inbox) = session.inboxes.get_mut(&query.peer_id) {
            inbox.push_back(SignalMessage::Join {
                peer_id: "ffmpeg-bot".to_owned(),
            });
        }
    }

    info!("join session={} peer={}", query.session_id, query.peer_id);
    api_ok()
}

pub async fn leave_session(
    state: AppState,
    query: SessionPeerQuery,
) -> (StatusCode, Json<ApiResponse>) {
    let mut sessions = state.sessions.write().await;
    if let Some(session) = sessions.get_mut(&query.session_id) {
        session.peers.remove(&query.peer_id);
        session.inboxes.remove(&query.peer_id);

        let leave_msg = SignalMessage::Leave {
            peer_id: query.peer_id.clone(),
        };
        enqueue_to_others(session, &query.peer_id, leave_msg);

        if session.peers.is_empty() {
            sessions.remove(&query.session_id);
        }
    }
    info!("leave session={} peer={}", query.session_id, query.peer_id);
    api_ok()
}

// Shared routing logic for all signaling message handlers.
pub async fn route_signal_message(
    state: AppState,
    query: SessionPeerQuery,
    msg: SignalMessage,
) -> (StatusCode, Json<ApiResponse>) {
    if let SignalMessage::Offer { from, to, sdp } = &msg {
        if MediaBridge::is_bot_target(to) {
            return match state
                .media_bridge
                .handle_offer(
                    state.clone(),
                    query.session_id.clone(),
                    from.clone(),
                    sdp.clone(),
                )
                .await
            {
                Ok(()) => api_ok(),
                Err(err) => {
                    warn!("ffmpeg_bot offer failed session={} error={err}", query.session_id);
                    api_error(StatusCode::BAD_REQUEST)
                }
            };
        }
    }

    if let SignalMessage::IceCandidate {
        from,
        to,
        candidate,
    } = &msg
    {
        if MediaBridge::is_bot_target(to) {
            return match state
                .media_bridge
                .handle_remote_ice(&query.session_id, from, candidate)
                .await
            {
                Ok(()) => api_ok(),
                Err(err) => {
                    warn!("ffmpeg_bot ice failed session={} error={err}", query.session_id);
                    api_error(StatusCode::BAD_REQUEST)
                }
            };
        }
    }

    let mut sessions = state.sessions.write().await;
    let Some(session) = sessions.get_mut(&query.session_id) else {
        return api_error(StatusCode::NOT_FOUND);
    };

    log_signal(&query.session_id, &msg);
    if let Some(target_peer) = target_peer(&msg) {
        if let Some(inbox) = session.inboxes.get_mut(target_peer) {
            inbox.push_back(msg);
            return api_ok();
        }
    }

    warn!(
        "route_failed session={} from_peer={}",
        query.session_id, query.peer_id
    );
    api_error(StatusCode::BAD_REQUEST)
}

// Uniform structured logs for each signaling message class.
pub fn log_signal(session_id: &str, msg: &SignalMessage) {
    match msg {
        SignalMessage::Offer { from, to, .. } => {
            info!("offer session={session_id} from={from} to={to}")
        }
        SignalMessage::Answer { from, to, .. } => {
            info!("answer session={session_id} from={from} to={to}")
        }
        SignalMessage::IceCandidate { from, to, .. } => {
            info!("ice_candidate session={session_id} from={from} to={to}")
        }
        SignalMessage::Join { peer_id } => info!("join_event session={session_id} peer={peer_id}"),
        SignalMessage::Leave { peer_id } => {
            info!("leave_event session={session_id} peer={peer_id}")
        }
    }
}

// Helper to resolve direct-routing target peer from a signal message.
pub fn target_peer(msg: &SignalMessage) -> Option<&str> {
    match msg {
        SignalMessage::Offer { to, .. }
        | SignalMessage::Answer { to, .. }
        | SignalMessage::IceCandidate { to, .. } => Some(to.as_str()),
        SignalMessage::Join { .. } | SignalMessage::Leave { .. } => None,
    }
}

// Pushes a join/leave event to all peers except the source peer.
pub fn enqueue_to_others(session: &mut SessionState, source_peer: &str, msg: SignalMessage) {
    for peer_id in session.peers.clone() {
        if peer_id == source_peer {
            continue;
        }
        if let Some(inbox) = session.inboxes.get_mut(&peer_id) {
            inbox.push_back(msg.clone());
        }
    }
}

fn api_ok() -> (StatusCode, Json<ApiResponse>) {
    (StatusCode::OK, Json(ApiResponse { ok: true }))
}

fn api_error(status: StatusCode) -> (StatusCode, Json<ApiResponse>) {
    (status, Json(ApiResponse { ok: false }))
}
