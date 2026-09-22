#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VncProxyDescriptor { pub host: String, pub port: u16 }

impl VncProxyDescriptor {
    pub fn loopback(port: u16) -> Result<Self, &'static str> {
        if port == 0 { return Err("VNC proxy port must be non-zero"); }
        Ok(Self { host: "127.0.0.1".into(), port })
    }
    pub fn is_loopback(&self) -> bool {
        matches!(self.host.as_str(), "127.0.0.1" | "::1" | "localhost")
    }
}
