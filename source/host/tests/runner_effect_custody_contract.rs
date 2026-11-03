//! Real Responses parsing, production persistence, attempt custody and local MCP.
//! Controlled transport establishes this boundary, not external-provider acceptance.

use std::collections::VecDeque;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::inference::codex_direct_responses::{
    CodexDirectError, CodexDirectOptions, CodexDirectTool, CodexDirectTransport,
    run_codex_direct_responses_with_lifecycle_and_tool_step_reminder,
};
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedProviderCheckpoint,
    RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::local_exec::extension::{
    HostLocalExecExtension, start_local_exec_extension,
};
use mahayana_host_runtime::extensions::local_exec::local_exec_bridge::LocalExecProviderRegistration;
use mahayana_host_runtime::extensions::local_tool_permission::extension::{
    HostLocalToolPermissionExtension, start_local_tool_permission_extension,
};
use mahayana_host_runtime::extensions::settings::settings_service::{
    SandLocalToolPermission, SettingsService,
};
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostTelemetryService;
use mahayana_host_runtime::gateway_server::GatewayEventHub;
use mahayana_host_runtime::host_event_bus::HostEventSubscription;
use mahayana_host_runtime::runner::coordinator_tool_relay::{
    CoordinatorToolRelay, CoordinatorToolRelayError, ROUTED_TOOL_EXECUTE_METHOD,
    ROUTED_TOOL_LIST_METHOD, RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
    ProductionTurnRunShellAdapter, ProviderRetryEvent, ProviderRetryOutcome,
    ProviderRetryReport, RoutedProviderAttemptControl, RoutedProviderAttemptExecutor,
    RoutedProviderCheckpointStore, ToolEffectCustodySnapshot, ToolEffectUnknownResult,
};
use mahayana_host_runtime::runner::production_turn_agent_owner::{
    ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
    ProductionTurnAgentStaticConfig,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedToolBridge,
    RunnerRequestContextSnapshot, coordinator_tool_execution_error,
    start_routed_mcp_server_for_attempt,
};
use mahayana_host_runtime::runner::host_external_machine_dependencies::ProductionExternalMachineExecutor;
use mahayana_host_runtime::runner::sand_agent_runner::SandAgentRunner;
use mahayana_host_runtime::runner::tools::sand_external_machine_tools::{
    ExternalMachineExecutor, ExternalMachineShellArgs,
};
use mahayana_host_runtime::runner::subagent_runtime::SubagentRuntime;
use mahayana_host_runtime::runner::turn_agent_composition::{
    SAND_AGENT_TOKEN_LIMIT, TurnAgentComposition,
};
use mahayana_host_runtime::runner::tools::mcp_meta_tools::execute_routed_tool_with_timeout_ms;
use mahayana_host_runtime::runner::{
    StreamAttemptPolicy, StreamAttemptRuntime, ToolEffectCustodyObservation,
    ToolEffectCustodyReport, TurnRunOptions,
};
use mahayana_host_runtime::runner_production_bridge::create_production_runner;
use serde_json::{Value, json};

const MODEL: &str = "Vendor/Nested/CaseSensitive@Custody";
const BOUND: Duration = Duration::from_secs(5);
const WATCHDOG: Duration = Duration::from_millis(500);

