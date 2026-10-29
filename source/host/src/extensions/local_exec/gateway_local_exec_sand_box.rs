use std::collections::{HashMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, AtomicU64, Ordering},
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
use crate::host_paths::get_sand_root_dir;

pub const FALLBACK_TERMINALS_FOLDER: &str = "terminals";
pub const DEFAULT_MAX_LOCAL_EXEC_FILE_BYTES: usize = 100 * 1024 * 1024;
pub const MAX_LOCAL_EXEC_SHELL_STREAM_BYTES: usize = 256 * 1024;
pub const MAX_LOCAL_EXEC_DURABLE_OUTPUT_BYTES: usize = 64 * 1024;
pub const DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT: usize = 128;
pub const LOCAL_EXEC_OPERATION_JOURNAL_VERSION: u32 = 2;
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
    cancel_requested: bool,
}

#[derive(Clone)]
pub struct GatewayLocalExecManager {
    sandbox: GatewayLocalExecSandBox,
    next_exec_id: Arc<AtomicU32>,
    active_operations: Arc<Mutex<HashMap<String, GatewayLocalExecActiveOperation>>>,
    operation_history: Arc<Mutex<VecDeque<GatewayLocalExecOperationRecord>>>,
    operation_history_limit: usize,
    operation_journal_path: Arc<PathBuf>,
    operation_journal_error: Arc<Mutex<Option<String>>>,
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
    pub fn new(bridge: SandLocalExecBridge, gate: Arc<dyn GatewayLocalToolGate>) -> Self {
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
        Self::new_with_operation_journal_path(
            sandbox,
            get_sand_root_dir().join(SAND_LOCAL_EXEC_OPERATION_JOURNAL_FILE_NAME),
        )
    }

