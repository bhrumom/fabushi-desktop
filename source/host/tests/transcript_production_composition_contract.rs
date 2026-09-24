use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::transcript::production_runtime::{
    ProductionSendError, ProductionTranscriptRuntime,
};
use mahayana_host_runtime::extensions::transcript::run_scheduler::{
    RunLane, WatchdogStage,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-production-transcript-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn production_send_runtime_persists_acceptance_and_replays_nonce_without_redispatch() {
    let root = temp_root("accepted");
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let dispatches = AtomicUsize::new(0);
    let persisted = AtomicUsize::new(0);
    let args = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "hello",
        "clientNonce": "nonce-a"
    });

    let first = runtime
        .execute_send(
            &args,
            || {
                dispatches.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::json!({
                    "accepted": true,
                    "operationId": "op-a"
                }))
            },
            |_| {
                persisted.fetch_add(1, Ordering::SeqCst);
                Ok(Some("t0u".to_string()))
            },
        )
        .expect("first send");
    assert_eq!(first["operationId"], "op-a");
    assert_eq!(dispatches.load(Ordering::SeqCst), 1);
    assert_eq!(persisted.load(Ordering::SeqCst), 1);
    assert_eq!(runtime.current_turn_epoch("agent-a"), 1);
    assert_eq!(runtime.in_flight_run_count("agent-a"), 0);

    let replay = runtime
        .execute_send(
            &args,
            || {
                dispatches.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::json!({"accepted": true, "operationId": "wrong"}))
            },
            |_| Ok(Some("t0u".to_string())),
        )
        .expect("replayed send");
    assert_eq!(replay["operationId"], "op-a");
    assert_eq!(dispatches.load(Ordering::SeqCst), 1);

    let status = runtime
        .prompt_acceptance_status(&serde_json::json!({
            "accountSlot": "host",
            "clientNonce": "nonce-a"
        }))
        .expect("acceptance status");
    assert_eq!(status["outcome"], "found");
    assert_eq!(status["record"]["status"], "accepted");
    assert_eq!(status["record"]["echoEntryId"], "t0u");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_send_runtime_clears_failed_unaccepted_nonce_for_retry() {
    let root = temp_root("retry");
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let args = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "retry",
        "clientNonce": "nonce-retry"
    });

    let first = runtime.execute_send(
        &args,
        || Err(ProductionSendError::Internal("transport failed".into())),
        |_| Ok(Some("t0u".to_string())),
    );
    assert!(first.is_err());

    let second = runtime
        .execute_send(
            &args,
            || Ok(serde_json::json!({"accepted": true, "operationId": "op-retry"})),
            |_| Ok(Some("t0u".to_string())),
        )
        .expect("retry dispatch");
    assert_eq!(second["operationId"], "op-retry");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_send_runtime_rejects_nonce_digest_reuse() {
    let root = temp_root("digest");
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let first = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "one",
        "clientNonce": "nonce-shared"
    });
    runtime
        .execute_send(
            &first,
            || Ok(serde_json::json!({"accepted": true, "operationId": "op-one"})),
            |_| Ok(Some("t0u".to_string())),
        )
        .expect("first");

    let mismatch = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "two",
        "clientNonce": "nonce-shared"
    });
    assert!(matches!(
        runtime.execute_send(
            &mismatch,
            || Ok(serde_json::json!({"accepted": true, "operationId": "op-two"})),
            |_| Ok(Some("t0u".to_string())),
        ),
        Err(ProductionSendError::Conflict(_))
    ));
    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_send_runtime_serializes_distinct_user_turns_for_the_same_agent() {
    let root = temp_root("same-agent-queue");
    let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
    let (first_entered_tx, first_entered_rx) = mpsc::channel();
    let (release_first_tx, release_first_rx) = mpsc::channel();
    let second_entered = Arc::new(AtomicBool::new(false));

    let first_runtime = Arc::clone(&runtime);
    let first = thread::spawn(move || {
        let args = serde_json::json!({
            "agentId": "agent-a",
            "prompt": "first",
            "clientNonce": "nonce-first"
        });
        first_runtime
            .execute_send(
                &args,
                || {
                    first_entered_tx.send(()).expect("signal first dispatch");
                    release_first_rx.recv().expect("release first dispatch");
                    Ok(serde_json::json!({
                        "accepted": true,
                        "operationId": "op-first"
                    }))
                },
                |_| Ok(Some("op-first:user".to_string())),
            )
            .expect("first queued send")
    });

    first_entered_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("first dispatch entered");

    let second_runtime = Arc::clone(&runtime);
    let second_entered_flag = Arc::clone(&second_entered);
    let second = thread::spawn(move || {
        let args = serde_json::json!({
            "agentId": "agent-a",
            "prompt": "second",
            "clientNonce": "nonce-second"
        });
        second_runtime
            .execute_send(
                &args,
                || {
                    second_entered_flag.store(true, Ordering::SeqCst);
                    Ok(serde_json::json!({
                        "accepted": true,
                        "operationId": "op-second"
                    }))
                },
                |_| Ok(Some("op-second:user".to_string())),
            )
            .expect("second queued send")
    });

    thread::sleep(Duration::from_millis(150));
    assert!(
        !second_entered.load(Ordering::SeqCst),
        "second same-agent dispatch must remain queued behind the active user turn"
    );
    assert_eq!(runtime.queued_turn_count("agent-a"), 1);

    release_first_tx.send(()).expect("release first");
    assert_eq!(
        first.join().expect("first thread")["operationId"],
        "op-first"
    );
    assert_eq!(
        second.join().expect("second thread")["operationId"],
        "op-second"
    );
    assert!(second_entered.load(Ordering::SeqCst));
    assert!(runtime.is_turn_dispatch_idle("agent-a"));
    assert_eq!(runtime.in_flight_run_count("agent-a"), 0);
    assert_eq!(runtime.current_turn_epoch("agent-a"), 2);

    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_watchdog_escapes_a_wedged_predecessor_and_fences_late_settlement() {
    let root = temp_root("watchdog");
    let runtime = Arc::new(ProductionTranscriptRuntime::with_watchdog(
        Some(&root),
        40,
        25,
    ));
    let (first_entered_tx, first_entered_rx) = mpsc::channel();
    let (release_first_tx, release_first_rx) = mpsc::channel();
    let (second_entered_tx, second_entered_rx) = mpsc::channel();
    let saw_trip = Arc::new(AtomicBool::new(false));
    let saw_escape = Arc::new(AtomicBool::new(false));
    let saw_late_settle = Arc::new(AtomicBool::new(false));

    let first_runtime = Arc::clone(&runtime);
    let late_flag = Arc::clone(&saw_late_settle);
    let first = thread::spawn(move || {
        let args = serde_json::json!({
            "agentId": "agent-watchdog",
            "prompt": "first",
            "clientNonce": "watchdog-first"
        });
        first_runtime.execute_send_with_watchdog(
            &args,
            || {
                first_entered_tx.send(()).expect("signal first");
                release_first_rx.recv().expect("release first");
                Ok(serde_json::json!({"accepted":true,"operationId":"op-first"}))
            },
            |_| Ok(Some("op-first:user".into())),
            move |event| {
                if event.stage == WatchdogStage::LateSettle {
                    late_flag.store(true, Ordering::SeqCst);
                }
                false
            },
        )
    });

    first_entered_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("first dispatch entered");

    let second_runtime = Arc::clone(&runtime);
    let trip_flag = Arc::clone(&saw_trip);
    let escape_flag = Arc::clone(&saw_escape);
    let second = thread::spawn(move || {
        let args = serde_json::json!({
            "agentId": "agent-watchdog",
            "prompt": "second",
            "clientNonce": "watchdog-second"
        });
        second_runtime.execute_send_with_watchdog(
            &args,
            || {
                second_entered_tx.send(()).expect("signal second");
                Ok(serde_json::json!({"accepted":true,"operationId":"op-second"}))
            },
            |_| Ok(Some("op-second:user".into())),
            move |event| {
                match event.stage {
                    WatchdogStage::Trip => trip_flag.store(true, Ordering::SeqCst),
                    WatchdogStage::Escape => escape_flag.store(true, Ordering::SeqCst),
                    WatchdogStage::LateSettle => {}
                }
                false
            },
        )
    });

    second_entered_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("watchdog must release the queued user turn");
    assert!(saw_trip.load(Ordering::SeqCst));
    assert!(saw_escape.load(Ordering::SeqCst));
    assert_eq!(
        second.join().expect("second thread").expect("second result")["operationId"],
        "op-second"
    );

    release_first_tx.send(()).expect("release predecessor");
    assert_eq!(
        first.join().expect("first thread").expect("first result")["operationId"],
        "op-first"
    );
    assert!(saw_late_settle.load(Ordering::SeqCst));
    assert!(runtime.is_turn_dispatch_idle("agent-watchdog"));
    assert_eq!(runtime.in_flight_run_count("agent-watchdog"), 0);

    let _ = fs::remove_dir_all(root);
}


#[test]
fn ack_redrive_send_uses_background_lane_and_source() {
    let root = temp_root("ack-redrive-lane");
    let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();

    let worker_runtime = Arc::clone(&runtime);
    let worker = thread::spawn(move || {
        let args = serde_json::json!({
            "agentId": "agent-redrive",
            "prompt": "[System recovery]",
            "clientNonce": "ack-redrive:agent-redrive:1:123",
            "requestSource": "handoff-resume",
            "ackRedrive": true,
            "appendUserMessage": false,
            "hidden": true,
            "skipAckObligation": true
        });
        worker_runtime.execute_send_with_watchdog(
            &args,
            || {
                entered_tx.send(()).expect("signal redrive dispatch");
                release_rx.recv().expect("release redrive dispatch");
                Ok(serde_json::json!({
                    "accepted": true,
                    "operationId": "op-redrive"
                }))
            },
            |_| Ok(None),
            |_| false,
        )
    });

    entered_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("redrive dispatch entered");
    assert_eq!(
        runtime.active_turn_lane("agent-redrive"),
        Some(RunLane::Background)
    );
    assert_eq!(
        runtime.active_turn_source("agent-redrive").as_deref(),
        Some("ack-redrive")
    );

    release_tx.send(()).expect("release redrive");
    assert_eq!(
        worker.join().expect("redrive worker").expect("redrive result")["operationId"],
        "op-redrive"
    );
    assert!(runtime.is_turn_dispatch_idle("agent-redrive"));
    let _ = fs::remove_dir_all(root);
}
