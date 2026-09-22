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

#[derive(Debug)]
pub struct GatewayHostSupervisor {
    reachability: GatewayReachability,
    health_epoch: u64,
    transport_live: bool,
    last_healthy_ms: Option<u64>,
    health_ttl_ms: u64,
}

impl GatewayHostSupervisor {
    pub fn new(health_ttl_ms: u64) -> Self {
        Self {
            reachability: GatewayReachability::Unknown,
            health_epoch: 0,
            transport_live: false,
            last_healthy_ms: None,
            health_ttl_ms,
        }
    }

    pub fn health_epoch(&self) -> u64 { self.health_epoch }

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

    pub fn invalidate(&mut self) {
        self.health_epoch = self.health_epoch.saturating_add(1);
        self.last_healthy_ms = None;
        self.transport_live = false;
        self.reachability = GatewayReachability::Unknown;
    }

    pub fn decision(&self, now_ms: u64) -> GatewayHealthDecision {
        if self.transport_live {
            return GatewayHealthDecision::UseCached;
        }
        if let Some(last) = self.last_healthy_ms {
            if now_ms.saturating_sub(last) < self.health_ttl_ms {
                return GatewayHealthDecision::UseCached;
            }
        }
        match self.reachability {
            GatewayReachability::Unreachable => GatewayHealthDecision::Reconnect,
            _ => GatewayHealthDecision::Probe,
        }
    }
}
