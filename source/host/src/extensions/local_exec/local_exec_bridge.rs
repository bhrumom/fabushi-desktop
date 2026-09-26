use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    mpsc::{self, Receiver, RecvTimeoutError, Sender},
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use uuid::Uuid;

use super::local_exec_error::SandLocalExecError;

pub const SUPERVISED_RANK: i32 = 4;
pub const SAND_LOCAL_EXEC_LIVENESS_WINDOW_MS: u64 = 30_000;
pub const SAND_LOCAL_EXEC_RESPONSE_TIMEOUT_MS: u64 = 10_000;
pub const DEFAULT_SAND_COMPUTER_ID: &str = "this-computer";
pub const SAND_NO_LOCAL_MACHINE_MESSAGE: &str =
    "Your local machine isn't connected right now (the Grok Bot desktop app must be open and online to run commands on it). Try again once it's reachable.";
const COMPUTER_UNAVAILABLE_SUFFIX: &str =
    "is unavailable — it looks disconnected. Reconnect it (or focus the computer you want commands to run on) and try again.";

pub fn sand_computer_unavailable_message(label: Option<&str>) -> String {
    match label.map(str::trim).filter(|value| !value.is_empty()) {
        Some(label) => format!("Your computer \"{label}\" {COMPUTER_UNAVAILABLE_SUFFIX}"),
        None => format!("Your computer {COMPUTER_UNAVAILABLE_SUFFIX}"),
    }
}

pub fn local_exec_provider_rank(supervised: bool, variant: Option<&str>) -> i32 {
    let variant_rank = match variant {
        Some("sand") => 2,
        Some("sand-lab") => 1,
        _ => 0,
    };
    (if supervised { SUPERVISED_RANK } else { 0 }) + variant_rank
}

