use std::sync::{Weak, mpsc};
use std::time::Duration;

use serde_json::json;

use super::*;
use crate::runner::agent_v1_wire::{
    CommunicateUpdateToolCall, SendMessageSuccess, SendMessageText, send_message_args,
};
use crate::runner::routed_provider_runtime::RoutedToolBridge;
use crate::runner::tools::send_message_tool::{
    ResolvedAttachmentSource, SendMessageSink, SendMessageToolBridge, send_message_tool_definition,
};

fn recorder() -> Arc<ToolCallRecorder> {
    let recorder = Arc::new(ToolCallRecorder::default());
    recorder.begin_run().expect("begin private invocation");
    recorder
}

fn register_send(recorder: &Arc<ToolCallRecorder>, id: &str) -> CallReceipt {
    recorder.register(&send_message_tool_definition(), json!({"type":"text","content":"raw input"}), id, 900)
        .expect("register send")
}

fn initial(id: &str) -> AgentToolCall {
    AgentToolCall {
        tool: Some(agent_tool_call::Tool::SendMessageToolCall(SendMessageToolCall {
            args: Some(SendMessageArgs {
                message: Some(send_message_args::Message::Text(SendMessageText {
                    content: "actual normalized text".into(),
                })),
            }),
            result: None,
        })),
        tool_call_id: Some(id.into()),
        started_at_ms: Some(40),
        completed_at_ms: None,
    }
}

fn send_body(call: &mut AgentToolCall) -> &mut SendMessageToolCall {
    match call.tool.as_mut().expect("fixture tool") {
        agent_tool_call::Tool::SendMessageToolCall(send) => send,
        _ => panic!("fixture must be SendMessage"),
    }
}

fn completed(initial: &AgentToolCall, id: &str) -> AgentToolCall {
    let mut call = initial.clone();
    send_body(&mut call).result = Some(SendMessageResult {
        result: Some(send_message_result::Result::Success(SendMessageSuccess {
            timestamp: 50,
            message_id: id.into(),
        })),
    });
    call.completed_at_ms = Some(60);
    call
}

fn successful_result(id: &str) -> Result<Value, ProviderSessionError> {
    Ok(json!({"sent":true,"timestampMs":50,"messageId":id}))
}

fn emit_pair(recorder: &Arc<ToolCallRecorder>, initial: &AgentToolCall, completed: &AgentToolCall) {
    let sink = recorder.interaction_sink(None);
    sink.on_tool_call("initial", initial);
    sink.on_tool_call("completed", completed);
}

fn record_success(recorder: &Arc<ToolCallRecorder>, id: &str) -> AgentToolCall {
    let receipt = register_send(recorder, id);
    let initial = initial(id);
    let completed = completed(&initial, "sql-id");
    emit_pair(recorder, &initial, &completed);
    recorder.finish(receipt, &successful_result("sql-id"), 10).expect("finish actual observations");
    completed
}

fn wire(call: &CompletedRoutedToolCall) -> AgentToolCall {
    AgentToolCall::decode(call.send_message_wire.as_ref().expect("qualified SendMessage").as_bytes())
        .expect("frozen wire")
}

fn other_tool() -> RoutedToolDefinition {
    RoutedToolDefinition {
        name: "ordinary_tool".into(), provider_identifier: "fixture-provider".into(),
        tool_name: "ordinary_tool".into(), description: Some("ordinary tool".into()),
        input_schema: json!({"type":"object"}),
    }
}

struct EmptyBridge;

impl RoutedToolBridge for EmptyBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(Vec::new())
    }

    fn call_tool(&self, _: &RoutedToolDefinition, _: Value, _: &str) -> Result<Value, ProviderSessionError> {
        panic!("the actual SendMessage producer must handle the fixture")
    }
}

struct ProducerSink {
    outcome: Mutex<Option<Result<Option<String>, ProviderSessionError>>>,
    messages: Mutex<Vec<Value>>,
}

impl ProducerSink {
    fn new(outcome: Result<Option<String>, ProviderSessionError>) -> Arc<Self> {
        Arc::new(Self { outcome: Mutex::new(Some(outcome)), messages: Mutex::new(Vec::new()) })
    }
}

