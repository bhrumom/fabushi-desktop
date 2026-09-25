use std::fs;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use mahayana_host_runtime::extensions::box_store_sync::chrome_session_stage::{
    ChromeStageRetryPolicy, stage_box_chrome_session_with,
};
use mahayana_host_runtime::extensions::box_store_sync::sqlite_snapshot::{
    SqliteSnapshotFailure, SqliteSnapshotOperation, SqliteSnapshotPathStage,
};
use uuid::Uuid;

fn scratch() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "fabushi-chrome-stage-{}",
        Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&root).expect("create scratch");
    root
}

fn failure(cause: &str) -> SqliteSnapshotFailure {
    SqliteSnapshotFailure {
        operation: SqliteSnapshotOperation::VacuumInto,
        path_stage: SqliteSnapshotPathStage::SourceMain,
        cause: cause.into(),
        error_class: "Error".into(),
        errno: None,
        sqlite_code: None,
    }
}

#[test]
fn stages_existing_database_and_reports_mode() {
    let root = scratch();
    let source_dir = root.join("Default");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("Cookies"), b"cookie-db").expect("seed db");

    let reports = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&reports);
    let staged = stage_box_chrome_session_with(
        &source_dir,
        "home/box/chrome-profile/Default",
        &["Cookies", "Missing"],
        ChromeStageRetryPolicy {
            max_attempts: 1,
            delay_ms: 0,
        },
        |src, dest| {
            fs::copy(src, dest).map(|_| ()).map_err(|_| failure("io"))
        },
        |_src, _dest| Err(failure("should-not-copy")),
        |_| {},
        move |report| capture.lock().expect("report lock").push(report),
    )
    .expect("stage chrome");

    assert_eq!(staged.files.len(), 1);
    assert_eq!(
        staged.files[0].rel_path,
        "home/box/chrome-profile/Default/Cookies"
    );
    assert!(staged.files[0].abs_path.is_file());
    assert_eq!(staged.skipped, 0);
    let reports = reports.lock().expect("reports");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].staged, 1);
    assert_eq!(reports[0].skipped, 0);
    drop(reports);

    staged.cleanup().expect("cleanup stage");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn busy_vacuum_uses_locked_copy_fallback() {
    let root = scratch();
    let source_dir = root.join("Default");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("Login Data"), b"locked-db").expect("seed db");

    let copies = Arc::new(AtomicUsize::new(0));
    let copies_for_closure = Arc::clone(&copies);
    let logs = Arc::new(Mutex::new(Vec::<String>::new()));
    let logs_for_closure = Arc::clone(&logs);

    let staged = stage_box_chrome_session_with(
        &source_dir,
        "profile",
        &["Login Data"],
        ChromeStageRetryPolicy {
            max_attempts: 1,
            delay_ms: 0,
        },
        |_src, _dest| Err(failure("busy")),
        move |src, dest| {
            copies_for_closure.fetch_add(1, Ordering::SeqCst);
            fs::copy(src, dest).map(|_| ()).map_err(|_| failure("io"))
        },
        move |message| logs_for_closure.lock().expect("logs").push(message),
        |_| {},
    )
    .expect("stage locked chrome db");

    assert_eq!(copies.load(Ordering::SeqCst), 1);
    assert_eq!(staged.files.len(), 1);
    assert_eq!(staged.skipped, 0);
    assert!(logs
        .lock()
        .expect("logs")
        .iter()
        .any(|message| message.contains("raw-copy fallback")));

    staged.cleanup().expect("cleanup stage");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn nonbusy_failure_is_skipped_and_reported() {
    let root = scratch();
    let source_dir = root.join("Default");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("Web Data"), b"db").expect("seed db");

    let reports = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&reports);
    let staged = stage_box_chrome_session_with(
        &source_dir,
        "profile",
        &["Web Data"],
        ChromeStageRetryPolicy {
            max_attempts: 1,
            delay_ms: 0,
        },
        |_src, _dest| Err(failure("corrupt")),
        |_src, _dest| panic!("raw copy should only run for busy/locked failures"),
        |_| {},
        move |report| capture.lock().expect("report lock").push(report),
    )
    .expect("stage with skip");

    assert!(staged.files.is_empty());
    assert_eq!(staged.skipped, 1);
    let reports = reports.lock().expect("reports");
    assert_eq!(reports[0].skipped_db_names, vec!["Web Data"]);
    assert_eq!(
        reports[0].failure.as_ref().expect("failure").phase,
        "vacuum"
    );
    drop(reports);

    staged.cleanup().expect("cleanup stage");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn busy_failure_retries_when_policy_allows_it() {
    let root = scratch();
    let source_dir = root.join("Default");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("Cookies"), b"db").expect("seed db");

    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_closure = Arc::clone(&attempts);
    let staged = stage_box_chrome_session_with(
        &source_dir,
        "profile",
        &["Cookies"],
        ChromeStageRetryPolicy {
            max_attempts: 2,
            delay_ms: 0,
        },
        move |src, dest| {
            if attempts_for_closure.fetch_add(1, Ordering::SeqCst) == 0 {
                fs::write(dest, b"partial").expect("partial first attempt");
                return Err(failure("busy"));
            }
            fs::copy(src, dest).map(|_| ()).map_err(|_| failure("io"))
        },
        |_src, _dest| panic!("retry should succeed before raw copy"),
        |_| {},
        |_| {},
    )
    .expect("retry stage");

    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert_eq!(staged.files.len(), 1);
    assert_eq!(fs::read(&staged.files[0].abs_path).expect("staged bytes"), b"db");

    staged.cleanup().expect("cleanup stage");
    let _ = fs::remove_dir_all(root);
}
