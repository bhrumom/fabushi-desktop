//! Real Responses parsing, production persistence, attempt custody and local MCP.
//! Controlled transport establishes this boundary, not external-provider acceptance.

use std::collections::VecDeque;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::inference::codex_direct_responses::{
    CodexDirectError, CodexDirectOptions, CodexDirectTool, CodexDirectTransport,
    run_codex_direct_responses_with_lifecycle_and_tool_step_reminder,
};
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedProviderCheckpoint, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
    ProductionTurnRunShellAdapter, ProviderRetryEvent, ProviderRetryOutcome,
    ProviderRetryReport, RoutedProviderAttemptControl, RoutedProviderAttemptExecutor,
    RoutedProviderCheckpointStore, ToolEffectUnknownResult,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedToolBridge,
    start_routed_mcp_server_for_attempt,
};
use mahayana_host_runtime::runner::tools::mcp_meta_tools::execute_routed_tool_with_timeout_ms;
use mahayana_host_runtime::runner::{StreamAttemptPolicy, StreamAttemptRuntime};
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
        run_codex_direct_responses_with_lifecycle_and_tool_step_reminder(
            &mut self.transport,
            &options,
            resume,
            &mut |tool, _args, _call_id| {
                let name = tool.name.clone();
                let worker_name = name.clone();
                let effects = Arc::clone(&effects);
                let action = Arc::clone(&action);
                let worker_attempt = attempt.clone();
                execute_routed_tool_with_timeout_ms(attempt, name, timeout_ms, move || {
                    effects.lock().expect("effect receipt").push(worker_name.clone());
                    action(&worker_attempt, &worker_name);
                    Ok(json!({"effect":worker_name}))
                }).map_err(|error| CodexDirectError::Tool(error.to_string()))
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
        let guard = self.open.lock().expect("release gate");
        let (guard, _) = self.changed.wait_timeout_while(guard, BOUND, |open| !*open)
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

fn post_mcp(url: &str, body: Value) -> Value {
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


// This final oracle reaches the private production executor through its public
// Runner owner. Child-scoped transport configuration cannot affect other tests.
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
                captured.lock().map_err(|_| "request capture poisoned")?.push(request);
                // Every replay without the missing result asks for the same
                // non-idempotent effect; the fixed owner sends only one request.
                let body = format!("data: {}\n\n", json!({
                    "type":"response.completed",
                    "response":{
                        "id":"wire-response",
                        "output":[{"type":"function_call","name":"effect","call_id":"wire-effect","arguments":"{}"}]
                    }
                }));
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
        .args(["--exact", WIRE_TEST, "--nocapture", "--test-threads=1"])
        .env(WIRE_ROOT_ENV, &root.0)
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
    assert_eq!(receipt["effects"], 1);
    assert_eq!(receipt["retries"], 0);
    let requests = server.requests.lock().expect("captured Responses");
    assert_eq!(requests.len(), 1, "production executor must not dispatch another attempt");
    assert_eq!(requests[0]["model"], MODEL);
}


