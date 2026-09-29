use std::sync::Arc;

use serde_json::{Value, json};

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
    json: Value,
    payload: ProductionExecClientPayload,
}

impl ProductionExecClientMessage {
    pub fn as_json(&self) -> &Value { &self.json }
    pub fn payload(&self) -> &ProductionExecClientPayload { &self.payload }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayExecControl {
    Throw { error: String, stack_trace: Option<String> },
    StreamClose,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RemoteResourceAccessor<M> { manager: Arc<M> }

impl<M> RemoteResourceAccessor<M> {
    pub fn new(manager: Arc<M>) -> Self { Self { manager } }
    pub fn manager(&self) -> Arc<M> { Arc::clone(&self.manager) }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProductionLocalExecCodec;

impl ProductionLocalExecCodec {
    pub fn decode_client(&self, json: Value) -> Result<ProductionExecClientMessage, String> {
        let object = json.as_object().ok_or_else(|| "ExecClientMessage JSON must be an object".to_string())?;
        let recognized = [object.contains_key("shellStream"), object.contains_key("readResult")]
            .into_iter().filter(|present| *present).count();
        if recognized > 1 {
            return Err("ExecClientMessage JSON has conflicting oneof fields".into());
        }
        let payload = if let Some(stream) = object.get("shellStream") {
            ProductionExecClientPayload::ShellStream(decode_shell_stream(stream)?)
        } else if let Some(result) = object.get("readResult") {
            if !result.is_object() {
                return Err("ExecClientMessage.readResult must be an object".into());
            }
            ProductionExecClientPayload::ReadResult(result.clone())
        } else {
            ProductionExecClientPayload::Other
        };
        Ok(ProductionExecClientMessage { json, payload })
    }

    pub fn decode_control(&self, json: &Value) -> Result<GatewayExecControl, String> {
        let object = json.as_object().ok_or_else(|| "ExecClientControlMessage JSON must be an object".to_string())?;
        let recognized = [
            object.contains_key("throw"),
            object.contains_key("streamClose"),
            object.contains_key("heartbeat"),
        ].into_iter().filter(|present| *present).count();
        if recognized > 1 {
            return Err("ExecClientControlMessage JSON has conflicting oneof fields".into());
        }
        if let Some(thrown) = object.get("throw") {
            let thrown = thrown.as_object().ok_or_else(|| "ExecClientControlMessage.throw must be an object".to_string())?;
            return Ok(GatewayExecControl::Throw {
                error: thrown.get("error").and_then(Value::as_str).unwrap_or_default().to_string(),
                stack_trace: thrown.get("stackTrace").and_then(Value::as_str).map(str::to_string),
            });
        }
        if object.contains_key("streamClose") {
            return Ok(GatewayExecControl::StreamClose);
        }
        Ok(GatewayExecControl::Unknown)
    }

    pub fn shell_stream_server_message(
        &self,
        id: u32,
        command: &str,
        working_directory: &str,
        tool_call_id: &str,
    ) -> Value {
        json!({
            "id": id,
            "shellStreamArgs": {
                "command": command,
                "workingDirectory": working_directory,
                "toolCallId": tool_call_id
            }
        })
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
        let mut args = json!({ "path": path, "toolCallId": tool_call_id });
        if let Some(offset) = offset { args["offset"] = json!(offset); }
        if let Some(limit) = limit { args["limit"] = json!(limit); }
        if let Some(encoding_hint) = encoding_hint { args["encodingHint"] = json!(encoding_hint); }
        json!({ "id": id, "readArgs": args })
    }

    pub fn create_remote_accessor<M>(&self, manager: Arc<M>) -> RemoteResourceAccessor<M> {
        RemoteResourceAccessor::new(manager)
    }
}

fn decode_shell_stream(value: &Value) -> Result<ProductionShellStreamEvent, String> {
    let object = value.as_object().ok_or_else(|| "ExecClientMessage.shellStream must be an object".to_string())?;
    let cases = [
        "stdout", "stderr", "exit", "start", "rejected", "permissionDenied",
        "backgrounded", "hookContext", "sandboxUnsupported",
    ];
    let recognized = cases.iter().filter(|key| object.contains_key(**key)).count();
    if recognized > 1 {
        return Err("ShellStream JSON has conflicting oneof fields".into());
    }
    if let Some(stdout) = object.get("stdout") {
        return Ok(ProductionShellStreamEvent::Stdout(
            stdout.get("data").and_then(Value::as_str).unwrap_or_default().to_string()
        ));
    }
    if let Some(stderr) = object.get("stderr") {
        return Ok(ProductionShellStreamEvent::Stderr(
            stderr.get("data").and_then(Value::as_str).unwrap_or_default().to_string()
        ));
    }
    if let Some(exit) = object.get("exit") {
        let exit = exit.as_object().ok_or_else(|| "ShellStream.exit must be an object".to_string())?;
        return Ok(ProductionShellStreamEvent::Exit {
            code: exit.get("code").and_then(Value::as_u64).and_then(|value| u32::try_from(value).ok()).unwrap_or_default(),
            cwd: exit.get("cwd").and_then(Value::as_str).unwrap_or_default().to_string(),
            aborted: exit.get("aborted").and_then(Value::as_bool).unwrap_or(false),
        });
    }
    if let Some(rejected) = object.get("rejected") {
        return Ok(ProductionShellStreamEvent::Rejected(
            rejected.get("reason").or_else(|| rejected.get("message")).and_then(Value::as_str)
                .unwrap_or("Shell command was rejected").to_string()
        ));
    }
    if let Some(denied) = object.get("permissionDenied") {
        return Ok(ProductionShellStreamEvent::PermissionDenied(
            denied.get("reason").or_else(|| denied.get("message")).and_then(Value::as_str)
                .unwrap_or("Shell command permission was denied").to_string()
        ));
    }
    if object.contains_key("start") { return Ok(ProductionShellStreamEvent::Start); }
    if object.contains_key("backgrounded") { return Ok(ProductionShellStreamEvent::Backgrounded); }
    Ok(ProductionShellStreamEvent::Other)
}

pub const PRODUCTION_LOCAL_EXEC_CODEC: ProductionLocalExecCodec = ProductionLocalExecCodec;
