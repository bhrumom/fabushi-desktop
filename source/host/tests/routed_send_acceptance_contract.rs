use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::transcript::production_runtime::{
    ProductionSendError, ProductionTranscriptRuntime,
};
use mahayana_host_runtime::extensions::transcript::send_pipeline::PersistedSendContext;
use mahayana_host_runtime::runner::RecoveryUserMessage;

#[test]
fn routed_prompt_admission_owns_nonce_durability_and_recovery_identity() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-routed-admission-{}",
        uuid::Uuid::new_v4()
    ));
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let lifecycle = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let lifecycle_observer = Arc::clone(&lifecycle);
    runtime.set_agent_run_lifecycle_observer(Some(Arc::new(move |event_type, request_id| {
        lifecycle_observer
            .lock()
            .expect("lifecycle")
            .push((event_type.to_string(), request_id.to_string()));
    })));
    let args = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "hello",
        "clientNonce": "routed-nonce",
        "streamId": "stream-a"
    });
    let persists = AtomicUsize::new(0);

    let first = runtime
        .accept_routed_send(&args, |_: &serde_json::Value| {
            persists.fetch_add(1, Ordering::SeqCst);
            Ok::<_, ProductionSendError>(PersistedSendContext {
                echo_entry_id: Some("user-message:1".into()),
                user_message_id: Some("user-message:1".into()),
                recent_user_messages: vec![RecoveryUserMessage {
                    id: "user-message:1".into(),
                    text: "hello".into(),
                    confirmed: None,
                }],
                ..PersistedSendContext::default()
            })
        })
        .expect("first routed admission");
    assert!(!first.duplicate);
    assert_eq!(first.context.user_message_id.as_deref(), Some("user-message:1"));
    assert_eq!(runtime.current_turn_epoch("agent-a"), 1);
    assert_eq!(runtime.in_flight_run_count("agent-a"), 1);
    assert!(!runtime.is_turn_dispatch_idle("agent-a"));
    assert!(runtime.has_routed_turn_lease("agent-a", "stream-a"));
    assert_eq!(
        *lifecycle.lock().expect("lifecycle"),
        vec![("started".to_string(), "stream-a".to_string())]
    );

    let duplicate = runtime
        .accept_routed_send(&args, |_: &serde_json::Value| {
            persists.fetch_add(1, Ordering::SeqCst);
            Ok::<_, ProductionSendError>(PersistedSendContext {
                echo_entry_id: Some("user-message:1".into()),
                user_message_id: Some("user-message:1".into()),
                recent_user_messages: vec![RecoveryUserMessage {
                    id: "user-message:1".into(),
                    text: "hello".into(),
                    confirmed: None,
                }],
                ..PersistedSendContext::default()
            })
        })
        .expect("duplicate routed admission");
    assert!(duplicate.duplicate);
    assert_eq!(duplicate.context.echo_entry_id.as_deref(), Some("user-message:1"));
    assert_eq!(runtime.current_turn_epoch("agent-a"), 1);
    assert_eq!(runtime.in_flight_run_count("agent-a"), 1);
    assert_eq!(persists.load(Ordering::SeqCst), 2);

    runtime
        .require_routed_turn_lease("agent-a", "stream-a")
        .expect("provider start lease");
    runtime
        .settle_routed_turn("agent-a", "stream-a", 10_000)
        .expect("terminal settlement")
        .expect("lease settlement");
    assert_eq!(runtime.in_flight_run_count("agent-a"), 0);
    assert!(runtime.is_turn_dispatch_idle("agent-a"));
    assert!(!runtime.has_routed_turn_lease("agent-a", "stream-a"));
    assert_eq!(
        *lifecycle.lock().expect("lifecycle"),
        vec![
            ("started".to_string(), "stream-a".to_string()),
            ("ended".to_string(), "stream-a".to_string()),
        ]
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn human_handoff_admission_binds_destination_to_the_host_owned_turn_lease() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-routed-human-handoff-{}",
        uuid::Uuid::new_v4()
    ));
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let args = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "continue this Human conversation",
        "clientNonce": "human-handoff-nonce",
        "streamId": "human-handoff-stream",
        "requestSource": "human-handoff",
        "humanHandoffConversationId": "human-conversation:abc"
    });
    runtime
        .accept_routed_send(&args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        })
        .expect("Human handoff admission");
    assert_eq!(
        runtime
            .routed_human_handoff_conversation_id("agent-a", "human-handoff-stream")
            .as_deref(),
        Some("human-conversation:abc")
    );
    runtime
        .settle_routed_turn("agent-a", "human-handoff-stream", 10_000)
        .expect("settlement")
        .expect("lease");
    assert_eq!(
        runtime.routed_human_handoff_conversation_id("agent-a", "human-handoff-stream"),
        None
    );

    let missing_destination = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "missing destination",
        "clientNonce": "missing-destination",
        "streamId": "missing-destination-stream",
        "requestSource": "human-handoff"
    });
    assert!(matches!(
        runtime.accept_routed_send(&missing_destination, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        }),
        Err(ProductionSendError::BadRequest(_))
    ));

    let forged_destination = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "not a handoff",
        "clientNonce": "forged-destination",
        "streamId": "forged-destination-stream",
        "humanHandoffConversationId": "human-conversation:abc"
    });
    assert!(matches!(
        runtime.accept_routed_send(&forged_destination, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        }),
        Err(ProductionSendError::BadRequest(_))
    ));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn routed_prompt_queue_lease_blocks_the_next_turn_until_runner_terminal() {
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    let root = std::env::temp_dir().join(format!(
        "fabushi-routed-admission-queue-{}",
        uuid::Uuid::new_v4()
    ));
    let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
    let first_args = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "first",
        "clientNonce": "nonce-a",
        "streamId": "stream-a"
    });
    runtime
        .accept_routed_send(&first_args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext {
                echo_entry_id: Some("user-message:1".into()),
                user_message_id: Some("user-message:1".into()),
                recent_user_messages: vec![],
                ..PersistedSendContext::default()
            })
        })
        .expect("first admission");

    let second_runtime = Arc::clone(&runtime);
    let (tx, rx) = mpsc::channel();
    let (queued_tx, queued_rx) = mpsc::channel();
    let join = std::thread::spawn(move || {
        let args = serde_json::json!({
            "agentId": "agent-a",
            "prompt": "second",
            "clientNonce": "nonce-b",
            "streamId": "stream-b"
        });
        let result = second_runtime.accept_routed_send_with_queue_observers(&args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext {
                echo_entry_id: Some("user-message:2".into()),
                user_message_id: Some("user-message:2".into()),
                recent_user_messages: vec![],
                ..PersistedSendContext::default()
            })
        }, |_| false, |event| {
            queued_tx.send(event.clone()).expect("publish actual queue admission");
        }, |_| {});
        tx.send(result).expect("send second admission");
    });

    let queued = queued_rx.recv_timeout(Duration::from_secs(5))
        .expect("second turn reached the real queue admission boundary");
    assert_eq!(queued.agent_id, "agent-a");
    assert_eq!(queued.lane,
        mahayana_host_runtime::extensions::transcript::run_scheduler::RunLane::User);
    assert!(queued.has_active, "the first Runner still owns its active lease");
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_eq!(runtime.in_flight_run_count("agent-a"), 2);
    assert_eq!(runtime.queued_turn_count("agent-a"), 1);

    runtime
        .settle_routed_turn("agent-a", "stream-a", 20_000)
        .expect("first terminal")
        .expect("first lease");
    let second = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("second admission released")
        .expect("second admission");
    assert!(!second.duplicate);
    assert!(runtime.has_routed_turn_lease("agent-a", "stream-b"));
    assert_eq!(runtime.in_flight_run_count("agent-a"), 1);

    runtime
        .settle_routed_turn("agent-a", "stream-b", 30_000)
        .expect("second terminal")
        .expect("second lease");
    assert_eq!(runtime.in_flight_run_count("agent-a"), 0);
    assert!(runtime.is_turn_dispatch_idle("agent-a"));
    join.join().expect("second admission thread");

    let _ = std::fs::remove_dir_all(root);
}


