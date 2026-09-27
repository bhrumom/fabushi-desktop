use crate::extensions::auth::credential_renewer::{
    SAND_CLIENT_TYPE, sand_box_namespace, sand_client_version,
};
use crate::notify_drain_gate::NotifyDrainGate;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const XUSER_RELAY_POLL_INTERVAL_MS: u64 = 4_000;
pub const XUSER_RELAY_ERROR_BACKOFF_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandXuserRelayHttpError {
    pub status: u16,
    pub path: String,
}
impl std::fmt::Display for SandXuserRelayHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sand sharing {} returned {}", self.path, self.status)
    }
}
impl std::error::Error for SandXuserRelayHttpError {}

pub trait XuserRelayTransport: Send + Sync {
    fn request(
        &self,
        method: &str,
        url: &str,
        token: &str,
        body: Option<&Value>,
    ) -> Result<Value, SandXuserRelayHttpError>;
}

pub struct ReqwestXuserRelayTransport {
    client: Client,
}
impl ReqwestXuserRelayTransport {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            client: Client::builder().build().map_err(|e| e.to_string())?,
        })
    }
}
impl XuserRelayTransport for ReqwestXuserRelayTransport {
    fn request(
        &self,
        method: &str,
        url: &str,
        token: &str,
        body: Option<&Value>,
    ) -> Result<Value, SandXuserRelayHttpError> {
        let mut req = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "DELETE" => self.client.delete(url),
            _ => self.client.post(url),
        }
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(CONTENT_TYPE, "application/json")
        .header("x-cursor-client-type", SAND_CLIENT_TYPE)
        .header("x-cursor-client-version", sand_client_version())
        .header("x-sand-box-namespace", sand_box_namespace());
        if let Some(body) = body {
            req = req.json(body);
        }
        let response = req.send().map_err(|_| SandXuserRelayHttpError {
            status: 0,
            path: url.into(),
        })?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(SandXuserRelayHttpError {
                status,
                path: url.into(),
            });
        }
        response
            .json::<Value>()
            .map_err(|_| SandXuserRelayHttpError {
                status,
                path: url.into(),
            })
    }
}

