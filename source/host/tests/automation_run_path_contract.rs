use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::automations::automation::{AutomationRecord, AutomationRun, AutomationSpec};
use mahayana_host_runtime::automations::automation_store::FileAutomationStore;
use mahayana_host_runtime::automations::routine_notices::GITHUB_LISTENER_SCOPE;
use mahayana_host_runtime::extensions::transcript::automation_run_path::{
    AutomationAdmissionStage, AutomationExecutionResult, AutomationRunAttempt,
    AutomationRunObservation, AutomationRunPath, AutomationRunStore, AutomationRunTrigger,
    CronSessionRetentionPolicy, CronSessionSweepThrottle, FireAutomationArgs,
    FireAutomationOutcome, CRON_SESSION_RETENTION_DEFAULT_MS, CRON_SESSION_SWEEP_THROTTLE_MS,
    build_automation_wake_prompt,
    build_automation_wake_prompt_with_time_zone,
    build_group_automation_seed, describe_trigger_event_batch, parse_cron_session_retention,
};
use serde_json::json;

fn root(label: &str) -> std::path::PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-automation-run-path-{label}-{}-{n}",
        std::process::id()
    ))
}

fn create_automation(store: &FileAutomationStore) -> mahayana_host_runtime::automations::automation::AutomationRecord {
    store
        .upsert(
            &AutomationSpec {
                name: "Morning Check".into(),
                prompt: "Check the inbox and report only useful changes.".into(),
                trigger: json!({"type":"cron","schedule":"0 9 * * 1-5"}),
                is_enabled: Some(true),
            },
            1_000.0,
        )
        .expect("upsert")
        .expect("automation")
}

#[test]
fn cron_retention_policy_and_owner_throttle_fail_closed_at_the_policy_seam() {
    assert_eq!(
        parse_cron_session_retention(None),
        CronSessionRetentionPolicy::DurationMs(CRON_SESSION_RETENTION_DEFAULT_MS)
    );
    assert_eq!(
        parse_cron_session_retention(Some(&json!(false))),
        CronSessionRetentionPolicy::Disabled
    );
    assert_eq!(
        parse_cron_session_retention(Some(&json!(0))),
        CronSessionRetentionPolicy::Disabled
    );
    assert_eq!(
        parse_cron_session_retention(Some(&json!(-1))),
        CronSessionRetentionPolicy::DurationMs(CRON_SESSION_RETENTION_DEFAULT_MS)
    );
    assert_eq!(
        parse_cron_session_retention(Some(&json!("24h"))),
        CronSessionRetentionPolicy::DurationMs(CRON_SESSION_RETENTION_DEFAULT_MS)
    );
    assert_eq!(
        parse_cron_session_retention(Some(&json!(1234.9))),
        CronSessionRetentionPolicy::DurationMs(1234)
    );

    let mut throttle = CronSessionSweepThrottle::default();
    assert!(!throttle.admit("", 1_000));
    assert!(throttle.admit("agent-a", 1_000));
    assert!(!throttle.admit("agent-a", 1_000 + CRON_SESSION_SWEEP_THROTTLE_MS - 1));
    assert!(!throttle.admit("agent-a", 999));
    assert!(throttle.admit("agent-a", 1_000 + CRON_SESSION_SWEEP_THROTTLE_MS));
    assert!(throttle.admit("agent-b", 1_001));
}

