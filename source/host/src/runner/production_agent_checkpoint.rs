use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError,
};
use crate::extensions::session::agent_db::{
    CanonicalTurnCommitRecord, OriginalTurnReceiptCommit, OriginalTurnReference,
    OriginalTurnTerminalRecord, StoredCompletedTerminal, ORIGINAL_TURN_MAX_COMMITS,
    ORIGINAL_TURN_TERMINAL_MAX_BYTES,
};
use crate::extensions::session::production::{
    OriginalTurnTerminalView, ProductionSessionWorkers,
};
use crate::extensions::session::production_agent_store::{
    ProductionAgentStore, ProductionWorkerBlobStore,
};
use crate::extensions::transcript::turn_runtime::{
    MAX_REPLY_NUDGES, TurnTerminalKind, TurnTerminalProjection, project_turn_terminal,
};
use crate::transcript_mirror::conversation_state_binary::{
    ConversationTurnStructureFields, decode_conversation_state_recovery_fields,
    decode_conversation_turn_structure_fields, decode_user_message_identity_fields,
    remove_conversation_state_token_details,
};
use crate::transcript_mirror::generated_occurrence_codec::{
    CanonicalGeneratedToolJsonProjection, GeneratedTranscriptOccurrenceCodec,
    encode_generated_agent_tool_step,
};
use crate::transcript_mirror::production_provider::{
    ProductionRoutedTranscriptMirror, ProductionTranscriptCheckpoint,
};
use crate::transcript_mirror::transcript_occurrence_deriver::{
    DecodedTranscriptStep, TranscriptOccurrenceCodec,
};
use crate::transcript_mirror::transcript_mirror_router::TranscriptMirrorRoute;

use super::{
    SettledTokenDetails, TokenDetailsPersistenceTracker, ToolEffectCustodyObservation,
    ToolEffectCustodyReport, TurnRunOptions,
};

/// An observation issued only by the real canonical/mirror commit boundary.
/// Cloning this value snapshots that boundary; JSON cannot construct one.
#[derive(Debug, Clone)]
pub struct CommittedOriginalTurnObservation {
    agent_id: String,
    canonical: CanonicalTurnCommitRecord,
}

/// A completed terminal and its sealed invocation observations. Session and
/// canonical bytes are still checked immediately before durable publication.
#[derive(Debug)]
pub struct CompletedOriginalTurnProof {
    original: OriginalTurnReference,
    terminal: TurnTerminalProjection,
    final_result_sha256: [u8; 32],
    sent_message_count: u64,
    reacted: bool,
    custody: ToolEffectCustodyReport,
    canonical: Vec<CanonicalTurnCommitRecord>,
}

/// Reconciliation authority minted after canonical reads and Session's final
/// raw-image recheck. This is deliberately neither Clone nor Deserialize.
#[derive(Debug)]
pub struct VerifiedOriginalTurnCompletion {
    original: OriginalTurnReference,
}

impl VerifiedOriginalTurnCompletion {
    pub fn original(&self) -> &OriginalTurnReference {
        &self.original
    }
}

/// Hashes the actual final successful Runner result before its Result is
/// consumed. This helper creates data, never a completed-turn capability.
pub fn original_turn_result_digest(content: &str) -> [u8; 32] {
    Sha256::digest(content.as_bytes()).into()
}

pub trait AgentStateCheckpointSink: Send + Sync {
    /// Failed generation may retain truthful history, but cannot contribute a
    /// completed-original-turn capability. Sinks without that capability have
    /// no observations to revoke; their default reader already refuses it.
    fn invalidate_original_turn_completion(&self) {}

    fn committed_original_turn_observations(
        &self,
    ) -> Result<Vec<CommittedOriginalTurnObservation>, ProviderSessionError> {
        Err(original_turn_error("committed checkpoint observations are unavailable"))
    }

    fn base_state_bytes(&self) -> Result<Vec<u8>, ProviderSessionError> {
        Ok(Vec::new())
    }

    fn read_state_blob(
        &self,
        _blob_id: &[u8],
    ) -> Result<Option<Vec<u8>>, ProviderSessionError> {
        Err(ProviderSessionError::Protocol(
            "Agent checkpoint sink does not support canonical state blob reads".into(),
        ))
    }

    fn stage_text_turn(
        &self,
        _messages: &[ProviderMessage],
        _options: &TurnRunOptions,
        _assistant_content: &str,
    ) -> Result<TextTurnCheckpointArtifacts, ProviderSessionError> {
        Err(ProviderSessionError::Protocol(
            "Agent checkpoint sink does not support staged generated state".into(),
        ))
    }

