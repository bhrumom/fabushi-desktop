const OWNER: &str = include_str!("../src/host_runner_composition.rs");
const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
const GENERATED_STREAM: &str = include_str!("../src/runner/generated_agent_turn_stream.rs");
const TURN_OWNER: &str = include_str!("../src/runner/production_turn_agent_owner.rs");
const MULTITASK_STATE: &str = include_str!("../src/runner/tools/sand_multitask_todo_tool.rs");

fn production_host_source(source: &str) -> Option<&str> {
    const TEST_MODULE: &str = "\n#[cfg(test)]\nmod tests {";
    let mut boundaries = source.match_indices(TEST_MODULE);
    let (start, _) = boundaries.next()?;
    if boundaries.next().is_some() {
        return None;
    }
    // Syntax, rather than indentation or a later closing brace, establishes
    // that no production item follows the excluded test module.
    let parsed = syn::parse_file(source).ok()?;
    let syn::Item::Mod(tests) = parsed.items.last()? else {
        return None;
    };
    if tests.ident != "tests" || tests.content.is_none()
        || !tests.attrs.iter().any(|attribute| {
            attribute.path().is_ident("cfg")
                && attribute.parse_args::<syn::Ident>()
                    .is_ok_and(|condition| condition == "test")
        })
    {
        return None;
    }
    if parsed.items[..parsed.items.len() - 1].iter()
        .any(|item| matches!(item, syn::Item::Mod(module) if module.ident == "tests"))
    {
        return None;
    }
    let production = &source[..start];
    let prefix = syn::parse_file(production).ok()?;
    if prefix.items.len().checked_add(1) != Some(parsed.items.len()) {
        return None;
    }
    prefix.items.iter()
        .any(|item| matches!(item, syn::Item::Fn(function) if function.sig.ident == "main"))
        .then_some(production)
}

fn shipping_host() -> &'static str {
    production_host_source(SHIPPING_HOST)
        .expect("shipping Host source must have main before its explicit cfg(test) tests module")
}

#[test]
fn shipping_source_oracle_requires_boundary_and_retains_production_violations() {
    let forbidden = "ProductionTurnAgentOwner::new(composition)";
    let fixture_only = format!(
        "\nfn main() {{ production_entry(); }}\n#[cfg(test)]\nmod tests {{ fn fixture() {{ {forbidden}; fixture_only_entry(); }} }}"
    );
    let production = production_host_source(&fixture_only).expect("bounded fixture source");
    assert!(production.contains("production_entry()"));
    assert!(!production.contains(forbidden));
    assert!(!production.contains("fixture_only_entry()"));

    let production_violation = format!(
        "\nfn main() {{ {forbidden}; }}\n#[cfg(test)]\nmod tests {{}}"
    );
    assert!(production_host_source(&production_violation).unwrap().contains(forbidden));
    assert!(production_host_source("\nfn main() {}").is_none());
    assert!(production_host_source("\n#[cfg(test)]\nmod tests { fn main() {} }").is_none());
    assert!(production_host_source("\nfn main() {}\nmod tests {}").is_none());
}

#[test]
fn shipping_source_oracle_rejects_production_after_any_test_module_close() {
    for constructor in [
        "ProductionTurnAgentOwner::new(composition)",
        "SandAgentRunner::new(owner)",
    ] {
        for tests in [
            "\n#[cfg(test)]\nmod tests {}\n",
            "\n#[cfg(test)]\nmod tests {\n    }\n",
            "\n#[cfg(test)]\nmod tests {\n}\n",
        ] {
            for indent in ["", "    "] {
                let source = format!(
                    "\nfn main() {{ worker_host_runner_composition.compose_production_runner(); }}{tests}{indent}fn bypass() {{\n{indent}    {constructor};\n{indent}}}\n"
                );
                assert!(production_host_source(&source).is_none(),
                    "a later production constructor must not be hidden: {source}");
            }
        }
    }
}

