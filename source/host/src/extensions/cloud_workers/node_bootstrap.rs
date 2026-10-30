use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{CloudWorkerArtifactDigest, CloudWorkerLeaseScope, CloudWorkerRuntimeIdentity, ReplayKey};

const MAX_DIAGNOSTIC_CHARS: usize = 512;
const MAX_DIAGNOSTICS: usize = 32;
const MAX_SECRET_REF_CHARS: usize = 256;
const MAX_REPLAY_KEY_CHARS: usize = 256;

/// An opaque handle resolved by Fabushi's existing secret/credential owner.
/// The value is never a secret payload and must not be replaced by one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerSecretRef(String);

impl CloudWorkerSecretRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudWorkerNodeBootstrapError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_SECRET_REF_CHARS
            || value.chars().any(|character| character.is_control())
        {
            return Err(CloudWorkerNodeBootstrapError::InvalidSecretRef);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerBootstrapRetryClass {
    Never,
    SafeReplay,
    ReconcileFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerBootstrapPhase {
    Pending,
    Preparing,
    Enrolling,
    Ready,
    Failed,
    Cancelled,
    OutcomeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerBootstrapRequest {
    pub scope: CloudWorkerLeaseScope,
    pub runtime: CloudWorkerRuntimeIdentity,
    pub artifact: CloudWorkerArtifactDigest,
    pub secret_refs: Vec<CloudWorkerSecretRef>,
    pub replay_key: ReplayKey,
}

impl CloudWorkerBootstrapRequest {
    pub fn validate(&self) -> Result<(), CloudWorkerNodeBootstrapError> {
        CloudWorkerLeaseScope::new(
            self.scope.account_scope.clone(),
            self.scope.device_scope.clone(),
        )
        .map_err(|_| CloudWorkerNodeBootstrapError::InvalidScope)?;
        CloudWorkerRuntimeIdentity::new(
            self.runtime.provider.clone(),
            self.runtime.profile.clone(),
            self.runtime.runtime_id.clone(),
        )
        .map_err(|_| CloudWorkerNodeBootstrapError::InvalidRuntimeIdentity)?;
        CloudWorkerArtifactDigest::new(self.artifact.0.clone())
            .map_err(|_| CloudWorkerNodeBootstrapError::InvalidArtifactDigest)?;
        if self.secret_refs.is_empty() {
            return Err(CloudWorkerNodeBootstrapError::MissingSecretRef);
        }
        for secret_ref in &self.secret_refs {
            CloudWorkerSecretRef::new(secret_ref.0.clone())
                .map_err(|_| CloudWorkerNodeBootstrapError::InvalidSecretRef)?;
        }
        if self.replay_key.0.trim().is_empty() {
            return Err(CloudWorkerNodeBootstrapError::MissingReplayKey);
        }
        if self.replay_key.0.len() > MAX_REPLAY_KEY_CHARS
            || self.replay_key.0.chars().any(|character| character.is_control())
        {
            return Err(CloudWorkerNodeBootstrapError::InvalidReplayKey);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerDiagnostic {
    pub code: String,
    pub message: String,
    pub redacted: bool,
}

impl CloudWorkerDiagnostic {
    pub fn redacted(code: impl Into<String>, message: impl Into<String>) -> Self {
        let mut message = message.into();
        if message.chars().count() > MAX_DIAGNOSTIC_CHARS {
            message = message.chars().take(MAX_DIAGNOSTIC_CHARS).collect();
        }
        Self {
            code: code.into(),
            message,
            redacted: true,
        }
    }

    fn validate(&self) -> Result<(), CloudWorkerNodeBootstrapError> {
        if self.code.trim().is_empty()
            || self.code.chars().count() > MAX_DIAGNOSTIC_CHARS
            || self.code.chars().any(|character| character.is_control())
            || self.message.chars().count() > MAX_DIAGNOSTIC_CHARS
            || self.message.chars().any(|character| character.is_control())
            || !self.redacted
        {
            return Err(CloudWorkerNodeBootstrapError::InvalidDiagnostic);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerBootstrapRecord {
    pub request: CloudWorkerBootstrapRequest,
    pub phase: CloudWorkerBootstrapPhase,
    pub retry_class: CloudWorkerBootstrapRetryClass,
    pub runtime_identity: String,
    pub diagnostics: Vec<CloudWorkerDiagnostic>,
}

impl CloudWorkerBootstrapRecord {
    pub fn pending(
        request: CloudWorkerBootstrapRequest,
    ) -> Result<Self, CloudWorkerNodeBootstrapError> {
        request.validate()?;
        Ok(Self {
            runtime_identity: request.runtime.runtime_id.clone(),
            request,
            phase: CloudWorkerBootstrapPhase::Pending,
            retry_class: CloudWorkerBootstrapRetryClass::ReconcileFirst,
            diagnostics: Vec::new(),
        })
    }

    pub fn settle(
        &mut self,
        phase: CloudWorkerBootstrapPhase,
        retry_class: CloudWorkerBootstrapRetryClass,
        diagnostic: Option<CloudWorkerDiagnostic>,
    ) -> Result<(), CloudWorkerNodeBootstrapError> {
        if !is_allowed_transition(self.phase, phase) {
            return Err(CloudWorkerNodeBootstrapError::InvalidPhaseTransition {
                from: self.phase,
                to: phase,
            });
        }
        if matches!(phase, CloudWorkerBootstrapPhase::Ready)
            && self.request.secret_refs.is_empty()
        {
            return Err(CloudWorkerNodeBootstrapError::MissingSecretRef);
        }
        if let Some(diagnostic) = diagnostic {
            if self.diagnostics.len() >= MAX_DIAGNOSTICS {
                return Err(CloudWorkerNodeBootstrapError::DiagnosticLimitReached);
            }
            diagnostic.validate()?;
            self.diagnostics.push(diagnostic);
        }
        self.phase = phase;
        self.retry_class = retry_class;
        Ok(())
    }
}

fn is_allowed_transition(
    from: CloudWorkerBootstrapPhase,
    to: CloudWorkerBootstrapPhase,
) -> bool {
    matches!(
        (from, to),
        (CloudWorkerBootstrapPhase::Pending, CloudWorkerBootstrapPhase::Preparing)
            | (CloudWorkerBootstrapPhase::Pending, CloudWorkerBootstrapPhase::OutcomeUnknown)
            | (CloudWorkerBootstrapPhase::Preparing, CloudWorkerBootstrapPhase::Enrolling)
            | (CloudWorkerBootstrapPhase::Preparing, CloudWorkerBootstrapPhase::Failed)
            | (CloudWorkerBootstrapPhase::Preparing, CloudWorkerBootstrapPhase::Cancelled)
            | (CloudWorkerBootstrapPhase::Preparing, CloudWorkerBootstrapPhase::OutcomeUnknown)
            | (CloudWorkerBootstrapPhase::Enrolling, CloudWorkerBootstrapPhase::Ready)
            | (CloudWorkerBootstrapPhase::Enrolling, CloudWorkerBootstrapPhase::Failed)
            | (CloudWorkerBootstrapPhase::Enrolling, CloudWorkerBootstrapPhase::Cancelled)
            | (CloudWorkerBootstrapPhase::Enrolling, CloudWorkerBootstrapPhase::OutcomeUnknown)
            | (CloudWorkerBootstrapPhase::OutcomeUnknown, CloudWorkerBootstrapPhase::Preparing)
    )
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CloudWorkerNodeBootstrapError {
    #[error("secret reference is empty, too long, or contains control characters")]
    InvalidSecretRef,
    #[error("bootstrap scope is invalid")]
    InvalidScope,
    #[error("bootstrap runtime identity is invalid")]
    InvalidRuntimeIdentity,
    #[error("bootstrap artifact digest is invalid")]
    InvalidArtifactDigest,
    #[error("at least one secret reference is required")]
    MissingSecretRef,
    #[error("bootstrap requires a replay key")]
    MissingReplayKey,
    #[error("bootstrap replay key is invalid")]
    InvalidReplayKey,
    #[error("bootstrap diagnostic is missing redaction or contains invalid text")]
    InvalidDiagnostic,
    #[error("bootstrap diagnostic limit reached")]
    DiagnosticLimitReached,
    #[error("invalid bootstrap phase transition from {from:?} to {to:?}")]
    InvalidPhaseTransition {
        from: CloudWorkerBootstrapPhase,
        to: CloudWorkerBootstrapPhase,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> CloudWorkerBootstrapRequest {
        CloudWorkerBootstrapRequest {
            scope: CloudWorkerLeaseScope::new("account", "device").unwrap(),
            runtime: CloudWorkerRuntimeIdentity::new("provider", "profile", "runtime")
                .unwrap(),
            artifact: CloudWorkerArtifactDigest::new("a".repeat(64)).unwrap(),
            secret_refs: vec![CloudWorkerSecretRef::new("secret://bootstrap").unwrap()],
            replay_key: ReplayKey("request-1".into()),
        }
    }

    #[test]
    fn keeps_bootstrap_credentials_as_opaque_refs() {
        let record = CloudWorkerBootstrapRecord::pending(request()).unwrap();
        assert_eq!(record.request.secret_refs[0].as_str(), "secret://bootstrap");
        assert_eq!(record.phase, CloudWorkerBootstrapPhase::Pending);
    }

    #[test]
    fn ambiguous_bootstrap_is_reconcile_first() {
        let mut record = CloudWorkerBootstrapRecord::pending(request()).unwrap();
        record
            .settle(
                CloudWorkerBootstrapPhase::OutcomeUnknown,
                CloudWorkerBootstrapRetryClass::ReconcileFirst,
                Some(CloudWorkerDiagnostic::redacted("timeout", "provider response lost")),
            )
            .unwrap();
        assert_eq!(
            record.retry_class,
            CloudWorkerBootstrapRetryClass::ReconcileFirst
        );
        assert!(record.diagnostics[0].redacted);
    }

    #[test]
    fn rejects_raw_controlled_secret_values() {
        assert!(matches!(
            CloudWorkerSecretRef::new("secret\nvalue"),
            Err(CloudWorkerNodeBootstrapError::InvalidSecretRef)
        ));
    }

    #[test]
    fn readiness_requires_preparation_and_enrollment() {
        let mut record = CloudWorkerBootstrapRecord::pending(request()).unwrap();
        assert_eq!(
            record.settle(
                CloudWorkerBootstrapPhase::Ready,
                CloudWorkerBootstrapRetryClass::Never,
                None,
            ),
            Err(CloudWorkerNodeBootstrapError::InvalidPhaseTransition {
                from: CloudWorkerBootstrapPhase::Pending,
                to: CloudWorkerBootstrapPhase::Ready,
            })
        );
        record
            .settle(
                CloudWorkerBootstrapPhase::Preparing,
                CloudWorkerBootstrapRetryClass::SafeReplay,
                None,
            )
            .unwrap();
        record
            .settle(
                CloudWorkerBootstrapPhase::Enrolling,
                CloudWorkerBootstrapRetryClass::ReconcileFirst,
                None,
            )
            .unwrap();
        record
            .settle(
                CloudWorkerBootstrapPhase::Ready,
                CloudWorkerBootstrapRetryClass::Never,
                None,
            )
            .unwrap();
        assert_eq!(record.phase, CloudWorkerBootstrapPhase::Ready);
    }

    #[test]
    fn request_validation_does_not_trust_deserialized_digest() {
        let mut request = request();
        request.artifact = CloudWorkerArtifactDigest("not-a-digest".into());
        assert_eq!(
            request.validate(),
            Err(CloudWorkerNodeBootstrapError::InvalidArtifactDigest)
        );
    }

    #[test]
    fn settlement_rejects_unredacted_diagnostics() {
        let mut record = CloudWorkerBootstrapRecord::pending(request()).unwrap();
        assert_eq!(
            record.settle(
                CloudWorkerBootstrapPhase::Preparing,
                CloudWorkerBootstrapRetryClass::SafeReplay,
                Some(CloudWorkerDiagnostic {
                    code: "raw-provider-error".into(),
                    message: "secret payload".into(),
                    redacted: false,
                }),
            ),
            Err(CloudWorkerNodeBootstrapError::InvalidDiagnostic)
        );
        assert_eq!(record.phase, CloudWorkerBootstrapPhase::Pending);
    }
}
