//! Causal shipping-path oracles for OCRM-CHANNEL-HOST-LANE-DELIVERY-v1.
//!
//! Session, the sink, lane dispatcher, delivery router and failure owners are
//! real. Only the external ChannelDeliveryPort chooses a transport outcome.

use super::super::*;
use mahayana_host_runtime::connectors::channel_delivery::{
    ChannelDeliveryPort, ChannelDeliveryRouter, ChannelOutboundMessage,
};
use mahayana_host_runtime::extensions::action_audit::action_audit_backend::create_sand_audit_batch_sender;
use mahayana_host_runtime::extensions::action_audit::action_audit_service::SandActionAuditor;
use mahayana_host_runtime::extensions::auth::auth_service::{
    HostAuthEnvironment, HostAuthServiceOptions,
};
use mahayana_host_runtime::extensions::auth::extension::start_host_auth_extension_with_options;
use mahayana_host_runtime::extensions::auth::user_full_name_service::production_user_full_name_fetch;
use mahayana_host_runtime::extensions::auto_review::extension::{
    no_op_auto_review_telemetry_sink, no_op_auto_review_update_sink,
    start_auto_review_extension,
};
use mahayana_host_runtime::extensions::box_lifecycle::box_lifecycle_service::BoxLifecycleService;
use mahayana_host_runtime::extensions::box_lifecycle::production::ProductionBoxLifecycleClient;
use mahayana_host_runtime::extensions::box_store_sync::extension::{
    BoxStoreScheduling, BoxStoreSyncExtensionDeps, BoxStoreSyncService,
};
use mahayana_host_runtime::extensions::box_store_sync::production::ProductionBoxStoreSyncService;
use mahayana_host_runtime::extensions::experiments::HostExperimentsOptions;
use mahayana_host_runtime::extensions::forever_box::host_box::HostBox;
use mahayana_host_runtime::extensions::inference::production::start_production_inference_extension;
use mahayana_host_runtime::extensions::local_exec::extension::start_local_exec_extension;
use mahayana_host_runtime::extensions::local_tool_permission::extension::start_local_tool_permission_extension;
use mahayana_host_runtime::extensions::managed_setup::team_rules::ProductionTeamRulesResolver;
use mahayana_host_runtime::extensions::source_map::source_map_service::SandSourceMap;
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostTelemetryService;
use mahayana_host_runtime::extensions::transcript::sand_pending_wake_store::SandPendingWakeStore;
use mahayana_host_runtime::extensions::trays::extension::start_trays_extension;
use mahayana_host_runtime::production_binding_providers::production_cloud_agent_trace_converter;
use mahayana_host_runtime::runner::background_work::CloudAgentWatchOutcome;
use serde_json::{Value, json};
use std::sync::Condvar;

const WAIT: Duration = Duration::from_secs(30);
const CLEANUP_WAIT: Duration = Duration::from_secs(5);
const FIXTURE_BACKEND: &str = "http://127.0.0.1:9";
const ADDRESS: &str = "slack:shipping-fixture-room";
const FOCUS_AT_MS: f64 = 123.0;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransportCall {
    agent_id: String,
    address: String,
    message: ChannelOutboundMessage,
}

