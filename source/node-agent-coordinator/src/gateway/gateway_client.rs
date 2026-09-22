use std::time::Duration;
use super::gateway_reachability::ReachabilityOutcome;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GatewayClientState { pub connected: bool, pub reconnect_attempt: u32 }

impl GatewayClientState {
    pub fn connected(&mut self) { self.connected = true; self.reconnect_attempt = 0; }
    pub fn disconnected(&mut self, outcome: ReachabilityOutcome) -> Option<Duration> {
        self.connected = false;
        if !outcome.retryable() { return None; }
        self.reconnect_attempt = self.reconnect_attempt.saturating_add(1);
        let exponent = self.reconnect_attempt.saturating_sub(1).min(4);
        Some(Duration::from_secs((1_u64 << exponent).min(10)))
    }
}
