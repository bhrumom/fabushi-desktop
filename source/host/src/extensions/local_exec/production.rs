use std::sync::Arc;

use serde_json::Value;

use super::agent_v1_exec_wire::{
    ExecClientControlMessage, ExecClientMessage, ExecServerMessage,
    exec_client_control_message, exec_client_message, shell_stream,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionShellStreamEvent {
    Stdout(String),
    Stderr(String),
    Exit { code: u32, cwd: String, aborted: bool },
    Rejected(String),
    PermissionDenied(String),
    Start,
    Backgrounded,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProductionExecClientPayload {
    ShellStream(ProductionShellStreamEvent),
    ReadResult(Value),
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductionExecClientMessage {
    message: ExecClientMessage,
    payload: ProductionExecClientPayload,
}

impl ProductionExecClientMessage {
    pub fn generated(&self) -> &ExecClientMessage {
        &self.message
    }

    pub fn payload(&self) -> &ProductionExecClientPayload {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayExecControl {
    Throw { error: String, stack_trace: Option<String> },
    StreamClose,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RemoteResourceAccessor<M> {
    manager: Arc<M>,
}

impl<M> RemoteResourceAccessor<M> {
    pub fn new(manager: Arc<M>) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> Arc<M> {
        Arc::clone(&self.manager)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProductionLocalExecCodec;

impl ProductionLocalExecCodec {
    pub fn decode_client(&self, json: Value) -> Result<ProductionExecClientMessage, String> {
        let message = ExecClientMessage::from_json_ignoring_unknown_fields(&json)?;
        let payload = match message.message.as_ref() {
            Some(exec_client_message::Message::ShellStream(stream)) => {
                let event = match stream.event.as_ref() {
                    Some(shell_stream::Event::Stdout(value)) => {
                        ProductionShellStreamEvent::Stdout(value.data.clone())
                    }
                    Some(shell_stream::Event::Stderr(value)) => {
                        ProductionShellStreamEvent::Stderr(value.data.clone())
                    }
                    Some(shell_stream::Event::Exit(value)) => ProductionShellStreamEvent::Exit {
                        code: value.code,
                        cwd: value.cwd.clone(),
                        aborted: value.aborted,
                    },
                    Some(shell_stream::Event::Rejected(value)) => {
                        ProductionShellStreamEvent::Rejected(value.reason.clone())
                    }
                    Some(shell_stream::Event::PermissionDenied(value)) => {
                        ProductionShellStreamEvent::PermissionDenied(value.error.clone())
                    }
                    Some(shell_stream::Event::Start(_)) => ProductionShellStreamEvent::Start,
                    Some(shell_stream::Event::Backgrounded(_)) => {
                        ProductionShellStreamEvent::Backgrounded
                    }
                    Some(shell_stream::Event::HookContext(_))
                    | Some(shell_stream::Event::SandboxUnsupported(_))
                    | None => ProductionShellStreamEvent::Other,
                };
                ProductionExecClientPayload::ShellStream(event)
            }
            Some(exec_client_message::Message::ReadResult(result)) => {
                ProductionExecClientPayload::ReadResult(result.to_json())
            }
            None => ProductionExecClientPayload::Other,
        };
        Ok(ProductionExecClientMessage { message, payload })
    }

    pub fn decode_control(&self, json: &Value) -> Result<GatewayExecControl, String> {
        let control =
            ExecClientControlMessage::from_json_ignoring_unknown_fields(json)?.message;
        Ok(match control {
            Some(exec_client_control_message::Message::Throw(value)) => {
                GatewayExecControl::Throw {
                    error: value.error,
                    stack_trace: value.stack_trace,
                }
            }
            Some(exec_client_control_message::Message::StreamClose(_)) => {
                GatewayExecControl::StreamClose
            }
            Some(exec_client_control_message::Message::Heartbeat(_)) | None => {
                GatewayExecControl::Unknown
            }
        })
    }

    pub fn shell_stream_server_message(
        &self,
        id: u32,
        command: &str,
        working_directory: &str,
        tool_call_id: &str,
    ) -> Value {
        ExecServerMessage::shell_stream(id, command, working_directory, tool_call_id).to_json()
    }

    pub fn read_server_message(
        &self,
        id: u32,
        path: &str,
        tool_call_id: &str,
        offset: Option<i32>,
        limit: Option<u32>,
        encoding_hint: Option<&str>,
    ) -> Value {
        ExecServerMessage::read(id, path, tool_call_id, offset, limit, encoding_hint).to_json()
    }

    pub fn create_remote_accessor<M>(&self, manager: Arc<M>) -> RemoteResourceAccessor<M> {
        RemoteResourceAccessor::new(manager)
    }
}

pub const PRODUCTION_LOCAL_EXEC_CODEC: ProductionLocalExecCodec = ProductionLocalExecCodec;