#[derive(Clone, Copy)]
enum TransportOutcome {
    Delivered,
    Rejected(&'static str),
    EffectThenError(&'static str),
}

struct ControlledChannelPort {
    calls: Mutex<Vec<TransportCall>>,
    effects: Mutex<Vec<TransportCall>>,
    entered: mpsc::Sender<TransportCall>,
    released: Mutex<bool>,
    release_changed: Condvar,
    outcome: TransportOutcome,
}

impl ControlledChannelPort {
    fn new(
        outcome: TransportOutcome,
        held: bool,
    ) -> (Arc<Self>, mpsc::Receiver<TransportCall>) {
        let (entered, receiver) = mpsc::channel();
        (
            Arc::new(Self {
                calls: Mutex::new(Vec::new()),
                effects: Mutex::new(Vec::new()),
                entered,
                released: Mutex::new(!held),
                release_changed: Condvar::new(),
                outcome,
            }),
            receiver,
        )
    }

    fn release(&self) {
        *self.released.lock().unwrap_or_else(|error| error.into_inner()) = true;
        self.release_changed.notify_all();
    }

    fn calls(&self) -> Vec<TransportCall> {
        self.calls.lock().expect("transport calls").clone()
    }

    fn effects(&self) -> Vec<TransportCall> {
        self.effects.lock().expect("transport effects").clone()
    }
}

impl ChannelDeliveryPort for ControlledChannelPort {
    fn deliver(
        &self,
        agent_id: &str,
        address_token: &str,
        message: &ChannelOutboundMessage,
    ) -> Result<(), String> {
        let call = TransportCall {
            agent_id: agent_id.to_string(),
            address: address_token.to_string(),
            message: message.clone(),
        };
        self.calls.lock().expect("transport calls").push(call.clone());
        let _ = self.entered.send(call.clone());
        let (released, timed_out) = self.release_changed
            .wait_timeout_while(
                self.released.lock().expect("transport release"),
                WAIT,
                |released| !*released,
            )
            .expect("held transport wait");
        if timed_out.timed_out() && !*released {
            return Err("fixture transport was not released before its deadline".into());
        }
        drop(released);
        match self.outcome {
            TransportOutcome::Delivered => {
                self.effects.lock().expect("transport effects").push(call);
                Ok(())
            }
            TransportOutcome::Rejected(reason) => Err(reason.to_string()),
            TransportOutcome::EffectThenError(reason) => {
                self.effects.lock().expect("transport effects").push(call);
                Err(reason.to_string())
            }
        }
    }
}

struct IdleRunnerOwners {
    deps: LocalRoutedRunnerDeps,
    box_store: ProductionBoxStoreSyncService,
    _telemetry: Arc<HostTelemetryService>,
}

fn idle_box_store(
    root: &Path,
    auth: &Arc<HostAuthExtension>,
    manager: &Arc<TranscriptManager>,
    logs: &HostStructuredLogTelemetry,
) -> ProductionBoxStoreSyncService {
    let source_map = Arc::new(SandSourceMap::new(Some(root.join("source-map.json"))));
    let idle_manager = Arc::clone(manager);
    let token_auth = Arc::clone(auth);
    let machine_auth = Arc::clone(auth);
    let diagnostics = logs.clone();
    let cycles = logs.clone();
    let captures = logs.clone();
    let conflicts = logs.clone();
    let chrome = logs.clone();
    // Construct its real API without starting unrelated sync/poller work.
    ProductionBoxStoreSyncService::new(BoxStoreSyncExtensionDeps {
        native_credentials: Some(auth.service().clone()),
        is_idle: Arc::new(move || {
            idle_manager.live_running_agent_ids().is_empty()
                && !idle_manager.has_carryable_pending_wake()
        }),
        resolve_store_id: Arc::new(move || {
            source_map.get_or_create_box_store()
                .map(|entry| entry.source_id)
                .map_err(|error| error.to_string())
        }),
        backend_url: Some(FIXTURE_BACKEND.into()),
        get_access_token: Some(Arc::new(move || {
            token_auth.get_access_token().map_err(|error| error.to_string())
        })),
        get_machine_id: Some(Arc::new(move || {
            machine_auth.get_machine_id().map_err(|error| error.to_string())
        })),
        log: Arc::new(|message| eprintln!("channel-lane-fixture {message}")),
        report_host_extension_diagnostic: Arc::new(move |value| {
            let _ = diagnostics.report_host_extension_diagnostic(&Value::Object(value.clone()));
        }),
        report_box_store_sync_cycle: Arc::new(move |level, value| {
            let _ = cycles.report_box_store_sync_cycle(level, value);
        }),
        report_box_store_db_capture: Arc::new(move |level, value| {
            let _ = captures.report_box_store_db_capture(level, value);
        }),
        report_box_store_manifest_conflict: Arc::new(move |level, value| {
            let _ = conflicts.report_box_store_manifest_conflict(level, value);
        }),
        report_chrome_session_stage: Arc::new(move |level, value| {
            let _ = chrome.report_chrome_session_stage(level, value);
        }),
        scheduling: BoxStoreScheduling::default(),
    })
}

fn idle_runner_owners(
    root: &Path,
    host_tx: mpsc::Sender<HostLaneRequest>,
    events: GatewayEventHub,
    sessions: Arc<ProductionSessionWorkers>,
    manager: Arc<TranscriptManager>,
) -> IdleRunnerOwners {
    let settings = Arc::new(SettingsService::new(root.join("settings.json")));
    let auth = Arc::new(start_host_auth_extension_with_options(
        HostAuthServiceOptions {
            environment: HostAuthEnvironment::from_pairs(std::iter::empty()),
            backend_url: Some(FIXTURE_BACKEND.into()),
            backend: None,
            machine_id: Arc::new(|| Ok("channel-lane-fixture-machine".into())),
            log: Arc::new(|message| eprintln!("channel-lane-fixture-auth {message}")),
            now_ms: Arc::new(started_at_ms),
        },
        production_user_full_name_fetch(FIXTURE_BACKEND.into()),
    ).expect("real credential-free Auth owner"));
    let experiments = Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
        is_dev_build: false,
        env_gate_overrides: None,
    }));
    let inference = Arc::new(start_production_inference_extension(
        Arc::clone(&auth), Arc::clone(&experiments), Arc::clone(&settings),
    ));
    let telemetry = Arc::new(HostTelemetryService::open_with_identity_tags(
        root.join("telemetry.jsonl"), BTreeMap::new(),
    ).expect("real local telemetry owner"));
    let local_tool_permission = Arc::new(start_local_tool_permission_extension(Arc::clone(&settings)));
    let local_exec = Arc::new(start_local_exec_extension(telemetry.logs.clone()));
    let auto_review = Arc::new(start_auto_review_extension(
        Arc::clone(&sessions), Arc::clone(&experiments), Arc::clone(&settings),
        format!("channel-lane-{}", uuid::Uuid::new_v4()),
        no_op_auto_review_update_sink(), no_op_auto_review_telemetry_sink(),
    ));
    let lifecycle = Arc::new(BoxLifecycleService::new(
        ProductionBoxLifecycleClient::new(Arc::clone(&auth), FIXTURE_BACKEND)
            .expect("real Box lifecycle client"),
    ));
    let forever_box = Arc::new(ForeverBoxService::new(
        HostBox::new(ProductionBoxEnvironment::new("127.0.0.1", 9, "fixture-token")),
        lifecycle, false, false, false,
    ));
    let prepare_box = Arc::clone(&forever_box);
    let screenshot_box = Arc::clone(&forever_box);
    let session_handoff = BoxHandoffService::new(BoxHandoffDeps {
        prepare: Some(Arc::new(move |request| {
            prepare_box.ensure(&request.agent_id).map(|_| ()).map_err(|error| error.to_string())
        })),
        grab_screenshot: Some(Arc::new(move |agent_id| {
            screenshot_box.capture_screenshot(agent_id)
                .map(|bytes| bytes.map(ScreenshotPayload::Bytes))
                .map_err(|error| error.to_string())
        })),
        ..BoxHandoffDeps::default()
    });
    manager.set_handoff_service(session_handoff.clone()).expect("bind actual handoff owner");
    let request_context: Arc<dyn RunnerRequestContextSource> = Arc::new(
        ProductionRunnerRequestContextSource::new(
            Arc::clone(&auth),
            Arc::new(ProductionTeamRulesResolver::new(FIXTURE_BACKEND.into(), Arc::clone(&auth))),
            root.join("transcripts"),
        ),
    );
    let routed_tool_relay = Arc::new(CoordinatorToolRelay::new(events.clone()));
    let mcp_relay = Arc::new(CoordinatorMcpLifecycleRelay::new(events.clone()));
    let mcp_box = Arc::clone(&forever_box);
    let box_status_loader: BoxServerStatusLoader = Arc::new(move |ids, kick_only| {
        let accessor = mcp_box.mcp_resource_accessor().map_err(|error| error.to_string())?;
        ProductionBoxMcpStateLoader::new(accessor).list_servers(ids, kick_only)
    });
    let mcp_service = Arc::new(McpHostService::new(
        Arc::new(CoordinatorMcpManagerBackend::new(mcp_relay, box_status_loader)), None,
    ));
    let cloud_agents = SandCloudAgentManager::production(
        FIXTURE_BACKEND.into(), Arc::clone(&auth), production_cloud_agent_trace_converter(),
    );
    let cloud_watch_manager = Arc::clone(&cloud_agents);
    let cloud_agent_watches = Arc::new(RunnerCloudAgentWatches::new(
        Arc::new(move |bc_id, wait_for_restart| {
            let outcome = cloud_watch_manager.await_completion(bc_id, wait_for_restart);
            CloudAgentWatchOutcome { status: outcome.status.to_string(), text: outcome.text }
        }),
        None, None, None,
    ));
    let shell_box = Arc::clone(&forever_box);
    let shell_permission = local_tool_permission.controller();
    let background_shell_watches = Arc::new(RunnerBackgroundShellWatches::new(
        Arc::new(move |agent_id, shell_id, cancelled| {
            let resources = ForeverBoxRunnerResourcePort::new(
                Arc::clone(&shell_box), agent_id.to_string(),
            ).with_background_read_permission(Arc::clone(&shell_permission));
            poll_shell_terminal_file(
                started_at_ms(), shell_rewatch_poll_ms(None),
                || resources.read_background_shell_terminal(shell_id),
                started_at_ms,
                |milliseconds| thread::sleep(Duration::from_millis(milliseconds)),
                || cancelled.load(Ordering::Acquire),
            ).map(|settlement| BackgroundShellWatchOutcome {
                status: match settlement.status {
                    ShellWatchStatus::Success => "success".into(),
                    ShellWatchStatus::Error => "error".into(),
                },
                detail: settlement.detail,
                output_path: settlement.output_path,
            })
        }),
        None, None, None,
    ));
    let audit_agents = root.join("agents");
    let audit_experiments = Arc::clone(&experiments);
    let production_action_auditor = ActionAuditExtension::new(Arc::new(SandActionAuditor::new(
        root.join("audit-outbox.json"),
        Arc::new(move |agent_id| audit_agents.join(agent_id).join("audit.jsonl")),
        Arc::new(move || audit_experiments.check_feature_gate("sand_action_audit_logs")),
        create_sand_audit_batch_sender(FIXTURE_BACKEND.into(), Arc::clone(&auth)),
        Duration::from_secs(60),
    )));
    let box_store = idle_box_store(root, &auth, &manager, &telemetry.logs);
    let deps = LocalRoutedRunnerDeps {
        routed_tool_relay,
        mcp_service,
        auth,
        auto_review,
        events,
        host_tx,
        data_dir: root.to_path_buf(),
        request_context,
        experiments,
        settings,
        inference,
        session_workers: Arc::clone(&sessions),
        runner_registry: manager.runner_registry(),
        ack_obligations: manager.ack_obligations(),
        transcript_runtime: manager.transcript_runtime(),
        transcript_manager: Arc::clone(&manager),
        generated_agent_runtime: Arc::new(Mutex::new(SubagentRuntime::default())),
        completion_revivals: Arc::new(CompletionRevivals::new(Arc::new(
            ProductionCompletionRevivalRuntime {
                gateway: Arc::new(Mutex::new(Weak::<UnifiedGatewayApi>::new())),
            },
        ))),
        forever_box,
        local_exec,
        local_tool_permission: Arc::clone(&local_tool_permission),
        session_handoff,
        trays: Arc::new(start_trays_extension()),
        telemetry_logs: telemetry.logs.clone(),
        product_analytics: telemetry.analytics.clone(),
        production_action_auditor,
        cloud_agents,
        cloud_agent_watches,
        background_shell_watches,
        host_runner_composition: Arc::new(HostRunnerComposition::production(
            local_tool_permission.controller(), sessions,
        )),
        box_store_sync: box_store.api(),
        automations_lifecycle: Arc::new(Mutex::new(Weak::<ProductionAutomationsLifecycle>::new())),
        generated_subagent_cleanup: Arc::new(Mutex::new(None)),
    };
    IdleRunnerOwners { deps, box_store, _telemetry: telemetry }
}

