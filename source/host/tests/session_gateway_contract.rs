use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::session::gateway::{
    SessionGatewayError, dispatch_production_session_gateway_call,
    persist_accepted_send_prompt,
};
use mahayana_host_runtime::extensions::session::native_messaging::FabushiNativeMessagingClient;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use serde_json::json;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-session-gateway-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn dispatch(
    runtime: &Arc<ProductionSessionWorkers>,
    method: &str,
    args: serde_json::Value,
) -> serde_json::Value {
    dispatch_production_session_gateway_call(runtime, method, &args)
        .expect("handled method")
        .expect("successful method")
}


fn spawn_remote_history_server() -> (String, thread::JoinHandle<Vec<String>>) {
    // Three ascending-ID coverage pages, then the existing latest snapshot.
    // Use the bounded fixture so a missing or unexpected request fails causally.
    let replies = [(1_u64, 200_u64), (201, 400), (401, 401), (202, 401)]
        .into_iter()
        .map(|(low, high)| {
            let messages = (low..=high)
                .map(|id| {
                    let minute = id / 60;
                    let second = id % 60;
                    json!({
                        "id": id,
                        "senderUserId": 2,
                        "senderUsername": "peer",
                        "recipientUserId": 1,
                        "recipientUsername": "local",
                        "text": format!("remote-{id}"),
                        "clientRequestId": serde_json::Value::Null,
                        "createdAt": format!("1970-01-01T00:{minute:02}:{second:02}Z"),
                        "readAt": serde_json::Value::Null,
                        "isOutgoing": false
                    })
                })
                .collect::<Vec<_>>();
            HumanMessageFixtureReply::Json(200, json!({
                "success": true,
                "data": { "messages": messages }
            }))
        })
        .collect();
    spawn_human_message_fixture(replies)
}

#[test]
fn production_gateway_reads_transcript_pages_and_counts_agents_without_compat_host() {
    let root = temp_root("transcript");
    let agents = root.join("agents");
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let record = runtime
        .materialize_new_session(None, "user", None)
        .expect("agent");
    runtime
        .append_agent_transcript_entries(
            &record.id,
            &[
                json!({"id":"m1","kind":"message","role":"user","content":"hello","timestampMs":10}),
                json!({"id":"m2","kind":"send-message","message":{"type":"text","content":"reply"},"timestampMs":20}),
            ],
        )
        .expect("entries");

    assert_eq!(dispatch(&runtime, "countAgents", json!({})), json!(1));
    assert_eq!(
        dispatch(&runtime, "getAgentTranscript", json!({"id":record.id}))["0"],
        serde_json::Value::Null
    );
    let transcript = dispatch(&runtime, "getAgentTranscript", json!({"id":record.id}));
    assert_eq!(transcript.as_array().map(Vec::len), Some(2));

    let page = dispatch(
        &runtime,
        "getAgentTranscriptPage",
        json!({"id":record.id,"untilMs":100,"limit":10}),
    );
    assert_eq!(page["entries"].as_array().map(Vec::len), Some(2));
    assert_eq!(page["nextBeforeSeq"], serde_json::Value::Null);

    let window = dispatch(
        &runtime,
        "getAgentTranscriptWindow",
        json!({"id":record.id,"limit":10}),
    );
    assert_eq!(window["entries"].as_array().map(Vec::len), Some(2));
    assert!(window["threadCounts"].is_object());

    let tail = dispatch(
        &runtime,
        "getAgentTranscriptTail",
        json!({"id":record.id,"limit":1}),
    );
    assert_eq!(tail["entries"].as_array().map(Vec::len), Some(1));

    // The shipping Renderer reloads a selected Agent through openAgentTail.
    // Rust-created Agents must stay on the same Session owner instead of
    // falling through to the compatibility Host roster.
    let opened_tail = dispatch(
        &runtime,
        "openAgentTail",
        json!({"id":record.id,"limit":1}),
    );
    assert_eq!(opened_tail, tail);

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_gateway_composes_channel_metadata_and_secrets() {
    let root = temp_root("channels");
    let agents = root.join("agents");
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let record = runtime
        .materialize_new_session(None, "user", None)
        .expect("agent");

    let connected = dispatch(
        &runtime,
        "connectChannel",
        json!({"id":record.id,"platform":"slack","token":"secret"}),
    );
    assert_eq!(connected.as_array().map(Vec::len), Some(1));
    assert_eq!(connected[0]["platform"], "slack");
    assert_eq!(
        dispatch(&runtime, "getAgentChannels", json!({"id":record.id})),
        connected
    );
    assert_eq!(
        dispatch(&runtime, "refreshChannel", json!({"id":record.id})),
        connected
    );
    let disconnected = dispatch(
        &runtime,
        "disconnectChannel",
        json!({"id":record.id,"platform":"slack"}),
    );
    assert_eq!(disconnected, json!([]));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_gateway_creates_a_real_session_record_for_the_shipping_new_chat_path() {
    let root = temp_root("create-agent");
    let agents = root.join("agents");
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));

    let created = dispatch(
        &runtime,
        "createAgent",
        json!({
            "name": "New chat",
            "description": "",
            "origin": "user",
            "isIntroductionSuppressed": true,
            "isKickstartRequested": false,
            "clientNonce": "e2e-create-agent"
        }),
    );

    let agent_id = created["agent"]["id"]
        .as_str()
        .expect("created agent id")
        .to_string();
    assert_eq!(created["agent"]["name"], "New chat");
    assert_eq!(created["agent"]["isActive"], true);
    assert_eq!(created["transcript"], json!([]));
    assert!(runtime.session_db_path(&agent_id).expect("db path").is_file());

    let listed = dispatch(&runtime, "listAgents", json!({}));
    assert_eq!(listed.as_array().map(Vec::len), Some(1));
    assert_eq!(listed[0]["id"], agent_id);
    assert_eq!(listed[0]["name"], "New chat");
    assert!(!runtime
        .get_agent_introduction_pending(&agent_id)
        .expect("introduction state"));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_gateway_cuts_over_safe_agent_lifecycle_mutations_to_rust_session_owner() {
    let root = temp_root("agent-lifecycle");
    let agents = root.join("agents");
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let record = runtime
        .materialize_new_session(
            Some(&SandAgentProfile {
                name: "Fresh Agent".into(),
                description: String::new(),
                title: String::new(),
                avatar_shape: String::new(),
                avatar_color: String::new(),
            }),
            "user",
            None,
        )
        .expect("agent");

    // Frozen Grok roster semantics intentionally hide a completely blank
    // default "Grok" agent. Give this lifecycle fixture a real identity so
    // listAgents tests the shipping cutover without weakening that filter.
    let listed = dispatch(&runtime, "listAgents", json!({}));
    assert_eq!(listed.as_array().map(Vec::len), Some(1));
    assert_eq!(listed[0]["id"], record.id);

    let updated = dispatch(
        &runtime,
        "updateAgent",
        json!({
            "id": record.id,
            "profile": {
                "name": "  Lifecycle Agent  ",
                "description": "  production gateway  ",
                "title": "  Operator  ",
                "avatarShape": " rounded ",
                "avatarColor": " violet "
            }
        }),
    );
    assert_eq!(updated["name"], "Lifecycle Agent");
    assert_eq!(updated["description"], "production gateway");
    assert_eq!(updated["title"], "Operator");

    assert_eq!(
        dispatch(
            &runtime,
            "setAgentUnread",
            json!({"id": record.id, "isUnread": true, "atMs": 1234}),
        ),
        serde_json::Value::Null
    );
    assert_eq!(
        dispatch(
            &runtime,
            "setAgentNotifyOnUpdates",
            json!({"id": record.id, "isEnabled": false}),
        ),
        serde_json::Value::Null
    );
    assert_eq!(
        dispatch(
            &runtime,
            "setAgentHiddenFromSidebar",
            json!({"id": record.id, "isHidden": true}),
        ),
        serde_json::Value::Null
    );
    let mutated = dispatch(&runtime, "listAgents", json!({}));
    assert_eq!(mutated[0]["hasUnread"], true);
    assert_eq!(mutated[0]["notifyOnUpdatesEnabled"], false);
    assert_eq!(mutated[0]["isHiddenFromSidebar"], true);

    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAusB9Y9Z2S8AAAAASUVORK5CYII=";
    let _summary = dispatch(
        &runtime,
        "setAgentAvatarBytes",
        json!({"id": record.id, "pngBase64": png}),
    );
    let avatar = dispatch(&runtime, "getAgentAvatar", json!({"id": record.id}));
    assert!(avatar["version"].as_str().is_some_and(|value| !value.is_empty()));
    assert!(avatar["dataUrl"]
        .as_str()
        .is_some_and(|value| value.starts_with("data:image/png;base64,")));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn accepted_send_prompt_is_persisted_into_the_rust_authoritative_transcript() {
    let root = temp_root("send-prompt-persistence");
    let agents = root.join("agents");
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let record = runtime
        .materialize_new_session(None, "user", None)
        .expect("agent");

    persist_accepted_send_prompt(
        &runtime,
        &json!({
            "agentId": record.id,
            "prompt": "",
            "attachmentPaths": ["/tmp/attachment-only.txt"],
            "attachmentNames": ["attachment-only.txt"],
            "clientNonce": "nonce-attachment-only",
            "composedAtMs": 1234
        }),
        &json!({
            "accepted": true,
            "operationId": "operation-attachment-only"
        }),
    )
    .expect("persist accepted sendPrompt");

    let transcript = dispatch(
        &runtime,
        "getAgentTranscript",
        json!({"id":record.id}),
    );
    let entries = transcript.as_array().expect("transcript array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["id"], "t0ua0");
    assert_eq!(entries[0]["kind"], "user-attachment");
    assert_eq!(entries[0]["file_path"], "/tmp/attachment-only.txt");
    assert_eq!(entries[0]["file_name"], "attachment-only.txt");
    assert_eq!(entries[0]["clientNonce"], "nonce-attachment-only");
    assert!(entries[0]["batchId"].as_str().is_some_and(|value| !value.is_empty()));

    // Production nonce admission normally coalesces this before persistence.
    // The persistence boundary itself also remains idempotent by clientNonce.
    persist_accepted_send_prompt(
        &runtime,
        &json!({
            "agentId": record.id,
            "prompt": "",
            "attachmentPaths": ["/tmp/attachment-only.txt"],
            "attachmentNames": ["attachment-only.txt"],
            "clientNonce": "nonce-attachment-only",
            "composedAtMs": 1234
        }),
        &json!({
            "accepted": true,
            "operationId": "operation-attachment-only"
        }),
    )
    .expect("idempotent persistence");
    assert_eq!(
        dispatch(&runtime, "getAgentTranscript", json!({"id":record.id}))
            .as_array()
            .map(Vec::len),
        Some(1)
    );


    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn accepted_send_prompt_uses_frozen_entry_ids_and_reply_fork_stamping() {
    let root = temp_root("send-prompt-threading");
    let agents = root.join("agents");
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let record = runtime.materialize_new_session(None, "user", None).expect("agent");

    let first = persist_accepted_send_prompt(
        &runtime,
        &json!({"agentId":record.id,"prompt":"first","clientNonce":"nonce-first","composedAtMs":1000}),
        &json!({"accepted":true,"operationId":"operation-first"}),
    ).expect("first send");
    assert_eq!(first.as_deref(), Some("t0u"));

    let second = persist_accepted_send_prompt(
        &runtime,
        &json!({
            "agentId":record.id,
            "prompt":" reply ",
            "richText":"**reply**",
            "replyToId":"t0u",
            "isFork":true,
            "clientNonce":"nonce-reply",
            "composedAtMs":2000
        }),
        &json!({"accepted":true,"operationId":"operation-reply"}),
    ).expect("reply send");
    assert_eq!(second.as_deref(), Some("t1u"));

    let transcript = dispatch(&runtime, "getAgentTranscript", json!({"id":record.id}));
    let entries = transcript.as_array().expect("transcript array");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["id"], "t0u");
    assert_eq!(entries[1]["id"], "t1u");
    assert_eq!(entries[1]["content"], "reply");
    assert_eq!(entries[1]["richText"], "**reply**");
    assert_eq!(entries[1]["replyTo"], "t0u");
    assert_eq!(entries[1]["branched"], true);
    assert_eq!(entries[1]["timestampMs"], 2000.0);
    assert_eq!(entries[1]["sentWhileOfflineAtMs"], 2000.0);


    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn malformed_session_gateway_requests_fail_closed_and_unknown_methods_fall_through() {
    let root = temp_root("bad-request");
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    assert_eq!(
        dispatch_production_session_gateway_call(
            &runtime,
            "getAgentTranscript",
            &json!({"id":""}),
        ),
        Some(Err(SessionGatewayError::BadRequest(
            "missing or invalid id".to_string()
        )))
    );
    assert!(dispatch_production_session_gateway_call(
        &runtime,
        "not-a-session-method",
        &json!({}),
    )
    .is_none());
    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_gateway_owns_conversation_outline_for_fresh_agents() {
    let root = temp_root("conversation-outline");
    let agents = root.join("agents");
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let record = runtime.materialize_new_session(None, "user", None).expect("agent");

    assert_eq!(
        dispatch(
            &runtime,
            "getConversationOutline",
            json!({"id":record.id}),
        ),
        json!([])
    );

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_gateway_owns_group_creation_membership_and_nested_group_rejection() {
    let root = temp_root("groups");
    let agents = root.join("agents");
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));

    let first = dispatch(
        &runtime,
        "createAgent",
        json!({"name":"Alice","description":"","origin":"user","isIntroductionSuppressed":true}),
    );
    let second = dispatch(
        &runtime,
        "createAgent",
        json!({"name":"Bob","description":"","origin":"user","isIntroductionSuppressed":true}),
    );
    let first_id = first["agent"]["id"].as_str().expect("alice id").to_string();
    let second_id = second["agent"]["id"].as_str().expect("bob id").to_string();

    let created = dispatch(
        &runtime,
        "createGroup",
        json!({
            "name":"Team",
            "description":"Production group",
            "memberIds":[first_id.clone(), second_id.clone(), first_id.clone()]
        }),
    );
    let group_id = created["agent"]["id"].as_str().expect("group id").to_string();
    assert_eq!(created["agent"]["isGroup"], true);
    assert_eq!(
        created["agent"]["memberIds"],
        json!([first_id.clone(), second_id.clone()])
    );
    assert_eq!(created["transcript"], json!([]));

    let duplicate = dispatch(
        &runtime,
        "createGroup",
        json!({
            "name":"Ignored duplicate name",
            "memberIds":[second_id.clone(), first_id.clone()]
        }),
    );
    assert_eq!(duplicate["agent"]["id"], group_id);

    let updated = dispatch(
        &runtime,
        "setGroupMembers",
        json!({"id":group_id.clone(),"memberIds":[second_id.clone()]}),
    );
    assert_eq!(updated["isGroup"], true);
    assert_eq!(updated["memberIds"], json!([second_id.clone()]));

    let nested = dispatch_production_session_gateway_call(
        &runtime,
        "createGroup",
        &json!({"name":"Nested","memberIds":[group_id.clone()]}),
    )
    .expect("createGroup handled")
    .expect_err("nested group rejected");
    assert!(matches!(nested, SessionGatewayError::BadRequest(message) if message.contains("individual agents")));

    let listed = dispatch(&runtime, "listAgents", json!({}));
    assert!(listed
        .as_array()
        .expect("agents")
        .iter()
        .any(|agent| agent["id"] == group_id && agent["isGroup"] == true));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn human_conversation_uses_session_transcript_owner_without_becoming_an_agent() {
    let root = temp_root("human-conversation");
    let agents = root.join("agents");
    let memory = Arc::new(mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(agents.clone()));
    let runtime = Arc::new(ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
        &agents,
        500,
        Some("human-local".into()),
        Arc::new(|| None),
        memory,
    ));
    let agent = runtime
        .materialize_new_session(
            Some(&SandAgentProfile {
                name: "Agent peer".into(),
                description: String::new(),
                title: String::new(),
                avatar_shape: String::new(),
                avatar_color: String::new(),
            }),
            "user",
            None,
        )
        .expect("agent");

    let created = dispatch(
        &runtime,
        "createHumanConversation",
        json!({
            "localHumanId": "human-local",
            "peerHumanId": "human-peer",
            "title": "Human peer"
        }),
    );
    let conversation_id = created["id"]
        .as_str()
        .expect("conversation id")
        .to_string();
    assert_eq!(created["kind"], "human");
    let human_db = runtime
        .open_human_conversation_db_owner(&conversation_id)
        .expect("Human conversation database");
    assert_eq!(
        human_db.get_metadata("conversationKind").expect("kind"),
        Some(json!("human"))
    );
    assert_eq!(human_db.get_metadata("agentId").expect("agent id"), None);
    assert_eq!(human_db.get_metadata("blobEncryptionKey").expect("blob key"), None);
    assert_eq!(human_db.get_metadata("mode").expect("agent mode"), None);
    assert_eq!(dispatch(&runtime, "countAgents", json!({})), json!(1));
    let agents_list = dispatch(&runtime, "listAgents", json!({}));
    assert_eq!(agents_list.as_array().map(Vec::len), Some(1));
    assert_eq!(agents_list[0]["id"], agent.id);

    let conversations = dispatch(&runtime, "listHumanConversations", json!({}));
    assert_eq!(conversations.as_array().map(Vec::len), Some(1));
    assert_eq!(conversations[0]["id"], conversation_id);
    assert_eq!(conversations[0]["updatedAt"], 0.0);

    let spoofed_sender = dispatch_production_session_gateway_call(
        &runtime,
        "sendHumanMessage",
        &json!({
            "conversationId": conversation_id,
            "senderId": "human-peer",
            "text": "spoofed",
            "clientNonce": "human-spoof"
        }),
    )
    .expect("handled spoofed Human send")
    .expect_err("renderer-provided peer identity must not become the local sender");
    assert!(matches!(spoofed_sender, SessionGatewayError::BadRequest(_)));

    let first = dispatch(
        &runtime,
        "sendHumanMessage",
        json!({
            "conversationId": conversation_id,
            "senderId": "human-local",
            "text": "hello human",
            "clientNonce": "human-nonce-1",
            "composedAtMs": 1234
        }),
    );
    assert_eq!(first["authorKind"], "human");
    assert_eq!(first["authorId"], "human-local");
    assert_eq!(first["clientNonce"], "human-nonce-1");
    assert_eq!(
        human_db
            .get_unread_state()
            .expect("Human conversation unread state")
            .unread_count,
        0.0,
        "outgoing Human messages must not reuse Agent unread activity semantics"
    );
    assert_eq!(
        human_db
            .get_metadata("lastActivityAt")
            .expect("Human conversation activity metadata"),
        Some(json!(1234.0))
    );

    let replay = dispatch(
        &runtime,
        "sendHumanMessage",
        json!({
            "conversationId": conversation_id,
            "senderId": "human-local",
            "text": "hello human",
            "clientNonce": "human-nonce-1",
            "composedAtMs": 1234
        }),
    );
    assert_eq!(replay, first);

    let reply = dispatch(
        &runtime,
        "sendHumanMessage",
        json!({
            "conversationId": conversation_id,
            "senderId": "human-local",
            "text": "reply with file",
            "clientNonce": "human-nonce-2",
            "composedAtMs": 1500,
            "replyToId": "human-message:human-nonce-1",
            "attachments": [{
                "path": "committed-resource-report",
                "name": "report.pdf"
            }]
        }),
    );
    assert_eq!(reply["replyToId"], "human-message:human-nonce-1");
    assert_eq!(
        reply["attachments"],
        json!([{
            "path": "committed-resource-report",
            "name": "report.pdf"
        }])
    );

    let attachment_only = dispatch(
        &runtime,
        "sendHumanMessage",
        json!({
            "conversationId": conversation_id,
            "senderId": "human-local",
            "text": "",
            "clientNonce": "human-nonce-3",
            "composedAtMs": 1600,
            "attachments": [{
                "path": "committed-resource-image",
                "name": "image.png"
            }]
        }),
    );
    assert_eq!(attachment_only["content"], "");
    assert_eq!(attachment_only["attachments"][0]["name"], "image.png");

    let invalid_reply = dispatch_production_session_gateway_call(
        &runtime,
        "sendHumanMessage",
        &json!({
            "conversationId": conversation_id,
            "senderId": "human-local",
            "text": "bad reply",
            "clientNonce": "human-nonce-invalid-reply",
            "replyToId": "foreign-message"
        }),
    )
    .expect("handled Human reply")
    .expect_err("reply target outside the Human transcript must fail closed");
    assert!(matches!(invalid_reply, SessionGatewayError::Internal(message) if message.contains("reply target")));

    let agent_result = runtime
        .append_human_agent_message(
            &conversation_id,
            &agent.id,
            "runner-stream-1",
            "trusted Agent result",
            2345.0,
        )
        .expect("Host-internal Agent result projection");
    assert_eq!(agent_result["role"], "assistant");
    assert_eq!(agent_result["authorKind"], "agent");
    assert_eq!(agent_result["authorId"], agent.id);
    assert_eq!(agent_result["sourceStreamId"], "runner-stream-1");
    assert_eq!(
        runtime
            .append_human_agent_message(
                &conversation_id,
                &agent.id,
                "runner-stream-1",
                "trusted Agent result",
                2345.0,
            )
            .expect("idempotent Agent result replay"),
        agent_result
    );
    assert!(
        runtime
            .append_human_agent_message(
                &conversation_id,
                &agent.id,
                "runner-stream-1",
                "different Agent result",
                2345.0,
            )
            .is_err(),
        "one Runner stream must not identify different Human-transcript content"
    );
    assert!(
        dispatch_production_session_gateway_call(
            &runtime,
            "appendHumanAgentMessage",
            &json!({
                "conversationId": conversation_id,
                "agentId": agent.id,
                "streamId": "renderer-forgery",
                "content": "forged"
            }),
        )
        .is_none(),
        "trusted Agent result projection must not be renderer-callable"
    );

    let transcript = dispatch(
        &runtime,
        "getHumanConversationTranscript",
        json!({"conversationId": conversation_id}),
    );
    assert_eq!(transcript.as_array().map(Vec::len), Some(4));
    assert_eq!(transcript[0]["id"], "human-message:human-nonce-1");
    assert_eq!(transcript[1], reply);
    assert_eq!(transcript[2], attachment_only);
    assert_eq!(transcript[3], agent_result);

    let human_tail = dispatch(
        &runtime,
        "getHumanConversationTranscriptTail",
        json!({
            "conversationId": conversation_id,
            "limit": 2
        }),
    );
    assert_eq!(human_tail["entries"], json!([attachment_only.clone(), agent_result.clone()]));
    let older_cursor = human_tail["nextBeforeSeq"]
        .as_i64()
        .expect("Human transcript tail should expose an older-page cursor");
    let human_older = dispatch(
        &runtime,
        "getHumanConversationTranscriptTail",
        json!({
            "conversationId": conversation_id,
            "limit": 2,
            "beforeSeq": older_cursor
        }),
    );
    assert_eq!(human_older["entries"], json!([first.clone(), reply.clone()]));
    assert!(human_older["nextBeforeSeq"].is_null());

    let attachment_search = dispatch(
        &runtime,
        "searchHumanMessages",
        json!({
            "conversationId": conversation_id,
            "query": "REPORT",
            "limit": 10
        }),
    );
    assert_eq!(attachment_search.as_array().map(Vec::len), Some(1));
    assert_eq!(attachment_search[0]["id"], reply["id"]);
    let agent_search = dispatch(
        &runtime,
        "searchHumanMessages",
        json!({
            "conversationId": conversation_id,
            "query": "trusted agent",
            "limit": 1
        }),
    );
    assert_eq!(agent_search, json!([agent_result.clone()]));
    let empty_search = dispatch_production_session_gateway_call(
        &runtime,
        "searchHumanMessages",
        &json!({
            "conversationId": conversation_id,
            "query": "   "
        }),
    )
    .expect("handled Human search")
    .expect_err("empty Human search query must fail closed");
    assert!(matches!(empty_search, SessionGatewayError::BadRequest(_)));

    let conversations_after_send = dispatch(&runtime, "listHumanConversations", json!({}));
    assert_eq!(conversations_after_send[0]["updatedAt"], 2345.0);

    let conflicting = dispatch_production_session_gateway_call(
        &runtime,
        "sendHumanMessage",
        &json!({
            "conversationId": conversation_id,
            "senderId": "human-local",
            "text": "different content",
            "clientNonce": "human-nonce-1",
            "composedAtMs": 1234
        }),
    )
    .expect("handled Human send")
    .expect_err("nonce collision must reject different content");
    match conflicting {
        SessionGatewayError::Internal(message) => {
            assert!(message.contains("different content"));
        }
        other => panic!("unexpected nonce collision error: {other:?}"),
    }

    runtime.shutdown();

    let restarted_memory = Arc::new(mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(agents.clone()));
    let restarted = Arc::new(ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
        &agents,
        500,
        Some("human-local".into()),
        Arc::new(|| None),
        restarted_memory,
    ));
    let restored = dispatch(
        &restarted,
        "getHumanConversationTranscript",
        json!({"conversationId": conversation_id}),
    );
    assert_eq!(restored, transcript);
    assert_eq!(dispatch(&restarted, "countAgents", json!({})), json!(1));

    let foreign_memory = Arc::new(mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(agents.clone()));
    let foreign = Arc::new(ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
        &agents,
        500,
        Some("human-other".into()),
        Arc::new(|| None),
        foreign_memory,
    ));
    assert_eq!(dispatch(&foreign, "listHumanConversations", json!({})), json!([]));
    let denied = dispatch_production_session_gateway_call(
        &foreign,
        "getHumanConversationTranscript",
        &json!({"conversationId": conversation_id}),
    )
    .expect("handled foreign Human transcript")
    .expect_err("foreign Human identity must not read the conversation");
    assert!(matches!(denied, SessionGatewayError::Internal(_)));
    let foreign_search = dispatch_production_session_gateway_call(
        &foreign,
        "searchHumanMessages",
        &json!({
            "conversationId": conversation_id,
            "query": "hello"
        }),
    )
    .expect("handled foreign Human search")
    .expect_err("foreign Human identity must not search the conversation");
    assert!(matches!(foreign_search, SessionGatewayError::Internal(_)));
    foreign.shutdown();
    restarted.shutdown();
    let _ = fs::remove_dir_all(root);
}


fn read_fixture_http_request(stream: &mut std::net::TcpStream) -> String {
    read_fixture_http_request_before(stream, None)
}

fn read_fixture_http_request_before(
    stream: &mut std::net::TcpStream,
    deadline: Option<std::time::Instant>,
) -> String {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut header_end = None;
    let mut content_length = 0usize;
    loop {
        if let Some(deadline) = deadline {
            let remaining = deadline.checked_duration_since(std::time::Instant::now())
                .filter(|remaining| !remaining.is_zero())
                .expect("fixture request exceeded its total read deadline");
            stream.set_read_timeout(Some(remaining)).expect("bound fixture request read");
        }
        let read = stream.read(&mut chunk).expect("read fixture request");
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        if header_end.is_none() {
            if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                header_end = Some(index + 4);
                let headers = String::from_utf8_lossy(&request[..index + 4]);
                content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.split_once(':').and_then(|(name, value)| {
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())
                                .flatten()
                        })
                    })
                    .unwrap_or(0);
            }
        }
        if let Some(header_end) = header_end {
            if request.len() >= header_end + content_length {
                break;
            }
        }
    }
    String::from_utf8_lossy(&request).into_owned()
}

fn spawn_remote_reaction_server() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind reaction fixture");
    let address = listener.local_addr().expect("fixture address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for (active, count, reacted_by_me) in [(true, 2_u64, true), (false, 1_u64, false)] {
            let (mut stream, _) = listener.accept().expect("accept reaction mutation");
            requests.push(read_fixture_http_request(&mut stream));
            let body = json!({
                "success": true,
                "messageId": 11,
                "reactions": [{
                    "emoji": "👍",
                    "count": count,
                    "reactedByMe": reacted_by_me
                }]
            }).to_string();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            ).expect("write reaction response");
            stream.flush().expect("flush reaction response");
            let _ = active;
        }
        requests
    });
    (format!("http://{address}"), handle)
}

