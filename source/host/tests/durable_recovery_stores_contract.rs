use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::transcript::production_runtime::ProductionTranscriptRuntime;
use mahayana_host_runtime::extensions::transcript::sand_pending_wake_store::{
    AutomationQuietOrigin, DurablePendingWakeMarker, PendingWakeKind, QuietWakeOrigin,
    PendingWakeStoreError, SandPendingWakeStore, parse_pending_wake_file,
};
use mahayana_host_runtime::extensions::transcript::sand_upgrade_resume_store::{
    upgrade_resume_marker_identity_is_well_formed, SandUpgradeResumeStore, UpgradeResumeMarker,
    parse_upgrade_resume_file, UpgradeResumeStoreError,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-durable-recovery-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn pending(agent_id: &str, kind: PendingWakeKind, work_id: &str, at: f64) -> DurablePendingWakeMarker {
    DurablePendingWakeMarker {
        agent_id: agent_id.into(),
        kind,
        work_id: work_id.into(),
        marked_at_ms: at,
        quiet_origin: None,
        title: None,
        subagent_type: None,
        interrupted_by_recreate: false,
        delivery_operation_id: None,
        address_token: None,
        failure_reason: None,
        session_key: None,
    }
}

#[test]
fn pending_wake_parser_preserves_frozen_coercion_and_filters_invalid_rows() {
    let rows = parse_pending_wake_file(Some(r#"{"pending":[
        {"agentId":"a","kind":"cloud-agent","workId":"w","markedAtMs":12,
         "quietOrigin":{"automation":{"id":"auto","name":"Daily"}},"title":"Cloud"},
        {"agentId":"b","kind":"shell","workId":"s","markedAtMs":"bad",
         "quietOrigin":{"automation":{"id":"","name":"ignored"}},"interruptedByRecreate":true},
        {"agentId":"","kind":"shell","workId":"bad"},
        {"agentId":"c","kind":"unknown","workId":"bad"}
    ]}"#));
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].marked_at_ms, 12.0);
    assert_eq!(
        rows[0].quiet_origin,
        Some(QuietWakeOrigin {
            automation: Some(AutomationQuietOrigin {
                id: "auto".into(),
                name: "Daily".into(),
            }),
        })
    );
    assert_eq!(rows[1].marked_at_ms, 0.0);
    assert_eq!(
        rows[1].quiet_origin,
        Some(QuietWakeOrigin { automation: None })
    );
    assert!(rows[1].interrupted_by_recreate);
    assert!(parse_pending_wake_file(Some("not-json")).is_empty());
}

#[test]
fn channel_delivery_failure_marker_round_trips_operation_identity() {
    let root = temp_root("channel-delivery");
    fs::create_dir_all(&root).expect("root");
    let store = SandPendingWakeStore::new(&root);
    let mut marker = pending("agent-channel", PendingWakeKind::ChannelDelivery, "op-1", 10.0);
    marker.delivery_operation_id = Some("op-1".into());
    marker.address_token = Some("telegram:123".into());
    marker.failure_reason = Some("provider unavailable".into());
    assert!(store.mark_pending(marker));
    let restored = store.list_pending().pop().expect("channel marker");
    assert_eq!(restored.kind, PendingWakeKind::ChannelDelivery);
    assert_eq!(restored.delivery_operation_id.as_deref(), Some("op-1"));
    assert_eq!(restored.address_token.as_deref(), Some("telegram:123"));
    assert_eq!(restored.failure_reason.as_deref(), Some("provider unavailable"));
}

