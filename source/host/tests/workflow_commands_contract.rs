use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::workflow_commands::{
    WORKFLOW_INJECTED_BODY_LIMIT, WorkflowRunNowPlan, build_workflow_run_prompt,
    dispatch_workflow_command, expand_workflow_references, prepare_workflow_run_now,
};
use mahayana_host_runtime::workflows::workflow_store::WorkflowRecord;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-workflow-commands-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn call(
    workers: Arc<ProductionSessionWorkers>,
    method: &str,
    args: serde_json::Value,
) -> serde_json::Value {
    dispatch_workflow_command(workers, method, &args)
        .expect("workflow command handled")
        .expect("workflow command result")
}

#[test]
fn shipping_workflow_commands_delegate_crud_and_imports_to_session_store() {
    let root = temp_root("crud");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let session = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = session.create_session(None, "user", None).expect("agent");

    let created = call(
        Arc::clone(&workers),
        "createAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "spec": {
                "name": "Research",
                "description": "Research workflow",
                "body": "Read sources and summarize.",
                "sourceRef": "local"
            }
        }),
    );
    assert_eq!(created.as_array().map(Vec::len), Some(1));
    let workflow_id = created[0]["id"].as_str().expect("workflow id").to_string();
    assert_eq!(created[0]["name"], "Research");
    assert_eq!(created[0]["sourceRef"], "local");
    assert_eq!(created[0]["isEnabledForAgent"], true);

    let listed = call(
        Arc::clone(&workers),
        "getAgentWorkflows",
        serde_json::json!({"id": agent.id}),
    );
    assert_eq!(listed[0]["id"], workflow_id);

    let updated = call(
        Arc::clone(&workers),
        "updateAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "workflowId": workflow_id,
            "spec": {
                "name": "Research updated",
                "description": "Updated",
                "body": "Read more sources and summarize."
            }
        }),
    );
    assert_eq!(updated[0]["name"], "Research updated");

    let disabled = call(
        Arc::clone(&workers),
        "setAgentWorkflowEnabled",
        serde_json::json!({
            "id": agent.id,
            "workflowId": workflow_id,
            "isEnabled": false
        }),
    );
    assert_eq!(disabled[0]["isEnabledForAgent"], false);

    let disabled_record = call(
        Arc::clone(&workers),
        "getAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "workflowId": workflow_id
        }),
    );
    assert_eq!(disabled_record["id"], workflow_id);
    assert_eq!(disabled_record["name"], "Research updated");
    assert_eq!(disabled_record["isEnabledForAgent"], false);

    let imported = call(
        Arc::clone(&workers),
        "importAgentWorkflowText",
        serde_json::json!({
            "id": agent.id,
            "name": "Imported text",
            "markdown": "---\nname: Imported\n---\nDo the imported thing."
        }),
    );
    assert_eq!(imported["result"]["imported"].as_array().map(Vec::len), Some(1));

    let linked = call(
        Arc::clone(&workers),
        "importAgentWorkflowUrl",
        serde_json::json!({
            "id": agent.id,
            "url": "https://example.test/SKILL.md",
            "name": "Live skill"
        }),
    );
    assert_eq!(linked["result"]["imported"].as_array().map(Vec::len), Some(1));
    assert!(linked["workflows"]
        .as_array()
        .expect("workflow list")
        .iter()
        .any(|workflow| workflow["sourceRef"] == "https://example.test/SKILL.md"));

    let after_delete = call(
        Arc::clone(&workers),
        "deleteAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "workflowId": workflow_id
        }),
    );
    assert!(!after_delete
        .as_array()
        .expect("workflow list")
        .iter()
        .any(|workflow| workflow["id"] == workflow_id));

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn automation_backed_workflow_creation_is_owned_by_rust_session_store() {
    let root = temp_root("automation-create");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let session = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = session.create_session(None, "user", None).expect("agent");

    let created = call(
        Arc::clone(&workers),
        "createAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "spec": {
                "name": "Scheduled",
                "body": "Run later",
                "trigger": {"schedule":"0 9 * * *","isEnabled":true}
            }
        }),
    );
    let created = created.as_array().expect("workflow list");
    assert_eq!(created.len(), 1);
    assert_eq!(created[0]["name"], "Scheduled");
    assert_eq!(created[0]["source"], "automation");
    assert_eq!(created[0]["trigger"]["schedule"], "0 9 * * *");
    assert_eq!(created[0]["trigger"]["isEnabled"], true);

    let listed = call(
        Arc::clone(&workers),
        "getAgentWorkflows",
        serde_json::json!({"id": agent.id}),
    );
    assert!(listed
        .as_array()
        .expect("workflow list")
        .iter()
        .any(|workflow| workflow["source"] == "automation"));

    let automation_id = created[0]["id"].as_str().expect("automation id");
    let plan = prepare_workflow_run_now(
        Arc::clone(&workers),
        &serde_json::json!({"id":agent.id,"workflowId":automation_id}),
    )
    .expect("automation plan")
    .expect("automation exists");
    assert_eq!(
        plan,
        WorkflowRunNowPlan::Automation {
            agent_id: agent.id.clone(),
            automation_id: automation_id.to_string(),
            automation_name: "Scheduled".to_string(),
        }
    );
    assert!(prepare_workflow_run_now(
        Arc::clone(&workers),
        &serde_json::json!({"id":agent.id,"workflowId":"missing"}),
    )
    .expect("missing workflow lookup")
    .is_none());

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn normal_send_expands_enabled_rich_text_workflow_reference() {
    let root = temp_root("normal-send-reference");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let session = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = session.create_session(None, "user", None).expect("agent");
    let created = call(
        Arc::clone(&workers),
        "createAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "spec": {
                "name": "Research",
                "description": "Read sources",
                "body": "Collect evidence and summarize."
            }
        }),
    );
    let workflow_id = created[0]["id"].as_str().expect("workflow id");
    let rich_text = serde_json::json!({
        "type":"doc",
        "content":[{
            "type":"paragraph",
            "content":[{
                "type":"workflowReference",
                "attrs":{"id":workflow_id,"label":"Research"}
            }]
        }]
    })
    .to_string();

    let expanded = expand_workflow_references(
        Arc::clone(&workers),
        &agent.id,
        "also compare the results",
        Some(&rich_text),
    )
    .expect("workflow expansion");
    assert!(expanded.contains("The user invoked the \"Research\" workflow"));
    assert!(expanded.contains("Collect evidence and summarize."));
    assert!(expanded.ends_with("also compare the results"));

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn committed_plugin_denial_reaches_session_commands_and_run_prompt_consumers() {
    use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::{
        PluginAuthBlock, PluginSkillsCacheWriteIndex, get_plugin_skills_dir,
        write_plugin_skills_cache,
    };

    let root = temp_root("plugin-denial");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let session = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = session.create_session(None, "user", None).unwrap();
    let skill_file = root.join("plugins/cache/market/review/v1/skills/review/SKILL.md");
    fs::create_dir_all(skill_file.parent().unwrap()).unwrap();
    fs::write(&skill_file, "---\nname: Installed review\n---\nPRIVATE_PLUGIN_RECIPE\n").unwrap();
    let workflow_id = "plugin-42-review";
    let rich_text = serde_json::json!({"type":"doc","content":[{
        "type":"workflowReference","attrs":{"id":workflow_id,"label":"Installed review"}
    }]}).to_string();
    let values = vec![serde_json::json!({
        "id":workflow_id,"pluginId":"42","pluginName":"Review","name":"Installed review",
        "description":"","filePath":skill_file.to_string_lossy(),"publisherUserId":9
    })];
    for blocked in [false, true, false] {
        write_plugin_skills_cache(
            get_plugin_skills_dir(&root),
            &PluginSkillsCacheWriteIndex {
                current_user_id: Some(Some(9)), skills: values.clone(),
                auth_blocked: Some(if blocked { vec![PluginAuthBlock {
                    plugin_id: "42".into(), plugin_name: "Review".into(), marketplace_name: None,
                }] } else { Vec::new() }),
            },
            || if blocked { 2.0 } else { 1.0 },
        ).unwrap();
        let listed = call(Arc::clone(&workers), "getAgentWorkflows", serde_json::json!({"id":agent.id}));
        assert_eq!(listed.as_array().unwrap().iter().any(|row| row["id"] == workflow_id), !blocked);
        let plan = prepare_workflow_run_now(Arc::clone(&workers),
            &serde_json::json!({"id":agent.id,"workflowId":workflow_id})).unwrap();
        assert_eq!(plan.is_some(), !blocked);
        let expanded = expand_workflow_references(Arc::clone(&workers), &agent.id,
            "keep the user request", Some(&rich_text)).unwrap();
        if blocked {
            assert_eq!(expanded, "keep the user request");
        } else {
            assert!(expanded.contains("PRIVATE_PLUGIN_RECIPE"));
        }
        assert!(expanded.ends_with("keep the user request"));
    }
    assert_eq!(fs::read_to_string(&skill_file).unwrap(),
        "---\nname: Installed review\n---\nPRIVATE_PLUGIN_RECIPE\n");
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn workflow_run_now_reference_plan_matches_frozen_rich_text_and_recipe_expansion() {
    let root = temp_root("run-now-reference");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let session = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = session.create_session(None, "user", None).expect("agent");
    let created = call(
        Arc::clone(&workers),
        "createAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "spec": {
                "name": "Research",
                "description": "Read sources",
                "body": "Collect evidence and summarize."
            }
        }),
    );
    let workflow_id = created[0]["id"].as_str().expect("workflow id").to_string();

    let plan = prepare_workflow_run_now(
        Arc::clone(&workers),
        &serde_json::json!({"id":agent.id,"workflowId":workflow_id}),
    )
    .expect("plan")
    .expect("workflow");
    let WorkflowRunNowPlan::Reference {
        agent_id,
        workflow_id,
        workflow_name,
        visible_prompt,
        rich_text,
        runtime_prompt,
    } = plan
    else {
        panic!("expected reference plan");
    };
    assert_eq!(agent_id, agent.id);
    assert_eq!(workflow_name, "Research");
    assert_eq!(visible_prompt, "@Research");
    assert!(rich_text.contains("\"type\":\"workflowReference\""));
    assert!(rich_text.contains(&format!("\"id\":\"{workflow_id}\"")));
    assert!(runtime_prompt.contains(
        "The user invoked the \"Research\" workflow (folder"
    ));
    assert!(runtime_prompt.contains("What it does: Read sources"));
    assert!(runtime_prompt.contains("Recipe to follow:\nCollect evidence and summarize."));
    assert!(runtime_prompt.ends_with("@Research"));

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn workflow_run_prompt_clamps_recipe_body_and_preserves_helper_location() {
    let body = "x".repeat(WORKFLOW_INJECTED_BODY_LIMIT + 50);
    let record = WorkflowRecord {
        id: "deep-research".into(),
        name: "Deep Research".into(),
        description: String::new(),
        body,
        trigger: None,
        source: "plugin".into(),
        source_ref: None,
        plugin_id: Some("plugin-deep-research".into()),
        published_by_current_user: true,
        disable_model_invocation: false,
        is_enabled_for_agent: true,
        created_at: 1.0,
        last_run_at: None,
        next_run_at: None,
        helper_scripts: vec!["collect.sh".into(), "parse.py".into()],
        file_path: std::path::PathBuf::from("/tmp/workflows/deep-research/SKILL.md"),
    };
    let prompt = build_workflow_run_prompt(&record);
    assert!(prompt.contains(
        "plugin skill id deep-research, file /tmp/workflows/deep-research/SKILL.md"
    ));
    assert!(prompt.contains(
        "Helper files live beside this workflow in /tmp/workflows/deep-research: collect.sh, parse.py."
    ));
    let recipe = prompt
        .split("Recipe to follow:\n")
        .nth(1)
        .expect("recipe")
        .split("\nHelper files")
        .next()
        .expect("recipe body");
    assert_eq!(recipe.chars().count(), WORKFLOW_INJECTED_BODY_LIMIT);
}