    fn stage_generated_turn(
        &self,
        messages: &[ProviderMessage],
        options: &TurnRunOptions,
        generated_step_bytes: &[Vec<u8>],
        assistant_content: &str,
    ) -> Result<TextTurnCheckpointArtifacts, ProviderSessionError> {
        if generated_step_bytes.is_empty() {
            return self.stage_text_turn(messages, options, assistant_content);
        }
        Err(ProviderSessionError::Protocol(
            "Agent checkpoint sink does not support generated tool steps".into(),
        ))
    }

    fn persist_staged_text_turn(
        &self,
        _options: &TurnRunOptions,
        _checkpoint: &TextTurnCheckpointArtifacts,
    ) -> Result<(), ProviderSessionError> {
        Err(ProviderSessionError::Protocol(
            "Agent checkpoint sink does not support staged generated state".into(),
        ))
    }

    fn checkpoint_text_turn(
        &self,
        messages: &[ProviderMessage],
        options: &TurnRunOptions,
        assistant_content: &str,
    ) -> Result<(), ProviderSessionError> {
        let checkpoint = self.stage_text_turn(messages, options, assistant_content)?;
        self.persist_staged_text_turn(options, &checkpoint)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextTurnCheckpointArtifacts {
    pub prior_state_bytes: Vec<u8>,
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
    build_text_turn_checkpoint_with_rich_text(
        prior_state_bytes,
        user_text,
        None,
        message_id,
        request_id,
        assistant_content,
    )
}

pub fn build_text_turn_checkpoint_with_rich_text(
    prior_state_bytes: &[u8],
    user_text: &str,
    user_rich_text: Option<&str>,
    message_id: &str,
    request_id: Option<&str>,
    assistant_content: &str,
) -> TextTurnCheckpointArtifacts {
    build_generated_turn_checkpoint_with_rich_text(
        prior_state_bytes,
        user_text,
        user_rich_text,
        message_id,
        request_id,
        &[],
        assistant_content,
    )
}

pub fn build_generated_turn_checkpoint_with_rich_text(
    prior_state_bytes: &[u8],
    user_text: &str,
    user_rich_text: Option<&str>,
    message_id: &str,
    request_id: Option<&str>,
    generated_step_bytes: &[Vec<u8>],
    assistant_content: &str,
) -> TextTurnCheckpointArtifacts {
    let prior_root_id = (!prior_state_bytes.is_empty())
        .then(|| Sha256::digest(prior_state_bytes).to_vec());

    let mut user_message_bytes = Vec::new();
    push_length_delimited(1, user_text.as_bytes(), &mut user_message_bytes);
    push_length_delimited(2, message_id.as_bytes(), &mut user_message_bytes);
    if let Some(user_rich_text) = user_rich_text
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        push_length_delimited(8, user_rich_text.as_bytes(), &mut user_message_bytes);
    }
    if let Some(prior_root_id) = prior_root_id.as_deref() {
        push_length_delimited(10, prior_root_id, &mut user_message_bytes);
    }
    let user_message_id = Sha256::digest(&user_message_bytes).to_vec();

    let mut step_bytes = generated_step_bytes.to_vec();
    let mut step_ids = step_bytes
        .iter()
        .map(|step| Sha256::digest(step).to_vec())
        .collect::<Vec<_>>();
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
        prior_state_bytes: prior_state_bytes.to_vec(),
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
    token_details_persistence: Mutex<TokenDetailsPersistenceTracker>,
    // None is permanent ineligibility, including skip mode, overflow, or an
    // unsupported committed shape. Observation failure never retries a turn.
    original_turn_observations: Mutex<Option<Vec<CommittedOriginalTurnObservation>>>,
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
        let observed_summary_archive_count = if prior_state_bytes.is_empty() {
            0
        } else {
            decode_conversation_state_recovery_fields(&prior_state_bytes)
                .map_err(|error| format!(
                    "Runner Agent could not decode base ConversationStateStructure for turn settlement: {error}"
                ))?
                .summary_archives
                .len()
        };
        transcript_mirror.recover(&agent_id, &prior, &blob_store)?;
        let observations_available = transcript_persistence_enabled && matches!(
            transcript_mirror.route(&agent_id), Ok(TranscriptMirrorRoute::Journal)
        );
        Ok(Self {
            agent_id,
            agent_store,
            blob_store,
            transcript_mirror,
            transcript_persistence_enabled,
            prior_state_bytes: Mutex::new(prior_state_bytes),
            token_details_persistence: Mutex::new(TokenDetailsPersistenceTracker::new(
                observed_summary_archive_count,
            )),
            original_turn_observations: Mutex::new(
                observations_available.then(Vec::new),
            ),
        })
    }

    fn persist_blob(
        &self,
        id: &[u8],
        bytes: &[u8],
    ) -> Result<(), ProviderSessionError> {
        self.blob_store
            .set_blob_blocking(id, bytes)
            .map_err(|error| {
                ProviderSessionError::Protocol(format!(
                    "Runner Agent checkpoint blob write failed: {error}"
                ))
            })
    }

