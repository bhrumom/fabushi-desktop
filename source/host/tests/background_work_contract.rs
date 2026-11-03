use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use mahayana_host_runtime::runner::background_work::{
    BackgroundWakeup, BackgroundWakeupPayload, BackgroundWorkRecord,
    BackgroundShellBackgroundCompletion, BackgroundShellWatchOptions,
    BackgroundShellWatchOutcome, CloudAgentBackgroundCompletion, CloudAgentPendingWatch,
    CloudAgentWatchOptions, CloudAgentWatchOutcome, RevivingBackgroundWorkRegistry, RunnerBackgroundShellWatches,
    RunnerCloudAgentWatches, SHELL_REWATCH_POLL_DEFAULT_MS,
    derive_background_subagent_title, format_steer_prompt, parse_shell_terminal_footer,
    shell_rewatch_poll_ms,
};
use serde_json::json;

fn record(id: &str, kind: &str, state: &str) -> BackgroundWorkRecord {
    BackgroundWorkRecord {
        id: id.into(),
        kind: kind.into(),
        state: state.into(),
        owner_id: None,
        metadata: Some(json!({"title":"test"})),
        abort: None,
    }
}

fn wake(id: &str, conversation_id: Option<&str>, kind: &str, reason: Option<&str>) -> BackgroundWakeup {
    BackgroundWakeup {
        id: Some(id.into()),
        conversation_id: conversation_id.map(ToOwned::to_owned),
        payload: BackgroundWakeupPayload {
            kind: kind.into(),
            reason: reason.map(ToOwned::to_owned),
            task_id: id.into(),
            title: Some(format!("task {id}")),
            status: None,
            detail: None,
            output_path: None,
        },
    }
}

#[test]
fn title_and_steer_prompt_match_frozen_runner_contract() {
    assert_eq!(derive_background_subagent_title("  one \n two\tthree  "), "one two three");
    assert_eq!(derive_background_subagent_title("   "), "Background task");
    let long = "x".repeat(100);
    let title = derive_background_subagent_title(&long);
    assert_eq!(title.chars().count(), 80);
    assert!(title.ends_with('…'));

    assert_eq!(
        format_steer_prompt("  go left  "),
        "[Steering message from the parent agent that dispatched you]\n\ngo left\n\nTake this into account and continue your task from where you are — do not start over."
    );
}

#[test]
fn shell_terminal_footer_and_poll_interval_follow_frozen_rules() {
    assert_eq!(shell_rewatch_poll_ms(None), SHELL_REWATCH_POLL_DEFAULT_MS);
    assert_eq!(shell_rewatch_poll_ms(Some(" 2500 ")), 2500);
    assert_eq!(shell_rewatch_poll_ms(Some("0")), SHELL_REWATCH_POLL_DEFAULT_MS);

    let complete = parse_shell_terminal_footer("body\n---\nexit_code: 7\n---");
    assert!(complete.is_complete);
    assert_eq!(complete.exit_code, Some(7));
    assert!(!complete.is_stream_failure);

    let malformed_exit = parse_shell_terminal_footer("body\n---\nexit_code: nope\n---");
    assert!(malformed_exit.is_complete);
    assert_eq!(malformed_exit.exit_code, None);

    let failed = parse_shell_terminal_footer("body\n---\nerror: stream lost\nended_at: 123\n---");
    assert!(failed.is_complete);
    assert!(failed.is_stream_failure);

    assert!(!parse_shell_terminal_footer("body only").is_complete);
}

