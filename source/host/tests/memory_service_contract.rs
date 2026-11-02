use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::session_mutations::{
    capture_profile_mutation_files, profile_mutation_files_match,
    restore_profile_mutation_files,
};
use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, MemoryOrigin, MemoryRecoveryRecord, MemoryService,
    MemorySynthesisBridge,
    SynthesisApplyResult,
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

struct AppendReadFixture {
    root: PathBuf,
    store: FileMemoryStore,
}

impl AppendReadFixture {
    fn new(label: &str) -> Self {
        let root = temp_root(label);
        fs::create_dir(&root).expect("exclusively owned append fixture");
        let memory = root.join("memory");
        fs::create_dir(&memory).expect("memory directory");
        Self {
            root,
            store: FileMemoryStore::with_synthesis_metadata(memory, true),
        }
    }
}

impl Drop for AppendReadFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Compare raw bytes and every directory entry, including all synthesis metadata
// and any unexpected staging files. The production fact reader is not the oracle.
fn append_memory_tree(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    let mut tree = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let kind = fs::symlink_metadata(&path).expect("owned entry metadata").file_type();
        let relative = path.strip_prefix(root).expect("owned relative path").to_path_buf();
        if kind.is_dir() {
            tree.insert(relative, None);
            for entry in fs::read_dir(&path).expect("owned directory") {
                pending.push(entry.expect("owned directory entry").path());
            }
        } else {
            assert!(kind.is_file(), "fixture contains only files and directories");
            tree.insert(relative, Some(fs::read(&path).expect("raw owned file bytes")));
        }
    }
    tree
}

fn append_test_time() -> i64 {
    chrono::DateTime::parse_from_rfc3339("2026-09-24T00:00:00Z")
        .expect("append date")
        .timestamp_millis()
}

#[test]
fn file_memory_store_append_read_errors_preserve_bytes_and_allow_repair() {
    for (kind, relative) in [
        (MemoryKind::Profile, "profile.md"),
        (MemoryKind::Log, "log/2026-09.md"),
    ] {
        let fixture = AppendReadFixture::new("append-corrupt-bytes");
        let store = &fixture.store;
        let now = append_test_time();
        let history = store
            .add_memory("History retained by repair", now - 86_400_000, kind)
            .expect("seed readable history")
            .expect("new history");
        let forgotten = store
            .add_memory("Previously forgotten fact", now, kind)
            .expect("seed forgotten fact")
            .expect("new forgotten fact");
        assert!(store.remove_memory(&forgotten.id).expect("seed tombstone"));
        store.mark_temporal_review(now - 86_400_000).expect("seed temporal metadata");

        let destination = store.get_location().join(relative);
        let repaired = fs::read(&destination).expect("history bytes before corruption");
        let mut corrupt = repaired.clone();
        corrupt.extend_from_slice(b"<!-- unreadable existing history: \xff\xfe -->\n");
        fs::write(&destination, &corrupt).expect("fixture-owned corruption");
        assert_eq!(fs::read(&destination).expect("corrupt bytes remain readable"), corrupt);
        let read_error = fs::read_to_string(&destination).expect_err("invalid UTF-8 fixture");
        assert_eq!(read_error.kind(), io::ErrorKind::InvalidData);
        let before = append_memory_tree(&store.get_location());

        let error = store
            .add_memory("  New append   after repair  ", now, kind)
            .expect_err("unreadable existing destination must refuse append");
        assert_eq!(error.kind(), read_error.kind());
        assert_eq!(error.raw_os_error(), read_error.raw_os_error());
        assert_eq!(append_memory_tree(&store.get_location()), before);
        assert!(store.add_memory(" \n\t ", now, kind).expect("empty no-op").is_none());
        assert_eq!(append_memory_tree(&store.get_location()), before);

        // A newly opened handle must preserve the same still-corrupt files.
        {
            let reopened = FileMemoryStore::with_synthesis_metadata(store.get_location(), true);
            assert_eq!(append_memory_tree(&store.get_location()), before);
            let reopen_error = reopened
                .add_memory("  New append   after repair  ", now, kind)
                .expect_err("handle reopen must not repair or replace unreadable history");
            assert_eq!(reopen_error.kind(), read_error.kind());
            assert_eq!(reopen_error.raw_os_error(), read_error.raw_os_error());
            assert_eq!(append_memory_tree(&store.get_location()), before);
        }

        fs::write(&destination, &repaired).expect("fixture-owned repair");
        let appended = store
            .add_memory("  New append   after repair  ", now, kind)
            .expect("same live store retries after repair")
            .expect("new repaired append");
        let mut expected = repaired;
        expected.extend_from_slice(b"- (2026-09-24) New append after repair\n");
        assert_eq!(fs::read(&destination).expect("repaired append bytes"), expected);
        assert_eq!(appended.id, memory_id_for("New append after repair"));
        assert_eq!(appended.content, "New append after repair");
        assert_eq!(appended.created_at, now);
        assert_eq!(appended.kind, kind);
        assert!(store.list_memories(10).iter().any(|record| record.id == history.id));
        assert_eq!(
            store.snapshot_for_recovery(std::slice::from_ref(&appended))[0].origin,
            MemoryOrigin::Explicit
        );
    }
}