#[test]
fn shipping_source_oracle_refuses_ambiguous_or_incomplete_boundaries() {
    for source in [
        "\nfn main() {}\n#[cfg(test)]\nmod tests {",
        "\nfn main() {}\n#[cfg(test)]\nmod tests {}\n#[cfg(test)]\nmod tests {}",
        "\nfn main() {}\n#[cfg(test)]\nmod tests {}\n#[cfg(test)]\nmod tests;",
        "\nconst NOTE: &str = \"fn main(\";\n#[cfg(test)]\nmod tests {}",
    ] {
        assert!(production_host_source(source).is_none(),
            "a missing/ambiguous boundary or test-only main is not shipping evidence: {source}");
    }
    let only_test_delegates = "\nfn main() {}\n#[cfg(test)]\nmod tests {\n    fn fixture() { worker_host_runner_composition.compose_production_runner(); }\n}\n";
    assert!(!production_host_source(only_test_delegates).expect("valid terminal test module")
        .contains("worker_host_runner_composition.compose_production_runner("));
}

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
        shipping_host().contains(
            "let worker_host_runner_composition = Arc::clone(&host_runner_composition);"
        ),
        "shipping Runner worker must use the same HostRunnerComposition owner via Arc::clone",
    );
    assert!(
        shipping_host().contains("worker_host_runner_composition.compose_production_turn("),
        "shipping Host must delegate turn assembly to HostRunnerComposition",
    );
    assert!(
        !shipping_host().contains("create_production_runner_composition("),
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
            !shipping_host().contains(needle),
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
        "transcript_providers: Mutex<HashMap<PathBuf, Arc<CanonicalTranscriptMirrorProvider>>>",
        ".entry(transcripts_dir.clone())",
        "ProductionTranscriptMirrorProvider::with_reporter(",
        "GeneratedTranscriptOccurrenceCodec::new(",
        "CanonicalGeneratedToolJsonProjection",
        "Arc::clone(&report_outcome)",
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
        shipping_host().contains(".compose_production_checkpoint_sink("),
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
            !shipping_host().contains(needle),
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
        "SandAgentState::new_with_workflow_store(",
        "sessions.open_workflow_store(agent_id)?",
        "avatar_box_file_reader: Option<AvatarBoxFileReader>",
        "state = state.with_box_file_reader(reader)",
        "state = state.with_agent_memory_remover(Arc::new(move |memory_id|",
        "memory_sessions.remove_agent_memory(&memory_agent_id, memory_id)",
        "state = state.with_agent_profile_writer(Arc::new(move |name, description|",
        "profile_sessions.update_agent_profile_fields(",
        "ProductionMultitaskTodoState::new(",
        "Arc::clone(&sessions),",
    ] {
        assert!(
            OWNER.contains(needle),
            "HostRunnerComposition missing state-surface responsibility: {needle}"
        );
    }
    // Session admission owns every todo mutation. The production adapter must
    // retain that facade, rather than opening a raw DB for the turn lifetime.
    for needle in [
        "sessions: Arc<ProductionSessionWorkers>",
        ".read_agent_kv_projection(&self.agent_id, SAND_MULTITASK_TODOS_KV)?",
        "self.sessions.write_agent_kv_projection(",
        "multitask todo write was rejected by session admission",
    ] {
        assert!(
            MULTITASK_STATE.contains(needle),
            "multitask state bypasses the canonical Session boundary: {needle}"
        );
    }
    assert!(
        !OWNER.contains("sessions.open_agent_db_owner(agent_id)?")
            && !MULTITASK_STATE.contains("SandAgentDb"),
        "turn state must not retain a raw DB handle outside Session admission",
    );
    assert!(
        shipping_host().contains(".compose_turn_state_surfaces(")
            && shipping_host().contains("Arc::clone(&session_workers),")
            && shipping_host().contains("&agent_id,")
            && shipping_host().contains("is_group_member_turn,")
            && shipping_host().contains("multitask_enabled,")
            && shipping_host().contains("let avatar_box_file_reader: AvatarBoxFileReader")
            && shipping_host().contains("forever_box")
            && shipping_host().contains(".download_file(&(), &avatar_agent_id, path)")
            && shipping_host().contains("Some(avatar_box_file_reader),"),
        "shipping Host must consume HostRunnerComposition state surfaces",
    );
    for needle in [
        "SandAgentState::new(state_sand_root",
        "could not open production multitask todo state for {agent_id}",
        "production memory agents root has no sand root parent",
    ] {
        assert!(
            !shipping_host().contains(needle),
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
    assert_shipping_runner_construction(SHIPPING_HOST);
}

fn assert_shipping_runner_construction(source: &str) {
    let production = production_host_source(source)
        .expect("shipping Host production scope must be unambiguous");
    assert!(
        production.contains("worker_host_runner_composition.compose_production_runner("),
        "shipping Host must delegate Runner facade construction",
    );
    for needle in [
        "ProductionTurnAgentOwner::new(composition)",
        "SandAgentRunner::new(owner)",
    ] {
        assert!(
            !production.contains(needle),
            "shipping Host still constructs the Runner owner path directly: {needle}",
        );
    }
}

const DELEGATING_RUNNER_PRODUCTION: &str =
    "\nfn main() { worker_host_runner_composition.compose_production_runner(); }\n";
const EMPTY_RUNNER_TEST_MODULE: &str = "\n#[cfg(test)]\nmod tests {\n}\n";

#[test]
fn runner_construction_oracle_allows_test_only_constructors_and_preserves_source() {
    let production = r#"// Preserve whitespace, strings and comments in the production prefix.
fn main() {
    let _quoted = "} // {";
    /* { /* nested comment } */ } */
    worker_host_runner_composition.compose_production_runner();
}

"#;
    let test_module = r##"
#[cfg(test)]
mod tests {
    fn fixture() {
        let _quoted = "} \" {";
        let _raw = r#"} /* { */"#;
        let _brace = '}';
        // } } {
        /* } /* { */ } */
        if true {
            let owner = ProductionTurnAgentOwner::new(composition);
            let _runner = SandAgentRunner::new(owner);
        }
    }
}
"##;
    let source = format!("{production}{test_module}");

    assert_eq!(
        production_host_source(&source).expect("valid production and test syntax"),
        production
    );
    assert_shipping_runner_construction(&source);
}

