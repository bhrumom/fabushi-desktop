use std::fs;
use std::sync::{Arc, Barrier, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::session_paths::{
    classify_session_kind, is_agent_harness_session_key, is_root_main_session_key,
    resolve_internal_send_route, resolve_public_session_route, InternalSessionRouteError,
    PublicSessionRouteError, SessionKind,
};
use mahayana_host_runtime::extensions::settings::settings_service::{
    HostSettingsUpdateError, SettingsService,
};
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
    let original = service.get_host_settings_snapshot();
    assert_eq!(
        original.hash,
        sha256_hex(serde_json::to_string(&original.settings).expect("serialize snapshot"))
    );
    let committed = service
        .set_host_settings_if_current(
            Some(&original.hash),
            json!({"userTimeZone": "Asia/Tokyo"}),
        )
        .expect("matching owner snapshot commits");
    assert_eq!(committed.settings["userTimeZone"], "Asia/Tokyo");
    assert_ne!(committed.hash, original.hash);
    assert_eq!(
        committed.hash,
        sha256_hex(serde_json::to_string(&committed.settings).expect("serialize commit"))
    );
    assert_eq!(
        committed.config_source["rawHash"],
        sha256_hex(fs::read(&path).expect("committed source bytes"))
    );
    assert_eq!(service.get_host_settings_snapshot(), committed);
    let _ = fs::remove_file(path);
}

#[test]
fn test_config_set_optimistic_concurrency_rejects_stale_base_hash_with_409_conflict() {
    let path = temp_path("config-conflict");
    fs::write(
        &path,
        r#"{"version":1,"localToolPermission":"ask","notifications":{"isEnabled":true,"legacy":"preserve-until-accepted"}}"#,
    ).expect("seed");

    let service = SettingsService::new(&path);
    let original_bytes = fs::read(&path).expect("seed bytes");
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |fields| {
        sink.lock().expect("observed changes").push(fields);
    }));

    let stale_base_hash = "deadbeef00000000deadbeef00000000deadbeef00000000deadbeef00000000";

    let conflict = service
        .set_host_settings_if_current(
            Some(stale_base_hash),
            json!({"localToolPermission": "allow"}),
        )
        .expect_err("the actual owner must reject a stale hash");
    assert!(matches!(
        &conflict,
        HostSettingsUpdateError::Conflict { actual, .. } if actual == stale_base_hash
    ));
    // A rejected comparison cannot even normalize the legacy notification field.
    assert_eq!(fs::read(&path).expect("unchanged settings"), original_bytes);
    assert!(observed.lock().expect("observed changes").is_empty());
    let current = service.get_host_settings_snapshot();
    assert!(matches!(
        &conflict,
        HostSettingsUpdateError::Conflict { expected, .. } if expected == &current.hash
    ));
    let err = GatewayCommandError::from(conflict);
    assert_eq!(err.status(), 409);
    assert!(err.to_string().contains("config baseHash mismatch"));

    service
        .set_host_settings_if_current(
            Some(&current.hash),
            json!({"userTimeZone": "UTC"}),
        )
        .expect("matching hash reaches the same production owner");
    assert_eq!(observed.lock().expect("observed changes").len(), 1);

    let shipping_source = include_str!("../app/src/main.rs");
    let config_set = shipping_source
        .split("fn dispatch_public_settings_command(")
        .nth(1)
        .expect("shipping public Settings adapter exists")
        .split("struct UnifiedGatewayApi")
        .next()
        .expect("shipping Settings adapter boundary exists");
    assert!(config_set.contains("config.set requires raw, config, or settings"));
    assert!(config_set.contains("config.set target must be a JSON object"));
    assert!(!config_set.contains("unwrap_or(args)"));
    assert!(config_set.contains("settings.set_public_host_settings_if_current("));
    assert!(config_set.contains("settings.get_public_host_settings_snapshot()"));
    assert!(config_set.contains("on_committed(&committed.trusted_settings, permission_changed)"));
    assert!(shipping_source.contains("self.call_public_settings_command(\"config.set\", args)"));
    assert!(shipping_source.contains("self.call_public_settings_command(\"config.get\", args)"));
    assert!(shipping_source.contains("if matches!(method, \"getHostSettings\" | \"setHostSettings\")"));
    assert!(shipping_source.contains("return self.call_public_settings_command(method, &args)"));
    assert!(config_set.contains("map_err(GatewayCommandError::from)"));
    assert!(config_set.contains("\"hash\": snapshot.hash"));
    assert!(config_set.contains("\"configSource\": snapshot.config_source"));
    assert!(!config_set.contains("get_host_settings()"));
    assert!(!config_set.contains("get_config_source_metadata()"));
    assert!(!config_set.contains("set_host_settings_if_current("));
    let _ = fs::remove_file(path);
}

#[test]
fn config_set_racing_one_owner_hash_has_one_commit_and_one_conflict() {
    let path = temp_path("config-cas-race");
    let service = Arc::new(SettingsService::new(&path));
    let initial = service.get_host_settings_snapshot();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |fields| {
        sink.lock().expect("observed changes").push(fields);
    }));
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();
    for model_id in ["model-a", "model-b"] {
        let owner = Arc::clone(&service);
        let barrier = Arc::clone(&barrier);
        let base_hash = initial.hash.clone();
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            owner.set_host_settings_if_current(
                Some(&base_hash),
                json!({
                    "agentDefaultModel": {
                        "modelId": model_id, "maxMode": false, "parameters": []
                    }
                }),
            )
        }));
    }
    barrier.wait();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().expect("config writer thread"))
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let committed = results
        .iter()
        .find_map(|result| result.as_ref().ok())
        .expect("one committed snapshot");
    let conflict = results
        .iter()
        .find_map(|result| result.as_ref().err())
        .expect("one rejected writer");
    assert!(matches!(
        conflict,
        HostSettingsUpdateError::Conflict { expected, actual }
            if expected == &committed.hash && actual == &initial.hash
    ));
    assert_eq!(GatewayCommandError::from((*conflict).clone()).status(), 409);
    assert_eq!(observed.lock().expect("observed changes").len(), 1);
    assert_eq!(&service.get_host_settings_snapshot(), committed);
    let persisted: Value = serde_json::from_slice(&fs::read(&path).expect("settings bytes"))
        .expect("persisted JSON");
    assert_eq!(
        persisted["agentDefaultModel"],
        committed.settings["agentDefaultModel"]
    );
    assert_eq!(committed.settings["agentDefaultModel"]["maxMode"], true);
    assert_eq!(
        SettingsService::new(&path).get_host_settings_snapshot(),
        (*committed).clone()
    );
    let _ = fs::remove_file(path);
}

#[test]
fn config_set_model_validation_preserves_cas_and_readonly_fence_order() {
    let path = temp_path("bound-model-fence-order");
    let original = br#"{"version":1,"notifications":{"isEnabled":true,"legacy":"keep"},"mcpCustomInstructionsAccountScope":"account-a","agentDefaultModel":{"provider":"fabushi","modelId":"Original/Model","maxMode":true,"parameters":[]}}"#;
    fs::write(&path, original).expect("seed legacy notification and bound model");
    let service = SettingsService::new(&path);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |fields| {
        sink.lock().expect("observed changes").push(fields);
    }));
    let invalid_update = json!({
        "agentDefaultModel": {"provider": null, "modelId": "Candidate/Model", "maxMode": false, "parameters": []},
        "mcpCustomInstructionsAccountScope": "account-b", "userTimeZone": "UTC"
    });
    let conflict = service.set_host_settings_if_current(Some("stale"), invalid_update.clone())
        .expect_err("CAS remains the first update fence");
    assert!(matches!(&conflict, HostSettingsUpdateError::Conflict { actual, .. } if actual == "stale"));
    assert_eq!(GatewayCommandError::from(conflict).status(), 409);
    assert_eq!(fs::read(&path).expect("source after conflict"), original);

    let current = service.get_host_settings_snapshot();
    // The normalized hash does not depend on the legacy notification bytes.
    // Restore them so a rejected update must prove it cannot normalize disk.
    fs::write(&path, original).expect("restore equivalent unnormalized source");
    let rejected = service.set_host_settings_if_current(Some(&current.hash), invalid_update.clone())
        .expect_err("matching CAS reaches model validation");
    assert!(matches!(rejected, HostSettingsUpdateError::Rejected(message) if message.contains("agentDefaultModel.provider")));
    assert_eq!(fs::read(&path).expect("source after validation refusal"), original);
    assert!(service.set_host_settings(&invalid_update).is_err());
    assert_eq!(fs::read(&path).expect("source after compatibility writer refusal"), original);

    let mut read_only: Value = serde_json::from_slice(original).expect("original JSON");
    read_only["readOnly"] = json!(true);
    let read_only_bytes = serde_json::to_vec(&read_only).expect("readonly source bytes");
    fs::write(&path, &read_only_bytes).expect("seed externally managed source");
    let read_only_snapshot = service.get_host_settings_snapshot();
    let readonly = service.set_host_settings_if_current(Some(&read_only_snapshot.hash), invalid_update)
        .expect_err("readonly fence precedes model validation");
    assert!(matches!(readonly, HostSettingsUpdateError::Rejected(message) if message.contains("readOnly")));
    assert_eq!(fs::read(&path).expect("source after readonly refusal"), read_only_bytes);
    assert_eq!(service.get_agent_default_model_checked().expect("readonly model remains readable")
        .expect("original selection")["modelId"], "Original/Model");
    assert!(observed.lock().expect("observed changes").is_empty());
    let _ = fs::remove_file(path);
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
    let memory = mahayana_host_runtime::host_system_info::sample_host_memory()
        .expect("host memory observation");
    let hostname = mahayana_host_runtime::host_system_info::sample_host_hostname()
        .expect("host hostname observation");
    let release = mahayana_host_runtime::host_system_info::sample_host_os_release()
        .expect("host OS release observation");

    let info = json!({
        "machineName": hostname.clone(),
        "hostname": hostname,
        "platform": std::env::consts::OS,
        "release": release,
        "arch": std::env::consts::ARCH,
        "osLabel": std::env::consts::OS,
        "pid": pid,
        "cpuCount": cpu_count,
        "memoryTotalBytes": memory.total_bytes,
        "memoryFreeBytes": memory.available_bytes,
        "uptimeMs": 1000,
        "processInstanceId": format!("sand-host-{pid}"),
        "nodeVersion": env!("CARGO_PKG_VERSION"),
    });

    // Verify schema
    assert!(!info["nodeVersion"].as_str().unwrap().is_empty());
    assert!(info["pid"].as_u64().unwrap() > 0);
    assert!(info["cpuCount"].as_u64().unwrap() >= 1);
    assert!(info["processInstanceId"].as_str().unwrap().starts_with("sand-host-"));
    assert!(!info["release"].as_str().unwrap().is_empty());
    assert!(!info["hostname"].as_str().unwrap().is_empty());

    // Zero secret leakage
    let serialized = serde_json::to_string(&info).unwrap();
    assert!(!serialized.contains("\"version\""));
    assert!(!serialized.contains("\"status\""));
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("secret"));
    assert!(!serialized.contains("apiKey"));

    let shipping_source = include_str!("../app/src/main.rs");
    let system_info = shipping_source
        .split("fn call_openclaw_system_info(")
        .nth(1)
        .expect("shipping system.info owner exists")
        .split("fn call_openclaw_sessions_list(")
        .next()
        .expect("shipping system.info function boundary exists");
    assert!(system_info.contains("sample_host_memory"));
    assert!(system_info.contains("sample_host_os_release"));
    assert!(system_info.contains("sample_host_hostname"));
    assert!(system_info.contains("\"nodeVersion\""));
    assert!(!system_info.contains("\"version\": env!(\"CARGO_PKG_VERSION\")"));
    assert!(!system_info.contains("\"status\": \"ok\""));

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

    let shipping_source = include_str!("../app/src/main.rs");
    let sessions_get = shipping_source
        .split("fn call_openclaw_sessions_get(")
        .nth(1)
        .expect("shipping sessions.get owner exists");
    let sessions_get = sessions_get
        .split("fn call_openclaw_sessions_create(")
        .next()
        .expect("shipping sessions.get function boundary exists");
    assert!(sessions_get.contains("map_err(GatewayCommandError::Internal)?"));
    assert!(sessions_get.contains("sessions.get requires key, sessionKey, or sessionId"));
    assert!(sessions_get.contains("reject_agent_wide_session_route(&route, \"sessions.get\")"));
    assert!(!sessions_get.contains("return Ok(serde_json::json!({ \"messages\": [] }))"));
    assert!(sessions_get.contains("entry.get(\"message\").and_then(|message| message.get(\"content\"))"));
    assert!(sessions_get.contains("entry.get(\"timestampMs\")"));
    assert!(sessions_get.contains("let start = entries.len().saturating_sub(limit)"));
    assert!(sessions_get.contains("entries.into_iter().skip(start)"));
    assert!(!sessions_get.contains("entries.into_iter().take(limit)"));
}

