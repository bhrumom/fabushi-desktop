use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, MemoryService, agent_memory_has_content,
    get_agent_memory_dir, memory_id_for, normalize_memory_content, parse_facts,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-memory-service-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn memory_fact_parser_matches_frozen_markdown_contract() {
    let facts = parse_facts(
        "# About the user\n\n- (2026-09-24)  User   likes   tea  \n- (bad-date) ignored\n- (2026-13-01) invalid month\n",
        MemoryKind::Profile,
    );
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].date, "2026-09-24");
    assert_eq!(facts[0].content, "User likes tea");
    assert_eq!(facts[0].kind, MemoryKind::Profile);
    assert_eq!(
        normalize_memory_content("  many   spaces\nacross\tlines  "),
        "many spaces across lines"
    );
}

#[test]
fn durable_memory_content_requires_a_valid_fact_in_profile_or_log() {
    let root = temp_root("content");
    let agent = root.join("agent");
    let memory = get_agent_memory_dir(&agent);
    fs::create_dir_all(memory.join("log")).expect("memory dirs");

    fs::write(
        memory.join("profile.md"),
        "# About the user\n\n<!-- metadata only -->\n",
    )
    .expect("empty profile");
    fs::write(memory.join("log").join("2026-09.md"), "not a fact\n")
        .expect("empty log");
    assert!(!agent_memory_has_content(&agent));

    fs::write(
        memory.join("log").join("2026-09.md"),
        "# Memory log\n\n- (2026-09-24) Planning a release\n",
    )
    .expect("fact");
    assert!(agent_memory_has_content(&agent));

    let _ = fs::remove_dir_all(root);
}


#[test]
fn file_memory_store_round_trips_recall_dedupe_remove_and_clear() {
    let root = temp_root("store");
    let agent = root.join("agent");
    let store = FileMemoryStore::new(get_agent_memory_dir(&agent));
    let day_one = chrono::DateTime::parse_from_rfc3339("2026-09-23T00:00:00Z")
        .expect("day one")
        .timestamp_millis();
    let day_two = chrono::DateTime::parse_from_rfc3339("2026-09-24T00:00:00Z")
        .expect("day two")
        .timestamp_millis();

    let profile = store
        .add_memory("  User   likes tea  ", day_one, MemoryKind::Profile)
        .expect("add profile")
        .expect("new profile");
    let recent = store
        .add_memory("Planning a release", day_two, MemoryKind::Log)
        .expect("add recent")
        .expect("new recent");
    assert_eq!(profile.id, memory_id_for("User likes tea"));
    assert_eq!(recent.id, memory_id_for("Planning a release"));
    assert!(
        store
            .add_memory("user likes TEA", day_two, MemoryKind::Log)
            .expect("dedupe")
            .is_none()
    );

    let recalled = store.recall(20);
    assert_eq!(recalled.profile, vec![profile.clone()]);
    assert_eq!(recalled.recent, vec![recent.clone()]);
    assert_eq!(store.list_memories(10), vec![profile.clone(), recent.clone()]);
    assert_eq!(store.count_memories(), 2);
    assert!(store.has_memories());

    assert!(store.remove_memory_by_content("planning A release").expect("remove"));
    assert!(!store.remove_memory("missing").expect("missing remove"));
    assert_eq!(store.list_memories(10), vec![profile]);

    store.clear_memories().expect("clear");
    assert!(!store.has_memories());
    assert_eq!(store.recall(20).profile, Vec::new());
    assert_eq!(store.recall(20).recent, Vec::new());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn memory_service_creates_real_agent_scoped_store() {
    let root = temp_root("service");
    let service = MemoryService::new(&root);
    let agent_dir = root.join("agent-a");
    let store = service.create_agent_store(&agent_dir);
    assert_eq!(service.agents_root_dir(), root.as_path());
    assert_eq!(store.get_location(), agent_dir.join("memory"));
    assert!(!service.agent_has_content(&agent_dir));
    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_session_workers_can_share_host_memory_extension_owner() {
    let root = temp_root("shared-owner");
    let memory = std::sync::Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(&root),
    );
    let workers = ProductionSessionWorkers::with_agents_root_and_dependencies(
        &root,
        500,
        std::sync::Arc::new(|| None),
        std::sync::Arc::clone(&memory),
    );
    assert!(std::sync::Arc::ptr_eq(&workers.memory_service(), &memory));
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
