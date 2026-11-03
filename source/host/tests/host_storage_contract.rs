use std::path::Path;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::{
    agents::settings_file::{get_sand_settings_path, read_sand_settings_file, write_sand_settings_file, DEFAULT_HIDDEN_FROM_SIDEBAR, DEFAULT_NOTIFY_ON_AGENT_UPDATES},
    extensions::{
        box_store_sync::{box_store_diagnostics::{pin_box_store_diagnostics_reporter, report_box_store_diagnostic}, box_store_sync_error::SandBoxStoreSyncError},
        cloud_agents::cloud_agent_launch_error::SandCloudAgentLaunchError,
        session::conversation_blobs_path::conversation_blobs_path,
        transcript::{channel_delivery_unregistered_error::SandChannelDeliveryUnregisteredError, send_not_persisted_error::SandSendNotPersistedError},
    },
    notify_drain_gate::NotifyDrainGate,
    storage::{
        agent_paths::{assert_valid_sand_agent_id, resolve_sand_agent_dir},
        sqlite_busy::{
            is_sqlite_busy_error, is_sqlite_cant_open_error, is_sqlite_corrupt_error,
            is_sqlite_io_error, retry_sqlite_busy_with_delay, RetrySqliteBusyOptions,
            SQLITE_BUSY, SQLITE_CANTOPEN, SQLITE_CORRUPT, SQLITE_IOERR,
        },
        sqlite_recovery::{
            copy_salvageable_sqlite_rows, open_sqlite_for_salvage,
            quarantine_corrupt_sqlite_db, remove_sqlite_sidecars,
        },
        store_db::{
            bump_db_write_generation, checkpoint_sand_agent_db,
            delete_sand_agent_db_write_generation, get_sand_agent_db_write_generation,
            has_live_sand_agent_db_handle, live_db_handle_count, register_live_db_handle,
            release_live_db_handle, wal_frames_fully_folded, DB_BUSY_TIMEOUT_MS,
            SQLITE_DB_SIDECAR_SUFFIXES,
        },
    },
};
use serde_json::{Map, Value};
use uuid::Uuid;

#[test]
fn notify_drain_gate_matches_notify_floor_and_safety_poll_semantics() {
    use std::cell::Cell;
    let now = Cell::new(1_000_u64);
    let mut gate = NotifyDrainGate::new(|| now.get(), || true, || true);
    assert!(gate.should_drain(false));
    gate.record_poll();
    assert!(!gate.should_drain(false));
    gate.record_notify();
    now.set(4_999);
    assert!(!gate.should_drain(false));
    now.set(5_000);
    assert!(gate.should_drain(false));
}

#[test]
fn agent_paths_reject_traversal_and_whitespace() {
    for id in ["agent-1", "agent-雪-☃"] {
        assert!(assert_valid_sand_agent_id(id).is_ok(), "valid agent id {id:?}");
        let dir = resolve_sand_agent_dir(id, Some(Path::new("/tmp/home"))).unwrap();
        assert_eq!(dir.file_name().unwrap(), id);
    }
    for id in ["", "../x", " x", "x ", "a/b", "a\\b", ".", "..", "a\0b"] {
        let expected = format!("Invalid Sand agent id: {id}");
        assert_eq!(assert_valid_sand_agent_id(id).unwrap_err().to_string(), expected);
        assert_eq!(resolve_sand_agent_dir(id, Some(Path::new("/tmp/home"))).unwrap_err().to_string(), expected);
    }
}

#[cfg(windows)]
#[test]
fn agent_paths_reject_native_windows_prefixes_and_roots() {
    for id in [
        "C:", "C:outside", "c:outside", "C:.", "C:..",
        r"C:\outside", r"\outside", r"\\server\share",
        r"\\?\C:\outside", r"\\.\C:\outside",
    ] {
        let expected = format!("Invalid Sand agent id: {id}");
        assert_eq!(assert_valid_sand_agent_id(id).unwrap_err().to_string(), expected);
        assert_eq!(resolve_sand_agent_dir(id, Some(Path::new(r"C:\owned-home"))).unwrap_err().to_string(), expected);
    }
}

#[cfg(unix)]
#[test]
fn agent_paths_preserve_literal_colon_components() {
    for id in ["C:", "C:outside", "c:outside", "C:.", "C:..", "name:part"] {
        assert!(assert_valid_sand_agent_id(id).is_ok(), "valid native component {id:?}");
        let dir = resolve_sand_agent_dir(id, Some(Path::new("/tmp/home"))).unwrap();
        assert_eq!(dir.file_name().unwrap(), id);
    }
}