impl SendMessageSink for ProducerSink {
    fn resolve_attachment_source(&self, _: &str, _: &str) -> Result<ResolvedAttachmentSource, ProviderSessionError> {
        Ok(ResolvedAttachmentSource {
            url: "attachment://normalized-resource".into(), file_name: Some("normalized.png".into()),
        })
    }

    fn send_message(&self, message: Value, _: u64, _: &str) -> Result<Option<String>, ProviderSessionError> {
        self.messages.lock().unwrap().push(message);
        self.outcome.lock().unwrap().take().expect("one actual producer call")
    }
}

fn invoke_producer(
    recorder: &Arc<ToolCallRecorder>,
    sink: Arc<ProducerSink>,
    args: Value,
    live: Option<Arc<dyn SendMessageInteractionSink>>,
) -> (Result<Value, ProviderSessionError>, Vec<CompletedRoutedToolCall>) {
    let tool = send_message_tool_definition();
    let receipt = recorder.register(&tool, args.clone(), "producer-call", 900).expect("register producer");
    let bridge = SendMessageToolBridge::new(Arc::new(EmptyBridge), sink)
        .with_interaction_sink(recorder.interaction_sink(live));
    let result = bridge.call_tool(&tool, args, "producer-call");
    recorder.finish(receipt, &result, 10).expect("qualify producer result");
    (result, recorder.take_completed_checked().expect("checked producer drain"))
}

struct LiveObserver {
    recorder: Weak<ToolCallRecorder>,
    events: Mutex<Vec<(&'static str, AgentToolCall)>>,
}

impl SendMessageInteractionSink for LiveObserver {
    fn on_tool_call(&self, phase: &'static str, call: &AgentToolCall) {
        let recorder = self.recorder.upgrade().expect("live recorder");
        let state = recorder.state.try_lock().expect("fanout must release the recorder lock");
        assert!(state.active.contains_key(&thread::current().id()));
        drop(state);
        self.events.lock().unwrap().push((phase, call.clone()));
    }
}

#[test]
fn actual_producer_normalization_is_captured_without_live_observation() {
    let recorder = recorder();
    let sink = ProducerSink::new(Ok(Some("actual-sink-id".into())));
    let raw = json!({"type":"attachment","url":"file:///raw.png","alt":"actual alt"});
    let (result, calls) = invoke_producer(&recorder, Arc::clone(&sink), raw.clone(), None);
    let public = result.expect("actual producer success");
    assert_eq!(public["messageId"], "actual-sink-id");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].args, raw);
    assert_eq!(calls[0].result, Ok(public.clone()));
    assert_eq!((calls[0].sequence, calls[0].started_at_ms, calls[0].completed_at_ms), (0, 900, 10));
    let mut captured = wire(&calls[0]);
    assert_eq!(captured.tool_call_id.as_deref(), Some("producer-call"));
    assert!(captured.started_at_ms.is_some());
    assert!(captured.completed_at_ms.is_some());
    let send = send_body(&mut captured);
    assert!(matches!(send.args.as_ref().unwrap().message.as_ref(),
        Some(send_message_args::Message::Attachment(value))
            if value.url == "attachment://normalized-resource" && value.alt.as_deref() == Some("actual alt")));
    assert!(matches!(send.result.as_ref().unwrap().result.as_ref(),
        Some(send_message_result::Result::Success(value))
            if value.message_id == "actual-sink-id" && Some(value.timestamp) == public["timestampMs"].as_u64()));
    assert_eq!(sink.messages.lock().unwrap().as_slice(), &[json!({
        "type":"attachment", "url":"attachment://normalized-resource", "alt":"actual alt", "file_name":"normalized.png",
    })]);
}

#[test]
fn generic_none_id_preserves_the_actual_generated_success() {
    let recorder = recorder();
    let (result, calls) = invoke_producer(&recorder, ProducerSink::new(Ok(None)),
        json!({"type":"text","content":"generic success"}), None);
    let public = result.expect("generic sink success");
    assert_eq!(public["messageId"], Value::Null);
    let mut captured = wire(&calls[0]);
    assert!(matches!(send_body(&mut captured).result.as_ref().unwrap().result.as_ref(),
        Some(send_message_result::Result::Success(value))
            if value.message_id.is_empty() && Some(value.timestamp) == public["timestampMs"].as_u64()));
}

