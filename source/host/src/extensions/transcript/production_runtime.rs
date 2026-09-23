use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::{Condvar, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use super::prompt_acceptance_ledger::{
    AcceptanceLookup, AcceptanceRecord, AcceptanceStatus, PromptAcceptanceError,
    PromptAcceptanceLedger, SendInput,
};
use super::run_lifecycle::RunLifecycleState;
use super::send_pipeline::{
    HOST_ACCOUNT_SLOT, SendBegin, SendEchoIdentity, SendPipelineState,
};

const COMPLETION_CACHE_MAX: usize = 256;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ProductionSendError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Rejected(String),
    #[error("{0}")]
    Internal(String),
}

struct RuntimeState {
    pipeline: SendPipelineState,
    ledger: PromptAcceptanceLedger,
    lifecycle: RunLifecycleState,
    completions: HashMap<String, Result<Value, ProductionSendError>>,
    completion_order: VecDeque<String>,
}

pub struct ProductionTranscriptRuntime {
    state: Mutex<RuntimeState>,
    send_settled: Condvar,
}

impl ProductionTranscriptRuntime {
    pub fn new(root_dir: Option<&Path>) -> Self {
        Self {
            state: Mutex::new(RuntimeState {
                pipeline: SendPipelineState::default(),
                ledger: PromptAcceptanceLedger::new(root_dir),
                lifecycle: RunLifecycleState::default(),
                completions: HashMap::new(),
                completion_order: VecDeque::new(),
            }),
            send_settled: Condvar::new(),
        }
    }

