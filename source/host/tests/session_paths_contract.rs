use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::session_paths::{
    ACTIVE_AGENT_FILENAME, CONNECTOR_SECRETS_DIRNAME, CONVERSATION_BLOBS_FILENAME,
    HIDDEN_ENTRY_REPAIR_VERSION, LEGACY_GROUP_MEMBERS_DIRNAME,
    SAND_CONVERSATION_ROOT_SLOT_ID, STALE_ROOT_CLEANUP_VERSION, STORE_FILENAME,
    get_agent_db_path, get_connector_secrets_root, get_sand_transcripts_dir_with,
    is_stale_root_gc_enabled_from_env, pin_stale_root_gc, stat_if_exists,
};

#[test]
fn session_paths_match_frozen_names_root_slot_and_gc_override_contract() {
    assert_eq!(STORE_FILENAME, "store.db");
    assert_eq!(CONVERSATION_BLOBS_FILENAME, "conversation-blobs.db");
    assert_eq!(
        SAND_CONVERSATION_ROOT_SLOT_ID,
        b"sand-live-conversation-root-v1__"
    );
    assert_eq!(STALE_ROOT_CLEANUP_VERSION, 1);
    assert_eq!(ACTIVE_AGENT_FILENAME, "active-agent.json");
    assert_eq!(HIDDEN_ENTRY_REPAIR_VERSION, 1);
    assert_eq!(LEGACY_GROUP_MEMBERS_DIRNAME, "members");
    assert_eq!(CONNECTOR_SECRETS_DIRNAME, "connector-secrets");

    let mut environment = BTreeMap::new();
    pin_stale_root_gc(false);
    assert!(!is_stale_root_gc_enabled_from_env(&environment));
    pin_stale_root_gc(true);
    assert!(is_stale_root_gc_enabled_from_env(&environment));

    environment.insert("SAND_STALE_ROOT_GC".into(), " off ".into());
    assert!(!is_stale_root_gc_enabled_from_env(&environment));
    environment.insert("SAND_STALE_ROOT_GC".into(), "TRUE".into());
    assert!(is_stale_root_gc_enabled_from_env(&environment));
    pin_stale_root_gc(false);
}

#[test]
fn session_paths_validate_agent_ids_and_preserve_connector_secret_sibling_layout() {
    let agents_root = Path::new("/tmp/fabushi-session-root/agents");
    assert_eq!(
        get_agent_db_path(agents_root, "agent-a").expect("valid agent path"),
        agents_root.join("agent-a").join(STORE_FILENAME)
    );
    assert!(get_agent_db_path(agents_root, "../escape").is_err());
    assert_eq!(
        get_connector_secrets_root(Some(agents_root)),
        Path::new("/tmp/fabushi-session-root").join(CONNECTOR_SECRETS_DIRNAME)
    );
}

#[test]
fn transcript_path_uses_the_same_sand_root_resolution_as_the_host() {
    let home = Path::new("/home/tester");
    let cwd = Path::new("/workspace");
    let argv = vec!["mahayana-app-host".to_string()];
    let environment = BTreeMap::from([(
        "SAND_DATA_ROOT".to_string(),
        "/var/lib/fabushi".to_string(),
    )]);
    assert_eq!(
        get_sand_transcripts_dir_with(home, &argv, &environment, cwd),
        Path::new("/var/lib/fabushi").join("agent-transcripts")
    );
}

#[test]
fn stat_if_exists_matches_the_frozen_missing_file_contract() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "fabushi-session-paths-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("fixture root");
    let present = root.join("store.db");
    fs::write(&present, b"db").expect("fixture file");

    assert!(stat_if_exists(&present).is_some());
    assert!(stat_if_exists(&root.join("missing.db")).is_none());

    fs::remove_dir_all(root).expect("remove fixture");
}
