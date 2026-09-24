use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};

use super::clock_skew_guard::{
    SEND_DISPATCH_MAX_PLAUSIBLE_MS, TTFT_MAX_PLAUSIBLE_MS,
    SanitizedCrossClockDuration, bucket_clock_skew_delta_ms,
    sanitize_cross_clock_duration_ms,
};
use super::conversation_outline::MCP_TOOL_CALL_OUTLINE_NAME;
use super::routed_provider_runtime::RoutedToolBridge;

pub const MCP_EXEC_STALL_THRESHOLD_MS: u64 = 15 * 60 * 1_000;
pub const RECENT_ACTIVITY_CAP: usize = 24;

pub type TurnObservationEventSink =
    Arc<dyn Fn(Value) + Send + Sync + 'static>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolActivity {
    pub status: String,
    pub name: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnObservationSnapshot {
    pub elapsed_ms: u64,
    pub last_tool: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingAwait {
    index: u64,
    block_until_ms: u64,
}

pub struct TurnObservation {
    conversation_id: String,
    event_sink: Option<TurnObservationEventSink>,
    turn_started_at_ms: u64,
    last_tool: Option<String>,
    recent_activity: Vec<String>,
    observed_tool_call_count: u64,
    await_observation_count: u64,
    pending_awaits: HashMap<String, PendingAwait>,
    first_token_observed: bool,
}

pub type TurnObservationHandle = Arc<Mutex<TurnObservation>>;

impl TurnObservation {
    pub fn new(
        conversation_id: impl Into<String>,
        event_sink: Option<TurnObservationEventSink>,
    ) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            event_sink,
            turn_started_at_ms: now_ms(),
            last_tool: None,
            recent_activity: Vec::new(),
            observed_tool_call_count: 0,
            await_observation_count: 0,
            pending_awaits: HashMap::new(),
            first_token_observed: false,
        }
    }

    pub fn shared(
        conversation_id: impl Into<String>,
        event_sink: Option<TurnObservationEventSink>,
    ) -> TurnObservationHandle {
        Arc::new(Mutex::new(Self::new(conversation_id, event_sink)))
    }

    pub fn turn_started(&mut self, at_ms: u64) {
        self.turn_started_at_ms = at_ms;
        self.first_token_observed = false;
        self.emit(json!({
            "type": "turn-started",
            "agentId": self.conversation_id,
            "at": at_ms,
        }));
    }

    pub fn tool_started(&mut self, name: &str) {
        self.last_tool = Some(name.to_string());
        self.record_tool_activity(ToolActivity {
            status: "pending".into(),
            name: name.to_string(),
            summary: None,
        });
        self.emit(json!({
            "type": "tool-started",
            "agentId": self.conversation_id,
            "name": name,
            "at": now_ms(),
        }));
    }

    pub fn tool_completed(
        &mut self,
        name: &str,
        failed: bool,
        summary: Option<&str>,
    ) {
        if self.last_tool.as_deref() == Some(name) {
            self.last_tool = None;
        }
        self.record_tool_activity(ToolActivity {
            status: if failed { "failed" } else { "done" }.into(),
            name: name.to_string(),
            summary: summary
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
        });
        self.emit(json!({
            "type": "tool-completed",
            "agentId": self.conversation_id,
            "name": name,
            "failed": failed,
            "at": now_ms(),
        }));
    }

    pub fn record_tool_activity(&mut self, update: ToolActivity) {
        if matches!(update.status.as_str(), "done" | "failed") {
            self.observed_tool_call_count =
                self.observed_tool_call_count.saturating_add(1);
        }
        let summary = update
            .summary
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(|value| format!(": {value}"))
            .unwrap_or_default();
        let line = format!("[{}] {}{summary}", update.status, update.name);
        if self.recent_activity.last() == Some(&line) {
            return;
        }
        self.recent_activity.push(line);
        if self.recent_activity.len() > RECENT_ACTIVITY_CAP {
            let drain = self.recent_activity.len() - RECENT_ACTIVITY_CAP;
            self.recent_activity.drain(0..drain);
        }
    }

    pub fn observe_first_token(
        &mut self,
        chunk_type: &str,
        dispatch_perf_ms: Option<f64>,
        observed_perf_ms: Option<f64>,
        model_id: Option<&str>,
        is_fork: bool,
    ) -> bool {
        if self.first_token_observed {
            return false;
        }
        self.first_token_observed = true;
        let timing = match (dispatch_perf_ms, observed_perf_ms) {
            (Some(dispatch), Some(observed)) => {
                let raw = observed - dispatch;
                let sanitized = sanitize_cross_clock_duration_ms(
                    raw,
                    TTFT_MAX_PLAUSIBLE_MS as f64,
                );
                match sanitized {
                    SanitizedCrossClockDuration::Millis(ms) => json!({
                        "ttftMs": ms,
                        "skew": false,
                    }),
                    SanitizedCrossClockDuration::Skew(reason) => json!({
                        "skew": true,
                        "skewReason": format!("{reason:?}"),
                        "skewBucket": bucket_clock_skew_delta_ms(raw),
                    }),
                }
            }
            _ => json!({}),
        };
        let mut event = json!({
            "type": "first-token",
            "agentId": self.conversation_id,
            "chunkType": chunk_type,
            "isFork": is_fork,
        });
        if let Some(model_id) = model_id.filter(|value| !value.is_empty()) {
            event["modelId"] = Value::String(model_id.to_string());
        }
        if let (Some(target), Some(source)) =
            (event.as_object_mut(), timing.as_object())
        {
            for (key, value) in source {
                target.insert(key.clone(), value.clone());
            }
        }
        self.emit(event);
        true
    }

    pub fn observe_send_dispatch(
        &self,
        host_receipt_perf_ms: f64,
        dispatch_perf_ms: f64,
        enter_epoch_ms: Option<f64>,
        dispatch_epoch_ms: f64,
        is_fork: bool,
        model_id: &str,
    ) {
        let host_dispatch_ms = (dispatch_perf_ms - host_receipt_perf_ms)
            .max(0.0)
            .round() as u64;
        let mut event = json!({
            "type": "send-dispatch",
            "agentId": self.conversation_id,
            "hostDispatchMs": host_dispatch_ms,
            "isFork": is_fork,
            "modelId": model_id,
        });
        if let Some(enter) = enter_epoch_ms.filter(|value| value.is_finite()) {
            let raw = dispatch_epoch_ms - enter;
            match sanitize_cross_clock_duration_ms(
                raw,
                SEND_DISPATCH_MAX_PLAUSIBLE_MS as f64,
            ) {
                SanitizedCrossClockDuration::Millis(ms) => {
                    event["dispatchMs"] = json!(ms);
                    event["skew"] = Value::Bool(false);
                }
                SanitizedCrossClockDuration::Skew(reason) => {
                    event["skew"] = Value::Bool(true);
                    event["skewReason"] =
                        Value::String(format!("{reason:?}"));
                    event["skewBucket"] =
                        Value::String(bucket_clock_skew_delta_ms(raw).into());
                }
            }
        }
        self.emit(event);
    }

    pub fn report_turn_retry(&self, event: Value) {
        self.emit(json!({
            "type": "turn-retry",
            "agentId": self.conversation_id,
            "event": event,
        }));
    }

    pub fn observe_await_tool_call(
        &mut self,
        started: bool,
        call_id: &str,
        block_until_ms: u64,
        result: Option<&str>,
        cancelled: bool,
        interrupted: bool,
    ) {
        if started {
            self.await_observation_count =
                self.await_observation_count.saturating_add(1);
            self.pending_awaits.insert(
                call_id.to_string(),
                PendingAwait {
                    index: self.await_observation_count,
                    block_until_ms,
                },
            );
            return;
        }

        let pending = self.pending_awaits.remove(call_id);
        let index = pending
            .as_ref()
            .map(|value| value.index)
            .unwrap_or_else(|| {
                self.await_observation_count =
                    self.await_observation_count.saturating_add(1);
                self.await_observation_count
            });
        let block_until_ms = pending
            .map(|value| value.block_until_ms)
            .unwrap_or(block_until_ms);
        let outcome = if cancelled {
            if interrupted { "aborted" } else { "clean_stop" }
        } else if result == Some("timeout") {
            "timeout"
        } else {
            "completed"
        };
        self.emit(json!({
            "type": "turn-await",
            "agentId": self.conversation_id,
            "awaitIndex": index,
            "blockUntilMs": block_until_ms,
            "outcome": outcome,
        }));
    }

    pub fn flush_pending_awaits_on_unwind(
        &mut self,
        interrupted: bool,
    ) {
        let outcome = if interrupted { "aborted" } else { "clean_stop" };
        let pending = self.pending_awaits.drain().collect::<Vec<_>>();
        for (_, item) in pending {
            self.emit(json!({
                "type": "turn-await",
                "agentId": self.conversation_id,
                "awaitIndex": item.index,
                "blockUntilMs": item.block_until_ms,
                "outcome": outcome,
            }));
        }
        self.await_observation_count = 0;
    }

    pub fn snapshot(&self, at_ms: u64) -> TurnObservationSnapshot {
        TurnObservationSnapshot {
            elapsed_ms: at_ms.saturating_sub(self.turn_started_at_ms),
            last_tool: self.last_tool.clone(),
        }
    }

    pub fn recent_activity(&self) -> Vec<String> {
        self.recent_activity.clone()
    }

    pub fn observed_tool_call_count(&self) -> u64 {
        self.observed_tool_call_count
    }

    fn emit(&self, event: Value) {
        if let Some(sink) = self.event_sink.as_ref() {
            sink(event);
        }
    }
}

