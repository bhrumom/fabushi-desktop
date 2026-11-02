use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::connectors::channel_delivery::{
    ChannelDeliveryPort, ChannelOutboundMessage, HostChannelDelivery,
};
use mahayana_host_runtime::extensions::session::agent_db::{
    ChannelInboundAppendAttempt, ChannelInboundAppendFailure, ChannelInboundAppendFailureKind,
    ChannelInboundAppendOutcome,
};
use mahayana_host_runtime::extensions::transcript::background_wakes::{
    BackgroundWakes, InboundAppendAction, InboundAppendState, InboundAppendWork,
    CHANNEL_DELIVERY_FAILED_WAKE_CUE,
    build_channel_delivery_failure_wake_prompt, build_channel_outbound_message,
    humanize_channel_delivery_failure, redrivable_inbound_envelopes,
};
use mahayana_host_runtime::extensions::transcript::send_message_shaping::collect_inbound_images;
use serde_json::{Value, json};

#[derive(Default)]
struct RecordingPort {
    deliveries: Mutex<Vec<(String, String, ChannelOutboundMessage)>>,
}

impl ChannelDeliveryPort for RecordingPort {
    fn deliver(
        &self,
        agent_id: &str,
        address_token: &str,
        message: &ChannelOutboundMessage,
    ) -> Result<(), String> {
        self.deliveries.lock().expect("deliveries").push((
            agent_id.to_string(),
            address_token.to_string(),
            message.clone(),
        ));
        Ok(())
    }
}

#[test]
fn channel_outbound_projection_matches_frozen_text_attachment_and_widget_fences() {
    assert_eq!(
        build_channel_outbound_message(&json!({
            "type":"text",
            "content":"caption",
            "deliveryOperationId":"transcript-entry-1",
            "images":[
                {"url":"https://example.test/first.png"},
                {"url":"https://example.test/second.png"}
            ]
        })),
        Some(ChannelOutboundMessage::Attachment {
            url: "https://example.test/first.png".into(),
            caption: Some("caption".into()),
            operation_id: Some("transcript-entry-1".into()),
        })
    );
    assert_eq!(
        build_channel_outbound_message(&json!({
            "type":"attachment",
            "url":"file:///workspace/report.pdf",
            "alt":"report"
        })),
        Some(ChannelOutboundMessage::Attachment {
            url: "file:///workspace/report.pdf".into(),
            caption: Some("report".into()),
            operation_id: None,
        })
    );
    assert_eq!(
        build_channel_outbound_message(&json!({
            "type":"attachment",
            "url":"",
            "alt":"fallback text"
        })),
        Some(ChannelOutboundMessage::Text {
            text: "fallback text".into(),
            operation_id: None,
        })
    );
    assert!(build_channel_outbound_message(&json!({
        "type":"widget",
        "channel":"slack:C1",
        "widget":{"prompt":"Choose","options":[]}
    }))
    .is_none());
}

#[test]
fn host_channel_delivery_defaults_fail_closed_and_allows_one_explicit_provider_port() {
    let delivery = HostChannelDelivery::default();
    assert_eq!(
        delivery
            .deliver(
                "agent-a",
                "slack:C1",
                &ChannelOutboundMessage::Text { text: "hello".into(), operation_id: None },
            )
            .expect_err("unregistered delivery"),
        "No channel delivery mechanism is registered."
    );

    let port = Arc::new(RecordingPort::default());
    delivery.set_port(port.clone());
    delivery
        .deliver(
            "agent-a",
            "slack:C1",
            &ChannelOutboundMessage::Text { text: "hello".into(), operation_id: None },
        )
        .expect("registered delivery");
    let deliveries = port.deliveries.lock().expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].0, "agent-a");
    assert_eq!(deliveries[0].1, "slack:C1");
}

#[test]
fn delivery_failure_humanization_and_hidden_revival_prompt_match_frozen_contract() {
    assert_eq!(
        humanize_channel_delivery_failure(
            "slack:C1",
            "No channel delivery mechanism is registered."
        ),
        "Channel messaging isn't available on this computer, so that message wasn't delivered."
    );
    assert_eq!(
        humanize_channel_delivery_failure("invalid", "not a valid channel address"),
        "\"invalid\" isn't a valid channel address, so that message wasn't delivered."
    );
    let prompt = build_channel_delivery_failure_wake_prompt(&[json!({
        "addressToken":"slack:C1",
        "reason":"Slack isn't connected"
    })]);
    assert!(prompt.starts_with(CHANNEL_DELIVERY_FAILED_WAKE_CUE));
    assert!(prompt.contains("- To slack:C1: Slack isn't connected"));
    assert!(prompt.contains("SendMessage with no channel target"));
    assert!(prompt.contains("Don't silently retry the same channel"));
}

#[test]
fn channel_failure_queue_is_agent_scoped_and_revival_fenced() {
    let mut wakes = BackgroundWakes::<serde_json::Value>::default();
    BackgroundWakes::enqueue(
        &mut wakes.pending_channel_failures,
        "agent-a",
        json!({"addressToken":"slack:C1","reason":"failed"}),
    );
    assert!(BackgroundWakes::<serde_json::Value>::begin_revival(
        &mut wakes.reviving_channel_failure_agent_ids,
        "agent-a",
    ));
    assert!(!BackgroundWakes::<serde_json::Value>::begin_revival(
        &mut wakes.reviving_channel_failure_agent_ids,
        "agent-a",
    ));
    let failures =
        BackgroundWakes::take_pending(&mut wakes.pending_channel_failures, "agent-a");
    assert_eq!(failures.len(), 1);
    BackgroundWakes::<serde_json::Value>::end_revival(
        &mut wakes.reviving_channel_failure_agent_ids,
        "agent-a",
    );
}

#[test]
fn inbound_idle_handoff_enqueue_wins_retains_current_worker_and_claims_batch_once() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<serde_json::Value>::default()));
    let mut ingress = wakes.lock().expect("background wakes");
    assert!(BackgroundWakes::<serde_json::Value>::begin_revival(
        &mut ingress.reviving_inbound_agent_ids,
        "agent-a",
    ));

    let (claiming_tx, claiming_rx) = mpsc::channel();
    let worker_wakes = Arc::clone(&wakes);
    let worker = thread::spawn(move || {
        claiming_tx.send(()).expect("announce batch claim");
        let mut state = worker_wakes.lock().expect("worker background wakes");
        state.take_pending_inbound_or_end_revival("agent-a")
    });

    // Ingress owns the mutex before the current worker attempts its idle check.
    claiming_rx.recv().expect("worker ready to claim");
    let envelopes = vec![json!({"id":"first"}), json!({"id":"second"})];
    for envelope in &envelopes {
        BackgroundWakes::enqueue(&mut ingress.pending_inbound, "agent-a", envelope.clone());
        assert!(!BackgroundWakes::<serde_json::Value>::begin_revival(
            &mut ingress.reviving_inbound_agent_ids,
            "agent-a",
        ));
    }
    drop(ingress);

    assert_eq!(worker.join().expect("current worker"), Some(envelopes));
    let mut state = wakes.lock().expect("background wakes after claim");
    assert!(state.reviving_inbound_agent_ids.contains("agent-a"));
    assert!(!state.pending_inbound.contains_key("agent-a"));
    assert_eq!(state.take_pending_inbound_or_end_revival("agent-a"), None);
    assert!(!state.reviving_inbound_agent_ids.contains("agent-a"));
}