mod live_admission {
    use super::*;
    use std::sync::Mutex;
    use mahayana_host_runtime::extensions::managed_setup::managed_skills_cache::{
        ManagedSkill, get_managed_skills_dir, write_managed_skills_cache,
    };
    use mahayana_host_runtime::extensions::mcp::plugin_skills::{
        LoadedPlugins, PluginSkillsLoader, SandPluginSkillsService,
    };
    use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::{
        PluginAuthBlock, get_plugin_skills_cache_path, get_plugin_skills_dir,
    };
    use mahayana_host_runtime::runner::system_prompt_assembly::{
        AgentSkillPromptItem, append_budgeted_workflows_system_prompt,
    };
    use mahayana_host_runtime::workflows::workflow_library::WorkflowSpec;
    use mahayana_host_runtime::workflows::workflow_store::FileWorkflowStore;

    struct Loader(Mutex<LoadedPlugins>);
    impl PluginSkillsLoader for Loader {
        fn load(&self) -> Result<LoadedPlugins, String> {
            Ok(self.0.lock().unwrap().clone())
        }
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
        assert!(!messages.is_empty(), "production builder emitted ProviderMessage values");
        assert!(messages.iter().all(|message| message.role == "system"));
        messages.into_iter().map(|message| message.content).collect::<Vec<_>>().join("\n")
    }

