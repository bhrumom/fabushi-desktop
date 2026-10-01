use std::sync::{Arc, Mutex};
use std::time::Duration;

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::turn_observation::{
    McpObservedRoutedToolBridge, ObservedRoutedToolBridge, RECENT_ACTIVITY_CAP, ToolActivity,
    ToolCallTelemetryEvent, TurnObservation,
};
use serde_json::{Value, json};

#[derive(Default)]
struct Delegate;

impl RoutedToolBridge for Delegate {
    fn list_tools(
        &self,
    ) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![])
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        if args.get("fail").and_then(Value::as_bool) == Some(true) {
            Err(ProviderSessionError::Tool("boom".into()))
        } else {
            Ok(json!({"status":"done","value":"ok"}))
        }
    }
}

fn tool(name: &str) -> RoutedToolDefinition {
    RoutedToolDefinition {
        name: name.into(),
        provider_identifier: "test".into(),
        tool_name: name.into(),
        description: None,
        input_schema: json!({"type":"object"}),
    }
}

#[test]
fn activity_dedupes_caps_and_counts_terminal_tool_calls() {
    let mut observation = TurnObservation::new("agent-a", None);
    observation.record_tool_activity(ToolActivity {
        status: "pending".into(),
        name: "Read".into(),
        summary: None,
    });
    observation.record_tool_activity(ToolActivity {
        status: "pending".into(),
        name: "Read".into(),
        summary: None,
    });
    for index in 0..(RECENT_ACTIVITY_CAP + 5) {
        observation.record_tool_activity(ToolActivity {
            status: "done".into(),
            name: format!("Tool{index}"),
            summary: Some("ok".into()),
        });
    }
    assert_eq!(observation.recent_activity().len(), RECENT_ACTIVITY_CAP);
    assert_eq!(
        observation.observed_tool_call_count(),
        (RECENT_ACTIVITY_CAP + 5) as u64
    );
}

#[test]
fn first_token_is_one_shot_and_snapshot_uses_turn_start() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink_events = Arc::clone(&events);
    let mut observation = TurnObservation::new(
        "agent-a",
        Some(Arc::new(move |event| {
            sink_events.lock().unwrap().push(event);
        })),
    );
    let first_token_events = Arc::new(Mutex::new(Vec::new()));
    let first_token_sink_events = Arc::clone(&first_token_events);
    observation.set_first_token_handler(Arc::new(move |event| {
        first_token_sink_events.lock().unwrap().push(event);
    }));
    observation.turn_started(100);
    assert!(observation.observe_first_token(
        "text",
        Some(10.0),
        Some(25.0),
        Some("model-a"),
        false,
    ));
    assert!(!observation.observe_first_token(
        "text",
        Some(10.0),
        Some(30.0),
        Some("model-a"),
        false,
    ));
    assert_eq!(observation.snapshot(175).elapsed_ms, 75);
    let first_token_events = first_token_events.lock().unwrap();
    assert_eq!(first_token_events.len(), 1);
    assert_eq!(first_token_events[0]["chunkType"], "text");
    assert_eq!(first_token_events[0]["ttftMs"], 15.0);
    assert_eq!(first_token_events[0]["skew"], false);
    assert_eq!(first_token_events[0]["modelId"], "model-a");
    drop(first_token_events);
    assert_eq!(
        events
            .lock()
            .unwrap()
            .iter()
            .filter(|event| event["type"] == "first-token")
            .count(),
        1
    );
}

