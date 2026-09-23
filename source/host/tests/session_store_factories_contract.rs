use std::path::Path;

use mahayana_host_runtime::extensions::session::session_store_factories::{
    NO_SESSION_MEMORY, automation_store_for_db_path, automation_store_location_for_db_path,
    channel_store_for_db_path, workflow_store_locations_for_db_path,
};

#[test]
fn frozen_store_factory_paths_follow_db_agent_and_sand_roots() {
    let db = Path::new("/tmp/sand/agents/agent-a/store.db");

    assert_eq!(
        automation_store_location_for_db_path(db),
        Path::new("/tmp/sand/agents/agent-a/automations")
    );
    assert_eq!(
        automation_store_for_db_path(db).get_location(),
        Path::new("/tmp/sand/agents/agent-a/automations")
    );

    let workflows = workflow_store_locations_for_db_path(db);
    assert_eq!(workflows.agent_dir, Path::new("/tmp/sand/agents/agent-a"));
    assert_eq!(
        workflows.global_workflows_dir,
        Path::new("/tmp/sand/workflows")
    );

    assert_eq!(
        channel_store_for_db_path(db).get_location(),
        Path::new("/tmp/sand/agents/agent-a/channels")
    );
}

#[test]
fn no_session_memory_matches_frozen_unavailable_store_contract() {
    let memory = NO_SESSION_MEMORY.create_agent_store();
    let recall = memory.recall();
    assert!(recall.profile.is_empty());
    assert!(recall.recent.is_empty());
    assert!(memory.list_memories().is_empty());
    assert!(memory.add_memory().is_none());
    assert!(!memory.remove_memory_by_content());
    assert!(memory.get_location().is_none());
    memory.set_on_change();
    assert!(!memory.remove_memory());
    memory.clear_memories();
    assert!(!NO_SESSION_MEMORY.agent_has_content(Path::new("/tmp/agent")));
}
