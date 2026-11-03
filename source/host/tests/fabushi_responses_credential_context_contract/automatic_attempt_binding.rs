//! One public Runner invocation owns one credential binding across its real retries.
//! Durable checkpoint bytes here prove in-process continuation, not restart recovery.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mahayana_host_runtime::extensions::inference::codex_direct_responses::CodexDirectUsage;
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, ProviderTokenUsage, RoutedProvider,
    RoutedProviderCheckpoint, RoutedToolDefinition,
};
use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
    ProviderRetryEvent, ProviderRetryOutcome, ProviderRetryReport, ToolEffectCustodySnapshot,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedProviderRun,
    RoutedToolBridge, RunnerRequestContextSnapshot, run_routed_provider_in_runner,
};
use mahayana_host_runtime::runner::{StreamAttemptPolicy, StreamAttemptRuntime};
use serde_json::{Value, json};

use super::{
    CapturedRequest, Loopback, OwnedChild, REFRESH_SECRET, REQUEST_PATH, TOKEN_A, TOKEN_B,
    TempRoot, child_panic_location, contains_private_diagnostic, fixture_io_error,
    install_image, read_child_output, record_child_observation, record_progress,
    replacement, write_child_receipt,
};

const TEST_NAME: &str =
    "automatic_attempt_binding::fabushi_runner_retains_binding_across_automatic_attempts";
const CHILD_ROOT: &str = "FABUSHI_AUTOMATIC_BINDING_CHILD_ROOT";
const CHILD_NONCE: &str = "FABUSHI_AUTOMATIC_BINDING_CHILD_NONCE";
const CHILD_CASE: &str = "FABUSHI_AUTOMATIC_BINDING_CHILD_CASE";
const MODEL: &str = "Vendor/Nested/CaseSensitive@Attempt-Binding";
const FALLBACK_MODEL: &str = "environment-must-not-replace-the-bound-model";
const AGENT: &str = "automatic-attempt-binding-agent";
const PRIMARY: &str = "original-account-a-input";
const LATER: &str = "independent-account-b-input";
const CALL_ID: &str = "automatic-binding-effect";
const CANCEL_REASON: &str = "attempt-binding retry callback cancellation";
const TOOL_RESPONSE: &str = "automatic-binding-tool-response";

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    replacement: &'static str,
    tool: bool,
    success: bool,
    cancel: bool,
    later: bool,
}

impl Case {
    fn refusal_request(self) -> usize { if self.tool { 2 } else { 1 } }

    fn expected_kind(self) -> &'static str {
        if self.cancel { "transport" } else if self.success { "success" } else { "authentication" }
    }

    fn request_count(self) -> usize {
        self.refusal_request() + usize::from(self.success) + usize::from(self.later)
    }
}

fn cases() -> Vec<Case> {
    let refusal = |name, replacement, later| Case {
        name, replacement, later, tool: false, success: false, cancel: false,
    };
    let mut cases = vec![
        refusal("reread-user", "user", true),
        refusal("reread-device", "device", false),
        refusal("reread-session", "session", false),
        refusal("reread-issuer", "issuer", false),
        refusal("reread-malformed", "malformed", false),
        refusal("reread-expired", "expired", false),
        refusal("reread-missing-identity", "missing-sessionId", false),
        refusal("reread-missing-file", "missing-file", false),
        refusal("reread-directory", "directory", false),
        Case {
            name: "same-context-rotation", replacement: "rotate", tool: false,
            success: true, cancel: false, later: false,
        },
        Case {
            name: "accepted-checkpoint-context-change", replacement: "user", tool: true,
            success: false, cancel: false, later: false,
        },
        Case {
            name: "accepted-checkpoint-token-rotation", replacement: "rotate", tool: true,
            success: true, cancel: false, later: false,
        },
        Case {
            name: "accepted-checkpoint-retry-cancellation", replacement: "user", tool: true,
            success: false, cancel: true, later: false,
        },
    ];
    #[cfg(unix)]
    cases.extend([
        refusal("reread-public-file", "public-file", false),
        refusal("reread-symlink", "symlink", false),
    ]);
    cases
}

fn reasoning_item() -> Value {
    json!({
        "type": "reasoning", "id": "automatic-binding-reasoning", "summary": [],
        "encrypted_content": "opaque-automatic-binding-reasoning",
    })
}

fn call_item() -> Value {
    json!({
        "type": "function_call", "name": "effect", "call_id": CALL_ID,
        "arguments": json!({"delta": 1}).to_string(),
    })
}

fn effect_result(sequence: usize) -> Value {
    json!({"effect": "appended", "sequence": sequence, "callId": CALL_ID})
}