#[test]
fn file_memory_store_duplicate_append_checks_corrupt_destination_before_metadata() {
    for (kind, relative, existing_kind) in [
        (MemoryKind::Profile, "profile.md", MemoryKind::Log),
        (MemoryKind::Log, "log/2026-09.md", MemoryKind::Profile),
    ] {
        let fixture = AppendReadFixture::new("append-corrupt-duplicate");
        let store = &fixture.store;
        let now = append_test_time();
        let existing = store
            .add_memory("Shared duplicate fact", now, existing_kind)
            .expect("seed duplicate in readable other kind")
            .expect("new duplicate control");
        store
            .restore_records_exact(&[MemoryRecoveryRecord {
                record: existing.clone(),
                origin: MemoryOrigin::Synthesis,
                tombstoned: true,
            }])
            .expect("seed exact origin and tombstone recovery state");
        store.mark_temporal_review(now).expect("seed temporal metadata");
        let destination = store.get_location().join(relative);
        fs::create_dir_all(destination.parent().expect("destination parent")).expect("parent");
        let corrupt = b"# Existing history\n<!-- bytes: \xff\xfe -->\n";
        fs::write(&destination, corrupt).expect("corrupt declared destination");
        assert_eq!(fs::read(&destination).expect("raw corrupt destination"), corrupt);
        let read_error = fs::read_to_string(&destination).expect_err("invalid UTF-8 fixture");
        assert_eq!(read_error.kind(), io::ErrorKind::InvalidData);
        let before = append_memory_tree(&store.get_location());

        let error = store
            .add_memory(" SHARED   DUPLICATE FACT ", now, kind)
            .expect_err("dedupe must not alter metadata before destination admission");
        assert_eq!(error.kind(), read_error.kind());
        assert_eq!(append_memory_tree(&store.get_location()), before);
        let retained = store.snapshot_for_recovery(std::slice::from_ref(&existing));
        assert_eq!(retained[0].origin, MemoryOrigin::Synthesis);
        assert!(retained[0].tombstoned);

        let repaired = b"# Repaired history\n<!-- retain this comment -->\n";
        fs::write(&destination, repaired).expect("fixture-owned repair");
        assert!(
            store
                .add_memory(" SHARED   DUPLICATE FACT ", now, kind)
                .expect("readable duplicate retains existing behavior")
                .is_none()
        );
        assert_eq!(fs::read(&destination).expect("duplicate leaves repaired bytes"), repaired);
        assert_eq!(store.list_memories(10), vec![existing.clone()]);
        let explicit = store.snapshot_for_recovery(std::slice::from_ref(&existing));
        assert_eq!(explicit[0].origin, MemoryOrigin::Explicit);
        assert!(!explicit[0].tombstoned);
    }
}

