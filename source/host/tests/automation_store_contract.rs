use std::fs;use std::io;use std::sync::{Arc,mpsc};use std::time::{Duration,SystemTime,UNIX_EPOCH};
use mahayana_host_runtime::automations::automation::{AutomationRecord,AutomationSpec,AUTOMATION_MAX_RUN_HISTORY};
use mahayana_host_runtime::automations::automation_store::{AutomationDefinitionInspectionState,AutomationStoreError,FileAutomationStore,agent_has_automations,inspect_agent_automation_definitions,parse_stored_config};
use mahayana_host_runtime::automations::automation_trigger::{parse_stored_trigger,slack_scope_matches,trigger_schedule};
use mahayana_host_runtime::automations::routine_notices::{GITHUB_LISTENER_SCOPE,routine_notice_ids_to_raise};
use serde_json::json;
fn root(label:&str)->std::path::PathBuf{let n=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();std::env::temp_dir().join(format!("fabushi-automation-{label}-{}-{n}",std::process::id()))}
#[test]fn trigger_parser_normalizes_slack_github_and_group_contract(){let slack=parse_stored_trigger(&json!({"type":"slack","channel":" #eng ","match":{"kind":"reaction","emoji":[":Ship:","ship","BAD SPACE"],"bySelf":true}})).unwrap();assert_eq!(slack["channel"],"#eng");assert_eq!(slack["match"]["emoji"],json!(["ship"]));assert!(slack_scope_matches("#Eng","eng"));assert!(!slack_scope_matches("@Eng","#eng"));let github=parse_stored_trigger(&json!({"type":"github","repo":"Org/Repo","events":["pr-opened","ci-passed"],"userAllowlist":["@Alice","alice"],"ciBranch":" main "})).unwrap();assert_eq!(github["userAllowlist"],json!(["Alice"]));assert_eq!(github["ciBranch"],"main");let group=parse_stored_trigger(&json!({"type":"group","listeners":[{"type":"cron","schedule":"0 9 * * 1-5"},slack]})).unwrap();assert_eq!(trigger_schedule(&group).as_deref(),Some("0 9 * * 1-5"));}
#[test]fn file_store_persists_crud_run_history_and_durable_footprint(){let root=root("crud");let agent=root.join("agent");let dir=agent.join("automations");fs::create_dir_all(&agent).unwrap();let store=FileAutomationStore::new(&dir);assert_eq!(inspect_agent_automation_definitions(&agent).state,AutomationDefinitionInspectionState::DirMissing);let spec=AutomationSpec{name:"Morning Check".into(),prompt:" check inbox ".into(),trigger:json!({"type":"cron","schedule":"0 9 * * 1-5"}),is_enabled:None};let record=store.upsert(&spec,1000.0).unwrap().unwrap();assert_eq!(record.id,"morning-check");assert_eq!(record.prompt,"check inbox");assert!(agent_has_automations(&agent));assert_eq!(inspect_agent_automation_definitions(&agent).valid_definition_count,1);let second=store.upsert(&spec,1100.0).unwrap().unwrap();assert_eq!(second.id,"morning-check-2");assert!(!store.set_enabled(&record.id,false).unwrap().unwrap().is_enabled);let run=store.begin_run(&record.id,"manual",1200.0,Some(" evt "),Some("run-1"),None).unwrap().unwrap();assert_eq!(run.trigger,"manual");let finished=store.finish_run(&record.id,"run-1","error",1300.0,Some(" failed ")).unwrap().unwrap();assert_eq!(finished.runs[0].status,"error");for i in 0..(AUTOMATION_MAX_RUN_HISTORY+5){store.begin_run(&record.id,"schedule",2000.0+i as f64,None,Some(&format!("r-{i}")),None).unwrap();}assert_eq!(store.read_runs(&record.id).len(),AUTOMATION_MAX_RUN_HISTORY);assert!(store.remove(&second.id).unwrap());let _=fs::remove_dir_all(root);}
#[test]fn config_and_notice_contract(){let raw=r#"{"name":" Test ","prompt":" go ","schedule":"@daily","enabled":true,"createdAt":9999,"raisedNotices":[]}"#;let config=parse_stored_config(raw,5000.0).unwrap();assert_eq!(config.created_at,5000.0);assert_eq!(config.name,"Test");let trigger=json!({"type":"github","repo":"org/repo","events":["pr-opened"]});let notices=routine_notice_ids_to_raise(1_700_000_000_000.0,&trigger,&[]);assert_eq!(notices,vec![GITHUB_LISTENER_SCOPE.to_string()]);assert!(routine_notice_ids_to_raise(1_700_000_000_000.0,&trigger,&notices).is_empty());}


