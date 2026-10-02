use std::collections::{HashMap, HashSet};

use serde_json::Value;

pub const TIMELINE_EVENT_WAKE_CUE: &str = "[event]";

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

#[derive(Debug, Default)]
pub struct BackgroundWakes<T> {
    pub pending_inbound: HashMap<String, Vec<T>>,
    pub reviving_inbound_agent_ids: HashSet<String>,
    pub pending_channel_failures: HashMap<String, Vec<T>>,
    pub reviving_channel_failure_agent_ids: HashSet<String>,
    pub pending_event_wakes: HashMap<String, Vec<T>>,
    pub reviving_event_agent_ids: HashSet<String>,
    pub dm_preempted_wake_agent_ids: HashSet<String>,
}

impl<T> BackgroundWakes<T> {
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
}