#[test]
fn inbound_idle_handoff_idle_wins_admits_one_successor_and_claims_batch_once() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<serde_json::Value>::default()));
    let mut retiring_worker = wakes.lock().expect("background wakes");
    assert!(BackgroundWakes::<serde_json::Value>::begin_revival(
        &mut retiring_worker.reviving_inbound_agent_ids,
        "agent-a",
    ));

    let (enqueuing_tx, enqueuing_rx) = mpsc::channel();
    let ingress_wakes = Arc::clone(&wakes);
    let ingress = thread::spawn(move || {
        enqueuing_tx.send(()).expect("announce inbound enqueue");
        let mut state = ingress_wakes.lock().expect("ingress background wakes");
        BackgroundWakes::enqueue(&mut state.pending_inbound, "agent-a", json!({"id":"first"}));
        let first_starts_worker = BackgroundWakes::<serde_json::Value>::begin_revival(
            &mut state.reviving_inbound_agent_ids,
            "agent-a",
        );
        BackgroundWakes::enqueue(&mut state.pending_inbound, "agent-a", json!({"id":"second"}));
        let second_starts_worker = BackgroundWakes::<serde_json::Value>::begin_revival(
            &mut state.reviving_inbound_agent_ids,
            "agent-a",
        );
        (first_starts_worker, second_starts_worker)
    });

    // The current worker releases ownership before the waiting ingress can lock.
    enqueuing_rx.recv().expect("ingress ready to enqueue");
    assert_eq!(
        retiring_worker.take_pending_inbound_or_end_revival("agent-a"),
        None,
    );
    assert!(!retiring_worker.reviving_inbound_agent_ids.contains("agent-a"));
    drop(retiring_worker);

    assert_eq!(ingress.join().expect("inbound ingress"), (true, false));
    let mut successor = wakes.lock().expect("successor background wakes");
    assert!(successor.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(
        successor.take_pending_inbound_or_end_revival("agent-a"),
        Some(vec![json!({"id":"first"}), json!({"id":"second"})]),
    );
    assert!(successor.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(successor.take_pending_inbound_or_end_revival("agent-a"), None);
    assert!(!successor.reviving_inbound_agent_ids.contains("agent-a"));
}

#[test]
fn inbound_idle_handoff_treats_stored_empty_batch_as_idle() {
    let mut wakes = BackgroundWakes::<serde_json::Value>::default();
    wakes.pending_inbound.insert("agent-a".into(), Vec::new());
    assert!(BackgroundWakes::<serde_json::Value>::begin_revival(
        &mut wakes.reviving_inbound_agent_ids,
        "agent-a",
    ));

    assert_eq!(wakes.take_pending_inbound_or_end_revival("agent-a"), None);
    assert!(!wakes.pending_inbound.contains_key("agent-a"));
    assert!(!wakes.reviving_inbound_agent_ids.contains("agent-a"));
    BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", json!({"id":"next"}));
    assert!(BackgroundWakes::<serde_json::Value>::begin_revival(
        &mut wakes.reviving_inbound_agent_ids,
        "agent-a",
    ));
    assert_eq!(
        wakes.take_pending_inbound_or_end_revival("agent-a"),
        Some(vec![json!({"id":"next"})]),
    );
}

#[test]
fn inbound_idle_handoff_preserves_other_agents_and_wake_families() {
    let mut wakes = BackgroundWakes::<serde_json::Value>::default();
    for agent_id in ["agent-a", "agent-b"] {
        BackgroundWakes::enqueue(
            &mut wakes.pending_inbound,
            agent_id,
            json!({"inbound":agent_id}),
        );
        BackgroundWakes::<serde_json::Value>::begin_revival(
            &mut wakes.reviving_inbound_agent_ids,
            agent_id,
        );
        BackgroundWakes::enqueue(
            &mut wakes.pending_channel_failures,
            agent_id,
            json!({"failure":agent_id}),
        );
        BackgroundWakes::<serde_json::Value>::begin_revival(
            &mut wakes.reviving_channel_failure_agent_ids,
            agent_id,
        );
        BackgroundWakes::enqueue(
            &mut wakes.pending_event_wakes,
            agent_id,
            json!({"event":agent_id}),
        );
        BackgroundWakes::<serde_json::Value>::begin_revival(
            &mut wakes.reviving_event_agent_ids,
            agent_id,
        );
        wakes.dm_preempted_wake_agent_ids.insert(agent_id.into());
    }
    let unrelated_state = |state: &BackgroundWakes<serde_json::Value>| {
        (
            state.pending_inbound.get("agent-b").cloned(),
            state.reviving_inbound_agent_ids.contains("agent-b"),
            state.pending_channel_failures.clone(),
            state.reviving_channel_failure_agent_ids.clone(),
            state.pending_event_wakes.clone(),
            state.reviving_event_agent_ids.clone(),
            state.dm_preempted_wake_agent_ids.clone(),
        )
    };
    let expected_unrelated = unrelated_state(&wakes);

    assert_eq!(
        wakes.take_pending_inbound_or_end_revival("agent-a"),
        Some(vec![json!({"inbound":"agent-a"})]),
    );
    assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(unrelated_state(&wakes), expected_unrelated);

    assert_eq!(wakes.take_pending_inbound_or_end_revival("agent-a"), None);
    assert!(!wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(unrelated_state(&wakes), expected_unrelated);
}

// These outcomes are inputs to the real queue boundary. Session/SQLite and the
// production append-to-Runner handoff have separate real-owner contract tests.
fn inbound_append_envelopes_fixture() -> Vec<Value> {
    vec![
        json!({
            "address":{"platform":"slack","chat":"C1"},
            "sender":"alice","text":"first original message","timestampMs":1000,
            "images":[{"data":"aGVsbG8=","mimeType":"image/png"}],
            "widgetContext":{"selectedOption":"first"}
        }),
        json!({
            "address":"discord:C2","sender":"bob",
            "text":"second original message","timestampMs":1001
        }),
    ]
}

fn inbound_append_attempt_fixture() -> ChannelInboundAppendAttempt {
    ChannelInboundAppendAttempt {
        session_key: "session-key-a".into(),
        session_id: "session-id-a".into(),
        lifecycle_revision: "original-lifecycle-a".into(),
        entries: Some(vec![
            json!({
                "kind":"message","id":"original-user-41","role":"user",
                "content":"first original message","timestampMs":1000,
                "channel":"slack:C1","channelSender":"alice",
                "sessionKey":"session-key-a"
            }),
            json!({
                "kind":"message","id":"original-user-42","role":"user",
                "content":"second original message","timestampMs":1001,
                "channel":"discord:C2","channelSender":"bob",
                "sessionKey":"session-key-a"
            }),
        ]),
    }
}

fn inbound_append_failure_fixture(
    kind: ChannelInboundAppendFailureKind,
) -> ChannelInboundAppendFailure {
    ChannelInboundAppendFailure {
        kind,
        detail: "queue-boundary outcome fixture".into(),
    }
}

fn one_inbound_readiness_ticket(wakes: &mut BackgroundWakes<Value>, agent_id: &str) -> InboundAppendWork<Value> {
    let mut tickets = wakes.request_inbound_readiness();
    assert_eq!(tickets.len(), 1, "one retained head owns one launch ticket");
    let (claimed_agent, work) = tickets.pop().unwrap();
    assert_eq!(claimed_agent, agent_id);
    assert!(wakes.owns_inbound_append(agent_id, &work));
    work
}

fn inbound_retry(action: InboundAppendAction<Value>) -> InboundAppendWork<Value> {
    match action {
        InboundAppendAction::Retry(work) => work,
        other => panic!("expected a retained-head Retry, got {other:?}"),
    }
}

#[test]
fn inbound_readiness_on_either_side_of_pause_reacquires_the_same_head_once() {
    for signal_before_pause in [true, false] {
        let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
        let envelope = json!({"text":"paused without new inbound input"});
        let original = wakes.lock().unwrap().enqueue_inbound_and_claim("agent-a", envelope.clone())
            .unwrap().expect("ingress reserves the actual head before launch");
        let (pause_tx, pause_rx) = mpsc::channel();
        let worker_wakes = Arc::clone(&wakes);
        let worker_ticket = original.clone();
        let worker = thread::spawn(move || {
            pause_rx.recv_timeout(Duration::from_secs(5)).expect("release exact worker to pause");
            worker_wakes.lock().unwrap().pause_inbound_append("agent-a", &worker_ticket)
        });
        if signal_before_pause {
            let mut state = wakes.lock().unwrap();
            for _ in 0..3 {
                assert!(state.request_inbound_readiness().is_empty(), "active signals coalesce without another launch");
            }
            assert!(state.owns_inbound_append("agent-a", &original));
        }
        pause_tx.send(()).unwrap();
        let action = worker.join().expect("bounded pause worker");
        let mut state = wakes.lock().unwrap();
        let retry = if signal_before_pause {
            inbound_retry(action)
        } else {
            assert_eq!(action, InboundAppendAction::Held);
            assert!(!state.reviving_inbound_agent_ids.contains("agent-a"));
            assert_eq!(state.take_inbound_append_or_end_revival("agent-a"), None);
            one_inbound_readiness_ticket(&mut state, "agent-a")
        };
        assert_eq!(retry.claim_id, original.claim_id);
        assert_ne!(retry.worker_token, original.worker_token);
        assert_eq!(retry.envelopes, vec![envelope]);
        assert_eq!(retry.attempt, None);
        assert!(!state.owns_inbound_append("agent-a", &original));
        assert!(state.owns_inbound_append("agent-a", &retry));
        assert_eq!(state.pause_inbound_append("agent-a", &original), InboundAppendAction::Stale);
        // No new signal arrived after acquisition: even three coalesced hints
        // permit only one recheck, never a spin while the gate stays closed.
        assert_eq!(state.pause_inbound_append("agent-a", &retry), InboundAppendAction::Held);
        assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::NotWritten));
        assert!(!state.reviving_inbound_agent_ids.contains("agent-a"));
        assert_eq!(state.take_inbound_append_or_end_revival("agent-a"), None);
        assert!(!state.pending_inbound.contains_key("agent-a"));
    }
}

#[test]
fn inbound_readiness_rollback_retry_uses_the_latest_binding_and_prepared_entries() {
    let mut wakes = BackgroundWakes::<Value>::default();
    let envelopes = inbound_append_envelopes_fixture();
    for envelope in &envelopes {
        BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", envelope.clone());
    }
    let original = one_inbound_readiness_ticket(&mut wakes, "agent-a");
    assert_eq!(original.attempt, None);
    assert!(wakes.request_inbound_readiness().is_empty());
    let mut binding = inbound_append_attempt_fixture();
    binding.entries = None;
    let rollback = ChannelInboundAppendOutcome::NotWritten(inbound_append_failure_fixture(
        ChannelInboundAppendFailureKind::Storage,
    ));
    let preparing = inbound_retry(wakes.record_inbound_append_outcome(
        "agent-a", &original, Some(binding.clone()), &rollback,
    ));
    assert_eq!(preparing.attempt, Some(binding));
    assert_eq!(preparing.claim_id, original.claim_id);
    assert_ne!(preparing.worker_token, original.worker_token);
    let tail = json!({"text":"new input signals readiness but stays behind preparation"});
    assert_eq!(wakes.enqueue_inbound_and_claim("agent-a", tail.clone()).unwrap(), None);
    assert!(wakes.request_inbound_readiness().is_empty(), "ingress and Settings hints share one latch");
    let prepared = inbound_append_attempt_fixture();
    let retry = inbound_retry(wakes.record_inbound_append_outcome(
        "agent-a", &preparing, Some(prepared.clone()), &rollback,
    ));
    assert_eq!(retry.attempt, Some(prepared.clone()), "use the just-recorded preparation, not the worker's older snapshot");
    assert_eq!(retry.envelopes, envelopes);
    assert_eq!(retry.claim_id, original.claim_id);
    assert_ne!(retry.worker_token, preparing.worker_token);
    assert_eq!(wakes.inbound_append_attempt("agent-a"), Some(&prepared));
    assert_eq!(wakes.pending_inbound.get("agent-a"), Some(&vec![tail]));
    assert_eq!(wakes.record_inbound_append_outcome("agent-a", &preparing, None, &rollback), InboundAppendAction::Stale);
    assert_eq!(wakes.pause_inbound_append("agent-a", &retry), InboundAppendAction::Held);
    assert_eq!(wakes.inbound_append_attempt("agent-a"), Some(&prepared));
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
}

