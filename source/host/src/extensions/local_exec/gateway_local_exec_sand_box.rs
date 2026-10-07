use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{Value, json};

use super::local_exec_bridge::{
    LocalExecComputer, SandLocalExecBridge, SAND_NO_LOCAL_MACHINE_MESSAGE,
};
use super::local_exec_error::SandLocalExecError;
use super::local_exec_failure_classifier::{
    LocalExecFailureClass, classify_local_exec_failure,
};
use super::production::{
    GatewayExecControl, ProductionExecClientPayload, ProductionShellStreamEvent,
    RemoteResourceAccessor, PRODUCTION_LOCAL_EXEC_CODEC,
};
use crate::extensions::local_tool_permission::extension::HostLocalToolPermissionExtension;
use crate::extensions::local_tool_permission::local_tool_permission_controller::{
    SandLocalToolRequest, SandLocalToolScope,
};
use crate::r#box::box_transfer::TransferBox;

pub const FALLBACK_TERMINALS_FOLDER: &str = "terminals";
pub const DEFAULT_MAX_LOCAL_EXEC_FILE_BYTES: usize = 100 * 1024 * 1024;

pub fn describe_local_exec_bytes(bytes: usize) -> String {
    format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
}

pub fn local_exec_file_too_large_message(actual_bytes: usize, max_bytes: usize) -> String {
    format!(
        "File is {}, which exceeds Fabushi's {} limit for reading or transferring a single file over local-exec. Read a slice with offset/limit, or use a shell command (grep, head, tail) to extract just what you need.",
        describe_local_exec_bytes(actual_bytes),
        describe_local_exec_bytes(max_bytes),
    )
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GatewayLocalToolScope {
    pub agent_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayLocalExecFailureReport {
    pub error_class: LocalExecFailureClass,
    pub errno: Option<String>,
    pub site: String,
    pub conversation_id: Option<String>,
}

pub type GatewayLocalExecFailureReporter =
    Arc<dyn Fn(GatewayLocalExecFailureReport) + Send + Sync>;

pub trait GatewayLocalToolGate: Send + Sync {
    fn blocked_reason(&self) -> Option<String>;
    fn requires_approval(&self) -> bool;
    fn authorize(
        &self,
        scope: &GatewayLocalToolScope,
        action: &str,
        target: &str,
    ) -> Result<Option<String>, SandLocalExecError>;
}

fn authorize_host_local_tool(
    permission: &HostLocalToolPermissionExtension,
    scope: &GatewayLocalToolScope,
    action: &str,
    target: &str,
    direction_epoch: Option<u64>,
) -> Result<Option<String>, SandLocalExecError> {
    let sand_scope = scope.agent_id.as_ref().map(|agent_id| SandLocalToolScope {
        agent_id: agent_id.clone(),
        tool_call_id: scope.tool_call_id.clone(),
        action: scope.action.clone(),
        direction_epoch,
    });
    let decision = HostLocalToolPermissionExtension::authorize(
        permission,
        sand_scope.as_ref(),
        &SandLocalToolRequest::simple(action, target),
    );
    if decision.allowed {
        Ok(decision.approval_id)
    } else {
        Err(SandLocalExecError::new(
            decision
                .reason
                .unwrap_or_else(|| "Local tool permission denied".to_string()),
        ))
    }
}

impl GatewayLocalToolGate for HostLocalToolPermissionExtension {
    fn blocked_reason(&self) -> Option<String> {
        HostLocalToolPermissionExtension::blocked_reason(self)
    }

    fn requires_approval(&self) -> bool {
        HostLocalToolPermissionExtension::requires_approval(self)
    }

    fn authorize(
        &self,
        scope: &GatewayLocalToolScope,
        action: &str,
        target: &str,
    ) -> Result<Option<String>, SandLocalExecError> {
        authorize_host_local_tool(self, scope, action, target, None)
    }
}

#[derive(Clone)]
pub struct TurnScopedGatewayLocalToolGate {
    permission: Arc<HostLocalToolPermissionExtension>,
    direction_epoch: u64,
}

impl TurnScopedGatewayLocalToolGate {
    pub fn new(
        permission: Arc<HostLocalToolPermissionExtension>,
        direction_epoch: u64,
    ) -> Self {
        Self {
            permission,
            direction_epoch,
        }
    }
}

impl GatewayLocalToolGate for TurnScopedGatewayLocalToolGate {
    fn blocked_reason(&self) -> Option<String> {
        self.permission.blocked_reason()
    }

    fn requires_approval(&self) -> bool {
        self.permission.requires_approval()
    }

    fn authorize(
        &self,
        scope: &GatewayLocalToolScope,
        action: &str,
        target: &str,
    ) -> Result<Option<String>, SandLocalExecError> {
        authorize_host_local_tool(
            self.permission.as_ref(),
            scope,
            action,
            target,
            Some(self.direction_epoch),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayLocalExecReady {
    pub vnc_url: String,
    pub terminals_folder: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GatewayLocalExecShellResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<u32>,
    pub cwd: Option<String>,
    pub aborted: bool,
}

#[derive(Clone)]
pub struct GatewayLocalExecManager {
    sandbox: GatewayLocalExecSandBox,
    next_exec_id: Arc<AtomicU32>,
}

#[derive(Clone)]
pub struct GatewayLocalExecSandBox {
    bridge: SandLocalExecBridge,
    gate: Arc<dyn GatewayLocalToolGate>,
    computer_id: Option<String>,
    max_file_bytes: usize,
    failure_reporter: Option<GatewayLocalExecFailureReporter>,
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
            failure_reporter: None,
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

    pub fn with_failure_reporter(
        mut self,
        failure_reporter: GatewayLocalExecFailureReporter,
    ) -> Self {
        self.failure_reporter = Some(failure_reporter);
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

    pub fn exec_manager(&self) -> GatewayLocalExecManager {
        GatewayLocalExecManager {
            sandbox: self.clone(),
            next_exec_id: Arc::new(AtomicU32::new(1)),
        }
    }

    pub fn remote_resource_accessor(&self) -> RemoteResourceAccessor<GatewayLocalExecManager> {
        PRODUCTION_LOCAL_EXEC_CODEC.create_remote_accessor(Arc::new(self.exec_manager()))
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
        self.upload_file_with_scope(
            &GatewayLocalToolScope {
                agent_id: agent_id.map(str::to_string),
                ..GatewayLocalToolScope::default()
            },
            box_path,
            data,
        )
    }

    pub fn upload_file_with_scope(
        &self,
        scope: &GatewayLocalToolScope,
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
        self.bridge
            .assert_computer_available(self.computer_id.as_deref(), "upload", scope.agent_id.as_deref())?;
        let approval_id = self
            .gate
            .authorize(scope, "write-file", box_path)?;
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
        self.download_file_with_scope(
            &GatewayLocalToolScope {
                agent_id: agent_id.map(str::to_string),
                ..GatewayLocalToolScope::default()
            },
            box_path,
        )
    }

    pub fn download_file_with_scope(
        &self,
        scope: &GatewayLocalToolScope,
        box_path: &str,
    ) -> Result<Vec<u8>, SandLocalExecError> {
        self.check_blocked()?;
        self.bridge
            .assert_computer_available(self.computer_id.as_deref(), "download", scope.agent_id.as_deref())?;
        let approval_id = self
            .gate
            .authorize(scope, "read-file", box_path)?;
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

    fn report_exec_failure(&self, scope: &GatewayLocalToolScope, message: &str) {
        let Some(reporter) = self.failure_reporter.as_ref() else {
            return;
        };
        let classification = classify_local_exec_failure(message);
        reporter(GatewayLocalExecFailureReport {
            error_class: classification.error_class,
            errno: classification.errno,
            site: "exec".to_string(),
            conversation_id: scope.agent_id.clone(),
        });
    }

    fn check_blocked(&self) -> Result<(), SandLocalExecError> {
        match self.gate.blocked_reason() {
            Some(reason) => Err(SandLocalExecError::new(reason)),
            None => Ok(()),
        }
    }
}


impl GatewayLocalExecManager {
    pub fn terminals_folder(&self) -> String {
        self.sandbox.terminals_folder()
    }

    fn next_id(&self) -> u32 {
        let id = self.next_exec_id.fetch_add(1, Ordering::Relaxed);
        if id == 0 { 1 } else { id }
    }

    fn open_exec(
        &self,
        scope: &GatewayLocalToolScope,
        action: &str,
        target: &str,
        server_message: Value,
    ) -> Result<super::local_exec_bridge::LocalExecRequest, SandLocalExecError> {
        self.sandbox.check_blocked()?;
        self.sandbox.bridge.assert_computer_available(
            self.sandbox.computer_id.as_deref(),
            "exec",
            scope.agent_id.as_deref(),
        )?;
        let approval_id = self.sandbox.gate.authorize(scope, action, target)?;
        let mut frame = json!({
            "kind": "exec",
            "serverMessage": server_message,
        });
        if let Some(approval_id) = approval_id {
            frame["approvalId"] = Value::String(approval_id);
        }
        self.sandbox.bridge.request(frame, self.sandbox.computer_id.as_deref())
    }

    pub fn execute_shell(
        &self,
        scope: &GatewayLocalToolScope,
        command: &str,
        working_directory: &str,
        tool_call_id: &str,
    ) -> Result<GatewayLocalExecShellResult, SandLocalExecError> {
        let message = PRODUCTION_LOCAL_EXEC_CODEC.shell_stream_server_message(
            self.next_id(), command, working_directory, tool_call_id,
        );
        let mut request = self.open_exec(scope, "run-command", command, message)?;
        let mut result = GatewayLocalExecShellResult::default();
        loop {
            let response = request
                .recv_timeout(Duration::from_secs(10))
                .map_err(|_| SandLocalExecError::new("local-exec exec response timed out"))?;
            match response.get("kind").and_then(Value::as_str) {
                Some("client") => {
                    let message = response.get("message").cloned().ok_or_else(|| {
                        SandLocalExecError::new("local-exec client frame is missing message")
                    })?;
                    let decoded = PRODUCTION_LOCAL_EXEC_CODEC
                        .decode_client(message)
                        .map_err(SandLocalExecError::new)?;
                    match decoded.payload() {
                        ProductionExecClientPayload::ShellStream(event) => match event {
                            ProductionShellStreamEvent::Stdout(data) => result.stdout.push_str(data),
                            ProductionShellStreamEvent::Stderr(data) => result.stderr.push_str(data),
                            ProductionShellStreamEvent::Exit { code, cwd, aborted } => {
                                result.exit_code = Some(*code);
                                result.cwd = (!cwd.is_empty()).then(|| cwd.clone());
                                result.aborted = *aborted;
                                request.close();
                                return Ok(result);
                            }
                            ProductionShellStreamEvent::Rejected(reason)
                            | ProductionShellStreamEvent::PermissionDenied(reason) => {
                                request.close();
                                return Err(SandLocalExecError::new(reason.clone()));
                            }
                            ProductionShellStreamEvent::Start
                            | ProductionShellStreamEvent::Backgrounded
                            | ProductionShellStreamEvent::Other => {}
                        },
                        ProductionExecClientPayload::ReadResult(_)
                        | ProductionExecClientPayload::Other => {}
                    }
                }
                Some("control") => {
                    let message = response.get("message").ok_or_else(|| {
                        SandLocalExecError::new("local-exec control frame is missing message")
                    })?;
                    match PRODUCTION_LOCAL_EXEC_CODEC
                        .decode_control(message)
                        .map_err(SandLocalExecError::new)?
                    {
                        GatewayExecControl::Throw { error, stack_trace } => {
                            self.sandbox.report_exec_failure(scope, &error);
                            request.close();
                            let detail = stack_trace
                                .filter(|stack| !stack.trim().is_empty())
                                .map(|stack| format!("{error}\n{stack}"))
                                .unwrap_or(error);
                            return Err(SandLocalExecError::new(detail));
                        }
                        GatewayExecControl::StreamClose => {
                            request.close();
                            return Ok(result);
                        }
                        GatewayExecControl::Unknown => {}
                    }
                }
                Some("file-error") => {
                    let error = response.get("error").and_then(Value::as_str)
                        .unwrap_or(SAND_NO_LOCAL_MACHINE_MESSAGE).to_string();
                    request.close();
                    return Err(SandLocalExecError::new(error));
                }
                _ => {}
            }
        }
    }

    pub fn execute_read(
        &self,
        scope: &GatewayLocalToolScope,
        path: &str,
        tool_call_id: &str,
        offset: Option<i32>,
        limit: Option<u32>,
        encoding_hint: Option<&str>,
    ) -> Result<Value, SandLocalExecError> {
        let message = PRODUCTION_LOCAL_EXEC_CODEC.read_server_message(
            self.next_id(), path, tool_call_id, offset, limit, encoding_hint,
        );
        let mut request = self.open_exec(scope, "read-file", path, message)?;
        loop {
            let response = request
                .recv_timeout(Duration::from_secs(10))
                .map_err(|_| SandLocalExecError::new("local-exec exec response timed out"))?;
            match response.get("kind").and_then(Value::as_str) {
                Some("client") => {
                    let message = response.get("message").cloned().ok_or_else(|| {
                        SandLocalExecError::new("local-exec client frame is missing message")
                    })?;
                    let decoded = PRODUCTION_LOCAL_EXEC_CODEC
                        .decode_client(message)
                        .map_err(SandLocalExecError::new)?;
                    if let ProductionExecClientPayload::ReadResult(result) = decoded.payload() {
                        let result = result.clone();
                        request.close();
                        return Ok(result);
                    }
                }
                Some("control") => {
                    let message = response.get("message").ok_or_else(|| {
                        SandLocalExecError::new("local-exec control frame is missing message")
                    })?;
                    match PRODUCTION_LOCAL_EXEC_CODEC
                        .decode_control(message)
                        .map_err(SandLocalExecError::new)?
                    {
                        GatewayExecControl::Throw { error, stack_trace } => {
                            self.sandbox.report_exec_failure(scope, &error);
                            request.close();
                            let detail = stack_trace
                                .filter(|stack| !stack.trim().is_empty())
                                .map(|stack| format!("{error}\n{stack}"))
                                .unwrap_or(error);
                            return Err(SandLocalExecError::new(detail));
                        }
                        GatewayExecControl::StreamClose => {
                            request.close();
                            return Err(SandLocalExecError::new(
                                "local-exec read stream closed without a ReadResult",
                            ));
                        }
                        GatewayExecControl::Unknown => {}
                    }
                }
                Some("file-error") => {
                    let error = response.get("error").and_then(Value::as_str)
                        .unwrap_or(SAND_NO_LOCAL_MACHINE_MESSAGE).to_string();
                    request.close();
                    return Err(SandLocalExecError::new(error));
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalExecTransferContext {
    pub agent_id: String,
    pub tool_call_id: String,
}

impl TransferBox<LocalExecTransferContext> for GatewayLocalExecSandBox {
    type Error = SandLocalExecError;

    fn download_file(
        &self,
        ctx: &LocalExecTransferContext,
        _agent_id: &str,
        path: &str,
    ) -> Result<Vec<u8>, Self::Error> {
        self.download_file_with_scope(
            &GatewayLocalToolScope {
                agent_id: Some(ctx.agent_id.clone()),
                tool_call_id: Some(ctx.tool_call_id.clone()),
                action: None,
            },
            path,
        )
    }

    fn upload_file(
        &self,
        ctx: &LocalExecTransferContext,
        _agent_id: &str,
        path: &str,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.upload_file_with_scope(
            &GatewayLocalToolScope {
                agent_id: Some(ctx.agent_id.clone()),
                tool_call_id: Some(ctx.tool_call_id.clone()),
                action: None,
            },
            path,
            data,
        )
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