fn spawn_remote_reply_attachment_server() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind reply attachment fixture");
    let address = listener.local_addr().expect("fixture address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();

        let (mut upload_stream, _) = listener.accept().expect("accept resource upload");
        requests.push(read_fixture_http_request(&mut upload_stream));
        let upload_body = json!({
            "success": true,
            "resource": {
                "resourceId": "resource-1",
                "name": "report.pdf",
                "contentType": "application/octet-stream",
                "size": 4,
                "createdAt": "2026-10-05T01:00:00Z"
            }
        }).to_string();
        write!(
            upload_stream,
            "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            upload_body.len(),
            upload_body
        ).expect("write resource response");
        upload_stream.flush().expect("flush resource response");

        let (mut send_stream, _) = listener.accept().expect("accept message send");
        requests.push(read_fixture_http_request(&mut send_stream));
        let send_body = json!({
            "success": true,
            "deduplicated": false,
            "message": {
                "id": 11,
                "senderUserId": 1,
                "senderUsername": "local",
                "recipientUserId": 2,
                "recipientUsername": "peer",
                "text": "reply with file",
                "clientRequestId": "shipping-reply-1",
                "createdAt": "2026-10-05T01:00:01Z",
                "readAt": serde_json::Value::Null,
                "isOutgoing": true,
                "replyToMessageId": 10,
                "attachments": [{
                    "resourceId": "resource-1",
                    "name": "report.pdf",
                    "contentType": "application/octet-stream",
                    "size": 4,
                    "createdAt": "2026-10-05T01:00:00Z"
                }],
                "reactions": []
            }
        }).to_string();
        write!(
            send_stream,
            "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            send_body.len(),
            send_body
        ).expect("write send response");
        send_stream.flush().expect("flush send response");

        let (mut resource_stream, _) = listener.accept().expect("accept resource download");
        requests.push(read_fixture_http_request(&mut resource_stream));
        let resource = [1_u8, 2, 3, 4];
        write!(
            resource_stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            resource.len()
        ).expect("write resource download headers");
        resource_stream.write_all(&resource).expect("write resource download body");
        resource_stream.flush().expect("flush resource download");

        requests
    });
    (format!("http://{address}"), handle)
}

#[test]
fn shipping_human_send_uploads_attachment_and_binds_reply_to_server_identity() {
    let root = temp_root("human-shipping-reply-attachment");
    let agents = root.join("agents");
    fs::create_dir_all(&root).expect("fixture root");
    let credential_path = root.join("host-credential.json");
    fs::write(
        &credential_path,
        r#"{"accessToken":"fixture-token","deviceId":"device-a","userId":1}"#,
    )
    .expect("credential");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&credential_path)
            .expect("credential metadata")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&credential_path, permissions).expect("private credential");
    }
    let attachment_path = root.join("report.pdf");
    fs::write(&attachment_path, [1_u8, 2, 3, 4]).expect("attachment fixture");

    let (base_url, server) = spawn_remote_reply_attachment_server();
    fs::write(&credential_path, serde_json::json!({
        "accessToken": "fixture-token", "deviceId": "device-a", "userId": 1, "issuer": &base_url
    }).to_string()).expect("issuer-bound fixture credential");
    let client = Arc::new(
        FabushiNativeMessagingClient::new(&base_url, &credential_path)
            .expect("shipping messaging client"),
    );
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    let runtime = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_messaging_and_dependencies(
            &agents,
            500,
            Some("1".into()),
            Some(client),
            true,
            None,
            Arc::new(|| None),
            memory,
        ),
    );
    let created = runtime
        .create_human_conversation("2", "Peer")
        .expect("Human conversation");
    let conversation_id = created["id"]
        .as_str()
        .expect("conversation id")
        .to_string();
    let owner = runtime
        .open_human_conversation_db_owner(&conversation_id)
        .expect("conversation owner");
    assert!(owner
        .append_transcript_entry(&json!({
            "id": "human-server-message:10",
            "kind": "message",
            "role": "user",
            "authorKind": "human",
            "authorId": "2",
            "content": "remote anchor",
            "timestampMs": 1000.0,
            "delivery": "sent",
            "remoteMessageId": "10",
            "remoteCreatedAt": "2026-10-05T01:03:21Z"
        }))
        .expect("remote anchor"));

    let sent = dispatch(
        &runtime,
        "sendHumanMessage",
        json!({
            "conversationId": conversation_id,
            "senderId": "1",
            "text": "reply with file",
            "clientNonce": "shipping-reply-1",
            "replyToId": "human-server-message:10",
            "attachments": [{
                "path": attachment_path.to_string_lossy(),
                "name": "report.pdf"
            }]
        }),
    );
    assert_eq!(sent["id"], "human-server-message:11");
    assert_eq!(sent["remoteMessageId"], "11");
    assert_eq!(sent["replyToId"], "human-server-message:10");
    assert_eq!(sent["remoteReplyToMessageId"], "10");
    assert_eq!(sent["attachments"][0]["resourceId"], "resource-1");
    let materialized_path = sent["attachments"][0]["path"]
        .as_str()
        .expect("materialized attachment path");
    assert!(materialized_path.ends_with("human-message-resources/resource-1/report.pdf"));
    assert_eq!(fs::read(materialized_path).expect("materialized attachment bytes"), [1_u8, 2, 3, 4]);
    assert_eq!(sent["attachments"][0]["name"], "report.pdf");

    let requests = server.join().expect("shipping send fixture");
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("POST /api/social/message-resources "));
    assert!(requests[0].contains("filename=\"report.pdf\""));
    assert!(requests[1].starts_with("POST /api/social/messages "));
    assert!(requests[1].contains(r#""replyToMessageId":"10""#));
    assert!(requests[1].contains(r#""resourceId":"resource-1""#));
    assert!(requests[1].contains(r#""clientRequestId":"shipping-reply-1""#));
    assert!(requests[2].starts_with("GET /api/social/message-resources/resource-1 "));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

fn spawn_ambiguous_human_send_server() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ambiguous send fixture");
    let address = listener.local_addr().expect("fixture address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();

        let (mut upload_stream, _) = listener.accept().expect("accept durable resource upload");
        requests.push(read_fixture_http_request(&mut upload_stream));
        let upload_body = json!({
            "success": true,
            "resource": {
                "resourceId": "durable-resource-1",
                "name": "resume.pdf",
                "contentType": "application/octet-stream",
                "size": 4,
                "createdAt": "2026-10-05T02:00:00Z"
            }
        }).to_string();
        write!(
            upload_stream,
            "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            upload_body.len(),
            upload_body
        ).expect("write durable resource response");
        upload_stream.flush().expect("flush durable resource response");

        let (mut first_send, _) = listener.accept().expect("accept ambiguous message send");
        requests.push(read_fixture_http_request(&mut first_send));
        drop(first_send);

        let (mut replay_send, _) = listener.accept().expect("accept replayed message send");
        requests.push(read_fixture_http_request(&mut replay_send));
        let send_body = json!({
            "success": true,
            "deduplicated": true,
            "message": {
                "id": 21,
                "senderUserId": 1,
                "senderUsername": "local",
                "recipientUserId": 2,
                "recipientUsername": "peer",
                "text": "durable restart",
                "clientRequestId": "durable-send-1",
                "createdAt": "2026-10-05T02:00:01Z",
                "readAt": serde_json::Value::Null,
                "isOutgoing": true,
                "replyToMessageId": serde_json::Value::Null,
                "attachments": [{
                    "resourceId": "durable-resource-1",
                    "name": "resume.pdf",
                    "contentType": "application/octet-stream",
                    "size": 4,
                    "createdAt": "2026-10-05T02:00:00Z"
                }],
                "reactions": []
            }
        }).to_string();
        write!(
            replay_send,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            send_body.len(),
            send_body
        ).expect("write replay response");
        replay_send.flush().expect("flush replay response");

        let (mut resource_stream, _) = listener.accept().expect("accept durable resource download");
        requests.push(read_fixture_http_request(&mut resource_stream));
        let resource = [5_u8, 6, 7, 8];
        write!(
            resource_stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            resource.len()
        ).expect("write durable resource download headers");
        resource_stream.write_all(&resource).expect("write durable resource download body");
        resource_stream.flush().expect("flush durable resource download");

        let (mut sync_stream, _) = listener.accept().expect("accept post-settlement sync");
        requests.push(read_fixture_http_request(&mut sync_stream));
        assert!(requests.last().expect("history request").contains("&afterId=0"));
        let sync_body = json!({"success":true,"data":{
            "messages":[],"historyCursor":{"mode":"after-id","afterId":0},"nextAfterId":0
        }}).to_string();
        write!(
            sync_stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            sync_body.len(),
            sync_body
        ).expect("write sync response");
        sync_stream.flush().expect("flush sync response");
        requests
    });
    (format!("http://{address}"), handle)
}

#[test]
fn shipping_human_send_recovers_ambiguous_dispatch_after_restart_without_reupload() {
    let root = temp_root("human-shipping-durable-send");
    let agents = root.join("agents");
    fs::create_dir_all(&root).expect("fixture root");
    let credential_path = root.join("host-credential.json");
    fs::write(
        &credential_path,
        r#"{"accessToken":"fixture-token","deviceId":"device-a","userId":1}"#,
    ).expect("credential");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&credential_path)
            .expect("credential metadata")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&credential_path, permissions).expect("private credential");
    }
    let attachment_path = root.join("resume.pdf");
    fs::write(&attachment_path, [5_u8, 6, 7, 8]).expect("attachment fixture");
    let (base_url, server) = spawn_ambiguous_human_send_server();
    fs::write(&credential_path, serde_json::json!({
        "accessToken": "fixture-token", "deviceId": "device-a", "userId": 1, "issuer": &base_url
    }).to_string()).expect("issuer-bound fixture credential");

    let make_runtime = || {
        let client = Arc::new(
            FabushiNativeMessagingClient::new(&base_url, &credential_path)
                .expect("shipping messaging client"),
        );
        let memory = Arc::new(
            mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
                agents.clone(),
            ),
        );
        ProductionSessionWorkers::with_agents_root_identity_messaging_and_dependencies(
            &agents,
            500,
            Some("1".into()),
            Some(client),
            true,
            None,
            Arc::new(|| None),
            memory,
        )
    };

    let runtime = make_runtime();
    let created = runtime
        .create_human_conversation("2", "Peer")
        .expect("Human conversation");
    let conversation_id = created["id"]
        .as_str()
        .expect("conversation id")
        .to_string();
    let first_error = runtime
        .append_human_message(
            &conversation_id,
            "durable restart",
            "durable-send-1",
            Some(2000.0),
            None,
            &[json!({
                "path": attachment_path.to_string_lossy(),
                "name": "resume.pdf"
            })],
        )
        .expect_err("lost response must remain unsettled");
    assert!(!first_error.is_empty());

    let pending_owner = runtime
        .open_human_conversation_db_owner(&conversation_id)
        .expect("pending owner");
    let pending = pending_owner
        .get_transcript_entries()
        .expect("pending transcript");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["id"], "human-message:durable-send-1");
    assert_eq!(pending[0]["delivery"], "pending");
    assert_eq!(
        pending[0]["remoteDispatchAttachments"][0]["resourceId"],
        "durable-resource-1"
    );
    runtime.shutdown();

    let restarted = make_runtime();
    let settled = restarted
        .sync_human_conversation(&conversation_id)
        .expect("restart sync settles pending send");
    assert_eq!(settled.len(), 1);
    assert_eq!(settled[0]["id"], "human-server-message:21");
    assert_eq!(settled[0]["remoteMessageId"], "21");
    assert_eq!(settled[0]["delivery"], "sent");
    assert_eq!(
        settled[0]["attachments"][0]["resourceId"],
        "durable-resource-1"
    );
    assert_eq!(settled[0].get("remoteDispatchAttachments"), None);
    restarted.shutdown();

    let requests = server.join().expect("durable send fixture");
    assert_eq!(requests.len(), 5);
    assert!(requests[0].starts_with("POST /api/social/message-resources "));
    assert!(requests[1].starts_with("POST /api/social/messages "));
    assert!(requests[2].starts_with("POST /api/social/messages "));
    assert!(requests[2].contains(r#""clientRequestId":"durable-send-1""#));
    assert!(requests[2].contains(r#""resourceId":"durable-resource-1""#));
    assert!(requests[3].starts_with("GET /api/social/message-resources/durable-resource-1 "));
    assert!(requests[4].starts_with("GET /api/social/messages?"));
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.starts_with("POST /api/social/message-resources "))
            .count(),
        1
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_reaction_toggles_backend_truth_and_updates_canonical_transcript() {
    let root = temp_root("human-shipping-reaction");
    let agents = root.join("agents");
    fs::create_dir_all(&root).expect("fixture root");
    let credential_path = root.join("host-credential.json");
    fs::write(
        &credential_path,
        r#"{"accessToken":"fixture-token","deviceId":"device-a","userId":1}"#,
    )
    .expect("credential");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&credential_path)
            .expect("credential metadata")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&credential_path, permissions).expect("private credential");
    }

    let (base_url, server) = spawn_remote_reaction_server();
    fs::write(&credential_path, serde_json::json!({
        "accessToken": "fixture-token", "deviceId": "device-a", "userId": 1, "issuer": &base_url
    }).to_string()).expect("issuer-bound fixture credential");
    let client = Arc::new(
        FabushiNativeMessagingClient::new(&base_url, &credential_path)
            .expect("shipping messaging client"),
    );
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    let runtime = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_messaging_and_dependencies(
            &agents,
            500,
            Some("1".into()),
            Some(client),
            true,
            None,
            Arc::new(|| None),
            memory,
        ),
    );
    let created = runtime
        .create_human_conversation("2", "Peer")
        .expect("Human conversation");
    let conversation_id = created["id"]
        .as_str()
        .expect("conversation id")
        .to_string();
    let owner = runtime
        .open_human_conversation_db_owner(&conversation_id)
        .expect("conversation owner");
    assert!(owner
        .append_transcript_entry(&json!({
            "id": "human-server-message:11",
            "kind": "message",
            "role": "user",
            "authorKind": "human",
            "authorId": "2",
            "content": "react to me",
            "timestampMs": 1000.0,
            "delivery": "sent",
            "remoteMessageId": "11",
            "remoteCreatedAt": "2026-10-05T01:00:01Z",
            "remoteReactions": [],
            "reactions": []
        }))
        .expect("remote message"));

    let reacted = dispatch(
        &runtime,
        "reactHumanMessage",
        json!({
            "conversationId": conversation_id,
            "entryId": "human-server-message:11",
            "emoji": "👍"
        }),
    );
    assert_eq!(reacted["remoteReactions"][0]["reactedByMe"], true);
    assert_eq!(reacted["remoteReactions"][0]["count"], 2);
    assert_eq!(
        reacted["reactions"],
        json!([
            {"emoji": "👍", "by": "me"},
            {"emoji": "👍", "by": "2"}
        ])
    );

    let unreacted = dispatch(
        &runtime,
        "reactHumanMessage",
        json!({
            "conversationId": conversation_id,
            "entryId": "human-server-message:11",
            "emoji": "👍"
        }),
    );
    assert_eq!(unreacted["remoteReactions"][0]["reactedByMe"], false);
    assert_eq!(unreacted["reactions"], json!([{"emoji": "👍", "by": "2"}]));
    let transcript = dispatch(
        &runtime,
        "getHumanConversationTranscript",
        json!({"conversationId": conversation_id}),
    );
    assert_eq!(transcript[0], unreacted);

    let requests = server.join().expect("reaction fixture");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].starts_with("POST /api/social/messages/11/reactions "));
    assert!(requests[0].contains(r#""active":true"#));
    assert!(requests[1].contains(r#""active":false"#));
    assert!(requests[0].contains(r#""emoji":"👍""#));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_sync_recovers_multi_device_gap_until_known_remote_overlap() {
    let root = temp_root("human-multi-device-gap");
    let agents = root.join("agents");
    fs::create_dir_all(&root).expect("fixture root");
    let credential_path = root.join("host-credential.json");
    fs::write(
        &credential_path,
        r#"{"accessToken":"fixture-token","deviceId":"device-a","userId":1}"#,
    )
    .expect("credential");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&credential_path)
            .expect("credential metadata")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&credential_path, permissions).expect("private credential");
    }

    let (base_url, server) = spawn_remote_history_server();
    fs::write(&credential_path, serde_json::json!({
        "accessToken": "fixture-token", "deviceId": "device-a", "userId": 1, "issuer": &base_url
    }).to_string()).expect("issuer-bound fixture credential");
    let client = Arc::new(
        FabushiNativeMessagingClient::new(&base_url, &credential_path)
            .expect("shipping messaging client"),
    );
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    let runtime = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_messaging_and_dependencies(
            &agents,
            500,
            Some("1".into()),
            Some(client),
            true,
            None,
            Arc::new(|| None),
            memory,
        ),
    );
    let created = runtime
        .create_human_conversation("2", "Peer")
        .expect("Human conversation");
    let conversation_id = created["id"]
        .as_str()
        .expect("conversation id")
        .to_string();
    let owner = runtime
        .open_human_conversation_db_owner(&conversation_id)
        .expect("conversation owner");
    let anchor = json!({
        "id": "human-server-message:1",
        "kind": "message",
        "role": "user",
        "authorKind": "human",
        "authorId": "2",
        "content": "remote-1",
        "timestampMs": 1000.0,
        "delivery": "sent",
        "remoteMessageId": "1",
        "remoteCreatedAt": "1970-01-01T00:00:01Z"
    });
    assert!(owner.append_transcript_entry(&anchor).expect("anchor"));

    let transcript = runtime
        .sync_human_conversation(&conversation_id)
        .expect("bounded reconnect recovery");
    assert_eq!(transcript.len(), 401);
    assert_eq!(transcript[0]["id"], "human-server-message:1");
    assert_eq!(transcript[1]["id"], "human-server-message:2");
    assert_eq!(transcript[400]["id"], "human-server-message:401");
    assert_eq!(
        owner
            .get_metadata("lastActivityAt")
            .expect("last activity"),
        Some(json!(401000.0))
    );

    let requests = server.join().expect("messaging fixture");
    assert_eq!(requests.len(), 4);
    assert!(requests[0].starts_with("GET /api/social/messages?contactId=2&limit=200&afterId=0 "));
    assert!(requests[0].contains("&afterId=0"));
    assert!(requests[1].contains("&afterId=200"));
    assert!(requests[2].contains("&afterId=400"));
    assert!(requests[3].starts_with("GET /api/social/messages?contactId=2&limit=200 "));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}





// NMR-001: real Session workers + the existing shipping HTTP boundary.
// Each request is scripted explicitly; unexpected resource fetches fail rather
// than silently passing a string/regex-only source inspection.
enum HumanMessageFixtureReply {
    Json(u16, serde_json::Value),
    Bytes(u16, Vec<u8>),
    Disconnect,
    Truncated(u16),
    Deferred(
        Box<HumanMessageFixtureReply>,
        std::sync::mpsc::Sender<()>,
        std::sync::mpsc::Receiver<()>,
    ),
}

fn spawn_human_message_fixture(
    responses: Vec<HumanMessageFixtureReply>,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind Human message fixture");
    listener.set_nonblocking(true).expect("bound Human fixture accept");
    let address = listener.local_addr().expect("Human fixture address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        let mut deferred = Vec::new();
        for (response_index, response) in responses.into_iter().enumerate() {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(std::time::Instant::now() < deadline,
                            "Human fixture timed out waiting for response {response_index}; received {} requests",
                            requests.len());
                        thread::sleep(std::time::Duration::from_millis(5));
                    }
                    Err(error) => panic!("Human fixture accept failed for response {response_index}: {error}"),
                }
            };
            stream.set_nonblocking(false).expect("restore blocking Human fixture stream");
            stream.set_write_timeout(Some(std::time::Duration::from_secs(15)))
                .expect("bound Human fixture response write");
            requests.push(read_fixture_http_request_before(&mut stream, Some(deadline)));
            let after_id = requests.last().and_then(|request| request.split("afterId=").nth(1))
                .and_then(|tail| tail.split(|character: char| !character.is_ascii_digit()).next())
                .and_then(|value| value.parse::<u64>().ok());
            let (response, release) = match response {
                HumanMessageFixtureReply::Deferred(response, accepted, release) => {
                    accepted.send(()).expect("observe accepted deferred request");
                    (*response, Some(release))
                }
                response => (response, None),
            };
            let is_deferred = release.is_some();
            let respond = move || {
                if let Some(release) = release {
                    release.recv_timeout(std::time::Duration::from_secs(15))
                        .expect("deferred Human response must be explicitly released");
                }
                let (status, content_type, body) = match response {
                    HumanMessageFixtureReply::Json(status, mut body) => {
                        // Existing pre-cursor cases model a compatible backend response.
                        // Tests that explicitly supply null or a conflicting cursor
                        // bypass this fixture default and assert fail-closed behavior.
                        if status == 200 {
                            if let (Some(after), Some(data)) = (
                                after_id, body.get_mut("data").and_then(serde_json::Value::as_object_mut)
                            ) {
                                if let Some(messages) = data.get("messages").and_then(serde_json::Value::as_array) {
                                    let next = messages.last().and_then(|message| message.get("id"))
                                        .cloned().unwrap_or(json!(after));
                                    data.entry("historyCursor").or_insert_with(|| json!({"mode":"after-id","afterId":after}));
                                    data.entry("nextAfterId").or_insert(next);
                                }
                            }
                        }
                        (status, "application/json", body.to_string().into_bytes())
                    },
                    HumanMessageFixtureReply::Bytes(status, bytes) => (
                        status, "application/octet-stream", bytes,
                    ),
                    HumanMessageFixtureReply::Disconnect => return,
                    HumanMessageFixtureReply::Truncated(status) => {
                        write!(stream, "HTTP/1.1 {status} Incomplete\r\nContent-Type: application/json\r\nContent-Length: 100\r\nConnection: close\r\n\r\n{{}}")
                            .expect("write deliberately truncated response");
                        stream.flush().expect("flush truncated response");
                        return;
                    }
                    HumanMessageFixtureReply::Deferred(..) => panic!("nested deferred replies are unsupported"),
                };
                let reason = match status {
                    200 => "OK",
                    201 => "Created",
                    503 => "Service Unavailable",
                    _ => "Error",
                };
                write!(
                    stream,
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                ).expect("write Human response headers");
                stream.write_all(&body).expect("write Human response body");
                stream.flush().expect("flush Human fixture response");
            };
            if is_deferred {
                deferred.push(thread::spawn(respond));
            } else {
                respond();
            }
        }
        for writer in deferred {
            writer.join().expect("deferred Human response writer");
        }
        requests
    });
    (format!("http://{address}"), handle)
}

fn human_message_fixture_runtime(
    root: &std::path::Path,
    base_url: &str,
) -> Arc<ProductionSessionWorkers> {
    let agents = root.join("agents");
    fs::create_dir_all(&agents).expect("Human fixture agent root");
    let credential_path = root.join("host-credential.json");
    fs::write(&credential_path, json!({
        "accessToken": "fixture-token",
        "deviceId": "device-a",
        "userId": 1,
        "issuer": base_url,
    }).to_string()).expect("write Human fixture credential");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&credential_path).expect("credential metadata").permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&credential_path, perms).expect("private Human credential");
    }
    let client = Arc::new(
        FabushiNativeMessagingClient::new(base_url, &credential_path)
            .expect("shipping Human messaging client")
    );
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone()
        )
    );
    Arc::new(ProductionSessionWorkers::with_agents_root_identity_messaging_and_dependencies(
        &agents,
        500,
        Some("1".into()),
        Some(client),
        true,
        None,
        Arc::new(|| None),
        memory,
    ))
}

fn fixture_remote_human_message(
    id: u64,
    sender: u64,
    recipient: u64,
    text: &str,
    nonce: Option<&str>,
    attachments: serde_json::Value,
) -> serde_json::Value {
    json!({
        "id": id,
        "senderUserId": sender,
        "senderUsername": if sender == 1 { "local" } else { "peer" },
        "recipientUserId": recipient,
        "recipientUsername": if recipient == 1 { "local" } else { "peer" },
        "text": text,
        "clientRequestId": nonce,
        "createdAt": format!("2026-10-05T01:00:{:02}Z", id % 60),
        "readAt": null,
        "isOutgoing": sender == 1,
        "replyToMessageId": null,
        "attachments": attachments,
        "reactions": []
    })
}

