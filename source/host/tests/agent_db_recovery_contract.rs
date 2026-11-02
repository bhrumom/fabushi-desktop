use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::session::agent_db_recovery::{
    AUTOMATION_CUSTODY_UNAVAILABLE_KV, AgentDbRecoveryError, DbRecoveryOptions,
    acquire_automation_custody_recovery_guard, automation_custody_recovery_marker_path,
    is_store_db_healthy, open_configured_db, recover_corrupt_store_db, salvage_store_db,
};
use mahayana_host_runtime::extensions::session::agent_db_schema::AGENT_DB_SCHEMA;
use mahayana_host_runtime::storage::store_db::SQLITE_DB_SIDECAR_SUFFIXES;
use mahayana_host_runtime::transcript_mutation_events::subscribe_transcript_mutations;
use rusqlite::{Connection, params};
use uuid::Uuid;

fn path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("fabushi-{label}-{}.sqlite", Uuid::new_v4()))
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
    // Call only after this fixture's connections/guards and any exact child
    // have ended. Production must never unlink the stable guard or marker.
    for suffix in [
        "-wal", "-shm", "-journal",
        ".automation-custody.guard", ".automation-custody-unavailable",
    ] {
        let _ = fs::remove_file(sibling_path(path, suffix));
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(
        "{}.corrupt-",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(&prefix) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

fn initialize(db: &Connection) {
    db.execute_batch(AGENT_DB_SCHEMA).expect("schema");
}

fn sibling_path(db_path: &Path, suffix: &str) -> PathBuf {
    let mut name = db_path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn fixture_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "fabushi-agent-db-recovery-{label}-{}-{}",
        std::process::id(), Uuid::new_v4(),
    ));
    fs::create_dir(&root).expect("exclusively reserve recovery fixture");
    eprintln!("[agent-db-recovery] allocated pid={} root={}",
        std::process::id(), root.display());
    root
}

fn assert_absent(path: &Path) {
    let error = fs::symlink_metadata(path).expect_err("path must be absent, including dangling links");
    assert_eq!(error.kind(), io::ErrorKind::NotFound, "{}: {error}", path.display());
}

fn authority_marker_count(db: &Connection) -> i64 {
    db.query_row(
        "SELECT COUNT(*) FROM kv WHERE key = ?1",
        params![AUTOMATION_CUSTODY_UNAVAILABLE_KV],
        |row| row.get(0),
    ).expect("query durable Automation authority marker")
}

fn assert_recovered_authority_unavailable(db_path: &Path, db: &Connection) {
    assert!(is_store_db_healthy(db));
    let marker = automation_custody_recovery_marker_path(db_path);
    assert_eq!(marker, sibling_path(db_path, ".automation-custody-unavailable"));
    assert!(fs::symlink_metadata(&marker).expect("permanent recovery marker").is_file());
    assert_eq!(authority_marker_count(db), 1);
    acquire_automation_custody_recovery_guard(db_path, db)
        .expect_err("reconstructed history cannot authorize Automation custody");
}

fn inspect_recovery_publication(db_path: &Path) -> Result<(), String> {
    let marker = automation_custody_recovery_marker_path(db_path);
    let metadata = fs::symlink_metadata(&marker).map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err("recovery callback observed no regular permanent marker".into());
    }
    let db = Connection::open(db_path).map_err(|error| error.to_string())?;
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM kv WHERE key = ?1",
        params![AUTOMATION_CUSTODY_UNAVAILABLE_KV],
        |row| row.get(0),
    ).map_err(|error| error.to_string())?;
    if count != 1 {
        return Err(format!("recovery callback observed {count} durable KV markers"));
    }
    // Availability deliberately refuses after recovery. Observe the actual
    // OS resource separately so that refusal cannot conceal an unreleased lock.
    let guard_file = OpenOptions::new().read(true).write(true)
        .open(sibling_path(db_path, ".automation-custody.guard"))
        .map_err(|error| error.to_string())?;
    fs2::FileExt::try_lock_exclusive(&guard_file).map_err(|error| error.to_string())?;
    fs2::FileExt::unlock(&guard_file).map_err(|error| error.to_string())?;
    Ok(())
}

fn database_bytes(db_path: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    ["", "-wal", "-shm", "-journal"].into_iter().map(|suffix| {
        let path = sibling_path(db_path, suffix);
        let bytes = match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => panic!("read owned fixture {}: {error}", path.display()),
        };
        (path, bytes)
    }).collect()
}

