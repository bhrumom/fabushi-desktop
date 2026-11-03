use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::provider_session::{
    OpenRouterCheckpoint, ProviderSessionError, RoutedProviderCheckpoint, RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::transcript::runner_registry::{
    RUN_DIRECT_USER_INTERRUPT_REASON, RUN_WATCHDOG_INTERRUPT_REASON, TranscriptRunnerRegistry,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::RoutedProviderCheckpointStore;
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, ROUTED_MCP_PROTOCOL_VERSION,
    RoutedProviderCancellation, RoutedProviderTaskRegistry, RoutedToolBridge, start_routed_mcp_server,
};
use serde_json::{Value, json};

struct FakeBridge;

impl RoutedToolBridge for FakeBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            name: "github_search".into(),
            provider_identifier: "github".into(),
            tool_name: "search".into(),
            description: Some("Search GitHub".into()),
            input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        assert_eq!(tool.provider_identifier, "github");
        assert_eq!(args["q"], "rust");
        assert!(!tool_call_id.is_empty());
        Ok(json!({"content":[{"type":"text","text":"tool-result"}]}))
    }
}

fn post_raw(url: &str, body: Value) -> String {
    let without_scheme = url.strip_prefix("http://").expect("http");
    let (authority, path) = without_scheme.split_once('/').expect("path");
    let mut stream = TcpStream::connect(authority).expect("connect");
    stream.set_read_timeout(Some(std::time::Duration::from_secs(10))).expect("read deadline");
    stream.set_write_timeout(Some(std::time::Duration::from_secs(10))).expect("write deadline");
    let payload = serde_json::to_vec(&body).expect("payload");
    write!(
        stream,
        "POST /{path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    )
    .expect("headers");
    stream.write_all(&payload).expect("body");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("response");
    response
}

fn post(url: &str, body: Value) -> Value {
    let response = post_raw(url, body);
    let body = response.split_once("\r\n\r\n").expect("http body").1;
    serde_json::from_str(body).expect("json")
}

#[test]
fn runner_owned_mcp_bridge_lists_and_executes_host_tools() {
    let mut server = start_routed_mcp_server(Arc::new(FakeBridge)).expect("server");
    let initialized = post(
        server.url(),
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    assert_eq!(
        initialized["result"]["protocolVersion"],
        ROUTED_MCP_PROTOCOL_VERSION
    );

    let tools = post(
        server.url(),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    assert_eq!(tools["result"]["tools"][0]["name"], "github_search");

    let called = post(
        server.url(),
        json!({
            "jsonrpc":"2.0",
            "id":3,
            "method":"tools/call",
            "params":{"name":"github_search","arguments":{"q":"rust"}}
        }),
    );
    assert_eq!(called["result"]["content"][0]["text"], "tool-result");
    server.close();
}

#[test]
fn runner_provider_registry_cancels_by_stream_and_retires_finished_runs() {
    let registry = RoutedProviderTaskRegistry::default();
    let cancellation = registry.register("stream-cancel").expect("register");
    assert_eq!(registry.active_count(), 1);
    assert!(!cancellation.is_cancelled());
    assert!(registry.cancel("stream-cancel", "user cancelled"));
    assert!(cancellation.is_cancelled());
    assert_eq!(cancellation.reason().as_deref(), Some("user cancelled"));
    assert_eq!(registry.active_count(), 1, "a request does not retire the worker");
    assert!(!registry.cancel("stream-cancel", "duplicate request"));
    assert_eq!(cancellation.reason().as_deref(), Some("user cancelled"));
    assert!(!registry.cancel("missing-stream", "ignored"));
    registry.finish("stream-cancel");
    assert_eq!(registry.active_count(), 0);
    assert!(!registry.cancel("stream-cancel", "late request"));
    assert_eq!(cancellation.reason().as_deref(), Some("user cancelled"));
}

#[test]
fn runner_provider_registry_preserves_supplied_cancellation_and_registration_identity() {
    let registry = RoutedProviderTaskRegistry::default();
    let admitted = RoutedProviderCancellation::default();
    assert!(admitted.cancel("cancelled before registration"));
    let attached = registry
        .register_for_agent_with_cancellation("agent-a", "shared-stream", admitted.clone())
        .expect("attach the cancelled admitted token");
    assert!(attached.is_cancelled());
    assert_eq!(attached.reason().as_deref(), Some("cancelled before registration"));
    assert!(!registry.cancel("shared-stream", "late reason"));
    assert_eq!(registry.active_count(), 1, "attachment and cancellation retain the task");

    let rejected = RoutedProviderCancellation::default();
    assert!(registry
        .register_for_agent_with_cancellation("agent-b", "shared-stream", rejected.clone())
        .is_err());
    assert_eq!(registry.agent_id_for_stream("shared-stream").as_deref(), Some("agent-a"));
    assert_eq!(registry.cancel_agent("agent-b", "foreign cancellation"), 0);
    assert!(!registry.cancel_for_agent("agent-b", "shared-stream", "foreign stream cancellation"));
    assert!(!rejected.is_cancelled());
    assert_eq!(admitted.reason().as_deref(), Some("cancelled before registration"));
    assert_eq!(registry.active_count(), 1);

    registry.finish("shared-stream");
    assert_eq!(registry.active_count(), 0);
    let replacement = RoutedProviderCancellation::default();
    let attached_replacement = registry
        .register_for_agent_with_cancellation("agent-b", "shared-stream", replacement.clone())
        .expect("a retired stream can attach a new token");
    assert!(!attached_replacement.is_cancelled());
    assert!(!admitted.cancel("old owner retries cancellation"));
    assert!(!registry.cancel_for_agent("agent-a", "shared-stream", "old agent retries cancellation"));
    assert!(!registry.cancel_for_agent("agent-b", "missing-stream", "unknown stream"));
    assert!(!replacement.is_cancelled(), "retired token cannot cancel its replacement");
    assert!(registry.cancel_for_agent("agent-b", "shared-stream", "replacement cancelled"));
    assert!(!registry.cancel_for_agent("agent-b", "shared-stream", "duplicate cancellation"));
    assert!(replacement.is_cancelled());
    assert_eq!(attached_replacement.reason().as_deref(), Some("replacement cancelled"));
    assert_eq!(registry.active_count(), 1);
    registry.finish("shared-stream");
    assert_eq!(registry.active_count(), 0);
}

#[test]
fn runner_provider_registry_rejects_empty_supplied_token_identity() {
    let registry = RoutedProviderTaskRegistry::default();
    for (agent, stream) in [("", "stream"), ("  ", "stream"), ("agent", ""), ("agent", "  ")] {
        let token = RoutedProviderCancellation::default();
        assert!(registry
            .register_for_agent_with_cancellation(agent, stream, token.clone())
            .is_err());
        assert!(!token.is_cancelled());
        assert_eq!(registry.active_count(), 0);
    }
}

#[test]
fn transcript_runner_registry_shares_direct_and_group_cancellation_across_attachment() {
    for group_member in [false, true] {
        for cancel_at in ["before", "lease-after", "registry-after"] {
            let registry = TranscriptRunnerRegistry::default();
            let admitted = RoutedProviderCancellation::default();
            let stream = format!("shared-{group_member}-{cancel_at}");
            if cancel_at == "before" {
                assert!(admitted.cancel("cancelled before registration"));
            }
            let attached = if group_member {
                registry.register_group_member_with_cancellation("agent-a", &stream, admitted.clone())
            } else {
                registry.register_routed_provider_with_cancellation("agent-a", &stream, true, admitted.clone())
            }.expect("attach the admitted token");
            assert_eq!(registry.agent_id_for_stream(&stream).as_deref(), Some("agent-a"));
            assert_eq!(registry.active_count(), 1);

            let foreign = RoutedProviderCancellation::default();
            let duplicate = if group_member {
                registry.register_group_member_with_cancellation("agent-b", &stream, foreign.clone())
            } else {
                registry.register_routed_provider_with_cancellation("agent-b", &stream, false, foreign.clone())
            };
            assert!(duplicate.is_err());
            assert!(!foreign.is_cancelled());
            assert_eq!(registry.agent_id_for_stream(&stream).as_deref(), Some("agent-a"));
            assert!(!registry.cancel_stream_for_agent("agent-b", &stream, "foreign cancellation"));

            let expected_reason = match cancel_at {
                "before" => "cancelled before registration",
                "lease-after" => {
                    assert!(admitted.cancel("cancelled through lease after registration"));
                    "cancelled through lease after registration"
                }
                "registry-after" => {
                    assert!(registry.cancel_stream_for_agent("agent-a", &stream, "cancelled through registry"));
                    "cancelled through registry"
                }
                _ => unreachable!(),
            };
            assert!(admitted.is_cancelled());
            assert!(attached.is_cancelled());
            assert_eq!(admitted.reason().as_deref(), Some(expected_reason));
            assert_eq!(attached.reason().as_deref(), Some(expected_reason));
            assert!(!registry.cancel_stream(&stream, "duplicate cancellation"));
            assert!(!registry.cancel_stream_for_agent("agent-a", &stream, "duplicate scoped cancellation"));
            assert_eq!(attached.reason().as_deref(), Some(expected_reason));
            assert_eq!(registry.active_count(), 1, "cancellation never retires the task");
            let current = if group_member {
                registry.current_group_member_stream_id_for_agent("agent-a")
            } else {
                registry.current_routed_stream_id_for_agent("agent-a")
            };
            assert_eq!(current.as_deref(), Some(stream.as_str()));

            registry.finish_routed_provider(&stream);
            assert_eq!(registry.active_count(), 0);
            assert!(registry.agent_id_for_stream(&stream).is_none());
            assert!(registry.current_routed_stream_id_for_agent("agent-a").is_none());
            assert!(registry.current_group_member_stream_id_for_agent("agent-a").is_none());
            assert!(!registry.cancel_stream(&stream, "late cancellation"));
            assert_eq!(admitted.reason().as_deref(), Some(expected_reason));
        }
    }
}

#[test]
fn transcript_runner_registry_scoped_cancel_visits_both_registries_and_rejects_replacement() {
    let registry = TranscriptRunnerRegistry::default();
    let direct = RoutedProviderCancellation::default();
    let group = RoutedProviderCancellation::default();
    registry
        .register_routed_provider_with_cancellation("agent-a", "both-stream", false, direct.clone())
        .expect("direct task");
    registry
        .register_group_member_with_cancellation("agent-a", "both-stream", group.clone())
        .expect("independent group task");
    assert!(!registry.cancel_stream_for_agent("agent-b", "both-stream", "foreign cancellation"));
    assert!(!direct.is_cancelled());
    assert!(!group.is_cancelled());
    assert!(registry.cancel_stream_for_agent("agent-a", "both-stream", "scoped cancellation"));
    assert!(direct.is_cancelled());
    assert!(group.is_cancelled(), "a matching direct task must not short-circuit the group task");
    assert_eq!(direct.reason().as_deref(), Some("scoped cancellation"));
    assert_eq!(group.reason().as_deref(), Some("scoped cancellation"));
    assert!(!registry.cancel_stream_for_agent("agent-a", "both-stream", "duplicate cancellation"));
    assert_eq!(registry.active_count(), 2, "requesting cancellation preserves both tasks");

    registry.finish_routed_provider("both-stream");
    let replacement = RoutedProviderCancellation::default();
    registry
        .register_group_member_with_cancellation("agent-b", "both-stream", replacement.clone())
        .expect("new owner for the retired stream");
    assert!(!registry.cancel_stream_for_agent("agent-a", "both-stream", "old agent cancellation"));
    assert!(!replacement.is_cancelled());
    assert!(registry.cancel_stream_for_agent("agent-b", "both-stream", "replacement cancellation"));
    assert_eq!(replacement.reason().as_deref(), Some("replacement cancellation"));
    assert_eq!(registry.active_count(), 1);
    registry.finish_routed_provider("both-stream");
    assert_eq!(registry.active_count(), 0);
}

#[test]
fn transcript_runner_registry_cancels_all_streams_for_deleted_agent_with_reason() {
    let tasks = Arc::new(RoutedProviderTaskRegistry::default());
    let registry = TranscriptRunnerRegistry::new(Arc::clone(&tasks));
    let first = registry
        .register_routed_provider("agent-a", "delete-a-1")
        .expect("a1");
    let second = registry
        .register_routed_provider("agent-a", "delete-a-2")
        .expect("a2");
    let other = registry
        .register_routed_provider("agent-b", "delete-b")
        .expect("b");

    assert_eq!(
        registry.agent_id_for_stream("delete-a-1").as_deref(),
        Some("agent-a")
    );
    assert_eq!(registry.agent_id_for_stream("missing"), None);
    assert_eq!(registry.cancel_agent("agent-a", "agent deleted"), 2);
    assert!(first.is_cancelled());
    assert!(second.is_cancelled());
    assert_eq!(first.reason().as_deref(), Some("agent deleted"));
    assert_eq!(second.reason().as_deref(), Some("agent deleted"));
    assert!(!other.is_cancelled());

    registry.finish_routed_provider("delete-a-1");
    assert_eq!(registry.agent_id_for_stream("delete-a-1"), None);
    registry.finish_routed_provider("delete-a-2");
    registry.finish_routed_provider("delete-b");
}

#[test]
fn production_routed_provider_checkpoint_store_persists_durable_json() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "fabushi-runner-checkpoint-{}-{nonce}",
        std::process::id()
    ));
    let store = ProductionRoutedProviderCheckpointStore::new(
        &root,
        "agent-private-id",
        "stream-private-id",
    );
    let directory = store.directory().to_string_lossy();
    assert!(!directory.contains("agent-private-id"));
    assert!(!directory.contains("stream-private-id"));

    let checkpoint = RoutedProviderCheckpoint::OpenRouter(OpenRouterCheckpoint {
        conversation: vec![
            json!({"role":"user","content":"calendar"}),
            json!({"role":"tool","tool_call_id":"call-1","content":"daily"}),
        ],
        text: "Checking ".into(),
        completed_steps: 1,
        tool_calls_completed: 1,
    });
    let cursor = store.persist(&checkpoint).expect("persist checkpoint");
    let cursor_path = Path::new(&cursor);
    assert!(cursor_path.starts_with(store.directory()));
    assert_eq!(
        cursor_path.extension().and_then(|value| value.to_str()),
        Some("json")
    );

    let decoded: RoutedProviderCheckpoint =
        serde_json::from_slice(&fs::read(cursor_path).expect("read persisted checkpoint"))
            .expect("decode persisted checkpoint");
    assert_eq!(decoded, checkpoint);

    fs::remove_dir_all(&root).expect("remove checkpoint fixture");
}