#[test]
fn group_member_routed_admission_refuses_missing_or_mismatched_selector_before_effects() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-routed-group-member-refusal-{}", uuid::Uuid::new_v4()
    ));
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let persisted = AtomicUsize::new(0);
    for (selector, reason) in [
        (None, "MissingSelector"),
        (Some("agent:other-member:group:room-a"), "SelectorAgentMismatch"),
    ] {
        let mut args = serde_json::json!({
            "agentId": "member-a",
            "prompt": "hidden group prompt",
            "clientNonce": "refused-group-nonce",
            "streamId": "refused-group-stream",
            "requestSource": "group-member",
            "groupMemberTurn": true,
        });
        if let Some(selector) = selector {
            args["sessionKey"] = selector.into();
        }
        let result = runtime.accept_routed_send(&args, |_| {
            persisted.fetch_add(1, Ordering::SeqCst);
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        });
        assert_eq!(result, Err(ProductionSendError::BadRequest(
            format!("invalid trusted session route: {reason}")
        )));
        assert_eq!(persisted.load(Ordering::SeqCst), 0);
        assert!(!runtime.has_routed_turn_lease("member-a", "refused-group-stream"));
        assert_eq!(runtime.in_flight_run_count("member-a"), 0);
        assert_eq!(runtime.queued_turn_count("member-a"), 0);
        assert!(runtime.is_turn_dispatch_idle("member-a"));
        assert!(runtime.active_turn_source("member-a").is_none());
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn group_member_routed_admission_projects_running_without_user_turn_badge() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-routed-group-member-{}",
        uuid::Uuid::new_v4()
    ));
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let args = serde_json::json!({
        "agentId": "member-a",
        "sessionKey": "agent:member-a:group:room-a",
        "prompt": "hidden group prompt",
        "clientNonce": "group-member-nonce",
        "streamId": "group-member-stream",
        "requestSource": "group-member",
        "groupMemberTurn": true,
    });
    runtime
        .accept_routed_send(&args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        })
        .expect("group member admission");

    let mut roster = serde_json::json!([{"id":"member-a"}]);
    runtime.decorate_agent_summaries(&mut roster);
    assert_eq!(roster[0]["isRunning"], true);
    assert_eq!(roster[0]["isRunningTurn"], false);
    assert_eq!(runtime.active_turn_source("member-a").as_deref(), Some("group-member"));

    runtime.begin_provider_run_with_kind("member-a", true);
    let mut provider_roster = serde_json::json!([{"id":"member-a"}]);
    runtime.decorate_agent_summaries(&mut provider_roster);
    assert_eq!(provider_roster[0]["isRunning"], true);
    assert_eq!(provider_roster[0]["isRunningTurn"], false);
    runtime.end_provider_run_with_kind("member-a", true);

    runtime
        .settle_routed_turn("member-a", "group-member-stream", 50_000)
        .expect("settlement")
        .expect("lease");
    assert_eq!(runtime.in_flight_run_count("member-a"), 0);
    let _ = std::fs::remove_dir_all(root);
}


