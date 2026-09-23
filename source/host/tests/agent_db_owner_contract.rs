use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_db::{
    SandAgentDb, SandAgentDbOptions, compare_and_set_persisted_latest_root_blob_id,
    set_persisted_awaiting_user_response, set_persisted_sand_profile,
};
use mahayana_host_runtime::extensions::session::agent_db_recovery::DbRecoveryOptions;
use mahayana_host_runtime::extensions::session::agent_db_serde::{
    AwaitingUserResponse, SandProfile,
};
use mahayana_host_runtime::storage::store_db::live_db_handle_count;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-agent-db-owner-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn owner_registers_live_handle_notifies_shipping_mutations_and_releases_on_close() {
    let root = temp_root("lifecycle");
    let agent_dir = root.join("agent-owner");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");

    let owner = SandAgentDb::open(&db_path, 50).expect("owner");
    assert_eq!(live_db_handle_count(&db_path), 1);

    let metadata_hits = Arc::new(AtomicUsize::new(0));
    let profile_hits = Arc::new(AtomicUsize::new(0));
    let awaiting_hits = Arc::new(AtomicUsize::new(0));

    let _metadata_sub = owner.subscribe_metadata(
        "latestRootBlobId",
        {
            let hits = Arc::clone(&metadata_hits);
            Arc::new(move || {
                hits.fetch_add(1, Ordering::SeqCst);
            })
        },
    );
    let _profile_sub = owner.subscribe_sand_profile({
        let hits = Arc::clone(&profile_hits);
        Arc::new(move || {
            hits.fetch_add(1, Ordering::SeqCst);
        })
    });
    let _awaiting_sub = owner.subscribe_awaiting_user_response({
        let hits = Arc::clone(&awaiting_hits);
        Arc::new(move || {
            hits.fetch_add(1, Ordering::SeqCst);
        })
    });

    assert!(compare_and_set_persisted_latest_root_blob_id(
        &db_path,
        50,
        &[],
        &[0xaa, 0xbb],
    )
    .expect("root cas"));
    assert!(set_persisted_sand_profile(
        &db_path,
        50,
        &SandProfile {
            description: "profile".into(),
            avatar_path: Some("/tmp/avatar.png".into()),
        },
    )
    .expect("profile"));
    assert!(set_persisted_awaiting_user_response(
        &db_path,
        50,
        Some(&AwaitingUserResponse {
            tab_id: "tab-a".into(),
            reason: "approval".into(),
            since: 1.0,
        }),
    )
    .expect("awaiting"));

    assert_eq!(metadata_hits.load(Ordering::SeqCst), 1);
    assert_eq!(profile_hits.load(Ordering::SeqCst), 1);
    assert_eq!(awaiting_hits.load(Ordering::SeqCst), 1);

    owner.close(true);
    assert!(owner.is_closed());
    assert_eq!(live_db_handle_count(&db_path), 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn owner_drops_busy_writes_and_reports_the_operation() {
    let root = temp_root("busy");
    let agent_dir = root.join("agent-busy");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");

    let observed = Arc::new(Mutex::new(Vec::<String>::new()));
    let callback_observed = Arc::clone(&observed);
    let owner = SandAgentDb::open_with_options(
        &db_path,
        SandAgentDbOptions {
            recovery: DbRecoveryOptions {
                busy_timeout_ms: 20,
                ..DbRecoveryOptions::default()
            },
            on_busy_error: Some(Arc::new(move |operation, error| {
                callback_observed
                    .lock()
                    .expect("busy events")
                    .push(format!("{operation}:{error}"));
            })),
        },
    )
    .expect("owner");

    let locker = rusqlite::Connection::open(&db_path).expect("locker");
    locker.execute_batch("BEGIN IMMEDIATE").expect("write lock");
    assert!(!owner.write_kv("busy-test", "value").expect("busy write"));
    locker.execute_batch("ROLLBACK").expect("unlock");
    drop(locker);

    let observed = observed.lock().expect("busy observations");
    assert_eq!(observed.len(), 1);
    assert!(observed[0].starts_with("writeKv:busy-test:"));
    drop(observed);

    owner.close(false);
    assert_eq!(live_db_handle_count(&db_path), 0);
    let _ = fs::remove_dir_all(root);
}