#[test]
fn shell_terminal_footer_handles_canonical_producer_endings_and_actual_poll() {
    use mahayana_host_runtime::runner::shell_terminal_watch::{
        ShellTerminalPollState, ShellWatchStatus, TerminalFileSnapshot,
    };

    // Keep the daemon's trailing LF unchanged. The old parser selected its
    // closing delimiter and could not settle these canonical producer bytes.
    for ending in ["", "\n", "\r\n"] {
        for code in [0, 7] {
            let content = format!(
                "---\nstatus: running\n---\nstdout-prefix\nstderr-tail\n\
                 \n---\nexit_code: {code}\nelapsed_ms: 19\n\
                 ended_at: 2026-11-03T00:00:00.000Z\n---{ending}"
            );
            let footer = parse_shell_terminal_footer(&content);
            assert!(footer.is_complete, "producer ending {ending:?} must be consumed");
            assert_eq!(footer.exit_code, Some(code));
            assert!(!footer.is_stream_failure);
            let settlement = ShellTerminalPollState::new(10).observe_snapshot(
                11, "/term/7.txt", &TerminalFileSnapshot { exists: true, content },
            ).expect("actual Runner poll must settle the unmodified terminal snapshot");
            assert_eq!(settlement.status, if code == 0 {
                ShellWatchStatus::Success
            } else {
                ShellWatchStatus::Error
            });
            assert_eq!(settlement.output_path.as_deref(), Some("/term/7.txt"));
            assert_eq!(settlement.detail, if code == 0 { None } else {
                Some("exit_code=7".into())
            });
        }

        let failure = format!(
            "actual-prefix\n---\nerror: background capture failed\n\
             ended_at: 2026-11-03T00:00:00.000Z\n---{ending}"
        );
        let footer = parse_shell_terminal_footer(&failure);
        assert!(footer.is_complete);
        assert!(footer.is_stream_failure);
        assert_eq!(footer.exit_code, None);
        let settlement = ShellTerminalPollState::new(10).observe_snapshot(
            11, "/term/8.txt", &TerminalFileSnapshot { exists: true, content: failure },
        ).expect("the existing explicit failure footer must settle as error");
        assert_eq!(settlement.status, ShellWatchStatus::Error);
        assert_eq!(settlement.detail.as_deref(),
            Some("the command's output stream failed; read its output file for the error"));
    }
}

#[test]
fn shell_terminal_footer_preserves_last_footer_and_incomplete_snapshot_refusal() {
    use mahayana_host_runtime::runner::shell_terminal_watch::{
        ShellTerminalPollState, TerminalFileSnapshot,
    };

    let previous = "earlier output\n---\nexit_code: 0\n---\nmore output";
    let latest = format!("{previous}\n---\nexit_code: 23\n---\n");
    assert_eq!(parse_shell_terminal_footer(&latest).exit_code, Some(23),
        "only the final complete footer determines the observation");
    let malformed = parse_shell_terminal_footer("body\n---\nexit_code: nope\n---\n");
    assert!(malformed.is_complete);
    assert_eq!(malformed.exit_code, None);
    assert!(!malformed.is_stream_failure);

    for content in [
        "body only",
        "body\n---\nexit_code: 0\n",
        "body\n---\nexit_code: 0\n--\n",
        "body\n---\nexit_code: 0\n---\ntrailing data",
        "body\n---\nexit_code: 0\n---\n\n",
        "body\n---\nerror: lost\n---\n",
        "body\n---\nerror: lost\nended_at: \n---\n",
        "body\n---\n---\n",
    ] {
        assert!(!parse_shell_terminal_footer(content).is_complete,
            "incomplete or trailing snapshot cannot complete: {content:?}");
        assert!(ShellTerminalPollState::new(10).observe_snapshot(
            11, "/term/pending.txt",
            &TerminalFileSnapshot { exists: true, content: content.into() },
        ).is_none(), "the actual poll must retain pending work");
    }
}