fn accepted_history(initial: &[Value]) -> Vec<Value> {
    let mut input = initial.to_vec();
    input.extend([
        reasoning_item(), call_item(),
        json!({
            "type": "function_call_output", "call_id": CALL_ID,
            "output": effect_result(1).to_string(),
        }),
    ]);
    input
}

fn response_body(tool: bool) -> String {
    let completed = json!({
        "type": "response.completed",
        "response": {
            "id": if tool { TOOL_RESPONSE } else { "automatic-binding-completion" },
            "output": if tool { vec![reasoning_item(), call_item()] } else { Vec::new() },
            "usage": {
                "input_tokens": if tool { 7 } else { 11 },
                "output_tokens": if tool { 3 } else { 2 },
                "input_tokens_details": {"cached_tokens": if tool { 2 } else { 4 }},
            },
        },
    });
    let delta = if tool { String::new() } else {
        format!("data: {}\n\n", json!({"type": "response.output_text.delta", "delta": "done"}))
    };
    format!("{delta}data: {completed}\n\ndata: [DONE]\n\n")
}

type CheckpointFiles = Vec<(PathBuf, Vec<u8>)>;

fn store(root: &Path, stream: &str) -> ProductionRoutedProviderCheckpointStore {
    ProductionRoutedProviderCheckpointStore::new(root, AGENT, stream)
}

fn checkpoint_files(store: &ProductionRoutedProviderCheckpointStore) -> Result<CheckpointFiles, String> {
    let entries = match fs::read_dir(store.directory()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(fixture_io_error("checkpoint-directory", error)),
    };
    let mut files = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| fixture_io_error("checkpoint-entry", error))?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            return Err("checkpoint directory contains an uncommitted or unexpected file".into());
        }
        let bytes = fs::read(&path).map_err(|error| fixture_io_error("checkpoint-read", error))?;
        files.push((path, bytes));
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}