#[test]
fn observed_bridge_wraps_success_failure_and_await_lifecycle() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink_events = Arc::clone(&events);
    let observation = TurnObservation::shared(
        "agent-a",
        Some(Arc::new(move |event| {
            sink_events.lock().unwrap().push(event);
        })),
    );
    let await_events = Arc::new(Mutex::new(Vec::new()));
    let await_sink_events = Arc::clone(&await_events);
    observation
        .lock()
        .unwrap()
        .set_turn_await_handler(Arc::new(move |event| {
            await_sink_events.lock().unwrap().push(event);
        }));
    let bridge = ObservedRoutedToolBridge::new(
        Arc::new(Delegate),
        Arc::clone(&observation),
    );

    bridge
        .call_tool(
            &tool("AwaitShell"),
            json!({"block_until_ms":123}),
            "call-1",
        )
        .expect("await tool");
    assert!(bridge
        .call_tool(&tool("Read"), json!({"fail":true}), "call-2")
        .is_err());

    let observation = observation.lock().unwrap();
    assert_eq!(observation.observed_tool_call_count(), 2);
    assert!(observation
        .recent_activity()
        .iter()
        .any(|line| line == "[failed] Read"));
    drop(observation);

    let events = events.lock().unwrap();
    assert!(events.iter().any(|event| event["type"] == "turn-await"));
    assert!(events.iter().any(|event| {
        event["type"] == "tool-completed" && event["failed"] == true
    }));
    drop(events);

    let await_events = await_events.lock().unwrap();
    assert_eq!(await_events.len(), 1);
    assert_eq!(await_events[0]["awaitIndex"], 1);
    assert_eq!(await_events[0]["blockUntilMs"], 123);
    assert_eq!(await_events[0]["outcome"], "completed");
}


#[test]
fn send_dispatch_handler_receives_the_sanitized_shipping_payload() {
    let dispatched = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&dispatched);
    let mut observation = TurnObservation::new("agent-dispatch", None);
    observation.set_send_dispatch_handler(Arc::new(move |event| {
        sink.lock().unwrap().push(event);
    }));
    observation.observe_send_dispatch(100.0, 115.4, Some(1_000.0), 1_025.2, false, "model-a");
    let dispatched = dispatched.lock().unwrap();
    assert_eq!(dispatched.len(), 1);
    assert_eq!(dispatched[0]["type"], "send-dispatch");
    assert_eq!(dispatched[0]["agentId"], "agent-dispatch");
    assert_eq!(dispatched[0]["hostDispatchMs"], 15);
    assert_eq!(dispatched[0]["dispatchMs"], 25.0);
    assert_eq!(dispatched[0]["skew"], false);
    assert_eq!(dispatched[0]["isFork"], false);
    assert_eq!(dispatched[0]["modelId"], "model-a");
}


#[test]
fn observed_bridge_emits_frozen_started_and_error_tool_call_facts() {
    let observation = TurnObservation::shared("agent-tool", None);
    let telemetry = Arc::new(Mutex::new(Vec::<ToolCallTelemetryEvent>::new()));
    let telemetry_sink = Arc::clone(&telemetry);
    {
        let mut observation = observation.lock().unwrap();
        observation.set_request_id(Some("request-tool".into()));
        observation.set_tool_call_telemetry_handler(Arc::new(move |event| {
            telemetry_sink.lock().unwrap().push(event);
        }));
    }
    let bridge = ObservedRoutedToolBridge::new(Arc::new(Delegate), Arc::clone(&observation));

    bridge
        .call_tool(&tool("Shell"), json!({}), "call-started")
        .expect("dual-surface tool");
    assert!(
        bridge
            .call_tool(&tool("Read"), json!({"fail":true}), "call-failed")
            .is_err()
    );

    let telemetry = telemetry.lock().unwrap();
    assert!(telemetry.iter().any(|event| matches!(
        event,
        ToolCallTelemetryEvent::Started {
            conversation_id,
            request_id,
            tool_name,
            tool_call_id,
            surface,
        } if conversation_id == "agent-tool"
            && request_id.as_deref() == Some("request-tool")
            && tool_name == "Shell"
            && tool_call_id == "call-started"
            && surface == "box"
    )));
    assert!(telemetry.iter().any(|event| matches!(
        event,
        ToolCallTelemetryEvent::Error {
            conversation_id,
            request_id,
            tool_name,
            tool_call_id,
            error_class,
            connector,
            ..
        } if conversation_id == "agent-tool"
            && request_id.as_deref() == Some("request-tool")
            && tool_name == "Read"
            && tool_call_id == "call-failed"
            && error_class == "task_error_result"
            && connector == "unknown"
    )));
    assert_eq!(
        telemetry
            .iter()
            .filter(|event| matches!(event, ToolCallTelemetryEvent::Started { .. }))
            .count(),
        2,
        "both Shell and Read are frozen dual-surface box tools"
    );
}