fn wait_until(label: &str, mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + BOUND;
    while !ready() {
        assert!(Instant::now() < deadline, "{label} exceeded its bound");
        thread::sleep(Duration::from_millis(1));
    }
}

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "runner-effect-custody-{}", uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&path).expect("private fixture directory");
        Self(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn policy(timeout: Duration) -> StreamAttemptPolicy {
    StreamAttemptPolicy {
        first_output_timeout: timeout,
        max_attempts: 2,
        initial_backoff: Duration::ZERO,
        max_backoff: Duration::ZERO,
        max_retry_after: Duration::ZERO,
    }
}

#[derive(Clone)]
enum Response {
    Calls(Vec<String>),
    Text,
    TransportFailure,
    Stall,
}

fn calls(names: &[&str]) -> Response {
    Response::Calls(names.iter().map(|name| (*name).to_string()).collect())
}

struct Responses {
    pending: VecDeque<Response>,
    requests: Vec<Value>,
}

impl CodexDirectTransport for Responses {
    fn stream_response(
        &mut self,
        request: &Value,
        on_event: &mut dyn FnMut(Value) -> Result<(), CodexDirectError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<(), CodexDirectError> {
        self.requests.push(request.clone());
        match self.pending.pop_front().expect("bounded Responses script") {
            Response::Calls(names) => on_event(json!({
                "type": "response.completed",
                "response": {
                    "id": format!("response-{}", self.requests.len()),
                    "output": names.iter().map(|name| json!({
                        "type": "function_call", "name": name,
                        "call_id": format!("call-{name}"), "arguments": "{}"
                    })).collect::<Vec<_>>()
                }
            })),
            Response::Text => {
                on_event(json!({"type":"response.output_text.delta","delta":"done"}))?;
                on_event(json!({
                    "type":"response.completed",
                    "response":{"id":"done","output":[]}
                }))
            }
            Response::TransportFailure => Err(CodexDirectError::Transport(
                "connection reset before the next response".into(),
            )),
            Response::Stall => {
                wait_until("real adapter watchdog", should_cancel);
                Err(CodexDirectError::Cancelled("watchdog observed by transport".into()))
            }
        }
    }
}

type ToolAction = Arc<dyn Fn(&RoutedProviderAttemptControl, &str) + Send + Sync>;
type ControlSlot = Arc<Mutex<Option<RoutedProviderAttemptControl>>>;

struct GuardReturn {
    tool: String,
    outcome_unknown: bool,
    custody: ToolEffectCustodySnapshot,
}

/// Only transport is scripted. This executor delegates tool selection, result
/// pairing, cancellation and checkpoint production to the real Responses loop.
struct ResponsesExecutor {
    transport: Responses,
    effects: Arc<Mutex<Vec<String>>>,
    action: ToolAction,
    control: ControlSlot,
    timeout_ms: u64,
    attempts: usize,
    resumed: Vec<Option<RoutedProviderCheckpoint>>,
    relay: Option<Arc<CoordinatorToolRelay>>,
    guard_returns: Vec<GuardReturn>,
}

impl RoutedProviderAttemptExecutor for ResponsesExecutor {
    fn run_attempt(
        &mut self,
        attempt: &RoutedProviderAttemptControl,
        resume_from: Option<&RoutedProviderCheckpoint>,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_checkpoint: &mut dyn FnMut(&RoutedProviderCheckpoint) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        self.attempts += 1;
        self.resumed.push(resume_from.cloned());
        *self.control.lock().expect("control slot") = Some(attempt.clone());
        let resume = match resume_from {
            Some(RoutedProviderCheckpoint::Fabushi(checkpoint)) => Some(checkpoint),
            None => None,
            Some(_) => panic!("fixture must preserve provider identity"),
        };
        let mut options = CodexDirectOptions::new(
            MODEL, "Exercise the real tool boundary",
            vec![json!({"role":"user","content":"perform the counted effects"})],
        );
        options.max_steps = 8;
        options.tools = ["a", "b", "c"].into_iter().map(|name| CodexDirectTool {
            name: name.into(),
            description: None,
            parameters: json!({"type":"object"}),
            source: Value::Null,
        }).collect();
        let effects = Arc::clone(&self.effects);
        let action = Arc::clone(&self.action);
        let timeout_ms = self.timeout_ms;
        let relay = self.relay.clone();
        let guard_returns = &mut self.guard_returns;
        run_codex_direct_responses_with_lifecycle_and_tool_step_reminder(
            &mut self.transport,
            &options,
            resume,
            &mut |tool, args, call_id| {
                let name = tool.name.clone();
                let worker_name = name.clone();
                let effects = Arc::clone(&effects);
                let action = Arc::clone(&action);
                let worker_attempt = attempt.clone();
                let relay = relay.clone();
                let call_id = call_id.to_string();
                let result = execute_routed_tool_with_timeout_ms(attempt, name, timeout_ms, move || {
                    effects.lock().expect("effect receipt").push(worker_name.clone());
                    action(&worker_attempt, &worker_name);
                    match relay {
                        Some(relay) => relay.request_tool_execution(json!({
                            "providerIdentifier":"custody-contract", "name":worker_name,
                            "toolName":worker_name, "args":args, "toolCallId":call_id,
                        })).map_err(coordinator_tool_execution_error),
                        None => Ok(json!({"effect":worker_name})),
                    }
                });
                guard_returns.push(GuardReturn {
                    tool: tool.name.clone(),
                    outcome_unknown: matches!(&result, Err(ProviderSessionError::ToolOutcomeUnknown(_))),
                    custody: attempt.effect_custody(),
                });
                result.map_err(|error| CodexDirectError::Tool(error.to_string()))
            },
            on_text_delta,
            &mut |checkpoint| {
                on_checkpoint(&RoutedProviderCheckpoint::Fabushi(checkpoint.clone()))
                    .map_err(|error| CodexDirectError::Transport(error.to_string()))
            },
            should_cancel,
            None,
        ).map(|result| result.text).map_err(ProviderSessionError::from)
    }
}

#[derive(Clone, Copy)]
enum StoreFault {
    None,
    Fail(usize),
    ExpireAfterWrite(usize),
    CancelAfterWrite(usize),
    EffectDuringWrite(usize),
}

struct FaultStore {
    production: ProductionRoutedProviderCheckpointStore,
    runtime: Arc<StreamAttemptRuntime>,
    cancellation: RoutedProviderCancellation,
    control: ControlSlot,
    effects: Arc<Mutex<Vec<String>>>,
    writes: AtomicUsize,
    fault: StoreFault,
}

impl RoutedProviderCheckpointStore for FaultStore {
    fn persist(&self, checkpoint: &RoutedProviderCheckpoint) -> Result<String, ProviderSessionError> {
        let ordinal = self.writes.fetch_add(1, Ordering::AcqRel) + 1;
        if matches!(self.fault, StoreFault::Fail(at) if at == ordinal) {
            return Err(ProviderSessionError::Transport(
                "connection reset while checkpoint persistence was unavailable".into(),
            ));
        }
        let cursor = self.production.persist(checkpoint)?;
        match self.fault {
            StoreFault::ExpireAfterWrite(at) if at == ordinal => {
                wait_until("expiry during checkpoint I/O", || self.runtime.active_attempt().is_none());
            }
            StoreFault::CancelAfterWrite(at) if at == ordinal => {
                self.cancellation.cancel("user cancelled during checkpoint I/O");
            }
            StoreFault::EffectDuringWrite(at) if at == ordinal => {
                let control = self.control.lock().expect("control slot").clone()
                    .expect("real executor supplied its attempt");
                let effects = Arc::clone(&self.effects);
                execute_routed_tool_with_timeout_ms(&control, "c".into(), 1_000, move || {
                    effects.lock().expect("concurrent receipt").push("c".into());
                    Ok(json!({"concurrent":true}))
                })?;
            }
            _ => {}
        }
        Ok(cursor)
    }
}

struct Harness {
    _root: TempRoot,
    runtime: Arc<StreamAttemptRuntime>,
    cancellation: RoutedProviderCancellation,
    store: FaultStore,
    executor: ResponsesExecutor,
}

impl Harness {
    fn new(script: Vec<Response>, action: ToolAction, fault: StoreFault, timeout: Duration) -> Self {
        let root = TempRoot::new();
        let runtime = Arc::new(StreamAttemptRuntime::new(policy(timeout)));
        let cancellation = RoutedProviderCancellation::default();
        let effects = Arc::new(Mutex::new(Vec::new()));
        let control = Arc::new(Mutex::new(None));
        let store = FaultStore {
            production: ProductionRoutedProviderCheckpointStore::new(&root.0, "agent", "stream"),
            runtime: Arc::clone(&runtime),
            cancellation: cancellation.clone(),
            control: Arc::clone(&control),
            effects: Arc::clone(&effects),
            writes: AtomicUsize::new(0),
            fault,
        };
        Self {
            _root: root, runtime, cancellation, store,
            executor: ResponsesExecutor {
                transport: Responses { pending: script.into(), requests: Vec::new() },
                effects, action, control, timeout_ms: 4_000,
                attempts: 0, resumed: Vec::new(),
                relay: None, guard_returns: Vec::new(),
            },
        }
    }

    fn run(&mut self) -> (Result<String, ProviderSessionError>, Vec<ProviderRetryEvent>, Vec<ProviderRetryReport>) {
        let adapter = ProductionTurnRunShellAdapter {
            policy: self.runtime.policy().clone(),
            watchdog_poll_interval: Duration::from_millis(1),
        };
        let mut retries = Vec::new();
        let mut reports = Vec::new();
        let result = adapter.run_with_attempt_runtime_reporting(
            Arc::clone(&self.runtime), &self.cancellation, &self.store, &mut self.executor,
            &mut |_, _| {}, &mut |event| retries.push(event.clone()),
            &mut |report| reports.push(report.clone()),
        );
        (result, retries, reports)
    }

    fn effects(&self) -> Vec<String> {
        self.executor.effects.lock().expect("receipts").clone()
    }

    fn checkpoints(&self) -> Vec<RoutedProviderCheckpoint> {
        let entries = match fs::read_dir(self.store.production.directory()) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
            Err(error) => panic!("read production checkpoints: {error}"),
        };
        entries.map(|entry| {
            let path = entry.expect("checkpoint entry").path();
            assert_eq!(path.extension().and_then(|value| value.to_str()), Some("json"));
            serde_json::from_slice(&fs::read(path).expect("checkpoint bytes"))
                .expect("real serialized provider checkpoint")
        }).collect()
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(generation) = self.runtime.active_attempt() {
            self.runtime.settle_attempt(generation);
        }
    }
}

fn assert_custody_refusal(
    result: Result<String, ProviderSessionError>,
    retries: &[ProviderRetryEvent],
    reports: &[ProviderRetryReport],
) {
    let error = result.expect_err("uncheckpointed effects must not be replayed");
    assert!(error.to_string().contains("refused automatic retry"), "{error}");
    assert!(retries.is_empty(), "an unsafe retry was announced: {retries:?}");
    let report = reports.last().expect("explicit custody diagnostic");
    assert_eq!(report.outcome, ProviderRetryOutcome::GaveUpIneligible);
    assert_eq!(report.error_type, "ToolEffectCustodyError");
    assert!(!report.resume_from_checkpoint);
}

#[test]
fn responses_effects_after_checkpoint_without_checkpoint_and_between_calls_do_not_replay() {
    for (label, first, expected) in [
        ("after-c0", vec![calls(&["a"]), calls(&["b"])], vec!["a", "b"]),
        ("without-c0", vec![calls(&["b"])], vec!["b"]),
        ("between-calls", vec![calls(&["b", "c"])], vec!["b"]),
    ] {
        let expected_requests = first.len();
        let mut script = first;
        // An unsafe retry reaches a second real B invocation, rather than a
        // mock "resumed" flag. The fixed path never consumes this response.
        script.extend([calls(&["b"]), Response::Text]);
        let action: ToolAction = Arc::new(|attempt, name| {
            if name == "b" {
                wait_until("watchdog while tool B owns its effect", || !attempt.is_current());
            }
        });
        let mut harness = Harness::new(script, action, StoreFault::None, WATCHDOG);
        let (result, retries, reports) = harness.run();
        assert_custody_refusal(result, &retries, &reports);
        assert_eq!(harness.effects(), expected, "{label}");
        assert_eq!(harness.executor.attempts, 1, "{label}");
        assert_eq!(harness.executor.transport.requests.len(), expected_requests, "{label}");
        let custody = harness.runtime.effect_custody();
        assert_eq!(custody.pending_effects, 0);
        assert!(custody.unknown_result.is_none());
        assert_eq!(custody.checkpoint_revision, u64::from(label == "after-c0"));
        assert_eq!(harness.checkpoints().len(), usize::from(label == "after-c0"));
    }
}

#[test]
fn responses_checkpoint_io_failure_expiry_and_new_effect_cannot_accept_stale_coverage() {
    for fault in [
        StoreFault::Fail(2),
        StoreFault::ExpireAfterWrite(2),
        StoreFault::EffectDuringWrite(2),
    ] {
        let mut harness = Harness::new(
            vec![calls(&["a"]), calls(&["b"]), calls(&["b"]), Response::Text],
            Arc::new(|_, _| {}), fault, WATCHDOG,
        );
        let (result, retries, reports) = harness.run();
        assert_custody_refusal(result, &retries, &reports);
        assert_eq!(harness.executor.attempts, 1);
        assert_eq!(harness.executor.transport.requests.len(), 2);
        let custody = harness.runtime.effect_custody();
        assert_eq!(custody.checkpoint_revision, 1, "C0 alone covers durable accepted effects");
        assert_eq!(custody.pending_effects, 0);
        let concurrent = matches!(fault, StoreFault::EffectDuringWrite(_));
        assert_eq!(harness.effects(), if concurrent { vec!["a", "b", "c"] } else { vec!["a", "b"] });
        assert_eq!(custody.started_revision, if concurrent { 3 } else { 2 });
        assert_eq!(harness.checkpoints().len(), if matches!(fault, StoreFault::Fail(_)) { 1 } else { 2 });
        for checkpoint in harness.checkpoints() {
            let RoutedProviderCheckpoint::Fabushi(checkpoint) = checkpoint else { panic!("provider changed") };
            assert_eq!(checkpoint.model.as_deref(), Some(MODEL));
        }
    }
}

#[test]
fn responses_pre_effect_retry_and_safe_checkpoint_retry_keep_model_and_completed_tool() {
    for script in [
        vec![Response::TransportFailure, Response::Text],
        vec![calls(&["a"]), Response::Stall, Response::Text],
    ] {
        let expected_effects = usize::from(matches!(script.first(), Some(Response::Calls(_))));
        let mut harness = Harness::new(script, Arc::new(|_, _| {}), StoreFault::None, WATCHDOG);
        let (result, retries, _) = harness.run();
        assert_eq!(result.expect("safe retry"), "done");
        assert_eq!(harness.executor.attempts, 2);
        assert_eq!(retries.len(), 1);
        assert_eq!(harness.effects().len(), expected_effects);
        assert_eq!(harness.checkpoints().len(), expected_effects);
        assert!(harness.runtime.effect_custody().retry_block_reason().is_none());
        assert!(harness.executor.transport.requests.iter().all(|request| request["model"] == MODEL));
        if expected_effects == 1 {
            let Some(RoutedProviderCheckpoint::Fabushi(checkpoint)) = &harness.executor.resumed[1]
            else { panic!("safe retry must use its accepted Fabushi checkpoint") };
            assert_eq!(checkpoint.model.as_deref(), Some(MODEL));
            assert!(checkpoint.input.iter().any(|item| {
                item["type"] == "function_call_output" && item["call_id"] == "call-a"
            }));
            assert_eq!(harness.effects(), vec!["a"]);
            assert!(retries[0].resume_from_checkpoint);
        } else {
            assert!(harness.executor.resumed.iter().all(Option::is_none));
        }
    }
}

#[derive(Default)]
struct ReleaseGate {
    open: Mutex<bool>,
    changed: Condvar,
}

impl ReleaseGate {
    fn wait(&self) {
        self.wait_for(BOUND);
    }

    fn wait_for(&self, timeout: Duration) {
        let guard = self.open.lock().expect("release gate");
        let (guard, _) = self.changed.wait_timeout_while(guard, timeout, |open| !*open)
            .expect("bounded release gate");
        assert!(*guard, "owned tool was not released within its bound");
    }

    fn release(&self) {
        *self.open.lock().expect("release gate") = true;
        self.changed.notify_all();
    }
}

struct ReleaseOnDrop(Arc<ReleaseGate>);

impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[derive(Clone)]
enum DownstreamReply {
    Timeout,
    Close,
    Remote(&'static str),
    MissingResult,
    Value(Value),
}

/// The event subscriber starts a real, separately owned non-idempotent worker.
/// Publication waits only for its entry receipt, so the relay timeout starts
/// after the effect. The external worker remains gated after the Host returns.
struct GatedDownstream {
    relay: Arc<CoordinatorToolRelay>,
    events: mpsc::Receiver<Value>,
    effects: Arc<AtomicUsize>,
    gate: Arc<ReleaseGate>,
    workers: Arc<Mutex<Vec<thread::JoinHandle<Result<Value, CoordinatorToolRelayError>>>>>,
    _subscription: HostEventSubscription,
}

impl GatedDownstream {
    fn new(reply: DownstreamReply, receipt: PathBuf) -> Self {
        let events = GatewayEventHub::default();
        let event_rx = events.subscribe();
        let timeout = if matches!(&reply, DownstreamReply::Timeout) {
            Duration::from_millis(250)
        } else {
            Duration::from_secs(3)
        };
        let relay = Arc::new(CoordinatorToolRelay::with_timeout(events.clone(), timeout));
        let effects = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(ReleaseGate::default());
        let workers = Arc::new(Mutex::new(Vec::new()));
        let worker_effects = Arc::clone(&effects);
        let worker_gate = Arc::clone(&gate);
        let worker_handles = Arc::clone(&workers);
        let weak_relay = Arc::downgrade(&relay);
        let subscription = events.subscribe_listener(move |event| {
            if event["channel"] != RUNNER_TOOL_REQUEST_EVENT_CHANNEL { return; }
            assert_eq!(event["payload"]["method"], ROUTED_TOOL_EXECUTE_METHOD);
            let relay = weak_relay.upgrade().expect("live execution relay");
            let request_id = event["payload"]["requestId"].as_str().expect("request ID").to_string();
            let name = event["payload"]["args"]["toolName"].as_str().expect("actual Responses tool");
            if name == "a" {
                relay.resolve(&json!({
                    "requestId":request_id,"ok":true,
                    "result":{"success":{"content":[],"structuredContent":{"receipt":"c0"}}},
                })).expect("settled C0 result");
                return;
            }
            assert_eq!(name, "b");
            let call_id = event["payload"]["args"]["toolCallId"].as_str()
                .expect("real Responses call ID").to_string();
            let late_relay = Arc::clone(&relay);
            let late_id = request_id.clone();
            let receipt = receipt.clone();
            let effects = Arc::clone(&worker_effects);
            let gate = Arc::clone(&worker_gate);
            let (entered_tx, entered_rx) = mpsc::channel();
            let worker = thread::spawn(move || {
                let mut file = fs::OpenOptions::new().create(true).append(true).open(receipt)
                    .expect("external non-idempotent effect receipt");
                writeln!(file, "{call_id}").expect("append external effect");
                file.sync_all().expect("durable external effect");
                effects.fetch_add(1, Ordering::AcqRel);
                entered_tx.send(()).expect("observe actual external entry");
                gate.wait();
                late_relay.resolve(&json!({
                    "requestId":late_id,"ok":true,
                    "result":{"success":{"content":[],"structuredContent":{"late":true}}},
                }))
            });
            worker_handles.lock().expect("owned external workers").push(worker);
            entered_rx.recv_timeout(BOUND).expect("effect starts before local reply or timeout");
            match &reply {
                DownstreamReply::Timeout => {}
                DownstreamReply::Close => relay.cancel_all("Host closes after external dispatch"),
                DownstreamReply::Remote(message) => {
                    relay.resolve(&json!({"requestId":request_id,"ok":false,"error":message}))
                        .expect("remote failure ends only the local waiter");
                }
                DownstreamReply::MissingResult => {
                    relay.resolve(&json!({"requestId":request_id,"ok":true}))
                        .expect("control success without an execution result");
                }
                DownstreamReply::Value(value) => {
                    relay.resolve(&json!({"requestId":request_id,"ok":true,"result":value}))
                        .expect("successful control reply carrying uncertain MCP outcome");
                }
            }
        });
        Self { relay, events: event_rx, effects, gate, workers, _subscription: subscription }
    }

    fn release(&mut self) -> Vec<Result<Value, CoordinatorToolRelayError>> {
        self.gate.release();
        let workers = std::mem::take(&mut *self.workers.lock().expect("owned external workers"));
        workers.into_iter().map(|worker| {
            wait_until("external worker late return", || worker.is_finished());
            worker.join().expect("owned external worker result")
        }).collect()
    }
}

impl Drop for GatedDownstream {
    fn drop(&mut self) {
        self.gate.release();
        if let Ok(mut workers) = self.workers.lock() {
            let deadline = Instant::now() + BOUND;
            for worker in workers.drain(..) {
                while !worker.is_finished() && Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(1));
                }
                if worker.is_finished() { let _ = worker.join(); }
            }
        }
    }
}

