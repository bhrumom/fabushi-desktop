use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::mcp::plugin_skills::{
    InstalledPlugin, InstalledSkill, ListedCacheKey, LoadedPlugins, PluginIdentifier,
    PluginLoadFailure, PluginSkillsLoader, PublisherFacts, SandPluginSkillsService,
    plugin_auth_blocks_from_failures, plugin_contents_to_skill_records,
    prune_uninstalled_plugin_dirs, skill_records_identity,
};
use mahayana_host_runtime::workflows::workflow_library::{WorkflowSpec, parse_workflow_file};
use mahayana_host_runtime::workflows::workflow_store::FileWorkflowStore;

#[test]
fn plugin_projection_is_stable_unique_and_carries_publisher_facts() {
    let plugin = InstalledPlugin {
        identifier: PluginIdentifier {
            source: "cursor-first-party".into(),
            name: "base".into(),
            plugin_db_id: Some("42".into()),
            version: Some("abc".into()),
        },
        display_name: Some("Display".into()),
        load_error: None,
        install_path: PathBuf::from("/tmp/plugin"),
        skills: vec![
            InstalledSkill {
                name: Some("Review".into()),
                description: Some("one".into()),
                path: "skills/review/SKILL.md".into(),
            },
            InstalledSkill {
                name: Some("Review".into()),
                description: Some("two".into()),
                path: "skills/review2/SKILL.md".into(),
            },
        ],
    };
    let records = plugin_contents_to_skill_records(
        &[plugin],
        &HashMap::from([(
            "42".into(),
            PublisherFacts {
                publisher_user_id: Some(7),
                marketplace_team_id: Some(9),
            },
        )]),
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].id, "plugin-42-review");
    assert_eq!(records[1].id, "plugin-42-review-2");
    assert_eq!(records[0].publisher_user_id, Some(7));
    assert_eq!(records[0].marketplace_team_id, Some(9));
    assert!(skill_records_identity(&records).contains("@abc"));
}

#[test]
fn auth_failures_are_deduplicated_and_non_auth_failures_are_ignored() {
    let failures = vec![
        PluginLoadFailure {
            plugin_db_id: Some("1".into()),
            plugin_id: None,
            plugin_name: "p".into(),
            marketplace_name: Some("m".into()),
            error_message: "permission denied".into(),
            error_kind: Some("user_git_access".into()),
        },
        PluginLoadFailure {
            plugin_db_id: Some("1".into()),
            plugin_id: None,
            plugin_name: "p".into(),
            marketplace_name: None,
            error_message: "permission denied".into(),
            error_kind: Some("user_git_access".into()),
        },
        PluginLoadFailure {
            plugin_db_id: Some("2".into()),
            plugin_id: None,
            plugin_name: "x".into(),
            marketplace_name: None,
            error_message: "network".into(),
            error_kind: Some("network".into()),
        },
        PluginLoadFailure {
            plugin_db_id: None,
            plugin_id: Some("1".into()),
            plugin_name: "looks like another canonical ID".into(),
            marketplace_name: None,
            error_message: "permission denied".into(),
            error_kind: Some("user_git_access".into()),
        },
        PluginLoadFailure {
            plugin_db_id: Some(" ".into()),
            plugin_id: Some("3".into()),
            plugin_name: "missing canonical identity".into(),
            marketplace_name: None,
            error_message: "permission denied".into(),
            error_kind: Some("user_git_access".into()),
        },
    ];
    let blocks = plugin_auth_blocks_from_failures(&failures);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].plugin_id, "1");
}

#[test]
fn prune_keeps_listed_or_indexed_plugin_directories() {
    let root = std::env::temp_dir().join(format!("fabushi-plugin-prune-{}", uuid::Uuid::new_v4()));
    for path in ["m/a", "m/b", "m/c"] {
        fs::create_dir_all(root.join(path)).unwrap();
    }
    let indexed = root.join("m/b/skills/x/SKILL.md");
    prune_uninstalled_plugin_dirs(
        &root,
        &[ListedCacheKey {
            marketplace_slug: "m".into(),
            plugin_id: "a".into(),
        }],
        &[indexed.to_string_lossy().into_owned()],
    );
    assert!(root.join("m/a").exists());
    assert!(root.join("m/b").exists());
    assert!(!root.join("m/c").exists());
    let _ = fs::remove_dir_all(root);
}

struct Loader {
    value: Mutex<Option<LoadedPlugins>>,
}
impl PluginSkillsLoader for Loader {
    fn load(&self) -> Result<LoadedPlugins, String> {
        self.value
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| "no load".into())
    }
}

