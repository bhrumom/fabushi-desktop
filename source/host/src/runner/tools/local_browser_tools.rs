use std::sync::Arc;

use serde_json::{Map, Value, json};

use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::runner::coordinator_tool_relay::{
    CoordinatorToolRelay, LOCAL_BROWSER_ACTION_METHOD,
};
use crate::runner::tools::sand_browser_tools::{
    BrowserAutoReviewCallback, BrowserDriverOutput, BrowserToolExecutor, BrowserToolSpec,
};

#[derive(Clone)]
pub struct LocalBrowserCoordinatorExecutor {
    relay: Arc<CoordinatorToolRelay>,
    conversation_id: String,
    auto_review: Option<BrowserAutoReviewCallback>,
}

impl LocalBrowserCoordinatorExecutor {
    pub fn new(
        relay: Arc<CoordinatorToolRelay>,
        conversation_id: impl Into<String>,
    ) -> Self {
        Self {
            relay,
            conversation_id: conversation_id.into(),
            auto_review: None,
        }
    }

    pub fn with_auto_review_callback(
        mut self,
        callback: BrowserAutoReviewCallback,
    ) -> Self {
        self.auto_review = Some(callback);
        self
    }

    fn call_main(&self, request: Value) -> Result<Value, ProviderSessionError> {
        self.relay
            .request(LOCAL_BROWSER_ACTION_METHOD, request)
            .map_err(|error| ProviderSessionError::Tool(format!(
                "Local Browser Workspace request failed: {error}"
            )))
    }

    fn resolve_target(
        &self,
        args: &Map<String, Value>,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        let mut request = json!({
            "type": "resolve-target",
            "conversationId": self.conversation_id,
            "operationId": format!("{tool_call_id}:resolve"),
        });
        if let Some(view_id) = args
            .get("viewId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            request["tabId"] = Value::String(view_id.to_string());
        }
        let target = self.call_main(request)?;
        if target.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(ProviderSessionError::Tool(
                "Local Browser Workspace target resolution failed.".into(),
            ));
        }
        if let Some(expected) = args
            .get("expectedGeneration")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let actual = target
                .get("generation")
                .and_then(Value::as_u64)
                .map(|value| value.to_string())
                .unwrap_or_default();
            if expected != actual {
                return Err(ProviderSessionError::Tool(format!(
                    "Local Browser Workspace generation is stale: expected {expected}, actual {actual}"
                )));
            }
        }
        Ok(target)
    }

    fn action_request(
        &self,
        target: &Value,
        tool_call_id: &str,
        action: &str,
    ) -> Result<Map<String, Value>, ProviderSessionError> {
        let workspace_id = target
            .get("workspaceId")
            .and_then(Value::as_str)
            .ok_or_else(|| ProviderSessionError::Tool(
                "Local Browser Workspace target is missing workspaceId.".into(),
            ))?;
        let tab_id = target
            .get("tabId")
            .and_then(Value::as_str)
            .ok_or_else(|| ProviderSessionError::Tool(
                "Local Browser Workspace target is missing tabId.".into(),
            ))?;
        let generation = target
            .get("generation")
            .and_then(Value::as_u64)
            .ok_or_else(|| ProviderSessionError::Tool(
                "Local Browser Workspace target is missing generation.".into(),
            ))?;
        Ok(Map::from_iter([
            ("type".into(), Value::String(action.to_string())),
            ("conversationId".into(), Value::String(self.conversation_id.clone())),
            ("workspaceId".into(), Value::String(workspace_id.to_string())),
            ("tabId".into(), Value::String(tab_id.to_string())),
            ("expectedGeneration".into(), Value::from(generation)),
            ("operationId".into(), Value::String(tool_call_id.to_string())),
        ]))
    }

    fn output_from_result(
        &self,
        result: &Value,
        text: String,
    ) -> Result<BrowserDriverOutput, ProviderSessionError> {
        if result.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(ProviderSessionError::Tool(
                result
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Local Browser Workspace action failed.")
                    .to_string(),
            ));
        }
        Ok(BrowserDriverOutput {
            text,
            image_b64: result
                .get("screenshotBase64")
                .and_then(Value::as_str)
                .map(str::to_string),
            is_error: false,
            view_id: result.get("tabId").and_then(Value::as_str).map(str::to_string),
            generation: result
                .get("generation")
                .and_then(Value::as_u64)
                .map(|value| value.to_string()),
            frame_name: None,
            frame_url: None,
        })
    }

    fn execute_tabs(
        &self,
        args: &Map<String, Value>,
        tool_call_id: &str,
    ) -> Result<BrowserDriverOutput, ProviderSessionError> {
        let action = args
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("list");
        let mut request = json!({
            "type": "tabs",
            "conversationId": self.conversation_id,
            "operationId": tool_call_id,
            "action": action,
        });
        if action == "new" {
            if let Some(url) = args.get("url").and_then(Value::as_str) {
                request["address"] = Value::String(url.to_string());
            }
        } else if matches!(action, "select" | "close") {
            let direct = args
                .get("viewId")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let tab_id = match direct {
                Some(tab_id) => tab_id,
                None => {
                    let listed = self.call_main(json!({
                        "type": "tabs",
                        "conversationId": self.conversation_id,
                        "operationId": format!("{tool_call_id}:list"),
                        "action": "list",
                    }))?;
                    let index = args
                        .get("index")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| ProviderSessionError::Tool(
                            "Local browser_tabs select/close requires viewId or a valid index.".into(),
                        ))?;
                    listed
                        .get("tabs")
                        .and_then(Value::as_array)
                        .and_then(|tabs| tabs.get(index as usize))
                        .and_then(|tab| tab.get("tabId"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .ok_or_else(|| ProviderSessionError::Tool(
                            "Local browser_tabs index is out of range.".into(),
                        ))?
                }
            };
            request["tabId"] = Value::String(tab_id);
        }

        let result = self.call_main(request)?;
        let text = serde_json::to_string_pretty(
            result.get("tabs").unwrap_or(&Value::Array(Vec::new())),
        )
        .unwrap_or_else(|_| "[]".into());
        self.output_from_result(&result, text)
    }
}

