use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use chrono::{SecondsFormat, Utc};
use path_clean::PathClean;
use prost::{Message as ProstMessage, Oneof};

pub const BOX_EXEC_DAEMON_HOST: &str = "127.0.0.1";
pub const BOX_EXEC_DAEMON_PORT: u16 = 1337;
pub const BOX_EXEC_DAEMON_AUTH_TOKEN: &str = "local";
pub const BOX_TERMINAL_VIRTUAL_PREFIX: &str =
    "/root/.cursor/projects/workspace/terminals/";

#[derive(Clone, PartialEq, prost::Message)]
pub struct PingRequest {}

#[derive(Clone, PartialEq, prost::Message)]
pub struct PingResponse {}

#[derive(Clone, PartialEq, prost::Message)]
pub struct GetCapabilitiesRequest {}

#[derive(Clone, PartialEq, prost::Message)]
pub struct GetCapabilitiesResponse {
    #[prost(bool, optional, tag = "1")]
    pub computer_use_supported: Option<bool>,
    #[prost(bool, optional, tag = "2")]
    pub install_plugin_artifact_supported: Option<bool>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct UpdateEnvironmentVariablesRequest {
    #[prost(map = "string, string", tag = "1")]
    pub env: HashMap<String, String>,
    #[prost(bool, tag = "2")]
    pub replace: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct UpdateEnvironmentVariablesResponse {
    #[prost(uint32, tag = "1")]
    pub applied: u32,
    #[prost(uint32, tag = "2")]
    pub removed: u32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct LoadMcpServersRequest {
    #[prost(string, tag = "1")]
    pub mcp_config_json: String,
    #[prost(bool, tag = "2")]
    pub remove_missing: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct LoadMcpServersResponse {
    #[prost(string, repeated, tag = "1")]
    pub loaded_server_names: Vec<String>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ReadArgs {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "2")]
    pub tool_call_id: String,
    #[prost(int32, optional, tag = "4")]
    pub offset: Option<i32>,
    #[prost(uint32, optional, tag = "5")]
    pub limit: Option<u32>,
    #[prost(string, optional, tag = "6")]
    pub encoding_hint: Option<String>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ReadResult {
    #[prost(oneof = "read_result::Result", tags = "1, 2, 3, 4, 5, 6")]
    pub result: Option<read_result::Result>,
}
pub mod read_result {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Result {
        #[prost(message, tag = "1")]
        Success(ReadSuccess),
        #[prost(message, tag = "2")]
        Error(ReadError),
        #[prost(message, tag = "3")]
        Rejected(ReadRejected),
        #[prost(message, tag = "4")]
        FileNotFound(ReadFileNotFound),
        #[prost(message, tag = "5")]
        PermissionDenied(ReadPermissionDenied),
        #[prost(message, tag = "6")]
        InvalidFile(ReadInvalidFile),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ReadSuccess {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(oneof = "read_success::Output", tags = "2, 5")]
    pub output: Option<read_success::Output>,
    #[prost(int32, tag = "3")]
    pub total_lines: i32,
    #[prost(int64, tag = "4")]
    pub file_size: i64,
    #[prost(bool, tag = "6")]
    pub truncated: bool,
    #[prost(bytes = "vec", optional, tag = "7")]
    pub output_blob_id: Option<Vec<u8>>,
    #[prost(bool, tag = "8")]
    pub range_applied: bool,
}
pub mod read_success {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Output {
        #[prost(string, tag = "2")]
        Content(String),
        #[prost(bytes, tag = "5")]
        Data(Vec<u8>),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ReadError {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "2")]
    pub error: String,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ReadRejected {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "2")]
    pub reason: String,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ReadFileNotFound {
    #[prost(string, tag = "1")]
    pub path: String,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ReadPermissionDenied {
    #[prost(string, tag = "1")]
    pub path: String,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ReadInvalidFile {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "2")]
    pub reason: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellArgs {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(int32, tag = "3")]
    pub timeout: i32,
    #[prost(string, tag = "4")]
    pub tool_call_id: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellResult {
    #[prost(oneof = "shell_result::Result", tags = "1, 2, 3, 5")]
    pub result: Option<shell_result::Result>,
}
pub mod shell_result {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Result {
        #[prost(message, tag = "1")]
        Success(ShellSuccess),
        #[prost(message, tag = "2")]
        Failure(ShellFailure),
        #[prost(message, tag = "3")]
        Timeout(ShellTimeout),
        #[prost(message, tag = "5")]
        SpawnError(ShellSpawnError),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellSuccess {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(int32, tag = "3")]
    pub exit_code: i32,
    #[prost(string, tag = "4")]
    pub signal: String,
    #[prost(string, tag = "5")]
    pub stdout: String,
    #[prost(string, tag = "6")]
    pub stderr: String,
    #[prost(int32, tag = "7")]
    pub execution_time: i32,
    #[prost(string, optional, tag = "10")]
    pub interleaved_output: Option<String>,
    #[prost(int32, optional, tag = "13")]
    pub local_execution_time_ms: Option<i32>,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellFailure {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(int32, tag = "3")]
    pub exit_code: i32,
    #[prost(string, tag = "4")]
    pub signal: String,
    #[prost(string, tag = "5")]
    pub stdout: String,
    #[prost(string, tag = "6")]
    pub stderr: String,
    #[prost(int32, tag = "7")]
    pub execution_time: i32,
    #[prost(string, optional, tag = "9")]
    pub interleaved_output: Option<String>,
    #[prost(bool, tag = "11")]
    pub aborted: bool,
    #[prost(int32, optional, tag = "12")]
    pub local_execution_time_ms: Option<i32>,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellTimeout {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(int32, tag = "3")]
    pub timeout_ms: i32,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellSpawnError {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(string, tag = "3")]
    pub error: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellStream {
    #[prost(oneof = "shell_stream::Event", tags = "1, 2, 3, 4")]
    pub event: Option<shell_stream::Event>,
}
pub mod shell_stream {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Event {
        #[prost(message, tag = "1")]
        Stdout(ShellStreamStdout),
        #[prost(message, tag = "2")]
        Stderr(ShellStreamStderr),
        #[prost(message, tag = "3")]
        Exit(ShellStreamExit),
        #[prost(message, tag = "4")]
        Start(ShellStreamStart),
    }
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellStreamStart {}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellStreamStdout {
    #[prost(string, tag = "1")]
    pub data: String,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellStreamStderr {
    #[prost(string, tag = "1")]
    pub data: String,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellStreamExit {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(string, tag = "2")]
    pub cwd: String,
    #[prost(bool, tag = "4")]
    pub aborted: bool,
    #[prost(int32, optional, tag = "6")]
    pub local_execution_time_ms: Option<i32>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct BackgroundShellSpawnArgs {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(string, tag = "3")]
    pub tool_call_id: String,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct BackgroundShellSpawnResult {
    #[prost(oneof = "background_shell_spawn_result::Result", tags = "1, 2")]
    pub result: Option<background_shell_spawn_result::Result>,
}
pub mod background_shell_spawn_result {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Result {
        #[prost(message, tag = "1")]
        Success(BackgroundShellSpawnSuccess),
        #[prost(message, tag = "2")]
        Error(BackgroundShellSpawnError),
    }
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct BackgroundShellSpawnSuccess {
    #[prost(uint32, tag = "1")]
    pub shell_id: u32,
    #[prost(string, tag = "2")]
    pub command: String,
    #[prost(string, tag = "3")]
    pub working_directory: String,
    #[prost(uint32, optional, tag = "4")]
    pub pid: Option<u32>,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct BackgroundShellSpawnError {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(string, tag = "3")]
    pub error: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct WriteShellStdinArgs {
    #[prost(uint32, tag = "1")]
    pub shell_id: u32,
    #[prost(string, tag = "2")]
    pub chars: String,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct WriteShellStdinResult {
    #[prost(oneof = "write_shell_stdin_result::Result", tags = "1, 2")]
    pub result: Option<write_shell_stdin_result::Result>,
}
pub mod write_shell_stdin_result {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Result {
        #[prost(message, tag = "1")]
        Success(WriteShellStdinSuccess),
        #[prost(message, tag = "2")]
        Error(WriteShellStdinError),
    }
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct WriteShellStdinSuccess {
    #[prost(uint32, tag = "1")]
    pub shell_id: u32,
    #[prost(uint32, tag = "2")]
    pub terminal_file_length_before_input_written: u32,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct WriteShellStdinError {
    #[prost(string, tag = "1")]
    pub error: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ExecServerMessage {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(string, tag = "15")]
    pub exec_id: String,
    #[prost(oneof = "exec_server_message::Message", tags = "2, 7, 29, 14, 16, 23, 52")]
    pub message: Option<exec_server_message::Message>,
}
pub mod exec_server_message {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Message {
        #[prost(message, tag = "2")]
        ShellArgs(ShellArgs),
        #[prost(message, tag = "7")]
        ReadArgs(ReadArgs),
        #[prost(message, tag = "29")]
        RedactedReadArgs(ReadArgs),
        #[prost(message, tag = "14")]
        ShellStreamArgs(ShellArgs),
        #[prost(message, tag = "16")]
        BackgroundShellSpawnArgs(BackgroundShellSpawnArgs),
        #[prost(message, tag = "23")]
        WriteShellStdinArgs(WriteShellStdinArgs),
        #[prost(message, tag = "52")]
        MiniSweAgentBashArgs(ShellArgs),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ExecClientMessage {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(string, tag = "15")]
    pub exec_id: String,
    #[prost(int32, optional, tag = "39")]
    pub local_execution_time_ms: Option<i32>,
    #[prost(oneof = "exec_client_message::Message", tags = "2, 7, 29, 14, 16, 23, 55")]
    pub message: Option<exec_client_message::Message>,
}
pub mod exec_client_message {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Message {
        #[prost(message, tag = "2")]
        ShellResult(ShellResult),
        #[prost(message, tag = "7")]
        ReadResult(ReadResult),
        #[prost(message, tag = "29")]
        RedactedReadResult(ReadResult),
        #[prost(message, tag = "14")]
        ShellStream(ShellStream),
        #[prost(message, tag = "16")]
        BackgroundShellSpawnResult(BackgroundShellSpawnResult),
        #[prost(message, tag = "23")]
        WriteShellStdinResult(WriteShellStdinResult),
        #[prost(message, tag = "55")]
        MiniSweAgentBashResult(ShellResult),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ExecClientControlMessage {
    #[prost(oneof = "exec_client_control_message::Message", tags = "1, 2")]
    pub message: Option<exec_client_control_message::Message>,
}
pub mod exec_client_control_message {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Message {
        #[prost(message, tag = "1")]
        StreamClose(ExecClientStreamClose),
        #[prost(message, tag = "2")]
        Throw(ExecClientThrow),
    }
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ExecClientStreamClose {
    #[prost(uint32, tag = "1")]
    pub id: u32,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ExecClientThrow {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(string, tag = "2")]
    pub error: String,
    #[prost(string, optional, tag = "3")]
    pub stack_trace: Option<String>,
    #[prost(string, optional, tag = "4")]
    pub error_code: Option<String>,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ExecStreamElement {
    #[prost(oneof = "exec_stream_element::Element", tags = "1, 2")]
    pub element: Option<exec_stream_element::Element>,
}
pub mod exec_stream_element {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Element {
        #[prost(message, tag = "1")]
        ExecClientMessage(ExecClientMessage),
        #[prost(message, tag = "2")]
        ExecClientControlMessage(ExecClientControlMessage),
    }
}

#[derive(Debug, Clone)]
pub struct BoxExecDaemonOptions {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub auth_token: Option<String>,
    pub workspace_root: PathBuf,
    pub terminals_directory: Option<PathBuf>,
    pub environment: Option<HashMap<String, String>>,
}

struct BackgroundProcess {
    stdin: Arc<Mutex<ChildStdin>>,
    terminal_path: PathBuf,
    pid: u32,
}

#[derive(Debug)]
struct ProcessOutcome {
    code: i32,
    signal: String,
    stdout: String,
    stderr: String,
    elapsed_ms: u64,
    timed_out: bool,
    aborted: bool,
}

#[derive(Debug)]
enum ShellPipeEvent {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

#[derive(Debug)]
struct PathRejectedError(String);

impl std::fmt::Display for PathRejectedError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub struct BoxExecRuntime {
    pub workspace_root: PathBuf,
    pub terminals_directory: PathBuf,
    environment: Mutex<HashMap<String, String>>,
    foreground_pids: Mutex<HashSet<u32>>,
    background: Arc<Mutex<HashMap<u32, BackgroundProcess>>>,
    next_shell_id: AtomicU32,
    stopping: AtomicBool,
}

impl BoxExecRuntime {
    pub fn new(
        workspace_root: PathBuf,
        terminals_directory: PathBuf,
        environment: HashMap<String, String>,
    ) -> Self {
        Self {
            workspace_root,
            terminals_directory,
            environment: Mutex::new(environment),
            foreground_pids: Mutex::new(HashSet::new()),
            background: Arc::new(Mutex::new(HashMap::new())),
            next_shell_id: AtomicU32::new(1),
            stopping: AtomicBool::new(false),
        }
    }

    pub fn apply_environment(
        &self,
        request: &UpdateEnvironmentVariablesRequest,
    ) -> UpdateEnvironmentVariablesResponse {
        let mut environment = self
            .environment
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut removed = 0_u32;
        if request.replace {
            let stale = environment
                .keys()
                .filter(|key| !request.env.contains_key(*key))
                .cloned()
                .collect::<Vec<_>>();
            removed = stale.len().min(u32::MAX as usize) as u32;
            for key in stale {
                environment.remove(&key);
            }
        }
        for (key, value) in &request.env {
            environment.insert(key.clone(), value.clone());
        }
        UpdateEnvironmentVariablesResponse {
            applied: request.env.len().min(u32::MAX as usize) as u32,
            removed,
        }
    }

    pub fn resolve_path(&self, requested: &str) -> Result<PathBuf, String> {
        let logical = if requested.is_empty() {
            "/workspace"
        } else {
            requested
        };
        if let Some(terminal_name) = logical.strip_prefix(BOX_TERMINAL_VIRTUAL_PREFIX) {
            let stem = terminal_name.strip_suffix(".txt").unwrap_or("");
            if stem.is_empty()
                || !stem.as_bytes().iter().all(u8::is_ascii_digit)
            {
                return Err(format!("Rejected terminal virtual path: {requested}"));
            }
            return Ok(self.terminals_directory.join(terminal_name).clean());
        }
        let mapped = if logical == "/workspace" {
            self.workspace_root.clone()
        } else if let Some(relative) = logical.strip_prefix("/workspace/") {
            self.workspace_root.join(relative)
        } else if Path::new(logical).is_absolute() {
            PathBuf::from(logical)
        } else {
            self.workspace_root.join(logical)
        }
        .clean();
        if path_within(&self.workspace_root, &mapped) {
            Ok(mapped)
        } else {
            Err(format!(
                "Path escapes configured workspace root: {requested}"
            ))
        }
    }

    fn assert_real_path_allowed(
        &self,
        target: &Path,
        requested: &str,
    ) -> Result<(), String> {
        if path_within(&self.workspace_root, target)
            || path_within(&self.terminals_directory, target)
        {
            Ok(())
        } else {
            Err(format!(
                "Resolved path escapes configured roots: {requested}"
            ))
        }
    }

    pub fn read(&self, args: &ReadArgs) -> ReadResult {
        let outcome = (|| -> Result<ReadSuccess, ReadFailure> {
            let target = self
                .resolve_path(&args.path)
                .map_err(ReadFailure::Rejected)?;
            let direct = fs::symlink_metadata(&target).map_err(|error| {
                ReadFailure::Io(error, args.path.clone())
            })?;
            if direct.file_type().is_symlink() {
                return Err(ReadFailure::Rejected(format!(
                    "Symbolic-link reads are not permitted: {}",
                    args.path
                )));
            }
            let canonical = fs::canonicalize(&target).map_err(|error| {
                ReadFailure::Io(error, args.path.clone())
            })?;
            self.assert_real_path_allowed(&canonical, &args.path)
                .map_err(ReadFailure::Rejected)?;
            let info = fs::metadata(&canonical)
                .map_err(|error| ReadFailure::Io(error, args.path.clone()))?;
            if !info.is_file() {
                return Err(ReadFailure::Invalid(
                    args.path.clone(),
                    "Path is not a regular file".into(),
                ));
            }
            let encoding = args.encoding_hint.as_deref().unwrap_or("utf8");
            if !matches!(encoding, "utf8" | "utf-8" | "latin1") {
                return Err(ReadFailure::Invalid(
                    args.path.clone(),
                    format!("Unsupported encoding hint: {encoding}"),
                ));
            }
            let data = fs::read(&canonical)
                .map_err(|error| ReadFailure::Io(error, args.path.clone()))?;
            let text = if encoding == "latin1" {
                data.iter().map(|byte| char::from(*byte)).collect::<String>()
            } else {
                String::from_utf8_lossy(&data).into_owned()
            };
            let lines = text.split('\n').collect::<Vec<_>>();
            let offset = args.offset.unwrap_or_default().max(0) as usize;
            let limit = args
                .limit
                .map(|value| value as usize)
                .unwrap_or(lines.len());
            let end = offset.saturating_add(limit).min(lines.len());
            let content = if offset >= lines.len() {
                String::new()
            } else {
                lines[offset..end].join("\n")
            };
            Ok(ReadSuccess {
                path: args.path.clone(),
                output: Some(read_success::Output::Content(content)),
                total_lines: lines.len().min(i32::MAX as usize) as i32,
                file_size: i64::try_from(data.len()).unwrap_or(i64::MAX),
                truncated: offset > 0 || offset.saturating_add(limit) < lines.len(),
                output_blob_id: None,
                range_applied: args.offset.is_some() || args.limit.is_some(),
            })
        })();

        match outcome {
            Ok(success) => ReadResult {
                result: Some(read_result::Result::Success(success)),
            },
            Err(ReadFailure::Rejected(reason)) => ReadResult {
                result: Some(read_result::Result::Rejected(ReadRejected {
                    path: args.path.clone(),
                    reason,
                })),
            },
            Err(ReadFailure::Invalid(path, reason)) => ReadResult {
                result: Some(read_result::Result::InvalidFile(ReadInvalidFile {
                    path,
                    reason,
                })),
            },
            Err(ReadFailure::Io(error, path)) => {
                use std::io::ErrorKind;
                let result = match error.kind() {
                    ErrorKind::NotFound => read_result::Result::FileNotFound(
                        ReadFileNotFound { path },
                    ),
                    ErrorKind::PermissionDenied => read_result::Result::PermissionDenied(
                        ReadPermissionDenied { path },
                    ),
                    ErrorKind::InvalidInput => read_result::Result::InvalidFile(
                        ReadInvalidFile {
                            path,
                            reason: error.to_string(),
                        },
                    ),
                    _ => read_result::Result::Error(ReadError {
                        path,
                        error: error.to_string(),
                    }),
                };
                ReadResult {
                    result: Some(result),
                }
            }
        }
    }

    pub fn shell(&self, args: &ShellArgs) -> ShellResult {
        let abort = AtomicBool::new(false);
        self.shell_with_abort(args, &abort)
    }

    fn shell_with_abort(
        &self,
        args: &ShellArgs,
        abort: &AtomicBool,
    ) -> ShellResult {
        let cwd = match self.resolve_path(&args.working_directory) {
            Ok(cwd) => cwd,
            Err(error) => {
                return ShellResult {
                    result: Some(shell_result::Result::SpawnError(ShellSpawnError {
                        command: args.command.clone(),
                        working_directory: args.working_directory.clone(),
                        error,
                    })),
                };
            }
        };
        let timeout = (args.timeout > 0).then_some(args.timeout as u64);
        let outcome = match self.run_process(&args.command, &cwd, timeout, abort) {
            Ok(outcome) => outcome,
            Err(error) => {
                return ShellResult {
                    result: Some(shell_result::Result::SpawnError(ShellSpawnError {
                        command: args.command.clone(),
                        working_directory: args.working_directory.clone(),
                        error,
                    })),
                };
            }
        };
        if outcome.timed_out {
            return ShellResult {
                result: Some(shell_result::Result::Timeout(ShellTimeout {
                    command: args.command.clone(),
                    working_directory: args.working_directory.clone(),
                    timeout_ms: args.timeout,
                })),
            };
        }
        let elapsed = saturating_i32(outcome.elapsed_ms);
        let interleaved = format!("{}{}", outcome.stdout, outcome.stderr);
        if outcome.code == 0 && !outcome.aborted {
            ShellResult {
                result: Some(shell_result::Result::Success(ShellSuccess {
                    command: args.command.clone(),
                    working_directory: args.working_directory.clone(),
                    exit_code: outcome.code,
                    signal: outcome.signal,
                    stdout: outcome.stdout,
                    stderr: outcome.stderr,
                    execution_time: elapsed,
                    interleaved_output: Some(interleaved),
                    local_execution_time_ms: Some(elapsed),
                })),
            }
        } else {
            ShellResult {
                result: Some(shell_result::Result::Failure(ShellFailure {
                    command: args.command.clone(),
                    working_directory: args.working_directory.clone(),
                    exit_code: outcome.code,
                    signal: outcome.signal,
                    stdout: outcome.stdout,
                    stderr: outcome.stderr,
                    execution_time: elapsed,
                    interleaved_output: Some(interleaved),
                    aborted: outcome.aborted,
                    local_execution_time_ms: Some(elapsed),
                })),
            }
        }
    }

    fn shell_stream_live<F>(
        &self,
        id: u32,
        exec_id: &str,
        args: &ShellArgs,
        abort: &AtomicBool,
        emit: &mut F,
    ) -> Result<(), String>
    where
        F: FnMut(ExecStreamElement) -> Result<(), String>,
    {
        let cwd = self.resolve_path(&args.working_directory)?;
        emit(client_message(
            id,
            exec_id,
            exec_client_message::Message::ShellStream(ShellStream {
                event: Some(shell_stream::Event::Start(ShellStreamStart {})),
            }),
            None,
        ))?;

        let mut child = self.spawn_shell(&args.command, &cwd)?;
        let pid = child.id();
        self.foreground_pids
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(pid);
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "shell stream stdout was unavailable".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "shell stream stderr was unavailable".to_string())?;
        let (events_tx, events_rx) = mpsc::channel::<ShellPipeEvent>();
        let stdout_worker = spawn_stream_pipe(
            stdout,
            events_tx.clone(),
            false,
        );
        let stderr_worker = spawn_stream_pipe(
            stderr,
            events_tx,
            true,
        );
        let started = Instant::now();
        let timeout_ms = (args.timeout > 0).then_some(args.timeout as u64);
        let mut kill_requested = false;
        let mut status = None;
        let mut stream_error = None::<String>;

        while status.is_none() {
            while let Ok(event) = events_rx.try_recv() {
                let stream = match event {
                    ShellPipeEvent::Stdout(bytes) => ShellStream {
                        event: Some(shell_stream::Event::Stdout(ShellStreamStdout {
                            data: String::from_utf8_lossy(&bytes).into_owned(),
                        })),
                    },
                    ShellPipeEvent::Stderr(bytes) => ShellStream {
                        event: Some(shell_stream::Event::Stderr(ShellStreamStderr {
                            data: String::from_utf8_lossy(&bytes).into_owned(),
                        })),
                    },
                };
                if let Err(error) = emit(client_message(
                    id,
                    exec_id,
                    exec_client_message::Message::ShellStream(stream),
                    None,
                )) {
                    abort.store(true, Ordering::Release);
                    stream_error = Some(error);
                    kill_process_group(pid);
                    kill_requested = true;
                    break;
                }
            }
            if stream_error.is_some() {
                status = child.wait().ok();
                break;
            }
            match child.try_wait().map_err(|error| error.to_string())? {
                Some(done) => {
                    status = Some(done);
                    break;
                }
                None => {}
            }
            if !kill_requested
                && (abort.load(Ordering::Acquire)
                    || self.stopping.load(Ordering::Acquire)
                    || timeout_ms.is_some_and(|timeout| {
                        started.elapsed() >= Duration::from_millis(timeout)
                    }))
            {
                kill_process_group(pid);
                kill_requested = true;
            }
            match events_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(event) => {
                    let stream = match event {
                        ShellPipeEvent::Stdout(bytes) => ShellStream {
                            event: Some(shell_stream::Event::Stdout(ShellStreamStdout {
                                data: String::from_utf8_lossy(&bytes).into_owned(),
                            })),
                        },
                        ShellPipeEvent::Stderr(bytes) => ShellStream {
                            event: Some(shell_stream::Event::Stderr(ShellStreamStderr {
                                data: String::from_utf8_lossy(&bytes).into_owned(),
                            })),
                        },
                    };
                    if let Err(error) = emit(client_message(
                        id,
                        exec_id,
                        exec_client_message::Message::ShellStream(stream),
                        None,
                    )) {
                        abort.store(true, Ordering::Release);
                        stream_error = Some(error);
                        kill_process_group(pid);
                        kill_requested = true;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {}
            }
        }

        let _ = stdout_worker.join();
        let _ = stderr_worker.join();
        while let Ok(event) = events_rx.try_recv() {
            if stream_error.is_some() {
                break;
            }
            let stream = match event {
                ShellPipeEvent::Stdout(bytes) => ShellStream {
                    event: Some(shell_stream::Event::Stdout(ShellStreamStdout {
                        data: String::from_utf8_lossy(&bytes).into_owned(),
                    })),
                },
                ShellPipeEvent::Stderr(bytes) => ShellStream {
                    event: Some(shell_stream::Event::Stderr(ShellStreamStderr {
                        data: String::from_utf8_lossy(&bytes).into_owned(),
                    })),
                },
            };
            if let Err(error) = emit(client_message(
                id,
                exec_id,
                exec_client_message::Message::ShellStream(stream),
                None,
            )) {
                abort.store(true, Ordering::Release);
                stream_error = Some(error);
            }
        }
        self.foreground_pids
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&pid);

        if let Some(error) = stream_error {
            return Err(error);
        }
        let status = match status {
            Some(status) => status,
            None => child.wait().map_err(|error| error.to_string())?,
        };
        emit(client_message(
            id,
            exec_id,
            exec_client_message::Message::ShellStream(ShellStream {
                event: Some(shell_stream::Event::Exit(ShellStreamExit {
                    code: status.code().unwrap_or(1).max(0) as u32,
                    cwd: args.working_directory.clone(),
                    aborted: abort.load(Ordering::Acquire),
                    local_execution_time_ms: Some(saturating_i32(
                        started.elapsed().as_millis().min(u64::MAX as u128) as u64,
                    )),
                })),
            }),
            None,
        ))
    }

    pub fn spawn_background(
        &self,
        args: &BackgroundShellSpawnArgs,
    ) -> BackgroundShellSpawnResult {
        let result = (|| -> Result<BackgroundShellSpawnSuccess, String> {
            let cwd = self.resolve_path(&args.working_directory)?;
            fs::create_dir_all(&self.terminals_directory)
                .map_err(|error| error.to_string())?;
            let shell_id = self.next_shell_id.fetch_add(1, Ordering::Relaxed);
            let terminal_path = self
                .terminals_directory
                .join(format!("{shell_id}.txt"));
            let started_at = Utc::now();
            let started_ms = started_at.timestamp_millis();
            let mut child = self.spawn_shell(&args.command, &cwd)?;
            let pid = child.id();
            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| "background shell stdin was unavailable".to_string())?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| "background shell stdout was unavailable".to_string())?;
            let stderr = child
                .stderr
                .take()
                .ok_or_else(|| "background shell stderr was unavailable".to_string())?;
            fs::write(
                &terminal_path,
                terminal_frontmatter(args, Some(pid), started_at),
            )
            .map_err(|error| error.to_string())?;
            let writer = Arc::new(Mutex::new(
                OpenOptions::new()
                    .append(true)
                    .open(&terminal_path)
                    .map_err(|error| error.to_string())?,
            ));
            let stdout_worker = spawn_output_copy(stdout, Arc::clone(&writer));
            let stderr_worker = spawn_output_copy(stderr, Arc::clone(&writer));
            self.background
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(
                    shell_id,
                    BackgroundProcess {
                        stdin: Arc::new(Mutex::new(stdin)),
                        terminal_path: terminal_path.clone(),
                        pid,
                    },
                );
            let processes = Arc::clone(&self.background);
            let footer_writer = Arc::clone(&writer);
            thread::spawn(move || {
                let status = child.wait();
                let _ = stdout_worker.join();
                let _ = stderr_worker.join();
                let code = status
                    .ok()
                    .and_then(|status| status.code())
                    .unwrap_or(1);
                if let Ok(mut writer) = footer_writer.lock() {
                    let _ = writer.write_all(
                        terminal_footer(code, started_ms).as_bytes(),
                    );
                    let _ = writer.flush();
                }
                if let Ok(mut processes) = processes.lock() {
                    processes.remove(&shell_id);
                }
            });
            Ok(BackgroundShellSpawnSuccess {
                shell_id,
                command: args.command.clone(),
                working_directory: args.working_directory.clone(),
                pid: Some(pid),
            })
        })();
        match result {
            Ok(success) => BackgroundShellSpawnResult {
                result: Some(background_shell_spawn_result::Result::Success(success)),
            },
            Err(error) => BackgroundShellSpawnResult {
                result: Some(background_shell_spawn_result::Result::Error(
                    BackgroundShellSpawnError {
                        command: args.command.clone(),
                        working_directory: args.working_directory.clone(),
                        error,
                    },
                )),
            },
        }
    }

    pub fn write_stdin(&self, args: &WriteShellStdinArgs) -> WriteShellStdinResult {
        let mut processes = self
            .background
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(process) = processes.get_mut(&args.shell_id) else {
            return WriteShellStdinResult {
                result: Some(write_shell_stdin_result::Result::Error(
                    WriteShellStdinError {
                        error: format!("Shell {} is not running", args.shell_id),
                    },
                )),
            };
        };
        let before = fs::metadata(&process.terminal_path)
            .map(|metadata| metadata.len().min(u32::MAX as u64) as u32)
            .unwrap_or_default();
        let write_result = process
            .stdin
            .lock()
            .map_err(|_| "background shell stdin is poisoned".to_string())
            .and_then(|mut stdin| {
                stdin
                    .write_all(args.chars.as_bytes())
                    .and_then(|_| stdin.flush())
                    .map_err(|error| error.to_string())
            });
        match write_result {
            Ok(()) => WriteShellStdinResult {
                result: Some(write_shell_stdin_result::Result::Success(
                    WriteShellStdinSuccess {
                        shell_id: args.shell_id,
                        terminal_file_length_before_input_written: before,
                    },
                )),
            },
            Err(error) => WriteShellStdinResult {
                result: Some(write_shell_stdin_result::Result::Error(
                    WriteShellStdinError { error },
                )),
            },
        }
    }

    pub fn execute_with_emitter<F>(
        &self,
        request: ExecServerMessage,
        abort: &AtomicBool,
        mut emit: F,
    ) -> Result<(), String>
    where
        F: FnMut(ExecStreamElement) -> Result<(), String>,
    {
        let id = request.id;
        let exec_id = request.exec_id.clone();
        match request.message {
            Some(exec_server_message::Message::ReadArgs(args)) => {
                emit(client_message(
                    id,
                    &exec_id,
                    exec_client_message::Message::ReadResult(self.read(&args)),
                    None,
                ))?;
            }
            Some(exec_server_message::Message::RedactedReadArgs(args)) => {
                emit(client_message(
                    id,
                    &exec_id,
                    exec_client_message::Message::RedactedReadResult(self.read(&args)),
                    None,
                ))?;
            }
            Some(exec_server_message::Message::ShellArgs(args)) => {
                let started = Instant::now();
                let result = self.shell_with_abort(&args, abort);
                emit(client_message(
                    id,
                    &exec_id,
                    exec_client_message::Message::ShellResult(result),
                    Some(saturating_i32(started.elapsed().as_millis() as u64)),
                ))?;
            }
            Some(exec_server_message::Message::MiniSweAgentBashArgs(args)) => {
                let started = Instant::now();
                let result = self.shell_with_abort(&args, abort);
                emit(client_message(
                    id,
                    &exec_id,
                    exec_client_message::Message::MiniSweAgentBashResult(result),
                    Some(saturating_i32(started.elapsed().as_millis() as u64)),
                ))?;
            }
            Some(exec_server_message::Message::ShellStreamArgs(args)) => {
                if let Err(error) =
                    self.shell_stream_live(id, &exec_id, &args, abort, &mut emit)
                {
                    if abort.load(Ordering::Acquire) {
                        return Err(error);
                    }
                    emit(thrown(id, error, "BOX_EXEC_DAEMON_ERROR"))?;
                }
            }
            Some(exec_server_message::Message::BackgroundShellSpawnArgs(args)) => {
                emit(client_message(
                    id,
                    &exec_id,
                    exec_client_message::Message::BackgroundShellSpawnResult(
                        self.spawn_background(&args),
                    ),
                    None,
                ))?;
            }
            Some(exec_server_message::Message::WriteShellStdinArgs(args)) => {
                emit(client_message(
                    id,
                    &exec_id,
                    exec_client_message::Message::WriteShellStdinResult(
                        self.write_stdin(&args),
                    ),
                    None,
                ))?;
            }
            None => emit(thrown(
                id,
                "Unsupported ExecServerMessage case: unset",
                "BOX_EXEC_UNSUPPORTED",
            ))?,
        }
        emit(close(id))
    }

    pub fn execute(&self, request: ExecServerMessage) -> Vec<ExecStreamElement> {
        let abort = AtomicBool::new(false);
        let mut output = Vec::new();
        let _ = self.execute_with_emitter(request, &abort, |element| {
            output.push(element);
            Ok(())
        });
        output
    }

    pub fn stop(&self) {
        self.stopping.store(true, Ordering::Release);
        let foreground = self
            .foreground_pids
            .lock()
            .map(|pids| pids.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        for pid in foreground {
            kill_process_group(pid);
        }
        let background = self
            .background
            .lock()
            .map(|processes| {
                processes.values().map(|process| process.pid).collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for pid in background {
            kill_process_group(pid);
        }
    }

    fn spawn_shell(&self, command: &str, cwd: &Path) -> Result<Child, String> {
        let environment = self
            .environment
            .lock()
            .map(|environment| environment.clone())
            .map_err(|_| "box exec environment is poisoned".to_string())?;
        let mut process = Command::new("/bin/sh");
        process
            .arg("-lc")
            .arg(command)
            .current_dir(cwd)
            .env_clear()
            .envs(environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            process.process_group(0);
        }
        process.spawn().map_err(|error| error.to_string())
    }

    fn run_process(
        &self,
        command: &str,
        cwd: &Path,
        timeout_ms: Option<u64>,
        abort: &AtomicBool,
    ) -> Result<ProcessOutcome, String> {
        let mut child = self.spawn_shell(command, cwd)?;
        let pid = child.id();
        self.foreground_pids
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(pid);
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "shell stdout was unavailable".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "shell stderr was unavailable".to_string())?;
        let stdout_worker = thread::spawn(move || read_all(stdout));
        let stderr_worker = thread::spawn(move || read_all(stderr));
        let started = Instant::now();
        let mut timed_out = false;
        let mut kill_requested = false;
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
                break status;
            }
            if !kill_requested
                && timeout_ms.is_some_and(|timeout| {
                    started.elapsed() >= Duration::from_millis(timeout)
                })
            {
                timed_out = true;
                kill_process_group(pid);
                kill_requested = true;
            }
            if !kill_requested
                && (abort.load(Ordering::Acquire)
                    || self.stopping.load(Ordering::Acquire))
            {
                kill_process_group(pid);
                kill_requested = true;
            }
            thread::sleep(Duration::from_millis(10));
        };
        self.foreground_pids
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&pid);
        let stdout = stdout_worker
            .join()
            .unwrap_or_else(|_| Vec::new());
        let stderr = stderr_worker
            .join()
            .unwrap_or_else(|_| Vec::new());
        let signal = status_signal(&status);
        Ok(ProcessOutcome {
            code: status.code().unwrap_or(1),
            signal,
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            elapsed_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            timed_out,
            aborted: abort.load(Ordering::Acquire)
                || self.stopping.load(Ordering::Acquire),
        })
    }

}

enum ReadFailure {
    Rejected(String),
    Invalid(String, String),
    Io(std::io::Error, String),
}

pub struct BoxExecDaemonHandle {
    pub host: String,
    pub port: u16,
    pub url: String,
    pub workspace_root: PathBuf,
    pub terminals_directory: PathBuf,
    ready: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    runtime: Arc<BoxExecRuntime>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl BoxExecDaemonHandle {
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire) && !self.stop.load(Ordering::Acquire)
    }

    pub fn stop(&self) -> Result<(), String> {
        if self.stop.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        self.ready.store(false, Ordering::Release);
        self.runtime.stop();
        if let Some(worker) = self
            .worker
            .lock()
            .map_err(|_| "box exec daemon worker lock poisoned".to_string())?
            .take()
        {
            worker
                .join()
                .map_err(|_| "box exec daemon accept worker panicked".to_string())?;
        }
        Ok(())
    }
}

pub fn start_box_exec_daemon(
    options: BoxExecDaemonOptions,
) -> Result<BoxExecDaemonHandle, String> {
    let host = options
        .host
        .unwrap_or_else(|| BOX_EXEC_DAEMON_HOST.to_string());
    let port = options.port.unwrap_or(BOX_EXEC_DAEMON_PORT);
    let auth_token = options
        .auth_token
        .unwrap_or_else(|| BOX_EXEC_DAEMON_AUTH_TOKEN.to_string());
    let requested_workspace_root = options.workspace_root.clean();
    let requested_terminals_directory = options
        .terminals_directory
        .unwrap_or_else(|| std::env::temp_dir().join("sand-box-terminals"))
        .clean();
    fs::create_dir_all(&requested_workspace_root).map_err(|error| error.to_string())?;
    fs::create_dir_all(&requested_terminals_directory)
        .map_err(|error| error.to_string())?;
    let workspace_root =
        fs::canonicalize(&requested_workspace_root).map_err(|error| error.to_string())?;
    let terminals_directory =
        fs::canonicalize(&requested_terminals_directory).map_err(|error| error.to_string())?;
    if !fs::metadata(&workspace_root)
        .map_err(|error| error.to_string())?
        .is_dir()
    {
        return Err(format!(
            "workspaceRoot is not a directory: {}",
            workspace_root.display()
        ));
    }
    let listener =
        TcpListener::bind((host.as_str(), port)).map_err(|error| error.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    let environment = options
        .environment
        .unwrap_or_else(|| std::env::vars().collect());
    let runtime = Arc::new(BoxExecRuntime::new(
        workspace_root.clone(),
        terminals_directory.clone(),
        environment,
    ));
    let ready = Arc::new(AtomicBool::new(true));
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let worker_runtime = Arc::clone(&runtime);
    let worker_auth = auth_token;
    let worker = thread::Builder::new()
        .name("box-exec-daemon-accept".into())
        .spawn(move || {
            while !worker_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let runtime = Arc::clone(&worker_runtime);
                        let auth_token = worker_auth.clone();
                        let _ = thread::Builder::new()
                            .name("box-exec-daemon-request".into())
                            .spawn(move || {
                                let _ = serve_connection(stream, runtime, &auth_token);
                            });
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            }
        })
        .map_err(|error| error.to_string())?;

    Ok(BoxExecDaemonHandle {
        host: host.clone(),
        port: address.port(),
        url: format!("http://{host}:{}", address.port()),
        workspace_root,
        terminals_directory,
        ready,
        stop,
        runtime,
        worker: Mutex::new(Some(worker)),
    })
}

fn serve_connection(
    mut stream: TcpStream,
    runtime: Arc<BoxExecRuntime>,
    auth_token: &str,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let request = read_http_request(&mut stream)?;
    let response = if request.method != "POST" {
        write_http_response(
            &mut stream,
            405,
            "text/plain; charset=utf-8",
            b"Method Not Allowed",
            &[],
        )
    } else if request
        .headers
        .get("authorization")
        .map(String::as_str)
        != Some(format!("Bearer {auth_token}").as_str())
    {
        write_http_response(
            &mut stream,
            401,
            "text/plain; charset=utf-8",
            b"Unauthorized",
            &[],
        )
    } else {
        match request.path.as_str() {
        "/agent.v1.ControlService/Ping" => {
            let _ = PingRequest::decode(request.body.as_slice())
                .map_err(|error| error.to_string())?;
            write_proto_response(&mut stream, &PingResponse {})
        }
        "/agent.v1.ControlService/GetCapabilities" => {
            let _ = GetCapabilitiesRequest::decode(request.body.as_slice())
                .map_err(|error| error.to_string())?;
            write_proto_response(
                &mut stream,
                &GetCapabilitiesResponse {
                    computer_use_supported: Some(false),
                    install_plugin_artifact_supported: Some(false),
                },
            )
        }
        "/agent.v1.ControlService/UpdateEnvironmentVariables" => {
            let request =
                UpdateEnvironmentVariablesRequest::decode(request.body.as_slice())
                    .map_err(|error| error.to_string())?;
            write_proto_response(
                &mut stream,
                &runtime.apply_environment(&request),
            )
        }
        "/agent.v1.ControlService/LoadMcpServers" => {
            let _ = LoadMcpServersRequest::decode(request.body.as_slice())
                .map_err(|error| error.to_string())?;
            write_proto_response(
                &mut stream,
                &LoadMcpServersResponse {
                    loaded_server_names: Vec::new(),
                },
            )
        }
        "/agent.v1.ExecService/Exec" => {
            let request = ExecServerMessage::decode(request.body.as_slice())
                .map_err(|error| error.to_string())?;
            write_connect_stream(&mut stream, runtime, request)
        }
            _ => write_http_response(
                &mut stream,
                404,
                "text/plain; charset=utf-8",
                b"Not Found",
                &[],
            ),
        }
    };
    if response.is_ok() {
        // HTTP/1.1 responses advertise Connection: close. Make the successful
        // write-half close explicit instead of relying on TcpStream::drop so
        // Darwin clients receive a FIN after the final response bytes rather
        // than an intermittent ECONNRESET while reading the completed reply.
        let _ = stream.shutdown(Shutdown::Write);
    }
    response
}

struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut data = Vec::with_capacity(4096);
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 {
            return Err("connection closed before HTTP headers".into());
        }
        data.extend_from_slice(&chunk[..count]);
        if let Some(index) = data.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
        if data.len() > 64 * 1024 {
            return Err("HTTP headers exceed 64KiB".into());
        }
    };
    let headers_text =
        std::str::from_utf8(&data[..header_end]).map_err(|error| error.to_string())?;
    let mut lines = headers_text.split("\r\n");
    let mut request_line = lines
        .next()
        .ok_or_else(|| "missing HTTP request line".to_string())?
        .split_whitespace();
    let method = request_line.next().unwrap_or_default().to_string();
    let path = request_line.next().unwrap_or_default().to_string();
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(
                name.trim().to_ascii_lowercase(),
                value.trim().to_string(),
            );
        }
    }
    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_default();
    let expected = header_end.saturating_add(content_length);
    while data.len() < expected {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        data.extend_from_slice(&chunk[..count]);
    }
    if data.len() < expected {
        return Err("HTTP request body ended early".into());
    }
    Ok(HttpRequest {
        method,
        path,
        headers,
        body: data[header_end..expected].to_vec(),
    })
}

fn write_proto_response<M: ProstMessage>(
    stream: &mut TcpStream,
    message: &M,
) -> Result<(), String> {
    let body = message.encode_to_vec();
    write_http_response(
        stream,
        200,
        "application/proto",
        &body,
        &[("Connect-Protocol-Version", "1")],
    )
}

fn write_connect_stream(
    stream: &mut TcpStream,
    runtime: Arc<BoxExecRuntime>,
    request: ExecServerMessage,
) -> Result<(), String> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/connect+proto\r\nTransfer-Encoding: chunked\r\nConnection: close\r\nConnect-Protocol-Version: 1\r\n\r\n"
    )
    .map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())?;

    let abort = Arc::new(AtomicBool::new(false));
    let monitor_done = Arc::new(AtomicBool::new(false));
    let monitor = spawn_disconnect_monitor(
        stream,
        Arc::clone(&abort),
        Arc::clone(&monitor_done),
    );
    let emitter_abort = Arc::clone(&abort);
    let execution = runtime.execute_with_emitter(
        request,
        abort.as_ref(),
        |message| {
            let payload = message.encode_to_vec();
            if let Err(error) = write_connect_chunk(stream, 0, &payload) {
                emitter_abort.store(true, Ordering::Release);
                return Err(error);
            }
            Ok(())
        },
    );

    let finish = if execution.is_ok() && !abort.load(Ordering::Acquire) {
        write_connect_chunk(stream, 0x02, b"{}")
            .and_then(|()| {
                stream
                    .write_all(b"0\r\n\r\n")
                    .map_err(|error| error.to_string())
            })
            .and_then(|()| stream.flush().map_err(|error| error.to_string()))
    } else {
        execution
    };
    monitor_done.store(true, Ordering::Release);
    if let Some(monitor) = monitor {
        let _ = monitor.join();
    }
    finish
}

fn write_connect_chunk(
    stream: &mut TcpStream,
    flags: u8,
    payload: &[u8],
) -> Result<(), String> {
    let mut envelope = Vec::with_capacity(payload.len().saturating_add(5));
    push_connect_envelope(flags, payload, &mut envelope);
    write!(stream, "{:X}\r\n", envelope.len())
        .map_err(|error| error.to_string())?;
    stream
        .write_all(&envelope)
        .map_err(|error| error.to_string())?;
    stream
        .write_all(b"\r\n")
        .map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn spawn_disconnect_monitor(
    stream: &TcpStream,
    abort: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
) -> Option<JoinHandle<()>> {
    let monitor = stream.try_clone().ok()?;
    let _ = monitor.set_read_timeout(Some(Duration::from_millis(50)));
    thread::Builder::new()
        .name("box-exec-daemon-abort-watch".into())
        .spawn(move || {
            let mut byte = [0_u8; 1];
            while !done.load(Ordering::Acquire) {
                match monitor.peek(&mut byte) {
                    Ok(0) => {
                        abort.store(true, Ordering::Release);
                        break;
                    }
                    Ok(_) => {}
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock
                                | std::io::ErrorKind::TimedOut
                                | std::io::ErrorKind::Interrupted
                        ) => {}
                    Err(_) => {
                        abort.store(true, Ordering::Release);
                        break;
                    }
                }
            }
        })
        .ok()
}

fn push_connect_envelope(flags: u8, payload: &[u8], output: &mut Vec<u8>) {
    output.push(flags);
    output.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    output.extend_from_slice(payload);
}

fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    extra_headers: &[(&str, &str)],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Response",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )
    .map_err(|error| error.to_string())?;
    for (name, value) in extra_headers {
        write!(stream, "{name}: {value}\r\n").map_err(|error| error.to_string())?;
    }
    write!(stream, "\r\n").map_err(|error| error.to_string())?;
    stream.write_all(body).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn client_message(
    id: u32,
    exec_id: &str,
    message: exec_client_message::Message,
    elapsed_ms: Option<i32>,
) -> ExecStreamElement {
    ExecStreamElement {
        element: Some(exec_stream_element::Element::ExecClientMessage(
            ExecClientMessage {
                id,
                exec_id: exec_id.to_string(),
                local_execution_time_ms: elapsed_ms,
                message: Some(message),
            },
        )),
    }
}

fn close(id: u32) -> ExecStreamElement {
    ExecStreamElement {
        element: Some(exec_stream_element::Element::ExecClientControlMessage(
            ExecClientControlMessage {
                message: Some(exec_client_control_message::Message::StreamClose(
                    ExecClientStreamClose { id },
                )),
            },
        )),
    }
}

fn thrown(id: u32, error: impl Into<String>, error_code: &str) -> ExecStreamElement {
    ExecStreamElement {
        element: Some(exec_stream_element::Element::ExecClientControlMessage(
            ExecClientControlMessage {
                message: Some(exec_client_control_message::Message::Throw(
                    ExecClientThrow {
                        id,
                        error: error.into(),
                        stack_trace: None,
                        error_code: Some(error_code.to_string()),
                    },
                )),
            },
        )),
    }
}

fn terminal_frontmatter(
    args: &BackgroundShellSpawnArgs,
    pid: Option<u32>,
    started_at: chrono::DateTime<Utc>,
) -> String {
    let pid_line = pid
        .map(|pid| format!("pid: {pid}\n"))
        .unwrap_or_default();
    format!(
        "---\n{pid_line}cwd: {}\ncommand: {}\nstatus: running\nstarted_at: {}\nrunning_for_ms: 0\n---\n",
        serde_json::to_string(&args.working_directory).unwrap_or_else(|_| "\"\"".into()),
        serde_json::to_string(&args.command).unwrap_or_else(|_| "\"\"".into()),
        started_at.to_rfc3339_opts(SecondsFormat::Millis, true),
    )
}

fn terminal_footer(exit_code: i32, started_at_ms: i64) -> String {
    let ended = Utc::now();
    format!(
        "\n---\nexit_code: {exit_code}\nelapsed_ms: {}\nended_at: {}\n---\n",
        ended.timestamp_millis().saturating_sub(started_at_ms),
        ended.to_rfc3339_opts(SecondsFormat::Millis, true),
    )
}

fn spawn_output_copy<R>(
    mut reader: R,
    writer: Arc<Mutex<File>>,
) -> JoinHandle<()>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        loop {
            let Ok(count) = reader.read(&mut buffer) else {
                break;
            };
            if count == 0 {
                break;
            }
            if let Ok(mut writer) = writer.lock() {
                let _ = writer.write_all(&buffer[..count]);
                let _ = writer.flush();
            }
        }
    })
}

fn read_all<R: Read>(mut reader: R) -> Vec<u8> {
    let mut output = Vec::new();
    let _ = reader.read_to_end(&mut output);
    output
}

fn spawn_stream_pipe<R>(
    mut reader: R,
    sender: mpsc::Sender<ShellPipeEvent>,
    stderr: bool,
) -> JoinHandle<()>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    let bytes = buffer[..count].to_vec();
                    let event = if stderr {
                        ShellPipeEvent::Stderr(bytes)
                    } else {
                        ShellPipeEvent::Stdout(bytes)
                    };
                    if sender.send(event).is_err() {
                        break;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    })
}

fn path_within(root: &Path, target: &Path) -> bool {
    target == root || target.strip_prefix(root).is_ok()
}

fn saturating_i32(value: u64) -> i32 {
    value.min(i32::MAX as u64) as i32
}

#[cfg(unix)]
fn kill_process_group(pid: u32) {
    unsafe {
        libc::kill(-(pid as i32), libc::SIGTERM);
    }
}

#[cfg(not(unix))]
fn kill_process_group(_pid: u32) {}

#[cfg(unix)]
fn status_signal(status: &std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    match status.signal() {
        Some(libc::SIGTERM) => "SIGTERM".into(),
        Some(libc::SIGKILL) => "SIGKILL".into(),
        Some(signal) => format!("SIG{signal}"),
        None => String::new(),
    }
}

#[cfg(not(unix))]
fn status_signal(_status: &std::process::ExitStatus) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn runtime() -> (tempfile::TempDir, BoxExecRuntime) {
        let root = tempfile::tempdir().expect("root");
        let workspace = root.path().join("workspace");
        let terminals = root.path().join("terminals");
        fs::create_dir_all(&workspace).expect("workspace");
        fs::create_dir_all(&terminals).expect("terminals");
        let runtime = BoxExecRuntime::new(
            fs::canonicalize(&workspace).expect("workspace realpath"),
            fs::canonicalize(&terminals).expect("terminals realpath"),
            std::env::vars().collect(),
        );
        (root, runtime)
    }

    #[test]
    fn path_mapping_is_workspace_scoped_and_terminal_virtual_names_are_strict() {
        let (_root, runtime) = runtime();
        assert_eq!(
            runtime.resolve_path("/workspace").expect("workspace"),
            runtime.workspace_root
        );
        assert_eq!(
            runtime.resolve_path("nested/file.txt").expect("relative"),
            runtime.workspace_root.join("nested/file.txt")
        );
        assert!(runtime.resolve_path("/workspace/../escape").is_err());
        assert!(runtime.resolve_path("/tmp/escape").is_err());
        assert_eq!(
            runtime
                .resolve_path(
                    "/root/.cursor/projects/workspace/terminals/12.txt"
                )
                .expect("terminal"),
            runtime.terminals_directory.join("12.txt")
        );
        assert!(
            runtime
                .resolve_path(
                    "/root/.cursor/projects/workspace/terminals/../../x"
                )
                .is_err()
        );
    }

    #[test]
    fn read_matches_range_encoding_and_symlink_rejection_contract() {
        let (root, runtime) = runtime();
        fs::write(
            runtime.workspace_root.join("sample.txt"),
            b"zero\none\ntwo\nthree",
        )
        .expect("sample");
        let result = runtime.read(&ReadArgs {
            path: "/workspace/sample.txt".into(),
            tool_call_id: String::new(),
            offset: Some(1),
            limit: Some(2),
            encoding_hint: Some("utf8".into()),
        });
        let Some(read_result::Result::Success(success)) = result.result else {
            panic!("expected read success");
        };
        assert_eq!(
            success.output,
            Some(read_success::Output::Content("one\ntwo".into()))
        );
        assert_eq!(success.total_lines, 4);
        assert!(success.truncated);
        assert!(success.range_applied);

        let outside = root.path().join("outside.txt");
        fs::write(&outside, b"secret").expect("outside");
        symlink(&outside, runtime.workspace_root.join("link.txt")).expect("link");
        let result = runtime.read(&ReadArgs {
            path: "/workspace/link.txt".into(),
            tool_call_id: String::new(),
            offset: None,
            limit: None,
            encoding_hint: None,
        });
        assert!(matches!(
            result.result,
            Some(read_result::Result::Rejected(_))
        ));
    }

    #[test]
    fn shell_environment_and_background_stdin_match_daemon_contract() {
        let (_root, runtime) = runtime();
        runtime.apply_environment(&UpdateEnvironmentVariablesRequest {
            env: HashMap::from([("BOX_EXEC_TEST".into(), "works".into())]),
            replace: false,
        });
        let result = runtime.shell(&ShellArgs {
            command: "printf '%s' \"$BOX_EXEC_TEST\"".into(),
            working_directory: "/workspace".into(),
            timeout: 5_000,
            tool_call_id: String::new(),
        });
        let Some(shell_result::Result::Success(success)) = result.result else {
            panic!("expected shell success");
        };
        assert_eq!(success.stdout, "works");

        let spawned = runtime.spawn_background(&BackgroundShellSpawnArgs {
            command: "read line; printf 'got:%s\\n' \"$line\"".into(),
            working_directory: "/workspace".into(),
            tool_call_id: String::new(),
        });
        let Some(background_shell_spawn_result::Result::Success(spawned)) =
            spawned.result
        else {
            panic!("expected background shell");
        };
        let stdin = runtime.write_stdin(&WriteShellStdinArgs {
            shell_id: spawned.shell_id,
            chars: "hello\n".into(),
        });
        assert!(matches!(
            stdin.result,
            Some(write_shell_stdin_result::Result::Success(_))
        ));
        let terminal = runtime
            .terminals_directory
            .join(format!("{}.txt", spawned.shell_id));
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let content = fs::read_to_string(&terminal).unwrap_or_default();
            if content.contains("got:hello") && content.contains("ended_at:") {
                break;
            }
            assert!(Instant::now() < deadline, "background terminal did not settle");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn shell_stream_emits_stdout_before_child_finishes() {
        let (_root, runtime) = runtime();
        let done_path = runtime.workspace_root.join("stream-done");
        let request = ExecServerMessage {
            id: 42,
            exec_id: "stream-live".into(),
            message: Some(exec_server_message::Message::ShellStreamArgs(ShellArgs {
                command: "printf first; sleep 0.4; touch stream-done; printf second".into(),
                working_directory: "/workspace".into(),
                timeout: 5_000,
                tool_call_id: "stream-tool".into(),
            })),
        };
        let abort = AtomicBool::new(false);
        let mut saw_first_before_done = false;
        let mut saw_exit = false;
        runtime
            .execute_with_emitter(request, &abort, |element| {
                if let Some(exec_stream_element::Element::ExecClientMessage(message)) =
                    element.element
                {
                    if let Some(exec_client_message::Message::ShellStream(stream)) =
                        message.message
                    {
                        match stream.event {
                            Some(shell_stream::Event::Stdout(stdout))
                                if stdout.data.contains("first") =>
                            {
                                saw_first_before_done = !done_path.exists();
                            }
                            Some(shell_stream::Event::Exit(_)) => saw_exit = true,
                            _ => {}
                        }
                    }
                }
                Ok(())
            })
            .expect("stream execution");
        assert!(saw_first_before_done, "stdout was buffered until after child completion");
        assert!(saw_exit);
    }

    #[cfg(unix)]
    #[test]
    fn disconnect_aborts_foreground_shell_process_group() {
        let root = tempfile::tempdir().expect("root");
        let workspace = root.path().join("workspace");
        fs::create_dir_all(&workspace).expect("workspace");
        let handle = start_box_exec_daemon(BoxExecDaemonOptions {
            host: Some("127.0.0.1".into()),
            port: Some(0),
            auth_token: Some("secret".into()),
            workspace_root: workspace.clone(),
            terminals_directory: Some(root.path().join("terminals")),
            environment: Some(std::env::vars().collect()),
        })
        .expect("daemon");
        let request = ExecServerMessage {
            id: 77,
            exec_id: "abort-wire".into(),
            message: Some(exec_server_message::Message::ShellArgs(ShellArgs {
                command: "echo $ > abort-pid.txt; sleep 10".into(),
                working_directory: "/workspace".into(),
                timeout: 0,
                tool_call_id: "abort-tool".into(),
            })),
        };
        let body = request.encode_to_vec();
        let mut stream = TcpStream::connect(("127.0.0.1", handle.port)).expect("connect");
        write!(
            stream,
            "POST /agent.v1.ExecService/Exec HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer secret\r\nContent-Type: application/proto\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .expect("headers");
        stream.write_all(&body).expect("body");
        stream.flush().expect("flush");

        let pid_path = workspace.join("abort-pid.txt");
        let deadline = Instant::now() + Duration::from_secs(2);
        while !pid_path.exists() {
            assert!(Instant::now() < deadline, "foreground shell never started");
            thread::sleep(Duration::from_millis(10));
        }
        let pid = fs::read_to_string(&pid_path)
            .expect("pid")
            .trim()
            .to_string();
        drop(stream);

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut exited = false;
        while Instant::now() < deadline {
            let alive = Command::new("kill")
                .args(["-0", &pid])
                .status()
                .map(|status| status.success())
                .unwrap_or(false);
            if !alive {
                exited = true;
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        if !exited {
            let _ = handle.stop();
        }
        assert!(exited, "client disconnect did not abort foreground process group");
        handle.stop().expect("stop");
    }

    #[test]
    fn authenticated_connect_routes_ping_and_exec_read_with_frozen_wire_tags() {
        let root = tempfile::tempdir().expect("root");
        let workspace = root.path().join("workspace");
        fs::create_dir_all(&workspace).expect("workspace");
        fs::write(workspace.join("wire.txt"), b"wire").expect("wire file");
        let handle = start_box_exec_daemon(BoxExecDaemonOptions {
            host: Some("127.0.0.1".into()),
            port: Some(0),
            auth_token: Some("secret".into()),
            workspace_root: workspace,
            terminals_directory: Some(root.path().join("terminals")),
            environment: Some(std::env::vars().collect()),
        })
        .expect("daemon");

        let ping = http_call(
            handle.port,
            "/agent.v1.ControlService/Ping",
            "secret",
            &PingRequest {}.encode_to_vec(),
        );
        assert!(ping.starts_with(b"HTTP/1.1 200 OK\r\n"));

        let request = ExecServerMessage {
            id: 7,
            exec_id: "wire-exec".into(),
            message: Some(exec_server_message::Message::ReadArgs(ReadArgs {
                path: "/workspace/wire.txt".into(),
                tool_call_id: "tool".into(),
                offset: None,
                limit: None,
                encoding_hint: None,
            })),
        };
        let response = http_call(
            handle.port,
            "/agent.v1.ExecService/Exec",
            "secret",
            &request.encode_to_vec(),
        );
        assert!(
            std::str::from_utf8(&response)
                .ok()
                .is_some_and(|value| value.to_ascii_lowercase().contains("transfer-encoding: chunked")),
            "Exec response must be a live chunked Connect stream"
        );
        let body = http_body(&response);
        assert!(body.len() > 5);
        assert_eq!(body[0], 0);
        let length =
            u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
        let element =
            ExecStreamElement::decode(&body[5..5 + length]).expect("stream element");
        let Some(exec_stream_element::Element::ExecClientMessage(message)) =
            element.element
        else {
            panic!("expected client message");
        };
        assert_eq!(message.id, 7);
        assert_eq!(message.exec_id, "wire-exec");
        let Some(exec_client_message::Message::ReadResult(read)) =
            message.message
        else {
            panic!("expected read result");
        };
        let Some(read_result::Result::Success(read)) = read.result else {
            panic!("expected read success");
        };
        assert_eq!(
            read.output,
            Some(read_success::Output::Content("wire".into()))
        );

        let unauthorized = http_call(
            handle.port,
            "/agent.v1.ControlService/Ping",
            "wrong",
            &[],
        );
        assert!(unauthorized.starts_with(b"HTTP/1.1 401 Unauthorized\r\n"));
        handle.stop().expect("stop");
    }

    fn http_call(
        port: u16,
        path: &str,
        token: &str,
        body: &[u8],
    ) -> Vec<u8> {
        let mut stream =
            TcpStream::connect(("127.0.0.1", port)).expect("connect");
        write!(
            stream,
            "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/proto\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .expect("headers");
        stream.write_all(body).expect("body");
        stream.flush().expect("flush");
        let mut response = Vec::new();
        let mut chunk = [0_u8; 4096];
        loop {
            if http_response_complete(&response) {
                break;
            }
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => response.extend_from_slice(&chunk[..count]),
                Err(error)
                    if error.kind() == std::io::ErrorKind::ConnectionReset
                        && http_response_complete(&response) =>
                {
                    break;
                }
                Err(error) => panic!("response: {error:?}"),
            }
        }
        assert!(
            http_response_complete(&response),
            "incomplete HTTP response: {} bytes",
            response.len()
        );
        response
    }

    fn http_response_complete(response: &[u8]) -> bool {
        let Some(header_index) = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
        else {
            return false;
        };
        let body_offset = header_index + 4;
        let Ok(headers) = std::str::from_utf8(&response[..body_offset]) else {
            return false;
        };
        if headers.lines().any(|line| {
            line.split_once(':').is_some_and(|(name, value)| {
                name.eq_ignore_ascii_case("transfer-encoding")
                    && value.trim().eq_ignore_ascii_case("chunked")
            })
        }) {
            return chunked_body_complete(&response[body_offset..]);
        }
        let Some(content_length) = headers.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        }) else {
            return false;
        };
        response.len() >= body_offset.saturating_add(content_length)
    }

    fn chunked_body_complete(body: &[u8]) -> bool {
        let mut cursor = 0usize;
        loop {
            let Some(line_end) = body[cursor..]
                .windows(2)
                .position(|window| window == b"\r\n")
                .map(|offset| cursor + offset)
            else {
                return false;
            };
            let Ok(size_text) = std::str::from_utf8(&body[cursor..line_end]) else {
                return false;
            };
            let Ok(size) = usize::from_str_radix(size_text.trim(), 16) else {
                return false;
            };
            cursor = line_end + 2;
            if size == 0 {
                return body.get(cursor..cursor + 2) == Some(b"\r\n");
            }
            let Some(data_end) = cursor.checked_add(size) else {
                return false;
            };
            if body.len() < data_end.saturating_add(2)
                || body.get(data_end..data_end + 2) != Some(b"\r\n")
            {
                return false;
            }
            cursor = data_end + 2;
        }
    }

    fn http_body(response: &[u8]) -> Vec<u8> {
        let offset = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("header end")
            + 4;
        let headers = std::str::from_utf8(&response[..offset]).expect("headers");
        let body = &response[offset..];
        if !headers.lines().any(|line| {
            line.split_once(':').is_some_and(|(name, value)| {
                name.eq_ignore_ascii_case("transfer-encoding")
                    && value.trim().eq_ignore_ascii_case("chunked")
            })
        }) {
            return body.to_vec();
        }

        let mut output = Vec::new();
        let mut cursor = 0usize;
        loop {
            let line_end = body[cursor..]
                .windows(2)
                .position(|window| window == b"\r\n")
                .map(|relative| cursor + relative)
                .expect("chunk size line");
            let size = usize::from_str_radix(
                std::str::from_utf8(&body[cursor..line_end])
                    .expect("chunk size")
                    .trim(),
                16,
            )
            .expect("chunk size hex");
            cursor = line_end + 2;
            if size == 0 {
                break;
            }
            output.extend_from_slice(&body[cursor..cursor + size]);
            cursor += size + 2;
        }
        output
    }
}
