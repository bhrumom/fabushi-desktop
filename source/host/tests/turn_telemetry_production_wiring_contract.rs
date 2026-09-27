const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
const TRANSCRIPT_RUNTIME: &str = include_str!("../src/extensions/transcript/production_runtime.rs");

#[test]
fn shipping_host_wires_user_message_and_watchdog_interrupt_turn_telemetry() {
    for needle in [
        "user_message_received_telemetry(",
        "send_was_in_flight",
        "self.transcript_runtime.is_agent_running(agent_id)",
        "turn_interrupt_telemetry(&TurnInterruptFields",
        "reason: \"watchdog\".into()",
        "watchdog_transcript_runtime.is_agent_running(&event.agent_id)",
        "interrupt_wedged_run_for_watchdog(&event.agent_id)",
        "worker_transcript_runtime.settle_turn_usage(",
        "turn_usage_telemetry(&TurnUsageFields",
        "observation.set_first_token_handler(",
        "ttft_telemetry(&TtftFields",
        "reason: \"agent_deleted\".into()",
        "let was_in_flight = transcript_runtime.is_agent_running(agent_id)",
        "runner_registry.cancel_agent(agent_id, \"agent deleted\") > 0",
    ] {
        assert!(
            SHIPPING_HOST.contains(needle),
            "shipping Host must preserve frozen turn telemetry wiring: {needle}"
        );
    }

    assert!(
        TRANSCRIPT_RUNTIME.contains("state.lifecycle.track_turn_request_id(agent_id, operation_id)"),
        "Transcript runtime must own per-turn request-id tracking"
    );
}