#[test]
fn transcript_runner_registry_watchdog_interrupts_only_current_direct_and_group_runs() {
    let tasks = Arc::new(RoutedProviderTaskRegistry::default());
    let registry = TranscriptRunnerRegistry::new(Arc::clone(&tasks));
    let stale_direct = registry
        .register_routed_provider("agent-a", "stream-a-stale")
        .expect("stale direct stream");
    let current_direct = registry
        .register_routed_provider("agent-a", "stream-a-current")
        .expect("current direct stream");
    let stale_group = registry
        .register_group_member("agent-a", "group-a-stale")
        .expect("stale group stream");
    let current_group = registry
        .register_group_member("agent-a", "group-a-current")
        .expect("current group stream");
    let other = registry
        .register_routed_provider("agent-b", "stream-b")
        .expect("other stream");

    assert_eq!(
        registry
            .current_routed_stream_id_for_agent("agent-a")
            .as_deref(),
        Some("stream-a-current")
    );
    assert_eq!(
        registry
            .current_group_member_stream_id_for_agent("agent-a")
            .as_deref(),
        Some("group-a-current")
    );

    assert!(registry.interrupt_wedged_run_for_watchdog("agent-a"));
    assert!(!stale_direct.is_cancelled());
    assert!(current_direct.is_cancelled());
    assert_eq!(
        current_direct.reason().as_deref(),
        Some(RUN_WATCHDOG_INTERRUPT_REASON)
    );
    assert!(!stale_group.is_cancelled());
    assert!(current_group.is_cancelled());
    assert_eq!(
        current_group.reason().as_deref(),
        Some(RUN_WATCHDOG_INTERRUPT_REASON)
    );
    assert!(!other.is_cancelled());
    assert!(!registry.interrupt_wedged_run_for_watchdog("missing"));

    for stream_id in [
        "stream-a-stale",
        "stream-a-current",
        "group-a-stale",
        "group-a-current",
        "stream-b",
    ] {
        registry.finish_routed_provider(stream_id);
    }
    assert_eq!(registry.active_count(), 0);
}