#[test]
fn shipping_human_known_refresh_preserves_omitted_reactions_and_clears_explicit_empty() {
    let root = temp_root("human-known-refresh");
    let mut initial = fixture_remote_human_message(51, 2, 1, "canonical", None, json!([]));
    initial["reactions"] = json!([{"emoji":"👍","count":1,"reactedByMe":false}]);
    let mut partial = initial.clone();
    partial.as_object_mut().expect("message object").remove("reactions");
    partial["readAt"] = json!("2026-10-05T02:00:00Z");
    let mut clear = partial.clone();
    clear["reactions"] = json!([]);
    clear["readAt"] = json!("2026-10-05T01:30:00Z");
    let pages = [initial, partial, clear].into_iter().map(|message|
        HumanMessageFixtureReply::Json(200, json!({
            "success":true, "data":{"messages":[message]}
        }))
    ).collect::<Vec<_>>();
    let (url, server) = spawn_human_message_fixture(pages);
    let runtime = human_message_fixture_runtime(&root, &url);
    let conversation_id = runtime.create_human_conversation("2","Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    let first = runtime.sync_human_conversation(&conversation_id).expect("initial");
    assert_eq!(first.len(),1);
    assert_eq!(first[0]["reactions"],json!([{"emoji":"👍","by":"2"}]));
    let second = runtime.sync_human_conversation(&conversation_id).expect("partial refresh");
    assert_eq!(second.len(),1,"known row has stable identity");
    assert_eq!(second[0]["reactions"],first[0]["reactions"]);
    assert_eq!(second[0]["readAt"],json!("2026-10-05T02:00:00Z"));
    let third = runtime.sync_human_conversation(&conversation_id).expect("explicit removal");
    assert_eq!(third.len(),1);
    assert_eq!(third[0]["reactions"],json!([]));
    assert_eq!(third[0]["readAt"],second[0]["readAt"],"read state is monotonic");
    runtime.shutdown();
    let reopened = human_message_fixture_runtime(&root,&url);
    let owner = reopened.open_human_conversation_db_owner(&conversation_id).expect("reopen owner");
    let stored = owner.get_transcript_entries().expect("durable entries");
    assert_eq!(stored.len(),1);
    assert_eq!(stored[0]["reactions"],json!([]));
    assert_eq!(stored[0]["readAt"],second[0]["readAt"]);
    let requests = server.join().expect("fixture requests");
    assert_eq!(requests.len(),3);
    reopened.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_sync_rejects_foreign_participants_before_resources_or_overlap() {
    let root = temp_root("nmr-foreign");
    let forbidden_resource = json!([{
        "resourceId": "foreign-resource",
        "name": "private.txt",
        "contentType": "text/plain",
        "size": 4,
        "createdAt": "2026-10-05T01:00:10Z"
    }]);
    let rows = json!([
        fixture_remote_human_message(10, 1, 3, "wrong peer", Some("foreign"), forbidden_resource),
        fixture_remote_human_message(12, 2, 1, "valid independent row", Some("peer-request"), json!([])),
    ]);
    let (base_url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":rows}}))
    ]);
    let runtime = human_message_fixture_runtime(&root, &base_url);
    let id = runtime.create_human_conversation("2", "Peer")
        .expect("Human conversation")["id"].as_str().expect("id").to_owned();
    let owner = runtime.open_human_conversation_db_owner(&id).expect("conversation owner");
    let anchor = json!({
        "id":"human-server-message:10", "kind":"message","role":"user",
        "authorKind":"human","authorId":"2","content":"old anchor",
        "timestampMs": 1000.0,"delivery":"sent","remoteMessageId":"10",
        "remoteCreatedAt":"2026-10-05T01:00:10Z"
    });
    assert!(owner.append_transcript_entry(&anchor).expect("anchor"));
    let error = runtime.sync_human_conversation(&id)
        .expect_err("foreign row must not be accepted");
    assert!(error.contains("selected Human conversation"), "{error}");
    let state = owner.get_transcript_entries().expect("durable transcript");
    assert_eq!(state.len(), 2, "valid row must progress despite independent rejection");
    assert_eq!(state[0], anchor);
    assert_eq!(state[1]["id"], "human-server-message:12");
    assert_eq!(state[1]["remoteClientRequestId"], "peer-request");
    assert!(state[1].get("clientNonce").is_none());
    let requests = server.join().expect("Human fixture");
    assert_eq!(requests.len(), 1, "foreign attachment must never be fetched");
    assert!(requests[0].starts_with("GET /api/social/messages?"));
    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_sync_keeps_peer_nonce_separate_through_local_settlement_and_restart() {
    let root = temp_root("nmr-peer-nonce");
    let incoming = fixture_remote_human_message(13, 2, 1, "peer text", Some("shared-nonce"), json!([]));
    let outgoing = fixture_remote_human_message(14, 1, 2, "local text", Some("shared-nonce"), json!([]));
    let failed_dispatch = json!({"success":false,"error":"temporary dispatch failure"});
    let (base_url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(503, failed_dispatch.clone()),
        HumanMessageFixtureReply::Json(503, failed_dispatch),
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[incoming]}})),
        HumanMessageFixtureReply::Json(201, json!({"success":true,"deduplicated":false,"message":outgoing})),
    ]);
    let runtime = human_message_fixture_runtime(&root, &base_url);
    let id = runtime.create_human_conversation("2", "Peer")
        .expect("Human conversation")["id"].as_str().expect("id").to_owned();

    runtime.append_human_message(
        &id, "local text", "shared-nonce", Some(1800.0), None, &[],
    ).expect_err("failed dispatch must leave a durable local intent before peer sync");
    let pending_rows = runtime.read_human_conversation_transcript(&id).expect("pending transcript");
    assert_eq!(pending_rows.len(), 1);
    assert_eq!(pending_rows[0]["id"], "human-message:shared-nonce");
    assert_eq!(pending_rows[0]["authorId"], "1");
    assert_eq!(pending_rows[0]["clientNonce"], "shared-nonce");
    assert_eq!(pending_rows[0]["content"], "local text");
    assert_eq!(pending_rows[0]["delivery"], "pending");
    assert!(pending_rows[0].get("remoteMessageId").is_none());

    let peer_rows = runtime.sync_human_conversation(&id)
        .expect("peer sync progresses despite a separate failed local retry");
    assert_eq!(peer_rows.len(), 2, "peer provenance must not settle or replace the pending local row");
    // The retry may update its failure clock; every other pending field remains identical.
    assert!(peer_rows[0]["lastDispatchFailedAtMs"].as_f64()
        .is_some_and(|timestamp| timestamp.is_finite() && timestamp >= 0.0));
    let mut expected_pending = pending_rows[0].clone();
    expected_pending["lastDispatchFailedAtMs"] = peer_rows[0]["lastDispatchFailedAtMs"].clone();
    assert_eq!(peer_rows[0], expected_pending);
    assert_eq!(peer_rows[1]["id"], "human-server-message:13");
    assert_eq!(peer_rows[1]["authorId"], "2");
    assert_eq!(peer_rows[1]["content"], "peer text");
    assert_eq!(peer_rows[1]["remoteClientRequestId"], "shared-nonce");
    assert!(peer_rows[1].get("clientNonce").is_none());
    runtime.shutdown();
    drop(runtime);

    let restarted = human_message_fixture_runtime(&root, &base_url);
    assert_eq!(
        restarted.read_human_conversation_transcript(&id).expect("restart pending and peer"),
        peer_rows,
    );
    let sent = restarted.append_human_message(
        &id, "local text", "shared-nonce", Some(1800.0), None, &[],
    ).expect("only the matching local acknowledgement may settle the pending intent");
    assert_eq!(sent["id"], "human-server-message:14");
    assert_eq!(sent["authorId"], "1");
    assert_eq!(sent["clientNonce"], "shared-nonce");
    assert_eq!(sent["content"], "local text");
    assert_eq!(sent["delivery"], "sent");
    assert!(sent.get("lastDispatchError").is_none());
    let entries = restarted.read_human_conversation_transcript(&id).expect("settled transcript");
    assert_eq!(entries, vec![sent, peer_rows[1].clone()], "settlement retains the local slot and the peer row");
    restarted.shutdown();
    drop(restarted);

    let reopened = human_message_fixture_runtime(&root, &base_url);
    assert_eq!(reopened.read_human_conversation_transcript(&id).expect("restart settled rows"), entries);
    reopened.shutdown();
    let requests = server.join().expect("Human fixture");
    assert_eq!(requests.len(), 4);
    assert!(requests[0].starts_with("POST /api/social/messages "));
    assert!(requests[1].starts_with("POST /api/social/messages "));
    assert!(requests[2].starts_with("GET /api/social/messages?"));
    assert!(requests[3].starts_with("POST /api/social/messages "));
    assert_eq!(fixture_request_json(&requests[0]), fixture_request_json(&requests[1]));
    assert_eq!(fixture_request_json(&requests[0]), fixture_request_json(&requests[3]));
    assert_eq!(fixture_request_json(&requests[3])["clientRequestId"], "shared-nonce");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_sync_rejects_conflicting_local_acknowledgements_before_resource_access() {
    for (difference, expected_error) in [
        ("text", "remote Human acknowledgement conflicts with the local send identity or text"),
        ("reply", "remote Human acknowledgement changed the dispatched reply"),
        ("resource-id", "remote Human acknowledgement changed dispatched resources"),
        ("resource-order", "remote Human acknowledgement changed dispatched resources"),
        ("missing-resource", "remote Human acknowledgement changed dispatched resources"),
    ] {
        let root = temp_root(&format!("nmr-conflict-{difference}"));
        let dispatched = json!([
            {
                "resourceId":"expected-resource-a","name":"first.txt","contentType":"text/plain",
                "size":4,"createdAt":"2026-10-05T01:00:19Z"
            },
            {
                "resourceId":"expected-resource-b","name":"second.txt","contentType":"text/plain",
                "size":4,"createdAt":"2026-10-05T01:00:20Z"
            },
        ]);
        let mut matching_ack = fixture_remote_human_message(
            21, 1, 2, "expected text", Some("pending-nonce"), dispatched.clone(),
        );
        matching_ack["replyToMessageId"] = json!(10);
        let mut conflict = matching_ack.clone();
        // Each rejected response differs in exactly one immutable acknowledgement field.
        match difference {
            "text" => conflict["text"] = json!("wrong text"),
            "reply" => conflict["replyToMessageId"] = json!(11),
            "resource-id" => conflict["attachments"][0]["resourceId"] = json!("untrusted-conflict"),
            "resource-order" => conflict["attachments"].as_array_mut().expect("resources").reverse(),
            "missing-resource" => {
                conflict["attachments"].as_array_mut().expect("resources").pop();
            }
            _ => unreachable!(),
        }
        let (base_url, server) = spawn_human_message_fixture(vec![
            HumanMessageFixtureReply::Json(503, json!({"success":false,"error":"temporary dispatch failure"})),
            HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[conflict]}})),
            HumanMessageFixtureReply::Json(201, json!({"success":true,"deduplicated":true,"message":matching_ack})),
            HumanMessageFixtureReply::Bytes(200, vec![1, 2, 3, 4]),
            HumanMessageFixtureReply::Bytes(200, vec![5, 6, 7, 8]),
        ]);
        let runtime = human_message_fixture_runtime(&root, &base_url);
        let id = runtime.create_human_conversation("2","Peer")
            .expect("Human conversation")["id"].as_str().expect("id").to_owned();
        let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
        let anchor = json!({
            "id":"human-server-message:10","kind":"message","role":"user",
            "authorKind":"human","authorId":"2","content":"reply anchor",
            "timestampMs":500.0,"delivery":"sent","remoteMessageId":"10",
            "remoteCreatedAt":"2026-10-05T01:00:10Z"
        });
        let attachments = vec![
            json!({"path":root.join("first.txt").to_string_lossy(),"name":"first.txt"}),
            json!({"path":root.join("second.txt").to_string_lossy(),"name":"second.txt"}),
        ];
        fs::write(root.join("first.txt"), [1_u8, 2, 3, 4]).expect("first local attachment");
        fs::write(root.join("second.txt"), [5_u8, 6, 7, 8]).expect("second local attachment");
        let pending = json!({
            "id":"human-message:pending-nonce","kind":"message","role":"user",
            "authorKind":"human","authorId":"1","clientNonce":"pending-nonce",
            "content":"expected text","composedAtMs":1000.0,"timestampMs":1000.0,
            "delivery":"pending","replyToId":"human-server-message:10",
            "attachments":attachments,"remoteDispatchAttachments":dispatched
        });
        assert!(owner.append_transcript_entry(&anchor).expect("reply anchor"));
        assert!(owner.append_transcript_entry(&pending).expect("durable pending dispatch"));

        let error = runtime.sync_human_conversation(&id).expect_err("conflicting local ack");
        assert_eq!(error, expected_error, "{difference} must reach its own admission guard");
        let rows = owner.get_transcript_entries().expect("durable rows after rejection");
        assert_eq!(rows.len(), 2, "{difference} must not append a conflicting canonical row");
        assert_eq!(rows[0], anchor);
        assert!(rows[1]["lastDispatchError"].as_str().is_some());
        assert!(rows[1]["lastDispatchFailedAtMs"].as_f64()
            .is_some_and(|timestamp| timestamp.is_finite() && timestamp >= 0.0));
        let mut expected_pending = pending.clone();
        expected_pending["lastDispatchError"] = rows[1]["lastDispatchError"].clone();
        expected_pending["lastDispatchFailedAtMs"] = rows[1]["lastDispatchFailedAtMs"].clone();
        assert_eq!(rows[1], expected_pending, "{difference} must preserve the complete pending intent");
        assert!(rows[1].get("remoteMessageId").is_none());
        runtime.shutdown();
        drop(owner);
        drop(runtime);

        let restarted = human_message_fixture_runtime(&root, &base_url);
        assert_eq!(
            restarted.read_human_conversation_transcript(&id).expect("restart rejected pending"),
            rows,
            "{difference} must remain rejected durably",
        );
        let sent = restarted.append_human_message(
            &id, "expected text", "pending-nonce", Some(1000.0),
            Some("human-server-message:10"), &attachments,
        ).expect("a fully matching acknowledgement must recover the same pending intent");
        assert_eq!(sent["id"], "human-server-message:21");
        assert_eq!(sent["clientNonce"], "pending-nonce");
        assert_eq!(sent["content"], "expected text");
        assert_eq!(sent["replyToId"], "human-server-message:10");
        assert_eq!(sent["remoteReplyToMessageId"], "10");
        assert_eq!(sent["delivery"], "sent");
        assert_eq!(sent["attachments"][0]["resourceId"], "expected-resource-a");
        assert_eq!(sent["attachments"][1]["resourceId"], "expected-resource-b");
        assert_eq!(sent["attachments"].as_array().expect("settled resources").len(), 2);
        assert!(sent.get("remoteDispatchAttachments").is_none());
        assert!(sent.get("lastDispatchError").is_none());
        assert_eq!(
            fs::read(sent["attachments"][0]["path"].as_str().expect("first resource path"))
                .expect("first materialized bytes"),
            [1_u8, 2, 3, 4],
        );
        assert_eq!(
            fs::read(sent["attachments"][1]["path"].as_str().expect("second resource path"))
                .expect("second materialized bytes"),
            [5_u8, 6, 7, 8],
        );
        let settled_rows = restarted.read_human_conversation_transcript(&id).expect("recovered transcript");
        assert_eq!(settled_rows, vec![anchor, sent]);
        restarted.shutdown();
        drop(restarted);

        let reopened = human_message_fixture_runtime(&root, &base_url);
        assert_eq!(reopened.read_human_conversation_transcript(&id).expect("restart recovered rows"), settled_rows);
        reopened.shutdown();
        let requests = server.join().expect("Human fixture");
        assert_eq!(requests.len(), 5, "{difference} must be rejected before any resource download");
        assert!(requests[0].starts_with("POST /api/social/messages "));
        assert!(requests[1].starts_with("GET /api/social/messages?"));
        assert!(requests[2].starts_with("POST /api/social/messages "));
        assert_eq!(fixture_request_json(&requests[0]), fixture_request_json(&requests[2]));
        assert!(requests[3].starts_with("GET /api/social/message-resources/expected-resource-a "));
        assert!(requests[4].starts_with("GET /api/social/message-resources/expected-resource-b "));
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn shipping_human_sync_rejects_canonical_ack_duplicates_with_missing_or_changed_nonce() {
    for nonce in [None, Some("changed")] {
        let root = temp_root("nmr-duplicate-canonical-id");
        let matching_ack = fixture_remote_human_message(
            21, 1, 2, "already sent", Some("original"), json!([]),
        );
        let mut conflict = matching_ack.clone();
        conflict["clientRequestId"] = json!(nonce);
        let (base_url, server) = spawn_human_message_fixture(vec![
            HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[conflict]}})),
            HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[matching_ack]}})),
        ]);
        let runtime = human_message_fixture_runtime(&root, &base_url);
        let id = runtime.create_human_conversation("2","Peer")
            .expect("Human conversation")["id"].as_str().expect("id").to_owned();
        let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
        let settled = json!({
            "id":"human-server-message:21","kind":"message","role":"user",
            "authorKind":"human","authorId":"1","clientNonce":"original",
            "content":"already sent","timestampMs":1000.0,"delivery":"sent",
            "remoteMessageId":"21","remoteCreatedAt":"2026-10-05T01:00:21Z"
        });
        assert!(owner.append_transcript_entry(&settled).expect("settled row"));
        let error = runtime.sync_human_conversation(&id)
            .expect_err("canonical id does not excuse a nonce mismatch on its own");
        assert_eq!(error, "remote Human acknowledgement conflicts with the local send identity or text");
        assert_eq!(owner.get_transcript_entries().expect("rows"), vec![settled.clone()]);
        runtime.shutdown();
        drop(owner);
        drop(runtime);

        let restarted = human_message_fixture_runtime(&root, &base_url);
        assert_eq!(
            restarted.read_human_conversation_transcript(&id).expect("restart canonical row"),
            vec![settled.clone()],
        );
        let mut refreshed = settled;
        refreshed["authorName"] = json!("local");
        refreshed["timestampMs"] = json!(1791162021000.0);
        refreshed["reactions"] = json!([]);
        refreshed["remoteReactions"] = json!([]);
        assert_eq!(
            restarted.sync_human_conversation(&id).expect("unchanged nonce admits an authoritative refresh"),
            vec![refreshed.clone()],
        );
        restarted.shutdown();
        drop(restarted);

        let reopened = human_message_fixture_runtime(&root, &base_url);
        assert_eq!(
            reopened.read_human_conversation_transcript(&id).expect("restart refreshed canonical row"),
            vec![refreshed],
        );
        reopened.shutdown();
        let requests = server.join().expect("Human fixture");
        assert_eq!(requests.len(), 2, "nonce admission must not dispatch or fetch resources");
        assert!(requests.iter().all(|request| request.starts_with("GET /api/social/messages?")));
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn shipping_human_legacy_peer_nonce_is_safe_in_reactions_and_local_send() {
    let root = temp_root("nmr-legacy-peer");
    let outgoing = fixture_remote_human_message(24, 1, 2, "new local text", Some("old-peer-nonce"), json!([]));
    let (base_url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({
            "success":true,"messageId":11,"reactions":[{"emoji":"👍","count":1,"reactedByMe":true}]
        })),
        HumanMessageFixtureReply::Json(201, json!({"success":true,"deduplicated":false,"message":outgoing})),
    ]);
    let runtime = human_message_fixture_runtime(&root, &base_url);
    let id = runtime.create_human_conversation("2","Peer")
        .expect("Human conversation")["id"].as_str().expect("id").to_owned();
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    assert!(owner.append_transcript_entry(&json!({
        "id":"human-server-message:11","kind":"message","role":"user",
        "authorKind":"human","authorId":"2","clientNonce":"old-peer-nonce",
        "content":"historical peer text","timestampMs":1000.0,"delivery":"sent",
        "remoteMessageId":"11","remoteCreatedAt":"2026-10-05T01:00:11Z",
        "remoteReactions":[],"reactions":[]
    })).expect("legacy peer"));
    let changed = runtime.toggle_human_message_reaction(&id,"human-server-message:11","👍")
        .expect("authorized reaction");
    assert_eq!(changed["content"], "historical peer text");
    assert_eq!(changed["remoteClientRequestId"], "old-peer-nonce");
    assert!(changed.get("clientNonce").is_none());
    let sent = runtime.append_human_message(&id,"new local text","old-peer-nonce",None,None,&[])
        .expect("old peer nonce cannot shadow a new local message");
    assert_eq!(sent["id"], "human-server-message:24");
    let entries = owner.get_transcript_entries().expect("rows");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], changed);
    assert_eq!(entries[1]["clientNonce"], "old-peer-nonce");
    runtime.shutdown();
    let restarted = human_message_fixture_runtime(&root, &base_url);
    assert_eq!(restarted.read_human_conversation_transcript(&id).expect("restarted rows"), entries);
    restarted.shutdown();
    let requests = server.join().expect("Human fixture");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].starts_with("POST /api/social/messages/11/reactions "));
    assert!(requests[1].starts_with("POST /api/social/messages "));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_sync_retries_resource_gap_after_restart_before_later_overlap() {
    let root = temp_root("nmr-resource-gap");
    let attachment = json!([{
        "resourceId":"retry-resource","name":"retry.txt","contentType":"text/plain",
        "size":4,"createdAt":"2026-10-05T01:00:31Z"
    }]);
    let rows = json!([
        fixture_remote_human_message(31, 2, 1, "older with resource", None, attachment),
        fixture_remote_human_message(32, 2, 1, "newer", None, json!([])),
    ]);
    let (base_url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":rows}})),
        HumanMessageFixtureReply::Json(503, json!({"success":false,"error":"retryable resource failure"})),
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":rows}})),
        HumanMessageFixtureReply::Bytes(200, vec![5, 6, 7, 8]),
    ]);
    let runtime = human_message_fixture_runtime(&root,&base_url);
    let id = runtime.create_human_conversation("2","Peer")
        .expect("Human conversation")["id"].as_str().expect("id").to_owned();
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    assert!(runtime.sync_human_conversation(&id).is_err(), "first resource GET fails");
    assert!(owner.get_transcript_entries().expect("no later overlap").is_empty());
    runtime.shutdown();
    let restarted = human_message_fixture_runtime(&root,&base_url);
    let recovered = restarted.sync_human_conversation(&id).expect("restart retries gap");
    assert_eq!(recovered.len(), 2);
    assert_eq!(recovered[0]["remoteMessageId"], "31");
    assert_eq!(recovered[1]["remoteMessageId"], "32");
    let downloaded = recovered[0]["attachments"][0]["path"].as_str().expect("resource path");
    assert_eq!(fs::read(downloaded).expect("resource bytes"), [5, 6, 7, 8]);
    restarted.shutdown();
    let requests = server.join().expect("Human fixture");
    assert_eq!(requests.len(), 4);
    assert!(requests[0].starts_with("GET /api/social/messages?"));
    assert!(requests[1].starts_with("GET /api/social/message-resources/retry-resource "));
    assert!(requests[2].starts_with("GET /api/social/messages?"));
    assert!(requests[3].starts_with("GET /api/social/message-resources/retry-resource "));
    let _ = fs::remove_dir_all(root);
}

fn invalid_human_reaction_summaries() -> Vec<(&'static str, serde_json::Value, &'static str)> {
    let inconsistent = "remote Human reaction summary is inconsistent with direct participants";
    vec![
        ("zero-with-local", json!([{"emoji":"👍","count":0,"reactedByMe":true}]), inconsistent),
        ("two-without-local", json!([{"emoji":"👍","count":2,"reactedByMe":false}]), inconsistent),
        ("third-participant", json!([{"emoji":"👍","count":3,"reactedByMe":true}]), inconsistent),
        ("u64-max", json!([{"emoji":"👍","count":u64::MAX,"reactedByMe":true}]), inconsistent),
        ("duplicate-emoji", json!([
            {"emoji":"👍","count":1,"reactedByMe":false},
            {"emoji":"👍","count":1,"reactedByMe":true}
        ]), "remote Human reaction summary contains duplicate emoji"),
    ]
}

#[test]
fn shipping_human_reaction_summaries_preserve_allowed_states_and_exact_unicode_after_restart() {
    let root = temp_root("nmr-reaction-allowed");
    let summary = json!([
        {"emoji":"🫥","count":0,"reactedByMe":false},
        {"emoji":"❤","count":1,"reactedByMe":false},
        {"emoji":"❤️","count":1,"reactedByMe":true},
        {"emoji":"👩🏽‍💻","count":2,"reactedByMe":true},
        {"emoji":" 👍","count":1,"reactedByMe":true},
        {"emoji":"👍 ","count":1,"reactedByMe":false},
        {"emoji":"é","count":1,"reactedByMe":true},
        {"emoji":"e\u{301}","count":1,"reactedByMe":false}
    ]);
    let mut remote = fixture_remote_human_message(
        890101, 2, 1, "all direct-pair states", None, json!([]),
    );
    remote["reactions"] = summary.clone();
    let (base_url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[remote.clone()]}})),
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[remote]}})),
    ]);
    let runtime = human_message_fixture_runtime(&root, &base_url);
    let id = runtime.create_human_conversation("2", "Peer")
        .expect("Human conversation")["id"].as_str().expect("id").to_owned();
    let rows = runtime.sync_human_conversation(&id).expect("all four allowed states");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"], "human-server-message:890101");
    assert_eq!(rows[0]["remoteReactions"], summary);
    assert_eq!(rows[0]["reactions"], json!([
        {"emoji":"❤","by":"2"},
        {"emoji":"❤️","by":"me"},
        {"emoji":"👩🏽‍💻","by":"me"},
        {"emoji":"👩🏽‍💻","by":"2"},
        {"emoji":" 👍","by":"me"},
        {"emoji":"👍 ","by":"2"},
        {"emoji":"é","by":"me"},
        {"emoji":"e\u{301}","by":"2"}
    ]));
    assert_eq!(runtime.read_human_conversation_transcript(&id).expect("durable rows"), rows);
    runtime.shutdown();
    drop(runtime);

    let restarted = human_message_fixture_runtime(&root, &base_url);
    assert_eq!(restarted.read_human_conversation_transcript(&id)
        .expect("pure reopen preserves zero state and exact Unicode"), rows);
    restarted.shutdown();
    drop(restarted);
    let requests = server.join().expect("bounded Human fixture");
    assert_eq!(requests.len(), 2, "only coverage and refresh; pure reopen must not request HTTP");
    assert!(requests[0].starts_with("GET /api/social/messages?contactId=2&limit=200&afterId=0 "));
    assert!(requests[1].starts_with("GET /api/social/messages?contactId=2&limit=200 "));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_sync_rejects_invalid_reactions_before_resources_and_known_overlap() {
    for (case_index, (label, summary, expected_error)) in
        invalid_human_reaction_summaries().into_iter().enumerate()
    {
        let root = temp_root(&format!("nmr-reaction-ingress-{label}"));
        let base = 900000_u64 + case_index as u64 * 1000;
        let known_id = base + 200;
        let known_entry_id = format!("human-server-message:{known_id}");
        let forbidden_id = format!("rejected-reaction-resource-{label}");
        let forbidden_attachment = json!([{
            "resourceId": forbidden_id, "name":"must-not-download.txt",
            "contentType":"text/plain", "size":4, "createdAt":"2026-10-05T01:00:00Z"
        }]);
        let mut bad_new = fixture_remote_human_message(
            base, 2, 1, "invalid new row", None, forbidden_attachment.clone(),
        );
        bad_new["createdAt"] = json!("2026-10-05T01:00:00Z");
        bad_new["reactions"] = summary.clone();
        let mut page = vec![bad_new];
        // The endpoint's newest window is oldest-first. Keep its raw length
        // at 200 even though two rows will be rejected before overlap lookup.
        for offset in 1_u64..=198 {
            let mut valid = fixture_remote_human_message(
                base + offset, 2, 1, &format!("valid-{offset}"), None, json!([]),
            );
            valid["createdAt"] = json!(format!(
                "2026-10-05T01:{:02}:{:02}Z", offset / 60, offset % 60,
            ));
            valid["reactions"] = json!([{"emoji":"👍","count":1,"reactedByMe":false}]);
            page.push(valid);
        }
        let mut bad_known = fixture_remote_human_message(
            known_id, 2, 1, "must not overwrite anchor", None, forbidden_attachment,
        );
        bad_known["createdAt"] = json!("2026-10-05T01:03:20Z");
        bad_known["reactions"] = summary;
        page.push(bad_known);
        assert_eq!(page.len(), 200);
        let mut older = fixture_remote_human_message(
            base + 201, 2, 1, "next afterId page still required", None, json!([]),
        );
        older["createdAt"] = json!("2026-10-05T00:59:59Z");
        let (base_url, server) = spawn_human_message_fixture(vec![
            HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":page}})),
            HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[older]}})),
        ]);
        let runtime = human_message_fixture_runtime(&root, &base_url);
        let id = runtime.create_human_conversation("2", "Peer")
            .expect("Human conversation")["id"].as_str().expect("id").to_owned();
        let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
        let anchor = json!({
            "id":known_entry_id, "kind":"message", "role":"user", "authorKind":"human",
            "authorId":"2", "content":"persisted anchor", "timestampMs":1000.0,
            "delivery":"sent", "remoteMessageId":known_id.to_string(),
            "remoteCreatedAt":"2026-10-05T01:03:20Z", "retainedLocalMetadata":{"keep":label},
            "remoteReactions":[{"emoji":"❤","count":1,"reactedByMe":false}],
            "reactions":[{"emoji":"❤","by":"2"}]
        });
        assert!(owner.append_transcript_entry(&anchor).expect("durable known row"));
        assert_eq!(runtime.sync_human_conversation(&id)
            .expect_err("malformed summary must reject before overlap or resources"), expected_error, "{label}");
        let rows = owner.get_transcript_entries().expect("independent valid progress");
        assert_eq!(rows.len(), 200, "anchor, next page, and 198 valid first-page rows: {label}");
        assert_eq!(rows[0], anchor, "known invalid row must not replace any persisted field: {label}");
        assert_eq!(rows[199]["remoteMessageId"], (base + 201).to_string(), "next afterId page was not skipped: {label}");
        for (index, row) in rows[1..199].iter().enumerate() {
            let offset = index as u64 + 1;
            assert_eq!(row["remoteMessageId"], (base + offset).to_string(), "ordered valid progress: {label}");
            assert_eq!(row["content"], format!("valid-{offset}"));
            assert_eq!(row["reactions"], json!([{"emoji":"👍","by":"2"}]));
        }
        let resource_root = root.join("agents").join("human-message-resources");
        assert!(!resource_root.exists(), "rejection precedes even resource-directory creation: {label}");
        drop(owner);
        runtime.shutdown();
        drop(runtime);

        let restarted = human_message_fixture_runtime(&root, &base_url);
        assert_eq!(restarted.read_human_conversation_transcript(&id)
            .expect("pure reopen after independent progress"), rows, "{label}");
        assert!(!resource_root.join(&forbidden_id).exists());
        restarted.shutdown();
        drop(restarted);
        let requests = server.join().expect("bounded two-page fixture");
        assert_eq!(requests.len(), 2, "only the two history GETs, no resource GET: {label}");
        assert!(requests.iter().all(|request| request.starts_with("GET /api/social/messages?")), "{label}");
        assert!(requests[0].contains("&afterId=0"), "{label}");
        assert!(requests[1].contains("&afterId="), "malformed known ID must not terminate traversal: {label}");
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn shipping_human_reaction_toggle_rejects_without_mutation_and_recovers_after_restart() {
    let root = temp_root("nmr-reaction-toggle-recovery");
    let target_id = "human-server-message:890303";
    let invalid = invalid_human_reaction_summaries();
    let mut responses = invalid.iter().map(|(_, summary, _)| {
        HumanMessageFixtureReply::Json(200, json!({
            "success":true, "messageId":890303, "reactions":summary
        }))
    }).collect::<Vec<_>>();
    let valid_summary = json!([{"emoji":"👍","count":2,"reactedByMe":true}]);
    responses.push(HumanMessageFixtureReply::Json(200, json!({
        "success":true, "messageId":890303, "reactions":valid_summary
    })));
    let (base_url, server) = spawn_human_message_fixture(responses);
    let runtime = human_message_fixture_runtime(&root, &base_url);
    let id = runtime.create_human_conversation("2", "Peer")
        .expect("Human conversation")["id"].as_str().expect("id").to_owned();
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    let original = json!({
        "id":target_id, "kind":"message", "role":"user", "authorKind":"human", "authorId":"2",
        "content":"retain every field until a valid response", "timestampMs":1000.0, "delivery":"sent",
        "remoteMessageId":"890303", "remoteCreatedAt":"2026-10-05T01:00:03Z",
        "clientNonce":"legacy-peer-reaction-intent", "retainedLocalMetadata":{"keep":true},
        "remoteReactions":[{"emoji":"👍","count":1,"reactedByMe":false}],
        "reactions":[{"emoji":"👍","by":"2"}]
    });
    assert!(owner.append_transcript_entry(&original).expect("durable reaction target"));
    let events = Arc::new(std::sync::Mutex::new(Vec::<serde_json::Value>::new()));
    let captured = events.clone();
    let event_conversation = id.clone();
    let subscription = mahayana_host_runtime::transcript_mutation_events::subscribe_transcript_mutations(
        move |mutation| {
            // Human conversation IDs are shared across isolated test roots;
            // the unique target ID also fences unrelated parallel test events.
            if mutation.get("agentId").and_then(serde_json::Value::as_str) != Some(event_conversation.as_str()) {
                return;
            }
            let targets_entry = mutation.get("entryId").and_then(serde_json::Value::as_str) == Some(target_id)
                || mutation.get("entries").and_then(serde_json::Value::as_array)
                    .is_some_and(|entries| entries.iter().any(|entry| entry["id"].as_str() == Some(target_id)));
            if targets_entry {
                captured.lock().expect("event recording lock").push(serde_json::Value::Object(mutation.clone()));
            }
        },
    );
    for (label, _, expected_error) in &invalid {
        assert_eq!(runtime.toggle_human_message_reaction(&id, target_id, "👍")
            .expect_err("invalid POST summary must not settle"), *expected_error, "{label}");
        assert_eq!(owner.get_transcript_entries().expect("unchanged durable target"), vec![original.clone()], "{label}");
        assert!(events.lock().expect("recorded events").is_empty(), "no target mutation on rejected POST: {label}");
    }
    drop(owner);
    runtime.shutdown();
    drop(runtime);

    let restarted = human_message_fixture_runtime(&root, &base_url);
    assert_eq!(restarted.read_human_conversation_transcript(&id)
        .expect("pure reopen before valid recovery"), vec![original.clone()]);
    assert!(events.lock().expect("recorded events").is_empty());
    let recovered = restarted.toggle_human_message_reaction(&id, target_id, "👍")
        .expect("matching valid POST recovers after rejected summaries");
    let mut expected = original;
    let expected_object = expected.as_object_mut().expect("expected message");
    let legacy_nonce = expected_object.remove("clientNonce").expect("legacy peer provenance");
    expected_object.insert("remoteClientRequestId".into(), legacy_nonce);
    expected_object.insert("remoteReactions".into(), valid_summary);
    expected_object.insert("reactions".into(), json!([{"emoji":"👍","by":"me"},{"emoji":"👍","by":"2"}]));
    assert_eq!(recovered, expected);
    assert_eq!(restarted.read_human_conversation_transcript(&id).expect("durable recovery"), vec![expected.clone()]);
    let published = events.lock().expect("recorded events").clone();
    assert_eq!(published.len(), 1, "only the successful durable update publishes for the target");
    assert_eq!(published[0]["kind"], "entries-upserted");
    assert_eq!(published[0]["agentId"], id);
    assert_eq!(published[0]["entries"], json!([expected.clone()]));
    restarted.shutdown();
    drop(restarted);
    let reopened = human_message_fixture_runtime(&root, &base_url);
    assert_eq!(reopened.read_human_conversation_transcript(&id).expect("reopen accepted recovery"), vec![expected]);
    assert_eq!(events.lock().expect("recorded events").len(), 1, "pure reads do not republish");
    reopened.shutdown();
    drop(reopened);
    subscription.unsubscribe();

    let requests = server.join().expect("bounded reaction fixture");
    assert_eq!(requests.len(), invalid.len() + 1);
    for request in requests {
        assert!(request.starts_with("POST /api/social/messages/890303/reactions "));
        assert_eq!(fixture_request_json(&request), json!({"emoji":"👍","active":true}));
    }
    let _ = fs::remove_dir_all(root);
}


