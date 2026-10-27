//! OpenClaw Node Host Runtime Manifest & Command Advertising
//!
//! Mirrors:
//! - `src/node-host/runtime-manifest.ts`
//! - `src/node-host/runtime.ts`
//! - `src/node-host/runner.ts`
//! - `src/node-host/worker-runtime.ts`
//! - `src/node-host/launcher-bootstrap.ts`

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

pub const NODE_AGENT_CLI_CLAUDE_RUN_COMMAND: &str = "node.agent_cli_claude_run";
pub const NODE_DEVICE_APPS_COMMAND: &str = "node.device_apps";
pub const NODE_EXEC_APPROVALS_COMMANDS: &str = "node.exec_approvals";
pub const NODE_FS_LIST_DIR_COMMAND: &str = "node.fs_list_dir";
pub const NODE_MCP_TOOLS_CALL_COMMAND: &str = "node.mcp_tools_call";
pub const NODE_SYSTEM_RUN_COMMANDS: &str = "node.system_run";
pub const NODE_TERMINAL_UPLOAD_COMMAND: &str = "node.terminal_upload";
pub const NODE_DESKTOP_STREAM_COMMAND: &str = "node.desktop_stream";

pub const SYSTEM_RUN_EXECUTION_CONTEXT_CAPABILITY: &str = "system_run.execution_context";
pub const NODE_CLAUDE_SKILLS_CAPABILITY: &str = "node.claude_skills";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeHostManifest {
    pub advertised_commands: Vec<String>,
    pub capabilities: Vec<String>,
    pub is_ephemeral: bool,
}

impl NodeHostManifest {
    pub fn build(
        claude_enabled: bool,
        desktop_streaming_enabled: bool,
        allowlist: Option<&HashSet<String>>,
    ) -> Self {
        let mut commands = vec![
            NODE_DEVICE_APPS_COMMAND.to_string(),
            NODE_EXEC_APPROVALS_COMMANDS.to_string(),
            NODE_FS_LIST_DIR_COMMAND.to_string(),
            NODE_MCP_TOOLS_CALL_COMMAND.to_string(),
            NODE_SYSTEM_RUN_COMMANDS.to_string(),
            NODE_TERMINAL_UPLOAD_COMMAND.to_string(),
        ];

        if claude_enabled {
            commands.push(NODE_AGENT_CLI_CLAUDE_RUN_COMMAND.to_string());
        }

        if desktop_streaming_enabled {
            commands.push(NODE_DESKTOP_STREAM_COMMAND.to_string());
        }

        if let Some(allowed) = allowlist {
            commands.retain(|c| allowed.contains(c));
        }

        let mut capabilities = vec![SYSTEM_RUN_EXECUTION_CONTEXT_CAPABILITY.to_string()];
        if claude_enabled {
            capabilities.push(NODE_CLAUDE_SKILLS_CAPABILITY.to_string());
        }

        Self {
            advertised_commands: commands,
            capabilities,
            is_ephemeral: false,
        }
    }
}