#[test]
#[should_panic(
    expected = "shipping Host still constructs the Runner owner path directly: ProductionTurnAgentOwner::new(composition)"
)]
fn runner_construction_oracle_rejects_production_turn_owner_constructor() {
    let source = format!(
        "{DELEGATING_RUNNER_PRODUCTION}\nfn bypass() {{ let _ = ProductionTurnAgentOwner::new(composition); }}{EMPTY_RUNNER_TEST_MODULE}"
    );
    assert_shipping_runner_construction(&source);
}

#[test]
#[should_panic(
    expected = "shipping Host still constructs the Runner owner path directly: SandAgentRunner::new(owner)"
)]
fn runner_construction_oracle_rejects_production_runner_constructor() {
    let source = format!(
        "{DELEGATING_RUNNER_PRODUCTION}\nfn bypass() {{ let _ = SandAgentRunner::new(owner); }}{EMPTY_RUNNER_TEST_MODULE}"
    );
    assert_shipping_runner_construction(&source);
}

#[test]
#[should_panic(expected = "shipping Host must delegate Runner facade construction")]
fn runner_construction_oracle_rejects_delegation_present_only_in_tests() {
    assert_shipping_runner_construction(
        "\nfn main() {}\n#[cfg(test)]\nmod tests {\n    fn fixture() { worker_host_runner_composition.compose_production_runner(); }\n}\n",
    );
}

#[test]
fn runner_construction_scope_refuses_production_after_test_module() {
    let valid = format!("{DELEGATING_RUNNER_PRODUCTION}{EMPTY_RUNNER_TEST_MODULE}");
    assert_eq!(
        production_host_source(&valid),
        Some(DELEGATING_RUNNER_PRODUCTION)
    );
    for trailing in [
        "fn bypass() { let _ = ProductionTurnAgentOwner::new(composition); }",
        "    fn bypass() { let _ = SandAgentRunner::new(owner); }",
        "const PRODUCTION_ITEM: bool = true;",
    ] {
        let source =
            format!("{DELEGATING_RUNNER_PRODUCTION}{EMPTY_RUNNER_TEST_MODULE}{trailing}\n");
        assert!(
            production_host_source(&source).is_none(),
            "shipping ownership assertions must refuse trailing production: {trailing}"
        );
    }
}