pub fn bounded_local_exec_variant(variant: Option<&str>) -> Option<&str> {
    match variant {
        Some("sand" | "sand-lab" | "sand-dev") => variant,
        Some(_) => Some("unknown"),
        None => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalExecComputer {
    pub id: String,
    pub label: String,
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalExecProviderInfo {
    pub local_root: String,
    pub terminals_folder: String,
}

#[derive(Clone)]
struct Provider {
    id: String,
    send: Sender<Value>,
    sequence: u64,
    registered_at_ms: u64,
    last_seen_at_ms: u64,
    has_heartbeat: bool,
    info: Option<LocalExecProviderInfo>,
    computer_id: Option<String>,
    label: Option<String>,
    supervised: bool,
    variant: Option<String>,
}

impl Provider {
    fn computer_id(&self) -> &str {
        self.computer_id.as_deref().unwrap_or(DEFAULT_SAND_COMPUTER_ID)
    }

    fn label(&self) -> &str {
        self.label.as_deref().unwrap_or("this computer")
    }

    fn live(&self, now_ms: u64) -> bool {
        !self.has_heartbeat
            || now_ms.saturating_sub(self.last_seen_at_ms) <= SAND_LOCAL_EXEC_LIVENESS_WINDOW_MS
    }

    fn rank(&self) -> i32 {
        local_exec_provider_rank(self.supervised, self.variant.as_deref())
    }
}

struct BridgeState {
    providers: HashMap<String, Provider>,
    pending: HashMap<String, Sender<Value>>,
    ever_registered: bool,
    empty_since_ms: u64,
    next_sequence: u64,
}

impl BridgeState {
    fn new(now_ms: u64) -> Self {
        Self {
            providers: HashMap::new(),
            pending: HashMap::new(),
            ever_registered: false,
            empty_since_ms: now_ms,
            next_sequence: 0,
        }
    }
}

type Now = Arc<dyn Fn() -> u64 + Send + Sync>;
type RandomId = Arc<dyn Fn() -> String + Send + Sync>;

#[derive(Clone)]
pub struct SandLocalExecBridge {
    state: Arc<Mutex<BridgeState>>,
    now_ms: Now,
    random_id: RandomId,
}

impl SandLocalExecBridge {
    pub fn production() -> Self {
        Self::with_sources(
            Arc::new(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
                    .try_into()
                    .unwrap_or(u64::MAX)
            }),
            Arc::new(|| Uuid::new_v4().to_string()),
        )
    }

    pub fn with_sources(now_ms: Now, random_id: RandomId) -> Self {
        let now = now_ms();
        Self {
            state: Arc::new(Mutex::new(BridgeState::new(now))),
            now_ms,
            random_id,
        }
    }

    pub fn register_provider(&self, send: Sender<Value>) -> LocalExecProviderRegistration {
        let now = (self.now_ms)();
        let id = (self.random_id)();
        {
            let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let sequence = state.next_sequence;
            state.next_sequence = state.next_sequence.saturating_add(1);
            state.ever_registered = true;
            state.providers.insert(
                id.clone(),
                Provider {
                    id: id.clone(),
                    send: send.clone(),
                    sequence,
                    registered_at_ms: now,
                    last_seen_at_ms: now,
                    has_heartbeat: false,
                    info: None,
                    computer_id: None,
                    label: None,
                    supervised: false,
                    variant: None,
                },
            );
        }
        let _ = send.send(json!({ "kind": "welcome", "providerId": id }));
        LocalExecProviderRegistration {
            bridge: self.clone(),
            provider_id: Some(id),
        }
    }

    fn detach_provider(&self, provider_id: &str) {
        let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.providers.remove(provider_id).is_some() && state.providers.is_empty() {
            state.empty_since_ms = (self.now_ms)();
        }
    }

    pub fn has_provider(&self) -> bool {
        !self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).providers.is_empty()
    }

    pub fn provider_count(&self) -> usize {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).providers.len()
    }

    pub fn ever_registered(&self) -> bool {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).ever_registered
    }

    pub fn get_provider_info(&self) -> Option<LocalExecProviderInfo> {
        let now = (self.now_ms)();
        let state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.best_provider(&state, now, None)
            .or_else(|| state.providers.values().max_by_key(|provider| provider.sequence))
            .and_then(|provider| provider.info.clone())
    }

    pub fn list_computers(&self) -> Vec<LocalExecComputer> {
        let now = (self.now_ms)();
        let state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut newest: HashMap<String, &Provider> = HashMap::new();
        for provider in state.providers.values() {
            let id = provider.computer_id().to_string();
            let replace = newest
                .get(&id)
                .is_none_or(|current| provider.last_seen_at_ms >= current.last_seen_at_ms);
            if replace {
                newest.insert(id, provider);
            }
        }
        let mut computers = newest
            .into_iter()
            .map(|(id, provider)| LocalExecComputer {
                id,
                label: provider.label().to_string(),
                connected: provider.live(now),
            })
            .collect::<Vec<_>>();
        computers.sort_by(|left, right| left.id.cmp(&right.id));
        computers
    }

    pub fn active_computer(&self) -> Option<LocalExecComputer> {
        let now = (self.now_ms)();
        let state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.best_provider(&state, now, None).map(|provider| LocalExecComputer {
            id: provider.computer_id().to_string(),
            label: provider.label().to_string(),
            connected: true,
        })
    }

    pub fn is_computer_live(&self, computer_id: &str) -> bool {
        let now = (self.now_ms)();
        let state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.best_provider(&state, now, Some(computer_id)).is_some()
    }

    pub fn check_live_computer_for_ask(&self) -> bool {
        self.active_computer().is_some()
    }

    pub fn submit_responses(&self, batch: Value) {
        let provider_id = batch.get("providerId").and_then(Value::as_str).map(str::to_string);
        let frames = batch
            .get("frames")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let now = (self.now_ms)();
        let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let fallback = state
            .providers
            .values()
            .max_by_key(|provider| provider.sequence)
            .map(|provider| provider.id.clone());
        let selected = provider_id.or(fallback);

        for frame in frames {
            let kind = frame.get("kind").and_then(Value::as_str);
            if matches!(kind, Some("hello" | "ping")) {
                if let Some(id) = selected.as_deref() {
                    if let Some(provider) = state.providers.get_mut(id) {
                        provider.last_seen_at_ms = now;
                        if kind == Some("ping") {
                            provider.has_heartbeat = true;
                        }
                        if let Some(supervised) = frame.get("supervised").and_then(Value::as_bool) {
                            provider.supervised = supervised;
                        }
                        if kind == Some("hello") {
                            let local_root = frame.get("localRoot").and_then(Value::as_str);
                            let terminals_folder = frame.get("terminalsFolder").and_then(Value::as_str);
                            if let (Some(local_root), Some(terminals_folder)) = (local_root, terminals_folder) {
                                provider.info = Some(LocalExecProviderInfo {
                                    local_root: local_root.to_string(),
                                    terminals_folder: terminals_folder.to_string(),
                                });
                            }
                            if let Some(computer_id) = frame
                                .get("computerId")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                            {
                                provider.computer_id = Some(computer_id.to_string());
                            }
                            if let Some(label) = frame
                                .get("label")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                            {
                                provider.label = Some(label.to_string());
                            }
                            if let Some(variant) = frame
                                .get("variant")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                            {
                                provider.variant = Some(variant.to_string());
                            }
                        }
                    }
                }
                continue;
            }

            if let Some(request_id) = frame.get("requestId").and_then(Value::as_str) {
                if let Some(sender) = state.pending.get(request_id) {
                    let _ = sender.send(frame.clone());
                }
            }
        }
    }

    pub fn retire_approval(&self, approval_id: &str) {
        let request_id = (self.random_id)();
        let frame = json!({
            "kind": "retire-approval",
            "requestId": request_id,
            "approvalId": approval_id,
        });
        let state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        for provider in state.providers.values() {
            let _ = provider.send.send(frame.clone());
        }
    }

    pub fn request(
        &self,
        mut frame: Value,
        computer_id: Option<&str>,
    ) -> Result<LocalExecRequest, SandLocalExecError> {
        let now = (self.now_ms)();
        let request_id = (self.random_id)();
        let (send, receiver) = mpsc::channel();
        let provider = {
            let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let provider = self
                .best_provider(&state, now, computer_id)
                .cloned()
                .ok_or_else(|| {
                    let has_any = !state.providers.is_empty();
                    if !has_any {
                        SandLocalExecError::new(SAND_NO_LOCAL_MACHINE_MESSAGE)
                    } else {
                        let label = computer_id.and_then(|id| {
                            state
                                .providers
                                .values()
                                .find(|provider| provider.computer_id() == id)
                                .map(Provider::label)
                        });
                        SandLocalExecError::new(sand_computer_unavailable_message(label))
                    }
                })?;
            state.pending.insert(request_id.clone(), send);
            provider
        };

        let Some(object) = frame.as_object_mut() else {
            self.finish_request(&request_id);
            return Err(SandLocalExecError::new(
                "local-exec request frame must be a JSON object",
            ));
        };
        object.insert("requestId".into(), Value::String(request_id.clone()));
        if provider.send.send(frame).is_err() {
            self.finish_request(&request_id);
            return Err(SandLocalExecError::new(SAND_NO_LOCAL_MACHINE_MESSAGE));
        }

        Ok(LocalExecRequest {
            bridge: self.clone(),
            request_id,
            provider_send: provider.send,
            receiver,
            closed: false,
        })
    }

    fn finish_request(&self, request_id: &str) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pending
            .remove(request_id);
    }

    fn best_provider<'a>(
        &self,
        state: &'a BridgeState,
        now_ms: u64,
        computer_id: Option<&str>,
    ) -> Option<&'a Provider> {
        state
            .providers
            .values()
            .filter(|provider| provider.live(now_ms))
            .filter(|provider| computer_id.is_none_or(|id| provider.computer_id() == id))
            .max_by(|left, right| {
                left.rank()
                    .cmp(&right.rank())
                    .then_with(|| left.last_seen_at_ms.cmp(&right.last_seen_at_ms))
                    .then_with(|| left.sequence.cmp(&right.sequence))
            })
    }
}

pub struct LocalExecProviderRegistration {
    bridge: SandLocalExecBridge,
    provider_id: Option<String>,
}

impl Drop for LocalExecProviderRegistration {
    fn drop(&mut self) {
        if let Some(provider_id) = self.provider_id.take() {
            self.bridge.detach_provider(&provider_id);
        }
    }
}

pub struct LocalExecRequest {
    bridge: SandLocalExecBridge,
    request_id: String,
    provider_send: Sender<Value>,
    receiver: Receiver<Value>,
    closed: bool,
}

impl LocalExecRequest {
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<Value, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }

    pub fn recv_response(&self) -> Result<Value, SandLocalExecError> {
        self.recv_timeout(Duration::from_millis(SAND_LOCAL_EXEC_RESPONSE_TIMEOUT_MS))
            .map_err(|_| SandLocalExecError::new("local-exec response timed out"))
    }

    pub fn close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.bridge.finish_request(&self.request_id);
        let _ = self.provider_send.send(json!({
            "kind": "cancel",
            "requestId": self.request_id,
        }));
    }
}

impl Drop for LocalExecRequest {
    fn drop(&mut self) {
        self.close();
    }
}
