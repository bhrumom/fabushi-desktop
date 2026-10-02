const OWNER: &str = include_str!("../src/host_runner_composition.rs");
const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

#[test]
fn host_runner_composition_owns_turn_decoration_order() {
    for needle in [
        "pub struct ProductionTurnCompositionHooks",
        "pub fn compose_production_turn(",
        "create_production_runner_composition(input)",
        ".with_agent_management_sink(hooks.agent_management_sink)",
        ".with_state_writer(hooks.state_writer)",
        ".with_routine_auto_review(hooks.routine_auto_review)",
        ".with_box_shell_review(hooks.box_shell_review)",
        "composition.with_subagent_task_sink(subagent_task_sink)",
        "composition.with_subagent_management(",
        "composition.with_routine_post_write(routine_post_write)",
        "composition.with_multitask_todo_state(multitask_todo_state)",
    ] {
        assert!(
            OWNER.contains(needle),
            "HostRunnerComposition missing production ownership: {needle}"
        );
    }
}

#[test]
fn shipping_host_has_one_runner_composition_entrypoint() {
    assert!(
        SHIPPING_HOST.contains("host_runner_composition.compose_production_turn("),
        "shipping Host must delegate turn assembly to HostRunnerComposition",
    );
    assert!(
        !SHIPPING_HOST.contains("create_production_runner_composition("),
        "shipping Host must not construct a parallel Runner composition",
    );
    for needle in [
        ".with_agent_management_sink(agent_management_sink)",
        ".with_state_writer(state_writer)",
        ".with_routine_auto_review(routine_auto_review)",
        ".with_box_shell_review(box_shell_review)",
        "composition.with_subagent_task_sink(",
        "composition.with_subagent_management(",
        "composition.with_routine_post_write(",
        "composition.with_multitask_todo_state(",
    ] {
        assert!(
            !SHIPPING_HOST.contains(needle),
            "shipping Host still owns Runner decoration outside HostRunnerComposition: {needle}",
        );
    }
}

#[test]
fn host_runner_composition_owns_transcript_checkpoint_wiring() {
    for needle in [
        "pub fn compose_production_checkpoint_sink(",
        "sessions.open_agent_store_owner(agent_id)?",
        "sessions.create_agent_blob_store(agent_id)?",
        "ProductionTranscriptMirrorProvider::with_reporter(",
        "GeneratedTranscriptOccurrenceCodec::new(RejectGeneratedToolJsonProjection)",
        "transcript_provider.route_for_session(",
        "ProductionAgentStateCheckpointSink::new(",
    ] {
        assert!(
            OWNER.contains(needle),
            "HostRunnerComposition missing checkpoint composition responsibility: {needle}"
        );
    }
    assert!(
        SHIPPING_HOST.contains(".compose_production_checkpoint_sink("),
        "shipping Host must delegate transcript/checkpoint composition",
    );
    for needle in [
        "ProductionTranscriptMirrorProvider::with_reporter(",
        "GeneratedTranscriptOccurrenceCodec::new(",
        "ProductionAgentStateCheckpointSink::new(",
        ".open_agent_store_owner(&agent_id)",
        ".create_agent_blob_store(&agent_id)",
    ] {
        assert!(
            !SHIPPING_HOST.contains(needle),
            "shipping Host still owns transcript/checkpoint wiring: {needle}",
        );
    }
}

#[test]
fn host_runner_composition_owns_turn_state_surface_wiring() {
    for needle in [
        "pub struct ProductionTurnStateSurfaces",
        "pub fn compose_turn_state_surfaces(",
        ".memory_service()",
        ".agents_root_dir()",
        "SandAgentState::new(sand_root, agent_id.to_string())",
        "sessions.open_agent_db_owner(agent_id)?",
    ] {
        assert!(
            OWNER.contains(needle),
            "HostRunnerComposition missing state-surface responsibility: {needle}"
        );
    }
    assert!(
        SHIPPING_HOST.contains(
            ".compose_turn_state_surfaces(&session_workers, &agent_id, multitask_enabled)"
        ),
        "shipping Host must consume HostRunnerComposition state surfaces",
    );
    for needle in [
        "SandAgentState::new(state_sand_root",
        "could not open production multitask todo state for {agent_id}",
        "production memory agents root has no sand root parent",
    ] {
        assert!(
            !SHIPPING_HOST.contains(needle),
            "shipping Host still owns turn state-surface composition: {needle}",
        );
    }
}

#[test]
fn host_runner_composition_owns_shipping_runner_construction() {
    for needle in [
        "pub fn compose_production_runner(",
        "ProductionTurnAgentOwner::new(composition)",
        ".with_agent_state_checkpoint_sink(checkpoint_sink)",
        ".with_upgrade_quiesce_signal(upgrade_quiesce_signal)",
        "SandAgentRunner::new(owner).with_generated_agent_runtime(generated_agent_runtime)",
    ] {
        assert!(
            OWNER.contains(needle),
            "HostRunnerComposition missing Runner construction responsibility: {needle}"
        );
    }
    assert!(
        SHIPPING_HOST.contains("host_runner_composition.compose_production_runner("),
        "shipping Host must delegate Runner facade construction",
    );
    for needle in [
        "ProductionTurnAgentOwner::new(composition)",
        "SandAgentRunner::new(owner)",
    ] {
        assert!(
            !SHIPPING_HOST.contains(needle),
            "shipping Host still constructs the Runner owner path directly: {needle}",
        );
    }
}