#[test]
fn reviving_registry_preserves_work_wakeup_completion_and_quiet_origin_semantics() {
    let completions = Arc::new(Mutex::new(Vec::<(String, Option<String>)>::new()));
    let registrations = Arc::new(Mutex::new(Vec::<(String, Option<String>)>::new()));
    let changed = Arc::new(Mutex::new(0usize));

    let mut registry = RevivingBackgroundWorkRegistry::with_callbacks(
        Some({
            let completions = Arc::clone(&completions);
            Arc::new(move |payload, origin| {
                completions.lock().expect("completion").push((
                    payload.task_id.clone(),
                    origin.map(ToOwned::to_owned),
                ));
            })
        }),
        Some({
            let registrations = Arc::clone(&registrations);
            Arc::new(move |record, origin| {
                registrations.lock().expect("registration").push((
                    record.id.clone(),
                    origin.map(ToOwned::to_owned),
                ));
            })
        }),
        Some({
            let changed = Arc::clone(&changed);
            Arc::new(move || *changed.lock().expect("changed") += 1)
        }),
    );

    registry.upsert_work_for_turn(record("shell-1", "shell", "running"), Some("quiet-a"));
    assert!(registry.has_running_work(Some("shell")));
    assert_eq!(
        registrations.lock().expect("registration").as_slice(),
        &[("shell-1".into(), Some("quiet-a".into()))]
    );

    registry.enqueue(wake("shell-1", Some("agent-a"), "shell", Some("task_progress")));
    assert_eq!(registry.pull("agent-a").len(), 1);
    registry.enqueue(wake("shell-1", Some("agent-a"), "shell", Some("completed")));
    assert_eq!(
        completions.lock().expect("completion").as_slice(),
        &[("shell-1".into(), Some("quiet-a".into()))]
    );
    assert_eq!(registry.pull("agent-a").len(), 1);

    registry.ack(&["shell-1".into()]);
    assert!(registry.pull("agent-a").is_empty());

    registry.enqueue(wake("wake-2", Some("agent-b"), "subagent", None));
    registry.suppress("agent-b", "wake-2");
    assert!(registry.pull("agent-b").is_empty());

    let payload = wake("done-1", None, "subagent", None).payload;
    registry.enqueue_completion(payload.clone());
    assert!(registry.has_pending_completions("agent-a"));
    assert_eq!(registry.drain_completions(), vec![payload]);
    assert!(!registry.has_pending_completions("agent-a"));

    assert!(registry.clear_work("shell-1").is_some());
    assert!(*changed.lock().expect("changed") >= 2);
}


