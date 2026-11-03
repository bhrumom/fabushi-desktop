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

pub const RUNNER_TOOL_REQUEST_EVENT_CHANNEL: &str = "runner-tool-request";
pub const RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD: &str =
    "runner.resolveRoutedToolRequest";
pub const ROUTED_TOOL_LIST_METHOD: &str = "listRoutedMcpTools";
pub const ROUTED_TOOL_EXECUTE_METHOD: &str = "executeRoutedMcpTool";

const DEFAULT_ROUTED_TOOL_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Error, PartialEq)]
pub enum CoordinatorToolRelayError {
    #[error("unsupported routed tool relay method: {0}")]
    UnsupportedMethod(String),
    #[error("routed tool relay request {0} timed out")]
    Timeout(String),
    #[error("routed tool relay request {request_id} failed: {message}")]
    Remote {
        request_id: String,
        message: String,
    },
    #[error("routed tool relay is closed: {0}")]
    Closed(String),
    #[error("invalid routed tool relay response: {0}")]
    Protocol(String),
}

/// Local request failure and the execution disposition observed by this owner.
/// An error reply after publication cannot prove the downstream tool stopped.
#[derive(Debug, Clone, Error, PartialEq)]
#[error("{error}")]
pub struct CoordinatorToolExecutionError {
    error: CoordinatorToolRelayError,
    outcome_unknown: bool,
}

impl CoordinatorToolExecutionError {
    fn before_publication(error: CoordinatorToolRelayError) -> Self {
        Self {
            error,
            outcome_unknown: false,
        }
    }

    fn after_publication(error: CoordinatorToolRelayError) -> Self {
        Self {
            error,
            outcome_unknown: true,
        }
    }

    pub fn outcome_unknown(&self) -> bool {
        self.outcome_unknown
    }

    pub fn into_error(self) -> CoordinatorToolRelayError {
        self.error
    }
}

type PendingReply = SyncSender<Result<Value, CoordinatorToolRelayError>>;

pub struct CoordinatorToolRelay {
    events: GatewayEventHub,
    pending: Mutex<HashMap<String, PendingReply>>,
    incarnation: uuid::Uuid,
    next_request_id: AtomicU64,
    timeout: Duration,
    closed: AtomicBool,
    #[cfg(test)]
    before_pending_registration: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
}

impl CoordinatorToolRelay {
    pub fn new(events: GatewayEventHub) -> Self {
        Self::with_timeout(events, DEFAULT_ROUTED_TOOL_REQUEST_TIMEOUT)
    }

    pub fn with_timeout(events: GatewayEventHub, timeout: Duration) -> Self {
        Self {
            events,
            pending: Mutex::new(HashMap::new()),
            incarnation: uuid::Uuid::new_v4(),
            next_request_id: AtomicU64::new(0),
            timeout,
            closed: AtomicBool::new(false),
            #[cfg(test)]
            before_pending_registration: None,
        }
    }

    pub fn request(
        &self,
        method: &str,
        args: Value,
    ) -> Result<Value, CoordinatorToolRelayError> {
        self.request_with_execution_outcome(method, args)
            .map_err(CoordinatorToolExecutionError::into_error)
    }

    /// Execute through the same relay while retaining uncertainty hidden by
    /// local waiter errors and the existing MCP transport-error result arm.
    pub fn request_tool_execution(
        &self,
        args: Value,
    ) -> Result<Value, CoordinatorToolExecutionError> {
        let value = self.request_with_execution_outcome(ROUTED_TOOL_EXECUTE_METHOD, args)?;
        if !has_definitive_mcp_result(&value) {
            return Err(CoordinatorToolExecutionError::after_publication(
                CoordinatorToolRelayError::Protocol(
                    "routed MCP execution returned no definitive result; its outcome remains unknown".into(),
                ),
            ));
        }
        Ok(value)
    }

    fn request_with_execution_outcome(
        &self,
        method: &str,
        args: Value,
    ) -> Result<Value, CoordinatorToolExecutionError> {
        if !matches!(method, ROUTED_TOOL_LIST_METHOD | ROUTED_TOOL_EXECUTE_METHOD) {
            return Err(CoordinatorToolExecutionError::before_publication(
                CoordinatorToolRelayError::UnsupportedMethod(method.to_string()),
            ));
        }
        if self.closed.load(Ordering::Acquire) {
            return Err(CoordinatorToolExecutionError::before_publication(
                CoordinatorToolRelayError::Closed("Mahayana Host is shutting down".into()),
            ));
        }

        let sequence = self
            .next_request_id
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |sequence| {
                sequence.checked_add(1)
            })
            .map_err(|_| {
                CoordinatorToolExecutionError::before_publication(
                    CoordinatorToolRelayError::Protocol(
                        "routed tool request sequence exhausted".into(),
                    ),
                )
            })? + 1;
        let request_id = format!("runner-tool-{}-{sequence}", self.incarnation);
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        #[cfg(test)]
        if let Some(before_registration) = &self.before_pending_registration {
            before_registration();
        }
        {
            let mut pending = self.pending.lock().map_err(|_| {
                CoordinatorToolExecutionError::before_publication(
                    CoordinatorToolRelayError::Protocol("pending relay lock poisoned".into()),
                )
            })?;
            // Host shutdown drains under the same lock. Register before that
            // drain, or refuse closure without publishing an unowned request.
            if self.closed.load(Ordering::Acquire) {
                return Err(CoordinatorToolExecutionError::before_publication(
                    CoordinatorToolRelayError::Closed("Mahayana Host is shutting down".into()),
                ));
            }
            pending.insert(request_id.clone(), reply_tx);
        }

