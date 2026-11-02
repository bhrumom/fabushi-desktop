use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::extensions::session::agent_db::{
    AgentDbProjectionError, SandAgentDb, SessionWorkAdmissionRecord,
};
use super::ProductionSessionWorkers;
use crate::extensions::transcript::transcript_entry_ids::{
    next_entry_id, TranscriptEntryIdKind,
};
use crate::storage::store_db::get_sand_agent_db_write_generation;
use crate::transcript_mutation_events::{
    TranscriptMutationSubscription, subscribe_transcript_mutations,
};
use rusqlite::params;
use serde_json::{Value, json};

struct Fixture {
    root: PathBuf,
    session: Arc<ProductionSessionWorkers>,
    owner: Arc<SandAgentDb>,
    agent: String,
    admission: SessionWorkAdmissionRecord,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!(
            "fabushi-fresh-append-{label}-{}-{suffix}", std::process::id(),
        ));
        let session = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 50));
        let agent = session.materialize_new_session(None, "user", None).unwrap().id;
        session.ensure_main_session_work_admission(&agent).unwrap();
        let admission = session.read_session_work_admission(&agent).unwrap().unwrap();
        let owner = session.open_agent_db_owner(&agent).unwrap();
        Self { root, session, owner, agent, admission }
    }

    fn draft(&self, content: &str) -> Value {
        json!({
            "kind": "message", "role": "user", "content": content,
            "sessionKey": self.admission.session_key, "timestampMs": 100,
        })
    }

    fn append(&self, drafts: &[Value]) -> Result<Vec<Value>, AgentDbProjectionError> {
        self.session.commit_agent_transcript_entries(
            &self.agent, Some(&self.admission.session_key),
            Some(&self.admission.lifecycle_revision), |snapshot| {
                let entries = plan_messages(snapshot, drafts);
                Ok((entries.clone(), entries))
            },
        )
    }

    fn raw_db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.owner.db_path()).unwrap()
    }

    fn durable_rows(&self) -> Vec<(i64, String, String)> {
        let db = self.raw_db();
        let mut statement = db.prepare("SELECT seq, id, entry FROM transcript_entries ORDER BY seq").unwrap();
        let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        rows
    }

    fn generation(&self) -> u64 {
        get_sand_agent_db_write_generation(self.owner.db_path())
    }

    fn assert_refused_without_append(&self, drafts: &[Value], events: &Publications) {
        let before = (self.durable_rows(), self.generation(), events.values());
        assert!(self.append(drafts).is_err());
        assert_eq!((self.durable_rows(), self.generation(), events.values()), before);
    }
}

// A pure producer plan using the existing allocator. All identity, kind,
// provenance, transaction and publication checks remain in the real owner.
// This deliberately does not recreate the removed public draft validator.
fn plan_messages(snapshot: &[Value], drafts: &[Value]) -> Vec<Value> {
    let mut existing = snapshot.to_vec();
    let mut entries = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let mut entry = draft.clone();
        entry["id"] = json!(next_entry_id(&existing, TranscriptEntryIdKind::UserMessage));
        existing.push(entry.clone());
        entries.push(entry);
    }
    entries
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // A lock-regression timeout must fail the test, not hang its cleanup.
        if !std::thread::panicking() {
            self.session.shutdown();
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

struct Subscription(Option<TranscriptMutationSubscription>);

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(subscription) = self.0.take() {
            subscription.unsubscribe();
        }
    }
}

struct Publications {
    observed: Arc<Mutex<Vec<Value>>>,
    _subscription: Subscription,
}

impl Publications {
    fn new(agent: &str) -> Self {
        let observed = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&observed);
        let agent = agent.to_owned();
        let subscription = subscribe_transcript_mutations(move |mutation| {
            if mutation.get("agentId").and_then(Value::as_str) == Some(agent.as_str()) {
                captured.lock().unwrap().push(Value::Object(mutation.clone()));
            }
        });
        Self { observed, _subscription: Subscription(Some(subscription)) }
    }

    fn values(&self) -> Vec<Value> {
        self.observed.lock().unwrap().clone()
    }
}

#[test]
fn fresh_user_message_batch_preserves_frozen_ids_and_publishes_one_committed_batch() {
    let fixture = Fixture::new("batch");
    let events = Publications::new(&fixture.agent);
    let before = fixture.generation();
    let drafts = [fixture.draft("first"), fixture.draft("second")];
    let committed = fixture.append(&drafts).unwrap();
    let mut expected = drafts.to_vec();
    expected[0]["id"] = json!("t0u");
    expected[1]["id"] = json!("t1u");
    assert_eq!(committed, expected);
    assert_eq!(fixture.session.read_agent_transcript_entries(&fixture.agent).unwrap(), expected);
    assert_eq!(fixture.generation(), before + 1);
    assert_eq!(events.values(), vec![json!({
        "kind": "entries-upserted", "agentId": fixture.agent, "entries": expected,
    })]);

    let unchanged = (fixture.durable_rows(), fixture.generation(), events.values());
    assert!(fixture.append(&[]).unwrap().is_empty());
    assert_eq!((fixture.durable_rows(), fixture.generation(), events.values()), unchanged);
}