#[test]
fn runner_cloud_agent_watches_match_frozen_ownership_and_settlement_contract() {
    let (await_started_tx, await_started_rx) = mpsc::channel::<(String, bool)>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let pending = Arc::new(Mutex::new(Vec::new()));
    let settled = Arc::new(Mutex::new(Vec::<CloudAgentBackgroundCompletion>::new()));
    let changed = Arc::new(Mutex::new(Vec::<String>::new()));
    let callback_order = Arc::new(Mutex::new(Vec::<&'static str>::new()));

    let watches = RunnerCloudAgentWatches::new(
        Arc::new({
            let release_rx = Arc::clone(&release_rx);
            move |id, after_followup| {
                await_started_tx
                    .send((id.to_string(), after_followup))
                    .expect("await started");
                release_rx
                    .lock()
                    .expect("release rx")
                    .recv_timeout(Duration::from_secs(2))
                    .expect("release");
                CloudAgentWatchOutcome {
                    status: "completed".into(),
                    text: "  finished work  ".into(),
                }
            }
        }),
        Some({
            let pending = Arc::clone(&pending);
            Arc::new(move |watch| pending.lock().expect("pending").push(watch.clone()))
        }),
        Some({
            let settled = Arc::clone(&settled);
            let callback_order = Arc::clone(&callback_order);
            Arc::new(move |completion| {
                callback_order.lock().expect("order").push("settled");
                settled.lock().expect("settled").push(completion);
            })
        }),
        Some({
            let changed = Arc::clone(&changed);
            let callback_order = Arc::clone(&callback_order);
            Arc::new(move |agent_id| {
                callback_order.lock().expect("order").push("changed");
                changed.lock().expect("changed").push(agent_id.to_string());
            })
        }),
    );

    let quiet_origin = json!({"automation":{"id":"routine-1","name":"Daily"}});
    assert!(watches.watch_cloud_agent(
        "agent-a",
        "bc-1",
        CloudAgentWatchOptions::new(Some(quiet_origin.clone()), true),
    ));
    assert!(!watches.watch_cloud_agent(
        "agent-a",
        "bc-1",
        CloudAgentWatchOptions::new(None, false),
    ));
    assert!(watches.is_cloud_watch_armed("agent-a", "bc-1"));
    assert_eq!(
        watches.pending_cloud_agent_watch_ids("agent-a"),
        vec!["bc-1".to_string()]
    );
    let cloud_tasks = watches.async_task_snapshots("agent-a");
    assert_eq!(cloud_tasks.len(), 1);
    assert_eq!(cloud_tasks[0].id, "bc-1");
    assert_eq!(cloud_tasks[0].label, "Cloud agent bc-1");
    assert!(cloud_tasks[0].started_at_ms > 0);
    assert_eq!(
        await_started_rx.recv_timeout(Duration::from_secs(2)).expect("started"),
        ("bc-1".to_string(), true)
    );
    {
        let pending = pending.lock().expect("pending");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].parent_agent_id, "agent-a");
        assert_eq!(pending[0].work_id, "bc-1");
        assert_eq!(pending[0].title, "Cloud agent bc-1");
        assert_eq!(pending[0].quiet_origin, Some(quiet_origin.clone()));
    }

    release_tx.send(()).expect("release watch");
    for _ in 0..100 {
        if !watches.is_cloud_watch_armed("agent-a", "bc-1")
            && settled.lock().expect("settled").len() == 1
            && changed.lock().expect("changed").len() >= 2
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!watches.is_cloud_watch_armed("agent-a", "bc-1"));
    let settled = settled.lock().expect("settled");
    assert_eq!(settled.len(), 1);
    assert_eq!(settled[0].parent_agent_id, "agent-a");
    assert_eq!(settled[0].work_id, "bc-1");
    assert_eq!(settled[0].status, "completed");
    assert_eq!(settled[0].result, "finished work");
    assert_eq!(settled[0].quiet_origin, Some(quiet_origin));
    assert!(changed.lock().expect("changed").len() >= 2);
    assert_eq!(
        *callback_order.lock().expect("order"),
        vec!["changed", "settled", "changed"],
        "completion must settle the durable/live owner before publishing the terminal task-set change",
    );
}

#[test]
fn runner_cloud_agent_rejected_recovery_admission_has_no_pending_notification_or_worker() {
    let (worker_tx, worker_rx) = mpsc::channel::<()>();
    let pending = Arc::new(Mutex::new(0usize));
    let changed = Arc::new(Mutex::new(0usize));
    let settled = Arc::new(Mutex::new(0usize));
    let watches = RunnerCloudAgentWatches::new(
        Arc::new(move |_, _| {
            worker_tx.send(()).expect("unexpected worker started");
            CloudAgentWatchOutcome {
                status: "completed".into(),
                text: "unexpected recovery".into(),
            }
        }),
        Some({
            let pending = Arc::clone(&pending);
            Arc::new(move |_| *pending.lock().expect("pending") += 1)
        }),
        Some({
            let settled = Arc::clone(&settled);
            Arc::new(move |_| *settled.lock().expect("settled") += 1)
        }),
        Some({
            let changed = Arc::clone(&changed);
            Arc::new(move |_| *changed.lock().expect("changed") += 1)
        }),
    );
    let candidates = Mutex::new(Vec::<CloudAgentPendingWatch>::new());
    let quiet_origin = json!({"automation":{"id":"routine-1","name":"Daily"}});

    assert!(!watches.watch_cloud_agent_if_admitted(
        " agent-a ",
        " bc-1 ",
        CloudAgentWatchOptions::new(Some(quiet_origin.clone()), false),
        &|pending| {
            candidates.lock().expect("candidates").push(pending.clone());
            false
        },
    ));

    let candidates = candidates.lock().expect("candidates");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].parent_agent_id, "agent-a");
    assert_eq!(candidates[0].work_id, "bc-1");
    assert_eq!(candidates[0].title, "Cloud agent bc-1");
    assert!(candidates[0].marked_at_ms > 0.0);
    assert_eq!(candidates[0].quiet_origin, Some(quiet_origin));
    assert!(!watches.is_cloud_watch_armed("agent-a", "bc-1"));
    assert!(watches.async_task_snapshots("agent-a").is_empty());
    assert_eq!(*pending.lock().expect("pending"), 0);
    assert_eq!(*changed.lock().expect("changed"), 0);
    drop(watches);
    assert_eq!(
        worker_rx.recv_timeout(Duration::from_secs(2)),
        Err(mpsc::RecvTimeoutError::Disconnected),
        "a rejected admission must not leave a worker holding the await callback",
    );
    assert_eq!(*settled.lock().expect("settled"), 0);
}

