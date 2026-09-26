use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::turn_observation::{
    ObservedRoutedToolBridge, RECENT_ACTIVITY_CAP, ToolActivity,
    TurnObservation,
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
}