#[test]
fn runner_construction_scope_refuses_missing_duplicate_or_malformed_boundaries() {
    let valid = format!("{DELEGATING_RUNNER_PRODUCTION}{EMPTY_RUNNER_TEST_MODULE}");
    assert_eq!(
        production_host_source(&valid),
        Some(DELEGATING_RUNNER_PRODUCTION)
    );
    for (label, source) in [
        (
            "absent boundary",
            DELEGATING_RUNNER_PRODUCTION.to_string(),
        ),
        (
            "duplicate boundary",
            format!("{valid}{EMPTY_RUNNER_TEST_MODULE}"),
        ),
        (
            "indirect cfg",
            valid.replace("#[cfg(test)]", "#[cfg(any(test))]"),
        ),
        (
            "different module",
            valid.replace("mod tests {", "mod fixtures {"),
        ),
        (
            "out-of-line module",
            valid.replace("mod tests {\n}", "mod tests;"),
        ),
        (
            "unclosed test module",
            format!("{DELEGATING_RUNNER_PRODUCTION}\n#[cfg(test)]\nmod tests {{"),
        ),
        (
            "malformed module body",
            format!("{DELEGATING_RUNNER_PRODUCTION}\n#[cfg(test)]\nmod tests {{ fn broken( }}"),
        ),
    ] {
        assert!(
            production_host_source(&source).is_none(),
            "shipping ownership assertions must refuse {label}"
        );
    }
}

#[test]
fn runner_construction_scope_refuses_boundaries_inside_incomplete_prefixes() {
    // A complete main precedes each bad prefix so the main-entry check cannot
    // mask a regression that bypasses grammar qualification.
    for source in [
        format!("{DELEGATING_RUNNER_PRODUCTION}fn enclosing() {{{EMPTY_RUNNER_TEST_MODULE}}}"),
        format!("{DELEGATING_RUNNER_PRODUCTION}mod production {{{EMPTY_RUNNER_TEST_MODULE}}}"),
        format!("{DELEGATING_RUNNER_PRODUCTION}/*{EMPTY_RUNNER_TEST_MODULE}*/"),
        format!(
            "{DELEGATING_RUNNER_PRODUCTION}const TEXT: &str = r#\"{EMPTY_RUNNER_TEST_MODULE}\"#;"
        ),
    ] {
        assert!(
            production_host_source(&source).is_none(),
            "shipping ownership assertions must refuse incomplete production prefixes"
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
        "pub fn record_computer_audit_intent(",
        "pub fn capture_computer_navigation_baseline(",
        "pub fn request_computer_navigation_probe(",
        "pub fn computer_navigation_probe_wait_ms(",
        "pub fn complete_computer_navigation_probe(",
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
        ".acquire_control_lease(&agent_id)",
        ".begin_preparation(&agent_id)",
        ".record_turn_ended(usage)",
        ".release_control_lease(lease)",
        ".free_window(&agent_id)",
    ] {
        assert!(
            !shipping_host().contains(needle),
            "shipping Host still owns computer-use session composition: {needle}"
        );
    }
    assert_eq!(
        shipping_host().matches("computer_use_coordination()").count(),
        1,
        "shipping Host may only read the canonical HostRunnerComposition computer-use owner to inject that same service into SandAgentRunner",
    );
    assert!(
        shipping_host().contains(
            "Some(worker_host_runner_composition.computer_use_coordination()),"
        ),
        "shipping Runner must receive the canonical HostRunnerComposition computer-use coordination rather than a second owner",
    );
    assert!(
        shipping_host().contains("host_runner_composition.begin_computer_use_preparation(&agent_id)"),
        "shipping Host must delegate computer-use preparation",
    );
    assert!(
        shipping_host().contains("worker_host_runner_composition.finish_computer_use_turn("),
        "shipping Host must delegate computer-use turn settlement",
    );
    for needle in [
        "navigation_probe_command(display_number)",
        "worker_host_runner_composition.capture_computer_navigation_baseline(stdout)",
        ".with_post_action_callback(",
        "computer_navigation_owner.request_computer_navigation_probe()",
        "navigation_owner.complete_computer_navigation_probe(",
    ] {
        assert!(
            shipping_host().contains(needle),
            "shipping Host missing frozen computer-use navigation-probe wiring: {needle}",
        );
    }
    let normalized_shipping_host = shipping_host()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        normalized_shipping_host
            .contains("remote_computer_audit_owner .record_computer_audit_intent(action_case);"),
        "shipping Host must delegate computer audit intent to HostRunnerComposition",
    );
}


