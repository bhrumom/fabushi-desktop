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
        fs::remove_dir_all(&self.root).expect("remove owned synthesis append fixture");
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

#[test]
fn checked_synthesis_snapshot_preserves_absent_empty_and_readable_projection() {
    let (root, store) = temp_store();
    let absent = store.prepare_synthesis_checked().expect("genuinely absent inputs");
    assert_eq!(absent, store.prepare_synthesis());
    assert!(!store.get_location().exists(), "acquisition must not create Memory");
    fs::create_dir_all(store.get_location().join("log")).unwrap();
    fs::write(store.get_location().join("profile.md"), b"").unwrap();
    assert_eq!(store.prepare_synthesis_checked().unwrap(), absent);
    store.add_memory("Explicit Unicode 中文 🦀 \u{fffd}", 1_700_000_000_000, MemoryKind::Profile).unwrap();
    let profile = store.get_location().join("profile.md");
    let mut raw = fs::read_to_string(&profile).unwrap();
    raw.push_str("- (2023-11-12) Older profile\ninvalid fact\n- (2023-99-99) Invalid date\n");
    fs::write(&profile, &raw).unwrap();
    fs::write(store.get_location().join("log/2023-10.md"),
        "- (2023-10-01) Repeated log\n- (2023-10-02) Older log\n").unwrap();
    fs::write(store.get_location().join("log/2023-11.md"),
        "- (2023-11-01) Repeated log\n- (2023-11-02) Newer log\n").unwrap();
    fs::create_dir(store.get_location().join("log/ignored")).unwrap();
    fs::write(store.get_location().join("log/ignored.bin"), [0xff]).unwrap();
    let checked = store.prepare_synthesis_checked().unwrap();
    assert_eq!(checked, store.prepare_synthesis(), "same fingerprint, parser, ranking and dedup");
    assert_eq!(checked, store.prepare_synthesis_checked().unwrap());
    assert_eq!(checked.memories.iter().map(|record| record.content.as_str()).collect::<Vec<_>>(),
        vec!["Explicit Unicode 中文 🦀 \u{fffd}", "Older profile", "Newer log", "Repeated log", "Older log"]);
    assert_eq!(checked.memories[0].origin, MemoryOrigin::Explicit);
    raw.push_str("<!-- fingerprint retains even unparsed raw content -->\n");
    fs::write(&profile, raw).unwrap();
    let changed = store.prepare_synthesis_checked().unwrap();
    assert_ne!(changed.fingerprint, checked.fingerprint);
    assert_eq!(changed.memories, checked.memories);
    drop(store);
    fs::remove_dir_all(root).expect("remove owned snapshot projection fixture");
}

#[test]
fn checked_synthesis_snapshot_refuses_corrupt_inputs_and_recovers() {
    for relative in ["profile.md", "log/2026-09.md"] {
        let fixture = SynthesisAppendFixture::new();
        let original = fixture.store.prepare_synthesis_checked().unwrap();
        let (path, repaired, observed_error) = fixture.corrupt_destination(relative);
        let before = synthesis_memory_tree(&fixture.store.get_location());
        let error = fixture.store.prepare_synthesis_checked().unwrap_err();
        assert_eq!(error.kind(), observed_error.kind());
        assert_eq!(error.raw_os_error(), observed_error.raw_os_error());
        let reopened = FileMemoryStore::with_synthesis_metadata(fixture.store.get_location(), true);
        assert_eq!(reopened.prepare_synthesis_checked().unwrap_err().kind(), observed_error.kind());
        assert_eq!(synthesis_memory_tree(&fixture.store.get_location()), before);
        fs::write(path, repaired).expect("explicit fixture repair");
        assert_eq!(fixture.store.prepare_synthesis_checked().unwrap(), original);
        assert_eq!(reopened.prepare_synthesis_checked().unwrap(), original);
    }
}

