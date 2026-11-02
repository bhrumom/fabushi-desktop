const OWNER: &str = include_str!("../src/host_runner_composition.rs");
const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
const GENERATED_STREAM: &str = include_str!("../src/runner/generated_agent_turn_stream.rs");
const TURN_OWNER: &str = include_str!("../src/runner/production_turn_agent_owner.rs");
const MULTITASK_STATE: &str = include_str!("../src/runner/tools/sand_multitask_todo_tool.rs");
const SESSION_FACADE: &str = include_str!("../src/extensions/session/agent_session.rs");

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

fn runner_workflow_constructor_uses_session_boundary(source: &str) -> bool {
    let Ok(file) = syn::parse_file(source) else { return false; };
    let mut methods = file.items.iter().filter_map(|item| {
        let syn::Item::Impl(item) = item else { return None; };
        let syn::Type::Path(owner) = item.self_ty.as_ref() else { return None; };
        (item.trait_.is_none() && owner.path.is_ident("SandAgentSessionStore"))
            .then_some(item)
    }).flat_map(|owner| owner.items.iter()).filter_map(|item| {
        let syn::ImplItem::Fn(method) = item else { return None; };
        (method.sig.ident == "with_runner_workflow_store").then_some(method)
    });
    let Some(method) = methods.next() else { return false; };
    if methods.next().is_some() { return false; }
    let [syn::Stmt::Expr(syn::Expr::MethodCall(gate), None)] = method.block.stmts.as_slice()
        else { return false; };
    if gate.method != "with_agent_admitted_db_mutation" || gate.args.len() != 2 {
        return false;
    }
    let syn::Expr::Field(receiver) = gate.receiver.as_ref() else { return false; };
    let syn::Expr::Path(receiver_owner) = receiver.base.as_ref() else { return false; };
    if !receiver_owner.path.is_ident("self")
        || !matches!(&receiver.member, syn::Member::Named(name) if name == "production")
    {
        return false;
    }
    let Some(syn::Expr::Closure(operation)) = gate.args.last() else { return false; };
    let syn::Expr::Block(body) = operation.body.as_ref() else { return false; };
    let constructed_inside = body.block.stmts.iter().any(|statement| {
        let syn::Stmt::Local(local) = statement else { return false; };
        let syn::Pat::Ident(name) = &local.pat else { return false; };
        if name.ident != "store" { return false; }
        let Some(initializer) = &local.init else { return false; };
        let syn::Expr::Try(initializer) = initializer.expr.as_ref() else { return false; };
        let syn::Expr::MethodCall(constructor) = initializer.expr.as_ref() else { return false; };
        let syn::Expr::Path(owner) = constructor.receiver.as_ref() else { return false; };
        constructor.method == "workflow_store_for" && owner.path.is_ident("self")
    });
    let Some(syn::Stmt::Expr(syn::Expr::Call(consume), None)) = body.block.stmts.last()
        else { return false; };
    constructed_inside
        && matches!(consume.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("operation"))
        && matches!(consume.args.last(), Some(syn::Expr::Path(path)) if path.path.is_ident("store"))
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
        "session_store.open_workflow_store_for_runner(agent_id)?",
        "&workflow_session_key,",
        "&workflow_lifecycle_revision,",
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
    assert!(runner_workflow_constructor_uses_session_boundary(SESSION_FACADE),
        "Runner workflow construction must execute inside the actual Session admission boundary");
    let wrong_gate = SESSION_FACADE.replacen(
        "self.production.with_agent_admitted_db_mutation(agent_id, |owner| {",
        "self.production.with_agent_filesystem_mutation(agent_id, |owner| {", 1,
    );
    assert_ne!(wrong_gate, SESSION_FACADE, "constructor gate counterfactual must change the source");
    let comment_only = format!(
        "// self.production.with_agent_admitted_db_mutation(agent_id, |owner| {{\n{wrong_gate}"
    );
    assert!(!runner_workflow_constructor_uses_session_boundary(&comment_only),
        "a comment cannot replace the actual admitted DB boundary");
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
fn production_memory_tool_refuses_corrupt_append_destinations_and_recovers_after_repair() {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use chrono::{Datelike, Months, Utc};
    use mahayana_host_runtime::extensions::inference::provider_session::{
        ProviderSessionError, RoutedToolDefinition,
    };
    use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
    use mahayana_host_runtime::extensions::memory::agent_state::{MemoryScope, MemoryTier};
    use mahayana_host_runtime::extensions::memory::memory_service::{
        MemoryKind, MemoryOrigin, MemorySynthesisBridge, memory_id_for,
    };
    use mahayana_host_runtime::extensions::memory::memory_synthesis_service::{
        MemorySynthesisOptions, MemorySynthesisService,
    };
    use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
    use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
    use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
    use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
    use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
    use mahayana_host_runtime::runner::tools::sand_state_tool::{
        SAND_UPDATE_STATE_TOOL_NAME, SandStateToolBridge,
    };
    use serde_json::{Value, json};

    struct NoMemoryFallback;
    impl RoutedToolBridge for NoMemoryFallback {
        fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
            Ok(Vec::new())
        }

        fn call_tool(
            &self,
            _tool: &RoutedToolDefinition,
            _args: Value,
            _tool_call_id: &str,
        ) -> Result<Value, ProviderSessionError> {
            panic!("the production update_state bridge must consume memory writes");
        }
    }

    fn memory_tree(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
        fn visit(root: &Path, path: &Path, tree: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
            tree.insert(path.strip_prefix(root).unwrap().to_path_buf(), None);
            for entry in fs::read_dir(path).expect("read complete owned memory tree") {
                let entry = entry.expect("memory entry");
                let path = entry.path();
                let kind = entry.file_type().expect("memory entry type");
                if kind.is_dir() {
                    visit(root, &path, tree);
                } else {
                    assert!(kind.is_file(), "fixture must contain only directories and files");
                    tree.insert(
                        path.strip_prefix(root).unwrap().to_path_buf(),
                        Some(fs::read(&path).expect("raw memory bytes")),
                    );
                }
            }
        }
        let mut tree = BTreeMap::new();
        visit(root, root, &mut tree);
        tree
    }

    let root = std::env::temp_dir().join(format!(
        "fabushi-production-memory-append-{}", uuid::Uuid::new_v4().simple(),
    ));
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let sessions = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = sessions.create_session(None, "user", None).expect("real admitted agent");
    let admission = workers.read_session_work_admission(&agent.id)
        .expect("read admission").expect("materialized admission");
    assert_eq!(admission.session_id, agent.id);
    assert_eq!(admission.session_key, format!("agent:{}:main", agent.id));
    assert!(!admission.lifecycle_revision.is_empty());
    assert!(!admission.deletion_pending);

    let memory = workers.memory_service();
    // Enable the real origin/tombstone policy without accepting background
    // synthesis or provider work. Memory appends still enter the Session owner.
    let synthesis = Arc::new(MemorySynthesisService::new(MemorySynthesisOptions::new(
        Arc::new(Vec::new),
        Arc::new(|_| None),
        Arc::new(|_, _| panic!("explicit-memory fixture must not request synthesis")),
    )));
    synthesis.start();
    let synthesis_bridge: Arc<dyn MemorySynthesisBridge> = synthesis.clone();
    memory.set_synthesis_bridge(Arc::downgrade(&synthesis_bridge));
    assert!(memory.synthesis_enabled());
    let store = memory.store_for_agent(&agent.id);
    let memory_dir = store.get_location();

    let controller = Arc::new(SandLocalToolPermissionController::production(Arc::new(
        SettingsService::new(root.join("settings.json")),
    )));
    let composition = HostRunnerComposition::production(controller, Arc::clone(&workers));
    let writer = composition.compose_turn_state_surfaces(
        Arc::clone(&workers), &agent.id, false, false, None,
    ).expect("production turn state").state_writer.expect("private state writer");
    let bridge = SandStateToolBridge::new(Arc::new(NoMemoryFallback), Arc::clone(&writer));
    let tool = bridge.list_tools().expect("real state tool registration").into_iter()
        .find(|tool| tool.name == SAND_UPDATE_STATE_TOOL_NAME).expect("update_state");
    let write_tool = |fact: &str, tier: &str| {
        bridge.call_tool(&tool, json!({
            "target": "memory", "action": "write", "scope": "agent",
            "fact": fact, "tier": tier,
        }), "mwr-append").expect("existing state-tool response contract")
            .as_str().expect("state tool message").to_string()
    };

    for (tier, kind, label, history, appended) in [
        (MemoryTier::Profile, MemoryKind::Profile, "profile",
            "Retained profile history", "Recovered profile append"),
        (MemoryTier::Log, MemoryKind::Log, "log",
            "Retained monthly log history", "Recovered monthly log append"),
    ] {
        let missing_destination = match kind {
            MemoryKind::Profile => memory_dir.join("profile.md"),
            MemoryKind::Log => memory_dir.join("log").join(format!("{}.md", Utc::now().format("%Y-%m"))),
        };
        assert!(!missing_destination.exists(), "first-write control: {label}");
        assert_eq!(write_tool(history, label), format!("Remembered in agent memory ({label}): {history}"));
        assert_eq!(write_tool(appended, label), format!("Remembered in agent memory ({label}): {appended}"));
        let removed = writer.remove_memory(appended, MemoryScope::Agent, None);
        assert!(removed.ok, "{}", removed.message);
        let tombstone = memory_dir.join(".dreaming/tombstones")
            .join(format!("{}.deleted", memory_id_for(appended)));
        assert!(tombstone.is_file(), "real removal must leave the retry's tombstone");
        store.mark_temporal_review(1_000).expect("retained review metadata");

        let destinations = match kind {
            MemoryKind::Profile => vec![memory_dir.join("profile.md")],
            MemoryKind::Log => {
                // Session chooses the real current time. Cover the following
                // month too so a UTC month boundary cannot escape the fault.
                let month = Utc::now().date_naive().with_day(1).unwrap();
                [month, month.checked_add_months(Months::new(1)).unwrap()]
                    .into_iter().map(|month| memory_dir.join("log")
                        .join(format!("{}.md", month.format("%Y-%m"))))
                    .collect()
            }
        };
        let mut repairs = Vec::new();
        for path in destinations {
            let mut repaired = match fs::read_to_string(&path) {
                Ok(raw) => raw,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound =>
                    "# Memory log\n\n<!-- Retained rollover history -->\n".to_string(),
                Err(error) => panic!("fixture history was unreadable before corruption: {error}"),
            };
            repaired.push_str("\n<!-- MWR-v1 retained destination comment -->\n");
            let mut corrupt = repaired.as_bytes().to_vec();
            corrupt.extend_from_slice(&[0xff, 0xfe]);
            fs::write(&path, &corrupt).expect("inject invalid UTF-8 into the real destination");
            assert_eq!(fs::read(&path).expect("corruption remains raw-readable"), corrupt);
            assert_eq!(fs::read_to_string(&path).unwrap_err().kind(), std::io::ErrorKind::InvalidData);
            repairs.push((path, repaired));
        }
        let before = memory_tree(&memory_dir);
        let read_error = fs::read_to_string(&repairs[0].0).unwrap_err().to_string();
        let refused = writer.write_memory(appended, tier, MemoryScope::Agent, None);
        assert!(!refused.ok, "corrupt {label} append must not report Remembered");
        assert_eq!(refused.message, format!("could not update agent memory: {read_error}"));
        assert_eq!(memory_tree(&memory_dir), before, "writer refusal changed {label} bytes or metadata");
        assert_eq!(write_tool(appended, label), format!("Not saved — {}", refused.message));
        assert_eq!(memory_tree(&memory_dir), before, "tool refusal changed {label} bytes or metadata");
        assert_eq!(workers.read_session_work_admission(&agent.id).unwrap(), Some(admission.clone()));

        // Only the fixture repairs the file; retain the same live Session,
        // composition, SandAgentState writer and tool bridge for the retry.
        for (path, repaired) in &repairs {
            fs::write(path, repaired).expect("fixture-owned repair");
        }
        assert_eq!(write_tool(appended, label), format!("Remembered in agent memory ({label}): {appended}"));
        for (path, repaired) in &repairs {
            assert!(fs::read_to_string(path).unwrap().starts_with(repaired.as_str()),
                "successful append must preserve repaired history and comments");
        }
        let records = store.prepare_synthesis().memories;
        assert!(records.iter().any(|record| record.content == history && record.kind == kind));
        let recovered = records.iter().filter(|record| record.content == appended).collect::<Vec<_>>();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].kind, kind);
        assert_eq!(recovered[0].origin, MemoryOrigin::Explicit);
        assert!(!tombstone.exists(), "only the successful retry clears the tombstone");

        let after = memory_tree(&memory_dir);
        assert_eq!(write_tool(&format!("  {}  ", appended.to_uppercase()), label),
            "Not saved — nothing was saved to agent memory - the fact was empty or already recorded.");
        assert_eq!(memory_tree(&memory_dir), after, "readable duplicate must preserve the tree");
    }

    // A retained production writer still obeys both Session deletion fences;
    // bypassing the composition callback for a raw SandAgentState cannot pass.
    let before_delete = memory_tree(&memory_dir);
    workers.begin_agent_delete(&agent.id).expect("begin canonical deletion fence");
    for durable_only in [false, true] {
        if durable_only {
            workers.clear_agent_delete_fast_path(&agent.id);
        }
        let prefix = if durable_only { "durable deletion" } else { "deletion" };
        assert_eq!(write_tool("Forbidden during deletion", "profile"), format!(
            "Not saved — could not update agent memory: agent {} {prefix} cleanup is pending; new session work is not admitted",
            agent.id,
        ));
        assert_eq!(memory_tree(&memory_dir), before_delete);
    }
    workers.end_agent_delete(&agent.id).expect("release fixture deletion fence");
    assert_eq!(workers.read_session_work_admission(&agent.id).unwrap(), Some(admission));
    assert_eq!(write_tool("Allowed after deletion fence release", "profile"),
        "Remembered in agent memory (profile): Allowed after deletion fence release");

    composition.dispose();
    synthesis.dispose();
    workers.shutdown();
    fs::remove_dir_all(root).expect("remove owned fixture");
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


