use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::tools::send_message_encoding::{
    encode_markdown_image_destination, encode_send_message, encode_text_content,
};
use mahayana_host_runtime::runner::tools::send_message_schema::{
    parse_send_message_input, refine_send_message, validate_send_message,
};
use mahayana_host_runtime::runner::tools::send_message_tool::{
    SAND_SEND_MESSAGE_TOOL_NAME, SendMessageSink, SendMessageToolBridge,
};
use serde_json::{Value, json};

struct Delegate;

impl RoutedToolBridge for Delegate {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            name: "mcp.demo".into(),
            provider_identifier: "demo".into(),
            tool_name: "demo".into(),
            description: None,
            input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Ok(json!({"delegated":true}))
    }
}

#[derive(Default)]
struct Sink {
    messages: Mutex<Vec<Value>>,
}

impl SendMessageSink for Sink {
    fn send_message(
        &self,
        message: Value,
        _timestamp_ms: u64,
        tool_call_id: &str,
    ) -> Result<Option<String>, ProviderSessionError> {
        self.messages.lock().expect("messages").push(message);
        Ok(Some(format!("runner-send:{tool_call_id}")))
    }
}

#[test]
fn schema_matches_frozen_type_fences_and_attachment_schemes() {
    let valid = parse_send_message_input(&json!({
        "type":"text",
        "content":" hello ",
        "images":[{"url":"file:///tmp/a.png","alt":" a "}]
    }))
    .expect("parse");
    assert!(validate_send_message(&valid).is_ok());
    assert_eq!(valid.content.as_deref(), Some("hello"));

    let invalid = parse_send_message_input(&json!({
        "type":"widget",
        "content":"must not ride widget",
        "widget":{"prompt":"Choose","options":[]},
        "channel":"slack:1"
    }))
    .expect("parse");
    let issues = refine_send_message(&invalid);
    assert!(issues.iter().any(|issue| issue.path == vec!["content".to_string()]));
    assert!(issues.iter().any(|issue| issue.path == vec!["channel".to_string()]));

    let attachment = parse_send_message_input(&json!({
        "type":"attachment",
        "url":"ftp://example.test/file"
    }))
    .expect("parse");
    assert!(validate_send_message(&attachment).is_err());
}

#[test]
fn encoding_preserves_frozen_text_markdown_and_summary_projection() {
    assert_eq!(
        encode_markdown_image_destination("https://example.test/a<1>\n"),
        "<https://example.test/a%3C1%3E%0A>"
    );
    let images = vec![json!({"url":"https://x.test/a.png","alt":"[cat]\nphoto"})];
    assert_eq!(
        encode_text_content("hello", Some(images.as_slice())),
        "hello\n\n![ cat  photo](<https://x.test/a.png>)"
    );
    assert_eq!(
        encode_send_message(&json!({
            "type":"connector",
            "variant":"connected",
            "connector":"Slack"
        }))
        .expect("encode")["message"]["value"]["content"],
        "Confirmed the Slack connector is connected"
    );
    assert_eq!(
        encode_send_message(&json!({
            "type":"permission-request",
            "permission":{"title":"Files","reason":"Read a report"}
        }))
        .expect("encode")["message"]["value"]["content"],
        "Legacy permission request (no longer actionable): Files — Read a report"
    );
}

#[test]
fn send_message_bridge_is_first_party_and_delegates_other_tools() {
    let sink = Arc::new(Sink::default());
    let bridge = SendMessageToolBridge::new(Arc::new(Delegate), sink.clone());
    let tools = bridge.list_tools().expect("tools");
    assert_eq!(tools[0].name, SAND_SEND_MESSAGE_TOOL_NAME);
    assert!(tools.iter().any(|tool| tool.name == "mcp.demo"));

    let result = bridge.call_tool(
        &tools[0],
        json!({
            "type":"text",
            "content":"hello",
            "images":[{"url":"https://example.test/a.png","alt":"a"}]
        }),
        "tool-1",
    )
    .expect("send");
    assert_eq!(result["sent"], true);
    assert_eq!(result["messageId"], "runner-send:tool-1");
    assert_eq!(
        sink.messages.lock().expect("messages")[0]["content"],
        "hello"
    );

    let delegated = bridge
        .call_tool(
            tools.iter().find(|tool| tool.name == "mcp.demo").expect("demo"),
            json!({}),
            "tool-2",
        )
        .expect("delegate");
    assert_eq!(delegated["delegated"], true);
}
