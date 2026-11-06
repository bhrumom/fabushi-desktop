//! NMR-34/35, NMR-SOCIAL-OWNER-02 / NMR-SOCIAL-AC-01 and
//! NMR-RETAINED-TELEGRAM-01 tests. Production fixtures have no MahayanaHost
//! runtime: they verify refusal before that runtime is touched.
//! Real Production AppHost construction, account handoff and process restart are
//! covered by the native_desktop_account_handoff integration target.

use super::*;
use fabushi_messaging_core::{
    Actor, BlobMetadata, ClientCommand, ClientEnvelope, MessagingEngine,
    MessagingSnapshot, MessagingStateStore, RequestContext,
};
use std::fs;

const DESKTOP_REFUSAL: &str =
    "standalone messaging is unavailable in the desktop host; use the Session/Transcript owner";
const DESKTOP_SOCIAL_REFUSAL: &str =
    "social conversations are unavailable in the desktop host; use the Session/Transcript owner";
const DESKTOP_TELEGRAM_REFUSAL: &str = "provider is unavailable: telegram";
const TELEGRAM_PUBLIC_INTENTS: [&str; 5] = [
    "telegram:chat:42",
    "telegram:chat:",
    " \ttelegram:chat:42\n",
    " \ttelegram:chat:\n",
    "telegram:chat:historical-unknown",
];

struct OwnedRoot(PathBuf);

impl OwnedRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "fabushi-desktop-messaging-boundary-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        assert!(!path.exists(), "fixture root must start absent");
        Self(path)
    }

    fn agents(&self) -> PathBuf {
        self.0.join("runtime/agents")
    }
}