#[test]
fn mcp_exec_observation_reports_stall_then_error_settlement() {
    let mut observation = TurnObservation::new("agent-mcp", None);
    observation.set_request_id(Some("request-mcp".into()));
    let telemetry = Arc::new(Mutex::new(Vec::<ToolCallTelemetryEvent>::new()));
    let telemetry_sink = Arc::clone(&telemetry);
    observation.set_tool_call_telemetry_handler(Arc::new(move |event| {
        telemetry_sink.lock().unwrap().push(event);
    }));

    let guard = observation
        .begin_mcp_exec_observation_with_threshold(
            "mcp-call-1",
            "github",
            Duration::from_millis(5),
        )
        .expect("mcp observation");
    std::thread::sleep(Duration::from_millis(20));

    {
        let telemetry = telemetry.lock().unwrap();
        assert!(telemetry.iter().any(|event| matches!(
            event,
            ToolCallTelemetryEvent::Stalled {
                conversation_id,
                request_id,
                tool_name,
                tool_call_id,
                connector,
                ..
            } if conversation_id == "agent-mcp"
                && request_id.as_deref() == Some("request-mcp")
                && tool_name == "mcpToolCall"
                && tool_call_id == "mcp-call-1"
                && connector == "github"
        )));
    }

    guard.settle(Some("ConnectError.Unavailable"));
    let telemetry = telemetry.lock().unwrap();
    assert!(telemetry.iter().any(|event| matches!(
        event,
        ToolCallTelemetryEvent::Error {
            conversation_id,
            request_id,
            tool_name,
            tool_call_id,
            error_class,
            connector,
            ..
        } if conversation_id == "agent-mcp"
            && request_id.as_deref() == Some("request-mcp")
            && tool_name == "mcpToolCall"
            && tool_call_id == "mcp-call-1"
            && error_class == "ConnectError.Unavailable"
            && connector == "github"
    )));
}


#[test]
fn routed_mcp_failure_reports_one_specialized_error_not_generic_duplicate() {
    let observation = TurnObservation::shared("agent-mcp-dedupe", None);
    let telemetry = Arc::new(Mutex::new(Vec::<ToolCallTelemetryEvent>::new()));
    let telemetry_sink = Arc::clone(&telemetry);
    observation
        .lock()
        .unwrap()
        .set_tool_call_telemetry_handler(Arc::new(move |event| {
            telemetry_sink.lock().unwrap().push(event);
        }));

    let base: Arc<dyn RoutedToolBridge> = Arc::new(Delegate);
    let mcp: Arc<dyn RoutedToolBridge> =
        Arc::new(McpObservedRoutedToolBridge::new(base, Arc::clone(&observation)));
    let observed = ObservedRoutedToolBridge::new(mcp, Arc::clone(&observation));

    assert!(
        observed
            .call_tool(&tool("RemoteMcpTool"), json!({"fail":true}), "mcp-failed")
            .is_err()
    );

    let telemetry = telemetry.lock().unwrap();
    let errors = telemetry
        .iter()
        .filter(|event| matches!(event, ToolCallTelemetryEvent::Error { .. }))
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0],
        ToolCallTelemetryEvent::Error {
            tool_call_id,
            error_class,
            ..
        } if tool_call_id == "mcp-failed" && error_class == "mcp_error_result"
    ));
}