#[test]
fn actual_producer_error_keeps_its_original_type_and_generated_error() {
    let recorder = recorder();
    let (result, calls) = invoke_producer(&recorder,
        ProducerSink::new(Err(ProviderSessionError::Transport("sink transport failed".into()))),
        json!({"type":"text","content":"failed send"}), None);
    assert!(matches!(result, Err(ProviderSessionError::Transport(ref message)) if message == "sink transport failed"));
    assert_eq!(calls[0].result, Err("sink transport failed".into()));
    let mut captured = wire(&calls[0]);
    assert!(send_body(&mut captured).args.as_ref().unwrap().message.is_some());
    assert!(matches!(send_body(&mut captured).result.as_ref().unwrap().result.as_ref(),
        Some(send_message_result::Result::Error(error)) if error.error == "sink transport failed"));
}

#[test]
fn actual_producer_pre_observation_rejection_has_no_invented_arguments() {
    let recorder = recorder();
    let sink = ProducerSink::new(Ok(Some("must-not-send".into())));
    let (result, calls) = invoke_producer(&recorder, Arc::clone(&sink), json!({"type":"unsupported"}), None);
    assert!(matches!(result, Err(ProviderSessionError::Tool(_))));
    assert!(sink.messages.lock().unwrap().is_empty());
    let original_error = result.as_ref().unwrap_err().to_string();
    let mut captured = wire(&calls[0]);
    assert_eq!((captured.started_at_ms, captured.completed_at_ms), (Some(900), Some(10)));
    let send = send_body(&mut captured);
    assert_eq!(send.args, Some(SendMessageArgs { message: None }));
    assert!(matches!(send.result.as_ref().unwrap().result.as_ref(),
        Some(send_message_result::Result::Error(error)) if error.error == original_error));
}

#[test]
fn fanout_preserves_actual_events_after_releasing_the_recorder_lock() {
    let recorder = recorder();
    let live = Arc::new(LiveObserver { recorder: Arc::downgrade(&recorder), events: Mutex::new(Vec::new()) });
    let (result, calls) = invoke_producer(&recorder, ProducerSink::new(Ok(Some("live-id".into()))),
        json!({"type":"text","content":"live text"}), Some(live.clone()));
    assert!(result.is_ok());
    let events = live.events.lock().unwrap();
    assert_eq!(events.iter().map(|(phase, _)| *phase).collect::<Vec<_>>(), vec!["initial", "completed"]);
    assert_eq!(calls[0].send_message_wire.as_ref().unwrap().as_bytes(), events[1].1.encode_to_vec());
    assert_eq!(events[0].1.started_at_ms, events[1].1.started_at_ms);
}

#[test]
fn capture_faults_keep_live_events_and_the_public_result_unchanged() {
    let recorder = recorder();
    let receipt = register_send(&recorder, "expected-call");
    let live = Arc::new(LiveObserver { recorder: Arc::downgrade(&recorder), events: Mutex::new(Vec::new()) });
    let sink = recorder.interaction_sink(Some(live.clone()));
    let wrong = initial("different-call");
    sink.on_tool_call("initial", &wrong);
    let public = successful_result("actual-id");
    assert!(recorder.finish(receipt, &public, 70).is_err());
    assert_eq!(public.as_ref().unwrap(), &json!({"sent":true,"timestampMs":50,"messageId":"actual-id"}));
    assert_eq!(live.events.lock().unwrap().as_slice(), &[("initial", wrong)]);
    assert!(recorder.take_completed_checked().is_err());
}