#[test]
fn file_memory_store_append_preserves_missing_readable_empty_and_duplicate_controls() {
    for (kind, relative, header) in [
        (
            MemoryKind::Profile,
            "profile.md",
            "# About the user\n\n<!-- Enduring facts, one per line as \"- (YYYY-MM-DD) <fact>\". -->\n\n",
        ),
        (
            MemoryKind::Log,
            "log/2026-09.md",
            "# Memory log\n\n<!-- Dated facts, one per line as \"- (YYYY-MM-DD) <fact>\". -->\n\n",
        ),
    ] {
        let fixture = AppendReadFixture::new("append-controls");
        let store = &fixture.store;
        let now = append_test_time();
        let destination = store.get_location().join(relative);
        assert_eq!(
            fs::read_to_string(&destination).expect_err("missing first-write destination").kind(),
            io::ErrorKind::NotFound
        );
        let empty = append_memory_tree(&store.get_location());
        assert!(store.add_memory(" \t\n ", now, kind).expect("empty input").is_none());
        assert_eq!(append_memory_tree(&store.get_location()), empty);

        let first = store
            .add_memory("  First   durable\n fact  ", now - 86_400_000, kind)
            .expect("missing destination admits first write")
            .expect("new first fact");
        let first_text = format!("{header}- (2026-09-23) First durable fact\n");
        assert_eq!(fs::read_to_string(&destination).expect("first file"), first_text);
        assert_eq!(first.id, memory_id_for("First durable fact"));
        assert_eq!(first.content, "First durable fact");
        assert_eq!(first.created_at, now - 86_400_000);
        assert_eq!(first.kind, kind);

        let readable = format!("{first_text}<!-- preserve this comment without a trailing newline -->");
        fs::write(&destination, &readable).expect("readable custom comment");
        let second = store
            .add_memory(" Later   note ", now, kind)
            .expect("readable destination admits append")
            .expect("new second fact");
        assert_eq!(
            fs::read_to_string(&destination).expect("appended readable file"),
            format!("{readable}\n- (2026-09-24) Later note\n")
        );
        assert_eq!(second.id, memory_id_for("Later note"));
        assert_eq!(second.created_at, now);
        assert_eq!(second.kind, kind);
        assert_eq!(store.list_memories(10), vec![second, first]);

        let before_noops = append_memory_tree(&store.get_location());
        assert!(
            store.add_memory(" FIRST DURABLE FACT ", now, kind).expect("duplicate").is_none()
        );
        assert!(store.add_memory(" \n ", now, kind).expect("empty append").is_none());
        assert_eq!(append_memory_tree(&store.get_location()), before_noops);
    }
}

#[test]
fn file_memory_store_append_directory_read_error_is_preserved() {
    for (kind, relative) in [
        (MemoryKind::Profile, "profile.md"),
        (MemoryKind::Log, "log/2026-09.md"),
    ] {
        let fixture = AppendReadFixture::new("append-directory-io");
        let store = &fixture.store;
        let now = append_test_time();
        let destination = store.get_location().join(relative);
        fs::create_dir_all(&destination).expect("directory at append destination");
        let sentinel = destination.join("owned-history.bin");
        fs::write(&sentinel, b"owned directory bytes\x00\xff").expect("directory sentinel");
        store.mark_temporal_review(now).expect("seed metadata");
        // This proves a real I/O read failure even when the test runs as root.
        let read_error = fs::read_to_string(&destination).expect_err("directory cannot be text");
        assert_ne!(read_error.kind(), io::ErrorKind::NotFound);
        assert!(read_error.raw_os_error().is_some());
        let before = append_memory_tree(&store.get_location());

        let error = store
            .add_memory("Append after directory repair", now, kind)
            .expect_err("destination I/O read error must propagate before staging");
        assert_eq!(error.kind(), read_error.kind());
        assert_eq!(error.raw_os_error(), read_error.raw_os_error());
        assert_eq!(append_memory_tree(&store.get_location()), before);

        fs::remove_file(&sentinel).expect("remove only fixture-owned sentinel");
        fs::remove_dir(&destination).expect("remove only fixture-owned empty directory");
        let repaired = "# Repaired history\n<!-- retain repaired comment -->\n";
        fs::write(&destination, repaired).expect("fixture-owned directory repair");
        assert!(
            store
                .add_memory("Append after directory repair", now, kind)
                .expect("same live store accepts repaired path")
                .is_some()
        );
        assert_eq!(
            fs::read_to_string(&destination).expect("repaired directory append"),
            format!("{repaired}- (2026-09-24) Append after directory repair\n")
        );
    }
}

