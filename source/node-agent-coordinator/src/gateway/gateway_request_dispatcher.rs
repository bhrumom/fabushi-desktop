use std::collections::HashMap;
use serde_json::Value;
use crate::protocol::Failure;

pub type GatewayHandler = Box<dyn Fn(Value) -> Result<Value, Failure> + Send + Sync>;

#[derive(Default)]
pub struct GatewayRequestDispatcher { handlers: HashMap<String, GatewayHandler> }

impl std::fmt::Debug for GatewayRequestDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayRequestDispatcher").field("method_count", &self.handlers.len()).finish()
    }
}

impl GatewayRequestDispatcher {
    pub fn register(&mut self, method: impl Into<String>, handler: GatewayHandler) {
        self.handlers.insert(method.into(), handler);
    }
    pub fn dispatch(&self, method: &str, args: Value) -> Result<Value, Failure> {
        self.handlers.get(method)
            .ok_or_else(|| Failure::new("GATEWAY_UNKNOWN_METHOD", format!("no gateway method {method}")))
            .and_then(|handler| handler(args))
    }
}
