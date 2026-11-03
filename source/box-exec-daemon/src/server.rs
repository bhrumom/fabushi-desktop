use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
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
const TIMEOUT_BEHAVIOR_BACKGROUND: i32 = 2;
const SHELL_BACKGROUND_REASON_TIMEOUT: i32 = 1;

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
pub struct WriteArgs {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "3")]
    pub tool_call_id: String,
    #[prost(bytes = "vec", tag = "5")]
    pub file_bytes: Vec<u8>,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct WriteResult {
    #[prost(oneof = "write_result::Result", tags = "1, 5, 6")]
    pub result: Option<write_result::Result>,
}
pub mod write_result {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Result {
        #[prost(message, tag = "1")]
        Success(WriteSuccess),
        #[prost(message, tag = "5")]
        Error(WriteFailure),
        #[prost(message, tag = "6")]
        Rejected(WriteFailure),
    }
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct WriteSuccess {
    #[prost(string, tag = "1")]
    pub path: String,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct WriteFailure {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "2")]
    pub reason: String,
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
    #[prost(bool, tag = "11")]
    pub is_background: bool,
    #[prost(bool, tag = "12")]
    pub skip_approval: bool,
    #[prost(int32, tag = "13")]
    pub timeout_behavior: i32,
    #[prost(int32, optional, tag = "14")]
    pub hard_timeout: Option<i32>,
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
    #[prost(oneof = "shell_stream::Event", tags = "1, 2, 3, 4, 5, 6, 7")]
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
        #[prost(message, tag = "5")]
        Rejected(ShellRejected),
        #[prost(message, tag = "6")]
        PermissionDenied(ShellPermissionDenied),
        #[prost(message, tag = "7")]
        Backgrounded(ShellStreamBackgrounded),
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
pub struct ShellRejected {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(string, tag = "3")]
    pub reason: String,
    #[prost(bool, tag = "4")]
    pub is_readonly: bool,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellPermissionDenied {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(string, tag = "3")]
    pub error: String,
    #[prost(bool, tag = "4")]
    pub is_readonly: bool,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct ShellStreamBackgrounded {
    #[prost(uint32, tag = "1")]
    pub shell_id: u32,
    #[prost(string, tag = "2")]
    pub command: String,
    #[prost(string, tag = "3")]
    pub working_directory: String,
    #[prost(uint32, optional, tag = "4")]
    pub pid: Option<u32>,
    #[prost(int32, optional, tag = "5")]
    pub ms_to_wait: Option<i32>,
    #[prost(int32, optional, tag = "6")]
    pub reason: Option<i32>,
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
    #[prost(oneof = "exec_server_message::Message", tags = "2, 3, 7, 29, 14, 16, 23, 52")]
    pub message: Option<exec_server_message::Message>,
}
pub mod exec_server_message {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Message {
        #[prost(message, tag = "2")]
        ShellArgs(ShellArgs),
        #[prost(message, tag = "3")]
        WriteArgs(WriteArgs),
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
    #[prost(oneof = "exec_client_message::Message", tags = "2, 3, 7, 29, 14, 16, 23, 55")]
    pub message: Option<exec_client_message::Message>,
}
pub mod exec_client_message {
    use super::*;
    #[derive(Clone, PartialEq, Oneof)]
    pub enum Message {
        #[prost(message, tag = "2")]
        ShellResult(ShellResult),
        #[prost(message, tag = "3")]
        WriteResult(WriteResult),
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
    // The outer lock only clones or closes this owner's reference. Pipe I/O
    // holds the inner lock after the registry and outer locks are released.
    stdin: Mutex<Option<Arc<Mutex<ChildStdin>>>>,
    terminal_path: PathBuf,
    writer: Arc<Mutex<File>>,
    state: Mutex<BackgroundState>,
    stop_requested: AtomicBool,
    active_input: AtomicUsize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BackgroundPhase {
    Setup,
    Admitting,
    Running,
    Recovery,
    Settled,
}

struct BackgroundState {
    resources: StreamResources,
    controller: Option<JoinHandle<()>>,
    phase: BackgroundPhase,
    admitted: bool,
    failure: Option<&'static str>,
    footer_written: bool,
    footer_failed: bool,
    started: Instant,
    started_at_ms: i64,
    hard_timeout_ms: Option<u64>,
    termination_started: Option<Instant>,
    force_sent: bool,
    #[cfg(test)]
    controller_fault: Option<BackgroundControllerFault>,
    #[cfg(test)]
    controller_exit_requested: bool,
    #[cfg(test)]
    probe: Option<Arc<CaptureProbe>>,
    #[cfg(test)]
    wait_fault_fired: bool,
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum BackgroundControllerFault {
    Create,
    Send,
    PanicBeforeCreate,
    PanicAfterReap,
    ExitAfterReap,
}

struct BackgroundRequest<'a> {
    runtime: &'a BoxExecRuntime,
    shell_id: u32,
    process: Arc<BackgroundProcess>,
    armed: bool,
}

impl BackgroundRequest<'_> {
    fn fail(&mut self, phase: &'static str) -> ShellExecutionUnknown {
        self.armed = false;
        self.runtime.cleanup_failed_background(self.shell_id, &self.process, phase);
        let unresolved = self.process.state.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()).phase != BackgroundPhase::Settled;
        let mut error = ShellExecutionUnknown::background(phase);
        error.cleanup_unresolved = unresolved;
        error
    }
}

impl Drop for BackgroundRequest<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.runtime.cleanup_failed_background(
                self.shell_id, &self.process, "background setup abandoned",
            );
        }
    }
}

struct BackgroundInput(Arc<BackgroundProcess>);

impl Drop for BackgroundInput {
    fn drop(&mut self) {
        self.0.active_input.fetch_sub(1, Ordering::AcqRel);
    }
}

struct CreatedTerminal {
    path: PathBuf,
    file: Option<File>,
}

impl Drop for CreatedTerminal {
    fn drop(&mut self) {
        if self.file.is_some() {
            drop(self.file.take());
            let _ = fs::remove_file(&self.path);
        }
    }
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


/// A child entered execution, but its result/capture could not be observed.
/// The private phase is bounded and never contains command or captured bytes.
#[derive(Debug, Clone)]
pub struct ShellExecutionUnknown {
    phase: &'static str,
    cleanup_unresolved: bool,
    background: bool,
}

impl std::fmt::Display for ShellExecutionUnknown {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} shell outcome unknown: {}",
            if self.background { "background" } else { "foreground" }, self.phase)?;
        if self.cleanup_unresolved {
            formatter.write_str("; owned child cleanup unresolved")?;
        }
        Ok(())
    }
}

impl std::error::Error for ShellExecutionUnknown {}

impl ShellExecutionUnknown {
    fn new(phase: &'static str) -> Self {
        Self { phase, cleanup_unresolved: false, background: false }
    }

    fn background(phase: &'static str) -> Self {
        Self { phase, cleanup_unresolved: false, background: true }
    }
}

enum ProcessFailure {
    Spawn(String),
    Unknown(ShellExecutionUnknown),
}

// A registration outlives both its direct child PID and any observed reap.
// Allocated once per invocation; no live or retired identity is reused.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ForegroundId(u64);

#[derive(Clone)]
enum ForegroundProcess {
    // The request owns the Child and consumes the runtime stopping flag.
    Stream,
    // The same registry retains a Child if bounded cleanup cannot observe reap.
    Capture(Arc<Mutex<CapturedChild>>),
}

struct CapturedChild {
    child: Child,
    status: Option<ExitStatus>,
    cleanup_unknown: bool,
    capture_complete: bool,
    stream_recovery: Option<StreamRecovery>,
    #[cfg(test)]
    signals_sent: usize,
}

impl CapturedChild {
    fn new(child: Child) -> Self {
        Self {
            child, status: None, cleanup_unknown: false, capture_complete: false,
            stream_recovery: None,
            #[cfg(test)]
            signals_sent: 0,
        }
    }

    fn observe_stream_recovery(&mut self) {
        let Some(recovery) = self.stream_recovery.as_mut() else { return; };
        for worker in [&mut recovery.stdout_worker, &mut recovery.stderr_worker] {
            if worker.as_ref().is_some_and(|handle| handle.is_finished()) {
                // A panic is still an observed worker termination. This path
                // certifies cleanup only; the request already returned unknown.
                let _ = worker.take().expect("finished stream worker").join();
            }
        }
        if self.status.is_some()
            && recovery.stdout_worker.is_none() && recovery.stderr_worker.is_none()
        {
            for path in &recovery.cleanup_paths {
                let _ = fs::remove_file(path);
            }
            drop(self.child.stdin.take());
            drop(self.child.stdout.take());
            drop(self.child.stderr.take());
            self.capture_complete = true;
            self.stream_recovery = None;
        }
    }

    fn poll(&mut self) -> Result<Option<ExitStatus>, ShellExecutionUnknown> {
        if self.status.is_none() {
            self.status = self.child.try_wait()
                .map_err(|_| ShellExecutionUnknown::new("child wait failed"))?;
        }
        Ok(self.status)
    }

    fn signal(&mut self, force: bool) -> Result<(), ShellExecutionUnknown> {
        // Poll and signal through the same owned state. An unreaped child
        // reserves its PID; after observing reap we never signal that number.
        if self.poll()?.is_some() {
            return Ok(());
        }
        #[cfg(unix)]
        {
            let signal = if force { libc::SIGKILL } else { libc::SIGTERM };
            let result = unsafe { libc::kill(-(self.child.id() as i32), signal) };
            if result != 0 {
                if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                    // Exit may race this syscall without any intervening reap.
                    // Keep observing the same Child; never infer non-execution.
                    self.poll()?;
                    return Ok(());
                }
                return Err(ShellExecutionUnknown::new("child termination failed"));
            }
        }
        #[cfg(not(unix))]
        self.child.kill()
            .map_err(|_| ShellExecutionUnknown::new("child termination failed"))?;
        #[cfg(test)]
        { self.signals_sent += 1; }
        Ok(())
    }
}

struct StreamRecovery {
    stdout_worker: Option<JoinHandle<Result<(), ShellExecutionUnknown>>>,
    stderr_worker: Option<JoinHandle<Result<(), ShellExecutionUnknown>>>,
    cleanup_paths: Vec<PathBuf>,
    failure: Arc<Mutex<Option<&'static str>>>,
}

struct StreamResources {
    state: CapturedChild,
    recovery: StreamRecovery,
}

impl StreamResources {
    fn remove_created_file(&mut self, path: &Path) {
        let Some(index) = self.recovery.cleanup_paths.iter().position(|owned| owned == path)
            else { return; };
        if fs::remove_file(path).is_ok() {
            // Another runtime can now claim this candidate. No later failure
            // may remove that successor through an obsolete cleanup entry.
            self.recovery.cleanup_paths.swap_remove(index);
        }
    }
}

// The active request retains its actual resources from spawn onward. Any
// exceptional return or unwind transfers them to the same invocation's
// captured-child recovery, which request admission or stop can observe without
// blocking on a pipe held open by a descendant.
struct StreamRequest<'a> {
    runtime: &'a BoxExecRuntime,
    registration_id: ForegroundId,
    resources: Option<StreamResources>,
}

impl StreamRequest<'_> {
    fn resources(&mut self) -> &mut StreamResources {
        self.resources.as_mut().expect("owned stream resources")
    }

    fn complete(&mut self) -> Result<(), String> {
        let resources = self.resources();
        if resources.state.status.is_none()
            || resources.recovery.stdout_worker.is_some()
            || resources.recovery.stderr_worker.is_some()
        {
            return Err("stream cleanup was not fully observed".into());
        }
        self.resources.take();
        self.runtime.foreground_processes.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&self.registration_id);
        Ok(())
    }
}

impl Drop for StreamRequest<'_> {
    fn drop(&mut self) {
        let Some(mut resources) = self.resources.take() else { return; };
        resources.state.capture_complete = false;
        resources.state.stream_recovery = Some(resources.recovery);
        let pid = resources.state.child.id();
        let owner = Arc::new(Mutex::new(resources.state));
        {
            let mut processes = self.runtime.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some((_, process)) = processes.get_mut(&self.registration_id) {
                *process = ForegroundProcess::Capture(Arc::clone(&owner));
            } else {
                processes.insert(self.registration_id,
                    (pid, ForegroundProcess::Capture(Arc::clone(&owner))));
            }
        }
        // This observes only the direct Child and is bounded. Reader handles
        // remain in recovery even if a descendant prevents EOF indefinitely.
        let _ = cleanup_captured_child(&owner);
        self.runtime.observe_retained_stream_recovery(self.registration_id, &owner);
        #[cfg(test)]
        if let Some(observer) = self.runtime.stream_recovery_observer.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()).as_ref()
        {
            let _ = observer.send((self.registration_id, Arc::clone(&owner)));
        }
    }
}

const FOREGROUND_POLL_INTERVAL: Duration = Duration::from_millis(10);
const FOREGROUND_TERM_GRACE: Duration = Duration::from_millis(250);
const FOREGROUND_CLEANUP_LIMIT: Duration = Duration::from_secs(2);
const STREAM_SPOOL_CREATE_ATTEMPTS: u32 = 64;

fn cleanup_captured_child(
    owner: &Arc<Mutex<CapturedChild>>,
) -> Result<(), ShellExecutionUnknown> {
    let started = Instant::now();
    let mut sent_term = false;
    let mut sent_kill = false;
    loop {
        {
            let mut state = owner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            match state.poll() {
                Ok(Some(_)) => {
                    state.cleanup_unknown = false;
                    return Ok(());
                }
                Ok(None) => {}
                Err(error) => {
                    state.cleanup_unknown = true;
                    return Err(error);
                }
            }
            let force = started.elapsed() >= FOREGROUND_TERM_GRACE;
            if !sent_term || (force && !sent_kill) {
                if let Err(error) = state.signal(force) {
                    state.cleanup_unknown = true;
                    return Err(error);
                }
                sent_term = true;
                sent_kill |= force;
            }
            if started.elapsed() >= FOREGROUND_CLEANUP_LIMIT {
                state.cleanup_unknown = true;
                return Err(ShellExecutionUnknown::new("child cleanup deadline elapsed"));
            }
        }
        thread::sleep(FOREGROUND_POLL_INTERVAL);
    }
}

struct CaptureWorker {
    worker: Option<JoinHandle<Result<Vec<u8>, ShellExecutionUnknown>>>,
    bytes: Option<Vec<u8>>,
}

impl CaptureWorker {
    fn new() -> Self {
        Self { worker: None, bytes: None }
    }

    fn observe(&mut self, wait: bool) -> Result<(), ShellExecutionUnknown> {
        if self.worker.as_ref().is_some_and(|worker| wait || worker.is_finished()) {
            let worker = self.worker.take().expect("observed capture worker");
            self.bytes = Some(worker.join()
                .map_err(|_| ShellExecutionUnknown::new("capture worker lost"))??);
        }
        Ok(())
    }
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
    // PID is observational process data; it never selects registration or signals.
    foreground_processes: Mutex<HashMap<ForegroundId, (u32, ForegroundProcess)>>,
    next_foreground_id: AtomicU64,
    #[cfg(test)]
    foreground_pid_alias: Mutex<Option<u32>>,
    #[cfg(test)]
    capture_observation_gate: Mutex<Option<mpsc::Receiver<()>>>,
    #[cfg(test)]
    stream_setup_gate: Mutex<Option<mpsc::Receiver<()>>>,
    #[cfg(test)]
    stream_panic_after_reap: AtomicBool,
    #[cfg(test)]
    stream_recovery_observer:
        Mutex<Option<mpsc::Sender<(ForegroundId, Arc<Mutex<CapturedChild>>)>>>,
    #[cfg(test)]
    capture_probe: Mutex<Option<Arc<CaptureProbe>>>,
    #[cfg(test)]
    background_setup_gate: Mutex<Option<mpsc::Receiver<()>>>,
    #[cfg(test)]
    background_controller_fault: Mutex<Option<BackgroundControllerFault>>,
    #[cfg(test)]
    background_controller_gate: Mutex<Option<mpsc::Receiver<()>>>,
    background: Arc<Mutex<HashMap<u32, Arc<BackgroundProcess>>>>,
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
            foreground_processes: Mutex::new(HashMap::new()),
            next_foreground_id: AtomicU64::new(1),
            #[cfg(test)]
            foreground_pid_alias: Mutex::new(None),
            #[cfg(test)]
            capture_observation_gate: Mutex::new(None),
            #[cfg(test)]
            stream_setup_gate: Mutex::new(None),
            #[cfg(test)]
            stream_panic_after_reap: AtomicBool::new(false),
            #[cfg(test)]
            stream_recovery_observer: Mutex::new(None),
            #[cfg(test)]
            capture_probe: Mutex::new(None),
            #[cfg(test)]
            background_setup_gate: Mutex::new(None),
            #[cfg(test)]
            background_controller_fault: Mutex::new(None),
            #[cfg(test)]
            background_controller_gate: Mutex::new(None),
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

    pub fn write(&self, args: &WriteArgs) -> WriteResult {
        if args.file_bytes.len() > 8 * 1024 * 1024 {
            return WriteResult { result: Some(write_result::Result::Rejected(WriteFailure {path:args.path.clone(),reason:"Write exceeds 8MiB limit".into()})) };
        }
        let target=match self.resolve_path(&args.path) {
            Ok(target)=>target,
            Err(_)=>return WriteResult {result:Some(write_result::Result::Rejected(WriteFailure {path:args.path.clone(),reason:"Write path outside workspace".into()}))},
        };
        match crate::workspace_write::write(&self.workspace_root,&target,&args.file_bytes) {
            Ok(())=>WriteResult {result:Some(write_result::Result::Success(WriteSuccess {path:args.path.clone()}))},
            Err(reason)=>WriteResult {result:Some(write_result::Result::Rejected(WriteFailure {path:args.path.clone(),reason}))},
        }
    }

    pub fn read(&self, args: &ReadArgs) -> ReadResult {
        self.observe_background_recoveries();
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
            if info.len()>8*1024*1024 {
                return Err(ReadFailure::Invalid(args.path.clone(),"Read exceeds 8MiB limit".into()));
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
            if args.encoding_hint.is_none() && (data.contains(&0) || std::str::from_utf8(&data).is_err()) {
                if args.offset.is_some() || args.limit.is_some() {
                    return Err(ReadFailure::Invalid(args.path.clone(),"Line ranges unsupported for binary reads".into()));
                }
                return Ok(ReadSuccess {path:args.path.clone(),file_size:data.len() as i64,output:Some(read_success::Output::Data(data)),total_lines:0,truncated:false,output_blob_id:None,range_applied:false});
            }
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

    pub fn shell(&self, args: &ShellArgs) -> Result<ShellResult, ShellExecutionUnknown> {
        let abort = AtomicBool::new(false);
        self.shell_with_abort(args, &abort)
    }

    fn shell_with_abort(
        &self,
        args: &ShellArgs,
        abort: &AtomicBool,
    ) -> Result<ShellResult, ShellExecutionUnknown> {
        let cwd = match self.resolve_path(&args.working_directory) {
            Ok(cwd) => cwd,
            Err(error) => {
                return Ok(ShellResult {
                    result: Some(shell_result::Result::SpawnError(ShellSpawnError {
                        command: args.command.clone(),
                        working_directory: args.working_directory.clone(),
                        error,
                    })),
                });
            }
        };
        let timeout = (args.timeout > 0).then_some(args.timeout as u64);
        let outcome = match self.run_process(&args.command, &cwd, timeout, abort) {
            Ok(outcome) => outcome,
            Err(ProcessFailure::Unknown(error)) => return Err(error),
            Err(ProcessFailure::Spawn(error)) => {
                return Ok(ShellResult {
                    result: Some(shell_result::Result::SpawnError(ShellSpawnError {
                        command: args.command.clone(),
                        working_directory: args.working_directory.clone(),
                        error,
                    })),
                });
            }
        };
        if outcome.timed_out {
            return Ok(ShellResult {
                result: Some(shell_result::Result::Timeout(ShellTimeout {
                    command: args.command.clone(),
                    working_directory: args.working_directory.clone(),
                    timeout_ms: args.timeout,
                })),
            });
        }
        let elapsed = saturating_i32(outcome.elapsed_ms);
        let interleaved = format!("{}{}", outcome.stdout, outcome.stderr);
        Ok(if outcome.code == 0 && !outcome.aborted {
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
        })
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
        self.observe_background_recoveries();
        self.observe_retained_stream_recoveries();
        let cwd = self.resolve_path(&args.working_directory)?;
        emit(client_message(
            id,
            exec_id,
            exec_client_message::Message::ShellStream(ShellStream {
                event: Some(shell_stream::Event::Start(ShellStreamStart {})),
            }),
            None,
        ))?;

        #[cfg(test)]
        let stream_setup_gate = self.stream_setup_gate.lock()
            .expect("stream setup gate").take();
        let started_at = Utc::now();
        let started = Instant::now();
        let mut request = {
            let mut processes = self.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if self.stopping.load(Ordering::Acquire) {
                return Err("box exec daemon is stopping".into());
            }
            let registration_id = self.allocate_foreground_id()?;
            let child = self.spawn_shell(&args.command, &cwd)?;
            let pid = child.id();
            let request = StreamRequest {
                runtime: self, registration_id,
                resources: Some(StreamResources {
                    state: CapturedChild::new(child),
                    recovery: StreamRecovery {
                        stdout_worker: None, stderr_worker: None, cleanup_paths: Vec::new(),
                        failure: Arc::new(Mutex::new(None)),
                    },
                }),
            };
            processes.insert(registration_id,
                self.foreground_registration(pid, ForegroundProcess::Stream));
            request
        };
        #[cfg(test)]
        if let Some(gate) = stream_setup_gate {
            gate.recv_timeout(Duration::from_secs(10))
                .map_err(|_| "test stream setup gate not released".to_string())?;
        }
        // Only this request owns/reaps this Child. Registry PID data is never
        // signal authority, including when a test presents a reused number.
        fs::create_dir_all(&self.terminals_directory).map_err(|error| error.to_string())?;
        let (stream_spool_path, stream_spool) =
            self.create_stream_spool(request.registration_id)?;
        request.resources().recovery.cleanup_paths.push(stream_spool_path.clone());
        let stream_spool_writer = Arc::new(Mutex::new(stream_spool));

        let (events_tx, events_rx) = mpsc::channel::<ShellPipeEvent>();
        let control = self.stream_capture_control(request.resources().state.child.id());
        start_stream_reader(
            request.resources(), Some(events_tx.clone()), false,
            Arc::clone(&stream_spool_writer), control.clone(),
        ).map_err(|error| error.to_string())?;
        start_stream_reader(
            request.resources(), Some(events_tx.clone()), true,
            Arc::clone(&stream_spool_writer), control,
        ).map_err(|error| error.to_string())?;
        drop(events_tx);

        let timeout_ms = (args.timeout > 0).then_some(args.timeout as u64);
        let hard_timeout_ms = args
            .hard_timeout
            .filter(|timeout| *timeout > 0)
            .map(|timeout| timeout as u64);
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
                    break;
                }
            }
            if let Some(error) = stream_error.take() {
                return Err(error);
            }
            match request.resources().state.poll()
                .map_err(|error| error.to_string())?
            {
                Some(done) => {
                    #[cfg(test)]
                    if self.stream_panic_after_reap.swap(false, Ordering::AcqRel) {
                        panic!("controlled stream unwind after actual child reap");
                    }
                    status = Some(done);
                    break;
                }
                None => {}
            }

            let hard_timeout_elapsed = hard_timeout_ms
                .is_some_and(|timeout| started.elapsed() >= Duration::from_millis(timeout));
            let block_timeout_elapsed = timeout_ms
                .is_some_and(|timeout| started.elapsed() >= Duration::from_millis(timeout));

            if !kill_requested
                && !abort.load(Ordering::Acquire)
                && !self.stopping.load(Ordering::Acquire)
                && !hard_timeout_elapsed
                && block_timeout_elapsed
                && args.timeout_behavior == TIMEOUT_BEHAVIOR_BACKGROUND
            {
                let backgrounded = self.handoff_shell_stream_to_background(
                    &mut request,
                    args,
                    started,
                    started_at,
                    Arc::clone(&stream_spool_writer),
                    stream_spool_path.clone(),
                )?;
                emit(client_message(
                    id,
                    exec_id,
                    exec_client_message::Message::ShellStream(ShellStream {
                        event: Some(shell_stream::Event::Backgrounded(backgrounded)),
                    }),
                    None,
                ))?;
                return Ok(());
            }

            if !kill_requested
                && (abort.load(Ordering::Acquire)
                    || self.stopping.load(Ordering::Acquire)
                    || hard_timeout_elapsed
                    || (block_timeout_elapsed
                        && args.timeout_behavior != TIMEOUT_BEHAVIOR_BACKGROUND))
            {
                request.resources().state.signal(false)
                    .map_err(|error| error.to_string())?;
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
                        return Err(error);
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {}
            }
        }

