use std::fs;
use std::sync::{Arc, Barrier, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::{
    SandAgentProfile, get_sand_profile_path, read_sand_profile_file,
};
use mahayana_host_runtime::agents::settings_file::{
    get_sand_settings_path, read_sand_settings_file,
};
use mahayana_host_runtime::extensions::session::agent_db::{
    read_persisted_agent_name, read_persisted_latest_root_blob_id,
};
use mahayana_host_runtime::extensions::session::production::{
    FallbackSession, ProductionSessionWorkers,
};
use mahayana_host_runtime::extensions::session::session_diagnostics::{
    SessionDiagnostic, pin_session_diagnostics_reporter,
};
use mahayana_host_runtime::extensions::session::session_materialization::{
    MAX_AGENTS_PER_USER, SessionMaterializationError, count_owned_agents,
    is_agent_cap_reached, list_agent_record_ids, list_pruned_placeholder_ids,
    materialize_new_session, open_existing_session,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-session-materialization-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn materialize_new_session_creates_frozen_identity_db_profile_and_settings() {
    let root = temp_root("create");
    let profile = SandAgentProfile {
        name: "  Agent One  ".into(),
        description: "  description  ".into(),
        title: "  title  ".into(),
        avatar_shape: "  circle  ".into(),
        avatar_color: "  blue  ".into(),
    };
    let session = materialize_new_session(
        &root,
        500,
        Some(&profile),
        "dev",
        Some("research"),
    )
    .expect("materialize");

    assert!(session.db_path.is_file());
    assert_eq!(count_owned_agents(&root).expect("count"), 1);
    assert_eq!(list_agent_record_ids(&root).expect("ids"), vec![session.id.clone()]);
    assert_eq!(
        read_persisted_agent_name(&session.db_path, 500)
            .expect("metadata")
            .as_deref(),
        Some("New Agent")
    );
    assert!(
        read_persisted_latest_root_blob_id(&session.db_path, 500)
            .expect("root")
            .is_empty()
    );
    let db = rusqlite::Connection::open(&session.db_path).expect("db");
    let kv = |key: &str| -> String {
        db.query_row("SELECT value FROM kv WHERE key=?1", [key], |row| row.get(0))
            .expect("kv")
    };
    assert_eq!(kv("origin"), "dev");
    assert_eq!(kv("purpose"), "research");
    assert_eq!(kv("introductionPending"), "1");

    assert_eq!(
        read_sand_profile_file(get_sand_profile_path(root.join(&session.id)))
            .expect("profile"),
        SandAgentProfile {
            name: "Agent One".into(),
            description: "description".into(),
            title: "title".into(),
            avatar_shape: "circle".into(),
            avatar_color: "blue".into(),
        }
    );
    let settings = read_sand_settings_file(get_sand_settings_path(root.join(&session.id)));
    assert!(settings.notify_on_agent_updates);
    assert!(!settings.hidden_from_sidebar);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn existing_session_open_repairs_missing_profile_and_settings_without_replacing_profile() {
    let root = temp_root("open");
    let created = materialize_new_session(&root, 500, None, "user", None)
        .expect("create");
    let agent_dir = root.join(&created.id);
    fs::remove_file(get_sand_profile_path(&agent_dir)).expect("remove profile");
    fs::remove_file(get_sand_settings_path(&agent_dir)).expect("remove settings");

    let opened = open_existing_session(&root, 500, &created.id)
        .expect("open")
        .expect("existing");
    assert_eq!(opened.id, created.id);
    assert_eq!(opened.profile.name, "New Agent");
    assert!(get_sand_profile_path(&agent_dir).is_file());
    assert!(get_sand_settings_path(&agent_dir).is_file());

    let mut custom = opened.profile.clone();
    custom.name = "Keep Me".into();
    mahayana_host_runtime::agents::agent_profile::write_sand_profile_file(
        get_sand_profile_path(&agent_dir),
        &custom,
    )
    .expect("custom profile");
    let reopened = open_existing_session(&root, 500, &created.id)
        .expect("reopen")
        .expect("existing");
    assert_eq!(reopened.profile.name, "Keep Me");

    assert!(
        open_existing_session(&root, 500, "missing")
            .expect("missing lookup")
            .is_none()
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn agent_cap_uses_owned_directory_count_and_blocks_minting() {
    let root = temp_root("cap");
    fs::create_dir_all(&root).expect("root");
    for index in 0..MAX_AGENTS_PER_USER {
        fs::create_dir_all(root.join(format!("agent-{index:02}"))).expect("slot");
    }
    assert!(is_agent_cap_reached(&root).expect("cap"));
    assert!(matches!(
        materialize_new_session(&root, 500, None, "user", None),
        Err(SessionMaterializationError::Limit)
    ));
    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_mint_queue_serializes_near_cap_create_sessions() {
    let root = temp_root("mint-queue");
    fs::create_dir_all(&root).expect("root");
    for index in 0..(MAX_AGENTS_PER_USER - 1) {
        fs::create_dir_all(root.join(format!("existing-{index:02}"))).expect("slot");
    }

    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let barrier = Arc::new(Barrier::new(3));
    let mut threads = Vec::new();
    for index in 0..2 {
        let workers = Arc::clone(&workers);
        let barrier = Arc::clone(&barrier);
        threads.push(std::thread::spawn(move || {
            barrier.wait();
            workers.materialize_new_session(
                Some(&SandAgentProfile {
                    name: format!("Concurrent {index}"),
                    description: String::new(),
                    title: String::new(),
                    avatar_shape: String::new(),
                    avatar_color: String::new(),
                }),
                "user",
                None,
            )
        }));
    }
    barrier.wait();

    let results = threads
        .into_iter()
        .map(|thread| thread.join().expect("mint thread"))
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);
    assert_eq!(count_owned_agents(&root).expect("final count"), MAX_AGENTS_PER_USER);
    assert!(is_agent_cap_reached(&root).expect("cap after concurrent mint"));

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn pruned_placeholder_selection_preserves_active_and_visible_agents() {
    use std::collections::BTreeSet;

    let root = temp_root("pruned-placeholders");
    let active = materialize_new_session(&root, 500, None, "user", None)
        .expect("active session");
    let visible = materialize_new_session(&root, 500, None, "user", None)
        .expect("visible session");
    let pruned = materialize_new_session(&root, 500, None, "user", None)
        .expect("pruned session");

    let visible_ids = BTreeSet::from([visible.id.clone()]);
    assert_eq!(
        list_pruned_placeholder_ids(
            &root,
            Some(active.id.as_str()),
            &visible_ids,
        )
        .expect("placeholder ids"),
        vec![pruned.id.clone()]
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_fallback_adopts_existing_session_when_cap_remains_full() {
    let root = temp_root("fallback-adopt");
    let existing = materialize_new_session(&root, 500, None, "user", None)
        .expect("existing session");
    for index in 0..(MAX_AGENTS_PER_USER - 1) {
        fs::create_dir_all(root.join(format!("slot-{index:02}"))).expect("slot");
    }
    assert_eq!(count_owned_agents(&root).expect("owned count"), MAX_AGENTS_PER_USER);

    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let fallback = workers
        .create_fallback_session(Some(existing.id.as_str()))
        .expect("fallback adoption");
    match fallback {
        FallbackSession::Existing(prepared) => assert_eq!(prepared.agent_id, existing.id),
        FallbackSession::Created(record) => panic!("unexpected fallback mint: {}", record.id),
    }
    assert_eq!(count_owned_agents(&root).expect("count after adoption"), MAX_AGENTS_PER_USER);

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_cap_reclaim_removes_invisible_placeholder_before_mint() {
    let root = temp_root("cap-reclaim");
    let active = materialize_new_session(
        &root,
        500,
        Some(&SandAgentProfile {
            name: "Active".into(),
            description: "visible".into(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        }),
        "user",
        None,
    )
    .expect("active session");
    let placeholder = materialize_new_session(&root, 500, None, "user", None)
        .expect("placeholder session");
    for index in 0..(MAX_AGENTS_PER_USER - 2) {
        fs::create_dir_all(root.join(format!("slot-{index:02}"))).expect("slot");
    }
    assert_eq!(count_owned_agents(&root).expect("owned count"), MAX_AGENTS_PER_USER);

    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let created = workers
        .materialize_new_session_with_active(
            Some(&SandAgentProfile {
                name: "After reclaim".into(),
                description: String::new(),
                title: String::new(),
                avatar_shape: String::new(),
                avatar_color: String::new(),
            }),
            "user",
            None,
            Some(active.id.as_str()),
        )
        .expect("mint after reclaim");

    assert!(!root.join(&placeholder.id).exists());
    assert!(root.join(&created.id).exists());
    assert_eq!(count_owned_agents(&root).expect("final count"), MAX_AGENTS_PER_USER);

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_fallback_reports_failed_adoption_and_continues_to_valid_session() {
    let root = temp_root("fallback-diagnostic");
    let existing = materialize_new_session(
        &root,
        500,
        Some(&SandAgentProfile {
            name: "Visible Fallback".into(),
            description: "keeps the durable session visible during cap reclaim".into(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        }),
        "user",
        None,
    )
    .expect("valid existing session");

    let corrupt_id = "000-corrupt";
    let corrupt_dir = root.join(corrupt_id);
    fs::create_dir_all(&corrupt_dir).expect("corrupt agent dir");
    let corrupt_db = corrupt_dir.join("store.db");
    let empty_db = rusqlite::Connection::open(&corrupt_db).expect("empty sqlite");
    drop(empty_db);

    for index in 0..(MAX_AGENTS_PER_USER - 2) {
        fs::create_dir_all(root.join(format!("slot-{index:02}"))).expect("cap slot");
    }
    assert_eq!(
        count_owned_agents(&root).expect("owned count"),
        MAX_AGENTS_PER_USER
    );

    let reports = Arc::new(Mutex::new(Vec::<SessionDiagnostic>::new()));
    let captured = Arc::clone(&reports);
    pin_session_diagnostics_reporter(Some(Arc::new(move |report| {
        captured
            .lock()
            .expect("diagnostic capture")
            .push(report.clone());
    })));

    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let fallback = workers
        .create_fallback_session(Some(corrupt_id))
        .expect("fallback after corrupt adoption");
    match fallback {
        FallbackSession::Existing(prepared) => assert_eq!(prepared.agent_id, existing.id),
        FallbackSession::Created(record) => panic!("unexpected fallback mint: {}", record.id),
    }

    let snapshot = reports.lock().expect("diagnostic snapshot").clone();
    assert!(snapshot.iter().any(|report| {
        report.family == "materialize"
            && report.kind == "fallback_adopt_failed"
            && report
                .metadata
                .get("agentId")
                .and_then(serde_json::Value::as_str)
                == Some(corrupt_id)
            && report
                .metadata
                .get("errorClass")
                .and_then(serde_json::Value::as_str)
                == Some("String")
    }));

    pin_session_diagnostics_reporter(None);
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
