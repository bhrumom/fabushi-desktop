use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerLeaseScope {
    pub account_scope: String,
    pub device_scope: String,
}

impl CloudWorkerLeaseScope {
    pub fn new(
        account_scope: impl Into<String>,
        device_scope: impl Into<String>,
    ) -> Result<Self, CloudWorkerLeaseError> {
        let account_scope = account_scope.into();
        let device_scope = device_scope.into();
        if account_scope.trim().is_empty() || device_scope.trim().is_empty() {
            return Err(CloudWorkerLeaseError::InvalidScope);
        }
        Ok(Self {
            account_scope,
            device_scope,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerRuntimeIdentity {
    pub provider: String,
    pub profile: String,
    pub runtime_id: String,
}

impl CloudWorkerRuntimeIdentity {
    pub fn new(
        provider: impl Into<String>,
        profile: impl Into<String>,
        runtime_id: impl Into<String>,
    ) -> Result<Self, CloudWorkerLeaseError> {
        let identity = Self {
            provider: provider.into(),
            profile: profile.into(),
            runtime_id: runtime_id.into(),
        };
        if [&identity.provider, &identity.profile, &identity.runtime_id]
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(CloudWorkerLeaseError::InvalidRuntimeIdentity);
        }
        Ok(identity)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseGeneration(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerEpoch(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayKey(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerLifecycle {
    Admitted,
    Preparing,
    Warming,
    Bootstrapping,
    Enrolled,
    Inspecting,
    Heartbeating,
    Stopping,
    Disposed,
    OutcomeUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerMutationOutcome {
    Submitted,
    Completed,
    Failed,
    Cancelled,
    OutcomeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerLease {
    pub scope: CloudWorkerLeaseScope,
    pub lease_id: LeaseId,
    pub owner_epoch: OwnerEpoch,
    pub generation: LeaseGeneration,
    pub runtime: CloudWorkerRuntimeIdentity,
    pub lifecycle: CloudWorkerLifecycle,
}

impl CloudWorkerLease {
    pub fn mutation(
        &self,
        owner_epoch: OwnerEpoch,
        generation: LeaseGeneration,
        runtime: CloudWorkerRuntimeIdentity,
        replay_key: ReplayKey,
    ) -> CloudWorkerLeaseMutation {
        CloudWorkerLeaseMutation {
            lease_id: self.lease_id.clone(),
            owner_epoch,
            generation,
            runtime,
            replay_key,
        }
    }

    pub fn validate(
        &self,
        mutation: &CloudWorkerLeaseMutation,
    ) -> Result<(), CloudWorkerLeaseError> {
        if mutation.lease_id != self.lease_id {
            return Err(CloudWorkerLeaseError::StaleLease);
        }
        if mutation.owner_epoch != self.owner_epoch {
            return Err(CloudWorkerLeaseError::StaleOwnerEpoch);
        }
        if mutation.generation != self.generation {
            return Err(CloudWorkerLeaseError::StaleGeneration);
        }
        if mutation.runtime != self.runtime {
            return Err(CloudWorkerLeaseError::StaleRuntimeIdentity);
        }
        if mutation.replay_key.0.trim().is_empty() {
            return Err(CloudWorkerLeaseError::MissingReplayKey);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerLeaseMutation {
    pub lease_id: LeaseId,
    pub owner_epoch: OwnerEpoch,
    pub generation: LeaseGeneration,
    pub runtime: CloudWorkerRuntimeIdentity,
    pub replay_key: ReplayKey,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CloudWorkerLeaseError {
    #[error("account and device scope must be non-empty")]
    InvalidScope,
    #[error("provider, profile and runtime identity must be non-empty")]
    InvalidRuntimeIdentity,
    #[error("mutation refers to another lease")]
    StaleLease,
    #[error("mutation refers to a stale owner epoch")]
    StaleOwnerEpoch,
    #[error("mutation refers to a stale lease generation")]
    StaleGeneration,
    #[error("mutation refers to another runtime identity")]
    StaleRuntimeIdentity,
    #[error("mutation requires a replay key")]
    MissingReplayKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lease() -> CloudWorkerLease {
        CloudWorkerLease {
            scope: CloudWorkerLeaseScope::new("account", "device").unwrap(),
            lease_id: LeaseId("lease-1".into()),
            owner_epoch: OwnerEpoch(3),
            generation: LeaseGeneration(7),
            runtime: CloudWorkerRuntimeIdentity::new("provider", "profile", "runtime")
                .unwrap(),
            lifecycle: CloudWorkerLifecycle::Admitted,
        }
    }

    #[test]
    fn accepts_a_matching_mutation_identity() {
        let lease = lease();
        let mutation = lease.mutation(
            OwnerEpoch(3),
            LeaseGeneration(7),
            lease.runtime.clone(),
            ReplayKey("request-1".into()),
        );
        assert_eq!(lease.validate(&mutation), Ok(()));
    }

    #[test]
    fn rejects_stale_generation_before_side_effects() {
        let lease = lease();
        let mutation = lease.mutation(
            OwnerEpoch(3),
            LeaseGeneration(8),
            lease.runtime.clone(),
            ReplayKey("request-1".into()),
        );
        assert_eq!(
            lease.validate(&mutation),
            Err(CloudWorkerLeaseError::StaleGeneration)
        );
    }

    #[test]
    fn rejects_empty_replay_keys() {
        let lease = lease();
        let mutation = lease.mutation(
            OwnerEpoch(3),
            LeaseGeneration(7),
            lease.runtime.clone(),
            ReplayKey("  ".into()),
        );
        assert_eq!(
            lease.validate(&mutation),
            Err(CloudWorkerLeaseError::MissingReplayKey)
        );
    }
}