fn fixture_request_json(request: &str) -> serde_json::Value {
    request
        .split_once("\r\n\r\n")
        .and_then(|(_, body)| serde_json::from_str(body).ok())
        .unwrap_or(serde_json::Value::Null)
}

fn spawn_call_ice_server() -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ICE fixture");
    let address = listener.local_addr().expect("ICE fixture address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept ICE request");
        let request = read_fixture_http_request(&mut stream);
        let body = json!({
            "success": true,
            "iceServers": [
                {"urls": ["stun:stun.cloudflare.com:3478"]},
                {
                    "urls": [
                        "turn:turn.cloudflare.com:3478?transport=udp",
                        "turns:turn.cloudflare.com:443?transport=tcp"
                    ],
                    "username": "temporary-user",
                    "credential": "temporary-credential"
                }
            ],
            "ttlSeconds": 3600
        }).to_string();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        ).expect("write ICE response");
        stream.flush().expect("flush ICE response");
        request
    });
    (format!("http://{address}"), handle)
}

#[test]
fn shipping_call_ice_credentials_remain_host_owned_and_device_scoped() {
    let root = temp_root("shipping-call-ice");
    let agents = root.join("agents");
    fs::create_dir_all(&root).expect("fixture root");
    let credential_path = root.join("host-credential.json");
    fs::write(
        &credential_path,
        r#"{"accessToken":"fixture-token","deviceId":"device-a","userId":1}"#,
    ).expect("credential");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&credential_path)
            .expect("credential metadata")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&credential_path, permissions).expect("private credential");
    }

    let (base_url, server) = spawn_call_ice_server();
    fs::write(&credential_path, serde_json::json!({
        "accessToken": "fixture-token", "deviceId": "device-a", "userId": 1, "issuer": &base_url
    }).to_string()).expect("issuer-bound fixture credential");
    let client = Arc::new(
        FabushiNativeMessagingClient::new(&base_url, &credential_path)
            .expect("shipping call ICE client"),
    );
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    let runtime = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_messaging_and_dependencies(
            &agents,
            500,
            Some("1".into()),
            Some(client),
            true,
            None,
            Arc::new(|| None),
            memory,
        ),
    );

    let ice = dispatch(&runtime, "getCallIceServers", json!({}));
    assert_eq!(ice["ttlSeconds"], 3600);
    assert_eq!(ice["iceServers"].as_array().map(Vec::len), Some(2));
    assert_eq!(
        ice["iceServers"][1]["urls"][1],
        "turns:turn.cloudflare.com:443?transport=tcp"
    );

    let request = server.join().expect("ICE fixture");
    let request_lower = request.to_ascii_lowercase();
    assert!(request.starts_with("GET /api/social/calls/ice "));
    assert!(request_lower.contains("authorization: bearer fixture-token"));
    assert!(request_lower.contains("x-fabushi-device-id: device-a"));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

fn spawn_remote_call_server(expected_requests: usize) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind call fixture");
    let address = listener.local_addr().expect("call fixture address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        let mut call_id = String::new();
        let mut state = "invited".to_string();
        let mut generation = 0_u64;
        let mut events: Vec<serde_json::Value> = Vec::new();

        for _ in 0..expected_requests {
            let (mut stream, _) = listener.accept().expect("accept call request");
            let request = read_fixture_http_request(&mut stream);
            let request_line = request.lines().next().unwrap_or_default().to_string();
            let request_json = fixture_request_json(&request);
            requests.push(request.clone());

            let body = if request_line.starts_with("POST /api/social/calls ") {
                call_id = request_json["callId"].as_str().expect("call id body").to_string();
                json!({
                    "success": true,
                    "deduplicated": false,
                    "call": {
                        "callId": call_id,
                        "creatorUserId": 1,
                        "peerUserId": 2,
                        "creatorDeviceId": "device-a",
                        "peerDeviceId": serde_json::Value::Null,
                        "state": state,
                        "generation": generation,
                        "eventSeq": events.len(),
                        "terminalState": serde_json::Value::Null,
                        "createdAt": "2026-10-05T04:00:00Z",
                        "updatedAt": "2026-10-05T04:00:00Z"
                    }
                })
            } else if request_line.starts_with("POST /api/social/calls/") {
                let kind = request_json["kind"].as_str().expect("event kind").to_string();
                let payload = request_json["payload"].clone();
                generation = request_json["generation"].as_u64().expect("event generation");
                if kind == "transition" {
                    state = payload["state"].as_str().expect("transition state").to_string();
                }
                let seq = events.len() as u64 + 1;
                let event = json!({
                    "callId": call_id,
                    "seq": seq,
                    "generation": generation,
                    "userId": 1,
                    "deviceId": "device-a",
                    "clientEventId": request_json["clientEventId"],
                    "kind": kind,
                    "payload": payload,
                    "createdAt": format!("2026-10-05T04:00:{seq:02}Z")
                });
                events.push(event.clone());
                json!({
                    "success": true,
                    "deduplicated": false,
                    "event": event,
                    "call": {
                        "callId": call_id,
                        "creatorUserId": 1,
                        "peerUserId": 2,
                        "creatorDeviceId": "device-a",
                        "peerDeviceId": serde_json::Value::Null,
                        "state": state,
                        "generation": generation,
                        "eventSeq": seq,
                        "terminalState": serde_json::Value::Null,
                        "createdAt": "2026-10-05T04:00:00Z",
                        "updatedAt": format!("2026-10-05T04:00:{seq:02}Z")
                    }
                })
            } else if request_line.starts_with("GET /api/social/calls/") {
                let after_seq = request_line
                    .split("afterSeq=")
                    .nth(1)
                    .and_then(|tail| tail.split('&').next())
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(0);
                let visible = events
                    .iter()
                    .filter(|event| event["seq"].as_u64().unwrap_or(0) > after_seq)
                    .cloned()
                    .collect::<Vec<_>>();
                json!({
                    "success": true,
                    "call": {
                        "callId": call_id,
                        "creatorUserId": 1,
                        "peerUserId": 2,
                        "creatorDeviceId": "device-a",
                        "peerDeviceId": serde_json::Value::Null,
                        "state": state,
                        "generation": generation,
                        "eventSeq": events.len(),
                        "terminalState": serde_json::Value::Null,
                        "createdAt": "2026-10-05T04:00:00Z",
                        "updatedAt": "2026-10-05T04:00:09Z"
                    },
                    "events": visible,
                    "nextAfterSeq": events.last().and_then(|event| event["seq"].as_u64()).unwrap_or(after_seq)
                })
            } else {
                panic!("unexpected call fixture request: {request_line}");
            };
            let body = body.to_string();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            ).expect("write call response");
            stream.flush().expect("flush call response");
        }
        requests
    });
    (format!("http://{address}"), handle)
}

#[test]
fn shipping_call_session_uses_backend_ordering_then_materializes_canonical_owner() {
    let root = temp_root("shipping-call-session");
    let agents = root.join("agents");
    fs::create_dir_all(&root).expect("fixture root");
    let credential_path = root.join("host-credential.json");
    fs::write(
        &credential_path,
        r#"{"accessToken":"fixture-token","deviceId":"device-a","userId":1}"#,
    ).expect("credential");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&credential_path)
            .expect("credential metadata")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&credential_path, permissions).expect("private credential");
    }

    let (base_url, server) = spawn_remote_call_server(11);
    fs::write(&credential_path, serde_json::json!({
        "accessToken": "fixture-token", "deviceId": "device-a", "userId": 1, "issuer": &base_url
    }).to_string()).expect("issuer-bound fixture credential");
    let client = Arc::new(
        FabushiNativeMessagingClient::new(&base_url, &credential_path)
            .expect("shipping call client"),
    );
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    let runtime = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_messaging_and_dependencies(
            &agents,
            500,
            Some("1".into()),
            Some(client),
            true,
            None,
            Arc::new(|| None),
            memory,
        ),
    );
    let transport_identity = dispatch(&runtime, "getCallTransportIdentity", json!({}));
    assert_eq!(transport_identity["userId"], "1");
    assert_eq!(transport_identity["deviceId"], "device-a");

    let conversation = runtime
        .create_human_conversation("2", "Peer")
        .expect("Human conversation");
    let scope_id = conversation["id"].as_str().expect("scope id").to_string();

    let created = dispatch(
        &runtime,
        "createCallSession",
        json!({"scopeId": scope_id, "participantIds": ["1", "2"]}),
    );
    let call_id = created["id"].as_str().expect("call id").to_string();
    assert_eq!(created["state"], "invited");

    let accepted = dispatch(
        &runtime,
        "transitionCallSession",
        json!({"callId": call_id, "generation": 0, "action": "accept"}),
    );
    assert_eq!(accepted["state"], "negotiating");

    let signal = dispatch(
        &runtime,
        "sendCallSignal",
        json!({
            "callId": call_id,
            "generation": 0,
            "seq": 1,
            "senderDeviceId": "device-a",
            "kind": "offer",
            "payload": {"sdp": "remote-first"}
        }),
    );
    assert_eq!(signal["seq"], 2);
    assert_eq!(signal["payload"]["sdp"], "remote-first");

    let media = dispatch(
        &runtime,
        "updateCallMedia",
        json!({
            "callId": call_id,
            "generation": 0,
            "mediaCapabilities": {"audio": true, "video": true},
            "deviceSelection": {"microphoneId": "mic-a", "cameraId": "cam-a"}
        }),
    );
    assert_eq!(media["mediaCapabilities"]["video"], true);
    assert_eq!(media["deviceSelection"]["cameraId"], "cam-a");

    let lease = dispatch(
        &runtime,
        "getCallTransportLease",
        json!({"callId": call_id}),
    );
    assert_eq!(lease["role"], "creator");
    assert_eq!(lease["claimedDeviceId"], "device-a");
    assert_eq!(lease["isOwner"], true);
    assert_eq!(lease["claimAvailable"], false);

    let requests = server.join().expect("call fixture");
    assert_eq!(requests.len(), 11);
    assert!(requests[0].starts_with("POST /api/social/calls "));
    assert!(requests[0].contains(r#""targetUserId":"2""#));
    assert!(requests[1].starts_with("GET /api/social/calls/"));
    assert!(requests[2].starts_with("POST /api/social/calls/"));
    assert!(requests[2].contains(r#""kind":"transition""#));
    assert!(requests[4].starts_with("GET /api/social/calls/"));
    assert!(requests[5].contains(r#""kind":"signal""#));
    assert!(requests[5].contains(r#""signalKind":"offer""#));
    assert!(!requests[5].contains(r#""signalSeq""#));
    assert!(requests[8].contains(r#""kind":"media""#));
    assert!(requests[8].contains(r#""cameraId":"cam-a""#));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

fn spawn_incoming_call_discovery_server() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind incoming call fixture");
    let address = listener.local_addr().expect("incoming fixture address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        let call_id = "remote-incoming-call-1";

        let (mut list_stream, _) = listener.accept().expect("accept call discovery");
        requests.push(read_fixture_http_request(&mut list_stream));
        let list_body = json!({
            "success": true,
            "calls": [{
                "callId": call_id,
                "creatorUserId": 2,
                "peerUserId": 1,
                "state": "negotiating",
                "generation": 0,
                "eventSeq": 1,
                "terminalState": serde_json::Value::Null,
                "createdAt": "2026-10-05T04:10:00Z",
                "updatedAt": "2026-10-05T04:10:01Z"
            }]
        }).to_string();
        write!(
            list_stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            list_body.len(),
            list_body
        ).expect("write call discovery");
        list_stream.flush().expect("flush call discovery");

        let (mut replay_stream, _) = listener.accept().expect("accept incoming call replay");
        requests.push(read_fixture_http_request(&mut replay_stream));
        let replay_body = json!({
            "success": true,
            "call": {
                "callId": call_id,
                "creatorUserId": 2,
                "peerUserId": 1,
                "state": "negotiating",
                "generation": 0,
                "eventSeq": 1,
                "terminalState": serde_json::Value::Null,
                "createdAt": "2026-10-05T04:10:00Z",
                "updatedAt": "2026-10-05T04:10:01Z"
            },
            "events": [{
                "callId": call_id,
                "seq": 1,
                "generation": 0,
                "userId": 1,
                "deviceId": "device-a",
                "clientEventId": "transition:0:accept",
                "kind": "transition",
                "payload": {"action": "accept", "state": "negotiating"},
                "createdAt": "2026-10-05T04:10:01Z"
            }],
            "nextAfterSeq": 1
        }).to_string();
        write!(
            replay_stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            replay_body.len(),
            replay_body
        ).expect("write incoming call replay");
        replay_stream.flush().expect("flush incoming call replay");
        requests
    });
    (format!("http://{address}"), handle)
}

#[test]
fn shipping_call_discovery_materializes_incoming_call_into_canonical_human_scope() {
    let root = temp_root("incoming-call-discovery");
    let agents = root.join("agents");
    fs::create_dir_all(&root).expect("fixture root");
    let credential_path = root.join("host-credential.json");
    fs::write(
        &credential_path,
        r#"{"accessToken":"fixture-token","deviceId":"device-a","userId":1}"#,
    ).expect("credential");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&credential_path)
            .expect("credential metadata")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&credential_path, permissions).expect("private credential");
    }
    let (base_url, server) = spawn_incoming_call_discovery_server();
    fs::write(&credential_path, serde_json::json!({
        "accessToken": "fixture-token", "deviceId": "device-a", "userId": 1, "issuer": &base_url
    }).to_string()).expect("issuer-bound fixture credential");
    let client = Arc::new(
        FabushiNativeMessagingClient::new(&base_url, &credential_path)
            .expect("incoming call client"),
    );
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    let runtime = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_messaging_and_dependencies(
            &agents,
            500,
            Some("1".into()),
            Some(client),
            true,
            None,
            Arc::new(|| None),
            memory,
        ),
    );

    let calls = dispatch(&runtime, "syncHumanCalls", json!({}));
    assert_eq!(calls.as_array().map(Vec::len), Some(1));
    assert_eq!(calls[0]["id"], "remote-incoming-call-1");
    assert_eq!(calls[0]["creatorId"], "2");
    assert_eq!(calls[0]["state"], "negotiating");
    assert_eq!(calls[0]["participantIds"], json!(["1", "2"]));

    let conversations = dispatch(&runtime, "listHumanConversations", json!({}));
    assert_eq!(conversations.as_array().map(Vec::len), Some(1));
    assert_eq!(calls[0]["scopeId"], conversations[0]["id"]);

    let requests = server.join().expect("incoming call fixture");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].starts_with("GET /api/social/calls?limit=200 "));
    assert!(requests[1].starts_with("GET /api/social/calls/remote-incoming-call-1?afterSeq=0&limit=200 "));

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn call_session_owner_fences_lifecycle_signaling_reconnect_and_restart() {
    let root = temp_root("call-session-owner");
    let agents = root.join("agents");
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    let runtime = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
            &agents,
            500,
            Some("human-local".into()),
            Arc::new(|| None),
            memory,
        ),
    );
    let conversation = dispatch(
        &runtime,
        "createHumanConversation",
        json!({
            "localHumanId": "human-local",
            "peerHumanId": "human-peer",
            "title": "Call peer"
        }),
    );
    let scope_id = conversation["id"].as_str().expect("conversation id");

    let invalid_scope = dispatch_production_session_gateway_call(
        &runtime,
        "createCallSession",
        &json!({
            "scopeId": "not-a-canonical-human-conversation",
            "participantIds": ["human-local", "human-peer"]
        }),
    )
    .expect("handled invalid call scope")
    .expect_err("unknown scope must fail closed");
    assert!(matches!(invalid_scope, SessionGatewayError::Internal(_)));

    let created = dispatch(
        &runtime,
        "createCallSession",
        json!({
            "scopeId": scope_id,
            "participantIds": ["human-local", "human-peer"]
        }),
    );
    let call_id = created["id"].as_str().expect("call id").to_string();
    assert_eq!(created["scopeId"], scope_id);
    assert_eq!(created["state"], "invited");
    assert_eq!(created["generation"], 0);
    assert_eq!(created["signalSeq"], 0);
    assert!(runtime.call_session_store_path().expect("call db").ends_with(".call-sessions.db"));

    let ringing = dispatch(
        &runtime,
        "transitionCallSession",
        json!({"callId": call_id, "generation": 0, "action": "ring"}),
    );
    assert_eq!(ringing["state"], "ringing");

    let negotiating = dispatch(
        &runtime,
        "transitionCallSession",
        json!({"callId": call_id, "generation": 0, "action": "accept"}),
    );
    assert_eq!(negotiating["state"], "negotiating");

    let offer = dispatch(
        &runtime,
        "sendCallSignal",
        json!({
            "callId": call_id,
            "generation": 0,
            "seq": 1,
            "senderDeviceId": "device-a",
            "kind": "offer",
            "payload": {"sdp": "offer-v1"}
        }),
    );
    assert_eq!(offer["seq"], 1);
    assert_eq!(offer["kind"], "offer");

    let replay = dispatch(
        &runtime,
        "sendCallSignal",
        json!({
            "callId": call_id,
            "generation": 0,
            "seq": 1,
            "senderDeviceId": "device-a",
            "kind": "offer",
            "payload": {"sdp": "offer-v1"}
        }),
    );
    assert_eq!(replay, offer);

    let conflicting_replay = dispatch_production_session_gateway_call(
        &runtime,
        "sendCallSignal",
        &json!({
            "callId": call_id,
            "generation": 0,
            "seq": 1,
            "senderDeviceId": "device-a",
            "kind": "offer",
            "payload": {"sdp": "different"}
        }),
    )
    .expect("handled conflicting replay")
    .expect_err("conflicting replay must fail");
    assert!(matches!(
        conflicting_replay,
        SessionGatewayError::Internal(message)
            if message.contains("conflicts with previously accepted payload")
    ));

    let gap = dispatch_production_session_gateway_call(
        &runtime,
        "sendCallSignal",
        &json!({
            "callId": call_id,
            "generation": 0,
            "seq": 3,
            "senderDeviceId": "device-b",
            "kind": "answer",
            "payload": {"sdp": "answer-gap"}
        }),
    )
    .expect("handled signal gap")
    .expect_err("signal gap must fail");
    assert!(matches!(
        gap,
        SessionGatewayError::Internal(message)
            if message.contains("sequence gap")
    ));

    let answer = dispatch(
        &runtime,
        "sendCallSignal",
        json!({
            "callId": call_id,
            "generation": 0,
            "seq": 2,
            "senderDeviceId": "device-b",
            "kind": "answer",
            "payload": {"sdp": "answer-v1"}
        }),
    );
    assert_eq!(answer["seq"], 2);

    let connected = dispatch(
        &runtime,
        "transitionCallSession",
        json!({"callId": call_id, "generation": 0, "action": "connected"}),
    );
    assert_eq!(connected["state"], "connected");

    let media = dispatch(
        &runtime,
        "updateCallMedia",
        json!({
            "callId": call_id,
            "generation": 0,
            "mediaCapabilities": {"audio": true, "video": true, "screenShare": true},
            "deviceSelection": {"microphoneId": "mic-1", "cameraId": "cam-1"}
        }),
    );
    assert_eq!(media["mediaCapabilities"]["audio"], true);
    assert_eq!(media["deviceSelection"]["microphoneId"], "mic-1");

    let reconnecting = dispatch(
        &runtime,
        "transitionCallSession",
        json!({"callId": call_id, "generation": 0, "action": "reconnect"}),
    );
    assert_eq!(reconnecting["state"], "reconnecting");
    assert_eq!(reconnecting["generation"], 1);
    assert_eq!(reconnecting["signalSeq"], 0);

    let stale = dispatch_production_session_gateway_call(
        &runtime,
        "sendCallSignal",
        &json!({
            "callId": call_id,
            "generation": 0,
            "seq": 3,
            "senderDeviceId": "device-a",
            "kind": "candidate",
            "payload": {"candidate": "stale"}
        }),
    )
    .expect("handled stale generation")
    .expect_err("stale generation must fail");
    assert!(matches!(
        stale,
        SessionGatewayError::Internal(message)
            if message.contains("stale call generation")
    ));

    let resumed = dispatch(
        &runtime,
        "transitionCallSession",
        json!({"callId": call_id, "generation": 1, "action": "resume"}),
    );
    assert_eq!(resumed["state"], "negotiating");
    let candidate = dispatch(
        &runtime,
        "sendCallSignal",
        json!({
            "callId": call_id,
            "generation": 1,
            "seq": 1,
            "senderDeviceId": "device-a",
            "kind": "candidate",
            "payload": {"candidate": "fresh"}
        }),
    );
    assert_eq!(candidate["generation"], 1);
    assert_eq!(candidate["seq"], 1);

    let signals = dispatch(
        &runtime,
        "listCallSignals",
        json!({"callId": call_id, "generation": 1, "afterSeq": 0, "limit": 20}),
    );
    assert_eq!(signals.as_array().map(Vec::len), Some(1));
    assert_eq!(signals[0]["payload"]["candidate"], "fresh");

    runtime.shutdown();
    drop(runtime);

    let restarted_memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    let restarted = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
            &agents,
            500,
            Some("human-local".into()),
            Arc::new(|| None),
            restarted_memory,
        ),
    );
    let restored = dispatch(
        &restarted,
        "getCallSession",
        json!({"callId": call_id}),
    );
    assert_eq!(restored["state"], "negotiating");
    assert_eq!(restored["generation"], 1);
    assert_eq!(restored["signalSeq"], 1);

    let ended = dispatch(
        &restarted,
        "transitionCallSession",
        json!({
            "callId": call_id,
            "generation": 1,
            "action": "hangup",
            "terminalReason": "user-ended"
        }),
    );
    assert_eq!(ended["state"], "ended");
    assert_eq!(ended["terminalReason"], "user-ended");

    let after_terminal = dispatch_production_session_gateway_call(
        &restarted,
        "sendCallSignal",
        &json!({
            "callId": call_id,
            "generation": 1,
            "seq": 2,
            "senderDeviceId": "device-a",
            "kind": "candidate",
            "payload": {"candidate": "late"}
        }),
    )
    .expect("handled terminal signal")
    .expect_err("terminal signal must fail");
    assert!(matches!(
        after_terminal,
        SessionGatewayError::Internal(message)
            if message.contains("terminal call session")
    ));

    restarted.shutdown();
    drop(restarted);

    let foreign_memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    let foreign = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
            &agents,
            500,
            Some("human-foreign".into()),
            Arc::new(|| None),
            foreign_memory,
        ),
    );
    let unauthorized = dispatch_production_session_gateway_call(
        &foreign,
        "getCallSession",
        &json!({"callId": call_id}),
    )
    .expect("handled foreign call lookup")
    .expect_err("foreign identity must not read call truth");
    assert!(matches!(
        unauthorized,
        SessionGatewayError::Internal(message)
            if message.contains("authenticated Human identity")
    ));
    foreign.shutdown();
    drop(foreign);

    let _ = fs::remove_dir_all(root);
}



#[test]
fn shipping_human_after_id_checkpoint_resumes_after_restart_without_replaying_committed_page() {
    let root = temp_root("human-coverage-restart");
    let first_page = (1..=200).map(|n| fixture_remote_human_message(n, 2, 1, &format!("row-{n}"), None, json!([]))).collect::<Vec<_>>();
    let (base_url, first_server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":first_page}})),
        HumanMessageFixtureReply::Json(503, json!({"success":false,"error":"retryable disconnect"})),
    ]);
    let first = human_message_fixture_runtime(&root, &base_url);
    let id = first.create_human_conversation("2", "Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    assert!(first.sync_human_conversation(&id).is_err());
    let owner = first.open_human_conversation_db_owner(&id).expect("owner");
    assert_eq!(owner.get_transcript_entries().expect("page persisted").len(), 200);
    assert_eq!(owner.get_metadata("humanHistoryCoverage").expect("watermark"),
        Some(json!({"afterId":200,"complete":false})));
    first.shutdown();
    drop(owner);
    drop(first);
    let requests = first_server.join().expect("fixture");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].contains("&afterId=0"));
    assert!(requests[1].contains("&afterId=200"));

    // Same address and private credential, but new Host instance.
    // The second server gets its own address; replacing the issuer-bound
    // credential is intentional because only the active service origin changed.
    let (resume_url, second_server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[
            fixture_remote_human_message(201, 2, 1, "row-201", None, json!([]))
        ]}})),
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[
            fixture_remote_human_message(201, 2, 1, "row-201", None, json!([]))
        ]}})),
    ]);
    let resumed = human_message_fixture_runtime(&root, &resume_url);
    let rows = resumed.sync_human_conversation(&id).expect("resume after durable page");
    assert_eq!(rows.len(), 201);
    assert_eq!(rows[0]["id"], "human-server-message:1");
    assert_eq!(rows[200]["id"], "human-server-message:201");
    let owner = resumed.open_human_conversation_db_owner(&id).expect("owner");
    assert_eq!(owner.get_metadata("humanHistoryCoverage").expect("finished"),
        Some(json!({"afterId":201,"complete":true})));
    let requests = second_server.join().expect("resume HTTP");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].contains("&afterId=200"));
    assert!(!requests[1].contains("&afterId="), "latest snapshot refresh");
    resumed.shutdown();
    drop(owner);
    drop(resumed);
    let _ = fs::remove_dir_all(root);
}


