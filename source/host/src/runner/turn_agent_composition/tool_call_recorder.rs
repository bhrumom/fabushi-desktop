use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, ThreadId};

use prost::Message;
use serde_json::Value;

use crate::extensions::inference::provider_session::{ProviderSessionError, RoutedToolDefinition};
use crate::runner::agent_v1_wire::{
    AgentToolCall, SendMessageArgs, SendMessageError, SendMessageResult, SendMessageToolCall,
    agent_tool_call, send_message_result,
};
use crate::runner::tools::send_message_tool::{
    SAND_SEND_MESSAGE_TOOL_NAME, SendMessageInteractionSink, provider_error_message,
};

use super::CompletedRoutedToolCall;

/// Only this recorder can qualify bytes for unchanged canonical publication.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecordedSendMessage {
    bytes: Vec<u8>,
}

impl RecordedSendMessage {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Phase {
    #[default]
    Unstarted,
    Recording,
    Sealed,
}

#[derive(Default)]
struct RecorderState {
    phase: Phase,
    next_sequence: u64,
    active: HashMap<ThreadId, ActiveCall>,
    send_message_ids: HashSet<String>,
    completed: Vec<CompletedRoutedToolCall>,
    integrity_fault: Option<&'static str>,
}

impl RecorderState {
    fn refuse(&mut self, reason: &'static str) -> ProviderSessionError {
        recorder_error(*self.integrity_fault.get_or_insert(reason))
    }

    fn require_recording(&mut self) -> Result<(), ProviderSessionError> {
        if let Some(reason) = self.integrity_fault {
            return Err(recorder_error(reason));
        }
        if self.phase != Phase::Recording {
            return Err(self.refuse("invocation is not recording"));
        }
        Ok(())
    }
}

struct ActiveCall {
    tool: RoutedToolDefinition,
    args: Value,
    tool_call_id: String,
    started_at_ms: u64,
    sequence: u64,
    is_send_message: bool,
    observation: SendObservation,
}

#[derive(Default)]
enum SendObservation {
    #[default]
    Absent,
    Initial {
        args: SendMessageArgs,
        started_at_ms: u64,
    },
    Completed {
        result: send_message_result::Result,
        wire: RecordedSendMessage,
    },
}

impl ActiveCall {
    fn observe(&mut self, phase: &str, call: &AgentToolCall) -> Result<(), &'static str> {
        if !self.is_send_message {
            return Err("SendMessage callback belongs to another tool");
        }
        if call.tool_call_id.as_deref() != Some(self.tool_call_id.as_str()) {
            return Err("SendMessage callback identity differs from its receipt");
        }
        let Some(agent_tool_call::Tool::SendMessageToolCall(send)) = call.tool.as_ref() else {
            return Err("SendMessage callback has another frozen tool variant");
        };
        let args = send.args.as_ref().ok_or("SendMessage callback has no arguments")?;
        if args.message.is_none() {
            return Err("SendMessage callback has no supported argument variant");
        }
        let started_at_ms = call.started_at_ms.ok_or("SendMessage callback has no start time")?;
        match (phase, &self.observation) {
            ("initial", SendObservation::Absent) => {
                if send.result.is_some() || call.completed_at_ms.is_some() {
                    return Err("initial SendMessage callback is already terminal");
                }
                self.observation = SendObservation::Initial {
                    args: args.clone(),
                    started_at_ms,
                };
                Ok(())
            }
            ("completed", SendObservation::Initial { args: initial_args, started_at_ms: initial_start }) => {
                if args != initial_args || started_at_ms != *initial_start {
                    return Err("completed SendMessage callback changed its initial arguments or start");
                }
                if call.completed_at_ms.is_none() {
                    return Err("completed SendMessage callback has no completion time");
                }
                let result = send.result.as_ref().and_then(|result| result.result.as_ref())
                    .ok_or("completed SendMessage callback has no terminal result")?;
                self.observation = SendObservation::Completed {
                    result: result.clone(),
                    wire: RecordedSendMessage { bytes: call.encode_to_vec() },
                };
                Ok(())
            }
            _ => Err("SendMessage callback phase is duplicate or out of order"),
        }
    }