#[test]
fn inbound_launch_failure_preserves_preparation_and_delayed_release_cannot_clear_successor() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
    let prepared = inbound_append_attempt_fixture();
    let failed_launch = {
        let mut state = wakes.lock().unwrap();
        let original = state.enqueue_inbound_and_claim("agent-a", json!({"text":"retained head"}))
            .unwrap().unwrap();
        assert_eq!(state.record_inbound_append_outcome(
            "agent-a", &original, Some(prepared.clone()),
            &ChannelInboundAppendOutcome::NotWritten(inbound_append_failure_fixture(ChannelInboundAppendFailureKind::Storage)),
        ), InboundAppendAction::Held);
        let failed = one_inbound_readiness_ticket(&mut state, "agent-a");
        assert!(state.release_inbound_launch("agent-a", &failed));
        assert!(!state.release_inbound_launch("agent-a", &failed));
        assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::NotWritten));
        assert_eq!(state.inbound_append_attempt("agent-a"), Some(&prepared));
        assert!(!state.reviving_inbound_agent_ids.contains("agent-a"));
        assert_eq!(state.take_inbound_append_or_end_revival("agent-a"), None);
        failed
    };
    let (release_tx, release_rx) = mpsc::channel();
    let delayed_wakes = Arc::clone(&wakes);
    let delayed_ticket = failed_launch.clone();
    let delayed_error = thread::spawn(move || {
        release_rx.recv_timeout(Duration::from_secs(5)).expect("successor before delayed launch error");
        let mut state = delayed_wakes.lock().unwrap();
        (
            state.release_inbound_launch("agent-a", &delayed_ticket),
            state.pause_inbound_append("agent-a", &delayed_ticket),
            state.record_inbound_append_outcome("agent-a", &delayed_ticket, None,
                &ChannelInboundAppendOutcome::Unknown(inbound_append_failure_fixture(ChannelInboundAppendFailureKind::Storage))),
        )
    });
    let successor = one_inbound_readiness_ticket(&mut wakes.lock().unwrap(), "agent-a");
    assert_eq!(successor.claim_id, failed_launch.claim_id);
    assert_ne!(successor.worker_token, failed_launch.worker_token);
    assert_eq!(successor.envelopes, failed_launch.envelopes);
    assert_eq!(successor.attempt, Some(prepared.clone()));
    release_tx.send(()).unwrap();
    assert_eq!(delayed_error.join().unwrap(), (false, InboundAppendAction::Stale, InboundAppendAction::Stale));
    let mut state = wakes.lock().unwrap();
    assert!(!state.release_inbound_launch("agent-b", &successor));
    let mut wrong_claim = successor.clone();
    wrong_claim.claim_id.push_str("-foreign");
    assert!(!state.release_inbound_launch("agent-a", &wrong_claim));
    assert!(state.owns_inbound_append("agent-a", &successor));
    assert!(state.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(state.inbound_append_attempt("agent-a"), Some(&prepared));
}

#[test]
fn inbound_unknown_discards_active_readiness_and_blocks_signals_and_new_tail_only_for_its_agent() {
    let mut wakes = BackgroundWakes::<Value>::default();
    let unknown = wakes.enqueue_inbound_and_claim("agent-a", json!({"text":"possibly committed"})).unwrap().unwrap();
    assert!(wakes.request_inbound_readiness().is_empty());
    let prepared = inbound_append_attempt_fixture();
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &unknown, Some(prepared.clone()),
        &ChannelInboundAppendOutcome::Unknown(inbound_append_failure_fixture(ChannelInboundAppendFailureKind::RollbackUnconfirmed)),
    ), InboundAppendAction::Held);
    let tail = vec![json!({"text":"later first"}), json!({"text":"later second"})];
    for envelope in &tail {
        assert_eq!(wakes.enqueue_inbound_and_claim("agent-a", envelope.clone()).unwrap(), None);
        assert!(wakes.request_inbound_readiness().is_empty());
    }
    BackgroundWakes::enqueue(&mut wakes.pending_event_wakes, "agent-a", json!({"event":"unchanged"}));
    wakes.reviving_event_agent_ids.insert("agent-a".into());
    BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-b", json!({"text":"independent"}));
    let other = one_inbound_readiness_ticket(&mut wakes, "agent-b");
    assert!(wakes.owns_inbound_append("agent-b", &other));
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::Unknown));
    assert_eq!(wakes.inbound_append_attempt("agent-a"), Some(&prepared));
    assert_eq!(wakes.pending_inbound.get("agent-a"), Some(&tail));
    assert!(!wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert!(!wakes.release_inbound_launch("agent-a", &unknown));
    assert_eq!(wakes.pause_inbound_append("agent-a", &unknown), InboundAppendAction::Stale);
    assert_eq!(wakes.pending_event_wakes["agent-a"], vec![json!({"event":"unchanged"})]);
    assert!(wakes.reviving_event_agent_ids.contains("agent-a"));
}

#[test]
fn inbound_readiness_cannot_steal_committed_drain_or_generation_bound_displayed_redrive() {
    let mut wakes = BackgroundWakes::<Value>::default();
    let original = wakes.enqueue_inbound_and_claim("agent-a", inbound_append_envelopes_fixture()[0].clone())
        .unwrap().unwrap();
    assert!(wakes.request_inbound_readiness().is_empty());
    let prepared = inbound_append_attempt_fixture();
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &original, Some(prepared.clone()),
        &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
    ), InboundAppendAction::Committed);
    let tail = json!({"text":"tail after the displayed continuation"});
    assert_eq!(wakes.enqueue_inbound_and_claim("agent-a", tail.clone()).unwrap(), None);
    assert!(wakes.request_inbound_readiness().is_empty());
    let displayed = redrivable_inbound_envelopes(&original.envelopes);
    assert!(wakes.finish_inbound_append_with_redrive("agent-a", &original, Some(displayed.clone())));
    assert!(wakes.request_inbound_readiness().is_empty(), "current drain owns the finish-to-take gap");
    let redrive = wakes.take_inbound_append_or_end_revival("agent-a").unwrap();
    let mut bound_empty = prepared;
    bound_empty.entries = Some(Vec::new());
    assert_eq!(redrive.envelopes, displayed);
    assert_eq!(redrive.attempt, Some(bound_empty.clone()));
    assert_ne!(redrive.claim_id, original.claim_id);
    assert_eq!(wakes.pause_inbound_append("agent-a", &redrive), InboundAppendAction::Held,
        "readiness on the old committed head cannot become a latch on its continuation");
    let resumed = one_inbound_readiness_ticket(&mut wakes, "agent-a");
    assert_eq!(resumed.claim_id, redrive.claim_id);
    assert_ne!(resumed.worker_token, redrive.worker_token);
    assert_eq!(resumed.attempt, Some(bound_empty));
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &resumed, resumed.attempt.clone(),
        &ChannelInboundAppendOutcome::Committed { inserted_count: 0 },
    ), InboundAppendAction::Committed);
    assert!(wakes.finish_inbound_append("agent-a", &resumed));
    assert!(wakes.request_inbound_readiness().is_empty(), "raw tail still belongs to this drain");
    let next = wakes.take_inbound_append_or_end_revival("agent-a").unwrap();
    assert_eq!(next.envelopes, vec![tail]);
    assert_eq!(next.attempt, None);
    assert_ne!(next.claim_id, resumed.claim_id);
}

#[test]
fn inbound_close_permanently_revokes_all_states_and_late_tickets_without_reopening() {
    let mut wakes = BackgroundWakes::<Value>::default();
    let mut tickets = Vec::new();
    for agent in ["active", "paused", "unknown", "committed"] {
        let work = wakes.enqueue_inbound_and_claim(agent, json!({"agent":agent})).unwrap().unwrap();
        match agent {
            "paused" => assert_eq!(wakes.pause_inbound_append(agent, &work), InboundAppendAction::Held),
            "unknown" => assert_eq!(wakes.record_inbound_append_outcome(agent, &work, Some(inbound_append_attempt_fixture()),
                &ChannelInboundAppendOutcome::Unknown(inbound_append_failure_fixture(ChannelInboundAppendFailureKind::Storage))), InboundAppendAction::Held),
            "committed" => assert_eq!(wakes.record_inbound_append_outcome(agent, &work, Some(inbound_append_attempt_fixture()),
                &ChannelInboundAppendOutcome::Committed { inserted_count: 2 }), InboundAppendAction::Committed),
            _ => {}
        }
        BackgroundWakes::enqueue(&mut wakes.pending_inbound, agent, json!({"tail":agent}));
        tickets.push((agent, work));
    }
    BackgroundWakes::enqueue(&mut wakes.pending_channel_failures, "active", json!({"failure":true}));
    BackgroundWakes::enqueue(&mut wakes.pending_event_wakes, "active", json!({"event":true}));
    wakes.reviving_channel_failure_agent_ids.insert("active".into());
    wakes.reviving_event_agent_ids.insert("active".into());
    wakes.dm_preempted_wake_agent_ids.insert("active".into());
    wakes.close();
    assert!(wakes.is_closed());
    assert!(wakes.request_inbound_readiness().is_empty());
    for (agent, work) in &tickets {
        assert_eq!(wakes.inbound_append_state(agent), None);
        assert_eq!(wakes.inbound_append_attempt(agent), None);
        assert!(!wakes.owns_inbound_append(agent, work));
        assert!(!wakes.release_inbound_launch(agent, work));
        assert_eq!(wakes.pause_inbound_append(agent, work), InboundAppendAction::Stale);
        assert_eq!(wakes.record_inbound_append_outcome(agent, work, Some(inbound_append_attempt_fixture()),
            &ChannelInboundAppendOutcome::Committed { inserted_count: 2 }), InboundAppendAction::Stale);
        assert!(!wakes.finish_inbound_append(agent, work));
        assert!(!wakes.finish_inbound_append_with_redrive(agent, work, Some(work.envelopes.clone())));
        assert_eq!(wakes.take_inbound_append_or_end_revival(agent), None);
        assert_eq!(wakes.take_pending_inbound_or_end_revival(agent), None);
        assert!(wakes.enqueue_inbound_and_claim(agent, json!({"late":true})).is_err());
    }
    wakes.close();
    assert!(wakes.is_closed());
    assert!(wakes.pending_inbound.is_empty());
    assert!(wakes.reviving_inbound_agent_ids.is_empty());
    assert!(wakes.pending_channel_failures.is_empty());
    assert!(wakes.reviving_channel_failure_agent_ids.is_empty());
    assert!(wakes.pending_event_wakes.is_empty());
    assert!(wakes.reviving_event_agent_ids.is_empty());
    assert!(wakes.dm_preempted_wake_agent_ids.is_empty());
}

