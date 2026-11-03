use super::super as shipping;
use shipping::{
    BackgroundWakes, ChannelInboundAppendAttempt, ChannelInboundAppendFailure,
    ChannelInboundAppendFailureKind, ChannelInboundAppendOutcome, InboundAppendAction,
    InboundAppendWork, ProductionSessionWorkers, RoutedProvider, SettingsService,
    SettingsSubscription, TranscriptManager,
};
use mahayana_host_runtime::extensions::transcript::background_wakes::InboundAppendState;
use serde_json::{Value, json};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

type ChangeSubscription = SettingsSubscription<dyn Fn(Vec<String>) + Send + Sync + 'static>;

#[derive(Debug)]
struct DispatchObservation {
    provider: RoutedProvider,
    envelopes: Vec<Value>,
    attempt: ChannelInboundAppendAttempt,
    rows: Vec<Value>,
    queue_unlocked: bool,
}

struct ReadinessFixture {
    root: PathBuf,
    agent_id: String,
    sessions: Arc<ProductionSessionWorkers>,
    manager: Arc<TranscriptManager>,
    settings: Arc<SettingsService>,
    wakes: Arc<Mutex<BackgroundWakes<Value>>>,
    readiness_calls: AtomicUsize,
    launches: AtomicUsize,
    gate_checks: AtomicUsize,
    appends: AtomicUsize,
    observations: Mutex<Vec<DispatchObservation>>,
}

