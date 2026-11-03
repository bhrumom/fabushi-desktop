use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::mcp::mcp_service::{
    BoxServerStatus, CatalogField, CatalogPlugin, CatalogSkill, EffectivePlugin, McpHostService,
    McpManagerBackend, McpServerSummary, PluginSkillsPort, PluginUninstallIntent,
    PluginUninstallSettlement, to_auth_result, to_installed_server, to_plugin_summary,
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
    reservations: Vec<PluginUninstallIntent>,
    settlements: Vec<(PluginUninstallIntent, PluginUninstallSettlement)>,
    reservation_error: Option<String>,
    settlement_error: Option<String>,
    after_reservation: Option<Arc<dyn Fn() + Send + Sync>>,
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
    fn begin_uninstall(&self, plugin_id: &str) -> Result<PluginUninstallIntent, String> {
        let (intent, after_reservation) = {
            let mut state = self.0.lock().unwrap();
            if let Some(error) = state.reservation_error.as_ref() {
                return Err(error.clone());
            }
            let intent = PluginUninstallIntent {
                plugin_id: plugin_id.to_string(),
                operation_id: uuid::Uuid::new_v4().to_string(),
            };
            state.reservations.push(intent.clone());
            (intent, state.after_reservation.clone())
        };
        if let Some(after_reservation) = after_reservation {
            after_reservation();
        }
        Ok(intent)
    }
    fn settle_uninstall(
        &self,
        intent: &PluginUninstallIntent,
        settlement: PluginUninstallSettlement,
    ) -> Result<(), String> {
        let mut state = self.0.lock().unwrap();
        assert!(state.reservations.contains(intent), "settlement must name the reserved ticket");
        state.settlements.push((intent.clone(), settlement));
        if settlement == PluginUninstallSettlement::Removed {
            state.confirmed_uninstalls.push(intent.plugin_id.clone());
        }
        match state.settlement_error.as_ref() {
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
    let skills = skills_state.lock().unwrap();
    assert_eq!(skills.reservations.len(), 1);
    assert_eq!(skills.reservations[0].plugin_id, "p1");
    assert_eq!(skills.settlements,
        vec![(skills.reservations[0].clone(), PluginUninstallSettlement::Removed)]);
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
        let not_removed = matches!(&outcome, Ok((false, _)));
        match outcome {
            Ok((removed, reason)) => assert_eq!(result.unwrap(), json!({
                "removed": removed, "reason": reason,
            })),
            Err(error) => assert_eq!(result.unwrap_err(), error),
        }
        assert_eq!(backend_state.lock().unwrap().uninstall_ids, vec!["9"]);
        let skills = skills_state.lock().unwrap();
        assert_eq!(skills.reservations.len(), 1);
        assert_eq!(skills.reservations[0].plugin_id, "9");
        assert_eq!(skills.settlements, if not_removed {
            vec![(skills.reservations[0].clone(), PluginUninstallSettlement::NotRemoved)]
        } else {
            Vec::new()
        });
        assert!(skills.confirmed_uninstalls.is_empty());
        assert!(skills.syncs.is_empty());
        assert!(skills.removed.is_empty());
    }

    for removed in [true, false] {
        let backend_state = Arc::new(Mutex::new(BackendState {
            uninstall_result: Some(Ok((removed, None))),
            ..BackendState::default()
        }));
        let skills_state = Arc::new(Mutex::new(SkillsState {
            settlement_error: Some("owner retired".into()),
            ..SkillsState::default()
        }));
        let service = McpHostService::new(
            Arc::new(Backend(backend_state.clone())),
            Some(Arc::new(Skills(skills_state.clone()))),
        );
        let error = service.uninstall_plugin("9").unwrap_err();
        assert!(error.contains("durable intent settlement failed"));
        assert!(error.contains("owner retired"));
        assert_eq!(backend_state.lock().unwrap().uninstall_ids, vec!["9"]);
        let skills = skills_state.lock().unwrap();
        assert_eq!(skills.confirmed_uninstalls, if removed { vec!["9"] } else { Vec::new() });
        assert_eq!(skills.settlements, vec![(skills.reservations[0].clone(), if removed {
            PluginUninstallSettlement::Removed
        } else {
            PluginUninstallSettlement::NotRemoved
        })]);
        assert!(skills.removed.is_empty(), "cleanup must not precede durable settlement");
        assert!(skills.syncs.is_empty(), "refresh must not precede durable settlement");
    }
}

#[test]
fn uninstall_without_skills_owner_refuses_all_outcomes_before_backend_dispatch() {
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
        for result in [
            service.uninstall_plugin("9"),
            service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"})).unwrap(),
        ] {
            let error = result.unwrap_err();
            assert!(error.contains("not dispatched"));
            assert!(error.contains("owner is unavailable"));
        }
        assert!(backend_state.lock().unwrap().uninstall_ids.is_empty());
        assert!(backend_state.lock().unwrap().catalog_forced.is_empty());
    }
}

#[test]
fn uninstall_reservation_failure_prevents_backend_mutation() {
    for desktop in [false, true] {
        let backend_state = Arc::new(Mutex::new(BackendState::default()));
        let skills_state = Arc::new(Mutex::new(SkillsState {
            reservation_error: Some("durable intent unavailable".into()),
            ..SkillsState::default()
        }));
        let service = McpHostService::new(
            Arc::new(Backend(backend_state.clone())),
            Some(Arc::new(Skills(skills_state.clone()))),
        );
        let result = if desktop {
            service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"})).unwrap()
        } else {
            service.uninstall_plugin("9")
        };
        assert_eq!(result.unwrap_err(), "durable intent unavailable");
        assert!(backend_state.lock().unwrap().uninstall_ids.is_empty());
        let skills = skills_state.lock().unwrap();
        assert!(skills.reservations.is_empty());
        assert!(skills.settlements.is_empty());
        assert!(skills.confirmed_uninstalls.is_empty());
        assert!(skills.syncs.is_empty());
        assert!(skills.removed.is_empty());
    }
}

