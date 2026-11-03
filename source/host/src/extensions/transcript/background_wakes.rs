use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::connectors::channel_delivery::ChannelOutboundMessage;
use crate::extensions::session::agent_db::{
    ChannelInboundAppendAttempt, ChannelInboundAppendOutcome,
};

pub const TIMELINE_EVENT_WAKE_CUE: &str = "[event]";
pub const CHANNEL_INBOUND_WAKE_CUE: &str = "[inbound]";
pub const CHANNEL_DELIVERY_FAILED_WAKE_CUE: &str = "[channel-delivery-failed]";

pub fn format_channel_address_value(address: &Value) -> Option<String> {
    if let Some(raw) = address.as_str() {
        let trimmed = raw.trim();
        return (!trimmed.is_empty()).then(|| trimmed.to_string());
    }
    let platform = address.get("platform")?.as_str()?.trim();
    let chat = address.get("chat")?.as_str()?.trim();
    if platform.is_empty() || chat.is_empty() {
        return None;
    }
    Some(format!("{platform}:{chat}"))
}

fn channel_platform_display_name(address: &Value) -> String {
    let platform = if let Some(raw) = address.as_str() {
        channel_platform(raw).unwrap_or_default()
    } else {
        address
            .get("platform")
            .and_then(Value::as_str)
            .unwrap_or_default()
    };
    channel_platform_name(platform).to_string()
}

pub fn distinct_inbound_channel_addresses(envelopes: &[Value]) -> Vec<String> {
    distinct_channel_addresses(
        envelopes
            .iter()
            .filter_map(|envelope| envelope.get("address"))
            .filter_map(format_channel_address_value),
    )
}

pub fn build_channel_inbound_wake_prompt(envelopes: &[Value]) -> String {
    let mut grouped: Vec<(String, String, Vec<&Value>)> = Vec::new();
    for envelope in envelopes {
        let Some(address) = envelope.get("address") else {
            continue;
        };
        let Some(token) = format_channel_address_value(address) else {
            continue;
        };
        if let Some((_, _, bucket)) = grouped.iter_mut().find(|(key, _, _)| key == &token) {
            bucket.push(envelope);
        } else {
            grouped.push((
                token,
                channel_platform_display_name(address),
                vec![envelope],
            ));
        }
    }

    let has_message = envelopes
        .iter()
        .any(|envelope| envelope.get("reaction").map_or(true, Value::is_null));
    let plural = if envelopes.len() == 1 { "" } else { "s" };
    let opening = if has_message {
        format!("{CHANNEL_INBOUND_WAKE_CUE} New message{plural} on a channel you are connected to.")
    } else {
        format!(
            "{CHANNEL_INBOUND_WAKE_CUE} New reaction{plural} on a channel you are connected to."
        )
    };
    let mut blocks = Vec::new();
    for (token, platform_name, bucket) in grouped {
        let mut lines = Vec::new();
        for envelope in bucket {
            let sender = envelope
                .get("sender")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if let Some(reaction) = envelope.get("reaction").filter(|value| !value.is_null()) {
                let emoji = reaction
                    .get("emoji")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let summary = reaction
                    .get("messageQuote")
                    .and_then(Value::as_str)
                    .filter(|quote| !quote.is_empty())
                    .map(|quote| format!("reacted {emoji} to your message: \"{quote}\""))
                    .unwrap_or_else(|| format!("reacted {emoji} to a message"));
                lines.push(format!("  {sender} {summary}"));
            } else {
                let text = envelope
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                lines.push(format!("  {sender}: {text}"));
            }
        }
        blocks.push(format!(
            "On {platform_name}, from {token}:\n{}",
            lines.join("\n")
        ));
    }
    let closing = if has_message {
        "Reply to them by calling SendMessage with the channel target set to the address shown above. Open with a quick one-line acknowledgement first, then send progress and the result as separate messages as they happen, never one long message at the end. Keep each message short: this is a messaging app, so reply in brief, chat-style messages (lead with the answer, a sentence or two), not long ones. Keep working the rest of your task too, but do not leave them hanging."
    } else {
        "You don't need to reply; act on a reaction only if it's useful (e.g. acknowledge, adjust, or continue). If you do choose to respond, use SendMessage with the channel target shown above."
    };
    [
        opening,
        "This is activity from someone on an outside platform, not the user typing in this app."
            .to_string(),
        String::new(),
        blocks.join("\n\n"),
        String::new(),
        closing.to_string(),
    ]
    .join("\n")
}