#[test]
fn transcript_runner_registry_preserves_frozen_pre_dispatch_supersede_rules() {
    let tasks = Arc::new(RoutedProviderTaskRegistry::default());
    let registry = TranscriptRunnerRegistry::new(Arc::clone(&tasks));

    let ordinary = registry
        .register_routed_provider_with_recovery_shape("agent-a", "ordinary", false)
        .expect("ordinary");
    assert_eq!(
        registry.preempt_routed_agent_for_supersede(
            "agent-a",
            RUN_DIRECT_USER_INTERRUPT_REASON,
            Some(false),
        ),
        0,
        "an undispatched ordinary run must survive a non-recovery supersede",
    );
    assert!(!ordinary.is_cancelled());
    assert_eq!(
        registry.preempt_routed_agent_for_supersede(
            "agent-a",
            RUN_DIRECT_USER_INTERRUPT_REASON,
            Some(true),
        ),
        0,
        "an undispatched ordinary run must survive even when the new turn carries recovery",
    );
    assert!(registry.mark_routed_provider_dispatched("ordinary"));
    assert_eq!(
        registry.preempt_routed_agent_for_supersede(
            "agent-a",
            RUN_DIRECT_USER_INTERRUPT_REASON,
            Some(false),
        ),
        1,
        "once dispatched, a direct user turn supersedes the active run",
    );
    assert!(ordinary.is_cancelled());
    registry.finish_routed_provider("ordinary");

    let recovery = registry
        .register_routed_provider_with_recovery_shape("agent-a", "recovery", true)
        .expect("recovery");
    assert_eq!(
        registry.preempt_routed_agent_for_supersede(
            "agent-a",
            RUN_DIRECT_USER_INTERRUPT_REASON,
            Some(true),
        ),
        1,
        "an undispatched recovery-shaped run may be superseded only by a recovery-carrying turn",
    );
    assert!(recovery.is_cancelled());
    registry.finish_routed_provider("recovery");
}

#[test]
fn transcript_runner_registry_preempts_only_current_one_to_one_run_for_direct_user_turn() {
    let tasks = Arc::new(RoutedProviderTaskRegistry::default());
    let registry = TranscriptRunnerRegistry::new(Arc::clone(&tasks));
    let stale = registry
        .register_routed_provider("agent-a", "direct-stale")
        .expect("stale direct stream");
    let current = registry
        .register_routed_provider("agent-a", "direct-current")
        .expect("current direct stream");

    registry.finish_routed_provider("direct-stale");
    assert_eq!(
        registry
            .current_routed_stream_id_for_agent("agent-a")
            .as_deref(),
        Some("direct-current")
    );
    assert_eq!(
        registry.preempt_routed_agent("agent-a", RUN_DIRECT_USER_INTERRUPT_REASON),
        1
    );
    assert!(!stale.is_cancelled());
    assert!(current.is_cancelled());
    assert_eq!(
        current.reason().as_deref(),
        Some(RUN_DIRECT_USER_INTERRUPT_REASON)
    );
    assert_eq!(
        registry.preempt_routed_agent("missing", RUN_DIRECT_USER_INTERRUPT_REASON),
        0
    );

    registry.finish_routed_provider("direct-current");
    assert_eq!(registry.current_routed_stream_id_for_agent("agent-a"), None);
}

#[test]
fn transcript_runner_registry_preempts_only_current_group_member_and_preserves_new_identity() {
    let tasks = Arc::new(RoutedProviderTaskRegistry::default());
    let registry = TranscriptRunnerRegistry::new(tasks);
    let stale = registry
        .register_group_member("agent-a", "group-stale")
        .expect("stale group stream");
    let current = registry
        .register_group_member("agent-a", "group-current")
        .expect("current group stream");

    registry.finish_routed_provider("group-stale");
    assert_eq!(
        registry
            .current_group_member_stream_id_for_agent("agent-a")
            .as_deref(),
        Some("group-current")
    );
    assert_eq!(
        registry.preempt_group_member_agent("agent-a", "direct user message"),
        1
    );
    assert!(!stale.is_cancelled());
    assert!(current.is_cancelled());
    assert!(registry.take_group_member_preempted("agent-a"));
    assert!(!registry.take_group_member_preempted("agent-a"));

    let next = registry
        .register_group_member("agent-a", "group-next")
        .expect("next group stream");
    registry.finish_routed_provider("group-current");
    assert_eq!(
        registry
            .current_group_member_stream_id_for_agent("agent-a")
            .as_deref(),
        Some("group-next")
    );
    assert_eq!(
        registry.preempt_group_member_agent("agent-a", "second direct user message"),
        1
    );
    assert!(next.is_cancelled());

    registry.finish_routed_provider("group-next");
    assert_eq!(
        registry.current_group_member_stream_id_for_agent("agent-a"),
        None
    );
}


