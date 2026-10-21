use std::sync::Mutex;

use mahayana_host_runtime::extensions::inference::provider_session::ProviderSessionError;
use mahayana_host_runtime::runner::box_tool_access::{
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest, RunnerBoxWriteRequest,
};
use mahayana_host_runtime::runner::large_output_spill::{
    AGENT_TOOLS_DIR, MAX_OUTPUT_FILE_SIZE, MCP_TEXT_FILE_THRESHOLD_BYTES,
    is_large_output_spill_enabled_from, maybe_spill_mcp_text_result,
    bound_mcp_inline_text,
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
    let spilled = maybe_spill_mcp_text_result(&box_port, original, "tool-call-7", 4);

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
        maybe_spill_mcp_text_result(&box_port, small.clone(), "small", 100),
        small
    );
    let errored = json!({
        "isError":true,
        "content":[{"type":"text","text":"x".repeat(100)}]
    });
    assert_eq!(
        maybe_spill_mcp_text_result(&box_port, errored.clone(), "error", 1),
        errored
    );
    let large = json!({"content":[{"type":"text","text":"x".repeat(100)}]});
    assert_eq!(
        maybe_spill_mcp_text_result(&FailingBox, large.clone(), "failed", 1),
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
    );
    let writes = box_port.writes.lock().expect("writes");
    assert_eq!(writes[0].data.len(), MAX_OUTPUT_FILE_SIZE);
}

#[test]
fn utf8_artifacts_obey_byte_limits_and_admit_partial_output() {
    let box_port = RecordingBox::default();
    let text = format!("HEAD{}TAIL", "中文🙂".repeat(MAX_OUTPUT_FILE_SIZE / 3));
    let original_bytes = text.len();
    let result = maybe_spill_mcp_text_result(&box_port,
        json!({"content":[{"type":"text","text":text}]}), "utf8", 1);
    let writes = box_port.writes.lock().unwrap();
    assert!(writes[0].data.len() <= MAX_OUTPUT_FILE_SIZE);
    let stored = std::str::from_utf8(&writes[0].data).unwrap();
    assert!(stored.starts_with("HEAD") && stored.ends_with("TAIL"));
    assert!(stored.contains("OUTPUT TRUNCATED"));
    assert_eq!(result["content"][0]["outputLocation"]["truncated"], true);
    assert_eq!(result["content"][0]["outputLocation"]["originalSizeBytes"], original_bytes);
}

#[test]
fn failed_spill_and_large_errors_have_bounded_explicitly_partial_context() {
    for error in [true, false] {
        let text = format!("HEAD{}TAIL", "🙂".repeat(40_000));
        let result = maybe_spill_mcp_text_result(&FailingBox,
            json!({"isError":error,"content":[{"type":"text","text":text},{"type":"image","data":"unchanged"}]}), "failed-utf8", 1);
        assert_eq!(result["isError"], error);
        let inline = result["content"][0]["text"].as_str().unwrap();
        assert!(inline.len() <= MCP_TEXT_FILE_THRESHOLD_BYTES);
        assert!(inline.starts_with("HEAD") && inline.ends_with("TAIL"));
        assert_eq!(result["outputTruncation"]["truncated"], true);
        assert_eq!(result["outputTruncation"]["materialized"], false);
        assert_eq!(result["content"][1]["data"], "unchanged");
    }
    let bounded = bound_mcp_inline_text(json!({"content":[{"type":"text","text":"🙂".repeat(1000)}]}), 7);
    assert!(bounded["content"][0]["text"].as_str().unwrap().len() <= 7);
    assert_eq!(bounded["outputTruncation"]["truncated"], true);
}
