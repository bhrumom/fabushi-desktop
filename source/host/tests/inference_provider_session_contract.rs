use std::fs;
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedProvider, RoutedProviderOptions, configured_routed_provider,
    decode_sse_stream, run_routed_provider_text,
};
use serde_json::json;

#[test]
fn provider_session_reads_router_settings_and_streams_sse_events() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fabushi-provider-settings-{}-{suffix}.json",
        std::process::id()
    ));
    fs::write(&path, r#"{"router":{"provider":"codex"}}"#)
        .expect("write settings");
    assert_eq!(
        configured_routed_provider(&path),
        Some(RoutedProvider::Codex)
    );
    fs::remove_file(path).expect("remove settings");

    let stream = b"event: ignored\r\ndata: {\"type\":\"response.output_text.delta\",\"delta\":\"hi\"}\r\n\r\ndata: [DONE]\r\n\r\n";
    let mut events = Vec::new();
    decode_sse_stream(Cursor::new(stream.as_slice()), |event| {
        events.push(event);
        Ok(())
    })
    .expect("decode SSE");
    assert_eq!(
        events,
        vec![json!({
            "type":"response.output_text.delta",
            "delta":"hi"
        })]
    );
}

#[test]
fn provider_session_honors_runner_cancellation_before_provider_setup() {
    let root = std::env::temp_dir();
    let messages = [];
    let tools = [];
    let should_cancel = || true;
    let mut execute_tool = |_tool: &mahayana_host_runtime::extensions::inference::provider_session::RoutedToolDefinition,
                            _args: serde_json::Value,
                            _call_id: &str| -> Result<serde_json::Value, ProviderSessionError> {
        unreachable!("cancelled provider must not execute tools")
    };
    let mut on_text_delta = |_delta: &str, _accumulated: &str| {
        panic!("cancelled provider must not emit text")
    };
    let error = run_routed_provider_text(
        RoutedProvider::Cursor,
        &messages,
        &mut RoutedProviderOptions {
            data_dir: &root,
            tools: &tools,
            mcp_server_url: None,
            execute_tool: &mut execute_tool,
            on_text_delta: &mut on_text_delta,
            should_cancel: &should_cancel,
        },
    )
    .expect_err("runner cancellation should win before provider setup");
    assert!(matches!(error, ProviderSessionError::Cancelled(_)));
}