#[test]
fn service_sync_writes_real_cache_and_reports_change() {
    let root =
        std::env::temp_dir().join(format!("fabushi-plugin-service-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let install = root.join("plugins/cache/m/p");
    fs::create_dir_all(install.join("skills/a")).unwrap();
    let loader = Arc::new(Loader {
        value: Mutex::new(Some(LoadedPlugins {
            plugins: vec![InstalledPlugin {
                identifier: PluginIdentifier {
                    source: "cursor-third-party".into(),
                    name: "p".into(),
                    plugin_db_id: Some("9".into()),
                    version: Some("v1".into()),
                },
                display_name: None,
                load_error: None,
                install_path: install,
                skills: vec![InstalledSkill {
                    name: Some("Skill".into()),
                    description: Some("desc".into()),
                    path: "skills/a/SKILL.md".into(),
                }],
            }],
            auth_blocked: vec![],
            listed_plugin_ids: vec!["9".into()],
            listed_cache_keys: vec![ListedCacheKey {
                marketplace_slug: "m".into(),
                plugin_id: "p".into(),
            }],
            publisher_facts: HashMap::new(),
            current_user_id: Some(12),
        })),
    });
    let events = Arc::new(Mutex::new(Vec::new()));
    let service = SandPluginSkillsService::new(&root, loader)
        .with_now(Arc::new(|| 1000))
        .with_reporter({
            let events = events.clone();
            Arc::new(move |event| events.lock().unwrap().push(event))
        });
    let records = service.sync("startup").unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(service.current().len(), 1);
    assert_eq!(service.current_index().unwrap().current_user_id, Some(12));
    assert_eq!(events.lock().unwrap()[0].outcome, "ok");
    assert!(events.lock().unwrap()[0].changed);
    service.dispose();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn successful_refresh_uses_only_its_current_publisher_identity_for_real_workflow_edits() {
    const ACCOUNT_A: u64 = 101;
    const ACCOUNT_B: u64 = 202;
    const SKILL_A: &str = "plugin-41-review";
    const SKILL_B: &str = "plugin-42-review";

    for next_identity in [Some(ACCOUNT_B), None] {
        let root = std::env::temp_dir().join(format!(
            "fabushi-plugin-publisher-identity-{}",
            uuid::Uuid::new_v4()
        ));
        let agent = root.join("agents/a");
        let global = root.join("workflows");
        fs::create_dir_all(&agent).unwrap();
        let mut plugins = Vec::new();
        let mut publisher_facts = HashMap::new();
        let mut listed_cache_keys = Vec::new();
        let mut skill_paths = Vec::new();
        for (plugin_id, name, publisher) in [("41", "first", ACCOUNT_A), ("42", "second", ACCOUNT_B)] {
            let install = root.join("plugins/cache/team").join(name);
            let skill_dir = install.join("skills/review");
            let skill_path = skill_dir.join("SKILL.md");
            fs::create_dir_all(&skill_dir).unwrap();
            fs::write(
                &skill_path,
                format!(
                    "---\nname: Review\ncustom-key: preserve\n---\nOriginal {name} recipe.\n"
                ),
            )
            .unwrap();
            skill_paths.push(skill_path);
            plugins.push(InstalledPlugin {
                identifier: PluginIdentifier {
                    source: "cursor-third-party".into(),
                    name: name.into(),
                    plugin_db_id: Some(plugin_id.into()),
                    version: Some("unchanged-version".into()),
                },
                display_name: None,
                load_error: None,
                install_path: install,
                skills: vec![InstalledSkill {
                    name: Some("Review".into()),
                    description: Some("Shared team recipe".into()),
                    path: "skills/review/SKILL.md".into(),
                }],
            });
            publisher_facts.insert(plugin_id.into(), PublisherFacts {
                publisher_user_id: Some(publisher),
                marketplace_team_id: Some(8),
            });
            listed_cache_keys.push(ListedCacheKey {
                marketplace_slug: "team".into(),
                plugin_id: name.into(),
            });
        }
        let mut loaded = LoadedPlugins {
            plugins,
            auth_blocked: Vec::new(),
            listed_plugin_ids: vec!["41".into(), "42".into()],
            listed_cache_keys,
            publisher_facts,
            current_user_id: Some(ACCOUNT_A),
        };
        let loader = Arc::new(Loader {
            value: Mutex::new(Some(loaded.clone())),
        });
        let events = Arc::new(Mutex::new(Vec::new()));
        let service = SandPluginSkillsService::new(&root, loader.clone())
            .with_now(Arc::new(|| 1000))
            .with_reporter({
                let events = Arc::clone(&events);
                Arc::new(move |event| events.lock().unwrap().push(event))
            });
        let original_records = service.sync("startup").unwrap();
        assert_eq!(original_records.len(), 2);
        let store = FileWorkflowStore::new(&agent, &global);
        assert!(store.get(SKILL_A).unwrap().published_by_current_user);
        assert!(!store.get(SKILL_B).unwrap().published_by_current_user);

        // Keep both the owner and the already-open workflow consumer alive.
        // An otherwise identical snapshot is not a permission change.
        *loader.value.lock().unwrap() = Some(loaded.clone());
        assert_eq!(service.sync("refresh").unwrap(), original_records);
        assert!(!events.lock().unwrap().last().unwrap().changed);

        loaded.current_user_id = next_identity;
        *loader.value.lock().unwrap() = Some(loaded.clone());
        let refreshed_records = service.sync("auth").unwrap();
        assert_eq!(refreshed_records, original_records);
        let committed = service.current_index().unwrap();
        assert_eq!(committed.current_user_id, next_identity);
        assert!(committed.auth_blocked.is_empty());
        assert!(events.lock().unwrap().last().unwrap().changed);

        let original_a_bytes = fs::read(&skill_paths[0]).unwrap();
        let original_b_bytes = fs::read(&skill_paths[1]).unwrap();
        let edit = WorkflowSpec {
            name: "Updated Review".into(),
            description: "Current publisher edit".into(),
            body: "Updated recipe from the current publisher.".into(),
            trigger: None,
            source_ref: None,
        };
        assert_eq!(store.list().len(), 2, "identity loss does not uninstall eligible skills");
        assert!(!store.get(SKILL_A).unwrap().published_by_current_user);
        assert_eq!(
            store.get(SKILL_B).unwrap().published_by_current_user,
            next_identity == Some(ACCOUNT_B)
        );
        assert!(store.update(SKILL_A, &edit).unwrap().is_none());
        assert_eq!(fs::read(&skill_paths[0]).unwrap(), original_a_bytes);
        assert_eq!(fs::read(&skill_paths[1]).unwrap(), original_b_bytes);

        // Reopening proves the newly committed publisher decision is durable;
        // it does not claim account admission for an unrefreshed old process.
        drop(store);
        let reopened = FileWorkflowStore::new(&agent, &global);
        assert!(!reopened.get(SKILL_A).unwrap().published_by_current_user);
        assert!(reopened.update(SKILL_A, &edit).unwrap().is_none());
        assert_eq!(fs::read(&skill_paths[0]).unwrap(), original_a_bytes);
        assert_eq!(fs::read(&skill_paths[1]).unwrap(), original_b_bytes);

        let updated_b = reopened.update(SKILL_B, &edit).unwrap();
        if next_identity == Some(ACCOUNT_B) {
            let updated_b = updated_b.expect("the current publisher must retain edit access");
            assert_eq!(updated_b.body, edit.body);
            assert!(updated_b.published_by_current_user);
            let updated_bytes = fs::read(&skill_paths[1]).unwrap();
            assert_ne!(updated_bytes, original_b_bytes);
            let updated_text = String::from_utf8(updated_bytes).unwrap();
            assert!(updated_text.contains(&edit.body));
            let updated_frontmatter = parse_workflow_file(&updated_text).unwrap().data;
            assert_eq!(
                updated_frontmatter.get("custom-key").and_then(serde_json::Value::as_str),
                Some("preserve")
            );
        } else {
            assert!(updated_b.is_none(), "unknown identity has no publisher edit grant");
            assert_eq!(fs::read(&skill_paths[1]).unwrap(), original_b_bytes);
        }
        assert_eq!(fs::read(&skill_paths[0]).unwrap(), original_a_bytes);

        *loader.value.lock().unwrap() = Some(loaded);
        assert_eq!(service.sync("refresh").unwrap(), original_records);
        let reports = events.lock().unwrap();
        assert_eq!(reports.len(), 4);
        assert!(reports.iter().all(|report| report.outcome == "ok"));
        assert_eq!(
            reports.iter().map(|report| report.changed).collect::<Vec<_>>(),
            vec![true, false, true, false]
        );
        drop(reports);
        service.dispose();
        drop(reopened);
        let _ = fs::remove_dir_all(root);
    }
}
