//! Live-custody oracles use the shipping adapters and real Settings, Session,
//! SQLite, TranscriptManager, roster and pending-marker owners. The last
//! follow-up call is controlled; invocation counts are not service acceptance.
//! Checked-settlement cases additionally consume an actual Runner terminal,
//! generated Session row and durable background-result receipt.

use super::super as shipping;
use super::channel_host_lane_tests::{IdleRunnerOwners, idle_runner_owners};
use mahayana_host_runtime::extensions::transcript::background_wakes::{
    ChannelFailureAction, ChannelFailureState, ChannelFailureWork, InboundAppendAction,
    InboundAppendWork,
};
use mahayana_host_runtime::extensions::transcript::sand_pending_wake_store::{
    AutomationQuietOrigin, DurablePendingWakeMarker, QuietWakeOrigin, SandPendingWakeStore,
};
use serde_json::{Value, json};
use shipping::{
    BackgroundWakes, ChannelFailureSettlementPoint, ChannelInboundAppendOutcome, GatewayEventHub, LocalRoutedRunnerDeps,
    ProductionRosterEmit, ProductionSessionWorkers, RevivalExecution, RoutedProvider,
    SettingsService, SettingsSubscription, TranscriptManager,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

type ChangeSubscription = SettingsSubscription<dyn Fn(Vec<String>) + Send + Sync + 'static>;

#[derive(Debug)]
struct FollowupObservation {
    provider: RoutedProvider,
    failures: Vec<Value>,
    rows: Vec<Value>,
}

#[derive(Debug)]
struct RosterObservation {
    event: Value,
    phase: Option<ChannelFailureState>,
    markers: Vec<DurablePendingWakeMarker>,
}

struct FailureReadinessFixture {
    root: PathBuf,
    agent_id: String,
    session_key: String,
    owners: Option<IdleRunnerOwners>,
    roster: Arc<ProductionRosterEmit>,
    wakes: Arc<Mutex<BackgroundWakes<Value>>>,
    initial_rows: Vec<Value>,
    readiness_calls: AtomicUsize,
    launches: AtomicUsize,
    passes: Mutex<Vec<ChannelFailureWork<Value>>>,
    followups: Mutex<Vec<FollowupObservation>>,
    receipts: Mutex<Vec<Value>>,
    inbound_tickets: Mutex<Vec<(String, InboundAppendWork<Value>)>>,
    roster_events: Arc<Mutex<Vec<RosterObservation>>>,
}

impl FailureReadinessFixture {
    fn new() -> Arc<Self> {
        let root = std::env::temp_dir().join(format!(
            "fabushi-channel-failure-readiness-{}", uuid::Uuid::new_v4(),
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("settings.json"), br#"{"version":1}"#).unwrap();
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let agent_id = sessions.materialize_new_session(None, "user", None).unwrap().id;
        let session_key = sessions.read_session_work_admission(&agent_id).unwrap().unwrap().session_key;
        let mut attempt = None;
        assert_eq!(shipping::append_channel_inbound_entries(
            &sessions, &agent_id,
            &[json!({"text":"durable history before failure", "address":"slack:room", "timestampMs":7})],
            &mut attempt,
        ), ChannelInboundAppendOutcome::Committed { inserted_count: 1 });
        let initial_rows = sessions.read_agent_transcript_entries(&agent_id).unwrap();
        let manager = Arc::new(TranscriptManager::new(&root, Arc::clone(&sessions)));
        let wakes = manager.background_wakes();
        let (host_tx, _host_rx) = mpsc::channel();
        let owners = idle_runner_owners(
            &root, host_tx, GatewayEventHub::default(), Arc::clone(&sessions), Arc::clone(&manager),
        );
        assert_eq!(owners.deps.settings.get_config_source_metadata()["valid"], json!(true));
        assert!(shipping::configured_routed_provider(owners.deps.settings.get_settings_path()).is_none(),
            "a valid Settings source remains unconfigured until the actual commit");
        let roster_events = Arc::new(Mutex::new(Vec::new()));
        let observed_events = Arc::clone(&roster_events);
        let observed_wakes = Arc::clone(&wakes);
        let observed_agent = agent_id.clone();
        let marker_store = manager.transcript_runtime().pending_wake_store().unwrap().clone();
        let roster = Arc::new(ProductionRosterEmit::new(
            sessions, manager.transcript_runtime(), Arc::new(move |event| {
                let phase = observed_wakes.try_lock().expect("roster callback must run outside queue lock")
                    .channel_failure_state(&observed_agent);
                observed_events.lock().unwrap().push(RosterObservation {
                    event, phase, markers: marker_store.list_pending_checked().unwrap(),
                });
            }),
        ));
        manager.bind_roster_emit(Arc::clone(&roster)).unwrap();
        Arc::new(Self {
            root, agent_id, session_key, owners: Some(owners), roster, wakes, initial_rows,
            readiness_calls: AtomicUsize::new(0), launches: AtomicUsize::new(0),
            passes: Mutex::new(Vec::new()), followups: Mutex::new(Vec::new()), receipts: Mutex::new(Vec::new()),
            inbound_tickets: Mutex::new(Vec::new()), roster_events,
        })
    }

    fn deps(&self) -> &LocalRoutedRunnerDeps {
        &self.owners.as_ref().unwrap().deps
    }

    fn settings(&self) -> &SettingsService {
        &self.deps().settings
    }

    fn manager(&self) -> &TranscriptManager {
        &self.deps().transcript_manager
    }

    fn store(&self) -> SandPendingWakeStore {
        self.deps().transcript_runtime.pending_wake_store().unwrap().clone()
    }

    fn markers(&self) -> Vec<DurablePendingWakeMarker> {
        let reopened = SandPendingWakeStore::new(&self.root).list_pending_checked().unwrap();
        assert_eq!(reopened, self.store().list_pending_checked().unwrap(), "reopened durable marker identity");
        reopened
    }

    fn failure(&self, operation: &str, marked_at_ms: f64) -> Value {
        self.failure_for_session(&self.agent_id, &self.session_key, operation, marked_at_ms)
    }

    fn failure_for_session(&self, agent_id: &str, session_key: &str, operation: &str, marked_at_ms: f64) -> Value {
        let legacy = json!({
            "addressToken":"slack:room", "deliveryOperationId":operation,
            "markedAtMs":marked_at_ms, "sessionKey":session_key,
            "reason":format!("delivery refused for {operation}"),
        });
        let marker = shipping::channel_failure_marker_from_failure(agent_id, &legacy).unwrap();
        shipping::channel_failure_from_marker(&marker).unwrap()
    }

    fn publish_marker(&self, agent_id: &str, failure: &Value) {
        let marker = shipping::channel_failure_marker_from_failure(agent_id, failure).unwrap();
        self.store().mark_pending_checked(marker).unwrap();
    }

    fn enqueue(&self, failure: &Value) -> Option<ChannelFailureWork<Value>> {
        self.publish_marker(&self.agent_id, failure);
        shipping::enqueue_channel_failure(
            &mut self.wakes.lock().unwrap(), &self.agent_id, failure.clone(),
        ).unwrap()
    }

    fn subscribe(self: &Arc<Self>) -> ChangeSubscription {
        let fixture = Arc::downgrade(self);
        shipping::bind_channel_inbound_settings(self.settings(), move || {
            if let Some(fixture) = fixture.upgrade() {
                fixture.request_readiness();
            }
        })
    }

    fn request_readiness(&self) {
        self.request_readiness_using(false);
    }

    fn subscribe_receipts(self: &Arc<Self>) -> ChangeSubscription {
        let fixture = Arc::downgrade(self);
        shipping::bind_channel_inbound_settings(self.settings(), move || {
            if let Some(fixture) = fixture.upgrade() {
                fixture.request_readiness_using(true);
            }
        })
    }

    fn request_readiness_using(&self, real_receipt: bool) {
        self.readiness_calls.fetch_add(1, Ordering::SeqCst);
        shipping::request_channel_inbound_readiness(&self.wakes, |agent_id, work| {
            shipping::launch_channel_inbound_work(&self.wakes, &agent_id, work, |work| {
                self.inbound_tickets.lock().unwrap().push((agent_id.clone(), work));
                Ok(())
            })
        });
        shipping::request_channel_failure_readiness(&self.wakes, |agent_id, work| {
            assert_eq!(agent_id, self.agent_id);
            shipping::launch_channel_failure_work(&self.wakes, &agent_id, work, |work| {
                assert!(self.wakes.try_lock().is_ok(), "launch is outside the queue mutex");
                self.launches.fetch_add(1, Ordering::SeqCst);
                self.run_ticket_using(work, real_receipt);
                Ok(())
            })
        });
    }

    fn observe_followup(&self, provider: RoutedProvider, failures: &[Value]) {
        let owner = self.wakes.try_lock().expect("follow-up must run outside queue lock");
        assert_eq!(owner.channel_failure_state(&self.agent_id), Some(ChannelFailureState::EnteredRunner));
        assert_eq!(owner.retained_channel_failures(&self.agent_id), Some(failures));
        drop(owner);
        let rows = self.deps().session_workers.read_agent_transcript_entries(&self.agent_id).unwrap();
        assert_eq!(rows, self.initial_rows, "failure notice must not append the original input again");
        self.followups.lock().unwrap().push(FollowupObservation {
            provider, failures: failures.to_vec(), rows,
        });
    }

    fn process_with(
        &self,
        work: &ChannelFailureWork<Value>,
        run: impl FnOnce(RoutedProvider, &[Value], (String, String)) -> Result<RevivalExecution, String>,
    ) -> ChannelFailureAction<Value> {
        self.passes.lock().unwrap().push(work.clone());
        shipping::process_channel_failure_work(
            self.deps(), self.roster.as_ref(), &self.wakes, &self.agent_id, work, run,
        )
    }

    fn process(&self, work: &ChannelFailureWork<Value>) -> ChannelFailureAction<Value> {
        self.process_with(work, |provider, failures, _| {
            self.observe_followup(provider, failures);
            Ok(successful_notice())
        })
    }

    fn run_ticket(&self, work: ChannelFailureWork<Value>) {
        self.run_ticket_using(work, false);
    }

    fn run_ticket_using(&self, work: ChannelFailureWork<Value>, real_receipt: bool) {
        let mut first = Some(work);
        shipping::drain_channel_revival_batches(
            || first.take().or_else(|| self.wakes.lock().unwrap()
                .take_channel_failure_work_or_end_revival(&self.agent_id)),
            |work| shipping::drive_channel_failure_work(work, |work| {
                if real_receipt {
                    self.process_with(work, |provider, failures, generation| {
                        self.observe_followup(provider, failures);
                        self.real_followup(generation, true, false)
                    })
                } else {
                    self.process(work)
                }
            }),
            || {},
        );
    }

    fn full_failure(&self, operation: &str, marked_at_ms: f64) -> Value {
        let mut marker = shipping::channel_failure_marker_from_failure(
            &self.agent_id, &self.failure(operation, marked_at_ms),
        ).unwrap();
        marker.title = Some("Delivery-failure notice with complete provenance".into());
        marker.quiet_origin = Some(QuietWakeOrigin {
            automation: Some(AutomationQuietOrigin {
                id: "checked-failure-origin".into(), name: "Origin before follow-up".into(),
            }),
        });
        marker.subagent_type = Some("delivery-observer".into());
        marker.interrupted_by_recreate = true;
        shipping::channel_failure_from_marker(&marker).unwrap()
    }

    fn reserve_batch(&self, failures: &[Value]) -> ChannelFailureWork<Value> {
        for failure in failures {
            self.publish_marker(&self.agent_id, failure);
        }
        let mut owner = self.wakes.lock().unwrap();
        for failure in failures {
            BackgroundWakes::enqueue(&mut owner.pending_channel_failures, &self.agent_id, failure.clone());
        }
        assert!(BackgroundWakes::<Value>::begin_revival(
            &mut owner.reviving_channel_failure_agent_ids, &self.agent_id,
        ));
        owner.take_channel_failure_work_or_end_revival(&self.agent_id).unwrap()
    }

    /// Supply a controlled final provider port, but use the real Session
    /// admission, generated append, terminal event and shipping result consumer.
    /// This is internal settlement evidence, not external service acceptance.
    fn real_followup(
        &self,
        expected_generation: (String, String),
        send: bool,
        cancelled: bool,
    ) -> Result<RevivalExecution, String> {
        let deps = self.deps();
        let sessions = deps.session_workers.as_ref();
        let current = sessions.read_session_work_admission(&self.agent_id)?
            .ok_or_else(|| "checked fixture Session admission is missing".to_string())?;
        let stream = format!("cfl-checked-stream-{}", uuid::Uuid::new_v4());
        let nonce = format!("cfl-checked-result-{}", uuid::Uuid::new_v4());
        let revision = shipping::capture_background_revival_admission(
            sessions, &self.agent_id, &current.session_key, &stream, Some(&expected_generation),
        )?;
        assert_eq!(sessions.read_session_work_admission(&self.agent_id)?.unwrap()
            .admitted_work_id.as_deref(), Some(stream.as_str()));
        let before = sessions.read_agent_transcript_entries(&self.agent_id)?;
        let admission_args = shipping::background_revival_admission_args(
            &self.agent_id, &current.session_key, "connector", "controlled checked follow-up",
            &nonce, &stream, None,
        );
        deps.transcript_runtime.accept_routed_send(&admission_args, |_| {
            Ok::<_, shipping::ProductionSendError>(shipping::PersistedSendContext::default())
        }).map_err(|error| error.to_string())?;
        deps.transcript_runtime.begin_provider_run_with_kind(&self.agent_id, false);
        let cancellation = deps.runner_registry.register_routed_provider(&self.agent_id, &stream)
            .map_err(|error| error.to_string())?;
        let mut lease = shipping::RoutedTurnLeaseGuard::new(
            Arc::clone(&deps.transcript_runtime), self.agent_id.clone(), stream.clone(), deps.events.clone(),
        );
        let receiver = deps.events.subscribe();
        if send {
            let entry = deps.transcript_runtime.append_generated_send_message_entry(
                sessions, &self.agent_id, &current.session_key,
                &json!({"type":"text", "content":"A checked delivery-failure follow-up."}),
                20_000, None, false,
            )?;
            assert_eq!(entry["kind"], "send-message");
            assert_eq!(entry["sessionKey"], current.session_key);
        }
        let after = sessions.read_agent_transcript_entries(&self.agent_id)?;
        let count = shipping::collect_new_member_send_messages(&before, &after).len();
        assert_eq!(count, usize::from(send), "count the actual generated Session append");
        if cancelled {
            assert!(deps.runner_registry.cancel_stream(&stream, "controlled checked cancellation"));
        }
        let terminal = shipping::RoutedProviderTerminal::capture(
            &stream, false, &cancellation, Ok("controlled checked follow-up completed".into()),
            count as u64, false, shipping::ToolEffectCustodyReport::new(&stream),
        );
        deps.transcript_runtime.end_provider_run_with_kind(&self.agent_id, false);
        deps.runner_registry.finish_routed_provider(&stream);
        lease.settle();
        terminal.finalize_observations(
            &Mutex::new(deps.telemetry_logs.start_turn(&self.agent_id, "connector", None)), None,
        );
        terminal.publish(&deps.events);
        let execution = shipping::wait_and_settle_background_revival(
            sessions, &receiver, &deps.runner_registry, &self.agent_id,
            &current.session_key, &revision, &stream, &nonce, &before,
            Instant::now() + Duration::from_secs(30), Instant::now, |_| {},
        )?;
        assert_eq!(execution.aborted, cancelled);
        assert_eq!(execution.sent_message_count, if cancelled { 0 } else { count });
        let persisted = sessions.open_agent_db_owner(&self.agent_id)?
            .get_transcript_entry_by_id(&format!("background-result:{nonce}"))
            .map_err(|error| error.to_string())?;
        assert!(sessions.read_session_work_admission(&self.agent_id)?.unwrap().admitted_work_id.is_none(),
            "the actual terminal consumer releases its admitted Session work");
        if cancelled {
            assert!(persisted.is_none(), "cancelled Runner release cannot manufacture a successful receipt");
        } else {
            let receipt = persisted.expect("the shipping terminal consumer commits the durable result receipt");
            assert_eq!(receipt["kind"], "background-result-receipt");
            assert_eq!(receipt["sessionKey"], current.session_key);
            assert_eq!(receipt["lifecycleRevision"], revision);
            assert_eq!(receipt["workId"], stream);
            assert_eq!(receipt["idempotencyKey"], nonce);
            assert_eq!(receipt["sentMessageCount"], json!(count));
            self.receipts.lock().unwrap().push(receipt);
        }
        Ok(execution)
    }

    fn assert_retained(&self, head: &[Value], tail: &[Value]) {
        let mut owner = self.wakes.lock().unwrap();
        assert_eq!(owner.channel_failure_state(&self.agent_id), Some(ChannelFailureState::RetainedAfterDispatch));
        assert_eq!(owner.retained_channel_failures(&self.agent_id), Some(head));
        assert_eq!(owner.pending_channel_failures.get(&self.agent_id).map(Vec::as_slice).unwrap_or(&[]), tail);
        assert!(owner.take_channel_failure_work_or_end_revival(&self.agent_id).is_none());
        assert!(!owner.reviving_channel_failure_agent_ids.contains(&self.agent_id));
    }

    fn rows_after_dispose(&self) -> Vec<Value> {
        // Disposal closes the original pool. Reopen only for observation; this
        // facade never admits work, resumes a manager or supplies execution.
        let observer = ProductionSessionWorkers::with_agents_root(self.root.join("agents"), 500);
        let rows = observer.read_agent_transcript_entries(&self.agent_id).unwrap();
        for receipt in self.receipts.lock().unwrap().iter() {
            assert_eq!(observer.open_agent_db_owner(&self.agent_id).unwrap()
                .get_transcript_entry_by_id(receipt["id"].as_str().unwrap()).unwrap().as_ref(), Some(receipt));
        }
        assert!(observer.read_session_work_admission(&self.agent_id).unwrap().unwrap().admitted_work_id.is_none());
        observer.shutdown();
        rows
    }

    fn assert_paused(&self, head: &[Value], tail: &[Value]) {
        let owner = self.wakes.lock().unwrap();
        assert_eq!(owner.channel_failure_state(&self.agent_id), Some(ChannelFailureState::PausedBeforeDispatch));
        assert_eq!(owner.retained_channel_failures(&self.agent_id), Some(head));
        assert_eq!(owner.pending_channel_failures.get(&self.agent_id).map(Vec::as_slice).unwrap_or(&[]), tail);
        assert!(!owner.reviving_channel_failure_agent_ids.contains(&self.agent_id));
    }
}

impl Drop for FailureReadinessFixture {
    fn drop(&mut self) {
        if let Some(owners) = self.owners.take() {
            owners.dispose(&self.agent_id);
            owners.deps.session_workers.shutdown();
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn successful_notice() -> RevivalExecution {
    // A controlled state/ordering result for the original readiness cases.
    // It is not a Session receipt oracle; checked cases use real_followup.
    RevivalExecution { aborted: false, quiesced_for_upgrade: false, sent_message_count: 1 }
}

fn supported_settings_signals(settings: &SettingsService) {
    for update in [
        json!({"inferenceProvider":"fabushi"}),
        json!({"inference_provider":"fabushi"}),
        json!({"router":{"provider":"fabushi"}}),
    ] {
        settings.set_host_settings(&update).unwrap();
    }
}

#[test]
fn channel_failure_settings_commit_rearms_paused_head_with_exact_marker() {
    for update in [
        json!({"inferenceProvider":"fabushi"}),
        json!({"inference_provider":"fabushi"}),
        json!({"router":{"provider":"fabushi"}}),
    ] {
        let fixture = FailureReadinessFixture::new();
        let failure = fixture.failure("retained-operation", 101.25);
        fixture.run_ticket(fixture.enqueue(&failure).unwrap());
        fixture.assert_paused(&[failure.clone()], &[]);
        let markers = fixture.markers();
        assert_eq!(markers, vec![shipping::channel_failure_marker_from_failure(&fixture.agent_id, &failure).unwrap()]);
        assert!(fixture.followups.lock().unwrap().is_empty());
        let _subscription = fixture.subscribe();
        if update.get("inferenceProvider").is_none() {
            // Existing compatibility keys are read from a real legacy file;
            // their actual Settings commit notification is still exercised.
            let mut stored = update.clone();
            stored["version"] = json!(1);
            fs::write(fixture.settings().get_settings_path(), stored.to_string()).unwrap();
        }
        fixture.settings().set_host_settings(&update).unwrap();
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
        let observations = fixture.followups.lock().unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].provider, RoutedProvider::Fabushi);
        assert_eq!(observations[0].failures, vec![failure]);
        assert_eq!(observations[0].rows, fixture.initial_rows);
        drop(observations);
        assert!(fixture.markers().is_empty());
        let events = fixture.roster_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].phase, Some(ChannelFailureState::EnteredRunner), "roster precedes matching head finish");
        assert_eq!(events[0].event["channel"], "agents");
        assert!(events[0].markers.is_empty(), "the exact marker settles before roster emission");
        drop(events);
        let owner = fixture.wakes.lock().unwrap();
        assert!(owner.retained_channel_failures(&fixture.agent_id).is_none());
        assert!(owner.pending_channel_failures.is_empty());
        assert!(!owner.reviving_channel_failure_agent_ids.contains(&fixture.agent_id));
        drop(owner);
        fixture.settings().set_host_settings(&json!({"webauthnProxyEnabled":false})).unwrap();
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.followups.lock().unwrap().len(), 1);
    }
}

#[test]
fn channel_failure_rejected_settings_do_not_launch() {
    let fixture = FailureReadinessFixture::new();
    let failure = fixture.failure("rejected-settings", 102.0);
    fixture.run_ticket(fixture.enqueue(&failure).unwrap());
    let markers = fixture.markers();
    let initial = fixture.settings().get_host_settings_snapshot();
    let _subscription = fixture.subscribe();
    fixture.settings().set_host_settings(&json!({"webauthnProxyEnabled":false})).unwrap();
    assert!(fixture.settings().set_host_settings_if_current(
        Some(&initial.hash), json!({"inferenceProvider":"fabushi"}),
    ).is_err());
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.markers(), markers);
    fixture.assert_paused(&[failure], &[]);
    fixture.settings().set_inference_provider("fabushi").unwrap();
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.followups.lock().unwrap().len(), 1);
    assert!(fixture.markers().is_empty());
}

