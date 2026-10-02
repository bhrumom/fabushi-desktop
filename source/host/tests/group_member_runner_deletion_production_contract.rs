use std::fs;
use std::sync::Arc;

use mahayana_host_runtime::extensions::transcript::runner_registry::TranscriptRunnerRegistry;
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedProviderTaskRegistry;

#[test]
fn group_member_registry_is_independent_from_direct_runner_registry() {
    let ordinary_tasks = Arc::new(RoutedProviderTaskRegistry::default());
    let registry = TranscriptRunnerRegistry::new(ordinary_tasks);

    let direct = registry
        .register_routed_provider("agent-a", "turn-1")
        .expect("direct runner");
    let group = registry
        .register_group_member("agent-a", "group-member-1")
        .expect("group member runner");

    assert_eq!(
        registry.active_group_member_stream_ids_for_agent("agent-a"),
        vec!["group-member-1".to_string()]
    );
    assert_eq!(
        registry.active_stream_ids_for_agent("agent-a"),
        vec!["group-member-1".to_string(), "turn-1".to_string()]
    );

    assert_eq!(registry.cancel_agent("agent-a", "direct cancelled"), 1);
    assert!(direct.is_cancelled());
    assert!(!group.is_cancelled());

    assert_eq!(
        registry.cancel_group_member_agent("agent-a", "agent deleted"),
        1
    );
    assert!(group.is_cancelled());

    registry.finish_routed_provider("turn-1");
    registry.finish_routed_provider("group-member-1");
    assert!(registry.active_stream_ids_for_agent("agent-a").is_empty());
}

#[test]
fn shipping_group_member_turn_and_deletion_use_the_independent_owner() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let main = fs::read_to_string(manifest_dir.join("app/src/main.rs")).expect("shipping main");

    assert!(main.contains(
        "runner_registry.register_group_member(&agent_id, &stream_id)"
    ));
    assert!(main.contains(
        ".active_group_member_stream_ids_for_agent(agent_id)"
    ));
    assert!(main.contains(
        "runner_registry.cancel_group_member_agent(agent_id, \"agent deleted\")"
    ));
    assert!(main.contains(
        "runner_registry.cancel_agent(agent_id, \"agent deleted\")"
    ));
    assert!(main.contains(
        "runner_registry.active_stream_ids_for_agent(agent_id).is_empty()"
    ));
}