// This oracle recognizes one direct shipping Gateway dispatch shape. It is not
// a general Rust control-flow or name-resolution proof.
fn desktop_mcp_dispatch_precedes_host_fallback(source: &str) -> bool {
    fn named(expr: &syn::Expr, name: &str) -> bool {
        matches!(expr, syn::Expr::Path(path) if path.path.is_ident(name))
    }
    fn self_field(expr: &syn::Expr, name: &str) -> bool {
        matches!(expr, syn::Expr::Field(field) if named(&field.base, "self")
            && matches!(&field.member, syn::Member::Named(member) if member == name))
    }
    let Some(source) = production_host_source(source) else { return false; };
    let Ok(parsed) = syn::parse_file(source) else { return false; };
    let mut methods = parsed.items.iter().filter_map(|item| {
        let syn::Item::Impl(item) = item else { return None; };
        let Some((_, gateway_trait, _)) = &item.trait_ else { return None; };
        let syn::Type::Path(owner) = item.self_ty.as_ref() else { return None; };
        (gateway_trait.is_ident("GatewayApi") && owner.path.is_ident("UnifiedGatewayApi"))
            .then_some(item)
    }).flat_map(|owner| owner.items.iter()).filter_map(|item| {
        let syn::ImplItem::Fn(method) = item else { return None; };
        (method.sig.ident == "call").then_some(method)
    });
    let Some(method) = methods.next() else { return false; };
    if methods.next().is_some() { return false; }
    let Some((fallback, before)) = method.block.stmts.split_last() else { return false; };
    let syn::Stmt::Expr(syn::Expr::Call(fallback), None) = fallback else { return false; };
    let method_index = if named(&fallback.func, "call_host_lane") {
        1
    } else if named(&fallback.func, "call_native_attachments_or_host_lane") {
        2
    } else {
        return false;
    };
    if fallback.args.len() != method_index + 2
        || !matches!(fallback.args.first(), Some(syn::Expr::Reference(reference))
            if reference.mutability.is_none() && self_field(&reference.expr, "host_tx"))
        || (method_index == 2 && !matches!(fallback.args.iter().nth(1), Some(syn::Expr::Reference(reference))
            if reference.mutability.is_none() && self_field(&reference.expr, "attachments")))
        || !fallback.args.iter().nth(method_index).is_some_and(|expr| named(expr, "method"))
        || !fallback.args.last().is_some_and(|expr| named(expr, "args"))
    {
        return false;
    }
    let Some(syn::Stmt::Expr(syn::Expr::If(dispatch), None)) = before.last() else { return false; };
    if dispatch.else_branch.is_some() { return false; }
    let syn::Expr::Let(binding) = dispatch.cond.as_ref() else { return false; };
    let syn::Pat::TupleStruct(pattern) = binding.pat.as_ref() else { return false; };
    if !pattern.path.is_ident("Some") || pattern.elems.len() != 1
        || !matches!(pattern.elems.first(), Some(syn::Pat::Ident(name))
            if name.ident == "result" && name.by_ref.is_none() && name.mutability.is_none()
                && name.subpat.is_none())
    {
        return false;
    }
    let syn::Expr::MethodCall(call) = binding.expr.as_ref() else { return false; };
    if call.method != "dispatch_desktop_command" || call.args.len() != 2
        || !self_field(&call.receiver, "mcp_service")
        || !call.args.first().is_some_and(|expr| named(expr, "method"))
        || !matches!(call.args.last(), Some(syn::Expr::Reference(reference))
            if reference.mutability.is_none() && named(&reference.expr, "args"))
    {
        return false;
    }
    let [syn::Stmt::Expr(syn::Expr::Return(returned), Some(_))] = dispatch.then_branch.stmts.as_slice()
        else { return false; };
    let Some(value) = &returned.expr else { return false; };
    let syn::Expr::MethodCall(projected) = value.as_ref() else { return false; };
    if projected.method != "map_err" || projected.args.len() != 1 || !named(&projected.receiver, "result") {
        return false;
    }
    let Some(syn::Expr::Path(error)) = projected.args.first() else { return false; };
    error.path.leading_colon.is_none()
        && error.path.segments.len() == 2
        && error.path.segments.first().is_some_and(|segment| segment.ident == "GatewayCommandError")
        && error.path.segments.last().is_some_and(|segment| segment.ident == "Internal")
}