#[cfg(unix)]
#[test]
fn channel_failure_failed_settings_persistence_does_not_launch() {
    let fixture = FailureReadinessFixture::new();
    let failure = fixture.failure("failed-settings-write", 103.0);
    fixture.run_ticket(fixture.enqueue(&failure).unwrap());
    let markers = fixture.markers();
    // Its checked input read succeeds; the real owner's longer staging name
    // exceeds NAME_MAX and fails without depending on effective-user rights.
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
    assert!(fixture.followups.lock().unwrap().is_empty());
    assert_eq!(fixture.markers(), markers);
    fixture.assert_paused(&[failure], &[]);
}

#[test]
fn channel_failure_active_signals_coalesce_and_resume_without_spin() {
    let fixture = FailureReadinessFixture::new();
    fixture.settings().set_inference_provider("fabushi").unwrap();
    fixture.manager().quiesce_for_upgrade().unwrap();
    let head = fixture.failure("quiesced-head", 104.0);
    let tail = fixture.failure("quiesced-tail", 105.0);
    let work = fixture.enqueue(&head).unwrap();
    let original_token = work.worker_token.clone();
    let original_claim = work.claim_id.clone();
    let _subscription = fixture.subscribe();
    supported_settings_signals(fixture.settings());
    assert!(fixture.enqueue(&tail).is_none());
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 0, "active signals coalesce on the real head");
    fixture.run_ticket(work);
    let passes = fixture.passes.lock().unwrap();
    assert_eq!(passes.len(), 2, "one coalesced recheck stops when the real upgrade gate remains closed");
    assert_eq!(passes[0].worker_token, original_token);
    assert_ne!(passes[1].worker_token, original_token);
    assert_eq!(passes[1].claim_id, original_claim);
    assert_eq!(passes[1].failures, vec![head.clone()]);
    drop(passes);
    fixture.assert_paused(&[head.clone()], &[tail.clone()]);
    assert_eq!(fixture.markers().len(), 2);
    assert!(fixture.followups.lock().unwrap().is_empty());

    let result = (|| -> Result<usize, String> {
        shipping::resume_channel_inbound_after_recreate(fixture.manager(), || fixture.request_readiness())?;
        fixture.manager().restore_recreate_pending_wakes(&[])
    })();
    assert!(result.unwrap_err().contains("not configured"), "readiness precedes later fallible restore");
    assert!(!fixture.manager().is_quiescing_for_upgrade());
    assert!(!fixture.manager().runner_registry().is_quiescing_for_upgrade());
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 4);
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    let observations = fixture.followups.lock().unwrap();
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].failures, vec![head]);
    assert_eq!(observations[1].failures, vec![tail]);
    drop(observations);
    assert!(fixture.markers().is_empty());
    assert!(fixture.wakes.lock().unwrap().channel_failure_state(&fixture.agent_id).is_none());
}

