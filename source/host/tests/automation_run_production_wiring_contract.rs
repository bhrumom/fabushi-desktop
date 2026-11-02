use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::automations::automation::{AutomationSpec, AUTOMATION_MAX_RUN_HISTORY};
use mahayana_host_runtime::automations::automation_id::stable_automation_id;
use mahayana_host_runtime::automations::automation_store::FileAutomationStore;
use mahayana_host_runtime::automations::automation_trigger::trigger_schedule;
use mahayana_host_runtime::automations::routine_notices::GITHUB_LISTENER_SCOPE;
use mahayana_host_runtime::extensions::automations::backend_transport::{
    AutomationsBackendError, AutomationsBackendTransport,
};
use mahayana_host_runtime::extensions::automations::fire_delivery::{
    PreparedBackendFire, prepare_backend_fire,
};
use mahayana_host_runtime::extensions::automations::production::{
    FireDispatch, ProductionAutomationsBackendRuntime,
};
use mahayana_host_runtime::extensions::automations::sand_automation_cloud_sync::{
    ScheduledCloudAutomation, sand_cloud_definition,
};
use mahayana_host_runtime::extensions::automations::sand_automation_fire_consumer::{
    AutomationFireBackendRuntime, BackendAutomationFire, FireCompletion,
};
use mahayana_host_runtime::extensions::browser_ua::extension::StopSubscription;
use mahayana_host_runtime::extensions::notify_bus::extension::{
    HostNotifyBusExtension, NotifyBusExperimentsApi, NotifyBusExtensionOptions,
    start_notify_bus_extension_with_options,
};
use mahayana_host_runtime::extensions::notify_bus::notify_bus_client::NotifyBusTiming;
use mahayana_host_runtime::extensions::session::agent_db::{CronSessionRunRecord, SandAgentDb};
use mahayana_host_runtime::extensions::session::agent_db_serde::{SpendGuardState, UnreadState};
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::automation_event_fires::{
    AutomationEventFires, EventBatchExecutor, EventFireBatch,
};
use mahayana_host_runtime::extensions::transcript::automation_run_path::{
    AutomationAdmissionStage, AutomationExecutionResult, AutomationRunAttempt,
    AutomationRunFenceSnapshot, AutomationRunObservation, AutomationRunTrigger, FireAutomationOutcome,
};
use mahayana_host_runtime::extensions::transcript::automation_runtime::{
    AutomationLifecycleAction, AutomationLifecycleEvent, AutomationLifecycleSource, AutomationRuntime,
};
use mahayana_host_runtime::extensions::transcript::sand_automation_spend_guard::{
    SPEND_GUARD_IDLE_TTL_MS, SpendGuardDecision,
};
use serde_json::{Value, json};

const CHILD_CASE_ENV: &str = "FABUSHI_AUTOMATION_CONTRACT_CASE";
const CHILD_ROOT_ENV: &str = "FABUSHI_AUTOMATION_CONTRACT_ROOT";
const ARTIFACT_ROOT_ENV: &str = "FABUSHI_AUTOMATION_CONTRACT_ARTIFACT_ROOT";
const ALLOCATION_RECORD: &str = "fixture-allocation.json";
static ROOT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn claim_fixture_root(path: PathBuf) -> io::Result<PathBuf> {
    // An existing directory belongs to somebody else, even if the clock and
    // process identity happen to repeat. Never acquire cleanup custody for it.
    fs::create_dir(&path)?;
    eprintln!("[automation-fixture] claimed pid={} case={:?} root={}",
        std::process::id(), std::thread::current().name(), path.display());
    // Session materialization can roll back its UUID directory before the
    // failure reaches this harness. Keep a file even if only empty dirs remain;
    // allocation is deliberately separate from the child's completion receipt.
    fs::write(path.join(ALLOCATION_RECORD), json!({
        "kind": "allocation", "processId": std::process::id(),
        "case": std::thread::current().name(), "root": path.to_string_lossy(),
    }).to_string())?;
    Ok(path)
}

fn reserve_fixture_root(base: &Path, timestamp: u128) -> io::Result<PathBuf> {
    let sequence = ROOT_SEQUENCE.fetch_update(Ordering::Relaxed, Ordering::Relaxed,
        |value| value.checked_add(1))
        .map_err(|_| io::Error::other("automation fixture root sequence exhausted"))?;
    claim_fixture_root(base.join(format!(
        "fabushi-automation-run-production-{}-{timestamp}-{sequence}",
        std::process::id(),
    )))
}

fn root() -> std::path::PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let base = std::env::var_os(CHILD_ROOT_ENV)
        .or_else(|| std::env::var_os(ARTIFACT_ROOT_ENV))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    reserve_fixture_root(&base, n).unwrap_or_else(|error| {
        panic!("reserve exclusive automation fixture under {}: {error}", base.display())
    })
}

struct OwnedAutomationChild {
    child: Child,
    root: std::path::PathBuf,
    completed: bool,
}

impl Drop for OwnedAutomationChild {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if self.completed {
            let _ = fs::remove_dir_all(&self.root);
        } else {
            eprintln!("[automation-fixture] retained failed child pid={} root={}",
                self.child.id(), self.root.display());
        }
    }
}

fn bounded_automation_case(test_name: &str, run: impl FnOnce()) {
    if std::env::var(CHILD_CASE_ENV).ok().as_deref() == Some(test_name) {
        eprintln!("[automation-fixture] entered child pid={} case={} root={:?}",
            std::process::id(), test_name, std::env::var_os(CHILD_ROOT_ENV));
        run();
        let root = std::env::var_os(CHILD_ROOT_ENV).expect("owned child fixture root");
        fs::write(std::path::PathBuf::from(root).join("completed"), test_name)
            .expect("record completed real-owner assertions");
        return;
    }

    // A recursive owner-lock regression must fail this case within a bound.
    // A hung thread cannot be safely joined or cleaned up in this test process;
    // the child owns only this exact case and its private filesystem fixtures.
    let child_root = root();
    let child = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(CHILD_CASE_ENV, test_name)
        .env(CHILD_ROOT_ENV, &child_root)
        .spawn()
        .expect("start bounded real-owner automation case");
    eprintln!("[automation-fixture] launched parent_pid={} child_pid={} case={} root={}",
        std::process::id(), child.id(), test_name, child_root.display());
    let mut owned = OwnedAutomationChild { child, root: child_root, completed: false };
    let deadline = Instant::now() + Duration::from_secs(45);
    let status = loop {
        if let Some(status) = owned.child.try_wait().expect("observe owned test child") {
            break status;
        }
        assert!(Instant::now() < deadline, "real automation owner case timed out: {test_name}");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(status.success(), "real automation owner case failed: {test_name}: {status}");
    assert_eq!(
        fs::read_to_string(owned.root.join("completed")).expect("case completion receipt"),
        test_name,
        "a successful child with no selected case is not evidence",
    );
    owned.completed = true;
}

#[test]
fn fixture_roots_are_exclusive_at_a_fixed_time_and_cleanup_keeps_siblings() {
    let base = root();
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let threads = (0..2).map(|_| {
        let base = base.clone();
        let barrier = Arc::clone(&barrier);
        std::thread::spawn(move || {
            barrier.wait();
            reserve_fixture_root(&base, 42).expect("exclusive fixed-time allocation")
        })
    }).collect::<Vec<_>>();
    let paths = threads.into_iter().map(|thread| thread.join().unwrap()).collect::<Vec<_>>();
    assert_ne!(paths[0], paths[1]);
    let sentinel = paths[1].join("owned-sentinel");
    fs::write(&sentinel, b"second owner remains live").unwrap();
    let allocation = fs::read(paths[1].join(ALLOCATION_RECORD)).unwrap();
    let recorded: serde_json::Value = serde_json::from_slice(&allocation).unwrap();
    assert_eq!(recorded["kind"], "allocation");
    assert_eq!(recorded["processId"], std::process::id());
    assert!(!paths[1].join("completed").exists(), "allocation is not completion");
    let error = claim_fixture_root(paths[1].clone()).expect_err("existing root has an owner");
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&sentinel).unwrap(), b"second owner remains live");
    assert_eq!(fs::read(paths[1].join(ALLOCATION_RECORD)).unwrap(), allocation);
    fs::remove_dir_all(&paths[0]).unwrap();
    assert!(!paths[0].exists());
    assert_eq!(fs::read(&sentinel).unwrap(), b"second owner remains live");
    assert_eq!(fs::read(paths[1].join(ALLOCATION_RECORD)).unwrap(), allocation);
    fs::write(paths[1].join("after-sibling-cleanup"), b"still writable").unwrap();
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn real_shipping_runtime_reports_after_terminal_settlement() {
    let root = root();
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let sessions = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = sessions.create_session(None, "user", None).expect("agent");
    let runtime = AutomationRuntime::new(Arc::clone(&workers));
    let (created, _) = runtime
        .create_agent_automation(
            &agent.id,
            &AutomationSpec {
                name: "Telemetry run".into(),
                prompt: "Inspect the queue".into(),
                trigger: json!({"type":"cron","schedule":"@daily"}),
                is_enabled: Some(true),
            },
        )
        .expect("create");
    let automation_id = created[0].id.clone();

    let observed = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
    let observed_sink = Arc::clone(&observed);
    runtime.run_path().set_run_reporter(Some(Arc::new(move |report| {
        observed_sink.lock().unwrap().push(report.clone());
    })));

    let outcome = runtime
        .run_agent_automation_now_with(&agent.id, &automation_id, |_| {
            Ok(AutomationExecutionResult::Completed)
        })
        .expect("run");
    assert_eq!(outcome, Some(FireAutomationOutcome::Ok));

    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 1);
    let report = &observed[0];
    assert_eq!(report.agent_id, agent.id);
    assert_eq!(report.automation_id, automation_id);
    assert_eq!(report.trigger, AutomationRunTrigger::Manual);
    assert_eq!(report.outcome, FireAutomationOutcome::Ok);
    assert_eq!(report.is_group, Some(false));
    assert_eq!(report.scheduled_for_ms, None);
    assert_eq!(report.lateness_ms, None);
    assert_eq!(report.event_batch_size, None);
    assert!(report.duration_ms >= 0.0);

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_composition_uses_canonical_mapper_and_unique_host_owner() {
    const HOST: &str = include_str!("../app/src/main.rs");
    const RUN_PATH: &str = include_str!("../src/extensions/transcript/automation_run_path.rs");
    const OWNER: &str = include_str!("../src/extensions/telemetry/host_telemetry_service.rs");
    const COORDINATOR: &str = include_str!("../../node-agent-coordinator/src/main.rs");
    const ELECTRON: &str = include_str!("../../electron-main/telemetry/desktop-structured-log-telemetry.ts");

    assert!(RUN_PATH.contains("reporter(&AutomationRunObservation"));
    assert!(HOST.contains("set_run_reporter(Some(Arc::new(move |observed|"));
    assert!(HOST.contains("stable_automation_id("));
    assert!(HOST.contains("run_logs.report_automation_run(&report)"));
    assert!(OWNER.contains("pub fn report_automation_run(&self, report: &AutomationRunReport)"));
    assert!(OWNER.contains("self.report_projection(&automation_run_telemetry(report))"));
    assert!(!COORDINATOR.contains("sand.automation.run"));
    assert!(!ELECTRON.contains("sand.automation.run"));
}

struct ShippingAutomation {
    root: std::path::PathBuf,
    workers: Arc<ProductionSessionWorkers>,
    runtime: AutomationRuntime,
    store: FileAutomationStore,
    agent_id: String,
    automation_id: String,
    observed: Arc<Mutex<Vec<AutomationRunObservation>>>,
}

impl ShippingAutomation {
    fn new(scenario: &str) -> Self {
        let root = root();
        let agents_root = root.join("agents");
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(&agents_root, 500));
        let sessions = SandAgentSessionStore::new(Arc::clone(&workers));
        let agent = sessions.create_session(None, "user", None).unwrap_or_else(|error| {
            // Inspect only this owned fixture, after the original failure. This
            // is diagnostic evidence, never a retry or a substitute Session.
            let metadata = [&root, &agents_root].map(|path| {
                let state = match fs::symlink_metadata(path) {
                    Ok(value) => format!(
                        "directory={}, symlink={}",
                        value.is_dir(), value.file_type().is_symlink(),
                    ),
                    Err(error) => format!(
                        "kind={:?}, os={:?}", error.kind(), error.raw_os_error(),
                    ),
                };
                format!("{path:?}: {state}")
            });
            panic!("agent setup failed for {scenario}: {error}; owned fixture metadata: {metadata:?}");
        });
        let runtime = AutomationRuntime::new(Arc::clone(&workers));
        let (created, _) = runtime.create_agent_automation(&agent.id, &AutomationSpec {
            name: "History acknowledgment".into(),
            prompt: "Inspect the queue".into(),
            trigger: json!({"type":"group","listeners":[
                {"type":"cron","schedule":"@daily"},
                {"type":"github","repo":"org/repo","events":["pr-opened"]}
            ]}),
            is_enabled: Some(true),
        }).expect("create automation");
        let store = sessions.automation_store_for(&agent.id).expect("admitted automation store");
        let observed = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
        let sink = Arc::clone(&observed);
        runtime.run_path().set_run_reporter(Some(Arc::new(move |report| {
            sink.lock().unwrap().push(report.clone());
        })));
        Self {
            root,
            workers,
            runtime,
            store,
            agent_id: agent.id,
            automation_id: created[0].id.clone(),
            observed,
        }
    }

    fn run<Execute>(
        &self,
        trigger: AutomationRunTrigger,
        run_id: &str,
        execute: Execute,
    ) -> Result<Option<FireAutomationOutcome>, String>
    where
        Execute: FnOnce(&str) -> Result<AutomationExecutionResult, String>,
    {
        match trigger {
            AutomationRunTrigger::Manual => self.runtime.run_agent_automation_now_with(
                &self.agent_id, &self.automation_id, execute,
            ),
            AutomationRunTrigger::Schedule => self.runtime.run_server_scheduled_automation_with(
                &self.agent_id, &self.automation_id, run_id.to_string(), None, execute,
            ),
            AutomationRunTrigger::Event => self.runtime.run_background_automation_with(
                &self.agent_id,
                &self.automation_id,
                trigger,
                vec![json!({"source":"github","kind":"pr-opened","repo":"org/repo","actor":"reviewer","title":"Check"})],
                Some(run_id.to_string()),
                Vec::new(),
                2_000.0,
                None,
                execute,
            ),
        }
    }

    fn write_failure_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(format!(
            "{}.{}.tmp", self.store.runs_path(&self.automation_id).display(), std::process::id()
        ))
    }

    fn inject_history_fault(&self, fault: HistoryFault) {
        let path = self.store.runs_path(&self.automation_id);
        match fault {
            HistoryFault::MissingFile => fs::remove_file(path).expect("remove admitted history"),
            HistoryFault::MissingRun => fs::write(path, b"[ ]\n").expect("remove admitted run"),
            HistoryFault::CorruptJson => fs::write(path, b"[ malformed").expect("corrupt history"),
            HistoryFault::MixedRows | HistoryFault::DuplicateIds => {
                let mut rows: Vec<serde_json::Value> = serde_json::from_slice(
                    &fs::read(&path).expect("admitted history bytes")
                ).expect("admitted history array");
                let extra = if matches!(fault, HistoryFault::MixedRows) {
                    json!({"id":"invalid","startedAt":"invalid"})
                } else {
                    rows[0].clone()
                };
                rows.push(extra);
                fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).expect("history fault");
            }
            HistoryFault::WriteFailure => {
                // The actual writer fails before replacing the original file.
                fs::create_dir(self.write_failure_path()).expect("block temporary file creation");
            }
        }
    }
}

