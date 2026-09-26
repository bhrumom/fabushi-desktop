use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use crate::runner::routed_provider_runtime::RoutedToolBridge;

use super::sand_browser_driver_source::{
    SAND_BROWSER_DRIVER_BOX_DIR, SAND_BROWSER_DRIVER_BOX_PATH,
    SAND_BROWSER_RESULT_MARKER,
};

pub const BOX_CDP_PORT_BASE: u32 = 9_222;
pub const PENDING_SCREENSHOT_CAP: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserEnvelope {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_key: Option<String>,
}

pub fn encode_envelope(envelope: &BrowserEnvelope) -> String {
    serde_json::to_string(envelope).unwrap_or_else(|_| {
        serde_json::json!({"text":envelope.text}).to_string()
    })
}

pub fn decode_envelope(raw: &str) -> BrowserEnvelope {
    serde_json::from_str::<BrowserEnvelope>(raw).unwrap_or_else(|_| BrowserEnvelope {
        text: raw.to_string(),
        image_key: None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserDriverResponse {
    pub ok: bool,
    pub error: Option<String>,
    pub summary: Option<String>,
    pub data: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub view_id: Option<String>,
    pub screenshot: Option<bool>,
}

pub fn to_driver_response(parsed: &Map<String, Value>) -> BrowserDriverResponse {
    BrowserDriverResponse {
        ok: parsed.get("ok").and_then(Value::as_bool).unwrap_or(false),
        error: string_field(parsed, "error"),
        summary: string_field(parsed, "summary"),
        data: string_field(parsed, "data"),
        url: string_field(parsed, "url"),
        title: string_field(parsed, "title"),
        view_id: string_field(parsed, "viewId"),
        screenshot: parsed.get("screenshot").and_then(Value::as_bool),
    }
}

pub fn parse_driver_response(stdout: &str) -> Option<BrowserDriverResponse> {
    for line in stdout.lines().rev() {
        let Some(marker_index) = line.find(SAND_BROWSER_RESULT_MARKER) else {
            continue;
        };
        let raw = &line[marker_index + SAND_BROWSER_RESULT_MARKER.len()..];
        let parsed: Value = serde_json::from_str(raw).ok()?;
        let object = parsed.as_object()?;
        return Some(to_driver_response(object));
    }
    None
}

pub fn sanitize_for_box_path(value: &str) -> String {
    let cleaned = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
        .take(48)
        .collect::<String>();
    if !cleaned.is_empty() {
        return cleaned;
    }
    format!("call-{}", now_ms())
}

fn screenshot_cache() -> &'static Mutex<VecDeque<(String, String)>> {
    static CACHE: OnceLock<Mutex<VecDeque<(String, String)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn stash_screenshot(image_b64: impl Into<String>) -> String {
    let key = format!("shot-{}-{}", now_ms(), next_screenshot_nonce());
    if let Ok(mut cache) = screenshot_cache().lock() {
        while cache.len() >= PENDING_SCREENSHOT_CAP {
            cache.pop_front();
        }
        cache.push_back((key.clone(), image_b64.into()));
    }
    key
}

pub fn take_stashed_screenshot(key: &str) -> Option<String> {
    let mut cache = screenshot_cache().lock().ok()?;
    let index = cache.iter().position(|(candidate, _)| candidate == key)?;
    cache.remove(index).map(|(_, image)| image)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserReviewAction {
    pub op: String,
    pub view_id: String,
    pub url: Option<String>,
    pub r#ref: Option<String>,
    pub element: Option<String>,
    pub text: Option<String>,
    pub value: Option<String>,
    pub values: Option<Vec<String>>,
    pub key: Option<String>,
    pub cdp_method: Option<String>,
    pub cdp_params: Option<String>,
    pub tabs_action: Option<String>,
    pub tab_index: Option<f64>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub source_ref: Option<String>,
    pub target_ref: Option<String>,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub new_tab: Option<bool>,
    pub submit: Option<bool>,
    pub clear: Option<bool>,
    pub double_click: Option<bool>,
    pub button: Option<String>,
    pub modifiers: Option<Vec<String>>,
}

pub fn to_browser_review_action(
    op: &str,
    args: &Map<String, Value>,
    default_view_id: &str,
) -> BrowserReviewAction {
    BrowserReviewAction {
        op: op.to_string(),
        view_id: string_ref(args, "viewId")
            .unwrap_or(default_view_id)
            .to_string(),
        url: string_field(args, "url"),
        r#ref: string_field(args, "ref"),
        element: string_field(args, "element"),
        text: string_field(args, "text"),
        value: string_field(args, "value"),
        values: string_array(args, "values"),
        key: string_field(args, "key"),
        cdp_method: (op == "cdp").then(|| string_field(args, "method")).flatten(),
        cdp_params: if op == "cdp" {
            args.get("params").map(Value::to_string)
        } else {
            None
        },
        tabs_action: (op == "tabs").then(|| string_field(args, "action")).flatten(),
        tab_index: if op == "tabs" {
            args.get("index").and_then(Value::as_f64)
        } else {
            None
        },
        x: args.get("x").and_then(Value::as_f64),
        y: args.get("y").and_then(Value::as_f64),
        source_ref: string_field(args, "sourceRef"),
        target_ref: string_field(args, "targetRef"),
        target_x: args.get("targetX").and_then(Value::as_f64),
        target_y: args.get("targetY").and_then(Value::as_f64),
        new_tab: args.get("newTab").and_then(Value::as_bool),
        submit: args.get("submit").and_then(Value::as_bool),
        clear: args.get("clear").and_then(Value::as_bool),
        double_click: args.get("doubleClick").and_then(Value::as_bool),
        button: string_field(args, "button"),
        modifiers: string_array(args, "modifiers"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserToolSchema {
    pub required: Vec<&'static str>,
    pub enum_values: BTreeMap<&'static str, Vec<&'static str>>,
}

impl Default for BrowserToolSchema {
    fn default() -> Self {
        Self {
            required: Vec::new(),
            enum_values: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserToolSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub op: &'static str,
    pub schema: BrowserToolSchema,
    pub can_navigate: bool,
    pub skip_screenshot: bool,
}

pub fn browser_tool_specs() -> Vec<BrowserToolSpec> {
    vec![
        spec("BROWSER_NAVIGATE", "browser_navigate", "navigate", "Navigate the box browser to a URL. By default reuses your tab; set newTab: true to open in a new tab. Returns the resulting page state with a screenshot.", &["url"], &[], true, false),
        spec("BROWSER_SNAPSHOT", "browser_snapshot", "snapshot", "Capture a structured snapshot of the current page with ref handles for interactive elements.", &[], &[], false, false),
        spec("BROWSER_CLICK", "browser_click", "click", "Click an element by ref from browser_snapshot.", &["ref"], &[], true, false),
        spec("BROWSER_MOUSE_CLICK_XY", "browser_mouse_click_xy", "mouse_click_xy", "Click at viewport coordinates.", &["x","y"], &[], true, false),
        spec("BROWSER_TYPE", "browser_type", "type", "Type text into an editable element by ref.", &["ref","text"], &[], true, false),
        spec("BROWSER_FILL", "browser_fill", "fill", "Set the value of an editable element by ref.", &["ref","value"], &[], false, false),
        spec("BROWSER_SELECT_OPTION", "browser_select_option", "select_option", "Select one or more options in a select element by ref.", &["ref","values"], &[], false, false),
        spec("BROWSER_PRESS_KEY", "browser_press_key", "press_key", "Press a key in the browser page.", &["key"], &[], true, false),
        spec("BROWSER_SCROLL", "browser_scroll", "scroll", "Scroll the page or scroll an element into view.", &[], &[], false, false),
        spec("BROWSER_DRAG", "browser_drag", "drag", "Drag an element by ref to another ref or viewport coordinates.", &["sourceRef"], &[], false, false),
        spec("BROWSER_GET_BOUNDING_BOX", "browser_get_bounding_box", "get_bounding_box", "Get the viewport bounding box for an element ref.", &["ref"], &[], false, true),
        spec("BROWSER_HIGHLIGHT", "browser_highlight", "highlight", "Highlight an element by ref for visual grounding.", &["ref"], &[], false, false),
        spec("BROWSER_CDP", "browser_cdp", "cdp", "Send a Chrome DevTools Protocol command to the target browser tab.", &["method"], &[], true, false),
        spec("BROWSER_TABS", "browser_tabs", "tabs", "List, create, close, or select a browser tab.", &["action"], &[("action", &["list","new","close","select"])], false, true),
        spec("BROWSER_TAKE_SCREENSHOT", "browser_take_screenshot", "screenshot", "Take a screenshot of the current page.", &[], &[], false, false),
    ]
}

pub fn validate_browser_arguments(
    schema: &BrowserToolSchema,
    args: &Map<String, Value>,
) -> Result<(), BrowserDriverError> {
    for key in &schema.required {
        if args.get(*key).is_none_or(|value| value.is_null() || value.as_str() == Some("")) {
            return Err(BrowserDriverError::new(format!("{key} is required")));
        }
    }
    for (key, values) in &schema.enum_values {
        let value = args
            .get(*key)
            .and_then(Value::as_str)
            .ok_or_else(|| BrowserDriverError::new(format!(
                "{key} must be one of {}",
                values.join(", ")
            )))?;
        if !values.contains(&value) {
            return Err(BrowserDriverError::new(format!(
                "{key} must be one of {}",
                values.join(", ")
            )));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserDriverInvocation {
    pub request: Value,
    pub screenshot_path: Option<String>,
    pub encoded_request: String,
    pub shell_command: String,
}

pub fn build_browser_driver_invocation(
    window_index: u32,
    default_view_id: &str,
    op: &str,
    tool_call_id: &str,
    args: &Map<String, Value>,
    skip_screenshot: bool,
) -> BrowserDriverInvocation {
    let screenshot_path = if skip_screenshot {
        None
    } else {
        Some(format!(
            "{SAND_BROWSER_DRIVER_BOX_DIR}/shot-{}.png",
            sanitize_for_box_path(tool_call_id)
        ))
    };
    let requested_view_id = string_ref(args, "viewId")
        .filter(|value| !value.is_empty())
        .unwrap_or(default_view_id);
    let mut request = args.clone();
    request.insert("op".into(), Value::String(op.to_string()));
    request.insert("display".into(), Value::from(window_index));
    request.insert(
        "cdpPort".into(),
        Value::from(BOX_CDP_PORT_BASE.saturating_add(window_index)),
    );
    request.insert("viewId".into(), Value::String(requested_view_id.to_string()));
    if let Some(path) = screenshot_path.as_deref() {
        request.insert("screenshotPath".into(), Value::String(path.to_string()));
    }
    let request = Value::Object(request);
    let encoded_request = base64::engine::general_purpose::STANDARD
        .encode(request.to_string().as_bytes());
    let shell_command = format!(
        "node {SAND_BROWSER_DRIVER_BOX_PATH} {encoded_request}"
    );
    BrowserDriverInvocation {
        request,
        screenshot_path,
        encoded_request,
        shell_command,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserDriverOutput {
    pub text: String,
    pub image_b64: Option<String>,
    pub is_error: bool,
}

pub fn render_driver_response(
    response: &BrowserDriverResponse,
    image_b64: Option<String>,
) -> BrowserDriverOutput {
    if !response.ok {
        return BrowserDriverOutput {
            text: response
                .error
                .clone()
                .unwrap_or_else(|| "The browser action failed.".into()),
            image_b64: None,
            is_error: true,
        };
    }
    let mut parts = vec![response.summary.clone().unwrap_or_else(|| "Done.".into())];
    if let Some(url) = response.url.as_deref().filter(|url| !url.is_empty()) {
        parts.push(format!(
            "Current page: {} ({url})",
            response.title.as_deref().unwrap_or_default()
        ));
    }
    if let Some(data) = response.data.as_deref().filter(|data| !data.is_empty()) {
        parts.push(data.to_string());
    }
    BrowserDriverOutput {
        text: parts.join("\n\n"),
        image_b64: if response.screenshot == Some(true) {
            image_b64
        } else {
            None
        },
        is_error: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct BrowserDriverError {
    pub message: String,
}

impl BrowserDriverError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

pub trait BrowserToolExecutor: Send + Sync {
    fn execute(
        &self,
        spec: &BrowserToolSpec,
        args: &Map<String, Value>,
        tool_call_id: &str,
    ) -> Result<BrowserDriverOutput, ProviderSessionError>;
}

pub struct SandBrowserToolBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    executor: Arc<dyn BrowserToolExecutor>,
}

impl SandBrowserToolBridge {
    pub fn new(
        delegate: Arc<dyn RoutedToolBridge>,
        executor: Arc<dyn BrowserToolExecutor>,
    ) -> Self {
        Self { delegate, executor }
    }
}

impl RoutedToolBridge for SandBrowserToolBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        let specs = browser_tool_specs();
        let names = specs.iter().map(|spec| spec.name).collect::<Vec<_>>();
        let mut delegated = self.delegate.list_tools()?;
        delegated.retain(|tool| {
            !names.contains(&tool.name.as_str())
                && !names.contains(&tool.tool_name.as_str())
        });
        let mut tools = specs
            .iter()
            .map(browser_tool_definition)
            .collect::<Vec<_>>();
        tools.extend(delegated);
        Ok(tools)
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        let specs = browser_tool_specs();
        let effective = if specs.iter().any(|spec| spec.name == tool.name) {
            tool.name.as_str()
        } else {
            tool.tool_name.as_str()
        };
        let Some(spec) = specs.iter().find(|spec| spec.name == effective) else {
            return self.delegate.call_tool(tool, args, tool_call_id);
        };
        let object = args
            .as_object()
            .ok_or_else(|| ProviderSessionError::Tool(format!(
                "{} arguments must be an object",
                spec.name
            )))?;
        validate_browser_arguments(&spec.schema, object)
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        let output = self.executor.execute(spec, object, tool_call_id)?;
        Ok(json!({
            "text": output.text,
            "imageB64": output.image_b64,
            "isError": output.is_error,
        }))
    }
}

fn browser_tool_definition(spec: &BrowserToolSpec) -> RoutedToolDefinition {
    let mut properties = Map::new();
    for key in &spec.schema.required {
        properties.insert((*key).to_string(), json!({}));
    }
    for (key, values) in &spec.schema.enum_values {
        properties.insert((*key).to_string(), json!({"type":"string","enum":values}));
    }
    properties.entry("viewId").or_insert_with(|| json!({"type":"string"}));
    RoutedToolDefinition {
        name: spec.name.into(),
        provider_identifier: "fabushi-runner".into(),
        tool_name: spec.name.into(),
        description: Some(spec.description.into()),
        input_schema: json!({
            "type":"object",
            "required":spec.schema.required,
            "additionalProperties":true,
            "properties":properties,
        }),
    }
}

fn spec(
    id: &'static str,
    name: &'static str,
    op: &'static str,
    description: &'static str,
    required: &[&'static str],
    enum_values: &[(&'static str, &[&'static str])],
    can_navigate: bool,
    skip_screenshot: bool,
) -> BrowserToolSpec {
    BrowserToolSpec {
        id,
        name,
        description,
        op,
        schema: BrowserToolSchema {
            required: required.to_vec(),
            enum_values: enum_values
                .iter()
                .map(|(key, values)| (*key, values.to_vec()))
                .collect(),
        },
        can_navigate,
        skip_screenshot,
    }
}

fn string_field(object: &Map<String, Value>, key: &str) -> Option<String> {
    object.get(key).and_then(Value::as_str).map(str::to_string)
}

fn string_ref<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

fn string_array(object: &Map<String, Value>, key: &str) -> Option<Vec<String>> {
    let values = object.get(key)?.as_array()?;
    Some(
        values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
    )
}

fn next_screenshot_nonce() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NONCE: AtomicU64 = AtomicU64::new(0);
    NONCE.fetch_add(1, Ordering::Relaxed)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