pub fn build_channel_outbound_message(message: &Value) -> Option<ChannelOutboundMessage> {
    let operation_id = message
        .get("deliveryOperationId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    match message.get("type").and_then(Value::as_str)? {
        "text" => {
            let content = message.get("content").and_then(Value::as_str).unwrap_or_default();
            let image_url = message
                .get("images")
                .and_then(Value::as_array)
                .and_then(|images| {
                    images.iter().find_map(|image| {
                        image
                            .get("url")
                            .and_then(Value::as_str)
                            .filter(|url| !url.is_empty())
                    })
                });
            if let Some(url) = image_url {
                return Some(ChannelOutboundMessage::Attachment {
                    url: url.to_string(),
                    caption: (!content.is_empty()).then(|| content.to_string()),
                    operation_id,
                });
            }
            (!content.is_empty()).then(|| ChannelOutboundMessage::Text {
                text: content.to_string(),
                operation_id,
            })
        }
        "attachment" => {
            let url = message.get("url").and_then(Value::as_str).unwrap_or_default();
            let alt = message
                .get("alt")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty());
            if !url.is_empty() {
                return Some(ChannelOutboundMessage::Attachment {
                    url: url.to_string(),
                    caption: alt.map(ToOwned::to_owned),
                    operation_id,
                });
            }
            alt.map(|text| ChannelOutboundMessage::Text {
                text: text.to_string(),
                operation_id,
            })
        }
        _ => None,
    }
}

fn channel_platform(address_token: &str) -> Option<&str> {
    let trimmed = address_token.trim();
    let separator = trimmed.find(':')?;
    if separator == 0 {
        return None;
    }
    let platform = trimmed[..separator].trim();
    let chat = trimmed[separator + 1..].trim();
    if platform.is_empty() || chat.is_empty() {
        return None;
    }
    Some(platform)
}

fn channel_platform_name(platform: &str) -> &str {
    match platform {
        "slack" => "Slack",
        "discord" => "Discord",
        other => other,
    }
}

pub fn humanize_channel_delivery_failure(address_token: &str, raw_message: &str) -> String {
    let trimmed = raw_message.trim();
    let Some(platform) = channel_platform(address_token) else {
        return format!(
            "\"{address_token}\" isn't a valid channel address, so that message wasn't delivered."
        );
    };
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("not a valid channel address") {
        return format!(
            "\"{address_token}\" isn't a valid channel address, so that message wasn't delivered."
        );
    }
    if trimmed == "No channel delivery mechanism is registered." {
        return "Channel messaging isn't available on this computer, so that message wasn't delivered."
            .to_string();
    }
    let platform_name = channel_platform_name(platform);
    if lower.contains("no live ") && lower.contains(" connection") {
        return format!(
            "{platform_name} isn't connected on this computer, so that message wasn't delivered. Connect {platform_name} (add its token) to send there."
        );
    }
    format!("Couldn't deliver that message to {platform_name}: {trimmed}")
}

pub fn build_channel_delivery_failure_wake_prompt(failures: &[Value]) -> String {
    let mut lines = vec![
        format!(
            "{CHANNEL_DELIVERY_FAILED_WAKE_CUE} A message you tried to send to a channel did not go through."
        ),
        "This is a system notice about your own outbound send, not the user typing in this app. You may have already told the user it was sent, so correct the record.".to_string(),
    ];
    lines.extend(failures.iter().map(|failure| {
        format!(
            "- To {}: {}",
            failure
                .get("addressToken")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            failure
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or_default()
        )
    }));
    let plural = if failures.len() == 1 { "" } else { "s" };
    lines.push(format!(
        "Tell the user plainly here, in this in-app chat (a SendMessage with no channel target), that the message{plural} didn't go through and why, so they aren't left believing it was delivered. Don't silently retry the same channel; if it isn't connected, offer to help connect it."
    ));
    lines.join("\n")
}