#[test]
fn channel_failure_pause_preserves_latest_filtered_head_before_tail() {
    let fixture = FailureReadinessFixture::new();
    let stale = fixture.failure_for_session(
        &fixture.agent_id, "agent:an-obsolete-session", "stale-provenance", 106.0,
    );
    let current = fixture.failure("current-provenance", 107.0);
    let tail = fixture.failure("later-provenance", 108.0);
    let work = {
        for failure in [&stale, &current] {
            fixture.publish_marker(&fixture.agent_id, failure);
        }
        let mut owner = fixture.wakes.lock().unwrap();
        for failure in [&stale, &current] {
            BackgroundWakes::enqueue(&mut owner.pending_channel_failures, &fixture.agent_id, failure.clone());
        }
        assert!(BackgroundWakes::<Value>::begin_revival(
            &mut owner.reviving_channel_failure_agent_ids, &fixture.agent_id,
        ));
        owner.take_channel_failure_work_or_end_revival(&fixture.agent_id).unwrap()
    };
    let original_claim = work.claim_id.clone();
    let original_token = work.worker_token.clone();
    let _subscription = fixture.subscribe();
    assert!(fixture.enqueue(&tail).is_none());
    // This committed hint does not write a provider. Both real gate passes
    // must read the missing configuration, including the retry after filtering.
    fixture.settings().set_host_settings(&json!({"router":{"provider":"fabushi"}})).unwrap();
    fixture.run_ticket(work);
    let passes = fixture.passes.lock().unwrap();
    assert_eq!(passes.len(), 2);
    assert_eq!(passes[0].failures, vec![stale, current.clone()]);
    assert_eq!(passes[1].claim_id, original_claim);
    assert_ne!(passes[1].worker_token, original_token);
    assert_eq!(passes[1].failures, vec![current.clone()], "the retry carries the newly filtered head");
    drop(passes);
    fixture.assert_paused(&[current.clone()], &[tail.clone()]);
    assert_eq!(fixture.markers(), vec![
        shipping::channel_failure_marker_from_failure(&fixture.agent_id, &current).unwrap(),
        shipping::channel_failure_marker_from_failure(&fixture.agent_id, &tail).unwrap(),
    ]);
    assert!(fixture.followups.lock().unwrap().is_empty());
    fixture.settings().set_inference_provider("fabushi").unwrap();
    let observations = fixture.followups.lock().unwrap();
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].failures, vec![current]);
    assert_eq!(observations[1].failures, vec![tail]);
    drop(observations);
    assert!(fixture.markers().is_empty());
}

#[test]
fn channel_failure_entered_nonsuccess_retains_under_all_later_signals() {
    for outcome in [
        Err("controlled provider error claiming nothing was sent".to_string()),
        Ok(RevivalExecution { aborted: true, quiesced_for_upgrade: false, sent_message_count: 1 }),
        Ok(RevivalExecution { aborted: false, quiesced_for_upgrade: true, sent_message_count: 1 }),
        Ok(RevivalExecution { aborted: false, quiesced_for_upgrade: false, sent_message_count: 0 }),
    ] {
        let fixture = FailureReadinessFixture::new();
        fixture.settings().set_inference_provider("fabushi").unwrap();
        let head = fixture.failure("entered-head", 109.0);
        let during = fixture.failure("tail-during-runner", 110.0);
        let later = fixture.failure("tail-after-runner", 111.0);
        let work = fixture.enqueue(&head).unwrap();
        let original_marker = fixture.markers()[0].clone();
        let _subscription = fixture.subscribe();
        assert!(matches!(fixture.process_with(&work, |provider, failures, _| {
            fixture.observe_followup(provider, failures);
            supported_settings_signals(fixture.settings());
            fixture.manager().quiesce_for_upgrade().unwrap();
            shipping::resume_channel_inbound_after_recreate(fixture.manager(), || fixture.request_readiness()).unwrap();
            assert!(fixture.enqueue(&during).is_none());
            assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
            assert_eq!(fixture.markers()[0], original_marker);
            assert_eq!(fixture.wakes.lock().unwrap().channel_failure_state(&fixture.agent_id), Some(ChannelFailureState::EnteredRunner));
            outcome.clone()
        }), ChannelFailureAction::Held));
        assert_eq!(fixture.wakes.lock().unwrap().channel_failure_state(&fixture.agent_id), Some(ChannelFailureState::RetainedAfterDispatch));

        supported_settings_signals(fixture.settings());
        fixture.manager().quiesce_for_upgrade().unwrap();
        shipping::resume_channel_inbound_after_recreate(fixture.manager(), || fixture.request_readiness()).unwrap();
        assert!(fixture.enqueue(&later).is_none());
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 8);
        assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
        assert!(matches!(fixture.process(&work), ChannelFailureAction::Stale));
        assert!(!shipping::launch_channel_failure_work(&fixture.wakes, &fixture.agent_id, work.clone(), |_| {
            panic!("a retained post-dispatch head cannot launch again")
        }).unwrap());
        let mut owner = fixture.wakes.lock().unwrap();
        assert_eq!(owner.retained_channel_failures(&fixture.agent_id), Some([head.clone()].as_slice()));
        assert_eq!(owner.pending_channel_failures.get(&fixture.agent_id), Some(&vec![during.clone(), later.clone()]));
        assert!(!owner.release_channel_failure_launch(&fixture.agent_id, &work));
        assert!(!owner.finish_channel_failure_work(&fixture.agent_id, &work));
        assert!(owner.take_channel_failure_work_or_end_revival(&fixture.agent_id).is_none());
        assert!(!owner.reviving_channel_failure_agent_ids.contains(&fixture.agent_id));
        drop(owner);
        assert_eq!(fixture.followups.lock().unwrap().len(), 1, "no signal may infer replay safety from the outcome");
        assert_eq!(fixture.markers(), vec![
            original_marker,
            shipping::channel_failure_marker_from_failure(&fixture.agent_id, &during).unwrap(),
            shipping::channel_failure_marker_from_failure(&fixture.agent_id, &later).unwrap(),
        ]);
        assert!(fixture.roster_events.lock().unwrap().is_empty());
        let trays = fixture.deps().trays.list();
        if outcome.is_err() {
            assert_eq!(trays.len(), 1);
            assert_eq!(trays[0].title, "Delivery-failure follow-up failed");
        } else {
            assert!(trays.is_empty());
        }
    }
}

