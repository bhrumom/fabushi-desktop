use std::fs;use std::io;use std::sync::{Arc,mpsc};use std::time::{Duration,SystemTime,UNIX_EPOCH};
use mahayana_host_runtime::automations::automation::{AutomationRecord,AutomationRun,AutomationSpec,AUTOMATION_MAX_RUN_HISTORY,MAX_EVENTS_IN_AUTOMATION_WAKE};
use mahayana_host_runtime::automations::automation_id::stable_automation_id;
use mahayana_host_runtime::automations::automation_store::{AutomationDefinitionInspectionState,AutomationStoreError,FileAutomationStore,agent_has_automations,inspect_agent_automation_definitions,parse_stored_config,parse_stored_run};
use mahayana_host_runtime::automations::automation_trigger::{parse_stored_trigger,slack_scope_matches,trigger_schedule};
use mahayana_host_runtime::automations::routine_notices::{GITHUB_LISTENER_SCOPE,routine_notice_ids_to_raise};
use serde_json::json;
fn root(label:&str)->std::path::PathBuf{let n=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();std::env::temp_dir().join(format!("fabushi-automation-{label}-{}-{n}",std::process::id()))}
#[test]fn trigger_parser_normalizes_slack_github_and_group_contract(){let slack=parse_stored_trigger(&json!({"type":"slack","channel":" #eng ","match":{"kind":"reaction","emoji":[":Ship:","ship","BAD SPACE"],"bySelf":true}})).unwrap();assert_eq!(slack["channel"],"#eng");assert_eq!(slack["match"]["emoji"],json!(["ship"]));assert!(slack_scope_matches("#Eng","eng"));assert!(!slack_scope_matches("@Eng","#eng"));let github=parse_stored_trigger(&json!({"type":"github","repo":"Org/Repo","events":["pr-opened","ci-passed"],"userAllowlist":["@Alice","alice"],"ciBranch":" main "})).unwrap();assert_eq!(github["userAllowlist"],json!(["Alice"]));assert_eq!(github["ciBranch"],"main");let group=parse_stored_trigger(&json!({"type":"group","listeners":[{"type":"cron","schedule":"0 9 * * 1-5"},slack]})).unwrap();assert_eq!(trigger_schedule(&group).as_deref(),Some("0 9 * * 1-5"));}
#[test]fn file_store_persists_crud_run_history_and_durable_footprint(){let root=root("crud");let agent=root.join("agent");let dir=agent.join("automations");fs::create_dir_all(&agent).unwrap();let store=FileAutomationStore::new(&dir);assert_eq!(inspect_agent_automation_definitions(&agent).state,AutomationDefinitionInspectionState::DirMissing);let spec=AutomationSpec{name:"Morning Check".into(),prompt:" check inbox ".into(),trigger:json!({"type":"cron","schedule":"0 9 * * 1-5"}),is_enabled:None};let record=store.upsert(&spec,1000.0).unwrap().unwrap();assert_eq!(record.id,"morning-check");assert_eq!(record.prompt,"check inbox");assert!(agent_has_automations(&agent));assert_eq!(inspect_agent_automation_definitions(&agent).valid_definition_count,1);let second=store.upsert(&spec,1100.0).unwrap().unwrap();assert_eq!(second.id,"morning-check-2");assert!(!store.set_enabled(&record.id,false).unwrap().unwrap().is_enabled);let run=store.begin_run(&record.id,"manual",1200.0,Some(" evt "),Some("run-1"),None).unwrap().unwrap();assert_eq!(run.trigger,"manual");let finished=store.finish_run(&record.id,"run-1","error",1300.0,Some(" failed ")).unwrap().unwrap();assert_eq!(finished.runs[0].status,"error");for i in 0..(AUTOMATION_MAX_RUN_HISTORY+5){let id=format!("r-{i}");store.begin_run(&record.id,"schedule",2000.0+i as f64,None,Some(&id),None).unwrap();store.finish_run(&record.id,&id,"ok",2100.0+i as f64,None).unwrap();}assert_eq!(store.read_runs(&record.id).len(),AUTOMATION_MAX_RUN_HISTORY);assert!(store.remove(&second.id).unwrap());let _=fs::remove_dir_all(root);}
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