    fn new_with_operation_journal_path(
        sandbox: GatewayLocalExecSandBox,
        operation_journal_path: PathBuf,
    ) -> Self {
        let (history, operation_journal_error) = match Self::load_operation_history(
            &operation_journal_path,
            DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT,
        ) {
            Ok(history) => (history, None),
            Err(error) => (VecDeque::new(), Some(error.message)),
        };
        Self {
            sandbox,
            next_exec_id: Arc::new(AtomicU32::new(1)),
            active_operations: Arc::new(Mutex::new(HashMap::new())),
            operation_history: Arc::new(Mutex::new(history)),
            operation_history_limit: DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT,
            operation_journal_path: Arc::new(operation_journal_path),
            operation_journal_error: Arc::new(Mutex::new(operation_journal_error)),
        }
    }

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }

    fn load_operation_history(
        path: &Path,
        limit: usize,
    ) -> Result<VecDeque<GatewayLocalExecOperationRecord>, SandLocalExecError> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(VecDeque::new()),
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
        if journal.version != LOCAL_EXEC_OPERATION_JOURNAL_VERSION {
            return Err(SandLocalExecError::new(format!(
                "local-exec operation journal version {} is unsupported; expected {}; execution is disabled",
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
        let mut recovered_uncertain = false;
        for record in &mut journal.records {
            if matches!(
                record.status,
                GatewayLocalExecOperationStatus::Accepted
                    | GatewayLocalExecOperationStatus::Running
            ) {
                record.status = GatewayLocalExecOperationStatus::OutcomeUnknown;
                record.settled_at_ms = Some(Self::now_ms());
                recovered_uncertain = true;
            }
        }
        if journal.records.len() > limit.max(1) {
            let keep_from = journal.records.len() - limit.max(1);
            journal.records.drain(..keep_from);
        }
        let history = journal.records.into_iter().collect::<VecDeque<_>>();
        if recovered_uncertain {
            Self::persist_operation_history(path, &history).map_err(|error| {
                SandLocalExecError::new(format!(
                    "local-exec process-death recovery could not persist outcome_unknown state; execution is disabled: {error}"
                ))
            })?;
        }
        Ok(history)
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

    fn set_operation_journal_error(&self, message: String) {
        *self
            .operation_journal_error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(message);
    }

    pub fn ensure_operation_journal_healthy(&self) -> Result<(), SandLocalExecError> {
        match self
            .operation_journal_error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            Some(error) => Err(SandLocalExecError::new(format!(
                "local-exec operation journal unavailable; shell execution is fail-closed: {error}"
            ))),
            None => Ok(()),
        }
    }

    pub fn stable_result_ref(operation_id: &str) -> String {
        format!("fabushi-local-exec://operation/{operation_id}")
    }

    fn persist_operation_snapshot(&self) -> Result<(), SandLocalExecError> {
        let mut records = self
            .operation_history
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        for (operation_id, active) in self
            .active_operations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
        {
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
        Self::persist_operation_history(&self.operation_journal_path, &records).map_err(|error| {
            let message =
                format!("local-exec operation journal persistence is fail-closed: {error}");
            self.set_operation_journal_error(message.clone());
            SandLocalExecError::new(message)
        })
    }

    pub fn operation(&self, operation_id: &str) -> Option<GatewayLocalExecOperationRecord> {
        if let Some(active) = self
            .active_operations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(operation_id)
            .cloned()
        {
            return Some(GatewayLocalExecOperationRecord {
                operation_id: operation_id.to_string(),
                agent_id: active.agent_id,
                bridge_request_id: active.bridge_request_id,
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
        self.operation_history
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .rev()
            .find(|record| record.operation_id == operation_id)
            .cloned()
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
        self.operation_history
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .cloned()
            .collect()
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

    pub fn cancel_operation(&self, operation_id: &str) -> bool {
        let mut active = self
            .active_operations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(operation) = active.get_mut(operation_id) else {
            return false;
        };
        operation.cancel_requested = true;
        operation.cancellation.cancel()
    }

    pub fn cancel_operation_for_agent(&self, agent_id: &str, operation_id: &str) -> bool {
        let owned = self
            .active_operations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(operation_id)
            .is_some_and(|operation| operation.agent_id.as_deref() == Some(agent_id));
        owned && self.cancel_operation(operation_id)
    }

    fn register_operation(
        &self,
        operation_id: &str,
        agent_id: Option<&str>,
        cancellation: LocalExecCancellationHandle,
    ) -> Result<(), SandLocalExecError> {
        self.ensure_operation_journal_healthy()?;
        let mut active = self
            .active_operations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if active.contains_key(operation_id)
            || self
                .operation_history
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .iter()
                .any(|record| record.operation_id == operation_id)
        {
            return Err(SandLocalExecError::new(format!(
                "local-exec operationId already exists and will not be replayed: {operation_id}"
            )));
        }
        active.insert(
            operation_id.to_string(),
            GatewayLocalExecActiveOperation {
                agent_id: agent_id.map(str::to_string),
                bridge_request_id: cancellation.request_id().to_string(),
                cancellation,
                started: false,
                cancel_requested: false,
            },
        );
        drop(active);
        if let Err(error) = self.persist_operation_snapshot() {
            self.active_operations
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(operation_id);
            return Err(error);
        }
        Ok(())
    }

    fn mark_operation_started(&self, operation_id: &str) -> Result<(), SandLocalExecError> {
        let changed = {
            let mut active = self
                .active_operations
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let operation = active.get_mut(operation_id).ok_or_else(|| {
                SandLocalExecError::new(format!(
                    "local-exec operationId disappeared before started journal: {operation_id}"
                ))
            })?;
            if operation.started {
                false
            } else {
                operation.started = true;
                true
            }
        };
        if changed {
            self.persist_operation_snapshot()?;
        }
        Ok(())
    }

    fn cancellation_requested(&self, operation_id: &str) -> bool {
        self.active_operations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(operation_id)
            .is_some_and(|operation| operation.cancel_requested)
    }

    fn finish_operation(
        &self,
        operation_id: &str,
        status: GatewayLocalExecOperationStatus,
        result: Option<&GatewayLocalExecShellResult>,
    ) -> Result<(), SandLocalExecError> {
        let active = self
            .active_operations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(operation_id);
        let Some(active) = active else {
            return Ok(());
        };
        let mut history = self
            .operation_history
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while history.len() >= self.operation_history_limit.max(1) {
            history.pop_front();
        }
        let (stdout, stdout_truncated) = result
            .map(|result| durable_shell_output(&result.stdout, result.stdout_truncated))
            .unwrap_or_default();
        let (stderr, stderr_truncated) = result
            .map(|result| durable_shell_output(&result.stderr, result.stderr_truncated))
            .unwrap_or_default();
        history.push_back(GatewayLocalExecOperationRecord {
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
        drop(history);
        self.persist_operation_snapshot().map_err(|error| {
            SandLocalExecError::new(format!(
                "{}; operation result is not safely durable (resultRef={})",
                error.message,
                Self::stable_result_ref(operation_id)
            ))
        })
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
        self.ensure_operation_journal_healthy()?;
        let message = PRODUCTION_LOCAL_EXEC_CODEC.shell_stream_server_message(
            self.next_id(),
            command,
            working_directory,
            tool_call_id,
        );
        let mut request = match self.open_exec_with_before_send(
            scope,
            "run-command",
            command,
            message,
            |cancellation| {
                self.register_operation(tool_call_id, scope.agent_id.as_deref(), cancellation)
            },
        ) {
            Ok(request) => request,
            Err(error) => {
                // The registration callback runs before the bridge sends the
                // frame. If that send fails, close the already-journaled
                // operation instead of leaving it indefinitely Accepted.
                if let Err(cleanup_error) = self.finish_operation(
                    tool_call_id,
                    GatewayLocalExecOperationStatus::Failed,
                    None,
                ) {
                    return Err(SandLocalExecError::new(format!(
                        "{}; operation cleanup was not safely durable: {}",
                        error.message, cleanup_error.message
                    )));
                }
                return Err(error);
            }
        };
        let mut result = GatewayLocalExecShellResult {
            request_id: tool_call_id.to_string(),
            ..GatewayLocalExecShellResult::default()
        };
        let mut terminal_status = None;
        let outcome = (|| -> Result<GatewayLocalExecShellResult, SandLocalExecError> {
            loop {
                let response = match request.recv_timeout(Duration::from_secs(10)) {
                    Ok(response) => response,
                    Err(_) => {
                        terminal_status = Some(GatewayLocalExecOperationStatus::OutcomeUnknown);
                        result.outcome_unknown = true;
                        request.close();
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
                                    self.mark_operation_started(tool_call_id)?;
                                    result.stdout_truncated |=
                                        append_bounded_shell_output(&mut result.stdout, data);
                                }
                                ProductionShellStreamEvent::Stderr(data) => {
                                    self.mark_operation_started(tool_call_id)?;
                                    result.stderr_truncated |=
                                        append_bounded_shell_output(&mut result.stderr, data);
                                }
                                ProductionShellStreamEvent::Exit { code, cwd, aborted } => {
                                    result.exit_code = Some(*code);
                                    result.cwd = (!cwd.is_empty()).then(|| cwd.clone());
                                    result.aborted = *aborted;
                                    terminal_status = Some(
                                        if *aborted || self.cancellation_requested(tool_call_id) {
                                            GatewayLocalExecOperationStatus::Cancelled
                                        } else {
                                            GatewayLocalExecOperationStatus::Completed
                                        },
                                    );
                                    request.close();
                                    return Ok(result.clone());
                                }
                                ProductionShellStreamEvent::Rejected(reason)
                                | ProductionShellStreamEvent::PermissionDenied(reason) => {
                                    terminal_status =
                                        Some(if self.cancellation_requested(tool_call_id) {
                                            GatewayLocalExecOperationStatus::Cancelled
                                        } else {
                                            GatewayLocalExecOperationStatus::Failed
                                        });
                                    request.close();
                                    return Err(SandLocalExecError::new(reason.clone()));
                                }
                                ProductionShellStreamEvent::Start
                                | ProductionShellStreamEvent::Backgrounded => {
                                    self.mark_operation_started(tool_call_id)?;
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
                            GatewayExecControl::Throw { error, stack_trace } => {
                                self.sandbox.report_exec_failure(scope, &error);
                                terminal_status =
                                    Some(if self.cancellation_requested(tool_call_id) {
                                        GatewayLocalExecOperationStatus::Cancelled
                                    } else {
                                        GatewayLocalExecOperationStatus::Failed
                                    });
                                request.close();
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
                                let cancelled = self.cancellation_requested(tool_call_id);
                                terminal_status = Some(if cancelled {
                                    GatewayLocalExecOperationStatus::Cancelled
                                } else {
                                    GatewayLocalExecOperationStatus::OutcomeUnknown
                                });
                                request.close();
                                if !cancelled {
                                    result.outcome_unknown = true;
                                }
                                return Ok(result.clone());
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
                        terminal_status = Some(if self.cancellation_requested(tool_call_id) {
                            GatewayLocalExecOperationStatus::Cancelled
                        } else {
                            GatewayLocalExecOperationStatus::Failed
                        });
                        request.close();
                        return Err(SandLocalExecError::new(format_command_error(
                            command,
                            &CommandErrorResult {
                                code: None,
                                termination: CommandTermination::Error,
                                signal: None,
                                output_limit_exceeded: false,
                            },
                            &error,
                            "",
                            0,
                        )));
                    }
                    _ => {}
                }
            }
        })();

        let status = terminal_status.unwrap_or_else(|| {
            if self.cancellation_requested(tool_call_id) {
                GatewayLocalExecOperationStatus::Cancelled
            } else {
                GatewayLocalExecOperationStatus::Failed
            }
        });
        if let Err(error) = self.finish_operation(tool_call_id, status, outcome.as_ref().ok()) {
            return Err(error);
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

    #[test]
    fn corrupt_journal_fails_closed_instead_of_becoming_empty_history() {
        let path = unique_test_path("corrupt");
        fs::write(&path, b"{not-json").expect("write corrupt journal");
        let error = GatewayLocalExecManager::load_operation_history(&path, 8)
            .expect_err("corrupt journal must fail closed");
        assert!(error.message.contains("corrupt"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn unknown_journal_version_fails_closed() {
        let path = unique_test_path("version");
        fs::write(&path, br#"{"version":999,"records":[]}"#).expect("write versioned journal");
        let error = GatewayLocalExecManager::load_operation_history(&path, 8)
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

        let recovered = GatewayLocalExecManager::load_operation_history(&path, 8)
            .expect("recover accepted journal");
        let record = recovered.front().expect("accepted record");
        assert_eq!(
            record.status,
            GatewayLocalExecOperationStatus::OutcomeUnknown
        );
        assert!(record.settled_at_ms.is_some());

        let second = GatewayLocalExecManager::load_operation_history(&path, 8)
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
            GatewayLocalExecManager::load_operation_history(&path, 8).expect("recover journal");
        let record = recovered.front().expect("record");
        assert_eq!(
            record.status,
            GatewayLocalExecOperationStatus::OutcomeUnknown
        );
        assert!(record.settled_at_ms.is_some());

        let second = GatewayLocalExecManager::load_operation_history(&path, 8)
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

        let recovered = GatewayLocalExecManager::load_operation_history(&path, 8)
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
