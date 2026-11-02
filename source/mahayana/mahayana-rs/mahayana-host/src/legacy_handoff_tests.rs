//! NMR-36: exercise the public Host constructors before any legacy handoff can
//! become a provider, read credentials or touch account state. All fixtures are
//! isolated public data; execution is restricted to the exact-head Actions gate.

use super::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

const REFUSAL: &str =
    "legacy Telegram handoff is unavailable in the Fabushi host; use the native Session/Transcript owner";

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct OwnedDirectory(PathBuf);

impl OwnedDirectory {
    fn absent() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "fabushi-host-handoff-{}-{stamp}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed),
        )))
    }

    fn config(&self) -> HostCreateConfig {
        HostCreateConfig {
            runtime: RuntimeConfig {
                data_dir: Some(self.0.join("runtime")),
                workspace_roots: vec![self.0.join("workspace")],
                ..RuntimeConfig::default()
            },
            product_session_path: Some(self.0.join("account-session.json")),
            product_surface_state_path: Some(self.0.join("product-surface.json")),
            automation_path: Some(self.0.join("automations.json")),
            cwd: Some(self.0.join("workspace")),
            inherit_installed_plugins: Some(false),
            ..HostCreateConfig::default()
        }
    }
}

impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        if self.0.exists() {
            fs::remove_dir_all(&self.0).expect("remove only this test's directory");
        }
    }
}

fn assert_refused(result: Result<MahayanaHost, HostError>) {
    let error = result.err().expect("legacy handoff must fail");
    assert_eq!(error.to_string(), REFUSAL);
}

fn retained_snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(
        root: &Path,
        path: &Path,
        output: &mut BTreeMap<PathBuf, Option<Vec<u8>>>,
    ) {
        for entry in fs::read_dir(path).expect("read retained directory") {
            let entry = entry.expect("retained entry");
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("owned path").to_path_buf();
            let kind = entry.file_type().expect("retained file type");
            if kind.is_dir() {
                output.insert(relative, None);
                visit(root, &path, output);
            } else {
                assert!(kind.is_file(), "fixture contains only files and directories");
                output.insert(relative, Some(fs::read(path).expect("read retained bytes")));
            }
        }
    }

    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

#[test]
fn legacy_json_handoff_value_matrix_is_rejected() {
    let root = OwnedDirectory::absent();
    let cases = [
        json!({"telegramClientId": 0, "telegramSelfUserId": null}),
        json!({"telegramClientId": u64::MAX, "telegramSelfUserId": null}),
        json!({"telegramClientId": null, "telegramSelfUserId": 0}),
        json!({"telegramClientId": null, "telegramSelfUserId": i64::MIN}),
        json!({"telegramClientId": null, "telegramSelfUserId": i64::MAX}),
        json!({"telegramClientId": 17, "telegramSelfUserId": 23}),
    ];
    for handoff in cases {
        let mut payload = serde_json::to_value(root.config()).expect("serialize host config");
        for key in ["telegramClientId", "telegramSelfUserId"] {
            payload[key] = handoff[key].clone();
        }
        let decoded: HostCreateConfig =
            serde_json::from_value(payload).expect("decode legacy camelCase fields");
        let encoded = serde_json::to_value(&decoded).expect("encode preserved fields");
        assert_eq!(encoded["telegramClientId"], handoff["telegramClientId"]);
        assert_eq!(encoded["telegramSelfUserId"], handoff["telegramSelfUserId"]);
        assert_refused(MahayanaHost::create(decoded));
    }
}

#[test]
fn absent_storage_roots_remain_absent_on_legacy_handoff() {
    for (client_id, self_user_id) in [(Some(0), None), (None, Some(0)), (Some(17), Some(23))] {
        let root = OwnedDirectory::absent();
        let mut config = root.config();
        config.telegram_client_id = client_id;
        config.telegram_self_user_id = self_user_id;
        // A later runtime constraint must not replace the admission error.
        config.runtime.remote_agent_enabled = true;
        assert!(!root.0.exists());
        assert_refused(MahayanaHost::create(config));
        assert!(!root.0.exists(), "refusal must not create any supplied root");
    }
}

