use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::provider_session::{
    McpCatalogKind, McpCatalogReason, McpCatalogRefusal, ProviderMessage,
    ProviderSessionError, RoutedMcpMetaToolDefinition, RoutedProvider,
    RoutedProviderCheckpoint, RoutedToolDefinition,
};
use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::extensions::session::production_agent_store::ProductionAgentCheckpointSnapshot;
use mahayana_host_runtime::runner::production_agent_checkpoint::AgentStateCheckpointSink;
use mahayana_host_runtime::runner::production_turn_input_projection::{
    ProductionTurnInputProjection, create_production_turn_agent_input_projection,
    create_production_turn_input_projection,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::RoutedProviderCheckpointStore;
use mahayana_host_runtime::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedToolBridge, RunnerRequestContextSnapshot,
};
use mahayana_host_runtime::runner::turn_agent_composition::TurnAgentComposition;
use mahayana_host_runtime::runner::tools::turn_toolset::project_turn_mcp_toolset;
use mahayana_host_runtime::runner::TurnRunOptions;
use serde_json::Value;

#[test]
fn production_turn_projection_binds_stream_identity_and_recovery_shape() {
    let args = serde_json::json!({
        "messageId": "msg-1",
        "recentUserMessages": [
            {"id":"msg-0","text":"older"},
            {"id":"msg-1","text":"hello from durable history"}
        ],
        "isFork": true,
        "attachmentPaths": ["/tmp/a", "/tmp/b"],
        "selectedImages": [{
            "uuid":"image",
            "path":"/tmp/image.png",
            "mimeType":"image/png",
            "data":[1,2,3]
        }],
        "selectedVideos": [{
            "uuid":"video",
            "path":"/tmp/video.mp4",
            "mimeType":"video/mp4",
            "filename":"video.mp4",
            "data":[0,0,0,0,102,116,121,112]
        }],
        "replyContext": {"id":"reply"}
    });
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "hello from the real turn".into(),
    }];

    let projected = create_production_turn_input_projection(
        &args,
        "stream-request-123",
        &messages,
    )
    .expect("projection");

    assert_eq!(
        projected.options.inference_request_id.as_deref(),
        Some("stream-request-123")
    );
    assert_eq!(projected.options.message_id.as_deref(), Some("msg-1"));
    assert_eq!(
        projected.options.recent_message_text.as_deref(),
        Some("hello from durable history")
    );
    assert_eq!(
        projected
            .options
            .recent_user_messages
            .iter()
            .map(|message| (message.id.as_str(), message.text.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("msg-0", "older"),
            ("msg-1", "hello from durable history")
        ]
    );
    assert!(projected.options.is_fork);
    assert_eq!(projected.options.attachment_count, 2);
    assert_eq!(projected.options.image_count, 1);
    assert_eq!(projected.options.video_count, 1);
    assert!(projected.options.has_reply_context);
    assert_eq!(projected.prompt_action.message_id.as_deref(), Some("msg-1"));
    assert_eq!(
        projected.prompt_action.selected_context.selected_images[0]
            .data
            .as_deref(),
        Some(&[1, 2, 3][..])
    );
    assert_eq!(
        projected.prompt_action.selected_context.selected_videos[0]
            .filename
            .as_deref(),
        Some("video.mp4")
    );
}

#[test]
fn production_turn_projection_uses_latest_user_fallback_without_durable_history() {
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "fallback user text".into(),
    }];
    let projected = create_production_turn_input_projection(
        &serde_json::json!({"messageId":"msg-missing"}),
        "stream-fallback",
        &messages,
    )
    .expect("projection");
    assert!(projected.options.recent_user_messages.is_empty());
    assert_eq!(
        projected.options.recent_message_text.as_deref(),
        Some("fallback user text")
    );
    assert!(!projected.options.is_fork);
}

#[test]
fn production_turn_projection_rejects_missing_request_identity() {
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "hello".into(),
    }];
    assert!(
        create_production_turn_input_projection(
            &serde_json::json!({}),
            "   ",
            &messages,
        )
        .is_err()
    );
}

