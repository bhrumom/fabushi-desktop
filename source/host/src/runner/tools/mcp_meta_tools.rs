use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};

pub const EXTRA_SHORT_TOOL_TIMEOUT_MS: u64 = 5 * 60 * 1_000;
pub const SHORT_TOOL_TIMEOUT_MS: u64 = 15 * 60 * 1_000;
pub const MEDIUM_TOOL_TIMEOUT_MS: u64 = 30 * 60 * 1_000;
pub const LONG_TOOL_TIMEOUT_MS: u64 = 60 * 60 * 1_000;
pub const EXTRA_LONG_TOOL_TIMEOUT_MS: u64 = 2 * 60 * 60 * 1_000;

const TIMEOUT_BUFFER_MS: u64 = 60 * 1_000;
const TOOL_CALL_GUARD_HEADROOM_MS: u64 = 60 * 1_000;
const TOOL_CALL_GUARD_BLOCK_GRACE_MS: u64 = 30 * 1_000;

pub const TOOL_CALL_TIMEOUT_TIERS_MS: [u64; 5] = [
    EXTRA_SHORT_TOOL_TIMEOUT_MS,
    SHORT_TOOL_TIMEOUT_MS,
    MEDIUM_TOOL_TIMEOUT_MS,
    LONG_TOOL_TIMEOUT_MS,
    EXTRA_LONG_TOOL_TIMEOUT_MS,
];

pub fn parse_block_until_ms(args: &Value) -> Option<u64> {
    let raw = args.get("block_until_ms")?.as_f64()?;
    if !raw.is_finite() || raw < 0.0 {
        return None;
    }
    if raw >= u64::MAX as f64 {
        Some(u64::MAX)
    } else {
        Some(raw.floor() as u64)
    }
}

pub fn is_subagent_tool_name(tool_name: &str) -> bool {
    matches!(
        tool_name.trim().to_ascii_lowercase().as_str(),
        "task" | "mcp_task" | "subagent"
    )
}

pub fn suggested_tool_timeout_ms(tool_name: &str, args: &Value) -> u64 {
    if is_subagent_tool_name(tool_name) {
        return LONG_TOOL_TIMEOUT_MS;
    }
    parse_block_until_ms(args)
        .map(|block| block.saturating_add(TIMEOUT_BUFFER_MS))
        .unwrap_or(SHORT_TOOL_TIMEOUT_MS)
}

pub fn pick_tool_call_timeout_tier_ms(suggested_ms: u64) -> u64 {
    TOOL_CALL_TIMEOUT_TIERS_MS
        .into_iter()
        .find(|tier| *tier >= suggested_ms)
        .unwrap_or(EXTRA_LONG_TOOL_TIMEOUT_MS)
}

pub fn tool_call_execution_guard_ms(
    tool_name: &str,
    args: &Value,
    is_computer_use_subagent: bool,
) -> u64 {
    let effective_name = if is_computer_use_subagent {
        "subagent"
    } else {
        tool_name
    };
    let tier_ms = pick_tool_call_timeout_tier_ms(
        suggested_tool_timeout_ms(effective_name, args),
    );
    let tier_headroom_ms = tier_ms.saturating_sub(TOOL_CALL_GUARD_HEADROOM_MS);
    let Some(block_ms) = parse_block_until_ms(args) else {
        return tier_headroom_ms;
    };
    let requested_ms = tier_headroom_ms.max(
        block_ms.saturating_add(TOOL_CALL_GUARD_BLOCK_GRACE_MS),
    );
    if requested_ms >= tier_ms {
        tier_headroom_ms
    } else {
        requested_ms
    }
}

pub fn build_tool_call_execution_timed_out_message(
    tool_name: &str,
    execution_timeout_ms: u64,
) -> String {
    let shell_hint = if tool_name.eq_ignore_ascii_case("shell") {
        " For long-running commands, re-run with block_until_ms set to a small value (or 0) so the command runs in the background, then poll its output instead of blocking on it."
    } else {
        ""
    };
    if execution_timeout_ms == 0 {
        format!(
            "The {tool_name} tool call could not start because activity setup exceeded the per-call time limit. The execution environment may be slow or overloaded.{shell_hint}"
        )
    } else {
        format!(
            "The {tool_name} tool call timed out after {} seconds and was terminated. The execution environment may be unresponsive, or the operation needs longer than the per-call time limit.{shell_hint}",
            execution_timeout_ms / 1_000
        )
    }
}

