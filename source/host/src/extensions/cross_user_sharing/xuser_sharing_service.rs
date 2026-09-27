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


    pub fn create_room_invite(&self, room_id: &str) -> Result<Value, String> {
        if !self.get_state().is_enabled {
            return Ok(serde_json::json!({
                "status": "error",
                "message": "Sharing isn't enabled."
            }));
        }
        self.relay.create_room_invite(room_id)
    }

    pub fn join_room(&self, link: &str) -> Result<Value, String> {
        if !self.get_state().is_enabled {
            return Ok(serde_json::json!({
                "status": "error",
                "message": "Sharing isn't enabled."
            }));
        }
        let result = self.relay.join_room(link)?;
        if result.get("status").and_then(Value::as_str) == Some("already-member") {
            let _ = self.reconcile_share_state();
        }
        Ok(result)
    }

    pub fn respond_to_join_request(&self, payload: &Value) -> Result<SandSharingState, String> {
        if !self.get_state().is_enabled {
            return Ok(self.get_state());
        }
        let request_id = payload
            .get("requestId")
            .and_then(Value::as_str)
            .map(str::to_string);
        let result = self.relay.respond_to_join_request(payload)?;
        {
            let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(request_id) = request_id.as_deref() {
                state.pending_join_requests.retain(|request| {
                    request.get("requestId").and_then(Value::as_str) != Some(request_id)
                });
            }
        }
        if result.get("status").and_then(Value::as_str) == Some("approved") {
            if let Some(room) = result.get("room") {
                self.install_room_value(room)?;
            }
        }
        Ok(self.get_state())
    }

    pub fn create_shared_room(&self, payload: &Value) -> Result<Value, String> {
        let current = self.get_state();
        if !current.is_enabled {
            return Ok(serde_json::json!({
                "status": "error",
                "message": "Sharing isn't enabled."
            }));
        }
        if current.self_auth_id.is_none() {
            return Ok(serde_json::json!({
                "status": "error",
                "message": "Sign in to create shared groups."
            }));
        }
        let result = self.relay.create_room(payload)?;
        if result.get("status").and_then(Value::as_str) != Some("created") {
            return Ok(serde_json::json!({
                "status": "error",
                "message": "The room couldn't be created."
            }));
        }
        let Some(room) = result.get("room") else {
            return Ok(serde_json::json!({
                "status": "error",
                "message": "The room couldn't be created."
            }));
        };
        let parsed: XuserRoom = serde_json::from_value(room.clone()).map_err(|e| e.to_string())?;
        let room_id = parsed.room_id.clone();
        self.install_room(parsed);
        Ok(serde_json::json!({ "status": "ok", "roomId": room_id }))
    }

    pub fn add_own_agent(&self, payload: &Value) -> Result<SandSharingState, String> {
        if !self.get_state().is_enabled {
            return Ok(self.get_state());
        }
        let result = self.relay.add_own_agent(payload)?;
        if let Some(room) = result.get("room") {
            self.install_room_value(room)?;
        }
        Ok(self.get_state())
    }

    pub fn remove_own_agent(&self, room_id: &str, agent_id: &str) -> Result<SandSharingState, String> {
        if !self.get_state().is_enabled {
            return Ok(self.get_state());
        }
        let result = self.relay.remove_own_agent(&serde_json::json!({
            "roomId": room_id,
            "agentId": agent_id,
        }))?;
        if let Some(room) = result.get("room") {
            self.install_room_value(room)?;
        }
        Ok(self.get_state())
    }

    pub fn leave_shared_room(
        &self,
        room_id: &str,
        target_auth_id: Option<&str>,
    ) -> Result<SandSharingState, String> {
        let current = self.get_state();
        if !current.is_enabled {
            return Ok(current);
        }
        let _ = self.relay.leave_room_with_target(room_id, target_auth_id)?;
        if target_auth_id.is_none() || target_auth_id == current.self_auth_id.as_deref() {
            self.state
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .rooms
                .retain(|room| room.room_id != room_id);
        }
        let _ = self.reconcile_share_state();
        Ok(self.get_state())
    }

    pub fn note_agent_deleted(&self, agent_id: &str) -> Result<(), String> {
        if !self.get_state().is_enabled {
            return Ok(());
        }
        let _ = self.relay.remove_deleted_agent(agent_id)?;
        let _ = self.reconcile_share_state();
        Ok(())
    }

    pub fn set_room_typing(&self, room_id: &str, is_typing: bool) -> Result<(), String> {
        if !self.get_state().is_enabled {
            return Ok(());
        }
        let _ = self.relay.send(&serde_json::json!({
            "kind": "room-typing",
            "roomId": room_id,
            "isTyping": is_typing,
        }))?;
        Ok(())
    }

    pub fn prepare_for_upgrade(&self) {
        self.stop();
    }

    fn install_room_value(&self, room: &Value) -> Result<(), String> {
        let parsed: XuserRoom = serde_json::from_value(room.clone()).map_err(|e| e.to_string())?;
        self.install_room(parsed);
        Ok(())
    }

    fn install_room(&self, room: XuserRoom) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        state.rooms.retain(|current| current.room_id != room.room_id);
        state.rooms.push(room);
        state.rooms.sort_by(|left, right| left.room_id.cmp(&right.room_id));
    }
}
