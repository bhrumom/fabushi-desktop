//! OpenClaw Node Host Runtime Manifest & Command Advertising
//!
//! Mirrors:
//! - `src/node-host/runtime-manifest.ts`
//! - `src/node-host/runtime.ts`
//! - `src/node-host/runner.ts`
//! - `src/node-host/worker-runtime.ts`
//! - `src/node-host/launcher-bootstrap.ts`

use std::collections::{BTreeSet, HashSet};
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
    /// Build the complete transport-neutral surface from current builtin and
    /// plugin availability. Plugin registries are expected to provide only
    /// currently registered commands/capabilities; the Gateway transport
    /// remains responsible for publishing the returned snapshot.
    pub fn build_with_plugin_manifest(
        available_commands: &HashSet<String>,
        plugin_commands: &[String],
        plugin_capabilities: &[String],
        claude_enabled: bool,
        desktop_streaming_enabled: bool,
        ephemeral: bool,
        allowlist: Option<&HashSet<String>>,
    ) -> Self {
        let mut commands = BTreeSet::new();
        for command in [
            NODE_DEVICE_APPS_COMMAND,
            NODE_EXEC_APPROVALS_COMMANDS,
            NODE_FS_LIST_DIR_COMMAND,
            NODE_MCP_TOOLS_CALL_COMMAND,
            NODE_SYSTEM_RUN_COMMANDS,
            NODE_TERMINAL_UPLOAD_COMMAND,
        ] {
            if available_commands.contains(command) {
                commands.insert(command.to_string());
            }
        }
        if claude_enabled && available_commands.contains(NODE_AGENT_CLI_CLAUDE_RUN_COMMAND) {
            commands.insert(NODE_AGENT_CLI_CLAUDE_RUN_COMMAND.to_string());
        }
        if desktop_streaming_enabled && available_commands.contains(NODE_DESKTOP_STREAM_COMMAND) {
            commands.insert(NODE_DESKTOP_STREAM_COMMAND.to_string());
        }
        commands.extend(plugin_commands.iter().filter_map(|command| {
            let command = command.trim();
            if command.is_empty()
                || (ephemeral && matches!(command, "screen.snapshot" | "computer.act"))
            {
                None
            } else {
                Some(command.to_string())
            }
        }));
        if let Some(allowed) = allowlist {
            commands.retain(|command| allowed.contains(command));
        }

        let mut capabilities = BTreeSet::new();
        if commands.contains(NODE_SYSTEM_RUN_COMMANDS) {
            capabilities.insert(SYSTEM_RUN_EXECUTION_CONTEXT_CAPABILITY.to_string());
        }
        if commands.contains(NODE_AGENT_CLI_CLAUDE_RUN_COMMAND) {
            capabilities.insert(NODE_CLAUDE_SKILLS_CAPABILITY.to_string());
        }
        capabilities.extend(
            plugin_capabilities
                .iter()
                .filter(|capability| {
                    !capability.trim().is_empty()
                        && (!ephemeral || !matches!(capability.as_str(), "computer" | "screen"))
                })
                .cloned(),
        );

        Self {
            advertised_commands: commands.into_iter().collect(),
            capabilities: capabilities.into_iter().collect(),
            is_ephemeral: ephemeral,
        }
    }

    pub fn build(
        available_commands: &HashSet<String>,
        claude_enabled: bool,
        desktop_streaming_enabled: bool,
        allowlist: Option<&HashSet<String>>,
    ) -> Self {
        Self::build_with_plugin_manifest(
            available_commands,
            &[],
            &[],
            claude_enabled,
            desktop_streaming_enabled,
            false,
            allowlist,
        )
    }
}
