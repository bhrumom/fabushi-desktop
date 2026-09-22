use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ROUTED_MCP_PROTOCOL_VERSION, RoutedToolBridge, start_routed_mcp_server,
};
use serde_json::{Value, json};

struct FakeBridge;

impl RoutedToolBridge for FakeBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            name: "github_search".into(),
            provider_identifier: "github".into(),
            tool_name: "search".into(),
            description: Some("Search GitHub".into()),
            input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        assert_eq!(tool.provider_identifier, "github");
        assert_eq!(args["q"], "rust");
        assert!(!tool_call_id.is_empty());
        Ok(json!({"content":[{"type":"text","text":"tool-result"}]}))
    }
}

fn post(url: &str, body: Value) -> Value {
    let without_scheme = url.strip_prefix("http://").expect("http");
    let (authority, path) = without_scheme.split_once('/').expect("path");
    let mut stream = TcpStream::connect(authority).expect("connect");
    let payload = serde_json::to_vec(&body).expect("payload");
    write!(
        stream,
        "POST /{path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    )
    .expect("headers");
    stream.write_all(&payload).expect("body");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("response");
    let body = response.split_once("\r\n\r\n").expect("http body").1;
    serde_json::from_str(body).expect("json")
}

#[test]
fn runner_owned_mcp_bridge_lists_and_executes_host_tools() {
    let mut server = start_routed_mcp_server(Arc::new(FakeBridge)).expect("server");
    let initialized = post(
        server.url(),
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    assert_eq!(
        initialized["result"]["protocolVersion"],
        ROUTED_MCP_PROTOCOL_VERSION
    );

    let tools = post(
        server.url(),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    assert_eq!(tools["result"]["tools"][0]["name"], "github_search");

    let called = post(
        server.url(),
        json!({
            "jsonrpc":"2.0",
            "id":3,
            "method":"tools/call",
            "params":{"name":"github_search","arguments":{"q":"rust"}}
        }),
    );
    assert_eq!(called["result"]["content"][0]["text"], "tool-result");
    server.close();
}