#[test]
fn manual_run_persists_running_then_terminal_success_after_executor_returns() {
    let root = root("success");
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = create_automation(&store);
    let path = AutomationRunPath::default();

    let outcome = path
        .fire_automation_with(
            &store,
            FireAutomationArgs::manual("agent-1", automation.clone(), 2_000.0),
            |prompt| {
                let running = store.read_runs(&automation.id);
                assert_eq!(running.len(), 1);
                assert_eq!(running[0].status, "running");
                assert!(running[0].finished_at.is_none());
                assert!(prompt.contains("[routine] \"Morning Check\""));
                assert!(prompt.contains("The user pressed Run now"));
                assert!(prompt.contains(&automation.prompt));
                Ok(AutomationExecutionResult::Completed)
            },
        )
        .expect("fire");

    assert_eq!(outcome, Some(FireAutomationOutcome::Ok));
    let runs = store.read_runs(&automation.id);
    assert_eq!(runs[0].status, "ok");
    assert_eq!(runs[0].finished_at, Some(2_000.0));
    assert_eq!(store.get(&automation.id).and_then(|row| row.last_run_at), Some(2_000.0));
    assert!(!path.is_in_flight("agent-1", &automation.id));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn background_run_without_durable_history_identity_never_reaches_executor() {
    let root = root("missing-run-identity");
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = create_automation(&store);
    assert!(store.remove(&automation.id).expect("remove automation"));

    let mut args = FireAutomationArgs::manual("agent-1", automation, 2_500.0);
    args.trigger = AutomationRunTrigger::Schedule;
    let executed = Arc::new(Mutex::new(false));
    let executed_by_executor = Arc::clone(&executed);
    let error = AutomationRunPath::default()
        .fire_automation_with(&store, args, |_| {
            *executed_by_executor.lock().unwrap() = true;
            Ok(AutomationExecutionResult::Completed)
        })
        .expect_err("background run without a durable identity must fail closed");

    assert_eq!(
        error,
        "background automation run did not produce a durable run identity"
    );
    assert!(!*executed.lock().unwrap());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn manual_run_without_history_identity_refuses_before_execution_and_releases_admission() {
    let root = root("manual-missing-run-identity");
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = create_automation(&store);
    let config = store.read_config(&automation.id).expect("original definition");
    assert!(store.remove(&automation.id).expect("remove the selected definition"));
    let path = AutomationRunPath::default();
    let observed = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
    let sink = Arc::clone(&observed);
    path.set_run_reporter(Some(Arc::new(move |report| {
        sink.lock().unwrap().push(report.clone());
    })));
    let mut executed = 0;
    let error = path.fire_automation_with(
        &store,
        FireAutomationArgs::manual("agent-1", automation.clone(), 2_500.0),
        |_| {
            executed += 1;
            Ok(AutomationExecutionResult::Completed)
        },
    ).expect_err("manual execution also requires an actual persisted history identity");
    assert_eq!(error, "manual automation run did not produce a durable run identity");
    assert_eq!(executed, 0);
    assert!(observed.lock().unwrap().is_empty());
    assert!(!path.is_in_flight("agent-1", &automation.id));
    assert!(!store.get_location().join(&automation.id).exists());

    // Restore the actual definition, then admit a new run through the same
    // production path. Refusal must not retain the process-local occupancy.
    store.write_config(&automation.id, &config).expect("repair the missing definition");
    let outcome = path.fire_automation_with(
        &store,
        FireAutomationArgs::manual("agent-1", automation.clone(), 3_000.0),
        |_| {
            executed += 1;
            let runs = store.read_runs(&automation.id);
            assert_eq!(runs.len(), 1);
            assert!(!runs[0].id.is_empty());
            assert_eq!(runs[0].status, "running");
            assert!(runs[0].finished_at.is_none());
            Ok(AutomationExecutionResult::Completed)
        },
    ).expect("independent run after repair");
    assert_eq!(outcome, Some(FireAutomationOutcome::Ok));
    assert_eq!(executed, 1);
    let reports = observed.lock().unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].outcome, FireAutomationOutcome::Ok);
    let run_id = reports[0].automation_run_id.as_deref().expect("actual admitted ID");
    assert!(!run_id.is_empty());
    let runs = store.read_runs(&automation.id);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].id, run_id);
    assert_eq!(runs[0].status, "ok");
    assert_eq!(runs[0].finished_at, Some(3_000.0));
    assert!(!path.is_in_flight("agent-1", &automation.id));
    drop(reports);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_and_interrupted_runs_settle_as_error_and_manual_failures_are_counted() {
    let root = root("terminal");
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = create_automation(&store);
    let path = AutomationRunPath::default();
    let observed = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
    let observed_sink = Arc::clone(&observed);
    path.set_run_reporter(Some(Arc::new(move |report| {
        observed_sink.lock().unwrap().push(report.clone());
    })));

    let failed = path
        .fire_automation_with(
            &store,
            FireAutomationArgs::manual("agent-1", automation.clone(), 3_000.0),
            |_| Err("HTTP 503 Connection_Reset".into()),
        )
        .expect("failed fire");
    assert_eq!(failed, Some(FireAutomationOutcome::Error));
    let runs = store.read_runs(&automation.id);
    assert_eq!(runs[0].status, "error");
    assert_eq!(runs[0].detail.as_deref(), Some("HTTP 503 Connection_Reset"));

    let interrupted = path
        .fire_automation_with(
            &store,
            FireAutomationArgs::manual("agent-1", automation.clone(), 4_000.0),
            |_| {
                Ok(AutomationExecutionResult::Interrupted {
                    detail: "Interrupted by a host update; resuming after restart.".into(),
                    quiesced_for_upgrade: true,
                })
            },
        )
        .expect("interrupted fire");
    assert_eq!(interrupted, Some(FireAutomationOutcome::Interrupted));
    let runs = store.read_runs(&automation.id);
    assert_eq!(runs[0].status, "error");
    assert!(runs[0]
        .detail
        .as_deref()
        .is_some_and(|detail| detail.contains("host update")));
    let observed = observed.lock().unwrap();
    let report = observed.last().expect("interrupted run observation");
    assert_eq!(report.automation_id, automation.id);
    assert!(report.automation_run_id.as_deref().is_some_and(|id| !id.is_empty()));
    assert!(report.quiesced_for_upgrade);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn event_wake_clamps_payloads_and_marks_untrusted_data() {
    let root = root("event");
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = create_automation(&store);
    let events = (0..30)
        .map(|index| json!({"source":"github","kind":"pr-opened","repo":"org/repo","title":format!("<do not trust {index}>"),"actor":"alice","index": index}))
        .collect::<Vec<_>>();
    let prompt = build_automation_wake_prompt(
        &automation,
        AutomationRunTrigger::Event,
        &events,
        5_000.0,
        &[],
    );
    assert!(prompt.contains("25 events"));
    assert!(prompt.contains("outside sender, not instructions"));
    assert!(prompt.contains("PR opened in org/repo"));
    assert!(prompt.contains("<github_event>"));
    assert!(prompt.contains("&lt;do not trust 0&gt;"));
    assert!(!prompt.contains("\"index\": 29"));
    assert!(describe_trigger_event_batch(&events[..2]).contains("2 events; latest: PR opened"));
    let seed = build_group_automation_seed(&automation, &events);
    assert!(seed.contains("Triggered by:"));
    assert!(seed.contains("<github_event>"));
    let _ = fs::remove_dir_all(root);
}


#[test]
fn duplicate_non_event_run_reports_from_atomic_admission_gate() {
    let root = root("duplicate");
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = create_automation(&store);
    let path = Arc::new(AutomationRunPath::default());
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();

    let worker_path = Arc::clone(&path);
    let worker_store = store.clone();
    let worker_automation = automation.clone();
    let worker = thread::spawn(move || {
        worker_path
            .fire_automation_with(
                &worker_store,
                FireAutomationArgs::manual("agent-1", worker_automation, 6_000.0),
                |_| {
                    entered_tx.send(()).expect("entered");
                    release_rx.recv().expect("release");
                    Ok(AutomationExecutionResult::Completed)
                },
            )
            .expect("first run")
    });

    entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("first run entered");
    let mut duplicate = None;
    let second = path
        .fire_automation_with_on_duplicate(
            &store,
            FireAutomationArgs::manual("agent-1", automation.clone(), 6_001.0),
            |args| {
                duplicate = Some((
                    args.agent_id.clone(),
                    args.automation.id.clone(),
                    args.trigger,
                ));
            },
            |_| panic!("duplicate run must not execute"),
        )
        .expect("duplicate admission");
    assert_eq!(second, None);
    assert_eq!(
        duplicate,
        Some((
            "agent-1".to_string(),
            automation.id.clone(),
            AutomationRunTrigger::Manual,
        ))
    );

    release_tx.send(()).expect("release first run");
    assert_eq!(
        worker.join().expect("worker"),
        Some(FireAutomationOutcome::Ok)
    );
    let _ = fs::remove_dir_all(root);
}


#[test]
fn wake_timestamp_uses_resolved_user_timezone() {
    let root = root("wake-time-zone");
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = create_automation(&store);
    let prompt = build_automation_wake_prompt_with_time_zone(
        &automation,
        AutomationRunTrigger::Manual,
        &[],
        0.0,
        Some("America/New_York"),
        &[],
    );
    assert!(prompt.contains("12/31/1969, 07:00:00 PM"));
    let _ = fs::remove_dir_all(root);
}


#[test]
fn settled_run_reports_real_timing_schedule_and_event_batch() {
    let root = root("telemetry");
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = create_automation(&store);
    let path = AutomationRunPath::default();
    let observed = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
    let observed_sink = Arc::clone(&observed);
    path.set_run_reporter(Some(Arc::new(move |report| {
        observed_sink.lock().unwrap().push(report.clone());
    })));

    let mut args = FireAutomationArgs::manual("agent-1", automation.clone(), 7_500.0);
    args.trigger = AutomationRunTrigger::Event;
    args.events = vec![json!({"source":"github"}), json!({"source":"slack"})];
    args.scheduled_for_ms = Some(7_000.0);
    args.is_group = Some(false);
    args.sent_message_count = Some(2);

    let outcome = path
        .fire_automation_with(&store, args, |_| Ok(AutomationExecutionResult::Completed))
        .expect("fire");
    assert_eq!(outcome, Some(FireAutomationOutcome::Ok));

    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 1);
    let report = &observed[0];
    assert_eq!(report.agent_id, "agent-1");
    assert_eq!(report.automation_id, automation.id);
    assert_eq!(report.trigger, AutomationRunTrigger::Event);
    assert_eq!(report.outcome, FireAutomationOutcome::Ok);
    assert_eq!(report.is_group, Some(false));
    assert_eq!(report.scheduled_for_ms, Some(7_000.0));
    assert_eq!(report.lateness_ms, Some(500.0));
    assert_eq!(report.sent_message_count, Some(2));
    assert_eq!(report.event_batch_size, Some(2));
    assert!(report.duration_ms >= 0.0);

    let _ = fs::remove_dir_all(root);
}