// Recognize the actual returned ingress edge, not an unused helper reference.
// Runtime behavior and checked-read error semantics have real-owner oracles.
fn backend_fire_dispatch_method(parsed: &syn::File) -> Option<&syn::ImplItemFn> {
    let mut methods = parsed.items.iter().filter_map(|item| {
        let syn::Item::Impl(owner) = item else { return None; };
        let syn::Type::Path(path) = owner.self_ty.as_ref() else { return None; };
        (owner.trait_.is_none() && path.path.is_ident("UnifiedGatewayApi")).then_some(owner)
    }).flat_map(|owner| owner.items.iter()).filter_map(|item| {
        let syn::ImplItem::Fn(method) = item else { return None; };
        (method.sig.ident == "dispatch_production_backend_fire").then_some(method)
    });
    let method = methods.next()?;
    methods.next().is_none().then_some(method)
}

fn backend_fire_uses_checked_shipping_boundary(source: &str) -> bool {
    fn named(expr: &syn::Expr, name: &str) -> bool {
        matches!(expr, syn::Expr::Path(path) if path.path.is_ident(name))
    }
    fn projection(expr: &syn::Expr, receiver: &str, method: &str) -> bool {
        matches!(expr, syn::Expr::MethodCall(call) if call.method == method
            && named(&call.receiver, receiver) && call.args.is_empty())
    }
    let Some(production) = production_host_source(source) else { return false; };
    let Ok(parsed) = syn::parse_file(production) else { return false; };
    let mut helpers = parsed.items.iter().filter_map(|item| {
        let syn::Item::Fn(function) = item else { return None; };
        (function.sig.ident == "dispatch_checked_production_backend_fire").then_some(function)
    });
    if helpers.next().is_none() || helpers.next().is_some() { return false; }
    let Some(method) = backend_fire_dispatch_method(&parsed) else { return false; };
    let Some((syn::Stmt::Expr(syn::Expr::Call(call), None), before)) = method.block.stmts.split_last()
        else { return false; };
    if !named(&call.func, "dispatch_checked_production_backend_fire") || call.args.len() != 6 {
        return false;
    }
    let args = call.args.iter().collect::<Vec<_>>();
    if !projection(args[0], "runtime", "as_ref")
        || !matches!(args[1], syn::Expr::Reference(reference)
            if reference.mutability.is_none() && named(&reference.expr, "fire"))
        || !projection(args[2], "definition_time_zone", "as_deref")
        || !projection(args[3], "completion", "as_ref")
        || !matches!(args[4], syn::Expr::Closure(_))
        || !matches!(args[5], syn::Expr::Closure(_))
    {
        return false;
    }
    let mut bindings = before.iter().filter_map(|stmt| {
        let syn::Stmt::Local(local) = stmt else { return None; };
        let syn::Pat::Ident(binding) = &local.pat else { return None; };
        (binding.ident == "runtime").then_some(local)
    });
    let Some(binding) = bindings.next() else { return false; };
    if bindings.next().is_some() { return false; }
    let Some(init) = &binding.init else { return false; };
    let syn::Expr::MethodCall(open) = init.expr.as_ref() else { return false; };
    let syn::Expr::Field(manager) = open.receiver.as_ref() else { return false; };
    open.method == "automation_runtime" && open.args.is_empty()
        && named(&manager.base, "self")
        && matches!(&manager.member, syn::Member::Named(name) if name == "transcript_manager")
}