#[test]
fn serialized_turn_request_cannot_supply_canonical_typed_replay() {
    let args = serde_json::json!({
        "typedReplay": [{
            "type": "tool-result",
            "toolCallId": "unadmitted-call",
            "result": {"approved": true}
        }]
    });
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "ordinary user request".into(),
    }];

    let projected = create_production_turn_input_projection(
        &args,
        "stream-untrusted-replay",
        &messages,
    )
    .expect("ordinary request projection");

    assert!(projected.typed_replay.is_none());
    assert_eq!(
        projected.options.recent_message_text.as_deref(),
        Some("ordinary user request")
    );
}


struct CountingBridge {
    list_calls: Arc<AtomicUsize>,
}

impl RoutedToolBridge for CountingBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        self.list_calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![RoutedToolDefinition {
            name: "mcp__demo".into(),
            provider_identifier: "demo".into(),
            tool_name: "demo".into(),
            description: Some("demo tool".into()),
            input_schema: serde_json::json!({"type":"object"}),
        }])
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Ok(Value::Null)
    }
}

struct MemoryProviderCheckpointStore;

impl RoutedProviderCheckpointStore for MemoryProviderCheckpointStore {
    fn persist(
        &self,
        _checkpoint: &RoutedProviderCheckpoint,
    ) -> Result<String, ProviderSessionError> {
        Ok("provider-checkpoint".into())
    }
}

struct SnapshotCheckpointSink {
    base: Mutex<Option<ProductionAgentCheckpointSnapshot>>,
}

impl AgentStateCheckpointSink for SnapshotCheckpointSink {
    fn checked_base_snapshot(
        &self,
    ) -> Result<Option<ProductionAgentCheckpointSnapshot>, ProviderSessionError> {
        Ok(self.base.lock().expect("base state lock").clone())
    }
}

#[test]
fn production_turn_agent_projection_freezes_action_mcp_ack_and_base_state() {
    let list_calls = Arc::new(AtomicUsize::new(0));
    let composition = TurnAgentComposition::new(
        RoutedProvider::OpenRouter,
        Arc::new(CountingBridge {
            list_calls: Arc::clone(&list_calls),
        }),
        RunnerRequestContextSnapshot {
            context: HostRequestContext {
                os_version: "test".into(),
                shell: None,
                time_zone: Some("UTC".into()),
                transcripts_folder: "/tmp/transcripts".into(),
                user_full_name: None,
            },
            rules: None,
        },
        RoutedProviderCancellation::default(),
        Arc::new(MemoryProviderCheckpointStore),
    );
    let lifecycle_messages = vec![ProviderMessage {
        role: "user".into(),
        content: "hello".into(),
    }];
    let provider_messages = vec![
        ProviderMessage {
            role: "system".into(),
            content: "system".into(),
        },
        lifecycle_messages[0].clone(),
    ];
    let checkpoint_sink: Arc<dyn AgentStateCheckpointSink> = Arc::new(
        SnapshotCheckpointSink {
            base: Mutex::new(Some(ProductionAgentCheckpointSnapshot {
                root_blob_id: vec![11; 32],
                checkpoint_bytes: vec![1, 2, 3, 4],
            })),
        },
    );
    let mut output = Vec::<String>::new();
    let mut emit_update = |delta: &str, _accumulated: &str| output.push(delta.to_string());
    let turn_input = ProductionTurnInputProjection {
        options: TurnRunOptions::default(),
        ack_token: Some("ack-1".into()),
        prompt_action: Default::default(),
        typed_replay: None,
    };

    let projected = create_production_turn_agent_input_projection(
        &composition,
        Some(&checkpoint_sink),
        &lifecycle_messages,
        &provider_messages,
        &turn_input,
        &mut emit_update,
    )
    .expect("production Agent input projection");

    assert_eq!(projected.action.lifecycle_messages, lifecycle_messages);
    assert_eq!(projected.action.provider_messages, provider_messages);
    assert_eq!(projected.action.prompt_action, turn_input.prompt_action);
    assert_eq!(projected.mcp_tools.len(), 1);
    assert_eq!(projected.mcp_tools[0].name, "mcp__demo");
    assert_eq!(projected.expected_base, Some(ProductionAgentCheckpointSnapshot {
        root_blob_id: vec![11; 32],
        checkpoint_bytes: vec![1, 2, 3, 4],
    }));
    assert_eq!(projected.ack_token.as_deref(), Some("ack-1"));
    assert_eq!(list_calls.load(Ordering::SeqCst), 1);
    assert!(!projected.cancel_this_run.is_cancelled());
    (projected.emit_update)("delta", "delta");
    drop(projected);
    assert_eq!(output, vec!["delta"]);

    let frozen = composition
        .with_projected_mcp_tools(vec![RoutedToolDefinition {
            name: "mcp__frozen".into(),
            provider_identifier: "demo".into(),
            tool_name: "frozen".into(),
            description: None,
            input_schema: serde_json::json!({"type":"object"}),
        }])
        .snapshot_mcp_tools()
        .expect("frozen MCP snapshot");
    assert_eq!(frozen[0].name, "mcp__frozen");
    assert_eq!(list_calls.load(Ordering::SeqCst), 1);
}