#[test]
fn fresh_allocation_reserves_physical_ids_hidden_from_presentation_reads() {
    let fixture = Fixture::new("opaque-id");
    let opaque = json!({"id": "t0u", "kind": "future-transcript-kind", "payload": "retain"});
    fixture.raw_db().execute(
        "INSERT INTO transcript_entries(id, entry) VALUES (?1, ?2)",
        params!["t0u", opaque.to_string()],
    ).unwrap();
    assert!(fixture.session.read_agent_transcript_entries(&fixture.agent).unwrap().is_empty());

    let entries = fixture.append(&[fixture.draft("one"), fixture.draft("two")]).unwrap();
    assert_eq!(entries.iter().map(|entry| entry["id"].as_str().unwrap()).collect::<Vec<_>>(),
        vec!["t1u", "t2u"]);
    let physical = fixture.durable_rows();
    assert_eq!(physical.len(), 3);
    assert_eq!(serde_json::from_str::<Value>(&physical[0].2).unwrap(), opaque);
    assert_eq!(serde_json::from_str::<Value>(&physical[1].2).unwrap(), entries[0]);
    assert_eq!(serde_json::from_str::<Value>(&physical[2].2).unwrap(), entries[1]);
}

#[test]
fn second_insert_ignore_or_abort_rolls_back_the_entire_fresh_batch() {
    for action in ["IGNORE", "ABORT, 'second fresh row rejected'"] {
        let fixture = Fixture::new("second-insert");
        let events = Publications::new(&fixture.agent);
        fixture.raw_db().execute_batch(&format!(
            "CREATE TRIGGER reject_second_fresh BEFORE INSERT ON transcript_entries
             WHEN NEW.id = 't1u' BEGIN SELECT RAISE({action}); END;",
        )).unwrap();
        fixture.assert_refused_without_append(
            &[fixture.draft("must also roll back"), fixture.draft("rejected")], &events,
        );
        assert!(fixture.durable_rows().is_empty());
        assert!(events.values().is_empty());

        fixture.raw_db().execute_batch("DROP TRIGGER reject_second_fresh").unwrap();
        let committed = fixture.append(&[fixture.draft("explicit later attempt")]).unwrap();
        assert_eq!(committed[0]["id"], "t0u");
        assert_eq!(fixture.durable_rows().len(), 1);
        assert_eq!(events.values().len(), 1);
    }
}

#[test]
fn invalid_fresh_plan_identity_kind_and_provenance_cannot_partially_commit() {
    let fixture = Fixture::new("invalid-plan");
    let events = Publications::new(&fixture.agent);
    for fault in ["blank-id", "missing-id", "duplicate-id", "missing-kind", "blank-kind", "missing-key", "foreign-key"] {
        let mut first = fixture.draft("valid first");
        first["id"] = json!("t0u");
        let mut invalid = fixture.draft("invalid second");
        invalid["id"] = json!("t1u");
        match fault {
            "blank-id" => invalid["id"] = json!(" "),
            "missing-id" => { invalid.as_object_mut().unwrap().remove("id"); }
            "duplicate-id" => invalid["id"] = first["id"].clone(),
            "missing-kind" => { invalid.as_object_mut().unwrap().remove("kind"); }
            "blank-kind" => invalid["kind"] = json!(" "),
            "missing-key" => { invalid.as_object_mut().unwrap().remove("sessionKey"); }
            _ => invalid["sessionKey"] = json!("agent:other:main"),
        }
        let before = (fixture.durable_rows(), fixture.generation(), events.values());
        let result = fixture.session.commit_agent_transcript_entries(
            &fixture.agent, Some(&fixture.admission.session_key),
            Some(&fixture.admission.lifecycle_revision), |_| Ok(((), vec![first, invalid])),
        );
        assert!(matches!(result, Err(AgentDbProjectionError::TranscriptAppendInvalid(_))), "{fault}: {result:?}");
        assert_eq!((fixture.durable_rows(), fixture.generation(), events.values()), before);
    }
    assert!(fixture.durable_rows().is_empty());
    assert!(events.values().is_empty());
}

#[test]
fn malformed_physical_transcript_identity_is_not_an_empty_snapshot() {
    for raw in ["not-json", "null", "[]", r#"{"kind":"notice","text":"no id"}"#,
        r#"{"id":"different","kind":"notice","text":"mismatch"}"#] {
        let fixture = Fixture::new("malformed-row");
        let events = Publications::new(&fixture.agent);
        let db = fixture.raw_db();
        // Expression indexes otherwise reject invalid JSON before the fixture
        // can reach the strict owner's physical-row reader.
        db.execute_batch("DROP INDEX idx_transcript_window; DROP INDEX idx_transcript_branched;").unwrap();
        db.execute("INSERT INTO transcript_entries(id, entry) VALUES ('t0u', ?1)", [raw]).unwrap();
        fixture.assert_refused_without_append(&[fixture.draft("must not allocate")], &events);
        assert_eq!(fixture.durable_rows().len(), 1);
        assert!(events.values().is_empty());
    }
}