fn effect_bytes(root: &Path) -> Result<Vec<u8>, String> {
    match fs::read(root.join("effects.txt")) {
        Ok(bytes) => Ok(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(fixture_io_error("effect-receipt-read", error)),
    }
}

fn require(condition: bool, failure: &'static str) -> Result<(), String> {
    if condition { Ok(()) } else { Err(failure.into()) }
}

#[derive(Clone)]
struct AcceptedBoundary {
    files: CheckpointFiles,
    effects: Vec<u8>,
}

fn accepted_boundary(root: &Path, request: &Value, initial: &[Value]) -> Result<AcceptedBoundary, String> {
    let files = checkpoint_files(&store(root, "primary"))?;
    require(files.len() == 1, "continuation must follow exactly one durable checkpoint")?;
    let checkpoint_text = String::from_utf8_lossy(&files[0].1);
    for material in [TOKEN_A, TOKEN_B, REFRESH_SECRET, "accessToken", "access_token",
        "expiresAtMs", "sessionId", "session_id", "deviceId", "device_id", "userId", "user_id", "issuer"]
    {
        require(!checkpoint_text.contains(material), "credential or binding state entered the durable checkpoint")?;
    }
    let checkpoint = serde_json::from_slice::<RoutedProviderCheckpoint>(&files[0].1)
        .map_err(|_| "actual checkpoint JSON could not be decoded")?;
    let RoutedProviderCheckpoint::Fabushi(checkpoint) = checkpoint else {
        return Err("actual checkpoint has the wrong provider tag".into());
    };
    require(checkpoint.model.as_deref() == Some(MODEL), "actual checkpoint changed model")?;
    require(checkpoint.completed_steps == 1 && checkpoint.tool_calls_completed == 1,
        "actual checkpoint must cover one completed tool step")?;
    require(checkpoint.text.is_empty(), "tool-only checkpoint must not invent text")?;
    require(checkpoint.response_id == TOOL_RESPONSE, "actual checkpoint lost the tool response ID")?;
    require(checkpoint.usage == CodexDirectUsage {
        input_tokens: 7, output_tokens: 3, cache_read_tokens: 2, cache_write_tokens: 0,
    }, "actual checkpoint lost accumulated tool-response usage")?;
    let expected = accepted_history(initial);
    require(checkpoint.input == expected, "actual checkpoint lost balanced tool/reasoning history")?;
    require(request["input"].as_array() == Some(&expected),
        "continuation input differs from its accepted checkpoint")?;
    let effects = effect_bytes(root)?;
    require(effects == format!("{CALL_ID}\n").as_bytes(),
        "continuation must follow one durable non-idempotent effect")?;
    Ok(AcceptedBoundary { files, effects })
}

/// Only the response script lives here. The parent module's existing Loopback
/// continues to own accept/read/capture/write, deadlines, stop and joined reports.
pub(super) struct ResponsePlan {
    case: Case,
    root: PathBuf,
    credential_path: PathBuf,
    initial_input: Option<Vec<Value>>,
    before_503: Arc<Mutex<Option<AcceptedBoundary>>>,
}

impl ResponsePlan {
    pub(super) fn response(
        &mut self,
        request: &CapturedRequest,
        connection: usize,
        origin: &str,
        progress: &Mutex<Vec<String>>,
    ) -> Result<(&'static str, &'static str, String), String> {
        let body: Value = serde_json::from_slice(&request.body)
            .map_err(|_| "automatic-attempt request is not JSON")?;
        if connection == 1 {
            self.initial_input = Some(body["input"].as_array()
                .ok_or("automatic-attempt request has no input array")?.clone());
        }
        if connection == self.case.refusal_request() {
            if self.case.tool {
                let boundary = accepted_boundary(&self.root, &body,
                    self.initial_input.as_deref().ok_or("initial request was not captured")?)?;
                *self.before_503.lock().map_err(|_| "checkpoint observation lock poisoned")? = Some(boundary);
                record_progress(progress, connection, "accepted-checkpoint-observed")?;
            }
            install_image(&self.credential_path, replacement(self.case.replacement, origin))?;
            record_progress(progress, connection, "automatic-replacement-installed")?;
            return Ok(("503 Service Unavailable", "application/json", json!({
                "error": "native-http-error-body", "accessToken": TOKEN_A,
                "replacementToken": TOKEN_B, "refreshToken": REFRESH_SECRET,
            }).to_string()));
        }
        // A replay that drops the accepted history asks for the effect again.
        // Do not close the server or make illicit requests fail at the fixture.
        let contains_result = body["input"].as_array().is_some_and(|items| items.iter().any(|item| {
            item["type"] == "function_call_output" && item["call_id"] == CALL_ID
        }));
        Ok(("200 OK", "text/event-stream", response_body(self.case.tool && !contains_result)))
    }
}

struct EffectBridge {
    enabled: bool,
    receipt: PathBuf,
    effects: Arc<AtomicUsize>,
}

impl RoutedToolBridge for EffectBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(if self.enabled { vec![RoutedToolDefinition {
            name: "effect".into(), provider_identifier: "automatic-attempt-contract".into(),
            tool_name: "effect".into(), description: Some("Append a counted effect receipt".into()),
            input_schema: json!({"type": "object", "properties": {"delta": {"type": "integer"}}}),
        }] } else { Vec::new() })
    }

    fn call_tool(&self, tool: &RoutedToolDefinition, args: Value, call_id: &str)
        -> Result<Value, ProviderSessionError>
    {
        if !self.enabled || tool.name != "effect" || call_id != CALL_ID || args != json!({"delta": 1}) {
            return Err(ProviderSessionError::Tool("unexpected counted-effect invocation".into()));
        }
        let mut file = fs::OpenOptions::new().create(true).append(true).open(&self.receipt)
            .map_err(|error| ProviderSessionError::Tool(fixture_io_error("effect-open", error)))?;
        writeln!(file, "{call_id}")
            .map_err(|error| ProviderSessionError::Tool(fixture_io_error("effect-append", error)))?;
        file.sync_all().map_err(|error| ProviderSessionError::Tool(fixture_io_error("effect-sync", error)))?;
        let sequence = self.effects.fetch_add(1, Ordering::AcqRel) + 1;
        Ok(effect_result(sequence))
    }
}

#[derive(Default)]
struct Callbacks {
    events: Vec<ProviderRetryEvent>,
    reports: Vec<ProviderRetryReport>,
    retry_custody: Vec<ToolEffectCustodySnapshot>,
    order: Vec<&'static str>,
}

struct Invocation {
    result: Result<String, ProviderSessionError>,
    deltas: Vec<(String, String)>,
    usages: Vec<ProviderTokenUsage>,
    effects: usize,
    files: Result<CheckpointFiles, String>,
    effect_bytes: Result<Vec<u8>, String>,
    callbacks: Callbacks,
    custody: ToolEffectCustodySnapshot,
    cancelled: bool,
    cancellation_reason: Option<String>,
}

fn usage_observation(usage: &ProviderTokenUsage) -> Value {
    json!({
        "input": usage.input_tokens, "output": usage.output_tokens,
        "cacheRead": usage.cache_read_tokens, "cacheWrite": usage.cache_write_tokens,
        "reasoning": usage.reasoning_tokens,
    })
}

