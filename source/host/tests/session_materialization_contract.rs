use std::fs;
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
use mahayana_host_runtime::extensions::session::session_materialization::{
    MAX_AGENTS_PER_USER, SessionMaterializationError, count_owned_agents,
    is_agent_cap_reached, list_agent_record_ids, materialize_new_session,
    open_existing_session,
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
