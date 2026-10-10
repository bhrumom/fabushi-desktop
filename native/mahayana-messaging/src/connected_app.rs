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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppManifest {
    pub url: String,
    pub name: String,
    pub icon_url: Option<String>,
}

impl ConnectedAppManifest {
    pub fn validate(&self) -> Result<(), ConnectedAppError> {
        let name = self.name.trim();
        if name.is_empty() || name.chars().count() > CONNECTED_APP_NAME_MAX_CHARS {
            return Err(ConnectedAppError::InvalidManifest);
        }
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
    pub request_id: String,
    pub method: String,
    pub kind: ConnectedAppRequestKind,
    pub trace_id: String,
    pub expires_at_ms: i64,
}

impl ConnectedAppRequest {
    pub fn validate(&self, now_ms: i64) -> Result<(), ConnectedAppError> {
        if self.session_id == 0
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
            self.pending_requests.retain(|_, request| request.session_id != session.id);
        } else {
            self.sessions.insert(session.id, session);
        }
        Ok(())
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
            if session.status == ConnectedAppSessionStatus::Closed { continue; }
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
                self.pending_requests.retain(|_, request| request.session_id != local.id);
            } else if !self.closed_session_ids.contains(&local.id) {
                self.sessions.insert(local.id, local);
            }
        }
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
        let key = request_key(request.session_id, &request.request_id);
        if self.pending_requests.contains_key(&key)
            || self.claims.iter().any(|claim| {
                claim.session_id == request.session_id && claim.request_id == request.request_id
            })
        {
            return Err(ConnectedAppError::DuplicateRequest);
        }
        self.pending_requests.insert(key, request);
        Ok(())
    }

    pub fn record_claim(
        &mut self,
        session_id: u64,
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

    fn record_claim_internal(
        &mut self,
        session_id: u64,
        request_id: &str,
        decision: ConnectedAppClaimDecision,
        wallet_identity: Option<WalletTransferIdentity>,
        operation_id: String,
        signed_payload: String,
        not_sent: Vec<u8>,
        answer: Vec<u8>,
        now_ms: i64,
    ) -> Result<ConnectedAppClaimRecord, ConnectedAppError> {
        let key = request_key(session_id, request_id);
        let request = self
            .pending_requests
            .get(&key)
            .cloned()
            .ok_or(ConnectedAppError::RequestNotFound)?;
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
            .retain(|_, request| request.expires_at_ms > now_ms);
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

fn request_key(session_id: u64, request_id: &str) -> String {
    format!("{session_id}:{request_id}")
}

fn normalize_domain(value: &str) -> String {
    value
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

fn valid_secure_url(value: &str) -> bool {
    let value = value.trim();
    value.starts_with("https://")
        && value.len() > "https://".len()
        && !value["https://".len()..].starts_with('/')
        && !value.chars().any(char::is_control)
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
                "r1",
                ConnectedAppClaimDecision::Confirm,
                "operation-1".into(),
                "signed-payload".into(),
                vec![9],
                vec![1, 2, 3],
                200,
            )
            .unwrap();
        assert_eq!(claim.trace_id, "trace");
        assert!(state.pending_requests.is_empty());
        assert_eq!(state.claims.len(), 1);
        assert_eq!(
            state.record_claim_with_recovery(
                7,
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
                "late",
                ConnectedAppClaimDecision::Answer,
                String::new(),
                String::new(),
                Vec::new(),
                500,
            ),
            Err(ConnectedAppError::RequestExpired)
        );
        state.queue_request(request("recover", 1_000), 600).unwrap();
        state
            .record_claim(
                7,
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
            "wallet",
            ConnectedAppClaimDecision::Confirm,
            wallet.clone(),
            "operation-wallet".into(),
            "signed".into(),
            vec![2],
            vec![1],
            200,
        ).unwrap();
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
    fn signed_payload_requires_operation_id_and_manifest_requires_https() {
        let mut state = ConnectedAppState::default();
        let mut active = session(ConnectedAppSessionStatus::Active);
        active.updated_at_ms = 11;
        state.upsert_session(active).unwrap();
        state.queue_request(request("r2", 1_000), 100).unwrap();
        assert_eq!(
            state.record_claim_with_recovery(
                7,
                "r2",
                ConnectedAppClaimDecision::Confirm,
                String::new(),
                "signed".into(),
                vec![8],
                Vec::new(),
                200,
            ),
            Err(ConnectedAppError::InvalidClaim)
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
