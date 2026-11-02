use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use super::ProductionSynthesisTarget;
use crate::extensions::memory::memory_service::{
    MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS, MemoryKind, MemoryOrigin, SynthesisApplyResult,
    SynthesisChange, format_memory_date, memory_id_for,
};
use crate::extensions::memory::memory_synthesis_service::{
    MemorySynthesisOptions, MemorySynthesisService, SynthesisCancelSignal, SynthesisOutcome,
    SynthesisProposalRequest, SynthesisReport, SynthesisReportOutcome, SynthesisTarget,
    parse_memory_synthesis_changes, protects_explicit_memories, uses_known_evidence,
};
use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::production::ProductionSessionWorkers;

const NOW_MS: i64 = 1_700_000_000_000;
const SOURCE_FACT: &str = "Synthesized source before temporal update";
const DESTINATION_HISTORY: &str = "Destination history must survive";
const EXPLICIT_FACT: &str = "Unrelated explicit fact must survive";
const RETIRED_FACT: &str = "Previously forgotten unrelated fact";
const UPDATED_FACT: &str = "Synthesized fact after successful retry";
const REFRESH_PATH: &str = ".dreaming/next-refresh-at";

struct Fixture {
    root: PathBuf,
    workers: Arc<ProductionSessionWorkers>,
    target: Arc<ProductionSynthesisTarget>,
    destination_kind: MemoryKind,
}

impl Fixture {
    fn new(label: &str, destination_kind: MemoryKind) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-production-temporal-{label}-{}", uuid::Uuid::new_v4().simple(),
        ));
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let sessions = SandAgentSessionStore::new(Arc::clone(&workers));
        let agent = sessions.create_session(None, "user", None).expect("real admitted agent");
        let admission = workers.read_session_work_admission(&agent.id).unwrap().unwrap();
        assert_eq!(admission.session_id, agent.id);
        assert_eq!(admission.session_key, format!("agent:{}:main", agent.id));
        assert!(!admission.lifecycle_revision.is_empty());
        assert!(!admission.deletion_pending);
        workers.bind_memory_synthesis_gate();
        let service = workers.memory_service();
        let store = service.synthesis_target_for_agent(&agent.id).expect("canonical synthesis store");
        let target = Arc::new(ProductionSynthesisTarget { agent_id: agent.id, store, service });
        let source_kind = match destination_kind {
            MemoryKind::Profile => MemoryKind::Log,
            MemoryKind::Log => MemoryKind::Profile,
        };
        assert_eq!(target.apply_synthesis(&target.prepare_synthesis(), &[
            SynthesisChange::Create { content: SOURCE_FACT.into(), kind: source_kind },
            SynthesisChange::Create { content: DESTINATION_HISTORY.into(), kind: destination_kind },
            SynthesisChange::Create { content: RETIRED_FACT.into(), kind: source_kind },
        ], NOW_MS), SynthesisApplyResult::Committed);
        workers.with_agent_admitted_filesystem_mutation(&target.agent_id, || {
            target.store.add_memory(EXPLICIT_FACT, NOW_MS, source_kind)
                .map_err(|error| error.to_string())?.expect("new explicit fixture fact");
            assert!(target.store.remove_memory(&memory_id_for(RETIRED_FACT))
                .map_err(|error| error.to_string())?);
            target.store.mark_temporal_review(NOW_MS - MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS - 1)
                .map_err(|error| error.to_string())
        }).expect("seed metadata through admitted filesystem owner");
        assert!(target.is_temporal_review_due(NOW_MS));
        assert_eq!(fs::read(target.store.get_location().join(REFRESH_PATH)).unwrap(),
            format!("{}\n", NOW_MS - 1).into_bytes());
        Self { root, workers, target, destination_kind }
    }

    fn memory_dir(&self) -> PathBuf {
        self.target.store.get_location()
    }

    fn destination(&self) -> PathBuf {
        match self.destination_kind {
            MemoryKind::Profile => self.memory_dir().join("profile.md"),
            MemoryKind::Log => self.memory_dir().join("log")
                .join(format!("{}.md", &format_memory_date(NOW_MS)[..7])),
        }
    }

    fn assert_committed_history(&self) {
        let records = self.target.prepare_synthesis().memories;
        assert!(!records.iter().any(|record| record.content == SOURCE_FACT));
        assert!(records.iter().any(|record| record.content == DESTINATION_HISTORY));
        assert!(records.iter().any(|record| record.content == EXPLICIT_FACT
            && record.origin == MemoryOrigin::Explicit));
        let updated = records.iter().filter(|record| record.content == UPDATED_FACT).collect::<Vec<_>>();
        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].kind, self.destination_kind);
        assert_eq!(updated[0].origin, MemoryOrigin::Synthesis);
        assert!(self.memory_dir().join(".dreaming/tombstones")
            .join(format!("{}.deleted", memory_id_for(RETIRED_FACT))).is_file());
        assert_eq!(fs::read(self.memory_dir().join(REFRESH_PATH)).unwrap(), reviewed_bytes());
        assert!(!self.target.is_temporal_review_due(NOW_MS));
    }

    fn close(self) {
        self.workers.shutdown();
        fs::remove_dir_all(self.root).expect("remove owned temporal fixture");
    }
}

