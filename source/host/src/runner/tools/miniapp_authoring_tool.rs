//! Generic Bot authoring over the existing reviewed Box Shell. GitHub calls stay
//! visible ordinary MCP calls, preserving their authentication/review ownership.
use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use crate::sand_activity::SAND_EXTERNAL_SHELL_TOOL_NAME;
use super::sand_external_machine_tools::EXTERNAL_MACHINE_TOOL_PROVIDER;

use crate::extensions::inference::provider_session::{
    ProviderPartialToolCall, ProviderSessionError, RoutedMcpMetaToolDefinition, RoutedToolDefinition,
};
use crate::runner::box_tool_access::{RUNNER_BOX_SHELL_TOOL_NAME, RUNNER_BOX_TOOL_PROVIDER};
use crate::runner::routed_provider_runtime::{RoutedProviderCancellation, RoutedToolBridge};

pub const MINIAPP_AUTHORING_TOOL_NAME: &str = "MiniApp";
const SDK: &str = include_str!("miniapp_authoring.py");
const PACKAGER: &str = include_str!("miniapp_package.py");
const RELEASER: &str = include_str!("miniapp_release.py");

pub struct MiniAppAuthoringToolBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    cancellation: RoutedProviderCancellation,
    local_agent_key: Option<String>,
}

impl MiniAppAuthoringToolBridge {
    pub fn new(delegate: Arc<dyn RoutedToolBridge>, cancellation: RoutedProviderCancellation) -> Self {
        Self { delegate, cancellation, local_agent_key: None }
    }

    pub fn with_local_agent_id(mut self, agent_id: &str) -> Self {
        if cfg!(unix) && !agent_id.trim().is_empty() {
            self.local_agent_key = Some(format!("{:x}", Sha256::digest(agent_id.as_bytes())));
        }
        self
    }
}