    fn record_original_turn_commit(
        &self,
        prior_root_blob_id: Option<Vec<u8>>,
        root_blob_id: Vec<u8>,
        artifacts: &TextTurnCheckpointArtifacts,
    ) {
        let Ok(mut log) = self.original_turn_observations.lock() else {
            return;
        };
        let Some(observations) = log.as_mut() else {
            return;
        };
        let observed = (|| {
            if observations.len() >= ORIGINAL_TURN_MAX_COMMITS {
                return Err(original_turn_error("committed observation bound exceeded"));
            }
            let user = decode_user_message_identity_fields(&artifacts.user_message_bytes)
                .map_err(original_turn_error)?;
            let turn = decode_conversation_turn_structure_fields(&artifacts.turn_bytes)
                .map_err(original_turn_error)?;
            let Some(ConversationTurnStructureFields::Agent { request_id: Some(request_id), .. }) = turn else {
                return Err(original_turn_error("committed turn has no Agent request identity"));
            };
            let canonical = CanonicalTurnCommitRecord {
                ordinal: observations.len() as u32,
                request_id,
                message_id: user.message_id,
                prior_root_blob_id,
                root_blob_id,
                user_blob_id: artifacts.user_message_id.clone(),
                turn_blob_id: artifacts.turn_id.clone(),
                step_blob_ids: artifacts.step_ids.clone(),
                assistant_result_sha256: canonical_assistant_result_digest(&artifacts.step_bytes)?,
            };
            let mut candidate = observations.iter()
                .map(|observation| &observation.canonical).collect::<Vec<_>>();
            candidate.push(&canonical);
            if serde_json::to_vec(&candidate).map_err(original_turn_error)?.len()
                > ORIGINAL_TURN_TERMINAL_MAX_BYTES
            {
                return Err(original_turn_error("committed observation byte bound exceeded"));
            }
            Ok(CommittedOriginalTurnObservation { agent_id: self.agent_id.clone(), canonical })
        })();
        match observed {
            Ok(observation) => observations.push(observation),
            Err(_) => *log = None,
        }
    }
}

impl AgentStateCheckpointSink for ProductionAgentStateCheckpointSink {
    fn invalidate_original_turn_completion(&self) {
        if let Ok(mut observations) = self.original_turn_observations.lock() {
            *observations = None;
        }
        // A poisoned log also refuses every completion-observation read.
    }

    fn committed_original_turn_observations(
        &self,
    ) -> Result<Vec<CommittedOriginalTurnObservation>, ProviderSessionError> {
        self.original_turn_observations.lock()
            .map_err(|_| original_turn_error("committed checkpoint observations are poisoned"))?
            .clone()
            .ok_or_else(|| original_turn_error("committed checkpoint observations are unavailable"))
    }

    fn base_state_bytes(&self) -> Result<Vec<u8>, ProviderSessionError> {
        self.prior_state_bytes
            .lock()
            .map(|prior| prior.clone())
            .map_err(|_| {
                ProviderSessionError::Protocol(
                    "Runner Agent checkpoint prior state is poisoned".into(),
                )
            })
    }

    fn read_state_blob(
        &self,
        blob_id: &[u8],
    ) -> Result<Option<Vec<u8>>, ProviderSessionError> {
        self.agent_store.get_blob(blob_id).map_err(|error| {
            ProviderSessionError::Protocol(format!(
                "Runner Agent checkpoint blob read failed: {error}"
            ))
        })
    }

    fn stage_text_turn(
        &self,
        messages: &[ProviderMessage],
        options: &TurnRunOptions,
        assistant_content: &str,
    ) -> Result<TextTurnCheckpointArtifacts, ProviderSessionError> {
        self.stage_generated_turn(messages, options, &[], assistant_content)
    }

    fn stage_generated_turn(
        &self,
        messages: &[ProviderMessage],
        options: &TurnRunOptions,
        generated_step_bytes: &[Vec<u8>],
        assistant_content: &str,
    ) -> Result<TextTurnCheckpointArtifacts, ProviderSessionError> {
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
        let live_prior = self.agent_store.checkpoint_snapshot_checked()
            .map_err(|error| ProviderSessionError::Protocol(format!(
                "Runner Agent checkpoint prior state is unavailable: {error}"
            )))?
            .map(|snapshot| snapshot.checkpoint_bytes)
            .unwrap_or_default();
        if live_prior != *prior {
            return Err(ProviderSessionError::Protocol(
                "Runner Agent checkpoint root changed while this turn was active".into(),
            ));
        }

        let mut artifacts = build_generated_turn_checkpoint_with_rich_text(
            &prior,
            user_text,
            options.recent_message_rich_text.as_deref(),
            message_id,
            options.inference_request_id.as_deref(),
            generated_step_bytes,
            assistant_content,
        );
        let turn_ended_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
            .unwrap_or_default();
        let mut token_details_persistence = self
            .token_details_persistence
            .lock()
            .map_err(|_| ProviderSessionError::Protocol(
                "Runner Agent token-details settlement state is poisoned".into(),
            ))?;
        artifacts.state_bytes = prepare_settled_checkpoint_state(
            &artifacts.state_bytes,
            &mut token_details_persistence,
            options.turn_started_at_ms,
            turn_ended_at_ms,
        )?;
        Ok(artifacts)
    }