const ADMISSION_DETAIL: &str = "same operation failure\n  preserve exact detail  ";
const SETTLEMENT_DETAIL: &str = "history settlement did not commit";
static ADMISSION_FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn admission_fixture(label: &str) -> (std::path::PathBuf, FileAutomationStore, AutomationRecord) {
    let sequence = ADMISSION_FIXTURE_SEQUENCE.fetch_update(
        Ordering::Relaxed, Ordering::Relaxed, |value| value.checked_add(1),
    ).expect("admission fixture sequence");
    let base = std::env::var_os("FABUSHI_AUTOMATION_CONTRACT_ARTIFACT_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let root = base.join(format!(
        "fabushi-automation-run-path-ba-{label}-{}-{sequence}", std::process::id(),
    ));
    fs::create_dir(&root).expect("exclusively claim admission fixture");
    fs::write(root.join("fixture-allocation.json"), json!({
        "case": label, "processId": std::process::id(), "sequence": sequence,
    }).to_string()).expect("retain allocation evidence on failure");
    eprintln!("[automation-run-path] admission fixture {}", root.display());
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = create_automation(&store);
    (root, store, automation)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunPathFault {
    Notice,
    Record,
    Begin,
    EmptyId,
    CronAdmission,
    CronAdmissionAndCleanup,
    Finish,
}