#[test]
fn runner_cloud_agent_admitted_recovery_skips_fresh_pending_and_notifies_outside_watch_lock() {
    let (started_tx, started_rx) = mpsc::channel::<(String, bool)>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Mutex::new(release_rx);
    let (settled_tx, settled_rx) = mpsc::channel::<CloudAgentBackgroundCompletion>();
    let (changed_tx, changed_rx) = mpsc::channel();
    let pending = Arc::new(Mutex::new(0usize));
    let owner = Arc::new(Mutex::new(None::<std::sync::Weak<RunnerCloudAgentWatches>>));
    let watches = Arc::new(RunnerCloudAgentWatches::new(
        Arc::new(move |work_id, after_followup| {
            started_tx.send((work_id.into(), after_followup)).expect("started");
            release_rx.lock().expect("release rx")
                .recv_timeout(Duration::from_secs(2)).expect("release");
            CloudAgentWatchOutcome { status: "completed".into(), text: "recovered".into() }
        }),
        Some({
            let pending = Arc::clone(&pending);
            Arc::new(move |_| *pending.lock().expect("pending") += 1)
        }),
        Some(Arc::new(move |completion| settled_tx.send(completion).expect("settled"))),
        Some({
            let owner = Arc::clone(&owner);
            Arc::new(move |agent_id| {
                let watches = owner.lock().expect("owner").as_ref()
                    .and_then(std::sync::Weak::upgrade).expect("live owner");
                // Production task projection also reads this owner from its callback.
                changed_tx.send(watches.async_task_snapshots(agent_id)).expect("changed");
            })
        }),
    ));
    *owner.lock().expect("owner") = Some(Arc::downgrade(&watches));
    let admitted = Arc::new(Mutex::new(Vec::<CloudAgentPendingWatch>::new()));
    let quiet_origin = json!({"automation":{"id":"routine-1","name":"Daily"}});
    let (registered_tx, registered_rx) = mpsc::channel();
    let registration = std::thread::spawn({
        let watches = Arc::clone(&watches);
        let admitted = Arc::clone(&admitted);
        let quiet_origin = quiet_origin.clone();
        move || {
            let registered = watches.watch_cloud_agent_if_admitted(
                "agent-a",
                "bc-1",
                CloudAgentWatchOptions::new(Some(quiet_origin), true),
                &|pending| {
                    admitted.lock().expect("admitted").push(pending.clone());
                    true
                },
            );
            registered_tx.send(registered).expect("registered");
        }
    });

    assert!(registered_rx.recv_timeout(Duration::from_secs(2)).expect("registration must not deadlock"));
    registration.join().expect("registration");
    assert_eq!(started_rx.recv_timeout(Duration::from_secs(2)).expect("started"), ("bc-1".into(), true));
    let tasks = changed_rx.recv_timeout(Duration::from_secs(2)).expect("admitted task projection");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "bc-1");
    assert!(!watches.watch_cloud_agent_if_admitted(
        "agent-a", "bc-1", CloudAgentWatchOptions::new(None, false),
        &|_| panic!("already-armed watches must not repeat durable admission"),
    ));
    let candidate = admitted.lock().expect("admitted")[0].clone();
    assert_eq!(admitted.lock().expect("admitted").len(), 1);
    assert_eq!(candidate.marked_at_ms, tasks[0].started_at_ms as f64);
    assert_eq!(candidate.title, tasks[0].label);
    assert_eq!(candidate.quiet_origin, Some(quiet_origin));
    assert_eq!(*pending.lock().expect("pending"), 0);

    release_tx.send(()).expect("release watch");
    let completion = settled_rx.recv_timeout(Duration::from_secs(2)).expect("settled");
    assert_eq!(completion.parent_agent_id, candidate.parent_agent_id);
    assert_eq!(completion.work_id, candidate.work_id);
    assert_eq!(completion.title, candidate.title);
    assert_eq!(completion.marked_at_ms, candidate.marked_at_ms);
    assert_eq!(completion.quiet_origin, candidate.quiet_origin);
    assert_eq!(completion.result, "recovered");
    assert!(changed_rx.recv_timeout(Duration::from_secs(2)).expect("settled task projection").is_empty());
    assert!(!watches.is_cloud_watch_armed("agent-a", "bc-1"));
    assert_eq!(*pending.lock().expect("pending"), 0);
}

