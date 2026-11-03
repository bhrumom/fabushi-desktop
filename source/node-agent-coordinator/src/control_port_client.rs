use std::collections::HashMap;
use std::io;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use serde_json::Value;

use crate::protocol::{
    CoordinatorFrame, Failure, LifecyclePhase, ReplyOutcome, ShutdownReason,
    COORDINATOR_DISCONNECTED, COORDINATOR_PROTOCOL_VERSION,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ClientAction {
    Post(CoordinatorFrame),
    Resolve { request_id: String, value: Value },
    Reject { request_id: String, failure: Failure },
    Event { family: String, payload: Value },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlPortPhase {
    Connecting,
    Serving,
    Settled,
}

pub type ControlCallResult = Result<Value, Failure>;

pub struct ControlPortWaiter {
    pub request_id: String,
    pub action: ClientAction,
    pub reply: Receiver<ControlCallResult>,
}

pub struct ControlPortClient {
    phase: ControlPortPhase,
    ready_observed: bool,
    next_request_id: u64,
    pending: HashMap<String, String>,
    waiters: HashMap<String, Sender<ControlCallResult>>,
}

struct ControlCallRetirement<'a> {
    client: &'a Mutex<ControlPortClient>,
    request_id: String,
}

impl Drop for ControlCallRetirement<'_> {
    fn drop(&mut self) {
        let mut client = self.client.lock().unwrap_or_else(|error| error.into_inner());
        client.retire_request(&self.request_id);
    }
}

/// Retires this call's local wait state on every exit, without cancelling remote work.
pub fn call_control_command<WriteFrame>(
    client: &Mutex<ControlPortClient>,
    method: &str,
    args: Value,
    timeout: Duration,
    write_frame: WriteFrame,
) -> ControlCallResult
where
    WriteFrame: FnOnce(&CoordinatorFrame) -> io::Result<()>,
{
    let waiter = client
        .lock()
        .map_err(|_| Failure::new(COORDINATOR_DISCONNECTED, "control port lock poisoned"))?
        .call_waiting(method.to_string(), args)?;
    let _retirement = ControlCallRetirement {
        client,
        request_id: waiter.request_id,
    };
    let frame = match waiter.action {
        ClientAction::Post(frame) => frame,
        _ => {
            return Err(Failure::new(
                "COORDINATOR_CONTROL_PROTOCOL_ERROR",
                "control command did not produce a request frame",
            ));
        }
    };
    write_frame(&frame).map_err(|error| {
        Failure::new(
            COORDINATOR_DISCONNECTED,
            format!("control request write failed: {error}"),
        )
    })?;
    waiter.reply.recv_timeout(timeout).map_err(|error| {
        Failure::new(
            COORDINATOR_DISCONNECTED,
            format!("control request {method} did not settle: {error}"),
        )
    })?
}

impl Default for ControlPortClient {
    fn default() -> Self {
        Self {
            phase: ControlPortPhase::Connecting,
            ready_observed: false,
            next_request_id: 0,
            pending: HashMap::new(),
            waiters: HashMap::new(),
        }
    }
}

impl ControlPortClient {
    pub fn start(&self) -> ClientAction { ClientAction::Post(CoordinatorFrame::hello()) }
    pub fn phase(&self) -> ControlPortPhase { self.phase }
    pub fn pending_count(&self) -> usize { self.pending.len() }
    pub fn waiter_count(&self) -> usize { self.waiters.len() }

    pub fn handle_value(&mut self, value: Value) -> Vec<ClientAction> {
        match serde_json::from_value::<CoordinatorFrame>(value) {
            Ok(frame) => self.handle_frame(frame),
            Err(error) => self.protocol_breach(&format!("invalid coordinator frame: {error}")),
        }
    }

    pub fn call(&mut self, method: impl Into<String>, args: Value) -> Result<(String, ClientAction), Failure> {
        if self.phase == ControlPortPhase::Settled {
            return Err(Failure::new(COORDINATOR_DISCONNECTED, "control port is settled"));
        }
        self.next_request_id += 1;
        let request_id = format!("c-{}", self.next_request_id);
        let method = method.into();
        self.pending.insert(request_id.clone(), method.clone());
        Ok((
            request_id.clone(),
            ClientAction::Post(CoordinatorFrame::Request { request_id, method, args }),
        ))
    }