#[derive(Debug)]
struct LaneObservation {
    method: String,
    args: Value,
    result: Result<Value, String>,
}

struct PersistedChannelSend {
    entry_id: String,
    entries: Vec<Value>,
}

struct ChannelHostLaneFixture {
    root: PathBuf,
    agent_id: String,
    session_key: String,
    host_tx: mpsc::Sender<HostLaneRequest>,
    owners: Option<IdleRunnerOwners>,
    sink: Option<ProductionSendMessageSink>,
    router: ChannelDeliveryRouter,
    ports: Vec<Arc<ControlledChannelPort>>,
    lane_observations: mpsc::Receiver<LaneObservation>,
    completions: mpsc::Receiver<ChannelDeliveryWorkerFinished>,
    lifecycle_events: Arc<Mutex<Vec<(String, String)>>>,
    lane: Option<thread::JoinHandle<()>>,
    lane_done: mpsc::Receiver<()>,
    lane_exited: bool,
    lane_panicked: bool,
    workers_settled: bool,
}

impl ChannelHostLaneFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-channel-host-lane-{label}-{}", uuid::Uuid::new_v4(),
        ));
        fs::create_dir_all(&root).expect("fixture directory");
        fs::write(root.join("settings.json"), b"{}").expect("actual missing-provider settings");
        let (host_tx, host_rx) = mpsc::channel();
        let events = GatewayEventHub::default();
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let record = sessions.materialize_new_session(None, "user", None).expect("real Session");
        let admission = sessions.read_session_work_admission(&record.id)
            .expect("Session admission read").expect("materialized Session admission");
        let manager = Arc::new(TranscriptManager::new(&root, Arc::clone(&sessions)));
        let owners = idle_runner_owners(&root, host_tx.clone(), events.clone(), sessions, manager);
        let deps = &owners.deps;
        let roster_events = events.clone();
        let roster = Arc::new(ProductionRosterEmit::new(
            Arc::clone(&deps.session_workers), Arc::clone(&deps.transcript_runtime),
            Arc::new(move |event| roster_events.publish(event)),
        ));
        deps.transcript_manager.bind_roster_emit(Arc::clone(&roster)).expect("canonical roster");
        deps.transcript_runtime.session_runtime().set_window_focused(
            &deps.session_workers, true, FOCUS_AT_MS,
        ).expect("actual focused desktop state");
        let lifecycle_events = Arc::new(Mutex::new(Vec::new()));
        let observed_lifecycle = Arc::clone(&lifecycle_events);
        deps.transcript_runtime.set_agent_run_lifecycle_observer(Some(Arc::new(
            move |event, request_id| {
                observed_lifecycle.lock().expect("run lifecycle observation")
                    .push((event.to_string(), request_id.to_string()));
            },
        )));
        let sink = ProductionSendMessageSink {
            host_tx: host_tx.clone(),
            cloud_agents: Arc::clone(&deps.cloud_agents),
            sessions: Arc::clone(&deps.session_workers),
            forever_box: Arc::clone(&deps.forever_box),
            transcript_manager: Arc::clone(&deps.transcript_manager),
            events,
            ack_obligations: Arc::clone(&deps.ack_obligations),
            transcript_runtime: Arc::clone(&deps.transcript_runtime),
            ack_token: None,
            agent_id: record.id.clone(),
            session_key: Some(admission.session_key.clone()),
            reply_thread_target: None,
            is_fork: false,
            pending_media: Mutex::new(HashMap::new()),
        };
        let router = ChannelDeliveryRouter::new();
        let (completion_tx, completions) = mpsc::channel();
        let dispatch_deps = ChannelDeliveryDispatchDeps {
            delivery: HostChannelDelivery::new(Arc::new(router.clone())),
            runner: deps.clone(),
            roster,
            wakes: deps.transcript_manager.background_wakes(),
            completion: Some(completion_tx),
        };
        let attachments = AttachmentsService::new(
            &root, Arc::clone(&deps.auth) as Arc<dyn GenerateImageAuth>,
            Arc::clone(&deps.forever_box)
                as Arc<dyn mahayana_host_runtime::extensions::attachments::attachments_service::AttachmentsBox>,
            None,
        );
        let forever_box = Arc::clone(&deps.forever_box);
        let lane_root = root.join("host");
        let (observation_tx, lane_observations) = mpsc::channel();
        let (ready_tx, ready) = mpsc::sync_channel(1);
        let (done_tx, lane_done) = mpsc::sync_channel(1);
        let lane = thread::Builder::new().name(format!("channel-fixture-lane-{label}"))
            .spawn(move || {
                // UnifiedAppHost is born and remains on this lane: its QuickJS
                // owner is never sent between threads. Only the shipping
                // receive/reply arm is mirrored here, not method dispatch.
                let host = ensure_managed_runtime_layout(&lane_root)
                    .map_err(|error| error.to_string())
                    .and_then(|()| UnifiedAppHost::new(lane_root).map_err(|error| error.to_string()));
                match host {
                    Ok(host) => {
                        let _ = ready_tx.send(Ok::<(), String>(()));
                        while let Ok(request) = host_rx.recv() {
                            match request {
                                HostLaneRequest::Gateway { method, args, reply } => {
                                    let observed_args = args.clone();
                                    let result = dispatch_gateway_call(
                                        &host, &forever_box, &attachments, &dispatch_deps,
                                        &method, args,
                                    );
                                    let observed_result = result.as_ref()
                                        .map(Clone::clone).map_err(ToString::to_string);
                                    let _ = reply.send(result);
                                    let _ = observation_tx.send(LaneObservation {
                                        method, args: observed_args, result: observed_result,
                                    });
                                }
                                HostLaneRequest::StdinClosed => break,
                                HostLaneRequest::Stdin(_) => panic!("unexpected fixture stdin request"),
                            }
                        }
                    }
                    Err(error) => { let _ = ready_tx.send(Err(error)); }
                }
                drop(attachments);
                drop(forever_box);
                drop(dispatch_deps);
                let _ = done_tx.send(());
            }).expect("owned Host lane thread");
        let fixture = Self {
            root, agent_id: record.id, session_key: admission.session_key, host_tx,
            owners: Some(owners), sink: Some(sink), router, ports: Vec::new(),
            lane_observations, completions, lifecycle_events,
            lane: Some(lane), lane_done, lane_exited: false, lane_panicked: false,
            workers_settled: true,
        };
        ready.recv_timeout(WAIT).expect("real Host lane startup deadline")
            .expect("real UnifiedAppHost startup");
        fixture
    }

    fn deps(&self) -> &LocalRoutedRunnerDeps {
        &self.owners.as_ref().expect("fixture owners are live").deps
    }

    fn register_transport(
        &mut self,
        outcome: TransportOutcome,
        held: bool,
    ) -> (Arc<ControlledChannelPort>, mpsc::Receiver<TransportCall>) {
        let (port, entered) = ControlledChannelPort::new(outcome, held);
        let provider: Arc<dyn ChannelDeliveryPort> = port.clone();
        self.router.register("slack", provider).expect("real router registration");
        self.ports.push(Arc::clone(&port));
        (port, entered)
    }

    fn entries(&self) -> Vec<Value> {
        self.deps().session_workers.read_agent_transcript_entries(&self.agent_id)
            .expect("actual Session SQLite transcript")
    }

    fn send(&mut self, message: Value, queued: bool) -> PersistedChannelSend {
        assert!(self.workers_settled, "settle the preceding send first");
        let before = self.entries();
        self.workers_settled = false;
        let entry_id = self.sink.as_ref().expect("real SendMessage sink")
            .send_message(message.clone(), 1_000, "channel-fixture-send")
            .expect("real sink append").expect("persisted SendMessage entry ID");
        let persisted = self.entries();
        assert_eq!(persisted.len(), before.len() + 1, "one actual sink append");
        assert_eq!(&persisted[..before.len()], before.as_slice());
        let entry = persisted.last().expect("new persisted entry");
        assert_eq!(entry["id"], entry_id);
        assert_eq!(entry["kind"], "send-message");
        assert_eq!(entry["sessionKey"], self.session_key);
        assert_eq!(entry["message"], message);
        let observed = self.lane_observations.recv_timeout(WAIT)
            .expect("the sink must reach the real Host-lane dispatcher");
        assert_eq!(observed.method, "deliverToChannel");
        assert_eq!(observed.args["agentId"], self.agent_id);
        assert_eq!(observed.args["addressToken"], ADDRESS);
        assert_eq!(observed.args["sessionKey"], self.session_key);
        assert_eq!(observed.args["message"]["deliveryOperationId"], entry_id);
        assert_eq!(observed.result.expect("real internal delivery dispatch"), json!({"queued": queued}));
        if !queued {
            self.workers_settled = true;
        }
        PersistedChannelSend { entry_id, entries: persisted }
    }

    fn request(&self, method: &str) -> Result<Value, GatewayCommandError> {
        let (reply, result) = mpsc::sync_channel(1);
        self.host_tx.send(HostLaneRequest::Gateway {
            method: method.to_string(), args: json!({}), reply,
        }).expect("send to the same actual Host lane");
        let result = result.recv_timeout(WAIT).expect("Host lane must remain responsive");
        let observed = self.lane_observations.recv_timeout(WAIT).expect("actual fallback dispatch");
        assert_eq!(observed.method, method);
        result
    }

    fn wait_workers(&mut self, send: &PersistedChannelSend, failure: bool) {
        let mut delivery_done = false;
        let mut revival_done = false;
        let expected_completions = if failure { 2 } else { 1 };
        for _ in 0..expected_completions {
            match self.completions.recv_timeout(WAIT).expect("actual worker completion deadline") {
                ChannelDeliveryWorkerFinished::Delivery { agent_id, operation_id } => {
                    assert!(!delivery_done, "no duplicate delivery worker");
                    assert_eq!(agent_id, self.agent_id);
                    assert_eq!(operation_id.as_deref(), Some(send.entry_id.as_str()));
                    delivery_done = true;
                }
                ChannelDeliveryWorkerFinished::FailureRevival { agent_id } => {
                    assert!(failure, "successful transport must not start failure revival");
                    assert!(!revival_done, "no duplicate failure-revival worker");
                    assert_eq!(agent_id, self.agent_id);
                    revival_done = true;
                }
            }
        }
        assert!(delivery_done);
        assert_eq!(revival_done, failure);
        self.workers_settled = true;
        self.assert_no_worker_observation();
    }

    fn assert_no_worker_observation(&self) {
        assert!(matches!(self.completions.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "no extra delivery or failure worker may complete");
    }

    fn assert_unchanged_append_and_no_followup(&self, send: &PersistedChannelSend) {
        assert_eq!(self.entries(), send.entries, "delivery must not append again");
        assert_eq!(self.deps().runner_registry.active_count(), 0);
        assert_eq!(self.deps().transcript_runtime.in_flight_run_count(&self.agent_id), 0);
        assert!(self.deps().transcript_runtime.is_turn_dispatch_idle(&self.agent_id));
        assert!(self.lifecycle_events.lock().expect("real run lifecycle observations").is_empty(),
            "no background follow-up may have been admitted and later hidden by settlement");
        assert_eq!(self.deps().transcript_runtime.session_runtime().window_focused_at_ms(),
            Some(FOCUS_AT_MS), "internal routing must not manufacture desktop contact");
    }

    fn assert_no_failure(&self) {
        assert!(self.deps().trays.list().is_empty());
        assert!(SandPendingWakeStore::new(&self.root).list_pending().is_empty());
        let wakes = self.deps().transcript_manager.background_wakes();
        let wakes = wakes.lock().expect("actual BackgroundWakes owner");
        assert!(wakes.pending_channel_failures.is_empty());
        assert!(wakes.reviving_channel_failure_agent_ids.is_empty());
    }

    fn assert_retained_failure(&self, send: &PersistedChannelSend, reason: &str) {
        assert!(self.workers_settled, "observe both actual workers before the failure oracle");
        self.assert_unchanged_append_and_no_followup(send);
        let trays = self.deps().trays.list();
        assert_eq!(trays.len(), 1);
        assert_eq!(trays[0].title, "Message not delivered");
        assert_eq!(trays[0].agent_id.as_deref(), Some(self.agent_id.as_str()));
        assert_eq!(trays[0].raw_detail.as_deref(), Some(reason));
        let runtime_store = self.deps().transcript_runtime.pending_wake_store()
            .expect("actual durable pending-wake owner");
        assert!(runtime_store.file_path().is_file(), "failure must reach durable storage");
        let reopened = SandPendingWakeStore::new(&self.root);
        let markers = reopened.list_pending();
        assert_eq!(markers, runtime_store.list_pending(), "fresh reader observes the actual persisted marker");
        assert_eq!(markers.len(), 1);
        let marker = &markers[0];
        assert_eq!(marker.agent_id, self.agent_id);
        assert_eq!(marker.kind, PendingWakeKind::ChannelDelivery);
        assert_eq!(marker.work_id, send.entry_id);
        assert_eq!(marker.delivery_operation_id.as_deref(), Some(send.entry_id.as_str()));
        assert_eq!(marker.address_token.as_deref(), Some(ADDRESS));
        assert_eq!(marker.session_key.as_deref(), Some(self.session_key.as_str()));
        assert_eq!(marker.failure_reason.as_deref(), Some(reason));
        assert!(marker.marked_at_ms.is_finite() && marker.marked_at_ms > 0.0);
        let wakes = self.deps().transcript_manager.background_wakes();
        let wakes = wakes.lock().expect("actual failure queue after worker completion");
        let failures = wakes.pending_channel_failures.get(&self.agent_id)
            .expect("deferred failure remains queued");
        assert_eq!(failures.as_slice(), &[json!({
            "addressToken": ADDRESS,
            "deliveryOperationId": send.entry_id,
            "markedAtMs": marker.marked_at_ms,
            "sessionKey": self.session_key,
            "reason": reason,
        })]);
        assert!(!wakes.reviving_channel_failure_agent_ids.contains(&self.agent_id),
            "actual deferred worker has ended revival");
    }

    fn stop_lane(&mut self) {
        for port in &self.ports {
            port.release();
        }
        let Some(lane) = self.lane.take() else { return };
        let _ = self.host_tx.send(HostLaneRequest::StdinClosed);
        self.lane_exited = self.lane_done.recv_timeout(CLEANUP_WAIT).is_ok();
        if self.lane_exited {
            self.lane_panicked = lane.join().is_err();
        }
        // A regressed self-wait must fail within the test deadline, not make
        // teardown wait forever. Dropping an unacknowledged handle detaches
        // only this fixture's lane; its directory is retained on that failure.
    }

    fn dispose_owners(&mut self) {
        drop(self.sink.take());
        if let Some(owners) = self.owners.take() {
            owners.deps.cloud_agent_watches.dispose_parent(&self.agent_id);
            owners.deps.background_shell_watches.dispose_parent(&self.agent_id);
            owners.deps.auto_review.stop();
            owners.deps.production_action_auditor.stop();
            owners.deps.cloud_agents.dispose();
            owners.deps.host_runner_composition.dispose();
            owners.deps.forever_box.dispose();
            owners.box_store.dispose();
            owners.deps.transcript_manager.dispose();
        }
    }

    fn close(mut self) {
        assert!(self.workers_settled, "workers must finish before owner disposal");
        self.assert_no_worker_observation();
        self.stop_lane();
        assert!(self.lane_exited && !self.lane_panicked, "owned Host lane shutdown deadline");
        self.dispose_owners();
        let root = self.root.clone();
        drop(self);
        fs::remove_dir_all(root).expect("remove only the completed fixture directory");
    }
}

