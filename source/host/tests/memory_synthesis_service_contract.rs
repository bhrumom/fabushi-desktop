use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak, mpsc};
use std::time::Duration;

use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, SynthesisApplyResult, SynthesisChange, SynthesisSnapshot,
};
use mahayana_host_runtime::extensions::memory::memory_synthesis_service::{
    MEMORY_SYNTHESIS_DEADLINE_MS, MEMORY_SYNTHESIS_DEBOUNCE_MS,
    MEMORY_SYNTHESIS_POLL_INTERVAL_MS, MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS, MemoryEvidence, MemorySynthesisOptions,
    MemorySynthesisService, SynthesisInferenceError, SynthesisModelJob, SynthesisOutcome,
    SynthesisReportOutcome, SynthesisTarget,
    bounded_evidence_text, parse_memory_synthesis_changes, synthesis_request_json,
    synthesis_system_prompt, uses_known_evidence, verification_request_json,
    verification_system_prompt,
};
use serde_json::json;

fn temp_store() -> (std::path::PathBuf, Arc<FileMemoryStore>) {
    let root = std::env::temp_dir().join(format!(
        "fabushi-memory-synthesis-service-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&root).unwrap();
    let store = Arc::new(FileMemoryStore::with_synthesis_metadata(
        root.join("memory"),
        true,
    ));
    (root, store)
}

#[test]
fn parser_rejects_unknown_evidence_and_clock_only_creation() {
    let raw = json!({"changes":[{
        "action":"create",
        "content":"Prefers concise replies",
        "kind":"profile",
        "sourceEvidenceIds":["ev-1"]
    }]});
    let changes = parse_memory_synthesis_changes(&raw).unwrap();
    let known = ["ev-1".to_string()].into_iter().collect();
    assert!(uses_known_evidence(&known, &changes, false));
    assert!(!uses_known_evidence(&Default::default(), &changes, false));

    let clock = json!({"changes":[{
        "action":"create",
        "content":"A new unsupported fact",
        "kind":"log",
        "sourceEvidenceIds":["clock"]
    }]});
    let clock_changes = parse_memory_synthesis_changes(&clock).unwrap();
    assert!(!uses_known_evidence(
        &Default::default(),
        &clock_changes,
        true
    ));
}

#[test]
fn proposal_and_verification_commit_through_synthesis_target() {
    let (root, store) = temp_store();
    let targets = Arc::new(Mutex::new(HashMap::from([(
        "agent-1".to_string(),
        store.clone(),
    )])));
    let list_targets = {
        let targets = targets.clone();
        Arc::new(move || {
            targets
                .lock()
                .unwrap()
                .iter()
                .map(|(id, store)| {
                    (
                        id.clone(),
                        store.clone()
                            as Arc<dyn mahayana_host_runtime::extensions::memory::memory_synthesis_service::SynthesisTarget>,
                    )
                })
                .collect()
        })
    };
    let get_target = {
        let targets = targets.clone();
        Arc::new(move |id: &str| {
            targets
                .lock()
                .unwrap()
                .get(id)
                .cloned()
                .map(|store| {
                    store
                        as Arc<dyn mahayana_host_runtime::extensions::memory::memory_synthesis_service::SynthesisTarget>
                })
        })
    };
    let proposes = Arc::new(AtomicUsize::new(0));
    let proposes_for_call = proposes.clone();
    let mut options = MemorySynthesisOptions::new(
        list_targets,
        get_target,
        Arc::new(move |_request, _cancel| {
            proposes_for_call.fetch_add(1, Ordering::SeqCst);
            Ok(json!({"changes":[{
                "action":"create",
                "content":"Prefers concise replies",
                "kind":"profile",
                "sourceEvidenceIds":["ev-1"]
            }]}))
        }),
    );
    options.set_verifier(Arc::new(|_request, _cancel| Ok(true)));
    options.debounce = Duration::from_millis(1);
    options.deadline = Duration::from_secs(1);
    options.poll_interval = Duration::from_secs(60);
    options.retry_initial = Duration::from_millis(1);
    options.retry_max = Duration::from_millis(2);

    let service = MemorySynthesisService::new(options);
    service.start();
    service.record_turn(
        "agent-1",
        Some("ev-1".into()),
        "Please keep replies concise",
        "Understood",
        1_700_000_000_000,
    );
    let outcomes = service.run_now();
    assert_eq!(outcomes, vec![SynthesisOutcome::Committed]);
    assert_eq!(proposes.load(Ordering::SeqCst), 1);
    let memories = store.prepare_synthesis().memories;
    assert_eq!(memories.len(), 1);
    assert_eq!(memories[0].content, "Prefers concise replies");
    service.dispose();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn verifier_rejection_does_not_commit_and_retryable_failures_stop_at_bound() {
    let (root, store) = temp_store();
    let target = store.clone()
        as Arc<dyn mahayana_host_runtime::extensions::memory::memory_synthesis_service::SynthesisTarget>;
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_call = attempts.clone();
    let mut options = MemorySynthesisOptions::new(
        Arc::new({
            let target = target.clone();
            move || vec![("agent".into(), target.clone())]
        }),
        Arc::new({
            let target = target.clone();
            move |_id| Some(target.clone())
        }),
        Arc::new(move |_request, _cancel| {
            attempts_for_call.fetch_add(1, Ordering::SeqCst);
            Ok(json!({"changes":[{
                "action":"create",
                "content":"Needs verification",
                "kind":"log",
                "sourceEvidenceIds":["ev"]
            }]}))
        }),
    );
    options.set_verifier(Arc::new(|_request, _cancel| Ok(false)));
    options.debounce = Duration::from_secs(60);
    options.deadline = Duration::from_secs(1);
    options.poll_interval = Duration::from_secs(60);
    options.retry_attempts = 1;
    let service = MemorySynthesisService::new(options);
    service.start();
    service.record_turn("agent", Some("ev".into()), "u", "a", 1);
    assert_eq!(service.run_now(), vec![SynthesisOutcome::Rejected]);
    assert_eq!(store.count_memories(), 0);
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    service.dispose();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn evidence_bounding_keeps_both_ends() {
    let long = format!("{}{}", "a".repeat(9_000), "z".repeat(9_000));
    let bounded = bounded_evidence_text(&long);
    assert!(bounded.starts_with('a'));
    assert!(bounded.ends_with('z'));
    assert!(bounded.contains("[...middle omitted...]"));
}


#[test]
fn frozen_synthesis_prompts_and_wire_payloads_are_preserved() {
    use mahayana_host_runtime::extensions::memory::memory_service::{
        MemoryKind, MemoryOrigin, SynthesisMemory,
    };
    use mahayana_host_runtime::extensions::memory::memory_synthesis_service::{
        MemoryChange, SynthesisProposalRequest, SynthesisVerificationRequest,
    };

    assert!(synthesis_system_prompt().starts_with("<<SAND_MEMORY_SYNTHESIS_V1>>"));
    assert!(verification_system_prompt()
        .starts_with("<<SAND_MEMORY_SYNTHESIS_VERIFICATION_V1>>"));

    let proposal = SynthesisProposalRequest {
        today: "2026-09-29".into(),
        current_memories: vec![SynthesisMemory {
            id: "m1".into(),
            content: "Prefers concise replies".into(),
            created_at: 123,
            kind: MemoryKind::Profile,
            origin: MemoryOrigin::Explicit,
        }],
        new_evidence: vec![MemoryEvidence {
            id: "ev1".into(),
            occurred_at: 456,
            user: "Please stay concise".into(),
            assistant: "Understood".into(),
        }],
    };
    let proposal_json: serde_json::Value =
        serde_json::from_str(&synthesis_request_json(&proposal)).unwrap();
    assert_eq!(proposal_json["today"], "2026-09-29");
    assert_eq!(proposal_json["currentMemories"][0]["origin"], "explicit");
    assert_eq!(proposal_json["newEvidence"][0]["id"], "ev1");

    let verification = SynthesisVerificationRequest {
        today: proposal.today,
        current_memories: proposal.current_memories,
        evidence: proposal.new_evidence,
        proposed_changes: vec![MemoryChange::Create {
            content: "Prefers concise replies".into(),
            kind: MemoryKind::Profile,
            source_evidence_ids: vec!["ev1".into()],
        }],
    };
    let verification_json: serde_json::Value =
        serde_json::from_str(&verification_request_json(&verification)).unwrap();
    assert_eq!(verification_json["proposedChanges"][0]["action"], "create");
    assert_eq!(
        verification_json["proposedChanges"][0]["sourceEvidenceIds"][0],
        "ev1"
    );
}


#[test]
fn frozen_production_scheduling_constants_are_enforced() {
    assert_eq!(MEMORY_SYNTHESIS_DEBOUNCE_MS, 15_000);
    assert_eq!(MEMORY_SYNTHESIS_DEADLINE_MS, 90_000);
    assert_eq!(MEMORY_SYNTHESIS_POLL_INTERVAL_MS, 3_600_000);
}

#[test]
fn synthesis_deadline_cancels_verification_and_reports_terminal_failure() {
    let (root, store) = temp_store();
    let target = store.clone()
        as Arc<dyn mahayana_host_runtime::extensions::memory::memory_synthesis_service::SynthesisTarget>;
    let saw_cancel = Arc::new(AtomicBool::new(false));
    let saw_cancel_for_verify = Arc::clone(&saw_cancel);
    let reports = Arc::new(Mutex::new(Vec::new()));
    let reports_for_callback = Arc::clone(&reports);

    let mut options = MemorySynthesisOptions::new(
        Arc::new({
            let target = target.clone();
            move || vec![("agent".into(), target.clone())]
        }),
        Arc::new({
            let target = target.clone();
            move |_id| Some(target.clone())
        }),
        Arc::new(|_request, _cancel| {
            Ok(json!({"changes":[{
                "action":"create",
                "content":"Deadline protected memory",
                "kind":"log",
                "sourceEvidenceIds":["ev"]
            }]}))
        }),
    );
    options.set_verifier(Arc::new(move |_request, cancel| {
        while !cancel.is_cancelled() {
            std::thread::sleep(Duration::from_millis(1));
        }
        saw_cancel_for_verify.store(true, Ordering::SeqCst);
        Err("cancelled".into())
    }));
    options.deadline = Duration::from_millis(25);
    options.retry_attempts = 1;
    options.report = Some(Arc::new(move |report| {
        reports_for_callback.lock().unwrap().push(report);
    }));

    let service = MemorySynthesisService::new(options);
    service.start();
    service.record_turn("agent", Some("ev".into()), "u", "a", 1);
    assert_eq!(service.run_now(), vec![SynthesisOutcome::Failed]);
    for _ in 0..100 {
        if saw_cancel.load(Ordering::SeqCst) {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(saw_cancel.load(Ordering::SeqCst));
    let reports = reports.lock().unwrap();
    assert_eq!(reports.last().unwrap().outcome, SynthesisReportOutcome::Failed);
    assert_eq!(store.count_memories(), 0);
    service.dispose();
    let _ = fs::remove_dir_all(root);
}

fn options_for_job(
    store: Arc<FileMemoryStore>,
    factory: Arc<dyn Fn() -> SynthesisModelJob + Send + Sync>,
) -> MemorySynthesisOptions {
    let list_target: Arc<dyn SynthesisTarget> = store;
    let get_target = Arc::clone(&list_target);
    let mut options = MemorySynthesisOptions::new_with_job_factory(
        Arc::new(move || vec![("agent".into(), Arc::clone(&list_target))]),
        Arc::new(move |id| (id == "agent").then(|| Arc::clone(&get_target))),
        factory,
    );
    options.retry_initial = Duration::from_millis(1);
    options.retry_max = Duration::from_millis(1);
    options
}

#[test]
fn typed_identity_refusals_settle_without_retry_or_temporal_review() {
    for refusal in [
        SynthesisInferenceError::Authentication,
        SynthesisInferenceError::Configuration,
        SynthesisInferenceError::Cancelled,
    ] {
        let (root, store) = temp_store();
        let now = 1_700_000_000_000;
        store.add_memory("Existing memory remains", now, MemoryKind::Profile).unwrap();
        store.mark_temporal_review(now
            - mahayana_host_runtime::extensions::memory::memory_synthesis_service::MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS
            - 1).unwrap();
        let marker = store.get_location().join(".dreaming/next-refresh-at");
        let before = fs::read(&marker).unwrap();
        let snapshot = store.prepare_synthesis();
        let factories = Arc::new(AtomicUsize::new(0));
        let attempts = Arc::new(AtomicUsize::new(0));
        let factory_count = Arc::clone(&factories);
        let proposed_count = Arc::clone(&attempts);
        let mut options = options_for_job(Arc::clone(&store), Arc::new(move || {
            factory_count.fetch_add(1, Ordering::SeqCst);
            let attempts = Arc::clone(&proposed_count);
            let refusal = refusal.clone();
            SynthesisModelJob::new(Arc::new(move |_, _| {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err(refusal.clone())
            }), None)
        }));
        options.now = Arc::new(move || now);
        options.retry_attempts = 3;
        let reports = Arc::new(Mutex::new(Vec::new()));
        let reported = Arc::clone(&reports);
        options.report = Some(Arc::new(move |report| reported.lock().unwrap().push(report)));
        let service = MemorySynthesisService::new(options);
        service.start();
        service.record_turn("agent", Some("refused-evidence".into()), "u", "a", now);
        assert_eq!(service.run_now(), vec![SynthesisOutcome::Failed]);
        assert_eq!(factories.load(Ordering::SeqCst), 1);
        assert_eq!(attempts.load(Ordering::SeqCst), 1, "typed refusals are terminal");
        assert_eq!(fs::read(&marker).unwrap(), before);
        assert_eq!(store.prepare_synthesis(), snapshot);
        assert!(store.is_temporal_review_due(now));
        assert_eq!(reports.lock().unwrap().len(), 1);
        assert_eq!(reports.lock().unwrap()[0].outcome, SynthesisReportOutcome::Failed);
        assert!(service.run_now().is_empty());
        service.dispose();
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn one_snapshot_factory_survives_worker_timeout_and_retry() {
    let (root, store) = temp_store();
    let factories = Arc::new(AtomicUsize::new(0));
    let factory_count = Arc::clone(&factories);
    let observed_jobs = Arc::new(Mutex::new(Vec::new()));
    let jobs = Arc::clone(&observed_jobs);
    let (cancelled_tx, cancelled_rx) = mpsc::sync_channel(1);
    let mut options = options_for_job(Arc::clone(&store), Arc::new(move || {
        let job_id = factory_count.fetch_add(1, Ordering::SeqCst) + 1;
        let attempts = Arc::new(AtomicUsize::new(0));
        let jobs = Arc::clone(&jobs);
        let cancelled = cancelled_tx.clone();
        SynthesisModelJob::new(Arc::new(move |request, cancel| {
            jobs.lock().unwrap().push(job_id);
            if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                while !cancel.is_cancelled() {
                    std::thread::sleep(Duration::from_millis(1));
                }
                let _ = cancelled.try_send(());
                return Err(SynthesisInferenceError::Cancelled);
            }
            Ok(json!({"changes":[{
                "action":"create","content":"Retained snapshot after timeout","kind":"profile",
                "sourceEvidenceIds":[request.new_evidence[0].id],
            }]}))
        }), Some(Arc::new(|_, _| Ok(true))))
    }));
    options.deadline = Duration::from_millis(200);
    options.retry_attempts = 2;
    let service = MemorySynthesisService::new(options);
    service.start();
    service.record_turn("agent", Some("timeout-evidence".into()), "u", "a", 1);
    assert_eq!(service.run_now(), vec![SynthesisOutcome::Committed]);
    cancelled_rx.recv_timeout(Duration::from_secs(3)).expect("first worker observed its deadline");
    assert_eq!(factories.load(Ordering::SeqCst), 1);
    assert_eq!(*observed_jobs.lock().unwrap(), vec![1, 1],
        "a timed-out worker and its retry must retain the same allocated job");
    assert_eq!(store.prepare_synthesis().memories.iter()
        .filter(|memory| memory.content == "Retained snapshot after timeout").count(), 1);
    assert!(service.run_now().is_empty());
    service.dispose();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn stale_resnapshot_allocates_new_job_and_preserves_concurrent_evidence() {
    let (root, store) = temp_store();
    let now = 1_700_000_000_000;
    store.add_memory("Initial explicit memory", now, MemoryKind::Profile).unwrap();
    let refresh = store.get_location().join(".dreaming/next-refresh-at");
    let before_refresh = fs::read(&refresh).ok();
    let factories = Arc::new(AtomicUsize::new(0));
    let factory_count = Arc::clone(&factories);
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&requests);
    let current_service = Arc::new(Mutex::new(None::<Weak<MemorySynthesisService>>));
    let queued_service = Arc::clone(&current_service);
    let changed_store = Arc::clone(&store);
    let mut options = options_for_job(Arc::clone(&store), Arc::new(move || {
        let job_id = factory_count.fetch_add(1, Ordering::SeqCst) + 1;
        let requests = Arc::clone(&captured);
        let service = Arc::clone(&queued_service);
        let store = Arc::clone(&changed_store);
        SynthesisModelJob::new(Arc::new(move |request, _| {
            requests.lock().unwrap().push(request.clone());
            if job_id == 1 {
                store.add_memory("Concurrent explicit memory", now, MemoryKind::Profile).unwrap();
                service.lock().unwrap().as_ref().unwrap().upgrade().unwrap()
                    .record_turn("agent", Some("evidence-2".into()), "new user", "new answer", now);
            }
            Ok(json!({"changes":[{
                "action":"create","content":"Grounded resnapshot result","kind":"profile",
                "sourceEvidenceIds":request.new_evidence.iter().map(|item| item.id.clone()).collect::<Vec<_>>(),
            }]}))
        }), Some(Arc::new(|_, _| Ok(true))))
    }));
    options.now = Arc::new(move || now);
    options.retry_attempts = 1;
    let service = Arc::new(MemorySynthesisService::new(options));
    *current_service.lock().unwrap() = Some(Arc::downgrade(&service));
    service.start();
    service.record_turn("agent", Some("evidence-1".into()), "user", "answer", now);
    assert_eq!(service.run_now(), vec![SynthesisOutcome::Stale]);
    assert_eq!(factories.load(Ordering::SeqCst), 1);
    assert_eq!(fs::read(&refresh).ok(), before_refresh);
    assert!(!store.prepare_synthesis().memories.iter()
        .any(|memory| memory.content == "Grounded resnapshot result"));
    assert_eq!(service.run_now(), vec![SynthesisOutcome::Committed]);
    assert_eq!(factories.load(Ordering::SeqCst), 2, "Stale starts a new snapshot admission");
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].new_evidence.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(),
        vec!["evidence-1"]);
    assert_eq!(requests[1].new_evidence.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(),
        vec!["evidence-1", "evidence-2"]);
    assert!(requests[1].current_memories.iter()
        .any(|memory| memory.content == "Concurrent explicit memory"));
    drop(requests);
    assert_eq!(store.prepare_synthesis().memories.iter()
        .filter(|memory| memory.content == "Grounded resnapshot result").count(), 1);
    assert!(service.run_now().is_empty());
    service.dispose();
    fs::remove_dir_all(root).unwrap();
}

struct BlockedSnapshotTarget {
    store: Arc<FileMemoryStore>,
    entered: mpsc::SyncSender<()>,
    release: Mutex<mpsc::Receiver<()>>,
    block_once: AtomicBool,
    applies: Arc<AtomicUsize>,
    reviews: Arc<AtomicUsize>,
}

impl SynthesisTarget for BlockedSnapshotTarget {
    fn prepare_synthesis(&self) -> std::io::Result<SynthesisSnapshot> {
        if self.block_once.swap(false, Ordering::SeqCst) {
            self.entered.send(()).expect("announce actual read boundary");
            self.release.lock().unwrap().recv_timeout(Duration::from_secs(5))
                .expect("release owned read boundary");
        }
        self.store.prepare_synthesis_checked()
    }

    fn apply_synthesis(&self, snapshot: &SynthesisSnapshot, changes: &[SynthesisChange],
        now: i64) -> SynthesisApplyResult {
        self.applies.fetch_add(1, Ordering::SeqCst);
        self.store.apply_synthesis(snapshot, changes, now).unwrap_or(SynthesisApplyResult::Failed)
    }

    fn has_memories(&self) -> bool { self.store.has_memories() }
    fn is_temporal_review_due(&self, now: i64) -> bool { self.store.is_temporal_review_due(now) }
    fn mark_temporal_review(&self, now: i64) {
        self.reviews.fetch_add(1, Ordering::SeqCst);
        self.store.mark_temporal_review(now).unwrap();
    }
}

fn blocked_snapshot_read(cancel: bool) {
    let (root, store) = temp_store();
    let now = 1_700_000_000_000;
    store.add_memory("Complete readable snapshot fact", now, MemoryKind::Profile).unwrap();
    store.mark_temporal_review(now - MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS - 1).unwrap();
    let profile = store.get_location().join("profile.md");
    let marker = store.get_location().join(".dreaming/next-refresh-at");
    let repaired = fs::read(&profile).unwrap();
    let review_bytes = fs::read(&marker).unwrap();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let applies = Arc::new(AtomicUsize::new(0));
    let reviews = Arc::new(AtomicUsize::new(0));
    let target: Arc<dyn SynthesisTarget> = Arc::new(BlockedSnapshotTarget {
        store: store.clone(), entered: entered_tx, release: Mutex::new(release_rx),
        block_once: AtomicBool::new(true), applies: applies.clone(), reviews: reviews.clone(),
    });
    let listed = target.clone();
    let selected = target.clone();
    let factories = Arc::new(AtomicUsize::new(0));
    let allocated = factories.clone();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let proposed = requests.clone();
    let mut options = MemorySynthesisOptions::new_with_job_factory(
        Arc::new(move || vec![("agent".into(), listed.clone())]),
        Arc::new(move |id| (id == "agent").then(|| selected.clone())),
        Arc::new(move || {
            allocated.fetch_add(1, Ordering::SeqCst);
            let proposed = proposed.clone();
            SynthesisModelJob::new(Arc::new(move |request, _| {
                proposed.lock().unwrap().push(request);
                Ok(json!({"changes":[]}))
            }), Some(Arc::new(|_, _| panic!("empty proposal does not verify"))))
        }),
    );
    options.now = Arc::new(move || now);
    options.retry_attempts = 3;
    let reports = Arc::new(Mutex::new(Vec::new()));
    let reported = reports.clone();
    options.report = Some(Arc::new(move |report| reported.lock().unwrap().push(report)));
    let service = Arc::new(MemorySynthesisService::new(options));
    service.start();
    service.record_turn("agent", Some("refused-evidence".into()), "first user", "first reply", now);
    let running = service.clone();
    let worker = std::thread::spawn(move || running.run_now());
    entered_rx.recv_timeout(Duration::from_secs(5)).expect("owned acquisition entered");
    let mut corrupt = repaired.clone();
    corrupt.push(0xff);
    fs::write(&profile, &corrupt).unwrap();
    assert_eq!(fs::read_to_string(&profile).unwrap_err().kind(), std::io::ErrorKind::InvalidData);
    service.record_turn("agent", Some("concurrent-evidence".into()), "later user", "later reply", now);
    if cancel { service.dispose(); }
    release_tx.send(()).unwrap();
    assert_eq!(worker.join().expect("owned acquisition worker"),
        vec![if cancel { SynthesisOutcome::NoWork } else { SynthesisOutcome::Failed }]);
    assert_eq!(factories.load(Ordering::SeqCst), 0);
    assert!(requests.lock().unwrap().is_empty());
    assert_eq!(applies.load(Ordering::SeqCst), 0);
    assert_eq!(reviews.load(Ordering::SeqCst), 0);
    assert_eq!(fs::read(&profile).unwrap(), corrupt);
    assert_eq!(fs::read(&marker).unwrap(), review_bytes);
    assert!(store.is_temporal_review_due(now));
    if cancel {
        assert!(reports.lock().unwrap().is_empty(), "cancelled acquisition cannot report Failed");
        assert!(service.run_now().is_empty());
    } else {
        let report = reports.lock().unwrap()[0].clone();
        assert_eq!(report.outcome, SynthesisReportOutcome::Failed);
        assert_eq!((report.evidence_count, report.input_memory_count, report.change_count), (1, 0, 0));
        assert_eq!(reports.lock().unwrap().len(), 1);
        fs::write(&profile, &repaired).expect("explicit repair after read refusal");
        assert_eq!(service.run_now(), vec![SynthesisOutcome::NoWork]);
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].new_evidence.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(),
            vec!["concurrent-evidence"], "only captured evidence settles on refusal");
        assert_eq!(requests[0].current_memories.len(), 1);
        assert_eq!(requests[0].current_memories[0].content, "Complete readable snapshot fact");
        drop(requests);
        assert_eq!(factories.load(Ordering::SeqCst), 1);
        assert_eq!(reports.lock().unwrap()[1].outcome, SynthesisReportOutcome::NoWork);
        assert!(service.run_now().is_empty(), "neither failed nor repaired attempt replays");
        assert_eq!(fs::read(&marker).unwrap(), review_bytes,
            "later evidence-only NoWork retains the existing temporal settlement policy");
    }
    service.dispose();
    drop(service);
    drop(target);
    drop(store);
    fs::remove_dir_all(root).expect("remove retired acquisition fixture");
}

#[test]
fn snapshot_read_refusal_settles_only_captured_evidence_and_recovers() {
    blocked_snapshot_read(false);
}

#[test]
fn snapshot_read_cancellation_suppresses_failure_and_model_work() {
    blocked_snapshot_read(true);
}
