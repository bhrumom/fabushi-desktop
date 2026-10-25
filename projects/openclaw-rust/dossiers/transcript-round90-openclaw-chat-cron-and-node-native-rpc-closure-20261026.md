# OCRM Round 90 — OpenClaw Chat, Cron, and Node Native RPC Closure — 2026-10-26

## 1. Executive Summary and Authority Context

- **Frozen Upstream Authority:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Target Implementation Branch:** `ocrm/round20-snapshot-lifecycle-audit-20261023`.
- **Target Responsibility Units:**
  - `OCRM-UNIT-CHAT-RUNNER-001` (Module `OCRM-M01 Agent Execution Engine & State Machine`)
  - `OCRM-UNIT-CRON-LIFECYCLE-001` (Module `OCRM-M10 Background Tasks, Automations & Cron`)
  - `OCRM-UNIT-NODE-CAPABILITIES-001` (Module `OCRM-M05 Device Network, Mesh & Node Transport`)
- **Core Milestone:** Completed native Rust integration for all 23 methods in `OPENCLAW_GATEWAY_METHODS` in `source/host/app/src/main.rs`. Implemented 12 falsifiable contract test oracles in `source/host/tests/openclaw_chat_cron_node_contract.rs` verifying prompt dispatch idempotency, runner cancellation tokens, cron automation aggregation, local/remote node capabilities, and permission checks.

---

## 2. Source Equivalence & Upstream Contract Mapping

| Upstream Source Path | Blob SHA | Fabushi Rust Implementation | Responsibility Boundary |
|---|---|---|---|
| `src/gateway/server-methods/chat.ts` (send) | `8f2c31e9a2b8e3089d7b415a782bcfbcae912301` | `source/host/app/src/main.rs:call_openclaw_chat_send` | Validates session key, normalizes `agent:{id}` prefix, applies clientNonce deduplication, dispatches prompt to `call_send_prompt`. |
| `src/gateway/server-methods/chat.ts` (abort) | `8f2c31e9a2b8e3089d7b415a782bcfbcae912301` | `source/host/app/src/main.rs:call_openclaw_chat_abort` | Validates session key, invokes `TranscriptRunnerRegistry::cancel_agent`, returns cancellation confirmation and run IDs. |
| `src/gateway/server-methods/cron.ts` (list) | `6c1092e098a0321f879bcde4a09154a72d7f8721` | `source/host/app/src/main.rs:call_openclaw_cron_list` | Aggregates all `SandAgentAutomation` records across agents via `SandAgentSessionStore::list_agent_automations` into OpenClaw `CronJob` schema. |
| `src/gateway/server-methods/cron.ts` (trigger) | `6c1092e098a0321f879bcde4a09154a72d7f8721` | `source/host/app/src/main.rs:call_openclaw_cron_trigger` | Validates `jobId`, locates matching agent automation, dispatches execution via `call_run_agent_automation_now`. |
| `src/gateway/server-methods/nodes.ts` (register) | `7cba099e2831f41b9e28a554efc15a41bca98124` | `source/host/app/src/main.rs:call_openclaw_node_register` | Fast-paths `sand-local-node` as auto-approved host node; delegates remote nodes to `DevicePairingOwner::handle_pair_request`. |
| `src/gateway/server-methods/nodes.ts` (list) | `7cba099e2831f41b9e28a554efc15a41bca98124` | `source/host/app/src/main.rs:call_openclaw_node_list` | Lists local host node with system commands (`system.run`, `canvas.navigate`, `computer.*`) + approved remote devices from `DevicePairingOwner`. |
| `src/gateway/server-methods/nodes.ts` (status) | `7cba099e2831f41b9e28a554efc15a41bca98124` | `source/host/app/src/main.rs:call_openclaw_node_status` | Returns active/connected status for `sand-local-node` or queries `DevicePairingOwner::handle_pair_status` for remote devices; rejects unknown nodes with 400. |
| `src/gateway/server-methods/nodes.ts` (invoke) | `7cba099e2831f41b9e28a554efc15a41bca98124` | `source/host/app/src/main.rs:call_openclaw_node_invoke` | Validates node connection status, requires `idempotencyKey`, and securely executes command. |

---

## 3. Rust Code Modifications & Architecture Details

### 3.1 100% OpenClaw Gateway Method Closure
With this round, all 23 methods declared in `OPENCLAW_GATEWAY_METHODS` are natively handled by `UnifiedGatewayApi::call_openclaw_gateway_command`:
1. `device.pair.request`
2. `device.pair.status`
3. `device.pair.approve`
4. `device.pair.reject`
5. `device.pair.list`
6. `device.pair.revoke`
7. `device.pair.rotateToken`
8. `device.pair.setupCode`
9. `config.get`
10. `config.set`
11. `system.info`
12. `sessions.list`
13. `sessions.get`
14. `sessions.create`
15. `sessions.delete`
16. `chat.send`
17. `chat.abort`
18. `cron.list`
19. `cron.trigger`
20. `node.register`
21. `node.list`
22. `node.status`
23. `node.invoke`

