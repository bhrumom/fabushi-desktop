use std::error::Error;

use thiserror::Error;

pub const SAND_BOX_NOT_READY_MESSAGE: &str =
    "The computer is currently reported as preparing. No command was executed; readiness has not been established.";
pub const SAND_BOX_NOT_RESPONDING_MESSAGE: &str =
    "The computer's execution service did not respond. No successful connection was established.";
pub const SAND_BOX_NO_MONITOR_AVAILABLE_MESSAGE: &str =
    "No private desktop monitor is available for this agent. This does not establish whether the computer is starting up.";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct SandBoxDaemonUnreachableError {
    pub outcome: String,
    pub message: String,
}

impl SandBoxDaemonUnreachableError {
    pub fn new(outcome: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            outcome: outcome.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct SandBoxNoMonitorAvailableError(pub String);

impl Default for SandBoxNoMonitorAvailableError {
    fn default() -> Self {
        Self("No private desktop monitor is available on the shared box.".into())
    }
}

pub fn box_not_ready_message_for_error(error: &(dyn Error + 'static)) -> String {
    let mut node = Some(error);
    let mut kind = "unclassified".to_string();
    let mut diagnostics = Vec::new();
    // Error implementations may have cycles. Bound both traversal and output.
    for _ in 0..16 {
        let Some(current) = node else { break; };
        let text: String = current.to_string().chars().take(1024)
            .map(|ch| if ch.is_control() { ' ' } else { ch }).collect();
        if !diagnostics.contains(&text) { diagnostics.push(text); }
        if current.downcast_ref::<SandBoxNoMonitorAvailableError>().is_some() {
            return SAND_BOX_NO_MONITOR_AVAILABLE_MESSAGE.to_string();
        }
        if let Some(daemon) = current.downcast_ref::<SandBoxDaemonUnreachableError>() {
            kind = match daemon.outcome.as_str() {
                "refused" => "daemon_refused",
                "timeout" => "daemon_timeout",
                "crash" => "daemon_crash",
                _ => "daemon_unreachable",
            }.to_string();
        } else if kind == "unclassified" {
            if let Some(io) = current.downcast_ref::<std::io::Error>() {
                kind = format!("io/{:?}", io.kind());
            }
        }
        node = current.source();
    }
    let detail: String = diagnostics.join("; caused by: ").chars().take(2048).collect();
    format!("Computer connection failed [{kind}]; startup state is unknown. Diagnostic: {detail}")
}

#[derive(Debug)]
pub struct NoMonitorComputerUseExecutor {
    identity: u8,
}

impl NoMonitorComputerUseExecutor {
    pub const fn detached() -> Self {
        Self { identity: 1 }
    }

    pub fn execute<T>(&self) -> Result<T, SandBoxNoMonitorAvailableError> {
        let _ = self.identity;
        Err(SandBoxNoMonitorAvailableError::default())
    }
}

pub static NO_MONITOR_COMPUTER_USE_EXECUTOR: NoMonitorComputerUseExecutor =
    NoMonitorComputerUseExecutor { identity: 0 };

pub fn is_no_monitor_computer_use_executor(executor: &NoMonitorComputerUseExecutor) -> bool {
    std::ptr::eq(executor, &NO_MONITOR_COMPUTER_USE_EXECUTOR)
}

pub const SAND_BOX_PRIMARY_WINDOW_INDEX: u32 = 1;
pub const SAND_BOX_FIRST_FORK_WINDOW_INDEX: u32 = 2;

pub fn is_primary_window_index(window_index: u32) -> bool {
    window_index <= SAND_BOX_PRIMARY_WINDOW_INDEX
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("This box does not support environment sync.")]
pub struct BoxEnvironmentSyncUnsupportedError;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("This box does not support running MCP servers.")]
pub struct BoxMcpUnsupportedError;