        if let Some(worker) = request.resources().recovery.stdout_worker.take() {
            let _ = worker.join();
        }
        if let Some(worker) = request.resources().recovery.stderr_worker.take() {
            let _ = worker.join();
        }
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
        drop(stream_spool_writer);
        request.resources().remove_created_file(&stream_spool_path);

        if let Some(error) = stream_error {
            return Err(error);
        }
        let status = status.ok_or_else(|| "stream child reap was not observed".to_string())?;
        request.complete()?;
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

    fn stream_spool_path(&self, registration_id: ForegroundId, attempt: u32) -> PathBuf {
        self.terminals_directory.join(format!(
            ".shell-stream-{}-{}-{attempt}.tmp",
            std::process::id(), registration_id.0,
        ))
    }

    fn create_stream_spool(&self, registration_id: ForegroundId) -> Result<(PathBuf, File), String> {
        // Process/local identity reduces collisions; only create_new grants
        // ownership across independent runtimes sharing this directory.
        for attempt in 0..STREAM_SPOOL_CREATE_ATTEMPTS {
            let path = self.stream_spool_path(registration_id, attempt);
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => return Ok((path, file)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
        Err("shell stream spool allocation exhausted".into())
    }

    fn handoff_shell_stream_to_background(
        &self,
        request: &mut StreamRequest<'_>,
        args: &ShellArgs,
        started: Instant,
        started_at: chrono::DateTime<Utc>,
        stream_spool_writer: Arc<Mutex<File>>,
        stream_spool_path: PathBuf,
    ) -> Result<ShellStreamBackgrounded, String> {
        let shell_id = self.allocate_shell_id()?;
        let pid = request.resources().state.child.id();
        if request.resources().state.child.stdin.is_none() {
            return Err("shell stream stdin was unavailable during background handoff".into());
        }
        let terminal_path = self.terminals_directory.join(format!("{shell_id}.txt"));
        let background_args = BackgroundShellSpawnArgs {
            command: args.command.clone(),
            working_directory: args.working_directory.clone(),
            tool_call_id: args.tool_call_id.clone(),
        };
        {
            let mut current_writer = stream_spool_writer.lock()
                .map_err(|_| "shell stream output writer is poisoned".to_string())?;
            current_writer.flush().map_err(|error| error.to_string())?;
            let spooled = fs::read(&stream_spool_path).map_err(|error| error.to_string())?;
            let mut terminal = OpenOptions::new().create_new(true).write(true).open(&terminal_path)
                .map_err(|error| error.to_string())?;
            request.resources().recovery.cleanup_paths.push(terminal_path.clone());
            terminal.write_all(
                terminal_frontmatter(&background_args, Some(pid), started_at).as_bytes(),
            ).map_err(|error| error.to_string())?;
            terminal.write_all(&spooled).map_err(|error| error.to_string())?;
            terminal.flush().map_err(|error| error.to_string())?;
            *current_writer = terminal;
        }
        request.resources().remove_created_file(&stream_spool_path);
        let mut background_request = {
            let mut foreground = self.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut background = self.background.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if self.stopping.load(Ordering::Acquire) {
                return Err("box exec daemon is stopping".into());
            }
            if background.contains_key(&shell_id) {
                return Err("background shell identity is already registered".into());
            }
            let resources = request.resources.take().expect("owned handoff resources");
            let process = self.background_process(
                resources, terminal_path, stream_spool_writer,
                started, started_at.timestamp_millis(),
                args.hard_timeout.filter(|timeout| *timeout > 0).map(|timeout| timeout as u64),
            );
            let guard = BackgroundRequest {
                runtime: self, shell_id, process: Arc::clone(&process), armed: true,
            };
            background.insert(shell_id, process);
            foreground.remove(&request.registration_id);
            guard
        };
        if let Err(error) = self.admit_background(shell_id, &background_request.process) {
            return Err(background_request.fail(error.phase).to_string());
        }
        background_request.armed = false;
        Ok(ShellStreamBackgrounded {
            shell_id, command: args.command.clone(),
            working_directory: args.working_directory.clone(), pid: Some(pid),
            ms_to_wait: Some(args.timeout), reason: Some(SHELL_BACKGROUND_REASON_TIMEOUT),
        })
    }

    pub fn spawn_background(
        &self,
        args: &BackgroundShellSpawnArgs,
    ) -> Result<BackgroundShellSpawnResult, ShellExecutionUnknown> {
        self.observe_background_recoveries();
        let known_error = |error: String| BackgroundShellSpawnResult {
            result: Some(background_shell_spawn_result::Result::Error(BackgroundShellSpawnError {
                command: args.command.clone(), working_directory: args.working_directory.clone(), error,
            })),
        };
        let cwd = match self.resolve_path(&args.working_directory) {
            Ok(cwd) => cwd,
            Err(error) => return Ok(known_error(error)),
        };
        #[cfg(test)]
        let setup_gate = self.background_setup_gate.lock().unwrap().take();
        let started_at = Utc::now();
        let started = Instant::now();
        let (mut request, pid) = {
            let mut background = self.background.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            if self.stopping.load(Ordering::Acquire) {
                return Ok(known_error("box exec daemon is stopping".into()));
            }
            let shell_id = match self.allocate_shell_id() {
                Ok(id) => id,
                Err(error) => return Ok(known_error(error)),
            };
            if let Err(error) = fs::create_dir_all(&self.terminals_directory) {
                return Ok(known_error(error.to_string()));
            }
            let path = self.terminals_directory.join(format!("{shell_id}.txt"));
            let file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => file,
                Err(error) => return Ok(known_error(error.to_string())),
            };
            let mut terminal = CreatedTerminal { path: path.clone(), file: Some(file) };
            let child = match self.spawn_shell(&args.command, &cwd) {
                Ok(child) => child,
                Err(error) => return Ok(known_error(error)),
            };
            // No post-spawn fallible operation precedes actual guarded custody.
            let pid = child.id();
            let resources = StreamResources {
                state: CapturedChild::new(child),
                recovery: StreamRecovery {
                    stdout_worker: None, stderr_worker: None,
                    cleanup_paths: vec![path.clone()],
                    failure: Arc::new(Mutex::new(None)),
                },
            };
            let writer = Arc::new(Mutex::new(terminal.file.take().expect("owned created terminal")));
            let process = self.background_process(
                resources, path, writer, started, started_at.timestamp_millis(), None,
            );
            let request = BackgroundRequest {
                runtime: self, shell_id, process: Arc::clone(&process), armed: true,
            };
            background.insert(shell_id, process);
            (request, pid)
        };
        let setup = (|| -> Result<(), ShellExecutionUnknown> {
            #[cfg(test)]
            if let Some(gate) = setup_gate {
                gate.recv_timeout(Duration::from_secs(10))
                    .map_err(|_| ShellExecutionUnknown::background("test background setup gate not released"))?;
            }
            {
                let mut writer = request.process.writer.lock()
                    .map_err(|_| ShellExecutionUnknown::background("background output writer poisoned"))?;
                writer.write_all(terminal_frontmatter(args, Some(pid), started_at).as_bytes())
                    .and_then(|_| writer.flush())
                    .map_err(|_| ShellExecutionUnknown::background("background terminal setup failed"))?;
            }
            let control = self.stream_capture_control(pid);
            {
                let mut state = request.process.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                start_stream_reader(
                    &mut state.resources, None, false, Arc::clone(&request.process.writer), control.clone(),
                )?;
            }
            #[cfg(test)]
            if self.wait_for_background_probe(CaptureFaultKind::BackgroundSetup)? {
                return Err(ShellExecutionUnknown::background("background capture setup failed"));
            }
            {
                let mut state = request.process.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                start_stream_reader(
                    &mut state.resources, None, true, Arc::clone(&request.process.writer), control,
                )?;
            }
            self.admit_background(request.shell_id, &request.process)
        })();
        if let Err(error) = setup {
            return Err(request.fail(error.phase));
        }
        request.armed = false;
        Ok(BackgroundShellSpawnResult {
            result: Some(background_shell_spawn_result::Result::Success(BackgroundShellSpawnSuccess {
                shell_id: request.shell_id, command: args.command.clone(),
                working_directory: args.working_directory.clone(), pid: Some(pid),
            })),
        })
    }


    fn allocate_shell_id(&self) -> Result<u32, String> {
        self.next_shell_id.fetch_update(Ordering::Relaxed, Ordering::Relaxed,
            |next| (next != 0).then(|| next.checked_add(1)).flatten())
            .map_err(|_| "background shell id space exhausted".to_string())
    }

    fn stream_capture_control(&self, pid: u32) -> CaptureControl {
        #[cfg(not(test))]
        let _ = pid;
        CaptureControl {
            cancelled: Arc::new(AtomicBool::new(false)),
            #[cfg(test)]
            probe: self.capture_probe.lock().expect("capture probe").clone(),
            #[cfg(test)]
            pid,
        }
    }

    fn background_process(
        &self,
        mut resources: StreamResources,
        terminal_path: PathBuf,
        writer: Arc<Mutex<File>>,
        started: Instant,
        started_at_ms: i64,
        hard_timeout_ms: Option<u64>,
    ) -> Arc<BackgroundProcess> {
        let stdin = resources.state.child.stdin.take().map(|stdin| Arc::new(Mutex::new(stdin)));
        Arc::new(BackgroundProcess {
            stdin: Mutex::new(stdin), terminal_path, writer,
            state: Mutex::new(BackgroundState {
                resources, controller: None, phase: BackgroundPhase::Setup,
                admitted: false, failure: None, footer_written: false, footer_failed: false,
                started, started_at_ms, hard_timeout_ms, termination_started: None, force_sent: false,
                #[cfg(test)]
                controller_fault: self.background_controller_fault.lock().unwrap().take(),
                #[cfg(test)]
                controller_exit_requested: false,
                #[cfg(test)]
                probe: self.capture_probe.lock().unwrap().clone(),
                #[cfg(test)]
                wait_fault_fired: false,
            }),
            stop_requested: AtomicBool::new(false),
            active_input: AtomicUsize::new(0),
        })
    }

    #[cfg(test)]
    fn wait_for_background_probe(&self, kind: CaptureFaultKind) -> Result<bool, ShellExecutionUnknown> {
        let probe = self.capture_probe.lock().unwrap().clone();
        let Some(probe) = probe.filter(|probe| probe.kind == kind) else { return Ok(false); };
        let deadline = Instant::now() + Duration::from_secs(3);
        while !probe.observed.load(Ordering::Acquire) {
            if Instant::now() >= deadline {
                return Err(ShellExecutionUnknown::background("test background prefix was not observed"));
            }
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        Ok(true)
    }

    fn admit_background(
        &self,
        shell_id: u32,
        process: &Arc<BackgroundProcess>,
    ) -> Result<(), ShellExecutionUnknown> {
        {
            let mut state = process.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.phase != BackgroundPhase::Setup || state.controller.is_some() {
                return Err(ShellExecutionUnknown::background("background admission already started"));
            }
            // Only this caller can admit the entry. Observers cannot finalize
            // Setup/Admitting, and the controller waits for start_rx. Thus the
            // final map -> state lock below cannot oppose a running state -> map
            // completion. Repeated admission is refused before taking the map.
            state.phase = BackgroundPhase::Admitting;
        }
        #[cfg(test)]
        {
            let gate = self.background_controller_gate.lock().unwrap().take();
            if let Some(gate) = gate {
                gate.recv_timeout(Duration::from_secs(10))
                    .map_err(|_| ShellExecutionUnknown::background("test background controller gate not released"))?;
            }
            if self.wait_for_background_probe(CaptureFaultKind::BackgroundController)? {
                return Err(ShellExecutionUnknown::background("background controller creation failed"));
            }
            let fault = process.state.lock().unwrap().controller_fault;
            if fault == Some(BackgroundControllerFault::Create) {
                return Err(ShellExecutionUnknown::background("background controller creation failed"));
            }
            if fault == Some(BackgroundControllerFault::PanicBeforeCreate) {
                panic!("controlled background setup unwind after actual reader start");
            }
        }
        let (start_tx, start_rx) = mpsc::channel();
        let processes = Arc::clone(&self.background);
        let worker_process = Arc::clone(process);
        #[cfg(test)]
        let refuse_send = process.state.lock().unwrap().controller_fault
            == Some(BackgroundControllerFault::Send);
        #[cfg(test)]
        let (refused_tx, refused_rx) = mpsc::channel();
        let controller = thread::Builder::new().name("box-background-controller".into())
            .spawn(move || {
                #[cfg(test)]
                if refuse_send {
                    drop(start_rx);
                    let _ = refused_tx.send(());
                    return;
                }
                if start_rx.recv().is_err() { return; }
                run_background_controller(processes, shell_id, worker_process);
            }).map_err(|_| ShellExecutionUnknown::background("background controller creation failed"))?;
        process.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
            .controller = Some(controller);
        #[cfg(test)]
        if refuse_send {
            refused_rx.recv_timeout(Duration::from_secs(3))
                .map_err(|_| ShellExecutionUnknown::background("test controller refusal was not observed"))?;
        }
        let processes = self.background.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.stopping.load(Ordering::Acquire)
            || !matches!(processes.get(&shell_id), Some(current) if Arc::ptr_eq(current, process))
        {
            return Err(ShellExecutionUnknown::background("background admission closed"));
        }
        let mut state = process.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        observe_background_readers(&mut state);
        if let Some(phase) = state.failure {
            return Err(ShellExecutionUnknown::background(phase));
        }
        if process.stdin.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).is_none()
            || state.resources.state.child.stdout.is_some()
            || state.resources.state.child.stderr.is_some()
        {
            return Err(ShellExecutionUnknown::background("background capture setup failed"));
        }
        state.phase = BackgroundPhase::Running;
        start_tx.send(())
            .map_err(|_| ShellExecutionUnknown::background("background controller admission failed"))?;
        state.admitted = true;
        // A successfully admitted terminal is the persistent result. Transient
        // spools, if any, retain their separate cleanup claims.
        state.resources.recovery.cleanup_paths.retain(|path| path != &process.terminal_path);
        Ok(())
    }

    fn observe_background_recoveries(&self) {
        let processes = self.background.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter().map(|(id, process)| (*id, Arc::clone(process))).collect::<Vec<_>>();
        for (shell_id, process) in processes {
            let recoverable = match process.state.try_lock() {
                Ok(state) => state.phase == BackgroundPhase::Recovery
                    || state.controller.as_ref().is_some_and(|worker| worker.is_finished()),
                Err(std::sync::TryLockError::Poisoned(poisoned)) => {
                    let state = poisoned.into_inner();
                    state.phase == BackgroundPhase::Recovery
                        || state.controller.as_ref().is_some_and(|worker| worker.is_finished())
                }
                Err(std::sync::TryLockError::WouldBlock) => false,
            };
            if recoverable {
                drive_background(&self.background, shell_id, &process, false);
            }
        }
    }

    fn cleanup_failed_background(
        &self,
        shell_id: u32,
        process: &Arc<BackgroundProcess>,
        phase: &'static str,
    ) {
        {
            let mut state = process.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.phase == BackgroundPhase::Settled { return; }
            state.failure.get_or_insert(phase);
            state.phase = BackgroundPhase::Recovery;
        }
        process.stop_requested.store(true, Ordering::Release);
        let deadline = Instant::now() + FOREGROUND_CLEANUP_LIMIT;
        loop {
            if drive_background(&self.background, shell_id, process, false) { return; }
            if Instant::now() >= deadline { return; }
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
    }

    pub fn write_stdin(&self, args: &WriteShellStdinArgs) -> WriteShellStdinResult {
        self.observe_background_recoveries();
        let error_result = |error: String| WriteShellStdinResult {
            result: Some(write_shell_stdin_result::Result::Error(WriteShellStdinError { error })),
        };
        let (stdin, terminal_path, _input) = {
            let processes = self.background.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(process) = processes.get(&args.shell_id) else {
                return error_result(format!("Shell {} is not running", args.shell_id));
            };
            let input = process.stdin.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(stdin) = input.as_ref() else {
                return error_result(format!("Shell {} has no input pipe", args.shell_id));
            };
            process.active_input.fetch_add(1, Ordering::AcqRel);
            (
                Arc::clone(stdin), process.terminal_path.clone(), BackgroundInput(Arc::clone(process)),
            )
        };
        // No registry or process-state lock is held during filesystem or pipe
        // I/O. The counted exact invocation prevents premature final removal.
        let before = fs::metadata(&terminal_path)
            .map(|metadata| metadata.len().min(u32::MAX as u64) as u32).unwrap_or_default();
        let result = stdin.lock()
            .map_err(|_| "background shell stdin is poisoned".to_string())
            .and_then(|mut stdin| stdin.write_all(args.chars.as_bytes())
                .and_then(|_| stdin.flush()).map_err(|error| error.to_string()));
        match result {
            Ok(()) => WriteShellStdinResult {
                result: Some(write_shell_stdin_result::Result::Success(WriteShellStdinSuccess {
                    shell_id: args.shell_id, terminal_file_length_before_input_written: before,
                })),
            },
            Err(error) => error_result(error),
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
            Some(exec_server_message::Message::WriteArgs(args)) => {
                emit(client_message(id,&exec_id,exec_client_message::Message::WriteResult(self.write(&args)),None))?;
            }
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
                match self.shell_with_abort(&args, abort) {
                    Ok(result) => emit(client_message(
                        id,
                        &exec_id,
                        exec_client_message::Message::ShellResult(result),
                        Some(saturating_i32(started.elapsed().as_millis() as u64)),
                    ))?,
                    Err(error) => emit(thrown(
                        id, error.to_string(), "BOX_EXEC_DAEMON_ERROR",
                    ))?,
                }
            }
            Some(exec_server_message::Message::MiniSweAgentBashArgs(args)) => {
                let started = Instant::now();
                match self.shell_with_abort(&args, abort) {
                    Ok(result) => emit(client_message(
                        id,
                        &exec_id,
                        exec_client_message::Message::MiniSweAgentBashResult(result),
                        Some(saturating_i32(started.elapsed().as_millis() as u64)),
                    ))?,
                    Err(error) => emit(thrown(
                        id, error.to_string(), "BOX_EXEC_DAEMON_ERROR",
                    ))?,
                }
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
                match self.spawn_background(&args) {
                    Ok(result) => emit(client_message(
                        id, &exec_id, exec_client_message::Message::BackgroundShellSpawnResult(result), None,
                    ))?,
                    Err(error) => emit(thrown(id, error.to_string(), "BOX_EXEC_DAEMON_ERROR"))?,
                }
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

    pub fn stop(&self) -> Result<(), String> {
        self.stopping.store(true, Ordering::Release);
        let mut unresolved = false;
        let mut attempted = Vec::new();
        let deadline = Instant::now() + FOREGROUND_CLEANUP_LIMIT;
        loop {
            let background = self.background.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .values().cloned().collect::<Vec<_>>();
            for process in background {
                process.stop_requested.store(true, Ordering::Release);
                match process.state.try_lock() {
                    Ok(mut state) => advance_background_termination(&mut state),
                    Err(std::sync::TryLockError::Poisoned(poisoned)) =>
                        advance_background_termination(&mut poisoned.into_inner()),
                    Err(std::sync::TryLockError::WouldBlock) => {}
                }
                close_background_input(&process);
            }
            self.observe_background_recoveries();
            let foreground = self.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .iter().map(|(id, (_, process))| (*id, process.clone())).collect::<Vec<_>>();
            for (registration_id, process) in foreground {
                if let ForegroundProcess::Capture(owner) = process {
                    if !attempted.contains(&registration_id) {
                        unresolved |= cleanup_captured_child(&owner).is_err();
                        attempted.push(registration_id);
                    }
                    {
                        let mut state = owner.lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        unresolved |= state.poll().is_err();
                        state.observe_stream_recovery();
                    }
                    self.remove_completed_capture(registration_id, &owner);
                }
                // Active streams consume stopping in their actual Child owner.
                // If one returns/unwinds, its guard transfers real resources
                // into Capture; the next iteration observes that transition.
            }
            let pending = !self.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .is_empty()
                || !self.background.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).is_empty();
            if !pending {
                return if unresolved {
                    Err("foreground child cleanup was not fully observed".into())
                } else {
                    Ok(())
                };
            }
            if Instant::now() >= deadline {
                return Err("owned shell cleanup remains unresolved".into());
            }
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
    }

    fn observe_retained_stream_recoveries(&self) {
        // Snapshot first: never acquire a Child mutex while holding the map.
        let recoveries = self.foreground_processes.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter().filter_map(|(id, (_, process))| match process {
                ForegroundProcess::Capture(owner) => Some((*id, Arc::clone(owner))),
                ForegroundProcess::Stream => None,
            }).collect::<Vec<_>>();
        for (registration_id, owner) in recoveries {
            self.observe_retained_stream_recovery(registration_id, &owner);
        }
    }

    fn observe_retained_stream_recovery(
        &self,
        registration_id: ForegroundId,
        owner: &Arc<Mutex<CapturedChild>>,
    ) {
        {
            let mut state = owner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            // Ordinary capture readers still belong to their active caller.
            // Only the transferred stream recovery may be observed here.
            if state.stream_recovery.is_none() || state.poll().is_err() {
                return;
            }
            state.observe_stream_recovery();
        }
        // Release the Child mutex before exact identity/Arc-checked removal.
        self.remove_completed_capture(registration_id, owner);
    }

    fn allocate_foreground_id(&self) -> Result<ForegroundId, String> {
        self.next_foreground_id.fetch_update(Ordering::Relaxed, Ordering::Relaxed,
            |next| next.checked_add(1))
            .map(ForegroundId)
            .map_err(|_| "foreground invocation id space exhausted".to_string())
    }

    fn foreground_registration(
        &self,
        pid: u32,
        process: ForegroundProcess,
    ) -> (u32, ForegroundProcess) {
        #[cfg(test)]
        let pid = self.foreground_pid_alias.lock().expect("foreground PID alias")
            .unwrap_or(pid);
        (pid, process)
    }

    fn remove_completed_capture(
        &self,
        registration_id: ForegroundId,
        owner: &Arc<Mutex<CapturedChild>>,
    ) {
        let completed = {
            let state = owner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            state.status.is_some() && state.capture_complete
        };
        if completed {
            let mut processes = self.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if matches!(processes.get(&registration_id),
                Some((_, ForegroundProcess::Capture(current)))
                if Arc::ptr_eq(current, owner))
            {
                processes.remove(&registration_id);
            }
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
    ) -> Result<ProcessOutcome, ProcessFailure> {
        self.observe_background_recoveries();
        self.observe_retained_stream_recoveries();
        #[cfg(test)]
        let capture_observation_gate = self.capture_observation_gate.lock()
            .expect("capture observation gate").take();
        let (registration_id, owner) = {
            // Admission and stop's snapshot share the existing registry mutex.
            // Stop cannot report clean between a successful spawn and insertion.
            let mut processes = self.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if self.stopping.load(Ordering::Acquire) {
                return Err(ProcessFailure::Spawn("box exec daemon is stopping".into()));
            }
            let registration_id = self.allocate_foreground_id().map_err(ProcessFailure::Spawn)?;
            let child = self.spawn_shell(command, cwd).map_err(ProcessFailure::Spawn)?;
            let pid = child.id();
            let owner = Arc::new(Mutex::new(CapturedChild::new(child)));
            processes.insert(registration_id, self.foreground_registration(
                pid, ForegroundProcess::Capture(Arc::clone(&owner)),
            ));
            (registration_id, owner)
        };
        let control = CaptureControl {
            cancelled: Arc::new(AtomicBool::new(false)),
            #[cfg(test)]
            probe: self.capture_probe.lock().expect("capture probe").clone(),
            #[cfg(test)]
            pid: owner.lock().expect("capture child PID").child.id(),
        };
        let mut stdout_worker = CaptureWorker::new();
        let mut stderr_worker = CaptureWorker::new();
        let started = Instant::now();
        let result = (|| -> Result<ProcessOutcome, ShellExecutionUnknown> {
            let (stdout, stderr) = {
                let mut state = owner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                let stdout = state.child.stdout.take()
                    .ok_or_else(|| ShellExecutionUnknown::new("stdout pipe unavailable"))?;
                let stderr = state.child.stderr.take()
                    .ok_or_else(|| ShellExecutionUnknown::new("stderr pipe unavailable"))?;
                (stdout, stderr)
            };
            prepare_capture_pipe(&stdout)?;
            prepare_capture_pipe(&stderr)?;
            stdout_worker.worker = Some(spawn_capture_worker(stdout, control.clone(), "stdout")?);
            stderr_worker.worker = Some(spawn_capture_worker(stderr, control.clone(), "stderr")?);
            #[cfg(test)]
            {
                // Hold the actual request before observing/joining either
                // worker while the test observes this owned Child's real reap.
                if let Some(gate) = capture_observation_gate {
                    gate.recv_timeout(Duration::from_secs(15))
                        .map_err(|_| ShellExecutionUnknown::new("test capture gate not released"))?;
                }
            }
            let mut timed_out = false;
            let mut termination_started = None;
            let mut sent_kill = false;
            let mut capture_stop_started = None;
            loop {
                stdout_worker.observe(false)?;
                stderr_worker.observe(false)?;
                #[cfg(test)]
                if control.probe.as_ref().is_some_and(|probe| probe.wait_fault_ready()) {
                    return Err(ShellExecutionUnknown::new("child wait failed"));
                }
                let status = owner.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).poll()?;
                if let Some(status) = status {
                    if stdout_worker.bytes.is_some() && stderr_worker.bytes.is_some() {
                        return Ok(ProcessOutcome {
                            code: status.code().unwrap_or(1),
                            signal: status_signal(&status),
                            stdout: String::from_utf8_lossy(
                                stdout_worker.bytes.as_ref().expect("observed stdout"),
                            ).into_owned(),
                            stderr: String::from_utf8_lossy(
                                stderr_worker.bytes.as_ref().expect("observed stderr"),
                            ).into_owned(),
                            elapsed_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
                            timed_out,
                            aborted: abort.load(Ordering::Acquire)
                                || self.stopping.load(Ordering::Acquire),
                        });
                    }
                    // A normally completed direct child may have a legitimate
                    // descendant still writing. Do not invent an independent
                    // output deadline; only request timeout/abort/stop ends that
                    // wait, with a bounded chance to observe the final EOF.
                    if timeout_ms.is_some_and(|timeout|
                        started.elapsed() >= Duration::from_millis(timeout))
                        || abort.load(Ordering::Acquire)
                        || self.stopping.load(Ordering::Acquire)
                    {
                        let drain = capture_stop_started.get_or_insert_with(Instant::now);
                        if drain.elapsed() >= FOREGROUND_TERM_GRACE {
                            return Err(ShellExecutionUnknown::new("capture cancelled before EOF"));
                        }
                    }
                } else {
                    if termination_started.is_none() {
                        timed_out = timeout_ms.is_some_and(|timeout|
                            started.elapsed() >= Duration::from_millis(timeout));
                        if timed_out || abort.load(Ordering::Acquire)
                            || self.stopping.load(Ordering::Acquire)
                        {
                            owner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
                                .signal(false)?;
                            termination_started = Some(Instant::now());
                        }
                    }
                    if let Some(termination) = termination_started {
                        if !sent_kill && termination.elapsed() >= FOREGROUND_TERM_GRACE {
                            owner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
                                .signal(true)?;
                            sent_kill = true;
                        }
                        if termination.elapsed() >= FOREGROUND_CLEANUP_LIMIT {
                            return Err(ShellExecutionUnknown::new("child termination deadline elapsed"));
                        }
                    }
                }
                thread::sleep(FOREGROUND_POLL_INTERVAL);
            }
        })();
        // Every return after spawn passes this boundary. Cancellation stops even
        // a pipe inherited by a still-running descendant; both workers are joined.
        control.cancelled.store(true, Ordering::Release);
        let cleanup = if result.is_err() { cleanup_captured_child(&owner) } else { Ok(()) };
        let stdout_observed = stdout_worker.observe(true);
        let stderr_observed = stderr_worker.observe(true);
        {
            let mut state = owner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            state.capture_complete = true;
            state.cleanup_unknown |= cleanup.is_err();
        }
        self.remove_completed_capture(registration_id, &owner);
        match result {
            Err(mut error) => {
                error.cleanup_unresolved = owner.lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()).cleanup_unknown;
                Err(ProcessFailure::Unknown(error))
            }
            Ok(outcome) => {
                stdout_observed.map_err(ProcessFailure::Unknown)?;
                stderr_observed.map_err(ProcessFailure::Unknown)?;
                Ok(outcome)
            }
        }
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
        self.stop.store(true, Ordering::Release);
        self.ready.store(false, Ordering::Release);
        // A previous incomplete stop is retryable observation, never a false
        // success caused solely by the accept-loop flag already being set.
        let cleanup = self.runtime.stop();
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
        cleanup
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
    // A nonblocking listener may yield an accepted socket that is still
    // nonblocking on some Unix platforms. Normalize the per-request socket
    // before the worker starts its bounded blocking HTTP read so a scheduling
    // race cannot turn the first read into WouldBlock and close the connection.
    stream
        .set_nonblocking(false)
        .map_err(|error| error.to_string())?;
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
            let payload=match exec_request_body(&request) {
                Ok(payload)=>payload,
                Err(reason)=>return write_http_response(&mut stream,400,"text/plain",reason.as_bytes(),&[]),
            };
            let request = ExecServerMessage::decode(payload)
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

fn exec_request_body(request: &HttpRequest) -> Result<&[u8], &'static str> {
    let content_type=request.headers.get("content-type").map(|value|value.split(';').next().unwrap_or("").trim());
    match content_type {
        Some("application/proto")=>Ok(&request.body),
        Some("application/connect+proto")=> {
            if request.body.len()<5 || request.body[0]!=0 {return Err("invalid Connect data envelope");}
            let size=u32::from_be_bytes(request.body[1..5].try_into().map_err(|_|"invalid Connect length")?) as usize;
            if size==0 || size>16*1024*1024 || size!=request.body.len()-5 {return Err("invalid Connect data length");}
            Ok(&request.body[5..])
        }
        _=>Err("unsupported Exec content type"),
    }
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
    if content_length>16*1024*1024 {return Err("HTTP request body exceeds 16MiB".into());}
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


fn close_background_input(process: &BackgroundProcess) -> bool {
    let mut input = match process.stdin.try_lock() {
        Ok(input) => input,
        Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => return false,
    };
    // Existing input calls retain their own counted Arc. Closing this owner's
    // reference cannot detach an in-flight write or close another invocation.
    drop(input.take());
    true
}

fn advance_background_termination(state: &mut BackgroundState) {
    if state.phase == BackgroundPhase::Settled { return; }
    if state.termination_started.is_none() {
        state.termination_started = Some(Instant::now());
        if state.resources.state.signal(false).is_err() {
            state.failure.get_or_insert("background child termination failed");
        }
    }
    if !state.force_sent && state.termination_started.is_some_and(|started|
        started.elapsed() >= FOREGROUND_TERM_GRACE)
    {
        state.force_sent = true;
        if state.resources.state.signal(true).is_err() {
            state.failure.get_or_insert("background child termination failed");
        }
    }
}

fn observe_background_readers(state: &mut BackgroundState) {
    let failure = *state.resources.recovery.failure.lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(phase) = failure {
        state.failure.get_or_insert(phase);
    }
    let recovery = &mut state.resources.recovery;
    for worker in [&mut recovery.stdout_worker, &mut recovery.stderr_worker] {
        if worker.as_ref().is_some_and(|worker| worker.is_finished()) {
            match worker.take().expect("finished background reader").join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => { state.failure.get_or_insert(error.phase); }
                Err(_) => { state.failure.get_or_insert("background capture worker lost"); }
            }
        }
    }
}

fn terminal_error_footer(phase: &'static str, started_at_ms: i64) -> String {
    let ended = Utc::now();
    format!(
        "\n---\nerror: {phase}\nelapsed_ms: {}\nended_at: {}\n---\n",
        ended.timestamp_millis().saturating_sub(started_at_ms),
        ended.to_rfc3339_opts(SecondsFormat::Millis, true),
    )
}

// Only the live controller, or an observer that has observed its termination,
// may drive completion. Every effect precedes the final exact registry removal.
fn drive_background(
    processes: &Arc<Mutex<HashMap<u32, Arc<BackgroundProcess>>>>,
    shell_id: u32,
    process: &Arc<BackgroundProcess>,
    controller: bool,
) -> bool {
    let mut state = match process.state.try_lock() {
        Ok(state) => state,
        Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => return false,
    };
    if state.phase == BackgroundPhase::Settled { return true; }
    if matches!(state.phase, BackgroundPhase::Setup | BackgroundPhase::Admitting) { return false; }
    if !controller {
        if state.controller.as_ref().is_some_and(|worker| !worker.is_finished()) {
            return false;
        }
        if let Some(worker) = state.controller.take() {
            if worker.join().is_err() {
                state.failure.get_or_insert("background controller lost");
            } else {
                state.failure.get_or_insert("background controller exited before completion");
            }
            state.phase = BackgroundPhase::Recovery;
        }
    }
    observe_background_readers(&mut state);
    #[cfg(test)]
    if !state.wait_fault_fired && state.probe.as_ref().is_some_and(|probe| probe.wait_fault_ready()) {
        state.wait_fault_fired = true;
        state.failure.get_or_insert("background child wait failed");
    }
    match state.resources.state.poll() {
        Ok(_) => {}
        Err(_) => { state.failure.get_or_insert("background child wait failed"); }
    }
    #[cfg(test)]
    if controller && state.resources.state.status.is_some() {
        match state.controller_fault.take() {
            Some(BackgroundControllerFault::PanicAfterReap) =>
                panic!("controlled background controller loss after actual reap"),
            Some(BackgroundControllerFault::ExitAfterReap) => {
                // The controller wrapper returns without committing terminal.
                state.controller_exit_requested = true;
                return false;
            }
            fault => { state.controller_fault = fault; }
        }
    }
    let deadline_elapsed = state.hard_timeout_ms.is_some_and(|timeout|
        state.started.elapsed() >= Duration::from_millis(timeout));
    if process.stop_requested.load(Ordering::Acquire) || deadline_elapsed || state.failure.is_some() {
        advance_background_termination(&mut state);
        close_background_input(process);
    }
    if state.resources.state.status.is_none() { return false; }
    if !close_background_input(process) { return false; }
    // Any pipe whose reader was never started remains owned through reap.
    drop(state.resources.state.child.stdin.take());
    drop(state.resources.state.child.stdout.take());
    drop(state.resources.state.child.stderr.take());
    if state.resources.recovery.stdout_worker.is_some()
        || state.resources.recovery.stderr_worker.is_some()
    {
        return false;
    }
    state.resources.state.capture_complete = true;
    if state.footer_failed { return false; }
    if state.admitted && !state.footer_written {
        let mut writer = match process.writer.try_lock() {
            Ok(writer) => writer,
            Err(std::sync::TryLockError::Poisoned(poisoned)) => {
                state.failure.get_or_insert("background output writer poisoned");
                poisoned.into_inner()
            }
            Err(std::sync::TryLockError::WouldBlock) => return false,
        };
        let status = state.resources.state.status.expect("observed background reap");
        let footer = match (state.failure, status.code()) {
            (None, Some(code)) => terminal_footer(code, state.started_at_ms),
            (failure, _) => terminal_error_footer(
                failure.unwrap_or("background child exited without a numeric status"),
                state.started_at_ms,
            ),
        };
        if writer.write_all(footer.as_bytes()).and_then(|_| writer.flush()).is_err() {
            state.footer_failed = true;
            state.failure.get_or_insert("background footer write failed");
            return false;
        }
        state.footer_written = true;
    }
    // Only unpublished terminals and still-owned temporary spools are listed.
    // Successful unlink retires its claim before another runtime can reuse it.
    for path in state.resources.recovery.cleanup_paths.clone() {
        state.resources.remove_created_file(&path);
    }
    if !state.resources.recovery.cleanup_paths.is_empty() {
        state.failure.get_or_insert("background owned-file cleanup failed");
        return false;
    }
    let mut processes = processes.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if process.active_input.load(Ordering::Acquire) != 0 { return false; }
    if matches!(processes.get(&shell_id), Some(current) if Arc::ptr_eq(current, process)) {
        state.phase = BackgroundPhase::Settled;
        processes.remove(&shell_id);
        // After this operation terminal: release settled handles and return.
        // No writer, signal, wait, or recovery action may follow.
        return true;
    }
    false
}

fn run_background_controller(
    processes: Arc<Mutex<HashMap<u32, Arc<BackgroundProcess>>>>,
    shell_id: u32,
    process: Arc<BackgroundProcess>,
) {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        loop {
            if drive_background(&processes, shell_id, &process, true) { return; }
            {
                let state = process.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                if state.footer_failed { return; }
                #[cfg(test)]
                if state.controller_exit_requested { return; }
            }
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
    }));
    if outcome.is_err() {
        let mut state = process.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.phase != BackgroundPhase::Settled {
            state.failure.get_or_insert("background controller lost");
            state.phase = BackgroundPhase::Recovery;
            process.stop_requested.store(true, Ordering::Release);
        }
    }
}

#[derive(Clone)]
struct CaptureControl {
    cancelled: Arc<AtomicBool>,
    #[cfg(test)]
    probe: Option<Arc<CaptureProbe>>,
    #[cfg(test)]
    pid: u32,
}

#[cfg(unix)]
fn prepare_capture_pipe<R: std::os::fd::AsRawFd>(
    reader: &R,
) -> Result<(), ShellExecutionUnknown> {
    let descriptor = reader.as_raw_fd();
    let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(ShellExecutionUnknown::new("nonblocking capture setup failed"));
    }
    Ok(())
}

#[cfg(not(unix))]
fn prepare_capture_pipe<R>(_reader: &R) -> Result<(), ShellExecutionUnknown> {
    // The existing daemon launches /bin/sh. Do not pretend that an unsupported
    // pipe cancellation primitive provides the Unix foreground contract.
    Err(ShellExecutionUnknown::new("cancellable pipe capture unavailable"))
}

fn spawn_capture_worker<R: Read + Send + 'static>(
    reader: R,
    control: CaptureControl,
    stream: &'static str,
) -> Result<JoinHandle<Result<Vec<u8>, ShellExecutionUnknown>>, ShellExecutionUnknown> {
    thread::Builder::new().name(format!("box-capture-{stream}"))
        .spawn(move || read_capture(reader, &control, stream))
        .map_err(|_| ShellExecutionUnknown::new("capture worker creation failed"))
}

