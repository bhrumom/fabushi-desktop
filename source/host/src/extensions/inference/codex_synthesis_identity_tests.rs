//! MSI-v1 reaches the real Memory worker, production summary factory, Codex
//! HTTP request, Session apply gate and file store. Only a built request's
//! destination is redirected, inside that worker's existing affine hook.

use super::*;
use std::sync::Condvar;

use crate::extensions::inference::production::SummaryAdmissionHookScope;
use crate::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, MemoryOrigin, MemoryService, MemorySynthesisBridge,
    MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS, SynthesisApplyResult, SynthesisChange, memory_id_for,
};
use crate::extensions::memory::memory_synthesis_service::{
    MemorySynthesisOptions, MemorySynthesisService, SynthesisOutcome, SynthesisReport, SynthesisReportOutcome,
    MEMORY_SYNTHESIS_PROMPT_MARKER, MEMORY_SYNTHESIS_VERIFICATION_PROMPT_MARKER,
};
use crate::extensions::memory::production::production_memory_synthesis_options;
use crate::extensions::session::agent_session::SandAgentSessionStore;

const NOW: i64 = 1_700_000_000_000;
const SOURCE: &str = "MSI historical synthesized fact";
const UPDATED: &str = "MSI verified replacement fact";
const EXPLICIT: &str = "MSI explicit historical fact remains";
const RETIRED: &str = "MSI tombstoned fact remains retired";
const EVIDENCE: &str = "msi-evidence-A";
const NEXT_EVIDENCE: &str = "msi-evidence-B";
const MODEL_B: &str = "Codex/MSI/Later@Model";

struct Expected {
    image: CredentialImage,
    model: &'static str,
    verification: bool,
}

impl Expected {
    fn proposal(image: &CredentialImage, model: &'static str) -> Self {
        Self { image: image.clone(), model, verification: false }
    }
    fn verify(image: &CredentialImage, model: &'static str) -> Self {
        Self { image: image.clone(), model, verification: true }
    }
}

struct Recording {
    expected: VecDeque<Expected>,
    observed: Vec<ObservedRequest>,
    dispatch_workers: Vec<usize>,
}

struct WorkerScope {
    hook: Option<RequestHookScope>,
    active: Arc<AtomicUsize>,
    worker: usize,
}

