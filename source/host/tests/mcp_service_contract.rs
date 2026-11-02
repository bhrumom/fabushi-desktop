use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::mcp::mcp_service::{
    BoxServerStatus, CatalogField, CatalogPlugin, CatalogSkill, EffectivePlugin, McpHostService,
    McpManagerBackend, McpServerSummary, PluginSkillsPort, to_auth_result, to_installed_server,
    to_plugin_summary,
};
use serde_json::{Value, json};

fn server(status: &str) -> McpServerSummary {
    McpServerSummary {
        id: "s1".into(),
        name: "Calendar".into(),
        server_identifier: "calendar".into(),
        account_key: "default".into(),
        plugin_id: Some("p1".into()),
        is_team_server: false,
        status: status.into(),
        status_detail: None,
        transport: "stdio".into(),
        tool_count: 2,
        disabled_tool_count: Some(1),
        custom_instructions: "be concise".into(),
    }
}

fn plugin() -> CatalogPlugin {
    CatalogPlugin {
        id: "p1".into(),
        name: "calendar".into(),
        display_name: Some("Calendar".into()),
        description: Some("calendar tools".into()),
        category: Some("productivity".into()),
        fields: vec![CatalogField {
            key: "token".into(),
            label: "Token".into(),
            hint: "secret".into(),
            is_required: true,
            is_secret: true,
        }],
        connector_count: 1,
        skills: vec![CatalogSkill {
            name: "Schedule".into(),
            description: Some("schedule events".into()),
            source_url: Some("https://github.com/o/r/blob/main/skills/s/SKILL.md".into()),
        }],
    }
}

#[derive(Default)]
struct BackendState {
    catalog_forced: Vec<bool>,
    box_calls: Vec<(Vec<String>, bool)>,
    notes: Vec<(String, String)>,
    disposed: usize,
    uninstall_result: Option<Result<(bool, Option<String>), String>>,
    uninstall_ids: Vec<String>,
}
struct Backend(Arc<Mutex<BackendState>>);
impl McpManagerBackend for Backend {
    fn list_servers(&self) -> Result<Vec<McpServerSummary>, String> {
        Ok(vec![server("connected")])
    }
    fn list_catalog(&self, force_refresh: bool) -> Result<Vec<CatalogPlugin>, String> {
        self.0.lock().unwrap().catalog_forced.push(force_refresh);
        Ok(vec![plugin()])
    }
    fn list_effective_plugins(&self) -> Result<Vec<EffectivePlugin>, String> {
        Ok(vec![EffectivePlugin {
            plugin_id: "p1".into(),
            install_mode: Some("user".into()),
            is_enabled: true,
            has_team_configured_variables: false,
        }])
    }
    fn uninstall_plugin(&self, plugin_id: &str) -> Result<(bool, Option<String>), String> {
        let mut state = self.0.lock().unwrap();
        state.uninstall_ids.push(plugin_id.to_string());
        state.uninstall_result.clone().unwrap_or(Ok((true, None)))
    }
    fn install_plugin(
        &self,
        _plugin_id: &str,
        _values: &HashMap<String, String>,
    ) -> Result<Vec<McpServerSummary>, String> {
        Ok(vec![server("connected")])
    }
    fn add_server(&self, _name: &str, _config_json: &str) -> Result<Vec<McpServerSummary>, String> {
        Ok(vec![server("connected")])
    }
    fn remove_server(&self, _server_id: &str) -> Result<(bool, Option<String>, Vec<McpServerSummary>), String> {
        Ok((true, None, Vec::new()))
    }
    fn restart_servers(&self) -> Result<Vec<McpServerSummary>, String> {
        Ok(vec![server("connected")])
    }
    fn set_server_custom_instructions(&self, _server_id: &str, instructions: &str) -> Result<Vec<McpServerSummary>, String> {
        let mut updated = server("connected");
        updated.custom_instructions = instructions.to_string();
        Ok(vec![updated])
    }
    fn authenticate_server(
        &self,
        _server_id: &str,
        _account_key: &str,
        _requesting_agent_id: Option<&str>,
        force_reauth: bool,
    ) -> Result<Value, String> {
        Ok(json!({
            "status": if force_reauth { "started" } else { "already-authenticated" },
            "serverName": "Calendar",
            "authorizationUrl": if force_reauth { Some("https://auth") } else { None::<&str> },
        }))
    }
    fn logout_account(&self, _server_id: &str, _account_key: &str) -> Result<Vec<McpServerSummary>, String> {
        Ok(vec![server("needsAuth")])
    }
    fn remove_account(&self, _server_id: &str, _account_key: &str) -> Result<Vec<McpServerSummary>, String> {
        Ok(Vec::new())
    }
    fn rename_account(&self, _server_id: &str, _account_key: &str, _new_account_key: &str) -> Result<Vec<McpServerSummary>, String> {
        Ok(vec![server("connected")])
    }
    fn list_box_servers(
        &self,
        ids: &[String],
        kick_only: bool,
    ) -> Result<Vec<BoxServerStatus>, String> {
        self.0
            .lock()
            .unwrap()
            .box_calls
            .push((ids.to_vec(), kick_only));
        Ok(ids
            .iter()
            .map(|id| BoxServerStatus {
                server_identifier: id.clone(),
                status: if kick_only {
                    "loading".into()
                } else {
                    "connected".into()
                },
                status_detail: None,
                tool_count: 2,
            })
            .collect())
    }
    fn note_auth_completed_elsewhere(&self, server_id: &str, account_key: &str) {
        self.0
            .lock()
            .unwrap()
            .notes
            .push((server_id.into(), account_key.into()));
    }
    fn dispose(&self) -> Result<(), String> {
        self.0.lock().unwrap().disposed += 1;
        Ok(())
    }
}