impl Invocation {
    fn observation(&self) -> Value {
        let (kind, message, debug) = match &self.result {
            Ok(text) => ("success", text.clone(), String::new()),
            Err(error) => (
                match error {
                    ProviderSessionError::Authentication(_) => "authentication",
                    ProviderSessionError::Transport(_) => "transport",
                    ProviderSessionError::Configuration(_) => "configuration",
                    ProviderSessionError::Cancelled(_) => "cancelled",
                    ProviderSessionError::Protocol(_) => "protocol",
                    ProviderSessionError::Tool(_) => "tool",
                    ProviderSessionError::McpCatalog(_) => "mcp-catalog",
                    ProviderSessionError::ToolOutcomeUnknown(_) => "unknown-tool",
                    ProviderSessionError::ProviderRefusal(_) => "provider-refusal",
                },
                error.to_string(), format!("{error:?}"),
            ),
        };
        json!({
            "kind": kind, "message": message, "debug": debug,
            "deltas": self.deltas, "usages": self.usages.iter().map(usage_observation).collect::<Vec<_>>(),
            "effects": self.effects, "checkpoints": self.files.as_ref().ok().map(Vec::len),
            "checkpointReadError": self.files.as_ref().err(),
            "effectBytes": self.effect_bytes.as_ref().ok().map(Vec::len),
            "effectReadError": self.effect_bytes.as_ref().err(),
            "custody": self.custody, "retryCustody": self.callbacks.retry_custody,
            "cancelled": self.cancelled, "cancellationReason": self.cancellation_reason,
            "callbackOrder": self.callbacks.order,
            "retries": self.callbacks.events.iter().map(|event| json!({
                "attempt": event.attempt, "nextAttempt": event.next_attempt,
                "delayMs": event.delay_ms, "resume": event.resume_from_checkpoint,
                "watchdog": event.watchdog_expired,
            })).collect::<Vec<_>>(),
            "reports": self.callbacks.reports.iter().map(|report| json!({
                "outcome": match report.outcome {
                    ProviderRetryOutcome::Retried => "Retried",
                    ProviderRetryOutcome::Exhausted => "Exhausted",
                    ProviderRetryOutcome::GaveUpIneligible => "GaveUpIneligible",
                },
                "attempt": report.attempt, "maxAttempts": report.max_attempts,
                "delayMs": report.delay_ms, "serverPaced": report.server_paced,
                "resume": report.resume_from_checkpoint, "watchdog": report.watchdog_expired,
                "errorType": report.error_type, "errorCode": report.error_code,
                "cause": report.cause, "error": report.error,
            })).collect::<Vec<_>>(),
        })
    }
}

fn run_invocation(root: &Path, nonce: &str, case: Case, later: bool) -> Invocation {
    let runtime = Arc::new(StreamAttemptRuntime::new(StreamAttemptPolicy {
        max_attempts: 3, initial_backoff: Duration::ZERO, max_backoff: Duration::ZERO,
        max_retry_after: Duration::ZERO, ..StreamAttemptPolicy::default()
    }));
    let cancellation = RoutedProviderCancellation::default();
    let checkpoint_store = Arc::new(store(root, if later { "later" } else { "primary" }));
    let effects = Arc::new(AtomicUsize::new(0));
    let callbacks = Arc::new(Mutex::new(Callbacks::default()));
    let usages = Arc::new(Mutex::new(Vec::new()));
    let retry_callbacks = Arc::clone(&callbacks);
    let report_callbacks = Arc::clone(&callbacks);
    let retry_runtime = Arc::clone(&runtime);
    let retry_cancellation = cancellation.clone();
    let usage_capture = Arc::clone(&usages);
    let mut deltas = Vec::new();
    let messages = [ProviderMessage {
        role: "user".into(), content: format!("{}:{nonce}", if later { LATER } else { PRIMARY }),
    }];
    let result = run_routed_provider_in_runner(
        RoutedProviderRun {
            provider: RoutedProvider::Fabushi, fabushi_model: Some(MODEL.into()),
            codex_model: None,
            fabushi_request_binding: None,
            data_dir: root, messages: &messages, typed_replay: None, cursor_auth: None,
            bridge: Arc::new(EffectBridge {
                enabled: case.tool && !later, receipt: root.join("effects.txt"), effects: Arc::clone(&effects),
            }),
            request_context: RunnerRequestContextSnapshot {
                context: HostRequestContext {
                    os_version: "automatic-attempt-contract".into(), shell: None,
                    time_zone: Some("UTC".into()), user_full_name: None,
                    transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(),
                },
                rules: None,
            },
            cancellation: cancellation.clone(), checkpoint_store: checkpoint_store.clone(),
            retry_sink: Some(Arc::new(move |event| {
                {
                    let mut observed = retry_callbacks.lock().expect("retry observations");
                    observed.events.push(event.clone());
                    observed.retry_custody.push(retry_runtime.effect_custody());
                    observed.order.push("retry");
                }
                if case.cancel && !later { retry_cancellation.cancel(CANCEL_REASON); }
            })),
            retry_report_sink: Some(Arc::new(move |report| {
                let mut observed = report_callbacks.lock().expect("report observations");
                observed.reports.push(report.clone());
                observed.order.push("report");
            })),
            stream_attempt_runtime: Some(Arc::clone(&runtime)),
            usage_sink: Some(Arc::new(move |usage| {
                usage_capture.lock().expect("usage observations").push(usage);
            })),
            cloud_agents_enabled: false, multitask_enabled: false, is_computer_use_subagent: false,
            max_steps: 4, tool_step_reminder: None,
        },
        &mut |delta, accumulated| deltas.push((delta.to_string(), accumulated.to_string())),
    );
    let callbacks = std::mem::take(&mut *callbacks.lock().expect("completed callbacks"));
    let usages = std::mem::take(&mut *usages.lock().expect("completed usages"));
    Invocation {
        result, deltas, usages, effects: effects.load(Ordering::Acquire),
        files: checkpoint_files(&checkpoint_store), effect_bytes: effect_bytes(root), callbacks,
        custody: runtime.effect_custody(), cancelled: cancellation.is_cancelled(),
        cancellation_reason: cancellation.reason(),
    }
}

