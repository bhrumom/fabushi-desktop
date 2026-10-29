use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::allow_from::is_normalized_sender_allowed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifierAuthentication {
    Mutable = 0,
    Asserted = 1,
    Authenticated = 2,
    Verified = 3,
    Signed = 4,
}

/// Checks whether an actual authentication level meets the required minimum threshold.
pub fn meets_identifier_authentication(
    actual: IdentifierAuthentication,
    minimum: IdentifierAuthentication,
) -> bool {
    actual >= minimum
}

#[derive(Clone)]
pub struct IngressLifecycle {
    pub deferred_heartbeat_interval_ms: Option<u64>,
    pub on_adopted: Arc<dyn Fn() -> Result<(), String> + Send + Sync>,
    pub on_adoption_finalizing: Arc<dyn Fn() + Send + Sync>,
    pub on_failed: Arc<dyn Fn(&str) + Send + Sync>,
    pub on_cancelled: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_abandoned: Arc<dyn Fn() + Send + Sync>,
}

pub struct FannedInIngressLifecycle {
    pub deferred_heartbeat_interval_ms: Option<u64>,
    pub supports_cancellation: bool,
    handed_off: Arc<AtomicBool>,
    lifecycles: Vec<IngressLifecycle>,
}

impl FannedInIngressLifecycle {
    pub fn new(lifecycles: Vec<Option<IngressLifecycle>>) -> Option<Self> {
        let valid: Vec<IngressLifecycle> = lifecycles.into_iter().flatten().collect();
        if valid.is_empty() {
            return None;
        }

        let intervals: Vec<u64> = valid
            .iter()
            .filter_map(|l| l.deferred_heartbeat_interval_ms)
            .filter(|&ms| ms > 0)
            .collect();
        let deferred_heartbeat_interval_ms = intervals.into_iter().min();

        let supports_cancellation = valid.iter().all(|l| l.on_cancelled.is_some());

        Some(Self {
            deferred_heartbeat_interval_ms,
            supports_cancellation,
            handed_off: Arc::new(AtomicBool::new(false)),
            lifecycles: valid,
        })
    }

    pub fn on_adoption_finalizing(&self) {
        for l in &self.lifecycles {
            (l.on_adoption_finalizing)();
        }
    }

    pub fn on_adopted(&self) -> Result<(), String> {
        self.handed_off.store(true, Ordering::SeqCst);
        for (i, l) in self.lifecycles.iter().enumerate() {
            if let Err(e) = (l.on_adopted)() {
                // Fail remaining lifecycles
                for rem in &self.lifecycles[i..] {
                    (rem.on_failed)(&e);
                }
                return Err(e);
            }
        }
        Ok(())
    }

    pub fn on_cancelled(&self) {
        self.handed_off.store(true, Ordering::SeqCst);
        for l in &self.lifecycles {
            if let Some(c) = &l.on_cancelled {
                c();
            } else {
                (l.on_abandoned)();
            }
        }
    }

    pub fn on_failed(&self, error: &str) {
        self.handed_off.store(true, Ordering::SeqCst);
        for l in &self.lifecycles {
            (l.on_failed)(error);
        }
    }

    pub fn on_abandoned(&self) {
        self.handed_off.store(true, Ordering::SeqCst);
        for l in &self.lifecycles {
            (l.on_abandoned)();
        }
    }

    pub fn settle(&self) -> Result<(), String> {
        if !self.handed_off.swap(true, Ordering::SeqCst) {
            self.on_adopted()
        } else {
            Ok(())
        }
    }

    pub fn abandon(&self, error: Option<&str>) {
        if !self.handed_off.swap(true, Ordering::SeqCst) {
            if let Some(e) = error {
                for l in &self.lifecycles {
                    (l.on_failed)(e);
                }
            } else {
                for l in &self.lifecycles {
                    (l.on_abandoned)();
                }
            }
        }
    }

    pub fn cancel(&self) {
        if !self.handed_off.swap(true, Ordering::SeqCst) {
            self.on_cancelled();
        }
    }
}

/// Combines multiple durable channel ingress lifecycles into one coordinated lifecycle.
pub fn fan_in_channel_ingress_lifecycles(
    lifecycles: Vec<Option<IngressLifecycle>>,
) -> Option<FannedInIngressLifecycle> {
    FannedInIngressLifecycle::new(lifecycles)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelMessageIngressResult {
    pub admission: String,       // "dispatch" | "drop" | "block"
    pub decision: String,        // "allow" | "block"
    pub reason_code: Option<String>,
    pub command_authorized: bool,
    pub effective_allow_from: Vec<String>,
}

/// Resolves message ingress access, evaluating allowlist membership, command authority, and route status.
pub fn resolve_channel_message_ingress_simple(
    sender_id: &str,
    allow_from: &[String],
    is_route_enabled: bool,
    is_group: bool,
    dm_policy: &str,
) -> ChannelMessageIngressResult {
    if !is_route_enabled {
        return ChannelMessageIngressResult {
            admission: "block".to_string(),
            decision: "block".to_string(),
            reason_code: Some("route_blocked".to_string()),
            command_authorized: false,
            effective_allow_from: allow_from.to_vec(),
        };
    }

    let is_sender_allowed = is_normalized_sender_allowed(sender_id, allow_from, None);

    if !is_group && dm_policy == "allowlist" && !is_sender_allowed {
        return ChannelMessageIngressResult {
            admission: "block".to_string(),
            decision: "block".to_string(),
            reason_code: Some("sender_not_allowed".to_string()),
            command_authorized: false,
            effective_allow_from: allow_from.to_vec(),
        };
    }

    ChannelMessageIngressResult {
        admission: "dispatch".to_string(),
        decision: "allow".to_string(),
        reason_code: None,
        command_authorized: true,
        effective_allow_from: allow_from.to_vec(),
    }
}