#[test]
fn channel_failure_success_settles_exact_marker_before_next_head() {
    let fixture = FailureReadinessFixture::new();
    fixture.settings().set_inference_provider("fabushi").unwrap();
    let head = fixture.failure("finished-head", 112.0);
    let tail = fixture.failure("next-head", 113.0);
    let work = fixture.enqueue(&head).unwrap();
    assert!(fixture.enqueue(&tail).is_none());
    let mut replacement = shipping::channel_failure_marker_from_failure(&fixture.agent_id, &head).unwrap();
    replacement.marked_at_ms = 114.0;
    replacement.failure_reason = Some("a newer failure for the same durable work key".into());
    assert!(matches!(fixture.process_with(&work, |provider, failures, _| {
        fixture.observe_followup(provider, failures);
        assert!(fixture.store().mark_pending(replacement.clone()));
        Ok(successful_notice())
    }), ChannelFailureAction::Finished));
    let next = fixture.wakes.lock().unwrap().take_channel_failure_work_or_end_revival(&fixture.agent_id).unwrap();
    assert_ne!(next.claim_id, work.claim_id);
    assert_ne!(next.worker_token, work.worker_token);
    assert_eq!(next.failures, vec![tail.clone()]);
    assert!(!fixture.wakes.lock().unwrap().finish_channel_failure_work(&fixture.agent_id, &work));
    assert!(fixture.wakes.lock().unwrap().owns_channel_failure_work(&fixture.agent_id, &next));
    fixture.run_ticket(next);
    let observations = fixture.followups.lock().unwrap();
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].failures, vec![head]);
    assert_eq!(observations[1].failures, vec![tail]);
    drop(observations);
    assert_eq!(fixture.markers(), vec![replacement.clone()], "completion cannot clear a superseding marker");
    let events = fixture.roster_events.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| event.phase == Some(ChannelFailureState::EnteredRunner)));
    assert_eq!(events[1].markers, vec![replacement]);
    drop(events);
    let owner = fixture.wakes.lock().unwrap();
    assert!(owner.channel_failure_state(&fixture.agent_id).is_none());
    assert!(owner.pending_channel_failures.is_empty());
    assert!(!owner.reviving_channel_failure_agent_ids.contains(&fixture.agent_id));
}

#[test]
fn channel_failure_spawn_error_cannot_release_successor() {
    let fixture = FailureReadinessFixture::new();
    let failure = fixture.failure("launch-failure-head", 115.0);
    let original = fixture.enqueue(&failure).unwrap();
    let markers = fixture.markers();
    assert!(shipping::launch_channel_failure_work(&fixture.wakes, &fixture.agent_id, original.clone(), |_| {
        assert!(fixture.wakes.try_lock().is_ok());
        Err("controlled spawn error".into())
    }).is_err());
    fixture.assert_paused(&[failure.clone()], &[]);
    let successor = fixture.wakes.lock().unwrap().request_channel_failure_readiness().pop().unwrap().1;
    assert_eq!(successor.claim_id, original.claim_id);
    assert_ne!(successor.worker_token, original.worker_token);
    assert_eq!(successor.failures, original.failures);
    let mut replacement = None;
    assert!(shipping::launch_channel_failure_work(&fixture.wakes, &fixture.agent_id, successor, |work| {
        assert!(matches!(fixture.process(&work), ChannelFailureAction::Held));
        replacement = fixture.wakes.lock().unwrap().request_channel_failure_readiness().pop().map(|(_, work)| work);
        Err("old spawn error arriving after the next acquisition".into())
    }).is_err());
    let replacement = replacement.unwrap();
    assert_eq!(replacement.claim_id, original.claim_id);
    assert_eq!(replacement.failures, vec![failure]);
    assert!(fixture.wakes.lock().unwrap().owns_channel_failure_work(&fixture.agent_id, &replacement));
    assert!(!shipping::launch_channel_failure_work(&fixture.wakes, &fixture.agent_id, original, |_| {
        panic!("a retired launch ticket must not create a worker")
    }).unwrap());
    assert!(fixture.wakes.lock().unwrap().owns_channel_failure_work(&fixture.agent_id, &replacement));
    assert_eq!(fixture.markers(), markers);
    assert!(fixture.followups.lock().unwrap().is_empty());
    fixture.settings().set_inference_provider("fabushi").unwrap();
    fixture.run_ticket(replacement);
    assert_eq!(fixture.followups.lock().unwrap().len(), 1);
    assert!(fixture.markers().is_empty());
}

#[test]
fn channel_failure_committed_settings_survives_launch_failure() {
    let fixture = FailureReadinessFixture::new();
    let failure = fixture.failure("committed-settings-launch-error", 116.0);
    fixture.run_ticket(fixture.enqueue(&failure).unwrap());
    let markers = fixture.markers();
    let attempts = Arc::new(AtomicUsize::new(0));
    let observed_attempts = Arc::clone(&attempts);
    let weak = Arc::downgrade(&fixture);
    let subscription = shipping::bind_channel_inbound_settings(fixture.settings(), move || {
        let fixture = weak.upgrade().unwrap();
        shipping::request_channel_failure_readiness(&fixture.wakes, |agent_id, work| {
            shipping::launch_channel_failure_work(&fixture.wakes, &agent_id, work, |_| {
                observed_attempts.fetch_add(1, Ordering::SeqCst);
                Err("controlled failure to create a thread".into())
            })
        });
    });
    fixture.settings().set_inference_provider("fabushi").unwrap();
    assert_eq!(shipping::configured_routed_provider(fixture.settings().get_settings_path()), Some(RoutedProvider::Fabushi));
    assert_eq!(attempts.load(Ordering::SeqCst), 1, "launch failure does not recursively retry");
    fixture.assert_paused(&[failure], &[]);
    assert_eq!(fixture.markers(), markers);
    assert!(fixture.followups.lock().unwrap().is_empty());
    drop(subscription);
    let _subscription = fixture.subscribe();
    fixture.settings().set_host_settings(&json!({"router":{"provider":"fabushi"}})).unwrap();
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.followups.lock().unwrap().len(), 1);
    assert!(fixture.markers().is_empty());
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
fn channel_failure_dispose_refuses_snapshotted_callback_and_reserved_ticket() {
    let fixture = FailureReadinessFixture::new();
    let failure = fixture.failure("reserved-before-close", 117.0);
    let ticket = fixture.enqueue(&failure).unwrap();
    let markers = fixture.markers();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let blocker = fixture.settings().subscribe_to_changes(Arc::new(move |keys| {
        if keys.iter().any(|key| key == "inferenceProvider") {
            entered_tx.send(()).unwrap();
            release_rx.lock().unwrap().recv_timeout(Duration::from_secs(10)).unwrap();
        }
    }));
    let subscription = fixture.subscribe();
    let settings = Arc::clone(&fixture.deps().settings);
    let writer = std::thread::spawn(move || settings.set_inference_provider("fabushi"));
    let release = ReleaseOnDrop(Some(release_tx));
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    // The actual Settings drainer has snapshotted the later weak callback.
    fixture.manager().dispose();
    drop(subscription);
    assert!(!shipping::launch_channel_failure_work(&fixture.wakes, &fixture.agent_id, ticket.clone(), |_| {
        panic!("closed owner cannot launch a reserved ticket")
    }).unwrap());
    drop(release);
    writer.join().unwrap().unwrap();
    drop(blocker);
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1, "unsubscription cannot retract the snapshot");
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
    assert!(fixture.followups.lock().unwrap().is_empty());
    assert_eq!(fixture.markers(), markers, "manager disposal does not settle a durable failure");
    let mut old = fixture.wakes.lock().unwrap();
    assert!(old.is_closed());
    assert!(old.retained_channel_failures(&fixture.agent_id).is_none());
    assert!(old.pending_channel_failures.is_empty());
    assert!(old.request_channel_failure_readiness().is_empty());
    assert!(!old.release_channel_failure_launch(&fixture.agent_id, &ticket));
    assert!(!old.finish_channel_failure_work(&fixture.agent_id, &ticket));
    assert!(old.enqueue_channel_failure_and_claim(&fixture.agent_id, failure.clone()).is_err());
    drop(old);

    let fresh_manager = TranscriptManager::new(&fixture.root, Arc::clone(&fixture.deps().session_workers));
    let fresh = fresh_manager.background_wakes();
    assert!(!Arc::ptr_eq(&fresh, &fixture.wakes));
    let fresh_ticket = fresh.lock().unwrap().enqueue_channel_failure_and_claim(&fixture.agent_id, failure).unwrap().unwrap();
    fixture.request_readiness();
    assert!(!fixture.wakes.lock().unwrap().release_channel_failure_launch(&fixture.agent_id, &ticket));
    assert!(fresh.lock().unwrap().owns_channel_failure_work(&fixture.agent_id, &fresh_ticket));
    assert_ne!(fresh_ticket.claim_id, ticket.claim_id);
    assert_eq!(fresh.lock().unwrap().channel_failure_state(&fixture.agent_id), Some(ChannelFailureState::ActiveBeforeDispatch));
    fresh_manager.dispose();
}