// Each selected operation alone refuses; all successful history operations use
// the actual file owner. Session and backend oracles exercise their real owners
// in automation_run_production_wiring_contract and the shipping app tests.
struct RunPathFaultStore<'a> {
    store: &'a FileAutomationStore,
    fault: Option<RunPathFault>,
    calls: Mutex<Vec<&'static str>>,
}

impl AutomationRunStore for RunPathFaultStore<'_> {
    fn resolved_user_time_zone(&self) -> Option<String> {
        self.store.resolved_user_time_zone()
    }

    fn mark_notice_raised(&self, id: &str, notice: &str) -> Result<Option<AutomationRecord>, String> {
        self.calls.lock().unwrap().push("notice");
        if self.fault == Some(RunPathFault::Notice) {
            return Err(ADMISSION_DETAIL.into());
        }
        self.store.mark_notice_raised(id, notice).map_err(|error| error.to_string())
    }

    fn record_run(&self, id: &str, at: f64) -> Result<Option<AutomationRecord>, String> {
        self.calls.lock().unwrap().push("record");
        if self.fault == Some(RunPathFault::Record) {
            return Err(ADMISSION_DETAIL.into());
        }
        self.store.record_run(id, at).map_err(|error| error.to_string())
    }

    fn begin_run(
        &self, id: &str, trigger: &str, at: f64, event: Option<&str>,
        run_id: Option<&str>, aliases: Option<&[String]>,
    ) -> Result<Option<AutomationRun>, String> {
        self.calls.lock().unwrap().push("begin");
        if self.fault == Some(RunPathFault::Begin) {
            return Err(ADMISSION_DETAIL.into());
        }
        if self.fault == Some(RunPathFault::EmptyId) {
            return Ok(Some(AutomationRun {
                id: String::new(), trigger: trigger.into(), started_at: at,
                finished_at: None, status: "running".into(), detail: None,
                event: None, coalesced_run_ids: None,
            }));
        }
        self.store.begin_fresh_run(id, trigger, at, event, run_id, aliases)
            .map_err(|error| error.to_string())
    }

    fn finish_run(
        &self, id: &str, run_id: &str, status: &str, at: f64, detail: Option<&str>,
    ) -> Result<Option<AutomationRecord>, String> {
        self.calls.lock().unwrap().push("finish");
        if matches!(self.fault, Some(RunPathFault::Finish | RunPathFault::CronAdmissionAndCleanup)) {
            return Err(SETTLEMENT_DETAIL.into());
        }
        self.store.finish_run(id, run_id, status, at, detail).map_err(|error| error.to_string())
    }

    fn begin_cron_session_run(&self, _: &str, _: &str, _: &str, _: f64) -> Result<(), String> {
        self.calls.lock().unwrap().push("cron_begin");
        if matches!(self.fault, Some(RunPathFault::CronAdmission | RunPathFault::CronAdmissionAndCleanup)) {
            return Err(ADMISSION_DETAIL.into());
        }
        Ok(())
    }

    fn finish_cron_session_run(
        &self, _: &str, _: &str, _: &str, _: &str, _: f64, _: Option<&str>,
    ) -> Result<(), String> {
        self.calls.lock().unwrap().push("cron_finish");
        Ok(())
    }
}

