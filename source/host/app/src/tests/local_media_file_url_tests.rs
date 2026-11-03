use super::super::*;
use super::channel_host_lane_tests::{IdleRunnerOwners, idle_runner_owners};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use mahayana_host_runtime::attachment_paths::{
    get_agent_assets_dir, get_agent_attachments_dir, get_agent_media_store_roots,
};
use mahayana_host_runtime::runner::tools::sand_agent_management_tools::AgentManagementToolBridge;
use mahayana_host_runtime::runner::tools::send_message_tool::{
    SendMessageToolBridge, send_message_tool_definition,
};

// The bytes and dimensions are local fixture oracles, not a codec-equivalence
// or live provider/channel acceptance claim.
const PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
    0, 0, 0, 1, 0, 0, 0, 1, 8, 4, 0, 0, 0, 181, 28, 12, 2,
    0, 0, 0, 11, 73, 68, 65, 84, 120, 218, 99, 100, 248, 15, 0,
    1, 5, 1, 1, 39, 24, 227, 102, 0, 0, 0, 0, 73, 69, 78, 68,
    174, 66, 96, 130,
];

struct UnusedLocalMediaDelegate;

impl RoutedToolBridge for UnusedLocalMediaDelegate {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(Vec::new())
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        panic!("first-party media tools must not dispatch to a provider delegate");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MediaAdmissionSnapshot {
    session_key: String,
    lifecycle_revision: String,
    work_id: Option<String>,
    paths: Vec<String>,
}

fn media_admission(sessions: &ProductionSessionWorkers, agent_id: &str) -> MediaAdmissionSnapshot {
    let record = sessions.read_session_work_admission(agent_id)
        .expect("read the actual Session admission")
        .expect("the fixture has a materialized Session");
    MediaAdmissionSnapshot {
        session_key: record.session_key,
        lifecycle_revision: record.lifecycle_revision,
        work_id: record.admitted_media_work_id,
        paths: record.admitted_media_paths,
    }
}

#[derive(Debug, Clone)]
struct ResolutionObservation {
    requested_url: String,
    resolved: ResolvedAttachmentSource,
    admission: MediaAdmissionSnapshot,
    pending_work_id: Option<String>,
    owned_bytes: Vec<(PathBuf, Vec<u8>)>,
}

// This is an observation port, not a replacement resolver or persistence
// pipeline. Every applicable sink operation delegates to the private shipping
// sink. Successful resolution is observed before the real bridge can process
// the next image, so rollback assertions cannot pass merely because no first
// image was ever materialized/admitted.
struct ObservedProductionSendSink {
    production: Arc<ProductionSendMessageSink>,
    resolutions: Mutex<Vec<ResolutionObservation>>,
    aborts: Mutex<Vec<String>>,
    sends: AtomicUsize,
}

impl ObservedProductionSendSink {
    fn new(production: Arc<ProductionSendMessageSink>) -> Self {
        Self {
            production,
            resolutions: Mutex::new(Vec::new()),
            aborts: Mutex::new(Vec::new()),
            sends: AtomicUsize::new(0),
        }
    }

    fn resolutions(&self) -> Vec<ResolutionObservation> {
        self.resolutions.lock().expect("resolution observations").clone()
    }
}

impl SendMessageSink for ObservedProductionSendSink {
    fn is_awaiting_user_selection(&self) -> bool {
        self.production.is_awaiting_user_selection()
    }

    fn resolve_cloud_agent_title(&self, bc_id: &str) -> Option<String> {
        self.production.resolve_cloud_agent_title(bc_id)
    }

    fn request_box_help(
        &self,
        request: BoxHelpRequest,
        timestamp_ms: u64,
        tool_call_id: &str,
    ) -> Result<BoxHelpOutcome, ProviderSessionError> {
        self.production.request_box_help(request, timestamp_ms, tool_call_id)
    }