#[test]
fn pending_wake_store_upserts_clears_and_prunes_atomically() {
    let root = temp_root("pending");
    let store = SandPendingWakeStore::new(&root);
    assert!(store.mark_pending(pending("a", PendingWakeKind::Shell, "w", 10.0)));
    assert!(store.mark_pending(DurablePendingWakeMarker {
        title: Some("updated".into()),
        ..pending("a", PendingWakeKind::Shell, "w", 20.0)
    }));
    assert_eq!(store.list_pending().len(), 1);
    assert_eq!(store.list_pending()[0].title.as_deref(), Some("updated"));
    assert!(store.has_pending("a", PendingWakeKind::Shell, "w"));
    assert!(!store.file_path().with_extension("json.part").exists());

    assert!(store.mark_pending(pending("b", PendingWakeKind::Subagent, "x", 1.0)));
    let pruned = store.prune_stale(5.0, 20.0);
    assert_eq!(pruned.len(), 1);
    assert_eq!(pruned[0].agent_id, "b");
    assert!(store.clear_one("a", PendingWakeKind::Shell, "w"));
    assert!(store.list_pending().is_empty());
    assert!(!store.file_path().exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn checked_pending_store_accepts_missing_versionless_and_v1_without_normalizing_markers() {
    let root = temp_root("checked-pending-valid");
    let store = SandPendingWakeStore::new(&root);
    let retained = DurablePendingWakeMarker {
        quiet_origin: Some(QuietWakeOrigin {
            automation: Some(AutomationQuietOrigin {
                id: " original-automation ".into(),
                name: String::new(),
            }),
        }),
        title: Some(String::new()),
        subagent_type: Some(" legacy-kind ".into()),
        interrupted_by_recreate: true,
        delivery_operation_id: Some(" distinct-operation ".into()),
        address_token: Some(" slack:retained-room ".into()),
        failure_reason: Some(" original failure ".into()),
        session_key: Some(" agent:channel-agent:main ".into()),
        ..pending(" channel-agent ", PendingWakeKind::ChannelDelivery, " recovery-work ", 10.0)
    };
    assert!(store.list_pending_checked().unwrap().is_empty());
    assert_eq!(store.with_matching_marker_checked(&retained, || 7).unwrap(), None);
    assert!(!root.exists(), "absence inspection must not create the store");

    fs::create_dir_all(&root).unwrap();
    for document in [
        serde_json::json!({"pending": []}),
        serde_json::json!({"version": 1, "pending": []}),
        serde_json::json!({"pending": [retained.clone()]}),
        serde_json::json!({"version": 1, "pending": [retained.clone()]}),
    ] {
        let raw = serde_json::to_vec(&document).unwrap();
        fs::write(store.file_path(), &raw).unwrap();
        let expected = if document["pending"].as_array().unwrap().is_empty() {
            Vec::new()
        } else {
            vec![retained.clone()]
        };
        assert_eq!(store.list_pending_checked().unwrap(), expected);
        assert_eq!(SandPendingWakeStore::new(&root).list_pending_checked().unwrap(), expected);
        assert_eq!(fs::read(store.file_path()).unwrap(), raw, "discovery is read-only");
    }

    // General storage validation keeps incomplete legacy channel metadata for
    // the recovery adapter to refuse without inventing a replacement identity.
    let legacy = pending("legacy", PendingWakeKind::ChannelDelivery, "work", 20.0);
    store.mark_pending_checked(legacy.clone()).unwrap();
    assert!(store.has_pending_checked("legacy", PendingWakeKind::ChannelDelivery, "work").unwrap());
    assert!(store.list_pending_checked().unwrap().contains(&legacy));
    store.clear_all_checked().unwrap();
    assert!(!store.file_path().exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn checked_pending_store_refuses_entire_invalid_images_and_all_compatibility_mutations() {
    let root = temp_root("checked-pending-invalid");
    fs::create_dir_all(&root).unwrap();
    let store = SandPendingWakeStore::new(&root);
    let retained = pending("retained", PendingWakeKind::CloudAgent, "work", 10.0);
    let incoming = pending("incoming", PendingWakeKind::Shell, "new-work", 20.0);
    let valid = serde_json::json!({"version": 1, "pending": [retained.clone()]});
    let mut invalid = vec![
        "".to_string(),
        "not-json".to_string(),
        "[]".to_string(),
        "{}".to_string(),
        r#"{"version":1}"#.to_string(),
        r#"{"version":1,"pending":null}"#.to_string(),
        r#"{"version":2,"pending":[]}"#.to_string(),
        r#"{"version":null,"pending":[]}"#.to_string(),
        r#"{"version":"1","pending":[]}"#.to_string(),
        r#"{"version":1,"pending":[],"unexpected":true}"#.to_string(),
        r#"{"version":1,"version":1,"pending":[]}"#.to_string(),
        r#"{"version":1,"pending":[],"pending":[]}"#.to_string(),
        r#"{"version":1,"pending":[{"agentId":"a","agentId":"a","kind":"shell","workId":"w","markedAtMs":1}]}"#.to_string(),
        r#"{"version":1,"pending":[{"agentId":"a","kind":"shell","workId":"w","markedAtMs":1,"title":null,"title":null}]}"#.to_string(),
        r#"{"version":1,"pending":[{"agentId":"a","kind":"shell","workId":"w","markedAtMs":1,"quietOrigin":{"automation":null,"automation":null}}]}"#.to_string(),
        r#"{"version":1,"pending":[["a","shell","w",1]]}"#.to_string(),
        serde_json::json!({"version": 1, "pending": [retained.clone(), retained.clone()]}).to_string(),
        serde_json::json!({"version": 1, "pending": [retained.clone(), {"agentId": "invalid-tail"}]}).to_string(),
    ];
    for (field, value) in [
        ("agentId", serde_json::json!(" ")),
        ("agentId", serde_json::json!(7)),
        ("workId", serde_json::json!("")),
        ("kind", serde_json::json!("unknown")),
        ("kind", serde_json::json!({"cloud-agent": null})),
        ("markedAtMs", serde_json::json!("10")),
        ("markedAtMs", serde_json::Value::Null),
        ("quietOrigin", serde_json::json!(false)),
        ("quietOrigin", serde_json::json!([])),
        ("quietOrigin", serde_json::json!({"extra": true})),
        ("quietOrigin", serde_json::json!({"automation": {"id": "auto"}})),
        ("quietOrigin", serde_json::json!({"automation": ["auto", "Name"]})),
        ("quietOrigin", serde_json::json!({"automation": {"id": " ", "name": "Name"}})),
        ("quietOrigin", serde_json::json!({"automation": {"id": "auto", "name": 7}})),
        ("quietOrigin", serde_json::json!({"automation": {"id": "auto", "name": "Name", "extra": true}})),
        ("title", serde_json::json!(7)),
        ("subagentType", serde_json::json!([])),
        ("interruptedByRecreate", serde_json::Value::Null),
        ("interruptedByRecreate", serde_json::json!("false")),
        ("deliveryOperationId", serde_json::json!(true)),
        ("addressToken", serde_json::json!(7)),
        ("failureReason", serde_json::json!({})),
        ("sessionKey", serde_json::json!(false)),
        ("unknownField", serde_json::json!(true)),
    ] {
        let mut document = valid.clone();
        document["pending"][0][field] = value;
        invalid.push(document.to_string());
    }
    for field in ["agentId", "kind", "workId", "markedAtMs"] {
        let mut document = valid.clone();
        document["pending"][0].as_object_mut().unwrap().remove(field);
        invalid.push(document.to_string());
    }

    for raw in invalid {
        fs::write(store.file_path(), &raw).unwrap();
        assert!(store.list_pending_checked().is_err(), "{raw}");
        assert!(store.has_pending_checked("retained", retained.kind, "work").is_err(), "{raw}");
        let mut admitted = false;
        assert!(store.with_matching_marker_checked(&retained, || admitted = true).is_err(), "{raw}");
        assert!(!admitted);
        assert!(store.mark_pending_checked(incoming.clone()).is_err(), "{raw}");
        assert!(store.mark_if_absent_or_matches_checked(&incoming).is_err(), "{raw}");
        assert!(store.replace_if_matches_checked(&retained, retained.clone()).is_err(), "{raw}");
        assert!(store.clear_if_matches_checked(&retained).is_err(), "{raw}");
        assert!(store.clear_one_checked("retained", retained.kind, "work").is_err(), "{raw}");
        assert!(store.clear_agent_checked("retained").is_err(), "{raw}");
        assert!(store.clear_all_checked().is_err(), "{raw}");
        assert!(store.prune_stale_checked(1.0, 100.0).is_err(), "{raw}");
        assert!(!store.mark_pending(incoming.clone()));
        assert!(!store.mark_if_absent_or_matches(&incoming));
        assert!(!store.replace_if_matches(&retained, retained.clone()));
        assert!(!store.clear_if_matches(&retained));
        assert!(!store.clear_one("retained", retained.kind, "work"));
        store.clear_agent("retained");
        store.clear_all();
        assert!(store.prune_stale(1.0, 100.0).is_empty());
        assert_eq!(fs::read_to_string(store.file_path()).unwrap(), raw);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1, "refusal creates no temporary file");
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn checked_pending_store_refuses_unreadable_images_and_preserves_the_target() {
    let root = temp_root("checked-pending-io");
    fs::create_dir_all(&root).unwrap();
    let store = SandPendingWakeStore::new(&root);
    let retained = pending("a", PendingWakeKind::Shell, "work", 10.0);
    fs::create_dir(store.file_path()).unwrap();
    let child = store.file_path().join("retained-evidence");
    fs::write(&child, b"not a recovery file").unwrap();
    assert!(matches!(store.list_pending_checked(), Err(PendingWakeStoreError::Io { operation: "read", .. })));
    assert!(store.with_matching_marker_checked(&retained, || -> () { panic!("unreadable admission") }).is_err());
    assert!(!store.mark_pending(retained.clone()));
    store.clear_all();
    assert_eq!(fs::read(&child).unwrap(), b"not a recovery file");
    fs::remove_file(&child).unwrap();
    fs::remove_dir(store.file_path()).unwrap();

    fs::write(store.file_path(), [0xff, 0xfe]).unwrap();
    assert!(matches!(store.list_pending_checked(), Err(PendingWakeStoreError::Io { operation: "read", .. })));
    assert!(!store.mark_pending(retained.clone()));
    store.clear_all();
    assert_eq!(fs::read(store.file_path()).unwrap(), [0xff, 0xfe]);
    #[cfg(unix)]
    {
        std::fs::remove_file(store.file_path()).unwrap();
        let missing_target = root.join("missing-target");
        std::os::unix::fs::symlink(&missing_target, store.file_path()).unwrap();
        assert!(matches!(store.list_pending_checked(), Err(PendingWakeStoreError::Io { operation: "read", .. })));
        assert!(!store.mark_pending(retained));
        store.clear_all();
        assert_eq!(fs::read_link(store.file_path()).unwrap(), missing_target);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn checked_pending_store_validates_incoming_markers_without_mutating_known_state() {
    let root = temp_root("checked-pending-incoming");
    let store = SandPendingWakeStore::new(&root);
    let retained = pending("a", PendingWakeKind::CloudAgent, "work", 10.0);
    store.mark_pending_checked(retained.clone()).unwrap();
    let original = fs::read(store.file_path()).unwrap();
    for invalid in [
        DurablePendingWakeMarker { agent_id: " ".into(), ..retained.clone() },
        DurablePendingWakeMarker { work_id: String::new(), ..retained.clone() },
        DurablePendingWakeMarker { marked_at_ms: f64::NAN, ..retained.clone() },
        DurablePendingWakeMarker { marked_at_ms: f64::INFINITY, ..retained.clone() },
    ] {
        assert!(store.mark_pending_checked(invalid.clone()).is_err());
        assert!(store.mark_if_absent_or_matches_checked(&invalid).is_err());
        assert!(store.replace_if_matches_checked(&retained, invalid.clone()).is_err());
        assert!(store.clear_if_matches_checked(&invalid).is_err());
        assert!(store.with_matching_marker_checked(&invalid, || -> () { panic!("invalid expected marker") }).is_err());
        assert_eq!(fs::read(store.file_path()).unwrap(), original);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn matching_pending_wake_admission_is_read_only_and_refuses_stale_or_settled_snapshots() {
    let root = temp_root("checked-pending-admission");
    let store = SandPendingWakeStore::new(&root);
    let observed = pending("a", PendingWakeKind::ChannelDelivery, "work", 10.0);
    assert_eq!(store.with_matching_marker_checked(&observed, || 7).unwrap(), None);
    assert!(!root.exists());
    store.mark_pending_checked(observed.clone()).unwrap();
    let original = fs::read(store.file_path()).unwrap();
    // A pre-existing unrelated temporary path is never touched by admission.
    let unrelated = root.join("unrelated.part");
    fs::write(&unrelated, b"owned elsewhere").unwrap();
    let token = String::from("move-only admission result");
    assert_eq!(store.with_matching_marker_checked(&observed, || token).unwrap(),
        Some("move-only admission result".to_string()));
    assert_eq!(fs::read(store.file_path()).unwrap(), original);
    assert_eq!(fs::read(&unrelated).unwrap(), b"owned elsewhere");

    let replacement = DurablePendingWakeMarker { title: Some("new title".into()), ..observed.clone() };
    store.mark_pending_checked(replacement.clone()).unwrap();
    let replaced = fs::read(store.file_path()).unwrap();
    assert_eq!(store.with_matching_marker_checked(&observed, || -> () { panic!("stale admission") }).unwrap(), None);
    assert_eq!(fs::read(store.file_path()).unwrap(), replaced);
    assert_eq!(SandPendingWakeStore::new(&root).list_pending_checked().unwrap(), vec![replacement.clone()]);
    assert!(store.clear_if_matches_checked(&replacement).unwrap());
    assert_eq!(store.with_matching_marker_checked(&observed, || -> () { panic!("settled admission") }).unwrap(), None);
    assert!(!store.file_path().exists(), "admission cannot resurrect a settled marker");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn matching_pending_wake_admission_compares_every_marker_field() {
    let root = temp_root("checked-pending-complete-identity");
    let store = SandPendingWakeStore::new(&root);
    let observed = DurablePendingWakeMarker {
        quiet_origin: Some(QuietWakeOrigin {
            automation: Some(AutomationQuietOrigin { id: "auto".into(), name: "Original".into() }),
        }),
        title: Some("Original title".into()),
        subagent_type: Some("legacy-notice".into()),
        interrupted_by_recreate: true,
        delivery_operation_id: Some("distinct-operation".into()),
        address_token: Some("slack:original".into()),
        failure_reason: Some("Original failure".into()),
        session_key: Some("agent:a:main".into()),
        ..pending("a", PendingWakeKind::ChannelDelivery, "work", 10.0)
    };
    for replacement in [
        DurablePendingWakeMarker { agent_id: "another-agent".into(), ..observed.clone() },
        DurablePendingWakeMarker { kind: PendingWakeKind::Shell, ..observed.clone() },
        DurablePendingWakeMarker { work_id: "another-work".into(), ..observed.clone() },
        DurablePendingWakeMarker { marked_at_ms: 11.0, ..observed.clone() },
        DurablePendingWakeMarker { quiet_origin: None, ..observed.clone() },
        DurablePendingWakeMarker { quiet_origin: Some(QuietWakeOrigin { automation: None }), ..observed.clone() },
        DurablePendingWakeMarker { quiet_origin: Some(QuietWakeOrigin {
            automation: Some(AutomationQuietOrigin { id: "replacement".into(), name: "Original".into() }),
        }), ..observed.clone() },
        DurablePendingWakeMarker { quiet_origin: Some(QuietWakeOrigin {
            automation: Some(AutomationQuietOrigin { id: "auto".into(), name: "Replacement".into() }),
        }), ..observed.clone() },
        DurablePendingWakeMarker { title: None, ..observed.clone() },
        DurablePendingWakeMarker { subagent_type: None, ..observed.clone() },
        DurablePendingWakeMarker { interrupted_by_recreate: false, ..observed.clone() },
        DurablePendingWakeMarker { delivery_operation_id: None, ..observed.clone() },
        DurablePendingWakeMarker { address_token: Some("slack:replacement".into()), ..observed.clone() },
        DurablePendingWakeMarker { failure_reason: Some("Replacement failure".into()), ..observed.clone() },
        DurablePendingWakeMarker { session_key: Some("agent:a:replacement".into()), ..observed.clone() },
    ] {
        store.clear_all_checked().unwrap();
        store.mark_pending_checked(replacement.clone()).unwrap();
        let original = fs::read(store.file_path()).unwrap();
        assert_eq!(store.with_matching_marker_checked(&observed, || -> () { panic!("changed marker admitted") }).unwrap(), None);
        assert!(!store.clear_if_matches_checked(&observed).unwrap());
        assert_eq!(fs::read(store.file_path()).unwrap(), original);
        assert_eq!(SandPendingWakeStore::new(&root).list_pending_checked().unwrap(), vec![replacement]);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn upgrade_resume_store_coerces_upserts_and_deletes_file_when_empty() {
    let root = temp_root("upgrade");
    let store = SandUpgradeResumeStore::new(&root);
    store.mark_pending(UpgradeResumeMarker {
        agent_id: "a".into(),
        marked_at_ms: 1.0,
        source: Some("turn".into()),
        automation_id: None,
        automation_run_id: None,
        original_work: None,
    });
    store.mark_pending(UpgradeResumeMarker {
        agent_id: "a".into(),
        marked_at_ms: 2.0,
        source: Some("automation".into()),
        automation_id: Some("auto".into()),
        automation_run_id: Some("run".into()),
        original_work: None,
    });
    assert_eq!(store.list_pending().len(), 1);
    assert_eq!(store.list_pending()[0].marked_at_ms, 2.0);
    assert_eq!(
        parse_upgrade_resume_file(Some(r#"{"pending":[{"agentId":"b","markedAtMs":"bad"},{"agentId":7}]}"#))
            .len(),
        1
    );
    store.clear("a");
    assert!(store.list_pending().is_empty());
    assert!(!store.file_path().exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn upgrade_resume_store_clears_only_the_observed_marker() {
    let root = temp_root("upgrade-cas");
    let store = SandUpgradeResumeStore::new(&root);
    let old = UpgradeResumeMarker {
        agent_id: "a".into(),
        marked_at_ms: 1.0,
        source: Some("turn".into()),
        automation_id: None,
        automation_run_id: None,
        original_work: None,
    };
    let newer = UpgradeResumeMarker {
        marked_at_ms: 2.0,
        ..old.clone()
    };
    store.mark_pending(old.clone());
    store.mark_pending(newer.clone());
    assert!(!store.clear_if_matches(&old));
    assert_eq!(store.list_pending(), vec![newer.clone()]);
    assert!(store.clear_if_matches(&newer));
    assert!(store.list_pending().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn upgrade_resume_marker_rejects_incomplete_automation_identity() {
    let marker = UpgradeResumeMarker {
        agent_id: "agent-a".into(),
        marked_at_ms: 1.0,
        source: Some("automation".into()),
        automation_id: None,
        automation_run_id: Some("run-a".into()),
        original_work: None,
    };
    assert!(!upgrade_resume_marker_identity_is_well_formed(&marker));
}

fn checked_upgrade_marker(agent_id: &str, at: f64) -> UpgradeResumeMarker {
    UpgradeResumeMarker {
        agent_id: agent_id.into(), marked_at_ms: at, source: Some("turn".into()),
        automation_id: None, automation_run_id: None,
        original_work: None,
    }
}

#[test]
fn checked_upgrade_store_accepts_only_absence_explicit_empty_and_complete_legacy_or_v1_rows() {
    let root = temp_root("checked-upgrade-format");
    let store = SandUpgradeResumeStore::new(&root);
    assert!(store.list_pending_checked().unwrap().is_empty());
    assert!(!store.file_path().exists());
    fs::create_dir_all(&root).unwrap();
    for document in [r#"{"pending":[]}"#, r#"{"version":1,"pending":[]}"#] {
        fs::write(store.file_path(), document).unwrap();
        assert!(store.list_pending_checked().unwrap().is_empty());
        assert_eq!(fs::read_to_string(store.file_path()).unwrap(), document);
    }
    fs::write(store.file_path(), r#"{"pending":[{"agentId":"legacy","markedAtMs":0,"source":null}]}"#).unwrap();
    assert_eq!(store.list_pending_checked().unwrap()[0].agent_id, "legacy");
    store.mark_pending_checked(checked_upgrade_marker("current", 2.0)).unwrap();
    let reopened = SandUpgradeResumeStore::new(&root);
    let rows = reopened.list_pending_checked().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].agent_id, "legacy");
    assert_eq!(rows[1], checked_upgrade_marker("current", 2.0));
    let written: serde_json::Value = serde_json::from_slice(&fs::read(store.file_path()).unwrap()).unwrap();
    assert_eq!(written["version"], 2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn checked_upgrade_store_refuses_corruption_without_replacing_or_clearing_evidence() {
    let root = temp_root("checked-upgrade-corrupt");
    fs::create_dir_all(&root).unwrap();
    let store = SandUpgradeResumeStore::new(&root);
    let saved = checked_upgrade_marker("a", 1.0);
    for (document, expected) in [
        ("", "malformed"),
        ("not-json", "malformed"),
        (r#"[]"#, "malformed"),
        (r#"{"version":1}"#, "malformed"),
        (r#"{"version":99,"pending":[]}"#, "unsupported"),
        (r#"{"version":null,"pending":[]}"#, "unsupported"),
        (r#"{"version":1,"pending":[{"agentId":"a","markedAtMs":1},{"agentId":" ","markedAtMs":2}]}"#, "invalid"),
        (r#"{"pending":[{"agentId":"a","markedAtMs":"bad"}]}"#, "invalid"),
        (r#"{"pending":[{"agentId":"a"}]}"#, "invalid"),
        (r#"{"pending":[{"agentId":"a","markedAtMs":1,"source":7}]}"#, "invalid"),
        (r#"{"pending":[{"agentId":"a","markedAtMs":1,"automationRunId":"run"}]}"#, "invalid"),
        (r#"{"pending":[{"agentId":"a","markedAtMs":1},{"agentId":"a","markedAtMs":2}]}"#, "invalid"),
    ] {
        fs::write(store.file_path(), document).unwrap();
        let error = store.list_pending_checked().unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
        assert!(store.mark_pending_batch_checked(&[checked_upgrade_marker("new", 3.0)]).is_err());
        assert!(store.clear_if_matches_checked(&saved).is_err());
        assert!(store.clear_checked("a").is_err());
        assert!(store.clear_all_checked().is_err());
        store.mark_pending(checked_upgrade_marker("new", 3.0));
        store.clear("a");
        store.clear_all();
        assert!(!store.clear_if_matches(&saved));
        assert_eq!(fs::read_to_string(store.file_path()).unwrap(), document);
    }
    fs::remove_file(store.file_path()).unwrap();
    fs::create_dir(store.file_path()).unwrap();
    assert!(matches!(store.list_pending_checked(), Err(UpgradeResumeStoreError::Io { operation: "read", .. })));
    assert!(store.mark_pending_checked(saved).is_err());
    assert!(store.file_path().is_dir());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn checked_upgrade_batch_is_all_or_nothing_and_retains_newest_same_agent_marker() {
    let root = temp_root("checked-upgrade-batch");
    let store = SandUpgradeResumeStore::new(&root);
    let original = checked_upgrade_marker("retained", 100.0);
    store.mark_pending_checked(original.clone()).unwrap();
    let bytes = fs::read(store.file_path()).unwrap();
    let mut invalid = checked_upgrade_marker("invalid", 2.0);
    invalid.automation_run_id = Some("run-without-automation".into());
    assert!(store.mark_pending_batch_checked(&[checked_upgrade_marker("first", 1.0), invalid]).is_err());
    assert_eq!(fs::read(store.file_path()).unwrap(), bytes);
    let handles = [30.0, 10.0, 20.0].into_iter().map(|at| {
        let owner = store.clone();
        std::thread::spawn(move || owner.mark_pending_checked(checked_upgrade_marker("racing", at)).unwrap())
    }).collect::<Vec<_>>();
    for handle in handles { handle.join().unwrap(); }
    store.mark_pending_checked(checked_upgrade_marker("retained", 90.0)).unwrap();
    let rows = store.list_pending_checked().unwrap();
    assert!(rows.contains(&original));
    assert!(rows.contains(&checked_upgrade_marker("racing", 30.0)));
    assert_eq!(rows.len(), 2);
    assert!(!store.clear_if_matches_checked(&checked_upgrade_marker("racing", 20.0)).unwrap());
    assert_eq!(store.list_pending_checked().unwrap(), rows);
    assert!(!store.clear_checked("absent").unwrap());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn pending_wake_store_does_not_clear_or_overwrite_a_newer_marker() {
    let root = temp_root("pending-cas");
    let store = SandPendingWakeStore::new(&root);
    let old = pending("agent-a", PendingWakeKind::Shell, "work", 1.0);
    let newer = DurablePendingWakeMarker {
        marked_at_ms: 2.0,
        title: Some("newer".into()),
        ..old.clone()
    };
    assert!(store.mark_pending(newer.clone()));
    assert!(!store.clear_if_matches(&old));
    assert_eq!(store.list_pending(), vec![newer.clone()]);
    assert!(!store.mark_if_absent_or_matches(&old));
    assert_eq!(store.list_pending(), vec![newer]);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_transcript_runtime_owns_both_durable_recovery_stores_and_cleans_deleted_agent() {
    let root = temp_root("runtime");
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let pending_store = runtime.pending_wake_store().expect("pending store");
    let upgrade_store = runtime.upgrade_resume_store().expect("upgrade store");

    assert!(pending_store.mark_pending(pending(
        "agent-a",
        PendingWakeKind::CloudAgent,
        "work",
        5.0,
    )));
    upgrade_store.mark_pending(UpgradeResumeMarker {
        agent_id: "agent-a".into(),
        marked_at_ms: 6.0,
        source: Some("turn".into()),
        automation_id: None,
        automation_run_id: None,
        original_work: None,
    });

    runtime.clear_agent_durable_recovery("agent-a");
    assert!(pending_store.list_pending().is_empty());
    assert!(upgrade_store.list_pending().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn recovery_watch_admission_replaces_only_the_exact_existing_marker() {
    let root = temp_root("watch-admission");
    let store = SandPendingWakeStore::new(&root);
    let observed = pending("agent-a", PendingWakeKind::CloudAgent, "work", 1.0);
    let admitted = DurablePendingWakeMarker {
        marked_at_ms: 2.0,
        title: Some("Cloud agent work".into()),
        ..observed.clone()
    };
    assert!(!store.replace_if_matches(&observed, admitted.clone()));
    assert!(!store.file_path().exists(), "settled absence cannot become new work");
    assert!(store.mark_pending(observed.clone()));
    let foreign = pending("agent-b", PendingWakeKind::CloudAgent, "work", 2.0);
    assert!(!store.replace_if_matches(&observed, foreign));
    assert_eq!(store.list_pending(), vec![observed.clone()]);
    assert!(store.replace_if_matches(&observed, admitted.clone()));
    assert_eq!(SandPendingWakeStore::new(&root).list_pending(), vec![admitted.clone()]);
    assert!(!store.replace_if_matches(&observed, observed.clone()));
    assert_eq!(store.list_pending(), vec![admitted.clone()]);
    assert!(store.clear_if_matches(&admitted));
    assert!(!store.replace_if_matches(&admitted, observed));
    assert!(SandPendingWakeStore::new(&root).list_pending().is_empty());
    let _ = fs::remove_dir_all(root);
}

fn original_marker_reference(agent_id: &str) -> mahayana_host_runtime::extensions::session::agent_db::OriginalTurnReference {
    use mahayana_host_runtime::extensions::session::agent_db::{OriginalTurnReference, OriginalTurnScope};
    // This is structural marker data only. No completion slot or verified
    // capability is created by the marker store's serialization tests.
    OriginalTurnReference {
        version: 1, scope: OriginalTurnScope::NativeOrdinaryUserV1,
        agent_id: agent_id.into(), session_key: format!("agent:{agent_id}:main"),
        session_id: agent_id.into(), original_lifecycle_revision: "admission-revision".into(),
        stream_id: "original-stream".into(), input_digest: "a".repeat(64),
        client_nonce: Some("accepted-nonce".into()), user_message_id: "user-message:1".into(),
        echo_entry_id: "user-message:1".into(), request_source: None,
        provider: "fabushi".into(), model_id: "admitted-native-model".into(),
    }
}

#[test]
fn checked_upgrade_v2_roundtrips_full_reference_and_exact_clear_rejects_reference_drift() {
    let root = temp_root("original-reference-cas");
    let store = SandUpgradeResumeStore::new(&root);
    let mut marker = checked_upgrade_marker("original-agent", 1.0);
    marker.original_work = Some(original_marker_reference(&marker.agent_id));
    store.mark_pending_checked(marker.clone()).unwrap();
    let bytes = fs::read(store.file_path()).unwrap();
    let image: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(image["version"], 2);
    assert_eq!(image["pending"][0]["originalWork"]["scope"], "nativeOrdinaryUserV1");
    assert!(image["pending"][0]["originalWork"]["requestSource"].is_null());
    assert_eq!(SandUpgradeResumeStore::new(&root).list_pending_checked().unwrap(), vec![marker.clone()]);
    for field in ["modelId", "inputDigest", "streamId", "clientNonce", "originalLifecycleRevision"] {
        let mut value = serde_json::to_value(&marker).unwrap();
        value["originalWork"][field] = if field == "inputDigest" {
            "b".repeat(64).into()
        } else {
            format!("different-{field}").into()
        };
        let foreign: UpgradeResumeMarker = serde_json::from_value(value).unwrap();
        assert!(!store.clear_if_matches_checked(&foreign).unwrap(), "{field}");
        assert_eq!(fs::read(store.file_path()).unwrap(), bytes, "{field}");
    }
    let mut unbound = marker.clone();
    unbound.original_work = None;
    assert!(!store.clear_if_matches_checked(&unbound).unwrap());
    assert_eq!(fs::read(store.file_path()).unwrap(), bytes);
    assert!(store.clear_if_matches_checked(&marker).unwrap());
    assert!(!store.clear_if_matches_checked(&marker).unwrap());
    assert!(!store.file_path().exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn checked_upgrade_migration_is_whole_image_atomic_and_leaves_legacy_work_unbound() {
    let root = temp_root("original-legacy-migration");
    fs::create_dir_all(&root).unwrap();
    let store = SandUpgradeResumeStore::new(&root);
    for legacy in [
        r#"{"pending":[{"agentId":"one","markedAtMs":1,"source":"turn"},{"agentId":"two","markedAtMs":2,"source":"automation","automationId":"a","automationRunId":"r"}]}"#,
        r#"{"version":1,"pending":[{"agentId":"one","markedAtMs":1,"source":"turn"},{"agentId":"two","markedAtMs":2,"source":"automation","automationId":"a","automationRunId":"r"}]}"#,
    ] {
        fs::write(store.file_path(), legacy).unwrap();
        let inspected = store.list_pending_checked().unwrap();
        assert_eq!(fs::read_to_string(store.file_path()).unwrap(), legacy, "inspection cannot migrate");
        let migrated = store.migrate_legacy_checked().unwrap();
        assert_eq!(migrated, inspected);
        assert_eq!(migrated.len(), 2);
        assert!(migrated.iter().all(|marker| marker.original_work.is_none()));
        let bytes = fs::read(store.file_path()).unwrap();
        let image: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(image["version"], 2);
        assert_eq!(SandUpgradeResumeStore::new(&root).migrate_legacy_checked().unwrap(), migrated);
        assert_eq!(fs::read(store.file_path()).unwrap(), bytes, "current v2 migration is a no-op");
    }
    let mut bound = checked_upgrade_marker("one", 1.0);
    bound.original_work = Some(original_marker_reference("one"));
    let bound_value = serde_json::to_value(&bound).unwrap();
    let mut unknown_reference = bound_value.clone();
    unknown_reference["originalWork"]["uncheckedCompletion"] = true.into();
    let mut foreign_reference = bound_value.clone();
    foreign_reference["originalWork"]["sessionId"] = "foreign".into();
    for image in [
        serde_json::json!({"version":1,"pending":[bound_value.clone()]}),
        serde_json::json!({"pending":[bound_value]}),
        serde_json::json!({"version":2,"pending":[checked_upgrade_marker("valid-first", 1.0), unknown_reference]}),
        serde_json::json!({"version":2,"pending":[checked_upgrade_marker("valid-first", 1.0), foreign_reference]}),
        serde_json::json!({"version":2,"pending":[checked_upgrade_marker("same", 1.0), checked_upgrade_marker("same", 2.0)]}),
        serde_json::json!({"version":99,"pending":[]}),
    ] {
        let bytes = serde_json::to_vec(&image).unwrap();
        fs::write(store.file_path(), &bytes).unwrap();
        assert!(store.migrate_legacy_checked().is_err());
        assert!(store.mark_pending_checked(checked_upgrade_marker("new", 3.0)).is_err());
        assert!(store.clear_if_matches_checked(&checked_upgrade_marker("valid-first", 1.0)).is_err());
        assert_eq!(fs::read(store.file_path()).unwrap(), bytes);
    }
    // A duplicate known nested field must not be normalized away by Value.
    let duplicate = serde_json::to_string(&serde_json::json!({"version":2,"pending":[bound]})).unwrap()
        .replace("\"modelId\":\"admitted-native-model\"", "\"modelId\":\"admitted-native-model\",\"modelId\":\"different-model\"");
    fs::write(store.file_path(), &duplicate).unwrap();
    assert!(store.migrate_legacy_checked().is_err());
    assert_eq!(fs::read_to_string(store.file_path()).unwrap(), duplicate);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn checked_ack_reads_fresh_complete_image_and_never_turns_corruption_into_absence() {
    use mahayana_host_runtime::extensions::transcript::sand_ack_obligation_store::SandAckObligationStore;

    let root = temp_root("fresh-ack");
    fs::create_dir_all(&root).unwrap();
    let store = SandAckObligationStore::new(&root);
    assert_eq!(store.get("held-agent"), None, "prime the compatibility cache with absence");
    let writer = SandAckObligationStore::new(&root);
    let recorded = writer.record_send("held-agent", 10.0).unwrap().obligation;
    assert_eq!(store.get("held-agent"), None);
    let bytes = fs::read(store.file_path()).unwrap();
    assert_eq!(store.get_checked("held-agent").unwrap(), Some(recorded.clone()));
    assert_eq!(store.list_checked().unwrap(), vec![recorded.clone()]);
    assert_eq!(fs::read(store.file_path()).unwrap(), bytes);
    let stale_present = SandAckObligationStore::new(&root);
    assert_eq!(stale_present.get("held-agent"), Some(recorded.clone()));
    writer.clear("held-agent").unwrap();
    assert_eq!(stale_present.get("held-agent"), Some(recorded.clone()));
    assert_eq!(stale_present.get_checked("held-agent").unwrap(), None);

    let valid = serde_json::to_value(&recorded).unwrap();
    let mut bad_count = valid.clone();
    bad_count["redriveAttempts"] = serde_json::json!(0.5);
    let mut unknown_field = valid.clone();
    unknown_field["futureCustody"] = true.into();
    for image in [
        serde_json::json!({"version":99,"pending":[]}),
        serde_json::json!({"version":1,"pending":[valid.clone(), valid.clone()]}),
        serde_json::json!({"version":1,"pending":[valid.clone(), {"agentId":"broken"}]}),
        serde_json::json!({"version":1,"pending":[bad_count]}),
        serde_json::json!({"version":1,"pending":[unknown_field]}),
        serde_json::json!({"version":1,"pending":null}),
    ] {
        let bytes = serde_json::to_vec(&image).unwrap();
        fs::write(store.file_path(), &bytes).unwrap();
        assert!(store.list_checked().is_err());
        assert!(store.get_checked("held-agent").is_err());
        assert!(store.get_checked("absent-agent").is_err(), "every row must validate before proving any absence");
        assert_eq!(fs::read(store.file_path()).unwrap(), bytes);
    }
    for corrupt in ["not-json", r#"{"version":1,"version":1,"pending":[]}"#] {
        fs::write(store.file_path(), corrupt).unwrap();
        assert!(store.get_checked("held-agent").is_err());
        assert_eq!(fs::read_to_string(store.file_path()).unwrap(), corrupt);
    }
    fs::remove_file(store.file_path()).unwrap();
    fs::create_dir(store.file_path()).unwrap();
    assert!(store.get_checked("held-agent").is_err());
    assert!(store.file_path().is_dir());
    let _ = fs::remove_dir_all(root);
}