fn assert_report(report: &ProviderRetryReport, attempt: u32, outcome: ProviderRetryOutcome, resume: bool) {
    assert_eq!(report.attempt, attempt);
    assert_eq!(report.max_attempts, 3);
    assert_eq!(report.outcome, outcome);
    assert_eq!(report.resume_from_checkpoint, resume);
    assert!(!report.watchdog_expired);
    assert!(!report.server_paced);
    let retried = outcome == ProviderRetryOutcome::Retried;
    assert_eq!(report.delay_ms, retried.then_some(0));
    assert!(report.error_type == if retried { "TransportError" } else { "AuthenticationError" },
        "actual retry report retained the wrong typed error");
    assert!(report.error_code == if retried { "SAND-E0406" } else { "SAND-E0405" },
        "actual retry report retained the wrong error code");
    assert!(!contains_private_diagnostic(&format!("{} {} {report:?}", report.cause, report.error)),
        "actual retry diagnostics contain credential or HTTP-body material");
}

fn assert_invocation(observed: &Invocation, case: Case, later: bool) {
    let success = later || case.success;
    if success {
        assert!(matches!(&observed.result, Ok(text) if text == "done"), "public Runner must complete");
    } else if case.cancel {
        // The public entry's final cancellation.check() retains this current projection.
        assert!(matches!(&observed.result, Err(ProviderSessionError::Transport(reason)) if reason == CANCEL_REASON),
            "retry-callback cancellation must retain the outer Transport reason");
    } else {
        assert!(matches!(&observed.result, Err(ProviderSessionError::Authentication(_))),
            "automatic attempt must preserve typed Authentication refusal");
    }
    assert!(!contains_private_diagnostic(&observed.observation().to_string()),
        "Runner observations must not expose credentials or error response bodies");
    assert_eq!(observed.cancelled, case.cancel && !later);
    assert_eq!(observed.cancellation_reason.as_deref(), (case.cancel && !later).then_some(CANCEL_REASON));
    let tools = usize::from(case.tool && !later);
    assert_eq!(observed.effects, tools);
    assert!(observed.files.is_ok() && observed.effect_bytes.is_ok(), "actual durable observations failed");
    assert_eq!(observed.files.as_ref().unwrap().len(), tools);
    let expected_effect_bytes = if tools == 1 { format!("{CALL_ID}\n").into_bytes() } else { Vec::new() };
    assert!(observed.effect_bytes.as_ref().unwrap() == &expected_effect_bytes,
        "counted non-idempotent effect must occur exactly once or remain absent");
    let custody = ToolEffectCustodySnapshot {
        started_revision: tools as u64, checkpoint_revision: tools as u64,
        pending_effects: 0, unknown_result: None,
    };
    assert_eq!(observed.custody, custody);
    if later {
        assert!(observed.callbacks.events.is_empty() && observed.callbacks.reports.is_empty());
        assert!(observed.callbacks.retry_custody.is_empty() && observed.callbacks.order.is_empty());
    } else {
        assert_eq!(observed.callbacks.events, [ProviderRetryEvent {
            attempt: 1, next_attempt: 2, delay_ms: 0,
            resume_from_checkpoint: case.tool, watchdog_expired: false,
        }]);
        assert_eq!(observed.callbacks.retry_custody, [custody]);
        let refused = !case.success && !case.cancel;
        assert_eq!(observed.callbacks.reports.len(), if refused { 2 } else { 1 });
        assert_report(&observed.callbacks.reports[0], 1, ProviderRetryOutcome::Retried, case.tool);
        if refused {
            assert_report(&observed.callbacks.reports[1], 2, ProviderRetryOutcome::GaveUpIneligible, case.tool);
        }
        assert_eq!(observed.callbacks.order,
            if refused { vec!["report", "retry", "report"] } else { vec!["report", "retry"] });
    }
    if success {
        assert_eq!(observed.deltas, [("done".to_string(), "done".to_string())]);
        assert_eq!(observed.usages, [ProviderTokenUsage {
            input_tokens: if tools == 1 { 18 } else { 11 },
            output_tokens: if tools == 1 { 5 } else { 2 },
            cache_read_tokens: if tools == 1 { 6 } else { 4 },
            cache_write_tokens: 0, reasoning_tokens: None,
        }]);
    } else {
        assert!(observed.deltas.is_empty(), "refusal cannot publish text");
        assert!(observed.usages.is_empty(), "refusal cannot publish terminal usage");
    }
}