#[test]
fn exact_memory_recovery_preserves_synthesis_metadata() {
    let root = temp_root("exact-recovery");
    let agent = root.join("agent");
    let store = FileMemoryStore::with_synthesis_metadata(get_agent_memory_dir(&agent), true);
    let record = store
        .add_memory("A durable preference", 1_000, MemoryKind::Profile)
        .expect("add memory")
        .expect("new memory");
    let captured = store.snapshot_for_recovery(std::slice::from_ref(&record));
    assert_eq!(captured[0].origin, MemoryOrigin::Explicit);

    store.remove_memory(&record.id).expect("remove memory");
    store.restore_records_exact(&captured).expect("restore memory");

    let restored = store.prepare_synthesis();
    assert_eq!(restored.memories.len(), 1);
    assert_eq!(restored.memories[0].origin, MemoryOrigin::Explicit);
    assert_eq!(restored.memories[0].content, record.content);
    assert!(!captured[0].tombstoned);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn profile_mutation_file_snapshots_are_exact_and_allowlisted() {
    let root = temp_root("profile-file-recovery");
    let agent = root.join("agent");
    fs::create_dir_all(&agent).expect("agent dir");
    fs::write(agent.join("profile.json"), b"old-profile\n").expect("profile");
    fs::write(agent.join("avatar.png"), [1_u8, 2, 3]).expect("avatar");
    let before = capture_profile_mutation_files(&agent, true).expect("capture before");
    fs::write(agent.join("profile.json"), b"new-profile\n").expect("new profile");
    fs::remove_file(agent.join("avatar.png")).expect("remove avatar");
    let after = capture_profile_mutation_files(&agent, true).expect("capture after");
    assert!(profile_mutation_files_match(&agent, &after));
    assert!(!profile_mutation_files_match(&agent, &before));
    restore_profile_mutation_files(&agent, &before).expect("restore before");
    assert!(profile_mutation_files_match(&agent, &before));
    assert_eq!(fs::read(agent.join("profile.json")).expect("restored profile"), b"old-profile\n");
    assert_eq!(fs::read(agent.join("avatar.png")).expect("restored avatar"), vec![1, 2, 3]);

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

#[test]
fn synthesis_apply_fails_closed_until_an_owner_gate_is_bound() {
    let root = temp_root("synthesis-gate");
    let service = MemoryService::new(&root);
    let store = service.store_for_agent("agent-gated");
    let snapshot = store.prepare_synthesis();
    assert_eq!(
        service.apply_synthesis_with_gate("agent-gated", &store, &snapshot, &[], 1),
        SynthesisApplyResult::Invalid
    );

    let calls = Arc::new(AtomicUsize::new(0));
    let observed_calls = Arc::clone(&calls);
    service.set_synthesis_apply_gate(Some(Arc::new(move |_, _, _, _, _| {
        observed_calls.fetch_add(1, Ordering::SeqCst);
        SynthesisApplyResult::Stale
    })));
    assert_eq!(
        service.apply_synthesis_with_gate("agent-gated", &store, &snapshot, &[], 2),
        SynthesisApplyResult::Stale
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn bound_synthesis_gate_rejects_a_noncanonical_store_path() {
    let root = temp_root("synthesis-canonical-store");
    let memory = Arc::new(MemoryService::new(&root));
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root_and_dependencies(
        &root,
        500,
        Arc::new(|| None),
        Arc::clone(&memory),
    ));
    workers.bind_memory_synthesis_gate();

    let canonical = memory.store_for_agent("agent-gated");
    let foreign = FileMemoryStore::new(root.join("foreign-memory"));
    let canonical_snapshot = canonical.prepare_synthesis();
    let foreign_snapshot = foreign.prepare_synthesis();
    assert_eq!(
        memory.apply_synthesis_with_gate(
            "agent-gated",
            &canonical,
            &canonical_snapshot,
            &[],
            1,
        ),
        SynthesisApplyResult::Committed
    );
    assert_eq!(
        memory.apply_synthesis_with_gate(
            "agent-gated",
            &foreign,
            &foreign_snapshot,
            &[],
            2,
        ),
        SynthesisApplyResult::Invalid
    );

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
fn existing_agent_store_tracks_dreaming_gate_enable_and_disable_dynamically() {
    let root = temp_root("dynamic-dreaming-gate");
    let agent_dir = root.join("agent-a");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let service = MemoryService::new(&root);
    let store = service.create_agent_store(&agent_dir);
    let created_at = 1_900_000_000_000;

    let before_gate = store
        .add_memory("before authenticated gate", created_at, MemoryKind::Profile)
        .expect("add before gate")
        .expect("new before gate memory");
    assert_eq!(
        store
            .prepare_synthesis()
            .memories
            .iter()
            .find(|memory| memory.id == before_gate.id)
            .map(|memory| memory.origin),
        Some(MemoryOrigin::Legacy)
    );

    let bridge = Arc::new(RecordingSynthesisBridge {
        enabled: true,
        ..RecordingSynthesisBridge::default()
    });
    let bridge_trait: Arc<dyn MemorySynthesisBridge> = bridge.clone();
    service.set_synthesis_bridge(Arc::downgrade(&bridge_trait));
    drop(bridge_trait);

    let after_enable = store
        .add_memory("after authenticated gate", created_at + 1, MemoryKind::Profile)
        .expect("add after enable")
        .expect("new enabled memory");
    assert_eq!(
        store
            .prepare_synthesis()
            .memories
            .iter()
            .find(|memory| memory.id == after_enable.id)
            .map(|memory| memory.origin),
        Some(MemoryOrigin::Explicit)
    );

    drop(bridge);
    assert!(!service.synthesis_enabled());
    let after_disable = store
        .add_memory("after gate disable", created_at + 2, MemoryKind::Profile)
        .expect("add after disable")
        .expect("new disabled memory");
    assert_eq!(
        store
            .prepare_synthesis()
            .memories
            .iter()
            .find(|memory| memory.id == after_disable.id)
            .map(|memory| memory.origin),
        Some(MemoryOrigin::Legacy)
    );

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
    FileMemoryStore::new(get_user_memory_shard_dir(&root, "agent-a"))
        .add_memory("shared winner", day_one, MemoryKind::Profile)
        .expect("shared profile older");
    FileMemoryStore::new(get_user_memory_shard_dir(&root, "agent-b"))
        .add_memory("SHARED WINNER", day_two, MemoryKind::Profile)
        .expect("shared profile newer");
    FileMemoryStore::new(get_user_memory_shard_dir(&root, "agent-a"))
        .add_memory("[episode] durable older episode", day_one, MemoryKind::Log)
        .expect("ranked episode");
    FileMemoryStore::new(get_user_memory_shard_dir(&root, "agent-b"))
        .add_memory("[note] newer low importance note", day_two, MemoryKind::Log)
        .expect("ranked note");

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
    assert_eq!(recalled.profile.len(), 3);
    assert_eq!(
        recalled.profile.iter().map(|item| item.agent_id.as_str()).collect::<Vec<_>>(),
        vec!["agent-b", "agent-b", "agent-a"]
    );
    assert_eq!(
        recalled
            .profile
            .iter()
            .filter(|item| item.memory.content.eq_ignore_ascii_case("shared winner"))
            .count(),
        1
    );
    assert_eq!(
        recalled
            .profile
            .iter()
            .find(|item| item.memory.content.eq_ignore_ascii_case("shared winner"))
            .map(|item| item.agent_id.as_str()),
        Some("agent-b")
    );
    assert_eq!(
        recalled.recent.iter().map(|item| item.memory.content.as_str()).collect::<Vec<_>>(),
        vec![
            "[episode] durable older episode",
            "newer event",
            "older event",
            "[note] newer low importance note",
        ]
    );
    assert_eq!(recalled.recent[0].agent_name, "name-agent-a");

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


#[test]
fn project_prompt_recall_uses_frontmatter_names_and_frozen_top_three_ordering() {
    let root = temp_root("project-prompt");
    let agents_root = root.join("agents");
    let service = MemoryService::new_with_sand_root(&root, &agents_root);
    let own_agent = agents_root.join("owner");
    let membership = service.create_project_membership(&own_agent);
    let projects = BTreeSet::from([
        "alpha".to_string(),
        "beta".to_string(),
        "gamma".to_string(),
        "zeta".to_string(),
    ]);
    membership.write(&projects).expect("write membership");

    for (slug, name) in [
        ("alpha", "Alpha Project"),
        ("beta", "Beta Project"),
        ("gamma", "Gamma Project"),
        ("zeta", "Zeta Project"),
    ] {
        let dir = root.join("projects").join(slug);
        fs::create_dir_all(&dir).expect("project dir");
        fs::write(
            dir.join("project.md"),
            format!("---\nname: {name}\ndescription: fixture\n---\n"),
        )
        .expect("project metadata");
    }

    let alpha_at = chrono::DateTime::parse_from_rfc3339("2026-09-20T00:00:00Z")
        .expect("alpha time")
        .timestamp_millis();
    let beta_at = chrono::DateTime::parse_from_rfc3339("2026-09-23T00:00:00Z")
        .expect("beta time")
        .timestamp_millis();
    let zeta_at = chrono::DateTime::parse_from_rfc3339("2026-09-22T00:00:00Z")
        .expect("zeta time")
        .timestamp_millis();
    FileMemoryStore::new(get_project_memory_shard_dir(&root, "alpha", "agent-a"))
        .add_memory("alpha fact", alpha_at, MemoryKind::Log)
        .expect("alpha fact");
    FileMemoryStore::new(get_project_memory_shard_dir(&root, "beta", "agent-b"))
        .add_memory("beta fact", beta_at, MemoryKind::Profile)
        .expect("beta fact");
    FileMemoryStore::new(get_project_memory_shard_dir(&root, "beta", "agent-b"))
        .add_memory("shared project rule", beta_at, MemoryKind::Profile)
        .expect("older shared project rule");
    FileMemoryStore::new(get_project_memory_shard_dir(&root, "beta", "agent-c"))
        .add_memory("SHARED PROJECT RULE", beta_at + 86_400_000, MemoryKind::Profile)
        .expect("newer shared project rule");
    FileMemoryStore::new(get_project_memory_shard_dir(&root, "zeta", "agent-c"))
        .add_memory("zeta fact", zeta_at, MemoryKind::Log)
        .expect("zeta fact");

    let project = service.project_memory_store(
        "owner",
        membership,
        Arc::new(|id| format!("display-{id}")),
    );
    let recalled = project.recall_for_prompt(25, 10, 3);
    assert_eq!(
        recalled
            .injected
            .iter()
            .map(|block| block.slug.as_str())
            .collect::<Vec<_>>(),
        vec!["beta", "zeta", "alpha"]
    );
    assert_eq!(recalled.injected[0].name, "Beta Project");
    assert_eq!(
        recalled.injected[0].recall.profile[0].agent_name,
        "display-agent-c"
    );
    assert_eq!(
        recalled.injected[0]
            .recall
            .profile
            .iter()
            .filter(|item| item.memory.content.eq_ignore_ascii_case("shared project rule"))
            .count(),
        1
    );
    assert_eq!(
        recalled.injected[0]
            .recall
            .profile
            .iter()
            .find(|item| item.memory.content.eq_ignore_ascii_case("shared project rule"))
            .map(|item| item.agent_id.as_str()),
        Some("agent-c")
    );
    assert_eq!(
        recalled.injected[0].own_shard_dir,
        get_project_memory_shard_dir(&root, "beta", "owner")
    );
    assert_eq!(recalled.also_member_of.len(), 1);
    assert_eq!(recalled.also_member_of[0].slug, "gamma");
    assert_eq!(recalled.also_member_of[0].name, "Gamma Project");

    let _ = fs::remove_dir_all(root);
}
