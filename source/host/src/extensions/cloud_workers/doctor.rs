use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{CloudWorkerLeaseScope, ReplayKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerDoctorAction {
    Inspect,
    RepairManagedArtifact,
    MigrateSecretRefs,
    ReconcileLease,
    ReconcileWarmImage,
}

impl CloudWorkerDoctorAction {
    fn requires_acknowledgement(self) -> bool {
        !matches!(self, Self::Inspect)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerDoctorRequest {
    pub scope: CloudWorkerLeaseScope,
    pub action: CloudWorkerDoctorAction,
    pub acknowledged_mutation: bool,
    pub replay_key: ReplayKey,
}

impl CloudWorkerDoctorRequest {
    pub fn validate(&self) -> Result<(), CloudWorkerDoctorError> {
        if self.replay_key.0.trim().is_empty() {
            return Err(CloudWorkerDoctorError::MissingReplayKey);
        }
        if self.action.requires_acknowledgement() && !self.acknowledged_mutation {
            return Err(CloudWorkerDoctorError::MutationNotAcknowledged);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerDoctorStatus {
    InspectionOnly,
    Pending,
    Completed,
    Failed,
    OutcomeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerDoctorResult {
    pub status: CloudWorkerDoctorStatus,
    pub mutation_started: bool,
    pub diagnostic_code: Option<String>,
}

impl CloudWorkerDoctorResult {
    pub fn inspection() -> Self {
        Self {
            status: CloudWorkerDoctorStatus::InspectionOnly,
            mutation_started: false,
            diagnostic_code: None,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CloudWorkerDoctorError {
    #[error("doctor request requires a replay key")]
    MissingReplayKey,
    #[error("mutating doctor action requires explicit acknowledgement")]
    MutationNotAcknowledged,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(action: CloudWorkerDoctorAction) -> CloudWorkerDoctorRequest {
        CloudWorkerDoctorRequest {
            scope: CloudWorkerLeaseScope::new("account", "device").unwrap(),
            action,
            acknowledged_mutation: false,
            replay_key: ReplayKey("doctor-1".into()),
        }
    }

    #[test]
    fn inspection_does_not_need_mutation_acknowledgement() {
        assert_eq!(request(CloudWorkerDoctorAction::Inspect).validate(), Ok(()));
        assert_eq!(
            CloudWorkerDoctorResult::inspection().mutation_started,
            false
        );
    }

    #[test]
    fn repair_requires_explicit_acknowledgement() {
        assert_eq!(
            request(CloudWorkerDoctorAction::RepairManagedArtifact).validate(),
            Err(CloudWorkerDoctorError::MutationNotAcknowledged)
        );
    }
}