fn memory_tree(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(root: &Path, path: &Path, tree: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        tree.insert(path.strip_prefix(root).unwrap().to_path_buf(), None);
        for entry in fs::read_dir(path).expect("read complete memory tree") {
            let entry = entry.expect("memory entry");
            let path = entry.path();
            let kind = entry.file_type().expect("memory entry type");
            if kind.is_dir() {
                visit(root, &path, tree);
            } else {
                assert!(kind.is_file(), "fixture contains only ordinary files and directories");
                tree.insert(path.strip_prefix(root).unwrap().to_path_buf(),
                    Some(fs::read(path).expect("raw memory bytes")));
            }
        }
    }
    let mut tree = BTreeMap::new();
    visit(root, root, &mut tree);
    tree
}

fn reviewed_bytes() -> Vec<u8> {
    format!("{}\n", NOW_MS + MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS).into_bytes()
}

fn update_proposal(id: &str, kind: MemoryKind, evidence_id: &str) -> Value {
    json!({"changes": [{
        "action": "update", "id": id, "content": UPDATED_FACT,
        "kind": match kind { MemoryKind::Profile => "profile", MemoryKind::Log => "log" },
        "sourceEvidenceIds": [evidence_id],
    }]})
}

fn service_for(
    fixture: &Fixture,
    propose: impl Fn(SynthesisProposalRequest, SynthesisCancelSignal) -> Result<Value, String>
        + Send + Sync + 'static,
) -> (MemorySynthesisService, Arc<Mutex<Vec<SynthesisReport>>>) {
    let list_target: Arc<dyn SynthesisTarget> = fixture.target.clone();
    let get_target = Arc::clone(&list_target);
    let list_agent = fixture.target.agent_id.clone();
    let get_agent = list_agent.clone();
    let mut options = MemorySynthesisOptions::new(
        Arc::new(move || vec![(list_agent.clone(), Arc::clone(&list_target))]),
        Arc::new(move |agent_id| (agent_id == get_agent).then(|| Arc::clone(&get_target))),
        Arc::new(propose),
    );
    options.now = Arc::new(|| NOW_MS);
    options.retry_attempts = 1;
    let reports = Arc::new(Mutex::new(Vec::new()));
    let reported = Arc::clone(&reports);
    options.report = Some(Arc::new(move |report| reported.lock().unwrap().push(report)));
    (MemorySynthesisService::new(options), reports)
}

