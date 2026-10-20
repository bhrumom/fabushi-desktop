use std::sync::{Arc, Mutex};
use std::time::Duration;

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::box_tool_access::{
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest,
    RunnerBoxWriteRequest,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::tools::sand_browser_driver_source::SAND_BROWSER_DRIVER_SOURCE;
use mahayana_host_runtime::gateway_server::GatewayEventHub;
use mahayana_host_runtime::runner::coordinator_tool_relay::CoordinatorToolRelay;
use mahayana_host_runtime::runner::tools::local_browser_tools::{
    ExplicitBrowserTargetExecutor, LocalBrowserCoordinatorExecutor,
};
use mahayana_host_runtime::runner::tools::sand_browser_tools::{
    BOX_CDP_PORT_BASE, BrowserDriverOutput, BrowserEnvelope, BrowserToolExecutor,
    BrowserShellSafetyCallback, BrowserToolSpec, ProductionBrowserToolExecutor,
    SandBrowserToolBridge,
    browser_tool_specs, capture_browser_review_state,
    build_browser_driver_invocation, decode_envelope, encode_envelope,
    parse_driver_response, sanitize_for_box_path, stash_screenshot,
    take_stashed_screenshot, to_browser_review_action, validate_browser_arguments,
};
use serde_json::{Map, Value, json};

const BROWSER_AUTO_REVIEW_SOURCE: &str = include_str!("../src/runner/sand_browser_auto_review.rs");
const LOCAL_BROWSER_TOOLS_SOURCE: &str = include_str!("../src/runner/tools/local_browser_tools.rs");
const AUTO_REVIEW_SUMMARIES_SOURCE: &str = include_str!("../src/runner/sand_auto_review_summaries.rs");

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
        if path.contains("/sand-browser-review-state-") {
            return Ok(json!({
                "kind":"success",
                "output":{
                    "kind":"content",
                    "content":"driver log\n[{\"id\":\"target-a\",\"type\":\"page\",\"url\":\"https://example.com/settings?tab=1\"}]\n"
                }
            }));
        }
        if path.ends_with("/views-4.json") {
            return Ok(json!({
                "kind":"success",
                "output":{
                    "kind":"content",
                    "content":"{\"views\":{\"agent-browser\":\"target-a\"},\"urls\":{\"agent-browser\":\"https://fallback.example/\"}}"
                }
            }));
        }
        if path.contains("/result-") {
            return Ok(json!({
                "kind":"success",
                "output":{
                    "kind":"content",
                    "content":"driver log\n__SAND_BROWSER_RESULT__{\"ok\":true,\"summary\":\"Opened page\",\"url\":\"https://example.com\",\"title\":\"Example\",\"viewId\":\"agent-browser\",\"generation\":\"generation-7\",\"screenshot\":true}\n"
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

#[derive(Clone)]
struct TrackingBrowserExecutor {
    label: &'static str,
    calls: Arc<Mutex<Vec<String>>>,
}

impl BrowserToolExecutor for TrackingBrowserExecutor {
    fn execute(
        &self,
        spec: &BrowserToolSpec,
        _args: &Map<String, Value>,
        _tool_call_id: &str,
    ) -> Result<BrowserDriverOutput, ProviderSessionError> {
        self.calls.lock().expect("calls").push(self.label.to_string());
        Ok(BrowserDriverOutput {
            text: format!("{}:{}", self.label, spec.op),
            image_b64: None,
            is_error: false,
            view_id: None,
            generation: None,
            frame_name: None,
            frame_url: None,
        })
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
            view_id: Some("view-fake".into()),
            generation: Some("generation-fake".into()),
            frame_name: Some("checkout-frame".into()),
            frame_url: Some("https://example.com/frame".into()),
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
        "diagnostic\n__SAND_BROWSER_RESULT__{\"ok\":true,\"summary\":\"Clicked\",\"url\":\"https://example.com\",\"viewId\":\"view-a\",\"generation\":\"generation-a\",\"screenshot\":true}\nafter-result diagnostic\n",
    ).expect("driver response");
    assert!(response.ok);
    assert_eq!(response.summary.as_deref(), Some("Clicked"));
    assert_eq!(response.url.as_deref(), Some("https://example.com"));
    assert_eq!(response.view_id.as_deref(), Some("view-a"));
    assert_eq!(response.generation.as_deref(), Some("generation-a"));
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
    assert!(invocation.shell_command.starts_with("node /workspace/.sand-browser/driver-v2.mjs "));
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
    assert_eq!(result["viewId"], "view-fake");
    assert_eq!(result["generation"], "generation-fake");
    assert_eq!(result["frameName"], "checkout-frame");
    assert_eq!(result["frameUrl"], "https://example.com/frame");
    assert_eq!(
        navigate.input_schema["properties"]["expectedGeneration"]["type"],
        "string"
    );
    assert_eq!(navigate.input_schema["properties"]["frameName"]["type"], "string");
    assert_eq!(navigate.input_schema["properties"]["frameUrl"]["type"], "string");
    assert_eq!(
        navigate.input_schema["properties"]["target"]["enum"],
        json!(["box","local"])
    );

    let mut invalid_target = Map::new();
    invalid_target.insert("url".into(), json!("https://example.com"));
    invalid_target.insert("target".into(), json!("auto"));
    let target_error = validate_browser_arguments(
        &browser_tool_specs().into_iter().find(|spec| spec.name == "browser_navigate").expect("navigate spec").schema,
        &invalid_target,
    ).expect_err("unknown browser target must fail closed");
    assert!(target_error.to_string().contains("box or local"));

    let base = tools.iter().find(|tool| tool.name == "base_tool").expect("base");
    assert_eq!(
        bridge.call_tool(base, json!({}), "tool-base").expect("delegate"),
        Value::String("delegated".into())
    );
}

#[test]
fn explicit_browser_target_defaults_to_box_and_never_falls_back() {
    let calls = Arc::new(Mutex::new(Vec::<String>::new()));
    let box_executor: Arc<dyn BrowserToolExecutor> = Arc::new(TrackingBrowserExecutor {
        label: "box",
        calls: Arc::clone(&calls),
    });
    let local_executor: Arc<dyn BrowserToolExecutor> = Arc::new(TrackingBrowserExecutor {
        label: "local",
        calls: Arc::clone(&calls),
    });
    let executor = ExplicitBrowserTargetExecutor::new(box_executor, local_executor);
    let spec = browser_tool_specs().into_iter()
        .find(|spec| spec.name == "browser_snapshot")
        .expect("snapshot");

    let default_args = Map::new();
    let output = executor.execute(&spec, &default_args, "default").expect("default box");
    assert_eq!(output.text, "box:snapshot");

    let local_args = json!({"target":"local"});
    let output = executor.execute(
        &spec,
        local_args.as_object().expect("local args"),
        "local",
    ).expect("explicit local");
    assert_eq!(output.text, "local:snapshot");

    let invalid_args = json!({"target":"auto"});
    let error = executor.execute(
        &spec,
        invalid_args.as_object().expect("invalid args"),
        "invalid",
    ).expect_err("unknown target");
    assert!(error.to_string().contains("box or local"));
    assert_eq!(
        calls.lock().expect("calls").as_slice(),
        &["box".to_string(), "local".to_string()]
    );
}

#[test]
fn browser_wait_is_bounded_generation_fenced_and_shared_by_explicit_targets() {
    let spec = browser_tool_specs().into_iter()
        .find(|spec| spec.name == "browser_wait")
        .expect("wait spec");
    assert_eq!(spec.op, "wait");
    assert_eq!(spec.schema.required, vec!["text"]);
    assert_eq!(spec.schema.max_text_len, Some(1_024));
    assert_eq!(spec.schema.max_timeout_ms, Some(15_000));
    assert!(!spec.can_navigate);
    assert!(spec.skip_screenshot);

    let valid = json!({"text":"Ready","timeoutMs":15000,"target":"local"});
    validate_browser_arguments(&spec.schema, valid.as_object().expect("valid wait"))
        .expect("bounded local wait");

    for invalid in [
        json!({"text":"","target":"local"}),
        json!({"text":"Ready","timeoutMs":0,"target":"local"}),
        json!({"text":"Ready","timeoutMs":15001,"target":"local"}),
    ] {
        assert!(
            validate_browser_arguments(&spec.schema, invalid.as_object().expect("invalid wait")).is_err(),
            "invalid wait must fail closed: {invalid}"
        );
    }

    for required in [
        "  \"wait\",",
        "wait: async ({ request, context, state }) => {",
        "Buffer.byteLength(text, \"utf8\") > 1024",
        "Math.max(1, Math.min(15000, request.timeoutMs))",
        "scope.getByText(text, { exact: false }).first().waitFor({",
    ] {
        assert!(
            SAND_BROWSER_DRIVER_SOURCE.contains(required),
            "managed Box wait lost invariant: {required}"
        );
    }
    for required in [
        "\"tabs\" | \"snapshot\" | \"wait\" | \"click\"",
        "\"wait\" => {",
        "self.action_request(&target, tool_call_id, \"wait\")?",
        "request.insert(\"timeoutMs\".into(), timeout_ms.clone());",
    ] {
        assert!(
            LOCAL_BROWSER_TOOLS_SOURCE.contains(required),
            "local Browser Workspace wait lost relay invariant: {required}"
        );
    }
}

#[test]
fn local_select_option_uses_the_generation_fenced_workspace_ref_path() {
    let spec = browser_tool_specs().into_iter()
        .find(|spec| spec.name == "browser_select_option")
        .expect("select option spec");
    assert_eq!(spec.op, "select_option");
    for required in [
        "\"select_option\" | \"type\"",
        "\"select_option\" => {",
        "self.action_request(&target, tool_call_id, \"select-option\")?",
        "request.insert(\"values\".into(), args.get(\"values\").cloned().unwrap_or(Value::Null));",
    ] {
        assert!(
            LOCAL_BROWSER_TOOLS_SOURCE.contains(required),
            "local select-option lost invariant: {required}"
        );
    }
}

#[test]
fn local_drag_uses_only_bounded_workspace_action_fields_and_never_local_cdp() {
    for required in [
        "\"scroll\" | \"drag\" | \"screenshot\"",
        "\"drag\" => {",
        "self.action_request(&target, tool_call_id, \"drag\")?",
        "request.insert(\"sourceRef\".into()",
        "request.insert(\"targetRef\".into()",
        "request.insert(\"targetX\".into()",
        "request.insert(\"targetY\".into()",
    ] {
        assert!(
            LOCAL_BROWSER_TOOLS_SOURCE.contains(required),
            "local drag lost invariant: {required}"
        );
    }
}

#[test]
fn local_browser_actions_carry_main_owned_control_epoch_and_tab_mutations_resolve_it() {
    for required in [
        "target.get(\"controlEpoch\")",
        "\"expectedControlEpoch\".into(), Value::from(control_epoch)",
        "if action != \"list\" {",
        "request[\"expectedControlEpoch\"] = target.get(\"controlEpoch\")",
    ] {
        assert!(
            LOCAL_BROWSER_TOOLS_SOURCE.contains(required),
            "local control lease lost invariant: {required}"
        );
    }
}

#[test]
fn local_browser_actions_carry_main_owned_task_identity() {
    for required in [
        "target.get(\"taskId\")",
        "\"expectedTaskId\".into(), Value::String(task_id.to_string())",
        "request[\"expectedTaskId\"] = target.get(\"taskId\")",
    ] {
        assert!(
            LOCAL_BROWSER_TOOLS_SOURCE.contains(required),
            "local task-space fence lost invariant: {required}"
        );
    }
}

#[test]
fn unsupported_local_browser_action_fails_before_auto_review_or_relay() {
    let reviewed = Arc::new(Mutex::new(Vec::<String>::new()));
    let reviewed_sink = Arc::clone(&reviewed);
    let relay = Arc::new(CoordinatorToolRelay::with_timeout(
        GatewayEventHub::default(),
        Duration::from_millis(20),
    ));
    let executor = LocalBrowserCoordinatorExecutor::new(relay, "agent-a")
        .with_auto_review_callback(Arc::new(move |spec, _args, _tool_call_id| {
            reviewed_sink.lock().expect("reviewed").push(spec.name.to_string());
            Ok(())
        }));
    let spec = browser_tool_specs().into_iter()
        .find(|spec| spec.name == "browser_cdp")
        .expect("cdp spec");
    let args = json!({"method":"Runtime.evaluate","target":"local"});
    let error = executor.execute(
        &spec,
        args.as_object().expect("args"),
        "local-cdp",
    ).expect_err("local cdp must fail closed");
    assert!(error.to_string().contains("not yet supported for target=local"));
    assert!(
        reviewed.lock().expect("reviewed").is_empty(),
        "unsupported local actions must fail before asking for approval"
    );
}

#[test]
fn local_upload_requires_user_authorized_sentinel_before_review_and_never_forwards_a_path() {
    let sentinel = LOCAL_BROWSER_TOOLS_SOURCE
        .find("Some(\"user-authorized\")")
        .expect("local upload sentinel");
    let review = LOCAL_BROWSER_TOOLS_SOURCE
        .find("if let Some(auto_review)")
        .expect("local auto review");
    assert!(sentinel < review, "local upload path sentinel must be checked before approval");

    for required in [
        "\"click\" | \"upload_file\" | \"fill\"",
        "\"upload_file\" => {",
        "self.action_request(&target, tool_call_id, \"upload-file\")?",
        "request.insert(\"ref\".into(), args.get(\"ref\").cloned().unwrap_or(Value::Null));",
        "never accepts an Agent-provided local path",
    ] {
        assert!(
            LOCAL_BROWSER_TOOLS_SOURCE.contains(required),
            "local upload lost fail-closed invariant: {required}"
        );
    }
    assert!(
        !LOCAL_BROWSER_TOOLS_SOURCE.contains("request.insert(\"filePath\".into()"),
        "local upload must never forward an Agent-controlled local filePath to Main"
    );
}

#[test]
fn screenshot_cache_returns_stashed_image_once() {
    let key = stash_screenshot("aW1hZ2U=");
    assert_eq!(take_stashed_screenshot(&key).as_deref(), Some("aW1hZ2U="));
    assert!(take_stashed_screenshot(&key).is_none());
}


#[test]
fn browser_review_state_uses_host_box_probe_and_view_mapping() {
    let port = ProductionBoxPort::default();
    let state = capture_browser_review_state(
        &port,
        4,
        Some("agent-browser"),
        "browser/review:1",
    )
    .expect("browser review state");
    assert_eq!(state.display_state_identity.len(), 64);
    assert_eq!(
        state.target_page_url.as_deref(),
        Some("https://example.com/settings")
    );
    let shells = port.shells.lock().expect("shells");
    assert!(shells.iter().any(|request| {
        request.command.contains("http://127.0.0.1:9226/json/list")
            && request.command.contains("/tmp/sand-browser-review-state-browserreview1.json")
    }));
}

#[test]
fn browser_claim_generation_is_persisted_under_the_existing_cross_process_lock() {
    for required in [
        "generations:",
        "freshViewGeneration",
        "ensureViewGeneration",
        "bumpViewGeneration",
        "withViewClaimLock(display, fn, required = false)",
        "if (required && !locked)",
        "saveState(display, state, required = false)",
        "if (required) throw error",
        "generations: { [viewId]: generation }",
        "await bumpViewGeneration(request.display, state, viewId);",
        "out.generation = generation",
    ] {
        assert!(
            SAND_BROWSER_DRIVER_SOURCE.contains(required),
            "missing browser generation invariant: {required}"
        );
    }
    assert!(
        SAND_BROWSER_DRIVER_SOURCE.contains("}, true);"),
        "generation-sensitive view claims must require the cross-process lock"
    );
    assert!(
        SAND_BROWSER_DRIVER_SOURCE.contains("delete generations[removed];"),
        "closing a view must remove its persisted generation"
    );
}

#[test]
fn browser_generation_fencing_rejects_missing_or_stale_claims_before_actions() {
    for required in [
        "const GENERATION_FENCED_OPS = new Set([",
        "function requireExpectedGeneration(request, viewId, generation)",
        "Browser action requires expectedGeneration for view",
        "Stale browser view generation for ",
        "if (expected !== generation)",
        "requireExpectedGeneration(request, viewId, generation);",
    ] {
        assert!(
            SAND_BROWSER_DRIVER_SOURCE.contains(required),
            "missing browser stale-generation invariant: {required}"
        );
    }
    let required_index = SAND_BROWSER_DRIVER_SOURCE
        .find("requireExpectedGeneration(request, viewId, generation);")
        .expect("generation fence call");
    let return_index = SAND_BROWSER_DRIVER_SOURCE[required_index..]
        .find("return { page, viewId };")
        .map(|offset| required_index + offset)
        .expect("resolved page return");
    assert!(
        required_index < return_index,
        "generation fence must run before handing the page to an operation"
    );
}

#[test]
fn browser_history_navigation_is_generation_fenced_and_rotates_the_claim() {
    let specs = browser_tool_specs();
    for (name, op) in [
        ("browser_refresh", "refresh"),
        ("browser_back", "back"),
        ("browser_forward", "forward"),
    ] {
        let spec = specs.iter().find(|spec| spec.name == name).expect(name);
        assert_eq!(spec.op, op);
        assert!(spec.can_navigate);
        assert!(!spec.skip_screenshot);
        assert!(
            SAND_BROWSER_DRIVER_SOURCE.contains(&format!("  \"{op}\",")),
            "{op} must be in the generation-fenced op set"
        );
    }

    for required in [
        "refresh: async ({ request, context, state }) => {",
        "await page.reload({ waitUntil: \"domcontentloaded\", timeout: NAVIGATE_TIMEOUT_MS });",
        "back: async ({ request, context, state }) => {",
        "return navigateHistory(request, context, state, -1);",
        "No previous browser history entry is available for this tab.",
        "forward: async ({ request, context, state }) => {",
        "return navigateHistory(request, context, state, 1);",
        "No forward browser history entry is available for this tab.",
        "await bumpViewGeneration(request.display, state, viewId);",
    ] {
        assert!(
            SAND_BROWSER_DRIVER_SOURCE.contains(required),
            "missing browser history-navigation invariant: {required}"
        );
    }
}

#[test]
fn browser_exact_frame_targeting_is_unambiguous_and_frame_scoped() {
    for required in [
        "function resolveFrameScope(page, request)",
        "if (frame === page.mainFrame()) return false;",
        "frame.name() !== frameName",
        "frame.url() !== frameUrl",
        "No browser frame matches frameName=",
        "Browser frame selector is ambiguous; provide both exact frameName and frameUrl.",
        "const { scope, frameName, frameUrl } = resolveFrameScope(page, request);",
        "const handle = await scope.evaluateHandle((r) => {",
        "if (result.frameName !== undefined) out.frameName = result.frameName;",
        "if (result.frameUrl !== undefined) out.frameUrl = result.frameUrl;",
    ] {
        assert!(
            SAND_BROWSER_DRIVER_SOURCE.contains(required),
            "missing exact-frame invariant: {required}"
        );
    }
}

#[test]
fn browser_file_transfer_is_workspace_bounded_reviewed_and_recoverable() {
    let specs = browser_tool_specs();
    let download = specs
        .iter()
        .find(|spec| spec.name == "browser_download")
        .expect("download tool");
    let upload = specs
        .iter()
        .find(|spec| spec.name == "browser_upload_file")
        .expect("upload tool");
    assert_eq!(download.op, "download");
    assert_eq!(upload.op, "upload_file");

    let tools = SandBrowserToolBridge::new(Arc::new(BaseBridge), Arc::new(FakeBrowserExecutor))
        .list_tools()
        .expect("browser tools");
    let upload_tool = tools
        .iter()
        .find(|tool| tool.name == "browser_upload_file")
        .expect("upload tool definition");
    assert_eq!(
        upload_tool.input_schema["properties"]["filePath"]["type"],
        "string"
    );

    let mut invalid = Map::new();
    invalid.insert("ref".into(), json!("e1"));
    invalid.insert("filePath".into(), json!(7));
    let invalid_path = validate_browser_arguments(&upload.schema, &invalid)
        .expect_err("non-string upload filePath must fail closed");
    assert!(invalid_path.to_string().contains("filePath"));

    let review = to_browser_review_action(
        "upload_file",
        json!({
            "viewId":"view-a",
            "expectedGeneration":"generation-a",
            "ref":"e1",
            "element":"attachment picker",
            "filePath":"/workspace/report.pdf"
        })
        .as_object()
        .expect("upload args"),
        "default",
    );
    assert_eq!(review.file_path.as_deref(), Some("/workspace/report.pdf"));

    for required in [
        "const WORKSPACE_ROOT = \"/workspace\";",
        "realpathSync(rawPath)",
        "relative(WORKSPACE_ROOT, resolved)",
        "rel.startsWith(\"../\")",
        "isAbsolute(rel)",
        "info.isFile()",
        "MAX_UPLOAD_BYTES = 25 * 1024 * 1024",
        "MAX_DOWNLOAD_BYTES = 64 * 1024 * 1024",
        "page.waitForEvent(\"download\"",
        "await download.saveAs(path);",
        "createHash(\"sha256\")",
        "await element.setInputFiles(upload.path);",
        "\"download\",",
        "\"upload_file\",",
    ] {
        assert!(
            SAND_BROWSER_DRIVER_SOURCE.contains(required),
            "missing browser file-transfer invariant: {required}"
        );
    }
    for required in [
        "\"download\" | \"upload_file\"",
        "SAND_BROWSER_AUTO_REVIEW_MAX_FILE_PATH_CHARS",
        "object.get(\"filePath\")",
    ] {
        assert!(
            BROWSER_AUTO_REVIEW_SOURCE.contains(required),
            "missing file-transfer review invariant: {required}"
        );
    }
    assert!(AUTO_REVIEW_SUMMARIES_SOURCE.contains("\"download\" =>"));
    assert!(AUTO_REVIEW_SUMMARIES_SOURCE.contains("\"upload_file\" =>"));
    assert!(AUTO_REVIEW_SUMMARIES_SOURCE.contains("redact_inline_secrets(args.file_path"));
}

#[test]
fn production_browser_executor_uses_only_host_box_port() {
    let port = Arc::new(ProductionBoxPort::default());
    let resources: Arc<dyn RunnerBoxResourcePort> = port.clone();
    let auto_reviews = Arc::new(Mutex::new(Vec::<String>::new()));
    let auto_reviews_sink = Arc::clone(&auto_reviews);
    let persisted_images = Arc::new(Mutex::new(Vec::<(Vec<u8>, String)>::new()));
    let persisted_images_sink = Arc::clone(&persisted_images);
    let navigation_events = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let navigation_events_sink = Arc::clone(&navigation_events);
    let guarded_shells = Arc::new(Mutex::new(Vec::<RunnerBoxShellRequest>::new()));
    let guarded_shells_sink = Arc::clone(&guarded_shells);
    let shell_safety: BrowserShellSafetyCallback = Arc::new(move |request| {
        guarded_shells_sink.lock().expect("guarded shells").push(request.clone());
        Ok(())
    });
    let executor = ProductionBrowserToolExecutor::new(resources, "agent-browser")
        .with_auto_review_callback(Arc::new(move |spec, args, tool_call_id| {
            assert_eq!(spec.name, "browser_navigate");
            assert_eq!(
                args.get("url").and_then(Value::as_str),
                Some("https://example.com")
            );
            auto_reviews_sink
                .lock()
                .expect("auto reviews")
                .push(tool_call_id.to_string());
            Ok(())
        }))
        .with_persist_image_callback(Arc::new(move |bytes, mime| {
            persisted_images_sink
                .lock()
                .expect("persisted images")
                .push((bytes.to_vec(), mime.to_string()));
        }))
        .with_possible_navigation_callback(Arc::new(move |response| {
            navigation_events_sink
                .lock()
                .expect("navigation events")
                .push((
                    response.url.clone().unwrap_or_default(),
                    response.title.clone().unwrap_or_default(),
                ));
        }))
        .with_shell_safety_callback(shell_safety);
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
    assert_eq!(
        auto_reviews.lock().expect("auto reviews").as_slice(),
        &["browser/call:1".to_string()]
    );
    assert!(output.text.contains("Opened page"));
    assert!(output.text.contains("https://example.com"));
    assert!(output.image_b64.as_deref().is_some_and(|value| !value.is_empty()));
    assert_eq!(output.view_id.as_deref(), Some("agent-browser"));
    assert_eq!(output.generation.as_deref(), Some("generation-7"));
    assert_eq!(
        persisted_images.lock().expect("persisted images").as_slice(),
        &[(vec![137, 80, 78, 71], "image/png".to_string())]
    );
    assert_eq!(
        navigation_events.lock().expect("navigation events").as_slice(),
        &[("https://example.com".to_string(), "Example".to_string())]
    );

    let writes = port.writes.lock().expect("writes");
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].path, "/workspace/.sand-browser/driver-v2.mjs");
    assert!(writes[0].data.len() > 35_000);
    drop(writes);

    let shells = port.shells.lock().expect("shells");
    assert!(shells.iter().any(|request| request.command.contains("mkdir -p /workspace/.sand-browser")));
    assert!(shells.iter().any(|request| {
        request.command.contains("node /workspace/.sand-browser/driver-v2.mjs")
            && request.command.contains("result-browsercall1.txt")
    }));
    let guarded = guarded_shells.lock().expect("guarded shells");
    assert_eq!(guarded.as_slice(), shells.as_slice());
    drop(guarded);
    drop(shells);

    let reads = port.reads.lock().expect("reads");
    assert!(reads.iter().any(|request| request.path.contains("result-browsercall1.txt")));
    assert!(reads.iter().any(|request| request.path.ends_with("shot-browsercall1.png")));
}


#[test]
fn production_browser_executor_fails_closed_before_internal_shell_when_host_safety_blocks() {
    let port = Arc::new(ProductionBoxPort::default());
    let resources: Arc<dyn RunnerBoxResourcePort> = port.clone();
    let executor = ProductionBrowserToolExecutor::new(resources, "agent-browser")
        .with_shell_safety_callback(Arc::new(|request| {
            Err(ProviderSessionError::Tool(format!(
                "pending approval blocks {}",
                request.tool_call_id
            )))
        }));
    let spec = browser_tool_specs().into_iter()
        .find(|spec| spec.name == "browser_navigate")
        .expect("navigate spec");
    let args = json!({"url":"https://example.com"});
    let error = executor.execute(&spec, args.as_object().expect("args"), "browser/blocked:1")
        .expect_err("host safety must fail closed");
    assert!(error.to_string().contains("pending approval"));
    assert!(port.shells.lock().expect("shells").is_empty());
    assert!(port.writes.lock().expect("writes").is_empty());
}
