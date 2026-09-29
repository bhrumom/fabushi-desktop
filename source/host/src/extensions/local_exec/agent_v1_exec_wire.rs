//! Artifact-backed Rust/prost bindings for the frozen Grok Bot 0.18
//! `agent.v1` local-exec messages consumed by the shipping Host.
//!
//! The canonical descriptor evidence lives in:
//! - source/packages/proto/generated/agent/v1/exec_pb.ts
//! - source/packages/proto/generated/agent/v1/shell_exec_pb.ts
//! - source/packages/proto/generated/agent/v1/read_exec_pb.ts
//!
//! Only the LocalExec production surface is materialized here. Unsupported
//! ExecClientMessage oneof branches are still recognized for oneof-conflict
//! validation and otherwise ignored exactly like a generated JSON decoder
//! whose result is not consumed by this manager.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use prost::Message;
use serde_json::{Map, Value, json};

pub const FROZEN_EXEC_PB_BLOB_SHA: &str =
    "b3d569d1ad5f923444d472a08efb8bad58c5941f";

fn json_object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{name} JSON must be an object"))
}

fn present_oneof(
    object: &Map<String, Value>,
    fields: &'static [&'static str],
    name: &str,
) -> Result<Option<&'static str>, String> {
    let mut present = fields.iter().copied().filter(|field| object.contains_key(*field));
    let first = present.next();
    if present.next().is_some() {
        return Err(format!("{name} JSON has conflicting oneof fields"));
    }
    Ok(first)
}

fn string_field(object: &Map<String, Value>, key: &str) -> Result<String, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(format!("{key} must be a string")),
    }
}

fn optional_string_field(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<String>, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(format!("{key} must be a string")),
    }
}

fn u32_field(object: &Map<String, Value>, key: &str) -> Result<u32, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(0),
        Some(Value::Number(value)) => value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| format!("{key} must be a uint32")),
        Some(Value::String(value)) => value
            .parse::<u32>()
            .map_err(|_| format!("{key} must be a uint32")),
        Some(_) => Err(format!("{key} must be a uint32")),
    }
}

fn optional_u32_field(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<u32>, String> {
    if !object.contains_key(key) || object.get(key).is_some_and(Value::is_null) {
        return Ok(None);
    }
    u32_field(object, key).map(Some)
}

fn i32_field(object: &Map<String, Value>, key: &str) -> Result<i32, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(0),
        Some(Value::Number(value)) => value
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| format!("{key} must be an int32")),
        Some(Value::String(value)) => value
            .parse::<i32>()
            .map_err(|_| format!("{key} must be an int32")),
        Some(_) => Err(format!("{key} must be an int32")),
    }
}

fn optional_i32_field(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<i32>, String> {
    if !object.contains_key(key) || object.get(key).is_some_and(Value::is_null) {
        return Ok(None);
    }
    i32_field(object, key).map(Some)
}

fn i64_field(object: &Map<String, Value>, key: &str) -> Result<i64, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(0),
        Some(Value::Number(value)) => value
            .as_i64()
            .ok_or_else(|| format!("{key} must be an int64")),
        Some(Value::String(value)) => value
            .parse::<i64>()
            .map_err(|_| format!("{key} must be an int64")),
        Some(_) => Err(format!("{key} must be an int64")),
    }
}

fn bool_field(object: &Map<String, Value>, key: &str) -> Result<bool, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(format!("{key} must be a bool")),
    }
}