pub struct ObservedRoutedToolBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    observation: TurnObservationHandle,
}

impl ObservedRoutedToolBridge {
    pub fn new(
        delegate: Arc<dyn RoutedToolBridge>,
        observation: TurnObservationHandle,
    ) -> Self {
        Self {
            delegate,
            observation,
        }
    }
}

impl RoutedToolBridge for ObservedRoutedToolBridge {
    fn list_tools(
        &self,
    ) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        self.delegate.list_tools()
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        let name = if tool.tool_name.trim().is_empty() {
            tool.name.as_str()
        } else {
            tool.tool_name.as_str()
        };
        if let Ok(mut observation) = self.observation.lock() {
            observation.tool_started(name);
            if name == "AwaitShell" || name == "awaitToolCall" {
                let block_until_ms = args
                    .get("block_until_ms")
                    .or_else(|| args.get("blockUntilMs"))
                    .and_then(Value::as_u64)
                    .unwrap_or_default();
                observation.observe_await_tool_call(
                    true,
                    tool_call_id,
                    block_until_ms,
                    None,
                    false,
                    false,
                );
            }
        }

        let result = self.delegate.call_tool(tool, args.clone(), tool_call_id);
        if let Ok(mut observation) = self.observation.lock() {
            let failed = result.is_err();
            let summary = result.as_ref().ok().map(Value::to_string);
            observation.tool_completed(name, failed, summary.as_deref());
            if name == "AwaitShell" || name == "awaitToolCall" {
                let block_until_ms = args
                    .get("block_until_ms")
                    .or_else(|| args.get("blockUntilMs"))
                    .and_then(Value::as_u64)
                    .unwrap_or_default();
                let outcome = result
                    .as_ref()
                    .ok()
                    .and_then(|value| value.get("status").and_then(Value::as_str));
                observation.observe_await_tool_call(
                    false,
                    tool_call_id,
                    block_until_ms,
                    outcome,
                    false,
                    false,
                );
            }
        }
        result
    }
}

pub fn mcp_tool_call_outline_name() -> &'static str {
    MCP_TOOL_CALL_OUTLINE_NAME
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default()
}