impl Drop for ChannelHostLaneFixture {
    fn drop(&mut self) {
        self.stop_lane();
        if self.lane_exited && self.workers_settled {
            self.dispose_owners();
        }
    }
}

fn text_message(text: &str) -> Value {
    json!({"type":"text", "content":text, "channel":ADDRESS})
}

fn assert_transport_identity(
    fixture: &ChannelHostLaneFixture,
    send: &PersistedChannelSend,
    call: &TransportCall,
    text: &str,
) {
    assert_eq!(call.agent_id, fixture.agent_id);
    assert_eq!(call.address, ADDRESS);
    assert_eq!(call.message, ChannelOutboundMessage::Text {
        text: text.to_string(), operation_id: Some(format!("{}:chunk-1", send.entry_id)),
    });
}

#[test]
fn channel_send_host_lane_reaches_existing_delivery_owner_once() {
    let mut fixture = ChannelHostLaneFixture::new("success");
    let (port, entered) = fixture.register_transport(TransportOutcome::Delivered, false);
    let send = fixture.send(text_message("one persisted channel message"), true);
    let call = entered.recv_timeout(WAIT).expect("actual external transport entry");
    fixture.wait_workers(&send, false);
    assert_transport_identity(&fixture, &send, &call, "one persisted channel message");
    assert_eq!(port.calls(), vec![call.clone()]);
    assert_eq!(port.effects(), vec![call]);
    fixture.assert_unchanged_append_and_no_followup(&send);
    fixture.assert_no_failure();
    fixture.close();
}