    fn persist_staged_text_turn(
        &self,
        options: &TurnRunOptions,
        artifacts: &TextTurnCheckpointArtifacts,
    ) -> Result<(), ProviderSessionError> {
        let mut prior = self
            .prior_state_bytes
            .lock()
            .map_err(|_| {
                ProviderSessionError::Protocol(
                    "Runner Agent checkpoint prior state is poisoned".into(),
                )
            })?;
        let live_prior = self.agent_store.checkpoint_snapshot_checked()
            .map_err(|error| ProviderSessionError::Protocol(format!(
                "Runner Agent checkpoint prior state is unavailable: {error}"
            )))?;
        let live_prior_bytes = live_prior.as_ref()
            .map(|snapshot| snapshot.checkpoint_bytes.as_slice()).unwrap_or_default();
        if live_prior_bytes != prior.as_slice() || artifacts.prior_state_bytes != *prior {
            return Err(ProviderSessionError::Protocol(
                "Runner Agent checkpoint root changed while this turn was active".into(),
            ));
        }
        // Preserve the actual reference, including Some(empty state) and legacy
        // fixed root IDs. Hashing the prior bytes would lose that identity.
        let prior_root_blob_id = live_prior.as_ref().map(|snapshot| snapshot.root_blob_id.clone());

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
        let mirror_prepared = self.transcript_persistence_enabled;
        if mirror_prepared {
            self.transcript_mirror
                .prepare_checkpoint(
                    &self.agent_id,
                    &checkpoint,
                    &self.blob_store,
                    true,
                    true,
                )
                .map_err(|error| {
                    ProviderSessionError::Protocol(format!(
                        "Runner Agent checkpoint transaction failed: transcript mirror prepare failed: {error}"
                    ))
                })?;
        }

        let confirmed_message_id = options
            .message_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        // The store compares this exact snapshot under its mutation guard.
        // Another writer between our read and mirror prepare must not silently
        // change the prior root that this observation will attest.
        let durable = self.agent_store.handle_checkpoint_bytes_if_current_blocking(
            &live_prior, &checkpoint.state_bytes, confirmed_message_id,
        );

        let root = match durable {
            Ok(root) => root,
            Err(error) => {
                if mirror_prepared {
                    let _ = self.transcript_mirror.abort_checkpoint(&self.agent_id);
                }
                return Err(ProviderSessionError::Protocol(format!(
                    "Runner Agent checkpoint transaction failed: durable Agent checkpoint failed: {error}"
                )));
            }
        };

        if mirror_prepared {
            if let Err(error) = self.transcript_mirror.commit_checkpoint(&self.agent_id, &root) {
                if let Ok(mut observations) = self.original_turn_observations.lock() {
                    *observations = None;
                }
                return Err(ProviderSessionError::Protocol(format!(
                    "Runner Agent checkpoint transaction failed: transcript mirror commit failed after durable checkpoint: {error}"
                )));
            }
        } else {
            self.transcript_mirror
                .skip_checkpoint(
                    &self.agent_id,
                    &checkpoint,
                    &self.blob_store,
                )
                .map_err(|error| {
                    ProviderSessionError::Protocol(format!(
                        "Runner Agent checkpoint transaction failed: transcript mirror skip failed: {error}"
                    ))
                })?;
        }
        *prior = artifacts.state_bytes.clone();
        if mirror_prepared && matches!(
            self.transcript_mirror.route(&self.agent_id), Ok(TranscriptMirrorRoute::Journal)
        ) {
            self.record_original_turn_commit(prior_root_blob_id, root, artifacts);
        } else if let Ok(mut observations) = self.original_turn_observations.lock() {
            // Legacy mirroring intentionally swallows its write errors. Its Ok
            // cannot establish the strict mirror success required by ORT-v1.
            *observations = None;
        }
        Ok(())
    }
}

fn original_turn_error(detail: impl std::fmt::Display) -> ProviderSessionError {
    ProviderSessionError::Protocol(format!("original turn completion is unavailable: {detail}"))
}