    fn qualified_send_message(
        &self,
        result: &Result<Value, ProviderSessionError>,
        completed_at_ms: u64,
    ) -> Result<RecordedSendMessage, &'static str> {
        match (&self.observation, result) {
            (SendObservation::Completed { result: send_message_result::Result::Success(success), wire }, Ok(value)) => {
                let id_matches = match value.get("messageId") {
                    Some(Value::String(message_id)) => message_id == &success.message_id,
                    Some(Value::Null) => success.message_id.is_empty(),
                    _ => false,
                };
                if value.get("sent").and_then(Value::as_bool) != Some(true)
                    || value.get("timestampMs").and_then(Value::as_u64) != Some(success.timestamp)
                    || !id_matches
                {
                    return Err("SendMessage success differs from the delegated public result");
                }
                Ok(wire.clone())
            }
            (SendObservation::Completed { result: send_message_result::Result::Error(error), wire }, Err(public_error)) => {
                if error.error != provider_error_message(public_error) {
                    return Err("SendMessage error differs from the delegated public error");
                }
                Ok(wire.clone())
            }
            (SendObservation::Absent, Err(ProviderSessionError::ToolOutcomeUnknown(_))) => {
                Err("unobserved SendMessage has unknown effect custody")
            }
            (SendObservation::Absent, Err(error)) => {
                // Argument parsing, permission, or normalization may refuse
                // before the producer emits anything. Preserve that rejection
                // without guessing normalized arguments or a message identity.
                let rejected = AgentToolCall {
                    tool: Some(agent_tool_call::Tool::SendMessageToolCall(SendMessageToolCall {
                        args: Some(SendMessageArgs { message: None }),
                        result: Some(SendMessageResult {
                            result: Some(send_message_result::Result::Error(SendMessageError {
                                error: provider_error_message(error),
                            })),
                        }),
                    })),
                    tool_call_id: Some(self.tool_call_id.clone()),
                    started_at_ms: Some(self.started_at_ms),
                    completed_at_ms: Some(completed_at_ms),
                };
                Ok(RecordedSendMessage { bytes: rejected.encode_to_vec() })
            }
            (SendObservation::Absent, Ok(_)) => Err("successful SendMessage has no producer observations"),
            (SendObservation::Initial { .. }, _) => Err("SendMessage has only an initial observation"),
            _ => Err("SendMessage terminal disposition differs from the delegated result"),
        }
    }
}

fn recorder_error(reason: &str) -> ProviderSessionError {
    ProviderSessionError::Protocol(format!("Runner tool-call recorder refused publication: {reason}"))
}

#[derive(Default)]
pub(super) struct ToolCallRecorder {
    state: Mutex<RecorderState>,
}

/// Not cloneable or constructible outside this module. The recorder allocation
/// and sequence authorize the call; the thread only locates its synchronous slot.
pub(super) struct CallReceipt {
    recorder: Arc<ToolCallRecorder>,
    sequence: u64,
    thread_id: ThreadId,
    armed: bool,
}

impl fmt::Debug for CallReceipt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("CallReceipt")
            .field("sequence", &self.sequence)
            .field("thread_id", &self.thread_id)
            .finish_non_exhaustive()
    }
}

impl Drop for CallReceipt {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        // A poisoned recorder is already permanently ineligible. Never recover
        // its mutex to manufacture completed observations or a usable drain.
        if let Ok(mut state) = self.recorder.state.lock() {
            if state.active.get(&self.thread_id).is_some_and(|call| call.sequence == self.sequence) {
                state.active.remove(&self.thread_id);
            }
            state.integrity_fault.get_or_insert("delegated call receipt was abandoned");
        }
    }
}

