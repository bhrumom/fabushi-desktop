use std::sync::Arc;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct ProductionExecClientMessage {
    json: Value,
}

impl ProductionExecClientMessage {
    pub fn as_json(&self) -> &Value {
        &self.json
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayExecControl {
    Throw {
        error: String,
        stack_trace: Option<String>,
    },
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
    pub fn decode_client(
        &self,
        json: Value,
    ) -> Result<ProductionExecClientMessage, String> {
        if !json.is_object() {
            return Err("ExecClientMessage JSON must be an object".into());
        }
        Ok(ProductionExecClientMessage { json })
    }

    pub fn decode_control(
        &self,
        json: &Value,
    ) -> Result<GatewayExecControl, String> {
        let object = json
            .as_object()
            .ok_or_else(|| "ExecClientControlMessage JSON must be an object".to_string())?;

        let recognized = [
            object.contains_key("throw"),
            object.contains_key("streamClose"),
            object.contains_key("heartbeat"),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if recognized > 1 {
            return Err("ExecClientControlMessage JSON has conflicting oneof fields".into());
        }

        if let Some(thrown) = object.get("throw") {
            let thrown = thrown
                .as_object()
                .ok_or_else(|| "ExecClientControlMessage.throw must be an object".to_string())?;
            let error = thrown
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let stack_trace = thrown
                .get("stackTrace")
                .and_then(Value::as_str)
                .map(str::to_string);
            return Ok(GatewayExecControl::Throw {
                error,
                stack_trace,
            });
        }

        if object.contains_key("streamClose") {
            return Ok(GatewayExecControl::StreamClose);
        }

        Ok(GatewayExecControl::Unknown)
    }

    pub fn create_remote_accessor<M>(
        &self,
        manager: Arc<M>,
    ) -> RemoteResourceAccessor<M> {
        RemoteResourceAccessor::new(manager)
    }
}

pub const PRODUCTION_LOCAL_EXEC_CODEC: ProductionLocalExecCodec = ProductionLocalExecCodec;