impl Drop for ShippingAutomation {
    fn drop(&mut self) {
        self.runtime.dispose();
        self.workers.shutdown();
        if std::thread::panicking() {
            eprintln!("[automation-fixture] retained unwinding fixture pid={} root={}",
                std::process::id(), self.root.display());
        } else {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum HistoryFault {
    MissingFile,
    MissingRun,
    CorruptJson,
    MixedRows,
    DuplicateIds,
    WriteFailure,
}

fn clear_fences() -> AutomationRunFenceSnapshot {
    AutomationRunFenceSnapshot {
        active_controller: false,
        pending_outgoing_media: false,
        unsettled_subagent: false,
    }
}

fn history_admission_failure_then_independent_run(
    trigger: AutomationRunTrigger,
    fault: HistoryFault,
) {
    let fixture = ShippingAutomation::new(&format!("history-admission/{trigger:?}/{fault:?}"));
    let runs_path = fixture.store.runs_path(&fixture.automation_id);
    let prior_history = br#"[ { "id": "prior-terminal", "trigger": "manual", "startedAt": 1000, "finishedAt": 1100, "status": "ok", "detail": "preserve historical result" } ]"#;
    fs::write(&runs_path, prior_history).expect("seed the original actual history");
    let prior_runs = fixture.store.read_runs(&fixture.automation_id);
    assert_eq!(prior_runs.len(), 1);
    assert!(fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap().is_empty());
    fixture.inject_history_fault(fault);
    let refused_bytes = fs::read(&runs_path).expect("capture invalid input before admission");
    let mut executed = 0;
    let error = fixture.run(trigger, "admission-refused", |_| {
        executed += 1;
        Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
    }).expect_err("pre-existing invalid history must refuse the actual production executor");
    let expected_error = match fault {
        HistoryFault::CorruptJson => "automation store json error",
        HistoryFault::MixedRows if trigger.is_background() => "history contains invalid delivery custody",
        HistoryFault::MixedRows => "history contains an invalid row",
        HistoryFault::DuplicateIds if trigger.is_background() => "run identity is claimed by different history rows",
        HistoryFault::DuplicateIds => "history contains duplicate run IDs",
        _ => panic!("unsupported history-admission control"),
    };
    assert!(error.contains(expected_error), "{trigger:?}/{fault:?}: expected {expected_error:?}, got {error:?}");
    assert_eq!(executed, 0, "{trigger:?}/{fault:?}");
    assert_eq!(fs::read(&runs_path).unwrap(), refused_bytes, "{trigger:?}/{fault:?}");
    assert!(fixture.observed.lock().unwrap().is_empty());
    assert!(fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap().is_empty(),
        "history refusal must precede real cron registration");
    assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
    // Background identity precheck precedes bookkeeping; manual fresh-history
    // admission follows it. Neither path claims cross-file rollback.

    fs::write(&runs_path, prior_history).expect("explicitly repair the invalid stored input");
    let repaired = fixture.run(trigger, "admission-repaired", |_| {
        executed += 1;
        let runs = fixture.store.read_runs(&fixture.automation_id);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].status, "running");
        assert!(!runs[0].id.is_empty());
        assert!(runs[0].finished_at.is_none());
        assert_eq!(runs[1], prior_runs[0]);
        Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
    }).expect("one independently admitted run after explicit repair");
    assert_eq!(repaired, Some(FireAutomationOutcome::Ok));
    assert_eq!(executed, 1);
    let observed = fixture.observed.lock().unwrap().clone();
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].trigger, trigger);
    assert_eq!(observed[0].outcome, FireAutomationOutcome::Ok);
    let run_id = observed[0].automation_run_id.as_deref().expect("new committed history identity");
    assert!(!run_id.is_empty());
    assert_ne!(run_id, "admission-refused");
    let runs = fixture.store.read_runs(&fixture.automation_id);
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].id, run_id);
    assert_eq!(runs[0].status, "ok");
    assert!(runs[0].finished_at.is_some());
    assert_eq!(runs[1], prior_runs[0]);
    assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
    let rows = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
    if trigger.is_background() {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].run_id, run_id);
        assert_eq!(rows[0].state, "terminal");
        assert_eq!(rows[0].terminal_outcome.as_deref(), Some("ok"));
        assert!(rows[0].fence_state_known);
        assert!(!rows[0].active_controller && !rows[0].pending_outgoing_media && !rows[0].unsettled_subagent);
    } else {
        assert!(rows.is_empty());
    }
}

#[test]
fn real_manual_history_admission_refuses_corrupt_input_before_execution() {
    bounded_automation_case("real_manual_history_admission_refuses_corrupt_input_before_execution", || {
        for fault in [HistoryFault::CorruptJson, HistoryFault::MixedRows, HistoryFault::DuplicateIds] {
            history_admission_failure_then_independent_run(AutomationRunTrigger::Manual, fault);
        }
    });
}

#[test]
fn real_scheduled_history_admission_refuses_corrupt_input_before_execution() {
    bounded_automation_case("real_scheduled_history_admission_refuses_corrupt_input_before_execution", || {
        for fault in [HistoryFault::CorruptJson, HistoryFault::MixedRows, HistoryFault::DuplicateIds] {
            history_admission_failure_then_independent_run(AutomationRunTrigger::Schedule, fault);
        }
    });
}

#[test]
fn real_event_history_admission_refuses_corrupt_input_before_execution() {
    bounded_automation_case("real_event_history_admission_refuses_corrupt_input_before_execution", || {
        for fault in [HistoryFault::CorruptJson, HistoryFault::MixedRows, HistoryFault::DuplicateIds] {
            history_admission_failure_then_independent_run(AutomationRunTrigger::Event, fault);
        }
    });
}

fn history_failure_then_independent_run(
    trigger: AutomationRunTrigger,
    fault: HistoryFault,
    execution: Result<AutomationExecutionResult, String>,
    expected_cron_outcome: &str,
    known_fences: bool,
) {
    eprintln!("[automation-fixture] history scenario pid={} trigger={trigger:?} fault={fault:?}",
        std::process::id());
    let fixture = ShippingAutomation::new(&format!("history/{trigger:?}/{fault:?}"));
    let runs_path = fixture.store.runs_path(&fixture.automation_id);
    let config_path = fixture.store.config_path(&fixture.automation_id);
    let mut executed = 0;
    let mut failed_run_id = String::new();
    let mut admitted_history = Vec::new();
    let mut admitted_config = Vec::new();
    let mut failure_history = None;
    let error = fixture.run(trigger, "faulted-run", |_| {
        executed += 1;
        let runs = fixture.store.read_runs(&fixture.automation_id);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, "running");
        assert!(runs[0].finished_at.is_none());
        failed_run_id = runs[0].id.clone();
        if trigger.is_background() {
            assert_eq!(failed_run_id, "faulted-run");
            let rows = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
            assert_eq!(rows.len(), 1, "real owner registration must precede execution");
            assert_eq!(rows[0].run_id, failed_run_id);
            assert_eq!(rows[0].state, "running");
            assert!(!rows[0].fence_state_known);
        }
        // Admission legitimately updates lastRunAt; preservation starts here.
        admitted_history = fs::read(&runs_path).unwrap();
        admitted_config = fs::read(&config_path).unwrap();
        fixture.inject_history_fault(fault);
        if !matches!(fault, HistoryFault::MissingFile) {
            failure_history = Some(fs::read(&runs_path).unwrap());
        }
        execution
    }).expect_err("history failure must propagate through the admitted production path");

    match fault {
        HistoryFault::MissingFile | HistoryFault::MissingRun => {
            assert_eq!(error, "automation history run settlement was not committed");
        }
        HistoryFault::CorruptJson => assert!(error.contains("automation store json error")),
        HistoryFault::MixedRows => assert!(error.contains("history contains an invalid row")),
        HistoryFault::DuplicateIds => assert!(error.contains("history contains duplicate run IDs")),
        HistoryFault::WriteFailure => assert!(error.contains("automation store io error")),
    }
    assert_eq!(executed, 1);
    assert!(fixture.observed.lock().unwrap().is_empty(), "{trigger:?}/{fault:?}");
    assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
    assert_eq!(fs::read(&config_path).unwrap(), admitted_config);
    if let Some(bytes) = failure_history {
        assert_eq!(fs::read(&runs_path).unwrap(), bytes, "{trigger:?}/{fault:?}");
    } else {
        assert!(!runs_path.exists(), "missing history must remain absent");
    }

    let failed_rows = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
    if trigger.is_background() {
        assert_eq!(failed_rows.len(), 1);
        let row = &failed_rows[0];
        assert_eq!(row.run_id, failed_run_id);
        assert_eq!(row.automation_id, fixture.automation_id);
        assert_eq!(row.session_key, format!(
            "agent:{}:cron:{}:{}", fixture.agent_id, fixture.automation_id, failed_run_id
        ));
        assert_eq!(row.session_id, row.session_key);
        assert!(!row.lifecycle_revision.is_empty());
        assert_eq!(row.state, "terminal", "cron owner commits before compatibility history");
        assert_eq!(row.terminal_outcome.as_deref(), Some(expected_cron_outcome));
        assert_eq!(row.fence_state_known, known_fences);
        assert!(!row.active_controller && !row.pending_outgoing_media && !row.unsettled_subagent);
    } else {
        assert!(failed_rows.is_empty());
    }

    // Repair the fixture, then admit a different run. Never replay the failed
    // run or treat a possibly committed execution as rolled back.
    if matches!(fault, HistoryFault::WriteFailure) {
        assert!(fixture.write_failure_path().is_dir());
        fs::remove_dir(fixture.write_failure_path()).expect("repair writer input");
    }
    fs::write(&runs_path, admitted_history).expect("repair history input");
    let repaired = fixture.run(trigger, "repaired-run", |_| {
        executed += 1;
        Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
    }).expect("independent run after repair");
    assert_eq!(repaired, Some(FireAutomationOutcome::Ok));
    assert_eq!(executed, 2);
    let observed = fixture.observed.lock().unwrap().clone();
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].outcome, FireAutomationOutcome::Ok);
    assert_eq!(observed[0].trigger, trigger);
    let repaired_run_id = observed[0].automation_run_id.as_deref().expect("new run identity");
    assert_ne!(repaired_run_id, failed_run_id);
    let runs = fixture.store.read_runs(&fixture.automation_id);
    let repaired_run = runs.iter().find(|run| run.id == repaired_run_id).expect("persisted new run");
    assert_eq!(repaired_run.status, "ok");
    assert!(repaired_run.finished_at.is_some());
    assert_eq!(runs.iter().find(|run| run.id == failed_run_id).unwrap().status, "running");
    assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
    let repaired_rows = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
    if trigger.is_background() {
        assert_eq!(repaired_rows.len(), 2);
        assert_eq!(repaired_rows.iter().find(|row| row.run_id == failed_run_id), failed_rows.first());
        let row = repaired_rows.iter().find(|row| row.run_id == repaired_run_id).unwrap();
        assert_eq!(row.state, "terminal");
        assert_eq!(row.terminal_outcome.as_deref(), Some("ok"));
        assert!(row.fence_state_known);
    } else {
        assert!(repaired_rows.is_empty());
    }
}

#[test]
fn real_manual_history_failures_suppress_observations_and_release_the_next_run() {
    bounded_automation_case("real_manual_history_failures_suppress_observations_and_release_the_next_run", || {
        history_failure_then_independent_run(
            AutomationRunTrigger::Manual,
            HistoryFault::MissingRun,
            Ok(AutomationExecutionResult::Completed),
            "ok",
            false,
        );
        history_failure_then_independent_run(
            AutomationRunTrigger::Manual,
            HistoryFault::CorruptJson,
            Ok(AutomationExecutionResult::Failed { detail: "provider failed".into() }),
            "error",
            false,
        );
    });
}

#[test]
fn real_background_history_failures_preserve_terminal_owner_rows_and_allow_a_new_run() {
    bounded_automation_case("real_background_history_failures_preserve_terminal_owner_rows_and_allow_a_new_run", || {
        history_failure_then_independent_run(
            AutomationRunTrigger::Schedule,
            HistoryFault::MissingFile,
            Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() }),
            "ok",
            true,
        );
        history_failure_then_independent_run(
            AutomationRunTrigger::Event,
            HistoryFault::MixedRows,
            Ok(AutomationExecutionResult::FailedWithFences {
                detail: "provider failed".into(), fences: clear_fences(),
            }),
            "error",
            true,
        );
        history_failure_then_independent_run(
            AutomationRunTrigger::Schedule,
            HistoryFault::DuplicateIds,
            Ok(AutomationExecutionResult::InterruptedWithFences {
                detail: "host upgrade".into(), quiesced_for_upgrade: true, fences: clear_fences(),
            }),
            "interrupted",
            true,
        );
        history_failure_then_independent_run(
            AutomationRunTrigger::Event,
            HistoryFault::WriteFailure,
            Err("executor failed".into()),
            "error",
            false,
        );
    });
}

#[test]
fn real_background_execution_failures_and_interruptions_report_when_history_commits() {
    bounded_automation_case("real_background_execution_failures_and_interruptions_report_when_history_commits", || {
        for (trigger, execution, expected_outcome, expected_detail, known_fences) in [
            (AutomationRunTrigger::Event, Ok(AutomationExecutionResult::FailedWithFences {
                detail: "provider failed".into(), fences: clear_fences(),
            }), FireAutomationOutcome::Error, "provider failed", true),
            (AutomationRunTrigger::Schedule, Ok(AutomationExecutionResult::InterruptedWithFences {
                detail: "host upgrade".into(), quiesced_for_upgrade: true, fences: clear_fences(),
            }), FireAutomationOutcome::Interrupted, "host upgrade", true),
            (AutomationRunTrigger::Event, Err("executor failed".to_string()),
             FireAutomationOutcome::Error, "executor failed", false),
        ] {
            let fixture = ShippingAutomation::new(&format!("terminal/{trigger:?}/{expected_outcome:?}"));
            let outcome = fixture.run(trigger, "terminal-run", |_| execution).expect("settled execution");
            assert_eq!(outcome, Some(expected_outcome));
            let runs = fixture.store.read_runs(&fixture.automation_id);
            assert_eq!(runs.len(), 1);
            assert_eq!(runs[0].id, "terminal-run");
            assert_eq!(runs[0].status, "error");
            assert_eq!(runs[0].detail.as_deref(), Some(expected_detail));
            assert!(runs[0].finished_at.is_some());
            let observed = fixture.observed.lock().unwrap().clone();
            assert_eq!(observed.len(), 1);
            assert_eq!(observed[0].outcome, expected_outcome);
            assert_eq!(observed[0].quiesced_for_upgrade, expected_outcome == FireAutomationOutcome::Interrupted);
            let rows = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].state, "terminal");
            assert_eq!(rows[0].terminal_outcome.as_deref(), Some(
                if expected_outcome == FireAutomationOutcome::Interrupted { "interrupted" } else { "error" }
            ));
            assert_eq!(rows[0].terminal_detail.as_deref(), Some(expected_detail));
            assert_eq!(rows[0].fence_state_known, known_fences);
        }
    });
}