#[test]
fn group_member_turn_keeps_generated_lifecycle_without_private_state() {
    let group_flag = shipping_host()
        .find("let is_group_member_turn = args")
        .expect("shipping Host must resolve group-member identity");
    let private_memory = shipping_host()
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
        shipping_host().contains("if !is_group_member_turn")
            && shipping_host().contains("with_persist_image_callback(browser_persist_image)")
            && shipping_host().contains("with_persist_image_callback(computer_persist_image)"),
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
        TURN_OWNER.contains("run_production_generated_agent_stream_with_custody(")
            && !TURN_OWNER.contains("if let Some(checkpoint_sink) = checkpoint_sink"),
        "absence of a private checkpoint must not downgrade the turn to a raw provider path",
    );
    let compatibility_wrapper = GENERATED_STREAM
        .split("pub fn run_production_generated_agent_stream(").nth(1).expect("public Result entry retained")
        .split("pub(crate) struct ProductionGeneratedAgentStreamOutcome").next().unwrap();
    assert!(compatibility_wrapper.contains("run_production_generated_agent_stream_with_custody(")
        && compatibility_wrapper.contains(".result"),
        "the compatibility Result entry delegates to the same generated lifecycle");
}


#[test]
fn host_runner_composition_settles_only_its_owned_shutdown_surfaces() {
    assert!(
        OWNER.contains("pub fn dispose(&self)")
            && OWNER.contains(".clear();"),
        "HostRunnerComposition must dispose its live permission subscriptions",
    );
    let cancel = shipping_host()
        .find("runner_registry.cancel_all(\"Mahayana Host shutting down\")")
        .expect("shipping shutdown must interrupt canonical Runner registry");
    let dispose = shipping_host()
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


#[test]
fn shell_terminal_watch_shipping_path_uses_canonical_permission_and_generated_watermark() {
    let host = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app/src/main.rs"),
    )
    .expect("shipping host");
    let host = production_host_source(&host)
        .expect("shipping Host source must have main before its explicit cfg(test) tests module");
    let composition = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/host_runner_composition.rs"),
    )
    .expect("host runner composition");
    let forever_box = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/extensions/forever_box/runner_tools.rs"),
    )
    .expect("forever box runner tools");

    assert!(host.contains(".with_background_read_permission(Arc::clone(&shell_watch_local_tool_permission))"));
    assert!(forever_box.contains("SandLocalToolScope {"));
    assert!(forever_box.contains("agent_id: self.agent_id.clone()"));
    assert!(forever_box.contains("SandLocalToolRequest::simple(\"read-file\", output_path.clone())"));
    assert!(host.contains("confirmed_user_turn_watermark("));
    assert!(composition.contains("checkpoint_snapshot_checked()"));
    assert!(composition.contains("create_agent_blob_store(agent_id)"));
    assert!(composition.contains("decode_conversation_turn_structure_fields"));
    assert!(composition.contains("decode_user_message_identity_fields"));
    assert!(composition.contains("find_confirmed_user_turn_watermark(&turns, None)"));
    assert!(composition.contains("event.request.session_key"));
    assert!(composition.contains("append_agent_transcript_entries_if_session_admitted"));
    assert!(composition.contains("update_agent_transcript_entry_if_session_admitted"));
}