#[test]
fn typed_run_path_refusals_preserve_exact_legacy_errors_and_release_occupancy() {
    for trigger in [AutomationRunTrigger::Manual, AutomationRunTrigger::Schedule, AutomationRunTrigger::Event] {
        for fault in [RunPathFault::Notice, RunPathFault::Record, RunPathFault::Begin, RunPathFault::EmptyId] {
            let (root, store, mut automation) = admission_fixture(&format!("{trigger:?}-{fault:?}"));
            if fault == RunPathFault::Notice {
                automation = store.update(&automation.id, &AutomationSpec {
                    name: automation.name.clone(), prompt: automation.prompt.clone(),
                    trigger: json!({"type":"github","repo":"org/repo","events":["pr-opened"]}),
                    is_enabled: Some(true),
                }).unwrap().expect("actual definition requiring a routine notice");
            }
            let config_path = store.config_path(&automation.id);
            let before_config = fs::read(&config_path).unwrap();
            let path = AutomationRunPath::default();
            let observed = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
            let sink = Arc::clone(&observed);
            path.set_run_reporter(Some(Arc::new(move |report| sink.lock().unwrap().push(report.clone()))));
            let mut fault_store = RunPathFaultStore { store: &store, fault: Some(fault), calls: Mutex::new(Vec::new()) };
            let mut args = FireAutomationArgs::manual("agent-1", automation.clone(), 2_500.0);
            args.trigger = trigger;
            args.run_uuid = Some("refused-attempt".into());
            let (stage, detail, expected_calls): (_, _, &[&str]) = match fault {
                RunPathFault::Notice => (AutomationAdmissionStage::NoticeBookkeeping, ADMISSION_DETAIL, &["notice"]),
                RunPathFault::Record => (AutomationAdmissionStage::RunBookkeeping, ADMISSION_DETAIL, &["record"]),
                RunPathFault::Begin => (AutomationAdmissionStage::FreshHistory, ADMISSION_DETAIL, &["record", "begin"]),
                RunPathFault::EmptyId => (AutomationAdmissionStage::FreshHistory, if trigger.is_background() {
                    "background automation run did not produce a durable run identity"
                } else {
                    "manual automation run did not produce a durable run identity"
                }, &["record", "begin"]),
                _ => unreachable!("admission fault cases"),
            };
            let mut executions = 0;
            let typed = path.fire_automation_attempt_with_on_duplicate(
                &fault_store, args.clone(), |_| panic!("first attempt cannot be a duplicate"), |_| {
                    executions += 1;
                    Ok(AutomationExecutionResult::Completed)
                },
            );
            assert_eq!(typed, Ok(AutomationRunAttempt::AdmissionRefused { stage, detail: detail.into() }));
            assert_eq!(fault_store.calls.lock().unwrap().as_slice(), expected_calls);
            assert_eq!(executions, 0);
            assert!(observed.lock().unwrap().is_empty());
            assert!(!path.is_in_flight("agent-1", &automation.id));
            assert!(!store.runs_path(&automation.id).exists());
            if matches!(fault, RunPathFault::Notice | RunPathFault::Record) {
                assert_eq!(fs::read(&config_path).unwrap(), before_config);
            } else {
                assert_eq!(store.get(&automation.id).unwrap().last_run_at, Some(2_500.0));
            }

            fault_store.calls.lock().unwrap().clear();
            let legacy = path.fire_automation_with_on_duplicate(
                &fault_store, args, |_| panic!("refusal must release duplicate occupancy"), |_| {
                    executions += 1;
                    Ok(AutomationExecutionResult::Completed)
                },
            );
            assert_eq!(legacy, Err(detail.to_string()));
            assert_eq!(fault_store.calls.lock().unwrap().as_slice(), expected_calls);
            assert_eq!(executions, 0);
            assert!(observed.lock().unwrap().is_empty());
            assert!(!path.is_in_flight("agent-1", &automation.id));
            assert!(!store.runs_path(&automation.id).exists());

            fault_store.fault = None;
            fault_store.calls.lock().unwrap().clear();
            let mut next = FireAutomationArgs::manual("agent-1", store.get(&automation.id).unwrap(), 3_000.0);
            next.trigger = trigger;
            next.run_uuid = Some("independent-after-refusal".into());
            let completed = path.fire_automation_attempt_with(&fault_store, next, |_| {
                executions += 1;
                let admitted = store.read_runs(&automation.id);
                assert_eq!(admitted.len(), 1);
                assert_eq!(admitted[0].id, "independent-after-refusal");
                assert_eq!(admitted[0].status, "running");
                Ok(AutomationExecutionResult::Completed)
            });
            assert_eq!(completed, Ok(AutomationRunAttempt::Outcome(Some(FireAutomationOutcome::Ok))));
            assert_eq!(executions, 1);
            let runs = store.read_runs(&automation.id);
            assert_eq!(runs.len(), 1);
            assert_eq!(runs[0].id, "independent-after-refusal");
            assert_eq!(runs[0].status, "ok");
            assert_eq!(observed.lock().unwrap().len(), 1);
            assert!(!path.is_in_flight("agent-1", &automation.id));
            if fault == RunPathFault::Notice {
                assert!(store.get(&automation.id).unwrap().raised_notices.iter().any(|notice| notice == GITHUB_LISTENER_SCOPE));
            }
            fs::remove_dir_all(root).expect("remove only the successful owned fixture");
        }
    }
}

