use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, MemoryOrigin, MemoryService, MemorySynthesisBridge,
    agent_memory_has_content,
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


#[derive(Default)]
struct RecordingSynthesisBridge {
    enabled: bool,
    evidence: Mutex<Vec<(String, String, String)>>,
}

impl MemorySynthesisBridge for RecordingSynthesisBridge {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn record_turn(
        &self,
        agent_id: &str,
        _evidence_id: Option<String>,
        user: &str,
        assistant: &str,
        _occurred_at: i64,
    ) {
        self.evidence.lock().unwrap().push((
            agent_id.to_string(),
            user.to_string(),
            assistant.to_string(),
        ));
    }
}

#[test]
fn memory_service_dreaming_bridge_is_weak_single_owner_and_marks_explicit_memory() {
    let root = temp_root("dreaming-bridge");
    let agent_dir = root.join("agent-a");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let service = MemoryService::new(&root);
    let bridge = Arc::new(RecordingSynthesisBridge {
        enabled: true,
        ..RecordingSynthesisBridge::default()
    });
    let bridge_trait: Arc<dyn MemorySynthesisBridge> = bridge.clone();
    service.set_synthesis_bridge(Arc::downgrade(&bridge_trait));
    drop(bridge_trait);

    assert!(service.synthesis_enabled());
    assert!(service.record_memory_evidence(
        "agent-a",
        Some("evidence-1".into()),
        "user evidence",
        "assistant evidence",
        1_900_000_000_000,
    ));
    assert_eq!(
        bridge.evidence.lock().unwrap().as_slice(),
        &[(
            "agent-a".to_string(),
            "user evidence".to_string(),
            "assistant evidence".to_string(),
        )]
    );

    let explicit = service
        .store_for_agent("agent-a")
        .add_memory("explicit preference", 1_900_000_000_000, MemoryKind::Profile)
        .expect("add explicit")
        .expect("new explicit");
    let target = service
        .synthesis_target_for_agent("agent-a")
        .expect("synthesis target");
    let snapshot = target.prepare_synthesis();
    assert_eq!(
        snapshot
            .memories
            .iter()
            .find(|memory| memory.id == explicit.id)
            .map(|memory| memory.origin),
        Some(MemoryOrigin::Explicit)
    );
    assert_eq!(service.list_synthesis_targets().len(), 1);

    drop(bridge);
    assert!(!service.synthesis_enabled());
    assert!(!service.record_memory_evidence(
        "agent-a",
        None,
        "ignored",
        "ignored",
        1_900_000_000_001,
    ));
    let _ = fs::remove_dir_all(root);
}