#[test]
fn real_cron_database_lifecycle_preserves_identity_terminal_and_admission_fences() {
    bounded_automation_case("real_cron_database_lifecycle_preserves_identity_terminal_and_admission_fences", || {
        let fixture = ShippingAutomation::new("cron database lifecycle");
        let workers = &fixture.workers;
        let agent = &fixture.agent_id;
        let automation = &fixture.automation_id;
        let owner = workers.open_agent_db_owner(agent).expect("existing Session DB owner");
        let admission = owner.read_session_work_admission().unwrap().expect("main admission");
        let revision = &admission.lifecycle_revision;
        let key = |run: &str| format!("agent:{agent}:cron:{automation}:{run}");
        let plain_key = key("plain");

        assert!(workers.begin_cron_session_run(agent, automation, "plain", 10.0).unwrap());
        assert!(owner.begin_cron_session_run(
            &plain_key, automation, "plain", &plain_key, revision, 11.0,
        ).unwrap(), "an exact running duplicate may refresh its timestamp");
        let running = owner.list_cron_session_runs().unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].updated_at_ms, 11.0);
        assert_eq!(running[0].state, "running");
        assert!(!running[0].fence_state_known);
        for (other_automation, other_run, other_session, other_revision) in [
            ("other-automation", "plain", plain_key.as_str(), revision.as_str()),
            (automation.as_str(), "other-run", plain_key.as_str(), revision.as_str()),
            (automation.as_str(), "plain", "other-session", revision.as_str()),
            (automation.as_str(), "plain", plain_key.as_str(), "stale-revision"),
        ] {
            assert!(!owner.begin_cron_session_run(
                &plain_key, other_automation, other_run, other_session, other_revision, 99.0,
            ).unwrap(), "a conflicting duplicate must not rebind the registered identity");
        }
        for (session, lifecycle) in [
            ("other-session", revision.as_str()),
            (plain_key.as_str(), "stale-revision"),
        ] {
            assert!(!owner.finish_cron_session_run(
                &plain_key, session, lifecycle, "ok", 99.0, None,
            ).unwrap());
            assert!(!owner.publish_cron_session_run_fences(
                &plain_key, session, lifecycle, true, true, true,
            ).unwrap());
            assert!(!owner.finish_cron_session_run_with_fences(
                &plain_key, session, lifecycle, "ok", 99.0, None, true, true, true,
            ).unwrap());
        }
        assert_eq!(owner.list_cron_session_runs().unwrap(), running);

        assert!(workers.publish_cron_session_run_fences(
            agent, automation, "plain", true, false, true,
        ).unwrap());
        let published = owner.list_cron_session_runs().unwrap();
        assert_eq!(published[0].state, "running");
        assert_eq!(published[0].updated_at_ms, 11.0);
        assert!(published[0].fence_state_known);
        assert!(published[0].active_controller && published[0].unsettled_subagent);
        assert!(!published[0].pending_outgoing_media);
        assert!(workers.finish_cron_session_run(
            agent, automation, "plain", "error", 20.0, Some("executor failed"),
        ).unwrap());
        let terminal = owner.list_cron_session_runs().unwrap();
        assert_eq!(terminal[0].state, "terminal");
        assert_eq!(terminal[0].updated_at_ms, 20.0);
        assert_eq!(terminal[0].terminal_outcome.as_deref(), Some("error"));
        assert_eq!(terminal[0].terminal_detail.as_deref(), Some("executor failed"));
        assert!(terminal[0].fence_state_known);
        assert!(terminal[0].active_controller && terminal[0].unsettled_subagent);
        assert!(!terminal[0].pending_outgoing_media);
        assert!(!workers.begin_cron_session_run(agent, automation, "plain", 99.0).unwrap());
        assert!(!workers.finish_cron_session_run(
            agent, automation, "plain", "ok", 99.0, None,
        ).unwrap());
        assert!(!workers.publish_cron_session_run_fences(
            agent, automation, "plain", false, false, false,
        ).unwrap());
        assert!(!workers.finish_cron_session_run_with_fences(
            agent, automation, "plain", "ok", 99.0, None, false, false, false,
        ).unwrap());
        assert_eq!(owner.list_cron_session_runs().unwrap(), terminal);

        assert!(workers.begin_cron_session_run(agent, automation, "combined", 30.0).unwrap());
        assert!(workers.finish_cron_session_run_with_fences(
            agent, automation, "combined", "ok", 40.0, None, false, true, false,
        ).unwrap());
        let rows = workers.list_cron_session_runs(agent).unwrap();
        let combined = rows.iter().find(|row| row.run_id == "combined").unwrap();
        assert_eq!(combined.state, "terminal");
        assert_eq!(combined.updated_at_ms, 40.0);
        assert_eq!(combined.terminal_outcome.as_deref(), Some("ok"));
        assert_eq!(combined.terminal_detail, None);
        assert!(combined.fence_state_known && combined.pending_outgoing_media);
        assert!(!combined.active_controller && !combined.unsettled_subagent);

        // The production facade must use the current admission generation,
        // rather than finishing a row retained from an earlier generation.
        assert!(workers.begin_cron_session_run(agent, automation, "stale", 50.0).unwrap());
        let mut next_admission = admission.clone();
        next_admission.lifecycle_revision = "next-automation-generation".into();
        assert!(owner.compare_and_set_session_work_admission(Some(revision), &next_admission).unwrap());
        let before_stale = owner.list_cron_session_runs().unwrap();
        assert!(!workers.begin_cron_session_run(agent, automation, "stale", 99.0).unwrap());
        assert!(!workers.finish_cron_session_run(
            agent, automation, "stale", "ok", 99.0, None,
        ).unwrap());
        assert!(!workers.publish_cron_session_run_fences(
            agent, automation, "stale", true, true, true,
        ).unwrap());
        assert!(!workers.finish_cron_session_run_with_fences(
            agent, automation, "stale", "ok", 99.0, None, true, true, true,
        ).unwrap());
        assert_eq!(owner.list_cron_session_runs().unwrap(), before_stale);

        assert!(workers.begin_cron_session_run(agent, automation, "deleting", 60.0).unwrap());
        let before_deletion = owner.list_cron_session_runs().unwrap();
        workers.begin_agent_delete(agent).expect("publish durable deletion custody");
        assert!(owner.read_session_work_admission().unwrap().unwrap().deletion_pending);
        for refused in [
            workers.begin_cron_session_run(agent, automation, "blocked", 99.0),
            workers.finish_cron_session_run(agent, automation, "deleting", "ok", 99.0, None),
            workers.publish_cron_session_run_fences(agent, automation, "deleting", true, true, true),
            workers.finish_cron_session_run_with_fences(
                agent, automation, "deleting", "ok", 99.0, None, true, true, true,
            ),
        ] {
            assert!(refused.expect_err("public facade must refuse deletion custody")
                .contains("new session work is not admitted"));
        }
        // A retained DB handle is independently fenced by the durable record,
        // so avoiding the public fast-path check cannot authorize a write.
        let deleting_key = key("deleting");
        let next_revision = &next_admission.lifecycle_revision;
        assert!(!owner.begin_cron_session_run(
            &key("blocked"), automation, "blocked", &key("blocked"), next_revision, 99.0,
        ).unwrap());
        assert!(!owner.finish_cron_session_run(
            &deleting_key, &deleting_key, next_revision, "ok", 99.0, None,
        ).unwrap());
        assert!(!owner.publish_cron_session_run_fences(
            &deleting_key, &deleting_key, next_revision, true, true, true,
        ).unwrap());
        assert!(!owner.finish_cron_session_run_with_fences(
            &deleting_key, &deleting_key, next_revision, "ok", 99.0, None, true, true, true,
        ).unwrap());
        assert_eq!(owner.list_cron_session_runs().unwrap(), before_deletion);
        workers.end_agent_delete(agent).expect("release only this fixture's deletion custody");
    });
}

#[derive(Debug, Clone, PartialEq)]
struct CustodyPersistedState {
    config: Vec<u8>,
    history: Vec<u8>,
    cron: Vec<CronSessionRunRecord>,
}

impl CustodyPersistedState {
    fn capture(
        workers: &ProductionSessionWorkers,
        store: &FileAutomationStore,
        agent_id: &str,
        automation_id: &str,
    ) -> Self {
        Self {
            config: fs::read(store.config_path(automation_id)).expect("custody definition bytes"),
            history: fs::read(store.runs_path(automation_id)).expect("custody history bytes"),
            cron: workers.list_cron_session_runs(agent_id).expect("actual custody cron owner"),
        }
    }

    fn preserve_evidence(&self, root: &Path, phase: &str) {
        fs::write(root.join(format!("{phase}-config.json")), &self.config).unwrap();
        fs::write(root.join(format!("{phase}-history.json")), &self.history).unwrap();
        fs::write(root.join(format!("{phase}-cron.txt")), format!("{:#?}", self.cron)).unwrap();
    }
}

#[test]
fn real_retained_terminal_preserves_post_admission_state_and_volatile_release() {
    bounded_automation_case(
        "real_retained_terminal_preserves_post_admission_state_and_volatile_release",
        || {
            for trigger in [AutomationRunTrigger::Manual, AutomationRunTrigger::Schedule] {
                for terminal in [
                    AutomationExecutionResult::CompletedWithFences { fences: clear_fences() },
                    AutomationExecutionResult::FailedWithFences {
                        detail: "provider failed".into(), fences: clear_fences(),
                    },
                    AutomationExecutionResult::InterruptedWithFences {
                        detail: "host upgrade".into(), quiesced_for_upgrade: true,
                        fences: clear_fences(),
                    },
                ] {
                    let fixture = ShippingAutomation::new(&format!("retained/{trigger:?}/{terminal:?}"));
                    let path = fixture.runtime.run_path();
                    assert_eq!(path.record_automation_failure(
                        &fixture.agent_id, &fixture.automation_id, "provider failed",
                        AutomationRunTrigger::Manual,
                    ), Some(1));
                    let mut entered = 0;
                    let mut admitted = None;
                    let result = fixture.run(trigger, "retained-run", |_| {
                        entered += 1;
                        assert!(path.is_in_flight(&fixture.agent_id, &fixture.automation_id));
                        let runs = fixture.store.read_runs(&fixture.automation_id);
                        assert_eq!(runs.len(), 1, "actual history admission precedes the executor");
                        assert!(!runs[0].id.is_empty());
                        assert_eq!(runs[0].status, "running");
                        assert!(runs[0].finished_at.is_none());
                        let state = CustodyPersistedState::capture(
                            &fixture.workers, &fixture.store, &fixture.agent_id, &fixture.automation_id,
                        );
                        if trigger.is_background() {
                            assert_eq!(runs[0].id, "retained-run");
                            assert_eq!(state.cron.len(), 1);
                            assert_eq!(state.cron[0].run_id, runs[0].id);
                            assert_eq!(state.cron[0].state, "running");
                            assert!(!state.cron[0].fence_state_known);
                            assert!(state.cron[0].terminal_outcome.is_none());
                        } else {
                            assert!(state.cron.is_empty(), "manual runs have no cron owner");
                        }
                        assert!(fixture.observed.lock().unwrap().is_empty());
                        state.preserve_evidence(&fixture.root, "post-admission");
                        admitted = Some(state);
                        // The actual guarded-tool producer and typed waiter are
                        // exercised by the Host app tests; this pins their consumer.
                        Ok(AutomationExecutionResult::RetainedUnsettled {
                            terminal: Box::new(terminal),
                        })
                    });
                    fs::write(fixture.root.join("retained-result.txt"), format!("{result:?}")).unwrap();
                    assert_eq!(result, Ok(None));
                    assert_eq!(entered, 1, "an early suspended/duplicate refusal is not this oracle");
                    assert!(!path.is_in_flight(&fixture.agent_id, &fixture.automation_id));
                    let admitted = admitted.expect("real post-admission baseline");
                    assert_eq!(CustodyPersistedState::capture(
                        &fixture.workers, &fixture.store, &fixture.agent_id, &fixture.automation_id,
                    ), admitted);
                    assert!(fixture.observed.lock().unwrap().is_empty());
                    // Resetting gives Some(1); incrementing gives the suppressed
                    // third occurrence. Only an unchanged count gives Some(2).
                    assert_eq!(path.record_automation_failure(
                        &fixture.agent_id, &fixture.automation_id, "provider failed",
                        AutomationRunTrigger::Manual,
                    ), Some(2));

                    let original = fixture.store.read_runs(&fixture.automation_id)[0].clone();
                    let next = fixture.run(trigger, "independent-run", |_| {
                        entered += 1;
                        Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
                    });
                    assert_eq!(next, Ok(Some(FireAutomationOutcome::Ok)));
                    assert_eq!(entered, 2, "the retained run is not an automation-wide fence");
                    let runs = fixture.store.read_runs(&fixture.automation_id);
                    assert_eq!(runs.len(), 2);
                    assert_eq!(runs.iter().find(|run| run.id == original.id), Some(&original));
                    let observed = fixture.observed.lock().unwrap().clone();
                    assert_eq!(observed.len(), 1);
                    assert_eq!(observed[0].outcome, FireAutomationOutcome::Ok);
                    let next_id = observed[0].automation_run_id.as_deref().unwrap();
                    assert_ne!(next_id, original.id);
                    assert_eq!(runs.iter().find(|run| run.id == next_id).unwrap().status, "ok");
                    let rows = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
                    if trigger.is_background() {
                        assert_eq!(rows.len(), 2);
                        assert_eq!(rows.iter().find(|row| row.run_id == original.id), admitted.cron.first());
                        let next_row = rows.iter().find(|row| row.run_id == next_id).unwrap();
                        assert_eq!(next_row.state, "terminal");
                        assert!(next_row.fence_state_known);
                    } else {
                        assert!(rows.is_empty());
                    }
                    assert!(!path.is_in_flight(&fixture.agent_id, &fixture.automation_id));
                }
            }
        },
    );
}

const CUSTODY_WAIT: Duration = Duration::from_secs(10);
const CUSTODY_RUN_UUIDS: [&str; 3] = [
    "00000000-0000-4000-8000-000000000101",
    "00000000-0000-4000-8000-000000000102",
    "00000000-0000-4000-8000-000000000103",
];

fn custody_receive<T>(receiver: &mpsc::Receiver<T>, phase: &str) -> T {
    receiver.recv_timeout(CUSTODY_WAIT)
        .unwrap_or_else(|error| panic!("custody {phase} receipt: {error}"))
}

struct CustodyRelease(Option<mpsc::Sender<()>>);

impl CustodyRelease {
    fn release(&mut self) {
        self.0.take().expect("owned custody gate").send(()).expect("live custody gate waiter");
    }
}

impl Drop for CustodyRelease {
    fn drop(&mut self) {
        if let Some(release) = self.0.take() {
            let _ = release.send(());
        }
    }
}

fn custody_trace(trace: &Mutex<Vec<String>>, path: &Path, entry: String) {
    let mut trace = trace.lock().unwrap();
    trace.push(entry);
    fs::write(path, trace.join("\n")).expect("preserve custody callback evidence");
}

struct CustodyBackendTransport {
    delivery: Value,
    calls: Mutex<Vec<(String, Value)>>,
    polls: AtomicUsize,
    evidence_path: PathBuf,
}

impl AutomationsBackendTransport for CustodyBackendTransport {
    fn post_json(&self, path: &str, body: &Value) -> Result<Value, AutomationsBackendError> {
        {
            let mut calls = self.calls.lock().unwrap();
            calls.push((path.to_string(), body.clone()));
            fs::write(&self.evidence_path, serde_json::to_vec_pretty(&*calls).unwrap()).unwrap();
        }
        match path {
            "/sand/listener-subscriptions" => Ok(json!({})),
            "/sand/listener-events/poll" => Ok(json!({"events":[]})),
            "/sand/automation-events/poll" => {
                self.polls.fetch_add(1, Ordering::SeqCst);
                Ok(self.delivery.clone())
            }
            _ => Err(AutomationsBackendError::Transport(format!(
                "unexpected custody backend request: {path}"
            ))),
        }
    }
}

impl CustodyBackendTransport {
    fn assert_no_completion_or_ack(&self) {
        let calls = self.calls.lock().unwrap();
        assert!(calls.iter().any(|(path, _)| path == "/sand/automation-events/poll"));
        for (path, body) in calls.iter() {
            match path.as_str() {
                "/sand/listener-subscriptions" | "/sand/listener-events/poll" => {}
                "/sand/automation-events/poll" => assert_eq!(body, &json!({"ackRunUuids":[]})),
                _ => panic!("retained original/alias must not POST completion: {path}: {body}"),
            }
        }
    }
}

struct CustodyNotifyExperiments;

impl NotifyBusExperimentsApi for CustodyNotifyExperiments {
    fn check_feature_gate(&self, _: &str) -> bool { false }
    fn subscribe(&self, _: Arc<dyn Fn() + Send + Sync>) -> StopSubscription { Box::new(|| {}) }
}

#[derive(Debug)]
struct CustodyDispatchReceipt {
    poll: usize,
    run_uuid: String,
    prepared: PreparedBackendFire,
}

struct CustodyBackend {
    runtime: Arc<ProductionAutomationsBackendRuntime>,
    notify: HostNotifyBusExtension,
    transport: Arc<CustodyBackendTransport>,
    dispatches: mpsc::Receiver<CustodyDispatchReceipt>,
    callbacks: mpsc::Receiver<(String, Result<(), String>)>,
    callback_threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
    errors: Arc<Mutex<Vec<String>>>,
}

