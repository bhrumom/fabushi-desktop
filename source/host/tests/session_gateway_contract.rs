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
        let sync_body = json!({"success":true,"data":{"messages":[]}}).to_string();
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
}

fn spawn_human_message_fixture(
    responses: Vec<HumanMessageFixtureReply>,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind Human message fixture");
    listener.set_nonblocking(true).expect("bound Human fixture accept");
    let address = listener.local_addr().expect("Human fixture address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
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
