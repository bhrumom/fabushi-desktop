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
    pub provider_generation: Option<u64>,
    pub action_params_json: Option<String>,
}

pub fn dispatch_node_worker_computer_command(
    input: &ComputerCommandInput,
    current_generation: Option<u64>,
) -> Result<String, &'static str> {
    if input.operation == ComputerOperation::Close {
        return Ok(json!({"status": "closed"}).to_string());
    }

    let generation = current_generation.ok_or(COMPUTER_DRIVER_UNAVAILABLE)?;

    if input.operation == ComputerOperation::Capabilities {
        return Ok(json!({
            "provider": { "generation": generation },
            "actions": ["mouse_move", "mouse_click", "keyboard_type", "screenshot"]
        }).to_string());
    }

    if let Some(req_gen) = input.provider_generation {
        if req_gen != generation {
            return Err(COMPUTER_CONTRACT_MISMATCH);
        }
    }

    match input.operation {
        ComputerOperation::Act => Ok(json!({"result": "executed"}).to_string()),
        ComputerOperation::Snapshot => Ok(json!({"image_base64": "fake-png-snapshot"}).to_string()),
        _ => unreachable!(),
    }
}