#[test]
fn uninstall_close_after_reservation_settles_only_not_dispatched() {
    for settlement_error in [None, Some("failed to release undispatched intent".to_string())] {
        let backend_state = Arc::new(Mutex::new(BackendState::default()));
        let skills_state = Arc::new(Mutex::new(SkillsState {
            settlement_error: settlement_error.clone(),
            ..SkillsState::default()
        }));
        let service = Arc::new(McpHostService::new(
            Arc::new(Backend(backend_state.clone())),
            Some(Arc::new(Skills(skills_state.clone()))),
        ));
        let weak_service = Arc::downgrade(&service);
        skills_state.lock().unwrap().after_reservation = Some(Arc::new(move || {
            weak_service.upgrade().unwrap().dispose().unwrap();
        }));
        let error = service.uninstall_plugin("9").unwrap_err();
        if let Some(expected) = settlement_error {
            assert!(error.contains("intent settlement failed") && error.contains(&expected));
        } else {
            assert!(error.contains("closed"));
        }
        assert!(backend_state.lock().unwrap().uninstall_ids.is_empty());
        assert_eq!(backend_state.lock().unwrap().disposed, 1);
        let skills = skills_state.lock().unwrap();
        assert_eq!(skills.reservations.len(), 1);
        assert_eq!(skills.settlements,
            vec![(skills.reservations[0].clone(), PluginUninstallSettlement::NotDispatched)]);
        assert!(skills.confirmed_uninstalls.is_empty());
        assert!(skills.syncs.is_empty());
        assert!(skills.removed.is_empty());
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
    assert!(skills_state.lock().unwrap().reservations.is_empty());
    assert!(skills_state.lock().unwrap().settlements.is_empty());
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
    assert!(skills_state.lock().unwrap().reservations.is_empty());
    assert!(skills_state.lock().unwrap().settlements.is_empty());
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
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, ExitStatus, Stdio};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use serde::{Deserialize, Serialize};
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
    use mahayana_host_runtime::host_event_bus::HostEventSubscription;
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
    use mahayana_host_runtime::workflows::workflow_store::{FileWorkflowStore, WorkflowRecord};

    const LEGACY_SOURCE: &str = "https://github.com/example/private/blob/main/skills/private/SKILL.md";

    struct RemoteIo {
        loaded: Result<Value, String>,
        uninstall: Result<Value, String>,
        calls: Vec<(String, Value)>,
        before_uninstall_reply: Option<Arc<dyn Fn(&Value) -> bool + Send + Sync>>,
    }

    struct RealMcp {
        relay: Arc<CoordinatorMcpLifecycleRelay>,
        remote: Arc<Mutex<RemoteIo>>,
        skills: Arc<SandPluginSkillsService>,
        backend: Arc<CoordinatorMcpManagerBackend>,
        service: Arc<McpHostService>,
        _subscription: HostEventSubscription,
    }

    impl RealMcp {
        fn open(root: &Path, loaded: Result<Value, String>, uninstall: Result<Value, String>,
            timeout: Duration) -> Self
        {
            let events = GatewayEventHub::default();
            let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(events.clone(), timeout));
            let remote = Arc::new(Mutex::new(RemoteIo {
                loaded, uninstall, calls: Vec::new(), before_uninstall_reply: None,
            }));
            let weak_relay = Arc::downgrade(&relay);
            let remote_io = Arc::clone(&remote);
            let subscription = events.subscribe_listener(move |event| {
                if event["channel"] != HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL {
                    return;
                }
                let method = event["payload"]["method"].as_str().unwrap();
                let (result, before_reply) = {
                    let mut io = remote_io.lock().unwrap();
                    io.calls.push((method.to_string(), event["payload"]["args"].clone()));
                    let result = match method {
                        MCP_LIST_CATALOG_METHOD => Ok(json!([{
                            "id":"9", "name":"private",
                            "skills":[{"name":"Private","sourceUrl":LEGACY_SOURCE}],
                        }])),
                        MCP_UNINSTALL_PLUGIN_METHOD => io.uninstall.clone(),
                        MCP_LOAD_PLUGIN_SKILLS_METHOD => io.loaded.clone(),
                        MCP_INSTALL_PLUGIN_METHOD => Ok(desktop_state()),
                        _ => Err(format!("unexpected remote method {method}")),
                    };
                    let before_reply = (method == MCP_UNINSTALL_PLUGIN_METHOD)
                        .then(|| io.before_uninstall_reply.clone()).flatten();
                    (result, before_reply)
                };
                // This controls remote I/O at the actual backend/relay entry;
                // neither the admission decision nor its durable image is mocked.
                if before_reply.is_some_and(|hook| !hook(event)) {
                    return;
                }
                let reply = match result {
                    Ok(value) => json!({"requestId":event["payload"]["requestId"],"ok":true,"result":value}),
                    Err(error) => json!({"requestId":event["payload"]["requestId"],"ok":false,"error":error}),
                };
                weak_relay.upgrade().unwrap().resolve(&reply).unwrap();
            });
            let skills = Arc::new(SandPluginSkillsService::new(
                root, Arc::new(CoordinatorPluginSkillsLoader::new(Arc::clone(&relay))),
            ));
            let backend = Arc::new(CoordinatorMcpManagerBackend::new(
                Arc::clone(&relay), Arc::new(|_, _| Ok(Vec::new())),
            ));
            let service = Arc::new(McpHostService::new(backend.clone(), Some(skills.clone())));
            Self { relay, remote, skills, backend, service, _subscription: subscription }
        }

        fn close(&self) {
            self.service.dispose().unwrap();
            self.skills.dispose();
            self.relay.cancel_all("fixture complete");
        }
    }

    fn seed_plugins(root: &Path) -> (Value, Vec<PathBuf>) {
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
        (json!({
            "plugins":plugins,"authBlocked":[],"listedPluginIds":["9","10"],
            "listedCacheKeys":[{"marketplaceSlug":"market","pluginId":"p9"},
                {"marketplaceSlug":"market","pluginId":"p10"}],
            "publisherFacts":{"9":{"publisherUserId":12},"10":{"publisherUserId":12}},
            "currentUserId":12,
        }), files)
    }

    struct ConsumerFixture {
        root: PathBuf,
        workers: Arc<ProductionSessionWorkers>,
        agent_id: String,
        retained: FileWorkflowStore,
        mcp: RealMcp,
        published: Value,
        files: Vec<PathBuf>,
        user: WorkflowRecord,
        managed: WorkflowRecord,
        legacy: WorkflowRecord,
        cache_path: PathBuf,
    }

    impl ConsumerFixture {
        fn seed(label: &str) -> Self {
            Self::seed_with_timeout(label, Duration::from_secs(10))
        }

        fn seed_with_timeout(label: &str, timeout: Duration) -> Self {
            let root = std::env::temp_dir().join(format!("mcp-uninstall-{label}-{}", uuid::Uuid::new_v4()));
            let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
            let session = SandAgentSessionStore::new(Arc::clone(&workers));
            let agent_id = session.create_session(None, "user", None).unwrap().id;
            let retained = workers.open_workflow_store(&agent_id).unwrap();
            let (published, files) = seed_plugins(&root);
            let mcp = RealMcp::open(&root, Ok(published.clone()),
                Ok(json!({"removed":true,"state":desktop_state()})), timeout);
            assert_eq!(mcp.skills.sync("seed").unwrap().len(), 2);
            workers.bind_plugin_skills_admission(mcp.skills.admission_reader()).unwrap();
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
            let cache_path = get_plugin_skills_cache_path(get_plugin_skills_dir(&root));
            assert_run_body(&workers, &agent_id, "plugin-9-private", "PRIVATE_RUN_RECIPE");
            assert!(skill_prompt(&retained).contains("PRIVATE_PROVIDER_DESCRIPTION"));
            assert_eq!(PluginSkillsPort::status(mcp.skills.as_ref()), json!({"authBlocked":[]}));
            mcp.remote.lock().unwrap().calls.clear();
            Self { root, workers, agent_id, retained, mcp, published, files, user, managed, legacy, cache_path }
        }

        fn manifest(&self) -> ConsumerManifest {
            let preserved = [&self.cache_path, &self.files[0], &self.files[1],
                &self.user.file_path, &self.managed.file_path].into_iter()
                .map(|path| (path.clone(), fs::read(path).unwrap())).collect();
            ConsumerManifest {
                agent_id: self.agent_id.clone(), files: self.files.clone(),
                user_id: self.user.id.clone(), user_file: self.user.file_path.clone(),
                managed_file: self.managed.file_path.clone(), legacy_file: self.legacy.file_path.clone(),
                legacy_bytes: fs::read(&self.legacy.file_path).unwrap(), preserved,
            }
        }

        fn finish(self) {
            self.mcp.close();
            self.workers.shutdown();
            let root = self.root.clone();
            drop(self);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct ConsumerManifest {
        agent_id: String,
        files: Vec<PathBuf>,
        user_id: String,
        user_file: PathBuf,
        managed_file: PathBuf,
        legacy_file: PathBuf,
        legacy_bytes: Vec<u8>,
        preserved: Vec<(PathBuf, Vec<u8>)>,
    }

    impl ConsumerManifest {
        fn assert_preserved(&self) {
            for (path, bytes) in &self.preserved {
                assert_eq!(fs::read(path).unwrap(), *bytes, "preserve {}", path.display());
            }
        }
    }

    fn uninstall_image_path(root: &Path) -> PathBuf {
        get_plugin_skills_dir(root).join("uninstall-state.json")
    }

    fn assert_recovery_state(root: &Path, skills: &SandPluginSkillsService,
        confirmed: bool, pending: bool) -> Option<PluginUninstallIntent>
    {
        let status = PluginSkillsPort::status(skills);
        let image: Value = serde_json::from_slice(&fs::read(uninstall_image_path(root)).unwrap()).unwrap();
        let confirmed_ids = if confirmed { vec!["9"] } else { Vec::new() };
        let intent = if pending {
            let entries = status["uninstallRecovery"]["pending"].as_array().unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0]["pluginId"], "9");
            let operation_id = entries[0]["operationId"].as_str().unwrap().to_string();
            assert_eq!(uuid::Uuid::parse_str(&operation_id).unwrap().to_string(), operation_id);
            Some(PluginUninstallIntent { plugin_id: "9".into(), operation_id })
        } else {
            None
        };
        let pending_rows = intent.as_ref().map(|intent| vec![json!({
            "pluginId":intent.plugin_id,"operationId":intent.operation_id,
        })]).unwrap_or_default();
        assert_eq!(image, json!({
            "formatVersion":1,"confirmedPluginIds":confirmed_ids,"pending":pending_rows,
        }));
        assert_eq!(fs::read(get_plugin_skills_dir(root).join("uninstall-state.version")).unwrap(), b"1\n");
        assert_eq!(status, if confirmed || pending {
            json!({"authBlocked":[],"uninstallRecovery":{
                "confirmedPluginIds":confirmed_ids,"pending":pending_rows,
            }})
        } else {
            json!({"authBlocked":[]})
        });
        assert!(skills.current_auth_blocked().is_empty(), "uninstall custody is never an auth refusal");
        intent
    }

    fn assert_actual_consumers_denied(workers: &Arc<ProductionSessionWorkers>,
        stores: &[&FileWorkflowStore], manifest: &ConsumerManifest)
    {
        for store in stores {
            assert!(store.get("plugin-9-private").is_none());
            assert!(!store.list().iter().any(|row| row.plugin_id.as_deref() == Some("9")));
            assert!(store.update("plugin-9-private", &WorkflowSpec {
                name: "Forbidden".into(), description: String::new(),
                body: "MUST_NOT_WRITE".into(), trigger: None, source_ref: None,
            }).unwrap().is_none());
            let prompt = skill_prompt(store);
            assert!(!prompt.contains("PRIVATE_PROVIDER_DESCRIPTION"));
            assert!(!prompt.contains(manifest.files[0].to_string_lossy().as_ref()));
            for (file, description) in [
                (&manifest.files[1], "ALLOWED_PROVIDER_DESCRIPTION"),
                (&manifest.user_file, "USER_PROVIDER_DESCRIPTION"),
                (&manifest.managed_file, "MANAGED_PROVIDER_DESCRIPTION"),
            ] {
                assert!(prompt.contains(file.to_string_lossy().as_ref()));
                assert!(prompt.contains(description));
            }
        }
        let fetched = dispatch_workflow_command(Arc::clone(workers), "getAgentWorkflow",
            &json!({"id":manifest.agent_id,"workflowId":"plugin-9-private"})).unwrap().unwrap();
        assert!(fetched.is_null());
        assert!(prepare_workflow_run_now(Arc::clone(workers),
            &json!({"id":manifest.agent_id,"workflowId":"plugin-9-private"})).unwrap().is_none());
        assert_eq!(expand_workflow_references(Arc::clone(workers), &manifest.agent_id,
            "USER_REQUEST", Some(&reference("plugin-9-private"))).unwrap(), "USER_REQUEST");
        for (id, body) in [
            ("plugin-10-allowed", "ALLOWED_RUN_RECIPE"),
            (manifest.user_id.as_str(), "USER_RUN_RECIPE"),
            ("managed-uninstall-control", "MANAGED_RUN_RECIPE"),
        ] {
            assert_run_body(workers, &manifest.agent_id, id, body);
        }
        manifest.assert_preserved();
    }

    fn assert_fixture_denied(fixture: &ConsumerFixture, manifest: &ConsumerManifest) {
        let reopened = SandAgentSessionStore::new(Arc::clone(&fixture.workers))
            .workflow_store_for(&fixture.agent_id).unwrap();
        assert_actual_consumers_denied(&fixture.workers, &[&fixture.retained, &reopened], manifest);
        assert_eq!(fixture.mcp.skills.current().iter()
            .map(|row| row.plugin_id.as_str()).collect::<Vec<_>>(), vec!["10"]);
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
    fn desktop_unknown_uninstall_retains_pending_and_denies_actual_consumers() {
        enum UnknownReply {
            Response(Result<Value, String>),
            Timeout,
            Close,
        }
        for (label, response, expected_error) in [
            ("missing-removed", UnknownReply::Response(Ok(json!({"state":desktop_state()}))), "boolean removed"),
            ("null-removed", UnknownReply::Response(Ok(json!({"removed":null,"state":desktop_state()}))), "boolean removed"),
            ("numeric-removed", UnknownReply::Response(Ok(json!({"removed":1,"state":desktop_state()}))), "boolean removed"),
            ("string-removed", UnknownReply::Response(Ok(json!({"removed":"true","state":desktop_state()}))), "boolean removed"),
            ("null-receipt", UnknownReply::Response(Ok(Value::Null)), "boolean removed"),
            ("remote-error", UnknownReply::Response(Err("desktop owner unavailable".into())), "desktop owner unavailable"),
            ("timeout", UnknownReply::Timeout, "timed out"),
            ("closed", UnknownReply::Close, "closed"),
        ] {
            let fixture = ConsumerFixture::seed_with_timeout(label, Duration::from_millis(250));
            let manifest = fixture.manifest();
            let entered = Arc::new(Mutex::new(None::<String>));
            let root = fixture.root.clone();
            let weak_skills = Arc::downgrade(&fixture.mcp.skills);
            let weak_relay = Arc::downgrade(&fixture.mcp.relay);
            let entered_capture = entered.clone();
            let no_receipt = matches!(&response, UnknownReply::Timeout | UnknownReply::Close);
            let close = matches!(&response, UnknownReply::Close);
            if let UnknownReply::Response(reply) = response {
                fixture.mcp.remote.lock().unwrap().uninstall = reply;
            }
            fixture.mcp.remote.lock().unwrap().before_uninstall_reply = Some(Arc::new(move |event| {
                // This is the actual relay dispatch, after durable reservation
                // and before any controlled remote receipt is delivered.
                assert_recovery_state(&root, &weak_skills.upgrade().unwrap(), false, true);
                *entered_capture.lock().unwrap() = Some(event["payload"]["requestId"].as_str().unwrap().into());
                if close {
                    weak_relay.upgrade().unwrap().cancel_all("owner closed after backend entry");
                }
                !no_receipt
            }));
            let error = fixture.mcp.service.dispatch_desktop_command("uninstallMcpPlugin",
                &json!({"pluginId":"9"})).unwrap().unwrap_err();
            assert!(error.contains(expected_error), "{label}: {error}");
            let request_id = entered.lock().unwrap().clone().expect("actual mutation backend entry");
            assert_eq!(fixture.mcp.remote.lock().unwrap().calls.iter()
                .map(|(method, _)| method.as_str()).collect::<Vec<_>>(),
                vec![MCP_LIST_CATALOG_METHOD, MCP_UNINSTALL_PLUGIN_METHOD]);
            let intent = assert_recovery_state(&fixture.root, &fixture.mcp.skills, false, true).unwrap();
            let journal = fs::read(uninstall_image_path(&fixture.root)).unwrap();
            assert_fixture_denied(&fixture, &manifest);
            assert_eq!(fs::read(&manifest.legacy_file).unwrap(), manifest.legacy_bytes);
            assert_eq!(fixture.mcp.relay.pending_count(), 0);
            assert!(fixture.mcp.relay.resolve(&json!({
                "requestId":request_id,"ok":true,"result":{"removed":false,"state":desktop_state()},
            })).is_err(), "a late receipt cannot settle a departed relay waiter");
            assert_eq!(fs::read(uninstall_image_path(&fixture.root)).unwrap(), journal);
            let retry = fixture.mcp.service.uninstall_plugin("9").unwrap_err();
            assert!(retry.contains("unresolved intent"), "{label}: {retry}");
            assert_eq!(fixture.mcp.remote.lock().unwrap().calls.iter()
                .filter(|(method, _)| method == MCP_UNINSTALL_PLUGIN_METHOD).count(), 1);
            assert_eq!(assert_recovery_state(&fixture.root, &fixture.mcp.skills, false, true), Some(intent.clone()));

            fixture.mcp.close();
            let fresh = RealMcp::open(&fixture.root, Err("recovery must not load".into()),
                Err("recovery must not replay uninstall".into()), Duration::from_secs(1));
            let fresh_workers = Arc::new(ProductionSessionWorkers::with_agents_root(fixture.root.join("agents"), 500));
            let fresh_store = fresh_workers.open_workflow_store(&manifest.agent_id).unwrap();
            fresh_workers.bind_plugin_skills_admission(fresh.skills.admission_reader()).unwrap();
            assert_eq!(assert_recovery_state(&fixture.root, &fresh.skills, false, true), Some(intent));
            assert_actual_consumers_denied(&fresh_workers, &[&fresh_store], &manifest);
            assert_eq!(fresh.skills.current().iter().map(|row| row.plugin_id.as_str()).collect::<Vec<_>>(), vec!["10"]);
            assert!(fresh.remote.lock().unwrap().calls.is_empty(), "fresh owner must not replay remote work");
            assert_eq!(fs::read(uninstall_image_path(&fixture.root)).unwrap(), journal);
            fresh.close();
            fresh_workers.shutdown();
            drop(fresh_store);
            drop(fresh_workers);
            drop(fresh);
            fixture.finish();
        }
    }

    #[test]
    fn pending_uninstall_cannot_be_released_by_fresh_sync() {
        let fixture = ConsumerFixture::seed("pending-fresh-sync");
        let before = fixture.manifest();
        let sentinel = fixture.root.join("plugins/cache/market/obsolete/v1/keep-until-settled.txt");
        fs::create_dir_all(sentinel.parent().unwrap()).unwrap();
        fs::write(&sentinel, b"pending mutation must retain materialization").unwrap();
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let release_rx = Mutex::new(release_rx);
        {
            let mut io = fixture.mcp.remote.lock().unwrap();
            io.uninstall = Ok(json!({"removed":false,"reason":"team-required","state":desktop_state()}));
            io.before_uninstall_reply = Some(Arc::new(move |_| {
                entered_tx.send(()).unwrap();
                release_rx.lock().unwrap().recv_timeout(Duration::from_secs(10)).unwrap();
                true
            }));
        }
        let service = fixture.mcp.service.clone();
        let (finished_tx, finished_rx) = mpsc::sync_channel(1);
        let call = std::thread::spawn(move || {
            let result = service.uninstall_plugin("9");
            finished_tx.send(result).unwrap();
        });
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let intent = assert_recovery_state(&fixture.root, &fixture.mcp.skills, false, true).unwrap();
        let journal = fs::read(uninstall_image_path(&fixture.root)).unwrap();
        assert_fixture_denied(&fixture, &before);
        assert_eq!(fixture.mcp.skills.sync("after-pending-reservation").unwrap().iter()
            .map(|row| row.plugin_id.as_str()).collect::<Vec<_>>(), vec!["10"]);
        assert_eq!(assert_recovery_state(&fixture.root, &fixture.mcp.skills, false, true), Some(intent));
        assert_eq!(fs::read(uninstall_image_path(&fixture.root)).unwrap(), journal);
        for (path, bytes) in before.preserved.iter().filter(|(path, _)| path != &fixture.cache_path) {
            assert_eq!(fs::read(path).unwrap(), *bytes);
        }
        assert_eq!(fs::read(&sentinel).unwrap(), b"pending mutation must retain materialization");
        assert_fixture_denied(&fixture, &fixture.manifest());
        release_tx.send(()).unwrap();
        assert_eq!(finished_rx.recv_timeout(Duration::from_secs(10)).unwrap().unwrap(),
            json!({"removed":false,"reason":"team-required"}));
        call.join().unwrap();
        assert_recovery_state(&fixture.root, &fixture.mcp.skills, false, false);
        assert_eq!(fixture.mcp.skills.current().len(), 2);
        assert_run_body(&fixture.workers, &fixture.agent_id, "plugin-9-private", "PRIVATE_RUN_RECIPE");
        assert!(skill_prompt(&fixture.retained).contains("PRIVATE_PROVIDER_DESCRIPTION"));
        fixture.mcp.skills.sync("after-not-removed").unwrap();
        assert!(!sentinel.exists(), "the complete settled refresh retains ordinary pruning");
        fixture.finish();
    }

    #[cfg(unix)]
    #[test]
    fn intent_persistence_failure_prevents_real_backend_dispatch() {
        use std::os::unix::fs::PermissionsExt;

        struct RestorePermissions(PathBuf, fs::Permissions);
        impl Drop for RestorePermissions {
            fn drop(&mut self) {
                fs::set_permissions(&self.0, self.1.clone()).expect("restore exact owned directory permissions");
            }
        }

        let fixture = ConsumerFixture::seed("intent-write-failure");
        let manifest = fixture.manifest();
        let directory = get_plugin_skills_dir(&fixture.root);
        let restore = RestorePermissions(directory.clone(), fs::metadata(&directory).unwrap().permissions());
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o500)).unwrap();
        let probe = directory.join(format!("owned-write-probe-{}", uuid::Uuid::new_v4()));
        let error = fs::OpenOptions::new().write(true).create_new(true).open(&probe)
            .expect_err("the Actions test identity must actually be unable to stage a file");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(fixture.mcp.service.dispatch_desktop_command("uninstallMcpPlugin",
            &json!({"pluginId":"9"})).unwrap().is_err());
        assert_eq!(fixture.mcp.remote.lock().unwrap().calls.iter()
            .map(|(method, _)| method.as_str()).collect::<Vec<_>>(), vec![MCP_LIST_CATALOG_METHOD]);
        assert!(!uninstall_image_path(&fixture.root).exists());
        assert!(!directory.join("uninstall-state.version").exists());
        assert_eq!(PluginSkillsPort::status(fixture.mcp.skills.as_ref()), json!({
            "authBlocked":[],"uninstallRecovery":{
                "confirmedPluginIds":[],"pending":[],"fault":"uninstall-state-unavailable",
            },
        }));
        assert!(fixture.mcp.skills.current().is_empty());
        assert!(fixture.retained.get("plugin-9-private").is_none());
        assert!(fixture.retained.get("plugin-10-allowed").is_none());
        let prompt = skill_prompt(&fixture.retained);
        for description in ["USER_PROVIDER_DESCRIPTION", "MANAGED_PROVIDER_DESCRIPTION"] {
            assert!(prompt.contains(description));
        }
        for (id, body) in [
            (manifest.user_id.as_str(), "USER_RUN_RECIPE"),
            ("managed-uninstall-control", "MANAGED_RUN_RECIPE"),
        ] {
            assert_run_body(&fixture.workers, &fixture.agent_id, id, body);
        }
        manifest.assert_preserved();
        assert_eq!(fs::read(&manifest.legacy_file).unwrap(), manifest.legacy_bytes);
        drop(restore);
        fixture.finish();
    }

    const UNINSTALL_CHILD_TEST: &str = "confirmed_uninstall_consumers::uninstall_recovery_child";
    const UNINSTALL_CHILD_REQUEST_ENV: &str = "FABUSHI_MCP_UNINSTALL_CHILD_REQUEST";

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    enum UninstallChildPhase {
        Reserved,
        BackendEntered,
        RemovedAfterFailedRefresh,
        RecoverPending,
        RecoverConfirmed,
    }

    impl UninstallChildPhase {
        fn exit_code(self) -> i32 {
            match self {
                Self::Reserved => 71,
                Self::BackendEntered => 72,
                Self::RemovedAfterFailedRefresh => 73,
                Self::RecoverPending | Self::RecoverConfirmed => 0,
            }
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct UninstallChildRequest {
        nonce: String,
        parent_pid: u32,
        root: PathBuf,
        phase: UninstallChildPhase,
        manifest: ConsumerManifest,
        expected_image: Option<Vec<u8>>,
        expected_status: Option<Value>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct UninstallChildReceipt {
        nonce: String,
        parent_pid: u32,
        pid: u32,
        phase: UninstallChildPhase,
        status: Value,
        calls: Vec<(String, Value)>,
    }

    fn publish_child_receipt(request_path: &Path, request: &UninstallChildRequest,
        skills: &SandPluginSkillsService, remote: &Arc<Mutex<RemoteIo>>)
    {
        let receipt = UninstallChildReceipt {
            nonce: request.nonce.clone(), parent_pid: request.parent_pid,
            pid: std::process::id(), phase: request.phase,
            status: PluginSkillsPort::status(skills), calls: remote.lock().unwrap().calls.clone(),
        };
        let run_dir = request_path.parent().unwrap();
        let mut file = fs::OpenOptions::new().write(true).create_new(true)
            .open(run_dir.join("completed.json")).unwrap();
        file.write_all(&serde_json::to_vec(&receipt).unwrap()).unwrap();
        file.sync_all().unwrap();
        #[cfg(unix)]
        fs::File::open(run_dir).unwrap().sync_all().unwrap();
    }

    struct OwnedUninstallChild {
        child: Child,
        reaped: bool,
    }

    impl OwnedUninstallChild {
        fn poll_until(&mut self, budget: Duration) -> Result<Option<ExitStatus>, String> {
            let deadline = Instant::now() + budget;
            loop {
                if let Some(status) = self.child.try_wait().map_err(|error| error.to_string())? {
                    self.reaped = true;
                    return Ok(Some(status));
                }
                if Instant::now() >= deadline {
                    return Ok(None);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        fn stop_and_reap(&mut self) -> Result<ExitStatus, String> {
            match self.poll_until(Duration::ZERO)? {
                Some(status) => return Ok(status),
                None => {}
            }
            // The retained Child handle is the entire owned process set; this
            // oracle never starts descendants or terminates by process name.
            let kill_error = self.child.kill().err();
            match self.poll_until(Duration::from_secs(5)) {
                Ok(Some(status)) => Ok(status),
                outcome => Err(format!("exact child {} not reaped: {outcome:?}; kill={kill_error:?}",
                    self.child.id())),
            }
        }

        fn wait_bounded(&mut self) -> Result<ExitStatus, String> {
            let reason = match self.poll_until(Duration::from_secs(45)) {
                Ok(Some(status)) => return Ok(status),
                Ok(None) => "owned uninstall child exceeded its 45-second deadline".to_string(),
                Err(error) => format!("owned uninstall child wait failed: {error}"),
            };
            let cleanup = self.stop_and_reap();
            Err(format!("{reason}; exact-child cleanup={cleanup:?}"))
        }
    }

    impl Drop for OwnedUninstallChild {
        fn drop(&mut self) {
            if !self.reaped {
                if let Err(error) = self.stop_and_reap() {
                    eprintln!("owned uninstall child cleanup unresolved: {error}");
                }
            }
        }
    }

    fn run_uninstall_child(root: &Path, manifest: &ConsumerManifest, phase: UninstallChildPhase,
        expected_image: Option<Vec<u8>>, expected_status: Option<Value>) -> UninstallChildReceipt
    {
        let nonce = uuid::Uuid::new_v4().to_string();
        let run_dir = root.join(format!("child-{phase:?}-{nonce}"));
        fs::create_dir(&run_dir).unwrap();
        let request_path = run_dir.join("request.json");
        let request = UninstallChildRequest {
            nonce: nonce.clone(), parent_pid: std::process::id(), root: root.to_path_buf(), phase,
            manifest: manifest.clone(), expected_image, expected_status,
        };
        fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        let stdout_path = run_dir.join("stdout.log");
        let stderr_path = run_dir.join("stderr.log");
        let mut owned = OwnedUninstallChild {
            child: Command::new(std::env::current_exe().expect("exact MCP contract executable"))
                .args(["--exact", UNINSTALL_CHILD_TEST, "--ignored", "--nocapture", "--test-threads=1"])
                .env(UNINSTALL_CHILD_REQUEST_ENV, &request_path)
                .env("SAND_CONVERSATION_GC", "0")
                .env("SAND_STALE_ROOT_GC", "0")
                .env("SAND_RETIRE_LEGACY_STORE_BLOBS", "0")
                .current_dir(&run_dir).stdin(Stdio::null())
                // Regular files keep failure output from filling an unread pipe.
                .stdout(Stdio::from(fs::File::create(&stdout_path).unwrap()))
                .stderr(Stdio::from(fs::File::create(&stderr_path).unwrap()))
                .spawn().expect("spawn exact owned uninstall child"),
            reaped: false,
        };
        let pid = owned.child.id();
        let result = owned.wait_bounded();
        let logs = format!("stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&fs::read(&stdout_path).unwrap()),
            String::from_utf8_lossy(&fs::read(&stderr_path).unwrap()));
        eprintln!("uninstall child phase={phase:?} pid={pid} nonce={nonce} result={result:?} root={}\n{logs}",
            run_dir.display());
        assert!(owned.reaped, "child ownership unresolved; retain {}", run_dir.display());
        let status = result.unwrap_or_else(|error| panic!("{error}; retain {}\n{logs}", run_dir.display()));
        assert_eq!(status.code(), Some(phase.exit_code()), "{phase:?}: {status}; {logs}");
        let receipt: UninstallChildReceipt = serde_json::from_slice(
            &fs::read(run_dir.join("completed.json")).expect("terminal exit requires a fresh child receipt"),
        ).unwrap();
        assert_eq!(receipt.nonce, nonce);
        assert_eq!(receipt.parent_pid, std::process::id());
        assert_eq!(receipt.pid, pid);
        assert_ne!(receipt.pid, receipt.parent_pid);
        assert_eq!(receipt.phase, phase);
        receipt
    }

    #[test]
    #[ignore = "private owned process-loss entrypoint; invoke only with --exact and a child-scoped request"]
    fn uninstall_recovery_child() {
        let request_path = PathBuf::from(std::env::var_os(UNINSTALL_CHILD_REQUEST_ENV)
            .expect("owned uninstall child request"));
        let request: UninstallChildRequest = serde_json::from_slice(&fs::read(&request_path).unwrap()).unwrap();
        assert_ne!(request.parent_pid, std::process::id(), "a fresh OS process is required");
        assert!(request.root.is_absolute());
        assert_eq!(uuid::Uuid::parse_str(&request.nonce).unwrap().to_string(), request.nonce);
        assert!(request_path.starts_with(&request.root));
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(request.root.join("agents"), 500));
        let session = SandAgentSessionStore::new(Arc::clone(&workers));
        assert!(session.agent_exists(&request.manifest.agent_id));
        // This store is opened before binding, as in startup before MCP. A
        // second Session-opened store below verifies newly opened consumers too.
        let retained = session.workflow_store_for(&request.manifest.agent_id).unwrap();
        let mcp = RealMcp::open(&request.root,
            Err("effective listing unavailable after confirmed uninstall".into()),
            Ok(json!({"removed":true,"state":desktop_state()})), Duration::from_secs(10));
        workers.bind_plugin_skills_admission(mcp.skills.admission_reader()).unwrap();
        request.manifest.assert_preserved();

        match request.phase {
            UninstallChildPhase::RecoverPending | UninstallChildPhase::RecoverConfirmed => {
                let confirmed = request.phase == UninstallChildPhase::RecoverConfirmed;
                assert_recovery_state(&request.root, &mcp.skills, confirmed, !confirmed);
                assert_eq!(PluginSkillsPort::status(mcp.skills.as_ref()), request.expected_status.clone().unwrap());
                let journal = request.expected_image.as_ref().unwrap();
                assert_eq!(fs::read(uninstall_image_path(&request.root)).unwrap(), *journal);
                let reopened = session.workflow_store_for(&request.manifest.agent_id).unwrap();
                assert_actual_consumers_denied(&workers, &[&retained, &reopened], &request.manifest);
                assert_eq!(mcp.skills.current().iter().map(|row| row.plugin_id.as_str()).collect::<Vec<_>>(), vec!["10"]);
                if confirmed {
                    assert!(!request.manifest.legacy_file.exists());
                } else {
                    assert_eq!(fs::read(&request.manifest.legacy_file).unwrap(), request.manifest.legacy_bytes);
                }
                assert!(mcp.remote.lock().unwrap().calls.is_empty(), "reconstruction and actual consumers never replay remote work");
                assert_eq!(fs::read(uninstall_image_path(&request.root)).unwrap(), *journal);
                mcp.close();
                workers.shutdown();
                publish_child_receipt(&request_path, &request, &mcp.skills, &mcp.remote);
            }
            UninstallChildPhase::Reserved => {
                assert_eq!(mcp.skills.current().len(), 2);
                assert_run_body(&workers, &request.manifest.agent_id, "plugin-9-private", "PRIVATE_RUN_RECIPE");
                let intent = mcp.skills.begin_uninstall("9").unwrap();
                assert_eq!(assert_recovery_state(&request.root, &mcp.skills, false, true), Some(intent));
                assert_actual_consumers_denied(&workers, &[&retained], &request.manifest);
                assert!(mcp.remote.lock().unwrap().calls.is_empty());
                publish_child_receipt(&request_path, &request, &mcp.skills, &mcp.remote);
                std::process::exit(request.phase.exit_code());
            }
            UninstallChildPhase::BackendEntered => {
                assert_eq!(mcp.skills.current().len(), 2);
                let captured_request = request.clone();
                let captured_path = request_path.clone();
                let weak_skills = Arc::downgrade(&mcp.skills);
                let weak_remote = Arc::downgrade(&mcp.remote);
                mcp.remote.lock().unwrap().before_uninstall_reply = Some(Arc::new(move |event| {
                    assert_eq!(event["payload"]["method"], MCP_UNINSTALL_PLUGIN_METHOD);
                    assert_eq!(event["payload"]["args"], json!({"pluginId":"9"}));
                    let skills = weak_skills.upgrade().unwrap();
                    assert_recovery_state(&captured_request.root, &skills, false, true);
                    captured_request.manifest.assert_preserved();
                    publish_child_receipt(&captured_path, &captured_request, &skills, &weak_remote.upgrade().unwrap());
                    // Exit in the real backend's event dispatch, before the
                    // lifecycle relay can deliver any mutation receipt.
                    std::process::exit(captured_request.phase.exit_code());
                }));
                let result = mcp.service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"}));
                panic!("backend-entry process must exit before any receipt: {result:?}");
            }
            UninstallChildPhase::RemovedAfterFailedRefresh => {
                assert_eq!(mcp.skills.current().len(), 2);
                assert_eq!(mcp.service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"}))
                    .unwrap().unwrap(), json!({"removed":true,"state":desktop_state()}));
                assert_recovery_state(&request.root, &mcp.skills, true, false);
                assert_actual_consumers_denied(&workers, &[&retained], &request.manifest);
                assert!(!request.manifest.legacy_file.exists());
                publish_child_receipt(&request_path, &request, &mcp.skills, &mcp.remote);
                std::process::exit(request.phase.exit_code());
            }
        }
    }

    #[test]
    fn uninstall_custody_survives_owned_process_exit_and_fresh_process_consumers() {
        for phase in [UninstallChildPhase::Reserved, UninstallChildPhase::BackendEntered,
            UninstallChildPhase::RemovedAfterFailedRefresh]
        {
            let fixture = ConsumerFixture::seed("process-recovery");
            let root = fixture.root.clone();
            let manifest = fixture.manifest();
            assert!(!uninstall_image_path(&root).exists());
            // Only normal production seeding happens in the parent. Close and
            // drop every owner before the first separately owned process opens.
            fixture.mcp.close();
            fixture.workers.shutdown();
            drop(fixture);
            let producer = run_uninstall_child(&root, &manifest, phase, None, None);
            let expected_methods = match phase {
                UninstallChildPhase::Reserved => Vec::new(),
                UninstallChildPhase::BackendEntered => vec![MCP_LIST_CATALOG_METHOD, MCP_UNINSTALL_PLUGIN_METHOD],
                UninstallChildPhase::RemovedAfterFailedRefresh => vec![
                    MCP_LIST_CATALOG_METHOD, MCP_UNINSTALL_PLUGIN_METHOD, MCP_LOAD_PLUGIN_SKILLS_METHOD,
                ],
                _ => unreachable!(),
            };
            assert_eq!(producer.calls.iter().map(|(method, _)| method.as_str()).collect::<Vec<_>>(), expected_methods);
            let journal = fs::read(uninstall_image_path(&root)).unwrap();
            let recovered = run_uninstall_child(&root, &manifest,
                if phase == UninstallChildPhase::RemovedAfterFailedRefresh {
                    UninstallChildPhase::RecoverConfirmed
                } else {
                    UninstallChildPhase::RecoverPending
                }, Some(journal.clone()), Some(producer.status.clone()));
            assert_eq!(recovered.status, producer.status);
            assert!(recovered.calls.is_empty());
            assert_eq!(fs::read(uninstall_image_path(&root)).unwrap(), journal);
            manifest.assert_preserved();
            // Reaching this cleanup proves both exact Child handles were reaped
            // and both nonce-qualified receipts passed every assertion.
            fs::remove_dir_all(root).unwrap();
        }
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
        let fixture = ConsumerFixture::seed("consumers");
        let manifest = fixture.manifest();
        let ConsumerFixture {
            root, workers, agent_id, retained, mcp, published, files,
            user: _, managed: _, legacy, cache_path,
        } = fixture;
        let RealMcp { relay, remote, skills, backend, service, _subscription } = mcp;
        let legacy_before = fs::read(&legacy.file_path).unwrap();
        let raw_reply = json!({
            "removed":true,
            "state":if malformed_display {
                json!({"servers":[{"id":"missing-required-display-fields"}]})
            } else {
                desktop_state()
            },
            "futureReply":{"keep":true},
        });

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
            assert_eq!(workers.open_workflow_store(&agent_id).unwrap()
                .get("plugin-9-private").unwrap().body, "REVISED_RUN_RECIPE");
            fs::write(&files[0], &original).unwrap();
            set_old_modified(&files[0]);
        }
        assert_run_body(&workers, &agent_id, "plugin-9-private", "PRIVATE_RUN_RECIPE");
        assert!(skill_prompt(&retained).contains("PRIVATE_PROVIDER_DESCRIPTION"));
        let cache_before = fs::read(&cache_path).unwrap();
        let private_before = fs::read(&files[0]).unwrap();
        let allowed_before = fs::read(&files[1]).unwrap();
        // An explicit false receipt releases only this intent and preserves
        // availability. Unknown receipts use independent fixtures below.
        let not_removed = json!({"removed":false,"reason":"team-required","state":desktop_state()});
        remote.lock().unwrap().uninstall = Ok(not_removed.clone());
        assert_eq!(service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"}))
            .unwrap().unwrap(), not_removed);
        assert_eq!(remote.lock().unwrap().calls.iter()
            .map(|(method, _)| method.as_str()).collect::<Vec<_>>(),
            vec![MCP_LIST_CATALOG_METHOD, MCP_UNINSTALL_PLUGIN_METHOD]);
        assert_eq!(skills.current().len(), 2);
        assert_eq!(retained.get("plugin-9-private").unwrap().body, "PRIVATE_RUN_RECIPE");
        assert!(skill_prompt(&retained).contains("PRIVATE_PROVIDER_DESCRIPTION"));
        assert_eq!(PluginSkillsPort::status(skills.as_ref()), json!({"authBlocked":[]}));
        assert_eq!(fs::read(&cache_path).unwrap(), cache_before);
        assert_eq!(fs::read(&files[0]).unwrap(), private_before);
        assert_eq!(fs::read(&files[1]).unwrap(), allowed_before);
        assert_eq!(fs::read(&legacy.file_path).unwrap(), legacy_before);
        {
            let mut io = remote.lock().unwrap();
            io.uninstall = Ok(raw_reply.clone());
            io.loaded = Err("effective listing unavailable after confirmed uninstall".into());
            io.calls.clear();
        }

        let missing = McpHostService::new(backend, None);
        let error = missing.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"}))
            .unwrap().unwrap_err();
        assert!(error.contains("not dispatched"));
        assert!(error.contains("owner is unavailable"));
        assert!(remote.lock().unwrap().calls.is_empty(), "missing owner must refuse before mutation");
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
        assert_recovery_state(&root, &skills, true, false);
        assert_eq!(skills.current().iter().map(|row| row.plugin_id.as_str()).collect::<Vec<_>>(), vec!["10"]);

        let reopened_workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        reopened_workers.bind_plugin_skills_admission(skills.admission_reader()).unwrap();
        let reopened = SandAgentSessionStore::new(Arc::clone(&reopened_workers))
            .workflow_store_for(&agent_id).unwrap();
        assert_actual_consumers_denied(&workers,
            &[&retained, &workers.open_workflow_store(&agent_id).unwrap(), &reopened], &manifest);

        // Backend install success alone cannot reopen a retained snapshot.
        assert_eq!(service.install_plugin("9", &HashMap::new()).unwrap().len(), 1);
        assert!(retained.get("plugin-9-private").is_none());
        assert_eq!(fs::read(&cache_path).unwrap(), cache_before);
        remote.lock().unwrap().loaded = Ok(published);
        assert_eq!(service.install_plugin("9", &HashMap::new()).unwrap().len(), 1);
        assert_run_body(&workers, &agent_id, "plugin-9-private", "PRIVATE_RUN_RECIPE");
        let restored = skill_prompt(&retained);
        assert!(restored.contains("PRIVATE_PROVIDER_DESCRIPTION"));
        assert!(restored.contains(files[0].to_string_lossy().as_ref()));
        assert!(retained.update("plugin-9-private", &WorkflowSpec {
            name: "Restored publisher".into(), description: "Restored permission".into(),
            body: "AUTHORIZED_PUBLISHER_EDIT".into(), trigger: None, source_ref: None,
        }).unwrap().is_some());
        assert!(fs::read_to_string(&files[0]).unwrap().contains("AUTHORIZED_PUBLISHER_EDIT"));
        assert!(skills.current_auth_blocked().is_empty());
        assert_recovery_state(&root, &skills, false, false);
        assert!(service.dispatch_desktop_command("syncPluginSkills", &json!({})).unwrap()
            .unwrap().as_array().unwrap().iter().any(|row| row["pluginId"] == "9"));
        assert_eq!(service.dispatch_desktop_command("refreshMcp", &json!({})).unwrap().unwrap(), Value::Null);
        assert_eq!(service.dispatch_desktop_command("getPluginSyncStatus", &json!({})).unwrap().unwrap(),
            PluginSkillsPort::status(skills.as_ref()));

        // A retired Skills owner refuses reservation before an otherwise
        // retained MCP service can enter the mutation backend.
        skills.dispose();
        let before_retired = fs::read(&cache_path).unwrap();
        remote.lock().unwrap().calls.clear();
        let error = service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"9"}))
            .unwrap().unwrap_err();
        assert!(error.contains("disposed"));
        assert!(!skills.admission_reader().is_denied("9"));
        assert_eq!(fs::read(&cache_path).unwrap(), before_retired);
        assert_eq!(remote.lock().unwrap().calls.iter()
            .map(|(method, _)| method.as_str()).collect::<Vec<_>>(),
            vec![MCP_LIST_CATALOG_METHOD]);
        service.dispose().unwrap();
        relay.cancel_all("fixture complete");
        reopened_workers.shutdown();
        workers.shutdown();
        fs::remove_dir_all(root).unwrap();
    }
}

