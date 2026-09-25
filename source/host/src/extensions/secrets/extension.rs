use std::collections::BTreeMap;
use std::sync::Arc;

use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::extensions::forever_box::forever_box_service::ForeverBoxService;

use super::secrets_service::{
    BoxSecretsApplier, BoxSecretsApplierOptions, BoxSecretsApplyError,
    BoxSecretsLog, BoxSecretsSetError, BoxSecretsStatus,
};

pub const SECRETS_DEPENDENCIES: &[HostExtensionId] = &[HostExtensionId::ForeverBox];

pub fn secrets_extension_id() -> HostExtensionId {
    HostExtensionId::Secrets
}

pub struct HostSecretsExtension {
    service: BoxSecretsApplier,
}

impl HostSecretsExtension {
    pub fn set_secrets(
        &self,
        secrets: BTreeMap<String, String>,
    ) -> Result<BoxSecretsStatus, BoxSecretsSetError> {
        self.service.set_secrets(secrets)
    }

    pub fn get_status(&self) -> BoxSecretsStatus {
        self.service.get_status()
    }

    pub fn stop(&self) {
        self.service.stop();
    }
}

pub fn start_secrets_extension(
    box_service: Arc<ForeverBoxService>,
    log: BoxSecretsLog,
) -> HostSecretsExtension {
    let apply_box = Arc::clone(&box_service);
    let mut options = BoxSecretsApplierOptions::new(Arc::new(move |update| {
        apply_box
            .apply_environment(update)
            .map_err(|error| BoxSecretsApplyError::Retryable(error.to_string()))
    }));
    options.log = log;
    let service = BoxSecretsApplier::new(options);
    if let Err(error) = service.apply_persisted() {
        eprintln!("box secrets: failed to load persisted secrets: {error}");
    }
    HostSecretsExtension { service }
}