Zero methods remain unhandled or fall back to an external runtime.

### 3.2 Single-Writer State Boundaries
- **Runner State:** Execution supervision is owned by `TranscriptRunnerRegistry` and `HostProductionComposition`. `chat.send` dispatches prompts through canonical `call_send_prompt` which enters the runner stream attempts and watchdogs. `chat.abort` triggers cooperative cancellation through `runner_registry.cancel_agent(agent_id, "rpc")`.
- **Automation State:** Single-writer authority over automations is maintained by `SandAgentSessionStore` and `AutomationRuntime`. OpenClaw `cron.*` RPCs operate purely as a typed projection layer over existing agent automation records.
- **Node & Device Network State:** Device pairing tables (`device_pairing_pending`, `device_pairing_paired`) owned by `DevicePairingOwner` provide the authoritative roster of remote nodes, while `sand-local-node` represents the local host boundary.

---

## 4. Falsifiable Test Oracles (`openclaw_chat_cron_node_contract.rs`)

1. `test_ocrm_chat_o01_send_requires_session_key_message_idempotency_key`: Verifies 400 BadRequest when any required parameter is absent or whitespace.
2. `test_ocrm_chat_o02_session_key_stripping_and_unknown_session_rejection`: Verifies canonical key stripping (`agent:{agentId}:sub`) and 400 BadRequest rejection for unknown agent IDs.
3. `test_ocrm_chat_o03_send_response_contract_fields`: Asserts wire compliance of response fields (`ok`, `runId`, `status: started`, `messageSeq: 1`, `interruptedActiveRun: false`).
4. `test_ocrm_chat_o04_abort_requires_session_key`: Rejects empty or whitespace abort sessionKey.
5. `test_ocrm_chat_o05_abort_invokes_runner_registry_cancellation`: Asserts cancellation signal dispatch to `TranscriptRunnerRegistry`.
6. `test_ocrm_cron_o01_cron_list_empty_and_populated_mappings`: Validates projection of `AutomationRecord` to `CronJob` schema.
7. `test_ocrm_cron_o02_cron_trigger_validation_and_not_found`: Validates `jobId` requirement and 400 BadRequest when job does not exist.
8. `test_ocrm_node_o01_node_register_sand_local_node`: Asserts auto-approval of `sand-local-node`.
9. `test_ocrm_node_o02_node_register_remote_device_delegates_to_pairing`: Validates delegation of remote nodes to `DevicePairingOwner`.
10. `test_ocrm_node_o03_node_list_includes_sand_local_node_and_paired_devices`: Verifies roster includes local host node and approved paired devices.
11. `test_ocrm_node_o04_node_status_local_and_remote`: Verifies active status of local node and approved devices, plus 400 BadRequest for unknown nodes.
12. `test_ocrm_node_o05_node_invoke_contract_and_security`: Verifies argument validation, local command invocation, and rejection of unconnected nodes.

---

## 5. GitHub Actions Verification Receipts

- **Tested Commit SHA**: `0bf05a9fe47098cbef0728c89b7cf395d9e504c5`
- **Workflow Run**: [GitHub Actions Run 40221028279](https://github.com/bhrumom/fabushi-desktop/actions/runs/40221028279) (`Rust desktop runtime`)
- **Overall Status**: `completed`
- **Overall Conclusion**: `success`
- **Matrix Job Receipts**:
  - `renderer`: `completed` (`success`)
  - `Pressure profiler / ubuntu-24.04`: `completed` (`success`)
  - `rust-host`: `completed` (`success`) (including full integration test suite with all 12 `openclaw_chat_cron_node_contract` oracles)
  - `platform-worker`: `completed` (`success`)
  - `Pressure profiler / macos-15`: `completed` (`success`)
  - `Pressure profiler / windows-2025`: `completed` (`success`)

---

## 6. Multi-Agent Synthesis (Modules OCRM-M08 & OCRM-M06)

- **Subagent B (`ec3ab771`)**: Completed deep architecture analysis of `OCRM-M08` (Memory, Vault & Knowledge Retrieval). Formulated 12 falsifiable test oracles covering single-writer Markdown sharding (`user/memory/shards/<agent_id>`, `projects/<slug>/memory/shards/<agent_id>`), two-phase LLM synthesis with `sourceEvidenceIds` grounding and CAS fingerprint checking, tombstone resurrection prevention, and `sqlite-vec` + FTS5 hybrid retrieval.
- **Subagent C (`cbb1e2a0`)**: Completed deep architecture analysis of `OCRM-M06` (Plugin SDK & Hook Lifecycle). Formulated 7 falsifiable test oracles covering fail-closed hook execution, parameter immutability during approval, 16 hook lifecycle stages, and mapping to Fabushi's 22-layered `RoutedToolBridge` pipeline.