#[test]
fn settings_file_uses_defaults_preserves_unknown_fields_and_atomic_replace() {
    let root = std::env::temp_dir().join(format!("fabushi-settings-{}", Uuid::new_v4()));
    let path = get_sand_settings_path(&root);
    let defaults = read_sand_settings_file(&path);
    assert_eq!(defaults.notify_on_agent_updates, DEFAULT_NOTIFY_ON_AGENT_UPDATES);
    assert_eq!(defaults.hidden_from_sidebar, DEFAULT_HIDDEN_FROM_SIDEBAR);
    let mut first = Map::new(); first.insert("unknown".into(), Value::String("keep".into())); first.insert("hiddenFromSidebar".into(), Value::Bool(true));
    write_sand_settings_file(&path, &first).unwrap();
    let mut update = Map::new(); update.insert("notifyOnAgentUpdates".into(), Value::Bool(false));
    write_sand_settings_file(&path, &update).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"unknown\": \"keep\""));
    let settings = read_sand_settings_file(&path);
    assert!(settings.hidden_from_sidebar);
    assert!(!settings.notify_on_agent_updates);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn session_and_error_contracts_match_reference_messages() {
    assert_eq!(conversation_blobs_path("/tmp/agent/store.db"), Path::new("/tmp/agent/conversation-blobs.db"));
    assert_eq!(SandChannelDeliveryUnregisteredError.to_string(), "No channel delivery mechanism is registered.");
    assert!(SandSendNotPersistedError.to_string().contains("client retry is not swallowed"));
    assert_eq!(SandBoxStoreSyncError::new("sync failed").to_string(), "sync failed");
    assert_eq!(SandCloudAgentLaunchError::new("launch failed").to_string(), "launch failed");
}

#[test]
fn box_store_diagnostics_reporter_can_be_pinned_and_cleared() {
    let seen = Arc::new(Mutex::new(0_u32));
    let sink = Arc::clone(&seen);
    pin_box_store_diagnostics_reporter(Some(Arc::new(move |_| *sink.lock().unwrap() += 1)));
    report_box_store_diagnostic(&Map::new());
    pin_box_store_diagnostics_reporter(None);
    assert_eq!(*seen.lock().unwrap(), 1);
}


#[test]
fn sqlite_busy_classification_and_retry_match_reference() {
    fn sqlite_failure(code: i32, message: &str) -> rusqlite::Error {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(code),
            Some(message.to_string()),
        )
    }

    assert!(is_sqlite_busy_error(&sqlite_failure(
        SQLITE_BUSY,
        "database is locked"
    )));
    assert!(is_sqlite_io_error(&sqlite_failure(
        SQLITE_IOERR | (3 << 8),
        "short read"
    )));
    assert!(is_sqlite_cant_open_error(&sqlite_failure(
        SQLITE_CANTOPEN,
        "unable to open database file"
    )));
    assert!(is_sqlite_corrupt_error(&sqlite_failure(
        SQLITE_CORRUPT,
        "database disk image is malformed"
    )));

    let mut attempts = 0_u32;
    let mut delays = Vec::new();
    let value = retry_sqlite_busy_with_delay(
        || {
            attempts += 1;
            if attempts < 3 {
                Err(sqlite_failure(SQLITE_BUSY, "database is locked"))
            } else {
                Ok("ready")
            }
        },
        RetrySqliteBusyOptions {
            attempts: 5,
            base_delay_ms: 10,
        },
        |delay_ms| delays.push(delay_ms),
    )
    .expect("busy retry should eventually succeed");
    assert_eq!(value, "ready");
    assert_eq!(attempts, 3);
    assert_eq!(delays, vec![10, 20]);
}

