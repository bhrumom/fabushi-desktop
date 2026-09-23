use mahayana_host_runtime::gateway_protocol::{
    GATEWAY_PREPARE_UPGRADE_PATH, is_grok_gateway_command, parse_command_args,
    slim_command_result, slim_event,
};
use serde_json::json;

#[test]
fn gateway_protocol_parses_empty_and_json_command_args() {
    assert_eq!(parse_command_args(b"").expect("empty body"), json!({}));
    assert_eq!(
        parse_command_args(br#"{"agentId":"agent-1","prompt":"hello"}"#)
            .expect("json body"),
        json!({"agentId":"agent-1","prompt":"hello"})
    );
    assert!(parse_command_args(b"{").is_err());
}

#[test]
fn gateway_protocol_keeps_frozen_command_inventory_and_upgrade_path() {
    assert!(is_grok_gateway_command("sendPrompt"));
    assert!(is_grok_gateway_command("requestWebAuthnCeremony"));
    assert!(is_grok_gateway_command("executeRoutedMcpTool"));
    assert!(!is_grok_gateway_command("feature.auth.status"));
    assert!(!is_grok_gateway_command("not-a-grok-command"));
    assert_eq!(GATEWAY_PREPARE_UPGRADE_PATH, "/prepare-upgrade");
}

#[test]
fn gateway_protocol_strips_inline_avatars_from_slim_commands() {
    let listed = slim_command_result(
        "listAgents",
        json!([
            {"id":"one","avatarDataUrl":"data:image/png;base64,one"},
            {"id":"two","avatarDataUrl":null}
        ]),
    );
    assert_eq!(listed[0]["avatarDataUrl"], json!(null));
    assert_eq!(listed[1]["avatarDataUrl"], json!(null));

    let created = slim_command_result(
        "createAgent",
        json!({
            "agent":{"id":"one","avatarDataUrl":"data:image/png;base64,one"},
            "other":"preserved"
        }),
    );
    assert_eq!(created["agent"]["avatarDataUrl"], json!(null));
    assert_eq!(created["other"], "preserved");

    let untouched = slim_command_result(
        "getTranscript",
        json!({"avatarDataUrl":"keep-on-non-summary-result"}),
    );
    assert_eq!(untouched["avatarDataUrl"], "keep-on-non-summary-result");
}

#[test]
fn gateway_protocol_strips_inline_avatars_from_agent_events_only() {
    let agents = slim_event(json!({
        "channel":"agents",
        "payload":{
            "agents":[
                {"id":"one","avatarDataUrl":"data:image/png;base64,one"}
            ],
            "activeAgentId":"one"
        }
    }));
    assert_eq!(agents["payload"]["agents"][0]["avatarDataUrl"], json!(null));
    assert_eq!(agents["payload"]["activeAgentId"], "one");

    let upserted = slim_event(json!({
        "channel":"agent-upserted",
        "payload":{
            "agent":{"id":"one","avatarDataUrl":"data:image/png;base64,one"}
        }
    }));
    assert_eq!(upserted["payload"]["agent"]["avatarDataUrl"], json!(null));

    let other = slim_event(json!({
        "channel":"transcript",
        "payload":{"avatarDataUrl":"preserve"}
    }));
    assert_eq!(other["payload"]["avatarDataUrl"], "preserve");
}