#[test]
fn reversed_worker_completion_drains_in_registration_order() {
    let recorder = recorder();
    let (registered_tx, registered_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let first_recorder = Arc::clone(&recorder);
    let first = thread::spawn(move || {
        let receipt = register_send(&first_recorder, "first");
        let initial = initial("first");
        let sink = first_recorder.interaction_sink(None);
        sink.on_tool_call("initial", &initial);
        registered_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(30)).expect("release first after second finishes");
        sink.on_tool_call("completed", &completed(&initial, "first-sql"));
        first_recorder.finish(receipt, &successful_result("first-sql"), 70).expect("first finish");
    });
    registered_rx.recv_timeout(Duration::from_secs(30)).expect("first registration");
    let second_recorder = Arc::clone(&recorder);
    let second = thread::spawn(move || record_success(&second_recorder, "second"));
    second.join().expect("second completed while first remained active");
    release_tx.send(()).unwrap();
    first.join().expect("first completion");
    let calls = recorder.take_completed_checked().expect("ordered drain");
    assert_eq!(calls.iter().map(|call| (call.sequence, call.tool_call_id.as_str())).collect::<Vec<_>>(),
        vec![(0, "first"), (1, "second")]);
    assert_eq!(wire(&calls[0]).tool_call_id.as_deref(), Some("first"));
    assert_eq!(wire(&calls[1]).tool_call_id.as_deref(), Some("second"));
}

#[test]
fn late_old_worker_with_identical_id_cannot_attach_to_a_fresh_invocation() {
    let old = recorder();
    let (registered_tx, registered_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let old_worker = Arc::clone(&old);
    let worker = thread::spawn(move || {
        let receipt = register_send(&old_worker, "reused-id");
        let initial = initial("reused-id");
        let sink = old_worker.interaction_sink(None);
        sink.on_tool_call("initial", &initial);
        registered_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(30)).expect("allow late completion");
        sink.on_tool_call("completed", &completed(&initial, "old-sql"));
        assert!(old_worker.finish(receipt, &successful_result("old-sql"), 70).is_err());
    });
    registered_rx.recv_timeout(Duration::from_secs(30)).expect("old active call");
    assert!(old.take_completed_checked().is_err(), "unfinished drain permanently seals the old recorder");
    let fresh = recorder();
    let receipt = register_send(&fresh, "reused-id");
    let initial = initial("reused-id");
    let fresh_sink = fresh.interaction_sink(None);
    fresh_sink.on_tool_call("initial", &initial);
    release_tx.send(()).unwrap();
    worker.join().expect("old worker finishes while fresh call is active");
    let completed = completed(&initial, "fresh-sql");
    fresh_sink.on_tool_call("completed", &completed);
    fresh.finish(receipt, &successful_result("fresh-sql"), 70).expect("fresh owner remains eligible");
    let calls = fresh.take_completed_checked().expect("fresh drain");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].sequence, 0);
    assert_eq!(calls[0].send_message_wire.as_ref().unwrap().as_bytes(), completed.encode_to_vec());
    assert!(old.begin_run().is_err());
    assert!(old.take_completed_checked().is_err());
}

#[test]
fn duplicate_send_ids_are_refused_while_active_and_after_completion() {
    let active = recorder();
    let receipt = register_send(&active, "duplicate");
    let contender = Arc::clone(&active);
    thread::spawn(move || {
        assert!(contender.register(&send_message_tool_definition(), json!({}), "duplicate", 1).is_err());
    }).join().unwrap();
    drop(receipt);
    assert!(active.take_completed_checked().is_err());

    let finished = recorder();
    record_success(&finished, "duplicate");
    assert!(finished.register(&send_message_tool_definition(), json!({}), "duplicate", 1).is_err());
    assert!(finished.take_completed_checked().is_err());
}

#[test]
fn nested_registration_never_replaces_the_outer_receipt() {
    let recorder = recorder();
    let receipt = register_send(&recorder, "outer");
    assert!(recorder.register(&other_tool(), json!({}), "nested", 1).is_err());
    {
        let state = recorder.state.lock().unwrap();
        assert_eq!(state.active.len(), 1);
        assert_eq!(state.active[&thread::current().id()].tool_call_id, "outer");
    }
    drop(receipt);
    assert!(recorder.take_completed_checked().is_err());
}

#[test]
fn send_message_registration_requires_the_exact_first_party_definition() {
    for field in ["provider", "name", "tool_name"] {
        let recorder = recorder();
        let mut tool = send_message_tool_definition();
        match field {
            "provider" => tool.provider_identifier = "foreign-provider".into(),
            "name" => tool.name = "aliased-send".into(),
            _ => tool.tool_name = "aliased-send".into(),
        }
        assert!(recorder.register(&tool, json!({}), "call", 1).is_err(), "{field}");
        assert!(recorder.state.lock().unwrap().active.is_empty());
        assert!(recorder.take_completed_checked().is_err());
    }
}