    fn resolve_attachment_source(
        &self,
        source_url: &str,
        tool_call_id: &str,
    ) -> Result<ResolvedAttachmentSource, ProviderSessionError> {
        let resolved = self.production.resolve_attachment_source(source_url, tool_call_id)?;
        let admission = media_admission(&self.production.sessions, &self.production.agent_id);
        let pending_work_id = self.production.pending_media.lock()
            .expect("actual in-flight custody map")
            .get(tool_call_id).and_then(|pending| pending.media_work_id.clone());
        let owned_bytes = admission.paths.iter().map(|path| {
            let path = PathBuf::from(path);
            let bytes = fs::read(&path).expect("actual admitted bytes already exist");
            (path, bytes)
        }).collect();
        self.resolutions.lock().expect("resolution observations").push(ResolutionObservation {
            requested_url: source_url.to_string(),
            resolved: resolved.clone(),
            admission,
            pending_work_id,
            owned_bytes,
        });
        Ok(resolved)
    }

    fn read_media_dimensions(&self, resolved_url: &str) -> Option<(u32, u32)> {
        self.production.read_media_dimensions(resolved_url)
    }

    fn abort_attachment_custody(&self, tool_call_id: &str) {
        self.production.abort_attachment_custody(tool_call_id);
        self.aborts.lock().expect("abort observations").push(tool_call_id.to_string());
    }

    fn send_message(
        &self,
        message: Value,
        timestamp_ms: u64,
        tool_call_id: &str,
    ) -> Result<Option<String>, ProviderSessionError> {
        self.sends.fetch_add(1, Ordering::SeqCst);
        self.production.send_message(message, timestamp_ms, tool_call_id)
    }
}

struct LocalMediaAppFixture {
    root: PathBuf,
    agent_id: String,
    session_key: String,
    owners: Option<IdleRunnerOwners>,
    sink: Option<Arc<ProductionSendMessageSink>>,
    host_requests: mpsc::Receiver<HostLaneRequest>,
    remove_on_drop: bool,
}

impl LocalMediaAppFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-local-media-app-{label}-{}", uuid::Uuid::new_v4(),
        ));
        fs::create_dir_all(&root).expect("owned fixture directory");
        fs::write(root.join("settings.json"), b"{}").expect("credential-free settings");
        Self::open(root, None)
    }

    fn open(root: PathBuf, existing_agent_id: Option<String>) -> Self {
        let (host_tx, host_requests) = mpsc::channel();
        let events = GatewayEventHub::default();
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let agent_id = existing_agent_id.unwrap_or_else(|| {
            sessions.materialize_new_session(None, "user", None).expect("actual Session").id
        });
        let admission = media_admission(&sessions, &agent_id);
        let manager = Arc::new(TranscriptManager::new(&root, Arc::clone(&sessions)));
        let owners = idle_runner_owners(&root, host_tx.clone(), events.clone(), sessions, manager);
        let deps = &owners.deps;
        let roster_events = events.clone();
        let roster = Arc::new(ProductionRosterEmit::new(
            Arc::clone(&deps.session_workers), Arc::clone(&deps.transcript_runtime),
            Arc::new(move |event| roster_events.publish(event)),
        ));
        deps.transcript_manager.bind_roster_emit(roster).expect("actual roster owner");
        deps.transcript_runtime.session_runtime().set_window_focused(
            &deps.session_workers, true, 1_000,
        ).expect("actual focused desktop state");
        let sink = Arc::new(ProductionSendMessageSink {
            host_tx,
            cloud_agents: Arc::clone(&deps.cloud_agents),
            sessions: Arc::clone(&deps.session_workers),
            forever_box: Arc::clone(&deps.forever_box),
            transcript_manager: Arc::clone(&deps.transcript_manager),
            events,
            ack_obligations: Arc::clone(&deps.ack_obligations),
            transcript_runtime: Arc::clone(&deps.transcript_runtime),
            ack_token: None,
            agent_id: agent_id.clone(),
            session_key: Some(admission.session_key.clone()),
            reply_thread_target: None,
            is_fork: false,
            pending_media: Mutex::new(HashMap::new()),
        });
        Self {
            root, agent_id, session_key: admission.session_key, owners: Some(owners),
            sink: Some(sink), host_requests, remove_on_drop: true,
        }
    }

    fn deps(&self) -> &LocalRoutedRunnerDeps {
        &self.owners.as_ref().expect("live actual owners").deps
    }

    fn source(&self, name: &str, bytes: &[u8]) -> (PathBuf, String) {
        let directory = self.root.join("incoming");
        fs::create_dir_all(&directory).expect("owned source directory");
        let path = directory.join(name);
        fs::write(&path, bytes).expect("owned local source");
        let url = file_url_for_path(&path).expect("native absolute file URL");
        (path, url)
    }

    fn bridge(&self) -> (SendMessageToolBridge, Arc<ObservedProductionSendSink>) {
        let sink = Arc::new(ObservedProductionSendSink::new(Arc::clone(
            self.sink.as_ref().expect("actual SendMessage sink"),
        )));
        let port: Arc<dyn SendMessageSink> = sink.clone();
        (SendMessageToolBridge::new(Arc::new(UnusedLocalMediaDelegate), port), sink)
    }

    fn entries(&self) -> Vec<Value> {
        self.deps().session_workers.read_agent_transcript_entries(&self.agent_id)
            .expect("actual Session transcript")
    }

    fn admission(&self) -> MediaAdmissionSnapshot {
        media_admission(&self.deps().session_workers, &self.agent_id)
    }

    fn agent_dir(&self) -> PathBuf {
        self.deps().session_workers.session_db_path(&self.agent_id)
            .expect("actual Session database path").parent()
            .expect("database parent").to_path_buf()
    }

    fn media_files(&self) -> BTreeSet<PathBuf> {
        fn collect(directory: &Path, files: &mut BTreeSet<PathBuf>) {
            if !directory.exists() { return; }
            for entry in fs::read_dir(directory).expect("owned media directory") {
                let entry = entry.expect("owned media directory entry");
                if entry.file_type().expect("owned media file type").is_dir() {
                    collect(&entry.path(), files);
                } else {
                    files.insert(entry.path());
                }
            }
        }
        let mut files = BTreeSet::new();
        for directory in get_agent_media_store_roots(self.agent_dir()) {
            collect(&directory, &mut files);
        }
        files
    }

    fn assert_custody_idle(&self) {
        let admission = self.admission();
        assert!(admission.work_id.is_none(), "durable media identity must be released");
        assert!(admission.paths.is_empty(), "durable media paths must be released");
        assert!(self.sink.as_ref().expect("actual sink").pending_media.lock()
            .expect("actual custody map").is_empty());
    }

    fn assert_no_host_request(&self) {
        assert!(matches!(self.host_requests.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "URL refusal must precede a Host-lane delivery request");
    }

    fn dispose_owners(&mut self) {
        drop(self.sink.take());
        if let Some(owners) = self.owners.take() {
            owners.dispose(&self.agent_id);
        }
    }

    fn reopen(mut self) -> Self {
        self.dispose_owners();
        self.remove_on_drop = false;
        let root = self.root.clone();
        let agent_id = self.agent_id.clone();
        drop(self);
        Self::open(root, Some(agent_id))
    }

    fn close(mut self) {
        self.dispose_owners();
        fs::remove_dir_all(&self.root).expect("remove only this fixture's closed directory");
        self.remove_on_drop = false;
    }
}