#[test]
fn channel_failure_close_after_gate_blocks_late_admission_and_cleanup() {
    let fixture = FailureReadinessFixture::new();
    fixture.settings().set_inference_provider("fabushi").unwrap();
    let failure = fixture.failure("close-before-admission", 118.0);
    let ticket = fixture.enqueue(&failure).unwrap();
    let markers = fixture.markers();
    // Read the real Session/provenance/provider gates, then close before the
    // production handoff's final locked admission. No gate result is authority.
    let summary = fixture.deps().session_workers.summarize_agent_by_id(&fixture.agent_id, None).unwrap().unwrap();
    assert!(!summary.is_group);
    let mut filtered = ticket.failures.clone();
    shipping::filter_current_channel_failure_markers(fixture.deps(), &fixture.agent_id, &mut filtered).unwrap();
    let (generation, stale) = shipping::filter_current_channel_failure_provenance(
        fixture.deps(), &fixture.agent_id, &mut filtered,
    ).unwrap();
    let admission = fixture.deps().session_workers.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
    assert_eq!(generation, (admission.session_key, admission.lifecycle_revision));
    assert!(stale.is_empty());
    assert_eq!(filtered, vec![failure.clone()]);
    assert_eq!(shipping::configured_routed_provider(fixture.settings().get_settings_path()), Some(RoutedProvider::Fabushi));
    assert!(!fixture.deps().transcript_runtime.is_quiescing_for_upgrade());
    fixture.manager().dispose();
    assert!(matches!(shipping::handoff_channel_failure_followup(
        &fixture.wakes, &fixture.agent_id, &ticket, filtered,
        |_| panic!("close before final admission must make zero follow-up calls"),
        |_| panic!("rejected admission cannot settle markers"),
        |_| panic!("rejected admission has no provider outcome"),
    ), ChannelFailureAction::Stale));
    assert!(matches!(fixture.process(&ticket), ChannelFailureAction::Stale));
    let mut owner = fixture.wakes.lock().unwrap();
    assert!(matches!(owner.pause_channel_failure_before_dispatch(&fixture.agent_id, &ticket, ticket.failures.clone()), ChannelFailureAction::Stale));
    assert!(!owner.begin_channel_failure_followup(&fixture.agent_id, &ticket, ticket.failures.clone()));
    assert!(!owner.retain_channel_failure_after_dispatch(&fixture.agent_id, &ticket));
    assert!(!owner.release_channel_failure_launch(&fixture.agent_id, &ticket));
    assert!(!owner.finish_channel_failure_work(&fixture.agent_id, &ticket));
    assert!(owner.take_channel_failure_work_or_end_revival(&fixture.agent_id).is_none());
    assert!(owner.enqueue_channel_failure_and_claim(&fixture.agent_id, failure).is_err());
    assert!(owner.pending_channel_failures.is_empty());
    assert!(owner.reviving_channel_failure_agent_ids.is_empty());
    drop(owner);
    assert_eq!(fixture.markers(), markers);
    assert!(fixture.followups.lock().unwrap().is_empty());
    assert!(fixture.roster_events.lock().unwrap().is_empty());
}

#[test]
fn channel_failure_readiness_preserves_other_agent_and_inbound_custody() {
    let fixture = FailureReadinessFixture::new();
    let failure = fixture.failure("failure-family", 119.0);
    fixture.run_ticket(fixture.enqueue(&failure).unwrap());
    let inbound = fixture.wakes.lock().unwrap().enqueue_inbound_and_claim(
        &fixture.agent_id, json!({"text":"independent inbound head", "timestampMs":17}),
    ).unwrap().unwrap();
    assert!(matches!(fixture.wakes.lock().unwrap().pause_inbound_append(&fixture.agent_id, &inbound), InboundAppendAction::Held));
    let other = fixture.deps().session_workers.materialize_new_session(None, "user", None).unwrap().id;
    let other_key = fixture.deps().session_workers.read_session_work_admission(&other).unwrap().unwrap().session_key;
    let other_failure = fixture.failure_for_session(&other, &other_key, "other-agent-failure", 120.0);
    fixture.publish_marker(&other, &other_failure);
    let other_ticket = fixture.wakes.lock().unwrap().enqueue_channel_failure_and_claim(&other, other_failure.clone()).unwrap().unwrap();
    let _subscription = fixture.subscribe();
    fixture.settings().set_inference_provider("fabushi").unwrap();
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1, "one committed callback requests both wake families");
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.followups.lock().unwrap().len(), 1);
    let inbound_tickets = fixture.inbound_tickets.lock().unwrap();
    assert_eq!(inbound_tickets.len(), 1);
    assert_eq!(inbound_tickets[0].0, fixture.agent_id);
    assert_eq!(inbound_tickets[0].1.claim_id, inbound.claim_id);
    assert_ne!(inbound_tickets[0].1.worker_token, inbound.worker_token);
    let owner = fixture.wakes.lock().unwrap();
    assert!(owner.owns_inbound_append(&fixture.agent_id, &inbound_tickets[0].1));
    assert!(owner.owns_channel_failure_work(&other, &other_ticket));
    assert_eq!(owner.retained_channel_failures(&other), Some([other_failure.clone()].as_slice()));
    assert!(owner.reviving_channel_failure_agent_ids.contains(&other));
    assert!(owner.reviving_inbound_agent_ids.contains(&fixture.agent_id));
    assert!(owner.retained_channel_failures(&fixture.agent_id).is_none());
    drop(owner);
    drop(inbound_tickets);
    assert_eq!(fixture.markers(), vec![shipping::channel_failure_marker_from_failure(&other, &other_failure).unwrap()]);
}

#[test]
fn channel_failure_remote_room_preserves_existing_policy() {
    use mahayana_host_runtime::groups::remote_room_store::{
        REMOTE_ROOM_CONFIG_VERSION, SandRemoteRoomConfig, write_sand_remote_room_config,
    };
    let fixture = FailureReadinessFixture::new();
    fixture.settings().set_inference_provider("fabushi").unwrap();
    write_sand_remote_room_config(fixture.root.join("agents").join(&fixture.agent_id), &SandRemoteRoomConfig {
        version: REMOTE_ROOM_CONFIG_VERSION, room_id: "remote-room".into(),
        host_auth_id: "fixture-host".into(), host_name: "Fixture host".into(),
        host_avatar_url: None, members: Vec::new(), is_revoked: None,
    }).unwrap();
    let summary = fixture.deps().session_workers.summarize_agent_by_id(&fixture.agent_id, None).unwrap().unwrap();
    assert!(!summary.is_group);
    assert!(summary.remote_room.is_some());
    let failure = fixture.failure("remote-room-failure-policy", 121.0);
    fixture.run_ticket(fixture.enqueue(&failure).unwrap());
    assert_eq!(fixture.followups.lock().unwrap().len(), 1, "failure policy retains its existing remote-room behavior");
    assert_eq!(fixture.followups.lock().unwrap()[0].failures, vec![failure]);
    assert!(fixture.markers().is_empty());
}

