use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::production_runtime::ProductionTranscriptRuntime;
use mahayana_host_runtime::extensions::transcript::send_group_fanout::{
    GroupMemberTurnExecutor, LocalGroupFanoutDisposition, dispatch_local_group_send,
};
use mahayana_host_runtime::groups::group_store::{
    GROUP_CONFIG_VERSION, SandGroupConfig, write_sand_group_config,
};
use serde_json::json;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-group-fanout-{label}-{}-{suffix}",
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
fn local_group_fanout_reads_room_history_executes_members_and_durably_posts_authored_messages() {
    let root = temp_root("local");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let alice = sessions
        .materialize_new_session(Some(&profile("Alice")), "user", None)
        .expect("Alice");
    let bob = sessions
        .materialize_new_session(Some(&profile("Bob")), "user", None)
        .expect("Bob");
    let room = sessions
        .materialize_new_session(Some(&profile("Team")), "user", None)
        .expect("room");

    write_sand_group_config(
        root.join(&room.id),
        &SandGroupConfig {
            version: GROUP_CONFIG_VERSION,
            member_ids: vec![alice.id.clone(), bob.id.clone()],
            remote_members: None,
            shared_room_id: None,
        },
    )
    .expect("group config");
    sessions
        .append_agent_transcript_entries(
            &room.id,
            &[json!({
                "kind":"message",
                "id":"t0u",
                "role":"user",
                "content":"Please answer as a team",
                "timestampMs":1,
            })],
        )
        .expect("user message");

    let calls = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&calls);
    let executor: GroupMemberTurnExecutor = Arc::new(move |request| {
        observed
            .lock()
            .expect("calls")
            .push((request.member.id.clone(), request.prompt.clone(), request.system_prompt.clone()));
        Ok(vec![format!("{} reply", request.member.name)])
    });
    let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
    let outcome = dispatch_local_group_send(
        Arc::clone(&sessions),
        runtime,
        &room.id,
        0,
        executor,
    )
    .expect("fanout");
    let LocalGroupFanoutDisposition::Completed {
        posted_messages,
        member_failures,
    } = outcome
    else {
        panic!("expected local completion");
    };
    assert!(posted_messages > 0);
    assert!(member_failures.is_empty());
    let calls = calls.lock().expect("calls");
    assert!(!calls.is_empty());
    assert!(calls.iter().all(|(_, _, system)| system.contains("SendMessage")));

    let entries = sessions
        .read_agent_transcript_entries(&room.id)
        .expect("room transcript");
    let authored = entries
        .iter()
        .filter(|entry| entry.get("kind").and_then(serde_json::Value::as_str) == Some("send-message"))
        .collect::<Vec<_>>();
    assert!(!authored.is_empty());
    assert!(authored.iter().all(|entry| {
        entry.get("author")
            .and_then(|author| author.get("id"))
            .and_then(serde_json::Value::as_str)
            .is_some()
    }));

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn remote_or_shared_group_is_deferred_instead_of_fake_local_execution() {
    let root = temp_root("remote");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let room = sessions
        .materialize_new_session(Some(&profile("Shared")), "user", None)
        .expect("room");
    write_sand_group_config(
        root.join(&room.id),
        &SandGroupConfig {
            version: GROUP_CONFIG_VERSION,
            member_ids: vec![],
            remote_members: None,
            shared_room_id: Some("shared-1".into()),
        },
    )
    .expect("group config");
    let executor: GroupMemberTurnExecutor = Arc::new(|_| panic!("must not execute"));
    let outcome = dispatch_local_group_send(
        Arc::clone(&sessions),
        Arc::new(ProductionTranscriptRuntime::new(Some(&root))),
        &room.id,
        0,
        executor,
    )
    .expect("fanout");
    assert_eq!(
        outcome,
        LocalGroupFanoutDisposition::DeferredRemote {
            shared_room_id: Some("shared-1".into()),
            remote_member_count: 0,
        }
    );
    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}