impl Drop for WorkerScope {
    fn drop(&mut self) {
        drop(self.hook.take());
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}

struct WorkerHttp {
    peer: OwnedPeer,
    recording: Arc<Mutex<Recording>>,
    active: Arc<AtomicUsize>,
    entries: Arc<AtomicUsize>,
}

impl WorkerHttp {
    fn new(expected: Vec<Expected>, replies: Vec<Reply>) -> Self {
        assert_eq!(expected.len(), replies.len());
        Self {
            peer: OwnedPeer::start(replies),
            recording: Arc::new(Mutex::new(Recording {
                expected: expected.into(), observed: Vec::new(), dispatch_workers: Vec::new(),
            })),
            active: Arc::new(AtomicUsize::new(0)),
            entries: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn worker_entry(&self) -> Arc<dyn Fn() -> Box<dyn std::any::Any> + Send + Sync> {
        let recording = self.recording.clone();
        let active = self.active.clone();
        let entries = self.entries.clone();
        let address = self.peer.address;
        let nonce = self.peer.nonce.clone();
        Arc::new(move || {
            assert_eq!(thread::current().name(), Some("sand-memory-synthesis-inference"));
            assert!(capture_request_hook_for_current_thread().is_none());
            active.fetch_add(1, Ordering::SeqCst);
            let worker = entries.fetch_add(1, Ordering::SeqCst) + 1;
            let recording = recording.clone();
            let nonce = nonce.clone();
            let hook = RequestHookScope::install(move |kind, request| {
                assert_eq!(kind, RequestKind::Responses, "no hidden refresh or other dispatch");
                let mut recording = recording.lock().unwrap();
                let expected = recording.expected.pop_front()
                    .expect("unexpected production summary request");
                let headers = request.headers().keys().map(|name| {
                    (name.as_str().to_ascii_lowercase(), request.headers().get_all(name).iter()
                        .map(|v| v.to_str().unwrap().to_string()).collect::<Vec<_>>())
                }).collect::<BTreeMap<_, _>>();
                let body = request.body().and_then(reqwest::Body::as_bytes)
                    .expect("actual reusable request body").to_vec();
                assert_eq!(headers.get("authorization"),
                    Some(&vec![format!("Bearer {}", expected.image.access)]));
                assert_eq!(headers.get("chatgpt-account-id"), Some(&vec![expected.image.workspace]));
                let value: Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(value["model"], expected.model);
                assert_eq!(value["stream"], true);
                assert_eq!(value["store"], false);
                assert!(value.get("tools").is_none_or(|tools| tools.as_array().is_some_and(Vec::is_empty)));
                let text = String::from_utf8(body.clone()).unwrap();
                assert!(text.contains(EXPLICIT), "actual historical memory must reach both phases");
                let marker = if expected.verification {
                    MEMORY_SYNTHESIS_VERIFICATION_PROMPT_MARKER
                } else { MEMORY_SYNTHESIS_PROMPT_MARKER };
                assert!(text.contains(marker));
                if expected.verification {
                    assert!(text.contains(UPDATED), "verification must carry real proposed changes");
                }
                let path = format!("/{nonce}/{}/responses", recording.observed.len());
                recording.observed.push(ObservedRequest {
                    kind, original_url: request.url().to_string(), redirected_path: path.clone(), headers, body,
                });
                recording.dispatch_workers.push(worker);
                reqwest::Url::parse(&format!("http://{address}{path}")).unwrap()
            });
            Box::new(WorkerScope { hook: Some(hook), active: active.clone(), worker })
        })
    }

    fn count(&self) -> usize { self.recording.lock().unwrap().observed.len() }

    fn dispatch_workers(&self) -> Vec<usize> {
        self.recording.lock().unwrap().dispatch_workers.clone()
    }

    fn finish(self, expected_workers: usize) -> Vec<ObservedRequest> {
        let deadline = Instant::now() + Duration::from_secs(3);
        while self.active.load(Ordering::SeqCst) != 0 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(self.active.load(Ordering::SeqCst), 0, "retire exact fixture worker scopes");
        assert_eq!(self.entries.load(Ordering::SeqCst), expected_workers);
        assert!(capture_request_hook_for_current_thread().is_none(), "caller TLS was never changed");
        let wire = self.peer.finish();
        let recording = self.recording.lock().unwrap();
        assert!(recording.expected.is_empty());
        assert_eq!(wire.len(), recording.observed.len());
        for (actual, built) in wire.iter().zip(&recording.observed) {
            assert_eq!(built.original_url, RESPONSES_URL);
            assert_eq!(actual.method, "POST");
            assert_eq!(actual.target, built.redirected_path);
            assert_eq!(actual.body, built.body);
            for (name, values) in &built.headers {
                assert_eq!(actual.headers.get(name), Some(values), "built application header changed");
            }
        }
        recording.observed.clone()
    }
}

struct MemoryFixture {
    workers: Arc<ProductionSessionWorkers>,
    memory: Arc<MemoryService>,
    store: FileMemoryStore,
    agent: String,
    inference: Arc<ProductionInferenceExtension>,
    settings: Arc<SettingsService>,
    data_dir: PathBuf,
}

impl MemoryFixture {
    fn new(root: &Path) -> Self {
        let (settings, inference) = codex_inference_owners(root);
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let sessions = SandAgentSessionStore::new(workers.clone());
        let agent = sessions.create_session(None, "user", None).unwrap().id;
        workers.bind_memory_synthesis_gate();
        let memory = workers.memory_service();
        let store = memory.synthesis_target_for_agent(&agent).unwrap();
        assert_eq!(memory.apply_synthesis_with_gate(&agent, &store, &store.prepare_synthesis(), &[
            SynthesisChange::Create { content: SOURCE.into(), kind: MemoryKind::Profile },
            SynthesisChange::Create { content: RETIRED.into(), kind: MemoryKind::Profile },
        ], NOW), SynthesisApplyResult::Committed);
        workers.with_agent_admitted_filesystem_mutation(&agent, || {
            store.add_memory(EXPLICIT, NOW, MemoryKind::Profile).map_err(|e| e.to_string())?
                .expect("actual explicit fact");
            assert!(store.remove_memory(&memory_id_for(RETIRED)).map_err(|e| e.to_string())?);
            store.mark_temporal_review(NOW - MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS - 1)
                .map_err(|e| e.to_string())
        }).unwrap();
        assert!(store.is_temporal_review_due(NOW));
        Self { workers, memory, store, agent, inference: Arc::new(inference), settings,
            data_dir: root.join("inference-data") }
    }

    fn service(&self, http: &WorkerHttp) -> (Arc<MemorySynthesisService>, Arc<Mutex<Vec<SynthesisReport>>>) {
        self.service_with_options(http, |_| {})
    }

    fn service_with_options(
        &self,
        http: &WorkerHttp,
        configure: impl FnOnce(&mut MemorySynthesisOptions),
    ) -> (Arc<MemorySynthesisService>, Arc<Mutex<Vec<SynthesisReport>>>) {
        let mut options = production_memory_synthesis_options(
            self.memory.clone(), self.inference.clone(), self.data_dir.clone(),
        );
        options.now = Arc::new(|| NOW);
        options.retry_attempts = 3;
        options.retry_initial = Duration::ZERO;
        options.retry_max = Duration::ZERO;
        options.deadline = Duration::from_secs(5);
        options.worker_entry = Some(http.worker_entry());
        configure(&mut options);
        let reports = Arc::new(Mutex::new(Vec::new()));
        let captured = reports.clone();
        options.report = Some(Arc::new(move |report| captured.lock().unwrap().push(report)));
        let service = Arc::new(MemorySynthesisService::new(options));
        service.start();
        let bridge: Arc<dyn MemorySynthesisBridge> = service.clone();
        self.memory.set_synthesis_bridge(Arc::downgrade(&bridge));
        assert!(self.memory.synthesis_enabled());
        (service, reports)
    }

    fn record(&self, id: &str) {
        assert!(self.memory.record_memory_evidence(&self.agent, Some(id.into()),
            "MSI distinctive user evidence", "MSI distinctive assistant evidence", NOW));
    }

    fn proposal(&self, evidence: &str) -> String {
        let snapshot = self.store.prepare_synthesis();
        let original = snapshot.memories.iter().find(|m| m.content == SOURCE).unwrap();
        assert_eq!(original.origin, MemoryOrigin::Synthesis);
        json!({"changes":[{"action":"update","id":original.id,"content":UPDATED,
            "kind":"profile","sourceEvidenceIds":[evidence]}]}).to_string()
    }

    fn assert_committed(&self) {
        let memories = self.store.prepare_synthesis().memories;
        assert!(!memories.iter().any(|m| m.content == SOURCE || m.content == RETIRED));
        assert_eq!(memories.iter().filter(|m| m.content == UPDATED && m.origin == MemoryOrigin::Synthesis).count(), 1);
        assert!(memories.iter().any(|m| m.content == EXPLICIT && m.origin == MemoryOrigin::Explicit));
        assert!(self.store.get_location().join(".dreaming/tombstones")
            .join(format!("{}.deleted", memory_id_for(RETIRED))).is_file());
        assert_eq!(fs::read(self.store.get_location().join(".dreaming/next-refresh-at")).unwrap(),
            format!("{}\n", NOW + MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS).into_bytes());
    }

    fn close(self, service: Arc<MemorySynthesisService>) {
        service.dispose();
        self.memory.clear_synthesis_bridge();
        self.workers.shutdown();
    }
}

fn memory_bytes(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        out.insert(path.strip_prefix(root).unwrap().to_path_buf(), None);
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() { visit(root, &path, out); }
            else {
                assert!(kind.is_file());
                out.insert(path.strip_prefix(root).unwrap().to_path_buf(), Some(fs::read(path).unwrap()));
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}

fn set_model(settings: &SettingsService, model: &str) {
    assert!(settings.set_agent_default_model(Some(&json!({
        "provider":"codex","modelId":model,"maxMode":true,"parameters":[],
    }))).unwrap());
}

fn assert_reports_safe(reports: &[SynthesisReport], images: &[CredentialImage]) {
    let rendered = format!("{reports:?}");
    for image in images {
        for secret in [&image.access, &image.refresh, &image.id] { assert!(!rendered.contains(secret)); }
    }
    assert!(!rendered.contains(SOURCE) && !rendered.contains(UPDATED) && !rendered.contains(EXPLICIT));
}

#[test]
fn memory_synthesis_codex_retains_snapshot_model_route_and_rotating_token() {
    isolated("synthesis_identity_tests::memory_synthesis_codex_retains_snapshot_model_route_and_rotating_token", "positive", |root| {
        let a0 = CredentialImage::new(ACCOUNT_A, "msi-initial");
        let a1 = CredentialImage::new(ACCOUNT_A, "msi-rotated");
        let path = root.join("codex/auth.json");
        write_image(&path, &a0);
        let fixture = MemoryFixture::new(root);
        let settings = fixture.settings.clone();
        let rotated = a1.clone();
        let http = WorkerHttp::new(vec![Expected::proposal(&a0, MODEL), Expected::verify(&a1, MODEL),
            Expected::proposal(&a1, MODEL_B)], vec![
            Reply::terminal(&fixture.proposal(EVIDENCE)).before(move |request| {
                assert!(String::from_utf8_lossy(&request.body).contains(EVIDENCE));
                write_image(&path, &rotated);
                set_model(&settings, MODEL_B);
                assert!(settings.set_inference_provider("openrouter").unwrap());
            }), Reply::terminal("{\"approved\":true}"), Reply::terminal("{\"changes\":[]}"),
        ]);
        let (service, reports) = fixture.service(&http);
        fixture.record(EVIDENCE);
        assert_eq!(service.run_now(), vec![SynthesisOutcome::Committed]);
        fixture.assert_committed();
        assert_eq!(http.count(), 2);
        assert_eq!(fixture.inference.route(), InferenceRoute::Routed(RoutedProvider::OpenRouter));
        assert!(fixture.settings.set_inference_provider("codex").unwrap());
        fixture.record(NEXT_EVIDENCE);
        assert_eq!(service.run_now(), vec![SynthesisOutcome::NoWork]);
        fixture.assert_committed();
        let observations = http.finish(2);
        assert_eq!(observations.len(), 3);
        assert_reports_safe(&reports.lock().unwrap(), &[a0, a1]);
        fixture.close(service);
    });
}

fn refusal_scenario(root: &Path, temporal: bool, first_transport_failure: bool) {
    let a = CredentialImage::new(ACCOUNT_A, "msi-before-refusal");
    let b = CredentialImage::new(ACCOUNT_B, "msi-next-workspace");
    let path = root.join("codex/auth.json");
    write_image(&path, &a);
    let fixture = MemoryFixture::new(root);
    let settings = fixture.settings.clone();
    let replacement = b.clone();
    let first = if first_transport_failure { Reply::status(503) }
        else { Reply::terminal(&fixture.proposal(if temporal { "clock" } else { EVIDENCE })) };
    let http = WorkerHttp::new(vec![Expected::proposal(&a, MODEL),
        Expected::proposal(&b, MODEL_B), Expected::verify(&b, MODEL_B)], vec![
        first.before(move |request| {
            assert!(String::from_utf8_lossy(&request.body).contains(SOURCE));
            write_image(&path, &replacement);
            set_model(&settings, MODEL_B);
        }), Reply::terminal(&fixture.proposal(NEXT_EVIDENCE)), Reply::terminal("{\"approved\":true}"),
    ]);
    let (service, reports) = fixture.service(&http);
    if !temporal { fixture.record(EVIDENCE); }
    let before = memory_bytes(&fixture.store.get_location());
    let admission = fixture.workers.read_session_work_admission(&fixture.agent).unwrap();
    assert_eq!(service.run_now(), vec![SynthesisOutcome::Failed]);
    assert_eq!(http.count(), 1, "refusal must precede verify or a fresh-identity retry");
    assert_eq!(memory_bytes(&fixture.store.get_location()), before);
    assert_eq!(fixture.workers.read_session_work_admission(&fixture.agent).unwrap(), admission);
    assert!(fixture.store.is_temporal_review_due(NOW));
    assert_eq!(reports.lock().unwrap().len(), 1);
    assert_eq!(reports.lock().unwrap()[0].outcome, SynthesisReportOutcome::Failed);
    fixture.record(NEXT_EVIDENCE);
    assert_eq!(service.run_now(), vec![SynthesisOutcome::Committed]);
    fixture.assert_committed();
    assert_eq!(http.finish(if first_transport_failure { 3 } else { 2 }).len(), 3);
    assert_reports_safe(&reports.lock().unwrap(), &[a, b]);
    fixture.close(service);
}

#[test]
fn memory_synthesis_codex_rejects_workspace_change_before_verification_and_recovers() {
    isolated("synthesis_identity_tests::memory_synthesis_codex_rejects_workspace_change_before_verification_and_recovers",
        "between-phases", |root| refusal_scenario(root, false, false));
}

#[test]
fn memory_synthesis_codex_retries_share_identity_and_refuse_recapture() {
    isolated("synthesis_identity_tests::memory_synthesis_codex_retries_share_identity_and_refuse_recapture",
        "retry-refusal", |root| refusal_scenario(root, false, true));
    isolated("synthesis_identity_tests::memory_synthesis_codex_retries_share_identity_and_refuse_recapture",
        "retry-rotation", |root| {
            let a0 = CredentialImage::new(ACCOUNT_A, "msi-retry-initial");
            let a1 = CredentialImage::new(ACCOUNT_A, "msi-retry-rotated");
            let path = root.join("codex/auth.json");
            write_image(&path, &a0);
            let fixture = MemoryFixture::new(root);
            let settings = fixture.settings.clone();
            let rotated = a1.clone();
            let http = WorkerHttp::new(vec![Expected::proposal(&a0, MODEL),
                Expected::proposal(&a1, MODEL), Expected::verify(&a1, MODEL)], vec![
                Reply::status(503).before(move |_| { write_image(&path, &rotated); set_model(&settings, MODEL_B); }),
                Reply::terminal(&fixture.proposal(EVIDENCE)), Reply::terminal("{\"approved\":true}"),
            ]);
            let (service, reports) = fixture.service(&http);
            fixture.record(EVIDENCE);
            assert_eq!(service.run_now(), vec![SynthesisOutcome::Committed]);
            fixture.assert_committed();
            assert_eq!(http.finish(2).len(), 3);
            assert_reports_safe(&reports.lock().unwrap(), &[a0, a1]);
            fixture.close(service);
        });
}

#[test]
fn memory_synthesis_codex_temporal_refusal_preserves_metadata() {
    isolated("synthesis_identity_tests::memory_synthesis_codex_temporal_refusal_preserves_metadata",
        "temporal", |root| refusal_scenario(root, true, false));
}

#[test]
fn memory_synthesis_codex_dispose_during_proposal_stops_verification() {
    isolated("synthesis_identity_tests::memory_synthesis_codex_dispose_during_proposal_stops_verification",
        "cancel", |root| {
            let a = CredentialImage::new(ACCOUNT_A, "msi-cancel");
            write_image(&root.join("codex/auth.json"), &a);
            let fixture = MemoryFixture::new(root);
            let cancel_owner = Arc::new(Mutex::new(None::<std::sync::Weak<MemorySynthesisService>>));
            let cancel = cancel_owner.clone();
            let http = WorkerHttp::new(vec![Expected::proposal(&a, MODEL)], vec![
                Reply::terminal(&fixture.proposal(EVIDENCE)).cancelling(move |request| {
                    assert!(String::from_utf8_lossy(&request.body).contains(EVIDENCE));
                    cancel.lock().unwrap().as_ref().unwrap().upgrade().unwrap().dispose();
                }),
            ]);
            let (service, reports) = fixture.service(&http);
            *cancel_owner.lock().unwrap() = Some(Arc::downgrade(&service));
            fixture.record(EVIDENCE);
            let before = memory_bytes(&fixture.store.get_location());
            assert_eq!(service.run_now(), vec![SynthesisOutcome::NoWork]);
            assert_eq!(http.finish(1).len(), 1);
            assert_eq!(memory_bytes(&fixture.store.get_location()), before);
            assert!(fixture.store.is_temporal_review_due(NOW));
            assert!(!fixture.memory.synthesis_enabled());
            assert!(reports.lock().unwrap().is_empty());
            fixture.close(service);
        });
}

#[derive(Default)]
struct AdmissionBarrierState {
    captures: Vec<(usize, bool)>,
    released: bool,
    retired: Vec<usize>,
}

#[derive(Default)]
struct AdmissionResultBarrier {
    state: Mutex<AdmissionBarrierState>,
    changed: Condvar,
}

impl AdmissionResultBarrier {
    fn observe_capture(&self, worker: usize, succeeded: bool) {
        let mut state = self.state.lock().unwrap();
        state.captures.push((worker, succeeded));
        self.changed.notify_all();
        if worker == 1 {
            let (state, _) = self.changed.wait_timeout_while(
                state, Duration::from_secs(15), |state| !state.released,
            ).unwrap();
            let released = state.released;
            drop(state);
            assert!(released, "admission fixture exhausted its bounded release wait");
        }
    }

    fn assert_first_worker_is_held(&self, succeeded: bool) {
        let (captures, released, retired) = {
            let state = self.state.lock().unwrap();
            (state.captures.clone(), state.released, state.retired.clone())
        };
        assert_eq!(captures, vec![(1, succeeded)], "actual capture precedes the first deadline");
        assert!(!released && !retired.contains(&1), "first worker must still own initialization");
    }

    fn release(&self) {
        self.state.lock().unwrap_or_else(|error| error.into_inner()).released = true;
        self.changed.notify_all();
    }

    fn retire(&self, worker: usize) {
        self.state.lock().unwrap_or_else(|error| error.into_inner()).retired.push(worker);
        self.changed.notify_all();
    }

    fn captures(&self) -> Vec<(usize, bool)> {
        self.state.lock().unwrap().captures.clone()
    }

    fn wait_for_retired_scopes(&self, expected: usize) {
        let (state, _) = self.changed.wait_timeout_while(
            self.state.lock().unwrap(), Duration::from_secs(3),
            |state| state.retired.len() < expected,
        ).unwrap();
        let mut retired = state.retired.clone();
        drop(state);
        retired.sort_unstable();
        assert_eq!(retired, (1..=expected).collect::<Vec<_>>(),
            "every exact worker scope must retire; an unknown late scope is not success");
    }
}

struct AdmissionWorkerScope {
    admission: Option<SummaryAdmissionHookScope>,
    http: Option<Box<dyn std::any::Any>>,
    barrier: Arc<AdmissionResultBarrier>,
    worker: usize,
}

impl Drop for AdmissionWorkerScope {
    fn drop(&mut self) {
        // Also release on a retry-entry assertion or mutation panic. The real
        // Memory deadline already cancelled worker1; no fixture cancellation
        // is substituted for that production signal.
        if self.worker == 2 { self.barrier.release(); }
        drop(self.admission.take());
        drop(self.http.take());
        self.barrier.retire(self.worker);
    }
}

struct AdmissionCleanup {
    service: Arc<MemorySynthesisService>,
    barrier: Arc<AdmissionResultBarrier>,
}

impl Drop for AdmissionCleanup {
    fn drop(&mut self) {
        self.service.dispose();
        self.barrier.release();
    }
}

fn admission_worker_entry(
    http: &WorkerHttp,
    barrier: Arc<AdmissionResultBarrier>,
    capture_succeeded: bool,
    mutate_while_held: impl FnOnce() + Send + 'static,
) -> Arc<dyn Fn() -> Box<dyn std::any::Any> + Send + Sync> {
    let enter_http = http.worker_entry();
    let mutation = Mutex::new(Some(mutate_while_held));
    Arc::new(move || {
        let http = enter_http();
        let worker = http.downcast_ref::<WorkerScope>().expect("actual HTTP worker scope").worker;
        let observed = barrier.clone();
        let admission = SummaryAdmissionHookScope::install(move |succeeded| {
            observed.observe_capture(worker, succeeded);
        });
        let scope = AdmissionWorkerScope {
            admission: Some(admission), http: Some(http), barrier: barrier.clone(), worker,
        };
        if worker == 2 {
            // The first worker cannot have returned while its capture Result
            // is held. Entry of this actual retry therefore observes the real
            // recv_timeout/cancel path, without sleeping to guess its order.
            barrier.assert_first_worker_is_held(capture_succeeded);
            let mutate = mutation.lock().unwrap().take().expect("one retry-entry mutation");
            mutate();
            barrier.release();
        }
        Box::new(scope)
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AdmissionDeadlineScenario {
    SameWorkspaceRotation,
    WorkspaceReplacement,
    InitialAdmissionRefusal,
}

fn admission_deadline_scenario(root: &Path, scenario: AdmissionDeadlineScenario) {
    let initial = CredentialImage::new(ACCOUNT_A, "msi-admission-initial");
    let rotating = scenario == AdmissionDeadlineScenario::SameWorkspaceRotation;
    let capture_succeeded = scenario != AdmissionDeadlineScenario::InitialAdmissionRefusal;
    let replacement = CredentialImage::new(
        if rotating { ACCOUNT_A } else { ACCOUNT_B }, "msi-admission-current",
    );
    let path = root.join("codex/auth.json");
    if capture_succeeded { write_image(&path, &initial); }
    else { assert!(!path.exists(), "owned initial credential must really be absent"); }
    let fixture = MemoryFixture::new(root);
    let memory_root = fixture.store.get_location();
    let before = memory_bytes(&memory_root);
    let admission_before = fixture.workers.read_session_work_admission(&fixture.agent).unwrap();
    let (expected, replies) = if rotating {
        (vec![Expected::proposal(&replacement, MODEL), Expected::verify(&replacement, MODEL),
            Expected::proposal(&replacement, MODEL_B)], vec![
            Reply::terminal(&fixture.proposal(EVIDENCE)), Reply::terminal("{\"approved\":true}"),
            Reply::terminal("{\"changes\":[]}"),
        ])
    } else {
        (vec![Expected::proposal(&replacement, MODEL_B), Expected::verify(&replacement, MODEL_B)], vec![
            Reply::terminal(&fixture.proposal(NEXT_EVIDENCE)), Reply::terminal("{\"approved\":true}"),
        ])
    };
    let http = WorkerHttp::new(expected, replies);
    let barrier = Arc::new(AdmissionResultBarrier::default());
    let settings = fixture.settings.clone();
    let changed = replacement.clone();
    let held_memory_root = memory_root.clone();
    let held_bytes = before.clone();
    let deadline = Duration::from_secs(5);
    let started = Instant::now();
    let entry = admission_worker_entry(&http, barrier.clone(), capture_succeeded, move || {
        assert!(started.elapsed() >= deadline, "retry must follow the real attempt deadline");
        assert_eq!(memory_bytes(&held_memory_root), held_bytes, "held capture cannot apply or mark review");
        write_image(&path, &changed);
        set_model(&settings, MODEL_B);
    });
    let (service, reports) = fixture.service_with_options(&http, move |options| {
        options.deadline = deadline;
        options.worker_entry = Some(entry);
    });
    let _cleanup = AdmissionCleanup { service: service.clone(), barrier: barrier.clone() };
    fixture.record(EVIDENCE);
    assert_eq!(memory_bytes(&memory_root), before);
    assert_eq!(http.entries.load(Ordering::SeqCst), 0);
    assert!(barrier.captures().is_empty(), "no worker capture before run_now");

    let expected_outcome = if rotating { SynthesisOutcome::Committed } else { SynthesisOutcome::Failed };
    assert_eq!(service.run_now(), vec![expected_outcome]);
    let settled = memory_bytes(&memory_root);
    barrier.wait_for_retired_scopes(2);
    assert_eq!(memory_bytes(&memory_root), settled, "timed-out worker must not apply a late result");
    assert_eq!(barrier.captures(), vec![(1, capture_succeeded)],
        "the concurrent retry must reuse the real capture Result, including an initial refusal");
    assert_eq!(http.entries.load(Ordering::SeqCst), 2, "snapshot must use exactly two attempts");
    if rotating {
        assert_eq!(http.dispatch_workers(), vec![2, 2], "only the live retry may propose and verify");
        fixture.assert_committed();
    } else {
        assert_eq!(http.count(), 0, "replacement or sticky admission refusal must precede all HTTP");
        assert!(http.dispatch_workers().is_empty());
        assert_eq!(settled, before, "identity refusal preserves all Memory and temporal bytes");
        assert_eq!(fixture.workers.read_session_work_admission(&fixture.agent).unwrap(), admission_before);
        assert!(fixture.store.is_temporal_review_due(NOW));
    }
    assert_eq!(reports.lock().unwrap().len(), 1);
    assert_eq!(reports.lock().unwrap()[0].outcome, if rotating {
        SynthesisReportOutcome::Committed
    } else { SynthesisReportOutcome::Failed });
    assert_eq!(fixture.inference.port().resolve_codex_turn_model(None, None, None).unwrap(), MODEL_B);

    // A new run_agent snapshot is the first allowed admission of the current
    // model/workspace after either failed identity path; the old job is retired.
    fixture.record(NEXT_EVIDENCE);
    assert_eq!(service.run_now(), vec![if rotating { SynthesisOutcome::NoWork }
        else { SynthesisOutcome::Committed }]);
    barrier.wait_for_retired_scopes(3);
    assert_eq!(barrier.captures(), vec![(1, capture_succeeded), (3, true)]);
    assert_eq!(http.dispatch_workers(), if rotating { vec![2, 2, 3] } else { vec![3, 3] });
    fixture.assert_committed();
    if rotating { assert_eq!(memory_bytes(&memory_root), settled); }
    assert_eq!(reports.lock().unwrap().len(), 2);
    assert_eq!(reports.lock().unwrap()[1].outcome, if rotating {
        SynthesisReportOutcome::NoWork
    } else { SynthesisReportOutcome::Committed });
    assert_reports_safe(&reports.lock().unwrap(), &[initial, replacement]);
    assert_eq!(http.finish(3).len(), if rotating { 3 } else { 2 });
    fixture.close(service);
}

#[test]
fn memory_synthesis_codex_admission_deadline_retains_identity_and_sticky_refusal() {
    let name = "synthesis_identity_tests::memory_synthesis_codex_admission_deadline_retains_identity_and_sticky_refusal";
    for (scenario, control) in [
        ("admission-deadline-rotation", AdmissionDeadlineScenario::SameWorkspaceRotation),
        ("admission-deadline-workspace", AdmissionDeadlineScenario::WorkspaceReplacement),
        ("admission-deadline-refusal", AdmissionDeadlineScenario::InitialAdmissionRefusal),
    ] {
        isolated(name, scenario, |root| admission_deadline_scenario(root, control));
    }
}