#[test]
fn inbound_append_enqueue_before_idle_retains_current_worker_and_claims_ordered_head_once() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
    let mut ingress = wakes.lock().expect("ingress owns queue mutex");
    assert!(BackgroundWakes::<Value>::begin_revival(
        &mut ingress.reviving_inbound_agent_ids,
        "agent-a",
    ));
    let (claiming_tx, claiming_rx) = mpsc::channel();
    let worker_wakes = Arc::clone(&wakes);
    let worker = thread::spawn(move || {
        claiming_tx.send(()).expect("announce retained-head claim");
        worker_wakes
            .lock()
            .expect("worker queue mutex")
            .take_inbound_append_or_end_revival("agent-a")
    });

    claiming_rx.recv().expect("worker ready while ingress holds mutex");
    let envelopes = inbound_append_envelopes_fixture();
    for envelope in &envelopes {
        BackgroundWakes::enqueue(&mut ingress.pending_inbound, "agent-a", envelope.clone());
        assert!(!BackgroundWakes::<Value>::begin_revival(
            &mut ingress.reviving_inbound_agent_ids,
            "agent-a",
        ));
    }
    drop(ingress);

    let work = worker.join().expect("current worker").expect("ordered head");
    assert_eq!(work.envelopes, envelopes);
    assert!(!work.claim_id.is_empty());
    assert!(!work.worker_token.is_empty());
    assert_eq!(work.attempt, None);
    let mut state = wakes.lock().expect("queue after claim");
    assert!(state.owns_inbound_append("agent-a", &work));
    assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::Active));
    assert!(!state.pending_inbound.contains_key("agent-a"));
    assert_eq!(state.take_inbound_append_or_end_revival("agent-a"), None);
    assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::Active));
    assert!(state.reviving_inbound_agent_ids.contains("agent-a"));
    assert!(state.owns_inbound_append("agent-a", &work));
}

#[test]
fn inbound_append_idle_before_enqueue_admits_one_successor_with_ordered_head() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
    let mut retiring = wakes.lock().expect("retiring worker owns queue mutex");
    assert!(BackgroundWakes::<Value>::begin_revival(
        &mut retiring.reviving_inbound_agent_ids,
        "agent-a",
    ));
    let (enqueuing_tx, enqueuing_rx) = mpsc::channel();
    let ingress_wakes = Arc::clone(&wakes);
    let envelopes = inbound_append_envelopes_fixture();
    let incoming = envelopes.clone();
    let ingress = thread::spawn(move || {
        enqueuing_tx.send(()).expect("announce ingress");
        let mut state = ingress_wakes.lock().expect("ingress queue mutex");
        incoming
            .into_iter()
            .map(|envelope| {
                BackgroundWakes::enqueue(&mut state.pending_inbound, "agent-a", envelope);
                BackgroundWakes::<Value>::begin_revival(
                    &mut state.reviving_inbound_agent_ids,
                    "agent-a",
                )
            })
            .collect::<Vec<_>>()
    });

    enqueuing_rx.recv().expect("ingress ready while retiring worker holds mutex");
    assert_eq!(retiring.take_inbound_append_or_end_revival("agent-a"), None);
    assert!(!retiring.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(retiring.inbound_append_state("agent-a"), None);
    drop(retiring);

    assert_eq!(ingress.join().expect("ingress"), vec![true, false]);
    let mut successor = wakes.lock().expect("successor queue mutex");
    let work = successor.take_inbound_append_or_end_revival("agent-a").expect("successor head");
    assert_eq!(work.envelopes, envelopes);
    assert_eq!(work.attempt, None);
    assert!(successor.owns_inbound_append("agent-a", &work));
    assert!(!successor.pending_inbound.contains_key("agent-a"));
    assert_eq!(successor.take_inbound_append_or_end_revival("agent-a"), None);
    assert!(successor.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(successor.inbound_append_state("agent-a"), Some(InboundAppendState::Active));
}

#[test]
fn inbound_append_requires_revival_and_releases_stored_empty_input_without_a_claim() {
    let mut wakes = BackgroundWakes::<Value>::default();
    let envelope = json!({"text":"waiting for an existing wake"});
    BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", envelope.clone());
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
    assert_eq!(wakes.pending_inbound.get("agent-a"), Some(&vec![envelope]));
    assert_eq!(wakes.inbound_append_state("agent-a"), None);

    wakes.pending_inbound.insert("agent-b".into(), Vec::new());
    assert!(BackgroundWakes::<Value>::begin_revival(
        &mut wakes.reviving_inbound_agent_ids,
        "agent-b",
    ));
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-b"), None);
    assert!(!wakes.pending_inbound.contains_key("agent-b"));
    assert!(!wakes.reviving_inbound_agent_ids.contains("agent-b"));
    assert_eq!(wakes.inbound_append_state("agent-b"), None);
    assert!(wakes.pending_inbound.contains_key("agent-a"));
}

#[test]
fn inbound_append_not_written_retry_keeps_original_preparation_ahead_of_tail_and_fences_old_worker() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
    let envelopes = inbound_append_envelopes_fixture();
    let original_attempt = inbound_append_attempt_fixture();
    let original = {
        let mut state = wakes.lock().expect("initial queue");
        for envelope in &envelopes {
            BackgroundWakes::enqueue(&mut state.pending_inbound, "agent-a", envelope.clone());
        }
        assert!(BackgroundWakes::<Value>::begin_revival(&mut state.reviving_inbound_agent_ids, "agent-a"));
        state.take_inbound_append_or_end_revival("agent-a").expect("original head")
    };
    let (paused_tx, paused_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let old_wakes = Arc::clone(&wakes);
    let old_work = original.clone();
    let prepared = original_attempt.clone();
    let old_worker = thread::spawn(move || {
        {
            let mut state = old_wakes.lock().expect("old worker queue");
            assert_eq!(state.record_inbound_append_outcome(
                "agent-a",
                &old_work,
                Some(prepared.clone()),
                &ChannelInboundAppendOutcome::NotWritten(inbound_append_failure_fixture(
                    ChannelInboundAppendFailureKind::Storage,
                )),
            ), InboundAppendAction::Held);
        }
        paused_tx.send(()).expect("head retained and revival released");
        resume_rx.recv().expect("successor acquired before old callback");
        let mut replaced_attempt = prepared;
        replaced_attempt.session_id = "replacement-session-must-not-leak".into();
        replaced_attempt.lifecycle_revision = "replacement-lifecycle-must-not-leak".into();
        replaced_attempt.entries = Some(vec![json!({"id":"reminted-id","content":"changed"})]);
        let mut state = old_wakes.lock().expect("late old worker queue");
        (
            state.pause_inbound_append("agent-a", &old_work),
            state.record_inbound_append_outcome(
                "agent-a",
                &old_work,
                Some(replaced_attempt),
                &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
            ),
            state.finish_inbound_append("agent-a", &old_work),
        )
    });

    paused_rx.recv().expect("old worker paused without a retry loop");
    let tail = vec![json!({"text":"later first"}), json!({"text":"later second"})];
    let retry = {
        let mut state = wakes.lock().expect("later ingress queue");
        assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::NotWritten));
        assert!(!state.owns_inbound_append("agent-a", &original));
        assert_eq!(state.inbound_append_attempt("agent-a"), Some(&original_attempt));
        assert!(!state.pending_inbound.contains_key("agent-a"));
        assert_eq!(state.take_inbound_append_or_end_revival("agent-a"), None);
        for (index, envelope) in tail.iter().enumerate() {
            BackgroundWakes::enqueue(&mut state.pending_inbound, "agent-a", envelope.clone());
            assert_eq!(BackgroundWakes::<Value>::begin_revival(&mut state.reviving_inbound_agent_ids, "agent-a"), index == 0);
        }
        let retry = state.take_inbound_append_or_end_revival("agent-a").expect("same head retried");
        assert_eq!(retry.claim_id, original.claim_id);
        assert_ne!(retry.worker_token, original.worker_token);
        assert_eq!(retry.envelopes, envelopes);
        assert_eq!(retry.attempt, Some(original_attempt.clone()));
        assert!(state.owns_inbound_append("agent-a", &retry));
        assert!(!state.owns_inbound_append("agent-a", &original));
        assert_eq!(state.pending_inbound.get("agent-a"), Some(&tail));
        retry
    };
    resume_tx.send(()).expect("release delayed old callback");
    assert_eq!(old_worker.join().expect("old worker"), (
        InboundAppendAction::Stale, InboundAppendAction::Stale, false,
    ));

    let mut state = wakes.lock().expect("queue after stale callback");
    assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::Active));
    assert_eq!(state.inbound_append_attempt("agent-a"), Some(&original_attempt));
    assert!(state.reviving_inbound_agent_ids.contains("agent-a"));
    assert!(state.owns_inbound_append("agent-a", &retry));
    assert!(!state.owns_inbound_append("agent-a", &original));
    assert_eq!(state.pending_inbound.get("agent-a"), Some(&tail));
    assert_eq!(state.record_inbound_append_outcome(
        "agent-a", &retry, retry.attempt.clone(),
        &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
    ), InboundAppendAction::Committed);
    assert!(!state.finish_inbound_append("agent-a", &original));
    assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::Committed));
    assert!(state.reviving_inbound_agent_ids.contains("agent-a"));
    assert!(state.finish_inbound_append("agent-a", &retry));
    let next = state.take_inbound_append_or_end_revival("agent-a").expect("tail after original head finishes");
    assert_eq!(next.envelopes, tail);
    assert_eq!(next.attempt, None);
    assert_ne!(next.claim_id, retry.claim_id);
    assert!(!state.pending_inbound.contains_key("agent-a"));
}