impl ReadinessFixture {
    fn new() -> Arc<Self> {
        let root = std::env::temp_dir().join(format!("fabushi-channel-readiness-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let agent_id = sessions.materialize_new_session(None, "user", None).unwrap().id;
        let manager = Arc::new(TranscriptManager::new(&root, Arc::clone(&sessions)));
        let wakes = manager.background_wakes();
        let settings = Arc::new(SettingsService::new(root.join("settings.json")));
        Arc::new(Self {
            root, agent_id, sessions, manager, settings, wakes,
            readiness_calls: AtomicUsize::new(0), launches: AtomicUsize::new(0),
            gate_checks: AtomicUsize::new(0), appends: AtomicUsize::new(0),
            observations: Mutex::new(Vec::new()),
        })
    }

    fn enqueue(&self, text: &str) -> Option<InboundAppendWork<Value>> {
        self.wakes.lock().unwrap().enqueue_inbound_and_claim(&self.agent_id, json!({
            "text":text, "address":"telegram:room", "timestampMs":7,
            "images":[{"data":"retained-image", "mimeType":"image/png"}],
        })).unwrap()
    }

    fn subscribe(self: &Arc<Self>) -> ChangeSubscription {
        let fixture = Arc::downgrade(self);
        shipping::bind_channel_inbound_settings(&self.settings, move || {
            if let Some(fixture) = fixture.upgrade() {
                fixture.request_readiness();
            }
        })
    }

    fn request_readiness(&self) {
        self.readiness_calls.fetch_add(1, Ordering::SeqCst);
        shipping::request_channel_inbound_readiness(&self.wakes, |agent_id, work| {
            assert_eq!(agent_id, self.agent_id);
            shipping::launch_channel_inbound_work(&self.wakes, &agent_id, work, |work| {
                self.launches.fetch_add(1, Ordering::SeqCst);
                self.run_ticket(work);
                Ok(())
            })
        });
    }

    fn gate(&self) -> Option<RoutedProvider> {
        self.gate_checks.fetch_add(1, Ordering::SeqCst);
        shipping::channel_inbound_ready_provider(
            &self.sessions, &self.manager.transcript_runtime(),
            self.settings.get_settings_path(), &self.agent_id,
        ).unwrap()
    }

    fn process(&self, work: &InboundAppendWork<Value>) -> InboundAppendAction<Value> {
        if !self.wakes.lock().unwrap().owns_inbound_append(&self.agent_id, work) {
            return InboundAppendAction::Stale;
        }
        let Some(provider) = self.gate() else {
            return self.wakes.lock().unwrap().pause_inbound_append(&self.agent_id, work);
        };
        shipping::handoff_channel_inbound_append(
            &self.wakes, &self.agent_id, work,
            |attempt| {
                self.appends.fetch_add(1, Ordering::SeqCst);
                shipping::append_channel_inbound_entries(&self.sessions, &self.agent_id, &work.envelopes, attempt)
            },
            |envelopes, attempt| {
                self.observations.lock().unwrap().push(DispatchObservation {
                    provider, envelopes: envelopes.to_vec(), attempt: attempt.clone(),
                    rows: self.rows(), queue_unlocked: self.wakes.try_lock().is_ok(),
                });
                None
            },
        )
    }

    fn run_ticket(&self, work: InboundAppendWork<Value>) {
        let mut first = Some(work);
        shipping::drain_channel_revival_batches(
            || first.take().or_else(|| self.wakes.lock().unwrap().take_inbound_append_or_end_revival(&self.agent_id)),
            |work| shipping::drive_channel_inbound_work(work, |work| self.process(work)),
            || {},
        );
    }

    fn rows(&self) -> Vec<Value> {
        self.sessions.read_agent_transcript_entries(&self.agent_id).unwrap()
    }

    fn connection(&self) -> rusqlite::Connection {
        let owner = self.sessions.open_agent_db_owner(&self.agent_id).unwrap();
        rusqlite::Connection::open(owner.db_path()).unwrap()
    }
}

impl Drop for ReadinessFixture {
    fn drop(&mut self) {
        self.manager.dispose();
        self.sessions.shutdown();
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn settings_commit_rearms_retained_input_without_new_ingress() {
    for update in [
        json!({"inferenceProvider":"fabushi"}),
        json!({"inference_provider":"fabushi"}),
        json!({"router":{"provider":"fabushi"}}),
    ] {
        let fixture = ReadinessFixture::new();
        let work = fixture.enqueue("retained before configuration").unwrap();
        let original = work.envelopes.clone();
        fixture.run_ticket(work);
        assert_eq!(fixture.appends.load(Ordering::SeqCst), 0);
        assert!(fixture.rows().is_empty());
        assert_eq!(fixture.wakes.lock().unwrap().inbound_append_state(&fixture.agent_id), Some(InboundAppendState::NotWritten));
        let _subscription = fixture.subscribe();
        if update.get("inferenceProvider").is_none() {
            // The aliases are existing read compatibility, not new Settings
            // writers. Seed that legacy source, then exercise the real owner's
            // commit notification while it preserves the stored value.
            let mut stored = update.clone();
            stored["version"] = json!(1);
            fs::write(fixture.settings.get_settings_path(), stored.to_string()).unwrap();
        }
        fixture.settings.set_host_settings(&update).unwrap();
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.appends.load(Ordering::SeqCst), 1);
        let observations = fixture.observations.lock().unwrap();
        assert_eq!(observations.len(), 1);
        let observed = &observations[0];
        assert_eq!(observed.provider, RoutedProvider::Fabushi);
        assert_eq!(observed.envelopes, original);
        assert!(observed.queue_unlocked);
        assert_eq!(observed.rows.len(), 1);
        assert_eq!(observed.attempt.entries.as_ref().unwrap(), &observed.rows);
        assert_eq!(observed.rows[0]["content"], "retained before configuration");
        assert_eq!(observed.rows[0]["timestampMs"], 7);
        assert_eq!(observed.rows[0]["sessionKey"], observed.attempt.session_key);
        let images = shipping::collect_inbound_images(&observed.envelopes);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].data, "retained-image");
        assert_eq!(images[0].mime_type, "image/png");
        let admission = fixture.sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
        assert_eq!(observed.attempt.lifecycle_revision, admission.lifecycle_revision);
        drop(observations);
        assert!(!fixture.wakes.lock().unwrap().reviving_inbound_agent_ids.contains(&fixture.agent_id));
        fixture.settings.set_host_settings(&json!({"webauthnProxyEnabled":false})).unwrap();
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.rows().len(), 1);
    }
}