#[derive(Default)]
struct SkillsState {
    syncs: Vec<String>,
    removed: Vec<Vec<String>>,
    confirmed_uninstalls: Vec<String>,
    observation_error: Option<String>,
}
struct Skills(Arc<Mutex<SkillsState>>);
impl PluginSkillsPort for Skills {
    fn sync(&self, trigger: &str) -> Result<Vec<Value>, String> {
        self.0.lock().unwrap().syncs.push(trigger.into());
        Ok(vec![json!({"ok":true})])
    }
    fn status(&self) -> Value {
        json!({"authBlocked":[]})
    }
    fn observe_confirmed_uninstall(&self, plugin_id: &str) -> Result<(), String> {
        let mut state = self.0.lock().unwrap();
        state.confirmed_uninstalls.push(plugin_id.to_string());
        match state.observation_error.as_ref() {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
    fn remove_live_references(&self, urls: &[String]) {
        self.0.lock().unwrap().removed.push(urls.to_vec());
    }
}

#[test]
fn frozen_projection_helpers_preserve_optional_fields_and_install_semantics() {
    let installed = to_installed_server(&server("connected"));
    assert_eq!(installed["pluginId"], "p1");
    assert_eq!(installed["disabledToolCount"], 1);
    assert_eq!(
        to_auth_result("started", "Calendar", Some("https://auth"), None)["kind"],
        "started"
    );
    let summary = to_plugin_summary(
        &plugin(),
        Some(&[EffectivePlugin {
            plugin_id: "p1".into(),
            install_mode: Some("team".into()),
            is_enabled: true,
            has_team_configured_variables: true,
        }]),
        &[server("connected")],
    );
    assert_eq!(summary["isInstalled"], true);
    assert_eq!(summary["installMode"], "team");
}

#[test]
fn live_server_registry_resolves_audit_transport_and_fails_closed_for_unknown_servers() {
    let backend_state = Arc::new(Mutex::new(BackendState::default()));
    let service = McpHostService::new(Arc::new(Backend(backend_state)), None);
    assert_eq!(
        service.transport_for_server_identifier("calendar").unwrap(),
        "stdio"
    );
    assert_eq!(
        service.transport_for_server_identifier("missing").unwrap(),
        "unknown"
    );
}

#[test]
fn install_and_uninstall_drive_plugin_skill_sync_and_live_ref_retirement() {
    let backend_state = Arc::new(Mutex::new(BackendState::default()));
    let skills_state = Arc::new(Mutex::new(SkillsState::default()));
    let service = McpHostService::new(
        Arc::new(Backend(backend_state)),
        Some(Arc::new(Skills(skills_state.clone()))),
    );
    service.install_plugin("p1", &HashMap::new()).unwrap();
    service.uninstall_plugin("p1").unwrap();
    assert_eq!(
        skills_state.lock().unwrap().syncs,
        vec!["install", "uninstall"]
    );
    assert_eq!(skills_state.lock().unwrap().removed.len(), 2);
    assert!(skills_state.lock().unwrap().removed[0][0].contains("SKILL.md"));
    assert_eq!(skills_state.lock().unwrap().confirmed_uninstalls, vec!["p1"]);
}

#[test]
fn box_status_followup_is_deduped_and_publishes_terminal_update() {
    let backend_state = Arc::new(Mutex::new(BackendState::default()));
    let service = McpHostService::new(Arc::new(Backend(backend_state.clone())), None);
    let updates = Arc::new(Mutex::new(Vec::<BoxServerStatus>::new()));
    let id = service.subscribe_to_servers_updated({
        let updates = updates.clone();
        Arc::new(move |servers| updates.lock().unwrap().extend(servers))
    });
    service.list_box_servers(&["calendar".into()]).unwrap();
    service.list_box_servers(&["calendar".into()]).unwrap();
    service.run_status_followup("calendar").unwrap();
    service.run_status_followup("calendar").unwrap();
    assert_eq!(updates.lock().unwrap().len(), 1);
    assert_eq!(updates.lock().unwrap()[0].status, "connected");
    assert_eq!(backend_state.lock().unwrap().box_calls.len(), 3);
    service.unsubscribe_servers_updated(id);
}

#[test]
fn service_dispose_is_idempotent_and_blocks_late_listener_delivery() {
    let backend_state = Arc::new(Mutex::new(BackendState::default()));
    let service = McpHostService::new(Arc::new(Backend(backend_state.clone())), None);
    let events = Arc::new(Mutex::new(0usize));
    let _unsubscribe = service.subscribe_to_auth_completion({
        let events = events.clone();
        Arc::new(move |_| *events.lock().unwrap() += 1)
    });
    service.emit_auth_completion(json!({"status":"ok"}));
    assert_eq!(*events.lock().unwrap(), 1);
    service.note_auth_completed_elsewhere("s", "a");
    service.dispose().unwrap();
    service.dispose().unwrap();
    service.emit_auth_completion(json!({"status":"late"}));
    assert_eq!(*events.lock().unwrap(), 1);
    assert_eq!(backend_state.lock().unwrap().disposed, 1);
    assert_eq!(
        backend_state.lock().unwrap().notes,
        vec![("s".into(), "a".into())]
    );
}


#[test]
fn full_lifecycle_surface_matches_frozen_grok_management_semantics() {
    let backend_state = Arc::new(Mutex::new(BackendState::default()));
    let service = McpHostService::new(Arc::new(Backend(backend_state)), None);

    assert_eq!(service.add_server("custom", r#"{"url":"https://example.test/mcp"}"#).unwrap().len(), 1);
    let removed = service.remove_server("s1").unwrap();
    assert_eq!(removed["removed"], true);
    assert_eq!(removed["servers"].as_array().unwrap().len(), 0);
    assert_eq!(service.restart_servers().unwrap().len(), 1);
    assert_eq!(
        service.set_server_custom_instructions("s1", "reply in threads").unwrap()[0]["customInstructions"],
        "reply in threads"
    );
    assert_eq!(
        service.authenticate_server("s1", "default", Some("agent-a"), true).unwrap()["kind"],
        "started"
    );
    assert_eq!(service.logout_account("s1", "default").unwrap()[0]["status"], "needsAuth");
    assert!(service.remove_account("s1", "default").unwrap().is_empty());
    assert_eq!(service.rename_account("s1", "default", "work").unwrap().len(), 1);
}

#[test]
fn uninstall_observation_only_follows_confirmed_removal_and_surfaces_owner_failure() {
    for outcome in [
        Ok((false, Some("team-required".to_string()))),
        Err("backend result unavailable".to_string()),
    ] {
        let backend_state = Arc::new(Mutex::new(BackendState {
            uninstall_result: Some(outcome.clone()),
            ..BackendState::default()
        }));
        let skills_state = Arc::new(Mutex::new(SkillsState::default()));
        let service = McpHostService::new(
            Arc::new(Backend(backend_state.clone())),
            Some(Arc::new(Skills(skills_state.clone()))),
        );
        let result = service.uninstall_plugin("9");
        match outcome {
            Ok((removed, reason)) => assert_eq!(result.unwrap(), json!({
                "removed": removed, "reason": reason,
            })),
            Err(error) => assert_eq!(result.unwrap_err(), error),
        }
        assert_eq!(backend_state.lock().unwrap().uninstall_ids, vec!["9"]);
        let skills = skills_state.lock().unwrap();
        assert!(skills.confirmed_uninstalls.is_empty());
        assert!(skills.syncs.is_empty());
        assert!(skills.removed.is_empty());
    }

    let backend_state = Arc::new(Mutex::new(BackendState::default()));
    let skills_state = Arc::new(Mutex::new(SkillsState {
        observation_error: Some("owner retired".into()),
        ..SkillsState::default()
    }));
    let service = McpHostService::new(
        Arc::new(Backend(backend_state.clone())),
        Some(Arc::new(Skills(skills_state.clone()))),
    );
    let error = service.uninstall_plugin("9").unwrap_err();
    assert!(error.contains("uninstall confirmed"));
    assert!(error.contains("owner retired"));
    assert_eq!(backend_state.lock().unwrap().uninstall_ids, vec!["9"]);
    let skills = skills_state.lock().unwrap();
    assert_eq!(skills.confirmed_uninstalls, vec!["9"]);
    assert!(skills.removed.is_empty(), "cleanup must not precede local observation");
    assert!(skills.syncs.is_empty(), "refresh must not precede local observation");
}

#[test]
fn uninstall_without_skills_owner_reports_confirmed_removal_and_preserves_unconfirmed_results() {
    for outcome in [
        Ok((true, None)),
        Ok((false, Some("team-required".to_string()))),
        Err("backend result unavailable".to_string()),
    ] {
        let backend_state = Arc::new(Mutex::new(BackendState {
            uninstall_result: Some(outcome.clone()),
            ..BackendState::default()
        }));
        let service = McpHostService::new(Arc::new(Backend(backend_state.clone())), None);
        let result = service.uninstall_plugin("9");
        match outcome {
            Ok((true, _)) => assert_eq!(
                result.unwrap_err(),
                "plugin uninstall confirmed, but local skill admission could not be retired: plugin skills owner is unavailable",
            ),
            Ok((removed, reason)) => assert_eq!(result.unwrap(), json!({
                "removed": removed, "reason": reason,
            })),
            Err(error) => assert_eq!(result.unwrap_err(), error),
        }
        assert_eq!(backend_state.lock().unwrap().uninstall_ids, vec!["9"]);
    }
}

#[test]
fn desktop_dispatch_validates_exact_arguments_before_backend_or_skills_calls() {
    let backend_state = Arc::new(Mutex::new(BackendState::default()));
    let skills_state = Arc::new(Mutex::new(SkillsState::default()));
    let service = McpHostService::new(
        Arc::new(Backend(backend_state.clone())),
        Some(Arc::new(Skills(skills_state.clone()))),
    );
    assert!(service.dispatch_desktop_command("unrelated", &Value::Null).is_none());
    for args in [
        Value::Null, json!([]), json!("9"), json!({}), json!({"id":"9"}),
        json!({"pluginId":9}), json!({"pluginId":null}),
        json!({"pluginId":"9","removed":true}),
        json!({"pluginId":"9","state":{"servers":[]}}),
    ] {
        assert!(service.dispatch_desktop_command("uninstallMcpPlugin", &args).unwrap().is_err());
    }
    for method in ["refreshMcp", "syncPluginSkills", "getPluginSyncStatus"] {
        for args in [Value::Null, json!([]), json!(""), json!({"completed":true})] {
            assert!(service.dispatch_desktop_command(method, &args).unwrap().is_err());
        }
    }
    assert!(backend_state.lock().unwrap().catalog_forced.is_empty());
    assert!(backend_state.lock().unwrap().uninstall_ids.is_empty());
    assert!(skills_state.lock().unwrap().confirmed_uninstalls.is_empty());
    assert!(skills_state.lock().unwrap().syncs.is_empty());
    assert!(skills_state.lock().unwrap().removed.is_empty());

    let missing = McpHostService::new(Arc::new(Backend(backend_state.clone())), None);
    for method in ["refreshMcp", "syncPluginSkills", "getPluginSyncStatus"] {
        assert!(missing.dispatch_desktop_command(method, &json!({})).unwrap()
            .unwrap_err().contains("owner is unavailable"));
    }
    service.dispose().unwrap();
    for (method, args) in [
        ("uninstallMcpPlugin", json!({"pluginId":"9"})),
        ("refreshMcp", json!({})),
        ("syncPluginSkills", json!({})),
        ("getPluginSyncStatus", json!({})),
    ] {
        assert!(service.dispatch_desktop_command(method, &args).unwrap()
            .unwrap_err().contains("closed"));
    }
    assert!(service.uninstall_plugin("9").unwrap_err().contains("closed"));
    assert!(service.dispatch_desktop_command("unrelated", &json!({})).is_none());
    assert!(backend_state.lock().unwrap().catalog_forced.is_empty());
    assert!(backend_state.lock().unwrap().uninstall_ids.is_empty());
    assert!(skills_state.lock().unwrap().syncs.is_empty());
}

#[test]
fn both_uninstall_entrypoints_reject_identity_aliases_without_remote_io() {
    let backend_state = Arc::new(Mutex::new(BackendState {
        uninstall_result: Some(Ok((false, Some("still-present".into())))),
        ..BackendState::default()
    }));
    let skills_state = Arc::new(Mutex::new(SkillsState::default()));
    let service = McpHostService::new(
        Arc::new(Backend(backend_state.clone())),
        Some(Arc::new(Skills(skills_state.clone()))),
    );
    for id in [
        "", " ", " 9", "9 ", "\u{a0}9", "9\n", "9\t0", "9\u{7f}", "9\u{85}",
        "01", "00", "0009", "\u{feff}", "\u{feff}9", "9\u{feff}",
        "\u{feff}9\u{feff}", " \u{feff}9\u{feff}\u{a0}",
        "\u{feff}01", "01\u{feff}", "\u{feff}01\u{feff}",
        "\u{feff}official:github/review-tools", "official:github/review-tools\u{feff}",
    ] {
        assert!(service.uninstall_plugin(id).unwrap_err().contains("canonical"));
        assert!(service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":id}))
            .unwrap().unwrap_err().contains("canonical"));
    }
    assert!(backend_state.lock().unwrap().catalog_forced.is_empty());
    assert!(backend_state.lock().unwrap().uninstall_ids.is_empty());
    assert!(skills_state.lock().unwrap().confirmed_uninstalls.is_empty());
    for id in ["9", "184467440737095516160", "official:github/review-tools"] {
        assert_eq!(service.uninstall_plugin(id).unwrap(),
            json!({"removed":false,"reason":"still-present"}));
    }
    assert_eq!(backend_state.lock().unwrap().uninstall_ids,
        vec!["9", "184467440737095516160", "official:github/review-tools"]);
    assert_eq!(backend_state.lock().unwrap().catalog_forced, vec![false; 3]);
    assert!(skills_state.lock().unwrap().confirmed_uninstalls.is_empty());
    assert!(skills_state.lock().unwrap().syncs.is_empty());
    assert!(skills_state.lock().unwrap().removed.is_empty());
}

#[test]
fn tuple_backend_default_preserves_confirmation_without_fabricating_desktop_state() {
    let backend_state = Arc::new(Mutex::new(BackendState::default()));
    let skills_state = Arc::new(Mutex::new(SkillsState::default()));
    let service = McpHostService::new(
        Arc::new(Backend(backend_state.clone())),
        Some(Arc::new(Skills(skills_state.clone()))),
    );
    let error = service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"}))
        .unwrap().unwrap_err();
    assert!(error.contains("desktop MCP uninstall response is unavailable"));
    assert_eq!(backend_state.lock().unwrap().uninstall_ids, vec!["9"]);
    let skills = skills_state.lock().unwrap();
    assert_eq!(skills.confirmed_uninstalls, vec!["9"]);
    assert!(skills.removed.is_empty());
    assert!(skills.syncs.is_empty());
}

mod confirmed_uninstall_consumers {
    use super::*;
    use std::fs;

    use mahayana_host_runtime::extensions::managed_setup::managed_skills_cache::{
        ManagedSkill, get_managed_skills_dir, write_managed_skills_cache,
    };
    use mahayana_host_runtime::extensions::mcp::coordinator_relay::{
        CoordinatorMcpLifecycleRelay, CoordinatorMcpManagerBackend, CoordinatorPluginSkillsLoader,
        HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL, MCP_INSTALL_PLUGIN_METHOD,
        MCP_LIST_CATALOG_METHOD, MCP_LOAD_PLUGIN_SKILLS_METHOD, MCP_UNINSTALL_PLUGIN_METHOD,
    };
    use mahayana_host_runtime::extensions::mcp::plugin_skills::SandPluginSkillsService;
    use mahayana_host_runtime::gateway_server::GatewayEventHub;
    use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::{
        get_plugin_skills_cache_path, get_plugin_skills_dir,
    };
    use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
    use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
    use mahayana_host_runtime::extensions::transcript::workflow_commands::{
        WorkflowRunNowPlan, dispatch_workflow_command, expand_workflow_references,
        prepare_workflow_run_now,
    };
    use mahayana_host_runtime::runner::system_prompt_assembly::{
        AgentSkillPromptItem, append_budgeted_workflows_system_prompt,
    };
    use mahayana_host_runtime::workflows::workflow_library::WorkflowSpec;
    use mahayana_host_runtime::workflows::workflow_store::FileWorkflowStore;

    const LEGACY_SOURCE: &str = "https://github.com/example/private/blob/main/skills/private/SKILL.md";

    struct RemoteIo {
        loaded: Result<Value, String>,
        uninstall: Result<Value, String>,
        calls: Vec<(String, Value)>,
    }

    fn desktop_state() -> Value {
        json!({
            "servers":[{
                "id":"allowed-server", "name":"Allowed", "serverIdentifier":"allowed",
                "accountKey":"default", "pluginId":"10", "isTeamServer":true,
                "status":"connected", "transport":"http", "toolCount":2,
                "customInstructions":"retained instructions", "isRequired":true,
                "managedByTeamPluginPolicy":true, "rowServerIdentifier":"allowed-row",
                "futurePolicy":{"keep":true},
            }],
            "warnings":["retain the original warning"],
            "futureState":{"keep":true},
        })
    }

    fn reference(id: &str) -> String {
        json!({"type":"doc","content":[{
            "type":"workflowReference","attrs":{"id":id,"label":"Requested"}
        }]}).to_string()
    }

    fn skill_prompt(store: &FileWorkflowStore) -> String {
        let location = store.get_location().to_string_lossy().into_owned();
        let skills = store.list().into_iter()
            .filter(|row| row.trigger.is_none() && !row.disable_model_invocation)
            .map(|row| AgentSkillPromptItem {
                full_path: row.file_path.to_string_lossy().into_owned(),
                description: (!row.description.trim().is_empty()).then_some(row.description),
            }).collect::<Vec<_>>();
        let mut messages = Vec::new();
        append_budgeted_workflows_system_prompt(&mut messages, Some(&location), &skills, 200_000);
        assert!(!messages.is_empty());
        assert!(messages.iter().all(|message| message.role == "system"));
        messages.into_iter().map(|message| message.content).collect::<Vec<_>>().join("\n")
    }

    fn assert_run_body(workers: &Arc<ProductionSessionWorkers>, agent: &str, id: &str, body: &str) {
        let plan = prepare_workflow_run_now(Arc::clone(workers),
            &json!({"id":agent,"workflowId":id})).unwrap().unwrap();
        let WorkflowRunNowPlan::Reference { runtime_prompt, .. } = plan else {
            panic!("expected the real workflow reference plan");
        };
        assert!(runtime_prompt.contains(body));
        let expanded = expand_workflow_references(Arc::clone(workers), agent,
            "USER_REQUEST", Some(&reference(id))).unwrap();
        assert!(expanded.contains(body));
        assert!(expanded.ends_with("USER_REQUEST"));
    }

    #[cfg(unix)]
    fn set_old_modified(path: &std::path::Path) {
        let old = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000);
        fs::File::open(path).unwrap().set_modified(old).unwrap();
        assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), old);
    }

    #[test]
    fn confirmed_uninstall_reaches_retained_session_reads_and_prompt_before_failed_refresh() {
        confirmed_uninstall_consumers_case(false, false);
    }

    #[test]
    fn desktop_confirmed_uninstall_reaches_actual_cache_sessions_and_prompt_after_sync_failure() {
        confirmed_uninstall_consumers_case(true, false);
    }

    #[test]
    fn desktop_confirmed_uninstall_fences_actual_consumers_before_display_error() {
        confirmed_uninstall_consumers_case(true, true);
    }

    #[test]
    fn runner_ignores_desktop_display_error_and_keeps_cleanup_and_sync() {
        confirmed_uninstall_consumers_case(false, true);
    }

    fn confirmed_uninstall_consumers_case(desktop: bool, malformed_display: bool) {
        let root = std::env::temp_dir().join(format!("mcp-uninstall-consumers-{}", uuid::Uuid::new_v4()));
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let session = SandAgentSessionStore::new(Arc::clone(&workers));
        let agent = session.create_session(None, "user", None).unwrap();
        let retained = workers.open_workflow_store(&agent.id).unwrap();
        let mut plugins = Vec::new();
        let mut files = Vec::new();
        for (id, name, description, body) in [
            ("9", "Private", "PRIVATE_PROVIDER_DESCRIPTION", "PRIVATE_RUN_RECIPE"),
            ("10", "Allowed", "ALLOWED_PROVIDER_DESCRIPTION", "ALLOWED_RUN_RECIPE"),
        ] {
            let install = root.join(format!("plugins/cache/market/p{id}/v1"));
            let relative = format!("skills/{}/SKILL.md", name.to_lowercase());
            let file = install.join(&relative);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, format!("---\nname: {name}\ndescription: {description}\n---\n{body}\n")).unwrap();
            plugins.push(json!({
                "identifier":{"source":"cursor-third-party","name":format!("p{id}"),
                    "pluginDbId":id,"version":"v1"},
                "installPath":install,
                "skills":[{"name":name,"description":description,"path":relative}],
            }));
            files.push(file);
        }
        let published = json!({
            "plugins":plugins,"authBlocked":[],"listedPluginIds":["9","10"],
            "listedCacheKeys":[{"marketplaceSlug":"market","pluginId":"p9"},
                {"marketplaceSlug":"market","pluginId":"p10"}],
            "publisherFacts":{"9":{"publisherUserId":12},"10":{"publisherUserId":12}},
            "currentUserId":12,
        });
        let raw_reply = json!({
            "removed":true,
            "state":if malformed_display {
                json!({"servers":[{"id":"missing-required-display-fields"}]})
            } else {
                desktop_state()
            },
            "futureReply":{"keep":true},
        });
        let events = GatewayEventHub::default();
        let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
            events.clone(), std::time::Duration::from_secs(10),
        ));
        let remote = Arc::new(Mutex::new(RemoteIo {
            loaded: Ok(published.clone()), uninstall: Ok(raw_reply.clone()), calls: Vec::new(),
        }));
        let weak_relay = Arc::downgrade(&relay);
        let remote_io = Arc::clone(&remote);
        let _subscription = events.subscribe_listener(move |event| {
            if event["channel"] != HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL {
                return;
            }
            let method = event["payload"]["method"].as_str().unwrap();
            let result = {
                let mut io = remote_io.lock().unwrap();
                io.calls.push((method.to_string(), event["payload"]["args"].clone()));
                match method {
                    MCP_LIST_CATALOG_METHOD => Ok(json!([{
                        "id":"9", "name":"private",
                        "skills":[{"name":"Private","sourceUrl":LEGACY_SOURCE}],
                    }])),
                    MCP_UNINSTALL_PLUGIN_METHOD => io.uninstall.clone(),
                    MCP_LOAD_PLUGIN_SKILLS_METHOD => io.loaded.clone(),
                    MCP_INSTALL_PLUGIN_METHOD => Ok(desktop_state()),
                    _ => Err(format!("unexpected remote method {method}")),
                }
            };
            let reply = match result {
                Ok(value) => json!({"requestId":event["payload"]["requestId"],"ok":true,"result":value}),
                Err(error) => json!({"requestId":event["payload"]["requestId"],"ok":false,"error":error}),
            };
            weak_relay.upgrade().unwrap().resolve(&reply).unwrap();
        });
        let skills = Arc::new(SandPluginSkillsService::new(
            &root, Arc::new(CoordinatorPluginSkillsLoader::new(Arc::clone(&relay))),
        ));
        assert_eq!(skills.sync("seed").unwrap().len(), 2);
        workers.bind_plugin_skills_admission(skills.admission_reader()).unwrap();
        let backend = Arc::new(CoordinatorMcpManagerBackend::new(
            Arc::clone(&relay), Arc::new(|_, _| Ok(Vec::new())),
        ));
        let service = McpHostService::new(backend.clone(), Some(skills.clone()));
        write_managed_skills_cache(get_managed_skills_dir(&root), &[ManagedSkill {
            id: "managed-uninstall-control".into(), name: "Managed".into(),
            description: "MANAGED_PROVIDER_DESCRIPTION".into(), body: "MANAGED_RUN_RECIPE".into(),
        }], 1_000.0).unwrap();
        let user = retained.create(&WorkflowSpec {
            name: "User uninstall control".into(), description: "USER_PROVIDER_DESCRIPTION".into(),
            body: "USER_RUN_RECIPE".into(), trigger: None, source_ref: None,
        }).unwrap().unwrap();
        let managed = retained.get("managed-uninstall-control").unwrap();
        let legacy = retained.create(&WorkflowSpec {
            name: "Legacy uninstall reference".into(), description: String::new(),
            body: "This is a live reference to the skill at the source URL.".into(),
            trigger: None, source_ref: Some(LEGACY_SOURCE.into()),
        }).unwrap().unwrap();
        let legacy_before = fs::read(&legacy.file_path).unwrap();
        let cache_path = get_plugin_skills_cache_path(get_plugin_skills_dir(&root));

        #[cfg(unix)]
        {
            for path in [cache_path.as_path(), files[0].as_path(), files[0].parent().unwrap()] {
                set_old_modified(path);
            }
            assert_eq!(retained.get("plugin-9-private").unwrap().body, "PRIVATE_RUN_RECIPE");
            let original = fs::read_to_string(&files[0]).unwrap();
            let changed = original.replace("PRIVATE_RUN_RECIPE", "REVISED_RUN_RECIPE");
            assert_ne!(changed, original);
            assert_eq!(changed.len(), original.len());
            fs::write(&files[0], &changed).unwrap();
            set_old_modified(&files[0]);
            assert_eq!(retained.get("plugin-9-private").unwrap().body, "PRIVATE_RUN_RECIPE",
                "actual old-mtime body cache must be warm before uninstall");
            assert_eq!(workers.open_workflow_store(&agent.id).unwrap()
                .get("plugin-9-private").unwrap().body, "REVISED_RUN_RECIPE");
            fs::write(&files[0], &original).unwrap();
            set_old_modified(&files[0]);
        }
        assert_run_body(&workers, &agent.id, "plugin-9-private", "PRIVATE_RUN_RECIPE");
        assert!(skill_prompt(&retained).contains("PRIVATE_PROVIDER_DESCRIPTION"));
        let cache_before = fs::read(&cache_path).unwrap();
        let private_before = fs::read(&files[0]).unwrap();
        let allowed_before = fs::read(&files[1]).unwrap();
        // Strict confirmation negatives traverse the same real relay/backend,
        // with an allowed committed snapshot and warm consumer retained.
        for response in [
            Ok(json!({"removed":false,"reason":"team-required","state":desktop_state()})),
            Ok(json!({"state":desktop_state()})),
            Ok(json!({"removed":null,"state":desktop_state()})),
            Ok(json!({"removed":1,"state":desktop_state()})),
            Ok(json!({"removed":"true","state":desktop_state()})),
            Err("desktop owner unavailable".to_string()),
        ] {
            {
                let mut io = remote.lock().unwrap();
                io.uninstall = response.clone();
                io.calls.clear();
            }
            let result = service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"}))
                .unwrap();
            if let Ok(value) = &response {
                if value.get("removed").and_then(Value::as_bool) == Some(false) {
                    assert_eq!(result.unwrap(), *value);
                } else {
                    assert!(result.unwrap_err().contains("boolean removed"));
                }
            } else {
                assert!(result.unwrap_err().contains("desktop owner unavailable"));
            }
            assert_eq!(remote.lock().unwrap().calls.iter()
                .map(|(method, _)| method.as_str()).collect::<Vec<_>>(),
                vec![MCP_LIST_CATALOG_METHOD, MCP_UNINSTALL_PLUGIN_METHOD]);
            assert_eq!(skills.current().len(), 2);
            assert_eq!(retained.get("plugin-9-private").unwrap().body, "PRIVATE_RUN_RECIPE");
            assert!(skill_prompt(&retained).contains("PRIVATE_PROVIDER_DESCRIPTION"));
            assert_eq!(fs::read(&cache_path).unwrap(), cache_before);
            assert_eq!(fs::read(&files[0]).unwrap(), private_before);
            assert_eq!(fs::read(&files[1]).unwrap(), allowed_before);
            assert_eq!(fs::read(&legacy.file_path).unwrap(), legacy_before);
        }
        {
            let mut io = remote.lock().unwrap();
            io.uninstall = Ok(raw_reply.clone());
            io.loaded = Err("effective listing unavailable after confirmed uninstall".into());
            io.calls.clear();
        }

        let missing = McpHostService::new(backend, None);
        let error = missing.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"}))
            .unwrap().unwrap_err();
        assert!(error.contains("uninstall confirmed"));
        assert!(error.contains("owner is unavailable"));
        assert_eq!(skills.current().len(), 2);
        assert_eq!(fs::read(&cache_path).unwrap(), cache_before);
        assert_eq!(fs::read(&legacy.file_path).unwrap(), legacy_before);
        remote.lock().unwrap().calls.clear();

        let result = if desktop {
            service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"})).unwrap()
        } else {
            service.uninstall_plugin("9")
        };
        if desktop && malformed_display {
            assert!(result.unwrap_err().contains("MCP server requires"));
            assert_eq!(fs::read(&legacy.file_path).unwrap(), legacy_before,
                "Desktop display errors must return before legacy cleanup");
        } else {
            assert_eq!(result.unwrap(), if desktop {
                raw_reply.clone()
            } else {
                json!({"removed":true,"reason":null})
            });
            assert!(!legacy.file_path.exists(), "successful projection keeps legacy cleanup");
        }
        let calls = remote.lock().unwrap().calls.clone();
        assert_eq!(calls.iter().filter(|(method, _)| method == MCP_UNINSTALL_PLUGIN_METHOD)
            .map(|(_, args)| args.clone()).collect::<Vec<_>>(), vec![json!({"pluginId":"9"})]);
        let expected = if desktop && malformed_display {
            vec![MCP_LIST_CATALOG_METHOD, MCP_UNINSTALL_PLUGIN_METHOD]
        } else {
            vec![MCP_LIST_CATALOG_METHOD, MCP_UNINSTALL_PLUGIN_METHOD, MCP_LOAD_PLUGIN_SKILLS_METHOD]
        };
        assert_eq!(calls.iter().map(|(method, _)| method.as_str()).collect::<Vec<_>>(), expected);
        assert_eq!(fs::read(&cache_path).unwrap(), cache_before);
        assert_eq!(fs::read(&files[0]).unwrap(), private_before);
        assert_eq!(fs::read(&files[1]).unwrap(), allowed_before);
        assert!(skills.current_auth_blocked().is_empty(), "uninstall is not an authentication refusal");
        assert_eq!(PluginSkillsPort::status(skills.as_ref()), json!({"authBlocked":[]}));
        assert_eq!(skills.current().iter().map(|row| row.plugin_id.as_str()).collect::<Vec<_>>(), vec!["10"]);

        let reopened_workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        reopened_workers.bind_plugin_skills_admission(skills.admission_reader()).unwrap();
        let reopened = SandAgentSessionStore::new(Arc::clone(&reopened_workers))
            .workflow_store_for(&agent.id).unwrap();
        for store in [&retained, &workers.open_workflow_store(&agent.id).unwrap(), &reopened] {
            assert!(store.get("plugin-9-private").is_none());
            assert!(!store.list().iter().any(|row| row.plugin_id.as_deref() == Some("9")));
            assert!(store.update("plugin-9-private", &WorkflowSpec {
                name: "Forbidden".into(), description: String::new(),
                body: "MUST_NOT_WRITE".into(), trigger: None, source_ref: None,
            }).unwrap().is_none());
            let prompt = skill_prompt(store);
            assert!(!prompt.contains("PRIVATE_PROVIDER_DESCRIPTION"));
            assert!(!prompt.contains(files[0].to_string_lossy().as_ref()));
            for (file, description) in [
                (&files[1], "ALLOWED_PROVIDER_DESCRIPTION"),
                (&user.file_path, "USER_PROVIDER_DESCRIPTION"),
                (&managed.file_path, "MANAGED_PROVIDER_DESCRIPTION"),
            ] {
                assert!(prompt.contains(file.to_string_lossy().as_ref()));
                assert!(prompt.contains(description));
            }
        }
        let fetched = dispatch_workflow_command(Arc::clone(&workers), "getAgentWorkflow",
            &json!({"id":agent.id,"workflowId":"plugin-9-private"})).unwrap().unwrap();
        assert!(fetched.is_null());
        assert!(prepare_workflow_run_now(Arc::clone(&workers),
            &json!({"id":agent.id,"workflowId":"plugin-9-private"})).unwrap().is_none());
        assert_eq!(expand_workflow_references(Arc::clone(&workers), &agent.id,
            "USER_REQUEST", Some(&reference("plugin-9-private"))).unwrap(), "USER_REQUEST");
        assert_eq!(fs::read(&files[0]).unwrap(), private_before);
        for (id, body) in [
            ("plugin-10-allowed", "ALLOWED_RUN_RECIPE"),
            (user.id.as_str(), "USER_RUN_RECIPE"),
            ("managed-uninstall-control", "MANAGED_RUN_RECIPE"),
        ] {
            assert_run_body(&workers, &agent.id, id, body);
        }

        // Backend install success alone cannot reopen a retained snapshot.
        assert_eq!(service.install_plugin("9", &HashMap::new()).unwrap().len(), 1);
        assert!(retained.get("plugin-9-private").is_none());
        assert_eq!(fs::read(&cache_path).unwrap(), cache_before);
        remote.lock().unwrap().loaded = Ok(published);
        assert_eq!(service.install_plugin("9", &HashMap::new()).unwrap().len(), 1);
        assert_run_body(&workers, &agent.id, "plugin-9-private", "PRIVATE_RUN_RECIPE");
        let restored = skill_prompt(&retained);
        assert!(restored.contains("PRIVATE_PROVIDER_DESCRIPTION"));
        assert!(restored.contains(files[0].to_string_lossy().as_ref()));
        assert!(retained.update("plugin-9-private", &WorkflowSpec {
            name: "Restored publisher".into(), description: "Restored permission".into(),
            body: "AUTHORIZED_PUBLISHER_EDIT".into(), trigger: None, source_ref: None,
        }).unwrap().is_some());
        assert!(fs::read_to_string(&files[0]).unwrap().contains("AUTHORIZED_PUBLISHER_EDIT"));
        assert!(skills.current_auth_blocked().is_empty());
        assert!(service.dispatch_desktop_command("syncPluginSkills", &json!({})).unwrap()
            .unwrap().as_array().unwrap().iter().any(|row| row["pluginId"] == "9"));
        assert_eq!(service.dispatch_desktop_command("refreshMcp", &json!({})).unwrap().unwrap(), Value::Null);
        assert_eq!(service.dispatch_desktop_command("getPluginSyncStatus", &json!({})).unwrap().unwrap(),
            PluginSkillsPort::status(skills.as_ref()));

        // A retired Skills owner cannot accept a late confirmed result through
        // an otherwise retained MCP service.
        skills.dispose();
        let before_retired = fs::read(&cache_path).unwrap();
        remote.lock().unwrap().calls.clear();
        let error = service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"}))
            .unwrap().unwrap_err();
        assert!(error.contains("uninstall confirmed") && error.contains("disposed"));
        assert!(!skills.admission_reader().is_denied("9"));
        assert_eq!(fs::read(&cache_path).unwrap(), before_retired);
        assert_eq!(remote.lock().unwrap().calls.iter()
            .map(|(method, _)| method.as_str()).collect::<Vec<_>>(),
            vec![MCP_LIST_CATALOG_METHOD, MCP_UNINSTALL_PLUGIN_METHOD]);
        service.dispose().unwrap();
        relay.cancel_all("fixture complete");
        reopened_workers.shutdown();
        workers.shutdown();
        fs::remove_dir_all(root).unwrap();
    }
}