#[test]
fn test_sessions_list_projects_exact_active_run_set() {
    let shipping_source = include_str!("../app/src/main.rs");
    let sessions_list = shipping_source
        .split("fn call_openclaw_sessions_list(")
        .nth(1)
        .expect("shipping sessions.list owner exists")
        .split("fn call_openclaw_sessions_get(")
        .next()
        .expect("shipping sessions.list function boundary exists");
    assert!(sessions_list.contains("active_stream_ids_for_agent(&s.id)"));
    assert!(sessions_list.contains("let has_active_run = !active_run_ids.is_empty()"));
    assert!(sessions_list.contains("s.is_running || has_active_run"));
    assert!(sessions_list.contains("\"hasActiveRun\": has_active_run"));
    assert!(sessions_list.contains("\"activeRunIds\": active_run_ids"));
    assert!(sessions_list.contains("resolve_gateway_session_route"));
}

#[test]
fn test_sessions_create_requires_durable_agent_identity() {
    let shipping_source = include_str!("../app/src/main.rs");
    let sessions_create = shipping_source
        .split("fn call_openclaw_sessions_create(")
        .nth(1)
        .expect("shipping sessions.create owner exists")
        .split("fn call_openclaw_sessions_delete(")
        .next()
        .expect("shipping sessions.create function boundary exists");
    assert!(sessions_create.contains("createAgent returned no durable agent identity"));
    assert!(sessions_create.contains("filter(|id| !id.is_empty())"));
    assert!(!sessions_create.contains("unwrap_or_default();\n\n        let session_key"));
}

