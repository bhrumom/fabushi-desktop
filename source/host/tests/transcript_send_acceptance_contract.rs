use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::session::agent_db::read_persisted_agent_serde_snapshot;
use mahayana_host_runtime::extensions::session::agent_db_serde::AwaitingUserResponse;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::send_acceptance::{
    LEGACY_SAND_DEFAULT_AGENT_NAME, SAND_DEFAULT_AGENT_NAME, build_seeded_agent_name,
    is_sand_default_agent_name, mark_accepted_send_activity, prepare_send_acceptance,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-send-acceptance-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn frozen_default_name_and_seed_normalization_contract_is_preserved() {
    assert!(is_sand_default_agent_name(SAND_DEFAULT_AGENT_NAME));
    assert!(is_sand_default_agent_name(LEGACY_SAND_DEFAULT_AGENT_NAME));
    assert!(is_sand_default_agent_name("  New Bot  "));
    assert!(!is_sand_default_agent_name("Custom"));

    assert_eq!(
        build_seeded_agent_name("  first   task\nwith\tspaces  "),
        "first task with spaces"
    );
    assert_eq!(build_seeded_agent_name("   "), "New conversation");
    let long = "a".repeat(90);
    assert_eq!(build_seeded_agent_name(&long).encode_utf16().count(), 72);
}

#[test]
fn production_acceptance_seeds_first_default_profile_and_clears_waiting_state() {
    let root = temp_root("production");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let record = workers
        .materialize_new_session(
            Some(&SandAgentProfile {
                name: SAND_DEFAULT_AGENT_NAME.into(),
                description: "keep description".into(),
                title: "keep title".into(),
                avatar_shape: "round".into(),
                avatar_color: "violet".into(),
            }),
            "user",
            None,
        )
        .expect("materialize agent");

    workers
        .set_agent_awaiting_user_response(
            &record.id,
            Some(&AwaitingUserResponse {
                tab_id: "tab-1".into(),
                reason: "ask".into(),
                since: 9.0,
            }),
        )
        .expect("seed awaiting response");
    assert!(workers
        .get_agent_introduction_pending(&record.id)
        .expect("introduction state"));

    let effects = prepare_send_acceptance(
        &workers,
        &record.id,
        0,
        "  first   accepted\nprompt  ",
    )
    .expect("prepare acceptance");
    assert_eq!(effects.seeded_name.as_deref(), Some("first accepted prompt"));
    assert!(effects.awaiting_cleared);
    assert!(effects.introduction_cleared);

    let profile = workers
        .get_agent_profile_text(&record.id)
        .expect("profile read")
        .expect("profile");
    assert_eq!(profile.name, "first accepted prompt");
    assert_eq!(profile.description, "keep description");
    assert_eq!(profile.title, "keep title");
    assert_eq!(profile.avatar_shape, "round");
    assert_eq!(profile.avatar_color, "violet");

    let snapshot = read_persisted_agent_serde_snapshot(&record.db_path, 500)
        .expect("session snapshot");
    assert!(snapshot.awaiting_user_response.is_none());
    assert!(!workers
        .get_agent_introduction_pending(&record.id)
        .expect("introduction cleared"));

    assert!(mark_accepted_send_activity(&workers, &record.id, 1234.0)
        .expect("activity update"));
    let snapshot = read_persisted_agent_serde_snapshot(&record.db_path, 500)
        .expect("updated snapshot");
    assert_eq!(snapshot.unread_state.last_activity_at, 1234.0);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_acceptance_does_not_rename_non_default_or_non_empty_conversation() {
    let root = temp_root("no-reseed");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let record = workers
        .materialize_new_session(
            Some(&SandAgentProfile {
                name: "Custom".into(),
                description: String::new(),
                title: String::new(),
                avatar_shape: String::new(),
                avatar_color: String::new(),
            }),
            "user",
            None,
        )
        .expect("materialize agent");

    let custom = prepare_send_acceptance(&workers, &record.id, 0, "rename me")
        .expect("custom acceptance");
    assert_eq!(custom.seeded_name, None);
    assert_eq!(
        workers
            .get_agent_profile_text(&record.id)
            .expect("profile read")
            .expect("profile")
            .name,
        "Custom"
    );

    let second = workers
        .materialize_new_session(
            Some(&SandAgentProfile {
                name: SAND_DEFAULT_AGENT_NAME.into(),
                description: String::new(),
                title: String::new(),
                avatar_shape: String::new(),
                avatar_color: String::new(),
            }),
            "user",
            None,
        )
        .expect("second agent");
    let non_empty = prepare_send_acceptance(&workers, &second.id, 1, "do not seed")
        .expect("non-empty acceptance");
    assert_eq!(non_empty.seeded_name, None);
    assert_eq!(
        workers
            .get_agent_profile_text(&second.id)
            .expect("profile read")
            .expect("profile")
            .name,
        SAND_DEFAULT_AGENT_NAME
    );

    let _ = fs::remove_dir_all(root);
}
