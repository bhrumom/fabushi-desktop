use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, MemoryOrigin, SynthesisApplyResult, SynthesisChange,
};

fn temp_store() -> (std::path::PathBuf, FileMemoryStore) {
    let root = std::env::temp_dir().join(format!(
        "fabushi-memory-synthesis-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&root).unwrap();
    let store = FileMemoryStore::with_synthesis_metadata(root.join("memory"), true);
    (root, store)
}

struct SynthesisAppendFixture {
    root: PathBuf,
    store: FileMemoryStore,
    now: i64,
}

impl SynthesisAppendFixture {
    fn new() -> Self {
        let (root, store) = temp_store();
        let now = chrono::DateTime::parse_from_rfc3339("2026-09-24T00:00:00Z")
            .expect("synthesis append date")
            .timestamp_millis();
        let fixture = Self { root, store, now };
        let empty = fixture.store.prepare_synthesis();
        assert_eq!(
            fixture.store.apply_synthesis(
                &empty,
                &[
                    SynthesisChange::Create {
                        content: "Existing profile history".into(),
                        kind: MemoryKind::Profile,
                    },
                    SynthesisChange::Create {
                        content: "Existing log history".into(),
                        kind: MemoryKind::Log,
                    },
                ],
                now - 86_400_000,
            ).expect("seed synthesized history"),
            SynthesisApplyResult::Committed
        );
        let previous_month = chrono::DateTime::parse_from_rfc3339("2026-08-24T00:00:00Z")
            .expect("separate explicit control month")
            .timestamp_millis();
        fixture.store
            .add_memory("Protected explicit control", previous_month, MemoryKind::Log)
            .expect("seed explicit origin")
            .expect("new explicit control");
        let deleted = fixture.store
            .add_memory("Previously deleted control", previous_month, MemoryKind::Log)
            .expect("seed deleted control")
            .expect("new deleted control");
        assert!(fixture.store.remove_memory(&deleted.id).expect("seed tombstone"));
        for relative in ["profile.md", "log/2026-09.md"] {
            let path = fixture.store.get_location().join(relative);
            let mut bytes = fs::read(&path).expect("seed history bytes");
            bytes.extend_from_slice(b"<!-- keep this fixture-owned history comment -->\n");
            fs::write(path, bytes).expect("seed preserved comment");
        }
        assert!(fixture.store.is_temporal_review_due(now));
        fixture
    }

    fn corrupt_destination(&self, relative: &str) -> (PathBuf, Vec<u8>, io::Error) {
        let destination = self.store.get_location().join(relative);
        let repaired = fs::read(&destination).expect("history before fixture corruption");
        let mut corrupt = repaired.clone();
        corrupt.extend_from_slice(b"<!-- existing unreadable bytes: \xff\xfe -->\n");
        fs::write(&destination, &corrupt).expect("fixture-owned destination corruption");
        assert_eq!(fs::read(&destination).expect("raw corrupt history"), corrupt);
        let error = fs::read_to_string(&destination).expect_err("invalid UTF-8 fixture");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        (destination, repaired, error)
    }
}

impl Drop for SynthesisAppendFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Include directory membership, raw memory bytes, both origin namespaces,
// tombstones, temporal review, and unexpected files; never parse away corruption.
fn synthesis_memory_tree(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
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

#[test]
fn synthesis_create_read_errors_preserve_memory_and_metadata() {
    for (kind, relative) in [
        (MemoryKind::Profile, "profile.md"),
        (MemoryKind::Log, "log/2026-09.md"),
    ] {
        let fixture = SynthesisAppendFixture::new();
        let store = &fixture.store;
        let (destination, repaired, read_error) = fixture.corrupt_destination(relative);
        // Corruption predates the snapshot, so a stale-snapshot rejection cannot
        // impersonate append admission. The existing fingerprint can ignore it.
        let snapshot = store.prepare_synthesis();
        assert_eq!(snapshot, store.prepare_synthesis());
        let before = synthesis_memory_tree(&store.get_location());
        let changes = [SynthesisChange::Create {
            content: "  Synthesized   after repair  ".into(),
            kind,
        }];

        let error = store.apply_synthesis(&snapshot, &changes, fixture.now)
            .expect_err("corrupt create destination must refuse before any mutation");
        assert_eq!(error.kind(), read_error.kind());
        assert_eq!(error.raw_os_error(), read_error.raw_os_error());
        assert_eq!(synthesis_memory_tree(&store.get_location()), before);
        assert!(store.is_temporal_review_due(fixture.now));

        fs::write(&destination, &repaired).expect("fixture-owned repair");
        let retry = store.prepare_synthesis();
        assert_eq!(
            store.apply_synthesis(&retry, &changes, fixture.now)
                .expect("same live store retries after repair"),
            SynthesisApplyResult::Committed
        );
        let mut expected = repaired;
        expected.extend_from_slice(b"- (2026-09-24) Synthesized after repair\n");
        assert_eq!(fs::read(&destination).expect("preserved and appended bytes"), expected);
        let final_state = store.prepare_synthesis();
        let created = final_state.memories.iter()
            .find(|memory| memory.content == "Synthesized after repair")
            .expect("new synthesized memory after repair");
        assert_eq!(created.kind, kind);
        assert_eq!(created.origin, MemoryOrigin::Synthesis);
        assert_eq!(created.created_at, fixture.now);
        assert!(final_state.memories.iter().any(|memory| {
            memory.content == "Protected explicit control" && memory.origin == MemoryOrigin::Explicit
        }));
        assert!(!store.is_temporal_review_due(fixture.now + 1));
    }
}

#[test]
fn synthesis_cross_kind_update_read_errors_preserve_source_and_metadata() {
    for (kind, relative, source_content) in [
        (MemoryKind::Profile, "profile.md", "Existing log history"),
        (MemoryKind::Log, "log/2026-09.md", "Existing profile history"),
    ] {
        let fixture = SynthesisAppendFixture::new();
        let store = &fixture.store;
        let (destination, repaired, read_error) = fixture.corrupt_destination(relative);
        let snapshot = store.prepare_synthesis();
        let source = snapshot.memories.iter()
            .find(|memory| memory.content == source_content)
            .expect("readable source remains in snapshot");
        assert_eq!(source.origin, MemoryOrigin::Synthesis);
        assert_ne!(source.kind, kind);
        let before = synthesis_memory_tree(&store.get_location());
        let changes = [SynthesisChange::Update {
            id: source.id.clone(),
            content: "Reclassified synthesized fact".into(),
            kind,
        }];

        let error = store.apply_synthesis(&snapshot, &changes, fixture.now)
            .expect_err("cross-kind destination must be readable before source removal");
        assert_eq!(error.kind(), read_error.kind());
        assert_eq!(error.raw_os_error(), read_error.raw_os_error());
        assert_eq!(synthesis_memory_tree(&store.get_location()), before);
        assert_eq!(store.prepare_synthesis(), snapshot);
        assert!(store.is_temporal_review_due(fixture.now));

        fs::write(&destination, &repaired).expect("fixture-owned repair");
        let retry = store.prepare_synthesis();
        assert_eq!(
            store.apply_synthesis(&retry, &changes, fixture.now)
                .expect("same live store retries cross-kind update"),
            SynthesisApplyResult::Committed
        );
        let mut expected = repaired;
        expected.extend_from_slice(b"- (2026-09-24) Reclassified synthesized fact\n");
        assert_eq!(fs::read(&destination).expect("destination history retained"), expected);
        let final_state = store.prepare_synthesis();
        assert!(!final_state.memories.iter().any(|memory| memory.id == source.id));
        let updated = final_state.memories.iter()
            .find(|memory| memory.content == "Reclassified synthesized fact")
            .expect("updated memory");
        assert_eq!(updated.kind, kind);
        assert_eq!(updated.origin, MemoryOrigin::Synthesis);
        assert_eq!(updated.created_at, fixture.now);
        assert!(!store.is_temporal_review_due(fixture.now + 1));
    }
}

#[test]
fn synthesis_batch_preflights_later_append_before_earlier_remove() {
    for (kind, relative, source_content) in [
        (MemoryKind::Profile, "profile.md", "Existing log history"),
        (MemoryKind::Log, "log/2026-09.md", "Existing profile history"),
    ] {
        let fixture = SynthesisAppendFixture::new();
        let store = &fixture.store;
        let (destination, repaired, read_error) = fixture.corrupt_destination(relative);
        let snapshot = store.prepare_synthesis();
        let source = snapshot.memories.iter()
            .find(|memory| memory.content == source_content)
            .expect("earlier remove has a valid readable source");
        assert_eq!(source.origin, MemoryOrigin::Synthesis);
        let changes = [
            SynthesisChange::Remove { id: source.id.clone() },
            SynthesisChange::Create {
                content: "Earlier valid synthesized append".into(),
                kind: source.kind,
            },
            SynthesisChange::Create {
                content: "Later repaired synthesized append".into(),
                kind,
            },
        ];
        let before = synthesis_memory_tree(&store.get_location());

        let error = store.apply_synthesis(&snapshot, &changes, fixture.now)
            .expect_err("all append destinations must pass before the earlier remove or create");
        assert_eq!(error.kind(), read_error.kind());
        assert_eq!(error.raw_os_error(), read_error.raw_os_error());
        assert_eq!(synthesis_memory_tree(&store.get_location()), before);
        assert_eq!(store.prepare_synthesis(), snapshot);
        assert!(store.is_temporal_review_due(fixture.now));

        fs::write(&destination, &repaired).expect("fixture-owned repair");
        let retry = store.prepare_synthesis();
        assert_eq!(
            store.apply_synthesis(&retry, &changes, fixture.now)
                .expect("same live store retries the complete valid batch"),
            SynthesisApplyResult::Committed
        );
        let mut expected = repaired;
        expected.extend_from_slice(b"- (2026-09-24) Later repaired synthesized append\n");
        assert_eq!(fs::read(&destination).expect("later destination history retained"), expected);
        let final_state = store.prepare_synthesis();
        assert!(!final_state.memories.iter().any(|memory| memory.id == source.id));
        for (content, expected_kind) in [
            ("Earlier valid synthesized append", source.kind),
            ("Later repaired synthesized append", kind),
        ] {
            let created = final_state.memories.iter()
                .find(|memory| memory.content == content)
                .expect("both batch appends committed after repair");
            assert_eq!(created.kind, expected_kind);
            assert_eq!(created.origin, MemoryOrigin::Synthesis);
        }
        assert!(!store.is_temporal_review_due(fixture.now + 1));
    }
}

#[test]
fn explicit_memories_are_protected_from_synthesis_updates_and_removals() {
    let (root, store) = temp_store();
    let explicit = store
        .add_memory(
            "Prefers concise replies",
            1_700_000_000_000,
            MemoryKind::Profile,
        )
        .unwrap()
        .unwrap();
    let snapshot = store.prepare_synthesis();
    assert_eq!(snapshot.memories.len(), 1);
    assert_eq!(snapshot.memories[0].origin, MemoryOrigin::Explicit);

    let update = store
        .apply_synthesis(
            &snapshot,
            &[SynthesisChange::Update {
                id: explicit.id.clone(),
                content: "Prefers long replies".into(),
                kind: MemoryKind::Profile,
            }],
            1_700_000_100_000,
        )
        .unwrap();
    assert_eq!(update, SynthesisApplyResult::Invalid);

    let remove = store
        .apply_synthesis(
            &snapshot,
            &[SynthesisChange::Remove { id: explicit.id }],
            1_700_000_100_000,
        )
        .unwrap();
    assert_eq!(remove, SynthesisApplyResult::Invalid);
    assert_eq!(store.count_memories(), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn synthesis_create_update_remove_round_trip_marks_origin_and_temporal_review() {
    let (root, store) = temp_store();
    let empty = store.prepare_synthesis();
    assert_eq!(
        store
            .apply_synthesis(
                &empty,
                &[SynthesisChange::Create {
                    content: "Working on project Apollo".into(),
                    kind: MemoryKind::Log,
                }],
                1_700_000_000_000,
            )
            .unwrap(),
        SynthesisApplyResult::Committed
    );
    let snapshot = store.prepare_synthesis();
    assert_eq!(snapshot.memories.len(), 1);
    assert_eq!(snapshot.memories[0].origin, MemoryOrigin::Synthesis);
    assert!(!store.is_temporal_review_due(1_700_000_000_001));

    let id = snapshot.memories[0].id.clone();
    assert_eq!(
        store
            .apply_synthesis(
                &snapshot,
                &[SynthesisChange::Update {
                    id,
                    content: "Project Apollo design is in review".into(),
                    kind: MemoryKind::Log,
                }],
                1_700_000_200_000,
            )
            .unwrap(),
        SynthesisApplyResult::Committed
    );
    let updated = store.prepare_synthesis();
    assert_eq!(
        updated.memories[0].content,
        "Project Apollo design is in review"
    );

    let id = updated.memories[0].id.clone();
    assert_eq!(
        store
            .apply_synthesis(
                &updated,
                &[SynthesisChange::Remove { id }],
                1_700_000_300_000,
            )
            .unwrap(),
        SynthesisApplyResult::Committed
    );
    assert_eq!(store.count_memories(), 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stale_snapshot_cannot_commit_and_user_deletion_tombstones_recreation() {
    let (root, store) = temp_store();
    let snapshot = store.prepare_synthesis();
    store
        .add_memory("User moved to Paris", 1_700_000_000_000, MemoryKind::Log)
        .unwrap();
    assert_eq!(
        store
            .apply_synthesis(
                &snapshot,
                &[SynthesisChange::Create {
                    content: "Should not commit".into(),
                    kind: MemoryKind::Log,
                }],
                1_700_000_100_000,
            )
            .unwrap(),
        SynthesisApplyResult::Stale
    );

    let current = store.list_memories(10)[0].clone();
    assert!(store.remove_memory(&current.id).unwrap());
    let after_delete = store.prepare_synthesis();
    assert_eq!(
        store
            .apply_synthesis(
                &after_delete,
                &[SynthesisChange::Create {
                    content: "User moved to Paris".into(),
                    kind: MemoryKind::Log,
                }],
                1_700_000_200_000,
            )
            .unwrap(),
        SynthesisApplyResult::Committed
    );
    assert_eq!(store.count_memories(), 0);
    let _ = fs::remove_dir_all(root);
}