#[test]
fn shipping_human_reaction_intent_survives_failed_http_and_replays_once_after_restart() {
    let root = temp_root("human-reaction-intent-restart");
    let (url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(503, json!({"success":false,"error":"unavailable"})),
        HumanMessageFixtureReply::Json(200, json!({
            "success":true,"messageId":777,
            "reactions":[{"emoji":"👍","count":1,"reactedByMe":true}]
        })),
    ]);
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2","Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    let entry_id = "human-server-message:777";
    let original = json!({
        "id":entry_id,"kind":"message","role":"user","authorKind":"human",
        "authorId":"2","content":"persist target","timestampMs":1000.0,
        "delivery":"sent","remoteMessageId":"777","remoteCreatedAt":"2026-10-05T01:00:17Z",
        "remoteReactions":[],"reactions":[]
    });
    assert!(owner.append_transcript_entry(&original).expect("seed"));
    assert!(runtime.toggle_human_message_reaction(&id,entry_id,"👍").is_err());
    assert_eq!(owner.get_metadata("humanReactionIntent").expect("intent"),Some(json!({
        "entryId":entry_id,"remoteMessageId":"777","emoji":"👍","active":true,"localHumanId":"1"
    })));
    assert_eq!(owner.get_transcript_entries().expect("unchanged"),vec![original]);
    runtime.shutdown(); drop(owner); drop(runtime);
    let restarted = human_message_fixture_runtime(&root, &url);
    let settled = restarted.toggle_human_message_reaction(&id,entry_id,"👍")
        .expect("retry staged desired reaction");
    assert_eq!(settled["remoteReactions"],json!([{"emoji":"👍","count":1,"reactedByMe":true}]));
    let owner = restarted.open_human_conversation_db_owner(&id).expect("reopen");
    assert_eq!(owner.get_metadata("humanReactionIntent").expect("ack"),Some(serde_json::Value::Null));
    assert_eq!(owner.get_transcript_entries().expect("recovered").len(),1);
    let requests = server.join().expect("fixture");
    assert_eq!(requests.len(),2);
    for request in requests {
        assert!(request.starts_with("POST /api/social/messages/777/reactions "));
        assert_eq!(fixture_request_json(&request),json!({"emoji":"👍","active":true}));
    }
    restarted.shutdown(); drop(owner); drop(restarted);
    let _ = fs::remove_dir_all(root);
}


#[test]
fn shipping_human_history_rejects_oversized_page_before_persisting_coverage() {
    let root = temp_root("human-oversized-history-page");
    let oversized = (1..=201_u64)
        .map(|id| fixture_remote_human_message(id, 2, 1, &format!("row-{id}"), None, json!([])))
        .collect::<Vec<_>>();
    let (url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":oversized}})),
    ]);
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2", "Peer").expect("conversation")["id"]
        .as_str().expect("conversation id").to_string();
    let error = runtime.sync_human_conversation(&id).expect_err("oversized page must fail");
    assert_eq!(error, "Fabushi Human history page exceeded the 200-message response budget");
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    assert!(owner.get_transcript_entries().expect("unchanged transcript").is_empty());
    assert_eq!(owner.get_metadata("humanHistoryCoverage").expect("checkpoint"), None);
    let requests = server.join().expect("fixture");
    assert_eq!(requests.len(), 1);
    assert!(requests[0].contains("&afterId=0"));
    runtime.shutdown(); drop(owner); drop(runtime);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_history_rejects_damaged_checkpoint_before_any_network_request() {
    let root = temp_root("human-bad-history-checkpoint");
    let (url, server) = spawn_human_message_fixture(vec![]);
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2", "Peer").expect("conversation")["id"]
        .as_str().expect("conversation id").to_string();
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    assert!(owner.set_metadata("humanHistoryCoverage", json!({"afterId":"corrupt","complete":false}))
        .expect("stage invalid checkpoint"));
    let error = runtime.sync_human_conversation(&id).expect_err("invalid checkpoint must fail");
    assert_eq!(error, "persisted Human history coverage cursor is invalid");
    assert_eq!(owner.get_metadata("humanHistoryCoverage").expect("untouched checkpoint"),
        Some(json!({"afterId":"corrupt","complete":false})));
    assert!(owner.get_transcript_entries().expect("unchanged transcript").is_empty());
    assert!(server.join().expect("no network").is_empty());
    runtime.shutdown(); drop(owner); drop(runtime);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_reaction_unconfirmed_success_keeps_intent_until_restart_replay() {
    let root = temp_root("human-unconfirmed-reaction");
    let (url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({
            "success":true,"messageId":777,"reactions":[]
        })),
        HumanMessageFixtureReply::Json(200, json!({
            "success":true,"messageId":777,
            "reactions":[{"emoji":"👍","count":1,"reactedByMe":true}]
        })),
    ]);
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2","Peer").expect("conversation")["id"]
        .as_str().expect("conversation id").to_string();
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    let entry_id = "human-server-message:777";
    let original = json!({
        "id":entry_id,"kind":"message","role":"user","authorKind":"human",
        "authorId":"2","content":"reaction target","timestampMs":1000.0,
        "delivery":"sent","remoteMessageId":"777","remoteReactions":[],"reactions":[]
    });
    assert!(owner.append_transcript_entry(&original).expect("seed"));
    let error = runtime.toggle_human_message_reaction(&id, entry_id, "👍")
        .expect_err("successful HTTP without desired state must fail closed");
    assert_eq!(error, "Fabushi Human reaction response did not confirm the desired state");
    assert_eq!(owner.get_metadata("humanReactionIntent").expect("staged intent"), Some(json!({
        "entryId":entry_id,"remoteMessageId":"777",
        "emoji":"👍","active":true,"localHumanId":"1"
    })));
    assert_eq!(owner.get_transcript_entries().expect("not falsely settled"), vec![original]);
    runtime.shutdown(); drop(owner); drop(runtime);

    let restarted = human_message_fixture_runtime(&root, &url);
    let settled = restarted.toggle_human_message_reaction(&id, entry_id, "👍")
        .expect("replay the same desired state instead of toggling off");
    assert_eq!(settled["remoteReactions"], json!([{"emoji":"👍","count":1,"reactedByMe":true}]));
    let owner = restarted.open_human_conversation_db_owner(&id).expect("owner");
    assert_eq!(owner.get_metadata("humanReactionIntent").expect("ack"), Some(serde_json::Value::Null));
    let requests = server.join().expect("fixture");
    assert_eq!(requests.len(), 2);
    for request in requests {
        assert!(request.starts_with("POST /api/social/messages/777/reactions "));
        assert_eq!(fixture_request_json(&request), json!({"emoji":"👍","active":true}));
    }
    restarted.shutdown(); drop(owner); drop(restarted);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_reaction_missing_summary_does_not_acknowledge_intent() {
    let root = temp_root("human-missing-reaction-summary");
    let (url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({"success":true,"messageId":778})),
    ]);
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2","Peer").expect("conversation")["id"]
        .as_str().expect("conversation id").to_string();
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    let entry_id = "human-server-message:778";
    let original = json!({
        "id":entry_id,"kind":"message","role":"user","authorKind":"human",
        "authorId":"2","content":"missing summary","timestampMs":1000.0,
        "delivery":"sent","remoteMessageId":"778","remoteReactions":[],"reactions":[]
    });
    assert!(owner.append_transcript_entry(&original).expect("seed"));
    let error = runtime.toggle_human_message_reaction(&id, entry_id, "👍")
        .expect_err("missing response summary must fail closed");
    assert_eq!(error, "Fabushi Human reaction response omitted the reaction summary");
    assert_eq!(owner.get_metadata("humanReactionIntent").expect("still staged"), Some(json!({
        "entryId":entry_id,"remoteMessageId":"778",
        "emoji":"👍","active":true,"localHumanId":"1"
    })));
    assert_eq!(owner.get_transcript_entries().expect("no false settlement"), vec![original]);
    assert_eq!(server.join().expect("fixture").len(), 1);
    runtime.shutdown(); drop(owner); drop(runtime);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_history_cursor_echo_rejects_legacy_and_conflicting_backend_pages() {
    for (label, cursor, next_after) in [
        ("no-cursor", serde_json::Value::Null, json!(1)),
        ("wrong-cursor", json!({"mode":"after-id","afterId":1}), json!(1)),
        ("wrong-mode", json!({"mode":"latest","afterId":0}), json!(1)),
        ("wrong-next", json!({"mode":"after-id","afterId":0}), json!(42)),
    ] {
        let root = temp_root(label);
        let row = fixture_remote_human_message(1, 2, 1, "first", None, json!([]));
        let (url, server) = spawn_human_message_fixture(vec![
            HumanMessageFixtureReply::Json(200, json!({
                "success":true,"data":{
                    "messages":[row], "historyCursor":cursor, "nextAfterId":next_after
                }
            })),
        ]);
        let runtime = human_message_fixture_runtime(&root, &url);
        let id = runtime.create_human_conversation("2", "Peer").expect("conversation")["id"]
            .as_str().expect("id").to_string();
        let failure = runtime.sync_human_conversation(&id).expect_err("untrusted page rejected");
        assert!(failure.contains("historyCursor") || failure.contains("nextAfterId"), "{label}: {failure}");
        let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
        assert!(owner.get_transcript_entries().expect("no materialization").is_empty(), "{label}");
        assert!(owner.get_metadata("humanHistoryCoverage").expect("no checkpoint").is_none(), "{label}");
        assert_eq!(server.join().expect("HTTP fixture").len(), 1, "{label}");
        runtime.shutdown();
        drop(owner);
        drop(runtime);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn shipping_human_6400_messages_use_exact_32_page_budget_and_resume_after_restart() {
    let root = temp_root("human-6400-page-budget");
    let responses = (0..32_u64).map(|page| {
        let rows = (page * 200 + 1..=(page + 1) * 200)
            .map(|id| fixture_remote_human_message(id, 2, 1, &format!("row-{id}"), None, json!([])))
            .collect::<Vec<_>>();
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":rows}}))
    }).collect::<Vec<_>>();
    let (url, first_server) = spawn_human_message_fixture(responses);
    let first = human_message_fixture_runtime(&root, &url);
    let id = first.create_human_conversation("2", "Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    let rows = first.sync_human_conversation(&id).expect("bounded first sync");
    assert_eq!(rows.len(), 6400);
    assert_eq!(rows.iter().map(|row| row["remoteMessageId"].as_str().expect("remote identity").to_string())
        .collect::<Vec<_>>(), (1..=6400).map(|id| id.to_string()).collect::<Vec<_>>(),
        "every row appears exactly once in canonical order across all 32 pages");
    let owner = first.open_human_conversation_db_owner(&id).expect("owner");
    assert_eq!(owner.get_metadata("humanHistoryCoverage").expect("checkpoint"),
        Some(json!({"afterId":6400,"complete":false})));
    first.shutdown();
    drop(owner);
    drop(first);
    let requests = first_server.join().expect("32-page fixture");
    assert_eq!(requests.len(), 32, "no 33rd probe or unbudgeted latest refresh");
    assert!(requests.first().expect("first").contains("&afterId=0"));
    assert!(requests.last().expect("last").contains("&afterId=6200"));

    let row = fixture_remote_human_message(6401, 2, 1, "row-6401", None, json!([]));
    let (resume_url, second_server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[row.clone()]}})),
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[row]}})),
    ]);
    let resumed = human_message_fixture_runtime(&root, &resume_url);
    let resumed_rows = resumed.sync_human_conversation(&id).expect("resumed cursor");
    assert_eq!(resumed_rows.len(), 6401);
    assert_eq!(&resumed_rows[..6400], rows.as_slice(), "restart preserves every previously covered row and its order");
    assert_eq!(resumed_rows[6400]["id"], "human-server-message:6401");
    let owner = resumed.open_human_conversation_db_owner(&id).expect("owner");
    assert_eq!(owner.get_metadata("humanHistoryCoverage").expect("complete checkpoint"),
        Some(json!({"afterId":6401,"complete":true})));
    let requests = second_server.join().expect("resume fixture");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].contains("&afterId=6400"), "resume must not replay first 6400");
    assert!(!requests[1].contains("&afterId="), "spare request refreshes known latest");
    resumed.shutdown();
    drop(owner);
    drop(resumed);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_concurrent_reactions_serialize_one_conversation_without_blocking_another() {
    let root = temp_root("human-concurrent-reactions");
    let (first_seen, first_wait) = std::sync::mpsc::channel();
    let (first_release, first_resume) = std::sync::mpsc::channel();
    let (second_seen, second_wait) = std::sync::mpsc::channel();
    let (second_release, second_resume) = std::sync::mpsc::channel();
    let (url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Deferred(Box::new(HumanMessageFixtureReply::Json(200, json!({
            "success":true,"messageId":777,
            "reactions":[{"emoji":"👍","count":1,"reactedByMe":true}]
        }))), first_seen, first_resume),
        HumanMessageFixtureReply::Json(200, json!({
            "success":true,"messageId":888,
            "reactions":[{"emoji":"👍","count":1,"reactedByMe":true}]
        })),
        HumanMessageFixtureReply::Deferred(Box::new(HumanMessageFixtureReply::Json(200, json!({
            "success":true,"messageId":777,"reactions":[]
        }))), second_seen, second_resume),
    ]);
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2", "Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    let other = runtime.create_human_conversation("3", "Other").expect("other conversation")["id"]
        .as_str().expect("other id").to_string();
    for (conversation, entry, remote, author) in [
        (id.as_str(), "human-server-message:777", "777", "2"),
        (other.as_str(), "human-server-message:888", "888", "3"),
    ] {
        assert!(runtime.open_human_conversation_db_owner(conversation).expect("owner")
            .append_transcript_entry(&json!({
                "id":entry,"kind":"message","role":"user","authorKind":"human",
                "authorId":author,"content":"target","timestampMs":1000.0,
                "delivery":"sent","remoteMessageId":remote,"remoteReactions":[],"reactions":[]
            })).expect("seed target"));
    }
    let first = {
        let runtime = Arc::clone(&runtime);
        let id = id.clone();
        thread::spawn(move || runtime.toggle_human_message_reaction(&id, "human-server-message:777", "👍"))
    };
    first_wait.recv_timeout(std::time::Duration::from_secs(15)).expect("first POST is held");
    let (started, waiting) = std::sync::mpsc::channel();
    let second = {
        let runtime = Arc::clone(&runtime);
        let id = id.clone();
        thread::spawn(move || {
            started.send(()).expect("second toggle started");
            runtime.toggle_human_message_reaction(&id, "human-server-message:777", "👍")
        })
    };
    waiting.recv_timeout(std::time::Duration::from_secs(15)).expect("competing toggle invoked");
    let other_settled = runtime.toggle_human_message_reaction(&other, "human-server-message:888", "👍")
        .expect("unrelated conversation progresses while first POST is held");
    assert_eq!(other_settled["reactions"], json!([{"emoji":"👍","by":"me"}]));
    assert!(matches!(second_wait.recv_timeout(std::time::Duration::from_millis(150)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)), "second same-conversation POST must wait");
    first_release.send(()).expect("release first settlement");
    let activated = first.join().expect("first thread").expect("first toggle");
    assert_eq!(activated["reactions"], json!([{"emoji":"👍","by":"me"}]));
    second_wait.recv_timeout(std::time::Duration::from_secs(15)).expect("second POST starts after first settlement");
    second_release.send(()).expect("release second settlement");
    let deactivated = second.join().expect("second thread").expect("second toggle");
    assert_eq!(deactivated["reactions"], json!([]));
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    assert_eq!(owner.get_metadata("humanReactionIntent").expect("settled intent"), Some(serde_json::Value::Null));
    let requests = server.join().expect("concurrent fixture");
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("POST /api/social/messages/777/reactions "));
    assert!(requests[1].starts_with("POST /api/social/messages/888/reactions "));
    assert!(requests[2].starts_with("POST /api/social/messages/777/reactions "));
    assert_eq!(fixture_request_json(&requests[0]), json!({"emoji":"👍","active":true}));
    assert_eq!(fixture_request_json(&requests[2]), json!({"emoji":"👍","active":false}));
    runtime.shutdown(); drop(owner); drop(runtime);
    let restarted = human_message_fixture_runtime(&root, &url);
    assert_eq!(restarted.read_human_conversation_transcript(&id).expect("durable serialized result"), vec![deactivated]);
    assert_eq!(restarted.read_human_conversation_transcript(&other).expect("durable independent result"), vec![other_settled]);
    restarted.shutdown(); drop(restarted);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_reaction_waits_for_inflight_history_before_deriving_desired_state() {
    let root = temp_root("human-history-reaction-race");
    let (history_seen, history_wait) = std::sync::mpsc::channel();
    let (history_release, history_resume) = std::sync::mpsc::channel();
    let (reaction_seen, reaction_wait) = std::sync::mpsc::channel();
    let (reaction_release, reaction_resume) = std::sync::mpsc::channel();
    let mut snapshot = fixture_remote_human_message(31, 2, 1, "history target", None, json!([]));
    snapshot["reactions"] = json!([{"emoji":"👍","count":1,"reactedByMe":true}]);
    let (url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Deferred(Box::new(HumanMessageFixtureReply::Json(200,
            json!({"success":true,"data":{"messages":[snapshot]}}))), history_seen, history_resume),
        HumanMessageFixtureReply::Deferred(Box::new(HumanMessageFixtureReply::Json(200,
            json!({"success":true,"messageId":31,"reactions":[]}))), reaction_seen, reaction_resume),
    ]);
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2", "Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    assert!(runtime.open_human_conversation_db_owner(&id).expect("owner")
        .append_transcript_entry(&json!({
            "id":"human-server-message:31","kind":"message","role":"user","authorKind":"human",
            "authorId":"2","content":"history target","timestampMs":1000.0,
            "delivery":"sent","remoteMessageId":"31","remoteReactions":[],"reactions":[]
        })).expect("seed"));
    let history = {
        let runtime = Arc::clone(&runtime);
        let id = id.clone();
        thread::spawn(move || runtime.sync_human_conversation(&id))
    };
    history_wait.recv_timeout(std::time::Duration::from_secs(15)).expect("history is in flight");
    let (started, waiting) = std::sync::mpsc::channel();
    let reaction = {
        let runtime = Arc::clone(&runtime);
        let id = id.clone();
        thread::spawn(move || {
            started.send(()).expect("reaction started");
            runtime.toggle_human_message_reaction(&id, "human-server-message:31", "👍")
        })
    };
    waiting.recv_timeout(std::time::Duration::from_secs(15)).expect("reaction invoked");
    assert!(matches!(reaction_wait.recv_timeout(std::time::Duration::from_millis(150)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)), "reaction must not overtake the pending snapshot");
    history_release.send(()).expect("release snapshot");
    let refreshed = history.join().expect("history thread").expect("history refresh");
    assert_eq!(refreshed[0]["reactions"], json!([{"emoji":"👍","by":"me"}]));
    reaction_wait.recv_timeout(std::time::Duration::from_secs(15)).expect("reaction follows snapshot");
    reaction_release.send(()).expect("release reaction");
    let settled = reaction.join().expect("reaction thread").expect("reaction settles");
    assert_eq!(settled["reactions"], json!([]));
    let requests = server.join().expect("ordered fixture");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].contains("&afterId=0"));
    assert_eq!(fixture_request_json(&requests[1]), json!({"emoji":"👍","active":false}));
    runtime.shutdown(); drop(runtime);
    let restarted = human_message_fixture_runtime(&root, &url);
    assert_eq!(restarted.read_human_conversation_transcript(&id).expect("pure reopen"), vec![settled]);
    restarted.shutdown(); drop(restarted);
    let _ = fs::remove_dir_all(root);
}

fn exercise_late_human_send_response(
    label: &str,
    late_response: HumanMessageFixtureReply,
    expected_error: Option<&str>,
) {
    let root = temp_root(label);
    let (send_seen, send_wait) = std::sync::mpsc::channel();
    let (send_release, send_resume) = std::sync::mpsc::channel();
    let mut accepted = fixture_remote_human_message(77, 1, 2, "late receipt", Some("late-send"), json!([]));
    accepted["readAt"] = json!("2026-10-05T01:05:00Z");
    accepted["reactions"] = json!([{"emoji":"👍","count":1,"reactedByMe":false}]);
    let (url, server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Deferred(Box::new(late_response), send_seen, send_resume),
        // Sync retries the same durable nonce before discovering the accepted
        // history row; the original POST remains held throughout both steps.
        HumanMessageFixtureReply::Json(200, json!({"success":true,"deduplicated":true,"message":accepted.clone()})),
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[accepted]}})),
        HumanMessageFixtureReply::Json(200, json!({
            "success":true,"messageId":77,
            "reactions":[{"emoji":"👍","count":2,"reactedByMe":true}]
        })),
    ]);
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2", "Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    let sending = {
        let runtime = Arc::clone(&runtime);
        let id = id.clone();
        thread::spawn(move || runtime.append_human_message(&id, "late receipt", "late-send", Some(1000.0), None, &[]))
    };
    send_wait.recv_timeout(std::time::Duration::from_secs(15)).expect("original POST in flight");
    let synced = runtime.sync_human_conversation(&id).expect("idempotent retry and history settle independently");
    assert_eq!(synced.len(), 1);
    assert_eq!(synced[0]["id"], "human-server-message:77");
    let reacted = runtime.toggle_human_message_reaction(&id, "human-server-message:77", "👍")
        .expect("newer reaction is durably acknowledged before original POST");
    assert_eq!(reacted["readAt"], "2026-10-05T01:05:00Z");
    assert_eq!(reacted["reactions"], json!([{"emoji":"👍","by":"me"},{"emoji":"👍","by":"2"}]));
    send_release.send(()).expect("release original response after newer settlement");
    let result = sending.join().expect("send thread");
    match expected_error {
        Some(expected) => assert!(result.expect_err("conflicting or malformed acknowledgement stays rejected").contains(expected), "{label}"),
        None => assert_eq!(result.expect("durable acknowledgement wins over late transport/state"), reacted, "{label}"),
    }
    assert_eq!(runtime.read_human_conversation_transcript(&id).expect("one settled row"), vec![reacted.clone()]);
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    assert!(owner.get_transcript_entry_by_id("human-message:late-send").expect("local intent lookup").is_none());
    assert_eq!(owner.get_metadata("humanReactionIntent").expect("reaction acknowledged"), Some(serde_json::Value::Null));
    let requests = server.join().expect("late acknowledgement fixture");
    assert_eq!(requests.len(), 4);
    assert!(requests[0].starts_with("POST /api/social/messages "));
    assert!(requests[1].starts_with("POST /api/social/messages "));
    assert_eq!(fixture_request_json(&requests[0]), fixture_request_json(&requests[1]), "retry preserves exact identity/body");
    assert!(requests[2].contains("&afterId=0"));
    assert_eq!(fixture_request_json(&requests[3]), json!({"emoji":"👍","active":true}));
    runtime.shutdown(); drop(owner); drop(runtime);
    let restarted = human_message_fixture_runtime(&root, &url);
    assert_eq!(restarted.read_human_conversation_transcript(&id).expect("restart cannot restore stale receipt"), vec![reacted]);
    restarted.shutdown(); drop(restarted);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_late_send_acknowledgement_preserves_newer_reaction_and_read_receipt() {
    let stale = fixture_remote_human_message(77, 1, 2, "late receipt", Some("late-send"), json!([]));
    exercise_late_human_send_response("human-late-valid-send",
        HumanMessageFixtureReply::Json(201, json!({"success":true,"message":stale})), None);
}

#[test]
fn shipping_human_late_send_transport_failure_returns_durable_settlement_after_restart() {
    exercise_late_human_send_response("human-late-disconnected-send", HumanMessageFixtureReply::Disconnect, None);
    exercise_late_human_send_response("human-late-truncated-success", HumanMessageFixtureReply::Truncated(200), None);
}