#[test]
fn rejected_settings_cas_and_unrelated_fields_do_not_rearm() {
    let fixture = ReadinessFixture::new();
    fixture.run_ticket(fixture.enqueue("wait for valid settings").unwrap());
    let initial = fixture.settings.get_host_settings_snapshot();
    let _subscription = fixture.subscribe();
    fixture.settings.set_host_settings(&json!({"webauthnProxyEnabled":false})).unwrap();
    assert!(fixture.settings.set_host_settings_if_current(
        Some(&initial.hash), json!({"inferenceProvider":"fabushi"}),
    ).is_err());
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.appends.load(Ordering::SeqCst), 0);
    assert!(fixture.rows().is_empty());
    assert!(fixture.gate().is_none());
    fixture.settings.set_inference_provider("fabushi").unwrap();
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.rows().len(), 1);
}

#[cfg(unix)]
#[test]
fn failed_settings_persistence_does_not_publish_readiness() {
    let fixture = ReadinessFixture::new();
    fixture.run_ticket(fixture.enqueue("wait after IO failure").unwrap());
    // A valid existing filename fits NAME_MAX; the actual owner's appended
    // PID/nonce staging name exceeds it. This fails real persistence after a
    // successful checked read, without relying on effective-user permissions.
    let path = fixture.root.join("s".repeat(244));
    let original = b"{\"version\":1,\"pinnedAgentIds\":[\"read-before-write\"]}";
    fs::write(&path, original).unwrap();
    let settings = SettingsService::new(&path);
    let weak = Arc::downgrade(&fixture);
    let _subscription = shipping::bind_channel_inbound_settings(&settings, move || {
        weak.upgrade().unwrap().request_readiness();
    });
    assert_eq!(settings.get_host_settings_snapshot().settings["pinnedAgentIds"], json!(["read-before-write"]));
    assert!(settings.set_host_settings(&json!({"inferenceProvider":"fabushi"})).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
    assert!(fixture.rows().is_empty());
}

#[test]
fn committed_settings_signal_during_gate_check_survives_pause_once() {
    let fixture = ReadinessFixture::new();
    let work = fixture.enqueue("signal before pause").unwrap();
    let initial_token = work.worker_token.clone();
    let _subscription = fixture.subscribe();
    let mut checks = 0;
    assert!(shipping::drive_channel_inbound_work(work, |work| {
        checks += 1;
        if checks == 1 {
            assert!(fixture.gate().is_none());
            fixture.settings.set_inference_provider("fabushi").unwrap();
            assert_eq!(fixture.launches.load(Ordering::SeqCst), 0, "the live claim coalesces the notification");
            fixture.wakes.lock().unwrap().pause_inbound_append(&fixture.agent_id, work)
        } else {
            assert_ne!(work.worker_token, initial_token);
            fixture.process(work)
        }
    }));
    assert_eq!(checks, 2);
    assert_eq!(fixture.appends.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.observations.lock().unwrap().len(), 1);
    assert_eq!(fixture.rows().len(), 1);
}

#[test]
fn notification_is_only_a_hint_and_does_not_invent_provider_configuration() {
    let fixture = ReadinessFixture::new();
    fixture.run_ticket(fixture.enqueue("a hint is not admission").unwrap());
    let _subscription = fixture.subscribe();
    fixture.settings.set_host_settings(&json!({"router":{"provider":"fabushi"}})).unwrap();
    // Existing Settings does not write this compatibility field. Its input-key
    // notification still asks for a check, which must read the actual file.
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.gate_checks.load(Ordering::SeqCst), 2);
    assert_eq!(fixture.appends.load(Ordering::SeqCst), 0);
    assert!(fixture.rows().is_empty());
    assert_eq!(fixture.wakes.lock().unwrap().inbound_append_state(&fixture.agent_id), Some(InboundAppendState::NotWritten));
    fixture.settings.set_inference_provider("fabushi").unwrap();
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 2);
    assert_eq!(fixture.appends.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.rows()[0]["content"], "a hint is not admission");
}