fn quarantine_paths(db_path: &Path) -> Vec<PathBuf> {
    let prefix = format!("{}.corrupt-", db_path.file_name().unwrap().to_string_lossy());
    let mut paths = fs::read_dir(db_path.parent().unwrap()).expect("fixture parent")
        .map(|entry| entry.expect("fixture entry").path())
        .filter(|path| path.file_name().unwrap().to_string_lossy().starts_with(&prefix))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn seed_corrupt_db_and_sidecars(db_path: &Path) {
    fs::write(db_path, b"original corrupt database bytes").expect("corrupt DB fixture");
    for suffix in ["-wal", "-shm", "-journal"] {
        fs::write(sibling_path(db_path, suffix), format!("original {suffix} bytes"))
            .expect("sidecar refusal sentinel");
    }
}

fn assert_recovery_refusal_preserves_files(db_path: &Path) {
    let original = database_bytes(db_path);
    let quarantines = quarantine_paths(db_path);
    recover_corrupt_store_db(
        db_path, "refused-recovery", &DbRecoveryOptions::default(), "owned refusal oracle",
    ).expect_err("invalid recovery authority must refuse before quarantine");
    assert_eq!(database_bytes(db_path), original, "refusal changed DB or sidecar bytes");
    assert_eq!(quarantine_paths(db_path), quarantines, "refusal created a quarantine");
}

#[test]
fn salvage_preserves_kv_blob_and_transcript_rows() {
    let source_path = path("agent-db-salvage-source");
    let target_path = path("agent-db-salvage-target");
    let source = Connection::open(&source_path).expect("source");
    initialize(&source);
    source.execute(
        "INSERT INTO kv(key,value) VALUES('metadata','abcd')",
        [],
    ).expect("kv");
    source.execute(
        "INSERT INTO blobs(id,data) VALUES('blob-a',?1)",
        params![b"blob-data".as_slice()],
    ).expect("blob");
    source.execute(
        "INSERT INTO transcript_entries(seq,id,entry) VALUES(7,'entry-a',?1)",
        params![r#"{"id":"entry-a","kind":"message","role":"user","content":"hi"}"#],
    ).expect("transcript");
    drop(source);

    let target = Connection::open(&target_path).expect("target");
    initialize(&target);
    let counts = salvage_store_db(&source_path, &target);
    assert_eq!(counts.kv, 1);
    assert_eq!(counts.blobs, 1);
    assert_eq!(counts.transcript, 1);
    let seq: i64 = target
        .query_row(
            "SELECT seq FROM transcript_entries WHERE id='entry-a'",
            [],
            |row| row.get(0),
        )
        .expect("transcript row");
    assert_eq!(seq, 7);

    drop(target);
    cleanup(&source_path);
    cleanup(&target_path);
}

#[test]
fn configured_open_recovers_corrupt_store_and_publishes_reindex() {
    let db_path = path("agent-db-corrupt");
    fs::write(&db_path, b"not-a-sqlite-database").expect("corrupt file");

    let observed = Arc::new(Mutex::new(Vec::<serde_json::Map<String, serde_json::Value>>::new()));
    let observed_for_listener = Arc::clone(&observed);
    let publication_checks = Arc::new(Mutex::new(Vec::<(&str, Result<(), String>)>::new()));
    let checks_for_listener = Arc::clone(&publication_checks);
    let path_for_listener = db_path.clone();
    let subscription = subscribe_transcript_mutations(move |mutation| {
        if mutation.get("kind").and_then(serde_json::Value::as_str) == Some("agent-needs-reindex")
            && mutation.get("agentId").and_then(serde_json::Value::as_str) == Some("agent-corrupt")
        {
            let check = inspect_recovery_publication(&path_for_listener);
            checks_for_listener.lock().expect("publication check lock").push(("reindex", check));
        }
        observed_for_listener
            .lock()
            .expect("mutation lock")
            .push(mutation.clone());
    });

    let checks_for_callback = Arc::clone(&publication_checks);
    let path_for_callback = db_path.clone();
    let options = DbRecoveryOptions {
        on_corruption_recovered: Some(Arc::new(move |_| {
            let check = inspect_recovery_publication(&path_for_callback);
            checks_for_callback.lock().expect("publication check lock").push(("callback", check));
            panic!("callback failures are intentionally ignored");
        })),
        ..DbRecoveryOptions::default()
    };
    let db = open_configured_db(
        &db_path,
        "agent-corrupt",
        &options,
        false,
    )
    .expect("recover");
    assert!(is_store_db_healthy(&db));
    let kv_present: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='kv'",
            [],
            |row| row.get(0),
        )
        .expect("kv table");
    assert_eq!(kv_present, 1);
    assert_recovered_authority_unavailable(&db_path, &db);
    drop(db);

    let mutations = observed.lock().expect("mutation lock");
    assert!(mutations.iter().any(|mutation| {
        mutation.get("kind").and_then(serde_json::Value::as_str) == Some("agent-needs-reindex")
            && mutation.get("agentId").and_then(serde_json::Value::as_str) == Some("agent-corrupt")
    }));
    drop(mutations);
    subscription.unsubscribe();
    let checks = publication_checks.lock().expect("publication check lock");
    assert_eq!(checks.len(), 2, "both external publication boundaries must be observed");
    assert_eq!(checks[0].0, "callback");
    assert_eq!(checks[1].0, "reindex");
    for (boundary, check) in checks.iter() {
        assert!(check.is_ok(), "{boundary} observed incomplete publication or a held guard: {check:?}");
    }
    drop(checks);

    cleanup(&db_path);
}

#[test]
fn healthy_automation_custody_guard_requires_the_same_existing_database() {
    let root = fixture_root("healthy");
    let db_path = root.join("store.db");
    let other_path = root.join("other.db");
    let db = open_configured_db(&db_path, "healthy", &DbRecoveryOptions::default(), false)
        .expect("configured healthy DB");
    let other = open_configured_db(&other_path, "other", &DbRecoveryOptions::default(), false)
        .expect("other configured healthy DB");
    let memory = Connection::open_in_memory().expect("unrelated memory DB");
    initialize(&memory);
    assert_eq!(AUTOMATION_CUSTODY_UNAVAILABLE_KV, "automationCustodyAuthorityUnavailable");
    assert_eq!(authority_marker_count(&db), 0);
    assert_absent(&automation_custody_recovery_marker_path(&db_path));

    let guard = acquire_automation_custody_recovery_guard(&db_path, &db)
        .expect("healthy authority is available");
    guard.require_available(&db).expect("same existing connection remains available");
    let shared = acquire_automation_custody_recovery_guard(&db_path, &db)
        .expect("checked guards share the real lock");
    shared.require_available(&db).expect("second shared guard is available");
    let original = database_bytes(&db_path);
    acquire_automation_custody_recovery_guard(&db_path, &other)
        .expect_err("another valid database is not this authority");
    guard.require_available(&other).expect_err("held guard cannot validate another database");
    acquire_automation_custody_recovery_guard(&db_path, &memory)
        .expect_err("an in-memory database cannot stand in for the named file");
    guard.require_available(&memory).expect_err("held guard cannot validate in-memory authority");
    let missing = root.join("missing.db");
    acquire_automation_custody_recovery_guard(&missing, &db)
        .expect_err("the supplied path must already name this database");
    assert_absent(&missing);
    assert_eq!(database_bytes(&db_path), original);
    guard.require_available(&db).expect("wrong-connection refusals leave the real authority usable");

    drop(shared);
    drop(guard);
    drop(memory);
    drop(other);
    drop(db);
    cleanup(&db_path);
    cleanup(&other_path);
    cleanup(&missing);
    fs::remove_dir_all(root).expect("remove completed owned fixture");
}

// Required Linux Actions evidence supplies both native-byte path forms.
#[cfg(target_os = "linux")]
#[test]
fn healthy_automation_custody_guard_preserves_native_main_path_identity() {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    for native_parent in [false, true] {
        let label = if native_parent { "healthy-native-parent" } else { "healthy-native-filename" };
        let root = fixture_root(label);
        let (db_path, twin_path) = if native_parent {
            (
                root.join(OsString::from_vec(b"session-\xff".to_vec())).join("store.db"),
                root.join("session-\u{fffd}").join("store.db"),
            )
        } else {
            (
                root.join(OsString::from_vec(b"store-\xff.db".to_vec())),
                root.join("store-\u{fffd}.db"),
            )
        };
        assert!(db_path.to_str().is_none(), "the actual main path must contain non-UTF-8 bytes");
        assert!(twin_path.to_str().is_some());
        assert_ne!(db_path.as_os_str().as_bytes(), twin_path.as_os_str().as_bytes());
        let db = open_configured_db(&db_path, label, &DbRecoveryOptions::default(), false)
            .expect("healthy native-path main database");
        let twin = open_configured_db(&twin_path, label, &DbRecoveryOptions::default(), false)
            .expect("independent healthy replacement-character main database");
        db.execute("INSERT INTO kv(key,value) VALUES('identity-sentinel','native-main')", [])
            .expect("native main sentinel");
        twin.execute("INSERT INTO kv(key,value) VALUES('identity-sentinel','twin-main')", [])
            .expect("twin main sentinel");
        for (path, connection) in [(&db_path, &db), (&twin_path, &twin)] {
            assert!(is_store_db_healthy(connection));
            assert_eq!(authority_marker_count(connection), 0);
            assert_absent(&automation_custody_recovery_marker_path(path));
        }
        let guard = acquire_automation_custody_recovery_guard(&db_path, &db)
            .expect("healthy native main is available");
        guard.require_available(&db).expect("held native-path authority remains available");
        let twin_guard = acquire_automation_custody_recovery_guard(&twin_path, &twin)
            .expect("healthy twin owns its independent authority");
        twin_guard.require_available(&twin).expect("held twin authority is available");
        let original = database_bytes(&db_path);
        let original_twin = database_bytes(&twin_path);

        for (path, wrong_connection, held_guard) in [
            (&db_path, &twin, &guard),
            (&twin_path, &db, &twin_guard),
        ] {
            let error = acquire_automation_custody_recovery_guard(path, wrong_connection)
                .expect_err("a replacement-character twin cannot authorize the other main");
            assert!(matches!(&error, AgentDbRecoveryError::AutomationCustodyUnavailable(
                "connection belongs to another store"
            )), "{error}");
            let error = held_guard.require_available(wrong_connection)
                .expect_err("held guard must retain native main identity");
            assert!(matches!(&error, AgentDbRecoveryError::AutomationCustodyUnavailable(
                "connection belongs to another store"
            )), "{error}");
        }
        guard.require_available(&db).expect("wrong-connection refusals preserve native authority");
        twin_guard.require_available(&twin).expect("wrong-connection refusals preserve twin authority");
        assert_eq!(database_bytes(&db_path), original);
        assert_eq!(database_bytes(&twin_path), original_twin);

        let marker = automation_custody_recovery_marker_path(&db_path);
        let marker_bytes = b"native-path custody unavailable";
        fs::write(&marker, marker_bytes).expect("owned native-path marker");
        for error in [
            guard.require_available(&db).expect_err("held guard must recheck the native marker"),
            acquire_automation_custody_recovery_guard(&db_path, &db)
                .expect_err("native marker denies a new checked guard"),
        ] {
            assert!(matches!(&error, AgentDbRecoveryError::AutomationCustodyUnavailable(
                "recovery marker is present"
            )), "{error}");
        }
        twin_guard.require_available(&twin).expect("native marker must not affect the twin");
        let twin_shared = acquire_automation_custody_recovery_guard(&twin_path, &twin)
            .expect("the twin still admits its own checked guard");
        assert_eq!(fs::read(&marker).expect("unchanged native marker"), marker_bytes);
        assert_absent(&automation_custody_recovery_marker_path(&twin_path));
        assert_eq!(database_bytes(&db_path), original);
        assert_eq!(database_bytes(&twin_path), original_twin);

        drop(twin_shared);
        drop(twin_guard);
        drop(guard);
        drop(twin);
        drop(db);
        fs::remove_dir_all(root).expect("remove completed owned native-main fixture");
    }
}

#[test]
fn automation_custody_guard_rejects_matching_attached_database() {
    let root = fixture_root("attached-main-identity");
    let db_path = root.join("store.db");
    let other_path = root.join("other.db");
    let db = open_configured_db(&db_path, "attached-target", &DbRecoveryOptions::default(), false)
        .expect("healthy target main database");
    let other = open_configured_db(&other_path, "attached-other", &DbRecoveryOptions::default(), false)
        .expect("healthy other main database");
    let memory = Connection::open_in_memory().expect("in-memory main database");
    initialize(&memory);
    db.execute("INSERT INTO kv(key,value) VALUES('attached-identity','actual-target-main')", [])
        .expect("actual target main sentinel");
    for connection in [&memory, &other] {
        connection.execute("ATTACH DATABASE ?1 AS matching_custody",
            params![db_path.to_str().expect("ordinary UTF-8 fixture path")])
            .expect("attach the actual target file to an unrelated main");
        let sentinel: String = connection.query_row(
            "SELECT value FROM matching_custody.kv WHERE key = 'attached-identity'",
            [], |row| row.get(0),
        ).expect("read the actual target through the attached schema");
        assert_eq!(sentinel, "actual-target-main");
    }
    let guard = acquire_automation_custody_recovery_guard(&db_path, &db)
        .expect("the actual target main remains available");
    guard.require_available(&db).expect("actual main recheck succeeds");
    let other_guard = acquire_automation_custody_recovery_guard(&other_path, &other)
        .expect("the other file owns authority only for its own main");
    let original = database_bytes(&db_path);
    let original_other = database_bytes(&other_path);
    for (connection, expected_reason) in [
        (&memory, "connection has no file identity"),
        (&other, "connection belongs to another store"),
    ] {
        let error = acquire_automation_custody_recovery_guard(&db_path, connection)
            .expect_err("an attached matching file cannot authorize a different main");
        assert!(matches!(&error, AgentDbRecoveryError::AutomationCustodyUnavailable(reason)
            if *reason == expected_reason), "{error}");
        let error = guard.require_available(connection)
            .expect_err("held guard must identify main, not an attached matching file");
        assert!(matches!(&error, AgentDbRecoveryError::AutomationCustodyUnavailable(reason)
            if *reason == expected_reason), "{error}");
    }
    guard.require_available(&db).expect("attached-file refusals preserve the real target authority");
    other_guard.require_available(&other).expect("the other main retains its own authority");
    assert_eq!(database_bytes(&db_path), original);
    assert_eq!(database_bytes(&other_path), original_other);

    drop(other_guard);
    drop(guard);
    drop(memory);
    drop(other);
    drop(db);
    fs::remove_dir_all(root).expect("remove completed owned attached-main fixture");
}

#[test]
fn every_regular_filesystem_marker_denies_new_and_held_availability() {
    for bytes in [b"automation custody unavailable\n".as_slice(), b"", b"{malformed\0marker"] {
        let root = fixture_root("marker-presence");
        let db_path = root.join("store.db");
        let db = open_configured_db(&db_path, "marker-presence", &DbRecoveryOptions::default(), false)
            .expect("healthy DB");
        let guard = acquire_automation_custody_recovery_guard(&db_path, &db)
            .expect("availability before marker");
        let marker = automation_custody_recovery_marker_path(&db_path);
        fs::write(&marker, bytes).expect("owned marker fixture");
        guard.require_available(&db).expect_err("a held guard must recheck marker presence");
        acquire_automation_custody_recovery_guard(&db_path, &db)
            .expect_err("any marker contents refuse new availability");
        assert_eq!(fs::read(&marker).expect("unchanged marker"), bytes);
        assert_eq!(authority_marker_count(&db), 0, "filesystem-only marker is sufficient");
        assert!(is_store_db_healthy(&db));
        drop(guard);
        drop(db);
        cleanup(&db_path);
        fs::remove_dir_all(root).expect("remove completed owned fixture");
    }
}

#[test]
fn kv_only_marker_denies_new_and_held_availability_for_any_value() {
    for value in ["unavailable", "", "{malformed"] {
        let root = fixture_root("kv-presence");
        let db_path = root.join("store.db");
        let db = open_configured_db(&db_path, "kv-presence", &DbRecoveryOptions::default(), false)
            .expect("healthy DB");
        let guard = acquire_automation_custody_recovery_guard(&db_path, &db)
            .expect("availability before KV marker");
        db.execute("INSERT INTO kv(key,value) VALUES(?1,?2)",
            params![AUTOMATION_CUSTODY_UNAVAILABLE_KV, value]).expect("KV-only marker");
        assert_absent(&automation_custody_recovery_marker_path(&db_path));
        guard.require_available(&db).expect_err("held guard must recheck KV marker presence");
        acquire_automation_custody_recovery_guard(&db_path, &db)
            .expect_err("KV marker alone refuses new availability");
        let retained: String = db.query_row("SELECT value FROM kv WHERE key = ?1",
            params![AUTOMATION_CUSTODY_UNAVAILABLE_KV], |row| row.get(0)).expect("retained KV marker");
        assert_eq!(retained, value);
        assert!(is_store_db_healthy(&db));
        drop(guard);
        drop(db);
        cleanup(&db_path);
        fs::remove_dir_all(root).expect("remove completed owned fixture");
    }
}

#[test]
fn repeated_corrupt_recovery_preserves_valid_empty_and_malformed_marker_bytes() {
    for marker_kind in ["valid", "empty", "malformed"] {
        let root = fixture_root(marker_kind);
        let db_path = root.join("store.db");
        fs::write(&db_path, b"first corrupt database").expect("initial corrupt DB");
        let db = open_configured_db(&db_path, marker_kind, &DbRecoveryOptions::default(), false)
            .expect("first foreground reconstruction");
        assert_recovered_authority_unavailable(&db_path, &db);
        drop(db);
        let marker = automation_custody_recovery_marker_path(&db_path);
        let expected = match marker_kind {
            "valid" => fs::read(&marker).expect("actual owner-created marker bytes"),
            "empty" => Vec::new(),
            "malformed" => b"{not-json\0partial-marker".to_vec(),
            _ => unreachable!(),
        };
        if marker_kind != "valid" {
            fs::write(&marker, &expected).expect("owned partial-marker fixture");
        }
        for corrupt_bytes in [b"second corrupt database".as_slice(), b"third corrupt database"] {
            fs::write(&db_path, corrupt_bytes).expect("repeated corruption fixture");
            let recovered = open_configured_db(&db_path, marker_kind, &DbRecoveryOptions::default(), false)
                .expect("existing regular marker permits foreground reconstruction");
            assert_eq!(fs::read(&marker).expect("retained marker"), expected,
                "recovery must resync existing marker without rewriting it");
            assert_recovered_authority_unavailable(&db_path, &recovered);
            drop(recovered);
        }
        cleanup(&db_path);
        fs::remove_dir_all(root).expect("remove completed owned fixture");
    }
}

#[test]
fn recovered_kv_marker_cannot_be_overwritten_by_salvage() {
    let root = fixture_root("salvage-marker");
    let db_path = root.join("store.db");
    let db = Connection::open(&db_path).expect("salvageable source DB");
    initialize(&db);
    db.execute("INSERT INTO kv(key,value) VALUES(?1,'stale-source-marker')",
        params![AUTOMATION_CUSTODY_UNAVAILABLE_KV]).expect("source marker");
    db.execute("INSERT INTO kv(key,value) VALUES('metadata','preserved-source-value')", [])
        .expect("source metadata");
    drop(db);

    // Invoke the real reconstruction owner over a readable source so salvage
    // must encounter the conflicting key after the new transaction commits.
    let recovered = recover_corrupt_store_db(
        &db_path, "salvage-marker", &DbRecoveryOptions::default(), "salvage conflict oracle",
    ).expect("real reconstruction with readable salvage source");
    assert_recovered_authority_unavailable(&db_path, &recovered);
    let marker: String = recovered.query_row("SELECT value FROM kv WHERE key = ?1",
        params![AUTOMATION_CUSTODY_UNAVAILABLE_KV], |row| row.get(0)).expect("fresh KV marker");
    assert_ne!(marker, "stale-source-marker", "salvage replaced recovery's authoritative marker");
    let metadata: String = recovered.query_row("SELECT value FROM kv WHERE key = 'metadata'",
        [], |row| row.get(0)).expect("salvaged metadata");
    assert_eq!(metadata, "preserved-source-value");
    // Simulate loss of only the sibling copy in this owned fixture. The real
    // recovery transaction's surviving KV marker must still deny authority.
    let filesystem_marker = automation_custody_recovery_marker_path(&db_path);
    fs::remove_file(&filesystem_marker).expect("inject filesystem-marker loss");
    assert_absent(&filesystem_marker);
    acquire_automation_custody_recovery_guard(&db_path, &recovered)
        .expect_err("the actual recovered KV marker independently denies custody");
    assert_eq!(authority_marker_count(&recovered), 1);
    drop(recovered);
    cleanup(&db_path);
    fs::remove_dir_all(root).expect("remove completed owned fixture");
}

#[test]
fn directory_main_refuses_before_creating_authority_or_quarantining_sentinels() {
    let root = fixture_root("directory-main");
    let db_path = root.join("store.db");
    seed_corrupt_db_and_sidecars(&db_path);
    let original = database_bytes(&db_path);
    fs::remove_file(&db_path).expect("replace owned main database fixture");
    fs::create_dir(&db_path).expect("nonregular main database path");
    let sentinel = db_path.join("owned-sentinel");
    fs::write(&sentinel, b"original main directory contents").expect("main directory sentinel");
    assert!(quarantine_paths(&db_path).is_empty());

    let error = recover_corrupt_store_db(
        &db_path, "directory-main", &DbRecoveryOptions::default(), "nonregular main oracle",
    ).expect_err("a directory main must refuse before recovery creates or moves anything");
    assert!(matches!(&error, AgentDbRecoveryError::Io(io_error)
        if io_error.kind() == io::ErrorKind::InvalidData), "{error:?}");
    assert!(fs::symlink_metadata(&db_path).expect("main remains at its original path").is_dir());
    assert_eq!(fs::read(&sentinel).expect("main sentinel remains in place"),
        b"original main directory contents");
    assert_eq!(fs::read_dir(&db_path).expect("retained main directory").count(), 1);
    for (path, expected) in original {
        if path != db_path {
            assert_eq!(Some(fs::read(&path).expect("unchanged main-adjacent sidecar")), expected,
                "recovery changed {}", path.display());
        }
    }
    assert_absent(&sibling_path(&db_path, ".automation-custody.guard"));
    assert_absent(&automation_custody_recovery_marker_path(&db_path));
    assert!(quarantine_paths(&db_path).is_empty(), "refusal must not relocate the main directory");
    fs::remove_dir_all(root).expect("remove completed owned fixture");
}

#[cfg(unix)]
#[test]
fn symlink_main_refuses_before_creating_authority_or_touching_targets() {
    for dangling in [false, true] {
        let root = fixture_root("symlink-main");
        let db_path = root.join("store.db");
        seed_corrupt_db_and_sidecars(&db_path);
        let original = database_bytes(&db_path);
        fs::remove_file(&db_path).expect("replace owned main database fixture");
        let target = root.join("owned-main-target");
        if !dangling {
            fs::write(&target, b"original main link target").expect("main link target sentinel");
        }
        std::os::unix::fs::symlink(&target, &db_path).expect("actual main database symlink");
        assert!(quarantine_paths(&db_path).is_empty());

        let error = recover_corrupt_store_db(
            &db_path, "symlink-main", &DbRecoveryOptions::default(), "main symlink oracle",
        ).expect_err("a main symlink must refuse before recovery creates or moves anything");
        assert!(matches!(&error, AgentDbRecoveryError::Io(io_error)
            if io_error.kind() == io::ErrorKind::InvalidData), "{error:?}");
        assert!(fs::symlink_metadata(&db_path).expect("main link remains in place")
            .file_type().is_symlink());
        assert_eq!(fs::read_link(&db_path).expect("unchanged original main link"), target);
        if dangling {
            assert_absent(&target);
        } else {
            assert_eq!(fs::read(&target).expect("unchanged original target"),
                b"original main link target");
        }
        for (path, expected) in original {
            if path != db_path {
                assert_eq!(Some(fs::read(&path).expect("unchanged main-adjacent sidecar")), expected,
                    "recovery changed {}", path.display());
            }
        }
        assert_absent(&sibling_path(&db_path, ".automation-custody.guard"));
        assert_absent(&automation_custody_recovery_marker_path(&db_path));
        assert!(quarantine_paths(&db_path).is_empty(), "refusal must not relocate the main link");
        fs::remove_dir_all(root).expect("remove completed owned fixture");
    }
}

#[test]
fn directory_marker_or_guard_refuses_without_quarantining_original_bytes() {
    for suffix in [".automation-custody-unavailable", ".automation-custody.guard"] {
        let root = fixture_root("directory-object");
        let db_path = root.join("store.db");
        let db = Connection::open(&db_path).expect("valid DB");
        initialize(&db);
        let object = sibling_path(&db_path, suffix);
        fs::create_dir(&object).expect("nonregular recovery path");
        let sentinel = object.join("owned-sentinel");
        fs::write(&sentinel, b"original directory child").expect("directory sentinel");
        acquire_automation_custody_recovery_guard(&db_path, &db)
            .expect_err("a directory is not valid recovery authority");
        drop(db);
        seed_corrupt_db_and_sidecars(&db_path);
        assert_recovery_refusal_preserves_files(&db_path);
        assert!(fs::symlink_metadata(&object).expect("retained directory").is_dir());
        assert_eq!(fs::read(&sentinel).expect("retained sentinel"), b"original directory child");
        cleanup(&db_path);
        fs::remove_dir_all(root).expect("remove completed owned fixture");
    }
}

#[cfg(unix)]
#[test]
fn symlink_marker_or_guard_refuses_without_touching_targets_or_database() {
    for suffix in [".automation-custody-unavailable", ".automation-custody.guard"] {
        for dangling in [false, true] {
            let root = fixture_root("symlink-object");
            let db_path = root.join("store.db");
            let db = Connection::open(&db_path).expect("valid DB");
            initialize(&db);
            let target = root.join("link-target");
            if !dangling {
                fs::write(&target, b"untouched symlink target").expect("symlink target");
            }
            let object = sibling_path(&db_path, suffix);
            std::os::unix::fs::symlink(&target, &object).expect("actual Unix symlink");
            acquire_automation_custody_recovery_guard(&db_path, &db)
                .expect_err("symlink presence cannot authorize availability");
            drop(db);
            seed_corrupt_db_and_sidecars(&db_path);
            assert_recovery_refusal_preserves_files(&db_path);
            assert_eq!(fs::read_link(&object).expect("symlink remains"), target);
            if dangling {
                assert_absent(&target);
            } else {
                assert_eq!(fs::read(&target).expect("retained link target"), b"untouched symlink target");
            }
            cleanup(&db_path);
            fs::remove_dir_all(root).expect("remove completed owned fixture");
        }
    }
}

// Required Linux Actions evidence supplies the raw-byte filesystem fixture;
// an ordinary macOS recovery pass is not evidence for this input class.
#[cfg(target_os = "linux")]
#[test]
fn recovery_preserves_native_filename_and_unrelated_replacement_character_twins() {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let root = fixture_root("native-filename");
    let db_path = root.join(OsString::from_vec(b"store-\xff.db".to_vec()));
    let twin = root.join("store-\u{fffd}.db");
    assert_ne!(db_path.as_os_str().as_bytes(), twin.as_os_str().as_bytes());
    seed_corrupt_db_and_sidecars(&db_path);
    let original = fs::read(&db_path).expect("original native-path bytes");
    fs::write(&twin, b"unrelated replacement-character database").expect("twin DB sentinel");
    for suffix in SQLITE_DB_SIDECAR_SUFFIXES {
        let directory = sibling_path(&twin, suffix);
        fs::create_dir(&directory).expect("distinct, unchecked twin sidecar directory");
        fs::write(directory.join("owned-sentinel"), format!("unrelated twin {suffix}"))
            .expect("twin directory sentinel");
    }

    let reported = Arc::new(Mutex::new(Vec::new()));
    let callback_reports = Arc::clone(&reported);
    let options = DbRecoveryOptions {
        on_corruption_recovered: Some(Arc::new(move |event| {
            callback_reports.lock().expect("callback receipt").push(event.quarantine_path.clone());
        })),
        ..DbRecoveryOptions::default()
    };
    let recovered = recover_corrupt_store_db(
        &db_path, "native-path-recovery", &options, "actual native-path recovery oracle",
    ).expect("valid native-byte files can be reconstructed");
    assert_recovered_authority_unavailable(&db_path, &recovered);
    let reports = reported.lock().expect("actual callback receipt");
    assert_eq!(reports.len(), 1, "recovery must publish exactly one actual quarantine receipt");
    let quarantine = reports[0].as_ref().expect("original corrupt DB must be quarantined");
    let canonical_parent = fs::canonicalize(&root).expect("actual owned parent identity");
    assert_eq!(quarantine.parent(), Some(canonical_parent.as_path()));
    let prefix = b"store-\xff.db.corrupt-";
    let name = quarantine.file_name().unwrap().as_bytes();
    let stamp = name.strip_prefix(prefix).expect("quarantine must retain the original native filename");
    assert!(!stamp.is_empty() && stamp.iter().all(u8::is_ascii_digit));
    assert_eq!(fs::read(quarantine).expect("exact original quarantine"), original);
    let matching_quarantines = fs::read_dir(&canonical_parent).expect("owned native directory")
        .map(|entry| entry.expect("owned entry").path())
        .filter(|entry| entry.file_name().unwrap().as_bytes().strip_prefix(prefix)
            .is_some_and(|suffix| !suffix.is_empty() && suffix.iter().all(u8::is_ascii_digit)))
        .collect::<Vec<_>>();
    assert_eq!(matching_quarantines, vec![quarantine.clone()]);

    assert_eq!(fs::read(&twin).expect("unrelated twin DB must remain"),
        b"unrelated replacement-character database");
    for suffix in SQLITE_DB_SIDECAR_SUFFIXES {
        let directory = sibling_path(&twin, suffix);
        assert!(fs::symlink_metadata(&directory).expect("unrelated twin directory").is_dir());
        assert_eq!(fs::read(directory.join("owned-sentinel")).expect("untouched twin sentinel"),
            format!("unrelated twin {suffix}").as_bytes());
        assert_eq!(fs::read_dir(directory).expect("twin directory entries").count(), 1);
    }
    drop(reports);
    drop(recovered);
    fs::remove_dir_all(root).expect("remove completed owned native-path fixture");
}

#[test]
fn recovery_refuses_missing_or_non_directory_parent_without_creating_authority() {
    let root = fixture_root("invalid-parent");
    let missing_parent = root.join("missing");
    let missing_db = missing_parent.join("store.db");
    recover_corrupt_store_db(&missing_db, "missing-parent", &DbRecoveryOptions::default(), "missing parent")
        .expect_err("recovery cannot create a missing authority parent");
    assert_absent(&missing_parent);
    let file_parent = root.join("file-parent");
    fs::write(&file_parent, b"original parent-object bytes").expect("invalid parent fixture");
    recover_corrupt_store_db(&file_parent.join("store.db"), "invalid-parent",
        &DbRecoveryOptions::default(), "non-directory parent")
        .expect_err("recovery cannot traverse a non-directory parent");
    assert_eq!(fs::read(&file_parent).expect("preserved parent object"), b"original parent-object bytes");
    fs::remove_dir_all(root).expect("remove completed owned fixture");
}

#[test]
fn directory_sidecars_refuse_before_moving_database_or_sentinel() {
    for suffix in SQLITE_DB_SIDECAR_SUFFIXES {
        let root = fixture_root("directory-sidecar");
        let db_path = root.join("store.db");
        seed_corrupt_db_and_sidecars(&db_path);
        let original = database_bytes(&db_path);
        let sidecar = sibling_path(&db_path, suffix);
        fs::remove_file(&sidecar).expect("replace owned sidecar fixture");
        fs::create_dir(&sidecar).expect("nonregular SQLite sidecar");
        let sentinel = sidecar.join("owned-sentinel");
        fs::write(&sentinel, b"original sidecar directory contents").expect("sidecar sentinel");
        let quarantines = quarantine_paths(&db_path);

        recover_corrupt_store_db(&db_path, "directory-sidecar", &DbRecoveryOptions::default(),
            "nonregular sidecar oracle").expect_err("sidecar type must refuse before quarantine");
        for (path, expected) in original {
            if path != sidecar {
                assert_eq!(Some(fs::read(&path).expect("preserved ordinary DB/sidecar")), expected,
                    "recovery changed {}", path.display());
            }
        }
        assert!(fs::symlink_metadata(&sidecar).expect("sidecar remains in place").is_dir());
        assert_eq!(fs::read(&sentinel).expect("sentinel remains in place"),
            b"original sidecar directory contents");
        assert_eq!(quarantine_paths(&db_path), quarantines);
        assert_absent(&automation_custody_recovery_marker_path(&db_path));
        fs::remove_dir_all(root).expect("remove completed owned fixture");
    }
}

#[cfg(unix)]
#[test]
fn symlink_sidecars_refuse_before_moving_database_or_touching_targets() {
    for suffix in SQLITE_DB_SIDECAR_SUFFIXES {
        for dangling in [false, true] {
            let root = fixture_root("symlink-sidecar");
            let db_path = root.join("store.db");
            seed_corrupt_db_and_sidecars(&db_path);
            let original = database_bytes(&db_path);
            let sidecar = sibling_path(&db_path, suffix);
            fs::remove_file(&sidecar).expect("replace owned sidecar fixture");
            let target = root.join("owned-link-target");
            if !dangling {
                fs::write(&target, b"untouched sidecar target").expect("sidecar target");
            }
            std::os::unix::fs::symlink(&target, &sidecar).expect("actual sidecar symlink");
            let quarantines = quarantine_paths(&db_path);

            recover_corrupt_store_db(&db_path, "symlink-sidecar", &DbRecoveryOptions::default(),
                "symlink sidecar oracle").expect_err("sidecar link must refuse before quarantine");
            for (path, expected) in original {
                if path != sidecar {
                    assert_eq!(Some(fs::read(&path).expect("preserved ordinary DB/sidecar")), expected,
                        "recovery changed {}", path.display());
                }
            }
            assert_eq!(fs::read_link(&sidecar).expect("sidecar link remains in place"), target);
            if dangling {
                assert_absent(&target);
            } else {
                assert_eq!(fs::read(&target).expect("original target remains"), b"untouched sidecar target");
            }
            assert_eq!(quarantine_paths(&db_path), quarantines);
            assert_absent(&automation_custody_recovery_marker_path(&db_path));
            fs::remove_dir_all(root).expect("remove completed owned fixture");
        }
    }
}

const GUARD_CHILD_TEST: &str = "automation_custody_public_guard_child";
const GUARD_CHILD_ROOT_ENV: &str = "FABUSHI_AGENT_DB_RECOVERY_CHILD_ROOT";
const GUARD_CHILD_NONCE_ENV: &str = "FABUSHI_AGENT_DB_RECOVERY_CHILD_NONCE";

fn child_receipt(nonce: &str, pid: u32, state: &str) -> String {
    format!("case={GUARD_CHILD_TEST}\nnonce={nonce}\npid={pid}\nstate={state}\n")
}

fn publish_child_receipt(root: &Path, name: &str, content: &str) {
    let temporary = root.join(format!("{name}.partial"));
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)
        .expect("exclusively create owned receipt");
    file.write_all(content.as_bytes()).expect("write complete owned receipt");
    drop(file);
    fs::rename(temporary, root.join(name)).expect("publish complete owned receipt");
}