#[test]
fn downstream_unknown_outcomes_block_responses_checkpoint_and_replay_after_local_return() {
    let cases = [
        ("Host timeout", DownstreamReply::Timeout),
        ("post-publication close", DownstreamReply::Close),
        ("Coordinator timeout", DownstreamReply::Remote("COORDINATOR_DISCONNECTED: command timed out")),
        ("opaque remote error", DownstreamReply::Remote("la demande a échoué; opaque response 42")),
        ("plain MCP error", DownstreamReply::Value(json!({
            "result":{"case":"error","value":{"error":"adapter failed after dispatch"}},
        }))),
        ("protobuf MCP error", DownstreamReply::Value(json!({"error":{"error":"opaque adapter failure"}}))),
        ("missing control result", DownstreamReply::MissingResult),
        ("null MCP result", DownstreamReply::Value(Value::Null)),
        ("untyped object", DownstreamReply::Value(json!({"receipt":"not a definitive result"}))),
        ("malformed success", DownstreamReply::Value(json!({"success":{"content":"not an array"}}))),
        ("missing oneof payload", DownstreamReply::Value(json!({"result":{"case":"success"}}))),
        ("competing oneof arms", DownstreamReply::Value(json!({"success":{},"error":{"error":"ambiguous"}}))),
    ];
    for (label, reply) in cases {
        for with_c0 in [false, true] {
            let mut script = Vec::new();
            if with_c0 { script.push(calls(&["a"])); }
            script.extend([calls(&["b"]), calls(&["b"]), Response::Text]);
            let mut harness = Harness::new(
                script, Arc::new(|_, _| {}), StoreFault::None, Duration::from_secs(3),
            );
            let receipt = harness._root.0.join("downstream-effects.txt");
            let mut downstream = GatedDownstream::new(reply.clone(), receipt.clone());
            harness.executor.relay = Some(Arc::clone(&downstream.relay));
            let (result, retries, reports) = harness.run();
            assert_custody_refusal(result, &retries, &reports);
            assert!(!reports.last().expect("custody report").watchdog_expired,
                "{label}: relay uncertainty must precede the provider watchdog");
            assert_eq!(harness.executor.attempts, 1, "{label}");
            assert_eq!(harness.executor.transport.requests.len(), 1 + usize::from(with_c0), "{label}");
            assert_eq!(harness.effects(), if with_c0 { vec!["a", "b"] } else { vec!["b"] });
            assert_eq!(downstream.effects.load(Ordering::Acquire), 1, "{label}");
            assert_eq!(fs::read_to_string(&receipt).expect("durable external receipt"), "call-b\n");
            {
                let workers = downstream.workers.lock().expect("external worker custody");
                assert_eq!(workers.len(), 1);
                assert!(!workers[0].is_finished(), "{label}: external worker must outlive Host return");
            }
            let returned = harness.executor.guard_returns.last().expect("actual guarded return");
            assert_eq!(returned.tool, "b");
            assert!(returned.outcome_unknown, "{label}: typed result must reach the common guard");
            assert_eq!(returned.custody.pending_effects, 0, "the local Host worker has ended");
            assert_eq!(returned.custody.unknown_result, Some(ToolEffectUnknownResult::DownstreamOutcomeUnknown));
            let custody = harness.runtime.effect_custody();
            assert_eq!(custody, returned.custody, "error serialization cannot erase unknown custody");
            assert_eq!(custody.started_revision, 1 + u64::from(with_c0));
            assert_eq!(custody.checkpoint_revision, u64::from(with_c0));
            assert_eq!(harness.store.writes.load(Ordering::Acquire), usize::from(with_c0),
                "the real checkpoint owner must reject the uncertain result before persistence");
            let checkpoints = harness.checkpoints();
            assert_eq!(checkpoints.len(), usize::from(with_c0));
            for checkpoint in checkpoints {
                let RoutedProviderCheckpoint::Fabushi(checkpoint) = checkpoint else { panic!("provider changed") };
                assert_eq!(checkpoint.model.as_deref(), Some(MODEL));
                assert!(checkpoint.input.iter().any(|item| item["call_id"] == "call-a"));
                assert!(!checkpoint.input.iter().any(|item| item["call_id"] == "call-b"));
            }
            assert!(harness.executor.transport.requests.iter().all(|request| request["model"] == MODEL));
            let checkpoint_bytes = if with_c0 {
                let path = fs::read_dir(harness.store.production.directory()).expect("C0 directory")
                    .next().expect("C0 file").expect("C0 entry").path();
                Some((path.clone(), fs::read(path).expect("original C0 bytes")))
            } else { None };
            let published = downstream.events.try_iter().collect::<Vec<_>>();
            assert_eq!(published.len(), 1 + usize::from(with_c0));
            assert_eq!(downstream.relay.pending_count(), 0);
            let replacement = RoutedProviderAttemptControl::new(
                Arc::clone(&harness.runtime), harness.runtime.begin_attempt(), harness.cancellation.clone(),
            );
            assert!(replacement.admit_tool().is_err(), "replacement attempt cannot bypass unknown custody");
            let late = downstream.release();
            assert_eq!(late.len(), 1);
            assert!(matches!(&late[0], Err(CoordinatorToolRelayError::Protocol(_))));
            assert!(matches!(downstream.relay.resolve(&json!({
                "requestId":published.last().expect("published B request")["payload"]["requestId"],
                "ok":false,"error":"late external failure",
            })), Err(CoordinatorToolRelayError::Protocol(_))));
            assert_eq!(harness.runtime.effect_custody(), custody, "{label}: late completion cannot clear unknown");
            let relay = Arc::clone(&downstream.relay);
            assert!(execute_routed_tool_with_timeout_ms(&replacement, "b".into(), 1_000, move || {
                relay.request_tool_execution(json!({"toolName":"b","args":{}}))
                    .map_err(coordinator_tool_execution_error)
            }).is_err(), "the real guard must refuse another dispatch after the late result");
            assert_eq!(harness.runtime.effect_custody(), custody);
            assert_eq!(downstream.effects.load(Ordering::Acquire), 1);
            assert_eq!(fs::read_to_string(&receipt).expect("final external receipt"), "call-b\n");
            assert!(matches!(downstream.events.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert_eq!(harness.checkpoints().len(), usize::from(with_c0));
            if let Some((path, bytes)) = checkpoint_bytes {
                assert_eq!(fs::read(path).expect("retained C0 bytes"), bytes, "late reply cannot corrupt C0");
            }
        }
    }
}

#[test]
fn definitive_relay_results_remain_checkpoint_eligible_and_preserve_returned_data() {
    for (value, executes) in [
        (json!({"result":{"case":"success","value":{"content":[],"isError":false}}}), true),
        (json!({"result":{"case":"success","value":{"content":[],"isError":true}}}), true),
        (json!({"success":{}}), true),
        (json!({"success":{"content":[{"text":{"text":"server error"}}],"isError":true}}), true),
        (json!({"content":[{"type":"text","text":"timeout is genuine tool data"}],"isError":true}), true),
        (json!({"structuredContent":{
            "error":"connection reset is user data", "result":{"case":"error","value":null},
        }}), true),
        (json!({"result":{"case":"rejected","value":{"reason":"explicit preflight denial","isReadonly":false}}}), false),
        (json!({"permissionDenied":{"error":"known refusal","isReadonly":true}}), false),
    ] {
        let mut harness = Harness::new(
            vec![calls(&["b"]), Response::Text], Arc::new(|_, _| {}),
            StoreFault::None, Duration::from_secs(3),
        );
        let events = GatewayEventHub::default();
        let event_rx = events.subscribe();
        let relay = Arc::new(CoordinatorToolRelay::with_timeout(events.clone(), BOUND));
        let weak_relay = Arc::downgrade(&relay);
        let effects = Arc::new(AtomicUsize::new(0));
        let worker_effects = Arc::clone(&effects);
        let returned_value = value.clone();
        let _subscription = events.subscribe_listener(move |event| {
            if event["channel"] != RUNNER_TOOL_REQUEST_EVENT_CHANNEL { return; }
            assert_eq!(event["payload"]["method"], ROUTED_TOOL_EXECUTE_METHOD);
            if executes { worker_effects.fetch_add(1, Ordering::AcqRel); }
            weak_relay.upgrade().expect("live relay").resolve(&json!({
                "requestId":event["payload"]["requestId"],"ok":true,"result":returned_value,
            })).expect("definitive external response");
        });
        harness.executor.relay = Some(relay);
        let (result, retries, reports) = harness.run();
        assert_eq!(result.expect("known result remains usable"), "done");
        assert!(retries.is_empty());
        assert!(reports.is_empty());
        assert_eq!(harness.executor.attempts, 1);
        assert_eq!(event_rx.try_iter().count(), 1);
        assert_eq!(effects.load(Ordering::Acquire), usize::from(executes));
        let returned = harness.executor.guard_returns.last().expect("known guarded return");
        assert!(!returned.outcome_unknown);
        assert_eq!(returned.custody.pending_effects, 0);
        assert!(returned.custody.unknown_result.is_none());
        let custody = harness.runtime.effect_custody();
        assert_eq!((custody.started_revision, custody.checkpoint_revision), (1, 1));
        assert!(custody.retry_block_reason().is_none());
        let checkpoints = harness.checkpoints();
        assert_eq!(checkpoints.len(), 1);
        let RoutedProviderCheckpoint::Fabushi(checkpoint) = &checkpoints[0] else { panic!("provider changed") };
        assert_eq!(checkpoint.model.as_deref(), Some(MODEL));
        let output = checkpoint.input.iter().find(|item| {
            item["type"] == "function_call_output" && item["call_id"] == "call-b"
        }).expect("actual Responses output paired in the production checkpoint");
        assert_eq!(serde_json::from_str::<Value>(output["output"].as_str().expect("serialized output"))
            .expect("tool result JSON"), value, "checkpoint preserves every returned field");
        assert!(harness.executor.transport.requests.iter().all(|request| request["model"] == MODEL));
    }
}

#[test]
fn prepublication_refusal_and_list_failure_add_no_unknown_effect_custody() {
    let mut harness = Harness::new(
        vec![calls(&["b"]), Response::Text], Arc::new(|_, _| {}),
        StoreFault::None, Duration::from_secs(3),
    );
    let events = GatewayEventHub::default();
    let event_rx = events.subscribe();
    let relay = Arc::new(CoordinatorToolRelay::with_timeout(events, BOUND));
    relay.cancel_all("closed before execution publication");
    harness.executor.relay = Some(Arc::clone(&relay));
    let (result, retries, reports) = harness.run();
    assert_eq!(result.expect("known refusal may be returned to the provider"), "done");
    assert!(retries.is_empty());
    assert!(reports.is_empty());
    let returned = harness.executor.guard_returns.last().expect("guarded preflight refusal");
    assert!(!returned.outcome_unknown);
    assert_eq!(returned.custody.pending_effects, 0);
    assert!(returned.custody.unknown_result.is_none());
    let custody = harness.runtime.effect_custody();
    assert_eq!(custody.checkpoint_revision, 1);
    assert!(custody.retry_block_reason().is_none());
    assert_eq!(harness.checkpoints().len(), 1);
    assert!(matches!(relay.request(ROUTED_TOOL_LIST_METHOD, json!({})),
        Err(CoordinatorToolRelayError::Closed(_))));
    assert_eq!(harness.runtime.effect_custody(), custody, "discovery must not create execution custody");
    assert_eq!(relay.pending_count(), 0);
    assert!(matches!(event_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));

    let discovery_events = GatewayEventHub::default();
    let discovery_rx = discovery_events.subscribe();
    let discovery = Arc::new(CoordinatorToolRelay::with_timeout(discovery_events.clone(), BOUND));
    let weak_discovery = Arc::downgrade(&discovery);
    let _subscription = discovery_events.subscribe_listener(move |event| {
        if event["channel"] != RUNNER_TOOL_REQUEST_EVENT_CHANNEL { return; }
        assert_eq!(event["payload"]["method"], ROUTED_TOOL_LIST_METHOD);
        weak_discovery.upgrade().expect("live discovery relay").resolve(&json!({
            "requestId":event["payload"]["requestId"],"ok":false,"error":"opaque discovery failure",
        })).expect("published discovery failure");
    });
    assert!(matches!(discovery.request(ROUTED_TOOL_LIST_METHOD, json!({})),
        Err(CoordinatorToolRelayError::Remote { .. })));
    assert_eq!(discovery_rx.try_iter().count(), 1);
    assert_eq!(discovery.pending_count(), 0);
    assert_eq!(harness.runtime.effect_custody(), custody, "remote discovery failure is not an effect");
}

#[test]
fn timed_out_tool_keeps_worker_receipt_and_late_result_stays_unknown_across_attempts() {
    let gate = Arc::new(ReleaseGate::default());
    let _release = ReleaseOnDrop(Arc::clone(&gate));
    let action_gate = Arc::clone(&gate);
    let mut harness = Harness::new(
        vec![calls(&["b"]), calls(&["b"]), Response::Text],
        Arc::new(move |_, _| action_gate.wait()),
        StoreFault::None, Duration::from_secs(3),
    );
    harness.executor.timeout_ms = 500;
    let (result, retries, reports) = harness.run();
    assert_custody_refusal(result, &retries, &reports);
    assert_eq!(harness.effects(), vec!["b"]);
    assert_eq!(harness.executor.attempts, 1);
    assert_eq!(harness.checkpoints().len(), 0, "timeout error JSON is not settled effect coverage");
    let before = harness.runtime.effect_custody();
    assert_eq!(before.pending_effects, 1, "the actual tool still owns its worker receipt");
    assert_eq!(before.unknown_result, Some(ToolEffectUnknownResult::TimedOut));
    let replacement = RoutedProviderAttemptControl::new(
        Arc::clone(&harness.runtime), harness.runtime.begin_attempt(), harness.cancellation.clone(),
    );
    assert!(replacement.admit_tool().is_err(), "a new attempt cannot bypass unresolved custody");
    assert_eq!(harness.runtime.effect_custody(), before);
    gate.release();
    wait_until("late tool completion receipt", || harness.runtime.effect_custody().pending_effects == 0);
    let after = harness.runtime.effect_custody();
    assert_eq!(after.unknown_result, before.unknown_result);
    assert_eq!(after.checkpoint_revision, 0);
    harness.runtime.begin_attempt();
    assert_eq!(harness.runtime.effect_custody(), after, "a fresh attempt must not erase custody");
}

#[test]
fn a_lost_tool_worker_cannot_turn_error_json_into_an_accepted_checkpoint() {
    let mut harness = Harness::new(
        vec![calls(&["b"]), calls(&["b"]), Response::Text],
        Arc::new(|_, _| panic!("deliberate owned tool worker loss")),
        StoreFault::None, WATCHDOG,
    );
    let (result, retries, reports) = harness.run();
    assert_custody_refusal(result, &retries, &reports);
    assert_eq!(harness.effects(), vec!["b"]);
    assert_eq!(harness.executor.attempts, 1);
    assert_eq!(harness.runtime.effect_custody().pending_effects, 0);
    assert_eq!(harness.runtime.effect_custody().unknown_result, Some(ToolEffectUnknownResult::WorkerLost));
    assert!(harness.checkpoints().is_empty());
}

#[test]
fn user_cancellation_between_tools_and_during_checkpoint_io_wins_without_retry() {
    for during_io in [false, true] {
        let mut harness = Harness::new(
            vec![calls(&["a"]), calls(&["b", "c"]), Response::Text],
            Arc::new(|_, _| {}),
            if during_io { StoreFault::CancelAfterWrite(2) } else { StoreFault::None },
            WATCHDOG,
        );
        if !during_io {
            let cancellation = harness.cancellation.clone();
            harness.executor.action = Arc::new(move |_, name| {
                if name == "b" {
                    cancellation.cancel("user cancelled between tools");
                }
            });
        }
        let (result, retries, reports) = harness.run();
        assert!(matches!(result, Err(ProviderSessionError::Cancelled(_))));
        assert!(retries.is_empty());
        assert!(reports.is_empty(), "cancellation must not be reported as a retry decision");
        assert_eq!(harness.executor.attempts, 1);
        assert_eq!(harness.effects(), if during_io { vec!["a", "b", "c"] } else { vec!["a", "b"] });
        assert_eq!(harness.runtime.effect_custody().checkpoint_revision, 1);
    }
}

fn open_mcp_request(url: &str, body: Value) -> TcpStream {
    let (authority, path) = url.strip_prefix("http://").expect("local HTTP").split_once('/')
        .expect("MCP endpoint");
    let address = authority.parse().expect("socket address");
    let mut stream = TcpStream::connect_timeout(&address, BOUND).expect("connect MCP");
    stream.set_read_timeout(Some(BOUND)).expect("bounded MCP read");
    stream.set_write_timeout(Some(BOUND)).expect("bounded MCP write");
    let body = serde_json::to_vec(&body).expect("MCP request");
    write!(
        stream,
        "POST /{path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len(),
    ).expect("MCP headers");
    stream.write_all(&body).expect("MCP payload");
    stream
}

fn post_mcp(url: &str, body: Value) -> Value {
    let mut stream = open_mcp_request(url, body);
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("MCP response");
    serde_json::from_str(response.split_once("\r\n\r\n").expect("HTTP framing").1)
        .expect("MCP JSON")
}

struct CountedBridge {
    runtime: Arc<StreamAttemptRuntime>,
    effects: Arc<AtomicUsize>,
    wait_for_expiry: bool,
}

impl RoutedToolBridge for CountedBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            origin: None,
            name: "effect".into(), provider_identifier: "custody-contract".into(),
            tool_name: "effect".into(), description: None,
            input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(&self, _tool: &RoutedToolDefinition, _args: Value, call_id: &str) -> Result<Value, ProviderSessionError> {
        assert!(!call_id.is_empty());
        self.effects.fetch_add(1, Ordering::AcqRel);
        if self.wait_for_expiry {
            wait_until("MCP tool observing its expired owner", || self.runtime.active_attempt().is_none());
        }
        Ok(json!({"effect":"performed"}))
    }
}

fn mcp_call(url: &str) -> Value {
    post_mcp(url, json!({
        "jsonrpc":"2.0", "id":2, "method":"tools/call",
        "params":{"name":"effect","arguments":{}}
    }))
}

#[test]
fn real_mcp_endpoint_keeps_immutable_attempt_admission_and_counts_effects() {
    let runtime = Arc::new(StreamAttemptRuntime::new(policy(WATCHDOG)));
    let cancellation = RoutedProviderCancellation::default();
    let first = RoutedProviderAttemptControl::new(
        Arc::clone(&runtime), runtime.begin_attempt(), cancellation.clone(),
    );
    let effects = Arc::new(AtomicUsize::new(0));
    let bridge: Arc<dyn RoutedToolBridge> = Arc::new(CountedBridge {
        runtime: Arc::clone(&runtime), effects: Arc::clone(&effects), wait_for_expiry: false,
    });
    let mut old_server = start_routed_mcp_server_for_attempt(Arc::clone(&bridge), first, false)
        .expect("first real MCP endpoint");
    post_mcp(old_server.url(), json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}));
    let second = RoutedProviderAttemptControl::new(
        Arc::clone(&runtime), runtime.begin_attempt(), cancellation,
    );
    let mut current_server = start_routed_mcp_server_for_attempt(bridge, second.clone(), false)
        .expect("replacement real MCP endpoint");
    assert_ne!(old_server.url(), current_server.url());
    assert_eq!(mcp_call(old_server.url())["result"]["isError"], true);
    assert_eq!(effects.load(Ordering::Acquire), 0, "old endpoint must not borrow the new attempt");
    post_mcp(current_server.url(), json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}));
    assert_eq!(post_mcp(current_server.url(), json!({
        "jsonrpc":"2.0", "id":9, "method":"tools/call",
        "params":{"name":"unregistered-tool","arguments":{}}
    }))["result"]["isError"], true);
    assert_eq!(runtime.effect_custody().started_revision, 0);
    assert!(runtime.effect_custody().unknown_result.is_none());
    assert_eq!(mcp_call(current_server.url())["result"]["isError"], false);
    assert_eq!(effects.load(Ordering::Acquire), 1);
    assert_eq!(runtime.effect_custody().started_revision, 1);
    assert_eq!(runtime.effect_custody().pending_effects, 0);
    assert_eq!(runtime.effect_custody().checkpoint_revision, 0);
    second.close_admission();
    assert_eq!(mcp_call(current_server.url())["result"]["isError"], true);
    assert_eq!(effects.load(Ordering::Acquire), 1);
    old_server.close();
    current_server.close();
}

