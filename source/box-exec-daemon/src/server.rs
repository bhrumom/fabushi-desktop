use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
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


/// A child entered execution, but its result/capture could not be observed.
/// The private phase is bounded and never contains command or captured bytes.
#[derive(Debug, Clone)]
pub struct ShellExecutionUnknown {
    phase: &'static str,
    cleanup_unresolved: bool,
}

impl std::fmt::Display for ShellExecutionUnknown {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "foreground shell outcome unknown: {}", self.phase)?;
        if self.cleanup_unresolved {
            formatter.write_str("; owned child cleanup unresolved")?;
        }
        Ok(())
    }
}

impl std::error::Error for ShellExecutionUnknown {}

impl ShellExecutionUnknown {
    fn new(phase: &'static str) -> Self {
        Self { phase, cleanup_unresolved: false }
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
    #[cfg(test)]
    signals_sent: usize,
}

impl CapturedChild {
    fn poll(&mut self) -> Result<Option<ExitStatus>, ShellExecutionUnknown> {
        if self.status.is_none() {
            self.status = self.child.try_wait()
                .map_err(|_| ShellExecutionUnknown::new("child wait failed"))?;
        }
        Ok(self.status)
    }

    fn signal(&mut self, force: bool) -> Result<(), ShellExecutionUnknown> {
        // Poll and signal under the same ownership mutex. An unreaped child
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

const FOREGROUND_POLL_INTERVAL: Duration = Duration::from_millis(10);
const FOREGROUND_TERM_GRACE: Duration = Duration::from_millis(250);
const FOREGROUND_CLEANUP_LIMIT: Duration = Duration::from_secs(2);

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
    capture_probe: Mutex<Option<Arc<CaptureProbe>>>,
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
            foreground_processes: Mutex::new(HashMap::new()),
            next_foreground_id: AtomicU64::new(1),
            #[cfg(test)]
            foreground_pid_alias: Mutex::new(None),
            #[cfg(test)]
            capture_observation_gate: Mutex::new(None),
            #[cfg(test)]
            capture_probe: Mutex::new(None),
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
        let cwd = self.resolve_path(&args.working_directory)?;
        emit(client_message(
            id,
            exec_id,
            exec_client_message::Message::ShellStream(ShellStream {
                event: Some(shell_stream::Event::Start(ShellStreamStart {})),
            }),
            None,
        ))?;

        let started_at = Utc::now();
        let started = Instant::now();
        let (registration_id, mut child) = {
            let mut processes = self.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if self.stopping.load(Ordering::Acquire) {
                return Err("box exec daemon is stopping".into());
            }
            let registration_id = self.allocate_foreground_id()?;
            let child = self.spawn_shell(&args.command, &cwd)?;
            processes.insert(registration_id, self.foreground_registration(
                child.id(), ForegroundProcess::Stream,
            ));
            (registration_id, Some(child))
        };
        // Only this request owns/reaps this Child. Registry PID data is never
        // signal authority, including when a test presents a reused number.
        let pid = child.as_ref().expect("stream child").id();
        let stdout = child
            .as_mut()
            .expect("stream child")
            .stdout
            .take()
            .ok_or_else(|| "shell stream stdout was unavailable".to_string())?;
        let stderr = child
            .as_mut()
            .expect("stream child")
            .stderr
            .take()
            .ok_or_else(|| "shell stream stderr was unavailable".to_string())?;

        fs::create_dir_all(&self.terminals_directory).map_err(|error| error.to_string())?;
        let stream_spool_path = self
            .terminals_directory
            .join(format!(".shell-stream-{}.tmp", registration_id.0));
        let stream_spool_writer = Arc::new(Mutex::new(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&stream_spool_path)
                .map_err(|error| error.to_string())?,
        ));