#[test]
fn group_member_routed_admission_refuses_invalid_selectors_before_side_effects() {
    for (label, selector, reason) in [
        ("missing", None, "MissingSelector"),
        ("blank", Some("  "), "MissingSelector"),
        (
            "wrong-agent",
            Some("agent:member-b:group:room-a"),
            "SelectorAgentMismatch",
        ),
        (
            "harness",
            Some("agent:member-a:harness:private"),
            "Public(HarnessNamespaceReserved)",
        ),
    ] {
        let root = std::env::temp_dir().join(format!(
            "fabushi-routed-group-refusal-{label}-{}",
            uuid::Uuid::new_v4()
        ));
        let runtime = ProductionTranscriptRuntime::new(Some(&root));
        let persists = AtomicUsize::new(0);
        let lifecycle_events = Arc::new(AtomicUsize::new(0));
        let observed_lifecycle = Arc::clone(&lifecycle_events);
        runtime.set_agent_run_lifecycle_observer(Some(Arc::new(move |_, _| {
            observed_lifecycle.fetch_add(1, Ordering::SeqCst);
        })));
        let mut args = serde_json::json!({
            "agentId": "member-a",
            "prompt": "hidden group prompt",
            "clientNonce": "refused-group-nonce",
            "streamId": "refused-group-stream",
            "requestSource": "group-member",
            "groupMemberTurn": true,
        });
        if let Some(selector) = selector {
            args["sessionKey"] = serde_json::Value::String(selector.into());
        }

        let error = runtime
            .accept_routed_send(&args, |_| {
                persists.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ProductionSendError>(PersistedSendContext::default())
            })
            .expect_err("invalid group selector must refuse admission");
        assert_eq!(
            error,
            ProductionSendError::BadRequest(format!(
                "invalid trusted session route: {reason}"
            )),
            "{label} must fail at the existing selector boundary"
        );
        assert_eq!(persists.load(Ordering::SeqCst), 0, "{label}");
        assert_eq!(lifecycle_events.load(Ordering::SeqCst), 0, "{label}");
        assert_eq!(runtime.current_turn_epoch("member-a"), 0, "{label}");
        assert_eq!(runtime.in_flight_run_count("member-a"), 0, "{label}");
        assert_eq!(runtime.queued_turn_count("member-a"), 0, "{label}");
        assert!(runtime.is_turn_dispatch_idle("member-a"), "{label}");
        assert_eq!(runtime.active_turn_source("member-a"), None, "{label}");
        assert!(!runtime.has_routed_turn_lease("member-a", "refused-group-stream"));

        // The refusal must not consume the nonce/stream or manufacture a lease.
        // This is the explicit-room selector used by the shipping producer.
        args["sessionKey"] = serde_json::json!("agent:member-a:group:room-a");
        let accepted = runtime
            .accept_routed_send(&args, |_| {
                persists.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ProductionSendError>(PersistedSendContext::default())
            })
            .expect("the same send identity can be admitted with its valid selector");
        assert!(!accepted.duplicate, "{label}");
        assert_eq!(persists.load(Ordering::SeqCst), 1, "{label}");
        assert_eq!(lifecycle_events.load(Ordering::SeqCst), 1, "{label}");
        assert_eq!(runtime.current_turn_epoch("member-a"), 1, "{label}");
        assert_eq!(runtime.in_flight_run_count("member-a"), 1, "{label}");
        assert!(runtime.has_routed_turn_lease("member-a", "refused-group-stream"));
        assert_eq!(runtime.active_turn_source("member-a").as_deref(), Some("group-member"));

        runtime
            .settle_routed_turn("member-a", "refused-group-stream", 50_000)
            .expect("valid group settlement")
            .expect("valid group lease");
        assert_eq!(runtime.in_flight_run_count("member-a"), 0, "{label}");
        assert_eq!(runtime.queued_turn_count("member-a"), 0, "{label}");
        assert!(runtime.is_turn_dispatch_idle("member-a"), "{label}");
        assert!(!runtime.has_routed_turn_lease("member-a", "refused-group-stream"));
        assert_eq!(lifecycle_events.load(Ordering::SeqCst), 2, "{label}");
        let _ = std::fs::remove_dir_all(root);
    }
}


