use std::cell::{Cell, RefCell};
use std::fs;
use std::path::PathBuf;

use mahayana_host_runtime::extensions::telemetry::host_crash_marker::{
    DeleteIfUnchangedResult, FileHostCrashMarkerStore, ForwardHostCrashMarkerResult,
    HostCrashErrorClass, HostCrashMarkerDelete, HostCrashMarkerRead, HostCrashMarkerStore,
    delete_if_unchanged, forward_host_crash_marker_with, host_crash_marker_metadata,
    parse_host_crash_marker,
};

fn signal_marker_raw() -> &'static str {
    r#"{"schemaVersion":1,"errorClass":"signal_exit","exitSignal":"SIGSEGV","startedAtMs":1000.2,"crashedAtMs":2400.7,"uptimeMs":1400.5}"#
}

#[test]
fn parses_frozen_marker_variants_and_rejects_invalid_combinations() {
    let marker = parse_host_crash_marker(signal_marker_raw()).unwrap();
    assert_eq!(marker.error_class, HostCrashErrorClass::SignalExit);
    assert_eq!(marker.exit_signal, "SIGSEGV");
    assert_eq!(marker.started_at_ms, Some(1000.2));
    assert_eq!(marker.crashed_at_ms, 2400.7);
    assert_eq!(marker.uptime_ms, Some(1400.5));

    let nonzero = parse_host_crash_marker(
        r#"{"schemaVersion":1,"errorClass":"nonzero_exit","exitSignal":"none","startedAtMs":1,"crashedAtMs":2,"uptimeMs":1}"#,
    )
    .unwrap();
    assert_eq!(nonzero.error_class, HostCrashErrorClass::NonzeroExit);

    let unobserved = parse_host_crash_marker(
        r#"{"schemaVersion":1,"errorClass":"unobserved_exit","exitSignal":"unknown","crashedAtMs":42}"#,
    )
    .unwrap();
    assert_eq!(unobserved.error_class, HostCrashErrorClass::UnobservedExit);
    assert_eq!(unobserved.started_at_ms, None);
    assert_eq!(unobserved.uptime_ms, None);

    assert!(parse_host_crash_marker(
        r#"{"schemaVersion":1,"errorClass":"signal_exit","exitSignal":"none","startedAtMs":1,"crashedAtMs":2,"uptimeMs":1}"#
    )
    .is_none());
    assert!(parse_host_crash_marker(
        r#"{"schemaVersion":1,"errorClass":"unobserved_exit","exitSignal":"unknown","crashedAtMs":-1}"#
    )
    .is_none());
    assert!(parse_host_crash_marker("not-json").is_none());
}

#[test]
fn projects_frozen_metadata_with_rounded_millisecond_values() {
    let marker = parse_host_crash_marker(signal_marker_raw()).unwrap();
    let metadata = host_crash_marker_metadata(&marker);
    assert_eq!(metadata.get("kind").map(String::as_str), Some("process_exit"));
    assert_eq!(metadata.get("error_class").map(String::as_str), Some("signal_exit"));
    assert_eq!(metadata.get("exit_signal").map(String::as_str), Some("SIGSEGV"));
    assert_eq!(metadata.get("started_at_ms").map(String::as_str), Some("1000"));
    assert_eq!(metadata.get("crashed_at_ms").map(String::as_str), Some("2401"));
    assert_eq!(metadata.get("uptime_ms").map(String::as_str), Some("1401"));
}

fn temp_marker_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "fabushi-host-crash-marker-{}-{name}.json",
        std::process::id()
    ))
}

#[test]
fn file_store_distinguishes_absent_present_and_delete() {
    let path = temp_marker_path("file-store");
    let _ = fs::remove_file(&path);
    let store = FileHostCrashMarkerStore::new(&path);
    assert_eq!(store.read(), HostCrashMarkerRead::Absent);

    fs::write(&path, signal_marker_raw()).unwrap();
    assert_eq!(
        store.read(),
        HostCrashMarkerRead::Present(signal_marker_raw().into())
    );
    assert_eq!(store.delete(), HostCrashMarkerDelete::Deleted);
    assert_eq!(store.read(), HostCrashMarkerRead::Absent);
}