/// A real Unix TCP reset forces the shipping response writer to fail after
/// the tool delegate has performed its effect. Windows needs its own socket
/// fault instrument in the final platform matrix; this oracle runs on macOS CI.
#[cfg(unix)]
#[test]
fn real_mcp_response_loss_refuses_more_effects_on_the_same_attempt() {
    use std::os::fd::AsRawFd;
    use std::sync::mpsc;

    struct GatedBridge {
        effects: Arc<AtomicUsize>,
        entered: mpsc::Sender<()>,
        gate: Arc<ReleaseGate>,
    }

    impl RoutedToolBridge for GatedBridge {
        fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
            Ok(vec![RoutedToolDefinition {
                origin: None,
                name: "effect".into(), provider_identifier: "custody-contract".into(),
                tool_name: "effect".into(), description: None,
                input_schema: json!({"type":"object"}),
            }])
        }

        fn call_tool(&self, _tool: &RoutedToolDefinition, _args: Value, _call_id: &str)
            -> Result<Value, ProviderSessionError>
        {
            self.effects.fetch_add(1, Ordering::AcqRel);
            self.entered.send(()).expect("observe actual tool execution");
            self.gate.wait();
            Ok(json!({"effect":"performed", "output":"x".repeat(256 * 1024)}))
        }
    }

    let runtime = Arc::new(StreamAttemptRuntime::new(policy(WATCHDOG)));
    let attempt = RoutedProviderAttemptControl::new(
        Arc::clone(&runtime), runtime.begin_attempt(), RoutedProviderCancellation::default(),
    );
    let effects = Arc::new(AtomicUsize::new(0));
    let gate = Arc::new(ReleaseGate::default());
    let (entered_tx, entered_rx) = mpsc::channel();
    let mut server = start_routed_mcp_server_for_attempt(Arc::new(GatedBridge {
        effects: Arc::clone(&effects), entered: entered_tx, gate: Arc::clone(&gate),
    }), attempt.clone(), false).expect("real MCP response-loss endpoint");
    // Drop the release guard before the server if an assertion unwinds.
    let _release = ReleaseOnDrop(Arc::clone(&gate));
    post_mcp(server.url(), json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}));
    let stream = open_mcp_request(server.url(), json!({
        "jsonrpc":"2.0", "id":2, "method":"tools/call",
        "params":{"name":"effect","arguments":{}}
    }));
    entered_rx.recv_timeout(BOUND).expect("non-idempotent effect reached its result gate");
    assert_eq!(runtime.effect_custody().pending_effects, 1);
    let linger = libc::linger { l_onoff: 1, l_linger: 0 };
    // SAFETY: a live owned socket and correctly sized initialized option are
    // passed synchronously; the socket is closed only after setsockopt returns.
    let reset = unsafe {
        libc::setsockopt(
            stream.as_raw_fd(), libc::SOL_SOCKET, libc::SO_LINGER,
            (&linger as *const libc::linger).cast(),
            std::mem::size_of::<libc::linger>() as libc::socklen_t,
        )
    };
    assert_eq!(reset, 0, "configure owned peer reset: {}", std::io::Error::last_os_error());
    drop(stream);
    gate.release();
    wait_until("observed failed MCP response delivery", || {
        runtime.effect_custody().unknown_result == Some(ToolEffectUnknownResult::DiscardedResult)
    });
    let custody = runtime.effect_custody();
    assert_eq!(custody.started_revision, 1);
    assert_eq!(custody.pending_effects, 0);
    assert_eq!(custody.checkpoint_revision, 0);
    assert_eq!(effects.load(Ordering::Acquire), 1);
    assert!(attempt.admit_tool().is_err());
    assert_eq!(mcp_call(server.url())["result"]["isError"], true);
    assert_eq!(effects.load(Ordering::Acquire), 1, "known result loss must block another effect");
    server.close();
}

struct McpExecutor {
    bridge: Arc<dyn RoutedToolBridge>,
    attempts: usize,
}

impl RoutedProviderAttemptExecutor for McpExecutor {
    fn run_attempt(
        &mut self,
        attempt: &RoutedProviderAttemptControl,
        _resume: Option<&RoutedProviderCheckpoint>,
        _on_delta: &mut dyn FnMut(&str, &str),
        _on_checkpoint: &mut dyn FnMut(&RoutedProviderCheckpoint) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        self.attempts += 1;
        let mut server = start_routed_mcp_server_for_attempt(Arc::clone(&self.bridge), attempt.clone(), false)?;
        post_mcp(server.url(), json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}));
        let response = mcp_call(server.url());
        server.close();
        assert_eq!(response["result"]["isError"], false, "the Host tool really returned");
        assert!(should_cancel(), "real watchdog must expire while the MCP effect runs");
        Err(ProviderSessionError::Cancelled("provider stopped after MCP effect".into()))
    }
}

#[test]
fn real_mcp_effect_prevents_provider_retry_after_attempt_expiry() {
    let root = TempRoot::new();
    let runtime = Arc::new(StreamAttemptRuntime::new(policy(WATCHDOG)));
    let effects = Arc::new(AtomicUsize::new(0));
    let mut executor = McpExecutor {
        bridge: Arc::new(CountedBridge {
            runtime: Arc::clone(&runtime), effects: Arc::clone(&effects), wait_for_expiry: true,
        }),
        attempts: 0,
    };
    let store = ProductionRoutedProviderCheckpointStore::new(&root.0, "agent", "mcp-stream");
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(WATCHDOG), watchdog_poll_interval: Duration::from_millis(1),
    };
    let mut retries = Vec::new();
    let mut reports = Vec::new();
    let result = adapter.run_with_attempt_runtime_reporting(
        Arc::clone(&runtime), &RoutedProviderCancellation::default(), &store, &mut executor,
        &mut |_, _| {}, &mut |event| retries.push(event.clone()),
        &mut |report| reports.push(report.clone()),
    );
    assert_custody_refusal(result, &retries, &reports);
    assert_eq!(executor.attempts, 1);
    assert_eq!(effects.load(Ordering::Acquire), 1);
    assert_eq!(runtime.effect_custody().started_revision, 1);
    assert_eq!(runtime.effect_custody().checkpoint_revision, 0);
    assert_eq!(runtime.effect_custody().pending_effects, 0);
}


// This oracle reaches the private production attempt executor. The separate
// generated-stream oracle below also enters the real turn owner and facade.
// Child-scoped transport configuration cannot affect other tests.
const WIRE_TEST: &str = "production_fabushi_executor_refuses_replay_after_tool_expiry";
const WIRE_ROOT_ENV: &str = "FABUSHI_EFFECT_CUSTODY_WIRE_ROOT";
const WIRE_TOKEN_ENV: &str = "FABUSHI_EFFECT_CUSTODY_WIRE_TOKEN";