    pub fn prompt_acceptance_status(
        &self,
        args: &Value,
    ) -> Result<Value, ProductionSendError> {
        let client_nonce = args
            .get("clientNonce")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ProductionSendError::BadRequest(
                    "promptAcceptanceStatus requires clientNonce".into(),
                )
            })?;
        let account_slot = args
            .get("accountSlot")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(HOST_ACCOUNT_SLOT);
        let mut state = self.lock_state();
        Ok(match state.ledger.lookup(account_slot, client_nonce) {
            AcceptanceLookup::Found(record) => json!({
                "outcome": "found",
                "record": record,
            }),
            AcceptanceLookup::UnknownDurability => json!({
                "outcome": "unknown-durability",
            }),
            AcceptanceLookup::NotFound => json!({
                "outcome": "not-found",
            }),
        })
    }

    pub fn execute_send<Dispatch, Persist>(
        &self,
        args: &Value,
        dispatch: Dispatch,
        persist_accepted: Persist,
    ) -> Result<Value, ProductionSendError>
    where
        Dispatch: FnOnce() -> Result<Value, ProductionSendError>,
        Persist: Fn(&Value) -> Result<(), ProductionSendError>,
    {
        let input = parse_send_input(args)?;
        let nonce = optional_non_empty(args, "clientNonce").map(ToOwned::to_owned);
        let agent_id = input.agent_id.clone();

        let mut state = self.lock_state();
        loop {
            let begin = {
                let RuntimeState {
                    pipeline, ledger, ..
                } = &mut *state;
                pipeline
                    .begin_send(ledger, &input, nonce.as_deref())
                    .map_err(map_acceptance_error)?
            };
            match begin {
                SendBegin::EmptyNoop => return Ok(Value::Null),
                SendBegin::Coalesced { client_nonce } => {
                    state = self
                        .send_settled
                        .wait_while(state, |state| {
                            state.pipeline.is_in_flight(&client_nonce)
                        })
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if let Some(result) = state.completions.get(&client_nonce) {
                        return result.clone();
                    }
                }
                SendBegin::DuplicateNoop { record } => {
                    if let Some(result) = state.completions.get(&record.client_nonce) {
                        return result.clone();
                    }
                    return Ok(replay_record(&record));
                }
                SendBegin::Dispatch { .. } => {
                    if let Some(agent_id) = agent_id.as_deref() {
                        state
                            .lifecycle
                            .begin_session_run(agent_id, system_now_ms(), false);
                        state.pipeline.next_turn_epoch(agent_id);
                    }
                    break;
                }
            }
        }
        drop(state);

        let mut result = dispatch();
        if let Ok(value) = result.as_ref() {
            if value.get("accepted").and_then(Value::as_bool) == Some(true) {
                if let Err(error) = persist_accepted(value) {
                    result = Err(error);
                }
            }
        }

        let mut state = self.lock_state();
        let accepted = result
            .as_ref()
            .ok()
            .is_some_and(|value| value.get("accepted").and_then(Value::as_bool) == Some(true));

        if accepted {
            if let (Some(client_nonce), Some(value)) = (nonce.as_deref(), result.as_ref().ok()) {
                let operation_id = value
                    .get("operationId")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                let echo_entry_id = operation_id.map(|value| format!("{value}:user"));
                let pending = {
                    let RuntimeState {
                        pipeline, ledger, ..
                    } = &mut *state;
                    pipeline.record_pending_acceptance(
                        ledger,
                        client_nonce,
                        SendEchoIdentity {
                            agent_id: agent_id.clone().unwrap_or_default(),
                            echo_entry_id,
                        },
                    )
                };
                if let Err(error) = pending {
                    result = Err(map_acceptance_error(error));
                } else {
                    let RuntimeState {
                        pipeline, ledger, ..
                    } = &mut *state;
                    pipeline.mark_send_accepted(ledger, Some(client_nonce));
                }
                if let (Some(agent_id), Some(operation_id)) =
                    (agent_id.as_deref(), operation_id)
                {
                    state.lifecycle.record_request_id(agent_id, operation_id);
                }
            }
        }

        let succeeded = result
            .as_ref()
            .ok()
            .is_some_and(|value| value.get("accepted").and_then(Value::as_bool) == Some(true));
        {
            let RuntimeState {
                pipeline, ledger, ..
            } = &mut *state;
            pipeline.finish_send(ledger, nonce.as_deref(), succeeded);
        }
        if let Some(agent_id) = agent_id.as_deref() {
            let _ = state.lifecycle.end_session_run(agent_id, system_now_ms());
        }
        if let Some(client_nonce) = nonce.as_deref() {
            cache_completion(&mut state, client_nonce, result.clone());
        }
        drop(state);
        self.send_settled.notify_all();
        result
    }

    pub fn current_turn_epoch(&self, agent_id: &str) -> u64 {
        self.lock_state().pipeline.current_turn_epoch(agent_id)
    }

    pub fn in_flight_run_count(&self, agent_id: &str) -> u64 {
        self.lock_state().lifecycle.in_flight_count(agent_id)
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, RuntimeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn cache_completion(
    state: &mut RuntimeState,
    client_nonce: &str,
    result: Result<Value, ProductionSendError>,
) {
    if !state.completions.contains_key(client_nonce) {
        state.completion_order.push_back(client_nonce.to_string());
    }
    state.completions.insert(client_nonce.to_string(), result);
    while state.completion_order.len() > COMPLETION_CACHE_MAX {
        if let Some(oldest) = state.completion_order.pop_front() {
            state.completions.remove(&oldest);
        }
    }
}

fn replay_record(record: &AcceptanceRecord) -> Value {
    let operation_id = record
        .echo_entry_id
        .as_deref()
        .and_then(|entry_id| entry_id.strip_suffix(":user"));
    json!({
        "accepted": record.status == AcceptanceStatus::Accepted,
        "duplicate": true,
        "acceptanceStatus": match record.status {
            AcceptanceStatus::Accepted => "accepted",
            AcceptanceStatus::Rejected => "rejected",
            AcceptanceStatus::Pending => "pending",
        },
        "agentId": record.agent_id,
        "clientNonce": record.client_nonce,
        "operationId": operation_id,
    })
}

fn parse_send_input(args: &Value) -> Result<SendInput, ProductionSendError> {
    let agent_id = optional_non_empty(args, "agentId")
        .or_else(|| optional_non_empty(args, "id"))
        .map(ToOwned::to_owned);
    let prompt = args
        .get("prompt")
        .or_else(|| args.get("text"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let attachment_paths = string_array(args, "attachmentPaths")?;
    let attachment_names = string_array(args, "attachmentNames")?;
    if prompt.trim().is_empty() && attachment_paths.is_empty() {
        return Ok(SendInput {
            agent_id,
            prompt,
            rich_text: optional_string(args, "richText")?,
            reply_to_id: optional_string(args, "replyToId")?,
            is_fork: optional_bool(args, "isFork")?.unwrap_or(false),
            attachment_paths,
            attachment_names,
        });
    }
    Ok(SendInput {
        agent_id,
        prompt,
        rich_text: optional_string(args, "richText")?,
        reply_to_id: optional_string(args, "replyToId")?,
        is_fork: optional_bool(args, "isFork")?.unwrap_or(false),
        attachment_paths,
        attachment_names,
    })
}

fn string_array(args: &Value, field: &str) -> Result<Vec<String>, ProductionSendError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| {
                        ProductionSendError::BadRequest(format!(
                            "{field} must contain only strings"
                        ))
                    })
            })
            .collect(),
        Some(_) => Err(ProductionSendError::BadRequest(format!(
            "{field} must be an array"
        ))),
    }
}

fn optional_string(args: &Value, field: &str) -> Result<Option<String>, ProductionSendError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ProductionSendError::BadRequest(format!("invalid {field}"))),
    }
}

fn optional_bool(args: &Value, field: &str) -> Result<Option<bool>, ProductionSendError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(ProductionSendError::BadRequest(format!("invalid {field}"))),
    }
}

fn optional_non_empty<'a>(args: &'a Value, field: &str) -> Option<&'a str> {
    args.get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn map_acceptance_error(error: PromptAcceptanceError) -> ProductionSendError {
    match error {
        PromptAcceptanceError::DigestMismatch { .. } => {
            ProductionSendError::Conflict(error.to_string())
        }
        PromptAcceptanceError::Rejected { .. } => {
            ProductionSendError::Rejected(error.to_string())
        }
        PromptAcceptanceError::InvalidPendingIdentity => {
            ProductionSendError::Internal(error.to_string())
        }
    }
}

fn system_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
