use std::collections::HashMap;

use uuid::Uuid;

use super::prompt_acceptance_ledger::{
    AcceptanceRecord, PromptAcceptanceError, PromptAcceptanceLedger, SendAdmission, SendInput,
    send_input_digest,
};

pub const HOST_ACCOUNT_SLOT: &str = "host";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendBegin {
    EmptyNoop,
    Dispatch {
        client_nonce: Option<String>,
        input_digest: Option<String>,
    },
    Coalesced {
        client_nonce: String,
    },
    DuplicateNoop {
        record: AcceptanceRecord,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendEchoIdentity {
    pub agent_id: String,
    pub echo_entry_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InFlightSend {
    digest: String,
}

#[derive(Debug, Default)]
pub struct SendPipelineState {
    in_flight: HashMap<String, InFlightSend>,
    turn_epochs: HashMap<String, u64>,
    attachment_batch_ids: HashMap<String, String>,
}

impl SendPipelineState {
    pub fn begin_send(
        &mut self,
        ledger: &mut PromptAcceptanceLedger,
        input: &SendInput,
        client_nonce: Option<&str>,
    ) -> Result<SendBegin, PromptAcceptanceError> {
        let has_prompt = !input.prompt.trim().is_empty();
        if !has_prompt && input.attachment_paths.is_empty() {
            return Ok(SendBegin::EmptyNoop);
        }

        let nonce = client_nonce
            .filter(|nonce| !nonce.is_empty())
            .map(str::to_owned);
        let Some(nonce) = nonce else {
            return Ok(SendBegin::Dispatch {
                client_nonce: None,
                input_digest: None,
            });
        };

        if self.in_flight.contains_key(&nonce) {
            return Ok(SendBegin::Coalesced {
                client_nonce: nonce,
            });
        }

        let digest = send_input_digest(input);
        match ledger.admit_send(HOST_ACCOUNT_SLOT, &nonce, &digest)? {
            SendAdmission::Duplicate(record) => Ok(SendBegin::DuplicateNoop { record }),
            SendAdmission::Dispatch => {
                self.in_flight.insert(
                    nonce.clone(),
                    InFlightSend {
                        digest: digest.clone(),
                    },
                );
                Ok(SendBegin::Dispatch {
                    client_nonce: Some(nonce),
                    input_digest: Some(digest),
                })
            }
        }
    }

    pub fn record_pending_acceptance(
        &mut self,
        ledger: &mut PromptAcceptanceLedger,
        client_nonce: &str,
        echo: SendEchoIdentity,
    ) -> Result<AcceptanceRecord, PromptAcceptanceError> {
        let digest = self
            .in_flight
            .get(client_nonce)
            .map(|send| send.digest.clone())
            .ok_or_else(|| PromptAcceptanceError::InvalidPendingIdentity)?;
        ledger.record_pending(
            HOST_ACCOUNT_SLOT,
            client_nonce,
            digest,
            echo.agent_id,
            echo.echo_entry_id,
        )
    }

    pub fn mark_send_accepted(
        &mut self,
        ledger: &mut PromptAcceptanceLedger,
        client_nonce: Option<&str>,
    ) {
        if let Some(client_nonce) = client_nonce {
            ledger.mark_accepted(HOST_ACCOUNT_SLOT, client_nonce);
        }
    }

    pub fn mark_send_rejected(
        &mut self,
        ledger: &mut PromptAcceptanceLedger,
        client_nonce: Option<&str>,
        rejection_code: impl Into<String>,
    ) {
        if let Some(client_nonce) = client_nonce {
            ledger.mark_rejected(HOST_ACCOUNT_SLOT, client_nonce, rejection_code);
        }
    }

    pub fn finish_send(
        &mut self,
        ledger: &mut PromptAcceptanceLedger,
        client_nonce: Option<&str>,
        succeeded: bool,
    ) {
        let Some(client_nonce) = client_nonce else {
            return;
        };
        if !succeeded {
            ledger.clear_unless_accepted(HOST_ACCOUNT_SLOT, client_nonce);
        }
        self.in_flight.remove(client_nonce);
    }

    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    pub fn is_in_flight(&self, client_nonce: &str) -> bool {
        self.in_flight.contains_key(client_nonce)
    }

    pub fn next_turn_epoch(&mut self, agent_id: &str) -> u64 {
        let epoch = self.turn_epochs.entry(agent_id.to_string()).or_insert(0);
        *epoch = epoch.saturating_add(1);
        *epoch
    }

    pub fn current_turn_epoch(&self, agent_id: &str) -> u64 {
        self.turn_epochs.get(agent_id).copied().unwrap_or(0)
    }

    pub fn claim_attachment_batch_id(&mut self, agent_id: &str) -> String {
        self.attachment_batch_ids
            .entry(agent_id.to_string())
            .or_insert_with(|| Uuid::new_v4().to_string())
            .clone()
    }

    pub fn clear_attachment_batch_id(&mut self, agent_id: &str) {
        self.attachment_batch_ids.remove(agent_id);
    }
}
