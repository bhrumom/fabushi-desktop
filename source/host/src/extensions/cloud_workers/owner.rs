use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    CloudWorkerBinaryIdentity, CloudWorkerCatalog, CloudWorkerCatalogCache,
    CloudWorkerCatalogError,
    CloudWorkerLease, CloudWorkerLeaseError, CloudWorkerLeaseMutation, CloudWorkerLeaseScope,
    CloudWorkerLeasedCapability, CloudWorkerLifecycle, CloudWorkerMutationOutcome,
    CloudWorkerRuntimeIdentity,
    CloudWorkerWarmImage, CloudWorkerWarmImageError, CloudWorkerWarmImageGeneration,
    CloudWorkerWarmImageOperation, LeaseId,
};

pub const CLOUD_WORKER_STATE_SCHEMA_VERSION: u32 = 1;
const MAX_MIGRATION_RECEIPTS: usize = 256;
const MAX_MIGRATION_OPERATION_CHARS: usize = 256;
const MAX_MIGRATION_BACKUP_PATH_CHARS: usize = 4096;
const MAX_MIGRATION_DETAIL_CHARS: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloudWorkerProvisioningConfig {
    pub enabled: bool,
    pub max_leases: usize,
    pub max_warm_images: usize,
}

impl Default for CloudWorkerProvisioningConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_leases: 64,
            max_warm_images: 128,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudWorkerOwnerStatus {
    Disabled,
    Enabled,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerMigrationOutcome {
    Previewed,
    Converted,
    RolledBack,
    OutcomeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerMigrationReceipt {
    pub operation_id: String,
    pub from_schema_version: u32,
    pub to_schema_version: u32,
    pub backup_path: String,
    pub outcome: CloudWorkerMigrationOutcome,
    pub detail: String,
}

impl CloudWorkerMigrationReceipt {
    fn validate(&self) -> Result<(), CloudWorkerOwnerError> {
        if !valid_text(&self.operation_id, MAX_MIGRATION_OPERATION_CHARS)
            || !valid_text(&self.backup_path, MAX_MIGRATION_BACKUP_PATH_CHARS)
            || !valid_text(&self.detail, MAX_MIGRATION_DETAIL_CHARS)
        {
            return Err(CloudWorkerOwnerError::InvalidMigrationReceipt);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CloudWorkerProvisioningOwner {
    config: CloudWorkerProvisioningConfig,
    leases: HashMap<LeaseId, CloudWorkerLease>,
    warm_images: HashMap<String, CloudWorkerWarmImage>,
    migration_receipts: Vec<CloudWorkerMigrationReceipt>,
    catalog_cache: CloudWorkerCatalogCache,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerPersistentSnapshot {
    pub schema_version: u32,
    pub leases: Vec<CloudWorkerLease>,
    pub warm_images: Vec<CloudWorkerWarmImage>,
    #[serde(default)]
    pub migration_receipts: Vec<CloudWorkerMigrationReceipt>,
}

impl CloudWorkerProvisioningOwner {
    pub fn new(config: CloudWorkerProvisioningConfig) -> Result<Self, CloudWorkerOwnerError> {
        if config.max_leases == 0 {
            return Err(CloudWorkerOwnerError::InvalidLeaseLimit);
        }
        if config.max_warm_images == 0 {
            return Err(CloudWorkerOwnerError::InvalidWarmImageLimit);
        }
        Ok(Self {
            config,
            leases: HashMap::new(),
            warm_images: HashMap::new(),
            migration_receipts: Vec::new(),
            catalog_cache: CloudWorkerCatalogCache::default(),
        })
    }

    pub fn status(&self) -> CloudWorkerOwnerStatus {
        if self.config.enabled {
            CloudWorkerOwnerStatus::Enabled
        } else {
            CloudWorkerOwnerStatus::Disabled
        }
    }

    /// Resolves provider capability data through the canonical owner while
    /// keeping the cache ephemeral and keyed by the exact binary identity.
    /// The loader remains an explicit provider/process boundary; this method
    /// does not discover, authorize, launch or mutate a worker.
    pub fn resolve_catalog<F>(
        &mut self,
        identity: CloudWorkerBinaryIdentity,
        load: F,
    ) -> Result<CloudWorkerCatalog, CloudWorkerCatalogError>
    where
        F: FnOnce() -> Result<CloudWorkerCatalog, CloudWorkerCatalogError>,
    {
        self.catalog_cache.resolve(identity, load)
    }

    pub fn snapshot(&self) -> CloudWorkerPersistentSnapshot {
        let mut leases = self.leases.values().cloned().collect::<Vec<_>>();
        leases.sort_by(|left, right| left.lease_id.0.cmp(&right.lease_id.0));
        let mut warm_images = self.warm_images.values().cloned().collect::<Vec<_>>();
        warm_images.sort_by(|left, right| left.image_id.cmp(&right.image_id));
        CloudWorkerPersistentSnapshot {
            schema_version: CLOUD_WORKER_STATE_SCHEMA_VERSION,
            leases,
            warm_images,
            migration_receipts: self.migration_receipts.clone(),
        }
    }

    pub fn from_snapshot(
        config: CloudWorkerProvisioningConfig,
        snapshot: CloudWorkerPersistentSnapshot,
    ) -> Result<Self, CloudWorkerOwnerError> {
        if snapshot.schema_version != CLOUD_WORKER_STATE_SCHEMA_VERSION {
            return Err(CloudWorkerOwnerError::UnsupportedSnapshotVersion(
                snapshot.schema_version,
            ));
        }
        let mut owner = Self::new(config)?;
        if snapshot.leases.len() > owner.config.max_leases {
            return Err(CloudWorkerOwnerError::LeaseLimitReached);
        }
        if snapshot.warm_images.len() > owner.config.max_warm_images {
            return Err(CloudWorkerOwnerError::WarmImageLimitReached);
        }
        for lease in snapshot.leases {
            lease.validate_persisted()?;
            if owner.leases.insert(lease.lease_id.clone(), lease).is_some() {
                return Err(CloudWorkerOwnerError::InvalidSnapshot);
            }
        }
        for image in snapshot.warm_images {
            image.validate_persisted()?;
            if owner
                .warm_images
                .insert(image.image_id.clone(), image)
                .is_some()
            {
                return Err(CloudWorkerOwnerError::InvalidSnapshot);
            }
        }
        if snapshot.migration_receipts.len() > MAX_MIGRATION_RECEIPTS {
            return Err(CloudWorkerOwnerError::MigrationReceiptLimitReached);
        }
        let mut operation_ids = std::collections::HashSet::new();
        for receipt in snapshot.migration_receipts {
            receipt.validate()?;
            if !operation_ids.insert(receipt.operation_id.clone()) {
                return Err(CloudWorkerOwnerError::InvalidSnapshot);
            }
            owner.migration_receipts.push(receipt);
        }
        Ok(owner)
    }

    pub fn restore_snapshot(
        &mut self,
        snapshot: CloudWorkerPersistentSnapshot,
    ) -> Result<(), CloudWorkerOwnerError> {
        let restored = Self::from_snapshot(self.config, snapshot)?;
        *self = restored;
        Ok(())
    }

    pub fn record_migration_receipt(
        &mut self,
        receipt: CloudWorkerMigrationReceipt,
    ) -> Result<(), CloudWorkerOwnerError> {
        receipt.validate()?;
        if self.migration_receipts.len() >= MAX_MIGRATION_RECEIPTS {
            return Err(CloudWorkerOwnerError::MigrationReceiptLimitReached);
        }
        if self
            .migration_receipts
            .iter()
            .any(|known| known.operation_id == receipt.operation_id)
        {
            return Err(CloudWorkerOwnerError::DuplicateMigrationReceipt);
        }
        self.migration_receipts.push(receipt);
        Ok(())
    }

    pub fn admit(
        &mut self,
        scope: CloudWorkerLeaseScope,
        lease_id: LeaseId,
        runtime: CloudWorkerRuntimeIdentity,
    ) -> Result<CloudWorkerLease, CloudWorkerOwnerError> {
        self.ensure_enabled()?;
        if self.leases.len() >= self.config.max_leases {
            return Err(CloudWorkerOwnerError::LeaseLimitReached);
        }
        if lease_id.0.trim().is_empty() {
            return Err(CloudWorkerOwnerError::InvalidLeaseId);
        }
        if self.leases.contains_key(&lease_id) {
            return Err(CloudWorkerOwnerError::DuplicateLease);
        }
        let lease = CloudWorkerLease {
            scope,
            lease_id: lease_id.clone(),
            owner_epoch: super::OwnerEpoch(0),
            generation: super::LeaseGeneration(0),
            runtime,
            lifecycle: CloudWorkerLifecycle::Admitted,
            last_outcome: CloudWorkerMutationOutcome::Completed,
        };
        lease.validate_persisted()?;
        self.leases.insert(lease_id, lease.clone());
        Ok(lease)
    }

    pub fn lease(&self, lease_id: &LeaseId) -> Result<&CloudWorkerLease, CloudWorkerOwnerError> {
        self.leases
            .get(lease_id)
            .ok_or(CloudWorkerOwnerError::UnknownLease)
    }

    /// Returns an ephemeral consumer capability only while the lease is
    /// enrolled and its last mutation completed successfully.
    pub fn leased_capability(
        &self,
        lease_id: &LeaseId,
    ) -> Result<CloudWorkerLeasedCapability, CloudWorkerOwnerError> {
        self.lease(lease_id)?
            .capability()
            .map_err(CloudWorkerOwnerError::from)
    }

    pub fn settle_lease(
        &mut self,
        mutation: &CloudWorkerLeaseMutation,
        next: CloudWorkerLifecycle,
        outcome: CloudWorkerMutationOutcome,
    ) -> Result<CloudWorkerLease, CloudWorkerOwnerError> {
        self.ensure_enabled()?;
        let lease = self
            .leases
            .get_mut(&mutation.lease_id)
            .ok_or(CloudWorkerOwnerError::UnknownLease)?;
        lease.validate(mutation)?;
        lease.apply_outcome(next, outcome)?;
        Ok(lease.clone())
    }

    pub fn register_warm_image(
        &mut self,
        image: CloudWorkerWarmImage,
    ) -> Result<(), CloudWorkerOwnerError> {
        self.ensure_enabled()?;
        if self.warm_images.len() >= self.config.max_warm_images {
            return Err(CloudWorkerOwnerError::WarmImageLimitReached);
        }
        image.validate_persisted()?;
        if self.warm_images.contains_key(&image.image_id) {
            return Err(CloudWorkerOwnerError::DuplicateWarmImage);
        }
        self.warm_images.insert(image.image_id.clone(), image);
        Ok(())
    }

    pub fn apply_warm_image(
        &mut self,
        image_id: &str,
        expected_generation: CloudWorkerWarmImageGeneration,
        operation: CloudWorkerWarmImageOperation,
    ) -> Result<CloudWorkerWarmImage, CloudWorkerOwnerError> {
        self.ensure_enabled()?;
        let image = self
            .warm_images
            .get_mut(image_id)
            .ok_or(CloudWorkerOwnerError::UnknownWarmImage)?;
        image.compare_and_apply(expected_generation, operation)?;
        Ok(image.clone())
    }

    fn ensure_enabled(&self) -> Result<(), CloudWorkerOwnerError> {
        if !self.config.enabled {
            return Err(CloudWorkerOwnerError::Disabled);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CloudWorkerOwnerError {
    #[error("cloud-worker provisioning owner is disabled")]
    Disabled,
    #[error("cloud-worker owner is unavailable")]
    Unavailable,
    #[error("cloud-worker leased capability is stale")]
    StaleCapability,
    #[error("maximum lease count must be greater than zero")]
    InvalidLeaseLimit,
    #[error("maximum warm-image count must be greater than zero")]
    InvalidWarmImageLimit,
    #[error("lease id must be non-empty")]
    InvalidLeaseId,
    #[error("lease already exists")]
    DuplicateLease,
    #[error("lease limit reached")]
    LeaseLimitReached,
    #[error("warm-image limit reached")]
    WarmImageLimitReached,
    #[error("lease was not found")]
    UnknownLease,
    #[error("warm image already exists")]
    DuplicateWarmImage,
    #[error("warm image was not found")]
    UnknownWarmImage,
    #[error("unsupported cloud-worker snapshot schema version: {0}")]
    UnsupportedSnapshotVersion(u32),
    #[error("cloud-worker snapshot contains invalid or duplicate records")]
    InvalidSnapshot,
    #[error("migration receipt operation id is empty or already recorded")]
    DuplicateMigrationReceipt,
    #[error("migration receipt fields are invalid")]
    InvalidMigrationReceipt,
    #[error("migration receipt limit reached")]
    MigrationReceiptLimitReached,
    #[error(transparent)]
    Lease(#[from] CloudWorkerLeaseError),
    #[error(transparent)]
    WarmImage(#[from] CloudWorkerWarmImageError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner() -> CloudWorkerProvisioningOwner {
        CloudWorkerProvisioningOwner::new(CloudWorkerProvisioningConfig {
            enabled: true,
            max_leases: 1,
            max_warm_images: 1,
        })
        .unwrap()
    }

    fn runtime() -> CloudWorkerRuntimeIdentity {
        CloudWorkerRuntimeIdentity::new("provider", "profile", "runtime").unwrap()
    }

    #[test]
    fn disabled_owner_has_no_mutation_side_effect() {
        let mut owner = CloudWorkerProvisioningOwner::new(Default::default()).unwrap();
        assert_eq!(owner.status(), CloudWorkerOwnerStatus::Disabled);
        assert_eq!(
            owner.admit(
                CloudWorkerLeaseScope::new("account", "device").unwrap(),
                LeaseId("lease-1".into()),
                runtime(),
            ),
            Err(CloudWorkerOwnerError::Disabled)
        );
    }

    #[test]
    fn owner_is_the_single_in_memory_lease_writer() {
        let mut owner = owner();
        let lease = owner
            .admit(
                CloudWorkerLeaseScope::new("account", "device").unwrap(),
                LeaseId("lease-1".into()),
                runtime(),
            )
            .unwrap();
        let mutation = lease.mutation(
            super::super::OwnerEpoch(0),
            lease.generation,
            lease.runtime.clone(),
            super::super::ReplayKey("request-1".into()),
        );
        let settled = owner
            .settle_lease(
                &mutation,
                CloudWorkerLifecycle::Preparing,
                CloudWorkerMutationOutcome::Completed,
            )
            .unwrap();
        assert_eq!(settled.lifecycle, CloudWorkerLifecycle::Preparing);
    }

    #[test]
    fn owner_enforces_lease_capacity() {
        let mut owner = owner();
        let scope = CloudWorkerLeaseScope::new("account", "device").unwrap();
        owner
            .admit(scope.clone(), LeaseId("lease-1".into()), runtime())
            .unwrap();
        assert_eq!(
            owner.admit(scope, LeaseId("lease-2".into()), runtime()),
            Err(CloudWorkerOwnerError::LeaseLimitReached)
        );
    }

    #[test]
    fn owner_rejects_zero_warm_image_capacity() {
        assert_eq!(
            CloudWorkerProvisioningOwner::new(CloudWorkerProvisioningConfig {
                enabled: true,
                max_leases: 1,
                max_warm_images: 0,
            })
            .unwrap_err(),
            CloudWorkerOwnerError::InvalidWarmImageLimit
        );
    }

    #[test]
    fn owner_caches_catalog_only_by_exact_binary_identity() {
        let mut owner = owner();
        let identity = CloudWorkerBinaryIdentity::new("sha256:provider").unwrap();
        let catalog = CloudWorkerCatalog {
            providers: vec![super::super::CloudWorkerProvider {
                id: "local".into(),
                profiles: vec![super::super::CloudWorkerProfile {
                    id: "default".into(),
                    machines: vec![super::super::CloudWorkerMachine {
                        os: super::super::CloudWorkerOs::Linux,
                        arch: "x86_64".into(),
                        resources: super::super::CloudWorkerResourceLimits {
                            cpu_cores: 2,
                            memory_bytes: 1024,
                        },
                    }],
                }],
            }],
        }
        .validate()
        .unwrap();
        let first = owner
            .resolve_catalog(identity.clone(), || Ok(catalog.clone()))
            .unwrap();
        let second = owner
            .resolve_catalog(identity, || {
                Err(super::super::CloudWorkerCatalogError::EmptyCatalog)
            })
            .unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn owner_rejects_invalid_direct_lease_records() {
        let mut owner = owner();
        let invalid_scope = CloudWorkerLeaseScope {
            account_scope: "account\nunsafe".into(),
            device_scope: "device".into(),
        };
        assert_eq!(
            owner.admit(
                invalid_scope,
                LeaseId("lease-unsafe".into()),
                runtime(),
            ),
            Err(CloudWorkerOwnerError::Lease(
                CloudWorkerLeaseError::InvalidPersistedRecord
            ))
        );
    }

    #[test]
    fn owner_can_restore_a_pre_mutation_snapshot() {
        let mut owner = owner();
        let before = owner.snapshot();
        owner
            .admit(
                CloudWorkerLeaseScope::new("account", "device").unwrap(),
                LeaseId("lease-1".into()),
                runtime(),
            )
            .unwrap();
        owner.restore_snapshot(before).unwrap();
        assert_eq!(
            owner.lease(&LeaseId("lease-1".into())),
            Err(CloudWorkerOwnerError::UnknownLease)
        );
    }
}

fn valid_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(|character| character.is_control())
}