#[test]
fn invalid_initial_callbacks_fault_the_invocation_without_latest_match_recovery() {
    for fault in ["id", "missing-id", "variant", "args", "argument-variant", "start", "result", "completion", "phase"] {
        let recorder = recorder();
        let receipt = register_send(&recorder, "call");
        let valid = initial("call");
        let mut invalid = valid.clone();
        match fault {
            "id" => invalid.tool_call_id = Some("another-call".into()),
            "missing-id" => invalid.tool_call_id = None,
            "variant" => invalid.tool = Some(agent_tool_call::Tool::CommunicateUpdateToolCall(CommunicateUpdateToolCall::default())),
            "args" => send_body(&mut invalid).args = None,
            "argument-variant" => send_body(&mut invalid).args = Some(SendMessageArgs { message: None }),
            "start" => invalid.started_at_ms = None,
            "result" => send_body(&mut invalid).result = Some(SendMessageResult { result: None }),
            "completion" => invalid.completed_at_ms = Some(60),
            _ => {}
        }
        let sink = recorder.interaction_sink(None);
        sink.on_tool_call(if fault == "phase" { "unexpected" } else { "initial" }, &invalid);
        sink.on_tool_call("initial", &valid);
        sink.on_tool_call("completed", &completed(&valid, "sql-id"));
        assert!(recorder.finish(receipt, &successful_result("sql-id"), 70).is_err(), "{fault}");
        assert!(recorder.take_completed_checked().is_err(), "{fault}");
    }
}

#[test]
fn completed_callbacks_require_unchanged_arguments_start_and_terminal_fields() {
    for fault in ["id", "variant", "args", "argument-variant", "changed-args", "start", "changed-start", "completion", "result", "result-variant"] {
        let recorder = recorder();
        let receipt = register_send(&recorder, "call");
        let initial = initial("call");
        let mut invalid = completed(&initial, "sql-id");
        match fault {
            "id" => invalid.tool_call_id = Some("another-call".into()),
            "variant" => invalid.tool = Some(agent_tool_call::Tool::CommunicateUpdateToolCall(CommunicateUpdateToolCall::default())),
            "args" => send_body(&mut invalid).args = None,
            "argument-variant" => send_body(&mut invalid).args = Some(SendMessageArgs { message: None }),
            "changed-args" => send_body(&mut invalid).args = Some(SendMessageArgs {
                message: Some(send_message_args::Message::Text(SendMessageText { content: "changed".into() })),
            }),
            "start" => invalid.started_at_ms = None,
            "changed-start" => invalid.started_at_ms = Some(41),
            "completion" => invalid.completed_at_ms = None,
            "result" => send_body(&mut invalid).result = None,
            _ => send_body(&mut invalid).result = Some(SendMessageResult { result: None }),
        }
        emit_pair(&recorder, &initial, &invalid);
        assert!(recorder.finish(receipt, &successful_result("sql-id"), 70).is_err(), "{fault}");
        assert!(recorder.take_completed_checked().is_err(), "{fault}");
    }
}

#[test]
fn duplicate_and_out_of_order_callback_phases_refuse_publication() {
    for phases in [vec!["completed"], vec!["initial", "initial"],
        vec!["initial", "completed", "completed"], vec!["initial", "completed", "initial"]]
    {
        let recorder = recorder();
        let receipt = register_send(&recorder, "call");
        let initial = initial("call");
        let completed = completed(&initial, "sql-id");
        let sink = recorder.interaction_sink(None);
        for phase in phases {
            sink.on_tool_call(phase, if phase == "initial" { &initial } else { &completed });
        }
        assert!(recorder.finish(receipt, &successful_result("sql-id"), 70).is_err());
        assert!(recorder.take_completed_checked().is_err());
    }
}