#[test]
fn generated_agent_stream_owns_canonical_stream_attempt_runtime() {
    const GENERATED_STREAM: &str =
        include_str!("../src/runner/generated_agent_turn_stream.rs");
    const TURN_COMPOSITION: &str =
        include_str!("../src/runner/turn_agent_composition.rs");
    const ROUTED_PROVIDER: &str =
        include_str!("../src/runner/routed_provider_runtime.rs");
    const ATTEMPT_ADAPTER: &str =
        include_str!("../src/runner/production_turn_run_shell_adapter.rs");

    assert!(GENERATED_STREAM.contains(
        "Arc::new(StreamAttemptRuntime::new("
    ));
    assert!(GENERATED_STREAM.contains(
        ".with_stream_attempt_runtime(Arc::clone(&stream_attempt_runtime))"
    ));
    let lifecycle_result = GENERATED_STREAM.find(
        "let result = futures::executor::block_on(path.run_lifecycle_with_output("
    ).expect("the generated owner waits for its real lifecycle result");
    let custody_snapshot = GENERATED_STREAM.find(
        "let tool_effect_custody = stream_attempt_runtime.effect_custody();"
    ).expect("terminal custody snapshots the same runtime injected into composition");
    assert!(lifecycle_result < custody_snapshot,
        "the retained canonical runtime is observed after the lifecycle invocation");
    assert!(TURN_COMPOSITION.contains(
        "stream_attempt_runtime: self.stream_attempt_runtime.clone()"
    ));
    assert!(ROUTED_PROVIDER.contains(
        "run_with_attempt_runtime_reporting("
    ));
    assert!(ATTEMPT_ADAPTER.contains(
        "let generation = runtime.begin_attempt();"
    ));
    assert!(ATTEMPT_ADAPTER.contains(
        "runtime_for_delta.mark_stream_output(generation)"
    ));
    assert!(ATTEMPT_ADAPTER.contains(
        "runtime_for_checkpoint.is_current(generation)"
    ));
}

fn shipping_skills_wiring_matches(source: &str) -> bool {
    let Some(production) = production_host_source(source) else { return false; };
    let Some((_, after_start)) = production.split_once("\nfn start_routed_provider_task(") else {
        return false;
    };
    let Some((turn, _)) = after_start.split_once("\nfn ") else { return false; };
    let Some(store) = turn.find("let workflow_store = session_workers\n        .open_workflow_store(&agent_id)") else {
        return false;
    };
    let Some(projection) = turn.find("let available_skill_prompts =") else { return false; };
    let Some(prompt) = turn.find("append_budgeted_workflows_system_prompt(") else { return false; };
    if !(store < projection && projection < prompt)
        || !turn[projection..prompt].contains("workflow_store")
        || !turn[projection..prompt].contains(".list()")
        || !turn[prompt..].starts_with(
            "append_budgeted_workflows_system_prompt(\n        &mut provider_messages,\n        Some(&workflow_location),\n        &available_skill_prompts,"
        )
    {
        return false;
    }
    production.contains("let mcp_sand_root = match session_workers.agents_root().parent() {")
        && production.contains("Session agents root has no SAND root parent")
        && production.contains("production_extensions.start_mcp(\n        mcp_sand_root,")
}

#[test]
fn shipping_skills_cache_and_prompt_use_the_bound_session_store_and_root() {
    assert!(shipping_skills_wiring_matches(SHIPPING_HOST),
        "actual shipping MCP root and provider skill projection must share the Session owner");
    for wrong in [
        SHIPPING_HOST.replacen(
            "production_extensions.start_mcp(\n        mcp_sand_root,",
            "production_extensions.start_mcp(\n        &app_data_dir,", 1,
        ),
        SHIPPING_HOST.replacen(
            "let workflow_store = session_workers\n        .open_workflow_store(&agent_id)",
            "let workflow_store = unbound_sessions\n        .open_workflow_store(&agent_id)", 1,
        ),
        SHIPPING_HOST.replacen(
            "Some(&workflow_location),\n        &available_skill_prompts,",
            "Some(&workflow_location),\n        &unbound_skill_prompts,", 1,
        ),
    ] {
        assert!(!shipping_skills_wiring_matches(&wrong),
            "a wrong root, unbound store or unrelated prompt projection must fail the wiring oracle");
    }
}
