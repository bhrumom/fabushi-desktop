const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

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
    ] {
        assert!(
            SHIPPING_HOST.contains(needle),
            "shipping Host must preserve frozen turn telemetry wiring: {needle}"
        );
    }
}
