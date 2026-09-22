use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;

use crate::r#box::box_store_backend_policy::{
    is_box_store_copy_in_enabled, is_box_store_sync_enabled,
};
use crate::r#box::production::ProductionBoxEnvironment;

use super::forever_box_service::{ForeverBoxLifecycle, ForeverBoxService};
use super::host_box::HostBox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForeverBoxExtensionOptions {
    pub auto_update_enabled: bool,
    pub host_bundle_auto_update_enabled: bool,
    pub is_in_box: bool,
}

fn env_switch(environment: &BTreeMap<String, String>, name: &str) -> Option<bool> {
    environment
        .get(name)
        .map(|value| value.trim().to_ascii_lowercase())
        .map(|value| !matches!(value.as_str(), "0" | "false" | "no"))
}

pub fn is_image_auto_update_enabled(environment: &BTreeMap<String, String>) -> bool {
    is_box_store_sync_enabled(environment)
        && is_box_store_copy_in_enabled(environment)
        && env_switch(environment, "SAND_BOX_AUTO_UPDATE").unwrap_or(true)
}

pub fn is_host_bundle_auto_update_enabled(environment: &BTreeMap<String, String>) -> bool {
    env_switch(environment, "SAND_BOX_AUTO_UPDATE").unwrap_or(true)
}

impl ForeverBoxExtensionOptions {
    pub fn from_process_env() -> Self {
        let environment = env::vars().collect::<BTreeMap<_, _>>();
        Self {
            auto_update_enabled: is_image_auto_update_enabled(&environment),
            host_bundle_auto_update_enabled: is_host_bundle_auto_update_enabled(&environment),
            is_in_box: environment.get("SAND_HOST_IN_BOX").is_some_and(|value| value == "1"),
        }
    }
}

pub fn start_forever_box_extension(
    environment: ProductionBoxEnvironment,
    lifecycle: Arc<dyn ForeverBoxLifecycle>,
    options: ForeverBoxExtensionOptions,
) -> Arc<ForeverBoxService> {
    let service = Arc::new(ForeverBoxService::new(
        HostBox::new(environment),
        lifecycle,
        options.auto_update_enabled,
        options.host_bundle_auto_update_enabled,
        options.is_in_box,
    ));
    service.start();
    service
}