#[test]
fn canonical_admission_and_metadata_corruption_refuse_fresh_append() {
    for case in ["missing", "json", "foreign-agent", "foreign-key", "empty-generation", "metadata"] {
        let fixture = Fixture::new(case);
        let events = Publications::new(&fixture.agent);
        let db = fixture.raw_db();
        match case {
            "missing" => { db.execute("DELETE FROM kv WHERE key = 'sessionWorkAdmission'", []).unwrap(); }
            "json" => { db.execute("UPDATE kv SET value = 'not-json' WHERE key = 'sessionWorkAdmission'", []).unwrap(); }
            "metadata" => { db.execute("UPDATE kv SET value = 'not-hex' WHERE key = 'metadata'", []).unwrap(); }
            _ => {
                let mut changed = fixture.admission.clone();
                match case {
                    "foreign-agent" => changed.session_id = "other-agent".into(),
                    "foreign-key" => changed.session_key = "agent:other-agent:main".into(),
                    "empty-generation" => changed.lifecycle_revision.clear(),
                    _ => unreachable!(),
                }
                db.execute("UPDATE kv SET value = ?1 WHERE key = 'sessionWorkAdmission'",
                    [serde_json::to_string(&changed).unwrap()]).unwrap();
            }
        }
        fixture.assert_refused_without_append(&[fixture.draft("must be refused")], &events);
        assert!(events.values().is_empty());
    }
}

#[test]
fn archive_restore_and_durable_delete_fence_reject_old_admission() {
    let mut fixture = Fixture::new("lifecycle");
    let events = Publications::new(&fixture.agent);
    let old_revision = fixture.admission.lifecycle_revision.clone();
    let archived = fixture.session.transition_session_work_archive(
        &fixture.agent, &fixture.admission.session_key, &fixture.agent,
        Some(&old_revision), true, Some("user"), None, 100.0,
    ).unwrap();
    fixture.assert_refused_without_append(&[fixture.draft("archived")], &events);
    let restored = fixture.session.transition_session_work_archive(
        &fixture.agent, &fixture.admission.session_key, &fixture.agent,
        Some(&archived.lifecycle_revision), false, Some("user"), None, 200.0,
    ).unwrap();
    assert_ne!(restored.lifecycle_revision, old_revision);
    fixture.assert_refused_without_append(&[fixture.draft("stale after restore")], &events);
    fixture.admission = restored;
    assert_eq!(fixture.append(&[fixture.draft("current generation")]).unwrap()[0]["id"], "t0u");

    fixture.session.begin_agent_delete(&fixture.agent).unwrap();
    fixture.session.clear_agent_delete_fast_path(&fixture.agent);
    assert!(!fixture.session.is_agent_being_deleted(&fixture.agent));
    assert!(fixture.owner.read_session_work_admission().unwrap().unwrap().deletion_pending);
    fixture.assert_refused_without_append(&[fixture.draft("durably deleting")], &events);
}

#[test]
fn fresh_append_publication_synchronously_reenters_the_same_session_after_commit() {
    let fixture = Fixture::new("callback");
    let before = fixture.generation();
    let (observed_tx, observed_rx) = mpsc::channel();
    let session = Arc::clone(&fixture.session);
    let agent = fixture.agent.clone();
    let db_path = fixture.owner.db_path().to_path_buf();
    let _subscription = Subscription(Some(subscribe_transcript_mutations(move |mutation| {
        if mutation.get("agentId").and_then(Value::as_str) == Some(agent.as_str()) {
            let rows = session.read_agent_transcript_entries(&agent);
            let _ = observed_tx.send((
                std::thread::current().id(), rows, Value::Object(mutation.clone()),
                get_sand_agent_db_write_generation(&db_path),
            ));
        }
    })));
    let (done_tx, done_rx) = mpsc::channel();
    let session = Arc::clone(&fixture.session);
    let agent = fixture.agent.clone();
    let admission = fixture.admission.clone();
    let draft = fixture.draft("visible inside synchronous callback");
    let worker = std::thread::spawn(move || {
        let result = session.commit_agent_transcript_entries(
            &agent, Some(&admission.session_key), Some(&admission.lifecycle_revision),
            |snapshot| {
                let entries = plan_messages(snapshot, &[draft]);
                Ok((entries.clone(), entries))
            },
        );
        let _ = done_tx.send((std::thread::current().id(), result));
    });

    let (callback_thread, rows, event, generation) = observed_rx.recv_timeout(Duration::from_secs(5))
        .expect("synchronous listener must read the same Session without a held owner lock");
    let (append_thread, result) = done_rx.recv_timeout(Duration::from_secs(5))
        .expect("append returns after its synchronous listener");
    worker.join().unwrap();
    let committed = result.unwrap();
    assert_eq!(callback_thread, append_thread);
    assert_eq!(rows.unwrap(), committed);
    assert_eq!(event["entries"], json!(committed));
    assert_eq!(generation, before + 1);
    assert!(observed_rx.try_recv().is_err(), "one batch publishes exactly once");
}