#[test]
fn inbound_append_pause_and_binding_without_entries_survive_later_wake() {
    let mut wakes = BackgroundWakes::<Value>::default();
    let envelope = json!({"text":"keep through provider or checked-read pause"});
    BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", envelope.clone());
    assert!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"));
    let first = wakes.take_inbound_append_or_end_revival("agent-a").expect("unprepared head");
    assert_eq!(wakes.pause_inbound_append("agent-a", &first), InboundAppendAction::Held);
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::NotWritten));
    assert_eq!(wakes.inbound_append_attempt("agent-a"), None);
    assert!(!wakes.owns_inbound_append("agent-a", &first));
    assert!(!wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert!(!wakes.pending_inbound.contains_key("agent-a"));

    assert!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"));
    let preparing = wakes.take_inbound_append_or_end_revival("agent-a").expect("existing wake resumes head");
    assert_eq!(preparing.claim_id, first.claim_id);
    assert_ne!(preparing.worker_token, first.worker_token);
    assert_eq!(preparing.envelopes, vec![envelope.clone()]);
    assert_eq!(preparing.attempt, None);
    let mut bound = inbound_append_attempt_fixture();
    bound.entries = None;
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &preparing, Some(bound.clone()),
        &ChannelInboundAppendOutcome::NotWritten(inbound_append_failure_fixture(
            ChannelInboundAppendFailureKind::Storage,
        )),
    ), InboundAppendAction::Held);
    let tail = json!({"text":"new history cannot replace the captured lifecycle"});
    BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", tail.clone());
    assert!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"));
    let retry = wakes.take_inbound_append_or_end_revival("agent-a").expect("bound head precedes tail");
    assert_eq!(retry.claim_id, first.claim_id);
    assert_ne!(retry.worker_token, preparing.worker_token);
    assert_eq!(retry.envelopes, vec![envelope]);
    assert_eq!(retry.attempt, Some(bound.clone()));
    assert_eq!(wakes.inbound_append_attempt("agent-a"), Some(&bound));
    assert!(wakes.owns_inbound_append("agent-a", &retry));
    assert!(!wakes.owns_inbound_append("agent-a", &preparing));
    assert_eq!(wakes.pending_inbound.get("agent-a"), Some(&vec![tail]));
    assert_eq!(wakes.pause_inbound_append("agent-a", &first), InboundAppendAction::Stale);
    assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
}

#[test]
fn inbound_append_unknown_is_sticky_with_no_pending_input_later_arrivals_and_late_commit_ack() {
    let mut wakes = BackgroundWakes::<Value>::default();
    for envelope in inbound_append_envelopes_fixture() {
        BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", envelope);
    }
    assert!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"));
    let work = wakes.take_inbound_append_or_end_revival("agent-a").expect("head before unknown");
    let attempt = inbound_append_attempt_fixture();
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &work, Some(attempt.clone()),
        &ChannelInboundAppendOutcome::Unknown(inbound_append_failure_fixture(
            ChannelInboundAppendFailureKind::RollbackUnconfirmed,
        )),
    ), InboundAppendAction::Held);
    assert!(!wakes.pending_inbound.contains_key("agent-a"));
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::Unknown));
    assert_eq!(wakes.inbound_append_attempt("agent-a"), Some(&attempt));
    assert!(!wakes.owns_inbound_append("agent-a", &work));
    assert!(!wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
    assert!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"));
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::Unknown));
    assert!(!wakes.reviving_inbound_agent_ids.contains("agent-a"));

    let tail = vec![json!({"text":"later one"}), json!({"text":"later two"})];
    for (index, envelope) in tail.iter().enumerate() {
        BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", envelope.clone());
        assert_eq!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"), index == 0);
    }
    let mut late_attempt = attempt.clone();
    late_attempt.lifecycle_revision = "late-binding-must-not-replace-unknown".into();
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &work, Some(late_attempt),
        &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
    ), InboundAppendAction::Stale);
    assert_eq!(wakes.pause_inbound_append("agent-a", &work), InboundAppendAction::Stale);
    assert!(!wakes.finish_inbound_append("agent-a", &work));
    assert!(!wakes.finish_inbound_append_with_redrive(
        "agent-a", &work, Some(redrivable_inbound_envelopes(&work.envelopes)),
    ));
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &work, None,
        &ChannelInboundAppendOutcome::NotWritten(inbound_append_failure_fixture(
            ChannelInboundAppendFailureKind::Storage,
        )),
    ), InboundAppendAction::Stale);
    assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
    assert!(!wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::Unknown));
    assert_eq!(wakes.inbound_append_attempt("agent-a"), Some(&attempt));
    assert_eq!(wakes.pending_inbound.get("agent-a"), Some(&tail));
}

#[test]
fn inbound_append_committed_head_grants_one_handoff_permission_and_one_finish_before_next_batch() {
    let mut wakes = BackgroundWakes::<Value>::default();
    for envelope in inbound_append_envelopes_fixture() {
        BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", envelope);
    }
    assert!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"));
    let first = wakes.take_inbound_append_or_end_revival("agent-a").expect("first head");
    assert!(!wakes.finish_inbound_append("agent-a", &first));
    let attempt = inbound_append_attempt_fixture();
    let committed = ChannelInboundAppendOutcome::Committed { inserted_count: 2 };
    assert_eq!(wakes.record_inbound_append_outcome("agent-a", &first, Some(attempt.clone()), &committed), InboundAppendAction::Committed);
    assert!(!wakes.owns_inbound_append("agent-a", &first));
    assert_eq!(wakes.record_inbound_append_outcome("agent-a", &first, Some(attempt.clone()), &committed), InboundAppendAction::Stale);
    let tail = vec![json!({"text":"next batch first"}), json!({"text":"next batch second"})];
    for envelope in &tail {
        BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", envelope.clone());
        assert!(!BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"));
    }
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
    assert_eq!(wakes.pause_inbound_append("agent-a", &first), InboundAppendAction::Stale);
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &first, None,
        &ChannelInboundAppendOutcome::Unknown(inbound_append_failure_fixture(
            ChannelInboundAppendFailureKind::Storage,
        )),
    ), InboundAppendAction::Stale);
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::Committed));
    assert_eq!(wakes.inbound_append_attempt("agent-a"), Some(&attempt));
    assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(wakes.pending_inbound.get("agent-a"), Some(&tail));

    assert!(wakes.finish_inbound_append("agent-a", &first));
    assert_eq!(wakes.inbound_append_state("agent-a"), None);
    assert_eq!(wakes.inbound_append_attempt("agent-a"), None);
    assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
    let next = wakes.take_inbound_append_or_end_revival("agent-a").expect("next batch after finish");
    assert_eq!(next.envelopes, tail);
    assert_ne!(next.claim_id, first.claim_id);
    assert_ne!(next.worker_token, first.worker_token);
    assert_eq!(next.attempt, None);
    assert!(wakes.owns_inbound_append("agent-a", &next));
    assert!(!wakes.owns_inbound_append("agent-a", &first));
    assert!(!wakes.finish_inbound_append("agent-a", &first));
    assert_eq!(wakes.record_inbound_append_outcome("agent-a", &first, Some(attempt), &committed), InboundAppendAction::Stale);
    assert_eq!(wakes.pause_inbound_append("agent-a", &first), InboundAppendAction::Stale);
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::Active));
    assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert!(!wakes.pending_inbound.contains_key("agent-a"));
}

