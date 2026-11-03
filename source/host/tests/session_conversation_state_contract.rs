use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_db_schema::AGENT_DB_SCHEMA;
use mahayana_host_runtime::extensions::session::agent_db_transcript_pages::{
    TranscriptPageQuery, TranscriptWindowQuery,
};
use mahayana_host_runtime::extensions::session::agent_db::compare_and_set_persisted_latest_root_blob_id;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::session_conversation_state::{
    ConversationOutlineItem, SessionConversationState,
};
use mahayana_host_runtime::runner::agent_v1_wire::{
    AgentToolCall, SendMessageArgs, SendMessageAttachment, SendMessageError, SendMessageResult,
    SendMessageSuccess, SendMessageText, SendMessageToolCall, agent_tool_call, send_message_args,
    send_message_result,
};
use prost::Message;
use rusqlite::params;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-session-conversation-state-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn seed(db_path: &std::path::Path) {
    let db = rusqlite::Connection::open(db_path).expect("db");
    db.execute_batch(AGENT_DB_SCHEMA).expect("schema");
    for (id, entry) in [
        ("root", json!({"id":"root","kind":"message","role":"user","content":"root","timestampMs":10})),
        ("branch-1", json!({"id":"branch-1","kind":"message","role":"assistant","content":"one","branched":true,"replyTo":"root","timestampMs":11})),
        ("branch-2", json!({"id":"branch-2","kind":"message","role":"assistant","content":"two","branched":true,"replyTo":"branch-1","timestampMs":12})),
        ("other-branch", json!({"id":"other-branch","kind":"message","role":"assistant","content":"other","branched":true,"replyTo":"other-root","timestampMs":13})),
        ("tool", json!({"id":"tool","kind":"tool-call","name":"search","timestampMs":14})),
        ("notice", json!({"id":"notice","kind":"notice","text":"notice","timestampMs":15})),
    ] {
        db.execute(
            "INSERT INTO transcript_entries (id, entry) VALUES (?1, ?2)",
            params![id, entry.to_string()],
        )
        .expect("entry");
    }
}