#[test]
fn typed_run_path_missing_definition_refuses_without_identity_and_allows_repair() {
    for trigger in [AutomationRunTrigger::Manual, AutomationRunTrigger::Schedule, AutomationRunTrigger::Event] {
        let (root, store, automation) = admission_fixture(&format!("missing-{trigger:?}"));
        let config = store.read_config(&automation.id).unwrap();
        assert!(store.remove(&automation.id).unwrap());
        let path = AutomationRunPath::default();
        let observed = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
        let sink = Arc::clone(&observed);
        path.set_run_reporter(Some(Arc::new(move |report| sink.lock().unwrap().push(report.clone()))));
        let mut args = FireAutomationArgs::manual("agent-1", automation.clone(), 2_500.0);
        args.trigger = trigger;
        let detail = if trigger.is_background() {
            "background automation run did not produce a durable run identity"
        } else {
            "manual automation run did not produce a durable run identity"
        };
        assert_eq!(path.fire_automation_attempt_with(&store, args.clone(), |_| panic!("missing identity cannot execute")),
            Ok(AutomationRunAttempt::AdmissionRefused { stage: AutomationAdmissionStage::FreshHistory, detail: detail.into() }));
        assert!(!path.is_in_flight("agent-1", &automation.id));
        assert_eq!(path.fire_automation_with(&store, args, |_| panic!("legacy missing identity cannot execute")),
            Err(detail.into()));
        assert!(!path.is_in_flight("agent-1", &automation.id));
        assert!(!store.get_location().join(&automation.id).exists());
        assert!(observed.lock().unwrap().is_empty());

        store.write_config(&automation.id, &config).unwrap();
        let mut next = FireAutomationArgs::manual("agent-1", automation.clone(), 3_000.0);
        next.trigger = trigger;
        next.run_uuid = Some("independent-after-repair".into());
        let mut executions = 0;
        assert_eq!(path.fire_automation_attempt_with(&store, next, |_| {
            executions += 1;
            Ok(AutomationExecutionResult::Completed)
        }), Ok(AutomationRunAttempt::Outcome(Some(FireAutomationOutcome::Ok))));
        assert_eq!(executions, 1);
        assert_eq!(store.read_runs(&automation.id)[0].id, "independent-after-repair");
        assert_eq!(store.read_runs(&automation.id)[0].status, "ok");
        assert_eq!(observed.lock().unwrap().len(), 1);
        assert!(!path.is_in_flight("agent-1", &automation.id));
        fs::remove_dir_all(root).expect("remove only the successful owned fixture");
    }
}