        // Gateway listeners may synchronously resolve or close this relay.
        self.events.publish(json!({
            "channel": RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
            "payload": {
                "requestId": request_id,
                "method": method,
                "args": args,
            }
        }));

        let result = match reply_rx.recv_timeout(self.timeout) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&request_id);
                }
                Err(CoordinatorToolRelayError::Timeout(request_id))
            }
            Err(RecvTimeoutError::Disconnected) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&request_id);
                }
                Err(CoordinatorToolRelayError::Closed(format!(
                    "reply channel disconnected for {request_id}"
                )))
            }
        };
        // This includes ordinary Remote errors: Coordinator may have timed out
        // waiting for Electron while the actual external operation continues.
        result.map_err(CoordinatorToolExecutionError::after_publication)
    }

    pub fn resolve(&self, args: &Value) -> Result<Value, CoordinatorToolRelayError> {
        let request_id = args
            .get("requestId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                CoordinatorToolRelayError::Protocol(
                    "runner.resolveRoutedToolRequest requires requestId".into(),
                )
            })?
            .to_string();
        let ok = args.get("ok").and_then(Value::as_bool).ok_or_else(|| {
            CoordinatorToolRelayError::Protocol(
                "runner.resolveRoutedToolRequest requires boolean ok".into(),
            )
        })?;
        let sender = self
            .pending
            .lock()
            .map_err(|_| {
                CoordinatorToolRelayError::Protocol("pending relay lock poisoned".into())
            })?
            .remove(&request_id)
            .ok_or_else(|| {
                CoordinatorToolRelayError::Protocol(format!(
                    "unknown or already-settled routed tool request {request_id}"
                ))
            })?;

        let outcome = if ok {
            Ok(args.get("result").cloned().unwrap_or(Value::Null))
        } else {
            let message = args
                .get("error")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("routed MCP tool request failed")
                .to_string();
            Err(CoordinatorToolRelayError::Remote {
                request_id: request_id.clone(),
                message,
            })
        };
        sender.send(outcome).map_err(|_| {
            CoordinatorToolRelayError::Closed(format!(
                "request waiter disappeared for {request_id}"
            ))
        })?;
        Ok(json!({
            "resolved": true,
            "requestId": request_id,
        }))
    }

    pub fn cancel_all(&self, reason: impl Into<String>) {
        self.closed.store(true, Ordering::Release);
        let reason = reason.into();
        let pending = self
            .pending
            .lock()
            .map(|mut pending| pending.drain().collect::<Vec<_>>())
            .unwrap_or_default();
        for (_request_id, sender) in pending {
            let _ = sender.send(Err(CoordinatorToolRelayError::Closed(reason.clone())));
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending
            .lock()
            .map(|pending| pending.len())
            .unwrap_or_default()
    }
}

fn optional_string(value: &serde_json::Map<String, Value>, field: &str) -> bool {
    value.get(field).is_none_or(Value::is_string)
}

fn optional_bool(value: &serde_json::Map<String, Value>, field: &str) -> bool {
    value.get(field).is_none_or(Value::is_boolean)
}

fn optional_strings(value: &serde_json::Map<String, Value>, field: &str) -> bool {
    value.get(field).is_none_or(|value| {
        value.as_array().is_some_and(|items| items.iter().all(Value::is_string))
    })
}

fn valid_mcp_content(value: &serde_json::Map<String, Value>) -> bool {
    optional_bool(value, "isError")
        && value.get("content").is_none_or(|content| {
            content.as_array().is_some_and(|items| items.iter().all(Value::is_object))
        })
        && value.get("structuredContent").is_none_or(Value::is_object)
}