impl Drop for OwnedRoot {
    fn drop(&mut self) {
        // This UUID root is exclusively owned by this test, never app data.
        if self.0.exists() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

fn owner(root: &OwnedRoot, mode: HostMode, platform: SurfacePlatform) -> FeatureHostController {
    let mut controller = FeatureHostController::create(
        HostConfig {
            profile_id: format!("messaging-boundary-{}", Uuid::new_v4()),
            mode: HostMode::Test,
        },
        platform,
    )
    .expect("create isolated owner fixture");
    controller.memory_root_path = Some(root.agents());
    controller.workflow_root_path = Some(root.0.join("runtime/workflows"));
    controller.config.mode = mode;
    controller
}

fn actor_for(account_id: &str) -> ActorId {
    let fingerprint = Sha256::digest(account_id.as_bytes())[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    ActorId::new(format!("human:account:{fingerprint}"))
}

fn retained_root(root: &OwnedRoot, account_id: &str) -> PathBuf {
    let fingerprint = Sha256::digest(account_id.as_bytes())[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    root.0
        .join("runtime/accounts")
        .join(fingerprint)
        .join("agents/_messaging")
}

fn profile_envelope(account_id: &str, name: &str) -> Value {
    let actor_id = actor_for(account_id);
    serde_json::to_value(ClientEnvelope::new(
        RequestContext {
            request_id: "profile-request".into(),
            device_id: "fixture-device".into(),
            actor_id: actor_id.clone(),
            session_id: "fixture-session".into(),
            sent_at_ms: 1,
        },
        ClientCommand::UpsertProfile {
            actor: Actor::human(actor_id.0, name),
        },
    ))
    .expect("serialize a real protocol envelope")
}

fn assert_refused<T>(result: Result<T, FeatureHostError>) {
    match result {
        Err(FeatureHostError::Contract(message)) => assert_eq!(message, DESKTOP_REFUSAL),
        Err(error) => panic!("expected the desktop contract refusal, got {error}"),
        Ok(_) => panic!("standalone messaging was accepted by the desktop owner"),
    }
}

fn event_image(controller: &FeatureHostController) -> Vec<Value> {
    controller
        .state()
        .expect("read fixture events")
        .events
        .iter()
        .map(|event| serde_json::to_value(event).expect("serialize fixture event"))
        .collect()
}

fn assert_no_new_events(controller: &FeatureHostController, before: &[Value]) {
    assert_eq!(event_image(controller).as_slice(), before);
    assert!(controller
        .state()
        .expect("inspect event variants")
        .events
        .iter()
        .all(|event| !matches!(event, HostEvent::MessagingEvent { .. })));
}

fn refuse_all_entries(controller: &FeatureHostController, account_id: &str) {
    let events = event_image(controller);
    for envelope in [profile_envelope(account_id, "must not persist"), json!(null)] {
        assert_refused(controller.execute_messaging_sync("direct".into(), envelope.clone()));
        assert_no_new_events(controller, &events);
        assert_refused(controller.execute(FeatureCommand::MessagingExecute {
            request_id: "typed".into(),
            envelope,
        }));
        assert_no_new_events(controller, &events);
    }
    for blob_id in ["retained", "../invalid"] {
        assert_refused(controller.read_messaging_blob_range(blob_id, 0, 8));
        assert_no_new_events(controller, &events);
    }
    for (device, session, scopes) in [
        ("fixture-device", "fixture-session", vec!["messaging".into()]),
        ("", "", vec!["unknown-scope".into()]),
    ] {
        assert_refused(controller.issue_messaging_access(
            device.into(),
            session.into(),
            scopes,
            60_000,
        ));
        assert_no_new_events(controller, &events);
    }
}

fn tree_image(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(root: &Path, current: &Path, image: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in fs::read_dir(current).expect("read owned fixture directory") {
            let entry = entry.expect("read fixture entry");
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("fixture-relative path").to_path_buf();
            let kind = entry.file_type().expect("fixture entry type");
            if kind.is_dir() {
                image.insert(relative, None);
                visit(root, &path, image);
            } else {
                assert!(kind.is_file(), "fixture contains only regular files and directories");
                image.insert(relative, Some(fs::read(path).expect("read fixture bytes")));
            }
        }
    }
    let mut image = BTreeMap::new();
    if root.exists() {
        visit(root, root, &mut image);
    }
    image
}

fn write_fixture(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture parent");
    fs::write(path, bytes).expect("write fixture bytes");
}

fn social_commands(conversation_id: &str) -> [FeatureCommand; 2] {
    [
        FeatureCommand::ConversationOpen {
            request_id: "social-open".into(),
            conversation_id: conversation_id.into(),
        },
        FeatureCommand::ChatSend {
            request_id: "social-send".into(),
            text: "Public retained Human text must never enter Agent memory or workflow context.".into(),
            agent_id: Some("mahayana-assistant".into()),
            conversation_id: Some(conversation_id.into()),
            client_message_id: Some("public-social-client-id".into()),
            retry_of_message_id: None,
            mode: AgentMode::Agent,
            mode_statement: Some("Do not reinterpret this Human intent as Agent input".into()),
            model: None,
            attachments: Vec::new(),
        },
    ]
}

fn telegram_commands(conversation_id: &str) -> [FeatureCommand; 3] {
    let send = |agent_id: &str| FeatureCommand::ChatSend {
        request_id: format!("telegram-send-{agent_id}"),
        text: "Retained Telegram text must not reach plugins or Agent context.".into(),
        agent_id: Some(agent_id.into()),
        conversation_id: Some(conversation_id.into()),
        client_message_id: Some(format!("public-telegram-client-{agent_id}")),
        retry_of_message_id: None,
        mode: AgentMode::Agent,
        mode_statement: Some("Keep the original retired-provider intent".into()),
        model: None,
        attachments: Vec::new(),
    };
    [
        FeatureCommand::ConversationOpen {
            request_id: "telegram-open".into(),
            conversation_id: conversation_id.into(),
        },
        send("mahayana-assistant"),
        send("retained-plugin-fixture"),
    ]
}

fn assert_telegram_refused(result: Result<CommandAccepted, FeatureHostError>) {
    match result {
        Err(FeatureHostError::Contract(message)) => assert_eq!(message, DESKTOP_TELEGRAM_REFUSAL),
        Err(error) => panic!("expected Telegram refusal before auth, state or plugin dispatch, got {error}"),
        Ok(_) => panic!("desktop accepted a retired Telegram conversation"),
    }
}

fn assert_social_refused(result: Result<CommandAccepted, FeatureHostError>) {
    match result {
        Err(FeatureHostError::Contract(message)) => assert_eq!(message, DESKTOP_SOCIAL_REFUSAL),
        Err(error) => panic!("expected Session/Transcript ownership before other work, got {error}"),
        Ok(_) => panic!("desktop accepted a generic social conversation"),
    }
}

#[test]
fn desktop_social_intents_refuse_before_auth_runtime_context_io_and_events() {
    let root = OwnedRoot::new();
    let controller = owner(&root, HostMode::Production, SurfacePlatform::Electron);
    let before = event_image(&controller);
    #[cfg(feature = "production")]
    assert!(controller.runtime.is_none(), "refusal must not require an initialized Runtime");
    for conversation_id in [
        "mahayana:contact:42",
        "mahayana:contact:",
        " \tmahayana:contact:42\n",
    ] {
        for command in social_commands(conversation_id) {
            assert_social_refused(controller.execute(command));
            assert_eq!(event_image(&controller), before);
            assert!(!root.0.exists(), "denial must precede any Agent workspace creation");
        }
    }
    for conversation_id in TELEGRAM_PUBLIC_INTENTS {
        for command in telegram_commands(conversation_id) {
            assert_telegram_refused(controller.execute(command));
            assert_eq!(event_image(&controller), before);
            assert!(!root.0.exists(), "Telegram refusal must precede plugin or Agent workspace creation");
        }
    }
}

#[test]
fn desktop_social_refusal_does_not_lock_state_or_touch_retained_context() {
    let root = OwnedRoot::new();
    write_fixture(&root.agents().join("mahayana-assistant/memory/retained.bin"), b"original public memory\0\xff");
    write_fixture(&root.0.join("runtime/workflows/retained.bin"), b"original public workflow\0\xfe");
    let before_files = tree_image(&root.0);
    let controller = owner(&root, HostMode::Production, SurfacePlatform::Electron);
    let before_events = event_image(&controller);
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _state = controller.state.lock().expect("lock before deliberate poison");
        panic!("intentional social-owner fixture state poison");
    }));
    assert!(poisoned.is_err());
    assert!(controller.state.is_poisoned());
    for command in social_commands("mahayana:contact:42") {
        assert_social_refused(controller.execute(command));
    }
    for conversation_id in TELEGRAM_PUBLIC_INTENTS {
        for command in telegram_commands(conversation_id) {
            assert_telegram_refused(controller.execute(command));
        }
    }
    let state = controller.state.lock().unwrap_or_else(|error| error.into_inner());
    let after_events = state.events.iter()
        .map(|event| serde_json::to_value(event).expect("serialize retained event"))
        .collect::<Vec<_>>();
    assert_eq!(after_events, before_events);
    assert_eq!(tree_image(&root.0), before_files);
}

#[test]
fn social_boundary_keeps_explicit_test_non_desktop_and_other_namespaces() {
    let root = OwnedRoot::new();
    let controller = owner(&root, HostMode::Test, SurfacePlatform::Electron);
    for command in social_commands("mahayana:contact:42") {
        controller.execute(command).expect("explicit Test dispatch remains available");
    }
    for conversation_id in TELEGRAM_PUBLIC_INTENTS {
        for command in telegram_commands(conversation_id) {
            controller.execute(command).expect("explicit Test Telegram dispatch remains available");
        }
    }
    for platform in [SurfacePlatform::Ios, SurfacePlatform::Android, SurfacePlatform::Wasm] {
        let controller = owner(&root, HostMode::Production, platform);
        for command in social_commands("mahayana:contact:42") {
            assert!(matches!(
                controller.execute(command),
                Err(FeatureHostError::ProductionUnavailable)
            ), "non-desktop request must reach its existing absent-runtime boundary");
        }
        for conversation_id in TELEGRAM_PUBLIC_INTENTS {
            for command in telegram_commands(conversation_id) {
                assert!(matches!(
                    controller.execute(command),
                    Err(FeatureHostError::ProductionUnavailable)
                ), "non-desktop Telegram intent must retain the existing absent-runtime boundary");
            }
        }
    }
    let controller = owner(&root, HostMode::Production, SurfacePlatform::Electron);
    for conversation_id in [
        "mahayana-ai:default",
        "codex:agent:research",
        "miniapp:public-fixture",
        "mahayana:contacts:42",
        "unrecognized:42",
    ] {
        assert!(matches!(
            controller.execute(FeatureCommand::ConversationOpen {
                request_id: "non-social-control".into(),
                conversation_id: conversation_id.into(),
            }),
            Err(FeatureHostError::ProductionUnavailable)
        ), "unrelated namespace must retain the existing dispatch boundary");
    }
    for conversation_id in [
        "telegram:chats:42",
        "telegram:channel:42",
        "telegram:chat",
        "telegram",
        " \ttelegram:chats:42\n",
    ] {
        for command in telegram_commands(conversation_id) {
            assert!(matches!(
                controller.execute(command),
                Err(FeatureHostError::ProductionUnavailable)
            ), "nearby Telegram namespaces must retain their existing dispatch boundary");
        }
    }
}

fn seed_snapshot(root: &Path, account_id: &str) -> MessagingSnapshot {
    let mut state = MessagingEngine::new().state().clone();
    let actor = Actor::human(actor_for(account_id).0, format!("retained {account_id}"));
    state.actors.insert(actor.id.clone(), actor);
    let snapshot = MessagingSnapshot::new(state, 7, 11);
    JsonFileStateStore::new(root.join("snapshot.json"))
        .save(&snapshot)
        .expect("write snapshot using original store");
    snapshot
}

fn seed_blob(root: &Path, id: &str, bytes: &[u8], unfinished: bool) -> BlobMetadata {
    let store = FileBlobStore::new(root.join("blobs"));
    let metadata = BlobMetadata {
        id: BlobId::new(id).expect("fixture blob id"),
        file_name: format!("{id}.bin"),
        mime_type: "application/octet-stream".into(),
        size_bytes: bytes.len() as u64 + if unfinished { 8 } else { 0 },
        content_hash: None,
        created_at_ms: 11,
    };
    store.begin_upload(&metadata).expect("original blob upload writer");
    store
        .append_chunk(&metadata.id, 0, bytes)
        .expect("original blob chunk writer");
    if !unfinished {
        store.finish_upload(&metadata.id).expect("original blob commit writer");
    }
    metadata
}

#[test]
fn desktop_entries_refuse_before_auth_parse_io_and_event_publication() {
    let root = OwnedRoot::new();
    let controller = owner(&root, HostMode::Production, SurfacePlatform::Electron);
    refuse_all_entries(&controller, "account-a");
    assert!(!root.0.exists(), "denied requests must not create any storage");
}

#[test]
fn desktop_refusal_does_not_touch_poisoned_owner_state() {
    let root = OwnedRoot::new();
    let controller = owner(&root, HostMode::Production, SurfacePlatform::Electron);
    let before = event_image(&controller);
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _state = controller.state.lock().expect("lock before deliberate poison");
        panic!("intentional fixture state poison");
    }));
    assert!(poisoned.is_err());
    assert!(controller.state.is_poisoned());
    let assert_untouched = || {
        let state = controller.state.lock().unwrap_or_else(|error| error.into_inner());
        let events = state
            .events
            .iter()
            .map(|event| serde_json::to_value(event).expect("serialize poisoned fixture events"))
            .collect::<Vec<_>>();
        assert_eq!(events, before);
        assert!(state.events.iter().all(|event| !matches!(event, HostEvent::MessagingEvent { .. })));
    };
    assert_refused(controller.execute_messaging_sync("direct".into(), json!(null)));
    assert_untouched();
    assert_refused(controller.execute(FeatureCommand::MessagingExecute {
        request_id: "typed".into(),
        envelope: profile_envelope("account-a", "blocked"),
    }));
    assert_untouched();
    assert_refused(controller.read_messaging_blob_range("retained", 0, 8));
    assert_untouched();
    assert_refused(controller.issue_messaging_access(
        "device".into(),
        "session".into(),
        vec!["messaging".into()],
        60_000,
    ));
    assert_untouched();
    assert!(!root.0.exists());
}