struct WireBridge {
    inner: CountedBridge,
    receipt: PathBuf,
}

impl RoutedToolBridge for WireBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        self.inner.list_tools()
    }

    fn call_tool(&self, tool: &RoutedToolDefinition, args: Value, call_id: &str) -> Result<Value, ProviderSessionError> {
        let mut file = fs::OpenOptions::new().create(true).append(true).open(&self.receipt)
            .expect("non-idempotent production tool receipt");
        writeln!(file, "{call_id}").expect("append real effect receipt");
        file.sync_all().expect("persist real effect receipt");
        self.inner.call_tool(tool, args, call_id)
    }
}

fn run_wire_child(root: PathBuf) {
    use mahayana_host_runtime::extensions::inference::provider_session::{ProviderMessage, RoutedProvider};
    use mahayana_host_runtime::host_request_context::HostRequestContext;
    use mahayana_host_runtime::runner::routed_provider_runtime::{
        RoutedProviderRun, RunnerRequestContextSnapshot, run_routed_provider_in_runner,
    };

    let runtime = Arc::new(StreamAttemptRuntime::new(policy(Duration::from_secs(2))));
    let effects = Arc::new(AtomicUsize::new(0));
    let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(&root, "wire-agent", "wire-stream"));
    let reports = Arc::new(Mutex::new(Vec::<ProviderRetryReport>::new()));
    let report_sink = Arc::clone(&reports);
    let retry_count = Arc::new(AtomicUsize::new(0));
    let retry_sink = Arc::clone(&retry_count);
    let receipt = root.join("tool-receipts.txt");
    let bridge: Arc<dyn RoutedToolBridge> = Arc::new(WireBridge {
        inner: CountedBridge {
            runtime: Arc::clone(&runtime), effects: Arc::clone(&effects), wait_for_expiry: true,
        },
        receipt: receipt.clone(),
    });
    let messages = [ProviderMessage { role: "user".into(), content: "perform the counted effect".into() }];
    let result = run_routed_provider_in_runner(
        RoutedProviderRun {
            provider: RoutedProvider::Fabushi,
            fabushi_model: Some(MODEL.into()),
            codex_model: None,
            codex_request_binding: None,
            fabushi_request_binding: None,
            data_dir: &root,
            messages: &messages,
            typed_replay: None,
            bridge,
            cursor_auth: None,
            request_context: RunnerRequestContextSnapshot {
                context: HostRequestContext {
                    os_version: "custody-contract".into(), shell: None, time_zone: Some("UTC".into()),
                    transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(),
                    user_full_name: None,
                },
                rules: None,
            },
            cancellation: RoutedProviderCancellation::default(),
            checkpoint_store: store.clone(),
            retry_sink: Some(Arc::new(move |_| { retry_sink.fetch_add(1, Ordering::AcqRel); })),
            retry_report_sink: Some(Arc::new(move |report| {
                report_sink.lock().expect("wire reports").push(report.clone());
            })),
            stream_attempt_runtime: Some(Arc::clone(&runtime)),
            usage_sink: None,
            cloud_agents_enabled: false,
            multitask_enabled: false,
            is_computer_use_subagent: false,
            max_steps: 8,
            tool_step_reminder: None,
        },
        &mut |_, _| {},
    );
    assert_custody_refusal(result, &[], &reports.lock().expect("wire reports"));
    assert_eq!(retry_count.load(Ordering::Acquire), 0);
    assert_eq!(effects.load(Ordering::Acquire), 1);
    assert_eq!(fs::read_to_string(receipt).expect("durable effect receipts").lines().count(), 1);
    assert!(!store.directory().exists(), "expired checkpoint must not be published");
    assert_eq!(runtime.effect_custody().pending_effects, 0);
    assert_eq!(runtime.effect_custody().started_revision, 1);
    assert_eq!(runtime.effect_custody().checkpoint_revision, 0);
    fs::write(root.join("child-result.json"), json!({
        "pid":std::process::id(),
        "token":std::env::var(WIRE_TOKEN_ENV).expect("private child receipt token"),
        "effects":1, "retries":0
    }).to_string()).expect("child completion receipt");
}

fn read_wire_request(stream: &mut TcpStream) -> Result<Value, String> {
    stream.set_nonblocking(false).map_err(|error| error.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).map_err(|error| error.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(2))).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 { return Err("closed request before headers".into()); }
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(index) = bytes.windows(4).position(|value| value == b"\r\n\r\n") {
            break index + 4;
        }
        if bytes.len() > 64 * 1024 { return Err("oversized request headers".into()); }
    };
    let headers = std::str::from_utf8(&bytes[..header_end]).map_err(|error| error.to_string())?;
    let length = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse::<usize>().ok()).flatten()
    }).ok_or("missing request length")?;
    if length > 1024 * 1024 { return Err("oversized request body".into()); }
    while bytes.len() < header_end + length {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 { return Err("closed request before body".into()); }
        bytes.extend_from_slice(&chunk[..count]);
    }
    serde_json::from_slice(&bytes[header_end..header_end + length]).map_err(|error| error.to_string())
}

struct WireServer {
    origin: String,
    requests: Arc<Mutex<Vec<Value>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<Result<(), String>>>,
}

impl WireServer {
    fn new() -> Self {
        Self::with_response(|_| {
            // Every replay without the missing result asks for the same
            // non-idempotent effect; the fixed owner sends only one request.
            Ok(format!("data: {}\n\n", json!({
                "type":"response.completed",
                "response":{
                    "id":"wire-response",
                    "output":[{"type":"function_call","name":"effect","call_id":"wire-effect","arguments":"{}"}]
                }
            })))
        })
    }

    fn with_response(
        response: impl Fn(&Value) -> Result<String, String> + Send + 'static,
    ) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("loopback Responses");
        listener.set_nonblocking(true).expect("bounded accept");
        let origin = format!("http://{}", listener.local_addr().expect("loopback address"));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(25);
            while !worker_stop.load(Ordering::Acquire) {
                if Instant::now() >= deadline { return Err("Responses fixture deadline".into()); }
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(error) => return Err(error.to_string()),
                };
                let request = read_wire_request(&mut stream)?;
                captured.lock().map_err(|_| "request capture poisoned")?.push(request.clone());
                let body = response(&request)?;
                write!(stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len(),
                ).map_err(|error| error.to_string())?;
            }
            Ok(())
        });
        Self { origin, requests, stop, worker: Some(worker) }
    }

    fn finish(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            wait_until("owned Responses server shutdown", || worker.is_finished());
            worker.join().expect("Responses worker panic").expect("Responses worker result");
        }
    }
}

impl Drop for WireServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let deadline = Instant::now() + BOUND;
            while !worker.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(2));
            }
            if worker.is_finished() {
                let _ = worker.join();
            }
        }
    }
}

struct OwnedWireChild(std::process::Child);

impl OwnedWireChild {
    fn wait(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(status) = self.0.try_wait().expect("poll exact owned child") {
                return status;
            }
            assert!(Instant::now() < deadline, "owned custody child exceeded its deadline");
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for OwnedWireChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_some() { return; }
        let _ = self.0.kill();
        let deadline = Instant::now() + BOUND;
        while Instant::now() < deadline {
            if self.0.try_wait().ok().flatten().is_some() { return; }
            thread::sleep(Duration::from_millis(5));
        }
        eprintln!("owned custody child could not be reaped within its cleanup deadline");
    }
}

#[test]
fn production_fabushi_executor_refuses_replay_after_tool_expiry() {
    if let Some(root) = std::env::var_os(WIRE_ROOT_ENV) {
        run_wire_child(PathBuf::from(root));
        return;
    }
    let root = TempRoot::new();
    let mut server = WireServer::new();
    let receipt = run_wire_process(&root, &mut server, WIRE_TEST, WIRE_ROOT_ENV);
    assert_eq!(receipt["effects"], 1);
    assert_eq!(receipt["retries"], 0);
    let requests = server.requests.lock().expect("captured Responses");
    assert_eq!(requests.len(), 1, "production executor must not dispatch another attempt");
    assert_eq!(requests[0]["model"], MODEL);
}

fn run_wire_process(
    root: &TempRoot,
    server: &mut WireServer,
    test_name: &str,
    root_env: &str,
) -> Value {
    let credential = root.0.join("credential.json");
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(&credential).expect("private fixture credential").write_all(json!({
        "version":1, "issuer":server.origin, "accessToken":"custody-fixture-token",
        "expiresAtMs":4_102_444_800_000_u64,
        "sessionId":"custody-session", "deviceId":"custody-device", "userId":"custody-account"
    }).to_string().as_bytes()).expect("fixture credential bytes");
    let token = uuid::Uuid::new_v4().to_string();
    let stdout_path = root.0.join("child-stdout.txt");
    let stderr_path = root.0.join("child-stderr.txt");
    let mut child = OwnedWireChild(std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(root_env, &root.0)
        .env("SAND_DATA_ROOT", &root.0)
        .env(WIRE_TOKEN_ENV, &token)
        .env("FABUSHI_RESPONSES_URL", format!("{}/v1/responses", server.origin))
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &credential)
        .env("FABUSHI_RESPONSES_MODEL", "default-must-not-replace-frozen-model")
        .env_remove("FABUSHI_PRODUCT_MODE")
        .env_remove("SAND_PACKAGED")
        .env_remove("SAND_AGENT_MODEL")
        .env_remove("SAND_AGENT_MOCK_RESPONSE")
        .env("NO_PROXY", "*").env("no_proxy", "*")
        .stdout(fs::File::create(&stdout_path).expect("child stdout"))
        .stderr(fs::File::create(&stderr_path).expect("child stderr"))
        .spawn().expect("spawn exact custody child"));
    let pid = child.0.id();
    let status = child.wait();
    server.finish();
    let stdout = fs::read_to_string(stdout_path).unwrap_or_default();
    let stderr = fs::read_to_string(stderr_path).unwrap_or_default();
    assert!(status.success(), "production custody child failed: {status}\n{stdout}\n{stderr}");
    let receipt: Value = serde_json::from_slice(&fs::read(root.0.join("child-result.json"))
        .expect("owned child completion receipt")).expect("completion JSON");
    assert_eq!(receipt["pid"].as_u64(), Some(u64::from(pid)));
    assert_eq!(receipt["token"], token);
    receipt
}

const GENERATED_WIRE_TEST: &str = "production_generated_runner_preserves_terminal_tool_custody";
const GENERATED_WIRE_ROOT_ENV: &str = "FABUSHI_GENERATED_CUSTODY_WIRE_ROOT";

struct GeneratedRelayBridge {
    relay: Arc<CoordinatorToolRelay>,
    cancel_after_local_return: Option<RoutedProviderCancellation>,
}

impl RoutedToolBridge for GeneratedRelayBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            origin: None,
            name: "b".into(), provider_identifier: "custody-contract".into(),
            tool_name: "b".into(), description: None,
            input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(&self, tool: &RoutedToolDefinition, args: Value, call_id: &str) -> Result<Value, ProviderSessionError> {
        let result = self.relay.request_tool_execution(json!({
            "providerIdentifier":tool.provider_identifier, "name":tool.name,
            "toolName":tool.tool_name, "args":args, "toolCallId":call_id,
        })).map_err(coordinator_tool_execution_error);
        if let Some(cancellation) = self.cancel_after_local_return.as_ref() {
            // The relay has observed the counted effect's entry, and its
            // external worker is still held when the actual guard receives this.
            cancellation.cancel("cancel generated turn after downstream outcome was lost");
        }
        result
    }
}

struct GeneratedWireRunner {
    runner: SandAgentRunner,
    cancellation: RoutedProviderCancellation,
    store: Arc<ProductionRoutedProviderCheckpointStore>,
    retries: Arc<AtomicUsize>,
}

impl GeneratedWireRunner {
    fn new(root: &Path, stream_id: &str, relay: Arc<CoordinatorToolRelay>, cancel: bool) -> Self {
        Self::with_external_machine(root, stream_id, relay, cancel, None)
    }