struct FailingDiscoveryBridge {
    list_calls: Arc<AtomicUsize>,
}

impl RoutedToolBridge for FailingDiscoveryBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        self.list_calls.fetch_add(1, Ordering::SeqCst);
        Err(ProviderSessionError::Tool("discovery unavailable".into()))
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Ok(Value::Null)
    }
}

#[test]
fn production_turn_agent_projection_degrades_failed_mcp_discovery_to_empty_snapshot() {
    let list_calls = Arc::new(AtomicUsize::new(0));
    let composition = TurnAgentComposition::new(
        RoutedProvider::OpenRouter,
        Arc::new(FailingDiscoveryBridge {
            list_calls: Arc::clone(&list_calls),
        }),
        RunnerRequestContextSnapshot {
            context: HostRequestContext {
                os_version: "test".into(),
                shell: None,
                time_zone: Some("UTC".into()),
                transcripts_folder: "/tmp/transcripts".into(),
                user_full_name: None,
            },
            rules: None,
        },
        RoutedProviderCancellation::default(),
        Arc::new(MemoryProviderCheckpointStore),
    );
    let lifecycle_messages = vec![ProviderMessage {
        role: "user".into(),
        content: "continue without MCP".into(),
    }];
    let provider_messages = lifecycle_messages.clone();
    let checkpoint_sink: Arc<dyn AgentStateCheckpointSink> = Arc::new(
        SnapshotCheckpointSink {
            base: Mutex::new(Some(ProductionAgentCheckpointSnapshot {
                root_blob_id: vec![12; 32],
                checkpoint_bytes: vec![9, 8, 7],
            })),
        },
    );
    let turn_input = ProductionTurnInputProjection {
        options: TurnRunOptions::default(),
        ack_token: Some("ack-fallback".into()),
        prompt_action: Default::default(),
        typed_replay: None,
    };
    let mut emit_update = |_delta: &str, _accumulated: &str| {};

    let projected = create_production_turn_agent_input_projection(
        &composition,
        Some(&checkpoint_sink),
        &lifecycle_messages,
        &provider_messages,
        &turn_input,
        &mut emit_update,
    )
    .expect("MCP discovery failure must not abort the turn projection");

    assert_eq!(list_calls.load(Ordering::SeqCst), 1);
    assert!(projected.mcp_tools.is_empty());
    assert_eq!(projected.action.lifecycle_messages, lifecycle_messages);
    assert_eq!(projected.action.provider_messages, provider_messages);
    assert_eq!(projected.expected_base, Some(ProductionAgentCheckpointSnapshot {
        root_blob_id: vec![12; 32],
        checkpoint_bytes: vec![9, 8, 7],
    }));
    assert_eq!(projected.ack_token.as_deref(), Some("ack-fallback"));
    assert!(!projected.cancel_this_run.is_cancelled());
}


#[derive(Default)]
struct CatalogProjectionBridge {
    tools: Vec<RoutedToolDefinition>,
    meta_tools: Vec<RoutedMcpMetaToolDefinition>,
    refusal: Option<McpCatalogRefusal>,
    tool_lists: AtomicUsize,
    meta_lists: AtomicUsize,
    tool_calls: AtomicUsize,
}

