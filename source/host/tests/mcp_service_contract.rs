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
    fn uninstall_plugin(&self, _plugin_id: &str) -> Result<(bool, Option<String>), String> {
        Ok((true, None))
    }
    fn install_plugin(
        &self,
        _plugin_id: &str,
        _values: &HashMap<String, String>,
    ) -> Result<Vec<McpServerSummary>, String> {
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
