use std::collections::{HashMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, AtomicU64, Ordering},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::local_exec_bridge::{
    LocalExecCancellationHandle, LocalExecComputer, SAND_NO_LOCAL_MACHINE_MESSAGE,
    SandLocalExecBridge,
};
use super::local_exec_error::SandLocalExecError;
use super::local_exec_failure_classifier::{
    CommandErrorResult, CommandTermination, LocalExecFailureClass, classify_local_exec_failure,
    format_command_error,
};
use super::production::{
    GatewayExecControl, PRODUCTION_LOCAL_EXEC_CODEC, ProductionExecClientPayload,
    ProductionShellStreamEvent, RemoteResourceAccessor,
};
use crate::r#box::box_transfer::TransferBox;
use crate::durable_file_policy::SAND_LOCAL_EXEC_OPERATION_JOURNAL_FILE_NAME;
use crate::extensions::local_tool_permission::extension::HostLocalToolPermissionExtension;
use crate::extensions::local_tool_permission::local_tool_permission_controller::{
    SandLocalToolRequest, SandLocalToolScope,
};
use crate::extensions::process::command_custody::CommandProcessCustody;
use crate::host_paths::get_sand_root_dir;

pub const FALLBACK_TERMINALS_FOLDER: &str = "terminals";
pub const DEFAULT_MAX_LOCAL_EXEC_FILE_BYTES: usize = 100 * 1024 * 1024;
pub const MAX_LOCAL_EXEC_SHELL_STREAM_BYTES: usize = 256 * 1024;
pub const MAX_LOCAL_EXEC_DURABLE_OUTPUT_BYTES: usize = 64 * 1024;
pub const DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT: usize = 128;
// Version 3 preserves unresolved custody under bounded retention. Version-2
// readers must reject this image instead of applying their older eviction rule.
pub const LOCAL_EXEC_OPERATION_JOURNAL_VERSION: u32 = 3;
static LOCAL_EXEC_OPERATION_JOURNAL_TMP_COUNTER: AtomicU64 = AtomicU64::new(1);

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

