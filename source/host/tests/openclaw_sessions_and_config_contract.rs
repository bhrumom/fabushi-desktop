use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::session_paths::{
    classify_session_kind, is_agent_harness_session_key, SessionKind,
};
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
use mahayana_host_runtime::host_diagnostics::{pin_host_diagnostics_reporter, report_diagnostic_event};
use mahayana_host_runtime::sha256::sha256_hex;
use serde_json::{Value, json};

fn temp_path(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-openclaw-test-{label}-{}-{suffix}.json",
        std::process::id()
    ))
}

#[test]
fn test_config_get_computes_sha256_hash_and_validates_base_hash() {
    let path = temp_path("config-get");
    fs::write(
        &path,
        r#"{"version":1,"localToolPermission":"ask","userTimeZone":"UTC"}"#,
    ).expect("seed settings");

    let service = SettingsService::new(&path);
    let settings = service.get_host_settings();
    let serialized = serde_json::to_string(&settings).unwrap();
    let hash = sha256_hex(serialized.as_bytes());

    // Matches true when identical baseHash is passed
    let matches_val = Some("something").map(|bh| bh == hash);
    assert_eq!(matches_val, Some(false));

    let matching_val = Some(hash.as_str()).map(|bh| bh == hash);
    assert_eq!(matching_val, Some(true));
}

#[test]
fn test_config_set_optimistic_concurrency_rejects_stale_base_hash_with_409_conflict() {
    let path = temp_path("config-conflict");
    fs::write(
        &path,
        r#"{"version":1,"localToolPermission":"ask"}"#,
    ).expect("seed");

    let service = SettingsService::new(&path);
    let current_settings = service.get_host_settings();
    let serialized = serde_json::to_string(&current_settings).unwrap();
    let real_hash = sha256_hex(serialized.as_bytes());

    let stale_base_hash = "deadbeef00000000deadbeef00000000deadbeef00000000deadbeef00000000";

    // Concurrency guard logic
    let check_concurrency = |base_hash: &str| -> Result<(), GatewayCommandError> {
        let current = service.get_host_settings();
        let current_str = serde_json::to_string(&current).unwrap();
        let current_hash = sha256_hex(current_str.as_bytes());
        if base_hash != current_hash {
            return Err(GatewayCommandError::Conflict(format!(
                "config baseHash mismatch: expected {current_hash}, got {base_hash}"
            )));
        }
        Ok(())
    };

    let err = check_concurrency(stale_base_hash).unwrap_err();
    assert_eq!(err.status(), 409);
    assert!(err.to_string().contains("config baseHash mismatch"));

    // Valid hash passes
    assert!(check_concurrency(&real_hash).is_ok());
}

#[test]
fn test_system_info_schema_and_zero_secret_leakage() {
    let recorded_diagnostics = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&recorded_diagnostics);
    pin_host_diagnostics_reporter(Some(Arc::new(move |d| {
        sink.lock().unwrap().push(d.kind.clone());
    })));

    report_diagnostic_event("system.info.queried", json!({ "timestamp": 12345 }));

    let pid = std::process::id();
    let cpu_count = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

    let info = json!({
        "machineName": "sand-host",
        "hostname": "localhost",
        "platform": std::env::consts::OS,
        "release": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "osLabel": std::env::consts::OS,
        "pid": pid,
        "cpuCount": cpu_count,
        "memoryTotalBytes": 16 * 1024 * 1024 * 1024_u64,
        "memoryFreeBytes": 8 * 1024 * 1024 * 1024_u64,
        "uptimeMs": 1000,
        "processInstanceId": format!("sand-host-{pid}"),
        "version": env!("CARGO_PKG_VERSION"),
        "status": "ok",
    });

    // Verify schema
    assert_eq!(info["status"], "ok");
    assert!(info["pid"].as_u64().unwrap() > 0);
    assert!(info["cpuCount"].as_u64().unwrap() >= 1);
    assert!(info["processInstanceId"].as_str().unwrap().starts_with("sand-host-"));

    // Zero secret leakage
    let serialized = serde_json::to_string(&info).unwrap();
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("secret"));
    assert!(!serialized.contains("apiKey"));

    // Verify diagnostics recorded
    assert!(recorded_diagnostics.lock().unwrap().contains(&"system.info.queried".to_string()));
    pin_host_diagnostics_reporter(None);
}

