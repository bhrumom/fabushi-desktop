use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::box_tool_access::{
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest,
    RunnerBoxWriteRequest,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::tools::sand_browser_tools::{
    BOX_CDP_PORT_BASE, BrowserDriverOutput, BrowserEnvelope, BrowserToolExecutor,
    BrowserToolSpec, ProductionBrowserToolExecutor, SandBrowserToolBridge,
    browser_tool_specs,
    build_browser_driver_invocation, decode_envelope, encode_envelope,
    parse_driver_response, sanitize_for_box_path, stash_screenshot,
    take_stashed_screenshot, to_browser_review_action,
};
use serde_json::{Map, Value, json};

struct BaseBridge;

impl RoutedToolBridge for BaseBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            name: "base_tool".into(),
            provider_identifier: "base".into(),
            tool_name: "base_tool".into(),
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
        Ok(Value::String("delegated".into()))
    }
}

#[derive(Default)]
struct ProductionBoxPort {
    shells: Mutex<Vec<RunnerBoxShellRequest>>,
    reads: Mutex<Vec<RunnerBoxReadRequest>>,
    writes: Mutex<Vec<RunnerBoxWriteRequest>>,
}

impl RunnerBoxResourcePort for ProductionBoxPort {
    fn execute_shell(
        &self,
        request: RunnerBoxShellRequest,
    ) -> Result<Value, ProviderSessionError> {
        self.shells.lock().expect("shells").push(request);
        Ok(json!({"kind":"success","exitCode":0,"stderr":""}))
    }

    fn execute_read(
        &self,
        request: RunnerBoxReadRequest,
    ) -> Result<Value, ProviderSessionError> {
        let path = request.path.clone();
        self.reads.lock().expect("reads").push(request);
        if path.contains("/result-") {
            return Ok(json!({
                "kind":"success",
                "output":{
                    "kind":"content",
                    "content":"driver log\n__SAND_BROWSER_RESULT__{\"ok\":true,\"summary\":\"Opened page\",\"url\":\"https://example.com\",\"title\":\"Example\",\"screenshot\":true}\n"
                }
            }));
        }
        if path.ends_with(".png") {
            return Ok(json!({
                "kind":"success",
                "output":{"kind":"data","data":[137,80,78,71]}
            }));
        }
        Ok(json!({"kind":"fileNotFound","path":path}))
    }

    fn execute_write(
        &self,
        request: RunnerBoxWriteRequest,
    ) -> Result<(), ProviderSessionError> {
        self.writes.lock().expect("writes").push(request);
        Ok(())
    }

    fn browser_window_index(&self) -> Result<u32, ProviderSessionError> {
        Ok(4)
    }
}

struct FakeBrowserExecutor;

impl BrowserToolExecutor for FakeBrowserExecutor {
    fn execute(
        &self,
        spec: &BrowserToolSpec,
        args: &Map<String, Value>,
        tool_call_id: &str,
    ) -> Result<BrowserDriverOutput, ProviderSessionError> {
        Ok(BrowserDriverOutput {
            text: format!(
                "{}:{}:{}",
                spec.op,
                tool_call_id,
                args.get("url").and_then(Value::as_str).unwrap_or_default()
            ),
            image_b64: None,
            is_error: false,
        })
    }
}

#[test]
fn browser_envelope_and_driver_result_round_trip_frozen_shapes() {
    let encoded = encode_envelope(&BrowserEnvelope {
        text: "hello".into(),
        image_key: Some("shot-1".into()),
    });
    assert_eq!(
        decode_envelope(&encoded),
        BrowserEnvelope {
            text: "hello".into(),
            image_key: Some("shot-1".into()),
        }
    );
    assert_eq!(
        decode_envelope("plain text"),
        BrowserEnvelope {
            text: "plain text".into(),
            image_key: None,
        }
    );

    let response = parse_driver_response(
        "diagnostic\n__SAND_BROWSER_RESULT__{\"ok\":true,\"summary\":\"Clicked\",\"url\":\"https://example.com\",\"screenshot\":true}\nafter-result diagnostic\n",
    ).expect("driver response");
    assert!(response.ok);
    assert_eq!(response.summary.as_deref(), Some("Clicked"));
    assert_eq!(response.url.as_deref(), Some("https://example.com"));
    assert_eq!(response.screenshot, Some(true));
}

