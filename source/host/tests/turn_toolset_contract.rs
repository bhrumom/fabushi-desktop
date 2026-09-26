use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::memory::agent_state::SandAgentState;
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::tools::sand_state_tool::{
    SAND_UPDATE_STATE_TOOL_NAME, SandStateWriter,
};
use mahayana_host_runtime::runner::tools::turn_toolset::{
    TurnToolsetDependencies, build_turn_toolset, fence_turn_toolset,
};
use serde_json::{Value, json};

struct BaseBridge;

impl RoutedToolBridge for BaseBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            name: "base_tool".into(),
            provider_identifier: "base".into(),
            tool_name: "base_tool".into(),
            description: None,
            input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Ok(json!({"content":[{"type":"text","text":"base result"}]}))
    }
}

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-turn-toolset-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn turn_toolset_composes_host_state_capability_without_hiding_base_tools() {
    let root = temp_root("state");
    let state = Arc::new(SandAgentState::new(&root, "agent-a").expect("state"));
    let state_writer: Arc<dyn SandStateWriter> = state;
    let bridge = build_turn_toolset(
        Arc::new(BaseBridge),
        TurnToolsetDependencies {
            state_writer: Some(state_writer),
            ..TurnToolsetDependencies::default()
        },
    );

    let tools = bridge.list_tools().expect("tools");
    assert!(tools.iter().any(|tool| tool.name == SAND_UPDATE_STATE_TOOL_NAME));
    assert!(tools.iter().any(|tool| tool.name == "base_tool"));

    let update_state = tools
        .iter()
        .find(|tool| tool.name == SAND_UPDATE_STATE_TOOL_NAME)
        .expect("update_state");
    let result = bridge.call_tool(
        update_state,
        json!({
            "target":"memory",
            "action":"write",
            "fact":"Turn toolset owns the per-turn state route",
            "tier":"log"
        }),
        "tool-state",
    ).expect("state result");
    assert!(result.as_str().is_some_and(|value| value.contains("Remembered")));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn turn_toolset_spotlight_fence_preserves_tool_inventory() {
    let bridge = build_turn_toolset(
        Arc::new(BaseBridge),
        TurnToolsetDependencies::default(),
    );
    let fenced = fence_turn_toolset(bridge, true);
    let tools = fenced.list_tools().expect("tools");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "base_tool");

    let result = fenced
        .call_tool(&tools[0], json!({}), "tool-base")
        .expect("result");
    let text = result
        .get("content")
        .and_then(Value::as_array)
        .and_then(|content| content.get(1))
        .and_then(|part| part.get("text"))
        .and_then(Value::as_str)
        .expect("fenced text");
    assert_eq!(text, "base result");
}
