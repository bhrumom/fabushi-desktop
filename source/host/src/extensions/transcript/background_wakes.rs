use std::collections::{HashMap, HashSet};

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

    pub fn take_pending(
        map: &mut HashMap<String, Vec<T>>,
        agent_id: &str,
    ) -> Vec<T> {
        map.remove(agent_id).unwrap_or_default()
    }
}