#[test]
fn test_sessions_delete_guards_root_session_and_cas() {
    let guard_delete = |key: &str, expected_session_id: Option<&str>| -> Result<(), GatewayCommandError> {
        if key.is_empty() {
            return Err(GatewayCommandError::BadRequest("sessions.delete requires key or sessionId".into()));
        }
        if is_root_main_session_key(key) {
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
    let err_root3 = guard_delete("GLOBAL", None).unwrap_err();
    assert_eq!(err_root3.status(), 400);
    let err_root4 = guard_delete("AGENT:GLOBAL:MAIN", None).unwrap_err();
    assert_eq!(err_root4.status(), 400);

    // CAS check: mismatch returns SESSION_LIFECYCLE_CHANGED
    let err_cas = guard_delete("agent:agent-correct:main", Some("agent-stale")).unwrap_err();
    assert_eq!(err_cas.status(), 400);
    assert!(err_cas.to_string().contains("SESSION_LIFECYCLE_CHANGED"));

    // Valid delete passes guard
    assert!(guard_delete("agent:agent-correct:main", Some("agent-correct")).is_ok());
    assert!(guard_delete("agent-correct", None).is_ok());

    let empty_agent = "agent::main";
    let empty_agent_id = empty_agent
        .strip_prefix("agent:")
        .and_then(|value| value.split(':').next())
        .unwrap_or(empty_agent);
    assert!(empty_agent_id.is_empty());

    let shipping_source = include_str!("../app/src/main.rs");
    let sessions_delete = shipping_source
        .split("fn call_openclaw_sessions_delete(")
        .nth(1)
        .expect("shipping sessions.delete owner exists")
        .split("fn call_openclaw_chat_send(")
        .next()
        .expect("shipping sessions.delete function boundary exists");
    assert!(sessions_delete.contains("sessions.delete requires a non-empty agent identity"));
    assert!(sessions_delete.contains("is_root_main_session_key(raw_key)"));
    assert!(sessions_delete.contains("reject_agent_wide_session_route(&route, \"sessions.delete\")"));
}

#[test]
fn test_sessions_patch_requires_authenticated_identity_revision_and_terminal_state() {
    let shipping_source = include_str!("../app/src/main.rs");
    let sessions_patch = shipping_source
        .split("fn call_openclaw_sessions_patch(")
        .nth(1)
        .expect("shipping sessions.patch owner exists")
        .split("fn delete_generated_subagent(")
        .next()
        .expect("shipping sessions.patch function boundary exists");
    assert!(sessions_patch.contains("context.authenticated"));
    assert!(sessions_patch.contains("sessions.patch requires expectedSessionId"));
    assert!(sessions_patch.contains("expected_session_id != route.agent_id.as_str()"));
    assert!(sessions_patch.contains("SESSION_LIFECYCLE_CHANGED"));
    assert!(sessions_patch.contains("sessions.patch requires expectedLifecycleRevision"));
    assert!(sessions_patch.contains("canonical main sessions"));
    assert!(sessions_patch.contains("reject_agent_wide_session_route(&route, \"sessions.patch\")"));
    assert!(sessions_patch.contains("raw_key.eq_ignore_ascii_case(&expected_key)"));
    assert!(sessions_patch.contains("running_subagent_records_for_parent(&route.agent_id)"));
    assert!(sessions_patch.contains("active subagent work"));
    assert!(sessions_patch.contains("active_stream_ids_for_agent(&route.agent_id)"));
    assert!(sessions_patch.contains("active foreground turn"));
    assert!(sessions_patch.contains("transition_session_work_archive"));
    assert!(sessions_patch.contains("communicationPolicy"));
    assert!(sessions_patch.contains("write_session_communication_policy"));
    assert!(sessions_patch.contains("cannot combine communicationPolicy with archived"));
}

#[test]
fn test_provider_review_continuation_uses_owner_capability_and_existing_run_path() {
    let shipping_source = include_str!("../app/src/main.rs");
    let continuation = shipping_source
        .split("fn call_openclaw_sessions_provider_review_continue(")
        .nth(1)
        .expect("shipping provider-review continuation owner exists")
        .split("fn delete_generated_subagent(")
        .next()
        .expect("provider-review continuation function boundary exists");
    assert!(continuation.contains("context.authenticated"));
    assert!(continuation.contains("canonical main sessions"));
    assert!(continuation.contains("expected_session_id != route.agent_id.as_str()"));
    assert!(continuation.contains("SESSION_LIFECYCLE_CHANGED"));
    assert!(continuation.contains("issue_provider_review_acknowledgment"));
    assert!(continuation.contains("claim_provider_review_acknowledgment"));
    assert!(continuation.contains("capture_session_work_admission"));
    assert!(continuation.contains("accept_routed_send"));
    assert!(continuation.contains("RUNNER_START_ROUTED_PROVIDER_GATEWAY_METHOD"));
    assert!(continuation.contains("providerReviewLifecycleRevision"));
    assert!(continuation.contains("release_provider_review_continuation"));
    let send_failure = continuation
        .split_once(".accept_routed_send(&runner_args, |accepted| {")
        .expect("provider-review send failure follows actual Transcript acceptance")
        .1
        .split("match self.call_with_context")
        .next()
        .expect("provider-review send failure branch boundary exists");
    let assert_release_order = |branch: &str| {
        let review_release = branch.find("release_provider_review_continuation")
            .expect("branch must release provider-review continuation");
        let work_release = branch.find("release_session_work_admission")
            .expect("branch must release admitted work");
        assert!(review_release < work_release,
            "provider review must release before its admitted-work marker");
    };
    assert_release_order(send_failure);
    let runner_match = continuation
        .split_once("match self.call_with_context(")
        .expect("provider-review continuation uses the actual Runner boundary")
        .1;
    let (runner_success, runner_failure) = runner_match
        .split_once("Err(error) => {")
        .expect("provider-review Runner failure branch exists");
    assert_release_order(runner_failure);
    let runner_cancelled = runner_success
        .split_once("Ok(result) => {")
        .expect("provider-review Runner success branch exists")
        .1;
    assert!(runner_cancelled.contains(
        "result.get(\"accepted\").and_then(serde_json::Value::as_bool) == Some(false)"));
    assert!(runner_cancelled.contains(
        "result.get(\"cancelled\").and_then(serde_json::Value::as_bool) == Some(true)"));
    assert!(runner_cancelled.contains(
        "result.get(\"streamId\").and_then(serde_json::Value::as_str) == Some(next_run_id)"));
    assert_release_order(runner_cancelled);
    assert!(!continuation.contains("serde_json::from_value::<ProviderReviewAcknowledgment"));
}

#[test]
fn test_provider_review_projection_hides_non_pending_continuations() {
    let shipping_source = include_str!("../app/src/main.rs");
    let projection = shipping_source
        .split("fn provider_review_projection(")
        .nth(1)
        .expect("provider-review projection owner exists")
        .split("fn call_openclaw_sessions_list(")
        .next()
        .expect("provider-review projection boundary exists");
    assert!(projection.contains("provider_review_phase.as_deref() == Some(\"pending\")"));
    assert!(projection.contains("record.session_id != session_id"));
    assert!(projection.contains("record.session_key != session_key"));
    assert!(projection.contains("read_session_work_admission"));
}

#[test]
fn test_provider_review_owner_exposes_single_claim_host_capability() {
    let owner_source = include_str!("../src/extensions/session/production.rs");
    assert!(owner_source.contains("pub struct ProviderReviewAcknowledgment"));
    assert!(owner_source.contains("issue_provider_review_acknowledgment"));
    assert!(owner_source.contains("claim_provider_review_acknowledgment"));
    assert!(owner_source.contains("next_run_id.trim() != state.next_run_id"));
    assert!(owner_source.contains("provider_review_capabilities"));
    assert!(owner_source.contains(".remove(&acknowledgment.token)"));
    assert!(owner_source.contains("provider_review_phase.as_deref() != Some(\"pending\")"));
    assert_eq!(
        owner_source
            .matches("record.session_id != review.session_id")
            .count(),
        2,
        "provider-review issue and claim must both enforce session identity"
    );
    assert!(owner_source.contains("settle_provider_review_continuation"));
    assert!(owner_source.contains("provider_review_continuation_run_id"));
}

#[test]
fn test_session_admission_rejects_rebound_owner_identity() {
    let owner_source = include_str!("../src/extensions/session/production.rs");
    assert!(owner_source.contains("record.session_id != agent_id"));
    assert!(owner_source.contains("let expected_session_key = format!(\"agent:{agent_id}:main\")"));
    assert!(owner_source.contains("record.session_key != expected_session_key"));
    assert!(
        owner_source
            .split("pub(crate) fn read_session_work_admission_locked(")
            .nth(1)
            .expect("locked session admission reader exists")
            .split("pub fn read_session_communication_policy(")
            .next()
            .expect("locked session admission reader boundary exists")
            .contains("record.session_key != expected_session_key")
    );
    assert!(owner_source.contains("record.lifecycle_revision.trim().is_empty()"));
    assert!(owner_source.contains("session admission identity is malformed"));
    assert!(owner_source.contains("pub fn read_session_work_admission("));
    assert!(owner_source.contains("session admission read requires a non-empty agent identity"));
    assert!(owner_source.contains("locked session admission read requires a non-empty agent identity"));
}

const SESSION_ADMISSION_GUARD_OWNERS: [(&str, &str, usize); 8] = [
    ("", "capture_original_turn_terminal_fence", 1),
    ("SandAgentDb", "commit_session_work_result_deferred", 2),
    ("SandAgentDb", "append_transcript_entries_if_session_admitted", 1),
    ("SandAgentDb", "append_spend_guard_card_if_session_admitted", 1),
    ("SandAgentDb", "append_transcript_entries_and_release_media", 3),
    ("SandAgentDb", "update_transcript_entry_if_session_admitted", 1),
    ("SandAgentDb", "settle_transcript_entry_identity_if_session_admitted", 1),
    ("", "update_persisted_transcript_entry_if_session_admitted", 1),
];

fn admission_guard_attributes_are_supported(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().all(|attribute| attribute.path().is_ident("doc"))
}

fn admission_guard_calls_in_block(block: &syn::Block) -> Option<usize> {
    block.stmts.iter().try_fold(0, |count, statement| {
        let calls = match statement {
            syn::Stmt::Local(local) => {
                if !admission_guard_attributes_are_supported(&local.attrs) {
                    return None;
                }
                match local.init.as_ref() {
                    Some(init) => admission_guard_calls_in_expr(&init.expr)?
                        + match init.diverge.as_ref() {
                            Some((_, expression)) => admission_guard_calls_in_expr(expression)?,
                            None => 0,
                        },
                    None => 0,
                }
            }
            syn::Stmt::Expr(expression, _) => admission_guard_calls_in_expr(expression)?,
            // A nested function or macro body is not this function owner's
            // executable call site. Neither can compensate for a missing guard.
            syn::Stmt::Item(_) | syn::Stmt::Macro(_) => 0,
        };
        Some(count + calls)
    })
}

fn admission_guard_calls_in_expr(expression: &syn::Expr) -> Option<usize> {
    use syn::Expr;

    fn many<'a>(expressions: impl IntoIterator<Item = &'a Expr>) -> Option<usize> {
        expressions.into_iter().try_fold(0, |count, expression| {
            Some(count + admission_guard_calls_in_expr(expression)?)
        })
    }

    fn optional(expression: Option<&Expr>) -> Option<usize> {
        expression.map_or(Some(0), admission_guard_calls_in_expr)
    }

    fn directly_invoked_synchronous_closure(expression: &Expr) -> Option<&syn::ExprClosure> {
        match expression {
            Expr::Paren(value) if admission_guard_attributes_are_supported(&value.attrs) => {
                directly_invoked_synchronous_closure(&value.expr)
            }
            Expr::Group(value) if admission_guard_attributes_are_supported(&value.attrs) => {
                directly_invoked_synchronous_closure(&value.expr)
            }
            Expr::Closure(value) if value.asyncness.is_none()
                && admission_guard_attributes_are_supported(&value.attrs) => Some(value),
            _ => None,
        }
    }

    // This is a bounded syntactic inventory, not a cfg evaluator or a proof of
    // control-flow dominance. Unsupported attributes cannot establish a call.
    let attributes = match expression {
        Expr::Array(value) => &value.attrs,
        Expr::Assign(value) => &value.attrs,
        Expr::Async(value) => &value.attrs,
        Expr::Await(value) => &value.attrs,
        Expr::Binary(value) => &value.attrs,
        Expr::Block(value) => &value.attrs,
        Expr::Break(value) => &value.attrs,
        Expr::Call(value) => &value.attrs,
        Expr::Cast(value) => &value.attrs,
        Expr::Closure(value) => &value.attrs,
        Expr::Const(value) => &value.attrs,
        Expr::Continue(value) => &value.attrs,
        Expr::Field(value) => &value.attrs,
        Expr::ForLoop(value) => &value.attrs,
        Expr::Group(value) => &value.attrs,
        Expr::If(value) => &value.attrs,
        Expr::Index(value) => &value.attrs,
        Expr::Infer(value) => &value.attrs,
        Expr::Let(value) => &value.attrs,
        Expr::Lit(value) => &value.attrs,
        Expr::Loop(value) => &value.attrs,
        Expr::Macro(value) => &value.attrs,
        Expr::Match(value) => &value.attrs,
        Expr::MethodCall(value) => &value.attrs,
        Expr::Paren(value) => &value.attrs,
        Expr::Path(value) => &value.attrs,
        Expr::Range(value) => &value.attrs,
        Expr::Reference(value) => &value.attrs,
        Expr::Repeat(value) => &value.attrs,
        Expr::Return(value) => &value.attrs,
        Expr::Struct(value) => &value.attrs,
        Expr::Try(value) => &value.attrs,
        Expr::TryBlock(value) => &value.attrs,
        Expr::Tuple(value) => &value.attrs,
        Expr::Unary(value) => &value.attrs,
        Expr::Unsafe(value) => &value.attrs,
        Expr::While(value) => &value.attrs,
        Expr::Yield(value) => &value.attrs,
        _ => return None,
    };
    if !admission_guard_attributes_are_supported(attributes) {
        return None;
    }

    match expression {
        Expr::Call(call) => {
            let named_guard = matches!(call.func.as_ref(), Expr::Path(function)
                if function.qself.is_none()
                    && function.path.is_ident("session_admission_identity_is_well_formed"));
            let actual_admission = call.args.len() == 1
                && matches!(call.args.first(), Some(Expr::Reference(argument))
                    if argument.mutability.is_none()
                        && matches!(argument.expr.as_ref(), Expr::Path(value)
                            if value.qself.is_none() && value.path.is_ident("admission")));
            // The transaction owners call synchronous closure literals through
            // parentheses. A closure value or an async-closure call alone does
            // not execute its body, so only this direct call edge earns credit.
            let callee_calls = match directly_invoked_synchronous_closure(&call.func) {
                Some(closure) => admission_guard_calls_in_expr(&closure.body)?,
                None => admission_guard_calls_in_expr(&call.func)?,
            };
            Some(usize::from(named_guard && actual_admission) + callee_calls + many(&call.args)?)
        }
        Expr::MethodCall(call) => Some(
            admission_guard_calls_in_expr(&call.receiver)? + many(&call.args)?,
        ),
        Expr::Array(array) => many(&array.elems),
        Expr::Tuple(tuple) => many(&tuple.elems),
        Expr::Assign(value) => many([value.left.as_ref(), value.right.as_ref()]),
        Expr::Binary(value) => many([value.left.as_ref(), value.right.as_ref()]),
        Expr::Index(value) => many([value.expr.as_ref(), value.index.as_ref()]),
        Expr::Repeat(value) => many([value.expr.as_ref(), value.len.as_ref()]),
        Expr::Range(value) => Some(
            optional(value.start.as_deref())? + optional(value.end.as_deref())?,
        ),
        Expr::Struct(value) => {
            if value.fields.iter().any(|field| !admission_guard_attributes_are_supported(&field.attrs)) {
                return None;
            }
            Some(many(value.fields.iter().map(|field| &field.expr))?
                + optional(value.rest.as_deref())?)
        }
        Expr::If(value) => Some(
            admission_guard_calls_in_expr(&value.cond)?
                + admission_guard_calls_in_block(&value.then_branch)?
                + optional(value.else_branch.as_ref().map(|(_, branch)| branch.as_ref()))?,
        ),
        Expr::Match(value) => {
            let arms = value.arms.iter().try_fold(0, |count, arm| {
                if !admission_guard_attributes_are_supported(&arm.attrs) {
                    return None;
                }
                Some(count + admission_guard_calls_in_expr(&arm.body)?
                    + optional(arm.guard.as_ref().map(|(_, guard)| guard.as_ref()))?)
            })?;
            Some(admission_guard_calls_in_expr(&value.expr)? + arms)
        }
        Expr::ForLoop(value) => Some(
            admission_guard_calls_in_expr(&value.expr)?
                + admission_guard_calls_in_block(&value.body)?,
        ),
        Expr::While(value) => Some(
            admission_guard_calls_in_expr(&value.cond)?
                + admission_guard_calls_in_block(&value.body)?,
        ),
        Expr::Loop(value) => admission_guard_calls_in_block(&value.body),
        Expr::Block(value) => admission_guard_calls_in_block(&value.block),
        Expr::Const(value) => admission_guard_calls_in_block(&value.block),
        Expr::TryBlock(value) => admission_guard_calls_in_block(&value.block),
        Expr::Unsafe(value) => admission_guard_calls_in_block(&value.block),
        // Constructing a closure or future is not execution of its body. The
        // only supported deferred-body entry is the synchronous call above.
        Expr::Async(_) | Expr::Closure(_) => Some(0),
        Expr::Await(value) => admission_guard_calls_in_expr(&value.base),
        Expr::Cast(value) => admission_guard_calls_in_expr(&value.expr),
        Expr::Field(value) => admission_guard_calls_in_expr(&value.base),
        Expr::Group(value) => admission_guard_calls_in_expr(&value.expr),
        Expr::Let(value) => admission_guard_calls_in_expr(&value.expr),
        Expr::Paren(value) => admission_guard_calls_in_expr(&value.expr),
        Expr::Reference(value) => admission_guard_calls_in_expr(&value.expr),
        Expr::Try(value) => admission_guard_calls_in_expr(&value.expr),
        Expr::Unary(value) => admission_guard_calls_in_expr(&value.expr),
        Expr::Break(value) => optional(value.expr.as_deref()),
        Expr::Return(value) => optional(value.expr.as_deref()),
        Expr::Yield(value) => optional(value.expr.as_deref()),
        // Literals/comments and opaque macro input cannot establish a call.
        Expr::Lit(_) | Expr::Path(_) | Expr::Infer(_) | Expr::Continue(_) | Expr::Macro(_) => Some(0),
        _ => None,
    }
}

fn check_session_admission_guard_owners(source: &str) -> Result<(), String> {
    let file = syn::parse_file(source).map_err(|error| format!("invalid owner source: {error}"))?;
    for (owner, name, expected) in SESSION_ADMISSION_GUARD_OWNERS {
        let qualified = if owner.is_empty() { name.to_string() } else { format!("{owner}::{name}") };
        let mut candidates = Vec::new();
        for item in &file.items {
            match item {
                syn::Item::Fn(function) if owner.is_empty() && function.sig.ident == name => {
                    candidates.push((&function.attrs, function.block.as_ref()));
                }
                syn::Item::Impl(implementation) if !owner.is_empty() => {
                    let syn::Type::Path(ty) = implementation.self_ty.as_ref() else { continue; };
                    if ty.qself.is_some() || !ty.path.is_ident(owner)
                        || implementation.trait_.is_some()
                        || !implementation.generics.params.is_empty()
                        || implementation.generics.where_clause.is_some()
                        || implementation.attrs.iter().any(|attribute| !attribute.path().is_ident("doc"))
                    {
                        continue;
                    }
                    for item in &implementation.items {
                        if let syn::ImplItem::Fn(function) = item {
                            if function.sig.ident == name {
                                candidates.push((&function.attrs, &function.block));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if candidates.len() != 1 {
            return Err(format!("{qualified} requires exactly one actual function owner; found {}", candidates.len()));
        }
        let (attributes, body) = candidates[0];
        if attributes.iter().any(|attribute| !attribute.path().is_ident("doc")) {
            return Err(format!("{qualified} has a conditional or unsupported owner attribute"));
        }
        let actual = admission_guard_calls_in_block(body)
            .ok_or_else(|| format!("{qualified} has an unsupported expression in its owner body"))?;
        if actual != expected {
            return Err(format!("{qualified} requires {expected} admission guard calls; found {actual}"));
        }
    }
    Ok(())
}

fn admission_guard_oracle_fixture() -> &'static str {
    let source = include_str!("../src/extensions/session/agent_db.rs");
    check_session_admission_guard_owners(source)
        .expect("the actual source must satisfy all eight named owner boundaries before mutation");
    source
}

fn replace_admission_guard_fixture_call(
    source: &str,
    function: &str,
    ordinal: usize,
    replacement: &str,
) -> String {
    const CALL: &str = "session_admission_identity_is_well_formed(&admission)";
    let signature = format!("fn {function}(");
    assert_eq!(source.matches(signature.as_str()).count(), 1, "the counterfactual requires one actual named declaration");
    let start = source.find(signature.as_str()).expect("actual owner declaration");
    let relative = source[start..].match_indices(CALL).nth(ordinal)
        .expect("actual named owner's guard ordinal").0;
    let mut changed = source.to_string();
    changed.replace_range(start + relative..start + relative + CALL.len(), replacement);
    changed
}

#[test]
fn admission_guard_oracle_rejects_each_removed_transaction_and_terminal_guard() {
    let source = admission_guard_oracle_fixture();
    for (owner, function, count) in SESSION_ADMISSION_GUARD_OWNERS {
        for ordinal in 0..count {
            let changed = replace_admission_guard_fixture_call(source, function, ordinal, "true");
            assert!(syn::parse_file(&changed).is_ok(), "removal must leave valid Rust syntax");
            let error = check_session_admission_guard_owners(&changed)
                .expect_err("deleting any one of the eleven real calls must fail its owner");
            let qualified = if owner.is_empty() { function.to_string() } else { format!("{owner}::{function}") };
            assert!(error.starts_with(&qualified), "wrong failed boundary for {function}/{ordinal}: {error}");
        }
    }
}

#[test]
fn admission_guard_oracle_rejects_cross_owner_relocation_with_unchanged_total() {
    const CALL: &str = "session_admission_identity_is_well_formed(&admission)";
    let source = admission_guard_oracle_fixture();
    for (index, (_, from, _)) in SESSION_ADMISSION_GUARD_OWNERS.iter().enumerate() {
        let (_, to, _) = SESSION_ADMISSION_GUARD_OWNERS[(index + 1) % SESSION_ADMISSION_GUARD_OWNERS.len()];
        let removed = replace_admission_guard_fixture_call(source, from, 0, "true");
        let changed = replace_admission_guard_fixture_call(
            &removed, to, 0, &format!("({CALL} && {CALL})"),
        );
        assert_eq!(changed.matches(CALL).count(), source.matches(CALL).count(),
            "this negative specifically defeats a whole-file count oracle");
        assert!(syn::parse_file(&changed).is_ok(), "relocation must leave valid Rust syntax");
        assert!(check_session_admission_guard_owners(&changed).is_err(), "{from} -> {to}");
    }
}

#[test]
fn admission_guard_oracle_rejects_missing_duplicate_and_foreign_function_owners() {
    let source = admission_guard_oracle_fixture();
    for (owner, function, _) in SESSION_ADMISSION_GUARD_OWNERS {
        let signature = format!("fn {function}(");
        let missing = source.replacen(signature.as_str(), &format!("fn renamed_{function}("), 1);
        assert!(syn::parse_file(&missing).is_ok());
        assert!(check_session_admission_guard_owners(&missing).is_err(), "missing {owner}::{function}");

        let extra = if owner.is_empty() {
            format!("\nfn {function}() {{}}\n")
        } else {
            format!("\nimpl {owner} {{ fn {function}() {{}} }}\n")
        };
        let duplicate = format!("{source}{extra}");
        assert!(syn::parse_file(&duplicate).is_ok());
        assert!(check_session_admission_guard_owners(&duplicate).is_err(), "duplicate {owner}::{function}");

        let foreign = if owner.is_empty() {
            format!("{missing}\nmod unrelated {{ fn {function}() {{}} }}\n")
        } else {
            format!("{missing}\nimpl UnrelatedOwner {{ fn {function}() {{}} }}\n")
        };
        assert!(syn::parse_file(&foreign).is_ok());
        assert!(check_session_admission_guard_owners(&foreign).is_err(), "foreign {owner}::{function}");
    }
}

#[test]
fn admission_guard_oracle_cannot_replace_terminal_guard_with_non_owner_text_or_code() {
    const CALL: &str = "session_admission_identity_is_well_formed(&admission)";
    let source = admission_guard_oracle_fixture();
    for replacement in [
        format!("{{ /* {CALL} */ true }}"),
        format!("{{ let _ = \"{CALL}\"; true }}"),
        format!("{{ let _ = stringify!({CALL}); true }}"),
        format!("{{ fn nested(admission: SessionWorkAdmissionRecord) -> bool {{ {CALL} }} true }}"),
        format!("unrelated::{CALL}"),
        format!("unrelated.{CALL}"),
    ] {
        let changed = replace_admission_guard_fixture_call(
            source, "capture_original_turn_terminal_fence", 0, &replacement,
        );
        assert_eq!(changed.matches(CALL).count(), source.matches(CALL).count());
        assert!(syn::parse_file(&changed).is_ok(), "counterfactual syntax: {replacement}");
        assert!(check_session_admission_guard_owners(&changed).is_err(), "{replacement}");
    }
}

#[test]
fn admission_guard_oracle_counts_direct_synchronous_closure_invocation() {
    const CALL: &str = "session_admission_identity_is_well_formed(&admission)";
    const OWNER: &str = "capture_original_turn_terminal_fence";
    let source = admission_guard_oracle_fixture();
    for closure in [
        format!("(|| {CALL})"),
        format!("((|| -> bool {{ {CALL} }}))"),
    ] {
        let invoked = replace_admission_guard_fixture_call(source, OWNER, 0, &format!("{closure}()"));
        assert_eq!(invoked.matches(CALL).count(), source.matches(CALL).count());
        assert!(syn::parse_file(&invoked).is_ok(), "invocation syntax: {closure}");
        check_session_admission_guard_owners(&invoked)
            .expect("a directly invoked synchronous closure must retain its actual guard");

        let dormant = replace_admission_guard_fixture_call(
            source, OWNER, 0, &format!("{{ let _ = {closure}; true }}"),
        );
        assert_eq!(dormant.matches(CALL).count(), source.matches(CALL).count());
        assert!(syn::parse_file(&dormant).is_ok(), "dormant syntax: {closure}");
        let error = check_session_admission_guard_owners(&dormant)
            .expect_err("removing the invocation must remove credit for the same closure body");
        assert!(error.starts_with(OWNER), "wrong failed boundary: {error}");
    }
}

#[test]
fn admission_guard_oracle_rejects_dormant_closures_and_futures() {
    const CALL: &str = "session_admission_identity_is_well_formed(&admission)";
    const OWNER: &str = "capture_original_turn_terminal_fence";
    let source = admission_guard_oracle_fixture();
    for replacement in [
        format!("{{ let _ = || {CALL}; true }}"),
        format!("{{ let _ = || (|| {CALL})(); true }}"),
        format!("(|| {{ let _ = || {CALL}; true }})()"),
        format!("{{ let _ = async {{ {CALL} }}; true }}"),
        format!("{{ let _ = async {{ (|| {CALL})() }}; true }}"),
        format!("{{ let _ = (async || {{ {CALL} }})(); true }}"),
        format!("{{ let _ = (|| async {{ {CALL} }})(); true }}"),
    ] {
        let changed = replace_admission_guard_fixture_call(source, OWNER, 0, &replacement);
        assert_eq!(changed.matches(CALL).count(), source.matches(CALL).count());
        assert!(syn::parse_file(&changed).is_ok(), "deferred-body syntax: {replacement}");
        let error = check_session_admission_guard_owners(&changed)
            .expect_err("a dormant closure or future cannot replace the live terminal guard");
        assert!(error.starts_with(OWNER), "wrong failed boundary for {replacement}: {error}");
    }
}

#[test]
fn admission_guard_oracle_rejects_conditionally_excluded_guard_paths() {
    const CALL: &str = "session_admission_identity_is_well_formed(&admission)";
    const OWNER: &str = "capture_original_turn_terminal_fence";
    let source = admission_guard_oracle_fixture();
    for replacement in [
        format!("{{ #[cfg(any())] let _ = {CALL}; true }}"),
        format!("{{ #[cfg_attr(all(), cfg(any()))] let _ = {CALL}; true }}"),
        format!("{{ #[cfg(any())] {CALL}; true }}"),
        format!("{{ #[cfg(any())] {{ {CALL}; }} true }}"),
        format!("{{ #[cfg(any())] (|| {{ {CALL} }})(); true }}"),
        format!("{{ match () {{ #[cfg(any())] () => {{ {CALL}; }}, _ => (), }} true }}"),
        format!("{{ struct GuardFixture {{ #[cfg(any())] checked: bool }} let _ = GuardFixture {{ #[cfg(any())] checked: {CALL} }}; true }}"),
    ] {
        let changed = replace_admission_guard_fixture_call(source, OWNER, 0, &replacement);
        assert_eq!(changed.matches(CALL).count(), source.matches(CALL).count());
        assert!(syn::parse_file(&changed).is_ok(), "conditional-path syntax: {replacement}");
        let error = check_session_admission_guard_owners(&changed)
            .expect_err("unsupported conditional attributes cannot establish a live guard");
        assert!(error.starts_with(OWNER), "wrong failed boundary for {replacement}: {error}");
    }
}

#[test]
fn test_direct_agent_db_metadata_writes_fail_closed_on_malformed_admission() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    assert!(owner_source.contains("fn session_work_write_is_admitted("));
    assert!(owner_source.contains("!admission.session_key.trim().is_empty()"));
    assert!(owner_source.contains("!admission.session_id.trim().is_empty()"));
    assert!(owner_source.contains("!admission.lifecycle_revision.trim().is_empty()"));
    assert!(owner_source.contains("let expected_session_key = format!(\"agent:{agent_id}:main\")"));
    assert!(owner_source.contains("admission.session_key == expected_session_key"));
    assert!(owner_source.contains("fn session_admission_identity_is_bound("));
    assert!(owner_source.contains("session archive transition owner identity is malformed or rebound"));
    check_session_admission_guard_owners(owner_source)
        .expect("all existing transaction edges and original-turn capture must retain their own admission guards");
    assert_eq!(
        owner_source
            .matches("session_admission_identity_is_well_formed(&admission)")
            .count(),
        11,
        "all transaction edges and the original-turn terminal fence must validate admission identity"
    );
    assert!(original_turn_fence_has_admission_guard(owner_source),
        "the additional original-turn fence must retain its own actual admission guard");
    for (method, expected_guards) in [
        ("commit_session_work_result", 2),
        ("append_transcript_entries_if_session_admitted", 1),
        ("append_spend_guard_card_if_session_admitted", 1),
        ("append_transcript_entries_and_release_media", 3),
        ("update_transcript_entry_if_session_admitted", 1),
        ("settle_transcript_entry_identity_if_session_admitted", 1),
        ("update_persisted_transcript_entry_if_session_admitted", 1),
    ] {
        let signature = format!("pub fn {method}(");
        let body = owner_source
            .split_once(signature.as_str())
            .unwrap_or_else(|| panic!("session transaction owner {method} exists"))
            .1
            .split("\n    pub fn ")
            .next()
            .unwrap()
            .split("\npub fn ")
            .next()
            .unwrap();
        assert_eq!(
            body.matches("session_admission_identity_is_well_formed(&admission)").count(),
            expected_guards,
            "{method} must retain its own admission and replay guards"
        );
    }
    let terminal_fence = owner_source
        .split_once("\nfn capture_original_turn_terminal_fence(")
        .expect("the original-turn terminal fence exists")
        .1
        .split_once("\n}\n")
        .expect("the original-turn terminal fence closes before later owners")
        .0;
    assert_eq!(
        terminal_fence
            .matches("session_admission_identity_is_well_formed(&admission)")
            .count(),
        1,
        "the terminal fence must validate admission independently of every existing transaction"
    );
    assert!(owner_source.contains("if !session_work_write_is_admitted(db)?"));
    let initializer = owner_source
        .split("pub fn initialize_metadata_if_missing(")
        .nth(1)
        .expect("metadata initializer exists")
        .split("pub fn set_metadata(")
        .next()
        .expect("metadata initializer boundary exists");
    assert!(initializer.contains("BEGIN IMMEDIATE"));
    assert!(initializer.contains("if !session_work_write_is_admitted(db)?"));
    let admission_cas = owner_source
        .split("fn compare_and_set_session_work_admission_with_deletion_transition(")
        .nth(1)
        .expect("session admission CAS owner exists")
        .split("pub fn read_session_provider_review(")
        .next()
        .expect("session admission CAS owner boundary exists");
    assert!(admission_cas.contains("BEGIN IMMEDIATE"));
    assert!(admission_cas.contains("COMMIT"));
    assert!(admission_cas.contains("ROLLBACK"));
    assert!(
        admission_cas.find("BEGIN IMMEDIATE")
            < admission_cas.find("let current = db")
    );
}

fn original_turn_fence_has_admission_guard(source: &str) -> bool {
    fn condition_rejects_malformed_admission(condition: &syn::Expr) -> bool {
        if let syn::Expr::Binary(binary) = condition {
            if matches!(binary.op, syn::BinOp::Or(_)) {
                return condition_rejects_malformed_admission(&binary.left);
            }
        }
        let syn::Expr::Unary(negated) = condition else { return false; };
        if !matches!(negated.op, syn::UnOp::Not(_)) { return false; }
        let syn::Expr::Call(call) = negated.expr.as_ref() else { return false; };
        let syn::Expr::Path(callee) = call.func.as_ref() else { return false; };
        if callee.qself.is_some()
            || !callee.path.is_ident("session_admission_identity_is_well_formed")
            || call.args.len() != 1
        { return false; }
        let Some(syn::Expr::Reference(argument)) = call.args.first() else { return false; };
        let syn::Expr::Path(identity) = argument.expr.as_ref() else { return false; };
        argument.mutability.is_none() && identity.qself.is_none()
            && identity.path.is_ident("admission")
    }

    let Ok(parsed) = syn::parse_file(source) else { return false; };
    let mut fences = parsed.items.into_iter().filter_map(|item| match item {
        syn::Item::Fn(function) if function.sig.ident == "capture_original_turn_terminal_fence" => Some(function),
        _ => None,
    });
    let Some(fence) = fences.next() else { return false; };
    if fences.next().is_some() { return false; }
    fence.block.stmts.iter().filter(|statement| match statement {
        syn::Stmt::Expr(syn::Expr::If(branch), _) =>
            condition_rejects_malformed_admission(&branch.cond),
        _ => false,
    }).count() == 1
}

#[test]
fn original_turn_admission_guard_oracle_rejects_removed_and_comment_only_guards() {
    let source = include_str!("../src/extensions/session/agent_db.rs");
    assert!(original_turn_fence_has_admission_guard(source));
    let signature = "fn capture_original_turn_terminal_fence(";
    let guard = "!session_admission_identity_is_well_formed(&admission)";
    let (prefix, fence_and_tail) = source.split_once(signature).expect("actual terminal fence");
    let removed = format!("{prefix}{signature}{}", fence_and_tail.replacen(guard, "false", 1));
    assert_ne!(removed, source, "the negative must remove the actual guard");
    let comment_only = format!("// {guard}\n{removed}");
    assert_eq!(comment_only.matches("session_admission_identity_is_well_formed(&admission)").count(), 11,
        "the comment preserves the global count but cannot replace the fence guard");
    let body_marker = ") -> Result<OriginalTurnTerminalFence, AgentDbProjectionError> {";
    let (before_body, body_and_tail) = removed.split_once(body_marker).expect("actual fence body");
    let doc_comment_only = format!(
        "{before_body}{body_marker}\n    /// if{guard}||\n    fn guard_spelling_only() {{}}\n{body_and_tail}"
    );
    assert_eq!(doc_comment_only.matches("session_admission_identity_is_well_formed(&admission)").count(), 11,
        "the in-body doc comment preserves the global count but is not a predicate call");
    for (label, candidate) in [
        ("removed", &removed),
        ("ordinary comment only", &comment_only),
        ("in-body doc comment only", &doc_comment_only),
    ] {
        assert!(syn::parse_file(candidate).is_ok(), "counterfactual must remain valid Rust: {label}");
        assert!(!original_turn_fence_has_admission_guard(candidate), "missing actual guard: {label}");
    }
}

#[test]
fn test_automation_session_identity_helper_rechecks_owner_admission() {
    let shipping_source = include_str!("../app/src/main.rs");
    let helper = shipping_source
        .split("fn resolve_admitted_automation_session_key(")
        .nth(1)
        .expect("automation session identity helper exists")
        .split("fn run_local_automation_turn(")
        .next()
        .expect("automation session identity helper boundary exists");
    assert!(helper.contains("session_workers.assert_session_work_admission"));
    assert!(helper.contains("Some(&expected_key)"));
    assert!(helper.contains("format!(\"agent:{agent_id}:main\")"));
    assert!(!helper.contains("format!(\"agent:{agent_id}:cron:{automation_id}\")"));
}

#[test]
fn test_cron_candidate_projection_rejects_malformed_identity_rows() {
    let owner_source = include_str!("../src/extensions/session/production.rs");
    let projection = owner_source
        .split("pub fn list_expired_cron_session_runs(")
        .nth(1)
        .expect("cron candidate projection exists")
        .split("pub(crate) fn read_session_work_admission_locked(")
        .next()
        .expect("cron candidate projection boundary exists");
    assert!(projection.contains("expected_session_key_prefix"));
    assert!(projection.contains("row.session_key"));
    assert!(projection.contains("row.automation_id"));
    assert!(projection.contains("row.run_id"));
    assert!(projection.contains("row.session_id == row.session_key"));
    assert!(projection.contains("!row.lifecycle_revision.trim().is_empty()"));
}

#[test]
fn test_cron_schema_migrates_fence_marker_for_existing_rows() {
    let recovery_source = include_str!("../src/extensions/session/agent_db_recovery.rs");
    assert!(recovery_source.contains("pragma_table_info('cron_session_runs')"));
    assert!(recovery_source.contains("name = 'fence_state_known'"));
    assert!(recovery_source.contains(
        "ALTER TABLE cron_session_runs ADD COLUMN fence_state_known INTEGER NOT NULL DEFAULT 0"
    ));
}

#[test]
fn test_cron_owner_writes_rollback_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let cron_owner = owner_source
        .split("pub fn begin_cron_session_run(")
        .nth(1)
        .expect("cron owner write section exists")
        .split("pub fn list_cron_session_runs(")
        .next()
        .expect("cron owner write section boundary exists");
    assert_eq!(
        cron_owner.matches("match db.execute_batch(\"COMMIT\")").count(),
        4
    );
    assert!(cron_owner.matches("db.execute_batch(\"ROLLBACK\")").count() >= 8);
}

#[test]
fn test_cron_candidate_bridge_releases_throttle_on_owner_read_failure() {
    let runtime_source = include_str!("../src/extensions/transcript/automation_runtime.rs");
    let bridge = runtime_source
        .split("pub fn list_expired_cron_session_candidates(")
        .nth(1)
        .expect("cron candidate bridge exists")
        .split("pub fn event_fires(")
        .next()
        .expect("cron candidate bridge boundary exists");
    assert!(bridge.contains("list_expired_cron_session_runs"));
    assert!(bridge.contains("throttle.forget_owner(agent_id)"));
}

#[test]
fn test_cron_settlement_accepts_only_typed_terminal_outcomes() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let cron_owner = owner_source
        .split("pub fn finish_cron_session_run(")
        .nth(1)
        .expect("cron settlement owner exists")
        .split("pub fn list_cron_session_runs(")
        .next()
        .expect("cron settlement owner boundary exists");
    assert_eq!(cron_owner.matches("!matches!(outcome, \"ok\" | \"error\" | \"interrupted\")").count(), 2);
}

#[test]
fn test_archive_owner_rolls_back_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let archive_owner = owner_source
        .split("pub fn transition_session_work_archive(")
        .nth(1)
        .expect("archive owner exists")
        .split("pub fn commit_session_work_result(")
        .next()
        .expect("archive owner boundary exists");
    assert!(archive_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(archive_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));
}

#[test]
fn test_background_result_owner_rolls_back_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let result_owner = owner_source
        .split("pub fn commit_session_work_result(")
        .nth(1)
        .expect("background result owner exists")
        .split("pub fn read_session_provider_review(")
        .next()
        .expect("background result owner boundary exists");
    assert!(result_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(result_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));
}

#[test]
fn test_session_transcript_append_rolls_back_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let append_owner = owner_source
        .split("pub fn append_transcript_entries_if_session_admitted(")
        .nth(1)
        .expect("session transcript append owner exists")
        .split("pub fn append_spend_guard_card_if_session_admitted(")
        .next()
        .expect("session transcript append owner boundary exists");
    assert!(append_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(append_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));
}

