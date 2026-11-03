use mahayana_host_runtime::extensions::transcript::production_runtime::{
    ProductionSendError, ProductionTranscriptRuntime, classify_send_dispatch,
};
use mahayana_host_runtime::extensions::transcript::run_scheduler::RunLane;
use mahayana_host_runtime::extensions::transcript::send_pipeline::PersistedSendContext;
use serde_json::json;

#[test]
fn dispatch_source_preserves_connector_identity_without_changing_lane_policy() {
    for (args, lane, source) in [
        (json!({}), RunLane::User, "turn"),
        (json!({"requestSource":"turn"}), RunLane::User, "turn"),
        (json!({"requestSource":"connector"}), RunLane::User, "connector"),
        (json!({"requestSource":" connector ","ackRedrive":true}), RunLane::User, "connector"),
        (json!({"requestSource":"handoff-resume"}), RunLane::Background, "handoff-resume"),
        (json!({"requestSource":"handoff-resume","ackRedrive":true}), RunLane::Background, "ack-redrive"),
        (json!({"requestSource":"automation"}), RunLane::Background, "automation"),
        (json!({"requestSource":"upgrade-resume"}), RunLane::Background, "upgrade-resume"),
        (json!({"requestSource":"event"}), RunLane::Background, "event"),
        (json!({"requestSource":"agent-inbound"}), RunLane::Agent, "agent-inbound"),
        (json!({"requestSource":"human-handoff"}), RunLane::User, "human-handoff"),
        (json!({"requestSource":"group-member"}), RunLane::User, "group-member"),
        (json!({"requestSource":"kickstart"}), RunLane::User, "kickstart"),
        (json!({"requestSource":"unknown","ackRedrive":true}), RunLane::User, "turn"),
        (json!({"requestSource":"  "}), RunLane::User, "turn"),
    ] {
        assert_eq!(classify_send_dispatch(&args), Ok((lane, source)), "{args}");
    }
    assert!(matches!(
        classify_send_dispatch(&json!({"requestSource":"connector","ackRedrive":"true"})),
        Err(ProductionSendError::BadRequest(_))
    ));
}

#[test]
fn routed_connector_source_reaches_the_active_lease_and_retires_with_it() {
    let runtime = ProductionTranscriptRuntime::new(None);
    let accepted = runtime.accept_routed_send(
        &json!({
            "agentId":"connector-agent", "prompt":"respond to the channel message",
            "clientNonce":"connector-nonce", "streamId":"connector-stream",
            "requestSource":"connector", "hidden":true, "appendUserMessage":false,
        }),
        |_| Ok(PersistedSendContext::default()),
    ).expect("routed connector acceptance");
    assert!(!accepted.duplicate);
    assert_eq!(runtime.active_turn_source("connector-agent").as_deref(), Some("connector"));
    assert_eq!(runtime.active_turn_lane("connector-agent"), Some(RunLane::User));
    assert!(runtime.has_routed_turn_lease("connector-agent", "connector-stream"));
    assert!(runtime.is_agent_running("connector-agent"));
    assert!(runtime.settle_routed_turn("connector-agent", "connector-stream", 1)
        .expect("settle exact lease").is_some());
    assert!(runtime.active_turn_source("connector-agent").is_none());
    assert!(runtime.active_turn_lane("connector-agent").is_none());
    assert!(!runtime.has_routed_turn_lease("connector-agent", "connector-stream"));
    assert!(!runtime.is_agent_running("connector-agent"));
}

#[test]
fn executed_connector_source_is_visible_to_the_actual_dispatch_callback() {
    let runtime = ProductionTranscriptRuntime::new(None);
    let result = runtime.execute_send(
        &json!({
            "agentId":"executed-connector", "prompt":"continue the channel turn",
            "clientNonce":"executed-connector-nonce", "requestSource":"connector",
        }),
        || {
            assert_eq!(runtime.active_turn_source("executed-connector").as_deref(), Some("connector"));
            assert_eq!(runtime.active_turn_lane("executed-connector"), Some(RunLane::User));
            assert!(runtime.is_agent_running("executed-connector"));
            Ok(json!({"accepted":true,"operationId":"connector-execution"}))
        },
        |_| Ok(PersistedSendContext::default()),
    ).expect("execute connector");
    assert_eq!(result["operationId"], "connector-execution");
    assert!(runtime.is_turn_dispatch_idle("executed-connector"));
    assert!(runtime.active_turn_source("executed-connector").is_none());
    assert!(!runtime.is_agent_running("executed-connector"));
}