#[test]
fn shipping_backend_fire_returns_checked_owner_discovery_and_callback_boundary() {
    assert!(backend_fire_uses_checked_shipping_boundary(SHIPPING_HOST),
        "shipping backend ingress must return the checked production helper with its canonical Runtime, fire and completion callback");
}

#[test]
fn checked_backend_fire_wiring_oracle_rejects_bypasses_and_dead_calls() {
    let open = "let runtime = self.transcript_manager.automation_runtime();";
    let dispatch = "dispatch_checked_production_backend_fire(runtime.as_ref(), &fire, definition_time_zone.as_deref(), completion.as_ref(), |_| {}, |_, _| true)";
    let fixture = |body: &str, helper: &str, test_body: &str| format!(
        "fn main() {{}}\n{helper}\nimpl UnifiedGatewayApi {{ fn dispatch_production_backend_fire(&self, fire: Fire, completion: Callback) -> bool {{ {body} }} }}\n#[cfg(test)]\nmod tests {{ {test_body} }}\n"
    );
    let helper = "fn dispatch_checked_production_backend_fire() {}";
    assert!(backend_fire_uses_checked_shipping_boundary(&fixture(
        &format!("{open} {dispatch}"), helper, "",
    )));
    for body in [
        format!("{} {dispatch}", open.replace("self.transcript_manager", "self.unbound_manager")),
        format!("{open} {}", dispatch.replace("&fire", "&another_fire")),
        format!("{open} {}", dispatch.replace("completion.as_ref()", "another_completion.as_ref()")),
        format!("{open} let _ = {dispatch}; true"),
        format!("{open} let _unused = || {{ {dispatch} }}; true"),
        format!("{open} // {dispatch}\ntrue"),
        format!("{open} prepare_backend_fire(&runtime.list_all_automations().unwrap(), &fire, revision, history)"),
    ] {
        let source = fixture(&body, helper, "");
        assert!(syn::parse_file(&source).is_ok(), "negative remains syntactically valid");
        assert!(!backend_fire_uses_checked_shipping_boundary(&source),
            "bypassed checked shipping boundary: {source}");
    }
    let test_only = fixture(&format!("{open} {dispatch}"), "", helper);
    assert!(!backend_fire_uses_checked_shipping_boundary(&test_only));
}

