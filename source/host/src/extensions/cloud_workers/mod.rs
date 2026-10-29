//! Typed boundary for native cloud-worker provisioning.
//!
//! This module deliberately contains no provider, process, device, or UI side
//! effects. It is the state/identity contract that later slices may consume.
//! Fabushi account/device identity, Computer authorization, Sand state and
//! Runner composition remain owned by their existing modules.

mod lease;

pub use lease::{
    CloudWorkerLease, CloudWorkerLeaseError, CloudWorkerLeaseMutation,
    CloudWorkerLeaseScope, CloudWorkerLifecycle, CloudWorkerMutationOutcome,
    CloudWorkerRuntimeIdentity, LeaseGeneration, LeaseId, OwnerEpoch,
    ReplayKey,
};