#[test]
fn channel_failure_exact_recovery_deduplicates_retained_head_and_tail_only() {
    let fixture = FailureReadinessFixture::new();
    fixture.settings().set_inference_provider("fabushi").unwrap();
    let head = fixture.failure("recovered-after-dispatch", 122.0);
    let ticket = fixture.enqueue(&head).unwrap();
    assert!(matches!(fixture.process_with(&ticket, |provider, failures, _| {
        fixture.observe_followup(provider, failures);
        Ok(RevivalExecution { aborted: false, quiesced_for_upgrade: false, sent_message_count: 0 })
    }), ChannelFailureAction::Held));
    let other = fixture.deps().session_workers.materialize_new_session(None, "user", None).unwrap().id;
    let other_key = fixture.deps().session_workers.read_session_work_admission(&other).unwrap().unwrap().session_key;
    let other_failure = fixture.failure_for_session(&other, &other_key, "unrelated-paused-recovery", 123.0);
    fixture.publish_marker(&other, &other_failure);
    let other_ticket = shipping::enqueue_channel_failure(
        &mut fixture.wakes.lock().unwrap(), &other, other_failure.clone(),
    ).unwrap().unwrap();
    fixture.manager().quiesce_for_upgrade().unwrap();
    assert!(matches!(shipping::process_channel_failure_work(
        fixture.deps(), fixture.roster.as_ref(), &fixture.wakes, &other, &other_ticket,
        |_, _, _| panic!("the real quiescence gate must pause the unrelated agent"),
    ), ChannelFailureAction::Held));

    let tail = fixture.failure("recovered-raw-tail", 124.0);
    assert!(fixture.enqueue(&tail).is_none());
    let head_marker = shipping::channel_failure_marker_from_failure(&fixture.agent_id, &head).unwrap();
    let tail_marker = shipping::channel_failure_marker_from_failure(&fixture.agent_id, &tail).unwrap();
    let (completion_tx, completions) = mpsc::channel();
    let recover = |marker: &DurablePendingWakeMarker| {
        shipping::queue_recovered_channel_delivery_failure(
            fixture.deps().clone(), Arc::clone(&fixture.roster), Arc::clone(&fixture.wakes),
            marker, Some(completion_tx.clone()),
        )
    };
    let bytes = fs::read(fixture.store().file_path()).unwrap();
    for marker in [&head_marker, &head_marker, &tail_marker, &head_marker, &tail_marker] {
        recover(marker).unwrap();
    }
    assert_eq!(fs::read(fixture.store().file_path()).unwrap(), bytes,
        "exact recovery admission does not rewrite the current image");
    {
        let owner = fixture.wakes.lock().unwrap();
        assert_eq!(owner.channel_failure_state(&fixture.agent_id), Some(ChannelFailureState::RetainedAfterDispatch));
        assert_eq!(owner.retained_channel_failures(&fixture.agent_id), Some([head.clone()].as_slice()));
        assert_eq!(owner.pending_channel_failures.get(&fixture.agent_id), Some(&vec![tail.clone()]));
        assert_eq!(owner.channel_failure_state(&other), Some(ChannelFailureState::PausedBeforeDispatch),
            "duplicate recovery requests only its own agent, not every paused head");
        assert_eq!(owner.retained_channel_failures(&other), Some([other_failure.clone()].as_slice()));
        assert!(owner.reviving_channel_failure_agent_ids.is_empty());
    }

    let mut replacement = head_marker.clone();
    replacement.marked_at_ms += 3.0;
    replacement.title = Some("Different exact metadata for the same operation".into());
    fixture.store().mark_pending_checked(replacement.clone()).unwrap();
    let replacement_failure = shipping::channel_failure_from_marker(&replacement).unwrap();
    let replacement_bytes = fs::read(fixture.store().file_path()).unwrap();
    recover(&replacement).unwrap();
    recover(&replacement).unwrap();
    assert!(recover(&head_marker).is_err(), "a replaced durable snapshot cannot be recovered again");
    assert_eq!(fs::read(fixture.store().file_path()).unwrap(), replacement_bytes);
    let mut owner = fixture.wakes.lock().unwrap();
    assert_eq!(owner.channel_failure_state(&fixture.agent_id), Some(ChannelFailureState::RetainedAfterDispatch));
    assert_eq!(owner.retained_channel_failures(&fixture.agent_id), Some([head].as_slice()),
        "marker replacement cannot evict a head that already entered Runner");
    assert_eq!(owner.pending_channel_failures.get(&fixture.agent_id), Some(&vec![tail, replacement_failure]),
        "different marker metadata remains distinct ordered tail input");
    assert_eq!(owner.channel_failure_state(&other), Some(ChannelFailureState::PausedBeforeDispatch));
    assert_eq!(owner.retained_channel_failures(&other), Some([other_failure].as_slice()));
    assert!(!owner.release_channel_failure_launch(&fixture.agent_id, &ticket));
    assert!(!owner.finish_channel_failure_work(&fixture.agent_id, &ticket));
    assert!(owner.take_channel_failure_work_or_end_revival(&fixture.agent_id).is_none());
    assert!(owner.reviving_channel_failure_agent_ids.is_empty());
    drop(owner);
    assert!(matches!(completions.try_recv(), Err(mpsc::TryRecvError::Empty)),
        "duplicate recovery must not start a worker for entered work or the unrelated paused agent");
    assert_eq!(fixture.followups.lock().unwrap().len(), 1);
    assert!(fixture.roster_events.lock().unwrap().is_empty());
    assert_eq!(fixture.deps().session_workers.read_agent_transcript_entries(&fixture.agent_id).unwrap(), fixture.initial_rows);
}

#[test]
fn channel_failure_checked_receipt_settles_full_metadata_once() {
    let fixture = FailureReadinessFixture::new();
    let failure = fixture.full_failure("checked-full-metadata", 201.25);
    let work = fixture.enqueue(&failure).unwrap();
    fixture.run_ticket(work);
    fixture.assert_paused(&[failure.clone()], &[]);
    let exact_marker = shipping::channel_failure_marker_from_failure(&fixture.agent_id, &failure).unwrap();
    assert_eq!(fixture.markers(), vec![exact_marker]);
    let _subscription = fixture.subscribe_receipts();
    fixture.settings().set_inference_provider("fabushi").unwrap();
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1, "the real committed Settings callback requests readiness");
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    let observed = fixture.followups.lock().unwrap();
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].provider, RoutedProvider::Fabushi);
    assert_eq!(observed[0].failures, vec![failure]);
    assert_eq!(observed[0].rows, fixture.initial_rows);
    drop(observed);
    assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
    let rows = fixture.deps().session_workers.read_agent_transcript_entries(&fixture.agent_id).unwrap();
    assert_eq!(shipping::collect_new_member_send_messages(&fixture.initial_rows, &rows).len(), 1);
    for original in &fixture.initial_rows {
        assert_eq!(rows.iter().filter(|row| *row == original).count(), 1,
            "follow-up appends its generated notice without duplicating original channel input");
    }
    assert!(fixture.markers().is_empty());
    let events = fixture.roster_events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].phase, Some(ChannelFailureState::EnteredRunner));
    assert!(events[0].markers.is_empty(), "checked durable cleanup precedes the roster observation");
    drop(events);
    assert!(fixture.wakes.lock().unwrap().channel_failure_state(&fixture.agent_id).is_none());
    supported_settings_signals(fixture.settings());
    shipping::resume_channel_inbound_after_recreate(fixture.manager(), || fixture.request_readiness_using(true)).unwrap();
    assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 5, "three full commits and successful resume are real later signals");
    assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.followups.lock().unwrap().len(), 1);
    assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
    assert_eq!(fixture.deps().session_workers.read_agent_transcript_entries(&fixture.agent_id).unwrap(), rows);
}

#[test]
fn channel_failure_checked_receipt_cleanup_refusal_retains_head_and_tail() {
    for partial_batch in [false, true] {
        let fixture = FailureReadinessFixture::new();
        fixture.settings().set_inference_provider("fabushi").unwrap();
        let first = fixture.full_failure("checked-cleanup-first", 202.0);
        let second = fixture.full_failure("checked-cleanup-second", 203.0);
        let failures = if partial_batch { vec![first.clone(), second.clone()] } else { vec![first.clone()] };
        let work = fixture.reserve_batch(&failures);
        let store = fixture.store();
        let _subscription = fixture.subscribe_receipts();
        let mut repair_image = None;
        let mut points = Vec::new();
        let action = shipping::process_channel_failure_work_observed(
            fixture.deps(), fixture.roster.as_ref(), &fixture.wakes, &fixture.agent_id, &work,
            |provider, failures, generation| {
                fixture.observe_followup(provider, failures);
                fixture.real_followup(generation, true, false)
            },
            |point| {
                assert!(fixture.wakes.try_lock().is_ok(), "checked I/O is observed outside the wake mutex");
                points.push(point);
                let inject = if partial_batch {
                    matches!(point, ChannelFailureSettlementPoint::MarkerDisposed { index: 0, removed: true })
                } else {
                    point == ChannelFailureSettlementPoint::RunAdmitted
                };
                if inject {
                    assert_eq!(fixture.receipts.lock().unwrap().len(), 1,
                        "the actual Session result receipt precedes the filesystem refusal");
                    let expected = if partial_batch { &second } else { &first };
                    assert_eq!(store.list_pending_checked().unwrap(), vec![
                        shipping::channel_failure_marker_from_failure(&fixture.agent_id, expected).unwrap(),
                    ]);
                    repair_image = Some(fs::read(store.file_path()).unwrap());
                    fs::remove_file(store.file_path()).unwrap();
                    fs::create_dir(store.file_path()).unwrap();
                    fs::write(store.file_path().join("unrelated-witness"), b"preserve actual unreadable image").unwrap();
                }
            },
        );
        assert!(matches!(action, ChannelFailureAction::Held));
        fixture.assert_retained(&work.failures, &[]);
        assert_eq!(fixture.followups.lock().unwrap().len(), 1);
        assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
        assert!(fixture.roster_events.lock().unwrap().is_empty());
        assert!(store.list_pending_checked().is_err());
        assert_eq!(fs::read(store.file_path().join("unrelated-witness")).unwrap(), b"preserve actual unreadable image");
        assert_eq!(points, if partial_batch {
            vec![ChannelFailureSettlementPoint::RunAdmitted,
                ChannelFailureSettlementPoint::MarkerDisposed { index: 0, removed: true }]
        } else { vec![ChannelFailureSettlementPoint::RunAdmitted] });
        assert!(fixture.deps().trays.list().iter().any(|tray| tray.title == "Delivery-failure follow-up failed"));

        fs::remove_file(store.file_path().join("unrelated-witness")).unwrap();
        fs::remove_dir(store.file_path()).unwrap();
        fs::write(store.file_path(), repair_image.expect("fault followed actual admitted cleanup")).unwrap();
        let tail = fixture.full_failure("checked-cleanup-tail", 204.0);
        assert!(fixture.enqueue(&tail).is_none());
        let bytes = fs::read(store.file_path()).unwrap();
        let rows = fixture.deps().session_workers.read_agent_transcript_entries(&fixture.agent_id).unwrap();
        let (completion_tx, completions) = mpsc::channel();
        for _ in 0..2 {
            supported_settings_signals(fixture.settings());
            shipping::resume_channel_inbound_after_recreate(fixture.manager(), || fixture.request_readiness_using(true)).unwrap();
            for (index, failure) in work.failures.iter().enumerate() {
                let marker = shipping::channel_failure_marker_from_failure(&fixture.agent_id, failure).unwrap();
                let recovered = shipping::queue_recovered_channel_delivery_failure(
                    fixture.deps().clone(), Arc::clone(&fixture.roster), Arc::clone(&fixture.wakes),
                    &marker, Some(completion_tx.clone()),
                );
                if partial_batch && index == 0 {
                    assert!(recovered.is_err(), "a genuinely removed first identity cannot be resurrected");
                } else {
                    recovered.unwrap();
                }
            }
            let tail_marker = shipping::channel_failure_marker_from_failure(&fixture.agent_id, &tail).unwrap();
            shipping::queue_recovered_channel_delivery_failure(
                fixture.deps().clone(), Arc::clone(&fixture.roster), Arc::clone(&fixture.wakes),
                &tail_marker, Some(completion_tx.clone()),
            ).unwrap();
        }
        fixture.assert_retained(&work.failures, &[tail]);
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 8);
        assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
        assert_eq!(fixture.followups.lock().unwrap().len(), 1);
        assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
        assert_eq!(fixture.deps().session_workers.read_agent_transcript_entries(&fixture.agent_id).unwrap(), rows);
        assert_eq!(fs::read(store.file_path()).unwrap(), bytes);
        assert!(matches!(completions.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "retained entered work and its ordered tail start no recovery worker");
    }
}

