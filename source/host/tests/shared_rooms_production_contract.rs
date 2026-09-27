use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::transcript_manager::TranscriptManager;
use mahayana_host_runtime::groups::group_store::{
    RemoteGroupMember, read_sand_group_config,
};
use mahayana_host_runtime::groups::remote_room_store::{
    RemoteRoomMember, read_sand_remote_room_config,
};
use serde_json::json;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-shared-rooms-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn profile(name: &str) -> SandAgentProfile {
    SandAgentProfile {
        name: name.into(),
        description: String::new(),
        title: String::new(),
        avatar_shape: String::new(),
        avatar_color: String::new(),
    }
}

#[test]
fn transcript_manager_owns_one_shared_rooms_service() {
    let root = temp_root("owner");
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, sessions);
    let first = manager.shared_rooms();
    let second = manager.shared_rooms();
    assert!(Arc::ptr_eq(&first, &second));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn hosted_room_materializes_filters_members_persists_notice_and_restamps_content() {
    let root = temp_root("hosted");
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let member = sessions
        .materialize_new_session(Some(&profile("Member")), "user", None)
        .expect("member");
    let group = sessions
        .materialize_new_session(Some(&profile("Nested group")), "user", None)
        .expect("nested group");
    mahayana_host_runtime::groups::group_store::write_sand_group_config(
        sessions.agents_root().join(&group.id),
        &mahayana_host_runtime::groups::group_store::SandGroupConfig {
            version: mahayana_host_runtime::groups::group_store::GROUP_CONFIG_VERSION,
            member_ids: vec![member.id.clone()],
            remote_members: None,
            shared_room_id: None,
        },
    )
    .expect("nested group marker");

    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));
    let rooms = manager.shared_rooms();
    let remote = RemoteGroupMember {
        owner_auth_id: "remote-owner".into(),
        agent_id: "remote-agent".into(),
        name: "Remote".into(),
        avatar_data_url: Some("data:image/png;base64,AA==".into()),
    };
    let room_agent = rooms
        .ensure_hosted_shared_room(
            "room-hosted",
            "Shared group",
            &[member.id.clone(), group.id.clone(), "missing".into()],
            &[remote.clone()],
            true,
        )
        .expect("ensure hosted")
        .expect("created");

    let binding = read_sand_group_config(sessions.agents_root().join(&room_agent))
        .expect("group binding");
    assert_eq!(binding.shared_room_id.as_deref(), Some("room-hosted"));
    assert_eq!(binding.member_ids, vec![member.id.clone()]);
    assert_eq!(binding.remote_members, Some(vec![remote]));
    assert_eq!(
        rooms.get_shared_room_id_for_agent(&room_agent).as_deref(),
        Some("room-hosted")
    );

    assert!(rooms
        .append_shared_room_activity_notice("room-hosted", "Member is responding.")
        .expect("notice"));
    let entries = sessions
        .read_agent_transcript_entries(&room_agent)
        .expect("entries");
    assert!(entries.iter().any(|entry| {
        entry.get("kind").and_then(serde_json::Value::as_str) == Some("notice")
            && entry.get("text").and_then(serde_json::Value::as_str)
                == Some("Member is responding.")
    }));

    sessions
        .append_agent_transcript_entries(
            &room_agent,
            &[json!({
                "kind": "message",
                "id": "t1u",
                "role": "user",
                "content": "hello",
                "timestampMs": 1.0
            })],
        )
        .expect("append content");
    assert!(rooms
        .restamp_room_entry("room-hosted", "t1u", 42.0)
        .expect("restamp"));
    let stamped = sessions
        .read_agent_transcript_entries(&room_agent)
        .expect("stamped entries")
        .into_iter()
        .find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some("t1u"))
        .expect("stamped entry");
    assert_eq!(
        stamped.get("timestampMs").and_then(serde_json::Value::as_f64),
        Some(42.0)
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn mirror_room_materializes_remote_binding_filters_self_and_can_be_revoked() {
    let root = temp_root("mirror");
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));
    let rooms = manager.shared_rooms();
    let members = vec![
        RemoteRoomMember {
            kind: "human".into(),
            auth_id: "self".into(),
            agent_id: String::new(),
            display_name: "Me".into(),
            avatar_url: None,
        },
        RemoteRoomMember {
            kind: "agent".into(),
            auth_id: "remote".into(),
            agent_id: "agent-r".into(),
            display_name: "Remote agent".into(),
            avatar_url: Some("avatar".into()),
        },
    ];

    let room_agent = rooms
        .ensure_mirror_room(
            "room-mirror",
            "Remote room",
            "host",
            "Host",
            Some("host-avatar"),
            &members,
            "self",
        )
        .expect("mirror");

    let binding = read_sand_remote_room_config(sessions.agents_root().join(&room_agent))
        .expect("remote binding");
    assert_eq!(binding.room_id, "room-mirror");
    assert_eq!(binding.host_auth_id, "host");
    assert_eq!(binding.members.len(), 1);
    assert_eq!(binding.members[0].auth_id, "remote");
    assert_eq!(binding.is_revoked, None);

    rooms
        .mark_mirror_room_revoked("room-mirror")
        .expect("revoke");
    let revoked = read_sand_remote_room_config(sessions.agents_root().join(&room_agent))
        .expect("revoked binding");
    assert_eq!(revoked.is_revoked, Some(true));

    let _ = fs::remove_dir_all(root);
}
