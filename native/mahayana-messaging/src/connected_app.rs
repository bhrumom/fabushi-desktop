use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    pub operation_id: String,
    pub signed_payload: String,
    pub answer: Vec<u8>,
    pub created_at_ms: i64,
}

impl ConnectedAppClaimRecord {
    pub fn validate(&self) -> Result<(), ConnectedAppError> {
        if self.session_id == 0
            || !connected_app_request_id_valid(&self.request_id)
            || self.trace_id.chars().count() > CONNECTED_APP_TRACE_ID_MAX_CHARS
            || self.expires_at_ms <= 0
            || self.created_at_ms < 0
            || self.operation_id.len() > CONNECTED_APP_OPERATION_ID_MAX_BYTES
            || self.signed_payload.len() > CONNECTED_APP_SIGNED_PAYLOAD_MAX_BYTES
            || self.answer.len() > CONNECTED_APP_ANSWER_MAX_BYTES
            || (!self.signed_payload.is_empty() && self.operation_id.is_empty())
        {
            return Err(ConnectedAppError::InvalidClaim);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ConnectedAppState {
    pub sessions: BTreeMap<u64, ConnectedAppSession>,
    pub pending_requests: BTreeMap<String, ConnectedAppRequest>,
    pub claims: Vec<ConnectedAppClaimRecord>,
}

impl ConnectedAppState {
    pub fn upsert_session(
        &mut self,
        session: ConnectedAppSession,
    ) -> Result<(), ConnectedAppError> {
        session.validate()?;
        if let Some(previous) = self.sessions.get(&session.id) {
            if previous.client_id != session.client_id
                || session.created_at_ms != previous.created_at_ms
                || session.updated_at_ms < previous.updated_at_ms
                || !valid_session_transition(previous.status, session.status)
            {
                return Err(ConnectedAppError::InvalidSessionTransition);
            }
        }
        self.sessions.insert(session.id, session);
        Ok(())
    }

    pub fn close_session(
        &mut self,
        session_id: u64,
        closed_at_ms: i64,
    ) -> Result<(), ConnectedAppError> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or(ConnectedAppError::SessionNotFound)?;
        if closed_at_ms < session.updated_at_ms {
            return Err(ConnectedAppError::StaleMutation);
        }
        session.status = ConnectedAppSessionStatus::Closed;
        session.updated_at_ms = closed_at_ms;
        self.pending_requests
            .retain(|_, request| request.session_id != session_id);
        Ok(())
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
        self.prune_claims(now_ms);
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
        if self.claims.len() >= CONNECTED_APP_CLAIM_MAX_RECORDS {
            return Err(ConnectedAppError::ClaimCapacity);
        }
        let claim = ConnectedAppClaimRecord {
            session_id,
            request_id: request.request_id.clone(),
            trace_id: request.trace_id.clone(),
            expires_at_ms: request.expires_at_ms,
            decision,
            operation_id,
            signed_payload,
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
            now_ms
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
                now_ms
                    <= claim
                        .expires_at_ms
                        .saturating_add(CONNECTED_APP_CLAIM_RETENTION_MS)
            })
            .cloned()
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
            .record_claim(
                7,
                "r1",
                ConnectedAppClaimDecision::Confirm,
                "operation-1".into(),
                "signed-payload".into(),
                vec![1, 2, 3],
                200,
            )
            .unwrap();
        assert_eq!(claim.trace_id, "trace");
        assert!(state.pending_requests.is_empty());
        assert_eq!(state.claims.len(), 1);
        assert_eq!(
            state.record_claim(
                7,
                "r1",
                ConnectedAppClaimDecision::Confirm,
                String::new(),
                String::new(),
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
    fn signed_payload_requires_operation_id_and_manifest_requires_https() {
        let mut state = ConnectedAppState::default();
        let mut active = session(ConnectedAppSessionStatus::Active);
        active.updated_at_ms = 11;
        state.upsert_session(active).unwrap();
        state.queue_request(request("r2", 1_000), 100).unwrap();
        assert_eq!(
            state.record_claim(
                7,
                "r2",
                ConnectedAppClaimDecision::Confirm,
                String::new(),
                "signed".into(),
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
