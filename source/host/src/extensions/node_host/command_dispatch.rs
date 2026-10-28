//! OpenClaw Node Host Command Dispatch (Computer, PTY, and CLI Handlers)
//!
//! Mirrors:
//! - `src/node-host/computer-command.ts`
//! - `src/node-host/pty-command.ts`
//! - `src/node-host/desktop-stream-command.ts`
//! - `src/node-host/portal-stream-command.ts`
//! - `src/node-host/invoke-agent-cli-claude-handler.ts`
//! - `src/node-host/claude-skill-session.ts`
//! - `src/node-host/exec-policy.ts`

use serde::{Deserialize, Serialize};
use serde_json::json;

pub const COMPUTER_CONTRACT_MISMATCH: &str =
    "COMPUTER_CONTRACT_MISMATCH: generation mismatch between provider and worker";
pub const COMPUTER_DRIVER_UNAVAILABLE: &str =
    "COMPUTER_DRIVER_UNAVAILABLE: worker computer provider is unavailable";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputerOperation {
    Close,
    Capabilities,
    Act,
    Snapshot,
}

impl ComputerOperation {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "close" => Some(ComputerOperation::Close),
            "capabilities" => Some(ComputerOperation::Capabilities),
            "act" => Some(ComputerOperation::Act),
            "snapshot" => Some(ComputerOperation::Snapshot),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputerCommandInput {
    pub operation: ComputerOperation,
    /// Opaque upstream provider generation; do not coerce it to a number.
    pub provider_generation: Option<String>,
    pub action_params_json: Option<String>,
}

/// The node-host dispatcher owns protocol validation and generation fencing.
/// Platform authorization and side effects stay with the injected native owner.
pub trait NodeWorkerComputerExecutor {
    fn capabilities(&self, generation: &str) -> Result<String, String>;
    fn act(&self, generation: &str, params: &serde_json::Value) -> Result<String, String>;
    fn snapshot(&self, generation: &str, params: &serde_json::Value) -> Result<String, String>;
}

pub fn dispatch_node_worker_computer_command_with_executor<E: NodeWorkerComputerExecutor>(
    input: &ComputerCommandInput,
    current_generation: Option<&str>,
    executor: &E,
) -> Result<String, String> {
    if input.operation == ComputerOperation::Close {
        return Ok(json!({"status": "closed"}).to_string());
    }

    let generation = current_generation
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| COMPUTER_DRIVER_UNAVAILABLE.to_string())?;
    if input.operation != ComputerOperation::Capabilities {
        if let Some(req_gen) = input.provider_generation.as_deref() {
            if req_gen != generation {
                return Err(COMPUTER_CONTRACT_MISMATCH.to_string());
            }
        }
    }

    let params = match input.action_params_json.as_deref() {
        Some(raw) => serde_json::from_str(raw)
            .map_err(|_| "COMPUTER_INVALID_ACTION: action parameters are invalid JSON".to_string())?,
        None => serde_json::Value::Null,
    };
    match input.operation {
        ComputerOperation::Capabilities => executor.capabilities(generation),
        ComputerOperation::Act => executor.act(generation, &params),
        ComputerOperation::Snapshot => executor.snapshot(generation, &params),
        ComputerOperation::Close => unreachable!(),
    }
}

pub fn dispatch_node_worker_computer_command(
    input: &ComputerCommandInput,
    current_generation: Option<&str>,
) -> Result<String, &'static str> {
    if input.operation == ComputerOperation::Close {
        return Ok(json!({"status": "closed"}).to_string());
    }
    let _ = current_generation;
    Err(COMPUTER_DRIVER_UNAVAILABLE)
}