struct OwnedRecoveryGuardChild {
    child: Child,
    root: PathBuf,
    terminal: Option<ExitStatus>,
    completed: bool,
}

impl OwnedRecoveryGuardChild {
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if self.terminal.is_none() {
            self.terminal = self.child.try_wait()?;
        }
        Ok(self.terminal)
    }

    fn require_alive(&mut self) {
        assert!(self.try_wait().expect("observe exact owned child").is_none(),
            "guard holder exited before exclusion was proved; root={}", self.root.display());
    }

    fn await_receipt(&mut self, name: &str, expected: &str) {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            self.require_alive();
            match fs::read_to_string(self.root.join(name)) {
                Ok(receipt) => {
                    assert_eq!(receipt, expected, "receipt must bind the exact case, nonce and Child PID");
                    return;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => panic!("read owned child receipt: {error}; root={}", self.root.display()),
            }
            assert!(Instant::now() < deadline, "owned guard child did not publish {name}; root={}",
                self.root.display());
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn await_terminal(&mut self) -> ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.try_wait().expect("observe exact child terminal exit") {
                return status;
            }
            assert!(Instant::now() < deadline, "owned guard child did not exit; root={}", self.root.display());
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for OwnedRecoveryGuardChild {
    fn drop(&mut self) {
        match self.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                // Only this retained Child handle can stop its process. An
                // unsuccessful kill/reap never grants filesystem cleanup.
                if let Err(error) = self.child.kill() {
                    eprintln!("[agent-db-recovery] exact-child kill failed pid={}: {error}", self.child.id());
                }
                let deadline = Instant::now() + Duration::from_secs(2);
                loop {
                    match self.try_wait() {
                        Ok(Some(_)) => break,
                        Ok(None) if Instant::now() < deadline => {
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        Ok(None) => break,
                        Err(error) => {
                            eprintln!("[agent-db-recovery] exact-child reap unresolved: {error}");
                            break;
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("[agent-db-recovery] child ownership observation unresolved: {error}");
            }
        }
        if self.completed && self.terminal.is_some_and(|status| status.success()) {
            cleanup(&self.root.join("store.db"));
            if let Err(error) = fs::remove_dir_all(&self.root) {
                eprintln!("[agent-db-recovery] completed fixture cleanup failed: {error}; root={}",
                    self.root.display());
            }
        } else {
            eprintln!("[agent-db-recovery] retained incomplete fixture child_pid={} terminal={:?} root={}",
                self.child.id(), self.terminal, self.root.display());
        }
    }
}

#[test]
#[ignore = "invoked only as the exact bounded child of the cross-process recovery oracle"]
fn automation_custody_public_guard_child() {
    let root = PathBuf::from(std::env::var_os(GUARD_CHILD_ROOT_ENV).expect("owned child root"));
    let nonce = std::env::var(GUARD_CHILD_NONCE_ENV).expect("owned child nonce");
    let db_path = root.join("store.db");
    let db = open_configured_db(&db_path, "guard-child", &DbRecoveryOptions::default(), false)
        .expect("child opens the real configured database");
    let guard = acquire_automation_custody_recovery_guard(&db_path, &db)
        .expect("child owns the public shared guard");
    guard.require_available(&db).expect("child checked actual authority");
    db.execute("INSERT INTO kv(key,value) VALUES('guard-child-nonce',?1)", params![nonce])
        .expect("write actual WAL-backed fixture data");
    let pid = std::process::id();
    publish_child_receipt(&root, "held", &child_receipt(&nonce, pid, "held"));

    let expected_release = child_receipt(&nonce, pid, "release");
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        match fs::read_to_string(root.join("release")) {
            Ok(receipt) => {
                assert_eq!(receipt, expected_release, "only the exact parent's release is accepted");
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => panic!("read exact-parent release: {error}"),
        }
        assert!(Instant::now() < deadline, "guard child timed out awaiting its exact release");
        std::thread::sleep(Duration::from_millis(10));
    }
    guard.require_available(&db).expect("refused recovery left the held authority available");
    drop(guard);
    drop(db);
    publish_child_receipt(&root, "completed", &child_receipt(&nonce, pid, "completed"));
}

#[test]
fn child_held_public_guard_blocks_recovery_until_exact_terminal_exit() {
    let root = fixture_root("child-exclusion");
    let db_path = root.join("store.db");
    let db = open_configured_db(&db_path, "guard-parent", &DbRecoveryOptions::default(), false)
        .expect("parent initializes real DB");
    drop(db);
    let nonce = Uuid::new_v4().to_string();
    let child = Command::new(std::env::current_exe().expect("current contract executable"))
        .args(["--exact", GUARD_CHILD_TEST, "--ignored", "--nocapture", "--test-threads=1"])
        .env(GUARD_CHILD_ROOT_ENV, &root)
        .env(GUARD_CHILD_NONCE_ENV, &nonce)
        .spawn()
        .unwrap_or_else(|error| panic!("launch owned recovery child: {error}; root={}", root.display()));
    let child_pid = child.id();
    eprintln!("[agent-db-recovery] launched parent_pid={} child_pid={child_pid} nonce={nonce} root={}",
        std::process::id(), root.display());
    let mut owned = OwnedRecoveryGuardChild { child, root, terminal: None, completed: false };
    owned.await_receipt("held", &child_receipt(&nonce, child_pid, "held"));

    let observer = Connection::open(&db_path).expect("parent observation connection");
    let shared = acquire_automation_custody_recovery_guard(&db_path, &observer)
        .expect("public guard remains shareable across processes");
    shared.require_available(&observer).expect("parent observes healthy authority");
    drop(shared);
    drop(observer);
    let guard_path = sibling_path(&db_path, ".automation-custody.guard");
    let original_guard_file = OpenOptions::new().read(true).write(true).open(&guard_path)
        .expect("open actual stable guard without recreating it");
    fs2::FileExt::try_lock_exclusive(&original_guard_file)
        .expect_err("child's public guard must hold a real cross-process OS lock");
    for suffix in ["-wal", "-shm"] {
        assert!(!fs::read(sibling_path(&db_path, suffix)).expect("real SQLite sidecar").is_empty(),
            "exclusion must preserve actual {suffix} bytes");
    }
    assert_absent(&automation_custody_recovery_marker_path(&db_path));
    assert!(quarantine_paths(&db_path).is_empty());
    owned.require_alive();
    assert_recovery_refusal_preserves_files(&db_path);
    owned.require_alive();
    assert_absent(&automation_custody_recovery_marker_path(&db_path));

    publish_child_receipt(&owned.root, "release", &child_receipt(&nonce, child_pid, "release"));
    let status = owned.await_terminal();
    assert!(status.success(), "the exact guard child must exit successfully: {status}");
    assert_eq!(fs::read_to_string(owned.root.join("completed")).expect("child completion receipt"),
        child_receipt(&nonce, child_pid, "completed"),
        "neither a receipt alone nor an empty selected-test run proves completion");

    let recovered = recover_corrupt_store_db(
        &db_path, "guard-parent", &DbRecoveryOptions::default(), "post-child recovery oracle",
    ).expect("recovery is admitted only after the exact child has ended");
    assert_recovered_authority_unavailable(&db_path, &recovered);
    let salvaged_nonce: String = recovered.query_row("SELECT value FROM kv WHERE key = 'guard-child-nonce'",
        [], |row| row.get(0)).expect("real child-written value survives salvage");
    assert_eq!(salvaged_nonce, nonce);

    // Retaining the original descriptor proves recovery did not replace the
    // lock object while preserving only its pathname.
    fs2::FileExt::try_lock_exclusive(&original_guard_file)
        .expect("old guard descriptor is released after recovery");
    let reopened_guard = OpenOptions::new().read(true).write(true).open(&guard_path)
        .expect("reopen stable sibling guard");
    fs2::FileExt::try_lock_exclusive(&reopened_guard)
        .expect_err("recovery must retain the very same lock object");
    fs2::FileExt::unlock(&original_guard_file).expect("release test's exclusive observation");
    drop(reopened_guard);
    drop(original_guard_file);
    drop(recovered);
    owned.completed = true;
}