impl ToolCallRecorder {
    fn lock(&self) -> Result<MutexGuard<'_, RecorderState>, ProviderSessionError> {
        self.state.lock().map_err(|_| recorder_error("recorder mutex is poisoned"))
    }

    pub(super) fn begin_run(&self) -> Result<(), ProviderSessionError> {
        let mut state = self.lock()?;
        if state.phase != Phase::Unstarted || state.integrity_fault.is_some() {
            return Err(state.refuse("invocation recorder was already used"));
        }
        state.phase = Phase::Recording;
        Ok(())
    }

    pub(super) fn register(
        self: &Arc<Self>,
        tool: &RoutedToolDefinition,
        args: Value,
        id: &str,
        started_at_ms: u64,
    ) -> Result<CallReceipt, ProviderSessionError> {
        let mut state = self.lock()?;
        state.require_recording()?;
        let thread_id = thread::current().id();
        if state.active.contains_key(&thread_id) {
            return Err(state.refuse("nested synchronous tool registration"));
        }
        let is_send_message = tool.name == SAND_SEND_MESSAGE_TOOL_NAME
            || tool.tool_name == SAND_SEND_MESSAGE_TOOL_NAME;
        if is_send_message && (tool.name != SAND_SEND_MESSAGE_TOOL_NAME
            || tool.tool_name != SAND_SEND_MESSAGE_TOOL_NAME
            || tool.provider_identifier != "fabushi-runner")
        {
            return Err(state.refuse("SendMessage definition is not the exact first-party tool"));
        }
        if is_send_message && !state.send_message_ids.insert(id.to_string()) {
            return Err(state.refuse("SendMessage provider call identity was reused"));
        }
        let sequence = state.next_sequence;
        let Some(next_sequence) = sequence.checked_add(1) else {
            return Err(state.refuse("tool sequence is exhausted"));
        };
        state.next_sequence = next_sequence;
        state.active.insert(thread_id, ActiveCall {
            tool: tool.clone(),
            args,
            tool_call_id: id.to_string(),
            started_at_ms,
            sequence,
            is_send_message,
            observation: SendObservation::Absent,
        });
        Ok(CallReceipt { recorder: Arc::clone(self), sequence, thread_id, armed: true })
    }

    pub(super) fn finish(
        &self,
        mut receipt: CallReceipt,
        result: &Result<Value, ProviderSessionError>,
        completed_at_ms: u64,
    ) -> Result<(), ProviderSessionError> {
        if !std::ptr::eq(self, Arc::as_ptr(&receipt.recorder)) {
            return Err(self.lock()?.refuse("receipt belongs to another invocation recorder"));
        }
        let mut state = self.lock()?;
        state.require_recording()?;
        if receipt.thread_id != thread::current().id() {
            return Err(state.refuse("receipt finished on another worker thread"));
        }
        if !state.active.get(&receipt.thread_id).is_some_and(|call| call.sequence == receipt.sequence) {
            return Err(state.refuse("receipt has no matching active call"));
        }
        let active = state.active.remove(&receipt.thread_id).expect("checked active receipt");
        receipt.armed = false;
        let send_message_wire = if active.is_send_message {
            match active.qualified_send_message(result, completed_at_ms) {
                Ok(wire) => Some(wire),
                Err(reason) => return Err(state.refuse(reason)),
            }
        } else {
            None
        };
        state.completed.push(CompletedRoutedToolCall {
            tool: active.tool,
            args: active.args,
            tool_call_id: active.tool_call_id,
            result: result.as_ref().map(Clone::clone).map_err(ToString::to_string),
            started_at_ms: active.started_at_ms,
            completed_at_ms,
            sequence: active.sequence,
            send_message_wire,
        });
        Ok(())
    }

    pub(super) fn take_completed_checked(&self) -> Result<Vec<CompletedRoutedToolCall>, ProviderSessionError> {
        let mut state = self.lock()?;
        let previous_phase = state.phase;
        state.phase = Phase::Sealed;
        if previous_phase != Phase::Recording {
            return Err(state.refuse("invocation recorder cannot be drained twice or before begin"));
        }
        if let Some(reason) = state.integrity_fault {
            return Err(recorder_error(reason));
        }
        if !state.active.is_empty() {
            return Err(state.refuse("invocation still has active delegated calls"));
        }
        if u64::try_from(state.completed.len()).ok() != Some(state.next_sequence) {
            return Err(state.refuse("invocation has unfinished tool sequences"));
        }
        let mut completed = std::mem::take(&mut state.completed);
        completed.sort_by_key(|call| call.sequence);
        Ok(completed)
    }

    fn capture(&self, phase: &str, call: &AgentToolCall) -> Result<(), ProviderSessionError> {
        let mut state = self.lock()?;
        state.require_recording()?;
        let Some(active) = state.active.get_mut(&thread::current().id()) else {
            return Err(state.refuse("SendMessage callback has no active call on this worker"));
        };
        if let Err(reason) = active.observe(phase, call) {
            return Err(state.refuse(reason));
        }
        Ok(())
    }

    pub(super) fn interaction_sink(
        self: &Arc<Self>,
        live: Option<Arc<dyn SendMessageInteractionSink>>,
    ) -> Arc<dyn SendMessageInteractionSink> {
        Arc::new(RecordingInteractionSink { recorder: Arc::clone(self), live })
    }
}

struct RecordingInteractionSink {
    recorder: Arc<ToolCallRecorder>,
    live: Option<Arc<dyn SendMessageInteractionSink>>,
}

impl SendMessageInteractionSink for RecordingInteractionSink {
    fn on_tool_call(&self, phase: &'static str, call: &AgentToolCall) {
        let _ = self.recorder.capture(phase, call);
        // Capture faults fence publication, not the actual producer outcome or
        // live observation. No recorder lock crosses this user-supplied callback.
        if let Some(live) = self.live.as_ref() {
            live.on_tool_call(phase, call);
        }
    }
}

#[cfg(test)]
#[path = "tool_call_recorder_tests.rs"]
mod tests;
