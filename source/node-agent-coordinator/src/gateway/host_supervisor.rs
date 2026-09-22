use std::fs;
use std::io;
use std::path::Path;
use serde::Deserialize;
use std::collections::BTreeMap;

pub const HEALTH_TIMEOUT_MS: u64 = 1_500;
pub const HEALTH_PROBE_TTL_MS: u64 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayReachability {
    Unknown,
    Reachable,
    Unreachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayHealthDecision {
    UseCached,
    Probe,
    Reconnect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayConnection {
    pub base_url: String,
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionAttempt {
    pub health_epoch: u64,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the gateway connection resolve was abandoned before it answered")]
pub struct GatewayConnectResolveAbandonedError;

#[derive(Debug)]
pub struct GatewayHostSupervisor {
    reachability: GatewayReachability,
    health_epoch: u64,
    transport_live: bool,
    last_healthy_ms: Option<u64>,
    health_ttl_ms: u64,
    connection: Option<GatewayConnection>,
    latest_attempt: Option<ConnectionAttempt>,
    next_attempt_generation: u64,
}

impl GatewayHostSupervisor {
    pub fn new(health_ttl_ms: u64) -> Self {
        Self {
            reachability: GatewayReachability::Unknown,
            health_epoch: 0,
            transport_live: false,
            last_healthy_ms: None,
            health_ttl_ms,
            connection: None,
            latest_attempt: None,
            next_attempt_generation: 0,
        }
    }

    pub fn health_epoch(&self) -> u64 {
        self.health_epoch
    }

    pub fn connection(&self) -> Option<&GatewayConnection> {
        self.connection.as_ref()
    }

    pub fn latest_attempt(&self) -> Option<ConnectionAttempt> {
        self.latest_attempt
    }

    pub fn mark_transport_live(&mut self, live: bool) {
        self.transport_live = live;
        if live {
            self.reachability = GatewayReachability::Reachable;
        }
    }

    pub fn record_health(&mut self, now_ms: u64, reachable: bool) {
        self.reachability = if reachable {
            GatewayReachability::Reachable
        } else {
            GatewayReachability::Unreachable
        };
        self.last_healthy_ms = reachable.then_some(now_ms);
    }

    pub fn begin_connection_attempt(&mut self) -> ConnectionAttempt {
        if let Some(current) = self.latest_attempt {
            if current.health_epoch == self.health_epoch {
                return current;
            }
        }
        self.next_attempt_generation = self.next_attempt_generation.saturating_add(1);
        let attempt = ConnectionAttempt {
            health_epoch: self.health_epoch,
            generation: self.next_attempt_generation,
        };
        self.latest_attempt = Some(attempt);
        attempt
    }

    pub fn settle_connection_attempt(
        &mut self,
        attempt: ConnectionAttempt,
        connection: GatewayConnection,
    ) -> Result<(), GatewayConnectResolveAbandonedError> {
        if attempt.health_epoch != self.health_epoch || self.latest_attempt != Some(attempt) {
            return Err(GatewayConnectResolveAbandonedError);
        }
        self.connection = Some(connection);
        self.last_healthy_ms = None;
        self.reachability = GatewayReachability::Unknown;
        self.latest_attempt = None;
        Ok(())
    }

    pub fn fail_connection_attempt(
        &mut self,
        attempt: ConnectionAttempt,
    ) -> Result<(), GatewayConnectResolveAbandonedError> {
        if attempt.health_epoch != self.health_epoch || self.latest_attempt != Some(attempt) {
            return Err(GatewayConnectResolveAbandonedError);
        }
        self.latest_attempt = None;
        self.reachability = GatewayReachability::Unreachable;
        Ok(())
    }

    pub fn invalidate(&mut self) {
        self.health_epoch = self.health_epoch.saturating_add(1);
        self.last_healthy_ms = None;
        self.transport_live = false;
        self.reachability = GatewayReachability::Unknown;
        self.latest_attempt = None;
    }

    pub fn decision(&self, now_ms: u64) -> GatewayHealthDecision {
        if self.connection.is_some() && self.transport_live {
            return GatewayHealthDecision::UseCached;
        }
        if self.connection.is_some() {
            if let Some(last) = self.last_healthy_ms {
                if now_ms.saturating_sub(last) < self.health_ttl_ms {
                    return GatewayHealthDecision::UseCached;
                }
            }
            if self.reachability == GatewayReachability::Unreachable {
                return GatewayHealthDecision::Reconnect;
            }
            return GatewayHealthDecision::Probe;
        }
        match self.reachability {
            GatewayReachability::Unreachable => GatewayHealthDecision::Reconnect,
            _ => GatewayHealthDecision::Probe,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayDiscoveryInfo {
    pub port: u16,
    pub pid: u32,
    pub started_at: u64,
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub token: Option<String>,
}

impl GatewayDiscoveryInfo {
    pub fn into_connection(self) -> io::Result<GatewayConnection> {
        if self.port == 0 || self.pid == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid Host gateway discovery port/pid",
            ));
        }
        let scheme = self.scheme.unwrap_or_else(|| "http".into());
        if scheme != "http" && scheme != "https" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid Host gateway discovery scheme",
            ));
        }
        let host = self
            .host
            .filter(|host| !host.trim().is_empty())
            .unwrap_or_else(|| "127.0.0.1".into());
        let host_for_url = if host.contains(':') && !host.starts_with('[') {
            format!("[{host}]")
        } else {
            host
        };
        let mut headers = BTreeMap::new();
        if let Some(token) = self.token.filter(|token| !token.is_empty()) {
            headers.insert("authorization".into(), format!("Bearer {token}"));
        }
        Ok(GatewayConnection {
            base_url: format!("{scheme}://{host_for_url}:{}", self.port),
            headers,
        })
    }
}

pub fn read_gateway_discovery(path: &Path) -> io::Result<GatewayConnection> {
    let bytes = fs::read(path)?;
    let info = serde_json::from_slice::<GatewayDiscoveryInfo>(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    info.into_connection()
}
