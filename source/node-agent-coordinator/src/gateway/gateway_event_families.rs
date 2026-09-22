pub const SSE_CHANNEL_BY_FAMILY: &[(&str, &str)] = &[
    ("transcript", "transcript"),
    ("client-side-tool-v2", "client-side-tool-v2"),
    ("agents", "agents"),
    ("agent-upserted", "agent-upserted"),
    ("tray", "tray"),
    ("agents-workflow", "workflows"),
    ("subagents", "subagents"),
    ("async-tasks", "async-tasks"),
    ("agents-automation", "automations"),
    ("mcp-servers-updated", "mcp-servers"),
    ("forever-box", "forever-box"),
    ("teach-recording", "teach-recording"),
    ("box-disk-pressure", "box-disk-pressure"),
    ("computer-action", "computer-action"),
    ("outline", "outline"),
    ("sharing", "sharing"),
    ("host-settings", "host-settings"),
];

pub fn sse_channel_for_family(family: &str) -> Option<&'static str> {
    SSE_CHANNEL_BY_FAMILY
        .iter()
        .find_map(|(candidate, channel)| (*candidate == family).then_some(*channel))
}

pub fn coordinator_event_family_for_sse_channel(channel: &str) -> Option<&'static str> {
    SSE_CHANNEL_BY_FAMILY
        .iter()
        .find_map(|(family, candidate)| (*candidate == channel).then_some(*family))
}

pub fn is_known(family: &str) -> bool {
    sse_channel_for_family(family).is_some()
}