pub fn prove_completed_original_turn(
    original: &OriginalTurnReference,
    terminal: TurnTerminalProjection,
    final_result_sha256: [u8; 32],
    sent_message_count: u64,
    reacted: bool,
    custody: &ToolEffectCustodyReport,
    observations: &[CommittedOriginalTurnObservation],
) -> Result<CompletedOriginalTurnProof, ProviderSessionError> {
    if observations.is_empty() || observations.len() > MAX_REPLY_NUDGES + 2
        || observations.len() > ORIGINAL_TURN_MAX_COMMITS
    {
        return Err(original_turn_error("committed observation count is outside the original-turn bound"));
    }
    if observations.iter().any(|observation| observation.agent_id != original.agent_id) {
        return Err(original_turn_error("checkpoint observation belongs to another Agent"));
    }
    let canonical = observations.iter()
        .map(|observation| observation.canonical.clone()).collect::<Vec<_>>();
    qualify_completed_original_turn(
        original, terminal, final_result_sha256, sent_message_count, reacted, custody, &canonical,
    )?;
    Ok(CompletedOriginalTurnProof {
        original: original.clone(), terminal, final_result_sha256,
        sent_message_count, reacted, custody: custody.clone(), canonical,
    })
}

pub fn persist_completed_original_turn(
    sessions: &ProductionSessionWorkers,
    proof: &CompletedOriginalTurnProof,
) -> Result<OriginalTurnReceiptCommit, ProviderSessionError> {
    let final_root = &proof.canonical.last()
        .ok_or_else(|| original_turn_error("completed proof has no canonical commits"))?
        .root_blob_id;
    let view = sessions.prepare_original_turn_terminal_view(&proof.original, final_root)
        .map_err(original_turn_error)?;
    qualify_completed_original_turn(
        &proof.original, proof.terminal, proof.final_result_sha256,
        proof.sent_message_count, proof.reacted, &proof.custody, &proof.canonical,
    )?;
    validate_original_turn_canonical(&view, &proof.canonical)?;
    let record = OriginalTurnTerminalRecord {
        version: 1,
        original: proof.original.clone(),
        terminal: StoredCompletedTerminal::Completed,
        final_result_sha256: proof.final_result_sha256,
        sent_message_count: proof.sent_message_count,
        reacted: proof.reacted,
        custody_json: serde_json::to_string(&proof.custody).map_err(original_turn_error)?,
        canonical: proof.canonical.clone(),
        settlement_lifecycle_revision: view.settlement_lifecycle_revision.clone(),
        compaction: view.compaction.clone(),
    };
    sessions.commit_original_turn_terminal_record(&view, &record).map_err(original_turn_error)
}

pub fn reconcile_completed_original_turn(
    sessions: &ProductionSessionWorkers,
    original: &OriginalTurnReference,
) -> Result<Option<VerifiedOriginalTurnCompletion>, ProviderSessionError> {
    original.validate().map_err(original_turn_error)?;
    let Some(read) = sessions.read_original_turn_terminal_candidate(original)
        .map_err(original_turn_error)? else {
        return Ok(None);
    };
    let record = &read.record;
    if record.version != 1 || record.original != *original
        || record.terminal != StoredCompletedTerminal::Completed
        || record.settlement_lifecycle_revision != read.view.settlement_lifecycle_revision
        || record.compaction != read.view.compaction
    {
        return Err(original_turn_error("stored terminal or Session binding does not match"));
    }
    let custody = decode_original_turn_custody(&record.custody_json)?;
    qualify_completed_original_turn(
        original,
        project_turn_terminal(false, false, true, record.sent_message_count, record.reacted),
        record.final_result_sha256, record.sent_message_count, record.reacted,
        &custody, &record.canonical,
    )?;
    validate_original_turn_canonical(&read.view, &record.canonical)?;
    // Blob reads happen outside the SQL transaction. Only the Session owner can
    // check that all of its raw rows and recovery availability still agree.
    sessions.recheck_original_turn_terminal_read(&read).map_err(original_turn_error)?;
    Ok(Some(VerifiedOriginalTurnCompletion { original: original.clone() }))
}