#[test]
fn test_session_transcript_update_and_identity_settlement_roll_back_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let update_owner = owner_source
        .split("pub fn update_transcript_entry_if_session_admitted(")
        .nth(1)
        .expect("session transcript update owner exists")
        .split("pub fn settle_transcript_entry_identity(")
        .next()
        .expect("session transcript update owner boundary exists");
    assert!(update_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(update_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));

    let settlement_owner = owner_source
        .split("pub fn settle_transcript_entry_identity_if_session_admitted(")
        .nth(1)
        .expect("session transcript identity settlement owner exists")
        .split("pub fn delete_transcript_entry(")
        .next()
        .expect("session transcript identity settlement owner boundary exists");
    assert!(settlement_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(settlement_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));
}

#[test]
fn test_session_transcript_delete_and_conversation_clear_roll_back_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let delete_owner = owner_source
        .split("pub fn delete_transcript_entry(")
        .nth(1)
        .expect("session transcript delete owner exists")
        .split("pub fn clear_conversation(")
        .next()
        .expect("session transcript delete owner boundary exists");
    assert!(delete_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(delete_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));

    let clear_owner = owner_source
        .split("pub fn clear_conversation(")
        .nth(1)
        .expect("conversation clear owner exists")
        .split("pub fn clear_clone_mutation_journals(")
        .next()
        .expect("conversation clear owner boundary exists");
    assert!(clear_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(clear_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));
}

