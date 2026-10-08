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
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind messaging fixture");
    let address = listener.local_addr().expect("fixture address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for page_index in 0..3 {
            let (mut stream, _) = listener.accept().expect("accept messaging request");
            let mut request = Vec::new();
            let mut chunk = [0_u8; 4096];
            loop {
                let read = stream.read(&mut chunk).expect("read messaging request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            requests.push(String::from_utf8_lossy(&request).into_owned());

            let (low, high) = match page_index {
                0 => (202_u64, 401_u64),
                1 => (2_u64, 201_u64),
                2 => (1_u64, 1_u64),
                _ => unreachable!(),
            };
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
            let body = json!({
                "success": true,
                "data": { "messages": messages }
            })
            .to_string();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .expect("write messaging response");
            stream.flush().expect("flush messaging response");
        }
        requests
    });
    (format!("http://{address}"), handle)
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
    let mut request = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut header_end = None;
    let mut content_length = 0usize;
    loop {
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
            "remoteCreatedAt": "2026-10-05T00:59:59Z"
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
            false,
            None,
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
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("GET /api/social/messages?contactId=2&limit=200 "));
    assert!(requests[1].contains("&before="));
    assert!(requests[2].contains("&before="));

    runtime.shutdown();
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