struct MemoryStore {
    reads: RefCell<Vec<HostCrashMarkerRead>>,
    delete_result: HostCrashMarkerDelete,
    delete_calls: Cell<usize>,
}

impl MemoryStore {
    fn new(reads: Vec<HostCrashMarkerRead>, delete_result: HostCrashMarkerDelete) -> Self {
        Self {
            reads: RefCell::new(reads),
            delete_result,
            delete_calls: Cell::new(0),
        }
    }
}

impl HostCrashMarkerStore for MemoryStore {
    fn read(&self) -> HostCrashMarkerRead {
        let mut reads = self.reads.borrow_mut();
        if reads.is_empty() {
            return HostCrashMarkerRead::Absent;
        }
        reads.remove(0)
    }

    fn delete(&self) -> HostCrashMarkerDelete {
        self.delete_calls.set(self.delete_calls.get() + 1);
        self.delete_result
    }
}

#[test]
fn unchanged_delete_is_race_safe() {
    let store = MemoryStore::new(
        vec![HostCrashMarkerRead::Present("new-marker".into())],
        HostCrashMarkerDelete::Deleted,
    );
    assert_eq!(
        delete_if_unchanged(&store, "old-marker"),
        DeleteIfUnchangedResult::Changed
    );
    assert_eq!(store.delete_calls.get(), 0);
}

#[test]
fn forwarding_preserves_defer_parse_dedupe_and_changed_marker_semantics() {
    let unavailable = MemoryStore::new(
        vec![HostCrashMarkerRead::Unavailable],
        HostCrashMarkerDelete::Deleted,
    );
    assert_eq!(
        forward_host_crash_marker_with(&unavailable, |_| false, |_| {}, |_| true),
        ForwardHostCrashMarkerResult::Deferred
    );

    let parse_error = MemoryStore::new(
        vec![
            HostCrashMarkerRead::Present("bad".into()),
            HostCrashMarkerRead::Present("bad".into()),
        ],
        HostCrashMarkerDelete::Deleted,
    );
    let marked = Cell::new(false);
    assert_eq!(
        forward_host_crash_marker_with(
            &parse_error,
            |_| false,
            |_| marked.set(true),
            |_| panic!("invalid marker must not emit"),
        ),
        ForwardHostCrashMarkerResult::ParseError
    );
    assert!(marked.get());

    let delivery_deferred = MemoryStore::new(
        vec![HostCrashMarkerRead::Present(signal_marker_raw().into())],
        HostCrashMarkerDelete::Deleted,
    );
    assert_eq!(
        forward_host_crash_marker_with(&delivery_deferred, |_| false, |_| {}, |_| false),
        ForwardHostCrashMarkerResult::Deferred
    );
    assert_eq!(delivery_deferred.delete_calls.get(), 0);

    let delivered = MemoryStore::new(
        vec![
            HostCrashMarkerRead::Present(signal_marker_raw().into()),
            HostCrashMarkerRead::Present(signal_marker_raw().into()),
        ],
        HostCrashMarkerDelete::Deleted,
    );
    let marked = Cell::new(false);
    assert_eq!(
        forward_host_crash_marker_with(
            &delivered,
            |_| false,
            |_| marked.set(true),
            |_| true,
        ),
        ForwardHostCrashMarkerResult::Delivered
    );
    assert!(marked.get());
    assert_eq!(delivered.delete_calls.get(), 1);

    let changed = MemoryStore::new(
        vec![
            HostCrashMarkerRead::Present(signal_marker_raw().into()),
            HostCrashMarkerRead::Present("replacement".into()),
        ],
        HostCrashMarkerDelete::Deleted,
    );
    assert_eq!(
        forward_host_crash_marker_with(&changed, |_| true, |_| {}, |_| true),
        ForwardHostCrashMarkerResult::Pending
    );
    assert_eq!(changed.delete_calls.get(), 0);
}
