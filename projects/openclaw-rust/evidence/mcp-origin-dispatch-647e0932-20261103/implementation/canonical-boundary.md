# MCP origin — canonical projection test implementation

## Exact subject and status

Authored one complete OD-04 / OD-T1 boundary test in the existing `source/host/tests/mcp_state_executor_contract.rs` target. The existing four tests and their assertions are unchanged. The root-owned `origin: None` constructor edit is preserved. The five finalized transport files were not modified.

- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Task HEAD and final observed HEAD: `9059d0c4493505851339ab228ec0d58a6023c904`.
- Supplied working preimage: `056971e6f985bef49b2612a8d0d381102489991b`.
- Final working blob: `c7aa74419cd77abb20c1da925e0a37c564063de9`; 274 lines.
- Worker delta from the supplied preimage: 104 inserted lines, one replaced import line. The full HEAD diff is +105/−1 because it also contains root's preexisting `origin: None` line.
- Frozen spec: `MCP-ORIGIN-DISPATCH-v1`, blob `1a5edb91bc4a943a1b7d73ffadcb52a1c9df0d59`; shared contract record `reviews/mcp-origin-shared-contract-v1.json`.
- **Validation: not run.** No compiler, formatter, parser, build, test, lint or generator was executed. No staging, commits, GitHub mutations, product/schema/CI edits or changes to another repository path.

## Required named positive

`canonical_mcp_state_preserves_origin_in_process_and_never_serializes_relay_authority`

The case is at final lines 100–197 and uses the existing portable `mcp_state_executor_contract` target. Root owns target/receipt wiring and the Actions run.

## State and custody oracles

| Boundary | Concrete fixture and oracle |
| --- | --- |
| Real owner-created binding | Two actual `CoordinatorToolRelay` instances share one actual `GatewayEventHub`. Each original origin is parsed with public `RoutedMcpOrigin::from_wire`, then sealed only by `relay.bind_tool_origin`. No private binding constructor, fabricated incarnation or custom relay is used. |
| Per-row projection | Input order is z-provider, a-provider, z-provider. The first and third rows have identical public fields but different parsed Attempt IDs and different sealed bindings. `execute_mcp_state` must keep first-seen provider order and retain all three rows, with binding order `[0, 2, 1]` after grouping. Each row's name, provider, raw name, description and complete schema is compared to its independently held original. |
| Current and foreign authority | Every projected row must pass the current relay's actual `validate_tool_origin`. The foreign relay must refuse each original binding with the typed `Protocol` error and `outcome_unknown() == false`. The foreign relay then binds the identical wire origin for itself: its wire identity compares equal, its sealed binding differs, and its own validation succeeds. This prevents a closed or universally rejecting foreign fixture from satisfying the negative oracle. |
| Canonical wire non-disclosure | Encode the bound projection and the separately projected metadata-free original catalog through the actual canonical encoder. Require exact `Vec<u8>` equality. Neither native origin nor private relay incarnation may change those bytes. |
| External decode is unbound | Decode the actual bytes. The whole decoded result must equal the metadata-free projection and differ from the bound in-process projection. Every decoded row has `origin == None`; passing that actual value to `relay.validate_tool_origin` must produce the same typed, pre-publication refusal. |
| No publication | Both actual relays have zero pending requests, and the subscribed actual EventHub receiver remains empty after validation. The case exercises the real local admission gate; it does not claim an external execution or a complete Coordinator request/reply round trip. |

## Manual design and source review

The independent row assertions avoid relying only on two calls to the same grouping implementation. An alphabetical-provider reorder fails the explicit z/a order assertion; deduplication fails row counts; binding loss, reuse or swap fails positional binding equality and the current-relay positive controls. Because the duplicate public rows are otherwise identical, a name-based origin lookup cannot satisfy both expected bindings.

Public-field controls include surrounding whitespace in public name, raw name and description; an empty raw name; absent description; nested object/array schemas; booleans, null and exactly representable fractional numbers. Each schema object has one map entry, keeping the byte-equality oracle independent of unrelated protobuf map-entry ordering. Ordered arrays and nesting still exercise the canonical Value conversion. No canonical codec or schema change was made.

Manual inspection checked the Rust borrow flow, type inference through the actual public constructors, immutable expected rows before moving the metadata-free catalog, generated grouping/encoding/decoding boundaries, and relay error ownership. These are source-review observations, not compilation or passing-test evidence.

