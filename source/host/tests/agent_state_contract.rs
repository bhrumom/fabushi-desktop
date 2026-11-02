use std::fs;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use mahayana_host_runtime::agents::agent_profile::{
    get_sand_profile_path, read_sand_profile_file,
};
use mahayana_host_runtime::agents::settings_file::{
    get_sand_settings_path, read_sand_settings_file,
};
use mahayana_host_runtime::automations::automation::AutomationSpec;
use mahayana_host_runtime::extensions::memory::agent_state::{
    AvatarBoxFileReader, MemoryScope, MemoryTier, SandAgentState,
};
use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, get_agent_memory_dir, get_project_memory_shard_dir,
    get_user_memory_shard_dir,
};
use mahayana_host_runtime::extensions::session::channel_store::{
    FileChannelStore, get_agent_channels_dir,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-agent-state-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn routes_memory_to_agent_user_and_joined_project_shards() {
    let root = temp_root("memory");
    let state = SandAgentState::new(&root, "agent-a")
        .expect("state")
        .with_clock(Arc::new(|| 1_780_000_000_000));

    assert!(state.write_memory(
        "User likes tea",
        MemoryTier::Profile,
        MemoryScope::Agent,
        None,
    ).ok);
    assert!(state.write_memory(
        "Remember jointly",
        MemoryTier::Note,
        MemoryScope::User,
        None,
    ).ok);
    assert!(!state.write_memory(
        "Project fact",
        MemoryTier::Log,
        MemoryScope::Project,
        Some("alpha"),
    ).ok);

    assert!(state.create_project("alpha", "Alpha", Some("Shared work")).ok);
    assert!(state.write_memory(
        "Project fact",
        MemoryTier::Log,
        MemoryScope::Project,
        Some("alpha"),
    ).ok);

    let agent_store = FileMemoryStore::new(get_agent_memory_dir(
        root.join("agents").join("agent-a"),
    ));
    assert_eq!(agent_store.count_memories(), 1);

    let user_store = FileMemoryStore::new(get_user_memory_shard_dir(&root, "agent-a"));
    assert_eq!(user_store.count_memories(), 1);
    assert_eq!(user_store.list_memories(10)[0].content, "Note: Remember jointly");

    let project_store = FileMemoryStore::new(get_project_memory_shard_dir(
        &root, "alpha", "agent-a",
    ));
    assert_eq!(project_store.count_memories(), 1);

    assert!(state.remove_memory(
        "Project fact",
        MemoryScope::Project,
        Some("alpha"),
    ).ok);
    assert_eq!(project_store.count_memories(), 0);

    assert!(state.leave_project("alpha").ok);
    assert!(!state.write_memory(
        "No longer allowed",
        MemoryTier::Log,
        MemoryScope::Project,
        Some("alpha"),
    ).ok);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn avatar_uses_box_reader_when_box_root_path_is_not_on_host() {
    let root = temp_root("box-avatar");
    let reads = Arc::new(Mutex::new(Vec::<String>::new()));
    let reads_for_reader = Arc::clone(&reads);
    let reader: AvatarBoxFileReader = Arc::new(move |path| {
        reads_for_reader
            .lock()
            .expect("reads")
            .push(path.to_string_lossy().into_owned());
        Ok(vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 0])
    });
    let state = SandAgentState::new(&root, "agent-box-avatar")
        .expect("state")
        .with_box_file_reader(reader);
    let source = std::path::Path::new("/workspace/fabushi-avatar-from-box.png");

    let outcome = state.set_avatar(source);
    assert!(outcome.ok, "{}", outcome.message);
    assert_eq!(
        reads.lock().expect("reads").as_slice(),
        &["/workspace/fabushi-avatar-from-box.png".to_string()]
    );
    assert!(
        root.join("agents")
            .join("agent-box-avatar")
            .join("avatar.png")
            .is_file()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn owns_profile_settings_channel_and_avatar_mutations() {
    let root = temp_root("surfaces");
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_count_for_state = Arc::clone(&callback_count);
    let state = SandAgentState::new(&root, "agent-b")
        .expect("state")
        .with_avatar_changed(Arc::new(move || {
            callback_count_for_state.fetch_add(1, Ordering::SeqCst);
        }));

    let routine = state.create_automation(&AutomationSpec {
        name: "Morning brief".into(),
        prompt: "Summarize important changes".into(),
        trigger: json!({"type":"cron","schedule":"0 9 * * *"}),
        is_enabled: Some(true),
    });
    assert!(routine.ok, "{}", routine.message);

    let workflow = state.write_workflow(
        None,
        "Deploy checklist",
        Some("Release steps"),
        "# Deploy\nVerify the package.",
    );
    assert!(workflow.ok, "{}", workflow.message);

    assert!(state.update_profile(Some("Builder"), Some("Ships releases")).ok);
    let stored_profile = read_sand_profile_file(get_sand_profile_path(
        root.join("agents").join("agent-b"),
    )).expect("profile");
    assert_eq!(stored_profile.name, "Builder");
    assert_eq!(stored_profile.description, "Ships releases");

    assert!(state.update_settings(Some(true), Some(false)).ok);
    let stored_settings = read_sand_settings_file(get_sand_settings_path(
        root.join("agents").join("agent-b"),
    ));
    assert!(stored_settings.hidden_from_sidebar);
    assert!(!stored_settings.notify_on_agent_updates);

    let channel_store = FileChannelStore::new(get_agent_channels_dir(
        &root.join("agents").join("agent-b"),
    ));
    assert!(channel_store.write_metadata("slack", "Workspace").expect("channel"));
    assert!(state.disconnect_channel("slack").ok);
    assert!(channel_store.list_platforms().is_empty());

    let image = root.join("avatar-source.png");
    fs::write(&image, [137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 0])
        .expect("avatar");
    assert!(state.set_avatar(&image).ok);
    assert!(root.join("agents").join("agent-b").join("avatar.png").is_file());
    assert!(state.clear_avatar().ok);
    assert_eq!(callback_count.load(Ordering::SeqCst), 2);

    let _ = fs::remove_dir_all(root);
}

mod live_admission {
    use super::*;
    use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
    use mahayana_host_runtime::extensions::managed_setup::managed_skills_cache::{
        ManagedSkill, get_managed_skills_dir, write_managed_skills_cache,
    };
    use mahayana_host_runtime::extensions::mcp::plugin_skills::{
        LoadedPlugins, PluginSkillsLoader, SandPluginSkillsService,
    };
    use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::{
        PluginAuthBlock, get_plugin_skills_cache_path, get_plugin_skills_dir,
    };
    use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
    use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
    use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
    use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
    use mahayana_host_runtime::workflows::workflow_library::{LEGACY_WORKFLOW_FILENAME, WorkflowSpec};
    use mahayana_host_runtime::workflows::workflow_store::{FileWorkflowStore, WorkflowRecord};

    struct Loader(Mutex<LoadedPlugins>);

    impl PluginSkillsLoader for Loader {
        fn load(&self) -> Result<LoadedPlugins, String> {
            Ok(self.0.lock().unwrap().clone())
        }
    }

    fn assert_records(records: Vec<WorkflowRecord>, private_allowed: bool) {
        assert_eq!(records.iter().any(|row| row.id == "plugin-9-private"), private_allowed);
        assert!(records.iter().any(|row| row.id == "plugin-10-allowed"));
        assert!(records.iter().any(|row| row.id == "managed-state-control"));
        assert!(records.iter().any(|row| row.name == "Retained callback workflow"));
    }

    #[test]
    fn supplied_workflow_store_does_not_construct_and_migrate_a_discarded_default() {
        let root = temp_root("supplied-store-no-migration");
        let agent_id = "agent-supplied";
        let agent_dir = root.join("agents").join(agent_id);
        let supplied = FileWorkflowStore::new(&agent_dir, root.join("workflows"));
        // A later constructor must not perform a second migration while injecting
        // this already-opened store. The legacy source is deliberately created
        // only after the caller opened the supplied store.
        let legacy = agent_dir.join("workflows/legacy-after-open");
        fs::create_dir_all(&legacy).unwrap();
        let legacy_file = legacy.join(LEGACY_WORKFLOW_FILENAME);
        let raw = "---\nname: Legacy after open\n---\nLEGACY_BODY\n";
        fs::write(&legacy_file, raw).unwrap();
        let supplied_state = SandAgentState::new_with_workflow_store(&root, agent_id, supplied).unwrap();
        assert_eq!(supplied_state.agent_dir(), agent_dir.as_path());
        assert_eq!(fs::read_to_string(&legacy_file).unwrap(), raw);
        assert!(!root.join("workflows/legacy-after-open/SKILL.md").exists());
        assert!(!supplied_state.workflow_records().iter().any(|row| row.id == "legacy-after-open"));

        let default_state = SandAgentState::new(&root, agent_id).unwrap();
        assert!(!legacy.exists(), "the original constructor retains legacy migration");
        assert_eq!(fs::read_to_string(root.join("workflows/legacy-after-open/SKILL.md")).unwrap(), raw);
        assert!(default_state.workflow_records().iter().any(|row| row.id == "legacy-after-open"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn both_state_constructors_reject_invalid_agent_ids_before_owned_paths_or_migration() {
        let root = temp_root("supplied-store-invalid-id");
        for (index, id) in ["", " agent", "agent ", "../escape", "a/b", "a\\b", ".", "..", "a\0b"]
            .into_iter().enumerate()
        {
            let invalid_root = root.join(format!("invalid-{index}"));
            assert!(!invalid_root.exists());
            assert!(SandAgentState::new(&invalid_root, id).is_err(), "default accepted {id:?}");
            assert!(!invalid_root.exists());
            let caller_root = root.join(format!("caller-{index}"));
            let supplied = FileWorkflowStore::new(caller_root.join("agents/valid"), caller_root.join("workflows"));
            assert!(SandAgentState::new_with_workflow_store(&invalid_root, id, supplied).is_err(),
                "supplied-store constructor accepted {id:?}");
            assert!(!invalid_root.exists(), "invalid owned root was created for {id:?}");
        }
        let invalid_root = root.join("invalid-with-migration-source");
        let legacy = invalid_root.join("agents/ invalid /workflows/untouched-invalid");
        fs::create_dir_all(&legacy).unwrap();
        let legacy_file = legacy.join(LEGACY_WORKFLOW_FILENAME);
        let raw = "---\nname: Untouched invalid\n---\nDO_NOT_MIGRATE\n";
        fs::write(&legacy_file, raw).unwrap();
        let caller_root = root.join("independent-caller");
        let supplied = FileWorkflowStore::new(caller_root.join("agents/valid"), caller_root.join("workflows"));
        assert!(SandAgentState::new(&invalid_root, " invalid ").is_err());
        assert!(SandAgentState::new_with_workflow_store(&invalid_root, " invalid ", supplied).is_err());
        assert_eq!(fs::read_to_string(&legacy_file).unwrap(), raw,
            "invalid IDs must be refused before a default-store migration can move this file");
        assert!(!invalid_root.join("workflows/untouched-invalid").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn session_store_live_admission_reaches_retained_state_and_real_host_runner_surfaces() {
        let root = temp_root("host-runner-live-admission");
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let session = SandAgentSessionStore::new(Arc::clone(&workers));
        let agent = session.create_session(None, "user", None).unwrap();
        let supplied_state = SandAgentState::new_with_workflow_store(
            &root, &agent.id, workers.open_workflow_store(&agent.id).unwrap(),
        ).unwrap();
        let controller = Arc::new(SandLocalToolPermissionController::production(Arc::new(
            SettingsService::new(root.join("settings.json")),
        )));
        let composition = HostRunnerComposition::production(controller, Arc::clone(&workers));
        let retained = composition.compose_turn_state_surfaces(
            Arc::clone(&workers), &agent.id, false, true, None,
        ).unwrap();
        let retained_writer = retained.state_writer.unwrap();
        let group = composition.compose_turn_state_surfaces(
            Arc::clone(&workers), "../must-not-open", true, true, None,
        ).unwrap();
        assert!(group.state_writer.is_none());
        assert!(group.multitask_todo_state.is_none());

        let mut plugins = Vec::new();
        let mut skill_files = Vec::new();
        for (id, name) in [("9", "Private"), ("10", "Allowed")] {
            let install = root.join(format!("plugins/cache/market/p{id}/v1"));
            let relative = format!("skills/{}/SKILL.md", name.to_lowercase());
            let file = install.join(&relative);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, format!("---\nname: {name}\n---\n{id}_ORIGINAL_BODY\n")).unwrap();
            plugins.push(json!({
                "identifier":{"source":"cursor-third-party","name":format!("p{id}"),
                    "pluginDbId":id,"version":"v1"},
                "installPath":install,"skills":[{"name":name,"path":relative}]
            }));
            skill_files.push(file);
        }
        let published: LoadedPlugins = serde_json::from_value(json!({
            "plugins":plugins,"authBlocked":[],"listedPluginIds":["9","10"],
            "listedCacheKeys":[{"marketplaceSlug":"market","pluginId":"p9"},
                {"marketplaceSlug":"market","pluginId":"p10"}],
            "publisherFacts":{"9":{"publisherUserId":12},"10":{"publisherUserId":12}},
            "currentUserId":12
        })).unwrap();
        let loader = Arc::new(Loader(Mutex::new(published.clone())));
        let service = SandPluginSkillsService::new(&root, loader.clone());
        assert_eq!(service.sync("seed").unwrap().len(), 2);
        write_managed_skills_cache(get_managed_skills_dir(&root), &[ManagedSkill {
            id: "managed-state-control".into(), name: "Managed state control".into(),
            description: String::new(), body: "MANAGED_STATE_BODY".into(),
        }], 1_000.0).unwrap();
        let created = retained_writer.write_workflow(None, "Retained callback workflow",
            Some("Session owned mutation"), "CALLBACK_ORIGINAL_BODY");
        assert!(created.ok, "{}", created.message);
        let callback_workflow = session.list_agent_workflows(&agent.id).unwrap().into_iter()
            .find(|row| row.name == "Retained callback workflow").unwrap();
        assert_records(supplied_state.workflow_records(), true);
        assert_records(retained_writer.workflow_records(), true);
        workers.bind_plugin_skills_admission(service.admission_reader()).unwrap();

        let cache_path = get_plugin_skills_cache_path(get_plugin_skills_dir(&root));
        let cache_before = fs::read(&cache_path).unwrap();
        let private_before = fs::read(&skill_files[0]).unwrap();
        let mut refusal = published.clone();
        refusal.plugins.retain(|plugin| plugin.identifier.plugin_db_id.as_deref() != Some("9"));
        refusal.plugins[0].install_path = std::path::PathBuf::from("relative/invalid-stage");
        refusal.auth_blocked = vec![PluginAuthBlock {
            plugin_id: "9".into(), plugin_name: "Private".into(), marketplace_name: None,
        }];
        *loader.0.lock().unwrap() = refusal;
        assert_eq!(service.sync("stage-failure").unwrap_err(), "invalid plugin skills cache snapshot");
        assert_eq!(fs::read(&cache_path).unwrap(), cache_before);
        assert_records(supplied_state.workflow_records(), false);
        assert_records(retained_writer.workflow_records(), false);
        let reopened = composition.compose_turn_state_surfaces(
            Arc::clone(&workers), &agent.id, false, true, None,
        ).unwrap();
        assert_records(reopened.state_writer.unwrap().workflow_records(), false);
        assert!(!supplied_state.write_workflow(Some("plugin-9-private"), "Forbidden", None, "MUST_NOT_WRITE").ok);
        // Session's existing update API returns its refreshed list even when no
        // record is editable. Its observable safety result is unchanged bytes.
        let _ = retained_writer.write_workflow(Some("plugin-9-private"), "Forbidden", None, "MUST_NOT_WRITE");
        assert_eq!(fs::read(&skill_files[0]).unwrap(), private_before);

        *loader.0.lock().unwrap() = published;
        assert_eq!(service.sync("fresh-success").unwrap().len(), 2);
        assert_records(supplied_state.workflow_records(), true);
        assert_records(retained_writer.workflow_records(), true);

        let updated = retained_writer.write_workflow(Some(&callback_workflow.id),
            "Retained callback workflow", None, "CALLBACK_ACTIVE_BODY");
        assert!(updated.ok, "{}", updated.message);
        let callback_before = fs::read(&callback_workflow.file_path).unwrap();
        assert!(std::str::from_utf8(&callback_before).unwrap().ends_with("\nCALLBACK_ACTIVE_BODY\n"));

        // Retain an observation handle before deletion custody rejects public
        // owner opens. Shared workflow edits retain Session's deletion fence.
        let owner = workers.open_agent_db_owner(&agent.id).unwrap();
        let admission = owner.read_session_work_admission().unwrap().unwrap();
        assert_eq!(admission.session_id, agent.id);
        assert_eq!(admission.session_key, format!("agent:{}:main", agent.id));
        assert!(!admission.deletion_pending);
        workers.begin_agent_delete(&agent.id).unwrap();
        let fenced = owner.read_session_work_admission().unwrap().unwrap();
        assert!(fenced.deletion_pending);
        assert_eq!(fenced.session_id, admission.session_id);
        assert_eq!(fenced.session_key, admission.session_key);

        // Isolate the durable fence: the process-local marker cannot be the
        // reason this retained production callback refuses the write.
        workers.clear_agent_delete_fast_path(&agent.id);
        assert!(!workers.is_agent_being_deleted(&agent.id));
        let refused = retained_writer.write_workflow(Some(&callback_workflow.id),
            "Retained callback workflow", None, "DELETION_MUST_NOT_WRITE");
        assert!(!refused.ok, "Runner must retain the Session mutation fence");
        assert_eq!(fs::read(&callback_workflow.file_path).unwrap(), callback_before);
        assert_eq!(owner.read_session_work_admission().unwrap().unwrap(), fenced);

        workers.end_agent_delete(&agent.id).unwrap();
        assert!(!owner.read_session_work_admission().unwrap().unwrap().deletion_pending);
        let resumed = retained_writer.write_workflow(Some(&callback_workflow.id),
            "Retained callback workflow", None, "CALLBACK_AFTER_DELETION_BODY");
        assert!(resumed.ok, "{}", resumed.message);
        assert!(fs::read_to_string(&callback_workflow.file_path).unwrap()
            .ends_with("\nCALLBACK_AFTER_DELETION_BODY\n"));
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn production_runner_workflow_generation_is_fenced_across_archive_and_restore() {
        let root = temp_root("runner-workflow-generation");
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let session = SandAgentSessionStore::new(Arc::clone(&workers));
        let agent = session.create_session(None, "user", None).unwrap();
        let unaffected = session.create_session(None, "user", None).unwrap();
        let controller = Arc::new(SandLocalToolPermissionController::production(Arc::new(
            SettingsService::new(root.join("settings.json")),
        )));
        let composition = HostRunnerComposition::production(controller, Arc::clone(&workers));
        let retained_writer = composition.compose_turn_state_surfaces(
            Arc::clone(&workers), &agent.id, false, true, None,
        ).unwrap().state_writer.unwrap();
        let unaffected_writer = composition.compose_turn_state_surfaces(
            Arc::clone(&workers), &unaffected.id, false, false, None,
        ).unwrap().state_writer.unwrap();
        assert!(retained_writer.write_workflow(None, "Retained callback workflow", None,
            "CALLBACK_ORIGINAL_BODY").ok);
        let callback_workflow = session.list_agent_workflows(&agent.id).unwrap().into_iter()
            .find(|row| row.name == "Retained callback workflow").unwrap();
        let callback_before = fs::read(&callback_workflow.file_path).unwrap();
        let legacy = root.join("agents").join(&agent.id).join("workflows/unmigrated-generation");
        fs::create_dir_all(&legacy).unwrap();
        let legacy_file = legacy.join(LEGACY_WORKFLOW_FILENAME);
        let legacy_before = "---\nname: Unmigrated generation\n---\nLEGACY_MUST_STAY\n";
        fs::write(&legacy_file, legacy_before).unwrap();
        let admission = workers.read_session_work_admission(&agent.id).unwrap().unwrap();
        let archived = workers.transition_session_work_archive(&agent.id, &admission.session_key,
            &agent.id, Some(&admission.lifecycle_revision), true, Some("contract"),
            Some("runner-generation-fence"), 1_000.0).unwrap();
        let refused = retained_writer.write_workflow(Some(&callback_workflow.id),
            "Retained callback workflow", None, "ARCHIVED_MUST_NOT_WRITE");
        assert!(!refused.ok, "Runner must retain the Session mutation fence");
        assert_eq!(fs::read(&callback_workflow.file_path).unwrap(), callback_before);
        let archived_records = retained_writer.workflow_records();
        assert!(!retained_writer.write_workflow(None, "Archived retained create", None,
            "ARCHIVED_CREATE_MUST_NOT_WRITE").ok);
        assert!(!retained_writer.delete_workflow(&callback_workflow.id).ok);
        assert_eq!(fs::read(&callback_workflow.file_path).unwrap(), callback_before);
        assert_eq!(retained_writer.workflow_records(), archived_records);
        assert!(!root.join("workflows/archived-retained-create").exists());
        assert_eq!(fs::read_to_string(&legacy_file).unwrap(), legacy_before);
        assert!(!root.join("workflows/unmigrated-generation").exists());
        assert!(composition.compose_turn_state_surfaces(
            Arc::clone(&workers), &agent.id, false, true, None,
        ).is_err());

        let unaffected_created = unaffected_writer.write_workflow(None,
            "Unaffected session workflow", None, "UNAFFECTED_BODY");
        assert!(unaffected_created.ok, "{}", unaffected_created.message);
        let unaffected_workflow = session.list_agent_workflows(&unaffected.id).unwrap()
            .into_iter().find(|row| row.name == "Unaffected session workflow").unwrap();
        assert!(fs::read_to_string(&unaffected_workflow.file_path).unwrap().contains("UNAFFECTED_BODY"));

        let restored = workers.transition_session_work_archive(&agent.id, &admission.session_key,
            &agent.id, Some(&archived.lifecycle_revision), false, None, None, 2_000.0).unwrap();
        assert_ne!(restored.lifecycle_revision, admission.lifecycle_revision);
        assert_ne!(restored.lifecycle_revision, archived.lifecycle_revision);
        let restored_records = retained_writer.workflow_records();
        assert!(!retained_writer.write_workflow(None, "Stale retained create", None,
            "STALE_CREATE_MUST_NOT_WRITE").ok);
        assert!(!retained_writer.write_workflow(Some(&callback_workflow.id),
            "Retained callback workflow", None, "STALE_UPDATE_MUST_NOT_WRITE").ok);
        assert!(!retained_writer.delete_workflow(&callback_workflow.id).ok);
        assert_eq!(fs::read(&callback_workflow.file_path).unwrap(), callback_before);
        assert_eq!(retained_writer.workflow_records(), restored_records);
        assert!(!root.join("workflows/stale-retained-create").exists());
        assert_eq!(fs::read_to_string(&legacy_file).unwrap(), legacy_before);
        assert!(!root.join("workflows/unmigrated-generation").exists());

        let fresh_writer = composition.compose_turn_state_surfaces(
            Arc::clone(&workers), &agent.id, false, true, None,
        ).unwrap().state_writer.unwrap();
        assert!(!legacy.exists(), "a fresh admitted store may perform its legacy migration");
        assert!(fresh_writer.workflow_records().iter().any(|row| row.name == "Unmigrated generation"));
        let fresh_update = fresh_writer.write_workflow(Some(&callback_workflow.id),
            "Retained callback workflow", None, "RESTORED_FRESH_BODY");
        assert!(fresh_update.ok, "{}", fresh_update.message);
        assert!(fs::read_to_string(&callback_workflow.file_path).unwrap().contains("RESTORED_FRESH_BODY"));
        let fresh_create = fresh_writer.write_workflow(None, "Fresh restored workflow", None,
            "FRESH_RESTORED_CREATE_BODY");
        assert!(fresh_create.ok, "{}", fresh_create.message);
        let fresh_workflow = fresh_writer.workflow_records().into_iter()
            .find(|row| row.name == "Fresh restored workflow").unwrap();
        assert!(fs::read_to_string(&fresh_workflow.file_path).unwrap().contains("FRESH_RESTORED_CREATE_BODY"));
        assert!(fresh_writer.delete_workflow(&fresh_workflow.id).ok);
        assert!(!fresh_workflow.file_path.exists());
        assert!(unaffected_writer.write_workflow(Some(&unaffected_workflow.id),
            "Unaffected session workflow", None, "UNAFFECTED_AFTER_RESTORE").ok);
        assert!(fs::read_to_string(&unaffected_workflow.file_path).unwrap().contains("UNAFFECTED_AFTER_RESTORE"));

        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn bound_runner_workflow_mutations_refuse_missing_and_malformed_admission() {
        let root = temp_root("runner-workflow-admission-identity");
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let session = SandAgentSessionStore::new(Arc::clone(&workers));
        let agent = session.create_session(None, "user", None).unwrap();
        let controller = Arc::new(SandLocalToolPermissionController::production(Arc::new(
            SettingsService::new(root.join("settings.json")),
        )));
        let composition = HostRunnerComposition::production(controller, Arc::clone(&workers));
        let writer = composition.compose_turn_state_surfaces(
            Arc::clone(&workers), &agent.id, false, false, None,
        ).unwrap().state_writer.unwrap();
        assert!(writer.write_workflow(None, "Bound workflow", None, "BOUND_ORIGINAL_BODY").ok);
        let records_before = writer.workflow_records();
        let workflow = records_before.iter().find(|row| row.name == "Bound workflow").unwrap();
        let bytes_before = fs::read(&workflow.file_path).unwrap();
        let admission = workers.read_session_work_admission(&agent.id).unwrap().unwrap();
        let legacy = root.join("agents").join(&agent.id).join("workflows/unmigrated-bound");
        fs::create_dir_all(&legacy).unwrap();
        let legacy_file = legacy.join(LEGACY_WORKFLOW_FILENAME);
        let legacy_before = "---\nname: Unmigrated bound\n---\nLEGACY_MUST_STAY\n";
        fs::write(&legacy_file, legacy_before).unwrap();

        // Corrupt the actual durable authority; no alternate admission model is used.
        let control = rusqlite::Connection::open(workers.session_db_path(&agent.id).unwrap()).unwrap();
        let mut faults: Vec<(&str, Option<String>)> = vec![
            ("missing", None),
            ("invalid JSON", Some("not-json".into())),
        ];
        for (field, value) in [
            ("sessionId", "another-agent"),
            ("sessionKey", "agent:another-agent:main"),
            ("sessionKey", ""),
            ("lifecycleRevision", ""),
        ] {
            let mut malformed = serde_json::to_value(&admission).unwrap();
            malformed[field] = json!(value);
            faults.push((field, Some(malformed.to_string())));
        }
        for (label, raw) in faults {
            control.execute("DELETE FROM kv WHERE key = 'sessionWorkAdmission'", []).unwrap();
            if let Some(raw) = raw {
                control.execute("INSERT INTO kv (key, value) VALUES ('sessionWorkAdmission', ?1)",
                    rusqlite::params![raw]).unwrap();
            }
            assert!(!writer.write_workflow(None, "Invalid admission create", None,
                "INVALID_CREATE_MUST_NOT_WRITE").ok, "{label}");
            assert!(!writer.write_workflow(Some(&workflow.id), "Bound workflow", None,
                "INVALID_UPDATE_MUST_NOT_WRITE").ok, "{label}");
            assert!(!writer.delete_workflow(&workflow.id).ok, "{label}");
            assert!(composition.compose_turn_state_surfaces(
                Arc::clone(&workers), &agent.id, false, false, None,
            ).is_err(), "{label}");
            assert_eq!(fs::read(&workflow.file_path).unwrap(), bytes_before, "{label}");
            assert_eq!(writer.workflow_records(), records_before, "{label}");
            assert_eq!(fs::read_to_string(&legacy_file).unwrap(), legacy_before, "{label}");
            assert!(!root.join("workflows/unmigrated-bound").exists(), "{label}");
            assert!(!root.join("workflows/invalid-admission-create").exists(), "{label}");
        }

        // Missing admission stays compatible only for the unbound Session facade.
        control.execute("DELETE FROM kv WHERE key = 'sessionWorkAdmission'", []).unwrap();
        let legacy_result = session.create_agent_workflow(&agent.id, &WorkflowSpec {
            name: "Unbound legacy workflow".into(), description: String::new(),
            body: "LEGACY_COMPATIBLE_BODY".into(), trigger: None, source_ref: None,
        }).unwrap();
        let legacy_workflow = legacy_result.iter()
            .find(|row| row.name == "Unbound legacy workflow").unwrap();
        assert!(fs::read_to_string(&legacy_workflow.file_path).unwrap().contains("LEGACY_COMPATIBLE_BODY"));
        assert!(workers.read_session_work_admission(&agent.id).unwrap().is_none());
        assert!(!writer.write_workflow(Some(&workflow.id), "Bound workflow", None,
            "STILL_BOUND_MUST_NOT_WRITE").ok);
        assert_eq!(fs::read(&workflow.file_path).unwrap(), bytes_before);
        drop(control);
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}