#[test]
fn shipping_human_late_invalid_send_acknowledgements_stay_rejected_without_rolling_back_settlement() {
    let mut wrong_nonce = fixture_remote_human_message(77, 1, 2, "late receipt", Some("other-send"), json!([]));
    let mut wrong_identity = wrong_nonce.clone();
    wrong_identity["clientRequestId"] = json!("late-send");
    wrong_identity["id"] = json!(78);
    wrong_nonce["reactions"] = json!([]);
    let mut invalid_reactions = fixture_remote_human_message(77, 1, 2, "late receipt", Some("late-send"), json!([]));
    invalid_reactions["reactions"] = json!([{"emoji":"👍","count":3,"reactedByMe":true}]);
    let mut invalid_timestamp = fixture_remote_human_message(77, 1, 2, "late receipt", Some("late-send"), json!([]));
    invalid_timestamp["createdAt"] = json!("not-a-timestamp");
    let mut invalid_receipt = fixture_remote_human_message(77, 1, 2, "late receipt", Some("late-send"), json!([]));
    invalid_receipt["readAt"] = json!("not-a-timestamp");
    for (label, response, expected) in [
        ("late-missing-message", HumanMessageFixtureReply::Json(201, json!({"success":true})), "omitted the persisted message"),
        ("late-invalid-json", HumanMessageFixtureReply::Bytes(201, b"not-json".to_vec()), "invalid JSON"),
        ("late-permission-rejection", HumanMessageFixtureReply::Json(403, json!({"success":false,"error":"denied"})), "send was rejected (HTTP 403)"),
        ("late-truncated-auth-rejection", HumanMessageFixtureReply::Truncated(401), "response could not be read"),
        ("late-truncated-permission-rejection", HumanMessageFixtureReply::Truncated(403), "response could not be read"),
        ("late-wrong-nonce", HumanMessageFixtureReply::Json(201, json!({"success":true,"message":wrong_nonce})), "mismatched persisted content"),
        ("late-wrong-identity", HumanMessageFixtureReply::Json(201, json!({"success":true,"message":wrong_identity})), "changed a settled remote identity"),
        ("late-invalid-reactions", HumanMessageFixtureReply::Json(201, json!({"success":true,"message":invalid_reactions})), "inconsistent with direct participants"),
        ("late-invalid-timestamp", HumanMessageFixtureReply::Json(201, json!({"success":true,"message":invalid_timestamp})), "timestamp is invalid"),
        ("late-invalid-receipt", HumanMessageFixtureReply::Json(201, json!({"success":true,"message":invalid_receipt})), "read timestamp is invalid"),
    ] {
        exercise_late_human_send_response(label, response, Some(expected));
    }
}

fn exercise_human_send_credential_change(
    with_attachment: bool,
    during_request: bool,
    field: &str,
    replacement: serde_json::Value,
    expected_error: &str,
) {
    let root = temp_root(&format!("human-credential-{with_attachment}-{during_request}-{field}"));
    let resource = json!({
        "resourceId":"credential-resource","name":"private.bin","contentType":"application/octet-stream",
        "size":4,"createdAt":"2026-10-05T01:00:00Z"
    });
    let remote = fixture_remote_human_message(91, 1, 2, "credential-bound send", Some("credential-send"),
        if with_attachment { json!([resource.clone()]) } else { json!([]) });
    let (seen, waiting) = std::sync::mpsc::channel();
    let (release, resume) = std::sync::mpsc::channel();
    let mut replies = Vec::new();
    if during_request {
        let first = if with_attachment {
            HumanMessageFixtureReply::Json(201, json!({"success":true,"resource":resource.clone()}))
        } else {
            HumanMessageFixtureReply::Json(201, json!({"success":true,"message":remote.clone()}))
        };
        replies.push(HumanMessageFixtureReply::Deferred(Box::new(first), seen, resume));
    }
    if with_attachment {
        replies.push(HumanMessageFixtureReply::Json(201, json!({"success":true,"resource":resource})));
    }
    replies.push(HumanMessageFixtureReply::Json(201, json!({"success":true,"message":remote})));
    if with_attachment {
        replies.push(HumanMessageFixtureReply::Bytes(200, vec![1, 2, 3, 4]));
    }
    let (url, server) = spawn_human_message_fixture(replies);
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2", "Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    let attachment_path = root.join("private.bin");
    fs::write(&attachment_path, [1, 2, 3, 4]).expect("attachment fixture");
    let attachments = if with_attachment {
        vec![json!({"path":attachment_path.to_string_lossy(),"name":"private.bin"})]
    } else { vec![] };
    let change_credentials = || {
        let path = root.join("host-credential.json");
        let mut credentials: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).expect("read private fixture credential"))
            .expect("credential JSON");
        credentials[field] = replacement.clone();
        fs::write(path, credentials.to_string()).expect("replace same-issuer credential");
    };
    let failure = if during_request {
        let sending = {
            let runtime = Arc::clone(&runtime);
            let id = id.clone();
            let attachments = attachments.clone();
            thread::spawn(move || runtime.append_human_message(&id, "credential-bound send", "credential-send", Some(1000.0), None, &attachments))
        };
        waiting.recv_timeout(std::time::Duration::from_secs(15)).expect("request accepted under original credential");
        change_credentials();
        release.send(()).expect("release response after credential change");
        sending.join().expect("send thread").expect_err("changed credential response is rejected")
    } else {
        change_credentials();
        runtime.append_human_message(&id, "credential-bound send", "credential-send", Some(1000.0), None, &attachments)
            .expect_err("changed-account or invalid credential is rejected before dispatch")
    };
    assert!(failure.contains(expected_error), "{failure}");
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    let pending = owner.get_transcript_entries().expect("recoverable outbox");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["id"], "human-message:credential-send");
    assert_eq!(pending[0]["delivery"], "pending");
    assert!(pending[0].get("remoteMessageId").is_none());
    assert!(pending[0].get("remoteDispatchAttachments").is_none(), "foreign/stale upload response is not reused");
    runtime.shutdown(); drop(owner); drop(runtime);

    // Reopen restores the original private account fixture. Retry keeps the
    // same durable nonce and content; no stale response supplied a settlement.
    let resumed = human_message_fixture_runtime(&root, &url);
    let settled = resumed.append_human_message(&id, "credential-bound send", "credential-send", Some(1000.0), None, &attachments)
        .expect("original account recovers pending send");
    assert_eq!(settled["id"], "human-server-message:91");
    assert_eq!(settled["delivery"], "sent");
    let requests = server.join().expect("credential boundary fixture");
    assert_eq!(requests.len(), (if with_attachment { 3 } else { 1 }) + usize::from(during_request));
    let message_posts = requests.iter().filter(|request| request.starts_with("POST /api/social/messages ")).collect::<Vec<_>>();
    assert_eq!(message_posts.len(), if during_request && !with_attachment { 2 } else { 1 });
    assert!(message_posts.iter().all(|request| fixture_request_json(request)["clientRequestId"] == "credential-send"));
    assert_eq!(requests.iter().filter(|request| request.starts_with("POST /api/social/message-resources ")).count(),
        if with_attachment { 1 + usize::from(during_request) } else { 0 }, "pre-dispatch account rejection sends no bytes");
    resumed.shutdown(); drop(resumed);
    let reopened = human_message_fixture_runtime(&root, &url);
    assert_eq!(reopened.read_human_conversation_transcript(&id).expect("pure reopen"), vec![settled]);
    reopened.shutdown(); drop(reopened);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_send_rejects_changed_account_before_message_or_resource_dispatch() {
    for with_attachment in [false, true] {
        exercise_human_send_credential_change(with_attachment, false, "userId", json!(3), "changed Human account");
    }
    exercise_human_send_credential_change(false, false, "accessToken", json!("invalid\nheader"), "send failed");
}

#[test]
fn shipping_human_send_rejects_midflight_credential_changes_and_recovers_after_restart() {
    for with_attachment in [false, true] {
        for (field, replacement) in [("userId", json!(3)), ("deviceId", json!("device-b")), ("accessToken", json!("replacement-token"))] {
            exercise_human_send_credential_change(with_attachment, true, field, replacement, "changed during Human request");
        }
    }
}

#[test]
fn shipping_human_read_receipts_compare_rfc3339_instants_and_reject_invalid_ingress_before_resources() {
    let root = temp_root("human-read-receipt-instants");
    let make_row = |id, read_at| {
        let mut row = fixture_remote_human_message(id, 2, 1, "receipt target", None, json!([]));
        row["readAt"] = read_at;
        row
    };
    let initial = "2026-10-05T00:20:00.500Z";
    let advanced = "2026-10-04T20:20:00.750-04:00";
    let precise = "2026-10-05T00:20:00.750000001Z";
    let repaired = "2026-10-05T00:30:00.250Z";
    let mut rows = vec![
        make_row(81, json!(initial)),
        make_row(81, json!("2026-10-05T01:10:00+01:00")),
        make_row(81, json!("2026-10-05T01:20:00.5000+01:00")),
        make_row(81, json!(advanced)),
        make_row(81, json!(precise)),
        make_row(81, serde_json::Value::Null),
    ];
    for id in [81, 82] {
        let mut invalid = make_row(id, json!("not-rfc3339"));
        invalid["attachments"] = json!([{
            "resourceId":"forbidden-invalid-receipt-resource","name":"private.bin",
            "contentType":"application/octet-stream","size":4,
            "createdAt":"2026-10-05T01:00:00Z"
        }]);
        rows.push(invalid);
    }
    rows.push(make_row(81, serde_json::Value::Null));
    rows.push(make_row(81, json!(repaired)));
    let (url, server) = spawn_human_message_fixture(rows.into_iter().map(|row| {
        HumanMessageFixtureReply::Json(200, json!({"success":true,"data":{"messages":[row]}}))
    }).collect());
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2", "Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    for (stage, expected) in [("initial", initial), ("older offset", initial),
        ("equivalent fractional instant", initial), ("newer offset", advanced),
        ("nanosecond advancement", precise), ("omitted", precise)]
    {
        let current = runtime.sync_human_conversation(&id).expect(stage);
        assert_eq!(current.len(), 1, "{stage}");
        assert_eq!(current[0]["id"], "human-server-message:81", "{stage}");
        assert_eq!(current[0]["readAt"], expected, "{stage}");
    }
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    let prior = owner.get_transcript_entries().expect("prior receipt");
    let coverage = owner.get_metadata("humanHistoryCoverage").expect("prior coverage");
    for stage in ["known row", "new row"] {
        let error = runtime.sync_human_conversation(&id).expect_err("invalid receipt fails before resource access");
        assert!(error.contains("read timestamp is invalid"), "{stage}: {error}");
        assert_eq!(owner.get_transcript_entries().expect("no durable mutation"), prior, "{stage}");
        assert_eq!(owner.get_metadata("humanHistoryCoverage").expect("no coverage advance"), coverage, "{stage}");
        assert!(!root.join("agents").join("human-message-resources").exists(), "{stage}");
    }
    let mut legacy = prior[0].clone();
    legacy["readAt"] = json!("legacy-invalid-receipt");
    owner.update_transcript_entry("human-server-message:81", &legacy).expect("legacy fixture").expect("target");
    assert_eq!(runtime.sync_human_conversation(&id).expect("omitted field preserves legacy value"), vec![legacy.clone()]);
    runtime.shutdown(); drop(owner); drop(runtime);
    let restarted = human_message_fixture_runtime(&root, &url);
    assert_eq!(restarted.read_human_conversation_transcript(&id).expect("reopen preserves omitted legacy receipt"), vec![legacy]);
    let repaired_rows = restarted.sync_human_conversation(&id).expect("valid receipt repairs legacy invalid value");
    assert_eq!(repaired_rows.len(), 1);
    assert_eq!(repaired_rows[0]["readAt"], repaired);
    let requests = server.join().expect("receipt fixture");
    assert_eq!(requests.len(), 10);
    assert!(requests.iter().all(|request| request.starts_with("GET /api/social/messages?contactId=2&limit=200&afterId=0 ")),
        "invalid timestamps never initiate resource requests");
    restarted.shutdown(); drop(restarted);
    let reopened = human_message_fixture_runtime(&root, &url);
    assert_eq!(reopened.read_human_conversation_transcript(&id).expect("pure reopen of repaired receipt"), repaired_rows);
    reopened.shutdown(); drop(reopened);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_reaction_rejects_midflight_credential_rotation_and_recovers_after_restart() {
    // The account-bound HTTP request is already dispatched when the private
    // credential rotates. A late 200 response must not settle the old intent.
    for (label, changed) in [
        ("token", json!({"accessToken":"replacement-token"})),
        ("device", json!({"deviceId":"replacement-device"})),
        ("account", json!({"userId":99})),
        ("session", json!({"sessionId":"replacement-session"})),
        ("expiry", json!({"expiresAtMs":12345})),
        ("issuer", json!({"issuer":"https://other.example"})),
    ] {
        let root = temp_root(&format!("human-reaction-credential-rotation-{label}"));
        let (request_seen, wait_for_request) = std::sync::mpsc::channel();
        let (release_response, wait_for_release) = std::sync::mpsc::channel();
        let successful = json!({
            "success":true,"messageId":777,
            "reactions":[{"emoji":"👍","count":1,"reactedByMe":true}]
        });
        let (url, server) = spawn_human_message_fixture(vec![
            HumanMessageFixtureReply::Deferred(
                Box::new(HumanMessageFixtureReply::Json(200, successful.clone())),
                request_seen, wait_for_release,
            ),
            HumanMessageFixtureReply::Json(200, successful),
        ]);
        let runtime = human_message_fixture_runtime(&root, &url);
        let id = runtime.create_human_conversation("2", "Peer")
            .expect("conversation")["id"].as_str().expect("id").to_string();
        let entry_id = "human-server-message:777";
        let original = json!({
            "id":entry_id,"kind":"message","role":"user","authorKind":"human",
            "authorId":"2","content":"retain on credential rotation","timestampMs":1000.0,
            "delivery":"sent","remoteMessageId":"777","remoteReactions":[],"reactions":[]
        });
        let owner = runtime.open_human_conversation_db_owner(&id).expect("durable owner");
        assert!(owner.append_transcript_entry(&original).expect("seed"));
        let running = {
            let runtime = Arc::clone(&runtime);
            let id = id.clone();
            thread::spawn(move || runtime.toggle_human_message_reaction(&id, entry_id, "👍"))
        };
        wait_for_request.recv_timeout(std::time::Duration::from_secs(15))
            .expect("old credential POST is in flight");
        let credential_path = root.join("host-credential.json");
        let mut credential: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&credential_path).expect("original private credential")
        ).expect("credential JSON");
        for (key, value) in changed.as_object().expect("credential patch") {
            credential[key] = value.clone();
        }
        fs::write(&credential_path, credential.to_string()).expect("rotate credential");
        release_response.send(()).expect("release obsolete success response");
        let error = running.join().expect("reaction thread")
            .expect_err("obsolete response must not commit");
        if label == "issuer" {
            assert_eq!(
                error,
                "Fabushi messaging API must use an approved Native issuer origin.",
                "{label}",
            );
        } else {
            assert!(error.contains("credential"), "{label}: {error}");
        }
        assert_eq!(owner.get_transcript_entries().expect("no stale projection"), vec![original.clone()], "{label}");
        assert_eq!(owner.get_metadata("humanReactionIntent").expect("retained intent"), Some(json!({
            "entryId":entry_id,"remoteMessageId":"777",
            "emoji":"👍","active":true,"localHumanId":"1"
        })), "{label}");
        runtime.shutdown();
        drop(owner);
        drop(runtime);

        // Restoring the same account's valid credential after restart must
        // retry the desired state, never toggle it back or duplicate the row.
        let resumed = human_message_fixture_runtime(&root, &url);
        let settled = resumed.toggle_human_message_reaction(&id, entry_id, "👍")
            .expect("restart replays durable intent");
        assert_eq!(settled["remoteReactions"], json!([
            {"emoji":"👍","count":1,"reactedByMe":true}
        ]), "{label}");
        let owner = resumed.open_human_conversation_db_owner(&id).expect("reopen");
        assert_eq!(owner.get_metadata("humanReactionIntent").expect("ack"), Some(serde_json::Value::Null), "{label}");
        assert_eq!(owner.get_transcript_entries().expect("single settled entry"), vec![settled], "{label}");
        let requests = server.join().expect("both authenticated POSTs");
        assert_eq!(requests.len(), 2, "{label}");
        for request in requests {
            assert!(request.starts_with("POST /api/social/messages/777/reactions "), "{label}");
            assert_eq!(fixture_request_json(&request), json!({"emoji":"👍","active":true}), "{label}");
        }
        resumed.shutdown();
        drop(owner);
        drop(resumed);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn shipping_human_reaction_rejects_credential_change_while_response_body_is_streaming() {
    let root = temp_root("human-reaction-credential-body-race");
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind streaming reaction fixture");
    let url = format!("http://{}", listener.local_addr().expect("bound address"));
    let (body_started, wait_for_body) = std::sync::mpsc::channel();
    let (release_body, wait_for_release) = std::sync::mpsc::channel();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept reaction POST");
        let request = read_fixture_http_request(&mut stream);
        let body = json!({
            "success":true,"messageId":778,
            "reactions":[{"emoji":"👍","count":1,"reactedByMe":true}]
        }).to_string();
        write!(stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        ).expect("write headers");
        stream.write_all(&body.as_bytes()[..1]).expect("partial response body");
        stream.flush().expect("send headers and first body byte");
        body_started.send(()).expect("notify response is streaming");
        wait_for_release.recv_timeout(std::time::Duration::from_secs(15))
            .expect("explicitly release final response body");
        stream.write_all(&body.as_bytes()[1..]).expect("remaining body");
        stream.flush().expect("flush response");
        request
    });
    let runtime = human_message_fixture_runtime(&root, &url);
    let id = runtime.create_human_conversation("2","Peer").expect("conversation")["id"]
        .as_str().expect("id").to_string();
    let entry_id = "human-server-message:778";
    let original = json!({
        "id":entry_id,"kind":"message","role":"user","authorKind":"human",
        "authorId":"2","content":"body-read race","timestampMs":1000.0,
        "delivery":"sent","remoteMessageId":"778","remoteReactions":[],"reactions":[]
    });
    let owner = runtime.open_human_conversation_db_owner(&id).expect("owner");
    assert!(owner.append_transcript_entry(&original).expect("seed"));
    let running = {
        let runtime = Arc::clone(&runtime);
        let id = id.clone();
        thread::spawn(move || runtime.toggle_human_message_reaction(&id, entry_id, "👍"))
    };
    wait_for_body.recv_timeout(std::time::Duration::from_secs(15))
        .expect("HTTP headers received while body is incomplete");
    let credential_path = root.join("host-credential.json");
    let mut credential: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&credential_path).expect("private credential")
    ).expect("credential JSON");
    credential["accessToken"] = json!("rotated-during-body");
    fs::write(&credential_path, credential.to_string()).expect("rotate during body");
    release_body.send(()).expect("release body");
    let error = running.join().expect("reaction thread").expect_err("stale body response");
    assert!(error.contains("credential"), "{error}");
    assert_eq!(owner.get_transcript_entries().expect("unmodified target"), vec![original]);
    assert_eq!(owner.get_metadata("humanReactionIntent").expect("pending intent").expect("staged intent")["active"], true);
    assert!(server.join().expect("old POST").starts_with("POST /api/social/messages/778/reactions "));
    runtime.shutdown();
    drop(owner);
    drop(runtime);

    let (recovered_url, replay_server) = spawn_human_message_fixture(vec![
        HumanMessageFixtureReply::Json(200, json!({
            "success":true,"messageId":778,
            "reactions":[{"emoji":"👍","count":1,"reactedByMe":true}]
        }))
    ]);
    let resumed = human_message_fixture_runtime(&root, &recovered_url);
    let recovered = resumed.toggle_human_message_reaction(&id, entry_id, "👍")
        .expect("durable desired state replayed");
    assert_eq!(recovered["reactions"], json!([{"emoji":"👍","by":"me"}]));
    assert_eq!(replay_server.join().expect("single replay").len(), 1);
    resumed.shutdown();
    drop(resumed);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_human_send_fences_credential_rotation_while_waiting_to_settle_after_http() {
    for (label, first_reply) in [
        ("accepted", HumanMessageFixtureReply::Json(201, json!({
            "success":true,"message":fixture_remote_human_message(
                96, 1, 2, "post-lock lease", Some("post-lock-lease"), json!([])
            )
        }))),
        ("disconnected", HumanMessageFixtureReply::Disconnect),
    ] {
        let root = temp_root(&format!("human-post-lock-credential-{label}"));
        let (send_seen, send_wait) = std::sync::mpsc::channel();
        let (send_release, send_resume) = std::sync::mpsc::channel();
        let (retry_seen, retry_wait) = std::sync::mpsc::channel();
        let (retry_release, retry_resume) = std::sync::mpsc::channel();
        let (url, server) = spawn_human_message_fixture(vec![
            HumanMessageFixtureReply::Deferred(Box::new(first_reply), send_seen, send_resume),
            // Sync owns the Session mutation lock during this pending-send retry.
            HumanMessageFixtureReply::Deferred(
                Box::new(HumanMessageFixtureReply::Json(503, json!({
                    "success":false,"error":"temporary backend failure"
                }))),
                retry_seen, retry_resume,
            ),
            HumanMessageFixtureReply::Json(200, json!({
                "success":true,"data":{"messages":[]}
            })),
            HumanMessageFixtureReply::Json(201, json!({
                "success":true,"message":fixture_remote_human_message(
                    96, 1, 2, "post-lock lease", Some("post-lock-lease"), json!([])
                )
            })),
        ]);
        let runtime = human_message_fixture_runtime(&root, &url);
        let conversation_id = runtime.create_human_conversation("2", "Peer")
            .expect("conversation")["id"].as_str().expect("id").to_string();
        let sending = {
            let runtime = Arc::clone(&runtime);
            let id = conversation_id.clone();
            thread::spawn(move || runtime.append_human_message(
                &id, "post-lock lease", "post-lock-lease", Some(1000.0), None, &[]
            ))
        };
        send_wait.recv_timeout(std::time::Duration::from_secs(15))
            .expect("original HTTP request accepted");
        let syncing = {
            let runtime = Arc::clone(&runtime);
            let id = conversation_id.clone();
            thread::spawn(move || runtime.sync_human_conversation(&id))
        };
        retry_wait.recv_timeout(std::time::Duration::from_secs(15))
            .expect("sync holds the Session lock for retry");
        send_release.send(()).expect("release older HTTP response");
        // This integration case proves pending-state and restart recovery
        // regardless of whether transport or settlement observes rotation.
        // The private shipping_human_settlement_credential_ tests use an
        // explicit HTTP-completion barrier to prove the post-lock boundary.
        let path = root.join("host-credential.json");
        let mut credential: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&path).expect("private credential")
        ).expect("credential JSON");
        credential["accessToken"] = json!("rotated-after-http");
        let replacement_path = root.join("host-credential-rotated.json");
        fs::write(&replacement_path, credential.to_string()).expect("write complete rotated credential");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&replacement_path, fs::Permissions::from_mode(0o600))
                .expect("private rotated credential");
        }
        fs::rename(&replacement_path, &path).expect("atomically rotate same-user bearer");
        retry_release.send(()).expect("release sync lock");
        syncing.join().expect("sync thread").expect("independent empty coverage");
        let error = sending.join().expect("original send")
            .expect_err("obsolete response cannot settle");
        assert!(matches!(error.as_str(),
            "Fabushi native messaging credential changed during Human request"
                | "Fabushi native messaging credential changed before Human settlement"
        ), "{label}: {error}");
        let owner = runtime.open_human_conversation_db_owner(&conversation_id).expect("owner");
        let entries = owner.get_transcript_entries().expect("durable outbox");
        assert_eq!(entries.len(), 1, "{label}");
        assert_eq!(entries[0]["id"], "human-message:post-lock-lease", "{label}");
        assert_eq!(entries[0]["delivery"], "pending", "{label}");
        assert!(entries[0].get("remoteMessageId").is_none(), "{label}");
        runtime.shutdown(); drop(owner); drop(runtime);

        // A fresh Host restores its original account fixture and retries the
        // persisted client nonce, never reusing the stale response.
        let resumed = human_message_fixture_runtime(&root, &url);
        let settled = resumed.append_human_message(
            &conversation_id, "post-lock lease", "post-lock-lease", Some(1000.0), None, &[]
        ).expect("replay from stable account");
        assert_eq!(settled["id"], "human-server-message:96", "{label}");
        let requests = server.join().expect("scripted fixture");
        assert_eq!(requests.len(), 4, "{label}");
        assert!(requests[0].starts_with("POST /api/social/messages "), "{label}");
        assert!(requests[1].starts_with("POST /api/social/messages "), "{label}");
        assert!(requests[2].contains("&afterId=0"), "{label}");
        assert!(requests[3].starts_with("POST /api/social/messages "), "{label}");
        for index in [0, 1, 3] {
            assert_eq!(
                fixture_request_json(&requests[index])["clientRequestId"],
                "post-lock-lease", "{label}"
            );
        }
        resumed.shutdown(); drop(resumed);
        let _ = fs::remove_dir_all(root);
    }
}

// NMR-47: these integration cases use the shipping gateway, an independently
// opened file-backed SQLite database and an actual child test process. The
// server accepts the first POST before releasing its response, making process
// termination and durable-progress failure deterministic rather than timed.
#[derive(Default)]
struct ForwardHttpState {
    requests: Vec<String>,
    accepted: std::collections::BTreeMap<String, serde_json::Value>,
    resources: std::collections::BTreeMap<String, (serde_json::Value, Vec<u8>)>,
    uploaded_files: Vec<(String, Vec<u8>)>,
    observe_source: Option<(std::path::PathBuf, String)>,
    first_upload_snapshot: Option<(serde_json::Value, Vec<Vec<u8>>)>,
    mutate_second_pin_after_first_upload: bool,
    mutated_second_pin: Option<(std::path::PathBuf, Vec<u8>)>,
    mutation_observed_by_second_upload: bool,
    mutation_restored_before_message_ack: bool,
}

// Keep multipart payloads binary. The older text helper is sufficient for JSON
// contracts but its UTF-8 replacement would destroy this test's byte oracle.
fn read_forward_http_bytes(stream: &mut std::net::TcpStream) -> Vec<u8> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut expected = None;
    loop {
        let remaining = deadline.checked_duration_since(std::time::Instant::now())
            .filter(|duration| !duration.is_zero()).expect("forward HTTP total deadline");
        stream.set_read_timeout(Some(remaining)).expect("bounded binary request");
        let count = stream.read(&mut chunk).expect("read binary forward request");
        assert_ne!(count, 0, "forward client closed an incomplete request");
        bytes.extend_from_slice(&chunk[..count]);
        if expected.is_none() {
            if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                let headers = std::str::from_utf8(&bytes[..end]).expect("HTTP headers are text");
                assert!(!headers.to_ascii_lowercase().contains("transfer-encoding: chunked"),
                    "the bytes-backed multipart client must provide a bounded Content-Length");
                let length = headers.lines().find_map(|line| line.split_once(':')
                    .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                    .map(|(_, value)| value.trim().parse::<usize>().expect("request length")))
                    .unwrap_or(0);
                assert!(length <= 32 * 1024 * 1024 + 64 * 1024, "fixture body budget");
                expected = Some(end + 4 + length);
            } else {
                assert!(bytes.len() <= 64 * 1024, "fixture header budget");
            }
        }
        if let Some(expected) = expected {
            if bytes.len() >= expected {
                assert_eq!(bytes.len(), expected, "one complete request per connection");
                return bytes;
            }
        }
    }
}

fn forward_multipart_file(request: &[u8]) -> (String, Vec<u8>) {
    let header_end = request.windows(4).position(|part| part == b"\r\n\r\n")
        .expect("multipart HTTP headers");
    let headers = std::str::from_utf8(&request[..header_end]).expect("multipart headers");
    let content_type = headers.lines().find_map(|line| line.split_once(':')
        .filter(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.trim())).expect("multipart content type");
    assert!(content_type.starts_with("multipart/form-data;"));
    let boundary = content_type.split(';').find_map(|value|
        value.trim().strip_prefix("boundary=")).expect("multipart boundary").trim_matches('"');
    let body = &request[header_end + 4..];
    let prefix = format!("--{boundary}\r\n");
    let suffix = format!("\r\n--{boundary}--\r\n");
    assert!(body.starts_with(prefix.as_bytes()));
    assert!(body.ends_with(suffix.as_bytes()), "single file part has an exact closing boundary");
    let part_end = body.windows(4).position(|part| part == b"\r\n\r\n")
        .expect("multipart file headers");
    let part_headers = std::str::from_utf8(&body[prefix.len()..part_end]).expect("file headers");
    assert!(part_headers.contains("name=\"file\""));
    let name = part_headers.split("filename=\"").nth(1)
        .and_then(|tail| tail.split('"').next()).expect("file name").to_string();
    (name, body[part_end + 4..body.len() - suffix.len()].to_vec())
}