    fn reference(id: &str) -> String {
        serde_json::json!({"type":"doc","content":[{
            "type":"workflowReference","attrs":{"id":id,"label":"Requested"}
        }]}).to_string()
    }

    fn assert_run_body(workers: &Arc<ProductionSessionWorkers>, agent_id: &str, id: &str, body: &str) {
        let plan = prepare_workflow_run_now(Arc::clone(workers),
            &serde_json::json!({"id":agent_id,"workflowId":id})).unwrap().unwrap();
        let WorkflowRunNowPlan::Reference { runtime_prompt, .. } = plan else {
            panic!("expected the real recipe reference plan");
        };
        assert!(runtime_prompt.contains(body), "run-now must contain {body}");
        let expanded = expand_workflow_references(Arc::clone(workers), agent_id,
            "USER_REQUEST", Some(&reference(id))).unwrap();
        assert!(expanded.contains(body), "reference must contain {body}");
        assert!(expanded.ends_with("USER_REQUEST"));
    }

    #[test]
    fn failed_refusal_publication_reaches_prebound_store_commands_and_actual_provider_prompt() {
        let root = temp_root("live-admission-consumers");
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let session = SandAgentSessionStore::new(Arc::clone(&workers));
        let agent = session.create_session(None, "user", None).unwrap();
        let opened_before_binding = workers.open_workflow_store(&agent.id).unwrap();
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
            plugins.push(serde_json::json!({
                "identifier":{"source":"cursor-third-party","name":format!("p{id}"),
                    "pluginDbId":id,"version":"v1"},
                "installPath":install,"skills":[{"name":name,"description":description,"path":relative}]
            }));
            files.push(file);
        }
        let published: LoadedPlugins = serde_json::from_value(serde_json::json!({
            "plugins":plugins,"authBlocked":[],"listedPluginIds":["9","10"],
            "listedCacheKeys":[{"marketplaceSlug":"market","pluginId":"p9"},
                {"marketplaceSlug":"market","pluginId":"p10"}],
            "publisherFacts":{"9":{"publisherUserId":12},"10":{"publisherUserId":12}},
            "currentUserId":12
        })).unwrap();
        let loader = Arc::new(Loader(Mutex::new(published.clone())));
        let service = SandPluginSkillsService::new(&root, loader.clone());
        assert_eq!(service.sync("seed").unwrap().len(), 2);
        assert!(opened_before_binding.get("plugin-9-private").is_some(), "unbound compatibility");
        workers.bind_plugin_skills_admission(service.admission_reader()).unwrap();
        workers.bind_plugin_skills_admission(service.admission_reader()).unwrap();
        let unrelated = SandPluginSkillsService::new(&root, loader.clone());
        assert!(workers.bind_plugin_skills_admission(unrelated.admission_reader()).is_err());
        write_managed_skills_cache(get_managed_skills_dir(&root), &[ManagedSkill {
            id: "managed-live".into(), name: "Managed".into(),
            description: "MANAGED_PROVIDER_DESCRIPTION".into(), body: "MANAGED_RUN_RECIPE".into(),
        }], 1_000.0).unwrap();
        let user = opened_before_binding.create(&WorkflowSpec {
            name: "User live".into(), description: "USER_PROVIDER_DESCRIPTION".into(),
            body: "USER_RUN_RECIPE".into(), trigger: None, source_ref: None,
        }).unwrap().unwrap();
        let managed = opened_before_binding.get("managed-live").unwrap();
        let before = skill_prompt(&opened_before_binding);
        assert!(before.contains("PRIVATE_PROVIDER_DESCRIPTION"));
        assert!(before.contains(files[0].to_string_lossy().as_ref()));
        assert_run_body(&workers, &agent.id, "plugin-9-private", "PRIVATE_RUN_RECIPE");