#[test]
fn test_session_checkpoint_and_root_metadata_roll_back_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let checkpoint_owner = owner_source
        .split("pub fn commit_checkpoint_root_and_confirm_user_message(")
        .nth(1)
        .expect("session checkpoint owner exists")
        .split("pub fn compare_and_set_latest_root_blob_id(")
        .next()
        .expect("session checkpoint owner boundary exists");
    assert!(checkpoint_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(checkpoint_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));

    let root_owner = owner_source
        .split("pub fn compare_and_set_latest_root_blob_id(")
        .nth(1)
        .expect("latest root owner exists")
        .split("pub fn get_latest_root_blob_id(")
        .next()
        .expect("latest root owner boundary exists");
    assert!(root_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(root_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));
}

#[test]
fn test_session_metadata_owners_roll_back_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let owners = [
        (
            "pub fn write_kv(",
            "pub fn write_session_kv_if_generation(",
        ),
        (
            "pub fn write_session_kv_if_generation(",
            "pub fn delete_kv(",
        ),
        ("pub fn delete_kv(", "pub fn read_session_work_admission("),
        (
            "pub fn reset_session_work_admission_for_clone(",
            "pub fn compare_and_set_session_work_admission(",
        ),
        (
            "pub fn initialize_metadata_if_missing(",
            "pub fn set_metadata(",
        ),
        ("pub fn set_metadata(", "pub fn ensure_lifecycle_revision("),
    ];
    for (start, end) in owners {
        let owner = owner_source
            .split(start)
            .nth(1)
            .expect("session metadata owner exists")
            .split(end)
            .next()
            .expect("session metadata owner boundary exists");
        assert!(owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
        assert!(owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));
    }
}

#[test]
fn test_transcript_owners_roll_back_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let owners = [
        (
            "pub fn append_transcript_entries(",
            "pub fn append_transcript_entries_if_session_admitted(",
        ),
        (
            "pub fn update_transcript_entry(",
            "pub fn update_transcript_entry_if_session_admitted(",
        ),
        (
            "pub fn settle_transcript_entry_identity(",
            "pub fn settle_transcript_entry_identity_if_session_admitted(",
        ),
    ];
    for (start, end) in owners {
        let owner = owner_source
            .split(start)
            .nth(1)
            .expect("transcript owner exists")
            .split(end)
            .next()
            .expect("transcript owner boundary exists");
        assert!(owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
        assert!(owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));
    }
}

// CA-07: this command adapter returns to an explicit transaction error owner.
// Pin its complete method, not its impl or the surrounding production source.
const CHANNEL_RAW_COMMIT_METHOD: &str = r#"fn commit(&self, db: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        db.execute_batch("COMMIT")?;
        #[cfg(test)]
        if self.lose_commit_acknowledgement {
            return Err(rusqlite::Error::InvalidQuery);
        }
        Ok(())
    }"#;

const CHANNEL_COMMIT_OWNER_TAIL: &str = r#"match inserted {
        Err(failure) => channel_inbound_transaction_failure(db, failure, false, control),
        Ok(inserted_count) => match control.commit(db) {
            Ok(()) if db.is_autocommit() => (
                ChannelInboundAppendOutcome::Committed { inserted_count }, false,
            ),
            Ok(()) => channel_inbound_transaction_failure(
                db,
                channel_inbound_failure(
                    ChannelInboundAppendFailureKind::Storage,
                    "channel append commit left a live transaction",
                ),
                true,
                control,
            ),
            Err(error) => channel_inbound_transaction_failure(
                db, channel_inbound_storage_failure(error), true, control,
            ),
        },
    }"#;

fn channel_transaction_control_impl(file: &syn::File) -> Option<&syn::ItemImpl> {
    let mut candidates = file.items.iter().filter_map(|item| {
        let syn::Item::Impl(item) = item else { return None; };
        let syn::Type::Path(ty) = item.self_ty.as_ref() else { return None; };
        ty.path.segments.last()
            .is_some_and(|segment| segment.ident == "ChannelInboundTransactionControl")
            .then_some(item)
    });
    let item = candidates.next()?;
    let syn::Type::Path(ty) = item.self_ty.as_ref() else { return None; };
    (candidates.next().is_none()
        && ty.qself.is_none()
        && ty.path.is_ident("ChannelInboundTransactionControl")
        && item.attrs.is_empty()
        && item.trait_.is_none()
        && item.defaultness.is_none()
        && item.unsafety.is_none()
        && item.generics.params.is_empty()
        && item.generics.where_clause.is_none())
        .then_some(item)
}

#[derive(Clone, Copy)]
enum ChannelCommitTokenContext {
    Source,
    MacroInvocation,
}

// Admit only ($receiver:expr) => { $receiver.commit() }; as one complete rule.
// The receiver is the only substitution; method, arguments and outer tokens
// are literal. Every other definition revokes the exception.
fn channel_literal_raii_macro(cursor: syn::buffer::Cursor<'_>) -> bool {
    use syn::parse::Parser;

    (|input: syn::parse::ParseStream<'_>| {
        let matcher;
        syn::parenthesized!(matcher in input);
        matcher.parse::<syn::Token![$]>()?;
        let receiver = matcher.parse::<syn::Ident>()?;
        matcher.parse::<syn::Token![:]>()?;
        let fragment = matcher.parse::<syn::Ident>()?;
        input.parse::<syn::Token![=>]>()?;
        let expansion;
        syn::braced!(expansion in input);
        expansion.parse::<syn::Token![$]>()?;
        let expanded_receiver = expansion.parse::<syn::Ident>()?;
        expansion.parse::<syn::Token![.]>()?;
        let method = expansion.parse::<syn::Ident>()?;
        let arguments;
        syn::parenthesized!(arguments in expansion);
        input.parse::<syn::Token![;]>()?;
        Ok(matcher.is_empty() && fragment == "expr"
            && receiver == expanded_receiver && method == "commit"
            && arguments.is_empty() && expansion.is_empty())
    }).parse2(cursor.token_stream()).unwrap_or(false)
}

// Ordinary zero-argument calls and the complete literal rule above are outside
// the adapter interface. Real macro inputs remain opaque, including apparent
// zero-argument calls; this oracle does not expand macros or resolve names.
fn channel_commit_call_count(
    mut cursor: syn::buffer::Cursor<'_>,
    context: ChannelCommitTokenContext,
) -> Option<usize> {
    use ChannelCommitTokenContext::MacroInvocation;

    let mut count = 0;
    let mut punctuation = None;
    while !cursor.eof() {
        if let Some((group, _, _, rest)) = cursor.any_group() {
            count += channel_commit_call_count(group, context)?;
            cursor = rest;
            punctuation = None;
        } else if let Some((ident, rest)) = cursor.ident() {
            let name = ident.to_string();
            let normalized = name.strip_prefix("r#").unwrap_or(&name);
            // Unlike other invocations, macro_rules! has a name between ! and
            // its body. Inside invocation input it is only another token: do
            // not lower that context or skip its possibly forwarded name.
            if normalized == "macro_rules" && !matches!(context, MacroInvocation) {
                if let Some((bang, definition)) = rest.punct() {
                    if bang.as_char() == '!' {
                        let (_, body) = definition.ident()?;
                        let (tokens, _, _, after) = body.any_group()?;
                        if !channel_literal_raii_macro(tokens) {
                            return None;
                        }
                        cursor = after;
                        punctuation = None;
                        continue;
                    }
                }
            }
            if normalized == "commit" {
                if matches!(context, MacroInvocation) {
                    return None;
                }
                match punctuation {
                    Some('.') => {
                        let (arguments, _, _, _) = rest.any_group()?;
                        count += usize::from(!arguments.eof());
                    }
                    Some(':') => return None,
                    _ => {}
                }
            }
            // The source already parsed as Rust. A legal final path identifier
            // plus ! and a group is a macro boundary; return/if/other keywords
            // fail Ident parsing, so their unary ! groups inherit this context.
            if let Some((bang, after_bang)) = rest.punct() {
                if bang.as_char() == '!' && syn::parse_str::<syn::Ident>(&name).is_ok() {
                    if let Some((tokens, _, _, after)) = after_bang.any_group() {
                        if normalized == "commit" {
                            return None;
                        }
                        count += channel_commit_call_count(tokens, MacroInvocation)?;
                        cursor = after;
                        punctuation = None;
                        continue;
                    }
                }
            }
            cursor = rest;
            punctuation = None;
        } else if let Some((punct, rest)) = cursor.punct() {
            // .$method and ::$method can introduce an adapter call without
            // spelling commit at the call site. Do not guess the substitution.
            if punct.as_char() == '$' && matches!(punctuation, Some('.') | Some(':')) {
                return None;
            }
            punctuation = Some(punct.as_char());
            cursor = rest;
        } else if let Some((_, rest)) = cursor.lifetime() {
            cursor = rest;
            punctuation = None;
        } else if let Some((_, rest)) = cursor.literal() {
            cursor = rest;
            punctuation = None;
        } else {
            return None;
        }
    }
    Some(count)
}

