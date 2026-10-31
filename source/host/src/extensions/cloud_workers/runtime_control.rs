use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    CloudWorkerLease, CloudWorkerLeaseError, CloudWorkerLeaseMutation,
    CloudWorkerLifecycle, ReplayKey,
};

/// The non-replayable control operations that a future provider producer may
/// perform against an already admitted worker. This type only admits a
/// lease-fenced mutation; it never performs the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerRuntimeControlAction {
    Prepare,
    Warmup,
    Bootstrap,
    Inspect,
    Heartbeat,
    Stop,
    Dispose,
}

impl CloudWorkerRuntimeControlAction {
    /// Returns the lifecycle that a completed operation is allowed to enter.
    /// Submitted, failed, cancelled and outcome-unknown results are settled
    /// by the lease owner separately.
    pub fn completed_lifecycle(
        self,
        current: CloudWorkerLifecycle,
    ) -> Result<CloudWorkerLifecycle, CloudWorkerRuntimeControlError> {
        let next = match (self, current) {
            // Inspecting and Heartbeating are transient lifecycle labels used
            // by the wider lease state machine. A completed read/control
            // operation returns the lease to Enrolled so its capability is not
            // stranded after one successful observation.
            (Self::Prepare, CloudWorkerLifecycle::Admitted) => CloudWorkerLifecycle::Preparing,
            (Self::Warmup, CloudWorkerLifecycle::Preparing) => CloudWorkerLifecycle::Warming,
            (Self::Bootstrap, CloudWorkerLifecycle::Warming) => {
                CloudWorkerLifecycle::Bootstrapping
            }
            (Self::Inspect, CloudWorkerLifecycle::Enrolled)
            | (Self::Heartbeat, CloudWorkerLifecycle::Enrolled) => CloudWorkerLifecycle::Enrolled,
            (Self::Stop, CloudWorkerLifecycle::Enrolled)
            | (Self::Stop, CloudWorkerLifecycle::OutcomeUnknown) => CloudWorkerLifecycle::Stopping,
            (Self::Dispose, CloudWorkerLifecycle::Stopping)
            | (Self::Dispose, CloudWorkerLifecycle::OutcomeUnknown) => {
                CloudWorkerLifecycle::Disposed
            }
            (_, current) => {
                return Err(CloudWorkerRuntimeControlError::InvalidLifecycle {
                    action: self,
                    current,
                });
            }
        };
        Ok(next)
    }