#[test]
fn production_synthesis_read_failure_preserves_temporal_bytes_and_recovers() {
    for destination_kind in [MemoryKind::Profile, MemoryKind::Log] {
        let fixture = Fixture::new("read-refusal", destination_kind);
        let admission = fixture.workers.read_session_work_admission(&fixture.target.agent_id).unwrap();
        let destination = fixture.destination();
        let repaired = format!("{}\n<!-- Retained destination comment -->\n",
            fs::read_to_string(&destination).unwrap());
        let mut corrupt = repaired.as_bytes().to_vec();
        corrupt.extend_from_slice(&[0xff, 0xfe]);
        fs::write(&destination, &corrupt).unwrap();
        assert_eq!(fs::read(&destination).unwrap(), corrupt);
        assert_eq!(fs::read_to_string(&destination).unwrap_err().kind(), std::io::ErrorKind::InvalidData);
        let before = memory_tree(&fixture.memory_dir());
        let snapshot = fixture.target.prepare_synthesis();
        let source = snapshot.memories.iter().find(|record| record.content == SOURCE_FACT).unwrap();
        assert_ne!(source.kind, destination_kind);
        assert_eq!(source.origin, MemoryOrigin::Synthesis);
        assert!(!snapshot.memories.iter().any(|record| record.content == DESTINATION_HISTORY));
        let changes = [SynthesisChange::Update {
            id: source.id.clone(), content: UPDATED_FACT.into(), kind: destination_kind,
        }];
        assert_eq!(fixture.target.apply_synthesis(&snapshot, &changes, NOW_MS),
            SynthesisApplyResult::Failed, "the actual Session gate must retain Store I/O failure");
        assert_eq!(memory_tree(&fixture.memory_dir()), before);

        let requests = Arc::new(Mutex::new(Vec::new()));
        let proposed = Arc::clone(&requests);
        let (service, reports) = service_for(&fixture, move |request, _| {
            let source = request.current_memories.iter()
                .find(|record| record.content == SOURCE_FACT).expect("readable synthesized source");
            assert_eq!(source.origin, MemoryOrigin::Synthesis);
            let evidence_id = if request.new_evidence.is_empty() {
                assert!(!request.current_memories.iter().any(|record| record.content == DESTINATION_HISTORY));
                "clock"
            } else {
                assert_eq!(request.new_evidence.len(), 1);
                assert!(request.current_memories.iter().any(|record| record.content == DESTINATION_HISTORY),
                    "repair retry must use a fresh snapshot containing restored history");
                request.new_evidence[0].id.as_str()
            };
            let output = update_proposal(&source.id, destination_kind, evidence_id);
            proposed.lock().unwrap().push(request);
            Ok(output)
        });
        // The due marker and corrupt file precede start, so this is an actual
        // temporal attempt. No background poller or provider account is started.
        service.start();
        assert_eq!(service.run_now(), vec![SynthesisOutcome::Failed]);
        assert_eq!(memory_tree(&fixture.memory_dir()), before,
            "temporal apply refusal must preserve all memory, origin, tombstone and review bytes");
        assert!(fixture.target.is_temporal_review_due(NOW_MS));
        assert_eq!(reports.lock().unwrap().as_slice(), &[SynthesisReport {
            outcome: SynthesisReportOutcome::Failed,
            agent_id: Some(fixture.target.agent_id.clone()),
            evidence_count: 0, input_memory_count: snapshot.memories.len(), change_count: 1, duration_ms: 0,
        }]);
        assert!(service.run_now().is_empty(), "failed apply finishes the attempt without implicit replay");
        assert_eq!(requests.lock().unwrap().len(), 1);
        assert_eq!(reports.lock().unwrap().len(), 1);
        assert_eq!(memory_tree(&fixture.memory_dir()), before);
        assert_eq!(fixture.workers.read_session_work_admission(&fixture.target.agent_id).unwrap(), admission);

        fs::write(&destination, &repaired).expect("fixture-owned repair");
        assert_ne!(fixture.target.prepare_synthesis().fingerprint, snapshot.fingerprint);
        service.record_turn(&fixture.target.agent_id, Some("repair-evidence".into()),
            "The destination was repaired; retain the updated fact.", "Acknowledged.", NOW_MS);
        assert_eq!(service.run_now(), vec![SynthesisOutcome::Committed]);
        assert!(fs::read_to_string(&destination).unwrap().starts_with(repaired.as_str()));
        fixture.assert_committed_history();
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].new_evidence.is_empty());
        assert_eq!(requests[1].new_evidence[0].id, "repair-evidence");
        drop(requests);
        let reports = reports.lock().unwrap();
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[1].outcome, SynthesisReportOutcome::Committed);
        assert_eq!(reports[1].evidence_count, 1);
        assert_eq!(reports[1].change_count, 1);
        drop(reports);
        assert!(service.run_now().is_empty());
        service.dispose();
        fixture.close();
    }
}

