#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReachabilityOutcome { Reachable, Timeout, Dns, Refused, Http(u16), Network }

impl ReachabilityOutcome {
    pub fn retryable(self) -> bool {
        matches!(self, Self::Timeout | Self::Dns | Self::Refused | Self::Network | Self::Http(500..=599))
    }
}