    fn with_external_machine(
        root: &Path,
        stream_id: &str,
        relay: Arc<CoordinatorToolRelay>,
        cancel: bool,
        external: Option<Arc<dyn ExternalMachineExecutor>>,
    ) -> Self {
        use mahayana_host_runtime::host_request_context::HostRequestContext;

        let cancellation = RoutedProviderCancellation::default();
        let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(
            root, "generated-custody-agent", stream_id,
        ));
        let retries = Arc::new(AtomicUsize::new(0));
        let retry_sink = Arc::clone(&retries);
        let composition = TurnAgentComposition::new(
            RoutedProvider::Fabushi,
            Arc::new(GeneratedRelayBridge {
                relay,
                cancel_after_local_return: cancel.then(|| cancellation.clone()),
            }),
            RunnerRequestContextSnapshot {
                context: HostRequestContext {
                    os_version: "generated-custody-contract".into(), shell: None,
                    time_zone: Some("UTC".into()),
                    transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(),
                    user_full_name: None,
                },
                rules: None,
            },
            cancellation.clone(),
            store.clone(),
        )
        .with_fabushi_model(MODEL)
        .with_retry_sink(Arc::new(move |_| {
            retry_sink.fetch_add(1, Ordering::AcqRel);
        }));
        let composition = match external {
            Some(executor) => composition.with_external_machine_executor(executor),
            None => composition,
        };
        // The same production bridge used by HostRunnerComposition binds the
        // real owner/facade. No attempt executor, runtime or custody is injected.
        let runner = create_production_runner(
            composition,
            ProductionTurnAgentBuildBindings::new(
                ProductionTurnAgentStaticConfig {
                    model_id: MODEL.into(), agent_token_limit: SAND_AGENT_TOKEN_LIMIT,
                    conversation_id: "generated-custody-agent".into(),
                    is_box_scoped_subagent: false, is_subagent_runner: false,
                    is_shared_room_runner: false, sand_send_message_delivery_owed: false,
                    transcripts_folder_available: true,
                },
                Arc::new(|| None),
                Arc::new(|_, _, _| panic!("bounded custody turn must not summarize")),
            ),
            ProductionTurnAgentLifecycleBindings::new("generated-custody-agent", stream_id),
            None,
            Arc::new(AtomicBool::new(false)),
            Arc::new(Mutex::new(SubagentRuntime::default())),
            None, None, None, None,
        );
        assert!(runner.last_tool_effect_custody().is_none());
        Self { runner, cancellation, store, retries }
    }

    fn run(&mut self, root: &Path, stream_id: &str, prompt: &str) -> Result<String, ProviderSessionError> {
        self.runner.run_routed_provider_with_options(
            root,
            &[ProviderMessage { role: "user".into(), content: prompt.into() }],
            TurnRunOptions {
                inference_request_id: Some(stream_id.into()),
                ..TurnRunOptions::default()
            },
            &mut |_, _| {},
        )
    }

    fn snapshot(&self, stream_id: &str, generation: u64) -> ToolEffectCustodySnapshot {
        let finished = self.runner.last_finished().expect("real shell terminal");
        assert_eq!(finished.owner.request_id, stream_id);
        assert_eq!(finished.owner.generation, generation);
        match self.runner.last_tool_effect_custody().expect("real generated observation") {
            ToolEffectCustodyObservation::Known { run_generation, snapshot } => {
                assert_eq!(*run_generation, generation, "custody belongs to the actual shell run");
                snapshot.clone()
            }
            observation => panic!("generated execution lost its typed observation: {observation:?}"),
        }
    }
}

fn generated_wire_prompt(request: &Value) -> Result<&str, String> {
    request["input"].as_array()
        .and_then(|input| input.iter().rev().find(|item| item["role"] == "user"))
        .and_then(|item| item["content"].as_str())
        .ok_or_else(|| "generated fixture request lost its user prompt".into())
}

fn generated_wire_response(request: &Value) -> Result<String, String> {
    let prompt = generated_wire_prompt(request)?;
    let events = match prompt {
        "generated-unknown" | "generated-cancel" => vec![json!({
            "type":"response.completed",
            "response":{
                "id":format!("{prompt}-response"),
                "output":[{"type":"function_call","name":"b","call_id":format!("{prompt}-call"),"arguments":"{}"}]
            }
        })],
        "generated-clear" => vec![
            json!({"type":"response.output_text.delta","delta":"clear completion"}),
            json!({"type":"response.completed","response":{"id":"clear-response","output":[]}}),
        ],
        "generated-failure" => vec![json!({
            "type":"response.failed",
            "response":{"error":{"message":"known fixture provider failure"}},
        })],
        _ => return Err(format!("unexpected generated fixture prompt: {prompt}")),
    };
    Ok(events.iter().map(|event| format!("data: {event}\n\n")).collect())
}

fn run_generated_wire_child(root: PathBuf) {
    let mut effects = 0;
    let mut total_retries = 0;
    let mut frozen_reports = Vec::new();
    for (stream_id, reply, cancel) in [
        ("generated-unknown", DownstreamReply::Timeout, false),
        ("generated-cancel", DownstreamReply::Value(json!({
            "result":{"case":"error","value":{"error":"downstream result was lost"}},
        })), true),
    ] {
        let receipt_path = root.join(format!("{stream_id}-effects.txt"));
        let mut downstream = GatedDownstream::new(reply, receipt_path.clone());
        let mut fixture = GeneratedWireRunner::new(&root, stream_id, Arc::clone(&downstream.relay), cancel);
        assert!(fixture.run(&root, stream_id, stream_id).is_err(),
            "unknown effect cannot be reported as provider success");
        assert_eq!(fixture.cancellation.is_cancelled(), cancel);
        let snapshot = fixture.snapshot(stream_id, 1);
        assert_eq!(snapshot, ToolEffectCustodySnapshot {
            started_revision: 1, checkpoint_revision: 0, pending_effects: 0,
            unknown_result: Some(ToolEffectUnknownResult::DownstreamOutcomeUnknown),
        });
        let observation = fixture.runner.last_tool_effect_custody().unwrap().clone();
        let mut report = ToolEffectCustodyReport::new(stream_id);
        report.record(fixture.runner.last_tool_effect_custody());
        let frozen = report.clone().for_terminal(stream_id);
        assert!(frozen.requires_retention());
        assert!(!fixture.store.directory().exists(), "unknown result must not reach checkpoint persistence");
        assert_eq!(fixture.retries.load(Ordering::Acquire), 0);
        assert_eq!(downstream.effects.load(Ordering::Acquire), 1);
        assert_eq!(fs::read_to_string(&receipt_path).expect("real external effect"), format!("{stream_id}-call\n"));
        let published = downstream.events.try_iter().collect::<Vec<_>>();
        assert_eq!(published.len(), 1, "the production bridge must publish one effect only");
        assert_eq!(downstream.relay.pending_count(), 0);
        {
            let workers = downstream.workers.lock().expect("exact owned external worker");
            assert_eq!(workers.len(), 1);
            assert!(!workers[0].is_finished(), "external execution must outlive generated Runner return");
        }
        let late = downstream.release();
        assert_eq!(late.len(), 1);
        assert!(matches!(&late[0], Err(CoordinatorToolRelayError::Protocol(_))),
            "the late external result must not settle an ended relay request");
        assert_eq!(fixture.runner.last_tool_effect_custody(), Some(&observation));
        assert_eq!(report, frozen, "late completion cannot rewrite the frozen value");

        assert!(matches!(fixture.run(&root, stream_id, " \t\n"),
            Err(ProviderSessionError::Configuration(_))));
        assert_eq!(fixture.runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::NotStarted),
            "a refused new invocation cannot inherit the preceding genuine Known value");
        report.record(fixture.runner.last_tool_effect_custody());
        assert_eq!(frozen.invocations(), std::slice::from_ref(&observation));
        assert!(report.requires_retention());

        if !cancel {
            // Explicit facade reuse proves invocation isolation. Host nudges
            // currently gate on success; this does not claim unknown-to-nudge reachability.
            for (prompt, generation) in [("generated-clear", 2), ("generated-failure", 3)] {
                let result = fixture.run(&root, stream_id, prompt);
                if generation == 2 {
                    assert_eq!(result.expect("real clear provider success"), "clear completion");
                } else {
                    assert!(matches!(result, Err(ProviderSessionError::Protocol(_))));
                }
                let clear = fixture.snapshot(stream_id, generation);
                assert_eq!(clear, ToolEffectCustodySnapshot {
                    started_revision: 0, checkpoint_revision: 0, pending_effects: 0,
                    unknown_result: None,
                });
                assert!(!fixture.runner.last_tool_effect_custody().unwrap().requires_retention());
                report.record(fixture.runner.last_tool_effect_custody());
                assert!(report.requires_retention(), "later clear invocations cannot erase earlier uncertainty");
                assert_eq!(frozen.invocations(), std::slice::from_ref(&observation));
            }
        }
        assert_eq!(report.invocations().len(), if cancel { 2 } else { 4 });
        assert_eq!(downstream.effects.load(Ordering::Acquire), 1);
        assert_eq!(fs::read_to_string(&receipt_path).expect("unchanged effect receipt"), format!("{stream_id}-call\n"));
        assert!(matches!(downstream.events.try_recv(), Err(mpsc::TryRecvError::Empty)));
        assert!(!fixture.store.directory().exists());
        assert_eq!(fixture.retries.load(Ordering::Acquire), 0);
        effects += downstream.effects.load(Ordering::Acquire);
        total_retries += fixture.retries.load(Ordering::Acquire);
        frozen_reports.push(report.for_terminal(stream_id));
    }
    fs::write(root.join("child-result.json"), json!({
        "pid":std::process::id(),
        "token":std::env::var(WIRE_TOKEN_ENV).expect("private child receipt token"),
        "effects":effects, "retries":total_retries, "reports":frozen_reports,
    }).to_string()).expect("generated child completion receipt");
}

#[test]
fn production_generated_runner_preserves_terminal_tool_custody() {
    if let Some(root) = std::env::var_os(GENERATED_WIRE_ROOT_ENV) {
        run_generated_wire_child(PathBuf::from(root));
        return;
    }
    let root = TempRoot::new();
    let mut server = WireServer::with_response(generated_wire_response);
    let receipt = run_wire_process(&root, &mut server, GENERATED_WIRE_TEST, GENERATED_WIRE_ROOT_ENV);
    assert_eq!(receipt["effects"], 2);
    assert_eq!(receipt["retries"], 0);
    let reports: Vec<ToolEffectCustodyReport> = serde_json::from_value(receipt["reports"].clone())
        .expect("actual child reports survive their serialized observation boundary");
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].invocations().len(), 4);
    assert_eq!(reports[1].invocations().len(), 2);
    assert!(reports.iter().all(ToolEffectCustodyReport::requires_retention));
    let requests = server.requests.lock().expect("genuine generated Responses requests");
    let prompts = requests.iter().map(|request| generated_wire_prompt(request).unwrap()).collect::<Vec<_>>();
    assert_eq!(prompts, ["generated-unknown", "generated-clear", "generated-failure", "generated-cancel"],
        "no retries, refused invocations or late worker results may dispatch another provider request");
    assert!(requests.iter().all(|request| request["model"] == MODEL));
}


const LXO_UNKNOWN_TEST: &str = "production_generated_runner_retains_unknown_external_shell_effects";
const LXO_KNOWN_TEST: &str = "production_generated_runner_preserves_known_external_shell_outcomes";
const LXO_ADAPTER_TEST: &str = "production_external_shell_adapter_types_uncertain_results_and_errors";
const LXO_ROOT_ENV: &str = "FABUSHI_LOCAL_EXEC_CUSTODY_WIRE_ROOT";
const LXO_AGENT: &str = "generated-custody-agent";
const LXO_OUTPUT: &str = "lxo-partial-output\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LocalExecReply {
    Close,
    CancelClose,
    Silence,
    MalformedClient,
    MalformedControl,
    OpaqueThrow,
    TaggedAfterStart,
    FileError,
    StartedJournalFailure,
    TerminalJournalFailure,
    Exit,
    AbortedExit,
    CancelThenExit,
    PreEntryTag,
    ShellRejected,
    ShellPermissionDenied,
    LocalPermission,
}

impl LocalExecReply {
    fn name(self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::CancelClose => "cancel-close",
            Self::Silence => "silence",
            Self::MalformedClient => "malformed-client",
            Self::MalformedControl => "malformed-control",
            Self::OpaqueThrow => "opaque-throw",
            Self::TaggedAfterStart => "tagged-after-start",
            Self::FileError => "file-error",
            Self::StartedJournalFailure => "started-journal-failure",
            Self::TerminalJournalFailure => "terminal-journal-failure",
            Self::Exit => "exit",
            Self::AbortedExit => "aborted-exit",
            Self::CancelThenExit => "cancel-then-exit",
            Self::PreEntryTag => "pre-entry-tag",
            Self::ShellRejected => "shell-rejected",
            Self::ShellPermissionDenied => "shell-permission-denied",
            Self::LocalPermission => "local-permission",
        }
    }

    fn parse(name: &str) -> Self {
        [
            Self::Close, Self::CancelClose, Self::Silence, Self::MalformedClient,
            Self::MalformedControl, Self::OpaqueThrow, Self::TaggedAfterStart,
            Self::FileError, Self::StartedJournalFailure, Self::TerminalJournalFailure,
            Self::Exit, Self::AbortedExit, Self::CancelThenExit, Self::PreEntryTag,
            Self::ShellRejected, Self::ShellPermissionDenied, Self::LocalPermission,
        ].into_iter().find(|case| case.name() == name).expect("declared LocalExec case")
    }

    fn is_unknown(self) -> bool {
        !matches!(self, Self::Exit | Self::AbortedExit | Self::CancelThenExit
            | Self::PreEntryTag | Self::ShellRejected | Self::ShellPermissionDenied
            | Self::LocalPermission)
    }

    fn enters_effect(self) -> bool {
        !matches!(self, Self::PreEntryTag | Self::ShellRejected
            | Self::ShellPermissionDenied | Self::LocalPermission)
    }

    fn has_exit(self) -> bool {
        matches!(self, Self::Exit | Self::AbortedExit | Self::CancelThenExit
            | Self::TerminalJournalFailure)
    }

    fn breaks_journal(self) -> bool {
        matches!(self, Self::StartedJournalFailure | Self::TerminalJournalFailure)
    }

    fn status(self) -> &'static str {
        match self {
            Self::AbortedExit => "cancelled",
            Self::Exit | Self::CancelThenExit | Self::TerminalJournalFailure => "completed",
            Self::PreEntryTag | Self::ShellRejected | Self::ShellPermissionDenied => "failed",
            Self::LocalPermission => "not_found",
            _ => "outcome_unknown",
        }
    }
}