#[test]
fn external_and_internal_store_changes_share_the_debounced_watcher(){
 let root=root("watch");let agent=root.join("agent");let dir=agent.join("automations");fs::create_dir_all(&agent).unwrap();
 let store=FileAutomationStore::new(&dir);let(tx,rx)=mpsc::channel();
 store.set_on_change(Some(Arc::new(move||{let _=tx.send(());})));
 let spec=AutomationSpec{name:"Watcher".into(),prompt:"watch".into(),trigger:json!({"type":"cron","schedule":"  0   9  * *  1-5 "}),is_enabled:None};
 let record=store.upsert(&spec,1000.0).unwrap().unwrap();
 rx.recv_timeout(Duration::from_secs(3)).expect("internal atomic write notification");
 assert_eq!(store.read_config(&record.id).unwrap().trigger["schedule"],"0 9 * * 1-5");
 while rx.try_recv().is_ok(){}
 let path=store.config_path(&record.id);let mut value:serde_json::Value=serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
 value["prompt"]=json!("externally edited");fs::write(&path,format!("{}\n",serde_json::to_string_pretty(&value).unwrap())).unwrap();
 rx.recv_timeout(Duration::from_secs(3)).expect("external edit notification");
 assert_eq!(store.get(&record.id).unwrap().prompt,"externally edited");
 store.set_on_change(None);let _=fs::remove_dir_all(root);
}

#[test]
fn definition_run_mutations_suppress_next_run_derivation(){
 let root=root("definition-run");let dir=root.join("automations");let store=FileAutomationStore::new(&dir);
 let spec=AutomationSpec{name:"Definition".into(),prompt:"check".into(),trigger:json!({"type":"cron","schedule":"@hourly"}),is_enabled:None};
 let record=store.upsert(&spec,1_000.0).unwrap().unwrap();
 let definition=store.record_run_definition(&record.id,2_000.0).unwrap().unwrap();
 assert!(definition.next_run_at.is_none());
 let run=store.begin_run(&record.id,"manual",2_100.0,None,Some("r"),None).unwrap().unwrap();
 let definition=store.finish_run_definition(&record.id,&run.id,"ok",2_200.0,None).unwrap().unwrap();
 assert!(definition.next_run_at.is_none());
 assert!(store.get(&record.id).unwrap().next_run_at.is_some());
 let _=fs::remove_dir_all(root);
}

fn history_store(label: &str) -> (std::path::PathBuf, FileAutomationStore, String) {
    let root = root(label);
    let store = FileAutomationStore::new(root.join("automations"));
    let automation = store.upsert(&AutomationSpec {
        name: "History acknowledgment".into(),
        prompt: "Inspect the queue".into(),
        trigger: json!({"type":"cron","schedule":"@hourly"}),
        is_enabled: Some(true),
    }, 1_000.0).expect("create automation").expect("automation");
    (root, store, automation.id)
}

fn finish_history(
    store: &FileAutomationStore,
    automation_id: &str,
    definition_only: bool,
) -> Result<Option<AutomationRecord>, AutomationStoreError> {
    if definition_only {
        store.finish_run_definition(automation_id, "run", "error", 2_000.0, Some(" failed "))
    } else {
        store.finish_run(automation_id, "run", "error", 2_000.0, Some(" failed "))
    }
}