#[test]
fn desktop_refusal_retains_original_stores_and_unfinished_uploads_across_reconstruction() {
    let root = OwnedRoot::new();
    let account_a = retained_root(&root, "account-a");
    let account_b = retained_root(&root, "account-b");
    let snapshot_a = seed_snapshot(&account_a, "account-a");
    let snapshot_b = seed_snapshot(&account_b, "account-b");
    let complete = seed_blob(&account_a, "retained", b"retained message attachment", false);
    let pending = seed_blob(&account_a, "pending", b"partial", true);
    seed_blob(&account_b, "other-account", b"other account bytes", false);
    write_fixture(&account_a.join("snapshot.json.tmp"), b"interrupted writer bytes");
    write_fixture(&account_b.join("future/unknown.bin"), b"unknown future format\0\xff");
    fs::create_dir_all(account_b.join("empty-directory")).expect("retain empty directory");

    let access = FileAccessTokenStore::new(root.agents().join("_messaging/access.json"));
    let issued = ["account-a", "account-b"].map(|account| {
        access
            .issue_random(AccessGrant {
                id: format!("grant:{account}"),
                actor_id: actor_for(account),
                device_id: "fixture-device".into(),
                session_id: "fixture-session".into(),
                scopes: BTreeSet::from([AccessScope::Messaging]),
                issued_at_ms: 1,
                expires_at_ms: None,
                revoked_at_ms: None,
            })
            .expect("write original access registry")
    });
    let before = tree_image(&root.0);
    for _reconstruction in 0..2 {
        for active in [Some("account-a"), Some("account-b"), None] {
            let controller = owner(&root, HostMode::Production, SurfacePlatform::Electron);
            *controller.active_account_id.lock().expect("fixture active account") =
                active.map(str::to_string);
            refuse_all_entries(&controller, active.unwrap_or("account-a"));
            assert_eq!(tree_image(&root.0), before);
        }
    }

    assert_eq!(JsonFileStateStore::new(account_a.join("snapshot.json")).load().unwrap(), Some(snapshot_a));
    assert_eq!(JsonFileStateStore::new(account_b.join("snapshot.json")).load().unwrap(), Some(snapshot_b));
    let blobs = FileBlobStore::new(account_a.join("blobs"));
    assert_eq!(blobs.metadata(&complete.id).unwrap(), complete);
    assert_eq!(blobs.read_range(&complete.id, 0, complete.size_bytes).unwrap(), b"retained message attachment");
    let pending_status = blobs.upload_status(&pending.id).expect("read original pending upload");
    assert!(!pending_status.complete);
    assert_eq!(pending_status.uploaded_bytes, 7);
    assert_eq!(pending_status.size_bytes, pending.size_bytes);
    for token in &issued {
        let authorized = access
            .authorize(token.token.as_bytes(), &token.grant.actor_id, "fixture-device", "fixture-session", AccessScope::Messaging, 100)
            .expect("original access reader still authorizes retained credential");
        assert_eq!(authorized.grant_id, token.grant.id);
    }
    assert!(access
        .authorize(issued[0].token.as_bytes(), &issued[1].grant.actor_id, "fixture-device", "fixture-session", AccessScope::Messaging, 100)
        .is_err());
    assert_eq!(tree_image(&root.0), before, "rollback readers must not rewrite retained data");
}

