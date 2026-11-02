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

type PendingReply = SyncSender<Result<Value, CoordinatorToolRelayError>>;

pub struct CoordinatorToolRelay {
    events: GatewayEventHub,
    pending: Mutex<HashMap<String, PendingReply>>,
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
        if !matches!(method, ROUTED_TOOL_LIST_METHOD | ROUTED_TOOL_EXECUTE_METHOD) {
            return Err(CoordinatorToolRelayError::UnsupportedMethod(
                method.to_string(),
            ));
        }
        if self.closed.load(Ordering::Acquire) {
            return Err(CoordinatorToolRelayError::Closed(
                "Mahayana Host is shutting down".into(),
            ));
        }

        let sequence = self.next_request_id.fetch_add(1, Ordering::AcqRel) + 1;
        let request_id = format!("runner-tool-{sequence}");
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        #[cfg(test)]
        if let Some(before_registration) = &self.before_pending_registration {
            before_registration();
        }
        {
            let mut pending = self.pending.lock().map_err(|_| {
                CoordinatorToolRelayError::Protocol("pending relay lock poisoned".into())
            })?;
            // Host shutdown drains under the same lock. Register before that
            // drain, or refuse closure without publishing an unowned request.
            if self.closed.load(Ordering::Acquire) {
                return Err(CoordinatorToolRelayError::Closed(
                    "Mahayana Host is shutting down".into(),
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

        match reply_rx.recv_timeout(self.timeout) {
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
        }
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

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

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