pub fn definition() -> RoutedToolDefinition {
    RoutedToolDefinition {
        name: MINIAPP_AUTHORING_TOOL_NAME.into(),
        tool_name: MINIAPP_AUTHORING_TOOL_NAME.into(),
        provider_identifier: "fabushi-runner".into(),
        description: Some("Create, import existing web source, read, inspect, edit and export an offline Mini App in your isolated Box. Generate application code in src/ and behavior tests in tests/*.test.cjs using edit, with expectedRevision and each file's expectedSha256 (null for a new file). Export returns exact githubFiles for the existing GitHub MCP connector. Continue through a repository branch, commit, pull request and actual CI behavior-test/package result when the user requests GitHub delivery. For requested GitHub publication, export includes an explicit tagged workflow_dispatch release pipeline: after actual tests succeed, dispatch on vVERSION with publish=true and the exact expectedSourceSha, then verify the remote release URL and asset digest. Read requires expectedRevision and a source path, with bounded UTF-8 byte pages. Import requires complete source/tests and config and never overwrites an existing project. Export alone is not GitHub publication, signing, approval or installation. executionTarget=local uses the connected Mac/Unix computer and its existing ExternalShell review/permission, with Python 3 required; source is data, never executed. executionTarget=box uses the isolated remote Box. Default is box when available, otherwise local if configured. These are separate project stores: never silently retry on another target after a failure. Existing native repositories retain their own build workflow.".into()),
        input_schema: json!({
            "type":"object", "additionalProperties":false, "required":["action","project"],
            "properties": {
                "action":{"type":"string","enum":["create","import","read","inspect","edit","export"]},
                "project":{"type":"string","pattern":"^[a-z0-9][a-z0-9-]{0,63}$"},
                "executionTarget":{"type":"string","enum":["box","local"]},
                "appId":{"type":"string","maxLength":128},
                "publisherId":{"type":"string","maxLength":128},
                "displayName":{"type":"string","maxLength":160},
                "version":{"type":"string","pattern":"^[0-9]+[.][0-9]+[.][0-9]+$"},
                "expectedRevision":{"type":"string","pattern":"^[a-f0-9]{64}$"},
                "path":{"type":"string","maxLength":240},
                "offset":{"type":"integer","minimum":0},
                "maxBytes":{"type":"integer","minimum":1,"maximum":16384},
                "files":{"type":"array","minItems":1,"maxItems":64,"items":{
                    "type":"object","additionalProperties":false,"required":["path","content","expectedSha256"],
                    "properties":{
                        "path":{"type":"string","maxLength":240},
                        "content":{"type":["string","null"],"maxLength":131072},
                        "expectedSha256":{"type":["string","null"],"pattern":"^[a-f0-9]{64}$"}
                    }
                }}
            }
        }),
    }
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

impl RoutedToolBridge for MiniAppAuthoringToolBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        let mut tools = self.delegate.list_tools()?;
        if tools.iter().any(|t| t.name == MINIAPP_AUTHORING_TOOL_NAME) {
            return Err(ProviderSessionError::Protocol("MiniApp tool name collision".into()));
        }
        let box_available = tools.iter().any(|t| t.name == RUNNER_BOX_SHELL_TOOL_NAME && t.provider_identifier == RUNNER_BOX_TOOL_PROVIDER);
        let local_available = self.local_agent_key.is_some() && tools.iter().any(|t|
            t.name == SAND_EXTERNAL_SHELL_TOOL_NAME && t.provider_identifier == EXTERNAL_MACHINE_TOOL_PROVIDER);
        if box_available || local_available {
            let mut tool = definition();
            let mut targets = Vec::new();
            if box_available { targets.push("box"); }
            if local_available { targets.push("local"); }
            tool.input_schema["properties"]["executionTarget"]["enum"] = json!(targets);
            if let Some(description) = tool.description.as_mut() {
                description.push_str(&format!(" Available execution targets for this turn: {}.", targets.join(", ")));
            }
            tools.push(tool);
        }
        Ok(tools)
    }

    fn list_mcp_meta_tools(&self) -> Result<Vec<RoutedMcpMetaToolDefinition>, ProviderSessionError> {
        self.delegate.list_mcp_meta_tools()
    }

    fn observe_partial_tool_call(&self, partial: &ProviderPartialToolCall) -> Result<(), ProviderSessionError> {
        self.delegate.observe_partial_tool_call(partial)
    }

    fn call_tool(&self, tool: &RoutedToolDefinition, args: Value, tool_call_id: &str) -> Result<Value, ProviderSessionError> {
        if tool.name != MINIAPP_AUTHORING_TOOL_NAME || tool.tool_name != MINIAPP_AUTHORING_TOOL_NAME || tool.provider_identifier != "fabushi-runner" {
            return self.delegate.call_tool(tool, args, tool_call_id);
        }
        self.cancellation.check()?;
        let tools = self.delegate.list_tools()?;
        let box_available = tools.iter().any(|t| t.name == RUNNER_BOX_SHELL_TOOL_NAME && t.provider_identifier == RUNNER_BOX_TOOL_PROVIDER);
        let target = match args.get("executionTarget") {
            Some(Value::String(value)) if value == "box" || value == "local" => value.as_str(),
            None if box_available => "box",
            None => "local",
            _ => return Err(ProviderSessionError::Tool("MINIAPP_EXECUTION_TARGET_INVALID".into())),
        };
        let local = target == "local";
        let mut request = args.clone();
        if let Some(object) = request.as_object_mut() { object.remove("executionTarget"); }
        let payload = serde_json::to_vec(&request).map_err(|_| ProviderSessionError::Tool("MINIAPP_REQUEST_INVALID".into()))?;
        if !args.is_object() || payload.len() > 600 * 1024 {
            return Err(ProviderSessionError::Tool("MINIAPP_REQUEST_LIMIT".into()));
        }
        let local_key = if local {
            Some(self.local_agent_key.as_ref().ok_or_else(|| ProviderSessionError::Tool("MINIAPP_LOCAL_UNAVAILABLE".into()))?)
        } else { None };
        let (shell_name, provider) = if local {
            (SAND_EXTERNAL_SHELL_TOOL_NAME, EXTERNAL_MACHINE_TOOL_PROVIDER)
        } else { (RUNNER_BOX_SHELL_TOOL_NAME, RUNNER_BOX_TOOL_PROVIDER) };
        let shell = tools.into_iter().find(|t| t.name == shell_name && t.provider_identifier == provider)
            .ok_or_else(|| ProviderSessionError::Tool("MINIAPP_EXECUTOR_UNAVAILABLE".into()))?;
        // Keep the trusted SDK readable to the existing Shell auto-review.
        // Only user-authored file data is encoded; it is parsed as JSON, never code.
        let bootstrap = format!("PACKAGER_SOURCE = {}\nRELEASE_SOURCE = {}\nAUTHORING_LOCAL_AGENT = {}\n{}",
            serde_json::to_string(PACKAGER).map_err(|_| ProviderSessionError::Tool("MINIAPP_SDK_INVALID".into()))?,
            serde_json::to_string(RELEASER).map_err(|_| ProviderSessionError::Tool("MINIAPP_SDK_INVALID".into()))?,
            local_key.map(|key| serde_json::to_string(key).unwrap()).unwrap_or_else(|| "None".into()), SDK);
        let command = format!("python3 -I -c {} {}", quote(&bootstrap), quote(&STANDARD.encode(payload)));
        // Delegate the real Shell invocation, rather than calling execute_write
        // directly: the existing Shell auto-review and transport remain active.
        let shell_args = if local { json!({"command": command, "workingDirectory":"/"}) }
            else { json!({"command": command, "workingDirectory":"/workspace", "block_until_ms":30000}) };
        let result = self.delegate.call_tool(&shell, shell_args, tool_call_id)?;
        self.cancellation.check()?;
        Ok(result)
    }
}