// A bounded production-shape proof: the actual spawned scheduled worker must
// pass the canonical Runtime's typed attempt directly to the actual callback.
// Real owner/backend tests establish the behavior of that attempt and mapper.
fn backend_schedule_attempt_reaches_completion(source: &str) -> bool {
    fn named(expr: &syn::Expr, name: &str) -> bool {
        matches!(expr, syn::Expr::Path(path) if path.path.is_ident(name))
    }
    fn path_is(path: &syn::Path, owner: &str, member: &str) -> bool {
        path.leading_colon.is_none() && path.segments.len() == 2
            && path.segments.first().is_some_and(|part| part.ident == owner)
            && path.segments.last().is_some_and(|part| part.ident == member)
    }
    fn borrowed(expr: &syn::Expr, name: &str) -> bool {
        matches!(expr, syn::Expr::Reference(reference)
            if reference.mutability.is_none() && named(&reference.expr, name))
    }
    fn binding<'a>(statements: &'a [syn::Stmt], name: &str) -> Option<&'a syn::Expr> {
        let mut locals = statements.iter().filter_map(|statement| {
            let syn::Stmt::Local(local) = statement else { return None; };
            let syn::Pat::Ident(pattern) = &local.pat else { return None; };
            (pattern.ident == name).then_some((local, pattern))
        });
        let (local, pattern) = locals.next()?;
        if locals.next().is_some() || pattern.by_ref.is_some()
            || pattern.mutability.is_some() || pattern.subpat.is_some()
        { return None; }
        let init = local.init.as_ref()?;
        init.diverge.is_none().then_some(init.expr.as_ref())
    }
    fn cloned_from(expr: &syn::Expr, name: &str) -> bool {
        let syn::Expr::Call(call) = expr else { return false; };
        let syn::Expr::Path(path) = call.func.as_ref() else { return false; };
        path_is(&path.path, "Arc", "clone") && call.args.len() == 1
            && call.args.first().is_some_and(|arg| borrowed(arg, name))
    }
    if !backend_fire_uses_checked_shipping_boundary(source) { return false; }
    let Some(production) = production_host_source(source) else { return false; };
    let Ok(parsed) = syn::parse_file(production) else { return false; };
    let Some(method) = backend_fire_dispatch_method(&parsed) else { return false; };
    let Some((syn::Stmt::Expr(syn::Expr::Call(dispatch), None), before)) = method.block.stmts.split_last()
        else { return false; };
    if !binding(before, "launch_runtime").is_some_and(|value| cloned_from(value, "runtime"))
        || !binding(before, "launch_completion").is_some_and(|value| cloned_from(value, "completion"))
    { return false; }
    let Some(syn::Expr::Closure(launch)) = dispatch.args.last() else { return false; };
    let syn::Expr::Block(launch) = launch.body.as_ref() else { return false; };
    let Some((syn::Stmt::Expr(syn::Expr::Match(prepared), None), bindings)) = launch.block.stmts.split_last()
        else { return false; };
    if !named(&prepared.expr, "prepared")
        || !binding(bindings, "runtime").is_some_and(|value| named(value, "launch_runtime"))
        || !binding(bindings, "completion").is_some_and(|value| named(value, "launch_completion"))
    { return false; }
    let mut schedules = prepared.arms.iter().filter(|arm| {
        matches!(&arm.pat, syn::Pat::Struct(pattern)
            if path_is(&pattern.path, "PreparedBackendFire", "Schedule"))
    });
    let Some(schedule) = schedules.next() else { return false; };
    if schedules.next().is_some() || schedule.guard.is_some() { return false; }
    let syn::Expr::Block(schedule) = schedule.body.as_ref() else { return false; };
    let Some(syn::Stmt::Expr(syn::Expr::MethodCall(accepted), None)) = schedule.block.stmts.last()
        else { return false; };
    if accepted.method != "is_ok" || !accepted.args.is_empty() { return false; }
    let syn::Expr::MethodCall(spawn) = accepted.receiver.as_ref() else { return false; };
    if spawn.method != "spawn" || spawn.args.len() != 1 { return false; }
    let Some(syn::Expr::Closure(worker)) = spawn.args.first() else { return false; };
    let syn::Expr::Block(worker) = worker.body.as_ref() else { return false; };
    let [result, syn::Stmt::Expr(syn::Expr::Call(callback), Some(_))] = worker.block.stmts.as_slice()
        else { return false; };
    let Some(syn::Expr::MethodCall(attempt)) = binding(std::slice::from_ref(result), "result")
        else { return false; };
    if attempt.method != "run_server_scheduled_automation_attempt_with"
        || !named(&attempt.receiver, "runtime") || attempt.args.len() != 5
    { return false; }
    let args = attempt.args.iter().collect::<Vec<_>>();
    if !borrowed(args[0], "agent_id") || !borrowed(args[1], "automation_id")
        || !named(args[2], "run_uuid") || !matches!(args[4], syn::Expr::Closure(_))
        || !matches!(args[3], syn::Expr::MethodCall(value)
            if value.method == "map" && named(&value.receiver, "scheduled_for_ms")
                && value.args.len() == 1 && matches!(value.args.first(), Some(syn::Expr::Closure(_))))
    { return false; }
    if !named(&callback.func, "completion") || callback.args.len() != 1 { return false; }
    let Some(syn::Expr::Call(mapper)) = callback.args.first() else { return false; };
    named(&mapper.func, "automation_fire_completion") && mapper.args.len() == 1
        && mapper.args.first().is_some_and(|value| named(value, "result"))
}

