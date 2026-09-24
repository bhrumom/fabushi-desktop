use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError,
};
use crate::extensions::session::production_agent_store::{
    ProductionAgentStore, ProductionWorkerBlobStore,
};
use crate::transcript_mirror::production_provider::{
    ProductionRoutedTranscriptMirror, ProductionTranscriptCheckpoint,
};

use super::{
    TurnRunOptions, persist_checkpoint_with_mirror,
};

pub trait AgentStateCheckpointSink: Send + Sync {
    fn checkpoint_text_turn(
        &self,
        messages: &[ProviderMessage],
        options: &TurnRunOptions,
        assistant_content: &str,
    ) -> Result<(), ProviderSessionError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextTurnCheckpointArtifacts {
    pub user_message_id: Vec<u8>,
    pub user_message_bytes: Vec<u8>,
    pub step_ids: Vec<Vec<u8>>,
    pub step_bytes: Vec<Vec<u8>>,
    pub turn_id: Vec<u8>,
    pub turn_bytes: Vec<u8>,
    pub state_bytes: Vec<u8>,
}

pub fn build_text_turn_checkpoint(
    prior_state_bytes: &[u8],
    user_text: &str,
    message_id: &str,
    request_id: Option<&str>,
    assistant_content: &str,
) -> TextTurnCheckpointArtifacts {
    let prior_root_id = (!prior_state_bytes.is_empty())
        .then(|| Sha256::digest(prior_state_bytes).to_vec());

    let mut user_message_bytes = Vec::new();
    push_length_delimited(1, user_text.as_bytes(), &mut user_message_bytes);
    push_length_delimited(2, message_id.as_bytes(), &mut user_message_bytes);
    if let Some(prior_root_id) = prior_root_id.as_deref() {
        push_length_delimited(10, prior_root_id, &mut user_message_bytes);
    }
    let user_message_id = Sha256::digest(&user_message_bytes).to_vec();

    let mut step_ids = Vec::new();
    let mut step_bytes = Vec::new();
    if !assistant_content.is_empty() {
        let mut assistant_message = Vec::new();
        push_length_delimited(1, assistant_content.as_bytes(), &mut assistant_message);
        let mut step = Vec::new();
        push_length_delimited(1, &assistant_message, &mut step);
        step_ids.push(Sha256::digest(&step).to_vec());
        step_bytes.push(step);
    }

    let mut agent_turn = Vec::new();
    push_length_delimited(1, &user_message_id, &mut agent_turn);
    for step_id in &step_ids {
        push_length_delimited(2, step_id, &mut agent_turn);
    }
    if let Some(request_id) = request_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        push_length_delimited(3, request_id.as_bytes(), &mut agent_turn);
    }

    let mut turn_bytes = Vec::new();
    push_length_delimited(1, &agent_turn, &mut turn_bytes);
    let turn_id = Sha256::digest(&turn_bytes).to_vec();

    // Preserve the exact prior ConversationStateStructure wire image,
    // including fields this migration has not decoded yet, and append one
    // legal repeated turns field (field 8).
    let mut state_bytes = prior_state_bytes.to_vec();
    push_length_delimited(8, &turn_id, &mut state_bytes);

    TextTurnCheckpointArtifacts {
        user_message_id,
        user_message_bytes,
        step_ids,
        step_bytes,
        turn_id,
        turn_bytes,
        state_bytes,
    }
}

pub struct ProductionAgentStateCheckpointSink {
    agent_id: String,
    agent_store: Arc<ProductionAgentStore>,
    blob_store: Arc<ProductionWorkerBlobStore>,
    transcript_mirror: Arc<ProductionRoutedTranscriptMirror>,
    transcript_persistence_enabled: bool,
    prior_state_bytes: Mutex<Vec<u8>>,
}