impl CustodyBackend {
    fn start(
        root: &Path,
        phase: &str,
        automation_runtime: &AutomationRuntime,
        delivery: Value,
        executor: Option<EventBatchExecutor>,
    ) -> Self {
        let errors = Arc::new(Mutex::new(Vec::new()));
        let auth_errors = Arc::clone(&errors);
        let url_errors = Arc::clone(&errors);
        let notify_errors = Arc::clone(&errors);
        let notify = start_notify_bus_extension_with_options(
            Arc::new(CustodyNotifyExperiments),
            NotifyBusExtensionOptions {
                get_backend_url: Arc::new(move || {
                    url_errors.lock().unwrap().push("unexpected notify URL lookup".into());
                    Err("custody fixture must not start an external notify stream".into())
                }),
                get_access_token: Arc::new(move |_| {
                    auth_errors.lock().unwrap().push("unexpected notify credential lookup".into());
                    Err("custody fixture must not read external credentials".into())
                }),
                now_ms: Arc::new(|| 0),
                timing: NotifyBusTiming::default(),
                log: Arc::new(move |message| notify_errors.lock().unwrap().push(message.into())),
            },
        ).expect("actual notify owner without an external stream");
        notify.mark_background_work_ready();
        assert!(!notify.is_connected());
        assert!(!notify.is_safety_poll_enabled());

        let transport = Arc::new(CustodyBackendTransport {
            delivery,
            calls: Mutex::new(Vec::new()),
            polls: AtomicUsize::new(0),
            evidence_path: root.join(format!("{phase}-requests.json")),
        });
        let trace = Arc::new(Mutex::new(Vec::new()));
        let trace_path = root.join(format!("{phase}-callbacks.txt"));
        let callback_threads = Arc::new(Mutex::new(Vec::new()));
        let (dispatch_tx, dispatches) = mpsc::channel();
        let (callback_tx, callbacks) = mpsc::channel();
        let dispatch_runtime = automation_runtime.clone();
        let coalescer = automation_runtime.event_fires();
        let dispatch_transport = Arc::clone(&transport);
        let dispatch_threads = Arc::clone(&callback_threads);
        let dispatch_errors = Arc::clone(&errors);
        let dispatch: FireDispatch = Arc::new(move |fire, completion| {
            let entries = dispatch_runtime.list_all_automations().expect("actual delivery history owner");
            let prepared = prepare_backend_fire(&entries, &fire, |entry| {
                sand_cloud_definition(&entry.agent_id, &ScheduledCloudAutomation {
                    id: entry.automation.id.clone(), name: entry.automation.name.clone(),
                    prompt: entry.automation.prompt.clone(), is_enabled: entry.automation.is_enabled,
                    trigger: entry.automation.trigger.clone(),
                }, None).map(|definition| definition.hash)
            }, |entry, run_id| dispatch_runtime.read_run_for_delivery(&entry.agent_id, &entry.automation.id, run_id));
            let accepted = match &prepared {
                PreparedBackendFire::Event { agent_id, automation_id, run_uuid, event } => {
                    if let Some(executor) = executor.as_ref() {
                        let automation = entries.iter().find(|entry|
                            entry.agent_id == *agent_id && entry.automation.id == *automation_id
                        ).expect("prepared actual event definition").automation.clone();
                        let receiver = coalescer.enqueue_event_automation_fire(
                            agent_id.clone(), automation, event.clone(), Some(run_uuid.clone()),
                            Arc::clone(executor),
                        );
                        let run_uuid = run_uuid.clone();
                        let callback_tx = callback_tx.clone();
                        let errors = Arc::clone(&dispatch_errors);
                        let trace = Arc::clone(&trace);
                        let trace_path = trace_path.clone();
                        let waiter = std::thread::spawn(move || {
                            let result = (|| -> Result<(), String> {
                                let received = receiver.recv_timeout(CUSTODY_WAIT)
                                    .map_err(|error| format!("actual batch receiver: {error}"))?;
                                if received.is_some() {
                                    return Err(format!("expected retained None, received {received:?}"));
                                }
                                // This is the callback supplied by the actual
                                // ProductionAutomationsBackendRuntime. The test
                                // does not implement completion/abandon policy.
                                completion(None);
                                match receiver.recv_timeout(CUSTODY_WAIT) {
                                    Err(mpsc::RecvTimeoutError::Disconnected) => Ok(()),
                                    other => Err(format!("batch receiver did not settle once: {other:?}")),
                                }
                            })();
                            custody_trace(&trace, &trace_path, format!("callback {run_uuid}: {result:?}"));
                            if let Err(error) = &result {
                                errors.lock().unwrap().push(error.clone());
                            }
                            callback_tx.send((run_uuid, result)).expect("owned callback observer");
                        });
                        dispatch_threads.lock().unwrap().push(waiter);
                        true
                    } else {
                        dispatch_errors.lock().unwrap().push(format!("reopen attempted execution: {prepared:?}"));
                        false
                    }
                }
                PreparedBackendFire::Abandon { .. } => false,
                _ => {
                    dispatch_errors.lock().unwrap().push(format!("unexpected custody preparation: {prepared:?}"));
                    false
                }
            };
            let receipt = CustodyDispatchReceipt {
                poll: dispatch_transport.polls.load(Ordering::SeqCst),
                run_uuid: fire.id,
                prepared,
            };
            custody_trace(&trace, &trace_path, format!("dispatch {receipt:?}; accepted={accepted}"));
            dispatch_tx.send(receipt).expect("owned dispatch observer");
            accepted
        });
        let state_runtime = automation_runtime.clone();
        let relay_errors = Arc::clone(&errors);
        let poll_errors = Arc::clone(&errors);
        let log_errors = Arc::clone(&errors);
        let runtime = ProductionAutomationsBackendRuntime::start(
            transport.clone(), notify.clone(), Arc::new(|| (Vec::new(), Vec::new())),
            Arc::new(move |_| {
                relay_errors.lock().unwrap().push("unexpected listener event".into());
                false
            }),
            dispatch,
            Arc::new(move || {
                let schedulable = state_runtime.list_all_automation_definitions()
                    .expect("actual poll eligibility definitions").iter()
                    .any(|entry| entry.automation.is_enabled && trigger_schedule(&entry.automation.trigger).is_some());
                (!state_runtime.wakes_suspended(), schedulable)
            }),
            Arc::new(move |error| poll_errors.lock().unwrap().push(error.to_string())),
            Arc::new(move |message| log_errors.lock().unwrap().push(message.into())),
        );
        Self { runtime, notify, transport, dispatches, callbacks, callback_threads, errors }
    }

    fn assert_abandoned_after(&self, earlier_poll: usize) {
        self.assert_abandoned_with_reason_after(earlier_poll, "existing_run_unsettled");
    }

    fn assert_abandoned_with_reason_after(&self, earlier_poll: usize, reason: &'static str) {
        let deadline = Instant::now() + CUSTODY_WAIT;
        let mut seen = std::collections::BTreeSet::new();
        while seen.len() < CUSTODY_RUN_UUIDS.len() {
            let receipt = self.dispatches.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("each actual backend UUID must redeliver after its None callback");
            assert!(CUSTODY_RUN_UUIDS.contains(&receipt.run_uuid.as_str()));
            assert_eq!(receipt.prepared, PreparedBackendFire::Abandon { reason });
            if receipt.poll > earlier_poll {
                seen.insert(receipt.run_uuid);
            }
        }
        assert_eq!(seen, CUSTODY_RUN_UUIDS.map(str::to_owned).into_iter().collect());
        self.transport.assert_no_completion_or_ack();
        let errors = self.errors.lock().unwrap().clone();
        assert!(errors.is_empty(), "{errors:?}");
    }

    fn join_callbacks(&self) {
        let threads = std::mem::take(&mut *self.callback_threads.lock().unwrap());
        for thread in threads {
            thread.join().expect("actual per-fire callback waiter");
        }
    }
}

impl Drop for CustodyBackend {
    fn drop(&mut self) {
        self.runtime.stop();
        self.notify.stop();
        let threads = std::mem::take(&mut *self.callback_threads.lock().unwrap());
        for thread in threads {
            let _ = thread.join();
        }
    }
}

struct ReopenedCustodyOwners {
    workers: Arc<ProductionSessionWorkers>,
    runtime: AutomationRuntime,
    store: FileAutomationStore,
    observed: Arc<Mutex<Vec<AutomationRunObservation>>>,
}

impl ReopenedCustodyOwners {
    fn open(root: &Path, agent_id: &str) -> Self {
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let store = SandAgentSessionStore::new(Arc::clone(&workers))
            .automation_store_for(agent_id).expect("reopened existing admitted file owner");
        let runtime = AutomationRuntime::new(Arc::clone(&workers));
        let observed = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&observed);
        runtime.run_path().set_run_reporter(Some(Arc::new(move |report| {
            sink.lock().unwrap().push(report.clone());
        })));
        Self { workers, runtime, store, observed }
    }
}

impl Drop for ReopenedCustodyOwners {
    fn drop(&mut self) {
        self.runtime.dispose();
        self.workers.shutdown();
    }
}

