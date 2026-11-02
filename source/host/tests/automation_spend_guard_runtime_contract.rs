use std::fs;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::automations::automation::AutomationSpec;
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::automation_event_fires::DroppedFire;
use mahayana_host_runtime::extensions::transcript::automation_run_path::{
    AutomationExecutionResult, AutomationRunTrigger, FireAutomationOutcome,
};
use mahayana_host_runtime::extensions::transcript::automation_runtime::AutomationRuntime;
use mahayana_host_runtime::extensions::transcript::sand_automation_spend_guard::{
    SPEND_GUARD_IDLE_TTL_MS, SPEND_GUARD_PAUSE_DELAY_MS, SPEND_GUARD_SNOOZE_MS,
};
use mahayana_host_runtime::extensions::transcript::transcript_entry_ids::{
    TranscriptEntryIdKind, next_entry_id,
};
use mahayana_host_runtime::storage::store_db::get_sand_agent_db_write_generation;
use serde_json::json;

const CHILD_CASE_ENV: &str = "FABUSHI_SPEND_GUARD_CONTRACT_CASE";
const CHILD_ROOT_ENV: &str = "FABUSHI_SPEND_GUARD_CONTRACT_ROOT";

fn root(label: &str) -> std::path::PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let base = std::env::var_os(CHILD_ROOT_ENV)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join(format!(
        "fabushi-spend-guard-runtime-{label}-{}-{n}",
        std::process::id()
    ))
}

struct OwnedSpendGuardChild {
    child: Child,
    root: std::path::PathBuf,
}