pub fn redrivable_inbound_envelopes(envelopes: &[Value]) -> Vec<Value> {
    envelopes
        .iter()
        .filter(|envelope| envelope.get("isRedriven").and_then(Value::as_bool) != Some(true))
        .map(|envelope| {
            let mut envelope = envelope.clone();
            if let Some(object) = envelope.as_object_mut() {
                object.insert("isDisplayed".into(), Value::Bool(true));
                object.insert("isRedriven".into(), Value::Bool(true));
            }
            envelope
        })
        .collect()
}

pub fn describe_timeline_event(event: &Value) -> String {
    let kind = event
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match kind {
        "name-changed" => format!(
            "Renamed to {}",
            event.get("to").and_then(Value::as_str).unwrap_or_default()
        ),
        "channel-connected" => format!(
            "Connected to {}",
            event
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or_default()
        ),
        "channel-disconnected" => format!(
            "Disconnected from {}",
            event
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or_default()
        ),
        "automation-changed" => {
            let action = match event
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or_default()
            {
                "created" => "Created",
                "updated" => "Updated",
                "enabled" => "Enabled",
                "disabled" => "Disabled",
                "deleted" => "Deleted",
                _ => "Changed",
            };
            format!(
                "{} automation \"{}\"",
                action,
                event
                    .get("automationName")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
            )
        }
        _ => "Updated this conversation".to_string(),
    }
}

pub fn build_timeline_event_wake_prompt(events: &[Value]) -> String {
    let mut lines = vec![
        format!(
            "{} Something about this conversation just changed.",
            TIMELINE_EVENT_WAKE_CUE
        ),
        "This is a system event recorded in your timeline, not the user typing in this app, and possibly something you did yourself.".to_string(),
    ];
    lines.extend(
        events
            .iter()
            .map(|event| format!("- {}", describe_timeline_event(event))),
    );
    lines.push(
        "If it is worth acknowledging to the user, reply with SendMessage; otherwise it is fine to stay silent.".to_string(),
    );
    lines.join("\n")
}