fn qualify_completed_original_turn(
    original: &OriginalTurnReference,
    terminal: TurnTerminalProjection,
    final_result_sha256: [u8; 32],
    sent_message_count: u64,
    reacted: bool,
    custody: &ToolEffectCustodyReport,
    canonical: &[CanonicalTurnCommitRecord],
) -> Result<(), ProviderSessionError> {
    original.validate().map_err(original_turn_error)?;
    if terminal.kind != TurnTerminalKind::Completed
        || terminal != project_turn_terminal(false, false, true, sent_message_count, reacted)
    {
        return Err(original_turn_error("terminal is not a coherent typed Completed result"));
    }
    if canonical.is_empty() || canonical.len() > MAX_REPLY_NUDGES + 2
        || canonical.len() > ORIGINAL_TURN_MAX_COMMITS
        || custody.stream_id() != original.stream_id
        || custody.invocations().len() != canonical.len()
        || custody.requires_retention()
        || custody.invocations().iter().any(|observation| !matches!(
            observation, ToolEffectCustodyObservation::Known { run_generation, .. }
                if *run_generation > 0
        ))
    {
        return Err(original_turn_error("invocation count, stream or completed custody does not qualify"));
    }
    for (index, commit) in canonical.iter().enumerate() {
        let expected_message_id = if index == 0 { &original.user_message_id } else { &original.stream_id };
        if commit.ordinal as usize != index || commit.request_id != original.stream_id
            || &commit.message_id != expected_message_id
            || commit.root_blob_id.len() != 32 || commit.turn_blob_id.len() != 32
            || commit.user_blob_id.len() != 32
            || commit.step_blob_ids.iter().any(|id| id.len() != 32)
            || commit.prior_root_blob_id.as_ref().is_some_and(Vec::is_empty)
            || (index > 0 && commit.prior_root_blob_id.as_ref() != Some(&canonical[index - 1].root_blob_id))
        {
            return Err(original_turn_error("canonical invocation identity or ordered root chain differs"));
        }
    }
    if canonical.last().map(|commit| commit.assistant_result_sha256) != Some(final_result_sha256) {
        return Err(original_turn_error("final result is not the last committed invocation result"));
    }
    if serde_json::to_vec(&(original, custody, canonical)).map_err(original_turn_error)?.len()
        > ORIGINAL_TURN_TERMINAL_MAX_BYTES
    {
        return Err(original_turn_error("completed evidence exceeds the terminal byte bound"));
    }
    Ok(())
}

fn decode_original_turn_custody(raw: &str) -> Result<ToolEffectCustodyReport, ProviderSessionError> {
    if raw.len() > ORIGINAL_TURN_TERMINAL_MAX_BYTES {
        return Err(original_turn_error("stored custody exceeds its byte bound"));
    }
    // Deserialize the original string first: a Value round trip before this
    // step would erase duplicate fields. Comparing the complete typed image
    // subsequently rejects unknown nested fields without another custody schema.
    let typed: ToolEffectCustodyReport = serde_json::from_str(raw).map_err(original_turn_error)?;
    let actual: serde_json::Value = serde_json::from_str(raw).map_err(original_turn_error)?;
    if serde_json::to_value(&typed).map_err(original_turn_error)? != actual {
        return Err(original_turn_error("stored custody has an unsupported nested shape"));
    }
    Ok(typed)
}

fn required_original_turn_blob(
    store: &ProductionAgentStore,
    id: &[u8],
) -> Result<Vec<u8>, ProviderSessionError> {
    store.get_blob(id).map_err(original_turn_error)?
        .ok_or_else(|| original_turn_error("a recorded canonical blob is missing"))
}

fn require_generated_blob_id(id: &[u8], bytes: &[u8]) -> Result<(), ProviderSessionError> {
    let expected: [u8; 32] = Sha256::digest(bytes).into();
    if id != expected.as_slice() {
        return Err(original_turn_error("a generated canonical blob digest differs"));
    }
    Ok(())
}