#[test]
fn active_hint_does_not_spin_while_upgrade_gate_remains_closed() {
    let fixture = ReadinessFixture::new();
    fixture.settings.set_inference_provider("fabushi").unwrap();
    fixture.manager.quiesce_for_upgrade().unwrap();
    let work = fixture.enqueue("still quiescing").unwrap();
    let _subscription = fixture.subscribe();
    let mut passes = 0;
    assert!(!shipping::drive_channel_inbound_work(work, |work| {
        passes += 1;
        if passes == 1 {
            assert!(fixture.gate().is_none());
            fixture.settings.set_host_settings(&json!({"inferenceProvider":"fabushi"})).unwrap();
            fixture.wakes.lock().unwrap().pause_inbound_append(&fixture.agent_id, work)
        } else {
            fixture.process(work)
        }
    }));
    assert_eq!(passes, 2, "one coalesced hint permits one fresh gate check");
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.appends.load(Ordering::SeqCst), 0);
    assert!(fixture.rows().is_empty());
    assert_eq!(fixture.wakes.lock().unwrap().inbound_append_state(&fixture.agent_id), Some(InboundAppendState::NotWritten));
    shipping::resume_channel_inbound_after_recreate(&fixture.manager, || fixture.request_readiness()).unwrap();
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.rows()[0]["content"], "still quiescing");
}

#[test]
fn rollback_signal_retries_latest_real_preparation_before_tail() {
    let fixture = ReadinessFixture::new();
    fixture.settings.set_inference_provider("fabushi").unwrap();
    let db = fixture.connection();
    db.execute_batch("CREATE TRIGGER readiness_reject_second BEFORE INSERT ON transcript_entries
        WHEN json_extract(NEW.entry, '$.content') = 'second'
        BEGIN SELECT RAISE(ABORT, 'reject second actual row'); END;").unwrap();
    let work = {
        let mut owner = fixture.wakes.lock().unwrap();
        for text in ["first", "second"] {
            BackgroundWakes::enqueue(&mut owner.pending_inbound, &fixture.agent_id,
                json!({"text":text,"address":"telegram:room","timestampMs":11}));
        }
        assert!(BackgroundWakes::<Value>::begin_revival(&mut owner.reviving_inbound_agent_ids, &fixture.agent_id));
        owner.take_inbound_append_or_end_revival(&fixture.agent_id).unwrap()
    };
    let _subscription = fixture.subscribe();
    let first_token = work.worker_token.clone();
    let first_preparation = std::cell::RefCell::new(None);
    let mut passes = 0;
    let mut dispatches = 0;
    assert!(shipping::drive_channel_inbound_work(work, |work| {
        passes += 1;
        shipping::handoff_channel_inbound_append(
            &fixture.wakes, &fixture.agent_id, work,
            |attempt| {
                if passes == 2 {
                    assert_ne!(work.worker_token, first_token);
                    assert_eq!(work.attempt.as_ref(), first_preparation.borrow().as_ref());
                    assert_eq!(attempt.as_ref(), first_preparation.borrow().as_ref());
                }
                let outcome = shipping::append_channel_inbound_entries(
                    &fixture.sessions, &fixture.agent_id, &work.envelopes, attempt,
                );
                if passes == 1 {
                    assert!(matches!(outcome, ChannelInboundAppendOutcome::NotWritten(_)));
                    assert!(fixture.rows().is_empty(), "the first real insert rolled back");
                    *first_preparation.borrow_mut() = attempt.clone();
                    assert_eq!(first_preparation.borrow().as_ref().unwrap().entries.as_ref().unwrap().len(), 2);
                    db.execute_batch("DROP TRIGGER readiness_reject_second").unwrap();
                    assert!(fixture.enqueue("later input").is_none());
                    fixture.settings.set_host_settings(&json!({"router":{"provider":"fabushi"}})).unwrap();
                    assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
                }
                outcome
            },
            |envelopes, attempt| {
                dispatches += 1;
                assert_eq!(Some(attempt), first_preparation.borrow().as_ref());
                assert_eq!(envelopes.len(), 2);
                assert_eq!(fixture.rows(), *attempt.entries.as_ref().unwrap());
                assert!(fixture.wakes.try_lock().is_ok());
                None
            },
        )
    }));
    assert_eq!((passes, dispatches), (2, 1));
    let prepared = first_preparation.into_inner().unwrap();
    assert_eq!(fixture.rows(), prepared.entries.unwrap());
    assert_eq!(fixture.wakes.lock().unwrap().pending_inbound[&fixture.agent_id][0]["text"], "later input");
    let tail = fixture.wakes.lock().unwrap().take_inbound_append_or_end_revival(&fixture.agent_id).unwrap();
    fixture.run_ticket(tail);
    assert_eq!(fixture.rows().len(), 3);
    assert_eq!(fixture.rows()[2]["content"], "later input");
}

#[test]
fn unknown_commit_rejects_settings_resume_and_later_ingress() {
    let fixture = ReadinessFixture::new();
    fixture.settings.set_inference_provider("fabushi").unwrap();
    let work = fixture.enqueue("durable but acknowledgement lost").unwrap();
    assert!(matches!(shipping::handoff_channel_inbound_append(
        &fixture.wakes, &fixture.agent_id, &work,
        |attempt| {
            let result = shipping::append_channel_inbound_entries(
                &fixture.sessions, &fixture.agent_id, &work.envelopes, attempt,
            );
            assert_eq!(result, ChannelInboundAppendOutcome::Committed { inserted_count: 1 });
            ChannelInboundAppendOutcome::Unknown(ChannelInboundAppendFailure {
                kind: ChannelInboundAppendFailureKind::Storage, detail:"owned acknowledgement lost".into(),
            })
        },
        |_, _| panic!("unknown acknowledgement cannot authorize Runner"),
    ), InboundAppendAction::Held));
    let committed = fixture.rows();
    let _subscription = fixture.subscribe();
    fixture.settings.set_host_settings(&json!({"router":{"provider":"fabushi"}})).unwrap();
    fixture.manager.quiesce_for_upgrade().unwrap();
    shipping::resume_channel_inbound_after_recreate(&fixture.manager, || fixture.request_readiness()).unwrap();
    assert!(fixture.enqueue("tail must not overtake").is_none());
    fixture.settings.set_host_settings(&json!({"inference_provider":"fabushi"})).unwrap();
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.appends.load(Ordering::SeqCst), 0);
    assert!(fixture.observations.lock().unwrap().is_empty());
    assert_eq!(fixture.rows(), committed);
    let owner = fixture.wakes.lock().unwrap();
    assert_eq!(owner.inbound_append_state(&fixture.agent_id), Some(InboundAppendState::Unknown));
    assert_eq!(owner.pending_inbound[&fixture.agent_id].len(), 1);
    assert!(!owner.reviving_inbound_agent_ids.contains(&fixture.agent_id));
}