#[test]
fn shipping_backend_schedule_returns_typed_attempt_to_the_actual_completion_callback() {
    assert!(backend_schedule_attempt_reaches_completion(SHIPPING_HOST),
        "the spawned shipping schedule worker must forward its actual typed attempt to the actual backend completion callback");
}

#[test]
fn typed_backend_attempt_wiring_oracle_rejects_wrong_owners_results_and_dead_calls() {
    let worker = "let result = runtime.run_server_scheduled_automation_attempt_with(&agent_id, &automation_id, run_uuid, scheduled_for_ms.map(|value| value as f64), |_| Ok(())); completion(automation_fire_completion(result));";
    let fixture = |worker: &str| format!(r#"
fn main() {{}}
fn dispatch_checked_production_backend_fire() {{}}
impl UnifiedGatewayApi {{
    fn dispatch_production_backend_fire(&self, fire: Fire, completion: Callback) -> bool {{
        let runtime = self.transcript_manager.automation_runtime();
        let launch_runtime = Arc::clone(&runtime);
        let launch_completion = Arc::clone(&completion);
        dispatch_checked_production_backend_fire(runtime.as_ref(), &fire, definition_time_zone.as_deref(), completion.as_ref(), |_| {{}}, |prepared, target| {{
            let runtime = launch_runtime;
            let completion = launch_completion;
            match prepared {{
                PreparedBackendFire::Schedule {{ agent_id, automation_id, run_uuid, scheduled_for_ms }} => {{
                    thread::Builder::new().spawn(move || {{ {worker} }}).is_ok()
                }}
                _ => false,
            }}
        }})
    }}
}}
#[cfg(test)]
mod tests {{}}
"#);
    assert!(backend_schedule_attempt_reaches_completion(&fixture(worker)));
    for wrong in [
        worker.replace("runtime.run_server", "unbound_runtime.run_server"),
        worker.replace("run_server_scheduled_automation_attempt_with", "run_server_scheduled_automation_with"),
        worker.replace("&agent_id", "&other_agent"),
        worker.replace("run_uuid,", "other_run_uuid,"),
        worker.replace("completion(automation_fire_completion", "other_completion(automation_fire_completion"),
        worker.replace("automation_fire_completion(result)", "automation_fire_completion(unrelated_result)"),
        worker.replace("let result =", "let _ignored ="),
        format!("let _unused = || {{ {worker} }};"),
    ] {
        let source = fixture(&wrong);
        assert!(syn::parse_file(&source).is_ok(), "negative remains syntactically valid");
        assert!(!backend_schedule_attempt_reaches_completion(&source), "false typed shipping edge: {source}");
    }
    for (from, to) in [
        ("Arc::clone(&runtime)", "Arc::clone(&unbound_runtime)"),
        ("Arc::clone(&completion)", "Arc::clone(&other_completion)"),
        ("let runtime = launch_runtime;", "let runtime = unbound_runtime;"),
        ("let completion = launch_completion;", "let completion = other_completion;"),
    ] {
        assert!(!backend_schedule_attempt_reaches_completion(&fixture(worker).replace(from, to)));
    }
}

#[test]
fn shipping_gateway_dispatches_desktop_mcp_to_the_existing_service_before_fallback() {
    assert!(desktop_mcp_dispatch_precedes_host_fallback(SHIPPING_HOST),
        "shipping UnifiedGatewayApi must return the existing MCP service result before Host fallback");
}

#[test]
fn desktop_mcp_dispatch_oracle_rejects_wrong_owner_ignored_result_and_dead_delegation() {
    let delegate = "if let Some(result) = self.mcp_service.dispatch_desktop_command(method, &args) { return result.map_err(GatewayCommandError::Internal); }";
    let native_fallback = "call_native_attachments_or_host_lane(&self.host_tx, &self.attachments, method, args)";
    let fixture = |body: &str| format!(
        "fn main() {{}}\nimpl GatewayApi for UnifiedGatewayApi {{ fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {{ {body} }} }}\n#[cfg(test)]\nmod tests {{}}\n"
    );
    for fallback in ["call_host_lane(&self.host_tx, method, args)", native_fallback] {
        assert!(desktop_mcp_dispatch_precedes_host_fallback(&fixture(&format!("{delegate} {fallback}"))));
        for body in [
            format!("{} {fallback}", delegate.replace("self.mcp_service", "self.other_service")),
            format!("if let Some(result) = self.mcp_service.dispatch_desktop_command(method, &args) {{ let _ = result; }} {fallback}"),
            format!("let _unused = || {{ {delegate} }}; {fallback}"),
            format!("let early = {fallback}; {delegate} early"),
            format!("// {delegate}\n{fallback}"),
        ] {
            let source = fixture(&body);
            assert!(syn::parse_file(&source).is_ok(), "negative control must remain syntactically valid");
            assert!(!desktop_mcp_dispatch_precedes_host_fallback(&source),
                "non-delegating production shape passed: {source}");
        }
        let test_only = fixture(fallback).replace("mod tests {}", &format!("mod tests {{ fn fixture() {{ {delegate} }} }}"));
        assert!(!desktop_mcp_dispatch_precedes_host_fallback(&test_only));
    }
    for wrong in [
        native_fallback.replace("call_native_attachments_or_host_lane", "unbound_fallback"),
        native_fallback.replace("&self.host_tx", "&self.other_host_tx"),
        native_fallback.replace("&self.host_tx", "&mut self.host_tx"),
        native_fallback.replace("&self.attachments", "&self.other_attachments"),
        native_fallback.replace("&self.attachments", "self.attachments"),
        native_fallback.replace("&self.attachments", "&mut self.attachments"),
        native_fallback.replace("method, args", "other_method, args"),
        native_fallback.replace("method, args", "method, other_args"),
        native_fallback.replace("&self.attachments, ", ""),
        native_fallback.replace("method, args", "method, args, extra"),
    ] {
        let source = fixture(&format!("{delegate} {wrong}"));
        assert!(syn::parse_file(&source).is_ok(), "negative control must remain syntactically valid");
        assert!(!desktop_mcp_dispatch_precedes_host_fallback(&source),
            "unbound preview-or-lane fallback passed: {source}");
    }
}

#[test]
fn desktop_uninstall_has_its_own_native_registry_group_without_changing_the_frozen_surface() {
    use mahayana_host_runtime::host_gateway_api::{
        FABUSHI_NATIVE_BOX_MCP_GATEWAY_METHODS, FABUSHI_NATIVE_MCP_MANAGEMENT_GATEWAY_METHODS,
        FROZEN_HOST_GATEWAY_METHODS, HostGatewayOwner, host_gateway_owner,
    };
    assert_eq!(FROZEN_HOST_GATEWAY_METHODS.len(), 122);
    assert_eq!(FABUSHI_NATIVE_MCP_MANAGEMENT_GATEWAY_METHODS, &["uninstallMcpPlugin"]);
    assert_eq!(host_gateway_owner("uninstallMcpPlugin"), Some(HostGatewayOwner::Mcp));
    assert!(!FROZEN_HOST_GATEWAY_METHODS.contains(&"uninstallMcpPlugin"));
    assert!(!FABUSHI_NATIVE_BOX_MCP_GATEWAY_METHODS.contains(&"uninstallMcpPlugin"));
    assert_eq!(FABUSHI_NATIVE_BOX_MCP_GATEWAY_METHODS,
        &["loadBoxMcpServers", "listBoxMcpToolsRaw", "executeBoxMcpToolRaw"]);
    for method in ["uninstallMcpPlugin.extra", "UninstallMcpPlugin", "uninstallMcpPlugin "] {
        assert_eq!(host_gateway_owner(method), None);
    }
}