        let (events_tx, events_rx) = mpsc::channel::<ShellPipeEvent>();
        let mut stdout_worker = Some(spawn_stream_pipe(
            stdout,
            events_tx.clone(),
            false,
            Arc::clone(&stream_spool_writer),
        ));
        let mut stderr_worker = Some(spawn_stream_pipe(
            stderr,
            events_tx.clone(),
            true,
            Arc::clone(&stream_spool_writer),
        ));
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
                    kill_process_group(pid);
                    kill_requested = true;
                    break;
                }
            }
            if stream_error.is_some() {
                status = child
                    .as_mut()
                    .expect("stream child")
                    .wait()
                    .ok();
                break;
            }
            match child
                .as_mut()
                .expect("stream child")
                .try_wait()
                .map_err(|error| error.to_string())?
            {
                Some(done) => {
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
                    child.take().expect("stream child"),
                    args,
                    pid,
                    registration_id,
                    started,
                    started_at,
                    stdout_worker.take().expect("stdout worker"),
                    stderr_worker.take().expect("stderr worker"),
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

        if let Some(worker) = stdout_worker.take() {
            let _ = worker.join();
        }
        if let Some(worker) = stderr_worker.take() {
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
        let status = match status {
            Some(status) => Ok(status),
            None => child
                .as_mut()
                .expect("stream child")
                .wait()
                .map_err(|error| error.to_string()),
        };
        if status.is_ok() {
            self.foreground_processes
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&registration_id);
        }

        drop(stream_spool_writer);
        let _ = fs::remove_file(&stream_spool_path);

        if let Some(error) = stream_error {
            return Err(error);
        }
        let status = status?;
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

    #[allow(clippy::too_many_arguments)]
    fn handoff_shell_stream_to_background(
        &self,
        mut child: Child,
        args: &ShellArgs,
        pid: u32,
        registration_id: ForegroundId,
        started: Instant,
        started_at: chrono::DateTime<Utc>,
        stdout_worker: JoinHandle<()>,
        stderr_worker: JoinHandle<()>,
        stream_spool_writer: Arc<Mutex<File>>,
        stream_spool_path: PathBuf,
    ) -> Result<ShellStreamBackgrounded, String> {
        let shell_id = self.next_shell_id.fetch_add(1, Ordering::Relaxed);
        if shell_id == 0 {
            kill_process_group(pid);
            let reaped = child.wait().is_ok();
            let _ = stdout_worker.join();
            let _ = stderr_worker.join();
            if reaped {
                self.foreground_processes
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .remove(&registration_id);
            }
            let _ = fs::remove_file(&stream_spool_path);
            return Err("background shell id space exhausted".into());
        }

        let stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                kill_process_group(pid);
                let reaped = child.wait().is_ok();
                let _ = stdout_worker.join();
                let _ = stderr_worker.join();
                if reaped {
                    self.foreground_processes
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .remove(&registration_id);
                }
                let _ = fs::remove_file(&stream_spool_path);
                return Err("shell stream stdin was unavailable during background handoff".into());
            }
        };
        let terminal_path = self
            .terminals_directory
            .join(format!("{shell_id}.txt"));
        let background_args = BackgroundShellSpawnArgs {
            command: args.command.clone(),
            working_directory: args.working_directory.clone(),
            tool_call_id: args.tool_call_id.clone(),
        };
        let started_at_ms = started_at.timestamp_millis();

        let setup = (|| -> Result<(), String> {
            let mut current_writer = stream_spool_writer
                .lock()
                .map_err(|_| "shell stream output writer is poisoned".to_string())?;
            current_writer.flush().map_err(|error| error.to_string())?;
            let spooled = fs::read(&stream_spool_path).map_err(|error| error.to_string())?;
            let mut terminal = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&terminal_path)
                .map_err(|error| error.to_string())?;
            terminal
                .write_all(
                    terminal_frontmatter(&background_args, Some(pid), started_at.clone()).as_bytes(),
                )
                .map_err(|error| error.to_string())?;
            terminal
                .write_all(&spooled)
                .map_err(|error| error.to_string())?;
            terminal.flush().map_err(|error| error.to_string())?;
            *current_writer = OpenOptions::new()
                .append(true)
                .open(&terminal_path)
                .map_err(|error| error.to_string())?;
            Ok(())
        })();

        if let Err(error) = setup {
            kill_process_group(pid);
            let reaped = child.wait().is_ok();
            let _ = stdout_worker.join();
            let _ = stderr_worker.join();
            if reaped {
                self.foreground_processes
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .remove(&registration_id);
            }
            let _ = fs::remove_file(&stream_spool_path);
            let _ = fs::remove_file(&terminal_path);
            return Err(error);
        }
        let _ = fs::remove_file(&stream_spool_path);

        let inserted = {
            // Stop's foreground snapshot and this transfer share admission.
            // Either stop sees the foreground invocation, or its later
            // background snapshot sees the admitted handoff.
            let mut foreground = self.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut processes = self.background.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if self.stopping.load(Ordering::Acquire) {
                Err("box exec daemon is stopping".to_string())
            } else if processes.contains_key(&shell_id) {
                Err(format!("background shell id {shell_id} is already registered"))
            } else {
                processes.insert(
                    shell_id,
                    BackgroundProcess {
                        stdin: Arc::new(Mutex::new(stdin)),
                        terminal_path: terminal_path.clone(),
                        pid,
                    },
                );
                foreground.remove(&registration_id);
                Ok(())
            }
        };
        if let Err(error) = inserted {
            kill_process_group(pid);
            let reaped = child.wait().is_ok();
            let _ = stdout_worker.join();
            let _ = stderr_worker.join();
            if reaped {
                self.foreground_processes
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .remove(&registration_id);
            }
            let _ = fs::remove_file(&terminal_path);
            return Err(error);
        }

        let processes = Arc::clone(&self.background);
        let footer_writer = Arc::clone(&stream_spool_writer);
        let hard_timeout_ms = args
            .hard_timeout
            .filter(|timeout| *timeout > 0)
            .map(|timeout| timeout as u64);
        thread::spawn(move || {
            let status = if let Some(hard_timeout_ms) = hard_timeout_ms {
                loop {
                    match child.try_wait() {
                        Ok(Some(status)) => break Some(status),
                        Ok(None) => {}
                        Err(_) => break child.wait().ok(),
                    }
                    if started.elapsed() >= Duration::from_millis(hard_timeout_ms) {
                        kill_process_group(pid);
                        break child.wait().ok();
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            } else {
                child.wait().ok()
            };
            let _ = stdout_worker.join();
            let _ = stderr_worker.join();
            let code = status
                .and_then(|status| status.code())
                .unwrap_or(1);
            if let Ok(mut writer) = footer_writer.lock() {
                let _ = writer.write_all(terminal_footer(code, started_at_ms).as_bytes());
                let _ = writer.flush();
            }
            if let Ok(mut processes) = processes.lock() {
                processes.remove(&shell_id);
            }
        });

        Ok(ShellStreamBackgrounded {
            shell_id,
            command: args.command.clone(),
            working_directory: args.working_directory.clone(),
            pid: Some(pid),
            ms_to_wait: Some(args.timeout),
            reason: Some(SHELL_BACKGROUND_REASON_TIMEOUT),
        })
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

    pub fn stop(&self) -> Result<(), String> {
        self.stopping.store(true, Ordering::Release);
        let foreground = self.foreground_processes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter().map(|(id, (_, process))| (*id, process.clone())).collect::<Vec<_>>();
        let mut unresolved = false;
        for (registration_id, process) in foreground {
            match process {
                // The stream request observes stopping and signals its own
                // unreaped Child. A snapshot's PID may already be recycled.
                ForegroundProcess::Stream => {},
                ForegroundProcess::Capture(owner) => {
                    unresolved |= cleanup_captured_child(&owner).is_err();
                    self.remove_completed_capture(registration_id, &owner);
                }
            }
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
        // Requests still own their workers, including streamed output. Reaping
        // alone is not permission to report that cleanup has finished.
        let deadline = Instant::now() + FOREGROUND_CLEANUP_LIMIT;
        loop {
            let pending = !self.foreground_processes.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .is_empty();
            if !pending {
                return if unresolved {
                    Err("foreground child cleanup was not fully observed".into())
                } else {
                    Ok(())
                };
            }
            if Instant::now() >= deadline {
                return Err("foreground child cleanup remains unresolved".into());
            }
            thread::sleep(FOREGROUND_POLL_INTERVAL);
        }
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
            let owner = Arc::new(Mutex::new(CapturedChild {
                child, status: None, cleanup_unknown: false, capture_complete: false,
                #[cfg(test)]
                signals_sent: 0,
            }));
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
                CaptureFaultKind::Wait => {}
            }
        }
        Ok(())
    }

    fn wait_fault_ready(&self) -> bool {
        self.kind == CaptureFaultKind::Wait && self.observed.load(Ordering::Acquire)
    }
}