#[test]
fn upgrade_resume_admission_uses_the_background_exclusive_lane() {
    use mahayana_host_runtime::extensions::transcript::production_runtime::classify_send_dispatch;
    use mahayana_host_runtime::extensions::transcript::run_scheduler::RunLane;

    let (lane, source) = classify_send_dispatch(&serde_json::json!({
        "requestSource": "upgrade-resume"
    }))
    .expect("upgrade resume dispatch");
    assert_eq!(lane, RunLane::Background);
    assert_eq!(source, "upgrade-resume");
}

struct OriginalAdmissionFixture {
    root: std::path::PathBuf,
    sessions: Arc<mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers>,
    runtime: Arc<ProductionTranscriptRuntime>,
    agent_id: String,
}

impl OriginalAdmissionFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-original-admission-{label}-{}", uuid::Uuid::new_v4(),
        ));
        std::fs::create_dir_all(&root).unwrap();
        let sessions = Arc::new(
            mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers::with_agents_root(
                root.join("agents"), 500,
            ),
        );
        let agent_id = sessions.materialize_new_session(None, "user", None).unwrap().id;
        Self {
            runtime: Arc::new(ProductionTranscriptRuntime::new(Some(&root))),
            root, sessions, agent_id,
        }
    }

    fn args(&self, stream: &str) -> serde_json::Value {
        serde_json::json!({
            "agentId": self.agent_id,
            "sessionKey": format!("agent:{}:main", self.agent_id),
            "streamId": stream, "clientNonce": format!("nonce-{stream}"),
            "prompt": "original accepted user input",
        })
    }

    fn persist(
        &self,
        args: &serde_json::Value,
        accepted: &serde_json::Value,
    ) -> Result<PersistedSendContext, ProductionSendError> {
        persist_original_admission(&self.sessions, args, accepted)
    }

    fn close(self) {
        self.sessions.shutdown();
        let root = self.root.clone();
        drop(self);
        std::fs::remove_dir_all(root).unwrap();
    }
}