#[test]
fn checked_synthesis_snapshot_reads_beyond_the_input_limit() {
    use mahayana_host_runtime::extensions::memory::memory_service::MEMORY_SYNTHESIS_INPUT_LIMIT;
    let (root, store) = temp_store();
    fs::create_dir_all(store.get_location().join("log")).unwrap();
    let profile = (0..MEMORY_SYNTHESIS_INPUT_LIMIT + 1)
        .map(|index| format!("- (2023-11-14) Profile fact {index}\n")).collect::<String>();
    fs::write(store.get_location().join("profile.md"), profile).unwrap();
    let older = store.get_location().join("log/2000-01.md");
    let repaired = b"- (2000-01-01) Older selected fact\n";
    fs::write(&older, repaired).unwrap();
    let healthy = store.prepare_synthesis_checked().unwrap();
    assert_eq!(healthy.memories.len(), MEMORY_SYNTHESIS_INPUT_LIMIT);
    assert!(healthy.memories.iter().all(|record| record.kind == MemoryKind::Profile));
    let mut corrupt = repaired.to_vec();
    corrupt.push(0xff);
    fs::write(&older, &corrupt).unwrap();
    assert_eq!(fs::read_to_string(&older).unwrap_err().kind(), io::ErrorKind::InvalidData);
    let before = synthesis_memory_tree(&store.get_location());
    assert_eq!(store.prepare_synthesis_checked().unwrap_err().kind(), io::ErrorKind::InvalidData);
    assert_eq!(synthesis_memory_tree(&store.get_location()), before);
    fs::write(older, repaired).unwrap();
    assert_eq!(store.prepare_synthesis_checked().unwrap(), healthy);
    drop(store);
    fs::remove_dir_all(root).expect("remove owned bounded-input fixture");
}

#[test]
fn checked_synthesis_snapshot_refuses_native_type_obstacles() {
    for obstacle in ["profile", "log-root", "selected-log", "ancestor"] {
        let (root, store) = temp_store();
        let memory = store.get_location();
        fs::create_dir_all(&memory).unwrap();
        match obstacle {
            "profile" => {
                fs::create_dir(memory.join("profile.md")).unwrap();
                fs::write(memory.join("profile.md/untouched"), b"owned sentinel").unwrap();
            }
            "log-root" => { fs::write(memory.join("log"), b"owned sentinel").unwrap(); }
            "selected-log" => {
                fs::create_dir_all(memory.join("log/selected.md")).unwrap();
                fs::write(memory.join("log/selected.md/untouched"), b"owned sentinel").unwrap();
            }
            "ancestor" => {
                fs::remove_dir(&memory).unwrap();
                fs::write(&memory, b"owned ancestor sentinel").unwrap();
            }
            _ => unreachable!(),
        }
        let before = synthesis_memory_tree(&memory);
        assert!(store.prepare_synthesis_checked().is_err(), "{obstacle} must refuse");
        assert_eq!(synthesis_memory_tree(&memory), before);
        if memory.is_dir() { fs::remove_dir_all(&memory).unwrap(); }
        else { fs::remove_file(&memory).unwrap(); }
        assert!(store.prepare_synthesis_checked().unwrap().memories.is_empty());
        assert!(!memory.exists(), "successful missing input remains read-only");
        drop(store);
        fs::remove_dir_all(root).expect("remove owned native obstacle fixture");
    }
}

#[cfg(unix)]
#[test]
fn checked_synthesis_snapshot_preserves_readable_symlinks_and_refuses_dangling_inputs() {
    use std::os::unix::fs::symlink;
    for relative in ["profile.md", "log/linked.md"] {
        let (root, store) = temp_store();
        fs::create_dir_all(store.get_location().join("log")).unwrap();
        let owned_target = root.join("owned-target.md");
        let link = store.get_location().join(relative);
        let raw = b"- (2023-11-14) Readable symlink fact\n";
        fs::write(&owned_target, raw).unwrap();
        symlink(&owned_target, &link).unwrap();
        let healthy = store.prepare_synthesis_checked().unwrap();
        assert_eq!(healthy, store.prepare_synthesis());
        assert_eq!(healthy.memories.len(), 1);
        fs::remove_file(&owned_target).unwrap();
        assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert!(store.prepare_synthesis_checked().is_err());
        assert_eq!(fs::read_link(&link).unwrap(), owned_target);
        assert!(!owned_target.exists());
        fs::write(&owned_target, raw).unwrap();
        assert_eq!(store.prepare_synthesis_checked().unwrap(), healthy);
        assert_eq!(fs::read(&owned_target).unwrap(), raw);
        drop(store);
        fs::remove_dir_all(root).expect("remove owned symlink fixture");
    }
}