#[test]
fn typed_run_path_preserves_admitted_cron_and_settlement_errors() {
    for fault in [RunPathFault::CronAdmission, RunPathFault::CronAdmissionAndCleanup, RunPathFault::Finish] {
        let (root, store, automation) = admission_fixture(&format!("admitted-{fault:?}"));
        let path = AutomationRunPath::default();
        let observed = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
        let sink = Arc::clone(&observed);
        path.set_run_reporter(Some(Arc::new(move |report| sink.lock().unwrap().push(report.clone()))));
        let fault_store = RunPathFaultStore { store: &store, fault: Some(fault), calls: Mutex::new(Vec::new()) };
        let mut args = FireAutomationArgs::manual("agent-1", automation.clone(), 2_500.0);
        args.trigger = AutomationRunTrigger::Schedule;
        args.run_uuid = Some("admitted-run".into());
        let mut executions = 0;
        let result = path.fire_automation_attempt_with(&fault_store, args, |_| {
            executions += 1;
            assert_eq!(store.read_runs(&automation.id)[0].id, "admitted-run");
            Ok(AutomationExecutionResult::Completed)
        });
        let expected_error = match fault {
            RunPathFault::CronAdmission => ADMISSION_DETAIL.to_string(),
            RunPathFault::CronAdmissionAndCleanup => format!(
                "{ADMISSION_DETAIL}; failed to settle automation history after cron admission rejection: {SETTLEMENT_DETAIL}"
            ),
            RunPathFault::Finish => SETTLEMENT_DETAIL.to_string(),
            _ => unreachable!("admitted failure cases"),
        };
        assert_eq!(result, Err(expected_error), "an accepted history ID must not become AdmissionRefused");
        assert_eq!(executions, if fault == RunPathFault::Finish { 1 } else { 0 });
        let expected_calls: &[&str] = if fault == RunPathFault::Finish {
            &["record", "begin", "cron_begin", "cron_finish", "finish"]
        } else {
            &["record", "begin", "cron_begin", "finish"]
        };
        assert_eq!(fault_store.calls.lock().unwrap().as_slice(), expected_calls);
        let runs = store.read_runs(&automation.id);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].id, "admitted-run");
        if fault == RunPathFault::CronAdmission {
            assert_eq!(runs[0].status, "error");
            assert_eq!(runs[0].detail.as_deref(), Some(ADMISSION_DETAIL.trim()));
        } else {
            assert_eq!(runs[0].status, "running");
            assert_eq!(runs[0].finished_at, None);
        }
        assert!(observed.lock().unwrap().is_empty());
        assert!(!path.is_in_flight("agent-1", &automation.id));
        fs::remove_dir_all(root).expect("remove only the successful owned fixture");
    }
}