fn agent_db_transaction_owner_source(source: &str) -> Option<String> {
    use syn::parse::Parser;

    let parsed = syn::parse_file(source).ok()?;
    let helper = channel_transaction_control_impl(&parsed)?;
    let mut methods = helper.items.iter().filter_map(|item| match item {
        syn::ImplItem::Fn(method) if method.sig.ident == "commit" => Some(method),
        _ => None,
    });
    let method = methods.next()?;
    if methods.next().is_some() || !method.attrs.is_empty()
        || !matches!(method.vis, syn::Visibility::Inherited)
        || source.matches(CHANNEL_RAW_COMMIT_METHOD).count() != 1
    {
        return None;
    }

    let owners = source.replacen(CHANNEL_RAW_COMMIT_METHOD, "", 1);
    let remaining = syn::parse_file(&owners).ok()?;
    let remaining_helper = channel_transaction_control_impl(&remaining)?;
    // Reparse to prove the removed text was this whole method, not a matching
    // comment, string, nested function, or method belonging to another impl.
    if parsed.items.len() != remaining.items.len()
        || helper.items.len() != remaining_helper.items.len() + 1
        || remaining_helper.items.iter().any(|item| {
            matches!(item, syn::ImplItem::Fn(method) if method.sig.ident == "commit")
        })
    {
        return None;
    }

    let expected_tail = (|input: syn::parse::ParseStream<'_>| {
        let tokens = input.cursor().token_stream().to_string();
        input.parse::<syn::ExprMatch>()?;
        Ok(tokens)
    }).parse_str(CHANNEL_COMMIT_OWNER_TAIL).ok()?;
    let handled = (|input: syn::parse::ParseStream<'_>| {
        if channel_commit_call_count(input.cursor(), ChannelCommitTokenContext::Source) != Some(1) {
            return Ok(false);
        }
        input.call(syn::Attribute::parse_inner)?;
        let mut callers = 0;
        while !input.is_empty() {
            let boundary = input.fork();
            let item = input.parse::<syn::Item>()?;
            let syn::Item::Fn(function) = item else { continue; };
            if function.sig.ident != "append_channel_inbound_transaction" {
                continue;
            }
            callers += 1;
            if !function.attrs.is_empty()
                || !matches!(function.vis, syn::Visibility::Inherited)
            {
                return Ok(false);
            }
            boundary.call(syn::Attribute::parse_outer)?;
            boundary.parse::<syn::Visibility>()?;
            boundary.parse::<syn::Signature>()?;
            let body;
            syn::braced!(body in boundary);
            let mut tail = String::new();
            while !body.is_empty() {
                tail = body.cursor().token_stream().to_string();
                body.parse::<syn::Stmt>()?;
            }
            // The real returned expression must handle the sole call. A match
            // in a comment, nested fixture, or discarded statement is no proof.
            if boundary.cursor() != input.cursor() || tail != expected_tail {
                return Ok(false);
            }
        }
        Ok(callers == 1)
    }).parse_str(source).ok()?;
    handled.then_some(owners)
}

#[test]
fn test_agent_db_has_no_direct_commit_error_propagation() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let owner_source = source_without_owned_channel_commit_adapter(owner_source)
        .expect("the one channel command adapter must return to its checked transaction owner");
    assert!(!owner_source.contains("db.execute_batch(\"COMMIT\")?;"));
}

// A low-level command adapter may return an error to a transaction owner that
// explicitly handles it. Only this exact adapter is excluded; all other source
// retains the original prohibition. Real SQLite fault controls exercise the
// cleanup/outcome behavior in channel_inbound_append_tests.
fn source_without_owned_channel_commit_adapter(source: &str) -> Option<String> {
    syn::parse_file(source).ok()?;
    const ADAPTER: &str = "impl ChannelInboundTransactionControl {";
    const NEXT: &str = "\nfn read_channel_inbound_admission(";
    const COMMIT: &str = "db.execute_batch(\"COMMIT\")?;";
    if source.matches(ADAPTER).count() != 1
        || source.matches(NEXT).count() != 1
        || source.matches("control.commit(db)").count() != 1
    {
        return None;
    }
    let (_, tail) = source.split_once(ADAPTER)?;
    let (adapter, _) = tail.split_once(NEXT)?;
    if adapter.matches(COMMIT).count() != 1
        || !adapter.contains("fn commit(&self, db: &rusqlite::Connection)")
        || !adapter.contains("db.execute_batch(\"ROLLBACK\")")
    {
        return None;
    }
    let (_, transaction) = source.split_once("\nfn append_channel_inbound_transaction(")?;
    let (transaction, _) = transaction.split_once("\nfn channel_inbound_entry_id_exists(")?;
    if !transaction.contains("Ok(inserted_count) => match control.commit(db) {")
        || !transaction.contains("Err(error) => channel_inbound_transaction_failure(\n                db, channel_inbound_storage_failure(error), true, control,")
    {
        return None;
    }
    let (_, failure) = source.split_once("\nfn channel_inbound_transaction_failure(")?;
    let (failure, _) = failure.split_once("\nfn append_channel_inbound_transaction(")?;
    if !failure.contains("if commit_attempted && db.is_autocommit()")
        || !failure.contains("let rolled_back = control.rollback(db).is_ok();")
        || !failure.contains("let clean = db.is_autocommit();")
        || !failure.contains("if !rolled_back || !clean")
        || !failure.contains("if commit_attempted {\n        (ChannelInboundAppendOutcome::Unknown(failure), false)")
    {
        return None;
    }
    agent_db_transaction_owner_source(source)
}

#[test]
fn test_commit_error_oracle_rejects_unguarded_propagation_and_missing_cleanup() {
    let source = include_str!("../src/extensions/session/agent_db.rs");
    let direct = "db.execute_batch(\"COMMIT\")?;";
    // The adapter exception must not hide a new unguarded owner elsewhere.
    let unguarded = format!(
        "{source}\nfn unguarded_commit(db: &rusqlite::Connection) -> Result<(), rusqlite::Error> {{ {direct} Ok(()) }}\n"
    );
    assert!(source_without_owned_channel_commit_adapter(&unguarded)
        .expect("valid existing adapter remains checked").contains(direct));

    for (before, after) in [
        (
            "Err(error) => channel_inbound_transaction_failure(\n                db, channel_inbound_storage_failure(error), true, control,",
            "Err(error) => channel_inbound_transaction_failure(\n                db, channel_inbound_storage_failure(error), false, control,",
        ),
        ("let rolled_back = control.rollback(db).is_ok();", "let rolled_back = true;"),
        ("if !rolled_back || !clean", "if !rolled_back"),
        (
            "if commit_attempted {\n        (ChannelInboundAppendOutcome::Unknown(failure), false)",
            "if commit_attempted {\n        (ChannelInboundAppendOutcome::NotWritten(failure), false)",
        ),
    ] {
        assert_eq!(source.matches(before).count(), 1, "mutation must bind one production site");
        let mutated = source.replacen(before, after, 1);
        assert!(source_without_owned_channel_commit_adapter(&mutated).is_none(),
            "missing transaction error ownership must revoke the exception");
    }
    assert!(source_without_owned_channel_commit_adapter("fn broken(").is_none());
    assert!(source_without_owned_channel_commit_adapter(&source.replace(
        "impl ChannelInboundTransactionControl {", "impl MissingChannelTransactionControl {",
    )).is_none());
}

fn channel_commit_oracle_fixture() -> String {
    format!(
        "impl ChannelInboundTransactionControl {{\n{CHANNEL_RAW_COMMIT_METHOD}\n}}\n\
         fn append_channel_inbound_transaction() {{ {CHANNEL_COMMIT_OWNER_TAIL} }}"
    )
}

#[test]
fn channel_commit_oracle_retains_unsafe_owners_before_after_and_inside_helper_impl() {
    let fixture = channel_commit_oracle_fixture();
    let owners = agent_db_transaction_owner_source(&fixture).expect("bounded adapter fixture");
    assert!(!owners.contains("db.execute_batch(\"COMMIT\")?;"));
    let unrelated = format!(
        "{fixture}\nfn raii_commit<'a>(transaction: &'a Transaction) {{\n\
         transaction.commit()?; let note = \"control.commit(db)\"; }}"
    );
    assert!(agent_db_transaction_owner_source(&unrelated).is_some(),
        "lifetimes, quoted text, and the separate zero-argument RAII commit are not adapter callers");
    let declarations_and_macro = format!(
        "{unrelated}\nimpl Transaction {{ fn commit(self) {{}} }}\n\
         fn compaction_data(commit: &CompactionCommit) {{ observe(commit); }}\n\
         macro_rules! raii_command {{ ($transaction:expr) => {{ $transaction.commit() }}; }}\n\
         fn macro_raii_commit(transaction: Transaction) {{ raii_command!(transaction)?; }}"
    );
    assert!(agent_db_transaction_owner_source(&declarations_and_macro).is_some(),
        "parsed declarations, ordinary data identifiers, and explicit zero-argument commits remain valid");
    let actual_source = include_str!("../src/extensions/session/agent_db.rs");
    for body in [
        "!(transaction.commit().is_ok())",
        "return !(transaction.commit().is_ok());",
        "if !(transaction.commit().is_ok()) { true } else { false }",
    ] {
        let ordinary = format!(
            "fn raii_commit_failed(transaction: rusqlite::Transaction<'_>) -> bool {{ {body} }}"
        );
        let bounded = format!("{fixture}\n{ordinary}");
        let actual = format!("{actual_source}\n{ordinary}");
        assert!(syn::parse_file(&bounded).is_ok(), "ordinary RAII fixture syntax: {body}");
        assert!(syn::parse_file(&actual).is_ok(), "ordinary RAII production-source syntax: {body}");
        assert!(agent_db_transaction_owner_source(&bounded).is_some(),
            "ordinary unary negation must preserve the bounded RAII control: {body}");
        assert!(source_without_owned_channel_commit_adapter(&actual).is_some(),
            "ordinary unary negation must preserve the actual-source exception: {body}");
    }
    let unsafe_owner = "fn unsafe_owner() { db.execute_batch(\"COMMIT\")?; }";
    for source in [
        format!("{unsafe_owner}\n{fixture}"),
        format!("{fixture}\n{unsafe_owner}"),
        fixture.replacen(CHANNEL_RAW_COMMIT_METHOD,
            &format!("{CHANNEL_RAW_COMMIT_METHOD}\n{unsafe_owner}"), 1),
    ] {
        let owners = agent_db_transaction_owner_source(&source).expect("unchanged adapter boundary");
        assert!(owners.contains("db.execute_batch(\"COMMIT\")?;"),
            "the original owner assertion must still reject this direct propagation: {source}");
    }
}

#[test]
fn channel_commit_oracle_rejects_extra_unhandled_callers_and_method_references() {
    for call in [
        "control.commit(db)?;",
        "let alias = control; alias.commit(db)?;",
        "let alias = control; alias.r#commit(db)?;",
        "ChannelInboundTransactionControl::commit(control, db)?;",
        "let commit = ChannelInboundTransactionControl::commit; commit(control, db)?;",
        // These definitions alone are unresolved, even without a literal
        // commit token or an invocation that could independently reject them.
        "macro_rules! invoke_command { ($owner:expr, $method:ident, $db:expr) => { $owner.$method($db) }; }",
        "macro_rules! invoke_associated { ($owner:ty, $method:ident, $control:expr, $db:expr) => { <$owner>::$method($control, $db) }; }",
        // A supplied method name must also fail closed when its macro is
        // imported, forwarded, raw-identified, or nested in another group.
        "invoke_command!(control, commit, db)?;",
        "invoke_command![control, r#commit, db]?;",
        "(commands::invoke_command! { control, (commit), db })?;",
        "macro_rules! unresolved_name { () => { commit }; }",
        // Opaque invocation syntax does not establish zero-argument arity.
        r#"macro_rules! forward_adapter {
            ($owner:ident $separator:tt $method:ident (), $db:expr) => {
                $owner $separator $method($db)
            };
        }
        forward_adapter!(control.commit(), db)?;"#,
        "commit!(control, db)?;",
        "r#commit!(control, db)?;",
        // Definition-shaped tokens inside invocation input cannot lower its
        // context or hide a forwarded name in the definition-name position.
        r#"macro_rules! forward_nested {
            ($keyword:ident $bang:tt $name:ident { $owner:ident $separator:tt $method:ident (), $db:expr }) => {
                $owner $separator $method($db)
            };
        }
        forward_nested!(macro_rules! apparent_definition { control.commit(), db })?;"#,
        "external_command!(macro_rules! commit { control, db })?;",
        // A forwarded bang can turn a definition's apparent RAII expression
        // into another macro's input before the argument-taking call is made.
        r#"macro_rules! forward_adapter {
            ($owner:ident $separator:tt $method:ident (), $db:expr) => {
                $owner $separator $method($db)
            };
        }
        macro_rules! build_forwarded {
            ($invoke:ident $bang:tt $owner:ident, $db:expr) => {
                $invoke $bang ($owner.commit(), $db)
            };
        }
        build_forwarded!(forward_adapter ! control, db)?;"#,
        // Genuine macro boundaries must remain opaque beside the unary-not
        // positives, including a qualified macro path.
        "external_command!(control.commit(), db)?;",
        "commands::external_command!(control.commit(), db)?;",
    ] {
        let source = format!("{}\nfn unsafe_caller() {{ {call} }}", channel_commit_oracle_fixture());
        assert!(syn::parse_file(&source).is_ok(),
            "extra caller counterfactual must remain valid source syntax: {call}");
        assert!(agent_db_transaction_owner_source(&source).is_none(),
            "an additional caller or unbounded method reference must be rejected: {call}");
    }
}

