use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::{
    SandLocalToolControllerEventKind, SandLocalToolPermissionController,
    SandLocalToolRequest, SandLocalToolScope,
};
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_resolution::SandLocalToolResolution;
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;

fn temp_settings() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-host-runner-permission-{}-{suffix}.json",
        std::process::id()
    ))
}

#[test]
fn direct_runner_surface_projects_created_and_settled_events_and_unbinds() {
    let path = temp_settings();
    let settings = Arc::new(SettingsService::new(path.clone()));
    let controller = Arc::new(SandLocalToolPermissionController::with_options(
        settings,
        1_000,
        Arc::new(|| 100),
        Arc::new(|| "request-1".to_string()),
    ));
    controller.bind_live_computer_check(Arc::new(|_| true));

    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_sink = Arc::clone(&observed);
    let composition = Arc::new(HostRunnerComposition::with_sink(
        Arc::clone(&controller),
        Arc::new(move |event| {
            observed_sink
                .lock()
                .expect("events")
                .push((event.kind, event.request.id.clone()));
        }),
    ));
    let ask_composition = Arc::clone(&composition);
    controller.bind_ask_surfaces(Arc::new(move |agent_id| {
        ask_composition.can_ask_local_tool_permission(agent_id)
    }));

    assert!(!composition.can_ask_local_tool_permission("agent-a"));
    composition.bind_local_permission_surface("agent-a");
    assert!(composition.can_ask_local_tool_permission("agent-a"));

    let scope = SandLocalToolScope {
        agent_id: "agent-a".into(),
        tool_call_id: Some("tool-a".into()),
        action: Some("read-file".into()),
        direction_epoch: None,
    };
    let request = SandLocalToolRequest::simple("read-file", "/tmp/a");
    let worker_controller = Arc::clone(&controller);
    let worker = thread::spawn(move || worker_controller.authorize(Some(&scope), &request));

    let pending = (0..100)
        .find_map(|_| {
            let pending = controller.get_pending_request_for_agent("agent-a");
            if pending.is_none() {
                thread::sleep(Duration::from_millis(2));
            }
            pending
        })
        .expect("pending");
    assert!(controller.resolve_request(&pending.id, SandLocalToolResolution::AllowOnce));
    assert!(worker.join().expect("worker").allowed);

    let events = observed.lock().expect("events").clone();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], (SandLocalToolControllerEventKind::Created, "request-1".into()));
    assert_eq!(events[1], (SandLocalToolControllerEventKind::Settled, "request-1".into()));

    composition.unbind_local_permission_surface("agent-a");
    assert!(!composition.can_ask_local_tool_permission("agent-a"));
    let _ = fs::remove_file(path);
}