pub type GatewayLocalExecFailureReporter = Arc<dyn Fn(GatewayLocalExecFailureReport) + Send + Sync>;

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
        session_key: None,
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
        Err(SandLocalExecError::new(decision.reason.unwrap_or_else(
            || "Local tool permission denied".to_string(),
        )))
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
    pub fn new(permission: Arc<HostLocalToolPermissionExtension>, direction_epoch: u64) -> Self {
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
    pub request_id: String,
    pub outcome_unknown: bool,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatewayLocalExecOperationStatus {
    Accepted,
    Running,
    Completed,
    Cancelled,
    OutcomeUnknown,
    Failed,
}

impl GatewayLocalExecOperationStatus {
    fn is_known_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayLocalExecOperationRecord {
    pub operation_id: String,
    pub agent_id: Option<String>,
    pub bridge_request_id: String,
    pub status: GatewayLocalExecOperationStatus,
    pub exit_code: Option<u32>,
    pub aborted: bool,
    #[serde(default)]
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
    #[serde(default)]
    pub stdout_truncated: bool,
    #[serde(default)]
    pub stderr_truncated: bool,
    #[serde(default)]
    pub settled_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GatewayLocalExecOperationJournal {
    version: u32,
    records: Vec<GatewayLocalExecOperationRecord>,
}

#[derive(Clone)]
struct GatewayLocalExecActiveOperation {
    agent_id: Option<String>,
    bridge_request_id: String,
    cancellation: LocalExecCancellationHandle,
    started: bool,
}

struct GatewayLocalExecOperationState {
    active_operations: HashMap<String, GatewayLocalExecActiveOperation>,
    operation_history: VecDeque<GatewayLocalExecOperationRecord>,
    operation_history_limit: usize,
    operation_journal_path: PathBuf,
    operation_journal_error: Option<String>,
    #[cfg(test)]
    before_journal_publish: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl GatewayLocalExecOperationState {
    fn load(path: PathBuf) -> Self {
        let mut state = Self {
            active_operations: HashMap::new(),
            operation_history: VecDeque::new(),
            operation_history_limit: DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT,
            operation_journal_path: path,
            operation_journal_error: None,
            #[cfg(test)]
            before_journal_publish: None,
        };
        if let Err(error) = GatewayLocalExecManager::load_operation_history(&mut state) {
            // A parsed image remains queryable when publishing its migration or
            // recovery fails. Journal health still forbids every new effect.
            state.operation_journal_error = Some(error.message);
        }
        state
    }
}

/// One Host extension shares this owner across its permission-scoped facades.
/// The state lock covers transitions and durable publication together: a newer
/// operation snapshot cannot be overwritten by an earlier in-flight writer.
pub(crate) struct GatewayLocalExecOperationOwner {
    state: Mutex<Option<GatewayLocalExecOperationState>>,
    journal_path: Option<PathBuf>,
}

impl GatewayLocalExecOperationOwner {
    pub(crate) fn new() -> Self {
        Self { state: Mutex::new(None), journal_path: None }
    }

    fn with_state<T>(
        &self,
        action: impl FnOnce(&mut GatewayLocalExecOperationState) -> T,
    ) -> T {
        let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        // Resolve the configured root only on first use, not while Host
        // extensions are being constructed before startup configuration.
        let state = state.get_or_insert_with(|| {
            GatewayLocalExecOperationState::load(self.journal_path.clone().unwrap_or_else(|| {
                get_sand_root_dir().join(SAND_LOCAL_EXEC_OPERATION_JOURNAL_FILE_NAME)
            }))
        });
        action(state)
    }

    #[cfg(test)]
    fn with_journal_path(path: PathBuf) -> Self {
        Self {
            state: Mutex::new(None),
            journal_path: Some(path),
        }
    }
}

#[derive(Clone)]
pub struct GatewayLocalExecManager {
    sandbox: GatewayLocalExecSandBox,
    process_custody: Arc<CommandProcessCustody>,
    next_exec_id: Arc<AtomicU32>,
    operation_owner: Arc<GatewayLocalExecOperationOwner>,
}

#[derive(Clone)]
pub struct GatewayLocalExecSandBox {
    bridge: SandLocalExecBridge,
    gate: Arc<dyn GatewayLocalToolGate>,
    process_custody: Arc<CommandProcessCustody>,
    operation_owner: Arc<GatewayLocalExecOperationOwner>,
    computer_id: Option<String>,
    max_file_bytes: usize,
    failure_reporter: Option<GatewayLocalExecFailureReporter>,
}

impl GatewayLocalExecSandBox {
    pub fn new(bridge: SandLocalExecBridge, gate: Arc<dyn GatewayLocalToolGate>) -> Self {
        Self {
            bridge,
            gate,
            process_custody: Arc::new(CommandProcessCustody::new()),
            operation_owner: Arc::new(GatewayLocalExecOperationOwner::new()),
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

    pub fn with_process_custody(mut self, process_custody: Arc<CommandProcessCustody>) -> Self {
        self.process_custody = process_custody;
        self
    }

    pub(crate) fn with_operation_owner(
        mut self,
        operation_owner: Arc<GatewayLocalExecOperationOwner>,
    ) -> Self {
        self.operation_owner = operation_owner;
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
        GatewayLocalExecManager::new(self.clone())
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
        const CHUNK_BYTES: usize = 256 * 1024;
        if data.len() > self.max_file_bytes {
            return Err(SandLocalExecError::new(local_exec_file_too_large_message(
                data.len(),
                self.max_file_bytes,
            )));
        }
        self.check_blocked()?;
        self.bridge.assert_computer_available(
            self.computer_id.as_deref(),
            "upload",
            scope.agent_id.as_deref(),
        )?;
        let approval_id = self.gate.authorize(scope, "write-file", box_path)?;
        let transfer_id = Uuid::new_v4().to_string();
        let digest = Sha256::digest(data);
        let sha256 = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();

        let send_frame = |mut frame: Value| -> Result<(), SandLocalExecError> {
            if let Some(approval_id) = approval_id.as_ref() {
                frame["approvalId"] = Value::String(approval_id.clone());
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
        };

        send_frame(json!({
            "kind": "upload-start",
            "transferId": transfer_id,
            "path": box_path,
            "totalBytes": data.len(),
            "sha256": sha256,
        }))?;

        for (index, chunk) in data.chunks(CHUNK_BYTES).enumerate() {
            send_frame(json!({
                "kind": "upload-chunk",
                "transferId": transfer_id,
                "path": box_path,
                "offset": index * CHUNK_BYTES,
                "bytesBase64": BASE64.encode(chunk),
            }))?;
        }

        send_frame(json!({
            "kind": "upload-commit",
            "transferId": transfer_id,
            "path": box_path,
        }))
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
        self.bridge.assert_computer_available(
            self.computer_id.as_deref(),
            "download",
            scope.agent_id.as_deref(),
        )?;
        let approval_id = self.gate.authorize(scope, "read-file", box_path)?;
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
                        .ok_or_else(|| {
                            SandLocalExecError::new(
                                "local-exec download response is missing bytesBase64",
                            )
                        })?;
                    let bytes = BASE64.decode(encoded).map_err(|error| {
                        SandLocalExecError::new(format!(
                            "local-exec download returned invalid base64: {error}"
                        ))
                    })?;
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

fn append_bounded_output(target: &mut String, chunk: &str, max_bytes: usize) -> bool {
    let remaining = max_bytes.saturating_sub(target.len());
    if remaining == 0 {
        return !chunk.is_empty();
    }
    if chunk.len() <= remaining {
        target.push_str(chunk);
        return false;
    }
    let mut end = 0;
    for (index, ch) in chunk.char_indices() {
        let next = index + ch.len_utf8();
        if next > remaining {
            break;
        }
        end = next;
    }
    target.push_str(&chunk[..end]);
    true
}

fn append_bounded_shell_output(target: &mut String, chunk: &str) -> bool {
    append_bounded_output(target, chunk, MAX_LOCAL_EXEC_SHELL_STREAM_BYTES)
}

fn durable_shell_output(value: &str, already_truncated: bool) -> (String, bool) {
    let mut output = String::new();
    let truncated = append_bounded_output(&mut output, value, MAX_LOCAL_EXEC_DURABLE_OUTPUT_BYTES)
        || already_truncated;
    (output, truncated)
}

impl GatewayLocalExecManager {
    fn new(sandbox: GatewayLocalExecSandBox) -> Self {
        let process_custody = Arc::clone(&sandbox.process_custody);
        let operation_owner = Arc::clone(&sandbox.operation_owner);
        Self {
            sandbox,
            process_custody,
            next_exec_id: Arc::new(AtomicU32::new(1)),
            operation_owner,
        }
    }

    #[cfg(test)]
    fn new_with_operation_journal_path(
        sandbox: GatewayLocalExecSandBox,
        operation_journal_path: PathBuf,
    ) -> Self {
        Self::new(sandbox.with_operation_owner(Arc::new(
            GatewayLocalExecOperationOwner::with_journal_path(operation_journal_path),
        )))
    }

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }

    fn prune_known_operation_history(
        history: &mut VecDeque<GatewayLocalExecOperationRecord>,
        limit: usize,
    ) -> bool {
        let mut remaining = history.len().saturating_sub(limit);
        let requested = remaining;
        // Retain order and skip every unresolved record, even if a legacy
        // image contains more protected custody than the current capacity.
        history.retain(|record| {
            if remaining > 0 && record.status.is_known_terminal() {
                remaining -= 1;
                false
            } else {
                true
            }
        });
        remaining != requested
    }

    fn load_operation_history(
        state: &mut GatewayLocalExecOperationState,
    ) -> Result<(), SandLocalExecError> {
        let bytes = match fs::read(&state.operation_journal_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(SandLocalExecError::new(format!(
                    "local-exec operation journal read failed closed: {error}"
                )));
            }
        };
        let mut journal = serde_json::from_slice::<GatewayLocalExecOperationJournal>(&bytes)
            .map_err(|error| {
                SandLocalExecError::new(format!(
                    "local-exec operation journal is corrupt and execution is disabled: {error}"
                ))
            })?;
        if journal.version != 2 && journal.version != LOCAL_EXEC_OPERATION_JOURNAL_VERSION {
            return Err(SandLocalExecError::new(format!(
                "local-exec operation journal version {} is unsupported; expected 2 or {}; execution is disabled",
                journal.version, LOCAL_EXEC_OPERATION_JOURNAL_VERSION
            )));
        }
        let mut seen = std::collections::HashSet::new();
        if journal
            .records
            .iter()
            .any(|record| !seen.insert(record.operation_id.clone()))
        {
            return Err(SandLocalExecError::new(
                "local-exec operation journal contains duplicate operationId records; execution is disabled",
            ));
        }
        let mut must_publish = journal.version != LOCAL_EXEC_OPERATION_JOURNAL_VERSION;
        for record in &mut journal.records {
            if matches!(
                record.status,
                GatewayLocalExecOperationStatus::Accepted
                    | GatewayLocalExecOperationStatus::Running
            ) {
                record.status = GatewayLocalExecOperationStatus::OutcomeUnknown;
                record.settled_at_ms = Some(Self::now_ms());
                must_publish = true;
            }
        }
        let mut history = journal.records.into_iter().collect::<VecDeque<_>>();
        must_publish |= Self::prune_known_operation_history(
            &mut history,
            state.operation_history_limit.max(1),
        );
        state.operation_history = history;
        if must_publish {
            // Keep parsed custody in the owner before attempting publication.
            // An error may follow atomic replacement; it proves neither that
            // the prior bytes survived nor that the new image is durable.
            Self::persist_operation_snapshot(state).map_err(|error| {
                SandLocalExecError::new(format!(
                    "local-exec operation journal migration, recovery or retention changes could not be durably published; execution is disabled: {}",
                    error.message
                ))
            })?;
        }
        Ok(())
    }

    #[cfg(windows)]
    fn replace_operation_journal(tmp: &Path, path: &Path) -> io::Result<()> {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "Kernel32")]
        unsafe extern "system" {
            fn MoveFileExW(
                existing_file_name: *const u16,
                new_file_name: *const u16,
                flags: u32,
            ) -> i32;
        }
        const MOVEFILE_REPLACE_EXISTING: u32 = 0x00000001;
        const MOVEFILE_WRITE_THROUGH: u32 = 0x00000008;
        let from = tmp
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let to = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let result = unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    #[cfg(not(windows))]
    fn replace_operation_journal(tmp: &Path, path: &Path) -> io::Result<()> {
        fs::rename(tmp, path)
    }

    fn persist_operation_history(
        path: &Path,
        history: &VecDeque<GatewayLocalExecOperationRecord>,
    ) -> io::Result<()> {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "journal path has no parent")
        })?;
        fs::create_dir_all(parent)?;
        let journal = GatewayLocalExecOperationJournal {
            version: LOCAL_EXEC_OPERATION_JOURNAL_VERSION,
            records: history.iter().cloned().collect(),
        };
        let bytes =
            serde_json::to_vec(&journal).map_err(|error| io::Error::other(error.to_string()))?;
        let nonce = LOCAL_EXEC_OPERATION_JOURNAL_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = path.with_extension(format!("tmp-{}-{nonce}", std::process::id()));
        let write_result = (|| -> io::Result<()> {
            let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            Self::replace_operation_journal(&tmp, path)?;
            #[cfg(unix)]
            File::open(parent)?.sync_all()?;
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        write_result
    }

    fn ensure_state_journal_healthy(
        state: &GatewayLocalExecOperationState,
    ) -> Result<(), SandLocalExecError> {
        match state.operation_journal_error.as_ref() {
            Some(error) => Err(SandLocalExecError::new(format!(
                "local-exec operation journal unavailable; shell execution is fail-closed: {error}"
            ))),
            None => Ok(()),
        }
    }

    pub fn ensure_operation_journal_healthy(&self) -> Result<(), SandLocalExecError> {
        self.operation_owner.with_state(|state| Self::ensure_state_journal_healthy(state))
    }

    pub fn stable_result_ref(operation_id: &str) -> String {
        format!("fabushi-local-exec://operation/{operation_id}")
    }

    // Called with the owner's state transaction held, including throughout
    // atomic replacement and its durability barriers.
    fn persist_operation_snapshot(
        state: &mut GatewayLocalExecOperationState,
    ) -> Result<(), SandLocalExecError> {
        let mut records = state.operation_history.clone();
        for (operation_id, active) in &state.active_operations {
            records.push_back(GatewayLocalExecOperationRecord {
                operation_id: operation_id.clone(),
                agent_id: active.agent_id.clone(),
                bridge_request_id: active.bridge_request_id.clone(),
                status: if active.started {
                    GatewayLocalExecOperationStatus::Running
                } else {
                    GatewayLocalExecOperationStatus::Accepted
                },
                exit_code: None,
                aborted: false,
                stdout: String::new(),
                stderr: String::new(),
                stdout_truncated: false,
                stderr_truncated: false,
                settled_at_ms: None,
            });
        }
        #[cfg(test)]
        if let Some(pause) = state.before_journal_publish.as_ref() {
            pause();
        }
        Self::persist_operation_history(&state.operation_journal_path, &records).map_err(|error| {
            let message = format!("local-exec operation journal persistence is fail-closed: {error}");
            state.operation_journal_error = Some(message.clone());
            SandLocalExecError::new(message)
        })
    }

    pub fn operation(&self, operation_id: &str) -> Option<GatewayLocalExecOperationRecord> {
        self.operation_owner.with_state(|state| {
            if let Some(active) = state.active_operations.get(operation_id) {
                return Some(GatewayLocalExecOperationRecord {
                    operation_id: operation_id.to_string(),
                    agent_id: active.agent_id.clone(),
                    bridge_request_id: active.bridge_request_id.clone(),
                    status: if active.started {
                        GatewayLocalExecOperationStatus::Running
                    } else {
                        GatewayLocalExecOperationStatus::Accepted
                    },
                    exit_code: None,
                    aborted: false,
                    stdout: String::new(),
                    stderr: String::new(),
                    stdout_truncated: false,
                    stderr_truncated: false,
                    settled_at_ms: None,
                });
            }
            state.operation_history.iter().rev()
                .find(|record| record.operation_id == operation_id)
                .cloned()
        })
    }

    pub fn operation_for_agent(
        &self,
        agent_id: &str,
        operation_id: &str,
    ) -> Option<GatewayLocalExecOperationRecord> {
        self.operation(operation_id)
            .filter(|record| record.agent_id.as_deref() == Some(agent_id))
    }

    pub fn operation_history(&self) -> Vec<GatewayLocalExecOperationRecord> {
        self.operation_owner.with_state(|state| {
            state.operation_history.iter().cloned().collect()
        })
    }

    pub fn operation_history_for_agent(
        &self,
        agent_id: &str,
    ) -> Vec<GatewayLocalExecOperationRecord> {
        self.operation_history()
            .into_iter()
            .filter(|record| record.agent_id.as_deref() == Some(agent_id))
            .collect()
    }

    fn cancel_matching_operation(
        state: &GatewayLocalExecOperationState,
        operation_id: &str,
        agent_id: Option<&str>,
        bridge_request_id: Option<&str>,
    ) -> bool {
        let Some(operation) = state.active_operations.get(operation_id) else {
            return false;
        };
        if agent_id.is_some_and(|agent| operation.agent_id.as_deref() != Some(agent))
            || bridge_request_id.is_some_and(|request| operation.bridge_request_id != request)
        {
            return false;
        }
        // Check ownership and deliver to this exact record under one lock.
        // A successful send requests cancellation; it is not an Exit.
        operation.cancellation.cancel()
    }

    pub fn cancel_operation(&self, operation_id: &str) -> bool {
        self.operation_owner.with_state(|state| {
            Self::cancel_matching_operation(state, operation_id, None, None)
        })
    }

    pub fn cancel_operation_for_agent(&self, agent_id: &str, operation_id: &str) -> bool {
        self.operation_owner.with_state(|state| {
            Self::cancel_matching_operation(state, operation_id, Some(agent_id), None)
        })
    }

    fn cancel_registered_operation(&self, operation_id: &str, bridge_request_id: &str) -> bool {
        self.operation_owner.with_state(|state| {
            Self::cancel_matching_operation(state, operation_id, None, Some(bridge_request_id))
        })
    }

    fn register_operation(
        &self,
        operation_id: &str,
        agent_id: Option<&str>,
        cancellation: LocalExecCancellationHandle,
    ) -> Result<String, SandLocalExecError> {
        let bridge_request_id = cancellation.request_id().to_string();
        self.operation_owner.with_state(|state| {
            Self::ensure_state_journal_healthy(state)?;
            if state.active_operations.contains_key(operation_id)
                || state.operation_history.iter().any(|record| record.operation_id == operation_id)
            {
                return Err(SandLocalExecError::new(format!(
                    "local-exec operationId already exists and will not be replayed: {operation_id}"
                )));
            }
            let capacity = state.operation_history_limit.max(1);
            let protected_history = state.operation_history.iter()
                .filter(|record| !record.status.is_known_terminal())
                .count();
            if protected_history.saturating_add(state.active_operations.len()) >= capacity {
                return Err(SandLocalExecError::new(format!(
                    "local-exec operation journal capacity {capacity} is occupied by unresolved operations; execution was refused before dispatch"
                )));
            }
            // Reserve one place for this exact invocation before sending its
            // frame. Finishing will move that place from active to history.
            let history_capacity = capacity - state.active_operations.len() - 1;
            let previous_history = (state.operation_history.len() > history_capacity)
                .then(|| state.operation_history.clone());
            Self::prune_known_operation_history(&mut state.operation_history, history_capacity);
            state.active_operations.insert(
                operation_id.to_string(),
                GatewayLocalExecActiveOperation {
                    agent_id: agent_id.map(str::to_string),
                    bridge_request_id: bridge_request_id.clone(),
                    cancellation,
                    started: false,
                },
            );
            if let Err(error) = Self::persist_operation_snapshot(state) {
                // No provider frame has been sent. This transaction still owns
                // the registration it is rolling back.
                state.active_operations.remove(operation_id);
                if let Some(previous_history) = previous_history {
                    state.operation_history = previous_history;
                }
                // This restores only in-memory history. A failed durability
                // barrier may already have replaced the on-disk image.
                return Err(error);
            }
            Ok(bridge_request_id)
        })
    }

    fn mark_operation_started(
        &self,
        operation_id: &str,
        bridge_request_id: &str,
    ) -> Result<(), SandLocalExecError> {
        self.operation_owner.with_state(|state| {
            let operation = state.active_operations.get_mut(operation_id).ok_or_else(|| {
                SandLocalExecError::new(format!(
                    "local-exec operationId disappeared before started journal: {operation_id}"
                ))
            })?;
            if operation.bridge_request_id != bridge_request_id {
                return Err(SandLocalExecError::new(format!(
                    "local-exec invocation does not own operationId: {operation_id}"
                )));
            }
            if operation.started {
                return Ok(());
            }
            operation.started = true;
            Self::persist_operation_snapshot(state)
        })
    }

    fn finish_operation(
        &self,
        operation_id: &str,
        bridge_request_id: &str,
        status: GatewayLocalExecOperationStatus,
        result: Option<&GatewayLocalExecShellResult>,
    ) -> Result<(), SandLocalExecError> {
        self.operation_owner.with_state(|state| {
            let active = state.active_operations.get(operation_id).ok_or_else(|| {
                SandLocalExecError::new(format!(
                    "local-exec operationId disappeared before terminal journal: {operation_id}"
                ))
            })?;
            if active.bridge_request_id != bridge_request_id {
                return Err(SandLocalExecError::new(format!(
                    "local-exec invocation does not own operationId: {operation_id}"
                )));
            }
            let active = state.active_operations.remove(operation_id)
                .expect("the matching registration is held by this state transaction");
            // Admission already reserved this place. Moving active custody
            // into history must never evict another unresolved operation.
            let (stdout, stdout_truncated) = result
                .map(|result| durable_shell_output(&result.stdout, result.stdout_truncated))
                .unwrap_or_default();
            let (stderr, stderr_truncated) = result
                .map(|result| durable_shell_output(&result.stderr, result.stderr_truncated))
                .unwrap_or_default();
            state.operation_history.push_back(GatewayLocalExecOperationRecord {
                operation_id: operation_id.to_string(),
                agent_id: active.agent_id,
                bridge_request_id: active.bridge_request_id,
                status,
                exit_code: result.and_then(|result| result.exit_code),
                aborted: result.is_some_and(|result| result.aborted),
                stdout,
                stderr,
                stdout_truncated,
                stderr_truncated,
                settled_at_ms: Some(Self::now_ms()),
            });
            Self::persist_operation_snapshot(state).map_err(|error| {
                SandLocalExecError::new(format!(
                    "{}; operation result is not safely durable (resultRef={})",
                    error.message,
                    Self::stable_result_ref(operation_id)
                ))
            })
        })
    }

    fn shell_outcome_unknown_error(
        operation_id: &str,
        message: impl AsRef<str>,
    ) -> SandLocalExecError {
        SandLocalExecError::outcome_unknown(format!(
            "{} (operationId={operation_id}; resultRef={})",
            message.as_ref(),
            Self::stable_result_ref(operation_id),
        ))
    }

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
        self.open_exec_with_before_send(scope, action, target, server_message, |_| Ok(()))
    }

    fn open_exec_with_before_send<F>(
        &self,
        scope: &GatewayLocalToolScope,
        action: &str,
        target: &str,
        server_message: Value,
        before_send: F,
    ) -> Result<super::local_exec_bridge::LocalExecRequest, SandLocalExecError>
    where
        F: FnOnce(LocalExecCancellationHandle) -> Result<(), SandLocalExecError>,
    {
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
        self.sandbox.bridge.request_with_before_send(
            frame,
            self.sandbox.computer_id.as_deref(),
            before_send,
        )
    }

    pub fn execute_shell(
        &self,
        scope: &GatewayLocalToolScope,
        command: &str,
        working_directory: &str,
        tool_call_id: &str,
    ) -> Result<GatewayLocalExecShellResult, SandLocalExecError> {
        self.execute_shell_with_timeout(scope, command, working_directory, tool_call_id, None)
    }

    /// Executes a shell operation while preserving the existing operation
    /// journal and provider cancellation owner. A node-host deadline requests
    /// cancellation through the same registered operation; if no terminal
    /// provider frame arrives, the result remains outcome-unknown rather than
    /// claiming that the side effect was undone.
    pub fn execute_shell_with_timeout(
        &self,
        scope: &GatewayLocalToolScope,
        command: &str,
        working_directory: &str,
        tool_call_id: &str,
        timeout: Option<Duration>,
    ) -> Result<GatewayLocalExecShellResult, SandLocalExecError> {
        self.ensure_operation_journal_healthy()?;
        let _process_slot = self
            .process_custody
            .acquire_slot("local-exec")
            .map_err(SandLocalExecError::new)?;
        let started_at = Instant::now();
        let message = PRODUCTION_LOCAL_EXEC_CODEC.shell_stream_server_message(
            self.next_id(),
            command,
            working_directory,
            tool_call_id,
        );
        let mut registered_request_id = None;
        let mut request = match self.open_exec_with_before_send(
            scope,
            "run-command",
            command,
            message,
            |cancellation| {
                let request_id =
                    self.register_operation(tool_call_id, scope.agent_id.as_deref(), cancellation)?;
                registered_request_id = Some(request_id);
                Ok(())
            },
        ) {
            Ok(request) => request,
            Err(error) => {
                // Only this invocation's successful registration can be retired.
                // A permission or duplicate refusal owns no older operation.
                if let Some(request_id) = registered_request_id.as_deref() {
                    if let Err(cleanup_error) = self.finish_operation(
                        tool_call_id,
                        request_id,
                        GatewayLocalExecOperationStatus::Failed,
                        None,
                    ) {
                        return Err(SandLocalExecError::new(format!(
                            "{}; operation cleanup was not safely durable: {}",
                            error.message, cleanup_error.message
                        )));
                    }
                }
                return Err(error);
            }
        };
        let bridge_request_id = request.request_id().to_string();
        let mut result = GatewayLocalExecShellResult {
            request_id: tool_call_id.to_string(),
            ..GatewayLocalExecShellResult::default()
        };
        let mut terminal_status = None;
        let mut observed_activity = false;
        let outcome = (|| -> Result<GatewayLocalExecShellResult, SandLocalExecError> {
            loop {
                let wait = timeout
                    .map(|timeout| timeout.saturating_sub(started_at.elapsed()))
                    .unwrap_or(Duration::from_secs(10))
                    .min(Duration::from_secs(10));
                let response = match request.recv_timeout(wait) {
                    Ok(response) => response,
                    Err(_) => {
                        if timeout.is_some_and(|timeout| started_at.elapsed() >= timeout) {
                            let _ = self.cancel_registered_operation(tool_call_id, &bridge_request_id);
                        }
                        // Neither an accepted cancellation request nor loss of
                        // the response proves that the provider observed Exit.
                        result.outcome_unknown = true;
                        return Ok(result.clone());
                    }
                };
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
                                ProductionShellStreamEvent::Stdout(data) => {
                                    observed_activity = true;
                                    result.stdout_truncated |=
                                        append_bounded_shell_output(&mut result.stdout, data);
                                    self.mark_operation_started(tool_call_id, &bridge_request_id)?;
                                }
                                ProductionShellStreamEvent::Stderr(data) => {
                                    observed_activity = true;
                                    result.stderr_truncated |=
                                        append_bounded_shell_output(&mut result.stderr, data);
                                    self.mark_operation_started(tool_call_id, &bridge_request_id)?;
                                }
                                ProductionShellStreamEvent::Exit { code, cwd, aborted } => {
                                    result.exit_code = Some(*code);
                                    result.cwd = (!cwd.is_empty()).then(|| cwd.clone());
                                    result.aborted = *aborted;
                                    terminal_status = Some(if *aborted {
                                        GatewayLocalExecOperationStatus::Cancelled
                                    } else {
                                        GatewayLocalExecOperationStatus::Completed
                                    });
                                    return Ok(result.clone());
                                }
                                ProductionShellStreamEvent::Rejected(reason)
                                | ProductionShellStreamEvent::PermissionDenied(reason) => {
                                    if !observed_activity {
                                        terminal_status = Some(GatewayLocalExecOperationStatus::Failed);
                                    }
                                    return Err(SandLocalExecError::new(reason.clone()));
                                }
                                ProductionShellStreamEvent::Start
                                | ProductionShellStreamEvent::Backgrounded => {
                                    observed_activity = true;
                                    self.mark_operation_started(tool_call_id, &bridge_request_id)?;
                                }
                                ProductionShellStreamEvent::Other => {}
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
                            GatewayExecControl::Throw { error, stack_trace, error_code } => {
                                self.sandbox.report_exec_failure(scope, &error);
                                if error_code.as_deref() == Some("LOCAL_EXEC_PRE_ENTRY_REFUSED")
                                    && !observed_activity
                                {
                                    terminal_status = Some(GatewayLocalExecOperationStatus::Failed);
                                }
                                let detail = stack_trace
                                    .filter(|stack| !stack.trim().is_empty())
                                    .map(|stack| format!("{error}\n{stack}"))
                                    .unwrap_or(error);
                                return Err(SandLocalExecError::new(format_command_error(
                                    command,
                                    &CommandErrorResult {
                                        code: None,
                                        termination: CommandTermination::Error,
                                        signal: None,
                                        output_limit_exceeded: false,
                                    },
                                    &detail,
                                    "",
                                    0,
                                )));
                            }
                            GatewayExecControl::StreamClose => {
                                result.outcome_unknown = true;
                                return Ok(result.clone());
                            }
                            GatewayExecControl::Unknown => {}
                        }
                    }
                    Some("file-error") => {
                        let error = response
                            .get("error")
                            .and_then(Value::as_str)
                            .unwrap_or(SAND_NO_LOCAL_MACHINE_MESSAGE);
                        return Err(SandLocalExecError::new(format_command_error(
                            command,
                            &CommandErrorResult {
                                code: None,
                                termination: CommandTermination::Error,
                                signal: None,
                                output_limit_exceeded: false,
                            },
                            error,
                            "",
                            0,
                        )));
                    }
                    _ => {}
                }
            }
        })();
        request.close();

        // Once the bridge accepted the frame, an unproved terminal is unknown.
        // Positive pre-entry refusals and authoritative Exit are the exceptions.
        let status = terminal_status.unwrap_or(GatewayLocalExecOperationStatus::OutcomeUnknown);
        let outcome = if status == GatewayLocalExecOperationStatus::OutcomeUnknown {
            result.outcome_unknown = true;
            match outcome {
                Ok(_) => Ok(result.clone()),
                Err(error) => Err(Self::shell_outcome_unknown_error(tool_call_id, &error.message)),
            }
        } else {
            outcome
        };
        if let Err(error) = self.finish_operation(
            tool_call_id,
            &bridge_request_id,
            status,
            Some(&result),
        ) {
            let detail = match outcome {
                Err(cause) => format!("{}; {}", cause.message, error.message),
                Ok(_) => error.message,
            };
            // A positive no-entry refusal remains a known no-effect failure,
            // even when recording that refusal fails. The journal stays poisoned.
            return Err(if status == GatewayLocalExecOperationStatus::Failed {
                SandLocalExecError::new(detail)
            } else {
                Self::shell_outcome_unknown_error(tool_call_id, detail)
            });
        }
        outcome
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
            self.next_id(),
            path,
            tool_call_id,
            offset,
            limit,
            encoding_hint,
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
                        GatewayExecControl::Throw { error, stack_trace, .. } => {
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
                    let error = response
                        .get("error")
                        .and_then(Value::as_str)
                        .unwrap_or(SAND_NO_LOCAL_MACHINE_MESSAGE)
                        .to_string();
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
    pub fn new(bridge: SandLocalExecBridge, gate: Arc<dyn GatewayLocalToolGate>) -> Self {
        Self { bridge, gate }
    }

    pub fn list(&self) -> Vec<LocalExecComputer> {
        self.bridge.list_computers()
    }

    pub fn resolve(
        &self,
        id: Option<&str>,
    ) -> Option<(LocalExecComputer, GatewayLocalExecSandBox)> {
        let selected = match id {
            Some(id) => self
                .bridge
                .list_computers()
                .into_iter()
                .find(|computer| computer.id == id),
            None => self.bridge.active_computer(),
        }?;
        let box_owner = GatewayLocalExecSandBox::new(self.bridge.clone(), Arc::clone(&self.gate))
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

#[cfg(test)]
mod operation_journal_fail_closed_tests {
    use super::*;
    use std::sync::mpsc;

    struct AllowGate;
    impl GatewayLocalToolGate for AllowGate {
        fn blocked_reason(&self) -> Option<String> {
            None
        }
        fn requires_approval(&self) -> bool {
            false
        }
        fn authorize(
            &self,
            _scope: &GatewayLocalToolScope,
            _action: &str,
            _target: &str,
        ) -> Result<Option<String>, SandLocalExecError> {
            Ok(None)
        }
    }

    fn unique_test_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "fabushi-local-exec-{label}-{}-{}.json",
            std::process::id(),
            LOCAL_EXEC_OPERATION_JOURNAL_TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn test_record(
        operation_id: &str,
        status: GatewayLocalExecOperationStatus,
    ) -> GatewayLocalExecOperationRecord {
        GatewayLocalExecOperationRecord {
            operation_id: operation_id.to_string(),
            agent_id: Some("agent-test".into()),
            bridge_request_id: "bridge-test".into(),
            status,
            exit_code: None,
            aborted: false,
            stdout: String::new(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            settled_at_ms: None,
        }
    }

    fn unloaded_operation_state(
        path: PathBuf,
        limit: usize,
    ) -> GatewayLocalExecOperationState {
        GatewayLocalExecOperationState {
            active_operations: HashMap::new(),
            operation_history: VecDeque::new(),
            operation_history_limit: limit,
            operation_journal_path: path,
            operation_journal_error: None,
            before_journal_publish: None,
        }
    }

    fn load_test_operation_history(
        path: &Path,
        limit: usize,
    ) -> Result<VecDeque<GatewayLocalExecOperationRecord>, SandLocalExecError> {
        let mut state = unloaded_operation_state(path.to_path_buf(), limit);
        GatewayLocalExecManager::load_operation_history(&mut state)?;
        Ok(state.operation_history)
    }

    const OPERATION_TEST_BOUND: Duration = Duration::from_secs(5);

    struct OperationTestDirectory(PathBuf);

    impl OperationTestDirectory {
        fn new(label: &str) -> Self {
            let path = unique_test_path(label).with_extension("dir");
            fs::create_dir(&path).expect("create owned operation fixture directory");
            Self(path)
        }

        fn journal(&self) -> PathBuf {
            self.0.join("operations.json")
        }

        fn refuse_publication(&self) {
            let journal = self.journal();
            fs::rename(&journal, self.0.join("before-fault.json"))
                .expect("retain the last real journal before the I/O fault");
            fs::create_dir(&journal).expect("a directory cannot be replaced by the journal file");
        }
    }

    impl Drop for OperationTestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct DeniedGate;

    impl GatewayLocalToolGate for DeniedGate {
        fn blocked_reason(&self) -> Option<String> {
            Some("fixture permission refusal".into())
        }

        fn requires_approval(&self) -> bool {
            true
        }

        fn authorize(
            &self,
            _scope: &GatewayLocalToolScope,
            _action: &str,
            _target: &str,
        ) -> Result<Option<String>, SandLocalExecError> {
            Err(SandLocalExecError::new("fixture permission refusal"))
        }
    }

    struct ShellOperationFixture {
        directory: OperationTestDirectory,
        owner: Arc<GatewayLocalExecOperationOwner>,
        bridge: SandLocalExecBridge,
        _registration: super::super::local_exec_bridge::LocalExecProviderRegistration,
        provider_id: String,
        receive: mpsc::Receiver<Value>,
        manager: Arc<GatewayLocalExecManager>,
    }

    impl ShellOperationFixture {
        fn new(label: &str) -> Self {
            let directory = OperationTestDirectory::new(label);
            let owner = Arc::new(GatewayLocalExecOperationOwner::with_journal_path(directory.journal()));
            let sequence = Arc::new(AtomicU64::new(1));
            let next = Arc::clone(&sequence);
            let bridge = SandLocalExecBridge::with_sources(
                Arc::new(|| 1000),
                Arc::new(move || format!("lxo-request-{}", next.fetch_add(1, Ordering::Relaxed))),
            );
            let (send, receive) = mpsc::channel();
            let registration = bridge.register_provider(send);
            let welcome = receive.recv_timeout(OPERATION_TEST_BOUND).expect("provider welcome");
            let provider_id = welcome["providerId"].as_str().expect("provider identity").to_string();
            bridge.submit_responses(json!({
                "providerId": provider_id,
                "frames": [{
                    "kind": "hello",
                    "localRoot": directory.0.to_string_lossy(),
                    "terminalsFolder": "terminals",
                    "computerId": "lxo-computer",
                    "label": "LXO fixture"
                }]
            }));
            let manager = Arc::new(
                GatewayLocalExecSandBox::new(bridge.clone(), Arc::new(AllowGate))
                    .with_operation_owner(Arc::clone(&owner))
                    .exec_manager(),
            );
            Self { directory, owner, bridge, _registration: registration, provider_id, receive, manager }
        }

        fn facade(&self, gate: Arc<dyn GatewayLocalToolGate>) -> Arc<GatewayLocalExecManager> {
            Arc::new(
                GatewayLocalExecSandBox::new(self.bridge.clone(), gate)
                    .with_operation_owner(Arc::clone(&self.owner))
                    .exec_manager(),
            )
        }

        fn start(
            &self,
            operation_id: &str,
            timeout: Option<Duration>,
        ) -> std::thread::JoinHandle<Result<GatewayLocalExecShellResult, SandLocalExecError>> {
            let manager = Arc::clone(&self.manager);
            let operation_id = operation_id.to_string();
            std::thread::spawn(move || {
                manager.execute_shell_with_timeout(
                    &GatewayLocalToolScope {
                        agent_id: Some("lxo-agent".into()),
                        tool_call_id: Some(operation_id.clone()),
                        action: Some("run-command".into()),
                    },
                    "fixture-command",
                    "/tmp",
                    &operation_id,
                    timeout,
                )
            })
        }

        fn frame(&self, kind: &str) -> Value {
            let frame = self.receive.recv_timeout(OPERATION_TEST_BOUND).expect("provider frame");
            assert_eq!(frame["kind"], kind);
            frame
        }

        fn reply(&self, request: &Value, mut frames: Vec<Value>) {
            for frame in &mut frames {
                frame["requestId"] = request["requestId"].clone();
            }
            self.bridge.submit_responses(json!({
                "providerId": self.provider_id,
                "frames": frames,
            }));
        }

        fn wait_running(&self, operation_id: &str) {
            let deadline = Instant::now() + OPERATION_TEST_BOUND;
            loop {
                if self.manager.operation(operation_id).is_some_and(|record| {
                    record.status == GatewayLocalExecOperationStatus::Running
                }) {
                    return;
                }
                assert!(Instant::now() < deadline, "operation did not enter Running");
                std::thread::yield_now();
            }
        }

        fn record(&self, operation_id: &str) -> GatewayLocalExecOperationRecord {
            self.manager.operation_for_agent("lxo-agent", operation_id).expect("owned operation")
        }
    }

    fn test_stdout(data: &str) -> Value {
        json!({"kind":"client","message":{"id":1,"shellStream":{"stdout":{"data":data}}}})
    }

    fn test_exit(code: u32, aborted: bool) -> Value {
        json!({"kind":"client","message":{"id":1,"shellStream":{"exit":{
            "code":code,"cwd":"/tmp","aborted":aborted
        }}}})
    }

    fn test_close() -> Value {
        json!({"kind":"control","message":{"streamClose":{"id":1}}})
    }

    fn test_stderr(data: &str) -> Value {
        json!({"kind":"client","message":{"id":1,"shellStream":{"stderr":{"data":data}}}})
    }

    fn read_test_operation_journal(path: &Path) -> GatewayLocalExecOperationJournal {
        serde_json::from_slice(&fs::read(path).expect("read actual operation journal"))
            .expect("decode actual operation journal")
    }

    fn write_test_operation_journal(
        path: &Path,
        version: u32,
        records: &[GatewayLocalExecOperationRecord],
    ) -> Vec<u8> {
        let bytes = serde_json::to_vec(&GatewayLocalExecOperationJournal {
            version,
            records: records.to_vec(),
        }).expect("encode fixture journal");
        fs::write(path, &bytes).expect("write fixture journal");
        bytes
    }

    #[test]
    fn operation_owner_is_lazy_and_shared_by_distinct_permission_facades() {
        let fixture = ShellOperationFixture::new("lazy-shared");
        let second = fixture.facade(Arc::new(AllowGate));
        assert!(Arc::ptr_eq(&fixture.manager.operation_owner, &second.operation_owner));
        assert!(fixture.owner.state.lock().expect("uninitialized owner").is_none());
        assert!(!fixture.directory.journal().exists(), "facade construction must not publish a journal");

        assert!(second.operation("absent").is_none());
        {
            let state = fixture.owner.state.lock().expect("initialized owner");
            assert_eq!(state.as_ref().expect("lazy initialization").operation_journal_path,
                fixture.directory.journal());
        }
        let shell = fixture.start("shared-live", None);
        let request = fixture.frame("exec");
        let active = second.operation_for_agent("lxo-agent", "shared-live").expect("shared admission");
        assert_eq!(active.bridge_request_id, request["requestId"].as_str().unwrap());
        assert_eq!(active.status, GatewayLocalExecOperationStatus::Accepted);
        fixture.reply(&request, vec![test_exit(0, false)]);
        let result = shell.join().expect("shell worker").expect("confirmed result");
        assert!(!result.outcome_unknown);
        assert_eq!(second.operation("shared-live").unwrap().status, GatewayLocalExecOperationStatus::Completed);
        assert_eq!(second.operation_history(), fixture.manager.operation_history());
    }

    #[test]
    fn duplicate_and_permission_refusal_cannot_retire_another_invocation() {
        let fixture = ShellOperationFixture::new("duplicate-identity");
        let shell = fixture.start("live-identity", None);
        let request = fixture.frame("exec");
        fixture.reply(&request, vec![test_stdout("owned output")]);
        fixture.wait_running("live-identity");
        let original = fixture.record("live-identity");
        let scope = GatewayLocalToolScope {
            agent_id: Some("other-agent".into()),
            tool_call_id: Some("live-identity".into()),
            action: Some("run-command".into()),
        };
        let second = fixture.facade(Arc::new(AllowGate));
        let duplicate = second.execute_shell(&scope, "must-not-run", "/tmp", "live-identity")
            .expect_err("duplicate is refused before publication");
        assert!(!duplicate.is_outcome_unknown());
        assert!(duplicate.message.contains("already exists"));
        let denied = fixture.facade(Arc::new(DeniedGate))
            .execute_shell(&scope, "must-not-run", "/tmp", "live-identity")
            .expect_err("permission refusal owns no registration");
        assert!(!denied.is_outcome_unknown());
        assert_eq!(fixture.record("live-identity"), original);
        assert!(fixture.manager.operation_history().is_empty());
        assert!(!second.cancel_operation_for_agent("other-agent", "live-identity"));
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));

        assert!(fixture.manager.mark_operation_started("live-identity", "stale-request").is_err());
        assert!(fixture.manager.finish_operation(
            "live-identity", "stale-request", GatewayLocalExecOperationStatus::Failed, None,
        ).is_err());
        assert_eq!(fixture.record("live-identity"), original, "a stale ticket cannot mutate the real call");

        assert!(second.cancel_operation_for_agent("lxo-agent", "live-identity"));
        let cancel = fixture.frame("cancel");
        assert_eq!(cancel["requestId"], request["requestId"]);
        fixture.reply(&request, vec![test_exit(17, false)]);
        let result = shell.join().expect("shell worker").expect("real Exit after a cancel request");
        assert_eq!(result.exit_code, Some(17));
        assert!(!result.aborted);
        assert!(!result.outcome_unknown);
        let record = fixture.record("live-identity");
        assert_eq!(record.status, GatewayLocalExecOperationStatus::Completed);
        assert_eq!(record.exit_code, Some(17));
        assert_eq!(record.bridge_request_id, original.bridge_request_id);
        assert_eq!(fixture.manager.operation_history().len(), 1);
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn unproved_shell_terminals_preserve_typed_uncertainty_and_observed_output() {
        let cases = [
            ("close", test_close(), true, true),
            ("missing-client", json!({"kind":"client"}), false, true),
            ("malformed-client", json!({"kind":"client","message":{"id":{"invalid":true}}}), false, true),
            ("missing-control", json!({"kind":"control"}), false, true),
            ("malformed-control", json!({"kind":"control","message":{"throw":{"error":[]}}}), false, true),
            ("opaque-throw", json!({"kind":"control","message":{"throw":{"error":"opaque failure"}}}), false, false),
            ("backend-error", json!({"kind":"control","message":{"throw":{
                "error":"unavailable","errorCode":"EXEC_BACKEND_UNAVAILABLE"
            }}}), false, false),
            ("stream-start-error", json!({"kind":"control","message":{"throw":{
                "error":"timed out","errorCode":"AGENT_EXEC_STREAM_START_TIMEOUT"
            }}}), false, false),
            ("entered-rejected", json!({"kind":"client","message":{"shellStream":{"rejected":{"reason":"late rejection"}}}}), false, true),
            ("entered-denied", json!({"kind":"client","message":{"shellStream":{"permissionDenied":{"error":"late denial"}}}}), false, true),
            ("entered-refusal-tag", json!({"kind":"control","message":{"throw":{
                "error":"late tag","errorCode":"LOCAL_EXEC_PRE_ENTRY_REFUSED"
            }}}), false, true),
            ("unexpected-file-error", json!({"kind":"file-error","error":"unexpected file failure"}), false, true),
        ];
        for (label, terminal, returns_result, activity) in cases {
            let fixture = ShellOperationFixture::new(label);
            let shell = fixture.start(label, None);
            let request = fixture.frame("exec");
            let mut frames = Vec::new();
            if activity {
                frames.push(test_stdout("bounded partial output"));
            }
            frames.push(terminal);
            fixture.reply(&request, frames);
            let outcome = shell.join().expect("uncertain shell worker");
            if returns_result {
                let result = outcome.expect("stream closure returns an explicit unknown snapshot");
                assert!(result.outcome_unknown, "{label}");
                assert_eq!(result.request_id, label);
                assert_eq!(result.exit_code, None);
                assert!(!result.aborted);
            } else {
                let error = outcome.expect_err("unproved error must remain uncertain");
                assert!(error.is_outcome_unknown(), "{label}: {}", error.message);
                assert!(error.message.contains(&GatewayLocalExecManager::stable_result_ref(label)), "{label}");
            }
            let record = fixture.record(label);
            assert_eq!(record.status, GatewayLocalExecOperationStatus::OutcomeUnknown, "{label}");
            assert_eq!(record.exit_code, None, "{label}");
            assert!(!record.aborted, "{label}");
            assert_eq!(record.stdout, if activity { "bounded partial output" } else { "" }, "{label}");
            assert_eq!(record.bridge_request_id, request["requestId"].as_str().unwrap());
            let persisted = load_test_operation_history(&fixture.directory.journal(), 128)
                .expect("actual durable unknown record");
            assert_eq!(persisted.front(), Some(&record), "{label}");
        }
    }

    #[test]
    fn preentry_refusals_remain_known_and_do_not_poison_journal_health() {
        for (label, refusal) in [
            ("rejected", json!({"kind":"client","message":{"shellStream":{"rejected":{"reason":"not admitted"}}}})),
            ("permission-denied", json!({"kind":"client","message":{"shellStream":{"permissionDenied":{"error":"not allowed"}}}})),
            ("tagged-refusal", json!({"kind":"control","message":{"throw":{
                "error":"producer refused before entry","errorCode":"LOCAL_EXEC_PRE_ENTRY_REFUSED"
            }}})),
        ] {
            let fixture = ShellOperationFixture::new(label);
            let shell = fixture.start(label, None);
            let request = fixture.frame("exec");
            fixture.reply(&request, vec![refusal]);
            let error = shell.join().expect("refused shell worker").expect_err("known refusal");
            assert!(!error.is_outcome_unknown(), "{label}");
            let record = fixture.record(label);
            assert_eq!(record.status, GatewayLocalExecOperationStatus::Failed);
            assert!(record.stdout.is_empty());
            fixture.manager.ensure_operation_journal_healthy().expect("known refusal is durably recorded");
            let cancel = fixture.frame("cancel");
            assert_eq!(cancel["requestId"], request["requestId"]);

            let next = fixture.start("confirmed-after-refusal", None);
            let next_request = fixture.frame("exec");
            assert_ne!(next_request["requestId"], request["requestId"]);
            fixture.reply(&next_request, vec![test_exit(0, false)]);
            assert!(!next.join().expect("next worker").expect("later authorized operation").outcome_unknown);
        }
    }

    #[test]
    fn cancellation_close_and_deadline_remain_unknown_after_durable_reopen() {
        for requested_cancel in [false, true] {
            let fixture = ShellOperationFixture::new("cancel-or-deadline");
            let shell = fixture.start(
                "uncertain-cancel",
                (!requested_cancel).then_some(Duration::from_millis(30)),
            );
            let request = fixture.frame("exec");
            if requested_cancel {
                fixture.reply(&request, vec![test_stdout("before cancellation")]);
                fixture.wait_running("uncertain-cancel");
                assert!(fixture.manager.cancel_operation_for_agent("lxo-agent", "uncertain-cancel"));
                fixture.reply(&request, vec![test_close()]);
            }
            let result = shell.join().expect("cancel/deadline worker").expect("unknown result snapshot");
            assert!(result.outcome_unknown);
            assert_eq!(result.exit_code, None);
            assert!(!result.aborted, "a requested cancellation is not an observed abort");
            let cancel = fixture.frame("cancel");
            assert_eq!(cancel["requestId"], request["requestId"]);
            let record = fixture.record("uncertain-cancel");
            assert_eq!(record.status, GatewayLocalExecOperationStatus::OutcomeUnknown);
            let reopened = GatewayLocalExecManager::new_with_operation_journal_path(
                GatewayLocalExecSandBox::new(fixture.bridge.clone(), Arc::new(AllowGate)),
                fixture.directory.journal(),
            );
            assert_eq!(reopened.operation("uncertain-cancel"), Some(record));
            let duplicate = reopened.execute_shell(
                &GatewayLocalToolScope {
                    agent_id: Some("lxo-agent".into()),
                    tool_call_id: Some("uncertain-cancel".into()),
                    action: None,
                },
                "do-not-replay", "/tmp", "uncertain-cancel",
            ).expect_err("reopening uncertainty cannot authorize replay");
            assert!(!duplicate.is_outcome_unknown());
            assert!(duplicate.message.contains("already exists"));
            assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }
    }

    #[test]
    fn uncertain_shell_output_remains_utf8_bounded_in_durable_history() {
        let fixture = ShellOperationFixture::new("unknown-output-bound");
        let shell = fixture.start("unknown-output-bound", None);
        let request = fixture.frame("exec");
        let output = "界".repeat(MAX_LOCAL_EXEC_SHELL_STREAM_BYTES / 3 + 100);
        fixture.reply(&request, vec![test_stdout(&output), test_close()]);
        let result = shell.join().expect("bounded worker").expect("unknown snapshot");
        assert!(result.outcome_unknown);
        assert!(result.stdout_truncated);
        assert!(result.stdout.len() <= MAX_LOCAL_EXEC_SHELL_STREAM_BYTES);
        assert!(output.starts_with(&result.stdout));
        let record = fixture.record("unknown-output-bound");
        assert_eq!(record.status, GatewayLocalExecOperationStatus::OutcomeUnknown);
        assert!(record.stdout_truncated);
        assert!(record.stdout.len() <= MAX_LOCAL_EXEC_DURABLE_OUTPUT_BYTES);
        assert!(output.starts_with(&record.stdout));
        let recovered = load_test_operation_history(&fixture.directory.journal(), 128)
            .expect("UTF-8 durable unknown output");
        assert_eq!(recovered.front(), Some(&record));
    }

    #[test]
    fn post_send_journal_failures_keep_observed_facts_and_unknown_custody() {
        for terminal_write in [false, true] {
            let fixture = ShellOperationFixture::new("journal-after-send");
            let shell = fixture.start("journal-after-send", None);
            let request = fixture.frame("exec");
            if terminal_write {
                fixture.reply(&request, vec![test_stdout("retained before Exit")]);
                fixture.wait_running("journal-after-send");
            }
            fixture.directory.refuse_publication();
            fixture.reply(&request, vec![if terminal_write {
                test_exit(17, false)
            } else {
                test_stdout("retained despite started-journal failure")
            }]);
            let error = shell.join().expect("journal-failure worker").expect_err("real journal I/O failure");
            assert!(error.is_outcome_unknown());
            assert!(error.message.contains("persistence"));
            assert!(error.message.contains("resultRef="));
            let record = fixture.record("journal-after-send");
            assert_eq!(record.status, if terminal_write {
                GatewayLocalExecOperationStatus::Completed
            } else {
                GatewayLocalExecOperationStatus::OutcomeUnknown
            });
            assert_eq!(record.exit_code, terminal_write.then_some(17));
            assert!(!record.aborted);
            assert_eq!(record.stdout, if terminal_write {
                "retained before Exit"
            } else {
                "retained despite started-journal failure"
            });
            let health = fixture.manager.ensure_operation_journal_healthy().expect_err("shared journal is poisoned");
            assert!(!health.is_outcome_unknown(), "a later admission failure entered no effect");
            let other = fixture.facade(Arc::new(AllowGate));
            assert!(other.ensure_operation_journal_healthy().is_err(), "health is shared across facades");

            fs::remove_dir(fixture.directory.journal()).expect("remove only the owned fault directory");
            fs::rename(fixture.directory.0.join("before-fault.json"), fixture.directory.journal())
                .expect("restore the last actually durable state");
            let reopened = GatewayLocalExecManager::new_with_operation_journal_path(
                GatewayLocalExecSandBox::new(fixture.bridge.clone(), Arc::new(AllowGate)),
                fixture.directory.journal(),
            );
            assert_eq!(reopened.operation("journal-after-send").unwrap().status,
                GatewayLocalExecOperationStatus::OutcomeUnknown,
                "an in-memory Exit cannot upgrade a prior durable Accepted/Running record");
        }
    }

    #[test]
    fn proven_preentry_refusal_stays_known_when_its_journal_publication_fails() {
        let fixture = ShellOperationFixture::new("refusal-journal-failure");
        let shell = fixture.start("refusal-journal-failure", None);
        let request = fixture.frame("exec");
        fixture.directory.refuse_publication();
        fixture.reply(&request, vec![json!({"kind":"control","message":{"throw":{
            "error":"positively refused before execution","errorCode":"LOCAL_EXEC_PRE_ENTRY_REFUSED"
        }}})]);
        let error = shell.join().expect("refusal worker").expect_err("known refusal and I/O failure");
        assert!(!error.is_outcome_unknown());
        assert!(error.message.contains("positively refused"));
        assert!(error.message.contains("persistence"));
        assert_eq!(fixture.record("refusal-journal-failure").status, GatewayLocalExecOperationStatus::Failed);
        assert!(fixture.manager.ensure_operation_journal_healthy().is_err());
    }

    #[test]
    fn finish_admission_and_snapshot_publication_share_one_transaction() {
        let fixture = ShellOperationFixture::new("publication-transaction");
        let shell = fixture.start("transaction-id", None);
        let request = fixture.frame("exec");
        fixture.reply(&request, vec![test_stdout("transaction output")]);
        fixture.wait_running("transaction-id");

        let (paused_send, paused_receive) = mpsc::channel();
        let (resume_send, resume_receive) = mpsc::channel();
        let resume_receive = Arc::new(Mutex::new(resume_receive));
        let pause_once = Arc::new(std::sync::atomic::AtomicBool::new(true));
        fixture.owner.with_state(|state| {
            state.before_journal_publish = Some(Arc::new(move || {
                if pause_once.swap(false, Ordering::AcqRel) {
                    paused_send.send(()).expect("announce real terminal publication");
                    resume_receive.lock().expect("resume gate")
                        .recv_timeout(OPERATION_TEST_BOUND).expect("bounded publication release");
                }
            }));
        });
        fixture.reply(&request, vec![test_exit(0, false)]);
        paused_receive.recv_timeout(OPERATION_TEST_BOUND).expect("terminal reached actual publication");
        assert!(matches!(fixture.owner.state.try_lock(), Err(std::sync::TryLockError::WouldBlock)),
            "the coherent state transaction must cover publication, not just snapshot construction");

        let second = fixture.facade(Arc::new(AllowGate));
        let (contending_send, contending_receive) = mpsc::channel();
        let contender = std::thread::spawn(move || {
            assert!(matches!(second.operation_owner.state.try_lock(),
                Err(std::sync::TryLockError::WouldBlock)));
            contending_send.send(()).expect("announce admission during publication");
            second.execute_shell(
                &GatewayLocalToolScope {
                    agent_id: Some("lxo-agent".into()),
                    tool_call_id: Some("transaction-id".into()),
                    action: None,
                },
                "must-not-reenter", "/tmp", "transaction-id",
            )
        });
        contending_receive.recv_timeout(OPERATION_TEST_BOUND)
            .expect("second facade reached the same publishing owner");
        resume_send.send(()).expect("allow the real atomic journal write");
        assert!(!shell.join().expect("finishing worker").expect("confirmed Exit").outcome_unknown);
        let error = contender.join().expect("duplicate contender").expect_err("terminal identity remains reserved");
        assert!(!error.is_outcome_unknown());
        assert!(error.message.contains("already exists"));
        let record = fixture.record("transaction-id");
        assert_eq!(record.status, GatewayLocalExecOperationStatus::Completed);
        assert_eq!(record.stdout, "transaction output");
        let persisted = load_test_operation_history(&fixture.directory.journal(), 128)
            .expect("real ordered journal publication");
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted.front(), Some(&record));
        let cancel = fixture.frame("cancel");
        assert_eq!(cancel["requestId"], request["requestId"]);
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "a terminal/admission handoff cannot dispatch a second invocation");
    }

    #[test]
    fn unknown_custody_survives_known_terminal_churn_and_reopen() {
        let fixture = ShellOperationFixture::new("retained-unknown-churn");
        fixture.owner.with_state(|state| state.operation_history_limit = 3);
        let unknown = fixture.start("retained-unknown", None);
        let unknown_request = fixture.frame("exec");
        fixture.reply(&unknown_request, vec![
            test_stdout("unresolved stdout 界"),
            test_stderr("unresolved stderr"),
            test_close(),
        ]);
        assert!(unknown.join().expect("unknown worker").expect("unknown result").outcome_unknown);
        assert_eq!(fixture.frame("cancel")["requestId"], unknown_request["requestId"]);
        let retained = fixture.record("retained-unknown");

        for index in 0usize..9 {
            let operation_id = format!("known-churn-{index}");
            let shell = fixture.start(&operation_id, None);
            let request = fixture.frame("exec");
            assert_ne!(request["requestId"], unknown_request["requestId"]);
            let admitted = read_test_operation_journal(&fixture.directory.journal());
            assert_eq!(admitted.version, 3);
            assert!(admitted.records.len() <= 3, "active registrations also consume capacity");
            assert!(admitted.records.contains(&retained));
            let status = match index % 3 {
                0 => GatewayLocalExecOperationStatus::Completed,
                1 => GatewayLocalExecOperationStatus::Cancelled,
                _ => GatewayLocalExecOperationStatus::Failed,
            };
            let terminal = if status == GatewayLocalExecOperationStatus::Failed {
                json!({"kind":"control","message":{"throw":{
                    "error":"known refusal","errorCode":"LOCAL_EXEC_PRE_ENTRY_REFUSED"
                }}})
            } else {
                test_exit(17, status == GatewayLocalExecOperationStatus::Cancelled)
            };
            fixture.reply(&request, vec![terminal]);
            let outcome = shell.join().expect("known terminal worker");
            if status == GatewayLocalExecOperationStatus::Failed {
                assert!(!outcome.expect_err("known refusal").is_outcome_unknown());
            } else {
                let result = outcome.expect("observed Exit");
                assert!(!result.outcome_unknown);
                assert_eq!(result.exit_code, Some(17));
                assert_eq!(result.aborted, status == GatewayLocalExecOperationStatus::Cancelled);
            }
            assert_eq!(fixture.frame("cancel")["requestId"], request["requestId"]);
            assert_eq!(fixture.record(&operation_id).status, status);
            assert_eq!(fixture.record("retained-unknown"), retained);
            let history = fixture.manager.operation_history();
            assert_eq!(history.len(), (index + 2).min(3));
            assert_eq!(history.first(), Some(&retained));
            if index >= 2 {
                assert!(fixture.manager.operation(&format!("known-churn-{}", index - 2)).is_none(),
                    "the oldest eligible known record is evicted");
            }
        }

        let before_refusal = fs::read(fixture.directory.journal()).expect("retained journal");
        let duplicate = fixture.manager.execute_shell(
            &GatewayLocalToolScope::default(),
            "must-not-replay", "/tmp", "retained-unknown",
        ).expect_err("churn cannot forget unresolved identity");
        assert!(!duplicate.is_outcome_unknown());
        assert!(duplicate.message.contains("already exists"));
        assert_eq!(fs::read(fixture.directory.journal()).unwrap(), before_refusal);
        assert_eq!(fixture.record("retained-unknown"), retained);
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));

        let reopened = GatewayLocalExecManager::new_with_operation_journal_path(
            GatewayLocalExecSandBox::new(fixture.bridge.clone(), Arc::new(AllowGate)),
            fixture.directory.journal(),
        );
        assert_eq!(reopened.operation_for_agent("lxo-agent", "retained-unknown"), Some(retained));
        assert_eq!(reopened.operation_history(), fixture.manager.operation_history());
        let duplicate = reopened.execute_shell(
            &GatewayLocalToolScope::default(),
            "must-not-replay-after-restart", "/tmp", "retained-unknown",
        ).expect_err("recovery preserves the duplicate fence");
        assert!(!duplicate.is_outcome_unknown());
        assert!(duplicate.message.contains("already exists"));
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn protected_history_and_active_capacity_refuse_before_dispatch_then_known_exit_releases_room() {
        let fixture = ShellOperationFixture::new("protected-active-capacity");
        fixture.owner.with_state(|state| state.operation_history_limit = 2);
        let unknown = fixture.start("protected-history", None);
        let unknown_request = fixture.frame("exec");
        fixture.reply(&unknown_request, vec![test_stdout("keep this output"), test_close()]);
        assert!(unknown.join().expect("unknown worker").expect("unknown result").outcome_unknown);
        assert_eq!(fixture.frame("cancel")["requestId"], unknown_request["requestId"]);
        let protected = fixture.record("protected-history");

        let active = fixture.start("reserved-active", None);
        let active_request = fixture.frame("exec");
        let second = fixture.facade(Arc::new(AllowGate));
        for (index, expected_status) in [
            GatewayLocalExecOperationStatus::Accepted,
            GatewayLocalExecOperationStatus::Running,
        ].into_iter().enumerate() {
            if expected_status == GatewayLocalExecOperationStatus::Running {
                fixture.reply(&active_request, vec![test_stdout("live output")]);
                fixture.wait_running("reserved-active");
            }
            let active_before = fixture.record("reserved-active");
            assert_eq!(active_before.status, expected_status);
            let bytes_before = fs::read(fixture.directory.journal()).expect("full journal");
            let operation_id = format!("capacity-refused-{index}");
            let error = second.execute_shell(
                &GatewayLocalToolScope::default(),
                "must-not-enter", "/tmp", &operation_id,
            ).expect_err("protected plus active consumes all capacity");
            assert!(!error.is_outcome_unknown());
            assert!(error.message.contains("capacity"));
            assert_eq!(fixture.record("protected-history"), protected);
            assert_eq!(fixture.record("reserved-active"), active_before);
            assert!(second.operation(&operation_id).is_none());
            assert_eq!(fs::read(fixture.directory.journal()).unwrap(), bytes_before);
            assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }

        fixture.reply(&active_request, vec![test_exit(0, false)]);
        assert!(!active.join().expect("active worker").expect("known Exit").outcome_unknown);
        assert_eq!(fixture.frame("cancel")["requestId"], active_request["requestId"]);
        assert_eq!(fixture.record("reserved-active").status, GatewayLocalExecOperationStatus::Completed);

        let next = fixture.start("after-known-exit", None);
        let next_request = fixture.frame("exec");
        assert_eq!(fixture.record("protected-history"), protected);
        assert!(second.operation("reserved-active").is_none(), "known history supplies the next slot");
        let admitted = read_test_operation_journal(&fixture.directory.journal());
        assert_eq!(admitted.records.len(), 2);
        assert!(admitted.records.contains(&protected));
        fixture.reply(&next_request, vec![test_exit(130, true)]);
        let result = next.join().expect("next worker").expect("observed aborted Exit");
        assert!(result.aborted);
        assert!(!result.outcome_unknown);
        assert_eq!(fixture.frame("cancel")["requestId"], next_request["requestId"]);
        assert_eq!(fixture.record("after-known-exit").status, GatewayLocalExecOperationStatus::Cancelled);
        assert_eq!(fixture.record("protected-history"), protected);
        assert_eq!(fixture.manager.operation_history().len(), 2);
    }

    #[test]
    fn concurrent_facades_reserve_capacity_inside_durable_admission() {
        let fixture = ShellOperationFixture::new("concurrent-capacity");
        let (paused_send, paused_receive) = mpsc::channel();
        let (resume_send, resume_receive) = mpsc::channel();
        let resume_receive = Arc::new(Mutex::new(resume_receive));
        let pause_once = Arc::new(std::sync::atomic::AtomicBool::new(true));
        fixture.owner.with_state(|state| {
            state.operation_history_limit = 1;
            state.before_journal_publish = Some(Arc::new(move || {
                if pause_once.swap(false, Ordering::AcqRel) {
                    paused_send.send(()).expect("admission reached durable publication");
                    resume_receive.lock().expect("admission release gate")
                        .recv_timeout(OPERATION_TEST_BOUND).expect("bounded admission release");
                }
            }));
        });

        let start = Arc::new(std::sync::Barrier::new(3));
        let (result_send, result_receive) = mpsc::channel();
        let mut workers = Vec::new();
        for (operation_id, manager) in [
            ("concurrent-left", fixture.facade(Arc::new(AllowGate))),
            ("concurrent-right", fixture.facade(Arc::new(AllowGate))),
        ] {
            let start = Arc::clone(&start);
            let result_send = result_send.clone();
            workers.push(std::thread::spawn(move || {
                start.wait();
                let result = manager.execute_shell(
                    &GatewayLocalToolScope {
                        agent_id: Some("lxo-agent".into()),
                        tool_call_id: Some(operation_id.into()),
                        action: None,
                    },
                    "concurrent-command", "/tmp", operation_id,
                );
                result_send.send((operation_id, result)).expect("publish contender outcome");
            }));
        }
        start.wait();
        paused_receive.recv_timeout(OPERATION_TEST_BOUND).expect("paused real admission");
        assert!(matches!(fixture.owner.state.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "the provider cannot see a frame before its slot is durably reserved");
        resume_send.send(()).expect("publish reserved capacity");
        let request = fixture.frame("exec");
        let (refused_id, refusal) = result_receive.recv_timeout(OPERATION_TEST_BOUND)
            .expect("the second facade is refused while the admitted operation remains active");
        let error = refusal.expect_err("only one active registration fits");
        assert!(!error.is_outcome_unknown());
        assert!(error.message.contains("capacity"));
        assert!(fixture.manager.operation(refused_id).is_none());
        let admitted_id = if refused_id == "concurrent-left" { "concurrent-right" } else { "concurrent-left" };
        let admitted = fixture.record(admitted_id);
        assert_eq!(admitted.status, GatewayLocalExecOperationStatus::Accepted);
        assert_eq!(admitted.bridge_request_id, request["requestId"].as_str().unwrap());
        let durable = read_test_operation_journal(&fixture.directory.journal());
        assert_eq!(durable.version, 3);
        assert_eq!(durable.records, vec![admitted]);
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));

        fixture.reply(&request, vec![test_exit(0, false)]);
        let (completed_id, completion) = result_receive.recv_timeout(OPERATION_TEST_BOUND)
            .expect("admitted invocation observes Exit");
        assert_eq!(completed_id, admitted_id);
        assert!(!completion.expect("known completion").outcome_unknown);
        for worker in workers {
            worker.join().expect("concurrent facade worker");
        }
        assert_eq!(fixture.frame("cancel")["requestId"], request["requestId"]);
        assert_eq!(fixture.manager.operation_history().len(), 1);
        assert_eq!(fixture.record(admitted_id).status, GatewayLocalExecOperationStatus::Completed);
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn version_two_migration_preserves_all_recoverable_record_fields() {
        for include_active in [false, true] {
            let directory = OperationTestDirectory::new("version-two-migration");
            let mut original = Vec::new();
            for (index, status) in [
                GatewayLocalExecOperationStatus::OutcomeUnknown,
                GatewayLocalExecOperationStatus::Completed,
                GatewayLocalExecOperationStatus::Cancelled,
                GatewayLocalExecOperationStatus::Failed,
                GatewayLocalExecOperationStatus::Accepted,
                GatewayLocalExecOperationStatus::Running,
            ].into_iter().enumerate() {
                if !include_active && matches!(status,
                    GatewayLocalExecOperationStatus::Accepted | GatewayLocalExecOperationStatus::Running)
                {
                    continue;
                }
                let mut record = test_record(&format!("legacy-field-{index}"), status);
                record.bridge_request_id = format!("legacy-bridge-{index}");
                record.agent_id = Some(format!("legacy-agent-{index}"));
                record.stdout = format!("legacy stdout 界 {index}");
                record.stderr = format!("legacy stderr {index}");
                record.stdout_truncated = index % 2 == 0;
                record.stderr_truncated = index % 2 != 0;
                record.exit_code = matches!(status,
                    GatewayLocalExecOperationStatus::Completed | GatewayLocalExecOperationStatus::Cancelled)
                    .then_some(17);
                record.aborted = status == GatewayLocalExecOperationStatus::Cancelled;
                record.settled_at_ms = Some(100 + index as u64);
                original.push(record);
            }
            let legacy_bytes = write_test_operation_journal(&directory.journal(), 2, &original);
            let recovered = load_test_operation_history(&directory.journal(), 8)
                .expect("migrate supported version 2");
            assert_eq!(recovered.len(), original.len());
            for (before, after) in original.iter().zip(recovered.iter()) {
                let mut expected = before.clone();
                if matches!(before.status,
                    GatewayLocalExecOperationStatus::Accepted | GatewayLocalExecOperationStatus::Running)
                {
                    expected.status = GatewayLocalExecOperationStatus::OutcomeUnknown;
                    assert!(after.settled_at_ms.is_some());
                    expected.settled_at_ms = after.settled_at_ms;
                }
                assert_eq!(after, &expected, "migration preserves identity, output and known facts");
            }
            let persisted_bytes = fs::read(directory.journal()).expect("durable version-3 migration");
            assert_ne!(persisted_bytes, legacy_bytes, "even a settled version-2 image is upgraded");
            let persisted = read_test_operation_journal(&directory.journal());
            assert_eq!(persisted.version, 3, "version 3 is the retention semantic boundary");
            assert_eq!(persisted.records, recovered.iter().cloned().collect::<Vec<_>>());
            let reopened = load_test_operation_history(&directory.journal(), 8)
                .expect("reopen migrated image");
            assert_eq!(reopened, recovered);
            assert_eq!(fs::read(directory.journal()).unwrap(), persisted_bytes,
                "an unchanged version-3 image requires no further rewrite");
        }
    }

    #[test]
    fn version_two_migration_is_durable_before_a_new_provider_frame() {
        let fixture = ShellOperationFixture::new("migration-before-admission");
        let mut retained = test_record("legacy-before-admission", GatewayLocalExecOperationStatus::OutcomeUnknown);
        retained.bridge_request_id = "legacy-bridge-before-admission".into();
        retained.stdout = "preexisting uncertain output 界".into();
        retained.stdout_truncated = true;
        write_test_operation_journal(&fixture.directory.journal(), 2, &[retained.clone()]);
        assert!(fixture.owner.state.lock().expect("lazy migration owner").is_none());

        let shell = fixture.start("after-migration", None);
        let request = fixture.frame("exec");
        let published = read_test_operation_journal(&fixture.directory.journal());
        assert_eq!(published.version, 3, "new effects require the version-3 retention boundary");
        assert_eq!(published.records.len(), 2);
        assert!(published.records.contains(&retained));
        let active = fixture.record("after-migration");
        assert_eq!(active.status, GatewayLocalExecOperationStatus::Accepted);
        assert_eq!(active.bridge_request_id, request["requestId"].as_str().unwrap());
        assert!(published.records.contains(&active));
        assert_eq!(fixture.manager.operation("legacy-before-admission"), Some(retained.clone()));

        fixture.reply(&request, vec![test_exit(0, false)]);
        assert!(!shell.join().expect("post-migration worker").expect("known Exit").outcome_unknown);
        assert_eq!(fixture.frame("cancel")["requestId"], request["requestId"]);
        let reopened = GatewayLocalExecManager::new_with_operation_journal_path(
            GatewayLocalExecSandBox::new(fixture.bridge.clone(), Arc::new(AllowGate)),
            fixture.directory.journal(),
        );
        assert_eq!(reopened.operation("legacy-before-admission"), Some(retained));
        assert_eq!(reopened.operation("after-migration").unwrap().status,
            GatewayLocalExecOperationStatus::Completed);
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn startup_prunes_only_oldest_known_terminals_and_persists_the_result() {
        for version in [2, 3] {
            let directory = OperationTestDirectory::new("startup-selective-pruning");
            let mut records = vec![
                test_record("old-completed", GatewayLocalExecOperationStatus::Completed),
                test_record("old-unknown", GatewayLocalExecOperationStatus::OutcomeUnknown),
                test_record("old-cancelled", GatewayLocalExecOperationStatus::Cancelled),
                test_record("old-failed", GatewayLocalExecOperationStatus::Failed),
                test_record("new-unknown", GatewayLocalExecOperationStatus::OutcomeUnknown),
                test_record("new-completed", GatewayLocalExecOperationStatus::Completed),
            ];
            records[1].stdout = "retain oldest unknown stdout".into();
            records[1].stdout_truncated = true;
            records[4].stderr = "retain newer unknown stderr".into();
            records[4].stderr_truncated = true;
            write_test_operation_journal(&directory.journal(), version, &records);
            let expected = vec![records[1].clone(), records[4].clone(), records[5].clone()];
            let loaded = load_test_operation_history(&directory.journal(), 3)
                .expect("prune eligible known history during load");
            assert_eq!(loaded.iter().cloned().collect::<Vec<_>>(), expected);
            let persisted = read_test_operation_journal(&directory.journal());
            assert_eq!(persisted.version, 3);
            assert_eq!(persisted.records, expected,
                "pruning is durably published even without active recovery or a format upgrade");
        }
    }

    #[test]
    fn protected_legacy_overcapacity_remains_queryable_and_blocks_new_effects() {
        for version in [2, 3] {
            let fixture = ShellOperationFixture::new("protected-legacy-overcapacity");
            let protected: Vec<_> = (0..=DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT).map(|index| {
                let status = match index {
                    1 => GatewayLocalExecOperationStatus::Accepted,
                    2 => GatewayLocalExecOperationStatus::Running,
                    _ => GatewayLocalExecOperationStatus::OutcomeUnknown,
                };
                let mut record = test_record(&format!("protected-legacy-{index}"), status);
                record.bridge_request_id = format!("legacy-request-{index}");
                record.stdout = format!("recoverable stdout 界 {index}");
                record.stderr = format!("recoverable stderr {index}");
                record.stdout_truncated = index % 2 == 0;
                record.stderr_truncated = index % 2 != 0;
                record
            }).collect();
            let mut records = vec![test_record("evictable-old", GatewayLocalExecOperationStatus::Completed)];
            records.extend(protected.iter().cloned());
            records.push(test_record("evictable-new", GatewayLocalExecOperationStatus::Failed));
            write_test_operation_journal(&fixture.directory.journal(), version, &records);

            fixture.manager.ensure_operation_journal_healthy().expect("valid overcapacity image loads");
            assert_eq!(fixture.manager.operation_history().len(), protected.len());
            for before in &protected {
                let after = fixture.manager.operation_for_agent("agent-test", &before.operation_id)
                    .expect("every protected legacy identity remains queryable");
                let mut expected = before.clone();
                if matches!(before.status,
                    GatewayLocalExecOperationStatus::Accepted | GatewayLocalExecOperationStatus::Running)
                {
                    expected.status = GatewayLocalExecOperationStatus::OutcomeUnknown;
                    assert!(after.settled_at_ms.is_some());
                    expected.settled_at_ms = after.settled_at_ms;
                }
                assert_eq!(after, expected);
                assert!(fixture.manager.operation_for_agent("other-agent", &before.operation_id).is_none());
            }
            assert!(fixture.manager.operation("evictable-old").is_none());
            assert!(fixture.manager.operation("evictable-new").is_none());
            let persisted = read_test_operation_journal(&fixture.directory.journal());
            assert_eq!(persisted.version, 3);
            assert_eq!(persisted.records, fixture.manager.operation_history());
            let before_refusals = fs::read(fixture.directory.journal()).unwrap();
            let error = fixture.manager.execute_shell(
                &GatewayLocalToolScope::default(),
                "must-not-enter", "/tmp", "overcapacity-refused",
            ).expect_err("protected legacy custody cannot be truncated to admit another effect");
            assert!(!error.is_outcome_unknown());
            assert!(error.message.contains("capacity"));
            assert!(fixture.manager.operation("overcapacity-refused").is_none());
            let duplicate = fixture.manager.execute_shell(
                &GatewayLocalToolScope::default(),
                "must-not-replay", "/tmp", "protected-legacy-0",
            ).expect_err("overcapacity recovery retains the duplicate identity");
            assert!(!duplicate.is_outcome_unknown());
            assert!(duplicate.message.contains("already exists"));
            assert_eq!(fs::read(fixture.directory.journal()).unwrap(), before_refusals);
            assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));

            let reopened = GatewayLocalExecManager::new_with_operation_journal_path(
                GatewayLocalExecSandBox::new(fixture.bridge.clone(), Arc::new(AllowGate)),
                fixture.directory.journal(),
            );
            assert_eq!(reopened.operation_history(), fixture.manager.operation_history());
        }
    }

    #[test]
    fn failed_migration_retains_parsed_custody_and_latches_admission_failure() {
        let fixture = ShellOperationFixture::new("failed-version-two-migration");
        let mut unknown = test_record("migration-unknown", GatewayLocalExecOperationStatus::OutcomeUnknown);
        unknown.stdout = "unknown output survives failed migration 界".into();
        unknown.stdout_truncated = true;
        let mut running = test_record("migration-running", GatewayLocalExecOperationStatus::Running);
        running.bridge_request_id = "legacy-running-request".into();
        running.stderr = "running output survives failed migration".into();
        running.stderr_truncated = true;
        let legacy_bytes = write_test_operation_journal(&fixture.directory.journal(), 2, &[
            unknown.clone(),
            running.clone(),
            test_record("migration-known", GatewayLocalExecOperationStatus::Completed),
        ]);
        let mut state = unloaded_operation_state(fixture.directory.journal(), 2);
        let journal_path = fixture.directory.journal();
        let retained_path = fixture.directory.0.join("before-fault.json");
        state.before_journal_publish = Some(Arc::new(move || {
            fs::rename(&journal_path, &retained_path).expect("retain the exact pre-migration bytes");
            fs::create_dir(&journal_path).expect("force actual replacement failure before rename");
        }));
        let error = GatewayLocalExecManager::load_operation_history(&mut state)
            .expect_err("real migration publication fails");
        assert!(error.message.contains("migration"));
        assert!(state.operation_journal_error.is_some());
        assert_eq!(state.operation_history.len(), 2);
        assert_eq!(state.operation_history.front(), Some(&unknown));
        let recovered_running = state.operation_history.back().unwrap().clone();
        let mut expected_running = running.clone();
        expected_running.status = GatewayLocalExecOperationStatus::OutcomeUnknown;
        expected_running.settled_at_ms = recovered_running.settled_at_ms;
        assert!(recovered_running.settled_at_ms.is_some());
        assert_eq!(recovered_running, expected_running);
        *fixture.owner.state.lock().expect("install loaded owner state") = Some(state);

        assert_eq!(fixture.manager.operation("migration-unknown"), Some(unknown.clone()));
        assert_eq!(fixture.manager.operation("migration-running"), Some(recovered_running));
        let error = fixture.manager.execute_shell(
            &GatewayLocalToolScope::default(),
            "must-not-enter", "/tmp", "failed-migration-admission",
        ).expect_err("failed migration blocks admission before a provider frame");
        assert!(!error.is_outcome_unknown());
        assert!(error.message.contains("fail-closed"));
        assert!(fixture.manager.operation("failed-migration-admission").is_none());
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
        assert_eq!(fs::read(fixture.directory.0.join("before-fault.json")).unwrap(), legacy_bytes,
            "this injected failure occurs before replacement; later durability failures need not preserve old bytes");

        fs::remove_dir(fixture.directory.journal()).expect("remove only the owned fault directory");
        fs::rename(fixture.directory.0.join("before-fault.json"), fixture.directory.journal())
            .expect("restore the retained version-2 input for fresh-owner recovery");
        assert!(fixture.manager.ensure_operation_journal_healthy().is_err(),
            "repairing the filesystem does not silently clear a live owner's failed migration");
        let reopened = GatewayLocalExecManager::new_with_operation_journal_path(
            GatewayLocalExecSandBox::new(fixture.bridge.clone(), Arc::new(AllowGate)),
            fixture.directory.journal(),
        );
        reopened.ensure_operation_journal_healthy().expect("fresh owner durably retries migration");
        assert_eq!(reopened.operation("migration-unknown"), Some(unknown));
        let recovered_running = reopened.operation("migration-running").unwrap();
        assert_eq!(recovered_running.status, GatewayLocalExecOperationStatus::OutcomeUnknown);
        assert_eq!(recovered_running.bridge_request_id, running.bridge_request_id);
        assert_eq!(recovered_running.stderr, running.stderr);
        assert!(recovered_running.stderr_truncated);
        assert_eq!(read_test_operation_journal(&fixture.directory.journal()).version, 3);
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn admission_publication_failure_restores_history_without_dispatch() {
        let fixture = ShellOperationFixture::new("failed-capacity-reservation");
        let mut unknown = test_record("reservation-unknown", GatewayLocalExecOperationStatus::OutcomeUnknown);
        unknown.stdout = "protected output".into();
        let known = test_record("reservation-known", GatewayLocalExecOperationStatus::Completed);
        write_test_operation_journal(&fixture.directory.journal(), 3, &[unknown.clone(), known.clone()]);
        let journal_path = fixture.directory.journal();
        let retained_path = fixture.directory.0.join("before-fault.json");
        fixture.owner.with_state(|state| {
            state.operation_history_limit = 2;
            state.before_journal_publish = Some(Arc::new(move || {
                fs::rename(&journal_path, &retained_path).expect("retain pre-admission input");
                fs::create_dir(&journal_path).expect("refuse actual reservation publication");
            }));
        });
        let error = fixture.manager.execute_shell(
            &GatewayLocalToolScope::default(),
            "must-not-enter", "/tmp", "failed-reservation",
        ).expect_err("unpublished capacity cannot authorize provider dispatch");
        assert!(!error.is_outcome_unknown());
        assert!(error.message.contains("persistence"));
        assert!(fixture.manager.ensure_operation_journal_healthy().is_err());
        assert_eq!(fixture.manager.operation_history(), vec![unknown.clone(), known]);
        assert_eq!(fixture.manager.operation("reservation-unknown"), Some(unknown));
        assert!(fixture.manager.operation("failed-reservation").is_none());
        assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn unsupported_and_duplicate_journals_block_admission_without_rewriting() {
        for (label, version, duplicate) in [
            ("old-unsupported", 1, false),
            ("future-unsupported", 4, false),
            ("legacy-duplicates", 2, true),
            ("current-duplicates", 3, true),
        ] {
            let fixture = ShellOperationFixture::new(label);
            let mut records = vec![test_record("invalid-image-identity",
                GatewayLocalExecOperationStatus::OutcomeUnknown)];
            if duplicate {
                records.push(test_record("invalid-image-identity",
                    GatewayLocalExecOperationStatus::Completed));
            }
            let original = write_test_operation_journal(&fixture.directory.journal(), version, &records);
            let error = fixture.manager.execute_shell(
                &GatewayLocalToolScope::default(),
                "must-not-enter", "/tmp", "invalid-image-admission",
            ).expect_err("invalid journal cannot authorize a provider frame");
            assert!(!error.is_outcome_unknown());
            assert!(error.message.contains(if duplicate { "duplicate" } else { "unsupported" }));
            assert_eq!(fs::read(fixture.directory.journal()).unwrap(), original);
            assert!(fixture.manager.operation("invalid-image-admission").is_none());
            assert!(matches!(fixture.receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }
    }

    #[test]
    fn corrupt_journal_fails_closed_instead_of_becoming_empty_history() {
        let path = unique_test_path("corrupt");
        fs::write(&path, b"{not-json").expect("write corrupt journal");
        let error = load_test_operation_history(&path, 8)
            .expect_err("corrupt journal must fail closed");
        assert!(error.message.contains("corrupt"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn unknown_journal_version_fails_closed() {
        let path = unique_test_path("version");
        fs::write(&path, br#"{"version":999,"records":[]}"#).expect("write versioned journal");
        let error = load_test_operation_history(&path, 8)
            .expect_err("unknown version must fail closed");
        assert!(error.message.contains("unsupported"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn accepted_start_ambiguity_recovers_durably_as_outcome_unknown() {
        let path = unique_test_path("accepted-recovery");
        let mut history = VecDeque::new();
        history.push_back(test_record(
            "operation-accepted-ambiguous",
            GatewayLocalExecOperationStatus::Accepted,
        ));
        GatewayLocalExecManager::persist_operation_history(&path, &history)
            .expect("persist accepted journal");

        let recovered = load_test_operation_history(&path, 8)
            .expect("recover accepted journal");
        let record = recovered.front().expect("accepted record");
        assert_eq!(
            record.status,
            GatewayLocalExecOperationStatus::OutcomeUnknown
        );
        assert!(record.settled_at_ms.is_some());

        let second = load_test_operation_history(&path, 8)
            .expect("re-read accepted recovery");
        assert_eq!(
            second.front().expect("accepted record").status,
            GatewayLocalExecOperationStatus::OutcomeUnknown
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn process_death_recovery_durably_marks_running_as_outcome_unknown() {
        let path = unique_test_path("recovery");
        let mut history = VecDeque::new();
        history.push_back(test_record(
            "operation-recover",
            GatewayLocalExecOperationStatus::Running,
        ));
        GatewayLocalExecManager::persist_operation_history(&path, &history)
            .expect("persist running journal");

        let recovered =
            load_test_operation_history(&path, 8).expect("recover journal");
        let record = recovered.front().expect("record");
        assert_eq!(
            record.status,
            GatewayLocalExecOperationStatus::OutcomeUnknown
        );
        assert!(record.settled_at_ms.is_some());

        let second = load_test_operation_history(&path, 8)
            .expect("re-read recovered journal");
        assert_eq!(
            second.front().expect("record").status,
            GatewayLocalExecOperationStatus::OutcomeUnknown
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn atomic_persistence_failure_is_reported() {
        let parent_file = unique_test_path("parent-file");
        fs::write(&parent_file, b"not-a-directory").expect("write parent sentinel");
        let path = parent_file.join("journal.json");
        let error = GatewayLocalExecManager::persist_operation_history(&path, &VecDeque::new())
            .expect_err("invalid parent must fail persistence");
        assert!(
            matches!(
                error.kind(),
                io::ErrorKind::NotADirectory
                    | io::ErrorKind::AlreadyExists
                    | io::ErrorKind::PermissionDenied
                    | io::ErrorKind::Other
            ),
            "unexpected persistence error: {error}"
        );
        let _ = fs::remove_file(parent_file);
    }

    #[test]
    fn pre_send_journal_failure_does_not_send_exec_frame() {
        let bridge = SandLocalExecBridge::with_sources(
            Arc::new(|| 1000),
            Arc::new(|| "provider-test".to_string()),
        );
        let (send, receive) = mpsc::channel();
        let _registration = bridge.register_provider(send);
        let _ = receive
            .recv_timeout(Duration::from_secs(1))
            .expect("welcome");
        bridge.submit_responses(json!({
            "providerId":"provider-test",
            "frames":[{
                "kind":"hello",
                "localRoot":"/tmp",
                "terminalsFolder":"terminals",
                "computerId":"computer-test",
                "label":"Test"
            }]
        }));

        let parent_file = unique_test_path("presend-parent");
        fs::write(&parent_file, b"not-a-directory").expect("write parent sentinel");
        let journal_path = parent_file.join("journal.json");
        let sandbox = GatewayLocalExecSandBox::new(bridge, Arc::new(AllowGate));
        let manager =
            GatewayLocalExecManager::new_with_operation_journal_path(sandbox, journal_path);
        let error = manager
            .execute_shell(
                &GatewayLocalToolScope {
                    agent_id: Some("agent-test".into()),
                    tool_call_id: Some("operation-presend".into()),
                    action: Some("run-command".into()),
                },
                "echo should-not-run",
                "/tmp",
                "operation-presend",
            )
            .expect_err("journal failure must fail before provider send");
        assert!(error.message.contains("fail-closed"));
        assert!(receive.recv_timeout(Duration::from_millis(50)).is_err());
        let _ = fs::remove_file(parent_file);
    }

    #[test]
    fn completed_bounded_output_round_trips_through_journal() {
        let path = unique_test_path("bounded-output");
        let oversized = "x".repeat(MAX_LOCAL_EXEC_DURABLE_OUTPUT_BYTES + 128);
        let (stdout, stdout_truncated) = durable_shell_output(&oversized, false);
        let mut record = test_record(
            "operation-output",
            GatewayLocalExecOperationStatus::Completed,
        );
        record.stdout = stdout;
        record.stdout_truncated = stdout_truncated;
        let mut history = VecDeque::new();
        history.push_back(record);
        GatewayLocalExecManager::persist_operation_history(&path, &history)
            .expect("persist completed output");

        let recovered = load_test_operation_history(&path, 8)
            .expect("reload completed output");
        let recovered = recovered.front().expect("recovered record");
        assert_eq!(recovered.status, GatewayLocalExecOperationStatus::Completed);
        assert_eq!(recovered.stdout.len(), MAX_LOCAL_EXEC_DURABLE_OUTPUT_BYTES);
        assert!(recovered.stdout_truncated);
        assert!(recovered.stderr.is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn duplicate_operation_id_from_durable_history_is_not_replayed() {
        let path = unique_test_path("duplicate");
        let mut history = VecDeque::new();
        history.push_back(test_record(
            "operation-stable",
            GatewayLocalExecOperationStatus::Completed,
        ));
        GatewayLocalExecManager::persist_operation_history(&path, &history)
            .expect("persist history");

        let bridge = SandLocalExecBridge::with_sources(
            Arc::new(|| 1000),
            Arc::new(|| "provider-test".to_string()),
        );
        let (send, receive) = mpsc::channel();
        let _registration = bridge.register_provider(send);
        let _ = receive
            .recv_timeout(Duration::from_secs(1))
            .expect("welcome");
        bridge.submit_responses(json!({
            "providerId":"provider-test",
            "frames":[{
                "kind":"hello",
                "localRoot":"/tmp",
                "terminalsFolder":"terminals",
                "computerId":"computer-test",
                "label":"Test"
            }]
        }));
        let sandbox = GatewayLocalExecSandBox::new(bridge, Arc::new(AllowGate));
        let manager =
            GatewayLocalExecManager::new_with_operation_journal_path(sandbox, path.clone());
        let error = manager
            .execute_shell(
                &GatewayLocalToolScope {
                    agent_id: Some("agent-test".into()),
                    tool_call_id: Some("operation-stable".into()),
                    action: Some("run-command".into()),
                },
                "echo should-not-replay",
                "/tmp",
                "operation-stable",
            )
            .expect_err("duplicate operationId must not replay");
        assert!(error.message.contains("already exists"));
        assert!(receive.recv_timeout(Duration::from_millis(50)).is_err());
        let _ = fs::remove_file(path);
    }
}
