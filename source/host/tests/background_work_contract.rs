use std::sync::{Arc, Mutex};

use mahayana_host_runtime::runner::background_work::{
    BackgroundWakeup, BackgroundWakeupPayload, BackgroundWorkRecord,
    RevivingBackgroundWorkRegistry, SHELL_REWATCH_POLL_DEFAULT_MS,
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