#[test]
fn channel_failure_checked_generation_drift_refuses_session_admission() {
    for change_generation in [false, true] {
        let fixture = FailureReadinessFixture::new();
        fixture.settings().set_inference_provider("fabushi").unwrap();
        let failure = fixture.full_failure("checked-generation", 205.0);
        let work = fixture.enqueue(&failure).unwrap();
        let bytes = fs::read(fixture.store().file_path()).unwrap();
        let sessions = fixture.deps().session_workers.as_ref();
        let initial = sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
        let action = fixture.process_with(&work, |provider, failures, generation| {
            fixture.observe_followup(provider, failures);
            assert_eq!(generation, (initial.session_key.clone(), initial.lifecycle_revision.clone()),
                "the actual provenance filter supplies one same-read key and revision");
            if change_generation {
                let mut next = initial.clone();
                next.lifecycle_revision = format!("checked-new-generation-{}", uuid::Uuid::new_v4());
                assert!(sessions.compare_and_set_session_work_admission(
                    &fixture.agent_id, Some(&initial.lifecycle_revision), &next,
                ).unwrap(), "publish the new generation after preflight and before Session capture");
            }
            let execution = fixture.real_followup(generation, true, false);
            if change_generation {
                assert!(execution.as_ref().unwrap_err().contains("generation changed before Runner admission"));
            }
            execution
        });
        assert!(sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap().admitted_work_id.is_none());
        assert_eq!(fixture.followups.lock().unwrap().len(), 1);
        if change_generation {
            assert!(matches!(action, ChannelFailureAction::Held));
            fixture.assert_retained(&work.failures, &[]);
            assert!(fixture.receipts.lock().unwrap().is_empty());
            assert_eq!(sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap(), fixture.initial_rows);
            assert_eq!(fs::read(fixture.store().file_path()).unwrap(), bytes);
            let _subscription = fixture.subscribe_receipts();
            supported_settings_signals(fixture.settings());
            let marker = shipping::channel_failure_marker_from_failure(&fixture.agent_id, &failure).unwrap();
            let (completion_tx, completions) = mpsc::channel();
            shipping::queue_recovered_channel_delivery_failure(
                fixture.deps().clone(), Arc::clone(&fixture.roster), Arc::clone(&fixture.wakes),
                &marker, Some(completion_tx.clone()),
            ).unwrap();
            assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 3);
            assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
            assert_eq!(fixture.followups.lock().unwrap().len(), 1);
            assert!(matches!(completions.try_recv(), Err(mpsc::TryRecvError::Empty)),
                "entered generation refusal starts no recovery worker");
            fixture.assert_retained(&work.failures, &[]);
            assert_eq!(sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap(), fixture.initial_rows);
        } else {
            assert!(matches!(action, ChannelFailureAction::Finished));
            assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
            assert!(fixture.markers().is_empty());
            assert!(fixture.wakes.lock().unwrap().take_channel_failure_work_or_end_revival(&fixture.agent_id).is_none());
        }
    }

    for admission_fault in ["missing", "empty-revision", "malformed-json"] {
        let fixture = FailureReadinessFixture::new();
        fixture.settings().set_inference_provider("fabushi").unwrap();
        let failure = fixture.full_failure("checked-unavailable-generation", 206.0);
        let work = fixture.enqueue(&failure).unwrap();
        let bytes = fs::read(fixture.store().file_path()).unwrap();
        // This already-open reader only observes the real faulted DB. The
        // shipping attempt always re-enters its normal validating Session owner.
        let db = fixture.deps().session_workers.open_agent_db_owner(&fixture.agent_id).unwrap();
        let rows = db.get_transcript_entries().unwrap();
        match admission_fault {
            "missing" => assert!(db.delete_kv("sessionWorkAdmission").unwrap()),
            "empty-revision" => {
                let mut admission = db.read_session_work_admission().unwrap().unwrap();
                admission.lifecycle_revision.clear();
                assert!(db.write_kv("sessionWorkAdmission", &serde_json::to_string(&admission).unwrap()).unwrap());
            }
            _ => assert!(db.write_kv("sessionWorkAdmission", "corrupt actual admission JSON").unwrap()),
        }
        let faulted_admission = db.read_kv("sessionWorkAdmission").unwrap();
        assert!(matches!(fixture.process(&work), ChannelFailureAction::Held));
        fixture.assert_paused(&work.failures, &[]);
        assert!(fixture.followups.lock().unwrap().is_empty());
        assert!(fixture.receipts.lock().unwrap().is_empty());
        let _subscription = fixture.subscribe_receipts();
        fixture.settings().set_host_settings(&json!({"inferenceProvider":"fabushi"})).unwrap();
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.launches.load(Ordering::SeqCst), 1,
            "the existing single readiness latch permits one real recheck, not a spin");
        assert_eq!(fixture.passes.lock().unwrap().len(), 2);
        fixture.assert_paused(&work.failures, &[]);
        assert!(fixture.followups.lock().unwrap().is_empty());
        assert_eq!(db.get_transcript_entries().unwrap(), rows);
        assert_eq!(db.read_kv("sessionWorkAdmission").unwrap(), faulted_admission,
            "failed readiness must not repair or replace unknown Session provenance");
        assert_eq!(fs::read(fixture.store().file_path()).unwrap(), bytes);
    }
}

#[test]
fn channel_failure_checked_preflight_cleanup_refusal_retains_original_batch() {
    for partial_batch in [false, true] {
        let fixture = FailureReadinessFixture::new();
        fixture.settings().set_inference_provider("fabushi").unwrap();
        let first = fixture.failure_for_session(&fixture.agent_id, "agent:obsolete-first:main", "checked-stale-first", 207.0);
        let second = fixture.failure_for_session(&fixture.agent_id, "agent:obsolete-second:main", "checked-stale-second", 208.0);
        let failures = if partial_batch { vec![first.clone(), second] } else { vec![first] };
        let work = fixture.reserve_batch(&failures);
        let store = fixture.store();
        let _subscription = fixture.subscribe_receipts();
        let mut repair_image = None;
        let mut points = Vec::new();
        let action = shipping::process_channel_failure_work_observed(
            fixture.deps(), fixture.roster.as_ref(), &fixture.wakes, &fixture.agent_id, &work,
            |_, _, _| panic!("stale Session provenance cannot enter Runner"),
            |point| {
                assert!(fixture.wakes.try_lock().is_ok());
                points.push(point);
                let inject = if partial_batch {
                    matches!(point, ChannelFailureSettlementPoint::MarkerDisposed { index: 0, removed: true })
                } else { point == ChannelFailureSettlementPoint::PreflightAdmitted };
                if inject {
                    assert!(fixture.receipts.lock().unwrap().is_empty());
                    repair_image = Some(fs::read(store.file_path()).unwrap());
                    fs::remove_file(store.file_path()).unwrap();
                    fs::create_dir(store.file_path()).unwrap();
                    fs::write(store.file_path().join("unrelated-witness"), b"preflight fault witness").unwrap();
                }
            },
        );
        assert!(matches!(action, ChannelFailureAction::Held));
        fixture.assert_paused(&work.failures, &[]);
        assert!(fixture.followups.lock().unwrap().is_empty());
        assert!(fixture.receipts.lock().unwrap().is_empty());
        assert!(fixture.roster_events.lock().unwrap().is_empty());
        assert!(store.list_pending_checked().is_err());
        assert_eq!(fs::read(store.file_path().join("unrelated-witness")).unwrap(), b"preflight fault witness");
        assert_eq!(points, if partial_batch {
            vec![ChannelFailureSettlementPoint::PreflightAdmitted,
                ChannelFailureSettlementPoint::MarkerDisposed { index: 0, removed: true }]
        } else { vec![ChannelFailureSettlementPoint::PreflightAdmitted] });
        fs::remove_file(store.file_path().join("unrelated-witness")).unwrap();
        fs::remove_dir(store.file_path()).unwrap();
        fs::write(store.file_path(), repair_image.expect("real preflight cleanup fault")).unwrap();
        fixture.settings().set_host_settings(&json!({"inferenceProvider":"fabushi"})).unwrap();
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.launches.load(Ordering::SeqCst), 1);
        assert!(fixture.wakes.lock().unwrap().channel_failure_state(&fixture.agent_id).is_none());
        assert!(fixture.markers().is_empty());
        assert!(fixture.followups.lock().unwrap().is_empty(), "rechecked stale-only batches finish before Runner");
        assert!(fixture.receipts.lock().unwrap().is_empty());
        assert!(fixture.roster_events.lock().unwrap().is_empty());
        assert_eq!(fixture.deps().session_workers.read_agent_transcript_entries(&fixture.agent_id).unwrap(), fixture.initial_rows);
    }
}