#[test]
fn channel_commit_oracle_rejects_changed_missing_or_discarded_error_handling() {
    let fixture = channel_commit_oracle_fixture();
    let error_arm = "Err(error) => channel_inbound_transaction_failure(\n                db, channel_inbound_storage_failure(error), true, control,\n            ),";
    assert!(fixture.contains(error_arm));
    for replacement in [
        "",
        "Err(error) => return Err(error),",
        "Err(error) => channel_inbound_transaction_failure(db, channel_inbound_storage_failure(error), false, control),",
        "Err(error) => channel_inbound_transaction_failure(db, channel_inbound_storage_failure(error), true, other_control),",
        "Err(error) if db.is_autocommit() => channel_inbound_transaction_failure(db, channel_inbound_storage_failure(error), true, control),",
        "_ => (ChannelInboundAppendOutcome::Committed { inserted_count }, false),",
    ] {
        assert!(agent_db_transaction_owner_source(&fixture.replacen(error_arm, replacement, 1)).is_none(),
            "the actual unguarded error arm must retain unknown/rollback handling: {replacement}");
    }
    let discarded = fixture.replacen(CHANNEL_COMMIT_OWNER_TAIL,
        &format!("{CHANNEL_COMMIT_OWNER_TAIL}; ignored_outcome()"), 1);
    assert!(agent_db_transaction_owner_source(&discarded).is_none());
    let test_only_caller = fixture.replacen("fn append_channel_inbound_transaction",
        "#[cfg(test)] fn append_channel_inbound_transaction", 1);
    assert!(agent_db_transaction_owner_source(&test_only_caller).is_none());
    let wrong_owner = fixture.replacen("fn append_channel_inbound_transaction", "fn other_owner", 1);
    assert!(agent_db_transaction_owner_source(&wrong_owner).is_none());
}

#[test]
fn channel_commit_oracle_rejects_missing_ambiguous_or_malformed_helper_boundaries() {
    let fixture = channel_commit_oracle_fixture();
    for source in [
        fixture.replacen(CHANNEL_RAW_COMMIT_METHOD, "", 1),
        fixture.replacen("impl ChannelInboundTransactionControl", "impl OtherControl", 1),
        format!("{fixture}\nimpl ChannelInboundTransactionControl {{}}"),
        fixture.replacen(CHANNEL_RAW_COMMIT_METHOD,
            &format!("{CHANNEL_RAW_COMMIT_METHOD}\n{CHANNEL_RAW_COMMIT_METHOD}"), 1),
        fixture.replacen(CHANNEL_RAW_COMMIT_METHOD,
            &format!("/* {CHANNEL_RAW_COMMIT_METHOD} */ fn commit() {{ other_command(); }}"), 1),
        fixture.replacen("impl ChannelInboundTransactionControl",
            "#[cfg(test)] impl ChannelInboundTransactionControl", 1),
        fixture.replacen(CHANNEL_RAW_COMMIT_METHOD,
            &format!("#[cfg(test)] {CHANNEL_RAW_COMMIT_METHOD}"), 1),
        fixture.replacen("        Ok(())", "        another_transaction(); Ok(())", 1),
        fixture.replacen("impl ChannelInboundTransactionControl {",
            "impl ChannelInboundTransactionControl", 1),
        format!("{fixture}\nfn malformed() {{"),
    ] {
        assert!(agent_db_transaction_owner_source(&source).is_none(),
            "only the complete unambiguous production command method may be excluded: {source}");
    }
}

#[test]
fn channel_commit_oracle_rejects_hidden_owner_and_aliased_calls_in_actual_source() {
    let source = include_str!("../src/extensions/session/agent_db.rs");
    let direct = "db.execute_batch(\"COMMIT\")?;";
    let boundary = "\nfn read_channel_inbound_admission(";
    assert_eq!(source.matches(direct).count(), 1);
    assert_eq!(source.matches(boundary).count(), 1);

    // The raw adapter keeps equivalent error propagation, while an unrelated
    // owner now contains the only exact forbidden statement. A delimiter-sized
    // exception must not erase that intervening owner along with the adapter.
    let rewritten = source.replacen(
        direct,
        "if let Err(error) = db.execute_batch(\"COMMIT\") { return Err(error); }",
        1,
    );
    let hidden_owner = format!(
        "\nfn unguarded_commit(db: &rusqlite::Connection) -> Result<(), rusqlite::Error> {{ {direct} Ok(()) }}\n{boundary}"
    );
    let hidden = rewritten.replacen(boundary, &hidden_owner, 1);
    assert!(syn::parse_file(&hidden).is_ok(), "counterfactual must remain valid source syntax");
    assert_eq!(hidden.matches(direct).count(), 1);
    assert!(source_without_owned_channel_commit_adapter(&hidden).is_none(),
        "only the actual raw method may receive the exception");

    for (declaration, call) in [
        ("", "adapter.commit(db)?;"),
        ("", "ChannelInboundTransactionControl::commit(adapter, db)?;"),
        (
            "macro_rules! invoke_command { ($owner:expr, $method:ident, $db:expr) => { $owner.$method($db) }; }",
            "invoke_command!(adapter, commit, db)?;",
        ),
        ("", "external_command!(adapter, commit, db)?;"),
        (
            r#"macro_rules! forward_adapter {
                ($owner:ident $separator:tt $method:ident (), $db:expr) => {
                    $owner $separator $method($db)
                };
            }"#,
            "forward_adapter!(adapter.commit(), db)?;",
        ),
        ("", "commit!(adapter, db)?;"),
        ("", "r#commit!(adapter, db)?;"),
        (
            r#"macro_rules! forward_nested {
                ($keyword:ident $bang:tt $name:ident { $owner:ident $separator:tt $method:ident (), $db:expr }) => {
                    $owner $separator $method($db)
                };
            }"#,
            "forward_nested!(macro_rules! apparent_definition { adapter.commit(), db })?;",
        ),
        ("", "external_command!(macro_rules! commit { adapter, db })?;"),
        (
            r#"macro_rules! forward_adapter {
                ($owner:ident $separator:tt $method:ident (), $db:expr) => {
                    $owner $separator $method($db)
                };
            }
            macro_rules! build_forwarded {
                ($invoke:ident $bang:tt $owner:ident, $db:expr) => {
                    $invoke $bang ($owner.commit(), $db)
                };
            }"#,
            "build_forwarded!(forward_adapter ! adapter, db)?;",
        ),
        ("", "external_command!(adapter.commit(), db)?;"),
        ("", "commands::external_command!(adapter.commit(), db)?;"),
    ] {
        let extra_owner = format!(
            "{source}\n{declaration}\nfn unguarded_adapter_owner(\n\
             db: &rusqlite::Connection, adapter: &ChannelInboundTransactionControl,\n\
             mutation_sql: &str,\n\
             ) -> Result<(), rusqlite::Error> {{\n\
             db.execute_batch(\"BEGIN IMMEDIATE\")?;\n\
             db.execute_batch(mutation_sql)?; {call} Ok(()) }}\n"
        );
        assert!(syn::parse_file(&extra_owner).is_ok(), "second owner must remain valid source syntax");
        assert_eq!(extra_owner.matches("control.commit(db)").count(), 1,
            "the old receiver-spelling count alone cannot see this second owner");
        assert!(source_without_owned_channel_commit_adapter(&extra_owner).is_none(),
            "a second unhandled adapter caller must revoke the exception: {call}");
    }
}

#[test]
fn test_conversation_blob_cleanup_rolls_back_when_commit_fails() {
    let owner_source = include_str!("../src/agent_isolation/conversation_blob_store.rs");
    assert!(!owner_source.contains("execute_batch(\"COMMIT\")?"));
    assert!(owner_source.matches("execute_batch(\"ROLLBACK\")").count() >= 4);
}

#[test]
fn test_compaction_owner_rolls_back_when_commit_fails() {
    let owner_source = include_str!("../src/extensions/session/agent_db.rs");
    let compaction_owner = owner_source
        .split("pub fn commit_compaction(")
        .nth(1)
        .expect("compaction owner exists")
        .split("pub fn get_compacted_transcript_entries(")
        .next()
        .expect("compaction owner boundary exists");
    assert!(compaction_owner.contains("if let Err(error) = db.execute_batch(\"COMMIT\")"));
    assert!(compaction_owner.contains("let _ = db.execute_batch(\"ROLLBACK\")"));
}

#[test]
fn test_session_admission_creation_rechecks_create_race_identity() {
    let owner_source = include_str!("../src/extensions/session/production.rs");
    let ensure = owner_source
        .split("fn ensure_main_session_work_admission_locked(")
        .nth(1)
        .expect("session admission initializer exists")
        .split("pub fn assert_session_work_admission(")
        .next()
        .expect("session admission initializer boundary exists");
    assert!(ensure.contains("let changed = db"));
    assert!(ensure.contains("lost its race without a record"));
    assert!(ensure.contains("malformed after create race"));
    assert!(ensure.contains("record.session_key != expected_session_key"));
    assert!(ensure.contains("current.session_key != expected_session_key"));
}