#[test]
fn inbound_append_old_worker_cannot_update_or_clear_replacement_owner_with_same_agent_and_payload() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
    let envelope = json!({"text":"identical payload across owner replacement"});
    let old = {
        let mut state = wakes.lock().expect("old owner queue");
        BackgroundWakes::enqueue(&mut state.pending_inbound, "agent-a", envelope.clone());
        assert!(BackgroundWakes::<Value>::begin_revival(&mut state.reviving_inbound_agent_ids, "agent-a"));
        state.take_inbound_append_or_end_revival("agent-a").expect("old head")
    };
    let (resume_tx, resume_rx) = mpsc::channel();
    let old_wakes = Arc::clone(&wakes);
    let stale = old.clone();
    let old_worker = thread::spawn(move || {
        resume_rx.recv().expect("replacement acquired before stale callbacks");
        let mut state = old_wakes.lock().expect("stale callback queue");
        let mut results = vec![state.pause_inbound_append("agent-a", &stale)];
        for outcome in [
            ChannelInboundAppendOutcome::NotWritten(inbound_append_failure_fixture(ChannelInboundAppendFailureKind::Storage)),
            ChannelInboundAppendOutcome::Unknown(inbound_append_failure_fixture(ChannelInboundAppendFailureKind::RollbackUnconfirmed)),
            ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
        ] {
            results.push(state.record_inbound_append_outcome(
                "agent-a", &stale, Some(inbound_append_attempt_fixture()), &outcome,
            ));
        }
        (results, state.finish_inbound_append("agent-a", &stale))
    });

    let replacement = {
        let mut state = wakes.lock().expect("replace queue owner under mutex");
        // This tests token rejection by a replacement queue, not production
        // dispose/recreate or durable recovery acceptance.
        *state = BackgroundWakes::<Value>::default();
        BackgroundWakes::enqueue(&mut state.pending_inbound, "agent-a", envelope.clone());
        assert!(BackgroundWakes::<Value>::begin_revival(&mut state.reviving_inbound_agent_ids, "agent-a"));
        state.take_inbound_append_or_end_revival("agent-a").expect("replacement head")
    };
    assert_ne!(replacement.claim_id, old.claim_id);
    assert_ne!(replacement.worker_token, old.worker_token);
    assert_eq!(replacement.envelopes, old.envelopes);
    resume_tx.send(()).expect("release stale callbacks");
    assert_eq!(old_worker.join().expect("old worker"), (
        vec![InboundAppendAction::Stale; 4], false,
    ));
    let mut state = wakes.lock().expect("replacement after stale callbacks");
    assert!(state.owns_inbound_append("agent-a", &replacement));
    assert!(!state.owns_inbound_append("agent-a", &old));
    assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::Active));
    assert_eq!(state.inbound_append_attempt("agent-a"), None);
    assert!(state.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(state.take_inbound_append_or_end_revival("agent-a"), None);
    let mut mismatched_claim = replacement.clone();
    mismatched_claim.claim_id = old.claim_id.clone();
    assert!(!state.owns_inbound_append("agent-a", &mismatched_claim));
    assert_eq!(state.pause_inbound_append("agent-a", &mismatched_claim), InboundAppendAction::Stale);
    assert_eq!(state.record_inbound_append_outcome(
        "agent-a", &mismatched_claim, Some(inbound_append_attempt_fixture()),
        &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
    ), InboundAppendAction::Stale);
    assert!(state.owns_inbound_append("agent-a", &replacement));
    assert_eq!(state.pause_inbound_append("agent-a", &replacement), InboundAppendAction::Held);
    assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::NotWritten));
}

#[test]
fn inbound_append_unknown_agent_does_not_block_other_agents_or_change_other_wake_families() {
    let mut wakes = BackgroundWakes::<Value>::default();
    for agent_id in ["agent-a", "agent-b"] {
        for envelope in inbound_append_envelopes_fixture() {
            BackgroundWakes::enqueue(&mut wakes.pending_inbound, agent_id, envelope);
        }
        assert!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, agent_id));
        BackgroundWakes::enqueue(&mut wakes.pending_channel_failures, agent_id, json!({"failure":agent_id}));
        BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_channel_failure_agent_ids, agent_id);
        BackgroundWakes::enqueue(&mut wakes.pending_event_wakes, agent_id, json!({"event":agent_id}));
        BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_event_agent_ids, agent_id);
        wakes.dm_preempted_wake_agent_ids.insert(agent_id.into());
    }
    let unrelated_families = |state: &BackgroundWakes<Value>| {
        (
            state.pending_channel_failures.clone(),
            state.reviving_channel_failure_agent_ids.clone(),
            state.pending_event_wakes.clone(),
            state.reviving_event_agent_ids.clone(),
            state.dm_preempted_wake_agent_ids.clone(),
        )
    };
    let expected_families = unrelated_families(&wakes);
    let work_a = wakes.take_inbound_append_or_end_revival("agent-a").expect("agent A head");
    let work_b = wakes.take_inbound_append_or_end_revival("agent-b").expect("agent B head");
    assert!(!wakes.owns_inbound_append("agent-b", &work_a));
    assert!(!wakes.owns_inbound_append("agent-a", &work_b));
    let attempt_a = inbound_append_attempt_fixture();
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &work_a, Some(attempt_a.clone()),
        &ChannelInboundAppendOutcome::Unknown(inbound_append_failure_fixture(ChannelInboundAppendFailureKind::Storage)),
    ), InboundAppendAction::Held);
    assert_eq!(wakes.pause_inbound_append("agent-b", &work_a), InboundAppendAction::Stale);
    assert!(!wakes.finish_inbound_append("agent-b", &work_a));
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-b", &work_a, Some(attempt_a.clone()),
        &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
    ), InboundAppendAction::Stale);
    assert_eq!(wakes.inbound_append_state("agent-b"), Some(InboundAppendState::Active));
    assert_eq!(wakes.inbound_append_attempt("agent-b"), None);
    assert!(wakes.reviving_inbound_agent_ids.contains("agent-b"));
    assert!(wakes.owns_inbound_append("agent-b", &work_b));
    assert_eq!(unrelated_families(&wakes), expected_families);

    let tail_a = json!({"inbound":"agent-a-later"});
    let tail_b = json!({"inbound":"agent-b-later"});
    BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", tail_a.clone());
    assert!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"));
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
    BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-b", tail_b.clone());
    assert!(!BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-b"));
    let mut attempt_b = inbound_append_attempt_fixture();
    attempt_b.session_key = "session-key-b".into();
    attempt_b.session_id = "session-id-b".into();
    attempt_b.lifecycle_revision = "lifecycle-b".into();
    for entry in attempt_b.entries.as_mut().expect("prepared B entries") {
        entry["sessionKey"] = json!("session-key-b");
    }
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-b", &work_b, Some(attempt_b),
        &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
    ), InboundAppendAction::Committed);
    assert!(wakes.finish_inbound_append("agent-b", &work_b));
    let next_b = wakes.take_inbound_append_or_end_revival("agent-b").expect("agent B progresses independently");
    assert_eq!(next_b.envelopes, vec![tail_b]);
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::Unknown));
    assert_eq!(wakes.inbound_append_attempt("agent-a"), Some(&attempt_a));
    assert_eq!(wakes.pending_inbound.get("agent-a"), Some(&vec![tail_a]));
    assert_eq!(unrelated_families(&wakes), expected_families);
}

#[test]
fn inbound_append_displayed_dm_redrive_keeps_empty_preparation_and_grants_zero_row_commit_once() {
    let mut wakes = BackgroundWakes::<Value>::default();
    let envelopes = inbound_append_envelopes_fixture();
    let redriven = redrivable_inbound_envelopes(&envelopes);
    assert_eq!(redriven.len(), 2);
    assert!(redriven.iter().all(|envelope| envelope["isDisplayed"] == true && envelope["isRedriven"] == true));
    assert!(redrivable_inbound_envelopes(&redriven).is_empty());
    for envelope in &envelopes {
        BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", envelope.clone());
    }
    assert!(BackgroundWakes::<Value>::begin_revival(&mut wakes.reviving_inbound_agent_ids, "agent-a"));
    let original = wakes.take_inbound_append_or_end_revival("agent-a").expect("original input head");
    assert_eq!(original.attempt, None);
    assert_eq!(wakes.record_inbound_append_outcome(
        "agent-a", &original, Some(inbound_append_attempt_fixture()),
        &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
    ), InboundAppendAction::Committed);
    assert!(wakes.finish_inbound_append_with_redrive("agent-a", &original, Some(redriven.clone())));
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::NotWritten));
    assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert!(!wakes.pending_inbound.contains_key("agent-a"));
    let work = wakes.take_inbound_append_or_end_revival("agent-a").expect("displayed redrive head");
    assert_eq!(work.envelopes, redriven);
    assert_ne!(work.claim_id, original.claim_id);
    assert_ne!(work.worker_token, original.worker_token);
    let mut attempt = inbound_append_attempt_fixture();
    attempt.entries = Some(Vec::new());
    assert_eq!(work.attempt, Some(attempt.clone()));
    let committed = ChannelInboundAppendOutcome::Committed { inserted_count: 0 };
    assert_eq!(wakes.record_inbound_append_outcome("agent-a", &work, Some(attempt.clone()), &committed), InboundAppendAction::Committed);
    assert_eq!(wakes.inbound_append_attempt("agent-a"), Some(&attempt));
    assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::Committed));
    assert!(!wakes.owns_inbound_append("agent-a", &work));
    assert_eq!(wakes.record_inbound_append_outcome("agent-a", &work, Some(attempt), &committed), InboundAppendAction::Stale);
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
    assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
    assert!(wakes.finish_inbound_append("agent-a", &work));
    assert!(!wakes.finish_inbound_append("agent-a", &work));
    assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
    assert_eq!(wakes.inbound_append_state("agent-a"), None);
    assert!(!wakes.reviving_inbound_agent_ids.contains("agent-a"));
}