fn validate_original_turn_canonical(
    view: &OriginalTurnTerminalView,
    canonical: &[CanonicalTurnCommitRecord],
) -> Result<(), ProviderSessionError> {
    if canonical.last().map(|commit| commit.root_blob_id.as_slice())
        != Some(view.checkpoint.root_blob_id.as_slice())
    {
        return Err(original_turn_error("the current checked root is not the recorded final root"));
    }
    for (index, commit) in canonical.iter().enumerate() {
        let prior_bytes = match commit.prior_root_blob_id.as_deref() {
            Some(id) => required_original_turn_blob(&view.store, id)?,
            None => Vec::new(),
        };
        // A prior fixed-ID blob is decoded but is not assigned a guessed SHA
        // identity. Only this writer's new blobs require content-addressed IDs.
        let prior = decode_conversation_state_recovery_fields(&prior_bytes).map_err(original_turn_error)?;
        let root_bytes = required_original_turn_blob(&view.store, &commit.root_blob_id)?;
        require_generated_blob_id(&commit.root_blob_id, &root_bytes)?;
        if index + 1 == canonical.len() && root_bytes != view.checkpoint.checkpoint_bytes {
            return Err(original_turn_error("final root bytes changed after the checked snapshot"));
        }
        let root = decode_conversation_state_recovery_fields(&root_bytes).map_err(original_turn_error)?;
        let mut expected_turns = prior.turns;
        expected_turns.push(commit.turn_blob_id.clone());
        if root.turns != expected_turns {
            return Err(original_turn_error("canonical turns are not the exact ordered prior-plus-one chain"));
        }
        let mut expected_root = prior_bytes.clone();
        push_length_delimited(8, &commit.turn_blob_id, &mut expected_root);
        match root.turn_timings.len().checked_sub(prior.turn_timings.len()) {
            Some(0) => {}
            Some(1) => {
                let timing = root.turn_timings.last().expect("one appended timing");
                append_turn_timing(&mut expected_root, timing.duration_ms, timing.timestamp_ms);
            }
            _ => return Err(original_turn_error("canonical root has an unsupported timing mutation")),
        }
        if root_bytes != expected_root
            && root_bytes != remove_conversation_state_token_details(&expected_root).map_err(original_turn_error)?
        {
            return Err(original_turn_error("canonical root differs from the settled checkpoint writer"));
        }

        let user_bytes = required_original_turn_blob(&view.store, &commit.user_blob_id)?;
        require_generated_blob_id(&commit.user_blob_id, &user_bytes)?;
        let user = decode_user_message_identity_fields(&user_bytes).map_err(original_turn_error)?;
        if user.message_id != commit.message_id
            || (index == 0 && user.text != view.original_user_content)
        {
            return Err(original_turn_error("canonical user identity or exact durable user content differs"));
        }
        let mut user_prefix = Vec::new();
        push_length_delimited(1, user.text.as_bytes(), &mut user_prefix);
        push_length_delimited(2, user.message_id.as_bytes(), &mut user_prefix);
        let mut user_suffix = Vec::new();
        if !prior_bytes.is_empty() {
            push_length_delimited(10, &Sha256::digest(&prior_bytes), &mut user_suffix);
        }
        let user_middle = user_bytes.strip_prefix(user_prefix.as_slice())
            .and_then(|remaining| remaining.strip_suffix(user_suffix.as_slice()))
            .ok_or_else(|| original_turn_error("canonical user differs from the checkpoint writer"))?;
        if !user_middle.is_empty() {
            let rich = std::str::from_utf8(canonical_embedded_field(user_middle, 8)?)
                .map_err(original_turn_error)?;
            if rich.is_empty() || rich.trim() != rich {
                return Err(original_turn_error("canonical rich text differs from the checkpoint writer"));
            }
        }

        let turn_bytes = required_original_turn_blob(&view.store, &commit.turn_blob_id)?;
        require_generated_blob_id(&commit.turn_blob_id, &turn_bytes)?;
        let turn = decode_conversation_turn_structure_fields(&turn_bytes).map_err(original_turn_error)?;
        let Some(ConversationTurnStructureFields::Agent { user_message, steps, request_id }) = turn else {
            return Err(original_turn_error("recorded turn is not an Agent turn"));
        };
        if user_message != commit.user_blob_id || steps != commit.step_blob_ids
            || request_id.as_deref() != Some(commit.request_id.as_str())
        {
            return Err(original_turn_error("canonical Agent turn references or request identity differ"));
        }
        let mut expected_agent = Vec::new();
        push_length_delimited(1, &commit.user_blob_id, &mut expected_agent);
        for id in &commit.step_blob_ids {
            push_length_delimited(2, id, &mut expected_agent);
        }
        push_length_delimited(3, commit.request_id.as_bytes(), &mut expected_agent);
        let mut expected_turn = Vec::new();
        push_length_delimited(1, &expected_agent, &mut expected_turn);
        if turn_bytes != expected_turn {
            return Err(original_turn_error("canonical Agent turn has unsupported or duplicate fields"));
        }
        let step_bytes = commit.step_blob_ids.iter().map(|id| {
            let bytes = required_original_turn_blob(&view.store, id)?;
            require_generated_blob_id(id, &bytes)?;
            Ok(bytes)
        }).collect::<Result<Vec<_>, ProviderSessionError>>()?;
        if canonical_assistant_result_digest(&step_bytes)? != commit.assistant_result_sha256 {
            return Err(original_turn_error("canonical Assistant result digest differs"));
        }
    }
    Ok(())
}

fn canonical_assistant_result_digest(steps: &[Vec<u8>]) -> Result<[u8; 32], ProviderSessionError> {
    let codec = GeneratedTranscriptOccurrenceCodec::new(CanonicalGeneratedToolJsonProjection);
    let mut result = original_turn_result_digest("");
    for (index, bytes) in steps.iter().enumerate() {
        match codec.decode_step(bytes).map_err(original_turn_error)? {
            DecodedTranscriptStep::Assistant { text } => {
                if index + 1 != steps.len() || text.is_empty() {
                    return Err(original_turn_error("Assistant result is not the single final generated step"));
                }
                let mut message = Vec::new();
                push_length_delimited(1, text.as_bytes(), &mut message);
                let mut expected = Vec::new();
                push_length_delimited(1, &message, &mut expected);
                if *bytes != expected {
                    return Err(original_turn_error("Assistant step has unsupported or duplicate fields"));
                }
                result = original_turn_result_digest(&text);
            }
            DecodedTranscriptStep::Tool { name, input, result } => {
                validate_generated_tool_step(bytes, &name, &input, result.as_ref())?;
            }
            _ => return Err(original_turn_error("unsupported generated checkpoint step")),
        }
    }
    Ok(result)
}

