//! Transport-neutral Mahayana gateway state and JSON-RPC dispatcher.
//!
//! The renderer, stdio CLI, WebSocket server and native clients must all consume
//! the same Rust-owned event semantics. This crate adapts the existing Mahayana
//! runtime; it does not introduce another agent kernel and it does not depend on
//! the Hermes Python runtime.

use chrono::{DateTime, SecondsFormat, Utc};
use mahayana_core::{MessageRole, RuntimeActivityStatus, RuntimeEvent};
use mahayana_gateway_protocol::{
    ApprovalRequestPayload, GatewayEvent, GatewayEventEnvelope, MessageCompletePayload,
    MessageCompletionStatus, MessageDeltaPayload, MessageStartPayload, StreamDeltaPayload,
    ToolCompletePayload, ToolGeneratingPayload, ToolStartPayload,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet, VecDeque};

pub const DEFAULT_REPLAY_EVENTS_PER_SESSION: usize = 512;
pub const DEFAULT_REPLAY_BYTES_PER_SESSION: usize = 4 * 1024 * 1024;
pub const DEFAULT_REPLAY_SESSIONS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayLimits {
    pub max_events_per_session: usize,
    pub max_bytes_per_session: usize,
    pub max_sessions: usize,
}

impl Default for ReplayLimits {
    fn default() -> Self {
        Self {
            max_events_per_session: DEFAULT_REPLAY_EVENTS_PER_SESSION,
            max_bytes_per_session: DEFAULT_REPLAY_BYTES_PER_SESSION,
            max_sessions: DEFAULT_REPLAY_SESSIONS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaySlice {
    pub replay_epoch: String,
    pub session_id: String,
    pub last_seen: u64,
    pub latest_seq: u64,
    pub truncated: bool,
    pub reset: bool,
    pub events: Vec<GatewayEventEnvelope>,
}

#[derive(Debug, Clone)]
struct GatewayEventDraft {
    session_id: String,
    turn_id: String,
    timestamp_ms: i64,
    event: GatewayEvent,
}

impl GatewayEventDraft {
    fn new(
        session_id: impl Into<String>,
        turn_id: impl Into<String>,
        timestamp_ms: i64,
        event: GatewayEvent,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            turn_id: turn_id.into(),
            timestamp_ms,
            event,
        }
    }
}

#[derive(Debug, Default)]
struct SessionReplay {
    next_seq: u64,
    bytes: usize,
    evicted_through: u64,
    events: VecDeque<(GatewayEventEnvelope, usize)>,
}

#[derive(Debug)]
struct ReplayStore {
    epoch: String,
    limits: ReplayLimits,
    sessions: HashMap<String, SessionReplay>,
    session_order: VecDeque<String>,
    forgotten_latest: HashMap<String, u64>,
    forgotten_order: VecDeque<String>,
}

impl ReplayStore {
    fn new(limits: ReplayLimits) -> Self {
        Self::with_epoch(limits, uuid::Uuid::new_v4().simple().to_string())
    }

    fn with_epoch(limits: ReplayLimits, epoch: impl Into<String>) -> Self {
        Self {
            epoch: epoch.into(),
            limits: ReplayLimits {
                max_events_per_session: limits.max_events_per_session.max(1),
                max_bytes_per_session: limits.max_bytes_per_session.max(1),
                max_sessions: limits.max_sessions.max(1),
            },
            sessions: HashMap::new(),
            session_order: VecDeque::new(),
            forgotten_latest: HashMap::new(),
            forgotten_order: VecDeque::new(),
        }
    }

    fn epoch(&self) -> &str {
        &self.epoch
    }

    fn push(&mut self, draft: GatewayEventDraft) -> GatewayEventEnvelope {
        self.ensure_session(&draft.session_id);
        let session = self
            .sessions
            .get_mut(&draft.session_id)
            .expect("session exists after ensure_session");
        session.next_seq = session.next_seq.saturating_add(1);
        let envelope = GatewayEventEnvelope::new(
            draft.session_id,
            draft.turn_id,
            session.next_seq,
            timestamp_string(draft.timestamp_ms),
            self.epoch.clone(),
            draft.event,
        );
        let encoded_size = serde_json::to_vec(&envelope)
            .map(|bytes| bytes.len())
            .unwrap_or(usize::MAX);

        // The event still consumes its sequence number. A client replaying from
        // before an oversized/evicted event receives truncated=true and must
        // reload canonical session history instead of silently skipping data.
        if encoded_size > self.limits.max_bytes_per_session {
            session.evicted_through = envelope.seq;
            return envelope;
        }

        session.events.push_back((envelope.clone(), encoded_size));
        session.bytes = session.bytes.saturating_add(encoded_size);
        while session.events.len() > self.limits.max_events_per_session
            || session.bytes > self.limits.max_bytes_per_session
        {
            if let Some((removed, bytes)) = session.events.pop_front() {
                session.bytes = session.bytes.saturating_sub(bytes);
                session.evicted_through = session.evicted_through.max(removed.seq);
            } else {
                break;
            }
        }
        envelope
    }

    fn since(&self, session_id: &str, last_seen: u64, reset: bool) -> ReplaySlice {
        if reset {
            let latest_seq = self
                .sessions
                .get(session_id)
                .map(|session| session.next_seq)
                .or_else(|| self.forgotten_latest.get(session_id).copied())
                .unwrap_or_default();
            return ReplaySlice {
                replay_epoch: self.epoch.clone(),
                session_id: session_id.to_string(),
                last_seen,
                latest_seq,
                truncated: true,
                reset: true,
                events: Vec::new(),
            };
        }

        if let Some(session) = self.sessions.get(session_id) {
            return ReplaySlice {
                replay_epoch: self.epoch.clone(),
                session_id: session_id.to_string(),
                last_seen,
                latest_seq: session.next_seq,
                truncated: last_seen < session.evicted_through || last_seen > session.next_seq,
                reset: false,
                events: session
                    .events
                    .iter()
                    .filter(|(event, _)| event.seq > last_seen)
                    .map(|(event, _)| event.clone())
                    .collect(),
            };
        }

        let latest_seq = self
            .forgotten_latest
            .get(session_id)
            .copied()
            .unwrap_or_default();
        ReplaySlice {
            replay_epoch: self.epoch.clone(),
            session_id: session_id.to_string(),
            last_seen,
            latest_seq,
            truncated: last_seen > 0 || last_seen < latest_seq,
            reset: false,
            events: Vec::new(),
        }
    }

    fn ensure_session(&mut self, session_id: &str) {
        if self.sessions.contains_key(session_id) {
            if let Some(index) = self
                .session_order
                .iter()
                .position(|item| item == session_id)
            {
                self.session_order.remove(index);
            }
            self.session_order.push_back(session_id.to_string());
            return;
        }

        while self.sessions.len() >= self.limits.max_sessions {
            let Some(oldest) = self.session_order.pop_front() else {
                break;
            };
            if let Some(removed) = self.sessions.remove(&oldest) {
                self.remember_forgotten(oldest, removed.next_seq);
            }
        }

        let mut session = SessionReplay::default();
        session.next_seq = self.forgotten_latest.remove(session_id).unwrap_or_default();
        if let Some(index) = self
            .forgotten_order
            .iter()
            .position(|item| item == session_id)
        {
            self.forgotten_order.remove(index);
        }
        session.evicted_through = session.next_seq;
        self.sessions.insert(session_id.to_string(), session);
        self.session_order.push_back(session_id.to_string());
    }

    fn remember_forgotten(&mut self, session_id: String, latest_seq: u64) {
        self.forgotten_latest.insert(session_id.clone(), latest_seq);
        if let Some(index) = self
            .forgotten_order
            .iter()
            .position(|item| item == &session_id)
        {
            self.forgotten_order.remove(index);
        }
        self.forgotten_order.push_back(session_id);
        while self.forgotten_order.len() > self.limits.max_sessions {
            if let Some(oldest) = self.forgotten_order.pop_front() {
                self.forgotten_latest.remove(&oldest);
            }
        }
    }
}

#[derive(Debug, Default)]
struct RuntimeProjection {
    sessions_by_turn: HashMap<String, String>,
    message_completed_turns: HashSet<String>,
    usage_by_turn: HashMap<String, Value>,
}

impl RuntimeProjection {
    fn register_turn(&mut self, turn_id: impl Into<String>, session_id: impl Into<String>) {
        self.sessions_by_turn
            .insert(turn_id.into(), session_id.into());
    }

    fn session_for_turn(&self, turn_id: &str) -> Option<&str> {
        self.sessions_by_turn.get(turn_id).map(String::as_str)
    }

    fn forget_turn(&mut self, turn_id: &str) {
        self.sessions_by_turn.remove(turn_id);
        self.message_completed_turns.remove(turn_id);
        self.usage_by_turn.remove(turn_id);
    }

    fn project(&mut self, event: &RuntimeEvent, timestamp_ms: i64) -> Vec<GatewayEventDraft> {
        match event {
            RuntimeEvent::Ready { .. }
            | RuntimeEvent::Lagged { .. }
            | RuntimeEvent::ProviderDegraded { .. } => Vec::new(),
            // Gateway v1 has no lifecycle frame. Native lifecycle state remains
            // authoritative; real message/operation events own this projection.
            // In particular, a late lifecycle event must not rebind an operation
            // already retired by interrupt or emit a second completion.
            RuntimeEvent::TurnStateChanged { .. } => Vec::new(),
            RuntimeEvent::MessageDelta {
                operation_id,
                conversation_id,
                delta,
            } => {
                self.register_turn(operation_id.0.clone(), conversation_id.0.clone());
                vec![GatewayEventDraft::new(
                    conversation_id.0.clone(),
                    operation_id.0.clone(),
                    timestamp_ms,
                    GatewayEvent::MessageDelta(MessageDeltaPayload {
                        text: delta.clone(),
                        rendered: None,
                    }),
                )]
            }
            RuntimeEvent::MessageCompleted {
                operation_id,
                message,
            } => {
                self.register_turn(operation_id.0.clone(), message.conversation_id.0.clone());
                if message.role != MessageRole::Assistant {
                    return Vec::new();
                }
                self.message_completed_turns.insert(operation_id.0.clone());
                let usage = self.usage_by_turn.get(operation_id.0.as_str()).cloned();
                vec![GatewayEventDraft::new(
                    message.conversation_id.0.clone(),
                    operation_id.0.clone(),
                    timestamp_ms,
                    GatewayEvent::MessageComplete(MessageCompletePayload {
                        status: MessageCompletionStatus::Complete,
                        text: Some(message.text.clone()),
                        partial: Some(false),
                        recoverable: None,
                        error: None,
                        usage,
                    }),
                )]
            }
            RuntimeEvent::ModelUsageUpdated {
                operation_id,
                usage,
            } => {
                if let Ok(value) = serde_json::to_value(usage) {
                    self.usage_by_turn.insert(operation_id.0.clone(), value);
                }
                Vec::new()
            }
            RuntimeEvent::ApprovalRequested {
                operation_id,
                approval_id,
                title,
                details,
            } => self.turn_draft(
                operation_id.0.as_str(),
                timestamp_ms,
                GatewayEvent::ApprovalRequest(ApprovalRequestPayload {
                    request_id: approval_id.0.clone(),
                    title: title.clone(),
                    description: details
                        .get("description")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    tool_id: details
                        .get("toolId")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    metadata: Some(details.clone()),
                }),
            ),
            RuntimeEvent::PluginProgress {
                operation_id,
                plugin_id,
                tool,
                progress,
                total,
                message,
            } => self.turn_draft(
                operation_id.0.as_str(),
                timestamp_ms,
                GatewayEvent::ToolGenerating(ToolGeneratingPayload {
                    tool_id: format!("plugin:{plugin_id}:{tool}"),
                    name: tool.clone(),
                    description: Some(format!("{message} ({progress}/{total})")),
                }),
            ),
            RuntimeEvent::AgentActivity {
                operation_id,
                step_id,
                kind,
                title,
                detail,
                status,
                metadata,
            } => {
                if kind == "reasoning" {
                    return self.turn_draft(
                        operation_id.0.as_str(),
                        timestamp_ms,
                        GatewayEvent::ReasoningDelta(StreamDeltaPayload {
                            text: detail.clone().unwrap_or_else(|| title.clone()),
                        }),
                    );
                }
                if kind == "thinking" {
                    return self.turn_draft(
                        operation_id.0.as_str(),
                        timestamp_ms,
                        GatewayEvent::ThinkingDelta(StreamDeltaPayload {
                            text: detail.clone().unwrap_or_else(|| title.clone()),
                        }),
                    );
                }

                let gateway_event = match status {
                    RuntimeActivityStatus::Running => GatewayEvent::ToolStart(ToolStartPayload {
                        tool_id: step_id.clone(),
                        name: title.clone(),
                        args: metadata.clone(),
                        description: detail.clone(),
                    }),
                    RuntimeActivityStatus::Completed => {
                        GatewayEvent::ToolComplete(ToolCompletePayload {
                            tool_id: step_id.clone(),
                            name: Some(title.clone()),
                            result: metadata
                                .clone()
                                .or_else(|| detail.clone().map(Value::String)),
                            error: None,
                        })
                    }
                    RuntimeActivityStatus::Failed => {
                        GatewayEvent::ToolComplete(ToolCompletePayload {
                            tool_id: step_id.clone(),
                            name: Some(title.clone()),
                            result: metadata.clone(),
                            error: Some(
                                detail.clone().unwrap_or_else(|| format!("{title} failed")),
                            ),
                        })
                    }
                };
                self.turn_draft(operation_id.0.as_str(), timestamp_ms, gateway_event)
            }
            RuntimeEvent::OperationCompleted { operation_id } => {
                let draft = if self
                    .message_completed_turns
                    .contains(operation_id.0.as_str())
                {
                    Vec::new()
                } else {
                    self.turn_draft(
                        operation_id.0.as_str(),
                        timestamp_ms,
                        GatewayEvent::MessageComplete(MessageCompletePayload {
                            status: MessageCompletionStatus::Complete,
                            text: None,
                            partial: None,
                            recoverable: None,
                            error: None,
                            usage: self.usage_by_turn.get(operation_id.0.as_str()).cloned(),
                        }),
                    )
                };
                self.forget_turn(operation_id.0.as_str());
                draft
            }
            RuntimeEvent::OperationFailed {
                operation_id,
                code,
                message,
            } => {
                let draft = self.turn_draft(
                    operation_id.0.as_str(),
                    timestamp_ms,
                    GatewayEvent::MessageComplete(MessageCompletePayload {
                        status: MessageCompletionStatus::Error,
                        text: None,
                        partial: Some(true),
                        recoverable: Some(true),
                        error: Some(format!("{code}: {message}")),
                        usage: self.usage_by_turn.get(operation_id.0.as_str()).cloned(),
                    }),
                );
                self.forget_turn(operation_id.0.as_str());
                draft
            }
        }
    }

    fn turn_draft(
        &self,
        turn_id: &str,
        timestamp_ms: i64,
        event: GatewayEvent,
    ) -> Vec<GatewayEventDraft> {
        self.session_for_turn(turn_id)
            .map(|session_id| {
                vec![GatewayEventDraft::new(
                    session_id,
                    turn_id,
                    timestamp_ms,
                    event,
                )]
            })
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct GatewayState {
    projection: RuntimeProjection,
    replay: ReplayStore,
}

impl GatewayState {
    pub fn new(limits: ReplayLimits) -> Self {
        Self {
            projection: RuntimeProjection::default(),
            replay: ReplayStore::new(limits),
        }
    }

    #[cfg(test)]
    fn with_epoch(limits: ReplayLimits, epoch: impl Into<String>) -> Self {
        Self {
            projection: RuntimeProjection::default(),
            replay: ReplayStore::with_epoch(limits, epoch),
        }
    }

    pub fn replay_epoch(&self) -> &str {
        self.replay.epoch()
    }

    pub fn register_turn(&mut self, turn_id: impl Into<String>, session_id: impl Into<String>) {
        self.projection.register_turn(turn_id, session_id);
    }

    pub fn emit(
        &mut self,
        session_id: impl Into<String>,
        turn_id: impl Into<String>,
        timestamp_ms: i64,
        event: GatewayEvent,
    ) -> GatewayEventEnvelope {
        self.replay.push(GatewayEventDraft::new(
            session_id,
            turn_id,
            timestamp_ms,
            event,
        ))
    }

    pub fn ingest_runtime_event(
        &mut self,
        event: &RuntimeEvent,
        timestamp_ms: i64,
    ) -> Vec<GatewayEventEnvelope> {
        self.projection
            .project(event, timestamp_ms)
            .into_iter()
            .map(|draft| self.replay.push(draft))
            .collect()
    }

    pub fn replay_since(
        &self,
        session_id: &str,
        last_seen: u64,
        expected_epoch: Option<&str>,
    ) -> ReplaySlice {
        let reset = expected_epoch.is_some_and(|epoch| epoch != self.replay_epoch());
        self.replay.since(session_id, last_seen, reset)
    }

    pub fn interrupt_turn(
        &mut self,
        turn_id: &str,
        timestamp_ms: i64,
    ) -> Vec<GatewayEventEnvelope> {
        let drafts = self.projection.turn_draft(
            turn_id,
            timestamp_ms,
            GatewayEvent::MessageComplete(MessageCompletePayload {
                status: MessageCompletionStatus::Interrupted,
                text: None,
                partial: Some(true),
                recoverable: Some(true),
                error: None,
                usage: self.projection.usage_by_turn.get(turn_id).cloned(),
            }),
        );
        self.projection.forget_turn(turn_id);
        drafts
            .into_iter()
            .map(|draft| self.replay.push(draft))
            .collect()
    }
}

pub trait GatewayRuntime {
    fn execute(&mut self, command: Value) -> Result<Value, String>;
    fn interrupt(&mut self, turn_id: &str) -> Result<Value, String>;
    fn resolve_approval(&mut self, approval_id: &str, decision: &str) -> Result<Value, String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispatchResult {
    pub result: Value,
    pub events: Vec<GatewayEventEnvelope>,
}

impl DispatchResult {
    fn value(result: Value) -> Self {
        Self {
            result,
            events: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcFailure {
    pub code: i64,
    pub message: String,
}

impl RpcFailure {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: message.into(),
        }
    }
}

pub fn dispatch_request<R: GatewayRuntime>(
    method: &str,
    params: &Value,
    runtime: &mut R,
    state: &mut GatewayState,
    timestamp_ms: i64,
) -> Result<DispatchResult, RpcFailure> {
    match method {
        "gateway.info" => Ok(DispatchResult::value(json!({
            "protocolVersion": mahayana_gateway_protocol::GATEWAY_PROTOCOL_VERSION,
            "replayEpoch": state.replay_epoch(),
            "runtime": "mahayana",
        }))),
        "gateway.shutdown" => Ok(DispatchResult::value(json!({ "shuttingDown": true }))),
        "prompt.submit" => {
            let session_id = string_param(params, "session_id", "sessionId")?;
            let text = params
                .get("text")
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
                .ok_or_else(|| RpcFailure::invalid("prompt.submit requires non-empty text"))?;
            let accepted = runtime
                .execute(json!({
                    "@type": "mahayana.conversation.send",
                    "conversationId": session_id,
                    "text": text,
                    "hidden": false,
                }))
                .map_err(RpcFailure::internal)?;
            let turn_id = accepted
                .get("operationId")
                .and_then(Value::as_str)
                .ok_or_else(|| RpcFailure::internal("runtime did not return operationId"))?
                .to_string();
            state.register_turn(turn_id.clone(), session_id.to_string());
            let start_event = state.emit(
                session_id,
                turn_id.as_str(),
                timestamp_ms,
                GatewayEvent::MessageStart(MessageStartPayload {
                    role: Some("assistant".to_string()),
                }),
            );
            Ok(DispatchResult {
                result: json!({ "sessionId": session_id, "turnId": turn_id }),
                events: vec![start_event],
            })
        }
        "session.list" => runtime
            .execute(json!({ "@type": "mahayana.conversation.list" }))
            .map(DispatchResult::value)
            .map_err(RpcFailure::internal),
        "session.history" => {
            let session_id = string_param(params, "session_id", "sessionId")?;
            let limit = params
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(100)
                .clamp(1, 500);
            runtime
                .execute(json!({
                    "@type": "mahayana.conversation.history",
                    "conversationId": session_id,
                    "limit": limit,
                }))
                .map(DispatchResult::value)
                .map_err(RpcFailure::internal)
        }
        "session.interrupt" => {
            let turn_id = string_param(params, "turn_id", "turnId")?;
            let result = runtime.interrupt(turn_id).map_err(RpcFailure::internal)?;
            let events = state.interrupt_turn(turn_id, timestamp_ms);
            Ok(DispatchResult { result, events })
        }
        "approval.respond" => {
            let approval_id = string_param(params, "approval_id", "approvalId")?;
            let decision = params
                .get("decision")
                .and_then(Value::as_str)
                .ok_or_else(|| RpcFailure::invalid("approval.respond requires decision"))?;
            let native_decision = match decision {
                "allow-once" | "accept" => "accept",
                "allow-session" | "accept-for-session" | "acceptForSession" => "acceptForSession",
                "deny" | "decline" => "decline",
                "cancel" => "cancel",
                _ => {
                    return Err(RpcFailure::invalid(
                        "decision must be allow-once/accept, allow-session/acceptForSession, or deny/decline/cancel",
                    ));
                }
            };
            runtime
                .resolve_approval(approval_id, native_decision)
                .map(DispatchResult::value)
                .map_err(RpcFailure::internal)
        }
        "session.events.since" => {
            let session_id = string_param(params, "session_id", "sessionId")?;
            let last_seen = params
                .get("last_seen")
                .or_else(|| params.get("lastSeen"))
                .and_then(Value::as_u64)
                .unwrap_or_default();
            let expected_epoch = params
                .get("replay_epoch")
                .or_else(|| params.get("replayEpoch"))
                .and_then(Value::as_str);
            serde_json::to_value(state.replay_since(session_id, last_seen, expected_epoch))
                .map(DispatchResult::value)
                .map_err(|error| RpcFailure::internal(error.to_string()))
        }
        _ => Err(RpcFailure {
            code: -32601,
            message: format!("method not found: {method}"),
        }),
    }
}

fn string_param<'a>(params: &'a Value, snake: &str, camel: &str) -> Result<&'a str, RpcFailure> {
    params
        .get(snake)
        .or_else(|| params.get(camel))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| RpcFailure::invalid(format!("missing {snake}")))
}

fn timestamp_string(timestamp_ms: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(timestamp_ms)
        .map(|timestamp| timestamp.to_rfc3339_opts(SecondsFormat::Millis, true))
        .unwrap_or_else(|| timestamp_ms.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahayana_core::{
        ApprovalId, ConversationId, Message, MessageId, OperationId, RunId,
        RuntimeActivityStatus, TurnId, TurnState,
    };

    #[derive(Default)]
    struct MockRuntime {
        commands: Vec<Value>,
        approval_calls: Vec<(String, String)>,
    }

    impl GatewayRuntime for MockRuntime {
        fn execute(&mut self, command: Value) -> Result<Value, String> {
            self.commands.push(command);
            Ok(json!({ "operationId": "turn-accepted" }))
        }

        fn interrupt(&mut self, turn_id: &str) -> Result<Value, String> {
            Ok(json!({ "operationId": turn_id, "interrupted": true }))
        }

        fn resolve_approval(&mut self, approval_id: &str, decision: &str) -> Result<Value, String> {
            self.approval_calls
                .push((approval_id.to_string(), decision.to_string()));
            Ok(json!({ "approvalId": approval_id, "decision": decision }))
        }
    }

    #[test]
    fn runtime_projection_preserves_reasoning_tool_text_completion_order() {
        let mut state = GatewayState::with_epoch(ReplayLimits::default(), "epoch-test");
        state.register_turn("turn-1", "session-1");

        let runtime_events = [
            RuntimeEvent::AgentActivity {
                operation_id: OperationId("turn-1".into()),
                step_id: "think-1".into(),
                kind: "reasoning".into(),
                title: "thinking".into(),
                detail: Some("先分析".into()),
                status: RuntimeActivityStatus::Running,
                metadata: None,
            },
            RuntimeEvent::MessageDelta {
                operation_id: OperationId("turn-1".into()),
                conversation_id: ConversationId("session-1".into()),
                delta: "先看文件。".into(),
            },
            RuntimeEvent::AgentActivity {
                operation_id: OperationId("turn-1".into()),
                step_id: "tool-1".into(),
                kind: "tool".into(),
                title: "read_repository".into(),
                detail: None,
                status: RuntimeActivityStatus::Running,
                metadata: Some(json!({ "path": "README.md" })),
            },
            RuntimeEvent::AgentActivity {
                operation_id: OperationId("turn-1".into()),
                step_id: "tool-1".into(),
                kind: "tool".into(),
                title: "read_repository".into(),
                detail: Some("ok".into()),
                status: RuntimeActivityStatus::Completed,
                metadata: None,
            },
            RuntimeEvent::MessageDelta {
                operation_id: OperationId("turn-1".into()),
                conversation_id: ConversationId("session-1".into()),
                delta: "结论如下。".into(),
            },
            RuntimeEvent::MessageCompleted {
                operation_id: OperationId("turn-1".into()),
                message: Message {
                    id: MessageId("message-1".into()),
                    conversation_id: ConversationId("session-1".into()),
                    role: MessageRole::Assistant,
                    text: "先看文件。结论如下。".into(),
                    created_at_ms: 6,
                    metadata: Value::Null,
                },
            },
            RuntimeEvent::OperationCompleted {
                operation_id: OperationId("turn-1".into()),
            },
        ];

        let projected: Vec<_> = runtime_events
            .iter()
            .enumerate()
            .flat_map(|(index, event)| state.ingest_runtime_event(event, index as i64 + 1))
            .collect();
        let kinds: Vec<_> = projected.iter().map(GatewayEventEnvelope::kind).collect();
        assert_eq!(
            kinds,
            vec![
                "reasoning.delta",
                "message.delta",
                "tool.start",
                "tool.complete",
                "message.delta",
                "message.complete",
            ]
        );
        assert_eq!(projected.last().map(|event| event.seq), Some(6));
    }

    #[test]
    fn replay_reports_eviction_instead_of_silently_skipping_events() {
        let limits = ReplayLimits {
            max_events_per_session: 2,
            max_bytes_per_session: usize::MAX,
            max_sessions: 2,
        };
        let mut state = GatewayState::with_epoch(limits, "epoch-test");
        for text in ["one", "two", "three"] {
            state.emit(
                "session-1",
                "turn-1",
                1,
                GatewayEvent::MessageDelta(MessageDeltaPayload {
                    text: text.into(),
                    rendered: None,
                }),
            );
        }
        let replay = state.replay_since("session-1", 0, Some("epoch-test"));
        assert!(replay.truncated);
        assert_eq!(replay.latest_seq, 3);
        assert_eq!(replay.events.len(), 2);
        assert_eq!(replay.events[0].seq, 2);
    }

    #[test]
    fn prompt_submit_registers_turn_and_returns_message_start_event() {
        let mut runtime = MockRuntime::default();
        let mut state = GatewayState::with_epoch(ReplayLimits::default(), "epoch-test");
        let response = dispatch_request(
            "prompt.submit",
            &json!({ "sessionId": "session-1", "text": "hello" }),
            &mut runtime,
            &mut state,
            123,
        )
        .unwrap();

        assert_eq!(response.result["sessionId"], "session-1");
        assert_eq!(response.result["turnId"], "turn-accepted");
        assert_eq!(response.events.len(), 1);
        assert_eq!(response.events[0].kind(), "message.start");
        assert_eq!(runtime.commands[0]["@type"], "mahayana.conversation.send");
    }

    #[test]
    fn replay_epoch_mismatch_requires_reset() {
        let mut state = GatewayState::with_epoch(ReplayLimits::default(), "epoch-current");
        state.emit(
            "session-1",
            "turn-1",
            1,
            GatewayEvent::MessageDelta(MessageDeltaPayload {
                text: "hello".into(),
                rendered: None,
            }),
        );
        let replay = state.replay_since("session-1", 1, Some("epoch-old"));
        assert!(replay.reset);
        assert!(replay.truncated);
        assert!(replay.events.is_empty());
        assert_eq!(replay.latest_seq, 1);
    }

    fn lifecycle_event(operation: &str, state: TurnState, sequence: u64) -> RuntimeEvent {
        RuntimeEvent::TurnStateChanged {
            operation_id: OperationId(operation.into()),
            turn_id: TurnId("logical-turn-1".into()),
            run_id: RunId(operation.into()),
            conversation_id: ConversationId("session-1".into()),
            state,
            sequence,
        }
    }

    #[test]
    fn lifecycle_projection_preserves_wire_sequence_and_operation_ownership() {
        for lifecycle in [
            TurnState::Accepted,
            TurnState::Queued,
            TurnState::Preparing,
            TurnState::Thinking,
            TurnState::ToolRunning,
            TurnState::Streaming,
            TurnState::WaitingUser,
            TurnState::Completed,
            TurnState::Failed,
            TurnState::Cancelled,
            TurnState::Recovering,
        ] {
            let mut state = GatewayState::with_epoch(ReplayLimits::default(), "epoch-test");
            state.register_turn("run-1", "session-1");
            state.emit(
                "session-1", "run-1", 1,
                GatewayEvent::MessageStart(MessageStartPayload::default()),
            );
            let before = state.replay_since("session-1", 0, Some("epoch-test"));

            assert!(state.ingest_runtime_event(&lifecycle_event("run-1", lifecycle, 42), 2).is_empty());
            assert_eq!(state.replay_since("session-1", 0, Some("epoch-test")), before);
            assert_eq!(state.projection.session_for_turn("run-1"), Some("session-1"));
            assert_eq!(state.projection.sessions_by_turn.len(), 1);
            assert!(state.projection.session_for_turn("logical-turn-1").is_none());

            // Lifecycle alone cannot admit an operation that prompt submission
            // or an existing semantic event has not registered.
            assert!(state.ingest_runtime_event(&lifecycle_event("unknown-run", lifecycle, 43), 3).is_empty());
            assert!(state.projection.session_for_turn("unknown-run").is_none());
            assert_eq!(state.replay_since("session-1", 0, Some("epoch-test")), before);
        }
    }

    #[test]
    fn lifecycle_projection_waits_for_real_completion_and_failure() {
        for (lifecycle, message_text) in [
            (TurnState::Completed, Some("authoritative final text")),
            (TurnState::Completed, None),
            (TurnState::Failed, None),
        ] {
            let mut state = GatewayState::with_epoch(ReplayLimits::default(), "epoch-test");
            state.register_turn("run-1", "session-1");
            let mut projected = Vec::new();
            if let Some(text) = message_text {
                projected.extend(state.ingest_runtime_event(&RuntimeEvent::MessageCompleted {
                    operation_id: OperationId("run-1".into()),
                    message: Message {
                        id: MessageId("message-1".into()),
                        conversation_id: ConversationId("session-1".into()),
                        role: MessageRole::Assistant,
                        text: text.into(),
                        created_at_ms: 1,
                        metadata: Value::Null,
                    },
                }, 1));
            }
            let before_terminal = state.replay_since("session-1", 0, Some("epoch-test"));
            assert!(state.ingest_runtime_event(&lifecycle_event("run-1", lifecycle, 9), 2).is_empty());
            assert_eq!(state.replay_since("session-1", 0, Some("epoch-test")), before_terminal);
            assert_eq!(state.projection.session_for_turn("run-1"), Some("session-1"));

            let terminal = if lifecycle == TurnState::Failed {
                RuntimeEvent::OperationFailed {
                    operation_id: OperationId("run-1".into()),
                    code: "provider_error".into(),
                    message: "actual provider refusal".into(),
                }
            } else {
                RuntimeEvent::OperationCompleted { operation_id: OperationId("run-1".into()) }
            };
            projected.extend(state.ingest_runtime_event(&terminal, 3));
            assert_eq!(projected.len(), 1, "lifecycle must not duplicate semantic completion");
            assert_eq!(projected[0].session_id, "session-1");
            assert_eq!(projected[0].turn_id, "run-1");
            assert_eq!(projected[0].seq, 1);
            let GatewayEvent::MessageComplete(payload) = &projected[0].event else {
                panic!("expected the existing semantic completion frame");
            };
            if lifecycle == TurnState::Failed {
                assert_eq!(payload.status, MessageCompletionStatus::Error);
                assert_eq!(payload.error.as_deref(), Some("provider_error: actual provider refusal"));
                assert_eq!(payload.partial, Some(true));
                assert_eq!(payload.recoverable, Some(true));
            } else {
                assert_eq!(payload.status, MessageCompletionStatus::Complete);
                assert_eq!(payload.text.as_deref(), message_text);
                assert!(payload.error.is_none());
            }
            assert!(state.projection.session_for_turn("run-1").is_none());
            let settled = state.replay_since("session-1", 0, Some("epoch-test"));
            assert!(state.ingest_runtime_event(&lifecycle_event("run-1", lifecycle, 10), 4).is_empty());
            assert!(state.ingest_runtime_event(&terminal, 5).is_empty());
            assert!(state.projection.session_for_turn("run-1").is_none());
            assert_eq!(state.replay_since("session-1", 0, Some("epoch-test")), settled);
        }
    }

    #[test]
    fn lifecycle_projection_cannot_resurrect_interrupted_run_or_rebind_retry() {
        let mut state = GatewayState::with_epoch(ReplayLimits::default(), "epoch-test");
        state.register_turn("run-old", "session-1");
        state.emit("session-1", "run-old", 1, GatewayEvent::MessageStart(MessageStartPayload::default()));
        let interrupted = state.interrupt_turn("run-old", 2);
        assert_eq!(interrupted.len(), 1);
        assert_eq!(interrupted[0].seq, 2);
        let GatewayEvent::MessageComplete(payload) = &interrupted[0].event else {
            panic!("interrupt must retain its existing completion frame");
        };
        assert_eq!(payload.status, MessageCompletionStatus::Interrupted);

        state.register_turn("run-retry", "session-1");
        state.emit("session-1", "run-retry", 3, GatewayEvent::MessageStart(MessageStartPayload::default()));
        for (sequence, lifecycle) in [TurnState::Cancelled, TurnState::Thinking, TurnState::Completed].into_iter().enumerate() {
            assert!(state.ingest_runtime_event(&lifecycle_event("run-old", lifecycle, sequence as u64 + 4), 4).is_empty());
        }
        assert!(state.ingest_runtime_event(&RuntimeEvent::OperationCompleted {
            operation_id: OperationId("run-old".into()),
        }, 5).is_empty());
        let approval = |operation: &str| RuntimeEvent::ApprovalRequested {
            operation_id: OperationId(operation.into()),
            approval_id: ApprovalId("approval-retry".into()),
            title: "Approve retry tool".into(),
            details: json!({"toolId": "tool-retry", "description": "confirmed request"}),
        };
        assert!(state.ingest_runtime_event(&approval("run-old"), 6).is_empty());
        assert!(state.ingest_runtime_event(&lifecycle_event("run-retry", TurnState::WaitingUser, 7), 7).is_empty());
        let projected = state.ingest_runtime_event(&approval("run-retry"), 8);
        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].session_id, "session-1");
        assert_eq!(projected[0].turn_id, "run-retry");
        assert_eq!(projected[0].seq, 4);
        let GatewayEvent::ApprovalRequest(payload) = &projected[0].event else {
            panic!("the new run must retain its approval frame");
        };
        assert_eq!(payload.request_id, "approval-retry");
        assert_eq!(payload.tool_id.as_deref(), Some("tool-retry"));
        assert_eq!(payload.description.as_deref(), Some("confirmed request"));
        assert!(state.projection.session_for_turn("run-old").is_none());
        assert_eq!(state.projection.session_for_turn("run-retry"), Some("session-1"));
        let replay = state.replay_since("session-1", 0, Some("epoch-test"));
        assert_eq!(replay.latest_seq, 4);
        assert_eq!(replay.events.iter().map(GatewayEventEnvelope::kind).collect::<Vec<_>>(),
            vec!["message.start", "message.complete", "message.start", "approval.request"]);
    }

    #[test]
    fn approval_respond_routes_all_legacy_choices_to_canonical_host_decisions() {
        for (input, expected) in [
            ("allow-once", "accept"),
            ("accept", "accept"),
            ("allow-session", "acceptForSession"),
            ("acceptForSession", "acceptForSession"),
            ("accept-for-session", "acceptForSession"),
            ("deny", "decline"),
            ("decline", "decline"),
            ("cancel", "cancel"),
        ] {
            let mut runtime = MockRuntime::default();
            let mut state = GatewayState::with_epoch(ReplayLimits::default(), "epoch-test");
            let response = dispatch_request(
                "approval.respond",
                &json!({"approvalId": "approval-1", "decision": input}),
                &mut runtime, &mut state, 123,
            ).expect("valid approval choice");
            assert_eq!(
                runtime.approval_calls,
                vec![("approval-1".to_string(), expected.to_string())],
                "approval choice {input} must dispatch exactly once with canonical parameters",
            );
            assert!(runtime.commands.is_empty());
            assert_eq!(response.result["approvalId"], "approval-1");
            assert_eq!(response.result["decision"], expected);
            let parsed: mahayana_core::ApprovalDecision =
                serde_json::from_value(json!(expected)).expect("Host accepts decision");
            assert_eq!(serde_json::to_value(parsed).unwrap(), json!(expected));
        }
    }

    #[test]
    fn approval_respond_unknown_choice_fails_before_host_dispatch() {
        for params in [
            json!({"approvalId": "approval-1", "decision": "invented"}),
            json!({"approvalId": "approval-1", "decision": "Accept"}),
            json!({"approvalId": "approval-1", "decision": ""}),
            json!({"approvalId": "approval-1", "decision": null}),
            json!({"approvalId": "approval-1", "decision": true}),
            json!({"approvalId": "approval-1", "decision": 1}),
            json!({"approvalId": "approval-1", "decision": ["accept"]}),
            json!({"approvalId": "approval-1", "decision": {"value": "accept"}}),
            json!({"approvalId": "approval-1"}),
        ] {
            let mut runtime = MockRuntime::default();
            let mut state = GatewayState::with_epoch(ReplayLimits::default(), "epoch-test");
            let failure = dispatch_request(
                "approval.respond", &params, &mut runtime, &mut state, 123,
            ).err().expect("invalid approval rejected");
            assert_eq!(failure.code, -32602, "invalid approval parameters: {params}");
            assert!(
                runtime.approval_calls.is_empty(),
                "invalid decision reached approval dispatch: {params}",
            );
            assert!(
                runtime.commands.is_empty(),
                "invalid decision reached ordinary dispatch: {params}",
            );
        }
    }

}
