use crate::wallet::WalletTransferIdentity;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const CONNECTED_APP_REQUEST_ID_MAX_CHARS: usize = 100;
pub const CONNECTED_APP_NAME_MAX_CHARS: usize = 64;
pub const CONNECTED_APP_TRACE_ID_MAX_CHARS: usize = 256;
pub const CONNECTED_APP_ANSWER_MAX_BYTES: usize = 64 * 1024;
pub const CONNECTED_APP_OPERATION_ID_MAX_BYTES: usize = 256;
pub const CONNECTED_APP_SIGNED_PAYLOAD_MAX_BYTES: usize = 24 * 1024;
pub const CONNECTED_APP_CLAIM_MAX_RECORDS: usize = 32;
pub const CONNECTED_APP_CLAIM_MAX_BYTES: usize = 1024 * 1024;
pub const CONNECTED_APP_CLAIM_RETENTION_MS: i64 = 24 * 60 * 60 * 1000;
pub const CONNECTED_APP_RESOLVE_TIMEOUT_MS: i64 = 20 * 1000;
pub const CONNECTED_APP_MANIFEST_POLL_INTERVAL_MS: i64 = 5 * 1000;
pub const CONNECTED_APP_MANIFEST_POLL_LIMIT: u32 = 12;
pub const CONNECTED_APP_SESSION_REFRESH_FLOOR_MS: i64 = 60 * 1000;
pub const CONNECTED_APP_QUERY_MAX_BYTES: usize = 8 * 1024;
pub const CONNECTED_APP_CLIENT_ID_HEX_CHARS: usize = 64;
pub const CONNECTED_APP_KEY_REFERENCE_MAX_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectedAppLinkKind {
    Bare,
    OpenPending,
    Connect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppLink {
    pub kind: ConnectedAppLinkKind,
    pub client_id: String,
    pub manifest_url: String,
    pub proof_payload: Option<String>,
    pub trace_id: String,
    pub return_target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectedAppAccess {
    Allowed,
    WalletNotReady,
    KeyChanging,
    NoCurrentKey,
    Busy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectedAppKeyError {
    None,
    Blocked,
    Cancelled,
    Locked,
    OtherKey,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectedAppSessionStatus {
    Pending,
    Active,
    Closing,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedAppKeyReference {
    pub session_id: u64,
    pub client_id: String,
    pub reference_id: String,
    pub wallet_address: String,
    pub address_epoch: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectedAppKeyRuntime {
    pub wallet_address: String,
    pub address_epoch: u64,
    pub references: BTreeMap<u64, ConnectedAppKeyReference>,
}

impl ConnectedAppKeyRuntime {
    fn update_wallet_address(&mut self, address: &str) -> Result<bool, ConnectedAppError> {
        if address.trim() != address
            || address.as_bytes().len() > 128
            || address.chars().any(char::is_control)
        {
            return Err(ConnectedAppError::InvalidWalletAddress);
        }
        if self.wallet_address == address {
            return Ok(false);
        }
        self.address_epoch = self
            .address_epoch
            .checked_add(1)
            .ok_or(ConnectedAppError::KeyEpochOverflow)?;
        self.wallet_address = address.to_string();
        self.references.clear();
        Ok(true)
    }

    fn retain_sessions(&mut self, sessions: &BTreeMap<u64, ConnectedAppSession>) {
        self.references
            .retain(|session_id, _| sessions.contains_key(session_id));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectedAppKeyPlan {
    Cached { reference_id: String },
    InteractiveUnlock,
    SilentUnlock,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectedAppDisconnectRuntime {
    pub disconnecting: BTreeSet<u64>,
    pub close_waiting: BTreeSet<u64>,
}

impl ConnectedAppDisconnectRuntime {
    fn clear_session(&mut self, session_id: u64) {
        self.disconnecting.remove(&session_id);
        self.close_waiting.remove(&session_id);
    }

    fn retain_sessions(&mut self, sessions: &BTreeMap<u64, ConnectedAppSession>) {
        self.disconnecting
            .retain(|session_id| sessions.contains_key(session_id));
        self.close_waiting
            .retain(|session_id| sessions.contains_key(session_id));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectedAppConnectPhase {
    Resolving,
    LoadingManifest,
    Confirm,
    Connecting,
    Restore,
    Notice,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedAppConnectFlow {
    pub client_id: String,
    pub manifest_url: String,
    pub manifest_host: String,
    pub proof_payload: Option<String>,
    pub trace_id: String,
    pub phase: ConnectedAppConnectPhase,
    pub started_at_ms: i64,
    pub resolve_deadline_ms: i64,
    pub manifest_poll_count: u32,
    pub next_manifest_poll_at_ms: Option<i64>,
    pub manifest: Option<ConnectedAppManifest>,
    pub pinned_wallet_address: Option<String>,
    pub retried_challenge: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectedAppConnectRuntime {
    pub flows: BTreeMap<String, ConnectedAppConnectFlow>,
}

impl ConnectedAppConnectRuntime {
    fn begin(
        &mut self,
        link: ConnectedAppLink,
        ownership_domain: &str,
        reserved_platform_domain: &str,
        now_ms: i64,
    ) -> Result<bool, ConnectedAppError> {
        if now_ms < 0
            || link.kind != ConnectedAppLinkKind::Connect
            || !connected_app_client_id_valid(&link.client_id)
        {
            return Err(ConnectedAppError::InvalidConnectLink);
        }
        let manifest_host = connected_app_secure_url_host(&link.manifest_url)
            .ok_or(ConnectedAppError::InvalidConnectLink)?;
        if link.proof_payload.is_some()
            && !connected_app_proof_domain_allowed(
                &manifest_host,
                ownership_domain,
                reserved_platform_domain,
            )
        {
            return Err(ConnectedAppError::ProofDomainRejected);
        }

        let key = link.client_id.to_ascii_lowercase();
        if self.flows.contains_key(&key) {
            return Ok(false);
        }
        self.flows.insert(
            key,
            ConnectedAppConnectFlow {
                client_id: link.client_id,
                manifest_url: link.manifest_url,
                manifest_host,
                proof_payload: link.proof_payload,
                trace_id: link.trace_id,
                phase: ConnectedAppConnectPhase::Resolving,
                started_at_ms: now_ms,
                resolve_deadline_ms: now_ms.saturating_add(CONNECTED_APP_RESOLVE_TIMEOUT_MS),
                manifest_poll_count: 0,
                next_manifest_poll_at_ms: None,
                manifest: None,
                pinned_wallet_address: None,
                retried_challenge: false,
            },
        );
        Ok(true)
    }

    fn flow_mut(
        &mut self,
        client_id: &str,
    ) -> Result<&mut ConnectedAppConnectFlow, ConnectedAppError> {
        self.flows
            .get_mut(&client_id.to_ascii_lowercase())
            .ok_or(ConnectedAppError::ConnectFlowNotFound)
    }

    fn flow(&self, client_id: &str) -> Result<&ConnectedAppConnectFlow, ConnectedAppError> {
        self.flows
            .get(&client_id.to_ascii_lowercase())
            .ok_or(ConnectedAppError::ConnectFlowNotFound)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectedAppOperationFate {
    Unknown,
    Sending,
    Unresolved,
    Settled,
    Absent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppClaimFate {
    pub session_id: u64,
    pub message_id: i64,
    pub fate: ConnectedAppOperationFate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectedAppRecoveryAction {
    Wait,
    Offer,
    RecoverAnswer,
    SubmitAnswer,
    SubmitNotSent,
    Forget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectedAppRecoverySubmissionKind {
    Answer,
    NotSent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedAppRecoverySubmission {
    pub session_id: u64,
    pub message_id: i64,
    pub request_id: String,
    pub trace_id: String,
    pub kind: ConnectedAppRecoverySubmissionKind,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppManifest {
    pub url: String,
    pub name: String,
    pub icon_url: Option<String>,
}

impl ConnectedAppManifest {
    pub fn validate(&self) -> Result<(), ConnectedAppError> {
        if !valid_secure_url(&self.url) {
            return Err(ConnectedAppError::InvalidManifest);
        }
        if self
            .icon_url
            .as_deref()
            .is_some_and(|value| !valid_secure_url(value))
        {
            return Err(ConnectedAppError::InvalidManifest);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppSession {
    pub id: u64,
    pub client_id: String,
    pub manifest: Option<ConnectedAppManifest>,
    pub status: ConnectedAppSessionStatus,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl ConnectedAppSession {
    pub fn validate(&self) -> Result<(), ConnectedAppError> {
        if self.id == 0
            || self.client_id.trim().is_empty()
            || self.client_id.len() > CONNECTED_APP_OPERATION_ID_MAX_BYTES
            || self.created_at_ms < 0
            || self.updated_at_ms < self.created_at_ms
        {
            return Err(ConnectedAppError::InvalidSession);
        }
        if let Some(manifest) = &self.manifest {
            manifest.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectedAppRequestKind {
    SendTransaction,
    Disconnect,
    SignData,
    Unsupported,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppRequest {
    pub session_id: u64,
    #[serde(default)]
    pub message_id: i64,
    pub request_id: String,
    pub method: String,
    pub kind: ConnectedAppRequestKind,
    pub trace_id: String,
    pub expires_at_ms: i64,
}

impl ConnectedAppRequest {
    pub fn validate(&self, now_ms: i64) -> Result<(), ConnectedAppError> {
        if self.session_id == 0
            || self.message_id <= 0
            || self.message_id > i32::MAX as i64
            || !connected_app_request_id_valid(&self.request_id)
            || self.method.trim().is_empty()
            || self.trace_id.chars().count() > CONNECTED_APP_TRACE_ID_MAX_CHARS
            || self.expires_at_ms <= now_ms
        {
            return Err(ConnectedAppError::InvalidRequest);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectedAppRequestRecovery {
    None,
    Offer,
    Answer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedAppRequestEntry {
    pub session_id: u64,
    pub message_id: i64,
    pub kind: ConnectedAppRequestKind,
    pub expires_at_ms: i64,
    pub order: u64,
    pub chosen: bool,
    pub recovery: ConnectedAppRequestRecovery,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectedAppRequestRuntime {
    pub active: Option<ConnectedAppRequestEntry>,
    pub silent: BTreeMap<i64, ConnectedAppRequestEntry>,
    pub waiting: Vec<ConnectedAppRequestEntry>,
    pub claimed_ids: BTreeSet<i64>,
    pub recovery_held: BTreeSet<i64>,
    pub recovery_submissions: BTreeMap<i64, ConnectedAppRecoverySubmission>,
    pub recovery_polling: bool,
    pub stopped: bool,
    order: u64,
}

impl ConnectedAppRequestRuntime {
    fn owns(&self, message_id: i64) -> bool {
        self.active.as_ref().is_some_and(|entry| entry.message_id == message_id)
            || self.silent.contains_key(&message_id)
            || self.waiting.iter().any(|entry| entry.message_id == message_id)
    }

    fn enqueue(
        &mut self,
        request: &ConnectedAppRequest,
        chosen: bool,
    ) -> Result<(), ConnectedAppError> {
        if self.stopped {
            return Err(ConnectedAppError::RequestRuntimeStopped);
        }
        if self.owns(request.message_id) || self.claimed_ids.contains(&request.message_id) {
            return Err(ConnectedAppError::DuplicateRequest);
        }
        self.order = self
            .order
            .checked_add(1)
            .ok_or(ConnectedAppError::RequestOrderOverflow)?;
        let entry = ConnectedAppRequestEntry {
            session_id: request.session_id,
            message_id: request.message_id,
            kind: request.kind,
            expires_at_ms: request.expires_at_ms,
            order: self.order,
            chosen,
            recovery: ConnectedAppRequestRecovery::None,
        };
        if request.kind == ConnectedAppRequestKind::Disconnect {
            self.silent.insert(entry.message_id, entry);
        } else {
            self.waiting.push(entry);
        }
        Ok(())
    }

    fn enqueue_recovery(
        &mut self,
        claim: &ConnectedAppClaimRecord,
        recovery: ConnectedAppRequestRecovery,
    ) -> Result<bool, ConnectedAppError> {
        if self.stopped {
            return Err(ConnectedAppError::RequestRuntimeStopped);
        }
        if self.owns(claim.message_id) || self.recovery_held.contains(&claim.message_id) {
            return Ok(false);
        }
        self.order = self
            .order
            .checked_add(1)
            .ok_or(ConnectedAppError::RequestOrderOverflow)?;
        self.waiting.push(ConnectedAppRequestEntry {
            session_id: claim.session_id,
            message_id: claim.message_id,
            kind: ConnectedAppRequestKind::SendTransaction,
            expires_at_ms: claim.expires_at_ms,
            order: self.order,
            chosen: false,
            recovery,
        });
        Ok(true)
    }

    fn hold_recovery_submission(
        &mut self,
        claim: &ConnectedAppClaimRecord,
        kind: ConnectedAppRecoverySubmissionKind,
        body: Vec<u8>,
    ) -> bool {
        if body.is_empty()
            || self.recovery_held.contains(&claim.message_id)
            || self.owns(claim.message_id)
        {
            return false;
        }
        self.recovery_held.insert(claim.message_id);
        self.recovery_submissions.insert(
            claim.message_id,
            ConnectedAppRecoverySubmission {
                session_id: claim.session_id,
                message_id: claim.message_id,
                request_id: claim.request_id.clone(),
                trace_id: claim.trace_id.clone(),
                kind,
                body,
            },
        );
        true
    }

    fn clear_recovery_submission(&mut self, message_id: i64) {
        self.recovery_submissions.remove(&message_id);
        self.recovery_held.remove(&message_id);
    }

    fn clear_session(&mut self, session_id: u64) {
        if self.active.as_ref().is_some_and(|entry| entry.session_id == session_id) {
            self.active = None;
        }
        self.silent.retain(|_, entry| entry.session_id != session_id);
        self.waiting.retain(|entry| entry.session_id != session_id);
    }

    fn retain_sessions(&mut self, sessions: &BTreeMap<u64, ConnectedAppSession>) {
        if self.active.as_ref().is_some_and(|entry| !sessions.contains_key(&entry.session_id)) {
            self.active = None;
        }
        self.silent.retain(|_, entry| sessions.contains_key(&entry.session_id));
        self.waiting.retain(|entry| sessions.contains_key(&entry.session_id));
    }

    fn take_next(
        &mut self,
        now_ms: i64,
    ) -> Result<Option<ConnectedAppRequestEntry>, ConnectedAppError> {
        if now_ms < 0 {
            return Err(ConnectedAppError::InvalidRequest);
        }
        if self.stopped || self.active.is_some() {
            return Ok(None);
        }
        let claimed = &self.claimed_ids;
        self.waiting.retain(|entry| match entry.recovery {
            ConnectedAppRequestRecovery::Answer => true,
            ConnectedAppRequestRecovery::Offer => entry.expires_at_ms > now_ms,
            ConnectedAppRequestRecovery::None => {
                entry.expires_at_ms > now_ms && !claimed.contains(&entry.message_id)
            }
        });
        if self.waiting.is_empty() {
            return Ok(None);
        }
        let mut index = self
            .waiting
            .iter()
            .enumerate()
            .min_by_key(|(_, entry)| entry.order)
            .map(|(index, _)| index)
            .expect("waiting is non-empty");
        if !self.waiting[index].chosen {
            let session_id = self.waiting[index].session_id;
            if let Some((same_index, _)) = self
                .waiting
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.session_id == session_id)
                .min_by_key(|(_, entry)| entry.message_id)
            {
                index = same_index;
            }
        }
        let entry = self.waiting.remove(index);
        self.active = Some(entry.clone());
        Ok(Some(entry))
    }

    fn choose(&mut self, message_id: i64) -> Result<(), ConnectedAppError> {
        if self.stopped {
            return Err(ConnectedAppError::RequestRuntimeStopped);
        }
        if self.silent.contains_key(&message_id) {
            return Err(ConnectedAppError::RequestOwnedSilently);
        }
        if self.active.as_ref().is_some_and(|entry| entry.message_id == message_id) {
            return Ok(());
        }
        let index = self
            .waiting
            .iter()
            .position(|entry| entry.message_id == message_id)
            .ok_or(ConnectedAppError::RequestNotFound)?;
        self.waiting[index].chosen = true;
        if self.active.is_some() {
            let mut entry = self.waiting.remove(index);
            entry.order = 0;
            self.waiting.insert(0, entry);
        } else {
            let entry = self.waiting.remove(index);
            self.active = Some(entry);
        }
        Ok(())
    }

    fn handled_elsewhere(&mut self, message_id: i64) -> bool {
        let mut changed = false;
        if self.active.as_ref().is_some_and(|entry| {
            entry.message_id == message_id && entry.recovery == ConnectedAppRequestRecovery::None
        }) {
            self.active = None;
            changed = true;
        }
        if self.silent.remove(&message_id).is_some() {
            changed = true;
        }
        let before = self.waiting.len();
        self.waiting.retain(|entry| {
            entry.message_id != message_id || entry.recovery != ConnectedAppRequestRecovery::None
        });
        changed || before != self.waiting.len()
    }

    fn finish(&mut self, message_id: i64, claimed: bool) -> Result<(), ConnectedAppError> {
        let mut found = false;
        if self.active.as_ref().is_some_and(|entry| entry.message_id == message_id) {
            self.active = None;
            found = true;
        }
        if self.silent.remove(&message_id).is_some() {
            found = true;
        }
        if !found {
            return Err(ConnectedAppError::RequestNotFound);
        }
        if claimed {
            self.claimed_ids.insert(message_id);
        }
        Ok(())
    }

    fn stop(&mut self) {
        self.active = None;
        self.silent.clear();
        self.waiting.clear();
        self.recovery_held.clear();
        self.recovery_submissions.clear();
        self.recovery_polling = false;
        self.stopped = true;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectedAppClaimDecision {
    Confirm,
    Answer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppClaimRecord {
    pub session_id: u64,
    #[serde(default)]
    pub message_id: i64,
    pub request_id: String,
    pub trace_id: String,
    pub expires_at_ms: i64,
    pub decision: ConnectedAppClaimDecision,
    #[serde(default)]
    pub wallet_identity: Option<WalletTransferIdentity>,
    pub operation_id: String,
    pub signed_payload: String,
    #[serde(default)]
    pub not_sent: Vec<u8>,
    pub answer: Vec<u8>,
    pub created_at_ms: i64,
}

impl ConnectedAppClaimRecord {
    pub fn validate(&self) -> Result<(), ConnectedAppError> {
        // Durable claims are recovery authority, not a best-effort cache.
        // Confirm keeps the encrypted "not sent" response independently from
        // the eventual success answer. This lets restart recovery distinguish
        // "never handed off" from "already answered" without guessing.
        let decision_shape_valid = match self.decision {
            ConnectedAppClaimDecision::Confirm => !self.not_sent.is_empty(),
            ConnectedAppClaimDecision::Answer => {
                !self.answer.is_empty()
                    && self.not_sent.is_empty()
                    && self.operation_id.is_empty()
                    && self.signed_payload.is_empty()
            }
        };
        if self.session_id == 0
            || self.message_id <= 0
            || self.message_id > i32::MAX as i64
            || !connected_app_request_id_valid(&self.request_id)
            || self.trace_id.chars().count() > CONNECTED_APP_TRACE_ID_MAX_CHARS
            || self.expires_at_ms <= 0
            || self.created_at_ms <= 0
            || self.operation_id.len() > CONNECTED_APP_OPERATION_ID_MAX_BYTES
            || self.signed_payload.len() > CONNECTED_APP_SIGNED_PAYLOAD_MAX_BYTES
            || self.not_sent.len() > CONNECTED_APP_ANSWER_MAX_BYTES
            || self.answer.len() > CONNECTED_APP_ANSWER_MAX_BYTES
            || self.wallet_identity.as_ref().is_some_and(|identity| identity.validate().is_err())
            || (!self.signed_payload.is_empty() && self.operation_id.is_empty())
            || !decision_shape_valid
        {
            return Err(ConnectedAppError::InvalidClaim);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectedAppSessionRefreshState {
    pub generation: u64,
    pub in_flight: bool,
    pub requested_at_ms: Option<i64>,
    pub changed_while_loading: BTreeMap<u64, ConnectedAppSession>,
}

impl ConnectedAppSessionRefreshState {
    fn begin(&mut self, now_ms: i64, wallet_ready: bool) -> Result<Option<u64>, ConnectedAppError> {
        if now_ms < 0 { return Err(ConnectedAppError::InvalidRefreshTimestamp); }
        if !wallet_ready || self.in_flight { return Ok(None); }
        if self.requested_at_ms.is_some_and(|last| now_ms.saturating_sub(last) < CONNECTED_APP_SESSION_REFRESH_FLOOR_MS) {
            return Ok(None);
        }
        self.generation = self.generation.checked_add(1).ok_or(ConnectedAppError::RefreshGenerationOverflow)?;
        self.in_flight = true;
        self.requested_at_ms = Some(now_ms);
        self.changed_while_loading.clear();
        Ok(Some(self.generation))
    }
    fn require(&self, generation: u64) -> Result<(), ConnectedAppError> {
        if self.in_flight && generation != 0 && generation == self.generation { Ok(()) }
        else { Err(ConnectedAppError::StaleSessionRefresh { current: self.generation, received: generation }) }
    }
    fn settle(&mut self) { self.in_flight = false; self.changed_while_loading.clear(); }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ConnectedAppState {
    pub sessions: BTreeMap<u64, ConnectedAppSession>,
    pub pending_requests: BTreeMap<String, ConnectedAppRequest>,
    pub claims: Vec<ConnectedAppClaimRecord>,
    #[serde(default)]
    pub closed_session_ids: BTreeSet<u64>,
    #[serde(skip, default)]
    pub session_refresh: ConnectedAppSessionRefreshState,
    #[serde(skip, default)]
    pub key_runtime: ConnectedAppKeyRuntime,
    #[serde(skip, default)]
    pub disconnect_runtime: ConnectedAppDisconnectRuntime,
    #[serde(skip, default)]
    pub connect_runtime: ConnectedAppConnectRuntime,
    #[serde(skip, default)]
    pub request_runtime: ConnectedAppRequestRuntime,
}

impl ConnectedAppState {
    pub fn upsert_session(
        &mut self,
        session: ConnectedAppSession,
    ) -> Result<(), ConnectedAppError> {
        self.write_session(session, true)
    }

    fn write_session(
        &mut self,
        session: ConnectedAppSession,
        track_refresh_change: bool,
    ) -> Result<(), ConnectedAppError> {
        session.validate()?;
        if session.status != ConnectedAppSessionStatus::Closed
            && self.closed_session_ids.contains(&session.id)
        {
            return Err(ConnectedAppError::InvalidSessionTransition);
        }
        if let Some(previous) = self.sessions.get(&session.id) {
            if previous.client_id != session.client_id
                || session.created_at_ms != previous.created_at_ms
                || session.updated_at_ms < previous.updated_at_ms
                || !valid_session_transition(previous.status, session.status)
            {
                return Err(ConnectedAppError::InvalidSessionTransition);
            }
        }
        if track_refresh_change && self.session_refresh.in_flight {
            self.session_refresh.changed_while_loading.insert(session.id, session.clone());
        }
        if session.status == ConnectedAppSessionStatus::Closed {
            self.closed_session_ids.insert(session.id);
            self.sessions.remove(&session.id);
            self.pending_requests
                .retain(|_, request| request.session_id != session.id);
            self.request_runtime.clear_session(session.id);
            self.key_runtime.references.remove(&session.id);
            self.disconnect_runtime.clear_session(session.id);
        } else {
            self.sessions.insert(session.id, session);
        }
        Ok(())
    }

    pub fn update_wallet_address(&mut self, address: &str) -> Result<bool, ConnectedAppError> {
        self.key_runtime.update_wallet_address(address)
    }

    pub fn cache_key_reference(
        &mut self,
        session_id: u64,
        client_id: &str,
        reference_id: &str,
    ) -> Result<(), ConnectedAppError> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(ConnectedAppError::SessionNotFound)?;
        if self.key_runtime.wallet_address.is_empty()
            || client_id.trim().is_empty()
            || reference_id.trim().is_empty()
            || reference_id.as_bytes().len() > CONNECTED_APP_KEY_REFERENCE_MAX_BYTES
            || !session.client_id.eq_ignore_ascii_case(client_id)
        {
            return Err(ConnectedAppError::InvalidKeyReference);
        }

        self.key_runtime.references.insert(
            session_id,
            ConnectedAppKeyReference {
                session_id,
                client_id: client_id.to_string(),
                reference_id: reference_id.to_string(),
                wallet_address: self.key_runtime.wallet_address.clone(),
                address_epoch: self.key_runtime.address_epoch,
            },
        );
        Ok(())
    }

    pub fn interactive_key_plan(
        &self,
        session_id: u64,
        need_grant: bool,
    ) -> Result<ConnectedAppKeyPlan, ConnectedAppKeyError> {
        if !self.sessions.contains_key(&session_id) {
            return Err(ConnectedAppKeyError::Failed);
        }
        if !need_grant {
            if let Some(reference) = self.key_runtime.references.get(&session_id) {
                return Ok(ConnectedAppKeyPlan::Cached {
                    reference_id: reference.reference_id.clone(),
                });
            }
        }
        Ok(ConnectedAppKeyPlan::InteractiveUnlock)
    }

    pub fn silent_key_plan(
        &self,
        session_id: u64,
        access: ConnectedAppAccess,
        silent_unlock_available: bool,
    ) -> Result<ConnectedAppKeyPlan, ConnectedAppKeyError> {
        if !self.sessions.contains_key(&session_id) {
            return Err(ConnectedAppKeyError::Failed);
        }
        if let Some(reference) = self.key_runtime.references.get(&session_id) {
            return Ok(ConnectedAppKeyPlan::Cached {
                reference_id: reference.reference_id.clone(),
            });
        }
        if access != ConnectedAppAccess::Allowed {
            return Err(ConnectedAppKeyError::Blocked);
        }
        if !silent_unlock_available {
            return Err(ConnectedAppKeyError::Locked);
        }
        Ok(ConnectedAppKeyPlan::SilentUnlock)
    }

    pub fn begin_disconnect(&mut self, session_id: u64) -> Result<bool, ConnectedAppError> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(ConnectedAppError::SessionNotFound)?;
        if !matches!(
            session.status,
            ConnectedAppSessionStatus::Active | ConnectedAppSessionStatus::Closing
        ) {
            return Err(ConnectedAppError::SessionClosed);
        }
        self.disconnect_runtime.close_waiting.remove(&session_id);
        Ok(self.disconnect_runtime.disconnecting.insert(session_id))
    }

    pub fn settle_disconnect(&mut self, session_id: u64) -> bool {
        self.disconnect_runtime.disconnecting.remove(&session_id)
    }

    pub fn defer_closing_for_key_error(
        &mut self,
        session_id: u64,
        error: ConnectedAppKeyError,
    ) -> Result<bool, ConnectedAppError> {
        self.disconnect_runtime.disconnecting.remove(&session_id);
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(ConnectedAppError::SessionNotFound)?;
        if session.status == ConnectedAppSessionStatus::Closing
            && matches!(error, ConnectedAppKeyError::Locked | ConnectedAppKeyError::Blocked)
        {
            return Ok(self.disconnect_runtime.close_waiting.insert(session_id));
        }
        self.disconnect_runtime.close_waiting.remove(&session_id);
        Ok(false)
    }

    pub fn take_close_waiting_after_vault_change(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.disconnect_runtime.close_waiting)
            .into_iter()
            .collect()
    }

    pub fn begin_connect_flow(
        &mut self,
        link: ConnectedAppLink,
        ownership_domain: &str,
        reserved_platform_domain: &str,
        observed_at_ms: i64,
    ) -> Result<bool, ConnectedAppError> {
        self.connect_runtime.begin(
            link,
            ownership_domain,
            reserved_platform_domain,
            observed_at_ms,
        )
    }

    pub fn resolve_connect_access(
        &mut self,
        client_id: &str,
        access: ConnectedAppAccess,
        observed_at_ms: i64,
    ) -> Result<ConnectedAppConnectPhase, ConnectedAppError> {
        let flow = self.connect_runtime.flow_mut(client_id)?;
        if observed_at_ms < 0 || observed_at_ms >= flow.resolve_deadline_ms {
            flow.phase = ConnectedAppConnectPhase::Notice;
            return Err(ConnectedAppError::ConnectResolveTimedOut);
        }

        flow.phase = match access {
            ConnectedAppAccess::Allowed => {
                flow.next_manifest_poll_at_ms =
                    Some(observed_at_ms.saturating_add(CONNECTED_APP_MANIFEST_POLL_INTERVAL_MS));
                ConnectedAppConnectPhase::LoadingManifest
            }
            ConnectedAppAccess::NoCurrentKey => ConnectedAppConnectPhase::Restore,
            blocked => {
                flow.phase = ConnectedAppConnectPhase::Notice;
                return Err(ConnectedAppError::AccessBlocked(blocked));
            }
        };
        Ok(flow.phase)
    }

    pub fn poll_connect_manifest(
        &mut self,
        client_id: &str,
        observed_at_ms: i64,
    ) -> Result<bool, ConnectedAppError> {
        let flow = self.connect_runtime.flow_mut(client_id)?;
        if flow.phase != ConnectedAppConnectPhase::LoadingManifest {
            return Err(ConnectedAppError::InvalidConnectPhase);
        }
        let due = flow.next_manifest_poll_at_ms.unwrap_or(observed_at_ms);
        if observed_at_ms < due {
            return Ok(false);
        }
        if flow.manifest_poll_count >= CONNECTED_APP_MANIFEST_POLL_LIMIT {
            flow.phase = ConnectedAppConnectPhase::Notice;
            return Err(ConnectedAppError::ManifestPollExhausted);
        }
        flow.manifest_poll_count += 1;
        flow.next_manifest_poll_at_ms =
            Some(observed_at_ms.saturating_add(CONNECTED_APP_MANIFEST_POLL_INTERVAL_MS));
        Ok(true)
    }

    pub fn accept_connect_manifest(
        &mut self,
        client_id: &str,
        manifest: ConnectedAppManifest,
    ) -> Result<(), ConnectedAppError> {
        manifest.validate()?;
        let host = connected_app_secure_url_host(&manifest.url)
            .ok_or(ConnectedAppError::InvalidManifest)?;
        let flow = self.connect_runtime.flow_mut(client_id)?;
        if !host.eq_ignore_ascii_case(&flow.manifest_host) {
            flow.phase = ConnectedAppConnectPhase::Notice;
            return Err(ConnectedAppError::ManifestHostMismatch);
        }
        flow.manifest = Some(manifest);
        flow.phase = ConnectedAppConnectPhase::Confirm;
        flow.next_manifest_poll_at_ms = None;
        Ok(())
    }

    pub fn pin_connect_wallet(
        &mut self,
        client_id: &str,
        wallet_address: &str,
    ) -> Result<(), ConnectedAppError> {
        if wallet_address.is_empty()
            || wallet_address.trim() != wallet_address
            || wallet_address.as_bytes().len() > 128
            || wallet_address.chars().any(char::is_control)
        {
            return Err(ConnectedAppError::InvalidWalletAddress);
        }
        let flow = self.connect_runtime.flow_mut(client_id)?;
        if flow.phase != ConnectedAppConnectPhase::Confirm {
            return Err(ConnectedAppError::InvalidConnectPhase);
        }
        flow.pinned_wallet_address = Some(wallet_address.to_string());
        flow.phase = ConnectedAppConnectPhase::Connecting;
        Ok(())
    }

    pub fn ensure_connect_wallet_unchanged(
        &self,
        client_id: &str,
        wallet_address: &str,
    ) -> Result<(), ConnectedAppError> {
        let flow = self.connect_runtime.flow(client_id)?;
        if flow.pinned_wallet_address.as_deref() != Some(wallet_address) {
            return Err(ConnectedAppError::WalletAddressChanged);
        }
        Ok(())
    }

    pub fn retry_connect_submit_failure(
        &mut self,
        client_id: &str,
        error_type: &str,
    ) -> Result<bool, ConnectedAppError> {
        let flow = self.connect_runtime.flow_mut(client_id)?;
        let retryable = matches!(
            error_type,
            "TONCONNECT_CHALLENGE_INVALID" | "TONCONNECT_SESSION_NOT_ACTIVE"
        );
        if !retryable || flow.retried_challenge {
            return Ok(false);
        }
        flow.retried_challenge = true;
        Ok(true)
    }

    pub fn finish_connect_flow(&mut self, client_id: &str) -> bool {
        self.connect_runtime
            .flows
            .remove(&client_id.to_ascii_lowercase())
            .is_some()
    }

    pub fn begin_session_refresh(
        &mut self,
        now_ms: i64,
        wallet_ready: bool,
    ) -> Result<Option<u64>, ConnectedAppError> {
        self.session_refresh.begin(now_ms, wallet_ready)
    }

    pub fn apply_session_refresh(
        &mut self,
        generation: u64,
        sessions: Vec<ConnectedAppSession>,
        observed_at_ms: i64,
    ) -> Result<(), ConnectedAppError> {
        if observed_at_ms < 0 { return Err(ConnectedAppError::InvalidRefreshTimestamp); }
        self.session_refresh.require(generation)?;
        let mut remote = BTreeMap::new();
        for session in sessions {
            session.validate()?;
            if session.status == ConnectedAppSessionStatus::Closed
                || self.closed_session_ids.contains(&session.id)
            {
                continue;
            }
            if remote.insert(session.id, session).is_some() {
                return Err(ConnectedAppError::DuplicateSessionInRefresh);
            }
        }
        let changed = std::mem::take(&mut self.session_refresh.changed_while_loading);
        self.sessions = remote;
        self.pending_requests.retain(|_, request| self.sessions.contains_key(&request.session_id));
        for (_, local) in changed {
            if local.status == ConnectedAppSessionStatus::Closed {
                self.closed_session_ids.insert(local.id);
                self.sessions.remove(&local.id);
                self.pending_requests
                    .retain(|_, request| request.session_id != local.id);
            } else if !self.closed_session_ids.contains(&local.id) {
                self.sessions.insert(local.id, local);
            }
        }
        self.key_runtime.retain_sessions(&self.sessions);
        self.disconnect_runtime.retain_sessions(&self.sessions);
        self.request_runtime.retain_sessions(&self.sessions);
        self.session_refresh.settle();
        Ok(())
    }

    pub fn fail_session_refresh(&mut self, generation: u64) -> Result<(), ConnectedAppError> {
        self.session_refresh.require(generation)?;
        self.session_refresh.settle();
        Ok(())
    }

    pub fn mark_session_closing(
        &mut self,
        session_id: u64,
        observed_at_ms: i64,
    ) -> Result<(), ConnectedAppError> {
        let current = self.sessions.get(&session_id).cloned().ok_or(ConnectedAppError::SessionNotFound)?;
        if observed_at_ms < current.updated_at_ms { return Err(ConnectedAppError::StaleMutation); }
        let mut next = current;
        next.status = ConnectedAppSessionStatus::Closing;
        next.updated_at_ms = observed_at_ms;
        self.write_session(next, true)
    }

    pub fn closing_session_ids(&self) -> Vec<u64> {
        self.sessions.iter().filter_map(|(&id, session)| {
            (session.status == ConnectedAppSessionStatus::Closing).then_some(id)
        }).collect()
    }

    pub fn close_session(
        &mut self,
        session_id: u64,
        closed_at_ms: i64,
    ) -> Result<(), ConnectedAppError> {
        let current = self.sessions.get(&session_id).cloned().ok_or(ConnectedAppError::SessionNotFound)?;
        if closed_at_ms < current.updated_at_ms { return Err(ConnectedAppError::StaleMutation); }
        let mut closed = current;
        closed.status = ConnectedAppSessionStatus::Closed;
        closed.updated_at_ms = closed_at_ms;
        self.write_session(closed, true)
    }

    pub fn acknowledge_session_gone(
        &mut self,
        session_id: u64,
        observed_at_ms: i64,
    ) -> Result<(), ConnectedAppError> {
        match self.sessions.get(&session_id).cloned() {
            Some(session) => {
                if observed_at_ms < session.updated_at_ms { return Err(ConnectedAppError::StaleMutation); }
                self.close_session(session_id, observed_at_ms)
            }
            None => Ok(()),
        }
    }

    pub fn queue_request(
        &mut self,
        request: ConnectedAppRequest,
        now_ms: i64,
    ) -> Result<(), ConnectedAppError> {
        request.validate(now_ms)?;
        let session = self
            .sessions
            .get(&request.session_id)
            .ok_or(ConnectedAppError::SessionNotFound)?;
        if !matches!(
            session.status,
            ConnectedAppSessionStatus::Pending | ConnectedAppSessionStatus::Active
        ) {
            return Err(ConnectedAppError::SessionClosed);
        }
        let key = request_key(request.session_id, request.message_id);
        if self.pending_requests.contains_key(&key)
            || self.claims.iter().any(|claim| {
                claim.session_id == request.session_id && claim.message_id == request.message_id
            })
        {
            return Err(ConnectedAppError::DuplicateRequest);
        }
        self.request_runtime.enqueue(&request, false)?;
        self.pending_requests.insert(key, request);
        Ok(())
    }

    pub fn take_next_request(
        &mut self,
        now_ms: i64,
    ) -> Result<Option<ConnectedAppRequestEntry>, ConnectedAppError> {
        let claims = &self.claims;
        self.pending_requests.retain(|_, request| {
            request.expires_at_ms > now_ms
                && !claims.iter().any(|claim| {
                    claim.session_id == request.session_id
                        && claim.message_id == request.message_id
                })
        });
        self.request_runtime.take_next(now_ms)
    }

    pub fn choose_request(&mut self, message_id: i64) -> Result<(), ConnectedAppError> {
        if !self.request_runtime.owns(message_id) {
            let request = self
                .pending_requests
                .values()
                .find(|request| request.message_id == message_id)
                .cloned()
                .ok_or(ConnectedAppError::RequestNotFound)?;
            self.request_runtime.enqueue(&request, true)?;
        }
        self.request_runtime.choose(message_id)
    }

    pub fn mark_request_handled_elsewhere(&mut self, message_id: i64) -> bool {
        let changed = self.request_runtime.handled_elsewhere(message_id);
        if changed {
            self.pending_requests
                .retain(|_, request| request.message_id != message_id);
        }
        changed
    }

    pub fn finish_request(
        &mut self,
        message_id: i64,
        claimed: bool,
    ) -> Result<(), ConnectedAppError> {
        self.request_runtime.finish(message_id, claimed)?;
        if claimed {
            self.pending_requests
                .retain(|_, request| request.message_id != message_id);
        }
        Ok(())
    }

    pub fn stop_requests(&mut self) {
        self.request_runtime.stop();
    }

    pub fn record_claim(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        decision: ConnectedAppClaimDecision,
        operation_id: String,
        signed_payload: String,
        answer: Vec<u8>,
        now_ms: i64,
    ) -> Result<ConnectedAppClaimRecord, ConnectedAppError> {
        if decision == ConnectedAppClaimDecision::Confirm {
            return Err(ConnectedAppError::MissingRecoveryFallback);
        }
        self.record_claim_with_recovery(
            session_id,
            message_id,
            request_id,
            decision,
            operation_id,
            signed_payload,
            Vec::new(),
            answer,
            now_ms,
        )
    }

    pub fn record_claim_with_recovery(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        decision: ConnectedAppClaimDecision,
        operation_id: String,
        signed_payload: String,
        not_sent: Vec<u8>,
        answer: Vec<u8>,
        now_ms: i64,
    ) -> Result<ConnectedAppClaimRecord, ConnectedAppError> {
        self.record_claim_internal(
            session_id,
            message_id,
            request_id,
            decision,
            None,
            operation_id,
            signed_payload,
            not_sent,
            answer,
            now_ms,
        )
    }

    pub fn record_wallet_claim(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        decision: ConnectedAppClaimDecision,
        wallet_identity: WalletTransferIdentity,
        operation_id: String,
        signed_payload: String,
        answer: Vec<u8>,
        now_ms: i64,
    ) -> Result<ConnectedAppClaimRecord, ConnectedAppError> {
        if decision == ConnectedAppClaimDecision::Confirm {
            return Err(ConnectedAppError::MissingRecoveryFallback);
        }
        self.record_wallet_claim_with_recovery(
            session_id,
            message_id,
            request_id,
            decision,
            wallet_identity,
            operation_id,
            signed_payload,
            Vec::new(),
            answer,
            now_ms,
        )
    }

    pub fn record_wallet_claim_with_recovery(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        decision: ConnectedAppClaimDecision,
        wallet_identity: WalletTransferIdentity,
        operation_id: String,
        signed_payload: String,
        not_sent: Vec<u8>,
        answer: Vec<u8>,
        now_ms: i64,
    ) -> Result<ConnectedAppClaimRecord, ConnectedAppError> {
        wallet_identity
            .validate()
            .map_err(|_| ConnectedAppError::InvalidWalletBinding)?;
        self.record_claim_internal(
            session_id,
            message_id,
            request_id,
            decision,
            Some(wallet_identity),
            operation_id,
            signed_payload,
            not_sent,
            answer,
            now_ms,
        )
    }

    pub fn link_claim_operation(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<(), ConnectedAppError> {
        if operation_id.trim().is_empty()
            || operation_id.as_bytes().len() > CONNECTED_APP_OPERATION_ID_MAX_BYTES
        {
            return Err(ConnectedAppError::ClaimTransition);
        }
        let claim = self.claim_mut(session_id, message_id, request_id)?;
        if claim.decision != ConnectedAppClaimDecision::Confirm
            || !claim.signed_payload.is_empty()
            || !claim.answer.is_empty()
            || (!claim.operation_id.is_empty() && claim.operation_id != operation_id)
        {
            return Err(ConnectedAppError::ClaimTransition);
        }
        claim.operation_id = operation_id.to_string();
        claim.validate()
    }

    pub fn record_claim_handoff(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        operation_id: &str,
        signed_payload: &str,
    ) -> Result<(), ConnectedAppError> {
        if signed_payload.is_empty()
            || signed_payload.as_bytes().len() > CONNECTED_APP_SIGNED_PAYLOAD_MAX_BYTES
        {
            return Err(ConnectedAppError::ClaimTransition);
        }
        let claim = self.claim_mut(session_id, message_id, request_id)?;
        if claim.decision != ConnectedAppClaimDecision::Confirm
            || claim.operation_id != operation_id
            || operation_id.is_empty()
            || !claim.answer.is_empty()
            || (!claim.signed_payload.is_empty() && claim.signed_payload != signed_payload)
        {
            return Err(ConnectedAppError::ClaimTransition);
        }
        claim.signed_payload = signed_payload.to_string();
        claim.validate()
    }

    pub fn record_claim_answer(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        answer: Vec<u8>,
    ) -> Result<(), ConnectedAppError> {
        if answer.is_empty() || answer.len() > CONNECTED_APP_ANSWER_MAX_BYTES {
            return Err(ConnectedAppError::ClaimTransition);
        }
        let claim = self.claim_mut(session_id, message_id, request_id)?;
        if claim.decision != ConnectedAppClaimDecision::Confirm
            || claim.operation_id.is_empty()
            || (!claim.answer.is_empty() && claim.answer != answer)
        {
            return Err(ConnectedAppError::ClaimTransition);
        }
        claim.answer = answer;
        claim.validate()
    }

    pub fn claim_recovery_action(
        &self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        fate: ConnectedAppOperationFate,
        now_ms: i64,
    ) -> Result<ConnectedAppRecoveryAction, ConnectedAppError> {
        let claim = self.claim(session_id, message_id, request_id)?;
        if !claim.answer.is_empty() || claim.decision == ConnectedAppClaimDecision::Answer {
            return Ok(ConnectedAppRecoveryAction::SubmitAnswer);
        }

        let handed_off = !claim.signed_payload.is_empty();
        if fate == ConnectedAppOperationFate::Unknown
            || fate == ConnectedAppOperationFate::Sending
            || (fate == ConnectedAppOperationFate::Unresolved && handed_off)
        {
            return Ok(ConnectedAppRecoveryAction::Wait);
        }

        let executed = fate == ConnectedAppOperationFate::Settled
            || (fate == ConnectedAppOperationFate::Absent && handed_off);
        if executed {
            return Ok(if handed_off {
                ConnectedAppRecoveryAction::RecoverAnswer
            } else {
                ConnectedAppRecoveryAction::Forget
            });
        }
        if fate == ConnectedAppOperationFate::Unresolved || now_ms >= claim.expires_at_ms {
            return Ok(ConnectedAppRecoveryAction::SubmitNotSent);
        }
        Ok(ConnectedAppRecoveryAction::Offer)
    }

    pub fn reconcile_claim_recovery(
        &mut self,
        wallet_identity: Option<WalletTransferIdentity>,
        fates: Vec<ConnectedAppClaimFate>,
        now_ms: i64,
    ) -> Result<(), ConnectedAppError> {
        if now_ms < 0
            || wallet_identity
                .as_ref()
                .is_some_and(|identity| identity.validate().is_err())
        {
            return Err(ConnectedAppError::InvalidWalletBinding);
        }
        self.prune_claims(now_ms);
        let live_ids = self.claims.iter().map(|claim| claim.message_id).collect::<BTreeSet<_>>();
        self.request_runtime.recovery_held.retain(|id| live_ids.contains(id));
        self.request_runtime.recovery_submissions.retain(|id, _| live_ids.contains(id));

        let mut fate_by_claim = BTreeMap::new();
        for item in fates {
            if item.session_id == 0 || item.message_id <= 0 || item.message_id > i32::MAX as i64 {
                return Err(ConnectedAppError::InvalidClaim);
            }
            fate_by_claim.insert((item.session_id, item.message_id), item.fate);
        }

        let records = self.claims.clone();
        for record in &records {
            self.request_runtime.claimed_ids.insert(record.message_id);
        }

        let mut journal_waits = false;
        for record in records {
            if self.request_runtime.recovery_held.contains(&record.message_id)
                || self.request_runtime.owns(record.message_id)
            {
                continue;
            }
            if !record.answer.is_empty() || record.decision == ConnectedAppClaimDecision::Answer {
                self.request_runtime.hold_recovery_submission(
                    &record,
                    ConnectedAppRecoverySubmissionKind::Answer,
                    record.answer.clone(),
                );
                continue;
            }
            let wallet_matches = match (&record.wallet_identity, &wallet_identity) {
                (Some(bound), Some(current)) => {
                    bound.network == current.network
                        && bound.address == current.address
                        && bound.public_key == current.public_key
                }
                _ => false,
            };
            if !wallet_matches {
                continue;
            }
            let fate = if record.operation_id.is_empty() {
                ConnectedAppOperationFate::Absent
            } else {
                fate_by_claim
                    .get(&(record.session_id, record.message_id))
                    .copied()
                    .unwrap_or(ConnectedAppOperationFate::Unknown)
            };
            let action = self.claim_recovery_action(
                record.session_id,
                record.message_id,
                &record.request_id,
                fate,
                now_ms,
            )?;
            match action {
                ConnectedAppRecoveryAction::Wait => {
                    let handed_off = !record.signed_payload.is_empty();
                    journal_waits |= fate == ConnectedAppOperationFate::Sending
                        || (fate == ConnectedAppOperationFate::Unresolved && handed_off);
                }
                ConnectedAppRecoveryAction::Offer => {
                    self.request_runtime
                        .enqueue_recovery(&record, ConnectedAppRequestRecovery::Offer)?;
                }
                ConnectedAppRecoveryAction::RecoverAnswer => {
                    self.request_runtime
                        .enqueue_recovery(&record, ConnectedAppRequestRecovery::Answer)?;
                }
                ConnectedAppRecoveryAction::SubmitAnswer => {
                    self.request_runtime.hold_recovery_submission(
                        &record,
                        ConnectedAppRecoverySubmissionKind::Answer,
                        record.answer.clone(),
                    );
                }
                ConnectedAppRecoveryAction::SubmitNotSent => {
                    self.request_runtime.hold_recovery_submission(
                        &record,
                        ConnectedAppRecoverySubmissionKind::NotSent,
                        record.not_sent.clone(),
                    );
                }
                ConnectedAppRecoveryAction::Forget => {
                    self.claims.retain(|claim| {
                        claim.session_id != record.session_id || claim.message_id != record.message_id
                    });
                }
            }
        }
        self.request_runtime.recovery_polling = journal_waits;
        Ok(())
    }

    pub fn settle_recovery_submission(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        retry_later: bool,
    ) -> Result<(), ConnectedAppError> {
        let submission = self
            .request_runtime
            .recovery_submissions
            .get(&message_id)
            .ok_or(ConnectedAppError::ClaimNotFound)?;
        if submission.session_id != session_id || submission.request_id != request_id {
            return Err(ConnectedAppError::ClaimNotFound);
        }
        if !retry_later {
            self.claims.retain(|claim| {
                claim.session_id != session_id
                    || claim.message_id != message_id
                    || claim.request_id != request_id
            });
            self.request_runtime.clear_recovery_submission(message_id);
        }
        Ok(())
    }

    fn claim(
        &self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
    ) -> Result<&ConnectedAppClaimRecord, ConnectedAppError> {
        self.claims
            .iter()
            .find(|claim| {
                claim.session_id == session_id
                    && claim.message_id == message_id
                    && claim.request_id == request_id
            })
            .ok_or(ConnectedAppError::ClaimNotFound)
    }

    fn claim_mut(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
    ) -> Result<&mut ConnectedAppClaimRecord, ConnectedAppError> {
        self.claims
            .iter_mut()
            .find(|claim| {
                claim.session_id == session_id
                    && claim.message_id == message_id
                    && claim.request_id == request_id
            })
            .ok_or(ConnectedAppError::ClaimNotFound)
    }

    fn record_claim_internal(
        &mut self,
        session_id: u64,
        message_id: i64,
        request_id: &str,
        decision: ConnectedAppClaimDecision,
        wallet_identity: Option<WalletTransferIdentity>,
        operation_id: String,
        signed_payload: String,
        not_sent: Vec<u8>,
        answer: Vec<u8>,
        now_ms: i64,
    ) -> Result<ConnectedAppClaimRecord, ConnectedAppError> {
        if decision == ConnectedAppClaimDecision::Confirm
            && (!operation_id.is_empty() || !signed_payload.is_empty() || !answer.is_empty())
        {
            return Err(ConnectedAppError::ClaimTransition);
        }
        let key = request_key(session_id, message_id);
        let request = self
            .pending_requests
            .get(&key)
            .cloned()
            .ok_or(ConnectedAppError::RequestNotFound)?;
        if request.request_id != request_id {
            return Err(ConnectedAppError::RequestNotFound);
        }
        if request.expires_at_ms <= now_ms {
            self.pending_requests.remove(&key);
            return Err(ConnectedAppError::RequestExpired);
        }
        self.prune_claims(now_ms);
        if self.claims.len() >= CONNECTED_APP_CLAIM_MAX_RECORDS {
            return Err(ConnectedAppError::ClaimCapacity);
        }
        let claim = ConnectedAppClaimRecord {
            session_id,
            message_id: request.message_id,
            request_id: request.request_id.clone(),
            trace_id: request.trace_id.clone(),
            expires_at_ms: request.expires_at_ms,
            decision,
            wallet_identity,
            operation_id,
            signed_payload,
            not_sent,
            answer,
            created_at_ms: now_ms,
        };
        claim.validate()?;
        let mut candidate = self.claims.clone();
        candidate.push(claim.clone());
        if serialized_claim_bytes(&candidate)? > CONNECTED_APP_CLAIM_MAX_BYTES {
            return Err(ConnectedAppError::ClaimCapacity);
        }
        self.claims = candidate;
        self.pending_requests.remove(&key);
        Ok(claim)
    }

    pub fn prune_claims(&mut self, now_ms: i64) {
        self.claims.retain(|claim| {
            claim.validate().is_ok()
                && now_ms
                    <= claim
                        .expires_at_ms
                        .saturating_add(CONNECTED_APP_CLAIM_RETENTION_MS)
        });
        self.pending_requests
            .retain(|_, request| request.validate(now_ms).is_ok());
    }

    pub fn recoverable_claims(&self, now_ms: i64) -> Vec<ConnectedAppClaimRecord> {
        self.claims
            .iter()
            .filter(|claim| {
                claim.validate().is_ok()
                    && now_ms
                        <= claim
                            .expires_at_ms
                            .saturating_add(CONNECTED_APP_CLAIM_RETENTION_MS)
            })
            .cloned()
            .collect()
    }

    pub fn recoverable_claims_for_wallet(
        &self,
        wallet_identity: &WalletTransferIdentity,
        now_ms: i64,
    ) -> Vec<ConnectedAppClaimRecord> {
        self.recoverable_claims(now_ms)
            .into_iter()
            .filter(|claim| {
                claim.wallet_identity.as_ref().is_some_and(|bound| {
                    bound.network == wallet_identity.network
                        && bound.address == wallet_identity.address
                        && bound.public_key == wallet_identity.public_key
                })
            })
            .collect()
    }
}

pub fn connected_app_start_param_query(start_param: &str) -> Option<String> {
    let mut result = start_param.strip_prefix("tonconnect-")?.to_string();
    result = result.replace("--", "%");
    result = result.replace("__", "=");
    result = result.replace('-', "&");
    Some(result)
}

pub fn connected_app_start_param(query: &str) -> String {
    let query = query.split('#').next().unwrap_or_default().replace('+', "%20");
    let mut parts = Vec::new();
    for pair in query.split('&').filter(|part| !part.is_empty()) {
        let part = pair
            .split('%')
            .map(percent_encode_start_param_chunk)
            .collect::<Vec<_>>()
            .join("%");
        if !part.starts_with('%') {
            parts.push(part);
        }
    }
    let mut encoded = parts.join("-");
    encoded = encoded.replace('=', "__");
    encoded = encoded.replace('%', "--");
    format!("tonconnect-{encoded}")
}

fn percent_encode_start_param_chunk(value: &str) -> String {
    let mut result = String::new();
    for byte in value.as_bytes() {
        // Match QUrl::toPercentEncoding(chunk, "=", "-._~") exactly:
        // '=' is explicitly excluded from encoding, while '-._~' are
        // explicitly included so a literal '-' can never alias the SDK's
        // single-dash pair separator during decoding.
        let safe = byte.is_ascii_alphanumeric() || *byte == b'=';
        if safe {
            result.push(*byte as char);
        } else {
            result.push_str(&format!("%{byte:02X}"));
        }
    }
    result
}

pub fn connected_app_display_name(name: &str, fallback: &str) -> String {
    let mut result = String::new();
    let mut units = 0usize;
    let mut previous_space = false;
    for ch in name.chars() {
        let code = ch as u32;
        if matches!(code, 0x200E..=0x200F | 0x202A..=0x202E | 0x2066..=0x2069) {
            continue;
        }
        let ch = if ch.is_whitespace() { ' ' } else { ch };
        if ch == ' ' && previous_space {
            continue;
        }
        let next = ch.len_utf16();
        if units + next > CONNECTED_APP_NAME_MAX_CHARS {
            break;
        }
        result.push(ch);
        units += next;
        previous_space = ch == ' ';
    }
    let result = result.trim();
    if result.is_empty() {
        fallback.to_string()
    } else {
        result.to_string()
    }
}

pub fn connected_app_client_id_valid(value: &str) -> bool {
    value.as_bytes().len() == CONNECTED_APP_CLIENT_ID_HEX_CHARS
        && value.as_bytes().iter().all(u8::is_ascii_hexdigit)
}

pub fn connected_app_trace_id_valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    bytes.iter().enumerate().all(|(index, byte)| {
        if matches!(index, 8 | 13 | 18 | 23) {
            *byte == b'-'
        } else {
            byte.is_ascii_hexdigit()
        }
    })
}

pub fn connected_app_secure_url_host(value: &str) -> Option<String> {
    if value.is_empty()
        || value.trim() != value
        || value.chars().any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return None;
    }
    let scheme_end = value.find("://")?;
    if !value[..scheme_end].eq_ignore_ascii_case("https") {
        return None;
    }
    let rest = &value[(scheme_end + 3)..];
    let authority_end = rest
        .find(|ch| matches!(ch, '/' | '?' | '#'))
        .unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() || authority.contains('@') {
        return None;
    }

    let host = if authority.starts_with('[') {
        let close = authority.find(']')?;
        if close <= 1 {
            return None;
        }
        let suffix = &authority[(close + 1)..];
        if !suffix.is_empty()
            && (!suffix.starts_with(':')
                || suffix.len() == 1
                || !suffix[1..].bytes().all(|byte| byte.is_ascii_digit())
                || suffix[1..].parse::<u16>().is_err())
        {
            return None;
        }
        &authority[1..close]
    } else {
        if authority.matches(':').count() > 1 {
            return None;
        }
        match authority.rsplit_once(':') {
            Some((host, port))
                if !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                if port.parse::<u16>().is_err() {
                    return None;
                }
                host
            }
            Some(_) => return None,
            None => authority,
        }
    };

    if host.is_empty() || host.starts_with('.') || host.ends_with('.') {
        return None;
    }
    Some(host.to_ascii_lowercase())
}

pub fn parse_connected_app_link(query: &str) -> Option<ConnectedAppLink> {
    if query.as_bytes().len() > CONNECTED_APP_QUERY_MAX_BYTES {
        return None;
    }
    let query = query.split('#').next().unwrap_or_default();
    let mut params = BTreeMap::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let name = percent_decode_component(name)?;
        if name.is_empty() || params.contains_key(&name) {
            continue;
        }
        params.insert(name, percent_decode_component(value)?);
    }

    let has_version = params.contains_key("v");
    let has_id = params.contains_key("id");
    let has_request = params.contains_key("r");
    let client_id = params.get("id").cloned().unwrap_or_default();
    if (has_version && params.get("v").map(String::as_str) != Some("2"))
        || (has_id && !connected_app_client_id_valid(&client_id))
        || (has_request && (!has_version || !has_id))
    {
        return None;
    }

    let mut result = ConnectedAppLink {
        kind: if has_request {
            ConnectedAppLinkKind::Connect
        } else if has_id {
            ConnectedAppLinkKind::OpenPending
        } else {
            ConnectedAppLinkKind::Bare
        },
        client_id,
        manifest_url: String::new(),
        proof_payload: None,
        trace_id: String::new(),
        return_target: params.get("ret").cloned().unwrap_or_default(),
    };

    if has_request {
        let request: serde_json::Value =
            serde_json::from_str(params.get("r").map(String::as_str).unwrap_or_default()).ok()?;
        let object = request.as_object()?;
        let manifest_url = object.get("manifestUrl")?.as_str()?.to_string();
        connected_app_secure_url_host(&manifest_url)?;
        let items = object.get("items")?.as_array()?;
        let mut has_address = false;
        for item in items {
            let item = item.as_object()?;
            match item.get("name").and_then(serde_json::Value::as_str) {
                Some("ton_addr") => has_address = true,
                Some("ton_proof") if result.proof_payload.is_none() => {
                    result.proof_payload = Some(item.get("payload")?.as_str()?.to_string());
                }
                _ => {}
            }
        }
        if !has_address {
            return None;
        }
        result.manifest_url = manifest_url;
    }

    if let Some(trace_id) = params.get("trace_id") {
        if connected_app_trace_id_valid(trace_id) {
            result.trace_id = trace_id.clone();
        }
    }
    Some(result)
}

fn percent_decode_component(value: &str) -> Option<String> {
    fn hex(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }

    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            decoded.push((hex(bytes[index + 1])? << 4) | hex(bytes[index + 2])?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

pub fn connected_app_request_id_valid(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= CONNECTED_APP_REQUEST_ID_MAX_CHARS
        && value
            .chars()
            .all(|ch| ('\u{20}'..='\u{7e}').contains(&ch))
}

pub fn connected_app_proof_domain_allowed(
    domain: &str,
    ownership_domain: &str,
    reserved_platform_domain: &str,
) -> bool {
    let normalized = normalize_domain(domain);
    let ownership = normalize_domain(ownership_domain);
    let reserved = normalize_domain(reserved_platform_domain);
    !normalized.is_empty()
        && (reserved.is_empty() || normalized != reserved)
        && (ownership.is_empty() || normalized != ownership)
}

fn request_key(session_id: u64, message_id: i64) -> String {
    format!("{session_id}:{message_id}")
}

fn normalize_domain(value: &str) -> String {
    value
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

fn valid_secure_url(value: &str) -> bool {
    connected_app_secure_url_host(value).is_some()
}

fn valid_session_transition(
    from: ConnectedAppSessionStatus,
    to: ConnectedAppSessionStatus,
) -> bool {
    use ConnectedAppSessionStatus::*;
    matches!(
        (from, to),
        (Pending, Pending)
            | (Pending, Active)
            | (Pending, Closing)
            | (Pending, Closed)
            | (Active, Active)
            | (Active, Closing)
            | (Active, Closed)
            | (Closing, Closing)
            | (Closing, Closed)
            | (Closed, Closed)
    )
}

fn serialized_claim_bytes(
    claims: &[ConnectedAppClaimRecord],
) -> Result<usize, ConnectedAppError> {
    serde_json::to_vec(claims)
        .map(|bytes| bytes.len())
        .map_err(|_| ConnectedAppError::InvalidClaim)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConnectedAppError {
    #[error("connected app manifest is invalid")]
    InvalidManifest,
    #[error("connected app session is invalid")]
    InvalidSession,
    #[error("connected app session transition is invalid")]
    InvalidSessionTransition,
    #[error("connected app session refresh timestamp is invalid")]
    InvalidRefreshTimestamp,
    #[error("connected app session refresh generation overflowed")]
    RefreshGenerationOverflow,
    #[error("connected app session refresh is stale: current {current}, received {received}")]
    StaleSessionRefresh { current: u64, received: u64 },
    #[error("connected app session refresh contains a duplicate session")]
    DuplicateSessionInRefresh,
    #[error("connected app session was not found")]
    SessionNotFound,
    #[error("connected app session is closed")]
    SessionClosed,
    #[error("connected app mutation is stale")]
    StaleMutation,
    #[error("connected app request is invalid")]
    InvalidRequest,
    #[error("connected app request already exists or was already claimed")]
    DuplicateRequest,
    #[error("connected app request runtime is stopped")]
    RequestRuntimeStopped,
    #[error("connected app request order overflowed")]
    RequestOrderOverflow,
    #[error("connected app request is owned by the silent flow")]
    RequestOwnedSilently,
    #[error("connected app request was not found")]
    RequestNotFound,
    #[error("connected app request expired")]
    RequestExpired,
    #[error("connected app claim is invalid")]
    InvalidClaim,
    #[error("connected app confirm claim is missing its durable recovery fallback")]
    MissingRecoveryFallback,
    #[error("connected app wallet binding is invalid")]
    InvalidWalletBinding,
    #[error("connected app claim store capacity was exceeded")]
    ClaimCapacity,
    #[error("connected app wallet address is invalid")]
    InvalidWalletAddress,
    #[error("connected app key-reference epoch overflowed")]
    KeyEpochOverflow,
    #[error("connected app key reference is invalid")]
    InvalidKeyReference,
    #[error("connected app connect link is invalid")]
    InvalidConnectLink,
    #[error("connected app proof domain is reserved or ownership-confused")]
    ProofDomainRejected,
    #[error("connected app connect flow was not found")]
    ConnectFlowNotFound,
    #[error("connected app connect flow is in an invalid phase")]
    InvalidConnectPhase,
    #[error("connected app wallet resolve timed out")]
    ConnectResolveTimedOut,
    #[error("connected app access is blocked: {0:?}")]
    AccessBlocked(ConnectedAppAccess),
    #[error("connected app manifest polling was exhausted")]
    ManifestPollExhausted,
    #[error("connected app manifest host does not match the requested host")]
    ManifestHostMismatch,
    #[error("connected app wallet address changed during authorization")]
    WalletAddressChanged,
    #[error("connected app durable claim transition is invalid")]
    ClaimTransition,
    #[error("connected app durable claim was not found")]
    ClaimNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(status: ConnectedAppSessionStatus) -> ConnectedAppSession {
        ConnectedAppSession {
            id: 7,
            client_id: "client-7".into(),
            manifest: Some(ConnectedAppManifest {
                url: "https://example.test/manifest.json".into(),
                name: "Example".into(),
                icon_url: Some("https://example.test/icon.png".into()),
            }),
            status,
            created_at_ms: 10,
            updated_at_ms: 10,
        }
    }

    fn request(id: &str, expires_at_ms: i64) -> ConnectedAppRequest {
        ConnectedAppRequest {
            session_id: 7,
            message_id: 101,
            request_id: id.into(),
            method: "sendTransaction".into(),
            kind: ConnectedAppRequestKind::SendTransaction,
            trace_id: "trace".into(),
            expires_at_ms,
        }
    }

    #[test]
    fn request_ids_are_ascii_printable_and_bounded() {
        assert!(connected_app_request_id_valid("request-1"));
        assert!(!connected_app_request_id_valid(""));
        assert!(!connected_app_request_id_valid("bad\nrequest"));
        assert!(!connected_app_request_id_valid(&"a".repeat(101)));
    }

    #[test]
    fn proof_domain_normalization_rejects_reserved_and_ownership_domains() {
        assert!(!connected_app_proof_domain_allowed(
            "FABUSHI.EXAMPLE.",
            "",
            "fabushi.example"
        ));
        assert!(!connected_app_proof_domain_allowed(
            "wallet.example.",
            "WALLET.EXAMPLE",
            "fabushi.example"
        ));
        assert!(connected_app_proof_domain_allowed(
            "app.example.",
            "wallet.example",
            "fabushi.example"
        ));
    }

    #[test]
    fn session_state_machine_never_reopens_closed_session() {
        let mut state = ConnectedAppState::default();
        state
            .upsert_session(session(ConnectedAppSessionStatus::Pending))
            .unwrap();
        let mut active = session(ConnectedAppSessionStatus::Active);
        active.updated_at_ms = 11;
        state.upsert_session(active).unwrap();
        state.close_session(7, 12).unwrap();
        let mut reopen = session(ConnectedAppSessionStatus::Active);
        reopen.updated_at_ms = 13;
        assert_eq!(
            state.upsert_session(reopen),
            Err(ConnectedAppError::InvalidSessionTransition)
        );
    }

    #[test]
    fn request_claim_is_durable_bounded_and_exactly_once() {
        let mut state = ConnectedAppState::default();
        let mut active = session(ConnectedAppSessionStatus::Active);
        active.updated_at_ms = 11;
        state.upsert_session(active).unwrap();
        state.queue_request(request("r1", 1_000), 100).unwrap();
        let claim = state
            .record_claim_with_recovery(
                7,
                 101,
                "r1",
                ConnectedAppClaimDecision::Confirm,
                String::new(),
                String::new(),
                vec![9],
                Vec::new(),
                200,
            )
            .unwrap();
        state.link_claim_operation(7, 101, "r1", "operation-1").unwrap();
        state
            .record_claim_handoff(7, 101, "r1", "operation-1", "signed-payload")
            .unwrap();
        state.record_claim_answer(7, 101, "r1", vec![1, 2, 3]).unwrap();
        assert_eq!(claim.trace_id, "trace");
        assert!(state.pending_requests.is_empty());
        assert_eq!(state.claims.len(), 1);
        assert_eq!(
            state.record_claim_with_recovery(
                7,
                 101,
                "r1",
                ConnectedAppClaimDecision::Confirm,
                String::new(),
                String::new(),
                vec![9],
                Vec::new(),
                201,
            ),
            Err(ConnectedAppError::RequestNotFound)
        );
    }

    #[test]
    fn expired_request_is_never_claimed_and_recovery_prunes_after_retention() {
        let mut state = ConnectedAppState::default();
        let mut active = session(ConnectedAppSessionStatus::Active);
        active.updated_at_ms = 11;
        state.upsert_session(active).unwrap();
        state.queue_request(request("late", 500), 100).unwrap();
        assert_eq!(
            state.record_claim(
                7,
                 101,
                "late",
                ConnectedAppClaimDecision::Answer,
                String::new(),
                String::new(),
                Vec::new(),
                500,
            ),
            Err(ConnectedAppError::RequestExpired)
        );
        let mut recover = request("recover", 1_000);
        recover.message_id = 102;
        state.queue_request(recover, 600).unwrap();
        state
            .record_claim(
                7,
                 102,
                "recover",
                ConnectedAppClaimDecision::Answer,
                String::new(),
                String::new(),
                vec![7],
                700,
            )
            .unwrap();
        assert_eq!(state.recoverable_claims(1_000).len(), 1);
        state.prune_claims(1_000 + CONNECTED_APP_CLAIM_RETENTION_MS + 1);
        assert!(state.claims.is_empty());
    }

    #[test]
    fn wallet_bound_claim_recovery_requires_exact_wallet_address_and_public_key() {
        let mut state = ConnectedAppState::default();
        let mut active = session(ConnectedAppSessionStatus::Active);
        active.updated_at_ms = 11;
        state.upsert_session(active).unwrap();
        state.queue_request(request("wallet", 1_000), 100).unwrap();
        let wallet = WalletTransferIdentity {
            network: 1,
            address: "EQ-wallet".into(),
            public_key: vec![4; 32],
            revision: 9,
        };
        state.record_wallet_claim_with_recovery(
            7,
             101,
            "wallet",
            ConnectedAppClaimDecision::Confirm,
            wallet.clone(),
            String::new(),
            String::new(),
            vec![2],
            Vec::new(),
            200,
        ).unwrap();
        state
            .link_claim_operation(7, 101, "wallet", "operation-wallet")
            .unwrap();
        state
            .record_claim_handoff(7, 101, "wallet", "operation-wallet", "signed")
            .unwrap();
        assert_eq!(state.recoverable_claims_for_wallet(&wallet, 500).len(), 1);
        let rotated_revision = WalletTransferIdentity { revision: 10, ..wallet.clone() };
        assert_eq!(state.recoverable_claims_for_wallet(&rotated_revision, 500).len(), 1);
        let another_key = WalletTransferIdentity { public_key: vec![5; 32], ..wallet };
        assert!(state.recoverable_claims_for_wallet(&another_key, 500).is_empty());
    }

    #[test]
    fn session_refresh_replays_local_changes_over_remote_snapshot() {
        let mut state = ConnectedAppState::default();
        let mut first = session(ConnectedAppSessionStatus::Active);
        first.updated_at_ms = 10;
        state.upsert_session(first.clone()).unwrap();
        let generation = state.begin_session_refresh(100, true).unwrap().unwrap();
        state.mark_session_closing(7, 120).unwrap();
        let mut stale = first;
        stale.updated_at_ms = 110;
        state.apply_session_refresh(generation, vec![stale], 130).unwrap();
        assert_eq!(state.sessions.get(&7).unwrap().status, ConnectedAppSessionStatus::Closing);
        assert_eq!(state.closing_session_ids(), vec![7]);
        assert!(!state.session_refresh.in_flight);
    }

    #[test]
    fn session_refresh_cannot_resurrect_a_closed_tombstone() {
        let mut state = ConnectedAppState::default();
        let active = session(ConnectedAppSessionStatus::Active);
        state.upsert_session(active.clone()).unwrap();
        state.close_session(7, 20).unwrap();
        let generation = state.begin_session_refresh(100, true).unwrap().unwrap();
        let mut stale = active;
        stale.updated_at_ms = 30;
        state.apply_session_refresh(generation, vec![stale], 110).unwrap();
        assert!(!state.sessions.contains_key(&7));
        assert!(state.closed_session_ids.contains(&7));
    }

    #[test]
    fn session_refresh_is_ready_gated_rate_limited_and_stale_safe() {
        let mut state = ConnectedAppState::default();
        assert_eq!(state.begin_session_refresh(100, false).unwrap(), None);
        let first = state.begin_session_refresh(100, true).unwrap().unwrap();
        assert_eq!(state.begin_session_refresh(101, true).unwrap(), None);
        state.fail_session_refresh(first).unwrap();
        assert_eq!(state.begin_session_refresh(100 + CONNECTED_APP_SESSION_REFRESH_FLOOR_MS - 1, true).unwrap(), None);
        let second = state.begin_session_refresh(100 + CONNECTED_APP_SESSION_REFRESH_FLOOR_MS, true).unwrap().unwrap();
        assert_eq!(
            state.apply_session_refresh(first, Vec::new(), 200),
            Err(ConnectedAppError::StaleSessionRefresh { current: second, received: first })
        );
    }

    #[test]
    fn session_gone_is_idempotent_and_removes_active_map_entry() {
        let mut state = ConnectedAppState::default();
        state.upsert_session(session(ConnectedAppSessionStatus::Active)).unwrap();
        state.mark_session_closing(7, 20).unwrap();
        state.acknowledge_session_gone(7, 21).unwrap();
        assert!(!state.sessions.contains_key(&7));
        state.acknowledge_session_gone(7, 22).unwrap();
    }

    #[test]
    fn durable_claim_shape_fails_closed_for_recovery_unsafe_combinations() {
        let base = ConnectedAppClaimRecord {
            session_id: 7,
            message_id: 101,
            request_id: "claim-shape".into(),
            trace_id: "trace".into(),
            expires_at_ms: 1_000,
            decision: ConnectedAppClaimDecision::Confirm,
            wallet_identity: None,
            operation_id: String::new(),
            signed_payload: String::new(),
            not_sent: vec![7],
            answer: Vec::new(),
            created_at_ms: 100,
        };
        assert_eq!(
            ConnectedAppClaimRecord {
                not_sent: Vec::new(),
                ..base.clone()
            }
            .validate(),
            Err(ConnectedAppError::InvalidClaim)
        );
        assert_eq!(
            ConnectedAppClaimRecord {
                decision: ConnectedAppClaimDecision::Answer,
                operation_id: "operation".into(),
                not_sent: Vec::new(),
                answer: vec![1],
                ..base.clone()
            }
            .validate(),
            Err(ConnectedAppError::InvalidClaim)
        );
        assert_eq!(
            ConnectedAppClaimRecord {
                decision: ConnectedAppClaimDecision::Answer,
                signed_payload: "signed".into(),
                not_sent: Vec::new(),
                answer: vec![1],
                ..base.clone()
            }
            .validate(),
            Err(ConnectedAppError::InvalidClaim)
        );
        assert_eq!(
            ConnectedAppClaimRecord {
                created_at_ms: 0,
                ..base.clone()
            }
            .validate(),
            Err(ConnectedAppError::InvalidClaim)
        );
        ConnectedAppClaimRecord {
            decision: ConnectedAppClaimDecision::Answer,
            not_sent: Vec::new(),
            answer: vec![1],
            ..base
        }
        .validate()
        .unwrap();
    }

    #[test]
    fn persisted_invalid_claims_are_pruned_and_never_recovered() {
        let mut state = ConnectedAppState::default();
        state.claims.push(ConnectedAppClaimRecord {
            session_id: 7,
            message_id: 101,
            request_id: "persisted-invalid".into(),
            trace_id: String::new(),
            expires_at_ms: 1_000,
            decision: ConnectedAppClaimDecision::Confirm,
            wallet_identity: None,
            operation_id: String::new(),
            signed_payload: String::new(),
            not_sent: Vec::new(),
            answer: vec![1],
            created_at_ms: 100,
        });
        assert!(state.recoverable_claims(200).is_empty());
        state.prune_claims(200);
        assert!(state.claims.is_empty());
    }

    #[test]
    fn transport_message_identity_allows_protocol_id_reuse_without_claim_collision() {
        let mut state = ConnectedAppState::default();
        let mut active = session(ConnectedAppSessionStatus::Active);
        active.updated_at_ms = 11;
        state.upsert_session(active).unwrap();
        let mut first = request("same-protocol-id", 1_000);
        first.message_id = 501;
        let mut second = first.clone();
        second.message_id = 502;
        state.queue_request(first, 100).unwrap();
        state.queue_request(second, 100).unwrap();
        state
            .record_claim(
                7,
                501,
                "same-protocol-id",
                ConnectedAppClaimDecision::Answer,
                String::new(),
                String::new(),
                vec![1],
                200,
            )
            .unwrap();
        state
            .record_claim(
                7,
                502,
                "same-protocol-id",
                ConnectedAppClaimDecision::Answer,
                String::new(),
                String::new(),
                vec![2],
                200,
            )
            .unwrap();
        assert_eq!(state.claims.len(), 2);
        assert_eq!(state.claims[0].message_id, 501);
        assert_eq!(state.claims[1].message_id, 502);
    }

    #[test]
    fn legacy_claim_without_message_identity_fails_closed() {
        let mut claim = ConnectedAppClaimRecord {
            session_id: 7,
            message_id: 0,
            request_id: "legacy".into(),
            trace_id: String::new(),
            expires_at_ms: 1_000,
            decision: ConnectedAppClaimDecision::Answer,
            wallet_identity: None,
            operation_id: String::new(),
            signed_payload: String::new(),
            not_sent: Vec::new(),
            answer: vec![1],
            created_at_ms: 100,
        };
        assert_eq!(claim.validate(), Err(ConnectedAppError::InvalidClaim));
        claim.message_id = 1;
        claim.validate().unwrap();
    }

    #[test]
    fn start_param_codec_preserves_sdk_separator_contract() {
        let query = "v=2&id=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa&trace_id=12345678-1234-1234-1234-123456789abc";
        let encoded = connected_app_start_param(query);
        assert_eq!(
            connected_app_start_param_query(&encoded).as_deref(),
            Some("v=2&id=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa&trace%5Fid=12345678%2D1234%2D1234%2D1234%2D123456789abc")
        );
        let unsafe_leading_escape = connected_app_start_param("%7Bignored&v=2");
        assert_eq!(
            connected_app_start_param_query(&unsafe_leading_escape).as_deref(),
            Some("v=2")
        );
        assert!(connected_app_start_param_query("other-v=2").is_none());

        let reserved = "ret=a-b.c_d~e";
        let reserved_encoded = connected_app_start_param(reserved);
        assert!(reserved_encoded.contains("--2D"));
        assert!(reserved_encoded.contains("--2E"));
        assert!(reserved_encoded.contains("--5F"));
        assert!(reserved_encoded.contains("--7E"));
        assert_eq!(
            connected_app_start_param_query(&reserved_encoded).as_deref(),
            Some("ret=a%2Db%2Ec%5Fd%7Ee")
        );
    }

    #[test]
    fn display_name_is_single_line_bidi_safe_utf16_bounded_and_falls_back() {
        let hostile = format!("  hello\n\u{202E}world {}  ", "\u{1F600}".repeat(40));
        let display = connected_app_display_name(&hostile, "fallback.example");
        assert!(!display.contains('\n'));
        assert!(!display.contains('\u{202E}'));
        assert!(display.encode_utf16().count() <= CONNECTED_APP_NAME_MAX_CHARS);
        assert_eq!(
            connected_app_display_name("\u{202E}\n", "fallback.example"),
            "fallback.example"
        );
        let manifest = ConnectedAppManifest {
            url: "https://fallback.example/manifest.json".into(),
            name: "\u{202E}\n".into(),
            icon_url: None,
        };
        manifest.validate().unwrap();
    }

    #[test]
    fn signed_payload_requires_operation_id_and_manifest_requires_https() {
        let mut state = ConnectedAppState::default();
        let mut active = session(ConnectedAppSessionStatus::Active);
        active.updated_at_ms = 11;
        state.upsert_session(active).unwrap();
        state.queue_request(request("r2", 1_000), 100).unwrap();
        assert_eq!(
            state.record_claim_with_recovery(
                7,
                 101,
                "r2",
                ConnectedAppClaimDecision::Confirm,
                String::new(),
                "signed".into(),
                vec![8],
                Vec::new(),
                200,
            ),
            Err(ConnectedAppError::ClaimTransition)
        );
        let mut insecure = session(ConnectedAppSessionStatus::Pending);
        insecure.id = 8;
        insecure.client_id = "client-8".into();
        insecure.manifest.as_mut().unwrap().url = "http://example.test/manifest".into();
        assert_eq!(
            state.upsert_session(insecure),
            Err(ConnectedAppError::InvalidManifest)
        );
    }
}

#[cfg(test)]
mod lifecycle_source_tests {
    use super::*;

    fn source_session(status: ConnectedAppSessionStatus) -> ConnectedAppSession {
        ConnectedAppSession {
            id: 77,
            client_id: "client-77".into(),
            manifest: Some(ConnectedAppManifest {
                url: "https://dapp.example/manifest.json".into(),
                name: "Dapp".into(),
                icon_url: None,
            }),
            status,
            created_at_ms: 10,
            updated_at_ms: 10,
        }
    }

    #[test]
    fn link_parser_enforces_client_trace_manifest_and_userinfo_boundaries() {
        let client = "a".repeat(CONNECTED_APP_CLIENT_ID_HEX_CHARS);
        let trace = "12345678-1234-1234-1234-123456789abc";
        let request = serde_json::json!({
            "manifestUrl": "https://dapp.example/manifest.json",
            "items": [
                { "name": "ton_addr" },
                { "name": "ton_proof", "payload": "proof-me" }
            ]
        })
        .to_string();
        let encoded = request
            .bytes()
            .map(|byte| format!("%{:02X}", byte))
            .collect::<String>();
        let link = parse_connected_app_link(&format!(
            "v=2&id={client}&r={encoded}&trace_id={trace}"
        ))
        .unwrap();

        assert_eq!(link.kind, ConnectedAppLinkKind::Connect);
        assert_eq!(link.manifest_url, "https://dapp.example/manifest.json");
        assert_eq!(link.proof_payload.as_deref(), Some("proof-me"));
        assert_eq!(link.trace_id, trace);
        assert!(connected_app_secure_url_host("https://user@dapp.example/m").is_none());
        assert!(!connected_app_client_id_valid("not-64-hex"));
        assert!(!connected_app_trace_id_valid("trace"));
    }

    #[test]
    fn key_references_survive_rotation_but_not_wallet_address_epoch() {
        let mut state = ConnectedAppState::default();
        state
            .upsert_session(source_session(ConnectedAppSessionStatus::Active))
            .unwrap();
        assert!(state.update_wallet_address("EQ-one").unwrap());
        state
            .cache_key_reference(77, "client-77", "vault-ref-77")
            .unwrap();

        assert_eq!(
            state
                .silent_key_plan(77, ConnectedAppAccess::Busy, false)
                .unwrap(),
            ConnectedAppKeyPlan::Cached {
                reference_id: "vault-ref-77".into()
            }
        );
        assert!(!state.update_wallet_address("EQ-one").unwrap());
        assert!(state.key_runtime.references.contains_key(&77));
        assert!(state.update_wallet_address("EQ-two").unwrap());
        assert!(state.key_runtime.references.is_empty());
        assert_eq!(
            state.silent_key_plan(77, ConnectedAppAccess::Allowed, false),
            Err(ConnectedAppKeyError::Locked)
        );
    }

    #[test]
    fn closing_disconnect_waits_for_vault_then_retries_once_unblocked() {
        let mut state = ConnectedAppState::default();
        state
            .upsert_session(source_session(ConnectedAppSessionStatus::Active))
            .unwrap();
        state.mark_session_closing(77, 20).unwrap();
        assert!(state.begin_disconnect(77).unwrap());
        assert!(state
            .defer_closing_for_key_error(77, ConnectedAppKeyError::Locked)
            .unwrap());
        assert!(!state.disconnect_runtime.disconnecting.contains(&77));
        assert_eq!(state.take_close_waiting_after_vault_change(), vec![77]);
        assert!(state.disconnect_runtime.close_waiting.is_empty());
    }

    #[test]
    fn connect_flow_fences_manifest_wallet_epoch_and_single_retry() {
        let client = "b".repeat(CONNECTED_APP_CLIENT_ID_HEX_CHARS);
        let mut state = ConnectedAppState::default();
        state
            .begin_connect_flow(
                ConnectedAppLink {
                    kind: ConnectedAppLinkKind::Connect,
                    client_id: client.clone(),
                    manifest_url: "https://dapp.example/manifest.json".into(),
                    proof_payload: None,
                    trace_id: String::new(),
                    return_target: String::new(),
                },
                "",
                "platform.example",
                100,
            )
            .unwrap();
        assert_eq!(
            state
                .resolve_connect_access(&client, ConnectedAppAccess::Allowed, 110)
                .unwrap(),
            ConnectedAppConnectPhase::LoadingManifest
        );
        state
            .accept_connect_manifest(
                &client,
                ConnectedAppManifest {
                    url: "https://dapp.example/manifest.json".into(),
                    name: "Dapp".into(),
                    icon_url: None,
                },
            )
            .unwrap();
        state.pin_connect_wallet(&client, "EQ-one").unwrap();
        assert!(state.ensure_connect_wallet_unchanged(&client, "EQ-two").is_err());
        assert!(state
            .retry_connect_submit_failure(&client, "TONCONNECT_CHALLENGE_INVALID")
            .unwrap());
        assert!(!state
            .retry_connect_submit_failure(&client, "TONCONNECT_SESSION_NOT_ACTIVE")
            .unwrap());
    }

    #[test]
    fn durable_confirm_claim_advances_before_handoff_and_recovers_by_fate() {
        let mut state = ConnectedAppState::default();
        state
            .upsert_session(source_session(ConnectedAppSessionStatus::Active))
            .unwrap();
        state
            .queue_request(
                ConnectedAppRequest {
                    session_id: 77,
                    message_id: 701,
                    request_id: "request-stage".into(),
                    method: "sendTransaction".into(),
                    kind: ConnectedAppRequestKind::SendTransaction,
                    trace_id: String::new(),
                    expires_at_ms: 10_000,
                },
                100,
            )
            .unwrap();
        state
            .record_claim_with_recovery(
                77,
                 701,
                "request-stage",
                ConnectedAppClaimDecision::Confirm,
                String::new(),
                String::new(),
                vec![9],
                Vec::new(),
                200,
            )
            .unwrap();

        assert_eq!(
            state
                .claim_recovery_action(
                    77,
                     701,
                    "request-stage",
                    ConnectedAppOperationFate::Absent,
                    300,
                )
                .unwrap(),
            ConnectedAppRecoveryAction::Offer
        );

        state
            .link_claim_operation(77, 701, "request-stage", "operation-1")
            .unwrap();
        assert_eq!(
            state
                .claim_recovery_action(
                    77,
                     701,
                    "request-stage",
                    ConnectedAppOperationFate::Unresolved,
                    300,
                )
                .unwrap(),
            ConnectedAppRecoveryAction::SubmitNotSent
        );

        state
            .record_claim_handoff(77, 701, "request-stage", "operation-1", "signed-payload")
            .unwrap();
        assert_eq!(
            state
                .claim_recovery_action(
                    77,
                     701,
                    "request-stage",
                    ConnectedAppOperationFate::Unresolved,
                    300,
                )
                .unwrap(),
            ConnectedAppRecoveryAction::Wait
        );
        assert_eq!(
            state
                .claim_recovery_action(
                    77,
                     701,
                    "request-stage",
                    ConnectedAppOperationFate::Settled,
                    300,
                )
                .unwrap(),
            ConnectedAppRecoveryAction::RecoverAnswer
        );

        state
            .record_claim_answer(77, 701, "request-stage", vec![1, 2, 3])
            .unwrap();
        assert_eq!(
            state
                .claim_recovery_action(
                    77,
                     701,
                    "request-stage",
                    ConnectedAppOperationFate::Settled,
                    300,
                )
                .unwrap(),
            ConnectedAppRecoveryAction::SubmitAnswer
        );
    }
}

#[cfg(test)]
mod connected_app_request_runtime_source_tests {
    use super::*;

    fn session() -> ConnectedAppSession {
        ConnectedAppSession {
            id: 77,
            client_id: "runtime-client".into(),
            manifest: None,
            status: ConnectedAppSessionStatus::Active,
            created_at_ms: 1,
            updated_at_ms: 1,
        }
    }

    fn request(message_id: i64, request_id: &str, kind: ConnectedAppRequestKind) -> ConnectedAppRequest {
        ConnectedAppRequest {
            session_id: 77,
            message_id,
            request_id: request_id.into(),
            method: match kind {
                ConnectedAppRequestKind::Disconnect => "disconnect",
                _ => "sendTransaction",
            }.into(),
            kind,
            trace_id: String::new(),
            expires_at_ms: 10_000,
        }
    }

    #[test]
    fn request_runtime_fences_transport_ownership_and_orders_same_session_by_message_id() {
        let mut state = ConnectedAppState::default();
        state.upsert_session(session()).unwrap();
        state.queue_request(request(300, "r300", ConnectedAppRequestKind::SendTransaction), 10).unwrap();
        state.queue_request(request(200, "r200", ConnectedAppRequestKind::SendTransaction), 10).unwrap();

        let selected = state.take_next_request(20).unwrap().unwrap();
        assert_eq!(selected.message_id, 200);
        assert_eq!(state.request_runtime.active.as_ref().map(|entry| entry.message_id), Some(200));

        state.finish_request(200, false).unwrap();
        let selected = state.take_next_request(21).unwrap().unwrap();
        assert_eq!(selected.message_id, 300);
    }

    #[test]
    fn chosen_request_moves_ahead_without_preempting_active_and_disconnect_is_silent_owned() {
        let mut state = ConnectedAppState::default();
        state.upsert_session(session()).unwrap();
        state.queue_request(request(100, "r100", ConnectedAppRequestKind::SendTransaction), 10).unwrap();
        state.queue_request(request(200, "r200", ConnectedAppRequestKind::SendTransaction), 10).unwrap();
        state.queue_request(request(300, "r300", ConnectedAppRequestKind::Disconnect), 10).unwrap();

        assert_eq!(state.take_next_request(20).unwrap().unwrap().message_id, 100);
        state.choose_request(200).unwrap();
        assert_eq!(state.request_runtime.active.as_ref().unwrap().message_id, 100);
        assert_eq!(state.request_runtime.waiting.first().unwrap().message_id, 200);
        assert!(state.request_runtime.silent.contains_key(&300));
        assert_eq!(state.choose_request(300), Err(ConnectedAppError::RequestOwnedSilently));

        state.finish_request(100, false).unwrap();
        assert_eq!(state.take_next_request(21).unwrap().unwrap().message_id, 200);
    }

    #[test]
    fn handled_elsewhere_drops_unclaimed_flow_and_runtime_state_is_not_persisted() {
        let mut state = ConnectedAppState::default();
        state.upsert_session(session()).unwrap();
        state.queue_request(request(400, "r400", ConnectedAppRequestKind::SendTransaction), 10).unwrap();
        assert_eq!(state.take_next_request(20).unwrap().unwrap().message_id, 400);
        assert!(state.mark_request_handled_elsewhere(400));
        assert!(state.request_runtime.active.is_none());
        assert!(state.pending_requests.is_empty());

        state.queue_request(request(500, "r500", ConnectedAppRequestKind::SendTransaction), 10).unwrap();
        assert_eq!(state.take_next_request(20).unwrap().unwrap().message_id, 500);
        let encoded = serde_json::to_string(&state).unwrap();
        let restored: ConnectedAppState = serde_json::from_str(&encoded).unwrap();
        assert!(restored.request_runtime.active.is_none());
        assert!(restored.request_runtime.waiting.is_empty());
        assert!(restored.request_runtime.silent.is_empty());
        assert_eq!(restored.pending_requests.len(), 1);
    }

    #[test]
    fn claimed_finish_fences_reappearance_and_stop_cancels_runtime_ownership() {
        let mut state = ConnectedAppState::default();
        state.upsert_session(session()).unwrap();
        state.queue_request(request(600, "r600", ConnectedAppRequestKind::Disconnect), 10).unwrap();
        state.finish_request(600, true).unwrap();
        assert!(state.request_runtime.claimed_ids.contains(&600));
        assert!(state.pending_requests.is_empty());

        state.queue_request(request(700, "r700", ConnectedAppRequestKind::SendTransaction), 10).unwrap();
        state.stop_requests();
        assert!(state.request_runtime.stopped);
        assert!(state.request_runtime.waiting.is_empty());
        assert_eq!(
            state.queue_request(request(800, "r800", ConnectedAppRequestKind::SendTransaction), 10),
            Err(ConnectedAppError::RequestRuntimeStopped)
        );
    }
}



#[cfg(test)]
mod connected_app_claim_recovery_scheduler_source_tests {
    use super::*;

    fn session() -> ConnectedAppSession {
        ConnectedAppSession {
            id: 771,
            client_id: "recover-client".into(),
            manifest: Some(ConnectedAppManifest {
                url: "https://recover.example/manifest.json".into(),
                name: "Recover".into(),
                icon_url: None,
            }),
            status: ConnectedAppSessionStatus::Active,
            created_at_ms: 10,
            updated_at_ms: 10,
        }
    }

    fn wallet(byte: u8) -> WalletTransferIdentity {
        WalletTransferIdentity {
            network: 1,
            address: "EQ-recover".into(),
            public_key: vec![byte; 32],
            revision: 1,
        }
    }

    fn request(message_id: i64, request_id: &str) -> ConnectedAppRequest {
        ConnectedAppRequest {
            session_id: 771,
            message_id,
            request_id: request_id.into(),
            method: "sendTransaction".into(),
            kind: ConnectedAppRequestKind::SendTransaction,
            trace_id: format!("trace-{message_id}"),
            expires_at_ms: 10_000,
        }
    }

    fn confirm_claim(
        state: &mut ConnectedAppState,
        message_id: i64,
        request_id: &str,
        identity: WalletTransferIdentity,
    ) {
        state.queue_request(request(message_id, request_id), 100).unwrap();
        state
            .record_wallet_claim_with_recovery(
                771,
                message_id,
                request_id,
                ConnectedAppClaimDecision::Confirm,
                identity,
                String::new(),
                String::new(),
                vec![9, message_id as u8],
                Vec::new(),
                200,
            )
            .unwrap();
    }

    #[test]
    fn durable_claims_rebuild_offer_not_sent_and_polling_runtime() {
        let identity = wallet(7);
        let mut offer = ConnectedAppState::default();
        offer.upsert_session(session()).unwrap();
        confirm_claim(&mut offer, 101, "offer", identity.clone());
        let encoded = serde_json::to_string(&offer).unwrap();
        let mut offer: ConnectedAppState = serde_json::from_str(&encoded).unwrap();
        assert!(offer.request_runtime.waiting.is_empty());
        offer.reconcile_claim_recovery(Some(identity.clone()), Vec::new(), 300).unwrap();
        assert!(offer.request_runtime.claimed_ids.contains(&101));
        assert_eq!(
            offer.request_runtime.waiting.first().map(|entry| entry.recovery),
            Some(ConnectedAppRequestRecovery::Offer)
        );

        let mut not_sent = ConnectedAppState::default();
        not_sent.upsert_session(session()).unwrap();
        confirm_claim(&mut not_sent, 102, "not-sent", identity.clone());
        not_sent.link_claim_operation(771, 102, "not-sent", "operation-102").unwrap();
        not_sent
            .reconcile_claim_recovery(
                Some(identity.clone()),
                vec![ConnectedAppClaimFate {
                    session_id: 771,
                    message_id: 102,
                    fate: ConnectedAppOperationFate::Unresolved,
                }],
                300,
            )
            .unwrap();
        assert_eq!(
            not_sent.request_runtime.recovery_submissions[&102].kind,
            ConnectedAppRecoverySubmissionKind::NotSent
        );
        not_sent.settle_recovery_submission(771, 102, "not-sent", true).unwrap();
        assert_eq!(not_sent.claims.len(), 1);
        not_sent.settle_recovery_submission(771, 102, "not-sent", false).unwrap();
        assert!(not_sent.claims.is_empty());

        let mut waiting = ConnectedAppState::default();
        waiting.upsert_session(session()).unwrap();
        confirm_claim(&mut waiting, 103, "sending", identity.clone());
        waiting.link_claim_operation(771, 103, "sending", "operation-103").unwrap();
        waiting
            .record_claim_handoff(771, 103, "sending", "operation-103", "signed-payload")
            .unwrap();
        waiting
            .reconcile_claim_recovery(
                Some(identity),
                vec![ConnectedAppClaimFate {
                    session_id: 771,
                    message_id: 103,
                    fate: ConnectedAppOperationFate::Sending,
                }],
                300,
            )
            .unwrap();
        assert!(waiting.request_runtime.recovery_polling);
        assert!(waiting.request_runtime.waiting.is_empty());
        assert!(waiting.request_runtime.recovery_submissions.is_empty());
    }

    #[test]
    fn recovery_is_wallet_fenced_and_answer_claims_submit_without_wallet() {
        let mut state = ConnectedAppState::default();
        state.upsert_session(session()).unwrap();
        confirm_claim(&mut state, 201, "wallet-fence", wallet(1));
        state.reconcile_claim_recovery(Some(wallet(2)), Vec::new(), 300).unwrap();
        assert!(state.request_runtime.waiting.is_empty());

        state.queue_request(request(202, "answer"), 100).unwrap();
        state
            .record_claim(
                771,
                202,
                "answer",
                ConnectedAppClaimDecision::Answer,
                String::new(),
                String::new(),
                vec![4, 5],
                200,
            )
            .unwrap();
        state.reconcile_claim_recovery(None, Vec::new(), 300).unwrap();
        assert_eq!(
            state.request_runtime.recovery_submissions[&202].kind,
            ConnectedAppRecoverySubmissionKind::Answer
        );
    }
}