#[test]
fn rejected_handoffs_preserve_existing_opaque_and_corrupt_files() {
    let root = OwnedDirectory::absent();
    let fixtures: &[(&str, &[u8])] = &[
        ("account-session.json", b"public corrupt account record\0\xff"),
        ("product-surface.json", b"public opaque encrypted product state\0"),
        ("automations.json", b"public retained automation archive\0\xfe"),
        ("runtime/runtime.sqlite3", b"public corrupt runtime SQLite fixture\0"),
        ("app-host/feature-host/telegram/retained.sqlite3", b"public retained encrypted SQLite\0\xff"),
        ("app-host/feature-host/telegram/retained.sqlite3-wal", b"public retained WAL"),
        ("app-host/feature-host/telegram/retained.sqlite3-shm", b"public retained SHM"),
        ("secure-store/telegram-key", b"public retained key marker"),
        ("unknown/nested/record.bin", b"public unknown retained bytes\0\xfe"),
    ];
    for (path, bytes) in fixtures {
        let destination = root.0.join(path);
        fs::create_dir_all(destination.parent().expect("fixture parent")).expect("fixture dirs");
        fs::write(destination, bytes).expect("write public retained fixture");
    }
    let before = retained_snapshot(&root.0);
    for (client_id, self_user_id) in [(Some(17), None), (None, Some(23)), (Some(17), Some(23))] {
        let mut config = root.config();
        config.telegram_client_id = client_id;
        config.telegram_self_user_id = self_user_id;
        assert_refused(MahayanaHost::create(config));
        assert_eq!(retained_snapshot(&root.0), before, "no data changes after refusal");
    }
}

#[test]
fn native_configuration_keeps_status_ready_and_durable_state() {
    let root = OwnedDirectory::absent();
    fs::create_dir_all(root.0.join("workspace")).expect("create native workspace");
    let mut payload = serde_json::to_value(root.config()).expect("serialize native config");
    payload["telegramClientId"] = Value::Null;
    payload["telegramSelfUserId"] = Value::Null;
    let config: HostCreateConfig =
        serde_json::from_value(payload).expect("explicit null remains native");
    let host = MahayanaHost::create(config).expect("create native Host");
    let status = host.execute(RuntimeCommand::Status).expect("native status");
    let encoded = serde_json::to_value(status).expect("encode status");
    assert_eq!(encoded["runtimeAbiVersion"], 1);
    assert_eq!(encoded["remoteAgentEnabled"], false);
    assert!(matches!(
        host.receive(Duration::from_secs(1)).expect("receive ready"),
        Some(RuntimeEvent::Ready { .. }),
    ));
    let marker = json!({"value": "public native state survives restart"});
    host.execute(RuntimeCommand::WorkspaceStateSet {
        key: "nmr36-native-control".into(),
        value: marker.clone(),
    })
    .expect("persist existing native workspace owner");
    drop(host);

    let restored = MahayanaHost::create(root.config()).expect("restore native Host");
    match restored
        .execute(RuntimeCommand::WorkspaceStateGet {
            key: "nmr36-native-control".into(),
        })
        .expect("read restored native state")
    {
        RuntimeResponse::RuntimeWorkspaceState { value, .. } => assert_eq!(value, Some(marker)),
        other => panic!("unexpected native state response: {other:?}"),
    }
}

#[cfg(feature = "test-support")]
struct FixtureBackend {
    reject_use: bool,
    calls: std::sync::atomic::AtomicUsize,
}

#[cfg(feature = "test-support")]
impl FixtureBackend {
    fn new(reject_use: bool) -> Self {
        Self {
            reject_use,
            calls: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn used(&self) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(!self.reject_use, "rejected creation must not consult its backend");
    }
}

#[cfg(feature = "test-support")]
#[async_trait::async_trait]
impl EngineBackend for FixtureBackend {
    fn descriptor(&self) -> mahayana_kernel::BackendDescriptor {
        self.used();
        mahayana_kernel::BackendDescriptor {
            id: "nmr36-native-fixture".into(),
            display_name: "Public native handoff fixture".into(),
            native: true,
            capabilities: mahayana_kernel::CapabilitySet::new([
                mahayana_kernel::Capability::Model,
            ]),
        }
    }

    async fn open_session(
        &self,
        _request: mahayana_kernel::OpenSessionRequest,
    ) -> Result<mahayana_kernel::SessionId, mahayana_kernel::KernelError> {
        self.used();
        Ok(mahayana_kernel::SessionId::new())
    }

    async fn run(
        &self,
        request: mahayana_kernel::RunRequest,
        events: mahayana_kernel::SharedKernelEventSink,
    ) -> Result<(), mahayana_kernel::KernelError> {
        self.used();
        events.emit(mahayana_kernel::KernelEvent::MessageCompleted {
            operation_id: request.operation_id,
            text: "Public native Agent completion".into(),
        })
    }