pub fn effective_routed_tool_name(tool: &RoutedToolDefinition) -> &str {
    let tool_name = tool.tool_name.trim();
    if tool_name.is_empty() {
        tool.name.as_str()
    } else {
        tool_name
    }
}

pub fn execute_routed_tool_with_timeout<F>(
    tool: &RoutedToolDefinition,
    args: &Value,
    is_computer_use_subagent: bool,
    operation: F,
) -> Result<Value, ProviderSessionError>
where
    F: FnOnce() -> Result<Value, ProviderSessionError> + Send + 'static,
{
    let tool_name = effective_routed_tool_name(tool).to_string();
    let execution_timeout_ms =
        tool_call_execution_guard_ms(&tool_name, args, is_computer_use_subagent);
    execute_with_timeout_ms(tool_name, execution_timeout_ms, operation)
}

fn execute_with_timeout_ms<F>(
    tool_name: String,
    execution_timeout_ms: u64,
    operation: F,
) -> Result<Value, ProviderSessionError>
where
    F: FnOnce() -> Result<Value, ProviderSessionError> + Send + 'static,
{
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name(format!("mahayana-tool-guard-{}", sanitize_thread_label(&tool_name)))
        .spawn(move || {
            let _ = sender.send(operation());
        })
        .map_err(|error| {
            ProviderSessionError::Tool(format!(
                "could not start {tool_name} tool execution guard: {error}"
            ))
        })?;

    match receiver.recv_timeout(Duration::from_millis(execution_timeout_ms)) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(ProviderSessionError::Tool(
            build_tool_call_execution_timed_out_message(
                &tool_name,
                execution_timeout_ms,
            ),
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(ProviderSessionError::Tool(
            format!("{tool_name} tool execution ended without a result"),
        )),
    }
}

fn sanitize_thread_label(value: &str) -> String {
    let label = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        .take(32)
        .collect::<String>();
    if label.is_empty() {
        "tool".to_string()
    } else {
        label
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolDescriptor {
    pub tool_name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDescriptor {
    pub server_identifier: String,
    pub server_name: String,
    pub tools: Vec<McpToolDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandMcpMetaToolOptions {
    pub enabled: bool,
    pub mcp_descriptors: Vec<McpDescriptor>,
}

pub fn create_sand_mcp_meta_tool_options(
    mcp_tools: &[RoutedToolDefinition],
) -> SandMcpMetaToolOptions {
    let mut descriptors = Vec::<McpDescriptor>::new();
    for tool in mcp_tools {
        let server_identifier = tool.provider_identifier.clone();
        let index = descriptors
            .iter()
            .position(|descriptor| descriptor.server_identifier == server_identifier);
        let descriptor = match index {
            Some(index) => &mut descriptors[index],
            None => {
                descriptors.push(McpDescriptor {
                    server_identifier: server_identifier.clone(),
                    server_name: server_identifier,
                    tools: Vec::new(),
                });
                descriptors.last_mut().expect("descriptor was just inserted")
            }
        };
        descriptor.tools.push(McpToolDescriptor {
            tool_name: tool.tool_name.clone(),
            description: tool.description.clone(),
            input_schema: tool.input_schema.clone(),
        });
    }
    for descriptor in &mut descriptors {
        descriptor
            .tools
            .sort_by(|left, right| left.tool_name.cmp(&right.tool_name));
    }
    SandMcpMetaToolOptions {
        enabled: true,
        mcp_descriptors: descriptors,
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use serde_json::json;

    use super::*;

    #[test]
    fn timeout_worker_returns_success_and_times_out_without_blocking_caller() {
        assert_eq!(
            execute_with_timeout_ms("fast".into(), 100, || Ok(json!({"ok":true})))
                .expect("fast tool"),
            json!({"ok":true})
        );

        let error = execute_with_timeout_ms("slow".into(), 5, || {
            thread::sleep(Duration::from_millis(50));
            Ok(json!({"late":true}))
        })
        .expect_err("slow tool should time out");
        assert!(error.to_string().contains("timed out"));
    }
}