#[test]
fn production_synthesis_preserves_invalid_and_stale_temporal_controls() {
    let fixture = Fixture::new("invalid-control", MemoryKind::Profile);
    let before = memory_tree(&fixture.memory_dir());
    let snapshot = fixture.target.prepare_synthesis();
    let raw = update_proposal("unknown-snapshot-id", fixture.destination_kind, "clock");
    let parsed = parse_memory_synthesis_changes(&raw).expect("valid parsed update");
    assert!(uses_known_evidence(&HashSet::new(), &parsed, true));
    assert!(protects_explicit_memories(&snapshot, &parsed));
    assert!(!snapshot.memories.iter().any(|record| record.id == "unknown-snapshot-id"));
    let changes = parsed.iter().map(|change| change.to_store_change()).collect::<Vec<_>>();
    assert_eq!(fixture.target.apply_synthesis(&snapshot, &changes, NOW_MS), SynthesisApplyResult::Invalid);
    assert_eq!(memory_tree(&fixture.memory_dir()), before);
    let proposals = Arc::new(AtomicUsize::new(0));
    let proposed = Arc::clone(&proposals);
    let (service, reports) = service_for(&fixture, move |request, _| {
        assert!(request.new_evidence.is_empty());
        proposed.fetch_add(1, Ordering::SeqCst);
        Ok(raw.clone())
    });
    service.start();
    assert_eq!(service.run_now(), vec![SynthesisOutcome::InvalidOutput]);
    let mut expected = before;
    expected.insert(PathBuf::from(REFRESH_PATH), Some(reviewed_bytes()));
    assert_eq!(memory_tree(&fixture.memory_dir()), expected,
        "ordinary Invalid retains the existing temporal marker policy and changes nothing else");
    assert!(!fixture.target.is_temporal_review_due(NOW_MS));
    assert_eq!(reports.lock().unwrap().as_slice(), &[SynthesisReport {
        outcome: SynthesisReportOutcome::InvalidOutput,
        agent_id: Some(fixture.target.agent_id.clone()),
        evidence_count: 0, input_memory_count: snapshot.memories.len(), change_count: 1, duration_ms: 0,
    }]);
    assert!(service.run_now().is_empty());
    assert_eq!(proposals.load(Ordering::SeqCst), 1);
    service.dispose();
    fixture.close();

    let fixture = Fixture::new("stale-control", MemoryKind::Log);
    let before = memory_tree(&fixture.memory_dir());
    let snapshot = fixture.target.prepare_synthesis();
    let destination = fixture.destination();
    let concurrent = format!("{}\n<!-- Concurrent readable edit during proposal -->\n",
        fs::read_to_string(&destination).unwrap());
    let mut expected = before;
    expected.insert(destination.strip_prefix(fixture.memory_dir()).unwrap().to_path_buf(),
        Some(concurrent.as_bytes().to_vec()));
    let proposals = Arc::new(AtomicUsize::new(0));
    let proposed = Arc::clone(&proposals);
    let edited_path = destination.clone();
    let edited_bytes = concurrent.clone();
    let destination_kind = fixture.destination_kind;
    let (service, reports) = service_for(&fixture, move |request, _| {
        assert!(request.new_evidence.is_empty(), "Stale retains the original temporal queue item");
        let source = request.current_memories.iter()
            .find(|record| record.content == SOURCE_FACT).expect("current source fact");
        if proposed.fetch_add(1, Ordering::SeqCst) == 0 {
            fs::write(&edited_path, &edited_bytes).expect("fixture mutation after snapshot acquisition");
        }
        Ok(update_proposal(&source.id, destination_kind, "clock"))
    });
    service.start();
    assert_eq!(service.run_now(), vec![SynthesisOutcome::Stale]);
    assert_eq!(memory_tree(&fixture.memory_dir()), expected,
        "Stale adds no writes beyond the fixture's concurrent edit, including no review marker");
    assert!(fixture.target.is_temporal_review_due(NOW_MS));
    assert_ne!(fixture.target.prepare_synthesis().fingerprint, snapshot.fingerprint);
    assert_eq!(reports.lock().unwrap().as_slice(), &[SynthesisReport {
        outcome: SynthesisReportOutcome::Stale,
        agent_id: Some(fixture.target.agent_id.clone()),
        evidence_count: 0, input_memory_count: snapshot.memories.len(), change_count: 1, duration_ms: 0,
    }]);
    assert_eq!(service.run_now(), vec![SynthesisOutcome::Committed],
        "the retained Stale attempt must take a fresh snapshot on the next run");
    assert!(fs::read_to_string(&destination).unwrap().starts_with(concurrent.as_str()));
    fixture.assert_committed_history();
    assert_eq!(proposals.load(Ordering::SeqCst), 2);
    assert_eq!(reports.lock().unwrap()[1].outcome, SynthesisReportOutcome::Committed);
    assert!(service.run_now().is_empty());
    service.dispose();
    fixture.close();
}