struct ForwardHttpFixture {
    url: String,
    state: Arc<std::sync::Mutex<ForwardHttpState>>,
    stop: Arc<std::sync::atomic::AtomicBool>,
    first_post: std::sync::mpsc::Receiver<()>,
    release: Option<std::sync::mpsc::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl ForwardHttpFixture {
    fn start(hold_first_post: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("forward listener");
        listener.set_nonblocking(true).expect("bounded forward listener");
        let url = format!("http://{}", listener.local_addr().expect("forward address"));
        let state = Arc::new(std::sync::Mutex::new(ForwardHttpState::default()));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (seen, first_post) = std::sync::mpsc::channel();
        let (release, released) = std::sync::mpsc::channel();
        let thread_state = Arc::clone(&state);
        let thread_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let mut post_count = 0;
            while !thread_stop.load(std::sync::atomic::Ordering::Acquire) {
                let mut stream = match listener.accept() {
                    Ok((stream, _)) => stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        // Accept polling is bounded by the fixture's owned stop
                        // flag; every behavior boundary uses a channel or return.
                        thread::sleep(std::time::Duration::from_millis(2));
                        continue;
                    }
                    Err(error) => panic!("forward accept: {error}"),
                };
                stream.set_nonblocking(false).expect("blocking forward stream");
                stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))
                    .expect("bounded forward write");
                let request_bytes = read_forward_http_bytes(&mut stream);
                let request = String::from_utf8_lossy(&request_bytes).into_owned();
                assert!(request.to_ascii_lowercase().contains("authorization: bearer fixture-token"));
                let is_post = request.starts_with("POST /api/social/messages ");
                let (body, content_type, status) = {
                    let mut state = thread_state.lock().expect("forward HTTP state");
                    state.requests.push(request.clone());
                    if request.starts_with("POST /api/social/message-resources ") {
                        if state.uploaded_files.len() == 1 {
                            if let Some((path, original)) = state.mutated_second_pin.as_ref() {
                                assert_eq!(fs::read(path).expect("pin remains altered during the next upload"),
                                    vec![b'X'; original.len()]);
                                state.mutation_observed_by_second_upload = true;
                            }
                        }
                        if state.uploaded_files.is_empty() {
                            if let Some((root, source)) = state.observe_source.clone() {
                                // Observe committed SQLite and the complete byte set
                                // before even the first resource upload is acknowledged.
                                let metadata = read_forward_metadata_from_disk(&root, &source);
                                let snapshots = metadata["humanForwardBatches"][0]["attachments"]
                                    .as_array().into_iter().flatten().map(|attachment|
                                        fs::read(attachment["path"].as_str().expect("staged file path"))
                                            .expect("snapshot exists before resource POST")
                                    ).collect();
                                state.first_upload_snapshot = Some((metadata, snapshots));
                            }
                        }
                        let (name, bytes) = forward_multipart_file(&request_bytes);
                        let id = format!("forward-resource-{}", state.resources.len() + 1);
                        let resource = json!({
                            "resourceId":id,"name":name,"size":bytes.len(),
                            "contentType":"application/octet-stream","createdAt":"2026-10-05T01:00:00Z"
                        });
                        state.uploaded_files.push((name, bytes.clone()));
                        state.resources.insert(id, (resource.clone(), bytes));
                        if state.uploaded_files.len() == 1 && state.mutate_second_pin_after_first_upload {
                            let (metadata, snapshots) = state.first_upload_snapshot.as_ref()
                                .expect("observe the committed snapshots before mutation");
                            let path = std::path::PathBuf::from(metadata["humanForwardBatches"][0]
                                ["attachments"][1]["path"].as_str().expect("second pin path"));
                            let original = snapshots[1].clone();
                            assert_ne!(original, vec![b'X'; original.len()]);
                            // The server has accepted the first file, while the
                            // client has not yet received its acknowledgement.
                            fs::write(&path, vec![b'X'; original.len()])
                                .expect("replace the next pin after preflight with same-length bytes");
                            state.mutated_second_pin = Some((path, original));
                        }
                        (json!({"success":true,"resource":resource}).to_string().into_bytes(),
                            "application/json", 201)
                    } else if request.starts_with("GET /api/social/message-resources/") {
                        let path = request.split_whitespace().nth(1).expect("resource GET path");
                        let id = path.strip_prefix("/api/social/message-resources/").expect("resource prefix");
                        let (_, bytes) = state.resources.get(id).expect("download only uploaded resource");
                        (bytes.clone(), "application/octet-stream", 200)
                    } else if is_post {
                        post_count += 1;
                        if post_count == 1 {
                            if let Some((path, original)) = state.mutated_second_pin.take() {
                                assert!(state.mutation_observed_by_second_upload,
                                    "next upload happened while its pin held different bytes");
                                fs::write(path, original).expect("restore pin before the next destination preflight");
                                state.mutation_restored_before_message_ack = true;
                            }
                        }
                        let submitted = fixture_request_json(&request);
                        let nonce = submitted["clientRequestId"].as_str().expect("forward nonce");
                        let peer = submitted["targetUserId"].as_str().expect("forward target")
                            .parse::<u64>().expect("numeric forward target");
                        let text = submitted["text"].as_str().expect("forward text");
                        let attachments = submitted["attachments"].as_array().into_iter().flatten()
                            .map(|item| {
                                let id = item["resourceId"].as_str().expect("submitted resource identity");
                                state.resources.get(id).expect("message binds an actually uploaded resource").0.clone()
                            }).collect::<Vec<_>>();
                        let existing = state.accepted.get(nonce).cloned();
                        let deduplicated = existing.is_some();
                        let message = existing.unwrap_or_else(|| fixture_remote_human_message(
                            70 + state.accepted.len() as u64, 1, peer, text, Some(nonce), json!(attachments)
                        ));
                        assert_eq!(message["recipientUserId"], peer, "nonce cannot rebind recipient");
                        assert_eq!(message["text"], text, "nonce cannot rebind submitted snapshot");
                        assert_eq!(message["attachments"], json!(attachments), "nonce cannot rebind uploaded resources");
                        state.accepted.insert(nonce.to_string(), message.clone());
                        (json!({"success":true,"deduplicated":deduplicated,"message":message})
                            .to_string().into_bytes(), "application/json", 200)
                    } else {
                        assert!(request.starts_with("GET /api/social/messages?"), "{request}");
                        let after = request.split("afterId=").nth(1)
                            .and_then(|tail| tail.split(|c: char| !c.is_ascii_digit()).next())
                            .and_then(|value| value.parse::<u64>().ok()).expect("coverage cursor");
                        (json!({"success":true,"data":{
                            "messages":[],"historyCursor":{"mode":"after-id","afterId":after},
                            "nextAfterId":after
                        }}).to_string().into_bytes(), "application/json", 200)
                    }
                };
                if is_post && post_count == 1 {
                    seen.send(()).expect("first POST observation");
                    if hold_first_post {
                        released.recv_timeout(std::time::Duration::from_secs(30))
                            .expect("first POST must be explicitly released");
                    }
                }
                let response = format!(
                    "HTTP/1.1 {status} OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let result = stream.write_all(response.as_bytes()).and_then(|_| stream.write_all(&body));
                // The killed child deliberately cannot receive the first ack.
                // All subsequent response writes must still be successful.
                if !(hold_first_post && is_post && post_count == 1) {
                    result.expect("forward response write");
                }
            }
        });
        Self { url, state, stop, first_post, release: Some(release), thread: Some(worker) }
    }

    fn wait_for_first_post(&self) {
        self.first_post.recv_timeout(std::time::Duration::from_secs(20))
            .expect("shipping process reached first POST");
    }

    fn release_first_post(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }

    fn post_count(&self) -> usize {
        self.state.lock().expect("forward requests").requests.iter()
            .filter(|request| request.starts_with("POST /api/social/messages ")).count()
    }

    fn history_count(&self) -> usize {
        self.state.lock().expect("forward history requests").requests.iter()
            .filter(|request| request.starts_with("GET /api/social/messages?")).count()
    }

    fn resource_post_count(&self) -> usize {
        self.state.lock().expect("forward resources").uploaded_files.len()
    }

    fn observe_snapshot_staging(&self, root: &std::path::Path, source: &str) {
        self.state.lock().expect("forward observer").observe_source =
            Some((root.to_path_buf(), source.to_string()));
    }

    fn mutate_second_pin_after_first_upload(&self) {
        self.state.lock().expect("owned pin mutation")
            .mutate_second_pin_after_first_upload = true;
    }

    fn finish(mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        self.release_first_post();
        self.thread.take().expect("owned fixture thread").join().expect("forward fixture");
    }
}

impl Drop for ForwardHttpFixture {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        self.release_first_post();
        if let Some(worker) = self.thread.take() { let _ = worker.join(); }
    }
}

struct ForwardProcess(std::process::Child);

impl ForwardProcess {
    fn start(root: &std::path::Path, url: &str) -> Self {
        Self(std::process::Command::new(std::env::current_exe().expect("integration executable"))
            .args(["--exact", "forward_process_child_entry", "--nocapture"])
            .env("FABUSHI_FORWARD_PROCESS_ROOT", root)
            .env("FABUSHI_FORWARD_PROCESS_URL", url)
            .spawn().expect("actual forward child process"))
    }

    fn terminate(&mut self) {
        assert!(self.0.try_wait().expect("inspect forward child").is_none(),
            "child must still be in the held HTTP request");
        self.0.kill().expect("terminate exact owned forward process");
        let status = self.0.wait().expect("reap terminated forward process");
        assert!(!status.success(), "termination cannot be normal Session shutdown");
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(status.signal(), Some(9), "require actual SIGKILL");
        }
    }
}

impl Drop for ForwardProcess {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

// A helper entry, not an extra selected shipping_human_ case. It has no
// side effects under an ordinary unfiltered test invocation.
#[test]
fn forward_process_child_entry() {
    let Some(root) = std::env::var_os("FABUSHI_FORWARD_PROCESS_ROOT") else { return; };
    let root = std::path::PathBuf::from(root);
    let url = std::env::var("FABUSHI_FORWARD_PROCESS_URL").expect("child fixture issuer");
    let args: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("forward-input.json")).expect("child input")
    ).expect("forward input JSON");
    let runtime = human_message_fixture_runtime(&root, &url);
    let _ = dispatch(&runtime, "forwardHumanMessage", args);
    panic!("parent must terminate the child before releasing the first response");
}

fn prepare_forward_process(root: &std::path::Path, url: &str) -> serde_json::Value {
    let runtime = human_message_fixture_runtime(root, url);
    let source = runtime.create_human_conversation("2", "Source").expect("source")["id"]
        .as_str().expect("source id").to_string();
    let first = runtime.create_human_conversation("3", "First").expect("first target")["id"]
        .as_str().expect("first id").to_string();
    let second = runtime.create_human_conversation("4", "Second").expect("second target")["id"]
        .as_str().expect("second id").to_string();
    let owner = runtime.open_human_conversation_db_owner(&source).expect("source owner");
    assert!(owner.append_transcript_entry(&json!({
        "id":"human-server-message:51","kind":"message","role":"user",
        "authorKind":"human","authorId":"2","authorName":"Source author",
        "content":"immutable forwarded snapshot","timestampMs":1000.0,"delivery":"sent",
        "remoteMessageId":"51","remoteCreatedAt":"1970-01-01T00:00:01Z"
    })).expect("canonical source row"));
    let args = json!({
        "sourceConversationId":source,"sourceEntryId":"human-server-message:51",
        "destinationConversationIds":[first,second],"clientNonce":"durable-process-forward",
        "dropSenderNames":false,"dropCaptions":false
    });
    fs::write(root.join("forward-input.json"), args.to_string()).expect("persist child arguments");
    runtime.shutdown();
    drop(owner); drop(runtime);
    args
}

fn forward_db_path(root: &std::path::Path, conversation: &str) -> std::path::PathBuf {
    root.join("agents").join(".conversations").join(conversation).join("store.db")
}

fn read_forward_metadata_from_disk(root: &std::path::Path, conversation: &str) -> serde_json::Value {
    let db = rusqlite::Connection::open_with_flags(
        forward_db_path(root, conversation), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
    ).expect("independent read-only SQLite connection");
    let hex: String = db.query_row("SELECT value FROM kv WHERE key = 'metadata'", [],
        |row| row.get(0)).expect("committed metadata row");
    assert_eq!(hex.len() % 2, 0);
    let bytes = (0..hex.len()).step_by(2).map(|offset|
        u8::from_str_radix(&hex[offset..offset + 2], 16).expect("metadata hex")
    ).collect::<Vec<_>>();
    serde_json::from_slice(&bytes).expect("committed metadata JSON")
}

fn forward_gateway_result(runtime: &Arc<ProductionSessionWorkers>, args: &serde_json::Value)
    -> Result<serde_json::Value, SessionGatewayError>
{
    dispatch_production_session_gateway_call(runtime, "forwardHumanMessage", args)
        .expect("registered forward gateway")
}

fn assert_forward_settled(runtime: &Arc<ProductionSessionWorkers>, args: &serde_json::Value) {
    let source = args["sourceConversationId"].as_str().expect("source id");
    let owner = runtime.open_human_conversation_db_owner(source).expect("source owner");
    let batch = owner.get_metadata("humanForwardBatches").expect("journal").expect("persisted batch");
    assert_eq!(batch.as_array().expect("batches").len(), 1);
    for (index, destination) in args["destinationConversationIds"].as_array().unwrap().iter().enumerate() {
        assert_eq!(batch[0]["destinations"][index]["completed"], true);
        let entries = runtime.read_human_conversation_transcript(destination.as_str().unwrap())
            .expect("settled destination");
        assert_eq!(entries.len(), 1, "one canonical row per target");
        assert_eq!(entries[0]["delivery"], "sent");
        assert_eq!(entries[0]["content"], "immutable forwarded snapshot");
        assert_eq!(entries[0]["clientNonce"], batch[0]["destinations"][index]["clientNonce"]);
        assert_eq!(entries[0]["forwardContext"]["requestId"], args["clientNonce"]);
        assert!(entries[0]["remoteMessageId"].is_string());
    }
}

#[test]
fn shipping_human_forward_process_kill_preserves_complete_intent_before_first_post() {
    let root = temp_root("forward-process-kill");
    let mut server = ForwardHttpFixture::start(true);
    let args = prepare_forward_process(&root, &server.url);
    let mut child = ForwardProcess::start(&root, &server.url);
    server.wait_for_first_post();
    // Read from another connection while the child is alive in its first POST.
    let metadata = read_forward_metadata_from_disk(&root, args["sourceConversationId"].as_str().unwrap());
    let batch = &metadata["humanForwardBatches"][0];
    assert_eq!(batch["version"], 1);
    assert_eq!(batch["accountId"], "1");
    assert_eq!(batch["issuer"], server.url);
    assert_eq!(batch["requestId"], args["clientNonce"]);
    assert_eq!(batch["text"], "immutable forwarded snapshot");
    assert!(!batch.to_string().contains("fixture-token"), "bearer must not enter the journal");
    assert_eq!(batch["destinations"].as_array().unwrap().len(), 2);
    for index in 0..2 {
        assert_eq!(batch["destinations"][index]["conversationId"], args["destinationConversationIds"][index]);
        assert_eq!(batch["destinations"][index]["completed"], false);
    }
    let first_request = {
        let state = server.state.lock().unwrap();
        assert_eq!(state.accepted.len(), 1, "backend accepted the first request before termination");
        fixture_request_json(&state.requests[0])
    };
    assert_eq!(first_request["clientRequestId"], batch["destinations"][0]["clientNonce"]);
    assert_eq!(first_request["targetUserId"], "3");
    child.terminate();
    server.release_first_post();
    let resumed = human_message_fixture_runtime(&root, &server.url);
    assert!(resumed.read_human_conversation_transcript(args["destinationConversationIds"][1].as_str().unwrap())
        .expect("unsubmitted second target").is_empty());
    assert_eq!(read_forward_metadata_from_disk(&root, args["sourceConversationId"].as_str().unwrap()), metadata);
    assert_eq!(server.post_count(), 1);
    resumed.shutdown(); drop(resumed); drop(child);
    server.finish();
    fs::remove_dir_all(root).expect("remove process-kill fixture");
}

#[test]
fn shipping_human_forward_restart_sync_recovers_unsubmitted_target_and_original_snapshot() {
    let root = temp_root("forward-process-resume");
    let mut server = ForwardHttpFixture::start(true);
    let args = prepare_forward_process(&root, &server.url);
    let mut child = ForwardProcess::start(&root, &server.url);
    server.wait_for_first_post();
    child.terminate();
    server.release_first_post();
    let resumed = human_message_fixture_runtime(&root, &server.url);
    let source = args["sourceConversationId"].as_str().unwrap();
    let owner = resumed.open_human_conversation_db_owner(source).expect("reopened source");
    let mut edited = owner.get_transcript_entries().expect("source entry")[0].clone();
    edited["content"] = json!("edited after accepted whole-batch intent");
    owner.update_transcript_entry(args["sourceEntryId"].as_str().unwrap(), &edited)
        .expect("source edited after crash").expect("updated source");
    let second = args["destinationConversationIds"][1].as_str().unwrap();
    assert!(resumed.read_human_conversation_transcript(second).expect("never dispatched target").is_empty());
    let transcript = dispatch(&resumed, "syncHumanConversation", json!({"conversationId":second}));
    assert_eq!(transcript.as_array().expect("gateway transcript").len(), 1);
    assert_forward_settled(&resumed, &args);
    assert_eq!(server.post_count(), 3, "accepted first POST must replay once and second target dispatch once");
    assert_eq!(server.state.lock().unwrap().accepted.len(), 2, "server nonce dedupe creates only two messages");
    resumed.shutdown(); drop(owner); drop(resumed); drop(child);
    let reopened = human_message_fixture_runtime(&root, &server.url);
    assert_forward_settled(&reopened, &args);
    assert_eq!(forward_gateway_result(&reopened, &args).expect("completed retry")["destinations"][1]["status"], "sent");
    assert_eq!(server.post_count(), 3, "completed journal must not POST on another restart");
    reopened.shutdown(); drop(reopened);
    server.finish(); fs::remove_dir_all(root).expect("remove process-resume fixture");
}

#[test]
fn shipping_human_forward_idempotent_replay_rejects_changed_scope_and_expanded_destinations() {
    let root = temp_root("forward-idempotent-replay");
    let server = ForwardHttpFixture::start(false);
    let args = prepare_forward_process(&root, &server.url);
    let runtime = human_message_fixture_runtime(&root, &server.url);
    let first = forward_gateway_result(&runtime, &args).expect("first complete batch");
    assert_eq!(first["destinations"][0]["status"], "sent");
    assert_eq!(first["destinations"][1]["status"], "sent");
    let mut subset = args.clone();
    subset["destinationConversationIds"] = json!([args["destinationConversationIds"][1]]);
    let replay = forward_gateway_result(&runtime, &subset).expect("original subset retry");
    assert_eq!(replay["destinations"][0]["clientNonce"], first["destinations"][1]["clientNonce"]);
    let concurrent = (0..2).map(|_| {
        let runtime = Arc::clone(&runtime);
        let args = args.clone();
        thread::spawn(move || forward_gateway_result(&runtime, &args).expect("concurrent identical retry"))
    }).collect::<Vec<_>>();
    for replay in concurrent { assert_eq!(replay.join().expect("replay thread"), first); }
    for changed in ["sourceEntryId", "sourceConversationId", "privacy", "destinations"] {
        let mut invalid = args.clone();
        match changed {
            "sourceEntryId" => invalid["sourceEntryId"] = json!("another-entry"),
            "sourceConversationId" => invalid["sourceConversationId"] = args["destinationConversationIds"][0].clone(),
            "privacy" => invalid["dropSenderNames"] = json!(true),
            _ => invalid["destinationConversationIds"].as_array_mut().unwrap().push(json!("human-extra-target")),
        }
        assert!(forward_gateway_result(&runtime, &invalid).is_err(), "reject changed {changed}");
    }
    assert_forward_settled(&runtime, &args);
    assert_eq!(server.post_count(), 2, "repeated, concurrent and invalid calls must not dispatch");
    runtime.shutdown(); drop(runtime);
    server.finish(); fs::remove_dir_all(root).expect("remove idempotent fixture");
}

#[test]
fn shipping_human_forward_restart_rechecks_source_permissions_and_credential_scope() {
    let root = temp_root("forward-permission-recovery");
    let mut server = ForwardHttpFixture::start(true);
    let args = prepare_forward_process(&root, &server.url);
    let mut child = ForwardProcess::start(&root, &server.url);
    server.wait_for_first_post(); child.terminate(); server.release_first_post();
    let runtime = human_message_fixture_runtime(&root, &server.url);
    let source = args["sourceConversationId"].as_str().unwrap();
    let first = args["destinationConversationIds"][0].as_str().unwrap();
    let second = args["destinationConversationIds"][1].as_str().unwrap();
    let owner = runtime.open_human_conversation_db_owner(source).expect("source owner");
    let original = owner.get_transcript_entries().unwrap()[0].clone();
    let journal = owner.get_metadata("humanForwardBatches").unwrap();
    let mut protected = original.clone(); protected["protectedContent"] = json!(true);
    owner.update_transcript_entry(args["sourceEntryId"].as_str().unwrap(), &protected).unwrap().unwrap();
    runtime.sync_human_conversation(second).expect("protected source cannot starve unrelated history");
    assert_eq!(server.history_count(), 1, "protected source still permits real history HTTP");
    let protected_result = forward_gateway_result(&runtime, &args).expect("durable policy-denial envelope");
    for destination in protected_result["destinations"].as_array().unwrap() {
        assert_eq!(destination["status"], "failed");
        assert!(destination["error"].as_str().unwrap().contains("protected"));
    }
    owner.delete_transcript_entry(args["sourceEntryId"].as_str().unwrap()).unwrap();
    runtime.sync_human_conversation(second).expect("deleted source cannot starve unrelated history");
    assert_eq!(server.history_count(), 2, "missing source still permits real history HTTP");
    let missing_result = forward_gateway_result(&runtime, &args).expect("deleted source failure envelope");
    assert!(missing_result["destinations"][0]["error"].as_str().unwrap().contains("missing"));
    assert!(owner.append_transcript_entry(&original).unwrap());
    assert!(owner.set_metadata("participantIds", json!(["2", "9"])).unwrap());
    // The source is excluded from the authorized conversation list, and the
    // first target has a real dispatching outbox row from the killed POST.
    // Its generic pending-send recovery must not bypass the source journal.
    let pending_before = runtime.read_human_conversation_transcript(first).unwrap();
    assert_eq!(pending_before.len(), 1);
    assert_eq!(pending_before[0]["delivery"], "dispatching");
    assert!(pending_before[0].get("remoteMessageId").is_none());
    runtime.sync_human_conversation(first).expect("authorized target can sync without bypassing source");
    assert_eq!(server.history_count(), 3, "removed membership cannot starve target history");
    assert_eq!(server.post_count(), 1, "generic outbox must not resend the revoked-source forward");
    assert_eq!(runtime.read_human_conversation_transcript(first).unwrap(), pending_before);
    let removed_result = forward_gateway_result(&runtime, &args).expect("removed membership failure envelope");
    assert_eq!(removed_result["destinations"][0]["status"], "failed");
    assert!(removed_result["destinations"][0]["error"].as_str().unwrap().contains("participant"));
    assert!(owner.set_metadata("participantIds", json!(["1", "2"])).unwrap());
    let credential_path = root.join("host-credential.json");
    let credential = fs::read_to_string(&credential_path).unwrap();
    for (key, value) in [("userId", json!(9)), ("issuer", json!("http://127.0.0.1:1"))] {
        let mut changed: serde_json::Value = serde_json::from_str(&credential).unwrap();
        changed[key] = value;
        fs::write(&credential_path, changed.to_string()).unwrap();
        assert!(runtime.sync_human_conversation(second).is_err(), "changed {key} cannot resume old intent");
    }
    fs::remove_file(&credential_path).unwrap();
    assert!(runtime.sync_human_conversation(second).is_err(), "missing private credentials cannot resume");
    assert_eq!(server.post_count(), 1, "all permission and credential negatives prevent further sends");
    let blocked = owner.get_metadata("humanForwardBatches").unwrap().unwrap();
    let original_journal = journal.unwrap();
    for key in ["version", "accountId", "issuer", "requestId", "sourceConversationId", "sourceEntryId", "text", "attachments", "context"] {
        assert_eq!(blocked[0][key], original_journal[0][key], "retain immutable blocked {key}");
    }
    for index in 0..2 {
        assert_eq!(blocked[0]["destinations"][index]["conversationId"], original_journal[0]["destinations"][index]["conversationId"]);
        assert_eq!(blocked[0]["destinations"][index]["clientNonce"], original_journal[0]["destinations"][index]["clientNonce"]);
        assert_eq!(blocked[0]["destinations"][index]["completed"], false);
        assert!(blocked[0]["destinations"][index]["error"].is_string(), "policy denial is durable");
    }
    assert!(runtime.read_human_conversation_transcript(second).unwrap().is_empty());
    runtime.shutdown(); drop(owner); drop(runtime); drop(child);
    let recovered = human_message_fixture_runtime(&root, &server.url);
    dispatch(&recovered, "syncHumanConversation", json!({"conversationId":second}));
    assert_forward_settled(&recovered, &args);
    assert_eq!(server.post_count(), 3);
    recovered.shutdown(); drop(recovered);
    server.finish(); fs::remove_dir_all(root).expect("remove permission fixture");
}