#[test]
fn desktop_refusal_leaves_corrupt_and_unknown_stores_byte_identical() {
    let root = OwnedRoot::new();
    let account = retained_root(&root, "account-a");
    let snapshot = account.join("snapshot.json");
    let access = root.agents().join("_messaging/access.json");
    write_fixture(&snapshot, b"{corrupt snapshot\0\xff");
    write_fixture(&access, b"[corrupt access registry");
    write_fixture(&account.join("blobs/retained.json"), b"bad blob metadata");
    write_fixture(&account.join("blobs/retained.blob"), b"untouched blob bytes");
    write_fixture(&account.join("future.schema"), b"version not yet understood");
    let before = tree_image(&root.0);
    let controller = owner(&root, HostMode::Production, SurfacePlatform::Electron);
    *controller.active_account_id.lock().unwrap() = Some("account-a".into());
    refuse_all_entries(&controller, "account-a");
    assert_eq!(tree_image(&root.0), before);
    assert!(JsonFileStateStore::new(snapshot).load().is_err());
    assert!(FileAccessTokenStore::new(access).load().is_err());
    assert!(FileBlobStore::new(account.join("blobs")).metadata(&BlobId::new("retained").unwrap()).is_err());
    assert_eq!(tree_image(&root.0), before, "corrupt data must stay available for recovery");
}

