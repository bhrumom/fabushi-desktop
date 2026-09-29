use std::collections::BTreeSet;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, MemoryOrigin, MemoryService, MemorySynthesisBridge,
    agent_memory_has_content, get_agent_memory_dir, get_project_memory_shard_dir,
    get_user_memory_shard_dir, memory_id_for, normalize_memory_content, parse_facts,
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


#[test]
fn user_memory_store_aggregates_frozen_shards_and_recent_order() {
    let root = temp_root("user-shards");
    let service = MemoryService::new_with_sand_root(&root, root.join("agents"));
    let day_one = chrono::DateTime::parse_from_rfc3339("2026-09-20T00:00:00Z")
        .expect("day one")
        .timestamp_millis();
    let day_two = chrono::DateTime::parse_from_rfc3339("2026-09-21T00:00:00Z")
        .expect("day two")
        .timestamp_millis();

    FileMemoryStore::new(get_user_memory_shard_dir(&root, "agent-a"))
        .add_memory("shared profile A", day_one, MemoryKind::Profile)
        .expect("profile a");
    FileMemoryStore::new(get_user_memory_shard_dir(&root, "agent-a"))
        .add_memory("older event", day_one, MemoryKind::Log)
        .expect("event a");
    FileMemoryStore::new(get_user_memory_shard_dir(&root, "agent-b"))
        .add_memory("shared profile B", day_two, MemoryKind::Profile)
        .expect("profile b");
    FileMemoryStore::new(get_user_memory_shard_dir(&root, "agent-b"))
        .add_memory("newer event", day_two, MemoryKind::Log)
        .expect("event b");

    let user = service.user_memory_store(
        "agent-a",
        Arc::new(|id| format!("name-{id}")),
    );
    assert_eq!(user.get_location(), root.join("user-memory"));
    assert_eq!(
        user.get_own_shard_location(),
        root.join("user-memory").join("agents").join("agent-a")
    );
    let recalled = user.recall(50, 20);
    assert_eq!(recalled.profile.len(), 2);
    assert_eq!(
        recalled.profile.iter().map(|item| item.agent_id.as_str()).collect::<Vec<_>>(),
        vec!["agent-a", "agent-b"]
    );
    assert_eq!(
        recalled.recent.iter().map(|item| item.memory.content.as_str()).collect::<Vec<_>>(),
        vec!["newer event", "older event"]
    );
    assert_eq!(recalled.recent[0].agent_name, "name-agent-b");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_memory_store_reads_membership_shards_and_applies_global_cap() {
    let root = temp_root("project-shards");
    let agents_root = root.join("agents");
    let service = MemoryService::new_with_sand_root(&root, &agents_root);
    let own_agent = agents_root.join("owner");
    let membership = service.create_project_membership(&own_agent);
    let mut projects = BTreeSet::new();
    projects.insert("alpha".to_string());
    projects.insert("beta".to_string());
    membership.write(&projects).expect("write membership");

    let first = chrono::DateTime::parse_from_rfc3339("2026-09-20T00:00:00Z")
        .expect("first")
        .timestamp_millis();
    let second = chrono::DateTime::parse_from_rfc3339("2026-09-22T00:00:00Z")
        .expect("second")
        .timestamp_millis();
    FileMemoryStore::new(get_project_memory_shard_dir(&root, "alpha", "agent-a"))
        .add_memory("alpha memory", first, MemoryKind::Log)
        .expect("alpha memory");
    FileMemoryStore::new(get_project_memory_shard_dir(&root, "beta", "agent-b"))
        .add_memory("beta memory", second, MemoryKind::Log)
        .expect("beta memory");

    let project = service.project_memory_store(
        "owner",
        membership,
        Arc::new(|id| format!("display-{id}")),
    );
    assert_eq!(project.get_location(), root.join("projects"));
    let recalled = project.recall(50, 20, 1);
    assert_eq!(recalled.len(), 1);
    assert_eq!(recalled[0].project, "beta");
    assert_eq!(recalled[0].agent_id, "agent-b");
    assert_eq!(recalled[0].agent_name, "display-agent-b");
    assert_eq!(recalled[0].memory.content, "beta memory");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn memory_service_frozen_facade_emits_only_for_real_mutations_and_disposes_listeners() {
    let root = temp_root("facade");
    let service = MemoryService::new(&root);
    let day = chrono::DateTime::parse_from_rfc3339("2026-09-24T00:00:00Z")
        .expect("day")
        .timestamp_millis();
    let record = service
        .store_for_agent("agent-a")
        .add_memory("mutable memory", day, MemoryKind::Profile)
        .expect("add")
        .expect("new");

    let emissions = Arc::new(AtomicUsize::new(0));
    let emissions_for_listener = Arc::clone(&emissions);
    let unsubscribe = service.subscribe(Arc::new(move || {
        emissions_for_listener.fetch_add(1, Ordering::SeqCst);
    }));

    assert_eq!(service.list("agent-a").len(), 1);
    service.set_active_agent(Some("agent-a".into()));
    assert_eq!(emissions.load(Ordering::SeqCst), 1);
    service.set_active_agent(Some("agent-a".into()));
    assert_eq!(emissions.load(Ordering::SeqCst), 1);
    assert_eq!(service.active_agent_id().as_deref(), Some("agent-a"));

    assert!(!service.remove("agent-a", "missing").expect("missing remove"));
    assert_eq!(emissions.load(Ordering::SeqCst), 1);
    assert!(service.remove("agent-a", &record.id).expect("remove"));
    assert_eq!(emissions.load(Ordering::SeqCst), 2);

    service
        .store_for_agent("agent-a")
        .add_memory("clear me", day, MemoryKind::Log)
        .expect("add clear");
    service.clear("agent-a").expect("clear");
    assert_eq!(emissions.load(Ordering::SeqCst), 3);

    unsubscribe();
    service.set_active_agent(None);
    assert_eq!(emissions.load(Ordering::SeqCst), 3);

    let disposed_emissions = Arc::new(AtomicUsize::new(0));
    let disposed_for_listener = Arc::clone(&disposed_emissions);
    let _subscription = service.subscribe(Arc::new(move || {
        disposed_for_listener.fetch_add(1, Ordering::SeqCst);
    }));
    service.dispose();
    service.set_active_agent(Some("agent-b".into()));
    assert_eq!(disposed_emissions.load(Ordering::SeqCst), 0);

    let _ = fs::remove_dir_all(root);
}
