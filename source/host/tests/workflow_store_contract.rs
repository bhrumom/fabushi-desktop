use std::fs;use std::sync::{Arc,mpsc,atomic::{AtomicUsize,Ordering}};use std::time::{Duration,SystemTime,UNIX_EPOCH};
use mahayana_host_runtime::workflows::stat_keyed_parse_cache::{StatKeyedParseCache,mtime_tick_could_still_hide_an_edit};
use mahayana_host_runtime::workflows::workflow_library::{GlobalWorkflowLibrary,LEGACY_WORKFLOW_FILENAME,WORKFLOW_FILENAME,WorkflowSpec,WorkflowTrigger,parse_workflow_file,serialize_workflow_file};
use mahayana_host_runtime::workflows::workflow_store::{FileWorkflowStore,agent_has_workflows};
fn root(label:&str)->std::path::PathBuf{let n=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();std::env::temp_dir().join(format!("fabushi-workflow-{label}-{}-{n}",std::process::id()))}
#[test]fn frontmatter_round_trip(){let raw="---\nname: \"Deploy helper\"\ndescription: \"desc\"\nmetadata:\n  source: \"https://example.test/skill.md\"\ntrigger:\n  schedule: \"0 9 * * 1-5\"\n  enabled: false\ncustom: true\n---\n# Body\nDo it.\n";let p=parse_workflow_file(raw).unwrap();assert_eq!(p.spec.name,"Deploy helper");assert_eq!(p.spec.source_ref.as_deref(),Some("https://example.test/skill.md"));assert_eq!(p.spec.trigger.as_ref().unwrap().schedule,"0 9 * * 1-5");assert!(!p.spec.trigger.as_ref().unwrap().is_enabled);let encoded=serialize_workflow_file(&p.spec,&p.data);assert!(encoded.contains("custom: true"));assert!(encoded.contains("# Body"));}
#[test]fn global_library_crud_and_legacy_rename(){let root=root("library");let library=GlobalWorkflowLibrary::new(&root);let spec=WorkflowSpec{name:"Review PR".into(),description:"review".into(),body:"Read the diff".into(),trigger:None,source_ref:None};let first=library.create(&spec).unwrap().unwrap();assert_eq!(first.id,"review-pr");let second=library.create(&spec).unwrap().unwrap();assert_eq!(second.id,"review-pr-2");fs::write(library.folder(&first.id).join("helper.sh"),"echo hi").unwrap();assert_eq!(library.get(&first.id).unwrap().helper_scripts,vec!["helper.sh"]);assert!(library.remove(&second.id).unwrap());let legacy=library.folder("legacy");fs::create_dir_all(&legacy).unwrap();fs::write(legacy.join(LEGACY_WORKFLOW_FILENAME),"---\nname: Legacy\n---\nbody\n").unwrap();library.rename_legacy_recipe_files();assert!(legacy.join(WORKFLOW_FILENAME).is_file());let _=fs::remove_dir_all(root);}
#[test]fn store_projects_automation_and_migrates_legacy(){let root=root("store");let agent=root.join("agents/a");let global=root.join("workflows");let legacy=agent.join("workflows/old");fs::create_dir_all(&legacy).unwrap();fs::write(legacy.join(LEGACY_WORKFLOW_FILENAME),"---\nname: Old\n---\nLegacy body\n").unwrap();assert!(agent_has_workflows(&agent));let store=FileWorkflowStore::new(&agent,&global);assert!(global.join("old/SKILL.md").is_file());let created=store.create(&WorkflowSpec{name:"Daily".into(),description:String::new(),body:"Do daily".into(),trigger:Some(WorkflowTrigger{schedule:"0 9 * * 1-5".into(),is_enabled:true}),source_ref:None}).unwrap().unwrap();assert_eq!(created.source,"automation");assert!(store.list_all().iter().any(|w|w.id==created.id));let _=fs::remove_dir_all(root);}
#[test]fn racy_mtime_boundary(){assert!(mtime_tick_could_still_hide_an_edit(9_000,10_999));assert!(!mtime_tick_could_still_hide_an_edit(9_000,11_000));}


#[test]
fn workflow_store_routes_enablement_mutations_through_agent_owner() {
    let root = root("enablement-owner");
    let agent = root.join("agents/a");
    let global = root.join("global-workflows");
    let store = FileWorkflowStore::new(&agent, &global);
    let created = store
        .create(&WorkflowSpec {
            name: "Enablement proof".into(),
            description: String::new(),
            body: "Do the thing".into(),
            trigger: None,
            source_ref: None,
        })
        .expect("create workflow")
        .expect("workflow record");

    assert!(created.is_enabled_for_agent);
    let disabled = store
        .set_enabled_for_agent(&created.id, false)
        .expect("disable workflow")
        .expect("disabled record");
    assert!(!disabled.is_enabled_for_agent);
    assert!(!store.list().iter().any(|row| row.id == created.id));

    let enablement_path = agent.join("enabled-workflows.json");
    let value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&enablement_path).expect("enablement file"),
    )
    .expect("enablement json");
    assert_eq!(value["disabled"], serde_json::json!([created.id.clone()]));

    let reenabled = store
        .set_enabled_for_agent(&created.id, true)
        .expect("reenable workflow")
        .expect("reenabled record");
    assert!(reenabled.is_enabled_for_agent);
    assert!(store.list().iter().any(|row| row.id == created.id));

    store
        .set_enabled_for_agent(&created.id, false)
        .expect("disable before remove");
    assert!(store.remove(&created.id).expect("remove workflow"));
    let value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&enablement_path).expect("enablement file after remove"),
    )
    .expect("enablement json");
    assert_eq!(value["disabled"], serde_json::json!([]));

    let _ = fs::remove_dir_all(root);
}