fn read_capture<R: Read>(
    mut reader: R,
    control: &CaptureControl,
    stream: &'static str,
) -> Result<Vec<u8>, ShellExecutionUnknown> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        if control.cancelled.load(Ordering::Acquire) {
            return Err(ShellExecutionUnknown::new("capture cancelled before EOF"));
        }
        match read_capture_chunk(&mut reader, &mut buffer, control, stream) {
            Ok(0) => return Ok(output),
            Ok(count) => {
                output.extend_from_slice(&buffer[..count]);
                #[cfg(test)]
                if let Some(probe) = control.probe.as_ref() {
                    probe.after_read(stream, &output, control.pid)?;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(FOREGROUND_POLL_INTERVAL);
            }
            Err(_) => return Err(ShellExecutionUnknown::new(
                if stream == "stdout" { "stdout read failed" } else { "stderr read failed" },
            )),
        }
    }
}

fn read_capture_chunk<R: Read>(
    reader: &mut R,
    buffer: &mut [u8],
    _control: &CaptureControl,
    _stream: &str,
) -> std::io::Result<usize> {
    #[cfg(test)]
    if let Some(probe) = _control.probe.as_ref() {
        probe.before_read(_stream)?;
    }
    reader.read(buffer)
}

#[cfg(test)]
fn publish_capture_probe_json(path: &Path, value: &serde_json::Value) -> std::io::Result<()> {
    let staging = path.with_extension("staging");
    let mut file = OpenOptions::new().write(true).create_new(true).open(&staging)?;
    file.write_all(value.to_string().as_bytes())?;
    drop(file);
    fs::rename(staging, path)
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum CaptureFaultKind {
    Read,
    Worker,
    Wait,
    BackgroundSetup,
    BackgroundController,
}

#[cfg(test)]
struct CaptureProbe {
    mode: String,
    stream: &'static str,
    kind: CaptureFaultKind,
    observed: AtomicBool,
    observation_path: PathBuf,
}

#[cfg(test)]
impl CaptureProbe {
    fn new(mode: &str, observation_path: PathBuf) -> Option<Self> {
        let (stream, kind) = match mode {
            "none" => return None,
            "stdout-read" => ("stdout", CaptureFaultKind::Read),
            "stderr-read" => ("stderr", CaptureFaultKind::Read),
            "stdout-worker" => ("stdout", CaptureFaultKind::Worker),
            "stderr-worker" => ("stderr", CaptureFaultKind::Worker),
            "stdout-wait" => ("stdout", CaptureFaultKind::Wait),
            "stderr-wait" => ("stderr", CaptureFaultKind::Wait),
            "background-setup" => ("stdout", CaptureFaultKind::BackgroundSetup),
            "background-controller" => ("stdout", CaptureFaultKind::BackgroundController),
            _ => panic!("unsupported cfg(test) capture fault"),
        };
        Some(Self {
            mode: mode.into(), stream, kind, observed: AtomicBool::new(false), observation_path,
        })
    }

    fn after_read(
        &self,
        stream: &str,
        output: &[u8],
        pid: u32,
    ) -> Result<(), ShellExecutionUnknown> {
        const PREFIX: &[u8] = b"SC_CAPTURE_PREFIX";
        if stream != self.stream || !output.windows(PREFIX.len()).any(|window| window == PREFIX) {
            return Ok(());
        }
        if self.observed.load(Ordering::Acquire) {
            return Ok(());
        }
        // Only this stream's single reader can publish the observation. Store
        // the release flag after the complete fixture observation is visible.
        let observation = serde_json::json!({
            "mode": self.mode, "stream": stream, "prefix": "SC_CAPTURE_PREFIX",
            "bytesObserved": output.len(), "childPid": pid,
        });
        publish_capture_probe_json(&self.observation_path, &observation)
            .map_err(|_| ShellExecutionUnknown::new("test observation write failed"))?;
        self.observed.store(true, Ordering::Release);
        Ok(())
    }

    fn before_read(&self, stream: &str) -> std::io::Result<()> {
        if stream == self.stream && self.observed.load(Ordering::Acquire) {
            match self.kind {
                CaptureFaultKind::Read => return Err(std::io::Error::other("cfg(test) capture read loss")),
                CaptureFaultKind::Worker => panic!("cfg(test) capture worker loss after real prefix"),
                CaptureFaultKind::Wait | CaptureFaultKind::BackgroundSetup
                    | CaptureFaultKind::BackgroundController => {}
            }
        }
        Ok(())
    }

    fn wait_fault_ready(&self) -> bool {
        self.kind == CaptureFaultKind::Wait && self.observed.load(Ordering::Acquire)
    }
}

fn record_stream_failure(failure: &Mutex<Option<&'static str>>, phase: &'static str) {
    failure.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).get_or_insert(phase);
}

fn start_stream_reader(
    resources: &mut StreamResources,
    events: Option<mpsc::Sender<ShellPipeEvent>>,
    stderr: bool,
    writer: Arc<Mutex<File>>,
    control: CaptureControl,
) -> Result<(), ShellExecutionUnknown> {
    let failure = Arc::clone(&resources.recovery.failure);
    let worker = if stderr {
        prepare_capture_pipe(resources.state.child.stderr.as_ref()
            .ok_or_else(|| ShellExecutionUnknown::new("stderr pipe unavailable"))?)?;
        spawn_stream_pipe(&mut resources.state.child.stderr, events, true, writer, control, failure)?
    } else {
        prepare_capture_pipe(resources.state.child.stdout.as_ref()
            .ok_or_else(|| ShellExecutionUnknown::new("stdout pipe unavailable"))?)?;
        spawn_stream_pipe(&mut resources.state.child.stdout, events, false, writer, control, failure)?
    };
    if stderr {
        resources.recovery.stderr_worker = Some(worker);
    } else {
        resources.recovery.stdout_worker = Some(worker);
    }
    Ok(())
}