impl Drop for OwnedSpendGuardChild {
    fn drop(&mut self) {
        let exited = match self.child.try_wait() {
            Ok(Some(_)) => true,
            _ => self.child.kill().is_ok() && self.child.wait().is_ok(),
        };
        if exited {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn bounded_spend_guard_case(test_name: &str, run: impl FnOnce()) {
    if std::env::var(CHILD_CASE_ENV).ok().as_deref() == Some(test_name) {
        run();
        let child_root = std::env::var_os(CHILD_ROOT_ENV).expect("owned fixture root");
        fs::write(std::path::PathBuf::from(child_root).join("completed"), test_name)
            .expect("record completion after the real-owner assertions");
        return;
    }

    // A recursive owner lock must fail within a bound. Only this exact test
    // child and its private fixture directory belong to the watchdog.
    let child_root = root("child");
    fs::create_dir_all(&child_root).expect("private spend-guard fixture root");
    let child = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(CHILD_CASE_ENV, test_name)
        .env(CHILD_ROOT_ENV, &child_root)
        .spawn()
        .expect("start bounded spend-guard case");
    let mut owned = OwnedSpendGuardChild { child, root: child_root };
    let deadline = Instant::now() + Duration::from_secs(45);
    let status = loop {
        if let Some(status) = owned.child.try_wait().expect("observe owned child") {
            break status;
        }
        assert!(Instant::now() < deadline, "real spend-guard case timed out: {test_name}");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(status.success(), "real spend-guard case failed: {test_name}: {status}");
    assert_eq!(
        fs::read_to_string(owned.root.join("completed")).expect("case completion receipt"),
        test_name,
        "a child that selected no test is not evidence",
    );
}

fn prepare_away_agent(
    workers: &ProductionSessionWorkers,
    agent_id: &str,
    now_ms: f64,
) {
    let last_viewed = now_ms - SPEND_GUARD_IDLE_TTL_MS - 10_000.0;
    workers
        .mark_agent_viewed(agent_id, last_viewed, false)
        .expect("mark viewed");
    for index in 0..15 {
        workers
            .mark_agent_activity(agent_id, last_viewed + index as f64 + 1.0)
            .expect("mark unread activity");
    }
}

#[test]
fn background_run_nudges_then_pauses_and_resume_answer_restores_only_guard_paused_routines() {
    bounded_spend_guard_case(
        "background_run_nudges_then_pauses_and_resume_answer_restores_only_guard_paused_routines",
        exercise_background_nudge_pause_and_resume,
    );
}

fn exercise_background_nudge_pause_and_resume() {
    let root = root("pause-resume");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let sessions = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = sessions.create_session(None, "user", None).expect("agent");
    let runtime = AutomationRuntime::new(Arc::clone(&workers));
    let drops = Arc::new(Mutex::new(Vec::<DroppedFire>::new()));
    runtime.set_dropped_fire_reporter(Some({
        let drops = Arc::clone(&drops);
        Arc::new(move |drop| drops.lock().expect("drops").push(drop))
    }));

    let mut ids = Vec::new();
    for name in ["Daily one", "Daily two"] {
        let (created, _) = runtime
            .create_agent_automation(
                &agent.id,
                &AutomationSpec {
                    name: name.into(),
                    prompt: "Do the task".into(),
                    trigger: json!({"type":"cron","schedule":"0 9 * * *"}),
                    is_enabled: Some(true),
                },
            )
            .expect("create automation");
        ids.push(created[0].id.clone());
    }

    let now_ms = 1_900_000_000_000.0;
    prepare_away_agent(&workers, &agent.id, now_ms);
    let first = runtime
        .run_background_automation_with(
            &agent.id,
            &ids[0],
            AutomationRunTrigger::Schedule,
            Vec::new(),
            Some("scheduled-run-1".into()),
            Vec::new(),
            now_ms,
            None,
            |prompt| {
                assert!(prompt.contains("The app has already asked them directly"));
                Ok(AutomationExecutionResult::Completed)
            },
        )
        .expect("first background run");
    assert_eq!(first, Some(FireAutomationOutcome::Ok));

    let state = workers
        .get_agent_automation_spend_guard_state(&agent.id)
        .expect("nudge state");
    assert_eq!(state.nudged_at_ms, Some(now_ms));
    assert_eq!(state.card_entry_ids.len(), 1);
    let nudge_entry_id = state.card_entry_ids[0].clone();
    let nudge_entry = workers
        .read_agent_transcript_entries(&agent.id)
        .expect("nudge transcript")
        .into_iter()
        .find(|entry| {
            entry.get("id").and_then(serde_json::Value::as_str)
                == Some(nudge_entry_id.as_str())
        })
        .expect("nudge card");
    let session_key = workers
        .read_session_work_admission(&agent.id)
        .expect("admission")
        .expect("admission record")
        .session_key;
    assert_eq!(
        nudge_entry.get("sessionKey").and_then(serde_json::Value::as_str),
        Some(session_key.as_str())
    );

    let called = Arc::new(AtomicBool::new(false));
    let called_in_run = Arc::clone(&called);
    let paused_at = now_ms + SPEND_GUARD_PAUSE_DELAY_MS + 1.0;
    let second = runtime
        .run_background_automation_with(
            &agent.id,
            &ids[0],
            AutomationRunTrigger::Schedule,
            Vec::new(),
            Some("scheduled-run-2".into()),
            Vec::new(),
            paused_at,
            None,
            move |_| {
                called_in_run.store(true, Ordering::SeqCst);
                Ok(AutomationExecutionResult::Completed)
            },
        )
        .expect("paused background run");
    assert_eq!(second, None);
    assert!(!called.load(Ordering::SeqCst));
    let drops = drops.lock().expect("drops");
    assert_eq!(drops.len(), 1);
    assert_eq!(drops[0].reason, "user_away_paused");
    assert_eq!(drops[0].trigger, AutomationRunTrigger::Schedule);
    assert_eq!(drops[0].run_uuid.as_deref(), Some("scheduled-run-2"));
    drop(drops);

    let paused = workers
        .get_agent_automation_spend_guard_state(&agent.id)
        .expect("paused state");
    assert_eq!(paused.paused_automation_ids.len(), 2);
    assert_eq!(paused.card_entry_ids.len(), 2);
    assert!(paused.card_entry_ids.contains(&nudge_entry_id));
    let paused_entry_id = paused.card_entry_ids.last().expect("paused card").clone();
    let listed = runtime.get_agent_automations(&agent.id).expect("automations");
    assert!(listed.iter().all(|automation| !automation.is_enabled));

    let answer_at = paused_at + 10.0;
    let owner = workers.open_agent_db_owner(&agent.id).expect("owner before fence");
    let store = workers.open_automation_store(&agent.id).expect("automation store");
    let config_before = ids
        .iter()
        .map(|id| fs::read(store.config_path(id)).expect("configuration before fence"))
        .collect::<Vec<_>>();
    let history_before = ids
        .iter()
        .map(|id| fs::read(store.runs_path(id)).ok())
        .collect::<Vec<_>>();
    let transcript_before = owner.get_transcript_entries().expect("transcript before fence");
    let unread_before = owner.get_unread_state().expect("unread before fence");
    workers.begin_agent_delete(&agent.id).expect("publish deletion fence");

    for durable_only in [false, true] {
        if durable_only {
            workers.clear_agent_delete_fast_path(&agent.id);
        }
        let background_error = runtime
            .run_background_automation_with(
                &agent.id,
                &ids[0],
                AutomationRunTrigger::Schedule,
                Vec::new(),
                Some(format!("deletion-refused-{durable_only}")),
                Vec::new(),
                answer_at,
                None,
                |_| panic!("deletion-fenced background work must not execute"),
            )
            .expect_err("deletion-fenced Spend Guard must refuse background work");
        assert!(background_error.contains("new session work is not admitted"));
        let answer_error = runtime
            .handle_spend_guard_answer(
                &agent.id,
                &paused_entry_id,
                "spend-guard:resume",
                answer_at,
            )
            .expect_err("a valid card cannot resume routines during deletion");
        assert!(answer_error.contains("new session work is not admitted"));
        assert_eq!(
            owner.get_automation_spend_guard_state().expect("guard state under fence"),
            paused,
        );
        assert_eq!(owner.get_transcript_entries().expect("transcript under fence"), transcript_before);
        assert_eq!(owner.get_unread_state().expect("unread under fence"), unread_before);
        for (index, id) in ids.iter().enumerate() {
            assert_eq!(fs::read(store.config_path(id)).expect("retained config"), config_before[index]);
            assert_eq!(fs::read(store.runs_path(id)).ok(), history_before[index]);
        }
    }
    workers.end_agent_delete(&agent.id).expect("release deletion fence");

    let ack = runtime
        .handle_spend_guard_answer(
            &agent.id,
            &paused_entry_id,
            "spend-guard:resume",
            answer_at,
        )
        .expect("answer")
        .expect("ack");
    assert!(ack.contains("start the paused routines back up"));
    let resumed = runtime.get_agent_automations(&agent.id).expect("resumed");
    assert!(resumed.iter().all(|automation| automation.is_enabled));
    let state = workers
        .get_agent_automation_spend_guard_state(&agent.id)
        .expect("resumed state");
    assert_eq!(
        state.snoozed_until_ms,
        Some(answer_at + SPEND_GUARD_SNOOZE_MS)
    );
    assert!(state.paused_automation_ids.is_empty());
    assert!(state.card_entry_ids.is_empty());

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn spend_guard_card_acknowledgment_requires_insertion_for_bound_and_legacy_sessions() {
    bounded_spend_guard_case(
        "spend_guard_card_acknowledgment_requires_insertion_for_bound_and_legacy_sessions",
        || {
            for (legacy, collision) in [(false, false), (false, true), (true, false), (true, true)] {
                let root = root(&format!("card-legacy-{legacy}-collision-{collision}"));
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
                            name: "Card conflict".into(),
                            prompt: "Do the task".into(),
                            trigger: json!({"type":"cron","schedule":"@daily"}),
                            is_enabled: Some(true),
                        },
                    )
                    .expect("create routine");
                let owner = workers.open_agent_db_owner(&agent.id).expect("owner");
                if legacy {
                    assert!(owner.delete_kv("sessionWorkAdmission").expect("legacy fixture"));
                }
                let now_ms = 1_900_000_000_000.0;
                prepare_away_agent(&workers, &agent.id, now_ms);
                let projected = workers.read_agent_transcript_entries(&agent.id).expect("projection");
                let card_id = next_entry_id(&projected, TranscriptEntryIdKind::SendMessage);
                if collision {
                    let hidden = json!({
                        "id": card_id,
                        "kind": "background-result-receipt",
                        "fixture": "existing hidden control row",
                    });
                    assert!(owner.append_transcript_entry(&hidden).expect("control-row fixture"));
                }
                assert_eq!(
                    workers.read_agent_transcript_entries(&agent.id).expect("same projection"),
                    projected,
                );
                let store = workers.open_automation_store(&agent.id).expect("store");
                let before = owner.get_transcript_entries().expect("rows before refusal");
                let guard_before = owner.get_automation_spend_guard_state().expect("guard before");
                let unread_before = owner.get_unread_state().expect("unread before");
                let generation_before = get_sand_agent_db_write_generation(owner.db_path());
                let observed = Arc::new(Mutex::new(Vec::new()));
                let subscription = if legacy {
                    None
                } else {
                    let weak = Arc::downgrade(&owner);
                    let observed = Arc::clone(&observed);
                    Some(owner.subscribe_metadata("automationSpendGuardState", Arc::new(move || {
                        let owner = weak.upgrade().expect("Runtime retains the DB owner");
                        observed.lock().expect("callback observations").push((
                            owner.get_automation_spend_guard_state().expect("same-owner committed guard"),
                            owner.get_unread_state().expect("same-owner committed unread"),
                            owner.get_transcript_entries().expect("same-owner committed card"),
                        ));
                    })))
                };
                let applied = runtime
                    .spend_guard()
                    .apply_with_store(&agent.id, &store, &created[0].id, now_ms);
                let applied = if collision {
                    let error = applied.expect_err("INSERT OR IGNORE is not a newly issued card");
                    assert!(error.contains("spend-guard card was not inserted"));
                    assert_eq!(owner.get_transcript_entries().expect("retained rows"), before);
                    assert_eq!(owner.get_automation_spend_guard_state().expect("retained guard"), guard_before);
                    assert_eq!(owner.get_unread_state().expect("retained unread"), unread_before);
                    assert_eq!(get_sand_agent_db_write_generation(owner.db_path()), generation_before);
                    assert!(observed.lock().expect("no refused publication").is_empty());
                    assert!(owner.delete_transcript_entry(&card_id).expect("repair the conflicting fixture row"));
                    runtime.spend_guard()
                        .apply_with_store(&agent.id, &store, &created[0].id, now_ms)
                        .expect("a rejected card must not suppress a subsequent actual nudge")
                } else {
                    applied.expect("healthy card insertion")
                };
                assert_eq!(applied.issued_card_entry_id.as_deref(), Some(card_id.as_str()));
                assert!(!applied.paused);
                let entries = owner.get_transcript_entries().expect("inserted card");
                assert_eq!(entries.len(), before.len() + usize::from(!collision));
                let entry = entries.iter().find(|entry| entry["id"] == card_id).expect("issued card row");
                assert_eq!(entry["kind"], "send-message");
                assert_eq!(entry["message"]["type"], "widget");
                assert_eq!(entry.get("sessionKey").is_none(), legacy);
                let state = owner.get_automation_spend_guard_state().expect("committed guard");
                assert_eq!(state.nudged_at_ms, Some(now_ms));
                assert_eq!(state.card_entry_ids, vec![card_id]);
                let unread = owner.get_unread_state().expect("card activity");
                assert_eq!(unread.last_activity_at, now_ms);
                assert_eq!(unread.unread_count, unread_before.unread_count + 1.0);
                if !legacy {
                    let callbacks = observed.lock().expect("committed callback observations");
                    assert_eq!(callbacks.len(), 1, "one successful bound publication after any refusal");
                    assert_eq!(callbacks[0], (state, unread, entries));
                }
                drop(subscription);
                workers.shutdown();
                let _ = fs::remove_dir_all(root);
            }
        },
    );
}

#[test]
fn manual_run_is_not_blocked_by_spend_guard() {
    let root = root("manual");
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
                name: "Manual".into(),
                prompt: "Do it".into(),
                trigger: json!({"type":"cron","schedule":"@daily"}),
                is_enabled: Some(true),
            },
        )
        .expect("create");
    prepare_away_agent(&workers, &agent.id, 1_900_000_000_000.0);

    let outcome = runtime
        .run_agent_automation_now_with(&agent.id, &created[0].id, |_| {
            Ok(AutomationExecutionResult::Completed)
        })
        .expect("manual");
    assert_eq!(outcome, Some(FireAutomationOutcome::Ok));
    assert!(
        workers
            .get_agent_automation_spend_guard_state(&agent.id)
            .expect("guard")
            .nudged_at_ms
            .is_none()
    );

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
