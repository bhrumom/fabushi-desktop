//! Typed boundary for native cloud-worker provisioning.
//!
//! This module deliberately contains no provider, process, device, or UI side
//! effects. It is the state/identity contract that later slices may consume.
//! Fabushi account/device identity, Computer authorization, Sand state and
//! Runner composition remain owned by their existing modules.

mod lease;
mod artifact_admission;
mod node_bootstrap;
mod provider_catalog;
mod warm_image;

pub use artifact_admission::{
    CloudWorkerArtifactAdmission, CloudWorkerArtifactAdmissionError,
    CloudWorkerArtifactAdmissionState, CloudWorkerArtifactDigest,
    CloudWorkerArtifactKind, CloudWorkerArtifactPolicy, CloudWorkerArtifactSource,
};
pub use node_bootstrap::{
    CloudWorkerBootstrapPhase, CloudWorkerBootstrapRecord, CloudWorkerBootstrapRequest,
    CloudWorkerBootstrapRetryClass, CloudWorkerDiagnostic, CloudWorkerNodeBootstrapError,
    CloudWorkerSecretRef,
};
pub use lease::{
    CloudWorkerLease, CloudWorkerLeaseError, CloudWorkerLeaseMutation,
    CloudWorkerLeaseScope, CloudWorkerLifecycle, CloudWorkerMutationOutcome,
    CloudWorkerRuntimeIdentity, LeaseGeneration, LeaseId, OwnerEpoch,
    ReplayKey,
};
pub use warm_image::{
    CloudWorkerWarmImage, CloudWorkerWarmImageError, CloudWorkerWarmImageGeneration,
    CloudWorkerWarmImageOperation, CloudWorkerWarmImageState,
};
pub use provider_catalog::{
    CloudWorkerCatalog, CloudWorkerCatalogError, CloudWorkerCatalogParser,
    CloudWorkerMachine, CloudWorkerOs, CloudWorkerProvider, CloudWorkerProfile,
    CloudWorkerResourceLimits,
};
