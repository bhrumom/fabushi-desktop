use super::xuser_relay::{SandXuserRelayClient, SandXuserRelayDriver};
use super::xuser_state_reconcile::{XuserRoom, reconcile_rooms};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
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
    relay_driver: Mutex<Option<Arc<SandXuserRelayDriver>>>,
    state: Mutex<SandSharingState>,
    started: AtomicBool,
}
impl SandXuserSharingService {
    pub fn new(relay: Arc<SandXuserRelayClient>) -> Self {
        Self {
            relay,
            relay_driver: Mutex::new(None),
            state: Mutex::new(SandSharingState::default()),
            started: AtomicBool::new(false),
        }
    }
    pub fn bind_relay_driver(&self, driver: Arc<SandXuserRelayDriver>) {
        let mut slot = self.relay_driver.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(previous) = slot.replace(driver) {
            previous.stop();
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        {
            let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
            s.is_enabled = enabled;
            if !enabled {
                s.pending_join_requests.clear();
                s.rooms.clear();
            }
        }
        if !enabled {
            self.stop();
        }
    }

    pub fn start(&self) -> Result<SandSharingState, String> {
        if !self.get_state().is_enabled {
            return Ok(self.get_state());
        }
        if self.started.swap(true, Ordering::AcqRel) {
            return Ok(self.get_state());
        }
        if let Err(error) = self.reconcile_share_state() {
            self.started.store(false, Ordering::Release);
            return Err(error);
        }
        if let Some(driver) = self
            .relay_driver
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .cloned()
        {
            driver.start();
        }
        Ok(self.get_state())
    }

    pub fn stop(&self) {
        if !self.started.swap(false, Ordering::AcqRel) {
            return;
        }
        if let Some(driver) = self
            .relay_driver
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .cloned()
        {
            driver.stop();
        }
    }

    pub fn request_relay_drain(&self) {
        if !self.started.load(Ordering::Acquire) {
            return;
        }
        if let Some(driver) = self
            .relay_driver
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .cloned()
        {
            driver.request_drain();
        }
    }

    pub fn is_started(&self) -> bool {
        self.started.load(Ordering::Acquire)
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