fn persist_original_admission(
    sessions: &Arc<mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers>,
    args: &serde_json::Value,
    accepted: &serde_json::Value,
) -> Result<PersistedSendContext, ProductionSendError> {
    mahayana_host_runtime::extensions::session::gateway::persist_accepted_send_prompt_context(
        sessions, args, accepted,
    ).map_err(|error| ProductionSendError::Internal(format!("{error:?}")))
}

fn retained_original_marker(agent_id: &str) -> mahayana_host_runtime::extensions::transcript::sand_upgrade_resume_store::UpgradeResumeMarker {
    mahayana_host_runtime::extensions::transcript::sand_upgrade_resume_store::UpgradeResumeMarker {
        agent_id: agent_id.into(), marked_at_ms: 1.0, source: Some("turn".into()),
        automation_id: None, automation_run_id: None, original_work: None,
    }
}

#[test]
fn original_turn_binding_preserves_accepted_raw_digest_and_durable_user_identity() {
    use mahayana_host_runtime::extensions::transcript::prompt_acceptance_ledger::{
        SendInput, send_input_digest,
    };

    let fixture = OriginalAdmissionFixture::new("raw-input");
    let mut args = fixture.args("raw-input-stream");
    args["prompt"] = "  accepted raw input\n\t".into();
    args["richText"] = "{\"type\":\"doc\",\"content\":[]}".into();
    let accepted = fixture.runtime.accept_routed_send(&args, |accepted| {
        fixture.persist(&args, accepted)
    }).unwrap();
    assert!(accepted.context.accepted_durably);
    assert_eq!(accepted.context.accepted_echoes.len(), 1);
    let user_id = accepted.context.user_message_id.as_deref().unwrap();
    let entries = fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap();
    let user = entries.iter().find(|entry| entry["id"] == user_id).unwrap();
    assert_eq!(user["content"], "accepted raw input");
    let admission = fixture.sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
    let bound = fixture.runtime.bind_original_turn_model(
        &fixture.agent_id, "raw-input-stream", &admission, "resolved-native-model",
    ).unwrap().unwrap();
    let input = SendInput {
        agent_id: Some(fixture.agent_id.clone()),
        prompt: args["prompt"].as_str().unwrap().into(),
        rich_text: args["richText"].as_str().map(str::to_string),
        reply_to_id: None, is_fork: false, attachment_paths: Vec::new(), attachment_names: Vec::new(),
    };
    assert_eq!(bound.input_digest, send_input_digest(&input));
    assert_ne!(bound.input_digest, send_input_digest(&SendInput {
        prompt: input.prompt.trim().into(), ..input
    }), "the recovery reference must not reconstruct a digest from the trimmed durable prompt");
    assert_eq!(bound.user_message_id, user_id);
    assert_eq!(bound.echo_entry_id, accepted.context.echo_entry_id.unwrap());
    assert_eq!(bound.client_nonce.as_deref(), Some("nonce-raw-input-stream"));
    assert_eq!(bound.request_source, None);
    assert_eq!(bound.model_id, "resolved-native-model");
    assert_eq!(bound.original_lifecycle_revision, admission.lifecycle_revision);
    assert_eq!(fixture.runtime.bind_original_turn_model(
        &fixture.agent_id, "raw-input-stream", &admission, "resolved-native-model",
    ).unwrap(), Some(bound.clone()));
    assert!(matches!(fixture.runtime.bind_original_turn_model(
        &fixture.agent_id, "raw-input-stream", &admission, "today-different-model",
    ), Err(ProductionSendError::Conflict(_))));
    assert!(matches!(fixture.runtime.bind_original_turn_model(
        "foreign-agent", "raw-input-stream", &admission, "resolved-native-model",
    ), Err(ProductionSendError::Conflict(_))));
    fixture.runtime.settle_routed_turn(&fixture.agent_id, "raw-input-stream", 10_000).unwrap().unwrap();
    assert!(fixture.runtime.bind_original_turn_model(
        &fixture.agent_id, "raw-input-stream", &admission, "resolved-native-model",
    ).is_err(), "a retired ticket cannot acquire a new original-work binding");
    let mut no_nonce = fixture.args("without-original-nonce");
    no_nonce.as_object_mut().unwrap().remove("clientNonce");
    no_nonce["requestSource"] = "turn".into();
    fixture.runtime.accept_routed_send(&no_nonce, |accepted| fixture.persist(&no_nonce, accepted)).unwrap();
    let admission = fixture.sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
    let without_nonce = fixture.runtime.bind_original_turn_model(
        &fixture.agent_id, "without-original-nonce", &admission, "resolved-native-model",
    ).unwrap().unwrap();
    assert_eq!(without_nonce.client_nonce, None, "absence cannot become a newly minted recovery nonce");
    assert_eq!(without_nonce.request_source.as_deref(), Some("turn"));
    assert_ne!(without_nonce.user_message_id, bound.user_message_id);
    fixture.runtime.settle_routed_turn(&fixture.agent_id, "without-original-nonce", 20_000).unwrap().unwrap();
    fixture.close();
}