    async fn interrupt(
        &self,
        _operation_id: &mahayana_kernel::OperationId,
    ) -> Result<(), mahayana_kernel::KernelError> {
        self.used();
        Ok(())
    }

    async fn resolve_approval(
        &self,
        _resolution: mahayana_kernel::ApprovalResolution,
    ) -> Result<(), mahayana_kernel::KernelError> {
        self.used();
        Ok(())
    }
}

#[cfg(feature = "test-support")]
#[test]
fn test_support_constructor_rejects_handoff_before_backend_use() {
    for (client_id, self_user_id) in [(Some(0), None), (None, Some(0)), (Some(17), Some(23))] {
        let root = OwnedDirectory::absent();
        let mut config = root.config();
        config.telegram_client_id = client_id;
        config.telegram_self_user_id = self_user_id;
        let backend = Arc::new(FixtureBackend::new(true));
        assert_refused(MahayanaHost::create_with_engine_backend_for_test(
            config,
            backend.clone(),
        ));
        assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
        assert!(!root.0.exists(), "test-support admission also precedes storage");
    }
}

#[cfg(feature = "test-support")]
#[test]
fn native_agent_lifecycle_survives_rejected_legacy_handoff() {
    let root = OwnedDirectory::absent();
    fs::create_dir_all(root.0.join("workspace")).expect("create Agent workspace");
    let backend = Arc::new(FixtureBackend::new(false));
    let host = MahayanaHost::create_with_engine_backend_for_test(root.config(), backend.clone())
        .expect("create native Agent host");
    assert!(matches!(
        host.receive(Duration::from_secs(1)).expect("native ready"),
        Some(RuntimeEvent::Ready { .. }),
    ));
    let conversation_id = match host
        .execute(RuntimeCommand::ListConversations)
        .expect("list native Agent conversations")
    {
        RuntimeResponse::Conversations { data } => data
            .into_iter()
            .find(|conversation| conversation.peer == mahayana_core::PeerKind::MahayanaAi)
            .expect("native Agent conversation")
            .id,
        other => panic!("unexpected conversation response: {other:?}"),
    };

    let refused_root = OwnedDirectory::absent();
    let mut refused_config = refused_root.config();
    refused_config.telegram_self_user_id = Some(23);
    assert_refused(MahayanaHost::create(refused_config));
    assert!(!refused_root.0.exists());
    host.execute(RuntimeCommand::Status).expect("existing native host remains usable");

    let operation_id = match host
        .execute(RuntimeCommand::SendMessage {
            conversation_id: conversation_id.clone(),
            text: "Public native Agent prompt".into(),
            client_message_id: Some("nmr36-native-agent-message".into()),
            retry_of_client_message_id: None,
            inference_provider: None,
            hidden: false,
        })
        .expect("send after rejected legacy handoff")
    {
        RuntimeResponse::Accepted { operation_id } => operation_id,
        other => panic!("unexpected send response: {other:?}"),
    };
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut saw_message = false;
    let mut saw_completion = false;
    while std::time::Instant::now() < deadline && !saw_completion {
        match host.receive(Duration::from_millis(100)).expect("native Agent event") {
            Some(RuntimeEvent::MessageCompleted { operation_id: event_id, message, .. }) => {
                assert_eq!(event_id, operation_id);
                assert_eq!(message.text, "Public native Agent completion");
                saw_message = true;
            }
            Some(RuntimeEvent::OperationCompleted { operation_id: event_id }) => {
                assert_eq!(event_id, operation_id);
                saw_completion = true;
            }
            Some(RuntimeEvent::OperationFailed { .. }) => panic!("native Agent operation failed"),
            _ => {}
        }
    }
    assert!(saw_message && saw_completion, "native Agent must complete on the existing host");
    assert!(backend.calls.load(Ordering::SeqCst) > 0);
    match host
        .execute(RuntimeCommand::ConversationHistory {
            conversation_id,
            limit: None,
        })
        .expect("read completed native Agent history")
    {
        RuntimeResponse::History { data } => {
            assert!(data.iter().any(|message| message.text == "Public native Agent prompt"));
            assert!(data.iter().any(|message| message.text == "Public native Agent completion"));
        }
        other => panic!("unexpected history response: {other:?}"),
    }
}