#[test]
fn local_skill_discovery_and_port_match_frozen_sources() {
    use mahayana_host_runtime::workflows::workflow_store::discover_local_skill_files;

    let root = root("local-skills");
    let home = root.join("home");
    let cwd = root.join("cwd");
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::create_dir_all(cwd.join(".cursor/rules")).unwrap();
    fs::write(home.join("CLAUDE.md"), "# Home memory\n").unwrap();
    fs::write(cwd.join("AGENTS.md"), "# Agents memory\n").unwrap();
    fs::write(cwd.join(".cursor/rules/REVIEW.MDC"), "# Review rule\n").unwrap();
    fs::write(cwd.join(".cursor/rules/ignore.txt"), "ignore").unwrap();

    let discovered = discover_local_skill_files(&home, &cwd);
    assert_eq!(discovered.len(), 3);
    assert!(discovered.iter().any(|row| row.fallback_name == "Claude memory"));
    assert!(discovered.iter().any(|row| row.fallback_name == "Agents memory"));
    assert!(discovered.iter().any(|row| row.fallback_name == "REVIEW"));
    assert!(!discovered.iter().any(|row| row.label.ends_with("ignore.txt")));

    let store = FileWorkflowStore::new(root.join("agent"), root.join("global"));
    let result = store.port_local_skills(&home, &cwd).expect("port local skills");
    assert_eq!(result.imported.len(), 3);
    assert!(result.skipped.is_empty());
    assert_eq!(store.list_all().len(), 3);
    assert!(store
        .list_all()
        .iter()
        .all(|workflow| workflow.source_ref.as_deref().is_some()));

    let _ = fs::remove_dir_all(root);
}


#[test]
fn workflow_library_watches_external_and_atomic_edits(){
 let root=root("watch-library");let library=GlobalWorkflowLibrary::new(&root);let(tx,rx)=mpsc::channel();
 library.set_on_change(Some(Arc::new(move||{let _=tx.send(());})));
 let spec=WorkflowSpec{name:"Watched".into(),description:"first".into(),body:"Body one".into(),trigger:None,source_ref:None};
 let record=library.create(&spec).unwrap().unwrap();
 rx.recv_timeout(Duration::from_secs(3)).expect("atomic library notification");
 assert_eq!(library.get(&record.id).unwrap().body,"Body one");
 while rx.try_recv().is_ok(){}
 let path=library.path(&record.id);
 fs::write(&path,"---\nname: Watched\ndescription: external\n---\nBody two\n").unwrap();
 rx.recv_timeout(Duration::from_secs(3)).expect("external library notification");
 let updated=library.get(&record.id).unwrap();
 assert_eq!(updated.description,"external");assert_eq!(updated.body,"Body two");
 while rx.try_recv().is_ok(){}
 fs::write(library.folder(&record.id).join("helper.py"),"print('ok')").unwrap();
 rx.recv_timeout(Duration::from_secs(3)).expect("helper directory notification");
 assert_eq!(library.get(&record.id).unwrap().helper_scripts,vec!["helper.py"]);
 library.set_on_change(None);let _=fs::remove_dir_all(root);
}

#[test]
fn stat_parse_cache_reuses_stable_fingerprints_and_invalidates_changes(){
 let root=root("stat-cache");fs::create_dir_all(&root).unwrap();let path=root.join("value.txt");fs::write(&path,"one").unwrap();
 let parses=Arc::new(AtomicUsize::new(0));let cache=StatKeyedParseCache::with_now_ms(2,Arc::new(||u128::MAX));
 let p=Arc::clone(&parses);let read_path=path.clone();let first=cache.read(std::slice::from_ref(&path),move||{p.fetch_add(1,Ordering::SeqCst);fs::read_to_string(&read_path).unwrap()}).unwrap();
 assert_eq!(first,"one");
 let p=Arc::clone(&parses);let path2=path.clone();let read_path=path2.clone();let second=cache.read(std::slice::from_ref(&path2),move||{p.fetch_add(1,Ordering::SeqCst);fs::read_to_string(&read_path).unwrap()}).unwrap();
 assert_eq!(second,"one");assert_eq!(parses.load(Ordering::SeqCst),1);
 fs::write(&path,"two-two").unwrap();
 let p=Arc::clone(&parses);let path3=path.clone();let read_path=path3.clone();let third=cache.read(std::slice::from_ref(&path3),move||{p.fetch_add(1,Ordering::SeqCst);fs::read_to_string(&read_path).unwrap()}).unwrap();
 assert_eq!(third,"two-two");assert_eq!(parses.load(Ordering::SeqCst),2);
 fs::remove_file(&path).unwrap();
 let p=Arc::clone(&parses);let path4=path.clone();
 assert!(cache.read(std::slice::from_ref(&path4),move||{p.fetch_add(1,Ordering::SeqCst);"missing".to_string()}).is_none());
 assert_eq!(parses.load(Ordering::SeqCst),2);
 let _=fs::remove_dir_all(root);
}


