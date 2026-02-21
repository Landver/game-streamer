use serde::{Deserialize, Serialize};

// Query used when a peer joins/leaves a session.
#[derive(Deserialize)]
pub struct SessionPeerQuery {
    pub session_id: String,
    pub peer_id: String,
}

// Query used when polling pending signaling messages.
#[derive(Deserialize)]
pub struct PollQuery {
    pub session_id: String,
    pub peer_id: String,
}

// Body for offer/answer signaling messages.
#[derive(Deserialize)]
pub struct SdpPayload {
    pub from: String,
    pub to: String,
    pub sdp: String,
}

// Body for ICE candidate signaling messages.
#[derive(Deserialize)]
pub struct IceCandidatePayload {
    pub from: String,
    pub to: String,
    pub candidate: String,
}

// Signal protocol messages exchanged between browser peers via server relay.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalMessage {
    Join { peer_id: String },
    Leave { peer_id: String },
    Offer { from: String, to: String, sdp: String },
    Answer { from: String, to: String, sdp: String },
    IceCandidate { from: String, to: String, candidate: String },
}

// Generic API response for simple command endpoints.
#[derive(Serialize)]
pub struct ApiResponse {
    pub ok: bool,
}