fn wait_for_custody_batch_to_drain(coalescer: &AutomationEventFires) {
    let deadline = Instant::now() + CUSTODY_WAIT;
    while coalescer.pending_batch_count() != 0 {
        assert!(Instant::now() < deadline, "actual coalescer did not retire its completed batch");
        // Result delivery precedes removing the drained key. This observes
        // cleanup only; enqueue ordering is proved by the held executor below.
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn real_coalesced_retained_event_reaches_each_backend_callback_and_survives_reopen() {
    bounded_automation_case(
        "real_coalesced_retained_event_reaches_each_backend_callback_and_survives_reopen",
        || retained_coalesced_event_survives_reopen(0),
    );
}

#[test]
fn real_retained_run_survives_twenty_manual_settlements_and_reopened_backend_redelivery() {
    bounded_automation_case(
        "real_retained_run_survives_twenty_manual_settlements_and_reopened_backend_redelivery",
        || retained_coalesced_event_survives_reopen(AUTOMATION_MAX_RUN_HISTORY),
    );
}

fn retained_coalesced_event_survives_reopen(manual_runs: usize) {
            let fixture = ShippingAutomation::new("retained coalesced backend delivery");
            let automation = fixture.store.get(&fixture.automation_id).expect("actual event definition");
            assert!(automation.runs.is_empty());
            assert!(fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap().is_empty());
            let coalescer = fixture.runtime.event_fires();

            let (primer_entered_tx, primer_entered) = mpsc::channel();
            let (primer_release_tx, primer_release_rx) = mpsc::channel();
            let primer_release_rx = Mutex::new(primer_release_rx);
            let primer = coalescer.enqueue_event_automation_fire(
                fixture.agent_id.clone(), automation, json!({"fixture":"held scheduling batch"}),
                None,
                Arc::new(move |batch| {
                    assert_eq!(batch.events, vec![json!({"fixture":"held scheduling batch"})]);
                    assert!(batch.run_uuid.is_none() && batch.coalesced_run_uuids.is_empty());
                    primer_entered_tx.send(()).unwrap();
                    primer_release_rx.lock().unwrap().recv_timeout(CUSTODY_WAIT)
                        .expect("release the owned scheduling batch");
                    // No Runtime, durable admission or tool runs for this
                    // scheduling control; its Some result is not custody proof.
                    Ok(Some(FireAutomationOutcome::Ok))
                }),
            );
            custody_receive(&primer_entered, "real coalescer flushing gate");

            let executions = Arc::new(AtomicUsize::new(0));
            let (batch_tx, batches) = mpsc::channel();
            let (admitted_tx, admission) = mpsc::channel();
            let (result_tx, results) = mpsc::channel();
            let (terminal_release_tx, terminal_release_rx) = mpsc::channel();
            let terminal_release_rx = Mutex::new(terminal_release_rx);
            let run_runtime = fixture.runtime.clone();
            let run_workers = Arc::clone(&fixture.workers);
            let run_store = fixture.store.clone();
            let run_agent = fixture.agent_id.clone();
            let run_automation = fixture.automation_id.clone();
            let run_root = fixture.root.clone();
            let run_executions = Arc::clone(&executions);
            let executor: EventBatchExecutor = Arc::new(move |batch: EventFireBatch| {
                fs::write(run_root.join("actual-event-batch.txt"), format!("{batch:#?}")).unwrap();
                batch_tx.send(batch.clone()).expect("actual batch observer");
                let result = run_runtime.run_background_automation_with(
                    &batch.agent_id, &batch.automation.id, AutomationRunTrigger::Event,
                    batch.events, batch.run_uuid, batch.coalesced_run_uuids, 2_000.0, None,
                    |prompt| {
                        assert_eq!(run_executions.fetch_add(1, Ordering::SeqCst), 0);
                        assert!(prompt.contains("3 events"));
                        let runs = run_store.read_runs(&run_automation);
                        assert_eq!(runs.len(), 1, "real admitted alias batch has one history row");
                        assert_eq!(runs[0].id, CUSTODY_RUN_UUIDS[0]);
                        assert_eq!(runs[0].coalesced_run_ids, Some(
                            CUSTODY_RUN_UUIDS[1..].iter().map(|id| (*id).to_string()).collect::<Vec<_>>()
                        ));
                        assert_eq!(runs[0].status, "running");
                        assert!(runs[0].finished_at.is_none());
                        let state = CustodyPersistedState::capture(
                            &run_workers, &run_store, &run_agent, &run_automation,
                        );
                        assert_eq!(state.cron.len(), 1);
                        assert_eq!(state.cron[0].run_id, CUSTODY_RUN_UUIDS[0]);
                        assert_eq!(state.cron[0].state, "running");
                        assert!(!state.cron[0].fence_state_known);
                        assert!(state.cron[0].terminal_outcome.is_none());
                        state.preserve_evidence(&run_root, "coalesced-post-admission");
                        admitted_tx.send(state).expect("post-admission observer");
                        terminal_release_rx.lock().unwrap().recv_timeout(CUSTODY_WAIT)
                            .expect("release this actual executor's local terminal");
                        Ok(AutomationExecutionResult::RetainedUnsettled {
                            terminal: Box::new(AutomationExecutionResult::InterruptedWithFences {
                                detail: "tool outcome remains unknown".into(),
                                quiesced_for_upgrade: true,
                                fences: clear_fences(),
                            }),
                        })
                    },
                );
                // The coalescer also converts Err to None. Preserve and inspect
                // the full real Runtime result before returning it unchanged.
                fs::write(run_root.join("inner-runtime-result.txt"), format!("{result:?}")).unwrap();
                result_tx.send(result.clone()).expect("inner Runtime result observer");
                assert_eq!(result, Ok(None), "assert the actual result before the coalescer can fold an error");
                result
            });
            let delivery = json!({"events": CUSTODY_RUN_UUIDS.iter().enumerate().map(|(index, id)| {
                json!({
                    "id": id,
                    "sandAgentId": fixture.agent_id,
                    "automationId": stable_automation_id(&fixture.agent_id, &fixture.automation_id),
                    "timestampMs": 100,
                    "event": {"source":"github", "repo":"org/repo", "kind":"pr-opened",
                        "actor":"reviewer", "title":format!("Event {index}"), "timestampMs":100},
                })
            }).collect::<Vec<_>>()});
            let backend = CustodyBackend::start(
                &fixture.root, "initial-backend", &fixture.runtime, delivery.clone(), Some(executor),
            );
            // These drop before the backend owner on failure, releasing only
            // our held workers before its bounded-child cleanup joins them.
            let mut primer_release = CustodyRelease(Some(primer_release_tx));
            let mut terminal_release = CustodyRelease(Some(terminal_release_tx));
            let mut prepared_events = Vec::new();
            for expected_id in CUSTODY_RUN_UUIDS {
                let receipt = custody_receive(&backend.dispatches, "actual event enqueue");
                assert_eq!(receipt.run_uuid, expected_id);
                match receipt.prepared {
                    PreparedBackendFire::Event { agent_id, automation_id, run_uuid, event } => {
                        assert_eq!(agent_id, fixture.agent_id);
                        assert_eq!(automation_id, fixture.automation_id);
                        assert_eq!(run_uuid, expected_id);
                        prepared_events.push(event);
                    }
                    other => panic!("fresh actual backend delivery was not admitted: {other:?}"),
                }
            }
            assert_eq!(executions.load(Ordering::SeqCst), 0, "all target items queued behind the held batch");
            assert!(matches!(backend.callbacks.try_recv(), Err(mpsc::TryRecvError::Empty)));
            primer_release.release();
            assert_eq!(custody_receive(&primer, "scheduling control result"), Some(FireAutomationOutcome::Ok));
            assert!(matches!(primer.recv_timeout(CUSTODY_WAIT), Err(mpsc::RecvTimeoutError::Disconnected)));

            let batch = custody_receive(&batches, "actual coalesced batch");
            assert_eq!(batch.agent_id, fixture.agent_id);
            assert_eq!(batch.automation.id, fixture.automation_id);
            assert_eq!(batch.events, prepared_events);
            assert_eq!(batch.run_uuid.as_deref(), Some(CUSTODY_RUN_UUIDS[0]));
            assert_eq!(batch.coalesced_run_uuids,
                CUSTODY_RUN_UUIDS[1..].iter().map(|id| (*id).to_string()).collect::<Vec<_>>());
            let admitted = custody_receive(&admission, "actual history and cron admission");
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            assert_eq!(CustodyPersistedState::capture(
                &fixture.workers, &fixture.store, &fixture.agent_id, &fixture.automation_id,
            ), admitted);
            assert!(fixture.observed.lock().unwrap().is_empty());
            assert!(matches!(results.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert!(matches!(backend.callbacks.try_recv(), Err(mpsc::TryRecvError::Empty)));
            backend.transport.assert_no_completion_or_ack();
            // Events use coalescer serialization, not the manual/schedule key.
            assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));

            terminal_release.release();
            assert_eq!(custody_receive(&results, "unmodified inner Runtime result"), Ok(None));
            let mut callbacks = std::collections::BTreeSet::new();
            for _ in CUSTODY_RUN_UUIDS {
                let (id, result) = custody_receive(&backend.callbacks, "actual None callback return");
                assert_eq!(result, Ok(()), "{id}");
                assert!(callbacks.insert(id), "each backend callback runs once");
            }
            assert_eq!(callbacks, CUSTODY_RUN_UUIDS.map(str::to_owned).into_iter().collect());
            backend.join_callbacks();
            wait_for_custody_batch_to_drain(&coalescer);
            assert!(!fixture.runtime.wakes_suspended(), "dispose/suspend must not supply None");
            assert!(matches!(backend.callbacks.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert!(matches!(batches.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert!(matches!(results.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            let retained = CustodyPersistedState::capture(
                &fixture.workers, &fixture.store, &fixture.agent_id, &fixture.automation_id,
            );
            retained.preserve_evidence(&fixture.root, "coalesced-after-retention");
            assert_eq!(retained, admitted);
            assert!(fixture.observed.lock().unwrap().is_empty());
            for _ in 0..3 {
                let earlier_poll = backend.transport.polls.load(Ordering::SeqCst);
                backend.runtime.request_reconcile();
                backend.assert_abandoned_after(earlier_poll);
                assert_eq!(CustodyPersistedState::capture(
                    &fixture.workers, &fixture.store, &fixture.agent_id, &fixture.automation_id,
                ), admitted);
                assert!(fixture.observed.lock().unwrap().is_empty());
                assert_eq!(executions.load(Ordering::SeqCst), 1);
            }
            backend.runtime.stop();
            backend.transport.assert_no_completion_or_ack();
            drop(backend);

            let original = fixture.runtime.read_run_for_delivery(
                &fixture.agent_id, &fixture.automation_id, CUSTODY_RUN_UUIDS[0],
            ).unwrap().expect("actual retained original");
            let mut manual_executions = 0;
            for index in 0..manual_runs {
                let result = fixture.runtime.run_agent_automation_now_with(
                    &fixture.agent_id, &fixture.automation_id, |_| {
                        manual_executions += 1;
                        Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
                    },
                );
                assert_eq!(result, Ok(Some(FireAutomationOutcome::Ok)), "manual run {index}");
                for run_id in CUSTODY_RUN_UUIDS {
                    assert_eq!(fixture.runtime.read_run_for_delivery(
                        &fixture.agent_id, &fixture.automation_id, run_id,
                    ).unwrap().as_ref(), Some(&original));
                }
                assert_eq!(fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap(), admitted.cron);
            }
            assert_eq!(manual_executions, manual_runs, "each independent manual run really entered execution");
            let before_reopen = CustodyPersistedState::capture(
                &fixture.workers, &fixture.store, &fixture.agent_id, &fixture.automation_id,
            );
            let rows: Vec<Value> = serde_json::from_slice(&before_reopen.history).unwrap();
            assert_eq!(rows.len(), (manual_runs + 1).min(AUTOMATION_MAX_RUN_HISTORY));
            assert_eq!(before_reopen.cron, admitted.cron, "manual settlements cannot refresh original cron authority");
            let initial_rows: Vec<Value> = serde_json::from_slice(&admitted.history).unwrap();
            assert_eq!(rows.iter().find(|row| row["id"] == CUSTODY_RUN_UUIDS[0]), initial_rows.first(),
                "every stored original field and alias survives the history window");
            let observations = fixture.observed.lock().unwrap().clone();
            assert_eq!(observations.len(), manual_runs);
            assert!(observations.iter().all(|report|
                report.outcome == FireAutomationOutcome::Ok
                    && report.automation_run_id.as_deref() != Some(CUSTODY_RUN_UUIDS[0])));
            if manual_runs == 0 {
                assert_eq!(before_reopen, admitted, "preserve the original no-follow-up custody control");
            }
            before_reopen.preserve_evidence(&fixture.root, "before-reopen-after-manual-runs");
            fixture.runtime.dispose();
            fixture.workers.shutdown();
            assert_eq!(fixture.workers.active_worker_count(), 0);

            // The original fixture retains only cleanup custody of the root.
            // Its actual Session DB/store workers are closed. Reopen existing
            // data without create_session, upsert, begin_run, or cron seeding.
            for generation in 0..2 {
                let owners = ReopenedCustodyOwners::open(&fixture.root, &fixture.agent_id);
                assert_eq!(CustodyPersistedState::capture(
                    &owners.workers, &owners.store, &fixture.agent_id, &fixture.automation_id,
                ), before_reopen);
                let replay_executions = Arc::new(AtomicUsize::new(0));
                let reopened_executor = if manual_runs == 0 {
                    None
                } else {
                    let runtime = owners.runtime.clone();
                    let entered = Arc::clone(&replay_executions);
                    let executor: EventBatchExecutor = Arc::new(move |batch: EventFireBatch| {
                        runtime.run_background_automation_with(
                            &batch.agent_id, &batch.automation.id, AutomationRunTrigger::Event,
                            batch.events, batch.run_uuid, batch.coalesced_run_uuids, 2_000.0, None,
                            |_| {
                                entered.fetch_add(1, Ordering::SeqCst);
                                Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
                            },
                        )
                    });
                    Some(executor)
                };
                let reopened_backend = CustodyBackend::start(
                    &fixture.root, &format!("reopened-backend-{generation}"),
                    &owners.runtime, delivery.clone(), reopened_executor,
                );
                reopened_backend.assert_abandoned_after(0);
                for _ in 0..2 {
                    let earlier_poll = reopened_backend.transport.polls.load(Ordering::SeqCst);
                    reopened_backend.runtime.request_reconcile();
                    reopened_backend.assert_abandoned_after(earlier_poll);
                    assert_eq!(CustodyPersistedState::capture(
                        &owners.workers, &owners.store, &fixture.agent_id, &fixture.automation_id,
                    ), before_reopen);
                    assert!(owners.observed.lock().unwrap().is_empty());
                    assert!(!owners.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
                    assert_eq!(executions.load(Ordering::SeqCst), 1);
                    assert_eq!(replay_executions.load(Ordering::SeqCst), 0, "actual reopened executor must stay unreachable");
                }
                reopened_backend.runtime.stop();
                reopened_backend.transport.assert_no_completion_or_ack();
                assert!(matches!(reopened_backend.callbacks.try_recv(), Err(mpsc::TryRecvError::Disconnected)));
                drop(reopened_backend);
                drop(owners);
            }
            assert_eq!(executions.load(Ordering::SeqCst), 1);
}

#[test]
fn real_full_running_history_refuses_before_cron_or_executor_and_checked_read_obeys_deletion() {
    bounded_automation_case(
        "real_full_running_history_refuses_before_cron_or_executor_and_checked_read_obeys_deletion",
        || {
            let fixture = ShippingAutomation::new("Running capacity admission");
            let mut admitted_executions = 0;
            for _ in 0..AUTOMATION_MAX_RUN_HISTORY {
                let result = fixture.runtime.run_agent_automation_now_with(
                    &fixture.agent_id, &fixture.automation_id, |_| {
                        admitted_executions += 1;
                        Ok(AutomationExecutionResult::RetainedUnsettled {
                            terminal: Box::new(AutomationExecutionResult::Completed),
                        })
                    },
                );
                assert_eq!(result, Ok(None));
            }
            assert_eq!(admitted_executions, AUTOMATION_MAX_RUN_HISTORY);
            let original = fixture.store.read_runs(&fixture.automation_id);
            assert_eq!(original.len(), AUTOMATION_MAX_RUN_HISTORY);
            assert!(original.iter().all(|run| run.status == "running"));
            let history = fs::read(fixture.store.runs_path(&fixture.automation_id)).unwrap();
            let cron = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
            assert!(cron.is_empty());
            let mut refused_executions = 0;
            let typed_refused = fixture.runtime.run_background_automation_attempt_with(
                &fixture.agent_id, &fixture.automation_id, AutomationRunTrigger::Schedule,
                Vec::new(), Some("refused-at-capacity".into()), Vec::new(), 2_000.0, None,
                |_| {
                    refused_executions += 1;
                    Ok(AutomationExecutionResult::Completed)
                },
            );
            let typed_detail = expect_admission_refusal(
                typed_refused, AutomationAdmissionStage::FreshHistory,
            );
            assert_eq!(typed_detail,
                "automation store io error: automation history has reached its unsettled run capacity");
            let refused = fixture.runtime.run_background_automation_with(
                &fixture.agent_id, &fixture.automation_id, AutomationRunTrigger::Schedule,
                Vec::new(), Some("refused-at-capacity".into()), Vec::new(), 2_000.0, None,
                |_| {
                    refused_executions += 1;
                    Ok(AutomationExecutionResult::Completed)
                },
            );
            let legacy_detail = refused.unwrap_err();
            assert!(legacy_detail.contains("unsettled run capacity"));
            assert_eq!(legacy_detail, typed_detail, "the legacy facade retains the exact admission error");
            assert_eq!(refused_executions, 0);
            assert_eq!(fs::read(fixture.store.runs_path(&fixture.automation_id)).unwrap(), history);
            assert_eq!(fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap(), cron);
            assert!(fixture.observed.lock().unwrap().is_empty());
            assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
            for run in &original {
                assert_eq!(fixture.runtime.read_run_for_delivery(
                    &fixture.agent_id, &fixture.automation_id, &run.id,
                ).unwrap().as_ref(), Some(run));
            }
            assert_eq!(fixture.runtime.read_run_for_delivery(
                &fixture.agent_id, &fixture.automation_id, "refused-at-capacity",
            ).unwrap(), None);

            fixture.workers.begin_agent_delete(&fixture.agent_id).unwrap();
            let blocked = fixture.runtime.read_run_for_delivery(
                &fixture.agent_id, &fixture.automation_id, &original[0].id,
            ).expect_err("checked read uses the canonical Session admission/deletion fence");
            assert!(blocked.contains("new session work is not admitted"));
            fixture.workers.end_agent_delete(&fixture.agent_id).unwrap();
            assert_eq!(fs::read(fixture.store.runs_path(&fixture.automation_id)).unwrap(), history);
            assert_eq!(fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap(), cron);

            // Explicitly settle one fixture run through the same admitted
            // store owner. This capacity control is not unknown-effect recovery.
            SandAgentSessionStore::new(Arc::clone(&fixture.workers))
                .with_admitted_filesystem_mutation(&fixture.agent_id, || {
                    fixture.store.finish_run(&fixture.automation_id, &original[0].id,
                        "ok", 3_000.0, None).map_err(|error| error.to_string())?
                        .ok_or_else(|| "capacity control did not settle its exact fixture row".to_string())?;
                    Ok(())
                }).unwrap();
            let next = fixture.runtime.run_background_automation_with(
                &fixture.agent_id, &fixture.automation_id, AutomationRunTrigger::Schedule,
                Vec::new(), Some("independent-after-capacity".into()), Vec::new(), 3_001.0, None,
                |_| {
                    refused_executions += 1;
                    Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
                },
            );
            assert_eq!(next, Ok(Some(FireAutomationOutcome::Ok)));
            assert_eq!(refused_executions, 1, "the refused call released its volatile key for real later work");
            assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
            let after = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
            assert_eq!(after.len(), 1);
            assert_eq!(after[0].run_id, "independent-after-capacity");
            assert_eq!(after[0].state, "terminal");
            assert!(after[0].fence_state_known);
            let reports = fixture.observed.lock().unwrap().clone();
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].automation_run_id.as_deref(), Some("independent-after-capacity"));
            for run in original.iter().skip(1) {
                assert_eq!(fixture.runtime.read_run_for_delivery(
                    &fixture.agent_id, &fixture.automation_id, &run.id,
                ).unwrap().as_ref(), Some(run));
            }
        },
    );
}

fn expect_admission_refusal(
    result: Result<AutomationRunAttempt, String>,
    expected_stage: AutomationAdmissionStage,
) -> String {
    match result {
        Ok(AutomationRunAttempt::AdmissionRefused { stage, detail }) => {
            assert_eq!(stage, expected_stage);
            detail
        }
        other => panic!("expected typed {expected_stage:?} refusal, got {other:?}"),
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AdmissionOwnerState {
    config: Option<Vec<u8>>,
    history: Option<Vec<u8>>,
    cron: Vec<CronSessionRunRecord>,
    guard: SpendGuardState,
    unread: UnreadState,
    transcript: Vec<Value>,
}

impl AdmissionOwnerState {
    fn capture(store: &FileAutomationStore, automation_id: &str, owner: &SandAgentDb) -> Self {
        let read_owned_file = |path: PathBuf| match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => panic!("read owned admission fixture {}: {error}", path.display()),
        };
        Self {
            config: read_owned_file(store.config_path(automation_id)),
            history: read_owned_file(store.runs_path(automation_id)),
            // Retain the actual DB owner before deletion. These reads must not
            // reopen Session admission while its deletion fence is raised.
            cron: owner.list_cron_session_runs().unwrap(),
            guard: owner.get_automation_spend_guard_state().unwrap(),
            unread: owner.get_unread_state().unwrap(),
            transcript: owner.get_transcript_entries().unwrap(),
        }
    }

    fn preserve_evidence(&self, root: &Path, phase: &str) {
        fs::write(root.join(format!("{phase}-admission-state.txt")), format!("{self:#?}")).unwrap();
    }
}

fn stale_admission_definition(fixture: &ShippingAutomation) {
    fixture.runtime.seed_known_automations(&fixture.agent_id).unwrap();
    // This is an external definition edit through the existing Session/store
    // owner. No directory watcher is attached. The next Agent/Updated event
    // therefore occurs synchronously after Runtime's checked history preflight.
    SandAgentSessionStore::new(Arc::clone(&fixture.workers))
        .with_admitted_filesystem_mutation(&fixture.agent_id, || {
            let mut config = fixture.store.read_config(&fixture.automation_id).unwrap();
            config.prompt.push_str(" after the external definition edit");
            config.created_at = 1_000.0;
            config.last_run_at = None;
            config.raised_notices.clear();
            fixture.store.write_config(&fixture.automation_id, &config)
                .map_err(|error| error.to_string())
        }).unwrap();
}

fn prepare_admission_nudge(fixture: &ShippingAutomation, now: f64) {
    let last_viewed = now - SPEND_GUARD_IDLE_TTL_MS - 10_000.0;
    fixture.workers.mark_agent_viewed(&fixture.agent_id, last_viewed, false).unwrap();
    for index in 0..15 {
        fixture.workers.mark_agent_activity(&fixture.agent_id, last_viewed + index as f64 + 1.0).unwrap();
    }
    assert_eq!(fixture.runtime.spend_guard().evaluate_with_store(
        &fixture.agent_id, &fixture.store, now,
    ).unwrap().decision, SpendGuardDecision::Nudge, "the unblocked guard has a real write to perform");
}

fn assert_distinct_admission_control(fixture: &ShippingAutomation, run_id: &str, fired_at_ms: f64) {
    let mut executions = 0;
    let result = fixture.runtime.run_background_automation_attempt_with(
        &fixture.agent_id, &fixture.automation_id, AutomationRunTrigger::Schedule,
        Vec::new(), Some(run_id.to_string()), Vec::new(), fired_at_ms, None,
        |_| {
            executions += 1;
            let admitted = fixture.store.read_run_for_delivery(&fixture.automation_id, run_id)
                .unwrap().expect("fresh control history exists before executor entry");
            assert_eq!(admitted.status, "running");
            let rows = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
            assert_eq!(rows.iter().find(|row| row.run_id == run_id).unwrap().state, "running");
            Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
        },
    );
    assert_eq!(result, Ok(AutomationRunAttempt::Outcome(Some(FireAutomationOutcome::Ok))));
    assert_eq!(executions, 1, "a distinct attempt enters the real executor after repair");
    assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
    let history = fixture.store.read_run_for_delivery(&fixture.automation_id, run_id).unwrap().unwrap();
    assert_eq!(history.status, "ok");
    assert_eq!(history.finished_at, Some(fired_at_ms));
    let rows = fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
    let row = rows.iter().find(|row| row.run_id == run_id).unwrap();
    assert_eq!(row.state, "terminal");
    assert_eq!(row.terminal_outcome.as_deref(), Some("ok"));
    assert!(row.fence_state_known);
    let reports = fixture.observed.lock().unwrap();
    assert_eq!(reports.len(), 1, "only the distinct admitted control reports terminal execution");
    assert_eq!(reports[0].automation_run_id.as_deref(), Some(run_id));
    assert_eq!(reports[0].outcome, FireAutomationOutcome::Ok);
}

#[test]
fn real_typed_history_precheck_preserves_bookkeeping_and_legacy_errors() {
    bounded_automation_case(
        "real_typed_history_precheck_preserves_bookkeeping_and_legacy_errors",
        || {
            for fault in ["primary", "alias", "terminal-primary", "coalesced-terminal-alias", "malformed", "deleting"] {
                let fixture = ShippingAutomation::new(&format!("typed history precheck/{fault}"));
                seed_backend_history(&fixture);
                if matches!(fault, "terminal-primary" | "coalesced-terminal-alias") {
                    assert!(fixture.workers.finish_cron_session_run(
                        &fixture.agent_id, &fixture.automation_id, CUSTODY_RUN_UUIDS[0], "ok", 200.0, None,
                    ).unwrap());
                    SandAgentSessionStore::new(Arc::clone(&fixture.workers))
                        .with_admitted_filesystem_mutation(&fixture.agent_id, || {
                            assert!(fixture.store.finish_run(
                                &fixture.automation_id, CUSTODY_RUN_UUIDS[0], "ok", 200.0, None,
                            ).map_err(|error| error.to_string())?.is_some());
                            Ok(())
                        }).unwrap();
                }
                stale_admission_definition(&fixture);
                let owner = fixture.workers.open_agent_db_owner(&fixture.agent_id).unwrap();
                let valid_history = fs::read(fixture.store.runs_path(&fixture.automation_id)).unwrap();
                let requested = match fault {
                    "primary" | "terminal-primary" => CUSTODY_RUN_UUIDS[0],
                    "alias" => CUSTODY_RUN_UUIDS[1],
                    _ => "precheck-request",
                };
                let aliases = if fault == "coalesced-terminal-alias" {
                    vec![CUSTODY_RUN_UUIDS[2].to_string()]
                } else {
                    Vec::new()
                };
                let trigger = if aliases.is_empty() {
                    AutomationRunTrigger::Schedule
                } else {
                    AutomationRunTrigger::Event
                };
                if fault == "malformed" {
                    SandAgentSessionStore::new(Arc::clone(&fixture.workers))
                        .with_admitted_filesystem_mutation(&fixture.agent_id, || {
                            fs::write(fixture.store.runs_path(&fixture.automation_id), b"[ malformed")
                                .map_err(|error| error.to_string())
                        }).unwrap();
                }
                if fault == "deleting" {
                    fixture.workers.begin_agent_delete(&fixture.agent_id).unwrap();
                }
                let expected_detail = match fault {
                    "malformed" => fixture.store.read_run_for_delivery(&fixture.automation_id, requested)
                        .unwrap_err().to_string(),
                    "deleting" => format!(
                        "agent {} deletion cleanup is pending; new session work is not admitted", fixture.agent_id,
                    ),
                    _ => "automation run identity has already been admitted".into(),
                };
                let lifecycle = Arc::new(Mutex::new(Vec::<AutomationLifecycleEvent>::new()));
                let sink = Arc::clone(&lifecycle);
                fixture.runtime.set_lifecycle_reporter(Some(Arc::new(move |event| {
                    sink.lock().unwrap().push(event.clone());
                })));
                let before = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                before.preserve_evidence(&fixture.root, "precheck-before");
                let executions = AtomicUsize::new(0);
                let typed = fixture.runtime.run_background_automation_attempt_with(
                    &fixture.agent_id, &fixture.automation_id, trigger,
                    Vec::new(), Some(requested.to_string()), aliases.clone(), 2_000.0, None,
                    |_| {
                        executions.fetch_add(1, Ordering::SeqCst);
                        Ok(AutomationExecutionResult::Completed)
                    },
                );
                assert_eq!(expect_admission_refusal(typed, AutomationAdmissionStage::HistoryPrecheck), expected_detail);
                let legacy = fixture.runtime.run_background_automation_with(
                    &fixture.agent_id, &fixture.automation_id, trigger,
                    Vec::new(), Some(requested.to_string()), aliases, 2_000.0, None,
                    |_| {
                        executions.fetch_add(1, Ordering::SeqCst);
                        Ok(AutomationExecutionResult::Completed)
                    },
                );
                assert_eq!(legacy, Err(expected_detail), "legacy precheck keeps its exact String detail");
                let after = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                after.preserve_evidence(&fixture.root, "precheck-after");
                assert_eq!(after, before, "precheck must precede notice, lastRunAt, guard, cron and transcript writes: {fault}");
                assert!(lifecycle.lock().unwrap().is_empty(), "precheck also precedes lifecycle bookkeeping");
                assert_eq!(executions.load(Ordering::SeqCst), 0);
                assert!(fixture.observed.lock().unwrap().is_empty());
                assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));

                fixture.runtime.set_lifecycle_reporter(None);
                if fault == "deleting" {
                    assert!(owner.read_session_work_admission().unwrap().unwrap().deletion_pending);
                    fixture.workers.end_agent_delete(&fixture.agent_id).unwrap();
                }
                if fault == "malformed" {
                    SandAgentSessionStore::new(Arc::clone(&fixture.workers))
                        .with_admitted_filesystem_mutation(&fixture.agent_id, || {
                            fs::write(fixture.store.runs_path(&fixture.automation_id), &valid_history)
                                .map_err(|error| error.to_string())
                        }).unwrap();
                }
                let original = fixture.store.read_runs(&fixture.automation_id);
                assert_distinct_admission_control(&fixture, "distinct-after-precheck", 3_000.0);
                let current = fixture.store.read_runs(&fixture.automation_id);
                for run in &original {
                    assert_eq!(current.iter().find(|current| current.id == run.id), Some(run));
                }
                let rows = owner.list_cron_session_runs().unwrap();
                for row in &before.cron {
                    assert_eq!(rows.iter().find(|current| current.run_id == row.run_id), Some(row));
                }
            }
        },
    );
}

#[test]
fn real_typed_fresh_history_refuses_lifecycle_claims_and_invalid_images() {
    bounded_automation_case(
        "real_typed_fresh_history_refuses_lifecycle_claims_and_invalid_images",
        || {
            for fault in ["primary", "alias", "terminal-primary", "malformed"] {
                for legacy in [false, true] {
                    let fixture = ShippingAutomation::new(&format!("late history/{fault}/legacy={legacy}"));
                    seed_backend_history(&fixture);
                    stale_admission_definition(&fixture);
                    let fired_at_ms = if fault == "terminal-primary" {
                        let now = 1_900_000_000_000.0;
                        prepare_admission_nudge(&fixture, now);
                        now
                    } else {
                        2_000.0
                    };
                    let owner = fixture.workers.open_agent_db_owner(&fixture.agent_id).unwrap();
                    let before = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                    let mut expected_config = fixture.store.read_config(&fixture.automation_id).unwrap();
                    expected_config.last_run_at = Some(fired_at_ms);
                    expected_config.raised_notices.push(GITHUB_LISTENER_SCOPE.to_string());
                    before.preserve_evidence(&fixture.root, "fresh-before");
                    let lifecycle = Arc::new(Mutex::new(Vec::<AutomationLifecycleEvent>::new()));
                    let published = Arc::new(Mutex::new(None::<(AdmissionOwnerState, String)>));
                    let callback_count = Arc::new(AtomicUsize::new(0));
                    let callback_workers = Arc::clone(&fixture.workers);
                    let callback_store = fixture.store.clone();
                    let callback_agent = fixture.agent_id.clone();
                    let callback_automation = fixture.automation_id.clone();
                    let callback_owner = Arc::clone(&owner);
                    let callback_events = Arc::clone(&lifecycle);
                    let callback_published = Arc::clone(&published);
                    let callback_seen = Arc::clone(&callback_count);
                    fixture.runtime.set_lifecycle_reporter(Some(Arc::new(move |event| {
                        assert_eq!(event.source, AutomationLifecycleSource::Agent);
                        assert_eq!(event.action, AutomationLifecycleAction::Updated);
                        assert_eq!(event.agent_id, callback_agent);
                        assert_eq!(event.automation_id, callback_automation);
                        assert_eq!(callback_seen.fetch_add(1, Ordering::SeqCst), 0,
                            "only the deliberately stale baseline opens this injection boundary");
                        callback_events.lock().unwrap().push(event.clone());
                        let competing_id = if fault == "alias" { "late-competing-primary" } else { "late-request" };
                        let aliases = (fault == "alias").then(|| vec!["late-request".to_string()]);
                        // Runtime still owns its mutation mutex, but its prior
                        // Session closure has returned. Use only Session/store
                        // here, and let this closure return before cron calls.
                        SandAgentSessionStore::new(Arc::clone(&callback_workers))
                            .with_admitted_filesystem_mutation(&callback_agent, || {
                                assert_eq!(callback_store.read_run_for_delivery(&callback_automation, "late-request")
                                    .map_err(|error| error.to_string())?, None);
                                if fault == "malformed" {
                                    fs::write(callback_store.runs_path(&callback_automation), b"[ malformed")
                                        .map_err(|error| error.to_string())?;
                                } else {
                                    let claimed = callback_store.begin_fresh_run(
                                        &callback_automation, "event", 150.0, None,
                                        Some(competing_id), aliases.as_deref(),
                                    ).map_err(|error| error.to_string())?.expect("competing fresh claim");
                                    assert_eq!(claimed.id, competing_id);
                                }
                                Ok(())
                            }).unwrap();
                        if fault != "malformed" {
                            assert!(callback_workers.begin_cron_session_run(
                                &callback_agent, &callback_automation, competing_id, 150.0,
                            ).unwrap());
                            if fault == "terminal-primary" {
                                assert!(callback_workers.finish_cron_session_run(
                                    &callback_agent, &callback_automation, competing_id, "ok", 160.0, None,
                                ).unwrap());
                                SandAgentSessionStore::new(Arc::clone(&callback_workers))
                                    .with_admitted_filesystem_mutation(&callback_agent, || {
                                        assert!(callback_store.finish_run(
                                            &callback_automation, competing_id, "ok", 160.0, None,
                                        ).map_err(|error| error.to_string())?.is_some());
                                        Ok(())
                                    }).unwrap();
                            }
                        }
                        let detail = if fault == "malformed" {
                            callback_store.read_run_for_delivery(&callback_automation, "late-request")
                                .unwrap_err().to_string()
                        } else {
                            "automation store io error: automation run identity has already been admitted".to_string()
                        };
                        *callback_published.lock().unwrap() = Some((
                            AdmissionOwnerState::capture(&callback_store, &callback_automation, &callback_owner),
                            detail,
                        ));
                    })));
                    let executions = AtomicUsize::new(0);
                    let result = if legacy {
                        fixture.runtime.run_background_automation_with(
                            &fixture.agent_id, &fixture.automation_id, AutomationRunTrigger::Schedule,
                            Vec::new(), Some("late-request".into()), Vec::new(), fired_at_ms, None,
                            |_| {
                                executions.fetch_add(1, Ordering::SeqCst);
                                Ok(AutomationExecutionResult::Completed)
                            },
                        ).map(AutomationRunAttempt::Outcome)
                    } else {
                        fixture.runtime.run_background_automation_attempt_with(
                            &fixture.agent_id, &fixture.automation_id, AutomationRunTrigger::Schedule,
                            Vec::new(), Some("late-request".into()), Vec::new(), fired_at_ms, None,
                            |_| {
                                executions.fetch_add(1, Ordering::SeqCst);
                                Ok(AutomationExecutionResult::Completed)
                            },
                        )
                    };
                    let (claimed, expected_detail) = published.lock().unwrap().clone()
                        .expect("the existing lifecycle event published the competing state");
                    claimed.preserve_evidence(&fixture.root, "fresh-competing-claim");
                    if legacy {
                        assert_eq!(result, Err(expected_detail));
                    } else {
                        assert_eq!(expect_admission_refusal(result, AutomationAdmissionStage::FreshHistory), expected_detail);
                    }
                    assert_eq!(callback_count.load(Ordering::SeqCst), 1);
                    assert_eq!(lifecycle.lock().unwrap().len(), 1);
                    assert_eq!(executions.load(Ordering::SeqCst), 0);
                    assert!(fixture.observed.lock().unwrap().is_empty());
                    assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
                    let after = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                    after.preserve_evidence(&fixture.root, "fresh-refused");
                    assert_eq!(after.history, claimed.history, "refusal cannot rewrite the competing image");
                    assert_eq!(after.cron, claimed.cron, "refusal cannot register or settle a competing cron row");
                    if fault == "terminal-primary" {
                        assert_eq!(claimed.guard, before.guard, "the lifecycle claim precedes Guard's admitted write");
                        assert_eq!(after.guard.nudged_at_ms, Some(fired_at_ms));
                        assert_eq!(after.guard.card_entry_ids.len(), 1);
                        assert_eq!(after.transcript.len(), before.transcript.len() + 1);
                        assert_eq!(after.unread.unread_count, before.unread.unread_count + 1.0);
                        assert!(after.transcript.iter().any(|entry|
                            entry.get("id").and_then(Value::as_str) == Some(after.guard.card_entry_ids[0].as_str())),
                            "FreshHistory refusal retains the already-committed Guard card");
                    } else {
                        assert_eq!(after.guard, before.guard);
                        assert_eq!(after.unread, before.unread);
                        assert_eq!(after.transcript, before.transcript);
                    }
                    assert_eq!(fixture.store.read_config(&fixture.automation_id), Some(expected_config),
                        "FreshHistory refusal follows the already-authorized notice and lastRunAt writes");

                    fixture.runtime.set_lifecycle_reporter(None);
                    if fault == "malformed" {
                        SandAgentSessionStore::new(Arc::clone(&fixture.workers))
                            .with_admitted_filesystem_mutation(&fixture.agent_id, || {
                                fs::write(fixture.store.runs_path(&fixture.automation_id), before.history.as_ref().unwrap())
                                    .map_err(|error| error.to_string())
                            }).unwrap();
                    }
                    let original = fixture.store.read_runs(&fixture.automation_id);
                    assert_distinct_admission_control(&fixture, "distinct-after-late-claim", fired_at_ms + 1_000.0);
                    let current = fixture.store.read_runs(&fixture.automation_id);
                    for run in &original {
                        assert_eq!(current.iter().find(|current| current.id == run.id), Some(run));
                    }
                    let rows = owner.list_cron_session_runs().unwrap();
                    for row in &claimed.cron {
                        assert_eq!(rows.iter().find(|current| current.run_id == row.run_id), Some(row));
                    }
                }
            }
        },
    );
}

#[test]
fn real_typed_spend_guard_refusal_preserves_order_and_recovers() {
    bounded_automation_case(
        "real_typed_spend_guard_refusal_preserves_order_and_recovers",
        || {
            for legacy in [false, true] {
                let fixture = ShippingAutomation::new(&format!("SpendGuard admission/legacy={legacy}"));
                seed_backend_history(&fixture);
                stale_admission_definition(&fixture);
                let now = 1_900_000_000_000.0;
                prepare_admission_nudge(&fixture, now);
                let owner = fixture.workers.open_agent_db_owner(&fixture.agent_id).unwrap();
                let before = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                before.preserve_evidence(&fixture.root, "guard-before");
                let callback_count = Arc::new(AtomicUsize::new(0));
                let callback_seen = Arc::clone(&callback_count);
                let callback_workers = Arc::clone(&fixture.workers);
                let callback_agent = fixture.agent_id.clone();
                let callback_automation = fixture.automation_id.clone();
                fixture.runtime.set_lifecycle_reporter(Some(Arc::new(move |event| {
                    assert_eq!(event.source, AutomationLifecycleSource::Agent);
                    assert_eq!(event.action, AutomationLifecycleAction::Updated);
                    assert_eq!(event.agent_id, callback_agent);
                    assert_eq!(event.automation_id, callback_automation);
                    assert_eq!(callback_seen.fetch_add(1, Ordering::SeqCst), 0);
                    // The history precheck has released Session admission. This
                    // existing callback raises its deletion fence before Guard.
                    callback_workers.begin_agent_delete(&callback_agent).unwrap();
                })));
                let executions = AtomicUsize::new(0);
                let result = if legacy {
                    fixture.runtime.run_background_automation_with(
                        &fixture.agent_id, &fixture.automation_id, AutomationRunTrigger::Schedule,
                        Vec::new(), Some("guard-refused".into()), Vec::new(), now, None,
                        |_| {
                            executions.fetch_add(1, Ordering::SeqCst);
                            Ok(AutomationExecutionResult::Completed)
                        },
                    ).map(AutomationRunAttempt::Outcome)
                } else {
                    fixture.runtime.run_background_automation_attempt_with(
                        &fixture.agent_id, &fixture.automation_id, AutomationRunTrigger::Schedule,
                        Vec::new(), Some("guard-refused".into()), Vec::new(), now, None,
                        |_| {
                            executions.fetch_add(1, Ordering::SeqCst);
                            Ok(AutomationExecutionResult::Completed)
                        },
                    )
                };
                let detail = format!(
                    "agent {} deletion cleanup is pending; new session work is not admitted", fixture.agent_id,
                );
                if legacy {
                    assert_eq!(result, Err(detail));
                } else {
                    assert_eq!(expect_admission_refusal(result, AutomationAdmissionStage::SpendGuard), detail);
                }
                assert_eq!(callback_count.load(Ordering::SeqCst), 1);
                assert!(owner.read_session_work_admission().unwrap().unwrap().deletion_pending);
                let after = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                after.preserve_evidence(&fixture.root, "guard-refused");
                assert_eq!(after, before, "Guard admission refuses before card, notice, lastRunAt and history writes");
                assert_eq!(executions.load(Ordering::SeqCst), 0);
                assert!(fixture.observed.lock().unwrap().is_empty());
                assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));

                fixture.runtime.set_lifecycle_reporter(None);
                fixture.workers.end_agent_delete(&fixture.agent_id).unwrap();
                assert_distinct_admission_control(&fixture, "distinct-after-guard-repair", now);
                let recovered = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                recovered.preserve_evidence(&fixture.root, "guard-recovered");
                assert_eq!(recovered.guard.nudged_at_ms, Some(now));
                assert_eq!(recovered.guard.card_entry_ids.len(), 1);
                assert_eq!(recovered.transcript.len(), before.transcript.len() + 1);
                assert!(recovered.transcript.iter().any(|entry|
                    entry.get("id").and_then(Value::as_str) == Some(recovered.guard.card_entry_ids[0].as_str())));
                for row in &before.cron {
                    assert_eq!(recovered.cron.iter().find(|current| current.run_id == row.run_id), Some(row));
                }
            }
        },
    );
}

#[test]
fn real_typed_admitted_failures_keep_existing_outcomes_and_outer_errors() {
    bounded_automation_case(
        "real_typed_admitted_failures_keep_existing_outcomes_and_outer_errors",
        || {
            for fault in ["failed", "executor-error", "interrupted", "retained", "post-history-error", "cron-rejected"] {
                let fixture = ShippingAutomation::new(&format!("typed post-admission/{fault}"));
                let owner = fixture.workers.open_agent_db_owner(&fixture.agent_id).unwrap();
                if fault == "cron-rejected" {
                    assert!(fixture.workers.begin_cron_session_run(
                        &fixture.agent_id, &fixture.automation_id, "typed-admitted", 100.0,
                    ).unwrap());
                    assert!(fixture.workers.finish_cron_session_run(
                        &fixture.agent_id, &fixture.automation_id, "typed-admitted", "ok", 200.0, None,
                    ).unwrap());
                    assert_eq!(fixture.store.read_run_for_delivery(&fixture.automation_id, "typed-admitted").unwrap(), None);
                }
                let before = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                before.preserve_evidence(&fixture.root, "admitted-before");
                let mut executions = 0;
                let mut at_execution = None::<AdmissionOwnerState>;
                let result = fixture.runtime.run_server_scheduled_automation_attempt_with(
                    &fixture.agent_id, &fixture.automation_id, "typed-admitted".into(), Some(500.0),
                    |_| {
                        executions += 1;
                        assert_ne!(fault, "cron-rejected", "cron rejection follows fresh history but precedes execution");
                        assert!(fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
                        let history = fixture.store.read_run_for_delivery(&fixture.automation_id, "typed-admitted")
                            .unwrap().expect("the typed scheduled facade accepted a fresh history identity");
                        assert_eq!(history.status, "running");
                        let admitted = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                        assert_eq!(admitted.cron.len(), 1);
                        assert_eq!(admitted.cron[0].state, "running");
                        assert_eq!(admitted.cron[0].run_id, "typed-admitted");
                        admitted.preserve_evidence(&fixture.root, "admitted-executor-entry");
                        at_execution = Some(admitted);
                        match fault {
                            "failed" => Ok(AutomationExecutionResult::FailedWithFences {
                                detail: "provider failed after admission".into(), fences: clear_fences(),
                            }),
                            "executor-error" => Err("executor failed after admission".into()),
                            "interrupted" => Ok(AutomationExecutionResult::InterruptedWithFences {
                                detail: "host upgrade after admission".into(), quiesced_for_upgrade: true,
                                fences: clear_fences(),
                            }),
                            "retained" => Ok(AutomationExecutionResult::RetainedUnsettled {
                                terminal: Box::new(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() }),
                            }),
                            "post-history-error" => {
                                SandAgentSessionStore::new(Arc::clone(&fixture.workers))
                                    .with_admitted_filesystem_mutation(&fixture.agent_id, || {
                                        fs::remove_file(fixture.store.runs_path(&fixture.automation_id))
                                            .map_err(|error| error.to_string())
                                    }).unwrap();
                                Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
                            }
                            _ => unreachable!("all admitted execution controls are explicit"),
                        }
                    },
                );
                assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
                let after = AdmissionOwnerState::capture(&fixture.store, &fixture.automation_id, &owner);
                after.preserve_evidence(&fixture.root, "admitted-after");
                if fault == "cron-rejected" {
                    assert_eq!(executions, 0);
                    assert_eq!(result, Err("cron session run registration was not committed".into()),
                        "a fresh identity was accepted; cron rejection remains an outer error");
                    assert_eq!(after.cron, before.cron, "the existing terminal owner cannot be reopened");
                    assert!(fixture.observed.lock().unwrap().is_empty());
                    let cleanup = fixture.store.read_run_for_delivery(&fixture.automation_id, "typed-admitted")
                        .unwrap().expect("cron rejection retains history cleanup");
                    assert_eq!(cleanup.status, "error");
                    assert!(cleanup.finished_at.is_some());
                    assert_eq!(cleanup.detail.as_deref(), Some("cron session run registration was not committed"));
                    assert_distinct_admission_control(&fixture, "distinct-after-cron-rejection", 3_000.0);
                    assert_eq!(fixture.store.read_run_for_delivery(&fixture.automation_id, "typed-admitted").unwrap(), Some(cleanup));
                } else if fault == "post-history-error" {
                    assert_eq!(executions, 1);
                    assert_eq!(result, Err("automation history run settlement was not committed".into()),
                        "ACK-05 preserves an outer error after execution and cron settlement");
                    assert_eq!(after.history, None);
                    assert_eq!(after.cron[0].state, "terminal");
                    assert_eq!(after.cron[0].terminal_outcome.as_deref(), Some("ok"));
                    assert!(after.cron[0].fence_state_known);
                    assert!(fixture.observed.lock().unwrap().is_empty());
                    let admitted = at_execution.as_ref().unwrap();
                    SandAgentSessionStore::new(Arc::clone(&fixture.workers))
                        .with_admitted_filesystem_mutation(&fixture.agent_id, || {
                            fs::write(fixture.store.runs_path(&fixture.automation_id), admitted.history.as_ref().unwrap())
                                .map_err(|error| error.to_string())
                        }).unwrap();
                    let restored = fixture.store.read_run_for_delivery(&fixture.automation_id, "typed-admitted")
                        .unwrap().unwrap();
                    assert_eq!(restored.status, "running", "repair restores the actual pre-fault image; it does not invent history completion");
                    assert_distinct_admission_control(&fixture, "distinct-after-history-repair", 3_000.0);
                    assert_eq!(fixture.store.read_run_for_delivery(&fixture.automation_id, "typed-admitted").unwrap(), Some(restored));
                } else if fault == "retained" {
                    assert_eq!(executions, 1);
                    assert_eq!(result, Ok(AutomationRunAttempt::Outcome(None)), "retention is an admitted outcome, not a refusal");
                    assert_eq!(after, at_execution.unwrap());
                    assert!(fixture.observed.lock().unwrap().is_empty());
                    let retained = fixture.store.read_run_for_delivery(&fixture.automation_id, "typed-admitted")
                        .unwrap().unwrap();
                    assert_distinct_admission_control(&fixture, "distinct-after-retention", 3_000.0);
                    assert_eq!(fixture.store.read_run_for_delivery(&fixture.automation_id, "typed-admitted").unwrap(), Some(retained));
                } else {
                    assert_eq!(executions, 1);
                    let (outcome, cron_outcome, detail) = match fault {
                        "interrupted" => (FireAutomationOutcome::Interrupted, "interrupted", "host upgrade after admission"),
                        "failed" => (FireAutomationOutcome::Error, "error", "provider failed after admission"),
                        "executor-error" => (FireAutomationOutcome::Error, "error", "executor failed after admission"),
                        _ => unreachable!(),
                    };
                    assert_eq!(result, Ok(AutomationRunAttempt::Outcome(Some(outcome))));
                    assert_eq!(after.cron[0].state, "terminal");
                    assert_eq!(after.cron[0].terminal_outcome.as_deref(), Some(cron_outcome));
                    assert_eq!(after.cron[0].terminal_detail.as_deref(), Some(detail));
                    assert_eq!(after.cron[0].fence_state_known, fault != "executor-error");
                    let history = fixture.store.read_run_for_delivery(&fixture.automation_id, "typed-admitted").unwrap().unwrap();
                    assert_eq!(history.status, "error");
                    assert_eq!(history.detail.as_deref(), Some(detail));
                    assert!(history.finished_at.is_some());
                    let reports = fixture.observed.lock().unwrap();
                    assert_eq!(reports.len(), 1);
                    assert_eq!(reports[0].automation_run_id.as_deref(), Some("typed-admitted"));
                    assert_eq!(reports[0].outcome, outcome);
                    assert_eq!(reports[0].quiesced_for_upgrade, fault == "interrupted");
                    assert_eq!(reports[0].scheduled_for_ms, Some(500.0));
                }
                if matches!(fault, "cron-rejected" | "post-history-error" | "retained") {
                    let rows = owner.list_cron_session_runs().unwrap();
                    for row in &after.cron {
                        assert_eq!(rows.iter().find(|current| current.run_id == row.run_id), Some(row),
                            "distinct work preserves the earlier post-admission cron custody");
                    }
                }
            }
        },
    );
}

#[derive(Debug, Clone, Copy)]
enum BackendHistoryFault {
    MissingStatus,
    FutureStatus,
    MalformedAlias,
    DuplicateOriginal,
    CrossRowAlias,
    InvalidTail,
    InvalidJson,
    InvalidUtf8,
}

fn seed_backend_history(fixture: &ShippingAutomation) -> CustodyPersistedState {
    let aliases = CUSTODY_RUN_UUIDS[1..].iter().map(|id| (*id).to_owned()).collect::<Vec<_>>();
    SandAgentSessionStore::new(Arc::clone(&fixture.workers))
        .with_admitted_filesystem_mutation(&fixture.agent_id, || {
            let run = fixture.store.begin_run(
                &fixture.automation_id, "event", 100.0, None,
                Some(CUSTODY_RUN_UUIDS[0]), Some(&aliases),
            ).map_err(|error| error.to_string())?.expect("actual running history admission");
            assert_eq!(run.status, "running");
            Ok(())
        }).expect("admitted actual history owner");
    assert!(fixture.workers.begin_cron_session_run(
        &fixture.agent_id, &fixture.automation_id, CUSTODY_RUN_UUIDS[0], 100.0,
    ).expect("actual cron registration"));
    CustodyPersistedState::capture(
        &fixture.workers, &fixture.store, &fixture.agent_id, &fixture.automation_id,
    )
}

fn raw_backend_history_fault(fault: BackendHistoryFault, admitted: &[u8]) -> Vec<u8> {
    let mut rows: Vec<Value> = serde_json::from_slice(admitted).expect("actual admitted image");
    match fault {
        BackendHistoryFault::MissingStatus => {
            rows[0].as_object_mut().unwrap().remove("status");
            rows[0]["finishedAt"] = json!(200);
        }
        BackendHistoryFault::FutureStatus => {
            rows[0]["status"] = json!("future_outcome_unknown");
            rows[0]["finishedAt"] = json!(200);
        }
        BackendHistoryFault::MalformedAlias => {
            rows[0]["coalescedRunIds"] = json!([CUSTODY_RUN_UUIDS[1], 7, CUSTODY_RUN_UUIDS[2]]);
        }
        BackendHistoryFault::DuplicateOriginal => {
            let mut terminal = rows[0].clone();
            terminal["startedAt"] = json!(200);
            terminal["finishedAt"] = json!(201);
            terminal["status"] = json!("ok");
            rows.push(terminal);
        }
        BackendHistoryFault::CrossRowAlias => rows.push(json!({
            "id":"other-row", "startedAt":200, "finishedAt":201,
            "status":"ok", "coalescedRunIds":[CUSTODY_RUN_UUIDS[1]],
        })),
        BackendHistoryFault::InvalidTail => {
            rows[0]["startedAt"] = json!(10_000);
            rows[0]["finishedAt"] = json!(10_001);
            rows[0]["status"] = json!("ok");
            rows.extend((0..AUTOMATION_MAX_RUN_HISTORY + 1).map(|index| json!({
                "id":format!("valid-sibling-{index}"), "startedAt":200 + index,
                "finishedAt":300 + index, "status":"ok",
            })));
            rows.push(json!({
                "id":"invalid-tail", "startedAt":1,
                "finishedAt":2, "status":"future_outcome_unknown",
            }));
        }
        BackendHistoryFault::InvalidJson => return b"[ malformed history".to_vec(),
        BackendHistoryFault::InvalidUtf8 => return b"[\xff]".to_vec(),
    }
    serde_json::to_vec_pretty(&rows).unwrap()
}

fn backend_history_delivery(fixture: &ShippingAutomation, event: bool, ids: &[&str]) -> Value {
    let automation = fixture.store.get(&fixture.automation_id).expect("current definition");
    let revision = sand_cloud_definition(&fixture.agent_id, &ScheduledCloudAutomation {
        id: automation.id, name: automation.name, prompt: automation.prompt,
        is_enabled: automation.is_enabled, trigger: automation.trigger,
    }, None).expect("real revisioned schedule definition").hash;
    json!({"events":ids.iter().map(|id| json!({
        "id":id, "sandAgentId":fixture.agent_id,
        "automationId":stable_automation_id(&fixture.agent_id, &fixture.automation_id),
        "timestampMs":100, "scheduledForMs":90,
        "definitionRevision":if event { None } else { Some(revision.clone()) },
        "event":if event { Some(json!({
            "source":"github", "repo":"org/repo", "kind":"pr-opened", "timestampMs":100,
        })) } else { None },
    })).collect::<Vec<_>>()})
}

fn backend_history_refusal_case(fault: BackendHistoryFault, event: bool) {
    let fixture = ShippingAutomation::new(&format!("backend history {fault:?}"));
    let admitted = seed_backend_history(&fixture);
    let raw = raw_backend_history_fault(fault, &admitted.history);
    fs::write(fixture.store.runs_path(&fixture.automation_id), &raw).expect("inject raw custody fault");
    let display = fixture.store.read_runs(&fixture.automation_id);
    match fault {
        BackendHistoryFault::MissingStatus | BackendHistoryFault::FutureStatus => {
            assert_eq!(display[0].status, "ok", "display compatibility is not custody authority");
        }
        BackendHistoryFault::MalformedAlias => assert_eq!(display[0].coalesced_run_ids,
            Some(CUSTODY_RUN_UUIDS[1..].iter().map(|id| (*id).to_owned()).collect())),
        BackendHistoryFault::DuplicateOriginal => assert_eq!(display.iter()
            .filter(|run| run.id == CUSTODY_RUN_UUIDS[0]).count(), 2),
        BackendHistoryFault::CrossRowAlias => assert_eq!(display.iter().filter(|run|
            run.coalesced_run_ids.as_ref().is_some_and(|ids|
                ids.iter().any(|id| id == CUSTODY_RUN_UUIDS[1]))).count(), 2),
        BackendHistoryFault::InvalidTail => {
            assert_eq!(display.len(), AUTOMATION_MAX_RUN_HISTORY);
            assert!(display.iter().all(|run| run.id != "invalid-tail"));
            assert_eq!(display[0].id, CUSTODY_RUN_UUIDS[0]);
            assert_eq!(display[0].status, "ok");
        }
        BackendHistoryFault::InvalidJson | BackendHistoryFault::InvalidUtf8 => assert!(display.is_empty()),
    }
    let baseline = CustodyPersistedState::capture(
        &fixture.workers, &fixture.store, &fixture.agent_id, &fixture.automation_id,
    );
    assert_eq!(baseline.config, admitted.config);
    assert_eq!(baseline.cron, admitted.cron);
    baseline.preserve_evidence(&fixture.root, "raw-refusal-baseline");
    let delivery = backend_history_delivery(&fixture, event, &CUSTODY_RUN_UUIDS);
    fixture.runtime.dispose();
    fixture.workers.shutdown();

    let executions = Arc::new(AtomicUsize::new(0));
    for generation in 0..2 {
        let owners = ReopenedCustodyOwners::open(&fixture.root, &fixture.agent_id);
        for id in CUSTODY_RUN_UUIDS {
            assert!(owners.runtime.read_run_for_delivery(
                &fixture.agent_id, &fixture.automation_id, id,
            ).is_err(), "raw custody must refuse before a display projection is trusted: {fault:?}");
        }
        assert_eq!(owners.workers.active_agent_db_owner_count(), 1,
            "checked delivery preserves canonical Session admission/open/recovery");
        let counted = Arc::clone(&executions);
        let executor: EventBatchExecutor = Arc::new(move |_| {
            counted.fetch_add(1, Ordering::SeqCst);
            Err("unavailable backend history reached the executor".into())
        });
        let backend = CustodyBackend::start(
            &fixture.root, &format!("raw-history-{generation}"),
            &owners.runtime, delivery.clone(), Some(executor),
        );
        backend.assert_abandoned_with_reason_after(0, "run_history_unavailable");
        for _ in 0..2 {
            let earlier_poll = backend.transport.polls.load(Ordering::SeqCst);
            backend.runtime.request_reconcile();
            backend.assert_abandoned_with_reason_after(earlier_poll, "run_history_unavailable");
            assert_eq!(CustodyPersistedState::capture(
                &owners.workers, &owners.store, &fixture.agent_id, &fixture.automation_id,
            ), baseline);
            assert!(owners.observed.lock().unwrap().is_empty());
            assert_eq!(executions.load(Ordering::SeqCst), 0);
            assert!(!owners.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
            assert!(matches!(backend.callbacks.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }
        backend.runtime.stop();
        backend.transport.assert_no_completion_or_ack();
        assert!(matches!(backend.callbacks.try_recv(), Err(mpsc::TryRecvError::Disconnected)));
        assert!(owners.observed.lock().unwrap().is_empty());
        assert_eq!(fs::read(owners.store.runs_path(&fixture.automation_id)).unwrap(), raw);
        drop(backend);
        drop(owners);
    }
}

#[test]
fn real_backend_history_status_refusal_survives_reopen_without_completion_or_ack() {
    bounded_automation_case(
        "real_backend_history_status_refusal_survives_reopen_without_completion_or_ack", || {
            backend_history_refusal_case(BackendHistoryFault::MissingStatus, true);
            backend_history_refusal_case(BackendHistoryFault::FutureStatus, false);
        },
    );
}

#[test]
fn real_backend_history_alias_ambiguity_survives_reopen_without_completion_or_ack() {
    bounded_automation_case(
        "real_backend_history_alias_ambiguity_survives_reopen_without_completion_or_ack", || {
            backend_history_refusal_case(BackendHistoryFault::MalformedAlias, true);
            backend_history_refusal_case(BackendHistoryFault::DuplicateOriginal, false);
            backend_history_refusal_case(BackendHistoryFault::CrossRowAlias, true);
        },
    );
}

#[test]
fn real_backend_history_invalid_image_and_tail_refuse_without_completion_or_ack() {
    bounded_automation_case(
        "real_backend_history_invalid_image_and_tail_refuse_without_completion_or_ack", || {
            backend_history_refusal_case(BackendHistoryFault::InvalidTail, false);
            backend_history_refusal_case(BackendHistoryFault::InvalidJson, true);
            backend_history_refusal_case(BackendHistoryFault::InvalidUtf8, false);
        },
    );
}

#[cfg(unix)]
#[test]
fn checked_backend_delivery_rejects_nonregular_files_before_reading() {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::{FileTypeExt, MetadataExt};

    bounded_automation_case(
        "checked_backend_delivery_rejects_nonregular_files_before_reading", || {
            for config_fault in [true, false] {
                let fixture = ShippingAutomation::new("nonregular backend delivery input");
                let baseline = seed_backend_history(&fixture);
                let path = if config_fault {
                    fixture.store.config_path(&fixture.automation_id)
                } else {
                    fixture.store.runs_path(&fixture.automation_id)
                };
                fs::remove_file(&path).unwrap();
                let native_path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
                // The existing owned child bounds the actual read. If either
                // checked boundary regresses to display/file reading, no peer
                // opens this FIFO and the child must fail its original deadline.
                let result = unsafe { libc::mkfifo(native_path.as_ptr(), 0o600) };
                assert_eq!(result, 0, "create owned FIFO: {}", io::Error::last_os_error());
                let before = fs::symlink_metadata(&path).unwrap();
                assert!(before.file_type().is_fifo());
                let backend_id = stable_automation_id(&fixture.agent_id, &fixture.automation_id);
                let target = fixture.runtime.find_automation_for_backend_delivery(
                    &fixture.agent_id, &backend_id,
                );
                if config_fault {
                    assert!(target.is_err(), "a matching nonregular definition is unavailable");
                } else {
                    let target = target.unwrap().expect("definition-only discovery does not read history");
                    assert_eq!(target.automation.id, fixture.automation_id);
                    assert!(target.automation.runs.is_empty());
                }
                for run_id in [CUSTODY_RUN_UUIDS[0], CUSTODY_RUN_UUIDS[1], CUSTODY_RUN_UUIDS[2], "new-run"] {
                    assert!(fixture.runtime.read_run_for_delivery(
                        &fixture.agent_id, &fixture.automation_id, run_id,
                    ).is_err(), "nonregular custody input cannot prove match or absence");
                }
                let after = fs::symlink_metadata(&path).unwrap();
                assert!(after.file_type().is_fifo());
                assert_eq!((after.dev(), after.ino()), (before.dev(), before.ino()));
                if config_fault {
                    assert_eq!(fs::read(fixture.store.runs_path(&fixture.automation_id)).unwrap(), baseline.history);
                } else {
                    assert_eq!(fs::read(fixture.store.config_path(&fixture.automation_id)).unwrap(), baseline.config);
                }
                assert_eq!(fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap(), baseline.cron);
                assert!(fixture.observed.lock().unwrap().is_empty());
                assert!(!fixture.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
            }
        },
    );
}

#[test]
fn real_backend_history_repair_restores_terminal_replay_and_distinct_run() {
    bounded_automation_case(
        "real_backend_history_repair_restores_terminal_replay_and_distinct_run", || {
            struct ReplayTransport { delivery: Value, calls: Mutex<Vec<(String, Value)>> }
            impl AutomationsBackendTransport for ReplayTransport {
                fn post_json(&self, path: &str, body: &Value) -> Result<Value, AutomationsBackendError> {
                    self.calls.lock().unwrap().push((path.into(), body.clone()));
                    match path {
                        "/sand/automation-events/poll" if body["ackRunUuids"] == json!([]) => Ok(self.delivery.clone()),
                        "/sand/automation-events/poll" => Ok(json!({"events":[]})),
                        "/sand/automation-runs/complete" => Ok(json!({})),
                        _ => Err(AutomationsBackendError::Transport(format!("unexpected repair request {path}"))),
                    }
                }
            }
            let fixture = ShippingAutomation::new("explicit backend history repair");
            let admitted = seed_backend_history(&fixture);
            let raw = raw_backend_history_fault(BackendHistoryFault::FutureStatus, &admitted.history);
            fs::write(fixture.store.runs_path(&fixture.automation_id), &raw).unwrap();
            let refused = CustodyBackend::start(
                &fixture.root, "before-explicit-repair", &fixture.runtime,
                backend_history_delivery(&fixture, false, &CUSTODY_RUN_UUIDS), None,
            );
            refused.assert_abandoned_with_reason_after(0, "run_history_unavailable");
            refused.runtime.stop();
            refused.transport.assert_no_completion_or_ack();
            assert_eq!(fs::read(fixture.store.runs_path(&fixture.automation_id)).unwrap(), raw);
            assert!(fixture.observed.lock().unwrap().is_empty());
            drop(refused);

            // Explicit repair restores the original evidence before the real
            // owners settle it; repeated/self aliases remain one claimant.
            let mut repaired: Vec<Value> = serde_json::from_slice(&admitted.history).unwrap();
            repaired[0]["coalescedRunIds"] = json!([
                CUSTODY_RUN_UUIDS[0], CUSTODY_RUN_UUIDS[1], CUSTODY_RUN_UUIDS[1], CUSTODY_RUN_UUIDS[2],
            ]);
            let repaired_raw = serde_json::to_vec(&repaired).unwrap();
            fs::write(fixture.store.runs_path(&fixture.automation_id), &repaired_raw).unwrap();
            let running = CustodyBackend::start(
                &fixture.root, "explicitly-repaired-running", &fixture.runtime,
                backend_history_delivery(&fixture, false, &CUSTODY_RUN_UUIDS), None,
            );
            running.assert_abandoned_after(0);
            running.runtime.stop();
            running.transport.assert_no_completion_or_ack();
            assert_eq!(fs::read(fixture.store.runs_path(&fixture.automation_id)).unwrap(), repaired_raw);
            assert_eq!(fixture.workers.list_cron_session_runs(&fixture.agent_id).unwrap(), admitted.cron);
            assert!(fixture.observed.lock().unwrap().is_empty());
            drop(running);
            assert!(fixture.workers.finish_cron_session_run(
                &fixture.agent_id, &fixture.automation_id, CUSTODY_RUN_UUIDS[0], "ok", 200.0, None,
            ).unwrap());
            SandAgentSessionStore::new(Arc::clone(&fixture.workers))
                .with_admitted_filesystem_mutation(&fixture.agent_id, || {
                    assert!(fixture.store.finish_run(
                        &fixture.automation_id, CUSTODY_RUN_UUIDS[0], "ok", 200.0, None,
                    ).map_err(|error| error.to_string())?.is_some());
                    Ok(())
                }).unwrap();
            let terminal = CustodyPersistedState::capture(
                &fixture.workers, &fixture.store, &fixture.agent_id, &fixture.automation_id,
            );
            terminal.preserve_evidence(&fixture.root, "explicitly-repaired-terminal");
            const DISTINCT: &str = "00000000-0000-4000-8000-000000000104";
            let ids = [CUSTODY_RUN_UUIDS[0], CUSTODY_RUN_UUIDS[1], CUSTODY_RUN_UUIDS[2], DISTINCT];
            let delivery = backend_history_delivery(&fixture, false, &ids);
            fixture.runtime.dispose();
            fixture.workers.shutdown();
            let owners = ReopenedCustodyOwners::open(&fixture.root, &fixture.agent_id);
            let transport = Arc::new(ReplayTransport { delivery, calls: Mutex::new(Vec::new()) });
            let mut backend = AutomationFireBackendRuntime::new(transport.clone());
            let mut fires = Vec::<BackendAutomationFire>::new();
            assert_eq!(backend.tick(1_000, false, false, |fire| {
                fires.push(fire.clone()); true
            }).unwrap(), ids.len());
            let executions = AtomicUsize::new(0);
            for fire in fires {
                let entries = owners.runtime.list_all_automations().unwrap();
                let prepared = prepare_backend_fire(&entries, &fire, |entry| {
                    sand_cloud_definition(&entry.agent_id, &ScheduledCloudAutomation {
                        id: entry.automation.id.clone(), name: entry.automation.name.clone(),
                        prompt: entry.automation.prompt.clone(), is_enabled: entry.automation.is_enabled,
                        trigger: entry.automation.trigger.clone(),
                    }, None).map(|definition| definition.hash)
                }, |entry, id| owners.runtime.read_run_for_delivery(&entry.agent_id, &entry.automation.id, id));
                let completion = if fire.id == DISTINCT {
                    let PreparedBackendFire::Schedule { agent_id, automation_id, run_uuid, scheduled_for_ms } = prepared else {
                        panic!("repaired history must admit the distinct schedule: {prepared:?}");
                    };
                    assert_eq!(owners.runtime.run_server_scheduled_automation_with(
                        &agent_id, &automation_id, run_uuid, scheduled_for_ms.map(|value| value as f64), |_| {
                            executions.fetch_add(1, Ordering::SeqCst);
                            Ok(AutomationExecutionResult::CompletedWithFences { fences: clear_fences() })
                        },
                    ), Ok(Some(FireAutomationOutcome::Ok)));
                    FireCompletion::succeeded()
                } else {
                    assert_eq!(prepared, PreparedBackendFire::Complete {
                        completion: FireCompletion::succeeded(), reason: "existing_run",
                    });
                    FireCompletion::succeeded()
                };
                backend.complete(&fire.id, completion).unwrap();
            }
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            assert_eq!(backend.tick(1_001, false, false, |_| panic!("ACK poll must not redispatch")).unwrap(), 0);
            assert_eq!(backend.pending_len(), 0);
            let calls = transport.calls.lock().unwrap();
            fs::write(fixture.root.join("repair-backend-requests.json"), serde_json::to_vec_pretty(&*calls).unwrap()).unwrap();
            assert_eq!(calls.first().unwrap(), &("/sand/automation-events/poll".into(), json!({"ackRunUuids":[]})));
            assert_eq!(calls.iter().filter(|(path, _)| path == "/sand/automation-runs/complete").count(), ids.len());
            assert_eq!(calls.last().unwrap(), &("/sand/automation-events/poll".into(), json!({"ackRunUuids":ids})));
            let observations = owners.observed.lock().unwrap();
            assert_eq!(observations.len(), 1);
            assert_eq!(observations[0].automation_run_id.as_deref(), Some(DISTINCT));
            let rows = owners.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
            assert_eq!(rows.iter().find(|row| row.run_id == CUSTODY_RUN_UUIDS[0]), terminal.cron.first());
            assert_eq!(rows.iter().find(|row| row.run_id == DISTINCT).unwrap().state, "terminal");
            assert_eq!(owners.store.read_runs(&fixture.automation_id).iter()
                .find(|run| run.id == CUSTODY_RUN_UUIDS[0]).unwrap().coalesced_run_ids,
                Some(vec![CUSTODY_RUN_UUIDS[0].into(), CUSTODY_RUN_UUIDS[1].into(), CUSTODY_RUN_UUIDS[1].into(), CUSTODY_RUN_UUIDS[2].into()]));
        },
    );
}