fn run_child(root: &Path, case: Case) {
    let nonce = std::env::var(CHILD_NONCE).expect("exact child nonce");
    let mut receipt = json!({
        "test": TEST_NAME, "pid": std::process::id(), "nonce": nonce,
        "case": case.name, "expected": case.expected_kind(), "phase_expected": case.expected_kind(),
        "phase": "first-running",
        "observation_field": "first", "completed": false, "diagnostics_redacted": false,
        "first": null, "later": null,
    });
    write_child_receipt(root, &receipt);
    let first = run_invocation(root, &nonce, case, false);
    record_child_observation(root, &mut receipt, "first", &first.observation());
    assert_invocation(&first, case, false);
    if case.later {
        receipt["phase"] = json!("later-running");
        receipt["observation_field"] = json!("later");
        receipt["phase_expected"] = json!("success");
        write_child_receipt(root, &receipt);
        let later = run_invocation(root, &nonce, case, true);
        record_child_observation(root, &mut receipt, "later", &later.observation());
        assert_invocation(&later, case, true);
    }
    assert_eq!(receipt["diagnostics_redacted"], false, "redaction remains a failed observation");
    receipt["completed"] = json!(true);
    receipt["phase"] = json!("complete");
    write_child_receipt(root, &receipt);
}

fn child_evidence(receipt: &Value, nonce: &str, pid: Option<u32>, case: Case) -> Value {
    let first = &receipt["first"];
    let later = receipt["observation_field"] == "later";
    let current = if later { &receipt["later"] } else { first };
    json!({
        "testMatches": receipt["test"] == TEST_NAME,
        "nonceMatches": receipt["nonce"] == nonce,
        "pidMatches": receipt["pid"].as_u64() == pid.map(u64::from),
        "caseMatches": receipt["case"] == case.name,
        "kindMatches": first["kind"] == case.expected_kind(),
        "observation": if later { "later" } else { "first" },
        "currentKind": match current["kind"].as_str() {
            Some("success") => "success", Some("authentication") => "authentication",
            Some("transport") => "transport", Some("configuration") => "configuration",
            Some("cancelled") => "cancelled", Some("protocol") => "protocol",
            Some("tool") => "tool", Some("unknown-tool") => "unknown-tool",
            Some("provider-refusal") => "provider-refusal", _ => "unavailable",
        },
        "currentKindMatches": current["kind"] == if later { "success" } else { case.expected_kind() },
        "phase": match receipt["phase"].as_str() {
            Some("first-running") => "first-running", Some("first-observed") => "first-observed",
            Some("later-running") => "later-running", Some("later-observed") => "later-observed",
            Some("complete") => "complete", _ => "unavailable",
        },
        "effects": current["effects"].as_u64(), "checkpoints": current["checkpoints"].as_u64(),
        "deltas": current["deltas"].as_array().map(Vec::len),
        "usages": current["usages"].as_array().map(Vec::len),
        "retries": current["retries"].as_array().map(Vec::len),
        "reports": current["reports"].as_array().map(Vec::len),
        "completed": receipt["completed"] == true,
        "diagnosticsRedacted": receipt["diagnostics_redacted"] == true,
    })
}

