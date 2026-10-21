//! Generic Bot authoring over the existing reviewed Box Shell. GitHub calls stay
//! visible ordinary MCP calls, preserving their authentication/review ownership.
use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

use crate::extensions::inference::provider_session::{
    ProviderPartialToolCall, ProviderSessionError, RoutedMcpMetaToolDefinition, RoutedToolDefinition,
};
use crate::runner::box_tool_access::{RUNNER_BOX_SHELL_TOOL_NAME, RUNNER_BOX_TOOL_PROVIDER};
use crate::runner::routed_provider_runtime::{RoutedProviderCancellation, RoutedToolBridge};

pub const MINIAPP_AUTHORING_TOOL_NAME: &str = "MiniApp";
const SDK: &str = include_str!("miniapp_authoring.py");
const PACKAGER: &str = include_str!("miniapp_package.py");

pub struct MiniAppAuthoringToolBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    cancellation: RoutedProviderCancellation,
}

impl MiniAppAuthoringToolBridge {
    pub fn new(delegate: Arc<dyn RoutedToolBridge>, cancellation: RoutedProviderCancellation) -> Self {
        Self { delegate, cancellation }
    }
}

pub fn definition() -> RoutedToolDefinition {
    RoutedToolDefinition {
        name: MINIAPP_AUTHORING_TOOL_NAME.into(),
        tool_name: MINIAPP_AUTHORING_TOOL_NAME.into(),
        provider_identifier: "fabushi-runner".into(),
        description: Some("Create, inspect, edit and export an offline Mini App in your isolated Box. Generate application code in src/ using edit, with expectedRevision and each file's expectedSha256 (null for a new file). Export returns exact githubFiles for the existing GitHub MCP connector. Continue through a repository branch, commit, pull request and actual CI result when the user requests GitHub delivery. Export alone is not GitHub publication, signing, approval or installation. Existing native repositories retain their own build workflow.".into()),
        input_schema: json!({
            "type":"object", "additionalProperties":false, "required":["action","project"],
            "properties": {
                "action":{"type":"string","enum":["create","inspect","edit","export"]},
                "project":{"type":"string","pattern":"^[a-z0-9][a-z0-9-]{0,63}$"},
                "appId":{"type":"string","maxLength":128},
                "publisherId":{"type":"string","maxLength":128},
                "displayName":{"type":"string","maxLength":160},
                "version":{"type":"string","pattern":"^[0-9]+[.][0-9]+[.][0-9]+$"},
                "expectedRevision":{"type":"string","pattern":"^[a-f0-9]{64}$"},
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
        if tools.iter().any(|t| t.name == RUNNER_BOX_SHELL_TOOL_NAME && t.provider_identifier == RUNNER_BOX_TOOL_PROVIDER) {
            tools.push(definition());
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
        let payload = serde_json::to_vec(&args).map_err(|_| ProviderSessionError::Tool("MINIAPP_REQUEST_INVALID".into()))?;
        if !args.is_object() || payload.len() > 600 * 1024 {
            return Err(ProviderSessionError::Tool("MINIAPP_REQUEST_LIMIT".into()));
        }
        let shell = self.delegate.list_tools()?.into_iter().find(|t|
            t.name == RUNNER_BOX_SHELL_TOOL_NAME && t.provider_identifier == RUNNER_BOX_TOOL_PROVIDER
        ).ok_or_else(|| ProviderSessionError::Tool("MINIAPP_BOX_UNAVAILABLE".into()))?;
        let bootstrap = format!(
            "import base64;exec(base64.b64decode(\"{}\"),{{\"__name__\":\"__main__\",\"PACKAGER_SOURCE\":base64.b64decode(\"{}\").decode()}})",
            STANDARD.encode(SDK), STANDARD.encode(PACKAGER)
        );
        let command = format!("python3 -I -c {} {}", quote(&bootstrap), quote(&STANDARD.encode(payload)));
        // Delegate the real Shell invocation, rather than calling execute_write
        // directly: the existing Shell auto-review and transport remain active.
        let result = self.delegate.call_tool(&shell, json!({
            "command": command, "workingDirectory":"/workspace", "block_until_ms":30000
        }), tool_call_id)?;
        self.cancellation.check()?;
        Ok(result)
    }
}