        let cache_path = get_plugin_skills_cache_path(get_plugin_skills_dir(&root));
        let cache_before = fs::read(&cache_path).unwrap();
        let private_before = fs::read(&files[0]).unwrap();
        let mut refusal = published.clone();
        refusal.plugins.retain(|plugin| plugin.identifier.plugin_db_id.as_deref() != Some("9"));
        refusal.plugins[0].install_path = std::path::PathBuf::from("relative/invalid-stage");
        refusal.auth_blocked = vec![PluginAuthBlock { plugin_id: "9".into(),
            plugin_name: "Private".into(), marketplace_name: None }];
        *loader.0.lock().unwrap() = refusal;
        assert_eq!(service.sync("live-denial-stage-failure").unwrap_err(),
            "invalid plugin skills cache snapshot");
        assert_eq!(fs::read(&cache_path).unwrap(), cache_before);
        assert!(opened_before_binding.get("plugin-9-private").is_none());
        let fetched = call(Arc::clone(&workers), "getAgentWorkflow",
            serde_json::json!({"id":agent.id,"workflowId":"plugin-9-private"}));
        assert!(fetched.is_null());
        let listed = call(Arc::clone(&workers), "getAgentWorkflows", serde_json::json!({"id":agent.id}));
        assert!(!listed.as_array().unwrap().iter().any(|row| row["id"] == "plugin-9-private"));
        assert!(prepare_workflow_run_now(Arc::clone(&workers),
            &serde_json::json!({"id":agent.id,"workflowId":"plugin-9-private"})).unwrap().is_none());
        assert_eq!(expand_workflow_references(Arc::clone(&workers), &agent.id,
            "USER_REQUEST", Some(&reference("plugin-9-private"))).unwrap(), "USER_REQUEST");
        let _ = call(Arc::clone(&workers), "updateAgentWorkflow", serde_json::json!({
            "id":agent.id,"workflowId":"plugin-9-private",
            "spec":{"name":"Forbidden","body":"MUST_NOT_WRITE"}
        }));
        assert_eq!(fs::read(&files[0]).unwrap(), private_before);

