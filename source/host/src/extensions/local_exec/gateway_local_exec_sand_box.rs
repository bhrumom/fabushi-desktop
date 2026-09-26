use std::sync::Arc;
use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{Value, json};

use super::local_exec_bridge::{
    LocalExecComputer, SandLocalExecBridge, SAND_NO_LOCAL_MACHINE_MESSAGE,
};
use super::local_exec_error::SandLocalExecError;

pub const FALLBACK_TERMINALS_FOLDER: &str = "terminals";
pub const DEFAULT_MAX_LOCAL_EXEC_FILE_BYTES: usize = 100 * 1024 * 1024;

pub fn describe_local_exec_bytes(bytes: usize) -> String {
    format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
}

pub fn local_exec_file_too_large_message(actual_bytes: usize, max_bytes: usize) -> String {
    format!(
        "File is {}, which exceeds Grok Bot's {} limit for reading or transferring a single file over local-exec. Read a slice with offset/limit, or use a shell command (grep, head, tail) to extract just what you need.",
        describe_local_exec_bytes(actual_bytes),
        describe_local_exec_bytes(max_bytes),
    )
}

pub trait GatewayLocalToolGate: Send + Sync {
    fn blocked_reason(&self) -> Option<String>;
    fn requires_approval(&self) -> bool;
    fn authorize(
        &self,
        agent_id: Option<&str>,
        action: &str,
        target: &str,
    ) -> Result<Option<String>, SandLocalExecError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayLocalExecReady {
    pub vnc_url: String,
    pub terminals_folder: String,
}

#[derive(Clone)]
pub struct GatewayLocalExecSandBox {
    bridge: SandLocalExecBridge,
    gate: Arc<dyn GatewayLocalToolGate>,
    computer_id: Option<String>,
    max_file_bytes: usize,
}

impl GatewayLocalExecSandBox {
    pub fn new(
        bridge: SandLocalExecBridge,
        gate: Arc<dyn GatewayLocalToolGate>,
    ) -> Self {
        Self {
            bridge,
            gate,
            computer_id: None,
            max_file_bytes: DEFAULT_MAX_LOCAL_EXEC_FILE_BYTES,
        }
    }

    pub fn for_computer(mut self, computer_id: impl Into<String>) -> Self {
        self.computer_id = Some(computer_id.into());
        self
    }

    pub fn with_max_file_bytes(mut self, max_file_bytes: usize) -> Self {
        self.max_file_bytes = max_file_bytes;
        self
    }

    pub fn terminals_folder(&self) -> String {
        self.bridge
            .get_provider_info()
            .map(|info| info.terminals_folder)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| FALLBACK_TERMINALS_FOLDER.to_string())
    }

    pub fn ensure_ready(&self) -> GatewayLocalExecReady {
        GatewayLocalExecReady {
            vnc_url: String::new(),
            terminals_folder: self.terminals_folder(),
        }
    }