#[test]
fn resume_rearms_before_later_pending_wake_restoration_failure() {
    let fixture = ReadinessFixture::new();
    fixture.settings.set_inference_provider("fabushi").unwrap();
    fixture.manager.quiesce_for_upgrade().unwrap();
    assert!(fixture.manager.is_quiescing_for_upgrade());
    fixture.run_ticket(fixture.enqueue("retained during upgrade").unwrap());
    assert_eq!(fixture.appends.load(Ordering::SeqCst), 0);
    let result = (|| -> Result<usize, String> {
        shipping::resume_channel_inbound_after_recreate(&fixture.manager, || fixture.request_readiness())?;
        // This real manager has no pending-wake runtime bound; the subsequent
        // owner restore fails, as it can after readiness in the shipping call.
        fixture.manager.restore_recreate_pending_wakes(&[])
    })();
    assert!(result.unwrap_err().contains("not configured"));
    assert!(!fixture.manager.is_quiescing_for_upgrade());
    assert!(!fixture.manager.runner_registry().is_quiescing_for_upgrade());
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.appends.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.rows()[0]["content"], "retained during upgrade");
}

#[test]
fn failed_launch_keeps_exact_head_and_stale_error_cannot_release_successor() {
    let fixture = ReadinessFixture::new();
    let original = fixture.enqueue("retained after failed spawn").unwrap();
    assert!(shipping::launch_channel_inbound_work(&fixture.wakes, &fixture.agent_id, original.clone(), |_| {
        assert!(fixture.wakes.try_lock().is_ok());
        Err(io::Error::other("controlled owned launch failure"))
    }).is_err());
    assert_eq!(fixture.wakes.lock().unwrap().inbound_append_state(&fixture.agent_id), Some(InboundAppendState::NotWritten));
    let successor = fixture.wakes.lock().unwrap().request_inbound_readiness().pop().unwrap().1;
    assert_eq!(successor.claim_id, original.claim_id);
    assert_ne!(successor.worker_token, original.worker_token);
    assert_eq!(successor.envelopes, original.envelopes);
    let mut replacement = None;
    assert!(shipping::launch_channel_inbound_work(&fixture.wakes, &fixture.agent_id, successor, |work| {
        let mut owner = fixture.wakes.lock().unwrap();
        assert!(matches!(owner.pause_inbound_append(&fixture.agent_id, &work), InboundAppendAction::Held));
        replacement = owner.request_inbound_readiness().pop().map(|(_, work)| work);
        Err(io::Error::other("old launch error after successor acquired"))
    }).is_err());
    let replacement = replacement.unwrap();
    assert!(fixture.wakes.lock().unwrap().owns_inbound_append(&fixture.agent_id, &replacement));
    assert!(!shipping::launch_channel_inbound_work(&fixture.wakes, &fixture.agent_id, original, |_| {
        panic!("stale ticket cannot spawn")
    }).unwrap());
    assert!(fixture.wakes.lock().unwrap().owns_inbound_append(&fixture.agent_id, &replacement));
    fixture.settings.set_inference_provider("fabushi").unwrap();
    fixture.run_ticket(replacement);
    assert_eq!(fixture.rows().len(), 1);
    assert_eq!(fixture.observations.lock().unwrap().len(), 1);
}

