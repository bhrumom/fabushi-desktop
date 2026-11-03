use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use url::Url;

use super::box_env::{BoxEnvironmentControlClient, BoxEnvironmentUpdate};
use super::box_file_transfer::{FileTransferAccessor, ShellExecResult, WriteExecResult};
use super::box_mcp::{
    BoxMcpControlClient, BoxMcpLoadRequest, BoxMcpLoadResponse, ConnectErrorCode,
    ConnectErrorCodeSource,
};
use super::box_remote_accessor::{
    BoxEndpoint, BoxPingControlClient, BoxPingErrorMetadata, BoxRemoteExecClient,
    BoxRemoteExecControlMessage, BoxRemoteExecEnvelope, BoxRemoteExecError, BoxRemoteExecManager,
    BoxTransportOptions, ConnectCode, create_box_transport,
};
use super::box_shell_command::{HostShellArgs, ShellCommandParsingResult};
use super::box_windows::{ShellAccessor, ShellExecutionOutcome, ShellExecutionResult};
use super::protected_path_guard::{SandProtectedPathError, assert_path_outside_protected_roots};
use crate::ports::r#box::SandBoxNoMonitorAvailableError;

pub const PING_PATH: &str = "/agent.v1.ControlService/Ping";
pub const UPDATE_ENVIRONMENT_VARIABLES_PATH: &str =
    "/agent.v1.ControlService/UpdateEnvironmentVariables";
pub const LOAD_MCP_SERVERS_PATH: &str = "/agent.v1.ControlService/LoadMcpServers";
pub const EXEC_PATH: &str = "/agent.v1.ExecService/Exec";
pub const CONNECT_PROTOCOL_VERSION: &str = "1";
pub const CONNECT_STREAM_CONTENT_TYPE: &str = "application/connect+proto";
pub const CONNECT_STREAM_END_FLAG: u8 = 0x02;
pub const PRODUCTION_BOX_RPC_TIMEOUT_MS: u64 = 15_000;
/// Exec calls can contain bounded browser navigation stages longer than unary RPC.
pub const PRODUCTION_BOX_EXEC_TIMEOUT_MS: u64 = 120_000;

#[derive(Debug)]
pub enum ProductionBoxTransportError {
    InvalidBaseUrl(String),
    InvalidHeader(String),
    Io(std::io::Error),
    InvalidHttpResponse(String),
    HttpStatus { status: u16, body: String },
    /// The request write was attempted; the operation may already have run.
    ExecutionOutcomeUnknown(Box<ProductionBoxTransportError>),
}