pub fn distinct_channel_addresses<I, S>(addresses: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for address in addresses {
        let address = address.into();
        if seen.insert(address.clone()) {
            out.push(address);
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboundAppendState {
    Active,
    NotWritten,
    Unknown,
    Committed,
}

/// A worker's snapshot of the queue-owned head. Tokens are checked when the
/// acknowledgement returns, so an old worker cannot settle a replacement.
#[derive(Debug, Clone, PartialEq)]
pub struct InboundAppendWork<T> {
    pub claim_id: String,
    pub worker_token: String,
    pub envelopes: Vec<T>,
    pub attempt: Option<ChannelInboundAppendAttempt>,
}

/// Retry is queue custody, never permission to dispatch a possibly written head.
#[derive(Debug, Clone, PartialEq)]
pub enum InboundAppendAction<T> {
    Committed,
    Retry(InboundAppendWork<T>),
    Held,
    Stale,
}

#[derive(Debug)]
struct InboundAppendClaim<T> {
    claim_id: String,
    worker_token: Option<String>,
    envelopes: Vec<T>,
    attempt: Option<ChannelInboundAppendAttempt>,
    state: InboundAppendState,
    readiness_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelFailureState {
    ActiveBeforeDispatch,
    PausedBeforeDispatch,
    EnteredRunner,
    RetainedAfterDispatch,
}

/// A snapshot of the queue-owned failure head, never dispatch authority by
/// itself. The last Runner admission compares both identities and the phase.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelFailureWork<T> {
    pub claim_id: String,
    pub worker_token: String,
    pub failures: Vec<T>,
}

/// Finished permits the current drain to continue; it is not a delivery receipt.
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelFailureAction<T> {
    Finished,
    Retry(ChannelFailureWork<T>),
    Held,
    Stale,
}

#[derive(Debug)]
struct ChannelFailureClaim<T> {
    claim_id: String,
    worker_token: Option<String>,
    failures: Vec<T>,
    state: ChannelFailureState,
    readiness_requested: bool,
}

#[derive(Debug, Default)]
pub struct BackgroundWakes<T> {
    closed: bool,
    pub pending_inbound: HashMap<String, Vec<T>>,
    pub reviving_inbound_agent_ids: HashSet<String>,
    inbound_append_claims: HashMap<String, InboundAppendClaim<T>>,
    pub pending_channel_failures: HashMap<String, Vec<T>>,
    pub reviving_channel_failure_agent_ids: HashSet<String>,
    channel_failure_claims: HashMap<String, ChannelFailureClaim<T>>,
    pub pending_event_wakes: HashMap<String, Vec<T>>,
    pub reviving_event_agent_ids: HashSet<String>,
    pub dm_preempted_wake_agent_ids: HashSet<String>,
}

impl<T> BackgroundWakes<T> {
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// This object never reopens. A replacement manager owns a fresh object.
    pub fn close(&mut self) {
        self.closed = true;
        self.pending_inbound.clear();
        self.reviving_inbound_agent_ids.clear();
        self.inbound_append_claims.clear();
        self.pending_channel_failures.clear();
        self.reviving_channel_failure_agent_ids.clear();
        self.channel_failure_claims.clear();
        self.pending_event_wakes.clear();
        self.reviving_event_agent_ids.clear();
        self.dm_preempted_wake_agent_ids.clear();
    }

    pub fn owns_channel_failure_work(&self, agent_id: &str, work: &ChannelFailureWork<T>) -> bool {
        !self.closed && self.channel_failure_claims.get(agent_id).is_some_and(|claim| {
            claim.claim_id == work.claim_id
                && claim.worker_token.as_deref() == Some(work.worker_token.as_str())
                && matches!(claim.state,
                    ChannelFailureState::ActiveBeforeDispatch | ChannelFailureState::EnteredRunner)
        })
    }

    pub fn channel_failure_state(&self, agent_id: &str) -> Option<ChannelFailureState> {
        self.channel_failure_claims.get(agent_id).map(|claim| claim.state)
    }

    pub fn retained_channel_failures(&self, agent_id: &str) -> Option<&[T]> {
        self.channel_failure_claims.get(agent_id).map(|claim| claim.failures.as_slice())
    }

    /// Thread creation failed before this ticket entered Runner. Retain the
    /// exact head for a later signal; even a latched hint cannot retry a launch.
    pub fn release_channel_failure_launch(&mut self, agent_id: &str, work: &ChannelFailureWork<T>) -> bool {
        if !self.owns_channel_failure_work(agent_id, work)
            || self.channel_failure_state(agent_id) != Some(ChannelFailureState::ActiveBeforeDispatch)
        {
            return false;
        }
        let claim = self.channel_failure_claims.get_mut(agent_id).expect("owned channel failure claim");
        claim.state = ChannelFailureState::PausedBeforeDispatch;
        claim.worker_token = None;
        claim.readiness_requested = false;
        self.reviving_channel_failure_agent_ids.remove(agent_id);
        true
    }

    /// The final live-custody check before the caller invokes Runner. No
    /// readiness signal can make an entered ticket eligible for another call.
    pub fn begin_channel_failure_followup(
        &mut self,
        agent_id: &str,
        work: &ChannelFailureWork<T>,
        retained_failures: Vec<T>,
    ) -> bool {
        if !self.owns_channel_failure_work(agent_id, work)
            || self.channel_failure_state(agent_id) != Some(ChannelFailureState::ActiveBeforeDispatch)
        {
            return false;
        }
        let claim = self.channel_failure_claims.get_mut(agent_id).expect("owned channel failure claim");
        claim.failures = retained_failures;
        claim.state = ChannelFailureState::EnteredRunner;
        claim.readiness_requested = false;
        true
    }

    /// An unsuccessful notification does not prove that Runner had no effects.
    /// Retain the head separately from raw input without any automatic replay.
    pub fn retain_channel_failure_after_dispatch(
        &mut self,
        agent_id: &str,
        work: &ChannelFailureWork<T>,
    ) -> bool {
        if !self.owns_channel_failure_work(agent_id, work)
            || self.channel_failure_state(agent_id) != Some(ChannelFailureState::EnteredRunner)
        {
            return false;
        }
        let claim = self.channel_failure_claims.get_mut(agent_id).expect("owned channel failure claim");
        claim.state = ChannelFailureState::RetainedAfterDispatch;
        claim.worker_token = None;
        claim.readiness_requested = false;
        self.reviving_channel_failure_agent_ids.remove(agent_id);
        true
    }

    /// The caller owns the definite no-work or successful-notification decision.
    /// Keep revival until the same drain makes its next atomic take/idle check.
    pub fn finish_channel_failure_work(&mut self, agent_id: &str, work: &ChannelFailureWork<T>) -> bool {
        if !self.owns_channel_failure_work(agent_id, work) {
            return false;
        }
        self.channel_failure_claims.remove(agent_id);
        true
    }

    pub fn owns_inbound_append(&self, agent_id: &str, work: &InboundAppendWork<T>) -> bool {
        !self.closed && self.inbound_append_claims.get(agent_id).is_some_and(|claim| {
            claim.claim_id == work.claim_id
                && claim.worker_token.as_deref() == Some(work.worker_token.as_str())
                && claim.state == InboundAppendState::Active
        })
    }

    pub fn inbound_append_state(&self, agent_id: &str) -> Option<InboundAppendState> {
        self.inbound_append_claims.get(agent_id).map(|claim| claim.state)
    }

    pub fn inbound_append_attempt(&self, agent_id: &str) -> Option<&ChannelInboundAppendAttempt> {
        self.inbound_append_claims.get(agent_id)?.attempt.as_ref()
    }

    /// A failed launch never ran this ticket. Preserve the head for a later
    /// actual signal, without recursively retrying a failed thread creation.
    pub fn release_inbound_launch(&mut self, agent_id: &str, work: &InboundAppendWork<T>) -> bool {
        if !self.owns_inbound_append(agent_id, work) {
            return false;
        }
        let claim = self.inbound_append_claims.get_mut(agent_id).expect("owned inbound claim");
        claim.state = InboundAppendState::NotWritten;
        claim.worker_token = None;
        claim.readiness_requested = false;
        self.reviving_inbound_agent_ids.remove(agent_id);
        true
    }

    /// The confirmed batch has finished the existing Runner path. Keep revival
    /// until the next atomic take/idle decision, so later input cannot overtake.
    pub fn finish_inbound_append(&mut self, agent_id: &str, work: &InboundAppendWork<T>) -> bool {
        self.finish_inbound_append_with_redrive(agent_id, work, None)
    }

    /// Retain a displayed continuation ahead of raw input without adopting a
    /// newer Session generation. The caller owns the existing redrive policy.
    pub fn finish_inbound_append_with_redrive(
        &mut self,
        agent_id: &str,
        work: &InboundAppendWork<T>,
        redrive: Option<Vec<T>>,
    ) -> bool {
        if self.closed {
            return false;
        }
        let Some(claim) = self.inbound_append_claims.get(agent_id) else {
            return false;
        };
        if claim.claim_id != work.claim_id
            || claim.worker_token.as_deref() != Some(work.worker_token.as_str())
            || claim.state != InboundAppendState::Committed
        {
            return false;
        }
        if let Some(envelopes) = redrive.filter(|items| !items.is_empty()) {
            let Some(source) = claim.attempt.as_ref().filter(|attempt| {
                attempt.entries.is_some()
                    && !attempt.session_key.trim().is_empty()
                    && !attempt.session_id.trim().is_empty()
                    && !attempt.lifecycle_revision.trim().is_empty()
            }) else {
                // The committed head remains held if its original binding or
                // preparation is unavailable; a continuation may not rebind.
                return false;
            };
            let continuation = InboundAppendClaim {
                claim_id: uuid::Uuid::new_v4().to_string(),
                worker_token: None,
                envelopes,
                attempt: Some(ChannelInboundAppendAttempt {
                    session_key: source.session_key.clone(),
                    session_id: source.session_id.clone(),
                    lifecycle_revision: source.lifecycle_revision.clone(),
                    entries: Some(Vec::new()),
                }),
                state: InboundAppendState::NotWritten,
                readiness_requested: false,
            };
            self.inbound_append_claims.insert(agent_id.to_string(), continuation);
        } else {
            self.inbound_append_claims.remove(agent_id);
        }
        true
    }

    pub fn enqueue(map: &mut HashMap<String, Vec<T>>, agent_id: impl Into<String>, item: T) {
        map.entry(agent_id.into()).or_default().push(item);
    }

    pub fn begin_revival(set: &mut HashSet<String>, agent_id: &str) -> bool {
        set.insert(agent_id.to_string())
    }

    pub fn end_revival(set: &mut HashSet<String>, agent_id: &str) {
        set.remove(agent_id);
    }

    pub fn take_pending(map: &mut HashMap<String, Vec<T>>, agent_id: &str) -> Vec<T> {
        map.remove(agent_id).unwrap_or_default()
    }

    /// Claim the next inbound batch or release this worker's revival ownership.
    /// Call under the same lock as enqueue. After `None`, the worker must return
    /// without clearing revival again: a successor may already own it.
    pub fn take_pending_inbound_or_end_revival(&mut self, agent_id: &str) -> Option<Vec<T>> {
        Self::take_pending_or_end_revival(
            self.closed,
            &mut self.pending_inbound,
            &mut self.reviving_inbound_agent_ids,
            agent_id,
        )
    }

    /// Claim the next delivery-failure batch or release idle revival ownership.
    /// Call under the same lock as enqueue. After `None`, the worker must return
    /// without clearing revival again: a successor may already own it.
    pub fn take_pending_channel_failures_or_end_revival(
        &mut self,
        agent_id: &str,
    ) -> Option<Vec<T>> {
        // This raw-batch compatibility helper cannot bypass a retained head or
        // clear the revival bit belonging to its active ticket.
        if self.channel_failure_claims.contains_key(agent_id) {
            return None;
        }
        Self::take_pending_or_end_revival(
            self.closed,
            &mut self.pending_channel_failures,
            &mut self.reviving_channel_failure_agent_ids,
            agent_id,
        )
    }

    fn take_pending_or_end_revival(
        closed: bool,
        pending: &mut HashMap<String, Vec<T>>,
        reviving: &mut HashSet<String>,
        agent_id: &str,
    ) -> Option<Vec<T>> {
        if closed {
            return None;
        }
        match pending.remove(agent_id) {
            Some(pending) if !pending.is_empty() => Some(pending),
            _ => {
                reviving.remove(agent_id);
                None
            }
        }
    }
}

impl<T: Clone> BackgroundWakes<T> {
    /// Reserve the real failure head before launch. New failures signal only a
    /// pre-dispatch head and always remain behind an existing retained head.
    pub fn enqueue_channel_failure_and_claim(
        &mut self,
        agent_id: &str,
        failure: T,
    ) -> Result<Option<ChannelFailureWork<T>>, String> {
        if self.closed {
            return Err("Background wakes are closed".to_string());
        }
        Self::enqueue(&mut self.pending_channel_failures, agent_id, failure);
        Ok(self.request_channel_failure_readiness_for_agent(agent_id))
    }

    pub fn request_channel_failure_readiness(&mut self) -> Vec<(String, ChannelFailureWork<T>)> {
        if self.closed {
            return Vec::new();
        }
        let agent_ids = self.channel_failure_claims.keys()
            .chain(self.pending_channel_failures.keys()).cloned().collect::<HashSet<_>>();
        agent_ids.into_iter().filter_map(|agent_id| {
            self.request_channel_failure_readiness_for_agent(&agent_id).map(|work| (agent_id, work))
        }).collect()
    }

    pub fn request_channel_failure_readiness_for_agent(&mut self, agent_id: &str) -> Option<ChannelFailureWork<T>> {
        if self.closed {
            return None;
        }
        if let Some(claim) = self.channel_failure_claims.get_mut(agent_id) {
            match claim.state {
                ChannelFailureState::ActiveBeforeDispatch => {
                    claim.readiness_requested = true;
                    return None;
                }
                ChannelFailureState::EnteredRunner | ChannelFailureState::RetainedAfterDispatch => return None,
                ChannelFailureState::PausedBeforeDispatch => {}
            }
        }
        // A finishing worker retains its drain through the finish-to-take gap.
        if !Self::begin_revival(&mut self.reviving_channel_failure_agent_ids, agent_id) {
            return None;
        }
        self.take_channel_failure_work_or_end_revival(agent_id)
    }

    /// Save the latest provenance-filtered failures before consuming the one
    /// coalesced hint or atomically releasing this pre-dispatch acquisition.
    pub fn pause_channel_failure_before_dispatch(
        &mut self,
        agent_id: &str,
        work: &ChannelFailureWork<T>,
        retained_failures: Vec<T>,
    ) -> ChannelFailureAction<T> {
        if !self.owns_channel_failure_work(agent_id, work)
            || self.channel_failure_state(agent_id) != Some(ChannelFailureState::ActiveBeforeDispatch)
        {
            return ChannelFailureAction::Stale;
        }
        let claim = self.channel_failure_claims.get_mut(agent_id).expect("owned channel failure claim");
        claim.failures = retained_failures;
        if claim.readiness_requested {
            claim.readiness_requested = false;
            let worker_token = uuid::Uuid::new_v4().to_string();
            claim.worker_token = Some(worker_token.clone());
            ChannelFailureAction::Retry(ChannelFailureWork {
                claim_id: claim.claim_id.clone(),
                worker_token,
                failures: claim.failures.clone(),
            })
        } else {
            claim.state = ChannelFailureState::PausedBeforeDispatch;
            claim.worker_token = None;
            self.reviving_channel_failure_agent_ids.remove(agent_id);
            ChannelFailureAction::Held
        }
    }

    /// Acquire one failure head or atomically release an idle drain. A duplicate
    /// take never clears another ticket, and entered/retained heads never retry.
    pub fn take_channel_failure_work_or_end_revival(&mut self, agent_id: &str) -> Option<ChannelFailureWork<T>> {
        if self.closed || !self.reviving_channel_failure_agent_ids.contains(agent_id) {
            return None;
        }
        if let Some(claim) = self.channel_failure_claims.get_mut(agent_id) {
            if claim.worker_token.is_some() {
                return None;
            }
            if claim.state != ChannelFailureState::PausedBeforeDispatch {
                self.reviving_channel_failure_agent_ids.remove(agent_id);
                return None;
            }
            claim.worker_token = Some(uuid::Uuid::new_v4().to_string());
            claim.state = ChannelFailureState::ActiveBeforeDispatch;
            claim.readiness_requested = false;
        } else {
            let Some(failures) = self.pending_channel_failures.remove(agent_id).filter(|items| !items.is_empty()) else {
                self.reviving_channel_failure_agent_ids.remove(agent_id);
                return None;
            };
            self.channel_failure_claims.insert(agent_id.to_string(), ChannelFailureClaim {
                claim_id: uuid::Uuid::new_v4().to_string(),
                worker_token: Some(uuid::Uuid::new_v4().to_string()),
                failures,
                state: ChannelFailureState::ActiveBeforeDispatch,
                readiness_requested: false,
            });
        }
        let claim = self.channel_failure_claims.get(agent_id)?;
        Some(ChannelFailureWork {
            claim_id: claim.claim_id.clone(),
            worker_token: claim.worker_token.clone()?,
            failures: claim.failures.clone(),
        })
    }

    /// Reserve the real head/token before the caller attempts to spawn. New
    /// input is also a readiness hint, but can never bypass an Unknown head.
    pub fn enqueue_inbound_and_claim(
        &mut self,
        agent_id: &str,
        envelope: T,
    ) -> Result<Option<InboundAppendWork<T>>, String> {
        if self.closed {
            return Err("Background wakes are closed".to_string());
        }
        Self::enqueue(&mut self.pending_inbound, agent_id, envelope);
        Ok(self.request_inbound_readiness_for_agent(agent_id))
    }

    /// Coalesce active heads and claim idle retained heads as well as raw input.
    /// Returned tickets contain snapshots; custody remains in this claim map.
    pub fn request_inbound_readiness(&mut self) -> Vec<(String, InboundAppendWork<T>)> {
        if self.closed {
            return Vec::new();
        }
        let agent_ids = self.inbound_append_claims.keys()
            .chain(self.pending_inbound.keys()).cloned().collect::<HashSet<_>>();
        agent_ids.into_iter().filter_map(|agent_id| {
            self.request_inbound_readiness_for_agent(&agent_id).map(|work| (agent_id, work))
        }).collect()
    }

    fn request_inbound_readiness_for_agent(&mut self, agent_id: &str) -> Option<InboundAppendWork<T>> {
        if let Some(claim) = self.inbound_append_claims.get_mut(agent_id) {
            match claim.state {
                InboundAppendState::Active => {
                    claim.readiness_requested = true;
                    return None;
                }
                InboundAppendState::Unknown | InboundAppendState::Committed => return None,
                InboundAppendState::NotWritten => {}
            }
        }
        // A committed worker may be between finish and its next take, including
        // a retained displayed redrive. That worker still owns the drain.
        if !Self::begin_revival(&mut self.reviving_inbound_agent_ids, agent_id) {
            return None;
        }
        self.take_inbound_append_or_end_revival(agent_id)
    }

    /// Atomically consume a readiness request or relinquish this exact worker.
    pub fn pause_inbound_append(
        &mut self,
        agent_id: &str,
        work: &InboundAppendWork<T>,
    ) -> InboundAppendAction<T> {
        if !self.owns_inbound_append(agent_id, work) {
            return InboundAppendAction::Stale;
        }
        self.hold_or_retry_inbound(agent_id)
    }

    /// Only Committed permits Runner dispatch. A confirmed rollback can consume
    /// a readiness request, using the attempt just stored by this settlement.
    pub fn record_inbound_append_outcome(
        &mut self,
        agent_id: &str,
        work: &InboundAppendWork<T>,
        attempt: Option<ChannelInboundAppendAttempt>,
        outcome: &ChannelInboundAppendOutcome,
    ) -> InboundAppendAction<T> {
        if !self.owns_inbound_append(agent_id, work) {
            return InboundAppendAction::Stale;
        }
        let claim = self.inbound_append_claims.get_mut(agent_id).expect("owned inbound claim");
        claim.attempt = attempt;
        match outcome {
            ChannelInboundAppendOutcome::NotWritten(_) => self.hold_or_retry_inbound(agent_id),
            ChannelInboundAppendOutcome::Committed { .. } => {
                claim.state = InboundAppendState::Committed;
                claim.readiness_requested = false;
                InboundAppendAction::Committed
            }
            ChannelInboundAppendOutcome::Unknown(_) => {
                claim.state = InboundAppendState::Unknown;
                claim.readiness_requested = false;
                claim.worker_token = None;
                self.reviving_inbound_agent_ids.remove(agent_id);
                InboundAppendAction::Held
            }
        }
    }

    fn hold_or_retry_inbound(&mut self, agent_id: &str) -> InboundAppendAction<T> {
        let claim = self.inbound_append_claims.get_mut(agent_id).expect("owned inbound claim");
        if claim.readiness_requested {
            claim.readiness_requested = false;
            let worker_token = uuid::Uuid::new_v4().to_string();
            claim.worker_token = Some(worker_token.clone());
            claim.state = InboundAppendState::Active;
            InboundAppendAction::Retry(InboundAppendWork {
                claim_id: claim.claim_id.clone(),
                worker_token,
                envelopes: claim.envelopes.clone(),
                attempt: claim.attempt.clone(),
            })
        } else {
            claim.state = InboundAppendState::NotWritten;
            claim.worker_token = None;
            self.reviving_inbound_agent_ids.remove(agent_id);
            InboundAppendAction::Held
        }
    }

    /// Called under the same mutex as ingress. Preparation is never merged back
    /// into raw pending input, and Unknown can never be automatically retried.
    pub fn take_inbound_append_or_end_revival(&mut self, agent_id: &str) -> Option<InboundAppendWork<T>> {
        if self.closed || !self.reviving_inbound_agent_ids.contains(agent_id) {
            return None;
        }
        if let Some(claim) = self.inbound_append_claims.get_mut(agent_id) {
            if claim.worker_token.is_some() {
                // Another worker still owns this head. In particular, do not
                // clear its revival bit on a duplicate or delayed acquisition.
                return None;
            }
            if claim.state != InboundAppendState::NotWritten {
                self.reviving_inbound_agent_ids.remove(agent_id);
                return None;
            }
            claim.worker_token = Some(uuid::Uuid::new_v4().to_string());
            claim.state = InboundAppendState::Active;
            claim.readiness_requested = false;
        } else {
            let Some(envelopes) = self.pending_inbound.remove(agent_id).filter(|items| !items.is_empty()) else {
                self.reviving_inbound_agent_ids.remove(agent_id);
                return None;
            };
            self.inbound_append_claims.insert(agent_id.to_string(), InboundAppendClaim {
                claim_id: uuid::Uuid::new_v4().to_string(),
                worker_token: Some(uuid::Uuid::new_v4().to_string()),
                envelopes,
                attempt: None,
                state: InboundAppendState::Active,
                readiness_requested: false,
            });
        }
        let claim = self.inbound_append_claims.get(agent_id)?;
        Some(InboundAppendWork {
            claim_id: claim.claim_id.clone(),
            worker_token: claim.worker_token.clone()?,
            envelopes: claim.envelopes.clone(),
            attempt: claim.attempt.clone(),
        })
    }
}