#[test]
fn channel_failure_checked_close_orders_settlement_admission() {
    for close_after_admission in [false, true] {
        let fixture = FailureReadinessFixture::new();
        fixture.settings().set_inference_provider("fabushi").unwrap();
        let failure = fixture.full_failure("checked-run-settlement-close", 209.0);
        let work = fixture.enqueue(&failure).unwrap();
        let store = fixture.store();
        let bytes = fs::read(store.file_path()).unwrap();
        let mut points = Vec::new();
        let action = shipping::process_channel_failure_work_observed(
            fixture.deps(), fixture.roster.as_ref(), &fixture.wakes, &fixture.agent_id, &work,
            |provider, failures, generation| {
                fixture.observe_followup(provider, failures);
                let execution = fixture.real_followup(generation, true, false)?;
                assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
                if !close_after_admission {
                    fixture.manager().dispose();
                }
                Ok(execution)
            },
            |point| {
                assert!(fixture.wakes.try_lock().is_ok());
                points.push(point);
                if point == ChannelFailureSettlementPoint::RunAdmitted {
                    assert!(close_after_admission);
                    fixture.manager().dispose();
                }
            },
        );
        assert!(matches!(action, ChannelFailureAction::Stale));
        assert!(fixture.wakes.lock().unwrap().is_closed());
        assert_eq!(fixture.followups.lock().unwrap().len(), 1);
        assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
        if close_after_admission {
            assert_eq!(points, vec![ChannelFailureSettlementPoint::RunAdmitted,
                ChannelFailureSettlementPoint::MarkerDisposed { index: 0, removed: true }]);
            assert!(store.list_pending_checked().unwrap().is_empty(),
                "closing the volatile owner cannot retract previously admitted matching I/O");
        } else {
            assert!(points.is_empty(), "close before settlement admission permits zero marker I/O");
            assert_eq!(fs::read(store.file_path()).unwrap(), bytes);
        }
        let rows = fixture.rows_after_dispose();
        assert_eq!(shipping::collect_new_member_send_messages(&fixture.initial_rows, &rows).len(), 1);
        fixture.request_readiness_using(true);
        assert_eq!(fixture.rows_after_dispose(), rows);
        assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
        let mut owner = fixture.wakes.lock().unwrap();
        assert!(!owner.finish_channel_failure_work(&fixture.agent_id, &work));
        assert!(owner.take_channel_failure_work_or_end_revival(&fixture.agent_id).is_none());
        assert!(owner.pending_channel_failures.is_empty());
    }

    for close_after_admission in [false, true] {
        let fixture = FailureReadinessFixture::new();
        let failure = fixture.failure_for_session(&fixture.agent_id, "agent:obsolete-close:main", "checked-preflight-close", 210.0);
        let work = fixture.enqueue(&failure).unwrap();
        let store = fixture.store();
        let bytes = fs::read(store.file_path()).unwrap();
        if !close_after_admission {
            fixture.manager().dispose();
        }
        let mut points = Vec::new();
        let admitted = shipping::settle_preflight_channel_failure_markers(
            fixture.deps(), &fixture.wakes, &fixture.agent_id, &work, &work.failures,
            &mut |point| {
                assert!(fixture.wakes.try_lock().is_ok());
                points.push(point);
                if point == ChannelFailureSettlementPoint::PreflightAdmitted {
                    assert!(close_after_admission);
                    fixture.manager().dispose();
                }
            },
        ).unwrap();
        assert_eq!(admitted, close_after_admission);
        assert!(fixture.wakes.lock().unwrap().is_closed());
        assert!(!fixture.wakes.lock().unwrap().finish_channel_failure_work(&fixture.agent_id, &work));
        assert!(fixture.followups.lock().unwrap().is_empty());
        assert!(fixture.receipts.lock().unwrap().is_empty());
        if close_after_admission {
            assert_eq!(points, vec![ChannelFailureSettlementPoint::PreflightAdmitted,
                ChannelFailureSettlementPoint::MarkerDisposed { index: 0, removed: true }]);
            assert!(store.list_pending_checked().unwrap().is_empty());
        } else {
            assert!(points.is_empty());
            assert_eq!(fs::read(store.file_path()).unwrap(), bytes);
        }
        assert_eq!(fixture.rows_after_dispose(), fixture.initial_rows,
            "preflight retirement never appends or admits generated Session work");
    }
}

#[test]
fn channel_failure_checked_success_preserves_replacement_and_volatile_identity() {
    for volatile in [false, true] {
        let fixture = FailureReadinessFixture::new();
        fixture.settings().set_inference_provider("fabushi").unwrap();
        let failure = fixture.full_failure("checked-replacement", 211.0);
        let mut replacement = shipping::channel_failure_marker_from_failure(&fixture.agent_id, &failure).unwrap();
        replacement.title = Some("New exact metadata for the same operation and timestamp".into());
        replacement.quiet_origin.as_mut().unwrap().automation.as_mut().unwrap().name = "New exact origin".into();
        let work = if volatile {
            let volatile_failure = json!({"addressToken":"slack:room", "sessionKey":fixture.session_key,
                "markedAtMs":211.0, "reason":"volatile failure without a delivery operation identity"});
            assert!(shipping::channel_failure_marker_from_failure(&fixture.agent_id, &volatile_failure).is_none());
            shipping::enqueue_channel_failure(
                &mut fixture.wakes.lock().unwrap(), &fixture.agent_id, volatile_failure,
            ).unwrap().unwrap()
        } else {
            fixture.enqueue(&failure).unwrap()
        };
        let store = fixture.store();
        let mut replacement_bytes = None;
        let mut points = Vec::new();
        let action = shipping::process_channel_failure_work_observed(
            fixture.deps(), fixture.roster.as_ref(), &fixture.wakes, &fixture.agent_id, &work,
            |provider, failures, generation| {
                fixture.observe_followup(provider, failures);
                let execution = fixture.real_followup(generation, true, false)?;
                if !volatile {
                    store.mark_pending_checked(replacement.clone()).unwrap();
                    replacement_bytes = Some(fs::read(store.file_path()).unwrap());
                }
                Ok(execution)
            },
            |point| points.push(point),
        );
        assert!(matches!(action, ChannelFailureAction::Finished));
        assert_eq!(fixture.followups.lock().unwrap().len(), 1);
        assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
        assert!(fixture.wakes.lock().unwrap().take_channel_failure_work_or_end_revival(&fixture.agent_id).is_none());
        assert!(fixture.wakes.lock().unwrap().channel_failure_state(&fixture.agent_id).is_none());
        if volatile {
            assert_eq!(points, vec![ChannelFailureSettlementPoint::RunAdmitted]);
            assert!(!store.file_path().exists(), "volatile success cannot manufacture a recovery marker");
        } else {
            assert_eq!(points, vec![ChannelFailureSettlementPoint::RunAdmitted,
                ChannelFailureSettlementPoint::MarkerDisposed { index: 0, removed: false }]);
            assert_eq!(store.list_pending_checked().unwrap(), vec![replacement.clone()]);
            assert_eq!(fs::read(store.file_path()).unwrap(), replacement_bytes.unwrap());
        }
        let events = fixture.roster_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].phase, Some(ChannelFailureState::EnteredRunner));
        assert_eq!(events[0].markers, if volatile { vec![] } else { vec![replacement] });
        drop(events);
        let _subscription = fixture.subscribe_receipts();
        supported_settings_signals(fixture.settings());
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 3);
        assert_eq!(fixture.launches.load(Ordering::SeqCst), 0,
            "readiness cannot scan disk and recreate a finished volatile claim");
        assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
    }
}

#[test]
fn channel_failure_checked_terminal_loss_and_cancel_never_replay() {
    for outcome in ["lost-reply-after-receipt", "cancelled", "zero-send"] {
        let fixture = FailureReadinessFixture::new();
        fixture.settings().set_inference_provider("fabushi").unwrap();
        let head = fixture.full_failure("checked-terminal-head", 212.0);
        let during = fixture.full_failure("checked-terminal-during", 213.0);
        let later = fixture.full_failure("checked-terminal-later", 214.0);
        let work = fixture.enqueue(&head).unwrap();
        let _subscription = fixture.subscribe_receipts();
        let action = fixture.process_with(&work, |provider, failures, generation| {
            fixture.observe_followup(provider, failures);
            let execution = fixture.real_followup(
                generation, outcome == "lost-reply-after-receipt", outcome == "cancelled",
            )?;
            supported_settings_signals(fixture.settings());
            shipping::resume_channel_inbound_after_recreate(fixture.manager(), || fixture.request_readiness_using(true)).unwrap();
            assert!(fixture.enqueue(&during).is_none());
            assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
            assert_eq!(fixture.wakes.lock().unwrap().channel_failure_state(&fixture.agent_id), Some(ChannelFailureState::EnteredRunner));
            if outcome == "lost-reply-after-receipt" {
                assert_eq!(fixture.receipts.lock().unwrap().len(), 1);
                Err("controlled acknowledgement loss after actual result receipt".into())
            } else {
                Ok(execution)
            }
        });
        assert!(matches!(action, ChannelFailureAction::Held));
        fixture.assert_retained(&work.failures, &[during.clone()]);
        assert_eq!(fixture.receipts.lock().unwrap().len(), usize::from(outcome != "cancelled"));
        let rows = fixture.deps().session_workers.read_agent_transcript_entries(&fixture.agent_id).unwrap();
        assert_eq!(shipping::collect_new_member_send_messages(&fixture.initial_rows, &rows).len(),
            usize::from(outcome == "lost-reply-after-receipt"));
        assert!(fixture.enqueue(&later).is_none());
        let bytes = fs::read(fixture.store().file_path()).unwrap();
        supported_settings_signals(fixture.settings());
        shipping::resume_channel_inbound_after_recreate(fixture.manager(), || fixture.request_readiness_using(true)).unwrap();
        let (completion_tx, completions) = mpsc::channel();
        for failure in [&head, &during, &later] {
            let marker = shipping::channel_failure_marker_from_failure(&fixture.agent_id, failure).unwrap();
            shipping::queue_recovered_channel_delivery_failure(
                fixture.deps().clone(), Arc::clone(&fixture.roster), Arc::clone(&fixture.wakes),
                &marker, Some(completion_tx.clone()),
            ).unwrap();
        }
        fixture.assert_retained(&work.failures, &[during, later]);
        assert_eq!(fixture.readiness_calls.load(Ordering::SeqCst), 8);
        assert_eq!(fixture.launches.load(Ordering::SeqCst), 0);
        assert_eq!(fixture.followups.lock().unwrap().len(), 1);
        assert_eq!(fixture.receipts.lock().unwrap().len(), usize::from(outcome != "cancelled"));
        assert_eq!(fixture.deps().session_workers.read_agent_transcript_entries(&fixture.agent_id).unwrap(), rows);
        assert_eq!(fs::read(fixture.store().file_path()).unwrap(), bytes);
        assert!(fixture.roster_events.lock().unwrap().is_empty());
        assert!(matches!(completions.try_recv(), Err(mpsc::TryRecvError::Empty)));
        assert!(matches!(fixture.process(&work), ChannelFailureAction::Stale));
        assert!(!shipping::launch_channel_failure_work(&fixture.wakes, &fixture.agent_id, work, |_| {
            panic!("unknown, cancelled and zero-send entered heads cannot replay")
        }).unwrap());
    }
}