#[test]
fn finish_requires_existing_configuration_and_matching_history_for_both_projections() {
    for definition_only in [false, true] {
        let (root, store, id) = history_store("finish-missing");
        let config = fs::read(store.config_path(&id)).expect("configuration bytes");
        let runs_path = store.runs_path(&id);

        assert!(finish_history(&store, "missing-definition", definition_only).unwrap().is_none());
        assert!(!store.get_location().join("missing-definition").exists());
        assert!(finish_history(&store, &id, definition_only).unwrap().is_none());
        assert!(!runs_path.exists(), "missing history must not be created");

        for raw in [
            &b"[ ]\n"[..],
            &br#"[ { "id": "other", "startedAt": 1000, "status": "running", "extra": true } ]
"#[..],
        ] {
            fs::write(&runs_path, raw).expect("history fixture");
            assert!(finish_history(&store, &id, definition_only).unwrap().is_none());
            assert_eq!(fs::read(&runs_path).unwrap(), raw);
            assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        }

        let history = fs::read(&runs_path).unwrap();
        fs::remove_file(store.config_path(&id)).expect("remove configuration");
        assert!(finish_history(&store, &id, definition_only).unwrap().is_none());
        assert!(!store.config_path(&id).exists());
        assert_eq!(fs::read(&runs_path).unwrap(), history);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn finish_rejects_malformed_and_ambiguous_history_without_salvaging_rows() {
    let cases: &[(&str, &[u8], bool)] = &[
        ("invalid-json", b"[", true),
        ("non-array", br#"{"id":"run","startedAt":1000}"#, true),
        ("invalid-row", br#"[42]"#, false),
        ("mixed-row", br#"[{"id":"run","startedAt":1000,"status":"running"},{"id":"bad","startedAt":"invalid"}]"#, false),
        ("duplicate-id", br#"[{"id":"run","startedAt":1000},{"id":"run","startedAt":2000}]"#, false),
        ("invalid-utf8", b"[\xff]", false),
    ];
    for definition_only in [false, true] {
        for &(label, raw, json_error) in cases {
            let (root, store, id) = history_store(label);
            let config = fs::read(store.config_path(&id)).unwrap();
            fs::write(store.runs_path(&id), raw).expect("malformed history fixture");
            if label == "mixed-row" {
                assert_eq!(store.read_runs(&id).len(), 1, "projection remains fail-soft");
            }
            match finish_history(&store, &id, definition_only).expect_err(label) {
                AutomationStoreError::Json(_) => assert!(json_error, "{label}"),
                AutomationStoreError::Io(error) => {
                    assert!(!json_error, "{label}");
                    assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{label}");
                }
            }
            assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw, "{label}");
            assert_eq!(fs::read(store.config_path(&id)).unwrap(), config, "{label}");
            let _ = fs::remove_dir_all(root);
        }
    }
}

#[test]
fn finish_rejects_duplicate_ids_before_applying_the_history_window() {
    for definition_only in [false, true] {
        let (root, store, id) = history_store("finish-duplicate-window");
        let mut rows = vec![json!({"id":"run","startedAt":1000,"status":"running"})];
        rows.extend((0..AUTOMATION_MAX_RUN_HISTORY).map(|index| {
            json!({"id":format!("other-{index}"),"startedAt":100 + index})
        }));
        rows.push(json!({"id":"duplicate","startedAt":1}));
        rows.push(json!({"id":"duplicate","startedAt":2}));
        let raw = serde_json::to_vec_pretty(&rows).unwrap();
        let config = fs::read(store.config_path(&id)).unwrap();
        fs::write(store.runs_path(&id), &raw).unwrap();
        assert!(store.read_runs(&id).iter().all(|run| run.id != "duplicate"));
        let error = finish_history(&store, &id, definition_only).expect_err("ambiguous history");
        assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == io::ErrorKind::InvalidData));
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn finish_does_not_acknowledge_a_run_outside_the_existing_history_window() {
    for definition_only in [false, true] {
        let (root, store, id) = history_store("finish-window-miss");
        let mut rows = vec![json!({"id":"run","startedAt":0,"status":"running"})];
        rows.extend((0..AUTOMATION_MAX_RUN_HISTORY).map(|index| {
            json!({"id":format!("newer-{index}"),"startedAt":index + 1})
        }));
        let raw = serde_json::to_vec_pretty(&rows).unwrap();
        let config = fs::read(store.config_path(&id)).unwrap();
        fs::write(store.runs_path(&id), &raw).unwrap();
        assert!(finish_history(&store, &id, definition_only).unwrap().is_none());
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn finish_propagates_history_read_errors_without_writing() {
    for definition_only in [false, true] {
        let (root, store, id) = history_store("finish-unreadable");
        let config = fs::read(store.config_path(&id)).unwrap();
        let runs_path = store.runs_path(&id);
        fs::create_dir(&runs_path).expect("unreadable history path");
        fs::write(runs_path.join("sentinel"), b"preserve directory").unwrap();
        assert!(store.read_runs(&id).is_empty(), "projection remains fail-soft");
        let error = finish_history(&store, &id, definition_only).expect_err("history read failure");
        assert!(matches!(error, AutomationStoreError::Io(_)));
        assert!(runs_path.is_dir());
        assert_eq!(fs::read(runs_path.join("sentinel")).unwrap(), b"preserve directory");
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn finish_propagates_prepublication_write_failure_and_preserves_bytes() {
    for definition_only in [false, true] {
        let (root, store, id) = history_store("finish-write-failure");
        let config = fs::read(store.config_path(&id)).unwrap();
        let raw = br#"[ { "id": "run", "startedAt": 1000, "status": "running" } ]"#;
        let runs_path = store.runs_path(&id);
        fs::write(&runs_path, raw).unwrap();
        // Block the real atomic writer before publication without depending
        // on permission bits or the test process's privilege level.
        let temporary = std::path::PathBuf::from(format!(
            "{}.{}.tmp", runs_path.display(), std::process::id()
        ));
        fs::create_dir(&temporary).expect("block temporary file creation");
        fs::write(temporary.join("sentinel"), b"preserve temporary directory").unwrap();
        let error = finish_history(&store, &id, definition_only).expect_err("write failure");
        assert!(matches!(error, AutomationStoreError::Io(_)));
        assert_eq!(fs::read(&runs_path).unwrap(), raw);
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        assert_eq!(fs::read(temporary.join("sentinel")).unwrap(), b"preserve temporary directory");
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn finish_preserves_valid_legacy_normalization_order_and_projection() {
    for definition_only in [false, true] {
        let (root, store, id) = history_store("finish-legacy");
        let config = fs::read(store.config_path(&id)).unwrap();
        let mut rows = vec![json!({
            "id":"run", "startedAt":1000, "finishedAt":"legacy", "event":" event ",
            "coalescedRunIds":["", "coalesced", 7]
        })];
        rows.extend((0..AUTOMATION_MAX_RUN_HISTORY + 3).map(|index| {
            json!({"id":format!("older-{index}"),"startedAt":index + 1,"trigger":"legacy","status":"legacy"})
        }));
        fs::write(store.runs_path(&id), serde_json::to_vec(&rows).unwrap()).unwrap();
        let record = finish_history(&store, &id, definition_only).unwrap().expect("committed run");
        assert_eq!(record.next_run_at.is_none(), definition_only);
        let persisted = store.read_runs(&id);
        assert_eq!(record.runs, persisted);
        assert_eq!(persisted.len(), AUTOMATION_MAX_RUN_HISTORY);
        let run = &persisted[0];
        assert_eq!(run.id, "run");
        assert_eq!(run.trigger, "schedule");
        assert_eq!(run.status, "error");
        assert_eq!(run.finished_at, Some(2_000.0));
        assert_eq!(run.detail.as_deref(), Some("failed"));
        assert_eq!(run.event.as_deref(), Some("event"));
        assert_eq!(run.coalesced_run_ids, Some(vec!["coalesced".to_string()]));
        assert!(persisted.windows(2).all(|pair| pair[0].started_at >= pair[1].started_at));
        assert!(persisted.iter().skip(1).all(|run| run.trigger == "schedule" && run.status == "ok"));
        assert!(!persisted.iter().any(|run| run.id == "older-0"));
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        let _ = fs::remove_dir_all(root);
    }
}