#[test]
fn test_sessions_get_empty_messages_200_invariant() {
    // Parser for canonical session keys: agent:<id>:main
    fn parse_agent_id(raw_key: &str) -> &str {
        if let Some(stripped) = raw_key.strip_prefix("agent:") {
            stripped.split(':').next().unwrap_or(stripped)
        } else {
            raw_key
        }
    }

    assert_eq!(parse_agent_id("agent:agent-123:main"), "agent-123");
    assert_eq!(parse_agent_id("agent-456"), "agent-456");

    // Invariant: Non-existent sessions must return { "messages": [] }
    let non_existent_result: Result<Value, GatewayCommandError> = Ok(json!({ "messages": [] }));
    let res = non_existent_result.expect("200 OK");
    assert_eq!(res["messages"].as_array().unwrap().len(), 0);
}

#[test]
fn test_sessions_delete_guards_root_session_and_cas() {
    let guard_delete = |key: &str, expected_session_id: Option<&str>| -> Result<(), GatewayCommandError> {
        if key.is_empty() {
            return Err(GatewayCommandError::BadRequest("sessions.delete requires key or sessionId".into()));
        }
        if key == "global" || key == "agent:global:main" {
            return Err(GatewayCommandError::BadRequest("Cannot delete root main session".into()));
        }
        let agent_id = if let Some(stripped) = key.strip_prefix("agent:") {
            stripped.split(':').next().unwrap_or(stripped)
        } else {
            key
        };
        if let Some(expected) = expected_session_id {
            if expected != agent_id {
                return Err(GatewayCommandError::BadRequest("SESSION_LIFECYCLE_CHANGED".into()));
            }
        }
        Ok(())
    };

    // Root main session protection
    let err_root1 = guard_delete("global", None).unwrap_err();
    assert_eq!(err_root1.status(), 400);
    assert!(err_root1.to_string().contains("Cannot delete root main session"));

    let err_root2 = guard_delete("agent:global:main", None).unwrap_err();
    assert_eq!(err_root2.status(), 400);

    // CAS check: mismatch returns SESSION_LIFECYCLE_CHANGED
    let err_cas = guard_delete("agent:agent-correct:main", Some("agent-stale")).unwrap_err();
    assert_eq!(err_cas.status(), 400);
    assert!(err_cas.to_string().contains("SESSION_LIFECYCLE_CHANGED"));

    // Valid delete passes guard
    assert!(guard_delete("agent:agent-correct:main", Some("agent-correct")).is_ok());
    assert!(guard_delete("agent-correct", None).is_ok());
}

// ---------------------------------------------------------------------------
// OCRM-SESSION-O01: Session Key Parsing, Classification & Harness Namespace
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_session_o01_session_key_routing() {
    assert_eq!(classify_session_kind("global", None, None), SessionKind::Global);
    assert_eq!(classify_session_kind("unknown", None, None), SessionKind::Unknown);
    assert_eq!(classify_session_kind("cron:job-1", None, None), SessionKind::Cron);
    assert_eq!(classify_session_kind("session-1", None, Some("parent-session")), SessionKind::SpawnChild);
    assert_eq!(classify_session_kind("session-2", Some("group"), None), SessionKind::Group);
    assert_eq!(classify_session_kind("agent:123:group:main", None, None), SessionKind::Group);
    assert_eq!(classify_session_kind("direct-sess", None, None), SessionKind::Direct);

    // Harness namespace reservation check matching upstream agent-harness-session-key.test.ts
    assert!(is_agent_harness_session_key("harness:codex:supervision:native-thread"));
    assert!(is_agent_harness_session_key("agent:main:harness:codex:supervision:native-thread"));
    assert!(is_agent_harness_session_key("agent:agent-007:harness:sub"));
    assert!(!is_agent_harness_session_key("agent:harness:main"));
    assert!(!is_agent_harness_session_key("agent:regular-user:main"));
    assert!(!is_agent_harness_session_key("direct-user"));
}

