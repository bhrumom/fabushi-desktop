use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::mcp::extension::{PollingHandle, PollingPort};
use mahayana_host_runtime::extensions::mcp::production::{
    DASHBOARD_GET_TEAMS_PATH, DASHBOARD_PUBLISH_PLUGIN_PATH, DASHBOARD_UNPUBLISH_PLUGIN_PATH,
    RealPluginSkillsPolling,
};

#[test]
fn production_rpc_paths_match_frozen_dashboard_contract() {
    assert_eq!(
        DASHBOARD_GET_TEAMS_PATH,
        "/aiserver.v1.DashboardService/GetTeams"
    );
    assert_eq!(
        DASHBOARD_PUBLISH_PLUGIN_PATH,
        "/aiserver.v1.DashboardService/PublishPlugin"
    );
    assert_eq!(
        DASHBOARD_UNPUBLISH_PLUGIN_PATH,
        "/aiserver.v1.DashboardService/UnpublishPlugin"
    );
}

#[test]
fn production_polling_runs_immediately_and_stops_after_dispose() {
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let calls_for_callback = Arc::clone(&calls);
    let seen_for_callback = Arc::clone(&seen);

    let polling = RealPluginSkillsPolling::with_interval(Duration::from_millis(10));
    let mut handle = polling.start(Arc::new(move || {
        let n = calls_for_callback.fetch_add(1, Ordering::SeqCst) + 1;
        seen_for_callback.lock().expect("seen poisoned").push(n);
    }));

    let deadline = Instant::now() + Duration::from_secs(1);
    while calls.load(Ordering::SeqCst) < 2 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    assert!(calls.load(Ordering::SeqCst) >= 2, "polling never advanced");

    handle.dispose();
    let after_dispose = calls.load(Ordering::SeqCst);
    thread::sleep(Duration::from_millis(40));
    assert_eq!(calls.load(Ordering::SeqCst), after_dispose);
    assert!(!seen.lock().expect("seen poisoned").is_empty());
}

#[test]
fn production_polling_dispose_is_idempotent() {
    let polling = RealPluginSkillsPolling::with_interval(Duration::from_secs(60));
    let mut handle = polling.start(Arc::new(|| {}));
    handle.dispose();
    handle.dispose();
}
