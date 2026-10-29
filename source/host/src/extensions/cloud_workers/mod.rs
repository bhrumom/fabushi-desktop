//! Typed boundary for native cloud-worker provisioning.
//!
//! This module deliberately contains no provider, process, device, or UI side
//! effects. It is the state/identity contract that later slices may consume.
//! Fabushi account/device identity, Computer authorization, Sand state and
//! Runner composition remain owned by their existing modules.

mod lease;
mod artifact_admission;
mod doctor;
mod node_bootstrap;
mod platform_bridge;
mod provider_catalog;
mod sandbox;
mod warm_image;

pub use artifact_admission::{
    CloudWorkerArtifactAdmission, CloudWorkerArtifactAdmissionError,
    CloudWorkerArtifactAdmissionState, CloudWorkerArtifactDigest,
    CloudWorkerArtifactKind, CloudWorkerArtifactPolicy, CloudWorkerArtifactSource,
};
pub use doctor::{
    CloudWorkerDoctorAction, CloudWorkerDoctorError, CloudWorkerDoctorRequest,
    CloudWorkerDoctorResult, CloudWorkerDoctorStatus,
};
pub use node_bootstrap::{
    CloudWorkerBootstrapPhase, CloudWorkerBootstrapRecord, CloudWorkerBootstrapRequest,
    CloudWorkerBootstrapRetryClass, CloudWorkerDiagnostic, CloudWorkerNodeBootstrapError,
    CloudWorkerSecretRef,
};
pub use platform_bridge::{
    CloudWorkerDesktopSession, CloudWorkerPlatform, CloudWorkerPlatformBridge,
    CloudWorkerPlatformBridgeError,
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
pub use sandbox::{
    CloudWorkerSandboxCapability, CloudWorkerSandboxError, CloudWorkerSandboxOperation,
    CloudWorkerSandboxRequest,
};
pub use provider_catalog::{
    CloudWorkerCatalog, CloudWorkerCatalogError, CloudWorkerCatalogParser,
    CloudWorkerMachine, CloudWorkerOs, CloudWorkerProvider, CloudWorkerProfile,
    CloudWorkerResourceLimits,
};