#[test]
fn checked_history_rejects_duplicate_object_members_without_custody_or_writes() {
    let cases: &[(&str, &[u8])] = &[
        ("status-overwrite", br#"[{"id":"run","startedAt":1000,"status":"running","status":"ok"}]"#),
        ("status-reverse", br#"[{"id":"run","startedAt":1000,"status":"ok","status":"running"}]"#),
        ("id-overwrite", br#"[{"id":"run","startedAt":1000,"id":"other","status":"running"}]"#),
        ("alias-overwrite", br#"[{"id":"run","startedAt":1000,"status":"running","coalescedRunIds":["alias"],"coalescedRunIds":[]}]"#),
        ("nested-unknown", br#"[{"id":"run","startedAt":1000,"status":"running","extension":{"key":1,"key":2}}]"#),
    ];
    for &(label, raw) in cases {
        let (root, store, id) = history_store(&format!("duplicate-member-{label}"));
        let config = fs::read(store.config_path(&id)).unwrap();
        fs::write(store.runs_path(&id), raw).unwrap();
        if label == "status-overwrite" {
            assert_eq!(store.read_runs(&id)[0].status, "ok",
                "the normal display projection is not checked delivery authority");
        }
        assert!(matches!(store.read_run_for_delivery(&id, "run"),
            Err(AutomationStoreError::Json(_))), "{label}: custody lookup");
        assert!(matches!(store.begin_run(&id, "manual", 2000.0, None, Some("new"), None),
            Err(AutomationStoreError::Json(_))), "{label}: new admission");
        assert!(matches!(store.begin_run(&id, "manual", 2000.0, None, Some("run"), None),
            Err(AutomationStoreError::Json(_))), "{label}: duplicate admission");
        assert!(matches!(store.finish_run(&id, "run", "ok", 3000.0, None),
            Err(AutomationStoreError::Json(_))), "{label}: settlement");
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw, "{label}: immutable history");
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config, "{label}: immutable definition");
        let _ = fs::remove_dir_all(root);
    }

    // Unique-key legacy rows, aliases, unknown extensions and idempotent begin
    // continue to work; this change must not block legitimate terminal ACKs.
    let (root, store, id) = history_store("unique-member-positive");
    let valid = br#"[{"id":"run","startedAt":1000,"status":"running","coalescedRunIds":["alias"],"extension":{"key":1,"other":2}}]"#;
    fs::write(store.runs_path(&id), valid).unwrap();
    assert_eq!(store.read_run_for_delivery(&id, "alias").unwrap().unwrap().id, "run");
    assert!(store.read_run_for_delivery(&id, "absent").unwrap().is_none());
    assert_eq!(store.begin_run(&id, "manual", 2000.0, None, Some("run"), None)
        .unwrap().unwrap().status, "running");
    assert_eq!(fs::read(store.runs_path(&id)).unwrap(), valid, "duplicate begin must not write");
    assert!(store.finish_run(&id, "run", "ok", 3000.0, None).unwrap().unwrap()
        .runs.iter().any(|run| run.id == "run" && run.status == "ok"));
    assert_eq!(store.read_run_for_delivery(&id, "alias").unwrap().unwrap().status, "ok");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn begin_rejects_malformed_and_ambiguous_history_without_salvaging_rows() {
    let cases: &[(&str, &[u8], bool)] = &[
        ("invalid-json", b"[", true),
        ("non-array", br#"{"id":"run","startedAt":1000}"#, true),
        ("invalid-row", br#"[42]"#, false),
        ("mixed-row", br#"[{"id":"run","startedAt":1000,"status":"running"},{"id":"bad","startedAt":"invalid"}]"#, false),
        ("duplicate-id", br#"[{"id":"run","startedAt":1000},{"id":"run","startedAt":2000}]"#, false),
        ("invalid-utf8", b"[\xff]", false),
    ];
    for &(label, raw, json_error) in cases {
        let (root, store, id) = history_store(&format!("begin-{label}"));
        let config = fs::read(store.config_path(&id)).unwrap();
        fs::write(store.runs_path(&id), raw).expect("persist invalid admission input");
        if label == "mixed-row" {
            assert_eq!(store.read_runs(&id).len(), 1, "display still projects the valid subset");
        }
        // Both a new run and the apparently matching existing run must inspect
        // the complete input before any mutation or duplicate return.
        for requested_id in ["new-run", "run"] {
            match store.begin_run(&id, "manual", 3_000.0, None, Some(requested_id), None)
                .expect_err(label)
            {
                AutomationStoreError::Json(_) => assert!(json_error, "{label}"),
                AutomationStoreError::Io(error) => {
                    assert!(!json_error, "{label}");
                    assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{label}");
                }
            }
            assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw, "{label}");
            assert_eq!(fs::read(store.config_path(&id)).unwrap(), config, "{label}");
        }
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn begin_validates_invalid_and_duplicate_tails_beyond_the_display_window() {
    for duplicate in [false, true] {
        let (root, store, id) = history_store("begin-invalid-tail");
        let mut rows = (0..AUTOMATION_MAX_RUN_HISTORY + 2).map(|index| {
            json!({"id":format!("visible-{index}"),"startedAt":1000 + index,"status":"ok"})
        }).collect::<Vec<_>>();
        if duplicate {
            rows.push(json!({"id":"old-duplicate","startedAt":1}));
            rows.push(json!({"id":"old-duplicate","startedAt":2}));
        } else {
            rows.push(json!({"id":"invalid-tail","startedAt":"not a timestamp"}));
        }
        let raw = serde_json::to_vec_pretty(&rows).unwrap();
        let config = fs::read(store.config_path(&id)).unwrap();
        fs::write(store.runs_path(&id), &raw).unwrap();
        let display = store.read_runs(&id);
        assert_eq!(display.len(), AUTOMATION_MAX_RUN_HISTORY);
        assert!(display.iter().all(|run| run.id.starts_with("visible-")));
        let error = store.begin_run(&id, "schedule", 5_000.0, None, Some("new-run"), None)
            .expect_err("the display window cannot authorize destructive admission");
        assert!(matches!(error, AutomationStoreError::Io(ref error)
            if error.kind() == io::ErrorKind::InvalidData));
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        let _ = fs::remove_dir_all(root);
    }
}


#[test]
fn execution_only_begin_refuses_replayed_primary_and_coalesced_ids_without_history_rewrite() {
    let (root, store, id) = history_store("execution-fresh-duplicate");
    let aliases = vec!["paired-alias".to_string()];
    let first = store.begin_run(
        &id, "event", 1_000.0, Some("first event"),
        Some("original"), Some(&aliases),
    ).unwrap().expect("first persisted run");
    assert_eq!(first.id, "original");
    let original_bytes = fs::read(store.runs_path(&id)).unwrap();
    for replay in ["original", "paired-alias"] {
        let error = store.begin_fresh_run(
            &id, "schedule", 9_000.0, None, Some(replay), None
        ).expect_err("previously claimed execution identity must not run again");
        assert!(matches!(error, AutomationStoreError::Io(ref error)
            if error.kind() == io::ErrorKind::AlreadyExists), "{replay}: {error}");
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), original_bytes, "{replay}");
    }
    let new_primary_alias_collision = store.begin_fresh_run(
        &id, "event", 9_000.0, None, Some("new"),
        Some(&["paired-alias".to_string()]),
    ).expect_err("coalesced replay must be rejected before any write");
    assert!(matches!(new_primary_alias_collision, AutomationStoreError::Io(ref error)
        if error.kind() == io::ErrorKind::AlreadyExists));
    assert_eq!(fs::read(store.runs_path(&id)).unwrap(), original_bytes);
    let distinct = store.begin_fresh_run(
        &id, "event", 10_000.0, None, Some("distinct"), None,
    ).unwrap().expect("new execution is admitted");
    assert_eq!(distinct.id, "distinct");
    assert_eq!(store.read_run_for_delivery(&id, "original").unwrap(), Some(first));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn begin_preserves_missing_history_and_duplicate_no_write_controls() {
    let (root, store, id) = history_store("begin-first-run");
    let config = fs::read(store.config_path(&id)).unwrap();
    assert!(store.begin_run("missing-definition", "manual", 2_000.0, None, Some(""), None)
        .unwrap().is_none(), "missing configuration retains its established precedence");
    assert!(!store.get_location().join("missing-definition").exists());
    assert!(!store.runs_path(&id).exists());
    let first = store.begin_run(&id, "manual", 2_000.0, None, None, None)
        .unwrap().expect("missing history permits the first run");
    assert!(!first.id.is_empty());
    assert_eq!(first.status, "running");
    assert_eq!(first.finished_at, None);
    let raw = fs::read(store.runs_path(&id)).unwrap();
    let duplicate = store.begin_run(&id, "event", 9_000.0, Some("must not replace"), Some(&first.id), None)
        .unwrap().expect("existing run remains the duplicate result");
    assert_eq!(duplicate, first);
    assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn begin_refuses_empty_explicit_id_before_creating_or_rewriting_history() {
    let (root, store, id) = history_store("begin-empty-id");
    let config = fs::read(store.config_path(&id)).unwrap();
    let runs_path = store.runs_path(&id);
    for raw in [None, Some(&b"[ ]\n"[..])] {
        if let Some(raw) = raw {
            fs::write(&runs_path, raw).unwrap();
        }
        let error = store.begin_run(&id, "manual", 2_000.0, None, Some(""), None)
            .expect_err("an empty ID must not poison the next history read");
        assert!(matches!(error, AutomationStoreError::Io(ref error)
            if error.kind() == io::ErrorKind::InvalidInput));
        if let Some(raw) = raw {
            assert_eq!(fs::read(&runs_path).unwrap(), raw);
        } else {
            assert!(!runs_path.exists());
        }
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    }
    // The existing parser accepts this identity literally. Empty rejection
    // must not introduce trimming or a new legacy normalization contract.
    let whitespace = br#"[ { "id": "  ", "startedAt": 1000, "status": "running" } ]"#;
    fs::write(&runs_path, whitespace).unwrap();
    let existing = store.begin_run(&id, "manual", 2_000.0, None, Some("  "), None)
        .unwrap().expect("literal legacy identity");
    assert_eq!(existing.id, "  ");
    assert_eq!(fs::read(&runs_path).unwrap(), whitespace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn begin_preserves_legacy_normalization_and_existing_history_window() {
    let (root, store, id) = history_store("begin-legacy");
    let config = fs::read(store.config_path(&id)).unwrap();
    let mut rows = vec![json!({
        "id":"legacy", "startedAt":3_000, "finishedAt":"legacy", "event":" event ",
        "trigger":"legacy", "status":"legacy", "coalescedRunIds":["", "coalesced", 7]
    })];
    rows.extend((0..AUTOMATION_MAX_RUN_HISTORY + 3).map(|index| {
        json!({"id":format!("older-{index}"),"startedAt":index + 1})
    }));
    fs::write(store.runs_path(&id), serde_json::to_vec(&rows).unwrap()).unwrap();
    let admitted = store.begin_run(&id, "event", 4_000.0, Some(" event "), Some("new-run"), None)
        .unwrap().expect("admit against valid legacy history");
    let persisted = store.read_runs(&id);
    assert_eq!(persisted.len(), AUTOMATION_MAX_RUN_HISTORY);
    assert_eq!(persisted[0], admitted);
    assert_eq!(admitted.trigger, "event");
    assert_eq!(admitted.event.as_deref(), Some("event"));
    let legacy = &persisted[1];
    assert_eq!(legacy.id, "legacy");
    assert_eq!(legacy.trigger, "schedule");
    assert_eq!(legacy.status, "ok");
    assert_eq!(legacy.finished_at, None);
    assert_eq!(legacy.event.as_deref(), Some("event"));
    assert_eq!(legacy.coalesced_run_ids, Some(vec!["coalesced".to_string()]));
    assert!(persisted.windows(2).all(|pair| pair[0].started_at >= pair[1].started_at));
    assert!(!persisted.iter().any(|run| run.id == "older-0"));
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn begin_propagates_read_and_prepublication_write_errors_without_replacing_history() {
    for read_failure in [true, false] {
        let (root, store, id) = history_store("begin-io-refusal");
        let config = fs::read(store.config_path(&id)).unwrap();
        let runs_path = store.runs_path(&id);
        let raw = br#"[ { "id": "prior", "startedAt": 1000, "status": "ok" } ]"#;
        let blocked = if read_failure {
            runs_path.clone()
        } else {
            fs::write(&runs_path, raw).unwrap();
            std::path::PathBuf::from(format!("{}.{}.tmp", runs_path.display(), std::process::id()))
        };
        fs::create_dir(&blocked).expect("block the actual read or atomic writer");
        fs::write(blocked.join("sentinel"), b"preserve owned directory").unwrap();
        let error = store.begin_run(&id, "manual", 3_000.0, None, Some("new-run"), None)
            .expect_err("filesystem refusal must not become an empty history");
        assert!(matches!(error, AutomationStoreError::Io(_)));
        assert!(blocked.is_dir());
        assert_eq!(fs::read(blocked.join("sentinel")).unwrap(), b"preserve owned directory");
        if !read_failure {
            assert_eq!(fs::read(&runs_path).unwrap(), raw);
        }
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        let _ = fs::remove_dir_all(root);
    }
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
fn finish_does_not_acknowledge_a_truly_absent_run_in_over_window_history() {
    for definition_only in [false, true] {
        let (root, store, id) = history_store("finish-window-miss");
        let mut rows = vec![json!({"id":"other","startedAt":0,"status":"running"})];
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
fn finish_settles_present_old_running_history_and_returns_the_exact_receipt() {
    for definition_only in [false, true] {
        let (root, store, id) = history_store("finish-retained-window");
        let mut rows = vec![json!({
            "id":"run","startedAt":0,"status":"running","coalescedRunIds":["alias"]
        })];
        rows.extend((0..AUTOMATION_MAX_RUN_HISTORY).map(|index| {
            json!({"id":format!("newer-{index}"),"startedAt":index + 1,"finishedAt":index + 2,"status":"ok"})
        }));
        fs::write(store.runs_path(&id), serde_json::to_vec(&rows).unwrap()).unwrap();
        let config = fs::read(store.config_path(&id)).unwrap();
        assert!(!store.read_runs(&id).iter().any(|run| run.id == "run"));
        let finished = finish_history(&store, &id, definition_only).unwrap().expect("exact old run exists");
        assert_eq!(finished.next_run_at.is_none(), definition_only);
        assert_eq!(finished.runs.len(), AUTOMATION_MAX_RUN_HISTORY);
        let receipt = finished.runs.iter().find(|run| run.id == "run").expect("pinned returned receipt");
        assert_eq!(receipt.started_at, 0.0);
        assert_eq!(receipt.finished_at, Some(2_000.0));
        assert_eq!(receipt.status, "error");
        assert_eq!(receipt.detail.as_deref(), Some("failed"));
        assert_eq!(receipt.coalesced_run_ids, Some(vec!["alias".into()]));
        assert_eq!(store.read_run_for_delivery(&id, "alias").unwrap().as_ref(), Some(receipt));
        let persisted = persisted_history(&store, &id);
        assert_eq!(persisted.len(), AUTOMATION_MAX_RUN_HISTORY);
        assert_eq!(persisted.iter().find(|run| run.id == "run"), Some(receipt));
        assert!(!persisted.iter().any(|run| run.id == "newer-0"));
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

fn persisted_history(store: &FileAutomationStore, id: &str) -> Vec<AutomationRun> {
    let rows: Vec<serde_json::Value> = serde_json::from_slice(
        &fs::read(store.runs_path(id)).expect("persisted history bytes"),
    ).expect("persisted history array");
    rows.iter().map(|row| parse_stored_run(row).expect("complete persisted row")).collect()
}

#[test]
fn begin_keeps_running_identity_and_aliases_across_settled_history_eviction() {
    let (root, store, id) = history_store("retained-across-window");
    let original = store.begin_run(&id, "event", 0.0, Some("original event"), Some("retained"),
        Some(&["alias-a".into(), "alias-b".into()])).unwrap().unwrap();
    let config = fs::read(store.config_path(&id)).unwrap();
    for index in 0..AUTOMATION_MAX_RUN_HISTORY + 3 {
        let run_id = format!("new-{index}");
        store.begin_run(&id, "manual", 100.0 + index as f64, None, Some(&run_id), None).unwrap().unwrap();
        store.finish_run(&id, &run_id, if index % 2 == 0 { "ok" } else { "error" },
            200.0 + index as f64, None).unwrap().unwrap();
        let all = persisted_history(&store, &id);
        assert!(all.len() <= AUTOMATION_MAX_RUN_HISTORY);
        assert_eq!(all.iter().find(|run| run.id == original.id), Some(&original));
        assert!(all.windows(2).all(|pair| pair[0].started_at >= pair[1].started_at));
        for query in ["retained", "alias-a", "alias-b"] {
            assert_eq!(store.read_run_for_delivery(&id, query).unwrap(), Some(original.clone()));
        }
    }
    let all = persisted_history(&store, &id);
    assert_eq!(all.len(), AUTOMATION_MAX_RUN_HISTORY);
    assert!(!all.iter().any(|run| run.id == "new-0"));
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn full_running_capacity_refuses_growth_but_keeps_existing_lookup_and_settlement() {
    let (root, store, id) = history_store("running-capacity");
    for index in 0..AUTOMATION_MAX_RUN_HISTORY {
        store.begin_run(&id, "schedule", index as f64, None,
            Some(&format!("running-{index}")), None).unwrap().unwrap();
    }
    let all = persisted_history(&store, &id);
    let before = fs::read(store.runs_path(&id)).unwrap();
    let config = fs::read(store.config_path(&id)).unwrap();
    for run in &all {
        assert_eq!(store.begin_run(&id, "event", 9_000.0, Some("different"),
            Some(&run.id), Some(&["different-alias".into()])).unwrap().as_ref(), Some(run));
        assert_eq!(store.read_run_for_delivery(&id, &run.id).unwrap().as_ref(), Some(run));
    }
    let error = store.begin_run(&id, "manual", 9_000.0, None, Some("over-capacity"), None)
        .expect_err("twenty Running authorities cannot be evicted");
    assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == io::ErrorKind::WouldBlock));
    assert_eq!(fs::read(store.runs_path(&id)).unwrap(), before);
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);

    let settled = store.finish_run(&id, "running-0", "ok", 10_000.0, None).unwrap().unwrap();
    let receipt = settled.runs.iter().find(|run| run.id == "running-0").unwrap();
    assert_eq!(receipt.status, "ok");
    assert_eq!(store.read_run_for_delivery(&id, "running-0").unwrap().as_ref(), Some(receipt));
    let backdated = store.begin_run(&id, "manual", -1.0, None, Some("backdated"), None).unwrap().unwrap();
    let after = persisted_history(&store, &id);
    assert_eq!(after.len(), AUTOMATION_MAX_RUN_HISTORY);
    assert_eq!(after.iter().find(|run| run.id == "backdated"), Some(&backdated));
    assert_eq!(backdated.started_at, -1.0);
    for run in all.iter().filter(|run| run.id != "running-0") {
        assert_eq!(after.iter().find(|entry| entry.id == run.id), Some(run));
    }
    assert!(!after.iter().any(|run| run.id == "running-0"), "only the settled row is now evictable");
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn legacy_running_excess_drains_without_growth_or_loss_and_returns_bounded_receipts() {
    for definition_only in [false, true] {
        let (root, store, id) = history_store("legacy-running-excess");
        let running_count = AUTOMATION_MAX_RUN_HISTORY + 3;
        let mut rows = (0..running_count).map(|index| {
            json!({"id":format!("old-{index}"),"startedAt":index,"status":"running"})
        }).collect::<Vec<_>>();
        rows.push(json!({"id":"terminal","startedAt":1000,"status":"ok"}));
        fs::write(store.runs_path(&id), serde_json::to_vec(&rows).unwrap()).unwrap();
        let original = persisted_history(&store, &id);
        let before = fs::read(store.runs_path(&id)).unwrap();
        let config = fs::read(store.config_path(&id)).unwrap();
        assert!(!store.read_runs(&id).iter().any(|run| run.id == "old-0"));
        assert_eq!(store.begin_run(&id, "manual", 9_000.0, None, Some("old-0"), None)
            .unwrap().as_ref(), original.iter().find(|run| run.id == "old-0"));
        let error = store.begin_run(&id, "manual", 9_000.0, None, Some("new"), None).unwrap_err();
        assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == io::ErrorKind::WouldBlock));
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), before);

        let mut prior_len = original.len();
        for index in 0..4 {
            let run_id = format!("old-{index}");
            let result = if definition_only {
                store.finish_run_definition(&id, &run_id, "error", 10_000.0 + index as f64, Some("known terminal"))
            } else {
                store.finish_run(&id, &run_id, "error", 10_000.0 + index as f64, Some("known terminal"))
            }.unwrap().unwrap();
            assert_eq!(result.next_run_at.is_none(), definition_only);
            assert!(result.runs.len() <= AUTOMATION_MAX_RUN_HISTORY);
            let receipt = result.runs.iter().find(|run| run.id == run_id).expect("old exact target in bounded receipt");
            assert_eq!(receipt.status, "error");
            assert_eq!(receipt.finished_at, Some(10_000.0 + index as f64));
            let persisted = persisted_history(&store, &id);
            assert!(persisted.len() <= prior_len, "legacy excess never grows");
            assert_eq!(persisted.len(), (running_count - index).max(AUTOMATION_MAX_RUN_HISTORY));
            assert_eq!(persisted.iter().find(|run| run.id == run_id), Some(receipt));
            for retained_index in index + 1..running_count {
                let retained = format!("old-{retained_index}");
                assert_eq!(persisted.iter().find(|run| run.id == retained),
                    original.iter().find(|run| run.id == retained));
            }
            assert!(!persisted.iter().any(|run| run.id == "terminal"));
            prior_len = persisted.len();
        }
        assert_eq!(prior_len, AUTOMATION_MAX_RUN_HISTORY);
        store.begin_run(&id, "manual", 20_000.0, None, Some("after-drain"), None).unwrap().unwrap();
        assert_eq!(persisted_history(&store, &id).len(), AUTOMATION_MAX_RUN_HISTORY);
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn delivery_lookup_reads_hidden_original_and_aliases_without_rewriting_history() {
    let (root, store, id) = history_store("delivery-hidden");
    let mut rows = (0..AUTOMATION_MAX_RUN_HISTORY).map(|index| {
        json!({"id":format!("newer-{index}"),"startedAt":index + 1,"finishedAt":index + 2,"status":"ok"})
    }).collect::<Vec<_>>();
    rows.push(json!({"id":"  ","startedAt":0,"status":"running","coalescedRunIds":["alias","alias"]}));
    let raw = serde_json::to_vec_pretty(&rows).unwrap();
    fs::write(store.runs_path(&id), &raw).unwrap();
    let config = fs::read(store.config_path(&id)).unwrap();
    assert_eq!(store.read_runs(&id).len(), AUTOMATION_MAX_RUN_HISTORY);
    assert!(!store.read_runs(&id).iter().any(|run| run.id == "  "));
    let original = store.read_run_for_delivery(&id, "  ").unwrap().expect("literal primary ID");
    assert_eq!(original.status, "running");
    assert_eq!(store.read_run_for_delivery(&id, "alias").unwrap(), Some(original));
    assert_eq!(store.read_run_for_delivery(&id, "missing").unwrap(), None);
    assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delivery_lookup_refuses_malformed_tails_and_cross_row_identity_ambiguity() {
    let mut cases = vec![
        b"[".to_vec(),
        br#"{"id":"run","startedAt":0}"#.to_vec(),
        b"[\xff]".to_vec(),
        br#"[{"id":"run","startedAt":0,"status":"running"},{"id":"bad","startedAt":"invalid","status":"running"}]"#.to_vec(),
        br#"[{"id":"run","startedAt":0,"status":"running"},{"id":"run","startedAt":1,"status":"running"}]"#.to_vec(),
        br#"[{"id":"a","startedAt":0,"status":"running","coalescedRunIds":["run"]},{"id":"b","startedAt":1,"status":"running","coalescedRunIds":["run"]}]"#.to_vec(),
        br#"[{"id":"run","startedAt":0,"status":"running"},{"id":"b","startedAt":1,"status":"running","coalescedRunIds":["run"]}]"#.to_vec(),
    ];
    let mut hidden_tail = (0..AUTOMATION_MAX_RUN_HISTORY + 2).map(|index| {
        json!({"id":format!("newer-{index}"),"startedAt":100 + index,"finishedAt":200 + index,"status":"ok"})
    }).collect::<Vec<_>>();
    hidden_tail.push(json!({"id":"run","startedAt":1,"status":"running"}));
    hidden_tail.push(json!({"id":"bad","startedAt":"invalid","status":"running"}));
    cases.push(serde_json::to_vec(&hidden_tail).unwrap());
    for (index, raw) in cases.iter().enumerate() {
        let (root, store, id) = history_store(&format!("delivery-invalid-{index}"));
        fs::write(store.runs_path(&id), raw).unwrap();
        let config = fs::read(store.config_path(&id)).unwrap();
        assert!(store.read_run_for_delivery(&id, "run").is_err(), "case {index}");
        assert!(store.read_run_for_delivery(&id, "absent").is_err(), "invalid whole input is not absence");
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), *raw);
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn delivery_lookup_distinguishes_missing_history_from_read_errors_and_unsafe_paths() {
    let (root, store, id) = history_store("delivery-read-boundary");
    let config = fs::read(store.config_path(&id)).unwrap();
    assert_eq!(store.read_run_for_delivery(&id, "run").unwrap(), None);
    assert!(!store.runs_path(&id).exists());
    for unsafe_id in ["", ".", "..", "../escape", "/absolute", "nested/folder", "nested\\folder"] {
        let error = store.read_run_for_delivery(unsafe_id, "run").unwrap_err();
        assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == io::ErrorKind::InvalidInput));
    }
    let error = store.read_run_for_delivery(&id, "").unwrap_err();
    assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == io::ErrorKind::InvalidInput));
    let error = store.read_run_for_delivery("missing-definition", "run").unwrap_err();
    assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == io::ErrorKind::NotFound));
    assert!(!store.get_location().join("missing-definition").exists());
    assert!(!store.get_location().join("nested").exists());
    assert!(!root.join("escape").exists());
    assert!(!store.runs_path(&id).exists());
    let absent_store = FileAutomationStore::new(root.join("not-created"));
    assert!(absent_store.read_run_for_delivery("missing", "run").is_err());
    assert!(!absent_store.get_location().exists());
    drop(absent_store);
    fs::create_dir(store.runs_path(&id)).unwrap();
    fs::write(store.runs_path(&id).join("sentinel"), b"owned history directory").unwrap();
    assert!(store.read_runs(&id).is_empty(), "display remains fail-soft");
    assert!(matches!(store.read_run_for_delivery(&id, "run"), Err(AutomationStoreError::Io(_))));
    assert_eq!(fs::read(store.runs_path(&id).join("sentinel")).unwrap(), b"owned history directory");
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    let _ = fs::remove_dir_all(root);
}

fn assert_delivery_history_refused(label: &str, raw: &[u8]) {
    let (root, store, id) = history_store(label);
    let config = fs::read(store.config_path(&id)).unwrap();
    fs::write(store.runs_path(&id), raw).unwrap();
    let reopened = FileAutomationStore::new(store.get_location());
    for reader in [&store, &reopened] {
        for requested in ["run", "alias", "bad", "other", "unrelated", "new-run"] {
            reader.read_run_for_delivery(&id, requested)
                .expect_err(&format!("{label}, requested={requested}"));
            assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw,
                "{label}: preserve the complete raw history");
            assert_eq!(fs::read(store.config_path(&id)).unwrap(), config,
                "{label}: preserve the definition");
        }
    }
    drop(reopened);
    drop(store);
    fs::remove_dir_all(root).expect("remove successful owned delivery fixture");
}

#[test]
fn delivery_lookup_rejects_raw_custody_fields_before_legacy_normalization() {
    for (label, raw) in [
        ("overflow-start", &br#"[{"id":"run","startedAt":1e400,"status":"running"}]"#[..]),
        ("overflow-finish", &br#"[{"id":"run","startedAt":1,"finishedAt":1e400,"status":"ok"}]"#[..]),
        ("nonobject-row", &br#"[{"id":"run","startedAt":1,"status":"running","coalescedRunIds":["alias"]},42]"#[..]),
    ] {
        assert_delivery_history_refused(label, raw);
    }
    let invalid_fields = [
        ("missing-id", "id", None),
        ("empty-id", "id", Some(json!(""))),
        ("null-id", "id", Some(json!(null))),
        ("numeric-id", "id", Some(json!(7))),
        ("missing-start", "startedAt", None),
        ("null-start", "startedAt", Some(json!(null))),
        ("text-start", "startedAt", Some(json!("1000"))),
        ("missing-status", "status", None),
        ("null-status", "status", Some(json!(null))),
        ("unknown-status", "status", Some(json!("future"))),
        ("nontext-status", "status", Some(json!(true))),
        ("running-finished", "finishedAt", Some(json!(1001))),
        ("running-text-finish", "finishedAt", Some(json!("legacy"))),
        ("running-false-finish", "finishedAt", Some(json!(false))),
        ("text-aliases", "coalescedRunIds", Some(json!("alias"))),
        ("object-aliases", "coalescedRunIds", Some(json!({}))),
        ("boolean-aliases", "coalescedRunIds", Some(json!(true))),
        ("empty-alias", "coalescedRunIds", Some(json!(["valid", ""]))),
        ("numeric-alias", "coalescedRunIds", Some(json!(["valid", 7]))),
        ("null-alias", "coalescedRunIds", Some(json!(["valid", null]))),
        ("too-many-aliases", "coalescedRunIds", Some(json!(vec!["same"; MAX_EVENTS_IN_AUTOMATION_WAKE + 1]))),
    ];
    for (label, field, replacement) in invalid_fields {
        let mut invalid = json!({"id":"bad","startedAt":1000,"status":"running"});
        if let Some(replacement) = replacement {
            invalid[field] = replacement;
        } else {
            invalid.as_object_mut().unwrap().remove(field);
        }
        // A valid matching first row cannot hide a later custody defect.
        let raw = serde_json::to_vec_pretty(&json!([
            {"id":"run","startedAt":2000,"status":"running","coalescedRunIds":["alias"]},
            invalid
        ])).unwrap();
        assert_delivery_history_refused(label, &raw);
    }
    for status in ["ok", "error"] {
        for (label, finished_at) in [
            ("missing", None),
            ("null", Some(json!(null))),
            ("text", Some(json!("1001"))),
            ("boolean", Some(json!(true))),
        ] {
            let mut row = json!({"id":"bad","startedAt":1000,"status":status});
            if let Some(finished_at) = finished_at {
                row["finishedAt"] = finished_at;
            }
            let raw = serde_json::to_vec_pretty(&json!([
                {"id":"run","startedAt":2000,"status":"running","coalescedRunIds":["alias"]},
                row
            ])).unwrap();
            assert_delivery_history_refused(&format!("terminal-{status}-{label}"), &raw);
        }
    }
}

#[test]
fn delivery_lookup_rejects_global_identity_collisions_in_both_orders() {
    for (label, first, second) in [
        ("duplicate-original",
            json!({"id":"run","startedAt":1,"status":"running"}),
            json!({"id":"run","startedAt":2,"finishedAt":3,"status":"ok"})),
        ("original-claimed-as-alias",
            json!({"id":"run","startedAt":1,"status":"running"}),
            json!({"id":"other","startedAt":2,"status":"running","coalescedRunIds":["run"]})),
        ("shared-alias",
            json!({"id":"run","startedAt":1,"status":"running","coalescedRunIds":["alias"]}),
            json!({"id":"other","startedAt":2,"status":"running","coalescedRunIds":["alias"]})),
    ] {
        for reverse in [false, true] {
            let mut rows = vec![first.clone(), second.clone()];
            if reverse {
                rows.reverse();
            }
            rows.insert(0, json!({"id":"unrelated","startedAt":4,"finishedAt":5,"status":"error"}));
            let raw = serde_json::to_vec_pretty(&rows).unwrap();
            assert_delivery_history_refused(&format!("{label}-reverse-{reverse}"), &raw);
        }
    }
}

#[test]
fn delivery_lookup_validates_custody_tails_beyond_the_display_window() {
    for (label, tail) in [
        ("missing-status", json!({"id":"tail","startedAt":0,"finishedAt":1})),
        ("unknown-status", json!({"id":"tail","startedAt":0,"finishedAt":1,"status":"future"})),
        ("terminal-missing-finish", json!({"id":"tail","startedAt":0,"status":"error"})),
        ("running-finished", json!({"id":"tail","startedAt":0,"finishedAt":1,"status":"running"})),
        ("bad-alias", json!({"id":"tail","startedAt":0,"status":"running","coalescedRunIds":[7]})),
        ("alias-overflow", json!({"id":"tail","startedAt":0,"status":"running","coalescedRunIds":vec!["same"; MAX_EVENTS_IN_AUTOMATION_WAKE + 1]})),
        ("duplicate", json!({"id":"run","startedAt":0,"status":"running"})),
        ("primary-as-alias", json!({"id":"tail","startedAt":0,"status":"running","coalescedRunIds":["run"]})),
        ("cross-row-alias", json!({"id":"tail","startedAt":0,"status":"running","coalescedRunIds":["alias"]})),
    ] {
        let (root, store, id) = history_store(&format!("delivery-tail-{label}"));
        let mut rows = vec![json!({
            "id":"run","startedAt":10000,"finishedAt":10001,"status":"ok","coalescedRunIds":["alias"]
        })];
        rows.extend((0..AUTOMATION_MAX_RUN_HISTORY + 2).map(|index| {
            json!({"id":format!("visible-{index}"),"startedAt":100 + index,"finishedAt":200 + index,"status":"ok"})
        }));
        rows.push(tail);
        let raw = serde_json::to_vec_pretty(&rows).unwrap();
        fs::write(store.runs_path(&id), &raw).unwrap();
        let config = fs::read(store.config_path(&id)).unwrap();
        let display = store.read_runs(&id);
        assert_eq!(display.len(), AUTOMATION_MAX_RUN_HISTORY);
        assert_eq!(display[0].id, "run");
        assert!(display.iter().skip(1).all(|run| run.id.starts_with("visible-")));
        let reopened = FileAutomationStore::new(store.get_location());
        for reader in [&store, &reopened] {
            for requested in ["run", "alias", "visible-0", "new-run"] {
                reader.read_run_for_delivery(&id, requested)
                    .expect_err(&format!("tail={label}, requested={requested}"));
                assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
                assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
            }
        }
        drop(reopened);
        drop(store);
        fs::remove_dir_all(root).expect("remove successful owned tail fixture");
    }
}

#[test]
fn delivery_lookup_preserves_checked_running_terminal_and_same_row_aliases() {
    let (root, store, id) = history_store("delivery-checked-rows");
    let config = fs::read(store.config_path(&id)).unwrap();
    let raw = br#"[
      { "id": "running", "startedAt": 10, "status": "running", "trigger": "legacy", "event": " event ", "extra": true },
      { "id": "ok", "startedAt": 3, "finishedAt": 4, "status": "ok", "coalescedRunIds": ["ok-alias", "ok-alias", "ok"] },
      { "id": "error", "startedAt": 1, "finishedAt": 2, "status": "error", "detail": " stored failure ", "coalescedRunIds": ["error-alias"] },
      { "id": "running-null", "startedAt": 5, "finishedAt": null, "status": "running", "coalescedRunIds": null },
      { "id": "empty-aliases", "startedAt": 6, "finishedAt": 7, "status": "ok", "coalescedRunIds": [] }
    ]
"#;
    fs::write(store.runs_path(&id), raw).unwrap();
    let reopened = FileAutomationStore::new(store.get_location());
    for reader in [&store, &reopened] {
        for (requested, original, status, finished_at) in [
            ("running", "running", "running", None),
            ("running-null", "running-null", "running", None),
            ("ok", "ok", "ok", Some(4.0)),
            ("ok-alias", "ok", "ok", Some(4.0)),
            ("error", "error", "error", Some(2.0)),
            ("error-alias", "error", "error", Some(2.0)),
            ("empty-aliases", "empty-aliases", "ok", Some(7.0)),
        ] {
            let run = reader.read_run_for_delivery(&id, requested).unwrap().expect("checked persisted row");
            assert_eq!(run.id, original, "{requested}");
            assert_eq!(run.status, status, "{requested}");
            assert_eq!(run.finished_at, finished_at, "{requested}");
            if original == "running" {
                assert_eq!(run.trigger, "schedule", "non-custody metadata keeps its legacy parser");
                assert_eq!(run.event.as_deref(), Some("event"));
            }
            if original == "error" {
                assert_eq!(run.detail.as_deref(), Some("stored failure"));
            }
            if original == "ok" {
                assert_eq!(run.coalesced_run_ids, Some(vec!["ok-alias".into(), "ok-alias".into(), "ok".into()]));
            }
            assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
            assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        }
    }
    drop(reopened);
    drop(store);
    fs::remove_dir_all(root).expect("remove successful owned custody fixture");
}

#[test]
fn delivery_lookup_preserves_literal_identities_and_the_exact_alias_bound() {
    let (root, store, id) = history_store("delivery-literal-identities");
    let config = fs::read(store.config_path(&id)).unwrap();
    let mut aliases = vec!["  ".to_owned(), " alias ".to_owned(), " alias ".to_owned(), "\t".to_owned()];
    aliases.extend((aliases.len()..MAX_EVENTS_IN_AUTOMATION_WAKE).map(|index| format!("alias-{index}")));
    assert_eq!(aliases.len(), MAX_EVENTS_IN_AUTOMATION_WAKE);
    let raw = serde_json::to_vec_pretty(&json!([
        {"id":"  ","startedAt":1,"status":"running","coalescedRunIds":aliases},
        {"id":"alias","startedAt":2,"finishedAt":3,"status":"ok"}
    ])).unwrap();
    fs::write(store.runs_path(&id), &raw).unwrap();
    for requested in &aliases {
        let run = store.read_run_for_delivery(&id, requested).unwrap().expect("literal original or alias");
        assert_eq!(run.id, "  ");
        assert_eq!(run.coalesced_run_ids.as_ref(), Some(&aliases));
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    }
    assert_eq!(store.read_run_for_delivery(&id, " ").unwrap(), None);
    let separate = store.read_run_for_delivery(&id, "alias").unwrap().expect("distinct literal identity");
    assert_eq!(separate.id, "alias");
    assert_eq!(separate.status, "ok");
    assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    drop(store);
    fs::remove_dir_all(root).expect("remove successful owned literal fixture");
}

#[test]
fn delivery_lookup_preserves_empty_and_nonmatching_history_without_writing() {
    let (root, store, id) = history_store("delivery-checked-absence");
    let config = fs::read(store.config_path(&id)).unwrap();
    for raw in [
        &b"[ ]\n"[..],
        &br#"[ { "id": "other", "startedAt": 1000, "status": "running" } ]"#[..],
    ] {
        fs::write(store.runs_path(&id), raw).unwrap();
        let reopened = FileAutomationStore::new(store.get_location());
        for reader in [&store, &reopened] {
            assert_eq!(reader.read_run_for_delivery(&id, "run").unwrap(), None);
            let error = reader.read_run_for_delivery(&id, "").unwrap_err();
            assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == io::ErrorKind::InvalidInput));
            assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
            assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        }
    }
    drop(store);
    fs::remove_dir_all(root).expect("remove successful owned absence fixture");
}

#[test]
fn delivery_lookup_requires_readable_valid_configuration_for_match_and_absence() {
    for has_history in [false, true] {
        for (label, replacement) in [
            ("missing", None),
            ("malformed-json", Some(&b"["[..])),
            ("invalid-fields", Some(&b"{}"[..])),
            ("invalid-utf8", Some(&b"\xff"[..])),
            ("duplicate-name", Some(&br#"{"name":"first","name":"second","prompt":"go","schedule":"@hourly"}"#[..])),
            ("duplicate-nested", Some(&br#"{"name":"valid","prompt":"go","schedule":"@hourly","extra":{"key":1,"key":2}}"#[..])),
        ] {
            let (root, store, id) = history_store(&format!("delivery-config-{label}-{has_history}"));
            let raw = br#"[{"id":"run","startedAt":1,"finishedAt":2,"status":"ok","coalescedRunIds":["alias"]}]"#;
            if has_history {
                fs::write(store.runs_path(&id), raw).unwrap();
            }
            let config_path = store.config_path(&id);
            if let Some(replacement) = replacement {
                fs::write(&config_path, replacement).unwrap();
            } else {
                fs::remove_file(&config_path).unwrap();
            }
            for requested in ["run", "alias", "new-run"] {
                let error = store.read_run_for_delivery(&id, requested).expect_err(label);
                let expected = if replacement.is_none() { io::ErrorKind::NotFound } else { io::ErrorKind::InvalidData };
                assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == expected), "{label}");
                if has_history {
                    assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
                } else {
                    assert!(!store.runs_path(&id).exists());
                }
                if let Some(replacement) = replacement {
                    assert_eq!(fs::read(&config_path).unwrap(), replacement);
                } else {
                    assert!(!config_path.exists());
                }
            }
            drop(store);
            fs::remove_dir_all(root).expect("remove successful owned configuration fixture");
        }
    }
    let (root, store, id) = history_store("delivery-config-directory");
    let raw = br#"[{"id":"run","startedAt":1,"finishedAt":2,"status":"ok"}]"#;
    fs::write(store.runs_path(&id), raw).unwrap();
    let config_path = store.config_path(&id);
    fs::remove_file(&config_path).unwrap();
    fs::create_dir(&config_path).unwrap();
    fs::write(config_path.join("sentinel"), b"preserve unreadable configuration").unwrap();
    for requested in ["run", "new-run"] {
        assert!(matches!(store.read_run_for_delivery(&id, requested), Err(AutomationStoreError::Io(_))));
        assert!(config_path.is_dir());
        assert_eq!(fs::read(config_path.join("sentinel")).unwrap(), b"preserve unreadable configuration");
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), raw);
    }
    drop(store);
    fs::remove_dir_all(root).expect("remove successful owned unreadable configuration fixture");
}

#[test]
fn backend_delivery_discovery_uses_exact_folder_identity_and_ignores_unrelated_definitions() {
    let (root, store, id) = history_store("delivery-discovery-target");
    let agent_id = "agent-1";
    let config = fs::read(store.config_path(&id)).unwrap();
    let history = br#"[{"id":"run","startedAt":1,"status":"running"}]"#;
    fs::write(store.runs_path(&id), history).unwrap();
    let literal_id = " other folder ";
    fs::create_dir(store.get_location().join(literal_id)).unwrap();
    fs::write(store.config_path(literal_id), &config).unwrap();
    let invalid_id = "unrelated-invalid-definition";
    fs::create_dir(store.get_location().join(invalid_id)).unwrap();
    fs::write(store.config_path(invalid_id), b"\xff unreadable definition").unwrap();
    let unreadable_id = "unrelated-definition-directory";
    fs::create_dir_all(store.config_path(unreadable_id)).unwrap();
    fs::write(store.config_path(unreadable_id).join("sentinel"), b"unrelated definition").unwrap();
    let missing_id = "unrelated-missing-definition";
    fs::create_dir(store.get_location().join(missing_id)).unwrap();

    let reopened = FileAutomationStore::new(store.get_location());
    for reader in [&store, &reopened] {
        for local_id in [&id[..], literal_id] {
            let found = reader.find_for_backend_delivery(
                agent_id, &stable_automation_id(agent_id, local_id),
            ).unwrap().expect("exact valid matching definition");
            assert_eq!(found.id, local_id);
            assert_eq!(found.name, "History acknowledgment");
            assert_eq!(found.file_path, store.config_path(local_id));
            assert!(found.next_run_at.is_some(), "retain the existing record projection");
            assert!(found.runs.is_empty(), "target discovery carries no display history");
            assert_eq!(reader.get(local_id).unwrap().runs.len(), if local_id == id { 1 } else { 0 },
                "ordinary records retain their display projection");
        }
        for backend_id in [
            stable_automation_id("another-agent", &id),
            stable_automation_id(agent_id, "missing"),
            stable_automation_id(agent_id, literal_id.trim()),
        ] {
            assert_eq!(reader.find_for_backend_delivery(agent_id, &backend_id).unwrap(), None);
        }
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        assert_eq!(fs::read(store.config_path(literal_id)).unwrap(), config);
        assert_eq!(fs::read(store.runs_path(&id)).unwrap(), history);
        assert_eq!(fs::read(store.config_path(invalid_id)).unwrap(), b"\xff unreadable definition");
        assert_eq!(fs::read(store.config_path(unreadable_id).join("sentinel")).unwrap(), b"unrelated definition");
        assert!(!store.config_path(missing_id).exists());
        assert!(!store.runs_path(literal_id).exists());
    }
    drop(reopened);
    drop(store);
    fs::remove_dir_all(root).expect("remove successful targeted discovery fixture");
}

#[test]
fn backend_delivery_discovery_refuses_invalid_matching_definitions_without_writes() {
    for (label, replacement) in [
        ("missing", None),
        ("invalid-json", Some(&b"["[..])),
        ("invalid-fields", Some(&b"{}"[..])),
        ("invalid-utf8", Some(&b"\xff"[..])),
        ("duplicate-enabled", Some(&br#"{"name":"valid","prompt":"go","schedule":"@hourly","enabled":false,"enabled":true}"#[..])),
        ("duplicate-trigger", Some(&br#"{"name":"valid","prompt":"go","trigger":{"type":"cron","schedule":"@daily","schedule":"@hourly"}}"#[..])),
        ("duplicate-extension", Some(&br#"{"name":"valid","prompt":"go","schedule":"@hourly","extension":[{"id":1,"id":2}]}"#[..])),
    ] {
        let (root, store, id) = history_store(&format!("discovery-config-{label}"));
        let history = br#"[{"id":"run","startedAt":1,"status":"running","coalescedRunIds":["alias"]}]"#;
        fs::write(store.runs_path(&id), history).unwrap();
        let path = store.config_path(&id);
        match replacement {
            Some(raw) => fs::write(&path, raw).unwrap(),
            None => fs::remove_file(&path).unwrap(),
        }
        let reopened = FileAutomationStore::new(store.get_location());
        for reader in [&store, &reopened] {
            let error = reader.find_for_backend_delivery("agent-1", &stable_automation_id("agent-1", &id))
                .expect_err(label);
            let expected = if replacement.is_none() { io::ErrorKind::NotFound } else { io::ErrorKind::InvalidData };
            assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == expected), "{label}");
            assert_eq!(reader.find_for_backend_delivery("agent-1", &stable_automation_id("agent-1", "missing"))
                .unwrap(), None, "unrelated config content is not parsed: {label}");
            for run_id in ["run", "alias", "new-run"] {
                assert!(reader.read_run_for_delivery(&id, run_id).is_err(), "shared definition boundary: {label}");
            }
            assert_eq!(fs::read(store.runs_path(&id)).unwrap(), history, "{label}");
            if let Some(raw) = replacement {
                assert_eq!(fs::read(&path).unwrap(), raw, "{label}");
            } else {
                assert!(!path.exists());
            }
        }
        drop(reopened);
        drop(store);
        fs::remove_dir_all(root).expect("remove successful invalid matching definition fixture");
    }

    let (root, store, id) = history_store("discovery-definition-directory");
    let path = store.config_path(&id);
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    fs::write(path.join("sentinel"), b"matching unreadable definition").unwrap();
    assert!(store.find_for_backend_delivery("agent-1", &stable_automation_id("agent-1", &id)).is_err());
    assert_eq!(fs::read(path.join("sentinel")).unwrap(), b"matching unreadable definition");
    assert!(!store.runs_path(&id).exists());
    drop(store);
    fs::remove_dir_all(root).expect("remove successful matching definition directory fixture");
}

#[test]
fn backend_delivery_discovery_distinguishes_confirmed_absence_from_directory_failures() {
    let root = root("delivery-discovery-absence");
    fs::create_dir(&root).unwrap();
    let store = FileAutomationStore::new(root.join("not-created").join("automations"));
    let backend_id = stable_automation_id("agent-1", "local-1");
    assert_eq!(store.find_for_backend_delivery("agent-1", &backend_id).unwrap(), None);
    assert!(!root.join("not-created").exists(), "checked absence cannot materialize a directory");
    for agent_id in ["", ".", "..", "../escape", "nested/agent", "nested\\agent", "bad\0agent"] {
        let error = store.find_for_backend_delivery(agent_id, &backend_id).unwrap_err();
        assert!(matches!(error, AutomationStoreError::Io(ref error) if error.kind() == io::ErrorKind::InvalidInput));
        assert!(!root.join("not-created").exists());
    }
    fs::write(root.join("not-created"), b"not a directory").unwrap();
    assert!(store.find_for_backend_delivery("agent-1", &backend_id).is_err(), "invalid ancestor is not absence");
    assert_eq!(fs::read(root.join("not-created")).unwrap(), b"not a directory");
    fs::remove_file(root.join("not-created")).unwrap();
    fs::create_dir(root.join("not-created")).unwrap();
    fs::write(store.get_location(), b"unreadable enumeration root").unwrap();
    assert!(store.find_for_backend_delivery("agent-1", &backend_id).is_err());
    assert_eq!(fs::read(store.get_location()).unwrap(), b"unreadable enumeration root");
    fs::remove_file(store.get_location()).unwrap();
    fs::create_dir(store.get_location()).unwrap();
    assert_eq!(store.find_for_backend_delivery("agent-1", &backend_id).unwrap(), None);
    let matched_file = store.get_location().join("local-1");
    fs::write(&matched_file, b"matching identity is not a folder").unwrap();
    assert!(store.find_for_backend_delivery("agent-1", &backend_id).is_err());
    assert_eq!(store.find_for_backend_delivery("agent-1", &stable_automation_id("agent-1", "missing"))
        .unwrap(), None);
    assert_eq!(fs::read(&matched_file).unwrap(), b"matching identity is not a folder");
    drop(store);
    fs::remove_dir_all(root).expect("remove successful checked directory fixture");
}

#[test]
fn backend_delivery_discovery_retains_valid_legacy_definition_semantics() {
    let (root, store, id) = history_store("delivery-discovery-legacy-definition");
    let raw = br#"{"name":"  Legacy   routine ","prompt":" go ","schedule":"@hourly","enabled":null,"createdAt":"legacy","lastRunAt":"legacy","raisedNotices":[7,"unknown"],"extra":{"unique":true}}"#;
    fs::write(store.config_path(&id), raw).unwrap();
    let checked = store.find_for_backend_delivery("agent-1", &stable_automation_id("agent-1", &id))
        .unwrap().expect("valid legacy matching definition");
    let displayed = store.get(&id).expect("existing legacy parser");
    assert_eq!(checked, displayed);
    assert_eq!(checked.name, "Legacy routine");
    assert_eq!(checked.prompt, "go");
    assert!(checked.is_enabled);
    assert!(checked.last_run_at.is_none());
    assert!(checked.raised_notices.is_empty());
    assert_eq!(store.read_run_for_delivery(&id, "new-run").unwrap(), None);
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), raw);
    assert!(!store.runs_path(&id).exists());
    drop(store);
    fs::remove_dir_all(root).expect("remove successful legacy definition fixture");
}

#[cfg(unix)]
#[test]
fn backend_delivery_discovery_checks_root_and_child_link_metadata_without_admitting_child_links() {
    use std::os::unix::fs::symlink;
    let (root, store, id) = history_store("delivery-discovery-links");
    let config = fs::read(store.config_path(&id)).unwrap();
    let history = br#"[{"id":"run","startedAt":1,"status":"running"}]"#;
    fs::write(store.runs_path(&id), history).unwrap();
    let backend_id = stable_automation_id("agent-1", &id);
    let linked_root = root.join("linked-root");
    symlink(store.get_location(), &linked_root).unwrap();
    let linked = FileAutomationStore::new(&linked_root);
    assert_eq!(linked.find_for_backend_delivery("agent-1", &backend_id).unwrap().unwrap().id, id);
    assert_eq!(linked.read_run_for_delivery(&id, "run").unwrap().unwrap().status, "running");

    let missing_target = root.join("missing-target");
    let broken_root = root.join("broken-root");
    symlink(&missing_target, &broken_root).unwrap();
    for path in [&broken_root, &broken_root.join("automations")] {
        let broken = FileAutomationStore::new(path);
        assert!(broken.find_for_backend_delivery("agent-1", &backend_id).is_err());
        assert_eq!(fs::read_link(&broken_root).unwrap(), missing_target);
        assert!(!missing_target.exists());
    }

    let child_id = "linked-child";
    let child_link = store.get_location().join(child_id);
    symlink(store.get_location().join(&id), &child_link).unwrap();
    assert!(store.find_for_backend_delivery("agent-1", &stable_automation_id("agent-1", child_id)).is_err());
    assert!(store.read_run_for_delivery(child_id, "run").is_err());
    assert!(store.find_for_backend_delivery("agent-1", &backend_id).unwrap().is_some(),
        "an unrelated valid link does not require parsing its definition");
    fs::remove_file(&child_link).unwrap();
    symlink(&missing_target, &child_link).unwrap();
    for requested in [&backend_id, &stable_automation_id("agent-1", "missing")] {
        assert!(store.find_for_backend_delivery("agent-1", requested).is_err(),
            "unavailable metadata invalidates both a different match and absence");
    }
    assert_eq!(fs::read_link(&child_link).unwrap(), missing_target);
    assert!(!missing_target.exists());
    assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    assert_eq!(fs::read(store.runs_path(&id)).unwrap(), history);
    drop(linked);
    drop(store);
    fs::remove_dir_all(root).expect("remove successful checked link fixture");
}

// Invalid UTF-8 filenames require Linux; macOS APFS does not provide this
// fixture prerequisite. Content-byte UTF-8 negatives above remain portable.
#[cfg(target_os = "linux")]
#[test]
fn backend_delivery_discovery_refuses_non_utf8_folder_identity_for_match_and_absence() {
    use std::os::unix::ffi::OsStringExt;
    let (root, store, id) = history_store("delivery-discovery-byte-identity");
    let config = fs::read(store.config_path(&id)).unwrap();
    let malformed = store.get_location().join(std::ffi::OsString::from_vec(vec![b'i', 0xff, b'd']));
    fs::create_dir(&malformed).unwrap();
    fs::write(malformed.join("sentinel"), b"preserve unknown folder identity").unwrap();
    for local_id in [&id[..], "missing"] {
        assert!(store.find_for_backend_delivery("agent-1", &stable_automation_id("agent-1", local_id)).is_err());
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
        assert_eq!(fs::read(malformed.join("sentinel")).unwrap(), b"preserve unknown folder identity");
        assert!(!store.runs_path(&id).exists());
    }
    drop(store);
    fs::remove_dir_all(root).expect("remove successful byte-identity fixture");
}

#[cfg(unix)]
#[test]
fn delivery_lookup_refuses_dangling_history_and_definition_links_without_creating_targets() {
    use std::os::unix::fs::symlink;
    let (root, store, id) = history_store("delivery-file-links");
    let config = fs::read(store.config_path(&id)).unwrap();
    let missing_history = root.join("missing-history");
    symlink(&missing_history, store.runs_path(&id)).unwrap();
    for requested in ["run", "alias", "new-run"] {
        assert!(store.read_run_for_delivery(&id, requested).is_err());
        assert_eq!(fs::read_link(store.runs_path(&id)).unwrap(), missing_history);
        assert!(!missing_history.exists());
        assert_eq!(fs::read(store.config_path(&id)).unwrap(), config);
    }
    assert!(store.read_runs(&id).is_empty(), "legacy display remains fail-soft");
    fs::remove_file(store.runs_path(&id)).unwrap();
    fs::remove_file(store.config_path(&id)).unwrap();
    let missing_config = root.join("missing-config");
    symlink(&missing_config, store.config_path(&id)).unwrap();
    for requested in ["run", "new-run"] {
        assert!(store.read_run_for_delivery(&id, requested).is_err());
        assert!(store.find_for_backend_delivery("agent-1", &stable_automation_id("agent-1", &id)).is_err());
        assert_eq!(fs::read_link(store.config_path(&id)).unwrap(), missing_config);
        assert!(!missing_config.exists());
        assert!(!store.runs_path(&id).exists());
    }
    drop(store);
    fs::remove_dir_all(root).expect("remove successful unavailable file-link fixture");
}
