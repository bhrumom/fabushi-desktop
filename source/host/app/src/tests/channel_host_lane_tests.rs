//! Causal shipping-path oracles for OCRM-CHANNEL-HOST-LANE-DELIVERY-v1/CWR-v1.
//!
//! Session, the sink, lane dispatcher, delivery router and failure owners are
//! real. Only the external ChannelDeliveryPort chooses a transport outcome.
//! SGF-v1 additionally drives the actual generated Runner and private App sink
//! with controlled Responses, preserving canonical wire and Journal projection.

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
use mahayana_host_runtime::extensions::transcript::sand_pending_wake_store::{
    AutomationQuietOrigin, QuietWakeOrigin, SandPendingWakeStore,
};
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

pub(super) struct IdleRunnerOwners {
    pub(super) deps: LocalRoutedRunnerDeps,
    box_store: ProductionBoxStoreSyncService,
    _telemetry: Arc<HostTelemetryService>,
}

impl IdleRunnerOwners {
    pub(super) fn dispose(&self, agent_id: &str) {
        self.deps.cloud_agent_watches.dispose_parent(agent_id);
        self.deps.background_shell_watches.dispose_parent(agent_id);
        self.deps.auto_review.stop();
        self.deps.production_action_auditor.stop();
        self.deps.cloud_agents.dispose();
        self.deps.host_runner_composition.dispose();
        self.deps.forever_box.dispose();
        self.box_store.dispose();
        self.deps.transcript_manager.dispose();
    }
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

pub(super) fn idle_runner_owners(
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
    roster: Arc<ProductionRosterEmit>,
    router: ChannelDeliveryRouter,
    ports: Vec<Arc<ControlledChannelPort>>,
    lane_observations: mpsc::Receiver<LaneObservation>,
    completions: mpsc::Receiver<ChannelDeliveryWorkerFinished>,
    completion_tx: mpsc::Sender<ChannelDeliveryWorkerFinished>,
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
        Self::open(label, root, None)
    }

    fn open(label: &str, root: PathBuf, existing_agent_id: Option<String>) -> Self {
        let (host_tx, host_rx) = mpsc::channel();
        let events = GatewayEventHub::default();
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let agent_id = existing_agent_id.unwrap_or_else(|| {
            sessions.materialize_new_session(None, "user", None).expect("real Session").id
        });
        let admission = sessions.read_session_work_admission(&agent_id)
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
            agent_id: agent_id.clone(),
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
            roster: Arc::clone(&roster),
            wakes: deps.transcript_manager.background_wakes(),
            completion: Some(completion_tx.clone()),
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
            root, agent_id, session_key: admission.session_key, host_tx,
            owners: Some(owners), sink: Some(sink), roster, router, ports: Vec::new(),
            lane_observations, completions, completion_tx, lifecycle_events,
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
                    assert!(failure, "this delivery must not start failure revival");
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
        self.assert_no_followup(&send.entries);
    }

    fn assert_no_followup(&self, entries: &[Value]) {
        assert_eq!(self.entries().as_slice(), entries, "delivery must not append again");
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
        assert!(wakes.retained_channel_failures(&self.agent_id).is_none());
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
        let failures = wakes.retained_channel_failures(&self.agent_id)
            .expect("deferred failure remains the retained head");
        assert_eq!(failures, &[json!({
            "addressToken": ADDRESS,
            "deliveryOperationId": send.entry_id,
            "markedAtMs": marker.marked_at_ms,
            "sessionKey": self.session_key,
            "reason": reason,
            "pendingWakeMarker": marker,
        })]);
        assert_eq!(wakes.channel_failure_state(&self.agent_id), Some(
            mahayana_host_runtime::extensions::transcript::background_wakes::ChannelFailureState::PausedBeforeDispatch,
        ));
        assert!(!wakes.pending_channel_failures.contains_key(&self.agent_id),
            "the original failure is retained once, with no raw tail or duplicate");
        assert!(!wakes.reviving_channel_failure_agent_ids.contains(&self.agent_id),
            "actual deferred worker has ended revival");
    }

    fn pending_store(&self) -> SandPendingWakeStore {
        self.deps().transcript_runtime.pending_wake_store()
            .expect("actual durable pending-wake owner").clone()
    }

    fn recovery_marker(&self, operation: &str) -> DurablePendingWakeMarker {
        DurablePendingWakeMarker {
            agent_id: self.agent_id.clone(),
            kind: PendingWakeKind::ChannelDelivery,
            work_id: operation.into(),
            marked_at_ms: 456_789.125,
            quiet_origin: Some(QuietWakeOrigin {
                automation: Some(AutomationQuietOrigin {
                    id: "channel-recovery-automation".into(),
                    name: "  Quiet channel wake  ".into(),
                }),
            }),
            title: Some("  Retained channel failure title  ".into()),
            subagent_type: Some("channel-recovery-origin".into()),
            interrupted_by_recreate: true,
            delivery_operation_id: Some(operation.into()),
            address_token: Some(ADDRESS.into()),
            failure_reason: Some("  Original delivery failure detail  ".into()),
            session_key: Some(self.session_key.clone()),
        }
    }

    fn queue_recovered_failure(&self, marker: &DurablePendingWakeMarker) -> Result<(), String> {
        queue_recovered_channel_delivery_failure(
            self.deps().clone(), Arc::clone(&self.roster),
            self.deps().transcript_manager.background_wakes(), marker,
            Some(self.completion_tx.clone()),
        )
    }

    fn recover_and_wait(&mut self, marker: &DurablePendingWakeMarker) {
        assert!(self.workers_settled);
        self.workers_settled = false;
        self.queue_recovered_failure(marker).expect("actual recovery admission");
        self.wait_revival_worker();
    }

    fn claim_failure_revival(&self) {
        assert!(self.workers_settled);
        let wakes = self.deps().transcript_manager.background_wakes();
        let mut wakes = wakes.lock().expect("actual BackgroundWakes owner");
        // Hold the existing drain between batches so queue admission and
        // durable replacement are deterministic. Reserve its actual ticket
        // before launching the shipping worker below.
        assert!(BackgroundWakes::<Value>::begin_revival(
            &mut wakes.reviving_channel_failure_agent_ids, &self.agent_id,
        ));
    }

    fn run_claimed_failure_revival(&mut self) {
        let wakes = self.deps().transcript_manager.background_wakes();
        let work = wakes.lock().expect("actual queue claim owner")
            .take_channel_failure_work_or_end_revival(&self.agent_id)
            .expect("reserve the real batch before spawning");
        self.run_failure_ticket(work);
    }

    fn run_failure_ticket(&mut self, work: ChannelFailureWork<Value>) {
        assert!(self.workers_settled);
        self.workers_settled = false;
        assert!(launch_channel_failure_worker(
            self.deps().clone(), Arc::clone(&self.roster),
            self.deps().transcript_manager.background_wakes(), self.agent_id.clone(), work,
            Some(self.completion_tx.clone()),
        ).expect("actual failure-revival worker spawn"), "the reserved ticket must still own admission");
        self.wait_revival_worker();
    }

    fn wait_revival_worker(&mut self) {
        match self.completions.recv_timeout(WAIT).expect("actual recovery worker completion") {
            ChannelDeliveryWorkerFinished::FailureRevival { agent_id } => {
                assert_eq!(agent_id, self.agent_id);
            }
            ChannelDeliveryWorkerFinished::Delivery { .. } => {
                panic!("recovering a persisted failure must not retry transport");
            }
        }
        self.workers_settled = true;
        self.assert_no_worker_observation();
    }

    fn queued_failures(&self) -> Vec<Value> {
        let wakes = self.deps().transcript_manager.background_wakes();
        let wakes = wakes.lock().expect("actual failure queue");
        // Read-only ordered custody view; this never acquires or settles work.
        let mut failures = wakes.retained_channel_failures(&self.agent_id)
            .unwrap_or(&[]).to_vec();
        failures.extend(wakes.pending_channel_failures.get(&self.agent_id)
            .into_iter().flatten().cloned());
        failures
    }

    fn assert_failure_custody(&self, phase: ChannelFailureState, head: &[Value], tail: &[Value]) {
        let wakes = self.deps().transcript_manager.background_wakes();
        let wakes = wakes.lock().expect("actual retained head and pending tail");
        assert_eq!(wakes.channel_failure_state(&self.agent_id), Some(phase));
        assert_eq!(wakes.retained_channel_failures(&self.agent_id), Some(head));
        assert_eq!(wakes.pending_channel_failures.get(&self.agent_id)
            .map(Vec::as_slice).unwrap_or(&[]), tail);
    }

    fn assert_failure_queue_idle(&self) {
        assert!(self.workers_settled);
        let wakes = self.deps().transcript_manager.background_wakes();
        let wakes = wakes.lock().expect("actual failure revival owner");
        assert!(wakes.reviving_channel_failure_agent_ids.is_empty());
        if let Some(phase) = wakes.channel_failure_state(&self.agent_id) {
            assert_eq!(phase, ChannelFailureState::PausedBeforeDispatch,
                "these actual missing-provider/read-refusal workers must never enter Runner");
        }
        self.assert_no_worker_observation();
    }

    fn assert_recovery_refused(&self, marker: &DurablePendingWakeMarker) {
        self.queue_recovered_failure(marker).expect_err("recovery must refuse admission");
        let wakes = self.deps().transcript_manager.background_wakes();
        let owner = wakes.lock().expect("actual failure queue");
        assert!(owner.pending_channel_failures.is_empty());
        assert!(owner.retained_channel_failures(&self.agent_id).is_none());
        drop(owner);
        self.assert_failure_queue_idle();
    }