impl Drop for LocalMediaAppFixture {
    fn drop(&mut self) {
        self.dispose_owners();
        if self.remove_on_drop {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn encoded_separator_url(native_url: &str) -> String {
    let (parent, leaf) = native_url.rsplit_once('/').expect("native source URL has a filename");
    // Decoding this separator would address the same existing owned file.
    // No negative fixture names an external host or network share.
    format!("{parent}%2F{leaf}")
}

fn assert_invalid_source(error: ProviderSessionError) {
    match error {
        ProviderSessionError::Tool(message) => assert!(
            message.starts_with("invalid local media file URL:"),
            "the real App resolver must propagate its refusal, got: {message}",
        ),
        other => panic!("expected the App's local source refusal, got: {other:?}"),
    }
}

fn call_send(
    bridge: &SendMessageToolBridge,
    args: Value,
    tool_call_id: &str,
) -> Result<Value, ProviderSessionError> {
    bridge.call_tool(&send_message_tool_definition(), args, tool_call_id)
}

fn assert_observed_owned_media(
    fixture: &LocalMediaAppFixture,
    observation: &ResolutionObservation,
    source_url: &str,
    source_name: &str,
    bytes: &[u8],
    tool_call_id: &str,
    image: bool,
) -> PathBuf {
    assert_eq!(observation.requested_url, source_url);
    assert_ne!(observation.resolved.url, source_url, "the real local source must materialize");
    assert_eq!(observation.resolved.file_name.as_deref(), Some(source_name));
    assert_eq!(observation.admission.session_key, fixture.session_key);
    let work_id = format!("send-media:{tool_call_id}");
    assert_eq!(observation.admission.work_id.as_deref(), Some(work_id.as_str()));
    assert_eq!(observation.pending_work_id.as_deref(), Some(work_id.as_str()));
    assert_eq!(observation.owned_bytes.len(), 1, "one real managed file before publication");
    let (path, observed_bytes) = &observation.owned_bytes[0];
    assert_eq!(observed_bytes, bytes);
    assert_eq!(observation.admission.paths, vec![path.to_str().expect("fixture path").to_string()]);
    assert_eq!(file_path_from_file_url(&observation.resolved.url).as_deref(), Some(path.as_path()));
    let expected_directory = if image {
        get_agent_assets_dir(fixture.agent_dir())
    } else {
        get_agent_attachments_dir(fixture.agent_dir())
    };
    assert_eq!(path.parent(), Some(expected_directory.as_path()));
    path.clone()
}

#[test]
fn local_media_file_url_app_rejects_invalid_before_materialization() {
    let fixture = LocalMediaAppFixture::new("reject");
    let (source, source_url) = fixture.source("owned.png", PNG);
    let initial_entries = fixture.entries();
    let initial_admission = fixture.admission();
    let initial_files = fixture.media_files();
    let events = fixture.deps().events.subscribe();
    let (bridge, observer) = fixture.bridge();
    let invalid_urls = [
        encoded_separator_url(&source_url),
        encoded_separator_url(&source_url).replace("%2F", "%2f"),
        format!("{source_url}%5Cignored"),
        format!("{source_url}%"),
        format!("{source_url}%XZ"),
        format!("{source_url}%FF"),
        format!("{source_url}%00"),
    ];
    for (index, url) in invalid_urls.iter().enumerate() {
        let call_id = format!("invalid-local-{index}");
        let error = call_send(&bridge, json!({
            "type": "attachment", "url": url,
            "channel": "slack:owned-fixture:thread",
        }), &call_id).expect_err("invalid file URL must never become original-URL success");
        assert_invalid_source(error);
        assert_eq!(fixture.entries(), initial_entries);
        assert_eq!(fixture.admission(), initial_admission);
        assert_eq!(fixture.media_files(), initial_files);
        assert_eq!(fs::read(&source).expect("original still exists"), PNG);
        fixture.assert_custody_idle();
        fixture.assert_no_host_request();
    }
    assert!(observer.resolutions().is_empty(), "no invalid source completes resolution");
    assert_eq!(observer.sends.load(Ordering::SeqCst), 0, "no real send or delivery is entered");
    assert_eq!(observer.aborts.lock().expect("abort observations").len(), invalid_urls.len());
    assert!(matches!(events.try_recv(), Err(mpsc::TryRecvError::Empty)),
        "no transcript publication on refusal");
    drop(bridge);
    drop(observer);
    fixture.close();
}

#[test]
fn local_media_file_url_app_rolls_back_earlier_image_on_later_refusal() {
    let fixture = LocalMediaAppFixture::new("partial-images");
    let (source, source_url) = fixture.source("first.png", PNG);
    let invalid_url = encoded_separator_url(&source_url);
    let initial_entries = fixture.entries();
    let initial_admission = fixture.admission();
    let initial_files = fixture.media_files();
    let events = fixture.deps().events.subscribe();
    let (bridge, observer) = fixture.bridge();
    let call_id = "two-images-second-invalid";
    let error = call_send(&bridge, json!({
        "type": "text", "content": "both images must be admitted before sending",
        "images": [{"url": source_url, "alt": "first"}, {"url": invalid_url, "alt": "second"}],
        "channel": "slack:owned-fixture:thread",
    }), call_id).expect_err("the second source must refuse the whole message");
    assert_invalid_source(error);
    let observations = observer.resolutions();
    assert_eq!(observations.len(), 1, "first real resolution succeeds; second is refused");
    let owned = assert_observed_owned_media(
        &fixture, &observations[0], &source_url, "first.png", PNG, call_id, true,
    );
    assert_ne!(owned, source);
    assert!(!owned.exists(), "actual bridge abort removes the earlier managed file");
    assert_eq!(fs::read(&source).expect("original source survives rollback"), PNG);
    assert_eq!(fixture.entries(), initial_entries);
    assert_eq!(fixture.admission(), initial_admission, "actual abort releases the durable media fence");
    assert_eq!(fixture.media_files(), initial_files);
    fixture.assert_custody_idle();
    fixture.assert_no_host_request();
    assert_eq!(observer.sends.load(Ordering::SeqCst), 0);
    assert_eq!(*observer.aborts.lock().expect("actual abort observation"), vec![call_id.to_string()]);
    assert!(matches!(events.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(bridge);
    drop(observer);
    fixture.close();
}

#[test]
fn local_media_file_url_app_commits_native_and_unicode_media_across_reopen() {
    let fixture = LocalMediaAppFixture::new("native-unicode");
    let initial_files = fixture.media_files();
    let (bridge, observer) = fixture.bridge();
    let cases: [(&str, &[u8], bool); 2] = [
        ("native attachment.bin", b"exact native attachment bytes\0\xFF", false),
        ("本地图 像.png", PNG, true),
    ];
    let mut preserved = Vec::new();
    for (index, (name, bytes, image)) in cases.into_iter().enumerate() {
        let (source, native_url) = fixture.source(name, bytes);
        let source_url = if image {
            format!("FiLe://localhost{}", native_url.strip_prefix("file://").expect("native scheme"))
        } else { native_url };
        let before = fixture.entries();
        let call_id = format!("native-media-{index}");
        let result = call_send(&bridge, json!({
            "type": "attachment", "url": source_url, "alt": "local media",
        }), &call_id).expect("valid native URL reaches the actual sink");
        assert_eq!(result["sent"], true);
        let observations = observer.resolutions();
        assert_eq!(observations.len(), index + 1);
        let observation = &observations[index];
        let owned = assert_observed_owned_media(
            &fixture, observation, &source_url, name, bytes, &call_id, image,
        );
        let entries = fixture.entries();
        assert_eq!(entries.len(), before.len() + 1);
        assert_eq!(&entries[..before.len()], before.as_slice());
        let entry = entries.last().expect("actual persisted message");
        assert_eq!(entry["id"], result["messageId"]);
        assert_eq!(entry["kind"], "send-message");
        assert_eq!(entry["sessionKey"], fixture.session_key);
        assert_eq!(entry["message"]["url"], observation.resolved.url);
        assert_eq!(entry["message"]["file_name"], name);
        assert_eq!(entry["message"]["alt"], "local media");
        if image {
            assert_eq!(entry["message"]["width"], 1);
            assert_eq!(entry["message"]["height"], 1);
        }
        assert_eq!(fs::read(&owned).expect("committed managed bytes"), bytes);
        assert_eq!(fs::read(&source).expect("original source preserved"), bytes);
        fixture.assert_custody_idle();
        fixture.assert_no_host_request();
        preserved.push((source, owned, bytes.to_vec()));
    }
    assert_eq!(observer.sends.load(Ordering::SeqCst), 2);
    assert!(observer.aborts.lock().expect("abort observations").is_empty());
    let expected_entries = fixture.entries();
    let expected_admission = fixture.admission();
    let expected_files = fixture.media_files();
    assert_eq!(expected_files.len(), initial_files.len() + 2);
    assert!(expected_files.is_superset(&initial_files));
    drop(bridge);
    drop(observer);
    let fixture = fixture.reopen();
    assert_eq!(fixture.entries(), expected_entries, "fresh Session owner reads exact persisted messages");
    assert_eq!(fixture.admission(), expected_admission);
    assert_eq!(fixture.media_files(), expected_files);
    for (source, owned, bytes) in preserved {
        assert_eq!(fs::read(source).expect("original source after reopen"), bytes);
        assert_eq!(fs::read(owned).expect("committed media after reopen"), bytes);
    }
    fixture.assert_custody_idle();
    fixture.close();
}

#[test]
fn local_media_file_url_app_repeated_refusal_then_repair_survives_reopen() {
    let fixture = LocalMediaAppFixture::new("repair-reopen");
    let (source, source_url) = fixture.source("repair.png", PNG);
    let invalid_url = encoded_separator_url(&source_url);
    let (bridge, observer) = fixture.bridge();
    let initial_entries = fixture.entries();
    let initial_admission = fixture.admission();
    let initial_files = fixture.media_files();
    for index in 0..3 {
        assert_invalid_source(call_send(&bridge, json!({
            "type": "attachment", "url": invalid_url,
        }), &format!("retry-invalid-{index}")).expect_err("repeat refusal is not a successful retry"));
        assert_eq!(fixture.entries(), initial_entries);
        assert_eq!(fixture.admission(), initial_admission);
        assert_eq!(fixture.media_files(), initial_files);
    }
    assert!(observer.resolutions().is_empty());
    assert_eq!(observer.sends.load(Ordering::SeqCst), 0);
    let repaired = call_send(&bridge, json!({"type": "attachment", "url": source_url}), "repaired-first")
        .expect("repair is a new real send");
    assert_eq!(repaired["sent"], true);
    let first_owned = assert_observed_owned_media(
        &fixture, &observer.resolutions()[0], &source_url, "repair.png", PNG, "repaired-first", true,
    );
    let committed_entries = fixture.entries();
    let committed_admission = fixture.admission();
    let committed_files = fixture.media_files();
    assert_eq!(committed_entries.len(), initial_entries.len() + 1);
    assert_eq!(committed_files.len(), initial_files.len() + 1);
    fixture.assert_custody_idle();
    drop(bridge);
    drop(observer);
    let fixture = fixture.reopen();
    assert_eq!(fixture.entries(), committed_entries);
    assert_eq!(fixture.admission(), committed_admission);
    assert_eq!(fs::read(&first_owned).expect("first repaired media survived real reopen"), PNG);
    let (bridge, observer) = fixture.bridge();
    for index in 0..2 {
        assert_invalid_source(call_send(&bridge, json!({
            "type": "attachment", "url": invalid_url,
        }), &format!("reopened-invalid-{index}")).expect_err("reopened owner still refuses invalid URL"));
        assert_eq!(fixture.entries(), committed_entries);
        assert_eq!(fixture.admission(), committed_admission);
        assert_eq!(fixture.media_files(), committed_files);
        assert_eq!(fs::read(&first_owned).expect("refusal preserves earlier committed media"), PNG);
        fixture.assert_custody_idle();
        fixture.assert_no_host_request();
    }
    assert_eq!(observer.sends.load(Ordering::SeqCst), 0);
    let second = call_send(&bridge, json!({"type": "attachment", "url": source_url}), "repaired-after-reopen")
        .expect("a fresh valid send remains available after repeated refusal");
    assert_eq!(second["sent"], true);
    assert_ne!(second["messageId"], repaired["messageId"]);
    let second_owned = assert_observed_owned_media(
        &fixture, &observer.resolutions()[0], &source_url, "repair.png", PNG, "repaired-after-reopen", true,
    );
    assert_ne!(second_owned, first_owned, "fresh send owns a distinct file");
    assert_eq!(fixture.entries().len(), committed_entries.len() + 1);
    assert_eq!(fixture.media_files().len(), committed_files.len() + 1);
    assert_eq!(fs::read(&second_owned).expect("second repaired managed bytes"), PNG);
    assert_eq!(fs::read(&source).expect("source survives all refused and accepted calls"), PNG);
    fixture.assert_custody_idle();
    fixture.assert_no_host_request();
    drop(bridge);
    drop(observer);
    fixture.close();
}

#[test]
fn local_media_file_url_app_agent_management_refuses_image_before_outward_send() {
    fn bridge_for(
        fixture: &LocalMediaAppFixture,
        wake_requests: &Arc<Mutex<Vec<AgentWakeRequest>>>,
        agent_messages: &Arc<Mutex<Vec<String>>>,
    ) -> (AgentManagementToolBridge, Arc<ProductionAgentManagementSink>, RoutedToolDefinition) {
        let observed_wakes = Arc::clone(wake_requests);
        let messaging = Arc::new(ProductionAgentToAgentMessaging::new(
            Arc::clone(&fixture.deps().session_workers),
            Arc::new(move |request| observed_wakes.lock().expect("wake observations").push(request.clone())),
            None,
        ));
        let sink = Arc::new(ProductionAgentManagementSink {
            sessions: Arc::clone(&fixture.deps().session_workers),
            forever_box: Arc::clone(&fixture.deps().forever_box),
            messaging,
            agent_messages: Arc::clone(agent_messages),
            agent_id: fixture.agent_id.clone(),
            session_key: Some(fixture.session_key.clone()),
            reply_to_id: None,
        });
        let sink_port: Arc<dyn AgentManagementSink> = sink.clone();
        let bridge = AgentManagementToolBridge::new(Arc::new(UnusedLocalMediaDelegate), sink_port);
        let tool = bridge.list_tools().expect("actual first-party tool definitions").into_iter()
            .find(|tool| tool.name == "SendToAgent").expect("actual SendToAgent definition");
        (bridge, sink, tool)
    }

    let mut fixture = LocalMediaAppFixture::new("agent-management");
    let (source, source_url) = fixture.source("agent-image.png", PNG);
    let invalid_url = encoded_separator_url(&source_url);
    let target_id = fixture.deps().session_workers.materialize_new_session(None, "user", None)
        .expect("actual recipient Session").id;
    let source_entries = fixture.entries();
    let target_entries = fixture.deps().session_workers.read_agent_transcript_entries(&target_id)
        .expect("actual target transcript");
    let source_admission = fixture.admission();
    let target_admission = media_admission(&fixture.deps().session_workers, &target_id);
    let source_files = fixture.media_files();
    let wake_requests = Arc::new(Mutex::new(Vec::<AgentWakeRequest>::new()));
    let agent_messages = Arc::new(Mutex::new(Vec::<String>::new()));

    for cycle in 0..2 {
        let (bridge, sink, tool) = bridge_for(&fixture, &wake_requests, &agent_messages);
        let image = AgentMessageImage { url: invalid_url.clone(), alt: Some("refuse".into()) };
        assert_invalid_source(sink.resolve_image_source(&image, "direct-agent-image")
            .expect_err("the actual AgentManagement sink must not return the invalid original URL"));
        assert_invalid_source(bridge.call_tool(&tool, json!({
            "target_id": target_id, "message": "must not escape URL admission",
            "images": [{"url": invalid_url, "alt": "refuse"}],
        }), "agent-image-invalid").expect_err("the actual bridge propagates image admission refusal"));
        for attempt in 0..2 {
            // A later invalid URL must refuse the value batch before the earlier
            // valid image can create an unowned managed file in this distinct sink.
            assert_invalid_source(bridge.call_tool(&tool, json!({
                "target_id": target_id, "message": "admit both image sources before materialization",
                "images": [
                    {"url": source_url, "alt": "valid first source"},
                    {"url": invalid_url, "alt": "invalid later source"},
                ],
            }), &format!("agent-image-later-invalid-{cycle}-{attempt}"))
                .expect_err("later invalid URL must refuse before any image resolves"));
            assert_eq!(fixture.entries(), source_entries);
            assert_eq!(fixture.deps().session_workers.read_agent_transcript_entries(&target_id)
                .expect("target remains unchanged"), target_entries);
            assert_eq!(fixture.admission(), source_admission);
            assert_eq!(media_admission(&fixture.deps().session_workers, &target_id), target_admission);
            assert_eq!(fixture.media_files(), source_files);
            assert_eq!(fs::read(&source).expect("original image remains"), PNG);
            assert!(wake_requests.lock().expect("actual messaging wake port").is_empty());
            assert!(agent_messages.lock().expect("actual outgoing message tracking").is_empty());
            fixture.assert_custody_idle();
            fixture.assert_no_host_request();
        }
        let remote = AgentMessageImage {
            url: "https://example.invalid/agent-image.png".into(), alt: Some("remote".into()),
        };
        let resolved = sink.resolve_image_source(&remote, "agent-image-https").expect("non-file passthrough");
        assert_eq!(resolved.url, remote.url);
        assert_eq!(resolved.alt, remote.alt);
        assert_eq!(fixture.media_files(), source_files);
        drop(bridge);
        drop(sink);
        fixture = fixture.reopen();
        assert_eq!(fixture.entries(), source_entries);
        assert_eq!(fixture.media_files(), source_files, "owner reopen must not conceal accumulated files");
        fixture.assert_custody_idle();
    }

    // Actual positive control: the same source and bridge can materialize a
    // local image and persist an outgoing message after the repeated refusal.
    // The wake port observes the real messaging owner; it does not run a target Agent.
    let (bridge, sink, tool) = bridge_for(&fixture, &wake_requests, &agent_messages);
    let message = "valid local image after refusal";
    let result = bridge.call_tool(&tool, json!({
        "target_id": target_id, "message": message,
        "images": [{"url": source_url, "alt": "admitted local source"}],
    }), "agent-image-valid-after-refusal").expect("valid native source reaches actual messaging");
    assert!(result.as_str().expect("actual messaging result").starts_with("Sent to "));
    let committed_files = fixture.media_files();
    let added = committed_files.difference(&source_files).cloned().collect::<Vec<_>>();
    assert_eq!(added.len(), 1, "exactly one actual managed image is materialized");
    assert_eq!(fs::read(&added[0]).expect("committed managed image bytes"), PNG);
    let managed_url = file_url_for_path(&added[0]).expect("native managed image URL");
    let committed_entries = fixture.entries();
    assert_eq!(committed_entries.len(), source_entries.len() + 1);
    assert_eq!(&committed_entries[..source_entries.len()], source_entries.as_slice());
    let entry = committed_entries.last().expect("actual outgoing transcript entry");
    assert_eq!(entry["toAgent"]["id"], target_id);
    assert_eq!(entry["content"], message);
    assert_eq!(entry["images"][0]["url"], managed_url);
    assert_eq!(entry["images"][0]["alt"], "admitted local source");
    {
        let wakes = wake_requests.lock().expect("actual messaging wake port");
        assert_eq!(wakes.len(), 1);
        assert_eq!(wakes[0].agent_id, target_id);
        assert_eq!(wakes[0].source_agent_id, fixture.agent_id);
        assert_eq!(wakes[0].selected_images.len(), 1);
        assert_eq!(wakes[0].selected_images[0]["data"], json!(PNG));
        assert_eq!(wakes[0].inbound.as_ref().expect("actual inbound envelope")["images"][0]["url"], managed_url);
    }
    assert_eq!(agent_messages.lock().expect("actual outgoing tracking").as_slice(), &[message.to_string()]);
    assert_eq!(fixture.deps().session_workers.read_agent_transcript_entries(&target_id)
        .expect("wake observation does not execute the target"), target_entries);
    fixture.assert_custody_idle();
    fixture.assert_no_host_request();
    drop(bridge);
    drop(sink);
    fixture = fixture.reopen();
    assert_eq!(fixture.entries(), committed_entries);
    assert_eq!(fixture.media_files(), committed_files);
    assert_eq!(fs::read(&added[0]).expect("managed bytes survive actual owner reopen"), PNG);
    assert_eq!(fs::read(&source).expect("source retained after successful send"), PNG);
    fixture.assert_custody_idle();
    fixture.close();
}

#[test]
fn local_media_file_url_app_preserves_https_without_local_custody() {
    let fixture = LocalMediaAppFixture::new("https");
    let initial_entries = fixture.entries();
    let initial_admission = fixture.admission();
    let initial_files = fixture.media_files();
    let (bridge, observer) = fixture.bridge();
    // The URI is only stored/passed through; no HTTP client or live account is
    // involved. Encoded slash is deliberately legal in this non-file URI.
    let url = "https://example.invalid/media/%2Fimage.png?download=1";
    let result = call_send(&bridge, json!({"type": "attachment", "url": url}), "https-passthrough")
        .expect("HTTPS remains a normal SendMessage attachment");
    assert_eq!(result["sent"], true);
    let observations = observer.resolutions();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].resolved.url, url);
    assert!(observations[0].resolved.file_name.is_none());
    assert_eq!(observations[0].admission, initial_admission);
    assert!(observations[0].pending_work_id.is_none());
    assert!(observations[0].owned_bytes.is_empty());
    assert_eq!(observer.sends.load(Ordering::SeqCst), 1);
    let entries = fixture.entries();
    assert_eq!(entries.len(), initial_entries.len() + 1);
    assert_eq!(&entries[..initial_entries.len()], initial_entries.as_slice());
    let entry = entries.last().expect("actual persisted HTTPS message");
    assert_eq!(entry["id"], result["messageId"]);
    assert_eq!(entry["kind"], "send-message");
    assert_eq!(entry["sessionKey"], fixture.session_key);
    assert_eq!(entry["message"]["url"], url);
    assert!(entry["message"].get("file_name").is_none());
    assert_eq!(fixture.media_files(), initial_files);
    fixture.assert_custody_idle();
    fixture.assert_no_host_request();
    drop(bridge);
    drop(observer);
    fixture.close();
}