#[test]
fn browser_invocation_projects_display_cdp_view_and_screenshot_path() {
    let args = json!({
        "url":"https://example.com",
        "newTab":true
    });
    let invocation = build_browser_driver_invocation(
        3,
        "agent-default",
        "navigate",
        "tool/call:1",
        args.as_object().expect("object"),
        false,
    );
    assert_eq!(invocation.request["display"], 3);
    assert_eq!(
        invocation.request["cdpPort"],
        u64::from(BOX_CDP_PORT_BASE + 3)
    );
    assert_eq!(invocation.request["viewId"], "agent-default");
    assert!(invocation
        .screenshot_path
        .as_deref()
        .is_some_and(|path| path.contains("toolcall1")));
    assert!(invocation.shell_command.starts_with("node /tmp/.sand-browser/driver-v2.mjs "));
    assert!(!invocation.encoded_request.is_empty());

    assert_eq!(sanitize_for_box_path("tool/call:1"), "toolcall1");
}

#[test]
fn browser_review_action_matches_operation_specific_projection() {
    let args = json!({
        "viewId":"view-a",
        "method":"Runtime.evaluate",
        "params":{"expression":"location.href"},
        "x":12,
        "modifiers":["Shift", 7]
    });
    let action = to_browser_review_action(
        "cdp",
        args.as_object().expect("object"),
        "default",
    );
    assert_eq!(action.op, "cdp");
    assert_eq!(action.view_id, "view-a");
    assert_eq!(action.cdp_method.as_deref(), Some("Runtime.evaluate"));
    assert!(action
        .cdp_params
        .as_deref()
        .is_some_and(|params| params.contains("location.href")));
    assert_eq!(action.x, Some(12.0));
    assert_eq!(action.modifiers, Some(vec!["Shift".into()]));
}

#[test]
fn browser_toolset_registers_frozen_inventory_validates_and_delegates() {
    let bridge = SandBrowserToolBridge::new(
        Arc::new(BaseBridge),
        Arc::new(FakeBrowserExecutor),
    );
    let tools = bridge.list_tools().expect("tools");
    let specs = browser_tool_specs();
    assert_eq!(
        tools.iter().filter(|tool| tool.name.starts_with("browser_")).count(),
        specs.len()
    );
    assert!(tools.iter().any(|tool| tool.name == "base_tool"));

    let navigate = tools
        .iter()
        .find(|tool| tool.name == "browser_navigate")
        .expect("navigate");
    let error = bridge
        .call_tool(navigate, json!({}), "tool-nav")
        .expect_err("url required");
    assert!(error.to_string().contains("url is required"));

    let result = bridge.call_tool(
        navigate,
        json!({"url":"https://example.com"}),
        "tool-nav",
    ).expect("navigate result");
    assert_eq!(result["text"], "navigate:tool-nav:https://example.com");
    assert_eq!(result["isError"], false);

    let base = tools.iter().find(|tool| tool.name == "base_tool").expect("base");
    assert_eq!(
        bridge.call_tool(base, json!({}), "tool-base").expect("delegate"),
        Value::String("delegated".into())
    );
}

#[test]
fn screenshot_cache_returns_stashed_image_once() {
    let key = stash_screenshot("aW1hZ2U=");
    assert_eq!(take_stashed_screenshot(&key).as_deref(), Some("aW1hZ2U="));
    assert!(take_stashed_screenshot(&key).is_none());
}


#[test]
fn production_browser_executor_uses_only_host_box_port() {
    let port = Arc::new(ProductionBoxPort::default());
    let resources: Arc<dyn RunnerBoxResourcePort> = port.clone();
    let executor = ProductionBrowserToolExecutor::new(resources, "agent-browser");
    let spec = browser_tool_specs()
        .into_iter()
        .find(|spec| spec.name == "browser_navigate")
        .expect("navigate spec");
    let args = json!({"url":"https://example.com"});
    let output = executor
        .execute(
            &spec,
            args.as_object().expect("args"),
            "browser/call:1",
        )
        .expect("browser output");

    assert!(!output.is_error);
    assert!(output.text.contains("Opened page"));
    assert!(output.text.contains("https://example.com"));
    assert!(output.image_b64.as_deref().is_some_and(|value| !value.is_empty()));

    let writes = port.writes.lock().expect("writes");
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].path, "/tmp/.sand-browser/driver-v2.mjs");
    assert!(writes[0].data.len() > 35_000);
    drop(writes);

    let shells = port.shells.lock().expect("shells");
    assert!(shells.iter().any(|request| request.command.contains("mkdir -p /tmp/.sand-browser")));
    assert!(shells.iter().any(|request| {
        request.command.contains("node /tmp/.sand-browser/driver-v2.mjs")
            && request.command.contains("result-browsercall1.txt")
    }));
    drop(shells);

    let reads = port.reads.lock().expect("reads");
    assert!(reads.iter().any(|request| request.path.contains("result-browsercall1.txt")));
    assert!(reads.iter().any(|request| request.path.ends_with("shot-browsercall1.png")));
}