#[test]
fn callbacks_after_finish_cannot_replace_history_or_reopen_a_sealed_recorder() {
    let finished = recorder();
    let completed = record_success(&finished, "call");
    finished.interaction_sink(None).on_tool_call("completed", &completed);
    assert!(finished.take_completed_checked().is_err());

    let sealed = recorder();
    let completed = record_success(&sealed, "call");
    let calls = sealed.take_completed_checked().expect("valid first drain");
    sealed.interaction_sink(None).on_tool_call("completed", &completed);
    assert!(sealed.take_completed_checked().is_err());
    assert!(sealed.begin_run().is_err());
    assert_eq!(calls[0].send_message_wire.as_ref().unwrap().as_bytes(), completed.encode_to_vec());
}

#[test]
fn observed_success_must_match_the_actual_flat_public_result() {
    for public in [
        json!({"sent":false,"timestampMs":50,"messageId":"sql-id"}),
        json!({"timestampMs":50,"messageId":"sql-id"}),
        json!({"sent":true,"timestampMs":51,"messageId":"sql-id"}),
        json!({"sent":true,"timestampMs":"50","messageId":"sql-id"}),
        json!({"sent":true,"messageId":"sql-id"}),
        json!({"sent":true,"timestampMs":50,"messageId":"another-id"}),
        json!({"sent":true,"timestampMs":50,"messageId":null}),
        json!({"sent":true,"timestampMs":50}),
        json!({"result":{"sent":true,"timestampMs":50,"messageId":"sql-id"}}),
    ] {
        let recorder = recorder();
        let receipt = register_send(&recorder, "call");
        let initial = initial("call");
        emit_pair(&recorder, &initial, &completed(&initial, "sql-id"));
        assert!(recorder.finish(receipt, &Ok(public), 70).is_err());
        assert!(recorder.take_completed_checked().is_err());
    }
    for public in [json!({"sent":true,"timestampMs":50}),
        json!({"sent":true,"timestampMs":50,"messageId":0})]
    {
        let recorder = recorder();
        let receipt = register_send(&recorder, "empty-id-call");
        let initial = initial("empty-id-call");
        emit_pair(&recorder, &initial, &completed(&initial, ""));
        assert!(recorder.finish(receipt, &Ok(public), 70).is_err(),
            "empty-ID compatibility requires an actual string or null public messageId");
        assert!(recorder.take_completed_checked().is_err());
    }
}

#[test]
fn terminal_disposition_and_error_text_must_match_the_original_result() {
    for mismatch in ["success-as-error", "error-as-success", "different-error"] {
        let recorder = recorder();
        let receipt = register_send(&recorder, "call");
        let initial = initial("call");
        let mut completed = completed(&initial, "sql-id");
        let public = if mismatch == "success-as-error" {
            Err(ProviderSessionError::Cancelled("cancelled".into()))
        } else {
            send_body(&mut completed).result = Some(SendMessageResult {
                result: Some(send_message_result::Result::Error(SendMessageError { error: "actual producer error".into() })),
            });
            if mismatch == "error-as-success" { successful_result("sql-id") }
            else { Err(ProviderSessionError::Tool("different public error".into())) }
        };
        emit_pair(&recorder, &initial, &completed);
        assert!(recorder.finish(receipt, &public, 70).is_err(), "{mismatch}");
        assert!(recorder.take_completed_checked().is_err());
    }
}

#[test]
fn initial_only_and_unobserved_success_are_never_qualified() {
    for (has_initial, public) in [(false, successful_result("sql-id")),
        (true, successful_result("sql-id")), (true, Err(ProviderSessionError::Tool("failed after initial".into())))]
    {
        let recorder = recorder();
        let receipt = register_send(&recorder, "call");
        if has_initial {
            recorder.interaction_sink(None).on_tool_call("initial", &initial("call"));
        }
        assert!(recorder.finish(receipt, &public, 70).is_err());
        assert!(recorder.take_completed_checked().is_err());
    }
}