#[test]
fn original_turn_binding_excludes_special_raw_producers_even_when_dispatch_maps_to_turn() {
    use mahayana_host_runtime::extensions::transcript::production_runtime::classify_send_dispatch;

    for (name, patch) in [
        ("connector", serde_json::json!({"requestSource":"connector"})),
        ("hidden", serde_json::json!({"hidden":true})),
        ("fork", serde_json::json!({"isFork":true})),
        ("child", serde_json::json!({"parentAgentId":"parent"})),
        ("group-context", serde_json::json!({"groupContext":{}})),
        ("no-visible-user", serde_json::json!({"appendUserMessage":false})),
        ("skip-ack", serde_json::json!({"skipAckObligation":true})),
        ("upgrade-flag", serde_json::json!({"upgradeResume":true})),
        ("ack-flag", serde_json::json!({"ackRedrive":true})),
    ] {
        let fixture = OriginalAdmissionFixture::new(name);
        let mut args = fixture.args(name);
        args.as_object_mut().unwrap().extend(patch.as_object().unwrap().clone());
        assert_eq!(classify_send_dispatch(&args).unwrap().1, "turn", "{name}");
        let accepted = fixture.runtime.accept_routed_send(&args, |accepted| {
            fixture.persist(&args, accepted)
        }).unwrap();
        assert!(!accepted.duplicate);
        let admission = fixture.sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
        assert_eq!(fixture.runtime.bind_original_turn_model(
            &fixture.agent_id, name, &admission, "native-model",
        ).unwrap(), None, "raw producer {name} cannot inherit ordinary-user eligibility from lane mapping");
        fixture.runtime.settle_routed_turn(&fixture.agent_id, name, 10_000).unwrap().unwrap();
        fixture.close();
    }
}

