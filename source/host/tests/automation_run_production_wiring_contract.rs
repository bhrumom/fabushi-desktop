use std::fs;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::automations::automation::AutomationSpec;
use mahayana_host_runtime::automations::automation_store::FileAutomationStore;
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::automation_run_path::{
    AutomationExecutionResult, AutomationRunFenceSnapshot, AutomationRunObservation,
    AutomationRunTrigger, FireAutomationOutcome,
};
use mahayana_host_runtime::extensions::transcript::automation_runtime::AutomationRuntime;
use serde_json::json;

const CHILD_CASE_ENV: &str = "FABUSHI_AUTOMATION_CONTRACT_CASE";
const CHILD_ROOT_ENV: &str = "FABUSHI_AUTOMATION_CONTRACT_ROOT";

fn root() -> std::path::PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let base = std::env::var_os(CHILD_ROOT_ENV)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join(format!(
        "fabushi-automation-run-production-{}-{n}",
        std::process::id()
    ))
}

struct OwnedAutomationChild {
    child: Child,
    root: std::path::PathBuf,
}

impl Drop for OwnedAutomationChild {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn bounded_automation_case(test_name: &str, run: impl FnOnce()) {
    if std::env::var(CHILD_CASE_ENV).ok().as_deref() == Some(test_name) {
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
    fs::create_dir_all(&child_root).expect("private automation fixture root");
    let child = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(CHILD_CASE_ENV, test_name)
        .env(CHILD_ROOT_ENV, &child_root)
        .spawn()
        .expect("start bounded real-owner automation case");
    let mut owned = OwnedAutomationChild { child, root: child_root };
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
    fn new() -> Self {
        let root = root();
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let sessions = SandAgentSessionStore::new(Arc::clone(&workers));
        let agent = sessions.create_session(None, "user", None).expect("agent");
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
        let _ = fs::remove_dir_all(&self.root);
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

fn history_failure_then_independent_run(
    trigger: AutomationRunTrigger,
    fault: HistoryFault,
    execution: Result<AutomationExecutionResult, String>,
    expected_cron_outcome: &str,
    known_fences: bool,
) {
    let fixture = ShippingAutomation::new();
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
            let fixture = ShippingAutomation::new();
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
        let fixture = ShippingAutomation::new();
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
