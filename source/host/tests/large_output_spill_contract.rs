use std::sync::Mutex;

use mahayana_host_runtime::extensions::inference::provider_session::ProviderSessionError;
use mahayana_host_runtime::runner::box_tool_access::{
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest, RunnerBoxWriteRequest,
};
use mahayana_host_runtime::runner::large_output_spill::{
    AGENT_TOOLS_DIR, MAX_OUTPUT_FILE_SIZE, MCP_TEXT_FILE_THRESHOLD_BYTES,
    is_large_output_spill_enabled_from, maybe_spill_mcp_text_result,
};
use serde_json::{Value, json};

#[derive(Default)]
struct RecordingBox {
    writes: Mutex<Vec<RunnerBoxWriteRequest>>,
}

impl RunnerBoxResourcePort for RecordingBox {
    fn execute_shell(
        &self,
        _request: RunnerBoxShellRequest,
    ) -> Result<Value, ProviderSessionError> {
        unreachable!("spill contract does not execute Shell")
    }

    fn execute_read(
        &self,
        _request: RunnerBoxReadRequest,
    ) -> Result<Value, ProviderSessionError> {
        unreachable!("spill contract does not execute Read")
    }

    fn execute_write(
        &self,
        request: RunnerBoxWriteRequest,
    ) -> Result<(), ProviderSessionError> {
        self.writes.lock().expect("writes").push(request);
        Ok(())
    }
}

struct FailingBox;

impl RunnerBoxResourcePort for FailingBox {
    fn execute_shell(
        &self,
        _request: RunnerBoxShellRequest,
    ) -> Result<Value, ProviderSessionError> {
        unreachable!("spill contract does not execute Shell")
    }

    fn execute_read(
        &self,
        _request: RunnerBoxReadRequest,
    ) -> Result<Value, ProviderSessionError> {
        unreachable!("spill contract does not execute Read")
    }

    fn execute_write(
        &self,
        _request: RunnerBoxWriteRequest,
    ) -> Result<(), ProviderSessionError> {
        Err(ProviderSessionError::Tool("box unavailable".into()))
    }
}

#[test]
fn large_mcp_text_is_materialized_into_agent_box_once() {
    let box_port = RecordingBox::default();
    let original = json!({
        "isError": false,
        "content": [
            {"type":"text","text":"alpha"},
            {"type":"image","data":"opaque"},
            {"type":"text","text":"beta"}
        ]
    });
    let spilled = maybe_spill_mcp_text_result(&box_port, original, "tool-call-7", 4)
        .expect("confirmed spill");

    let content = spilled["content"].as_array().expect("content");
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[0]["text"], "");
    assert!(content[0]["outputLocation"]["filePath"]
        .as_str()
        .expect("path")
        .starts_with(AGENT_TOOLS_DIR));
    assert_eq!(content[1]["type"], "image");

    let writes = box_port.writes.lock().expect("writes");
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].tool_call_id, "tool-call-7");
    assert_eq!(writes[0].data, b"alpha\n\nbeta");
    assert_eq!(
        content[0]["outputLocation"]["sizeBytes"].as_u64(),
        Some(writes[0].data.len() as u64)
    );
    assert_eq!(content[0]["outputLocation"]["lineCount"], 3);
}

#[test]
fn small_error_and_failed_write_results_stay_inline() {
    let box_port = RecordingBox::default();
    let small = json!({"content":[{"type":"text","text":"short"}]});
    assert_eq!(
        maybe_spill_mcp_text_result(&box_port, small.clone(), "small", 100).expect("small result"),
        small
    );
    let errored = json!({
        "isError":true,
        "content":[{"type":"text","text":"x".repeat(100)}]
    });
    assert_eq!(
        maybe_spill_mcp_text_result(&box_port, errored.clone(), "error", 1).expect("existing error result"),
        errored
    );
    let large = json!({"content":[{"type":"text","text":"x".repeat(100)}]});
    assert_eq!(
        maybe_spill_mcp_text_result(&FailingBox, large.clone(), "failed", 1).expect("known refusal keeps inline output"),
        large
    );
    assert!(box_port.writes.lock().expect("writes").is_empty());
}

#[test]
fn spill_policy_matches_frozen_threshold_disable_and_cap_contracts() {
    assert_eq!(MCP_TEXT_FILE_THRESHOLD_BYTES, 40_000);
    assert_eq!(MAX_OUTPUT_FILE_SIZE, 1_000_000);
    assert!(is_large_output_spill_enabled_from(None));
    assert!(is_large_output_spill_enabled_from(Some("0")));
    assert!(!is_large_output_spill_enabled_from(Some("1")));

    let box_port = RecordingBox::default();
    let oversized = "z".repeat(MAX_OUTPUT_FILE_SIZE + 10);
    let _ = maybe_spill_mcp_text_result(
        &box_port,
        json!({"content":[{"type":"text","text":oversized}]}),
        "cap",
        1,
    ).expect("bounded spill");
    let writes = box_port.writes.lock().expect("writes");
    assert_eq!(writes[0].data.len(), MAX_OUTPUT_FILE_SIZE);
}

#[derive(Default)]
struct UnknownWriteBox {
    writes: Mutex<Vec<RunnerBoxWriteRequest>>,
}

impl RunnerBoxResourcePort for UnknownWriteBox {
    fn execute_shell(
        &self,
        _request: RunnerBoxShellRequest,
    ) -> Result<Value, ProviderSessionError> {
        unreachable!("spill must not execute Shell")
    }

    fn execute_read(
        &self,
        _request: RunnerBoxReadRequest,
    ) -> Result<Value, ProviderSessionError> {
        unreachable!("spill must not execute Read")
    }

    fn execute_write(
        &self,
        request: RunnerBoxWriteRequest,
    ) -> Result<(), ProviderSessionError> {
        self.writes.lock().expect("writes").push(request);
        Err(ProviderSessionError::ToolOutcomeUnknown("box unavailable".into()))
    }
}

#[test]
fn unknown_spill_write_is_not_published_as_inline_success_or_retried() {
    let box_port = UnknownWriteBox::default();
    let original = json!({"content":[{"type":"text","text":"large tool output"}]});
    let result = maybe_spill_mcp_text_result(&box_port, original, "unknown-spill", 1);
    match result {
        Err(ProviderSessionError::ToolOutcomeUnknown(message)) => {
            assert_eq!(message, "box unavailable");
        }
        other => panic!("unknown write must not publish a successful result: {other:?}"),
    }
    let writes = box_port.writes.lock().expect("writes");
    assert_eq!(writes.len(), 1, "an ambiguous write must never be replayed");
    assert_eq!(writes[0].tool_call_id, "unknown-spill");
    assert_eq!(writes[0].data, b"large tool output");
    assert!(writes[0].path.starts_with(".sand/tools/"));
}