#[test]
fn explicit_test_mode_still_executes_persists_issues_access_and_reads_blobs() {
    let root = OwnedRoot::new();
    let controller = owner(&root, HostMode::Test, SurfacePlatform::Electron);
    controller
        .password_login("boundary@example.invalid".into(), "fixture-only".into())
        .expect("explicit test account login");
    let first = controller
        .execute_messaging_sync("direct-profile".into(), profile_envelope("fast-e2e-user", "first profile"))
        .expect("Test mode retains actual MessagingService execution");
    assert!(!first.is_empty());
    let accepted = controller
        .execute(FeatureCommand::MessagingExecute {
            request_id: "typed-profile".into(),
            envelope: profile_envelope("fast-e2e-user", "updated profile"),
        })
        .expect("Test mode retains typed command route");
    assert_eq!(accepted.request_id, "typed-profile");
    #[cfg(feature = "production")]
    let snapshot_root = retained_root(&root, "fast-e2e-user");
    #[cfg(not(feature = "production"))]
    let snapshot_root = root.agents().join("_messaging");
    let persisted = JsonFileStateStore::new(snapshot_root.join("snapshot.json"))
        .load().expect("read actual service snapshot").expect("snapshot exists");
    assert_eq!(persisted.state.actors[&actor_for("fast-e2e-user")].display_name, "updated profile");
    assert!(persisted.cursor >= 2);
    let state = controller.state().unwrap();
    for expected_request in ["direct-profile", "typed-profile"] {
        assert!(state.events.iter().any(|event| matches!(
            event,
            HostEvent::MessagingEvent { request_id, .. } if request_id == expected_request
        )));
    }
    drop(state);

    let token = controller
        .issue_messaging_access("fixture-device".into(), "fixture-session".into(), vec!["messaging".into()], 600_000)
        .expect("Test mode retains access issuance");
    FileAccessTokenStore::new(root.agents().join("_messaging/access.json"))
        .authorize(token["accessToken"].as_str().unwrap().as_bytes(), &actor_for("fast-e2e-user"), "fixture-device", "fixture-session", AccessScope::Messaging, now_millis())
        .expect("issued Test credential is accepted by the original reader");
    // Test-mode blob access deliberately retains its existing unscoped root.
    let metadata = seed_blob(&root.agents().join("_messaging"), "test-blob", b"test bytes", false);
    let (actual, bytes) = controller.read_messaging_blob_range("test-blob", 1, 4).expect("Test blob read");
    assert_eq!(actual, metadata);
    assert_eq!(bytes, b"est ");
}