mod received_catalog {
    use super::{post, post_raw};
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex, mpsc};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    use mahayana_host_runtime::extensions::inference::provider_session::{
        McpCatalogKind, McpCatalogReason, McpCatalogRefusal, ProviderSessionError,
        RoutedToolDefinition,
    };
    use mahayana_host_runtime::runner::production_turn_run_shell_adapter::RoutedProviderAttemptControl;
    use mahayana_host_runtime::runner::routed_provider_runtime::{
        RoutedProviderCancellation, RoutedToolBridge, start_routed_mcp_server,
        start_routed_mcp_server_for_attempt,
    };
    use mahayana_host_runtime::runner::{StreamAttemptPolicy, StreamAttemptRuntime};
    use serde_json::{Value, json};

    const BOUND: Duration = Duration::from_secs(10);

    fn refusal() -> McpCatalogRefusal {
        McpCatalogRefusal {
            kind: McpCatalogKind::Tools,
            row: Some(1),
            reason: McpCatalogReason::NameMustBeString,
        }
    }

    fn tool(name: &str, provider: &str, raw: &str) -> RoutedToolDefinition {
        RoutedToolDefinition {
            name: name.into(),
            provider_identifier: provider.into(),
            tool_name: raw.into(),
            description: None,
            input_schema: json!({"type":"object"}),
        }
    }

    enum CatalogStep {
        Valid(Vec<RoutedToolDefinition>),
        Refused,
        Unavailable,
        HeldRefusal { entered: mpsc::Sender<()>, release: mpsc::Receiver<()> },
        Panic,
    }

    struct CatalogBridge {
        steps: Mutex<VecDeque<CatalogStep>>,
        listings: AtomicUsize,
        calls: Mutex<Vec<Value>>,
        server_error: bool,
    }

    impl CatalogBridge {
        fn new(steps: Vec<CatalogStep>, server_error: bool) -> Self {
            Self {
                steps: Mutex::new(steps.into()),
                listings: AtomicUsize::new(0),
                calls: Mutex::new(Vec::new()),
                server_error,
            }
        }

        fn calls(&self) -> Vec<Value> {
            self.calls.lock().expect("captured tool calls").clone()
        }
    }

    impl RoutedToolBridge for CatalogBridge {
        fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
            self.listings.fetch_add(1, Ordering::SeqCst);
            let step = self.steps.lock().expect("catalog sequence")
                .pop_front().expect("unexpected discovery or automatic replay");
            match step {
                CatalogStep::Valid(tools) => Ok(tools),
                CatalogStep::Refused => Err(ProviderSessionError::McpCatalog(refusal())),
                CatalogStep::Unavailable => Err(ProviderSessionError::Transport(
                    "fixture discovery unavailable".into(),
                )),
                CatalogStep::HeldRefusal { entered, release } => {
                    entered.send(()).expect("entered refresh receiver");
                    release.recv_timeout(BOUND).expect("release entered refresh");
                    Err(ProviderSessionError::McpCatalog(refusal()))
                }
                CatalogStep::Panic => panic!("fixture catalog worker failure"),
            }
        }

        fn call_tool(
            &self,
            tool: &RoutedToolDefinition,
            args: Value,
            tool_call_id: &str,
        ) -> Result<Value, ProviderSessionError> {
            assert!(!tool_call_id.is_empty());
            let result = json!({
                "isError": self.server_error,
                "content": [{
                    "type": "text",
                    "text": if self.server_error { "known server error" } else { "known tool result" }
                }],
                "structuredContent": {"callId": tool_call_id, "value": 17}
            });
            self.calls.lock().expect("capture tool call").push(json!({
                "provider": tool.provider_identifier,
                "raw": tool.tool_name,
                "name": tool.name,
                "args": args,
                "callId": tool_call_id,
                "result": result
            }));
            Ok(result)
        }
    }

    fn list(url: &str, id: u64) -> Value {
        let response = post(url, json!({
            "jsonrpc":"2.0", "id":id, "method":"tools/list", "params":{}
        }));
        assert_eq!(response["id"], id);
        response["result"].clone()
    }

    fn call(url: &str, id: u64, name: &str) -> Value {
        let response = post(url, json!({
            "jsonrpc":"2.0", "id":id, "method":"tools/call",
            "params":{"name":name, "arguments":{"value":17}}
        }));
        assert_eq!(response["id"], id);
        response["result"].clone()
    }

    fn assert_refused(result: Result<(), ProviderSessionError>) {
        assert!(matches!(result, Err(ProviderSessionError::McpCatalog(value)) if value == refusal()));
    }

    fn wait_until(label: &str, mut ready: impl FnMut() -> bool) {
        let deadline = Instant::now() + BOUND;
        while !ready() {
            assert!(Instant::now() < deadline, "timed out waiting for {label}");
            thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn invalid_refresh_clears_old_tools_until_valid_or_empty_recovery() {
        let bridge = Arc::new(CatalogBridge::new(vec![
            CatalogStep::Valid(vec![tool("public_alias", "plugin-a", "raw_a")]),
            CatalogStep::Refused,
            CatalogStep::Unavailable,
            CatalogStep::Valid(vec![tool("replacement", "plugin-b", "raw_b")]),
            CatalogStep::Valid(Vec::new()),
        ], false));
        let mut server = start_routed_mcp_server(bridge.clone()).expect("MCP endpoint");
        assert_eq!(list(server.url(), 1)["tools"][0]["name"], "public_alias");
        let accepted = call(server.url(), 2, "public_alias");
        assert_eq!(accepted["isError"], false);
        assert_eq!(list(server.url(), 3)["content"][0]["text"], refusal().to_string());
        assert_eq!(call(server.url(), 4, "public_alias")["isError"], true);
        assert_eq!(list(server.url(), 5)["isError"], true);
        assert_eq!(call(server.url(), 6, "public_alias")["isError"], true);
        assert_eq!(bridge.calls().len(), 1, "invalidated names must not reach execution");

        assert_eq!(list(server.url(), 7)["tools"][0]["name"], "replacement");
        assert_eq!(call(server.url(), 8, "public_alias")["isError"], true);
        assert_eq!(call(server.url(), 9, "replacement")["isError"], false);
        assert_eq!(list(server.url(), 10)["tools"], json!([]));
        assert_eq!(call(server.url(), 11, "replacement")["isError"], true);
        server.close_and_observe_catalog().expect("valid empty refresh clears fault");

        let calls = bridge.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0]["result"], accepted);
        assert_eq!(calls[0]["provider"], "plugin-a");
        assert_eq!(calls[0]["raw"], "raw_a");
        assert_eq!(calls[1]["provider"], "plugin-b");
        assert_ne!(calls[0]["callId"], calls[1]["callId"]);
        assert_eq!(bridge.listings.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn ordinary_unavailability_preserves_last_wins_public_alias_and_real_results() {
        for server_error in [false, true] {
            let bridge = Arc::new(CatalogBridge::new(vec![
                CatalogStep::Valid(vec![
                    tool("automations", "plugin-old", "old_raw"),
                    tool("automations", "plugin-current", "cron"),
                ]),
                CatalogStep::Unavailable,
            ], server_error));
            let mut server = start_routed_mcp_server(bridge.clone()).expect("MCP endpoint");
            assert_eq!(list(server.url(), 1)["tools"].as_array().expect("tools").len(), 1);
            assert_eq!(list(server.url(), 2)["isError"], true);
            let result = call(server.url(), 3, "automations");
            assert_eq!(result["isError"], server_error, "preserve the real known tool result");
            assert_eq!(result["content"][0]["text"],
                if server_error { "known server error" } else { "known tool result" });
            server.close_and_observe_catalog().expect("unavailable is not catalog corruption");
            let calls = bridge.calls();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0]["provider"], "plugin-current");
            assert_eq!(calls[0]["raw"], "cron");
            assert_eq!(calls[0]["result"], result);
        }
    }

    #[test]
    fn unresolved_catalog_fault_survives_unavailable_refresh() {
        let bridge = Arc::new(CatalogBridge::new(vec![
            CatalogStep::Valid(vec![tool("old", "plugin", "raw")]),
            CatalogStep::Refused,
            CatalogStep::Unavailable,
        ], false));
        let mut server = start_routed_mcp_server(bridge.clone()).expect("MCP endpoint");
        list(server.url(), 1);
        assert_eq!(list(server.url(), 2)["isError"], true);
        assert_eq!(list(server.url(), 3)["isError"], true);
        assert_eq!(call(server.url(), 4, "old")["isError"], true);
        assert_refused(server.close_and_observe_catalog());
        assert_refused(server.close_and_observe_catalog());
        assert!(bridge.calls().is_empty());
    }

    #[test]
    fn close_observes_entered_invalid_refresh_only_after_join() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let bridge = Arc::new(CatalogBridge::new(vec![
            CatalogStep::HeldRefusal { entered: entered_tx, release: release_rx },
        ], false));
        let runtime = Arc::new(StreamAttemptRuntime::new(StreamAttemptPolicy::default()));
        let generation = runtime.begin_attempt();
        let control = RoutedProviderAttemptControl::new(
            runtime, generation, RoutedProviderCancellation::default(),
        );
        let mut server = start_routed_mcp_server_for_attempt(
            bridge.clone(), control.clone(), false,
        ).expect("MCP endpoint");
        let url = server.url().to_string();
        let request = thread::spawn(move || list(&url, 1));
        entered_rx.recv_timeout(BOUND).expect("refresh entered before close");
        let close = thread::spawn(move || server.close_and_observe_catalog());
        wait_until("endpoint admission closure", || !control.is_current());
        assert!(!control.is_cancelled(), "endpoint close is not user cancellation");
        assert!(!close.is_finished(), "close must still own the entered worker");
        release_tx.send(()).expect("release refused refresh during close");
        assert_eq!(request.join().expect("HTTP request")["isError"], true);
        assert_refused(close.join().expect("joined endpoint"));
        assert!(bridge.calls().is_empty());
    }

    #[test]
    fn worker_join_failure_cannot_erase_recorded_refusal_or_return_clean_observation() {
        for recorded in [false, true] {
            let mut steps = Vec::new();
            if recorded {
                steps.push(CatalogStep::Refused);
            }
            steps.push(CatalogStep::Panic);
            let bridge = Arc::new(CatalogBridge::new(steps, false));
            let mut server = start_routed_mcp_server(bridge.clone()).expect("MCP endpoint");
            if recorded {
                assert_eq!(list(server.url(), 1)["isError"], true);
            }
            let raw = post_raw(server.url(), json!({
                "jsonrpc":"2.0", "id":2, "method":"tools/list", "params":{}
            }));
            assert!(raw.is_empty(), "panicked worker cannot return an accepted catalog");
            for _ in 0..2 {
                let observed = server.close_and_observe_catalog();
                if recorded {
                    assert_refused(observed);
                } else {
                    assert!(matches!(observed, Err(ProviderSessionError::Protocol(message))
                        if message == "routed MCP catalog observation failed: worker did not complete"));
                }
            }
            assert!(bridge.calls().is_empty());
        }
    }

    // The process fixture below enters the real public runtime, private
    // production attempt, owned Claude child and this actual HTTP endpoint.
    #[cfg(unix)]
    mod claude_process {
        use super::*;
        use std::fs;
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        use std::path::{Path, PathBuf};
        use std::process::{Child, Command, ExitStatus};
        use std::sync::atomic::AtomicBool;

        use mahayana_host_runtime::extensions::inference::provider_session::{
            ProviderMessage, RoutedProvider,
        };
        use mahayana_host_runtime::host_request_context::HostRequestContext;
        use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
            ProviderRetryEvent, ProviderRetryOutcome, ProviderRetryReport,
        };
        use mahayana_host_runtime::runner::routed_provider_runtime::{
            ProductionRoutedProviderCheckpointStore, RoutedProviderRun,
            RunnerRequestContextSnapshot, run_routed_provider_in_runner,
        };

        const CHILD_TEST: &str =
            "received_catalog::claude_process::production_claude_catalog_fault_survives_cli_and_retry_boundaries";
        const ROLE_ENV: &str = "FABUSHI_MCP_CATALOG_CONTRACT_ROLE";
        const ROOT_ENV: &str = "FABUSHI_MCP_CATALOG_CONTRACT_ROOT";
        const NONCE_ENV: &str = "FABUSHI_MCP_CATALOG_CONTRACT_NONCE";
        const CASE_ENV: &str = "FABUSHI_MCP_CATALOG_CONTRACT_CASE";
        const CONFIG_ENV: &str = "FABUSHI_MCP_CATALOG_CONTRACT_CONFIG";
        const CLI_TEXT: &str = "owned Claude catalog fixture output";
        const CANCEL_REASON: &str = "catalog fixture user cancellation";

        struct FixtureDirectory(PathBuf);

        impl FixtureDirectory {
            fn new() -> Self {
                let path = std::env::temp_dir().join(format!(
                    "fabushi-mcp-catalog-contract-{}", uuid::Uuid::new_v4()
                ));
                fs::create_dir(&path).expect("isolated catalog fixture");
                fs::create_dir(path.join("launches")).expect("owned CLI receipts");
                Self(path)
            }
        }

        impl Drop for FixtureDirectory {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }

        struct OwnedChild(Child);

        impl OwnedChild {
            fn wait(&mut self) -> ExitStatus {
                let deadline = Instant::now() + Duration::from_secs(30);
                loop {
                    if let Some(status) = self.0.try_wait().expect("poll owned catalog driver") {
                        return status;
                    }
                    assert!(Instant::now() < deadline, "owned catalog driver exceeded its deadline");
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }

        impl Drop for OwnedChild {
            fn drop(&mut self) {
                if self.0.try_wait().ok().flatten().is_some() {
                    return;
                }
                let _ = self.0.kill();
                let deadline = Instant::now() + BOUND;
                while Instant::now() < deadline {
                    if self.0.try_wait().ok().flatten().is_some() {
                        return;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                eprintln!("owned catalog driver could not be reaped within its cleanup deadline");
            }
        }

        fn read_json(path: &Path) -> Value {
            serde_json::from_slice(&fs::read(path).expect("fixture JSON bytes")).expect("fixture JSON")
        }

        fn publish_json(path: &Path, value: Value) {
            let temporary = path.with_extension("writing");
            fs::write(&temporary, serde_json::to_vec(&value).expect("fixture encoding"))
                .expect("fixture temporary receipt");
            fs::rename(temporary, path).expect("publish complete fixture receipt");
        }

        fn launch_receipts(root: &Path) -> Vec<Value> {
            fs::read_dir(root.join("launches")).expect("owned CLI receipt directory")
                .map(|entry| entry.expect("launch receipt entry").path())
                .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
                .map(|path| read_json(&path))
                .collect()
        }

        fn shell_quote(value: &str) -> String {
            format!("'{}'", value.replace('\'', "'\"'\"'"))
        }

        fn run_owned_case(case: &str) -> Value {
            let root = FixtureDirectory::new();
            let nonce = uuid::Uuid::new_v4().to_string();
            let executable = std::env::current_exe().expect("exact contract executable");
            let wrapper = root.0.join("claude-fixture");
            // Exec replaces the shell with the exact test binary. Only this
            // owned child receives the CLI override and fixture role.
            fs::write(&wrapper, format!(
                "#!/bin/sh\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"--mcp-config\" ]; then\n    shift\n    export {CONFIG_ENV}=\"$1\"\n  fi\n  shift\ndone\nexport {ROLE_ENV}=cli\nexec {} --exact {CHILD_TEST} --nocapture --test-threads=1\n",
                shell_quote(executable.to_str().expect("test executable UTF-8"))
            )).expect("owned Claude trampoline");
            fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700))
                .expect("owned executable permission");
            let stdout_path = root.0.join("driver-stdout.txt");
            let stderr_path = root.0.join("driver-stderr.txt");
            let mut child = OwnedChild(Command::new(&executable)
                .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
                .env(ROLE_ENV, "driver")
                .env(ROOT_ENV, &root.0)
                .env(NONCE_ENV, &nonce)
                .env(CASE_ENV, case)
                .env("CLAUDE_CODE_PATH", &wrapper)
                .env_remove(CONFIG_ENV)
                .env_remove("SAND_AGENT_MODEL")
                .env_remove("SAND_AGENT_MOCK_RESPONSE")
                .env_remove("SAND_CLAUDE_MODEL")
                .stdout(fs::File::create(&stdout_path).expect("owned driver stdout"))
                .stderr(fs::File::create(&stderr_path).expect("owned driver stderr"))
                .spawn().expect("spawn exact catalog driver"));
            let pid = child.0.id();
            let status = child.wait();
            let stdout = fs::read_to_string(stdout_path).unwrap_or_default();
            let stderr = fs::read_to_string(stderr_path).unwrap_or_default();
            assert!(status.success(), "catalog case {case} failed: {status}\n{stdout}\n{stderr}");
            let receipt = read_json(&root.0.join("driver-completed.json"));
            assert_eq!(receipt["pid"].as_u64(), Some(u64::from(pid)));
            assert_eq!(receipt["nonce"], nonce);
            assert_eq!(receipt["case"], case);
            receipt
        }

        fn run_driver() {
            let root = PathBuf::from(std::env::var_os(ROOT_ENV).expect("owned driver root"));
            let case = std::env::var(CASE_ENV).expect("owned driver case");
            let nonce = std::env::var(NONCE_ENV).expect("owned driver nonce");
            let turns = if case == "next-turn" {
                let bridge = Arc::new(CatalogBridge::new(vec![
                    CatalogStep::Refused,
                    CatalogStep::Valid(vec![tool("public_alias", "fixture-plugin", "raw_tool")]),
                ], false));
                vec![
                    run_turn(&root, "invalid-success", bridge.clone()),
                    run_turn(&root, "fresh-turn", bridge),
                ]
            } else {
                let steps = match case.as_str() {
                    "recover-valid" => vec![
                        CatalogStep::Refused,
                        CatalogStep::Valid(vec![tool("public_alias", "fixture-plugin", "raw_tool")]),
                    ],
                    "recover-empty" => vec![CatalogStep::Refused, CatalogStep::Valid(Vec::new())],
                    "availability" => vec![
                        CatalogStep::Valid(vec![tool("public_alias", "fixture-plugin", "raw_tool")]),
                        CatalogStep::Unavailable,
                    ],
                    "tool-success-invalid" | "tool-server-error-invalid" => vec![
                        CatalogStep::Valid(vec![tool("public_alias", "fixture-plugin", "raw_tool")]),
                        CatalogStep::Refused,
                    ],
                    "worker-panic" => vec![CatalogStep::Panic],
                    "invalid-then-worker-panic" => vec![CatalogStep::Refused, CatalogStep::Panic],
                    "invalid-success" | "invalid-nonzero" | "invalid-protocol"
                    | "watchdog" | "user-cancel" => vec![CatalogStep::Refused],
                    _ => panic!("unknown owned catalog case"),
                };
                let bridge = Arc::new(CatalogBridge::new(steps, case == "tool-server-error-invalid"));
                vec![run_turn(&root, &case, bridge)]
            };
            publish_json(&root.join("driver-completed.json"), json!({
                "pid":std::process::id(), "nonce":nonce, "case":case, "turns":turns
            }));
        }

        fn run_turn(root: &Path, scenario: &str, bridge: Arc<CatalogBridge>) -> Value {
            fs::write(root.join("scenario"), scenario).expect("owned CLI scenario");
            let launches_before = launch_receipts(root).len();
            let calls_before = bridge.calls().len();
            let listings_before = bridge.listings.load(Ordering::SeqCst);
            let observed_path = root.join(format!("observed-{scenario}.json"));
            let timed = matches!(scenario, "watchdog" | "user-cancel");
            let runtime = Arc::new(StreamAttemptRuntime::new(StreamAttemptPolicy {
                first_output_timeout: if timed { Duration::from_secs(1) } else { BOUND },
                max_attempts: 3,
                initial_backoff: Duration::ZERO,
                max_backoff: Duration::ZERO,
                max_retry_after: Duration::ZERO,
            }));
            let cancellation = RoutedProviderCancellation::default();
            let monitor_stop = Arc::new(AtomicBool::new(false));
            let monitor = timed.then(|| {
                let runtime = Arc::clone(&runtime);
                let cancellation = cancellation.clone();
                let stop = Arc::clone(&monitor_stop);
                let observed = observed_path.clone();
                let user_cancel = scenario == "user-cancel";
                thread::spawn(move || {
                    let deadline = Instant::now() + BOUND;
                    while !stop.load(Ordering::Acquire) && !observed.exists() {
                        assert!(Instant::now() < deadline, "CLI did not acknowledge catalog refusal");
                        if let Some(generation) = runtime.active_attempt() {
                            // Use the existing reset hook only while the actual
                            // child is starting. The real watchdog must expire
                            // after the HTTP refusal acknowledgement.
                            runtime.reset_first_output_deadline(generation);
                        }
                        thread::sleep(Duration::from_millis(2));
                    }
                    if observed.exists() && user_cancel {
                        cancellation.cancel(CANCEL_REASON);
                    }
                })
            });
            let retries = Arc::new(AtomicUsize::new(0));
            let retry_count = Arc::clone(&retries);
            let reports = Arc::new(Mutex::new(Vec::<ProviderRetryReport>::new()));
            let report_capture = Arc::clone(&reports);
            let checkpoint_store = Arc::new(ProductionRoutedProviderCheckpointStore::new(
                root, "catalog-fixture-agent", scenario,
            ));
            let messages = [ProviderMessage { role:"user".into(), content:"catalog fixture turn".into() }];
            let mut deltas = Vec::new();
            let result = run_routed_provider_in_runner(
                RoutedProviderRun {
                    provider: RoutedProvider::ClaudeCode,
                    fabushi_model: None,
                    codex_model: None,
                    fabushi_request_binding: None,
                    data_dir: root,
                    messages: &messages,
                    typed_replay: None,
                    bridge: bridge.clone(),
                    cursor_auth: None,
                    request_context: RunnerRequestContextSnapshot {
                        context: HostRequestContext {
                            os_version: "catalog fixture".into(),
                            shell: None,
                            time_zone: Some("UTC".into()),
                            transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(),
                            user_full_name: None,
                        },
                        rules: None,
                    },
                    cancellation: cancellation.clone(),
                    checkpoint_store: checkpoint_store.clone(),
                    retry_sink: Some(Arc::new(move |_: &ProviderRetryEvent| {
                        retry_count.fetch_add(1, Ordering::SeqCst);
                    })),
                    retry_report_sink: Some(Arc::new(move |report: &ProviderRetryReport| {
                        report_capture.lock().expect("retry report").push(report.clone());
                    })),
                    stream_attempt_runtime: Some(Arc::clone(&runtime)),
                    usage_sink: None,
                    cloud_agents_enabled: false,
                    multitask_enabled: false,
                    is_computer_use_subagent: false,
                    max_steps: 8,
                    tool_step_reminder: None,
                },
                &mut |delta, accumulated| deltas.push((delta.to_string(), accumulated.to_string())),
            );
            monitor_stop.store(true, Ordering::Release);
            if let Some(monitor) = monitor {
                monitor.join().expect("owned deadline monitor");
            }

            let catalog_error = matches!(scenario,
                "invalid-success" | "invalid-nonzero" | "invalid-protocol" | "watchdog"
                | "invalid-then-worker-panic" | "tool-success-invalid" | "tool-server-error-invalid"
            );
            let reports = reports.lock().expect("captured retry reports");
            let outcome = if catalog_error {
                assert!(matches!(&result, Err(ProviderSessionError::McpCatalog(value))
                    if *value == refusal()), "catalog type must survive actual CLI and adapter: {result:?}");
                assert_eq!(reports.len(), 1, "one permanent refusal report");
                assert_eq!(reports[0].outcome, ProviderRetryOutcome::GaveUpIneligible);
                assert_eq!(reports[0].error_type, "McpCatalogError");
                assert_eq!(reports[0].error_code, "SAND-E0405");
                assert_eq!(reports[0].attempt, 1);
                assert_eq!(reports[0].watchdog_expired, scenario == "watchdog");
                assert_eq!(reports[0].delay_ms, None);
                assert!(!reports[0].resume_from_checkpoint);
                assert_eq!(reports[0].error, refusal().to_string());
                assert!(!cancellation.is_cancelled(), "watchdog and close are not user cancellation");
                "catalog"
            } else if scenario == "user-cancel" {
                assert!(cancellation.is_cancelled());
                assert_eq!(cancellation.reason().as_deref(), Some(CANCEL_REASON));
                // The public runtime retains its existing cancellation check,
                // which returns Transport with the actual user reason.
                assert!(matches!(&result, Err(ProviderSessionError::Transport(message))
                    if message == CANCEL_REASON), "user cancellation must retain precedence: {result:?}");
                assert!(reports.is_empty());
                "cancelled"
            } else if scenario == "worker-panic" {
                assert!(matches!(&result, Err(ProviderSessionError::Protocol(message))
                    if message == "routed MCP catalog observation failed: worker did not complete"),
                    "failed worker join must reject actual CLI success: {result:?}");
                assert!(reports.is_empty());
                "worker-failed"
            } else {
                assert!(matches!(&result, Ok(text) if text == CLI_TEXT), "recovered CLI result: {result:?}");
                assert!(reports.is_empty());
                "success"
            };
            assert_eq!(retries.load(Ordering::SeqCst), 0, "no authorized result may be replayed");
            let launches = launch_receipts(root);
            assert_eq!(launches.len() - launches_before, 1, "one actual CLI process per turn");
            let launch = launches.iter().find(|launch| launch["scenario"] == scenario)
                .expect("exact scenario launch receipt");
            let nonce = std::env::var(NONCE_ENV).expect("driver nonce");
            assert_eq!(launch["nonce"], nonce);
            let cli_pid = launch["pid"].as_u64().expect("actual CLI PID");
            assert_ne!(cli_pid, u64::from(std::process::id()));
            let observed = read_json(&observed_path);
            assert_eq!(observed["pid"].as_u64(), Some(cli_pid));
            assert_eq!(observed["nonce"], nonce);
            assert_eq!(observed["scenario"], scenario);

            let expected_calls = usize::from(matches!(scenario,
                "recover-valid" | "availability" | "fresh-turn"
                | "tool-success-invalid" | "tool-server-error-invalid"
            ));
            let calls = bridge.calls();
            assert_eq!(calls.len() - calls_before, expected_calls, "invalid names never reach execution");
            if expected_calls == 1 {
                let accepted = &calls[calls_before];
                assert_eq!(accepted["result"], observed["toolResult"], "keep the original real result");
                assert_eq!(accepted["callId"], observed["toolResult"]["structuredContent"]["callId"]);
                assert_eq!(accepted["provider"], "fixture-plugin");
                assert_eq!(accepted["raw"], "raw_tool");
                assert_eq!(accepted["args"], json!({"value":17}));
                assert_eq!(accepted["result"]["isError"], scenario == "tool-server-error-invalid");
            } else {
                assert!(observed["toolResult"].is_null());
            }
            let expected_listings = if matches!(scenario,
                "recover-valid" | "recover-empty" | "availability" | "invalid-then-worker-panic"
                | "tool-success-invalid" | "tool-server-error-invalid"
            ) { 2 } else { 1 };
            assert_eq!(bridge.listings.load(Ordering::SeqCst) - listings_before, expected_listings);
            let custody = runtime.effect_custody();
            assert_eq!(custody.started_revision, expected_calls as u64);
            assert_eq!(custody.checkpoint_revision, 0, "CLI catalog handling cannot invent a checkpoint");
            assert_eq!(custody.pending_effects, 0);
            assert_eq!(custody.unknown_result, None, "known server errors are not unknown effects");
            assert!(!checkpoint_store.directory().exists(), "catalog handling cannot persist a fake checkpoint");
            let emits_text = !matches!(scenario,
                "invalid-nonzero" | "invalid-protocol" | "watchdog" | "user-cancel"
            );
            if emits_text {
                assert_eq!(deltas, vec![(CLI_TEXT.to_string(), CLI_TEXT.to_string())],
                    "catalog refusal does not retract already delivered CLI text");
            } else {
                assert!(deltas.is_empty());
            }
            json!({
                "scenario":scenario, "outcome":outcome, "launches":1, "retries":0,
                "effects":expected_calls, "custody":custody, "cliPid":cli_pid
            })
        }

        fn run_cli() -> ! {
            // Bound the exact owned fixture process even if its parent fails.
            thread::spawn(|| {
                thread::sleep(Duration::from_secs(15));
                std::process::exit(71);
            });
            let root = PathBuf::from(std::env::var_os(ROOT_ENV).expect("owned CLI root"));
            let nonce = std::env::var(NONCE_ENV).expect("owned CLI nonce");
            let scenario = fs::read_to_string(root.join("scenario")).expect("owned CLI scenario");
            publish_json(&root.join("launches").join(format!("{}.json", uuid::Uuid::new_v4())), json!({
                "pid":std::process::id(), "nonce":nonce, "scenario":scenario
            }));
            let config = read_json(&PathBuf::from(std::env::var_os(CONFIG_ENV).expect("real --mcp-config")));
            assert_eq!(config["mcpServers"]["fabushi_plugins"]["type"], "http");
            let url = config["mcpServers"]["fabushi_plugins"]["url"].as_str().expect("actual MCP endpoint");
            let initialized = post(url, json!({
                "jsonrpc":"2.0", "id":100, "method":"initialize", "params":{}
            }));
            assert_eq!(initialized["result"]["protocolVersion"], super::super::ROUTED_MCP_PROTOCOL_VERSION);
            let first = if scenario == "worker-panic" {
                assert!(post_raw(url, json!({
                    "jsonrpc":"2.0", "id":1, "method":"tools/list", "params":{}
                })).is_empty());
                Value::Null
            } else {
                list(url, 1)
            };
            let mut tool_result = Value::Null;
            match scenario.as_str() {
                "worker-panic" => {}
                "tool-success-invalid" | "tool-server-error-invalid" => {
                    assert_eq!(first["tools"][0]["name"], "public_alias");
                    tool_result = call(url, 2, "public_alias");
                    assert_eq!(tool_result["isError"], scenario == "tool-server-error-invalid");
                    assert_eq!(list(url, 3)["content"][0]["text"], refusal().to_string());
                    assert_eq!(call(url, 4, "public_alias")["isError"], true);
                }
                "availability" => {
                    assert_eq!(first["tools"][0]["name"], "public_alias");
                    assert_eq!(list(url, 2)["isError"], true);
                    tool_result = call(url, 3, "public_alias");
                    assert_eq!(tool_result["isError"], false);
                }
                "fresh-turn" => {
                    assert_eq!(first["tools"][0]["name"], "public_alias");
                    tool_result = call(url, 2, "public_alias");
                    assert_eq!(tool_result["isError"], false);
                }
                _ => {
                    assert_eq!(first["content"][0]["text"], refusal().to_string());
                    assert_eq!(call(url, 2, "public_alias")["isError"], true);
                    if scenario == "recover-valid" {
                        assert_eq!(list(url, 3)["tools"][0]["name"], "public_alias");
                        tool_result = call(url, 4, "public_alias");
                        assert_eq!(tool_result["isError"], false);
                    } else if scenario == "recover-empty" {
                        assert_eq!(list(url, 3)["tools"], json!([]));
                        assert_eq!(call(url, 4, "public_alias")["isError"], true);
                    } else if scenario == "invalid-then-worker-panic" {
                        assert!(post_raw(url, json!({
                            "jsonrpc":"2.0", "id":3, "method":"tools/list", "params":{}
                        })).is_empty());
                    }
                }
            }
            publish_json(&root.join(format!("observed-{scenario}.json")), json!({
                "pid":std::process::id(), "nonce":nonce, "scenario":scenario,
                "toolResult":tool_result
            }));
            if matches!(scenario.as_str(), "watchdog" | "user-cancel") {
                // Production must kill and reap this child after the actual
                // watchdog or the user cancellation; no fake attempt result.
                let deadline = Instant::now() + BOUND;
                while Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(5));
                }
                panic!("actual Claude child was not stopped");
            }
            if scenario == "invalid-nonzero" {
                eprintln!("owned CLI nonzero exit");
                std::process::exit(23);
            }
            if scenario == "invalid-protocol" {
                println!("\nowned CLI invalid JSON");
            } else {
                println!("\n{}", json!({"subtype":"success", "result":CLI_TEXT}));
            }
            std::io::stdout().flush().expect("CLI output delivery");
            std::process::exit(0);
        }

        #[test]
        fn production_claude_catalog_fault_survives_cli_and_retry_boundaries() {
            match std::env::var(ROLE_ENV).as_deref() {
                Ok("driver") => {
                    run_driver();
                    return;
                }
                Ok("cli") => run_cli(),
                Ok(_) => panic!("unexpected owned catalog role"),
                Err(_) => {}
            }
            for case in [
                "invalid-success", "invalid-nonzero", "invalid-protocol",
                "recover-valid", "recover-empty", "availability",
                "watchdog", "user-cancel", "next-turn", "worker-panic", "invalid-then-worker-panic",
            ] {
                let receipt = run_owned_case(case);
                let turns = receipt["turns"].as_array().expect("completed real turns");
                assert_eq!(turns.len(), if case == "next-turn" { 2 } else { 1 });
                for turn in turns {
                    assert_eq!(turn["launches"], 1);
                    assert_eq!(turn["retries"], 0);
                }
                if case == "next-turn" {
                    assert_eq!(turns[0]["outcome"], "catalog");
                    assert_eq!(turns[1]["outcome"], "success");
                    assert_ne!(turns[0]["cliPid"], turns[1]["cliPid"]);
                }
            }
        }

        #[test]
        fn production_claude_catalog_refusal_retains_completed_success_and_server_error_without_replay() {
            for case in ["tool-success-invalid", "tool-server-error-invalid"] {
                let receipt = run_owned_case(case);
                let turn = &receipt["turns"][0];
                assert_eq!(turn["outcome"], "catalog");
                assert_eq!(turn["effects"], 1);
                assert_eq!(turn["launches"], 1);
                assert_eq!(turn["retries"], 0);
                assert_eq!(turn["custody"]["startedRevision"], 1);
                assert_eq!(turn["custody"]["checkpointRevision"], 0);
                assert_eq!(turn["custody"]["pendingEffects"], 0);
                assert!(turn["custody"]["unknownResult"].is_null());
            }
        }
    }
}