#[test]
fn sqlite_recovery_quarantines_salvages_rows_and_store_db_tracks_handles() {
    assert_eq!(DB_BUSY_TIMEOUT_MS, 5_000);
    assert_eq!(SQLITE_DB_SIDECAR_SUFFIXES, ["-wal", "-shm", "-journal"]);

    let root = std::env::temp_dir().join(format!("fabushi-sqlite-storage-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).expect("create storage root");
    let db_path = root.join("agent.sqlite");
    {
        let db = rusqlite::Connection::open(&db_path).expect("create sqlite db");
        db.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO items(value) VALUES ('a'), ('b');",
        )
        .expect("seed sqlite db");
    }

    let salvage = open_sqlite_for_salvage(&db_path, Some(DB_BUSY_TIMEOUT_MS))
        .expect("salvage should open a valid database");
    let destination = rusqlite::Connection::open_in_memory().expect("open destination");
    destination
        .execute_batch("CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT NOT NULL)")
        .expect("create destination table");
    let copied = copy_salvageable_sqlite_rows(
        &salvage,
        "SELECT id, value FROM items ORDER BY id",
        |row| {
            destination.execute(
                "INSERT INTO items(id, value) VALUES (?1, ?2)",
                rusqlite::params![row.get::<_, i64>(0)?, row.get::<_, String>(1)?],
            )?;
            Ok(())
        },
    );
    assert_eq!(copied, 2);
    drop(salvage);
    assert!(checkpoint_sand_agent_db(&db_path, DB_BUSY_TIMEOUT_MS));

    assert_eq!(get_sand_agent_db_write_generation(&db_path), 0);
    bump_db_write_generation(&db_path);
    bump_db_write_generation(&db_path);
    assert_eq!(get_sand_agent_db_write_generation(&db_path), 2);
    delete_sand_agent_db_write_generation(&db_path);
    assert_eq!(get_sand_agent_db_write_generation(&db_path), 0);

    assert_eq!(live_db_handle_count(&db_path), 0);
    register_live_db_handle(&db_path);
    register_live_db_handle(&db_path);
    assert!(has_live_sand_agent_db_handle(&db_path));
    assert_eq!(live_db_handle_count(&db_path), 2);
    release_live_db_handle(&db_path);
    release_live_db_handle(&db_path);
    assert!(!has_live_sand_agent_db_handle(&db_path));
    assert_eq!(wal_frames_fully_folded(3, 3), Some(true));
    assert_eq!(wal_frames_fully_folded(4, 3), Some(false));
    assert_eq!(wal_frames_fully_folded(-1, 0), None);

    let corrupt = root.join("corrupt.sqlite");
    let quarantine = root.join("quarantine.sqlite");
    std::fs::write(&corrupt, b"corrupt").expect("write corrupt source");
    std::fs::write(format!("{}-wal", corrupt.display()), b"wal")
        .expect("write source wal");
    let result = quarantine_corrupt_sqlite_db(
        &corrupt,
        Some(&quarantine),
        false,
        2,
        1,
    );
    assert_eq!(result.quarantine_path.as_deref(), Some(quarantine.as_path()));
    assert!(!result.copied);
    assert!(quarantine.exists());
    assert!(!corrupt.exists());

    std::fs::write(format!("{}-wal", db_path.display()), b"wal")
        .expect("write temporary wal");
    std::fs::write(format!("{}-shm", db_path.display()), b"shm")
        .expect("write temporary shm");
    remove_sqlite_sidecars(&db_path);
    assert!(!Path::new(&format!("{}-wal", db_path.display())).exists());
    assert!(!Path::new(&format!("{}-shm", db_path.display())).exists());

    drop(destination);
    let _ = std::fs::remove_dir_all(root);
}

// The raw-byte creation prerequisite is exercised by the required Linux
// Actions job. APFS creation does not accept these invalid UTF-8 components.
#[cfg(target_os = "linux")]
mod sqlite_raw_path_contract {
    use std::ffi::OsString;
    use std::fs;
    use std::io::ErrorKind;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    use std::path::{Path, PathBuf};

    use mahayana_host_runtime::storage::{
        sqlite_busy::is_sqlite_corrupt_error,
        sqlite_recovery::{
            open_sqlite_for_salvage, quarantine_corrupt_sqlite_db, remove_sqlite_db,
            remove_sqlite_sidecars,
        },
    };
    use uuid::Uuid;

    const FAMILY_SUFFIXES: [&str; 4] = ["", "-wal", "-shm", "-journal"];

    // Construct expected locations directly from Unix bytes, independently of
    // the production OsString suffix helper and any lossy display conversion.
    fn suffixed(path: &Path, suffix: &str) -> PathBuf {
        let mut bytes = path.as_os_str().as_bytes().to_vec();
        bytes.extend_from_slice(suffix.as_bytes());
        PathBuf::from(OsString::from_vec(bytes))
    }

    fn write_family(path: &Path, label: &str) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = Vec::new();
        for suffix in FAMILY_SUFFIXES {
            let file = suffixed(path, suffix);
            let bytes = format!("{label}:{suffix}").into_bytes();
            fs::write(&file, &bytes).expect("write SQLite family fixture");
            files.push((file, bytes));
        }
        files
    }

    fn assert_family(path: &Path, label: &str) {
        for suffix in FAMILY_SUFFIXES {
            let file = suffixed(path, suffix);
            assert_eq!(
                fs::read(&file).expect("read SQLite family fixture"),
                format!("{label}:{suffix}").as_bytes(),
                "unexpected bytes at {file:?}",
            );
        }
    }

    fn assert_absent(path: &Path) {
        let error = fs::symlink_metadata(path).expect_err("native path must be absent");
        assert_eq!(error.kind(), ErrorKind::NotFound, "{path:?}");
    }

    struct Fixture {
        root: PathBuf,
        non_utf8_parent: bool,
        preserved: Vec<(PathBuf, Vec<u8>)>,
    }

    impl Fixture {
        fn new(label: &str, non_utf8_parent: bool) -> Self {
            let root = std::env::temp_dir().join(format!(
                "fabushi-sqlite-raw-{label}-{non_utf8_parent}-{}",
                Uuid::new_v4(),
            ));
            assert!(root.to_str().is_some(), "fixture root must be UTF-8");
            fs::create_dir(&root).expect("exclusively create SQLite byte-path fixture root");
            eprintln!("SQLite byte-path fixture retained on failure: {root:?}");
            Self {
                root,
                non_utf8_parent,
                preserved: Vec::new(),
            }
        }

        fn preserve_file(&mut self, path: PathBuf, bytes: &[u8]) {
            fs::write(&path, bytes).expect("write unrelated fixture sentinel");
            self.preserved.push((path, bytes.to_vec()));
        }

        fn pair(&mut self, label: &str) -> (PathBuf, PathBuf) {
            let mut native_component = label.as_bytes().to_vec();
            native_component.extend_from_slice(b"-\xff");
            let mut twin_component = format!("{label}-\u{fffd}");
            if !self.non_utf8_parent {
                native_component.extend_from_slice(b".sqlite");
                twin_component.push_str(".sqlite");
            }
            let mut native = self.root.join(OsString::from_vec(native_component));
            let mut twin = self.root.join(twin_component);
            if self.non_utf8_parent {
                fs::create_dir(&native).expect("create native-byte parent");
                fs::create_dir(&twin).expect("create distinct Unicode lookalike parent");
                self.preserve_file(native.join("keep"), b"native parent sentinel");
                self.preserve_file(twin.join("keep"), b"lookalike parent sentinel");
                native.push("store.sqlite");
                twin.push("store.sqlite");
            }
            assert!(native.to_str().is_none());
            assert!(twin.to_str().is_some());
            assert_ne!(native.as_os_str().as_bytes(), twin.as_os_str().as_bytes());
            assert_eq!(native.to_string_lossy(), twin.to_string_lossy());
            self.preserved
                .extend(write_family(&twin, &format!("unrelated {label}")));
            (native, twin)
        }

        fn assert_preserved(&self) {
            for (path, bytes) in &self.preserved {
                assert_eq!(
                    fs::read(path).expect("read retained sentinel"),
                    *bytes,
                    "{path:?}",
                );
            }
        }

        // No Drop cleanup: every failure leaves this exact owned root for inspection.
        fn finish(self) {
            fs::remove_dir_all(&self.root).expect("remove successful owned fixture root");
        }
    }

    #[test]
    fn explicit_quarantine_preserves_native_source_and_destination_bytes() {
        for non_utf8_parent in [false, true] {
            let mut fixture = Fixture::new("explicit", non_utf8_parent);
            let (source, _) = fixture.pair("source");
            let (destination, _) = fixture.pair("destination");
            write_family(&source, "native source");
            write_family(&destination, "stale native destination");
            fixture.assert_preserved();

            let result =
                quarantine_corrupt_sqlite_db(&source, Some(&destination), false, 1, 0);
            assert_eq!(result.quarantine_path.as_deref(), Some(destination.as_path()));
            assert!(!result.copied);
            assert_eq!(result.rename_error_code, None);
            assert_family(&destination, "native source");
            for suffix in FAMILY_SUFFIXES {
                assert_absent(&suffixed(&source, suffix));
            }
            fixture.assert_preserved();
            fixture.finish();
        }
    }

    #[test]
    fn default_quarantine_appends_to_the_complete_native_path() {
        for non_utf8_parent in [false, true] {
            let mut fixture = Fixture::new("default", non_utf8_parent);
            let (source, _) = fixture.pair("source");
            write_family(&source, "native source");
            fixture.assert_preserved();

            let result = quarantine_corrupt_sqlite_db(&source, None, false, 1, 0);
            assert!(!result.copied);
            assert_eq!(result.rename_error_code, None);
            let destination = result.quarantine_path.expect("default quarantine path");
            let suffix = destination
                .as_os_str()
                .as_bytes()
                .strip_prefix(source.as_os_str().as_bytes())
                .expect("default quarantine must preserve every original native path byte");
            let stamp = suffix
                .strip_prefix(b".corrupt-")
                .expect("default quarantine suffix");
            assert!(!stamp.is_empty() && stamp.iter().all(u8::is_ascii_digit));
            assert_eq!(destination.parent(), source.parent());
            assert_family(&destination, "native source");
            for suffix in FAMILY_SUFFIXES {
                assert_absent(&suffixed(&source, suffix));
            }
            fixture.assert_preserved();
            fixture.finish();
        }
    }

    #[test]
    fn sidecar_and_database_removal_preserve_native_path_lookalikes() {
        for non_utf8_parent in [false, true] {
            let mut fixture = Fixture::new("remove", non_utf8_parent);
            let (source, _) = fixture.pair("source");
            write_family(&source, "native source");
            fixture.assert_preserved();

            remove_sqlite_sidecars(&source);
            assert_eq!(
                fs::read(&source).expect("main database remains"),
                b"native source:",
            );
            for suffix in &FAMILY_SUFFIXES[1..] {
                assert_absent(&suffixed(&source, suffix));
            }
            fixture.assert_preserved();

            write_family(&source, "native removal");
            let journal = suffixed(&source, "-journal");
            fs::remove_file(&journal).expect("replace owned journal fixture");
            fs::create_dir(&journal).expect("create owned sidecar directory");
            fs::write(journal.join("owned"), b"native directory sidecar")
                .expect("seed owned sidecar directory");
            remove_sqlite_db(&source, 1, 0).expect("remove exact native database family");
            for suffix in FAMILY_SUFFIXES {
                assert_absent(&suffixed(&source, suffix));
            }
            fixture.assert_preserved();
            fixture.finish();
        }
    }

    #[test]
    fn salvage_sql_failure_cleans_only_native_sidecars() {
        for non_utf8_parent in [false, true] {
            let mut fixture = Fixture::new("salvage", non_utf8_parent);
            let (source, _) = fixture.pair("source");
            let corrupt_bytes = b"this fixture is not a SQLite database";
            fs::write(&source, corrupt_bytes).expect("write corrupt native database");
            {
                let probe = rusqlite::Connection::open(&source).expect("open native-byte source");
                let error = probe
                    .query_row("PRAGMA schema_version", [], |_| Ok(()))
                    .expect_err("real schema query must fail before salvage cleanup");
                assert!(is_sqlite_corrupt_error(&error), "{error:?}");
            }
            for suffix in ["-wal", "-journal"] {
                fs::write(suffixed(&source, suffix), b"native failed-query sidecar")
                    .expect("write native salvage sidecar");
            }
            // SQLite cannot unlink a directory as an ordinary sidecar file.
            // Its absence below therefore pins the helper's error cleanup.
            let shm = suffixed(&source, "-shm");
            fs::create_dir(&shm).expect("create owned salvage sidecar directory");
            fs::write(shm.join("owned"), b"native salvage directory sentinel")
                .expect("seed owned salvage sidecar directory");
            fixture.assert_preserved();

            let salvage = open_sqlite_for_salvage(&source, Some(0))
                .expect("salvage retains its final unchecked connection fallback");
            let error = salvage
                .query_row("PRAGMA schema_version", [], |_| Ok(()))
                .expect_err("cleanup does not repair the corrupt main database");
            assert!(is_sqlite_corrupt_error(&error), "{error:?}");
            drop(salvage);

            assert_eq!(
                fs::read(&source).expect("corrupt main bytes retained"),
                corrupt_bytes,
            );
            for suffix in &FAMILY_SUFFIXES[1..] {
                assert_absent(&suffixed(&source, suffix));
            }
            fixture.assert_preserved();
            fixture.finish();
        }
    }
}