impl RoutedToolBridge for CatalogProjectionBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        self.tool_lists.fetch_add(1, Ordering::SeqCst);
        match self.refusal {
            Some(refusal) => Err(ProviderSessionError::McpCatalog(refusal)),
            None => Ok(self.tools.clone()),
        }
    }

    fn list_mcp_meta_tools(
        &self,
    ) -> Result<Vec<RoutedMcpMetaToolDefinition>, ProviderSessionError> {
        self.meta_lists.fetch_add(1, Ordering::SeqCst);
        match self.refusal {
            Some(refusal) => Err(ProviderSessionError::McpCatalog(refusal)),
            None => Ok(self.meta_tools.clone()),
        }
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        self.tool_calls.fetch_add(1, Ordering::SeqCst);
        Err(ProviderSessionError::Tool("unexpected projection dispatch".into()))
    }
}

fn catalog_projection_composition(bridge: Arc<dyn RoutedToolBridge>) -> TurnAgentComposition {
    TurnAgentComposition::new(
        RoutedProvider::OpenRouter,
        bridge,
        RunnerRequestContextSnapshot {
            context: HostRequestContext {
                os_version: "test".into(),
                shell: None,
                time_zone: Some("UTC".into()),
                transcripts_folder: "/tmp/transcripts".into(),
                user_full_name: None,
            },
            rules: None,
        },
        RoutedProviderCancellation::default(),
        Arc::new(MemoryProviderCheckpointStore),
    )
}

#[test]
fn production_turn_agent_projection_keeps_root_presence_and_identity_frozen() {
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "capture the canonical base".into(),
    }];
    let turn_input = create_production_turn_input_projection(
        &serde_json::json!({}), "snapshot-presence", &messages,
    ).expect("turn input");
    for snapshot in [
        None,
        Some(ProductionAgentCheckpointSnapshot {
            root_blob_id: vec![21; 32], checkpoint_bytes: Vec::new(),
        }),
        Some(ProductionAgentCheckpointSnapshot {
            root_blob_id: vec![22; 32], checkpoint_bytes: Vec::new(),
        }),
    ] {
        let composition = catalog_projection_composition(Arc::new(
            CatalogProjectionBridge::default(),
        ));
        let concrete_sink = Arc::new(SnapshotCheckpointSink {
            base: Mutex::new(snapshot.clone()),
        });
        let checkpoint_sink: Arc<dyn AgentStateCheckpointSink> = concrete_sink.clone();
        let mut emit_update = |_delta: &str, _accumulated: &str| {};
        let projected = create_production_turn_agent_input_projection(
            &composition, Some(&checkpoint_sink), &messages, &messages,
            &turn_input, &mut emit_update,
        ).expect("explicit checked snapshot");
        *concrete_sink.base.lock().expect("controlled snapshot") = Some(
            ProductionAgentCheckpointSnapshot {
                root_blob_id: vec![23; 32], checkpoint_bytes: vec![1, 2],
            },
        );
        assert_eq!(projected.expected_base, snapshot);
    }
}

#[test]
fn production_turn_agent_projection_refuses_unsupported_snapshot_capability() {
    struct UnsupportedCheckpointSink;
    impl AgentStateCheckpointSink for UnsupportedCheckpointSink {}

    let checkpoint_sink: Arc<dyn AgentStateCheckpointSink> = Arc::new(
        UnsupportedCheckpointSink,
    );
    assert!(matches!(checkpoint_sink.base_state_bytes(), Err(ProviderSessionError::Protocol(_))));
    let composition = catalog_projection_composition(Arc::new(
        CatalogProjectionBridge::default(),
    ));
    let messages = vec![ProviderMessage {
        role: "user".into(), content: "do not infer an absent root".into(),
    }];
    let turn_input = create_production_turn_input_projection(
        &serde_json::json!({}), "unsupported-snapshot", &messages,
    ).expect("turn input");
    let mut emit_update = |_delta: &str, _accumulated: &str| {};
    let result = create_production_turn_agent_input_projection(
        &composition, Some(&checkpoint_sink), &messages, &messages,
        &turn_input, &mut emit_update,
    );
    assert!(matches!(result, Err(ProviderSessionError::Protocol(_))));
    let transient = create_production_turn_agent_input_projection(
        &composition, None, &messages, &messages, &turn_input, &mut emit_update,
    ).expect("explicit transient turn remains supported");
    assert_eq!(transient.expected_base, None);
}