#[test]
fn original_turn_quiesce_copies_current_binding_and_late_model_binding_cannot_backfill() {
    let fixture = OriginalAdmissionFixture::new("quiesce");
    let args = fixture.args("bound-stream");
    fixture.runtime.accept_routed_send(&args, |accepted| fixture.persist(&args, accepted)).unwrap();
    let admission = fixture.sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
    let original = fixture.runtime.bind_original_turn_model(
        &fixture.agent_id, "bound-stream", &admission, "native-model",
    ).unwrap().unwrap();
    let unbound = fixture.sessions.materialize_new_session(None, "user", None).unwrap().id;
    let unbound_args = serde_json::json!({
        "agentId":unbound, "streamId":"not-yet-model-bound", "prompt":"another ordinary user",
    });
    fixture.runtime.accept_routed_send(&unbound_args, |accepted| fixture.persist(&unbound_args, accepted)).unwrap();
    let unbound_admission = fixture.sessions.read_session_work_admission(&unbound).unwrap().unwrap();
    fixture.runtime.quiesce_for_upgrade().unwrap();
    let store = fixture.runtime.upgrade_resume_store().unwrap();
    let markers = store.list_pending_checked().unwrap();
    assert_eq!(markers.len(), 2);
    assert_eq!(markers.iter().find(|marker| marker.agent_id == fixture.agent_id).unwrap().original_work, Some(original));
    assert_eq!(markers.iter().find(|marker| marker.agent_id == unbound).unwrap().original_work, None);
    let frozen = std::fs::read(store.file_path()).unwrap();
    assert_eq!(fixture.runtime.bind_original_turn_model(
        &unbound, "not-yet-model-bound", &unbound_admission, "native-model",
    ).unwrap(), None);
    assert_eq!(std::fs::read(store.file_path()).unwrap(), frozen);
    fixture.runtime.settle_routed_turn(&fixture.agent_id, "bound-stream", 10_000).unwrap().unwrap();
    fixture.runtime.settle_routed_turn(&unbound, "not-yet-model-bound", 10_000).unwrap().unwrap();
    fixture.close();
}

#[test]
fn retained_original_marker_blocks_both_fresh_automatic_admission_paths_but_allows_users() {
    for routed in [true, false] {
        let fixture = OriginalAdmissionFixture::new(if routed { "accept-gate" } else { "execute-gate" });
        let store = fixture.runtime.upgrade_resume_store().unwrap();
        store.mark_pending_checked(retained_original_marker(&fixture.agent_id)).unwrap();
        let marker_bytes = std::fs::read(store.file_path()).unwrap();
        let persists = AtomicUsize::new(0);
        let dispatches = AtomicUsize::new(0);
        for (name, patch) in [
            ("handoff-ack", serde_json::json!({"requestSource":"handoff-resume","ackRedrive":true})),
            ("upgrade-source", serde_json::json!({"requestSource":"upgrade-resume"})),
            ("upgrade-flag", serde_json::json!({"requestSource":"turn","upgradeResume":true})),
            ("ack-source", serde_json::json!({"requestSource":"ack-redrive"})),
        ] {
            let mut args = fixture.args(name);
            args.as_object_mut().unwrap().extend(patch.as_object().unwrap().clone());
            let persist = |accepted: &serde_json::Value| {
                persists.fetch_add(1, Ordering::SeqCst);
                fixture.persist(&args, accepted)
            };
            let result = if routed {
                fixture.runtime.accept_routed_send(&args, persist).map(|_| ())
            } else {
                fixture.runtime.execute_send(&args, || {
                    dispatches.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::json!({"accepted":true}))
                }, persist).map(|_| ())
            };
            assert!(matches!(result, Err(ProductionSendError::Rejected(_))), "{name}/{routed}");
        }
        assert_eq!(persists.load(Ordering::SeqCst), 0);
        assert_eq!(dispatches.load(Ordering::SeqCst), 0);
        assert_eq!(fixture.runtime.current_turn_epoch(&fixture.agent_id), 0);
        assert_eq!(fixture.runtime.queued_turn_count(&fixture.agent_id), 0);
        assert!(fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap().is_empty());
        assert_eq!(std::fs::read(store.file_path()).unwrap(), marker_bytes);

        // A retained automatic continuation must not block a deliberate new
        // user direction, nor steal the result of its already accepted nonce.
        let args = fixture.args("ordinary-user");
        let count_before = fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap().len();
        if routed {
            let first = fixture.runtime.accept_routed_send(&args, |accepted| fixture.persist(&args, accepted)).unwrap();
            assert!(!first.duplicate);
            let mut replay = args.clone();
            replay["ackRedrive"] = true.into();
            let duplicate = fixture.runtime.accept_routed_send(&replay, |accepted| fixture.persist(&replay, accepted)).unwrap();
            assert!(duplicate.duplicate);
            assert_eq!(duplicate.context.user_message_id, first.context.user_message_id);
            fixture.runtime.settle_routed_turn(&fixture.agent_id, "ordinary-user", 10_000).unwrap().unwrap();
        } else {
            fixture.runtime.execute_send(&args, || {
                dispatches.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::json!({"accepted":true,"result":"owned result"}))
            }, |accepted| fixture.persist(&args, accepted)).unwrap();
            let mut replay = args.clone();
            replay["ackRedrive"] = true.into();
            let duplicate = fixture.runtime.execute_send(&replay,
                || panic!("accepted nonce cannot dispatch twice"),
                |_| -> Result<PersistedSendContext, ProductionSendError> { panic!("accepted nonce cannot append twice") },
            ).unwrap();
            assert_eq!(duplicate["result"], "owned result");
            assert_eq!(dispatches.load(Ordering::SeqCst), 1);
        }
        assert_eq!(fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap().len(), count_before + 1);
        assert_eq!(std::fs::read(store.file_path()).unwrap(), marker_bytes);
        fixture.close();
    }
}