#[test]
fn pre_observation_typed_errors_encode_rejection_without_raw_argument_inference() {
    for error in [ProviderSessionError::Tool("tool rejection".into()),
        ProviderSessionError::Cancelled("cancelled before dispatch".into()),
        ProviderSessionError::Authentication("authentication refused".into())]
    {
        let recorder = recorder();
        let receipt = register_send(&recorder, "call");
        let expected = error.to_string();
        let public = Err(error);
        recorder.finish(receipt, &public, 70).expect("known rejection");
        let calls = recorder.take_completed_checked().expect("error history drain");
        assert_eq!(calls[0].result, Err(expected.clone()));
        let mut captured = wire(&calls[0]);
        let send = send_body(&mut captured);
        assert_eq!(send.args, Some(SendMessageArgs { message: None }));
        assert!(matches!(send.result.as_ref().unwrap().result.as_ref(),
            Some(send_message_result::Result::Error(error)) if error.error == expected));
    }
}

#[test]
fn unobserved_unknown_effect_is_never_recast_as_a_known_rejection() {
    let recorder = recorder();
    let receipt = register_send(&recorder, "call");
    let public = Err(ProviderSessionError::ToolOutcomeUnknown("remote disposition unknown".into()));
    assert!(recorder.finish(receipt, &public, 70).is_err());
    assert!(matches!(public, Err(ProviderSessionError::ToolOutcomeUnknown(ref message)) if message == "remote disposition unknown"));
    assert!(recorder.take_completed_checked().is_err());
}

#[test]
fn foreign_thread_callbacks_never_search_other_active_slots_by_provider_id() {
    let recorder = recorder();
    let receipt = register_send(&recorder, "matching-id");
    let initial = initial("matching-id");
    let sink = recorder.interaction_sink(None);
    sink.on_tool_call("initial", &initial);
    let completed = completed(&initial, "sql-id");
    let foreign_sink = Arc::clone(&sink);
    let foreign_completed = completed.clone();
    thread::spawn(move || foreign_sink.on_tool_call("completed", &foreign_completed)).join().unwrap();
    sink.on_tool_call("completed", &completed);
    assert!(recorder.finish(receipt, &successful_result("sql-id"), 70).is_err());
    assert!(recorder.take_completed_checked().is_err());
}

#[test]
fn callbacks_cannot_attach_to_a_non_send_message_call() {
    let recorder = recorder();
    let receipt = recorder.register(&other_tool(), json!({}), "matching-id", 1).unwrap();
    recorder.interaction_sink(None).on_tool_call("initial", &initial("matching-id"));
    assert!(recorder.finish(receipt, &Ok(json!({"ok":true})), 2).is_err());
    assert!(recorder.take_completed_checked().is_err());
}

#[test]
fn receipt_from_another_recorder_faults_both_invocations() {
    let origin = recorder();
    let recipient = recorder();
    let receipt = register_send(&origin, "call");
    assert!(recipient.finish(receipt, &successful_result("sql-id"), 70).is_err());
    assert!(origin.take_completed_checked().is_err());
    assert!(recipient.take_completed_checked().is_err());
}

#[test]
fn receipt_cannot_be_finished_on_a_foreign_worker_thread() {
    let recorder = recorder();
    let receipt = register_send(&recorder, "call");
    let foreign = Arc::clone(&recorder);
    thread::spawn(move || assert!(foreign.finish(receipt, &successful_result("sql-id"), 70).is_err()))
        .join().unwrap();
    assert!(recorder.take_completed_checked().is_err());
}

#[test]
fn abandoned_receipts_and_active_drains_permanently_refuse_publication() {
    let abandoned = recorder();
    let receipt = register_send(&abandoned, "call");
    abandoned.interaction_sink(None).on_tool_call("initial", &initial("call"));
    drop(receipt);
    assert!(abandoned.take_completed_checked().is_err());
    assert!(abandoned.begin_run().is_err());

    let active = recorder();
    let receipt = active.register(&other_tool(), json!({}), "unfinished", 1).unwrap();
    assert!(active.take_completed_checked().is_err());
    assert!(active.finish(receipt, &Ok(json!({"ok":true})), 2).is_err());
    assert!(active.take_completed_checked().is_err());
}

#[test]
fn poisoned_mutex_is_never_recovered_by_capture_finish_begin_or_drain() {
    let recorder = recorder();
    let receipt = register_send(&recorder, "call");
    let poison = Arc::clone(&recorder);
    assert!(thread::spawn(move || {
        let _state = poison.state.lock().unwrap();
        panic!("poison recorder state");
    }).join().is_err());
    recorder.interaction_sink(None).on_tool_call("initial", &initial("call"));
    assert!(recorder.finish(receipt, &successful_result("sql-id"), 70).is_err());
    assert!(recorder.begin_run().is_err());
    assert!(recorder.register(&other_tool(), json!({}), "another", 1).is_err());
    assert!(recorder.take_completed_checked().is_err());
    assert!(recorder.take_completed_checked().is_err());
}