pub struct SandXuserRelayClient {
    backend_url: String,
    access_token: Arc<dyn Fn() -> Result<String, String> + Send + Sync>,
    transport: Arc<dyn XuserRelayTransport>,
}
impl SandXuserRelayClient {
    pub fn new(
        backend_url: String,
        access_token: Arc<dyn Fn() -> Result<String, String> + Send + Sync>,
        transport: Arc<dyn XuserRelayTransport>,
    ) -> Self {
        Self {
            backend_url: backend_url.trim_end_matches('/').into(),
            access_token,
            transport,
        }
    }
    fn call(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value, String> {
        let token = (self.access_token)()?;
        self.transport
            .request(
                method,
                &format!("{}{}", self.backend_url, path),
                &token,
                body,
            )
            .map_err(|e| describe_relay_error(&e))
    }
    pub fn poll(&self, ack_ids: &[String]) -> Result<Value, String> {
        self.call(
            "POST",
            "/sand/xuser/poll",
            Some(&serde_json::json!({ "ackIds": ack_ids })),
        )
    }
    pub fn fetch_share_state(&self) -> Result<Value, String> {
        self.call("POST", "/sand/share-state", Some(&serde_json::json!({})))
    }
    pub fn send(&self, payload: &Value) -> Result<Value, String> {
        self.call("POST", "/sand/xuser/send", Some(payload))
    }
    pub fn create_room(&self, payload: &Value) -> Result<Value, String> {
        self.call("POST", "/sand/share-rooms", Some(payload))
    }
    pub fn create_room_from_agent(&self, payload: &Value) -> Result<Value, String> {
        self.call("POST", "/sand/share-rooms/from-agent", Some(payload))
    }
    pub fn create_room_invite(&self, room_id: &str) -> Result<Value, String> {
        self.call(
            "POST",
            "/sand/share-rooms/invite-links",
            Some(&serde_json::json!({ "roomId": room_id })),
        )
    }
    pub fn join_room(&self, link: &str) -> Result<Value, String> {
        self.call(
            "POST",
            "/sand/share-rooms/join",
            Some(&serde_json::json!({ "link": link })),
        )
    }
    pub fn respond_to_join_request(&self, payload: &Value) -> Result<Value, String> {
        self.call("POST", "/sand/share-rooms/join/respond", Some(payload))
    }
    pub fn add_own_agent(&self, payload: &Value) -> Result<Value, String> {
        self.call("POST", "/sand/share-rooms/agents/add", Some(payload))
    }
    pub fn set_room_picture(&self, payload: &Value) -> Result<Value, String> {
        self.call("POST", "/sand/share-rooms/picture", Some(payload))
    }
    pub fn remove_own_agent(&self, payload: &Value) -> Result<Value, String> {
        self.call("POST", "/sand/share-rooms/agents/remove", Some(payload))
    }
    pub fn leave_room(&self, room_id: &str) -> Result<Value, String> {
        self.call(
            "POST",
            "/sand/share-rooms/leave",
            Some(&serde_json::json!({ "roomId": room_id })),
        )
    }
    pub fn remove_deleted_agent(&self, agent_id: &str) -> Result<Value, String> {
        self.call(
            "POST",
            "/sand/share-rooms/agents/remove-deleted",
            Some(&serde_json::json!({ "agentId": agent_id })),
        )
    }
}

type RelayClock = Arc<dyn Fn() -> u64 + Send + Sync>;
type RelayBool = Arc<dyn Fn() -> bool + Send + Sync>;
type RelayGateClock = Box<dyn Fn() -> u64 + Send + Sync>;
type RelayGateBool = Box<dyn Fn() -> bool + Send + Sync>;
type RelayNotifyGate = NotifyDrainGate<RelayGateClock, RelayGateBool, RelayGateBool>;
pub type XuserRelayEventHandler =
    Arc<dyn Fn(&Value) -> Result<bool, String> + Send + Sync>;

#[derive(Debug, Default)]
struct RelayRuntimeState {
    started: bool,
    is_ticking: bool,
    backoff_until_ms: u64,
    pending_ack_ids: Vec<String>,
}

/// Frozen Grok relay state machine without owning a second Host runtime.
///
/// The production owner may drive `tick` from the canonical Host scheduling
/// policy and `request_drain` from NotifyBus. This type owns only the relay
/// admission/ack/backoff semantics and remains inside the CrossUserSharing
/// extension boundary.
pub struct SandXuserRelayRuntime {
    client: Arc<SandXuserRelayClient>,
    on_event: XuserRelayEventHandler,
    now_ms: RelayClock,
    gate: Mutex<RelayNotifyGate>,
    state: Mutex<RelayRuntimeState>,
}

impl SandXuserRelayRuntime {
    pub fn new(
        client: Arc<SandXuserRelayClient>,
        on_event: XuserRelayEventHandler,
        now_ms: RelayClock,
        is_notify_connected: RelayBool,
        is_notify_safety_poll_enabled: RelayBool,
    ) -> Self {
        let gate_now = Arc::clone(&now_ms);
        let gate_connected = Arc::clone(&is_notify_connected);
        let gate_safety_poll = Arc::clone(&is_notify_safety_poll_enabled);
        let gate: RelayNotifyGate = NotifyDrainGate::new(
            Box::new(move || gate_now()) as RelayGateClock,
            Box::new(move || gate_connected()) as RelayGateBool,
            Box::new(move || gate_safety_poll()) as RelayGateBool,
        );
        Self {
            client,
            on_event,
            now_ms,
            gate: Mutex::new(gate),
            state: Mutex::new(RelayRuntimeState::default()),
        }
    }

    pub fn start(&self) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if state.started {
            return;
        }
        state.started = true;
        drop(state);
        self.gate
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .reset();
    }