        for store in [opened_before_binding, workers.open_workflow_store(&agent.id).unwrap()] {
            let prompt = skill_prompt(&store);
            assert!(!prompt.contains(files[0].to_string_lossy().as_ref()));
            assert!(!prompt.contains("PRIVATE_PROVIDER_DESCRIPTION"));
            for (file, description) in [(&files[1], "ALLOWED_PROVIDER_DESCRIPTION"),
                (&user.file_path, "USER_PROVIDER_DESCRIPTION"),
                (&managed.file_path, "MANAGED_PROVIDER_DESCRIPTION")] {
                assert!(prompt.contains(file.to_string_lossy().as_ref()), "missing actual skill path");
                assert!(prompt.contains(description), "missing {description}");
            }
        }
        for (id, body) in [("plugin-10-allowed", "ALLOWED_RUN_RECIPE"),
            (user.id.as_str(), "USER_RUN_RECIPE"), ("managed-live", "MANAGED_RUN_RECIPE")] {
            assert_run_body(&workers, &agent.id, id, body);
        }
        *loader.0.lock().unwrap() = published;
        assert_eq!(service.sync("fresh-success").unwrap().len(), 2);
        assert_run_body(&workers, &agent.id, "plugin-9-private", "PRIVATE_RUN_RECIPE");
        let restored = skill_prompt(&workers.open_workflow_store(&agent.id).unwrap());
        assert!(restored.contains("PRIVATE_PROVIDER_DESCRIPTION"));
        assert!(restored.contains(files[0].to_string_lossy().as_ref()));
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}