#[test]
fn transcript_read_owner_matches_frozen_page_window_tail_and_thread_semantics() {
    let root = temp_root("reads");
    let agent_dir = root.join("agent-a");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    seed(&db_path);
    let state = SessionConversationState::new(500);

    let all = state
        .read_agent_transcript_entries(&db_path)
        .expect("all entries");
    assert_eq!(all.len(), 6);

    let page = state
        .read_agent_transcript_page(
            &db_path,
            TranscriptPageQuery {
                before_seq: None,
                since_ms: None,
                until_ms: 100,
                limit: 5,
            },
        )
        .expect("page");
    assert_eq!(
        page.entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["root"]
    );

    let window = state
        .read_agent_transcript_window(
            &db_path,
            TranscriptWindowQuery {
                before_seq: None,
                limit: 10,
            },
        )
        .expect("window");
    assert_eq!(
        window
            .entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["root", "notice"]
    );
    assert_eq!(
        window.thread_counts,
        BTreeMap::from([("root".to_string(), 2usize)])
    );

    let tail = state
        .read_agent_transcript_tail(
            &db_path,
            TranscriptWindowQuery {
                before_seq: None,
                limit: 10,
            },
        )
        .expect("tail");
    assert_eq!(tail.entries.len(), 6);

    let thread = state
        .read_agent_thread(&db_path, "root")
        .expect("thread");
    assert_eq!(
        thread
            .entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["root", "branch-1", "branch-2"]
    );
    assert!(
        state
            .read_agent_thread(&db_path, "missing")
            .expect("missing thread")
            .entries
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}


fn encode_varint(mut value: u64, output: &mut Vec<u8>) {
    while value >= 0x80 {
        output.push((value as u8) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

fn push_bytes(field: u64, value: &[u8], output: &mut Vec<u8>) {
    encode_varint((field << 3) | 2, output);
    encode_varint(value.len() as u64, output);
    output.extend_from_slice(value);
}

fn push_string(field: u64, value: &str, output: &mut Vec<u8>) {
    push_bytes(field, value.as_bytes(), output);
}

fn push_varint(field: u64, value: u64, output: &mut Vec<u8>) {
    encode_varint(field << 3, output);
    encode_varint(value, output);
}

fn blob_id(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

#[test]
fn durable_outline_resolves_frozen_user_steps_send_message_task_and_shell_semantics() {
    let root = temp_root("outline");
    let agents = root.join("agents");
    let runtime = std::sync::Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let record = runtime.materialize_new_session(None, "user", None).expect("agent");
    let store = runtime.create_agent_blob_store(&record.id).expect("blob store");

    let mut user = Vec::new();
    push_string(1, "[SAND_HIDDEN_PROMPT][SAND_TRUSTED_AUTOMATION_PROMPT]secret", &mut user);
    push_string(2, "message-1", &mut user);
    let user_id = blob_id(&user);
    futures::executor::block_on(store.set_blob(&(), &user_id, &user)).expect("user blob");

    let mut assistant = Vec::new();
    push_string(1, "assistant text", &mut assistant);
    let mut assistant_step = Vec::new();
    push_bytes(1, &assistant, &mut assistant_step);
    let assistant_step_id = blob_id(&assistant_step);
    futures::executor::block_on(store.set_blob(&(), &assistant_step_id, &assistant_step))
        .expect("assistant step");

    let mut thinking = Vec::new();
    push_string(1, "thinking", &mut thinking);
    push_varint(2, 17, &mut thinking);
    let mut thinking_step = Vec::new();
    push_bytes(3, &thinking, &mut thinking_step);
    let thinking_step_id = blob_id(&thinking_step);
    futures::executor::block_on(store.set_blob(&(), &thinking_step_id, &thinking_step))
        .expect("thinking step");

    let mut send_text = Vec::new();
    push_string(1, "sent text", &mut send_text);
    let mut send_args = Vec::new();
    push_bytes(1, &send_text, &mut send_args);
    let mut send_call = Vec::new();
    push_bytes(1, &send_args, &mut send_call);
    let send_result = send_success("sent-message-1").encode_to_vec();
    push_bytes(2, &send_result, &mut send_call);
    let mut send_tool = Vec::new();
    push_bytes(55, &send_call, &mut send_tool);
    let mut send_step = Vec::new();
    push_bytes(2, &send_tool, &mut send_step);
    let send_step_id = blob_id(&send_step);
    futures::executor::block_on(store.set_blob(&(), &send_step_id, &send_step))
        .expect("send-message step");

    let mut task_args = Vec::new();
    push_string(1, "  investigate  ", &mut task_args);
    let mut task_error = Vec::new();
    push_string(1, "task failed", &mut task_error);
    let mut task_result = Vec::new();
    push_bytes(2, &task_error, &mut task_result);
    let mut task_call = Vec::new();
    push_bytes(1, &task_args, &mut task_call);
    push_bytes(2, &task_result, &mut task_call);
    let mut task_tool = Vec::new();
    push_bytes(19, &task_call, &mut task_tool);
    let mut task_step = Vec::new();
    push_bytes(2, &task_tool, &mut task_step);
    let task_step_id = blob_id(&task_step);
    futures::executor::block_on(store.set_blob(&(), &task_step_id, &task_step))
        .expect("task step");

    let mut agent_turn = Vec::new();
    push_bytes(1, &user_id, &mut agent_turn);
    for step in [&assistant_step_id, &thinking_step_id, &send_step_id, &task_step_id] {
        push_bytes(2, step, &mut agent_turn);
    }
    let mut agent_turn_wrapper = Vec::new();
    push_bytes(1, &agent_turn, &mut agent_turn_wrapper);
    let agent_turn_id = blob_id(&agent_turn_wrapper);
    futures::executor::block_on(store.set_blob(&(), &agent_turn_id, &agent_turn_wrapper))
        .expect("agent turn");

    let mut shell_command = Vec::new();
    push_string(1, "pwd", &mut shell_command);
    let shell_command_id = blob_id(&shell_command);
    futures::executor::block_on(store.set_blob(&(), &shell_command_id, &shell_command))
        .expect("shell command");
    let mut shell_output = Vec::new();
    push_string(1, "/tmp", &mut shell_output);
    let shell_output_id = blob_id(&shell_output);
    futures::executor::block_on(store.set_blob(&(), &shell_output_id, &shell_output))
        .expect("shell output");
    let mut shell_turn = Vec::new();
    push_bytes(1, &shell_command_id, &mut shell_turn);
    push_bytes(2, &shell_output_id, &mut shell_turn);
    let mut shell_turn_wrapper = Vec::new();
    push_bytes(2, &shell_turn, &mut shell_turn_wrapper);
    let shell_turn_id = blob_id(&shell_turn_wrapper);
    futures::executor::block_on(store.set_blob(&(), &shell_turn_id, &shell_turn_wrapper))
        .expect("shell turn");

    let mut todo = Vec::new();
    push_string(1, "todo-1", &mut todo);
    push_string(2, "finish parity", &mut todo);
    push_varint(3, 2, &mut todo);
    push_varint(4, 100, &mut todo);
    push_varint(5, 200, &mut todo);
    push_string(6, "dependency-a", &mut todo);
    let todo_id = blob_id(&todo);
    futures::executor::block_on(store.set_blob(&(), &todo_id, &todo)).expect("todo");

    let mut summary = Vec::new();
    push_string(1, "conversation summary", &mut summary);
    let summary_id = blob_id(&summary);
    futures::executor::block_on(store.set_blob(&(), &summary_id, &summary)).expect("summary");

    let mut root_blob = Vec::new();
    push_bytes(3, &todo_id, &mut root_blob);
    push_bytes(6, &summary_id, &mut root_blob);
    push_bytes(8, &agent_turn_id, &mut root_blob);
    push_bytes(8, &shell_turn_id, &mut root_blob);
    let root_id = blob_id(&root_blob);
    futures::executor::block_on(store.set_blob(&(), &root_id, &root_blob)).expect("root blob");
    assert!(compare_and_set_persisted_latest_root_blob_id(
        &record.db_path,
        500,
        &[],
        &root_id,
    ).expect("root CAS"));

    let resolved = runtime
        .read_agent_conversation_state(&record.id)
        .expect("resolved state")
        .expect("materialized state");
    assert_eq!(resolved.turns.len(), 2);
    assert_eq!(resolved.todos.len(), 1);
    assert_eq!(resolved.todos[0].id, "todo-1");
    assert_eq!(resolved.todos[0].content, "finish parity");
    assert_eq!(resolved.todos[0].status, 2);
    assert_eq!(resolved.todos[0].created_at, 100);
    assert_eq!(resolved.todos[0].updated_at, 200);
    assert_eq!(resolved.todos[0].dependencies, vec!["dependency-a".to_string()]);
    assert_eq!(resolved.summary.as_deref(), Some("conversation summary"));

    let outline = runtime.read_agent_outline(&record.id).expect("outline");
    assert_eq!(outline.len(), 6);
    assert!(matches!(
        &outline[0],
        ConversationOutlineItem::User { id, text, hidden }
            if id == "outline-user-0" && text == "secret" && *hidden
    ));
    assert!(matches!(
        &outline[1],
        ConversationOutlineItem::AssistantText { id, text }
            if id == "outline-0-0" && text == "assistant text"
    ));
    assert!(matches!(
        &outline[2],
        ConversationOutlineItem::Thinking { id, text, duration_ms }
            if id == "outline-0-1" && text == "thinking" && *duration_ms == Some(17)
    ));
    assert!(matches!(
        &outline[3],
        ConversationOutlineItem::SendMessage { id, message }
            if id == "outline-0-2"
                && message == &json!({"type":"text","content":"sent text"})
    ));
    assert!(matches!(
        &outline[4],
        ConversationOutlineItem::ToolCall { id, name, status, summary }
            if id == "outline-0-3"
                && name == "Task"
                && status == "failed"
                && summary.as_deref() == Some("task failed")
    ));
    assert!(matches!(
        &outline[5],
        ConversationOutlineItem::ToolCall { id, name, status, summary }
            if id == "outline-shell-1"
                && name == "shellToolCall"
                && status == "done"
                && summary.as_deref() == Some("pwd")
    ));

    let prepared = runtime
        .prepare_existing_agent(&record.id)
        .expect("prepare existing agent")
        .expect("prepared agent");
    assert_eq!(
        prepared
            .transcript_tail
            .entries
            .iter()
            .map(|entry| entry.get("kind").and_then(serde_json::Value::as_str).unwrap_or(""))
            .collect::<Vec<_>>(),
        vec!["send-message", "tool-call", "tool-call"]
    );

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}

fn send_text_args(content: &str) -> SendMessageArgs {
    SendMessageArgs {
        message: Some(send_message_args::Message::Text(SendMessageText {
            content: content.to_string(),
        })),
    }
}

fn send_attachment_args(url: &str, alt: Option<&str>) -> SendMessageArgs {
    SendMessageArgs {
        message: Some(send_message_args::Message::Attachment(SendMessageAttachment {
            url: url.to_string(),
            alt: alt.map(ToOwned::to_owned),
        })),
    }
}

fn send_success(message_id: &str) -> SendMessageResult {
    SendMessageResult {
        result: Some(send_message_result::Result::Success(SendMessageSuccess {
            timestamp: 200,
            message_id: message_id.to_string(),
        })),
    }
}

fn send_error() -> SendMessageResult {
    SendMessageResult {
        result: Some(send_message_result::Result::Error(SendMessageError {
            error: "sink refused the send".into(),
        })),
    }
}

fn frozen_send_tool(args: Option<SendMessageArgs>, result: Option<SendMessageResult>) -> Vec<u8> {
    let completed_at_ms = result.is_some().then_some(200);
    AgentToolCall {
        tool: Some(agent_tool_call::Tool::SendMessageToolCall(SendMessageToolCall {
            args,
            result,
        })),
        tool_call_id: Some("send-call-1".into()),
        started_at_ms: Some(100),
        completed_at_ms,
    }
    .encode_to_vec()
}

fn send_payload_tool(payload: &[u8]) -> Vec<u8> {
    let mut tool = Vec::new();
    push_bytes(55, payload, &mut tool);
    push_string(57, "send-call-1", &mut tool);
    push_varint(59, 100, &mut tool);
    push_varint(60, 200, &mut tool);
    tool
}

fn raw_send_payload(args: &[u8], result: &[u8]) -> Vec<u8> {
    let mut payload = Vec::new();
    push_bytes(1, args, &mut payload);
    push_bytes(2, result, &mut payload);
    payload
}

fn nested_message_fields(fields: &[u64], value: &[u8]) -> Vec<u8> {
    let mut bytes = value.to_vec();
    for &field in fields.iter().rev() {
        let mut parent = Vec::new();
        push_bytes(field, &bytes, &mut parent);
        bytes = parent;
    }
    bytes
}

fn nested_task_tools(cycles: usize, leaf: &[u8]) -> Vec<u8> {
    // Frozen agent_pb.ts: ToolCall.task_tool_call(19) -> result(2) ->
    // success(1) -> conversation_steps(1) -> ConversationStep.tool_call(2).
    // Each complete cycle descends exactly five messages from its ToolCall.
    let mut tool = leaf.to_vec();
    for _ in 0..cycles {
        tool = nested_message_fields(&[19, 2, 1, 1, 2], &tool);
    }
    tool
}

fn unknown_group_chain(depth: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    for _ in 0..depth {
        encode_varint((200 << 3) | 3, &mut bytes);
    }
    for _ in 0..depth {
        encode_varint((200 << 3) | 4, &mut bytes);
    }
    bytes
}

fn persisted_transcript_rows(db_path: &std::path::Path) -> Vec<(i64, String)> {
    let db = rusqlite::Connection::open(db_path).expect("read transcript database");
    let mut query = db
        .prepare("SELECT seq, entry FROM transcript_entries ORDER BY seq")
        .expect("transcript query");
    query
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("transcript rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("persisted transcript rows")
}

fn assert_tool_projection_readback(
    runtime: &ProductionSessionWorkers,
    agent_id: &str,
    blobs: &[Vec<u8>],
    expected_outline: &[ConversationOutlineItem],
    expected_entries: &[Value],
) {
    assert_eq!(runtime.read_agent_outline(agent_id).expect("outline"), expected_outline);
    let resolved = runtime.read_agent_conversation_state(agent_id)
        .expect("read canonical state").expect("canonical turn remains readable");
    assert_eq!(resolved.turns.len(), 1);
    assert_eq!(resolved.turns[0].user_message_id, "send-fixture-user");
    assert_eq!(resolved.turns[0].items, expected_outline);

    let store = runtime.create_agent_blob_store(agent_id).expect("read canonical blobs");
    for bytes in blobs {
        assert_eq!(
            futures::executor::block_on(store.get_blob(&(), &blob_id(bytes)))
                .expect("read stored blob"),
            Some(bytes.clone()),
            "projection and recovery must preserve the exact canonical bytes",
        );
    }
    assert_eq!(
        runtime.read_agent_transcript_entries(agent_id).expect("owner transcript"),
        expected_entries,
    );
}

fn assert_durable_tool_projection(
    label: &str,
    tool: Vec<u8>,
    expected_item: Option<ConversationOutlineItem>,
    expected_entry: Option<Value>,
) {
    let root = temp_root(label);
    let agents = root.join("agents");
    let runtime = ProductionSessionWorkers::with_agents_root(&agents, 500);
    let record = runtime.materialize_new_session(None, "user", None).expect("agent");
    let store = runtime.create_agent_blob_store(&record.id).expect("blob store");

    let mut user = Vec::new();
    push_string(1, "[SAND_HIDDEN_PROMPT]send fixture", &mut user);
    push_string(2, "send-fixture-user", &mut user);
    let mut step = Vec::new();
    push_bytes(2, &tool, &mut step);
    let mut agent_turn = Vec::new();
    push_bytes(1, &blob_id(&user), &mut agent_turn);
    push_bytes(2, &blob_id(&step), &mut agent_turn);
    let mut turn = Vec::new();
    push_bytes(1, &agent_turn, &mut turn);
    let mut state = Vec::new();
    push_bytes(8, &blob_id(&turn), &mut state);
    let root_id = blob_id(&state);
    let blobs = [user, step, turn, state];
    for bytes in &blobs {
        futures::executor::block_on(store.set_blob(&(), &blob_id(bytes), bytes))
            .expect("write canonical blob");
    }
    assert!(compare_and_set_persisted_latest_root_blob_id(
        &record.db_path, 500, &[], &root_id,
    ).expect("root CAS"));

    let mut expected_outline = vec![ConversationOutlineItem::User {
        id: "outline-user-0".into(),
        text: "send fixture".into(),
        hidden: true,
    }];
    expected_outline.extend(expected_item);
    let expected_entries = expected_entry.into_iter().collect::<Vec<_>>();
    assert!(persisted_transcript_rows(&record.db_path).is_empty());
    assert_tool_projection_readback(&runtime, &record.id, &blobs, &expected_outline, &[]);

    let prepared = runtime.prepare_existing_agent(&record.id)
        .expect("prepare existing agent").expect("prepared agent");
    assert_eq!(prepared.persisted_root_blob_id, root_id);
    assert_eq!(prepared.transcript_tail.entries, expected_entries);
    drop(prepared);
    let rows = persisted_transcript_rows(&record.db_path);
    assert_eq!(
        rows.iter().map(|(_, entry)| serde_json::from_str::<Value>(entry)
            .expect("persisted JSON")).collect::<Vec<_>>(),
        expected_entries,
        "only qualified outline items may become real SQL backfill rows",
    );
    assert_tool_projection_readback(
        &runtime, &record.id, &blobs, &expected_outline, &expected_entries,
    );
    runtime.prepare_existing_agent(&record.id).expect("repeat preparation")
        .expect("repeat prepared agent");
    assert_eq!(persisted_transcript_rows(&record.db_path), rows);
    drop(store);
    runtime.shutdown();
    drop(runtime);

    let reopened = ProductionSessionWorkers::with_agents_root(&agents, 500);
    assert_tool_projection_readback(
        &reopened, &record.id, &blobs, &expected_outline, &expected_entries,
    );
    let prepared = reopened.prepare_existing_agent(&record.id)
        .expect("fresh owner preparation").expect("reopened agent");
    assert_eq!(prepared.persisted_root_blob_id, root_id);
    assert_eq!(prepared.transcript_tail.entries, expected_entries);
    drop(prepared);
    assert_eq!(persisted_transcript_rows(&record.db_path), rows);
    assert_tool_projection_readback(
        &reopened, &record.id, &blobs, &expected_outline, &expected_entries,
    );
    reopened.shutdown();
    drop(reopened);
    fs::remove_dir_all(root).expect("remove fixture");
}

fn assert_durable_send_projection(label: &str, tool: Vec<u8>, message: Option<Value>) {
    let item = message.as_ref().map(|message| ConversationOutlineItem::SendMessage {
        id: "outline-0-0".into(),
        message: message.clone(),
    });
    // SGF preserves the existing recovery identity contract. Reconstructing an
    // original SQL message ID from an outline remains a separate responsibility.
    let entry = message.map(|message| json!({
        "id": "recovered-outline-0-0", "kind": "send-message", "message": message,
    }));
    assert_durable_tool_projection(label, tool, item, entry);
}

#[test]
fn generated_send_message_text_success_survives_session_backfill_reopen() {
    assert_durable_send_projection(
        "send-text-success",
        frozen_send_tool(Some(send_text_args("sent text 東京")), Some(send_success("sql-message-1"))),
        Some(json!({"type":"text","content":"sent text 東京"})),
    );

    // ToolCall54 and HookAdditionalContext string fields 1/2 are declared by
    // agent_pb.ts and hook_additional_context_pb.ts, even though prost omits 54.
    let mut context = Vec::new();
    push_string(1, "afterToolUse", &mut context);
    push_string(2, "valid metadata 東京", &mut context);
    let mut metadata = nested_message_fields(&[54], &context);
    // Unknown length-delimited extensions remain opaque, even for these bytes.
    push_bytes(200, &[0x80, 0xff, 0x00], &mut metadata);

    // coordinator_tools_pb.ts: CreateAgentToolCall.args(1).labels(9) is a
    // string/string map; ToolCall.create_agent_tool_call is field 76.
    let mut labels_entry = Vec::new();
    push_string(1, "platform", &mut labels_entry);
    push_string(2, "valid label", &mut labels_entry);

    // mcp_tool_pb.ts/mcp_exec_pb.ts: ToolCall15.args(1).args(2) is a
    // string/google.protobuf.Value map. Value.string_value is field 3.
    let mut mcp_entry = Vec::new();
    push_string(1, "argument", &mut mcp_entry);
    push_bytes(2, &nested_message_fields(&[3], b"valid value"), &mut mcp_entry);

    // shell_tool_pb.ts/shell_exec_pb.ts: ToolCall1.args(1).parsing_result(8)
    // .redirects(6).destination_fds(2) is repeated uint32, packed or unpacked.
    let mut redirect = Vec::new();
    push_bytes(2, &[1, 2, 0x80, 0x01], &mut redirect);
    push_varint(2, 3, &mut redirect);
    for (label, mut prefix) in [
        ("send-known-metadata-unknown-opaque", metadata),
        ("send-known-scalar-map", nested_message_fields(&[76, 1, 9], &labels_entry)),
        ("send-known-message-map", nested_message_fields(&[15, 1, 2], &mcp_entry)),
        ("send-known-packed-and-unpacked", nested_message_fields(&[1, 1, 8, 6], &redirect)),
        ("send-known-recursion-moderate", nested_task_tools(3, &nested_message_fields(&[54], &context))),
        ("send-known-message-depth-100", nested_task_tools(20, &[])),
        // 19 cycles = 95; McpToolCall/args/map entry/Value/Struct add five.
        ("send-known-map-value-depth-100", nested_task_tools(19, &nested_message_fields(&[15, 1, 2, 2, 5], &[]))),
        // Metadata consumes one descent and each unknown group consumes one.
        ("send-known-metadata-group-depth-100", nested_message_fields(&[54], &unknown_group_chain(99))),
    ] {
        prefix.extend_from_slice(&frozen_send_tool(
            Some(send_text_args("sent text 東京")), Some(send_success("sql-message-1")),
        ));
        assert_durable_send_projection(
            label, prefix, Some(json!({"type":"text","content":"sent text 東京"})),
        );
    }

    // Unknown root groups are one framed field. Numbers 19/55 inside them
    // do not select Task/SendMessage, and their byte payloads remain opaque.
    let mut nested_group = Vec::new();
    encode_varint((200 << 3) | 3, &mut nested_group);
    encode_varint((201 << 3) | 3, &mut nested_group);
    push_bytes(19, &[0x80], &mut nested_group);
    push_bytes(55, &[0x80], &mut nested_group);
    encode_varint((201 << 3) | 4, &mut nested_group);
    encode_varint((200 << 3) | 4, &mut nested_group);
    let valid = frozen_send_tool(Some(send_text_args("sent text 東京")), Some(send_success("sql-message-1")));
    for (label, mut prefix, suffix) in [
        ("send-root-group-prefix", unknown_group_chain(1), Vec::new()),
        ("send-root-group-exact-suffix", Vec::new(), vec![0xc3, 0x0c, 0xc4, 0x0c]),
        ("send-root-group-nested-tool-looking-suffix", Vec::new(), nested_group),
        ("send-root-group-depth-100", Vec::new(), unknown_group_chain(100)),
    ] {
        prefix.extend_from_slice(&valid);
        prefix.extend_from_slice(&suffix);
        assert_durable_send_projection(
            label, prefix, Some(json!({"type":"text","content":"sent text 東京"})),
        );
    }
}

#[test]
fn generated_send_message_attachment_success_survives_session_backfill_reopen() {
    for (label, alt, message) in [
        ("send-attachment-alt", Some("添付"), json!({"type":"attachment","url":"attachment://resource-1","alt":"添付"})),
        ("send-attachment-empty-alt", Some(""), json!({"type":"attachment","url":"attachment://resource-1"})),
        ("send-attachment-no-alt", None, json!({"type":"attachment","url":"attachment://resource-1"})),
    ] {
        assert_durable_send_projection(
            label,
            frozen_send_tool(
                Some(send_attachment_args("attachment://resource-1", alt)),
                Some(send_success("sql-attachment-1")),
            ),
            Some(message),
        );
    }
}

#[test]
fn generated_send_message_empty_success_id_remains_compatible_after_reopen() {
    assert_durable_send_projection(
        "send-empty-id",
        frozen_send_tool(Some(send_text_args("generic sink success")), Some(send_success(""))),
        Some(json!({"type":"text","content":"generic sink success"})),
    );
}

#[test]
fn generated_send_message_initial_error_and_empty_results_never_backfill() {
    let args = send_text_args("not sent");
    let mut old_args_only = Vec::new();
    push_bytes(55, &SendMessageToolCall {
        args: Some(args.clone()), result: None,
    }.encode_to_vec(), &mut old_args_only);
    for (label, tool) in [
        ("send-initial", frozen_send_tool(Some(args.clone()), None)),
        ("send-old-args-only", old_args_only),
        ("send-error", frozen_send_tool(Some(args.clone()), Some(send_error()))),
        ("send-empty-result", frozen_send_tool(Some(args), Some(SendMessageResult { result: None }))),
    ] {
        assert_durable_send_projection(label, tool, None);
    }
}

#[test]
fn generated_send_message_absent_empty_and_unsupported_arguments_never_backfill() {
    let mut unsupported_args = Vec::new();
    push_bytes(3, b"future-message-variant", &mut unsupported_args);
    let unsupported = raw_send_payload(&unsupported_args, &send_success("sql-id").encode_to_vec());
    for (label, tool) in [
        ("send-absent-args", frozen_send_tool(None, Some(send_success("sql-id")))),
        ("send-empty-args", frozen_send_tool(Some(SendMessageArgs { message: None }), Some(send_success("sql-id")))),
        ("send-unsupported-args", send_payload_tool(&unsupported)),
    ] {
        assert_durable_send_projection(label, tool, None);
    }
}

#[test]
fn generated_send_message_malformed_outer_envelopes_never_backfill() {
    let valid = frozen_send_tool(Some(send_text_args("valid prefix")), Some(send_success("sql-id")));
    let mut truncated_tag = valid.clone();
    truncated_tag.push(0x80);
    let mut invalid_id = valid.clone();
    push_bytes(57, &[0xff], &mut invalid_id);
    let mut overwritten_invalid_id = invalid_id.clone();
    push_string(57, "later-valid-id", &mut overwritten_invalid_id);
    let mut invalid_completed_wire = valid.clone();
    push_bytes(60, b"wrong wire type", &mut invalid_completed_wire);
    let payload = SendMessageToolCall {
        args: Some(send_text_args("valid prefix")), result: Some(send_success("sql-id")),
    }.encode_to_vec();
    let mut truncated_payload = Vec::new();
    encode_varint((55 << 3) | 2, &mut truncated_payload);
    encode_varint(payload.len() as u64 + 1, &mut truncated_payload);
    truncated_payload.extend_from_slice(&payload);
    for (label, tool) in [
        ("send-truncated-outer-tag", truncated_tag),
        ("send-truncated-outer-payload", truncated_payload),
        ("send-invalid-id-utf8", invalid_id),
        ("send-overwritten-invalid-id-utf8", overwritten_invalid_id),
        ("send-invalid-completed-wire", invalid_completed_wire),
    ] {
        assert_durable_send_projection(label, tool, None);
    }

    let mut truncated_context = valid.clone();
    truncated_context.extend_from_slice(&[0xb2, 0x03, 0x01, 0x80]);
    assert_durable_send_projection("send-known-metadata-exact-truncated-suffix", truncated_context, None);
    let mut wrong_context_wire = valid.clone();
    push_varint(54, 0, &mut wrong_context_wire);
    assert_durable_send_projection("send-known-metadata-wrong-wire", wrong_context_wire, None);
    for (label, context) in [
        ("send-known-hook-name-invalid-utf8", vec![0x0a, 0x01, 0xff]),
        ("send-known-hook-content-invalid-utf8", vec![0x12, 0x01, 0xff]),
        ("send-known-hook-name-wrong-wire", vec![0x08, 0x00]),
        ("send-known-hook-content-wrong-wire", vec![0x10, 0x00]),
    ] {
        let mut tool = valid.clone();
        push_bytes(54, &context, &mut tool);
        assert_durable_send_projection(label, tool, None);
    }

    let mut wrong_task_wire = Vec::new();
    push_varint(19, 0, &mut wrong_task_wire);
    for (label, mut prefix) in [
        ("send-known-task-wrong-wire-before-success", wrong_task_wire),
        ("send-known-task-truncated-before-success", nested_message_fields(&[19], &[0x80])),
        // TaskToolCall.args(1).TaskArgs.description(1) is a known string.
        ("send-known-task-invalid-utf8-before-success", nested_message_fields(&[19, 1, 1], &[0xff])),
    ] {
        prefix.extend_from_slice(&valid);
        assert_durable_send_projection(label, prefix, None);
    }

    let mut unmatched_group = Vec::new();
    encode_varint((200 << 3) | 3, &mut unmatched_group);
    let unclosed_group = unmatched_group.clone();
    encode_varint((201 << 3) | 4, &mut unmatched_group);
    let mut end_only_group = Vec::new();
    encode_varint((200 << 3) | 4, &mut end_only_group);
    // agent_pb.ts declares TaskToolCall19 as a message, never a wire-3 group.
    let mut task_group = Vec::new();
    encode_varint((19 << 3) | 3, &mut task_group);
    encode_varint((19 << 3) | 4, &mut task_group);
    for (label, suffix) in [
        ("send-root-group-unmatched", unmatched_group),
        ("send-root-group-unclosed", unclosed_group),
        ("send-root-group-end-only", end_only_group),
        ("send-root-group-depth-101", unknown_group_chain(101)),
        ("send-known-task-root-group-wrong-wire", task_group),
    ] {
        let mut tool = valid.clone();
        tool.extend_from_slice(&suffix);
        assert_durable_send_projection(label, tool, None);
    }
}

#[test]
fn generated_send_message_malformed_nested_payloads_never_backfill() {
    let args = send_text_args("valid prefix").encode_to_vec();
    let success = send_success("sql-id").encode_to_vec();
    let mut truncated_call = raw_send_payload(&args, &success);
    truncated_call.push(0x80);
    let mut truncated_args = args.clone();
    truncated_args.extend_from_slice(&[0x12, 0x02, 0x0a]);
    let mut truncated_result = success.clone();
    truncated_result.push(0x80);
    let mut invalid_text = SendMessageText { content: "valid prefix".into() }.encode_to_vec();
    push_bytes(1, &[0xff], &mut invalid_text);
    let mut invalid_text_args = Vec::new();
    push_bytes(1, &invalid_text, &mut invalid_text_args);
    let mut invalid_attachment = SendMessageAttachment {
        url: "attachment://resource-1".into(), alt: None,
    }.encode_to_vec();
    push_bytes(2, &[0xff], &mut invalid_attachment);
    let mut invalid_attachment_args = Vec::new();
    push_bytes(2, &invalid_attachment, &mut invalid_attachment_args);
    let mut invalid_success = SendMessageSuccess {
        timestamp: 200, message_id: "valid-prefix-id".into(),
    }.encode_to_vec();
    push_bytes(2, &[0xff], &mut invalid_success);
    let mut invalid_success_result = Vec::new();
    push_bytes(1, &invalid_success, &mut invalid_success_result);
    let mut invalid_error = Vec::new();
    push_bytes(1, &[0xff], &mut invalid_error);
    let mut overwritten_invalid_error = Vec::new();
    push_bytes(2, &invalid_error, &mut overwritten_invalid_error);
    overwritten_invalid_error.extend_from_slice(&success);
    for (label, payload) in [
        ("send-truncated-call", truncated_call),
        ("send-truncated-args", raw_send_payload(&truncated_args, &success)),
        ("send-truncated-result", raw_send_payload(&args, &truncated_result)),
        ("send-invalid-text-utf8", raw_send_payload(&invalid_text_args, &success)),
        ("send-invalid-attachment-utf8", raw_send_payload(&invalid_attachment_args, &success)),
        ("send-invalid-success-id-utf8", raw_send_payload(&args, &invalid_success_result)),
        ("send-overwritten-invalid-error-utf8", raw_send_payload(&args, &overwritten_invalid_error)),
    ] {
        assert_durable_send_projection(label, send_payload_tool(&payload), None);
    }

    // These declared metadata/tool fields precede a complete successful SendMessage.
    // Validating only the selected oneof or skipping its unknown prost siblings
    // would incorrectly turn each malformed prefix into a durable sent row.
    let valid = frozen_send_tool(Some(send_text_args("valid final send")), Some(send_success("sql-id")));
    let overflow = [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02];
    let mut overflow_context_tag = overflow.to_vec();
    overflow_context_tag[0] = 0x8a; // Without the overflow check this wraps to string field 1.
    overflow_context_tag.push(0); // Otherwise-valid empty string length.
    let mut overflow_context_length = vec![0x0a];
    overflow_context_length.extend_from_slice(&overflow);
    let mut overflow_duration = vec![0x20]; // TaskSuccess.duration_ms(4), uint64.
    overflow_duration.extend_from_slice(&overflow);
    let mut overlong_duration = vec![0x20];
    overlong_duration.extend_from_slice(&[0x80; 10]);
    overlong_duration.push(0);
    let mut out_of_range_field = Vec::new();
    push_varint(0x2000_0000, 0, &mut out_of_range_field);
    let mut overwritten_invalid_map_value = vec![0x12, 0x01, 0xff];
    push_string(2, "later valid map value", &mut overwritten_invalid_map_value);
    let mut unmatched_group = Vec::new();
    encode_varint((200 << 3) | 3, &mut unmatched_group);
    let unclosed_group = unmatched_group.clone();
    encode_varint((201 << 3) | 4, &mut unmatched_group);
    for (label, mut prefix) in [
        // CreateAgentArgs.labels(9) map entries have string key1/value2.
        ("send-known-map-key-invalid-utf8", nested_message_fields(&[76, 1, 9], &[0x0a, 0x01, 0xff])),
        ("send-known-map-key-wrong-wire", nested_message_fields(&[76, 1, 9], &[0x08, 0x00])),
        ("send-known-map-overwritten-invalid-value", nested_message_fields(&[76, 1, 9], &overwritten_invalid_map_value)),
        ("send-known-map-value-wrong-wire", nested_message_fields(&[76, 1, 9], &[0x10, 0x00])),
        // McpArgs.args(2) map value2 is Value, with string_value(3).
        ("send-known-map-message-invalid-utf8", nested_message_fields(&[15, 1, 2, 2, 3], &[0xff])),
        ("send-known-map-message-wrong-wire", nested_message_fields(&[15, 1, 2], &[0x10, 0x00])),
        ("send-known-packed-truncated", nested_message_fields(&[1, 1, 8, 6, 2], &[0x80])),
        ("send-known-packed-overflow", nested_message_fields(&[1, 1, 8, 6, 2], &overflow)),
        ("send-known-packed-wrong-wire", nested_message_fields(&[1, 1, 8, 6], &[0x15, 0, 0, 0, 0])),
        ("send-known-nested-tag-overflow", nested_message_fields(&[54], &overflow_context_tag)),
        ("send-known-nested-length-overflow", nested_message_fields(&[54], &overflow_context_length)),
        ("send-known-nested-scalar-overflow", nested_message_fields(&[19, 2, 1], &overflow_duration)),
        ("send-known-nested-overlong-varint", nested_message_fields(&[19, 2, 1], &overlong_duration)),
        ("send-known-nested-zero-field", nested_message_fields(&[54], &[0x00, 0x00])),
        ("send-known-nested-out-of-range-field", nested_message_fields(&[54], &out_of_range_field)),
        // The empty metadata child is descent 101 after twenty Task cycles.
        ("send-known-message-depth-101", nested_task_tools(20, &nested_message_fields(&[54], &[]))),
        // Struct.fields(1) is another map entry: 95 + six descents = 101.
        ("send-known-map-entry-depth-101", nested_task_tools(19, &nested_message_fields(&[15, 1, 2, 2, 5, 1], &[]))),
        ("send-known-metadata-group-depth-101", nested_message_fields(&[54], &unknown_group_chain(100))),
        ("send-known-metadata-unmatched-group", nested_message_fields(&[54], &unmatched_group)),
        ("send-known-metadata-unclosed-group", nested_message_fields(&[54], &unclosed_group)),
    ] {
        prefix.extend_from_slice(&valid);
        assert_durable_send_projection(label, prefix, None);
    }
}

#[test]
fn generated_send_message_effective_later_error_never_backfills() {
    let args = send_text_args("not sent").encode_to_vec();
    let success = send_success("earlier-id").encode_to_vec();
    let error = send_error().encode_to_vec();
    let mut both_outcomes = success.clone();
    both_outcomes.extend_from_slice(&error);
    let mut repeated_result = raw_send_payload(&args, &success);
    push_bytes(2, &error, &mut repeated_result);
    let mut repeated_tool = send_payload_tool(&raw_send_payload(&args, &success));
    push_bytes(55, &SendMessageToolCall {
        args: None, result: Some(send_error()),
    }.encode_to_vec(), &mut repeated_tool);
    for (label, tool) in [
        ("send-later-result-oneof-error", send_payload_tool(&raw_send_payload(&args, &both_outcomes))),
        ("send-later-result-field-error", send_payload_tool(&repeated_result)),
        ("send-later-tool-field-error", repeated_tool),
    ] {
        assert_durable_send_projection(label, tool, None);
    }
}

#[test]
fn generated_send_message_repeated_payloads_merge_through_session_backfill_reopen() {
    let mut tool = frozen_send_tool(Some(send_text_args("merged text")), None);
    push_bytes(55, &SendMessageToolCall {
        args: None, result: Some(send_success("merged-id")),
    }.encode_to_vec(), &mut tool);
    // Empty repeated singular submessages do not clear their earlier fields.
    let mut empty_repeats = Vec::new();
    push_bytes(1, &[], &mut empty_repeats);
    push_bytes(2, &[], &mut empty_repeats);
    push_bytes(55, &empty_repeats, &mut tool);
    push_bytes(200, b"unknown envelope field", &mut tool);
    assert_durable_send_projection(
        "send-merged-payloads", tool,
        Some(json!({"type":"text","content":"merged text"})),
    );
}

#[test]
fn generated_send_message_effective_arguments_follow_protobuf_merge_semantics() {
    let success = send_success("sql-id").encode_to_vec();
    let mut repeated_text = SendMessageText { content: "earlier text".into() }.encode_to_vec();
    push_string(1, "effective text", &mut repeated_text);
    let mut text_args = Vec::new();
    push_bytes(1, &repeated_text, &mut text_args);
    push_bytes(1, &[], &mut text_args);
    assert_durable_send_projection(
        "send-effective-text",
        send_payload_tool(&raw_send_payload(&text_args, &success)),
        Some(json!({"type":"text","content":"effective text"})),
    );

    let mut switched_args = raw_send_payload(&send_text_args("earlier text").encode_to_vec(), &success);
    push_bytes(1, &send_attachment_args("attachment://effective", Some("effective alt"))
        .encode_to_vec(), &mut switched_args);
    assert_durable_send_projection(
        "send-effective-attachment", send_payload_tool(&switched_args),
        Some(json!({"type":"attachment","url":"attachment://effective","alt":"effective alt"})),
    );
}

#[test]
fn generated_send_message_intervening_tools_reset_success_before_backfill() {
    for (label, intervening_tool) in [("send-reset-by-task", 19), ("send-reset-by-update", 48)] {
        let mut tool = frozen_send_tool(Some(send_text_args("earlier send")), Some(send_success("earlier-id")));
        push_bytes(intervening_tool, &[], &mut tool);
        push_bytes(55, &SendMessageToolCall {
            args: Some(send_text_args("later initial only")), result: None,
        }.encode_to_vec(), &mut tool);
        assert_durable_send_projection(label, tool, None);
    }
    let mut wrong_task_wire = Vec::new();
    push_varint(19, 0, &mut wrong_task_wire);
    for (label, intervening) in [
        ("send-reset-by-invalid-task-wire", wrong_task_wire),
        ("send-reset-by-invalid-task-payload", nested_message_fields(&[19], &[0x80])),
    ] {
        let mut tool = frozen_send_tool(Some(send_text_args("earlier send")), Some(send_success("earlier-id")));
        tool.extend_from_slice(&intervening);
        push_bytes(55, &SendMessageToolCall {
            args: Some(send_text_args("later initial only")), result: None,
        }.encode_to_vec(), &mut tool);
        assert_durable_send_projection(label, tool, None);
    }
}

#[test]
fn generated_send_message_final_other_tool_keeps_complete_oneof_selection() {
    let mut tool = frozen_send_tool(Some(send_text_args("superseded send")), Some(send_success("earlier-id")));
    let mut task_args = Vec::new();
    push_string(1, "selected task", &mut task_args);
    let mut task = Vec::new();
    push_bytes(1, &task_args, &mut task);
    push_bytes(19, &task, &mut tool);
    assert_durable_tool_projection(
        "send-final-task", tool,
        Some(ConversationOutlineItem::ToolCall {
            id: "outline-0-0".into(), name: "Task".into(),
            status: "done".into(), summary: Some("selected task".into()),
        }),
        Some(json!({
            "id":"recovered-outline-0-0", "kind":"tool-call", "name":"Task",
            "status":"done", "summary":"selected task",
        })),
    );
}