    fn assert_recovered_marker_retained(&self, marker: &DurablePendingWakeMarker, bytes: &[u8]) {
        assert_eq!(fs::read(self.pending_store().file_path()).expect("retained durable bytes"), bytes);
        assert_eq!(SandPendingWakeStore::new(&self.root).list_pending_checked()
            .expect("fresh checked marker read"), vec![marker.clone()]);
        assert_eq!(self.queued_failures(), vec![expected_recovered_failure(marker)]);
        self.assert_failure_custody(ChannelFailureState::PausedBeforeDispatch,
            &[expected_recovered_failure(marker)], &[]);
        self.assert_failure_queue_idle();
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
            owners.dispose(&self.agent_id);
        }
    }

    fn stop_for_reopen(mut self) -> (PathBuf, String) {
        assert!(self.workers_settled, "workers must finish before owner disposal");
        self.assert_no_worker_observation();
        self.stop_lane();
        assert!(self.lane_exited && !self.lane_panicked, "owned Host lane shutdown deadline");
        self.dispose_owners();
        (self.root.clone(), self.agent_id.clone())
    }

    fn close(self) {
        let (root, _) = self.stop_for_reopen();
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

fn expected_recovered_failure(marker: &DurablePendingWakeMarker) -> Value {
    json!({
        "addressToken": marker.address_token,
        "deliveryOperationId": marker.delivery_operation_id,
        "markedAtMs": marker.marked_at_ms,
        "sessionKey": marker.session_key,
        "reason": marker.failure_reason,
        "pendingWakeMarker": marker,
    })
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

#[test]
fn channel_send_host_lane_retains_exact_head_before_new_failure_tail() {
    let mut fixture = ChannelHostLaneFixture::new("retained-head-and-tail");
    let (port, entered) = fixture.register_transport(
        TransportOutcome::EffectThenError("acknowledgement lost after effect"), false,
    );
    let first = fixture.send(text_message("first explicit send"), true);
    let first_call = entered.recv_timeout(WAIT).expect("first actual transport call");
    fixture.wait_workers(&first, true);
    assert_transport_identity(&fixture, &first, &first_call, "first explicit send");
    fixture.assert_retained_failure(&first,
        "Couldn't deliver that message to Slack: acknowledgement lost after effect");
    let original_marker = SandPendingWakeStore::new(&fixture.root).list_pending()[0].clone();
    let wakes = fixture.deps().transcript_manager.background_wakes();
    let original_head = wakes.lock().unwrap().retained_channel_failures(&fixture.agent_id).unwrap().to_vec();

    let second = fixture.send(text_message("second explicit send"), true);
    let second_call = entered.recv_timeout(WAIT).expect("second actual transport call");
    fixture.wait_workers(&second, true);
    assert_transport_identity(&fixture, &second, &second_call, "second explicit send");
    assert_ne!(first.entry_id, second.entry_id);
    assert_eq!(port.calls(), vec![first_call.clone(), second_call.clone()],
        "each explicit send reaches transport once, with no retry of the ambiguous first effect");
    assert_eq!(port.effects(), vec![first_call, second_call]);
    assert!(matches!(entered.try_recv(), Err(mpsc::TryRecvError::Empty)));
    fixture.assert_unchanged_append_and_no_followup(&second);
    assert_eq!(second.entries.len(), first.entries.len() + 1);
    let markers = SandPendingWakeStore::new(&fixture.root).list_pending();
    assert_eq!(markers.len(), 2);
    assert!(markers.contains(&original_marker), "the retained head keeps its original durable identity");
    let tail_marker = markers.iter().find(|marker| marker.work_id == second.entry_id)
        .expect("new delivery failure has its own exact marker");
    let owner = wakes.lock().expect("actual retained failure owner");
    assert_eq!(owner.retained_channel_failures(&fixture.agent_id), Some(original_head.as_slice()));
    assert_eq!(owner.pending_channel_failures.get(&fixture.agent_id), Some(&vec![json!({
        "addressToken": ADDRESS,
        "deliveryOperationId": second.entry_id,
        "markedAtMs": tail_marker.marked_at_ms,
        "sessionKey": fixture.session_key,
        "reason": "Couldn't deliver that message to Slack: acknowledgement lost after effect",
        "pendingWakeMarker": tail_marker,
    })]));
    assert_eq!(owner.channel_failure_state(&fixture.agent_id), Some(
        mahayana_host_runtime::extensions::transcript::background_wakes::ChannelFailureState::PausedBeforeDispatch,
    ));
    assert!(!owner.reviving_channel_failure_agent_ids.contains(&fixture.agent_id));
    drop(owner);
    fixture.close();
}

#[test]
fn channel_send_host_lane_failure_after_manager_close_preserves_durable_marker() {
    let mut fixture = ChannelHostLaneFixture::new("transport-fails-after-manager-close");
    fs::write(fixture.root.join("settings.json"), br#"{"inferenceProvider":"fabushi"}"#)
        .expect("configure the real provider gate before transport admission");
    assert_eq!(configured_routed_provider(&fixture.root.join("settings.json")), Some(RoutedProvider::Fabushi));
    let (port, entered) = fixture.register_transport(
        TransportOutcome::EffectThenError("acknowledgement lost after manager close"), true,
    );
    let send = fixture.send(text_message("transport admitted before manager close"), true);
    let call = entered.recv_timeout(WAIT).expect("actual transport passed Session admission and is held");
    assert_transport_identity(&fixture, &send, &call, "transport admitted before manager close");
    assert_eq!(port.calls(), vec![call.clone()]);
    assert!(port.effects().is_empty());
    fixture.assert_no_worker_observation();
    let store = fixture.pending_store();
    assert!(store.list_pending_checked().expect("pending state before transport outcome").is_empty());
    let wakes = fixture.deps().transcript_manager.background_wakes();

    // Only the manager is disposed. The already admitted external operation
    // remains held, and its actual error will arrive after the queue closes.
    fixture.deps().transcript_manager.dispose();
    assert!(fixture.deps().transcript_manager.is_disposed());
    assert!(wakes.lock().unwrap().is_closed());
    assert!(store.list_pending_checked().expect("closing is not a delivery outcome").is_empty());
    assert!(port.effects().is_empty());
    fixture.assert_no_worker_observation();
    port.release();
    // The actual delivery closure reports completion after marker publication
    // and closed-owner enqueue refusal. There must be no revival worker.
    fixture.wait_workers(&send, false);
    assert_eq!(port.calls(), vec![call.clone()], "the ambiguous late failure must not retry transport");
    assert_eq!(port.effects(), vec![call]);
    assert!(matches!(entered.try_recv(), Err(mpsc::TryRecvError::Empty)));

    let reason = "Couldn't deliver that message to Slack: acknowledgement lost after manager close";
    let markers = SandPendingWakeStore::new(&fixture.root).list_pending_checked()
        .expect("fresh reader observes the late failure's durable custody");
    assert!(store.file_path().is_file());
    assert_eq!(markers, store.list_pending_checked().expect("same actual pending owner"));
    assert_eq!(markers.len(), 1);
    assert!(markers[0].marked_at_ms.is_finite() && markers[0].marked_at_ms > 0.0);
    assert_eq!(markers[0], DurablePendingWakeMarker {
        agent_id: fixture.agent_id.clone(),
        kind: PendingWakeKind::ChannelDelivery,
        work_id: send.entry_id.clone(),
        marked_at_ms: markers[0].marked_at_ms,
        quiet_origin: None,
        title: Some("Channel delivery failure".into()),
        subagent_type: None,
        interrupted_by_recreate: false,
        delivery_operation_id: Some(send.entry_id.clone()),
        address_token: Some(ADDRESS.into()),
        failure_reason: Some(reason.into()),
        session_key: Some(fixture.session_key.clone()),
    });
    let bytes = fs::read(store.file_path()).expect("actual durable marker bytes");
    let mut owner = wakes.lock().expect("closed owner after actual delivery completion");
    assert!(owner.is_closed());
    assert!(owner.pending_channel_failures.is_empty());
    assert!(owner.retained_channel_failures(&fixture.agent_id).is_none());
    assert!(owner.channel_failure_state(&fixture.agent_id).is_none());
    assert!(owner.reviving_channel_failure_agent_ids.is_empty());
    assert!(owner.request_channel_failure_readiness().is_empty());
    drop(owner);
    assert_eq!(fs::read(store.file_path()).expect("closed readiness retains custody"), bytes);
    assert_eq!(fixture.deps().runner_registry.active_count(), 0);
    assert_eq!(fixture.deps().transcript_runtime.in_flight_run_count(&fixture.agent_id), 0);
    assert!(fixture.deps().transcript_runtime.is_turn_dispatch_idle(&fixture.agent_id));
    let trays = fixture.deps().trays.list();
    assert_eq!(trays.len(), 1);
    assert_eq!(trays[0].title, "Message not delivered");
    assert_eq!(trays[0].agent_id.as_deref(), Some(fixture.agent_id.as_str()));
    assert_eq!(trays[0].raw_detail.as_deref(), Some(reason));

    // Manager disposal closed the original Session owners. Reopen the real
    // Session facade to observe exact persisted rows without reopening wakes.
    let reader = ProductionSessionWorkers::with_agents_root(fixture.root.join("agents"), 500);
    assert_eq!(reader.read_agent_transcript_entries(&fixture.agent_id)
        .expect("actual reopened Session transcript"), send.entries,
        "late delivery failure must not duplicate or rewrite the original append");
    reader.shutdown();
    fixture.assert_no_worker_observation();
    fixture.close();
}

#[test]
fn channel_send_host_lane_persistence_refusal_retains_live_failure_without_overwriting_store() {
    for unreadable_path in [false, true] {
        let label = if unreadable_path { "live-failure-unreadable" } else { "live-failure-corrupt" };
        let mut fixture = ChannelHostLaneFixture::new(label);
        assert_eq!(configured_routed_provider(&fixture.root.join("settings.json")), None);
        assert!(!fixture.deps().transcript_runtime.is_quiescing_for_upgrade());
        let (port, entered) = fixture.register_transport(
            TransportOutcome::EffectThenError("transport acknowledgement lost after effect"), true,
        );
        let send = fixture.send(text_message("live failure with refused persistence"), true);
        let call = entered.recv_timeout(WAIT).expect("actual transport held before failure persistence");
        assert!(port.effects().is_empty());
        fixture.assert_no_worker_observation();

        let store = fixture.pending_store();
        let corrupt = br#"{"version":1,"pending":[{"agentId":"unrelated-retained-work"},"#;
        let witness = store.file_path().join("retained-witness");
        if unreadable_path {
            fs::create_dir(store.file_path()).expect("make the actual store path unreadable as a file");
            fs::write(&witness, b"unrelated custody must survive the live failure")
                .expect("retain directory fault witness");
        } else {
            fs::write(store.file_path(), corrupt).expect("retain corrupt pending-wake bytes");
        }
        assert!(store.list_pending_checked().is_err(), "the actual store must observe this refusal");
        port.release();
        fixture.wait_workers(&send, true);

        assert_transport_identity(&fixture, &send, &call, "live failure with refused persistence");
        assert_eq!(port.calls(), vec![call.clone()], "persistence refusal must not retry transport");
        assert_eq!(port.effects(), vec![call], "the ambiguous failure retains its single external effect");
        assert!(matches!(entered.try_recv(), Err(mpsc::TryRecvError::Empty)));
        fixture.assert_unchanged_append_and_no_followup(&send);
        let reason = "Couldn't deliver that message to Slack: transport acknowledgement lost after effect";
        let trays = fixture.deps().trays.list();
        assert_eq!(trays.len(), 1);
        assert_eq!(trays[0].title, "Message not delivered");
        assert_eq!(trays[0].agent_id.as_deref(), Some(fixture.agent_id.as_str()));
        assert_eq!(trays[0].raw_detail.as_deref(), Some(reason));
        let failures = fixture.queued_failures();
        assert_eq!(failures.len(), 1, "fresh unpersisted failure retains volatile custody");
        let marked_at_ms = failures[0]["markedAtMs"].as_f64().expect("original live failure timestamp");
        assert!(marked_at_ms.is_finite() && marked_at_ms > 0.0);
        assert_eq!(failures, vec![json!({
            "addressToken": ADDRESS,
            "deliveryOperationId": send.entry_id,
            "markedAtMs": marked_at_ms,
            "sessionKey": fixture.session_key,
            "reason": reason,
        })], "retain the exact legacy five-key payload without fabricating durable admission");
        assert!(failures[0].get("pendingWakeMarker").is_none());
        fixture.assert_failure_custody(ChannelFailureState::PausedBeforeDispatch, &failures, &[]);
        fixture.assert_failure_queue_idle();
        assert!(SandPendingWakeStore::new(&fixture.root).list_pending_checked().is_err(),
            "failure handling must not replace an unknown image with a readable marker");
        if unreadable_path {
            assert!(store.file_path().is_dir());
            assert_eq!(fs::read(&witness).expect("directory witness remains untouched"),
                b"unrelated custody must survive the live failure");
        } else {
            assert_eq!(fs::read(store.file_path()).expect("corrupt bytes remain untouched"), corrupt);
        }
        fixture.close();
    }
}

#[test]
fn channel_recovery_reopens_actual_owners_and_retains_exact_failure_without_provider() {
    let mut original = ChannelHostLaneFixture::new("recovery-original");
    let send = original.send(text_message("persist before recovery restart"), true);
    original.wait_workers(&send, true);
    original.assert_retained_failure(&send,
        "Channel messaging isn't available on this computer, so that message wasn't delivered.");
    let marker = original.recovery_marker(&send.entry_id);
    let store = original.pending_store();
    store.mark_pending_checked(marker.clone()).expect("persist original complete metadata");
    let bytes = fs::read(store.file_path()).expect("original durable image");
    let session_key = original.session_key.clone();
    drop(store);
    let (root, agent_id) = original.stop_for_reopen();

    let mut reopened = ChannelHostLaneFixture::open("recovery-reopened", root, Some(agent_id));
    assert_eq!(reopened.session_key, session_key, "reopen the same actual Session");
    assert_eq!(configured_routed_provider(&reopened.root.join("settings.json")), None);
    assert!(reopened.queued_failures().is_empty(), "fresh owners have no inherited memory queue");
    assert_eq!(reopened.pending_store().list_pending_checked().expect("reopened marker"), vec![marker.clone()]);
    let (port, entered) = reopened.register_transport(TransportOutcome::Delivered, false);
    reopened.recover_and_wait(&marker);
    reopened.assert_recovered_marker_retained(&marker, &bytes);
    reopened.assert_unchanged_append_and_no_followup(&send);
    assert!(reopened.deps().trays.list().is_empty(), "recovery does not manufacture a new failure");
    assert!(port.calls().is_empty(), "failure recovery must not retry the original delivery");
    assert!(port.effects().is_empty());
    assert!(matches!(entered.try_recv(), Err(mpsc::TryRecvError::Empty)));
    reopened.close();
}

#[test]
fn channel_recovery_quiescence_retains_original_marker_and_queue() {
    let mut fixture = ChannelHostLaneFixture::new("recovery-quiescence");
    let entries = fixture.entries();
    let marker = fixture.recovery_marker("quiesced-recovery-operation");
    let store = fixture.pending_store();
    store.mark_pending_checked(marker.clone()).expect("persist before upgrade pause");
    let bytes = fs::read(store.file_path()).expect("pre-recovery image");
    fs::write(fixture.root.join("settings.json"), br#"{"inferenceProvider":"fabushi"}"#)
        .expect("actual configured-provider settings");
    assert_eq!(configured_routed_provider(&fixture.root.join("settings.json")), Some(RoutedProvider::Fabushi));
    fixture.deps().transcript_runtime.quiesce_for_upgrade().expect("actual runtime quiescence");
    assert!(fixture.deps().transcript_runtime.is_quiescing_for_upgrade());

    fixture.recover_and_wait(&marker);
    fixture.assert_recovered_marker_retained(&marker, &bytes);
    fixture.assert_no_followup(&entries);
    assert!(fixture.deps().trays.list().is_empty());
    fixture.close();
}

#[test]
fn channel_recovery_repeated_enqueue_coalesces_and_preserves_batch_order() {
    let mut fixture = ChannelHostLaneFixture::new("recovery-coalescing");
    let entries = fixture.entries();
    let first = fixture.recovery_marker("first-retained-operation");
    let mut second = fixture.recovery_marker("second-retained-operation");
    second.marked_at_ms += 17.5;
    second.session_key = None;
    let store = fixture.pending_store();
    store.mark_pending_checked(first.clone()).expect("first durable failure");
    store.mark_pending_checked(second.clone()).expect("second durable failure");
    let bytes = fs::read(store.file_path()).expect("two-marker image");
    let expected = vec![expected_recovered_failure(&first), expected_recovered_failure(&second)];
    fixture.claim_failure_revival();
    for marker in [&first, &first, &second, &first, &second] {
        fixture.queue_recovered_failure(marker).expect("repeated recovery admission");
    }
    assert_eq!(fixture.queued_failures(), expected, "coalesce before the worker can drain");
    let wakes = fixture.deps().transcript_manager.background_wakes();
    let claimed = wakes.lock().expect("actual queue batch owner")
        .take_channel_failure_work_or_end_revival(&fixture.agent_id)
        .expect("reserve the admitted batch under the existing revival bit");
    assert_eq!(claimed.failures, expected);
    for marker in [&second, &first] {
        fixture.queue_recovered_failure(marker).expect("duplicates admitted while batch is held");
    }
    // Exact duplicates remain in their retained head, rather than being
    // reinserted into the raw tail. Their hint is consumed by the real worker.
    fixture.assert_failure_custody(ChannelFailureState::ActiveBeforeDispatch, &expected, &[]);
    assert!(wakes.lock().expect("legacy raw-take observation")
        .take_pending_channel_failures_or_end_revival(&fixture.agent_id).is_none(),
        "raw compatibility take cannot acquire a live retained head");
    assert!(wakes.lock().unwrap().owns_channel_failure_work(&fixture.agent_id, &claimed));
    assert_eq!(fixture.queued_failures(), expected, "live custody preserves original order without duplicates");
    fixture.assert_no_worker_observation();
    fixture.run_failure_ticket(claimed);
    assert_eq!(fixture.queued_failures(), expected, "real missing-provider pause preserves order");
    fixture.assert_failure_custody(ChannelFailureState::PausedBeforeDispatch, &expected, &[]);
    fixture.assert_failure_queue_idle();

    fixture.recover_and_wait(&first);
    assert_eq!(fixture.queued_failures(), expected, "a later rearm must not duplicate retained work");
    fixture.assert_failure_custody(ChannelFailureState::PausedBeforeDispatch, &expected, &[]);
    assert_eq!(fs::read(store.file_path()).expect("retained image"), bytes);
    assert_eq!(SandPendingWakeStore::new(&fixture.root).list_pending_checked()
        .expect("fresh reader sees both markers"), vec![first, second]);
    fixture.assert_failure_queue_idle();
    fixture.assert_no_followup(&entries);
    fixture.close();
}

#[test]
fn channel_recovery_cleared_or_replaced_snapshot_cannot_resurrect() {
    let fixture = ChannelHostLaneFixture::new("recovery-stale-admission");
    let entries = fixture.entries();
    let marker = fixture.recovery_marker("observed-recovery-operation");
    let store = fixture.pending_store();

    fixture.assert_recovery_refused(&marker);
    assert!(!store.file_path().exists(), "absent custody cannot be manufactured by recovery");
    let unrelated = fixture.recovery_marker("unrelated-retained-operation");
    store.mark_pending_checked(unrelated.clone()).expect("preserve unrelated durable custody");
    store.mark_pending_checked(marker.clone()).expect("persist observed marker");
    assert!(store.clear_if_matches(&marker), "settle the exact observed snapshot first");
    let cleared = fs::read(store.file_path()).expect("image after settling only the observed marker");
    fixture.assert_recovery_refused(&marker);
    assert_eq!(fs::read(store.file_path()).expect("still cleared"), cleared);
    assert_eq!(store.list_pending_checked().expect("cleared image"), vec![unrelated.clone()]);

    let mut replacement = marker.clone();
    replacement.marked_at_ms += 1.0;
    replacement.title = Some("Newer same-operation failure".into());
    store.mark_pending_checked(replacement.clone()).expect("persist newer identity");
    let replaced = fs::read(store.file_path()).expect("replacement bytes");
    fixture.assert_recovery_refused(&marker);
    assert_eq!(fs::read(store.file_path()).expect("unmodified replacement"), replaced);
    assert_eq!(SandPendingWakeStore::new(&fixture.root).list_pending_checked()
        .expect("fresh replacement read"), vec![unrelated, replacement]);
    fixture.assert_no_followup(&entries);
    fixture.close();
}

#[test]
fn channel_recovery_refuses_incomplete_context_without_mutating_durable_bytes() {
    let fixture = ChannelHostLaneFixture::new("recovery-incomplete");
    let entries = fixture.entries();
    let complete = fixture.recovery_marker("incomplete-recovery-operation");
    let mut missing_operation = complete.clone();
    missing_operation.delivery_operation_id = None;
    let mut mismatched_operation = complete.clone();
    mismatched_operation.delivery_operation_id = Some("another-operation".into());
    let mut missing_address = complete.clone();
    missing_address.address_token = None;
    let mut blank_address = complete.clone();
    blank_address.address_token = Some("  ".into());
    let mut missing_reason = complete.clone();
    missing_reason.failure_reason = None;
    let mut blank_reason = complete.clone();
    blank_reason.failure_reason = Some("\t".into());
    let mut blank_session = complete.clone();
    blank_session.session_key = Some(" ".into());
    let store = fixture.pending_store();
    for marker in [missing_operation, mismatched_operation, missing_address, blank_address,
        missing_reason, blank_reason, blank_session] {
        // An already-written incomplete record is retained evidence, not
        // permission to default its operation or delivery context.
        let bytes = serde_json::to_vec(&json!({"version": 1, "pending": [marker]}))
            .expect("incomplete persisted marker bytes");
        fs::write(store.file_path(), &bytes).expect("write exact incomplete snapshot");
        fixture.assert_recovery_refused(&marker);
        assert_eq!(fs::read(store.file_path()).expect("preserved incomplete bytes"), bytes);
    }
    fixture.assert_no_followup(&entries);
    fixture.close();
}

#[test]
fn channel_recovery_corrupt_or_unreadable_store_refuses_enqueue() {
    let fixture = ChannelHostLaneFixture::new("recovery-unknown-store");
    let entries = fixture.entries();
    let marker = fixture.recovery_marker("unknown-store-operation");
    let store = fixture.pending_store();
    let corrupt = br#"{"version":1,"pending":[{"agentId":"unrelated-retained-work"},"#;
    fs::write(store.file_path(), corrupt).expect("inject malformed whole image");
    assert!(store.list_pending_checked().is_err(), "fault must reach the actual store reader");
    fixture.assert_recovery_refused(&marker);
    assert_eq!(fs::read(store.file_path()).expect("corrupt bytes remain"), corrupt);

    fs::remove_file(store.file_path()).expect("replace only this fixture's corrupt image");
    fs::create_dir(store.file_path()).expect("deterministic unreadable file path");
    let witness = store.file_path().join("retained-witness");
    fs::write(&witness, b"must survive the store read refusal").expect("unreadable-path witness");
    assert!(store.list_pending_checked().is_err(), "a directory is not a missing image");
    fixture.assert_recovery_refused(&marker);
    assert!(store.file_path().is_dir());
    assert_eq!(fs::read(witness).expect("unreadable image preserved"), b"must survive the store read refusal");
    fixture.assert_no_followup(&entries);
    fixture.close();
}

#[test]
fn channel_recovery_closed_wakes_refuses_admission_without_retiring_marker() {
    let fixture = ChannelHostLaneFixture::new("recovery-closed-wakes");
    let entries = fixture.entries();
    let marker = fixture.recovery_marker("closed-owner-operation");
    let store = fixture.pending_store();
    store.mark_pending_checked(marker.clone()).expect("persist before wake owner closes");
    let bytes = fs::read(store.file_path()).expect("retained image");
    fixture.deps().transcript_manager.background_wakes().lock()
        .expect("actual BackgroundWakes owner").close();

    fixture.assert_recovery_refused(&marker);
    assert_eq!(fs::read(store.file_path()).expect("closed owner preserves bytes"), bytes);
    assert_eq!(SandPendingWakeStore::new(&fixture.root).list_pending_checked()
        .expect("marker remains recoverable by a fresh owner"), vec![marker]);
    fixture.assert_no_followup(&entries);
    fixture.close();
}

#[test]
fn channel_recovery_worker_suppresses_cleared_or_replaced_queued_snapshots() {
    let mut fixture = ChannelHostLaneFixture::new("recovery-stale-batch");
    let entries = fixture.entries();
    let cleared = fixture.recovery_marker("queued-then-cleared");
    let observed = fixture.recovery_marker("queued-then-replaced");
    let store = fixture.pending_store();
    store.mark_pending_checked(cleared.clone()).expect("persist first queued marker");
    store.mark_pending_checked(observed.clone()).expect("persist second queued marker");
    fixture.claim_failure_revival();
    fixture.queue_recovered_failure(&cleared).expect("first actual queued snapshot");
    fixture.queue_recovered_failure(&observed).expect("second actual queued snapshot");
    assert_eq!(fixture.queued_failures(), vec![
        expected_recovered_failure(&cleared), expected_recovered_failure(&observed),
    ]);
    assert!(store.clear_if_matches(&cleared));
    let mut replacement = observed.clone();
    replacement.marked_at_ms += 2.0;
    replacement.failure_reason = Some("A newer failure owns this operation".into());
    store.mark_pending_checked(replacement.clone()).expect("replace after enqueue, before drain");
    let bytes = fs::read(store.file_path()).expect("replacement snapshot");
    fs::write(fixture.root.join("settings.json"), br#"{"inferenceProvider":"fabushi"}"#)
        .expect("configure provider so the identity guard prevents admission");
    assert_eq!(configured_routed_provider(&fixture.root.join("settings.json")), Some(RoutedProvider::Fabushi));
    assert!(!fixture.deps().transcript_runtime.is_quiescing_for_upgrade());

    fixture.run_claimed_failure_revival();
    assert!(fixture.queued_failures().is_empty(), "both stale snapshots are consumed without replay");
    fixture.assert_failure_queue_idle();
    assert_eq!(fs::read(store.file_path()).expect("new marker unchanged"), bytes);
    assert_eq!(SandPendingWakeStore::new(&fixture.root).list_pending_checked()
        .expect("replacement remains pending"), vec![replacement]);
    fixture.assert_no_followup(&entries);
    assert!(fixture.deps().trays.list().is_empty(), "no provider-failure attempt may be hidden");
    fixture.close();
}

#[test]
fn channel_recovery_worker_read_error_retains_original_queued_marker() {
    let mut fixture = ChannelHostLaneFixture::new("recovery-worker-read-error");
    let entries = fixture.entries();
    let marker = fixture.recovery_marker("queued-before-read-error");
    let store = fixture.pending_store();
    store.mark_pending_checked(marker.clone()).expect("persist original worker input");
    fixture.claim_failure_revival();
    fixture.queue_recovered_failure(&marker).expect("admit from the readable snapshot");
    let corrupt = b"pending-wake image became unreadable JSON after enqueue";
    fs::write(store.file_path(), corrupt).expect("fault the store after actual admission");
    assert!(store.list_pending_checked().is_err());

    fixture.run_claimed_failure_revival();
    assert_eq!(fixture.queued_failures(), vec![expected_recovered_failure(&marker)],
        "unknown current identity retains the exact claimed batch");
    fixture.assert_failure_custody(ChannelFailureState::PausedBeforeDispatch,
        &[expected_recovered_failure(&marker)], &[]);
    fixture.assert_failure_queue_idle();
    assert_eq!(fs::read(store.file_path()).expect("failed read cannot rewrite custody"), corrupt);
    fixture.assert_no_followup(&entries);
    assert!(fixture.deps().trays.list().is_empty());
    fixture.close();
}

#[test]
fn channel_recovery_exact_settlement_preserves_newer_marker_and_full_metadata() {
    let mut fixture = ChannelHostLaneFixture::new("recovery-exact-settlement");
    let entries = fixture.entries();
    let observed = fixture.recovery_marker("same-key-settlement-operation");
    let store = fixture.pending_store();
    store.mark_pending_checked(observed.clone()).expect("persist full original metadata");
    fixture.claim_failure_revival();
    fixture.queue_recovered_failure(&observed).expect("queue original settlement identity");
    let original_failures = fixture.queued_failures();
    assert_eq!(original_failures, vec![expected_recovered_failure(&observed)]);
    let mut replacement = observed.clone();
    replacement.marked_at_ms += 3.0;
    replacement.title = Some("Full metadata on the newer marker".into());
    store.mark_pending_checked(replacement.clone()).expect("new marker wins before old settlement");
    let bytes = fs::read(store.file_path()).expect("newer marker bytes");

    // Exercise the real settlement owner directly; this is not a fabricated
    // provider completion or a claim that a follow-up turn sent a message.
    clear_durable_channel_failure_markers(fixture.deps(), &fixture.agent_id, &original_failures).unwrap();
    assert_eq!(fs::read(store.file_path()).expect("old settlement cannot touch replacement"), bytes);
    assert_eq!(store.list_pending_checked().expect("replacement survived"), vec![replacement.clone()]);
    fixture.queue_recovered_failure(&replacement).expect("queue replacement's exact metadata");
    let mut current_failures = fixture.queued_failures();
    filter_current_channel_failure_markers(fixture.deps(), &fixture.agent_id, &mut current_failures)
        .expect("select the still-current queued identity");
    assert_eq!(current_failures, vec![expected_recovered_failure(&replacement)]);
    clear_durable_channel_failure_markers(fixture.deps(), &fixture.agent_id, &current_failures).unwrap();
    assert!(SandPendingWakeStore::new(&fixture.root).list_pending_checked()
        .expect("fresh reader after exact settlement").is_empty(),
        "settlement must preserve optional metadata instead of reconstructing defaults");

    fixture.run_claimed_failure_revival();
    assert!(fixture.queued_failures().is_empty(), "settled queue snapshots cannot execute later");
    fixture.assert_failure_queue_idle();
    fixture.assert_no_followup(&entries);
    fixture.close();
}

// The App owns ProductionSendMessageSink, so this oracle lives beside its
// existing shipping fixture. The Responses adapter, generated Runner, checked
// checkpoint, Session SQL, media custody and Journal remain production code.
mod generated_send_message_fidelity {
    use super::*;
    use super::super::UnusedCompactionProviderPorts;
    use mahayana_host_runtime::extensions::inference::provider_session::{
        ProviderMessage, ProviderSessionError, RoutedProvider,
    };
    use mahayana_host_runtime::extensions::session::agent_db::{
        OriginalTurnReceiptCommit, OriginalTurnReference,
    };
    use mahayana_host_runtime::extensions::session::gateway::persist_accepted_send_prompt_context;
    use mahayana_host_runtime::extensions::session::session_conversation_state::ConversationOutlineItem;
    use mahayana_host_runtime::extensions::transcript::production_runtime::ProductionSendError;
    use mahayana_host_runtime::extensions::transcript::sand_ack_obligation_store::SandAckObligationStore;
    use mahayana_host_runtime::runner::production_agent_checkpoint::{
        AgentStateCheckpointSink, reconcile_completed_original_turn,
    };
    use mahayana_host_runtime::runner::production_turn_agent_owner::{
        ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
        ProductionTurnAgentStaticConfig,
    };
    use mahayana_host_runtime::runner::routed_provider_runtime::{
        ProductionRoutedProviderCheckpointStore, RunnerRequestContextSnapshot,
    };
    use mahayana_host_runtime::runner::subagent_runtime::SubagentRuntime;
    use mahayana_host_runtime::runner::tools::send_message_tool::{
        CountingSendMessageSink, SendMessageDeliveryCounter, SendMessageSink,
    };
    use mahayana_host_runtime::runner::turn_agent_composition::{
        SAND_AGENT_TOKEN_LIMIT, TurnAgentComposition,
    };
    use mahayana_host_runtime::runner::turn_observation::TurnObservation;
    use mahayana_host_runtime::runner::{
        TerminalOutcome, ToolEffectCustodyObservation, ToolEffectCustodyReport,
        ToolEffectCustodySnapshot, TurnRunOptions,
    };
    use mahayana_host_runtime::runner_production_bridge::create_production_runner;
    use mahayana_host_runtime::transcript_mirror::conversation_state_binary::{
        ConversationTurnStructureFields, decode_conversation_state_recovery_fields,
        decode_conversation_turn_structure_fields, decode_user_message_identity_fields,
    };
    use mahayana_host_runtime::transcript_mirror::generated_occurrence_codec::{
        CanonicalGeneratedToolJsonProjection, GeneratedTranscriptOccurrenceCodec,
    };
    use mahayana_host_runtime::transcript_mirror::transcript_occurrence_deriver::{
        DecodedTranscriptStep, TranscriptOccurrenceCodec,
    };
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::process::{Child, Command, ExitStatus, Stdio};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::Instant;

    const CHILD_REQUEST_ENV: &str = "FABUSHI_SEND_MESSAGE_FIDELITY_REQUEST";
    const STREAM: &str = "generated-send-message-app-stream";
    const MODEL: &str = "Vendor/SendMessage/Fidelity@ActualApp";
    const PROMPT: &str = "Send the text and the local attachment once, in that order.";
    const TEXT: &str = "Actual App send: preserved text, Unicode 雪, and\na second line.";
    const ALT: &str = "The normalized local attachment";
    const ATTACHMENT_BYTES: &[u8] = b"actual local attachment custody\n";
    const FINAL_TEXT: &str = "Both local messages were recorded.";
    const PARTIAL_TEXT: &str = "This interrupted continuation is not a final result.";
    const REFUSAL: &str = "controlled permanent refusal after actual App sends";
    const TERMINAL_SLOT: &str = "original-routed-terminal:current";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Case { Observed, Unobserved, ProviderFailure }

    impl Case {
        fn label(self) -> &'static str {
            match self {
                Self::Observed => "observed",
                Self::Unobserved => "unobserved",
                Self::ProviderFailure => "provider-failure",
            }
        }

        fn test_name(self) -> &'static str {
            match self {
                Self::Observed => "tests::channel_host_lane_tests::generated_send_message_fidelity::generated_send_message_app_sink_preserves_live_sql_and_canonical_bytes",
                Self::Unobserved => "tests::channel_host_lane_tests::generated_send_message_fidelity::generated_send_message_app_sink_preserves_history_without_observation",
                Self::ProviderFailure => "tests::channel_host_lane_tests::generated_send_message_fidelity::generated_send_message_app_sink_retains_failed_turn_without_completion_or_replay",
            }
        }
    }

    fn call_id(ordinal: usize) -> String { format!("actual-app-send-{ordinal}") }

    fn arguments(root: &Path, ordinal: usize) -> Value {
        match ordinal {
            0 => json!({"type":"text", "content":TEXT}),
            1 => json!({"type":"attachment", "url":file_url_for_path(
                root.join("attachment-source.txt")).expect("actual local fixture file URL"), "alt":ALT}),
            _ => panic!("unexpected SendMessage ordinal"),
        }
    }

    fn response_events(root: &Path, case: Case, ordinal: usize) -> Result<Vec<Value>, String> {
        match ordinal {
            0 => Ok(vec![json!({"type":"response.completed", "response":{
                "id":"actual-app-send-tools", "output":(0..2).map(|index| json!({
                    "type":"function_call", "name":"SendMessage", "call_id":call_id(index),
                    "arguments":arguments(root, index).to_string(),
                })).collect::<Vec<_>>(),
            }})]),
            1 if case == Case::ProviderFailure => Ok(vec![
                json!({"type":"response.output_text.delta", "delta":PARTIAL_TEXT}),
                json!({"type":"response.failed", "response":{"error":{
                    "code":"misalignment_policy_violation", "misalignment":{
                        "detailed_explanation":REFUSAL,
                        "steer":{"message":"retain the original typed provider refusal"},
                    },
                }}}),
            ]),
            1 => Ok(vec![
                json!({"type":"response.output_text.delta", "delta":FINAL_TEXT}),
                json!({"type":"response.completed", "response":{"id":"actual-app-send-final", "output":[]}}),
            ]),
            _ => Err(format!("unexpected extra Responses request {ordinal}")),
        }
    }

    fn read_request(stream: &mut TcpStream) -> Result<Value, String> {
        // Accepted sockets can inherit a nonblocking listener's mode. Match
        // the existing Responses fixture contract before bounded Read/Write.
        stream.set_nonblocking(false).map_err(|error| format!("request blocking mode: {error}"))?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|error| format!("request timeout: {error}"))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|error| format!("response timeout: {error}"))?;
        let deadline = Instant::now() + Duration::from_secs(4);
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 4096];
        let header_end = loop {
            if Instant::now() >= deadline { return Err("request header deadline".into()); }
            let count = stream.read(&mut buffer).map_err(|error| format!("request headers: {error}"))?;
            if count == 0 { return Err("closed request headers".into()); }
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(index) = bytes.windows(4).position(|part| part == b"\r\n\r\n") { break index + 4; }
            if bytes.len() > 64 * 1024 { return Err("oversized request headers".into()); }
        };
        let headers = std::str::from_utf8(&bytes[..header_end]).map_err(|_| "request header encoding")?;
        if !headers.starts_with("POST /v1/responses HTTP/1.1\r\n") {
            return Err("unexpected Responses request route".into());
        }
        let length = headers.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse::<usize>().ok()).flatten()
        }).ok_or("missing request content length")?;
        if length > 1024 * 1024 { return Err("oversized Responses request".into()); }
        while bytes.len() < header_end + length {
            if Instant::now() >= deadline { return Err("request body deadline".into()); }
            let count = stream.read(&mut buffer).map_err(|error| format!("request body: {error}"))?;
            if count == 0 { return Err("closed request body".into()); }
            bytes.extend_from_slice(&buffer[..count]);
        }
        serde_json::from_slice(&bytes[header_end..header_end + length])
            .map_err(|error| format!("request JSON: {error}"))
    }

    struct Responses {
        endpoint: String,
        credential: PathBuf,
        requests: Arc<Mutex<Vec<Value>>>,
        stop: Arc<AtomicBool>,
        worker: Option<thread::JoinHandle<Result<(), String>>>,
        outcome: Option<Result<(), String>>,
    }

    impl Responses {
        fn new(root: &Path, case: Case) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("owned Responses listener");
            listener.set_nonblocking(true).expect("bounded Responses accept");
            let origin = format!("http://{}", listener.local_addr().unwrap());
            let credential = root.join("responses-credential.json");
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            options.open(&credential).expect("private controlled Responses credential")
                .write_all(json!({"version":1, "issuer":origin,
                    "accessToken":"send-message-fixture-token", "expiresAtMs":4_102_444_800_000_u64,
                    "sessionId":"send-message-fixture-session", "deviceId":"send-message-fixture-device",
                    "userId":"send-message-fixture-account",
                }).to_string().as_bytes()).expect("write fixture credential");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let captured = Arc::clone(&requests);
            let stop = Arc::new(AtomicBool::new(false));
            let worker_stop = Arc::clone(&stop);
            let worker_root = root.to_path_buf();
            let worker = thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(90);
                while !worker_stop.load(Ordering::Acquire) {
                    if Instant::now() >= deadline { return Err("Responses fixture deadline".into()); }
                    let (mut stream, _) = match listener.accept() {
                        Ok(connection) => connection,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(2));
                            continue;
                        }
                        Err(error) => return Err(format!("Responses accept: {error}")),
                    };
                    let request = read_request(&mut stream)?;
                    let ordinal = {
                        let mut captured = captured.lock().map_err(|_| "request capture poisoned")?;
                        let ordinal = captured.len();
                        captured.push(request);
                        ordinal
                    };
                    let body = response_events(&worker_root, case, ordinal)?.iter()
                        .map(|event| format!("data: {event}\n\n")).collect::<String>();
                    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
                        .map_err(|error| format!("Responses write: {error}"))?;
                }
                Ok(())
            });
            Self { endpoint: format!("{origin}/v1/responses"), credential, requests,
                stop, worker: Some(worker), outcome: None }
        }

        fn finish(&mut self) -> Result<(), String> {
            self.stop.store(true, Ordering::Release);
            if let Some(outcome) = &self.outcome {
                let outcome = outcome.clone();
                if self.worker.as_ref().is_some_and(|worker| worker.is_finished()) {
                    let _ = self.worker.take().unwrap().join();
                }
                return outcome;
            }
            let worker = self.worker.take().expect("one owned Responses worker");
            let deadline = Instant::now() + CLEANUP_WAIT;
            while !worker.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(2));
            }
            let result = if worker.is_finished() {
                worker.join().unwrap_or_else(|_| Err("Responses worker panicked".into()))
            } else {
                self.worker = Some(worker);
                Err("owned Responses worker did not stop before its deadline".into())
            };
            self.outcome = Some(result.clone());
            result
        }
    }

    impl Drop for Responses {
        fn drop(&mut self) {
            if let Err(error) = self.finish() { eprintln!("SGF Responses cleanup: {error}"); }
        }
    }

    struct OwnedChild(Option<Child>);

    impl OwnedChild {
        fn wait(&mut self) -> Result<ExitStatus, String> {
            let deadline = Instant::now() + Duration::from_secs(60);
            loop {
                match self.0.as_mut().expect("owned App child").try_wait() {
                    Ok(Some(status)) => { self.0.take(); return Ok(status); }
                    Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(5)),
                    Ok(None) => return Err("owned App child exceeded its deadline".into()),
                    Err(error) => return Err(format!("owned App child wait: {error}")),
                }
            }
        }
    }

    impl Drop for OwnedChild {
        fn drop(&mut self) {
            if let Some(child) = self.0.as_mut() {
                // This handle identifies only the subprocess started below.
                let _ = child.kill();
                let deadline = Instant::now() + CLEANUP_WAIT;
                while Instant::now() < deadline {
                    match child.try_wait() {
                        Ok(Some(_)) => { self.0.take(); break; }
                        Ok(None) => thread::sleep(Duration::from_millis(5)),
                        Err(error) => { eprintln!("SGF owned child cleanup wait: {error}"); break; }
                    }
                }
                if self.0.is_some() { eprintln!("SGF owned child cleanup remains unresolved"); }
            }
        }
    }

    fn optional_bytes(path: &Path) -> Option<Vec<u8>> {
        match fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => panic!("read owned fixture {}: {error}", path.display()),
        }
    }

    fn file_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn visit(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
            if !directory.exists() { return; }
            for entry in fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() { visit(root, &path, files); }
                else { files.insert(path.strip_prefix(root).unwrap().to_path_buf(), fs::read(path).unwrap()); }
            }
        }
        let mut files = BTreeMap::new();
        visit(root, root, &mut files);
        files
    }

    fn varint(bytes: &[u8], position: &mut usize) -> u64 {
        let mut value = 0_u64;
        for shift in (0..70).step_by(7) {
            let byte = *bytes.get(*position).expect("complete actual generated varint");
            *position += 1;
            assert!(shift < 63 || byte <= 1, "bounded generated varint");
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 { return value; }
        }
        panic!("oversized actual generated varint");
    }

    #[derive(Debug, PartialEq)]
    enum Field<'a> { Number(u64), Bytes(&'a [u8]) }

    fn fields(bytes: &[u8]) -> BTreeMap<u64, Field<'_>> {
        // This reads the frozen producer's observed wire for byte/identity
        // assertions only; production decoding remains with its actual owners.
        let mut position = 0;
        let mut fields = BTreeMap::new();
        while position < bytes.len() {
            let tag = varint(bytes, &mut position);
            assert!(tag >> 3 > 0);
            let field = match tag & 7 {
                0 => Field::Number(varint(bytes, &mut position)),
                2 => {
                    let length = usize::try_from(varint(bytes, &mut position)).unwrap();
                    let end = position.checked_add(length).unwrap();
                    let value = bytes.get(position..end).expect("complete generated field");
                    position = end;
                    Field::Bytes(value)
                }
                _ => panic!("unexpected frozen SendMessage wire type"),
            };
            assert!(fields.insert(tag >> 3, field).is_none(), "one actual producer field per identity");
        }
        fields
    }

    fn sql_timestamp(entry: &Value) -> u64 {
        // The shipping transcript writer stores its timestamp as JSON f64;
        // SendMessage's public/protobuf result retains the original integer.
        let timestamp = entry["timestampMs"].as_f64().expect("actual SQL send timestamp");
        assert!(timestamp > 0.0 && timestamp <= 9_007_199_254_740_991.0);
        assert_eq!(timestamp.fract(), 0.0, "lossless integer millisecond timestamp");
        timestamp as u64
    }

    fn assert_wire_identity(wire: &[u8], ordinal: usize, entry: &Value) {
        let call = fields(wire);
        assert_eq!(call.len(), 4);
        assert_eq!(call.get(&57), Some(&Field::Bytes(call_id(ordinal).as_bytes())));
        let Some(Field::Number(started)) = call.get(&59) else { panic!("producer start timestamp"); };
        let Some(Field::Number(completed)) = call.get(&60) else { panic!("producer completion timestamp"); };
        let timestamp = sql_timestamp(entry);
        assert!(*started > 0 && *completed > 0, "actual producer wall-clock timestamps are positive");
        let Some(Field::Bytes(send)) = call.get(&55) else { panic!("actual SendMessage variant"); };
        let send = fields(send);
        assert_eq!(send.len(), 2, "actual args and completed result");
        let Some(Field::Bytes(result)) = send.get(&2) else { panic!("completed result"); };
        let result = fields(result);
        assert_eq!(result.len(), 1);
        let Some(Field::Bytes(success)) = result.get(&1) else { panic!("actual successful result"); };
        let success = fields(success);
        assert_eq!(success.get(&1), Some(&Field::Number(timestamp)));
        assert_eq!(success.get(&2), Some(&Field::Bytes(entry["id"].as_str().unwrap().as_bytes())));
    }

    fn read_history(fixture: &ChannelHostLaneFixture, original: &OriginalTurnReference, case: Case) -> Value {
        let sessions = &fixture.deps().session_workers;
        let store = sessions.open_agent_store_owner(&fixture.agent_id).unwrap();
        let checkpoint = store.checkpoint_snapshot_checked().unwrap().expect("actual committed generated history");
        let state = decode_conversation_state_recovery_fields(&checkpoint.checkpoint_bytes).unwrap();
        assert_eq!(state.turns.len(), 1);
        let turn = store.get_blob(&state.turns[0]).unwrap().unwrap();
        let Some(ConversationTurnStructureFields::Agent { user_message, steps, request_id }) =
            decode_conversation_turn_structure_fields(&turn).unwrap() else { panic!("actual generated Agent turn"); };
        assert_eq!(request_id.as_deref(), Some(STREAM));
        let user_bytes = store.get_blob(&user_message).unwrap().unwrap();
        let user = decode_user_message_identity_fields(&user_bytes).unwrap();
        assert_eq!(user.message_id, original.user_message_id);
        assert_eq!(user.text, PROMPT);
        let entries = fixture.entries();
        assert_eq!(entries.len(), 3, "one accepted user and two actual App SQL sends");
        assert_eq!(entries[0]["id"], original.user_message_id);
        assert_eq!(entries[0]["confirmed"], true);
        let sends = entries.iter().filter(|entry| entry["kind"] == "send-message").cloned().collect::<Vec<_>>();
        assert_eq!(sends.len(), 2);
        assert_ne!(sends[0]["id"], sends[1]["id"]);
        for entry in &sends {
            assert!(!entry["id"].as_str().unwrap().is_empty());
            assert_eq!(entry["sessionKey"], fixture.session_key);
            assert!(entry["message"].get("channel").is_none(), "local SQL success makes no external delivery claim");
        }
        assert_eq!(sends[0]["message"], json!({"type":"text", "content":TEXT}));
        let normalized_url = sends[1]["message"]["url"].as_str().unwrap();
        assert_ne!(normalized_url, arguments(&fixture.root, 1)["url"].as_str().unwrap());
        let normalized_path = file_path_from_file_url(normalized_url).unwrap();
        assert!(normalized_path.starts_with(fixture.root.join("agents").join(&fixture.agent_id)));
        assert_eq!(fs::read(&normalized_path).unwrap(), ATTACHMENT_BYTES);
        assert_eq!(sends[1]["message"], json!({"type":"attachment", "url":normalized_url,
            "alt":ALT, "file_name":"attachment-source.txt"}));
        let normalized = vec![
            json!({"type":"text", "content":TEXT}),
            json!({"type":"attachment", "url":normalized_url, "alt":ALT}),
        ];
        let outline_sends = sessions.read_agent_outline(&fixture.agent_id).unwrap().into_iter()
            .filter_map(|item| match item { ConversationOutlineItem::SendMessage { message, .. } => Some(message), _ => None })
            .collect::<Vec<_>>();
        assert_eq!(outline_sends, normalized, "the real Session consumer selects the two completed sends in order");
        let success = case != Case::ProviderFailure;
        assert_eq!(steps.len(), 2 + usize::from(success), "failed generation cannot invent a final Assistant step");
        let codec = GeneratedTranscriptOccurrenceCodec::new(CanonicalGeneratedToolJsonProjection);
        let mut blobs = vec![(checkpoint.root_blob_id.clone(), checkpoint.checkpoint_bytes.clone()),
            (state.turns[0].clone(), turn), (user_message, user_bytes)];
        let mut wires = Vec::new();
        let mut journal = vec![json!({"role":"user", "message":{"content":[{"type":"text", "text":PROMPT}]}})];
        for (ordinal, id) in steps.iter().enumerate() {
            let bytes = store.get_blob(id).unwrap().unwrap();
            if ordinal < 2 {
                let step = fields(&bytes);
                assert_eq!(step.len(), 1);
                let Some(Field::Bytes(wire)) = step.get(&2) else { panic!("canonical ConversationStep field 2"); };
                assert_wire_identity(wire, ordinal, &sends[ordinal]);
                wires.push(wire.to_vec());
                let input = if ordinal == 0 { json!({"text":{"content":TEXT}}) }
                    else { json!({"attachment":{"url":normalized_url, "alt":ALT}}) };
                let result = json!({"success":{"timestamp":sql_timestamp(&sends[ordinal]).to_string(),
                    "messageId":sends[ordinal]["id"]}});
                assert_eq!(codec.decode_step(&bytes).unwrap(), DecodedTranscriptStep::Tool {
                    name:"send_message".into(), input:input.clone(), result:Some(result.clone()),
                });
                journal.push(json!({"role":"assistant", "message":{"content":[{
                    "type":"tool_use", "name":"send_message", "input":input,
                }]}}));
                journal.push(json!({"role":"tool", "message":{"content":[{
                    "type":"tool_result", "name":"send_message", "result":result,
                }]}}));
            } else {
                assert_eq!(codec.decode_step(&bytes).unwrap(), DecodedTranscriptStep::Assistant { text:FINAL_TEXT.into() });
                journal.push(json!({"role":"assistant", "message":{"content":[{"type":"text", "text":FINAL_TEXT}]}}));
            }
            blobs.push((id.clone(), bytes));
        }
        let journal_path = fixture.root.join("transcripts").join(&fixture.agent_id).join(format!("{}.jsonl", fixture.agent_id));
        let actual_journal = fs::read_to_string(journal_path).unwrap().lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap()).collect::<Vec<_>>();
        assert_eq!(actual_journal, journal, "Journal preserves projected normalized args, result, SQL identity and order");
        let connection = rusqlite::Connection::open_with_flags(sessions.session_db_path(&fixture.agent_id).unwrap(),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let sql_rows = connection.prepare("SELECT seq,id,CAST(entry AS BLOB) FROM transcript_entries ORDER BY seq").unwrap()
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, Vec<u8>>(2)?))).unwrap()
            .collect::<Result<Vec<_>, _>>().unwrap();
        let terminal = sessions.open_agent_db_owner(&fixture.agent_id).unwrap().read_kv(TERMINAL_SLOT).unwrap();
        assert_eq!(terminal.is_some(), success, "only the actual successful invocation may own a completion receipt");
        json!({"root":checkpoint.root_blob_id, "checkpoint":checkpoint.checkpoint_bytes, "blobs":blobs,
            "entries":entries, "sends":sends, "wires":wires, "sqlRows":sql_rows, "terminal":terminal,
            "journal":actual_journal, "mirrorFiles":file_bytes(&fixture.root.join("transcripts")),
            "providerFiles":file_bytes(&fixture.root.join("runner-provider-checkpoints"))})
    }

    fn produce_and_reopen(root: PathBuf, case: Case) -> Value {
        let mut fixture = ChannelHostLaneFixture::open("generated-send-message", root.clone(), None);
        let sessions = Arc::clone(&fixture.deps().session_workers);
        let runtime = Arc::clone(&fixture.deps().transcript_runtime);
        let registry = fixture.deps().runner_registry.clone();
        let args = json!({"agentId":fixture.agent_id, "sessionKey":fixture.session_key,
            "streamId":STREAM, "clientNonce":"actual-app-send-nonce", "prompt":PROMPT});
        let accepted = runtime.accept_routed_send(&args, |accepted| {
            persist_accepted_send_prompt_context(&sessions, &args, accepted)
                .map_err(|error| ProductionSendError::Internal(format!("{error:?}")))
        }).expect("actual accepted user input");
        assert!(accepted.context.accepted_durably && !accepted.duplicate);
        let control = runtime.capture_routed_turn_start(&fixture.agent_id, STREAM).unwrap();
        let cancellation = registry.register_routed_provider_with_cancellation(
            &fixture.agent_id, STREAM, false, control.cancellation(),
        ).unwrap();
        assert!(registry.mark_routed_provider_dispatched(STREAM));
        let admission = sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
        let original = runtime.bind_original_turn_model(&fixture.agent_id, STREAM, &admission, MODEL).unwrap().unwrap();
        assert_eq!(accepted.context.user_message_id.as_deref(), Some(original.user_message_id.as_str()));
        let checkpoint: Arc<dyn AgentStateCheckpointSink> = fixture.deps().host_runner_composition.compose_production_checkpoint_sink(
            &sessions, &root, &fixture.agent_id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
        ).unwrap().unwrap();
        assert!(checkpoint.committed_original_turn_observations().unwrap().is_empty());
        let events = Arc::new(Mutex::new(Vec::<Value>::new()));
        let observed = Arc::clone(&events);
        let observation = (case != Case::Unobserved).then(|| TurnObservation::shared(
            &fixture.agent_id, Some(Arc::new(move |event| { observed.lock().unwrap().push(event); })),
        ));
        let send_counter = SendMessageDeliveryCounter::default();
        // Transfer the fixture's actual private App sink into the production
        // wrapper. Direct fixture.send() would bypass this entire oracle.
        let app_sink: Arc<dyn SendMessageSink> = Arc::new(fixture.sink.take().unwrap());
        let send_sink: Arc<dyn SendMessageSink> = Arc::new(CountingSendMessageSink::new(app_sink, send_counter.clone()));
        let provider_store = Arc::new(ProductionRoutedProviderCheckpointStore::new(&root, &fixture.agent_id, STREAM));
        let retries = Arc::new(AtomicUsize::new(0));
        let retry_count = Arc::clone(&retries);
        let mut composition = TurnAgentComposition::new(
            RoutedProvider::Fabushi, Arc::new(UnusedCompactionProviderPorts),
            RunnerRequestContextSnapshot { context: mahayana_host_runtime::host_request_context::HostRequestContext {
                os_version:"send-message-fidelity".into(), shell:None, time_zone:Some("UTC".into()),
                transcripts_folder:root.join("transcripts").to_string_lossy().into_owned(), user_full_name:None,
            }, rules:None }, cancellation.clone(), provider_store,
        ).with_fabushi_model(MODEL).with_send_message_sink(send_sink)
            .with_retry_sink(Arc::new(move |_| { retry_count.fetch_add(1, Ordering::AcqRel); }));
        if let Some(observation) = &observation { composition = composition.with_observation(Arc::clone(observation)); }
        let successful_hooks = Arc::new(AtomicUsize::new(0));
        let hook_count = Arc::clone(&successful_hooks);
        let mut runner = create_production_runner(composition,
            ProductionTurnAgentBuildBindings::new(ProductionTurnAgentStaticConfig {
                model_id:MODEL.into(), agent_token_limit:SAND_AGENT_TOKEN_LIMIT,
                conversation_id:fixture.agent_id.clone(), is_box_scoped_subagent:false,
                is_subagent_runner:false, is_shared_room_runner:false,
                sand_send_message_delivery_owed:false, transcripts_folder_available:true,
            }, Arc::new(|| None), Arc::new(|_, _, _| panic!("bounded SendMessage fixture must not summarize"))),
            ProductionTurnAgentLifecycleBindings::new(&fixture.agent_id, STREAM)
                .with_profile_announcement_commit(Some(Arc::new(move || { hook_count.fetch_add(1, Ordering::AcqRel); }))),
            Some(Arc::clone(&checkpoint)), Arc::new(AtomicBool::new(false)),
            Arc::new(Mutex::new(SubagentRuntime::default())), observation, None, None, None,
        );
        let mut deltas = Vec::new();
        let result = runner.run_routed_provider_with_options(&root,
            &[ProviderMessage { role:"user".into(), content:PROMPT.into() }],
            TurnRunOptions { inference_request_id:Some(STREAM.into()), message_id:Some(original.user_message_id.clone()),
                recent_message_text:Some(PROMPT.into()), turn_started_at_ms:Some(1), ..TurnRunOptions::default() },
            &mut |delta, _| deltas.push(delta.to_string()),
        );
        assert_eq!(send_counter.count(), 2, "the real counting wrapper observes exactly two SQL sends");
        assert_eq!(retries.load(Ordering::Acquire), 0);
        let failed = case == Case::ProviderFailure;
        assert_eq!(deltas, vec![if failed { PARTIAL_TEXT } else { FINAL_TEXT }.to_string()]);
        let finished = runner.last_finished().expect("actual generated shell terminal");
        assert_eq!(finished.owner.request_id, STREAM);
        assert!(finished.owner.generation > 0);
        if failed {
            let Err(ProviderSessionError::ProviderRefusal(refusal)) = &result else {
                panic!("generated history must preserve the primary typed refusal: {result:?}");
            };
            assert_eq!(refusal.category, "misalignment");
            assert_eq!(refusal.explanation.as_deref(), Some(REFUSAL));
            assert_eq!(finished.outcome, TerminalOutcome::Failed { retryable:false,
                message:result.as_ref().unwrap_err().to_string() });
            assert!(checkpoint.committed_original_turn_observations().is_err());
        } else {
            assert_eq!(result.as_ref().unwrap(), FINAL_TEXT);
            assert_eq!(finished.outcome, TerminalOutcome::Completed);
            assert_eq!(checkpoint.committed_original_turn_observations().unwrap().len(), 1);
        }
        assert_eq!(successful_hooks.load(Ordering::Acquire), usize::from(!failed));
        assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::Known {
            run_generation:finished.owner.generation, snapshot:ToolEffectCustodySnapshot {
                started_revision:2, checkpoint_revision:2, pending_effects:0, unknown_result:None,
            },
        }), "canonical history must not advance the accepted provider checkpoint custody");
        let mut custody = ToolEffectCustodyReport::new(STREAM);
        custody.record(runner.last_tool_effect_custody());
        let terminal = crate::RoutedProviderTerminal::capture(STREAM, false, &cancellation,
            result, send_counter.count(), false, custody);
        assert_eq!(terminal.event["payload"]["type"], if failed { "failed" } else { "completed" });
        if failed {
            assert!(terminal.persist_original_turn(&sessions, &original, checkpoint.as_ref()).is_err());
            assert!(reconcile_completed_original_turn(&sessions, &original).unwrap().is_none());
        } else {
            assert_eq!(terminal.persist_original_turn(&sessions, &original, checkpoint.as_ref()).unwrap(),
                OriginalTurnReceiptCommit::Inserted);
            assert!(reconcile_completed_original_turn(&sessions, &original).unwrap().is_some());
        }
        let history = read_history(&fixture, &original, case);
        let tool_events = events.lock().unwrap().iter().filter(|event| event["type"] == "agent-tool-call")
            .cloned().collect::<Vec<_>>();
        assert_eq!(tool_events.len(), if case == Case::Unobserved { 0 } else { 4 });
        if case != Case::Unobserved {
            for ordinal in 0..2 {
                let initial = &tool_events[ordinal * 2];
                let completed = &tool_events[ordinal * 2 + 1];
                for (event, phase) in [(initial, "initial"), (completed, "completed")] {
                    assert_eq!(event["agentId"], fixture.agent_id);
                    assert_eq!(event["tool"], "SendMessage");
                    assert_eq!(event["toolCallId"], call_id(ordinal));
                    assert_eq!(event["encoding"], "agent.v1.protobuf");
                    assert_eq!(event["phase"], phase);
                }
                assert_eq!(completed["wire"], history["wires"][ordinal],
                    "canonical field 2 must contain the unchanged actual completed producer bytes");
                let initial_wire = serde_json::from_value::<Vec<u8>>(initial["wire"].clone()).unwrap();
                let completed_wire = serde_json::from_value::<Vec<u8>>(completed["wire"].clone()).unwrap();
                let initial_fields = fields(&initial_wire);
                let completed_fields = fields(&completed_wire);
                assert_eq!(initial_fields.get(&57), completed_fields.get(&57));
                assert_eq!(initial_fields.get(&59), completed_fields.get(&59));
                assert!(!initial_fields.contains_key(&60));
                let (Some(Field::Bytes(initial_send)), Some(Field::Bytes(completed_send))) =
                    (initial_fields.get(&55), completed_fields.get(&55)) else { panic!("actual paired SendMessage variant"); };
                let initial_send = fields(initial_send);
                let completed_send = fields(completed_send);
                assert!(!initial_send.contains_key(&2));
                assert_eq!(initial_send.get(&1), completed_send.get(&1), "producer normalization remains frozen across phases");
            }
        }
        // A failed original invocation has a real pending upgrade marker, but
        // neither a completion capability nor an independent ACK obligation.
        let marker = if failed {
            runtime.quiesce_for_upgrade().unwrap();
            let markers = runtime.upgrade_resume_store().unwrap().list_pending_checked().unwrap();
            assert_eq!(markers.len(), 1);
            assert_eq!(markers[0].original_work.as_ref(), Some(&original));
            Some(markers[0].clone())
        } else { None };
        registry.finish_routed_provider(STREAM);
        runtime.settle_routed_turn(&fixture.agent_id, STREAM, started_at_ms()).unwrap().unwrap();
        runtime.resume_after_recreate();
        assert!(!runtime.is_agent_running(&fixture.agent_id));
        assert_eq!(registry.active_count(), 0, "marker refusal must not rely on a still-running turn");
        let ack = SandAckObligationStore::new(&root);
        assert!(ack.get_checked(&fixture.agent_id).unwrap().is_none());
        let ack_bytes = optional_bytes(ack.file_path());
        let marker_store = runtime.upgrade_resume_store().unwrap();
        let marker_bytes = optional_bytes(marker_store.file_path());
        if let Some(marker) = &marker {
            assert_eq!(crate::acknowledge_completed_upgrade_marker(&sessions, &runtime, &registry, &ack, marker), Ok(false));
            assert_eq!(optional_bytes(marker_store.file_path()), marker_bytes);
            assert!(checkpoint.committed_original_turn_observations().is_err(), "settlement cannot restore failed completion authority");
        }
        assert_eq!(optional_bytes(ack.file_path()), ack_bytes);
        fixture.assert_no_worker_observation();
        assert!(matches!(fixture.lane_observations.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "channel-free sends must not enter external delivery");
        fixture.assert_no_failure();
        drop((runner, checkpoint, terminal, control));
        let (root, agent_id) = fixture.stop_for_reopen();
        sessions.shutdown_with_checkpoint();
        assert_eq!(sessions.active_worker_count(), 0);
        drop((sessions, runtime, registry));
        fs::remove_file(root.join("responses-credential.json")).expect("no provider credential for fresh consumers");
        for reopen in 0..2 {
            let fresh = ChannelHostLaneFixture::open("send-message-fresh-reader", root.clone(), Some(agent_id.clone()));
            let sessions = Arc::clone(&fresh.deps().session_workers);
            let prepared = sessions.prepare_existing_agent(&agent_id).unwrap().unwrap();
            assert_eq!(json!(prepared.persisted_root_blob_id), history["root"]);
            let fresh_sink = fresh.deps().host_runner_composition.compose_production_checkpoint_sink(
                &sessions, &root, &agent_id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
            ).unwrap().unwrap();
            assert!(fresh_sink.committed_original_turn_observations().unwrap().is_empty(),
                "reading history does not issue a new invocation's completion observation");
            assert_eq!(read_history(&fresh, &original, case), history,
                "fresh owner {reopen} cannot append SQL rows, alter wire/Journal, retry tools or rewrite terminal authority");
            assert_eq!(reconcile_completed_original_turn(&sessions, &original).unwrap().is_some(), !failed);
            assert!(!fresh.deps().transcript_runtime.is_agent_running(&agent_id));
            assert_eq!(fresh.deps().runner_registry.active_count(), 0);
            if let Some(marker) = &marker {
                assert_eq!(fresh.deps().transcript_runtime.upgrade_resume_store().unwrap().list_pending_checked().unwrap(), vec![marker.clone()]);
                assert_eq!(crate::acknowledge_completed_upgrade_marker(&sessions, &fresh.deps().transcript_runtime,
                    &fresh.deps().runner_registry, &ack, marker), Ok(false));
            }
            assert_eq!(optional_bytes(fresh.deps().transcript_runtime.upgrade_resume_store().unwrap().file_path()), marker_bytes);
            assert_eq!(optional_bytes(ack.file_path()), ack_bytes);
            assert_eq!(read_history(&fresh, &original, case), history,
                "completion reconciliation and marker refusal cannot mutate SQL, canonical history or Journal");
            let entries = history["entries"].as_array().unwrap();
            fresh.assert_no_followup(entries);
            fresh.assert_no_failure();
            assert!(matches!(fresh.lane_observations.try_recv(), Err(mpsc::TryRecvError::Empty)));
            drop((fresh_sink, prepared));
            let _ = fresh.stop_for_reopen();
            sessions.shutdown_with_checkpoint();
            assert_eq!(sessions.active_worker_count(), 0);
        }
        json!({"history":history, "events":tool_events, "reopens":2, "marker":marker,
            "sentCount":send_counter.count(), "retryCount":retries.load(Ordering::Acquire)})
    }

    fn run_case(case: Case) {
        if let Some(path) = std::env::var_os(CHILD_REQUEST_ENV) {
            let request: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert_eq!(request["case"], case.label());
            assert_ne!(request["parentPid"].as_u64().unwrap(), u64::from(std::process::id()));
            let root = PathBuf::from(request["root"].as_str().unwrap());
            assert!(root.is_absolute() && PathBuf::from(path).starts_with(&root));
            let proof = produce_and_reopen(root.clone(), case);
            let mut receipt = fs::OpenOptions::new().write(true).create_new(true)
                .open(root.join("completed.json")).expect("fresh post-shutdown child receipt");
            receipt.write_all(json!({"token":request["token"], "parentPid":request["parentPid"],
                "pid":std::process::id(), "case":case.label(), "proof":proof}).to_string().as_bytes()).unwrap();
            receipt.sync_all().unwrap();
            return;
        }
        let root = std::env::temp_dir().join(format!("fabushi-send-message-fidelity-{}-{}", case.label(), uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        fs::write(root.join("settings.json"), b"{}").unwrap();
        fs::write(root.join("attachment-source.txt"), ATTACHMENT_BYTES).unwrap();
        let token = uuid::Uuid::new_v4().to_string();
        let request_path = root.join("request.json");
        fs::write(&request_path, json!({"token":token, "parentPid":std::process::id(),
            "case":case.label(), "root":root}).to_string()).unwrap();
        let mut responses = Responses::new(&root, case);
        let stdout_path = root.join("child.stdout");
        let stderr_path = root.join("child.stderr");
        let mut child = OwnedChild(Some(Command::new(std::env::current_exe().unwrap())
            .args(["--exact", case.test_name(), "--nocapture", "--test-threads=1"])
            .env(CHILD_REQUEST_ENV, &request_path)
            .env("FABUSHI_RESPONSES_URL", &responses.endpoint)
            .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &responses.credential)
            .env("FABUSHI_RESPONSES_MODEL", MODEL)
            .env("NO_PROXY", "127.0.0.1,localhost")
            .env("no_proxy", "127.0.0.1,localhost")
            .env_remove("HTTP_PROXY").env_remove("HTTPS_PROXY").env_remove("ALL_PROXY")
            .env_remove("http_proxy").env_remove("https_proxy").env_remove("all_proxy")
            .current_dir(&root).stdin(Stdio::null())
            .stdout(Stdio::from(fs::File::create(&stdout_path).unwrap()))
            .stderr(Stdio::from(fs::File::create(&stderr_path).unwrap()))
            .spawn().expect("isolated actual App test process")));
        let pid = child.0.as_ref().unwrap().id();
        let child_result = child.wait();
        drop(child);
        let server_result = responses.finish();
        let stdout = fs::read_to_string(stdout_path).unwrap();
        let stderr = fs::read_to_string(stderr_path).unwrap();
        assert!(child_result.as_ref().is_ok_and(|status| status.success()) && server_result.is_ok(),
            "SGF case={case:?} child={child_result:?} server={server_result:?}\n{stdout}\n{stderr}\nretained fixture={}", root.display());
        let receipt: Value = serde_json::from_slice(&fs::read(root.join("completed.json")).unwrap()).unwrap();
        assert_eq!(receipt["token"], token);
        assert_eq!(receipt["pid"], pid);
        assert_eq!(receipt["parentPid"], std::process::id());
        assert_eq!(receipt["case"], case.label());
        assert_eq!(receipt["proof"]["reopens"], 2);
        assert_eq!(receipt["proof"]["sentCount"], 2);
        assert_eq!(receipt["proof"]["retryCount"], 0);
        let requests = responses.requests.lock().unwrap();
        assert_eq!(requests.len(), 2, "fresh consumers cannot send another provider request");
        assert!(requests.iter().all(|request| request["model"] == MODEL));
        let input = requests[1]["input"].as_array().expect("actual serialized provider continuation");
        let tools = input.iter().filter(|item| matches!(item["type"].as_str(),
            Some("function_call" | "function_call_output"))).collect::<Vec<_>>();
        assert_eq!(tools.len(), 4);
        for ordinal in 0..2 {
            assert_eq!(tools[ordinal * 2]["type"], "function_call");
            assert_eq!(tools[ordinal * 2]["name"], "SendMessage");
            assert_eq!(tools[ordinal * 2]["call_id"], call_id(ordinal));
            let actual_args: Value = serde_json::from_str(tools[ordinal * 2]["arguments"].as_str().unwrap()).unwrap();
            assert_eq!(actual_args, arguments(&root, ordinal));
            assert_eq!(tools[ordinal * 2 + 1]["type"], "function_call_output");
            assert_eq!(tools[ordinal * 2 + 1]["call_id"], call_id(ordinal));
            let result: Value = serde_json::from_str(tools[ordinal * 2 + 1]["output"].as_str().unwrap()).unwrap();
            let entry = &receipt["proof"]["history"]["sends"][ordinal];
            assert_eq!(result, json!({"sent":true, "timestampMs":sql_timestamp(entry), "messageId":entry["id"]}),
                "the public result stays flat and identifies the actual SQL append");
        }
        drop(requests);
        eprintln!("SGF case={} producer_pid={pid} requests=2 sends=2 fresh_owners=2 outcome=complete", case.label());
        drop(responses);
        fs::remove_dir_all(root).expect("remove only this fully settled fixture");
    }

    #[test]
    fn generated_send_message_app_sink_preserves_live_sql_and_canonical_bytes() {
        run_case(Case::Observed);
    }

    #[test]
    fn generated_send_message_app_sink_preserves_history_without_observation() {
        run_case(Case::Unobserved);
    }

    #[test]
    fn generated_send_message_app_sink_retains_failed_turn_without_completion_or_replay() {
        run_case(Case::ProviderFailure);
    }
}