#[test]
fn non_desktop_production_retains_blob_reads_and_existing_validation_boundaries() {
    for platform in [SurfacePlatform::Ios, SurfacePlatform::Android, SurfacePlatform::Wasm] {
        let root = OwnedRoot::new();
        let controller = owner(&root, HostMode::Production, platform);
        *controller.active_account_id.lock().unwrap() = Some("account-a".into());
        #[cfg(feature = "production")]
        let blob_root = retained_root(&root, "account-a");
        #[cfg(not(feature = "production"))]
        let blob_root = root.agents().join("_messaging");
        let metadata = seed_blob(&blob_root, "mobile-blob", b"mobile retained bytes", false);
        let before = tree_image(&root.0);
        let (actual, bytes) = controller.read_messaging_blob_range("mobile-blob", 0, 6).expect("non-desktop production blob read");
        assert_eq!(actual, metadata);
        assert_eq!(bytes, b"mobile");
        let invalid = controller.execute_messaging_sync("invalid".into(), json!(null)).unwrap_err();
        assert!(invalid.to_string().starts_with("invalid messaging envelope:"));
        // This owner fixture has no runtime. Reaching ProductionUnavailable is
        // the original authentication boundary, not mobile service acceptance.
        assert!(matches!(
            controller.issue_messaging_access("device".into(), "session".into(), vec!["messaging".into()], 600_000),
            Err(FeatureHostError::ProductionUnavailable)
        ));
        assert_eq!(tree_image(&root.0), before);
    }
}