/// Only the remote transport is controlled. The request must have passed the
/// real permission owner, shared journal admission and production bridge before
/// this fixture starts its independently owned, counted filesystem effect.
struct LocalExecDownstream {
    local_exec: Arc<HostLocalExecExtension>,
    permission: Arc<HostLocalToolPermissionExtension>,
    executor: Arc<ProductionExternalMachineExecutor>,
    _registration: LocalExecProviderRegistration,
    effects: Arc<AtomicUsize>,
    requests: Arc<Mutex<Vec<String>>>,
    cancel_frames: Arc<AtomicUsize>,
    gate: Arc<ReleaseGate>,
    workers: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
    stop: Arc<AtomicBool>,
    provider: Option<thread::JoinHandle<()>>,
    receipt: PathBuf,
}

impl LocalExecDownstream {
    fn new(root: &Path, reply: LocalExecReply) -> Self {
        assert_eq!(mahayana_host_runtime::host_paths::get_sand_root_dir(), root);
        let settings = Arc::new(SettingsService::new(root.join("settings.json")));
        settings.set_local_tool_permission(if reply == LocalExecReply::LocalPermission {
            SandLocalToolPermission::Never
        } else {
            SandLocalToolPermission::Always
        }).expect("persist actual local permission");
        let permission = Arc::new(start_local_tool_permission_extension(settings));
        permission.controller().begin_turn(LXO_AGENT);
        let epoch = permission.controller().direction_epoch(LXO_AGENT);
        let telemetry = HostTelemetryService::open_with_identity_tags(
            root.join("local-exec-telemetry.jsonl"), Default::default(),
        ).expect("real telemetry owner");
        let local_exec = Arc::new(start_local_exec_extension(telemetry.logs.clone()));
        let (send, receive) = mpsc::channel();
        let registration = local_exec.register_provider(send);
        let welcome = receive.recv_timeout(BOUND).expect("production provider welcome");
        let provider_id = welcome["providerId"].as_str().expect("production provider identity").to_string();
        local_exec.submit_responses(json!({
            "providerId": provider_id,
            "frames":[{
                "kind":"hello", "localRoot":root, "terminalsFolder":root.join("terminals"),
                "computerId":"lxo-controlled-computer", "label":"LocalExec contract",
            }],
        }));
        let executor = Arc::new(ProductionExternalMachineExecutor::for_turn(
            Arc::clone(&local_exec), Arc::clone(&permission), LXO_AGENT, epoch,
        ));
        let effects = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::<String>::new()));
        let cancel_frames = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(ReleaseGate::default());
        let workers = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let receipt = root.join("external-effects.txt");
        let provider_owner = Arc::clone(&local_exec);
        let provider_executor = Arc::clone(&executor);
        let provider_effects = Arc::clone(&effects);
        let provider_requests = Arc::clone(&requests);
        let provider_cancels = Arc::clone(&cancel_frames);
        let provider_gate = Arc::clone(&gate);
        let provider_workers = Arc::clone(&workers);
        let provider_stop = Arc::clone(&stop);
        let provider_receipt = receipt.clone();
        let journal = root.join("host-local-exec-operations.json");
        let provider = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(30);
            while !provider_stop.load(Ordering::Acquire) {
                assert!(Instant::now() < deadline, "owned LocalExec provider exceeded its bound");
                let frame = match receive.recv_timeout(Duration::from_millis(20)) {
                    Ok(frame) => frame,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                };
                if frame["kind"] == "cancel" {
                    provider_cancels.fetch_add(1, Ordering::AcqRel);
                    continue;
                }
                assert_eq!(frame["kind"], "exec", "only actual shell requests may enter");
                let request_id = frame["requestId"].as_str().expect("bridge request ID").to_string();
                assert_ne!(request_id, provider_id, "the production bridge must not reuse provider ID");
                let ordinal = {
                    let mut captured = provider_requests.lock().expect("captured requests");
                    assert!(!captured.contains(&request_id), "distinct invocation transport IDs");
                    captured.push(request_id.clone());
                    captured.len()
                };
                let call_id = frame["serverMessage"]["shellStreamArgs"]["toolCallId"]
                    .as_str().expect("real generated shell operation ID").to_string();
                assert_eq!(frame["serverMessage"]["shellStreamArgs"]["command"], "count-owned-effect");
                let send = |kind: &str, message: Value| {
                    provider_owner.submit_responses(json!({
                        "providerId":provider_id,
                        "frames":[{"kind":kind,"requestId":request_id,"message":message}],
                    }));
                };
                if !reply.enters_effect() {
                    match reply {
                        LocalExecReply::PreEntryTag => send("control", json!({
                            "throw":{"error":"lxo pre-entry refusal","errorCode":"LOCAL_EXEC_PRE_ENTRY_REFUSED"},
                        })),
                        LocalExecReply::ShellRejected => send("client", json!({
                            "shellStream":{"rejected":{"reason":"lxo rejected before execution"}},
                        })),
                        LocalExecReply::ShellPermissionDenied => send("client", json!({
                            "shellStream":{"permissionDenied":{"error":"lxo provider permission denied"}},
                        })),
                        LocalExecReply::LocalPermission => panic!("local permission refusal reached transport"),
                        _ => unreachable!(),
                    }
                    continue;
                }

                let receipt = provider_receipt.clone();
                let effect_call = call_id.clone();
                let counted = Arc::clone(&provider_effects);
                let effect_gate = Arc::clone(&provider_gate);
                let (entered_tx, entered_rx) = mpsc::channel();
                let effect = thread::spawn(move || {
                    let mut file = fs::OpenOptions::new().create(true).append(true).open(receipt)
                        .expect("non-idempotent external receipt");
                    writeln!(file, "{effect_call}").expect("append exactly one external effect");
                    file.sync_all().expect("persist external entry");
                    counted.fetch_add(1, Ordering::AcqRel);
                    entered_tx.send(()).expect("effect entry acknowledgement");
                    effect_gate.wait_for(Duration::from_secs(30));
                });
                provider_workers.lock().expect("owned effects").push(effect);
                entered_rx.recv_timeout(BOUND).expect("effect really entered before local result");
                // A broken implementation may admit the second generated call.
                // Let it finish so the count assertion fails without a timeout.
                let delivered_reply = if ordinal == 1 { reply } else { LocalExecReply::Exit };
                if delivered_reply == LocalExecReply::StartedJournalFailure {
                    fs::remove_file(&journal).expect("accepted journal was published before send");
                    fs::create_dir(&journal).expect("portable journal publication failure");
                }
                send("client", json!({"shellStream":{"stdout":{"data":LXO_OUTPUT}}}));
                if delivered_reply == LocalExecReply::StartedJournalFailure {
                    continue;
                }
                wait_until("manager observed real shell activity", || {
                    provider_executor.operation_status(&call_id).expect("owner query")["status"] == "running"
                });
                if matches!(delivered_reply, LocalExecReply::CancelClose
                    | LocalExecReply::CancelThenExit | LocalExecReply::AbortedExit)
                {
                    assert_eq!(provider_executor.cancel_operation(&call_id)
                        .expect("actual scoped cancellation")["cancelRequested"], true);
                    let cancel = receive.recv_timeout(BOUND).expect("real cancellation frame");
                    assert_eq!(cancel["kind"], "cancel");
                    assert_eq!(cancel["requestId"], request_id);
                    provider_cancels.fetch_add(1, Ordering::AcqRel);
                }
                if delivered_reply == LocalExecReply::TerminalJournalFailure {
                    fs::remove_file(&journal).expect("running journal is durable before failure");
                    fs::create_dir(&journal).expect("portable terminal publication failure");
                }
                if delivered_reply.has_exit() {
                    provider_gate.release();
                    wait_until("counted effect completes before acknowledged Exit", || {
                        provider_workers.lock().expect("owned effects").iter().all(|worker| worker.is_finished())
                    });
                    send("client", json!({"shellStream":{"exit":{
                        "code":17, "cwd":"/lxo", "aborted":delivered_reply == LocalExecReply::AbortedExit,
                    }}}));
                    continue;
                }
                match delivered_reply {
                    LocalExecReply::Close | LocalExecReply::CancelClose => {
                        send("control", json!({"streamClose":{}}));
                    }
                    LocalExecReply::Silence => {}
                    LocalExecReply::MalformedClient => send("client", json!(["malformed", "client"])),
                    LocalExecReply::MalformedControl => send("control", json!({
                        "throw":{"error":"conflicting control"}, "streamClose":{},
                    })),
                    LocalExecReply::OpaqueThrow => send("control", json!({
                        "throw":{"error":"lxo entered failure"},
                    })),
                    LocalExecReply::TaggedAfterStart => send("control", json!({
                        "throw":{"error":"contradictory refusal","errorCode":"LOCAL_EXEC_PRE_ENTRY_REFUSED"},
                    })),
                    LocalExecReply::FileError => {
                        provider_owner.submit_responses(json!({
                            "providerId":provider_id,
                            "frames":[{"kind":"file-error","requestId":request_id,"error":"unexpected entered file error"}],
                        }));
                    }
                    _ => unreachable!(),
                }
            }
        });
        Self {
            local_exec, permission, executor, _registration:registration, effects, requests,
            cancel_frames, gate, workers, stop, provider:Some(provider), receipt,
        }
    }

    fn same_owner_status(&self, operation_id: &str) -> Value {
        let another_turn_facade = ProductionExternalMachineExecutor::new(
            Arc::clone(&self.local_exec), Arc::clone(&self.permission), LXO_AGENT,
        );
        let status = another_turn_facade.operation_status(operation_id).expect("shared owner status");
        assert_eq!(status, self.executor.operation_status(operation_id).expect("original status"));
        let foreign_agent = ProductionExternalMachineExecutor::new(
            Arc::clone(&self.local_exec), Arc::clone(&self.permission), "another-agent",
        );
        assert_eq!(foreign_agent.operation_status(operation_id).expect("foreign status")["status"], "not_found");
        status
    }

    fn assert_effect(&self, reply: LocalExecReply, operation_id: &str) -> bool {
        let expected = if reply.enters_effect() { 1 } else { 0 };
        let mut alive_at_return = false;
        assert_eq!(self.effects.load(Ordering::Acquire), expected);
        assert_eq!(self.requests.lock().expect("received requests").len(),
            if reply == LocalExecReply::LocalPermission { 0 } else { 1 });
        if expected == 1 {
            assert_eq!(fs::read_to_string(&self.receipt).expect("actual external effect"),
                format!("{operation_id}\n"));
            let workers = self.workers.lock().expect("owned external workers");
            assert_eq!(workers.len(), 1);
            alive_at_return = !workers[0].is_finished();
            assert_eq!(alive_at_return, !reply.has_exit(),
                "no-Exit unknown must outlive Host return; Exit must follow actual fixture completion");
        } else {
            assert!(!self.receipt.exists());
        }
        if matches!(reply, LocalExecReply::CancelClose | LocalExecReply::CancelThenExit | LocalExecReply::AbortedExit) {
            assert_eq!(self.cancel_frames.load(Ordering::Acquire), 1);
        }
        alive_at_return
    }

    fn finish(&mut self) {
        self.gate.release();
        self.stop.store(true, Ordering::Release);
        if let Some(provider) = self.provider.take() {
            wait_until("owned LocalExec provider stops", || provider.is_finished());
            provider.join().expect("LocalExec provider result");
        }
        for worker in std::mem::take(&mut *self.workers.lock().expect("owned effects")) {
            wait_until("owned external effect stops", || worker.is_finished());
            worker.join().expect("external effect result");
        }
    }
}

impl Drop for LocalExecDownstream {
    fn drop(&mut self) {
        self.gate.release();
        self.stop.store(true, Ordering::Release);
        if let Some(provider) = self.provider.take() {
            let deadline = Instant::now() + BOUND;
            while !provider.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(2));
            }
            if provider.is_finished() { let _ = provider.join(); }
        }
        if let Ok(mut workers) = self.workers.lock() {
            let deadline = Instant::now() + BOUND;
            for worker in workers.drain(..) {
                while !worker.is_finished() && Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(2));
                }
                if worker.is_finished() { let _ = worker.join(); }
            }
        }
    }
}