#[test]
fn inbound_append_bound_redrive_precedes_raw_tail_and_rejects_delayed_original_worker() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
    let envelopes = inbound_append_envelopes_fixture();
    let prepared = inbound_append_attempt_fixture();
    let mut continuation_attempt = prepared.clone();
    continuation_attempt.entries = Some(Vec::new());
    let original = {
        let mut state = wakes.lock().expect("original input queue");
        for envelope in &envelopes {
            BackgroundWakes::enqueue(&mut state.pending_inbound, "agent-a", envelope.clone());
        }
        assert!(BackgroundWakes::<Value>::begin_revival(
            &mut state.reviving_inbound_agent_ids, "agent-a",
        ));
        let work = state.take_inbound_append_or_end_revival("agent-a").expect("original head");
        assert_eq!(state.record_inbound_append_outcome(
            "agent-a", &work, Some(prepared.clone()),
            &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
        ), InboundAppendAction::Committed);
        work
    };
    let (resume_tx, resume_rx) = mpsc::channel();
    let old_wakes = Arc::clone(&wakes);
    let old_work = original.clone();
    let old_worker = thread::spawn(move || {
        resume_rx.recv().expect("continuation acquired before stale callback");
        let mut changed_attempt = inbound_append_attempt_fixture();
        changed_attempt.lifecycle_revision = "later-generation-must-not-rebind".into();
        let mut state = old_wakes.lock().expect("delayed original worker queue");
        (
            state.pause_inbound_append("agent-a", &old_work),
            state.record_inbound_append_outcome(
                "agent-a", &old_work, Some(changed_attempt),
                &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
            ),
            state.finish_inbound_append("agent-a", &old_work),
            state.finish_inbound_append_with_redrive(
                "agent-a", &old_work,
                Some(redrivable_inbound_envelopes(&old_work.envelopes)),
            ),
        )
    });

    let tail = vec![json!({"text":"later input first"}), json!({"text":"later input second"})];
    let redriven = redrivable_inbound_envelopes(&envelopes);
    let continuation = {
        let mut state = wakes.lock().expect("atomic redrive queue");
        for envelope in &tail {
            BackgroundWakes::enqueue(&mut state.pending_inbound, "agent-a", envelope.clone());
            assert!(!BackgroundWakes::<Value>::begin_revival(
                &mut state.reviving_inbound_agent_ids, "agent-a",
            ));
        }
        let mut wrong_token = original.clone();
        wrong_token.worker_token.push_str("-stale");
        assert!(!state.finish_inbound_append_with_redrive(
            "agent-a", &wrong_token, Some(redriven.clone()),
        ));
        let mut wrong_claim = original.clone();
        wrong_claim.claim_id.push_str("-stale");
        assert!(!state.finish_inbound_append_with_redrive(
            "agent-a", &wrong_claim, Some(redriven.clone()),
        ));
        assert!(state.finish_inbound_append_with_redrive(
            "agent-a", &original, Some(redriven.clone()),
        ));
        assert_eq!(state.inbound_append_state("agent-a"), Some(InboundAppendState::NotWritten));
        assert_eq!(state.inbound_append_attempt("agent-a"), Some(&continuation_attempt));
        assert_eq!(state.pending_inbound.get("agent-a"), Some(&tail));
        assert!(state.reviving_inbound_agent_ids.contains("agent-a"));
        assert!(!state.finish_inbound_append("agent-a", &original));
        // The same drain acquires this continuation without a fresh ingress or
        // revival claim; raw later input is still separately queued behind it.
        let next = state.take_inbound_append_or_end_revival("agent-a").expect("bound continuation");
        assert_ne!(next.claim_id, original.claim_id);
        assert_ne!(next.worker_token, original.worker_token);
        assert_eq!(next.envelopes, redriven);
        assert_eq!(next.attempt, Some(continuation_attempt.clone()));
        assert_eq!(state.pending_inbound.get("agent-a"), Some(&tail));
        next
    };
    resume_tx.send(()).expect("release original worker after continuation acquisition");
    assert_eq!(old_worker.join().expect("original worker"), (
        InboundAppendAction::Stale, InboundAppendAction::Stale, false, false,
    ));

    let mut state = wakes.lock().expect("continuation after delayed original worker");
    assert!(state.owns_inbound_append("agent-a", &continuation));
    assert!(!state.owns_inbound_append("agent-a", &original));
    assert_eq!(state.inbound_append_attempt("agent-a"), Some(&continuation_attempt));
    assert_eq!(state.pending_inbound.get("agent-a"), Some(&tail));
    assert!(state.reviving_inbound_agent_ids.contains("agent-a"));
    assert_eq!(state.record_inbound_append_outcome(
        "agent-a", &continuation, continuation.attempt.clone(),
        &ChannelInboundAppendOutcome::Committed { inserted_count: 0 },
    ), InboundAppendAction::Committed);
    assert!(!state.finish_inbound_append_with_redrive(
        "agent-a", &original, Some(redriven),
    ));
    let no_second_redrive = redrivable_inbound_envelopes(&continuation.envelopes);
    assert!(no_second_redrive.is_empty());
    assert!(state.finish_inbound_append_with_redrive(
        "agent-a", &continuation, Some(no_second_redrive),
    ));
    assert_eq!(state.inbound_append_state("agent-a"), None);
    assert!(state.reviving_inbound_agent_ids.contains("agent-a"));
    let next = state.take_inbound_append_or_end_revival("agent-a").expect("raw tail follows continuation");
    assert_eq!(next.envelopes, tail);
    assert_eq!(next.attempt, None);
    assert_ne!(next.claim_id, continuation.claim_id);
    assert!(!state.pending_inbound.contains_key("agent-a"));
}

#[test]
fn inbound_append_redrive_refuses_missing_preparation_or_binding_and_retains_original_head() {
    let mut unprepared = inbound_append_attempt_fixture();
    unprepared.entries = None;
    let mut missing_key = inbound_append_attempt_fixture();
    missing_key.session_key.clear();
    let mut missing_session_id = inbound_append_attempt_fixture();
    missing_session_id.session_id.clear();
    let mut missing_revision = inbound_append_attempt_fixture();
    missing_revision.lifecycle_revision = " ".into();

    for (label, captured) in [
        ("missing attempt", None),
        ("preparation incomplete", Some(unprepared)),
        ("missing session key", Some(missing_key)),
        ("missing session ID", Some(missing_session_id)),
        ("blank lifecycle revision", Some(missing_revision)),
    ] {
        let mut wakes = BackgroundWakes::<Value>::default();
        for envelope in inbound_append_envelopes_fixture() {
            BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", envelope);
        }
        assert!(BackgroundWakes::<Value>::begin_revival(
            &mut wakes.reviving_inbound_agent_ids, "agent-a",
        ));
        let work = wakes.take_inbound_append_or_end_revival("agent-a").expect("original head");
        let redriven = redrivable_inbound_envelopes(&work.envelopes);
        assert!(!wakes.finish_inbound_append_with_redrive(
            "agent-a", &work, Some(redriven.clone()),
        ), "Active input cannot redrive: {label}");
        assert!(wakes.owns_inbound_append("agent-a", &work));
        assert_eq!(wakes.record_inbound_append_outcome(
            "agent-a", &work, captured.clone(),
            &ChannelInboundAppendOutcome::Committed { inserted_count: 2 },
        ), InboundAppendAction::Committed);
        let tail = vec![json!({"text":"later raw input stays behind held head"})];
        BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent-a", tail[0].clone());
        assert!(!wakes.finish_inbound_append_with_redrive(
            "agent-a", &work, Some(redriven),
        ), "Must not recapture an unavailable binding: {label}");
        assert_eq!(wakes.inbound_append_state("agent-a"), Some(InboundAppendState::Committed));
        assert_eq!(wakes.inbound_append_attempt("agent-a"), captured.as_ref());
        assert_eq!(wakes.pending_inbound.get("agent-a"), Some(&tail));
        assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
        assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
        // No-redrive completion preserves the existing API's behavior even
        // when this defensive queue-boundary fixture has no usable preparation.
        assert!(wakes.finish_inbound_append_with_redrive("agent-a", &work, None));
        assert_eq!(wakes.inbound_append_state("agent-a"), None);
        assert!(wakes.reviving_inbound_agent_ids.contains("agent-a"));
        assert_eq!(
            wakes.take_inbound_append_or_end_revival("agent-a").expect("tail after ordinary finish").envelopes,
            tail,
        );
    }
}

#[test]
fn inbound_image_projection_preserves_all_frozen_image_payloads() {
    let images = collect_inbound_images(&[
        json!({"images":[{"data":"aGVsbG8=","mimeType":"image/png"}]}),
        json!({"images":[{"data":"d29ybGQ=","mimeType":"image/webp"}]}),
    ]);
    assert_eq!(images.len(), 2);
    assert_eq!(images[0].data, "aGVsbG8=");
    assert_eq!(images[0].mime_type, "image/png");
    assert_eq!(images[1].data, "d29ybGQ=");
}