#[test]
fn managed_and_plugin_skills_are_aggregated_with_frozen_precedence() {
    use mahayana_host_runtime::extensions::managed_setup::managed_skills_cache::{
        ManagedSkill, get_managed_skills_dir, write_managed_skills_cache,
    };
    use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::{
        PluginSkillsCacheWriteIndex, get_plugin_skills_dir, write_plugin_skills_cache,
    };

    let root = root("managed-plugin-aggregation");
    let agent = root.join("agents/a");
    let global = root.join("workflows");
    fs::create_dir_all(&agent).unwrap();

    let managed_dir = get_managed_skills_dir(&root);
    write_managed_skills_cache(
        &managed_dir,
        &[
            ManagedSkill {
                id: "managed-one".into(),
                name: "Managed One".into(),
                description: "managed description".into(),
                body: "Managed body".into(),
            },
            ManagedSkill {
                id: "user-wins".into(),
                name: "Managed Shadow".into(),
                description: String::new(),
                body: "Should be shadowed".into(),
            },
        ],
        1234.0,
    )
    .unwrap();

    let plugin_dir = get_plugin_skills_dir(&root);
    let own_skill_dir = root.join("plugins/own");
    let foreign_skill_dir = root.join("plugins/foreign");
    fs::create_dir_all(&own_skill_dir).unwrap();
    fs::create_dir_all(&foreign_skill_dir).unwrap();
    let own_path = own_skill_dir.join("SKILL.md");
    let foreign_path = foreign_skill_dir.join("SKILL.md");
    fs::write(
        &own_path,
        "---\nname: \"Own Plugin\"\ndescription: \"plugin desc\"\ndisable-model-invocation: true\ncustom-key: \"preserve-me\"\n---\nPlugin body\n",
    )
    .unwrap();
    fs::write(own_skill_dir.join("helper.sh"), "echo helper").unwrap();
    fs::write(
        &foreign_path,
        "---\nname: \"Foreign Plugin\"\n---\nForeign body\n",
    )
    .unwrap();

    let skills = vec![
        serde_json::json!({
            "id":"plugin-own-1",
            "pluginId":"plugin-own",
            "pluginName":"Own",
            "name":"Own Plugin",
            "description":"plugin desc",
            "filePath":own_path.to_string_lossy(),
            "pluginVersion":"1",
            "installPath":own_skill_dir.to_string_lossy(),
            "skillRelativePath":"SKILL.md",
            "publisherUserId":42
        }),
        serde_json::json!({
            "id":"plugin-foreign-1",
            "pluginId":"plugin-foreign",
            "pluginName":"Foreign",
            "name":"Foreign Plugin",
            "description":"",
            "filePath":foreign_path.to_string_lossy(),
            "pluginVersion":"1",
            "installPath":foreign_skill_dir.to_string_lossy(),
            "skillRelativePath":"SKILL.md",
            "publisherUserId":7
        }),
        serde_json::json!({
            "id":"managed-one",
            "pluginId":"plugin-shadowed",
            "pluginName":"Shadowed",
            "name":"Plugin Shadow",
            "description":"",
            "filePath":foreign_path.to_string_lossy(),
            "pluginVersion":"1",
            "installPath":foreign_skill_dir.to_string_lossy(),
            "skillRelativePath":"SKILL.md",
            "publisherUserId":42
        }),
    ];
    write_plugin_skills_cache(
        &plugin_dir,
        &PluginSkillsCacheWriteIndex {
            current_user_id: Some(Some(42)),
            skills,
            auth_blocked: None,
        },
        || 2345.0,
    )
    .unwrap();

    let store = FileWorkflowStore::new(&agent, &global);
    let user = store
        .create(&WorkflowSpec {
            name: "User Wins".into(),
            description: String::new(),
            body: "User body".into(),
            trigger: None,
            source_ref: None,
        })
        .unwrap()
        .unwrap();
    assert_eq!(user.id, "user-wins");

    let all = store.list_all();
    assert_eq!(all.iter().filter(|row| row.id == "user-wins").count(), 1);
    assert_eq!(
        all.iter().find(|row| row.id == "user-wins").unwrap().source,
        "workflow"
    );
    assert_eq!(all.iter().filter(|row| row.id == "managed-one").count(), 1);
    assert_eq!(
        all.iter().find(|row| row.id == "managed-one").unwrap().source,
        "managed"
    );

    let own = all.iter().find(|row| row.id == "plugin-own-1").unwrap();
    assert_eq!(own.source, "plugin");
    assert_eq!(own.plugin_id.as_deref(), Some("plugin-own"));
    assert!(own.published_by_current_user);
    assert!(own.disable_model_invocation);
    assert_eq!(own.helper_scripts, vec!["helper.sh"]);

    let foreign = all.iter().find(|row| row.id == "plugin-foreign-1").unwrap();
    assert!(!foreign.published_by_current_user);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn only_current_user_plugin_skill_can_be_edited_and_frontmatter_is_preserved() {
    use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::{
        PluginSkillsCacheWriteIndex, get_plugin_skills_dir, write_plugin_skills_cache,
    };

    let root = root("plugin-edit");
    let agent = root.join("agents/a");
    let global = root.join("workflows");
    let plugin_dir = get_plugin_skills_dir(&root);
    let own_dir = root.join("plugins/own");
    let foreign_dir = root.join("plugins/foreign");
    fs::create_dir_all(&own_dir).unwrap();
    fs::create_dir_all(&foreign_dir).unwrap();
    let own_path = own_dir.join("SKILL.md");
    let foreign_path = foreign_dir.join("SKILL.md");
    fs::write(
        &own_path,
        "---\nname: \"Owned\"\ndisable-model-invocation: true\ncustom-key: \"keep\"\n---\nOld body\n",
    )
    .unwrap();
    fs::write(&foreign_path, "---\nname: \"Foreign\"\n---\nOld foreign\n").unwrap();

    write_plugin_skills_cache(
        &plugin_dir,
        &PluginSkillsCacheWriteIndex {
            current_user_id: Some(Some(99)),
            skills: vec![
                serde_json::json!({
                    "id":"owned-skill-1","pluginId":"owned","pluginName":"Owned",
                    "name":"Owned","description":"","filePath":own_path.to_string_lossy(),
                    "pluginVersion":"1","installPath":own_dir.to_string_lossy(),
                    "skillRelativePath":"SKILL.md","publisherUserId":99
                }),
                serde_json::json!({
                    "id":"foreign-skill-1","pluginId":"foreign","pluginName":"Foreign",
                    "name":"Foreign","description":"","filePath":foreign_path.to_string_lossy(),
                    "pluginVersion":"1","installPath":foreign_dir.to_string_lossy(),
                    "skillRelativePath":"SKILL.md","publisherUserId":5
                }),
            ],
            auth_blocked: None,
        },
        || 1.0,
    )
    .unwrap();

    let store = FileWorkflowStore::new(&agent, &global);
    let updated = store
        .update(
            "owned-skill-1",
            &WorkflowSpec {
                name: "Owned Updated".into(),
                description: "new desc".into(),
                body: "New body".into(),
                trigger: None,
                source_ref: None,
            },
        )
        .unwrap()
        .unwrap();
    assert_eq!(updated.name, "Owned Updated");
    assert_eq!(updated.body, "New body");
    let raw = fs::read_to_string(&own_path).unwrap();
    assert!(raw.contains("disable-model-invocation: true"));
    assert!(raw.contains("custom-key: \"keep\""));
    assert!(raw.contains("New body"));

    let denied = store
        .update(
            "foreign-skill-1",
            &WorkflowSpec {
                name: "Denied".into(),
                description: String::new(),
                body: "Denied body".into(),
                trigger: None,
                source_ref: None,
            },
        )
        .unwrap();
    assert!(denied.is_none());
    assert!(fs::read_to_string(&foreign_path).unwrap().contains("Old foreign"));
    assert!(!store.remove("owned-skill-1").unwrap());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn committed_plugin_admission_is_shared_by_list_get_and_update() {
    use mahayana_host_runtime::extensions::managed_setup::managed_skills_cache::{
        ManagedSkill, get_managed_skills_dir, write_managed_skills_cache,
    };
    use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::{
        get_plugin_skills_cache_path, get_plugin_skills_dir,
    };

    let root = root("plugin-admission");
    let cache_dir = get_plugin_skills_dir(&root);
    fs::create_dir_all(&cache_dir).unwrap();
    let cache_path = get_plugin_skills_cache_path(&cache_dir);
    let files = [root.join("plugins/one/SKILL.md"), root.join("plugins/two/SKILL.md")];
    for file in &files {
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, "---\nname: Review\n---\nOriginal body\n").unwrap();
    }
    write_managed_skills_cache(
        get_managed_skills_dir(&root),
        &[ManagedSkill { id: "managed-admission-control".into(), name: "Managed".into(),
            description: String::new(), body: "Managed body".into() }],
        1.0,
    ).unwrap();
    let store = FileWorkflowStore::new(root.join("agents/a"), root.join("workflows"));
    let user = store.create(&WorkflowSpec { name: "User control".into(), description: String::new(),
        body: "User body".into(), trigger: None, source_ref: None }).unwrap().unwrap();
    let base = serde_json::json!({
        "fetchedAt": 1, "currentUserId": 9,
        "skills": [
            {"id":"plugin-11-review","pluginId":"11","pluginName":"22","name":"Review",
             "description":"","filePath":files[0].to_string_lossy(),"publisherUserId":9},
            {"id":"plugin-22-review","pluginId":"22","pluginName":"Other","name":"Review",
             "description":"","filePath":files[1].to_string_lossy(),"publisherUserId":9}
        ]
    });
    let denied = serde_json::json!([{"pluginId":"22","pluginName":"display name is not authority"}]);
    let cases = [
        (None, None, vec!["plugin-11-review", "plugin-22-review"]),
        (None, Some(serde_json::json!([])), vec!["plugin-11-review", "plugin-22-review"]),
        (Some(serde_json::json!(1)), Some(denied.clone()), vec!["plugin-11-review"]),
        (None, Some(denied), vec![]),
        (Some(serde_json::json!(1)), Some(serde_json::json!(null)), vec![]),
        (Some(serde_json::json!(1)), None, vec![]),
        (Some(serde_json::json!(7)), Some(serde_json::json!([])), vec![]),
        (Some(serde_json::json!(1)), Some(serde_json::json!([])), vec!["plugin-11-review", "plugin-22-review"]),
    ];
    for (format, blocks, admitted) in cases {
        let mut value = base.clone();
        if let Some(format) = format { value["cacheFormatVersion"] = format; }
        if let Some(blocks) = blocks { value["authBlocked"] = blocks; }
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(&cache_path, &bytes).unwrap();
        let all = store.list_all();
        assert!(all.iter().any(|record| record.id == user.id));
        assert!(all.iter().any(|record| record.id == "managed-admission-control"));
        let mut actual = all.iter().filter(|record| record.source == "plugin")
            .map(|record| record.id.as_str()).collect::<Vec<_>>();
        actual.sort();
        assert_eq!(actual, admitted, "{value}");
        for (id, file) in ["plugin-11-review", "plugin-22-review"].into_iter().zip(&files) {
            let allowed = admitted.contains(&id);
            assert_eq!(store.get(id).is_some(), allowed, "get {id}: {value}");
            assert_eq!(store.list().iter().any(|record| record.id == id), allowed, "list {id}: {value}");
            let before = fs::read(file).unwrap();
            let updated = store.update(id, &WorkflowSpec { name: "Updated".into(),
                description: "new description".into(), body: "Updated body".into(),
                trigger: None, source_ref: None }).unwrap();
            assert_eq!(updated.is_some(), allowed, "update {id}: {value}");
            if !allowed { assert_eq!(fs::read(file).unwrap(), before); }
        }
        assert_eq!(fs::read(&cache_path).unwrap(), bytes);
    }
    drop(store);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn workflow_store_watches_managed_and_plugin_skill_caches() {
    use mahayana_host_runtime::extensions::managed_setup::managed_skills_cache::{
        ManagedSkill, get_managed_skills_dir, write_managed_skills_cache,
    };
    use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::{
        PluginSkillsCacheWriteIndex, get_plugin_skills_dir, write_plugin_skills_cache,
    };

    let root = root("skill-watchers");
    let agent = root.join("agents/a");
    let global = root.join("workflows");
    let store = FileWorkflowStore::new(&agent, &global);
    let (tx, rx) = mpsc::channel();
    store.set_on_change(Some(Arc::new(move || {
        let _ = tx.send(());
    })));

    write_managed_skills_cache(
        get_managed_skills_dir(&root),
        &[ManagedSkill {
            id: "managed-watch".into(),
            name: "Managed Watch".into(),
            description: String::new(),
            body: "body".into(),
        }],
        1.0,
    )
    .unwrap();
    rx.recv_timeout(Duration::from_secs(3))
        .expect("managed cache watcher");

    while rx.try_recv().is_ok() {}
    write_plugin_skills_cache(
        get_plugin_skills_dir(&root),
        &PluginSkillsCacheWriteIndex {
            current_user_id: Some(Some(1)),
            skills: Vec::new(),
            auth_blocked: None,
        },
        || 2.0,
    )
    .unwrap();
    rx.recv_timeout(Duration::from_secs(3))
        .expect("plugin cache watcher");

    store.set_on_change(None);
    let _ = fs::remove_dir_all(root);
}

mod live_admission {
    use super::*;
    use std::path::{Path, PathBuf};
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

    struct Loader(Mutex<LoadedPlugins>);
    impl PluginSkillsLoader for Loader {
        fn load(&self) -> Result<LoadedPlugins, String> {
            Ok(self.0.lock().unwrap().clone())
        }
    }

    struct Fixture {
        root: PathBuf,
        loader: Arc<Loader>,
        service: SandPluginSkillsService,
        published: LoadedPlugins,
        private_path: PathBuf,
        control_path: PathBuf,
        break_rename: Arc<std::sync::atomic::AtomicBool>,
        removed_staged_files: Arc<AtomicUsize>,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = root(label);
            let mut plugins = Vec::new();
            let mut paths = Vec::new();
            for (id, name, description, body) in [
                ("9", "Private", "PRIVATE_DESCRIPTION_A", "WARM_BODY_OLD"),
                ("10", "Control", "ALLOWED_DESCRIPTION", "ALLOWED_BODY"),
            ] {
                let install = root.join(format!("plugins/cache/market/p{id}/v1"));
                let path = install.join(format!("skills/{}/SKILL.md", name.to_lowercase()));
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, format!("---\nname: {name}\n---\n{body}\n")).unwrap();
                plugins.push(serde_json::json!({
                    "identifier":{"source":"cursor-third-party","name":format!("p{id}"),
                        "pluginDbId":id,"version":"v1"},
                    "installPath":install,
                    "skills":[{"name":name,"description":description,
                        "path":format!("skills/{}/SKILL.md", name.to_lowercase())}]
                }));
                paths.push(path);
            }
            let published: LoadedPlugins = serde_json::from_value(serde_json::json!({
                "plugins":plugins,"authBlocked":[],"listedPluginIds":["9","10"],
                "listedCacheKeys":[{"marketplaceSlug":"market","pluginId":"p9"},
                    {"marketplaceSlug":"market","pluginId":"p10"}],
                "publisherFacts":{"9":{"publisherUserId":12},"10":{"publisherUserId":12}},
                "currentUserId":12
            })).unwrap();
            let loader = Arc::new(Loader(Mutex::new(published.clone())));
            let break_rename = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let removed_staged_files = Arc::new(AtomicUsize::new(0));
            let clock_break = Arc::clone(&break_rename);
            let clock_removed = Arc::clone(&removed_staged_files);
            let cache_dir = get_plugin_skills_dir(&root);
            let service = SandPluginSkillsService::new(&root, loader.clone()).with_now(Arc::new(move || {
                if clock_break.load(Ordering::SeqCst) {
                    let staged = fs::read_dir(&cache_dir).unwrap().map(Result::unwrap)
                        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
                        .map(|entry| entry.path()).collect::<Vec<_>>();
                    if !staged.is_empty() {
                        assert_eq!(staged.len(), 1, "only this fixture's real staged file exists");
                        fs::remove_file(&staged[0]).unwrap();
                        clock_removed.fetch_add(1, Ordering::SeqCst);
                        clock_break.store(false, Ordering::SeqCst);
                    }
                }
                1_000
            }));
            assert_eq!(service.sync("seed").unwrap().len(), 2);
            write_managed_skills_cache(get_managed_skills_dir(&root), &[ManagedSkill {
                id: "managed-live-control".into(), name: "Managed".into(),
                description: "MANAGED_DESCRIPTION".into(), body: "MANAGED_BODY".into(),
            }], 1_000.0).unwrap();
            Self { root, loader, service, published, private_path: paths.remove(0),
                control_path: paths.remove(0), break_rename, removed_staged_files }
        }

        fn store(&self) -> FileWorkflowStore {
            FileWorkflowStore::new(self.root.join("agents/a"), self.root.join("workflows"))
                .with_plugin_skills_admission(self.service.admission_reader())
        }

        fn cache_path(&self) -> PathBuf {
            get_plugin_skills_cache_path(get_plugin_skills_dir(&self.root))
        }

        fn refuse_with_failure(&self, rename: bool) {
            let mut loaded = self.published.clone();
            loaded.plugins.retain(|plugin| plugin.identifier.plugin_db_id.as_deref() != Some("9"));
            loaded.auth_blocked = vec![PluginAuthBlock { plugin_id: "9".into(),
                plugin_name: "not an identity".into(), marketplace_name: None }];
            if rename {
                self.break_rename.store(true, Ordering::SeqCst);
            } else {
                // The aggregate and refusal are valid. The actual strict
                // staging parser rejects this generated relative file path.
                loaded.plugins[0].install_path = PathBuf::from("relative/plugin");
            }
            *self.loader.0.lock().unwrap() = loaded;
            let error = self.service.sync("refused-publication").unwrap_err();
            if !rename {
                assert_eq!(error, "invalid plugin skills cache snapshot");
            }
            assert_eq!(self.removed_staged_files.load(Ordering::SeqCst), if rename { 1 } else { 0 });
        }

        fn assert_refused(&self, store: &FileWorkflowStore, user_id: &str) {
            assert!(self.service.admission_reader().is_denied("9"));
            assert!(!self.service.admission_reader().is_denied("10"));
            assert!(self.service.current().iter().all(|record| record.plugin_id != "9"));
            assert!(store.get("plugin-9-private").is_none());
            for rows in [store.list(), store.list_all()] {
                assert!(!rows.iter().any(|row| row.id == "plugin-9-private"));
                for id in ["plugin-10-control", "managed-live-control", user_id] {
                    assert!(rows.iter().any(|row| row.id == id), "missing {id}");
                }
            }
            let bytes = fs::read(&self.private_path).unwrap();
            assert!(store.update("plugin-9-private", &WorkflowSpec {
                name: "Forbidden edit".into(), description: String::new(),
                body: "MUST_NOT_WRITE".into(), trigger: None, source_ref: None,
            }).unwrap().is_none());
            assert_eq!(fs::read(&self.private_path).unwrap(), bytes);
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); }
    }

    fn run_failure_case(rename: bool) {
        let fixture = Fixture::new(if rename { "live-rename" } else { "live-stage" });
        let old = fixture.store();
        let user = old.create(&WorkflowSpec { name: "User live control".into(),
            description: String::new(), body: "USER_BODY".into(), trigger: None,
            source_ref: None }).unwrap().unwrap();
        assert_eq!(old.get("plugin-9-private").unwrap().body, "WARM_BODY_OLD");
        let cache_before = fs::read(fixture.cache_path()).unwrap();
        let private_before = fs::read(&fixture.private_path).unwrap();
        let control_before = fs::read(&fixture.control_path).unwrap();
        fixture.refuse_with_failure(rename);
        assert_eq!(fs::read(fixture.cache_path()).unwrap(), cache_before);
        assert_eq!(fs::read(&fixture.private_path).unwrap(), private_before);
        assert_eq!(fs::read(&fixture.control_path).unwrap(), control_before);
        fixture.assert_refused(&old, &user.id);
        fixture.assert_refused(&fixture.store(), &user.id);

        *fixture.loader.0.lock().unwrap() = fixture.published.clone();
        assert_eq!(fixture.service.sync("fresh-committed-success").unwrap().len(), 2);
        assert!(!fixture.service.admission_reader().is_denied("9"));
        assert!(old.get("plugin-9-private").is_some());
        assert!(old.update("plugin-9-private", &WorkflowSpec {
            name: "Permitted publisher edit".into(), description: "restored".into(),
            body: "AUTHORIZED_BODY".into(), trigger: None, source_ref: None,
        }).unwrap().is_some());
        assert!(fs::read_to_string(&fixture.private_path).unwrap().contains("AUTHORIZED_BODY"));
        assert!(old.get(&user.id).is_some());
        assert!(old.get("managed-live-control").is_some());
    }

    #[test]
    fn real_stage_failure_keeps_live_refusal_in_old_and_new_workflow_stores() {
        run_failure_case(false);
    }

    #[test]
    fn real_rename_failure_keeps_live_refusal_in_old_and_new_workflow_stores() {
        run_failure_case(true);
    }



    struct GatedMutationAdmission {
        inner: mahayana_host_runtime::extensions::mcp::plugin_skills::PluginSkillsAdmissionReader,
        entered: mpsc::Sender<String>,
        resume: Mutex<mpsc::Receiver<()>>,
        acquisitions: Arc<AtomicUsize>,
    }

    impl mahayana_host_runtime::extensions::mcp::plugin_skills::PluginSkillsAdmissionQuery
        for GatedMutationAdmission
    {
        fn is_denied(&self, plugin_id: &str) -> bool {
            self.inner.is_denied(plugin_id)
        }

        fn acquire(&self, plugin_id: &str) -> Result<Option<
            mahayana_host_runtime::extensions::mcp::plugin_skills::PluginSkillsAdmissionLease
        >, String> {
            assert_eq!(self.acquisitions.fetch_add(1, Ordering::SeqCst), 0,
                "the actual update must acquire its mutation lease exactly once");
            self.entered.send(plugin_id.to_string()).expect("observe actual acquire entry");
            self.resume.lock().unwrap().recv_timeout(Duration::from_secs(5))
                .expect("bounded release before the actual owner acquire");
            self.inner.acquire(plugin_id)
        }
    }

    struct ReleaseAcquisition(Option<mpsc::Sender<()>>);
    impl Drop for ReleaseAcquisition {
        fn drop(&mut self) {
            if let Some(release) = self.0.take() { let _ = release.send(()); }
        }
    }

    fn update_across_committed_publication(mode: &str) {
        let fixture = Fixture::new(&format!("mutation-currentness-{mode}"));
        let (entered_tx, entered_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let acquisitions = Arc::new(AtomicUsize::new(0));
        let admission = Arc::new(GatedMutationAdmission {
            inner: fixture.service.admission_reader(), entered: entered_tx,
            resume: Mutex::new(resume_rx), acquisitions: Arc::clone(&acquisitions),
        });
        let store = FileWorkflowStore::new(
            fixture.root.join("agents/a"), fixture.root.join("workflows"),
        ).with_plugin_skills_admission(admission);
        let original = store.get("plugin-9-private").expect("current committed skill");
        assert!(original.published_by_current_user);
        let original_bytes = fs::read(&fixture.private_path).unwrap();
        let control_bytes = fs::read(&fixture.control_path).unwrap();
        let remapped = fixture.private_path.parent().unwrap().parent().unwrap()
            .join("remapped/SKILL.md");
        fs::create_dir_all(remapped.parent().unwrap()).unwrap();
        fs::write(&remapped, "---\nname: Private\n---\nREMAPPED_BODY\n").unwrap();
        let remapped_bytes = fs::read(&remapped).unwrap();

        std::thread::scope(|scope| {
            let mut release = ReleaseAcquisition(Some(resume_tx));
            let (finished_tx, finished_rx) = mpsc::channel();
            let store_ref = &store;
            let worker = scope.spawn(move || {
                let result = store_ref.update("plugin-9-private", &WorkflowSpec {
                    name: "Updated private".into(), description: "authorized only if still current".into(),
                    body: "MUTATION_CURRENTNESS_WRITE".into(), trigger: None, source_ref: None,
                });
                finished_tx.send(result).expect("actual mutation completion");
            });
            assert_eq!(entered_rx.recv_timeout(Duration::from_secs(5)).unwrap(), "9");
            assert_eq!(acquisitions.load(Ordering::SeqCst), 1);
            assert!(matches!(finished_rx.try_recv(), Err(mpsc::TryRecvError::Empty)),
                "the observed acquisition gate precedes actual mutation");

            let mut loaded = fixture.published.clone();
            match mode {
                "different-user" => loaded.current_user_id = Some(99),
                "unknown-user" => loaded.current_user_id = None,
                "remapped-record" => {
                    let plugin = loaded.plugins.iter_mut()
                        .find(|plugin| plugin.identifier.plugin_db_id.as_deref() == Some("9")).unwrap();
                    plugin.skills[0].path = "skills/remapped/SKILL.md".into();
                }
                "removed-record" => {
                    loaded.plugins.iter_mut()
                        .find(|plugin| plugin.identifier.plugin_db_id.as_deref() == Some("9"))
                        .unwrap().skills.clear();
                }
                "unchanged-grant" => {}
                _ => panic!("unknown currentness control"),
            }
            *fixture.loader.0.lock().unwrap() = loaded;
            let current = fixture.service.sync("publish-during-workflow-acquire")
                .expect("the real publisher commits while acquisition is held before SH");
            assert_eq!(current.len(), if mode == "removed-record" { 1 } else { 2 });
            assert!(!fixture.service.admission_reader().is_denied("9"),
                "this is a publisher/record race, not a negative-admission rejection");
            let committed_index = fs::read(fixture.cache_path()).unwrap();
            let published_index = fixture.service.current_index().unwrap();
            match mode {
                "different-user" => assert_eq!(published_index.current_user_id, Some(99)),
                "unknown-user" => assert_eq!(published_index.current_user_id, None),
                "remapped-record" => assert_eq!(
                    published_index.skills.iter().find(|record| record.id == "plugin-9-private")
                        .unwrap().file_path, remapped.to_string_lossy()),
                "removed-record" => assert!(!published_index.skills.iter()
                    .any(|record| record.id == "plugin-9-private")),
                "unchanged-grant" => assert_eq!(published_index.current_user_id, Some(12)),
                _ => unreachable!(),
            }
            assert_eq!(fs::read(&fixture.private_path).unwrap(), original_bytes,
                "publisher itself leaves the earlier skill bytes intact");
            assert_eq!(fs::read(&remapped).unwrap(), remapped_bytes);
            release.0.take().unwrap().send(()).expect("resume actual acquire");
            let result = finished_rx.recv_timeout(Duration::from_secs(5))
                .expect("bounded actual mutation return").expect("workflow update protocol");
            worker.join().expect("actual workflow update worker");
            if mode == "unchanged-grant" {
                let updated = result.expect("current publisher remains authorized across an unchanged refresh");
                assert!(updated.published_by_current_user);
                assert_eq!(updated.body, "MUTATION_CURRENTNESS_WRITE");
                assert!(fs::read_to_string(&fixture.private_path).unwrap()
                    .contains("MUTATION_CURRENTNESS_WRITE"));
            } else {
                assert!(result.is_none(), "{mode}: stale publisher/record must be refused");
                assert_eq!(fs::read(&fixture.private_path).unwrap(), original_bytes,
                    "{mode}: the earlier selected path was modified");
            }
            assert_eq!(fs::read(&remapped).unwrap(), remapped_bytes,
                "the resumed old request must not overwrite a remapped current skill");
            assert_eq!(fs::read(&fixture.control_path).unwrap(), control_bytes);
            assert_eq!(fs::read(fixture.cache_path()).unwrap(), committed_index,
                "workflow mutation cannot replace the committed permission index");
        });
        assert_eq!(acquisitions.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn real_workflow_mutation_rechecks_committed_publisher_and_record_after_lease() {
        for mode in ["different-user", "unknown-user", "remapped-record", "removed-record"] {
            update_across_committed_publication(mode);
        }
    }

    #[test]
    fn real_workflow_mutation_keeps_current_publisher_after_unchanged_publication() {
        update_across_committed_publication("unchanged-grant");
    }

    #[cfg(unix)]
    fn set_old_modified(path: &Path, modified: SystemTime) {
        fs::File::open(path).unwrap().set_modified(modified).unwrap();
        assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), modified);
    }

    #[cfg(unix)]
    #[test]
    fn real_old_mtime_index_and_body_cache_hits_still_consult_live_admission() {
        let fixture = Fixture::new("live-warm-cache");
        let old_time = UNIX_EPOCH + Duration::from_secs(1_000);
        for path in [fixture.cache_path(), fixture.private_path.clone(),
            fixture.private_path.parent().unwrap().to_path_buf()] {
            set_old_modified(&path, old_time);
        }
        let warm = fixture.store();
        let user = warm.create(&WorkflowSpec { name: "User warm control".into(),
            description: String::new(), body: "USER_BODY".into(), trigger: None,
            source_ref: None }).unwrap().unwrap();
        let first = warm.get("plugin-9-private").unwrap();
        assert_eq!(first.body, "WARM_BODY_OLD");
        assert_eq!(first.description, "PRIVATE_DESCRIPTION_A");

        // Same inode, byte length and old mtime force the actual production
        // caches to reuse their parsed values. A cold store must see new bytes.
        let cache_path = fixture.cache_path();
        let old_index = fs::read_to_string(&cache_path).unwrap();
        let new_index = old_index.replace("PRIVATE_DESCRIPTION_A", "PRIVATE_DESCRIPTION_B");
        assert_ne!(old_index, new_index);
        assert_eq!(old_index.len(), new_index.len());
        fs::write(&cache_path, &new_index).unwrap();
        set_old_modified(&cache_path, old_time);
        let old_body = fs::read_to_string(&fixture.private_path).unwrap();
        let new_body = old_body.replace("WARM_BODY_OLD", "WARM_BODY_NEW");
        assert_ne!(old_body, new_body);
        assert_eq!(old_body.len(), new_body.len());
        fs::write(&fixture.private_path, &new_body).unwrap();
        set_old_modified(&fixture.private_path, old_time);
        let hit = warm.get("plugin-9-private").unwrap();
        assert_eq!(hit.description, "PRIVATE_DESCRIPTION_A", "actual index cache hit");
        assert_eq!(hit.body, "WARM_BODY_OLD", "actual body cache hit");
        let cold = fixture.store().get("plugin-9-private").unwrap();
        assert_eq!(cold.description, "PRIVATE_DESCRIPTION_B");
        assert_eq!(cold.body, "WARM_BODY_NEW");

        let bytes = fs::read(&cache_path).unwrap();
        fixture.refuse_with_failure(false);
        assert_eq!(fs::read(&cache_path).unwrap(), bytes);
        fixture.assert_refused(&warm, &user.id);
        fixture.assert_refused(&fixture.store(), &user.id);
        assert_eq!(fs::read_to_string(&fixture.private_path).unwrap(), new_body);
    }
}