impl ProductionAgentStateCheckpointSink {
    pub fn new(
        agent_id: impl Into<String>,
        agent_store: Arc<ProductionAgentStore>,
        blob_store: Arc<ProductionWorkerBlobStore>,
        transcript_mirror: Arc<ProductionRoutedTranscriptMirror>,
        prior_state_bytes: Vec<u8>,
        transcript_persistence_enabled: bool,
    ) -> Result<Self, String> {
        let agent_id = agent_id.into();
        let prior = ProductionTranscriptCheckpoint::from_state_bytes(&prior_state_bytes)?;
        transcript_mirror.recover(&agent_id, &prior, &blob_store)?;
        Ok(Self {
            agent_id,
            agent_store,
            blob_store,
            transcript_mirror,
            transcript_persistence_enabled,
            prior_state_bytes: Mutex::new(prior_state_bytes),
        })
    }

    fn persist_blob(
        &self,
        id: &[u8],
        bytes: &[u8],
    ) -> Result<(), ProviderSessionError> {
        futures::executor::block_on(self.blob_store.set_blob(&(), id, bytes))
            .map_err(|error| {
                ProviderSessionError::Protocol(format!(
                    "Runner Agent checkpoint blob write failed: {error}"
                ))
            })
    }
}

impl AgentStateCheckpointSink for ProductionAgentStateCheckpointSink {
    fn checkpoint_text_turn(
        &self,
        messages: &[ProviderMessage],
        options: &TurnRunOptions,
        assistant_content: &str,
    ) -> Result<(), ProviderSessionError> {
        let user_text = options
            .recent_message_text
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                messages
                    .iter()
                    .rev()
                    .find(|message| {
                        message.role == "user" && !message.content.trim().is_empty()
                    })
                    .map(|message| message.content.as_str())
            })
            .ok_or_else(|| {
                ProviderSessionError::Protocol(
                    "Runner Agent checkpoint requires the current user message".into(),
                )
            })?;
        let message_id = options
            .message_id
            .as_deref()
            .or(options.inference_request_id.as_deref())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                ProviderSessionError::Protocol(
                    "Runner Agent checkpoint requires messageId or requestId".into(),
                )
            })?;

        let mut prior = self
            .prior_state_bytes
            .lock()
            .map_err(|_| {
                ProviderSessionError::Protocol(
                    "Runner Agent checkpoint prior state is poisoned".into(),
                )
            })?;
        let live_prior = self.agent_store.latest_checkpoint_bytes().unwrap_or_default();
        if live_prior != *prior {
            return Err(ProviderSessionError::Protocol(
                "Runner Agent checkpoint root changed while this turn was active".into(),
            ));
        }

        let artifacts = build_text_turn_checkpoint(
            &prior,
            user_text,
            message_id,
            options.inference_request_id.as_deref(),
            assistant_content,
        );
        self.persist_blob(
            &artifacts.user_message_id,
            &artifacts.user_message_bytes,
        )?;
        for (id, bytes) in artifacts.step_ids.iter().zip(&artifacts.step_bytes) {
            self.persist_blob(id, bytes)?;
        }
        self.persist_blob(&artifacts.turn_id, &artifacts.turn_bytes)?;

        let checkpoint =
            ProductionTranscriptCheckpoint::from_state_bytes(&artifacts.state_bytes)
                .map_err(|error| {
                    ProviderSessionError::Protocol(format!(
                        "Runner produced invalid ConversationStateStructure: {error}"
                    ))
                })?;
        futures::executor::block_on(persist_checkpoint_with_mirror(
            Some(self.transcript_mirror.as_ref()),
            Some(self.agent_store.as_ref()),
            &self.agent_id,
            &checkpoint,
            &self.blob_store,
            true,
            false,
            self.transcript_persistence_enabled,
            |_| {
                Err(
                    "production AgentStore is required for shipping Runner checkpoint"
                        .to_string(),
                )
            },
        ))
        .map_err(|error| {
            ProviderSessionError::Protocol(format!(
                "Runner Agent checkpoint transaction failed: {error}"
            ))
        })?;
        *prior = artifacts.state_bytes;
        Ok(())
    }
}

fn push_length_delimited(field_number: u64, value: &[u8], output: &mut Vec<u8>) {
    push_varint((field_number << 3) | 2, output);
    push_varint(value.len() as u64, output);
    output.extend_from_slice(value);
}

fn push_varint(mut value: u64, output: &mut Vec<u8>) {
    while value >= 0x80 {
        output.push((value as u8) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}
