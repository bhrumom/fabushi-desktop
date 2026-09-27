use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::host_bundle_upgrade::{HOST_BUNDLE_WATCH_INTERVAL_MS, HOST_BUNDLE_WATCH_JITTER_RATIO};
use super::host_upgrade_service::{HostUpgradeDependencies, HostUpgradeService};

pub const HOST_UPGRADE_EXTENSION_ID: &str = "host-upgrade";
pub const HOST_UPGRADE_EXTENSION_DEPENDENCIES: &[&str] = &[
    "automations",
    "cross-user-sharing",
    "telemetry",
    "transcript",
];
pub const HOST_UPGRADE_MARKER_FORWARD_INTERVAL_MS: u64 = 5 * 60_000;

pub fn is_host_bundle_auto_update_enabled(raw: Option<&str>) -> bool {
    !matches!(
        raw.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("0" | "false" | "no")
    )
}

pub fn host_bundle_watch_interval_ms(raw: Option<&str>) -> u64 {
    raw.and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(HOST_BUNDLE_WATCH_INTERVAL_MS)
}

pub fn host_bundle_watch_jitter_ratio(raw: Option<&str>) -> f64 {
    raw.and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0)
        .unwrap_or(HOST_BUNDLE_WATCH_JITTER_RATIO)
}

#[derive(Debug, Clone)]
pub struct HostUpgradeMarkerStore {
    path: PathBuf,
}

impl HostUpgradeMarkerStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read_raw(&self) -> Option<String> {
        fs::read_to_string(&self.path).ok()
    }

    pub fn delete_marker(&self) {
        match fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {}
        }
    }
}

pub struct HostUpgradeExtension<D: HostUpgradeDependencies> {
    service: Arc<HostUpgradeService<D>>,
    marker_store: HostUpgradeMarkerStore,
    update_watch_interval_ms: u64,
    update_watch_jitter_ratio: f64,
}

impl<D: HostUpgradeDependencies> HostUpgradeExtension<D> {
    pub fn new(
        deps: Arc<D>,
        marker_path: impl Into<PathBuf>,
        auto_update_enabled: bool,
        update_watch_interval_ms: u64,
        update_watch_jitter_ratio: f64,
    ) -> Self {
        Self {
            service: Arc::new(HostUpgradeService::new(deps, auto_update_enabled)),
            marker_store: HostUpgradeMarkerStore::new(marker_path),
            update_watch_interval_ms,
            update_watch_jitter_ratio,
        }
    }

    pub fn service(&self) -> Arc<HostUpgradeService<D>> {
        Arc::clone(&self.service)
    }

    pub fn marker_store(&self) -> &HostUpgradeMarkerStore {
        &self.marker_store
    }

    pub fn update_watch_interval_ms(&self) -> u64 {
        self.update_watch_interval_ms
    }

    pub fn update_watch_jitter_ratio(&self) -> f64 {
        self.update_watch_jitter_ratio
    }
}
