use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::{
    SandAgentProfile, get_sand_profile_path, write_sand_profile_file,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::profile_watch::{
    SAND_DEFAULT_AGENT_NAME, get_agent_display_profile, is_profile_watch_filename,
    resolve_agent_profile, watched_profile_path_agent_id,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-profile-watch-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn profile_watch_filters_exact_frozen_file_family() {
    assert!(is_profile_watch_filename("profile.json"));
    assert!(is_profile_watch_filename("settings.json"));
    assert!(is_profile_watch_filename("avatar.png"));
    assert!(is_profile_watch_filename("avatar.webp"));
    assert!(!is_profile_watch_filename("notes.txt"));

    let root = temp_root("paths");
    assert_eq!(
        watched_profile_path_agent_id(
            &root,
            &root.join("agent-a").join("profile.json")
        )
        .as_deref(),
        Some("agent-a")
    );
    assert!(watched_profile_path_agent_id(
        &root,
        &root.join("agent-a").join("nested").join("profile.json")
    )
    .is_none());
}

#[test]
fn display_and_resolved_profile_match_frozen_defaulting_contract() {
    let root = temp_root("profile");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let record = sessions
        .materialize_new_session(None, "user", None)
        .expect("materialize");
    let profile_path = get_sand_profile_path(root.join(&record.id));
    write_sand_profile_file(
        &profile_path,
        &SandAgentProfile {
            name: "   ".into(),
            description: "description".into(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        },
    )
    .expect("profile");

    let display = get_agent_display_profile(&sessions, &record.id).expect("display");
    assert_eq!(display.name, SAND_DEFAULT_AGENT_NAME);
    assert_eq!(display.description, "description");

    let resolved = resolve_agent_profile(&root.join(&record.id).join("store.db"));
    assert_eq!(resolved.name, SAND_DEFAULT_AGENT_NAME);
    assert_eq!(resolved.description, "description");
    assert_eq!(resolved.file_path, profile_path);
    assert_eq!(
        resolved.settings_file_path,
        root.join(&record.id).join("settings.json")
    );

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}