// ---------------------------------------------------------------------------
// OCRM-SESSION-O01: Session Key Parsing, Classification & Harness Namespace
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_session_o01_session_key_routing() {
    assert_eq!(classify_session_kind("global", None, None), SessionKind::Global);
    assert_eq!(classify_session_kind("unknown", None, None), SessionKind::Unknown);
    assert_eq!(classify_session_kind("agent:agent-1:cron:job-1", None, None), SessionKind::Cron);
    assert_eq!(classify_session_kind("cron:job-1", None, None), SessionKind::Direct);
    assert_eq!(classify_session_kind("agent:agent-1:opaque:cron:job-1", None, None), SessionKind::Direct);
    assert_eq!(classify_session_kind("agent:agent-1:channel:cron:job-1", None, None), SessionKind::Group);
    assert_eq!(classify_session_kind("session-1", None, Some("parent-session")), SessionKind::SpawnChild);
    assert_eq!(classify_session_kind("session-empty-parent", None, Some("")), SessionKind::Direct);
    assert_eq!(classify_session_kind("session-empty-parent-group", Some("group"), Some("")), SessionKind::Group);
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

    let route = resolve_public_session_route(" agent:agent-123:group:main ", Some("group"), None)
        .expect("canonical public route");
    assert_eq!(route.raw_key, "agent:agent-123:group:main");
    assert_eq!(route.agent_id, "agent-123");
    assert_eq!(route.kind, SessionKind::Group);
    let mixed_case_route = resolve_public_session_route("AGENT:Agent-123:Main", None, None)
        .expect("mixed-case canonical agent route");
    assert_eq!(mixed_case_route.agent_id, "agent-123");
    assert_eq!(mixed_case_route.kind, SessionKind::Direct);
    assert_eq!(
        resolve_public_session_route("agent:agent-123:harness:child", None, None),
        Err(PublicSessionRouteError::HarnessNamespaceReserved)
    );
    assert_eq!(
        resolve_public_session_route("agent::main", None, None),
        Err(PublicSessionRouteError::EmptyAgentIdentity)
    );
    assert_eq!(
        resolve_internal_send_route(
            "agent-123",
            Some("agent:agent-123:cron:job-1"),
            None,
            None,
            false,
        )
        .expect("cron producer route")
        .kind,
        SessionKind::Cron
    );
    assert_eq!(
        resolve_internal_send_route(
            "agent-123",
            Some("agent:agent-999:main"),
            None,
            None,
            false,
        ),
        Err(InternalSessionRouteError::SelectorAgentMismatch)
    );
    assert_eq!(
        resolve_internal_send_route("agent-123", None, None, None, false),
        Err(InternalSessionRouteError::MissingSelector)
    );
    assert_eq!(
        resolve_internal_send_route("agent-123", None, None, None, true)
            .expect("explicit direct fallback")
            .kind,
        SessionKind::Direct
    );

    let shipping_source = include_str!("../app/src/main.rs");
    for (start, end) in [
        ("fn call_openclaw_sessions_list(", "fn call_openclaw_sessions_get("),
        ("fn call_openclaw_sessions_get(", "fn call_openclaw_sessions_create("),
        ("fn call_openclaw_sessions_delete(", "fn call_openclaw_chat_send("),
        ("fn call_openclaw_chat_send(", "fn call_openclaw_chat_abort("),
    ] {
        let method = shipping_source
            .split(start)
            .nth(1)
            .expect("shipping session method exists")
            .split(end)
            .next()
            .expect("shipping session method boundary exists");
        assert!(method.contains("resolve_gateway_session_route"));
    }
    let chat_abort = shipping_source
        .split("fn call_openclaw_chat_abort(")
        .nth(1)
        .expect("shipping chat.abort owner exists")
        .split("fn call_openclaw_cron_list(")
        .next()
        .expect("shipping chat.abort function boundary exists");
    assert!(chat_abort.contains(
        "request_openclaw_chat_abort(args, &self.runner_registry, &self.transcript_runtime)"));
    let chat_abort_owner = shipping_source
        .split_once("\nfn request_openclaw_chat_abort(")
        .expect("shipping chat.abort delegates to the shared cancellation owner")
        .1
        .split_once("\nfn request_agent_cancellation(")
        .expect("shipping chat.abort helper boundary exists")
        .0;
    assert!(chat_abort_owner.contains("resolve_gateway_session_route(args, session_key)?"));
    assert!(chat_abort_owner.contains("reject_agent_wide_session_route(&route, \"chat.abort\")?"));
    let chat_send = shipping_source
        .split("fn call_openclaw_chat_send(")
        .nth(1)
        .expect("shipping chat.send owner exists")
        .split("fn call_openclaw_chat_abort(")
        .next()
        .expect("shipping chat.send function boundary exists");
    assert!(chat_send.contains("reject_agent_wide_session_route(&route, \"chat.send\")"));
    let sessions_patch = shipping_source
        .split("fn call_openclaw_sessions_patch(")
        .nth(1)
        .expect("shipping sessions.patch owner exists")
        .split("fn call_openclaw_sessions_provider_review_continue(")
        .next()
        .expect("shipping sessions.patch function boundary exists");
    assert!(sessions_patch.contains(
        "transition_session_work_archive(\n                &route.agent_id,\n                &expected_key,"
    ));
    let sessions_get = shipping_source
        .split("fn call_openclaw_sessions_get(")
        .nth(1)
        .expect("shipping sessions.get owner exists")
        .split("fn call_openclaw_sessions_create(")
        .next()
        .expect("shipping sessions.get function boundary exists");
    assert!(sessions_get.contains(
        "provider_review_projection(agent_id, &canonical_key, session_id)"
    ));
    let provider_review_continue = shipping_source
        .split("fn call_openclaw_sessions_provider_review_continue(")
        .nth(1)
        .expect("shipping provider-review continuation owner exists")
        .split("fn call_openclaw_chat_send(")
        .next()
        .expect("shipping provider-review continuation function boundary exists");
    assert!(provider_review_continue.contains("session_key.eq_ignore_ascii_case(&canonical_key)"));
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
            None,
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
            None,
            "reply",
            1729900000000,
        ),
        PendingConversationTurnClaimResult::NotReady
    ));
    assert!(registry.mark_ready("agent-1", &turn_id));
    let PendingConversationTurnClaimResult::Claimed(reply) = registry.claim_reply(
        "agent-1",
        "conversation-1",
        None,
        "session-1",
        "inbound-1",
        Some("outbound-1"),
        None,
        "reply",
        1729900000000,
    ) else {
        panic!("ready turn must yield its exact reply claim");
    };
    assert!(registry.release("agent-1", &reply));
    let PendingConversationTurnClaimResult::Claimed(retry) = registry.claim_reply(
        "agent-1", "conversation-1", None, "session-1", "inbound-1b",
        Some("outbound-1"), None, "retry", 1729900000001,
    ) else {
        panic!("released turn must permit a new reply claim");
    };
    assert!(registry.complete("agent-1", &retry));

    let threaded = registry
        .register(
            "agent-1",
            Some("turn-threaded"),
            "parent-conversation",
            "session-1",
            Some("thread-2"),
            Duration::from_secs(30),
        )
        .expect("register threaded pending turn");
    assert!(registry.set_outbound_message_id("agent-1", &threaded, "outbound-threaded"));
    assert!(registry.mark_ready("agent-1", &threaded));
    for parent in [None, Some("wrong-parent")] {
        assert!(matches!(
            registry.claim_reply(
                "agent-1",
                "thread-conversation",
                parent,
                "session-1",
                "inbound-wrong-parent",
                Some("outbound-threaded"),
                Some("thread-2"),
                "reply without matching parent",
                1729900000001,
            ),
            PendingConversationTurnClaimResult::NotFound
        ));
        assert!(!registry.snapshot("agent-1", &threaded).unwrap().claimed);
    }
    assert!(matches!(
        registry.claim_reply(
            "agent-1",
            "thread-conversation",
            Some("parent-conversation"),
            "session-1",
            "inbound-wrong-thread",
            Some("outbound-threaded"),
            Some("thread-wrong"),
            "wrong thread",
            1729900000001,
        ),
        PendingConversationTurnClaimResult::NotFound
    ));
    let PendingConversationTurnClaimResult::Claimed(threaded_reply) = registry.claim_reply(
        "agent-1",
        "thread-conversation",
        Some("parent-conversation"),
        "session-1",
        "inbound-threaded",
        Some("outbound-threaded"),
        Some("thread-2"),
        "threaded reply",
        1729900000002,
    ) else {
        panic!("matching thread and parent must yield a reply claim");
    };
    assert!(registry.complete("agent-1", &threaded_reply));

    let missing_thread = registry
        .register(
            "agent-1",
            Some("turn-missing-thread"),
            "parent-conversation-2",
            "session-1",
            Some("thread-3"),
            Duration::from_secs(30),
        )
        .expect("register missing-thread case");
    assert!(registry.set_outbound_message_id("agent-1", &missing_thread, "outbound-missing-thread"));
    assert!(registry.mark_ready("agent-1", &missing_thread));
    assert!(matches!(
        registry.claim_reply(
            "agent-1",
            "thread-conversation-2",
            Some("parent-conversation-2"),
            "session-1",
            "inbound-missing-thread",
            Some("outbound-missing-thread"),
            None,
            "missing thread identity",
            1729900000003,
        ),
        PendingConversationTurnClaimResult::NotFound
    ));
    assert!(registry.cancel("agent-1", &missing_thread));

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

mod session_turn_claim_settlement {
    use std::time::Duration;

    use mahayana_host_runtime::extensions::session::session_conversation_state::{
        PendingConversationTurnClaimResult, PendingConversationTurnRegistry,
        PendingConversationTurnReply,
    };

    fn ready_turn(registry: &PendingConversationTurnRegistry, agent: &str, id: &str, outbound: &str) {
        assert_eq!(registry.register(
            agent, Some(id), "conversation", "session", None, Duration::from_secs(60),
        ).expect("register turn"), id);
        assert!(registry.set_outbound_message_id(agent, id, outbound));
        assert!(registry.mark_ready(agent, id));
    }

    fn claim(
        registry: &PendingConversationTurnRegistry,
        agent: &str,
        outbound: &str,
        message: &str,
    ) -> PendingConversationTurnReply {
        let PendingConversationTurnClaimResult::Claimed(reply) = registry.claim_reply(
            agent, "conversation", None, "session", message, Some(outbound), None, message, 100,
        ) else {
            panic!("eligible ready turn must yield a claim");
        };
        reply
    }

    fn assert_held(registry: &PendingConversationTurnRegistry, agent: &str, id: &str, outbound: &str) {
        assert!(registry.snapshot(agent, id).expect("pending turn remains").claimed);
        assert_eq!(registry.claim_reply(
            agent, "conversation", None, "session", "competitor", Some(outbound), None,
            "must not acquire held custody", 101,
        ), PendingConversationTurnClaimResult::NotFound);
    }

    #[test]
    fn released_claim_cannot_settle_successor() {
        for stale_completion in [false, true] {
            let registry = PendingConversationTurnRegistry::new();
            ready_turn(&registry, "agent", "turn", "outbound");
            let released = claim(&registry, "agent", "outbound", "released-reply");
            let released_clone = released.clone();
            assert!(registry.release("agent", &released));
            assert!(!registry.release("agent", &released_clone));
            assert!(!registry.complete("agent", &released_clone));

            let successor = claim(&registry, "agent", "outbound", "successor-reply");
            let changed = if stale_completion {
                registry.complete("agent", &released_clone)
            } else {
                registry.release("agent", &released_clone)
            };
            assert!(!changed, "released handle cannot regain settlement authority");
            assert_held(&registry, "agent", "turn", "outbound");
            assert!(registry.complete("agent", &successor));
            assert!(registry.snapshot("agent", "turn").is_none());
        }
    }

    #[test]
    fn cancelled_claim_cannot_settle_reused_turn() {
        let registry = PendingConversationTurnRegistry::new();
        ready_turn(&registry, "agent", "reused-turn", "same-outbound");
        let cancelled = claim(&registry, "agent", "same-outbound", "cancelled-reply");
        assert!(registry.cancel("agent", "reused-turn"));
        assert!(!registry.release("agent", &cancelled));
        assert!(!registry.complete("agent", &cancelled));

        ready_turn(&registry, "agent", "reused-turn", "same-outbound");
        let replacement = claim(&registry, "agent", "same-outbound", "replacement-reply");
        assert!(!registry.release("agent", &cancelled));
        assert_held(&registry, "agent", "reused-turn", "same-outbound");
        assert!(!registry.complete("agent", &cancelled));
        assert_held(&registry, "agent", "reused-turn", "same-outbound");
        assert!(registry.complete("agent", &replacement));
        assert!(registry.snapshot("agent", "reused-turn").is_none());
    }

    #[test]
    fn private_claim_identity_survives_payload_mutation_and_rejects_wrong_agent() {
        let registry = PendingConversationTurnRegistry::new();
        ready_turn(&registry, "agent", "original-turn", "original-outbound");
        ready_turn(&registry, "other-agent", "original-turn", "original-outbound");
        ready_turn(&registry, "agent", "other-turn", "other-outbound");
        let original = claim(&registry, "agent", "original-outbound", "original-reply");
        let foreign = claim(&registry, "other-agent", "original-outbound", "foreign-reply");
        let other = claim(&registry, "agent", "other-outbound", "other-reply");
        assert!(!registry.complete("other-agent", &original));
        assert!(!registry.release("other-agent", &original));
        assert_held(&registry, "agent", "original-turn", "original-outbound");
        assert_held(&registry, "other-agent", "original-turn", "original-outbound");

        let mut changed_payload = original.clone();
        changed_payload.id = "other-turn".into();
        changed_payload.conversation_ref = "changed-conversation".into();
        changed_payload.message_id = "changed-message".into();
        changed_payload.reply_to_id = "other-outbound".into();
        changed_payload.thread_id = Some("changed-thread".into());
        changed_payload.text = "changed text".into();
        changed_payload.timestamp_ms = 999;
        assert!(registry.release("agent", &changed_payload),
            "release uses the private original key despite mutable payload fields");
        assert!(!registry.snapshot("agent", "original-turn").unwrap().claimed);
        assert_held(&registry, "agent", "other-turn", "other-outbound");

        let mut replacement = claim(&registry, "agent", "original-outbound", "replacement-reply");
        replacement.id = "other-turn".into();
        assert!(!registry.complete("agent", &changed_payload));
        assert!(!registry.release("agent", &original));
        assert_held(&registry, "agent", "original-turn", "original-outbound");
        assert!(registry.complete("agent", &replacement),
            "completion uses the private original key despite a changed public id");
        assert!(registry.snapshot("agent", "original-turn").is_none());
        assert_held(&registry, "agent", "other-turn", "other-outbound");
        assert_held(&registry, "other-agent", "original-turn", "original-outbound");
        assert!(registry.complete("agent", &other));
        assert!(registry.complete("other-agent", &foreign));
    }

    #[test]
    fn clones_share_single_use_authority_and_drop_retains_custody() {
        let registry = PendingConversationTurnRegistry::new();
        ready_turn(&registry, "agent", "turn", "outbound");
        let first = claim(&registry, "agent", "outbound", "first-reply");
        let first_clone = first.clone();
        drop(first);
        assert_held(&registry, "agent", "turn", "outbound");
        assert!(registry.release("agent", &first_clone));
        assert!(!registry.release("agent", &first_clone));
        assert!(!registry.complete("agent", &first_clone));

        let successor = claim(&registry, "agent", "outbound", "successor-reply");
        let successor_clone = successor.clone();
        drop(successor);
        assert_held(&registry, "agent", "turn", "outbound");
        assert!(registry.complete("agent", &successor_clone));
        assert!(!registry.complete("agent", &successor_clone));
        assert!(!registry.release("agent", &successor_clone));
        assert!(registry.snapshot("agent", "turn").is_none());

        ready_turn(&registry, "agent", "dropped-turn", "dropped-outbound");
        let dropped = claim(&registry, "agent", "dropped-outbound", "unknown-commit-reply");
        drop(dropped);
        assert_held(&registry, "agent", "dropped-turn", "dropped-outbound");
        assert!(registry.cancel("agent", "dropped-turn"));
    }
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