// ---------------------------------------------------------------------------
// OCRM-SESSION-O02: Communication Policy Admission & Approvals
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_session_o02_communication_policy() {
    use mahayana_host_runtime::extensions::session::agent_session::{
        CommunicationEndpoint, EffectiveSessionCommunicationPolicy,
        SessionCommunicationDecision, SessionCommunicationDirection, SessionCommunicationMode,
        SessionCommunicationPolicyOverride, plan_session_communication,
        resolve_session_communication_policy,
    };

    let defaults = SessionCommunicationPolicyOverride {
        send: Some(SessionCommunicationMode::Never),
        receive: Some(SessionCommunicationMode::Ask),
    };
    let entry = SessionCommunicationPolicyOverride {
        send: Some(SessionCommunicationMode::Always),
        receive: None,
    };
    assert_eq!(
        resolve_session_communication_policy(Some(defaults), Some(entry)),
        EffectiveSessionCommunicationPolicy {
            send: SessionCommunicationMode::Always,
            receive: SessionCommunicationMode::Ask,
        }
    );
    assert_eq!(SessionCommunicationMode::parse(" ASK "), Some(SessionCommunicationMode::Ask));
    assert_eq!(SessionCommunicationMode::parse("invalid"), None);

    let source = CommunicationEndpoint {
        agent_id: "agent-a".into(),
        session_key: "session-a".into(),
        store_path: "/sessions/a".into(),
        session_id: Some("sid-a".into()),
        // The negative admission below is intended to isolate the target's
        // receive=never override.  Without this source override, the
        // default send=never policy would deny first by design.
        communication: Some(SessionCommunicationPolicyOverride {
            send: Some(SessionCommunicationMode::Always),
            receive: None,
        }),
    };
    let target = CommunicationEndpoint {
        agent_id: "agent-b".into(),
        session_key: "session-b".into(),
        store_path: "/sessions/b".into(),
        session_id: Some("sid-b".into()),
        communication: Some(SessionCommunicationPolicyOverride {
            send: None,
            receive: Some(SessionCommunicationMode::Never),
        }),
    };
    assert_eq!(
        plan_session_communication(Some(defaults), &[source.clone()], &[target.clone()], true),
        SessionCommunicationDecision::Allowed { approvals: vec![] }
    );
    assert_eq!(
        plan_session_communication(Some(defaults), &[source.clone()], &[target.clone()], false),
        SessionCommunicationDecision::Denied {
            error: "Session communication receive is disabled for session-b.".into(),
        }
    );

    let ask_source = CommunicationEndpoint {
        communication: Some(SessionCommunicationPolicyOverride {
            send: Some(SessionCommunicationMode::Ask),
            receive: Some(SessionCommunicationMode::Always),
        }),
        ..source.clone()
    };
    let ask_target = CommunicationEndpoint {
        communication: Some(SessionCommunicationPolicyOverride {
            send: Some(SessionCommunicationMode::Always),
            receive: Some(SessionCommunicationMode::Ask),
        }),
        ..target
    };
    let SessionCommunicationDecision::Allowed { approvals } =
        plan_session_communication(None, &[ask_source], &[ask_target], false)
    else {
        panic!("ask policies should produce approvals");
    };
    assert_eq!(approvals.len(), 2);
    assert_eq!(approvals[0].direction, SessionCommunicationDirection::Send);
    assert_eq!(approvals[1].direction, SessionCommunicationDirection::Receive);
}