fn local_exec_wire_response(request: &Value) -> Result<String, String> {
    let prompt = generated_wire_prompt(request)?;
    let case = prompt.strip_prefix("lxo-").ok_or("missing LocalExec case prefix")?;
    let reply = LocalExecReply::parse(case);
    let has_output = request["input"].as_array().is_some_and(|input| {
        input.iter().any(|item| item["type"] == "function_call_output")
    });
    let events = if has_output {
        vec![
            json!({"type":"response.output_text.delta","delta":"known external completion"}),
            json!({"type":"response.completed","response":{"id":"lxo-done","output":[]}}),
        ]
    } else {
        let mut calls = vec![json!({
            "type":"function_call", "name":"ExternalShell", "call_id":format!("{prompt}-first"),
            "arguments":json!({"command":"count-owned-effect","workingDirectory":"/lxo"}).to_string(),
        })];
        if reply.is_unknown() {
            calls.push(json!({
                "type":"function_call", "name":"ExternalShell", "call_id":format!("{prompt}-second"),
                "arguments":json!({"command":"count-owned-effect","workingDirectory":"/lxo"}).to_string(),
            }));
        }
        vec![json!({"type":"response.completed","response":{"id":"lxo-response","output":calls}})]
    };
    Ok(events.iter().map(|event| format!("data: {event}\n\n")).collect())
}

fn local_exec_child_case(root: &Path) -> LocalExecReply {
    LocalExecReply::parse(&fs::read_to_string(root.join("local-exec-case.txt")).expect("exact private case"))
}

fn assert_local_exec_record(status: &Value, reply: LocalExecReply, operation_id: &str) {
    assert_eq!(status["status"], reply.status());
    assert_eq!(status["operationId"], operation_id);
    if reply == LocalExecReply::LocalPermission { return; }
    assert_eq!(status["requestId"], operation_id);
    assert_eq!(status["resultRef"], format!("fabushi-local-exec://operation/{operation_id}"));
    assert_eq!(status["aborted"], reply == LocalExecReply::AbortedExit);
    if reply.enters_effect() {
        assert_eq!(status["stdout"], LXO_OUTPUT, "partial observed output must survive the error path");
        assert_eq!(status["stdoutTruncated"], false);
        assert_eq!(status["stderrTruncated"], false);
    }
    if reply.has_exit() {
        assert_eq!(status["exitCode"], 17);
    } else {
        assert!(status["exitCode"].is_null());
    }
}

fn reopen_local_exec_result(root: &Path, reply: LocalExecReply, operation_id: &str) {
    let telemetry = HostTelemetryService::open_with_identity_tags(
        root.join("local-exec-reopened-telemetry.jsonl"), Default::default(),
    ).expect("fresh telemetry owner");
    let local_exec = Arc::new(start_local_exec_extension(telemetry.logs.clone()));
    let permission = Arc::new(start_local_tool_permission_extension(Arc::new(
        SettingsService::new(root.join("settings.json")),
    )));
    let executor = ProductionExternalMachineExecutor::new(local_exec, permission, LXO_AGENT);
    if reply.breaks_journal() {
        let refused = executor.execute_shell(&ExternalMachineShellArgs {
            command:"must-not-replay".into(), working_directory:"/lxo".into(),
            tool_call_id:"fresh-owner-refusal".into(),
        }).expect_err("journal failure must remain fail-closed after restart");
        assert!(matches!(refused, ProviderSessionError::Tool(_)));
        assert!(refused.to_string().contains("journal"));
    } else {
        let recovered = executor.operation_status(operation_id).expect("fresh owner query");
        assert_local_exec_record(&recovered, reply, operation_id);
    }
}

fn run_local_exec_runner_child(root: PathBuf) {
    let reply = local_exec_child_case(&root);
    let prompt = format!("lxo-{}", reply.name());
    let operation_id = format!("{prompt}-first");
    let mut downstream = LocalExecDownstream::new(&root, reply);
    let relay = Arc::new(CoordinatorToolRelay::with_timeout(GatewayEventHub::default(), BOUND));
    let external: Arc<dyn ExternalMachineExecutor> = downstream.executor.clone();
    let mut fixture = GeneratedWireRunner::with_external_machine(
        &root, &prompt, relay, false, Some(external),
    );
    let result = fixture.run(&root, &prompt, &prompt);
    let snapshot = fixture.snapshot(&prompt, 1);
    let mut report = ToolEffectCustodyReport::new(&prompt);
    report.record(fixture.runner.last_tool_effect_custody());
    let frozen = report.for_terminal(&prompt);
    assert_eq!(snapshot.started_revision, 1, "the second generated tool must not run after uncertainty");
    assert_eq!(snapshot.pending_effects, 0, "the adapter really returned through the common guard");
    if reply.is_unknown() {
        assert!(result.is_err(), "unknown ExternalShell cannot finish the real Runner successfully");
        assert_eq!(snapshot.unknown_result, Some(ToolEffectUnknownResult::DownstreamOutcomeUnknown));
        assert_eq!(snapshot.checkpoint_revision, 0);
        assert!(frozen.requires_retention());
        assert!(!fixture.store.directory().exists(), "unknown tool result must never be checkpointed");
    } else {
        assert_eq!(snapshot.unknown_result, None);
        assert!(!frozen.requires_retention(), "positive known controls must not acquire false uncertainty");
        // Fabushi's existing Responses loop publishes known Tool errors as
        // isError outputs and permits its next model step. Preserve that contract.
        assert_eq!(result.expect("known outcome permits the next model step"), "known external completion");
        assert_eq!(snapshot.checkpoint_revision, 1);
        assert!(fixture.store.directory().exists());
    }
    assert_eq!(fixture.retries.load(Ordering::Acquire), 0);
    let external_alive_at_return = downstream.assert_effect(reply, &operation_id);
    let status = downstream.same_owner_status(&operation_id);
    assert_local_exec_record(&status, reply, &operation_id);
    let effects = downstream.effects.load(Ordering::Acquire);
    let requests = downstream.requests.lock().expect("captured transport IDs").clone();
    let observation = fixture.runner.last_tool_effect_custody().cloned();
    downstream.finish();
    assert_eq!(fixture.runner.last_tool_effect_custody(), observation.as_ref(),
        "late owned-worker completion cannot clear frozen custody");
    drop(fixture);
    drop(downstream);
    reopen_local_exec_result(&root, reply, &operation_id);
    fs::write(root.join("child-result.json"), json!({
        "pid":std::process::id(), "token":std::env::var(WIRE_TOKEN_ENV).expect("private child token"),
        "case":reply.name(), "effects":effects, "retries":0, "requests":requests,
        "snapshot":snapshot, "report":frozen, "status":status,
        "externalAliveAtReturn":external_alive_at_return,
    }).to_string()).expect("LocalExec generated Runner receipt");
}

fn run_local_exec_adapter_child(root: PathBuf) {
    let reply = local_exec_child_case(&root);
    let operation_id = format!("lxo-{}-first", reply.name());
    let mut downstream = LocalExecDownstream::new(&root, reply);
    let error = downstream.executor.execute_shell(&ExternalMachineShellArgs {
        command:"count-owned-effect".into(), working_directory:"/lxo".into(),
        tool_call_id:operation_id.clone(),
    }).expect_err("actual manager uncertainty must reach typed adapter error");
    let ProviderSessionError::ToolOutcomeUnknown(message) = error else {
        panic!("actual external adapter lost the typed uncertainty: {error}");
    };
    let detail: Value = serde_json::from_str(&message).expect("structured unknown result detail");
    assert_eq!(detail["operationId"], operation_id);
    assert_eq!(detail["requestId"], operation_id);
    assert_eq!(detail["resultRef"], format!("fabushi-local-exec://operation/{operation_id}"));
    assert_eq!(detail["outcomeUnknown"], true);
    assert_eq!(detail["stdout"], LXO_OUTPUT);
    assert_eq!(detail["stdoutTruncated"], false);
    assert!(detail["cause"].as_str().is_some_and(|cause| !cause.is_empty()));
    if reply == LocalExecReply::TerminalJournalFailure {
        assert_eq!(detail["status"], "completed", "observed Exit remains an execution fact");
        assert_eq!(detail["exitCode"], 17);
        assert!(detail["cause"].as_str().unwrap().contains("journal"));
    }
    let external_alive_at_return = downstream.assert_effect(reply, &operation_id);
    let status = downstream.same_owner_status(&operation_id);
    assert_local_exec_record(&status, reply, &operation_id);
    downstream.finish();
    drop(downstream);
    reopen_local_exec_result(&root, reply, &operation_id);
    fs::write(root.join("child-result.json"), json!({
        "pid":std::process::id(), "token":std::env::var(WIRE_TOKEN_ENV).expect("private child token"),
        "case":reply.name(), "effects":1, "detail":detail,
        "externalAliveAtReturn":external_alive_at_return,
    }).to_string()).expect("LocalExec adapter receipt");
}

fn run_local_exec_cases(test_name: &str, cases: &[LocalExecReply], adapter_only: bool) {
    for reply in cases {
        let root = TempRoot::new();
        fs::write(root.0.join("local-exec-case.txt"), reply.name()).expect("freeze exact child case");
        let mut server = WireServer::with_response(local_exec_wire_response);
        let receipt = run_wire_process(&root, &mut server, test_name, LXO_ROOT_ENV);
        assert_eq!(receipt["case"], reply.name());
        assert_eq!(receipt["effects"].as_u64(), Some(if reply.enters_effect() { 1 } else { 0 }));
        assert_eq!(receipt["externalAliveAtReturn"], reply.enters_effect() && !reply.has_exit());
        println!("LXO_CASE_RECEIPT {}", json!({"test":test_name,"receipt":receipt}));
        let requests = server.requests.lock().expect("actual Responses requests");
        assert_eq!(requests.len(), if adapter_only { 0 } else if reply.is_unknown() { 1 } else { 2 },
            "{}: no uncertain checkpoint, replay or extra provider dispatch", reply.name());
        if !adapter_only {
            assert_eq!(receipt["retries"], 0);
            let report: ToolEffectCustodyReport = serde_json::from_value(receipt["report"].clone())
                .expect("actual generated terminal report");
            assert_eq!(report.requires_retention(), reply.is_unknown());
            assert!(requests.iter().all(|request| request["model"] == MODEL));
            if !reply.is_unknown() {
                let output = requests[1]["input"].as_array().expect("accepted checkpoint replay")
                    .iter().find(|item| item["type"] == "function_call_output")
                    .expect("known result is paired with its actual call");
                let value: Value = serde_json::from_str(output["output"].as_str().expect("tool result JSON"))
                    .expect("known tool output");
                if reply.has_exit() {
                    assert_eq!(value["exitCode"], 17);
                    assert_eq!(value["aborted"], *reply == LocalExecReply::AbortedExit);
                    assert_eq!(value["outcomeUnknown"], false);
                    assert_eq!(value["resultRef"], format!(
                        "fabushi-local-exec://operation/lxo-{}-first", reply.name(),
                    ));
                } else {
                    assert_eq!(value["isError"], true);
                    assert!(value["error"].as_str().is_some_and(|message| !message.is_empty()));
                }
            }
        }
    }
}

#[test]
fn production_generated_runner_retains_unknown_external_shell_effects() {
    if let Some(root) = std::env::var_os(LXO_ROOT_ENV) {
        run_local_exec_runner_child(PathBuf::from(root));
        return;
    }
    run_local_exec_cases(LXO_UNKNOWN_TEST, &[
        LocalExecReply::Close, LocalExecReply::CancelClose, LocalExecReply::Silence,
        LocalExecReply::MalformedClient, LocalExecReply::MalformedControl, LocalExecReply::OpaqueThrow,
        LocalExecReply::TaggedAfterStart, LocalExecReply::FileError,
        LocalExecReply::StartedJournalFailure, LocalExecReply::TerminalJournalFailure,
    ], false);
}

#[test]
fn production_generated_runner_preserves_known_external_shell_outcomes() {
    if let Some(root) = std::env::var_os(LXO_ROOT_ENV) {
        run_local_exec_runner_child(PathBuf::from(root));
        return;
    }
    run_local_exec_cases(LXO_KNOWN_TEST, &[
        LocalExecReply::Exit, LocalExecReply::AbortedExit, LocalExecReply::CancelThenExit,
        LocalExecReply::PreEntryTag, LocalExecReply::ShellRejected,
        LocalExecReply::ShellPermissionDenied, LocalExecReply::LocalPermission,
    ], false);
}

#[test]
fn production_external_shell_adapter_types_uncertain_results_and_errors() {
    if let Some(root) = std::env::var_os(LXO_ROOT_ENV) {
        run_local_exec_adapter_child(PathBuf::from(root));
        return;
    }
    run_local_exec_cases(LXO_ADAPTER_TEST, &[
        LocalExecReply::Close, LocalExecReply::MalformedClient, LocalExecReply::TerminalJournalFailure,
    ], true);
}
