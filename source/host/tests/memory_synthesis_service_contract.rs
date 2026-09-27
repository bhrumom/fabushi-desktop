use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mahayana_host_runtime::extensions::memory::memory_service::{FileMemoryStore, MemoryKind};
use mahayana_host_runtime::extensions::memory::memory_synthesis_service::{
    MemoryEvidence, MemorySynthesisOptions, MemorySynthesisService, SynthesisOutcome,
    bounded_evidence_text, parse_memory_synthesis_changes, uses_known_evidence,
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
    options.verify = Some(Arc::new(|_request, _cancel| Ok(true)));
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
    options.verify = Some(Arc::new(|_request, _cancel| Ok(false)));
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