#[test]
fn shipping_human_forward_sqlite_failures_stop_initial_and_progress_dispatch() {
    for fail_progress in [false, true] {
        let root = temp_root(if fail_progress { "forward-sqlite-progress" } else { "forward-sqlite-staging" });
        let mut server = ForwardHttpFixture::start(fail_progress);
        let args = prepare_forward_process(&root, &server.url);
        let runtime = human_message_fixture_runtime(&root, &server.url);
        let source = args["sourceConversationId"].as_str().unwrap();
        let db = rusqlite::Connection::open(forward_db_path(&root, source)).expect("SQLite fault injector");
        db.busy_timeout(std::time::Duration::from_secs(5)).unwrap();
        let running = if fail_progress {
            let runtime = Arc::clone(&runtime); let args = args.clone();
            let running = thread::spawn(move || forward_gateway_result(&runtime, &args));
            server.wait_for_first_post();
            Some(running)
        } else { None };
        let before = read_forward_metadata_from_disk(&root, source);
        db.execute_batch("CREATE TRIGGER reject_forward_metadata BEFORE UPDATE OF value ON kv
            WHEN NEW.key = 'metadata' BEGIN SELECT RAISE(ABORT, 'NMR-47 injected SQLite persistence failure'); END;")
            .expect("install real SQLite write rejection");
        if let Some(running) = running {
            server.release_first_post();
            assert!(running.join().expect("forward after ack").is_err(), "progress commit must fail");
        } else {
            for _ in 0..2 {
                assert!(forward_gateway_result(&runtime, &args).is_err(), "failed staging must not leak in-memory success");
            }
        }
        assert_eq!(server.post_count(), usize::from(fail_progress));
        assert_eq!(read_forward_metadata_from_disk(&root, source), before, "failed transaction preserves durable record");
        assert!(runtime.read_human_conversation_transcript(args["destinationConversationIds"][1].as_str().unwrap())
            .unwrap().is_empty(), "never dispatch second target after failed durability");
        if fail_progress {
            let first = runtime.read_human_conversation_transcript(args["destinationConversationIds"][0].as_str().unwrap()).unwrap();
            assert_eq!(first.len(), 1);
            assert_eq!(first[0]["delivery"], "sent", "ack settled before batch-progress failure");
            assert_eq!(before["humanForwardBatches"][0]["destinations"][0]["completed"], false);
        } else {
            assert!(before.get("humanForwardBatches").is_none());
        }
        db.execute_batch("DROP TRIGGER reject_forward_metadata;").expect("remove scoped injected fault");
        drop(db); runtime.shutdown(); drop(runtime);
        let resumed = human_message_fixture_runtime(&root, &server.url);
        if fail_progress {
            dispatch(&resumed, "syncHumanConversation", json!({"conversationId":args["destinationConversationIds"][1]}));
        } else {
            forward_gateway_result(&resumed, &args).expect("retry after persistence recovery");
        }
        assert_forward_settled(&resumed, &args);
        assert_eq!(server.post_count(), 2, "settled first target cannot be sent again after progress recovery");
        assert_eq!(server.state.lock().unwrap().accepted.len(), 2);
        resumed.shutdown(); drop(resumed);
        server.finish(); fs::remove_dir_all(root).expect("remove SQLite fault fixture");
    }
}

// NMR-48 byte oracles start from independently specified binary input and the
// actual multipart upload, not a serializer/helper from the snapshot owner.
type ForwardFileFixture = (String, std::path::PathBuf, Vec<u8>);

fn forward_attachment_files(root: &std::path::Path, count: usize) -> Vec<ForwardFileFixture> {
    fs::create_dir_all(root).expect("attachment fixture root");
    (0..count).map(|index| {
        let name = format!("report-{index}.bin");
        let path = root.join(format!("original-cache-{index}.dat"));
        let mut bytes = b"\x00original forward payload\xff\r\n".to_vec();
        bytes.extend_from_slice(&[index as u8, 0x80, 0]);
        fs::write(&path, &bytes).expect("original binary attachment");
        (name, path, bytes)
    }).collect()
}

fn prepare_forward_attachment_process(
    root: &std::path::Path,
    url: &str,
    files: &[ForwardFileFixture],
) -> serde_json::Value {
    let args = prepare_forward_process(root, url);
    let runtime = human_message_fixture_runtime(root, url);
    let owner = runtime.open_human_conversation_db_owner(
        args["sourceConversationId"].as_str().unwrap()
    ).expect("attachment source owner");
    let mut source = owner.get_transcript_entries().expect("source rows")[0].clone();
    source["attachments"] = json!(files.iter().enumerate().map(|(index, (name, path, bytes))|
        json!({"name":name,"path":path,"size":bytes.len(),"resourceId":format!("original-resource-{index}")})
    ).collect::<Vec<_>>());
    owner.update_transcript_entry(args["sourceEntryId"].as_str().unwrap(), &source)
        .expect("store source attachments").expect("source exists");
    runtime.shutdown(); drop(owner); drop(runtime);
    args
}

fn assert_forward_staged_bytes(
    root: &std::path::Path,
    args: &serde_json::Value,
    files: &[ForwardFileFixture],
    server: &ForwardHttpFixture,
) -> Vec<std::path::PathBuf> {
    use sha2::Digest;
    let source = args["sourceConversationId"].as_str().unwrap();
    let metadata = read_forward_metadata_from_disk(root, source);
    let batch = &metadata["humanForwardBatches"][0];
    assert_eq!(batch["destinations"].as_array().unwrap().len(), 2);
    let attachments = batch["attachments"].as_array().expect("durable pinned attachments");
    assert_eq!(attachments.len(), files.len());
    let directory = fs::canonicalize(forward_db_path(root, source).parent().unwrap())
        .expect("canonical source Session directory").join("forward-attachments");
    let paths = attachments.iter().zip(files.iter()).map(|(attachment, (name, original, bytes))| {
        assert_eq!(attachment["name"], json!(name));
        let path = std::path::PathBuf::from(attachment["path"].as_str().expect("snapshot path"));
        assert_ne!(&path, original, "accepted bytes cannot reference the mutable cache");
        assert!(path.starts_with(&directory), "snapshot remains inside this source Session owner");
        assert_eq!(fs::read(&path).expect("durable snapshot bytes"), *bytes);
        assert_eq!(attachment["forwardSnapshot"]["version"], 1);
        assert_eq!(attachment["forwardSnapshot"]["size"], json!(bytes.len()));
        assert_eq!(attachment["forwardSnapshot"]["sha256"],
            json!(format!("{:x}", sha2::Sha256::digest(bytes))));
        path
    }).collect::<Vec<_>>();
    let state = server.state.lock().expect("independent upload observation");
    let (observed, observed_bytes) = state.first_upload_snapshot.as_ref()
        .expect("observe committed intent during the first resource POST");
    assert_eq!(observed["humanForwardBatches"][0]["attachments"], batch["attachments"]);
    assert_eq!(*observed_bytes, files.iter().map(|(_, _, bytes)| bytes.clone()).collect::<Vec<_>>(),
        "all snapshots already exist before the first uploaded resource is acknowledged");
    assert!(observed["humanForwardBatches"][0]["destinations"].as_array().unwrap()
        .iter().all(|destination| destination["completed"] == false));
    paths
}

fn assert_forward_uploaded_bytes(server: &ForwardHttpFixture, files: &[ForwardFileFixture]) {
    let state = server.state.lock().expect("captured multipart data");
    assert_eq!(state.uploaded_files.len(), files.len() * 2, "one upload set per destination");
    for (index, (name, bytes)) in state.uploaded_files.iter().enumerate() {
        let expected = &files[index % files.len()];
        assert_eq!(name, &expected.0, "preserve submitted file name and order");
        assert_eq!(bytes, &expected.2, "actual multipart bytes match the original submitted input");
    }
    assert_eq!(state.accepted.len(), 2, "nonce dedupe creates one message per destination");
    let mut peers = std::collections::BTreeSet::new();
    for message in state.accepted.values() {
        peers.insert(message["recipientUserId"].as_u64().unwrap());
        let attachments = message["attachments"].as_array().unwrap();
        assert_eq!(attachments.len(), files.len());
        for (attachment, (name, _, expected)) in attachments.iter().zip(files.iter()) {
            assert_eq!(attachment["name"], json!(name));
            let (_, bytes) = state.resources.get(attachment["resourceId"].as_str().unwrap()).unwrap();
            assert_eq!(bytes, expected, "each accepted destination binds the actual uploaded bytes");
        }
    }
    assert_eq!(peers, [3_u64, 4].into_iter().collect::<std::collections::BTreeSet<_>>());
}

fn assert_forward_intent_unchanged(before: &serde_json::Value, after: &serde_json::Value) {
    for key in ["version", "accountId", "issuer", "requestId", "sourceConversationId", "sourceEntryId", "text", "attachments", "context"] {
        assert_eq!(after[key], before[key], "retain immutable forward {key}");
    }
    for index in 0..2 {
        for key in ["conversationId", "clientNonce"] {
            assert_eq!(after["destinations"][index][key], before["destinations"][index][key]);
        }
    }
}

#[test]
fn shipping_human_forward_attachment_bytes_survive_process_kill_and_source_cache_replacement() {
    for delete_original in [false, true] {
        let root = temp_root(if delete_original { "forward-byte-deleted-cache" } else { "forward-byte-replaced-cache" });
        let files = forward_attachment_files(&root, 2);
        let mut server = ForwardHttpFixture::start(true);
        let args = prepare_forward_attachment_process(&root, &server.url, &files);
        let source = args["sourceConversationId"].as_str().unwrap();
        server.observe_snapshot_staging(&root, source);
        let mut child = ForwardProcess::start(&root, &server.url);
        server.wait_for_first_post();
        let snapshots = assert_forward_staged_bytes(&root, &args, &files, &server);
        assert_eq!(server.resource_post_count(), 2);
        assert_eq!(server.post_count(), 1);
        child.terminate(); server.release_first_post();
        for (_, path, bytes) in &files {
            if delete_original { fs::remove_file(path).expect("delete mutable original cache"); }
            else { fs::write(path, vec![b'X'; bytes.len()]).expect("replace original with same-size bytes"); }
        }
        let resumed = human_message_fixture_runtime(&root, &server.url);
        dispatch(&resumed, "syncHumanConversation", json!({"conversationId":args["destinationConversationIds"][1]}));
        assert_forward_settled(&resumed, &args);
        assert_forward_uploaded_bytes(&server, &files);
        assert_eq!(server.post_count(), 3, "replay accepted first nonce once, send second once");
        for (path, (_, original, bytes)) in snapshots.iter().zip(files.iter()) {
            assert_eq!(fs::read(path).unwrap(), *bytes, "the retained snapshot is not rewritten on recovery");
            if delete_original { assert!(!original.exists(), "recovery must not recreate a deleted user cache"); }
            else { assert_eq!(fs::read(original).unwrap(), vec![b'X'; bytes.len()]); }
        }
        resumed.shutdown(); drop(resumed); drop(child);
        let reopened = human_message_fixture_runtime(&root, &server.url);
        dispatch(&reopened, "syncHumanConversation", json!({"conversationId":args["destinationConversationIds"][1]}));
        assert_forward_settled(&reopened, &args);
        assert_eq!(server.post_count(), 3);
        assert_eq!(server.resource_post_count(), 4, "another restart cannot re-upload completed resources");
        reopened.shutdown(); drop(reopened);
        server.finish(); fs::remove_dir_all(root).expect("remove immutable attachment fixture");
    }
}

#[test]
fn shipping_human_forward_missing_or_corrupt_snapshot_retains_intent_until_exact_byte_repair() {
    for fault in ["missing", "same-size-corrupt", "oversized", "directory"] {
        let root = temp_root(&format!("forward-snapshot-{fault}"));
        let files = forward_attachment_files(&root, 2);
        let mut server = ForwardHttpFixture::start(true);
        let args = prepare_forward_attachment_process(&root, &server.url, &files);
        let source = args["sourceConversationId"].as_str().unwrap();
        let second = args["destinationConversationIds"][1].as_str().unwrap();
        server.observe_snapshot_staging(&root, source);
        let mut child = ForwardProcess::start(&root, &server.url);
        server.wait_for_first_post();
        let snapshots = assert_forward_staged_bytes(&root, &args, &files, &server);
        child.terminate(); server.release_first_post();
        let runtime = human_message_fixture_runtime(&root, &server.url);
        let mut first_only = args.clone();
        first_only["destinationConversationIds"] = json!([args["destinationConversationIds"][0]]);
        assert_eq!(forward_gateway_result(&runtime, &first_only).expect("settle accepted resource identity")["destinations"][0]["status"], "sent");
        assert_eq!(server.post_count(), 2);
        assert_eq!(server.resource_post_count(), 2, "ambiguous POST reuses its persisted resource IDs");
        let before = read_forward_metadata_from_disk(&root, source)["humanForwardBatches"][0].clone();
        // The first file stays valid: a per-file read/upload loop would leak a
        // resource POST before discovering this later invalid attachment.
        let snapshot = &snapshots[1];
        match fault {
            "missing" => fs::remove_file(snapshot).unwrap(),
            "same-size-corrupt" => fs::write(snapshot, vec![b'X'; files[1].2.len()]).unwrap(),
            "oversized" => fs::File::create(snapshot).unwrap().set_len(32 * 1024 * 1024 + 1).unwrap(),
            _ => { fs::remove_file(snapshot).unwrap(); fs::create_dir(snapshot).unwrap(); }
        }
        let failed = forward_gateway_result(&runtime, &args).expect("snapshot failure is a durable per-target result");
        assert_eq!(failed["destinations"][0]["status"], "sent");
        assert_eq!(failed["destinations"][1]["status"], "failed");
        let expected_error = match fault {
            "missing" => "unavailable",
            "same-size-corrupt" => "snapshot digest",
            "oversized" => "size",
            _ => "regular non-symlink",
        };
        assert!(failed["destinations"][1]["error"].as_str()
            .is_some_and(|error| error.contains(expected_error)), "report the actual {fault} cause");
        let pending = runtime.read_human_conversation_transcript(second).expect("retryable pinned outbox");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0]["delivery"], "pending");
        assert!(pending[0].get("remoteMessageId").is_none());
        assert!(pending[0].get("remoteDispatchAttachments").is_none());
        let history_before = server.history_count();
        dispatch(&runtime, "syncHumanConversation", json!({"conversationId":second}));
        assert!(server.history_count() > history_before, "a damaged pin cannot starve unrelated inbound history");
        assert_eq!(server.post_count(), 2, "no message POST while pin is invalid");
        assert_eq!(server.resource_post_count(), 2, "no resource POST even for the valid first pin when a later pin is invalid");
        let blocked = read_forward_metadata_from_disk(&root, source)["humanForwardBatches"][0].clone();
        assert_forward_intent_unchanged(&before, &blocked);
        assert_eq!(blocked["destinations"][0]["completed"], true);
        assert_eq!(blocked["destinations"][1]["completed"], false);
        assert!(blocked["destinations"][1]["error"].is_string());
        runtime.shutdown(); drop(runtime); drop(child);
        let resumed = human_message_fixture_runtime(&root, &server.url);
        let reopened_pending = resumed.read_human_conversation_transcript(second).unwrap();
        for key in ["id", "content", "clientNonce", "attachments", "forwardContext", "delivery"] {
            assert_eq!(reopened_pending[0][key], pending[0][key], "persist {fault} outbox {key}");
        }
        if fault == "directory" { fs::remove_dir(snapshot).expect("remove invalid snapshot directory"); }
        fs::write(snapshot, &files[1].2).expect("restore the exact independently retained input bytes");
        dispatch(&resumed, "syncHumanConversation", json!({"conversationId":second}));
        assert_forward_settled(&resumed, &args);
        assert_forward_uploaded_bytes(&server, &files);
        assert_eq!(server.post_count(), 3);
        resumed.shutdown(); drop(resumed);
        server.finish(); fs::remove_dir_all(root).expect("remove damaged snapshot fixture");
    }
}

#[test]
fn shipping_human_forward_invalid_attachment_inputs_and_snapshot_staging_fail_before_post() {
    for fault in ["too-many", "empty", "oversized", "directory", "staging-path"] {
        let root = temp_root(&format!("forward-attachment-admission-{fault}"));
        let files = forward_attachment_files(&root, if fault == "too-many" { 7 } else { 1 });
        let server = ForwardHttpFixture::start(false);
        let args = prepare_forward_attachment_process(&root, &server.url, &files);
        let source = args["sourceConversationId"].as_str().unwrap();
        let staging = forward_db_path(&root, source).parent().unwrap().join("forward-attachments");
        match fault {
            "empty" => fs::write(&files[0].1, []).unwrap(),
            "oversized" => fs::File::create(&files[0].1).unwrap().set_len(32 * 1024 * 1024 + 1).unwrap(),
            "directory" => { fs::remove_file(&files[0].1).unwrap(); fs::create_dir(&files[0].1).unwrap(); }
            "staging-path" => fs::write(&staging, b"retained unrelated file").unwrap(),
            _ => {}
        }
        let runtime = human_message_fixture_runtime(&root, &server.url);
        for _ in 0..2 {
            assert!(forward_gateway_result(&runtime, &args).is_err(), "reject new batch for {fault}");
            assert!(read_forward_metadata_from_disk(&root, source).get("humanForwardBatches").is_none(),
                "failed staging cannot accept a durable batch for {fault}");
        }
        for destination in args["destinationConversationIds"].as_array().unwrap() {
            assert!(runtime.read_human_conversation_transcript(destination.as_str().unwrap()).unwrap().is_empty());
        }
        assert_eq!(server.post_count(), 0, "no message POST for {fault}");
        assert_eq!(server.resource_post_count(), 0, "no resource POST for {fault}");
        assert!(server.state.lock().unwrap().accepted.is_empty());
        if fault == "staging-path" {
            assert_eq!(fs::read(&staging).unwrap(), b"retained unrelated file");
        }
        runtime.shutdown(); drop(runtime);
        server.finish(); fs::remove_dir_all(root).expect("remove attachment admission fixture");
    }
}

#[test]
fn shipping_human_forward_legacy_unpinned_attachments_fail_without_blocking_history_or_rebinding_bytes() {
    let root = temp_root("forward-legacy-unpinned");
    let files = forward_attachment_files(&root, 1);
    let mut server = ForwardHttpFixture::start(true);
    let args = prepare_forward_attachment_process(&root, &server.url, &files);
    let source = args["sourceConversationId"].as_str().unwrap();
    let first = args["destinationConversationIds"][0].as_str().unwrap();
    let second = args["destinationConversationIds"][1].as_str().unwrap();
    server.observe_snapshot_staging(&root, source);
    let mut child = ForwardProcess::start(&root, &server.url);
    server.wait_for_first_post();
    let snapshots = assert_forward_staged_bytes(&root, &args, &files, &server);
    child.terminate(); server.release_first_post();
    let runtime = human_message_fixture_runtime(&root, &server.url);
    let mut first_only = args.clone(); first_only["destinationConversationIds"] = json!([first]);
    assert_eq!(forward_gateway_result(&runtime, &first_only).unwrap()["destinations"][0]["status"], "sent");
    let completed_first = runtime.read_human_conversation_transcript(first).unwrap();
    let owner = runtime.open_human_conversation_db_owner(source).unwrap();
    let mut legacy = owner.get_metadata("humanForwardBatches").unwrap().unwrap();
    legacy[0]["attachments"][0].as_object_mut().unwrap().remove("forwardSnapshot");
    legacy[0]["attachments"][0]["path"] = json!(files[0].1);
    assert!(owner.set_metadata("humanForwardBatches", legacy.clone()).unwrap());
    fs::write(&files[0].1, vec![b'X'; files[0].2.len()]).expect("old mutable path now holds different bytes");
    runtime.shutdown(); drop(owner); drop(runtime); drop(child);
    let resumed = human_message_fixture_runtime(&root, &server.url);
    let before_history = server.history_count();
    dispatch(&resumed, "syncHumanConversation", json!({"conversationId":second}));
    assert!(server.history_count() > before_history);
    let failure = forward_gateway_result(&resumed, &args).expect("legacy retry remains a visible failure envelope");
    assert_eq!(failure["destinations"][0]["status"], "sent");
    assert_eq!(failure["destinations"][1]["status"], "failed");
    assert!(failure["destinations"][1]["error"].as_str().is_some_and(|error|
        error.contains("snapshot") || error.contains("unpinned") || error.contains("legacy")));
    assert_eq!(server.post_count(), 2, "never upload/send mutable legacy input to the unfinished target");
    assert_eq!(server.resource_post_count(), 1);
    assert_eq!(resumed.read_human_conversation_transcript(first).unwrap(), completed_first);
    let blocked = read_forward_metadata_from_disk(&root, source)["humanForwardBatches"][0].clone();
    assert_forward_intent_unchanged(&legacy[0], &blocked);
    assert_eq!(blocked["destinations"][0]["completed"], true);
    assert_eq!(blocked["destinations"][1]["completed"], false);
    assert!(blocked["attachments"][0].get("forwardSnapshot").is_none(), "do not invent historical byte evidence");
    assert_eq!(fs::read(&snapshots[0]).unwrap(), files[0].2, "retain existing immutable data for recovery review");
    resumed.shutdown(); drop(resumed);
    server.finish(); fs::remove_dir_all(root).expect("remove legacy attachment fixture");
}

#[test]
fn shipping_human_forward_attachment_snapshot_respects_source_policy_and_normalized_privacy() {
    let root = temp_root("forward-attachment-privacy");
    let files = forward_attachment_files(&root, 6);
    let server = ForwardHttpFixture::start(false);
    let mut args = prepare_forward_attachment_process(&root, &server.url, &files);
    args["dropCaptions"] = json!(true);
    args["dropSenderNames"] = json!(false);
    let source = args["sourceConversationId"].as_str().unwrap();
    server.observe_snapshot_staging(&root, source);
    server.mutate_second_pin_after_first_upload();
    let runtime = human_message_fixture_runtime(&root, &server.url);
    let owner = runtime.open_human_conversation_db_owner(source).unwrap();
    let original = owner.get_transcript_entries().unwrap()[0].clone();
    let mut protected = original.clone(); protected["protectedContent"] = json!(true);
    owner.update_transcript_entry(args["sourceEntryId"].as_str().unwrap(), &protected).unwrap().unwrap();
    assert!(forward_gateway_result(&runtime, &args).is_err(), "source permission precedes snapshot and upload");
    let staging = forward_db_path(&root, source).parent().unwrap().join("forward-attachments");
    assert!(!staging.exists());
    assert!(read_forward_metadata_from_disk(&root, source).get("humanForwardBatches").is_none());
    assert_eq!(server.post_count(), 0); assert_eq!(server.resource_post_count(), 0);
    owner.update_transcript_entry(args["sourceEntryId"].as_str().unwrap(), &original).unwrap().unwrap();
    let sent = forward_gateway_result(&runtime, &args).expect("authorized six-attachment forward");
    assert_eq!(sent["privacy"], json!({"dropSenderNames":true,"dropCaptions":true}));
    assert_forward_staged_bytes(&root, &args, &files, &server);
    assert_forward_uploaded_bytes(&server, &files);
    {
        let state = server.state.lock().expect("preflight byte lifetime observation");
        assert!(state.mutation_observed_by_second_upload,
            "the second pin changed after preflight and before its actual upload");
        assert!(state.mutation_restored_before_message_ack,
            "restore the retained bytes before the next destination begins");
    }
    assert_eq!(server.post_count(), 2);
    for destination in args["destinationConversationIds"].as_array().unwrap() {
        let rows = runtime.read_human_conversation_transcript(destination.as_str().unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["content"], "", "caption privacy applies to actual persisted target");
        assert_eq!(rows[0]["delivery"], "sent");
        assert_eq!(rows[0]["forwardContext"]["privacy"], sent["privacy"]);
        assert!(rows[0]["forwardContext"].get("origin").is_none(), "hidden sender is not retained in target provenance");
        let attachments = rows[0]["attachments"].as_array().unwrap();
        for (attachment, (name, _, bytes)) in attachments.iter().zip(files.iter()) {
            assert_eq!(attachment["name"], json!(name));
            assert_eq!(fs::read(attachment["path"].as_str().unwrap()).unwrap(), *bytes,
                "downloaded target resource has the same uploaded bytes");
        }
    }
    assert_eq!(owner.get_transcript_entries().unwrap()[0], original, "privacy cannot mutate the source");
    runtime.shutdown(); drop(owner); drop(runtime);
    let reopened = human_message_fixture_runtime(&root, &server.url);
    let replay = forward_gateway_result(&reopened, &args).expect("completed privacy replay");
    assert_eq!(replay, sent);
    assert_eq!(server.post_count(), 2); assert_eq!(server.resource_post_count(), 12);
    reopened.shutdown(); drop(reopened);
    server.finish(); fs::remove_dir_all(root).expect("remove privacy snapshot fixture");
}

#[test]
fn shipping_human_forward_restart_corruption_fails_closed_and_recovers_exact_journal() {
    let root = temp_root("forward-corrupt-recovery");
    let mut server = ForwardHttpFixture::start(true);
    let args = prepare_forward_process(&root, &server.url);
    let mut child = ForwardProcess::start(&root, &server.url);
    server.wait_for_first_post();
    child.terminate();
    server.release_first_post();
    drop(child);
    let source = args["sourceConversationId"].as_str().unwrap();
    let db = rusqlite::Connection::open(forward_db_path(&root, source)).unwrap();
    let original_hex: String = db.query_row("SELECT value FROM kv WHERE key='metadata'", [], |row| row.get(0)).unwrap();
    assert_eq!(read_forward_metadata_from_disk(&root, source)["humanForwardBatches"][0]["requestId"], args["clientNonce"]);
    for cached_owner in [false, true] {
        // Independent bytes: invalid hex, invalid JSON, null, an array and a
        // string. Valid non-object JSON must not become "no forward journal".
        for damaged in ["zz", "6e6f742d6a736f6e", "6e756c6c", "5b5d", "2264616d6167656422"] {
            db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", [&original_hex]).unwrap();
            let resumed = human_message_fixture_runtime(&root, &server.url);
            if cached_owner {
                resumed.open_human_conversation_db_owner(source).expect("cache valid source owner before damage");
            }
            db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", [damaged]).unwrap();
            let target = args["destinationConversationIds"][1].as_str().unwrap();
            let error = resumed.sync_human_conversation(target).expect_err("corrupt source metadata must block recovery");
            assert!(!error.is_empty());
            assert_eq!(server.post_count(), 1);
            assert_eq!(server.history_count(), 0);
            let stored: String = db.query_row("SELECT value FROM kv WHERE key='metadata'", [], |row| row.get(0)).unwrap();
            assert_eq!(stored, damaged);
            resumed.shutdown(); drop(resumed);
        }
    }
    // Restore the original serialized metadata after malformed hex/JSON
    // cases before deriving the next valid-JSON corruption fixtures.
    db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", [&original_hex]).unwrap();
    // Valid JSON may still contain a damaged journal or a no-longer-Human
    // source. Both must fail closed even when a Session cached its owner
    // before an independent SQLite handle changed the committed metadata.
    let original_metadata = read_forward_metadata_from_disk(&root, source);
    let valid_batch = original_metadata["humanForwardBatches"][0].clone();
    let mut wrong_version = valid_batch.clone();
    wrong_version["version"] = json!(2);
    let mut wrong_source = valid_batch.clone();
    wrong_source["sourceConversationId"] = json!("wrong-source");
    let mut wrong_nonce = valid_batch.clone();
    wrong_nonce["destinations"][0]["clientNonce"] = json!("wrong-nonce");
    let mut invalid_documents = Vec::new();
    for (label, journal) in [
        ("journal-null", serde_json::Value::Null),
        ("journal-object", json!({"not":"a batch array"})),
        ("journal-version", json!([wrong_version])),
        ("journal-source", json!([wrong_source])),
        ("journal-nonce", json!([wrong_nonce])),
        ("journal-duplicate", json!([valid_batch.clone(), valid_batch])),
    ] {
        let mut damaged = original_metadata.clone();
        damaged["humanForwardBatches"] = journal;
        invalid_documents.push((label, damaged));
    }
    let mut wrong_kind = original_metadata.clone();
    wrong_kind["conversationKind"] = json!("agent");
    invalid_documents.push(("source-kind", wrong_kind));
    let mut wrong_identity = original_metadata.clone();
    wrong_identity["conversationId"] = json!("different-conversation");
    invalid_documents.push(("source-identity", wrong_identity));
    for cached_owner in [false, true] {
        for (label, damaged) in &invalid_documents {
            db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", [&original_hex]).unwrap();
            let resumed = human_message_fixture_runtime(&root, &server.url);
            if cached_owner {
                resumed.open_human_conversation_db_owner(source)
                    .expect("cache the authentic source before independent corruption");
            }
            let damaged_hex = damaged.to_string().as_bytes().iter()
                .map(|byte| format!("{byte:02x}")).collect::<String>();
            db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", [&damaged_hex])
                .expect("inject valid JSON but invalid source/journal");
            let target = args["destinationConversationIds"][1].as_str().unwrap();
            let error = resumed.sync_human_conversation(target)
                .expect_err("damaged source or journal cannot resume dispatch");
            assert!(!error.is_empty(), "{label}: fail closed for both cold/cached owner");
            assert_eq!(server.post_count(), 1, "{label}: never retry an unverified forward");
            assert_eq!(server.history_count(), 0, "{label}: no later history after a corrupt source");
            let observed: String = db.query_row(
                "SELECT value FROM kv WHERE key='metadata'", [], |row| row.get(0)
            ).unwrap();
            assert_eq!(observed, damaged_hex, "{label}: recovery must not rewrite corrupt metadata");
            resumed.shutdown();
            drop(resumed);
        }
    }
    db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", [&original_hex]).unwrap();
    drop(db);
    let restored = human_message_fixture_runtime(&root, &server.url);
    restored.sync_human_conversation(args["destinationConversationIds"][1].as_str().unwrap())
        .expect("recover original intent after repairing injected metadata");
    assert_forward_settled(&restored, &args);
    assert_eq!(server.post_count(), 3);
    assert_eq!(server.state.lock().unwrap().accepted.len(), 2);
    restored.shutdown(); drop(restored);
    server.finish(); fs::remove_dir_all(root).unwrap();
}