#[test]
fn retained_original_marker_preserves_queued_and_coalesced_automatic_nonce_ownership() {
    use std::sync::mpsc;
    use std::time::Duration;

    let fixture = OriginalAdmissionFixture::new("queued-owned");
    let first = fixture.args("foreground");
    fixture.runtime.accept_routed_send(&first, |accepted| fixture.persist(&first, accepted)).unwrap();
    let mut queued = fixture.args("already-queued-ack");
    queued["requestSource"] = "handoff-resume".into();
    queued["ackRedrive"] = true.into();
    let (queued_tx, queued_rx) = mpsc::channel();
    let runtime = Arc::clone(&fixture.runtime);
    let sessions = Arc::clone(&fixture.sessions);
    let original_args = queued.clone();
    let (original_tx, original_rx) = mpsc::channel();
    let original = std::thread::spawn(move || {
        original_tx.send(runtime.accept_routed_send_with_queue_observers(
            &original_args,
            |accepted| persist_original_admission(&sessions, &original_args, accepted),
            |_| false,
            |event| { queued_tx.send(event.clone()).unwrap(); },
            |_| {},
        )).unwrap();
    });
    queued_rx.recv_timeout(Duration::from_secs(5)).expect("actual queue owner admitted the ACK nonce before retention");
    assert_eq!(fixture.runtime.queued_turn_count(&fixture.agent_id), 1);
    let store = fixture.runtime.upgrade_resume_store().unwrap();
    store.mark_pending_checked(retained_original_marker(&fixture.agent_id)).unwrap();
    let frozen = std::fs::read(store.file_path()).unwrap();
    let runtime = Arc::clone(&fixture.runtime);
    let sessions = Arc::clone(&fixture.sessions);
    let (duplicate_tx, duplicate_rx) = mpsc::channel();
    let duplicate = std::thread::spawn(move || {
        duplicate_tx.send(runtime.accept_routed_send(&queued, |accepted| {
            persist_original_admission(&sessions, &queued, accepted)
        })).unwrap();
    });
    assert!(matches!(duplicate_rx.recv_timeout(Duration::from_millis(50)), Err(mpsc::RecvTimeoutError::Timeout)),
        "the duplicate waits on the existing queued nonce instead of receiving a fresh marker refusal");
    fixture.runtime.settle_routed_turn(&fixture.agent_id, "foreground", 10_000).unwrap().unwrap();
    let accepted = original_rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    original.join().unwrap();
    let replay = duplicate_rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    duplicate.join().unwrap();
    assert!(!accepted.duplicate);
    assert!(replay.duplicate);
    assert_eq!(replay.context.user_message_id, accepted.context.user_message_id);
    assert_eq!(fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap().len(), 2);
    assert!(fixture.runtime.has_routed_turn_lease(&fixture.agent_id, "already-queued-ack"));
    assert_eq!(std::fs::read(store.file_path()).unwrap(), frozen);
    fixture.runtime.settle_routed_turn(&fixture.agent_id, "already-queued-ack", 20_000).unwrap().unwrap();
    fixture.close();
}