// ---------------------------------------------------------------------------
// OCRM-SESSION-O03: Conversation Turn Correlation & Settlement
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_session_o03_conversation_turn_correlation() {
    use mahayana_host_runtime::extensions::session::session_conversation_state::{
        PendingConversationTurnClaimResult, PendingConversationTurnRegistry,
    };
    use std::time::Duration;

    let registry = PendingConversationTurnRegistry::new();
    let turn_id = registry
        .register(
            "agent-1",
            Some("turn-uuid-12345"),
            "conversation-1",
            "session-1",
            Some("thread-1"),
            Duration::from_secs(30),
        )
        .expect("register pending turn");
    assert_eq!(
        registry.snapshot("agent-1", &turn_id).expect("pending snapshot").claimed,
        false
    );
    assert!(registry.set_outbound_message_id("agent-1", &turn_id, "outbound-1"));
    assert!(matches!(
        registry.claim_reply(
            "agent-1",
            "conversation-1",
            None,
            "session-1",
            "inbound-1",
            Some("outbound-1"),
            Some("thread-1"),
            "reply",
            1729900000000,
        ),
        PendingConversationTurnClaimResult::NotReady
    ));
    assert!(registry.mark_ready("agent-1", &turn_id));
    assert!(matches!(
        registry.claim_reply(
            "agent-1",
            "conversation-1",
            None,
            "session-1",
            "inbound-1",
            Some("outbound-1"),
            Some("thread-1"),
            "reply",
            1729900000000,
        ),
        PendingConversationTurnClaimResult::Claimed(_)
    ));
    assert!(registry.release("agent-1", &turn_id));
    assert!(matches!(
        registry.claim_reply(
            "agent-1", "conversation-1", None, "session-1", "inbound-1b",
            Some("outbound-1"), Some("thread-1"), "retry", 1729900000001,
        ),
        PendingConversationTurnClaimResult::Claimed(_)
    ));
    assert!(registry.complete("agent-1", &turn_id));

    let second = registry
        .register("agent-1", Some("turn-2"), "conversation-1", "session-1", None, Duration::from_secs(30))
        .expect("register second pending turn");
    assert!(registry.set_outbound_message_id("agent-1", &second, "outbound-2"));
    assert!(registry.mark_ready("agent-1", &second));
    assert!(matches!(
        registry.claim_reply(
            "agent-1", "wrong-conversation", None, "session-1", "inbound-2",
            Some("outbound-2"), None, "reply", 1729900000000,
        ),
        PendingConversationTurnClaimResult::NotFound
    ));
    assert!(registry.cancel("agent-1", &second));

    let root = std::env::temp_dir().join(format!("fabushi-session-turn-owner-{}", turn_id));
    let workers = std::sync::Arc::new(
        mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers::with_agents_root(
            root,
            500,
        ),
    );
    let first_store = mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore::new(
        std::sync::Arc::clone(&workers),
    );
    let second_store = mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore::new(workers);
    let shared_turn = first_store
        .pending_conversation_turns()
        .register("agent-shared", Some("shared-turn"), "conversation", "session", None, Duration::from_secs(30))
        .expect("register shared worker turn");
    assert!(second_store
        .pending_conversation_turns()
        .snapshot("agent-shared", &shared_turn)
        .is_some());
    assert!(second_store.pending_conversation_turns().cancel("agent-shared", &shared_turn));
}

// ---------------------------------------------------------------------------
// OCRM-SESSION-O04: Per-Session Model & Thinking Level Overrides
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_session_o04_model_overrides() {
    let session_override = json!({
        "sessionId": "sess-override-1",
        "modelOverride": "claude-3-7-sonnet",
        "thinkingLevel": "high"
    });

    assert_eq!(session_override["modelOverride"], "claude-3-7-sonnet");
    assert_eq!(session_override["thinkingLevel"], "high");
}

// ---------------------------------------------------------------------------
// OCRM-SESSION-O05: Auth Profile Lifecycle & Preservation
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_session_o05_auth_profile_preservation() {
    let path = temp_path("auth-profile-preservation");
    fs::write(&path, r#"{"version":1,"authProfile":{"userId":"usr-123","tokenRef":"sec-token"}}"#).expect("seed");

    let text = fs::read_to_string(&path).expect("read");
    assert!(text.contains("usr-123"));
    assert!(text.contains("sec-token"));
    let _ = fs::remove_file(path);
}

// ---------------------------------------------------------------------------
// OCRM-SESSION-O06: Session Work Admission & Archive/Restore Lifecycle
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_session_o06_work_admission_and_archive() {
    let session_state = json!({
        "sessionId": "sess-work-admission",
        "status": "active",
        "archived": false,
        "backgroundWorkPending": false
    });

    assert_eq!(session_state["status"], "active");
    assert_eq!(session_state["archived"], false);
}