impl BrowserToolExecutor for LocalBrowserCoordinatorExecutor {
    fn execute(
        &self,
        spec: &BrowserToolSpec,
        args: &Map<String, Value>,
        tool_call_id: &str,
    ) -> Result<BrowserDriverOutput, ProviderSessionError> {
        if !matches!(
            spec.op,
            "tabs" | "snapshot" | "wait" | "click" | "fill" | "select_option" | "type" | "press_key"
                | "scroll" | "screenshot" | "navigate" | "back" | "forward" | "refresh"
        ) {
            return Err(ProviderSessionError::Tool(format!(
                "browser_{} is not yet supported for target=local; Fabushi will not fall back to the managed Box browser.",
                spec.op
            )));
        }
        if let Some(auto_review) = self.auto_review.as_ref() {
            auto_review(spec, args, tool_call_id)?;
        }
        if spec.op == "tabs" {
            return self.execute_tabs(args, tool_call_id);
        }

        let target = self.resolve_target(args, tool_call_id)?;
        let mut request = match spec.op {
            "snapshot" => self.action_request(&target, tool_call_id, "snapshot")?,
            "wait" => {
                let mut request = self.action_request(&target, tool_call_id, "wait")?;
                request.insert("text".into(), args.get("text").cloned().unwrap_or(Value::Null));
                if let Some(timeout_ms) = args.get("timeoutMs") {
                    request.insert("timeoutMs".into(), timeout_ms.clone());
                }
                request
            },
            "click" => {
                let mut request = self.action_request(&target, tool_call_id, "click")?;
                request.insert("ref".into(), args.get("ref").cloned().unwrap_or(Value::Null));
                request
            }
            "fill" => {
                let mut request = self.action_request(&target, tool_call_id, "fill")?;
                request.insert("ref".into(), args.get("ref").cloned().unwrap_or(Value::Null));
                request.insert("value".into(), args.get("value").cloned().unwrap_or(Value::Null));
                request
            }
            "select_option" => {
                let mut request = self.action_request(&target, tool_call_id, "select-option")?;
                request.insert("ref".into(), args.get("ref").cloned().unwrap_or(Value::Null));
                request.insert("values".into(), args.get("values").cloned().unwrap_or(Value::Null));
                request
            }
            "type" => {
                let mut request = self.action_request(&target, tool_call_id, "type")?;
                request.insert("ref".into(), args.get("ref").cloned().unwrap_or(Value::Null));
                request.insert("text".into(), args.get("text").cloned().unwrap_or(Value::Null));
                request
            }
            "press_key" => {
                let mut request = self.action_request(&target, tool_call_id, "press-key")?;
                request.insert("key".into(), args.get("key").cloned().unwrap_or(Value::Null));
                request
            }
            "scroll" => {
                let mut request = self.action_request(&target, tool_call_id, "scroll")?;
                request.insert(
                    "deltaX".into(),
                    Value::from(args.get("x").and_then(Value::as_f64).unwrap_or(0.0)),
                );
                request.insert(
                    "deltaY".into(),
                    Value::from(args.get("y").and_then(Value::as_f64).unwrap_or(600.0)),
                );
                request
            }
            "screenshot" => self.action_request(&target, tool_call_id, "screenshot")?,
            "navigate" => {
                let mut request = self.action_request(&target, tool_call_id, "navigate")?;
                request.insert("address".into(), args.get("url").cloned().unwrap_or(Value::Null));
                request
            }
            "back" => self.action_request(&target, tool_call_id, "back")?,
            "forward" => self.action_request(&target, tool_call_id, "forward")?,
            "refresh" => self.action_request(&target, tool_call_id, "reload")?,
            unsupported => unreachable!("local browser support list drifted for {unsupported}"),
        };

        request.remove("target");
        let result = self.call_main(Value::Object(request))?;
        let text = if spec.op == "snapshot" {
            serde_json::to_string_pretty(result.get("snapshot").unwrap_or(&Value::Null))
                .unwrap_or_else(|_| "{}".into())
        } else {
            let title = result
                .get("observedTitle")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let url = result
                .get("observedUrl")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if title.is_empty() && url.is_empty() {
                "Done.".into()
            } else {
                format!("Current page: {title} ({url})")
            }
        };
        self.output_from_result(&result, text)
    }
}

pub struct ExplicitBrowserTargetExecutor {
    box_executor: Arc<dyn BrowserToolExecutor>,
    local_executor: Arc<dyn BrowserToolExecutor>,
}

impl ExplicitBrowserTargetExecutor {
    pub fn new(
        box_executor: Arc<dyn BrowserToolExecutor>,
        local_executor: Arc<dyn BrowserToolExecutor>,
    ) -> Self {
        Self {
            box_executor,
            local_executor,
        }
    }
}

impl BrowserToolExecutor for ExplicitBrowserTargetExecutor {
    fn execute(
        &self,
        spec: &BrowserToolSpec,
        args: &Map<String, Value>,
        tool_call_id: &str,
    ) -> Result<BrowserDriverOutput, ProviderSessionError> {
        match args.get("target").and_then(Value::as_str).unwrap_or("box") {
            "box" => self.box_executor.execute(spec, args, tool_call_id),
            "local" => self.local_executor.execute(spec, args, tool_call_id),
            target => Err(ProviderSessionError::Tool(format!(
                "browser target must be box or local, got {target}"
            ))),
        }
    }
}
