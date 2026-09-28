const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
const INFERENCE_PRODUCTION: &str = include_str!("../src/extensions/inference/production.rs");
const TRANSCRIPT_RUNTIME: &str = include_str!("../src/extensions/transcript/production_runtime.rs");

#[test]
fn shipping_host_wires_frozen_turn_telemetry_sources() {
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
        "ProviderTokenUsage",
        "merge_provider_token_usage",
        "usage_sink: Some(usage_sink)",
        "worker_provider_usage",
        "usage_inference.record_usage(",
        "InferenceUsage {",
        "usage_provider",
        "TelemetryTokenUsage",
        "GatewayCommandContext",
        "call_with_context(",
        "context.trace_id.clone()",
        "context.span_id.clone()",
        "dispatch_started.elapsed().as_secs_f64()",
        "observation.set_first_token_handler(",
        "ttft_telemetry(&TtftFields",
        "reason: \"agent_deleted\".into()",
        "let was_in_flight = transcript_runtime.is_agent_running(agent_id)",
        "runner_registry.cancel_agent(agent_id, \"agent deleted\") > 0",
        "self.runner_registry.agent_id_for_stream(stream_id)",
        "reason.to_ascii_lowercase().contains(\"superseded\")",
        "reason: \"superseded\".into()",
        "had_active_run: cancelled",
    ] {
        assert!(
            SHIPPING_HOST.contains(needle),
            "shipping Host must preserve frozen turn telemetry wiring: {needle}"
        );
    }

    for needle in [
        "self.settings.record_inference_usage(",
        "provider.as_str()",
        "inference_usage_persist_failed",
    ] {
        assert!(
            INFERENCE_PRODUCTION.contains(needle),
            "Host Inference owner must preserve routed-provider usage persistence: {needle}"
        );
    }

    assert!(
        TRANSCRIPT_RUNTIME.contains("state.lifecycle.track_turn_request_id(agent_id, operation_id)"),
        "Transcript runtime must own per-turn request-id tracking"
    );
}