#[test]
fn channel_send_host_lane_failure_reaches_tray_and_pending_wake() {
    let mut fixture = ChannelHostLaneFixture::new("quiesced-failure");
    fs::write(fixture.root.join("settings.json"), br#"{"inferenceProvider":"fabushi"}"#)
        .expect("actual configured-provider settings");
    assert_eq!(configured_routed_provider(&fixture.root.join("settings.json")), Some(RoutedProvider::Fabushi));
    let (port, entered) = fixture.register_transport(
        TransportOutcome::Rejected("transport rejected the channel message"), true,
    );
    let send = fixture.send(text_message("failure held through quiescence"), true);
    let call = entered.recv_timeout(WAIT).expect("registered transport was actually called");
    fixture.deps().transcript_runtime.quiesce_for_upgrade().expect("actual runtime quiescence");
    assert!(fixture.deps().transcript_runtime.is_quiescing_for_upgrade());
    port.release();
    fixture.wait_workers(&send, true);
    assert_transport_identity(&fixture, &send, &call, "failure held through quiescence");
    assert_eq!(port.calls(), vec![call]);
    assert!(port.effects().is_empty());
    fixture.assert_retained_failure(&send,
        "Couldn't deliver that message to Slack: transport rejected the channel message");
    fixture.close();
}

#[test]
fn channel_send_host_lane_unregistered_provider_uses_existing_failure_owner() {
    let mut fixture = ChannelHostLaneFixture::new("unregistered");
    assert_eq!(configured_routed_provider(&fixture.root.join("settings.json")), None);
    assert!(!fixture.deps().transcript_runtime.is_quiescing_for_upgrade());
    let send = fixture.send(text_message("unregistered channel must retain failure"), true);
    fixture.wait_workers(&send, true);
    fixture.assert_retained_failure(&send,
        "Channel messaging isn't available on this computer, so that message wasn't delivered.");
    fixture.close();
}

#[test]
fn channel_send_host_lane_returns_before_transport_and_preserves_fallback() {
    let mut fixture = ChannelHostLaneFixture::new("held-success");
    let (port, entered) = fixture.register_transport(TransportOutcome::Delivered, true);
    let send = fixture.send(text_message("transport is still held"), true);
    let call = entered.recv_timeout(WAIT).expect("actual held external port");
    assert!(port.effects().is_empty(), "queued acknowledgement precedes the transport effect");
    fixture.assert_no_worker_observation();
    let platform = fixture.request("host.platform").expect("unrelated real Host fallback stays available");
    assert!(platform["platform"].as_str().is_some_and(|value| !value.is_empty()));
    let unknown = "channel.fixture.unknown";
    assert!(matches!(fixture.request(unknown), Err(GatewayCommandError::UnknownMethod(method)) if method == unknown));
    assert!(port.effects().is_empty(), "both fallback requests returned before transport release");
    fixture.assert_no_worker_observation();
    port.release();
    fixture.wait_workers(&send, false);
    assert_transport_identity(&fixture, &send, &call, "transport is still held");
    assert_eq!(port.calls(), vec![call.clone()]);
    assert_eq!(port.effects(), vec![call]);
    fixture.assert_unchanged_append_and_no_followup(&send);
    fixture.assert_no_failure();
    fixture.close();
}

#[test]
fn channel_send_host_lane_preserves_projection_noop_and_no_retry() {
    let mut fixture = ChannelHostLaneFixture::new("noop-unknown-effect");
    let (port, entered) = fixture.register_transport(
        TransportOutcome::EffectThenError("transport acknowledgement lost after effect"), false,
    );
    let noop = fixture.send(text_message(""), false);
    fixture.request("host.platform").expect("lane remains usable after projection no-op");
    fixture.assert_no_worker_observation();
    assert!(port.calls().is_empty());
    assert!(port.effects().is_empty());
    assert!(matches!(entered.try_recv(), Err(mpsc::TryRecvError::Empty)));
    fixture.assert_unchanged_append_and_no_followup(&noop);
    fixture.assert_no_failure();

    assert_eq!(configured_routed_provider(&fixture.root.join("settings.json")), None);
    let send = fixture.send(text_message("effect exists despite the error"), true);
    let call = entered.recv_timeout(WAIT).expect("actual effect-then-error transport");
    fixture.wait_workers(&send, true);
    fixture.request("host.platform").expect("lane survives the failed transport");
    assert_transport_identity(&fixture, &send, &call, "effect exists despite the error");
    assert_eq!(port.calls(), vec![call.clone()], "an ambiguous transport error must not trigger retransmission");
    assert_eq!(port.effects(), vec![call], "one external effect only");
    assert!(matches!(entered.try_recv(), Err(mpsc::TryRecvError::Empty)));
    fixture.assert_no_worker_observation();
    fixture.assert_retained_failure(&send,
        "Couldn't deliver that message to Slack: transport acknowledgement lost after effect");
    assert_ne!(send.entry_id, noop.entry_id);
    assert_eq!(send.entries.len(), noop.entries.len() + 1);
    fixture.close();
}