    pub fn run_state(&self) -> &'static str {
        if self.bridge.has_provider() {
            "running"
        } else {
            "absent"
        }
    }

    pub fn list_boxes(&self) -> Vec<Value> {
        Vec::new()
    }

    pub fn upload_file(
        &self,
        agent_id: Option<&str>,
        box_path: &str,
        data: &[u8],
    ) -> Result<(), SandLocalExecError> {
        if data.len() > self.max_file_bytes {
            return Err(SandLocalExecError::new(local_exec_file_too_large_message(
                data.len(),
                self.max_file_bytes,
            )));
        }
        self.check_blocked()?;
        let approval_id = self
            .gate
            .authorize(agent_id, "write-file", box_path)?;
        let mut frame = json!({
            "kind": "upload",
            "path": box_path,
            "bytesBase64": BASE64.encode(data),
        });
        if let Some(approval_id) = approval_id {
            frame["approvalId"] = Value::String(approval_id);
        }
        let mut request = self.bridge.request(frame, self.computer_id.as_deref())?;
        loop {
            let response = request
                .recv_timeout(Duration::from_secs(10))
                .map_err(|_| SandLocalExecError::new(SAND_NO_LOCAL_MACHINE_MESSAGE))?;
            match response.get("kind").and_then(Value::as_str) {
                Some("file") => {
                    request.close();
                    return Ok(());
                }
                Some("file-error") => {
                    let message = response
                        .get("error")
                        .and_then(Value::as_str)
                        .unwrap_or(SAND_NO_LOCAL_MACHINE_MESSAGE);
                    request.close();
                    return Err(SandLocalExecError::new(message));
                }
                _ => {}
            }
        }
    }

    pub fn download_file(
        &self,
        agent_id: Option<&str>,
        box_path: &str,
    ) -> Result<Vec<u8>, SandLocalExecError> {
        self.check_blocked()?;
        let approval_id = self
            .gate
            .authorize(agent_id, "read-file", box_path)?;
        let mut frame = json!({
            "kind": "download",
            "path": box_path,
        });
        if let Some(approval_id) = approval_id {
            frame["approvalId"] = Value::String(approval_id);
        }
        let mut request = self.bridge.request(frame, self.computer_id.as_deref())?;
        loop {
            let response = request
                .recv_timeout(Duration::from_secs(10))
                .map_err(|_| SandLocalExecError::new(SAND_NO_LOCAL_MACHINE_MESSAGE))?;
            match response.get("kind").and_then(Value::as_str) {
                Some("file") => {
                    let encoded = response
                        .get("bytesBase64")
                        .and_then(Value::as_str)
                        .ok_or_else(|| SandLocalExecError::new(
                            "local-exec download response is missing bytesBase64",
                        ))?;
                    let bytes = BASE64
                        .decode(encoded)
                        .map_err(|error| SandLocalExecError::new(format!(
                            "local-exec download returned invalid base64: {error}"
                        )))?;
                    if bytes.len() > self.max_file_bytes {
                        request.close();
                        return Err(SandLocalExecError::new(local_exec_file_too_large_message(
                            bytes.len(),
                            self.max_file_bytes,
                        )));
                    }
                    request.close();
                    return Ok(bytes);
                }
                Some("file-error") => {
                    let message = response
                        .get("error")
                        .and_then(Value::as_str)
                        .unwrap_or(SAND_NO_LOCAL_MACHINE_MESSAGE);
                    request.close();
                    return Err(SandLocalExecError::new(message));
                }
                _ => {}
            }
        }
    }

    fn check_blocked(&self) -> Result<(), SandLocalExecError> {
        match self.gate.blocked_reason() {
            Some(reason) => Err(SandLocalExecError::new(reason)),
            None => Ok(()),
        }
    }
}

#[derive(Clone)]
pub struct BridgeUserComputers {
    bridge: SandLocalExecBridge,
    gate: Arc<dyn GatewayLocalToolGate>,
}

impl BridgeUserComputers {
    pub fn new(
        bridge: SandLocalExecBridge,
        gate: Arc<dyn GatewayLocalToolGate>,
    ) -> Self {
        Self { bridge, gate }
    }

    pub fn list(&self) -> Vec<LocalExecComputer> {
        self.bridge.list_computers()
    }

    pub fn resolve(&self, id: Option<&str>) -> Option<(LocalExecComputer, GatewayLocalExecSandBox)> {
        let selected = match id {
            Some(id) => self
                .bridge
                .list_computers()
                .into_iter()
                .find(|computer| computer.id == id),
            None => self.bridge.active_computer(),
        }?;
        let box_owner = GatewayLocalExecSandBox::new(
            self.bridge.clone(),
            Arc::clone(&self.gate),
        )
        .for_computer(selected.id.clone());
        Some((selected, box_owner))
    }
}

pub fn create_bridge_user_computers(
    bridge: SandLocalExecBridge,
    gate: Arc<dyn GatewayLocalToolGate>,
) -> BridgeUserComputers {
    BridgeUserComputers::new(bridge, gate)
}
