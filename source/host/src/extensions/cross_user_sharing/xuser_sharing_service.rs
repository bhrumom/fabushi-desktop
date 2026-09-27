use super::xuser_relay::SandXuserRelayClient;
use super::xuser_state_reconcile::{XuserRoom, reconcile_rooms};
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub const STATE_RECONCILE_INTERVAL_MS: u64 = 5 * 60_000;
pub const SELF_IDENTITY_RETRY_MS: u64 = 1_000;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SandSharingState {
    pub is_enabled: bool,
    pub self_auth_id: Option<String>,
    pub pending_join_requests: Vec<Value>,
    pub rooms: Vec<XuserRoom>,
}

pub struct SandXuserSharingService {
    relay: Arc<SandXuserRelayClient>,
    state: Mutex<SandSharingState>,
}
impl SandXuserSharingService {
    pub fn new(relay: Arc<SandXuserRelayClient>) -> Self {
        Self {
            relay,
            state: Mutex::new(SandSharingState::default()),
        }
    }
    pub fn set_enabled(&self, enabled: bool) {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        s.is_enabled = enabled;
        if !enabled {
            s.pending_join_requests.clear();
            s.rooms.clear();
        }
    }
    pub fn set_self_auth_id(&self, id: Option<String>) {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .self_auth_id = id
    }
    pub fn get_state(&self) -> SandSharingState {
        self.state.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }
    pub fn reconcile_share_state(&self) -> Result<SandSharingState, String> {
        let current = self.get_state();
        if !current.is_enabled {
            return Ok(current);
        }
        let raw = self.relay.fetch_share_state()?;
        let pending = raw
            .get("pendingJoinRequests")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let remote: Vec<XuserRoom> = serde_json::from_value(
            raw.get("rooms")
                .cloned()
                .unwrap_or_else(|| Value::Array(vec![])),
        )
        .map_err(|e| e.to_string())?;
        let result = reconcile_rooms(&current.rooms, remote, current.self_auth_id.as_deref());
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        s.pending_join_requests = pending;
        s.rooms = result.rooms;
        Ok(s.clone())
    }
    pub fn create_room_from_agent(&self, agent_id: &str) -> Result<Value, String> {
        if !self.get_state().is_enabled {
            return Err("Sharing isn't enabled for your account.".into());
        }
        self.relay
            .create_room_from_agent(&serde_json::json!({"agentId":agent_id}))
    }
}
