use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::models::SignalMessage;

// Global in-memory signaling state keyed by session ID.
#[derive(Clone, Default)]
pub struct AppState {
    pub sessions: Arc<RwLock<HashMap<String, SessionState>>>,
}

// Per-session peer registry and inbox queues used by HTTP polling.
#[derive(Default)]
pub struct SessionState {
    pub peers: HashSet<String>,
    pub inboxes: HashMap<String, VecDeque<SignalMessage>>,
}