#[test]
fn automation_store_preserves_reserved_messaging_tree_and_normal_agent_lifecycle() {
    let root = OwnedRoot::new();
    let reserved = retained_root(&root, "account-a");
    let agents = reserved.parent().expect("account Agent root").to_path_buf();
    let legacy_path = agents
        .parent()
        .expect("account root")
        .join("automations.json");
    let make = |agent: &str, id: &str| AutomationSummary {
        id: id.into(),
        agent_id: Some(agent.into()),
        name: "daily brief".into(),
        prompt: "Summarize the day.".into(),
        schedule: "@daily".into(),
        trigger: Some(AutomationTrigger::Schedule { schedule: "@daily".into() }),
        enabled: true,
        created_at_ms: 1,
        last_run_at_ms: None,
        next_run_at_ms: None,
    };
    write_fixture(
        &reserved.join("automations/valid/automation.json"),
        &serde_json::to_vec(&fabu_automation_payload(&make("_messaging", "valid"))).unwrap(),
    );
    write_fixture(&reserved.join("automations/corrupt/automation.json"), b"{corrupt\0");
    write_fixture(&reserved.join("automations/unknown/automation.json"), b"{\"futureSchema\":42}");
    write_fixture(&reserved.join("automations/unknown/draft.bin"), b"unfinished unrelated bytes");
    let before = tree_image(&reserved);
    let override_root = agents.join("research/automations/override");
    write_fixture(
        &override_root.join("automation.json"),
        &serde_json::to_vec(&fabu_automation_payload(&make("_messaging", "override"))).unwrap(),
    );
    write_fixture(&override_root.join("companion.bin"), b"retained external owner bytes\0\xff");
    fs::create_dir_all(override_root.join("empty-directory")).expect("retained override directory");
    let override_before = tree_image(&override_root);

    // The legacy reader remains available for a reviewed code revert. Reading
    // the original archive must not mutate it or silently discard its owners.
    let legacy = BTreeMap::from([
        ("legacy-reserved".into(), make("_messaging", "legacy-reserved")),
        ("legacy-normal".into(), make("research", "legacy-normal")),
    ]);
    persist_automations(&legacy_path, &legacy).expect("write original legacy automation archive");
    let legacy_before = fs::read(&legacy_path).expect("capture original archive bytes");
    let legacy_read = load_automations(&legacy_path);
    assert_eq!(legacy_read.len(), 2);
    assert_eq!(fabu_automation_owner(&legacy_read["legacy-reserved"]), "_messaging");
    assert_eq!(fabu_automation_owner(&legacy_read["legacy-normal"]), "research");

    let assert_retained = || {
        assert_eq!(tree_image(&reserved), before);
        assert_eq!(tree_image(&override_root), override_before);
        assert_eq!(fs::read(&legacy_path).expect("read retained archive"), legacy_before);
    };
    let mut loaded = load_fabu_agent_automations(&agents);
    assert!(loaded.values().all(|item| item.id != "valid" && item.id != "corrupt" && item.id != "unknown"));
    assert!(loaded.values().all(|item| fabu_automation_owner(item) != "_messaging"));
    assert_retained();

    // Even a caller that supplies every old archive entry cannot make the
    // canonical writer import the reserved owner or rewrite the archive. The
    // real account-boundary active-state filter is exercised by AppHost tests.
    for (id, automation) in legacy_read {
        let key = automation_state_key(fabu_automation_owner(&automation), &id);
        loaded.insert(key, automation);
    }

    // A caller cannot bypass the directory scan by supplying a reserved owner
    // directly in desired state; the writer must also enforce the boundary.
    loaded.insert(automation_state_key("_messaging", "injected"), make("_messaging", "injected"));
    let normal_key = automation_state_key("research", "daily");
    loaded.insert(normal_key.clone(), make("research", "daily"));
    persist_fabu_agent_automations(&agents, &loaded).expect("persist ordinary Agent automation");
    assert_retained();
    let normal_path = agents.join("research/automations/daily/automation.json");
    assert!(normal_path.is_file());
    let restored = load_fabu_agent_automations(&agents);
    assert_eq!(restored[&normal_key].name, "daily brief");
    assert_eq!(restored[&automation_state_key("research", "legacy-normal")].name, "daily brief");
    assert!(restored.values().all(|item| fabu_automation_owner(item) != "_messaging"));

    // A normal owner can still collide with a retained marker at the same
    // physical path. Refuse the entire batch before cleanup or any write,
    // including unrelated desired updates and stale ordinary directories.
    let before_collision = tree_image(&root.0);
    let mut collision = loaded.clone();
    collision.insert(
        automation_state_key("research", "override"),
        make("research", "override"),
    );
    collision.get_mut(&normal_key).unwrap().name = "must never be written".into();
    collision.remove(&automation_state_key("research", "legacy-normal"));
    match persist_fabu_agent_automations(&agents, &collision) {
        Err(FeatureHostError::Contract(message)) => assert_eq!(
            message,
            "Agent automation path contains retained standalone messaging metadata"
        ),
        Err(error) => panic!("expected the retained-metadata collision refusal, got {error}"),
        Ok(()) => panic!("canonical writer accepted a retained-metadata path collision"),
    }
    assert_eq!(tree_image(&root.0), before_collision);
    assert_retained();

    loaded.get_mut(&normal_key).unwrap().name = "updated brief".into();
    persist_fabu_agent_automations(&agents, &loaded).expect("update ordinary Agent automation");
    assert_eq!(load_fabu_agent_automations(&agents)[&normal_key].name, "updated brief");
    assert_retained();
    loaded.remove(&normal_key);
    persist_fabu_agent_automations(&agents, &loaded).expect("delete ordinary Agent automation");
    assert!(!normal_path.parent().unwrap().exists());
    assert!(!load_fabu_agent_automations(&agents).contains_key(&normal_key));
    assert_retained();

    // The public command must not leave an active task behind when persistence
    // refuses its path. Use the real default Agent resolution with no agentId.
    let upsert_id = "retained-upsert-collision";
    let upsert_marker = agents
        .join("mahayana-assistant/automations")
        .join(upsert_id);
    write_fixture(
        &upsert_marker.join("automation.json"),
        &serde_json::to_vec(&fabu_automation_payload(&make("_messaging", upsert_id))).unwrap(),
    );
    write_fixture(&upsert_marker.join("companion.bin"), b"retained command-collision bytes");
    let mut controller = owner(&root, HostMode::Test, SurfacePlatform::Electron);
    controller.memory_root_path = Some(agents.clone());
    let original_automations = load_fabu_agent_automations(&agents);
    assert!(original_automations.contains_key(&automation_state_key("research", "legacy-normal")));
    controller.state().unwrap().automations = original_automations.clone();
    let original_sequence = controller.state().unwrap().sequence;
    let original_events = event_image(&controller);
    let before_upsert = tree_image(&root.0);
    match controller.execute(FeatureCommand::AutomationUpsert {
        request_id: "retained-upsert".into(),
        id: Some(upsert_id.into()),
        agent_id: None,
        name: "must not become active".into(),
        prompt: "Must not run after a rejected write.".into(),
        schedule: "@daily".into(),
        trigger: None,
        enabled: true,
    }) {
        Err(FeatureHostError::Contract(message)) => assert_eq!(
            message,
            "Agent automation path contains retained standalone messaging metadata"
        ),
        Err(error) => panic!("expected Upsert persistence refusal, got {error}"),
        Ok(_) => panic!("Upsert accepted a retained-metadata path collision"),
    }
    assert_eq!(controller.state().unwrap().automations, original_automations);
    assert_eq!(controller.state().unwrap().sequence, original_sequence);
    assert_no_new_events(&controller, &original_events);
    assert_eq!(tree_image(&root.0), before_upsert);
    assert_retained();

    controller
        .execute(FeatureCommand::AutomationList {
            request_id: "list-after-rejected-upsert".into(),
            agent_id: None,
        })
        .expect("list remains available after a rejected Upsert");
    let state = controller.state().unwrap();
    assert_eq!(state.automations, original_automations);
    assert_eq!(state.events.len(), original_events.len() + 1);
    match state.events.back() {
        Some(HostEvent::AutomationListed { automations, .. }) => {
            assert_eq!(automations.len(), original_automations.len());
            assert!(automations.iter().all(|item| item.id != upsert_id));
            assert!(automations.iter().any(|item| item.id == "legacy-normal"));
        }
        _ => panic!("expected the actual AutomationList event"),
    }
    drop(state);
    assert_eq!(tree_image(&root.0), before_upsert);
    assert_retained();
}