fn permissive_catalog() -> Vec<RoutedMcpMetaToolDefinition> {
    let first = RoutedToolDefinition {
        name: "public_z".into(),
        provider_identifier: "z-provider".into(),
        tool_name: "raw-z".into(),
        description: Some(String::new()),
        input_schema: Value::Null,
    };
    let tools = vec![
        first.clone(),
        RoutedToolDefinition {
            name: "public_a".into(),
            provider_identifier: "a-provider".into(),
            tool_name: "raw-a".into(),
            description: None,
            input_schema: serde_json::json!([1, {"type":"string"}]),
        },
        first,
        RoutedToolDefinition {
            name: String::new(),
            provider_identifier: String::new(),
            tool_name: String::new(),
            description: None,
            input_schema: serde_json::json!(false),
        },
        RoutedToolDefinition {
            name: "public_a_second".into(),
            provider_identifier: "a-provider".into(),
            tool_name: String::new(),
            description: Some("  description  ".into()),
            input_schema: serde_json::json!(17),
        },
        RoutedToolDefinition {
            name: "  public_z_last  ".into(),
            provider_identifier: "z-provider".into(),
            tool_name: "  raw-z-last  ".into(),
            description: None,
            input_schema: serde_json::json!("opaque schema"),
        },
    ];
    let mut meta: Vec<_> = tools.into_iter().map(RoutedMcpMetaToolDefinition::from).collect();
    meta[0].plugin = Some(serde_json::json!(false));
    meta[0].marketplace = Some(serde_json::json!(["market", 7]));
    meta[0].plugin_id = Some(String::new());
    meta[0].marketplace_id = Some("marketplace-id".into());
    meta
}

#[test]
fn production_turn_agent_projection_preserves_typed_catalog_refusal() {
    for kind in [McpCatalogKind::Tools, McpCatalogKind::MetaTools] {
        let refusal = McpCatalogRefusal {
            kind,
            row: Some(2),
            reason: McpCatalogReason::ProviderIdentifierMustBeString,
        };
        let bridge = Arc::new(CatalogProjectionBridge {
            refusal: Some(refusal),
            ..CatalogProjectionBridge::default()
        });
        let composition = catalog_projection_composition(bridge.clone());
        let messages = vec![ProviderMessage { role: "user".into(), content: "catalog turn".into() }];
        let turn_input = create_production_turn_input_projection(
            &serde_json::json!({}), "catalog-refusal", &messages,
        ).expect("turn input");
        let mut updates = Vec::new();
        let mut emit_update = |delta: &str, _accumulated: &str| updates.push(delta.to_string());
        let error = create_production_turn_agent_input_projection(
            &composition, None, &messages, &messages, &turn_input, &mut emit_update,
        ).err().expect("structural catalog refusal");
        assert!(matches!(
            error,
            ProviderSessionError::McpCatalog(actual) if actual == refusal
        ), "typed catalog refusal must not become an empty snapshot");
        assert_eq!(bridge.meta_lists.load(Ordering::SeqCst), 1);
        assert_eq!(bridge.tool_lists.load(Ordering::SeqCst), 0);
        assert_eq!(bridge.tool_calls.load(Ordering::SeqCst), 0);
        assert!(updates.is_empty());
    }
}