/// Interpret only the execution result envelope, never strings or nested user
/// payloads. Generated protobuf JSON can omit a message's default fields.
fn has_definitive_mcp_result(value: &Value) -> bool {
    const ARMS: [&str; 7] = [
        "success", "error", "rejected", "permissionDenied",
        "toolNotFound", "serverNotFound", "approved",
    ];
    let Some(root) = value.as_object() else {
        return false;
    };
    let arms = ARMS
        .iter()
        .filter(|arm| root.contains_key(**arm))
        .copied()
        .collect::<Vec<_>>();
    let raw_content = root.contains_key("content") || root.contains_key("structuredContent");
    if raw_content {
        return !root.contains_key("result") && arms.is_empty() && valid_mcp_content(root);
    }
    let (case, payload) = if let Some(result) = root.get("result") {
        if !arms.is_empty() {
            return false;
        }
        let Some(result) = result.as_object() else {
            return false;
        };
        let Some(case) = result.get("case").and_then(Value::as_str) else {
            return false;
        };
        let Some(payload) = result.get("value").and_then(Value::as_object) else {
            return false;
        };
        (case, payload)
    } else {
        if arms.len() != 1 {
            return false;
        }
        let case = arms[0];
        let Some(payload) = root.get(case).and_then(Value::as_object) else {
            return false;
        };
        (case, payload)
    };
    match case {
        "success" => valid_mcp_content(payload),
        // These variants explicitly describe a pre-effect decision. Preserve
        // their value; approval alone is not represented as executed success.
        "rejected" => optional_string(payload, "reason") && optional_bool(payload, "isReadonly"),
        "permissionDenied" => {
            optional_string(payload, "error") && optional_bool(payload, "isReadonly")
        }
        "toolNotFound" => {
            optional_string(payload, "name") && optional_strings(payload, "availableTools")
        }
        "serverNotFound" => {
            optional_string(payload, "name") && optional_strings(payload, "availableServers")
        }
        "approved" => true,
        // Both official and backend adapters put uncertain transport failures
        // in error. Unknown/malformed arms supply no completion authority.
        _ => false,
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn execution_sequence_exhaustion_refuses_before_publication_without_losing_pending_reply() {
        let events = GatewayEventHub::default();
        let receiver = events.subscribe();
        let relay = Arc::new(CoordinatorToolRelay::with_timeout(events, Duration::from_secs(30)));
        relay.next_request_id.store(u64::MAX - 1, Ordering::Release);
        let worker_relay = Arc::clone(&relay);
        let worker = thread::spawn(move || {
            worker_relay.request_tool_execution(json!({"toolName":"last-sequence"}))
        });
        let event = receiver.recv_timeout(Duration::from_secs(5)).expect("last request published");
        let request_id = event["payload"]["requestId"].as_str().expect("request id");
        assert_eq!(request_id, format!("runner-tool-{}-{}", relay.incarnation, u64::MAX));
        assert_eq!(relay.pending_count(), 1);

        let error = relay.request_tool_execution(json!({"toolName":"must-not-publish"}))
            .expect_err("checked sequence must not recycle");
        assert!(!error.outcome_unknown(), "refused before external dispatch");
        assert_eq!(error.into_error(), CoordinatorToolRelayError::Protocol(
            "routed tool request sequence exhausted".into(),
        ));
        assert_eq!(relay.request(ROUTED_TOOL_LIST_METHOD, json!({})),
            Err(CoordinatorToolRelayError::Protocol("routed tool request sequence exhausted".into())));
        assert_eq!(relay.next_request_id.load(Ordering::Acquire), u64::MAX);
        assert_eq!(relay.pending_count(), 1, "exhaustion cannot displace the last admitted waiter");
        assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));

        let result = json!({"content":[{"type":"text","text":"last request completed"}]});
        relay.resolve(&json!({"requestId":request_id,"ok":true,"result":result.clone()}))
            .expect("resolve still-owned last request");
        assert_eq!(worker.join().expect("last request worker"), Ok(result));
        assert_eq!(relay.pending_count(), 0);
        assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn close_before_registration_refuses_both_real_tool_request_methods() {
        for method in [ROUTED_TOOL_LIST_METHOD, ROUTED_TOOL_EXECUTE_METHOD] {
            let events = GatewayEventHub::default();
            let event_rx = events.subscribe();
            let (entered_tx, entered_rx) = mpsc::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let release_rx = Mutex::new(release_rx);
            let mut relay = CoordinatorToolRelay::with_timeout(events, Duration::from_secs(30));
            relay.before_pending_registration = Some(Arc::new(move || {
                entered_tx.send(()).expect("actual request passed early open check");
                release_rx.lock().unwrap().recv_timeout(Duration::from_secs(5))
                    .expect("release registration after close");
            }));
            let relay = Arc::new(relay);
            let (result_tx, result_rx) = mpsc::channel();
            let worker_relay = Arc::clone(&relay);
            let worker = thread::spawn(move || {
                result_tx.send(worker_relay.request(method, json!({"toolName":"bounded-probe"})))
                    .expect("return actual request result");
            });
            entered_rx.recv_timeout(Duration::from_secs(5)).expect("registration barrier");
            assert_eq!(relay.pending_count(), 0);
            relay.cancel_all("Host shutdown before registration");
            release_tx.send(()).expect("release exact request");
            assert_eq!(
                result_rx.recv_timeout(Duration::from_secs(5))
                    .expect("close must settle before the ordinary request timeout"),
                Err(CoordinatorToolRelayError::Closed("Mahayana Host is shutting down".into())),
            );
            worker.join().expect("owned request worker");
            assert_eq!(relay.pending_count(), 0);
            assert!(matches!(event_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert!(matches!(relay.request(method, json!({})), Err(CoordinatorToolRelayError::Closed(_))));
            assert!(matches!(event_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }
    }
}
