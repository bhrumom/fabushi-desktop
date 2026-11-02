use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::connector_secret_store::{
    ConnectorSecretStoreError, SandConnectorSecretStore,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::session_diagnostics::{
    pin_session_diagnostics_reporter, SessionDiagnostic,
};
use serde_json::{json, Value};

const STORED_SENTINEL: &str = "ccwa_stored_credential_do_not_report";
const SUBMITTED_SENTINEL: &str = "ccwa_submitted_credential_do_not_report";

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-connector-secrets-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn assert_secret_free(text: &str) {
    assert!(
        !text.contains(STORED_SENTINEL) && !text.contains(SUBMITTED_SENTINEL),
        "credential refusal must not expose either secret"
    );
}

fn assert_read_admission_error(error: &ConnectorSecretStoreError, expected: io::ErrorKind) {
    assert!(
        matches!(error, ConnectorSecretStoreError::Io(error) if error.kind() == expected),
        "write admission must retain the bounded I/O error class"
    );
    assert_secret_free(&error.to_string());
    assert_secret_free(&format!("{error:?}"));
}

fn assert_file_bytes(path: &Path, expected: &[u8]) {
    let actual = fs::read(path).expect("read fixture bytes");
    assert!(actual == expected, "fixture bytes must remain exactly as expected");
}

fn directory_entries(path: &Path) -> Vec<PathBuf> {
    let mut entries = fs::read_dir(path)
        .expect("read fixture directory")
        .map(|entry| entry.expect("fixture entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

struct ResetDiagnostics;

impl Drop for ResetDiagnostics {
    fn drop(&mut self) {
        pin_session_diagnostics_reporter(None);
    }
}

#[cfg(unix)]
struct RestorePermissions {
    path: PathBuf,
    original: fs::Permissions,
}

#[cfg(unix)]
impl RestorePermissions {
    fn restore(&self) -> io::Result<()> {
        fs::set_permissions(&self.path, self.original.clone())
    }
}

#[cfg(unix)]
impl Drop for RestorePermissions {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[test]
fn connector_secret_store_matches_frozen_validation_merge_and_remove_contract() {
    let root = temp_root("store");
    let store = SandConnectorSecretStore::new(&root);

    assert!(!store
        .set_secret("../agent", "slack", "token", "bad")
        .expect("invalid agent"));
    assert!(!store
        .set_secret("agent-a", "../slack", "token", "bad")
        .expect("invalid platform"));
    assert!(!store
        .set_secret("agent-a", "slack", "", "bad")
        .expect("invalid field"));
    assert!(!root.exists(), "invalid input must not create a store directory");

    assert!(store
        .set_secret("agent-a", "slack", "token", "secret")
        .expect("set token"));
    assert!(store
        .set_secret("agent-a", "slack", "team", "team-a")
        .expect("merge team"));
    assert_eq!(
        store.get_secret("agent-a", "slack", "token").as_deref(),
        Some("secret")
    );
    assert_eq!(
        store.get_secret("agent-a", "slack", "team").as_deref(),
        Some("team-a")
    );
    assert!(fs::read_to_string(store.file_path("agent-a", "slack"))
        .expect("secret json")
        .ends_with('\n'));

    assert!(store
        .set_secret("agent-a", "slack", "empty", "")
        .expect("set empty"));
    assert_eq!(store.get_secret("agent-a", "slack", "empty"), None);

    fs::write(store.file_path("agent-a", "slack"), "not-json")
        .expect("corrupt secret file");
    assert!(store.read("agent-a", "slack").is_empty());

    assert!(store
        .set_secret("agent-a", "slack", "token", "recovered")
        .is_err());
    assert_file_bytes(&store.file_path("agent-a", "slack"), b"not-json");
    assert!(store
        .remove_agent_platform("agent-a", "slack")
        .expect("explicitly remove unreadable platform"));
    assert!(store
        .set_secret("agent-a", "slack", "token", "recovered")
        .expect("recover write"));
    assert!(store
        .remove_agent_platform("agent-a", "slack")
        .expect("remove platform"));
    assert_eq!(store.get_secret("agent-a", "slack", "token"), None);
    assert!(!store
        .remove_agent_platform("agent-a", "slack")
        .expect("idempotent remove"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn connector_secret_write_admission_preserves_invalid_images_and_redacts_errors() {
    let root = temp_root("invalid-images");
    let store = SandConnectorSecretStore::new(&root);
    let agent_id = "ccwa-invalid-images";
    fs::create_dir_all(root.join(agent_id)).expect("fixture directory");
    let captured = Arc::new(Mutex::new(Vec::<SessionDiagnostic>::new()));
    let captured_for_reporter = Arc::clone(&captured);
    pin_session_diagnostics_reporter(Some(Arc::new(move |report| {
        if report.metadata.get("agentId").and_then(Value::as_str) == Some(agent_id) {
            captured_for_reporter.lock().expect("diagnostics lock").push(report.clone());
        }
    })));
    let reporter_guard = ResetDiagnostics;

    let mut invalid_utf8 = format!("{{\"token\":\"{STORED_SENTINEL}\",\"bad\":\"").into_bytes();
    invalid_utf8.extend_from_slice(&[0xff, b'"', b'}']);
    let cases = [
        ("empty", Vec::new(), "Eof"),
        ("whitespace", b" \n\t".to_vec(), "Eof"),
        ("malformed", format!("{{\"token\":\"{STORED_SENTINEL}\",\"bad\":}}").into_bytes(), "Syntax"),
        ("truncated", format!("{{\"token\":\"{STORED_SENTINEL}\"").into_bytes(), "Eof"),
        ("string", format!("\"{STORED_SENTINEL}\"").into_bytes(), "NonObject"),
        ("array", format!("[\"{STORED_SENTINEL}\"]").into_bytes(), "NonObject"),
        ("null", b"null".to_vec(), "NonObject"),
        ("number", b"42".to_vec(), "NonObject"),
        ("boolean", b"true".to_vec(), "NonObject"),
        ("utf8", invalid_utf8, "InvalidData"),
    ];
    for (platform, raw, error_class) in cases {
        let path = store.file_path(agent_id, platform);
        fs::write(&path, &raw).expect("write unreadable fixture image");
        let before_entries = directory_entries(path.parent().expect("fixture parent"));
        captured.lock().expect("diagnostics lock").clear();

        let error = store
            .set_secret(agent_id, platform, "token", SUBMITTED_SENTINEL)
            .expect_err("unknown credential image must be refused");
        assert_read_admission_error(&error, io::ErrorKind::InvalidData);
        let reopened = SandConnectorSecretStore::new(&root);
        let reopened_error = reopened
            .set_secret(agent_id, platform, "token", SUBMITTED_SENTINEL)
            .expect_err("a fresh store owner must also refuse the same image");
        assert_read_admission_error(&reopened_error, io::ErrorKind::InvalidData);
        assert_file_bytes(&path, &raw);
        assert!(
            directory_entries(path.parent().expect("fixture parent")) == before_entries,
            "rejection must leave no temporary file or extra directory"
        );
        let reports = captured.lock().expect("diagnostics lock").clone();
        assert!(reports.len() == 2, "each rejected write must report one bounded diagnostic");
        for report in reports {
            assert!(
                report.family == "store_db"
                    && report.kind == "connector_secrets_unreadable"
                    && report.metadata.len() == 2
                    && report.metadata.get("agentId").and_then(Value::as_str) == Some(agent_id)
                    && report.metadata.get("errorClass").and_then(Value::as_str) == Some(error_class),
                "admission diagnostics must retain only Agent identity and the bounded class"
            );
            assert_secret_free(&format!("{report:?}"));
        }

        assert!(reopened.read(agent_id, platform).is_empty());
        assert!(reopened.get_secret(agent_id, platform, "token").is_none());
        assert_file_bytes(&path, &raw);
    }

    drop(reporter_guard);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn connector_secret_write_admission_merges_valid_objects_and_reopens() {
    let root = temp_root("valid-objects");
    let store = SandConnectorSecretStore::new(&root);
    let path = store.file_path("agent-a", "slack");
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directory");
    let mut expected = json!({
        "token": STORED_SENTINEL,
        "nested": {"enabled": true, "values": [1, null, {"preserved": STORED_SENTINEL}]},
        "count": 7,
        "optional": null
    });
    fs::write(&path, serde_json::to_vec(&expected).expect("fixture JSON")).expect("valid fixture");

    assert!(store.set_secret("agent-a", "slack", "token", SUBMITTED_SENTINEL).expect("replace known field"));
    expected["token"] = json!(SUBMITTED_SENTINEL);
    let expected_bytes = format!("{}\n", serde_json::to_string_pretty(&expected).expect("expected JSON"));
    assert_file_bytes(&path, expected_bytes.as_bytes());
    let reopened = SandConnectorSecretStore::new(&root);
    assert!(
        Value::Object(reopened.read("agent-a", "slack")) == expected,
        "replacement must preserve unrelated nested and non-string values"
    );
    assert!(
        reopened.get_secret("agent-a", "slack", "token").as_deref() == Some(SUBMITTED_SENTINEL),
        "a reopened owner must read the committed value"
    );
    assert!(reopened.get_secret("agent-a", "slack", "nested").is_none());
    assert!(reopened.set_secret("agent-a", "slack", " token ", " value ").expect("preserve input spelling"));
    assert!(reopened.get_secret("agent-a", "slack", " token ").as_deref() == Some(" value "));

    let empty_path = store.file_path("agent-a", "discord");
    fs::write(&empty_path, b"{}").expect("empty object fixture");
    assert!(store.set_secret("agent-a", "discord", "token", SUBMITTED_SENTINEL).expect("write into empty object"));
    assert!(store.get_secret("agent-a", "discord", "token").as_deref() == Some(SUBMITTED_SENTINEL));

    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn connector_secret_write_admission_rejects_read_denied_regular_file_before_replacement() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_root("read-denied");
    let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let created = workers.materialize_new_session(None, "user", None).expect("materialize");
    assert!(workers.store_connector_credential(&created.id, "slack", "token", STORED_SENTINEL).expect("seed credential"));
    let channels = workers.open_channel_store(&created.id).expect("channel owner");
    assert!(channels.write_metadata("slack", "preserve existing label").expect("seed metadata"));
    let store = workers.connector_secret_store();
    let path = store.file_path(&created.id, "slack");
    let raw = fs::read(&path).expect("original credential bytes");
    let metadata_path = channels.config_path("slack");
    let metadata = fs::read(&metadata_path).expect("original metadata bytes");
    let original = fs::metadata(&path).expect("regular file metadata");
    assert!(original.is_file(), "read denial must target a regular fixture file");
    let permissions = RestorePermissions { path: path.clone(), original: original.permissions() };
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("deny fixture read");
    assert!(
        matches!(fs::read(&path), Err(error) if error.kind() == io::ErrorKind::PermissionDenied),
        "fixture must establish actual PermissionDenied; privileged execution is not a passing oracle"
    );
    let parent = path.parent().expect("secret parent");
    let probe = parent.join("ccwa-parent-write-probe");
    fs::write(&probe, b"writable parent").expect("parent must permit replacement writes");
    fs::remove_file(&probe).expect("remove owned parent probe");
    let before_entries = directory_entries(parent);

    // The old permissive reader could write and rename over this unreadable
    // regular file because its parent is writable. A directory fixture cannot
    // establish that destructive success path.
    let store_result = store.set_secret(&created.id, "slack", "token", SUBMITTED_SENTINEL);
    let session_result = workers.store_connector_credential(&created.id, "slack", "token", SUBMITTED_SENTINEL);
    permissions.restore().expect("restore fixture read before byte comparison");
    let error = store_result.expect_err("direct store must reject unavailable existing data");
    assert_read_admission_error(&error, io::ErrorKind::PermissionDenied);
    let session_error = session_result.expect_err("Session must propagate the same refusal");
    assert_secret_free(&session_error);
    assert_file_bytes(&path, &raw);
    assert_file_bytes(&metadata_path, &metadata);
    assert!(directory_entries(parent) == before_entries, "refusal must leave no write artifacts");

    assert!(workers.store_connector_credential(&created.id, "slack", "token", SUBMITTED_SENTINEL).expect("readable-after-repair write"));
    let reopened = SandConnectorSecretStore::new(store.secrets_root());
    assert!(reopened.get_secret(&created.id, "slack", "token").as_deref() == Some(SUBMITTED_SENTINEL));
    drop(permissions);
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

fn assert_session_write_refused(
    workers: &ProductionSessionWorkers,
    agent_id: &str,
    platform: &str,
    secret_bytes: &[u8],
    metadata_bytes: Option<&[u8]>,
) {
    let path = workers.connector_secret_store().file_path(agent_id, platform);
    let parent = path.parent().expect("secret parent");
    let before_entries = directory_entries(parent);
    let error = workers.store_connector_credential(agent_id, platform, "token", SUBMITTED_SENTINEL)
        .expect_err("Session must reject unreadable credentials before metadata");
    assert!(error.starts_with("connector secret filesystem error:"), "refusal must originate from the real secret writer");
    assert_secret_free(&error);
    assert_file_bytes(&path, secret_bytes);
    assert!(directory_entries(parent) == before_entries, "Session refusal must leave no secret write artifacts");
    let metadata_path = workers.open_channel_store(agent_id).expect("channel owner").config_path(platform);
    if let Some(expected) = metadata_bytes {
        assert_file_bytes(&metadata_path, expected);
    } else {
        assert!(
            matches!(fs::symlink_metadata(&metadata_path), Err(error) if error.kind() == io::ErrorKind::NotFound),
            "refusal must not create previously absent channel metadata"
        );
        assert!(!metadata_path.parent().expect("platform directory").exists());
    }
    assert!(workers.get_connector_secret(agent_id, platform, "token").expect("compatible inspection").is_none());
}

#[test]
fn production_session_connector_refusal_preserves_metadata_and_reopens() {
    let root = temp_root("session-admission");
    let agents_root = root.join("agents");
    let workers = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
    let created = workers.materialize_new_session(None, "user", None).expect("materialize affected Agent");
    let other = workers.materialize_new_session(None, "user", None).expect("materialize unrelated Agent");
    for (agent_id, platform) in [(&created.id, "slack"), (&created.id, "discord"), (&other.id, "slack")] {
        assert!(workers.store_connector_credential(agent_id, platform, "token", STORED_SENTINEL).expect("seed credential through Session"));
    }
    let channels = workers.open_channel_store(&created.id).expect("channel owner");
    assert!(channels.write_metadata("slack", "keep this label").expect("seed existing metadata"));
    let metadata = fs::read(channels.config_path("slack")).expect("metadata baseline");
    assert!(!channels.config_path("matrix").parent().expect("platform directory").exists());

    let store = workers.connector_secret_store();
    let preserved_paths = [
        store.file_path(&created.id, "discord"),
        channels.config_path("discord"),
        store.file_path(&other.id, "slack"),
        workers.open_channel_store(&other.id).expect("other channel owner").config_path("slack"),
    ];
    let preserved = preserved_paths.iter().map(|path| fs::read(path).expect("unrelated baseline")).collect::<Vec<_>>();
    let corrupt = format!("{{\"token\":\"{STORED_SENTINEL}\",").into_bytes();
    for platform in ["slack", "matrix"] {
        fs::write(store.file_path(&created.id, platform), &corrupt).expect("seed unreadable credential image");
    }
    assert_session_write_refused(&workers, &created.id, "slack", &corrupt, Some(&metadata));
    assert_session_write_refused(&workers, &created.id, "matrix", &corrupt, None);
    workers.shutdown();
    drop(workers);

    let reopened = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
    assert_session_write_refused(&reopened, &created.id, "slack", &corrupt, Some(&metadata));
    assert_session_write_refused(&reopened, &created.id, "matrix", &corrupt, None);
    for (path, expected) in preserved_paths.iter().zip(&preserved) {
        assert_file_bytes(path, expected);
    }
    for platform in ["slack", "matrix"] {
        assert!(reopened.remove_connector_platform_secret(&created.id, platform).expect("explicitly remove unreadable image"));
        assert!(reopened.store_connector_credential(&created.id, platform, "token", SUBMITTED_SENTINEL).expect("legitimate write after explicit removal"));
        assert!(reopened.get_connector_secret(&created.id, platform, "token").expect("repaired credential").as_deref() == Some(SUBMITTED_SENTINEL));
        assert!(reopened.open_channel_store(&created.id).expect("channel owner").config_path(platform).is_file());
    }
    reopened.shutdown();
    drop(reopened);

    let final_owner = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
    for platform in ["slack", "matrix"] {
        assert!(final_owner.get_connector_secret(&created.id, platform, "token").expect("durable repaired credential").as_deref() == Some(SUBMITTED_SENTINEL));
    }
    for (path, expected) in preserved_paths.iter().zip(&preserved) {
        assert_file_bytes(path, expected);
    }
    final_owner.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_session_exposes_connector_credential_ownership_at_canonical_root() {
    let agents_root = temp_root("production").join("agents");
    let workers = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
    let created = workers
        .materialize_new_session(None, "user", None)
        .expect("materialize");

    assert!(workers
        .store_connector_credential(&created.id, "discord", "token", "abc")
        .expect("store credential"));
    assert_eq!(
        workers
            .get_connector_secret(&created.id, "discord", "token")
            .expect("get secret")
            .as_deref(),
        Some("abc")
    );
    assert!(workers
        .remove_connector_platform_secret(&created.id, "discord")
        .expect("remove secret"));
    assert_eq!(
        workers
            .get_connector_secret(&created.id, "discord", "token")
            .expect("get removed"),
        None
    );

    workers.shutdown();
    let _ = fs::remove_dir_all(
        agents_root
            .parent()
            .expect("agents parent"),
    );
}