    pub fn call_waiting(
        &mut self,
        method: impl Into<String>,
        args: Value,
    ) -> Result<ControlPortWaiter, Failure> {
        let (request_id, action) = self.call(method, args)?;
        let (sender, reply) = mpsc::channel();
        self.waiters.insert(request_id.clone(), sender);
        Ok(ControlPortWaiter {
            request_id,
            action,
            reply,
        })
    }

    pub fn cancel(&mut self, request_id: &str) -> Option<ClientAction> {
        self.pending.contains_key(request_id).then(|| ClientAction::Post(CoordinatorFrame::Cancel {
            request_id: request_id.to_string(),
        }))
    }

    pub fn post_event(&self, family: impl Into<String>, payload: Value) -> Option<ClientAction> {
        (self.phase != ControlPortPhase::Settled).then(|| ClientAction::Post(CoordinatorFrame::Event {
            family: family.into(),
            payload,
        }))
    }

    pub fn handle_frame(&mut self, frame: CoordinatorFrame) -> Vec<ClientAction> {
        if self.phase == ControlPortPhase::Settled {
            return Vec::new();
        }
        match frame {
            CoordinatorFrame::Lifecycle { phase: LifecyclePhase::Ready, protocol_version, .. } => {
                if self.ready_observed || protocol_version != Some(COORDINATOR_PROTOCOL_VERSION) {
                    return self.protocol_breach("invalid or repeated ready frame");
                }
                self.ready_observed = true;
                self.phase = ControlPortPhase::Serving;
                Vec::new()
            }
            CoordinatorFrame::Lifecycle { phase: LifecyclePhase::Shutdown, reason, detail, .. } => {
                let message = detail.unwrap_or_else(|| format!("coordinator shutdown: {reason:?}"));
                self.settle(Failure::new(COORDINATOR_DISCONNECTED, message))
            }
            CoordinatorFrame::Reply { request_id, outcome } => {
                if self.pending.remove(&request_id).is_none() {
                    return Vec::new();
                }
                match outcome {
                    ReplyOutcome::Ok { value } => {
                        if let Some(waiter) = self.waiters.remove(&request_id) {
                            let _ = waiter.send(Ok(value.clone()));
                        }
                        vec![ClientAction::Resolve { request_id, value }]
                    }
                    ReplyOutcome::Failed { failure } => {
                        if let Some(waiter) = self.waiters.remove(&request_id) {
                            let _ = waiter.send(Err(failure.clone()));
                        }
                        vec![ClientAction::Reject { request_id, failure }]
                    }
                }
            }
            CoordinatorFrame::Event { .. }
            | CoordinatorFrame::Request { .. }
            | CoordinatorFrame::Cancel { .. }
            | CoordinatorFrame::Lifecycle { phase: LifecyclePhase::Hello, .. } => {
                self.protocol_breach("server posted a client-direction frame")
            }
        }
    }

    pub fn handle_port_closed(&mut self) -> Vec<ClientAction> {
        self.settle(Failure::new(COORDINATOR_DISCONNECTED, "control port closed"))
    }

    pub fn shutdown(&mut self) -> Vec<ClientAction> {
        if self.phase == ControlPortPhase::Settled {
            return Vec::new();
        }
        let mut actions = vec![ClientAction::Post(CoordinatorFrame::shutdown(
            ShutdownReason::Requested,
            None,
        ))];
        actions.extend(self.settle(Failure::new(COORDINATOR_DISCONNECTED, "shutdown requested")));
        actions
    }

    fn retire_request(&mut self, request_id: &str) {
        self.pending.remove(request_id);
        self.waiters.remove(request_id);
    }

    fn protocol_breach(&mut self, detail: &str) -> Vec<ClientAction> {
        let mut actions = vec![ClientAction::Post(CoordinatorFrame::shutdown(
            ShutdownReason::ProtocolError,
            Some(detail.to_string()),
        ))];
        actions.extend(self.settle(Failure::new(COORDINATOR_DISCONNECTED, detail)));
        actions
    }

    fn settle(&mut self, failure: Failure) -> Vec<ClientAction> {
        if self.phase == ControlPortPhase::Settled {
            return Vec::new();
        }
        self.phase = ControlPortPhase::Settled;
        let pending = std::mem::take(&mut self.pending);
        let mut actions = Vec::with_capacity(pending.len().saturating_add(1));
        for request_id in pending.into_keys() {
            if let Some(waiter) = self.waiters.remove(&request_id) {
                let _ = waiter.send(Err(failure.clone()));
            }
            actions.push(ClientAction::Reject {
                request_id,
                failure: failure.clone(),
            });
        }
        self.waiters.clear();
        actions.push(ClientAction::Close);
        actions
    }
}
