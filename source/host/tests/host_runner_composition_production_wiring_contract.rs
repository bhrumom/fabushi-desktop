const OWNER: &str = include_str!("../src/host_runner_composition.rs");
const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
const GENERATED_STREAM: &str = include_str!("../src/runner/generated_agent_turn_stream.rs");
const TURN_OWNER: &str = include_str!("../src/runner/production_turn_agent_owner.rs");

#[test]
fn host_runner_composition_owns_turn_decoration_order() {
    for needle in [
        "ProductionRunnerCompositionHooks as ProductionTurnCompositionHooks",
        "pub fn compose_production_turn(",
        "create_production_runner_composition_with_hooks(input, hooks)",
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
        SHIPPING_HOST.contains(
            "let worker_host_runner_composition = Arc::clone(&host_runner_composition);"
        ),
        "shipping Runner worker must use the same HostRunnerComposition owner via Arc::clone",
    );
    assert!(
        SHIPPING_HOST.contains("worker_host_runner_composition.compose_production_turn("),
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
        "if group_member_turn",
        "return Ok(None)",
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
        "group_member_turn: bool",
        "state_writer: None",
        "SandAgentState::new(sand_root, agent_id.to_string())",
        "sessions.open_agent_db_owner(agent_id)?",
    ] {
        assert!(
            OWNER.contains(needle),
            "HostRunnerComposition missing state-surface responsibility: {needle}"
        );
    }
    assert!(
        SHIPPING_HOST.contains(".compose_turn_state_surfaces(")
            && SHIPPING_HOST.contains("&session_workers,")
            && SHIPPING_HOST.contains("&agent_id,")
            && SHIPPING_HOST.contains("is_group_member_turn,")
            && SHIPPING_HOST.contains("multitask_enabled,"),
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
        "create_production_runner(",
        "composition,",
        "checkpoint_sink,",
        "upgrade_quiesce_signal,",
        "generated_agent_runtime,",
    ] {
        assert!(
            OWNER.contains(needle),
            "HostRunnerComposition missing Runner construction responsibility: {needle}"
        );
    }
    assert!(
        SHIPPING_HOST.contains("worker_host_runner_composition.compose_production_runner("),
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

#[test]
fn host_runner_composition_owns_computer_use_session_lifecycle() {
    for needle in [
        "pub fn begin_computer_use_preparation(",
        "owner.acquire_control_lease(agent_id)",
        "owner.begin_preparation(agent_id)",
        "ComputerUsePrewarmStage::Box",
        "pub fn mark_computer_use_preparation_ready(",
        "pub fn mark_computer_use_preparation_failed(",
        "pub fn owns_computer_control_lease(",
        "pub fn finish_computer_use_turn(",
        "owner.record_model_id(model_id)",
        "owner.record_turn_ended(usage)",
        "owner.release_control_lease(lease)",
        "owner.free_window(agent_id)",
    ] {
        assert!(
            OWNER.contains(needle),
            "HostRunnerComposition missing computer-use lifecycle ownership: {needle}"
        );
    }
    for needle in [
        "computer_use_coordination()",
        ".acquire_control_lease(&agent_id)",
        ".begin_preparation(&agent_id)",
        ".record_turn_ended(usage)",
        ".release_control_lease(lease)",
        ".free_window(&agent_id)",
    ] {
        assert!(
            !SHIPPING_HOST.contains(needle),
            "shipping Host still owns computer-use session composition: {needle}"
        );
    }
    assert!(
        SHIPPING_HOST.contains("host_runner_composition.begin_computer_use_preparation(&agent_id)"),
        "shipping Host must delegate computer-use preparation",
    );
    assert!(
        SHIPPING_HOST.contains("worker_host_runner_composition.finish_computer_use_turn("),
        "shipping Host must delegate computer-use turn settlement",
    );
}


#[test]
fn group_member_turn_keeps_generated_lifecycle_without_private_state() {
    let group_flag = SHIPPING_HOST
        .find("let is_group_member_turn = args")
        .expect("shipping Host must resolve group-member identity");
    let private_memory = SHIPPING_HOST
        .find("let memory_store = if is_group_member_turn")
        .expect("private memory must be explicitly gated for group-member turns");
    assert!(
        group_flag < private_memory,
        "group-member identity must be known before private memory is resolved",
    );
    assert!(
        OWNER.contains("return Ok(None)")
            && OWNER.contains("state_writer: None")
            && OWNER.contains("group_member_turn: bool"),
        "HostRunnerComposition must omit private checkpoint/state surfaces for group-member turns",
    );
    assert!(
        SHIPPING_HOST.contains("if !is_group_member_turn")
            && SHIPPING_HOST.contains("with_persist_image_callback(browser_persist_image)")
            && SHIPPING_HOST.contains("with_persist_image_callback(computer_persist_image)"),
        "group-member turns must not bind private image persistence callbacks",
    );
    assert!(
        GENERATED_STREAM.contains(
            "checkpoint_sink: Option<Arc<dyn AgentStateCheckpointSink>>"
        ) && GENERATED_STREAM.contains(
            "pub checkpoint: Option<TextTurnCheckpointArtifacts>"
        ),
        "generated Agent lifecycle must model private checkpointing as optional",
    );
    assert!(
        TURN_OWNER.contains("run_production_generated_agent_stream(")
            && !TURN_OWNER.contains("if let Some(checkpoint_sink) = checkpoint_sink"),
        "absence of a private checkpoint must not downgrade the turn to a raw provider path",
    );
}


#[test]
fn host_runner_composition_settles_only_its_owned_shutdown_surfaces() {
    assert!(
        OWNER.contains("pub fn dispose(&self)")
            && OWNER.contains(".clear();"),
        "HostRunnerComposition must dispose its live permission subscriptions",
    );
    let cancel = SHIPPING_HOST
        .find("runner_registry.cancel_all(\"Mahayana Host shutting down\")")
        .expect("shipping shutdown must interrupt canonical Runner registry");
    let dispose = SHIPPING_HOST
        .find("host_runner_composition.dispose();")
        .expect("shipping shutdown must settle HostRunnerComposition");
    assert!(
        cancel < dispose,
        "RunnerRegistry must interrupt active turns before composition-owned surfaces are dropped",
    );
    assert!(
        !OWNER.contains("runner_registry.cancel_all"),
        "HostRunnerComposition must not duplicate TranscriptRunnerRegistry cancellation ownership",
    );
}