| Actual source read | Working blob | Relevant spans |
| --- | --- | --- |
| `source/host/src/extensions/inference/provider_session.rs` | `5cba4171ca0ea2806c1f5b715ca366581196c841` | 185–253: canonical UUID parsing, sealed binding, origin access and routed row fields |
| `source/host/src/runner/coordinator_tool_relay.rs` | `64f0548dd1c93c6813039aaa5dce6b2ab3b8f0d1` | 1–185 and 230–321: actual EventHub owner, bind/validate methods, pre-publication outcome and pending count |
| `source/host/src/ports/mcp_state_executor.rs` | `82aae3d6897b758c6c74eee206015409318c5be6` | Complete file: in-process tool origin, actual grouping, canonical encoder excluding origin and decoder assigning None |
| `source/host/src/host_event_bus.rs` | `0ac14e8c95f12affdaab95e50fd925ef8a5d2148` | Complete file: real stream subscription and synchronous publication |

## Limits and integration

This adds the requested canonical boundary oracle only. Native account/Attempt lifetime, AutoReview, both turn reconstruction integrations and real provider compatibility remain covered by their separate owners and required campaign evidence. No assertion has been relaxed or removed, and no source-only review is promoted to verification or acceptance.

Root should include this exact positive in the already planned portable campaign, freeze the integrated candidate and run it in GitHub Actions. The supplied preimage already contained the root constructor edit, so the following complete shared-worktree diff includes that one preexisting line for accurate HEAD context.

## Complete diff against task HEAD

