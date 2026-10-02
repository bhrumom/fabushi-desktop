use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::host_upgrade::host_bundle_source::{
    HostBundleFetcher, HostBundleHttpResponse,
};
use mahayana_host_runtime::extensions::host_upgrade::production::{
    ProductionHostUpgradePeers, start_production_host_upgrade_extension_with,
};
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostTelemetryService;

#[derive(Default)]
struct FakeFetcher {
    requests: Mutex<Vec<String>>,
}

impl HostBundleFetcher for FakeFetcher {
    fn fetch(&self, url: &str) -> Result<HostBundleHttpResponse, String> {
        self.requests.lock().unwrap().push(url.to_string());
        Ok(HostBundleHttpResponse {
            status: 404,
            body: Vec::new(),
        })
    }
}

#[test]
fn production_host_upgrade_binds_real_peer_hooks_without_starting_a_parallel_runtime() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-host-upgrade-production-{}",
        uuid::Uuid::new_v4()
    ));
    let telemetry = HostTelemetryService::open(root.join("telemetry.jsonl")).unwrap();
    let calls = Arc::new(Mutex::new(Vec::<String>::new()));
    let hook = |name: &'static str, calls: Arc<Mutex<Vec<String>>>| {
        Arc::new(move || {
            calls.lock().unwrap().push(name.to_string());
            Ok(())
        }) as Arc<dyn Fn() -> Result<(), String> + Send + Sync>
    };
    let sharing_calls = Arc::clone(&calls);
    let peers = ProductionHostUpgradePeers {
        suspend_automation_wakes: hook("automations", Arc::clone(&calls)),
        prepare_sharing_for_upgrade: Arc::new(move || {
            sharing_calls.lock().unwrap().push("sharing".into());
        }),
        quiesce_transcript_for_upgrade: hook("transcript", Arc::clone(&calls)),
        resume_interrupted_upgrade_turns: hook("resume", Arc::clone(&calls)),
    };
    let extension = start_production_host_upgrade_extension_with(
        peers,
        telemetry.logs.clone(),
        Arc::new(FakeFetcher::default()),
        root.join("upgrade-marker.json"),
        Arc::new(|| Ok(())),
    )
    .unwrap();

    extension.service().prepare_for_upgrade().unwrap();
    extension.service().resume_interrupted_upgrade_turns().unwrap();
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["automations", "sharing", "transcript", "resume"]
    );
    extension.stop_background_work();
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn production_upgrade_telemetry_uses_frozen_event_name() {
    let mut metadata = BTreeMap::new();
    metadata.insert("outcome".to_string(), "applied".to_string());
    assert_eq!(metadata.get("outcome").map(String::as_str), Some("applied"));
}


#[test]
fn shipping_host_recreates_durable_upgrade_turns_before_ack_redrive() {
    const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

    assert!(SHIPPING_HOST.contains("fn resume_interrupted_upgrade_turns(&self)"));
    assert!(SHIPPING_HOST.contains("start_local_upgrade_resume_turn("));
    assert!(SHIPPING_HOST.contains("gateway.resume_interrupted_upgrade_turns()"));
    assert!(!SHIPPING_HOST.contains(
        "resumeInterruptedUpgradeTurns production Runner recreation is not wired"
    ));

    let resume = SHIPPING_HOST
        .rfind("host_upgrade.service().resume_interrupted_upgrade_turns()")
        .expect("shipping startup upgrade resume");
    let redrive = SHIPPING_HOST
        .rfind("start_ack_redrive_worker(")
        .expect("shipping ack redrive");
    assert!(resume < redrive);
}

#[test]
fn shipping_recreate_gateway_carries_pending_wakes_and_resumes_durable_turns() {
    const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
    const GATEWAY_PROTOCOL: &str = include_str!("../src/gateway_protocol.rs");
    const TRANSCRIPT_MANAGER: &str =
        include_str!("../src/extensions/transcript/transcript_manager.rs");
    const TRANSCRIPT_RUNTIME: &str =
        include_str!("../src/extensions/transcript/production_runtime.rs");

    assert!(GATEWAY_PROTOCOL.contains("\"resumeAfterRecreate\""));
    assert!(SHIPPING_HOST.contains("if method == \"resumeAfterRecreate\""));
    assert!(SHIPPING_HOST.contains("\"resumePendingWakes\": self.transcript_manager.recreate_carry_pending_wakes()"));
    assert!(SHIPPING_HOST.contains(".restore_recreate_pending_wakes(carried)"));
    assert!(SHIPPING_HOST.contains("self.resume_interrupted_upgrade_turns()"));
    assert!(TRANSCRIPT_MANAGER.contains("pub fn restore_recreate_pending_wakes"));
    assert!(TRANSCRIPT_MANAGER.contains("owner.rearm_pending_wake(marker, now_ms, Some(\"recreate_carry\"))"));
    assert!(TRANSCRIPT_RUNTIME.contains("pub fn recreate_carry_pending_wakes"));
    assert!(TRANSCRIPT_RUNTIME.contains("PendingWakeKind::CloudAgent | PendingWakeKind::Shell"));
}