// A bounded source-wiring supplement to the actual image projection controls.
// Parse the named shipping function so a fixture, comment or unrelated worker
// cannot satisfy this check. Borrowing an already borrowed slice is equivalent.
fn shipping_inbound_worker_collects_envelope_images(source: &str) -> bool {
    fn is_unconditional(attributes: &[syn::Attribute]) -> bool {
        !attributes.iter().any(|attr| {
            attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr")
        })
    }
    fn expression_is_unconditional(expression: &syn::Expr) -> bool {
        let attributes = match expression {
            syn::Expr::Path(value) => &value.attrs,
            syn::Expr::Call(value) => &value.attrs,
            syn::Expr::MethodCall(value) => &value.attrs,
            syn::Expr::Closure(value) => &value.attrs,
            syn::Expr::Block(value) => &value.attrs,
            syn::Expr::If(value) => &value.attrs,
            syn::Expr::Match(value) => &value.attrs,
            syn::Expr::Paren(value) => &value.attrs,
            syn::Expr::Group(value) => &value.attrs,
            syn::Expr::Try(value) => &value.attrs,
            syn::Expr::Await(value) => &value.attrs,
            syn::Expr::Reference(value) => &value.attrs,
            _ => return false,
        };
        is_unconditional(attributes)
    }
    let Ok(file) = syn::parse_file(source) else { return false; };
    let mut workers = file.items.iter().filter_map(|item| match item {
        syn::Item::Fn(function) if function.sig.ident == "run_channel_inbound_revival_worker" => Some(function),
        _ => None,
    });
    let Some(worker) = workers.next() else { return false; };
    if workers.next().is_some() || !is_unconditional(&worker.attrs) {
        return false;
    }

    fn envelopes(expression: &syn::Expr) -> bool {
        if !expression_is_unconditional(expression) {
            return false;
        }
        match expression {
            syn::Expr::Path(path) => path.qself.is_none() && path.path.is_ident("envelopes"),
            syn::Expr::Reference(reference) if reference.mutability.is_none() => envelopes(&reference.expr),
            syn::Expr::Paren(paren) => envelopes(&paren.expr),
            _ => false,
        }
    }
    fn block_has_projection(block: &syn::Block) -> bool {
        block.stmts.iter().any(|statement| match statement {
            syn::Stmt::Local(local) if is_unconditional(&local.attrs) =>
                local.init.as_ref().is_some_and(|init| expression_has_projection(&init.expr)),
            syn::Stmt::Expr(expression, _) => expression_has_projection(expression),
            _ => false,
        })
    }
    fn expression_has_projection(expression: &syn::Expr) -> bool {
        if !expression_is_unconditional(expression) {
            return false;
        }
        match expression {
            syn::Expr::Call(call) => {
                let is_projection = matches!(call.func.as_ref(), syn::Expr::Path(path)
                    if is_unconditional(&path.attrs) && path.qself.is_none()
                        && path.path.is_ident("collect_inbound_images"));
                (is_projection && call.args.len() == 1 && envelopes(&call.args[0]))
                    || call.args.iter().any(expression_has_projection)
            }
            syn::Expr::MethodCall(call) => expression_has_projection(&call.receiver)
                || call.args.iter().any(expression_has_projection),
            syn::Expr::Closure(closure) => expression_has_projection(&closure.body),
            syn::Expr::Block(block) => block_has_projection(&block.block),
            syn::Expr::If(branch) => block_has_projection(&branch.then_branch)
                || branch.else_branch.as_ref().is_some_and(|(_, expression)| expression_has_projection(expression)),
            syn::Expr::Match(branch) => branch.arms.iter().any(|arm|
                is_unconditional(&arm.attrs) && expression_has_projection(&arm.body)),
            syn::Expr::Paren(paren) => expression_has_projection(&paren.expr),
            syn::Expr::Group(group) => expression_has_projection(&group.expr),
            syn::Expr::Try(attempt) => expression_has_projection(&attempt.expr),
            syn::Expr::Await(awaited) => expression_has_projection(&awaited.base),
            syn::Expr::Reference(reference) => expression_has_projection(&reference.expr),
            _ => false,
        }
    }
    block_has_projection(&worker.block)
}

#[test]
fn inbound_image_wiring_oracle_accepts_slice_calls_and_rejects_missing_or_wrong_inputs() {
    const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
    assert!(shipping_inbound_worker_collects_envelope_images(SHIPPING_HOST));
    for expression in ["collect_inbound_images(envelopes)", "collect_inbound_images(&envelopes)",
        "collect_inbound_images(\n  & (envelopes),\n)"] {
        let source = format!("fn run_channel_inbound_revival_worker() {{ let selected_images = {expression}; }}");
        assert!(syn::parse_file(&source).is_ok(), "control must parse");
        assert!(shipping_inbound_worker_collects_envelope_images(&source));
    }
    for expression in ["Vec::<InboundImage>::new()", "collect_inbound_images(&[])", "collect_inbound_images(other_envelopes)"] {
        let source = format!("fn run_channel_inbound_revival_worker() {{ let selected_images = {expression}; }}");
        assert!(syn::parse_file(&source).is_ok(), "control must parse");
        assert!(!shipping_inbound_worker_collects_envelope_images(&source),
            "missing or wrong-input projection must fail: {expression}");
    }
    for body in [
        "#[cfg(test)] { let _ = collect_inbound_images(envelopes); } let selected_images = Vec::new();",
        "#[cfg(test)] collect_inbound_images(envelopes); let selected_images = Vec::new();",
        "#[cfg_attr(test, allow(unused))] { collect_inbound_images(envelopes); }",
        "match value { #[cfg(test)] Some(_) => collect_inbound_images(envelopes), _ => Vec::new() };",
    ] {
        let source = format!("fn run_channel_inbound_revival_worker() {{ {body} }}");
        assert!(syn::parse_file(&source).is_ok(), "cfg control must parse: {body}");
        assert!(!shipping_inbound_worker_collects_envelope_images(&source),
            "conditional expression cannot provide shipping evidence: {body}");
    }
    assert!(!shipping_inbound_worker_collects_envelope_images(r#"
        fn run_channel_inbound_revival_worker() { let _ = "collect_inbound_images(envelopes)"; }
        #[cfg(test)] mod tests {
            fn fixture() { collect_inbound_images(envelopes); }
        }
    "#));
    assert!(!shipping_inbound_worker_collects_envelope_images(
        "#[cfg(test)] fn run_channel_inbound_revival_worker() { collect_inbound_images(envelopes); }",
    ));
}

#[test]
fn shipping_send_message_reaches_host_owned_delivery_failure_broadcast_and_context_paths() {
    const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

    let send_sink = SHIPPING_HOST
        .find("struct ProductionSendMessageSink")
        .expect("shipping SendMessage sink");
    let dispatch = SHIPPING_HOST[send_sink..]
        .find("\"deliverToChannel\"")
        .map(|offset| send_sink + offset)
        .expect("channel delivery dispatch");
    let gateway = SHIPPING_HOST[dispatch..]
        .find("if method == \"deliverToChannel\"")
        .map(|offset| dispatch + offset)
        .expect("Host channel owner");
    assert!(send_sink < dispatch && dispatch < gateway);
    let persisted_entry = SHIPPING_HOST[send_sink..]
        .find("append_generated_send_message_entry(")
        .map(|offset| send_sink + offset)
        .expect("transcript entry persistence");
    assert!(persisted_entry < dispatch, "delivery dispatch must follow transcript persistence");
    assert!(SHIPPING_HOST.contains("deliveryOperationId"));
    assert!(SHIPPING_HOST.contains("delivery_operation_id: Option<String>"));
    assert!(SHIPPING_HOST.contains(
        "for cleanup_error in custody.rollback()"
    ));
    assert!(SHIPPING_HOST.contains(
        "outgoing_media_cleanup_failed agent={} tool={} error={cleanup_error}"
    ));

    assert!(SHIPPING_HOST.contains("queue_channel_delivery_failure("));
    assert!(SHIPPING_HOST.contains("\"markedAtMs\": marked_at_ms"));
    assert!(SHIPPING_HOST.contains("fn channel_failure_marker_from_failure("));
    assert!(SHIPPING_HOST.contains("run_channel_failure_revival_worker"));
    assert!(SHIPPING_HOST.contains("clear_durable_channel_failure_markers"));
    assert!(SHIPPING_HOST.contains("store.clear_if_matches(&marker)"));
    assert!(SHIPPING_HOST.contains("channel_failure_marker_identity_incomplete"));
    assert!(SHIPPING_HOST.contains(
        "Some(_) => {\n                stale.push(failure.clone());\n                false\n            }\n            None => true"
    ));
    assert!(SHIPPING_HOST.contains(
        "return Err(\"channel failure session provenance is unavailable\".into());"
    ));
    assert!(SHIPPING_HOST.contains(
        "Ok(None) => {\n                clear_durable_channel_failure_markers(&deps, &agent_id, &failures);\n                continue;\n            }"
    ));
    assert!(SHIPPING_HOST.contains(
        "!execution.aborted\n                    && !execution.quiesced_for_upgrade\n                    && execution.sent_message_count > 0"
    ));
    assert!(SHIPPING_HOST.contains("requeue_channel_failures(&wakes, &agent_id, failures)"));
    assert!(SHIPPING_HOST.contains("tray.title = \"Delivery-failure follow-up failed\".into();\n                deps.trays.push_error(tray);\n                requeue_channel_failures(&wakes, &agent_id, failures);\n                break;"));
    assert!(SHIPPING_HOST.contains("tray.title = \"Message not delivered\""));
    assert!(SHIPPING_HOST.contains("tray.title = \"Delivery-failure follow-up failed\""));
    assert!(SHIPPING_HOST.contains("summary.is_group || summary.remote_room.is_some()"));
    assert!(shipping_inbound_worker_collects_envelope_images(SHIPPING_HOST));
    assert!(SHIPPING_HOST.contains("collect_unanswered_question_prompts(&agent_id)"));
    assert!(SHIPPING_HOST.contains("\"selectedImages\""));
    assert!(SHIPPING_HOST.contains("\"skippedQuestionPrompts\""));
    assert!(SHIPPING_HOST.contains("\"dismissedQuestionPrompts\""));
    assert!(SHIPPING_HOST.contains("if method == \"broadcastToAgents\""));
    assert!(SHIPPING_HOST.contains("build_admin_broadcast_wake_prompt(&text)"));
    assert!(SHIPPING_HOST.contains("REPLY_NUDGE_PROMPT"));
    assert!(SHIPPING_HOST.contains("\"channel-connected\""));
    assert!(SHIPPING_HOST.contains("\"channel-disconnected\""));
    assert!(SHIPPING_HOST.contains("\"automation-changed\""));
}
