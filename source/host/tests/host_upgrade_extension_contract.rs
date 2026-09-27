use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::sync::Arc;

use mahayana_host_runtime::extensions::host_upgrade::extension::{
    HOST_UPGRADE_EXTENSION_DEPENDENCIES, HOST_UPGRADE_EXTENSION_ID,
    HOST_UPGRADE_MARKER_FORWARD_INTERVAL_MS, HostUpgradeExtension, HostUpgradeMarkerStore,
    host_bundle_watch_interval_ms, host_bundle_watch_jitter_ratio,
    is_host_bundle_auto_update_enabled,
};
use mahayana_host_runtime::extensions::host_upgrade::host_bundle_upgrade::{
    HOST_BUNDLE_WATCH_INTERVAL_MS, HOST_BUNDLE_WATCH_JITTER_RATIO,
};
use mahayana_host_runtime::extensions::host_upgrade::host_upgrade_service::HostUpgradeDependencies;

struct NoopDeps;

impl HostUpgradeDependencies for NoopDeps {
    fn suspend_automation_wakes(&self) -> Result<(), String> {
        Ok(())
    }
    fn prepare_sharing_for_upgrade(&self) {}
    fn quiesce_transcript_for_upgrade(&self) -> Result<(), String> {
        Ok(())
    }
    fn resume_interrupted_upgrade_turns(&self) -> Result<(), String> {
        Ok(())
    }
    fn resolve_bundle_version(&self) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn load_bundle_bytes(&self, _version: &str) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }
    fn read_local_version(&self) -> Option<String> {
        None
    }
    fn is_version_swap_vetoed(&self, _version: &str, _local: Option<&str>) -> bool {
        false
    }
    fn stage_upgrade(
        &self,
        _version: &str,
        _bytes: &[u8],
        _force: bool,
        _reason: &str,
    ) -> Result<(), String> {
        Ok(())
    }
    fn read_marker_raw(&self) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn delete_marker(&self) -> Result<(), String> {
        Ok(())
    }
    fn report_host_upgrade(&self, _metadata: HashMap<String, String>) {}
    fn report_host_upgrade_confirmed(
        &self,
        _metadata: &BTreeMap<String, String>,
    ) -> Result<bool, String> {
        Ok(true)
    }
    fn log(&self, _level: &str, _message: &str) {}
    fn now_ms(&self) -> i64 {
        0
    }
}

#[test]
fn frozen_extension_identity_and_environment_policy_are_preserved() {
    assert_eq!(HOST_UPGRADE_EXTENSION_ID, "host-upgrade");
    assert_eq!(
        HOST_UPGRADE_EXTENSION_DEPENDENCIES,
        [
            "automations",
            "cross-user-sharing",
            "telemetry",
            "transcript"
        ]
    );
    assert_eq!(HOST_UPGRADE_MARKER_FORWARD_INTERVAL_MS, 300_000);
    assert!(is_host_bundle_auto_update_enabled(None));
    assert!(!is_host_bundle_auto_update_enabled(Some(" false ")));
    assert!(!is_host_bundle_auto_update_enabled(Some("NO")));
    assert!(is_host_bundle_auto_update_enabled(Some("yes")));
    assert_eq!(host_bundle_watch_interval_ms(Some("1234")), 1234);
    assert_eq!(
        host_bundle_watch_interval_ms(Some("0")),
        HOST_BUNDLE_WATCH_INTERVAL_MS
    );
    assert_eq!(host_bundle_watch_jitter_ratio(Some("0.25")), 0.25);
    assert_eq!(
        host_bundle_watch_jitter_ratio(Some("-1")),
        HOST_BUNDLE_WATCH_JITTER_RATIO
    );
}

#[test]
fn marker_store_is_fail_closed_and_delete_is_idempotent() {
    let root = std::env::temp_dir().join(format!("fabushi-host-marker-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("marker.json");
    let store = HostUpgradeMarkerStore::new(&path);
    assert_eq!(store.read_raw(), None);
    fs::write(&path, "{\"outcome\":\"applied\"}").unwrap();
    assert_eq!(
        store.read_raw().as_deref(),
        Some("{\"outcome\":\"applied\"}")
    );
    store.delete_marker();
    assert_eq!(store.read_raw(), None);
    store.delete_marker();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn extension_owns_service_and_runtime_watch_configuration() {
    let root =
        std::env::temp_dir().join(format!("fabushi-host-extension-{}", uuid::Uuid::new_v4()));
    let extension =
        HostUpgradeExtension::new(Arc::new(NoopDeps), root.join("marker"), true, 42, 0.1);
    assert!(extension.service().is_auto_update_enabled());
    assert_eq!(extension.update_watch_interval_ms(), 42);
    assert_eq!(extension.update_watch_jitter_ratio(), 0.1);
    assert_eq!(extension.marker_store().path(), root.join("marker"));
}