fn assert_wire_requests(requests: &[CapturedRequest], origin: &str, nonce: &str, case: Case) {
    assert_eq!(requests.len(), case.request_count(), "actual HTTP request count for {}", case.name);
    let mut bodies = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        assert!(request.request_line == format!("POST {REQUEST_PATH} HTTP/1.1"), "captured complete Responses path");
        assert!(request.host == origin.strip_prefix("http://").unwrap(), "captured Responses host");
        let token = if index < case.refusal_request() { TOKEN_A } else { TOKEN_B };
        assert!(request.authorization == format!("Bearer {token}"), "captured bearer changed at the wrong boundary");
        let expected_status = if index + 1 == case.refusal_request() { "503 Service Unavailable" } else { "200 OK" };
        assert_eq!(request.response_status, expected_status);
        let body: Value = serde_json::from_slice(&request.body)
            .unwrap_or_else(|_| panic!("captured actual Responses JSON is malformed"));
        assert!(body["model"] == MODEL, "actual request must keep the frozen model");
        assert!(body["stream"] == true, "actual request must remain streaming");
        assert!(body["store"] == false, "actual request must retain disabled provider storage");
        assert!(body["include"] == json!(["reasoning.encrypted_content"]), "actual request must retain encrypted reasoning inclusion");
        assert!(body["instructions"].as_str().is_some_and(|text| text.contains(&format!(
            "Runtime inference metadata: provider=Fabushi first-party Responses API; model={MODEL}."
        ))), "actual instructions must retain frozen runtime metadata");
        assert!(!String::from_utf8_lossy(&request.body).contains(FALLBACK_MODEL));
        for secret in [TOKEN_A, TOKEN_B, REFRESH_SECRET, "accessToken", "expiresAtMs"] {
            assert!(!String::from_utf8_lossy(&request.body).contains(secret), "credential material entered Responses JSON");
        }
        let later = case.later && index + 1 == requests.len();
        let input = body["input"].to_string();
        assert!(input.contains(&format!("{}:{nonce}", if later { LATER } else { PRIMARY })),
            "captured request belongs to the wrong invocation");
        assert!(!input.contains(if later { PRIMARY } else { LATER }),
            "a new account request inherited the preceding invocation input");
        if case.tool {
            assert!(body["tool_choice"] == "auto", "actual request must retain automatic tool selection");
            assert!(body["parallel_tool_calls"] == true, "actual request must retain its tool-call bit");
            assert_eq!(body["tools"].as_array().map(Vec::len), Some(1));
            assert!(body["tools"][0]["name"] == "effect", "actual request must declare the guarded effect tool");
        } else {
            assert!(body.get("tools").is_none() && body.get("tool_choice").is_none()
                && body.get("parallel_tool_calls").is_none(), "effect-free requests must remain tool-free");
        }
        bodies.push(body);
    }
    assert!(bodies[0]["input"] == json!([{"role": "user", "content": format!("{PRIMARY}:{nonce}")}]),
        "the initial request must contain only its own user input");
    if case.tool {
        let expected = accepted_history(bodies[0]["input"].as_array().unwrap());
        assert!(bodies[1]["input"].as_array() == Some(&expected), "accepted continuation history changed");
        assert!(bodies[0]["instructions"] == bodies[1]["instructions"], "tool continuation changed instructions");
        if case.success {
            assert!(requests[1].body == requests[2].body, "automatic resume must preserve the exact continuation body");
        }
    } else if case.success {
        assert!(requests[0].body == requests[1].body, "effect-free retry must preserve the exact request body");
    }
    if case.later {
        assert!(bodies.last().unwrap()["input"] == json!([{"role": "user", "content": format!("{LATER}:{nonce}")}]),
            "a fresh invocation must not inherit the old input or tool history");
    }
}