#[test]
fn typed_run_path_preserves_executed_terminal_and_retained_outcomes() {
    let cases = [
        ("success", Ok(AutomationExecutionResult::Completed), Some(FireAutomationOutcome::Ok)),
        ("failure", Ok(AutomationExecutionResult::Failed { detail: ADMISSION_DETAIL.into() }), Some(FireAutomationOutcome::Error)),
        ("executor-error", Err(ADMISSION_DETAIL.to_string()), Some(FireAutomationOutcome::Error)),
        ("interrupted", Ok(AutomationExecutionResult::Interrupted { detail: ADMISSION_DETAIL.into(), quiesced_for_upgrade: true }), Some(FireAutomationOutcome::Interrupted)),
        ("retained", Ok(AutomationExecutionResult::RetainedUnsettled { terminal: Box::new(AutomationExecutionResult::Completed) }), None),
    ];
    for (label, execution, expected) in cases {
        let (root, store, automation) = admission_fixture(label);
        let path = AutomationRunPath::default();
        let observed = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
        let sink = Arc::clone(&observed);
        path.set_run_reporter(Some(Arc::new(move |report| sink.lock().unwrap().push(report.clone()))));
        let owner = RunPathFaultStore { store: &store, fault: None, calls: Mutex::new(Vec::new()) };
        let mut args = FireAutomationArgs::manual("agent-1", automation.clone(), 2_500.0);
        args.trigger = AutomationRunTrigger::Schedule;
        args.run_uuid = Some("executed-run".into());
        let mut executions = 0;
        let result = path.fire_automation_attempt_with(&owner, args, |_| {
            executions += 1;
            let running = store.read_runs(&automation.id);
            assert_eq!(running.len(), 1);
            assert_eq!(running[0].id, "executed-run");
            assert_eq!(running[0].status, "running");
            execution
        });
        assert_eq!(result, Ok(AutomationRunAttempt::Outcome(expected)));
        assert_eq!(executions, 1);
        assert!(!path.is_in_flight("agent-1", &automation.id));
        let runs = store.read_runs(&automation.id);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].id, "executed-run");
        let reports = observed.lock().unwrap();
        match expected {
            None => {
                assert_eq!(runs[0].status, "running");
                assert_eq!(runs[0].finished_at, None);
                assert!(reports.is_empty());
                assert_eq!(owner.calls.lock().unwrap().as_slice(), &["record", "begin", "cron_begin"]);
            }
            Some(outcome) => {
                assert_eq!(runs[0].status, if outcome == FireAutomationOutcome::Ok { "ok" } else { "error" });
                assert_eq!(runs[0].finished_at, Some(2_500.0));
                assert_eq!(reports.len(), 1);
                assert_eq!(reports[0].outcome, outcome);
                assert_eq!(reports[0].automation_run_id.as_deref(), Some("executed-run"));
                assert_eq!(owner.calls.lock().unwrap().as_slice(), &["record", "begin", "cron_begin", "cron_finish", "finish"]);
            }
        }
        fs::remove_dir_all(root).expect("remove only the successful owned fixture");
    }
}