#[test]
fn runner_cloud_agent_watch_cancellation_fences_stale_completion() {
    let (await_started_tx, await_started_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let settled = Arc::new(Mutex::new(Vec::<CloudAgentBackgroundCompletion>::new()));

    let watches = RunnerCloudAgentWatches::new(
        Arc::new({
            let release_rx = Arc::clone(&release_rx);
            move |_id, _after_followup| {
                await_started_tx.send(()).expect("await started");
                release_rx
                    .lock()
                    .expect("release rx")
                    .recv_timeout(Duration::from_secs(2))
                    .expect("release");
                CloudAgentWatchOutcome {
                    status: "error".into(),
                    text: String::new(),
                }
            }
        }),
        None,
        Some({
            let settled = Arc::clone(&settled);
            Arc::new(move |completion| {
                settled.lock().expect("settled").push(completion);
            })
        }),
        None,
    );

    assert!(watches.watch_cloud_agent(
        "agent-a",
        "bc-stale",
        CloudAgentWatchOptions::new(None, false),
    ));
    await_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("started");
    assert!(watches.cancel_cloud_watch("agent-a", "bc-stale"));
    release_tx.send(()).expect("release stale watch");
    std::thread::sleep(Duration::from_millis(30));
    assert!(settled.lock().expect("settled").is_empty());
    assert!(!watches.is_cloud_watch_armed("agent-a", "bc-stale"));
}


#[test]
fn runner_background_shell_watches_dedupe_before_pending_persist_and_fence_settlement() {
    let (started_tx, started_rx) = mpsc::channel::<(String, String)>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let pending = Arc::new(Mutex::new(Vec::new()));
    let settled = Arc::new(Mutex::new(Vec::<BackgroundShellBackgroundCompletion>::new()));
    let changed = Arc::new(Mutex::new(Vec::<String>::new()));
    let callback_order = Arc::new(Mutex::new(Vec::<&'static str>::new()));

    let watches = RunnerBackgroundShellWatches::new(
        Arc::new({
            let release_rx = Arc::clone(&release_rx);
            move |agent_id, shell_id, cancelled| {
                started_tx
                    .send((agent_id.to_string(), shell_id.to_string()))
                    .expect("started");
                release_rx
                    .lock()
                    .expect("release rx")
                    .recv_timeout(Duration::from_secs(2))
                    .expect("release");
                if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                    return None;
                }
                Some(BackgroundShellWatchOutcome {
                    status: "success".into(),
                    detail: None,
                    output_path: Some("/term/42.txt".into()),
                })
            }
        }),
        Some({
            let pending = Arc::clone(&pending);
            Arc::new(move |watch| pending.lock().expect("pending").push(watch.clone()))
        }),
        Some({
            let settled = Arc::clone(&settled);
            let callback_order = Arc::clone(&callback_order);
            Arc::new(move |completion| {
                callback_order.lock().expect("order").push("settled");
                settled.lock().expect("settled").push(completion);
            })
        }),
        Some({
            let changed = Arc::clone(&changed);
            let callback_order = Arc::clone(&callback_order);
            Arc::new(move |agent_id| {
                callback_order.lock().expect("order").push("changed");
                changed.lock().expect("changed").push(agent_id.to_string());
            })
        }),
    );

    let origin = json!({"automation":{"id":"routine-shell"}});
    assert!(watches.watch_background_shell(
        "agent-a",
        "42",
        BackgroundShellWatchOptions::new(Some("compile".into()), Some(origin.clone())),
    ));
    assert!(!watches.watch_background_shell(
        "agent-a",
        "42",
        BackgroundShellWatchOptions::new(Some("compile".into()), Some(origin.clone())),
    ));
    assert!(watches.is_shell_watch_armed("agent-a", "42"));
    assert!(watches.has_running_background_shell_work());
    assert_eq!(
        watches.pending_shell_rewatch_ids("agent-a"),
        vec!["42".to_string()]
    );
    let shell_tasks = watches.async_task_snapshots("agent-a");
    assert_eq!(shell_tasks.len(), 1);
    assert_eq!(shell_tasks[0].id, "42");
    assert_eq!(shell_tasks[0].label, "compile");
    assert!(shell_tasks[0].started_at_ms > 0);
    assert_eq!(
        started_rx.recv_timeout(Duration::from_secs(2)).expect("started"),
        ("agent-a".to_string(), "42".to_string())
    );
    assert_eq!(pending.lock().expect("pending").len(), 1, "duplicate rearm must not persist/project a second pending wake");

    release_tx.send(()).expect("release");
    for _ in 0..100 {
        if !watches.is_shell_watch_armed("agent-a", "42")
            && settled.lock().expect("settled").len() == 1
            && changed.lock().expect("changed").len() >= 2
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!watches.has_running_background_shell_work());
    let settled = settled.lock().expect("settled");
    assert_eq!(settled.len(), 1);
    assert_eq!(settled[0].parent_agent_id, "agent-a");
    assert_eq!(settled[0].work_id, "42");
    assert_eq!(settled[0].title, "compile");
    assert_eq!(settled[0].status, "success");
    assert_eq!(settled[0].output_path.as_deref(), Some("/term/42.txt"));
    assert_eq!(settled[0].quiet_origin, Some(origin));
    assert!(changed.lock().expect("changed").len() >= 2);
    assert_eq!(
        *callback_order.lock().expect("order"),
        vec!["changed", "settled", "changed"],
        "completion must settle the durable/live owner before publishing the terminal task-set change",
    );
}

#[test]
fn runner_background_shell_watch_cancellation_stops_terminal_delivery() {
    let (started_tx, started_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let settled = Arc::new(Mutex::new(Vec::<BackgroundShellBackgroundCompletion>::new()));
    let watches = RunnerBackgroundShellWatches::new(
        Arc::new({
            let release_rx = Arc::clone(&release_rx);
            move |_agent_id, _shell_id, cancelled| {
                started_tx.send(()).expect("started");
                release_rx
                    .lock()
                    .expect("release")
                    .recv_timeout(Duration::from_secs(2))
                    .expect("release");
                if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                    None
                } else {
                    Some(BackgroundShellWatchOutcome {
                        status: "error".into(),
                        detail: Some("late".into()),
                        output_path: None,
                    })
                }
            }
        }),
        None,
        Some({
            let settled = Arc::clone(&settled);
            Arc::new(move |completion| {
                settled.lock().expect("settled").push(completion)
            })
        }),
        None,
    );
    assert!(watches.watch_background_shell(
        "agent-a",
        "stale",
        BackgroundShellWatchOptions::new(None, None),
    ));
    started_rx.recv_timeout(Duration::from_secs(2)).expect("started");
    assert!(watches.cancel_shell_watch("agent-a", "stale"));
    release_tx.send(()).expect("release");
    std::thread::sleep(Duration::from_millis(30));
    assert!(settled.lock().expect("settled").is_empty());
}