struct ReleaseOnDrop(Option<mpsc::Sender<()>>);
impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

#[test]
fn dispose_refuses_snapshotted_settings_callback_and_reserved_ticket() {
    let fixture = ReadinessFixture::new();
    let ticket = fixture.enqueue("reserved before close").unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let blocker = fixture.settings.subscribe_to_changes(Arc::new(move |keys| {
        if keys.iter().any(|key| key == "inferenceProvider") {
            entered_tx.send(()).unwrap();
            release_rx.lock().unwrap().recv_timeout(Duration::from_secs(10)).unwrap();
        }
    }));
    let subscription = fixture.subscribe();
    let settings = Arc::clone(&fixture.settings);
    let writer = std::thread::spawn(move || settings.set_inference_provider("fabushi"));
    let release = ReleaseOnDrop(Some(release_tx));
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    // The real Settings drainer has already cloned both listeners in ID order.
    fixture.manager.dispose();
    drop(subscription);
    assert!(!shipping::launch_channel_inbound_work(&fixture.wakes, &fixture.agent_id, ticket, |_| {
        panic!("closed owner cannot launch a reserved ticket")
    }).unwrap());
    drop(release);
    writer.join().unwrap().unwrap();
    drop(blocker);
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1, "unsubscription cannot retract the captured callback");
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.appends.load(Ordering::SeqCst), 0);
    let mut owner = fixture.wakes.lock().unwrap();
    assert!(owner.is_closed());
    assert!(owner.pending_inbound.is_empty());
    assert!(owner.request_inbound_readiness().is_empty());
    assert!(owner.enqueue_inbound_and_claim(&fixture.agent_id, json!({"text":"late input"})).is_err());
}

#[test]
fn disposal_after_real_append_prevents_late_handoff_without_replaying_rows() {
    let fixture = ReadinessFixture::new();
    let db = fixture.connection();
    let work = fixture.enqueue("real row before close").unwrap();
    assert!(matches!(shipping::handoff_channel_inbound_append(
        &fixture.wakes, &fixture.agent_id, &work,
        |attempt| {
            let result = shipping::append_channel_inbound_entries(
                &fixture.sessions, &fixture.agent_id, &work.envelopes, attempt,
            );
            assert_eq!(result, ChannelInboundAppendOutcome::Committed { inserted_count: 1 });
            fixture.manager.dispose();
            result
        },
        |_, _| panic!("a closed queue cannot authorize a late dispatch"),
    ), InboundAppendAction::Stale));
    let raw: String = db.query_row("SELECT entry FROM transcript_entries", [], |row| row.get(0)).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&raw).unwrap()["content"], "real row before close");
    assert_eq!(db.query_row("SELECT COUNT(*) FROM transcript_entries", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    assert!(fixture.wakes.lock().unwrap().request_inbound_readiness().is_empty());
    assert!(fixture.wakes.lock().unwrap().is_closed());
}
