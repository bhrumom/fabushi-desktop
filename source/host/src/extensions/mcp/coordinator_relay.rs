use crate::gateway_server::GatewayEventHub;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, RecvTimeoutError, SyncSender},
};
use std::time::Duration;
use thiserror::Error;

pub const HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL: &str = "host-mcp-lifecycle-request";
pub const MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD: &str = "mcp.resolveLifecycleRequest";
pub const MCP_LIST_SERVERS_METHOD: &str = "listHostMcpServers";
pub const MCP_LIST_CATALOG_METHOD: &str = "listHostMcpCatalog";
pub const MCP_LIST_EFFECTIVE_PLUGINS_METHOD: &str = "listHostEffectiveMcpPlugins";
pub const MCP_INSTALL_PLUGIN_METHOD: &str = "installHostMcpPlugin";
pub const MCP_UNINSTALL_PLUGIN_METHOD: &str = "uninstallHostMcpPlugin";
pub const MCP_NOTE_AUTH_COMPLETED_METHOD: &str = "noteHostMcpAuthCompleted";

const DEFAULT_MCP_LIFECYCLE_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Error, PartialEq)]
pub enum CoordinatorMcpLifecycleRelayError {
    #[error("unsupported Host MCP lifecycle relay method: {0}")]
    UnsupportedMethod(String),
    #[error("Host MCP lifecycle relay request {0} timed out")]
    Timeout(String),
    #[error("Host MCP lifecycle relay request {request_id} failed: {message}")]
    Remote { request_id: String, message: String },
    #[error("Host MCP lifecycle relay is closed: {0}")]
    Closed(String),
    #[error("invalid Host MCP lifecycle relay response: {0}")]
    Protocol(String),
}

type PendingReply = SyncSender<Result<Value, CoordinatorMcpLifecycleRelayError>>;

pub fn is_host_mcp_lifecycle_method(method: &str) -> bool {
    matches!(
        method,
        MCP_LIST_SERVERS_METHOD
            | MCP_LIST_CATALOG_METHOD
            | MCP_LIST_EFFECTIVE_PLUGINS_METHOD
            | MCP_INSTALL_PLUGIN_METHOD
            | MCP_UNINSTALL_PLUGIN_METHOD
            | MCP_NOTE_AUTH_COMPLETED_METHOD
    )
}

pub struct CoordinatorMcpLifecycleRelay {
    events: GatewayEventHub,
    pending: Mutex<HashMap<String, PendingReply>>,
    next_request_id: AtomicU64,
    timeout: Duration,
    closed: AtomicBool,
}

impl CoordinatorMcpLifecycleRelay {
    pub fn new(events: GatewayEventHub) -> Self {
        Self::with_timeout(events, DEFAULT_MCP_LIFECYCLE_REQUEST_TIMEOUT)
    }

    pub fn with_timeout(events: GatewayEventHub, timeout: Duration) -> Self {
        Self {
            events,
            pending: Mutex::new(HashMap::new()),
            next_request_id: AtomicU64::new(0),
            timeout,
            closed: AtomicBool::new(false),
        }
    }

    pub fn request(&self, method: &str, args: Value) -> Result<Value, CoordinatorMcpLifecycleRelayError> {
        if !is_host_mcp_lifecycle_method(method) {
            return Err(CoordinatorMcpLifecycleRelayError::UnsupportedMethod(method.to_string()));
        }
        if self.closed.load(Ordering::Acquire) {
            return Err(CoordinatorMcpLifecycleRelayError::Closed("Mahayana Host is shutting down".into()));
        }

        let sequence = self.next_request_id.fetch_add(1, Ordering::AcqRel) + 1;
        let request_id = format!("host-mcp-{sequence}");
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.pending
            .lock()
            .map_err(|_| CoordinatorMcpLifecycleRelayError::Protocol("pending relay lock poisoned".into()))?
            .insert(request_id.clone(), reply_tx);

        self.events.publish(json!({
            "channel": HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL,
            "payload": {
                "requestId": request_id,
                "method": method,
                "args": args,
            }
        }));

        match reply_rx.recv_timeout(self.timeout) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&request_id);
                }
                Err(CoordinatorMcpLifecycleRelayError::Timeout(request_id))
            }
            Err(RecvTimeoutError::Disconnected) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&request_id);
                }
                Err(CoordinatorMcpLifecycleRelayError::Closed(format!("reply channel disconnected for {request_id}")))
            }
        }
    }

    pub fn resolve(&self, args: &Value) -> Result<Value, CoordinatorMcpLifecycleRelayError> {
        let request_id = args
            .get("requestId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| CoordinatorMcpLifecycleRelayError::Protocol("mcp.resolveLifecycleRequest requires requestId".into()))?
            .to_string();
        let ok = args
            .get("ok")
            .and_then(Value::as_bool)
            .ok_or_else(|| CoordinatorMcpLifecycleRelayError::Protocol("mcp.resolveLifecycleRequest requires boolean ok".into()))?;
        let sender = self.pending
            .lock()
            .map_err(|_| CoordinatorMcpLifecycleRelayError::Protocol("pending relay lock poisoned".into()))?
            .remove(&request_id)
            .ok_or_else(|| CoordinatorMcpLifecycleRelayError::Protocol(format!("unknown or already-settled Host MCP lifecycle request {request_id}")))?;

        let outcome = if ok {
            Ok(args.get("result").cloned().unwrap_or(Value::Null))
        } else {
            Err(CoordinatorMcpLifecycleRelayError::Remote {
                request_id: request_id.clone(),
                message: args
                    .get("error")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("Host MCP lifecycle request failed")
                    .to_string(),
            })
        };
        sender
            .send(outcome)
            .map_err(|_| CoordinatorMcpLifecycleRelayError::Closed(format!("request waiter disappeared for {request_id}")))?;
        Ok(json!({"resolved": true, "requestId": request_id}))
    }

    pub fn cancel_all(&self, reason: impl Into<String>) {
        self.closed.store(true, Ordering::Release);
        let reason = reason.into();
        let pending = self.pending
            .lock()
            .map(|mut pending| pending.drain().collect::<Vec<_>>())
            .unwrap_or_default();
        for (_request_id, sender) in pending {
            let _ = sender.send(Err(CoordinatorMcpLifecycleRelayError::Closed(reason.clone())));
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending.lock().map(|pending| pending.len()).unwrap_or_default()
    }
}