    /// Builds the exact mutation identity a producer must use for settlement.
    /// The owner remains the only component allowed to apply the result.
    pub fn admit(
        self,
        lease: &CloudWorkerLease,
        replay_key: ReplayKey,
    ) -> Result<CloudWorkerLeaseMutation, CloudWorkerRuntimeControlError> {
        self.completed_lifecycle(lease.lifecycle)?;
        let mutation = lease.mutation(
            lease.owner_epoch,
            lease.generation,
            lease.runtime.clone(),
            replay_key,
        );
        lease.validate(&mutation)?;
        Ok(mutation)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CloudWorkerRuntimeControlError {
    #[error("runtime control {action:?} is invalid while lease is {current:?}")]
    InvalidLifecycle {
        action: CloudWorkerRuntimeControlAction,
        current: CloudWorkerLifecycle,
    },
    #[error(transparent)]
    Lease(#[from] CloudWorkerLeaseError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::cloud_workers::{
        CloudWorkerLeaseScope, CloudWorkerMutationOutcome, CloudWorkerRuntimeIdentity,
        LeaseGeneration, LeaseId, OwnerEpoch,
    };

    fn lease(lifecycle: CloudWorkerLifecycle) -> CloudWorkerLease {
        CloudWorkerLease {
            scope: CloudWorkerLeaseScope::new("account", "device").unwrap(),
            lease_id: LeaseId("lease-1".into()),
            owner_epoch: OwnerEpoch(4),
            generation: LeaseGeneration(9),
            runtime: CloudWorkerRuntimeIdentity::new("provider", "profile", "runtime").unwrap(),
            lifecycle,
            last_outcome: if lifecycle == CloudWorkerLifecycle::OutcomeUnknown {
                CloudWorkerMutationOutcome::OutcomeUnknown
            } else {
                CloudWorkerMutationOutcome::Completed
            },
        }
    }

    #[test]
    fn inspect_and_heartbeat_require_enrolled_lease() {
        assert_eq!(
            CloudWorkerRuntimeControlAction::Inspect.completed_lifecycle(
                CloudWorkerLifecycle::Enrolled
            ),
            Ok(CloudWorkerLifecycle::Enrolled)
        );
        assert_eq!(
            CloudWorkerRuntimeControlAction::Heartbeat.completed_lifecycle(
                CloudWorkerLifecycle::Enrolled
            ),
            Ok(CloudWorkerLifecycle::Enrolled)
        );
        assert!(matches!(
            CloudWorkerRuntimeControlAction::Inspect
                .completed_lifecycle(CloudWorkerLifecycle::Preparing),
            Err(CloudWorkerRuntimeControlError::InvalidLifecycle { .. })
        ));
    }

    #[test]
    fn preparation_actions_follow_the_declared_lifecycle_order() {
        assert_eq!(
            CloudWorkerRuntimeControlAction::Prepare
                .completed_lifecycle(CloudWorkerLifecycle::Admitted),
            Ok(CloudWorkerLifecycle::Preparing)
        );
        assert_eq!(
            CloudWorkerRuntimeControlAction::Warmup
                .completed_lifecycle(CloudWorkerLifecycle::Preparing),
            Ok(CloudWorkerLifecycle::Warming)
        );
        assert_eq!(
            CloudWorkerRuntimeControlAction::Bootstrap
                .completed_lifecycle(CloudWorkerLifecycle::Warming),
            Ok(CloudWorkerLifecycle::Bootstrapping)
        );
        assert!(matches!(
            CloudWorkerRuntimeControlAction::Bootstrap
                .completed_lifecycle(CloudWorkerLifecycle::Admitted),
            Err(CloudWorkerRuntimeControlError::InvalidLifecycle { .. })
        ));
    }

    #[test]
    fn stop_and_dispose_preserve_outcome_unknown_recovery() {
        assert_eq!(
            CloudWorkerRuntimeControlAction::Stop
                .completed_lifecycle(CloudWorkerLifecycle::OutcomeUnknown),
            Ok(CloudWorkerLifecycle::Stopping)
        );
        assert_eq!(
            CloudWorkerRuntimeControlAction::Dispose
                .completed_lifecycle(CloudWorkerLifecycle::OutcomeUnknown),
            Ok(CloudWorkerLifecycle::Disposed)
        );
        assert!(matches!(
            CloudWorkerRuntimeControlAction::Dispose
                .completed_lifecycle(CloudWorkerLifecycle::Enrolled),
            Err(CloudWorkerRuntimeControlError::InvalidLifecycle { .. })
        ));
    }

    #[test]
    fn admission_preserves_fencing_identity() {
        let lease = lease(CloudWorkerLifecycle::Enrolled);
        let mutation = CloudWorkerRuntimeControlAction::Heartbeat
            .admit(&lease, ReplayKey("heartbeat-1".into()))
            .unwrap();
        assert_eq!(mutation.lease_id, lease.lease_id);
        assert_eq!(mutation.owner_epoch, lease.owner_epoch);
        assert_eq!(mutation.generation, lease.generation);
        assert_eq!(mutation.runtime, lease.runtime);
    }

    #[test]
    fn admission_rejects_invalid_replay_key_before_side_effects() {
        let lease = lease(CloudWorkerLifecycle::Enrolled);
        assert_eq!(
            CloudWorkerRuntimeControlAction::Inspect.admit(&lease, ReplayKey("  ".into())),
            Err(CloudWorkerRuntimeControlError::Lease(
                CloudWorkerLeaseError::MissingReplayKey
            ))
        );
    }
}
