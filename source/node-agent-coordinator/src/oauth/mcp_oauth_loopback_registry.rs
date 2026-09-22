use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct McpOAuthLoopbackRegistry { by_origin: HashMap<String, u16> }

impl McpOAuthLoopbackRegistry {
    pub fn bind(&mut self, origin: impl Into<String>, port: u16) -> Result<(), &'static str> {
        if port == 0 { return Err("loopback OAuth port must be non-zero"); }
        self.by_origin.insert(origin.into(), port);
        Ok(())
    }
    pub fn port(&self, origin: &str) -> Option<u16> { self.by_origin.get(origin).copied() }
    pub fn unbind(&mut self, origin: &str) { self.by_origin.remove(origin); }
}