impl fmt::Display for ProductionBoxTransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBaseUrl(message)
            | Self::InvalidHeader(message)
            | Self::InvalidHttpResponse(message) => formatter.write_str(message),
            Self::Io(error) => error.fmt(formatter),
            Self::ExecutionOutcomeUnknown(cause) => {
                write!(formatter, "box ExecService execution outcome unknown: {cause}")
            }
            Self::HttpStatus { status, body } => {
                write!(formatter, "box ControlService returned HTTP {status}")?;
                if !body.is_empty() {
                    write!(formatter, ": {body}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ProductionBoxTransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::ExecutionOutcomeUnknown(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProductionBoxTransportError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl ProductionBoxTransportError {
    fn execution_outcome_unknown(self) -> Self {
        if matches!(&self, Self::ExecutionOutcomeUnknown(_)) {
            self
        } else {
            Self::ExecutionOutcomeUnknown(Box::new(self))
        }
    }
}

fn http_status_connect_code(status: u16) -> Option<i64> {
    match status {
        400 => Some(3),
        401 => Some(16),
        403 => Some(7),
        404 => Some(5),
        409 => Some(10),
        429 => Some(8),
        499 => Some(1),
        501 => Some(12),
        503 => Some(14),
        504 => Some(4),
        _ => None,
    }
}

impl BoxPingErrorMetadata for ProductionBoxTransportError {
    fn connect_code(&self) -> Option<ConnectCode> {
        match self {
            Self::ExecutionOutcomeUnknown(cause) => cause.connect_code(),
            Self::Io(error) if error.kind() == std::io::ErrorKind::TimedOut => {
                Some(ConnectCode::Number(4))
            }
            Self::HttpStatus { status, .. } => {
                http_status_connect_code(*status).map(ConnectCode::Number)
            }
            _ => None,
        }
    }

    fn system_errno(&self) -> Option<&str> {
        match self {
            Self::ExecutionOutcomeUnknown(cause) => cause.system_errno(),
            Self::Io(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
                Some("ECONNREFUSED")
            }
            _ => None,
        }
    }
}

impl ConnectErrorCodeSource for ProductionBoxTransportError {
    fn connect_error_code(&self) -> Option<ConnectErrorCode> {
        match self {
            Self::ExecutionOutcomeUnknown(cause) => cause.connect_error_code(),
            Self::Io(error) if error.kind() == std::io::ErrorKind::TimedOut => {
                Some(ConnectErrorCode::Number(4))
            }
            Self::HttpStatus { status, .. } => {
                http_status_connect_code(*status).map(ConnectErrorCode::Number)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionBoxTransport {
    options: BoxTransportOptions,
}

impl ProductionBoxTransport {
    pub fn from_endpoint(endpoint: &BoxEndpoint) -> Self {
        create_box_transport(endpoint, |options| Self { options })
    }

    pub fn options(&self) -> &BoxTransportOptions {
        &self.options
    }

    fn request_headers(&self) -> Result<BTreeMap<String, String>, ProductionBoxTransportError> {
        let mut headers = BTreeMap::new();
        for interceptor in &self.options.interceptors {
            interceptor.apply(&mut headers);
        }
        for (name, value) in &headers {
            validate_header(name, value)?;
        }
        Ok(headers)
    }
}

pub struct ProductionBoxControlClient {
    transport: ProductionBoxTransport,
}

pub fn create_production_box_control_client(
    transport: &ProductionBoxTransport,
) -> ProductionBoxControlClient {
    ProductionBoxControlClient {
        transport: transport.clone(),
    }
}

impl<Ctx> BoxPingControlClient<Ctx> for ProductionBoxControlClient {
    type Error = ProductionBoxTransportError;

    fn ping(&mut self, _ctx: &Ctx, _timeout_ms: u64) -> Result<(), Self::Error> {
        send_connect_unary(&self.transport, PING_PATH, &[])
    }
}

impl<Ctx> BoxEnvironmentControlClient<Ctx> for ProductionBoxControlClient {
    type Error = ProductionBoxTransportError;

    fn update_environment_variables(
        &mut self,
        _ctx: &Ctx,
        request: BoxEnvironmentUpdate,
    ) -> Result<(), Self::Error> {
        let body = encode_update_environment_variables_request(&request);
        send_connect_unary(
            &self.transport,
            UPDATE_ENVIRONMENT_VARIABLES_PATH,
            &body,
        )
    }
}

impl<Ctx> BoxMcpControlClient<Ctx> for ProductionBoxControlClient {
    type Error = ProductionBoxTransportError;

    fn load_mcp_servers(
        &mut self,
        _ctx: &Ctx,
        request: BoxMcpLoadRequest,
    ) -> Result<BoxMcpLoadResponse, Self::Error> {
        let body = encode_load_mcp_servers_request(&request);
        let response = send_connect_unary_response(
            &self.transport,
            LOAD_MCP_SERVERS_PATH,
            &body,
        )?;
        Ok(BoxMcpLoadResponse {
            loaded_server_names: decode_load_mcp_servers_response(&response)?,
        })
    }
}

pub const BOX_GENERATED_PROTOBUF_VERSION: &str = "1.10.1";
pub const BOX_GENERATED_CONNECT_VERSION: &str = "1.6.1";
pub const BOX_GENERATED_CONNECT_NODE_VERSION: &str = "1.6.1";
pub const SHELL_STREAM_FIELD_NUMBER: u32 = 14;
pub const BACKGROUND_SHELL_SPAWN_FIELD_NUMBER: u32 = 16;
pub const SHELL_STREAM_TIMEOUT_BEHAVIOR_BACKGROUND: i32 = 2;
pub const SHELL_BACKGROUND_REASON_TIMEOUT: i32 = 1;
pub const SHELL_BACKGROUND_REASON_USER_REQUEST: i32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionReadArgs {
    pub path: String,
    pub tool_call_id: String,
    pub offset: Option<i32>,
    pub limit: Option<u32>,
    pub encoding_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionBackgroundShellSpawnArgs {
    pub command: String,
    pub working_directory: String,
    pub tool_call_id: String,
    pub skip_approval: bool,
    pub parsing_result: ShellCommandParsingResult,
}

impl From<HostShellArgs> for ProductionBackgroundShellSpawnArgs {
    fn from(args: HostShellArgs) -> Self {
        Self {
            command: args.command,
            working_directory: args.working_directory,
            tool_call_id: args.tool_call_id,
            skip_approval: args.skip_approval,
            parsing_result: args.parsing_result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionBackgroundShellSpawnResult {
    Success { shell_id: u32, command: String, working_directory: String, pid: Option<u32> },
    Error { command: String, working_directory: String, error: String },
    Rejected { command: String, working_directory: String, reason: String, is_readonly: bool },
    PermissionDenied { command: String, working_directory: String, error: String, is_readonly: bool },
    SandboxUnsupported { command: String, working_directory: String, sandbox_policy_type: String, reason: String, is_readonly: bool },
    Other { case: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionShellStreamArgs {
    pub shell_args: HostShellArgs,
    pub timeout_ms: i32,
    pub hard_timeout_ms: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionShellStreamEvent {
    Start,
    Stdout(String),
    Stderr(String),
    Exit {
        code: u32,
        cwd: String,
        aborted: bool,
        local_execution_time_ms: Option<i32>,
    },
    Rejected {
        command: String,
        working_directory: String,
        reason: String,
        is_readonly: bool,
    },
    PermissionDenied {
        command: String,
        working_directory: String,
        error: String,
        is_readonly: bool,
    },
    SandboxUnsupported {
        command: String,
        working_directory: String,
        sandbox_policy_type: String,
        reason: String,
        is_readonly: bool,
    },
    Backgrounded {
        shell_id: u32,
        command: String,
        working_directory: String,
        pid: Option<u32>,
        ms_to_wait: Option<i32>,
        reason: Option<i32>,
    },
    Other {
        case: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionReadOutput {
    Content(String),
    Data(Vec<u8>),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionReadResult {
    Success {
        path: String,
        output: ProductionReadOutput,
        total_lines: i32,
        file_size: i64,
        truncated: bool,
        output_blob_id: Option<Vec<u8>>,
        range_applied: bool,
    },
    Error { path: String, error: String },
    Rejected { path: String, reason: String },
    FileNotFound { path: String },
    PermissionDenied { path: String },
    InvalidFile { path: String, reason: String },
    Other { case: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionExecRequest {
    Shell { id: u64, args: HostShellArgs },
    Write {
        id: u64,
        path: String,
        file_bytes: Vec<u8>,
        tool_call_id: String,
    },
    Read { id: u64, args: ProductionReadArgs },
    ComputerUse { id: u64, protobuf_args: Vec<u8> },
    RawResource {
        id: u64,
        field_number: u32,
        protobuf_args: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionShellResult {
    Success {
        exit_code: i32,
        stdout: String,
        stderr: String,
    },
    Failure {
        exit_code: i32,
        signal: String,
        stdout: String,
        stderr: String,
        aborted: bool,
    },
    SpawnError { error: String },
    PermissionDenied { error: String },
    Rejected { reason: String },
    Timeout { timeout_ms: u64 },
    Other { case: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionExecClientMessage {
    Shell(ProductionShellResult),
    Write(WriteExecResult),
    Read(ProductionReadResult),
    ComputerUse(Vec<u8>),
    RawResource {
        field_number: u32,
        protobuf_result: Vec<u8>,
    },
    Other,
}

#[derive(Debug)]
pub enum ProductionBoxExecError {
    Remote(BoxRemoteExecError<ProductionBoxTransportError>),
    ProtectedPath(SandProtectedPathError),
    NoMonitor(SandBoxNoMonitorAvailableError),
    InvalidResult(ProductionBoxTransportError),
    MissingResult(&'static str),
}

impl ProductionBoxExecError {
    /// Whether a dispatched operation lacks a trustworthy completed result.
    pub fn outcome_unknown(&self) -> bool {
        match self {
            Self::Remote(BoxRemoteExecError::Transport(error)) => {
                matches!(error, ProductionBoxTransportError::ExecutionOutcomeUnknown(_))
            }
            Self::Remote(BoxRemoteExecError::ProtocolViolation(_))
            | Self::Remote(BoxRemoteExecError::RemoteThrow { .. })
            | Self::InvalidResult(_)
            | Self::MissingResult(_) => true,
            Self::Remote(BoxRemoteExecError::IdExhausted)
            | Self::ProtectedPath(_)
            | Self::NoMonitor(_) => false,
        }
    }
}

impl fmt::Display for ProductionBoxExecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Remote(error) => error.fmt(formatter),
            Self::ProtectedPath(error) => error.fmt(formatter),
            Self::NoMonitor(error) => error.fmt(formatter),
            Self::InvalidResult(error) => error.fmt(formatter),
            Self::MissingResult(kind) => {
                write!(formatter, "box ExecService closed without a {kind} result")
            }
        }
    }
}

impl std::error::Error for ProductionBoxExecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Remote(error) => Some(error),
            Self::ProtectedPath(error) => Some(error),
            Self::NoMonitor(error) => Some(error),
            Self::InvalidResult(error) => Some(error),
            Self::MissingResult(_) => None,
        }
    }
}

impl From<BoxRemoteExecError<ProductionBoxTransportError>> for ProductionBoxExecError {
    fn from(error: BoxRemoteExecError<ProductionBoxTransportError>) -> Self {
        Self::Remote(error)
    }
}

impl From<SandProtectedPathError> for ProductionBoxExecError {
    fn from(error: SandProtectedPathError) -> Self {
        Self::ProtectedPath(error)
    }
}

impl From<SandBoxNoMonitorAvailableError> for ProductionBoxExecError {
    fn from(error: SandBoxNoMonitorAvailableError) -> Self {
        Self::NoMonitor(error)
    }
}

impl From<ProductionBoxTransportError> for ProductionBoxExecError {
    fn from(error: ProductionBoxTransportError) -> Self {
        Self::InvalidResult(error)
    }
}

pub struct ProductionBoxExecClient {
    transport: ProductionBoxTransport,
}

pub fn create_production_box_exec_client(
    transport: &ProductionBoxTransport,
) -> ProductionBoxExecClient {
    ProductionBoxExecClient {
        transport: transport.clone(),
    }
}

impl<Ctx> BoxRemoteExecClient<Ctx, ProductionExecRequest, ProductionExecClientMessage>
    for ProductionBoxExecClient
{
    type Error = ProductionBoxTransportError;

    fn exec(
        &mut self,
        _ctx: &Ctx,
        args: ProductionExecRequest,
    ) -> Result<Vec<BoxRemoteExecEnvelope<ProductionExecClientMessage>>, Self::Error> {
        let id = match &args {
            ProductionExecRequest::Shell { id, .. }
            | ProductionExecRequest::Write { id, .. }
            | ProductionExecRequest::Read { id, .. }
            | ProductionExecRequest::ComputerUse { id, .. }
            | ProductionExecRequest::RawResource { id, .. } => *id,
        };
        if id > u64::from(u32::MAX) {
            return Err(invalid_exec_response("request identity exceeds uint32 before dispatch"));
        }
        let request = encode_exec_server_message(&args);
        send_connect_server_stream(&self.transport, EXEC_PATH, &request)
    }
}

pub struct ProductionBoxResourceAccessor {
    manager: BoxRemoteExecManager<ProductionBoxExecClient>,
    protected_read_roots: Vec<PathBuf>,
    no_monitor_computer_use: bool,
}

pub fn create_production_box_resource_accessor(
    transport: &ProductionBoxTransport,
) -> ProductionBoxResourceAccessor {
    ProductionBoxResourceAccessor {
        manager: BoxRemoteExecManager::new(create_production_box_exec_client(transport)),
        protected_read_roots: Vec::new(),
        no_monitor_computer_use: false,
    }
}

impl ProductionBoxResourceAccessor {
    pub fn with_file_read_guard(mut self, protected_read_roots: Vec<PathBuf>) -> Self {
        self.protected_read_roots = protected_read_roots;
        self
    }

    pub fn with_no_monitor_computer_use(mut self) -> Self {
        self.no_monitor_computer_use = true;
        self
    }

    pub fn execute_read<Ctx>(
        &mut self,
        ctx: &Ctx,
        args: ProductionReadArgs,
    ) -> Result<ProductionReadResult, ProductionBoxExecError> {
        assert_path_outside_protected_roots(
            &self.protected_read_roots,
            Path::new(&args.path),
            Path::new("/workspace"),
        )?;
        let messages = self.manager.create_exec_instance(ctx, |id| {
            ProductionExecRequest::Read { id, args }
        })?;
        single_exec_result(messages, "read", |message| match message {
            ProductionExecClientMessage::Read(result) => Some(result),
            _ => None,
        })
    }

    pub fn execute_computer_use_protobuf<Ctx>(
        &mut self,
        ctx: &Ctx,
        protobuf_args: Vec<u8>,
    ) -> Result<Vec<u8>, ProductionBoxExecError> {
        if self.no_monitor_computer_use {
            return Err(SandBoxNoMonitorAvailableError::default().into());
        }
        let messages = self.manager.create_exec_instance(ctx, |id| {
            ProductionExecRequest::ComputerUse { id, protobuf_args }
        })?;
        single_exec_result(messages, "computer-use", |message| match message {
            ProductionExecClientMessage::ComputerUse(result) => Some(result),
            _ => None,
        })
    }

    pub fn execute_raw_resource<Ctx>(
        &mut self,
        ctx: &Ctx,
        field_number: u32,
        protobuf_args: Vec<u8>,
    ) -> Result<Vec<u8>, ProductionBoxExecError> {
        let messages = self.manager.create_exec_instance(ctx, |id| {
            ProductionExecRequest::RawResource {
                id,
                field_number,
                protobuf_args,
            }
        })?;
        single_exec_result(messages, "raw-resource", |message| match message {
            ProductionExecClientMessage::RawResource {
                field_number: returned_field,
                protobuf_result,
            } if returned_field == field_number => Some(protobuf_result),
            _ => None,
        })
    }

    pub fn execute_background_shell_spawn<Ctx>(
        &mut self,
        ctx: &Ctx,
        args: ProductionBackgroundShellSpawnArgs,
    ) -> Result<ProductionBackgroundShellSpawnResult, ProductionBoxExecError> {
        let protobuf_args = encode_background_shell_spawn_args(&args);
        let protobuf_result =
            self.execute_raw_resource(ctx, BACKGROUND_SHELL_SPAWN_FIELD_NUMBER, protobuf_args)?;
        Ok(decode_background_shell_spawn_result(&protobuf_result)?)
    }

    pub fn execute_shell_stream<Ctx>(
        &mut self,
        ctx: &Ctx,
        args: ProductionShellStreamArgs,
    ) -> Result<Vec<ProductionShellStreamEvent>, ProductionBoxExecError> {
        let protobuf_args = encode_shell_stream_args(&args);
        let messages = self.manager.create_exec_instance(ctx, |id| {
            ProductionExecRequest::RawResource {
                id,
                field_number: SHELL_STREAM_FIELD_NUMBER,
                protobuf_args,
            }
        })?;
        let mut events = Vec::new();
        for message in messages {
            match message {
                ProductionExecClientMessage::RawResource {
                    field_number: SHELL_STREAM_FIELD_NUMBER,
                    protobuf_result,
                } => events.push(decode_shell_stream_event(&protobuf_result)?),
                _ => {
                    return Err(invalid_exec_response(
                        "unexpected operation result in shell-stream",
                    ).into());
                }
            }
        }
        if events.is_empty() {
            return Err(ProductionBoxExecError::MissingResult("shell-stream"));
        }
        Ok(events)
    }

    pub fn execute_shell_result<Ctx>(
        &mut self,
        ctx: &Ctx,
        args: HostShellArgs,
    ) -> Result<ProductionShellResult, ProductionBoxExecError> {
        let messages = self.manager.create_exec_instance(ctx, |id| {
            ProductionExecRequest::Shell { id, args }
        })?;
        single_exec_result(messages, "shell", |message| match message {
            ProductionExecClientMessage::Shell(result) => Some(result),
            _ => None,
        })
    }
}

impl<Ctx> ShellAccessor<Ctx> for ProductionBoxResourceAccessor {
    type Error = ProductionBoxExecError;

    fn execute(
        &mut self,
        ctx: &Ctx,
        args: HostShellArgs,
    ) -> Result<ShellExecutionResult, Self::Error> {
        let result = self.execute_shell_result(ctx, args)?;
        Ok(match result {
            ProductionShellResult::Success {
                exit_code, stderr, ..
            } => ShellExecutionResult {
                result: ShellExecutionOutcome::Success { exit_code, stderr },
            },
            ProductionShellResult::Failure {
                exit_code,
                signal,
                stderr,
                aborted,
                ..
            } => ShellExecutionResult {
                result: ShellExecutionOutcome::Failure {
                    case: format!(
                        "failure(exit={exit_code},signal={signal},aborted={aborted},stderr={stderr})"
                    ),
                },
            },
            ProductionShellResult::SpawnError { error } => ShellExecutionResult {
                result: ShellExecutionOutcome::Failure {
                    case: format!("spawn_error({error})"),
                },
            },
            ProductionShellResult::PermissionDenied { error } => ShellExecutionResult {
                result: ShellExecutionOutcome::Failure {
                    case: format!("permission_denied({error})"),
                },
            },
            ProductionShellResult::Rejected { reason } => ShellExecutionResult {
                result: ShellExecutionOutcome::Failure {
                    case: format!("rejected({reason})"),
                },
            },
            ProductionShellResult::Timeout { timeout_ms } => ShellExecutionResult {
                result: ShellExecutionOutcome::Failure {
                    case: format!("timeout({timeout_ms}ms)"),
                },
            },
            ProductionShellResult::Other { case } => ShellExecutionResult {
                result: ShellExecutionOutcome::Failure { case },
            },
        })
    }
}

impl<Ctx> FileTransferAccessor<Ctx> for ProductionBoxResourceAccessor {
    type Error = ProductionBoxExecError;

    fn execute_shell(
        &mut self,
        ctx: &Ctx,
        args: HostShellArgs,
    ) -> Result<ShellExecResult, Self::Error> {
        let result = self.execute_shell_result(ctx, args)?;
        Ok(match result {
            ProductionShellResult::Success { exit_code, .. } => {
                ShellExecResult::Success { exit_code }
            }
            ProductionShellResult::Failure {
                exit_code,
                signal,
                stderr,
                aborted,
                ..
            } => ShellExecResult::Failure {
                exit_code,
                signal,
                stderr,
                aborted,
            },
            ProductionShellResult::SpawnError { error } => ShellExecResult::SpawnError { error },
            ProductionShellResult::PermissionDenied { error } => {
                ShellExecResult::PermissionDenied { error }
            }
            ProductionShellResult::Rejected { reason } => ShellExecResult::Rejected { reason },
            ProductionShellResult::Timeout { timeout_ms } => {
                ShellExecResult::Timeout { timeout_ms }
            }
            ProductionShellResult::Other { case } => ShellExecResult::Other { case },
        })
    }

    fn execute_write(
        &mut self,
        ctx: &Ctx,
        path: &str,
        file_bytes: &[u8],
        tool_call_id: &str,
    ) -> Result<WriteExecResult, Self::Error> {
        let messages = self.manager.create_exec_instance(ctx, |id| {
            ProductionExecRequest::Write {
                id,
                path: path.to_string(),
                file_bytes: file_bytes.to_vec(),
                tool_call_id: tool_call_id.to_string(),
            }
        })?;
        single_exec_result(messages, "write", |message| match message {
            ProductionExecClientMessage::Write(result) => Some(result),
            _ => None,
        })
    }
}

fn single_exec_result<T>(
    messages: Vec<ProductionExecClientMessage>,
    kind: &'static str,
    project: impl FnOnce(ProductionExecClientMessage) -> Option<T>,
) -> Result<T, ProductionBoxExecError> {
    let mut messages = messages.into_iter();
    let message = messages.next().ok_or(ProductionBoxExecError::MissingResult(kind))?;
    if messages.next().is_some() {
        return Err(invalid_exec_response("multiple one-shot operation results").into());
    }
    project(message).ok_or_else(|| {
        invalid_exec_response("unexpected one-shot operation result").into()
    })
}

fn encode_key(field_number: u32, wire_type: u8, out: &mut Vec<u8>) {
    encode_varint(((field_number as usize) << 3) | usize::from(wire_type), out);
}

fn encode_uint32_field(field_number: u32, mut value: u64, out: &mut Vec<u8>) {
    // The manager and client check request IDs before this encoder is reached.
    // Do not silently change the value if this wire helper is used directly.
    encode_key(field_number, 0, out);
    while value >= 0x80 {
        out.push(((value as u8) & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn encode_bool_field(field_number: u32, value: bool, out: &mut Vec<u8>) {
    if value {
        encode_key(field_number, 0, out);
        out.push(1);
    }
}

fn encode_shell_parsing_result(parsing_result: &ShellCommandParsingResult) -> Vec<u8> {
    let mut body = Vec::new();
    if parsing_result.parsing_failed {
        encode_bool_field(1, true, &mut body);
    }
    for executable in &parsing_result.executable_commands {
        let mut command = Vec::new();
        if !executable.name.is_empty() {
            encode_len_delimited(1, executable.name.as_bytes(), &mut command);
        }
        for argument in &executable.args {
            let mut encoded_argument = Vec::new();
            encode_len_delimited(2, argument.as_bytes(), &mut encoded_argument);
            encode_len_delimited(2, &encoded_argument, &mut command);
        }
        if !executable.full_text.is_empty() {
            encode_len_delimited(3, executable.full_text.as_bytes(), &mut command);
        }
        encode_len_delimited(2, &command, &mut body);
    }
    if parsing_result.has_redirects {
        encode_bool_field(3, true, &mut body);
    }
    if parsing_result.has_command_substitution {
        encode_bool_field(4, true, &mut body);
    }
    body
}

fn encode_shell_args(args: &HostShellArgs) -> Vec<u8> {
    let mut body = Vec::new();
    if !args.command.is_empty() {
        encode_len_delimited(1, args.command.as_bytes(), &mut body);
    }
    if !args.working_directory.is_empty() {
        encode_len_delimited(2, args.working_directory.as_bytes(), &mut body);
    }
    if !args.tool_call_id.is_empty() {
        encode_len_delimited(4, args.tool_call_id.as_bytes(), &mut body);
    }
    let parsing = encode_shell_parsing_result(&args.parsing_result);
    if !parsing.is_empty() {
        encode_len_delimited(8, &parsing, &mut body);
    }
    encode_bool_field(12, args.skip_approval, &mut body);
    body
}

pub fn encode_shell_stream_args(args: &ProductionShellStreamArgs) -> Vec<u8> {
    let mut body = Vec::new();
    let shell = &args.shell_args;
    if !shell.command.is_empty() {
        encode_len_delimited(1, shell.command.as_bytes(), &mut body);
    }
    if !shell.working_directory.is_empty() {
        encode_len_delimited(2, shell.working_directory.as_bytes(), &mut body);
    }
    encode_int32_field(3, args.timeout_ms, &mut body);
    if !shell.tool_call_id.is_empty() {
        encode_len_delimited(4, shell.tool_call_id.as_bytes(), &mut body);
    }
    let parsing = encode_shell_parsing_result(&shell.parsing_result);
    if !parsing.is_empty() {
        encode_len_delimited(8, &parsing, &mut body);
    }
    encode_bool_field(12, shell.skip_approval, &mut body);
    encode_int32_field(
        13,
        SHELL_STREAM_TIMEOUT_BEHAVIOR_BACKGROUND,
        &mut body,
    );
    if let Some(hard_timeout_ms) = args.hard_timeout_ms {
        encode_int32_field(14, hard_timeout_ms, &mut body);
    }
    body
}

pub fn encode_background_shell_spawn_args(args: &ProductionBackgroundShellSpawnArgs) -> Vec<u8> {
    let mut body = Vec::new();
    if !args.command.is_empty() {
        encode_len_delimited(1, args.command.as_bytes(), &mut body);
    }
    if !args.working_directory.is_empty() {
        encode_len_delimited(2, args.working_directory.as_bytes(), &mut body);
    }
    if !args.tool_call_id.is_empty() {
        encode_len_delimited(3, args.tool_call_id.as_bytes(), &mut body);
    }
    let parsing = encode_shell_parsing_result(&args.parsing_result);
    if !parsing.is_empty() {
        encode_len_delimited(4, &parsing, &mut body);
    }
    encode_bool_field(12, args.skip_approval, &mut body);
    body
}

fn encode_write_args(path: &str, file_bytes: &[u8], tool_call_id: &str) -> Vec<u8> {
    let mut body = Vec::new();
    if !path.is_empty() {
        encode_len_delimited(1, path.as_bytes(), &mut body);
    }
    if !tool_call_id.is_empty() {
        encode_len_delimited(3, tool_call_id.as_bytes(), &mut body);
    }
    if !file_bytes.is_empty() {
        encode_len_delimited(5, file_bytes, &mut body);
    }
    body
}

fn encode_int32_field(field_number: u32, value: i32, out: &mut Vec<u8>) {
    encode_key(field_number, 0, out);
    let encoded = value as i64 as u64;
    let encoded = usize::try_from(encoded).unwrap_or(usize::MAX);
    encode_varint(encoded, out);
}

fn encode_read_args(args: &ProductionReadArgs) -> Vec<u8> {
    let mut body = Vec::new();
    if !args.path.is_empty() {
        encode_len_delimited(1, args.path.as_bytes(), &mut body);
    }
    if !args.tool_call_id.is_empty() {
        encode_len_delimited(2, args.tool_call_id.as_bytes(), &mut body);
    }
    if let Some(offset) = args.offset {
        encode_int32_field(4, offset, &mut body);
    }
    if let Some(limit) = args.limit {
        encode_uint32_field(5, u64::from(limit), &mut body);
    }
    if let Some(encoding_hint) = args.encoding_hint.as_deref().filter(|value| !value.is_empty()) {
        encode_len_delimited(6, encoding_hint.as_bytes(), &mut body);
    }
    body
}

pub fn encode_exec_server_message(request: &ProductionExecRequest) -> Vec<u8> {
    let mut body = Vec::new();
    match request {
        ProductionExecRequest::Shell { id, args } => {
            encode_uint32_field(1, *id, &mut body);
            let args = encode_shell_args(args);
            encode_len_delimited(2, &args, &mut body);
        }
        ProductionExecRequest::Write {
            id,
            path,
            file_bytes,
            tool_call_id,
        } => {
            encode_uint32_field(1, *id, &mut body);
            let args = encode_write_args(path, file_bytes, tool_call_id);
            encode_len_delimited(3, &args, &mut body);
        }
        ProductionExecRequest::Read { id, args } => {
            encode_uint32_field(1, *id, &mut body);
            let args = encode_read_args(args);
            encode_len_delimited(7, &args, &mut body);
        }
        ProductionExecRequest::ComputerUse { id, protobuf_args } => {
            encode_uint32_field(1, *id, &mut body);
            encode_len_delimited(22, protobuf_args, &mut body);
        }
        ProductionExecRequest::RawResource {
            id,
            field_number,
            protobuf_args,
        } => {
            encode_uint32_field(1, *id, &mut body);
            encode_len_delimited(*field_number, protobuf_args, &mut body);
        }
    }
    body
}

fn invalid_exec_response(message: &str) -> ProductionBoxTransportError {
    ProductionBoxTransportError::InvalidHttpResponse(format!("box ExecService {message}"))
}

fn decode_protobuf_key(
    input: &[u8],
    cursor: &mut usize,
) -> Result<(u64, u8), ProductionBoxTransportError> {
    let key = decode_varint(input, cursor)?;
    let field = key >> 3;
    if field == 0 || field > 0x1fff_ffff {
        return Err(invalid_exec_response("returned an invalid protobuf field number"));
    }
    Ok((field, (key & 0x07) as u8))
}

fn require_wire(actual: u8, expected: u8) -> Result<(), ProductionBoxTransportError> {
    if actual != expected {
        return Err(invalid_exec_response("returned a known field with the wrong wire type"));
    }
    Ok(())
}

fn decode_len_delimited<'a>(
    input: &'a [u8],
    cursor: &mut usize,
) -> Result<&'a [u8], ProductionBoxTransportError> {
    let length = usize::try_from(decode_varint(input, cursor)?)
        .map_err(|_| invalid_exec_response("protobuf length overflow"))?;
    let end = cursor.checked_add(length)
        .filter(|end| *end <= input.len())
        .ok_or_else(|| invalid_exec_response("returned truncated protobuf"))?;
    let value = &input[*cursor..end];
    *cursor = end;
    Ok(value)
}

fn validate_protobuf_message(input: &[u8]) -> Result<(), ProductionBoxTransportError> {
    let mut cursor = 0;
    while cursor < input.len() {
        let (_, wire) = decode_protobuf_key(input, &mut cursor)?;
        skip_protobuf_field(input, &mut cursor, wire)?;
    }
    Ok(())
}

fn decode_varint_field<T>(
    input: &[u8],
    wanted: u64,
    decode: impl Fn(u64) -> Result<T, ProductionBoxTransportError>,
) -> Result<Option<T>, ProductionBoxTransportError> {
    let mut cursor = 0;
    let mut value = None;
    while cursor < input.len() {
        let (field, wire) = decode_protobuf_key(input, &mut cursor)?;
        if field == wanted {
            require_wire(wire, 0)?;
            // Validate each occurrence before retaining the legal final value.
            value = Some(decode(decode_varint(input, &mut cursor)?)?);
        } else {
            skip_protobuf_field(input, &mut cursor, wire)?;
        }
    }
    Ok(value)
}

fn decode_u64_field(
    input: &[u8],
    wanted: u64,
) -> Result<Option<u64>, ProductionBoxTransportError> {
    decode_varint_field(input, wanted, |value| Ok(value))
}

fn decode_u32_field(
    input: &[u8],
    wanted: u64,
) -> Result<Option<u32>, ProductionBoxTransportError> {
    decode_varint_field(input, wanted, |value| {
        u32::try_from(value).map_err(|_| invalid_exec_response("returned uint32 overflow"))
    })
}

fn decode_i32_field(
    input: &[u8],
    wanted: u64,
) -> Result<Option<i32>, ProductionBoxTransportError> {
    Ok(decode_u64_field(input, wanted)?.map(|value| value as u32 as i32))
}

fn decode_string_field(
    input: &[u8],
    wanted: u64,
) -> Result<Option<String>, ProductionBoxTransportError> {
    decode_len_delimited_field(input, wanted, |value| {
        std::str::from_utf8(value)
            .map(|value| value.to_string())
            .map_err(|_| invalid_exec_response("returned non-UTF-8 protobuf string"))
    })
}

fn decode_len_delimited_field<T>(
    input: &[u8],
    wanted: u64,
    decode: impl Fn(&[u8]) -> Result<T, ProductionBoxTransportError>,
) -> Result<Option<T>, ProductionBoxTransportError> {
    let mut cursor = 0;
    let mut value = None;
    while cursor < input.len() {
        let (field, wire) = decode_protobuf_key(input, &mut cursor)?;
        if field == wanted {
            require_wire(wire, 2)?;
            // A later valid duplicate must not conceal an earlier typed error.
            value = Some(decode(decode_len_delimited(input, &mut cursor)?)?);
        } else {
            skip_protobuf_field(input, &mut cursor, wire)?;
        }
    }
    Ok(value)
}

fn decode_bytes_field(
    input: &[u8],
    wanted: u64,
) -> Result<Option<Vec<u8>>, ProductionBoxTransportError> {
    decode_len_delimited_field(input, wanted, |value| Ok(value.to_vec()))
}

fn record_identity<T: PartialEq>(
    previous: &mut Option<T>,
    value: T,
) -> Result<(), ProductionBoxTransportError> {
    if previous.as_ref().is_some_and(|previous| previous != &value) {
        return Err(invalid_exec_response("returned contradictory repeated identity"));
    }
    *previous = Some(value);
    Ok(())
}

fn decode_control_identity(input: &[u8]) -> Result<Option<u64>, ProductionBoxTransportError> {
    let mut cursor = 0;
    let mut id = None;
    while cursor < input.len() {
        let (field, wire) = decode_protobuf_key(input, &mut cursor)?;
        if field == 1 {
            require_wire(wire, 0)?;
            let value = decode_varint(input, &mut cursor)?;
            u32::try_from(value)
                .map_err(|_| invalid_exec_response("returned uint32 identity overflow"))?;
            record_identity(&mut id, value)?;
        } else {
            skip_protobuf_field(input, &mut cursor, wire)?;
        }
    }
    Ok(id)
}

/// Decode one declared message oneof, scanning metadata and trailing fields too.
/// Unknown fields remain opaque protobuf fields, never inferred operation kinds.
fn decode_message_oneof<'a>(
    input: &'a [u8],
    cases: &[u64],
    metadata: &[(u64, u8)],
) -> Result<(u64, &'a [u8]), ProductionBoxTransportError> {
    let mut cursor = 0;
    let mut result = None;
    while cursor < input.len() {
        let (field, wire) = decode_protobuf_key(input, &mut cursor)?;
        if cases.contains(&field) {
            require_wire(wire, 2)?;
            if result.is_some() {
                return Err(invalid_exec_response("returned multiple protobuf oneof payloads"));
            }
            let nested = decode_len_delimited(input, &mut cursor)?;
            validate_protobuf_message(nested)?;
            result = Some((field, nested));
        } else {
            if let Some((_, expected_wire)) = metadata.iter().find(|(tag, _)| *tag == field) {
                require_wire(wire, *expected_wire)?;
            }
            skip_protobuf_field(input, &mut cursor, wire)?;
        }
    }
    result.ok_or_else(|| invalid_exec_response("returned no supported protobuf oneof payload"))
}

fn decode_read_result(
    input: &[u8],
) -> Result<ProductionReadResult, ProductionBoxTransportError> {
    let (field, nested) = decode_message_oneof(input, &[1, 2, 3, 4, 5, 6], &[])?;
    match field {
        1 => {
            let content = decode_string_field(nested, 2)?;
            let data = decode_bytes_field(nested, 5)?;
            let output = match (content, data) {
                (Some(_), Some(_)) => {
                    return Err(invalid_exec_response("returned multiple Read output cases"));
                }
                (Some(content), None) => ProductionReadOutput::Content(content),
                (None, Some(data)) => ProductionReadOutput::Data(data),
                (None, None) => ProductionReadOutput::None,
            };
            Ok(ProductionReadResult::Success {
                path: decode_string_field(nested, 1)?.unwrap_or_default(),
                output,
                total_lines: decode_u64_field(nested, 3)?.unwrap_or_default() as u32 as i32,
                file_size: decode_u64_field(nested, 4)?.unwrap_or_default() as i64,
                truncated: decode_u64_field(nested, 6)?.unwrap_or_default() != 0,
                output_blob_id: decode_bytes_field(nested, 7)?,
                range_applied: decode_u64_field(nested, 8)?.unwrap_or_default() != 0,
            })
        }
        2 => Ok(ProductionReadResult::Error {
            path: decode_string_field(nested, 1)?.unwrap_or_default(),
            error: decode_string_field(nested, 2)?.unwrap_or_default(),
        }),
        3 => Ok(ProductionReadResult::Rejected {
            path: decode_string_field(nested, 1)?.unwrap_or_default(),
            reason: decode_string_field(nested, 2)?.unwrap_or_default(),
        }),
        4 => Ok(ProductionReadResult::FileNotFound {
            path: decode_string_field(nested, 1)?.unwrap_or_default(),
        }),
        5 => Ok(ProductionReadResult::PermissionDenied {
            path: decode_string_field(nested, 1)?.unwrap_or_default(),
        }),
        6 => Ok(ProductionReadResult::InvalidFile {
            path: decode_string_field(nested, 1)?.unwrap_or_default(),
            reason: decode_string_field(nested, 2)?.unwrap_or_default(),
        }),
        _ => unreachable!("declared ReadResult oneof"),
    }
}

fn decode_shell_success(
    input: &[u8],
) -> Result<ProductionShellResult, ProductionBoxTransportError> {
    let exit_code = decode_u64_field(input, 3)?.unwrap_or_default() as u32 as i32;
    let stdout = decode_string_field(input, 5)?.unwrap_or_default();
    let stderr = decode_string_field(input, 6)?.unwrap_or_default();
    Ok(ProductionShellResult::Success {
        exit_code,
        stdout,
        stderr,
    })
}

fn decode_shell_failure(
    input: &[u8],
) -> Result<ProductionShellResult, ProductionBoxTransportError> {
    Ok(ProductionShellResult::Failure {
        exit_code: decode_u64_field(input, 3)?.unwrap_or_default() as u32 as i32,
        signal: decode_string_field(input, 4)?.unwrap_or_default(),
        stdout: decode_string_field(input, 5)?.unwrap_or_default(),
        stderr: decode_string_field(input, 6)?.unwrap_or_default(),
        aborted: decode_u64_field(input, 11)?.unwrap_or_default() != 0,
    })
}

fn decode_shell_result(
    input: &[u8],
) -> Result<ProductionShellResult, ProductionBoxTransportError> {
    // Frozen ShellResult metadata is not part of its operation result oneof.
    let (field, nested) = decode_message_oneof(
        input, &[1, 2, 3, 4, 5, 7], &[(101, 2), (102, 0), (103, 2), (104, 0)],
    )?;
    let _ = decode_len_delimited_field(input, 101, validate_protobuf_message)?;
    let _ = decode_string_field(input, 103)?;
    let _ = decode_u32_field(input, 104)?;
    match field {
        1 => decode_shell_success(nested),
        2 => decode_shell_failure(nested),
        3 => Ok(ProductionShellResult::Timeout {
            timeout_ms: decode_u64_field(nested, 3)?.unwrap_or_default(),
        }),
        4 => Ok(ProductionShellResult::Rejected {
            reason: decode_string_field(nested, 3)?.unwrap_or_default(),
        }),
        5 => Ok(ProductionShellResult::SpawnError {
            error: decode_string_field(nested, 3)?.unwrap_or_default(),
        }),
        7 => Ok(ProductionShellResult::PermissionDenied {
            error: decode_string_field(nested, 3)?.unwrap_or_default(),
        }),
        _ => unreachable!("declared ShellResult oneof"),
    }
}

pub fn decode_background_shell_spawn_result(
    input: &[u8],
) -> Result<ProductionBackgroundShellSpawnResult, ProductionBoxTransportError> {
    let (field, nested) = decode_message_oneof(input, &[1, 2, 3, 4, 5], &[])?;
    if field == 1 {
        return Ok(ProductionBackgroundShellSpawnResult::Success {
            shell_id: decode_u32_field(nested, 1)?.unwrap_or_default(),
            command: decode_string_field(nested, 2)?.unwrap_or_default(),
            working_directory: decode_string_field(nested, 3)?.unwrap_or_default(),
            pid: decode_u32_field(nested, 4)?,
        });
    }
    let command = decode_string_field(nested, 1)?.unwrap_or_default();
    let working_directory = decode_string_field(nested, 2)?.unwrap_or_default();
    match field {
        2 => Ok(ProductionBackgroundShellSpawnResult::Error {
            command,
            working_directory,
            error: decode_string_field(nested, 3)?.unwrap_or_default(),
        }),
        3 => Ok(ProductionBackgroundShellSpawnResult::Rejected {
            command,
            working_directory,
            reason: decode_string_field(nested, 3)?.unwrap_or_default(),
            is_readonly: decode_u64_field(nested, 4)?.unwrap_or_default() != 0,
        }),
        4 => Ok(ProductionBackgroundShellSpawnResult::PermissionDenied {
            command,
            working_directory,
            error: decode_string_field(nested, 3)?.unwrap_or_default(),
            is_readonly: decode_u64_field(nested, 4)?.unwrap_or_default() != 0,
        }),
        5 => Ok(ProductionBackgroundShellSpawnResult::SandboxUnsupported {
            command,
            working_directory,
            sandbox_policy_type: decode_string_field(nested, 3)?.unwrap_or_default(),
            reason: decode_string_field(nested, 4)?.unwrap_or_default(),
            is_readonly: decode_u64_field(nested, 5)?.unwrap_or_default() != 0,
        }),
        _ => unreachable!("declared BackgroundShellSpawnResult oneof"),
    }
}

pub fn decode_shell_stream_event(
    input: &[u8],
) -> Result<ProductionShellStreamEvent, ProductionBoxTransportError> {
    let (field, nested) = decode_message_oneof(input, &[1, 2, 3, 4, 5, 6, 7, 8, 9], &[])?;
    match field {
        1 => Ok(ProductionShellStreamEvent::Stdout(
            decode_string_field(nested, 1)?.unwrap_or_default(),
        )),
        2 => Ok(ProductionShellStreamEvent::Stderr(
            decode_string_field(nested, 1)?.unwrap_or_default(),
        )),
        3 => Ok(ProductionShellStreamEvent::Exit {
            code: decode_u32_field(nested, 1)?.unwrap_or_default(),
            cwd: decode_string_field(nested, 2)?.unwrap_or_default(),
            aborted: decode_u64_field(nested, 4)?.unwrap_or_default() != 0,
            local_execution_time_ms: decode_i32_field(nested, 6)?,
        }),
        4 => Ok(ProductionShellStreamEvent::Start),
        5 => Ok(ProductionShellStreamEvent::Rejected {
            command: decode_string_field(nested, 1)?.unwrap_or_default(),
            working_directory: decode_string_field(nested, 2)?.unwrap_or_default(),
            reason: decode_string_field(nested, 3)?.unwrap_or_default(),
            is_readonly: decode_u64_field(nested, 4)?.unwrap_or_default() != 0,
        }),
        6 => Ok(ProductionShellStreamEvent::PermissionDenied {
            command: decode_string_field(nested, 1)?.unwrap_or_default(),
            working_directory: decode_string_field(nested, 2)?.unwrap_or_default(),
            error: decode_string_field(nested, 3)?.unwrap_or_default(),
            is_readonly: decode_u64_field(nested, 4)?.unwrap_or_default() != 0,
        }),
        7 => Ok(ProductionShellStreamEvent::Backgrounded {
            shell_id: decode_u32_field(nested, 1)?.unwrap_or_default(),
            command: decode_string_field(nested, 2)?.unwrap_or_default(),
            working_directory: decode_string_field(nested, 3)?.unwrap_or_default(),
            pid: decode_u32_field(nested, 4)?,
            ms_to_wait: decode_i32_field(nested, 5)?,
            reason: decode_i32_field(nested, 6)?,
        }),
        9 => Ok(ProductionShellStreamEvent::SandboxUnsupported {
            command: decode_string_field(nested, 1)?.unwrap_or_default(),
            working_directory: decode_string_field(nested, 2)?.unwrap_or_default(),
            sandbox_policy_type: decode_string_field(nested, 3)?.unwrap_or_default(),
            reason: decode_string_field(nested, 4)?.unwrap_or_default(),
            is_readonly: decode_u64_field(nested, 5)?.unwrap_or_default() != 0,
        }),
        _ => Ok(ProductionShellStreamEvent::Other {
            case: format!("shell_stream_event_field_{field}"),
        }),
    }
}

fn decode_write_result(
    input: &[u8],
) -> Result<WriteExecResult, ProductionBoxTransportError> {
    let (field, nested) = decode_message_oneof(input, &[1, 3, 4, 5, 6], &[])?;
    match field {
        1 => Ok(WriteExecResult::Success),
        5 => Ok(WriteExecResult::Error {
            error: decode_string_field(nested, 2)?.unwrap_or_default(),
        }),
        6 => Ok(WriteExecResult::Rejected {
            reason: decode_string_field(nested, 2)?.unwrap_or_default(),
        }),
        3 => Ok(WriteExecResult::Other { case: "permission_denied".into() }),
        4 => Ok(WriteExecResult::Other { case: "no_space".into() }),
        _ => unreachable!("declared WriteResult oneof"),
    }
}

fn is_exec_result_field(field: u64) -> bool {
    // agent/v1/exec_pb.ts, blob b3d569d1ad5f923444d472a08efb8bad58c5941f.
    // In particular 15, 39 and 45 are metadata, not raw resource results.
    matches!(field,
        2 | 3 | 4 | 5 | 7..=11 | 14 | 16..=18 | 20..=23 | 27..=31
        | 36..=38 | 40..=44 | 46..=56
    )
}

fn decode_exec_client_message(
    input: &[u8],
) -> Result<BoxRemoteExecEnvelope<ProductionExecClientMessage>, ProductionBoxTransportError> {
    let mut cursor = 0;
    let mut id = None;
    let mut exec_id = None;
    let mut message = None;
    while cursor < input.len() {
        let (field, wire) = decode_protobuf_key(input, &mut cursor)?;
        match field {
            1 => {
                require_wire(wire, 0)?;
                let value = decode_varint(input, &mut cursor)?;
                u32::try_from(value)
                    .map_err(|_| invalid_exec_response("returned uint32 identity overflow"))?;
                record_identity(&mut id, value)?;
            }
            15 => {
                require_wire(wire, 2)?;
                let value = decode_len_delimited(input, &mut cursor)?;
                let value = std::str::from_utf8(value)
                    .map_err(|_| invalid_exec_response("returned non-UTF-8 execution identity"))?;
                record_identity(&mut exec_id, value.to_string())?;
            }
            39 => {
                require_wire(wire, 0)?;
                let _ = decode_varint(input, &mut cursor)?;
            }
            45 => {
                require_wire(wire, 2)?;
                validate_protobuf_message(decode_len_delimited(input, &mut cursor)?)?;
            }
            field if is_exec_result_field(field) => {
                require_wire(wire, 2)?;
                if message.is_some() {
                    return Err(invalid_exec_response("returned multiple Exec result payloads"));
                }
                let nested = decode_len_delimited(input, &mut cursor)?;
                validate_protobuf_message(nested)?;
                message = Some(match field {
                    2 => ProductionExecClientMessage::Shell(decode_shell_result(nested)?),
                    3 => ProductionExecClientMessage::Write(decode_write_result(nested)?),
                    7 => ProductionExecClientMessage::Read(decode_read_result(nested)?),
                    22 => ProductionExecClientMessage::ComputerUse(nested.to_vec()),
                    _ => ProductionExecClientMessage::RawResource {
                        field_number: field as u32,
                        protobuf_result: nested.to_vec(),
                    },
                });
            }
            _ => skip_protobuf_field(input, &mut cursor, wire)?,
        }
    }
    let message = message
        .ok_or_else(|| invalid_exec_response("returned no supported Exec result payload"))?;
    Ok(BoxRemoteExecEnvelope::ExecClientMessage {
        id: id.unwrap_or_default(),
        exec_id: exec_id.unwrap_or_default(),
        message,
    })
}

fn decode_exec_control_message(
    input: &[u8],
) -> Result<BoxRemoteExecControlMessage, ProductionBoxTransportError> {
    let (field, nested) = decode_message_oneof(input, &[1, 2, 3], &[])?;
    let id = decode_control_identity(nested)?;
    match field {
        1 => Ok(BoxRemoteExecControlMessage::StreamClose { id: id.unwrap_or_default() }),
        2 => Ok(BoxRemoteExecControlMessage::Throw {
            id,
            error: decode_string_field(nested, 2)?
                .unwrap_or_else(|| "remote ExecService error".into()),
            stack_trace: decode_string_field(nested, 3)?,
            error_code: decode_string_field(nested, 4)?,
        }),
        3 => Ok(BoxRemoteExecControlMessage::Heartbeat { id: id.unwrap_or_default() }),
        _ => unreachable!("declared Exec control oneof"),
    }
}

fn decode_exec_stream_element(
    input: &[u8],
) -> Result<BoxRemoteExecEnvelope<ProductionExecClientMessage>, ProductionBoxTransportError> {
    let (field, nested) = decode_message_oneof(input, &[1, 2], &[])?;
    match field {
        1 => decode_exec_client_message(nested),
        2 => Ok(BoxRemoteExecEnvelope::ExecClientControlMessage(
            decode_exec_control_message(nested)?,
        )),
        _ => unreachable!("declared Exec stream oneof"),
    }
}

fn encode_connect_envelope(
    flags: u8,
    payload: &[u8],
) -> Result<Vec<u8>, ProductionBoxTransportError> {
    let length = u32::try_from(payload.len())
        .map_err(|_| invalid_exec_response("request exceeds Connect frame length"))?;
    let capacity = payload.len().checked_add(5)
        .ok_or_else(|| invalid_exec_response("request length overflow"))?;
    // Match the existing daemon's authenticated Exec request body limit.
    if capacity > 16 * 1024 * 1024 {
        return Err(invalid_exec_response("request exceeds the 16 MiB body limit"));
    }
    let mut envelope = Vec::with_capacity(capacity);
    envelope.push(flags);
    envelope.extend_from_slice(&length.to_be_bytes());
    envelope.extend_from_slice(payload);
    Ok(envelope)
}

#[derive(serde::Deserialize)]
struct ConnectEndEnvelope {
    // Keep explicit null distinguishable from omission. Deriving the containing
    // object also rejects duplicate error/metadata keys instead of last-wins.
    #[serde(default, deserialize_with = "deserialize_present_json")]
    error: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_present_json")]
    metadata: Option<serde_json::Value>,
}

fn deserialize_present_json<'de, D>(
    deserializer: D,
) -> Result<Option<serde_json::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    <serde_json::Value as serde::Deserialize>::deserialize(deserializer).map(Some)
}

fn decode_connect_end(
    payload: &[u8],
) -> Result<Option<ProductionBoxTransportError>, ProductionBoxTransportError> {
    if payload.iter().find(|byte| !byte.is_ascii_whitespace()) != Some(&b'{') {
        return Err(invalid_exec_response("returned a malformed Connect end object"));
    }
    let end: ConnectEndEnvelope = serde_json::from_slice(payload)
        .map_err(|_| invalid_exec_response("returned a malformed Connect end object"))?;
    if let Some(metadata) = end.metadata {
        let metadata = metadata.as_object()
            .ok_or_else(|| invalid_exec_response("returned malformed Connect end metadata"))?;
        for values in metadata.values() {
            if !values.as_array().is_some_and(|values| values.iter().all(|value| value.is_string())) {
                return Err(invalid_exec_response("returned malformed Connect end metadata"));
            }
        }
    }
    if let Some(error) = end.error {
        let object = error.as_object()
            .ok_or_else(|| invalid_exec_response("returned a malformed Connect error"))?;
        if !object.get("code").and_then(|code| code.as_str()).is_some_and(|code| !code.is_empty())
            || object.get("message").is_some_and(|message| !message.is_string())
            || object.get("details").is_some_and(|details| !details.is_array())
        {
            return Err(invalid_exec_response("returned a malformed Connect error"));
        }
        return Ok(Some(ProductionBoxTransportError::InvalidHttpResponse(format!(
            "box ExecService returned Connect error: {error}"
        ))));
    }
    Ok(None)
}

fn decode_connect_stream(
    body: &[u8],
) -> Result<Vec<BoxRemoteExecEnvelope<ProductionExecClientMessage>>, ProductionBoxTransportError> {
    let mut cursor = 0usize;
    let mut messages = Vec::new();
    let mut ended = false;
    let mut remote_error = None;
    while cursor < body.len() {
        if ended {
            return Err(invalid_exec_response("returned data after the Connect end envelope"));
        }
        let header_end = cursor.checked_add(5)
            .filter(|end| *end <= body.len())
            .ok_or_else(|| invalid_exec_response("returned a truncated Connect envelope"))?;
        let flags = body[cursor];
        if flags != 0 && flags != CONNECT_STREAM_END_FLAG {
            return Err(invalid_exec_response("returned unsupported Connect envelope flags"));
        }
        let length = u32::from_be_bytes([
            body[cursor + 1], body[cursor + 2], body[cursor + 3], body[cursor + 4],
        ]) as usize;
        cursor = header_end;
        let end = cursor.checked_add(length)
            .filter(|end| *end <= body.len())
            .ok_or_else(|| invalid_exec_response("returned a truncated Connect payload"))?;
        let payload = &body[cursor..end];
        cursor = end;
        if flags == CONNECT_STREAM_END_FLAG {
            remote_error = decode_connect_end(payload)?;
            ended = true;
        } else {
            messages.push(decode_exec_stream_element(payload)?);
        }
    }
    if !ended {
        return Err(invalid_exec_response("ended without a Connect end envelope"));
    }
    if let Some(error) = remote_error {
        return Err(error);
    }
    Ok(messages)
}

fn is_http_token(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(byte, b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+'
            | b'-' | b'.' | b'^' | b'_' | 0x60 | b'|' | b'~')
}

fn parse_http_header_line(
    line: &[u8],
) -> Result<(&[u8], &[u8]), ProductionBoxTransportError> {
    let colon = line.iter().position(|byte| *byte == b':')
        .ok_or_else(|| invalid_exec_response("returned a malformed HTTP header"))?;
    let name = &line[..colon];
    let mut value = &line[colon + 1..];
    if name.is_empty() || !name.iter().all(|byte| is_http_token(*byte))
        || value.iter().any(|byte| (*byte < b' ' && *byte != b'\t') || *byte == 0x7f)
    {
        return Err(invalid_exec_response("returned a malformed HTTP header"));
    }
    // Ordinary field values are octets, including obs-text. Only framing
    // consumers below interpret their values; trim HTTP OWS without UTF-8.
    while value.first().is_some_and(|byte| matches!(*byte, b' ' | b'\t')) {
        value = &value[1..];
    }
    while value.last().is_some_and(|byte| matches!(*byte, b' ' | b'\t')) {
        value = &value[..value.len() - 1];
    }
    Ok((name, value))
}

fn read_http_line<'a>(
    input: &'a [u8],
    cursor: &mut usize,
) -> Result<&'a [u8], ProductionBoxTransportError> {
    let remainder = input.get(*cursor..)
        .ok_or_else(|| invalid_exec_response("HTTP framing cursor overflow"))?;
    let offset = remainder.windows(2).position(|window| window == b"\r\n")
        .ok_or_else(|| invalid_exec_response("returned incomplete chunk or trailer framing"))?;
    let end = cursor.checked_add(offset)
        .ok_or_else(|| invalid_exec_response("HTTP line length overflow"))?;
    let line = &input[*cursor..end];
    *cursor = end.checked_add(2)
        .ok_or_else(|| invalid_exec_response("HTTP line length overflow"))?;
    Ok(line)
}

fn validate_chunk_extensions(input: &[u8]) -> Result<(), ProductionBoxTransportError> {
    let mut cursor = 0;
    while cursor < input.len() {
        while input.get(cursor).is_some_and(|byte| matches!(*byte, b' ' | b'\t')) {
            cursor += 1;
        }
        if input.get(cursor) != Some(&b';') {
            return Err(invalid_exec_response("returned malformed chunk extensions"));
        }
        cursor += 1;
        while input.get(cursor).is_some_and(|byte| matches!(*byte, b' ' | b'\t')) {
            cursor += 1;
        }
        let start = cursor;
        while input.get(cursor).is_some_and(|byte| is_http_token(*byte)) {
            cursor += 1;
        }
        if cursor == start {
            return Err(invalid_exec_response("returned malformed chunk extensions"));
        }
        while input.get(cursor).is_some_and(|byte| matches!(*byte, b' ' | b'\t')) {
            cursor += 1;
        }
        if input.get(cursor) == Some(&b'=') {
            cursor += 1;
            while input.get(cursor).is_some_and(|byte| matches!(*byte, b' ' | b'\t')) {
                cursor += 1;
            }
            if input.get(cursor) == Some(&b'"') {
                cursor += 1;
                loop {
                    let byte = *input.get(cursor)
                        .ok_or_else(|| invalid_exec_response("returned unterminated chunk extension"))?;
                    cursor += 1;
                    if byte == b'"' {
                        break;
                    }
                    if byte == b'\\' {
                        let escaped = *input.get(cursor)
                            .ok_or_else(|| invalid_exec_response("returned truncated chunk extension"))?;
                        if (escaped < b' ' && escaped != b'\t') || escaped == 0x7f {
                            return Err(invalid_exec_response("returned malformed chunk extension"));
                        }
                        cursor += 1;
                    } else if (byte < b' ' && byte != b'\t') || byte == 0x7f {
                        return Err(invalid_exec_response("returned malformed chunk extension"));
                    }
                }
            } else {
                let start = cursor;
                while input.get(cursor).is_some_and(|byte| is_http_token(*byte)) {
                    cursor += 1;
                }
                if cursor == start {
                    return Err(invalid_exec_response("returned malformed chunk extension value"));
                }
            }
            while input.get(cursor).is_some_and(|byte| matches!(*byte, b' ' | b'\t')) {
                cursor += 1;
            }
        }
        if cursor < input.len() && input[cursor] != b';' {
            return Err(invalid_exec_response("returned malformed chunk extensions"));
        }
    }
    Ok(())
}

fn decode_chunked_http_body(input: &[u8]) -> Result<Vec<u8>, ProductionBoxTransportError> {
    let mut cursor = 0usize;
    let mut body = Vec::new();
    loop {
        let line = read_http_line(input, &mut cursor)?;
        let size_end = line.iter().position(|byte| !byte.is_ascii_hexdigit()).unwrap_or(line.len());
        if size_end == 0 {
            return Err(invalid_exec_response("returned an invalid HTTP chunk size"));
        }
        validate_chunk_extensions(&line[size_end..])?;
        let size_text = std::str::from_utf8(&line[..size_end])
            .map_err(|_| invalid_exec_response("returned an invalid HTTP chunk size"))?;
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| invalid_exec_response("returned overflowing HTTP chunk size"))?;
        if size == 0 {
            loop {
                let trailer = read_http_line(input, &mut cursor)?;
                if trailer.is_empty() {
                    if cursor != input.len() {
                        return Err(invalid_exec_response("returned trailing bytes after HTTP trailers"));
                    }
                    return Ok(body);
                }
                let colon = trailer.iter().position(|byte| *byte == b':')
                    .ok_or_else(|| invalid_exec_response("returned a malformed HTTP trailer"))?;
                let (name, value) = trailer.split_at(colon);
                if name.is_empty() || !name.iter().all(|byte| is_http_token(*byte))
                    || value.iter().any(|byte| (*byte < b' ' && *byte != b'\t') || *byte == 0x7f)
                {
                    return Err(invalid_exec_response("returned a malformed HTTP trailer"));
                }
                // Header names are ASCII tokens. Trailer values may contain
                // HTTP obs-text bytes; no UTF-8 interpretation is required.
                let name = std::str::from_utf8(name)
                    .map_err(|_| invalid_exec_response("returned a malformed HTTP trailer name"))?;
                if name.eq_ignore_ascii_case("content-length")
                    || name.eq_ignore_ascii_case("transfer-encoding")
                {
                    return Err(invalid_exec_response("returned framing fields in HTTP trailers"));
                }
            }
        }
        let end = cursor.checked_add(size)
            .ok_or_else(|| invalid_exec_response("HTTP chunk length overflow"))?;
        let framed_end = end.checked_add(2)
            .filter(|end| *end <= input.len())
            .ok_or_else(|| invalid_exec_response("returned a truncated HTTP chunk"))?;
        if &input[end..framed_end] != b"\r\n" {
            return Err(invalid_exec_response("returned a malformed HTTP chunk terminator"));
        }
        body.extend_from_slice(&input[cursor..end]);
        cursor = framed_end;
    }
}

fn decode_exec_http_response(
    response: &[u8],
) -> Result<Vec<BoxRemoteExecEnvelope<ProductionExecClientMessage>>, ProductionBoxTransportError> {
    let header_end = response.windows(4).position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| invalid_exec_response("returned incomplete HTTP headers"))?;
    let body_start = header_end.checked_add(4)
        .ok_or_else(|| invalid_exec_response("HTTP header length overflow"))?;
    let headers = &response[..body_start];
    let mut cursor = 0;
    let status_line = read_http_line(headers, &mut cursor)?;
    let mut status_parts = status_line.splitn(3, |byte| *byte == b' ');
    let version = status_parts.next().unwrap_or_default();
    let status_text = status_parts.next().unwrap_or_default();
    // The second SP is mandatory even when the optional reason is empty.
    let reason = status_parts.next()
        .ok_or_else(|| invalid_exec_response("returned an invalid HTTP status line"))?;
    if (version != b"HTTP/1.1" && version != b"HTTP/1.0")
        || status_text.len() != 3 || !status_text.iter().all(|byte| byte.is_ascii_digit())
        || reason.iter().any(|byte| (*byte < b' ' && *byte != b'\t') || *byte == 0x7f)
    {
        return Err(invalid_exec_response("returned an invalid HTTP status line"));
    }
    let status = status_text.iter().fold(0u16, |value, byte| {
        value * 10 + u16::from(*byte - b'0')
    });
    let mut content_length = None;
    let mut chunked = false;
    loop {
        let line = read_http_line(headers, &mut cursor)?;
        if line.is_empty() {
            break;
        }
        let (name, value) = parse_http_header_line(line)?;
        if name.eq_ignore_ascii_case(b"content-length") {
            // HTTP allows a repeated identical length. Conflicting values and
            // transfer-encoding plus length are rejected before body decoding.
            let value = std::str::from_utf8(value)
                .map_err(|_| invalid_exec_response("returned an invalid Content-Length"))?;
            for value in value.split(',') {
                let value = value.trim_matches(|character| character == ' ' || character == '\t');
                if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(invalid_exec_response("returned an invalid Content-Length"));
                }
                let length = value.parse::<usize>()
                    .map_err(|_| invalid_exec_response("returned overflowing Content-Length"))?;
                if content_length.is_some_and(|previous| previous != length) {
                    return Err(invalid_exec_response("returned conflicting Content-Length fields"));
                }
                content_length = Some(length);
            }
        } else if name.eq_ignore_ascii_case(b"transfer-encoding") {
            if version == b"HTTP/1.0" {
                return Err(invalid_exec_response("returned Transfer-Encoding in an HTTP/1.0 response"));
            }
            if chunked || !value.eq_ignore_ascii_case(b"chunked") {
                return Err(invalid_exec_response("returned unsupported or ambiguous Transfer-Encoding"));
            }
            chunked = true;
        }
    }
    if chunked && content_length.is_some() {
        return Err(invalid_exec_response("returned both Transfer-Encoding and Content-Length"));
    }
    let raw_body = &response[body_start..];
    if content_length.is_some_and(|length| length != raw_body.len()) {
        return Err(invalid_exec_response("returned a Content-Length body mismatch"));
    }
    let response_body = if chunked {
        decode_chunked_http_body(raw_body)?
    } else {
        raw_body.to_vec()
    };
    if status != 200 {
        let body = String::from_utf8_lossy(&response_body).chars().take(512).collect();
        return Err(ProductionBoxTransportError::HttpStatus { status, body });
    }
    decode_connect_stream(&response_body)
}

fn send_connect_server_stream(
    transport: &ProductionBoxTransport,
    path: &str,
    protobuf_body: &[u8],
) -> Result<Vec<BoxRemoteExecEnvelope<ProductionExecClientMessage>>, ProductionBoxTransportError> {
    let base = Url::parse(&transport.options.base_url)
        .map_err(|error| ProductionBoxTransportError::InvalidBaseUrl(error.to_string()))?;
    if base.scheme() != "http" {
        return Err(ProductionBoxTransportError::InvalidBaseUrl(
            "box transport requires an http:// endpoint".into(),
        ));
    }
    let host = base.host_str().ok_or_else(|| {
        ProductionBoxTransportError::InvalidBaseUrl("box transport base URL has no host".into())
    })?;
    let port = base.port_or_known_default().ok_or_else(|| {
        ProductionBoxTransportError::InvalidBaseUrl("box transport base URL has no port".into())
    })?;
    let body = encode_connect_envelope(0, protobuf_body)?;
    let mut request = Vec::new();
    write!(
        request,
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: {CONNECT_STREAM_CONTENT_TYPE}\r\nAccept: {CONNECT_STREAM_CONTENT_TYPE}\r\nConnect-Protocol-Version: {CONNECT_PROTOCOL_VERSION}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    ).map_err(ProductionBoxTransportError::Io)?;
    for (name, value) in transport.request_headers()? {
        write!(request, "{name}: {value}\r\n").map_err(ProductionBoxTransportError::Io)?;
    }
    request.extend_from_slice(b"\r\n");
    request.extend_from_slice(&body);

    let timeout = Duration::from_millis(PRODUCTION_BOX_EXEC_TIMEOUT_MS);
    let mut stream = TcpStream::connect((host, port))?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    // Even an error from the first write cannot prove that zero request bytes
    // reached the daemon. Keep every subsequent error in the unknown class.
    let dispatched = (|| {
        stream.write_all(&request)?;
        stream.flush()?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
        decode_exec_http_response(&response)
    })();
    dispatched.map_err(ProductionBoxTransportError::execution_outcome_unknown)
}

fn validate_header(name: &str, value: &str) -> Result<(), ProductionBoxTransportError> {
    if name.is_empty()
        || name.bytes().any(|byte| byte <= b' ' || byte == b':' || byte >= 0x7f)
        || value.contains('\r')
        || value.contains('\n')
    {
        return Err(ProductionBoxTransportError::InvalidHeader(format!(
            "invalid box transport header {name:?}"
        )));
    }
    Ok(())
}

fn encode_varint(mut value: usize, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push(((value as u8) & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn encode_len_delimited(field_number: u32, value: &[u8], out: &mut Vec<u8>) {
    encode_key(field_number, 2, out);
    encode_varint(value.len(), out);
    out.extend_from_slice(value);
}

fn encode_string_map_entry(key: &str, value: &str) -> Vec<u8> {
    let mut entry = Vec::new();
    encode_len_delimited(1, key.as_bytes(), &mut entry);
    encode_len_delimited(2, value.as_bytes(), &mut entry);
    entry
}

pub fn encode_update_environment_variables_request(update: &BoxEnvironmentUpdate) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in &update.env {
        let entry = encode_string_map_entry(name, value);
        encode_len_delimited(1, &entry, &mut body);
    }
    if update.replace {
        body.extend_from_slice(&[0x10, 0x01]);
    }
    body
}

pub fn encode_load_mcp_servers_request(request: &BoxMcpLoadRequest) -> Vec<u8> {
    let mut body = Vec::new();
    if !request.mcp_config_json.is_empty() {
        encode_len_delimited(1, request.mcp_config_json.as_bytes(), &mut body);
    }
    if request.remove_missing {
        body.extend_from_slice(&[0x10, 0x01]);
    }
    body
}

fn decode_varint(input: &[u8], cursor: &mut usize) -> Result<u64, ProductionBoxTransportError> {
    let mut value = 0u64;
    for shift in (0..=63).step_by(7) {
        let byte = *input.get(*cursor)
            .ok_or_else(|| invalid_exec_response("returned truncated protobuf varint"))?;
        *cursor += 1;
        if shift == 63 && byte > 1 {
            return Err(invalid_exec_response("returned overflowing protobuf varint"));
        }
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(invalid_exec_response("returned malformed protobuf varint"))
}

fn skip_protobuf_field(
    input: &[u8],
    cursor: &mut usize,
    wire_type: u8,
) -> Result<(), ProductionBoxTransportError> {
    let length = match wire_type {
        0 => {
            let _ = decode_varint(input, cursor)?;
            return Ok(());
        }
        1 => 8,
        2 => {
            let _ = decode_len_delimited(input, cursor)?;
            return Ok(());
        }
        5 => 4,
        _ => return Err(invalid_exec_response("returned unsupported protobuf wire type")),
    };
    *cursor = cursor.checked_add(length)
        .filter(|end| *end <= input.len())
        .ok_or_else(|| invalid_exec_response("returned truncated protobuf field"))?;
    Ok(())
}

pub fn decode_load_mcp_servers_response(
    input: &[u8],
) -> Result<Vec<String>, ProductionBoxTransportError> {
    let mut cursor = 0usize;
    let mut names = Vec::new();
    while cursor < input.len() {
        let key = decode_varint(input, &mut cursor)?;
        let field_number = key >> 3;
        let wire_type = (key & 0x07) as u8;
        if field_number == 1 && wire_type == 2 {
            let length = usize::try_from(decode_varint(input, &mut cursor)?).map_err(|_| {
                ProductionBoxTransportError::InvalidHttpResponse(
                    "box ControlService protobuf length overflow".into(),
                )
            })?;
            let end = cursor.saturating_add(length);
            if end > input.len() {
                return Err(ProductionBoxTransportError::InvalidHttpResponse(
                    "box ControlService returned truncated MCP server name".into(),
                ));
            }
            let name = std::str::from_utf8(&input[cursor..end]).map_err(|_| {
                ProductionBoxTransportError::InvalidHttpResponse(
                    "box ControlService returned non-UTF-8 MCP server name".into(),
                )
            })?;
            names.push(name.to_string());
            cursor = end;
            continue;
        }
        skip_protobuf_field(input, &mut cursor, wire_type)?;
    }
    Ok(names)
}

fn send_connect_unary(
    transport: &ProductionBoxTransport,
    path: &str,
    body: &[u8],
) -> Result<(), ProductionBoxTransportError> {
    send_connect_unary_response(transport, path, body).map(|_| ())
}

fn send_connect_unary_response(
    transport: &ProductionBoxTransport,
    path: &str,
    body: &[u8],
) -> Result<Vec<u8>, ProductionBoxTransportError> {
    let base = Url::parse(&transport.options.base_url)
        .map_err(|error| ProductionBoxTransportError::InvalidBaseUrl(error.to_string()))?;
    if base.scheme() != "http" {
        return Err(ProductionBoxTransportError::InvalidBaseUrl(format!(
            "box transport requires http:// loopback, got {}",
            transport.options.base_url
        )));
    }
    let host = base
        .host_str()
        .ok_or_else(|| ProductionBoxTransportError::InvalidBaseUrl(
            "box transport base URL has no host".into(),
        ))?;
    let port = base
        .port_or_known_default()
        .ok_or_else(|| ProductionBoxTransportError::InvalidBaseUrl(
            "box transport base URL has no port".into(),
        ))?;

    let timeout = Duration::from_millis(PRODUCTION_BOX_RPC_TIMEOUT_MS);
    let mut stream = TcpStream::connect((host, port))?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    let mut request = Vec::new();
    write!(
        request,
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/proto\r\nConnect-Protocol-Version: {CONNECT_PROTOCOL_VERSION}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )
    .map_err(ProductionBoxTransportError::Io)?;
    for (name, value) in transport.request_headers()? {
        write!(request, "{name}: {value}\r\n").map_err(ProductionBoxTransportError::Io)?;
    }
    request.extend_from_slice(b"\r\n");
    request.extend_from_slice(body);
    stream.write_all(&request)?;
    stream.flush()?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| ProductionBoxTransportError::InvalidHttpResponse(
            "box ControlService returned an incomplete HTTP response".into(),
        ))?;
    let header_text = std::str::from_utf8(&response[..header_end])
        .map_err(|_| ProductionBoxTransportError::InvalidHttpResponse(
            "box ControlService returned non-UTF-8 HTTP headers".into(),
        ))?;
    let status_line = header_text.lines().next().unwrap_or_default();
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| ProductionBoxTransportError::InvalidHttpResponse(format!(
            "box ControlService returned an invalid status line: {status_line}"
        )))?;
    if (200..300).contains(&status) {
        return Ok(response[header_end + 4..].to_vec());
    }
    let body_text = String::from_utf8_lossy(&response[header_end + 4..]);
    let body_text = body_text.chars().take(512).collect::<String>();
    Err(ProductionBoxTransportError::HttpStatus {
        status,
        body: body_text,
    })
}
