const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
const SESSION_GATEWAY: &str = include_str!("../src/extensions/session/gateway.rs");
const TRANSCRIPT_RUNTIME: &str = include_str!("../src/extensions/transcript/production_runtime.rs");
const SEND_ACCEPTANCE: &str = include_str!("../src/extensions/transcript/send_acceptance.rs");

#[test]
fn shipping_send_acceptance_projects_durable_echoes_after_unlock_and_before_dispatch() {
    for needle in [
        "accepted_echoes: staged",
        "acceptance_effects_applied: true",
        "accepted_durably: true",
        "needs_roster_refresh",
    ] {
        assert!(SESSION_GATEWAY.contains(needle), "missing durable acceptance projection: {needle}");
    }

    let unlock = TRANSCRIPT_RUNTIME.find("drop(state);\n        on_accepted(&persisted_send_context);")
        .expect("acceptance observer must run after releasing Transcript runtime state");
    let dispatch = TRANSCRIPT_RUNTIME.find("dispatch()")
        .expect("shipping provider dispatch");
    assert!(unlock < dispatch, "durable acceptance projection must precede provider dispatch");

    for needle in [
        "execute_send_with_acceptance_observer(",
        "mark_accepted_echoes_on_active_transcript",
        "emit_persisted_send_acceptance(",
        "self.trays.clear_for_agent(agent_id)",
        "emit_accepted_send_echoes(",
        "roster_emit: Arc<ProductionRosterEmit>",
    ] {
        assert!(SHIPPING_HOST.contains(needle), "shipping Host missing send-acceptance owner wiring: {needle}");
    }

    for needle in [
        "\"channel\": \"transcript\"",
        "\"type\": \"appended\"",
        "direct_addressed_acceptance",
        "roster.emit_agent_update(agent_id)",
        "has_offscreen_entries",
    ] {
        assert!(SEND_ACCEPTANCE.contains(needle), "missing frozen accepted-echo behavior: {needle}");
    }
}