// These small envelope checks use prost's wire primitive and the existing
// generated decoder/encoder. They do not define another tool or Agent schema.
fn take_wire_bytes<'a>(input: &mut &'a [u8]) -> Result<&'a [u8], ProviderSessionError> {
    let length = usize::try_from(prost::encoding::decode_varint(input).map_err(original_turn_error)?)
        .map_err(original_turn_error)?;
    if length > input.len() {
        return Err(original_turn_error("generated field length exceeds its envelope"));
    }
    let bytes = *input;
    let (field, rest) = bytes.split_at(length);
    *input = rest;
    Ok(field)
}

fn canonical_embedded_field(bytes: &[u8], field: u64) -> Result<&[u8], ProviderSessionError> {
    let mut remaining = bytes;
    if prost::encoding::decode_varint(&mut remaining).map_err(original_turn_error)? != ((field << 3) | 2) {
        return Err(original_turn_error("generated envelope has the wrong field"));
    }
    let value = take_wire_bytes(&mut remaining)?;
    let mut expected = Vec::new();
    push_length_delimited(field, value, &mut expected);
    if !remaining.is_empty() || bytes != expected {
        return Err(original_turn_error("generated envelope has unsupported or duplicate fields"));
    }
    Ok(value)
}

fn validate_generated_tool_step(
    bytes: &[u8],
    name: &str,
    input: &serde_json::Value,
    result: Option<&serde_json::Value>,
) -> Result<(), ProviderSessionError> {
    let mut tool = canonical_embedded_field(bytes, 2)?;
    let variant = prost::encoding::decode_varint(&mut tool).map_err(original_turn_error)?;
    if variant & 7 != 2 {
        return Err(original_turn_error("generated tool variant is not length-delimited"));
    }
    let _ = take_wire_bytes(&mut tool)?;
    let mut call_id = None;
    let mut started = None;
    let mut completed = None;
    while !tool.is_empty() {
        match prost::encoding::decode_varint(&mut tool).map_err(original_turn_error)? {
            tag if tag == ((57 << 3) | 2) && call_id.is_none() => {
                call_id = Some(std::str::from_utf8(take_wire_bytes(&mut tool)?).map_err(original_turn_error)?);
            }
            tag if tag == 59 << 3 && started.is_none() => {
                started = Some(prost::encoding::decode_varint(&mut tool).map_err(original_turn_error)?);
            }
            tag if tag == 60 << 3 && completed.is_none() => {
                completed = Some(prost::encoding::decode_varint(&mut tool).map_err(original_turn_error)?);
            }
            _ => return Err(original_turn_error("generated tool envelope has unsupported or duplicate fields")),
        }
    }
    // The canonical codec exposes an MCP tool's display name; the same writer
    // still encodes it through the generated MCP variant (field 15).
    let generated_name = if variant >> 3 == 15 { "mcp" } else { name };
    let expected = encode_generated_agent_tool_step(
        generated_name, input, result, call_id.unwrap_or_default(), started, completed,
    ).map_err(original_turn_error)?;
    if bytes != expected {
        return Err(original_turn_error("tool step differs from the generated checkpoint writer"));
    }
    Ok(())
}


pub fn prepare_settled_checkpoint_state(
    state_bytes: &[u8],
    token_details_persistence: &mut TokenDetailsPersistenceTracker,
    turn_started_at_ms: Option<u64>,
    turn_ended_at_ms: u64,
) -> Result<Vec<u8>, ProviderSessionError> {
    let mut settled = state_bytes.to_vec();
    if let Some(turn_started_at_ms) = turn_started_at_ms {
        append_turn_timing(
            &mut settled,
            turn_ended_at_ms.saturating_sub(turn_started_at_ms),
            turn_ended_at_ms,
        );
    }
    let recovery = decode_conversation_state_recovery_fields(&settled)
        .map_err(|error| ProviderSessionError::Protocol(format!(
            "Runner Agent could not decode settled ConversationStateStructure: {error}"
        )))?;
    let token_details = recovery.token_details.map(|details| SettledTokenDetails {
        used_tokens: details.used_tokens,
        max_tokens: details.max_tokens,
    });
    if token_details_persistence.should_suppress(
        recovery.summary_archives.len(),
        token_details,
    ) && recovery.token_details.is_some()
    {
        settled = remove_conversation_state_token_details(&settled)
            .map_err(|error| ProviderSessionError::Protocol(format!(
                "Runner Agent could not suppress stale token details: {error}"
            )))?;
    }
    Ok(settled)
}

fn append_turn_timing(
    state_bytes: &mut Vec<u8>,
    duration_ms: u64,
    timestamp_ms: u64,
) {
    let mut timing = Vec::new();
    push_varint(1_u64 << 3, &mut timing);
    push_varint(duration_ms, &mut timing);
    push_varint(2_u64 << 3, &mut timing);
    push_varint(timestamp_ms, &mut timing);
    push_length_delimited(14, &timing, state_bytes);
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