    pub fn stop(&self) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        state.started = false;
        state.is_ticking = false;
        state.backoff_until_ms = 0;
    }

    pub fn record_notify(&self) {
        self.gate
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .record_notify();
    }

    pub fn request_drain(&self) -> Result<bool, String> {
        self.record_notify();
        self.tick()
    }

    pub fn pending_ack_ids(&self) -> Vec<String> {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .pending_ack_ids
            .clone()
    }

    pub fn backoff_until_ms(&self) -> u64 {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .backoff_until_ms
    }

    pub fn tick(&self) -> Result<bool, String> {
        let now = (self.now_ms)();
        let pending = {
            let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            if !state.started || state.is_ticking || now < state.backoff_until_ms {
                return Ok(false);
            }
            let should_drain = self
                .gate
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .should_drain(!state.pending_ack_ids.is_empty());
            if !should_drain {
                return Ok(false);
            }
            state.is_ticking = true;
            state.pending_ack_ids.clone()
        };

        let result = self.client.poll(&pending);
        let payload = match result {
            Ok(payload) => payload,
            Err(error) => {
                let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
                state.is_ticking = false;
                state.backoff_until_ms = now.saturating_add(XUSER_RELAY_ERROR_BACKOFF_MS);
                return Err(error);
            }
        };

        let previously_acked = pending.iter().cloned().collect::<BTreeSet<_>>();
        let mut next_ack_ids = Vec::new();
        let mut next_ack_set = BTreeSet::new();
        if let Some(events) = payload.get("events").and_then(Value::as_array) {
            for event in events {
                let Some(record) = event.as_object() else {
                    continue;
                };
                let Some(id) = record.get("id").and_then(Value::as_str) else {
                    continue;
                };
                if record.get("kind").and_then(Value::as_str).is_none() {
                    continue;
                }
                if previously_acked.contains(id) || next_ack_set.contains(id) {
                    if next_ack_set.insert(id.to_string()) {
                        next_ack_ids.push(id.to_string());
                    }
                    continue;
                }
                if (self.on_event)(event).unwrap_or(false)
                    && next_ack_set.insert(id.to_string())
                {
                    next_ack_ids.push(id.to_string());
                }
            }
        }

        {
            let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            state.pending_ack_ids = next_ack_ids;
            state.is_ticking = false;
            state.backoff_until_ms = 0;
        }
        self.gate
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .record_poll();
        Ok(true)
    }
}


pub struct SandXuserRelayDriver {
    runtime: Arc<SandXuserRelayRuntime>,
    stopped: Arc<AtomicBool>,
    wake_tx: Mutex<Option<mpsc::Sender<()>>>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl SandXuserRelayDriver {
    pub fn new(runtime: Arc<SandXuserRelayRuntime>) -> Self {
        Self {
            runtime,
            stopped: Arc::new(AtomicBool::new(true)),
            wake_tx: Mutex::new(None),
            worker: Mutex::new(None),
        }
    }

    pub fn runtime(&self) -> Arc<SandXuserRelayRuntime> {
        Arc::clone(&self.runtime)
    }

    pub fn start(&self) {
        let mut worker = self.worker.lock().unwrap_or_else(|p| p.into_inner());
        if worker.is_some() {
            return;
        }
        self.stopped.store(false, Ordering::Release);
        self.runtime.start();

        let (wake_tx, wake_rx) = mpsc::channel();
        *self.wake_tx.lock().unwrap_or_else(|p| p.into_inner()) = Some(wake_tx);

        let runtime = Arc::clone(&self.runtime);
        let stopped = Arc::clone(&self.stopped);
        *worker = thread::Builder::new()
            .name("sand-xuser-relay".into())
            .spawn(move || {
                while !stopped.load(Ordering::Acquire) {
                    if let Err(error) = runtime.tick() {
                        eprintln!("[sand:sharing] relay poll failed: {error}");
                    }
                    if stopped.load(Ordering::Acquire) {
                        break;
                    }
                    match wake_rx.recv_timeout(Duration::from_millis(XUSER_RELAY_POLL_INTERVAL_MS)) {
                        Ok(()) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .ok();
    }

    pub fn request_drain(&self) {
        self.runtime.record_notify();
        if let Some(wake) = self
            .wake_tx
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
        {
            let _ = wake.send(());
        }
    }

    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
        if let Some(wake) = self
            .wake_tx
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take()
        {
            let _ = wake.send(());
        }
        if let Some(worker) = self
            .worker
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take()
        {
            let _ = worker.join();
        }
        self.runtime.stop();
    }

    pub fn is_started(&self) -> bool {
        self.worker
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_some()
    }
}

impl Drop for SandXuserRelayDriver {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        if let Ok(slot) = self.wake_tx.get_mut() {
            if let Some(wake) = slot.take() {
                let _ = wake.send(());
            }
        }
        if let Ok(slot) = self.worker.get_mut() {
            if let Some(worker) = slot.take() {
                let _ = worker.join();
            }
        }
        self.runtime.stop();
    }
}

pub fn describe_relay_error(error: &SandXuserRelayHttpError) -> String {
    match error.status {
        403 => "Sharing isn't enabled for your account.".into(),
        429 => "You're doing that too often. Try again in a minute.".into(),
        _ => "Couldn't reach the sharing service. Check your connection and try again.".into(),
    }
}
