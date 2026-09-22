use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsDiagnostic { pub host: String, pub addresses: Vec<IpAddr> }

impl DnsDiagnostic {
    pub fn resolved(&self) -> bool { !self.addresses.is_empty() }
}