#[test]
fn begin_and_seal_are_one_use_and_a_new_invocation_requires_a_fresh_recorder() {
    let before_begin = Arc::new(ToolCallRecorder::default());
    assert!(before_begin.register(&other_tool(), json!({}), "early", 1).is_err());
    assert!(before_begin.begin_run().is_err());
    let unstarted = ToolCallRecorder::default();
    assert!(unstarted.take_completed_checked().is_err());
    assert!(unstarted.begin_run().is_err());
    let repeated_begin = recorder();
    assert!(repeated_begin.begin_run().is_err());
    assert!(repeated_begin.register(&other_tool(), json!({}), "refused", 1).is_err());
    let first = recorder();
    record_success(&first, "same-id");
    assert_eq!(first.take_completed_checked().unwrap().len(), 1);
    assert!(first.take_completed_checked().is_err());
    assert!(first.begin_run().is_err());
    let fresh = recorder();
    record_success(&fresh, "same-id");
    assert_eq!(fresh.take_completed_checked().unwrap()[0].sequence, 0);
    let empty = recorder();
    assert!(empty.take_completed_checked().unwrap().is_empty());
}

#[test]
fn checked_sequence_exhaustion_never_wraps_or_delegates_an_unregistered_call() {
    let recorder = recorder();
    recorder.state.lock().unwrap().next_sequence = u64::MAX;
    assert!(recorder.register(&other_tool(), json!({}), "overflow", 1).is_err());
    let state = recorder.state.lock().unwrap();
    assert_eq!(state.next_sequence, u64::MAX);
    assert!(state.active.is_empty());
    drop(state);
    assert!(recorder.take_completed_checked().is_err());
}

#[test]
fn non_send_message_calls_keep_existing_results_arguments_and_duplicate_id_behavior() {
    let recorder = recorder();
    let tool = other_tool();
    let first = recorder.register(&tool, json!({"first":1}), "shared-id", 10).unwrap();
    recorder.finish(first, &Ok(json!({"output":"first"})), 11).unwrap();
    let second = recorder.register(&tool, json!({"second":2}), "shared-id", 20).unwrap();
    recorder.finish(second, &Err(ProviderSessionError::ToolOutcomeUnknown("ordinary tool unknown".into())), 21).unwrap();
    let calls = recorder.take_completed_checked().unwrap();
    assert_eq!(calls, vec![
        CompletedRoutedToolCall {
            tool: tool.clone(), args: json!({"first":1}), tool_call_id: "shared-id".into(),
            result: Ok(json!({"output":"first"})), started_at_ms: 10, completed_at_ms: 11, sequence: 0,
            send_message_wire: None,
        },
        CompletedRoutedToolCall {
            tool, args: json!({"second":2}), tool_call_id: "shared-id".into(),
            result: Err("tool execution outcome is unknown: ordinary tool unknown".into()),
            started_at_ms: 20, completed_at_ms: 21, sequence: 1, send_message_wire: None,
        },
    ]);
}

#[test]
fn producer_timestamps_are_preserved_without_wrapper_clock_or_monotonicity_assumptions() {
    let recorder = recorder();
    let receipt = register_send(&recorder, "call");
    let mut initial = initial("call");
    initial.started_at_ms = Some(1000);
    let mut completed = completed(&initial, "sql-id");
    completed.completed_at_ms = Some(2);
    emit_pair(&recorder, &initial, &completed);
    recorder.finish(receipt, &successful_result("sql-id"), 1).unwrap();
    let calls = recorder.take_completed_checked().unwrap();
    assert_eq!(calls[0].send_message_wire.as_ref().unwrap().as_bytes(), completed.encode_to_vec());
    assert_eq!((wire(&calls[0]).started_at_ms, wire(&calls[0]).completed_at_ms), (Some(1000), Some(2)));
}