fn run_parent(case: Case) {
    let root = TempRoot::new();
    let credential_path = root.0.join("credential.json");
    let before_503 = Arc::new(Mutex::new(None));
    let plan = ResponsePlan {
        case, root: root.0.clone(), credential_path: credential_path.clone(), initial_input: None,
        before_503: Arc::clone(&before_503),
    };
    let mut server = Loopback::start(&credential_path, case.replacement, false, false, None, None, Some(plan));
    server.case = case.name.into();
    server.phase = "automatic-attempt-binding".into();
    let nonce = uuid::Uuid::new_v4().to_string();
    let stdout_path = root.0.join("child.stdout");
    let stderr_path = root.0.join("child.stderr");
    let mut child_pid = None;
    let mut cleanup = None;
    // Keep errors as values until the exact child is stopped and the server is joined.
    let status = (|| -> Result<ExitStatus, String> {
        let executable = std::env::current_exe().map_err(|error| fixture_io_error("child-executable", error))?;
        let mut child = OwnedChild::new(Command::new(executable)
            .args(["--exact", TEST_NAME, "--nocapture", "--test-threads=1"])
            .env(CHILD_ROOT, &root.0).env(CHILD_NONCE, &nonce).env(CHILD_CASE, case.name)
            .env("FABUSHI_RESPONSES_URL", &server.endpoint)
            .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &credential_path)
            .env("FABUSHI_RESPONSES_MODEL", FALLBACK_MODEL)
            .env("NO_PROXY", "*").env("no_proxy", "*")
            .env_remove("SAND_AGENT_MOCK_RESPONSE").env_remove("SAND_AGENT_MODEL")
            .env_remove("SAND_PACKAGED").env_remove("FABUSHI_PRODUCT_MODE")
            .stdout(fs::File::create(&stdout_path).map_err(|error| fixture_io_error("child-stdout-create", error))?)
            .stderr(fs::File::create(&stderr_path).map_err(|error| fixture_io_error("child-stderr-create", error))?)
            .spawn().map_err(|error| fixture_io_error("child-spawn", error))?, &root);
        child_pid = Some(child.0.id());
        let result = child.wait();
        cleanup = child.1.clone();
        result
    })();
    let report = server.finish();
    let receipt = fs::read(root.0.join("receipt.json"))
        .map_err(|error| fixture_io_error("child-receipt-read", error))
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).map_err(|_| "invalid child receipt JSON".into()));
    let stdout = read_child_output(&stdout_path, "child-stdout-read");
    let stderr = read_child_output(&stderr_path, "child-stderr-read");
    let private_output = [&stdout, &stderr].iter().any(|output| {
        output.as_ref().is_ok_and(|text| contains_private_diagnostic(text))
    });
    let evidence = json!({
        "test": TEST_NAME, "case": case.name, "childPid": child_pid,
        "childExit": format!("{status:?}"), "childCleanup": cleanup,
        "server": report.evidence(), "childPanic": child_panic_location(&stderr),
        "child": receipt.as_ref().map(|value| child_evidence(value, &nonce, child_pid, case))
            .unwrap_or_else(|_| json!({"phase": "unavailable"})),
        "stdoutBytes": stdout.as_ref().ok().map(String::len),
        "stderrBytes": stderr.as_ref().ok().map(String::len), "privateOutput": private_output,
    });
    eprintln!("automatic-attempt-binding evidence: {evidence}");
    assert!(status.as_ref().is_ok_and(|status| status.success()) && report.result.is_ok()
        && receipt.is_ok() && stdout.is_ok() && stderr.is_ok() && !private_output,
        "automatic-attempt fixture failed: {evidence}; progress={:?}", report.progress);
    let receipt = receipt.unwrap();
    assert!(receipt["test"] == TEST_NAME && receipt["nonce"] == nonce
        && receipt["pid"].as_u64() == child_pid.map(u64::from)
        && receipt["case"] == case.name && receipt["expected"] == case.expected_kind(),
        "exact child identity or scenario receipt mismatch: {evidence}");
    assert!(receipt["completed"] == true && receipt["phase"] == "complete"
        && receipt["diagnostics_redacted"] == false, "incomplete or redacted child receipt: {evidence}");
    assert!(receipt["first"]["kind"] == case.expected_kind(), "wrong observed primary kind: {evidence}");
    assert_eq!(receipt["observation_field"], if case.later { "later" } else { "first" });
    if case.later { assert!(receipt["later"]["kind"] == "success", "fresh B invocation failed"); }
    assert_eq!(report.evidence()["accepted"], case.request_count());
    assert_eq!(report.evidence()["parsed"], case.request_count());
    assert_eq!(report.evidence()["responded"], case.request_count());
    assert_wire_requests(&report.requests, &server.origin, &nonce, case);
    let installed = format!("connection={} automatic-replacement-installed", case.refusal_request());
    let planned = format!("connection={} response-planned status=503 Service Unavailable", case.refusal_request());
    let written = format!("connection={} response-written", case.refusal_request());
    let position = |event: &str| report.progress.iter().position(|value| value == event).expect("ordered fixture phase");
    assert!(position(&installed) < position(&planned) && position(&planned) < position(&written),
        "credential replacement must finish before the complete 503 response");
    let before = before_503.lock().expect("accepted boundary observation").clone();
    let final_files = checkpoint_files(&store(&root.0, "primary")).expect("final actual checkpoint files");
    let final_effects = effect_bytes(&root.0).expect("final actual effect receipts");
    if case.tool {
        let before = before.expect("accepted checkpoint was observed before releasing 503");
        assert!(before.files == final_files, "retry/refusal must retain exactly the original durable checkpoint bytes");
        assert!(before.effects == final_effects && final_effects == format!("{CALL_ID}\n").as_bytes(),
            "retry/refusal must retain exactly one completed effect");
        let observed = format!("connection={} accepted-checkpoint-observed", case.refusal_request());
        assert!(position(&observed) < position(&installed), "accepted checkpoint must precede the credential swap");
    } else {
        assert!(before.is_none() && final_files.is_empty() && final_effects.is_empty());
    }
    assert!(checkpoint_files(&store(&root.0, "later")).expect("later checkpoint files").is_empty());
}

#[test]
fn fabushi_runner_retains_binding_across_automatic_attempts() {
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let selected = std::env::var(CHILD_CASE).expect("exact child scenario");
        let case = cases().into_iter().find(|case| case.name == selected).expect("known child scenario");
        run_child(Path::new(&root), case);
        return;
    }
    for case in cases() { run_parent(case); }
}