#[test]
fn composition_and_live_toolset_projection_preserve_typed_catalog_refusal() {
    for kind in [McpCatalogKind::Tools, McpCatalogKind::MetaTools] {
        let refusal = McpCatalogRefusal {
            kind,
            row: None,
            reason: McpCatalogReason::ExpectedArray,
        };
        let bridge = Arc::new(CatalogProjectionBridge {
            refusal: Some(refusal),
            ..CatalogProjectionBridge::default()
        });
        let composition = catalog_projection_composition(bridge.clone());
        let snapshot = match kind {
            McpCatalogKind::Tools => composition.snapshot_mcp_tools().map(|_| ()),
            McpCatalogKind::MetaTools => composition.snapshot_mcp_meta_tools().map(|_| ()),
        };
        assert!(matches!(
            snapshot,
            Err(ProviderSessionError::McpCatalog(actual)) if actual == refusal
        ), "composition must retain the typed discovery error");
        let live = project_turn_mcp_toolset(
            bridge.clone(), None, None, kind == McpCatalogKind::MetaTools,
        ).and_then(|projected| projected.list_tools());
        assert!(matches!(
            live,
            Err(ProviderSessionError::McpCatalog(actual)) if actual == refusal
        ), "live canonical projection must retain the typed discovery error");
        let expected_lists = match kind {
            McpCatalogKind::Tools => (2, 0),
            McpCatalogKind::MetaTools => (0, 2),
        };
        assert_eq!(
            (bridge.tool_lists.load(Ordering::SeqCst), bridge.meta_lists.load(Ordering::SeqCst)),
            expected_lists,
        );
        assert_eq!(bridge.tool_calls.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn live_mcp_projection_preserves_first_seen_grouping_and_complete_descriptors() {
    let meta_tools = permissive_catalog();
    let tools: Vec<_> = meta_tools.iter().map(|meta| meta.tool.clone()).collect();
    // Explicit oracle: providers appear z, a, blank, and rows retain their
    // original order inside each group, including the duplicate z row.
    let expected: Vec<_> = [0, 2, 5, 1, 4, 3].iter().map(|&index| tools[index].clone()).collect();
    let bridge = Arc::new(CatalogProjectionBridge {
        tools: tools.clone(),
        meta_tools: meta_tools.clone(),
        ..CatalogProjectionBridge::default()
    });
    let composition = catalog_projection_composition(bridge.clone());
    assert_eq!(composition.snapshot_mcp_tools().expect("canonical snapshot"), expected);
    let live = project_turn_mcp_toolset(bridge.clone(), None, None, false)
        .expect("live projection");
    assert_eq!(live.list_tools().expect("live catalog"), expected);

    let messages = vec![ProviderMessage { role: "user".into(), content: "accepted catalog".into() }];
    let turn_input = create_production_turn_input_projection(
        &serde_json::json!({}), "accepted-catalog", &messages,
    ).expect("turn input");
    let mut emit_update = |_delta: &str, _accumulated: &str| {};
    let projected = create_production_turn_agent_input_projection(
        &composition, None, &messages, &messages, &turn_input, &mut emit_update,
    ).expect("permissive catalog remains accepted");
    assert_eq!(projected.mcp_tools, expected);
    assert_eq!(projected.mcp_meta_tools, meta_tools);
    assert_eq!(bridge.tool_lists.load(Ordering::SeqCst), 2);
    assert_eq!(bridge.meta_lists.load(Ordering::SeqCst), 1);
    assert_eq!(bridge.tool_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn frozen_mcp_projection_keeps_exact_vectors_without_live_discovery() {
    for meta_tools in [Vec::new(), permissive_catalog()] {
        let tools: Vec<_> = meta_tools.iter().map(|meta| meta.tool.clone()).collect();
        let bridge = Arc::new(CatalogProjectionBridge {
            refusal: Some(McpCatalogRefusal {
                kind: McpCatalogKind::MetaTools,
                row: Some(0),
                reason: McpCatalogReason::ToolNameMustBeString,
            }),
            ..CatalogProjectionBridge::default()
        });
        let composition = catalog_projection_composition(bridge.clone())
            .with_projected_mcp_tools(tools.clone())
            .with_projected_mcp_meta_tools(meta_tools.clone());
        assert_eq!(composition.snapshot_mcp_tools().expect("frozen tools"), tools);
        assert_eq!(composition.snapshot_mcp_meta_tools().expect("frozen meta tools"), meta_tools);
        assert_eq!(
            composition.snapshot_mcp_projection().expect("frozen projection"),
            (tools.clone(), meta_tools.clone()),
        );
        let frozen = project_turn_mcp_toolset(
            bridge.clone(), Some(Arc::new(tools.clone())), None, false,
        ).expect("frozen toolset");
        assert_eq!(frozen.list_tools().expect("frozen tool inventory"), tools);
        let frozen_meta = project_turn_mcp_toolset(
            bridge.clone(), Some(Arc::new(tools)), Some(Arc::new(meta_tools.clone())), true,
        ).expect("frozen meta toolset");
        assert_eq!(frozen_meta.list_mcp_meta_tools().expect("frozen metadata"), meta_tools);
        assert_eq!(bridge.tool_lists.load(Ordering::SeqCst), 0);
        assert_eq!(bridge.meta_lists.load(Ordering::SeqCst), 0);
        assert_eq!(bridge.tool_calls.load(Ordering::SeqCst), 0);
    }
}