```diff
diff --git a/source/host/tests/mcp_state_executor_contract.rs b/source/host/tests/mcp_state_executor_contract.rs
index c36d4f6c5b0314d6832962ad689a501e65bb1a99..c7aa74419cd77abb20c1da925e0a37c564063de9 100644
--- a/source/host/tests/mcp_state_executor_contract.rs
+++ b/source/host/tests/mcp_state_executor_contract.rs
@@ -1,7 +1,11 @@
 use std::sync::{Arc, Mutex};
 
 use mahayana_host_runtime::extensions::inference::provider_session::{
-    ProviderPartialToolCall, ProviderSessionError, RoutedToolDefinition,
+    ProviderPartialToolCall, ProviderSessionError, RoutedMcpOrigin, RoutedToolDefinition,
+};
+use mahayana_host_runtime::gateway_server::GatewayEventHub;
+use mahayana_host_runtime::runner::coordinator_tool_relay::{
+    CoordinatorToolRelay, CoordinatorToolRelayError,
 };
 use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
 use mahayana_host_runtime::runner::tools::turn_toolset::project_turn_mcp_toolset;
@@ -28,6 +32,7 @@ impl SandMcpToolProvider for Provider {
 
 fn tool(provider: &str, name: &str, tool_name: &str) -> RoutedToolDefinition {
     RoutedToolDefinition {
+        origin: None,
         name: name.into(),
         provider_identifier: provider.into(),
         tool_name: tool_name.into(),
@@ -92,6 +97,105 @@ fn canonical_agent_v1_wire_round_trip_preserves_grouping_and_tool_schema() {
     assert_eq!(decoded, projected);
 }
 
+#[test]
+fn canonical_mcp_state_preserves_origin_in_process_and_never_serializes_relay_authority() {
+    let events = GatewayEventHub::default();
+    let published = events.subscribe();
+    let relay = CoordinatorToolRelay::new(events.clone());
+    let foreign_relay = CoordinatorToolRelay::new(events);
+
+    let mut repeated = tool("z-provider", "  repeated-public  ", "  raw-name  ");
+    repeated.description = Some("  literal description  ".into());
+    // Single-entry maps keep byte comparison independent of protobuf map order.
+    repeated.input_schema = json!({"allOf": [
+        {"type": "object"},
+        {"properties": {"q": {"type": "string"}}},
+        {"additionalProperties": false},
+        {"default": {"nested": [null, true, 1.25, {"mode": "literal"}]}},
+    ]});
+    let mut other = tool("a-provider", "public-other", "");
+    other.description = None;
+    other.input_schema = json!([{"type": "string"}, false, null, 2.5]);
+    let metadata_free_tools = vec![repeated.clone(), other, repeated];
+    let bindings: Vec<_> = (0..metadata_free_tools.len())
+        .map(|index| {
+            let origin = RoutedMcpOrigin::from_wire(&json!({
+                "version": 1,
+                "coordinatorSourceId": "1b42cc41-58cd-46a0-aa38-20dfcd52b5d7",
+                "mcpAttemptId": format!("cb62c476-c3f1-4d22-a6e4-{:012x}", index + 1),
+            }))
+            .expect("parse canonical native origin");
+            relay.bind_tool_origin(origin).expect("bind to current relay")
+        })
+        .collect();
+    assert_ne!(bindings[0], bindings[2], "duplicate public rows retain different origins");
+    let bound_tools = metadata_free_tools
+        .iter()
+        .cloned()
+        .zip(&bindings)
+        .map(|(mut tool, binding)| {
+            tool.origin = Some(binding.clone());
+            tool
+        })
+        .collect();
+    let projected = execute_mcp_state(&Provider { tools: bound_tools, fail: false })
+        .expect("project real relay bindings");
+    let McpStateExecResult::Success(state) = &projected;
+    assert_eq!(state.servers.len(), 2);
+    assert_eq!(state.servers[0].server_identifier, "z-provider");
+    assert_eq!(state.servers[1].server_identifier, "a-provider");
+    assert_eq!(state.servers[0].tools.len(), 2);
+    assert_eq!(state.servers[1].tools.len(), 1);
+    for server in &state.servers {
+        assert_eq!(server.server_name, server.server_identifier);
+        assert_eq!(server.status, "connected");
+        assert_eq!(server.error_message, None);
+    }
+    let grouped_rows: Vec<_> = state.servers.iter().flat_map(|server| &server.tools).collect();
+    for (row, original_index) in grouped_rows.iter().zip([0, 2, 1]) {
+        let expected = &metadata_free_tools[original_index];
+        assert_eq!(row.name, expected.name);
+        assert_eq!(row.provider_identifier, expected.provider_identifier);
+        assert_eq!(row.tool_name, expected.tool_name);
+        assert_eq!(row.description, expected.description);
+        assert_eq!(row.input_schema, expected.input_schema);
+        assert_eq!(row.origin.as_ref(), Some(&bindings[original_index]));
+        relay.validate_tool_origin(row.origin.as_ref()).expect("current row remains executable");
+        let foreign = foreign_relay.validate_tool_origin(row.origin.as_ref())
+            .expect_err("another relay cannot borrow this row's binding");
+        assert!(!foreign.outcome_unknown(), "foreign binding refuses before publication");
+        assert!(matches!(foreign.into_error(), CoordinatorToolRelayError::Protocol(_)));
+    }
+
+    let metadata_free = execute_mcp_state(&Provider { tools: metadata_free_tools, fail: false })
+        .expect("project corresponding public fields without metadata");
+    let bytes = encode_canonical_mcp_state_result(&projected).expect("encode bound projection");
+    let public_bytes = encode_canonical_mcp_state_result(&metadata_free)
+        .expect("encode metadata-free projection");
+    assert_eq!(bytes, public_bytes, "canonical bytes must not expose relay authority or native origin");
+    let decoded = decode_canonical_mcp_state_result(&bytes).expect("decode external protobuf");
+    assert_eq!(decoded, metadata_free, "wire preserves every public field, schema and row order");
+    assert_ne!(decoded, projected, "wire decode must not recreate the in-process bindings");
+    let McpStateExecResult::Success(decoded_state) = &decoded;
+    for row in decoded_state.servers.iter().flat_map(|server| &server.tools) {
+        assert!(row.origin.is_none());
+        let unbound = relay.validate_tool_origin(row.origin.as_ref())
+            .expect_err("decoded wire rows have no local execution authority");
+        assert!(!unbound.outcome_unknown(), "unbound wire row refuses before publication");
+        assert!(matches!(unbound.into_error(), CoordinatorToolRelayError::Protocol(_)));
+    }
+
+    let foreign_binding = foreign_relay.bind_tool_origin(bindings[0].origin().clone())
+        .expect("foreign relay can bind the same wire identity for itself");
+    assert_eq!(foreign_binding.origin(), bindings[0].origin());
+    assert_ne!(foreign_binding, bindings[0], "wire identity is not the private relay binding");
+    foreign_relay.validate_tool_origin(Some(&foreign_binding))
+        .expect("foreign relay positive control is live");
+    assert_eq!(relay.pending_count(), 0);
+    assert_eq!(foreign_relay.pending_count(), 0);
+    assert!(matches!(published.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)));
+}
+
 struct RecordingBridge {
     tools: Vec<RoutedToolDefinition>,
     observed: Arc<Mutex<Vec<String>>>,
```