fn spawn_stream_pipe<R>(
    mut reader: R,
    sender: mpsc::Sender<ShellPipeEvent>,
    stderr: bool,
    output_writer: Arc<Mutex<File>>,
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
                    {
                        let mut writer = output_writer
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let _ = writer.write_all(&bytes);
                        let _ = writer.flush();
                    }
                    let event = if stderr {
                        ShellPipeEvent::Stderr(bytes)
                    } else {
                        ShellPipeEvent::Stdout(bytes)
                    };
                    // The ShellStream receiver intentionally disappears after a timed
                    // background handoff. Keep draining the same child into the
                    // canonical terminal writer instead of terminating the pipe worker.
                    let _ = sender.send(event);
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
        publish_capture_probe_json(&root.join("stopped.json"), &serde_json::json!({
            "pid": pid, "clean": true, "foregroundCount": remaining,
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
            status: None, cleanup_unknown: false, capture_complete: true, signals_sent: 0,
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
        let mut args = foreground_args("printf handoff-prefix; sleep 1; printf handoff-tail");
        args.timeout = 20;
        args.timeout_behavior = TIMEOUT_BEHAVIOR_BACKGROUND;
        let mut events = Vec::new();
        runtime.shell_stream_live(
            77, "identity-handoff", &args, &AtomicBool::new(false),
            &mut |event| { events.push(event); Ok(()) },
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