fn spawn_stream_pipe<R>(
    reader: &mut Option<R>,
    sender: Option<mpsc::Sender<ShellPipeEvent>>,
    stderr: bool,
    output_writer: Arc<Mutex<File>>,
    control: CaptureControl,
    failure: Arc<Mutex<Option<&'static str>>>,
) -> Result<JoinHandle<Result<(), ShellExecutionUnknown>>, ShellExecutionUnknown>
where
    R: Read + Send + 'static,
{
    if reader.is_none() {
        return Err(ShellExecutionUnknown::new("stream pipe unavailable"));
    }
    // Thread creation never captures the only pipe. Its original owner retains
    // the pipe until a receiver exists, and receives it back on send refusal.
    let (pipe_tx, pipe_rx) = mpsc::channel::<R>();
    let stream = if stderr { "stderr" } else { "stdout" };
    let worker = thread::Builder::new().name(format!("box-stream-{stream}")).spawn(move || {
        let mut reader = pipe_rx.recv()
            .map_err(|_| ShellExecutionUnknown::new("stream pipe admission failed"))?;
        let mut buffer = [0_u8; 8192];
        #[cfg(test)]
        let mut observed = Vec::new();
        loop {
            match read_capture_chunk(&mut reader, &mut buffer, &control, stream) {
                Ok(0) => {
                    let failure = *failure.lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    return failure.map_or(Ok(()), |phase| Err(ShellExecutionUnknown::new(phase)));
                }
                Ok(count) => {
                    let bytes = &buffer[..count];
                    {
                        let mut writer = match output_writer.lock() {
                            Ok(writer) => writer,
                            Err(poisoned) => {
                                record_stream_failure(&failure, "stream output writer poisoned");
                                poisoned.into_inner()
                            }
                        };
                        if writer.write_all(bytes).and_then(|_| writer.flush()).is_err() {
                            // Keep draining the actual pipe. This sticky failure
                            // forbids a background success footer or clean handoff.
                            record_stream_failure(&failure, "stream output write failed");
                        }
                    }
                    #[cfg(test)]
                    if let Some(probe) = control.probe.as_ref() {
                        observed.extend_from_slice(bytes);
                        probe.after_read(stream, &observed, control.pid)?;
                    }
                    if let Some(sender) = sender.as_ref() {
                        let event = if stderr {
                            ShellPipeEvent::Stderr(bytes.to_vec())
                        } else {
                            ShellPipeEvent::Stdout(bytes.to_vec())
                        };
                        // Timed handoff removes the stream receiver; file
                        // capture remains owned until its actual EOF.
                        let _ = sender.send(event);
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(FOREGROUND_POLL_INTERVAL);
                }
                Err(_) => {
                    let phase = if stderr { "stderr read failed" } else { "stdout read failed" };
                    record_stream_failure(&failure, phase);
                    return Err(ShellExecutionUnknown::new(phase));
                }
            }
        }
    }).map_err(|_| ShellExecutionUnknown::new("stream reader creation failed"))?;
    if let Err(mpsc::SendError(pipe)) = pipe_tx.send(reader.take().expect("owned stream pipe")) {
        *reader = Some(pipe);
        let _ = worker.join();
        return Err(ShellExecutionUnknown::new("stream pipe admission failed"));
    }
    Ok(worker)
}

fn path_within(root: &Path, target: &Path) -> bool {
    target == root || target.strip_prefix(root).is_ok()
}

fn saturating_i32(value: u64) -> i32 {
    value.min(i32::MAX as u64) as i32
}

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


    /// Owned fixture entrypoint in the actual daemon *test executable*.
    /// No production entrypoint reads these environment variables.
    #[test]
    #[ignore = "owned child fixture for Host foreground capture contracts"]
    fn foreground_shell_capture_probe_child() {
        let root = PathBuf::from(std::env::var_os("FABUSHI_SC_PROBE_ROOT")
            .expect("probe root is required"));
        let auth = std::env::var("FABUSHI_SC_PROBE_AUTH").expect("probe auth is required");
        assert!(!auth.is_empty(), "probe auth must not be empty");
        let mode = std::env::var("FABUSHI_SC_PROBE_FAULT").unwrap_or_else(|_| "none".into());
        let port = std::env::var("FABUSHI_SC_PROBE_PORT").ok()
            .map(|value| value.parse::<u16>().expect("probe port")).unwrap_or(0);
        fs::create_dir_all(&root).expect("probe root");
        let handle = start_box_exec_daemon(BoxExecDaemonOptions {
            host: Some("127.0.0.1".into()), port: Some(port), auth_token: Some(auth),
            workspace_root: root.join("workspace"),
            terminals_directory: Some(root.join("terminals")),
            environment: Some(std::env::vars().collect()),
        }).expect("owned probe daemon");
        *handle.runtime.capture_probe.lock().expect("probe") =
            CaptureProbe::new(&mode, root.join("fault-observed.json")).map(Arc::new);
        let pid = std::process::id();
        publish_capture_probe_json(&root.join("ready.json"), &serde_json::json!({
            "pid": pid, "host": "127.0.0.1", "port": handle.port,
            "workspaceRoot": handle.workspace_root,
            "terminalsDirectory": handle.terminals_directory,
        })).expect("probe readiness");
        let mut buffer = [0_u8; 64];
        let mut stdin = std::io::stdin().lock();
        loop {
            match stdin.read(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => panic!("probe parent pipe failed: {error}"),
            }
        }
        handle.stop().expect("owned probe graceful stop");
        let remaining = handle.runtime.foreground_processes.lock().expect("foreground registry").len();
        assert_eq!(remaining, 0, "probe retained unresolved foreground ownership");
        let background_remaining = handle.runtime.background.lock().expect("background registry").len();
        assert_eq!(background_remaining, 0, "probe retained unresolved background ownership");
        publish_capture_probe_json(&root.join("stopped.json"), &serde_json::json!({
            "pid": pid, "clean": true, "foregroundCount": remaining,
            "backgroundCount": background_remaining,
        })).expect("probe stop receipt");
    }

    fn foreground_args(command: &str) -> ShellArgs {
        ShellArgs {
            command: command.into(), working_directory: "/workspace".into(),
            timeout: 5_000, tool_call_id: "capture-contract".into(),
            is_background: false, skip_approval: false, timeout_behavior: 0,
            hard_timeout: None,
        }
    }

    fn foreground_request(args: ShellArgs, mini: bool) -> ExecServerMessage {
        ExecServerMessage {
            id: 73, exec_id: "capture-wire".into(),
            message: Some(if mini {
                exec_server_message::Message::MiniSweAgentBashArgs(args)
            } else {
                exec_server_message::Message::ShellArgs(args)
            }),
        }
    }

    fn assert_capture_throw(output: &[ExecStreamElement]) {
        assert_eq!(output.len(), 2, "unknown must have only Throw and Close");
        match &output[0].element {
            Some(exec_stream_element::Element::ExecClientControlMessage(control)) =>
                match &control.message {
                    Some(exec_client_control_message::Message::Throw(thrown)) => {
                        assert_eq!(thrown.id, 73);
                        assert_eq!(thrown.error_code.as_deref(), Some("BOX_EXEC_DAEMON_ERROR"));
                        assert!(thrown.error.starts_with("foreground shell outcome unknown: "));
                        assert!(!thrown.error.contains("SC_CAPTURE_PREFIX"));
                        assert!(!thrown.error.contains("effect-count"));
                    }
                    other => panic!("expected Throw, got {other:?}"),
                },
            other => panic!("expected control, got {other:?}"),
        }
        assert!(matches!(&output[1].element,
            Some(exec_stream_element::Element::ExecClientControlMessage(
                ExecClientControlMessage {
                    message: Some(exec_client_control_message::Message::StreamClose(
                        ExecClientStreamClose { id: 73 },
                    )),
                },
            ))
        ));
    }

    #[test]
    fn foreground_capture_faults_preserve_unknown_for_both_dispatches() {
        for mini in [false, true] {
            for mode in ["stdout-read", "stderr-read", "stdout-worker", "stderr-worker",
                "stdout-wait", "stderr-wait"]
            {
                let (root, runtime) = runtime();
                let observation = root.path().join("fault-observed.json");
                *runtime.capture_probe.lock().expect("probe") =
                    CaptureProbe::new(mode, observation.clone()).map(Arc::new);
                let redirect = if mode.starts_with("stderr") { " >&2" } else { "" };
                let command = format!(
                    "printf 'effect\\n' >> effect-count; printf '%s\\n' \"$$\" > child-pid; \
                     printf SC_CAPTURE_PREFIX{redirect}; sleep 30",
                );
                let started = Instant::now();
                let output = runtime.execute(foreground_request(foreground_args(&command), mini));
                assert_capture_throw(&output);
                assert!(started.elapsed() < Duration::from_secs(5), "unbounded cleanup: {mode}");
                let record: serde_json::Value = serde_json::from_slice(
                    &fs::read(&observation).expect("real prefix observed"),
                ).expect("observation JSON");
                assert_eq!(record["mode"], mode);
                assert_eq!(record["prefix"], "SC_CAPTURE_PREFIX");
                assert!(record["bytesObserved"].as_u64().unwrap() >= 17);
                assert_eq!(fs::read_to_string(runtime.workspace_root.join("effect-count")).unwrap(), "effect\n");
                assert!(runtime.foreground_processes.lock().unwrap().is_empty(), "{mode}");
                runtime.stop().expect("no unresolved capture owner");
            }
        }
    }

    #[test]
    fn foreground_capture_complete_results_preserve_text_failure_and_refusal() {
        let (_root, runtime) = runtime();
        let result = runtime.shell(&foreground_args(
            "printf 'complete\\377'; printf 'diagnostic' >&2",
        )).expect("observed complete capture");
        let Some(shell_result::Result::Success(result)) = result.result else {
            panic!("success control");
        };
        assert_eq!(result.stdout, "complete\u{fffd}");
        assert_eq!(result.stderr, "diagnostic");
        let result = runtime.shell(&foreground_args(
            "printf 'partial is complete here'; printf 'failure' >&2; exit 7",
        )).expect("observed nonzero exit");
        let Some(shell_result::Result::Failure(result)) = result.result else {
            panic!("nonzero control");
        };
        assert_eq!(result.exit_code, 7);
        assert_eq!(result.stdout, "partial is complete here");
        assert_eq!(result.stderr, "failure");
        assert!(!result.aborted);
        let mut refused = foreground_args("printf should-not-run > refused-effect");
        refused.working_directory = "/workspace/absent-directory".into();
        assert!(matches!(runtime.shell(&refused).expect("known pre-spawn refusal").result,
            Some(shell_result::Result::SpawnError(_))));
        assert!(!runtime.workspace_root.join("refused-effect").exists());
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
    }

    #[test]
    fn foreground_capture_timeout_and_abort_escalate_owned_term_ignoring_child() {
        let (_root, runtime) = runtime();
        let mut timed = foreground_args("trap '' TERM; printf timeout-prefix; while :; do sleep 1; done");
        timed.timeout = 100;
        let started = Instant::now();
        assert!(matches!(runtime.shell(&timed).expect("timeout observed").result,
            Some(shell_result::Result::Timeout(_))));
        assert!(started.elapsed() < Duration::from_secs(5));
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());

        let abort = AtomicBool::new(true);
        let started = Instant::now();
        let result = runtime.shell_with_abort(
            &foreground_args("trap '' TERM; while :; do sleep 1; done"), &abort,
        ).expect("abort observed");
        assert!(matches!(result.result,
            Some(shell_result::Result::Failure(ShellFailure { aborted: true, .. }))));
        assert!(started.elapsed() < Duration::from_secs(5));
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
    }

    #[test]
    fn foreground_capture_stop_race_retains_owner_until_reap_and_never_resignals() {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let worker_runtime = Arc::clone(&runtime);
        let worker = thread::spawn(move || worker_runtime.shell(&foreground_args(
            "trap '' TERM; printf SC_CAPTURE_PREFIX > started; while :; do sleep 1; done",
        )));
        let deadline = Instant::now() + Duration::from_secs(3);
        while !runtime.workspace_root.join("started").exists() {
            assert!(Instant::now() < deadline, "real child did not start");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        let owner = runtime.foreground_processes.lock().unwrap().values().find_map(|(_, process)| {
            match process {
                ForegroundProcess::Capture(owner) => Some(Arc::clone(owner)),
                ForegroundProcess::Stream => None,
            }
        }).expect("owned child registry");
        runtime.stop().expect("bounded stop and joined capture");
        let result = worker.join().expect("foreground caller").expect("observed abort");
        assert!(matches!(result.result,
            Some(shell_result::Result::Failure(ShellFailure { aborted: true, .. }))));
        let mut state = owner.lock().unwrap();
        assert!(state.status.is_some());
        assert!(state.capture_complete);
        let signals = state.signals_sent;
        state.signal(true).expect("already reaped is a no-op");
        assert_eq!(state.signals_sent, signals, "never signal a reaped PID");
        drop(state);
        runtime.stop().expect("repeated stop is observation");
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        assert!(matches!(runtime.shell(&foreground_args(
            "printf forbidden > after-stop",
        )).expect("stopped owner refuses before spawn").result,
            Some(shell_result::Result::SpawnError(_))));
        assert!(!runtime.workspace_root.join("after-stop").exists());
    }

    #[test]
    fn foreground_capture_inherited_pipe_refuses_incomplete_eof_without_hanging() {
        let (_root, runtime) = runtime();
        let complete = runtime.shell(&foreground_args(
            "(sleep 2; printf delayed) & printf initial; exit 0",
        )).expect("normal delayed descendant output retains request timeout");
        let Some(shell_result::Result::Success(complete)) = complete.result else {
            panic!("delayed capture success");
        };
        assert_eq!(complete.stdout, "initialdelayed");
        // This bounded descendant keeps real capture pipes open after the
        // direct child exits. Explicit request timeout authorizes cancellation;
        // absent that authority normal delayed output above remains complete.
        let mut args = foreground_args(
            "(trap '' TERM; sleep 3) & printf SC_CAPTURE_PREFIX; exit 0",
        );
        args.timeout = 100;
        let started = Instant::now();
        let result = runtime.shell(&args);
        let error = result.expect_err("explicit timeout cannot certify missing EOF");
        assert_eq!(error.phase, "capture cancelled before EOF");
        assert!(started.elapsed() < Duration::from_secs(2));
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        // Let the deliberately bounded fixture descendant end before deleting
        // its workspace; its PID is never used as cleanup authority.
        while started.elapsed() < Duration::from_millis(3_100) {
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
    }


    // A private gate pauses the real run_process caller before it observes its
    // real capture workers. Tests observe the same owned Child's actual reap.
    // Neither Child.id nor signal authority is changed by the PID alias seam.
    struct RetainedCaptureFixture {
        registration_id: ForegroundId,
        owner: Arc<Mutex<CapturedChild>>,
        release: mpsc::Sender<()>,
        worker: Option<JoinHandle<Result<ShellResult, ShellExecutionUnknown>>>,
    }

    impl RetainedCaptureFixture {
        fn new(runtime: &Arc<BoxExecRuntime>) -> Self {
            let previous = runtime.foreground_processes.lock().unwrap()
                .keys().copied().collect::<Vec<_>>();
            let (release, gate) = mpsc::channel();
            *runtime.capture_observation_gate.lock().unwrap() = Some(gate);
            let worker_runtime = Arc::clone(runtime);
            let worker = thread::spawn(move || worker_runtime.shell(&foreground_args(
                "printf capture-stdout; printf capture-stderr >&2",
            )));
            let deadline = Instant::now() + Duration::from_secs(3);
            let (registration_id, owner) = loop {
                let entry = runtime.foreground_processes.lock().unwrap().iter()
                    .find_map(|(id, (_, process))| match process {
                        ForegroundProcess::Capture(owner) if !previous.contains(id) =>
                            Some((*id, Arc::clone(owner))),
                        _ => None,
                    });
                if let Some(entry) = entry { break entry; }
                assert!(Instant::now() < deadline, "real capture invocation not registered");
                thread::sleep(FOREGROUND_POLL_INTERVAL);
            };
            loop {
                if owner.lock().unwrap().poll().expect("actual fixture child wait").is_some() {
                    break;
                }
                assert!(Instant::now() < deadline, "fixture child did not exit");
                thread::sleep(FOREGROUND_POLL_INTERVAL);
            }
            assert!(!owner.lock().unwrap().capture_complete);
            assert!(!worker.is_finished(), "real caller has not observed its workers");
            Self { registration_id, owner, release, worker: Some(worker) }
        }

        fn complete(&mut self, runtime: &BoxExecRuntime) {
            self.release.send(()).expect("release actual capture caller");
            let result = self.worker.take().unwrap().join().expect("capture caller joined")
                .expect("real pipes and reap completely observed");
            let (stdout, stderr) = match result.result {
                Some(shell_result::Result::Success(result)) => (result.stdout, result.stderr),
                Some(shell_result::Result::Failure(result)) => {
                    assert!(result.aborted, "stop preserves the observed abort shape");
                    (result.stdout, result.stderr)
                }
                _ => panic!("complete fixture capture retains its actual text"),
            };
            assert_eq!(stdout, "capture-stdout");
            assert_eq!(stderr, "capture-stderr");
            let state = self.owner.lock().unwrap();
            assert!(state.status.is_some(), "actual child reap remains observed");
            assert!(state.capture_complete, "actual request observed both capture workers");
            assert_eq!(state.signals_sent, 0, "reaped fixture must never be signaled");
            drop(state);
            assert!(!runtime.foreground_processes.lock().unwrap()
                .contains_key(&self.registration_id), "actual completion removed its identity");
        }
    }

    #[test]
    fn foreground_capture_reused_pid_retains_each_invocation_until_observed() {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let mut older = RetainedCaptureFixture::new(&runtime);
        let old_pid = older.owner.lock().unwrap().child.id();
        *runtime.foreground_pid_alias.lock().unwrap() = Some(old_pid);
        let mut successor = RetainedCaptureFixture::new(&runtime);
        assert_ne!(older.registration_id, successor.registration_id);
        {
            let processes = runtime.foreground_processes.lock().unwrap();
            assert_eq!(processes.len(), 2, "reused PID cannot replace unfinished capture");
            assert_eq!(processes.get(&older.registration_id).unwrap().0, old_pid);
            assert_eq!(processes.get(&successor.registration_id).unwrap().0, old_pid);
        }
        let started = Instant::now();
        assert!(runtime.stop().is_err(), "unobserved readers prevent clean stop");
        assert!(started.elapsed() < Duration::from_secs(4));
        assert_eq!(runtime.foreground_processes.lock().unwrap().len(), 2);
        older.complete(&runtime);
        // Neither a repeated older completion nor a mismatched owner may erase
        // the newer invocation whose observational PID is the same.
        runtime.remove_completed_capture(older.registration_id, &older.owner);
        runtime.remove_completed_capture(successor.registration_id, &older.owner);
        {
            let processes = runtime.foreground_processes.lock().unwrap();
            assert_eq!(processes.len(), 1);
            assert!(processes.contains_key(&successor.registration_id));
        }
        let started = Instant::now();
        assert!(runtime.stop().is_err(), "the successor still has unobserved readers");
        assert!(started.elapsed() < Duration::from_secs(4));
        successor.complete(&runtime);
        runtime.stop().expect("both actual reaps and both pairs of readers observed");
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        for owner in [&older.owner, &successor.owner] {
            let mut state = owner.lock().unwrap();
            state.signal(true).expect("late signal is an observed-reap no-op");
            assert_eq!(state.signals_sent, 0);
        }
    }

    #[test]
    fn foreground_capture_identity_exhaustion_refuses_before_capture_or_stream_spawn() {
        let (_root, runtime) = runtime();
        runtime.next_foreground_id.store(u64::MAX - 1, Ordering::Relaxed);
        let accepted = runtime.shell(&foreground_args("printf accepted > accepted-effect"))
            .expect("last available invocation");
        assert!(matches!(accepted.result, Some(shell_result::Result::Success(_))));
        assert_eq!(fs::read_to_string(runtime.workspace_root.join("accepted-effect")).unwrap(),
            "accepted");
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        for mini in [false, true] {
            let output = runtime.execute(foreground_request(
                foreground_args("printf forbidden > exhausted-effect"), mini,
            ));
            assert_eq!(output.len(), 2);
            let Some(exec_stream_element::Element::ExecClientMessage(message)) =
                &output[0].element else { panic!("known pre-spawn refusal"); };
            let result = match &message.message {
                Some(exec_client_message::Message::ShellResult(result)) |
                Some(exec_client_message::Message::MiniSweAgentBashResult(result)) => result,
                _ => panic!("foreground refusal retains its dispatch result"),
            };
            assert!(matches!(&result.result,
                Some(shell_result::Result::SpawnError(error))
                    if error.error == "foreground invocation id space exhausted"));
        }
        let mut events = Vec::new();
        let result = runtime.shell_stream_live(
            74, "exhaustion", &foreground_args("printf forbidden > exhausted-stream-effect"),
            &AtomicBool::new(false), &mut |event| { events.push(event); Ok(()) },
        );
        assert_eq!(result.unwrap_err(), "foreground invocation id space exhausted");
        assert_eq!(events.len(), 1, "only the existing stream Start precedes refusal");
        assert!(matches!(&events[0].element,
            Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                message: Some(exec_client_message::Message::ShellStream(ShellStream {
                    event: Some(shell_stream::Event::Start(_)),
                })), ..
            }))));
        assert!(!runtime.workspace_root.join("exhausted-effect").exists());
        assert!(!runtime.workspace_root.join("exhausted-stream-effect").exists());
        assert_eq!(runtime.next_foreground_id.load(Ordering::Relaxed), u64::MAX);
        assert!(runtime.allocate_foreground_id().is_err(), "exhaustion never wraps");
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        runtime.stop().expect("no child entered execution after exhaustion");
    }

    #[test]
    fn foreground_capture_stream_stop_preserves_identity_and_never_signals_alias() {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let mut older = RetainedCaptureFixture::new(&runtime);
        let old_pid = older.owner.lock().unwrap().child.id();
        *runtime.foreground_pid_alias.lock().unwrap() = Some(old_pid);
        let (ready_tx, ready_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker_runtime = Arc::clone(&runtime);
        let worker = thread::spawn(move || {
            let mut args = foreground_args(
                "trap 'printf stopped > stream-stopped; exit 0' TERM; \
                 printf stream-prefix; while :; do sleep 1; done",
            );
            args.timeout = 1_000;
            args.timeout_behavior = TIMEOUT_BEHAVIOR_BACKGROUND;
            let mut held = false;
            let mut events = Vec::new();
            worker_runtime.shell_stream_live(
                75, "identity-stop", &args, &AtomicBool::new(false), &mut |event| {
                    if !held && matches!(&event.element,
                        Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                            message: Some(exec_client_message::Message::ShellStream(ShellStream {
                                event: Some(shell_stream::Event::Stdout(_)),
                            })), ..
                        })))
                    {
                        held = true;
                        ready_tx.send(()).map_err(|error| error.to_string())?;
                        release_rx.recv_timeout(Duration::from_secs(10))
                            .map_err(|error| error.to_string())?;
                    }
                    events.push(event);
                    Ok(())
                },
            )?;
            Ok::<_, String>(events)
        });
        ready_rx.recv_timeout(Duration::from_secs(3)).expect("live stream emitted prefix");
        let stream_id = {
            let processes = runtime.foreground_processes.lock().unwrap();
            assert_eq!(processes.len(), 2, "stream insertion retains older capture");
            processes.iter().find_map(|(id, (pid, process))| {
                if matches!(process, ForegroundProcess::Stream) {
                    assert_eq!(*pid, old_pid);
                    Some(*id)
                } else { None }
            }).expect("stream invocation registered")
        };
        older.complete(&runtime);
        runtime.remove_completed_capture(older.registration_id, &older.owner);
        assert!(runtime.foreground_processes.lock().unwrap().contains_key(&stream_id));

        // An observational PID alias must not authorize signaling any other
        // live Child. This owned sentinel is deliberately outside the registry.
        struct Sentinel(Arc<Mutex<CapturedChild>>);
        impl Drop for Sentinel {
            fn drop(&mut self) { let _ = cleanup_captured_child(&self.0); }
        }
        let sentinel = Sentinel(Arc::new(Mutex::new(CapturedChild {
            child: runtime.spawn_shell("sleep 30", &runtime.workspace_root).unwrap(),
            status: None, cleanup_unknown: false, capture_complete: true,
            stream_recovery: None, signals_sent: 0,
        })));
        let sentinel_pid = {
            let mut state = sentinel.0.lock().unwrap();
            assert!(state.poll().unwrap().is_none());
            state.child.id()
        };
        runtime.foreground_processes.lock().unwrap().get_mut(&stream_id).unwrap().0 = sentinel_pid;
        let started = Instant::now();
        assert!(runtime.stop().is_err(), "blocked stream completion is unresolved");
        assert!(started.elapsed() < Duration::from_secs(4));
        {
            let mut state = sentinel.0.lock().unwrap();
            assert!(state.poll().unwrap().is_none(), "stop must not signal the PID alias");
            assert_eq!(state.signals_sent, 0);
        }
        assert!(runtime.background.lock().unwrap().is_empty(), "stop forbids timed handoff");
        release_tx.send(()).expect("release actual stream request owner");
        let events = worker.join().expect("stream request joined").expect("stream stop observed");
        assert!(events.iter().any(|event| matches!(&event.element,
            Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                message: Some(exec_client_message::Message::ShellStream(ShellStream {
                    event: Some(shell_stream::Event::Exit(_)),
                })), ..
            })))));
        assert!(!events.iter().any(|event| matches!(&event.element,
            Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                message: Some(exec_client_message::Message::ShellStream(ShellStream {
                    event: Some(shell_stream::Event::Backgrounded(_)),
                })), ..
            })))));
        assert_eq!(fs::read_to_string(runtime.workspace_root.join("stream-stopped")).unwrap(),
            "stopped", "actual stream Child handled termination before exit");
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        assert_eq!(older.owner.lock().unwrap().signals_sent, 0);
        cleanup_captured_child(&sentinel.0).expect("sentinel cleanup uses its actual Child");
        {
            let mut state = sentinel.0.lock().unwrap();
            assert!(state.poll().unwrap().is_some());
            assert!(state.signals_sent > 0);
            let signals = state.signals_sent;
            state.signal(true).unwrap();
            assert_eq!(state.signals_sent, signals, "no signal after sentinel reap");
        }
        runtime.stop().expect("stream reap and workers observed");
        let mut events = Vec::new();
        let refused = runtime.shell_stream_live(
            76, "stopped", &foreground_args("printf forbidden > stream-after-stop"),
            &AtomicBool::new(false), &mut |event| { events.push(event); Ok(()) },
        );
        assert_eq!(refused.unwrap_err(), "box exec daemon is stopping");
        assert!(!runtime.workspace_root.join("stream-after-stop").exists());
    }

    #[test]
    fn foreground_capture_stream_handoff_retains_reused_pid_capture() {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let mut older = RetainedCaptureFixture::new(&runtime);
        let old_pid = older.owner.lock().unwrap().child.id();
        *runtime.foreground_pid_alias.lock().unwrap() = Some(old_pid);
        struct HandoffRelease {
            runtime: Arc<BoxExecRuntime>,
            shell_id: Option<u32>,
        }
        impl Drop for HandoffRelease {
            fn drop(&mut self) {
                if let Some(shell_id) = self.shell_id {
                    let _ = self.runtime.write_stdin(&WriteShellStdinArgs {
                        shell_id, chars: "finish\n".into(),
                    });
                    let deadline = Instant::now() + Duration::from_secs(3);
                    while self.runtime.background.lock().unwrap().contains_key(&shell_id)
                        && Instant::now() < deadline
                    {
                        thread::sleep(FOREGROUND_POLL_INTERVAL);
                    }
                }
                let background_empty = self.runtime.background.lock().unwrap().is_empty();
                if background_empty {
                    let _ = self.runtime.stop();
                }
            }
        }
        let mut release = HandoffRelease { runtime: Arc::clone(&runtime), shell_id: None };
        // The actual Child cannot exit independently before Backgrounded:
        // its stdin remains owned until this fixture explicitly releases it.
        let mut args = foreground_args(
            "printf handoff-prefix; IFS= read -r release; \
             [ \"$release\" = finish ] || exit 91; printf handoff-tail",
        );
        args.timeout = 20;
        args.timeout_behavior = TIMEOUT_BEHAVIOR_BACKGROUND;
        let mut events = Vec::new();
        runtime.shell_stream_live(
            77, "identity-handoff", &args, &AtomicBool::new(false),
            &mut |event| {
                if let Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                    message: Some(exec_client_message::Message::ShellStream(ShellStream {
                        event: Some(shell_stream::Event::Backgrounded(backgrounded)),
                    })), ..
                })) = &event.element {
                    release.shell_id = Some(backgrounded.shell_id);
                }
                events.push(event);
                Ok(())
            },
        ).expect("actual timed handoff");
        let backgrounded = events.iter().find_map(|event| match &event.element {
            Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                message: Some(exec_client_message::Message::ShellStream(ShellStream {
                    event: Some(shell_stream::Event::Backgrounded(backgrounded)),
                })), ..
            })) => Some(backgrounded),
            _ => None,
        }).expect("timed stream retained the same real child");
        {
            let processes = runtime.foreground_processes.lock().unwrap();
            assert_eq!(processes.len(), 1, "handoff removes only its own invocation");
            assert!(processes.contains_key(&older.registration_id));
            assert_eq!(processes.get(&older.registration_id).unwrap().0, old_pid);
        }
        let terminal = runtime.terminals_directory.join(format!("{}.txt", backgrounded.shell_id));
        older.complete(&runtime);
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        let released = runtime.write_stdin(&WriteShellStdinArgs {
            shell_id: backgrounded.shell_id, chars: "finish\n".into(),
        });
        assert!(matches!(released.result, Some(write_shell_stdin_result::Result::Success(_))));
        let deadline = Instant::now() + Duration::from_secs(4);
        while runtime.background.lock().unwrap().contains_key(&backgrounded.shell_id) {
            assert!(Instant::now() < deadline, "bounded background fixture did not settle");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        let output = fs::read_to_string(terminal).expect("actual handoff output");
        assert!(output.contains("handoff-prefix"));
        assert!(output.contains("handoff-tail"));
        assert!(output.contains("exit_code: 0"));
        assert_eq!(older.owner.lock().unwrap().signals_sent, 0);
        runtime.stop().expect("completed handoff and capture");
    }


    #[test]
    fn foreground_capture_stream_setup_error_retains_owned_child_for_cleanup() {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let (observed_tx, observed_rx) = mpsc::channel();
        *runtime.stream_recovery_observer.lock().unwrap() = Some(observed_tx);
        for attempt in 0..3 {
            let (release, gate) = mpsc::channel();
            *runtime.stream_setup_gate.lock().unwrap() = Some(gate);
            let worker_runtime = Arc::clone(&runtime);
            let worker = thread::spawn(move || {
                let mut events = Vec::new();
                let result = worker_runtime.shell_stream_live(
                    78, "setup-error",
                    &foreground_args(
                        "trap '' TERM; printf 'effect\\n' >> setup-effect; \
                         while :; do sleep 1; done",
                    ),
                    &AtomicBool::new(false), &mut |event| { events.push(event); Ok(()) },
                );
                (result, events)
            });
            let expected_effects = "effect\n".repeat(attempt + 1);
            let deadline = Instant::now() + Duration::from_secs(3);
            while fs::read_to_string(runtime.workspace_root.join("setup-effect"))
                .unwrap_or_default() != expected_effects
            {
                assert!(Instant::now() < deadline, "real post-spawn effect was not observed");
                thread::sleep(FOREGROUND_POLL_INTERVAL);
            }
            let registration_id = {
                let processes = runtime.foreground_processes.lock().unwrap();
                assert_eq!(processes.len(), 1, "previous completed errors must not accumulate");
                let (id, (_, process)) = processes.iter().next().unwrap();
                assert!(matches!(process, ForegroundProcess::Stream));
                *id
            };
            // A real filesystem failure, after successful spawn and a real effect.
            // The private gate controls ordering, not the I/O result.
            fs::remove_dir(&runtime.terminals_directory).expect("empty fixture terminals");
            fs::write(&runtime.terminals_directory, b"not a directory").unwrap();
            release.send(()).unwrap();
            let started = Instant::now();
            let (result, events) = worker.join().expect("stream request returned");
            assert!(result.is_err(), "actual spool setup failure remains unknown");
            assert!(started.elapsed() < Duration::from_secs(4), "bounded direct-child cleanup");
            assert_eq!(events.len(), 1, "only Start, no definitive Exit after setup failure");

            // Observe the actual state retained by the guard, even though its
            // completed registration has already been removed without stop.
            let (observed_id, owner) = observed_rx.recv_timeout(Duration::from_secs(1))
                .expect("actual cleanup owner observation");
            assert_eq!(observed_id, registration_id);
            assert!(runtime.foreground_processes.lock().unwrap().is_empty());
            assert!(!runtime.stopping.load(Ordering::Acquire), "cleanup must not require shutdown");
            assert_eq!(Arc::strong_count(&owner), 1, "only this observation retains the owner");
            {
                let mut state = owner.lock().unwrap();
                assert!(state.poll().unwrap().is_some(), "guard observed actual reap");
                assert!(state.signals_sent > 0, "guard terminated its entered Child");
                assert!(state.capture_complete, "worker-free recovery is observed immediately");
                assert!(state.stream_recovery.is_none(), "no completed recovery remains owned");
                assert!(state.child.stdin.is_none());
                assert!(state.child.stdout.is_none());
                assert!(state.child.stderr.is_none());
                let signals = state.signals_sent;
                state.signal(true).unwrap();
                assert_eq!(state.signals_sent, signals, "no signal after observed reap");
            }
            assert_eq!(fs::read_to_string(runtime.workspace_root.join("setup-effect")).unwrap(),
                expected_effects, "exactly one effect per failed invocation");
            fs::remove_file(&runtime.terminals_directory).unwrap();
            fs::create_dir(&runtime.terminals_directory).unwrap();
        }
        *runtime.stream_recovery_observer.lock().unwrap() = None;
        let result = runtime.shell(&foreground_args("printf still-active")).unwrap();
        assert!(matches!(result.result,
            Some(shell_result::Result::Success(result)) if result.stdout == "still-active"));
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        runtime.stop().expect("final fixture shutdown after live-runtime cleanup assertions");
    }

    #[derive(Clone, Copy)]
    enum StreamRecoveryBoundary { Stop, Shell, Stream }

    fn assert_stream_unwind_recovery(boundary: StreamRecoveryBoundary) {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        struct PipeRelease {
            runtime: Arc<BoxExecRuntime>,
        }
        impl Drop for PipeRelease {
            fn drop(&mut self) {
                let _ = fs::write(self.runtime.workspace_root.join("release-pipe"), b"release");
                let _ = self.runtime.stop();
            }
        }
        let _release = PipeRelease { runtime: Arc::clone(&runtime) };
        runtime.stream_panic_after_reap.store(true, Ordering::Release);
        let mut prefix_observed = false;
        let mut prefix_bytes = String::new();
        let mut events = Vec::new();
        // The direct Child exits only after its actual prefix reaches the
        // emitter. Its descendant retains both pipes until the owned explicit
        // release; the failure guard releases it before bounded cleanup.
        let args = foreground_args(
            "(trap '' TERM; while [ ! -f release-pipe ]; do sleep 0.01; done; \
             printf later-tail) & \
             printf 'effect\\n' > unwind-effect; printf unwind-prefix; \
             while [ ! -f prefix-observed ]; do sleep 0.01; done; exit 0",
        );
        // This is a controlled unwind at the real successful wait boundary,
        // not a claim to reproduce an operating-system wait failure.
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runtime.shell_stream_live(
                79, "worker-unwind", &args, &AtomicBool::new(false), &mut |event| {
                    if let Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                        message: Some(exec_client_message::Message::ShellStream(ShellStream {
                            event: Some(shell_stream::Event::Stdout(stdout)),
                        })), ..
                    })) = &event.element {
                        prefix_bytes.push_str(&stdout.data);
                        if prefix_bytes.contains("unwind-prefix") {
                            prefix_observed = true;
                            fs::write(runtime.workspace_root.join("prefix-observed"), b"observed")
                                .map_err(|error| error.to_string())?;
                        }
                    }
                    events.push(event);
                    Ok(())
                },
            )
        }));
        assert!(unwound.is_err());
        assert!(prefix_observed, "real pipe bytes precede the controlled unwind");
        assert_eq!(fs::read_to_string(runtime.workspace_root.join("unwind-effect")).unwrap(), "effect\n");
        assert!(!events.iter().any(|event| matches!(&event.element,
            Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                message: Some(exec_client_message::Message::ShellStream(ShellStream {
                    event: Some(shell_stream::Event::Exit(_)),
                })), ..
            })))));
        let (registration_id, owner) = {
            let processes = runtime.foreground_processes.lock().unwrap();
            assert_eq!(processes.len(), 1);
            let (id, (_, process)) = processes.iter().next().unwrap();
            let ForegroundProcess::Capture(owner) = process else {
                panic!("unwind retains real Child and readers");
            };
            (*id, Arc::clone(owner))
        };
        let spool = {
            let state = owner.lock().unwrap();
            assert!(state.status.is_some(), "real direct-child reap preceded unwind");
            assert_eq!(state.signals_sent, 0, "never signal the reaped direct child");
            assert!(!state.capture_complete);
            let recovery = state.stream_recovery.as_ref().expect("retained reader handles");
            assert!(recovery.stdout_worker.is_some());
            assert!(recovery.stderr_worker.is_some());
            recovery.cleanup_paths[0].clone()
        };
        match boundary {
            StreamRecoveryBoundary::Stop => {
                let started = Instant::now();
                assert!(runtime.stop().is_err(), "inherited pipes keep cleanup unresolved");
                assert!(started.elapsed() < Duration::from_secs(4));
            }
            StreamRecoveryBoundary::Shell | StreamRecoveryBoundary::Stream => {
                // A normal request before release must not complete the older
                // reaped invocation while either real reader still awaits EOF.
                let result = runtime.shell(&foreground_args("printf before-release")).unwrap();
                assert!(matches!(result.result,
                    Some(shell_result::Result::Success(result)) if result.stdout == "before-release"));
                assert!(!runtime.stopping.load(Ordering::Acquire));
                assert!(!owner.lock().unwrap().capture_complete);
            }
        }
        assert!(runtime.foreground_processes.lock().unwrap().contains_key(&registration_id));
        assert!(spool.exists(), "retain owned output until reader observation");
        assert_eq!(owner.lock().unwrap().signals_sent, 0);
        fs::write(runtime.workspace_root.join("release-pipe"), b"release").unwrap();
        match boundary {
            StreamRecoveryBoundary::Stop => {
                runtime.stop().expect("later actual EOF permits retained reader joins");
            }
            StreamRecoveryBoundary::Shell | StreamRecoveryBoundary::Stream => {
                let deadline = Instant::now() + Duration::from_secs(3);
                loop {
                    let finished = {
                        let state = owner.lock().unwrap();
                        let recovery = state.stream_recovery.as_ref().expect("retained recovery");
                        recovery.stdout_worker.as_ref().unwrap().is_finished()
                            && recovery.stderr_worker.as_ref().unwrap().is_finished()
                    };
                    if finished { break; }
                    assert!(Instant::now() < deadline, "released readers did not reach real EOF");
                    thread::sleep(FOREGROUND_POLL_INTERVAL);
                }
                // is_finished joins no JoinHandle and removes no registry
                // entry. The actual normal admission below must do that work.
                assert!(!owner.lock().unwrap().capture_complete);
                assert!(runtime.foreground_processes.lock().unwrap().contains_key(&registration_id));
                assert!(fs::read_to_string(&spool).unwrap().contains("later-tail"));
                match boundary {
                    StreamRecoveryBoundary::Shell => {
                        let result = runtime.shell(&foreground_args("printf normal-admission")).unwrap();
                        assert!(matches!(result.result,
                            Some(shell_result::Result::Success(result))
                                if result.stdout == "normal-admission"));
                    }
                    StreamRecoveryBoundary::Stream => {
                        let mut events = Vec::new();
                        runtime.shell_stream_live(
                            80, "normal-admission", &foreground_args("printf normal-admission"),
                            &AtomicBool::new(false), &mut |event| { events.push(event); Ok(()) },
                        ).expect("actual normal stream admission");
                        let stdout = events.iter().filter_map(|event| match &event.element {
                            Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                                message: Some(exec_client_message::Message::ShellStream(ShellStream {
                                    event: Some(shell_stream::Event::Stdout(stdout)),
                                })), ..
                            })) => Some(stdout.data.as_str()),
                            _ => None,
                        }).collect::<String>();
                        assert_eq!(stdout, "normal-admission");
                        assert!(events.iter().any(|event| matches!(&event.element,
                            Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                                message: Some(exec_client_message::Message::ShellStream(ShellStream {
                                    event: Some(shell_stream::Event::Exit(_)),
                                })), ..
                            })))));
                    }
                    StreamRecoveryBoundary::Stop => unreachable!(),
                }
                assert!(!runtime.stopping.load(Ordering::Acquire),
                    "normal admission, not shutdown, must observe late completion");
            }
        }
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        assert!(!spool.exists(), "observed recovery removes its own spool");
        let mut state = owner.lock().unwrap();
        assert!(state.capture_complete);
        assert!(state.stream_recovery.is_none(), "both JoinHandles were observed");
        state.signal(true).unwrap();
        assert_eq!(state.signals_sent, 0, "late cleanup never re-signals a reaped PID");
    }

    #[test]
    fn foreground_capture_stream_unwind_retains_readers_until_later_eof() {
        assert_stream_unwind_recovery(StreamRecoveryBoundary::Stop);
    }

    #[test]
    fn foreground_capture_stream_late_eof_is_observed_by_normal_admission() {
        assert_stream_unwind_recovery(StreamRecoveryBoundary::Shell);
        assert_stream_unwind_recovery(StreamRecoveryBoundary::Stream);
    }


    fn stream_file_request(args: ShellArgs) -> ExecServerMessage {
        ExecServerMessage {
            id: 81, exec_id: "stream-files".into(),
            message: Some(exec_server_message::Message::ShellStreamArgs(args)),
        }
    }

    fn stream_file_stdout(events: &[ExecStreamElement]) -> String {
        events.iter().filter_map(|event| match &event.element {
            Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                message: Some(exec_client_message::Message::ShellStream(ShellStream {
                    event: Some(shell_stream::Event::Stdout(stdout)),
                })), ..
            })) => Some(stdout.data.as_str()),
            _ => None,
        }).collect()
    }

    fn assert_stream_file_result(events: &[ExecStreamElement], unknown: bool) {
        let mut starts = 0;
        let mut exits = 0;
        let mut throws = 0;
        let mut closes = 0;
        for event in events {
            match &event.element {
                Some(exec_stream_element::Element::ExecClientMessage(message)) => {
                    assert_eq!(message.id, 81);
                    assert_eq!(message.exec_id, "stream-files");
                    match &message.message {
                        Some(exec_client_message::Message::ShellStream(ShellStream {
                            event: Some(shell_stream::Event::Start(_)),
                        })) => starts += 1,
                        Some(exec_client_message::Message::ShellStream(ShellStream {
                            event: Some(shell_stream::Event::Stdout(_) | shell_stream::Event::Stderr(_)),
                        })) => {}
                        Some(exec_client_message::Message::ShellStream(ShellStream {
                            event: Some(shell_stream::Event::Exit(exit)),
                        })) => {
                            assert_eq!(exit.code, 0);
                            assert!(!exit.aborted);
                            exits += 1;
                        }
                        other => panic!("unexpected stream file result: {other:?}"),
                    }
                }
                Some(exec_stream_element::Element::ExecClientControlMessage(control)) =>
                    match &control.message {
                        Some(exec_client_control_message::Message::Throw(error)) => {
                            assert_eq!(error.id, 81);
                            assert_eq!(error.error_code.as_deref(), Some("BOX_EXEC_DAEMON_ERROR"));
                            throws += 1;
                        }
                        Some(exec_client_control_message::Message::StreamClose(close)) => {
                            assert_eq!(close.id, 81);
                            closes += 1;
                        }
                        other => panic!("unexpected stream file control: {other:?}"),
                    },
                other => panic!("unexpected stream file envelope: {other:?}"),
            }
        }
        assert_eq!(starts, 1);
        assert_eq!(exits, if unknown { 0 } else { 1 });
        assert_eq!(throws, if unknown { 1 } else { 0 });
        assert_eq!(closes, 1);
        assert!(matches!(events.last().and_then(|event| event.element.as_ref()),
            Some(exec_stream_element::Element::ExecClientControlMessage(
                ExecClientControlMessage {
                    message: Some(exec_client_control_message::Message::StreamClose(_)),
                },
            ))));
    }

    // Actual output is observed before this request resumes. Its Child remains
    // alive until the explicit file release, independently of the emitter gate.
    struct HeldFileStream {
        release_path: PathBuf,
        finish_path: PathBuf,
        output_observed: mpsc::Receiver<()>,
        resume: Option<mpsc::Sender<()>>,
        worker: Option<JoinHandle<(Result<(), String>, Vec<ExecStreamElement>)>>,
    }

    impl HeldFileStream {
        fn new(runtime: &Arc<BoxExecRuntime>, name: &'static str, background: bool) -> Self {
            let release_path = runtime.workspace_root.join(format!("{name}-release"));
            let finish_path = runtime.workspace_root.join(format!("{name}-finish"));
            let (ready_tx, ready_rx) = mpsc::channel();
            let (output_tx, output_observed) = mpsc::channel();
            let (resume_tx, resume_rx) = mpsc::channel();
            let worker_runtime = Arc::clone(runtime);
            let worker = thread::spawn(move || {
                let mut args = foreground_args(&format!(
                    "printf 'effect\\n' > {name}-effect; printf {name}-prefix; \
                     while [ ! -f {name}-release ]; do sleep 0.01; done; printf {name}-tail; \
                     while [ ! -f {name}-finish ]; do sleep 0.01; done",
                ));
                args.timeout = if background { 5_000 } else { 30_000 };
                args.timeout_behavior = if background { TIMEOUT_BEHAVIOR_BACKGROUND } else { 0 };
                let mut events = Vec::new();
                let mut stdout = String::new();
                let mut ready = Some(ready_tx);
                let mut output_ready = Some(output_tx);
                let result = worker_runtime.execute_with_emitter(
                    stream_file_request(args), &AtomicBool::new(false), |event| {
                        if let Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                            message: Some(exec_client_message::Message::ShellStream(ShellStream {
                                event: Some(shell_stream::Event::Stdout(output)),
                            })), ..
                        })) = &event.element {
                            stdout.push_str(&output.data);
                            if stdout.contains(&format!("{name}-prefix")) {
                                if let Some(ready) = ready.take() {
                                    ready.send(()).map_err(|error| error.to_string())?;
                                    resume_rx.recv_timeout(Duration::from_secs(15))
                                        .map_err(|error| error.to_string())?;
                                }
                            }
                            if stdout == format!("{name}-prefix{name}-tail") {
                                if let Some(ready) = output_ready.take() {
                                    ready.send(()).map_err(|error| error.to_string())?;
                                }
                            }
                        }
                        events.push(event);
                        Ok(())
                    },
                );
                (result, events)
            });
            let fixture = Self {
                release_path, finish_path, output_observed,
                resume: Some(resume_tx), worker: Some(worker),
            };
            ready_rx.recv_timeout(Duration::from_secs(3))
                .expect("real prefix reached the stream emitter");
            fixture
        }

        fn resume(&mut self) {
            if let Some(resume) = self.resume.take() {
                let _ = resume.send(());
            }
        }

        fn release_output(&mut self) {
            fs::write(&self.release_path, b"release").expect("release actual output");
            self.resume();
            self.output_observed.recv_timeout(Duration::from_secs(3))
                .expect("complete real output reached the emitter before child release");
        }

        fn finish(&mut self) -> Vec<ExecStreamElement> {
            fs::write(&self.release_path, b"release").expect("release actual output");
            fs::write(&self.finish_path, b"finish").expect("release actual child");
            self.resume();
            let (result, events) = self.worker.take().unwrap().join().expect("stream caller joined");
            result.expect("stream response including any matching Throw completed");
            events
        }
    }

    impl Drop for HeldFileStream {
        fn drop(&mut self) {
            let _ = fs::write(&self.release_path, b"release");
            let _ = fs::write(&self.finish_path, b"finish");
            self.resume();
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
    }

    fn shared_terminal_runtime(runtime: &BoxExecRuntime) -> Arc<BoxExecRuntime> {
        Arc::new(BoxExecRuntime::new(
            runtime.workspace_root.clone(), runtime.terminals_directory.clone(),
            std::env::vars().collect(),
        ))
    }

    #[test]
    fn foreground_capture_independent_runtimes_keep_distinct_spools_and_cleanup() {
        let (_root, runtime) = runtime();
        let first_runtime = Arc::new(runtime);
        let second_runtime = shared_terminal_runtime(&first_runtime);
        let mut first = HeldFileStream::new(&first_runtime, "first", false);
        let first_spool = first_runtime.stream_spool_path(ForegroundId(1), 0);
        assert_eq!(fs::read(&first_spool).unwrap(), b"first-prefix");
        let mut second = HeldFileStream::new(&second_runtime, "second", false);
        let second_spool = second_runtime.stream_spool_path(ForegroundId(1), 1);
        for runtime in [&first_runtime, &second_runtime] {
            let processes = runtime.foreground_processes.lock().unwrap();
            assert_eq!(processes.len(), 1);
            assert!(matches!(processes.get(&ForegroundId(1)),
                Some((_, ForegroundProcess::Stream))), "equal actual runtime-local identities");
        }
        assert_ne!(first_spool, second_spool);
        assert_eq!(fs::read(&first_spool).unwrap(), b"first-prefix");
        assert_eq!(fs::read(&second_spool).unwrap(), b"second-prefix");
        first.release_output();
        assert_eq!(fs::read(&first_spool).unwrap(), b"first-prefixfirst-tail");
        let first_events = first.finish();
        assert_stream_file_result(&first_events, false);
        assert_eq!(stream_file_stdout(&first_events), "first-prefixfirst-tail");
        assert!(!first_spool.exists());
        assert_eq!(fs::read(&second_spool).unwrap(), b"second-prefix",
            "first completion must not remove or overwrite the other active stream");
        assert!(second_runtime.foreground_processes.lock().unwrap().contains_key(&ForegroundId(1)));
        second.release_output();
        assert_eq!(fs::read(&second_spool).unwrap(), b"second-prefixsecond-tail");
        let second_events = second.finish();
        assert_stream_file_result(&second_events, false);
        assert_eq!(stream_file_stdout(&second_events), "second-prefixsecond-tail");
        assert!(!second_spool.exists());
        for name in ["first", "second"] {
            assert_eq!(fs::read(first_runtime.workspace_root.join(format!("{name}-effect"))).unwrap(),
                b"effect\n");
        }
        first_runtime.stop().expect("first real child and readers completed");
        second_runtime.stop().expect("second real child and readers completed");
        assert_eq!(fs::read_dir(&first_runtime.terminals_directory).unwrap().count(), 0);
    }

    fn assert_file_sentinel(path: &Path, target: &Path, symbolic: bool) {
        if symbolic {
            assert!(fs::symlink_metadata(path).unwrap().file_type().is_symlink());
            assert_eq!(fs::read_link(path).unwrap(), target);
        } else {
            assert!(fs::symlink_metadata(path).unwrap().is_file());
        }
        assert_eq!(fs::read(path).unwrap(), b"foreign sentinel\n");
        assert_eq!(fs::read(target).unwrap(), b"foreign sentinel\n");
    }

    fn create_file_sentinel(path: &Path, target: &Path, symbolic: bool) {
        if symbolic {
            symlink(target, path).unwrap();
        } else {
            fs::write(path, b"foreign sentinel\n").unwrap();
        }
    }

    fn assert_occupied_stream_spools_preserved(symbolic: bool) {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let target = runtime.workspace_root.join("sentinel-target");
        fs::write(&target, b"foreign sentinel\n").unwrap();
        let mut sentinels = Vec::new();
        for attempt in 0..STREAM_SPOOL_CREATE_ATTEMPTS {
            let path = runtime.stream_spool_path(ForegroundId(1), attempt);
            create_file_sentinel(&path, &target, symbolic);
            sentinels.push(path);
        }
        let (release, gate) = mpsc::channel();
        *runtime.stream_setup_gate.lock().unwrap() = Some(gate);
        let (observed_tx, observed_rx) = mpsc::channel();
        *runtime.stream_recovery_observer.lock().unwrap() = Some(observed_tx);
        let worker_runtime = Arc::clone(&runtime);
        let worker = thread::spawn(move || worker_runtime.execute(stream_file_request(
            foreground_args("printf spool-prefix; printf 'effect\\n' > spool-effect; \
                while :; do sleep 1; done"),
        )));
        let deadline = Instant::now() + Duration::from_secs(3);
        while fs::read(runtime.workspace_root.join("spool-effect")).unwrap_or_default() != b"effect\n" {
            assert!(Instant::now() < deadline, "real effect must precede failed spool allocation");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        assert!(runtime.foreground_processes.lock().unwrap().contains_key(&ForegroundId(1)));
        release.send(()).unwrap();
        let started = Instant::now();
        let events = worker.join().expect("bounded actual setup failure");
        assert!(started.elapsed() < Duration::from_secs(4));
        assert_stream_file_result(&events, true);
        assert!(events.iter().any(|event| matches!(&event.element,
            Some(exec_stream_element::Element::ExecClientControlMessage(
                ExecClientControlMessage {
                    message: Some(exec_client_control_message::Message::Throw(error)),
                },
            )) if error.error == "shell stream spool allocation exhausted")));
        let (id, owner) = observed_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(id, ForegroundId(1));
        {
            let mut state = owner.lock().unwrap();
            assert!(state.poll().unwrap().is_some());
            assert!(state.capture_complete);
            assert!(state.stream_recovery.is_none());
            assert!(state.signals_sent > 0);
            let signals = state.signals_sent;
            state.signal(true).unwrap();
            assert_eq!(state.signals_sent, signals);
        }
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        assert!(!runtime.stopping.load(Ordering::Acquire));
        assert_eq!(fs::read(runtime.workspace_root.join("spool-effect")).unwrap(), b"effect\n");
        for path in &sentinels {
            assert_file_sentinel(path, &target, symbolic);
        }
        // The next invocation also meets an occupied candidate, but has room
        // to exclusively claim another. It must capture only its own bytes.
        let next_collision = runtime.stream_spool_path(ForegroundId(2), 0);
        create_file_sentinel(&next_collision, &target, symbolic);
        sentinels.push(next_collision);
        let events = runtime.execute(stream_file_request(foreground_args("printf recovered")));
        assert_stream_file_result(&events, false);
        assert_eq!(stream_file_stdout(&events), "recovered");
        for path in &sentinels {
            assert_file_sentinel(path, &target, symbolic);
        }
        assert_eq!(fs::read_dir(&runtime.terminals_directory).unwrap().count(), sentinels.len());
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        runtime.stop().expect("failed and successful children both observed");
    }

    #[test]
    fn foreground_capture_occupied_spool_files_survive_failure_and_recovery() {
        assert_occupied_stream_spools_preserved(false);
    }

    #[cfg(unix)]
    #[test]
    fn foreground_capture_occupied_spool_symlinks_survive_failure_and_recovery() {
        assert_occupied_stream_spools_preserved(true);
    }

    fn assert_occupied_stream_terminal_preserved(symbolic: bool) {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        struct PipeRelease(Arc<BoxExecRuntime>);
        impl Drop for PipeRelease {
            fn drop(&mut self) {
                let _ = fs::write(self.0.workspace_root.join("release-file-pipe"), b"release");
                let _ = self.0.stop();
            }
        }
        let _release = PipeRelease(Arc::clone(&runtime));
        let terminal = runtime.terminals_directory.join("1.txt");
        let target = runtime.workspace_root.join("terminal-sentinel-target");
        fs::write(&target, b"foreign sentinel\n").unwrap();
        create_file_sentinel(&terminal, &target, symbolic);
        let (release_setup, setup_gate) = mpsc::channel();
        *runtime.stream_setup_gate.lock().unwrap() = Some(setup_gate);
        let (observed_tx, observed_rx) = mpsc::channel();
        *runtime.stream_recovery_observer.lock().unwrap() = Some(observed_tx);
        let worker_runtime = Arc::clone(&runtime);
        let worker = thread::spawn(move || {
            let mut args = foreground_args(
                "(trap '' TERM; printf ready > file-pipe-ready; \
                  while [ ! -f release-file-pipe ]; do sleep 0.01; done; \
                  printf inherited-tail) & \
                 while [ ! -f file-pipe-ready ]; do sleep 0.01; done; \
                 printf handoff-prefix; printf 'effect\\n' > terminal-effect; \
                 IFS= read -r release",
            );
            args.timeout = 1;
            args.timeout_behavior = TIMEOUT_BEHAVIOR_BACKGROUND;
            worker_runtime.execute(stream_file_request(args))
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        while fs::read(runtime.workspace_root.join("terminal-effect")).unwrap_or_default()
            != b"effect\n"
        {
            assert!(Instant::now() < deadline, "actual prefix and effect must precede handoff");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        assert_eq!(fs::read(runtime.workspace_root.join("file-pipe-ready")).unwrap(), b"ready");
        release_setup.send(()).unwrap();
        let started = Instant::now();
        let events = worker.join().expect("occupied terminal returns without waiting for inherited EOF");
        assert!(started.elapsed() < Duration::from_secs(4));
        assert_stream_file_result(&events, true);
        let (registration_id, owner) = observed_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(registration_id, ForegroundId(1));
        let spool = {
            let mut state = owner.lock().unwrap();
            assert!(state.poll().unwrap().is_some(), "guard reaped its actual Child");
            assert!(!state.capture_complete, "explicit release still owns inherited pipes");
            let recovery = state.stream_recovery.as_ref().expect("actual reader custody retained");
            assert!(recovery.stdout_worker.is_some());
            assert!(recovery.stderr_worker.is_some());
            assert_eq!(recovery.cleanup_paths.len(), 1, "occupied terminal grants no cleanup claim");
            assert_eq!(recovery.cleanup_paths[0], runtime.stream_spool_path(ForegroundId(1), 0));
            recovery.cleanup_paths[0].clone()
        };
        assert!(runtime.foreground_processes.lock().unwrap().contains_key(&registration_id));
        assert!(runtime.background.lock().unwrap().is_empty(), "failed handoff is not admitted");
        assert_file_sentinel(&terminal, &target, symbolic);
        let deadline = Instant::now() + Duration::from_secs(3);
        while fs::read(&spool).unwrap_or_default() != b"handoff-prefix" {
            assert!(Instant::now() < deadline, "real reader did not capture its prefix");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        let normal = runtime.shell(&foreground_args("printf before-file-release")).unwrap();
        assert!(matches!(normal.result,
            Some(shell_result::Result::Success(result)) if result.stdout == "before-file-release"));
        assert!(runtime.foreground_processes.lock().unwrap().contains_key(&registration_id));
        assert!(!owner.lock().unwrap().capture_complete, "normal admission must retain pending readers");
        assert_eq!(fs::read(&spool).unwrap(), b"handoff-prefix");
        assert_file_sentinel(&terminal, &target, symbolic);

        fs::write(runtime.workspace_root.join("release-file-pipe"), b"release").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let finished = {
                let state = owner.lock().unwrap();
                let recovery = state.stream_recovery.as_ref().unwrap();
                recovery.stdout_worker.as_ref().unwrap().is_finished()
                    && recovery.stderr_worker.as_ref().unwrap().is_finished()
            };
            if finished { break; }
            assert!(Instant::now() < deadline, "explicitly released readers did not finish");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        assert_eq!(fs::read(&spool).unwrap(), b"handoff-prefixinherited-tail");
        assert!(!owner.lock().unwrap().capture_complete, "fixture has not joined either worker");
        let events = runtime.execute(stream_file_request(foreground_args("printf recovered")));
        assert_stream_file_result(&events, false);
        assert_eq!(stream_file_stdout(&events), "recovered");
        assert!(!spool.exists(), "actual normal admission observes readers before deleting its spool");
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        assert!(!runtime.stopping.load(Ordering::Acquire));
        {
            let mut state = owner.lock().unwrap();
            assert!(state.capture_complete);
            assert!(state.stream_recovery.is_none());
            let signals = state.signals_sent;
            state.signal(true).unwrap();
            assert_eq!(state.signals_sent, signals, "completed recovery cannot signal a reused PID");
        }
        assert_file_sentinel(&terminal, &target, symbolic);
        assert_eq!(fs::read(runtime.workspace_root.join("terminal-effect")).unwrap(), b"effect\n");
        assert_eq!(fs::read_dir(&runtime.terminals_directory).unwrap().count(), 1);
    }

    #[test]
    fn foreground_capture_occupied_terminal_files_preserve_pending_reader_ownership() {
        assert_occupied_stream_terminal_preserved(false);
    }

    #[cfg(unix)]
    #[test]
    fn foreground_capture_occupied_terminal_symlinks_preserve_pending_reader_ownership() {
        assert_occupied_stream_terminal_preserved(true);
    }

    #[test]
    fn foreground_capture_retired_spool_claim_cannot_remove_reused_candidate() {
        let (_root, runtime) = runtime();
        let first_runtime = Arc::new(runtime);
        let second_runtime = shared_terminal_runtime(&first_runtime);
        let mut first = HeldFileStream::new(&first_runtime, "retired", true);
        let spool = first_runtime.stream_spool_path(ForegroundId(1), 0);
        assert_eq!(fs::read(&spool).unwrap(), b"retired-prefix");

        // The existing admission mutex holds the real handoff after writer
        // transfer and spool deletion, before its fallible stop admission.
        let background_admission = first_runtime.background.lock().unwrap();
        first.resume();
        let terminal = first_runtime.terminals_directory.join("1.txt");
        let deadline = Instant::now() + Duration::from_secs(8);
        while spool.exists() || !terminal.exists() {
            assert!(Instant::now() < deadline, "real handoff did not retire the spool");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        assert!(fs::read_to_string(&terminal).unwrap().contains("retired-prefix"));
        let mut second = HeldFileStream::new(&second_runtime, "reused", false);
        assert_eq!(second_runtime.stream_spool_path(ForegroundId(1), 0), spool);
        assert_eq!(fs::read(&spool).unwrap(), b"reused-prefix",
            "the second real runtime exclusively reclaimed the removed candidate");

        let stopping_runtime = Arc::clone(&first_runtime);
        let stopper = thread::spawn(move || stopping_runtime.stop());
        let deadline = Instant::now() + Duration::from_secs(3);
        while !first_runtime.stopping.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline, "actual stop did not close admission");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        drop(background_admission);
        let events = first.finish();
        assert_stream_file_result(&events, true);
        stopper.join().expect("actual stopper joined").expect("failed handoff resources observed");
        assert!(first_runtime.foreground_processes.lock().unwrap().is_empty());
        assert!(first_runtime.background.lock().unwrap().is_empty());
        assert!(!terminal.exists(), "failed handoff removes only the terminal it created");
        assert_eq!(fs::read(&spool).unwrap(), b"reused-prefix",
            "later predecessor recovery must not delete a successor's claimed file");
        second.release_output();
        assert_eq!(fs::read(&spool).unwrap(), b"reused-prefixreused-tail");
        let events = second.finish();
        assert_stream_file_result(&events, false);
        assert_eq!(stream_file_stdout(&events), "reused-prefixreused-tail");
        assert!(!spool.exists());
        for name in ["retired", "reused"] {
            assert_eq!(fs::read(first_runtime.workspace_root.join(format!("{name}-effect"))).unwrap(),
                b"effect\n");
        }
        second_runtime.stop().expect("successor child and readers completed");
        assert_eq!(fs::read_dir(&first_runtime.terminals_directory).unwrap().count(), 0);
    }

    #[test]
    fn foreground_capture_interrupted_read_retries_without_losing_bytes() {
        struct InterruptedOnce {
            interrupted: bool,
            bytes: std::io::Cursor<Vec<u8>>,
        }
        impl Read for InterruptedOnce {
            fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
                if !self.interrupted {
                    self.interrupted = true;
                    return Err(std::io::ErrorKind::Interrupted.into());
                }
                self.bytes.read(output)
            }
        }
        let control = CaptureControl {
            cancelled: Arc::new(AtomicBool::new(false)), probe: None, pid: 0,
        };
        let bytes = read_capture(InterruptedOnce {
            interrupted: false, bytes: std::io::Cursor::new(b"complete".to_vec()),
        }, &control, "stdout").expect("interrupted read retried");
        assert_eq!(bytes, b"complete");
    }


    fn background_args(command: &str) -> BackgroundShellSpawnArgs {
        BackgroundShellSpawnArgs {
            command: command.into(), working_directory: "/workspace".into(),
            tool_call_id: "background-custody".into(),
        }
    }

    fn background_request(command: &str) -> ExecServerMessage {
        ExecServerMessage {
            id: 82, exec_id: "background-custody".into(),
            message: Some(exec_server_message::Message::BackgroundShellSpawnArgs(background_args(command))),
        }
    }

    fn background_success(result: Result<BackgroundShellSpawnResult, ShellExecutionUnknown>)
        -> BackgroundShellSpawnSuccess
    {
        match result.expect("observed background admission").result {
            Some(background_shell_spawn_result::Result::Success(success)) => success,
            other => panic!("expected background success, got {other:?}"),
        }
    }

    fn background_owner(runtime: &BoxExecRuntime, shell_id: u32) -> Arc<BackgroundProcess> {
        runtime.background.lock().unwrap().get(&shell_id).cloned().expect("actual background owner")
    }

    fn wait_background_removed(runtime: &BoxExecRuntime, shell_id: u32) {
        let deadline = Instant::now() + Duration::from_secs(4);
        while runtime.background.lock().unwrap().contains_key(&shell_id) {
            assert!(Instant::now() < deadline, "actual controller did not remove its completed invocation");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
    }

    fn wait_file_bytes(path: &Path, expected: &[u8]) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while fs::read(path).unwrap_or_default() != expected {
            assert!(Instant::now() < deadline, "actual fixture bytes were not observed: {}", path.display());
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
    }

    struct BackgroundFixture {
        runtime: Arc<BoxExecRuntime>,
        releases: Vec<PathBuf>,
    }

    impl BackgroundFixture {
        fn new(runtime: &Arc<BoxExecRuntime>, releases: &[&str]) -> Self {
            Self {
                runtime: Arc::clone(runtime),
                releases: releases.iter().map(|name| runtime.workspace_root.join(name)).collect(),
            }
        }
    }

    impl Drop for BackgroundFixture {
        fn drop(&mut self) {
            for path in &self.releases {
                let _ = fs::write(path, b"release");
            }
            let _ = self.runtime.stop();
        }
    }

    fn assert_background_throw(events: &[ExecStreamElement], phase: &str) {
        assert_eq!(events.len(), 2, "entered background failure has only Throw and Close");
        assert!(matches!(&events[0].element,
            Some(exec_stream_element::Element::ExecClientControlMessage(
                ExecClientControlMessage {
                    message: Some(exec_client_control_message::Message::Throw(error)),
                },
            )) if error.id == 82
                && error.error_code.as_deref() == Some("BOX_EXEC_DAEMON_ERROR")
                && error.error.starts_with(&format!("background shell outcome unknown: {phase}"))
                && !error.error.contains("SC_CAPTURE_PREFIX")
                && !error.error.contains("effect-count")));
        assert!(matches!(&events[1].element,
            Some(exec_stream_element::Element::ExecClientControlMessage(
                ExecClientControlMessage {
                    message: Some(exec_client_control_message::Message::StreamClose(
                        ExecClientStreamClose { id: 82 },
                    )),
                },
            ))));
    }

    #[test]
    fn background_custody_normal_and_handoff_preserve_output_stdin_and_autonomous_completion() {
        for timed in [false, true] {
            let (_root, runtime) = runtime();
            let runtime = Arc::new(runtime);
            let _cleanup = BackgroundFixture::new(&runtime, &[]);
            let command = "printf 'effect\\n' > effect-count; printf '%s\\n' \"$$\" > child-pid; \
                printf 'stdout-prefix\\n'; printf 'stderr-prefix\\n' >&2; \
                IFS= read -r input; [ \"$input\" = finish ] || exit 91; \
                printf 'stdout-tail\\n'; printf 'stderr-tail\\n' >&2";
            let (shell_id, pid) = if timed {
                let mut args = foreground_args(command);
                args.timeout = 20;
                args.timeout_behavior = TIMEOUT_BEHAVIOR_BACKGROUND;
                let events = runtime.execute(stream_file_request(args));
                let result = events.iter().find_map(|event| match &event.element {
                    Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                        message: Some(exec_client_message::Message::ShellStream(ShellStream {
                            event: Some(shell_stream::Event::Backgrounded(result)),
                        })), ..
                    })) => Some(result),
                    _ => None,
                }).expect("actual timed handoff");
                assert_eq!(result.reason, Some(SHELL_BACKGROUND_REASON_TIMEOUT));
                (result.shell_id, result.pid.unwrap())
            } else {
                let result = background_success(runtime.spawn_background(&background_args(command)));
                (result.shell_id, result.pid.unwrap())
            };
            assert_ne!(shell_id, 0);
            let owner = background_owner(&runtime, shell_id);
            let terminal = owner.terminal_path.clone();
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                let text = fs::read_to_string(&terminal).unwrap_or_default();
                if text.contains("stdout-prefix\n") && text.contains("stderr-prefix\n") { break; }
                assert!(Instant::now() < deadline, "actual stdout/stderr prefix not captured");
                thread::sleep(FOREGROUND_POLL_INTERVAL);
            }
            assert_eq!(fs::read(runtime.workspace_root.join("effect-count")).unwrap(), b"effect\n");
            assert_eq!(fs::read_to_string(runtime.workspace_root.join("child-pid")).unwrap()
                .trim().parse::<u32>().unwrap(), pid);
            {
                let mut state = owner.state.lock().unwrap();
                assert_eq!(state.resources.state.child.id(), pid);
                assert!(state.resources.state.poll().unwrap().is_none());
                assert!(state.admitted);
            }
            assert!(!fs::read_to_string(&terminal).unwrap().contains("\nexit_code:"));
            assert_eq!(runtime.admit_background(shell_id, &owner).unwrap_err().phase,
                "background admission already started");
            assert!(Arc::ptr_eq(&background_owner(&runtime, shell_id), &owner));
            let input = runtime.write_stdin(&WriteShellStdinArgs { shell_id, chars: "finish\n".into() });
            assert!(matches!(input.result, Some(write_shell_stdin_result::Result::Success(_))));
            // No observer/admission call helps this assertion: the existing
            // controller must commit its own actual operation terminal.
            wait_background_removed(&runtime, shell_id);
            let text = fs::read_to_string(&terminal).unwrap();
            for line in ["stdout-prefix\n", "stderr-prefix\n", "stdout-tail\n", "stderr-tail\n"] {
                assert_eq!(text.matches(line).count(), 1);
            }
            assert_eq!(text.matches("\nexit_code: 0\n").count(), 1);
            assert!(text.ends_with("\n---\n"));
            let mut state = owner.state.lock().unwrap();
            assert!(state.phase == BackgroundPhase::Settled);
            assert!(state.resources.state.status.is_some());
            assert!(state.resources.state.capture_complete);
            assert!(state.resources.recovery.stdout_worker.is_none());
            assert!(state.resources.recovery.stderr_worker.is_none());
            assert!(state.footer_written);
            assert!(state.failure.is_none());
            let signals = state.resources.state.signals_sent;
            state.resources.state.signal(true).unwrap();
            assert_eq!(state.resources.state.signals_sent, signals);
        }
    }

    #[test]
    fn background_custody_pre_spawn_refusals_preserve_files_and_nonreusable_ids() {
        for symbolic in [false, true] {
            let (_root, runtime) = runtime();
            let target = runtime.workspace_root.join("foreign-target");
            fs::write(&target, b"foreign sentinel\n").unwrap();
            let terminal = runtime.terminals_directory.join("1.txt");
            create_file_sentinel(&terminal, &target, symbolic);
            let refused = runtime.spawn_background(&background_args("printf forbidden > refused-effect"))
                .expect("occupied terminal is a known pre-spawn refusal");
            assert!(matches!(refused.result, Some(background_shell_spawn_result::Result::Error(_))));
            assert!(!runtime.workspace_root.join("refused-effect").exists());
            assert!(runtime.background.lock().unwrap().is_empty());
            assert_file_sentinel(&terminal, &target, symbolic);
            let later = background_success(runtime.spawn_background(&background_args("printf later")));
            assert_eq!(later.shell_id, 2, "an occupied identity is not reused");
            wait_background_removed(&runtime, later.shell_id);
            assert_file_sentinel(&terminal, &target, symbolic);
            assert!(fs::read_to_string(runtime.terminals_directory.join("2.txt")).unwrap()
                .contains("\nexit_code: 0\n"));
            runtime.stop().unwrap();
        }
        let (_root, stopped) = runtime();
        stopped.stop().unwrap();
        let refused = stopped.spawn_background(&background_args("printf forbidden > after-stop")).unwrap();
        assert!(matches!(refused.result, Some(background_shell_spawn_result::Result::Error(_))));
        assert!(!stopped.workspace_root.join("after-stop").exists());
        assert_eq!(fs::read_dir(&stopped.terminals_directory).unwrap().count(), 0);

        let (_root, exhausted) = runtime();
        exhausted.next_shell_id.store(u32::MAX - 1, Ordering::Relaxed);
        let last = background_success(exhausted.spawn_background(&background_args(
            "printf 'effect\\n' > final-id-effect",
        )));
        assert_eq!(last.shell_id, u32::MAX - 1);
        wait_background_removed(&exhausted, last.shell_id);
        for _ in 0..2 {
            let refused = exhausted.spawn_background(&background_args("printf forbidden > after-exhaustion"))
                .expect("ID exhaustion is known before spawn");
            assert!(matches!(refused.result,
                Some(background_shell_spawn_result::Result::Error(error))
                    if error.error == "background shell id space exhausted"));
        }
        assert_eq!(exhausted.next_shell_id.load(Ordering::Relaxed), u32::MAX);
        assert!(!exhausted.workspace_root.join("after-exhaustion").exists());
        assert_eq!(fs::read(exhausted.workspace_root.join("final-id-effect")).unwrap(), b"effect\n");
        exhausted.stop().unwrap();
    }

    #[test]
    fn background_custody_setup_and_controller_refusals_retain_child_and_readers() {
        for mode in ["background-setup", "background-controller", "controller-send"] {
            let (root, runtime) = runtime();
            let runtime = Arc::new(runtime);
            let _cleanup = BackgroundFixture::new(&runtime, &["release-background-pipe"]);
            if mode != "controller-send" {
                *runtime.capture_probe.lock().unwrap() =
                    CaptureProbe::new(mode, root.path().join("fault-observed.json")).map(Arc::new);
            } else {
                *runtime.background_controller_fault.lock().unwrap() = Some(BackgroundControllerFault::Send);
            }
            let (setup_tx, setup_rx) = mpsc::channel();
            *runtime.background_setup_gate.lock().unwrap() = Some(setup_rx);
            let (controller_tx, controller_rx) = mpsc::channel();
            if mode == "controller-send" {
                *runtime.background_controller_gate.lock().unwrap() = Some(controller_rx);
            }
            let worker_runtime = Arc::clone(&runtime);
            let worker = thread::spawn(move || worker_runtime.execute(background_request(
                "(trap '' TERM; printf ready > holder-ready; \
                  while [ ! -f release-background-pipe ]; do sleep 0.01; done; \
                  printf inherited-tail) & \
                 while [ ! -f holder-ready ]; do sleep 0.01; done; \
                 printf 'effect\\n' > effect-count; printf SC_CAPTURE_PREFIX; \
                 printf ready > child-ready; while :; do sleep 1; done",
            )));
            wait_file_bytes(&runtime.workspace_root.join("child-ready"), b"ready");
            let owner = background_owner(&runtime, 1);
            let pid = {
                let mut state = owner.state.lock().unwrap();
                assert!(state.resources.state.poll().unwrap().is_none());
                state.resources.state.child.id()
            };
            setup_tx.send(()).unwrap();
            if mode == "controller-send" {
                let deadline = Instant::now() + Duration::from_secs(3);
                while !fs::read_to_string(&owner.terminal_path).unwrap_or_default().contains("SC_CAPTURE_PREFIX") {
                    assert!(Instant::now() < deadline, "real prefix precedes actual channel refusal");
                    thread::sleep(FOREGROUND_POLL_INTERVAL);
                }
                controller_tx.send(()).unwrap();
            }
            let started = Instant::now();
            let events = worker.join().expect("entered background refusal returned");
            assert!(started.elapsed() < Duration::from_secs(4));
            let phase = match mode {
                "background-setup" => "background capture setup failed",
                "background-controller" => "background controller creation failed",
                _ => "background controller admission failed",
            };
            assert_background_throw(&events, phase);
            assert_eq!(fs::read(runtime.workspace_root.join("effect-count")).unwrap(), b"effect\n");
            if mode != "controller-send" {
                let observed: serde_json::Value = serde_json::from_slice(
                    &fs::read(root.path().join("fault-observed.json")).unwrap(),
                ).unwrap();
                assert_eq!(observed["mode"], mode);
                assert_eq!(observed["prefix"], "SC_CAPTURE_PREFIX");
                assert_eq!(observed["childPid"], pid);
                assert!(observed["bytesObserved"].as_u64().unwrap() >= 17);
            }
            {
                let mut state = owner.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                assert!(state.resources.state.poll().unwrap().is_some());
                assert!(state.resources.recovery.stdout_worker.is_some());
                assert!(!state.resources.state.capture_complete);
                assert!(!state.admitted);
                let signals = state.resources.state.signals_sent;
                state.resources.state.signal(true).unwrap();
                assert_eq!(state.resources.state.signals_sent, signals);
            }
            assert!(runtime.background.lock().unwrap().contains_key(&1));
            assert!(owner.terminal_path.exists());
            fs::write(runtime.workspace_root.join("release-background-pipe"), b"release").unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                let finished = {
                    let state = owner.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    state.resources.recovery.stdout_worker.as_ref().is_none_or(|worker| worker.is_finished())
                        && state.resources.recovery.stderr_worker.as_ref().is_none_or(|worker| worker.is_finished())
                };
                if finished { break; }
                assert!(Instant::now() < deadline, "released background readers did not reach EOF");
                thread::sleep(FOREGROUND_POLL_INTERVAL);
            }
            *runtime.capture_probe.lock().unwrap() = None;
            let later = background_success(runtime.spawn_background(&background_args("printf recovered")));
            wait_background_removed(&runtime, later.shell_id);
            assert!(!runtime.background.lock().unwrap().contains_key(&1));
            assert!(!owner.terminal_path.exists(), "only the unpublished owned terminal is removed");
            assert!(!runtime.stopping.load(Ordering::Acquire));
        }
    }

    #[test]
    fn background_custody_stop_during_setup_retains_the_entered_request() {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let _cleanup = BackgroundFixture::new(&runtime, &[]);
        let (release, gate) = mpsc::channel();
        *runtime.background_setup_gate.lock().unwrap() = Some(gate);
        let worker_runtime = Arc::clone(&runtime);
        let worker = thread::spawn(move || worker_runtime.execute(background_request(
            "trap '' TERM; printf 'effect\\n' > effect-count; printf ready > term-ready; \
             printf SC_CAPTURE_PREFIX; while :; do sleep 1; done",
        )));
        wait_file_bytes(&runtime.workspace_root.join("term-ready"), b"ready");
        let owner = background_owner(&runtime, 1);
        assert!(owner.state.lock().unwrap().resources.state.poll().unwrap().is_none());
        let started = Instant::now();
        assert!(runtime.stop().is_err(), "the real setup request remains unobserved");
        assert!(started.elapsed() < Duration::from_secs(4));
        {
            let mut state = owner.state.lock().unwrap();
            assert!(state.resources.state.poll().unwrap().is_some(), "stop reaps the actual entered child");
            assert!(state.resources.state.signals_sent >= 2, "installed TERM-ignore requires force");
            assert!(state.phase == BackgroundPhase::Setup);
        }
        assert!(runtime.background.lock().unwrap().contains_key(&1));
        release.send(()).unwrap();
        let events = worker.join().expect("entered setup caller returned");
        assert_background_throw(&events, "background admission closed");
        assert_eq!(fs::read(runtime.workspace_root.join("effect-count")).unwrap(), b"effect\n");
        runtime.stop().expect("setup return and actual readers now observed");
        assert!(runtime.background.lock().unwrap().is_empty());
        assert!(!owner.terminal_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn background_custody_stop_and_hard_timeout_force_only_the_owned_unreaped_child() {
        use std::os::unix::process::ExitStatusExt;
        for hard_timeout in [false, true] {
            let (_root, runtime) = runtime();
            let runtime = Arc::new(runtime);
            let _cleanup = BackgroundFixture::new(&runtime, &[]);
            let command = "trap '' TERM; printf 'effect\\n' > effect-count; \
                printf ready > term-ready; printf term-prefix; while :; do sleep 1; done";
            let shell_id = if hard_timeout {
                let mut args = foreground_args(command);
                args.timeout = 20;
                args.timeout_behavior = TIMEOUT_BEHAVIOR_BACKGROUND;
                args.hard_timeout = Some(5_000);
                runtime.execute(stream_file_request(args)).iter().find_map(|event| match &event.element {
                    Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {
                        message: Some(exec_client_message::Message::ShellStream(ShellStream {
                            event: Some(shell_stream::Event::Backgrounded(result)),
                        })), ..
                    })) => Some(result.shell_id),
                    _ => None,
                }).expect("same child admitted through timed handoff")
            } else {
                background_success(runtime.spawn_background(&background_args(command))).shell_id
            };
            wait_file_bytes(&runtime.workspace_root.join("term-ready"), b"ready");
            let owner = background_owner(&runtime, shell_id);
            {
                let mut state = owner.state.lock().unwrap();
                assert!(state.resources.state.poll().unwrap().is_none(), "trap readiness precedes termination");
                assert_eq!(state.resources.state.signals_sent, 0);
            }
            let started = Instant::now();
            if hard_timeout {
                let deadline = Instant::now() + Duration::from_secs(7);
                while runtime.background.lock().unwrap().contains_key(&shell_id) {
                    assert!(Instant::now() < deadline, "actual hard timeout did not force and reap");
                    thread::sleep(FOREGROUND_POLL_INTERVAL);
                }
            } else {
                runtime.stop().expect("bounded TERM/KILL, actual reap and output observation");
                assert!(started.elapsed() < Duration::from_secs(4));
            }
            let mut state = owner.state.lock().unwrap();
            let status = state.resources.state.poll().unwrap().expect("actual child reaped");
            assert_eq!(status.signal(), Some(libc::SIGKILL));
            assert!(state.resources.state.signals_sent >= 2);
            assert!(state.phase == BackgroundPhase::Settled);
            let signals = state.resources.state.signals_sent;
            state.resources.state.signal(true).unwrap();
            assert_eq!(state.resources.state.signals_sent, signals, "no cached PID may be signaled after reap");
            drop(state);
            let text = fs::read_to_string(&owner.terminal_path).unwrap();
            assert!(text.contains("\nerror: background child exited without a numeric status\n"));
            assert!(text.contains("\nended_at:"));
            assert!(!text.contains("\nexit_code:"), "no synthetic exit 1 replaces the actual signal");
            assert_eq!(fs::read(runtime.workspace_root.join("effect-count")).unwrap(), b"effect\n");
            runtime.stop().expect("repeated stop only observes completed ownership");
        }
    }

    #[test]
    fn background_custody_reaped_child_keeps_readers_until_explicit_eof() {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let _cleanup = BackgroundFixture::new(&runtime, &["release-inherited"]);
        let result = background_success(runtime.spawn_background(&background_args(
            "exec 3<&0; \
             (trap '' TERM; printf ready > inherited-ready; IFS= read -r input <&3; \
              printf eof > inherited-input-eof; \
              while [ ! -f release-inherited ]; do sleep 0.01; done; printf inherited-tail) & \
             exec 3<&-; while [ ! -f inherited-ready ]; do sleep 0.01; done; \
             printf 'effect\\n' > effect-count; printf direct-prefix; exit 0",
        )));
        let owner = background_owner(&runtime, result.shell_id);
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if owner.state.lock().unwrap().resources.state.poll().unwrap().is_some() { break; }
            assert!(Instant::now() < deadline, "actual direct child did not reap");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        {
            let state = owner.state.lock().unwrap();
            assert!(state.resources.recovery.stdout_worker.is_some());
            assert!(state.resources.recovery.stderr_worker.is_some());
            assert!(!state.resources.state.capture_complete);
            assert_eq!(state.resources.state.signals_sent, 0);
        }
        wait_file_bytes(&runtime.workspace_root.join("inherited-input-eof"), b"eof");
        assert!(owner.stdin.lock().unwrap().is_none(),
            "observed direct-child reap closes only the owner's stdin reference");
        let started = Instant::now();
        assert!(runtime.stop().is_err(), "actual inherited pipes prevent a clean stop");
        assert!(started.elapsed() < Duration::from_secs(4));
        assert!(runtime.background.lock().unwrap().contains_key(&result.shell_id));
        assert!(!fs::read_to_string(&owner.terminal_path).unwrap().contains("\nexit_code:"));
        assert_eq!(owner.state.lock().unwrap().resources.state.signals_sent, 0);
        fs::write(runtime.workspace_root.join("release-inherited"), b"release").unwrap();
        wait_background_removed(&runtime, result.shell_id);
        let text = fs::read_to_string(&owner.terminal_path).unwrap();
        assert!(text.contains("direct-prefix"));
        assert!(text.contains("inherited-tail"));
        assert!(text.contains("\nexit_code: 0\n"), "actual zero status and complete readers remain known");
        let mut state = owner.state.lock().unwrap();
        assert!(state.resources.state.capture_complete);
        state.resources.state.signal(true).unwrap();
        assert_eq!(state.resources.state.signals_sent, 0);
        drop(state);
        runtime.stop().expect("later actual EOF completed autonomous cleanup");
    }

    #[test]
    fn background_custody_controller_loss_recovers_through_the_same_entry() {
        for fault in [BackgroundControllerFault::PanicAfterReap, BackgroundControllerFault::ExitAfterReap] {
            let (_root, runtime) = runtime();
            let runtime = Arc::new(runtime);
            let _cleanup = BackgroundFixture::new(&runtime, &["release-controller-child", "release-controller-pipe"]);
            *runtime.background_controller_fault.lock().unwrap() = Some(fault);
            let result = background_success(runtime.spawn_background(&background_args(
                "(trap '' TERM; printf ready > controller-holder-ready; \
                  while [ ! -f release-controller-pipe ]; do sleep 0.01; done; printf controller-tail) & \
                 while [ ! -f controller-holder-ready ]; do sleep 0.01; done; \
                 printf 'effect\\n' > effect-count; printf controller-prefix; \
                 while [ ! -f release-controller-child ]; do sleep 0.01; done; exit 0",
            )));
            let owner = background_owner(&runtime, result.shell_id);
            fs::write(runtime.workspace_root.join("release-controller-child"), b"release").unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                let finished = owner.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
                    .controller.as_ref().unwrap().is_finished();
                if finished { break; }
                assert!(Instant::now() < deadline, "controlled controller exit was not observed");
                thread::sleep(FOREGROUND_POLL_INTERVAL);
            }
            assert!(runtime.background.lock().unwrap().contains_key(&result.shell_id));
            {
                let state = owner.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                assert!(state.resources.state.status.is_some());
                assert!(state.resources.recovery.stdout_worker.is_some());
                assert!(state.resources.recovery.stderr_worker.is_some());
                assert_eq!(state.resources.state.signals_sent, 0);
            }
            let read = ReadArgs {
                path: format!("{BOX_TERMINAL_VIRTUAL_PREFIX}{}.txt", result.shell_id),
                tool_call_id: "controller-observation".into(), offset: None, limit: None, encoding_hint: None,
            };
            assert!(matches!(runtime.read(&read).result, Some(read_result::Result::Success(_))));
            {
                let state = owner.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                assert!(state.controller.is_none(), "actual terminated JoinHandle was observed");
                assert!(state.phase == BackgroundPhase::Recovery);
                assert!(state.failure.is_some());
                assert!(!state.resources.state.capture_complete);
            }
            assert!(runtime.stop().is_err(), "controller loss does not erase inherited reader ownership");
            assert_eq!(owner.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
                .resources.state.signals_sent, 0);
            fs::write(runtime.workspace_root.join("release-controller-pipe"), b"release").unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                let finished = {
                    let state = owner.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    state.resources.recovery.stdout_worker.as_ref().unwrap().is_finished()
                        && state.resources.recovery.stderr_worker.as_ref().unwrap().is_finished()
                };
                if finished { break; }
                assert!(Instant::now() < deadline, "released actual readers did not reach EOF");
                thread::sleep(FOREGROUND_POLL_INTERVAL);
            }
            assert!(matches!(runtime.read(&read).result, Some(read_result::Result::Success(_))));
            assert!(!runtime.background.lock().unwrap().contains_key(&result.shell_id));
            let text = fs::read_to_string(&owner.terminal_path).unwrap();
            assert!(text.contains("controller-prefix"));
            assert!(text.contains("controller-tail"));
            assert!(text.contains("\nerror: background controller "));
            assert!(text.contains("\nended_at:"));
            assert!(!text.contains("\nexit_code:"), "controller loss never becomes fabricated status");
            assert_eq!(fs::read(runtime.workspace_root.join("effect-count")).unwrap(), b"effect\n");
            runtime.stop().expect("same registry observed later real recovery");
        }
    }

    #[cfg(unix)]
    #[test]
    fn background_custody_full_pipe_input_does_not_block_registry_or_stop() {
        use std::os::fd::{AsRawFd, FromRawFd};
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let _cleanup = BackgroundFixture::new(&runtime, &["release-input"]);
        let result = background_success(runtime.spawn_background(&background_args(
            "printf ready > input-ready; while [ ! -f release-input ]; do sleep 0.01; done; exit 0",
        )));
        wait_file_bytes(&runtime.workspace_root.join("input-ready"), b"ready");
        let owner = background_owner(&runtime, result.shell_id);
        // Establish actual kernel backpressure before the API write. The child
        // cannot consume input: its explicit release loop contains no read.
        {
            let input = owner.stdin.lock().unwrap().as_ref().cloned().unwrap();
            let stdin = input.lock().unwrap();
            let descriptor = stdin.as_raw_fd();
            let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
            assert!(flags >= 0);
            let duplicate = unsafe { libc::dup(descriptor) };
            assert!(duplicate >= 0);
            let mut filler = unsafe { File::from_raw_fd(duplicate) };
            assert_eq!(unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) }, 0);
            let mut filled = 0;
            let block = [b'x'; 4096];
            for bytes in [block.as_slice(), &b"x"[..]] {
                loop {
                    match filler.write(bytes) {
                        Ok(count) => { assert!(count > 0); filled += count; }
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(error) => panic!("actual input pipe fill failed: {error}"),
                    }
                }
            }
            assert!(filled > 0);
            assert_eq!(unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags) }, 0);
        }
        let writer_runtime = Arc::clone(&runtime);
        let shell_id = result.shell_id;
        let writer = thread::spawn(move || writer_runtime.write_stdin(&WriteShellStdinArgs {
            shell_id, chars: "z".into(),
        }));
        let deadline = Instant::now() + Duration::from_secs(3);
        while owner.active_input.load(Ordering::Acquire) != 1 {
            assert!(Instant::now() < deadline, "actual input call did not enter");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        assert!(!writer.is_finished(), "a full pipe cannot complete before child release");

        let sibling_runtime = Arc::clone(&runtime);
        let (sibling_tx, sibling_rx) = mpsc::channel();
        let sibling = thread::spawn(move || {
            let result = sibling_runtime.spawn_background(&background_args("printf sibling"));
            let _ = sibling_tx.send(result);
        });
        let admitted = sibling_rx.recv_timeout(Duration::from_secs(2));
        if admitted.is_err() {
            fs::write(runtime.workspace_root.join("release-input"), b"release").unwrap();
        }
        let sibling_result = background_success(admitted.expect("blocked input must release the registry"));
        sibling.join().unwrap();
        wait_background_removed(&runtime, sibling_result.shell_id);

        let stopping_runtime = Arc::clone(&runtime);
        let (stopped_tx, stopped_rx) = mpsc::channel();
        let stopper = thread::spawn(move || {
            let _ = stopped_tx.send(stopping_runtime.stop());
        });
        let stopped = stopped_rx.recv_timeout(Duration::from_secs(4));
        if stopped.is_err() {
            fs::write(runtime.workspace_root.join("release-input"), b"release").unwrap();
        }
        stopped.expect("blocked input must not deadlock stop").expect("owned child and input settled");
        stopper.join().unwrap();
        let written = writer.join().expect("actual blocked writer returned after child termination");
        assert!(matches!(written.result, Some(write_shell_stdin_result::Result::Error(_))));
        assert_eq!(owner.active_input.load(Ordering::Acquire), 0);
        assert!(runtime.background.lock().unwrap().is_empty());
        let state = owner.state.lock().unwrap();
        assert!(state.resources.state.status.is_some());
        assert!(state.phase == BackgroundPhase::Settled);
    }

    #[test]
    fn background_custody_observed_read_wait_and_worker_failures_never_fabricate_exit() {
        for mode in ["stdout-read", "stderr-read", "stdout-worker", "stderr-worker",
            "stdout-wait", "stderr-wait"]
        {
            let (root, runtime) = runtime();
            let runtime = Arc::new(runtime);
            let _cleanup = BackgroundFixture::new(&runtime, &["release-fault-output"]);
            *runtime.capture_probe.lock().unwrap() =
                CaptureProbe::new(mode, root.path().join("background-fault.json")).map(Arc::new);
            let redirect = if mode.starts_with("stderr") { " >&2" } else { "" };
            let command = format!(
                "printf 'effect\\n' > effect-count; \
                 while [ ! -f release-fault-output ]; do sleep 0.01; done; \
                 printf SC_CAPTURE_PREFIX{redirect}; while :; do sleep 1; done",
            );
            let result = background_success(runtime.spawn_background(&background_args(&command)));
            let owner = background_owner(&runtime, result.shell_id);
            assert!(!root.path().join("background-fault.json").exists(),
                "valid admission precedes the explicitly released fault output");
            fs::write(runtime.workspace_root.join("release-fault-output"), b"release").unwrap();
            wait_background_removed(&runtime, result.shell_id);
            let observed: serde_json::Value = serde_json::from_slice(
                &fs::read(root.path().join("background-fault.json")).unwrap(),
            ).unwrap();
            assert_eq!(observed["mode"], mode);
            assert_eq!(observed["prefix"], "SC_CAPTURE_PREFIX");
            assert_eq!(observed["childPid"], result.pid.unwrap());
            assert!(observed["bytesObserved"].as_u64().unwrap() >= 17);
            let text = fs::read_to_string(&owner.terminal_path).unwrap();
            assert!(text.contains("SC_CAPTURE_PREFIX"), "actual prefix remains diagnostic output");
            assert!(text.contains("\nerror:"));
            assert!(text.contains("\nended_at:"));
            assert!(!text.contains("\nexit_code:"), "{mode} must not become exit 1 or success");
            assert_eq!(fs::read(runtime.workspace_root.join("effect-count")).unwrap(), b"effect\n");
            let state = owner.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            assert!(state.resources.state.status.is_some());
            assert!(state.resources.state.capture_complete);
            assert!(state.failure.is_some());
            assert!(state.footer_written);
            assert!(state.phase == BackgroundPhase::Settled);
        }
    }

    #[test]
    fn background_custody_failed_writer_and_footer_publication_remain_pending() {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let _cleanup = BackgroundFixture::new(&runtime, &["release-unwritable"]);
        let result = background_success(runtime.spawn_background(&background_args(
            "printf 'effect\\n' > effect-count; \
             while [ ! -f release-unwritable ]; do sleep 0.01; done; \
             printf unwritable-output; while :; do sleep 1; done",
        )));
        let owner = background_owner(&runtime, result.shell_id);
        // Replace only this fixture's actual writer with a read-only descriptor
        // to the same owned file. The next real write, then footer write, fail.
        *owner.writer.lock().unwrap() = File::open(&owner.terminal_path).unwrap();
        fs::write(runtime.workspace_root.join("release-unwritable"), b"release").unwrap();
        let deadline = Instant::now() + Duration::from_secs(4);
        loop {
            let failed = owner.state.lock().unwrap().footer_failed;
            if failed { break; }
            assert!(Instant::now() < deadline, "actual writer failure did not reach footer observation");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        {
            let state = owner.state.lock().unwrap();
            assert_eq!(*state.resources.recovery.failure.lock().unwrap(), Some("stream output write failed"));
            assert!(state.resources.state.status.is_some());
            assert!(state.resources.state.capture_complete);
            assert!(state.footer_failed);
            assert!(!state.footer_written);
            assert!(state.phase != BackgroundPhase::Settled);
        }
        assert!(runtime.background.lock().unwrap().contains_key(&result.shell_id));
        let text = fs::read_to_string(&owner.terminal_path).unwrap();
        assert!(!text.contains("\nexit_code:"));
        assert!(!text.contains("\nended_at:"), "failed publication is not a terminal receipt");
        let started = Instant::now();
        assert!(runtime.stop().is_err(), "unobserved terminal publication cannot claim clean completion");
        assert!(started.elapsed() < Duration::from_secs(4));
        assert!(runtime.background.lock().unwrap().contains_key(&result.shell_id));
        assert_eq!(fs::read(runtime.workspace_root.join("effect-count")).unwrap(), b"effect\n");
        let mut state = owner.state.lock().unwrap();
        let signals = state.resources.state.signals_sent;
        state.resources.state.signal(true).unwrap();
        assert_eq!(state.resources.state.signals_sent, signals);
    }

    #[test]
    fn background_custody_handoff_id_exhaustion_retains_entered_capture() {
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let _cleanup = BackgroundFixture::new(&runtime, &[]);
        runtime.next_shell_id.store(u32::MAX, Ordering::Relaxed);
        let mut held = HeldFileStream::new(&runtime, "exhausted-handoff", true);
        held.resume();
        let deadline = Instant::now() + Duration::from_secs(8);
        while !held.worker.as_ref().unwrap().is_finished() {
            assert!(Instant::now() < deadline, "timed handoff refusal did not observe entered cleanup");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
        let events = held.finish();
        assert_stream_file_result(&events, true);
        assert!(events.iter().any(|event| matches!(&event.element,
            Some(exec_stream_element::Element::ExecClientControlMessage(
                ExecClientControlMessage {
                    message: Some(exec_client_control_message::Message::Throw(error)),
                },
            )) if error.error == "background shell id space exhausted")));
        assert_eq!(runtime.next_shell_id.load(Ordering::Relaxed), u32::MAX);
        assert_eq!(fs::read(runtime.workspace_root.join("exhausted-handoff-effect")).unwrap(), b"effect\n");
        assert!(runtime.foreground_processes.lock().unwrap().is_empty());
        assert!(runtime.background.lock().unwrap().is_empty());
        assert_eq!(fs::read_dir(&runtime.terminals_directory).unwrap().count(), 0);
        let events = runtime.execute(stream_file_request(foreground_args("printf foreground-still-works")));
        assert_stream_file_result(&events, false);
        assert_eq!(stream_file_stdout(&events), "foreground-still-works");
    }

    #[test]
    fn background_custody_setup_unwind_preserves_actual_child_and_capture_cleanup() {
        // The command is repeated in frontmatter before any reader starts.
        // Only the body can establish the real captured-output precondition.
        let captured_prefix = |content: &str| content.strip_prefix("---\n")
            .and_then(|rest| rest.split_once("\n---\n"))
            .is_some_and(|(_, output)| output.contains("SC_CAPTURE_PREFIX"));
        let command = "trap '' TERM; printf 'effect\\n' > effect-count; printf SC_CAPTURE_PREFIX; \
                       while :; do sleep 1; done";
        let header_only = terminal_frontmatter(&background_args(command), None, Utc::now());
        assert!(header_only.contains("SC_CAPTURE_PREFIX"));
        assert!(!captured_prefix(&header_only), "command metadata is not captured child output");
        let (_root, runtime) = runtime();
        let runtime = Arc::new(runtime);
        let _cleanup = BackgroundFixture::new(&runtime, &[]);
        *runtime.background_controller_fault.lock().unwrap() = Some(BackgroundControllerFault::PanicBeforeCreate);
        let (release, gate) = mpsc::channel();
        *runtime.background_controller_gate.lock().unwrap() = Some(gate);
        let worker_runtime = Arc::clone(&runtime);
        let worker = thread::spawn(move || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker_runtime.execute(background_request(command))))
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        let owner = loop {
            let entry = runtime.background.lock().unwrap().get(&1).cloned();
            if let Some(owner) = entry {
                if captured_prefix(&fs::read_to_string(&owner.terminal_path).unwrap_or_default()) {
                    break owner;
                }
            }
            assert!(Instant::now() < deadline, "actual prefix must precede controlled setup unwind");
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        };
        assert_eq!(fs::read(runtime.workspace_root.join("effect-count")).unwrap(), b"effect\n");
        release.send(()).unwrap();
        let started = Instant::now();
        assert!(worker.join().expect("unwind was caught by the fixture").is_err());
        assert!(started.elapsed() < Duration::from_secs(4));
        assert!(runtime.background.lock().unwrap().is_empty());
        assert!(!owner.terminal_path.exists());
        let mut state = owner.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(state.resources.state.status.is_some());
        assert!(state.resources.state.capture_complete);
        assert!(state.resources.recovery.stdout_worker.is_none());
        assert!(state.resources.recovery.stderr_worker.is_none());
        assert!(state.resources.state.signals_sent >= 2);
        let signals = state.resources.state.signals_sent;
        state.resources.state.signal(true).unwrap();
        assert_eq!(state.resources.state.signals_sent, signals);
    }

    #[test]
    fn binary_read_preserves_bytes_and_rejects_ranges_and_oversize() {
        let (_root,runtime)=runtime();let bytes=b"\x89PNG\r\n\x1a\n\x00\xff";
        fs::write(runtime.workspace_root.join("image.png"),bytes).unwrap();
        let args=ReadArgs {path:"/workspace/image.png".into(),tool_call_id:"binary".into(),offset:None,limit:None,encoding_hint:None};
        let result=runtime.read(&args);let Some(read_result::Result::Success(result))=result.result else {panic!("binary read rejected")};
        assert_eq!(result.output,Some(read_success::Output::Data(bytes.to_vec())));
        assert!(!result.truncated);assert_eq!(result.file_size,bytes.len() as i64);
        assert!(matches!(runtime.read(&ReadArgs {limit:Some(1),..args.clone()}).result,Some(read_result::Result::InvalidFile(_))));
        let file=File::create(runtime.workspace_root.join("large")).unwrap();file.set_len(8*1024*1024+1).unwrap();
        assert!(matches!(runtime.read(&ReadArgs {path:"/workspace/large".into(),..args}).result,Some(read_result::Result::InvalidFile(_))));
    }

    #[test]
    fn exec_connect_envelope_is_exact_and_bare_proto_stays_compatible() {
        let payload=ExecServerMessage {id:7,exec_id:"wire".into(),message:Some(exec_server_message::Message::ReadArgs(ReadArgs {path:"/workspace/a".into(),tool_call_id:"t".into(),offset:None,limit:None,encoding_hint:None}))}.encode_to_vec();
        let request=|body:Vec<u8>,content:&str|HttpRequest {method:"POST".into(),path:"/agent.v1.ExecService/Exec".into(),headers:HashMap::from([("content-type".into(),content.into())]),body};
        assert_eq!(exec_request_body(&request(payload.clone(),"application/proto")).unwrap(),payload);
        let mut framed=vec![0];framed.extend_from_slice(&(payload.len() as u32).to_be_bytes());framed.extend_from_slice(&payload);
        assert_eq!(exec_request_body(&request(framed.clone(),"application/connect+proto")).unwrap(),payload);
        let mut wrong_flag=framed.clone();wrong_flag[0]=1;
        let mut trailing=framed.clone();trailing.push(0);
        let mut bad_length=framed.clone();bad_length[4]=0;
        for body in [vec![],vec![0,0,0,0,0],wrong_flag,trailing,bad_length] {
            assert!(exec_request_body(&request(body,"application/connect+proto")).is_err());
        }
        assert!(exec_request_body(&request(payload,"application/json")).is_err());
    }

    #[test]
    fn frozen_write_tags_dispatch_actual_atomic_bytes_and_replace() {
        let (_root,runtime)=runtime();
        let args=WriteArgs {path:"/workspace/wire.txt".into(),tool_call_id:"tool".into(),file_bytes:b"wire".to_vec()};
        let encoded=args.encode_to_vec();assert!(encoded.windows(2).any(|bytes|bytes==[0x1a,4]));assert!(encoded.windows(2).any(|bytes|bytes==[0x2a,4]));
        let request=ExecServerMessage {id:9,exec_id:"write".into(),message:Some(exec_server_message::Message::WriteArgs(args.clone()))};
        let output=runtime.execute(request);
        assert!(matches!(&output[0].element,Some(exec_stream_element::Element::ExecClientMessage(ExecClientMessage {message:Some(exec_client_message::Message::WriteResult(WriteResult {result:Some(write_result::Result::Success(_))})),..}))));
        assert_eq!(fs::read(runtime.workspace_root.join("wire.txt")).unwrap(),b"wire");
        assert!(matches!(runtime.write(&WriteArgs {file_bytes:b"replace".to_vec(),..args}).result,Some(write_result::Result::Success(_))));
        assert_eq!(fs::read(runtime.workspace_root.join("wire.txt")).unwrap(),b"replace");
        assert!(!fs::read_dir(&runtime.workspace_root).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with(".fabushi-write-")));
    }

    #[test]
    fn write_rejects_escape_symlink_parent_target_and_oversize_without_side_effects() {
        let (root,runtime)=runtime();let outside=root.path().join("outside");fs::create_dir(&outside).unwrap();fs::write(outside.join("protected"),b"original").unwrap();
        symlink(&outside,runtime.workspace_root.join("linked-parent")).unwrap();
        symlink(outside.join("protected"),runtime.workspace_root.join("linked-file")).unwrap();
        for (path,bytes) in [("/workspace/linked-parent/protected",b"overwrite".to_vec()),("/workspace/linked-file",b"overwrite".to_vec()),("/workspace/../outside/protected",b"overwrite".to_vec()),("/workspace/large",vec![0;8*1024*1024+1])] {
            assert!(matches!(runtime.write(&WriteArgs {path:path.into(),tool_call_id:"deny".into(),file_bytes:bytes}).result,Some(write_result::Result::Rejected(_))));
        }
        assert_eq!(fs::read(outside.join("protected")).unwrap(),b"original");assert!(!runtime.workspace_root.join("large").exists());
        assert!(!fs::read_dir(&runtime.workspace_root).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with(".fabushi-write-")));
    }

    #[test]
    fn write_parent_symlink_replacement_cannot_redirect_to_outside_file() {
        let (root,runtime)=runtime();let outside=root.path().join("outside");fs::create_dir(&outside).unwrap();fs::write(outside.join("protected"),b"original").unwrap();
        let parent=runtime.workspace_root.join("parent");let held=runtime.workspace_root.join("held");fs::create_dir(&parent).unwrap();
        let attack_parent=parent.clone();let attack_held=held.clone();let attack_outside=outside.clone();
        let attacker=std::thread::spawn(move || {for _ in 0..100 {if fs::rename(&attack_parent,&attack_held).is_ok() {let _=symlink(&attack_outside,&attack_parent);std::thread::yield_now();let _=fs::remove_file(&attack_parent);let _=fs::rename(&attack_held,&attack_parent);}}});
        for _ in 0..100 {let _=runtime.write(&WriteArgs {path:"/workspace/parent/protected".into(),tool_call_id:"race".into(),file_bytes:b"inside".to_vec()});}
        attacker.join().unwrap();assert_eq!(fs::read(outside.join("protected")).unwrap(),b"original");
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
            is_background: false,
            skip_approval: false,
            timeout_behavior: 0,
            hard_timeout: None,
        }).expect("foreground execution and capture observed");
        let Some(shell_result::Result::Success(success)) = result.result else {
            panic!("expected shell success");
        };
        assert_eq!(success.stdout, "works");

        let spawned = runtime.spawn_background(&BackgroundShellSpawnArgs {
            command: "read line; printf 'got:%s\\n' \"$line\"".into(),
            working_directory: "/workspace".into(),
            tool_call_id: String::new(),
        }).expect("background spawn and capture admission observed");
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
                is_background: false,
                skip_approval: false,
                timeout_behavior: 0,
                hard_timeout: None,
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

    #[test]
    fn shell_stream_timeout_backgrounds_the_same_process_into_terminal_registry() {
        let (_root, runtime) = runtime();
        let request = ExecServerMessage {
            id: 43,
            exec_id: "stream-background".into(),
            message: Some(exec_server_message::Message::ShellStreamArgs(ShellArgs {
                command: "printf 'pre\\n'; printf 'start\\n' >> run-count.txt; sleep 0.20; printf 'post\\n'; printf 'end\\n' >> run-count.txt".into(),
                working_directory: "/workspace".into(),
                timeout: 50,
                tool_call_id: "stream-background-tool".into(),
                is_background: false,
                skip_approval: true,
                timeout_behavior: TIMEOUT_BEHAVIOR_BACKGROUND,
                hard_timeout: Some(2_000),
            })),
        };
        let abort = AtomicBool::new(false);
        let mut backgrounded = None;
        runtime
            .execute_with_emitter(request, &abort, |element| {
                if let Some(exec_stream_element::Element::ExecClientMessage(message)) =
                    element.element
                {
                    if let Some(exec_client_message::Message::ShellStream(stream)) =
                        message.message
                    {
                        if let Some(shell_stream::Event::Backgrounded(event)) = stream.event {
                            backgrounded = Some(event);
                        }
                    }
                }
                Ok(())
            })
            .expect("timed stream execution");
        let backgrounded = backgrounded.expect("stream must background after the block window");
        assert_ne!(backgrounded.shell_id, 0);
        assert_eq!(backgrounded.ms_to_wait, Some(50));
        assert_eq!(backgrounded.reason, Some(SHELL_BACKGROUND_REASON_TIMEOUT));
        assert_eq!(backgrounded.command, "printf 'pre\\n'; printf 'start\\n' >> run-count.txt; sleep 0.20; printf 'post\\n'; printf 'end\\n' >> run-count.txt");

        let terminal = runtime
            .terminals_directory
            .join(format!("{}.txt", backgrounded.shell_id));
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let content = fs::read_to_string(&terminal).unwrap_or_default();
            if content.contains("pre")
                && content.contains("post")
                && content.contains("ended_at:")
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "timed-background terminal did not settle"
            );
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            fs::read_to_string(runtime.workspace_root.join("run-count.txt"))
                .expect("run-count"),
            "start\nend\n",
            "timed backgrounding must transfer the same child instead of restarting it"
        );
        let registry_deadline = Instant::now() + Duration::from_secs(1);
        loop {
            let registered = runtime
                .background
                .lock()
                .expect("background registry")
                .contains_key(&backgrounded.shell_id);
            if !registered {
                break;
            }
            assert!(
                Instant::now() < registry_deadline,
                "settled background process must leave the single registry"
            );
            thread::sleep(Duration::from_millis(10));
        }
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
                command: "echo $$ > abort-pid.txt; sleep 10".into(),
                working_directory: "/workspace".into(),
                timeout: 0,
                tool_call_id: "abort-tool".into(),
                is_background: false,
                skip_approval: false,
                timeout_behavior: 0,
                hard_timeout: None,
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
        let pid = loop {
            if let Ok(value) = fs::read_to_string(&pid_path) {
                let value = value.trim();
                if let Ok(pid) = value.parse::<u32>() {
                    if pid > 0 {
                        break pid;
                    }
                }
            }
            assert!(Instant::now() < deadline, "foreground shell did not publish a numeric PID");
            thread::sleep(Duration::from_millis(10));
        };
        let owner = {
            let processes = handle.runtime.foreground_processes.lock().expect("foreground owner");
            processes.values().find_map(|(registered_pid, process)| {
                match process {
                    ForegroundProcess::Capture(owner) if *registered_pid == pid =>
                        Some(Arc::clone(owner)),
                    _ => None,
                }
            }).expect("published PID must belong to the actual foreground capture")
        };
        {
            let mut child = owner.lock().expect("owned child");
            assert_eq!(child.child.id(), pid);
            assert!(child.poll().expect("observe child before disconnect").is_none(),
                "disconnect must act on a live owned child");
            assert_eq!(child.signals_sent, 0, "no termination precedes disconnect");
        }
        drop(stream);

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut exited = false;
        while Instant::now() < deadline {
            let mut child = owner.lock().expect("same owned child");
            if child.poll().expect("observe actual child reap after disconnect").is_some() {
                assert!(child.signals_sent > 0, "disconnect must signal the owned child");
                exited = true;
                break;
            }
            drop(child);
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