fn optional_bytes_field(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<Vec<u8>>, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => BASE64
            .decode(value)
            .map(Some)
            .map_err(|_| format!("{key} must be base64 bytes")),
        Some(_) => Err(format!("{key} must be base64 bytes")),
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ExecClientStreamClose {
    #[prost(uint32, tag = "1")]
    pub id: u32,
}

#[derive(Clone, PartialEq, Message)]
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

#[derive(Clone, PartialEq, Message)]
pub struct ExecClientHeartbeat {
    #[prost(uint32, tag = "1")]
    pub id: u32,
}

#[derive(Clone, PartialEq, Message)]
pub struct ExecClientControlMessage {
    #[prost(oneof = "exec_client_control_message::Message", tags = "1, 2, 3")]
    pub message: Option<exec_client_control_message::Message>,
}

pub mod exec_client_control_message {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Message {
        #[prost(message, tag = "1")]
        StreamClose(super::ExecClientStreamClose),
        #[prost(message, tag = "2")]
        Throw(super::ExecClientThrow),
        #[prost(message, tag = "3")]
        Heartbeat(super::ExecClientHeartbeat),
    }
}

impl ExecClientControlMessage {
    pub fn from_json_ignoring_unknown_fields(value: &Value) -> Result<Self, String> {
        let object = json_object(value, "ExecClientControlMessage")?;
        let message = match present_oneof(
            object,
            &["streamClose", "throw", "heartbeat"],
            "ExecClientControlMessage",
        )? {
            Some("streamClose") => {
                let value = json_object(object.get("streamClose").unwrap(), "ExecClientStreamClose")?;
                Some(exec_client_control_message::Message::StreamClose(
                    ExecClientStreamClose { id: u32_field(value, "id")? },
                ))
            }
            Some("throw") => {
                let value = json_object(object.get("throw").unwrap(), "ExecClientThrow")?;
                Some(exec_client_control_message::Message::Throw(ExecClientThrow {
                    id: u32_field(value, "id")?,
                    error: string_field(value, "error")?,
                    stack_trace: optional_string_field(value, "stackTrace")?,
                    error_code: optional_string_field(value, "errorCode")?,
                }))
            }
            Some("heartbeat") => {
                let value = json_object(object.get("heartbeat").unwrap(), "ExecClientHeartbeat")?;
                Some(exec_client_control_message::Message::Heartbeat(
                    ExecClientHeartbeat { id: u32_field(value, "id")? },
                ))
            }
            _ => None,
        };
        Ok(Self { message })
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ShellStreamStdout {
    #[prost(string, tag = "1")]
    pub data: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct ShellStreamStderr {
    #[prost(string, tag = "1")]
    pub data: String,
}

#[derive(Clone, PartialEq, Message)]
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

#[derive(Clone, PartialEq, Message)]
pub struct ShellStreamStart {}

#[derive(Clone, PartialEq, Message)]
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

#[derive(Clone, PartialEq, Message)]
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

#[derive(Clone, PartialEq, Message)]
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

#[derive(Clone, PartialEq, Message)]
pub struct ShellStreamHookContext {}

#[derive(Clone, PartialEq, Message)]
pub struct ShellSandboxUnsupported {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub working_directory: String,
    #[prost(string, tag = "3")]
    pub sandbox_policy_type: String,
    #[prost(string, tag = "4")]
    pub reason: String,
    #[prost(bool, tag = "5")]
    pub is_readonly: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct ShellStream {
    #[prost(oneof = "shell_stream::Event", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9")]
    pub event: Option<shell_stream::Event>,
}

pub mod shell_stream {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Event {
        #[prost(message, tag = "1")]
        Stdout(super::ShellStreamStdout),
        #[prost(message, tag = "2")]
        Stderr(super::ShellStreamStderr),
        #[prost(message, tag = "3")]
        Exit(super::ShellStreamExit),
        #[prost(message, tag = "4")]
        Start(super::ShellStreamStart),
        #[prost(message, tag = "5")]
        Rejected(super::ShellRejected),
        #[prost(message, tag = "6")]
        PermissionDenied(super::ShellPermissionDenied),
        #[prost(message, tag = "7")]
        Backgrounded(super::ShellStreamBackgrounded),
        #[prost(message, tag = "8")]
        HookContext(super::ShellStreamHookContext),
        #[prost(message, tag = "9")]
        SandboxUnsupported(super::ShellSandboxUnsupported),
    }
}

impl ShellStream {
    fn from_json(value: &Value) -> Result<Self, String> {
        let object = json_object(value, "ShellStream")?;
        let event = match present_oneof(
            object,
            &[
                "stdout",
                "stderr",
                "exit",
                "start",
                "rejected",
                "permissionDenied",
                "backgrounded",
                "hookContext",
                "sandboxUnsupported",
            ],
            "ShellStream",
        )? {
            Some("stdout") => {
                let value = json_object(object.get("stdout").unwrap(), "ShellStreamStdout")?;
                Some(shell_stream::Event::Stdout(ShellStreamStdout {
                    data: string_field(value, "data")?,
                }))
            }
            Some("stderr") => {
                let value = json_object(object.get("stderr").unwrap(), "ShellStreamStderr")?;
                Some(shell_stream::Event::Stderr(ShellStreamStderr {
                    data: string_field(value, "data")?,
                }))
            }
            Some("exit") => {
                let value = json_object(object.get("exit").unwrap(), "ShellStreamExit")?;
                Some(shell_stream::Event::Exit(ShellStreamExit {
                    code: u32_field(value, "code")?,
                    cwd: string_field(value, "cwd")?,
                    aborted: bool_field(value, "aborted")?,
                    local_execution_time_ms: optional_i32_field(value, "localExecutionTimeMs")?,
                }))
            }
            Some("start") => {
                json_object(object.get("start").unwrap(), "ShellStreamStart")?;
                Some(shell_stream::Event::Start(ShellStreamStart {}))
            }
            Some("rejected") => {
                let value = json_object(object.get("rejected").unwrap(), "ShellRejected")?;
                Some(shell_stream::Event::Rejected(ShellRejected {
                    command: string_field(value, "command")?,
                    working_directory: string_field(value, "workingDirectory")?,
                    reason: string_field(value, "reason")?,
                    is_readonly: bool_field(value, "isReadonly")?,
                }))
            }
            Some("permissionDenied") => {
                let value = json_object(object.get("permissionDenied").unwrap(), "ShellPermissionDenied")?;
                Some(shell_stream::Event::PermissionDenied(ShellPermissionDenied {
                    command: string_field(value, "command")?,
                    working_directory: string_field(value, "workingDirectory")?,
                    error: string_field(value, "error")?,
                    is_readonly: bool_field(value, "isReadonly")?,
                }))
            }
            Some("backgrounded") => {
                let value = json_object(object.get("backgrounded").unwrap(), "ShellStreamBackgrounded")?;
                Some(shell_stream::Event::Backgrounded(ShellStreamBackgrounded {
                    shell_id: u32_field(value, "shellId")?,
                    command: string_field(value, "command")?,
                    working_directory: string_field(value, "workingDirectory")?,
                    pid: optional_u32_field(value, "pid")?,
                    ms_to_wait: optional_i32_field(value, "msToWait")?,
                    reason: optional_i32_field(value, "reason")?,
                }))
            }
            Some("hookContext") => {
                json_object(object.get("hookContext").unwrap(), "ShellStreamHookContext")?;
                Some(shell_stream::Event::HookContext(ShellStreamHookContext {}))
            }
            Some("sandboxUnsupported") => {
                let value = json_object(object.get("sandboxUnsupported").unwrap(), "ShellSandboxUnsupported")?;
                Some(shell_stream::Event::SandboxUnsupported(ShellSandboxUnsupported {
                    command: string_field(value, "command")?,
                    working_directory: string_field(value, "workingDirectory")?,
                    sandbox_policy_type: string_field(value, "sandboxPolicyType")?,
                    reason: string_field(value, "reason")?,
                    is_readonly: bool_field(value, "isReadonly")?,
                }))
            }
            _ => None,
        };
        Ok(Self { event })
    }
}

#[derive(Clone, PartialEq, Message)]
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
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Output {
        #[prost(string, tag = "2")]
        Content(String),
        #[prost(bytes = "vec", tag = "5")]
        Data(Vec<u8>),
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ReadError {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "2")]
    pub error: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct ReadRejected {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "2")]
    pub reason: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct ReadFileNotFound {
    #[prost(string, tag = "1")]
    pub path: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct ReadPermissionDenied {
    #[prost(string, tag = "1")]
    pub path: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct ReadInvalidFile {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "2")]
    pub reason: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct ReadResult {
    #[prost(oneof = "read_result::Result", tags = "1, 2, 3, 4, 5, 6")]
    pub result: Option<read_result::Result>,
}

pub mod read_result {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Result {
        #[prost(message, tag = "1")]
        Success(super::ReadSuccess),
        #[prost(message, tag = "2")]
        Error(super::ReadError),
        #[prost(message, tag = "3")]
        Rejected(super::ReadRejected),
        #[prost(message, tag = "4")]
        FileNotFound(super::ReadFileNotFound),
        #[prost(message, tag = "5")]
        PermissionDenied(super::ReadPermissionDenied),
        #[prost(message, tag = "6")]
        InvalidFile(super::ReadInvalidFile),
    }
}

impl ReadResult {
    fn from_json(value: &Value) -> Result<Self, String> {
        let object = json_object(value, "ReadResult")?;
        let result = match present_oneof(
            object,
            &["success", "error", "rejected", "fileNotFound", "permissionDenied", "invalidFile"],
            "ReadResult",
        )? {
            Some("success") => {
                let value = json_object(object.get("success").unwrap(), "ReadSuccess")?;
                let output = match present_oneof(value, &["content", "data"], "ReadSuccess")? {
                    Some("content") => Some(read_success::Output::Content(string_field(value, "content")?)),
                    Some("data") => {
                        let bytes = optional_bytes_field(value, "data")?.unwrap_or_default();
                        Some(read_success::Output::Data(bytes))
                    }
                    _ => None,
                };
                Some(read_result::Result::Success(ReadSuccess {
                    path: string_field(value, "path")?,
                    output,
                    total_lines: i32_field(value, "totalLines")?,
                    file_size: i64_field(value, "fileSize")?,
                    truncated: bool_field(value, "truncated")?,
                    output_blob_id: optional_bytes_field(value, "outputBlobId")?,
                    range_applied: bool_field(value, "rangeApplied")?,
                }))
            }
            Some("error") => {
                let value = json_object(object.get("error").unwrap(), "ReadError")?;
                Some(read_result::Result::Error(ReadError {
                    path: string_field(value, "path")?,
                    error: string_field(value, "error")?,
                }))
            }
            Some("rejected") => {
                let value = json_object(object.get("rejected").unwrap(), "ReadRejected")?;
                Some(read_result::Result::Rejected(ReadRejected {
                    path: string_field(value, "path")?,
                    reason: string_field(value, "reason")?,
                }))
            }
            Some("fileNotFound") => {
                let value = json_object(object.get("fileNotFound").unwrap(), "ReadFileNotFound")?;
                Some(read_result::Result::FileNotFound(ReadFileNotFound {
                    path: string_field(value, "path")?,
                }))
            }
            Some("permissionDenied") => {
                let value = json_object(object.get("permissionDenied").unwrap(), "ReadPermissionDenied")?;
                Some(read_result::Result::PermissionDenied(ReadPermissionDenied {
                    path: string_field(value, "path")?,
                }))
            }
            Some("invalidFile") => {
                let value = json_object(object.get("invalidFile").unwrap(), "ReadInvalidFile")?;
                Some(read_result::Result::InvalidFile(ReadInvalidFile {
                    path: string_field(value, "path")?,
                    reason: string_field(value, "reason")?,
                }))
            }
            _ => None,
        };
        Ok(Self { result })
    }

    pub fn to_json(&self) -> Value {
        match self.result.as_ref() {
            Some(read_result::Result::Success(value)) => {
                let mut success = Map::new();
                success.insert("path".into(), Value::String(value.path.clone()));
                match value.output.as_ref() {
                    Some(read_success::Output::Content(content)) => {
                        success.insert("content".into(), Value::String(content.clone()));
                    }
                    Some(read_success::Output::Data(data)) => {
                        success.insert("data".into(), Value::String(BASE64.encode(data)));
                    }
                    None => {}
                }
                if value.total_lines != 0 {
                    success.insert("totalLines".into(), json!(value.total_lines));
                }
                if value.file_size != 0 {
                    success.insert("fileSize".into(), Value::String(value.file_size.to_string()));
                }
                if value.truncated {
                    success.insert("truncated".into(), Value::Bool(true));
                }
                if let Some(output_blob_id) = value.output_blob_id.as_ref() {
                    success.insert("outputBlobId".into(), Value::String(BASE64.encode(output_blob_id)));
                }
                if value.range_applied {
                    success.insert("rangeApplied".into(), Value::Bool(true));
                }
                json!({"success": Value::Object(success)})
            }
            Some(read_result::Result::Error(value)) => {
                json!({"error": {"path": value.path, "error": value.error}})
            }
            Some(read_result::Result::Rejected(value)) => {
                json!({"rejected": {"path": value.path, "reason": value.reason}})
            }
            Some(read_result::Result::FileNotFound(value)) => {
                json!({"fileNotFound": {"path": value.path}})
            }
            Some(read_result::Result::PermissionDenied(value)) => {
                json!({"permissionDenied": {"path": value.path}})
            }
            Some(read_result::Result::InvalidFile(value)) => {
                json!({"invalidFile": {"path": value.path, "reason": value.reason}})
            }
            None => json!({}),
        }
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ExecClientMessage {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(string, tag = "15")]
    pub exec_id: String,
    #[prost(int32, optional, tag = "39")]
    pub local_execution_time_ms: Option<i32>,
    #[prost(oneof = "exec_client_message::Message", tags = "7, 14")]
    pub message: Option<exec_client_message::Message>,
}

pub mod exec_client_message {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Message {
        #[prost(message, tag = "7")]
        ReadResult(super::ReadResult),
        #[prost(message, tag = "14")]
        ShellStream(super::ShellStream),
    }
}

const EXEC_CLIENT_ONEOF_JSON_FIELDS: &[&str] = &[
    "shellResult", "writeResult", "deleteResult", "grepResult", "readResult",
    "redactedReadResult", "lsResult", "diagnosticsResult", "requestContextResult",
    "mcpResult", "shellStream", "backgroundShellSpawnResult",
    "listMcpResourcesExecResult", "readMcpResourceExecResult", "mcpStateExecResult",
    "fetchResult", "recordScreenResult", "computerUseResult", "writeShellStdinResult",
    "executeHookResult", "subagentResult", "forceBackgroundShellResult",
    "forceBackgroundSubagentResult", "subagentAwaitResult",
    "smartModeClassifierResult", "canvasDiagnosticsResult",
    "shellAllowlistPrecheckResult", "mcpAllowlistPrecheckResult",
    "webFetchAllowlistPrecheckResult", "gitDiffResponse", "piReadResult",
    "piBashResult", "piEditResult", "piWriteResult", "piGrepResult",
    "piFindResult", "piLsResult", "conversationSearchResult",
    "agentStoreConflictResult", "miniSweAgentBashResult", "adoptResult",
];

impl ExecClientMessage {
    pub fn from_json_ignoring_unknown_fields(value: &Value) -> Result<Self, String> {
        let object = json_object(value, "ExecClientMessage")?;
        let branch = present_oneof(object, EXEC_CLIENT_ONEOF_JSON_FIELDS, "ExecClientMessage")?;
        let message = match branch {
            Some("shellStream") => Some(exec_client_message::Message::ShellStream(
                ShellStream::from_json(object.get("shellStream").unwrap())?,
            )),
            Some("readResult") => Some(exec_client_message::Message::ReadResult(
                ReadResult::from_json(object.get("readResult").unwrap())?,
            )),
            _ => None,
        };
        Ok(Self {
            id: u32_field(object, "id")?,
            exec_id: string_field(object, "execId")?,
            local_execution_time_ms: optional_i32_field(object, "localExecutionTimeMs")?,
            message,
        })
    }
}

#[derive(Clone, PartialEq, Message)]
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

#[derive(Clone, PartialEq, Message)]
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

#[derive(Clone, PartialEq, Message)]
pub struct ExecServerMessage {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(string, tag = "15")]
    pub exec_id: String,
    #[prost(oneof = "exec_server_message::Message", tags = "7, 14")]
    pub message: Option<exec_server_message::Message>,
}

pub mod exec_server_message {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Message {
        #[prost(message, tag = "7")]
        ReadArgs(super::ReadArgs),
        #[prost(message, tag = "14")]
        ShellStreamArgs(super::ShellArgs),
    }
}

impl ExecServerMessage {
    pub fn shell_stream(
        id: u32,
        command: &str,
        working_directory: &str,
        tool_call_id: &str,
    ) -> Self {
        Self {
            id,
            exec_id: String::new(),
            message: Some(exec_server_message::Message::ShellStreamArgs(ShellArgs {
                command: command.to_string(),
                working_directory: working_directory.to_string(),
                timeout: 0,
                tool_call_id: tool_call_id.to_string(),
            })),
        }
    }

    pub fn read(
        id: u32,
        path: &str,
        tool_call_id: &str,
        offset: Option<i32>,
        limit: Option<u32>,
        encoding_hint: Option<&str>,
    ) -> Self {
        Self {
            id,
            exec_id: String::new(),
            message: Some(exec_server_message::Message::ReadArgs(ReadArgs {
                path: path.to_string(),
                tool_call_id: tool_call_id.to_string(),
                offset,
                limit,
                encoding_hint: encoding_hint.map(str::to_string),
            })),
        }
    }

    pub fn to_json(&self) -> Value {
        let mut root = Map::new();
        if self.id != 0 {
            root.insert("id".into(), json!(self.id));
        }
        if !self.exec_id.is_empty() {
            root.insert("execId".into(), Value::String(self.exec_id.clone()));
        }
        match self.message.as_ref() {
            Some(exec_server_message::Message::ShellStreamArgs(args)) => {
                let mut value = Map::new();
                if !args.command.is_empty() {
                    value.insert("command".into(), Value::String(args.command.clone()));
                }
                if !args.working_directory.is_empty() {
                    value.insert(
                        "workingDirectory".into(),
                        Value::String(args.working_directory.clone()),
                    );
                }
                if args.timeout != 0 {
                    value.insert("timeout".into(), json!(args.timeout));
                }
                if !args.tool_call_id.is_empty() {
                    value.insert("toolCallId".into(), Value::String(args.tool_call_id.clone()));
                }
                root.insert("shellStreamArgs".into(), Value::Object(value));
            }
            Some(exec_server_message::Message::ReadArgs(args)) => {
                let mut value = Map::new();
                if !args.path.is_empty() {
                    value.insert("path".into(), Value::String(args.path.clone()));
                }
                if !args.tool_call_id.is_empty() {
                    value.insert("toolCallId".into(), Value::String(args.tool_call_id.clone()));
                }
                if let Some(offset) = args.offset {
                    value.insert("offset".into(), json!(offset));
                }
                if let Some(limit) = args.limit {
                    value.insert("limit".into(), json!(limit));
                }
                if let Some(encoding_hint) = args.encoding_hint.as_ref() {
                    value.insert("encodingHint".into(), Value::String(encoding_hint.clone()));
                }
                root.insert("readArgs".into(), Value::Object(value));
            }
            None => {}
        }
        Value::Object(root)
    }
}
