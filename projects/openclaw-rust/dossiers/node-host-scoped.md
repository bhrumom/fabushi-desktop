# OpenClaw Node Host, Command Dispatch & Manifest Parity Dossier

## 1. Overview & Responsibility Mapping
- **Module ID**: `OCRM-RESP-NODE-HOST-CONNECTION-001`, `OCRM-RESP-NODE-HOST-COMMAND-DISPATCH-001`, `OCRM-RESP-NODE-HOST-RUNTIME-MANIFEST-001`
- **Scope**: Native Rust host runtime extensions and contract tests for node host gateway connection, device authorization, command dispatch, and runtime manifest verification. Bulk shard coverage claims for `src/node-host/` within `part-0042` have been revoked and downgraded to unread/unclassified pending individual per-file reading per Spec §6.2-6.3.
- **Owner**: `mahayana_host_runtime::extensions::node_host` (`gateway_connection.rs`, `command_dispatch.rs`, `runtime_manifest.rs`).

## 2. Audited Files & Upstream Anchors
- **Node Host Connection & Device Auth** (110 files):
  - `src/node-host/gateway-auth.ts`, `src/node-host/gateway-candidate-connection.ts`
  - `src/node-host/client.ts`, `src/node-host/connection.ts`, `src/node-host/connection.test.ts`
  - `src/node-host/node-event-params.ts`, `src/node-host/local-id.ts`, `src/node-host/local-id.test.ts`
  - `src/node-host/host-stats.ts`, `src/node-host/host-stats.test.ts`
  - `src/node-host/config-runtime.test-support.ts`, `src/node-host/startup-state-readiness.ts`
  - And all 97 related node event, local ID, and connection candidate test suites.
- **Command Dispatch, Computer & PTY** (61 files):
  - `src/node-host/computer-command.ts`, `src/node-host/pty-command.ts`, `src/node-host/pty-command.test.ts`
  - `src/node-host/desktop-stream-command.ts`, `src/node-host/portal-stream-command.ts`
  - `src/node-host/invoke-agent-cli-claude-handler.ts`, `src/node-host/claude-skill-session.ts`
  - `src/node-host/exec-policy.ts`, `src/node-host/exec-policy.test.ts`
  - And all 30 related invoke command handlers, PTY streams, and desktop streaming runners.
- **Runtime Manifest & Worker Lifecycle** (38 files):
  - `src/node-host/runtime-manifest.ts`, `src/node-host/runtime.ts`
  - `src/node-host/runner.ts`, `src/node-host/worker-runtime.ts`
  - `src/node-host/launcher-bootstrap.ts`, `src/node-host/auto-update-compatibility.test.ts`
  - And all related worker hosting, macOS/Linux worker lifecycle, and runtime update pause suites.

## 3. Rust Native Equivalence Architecture
1. **Gateway Connection & Origin Scopes** (`gateway_connection.rs`):
   - Canonical `gateway_origin_scope` normalizer (extracting normalized scheme://host:port).
   - Strict `can_reuse_node_host_device_token` logic requiring all candidate URLs to match saved gateway scope.
2. **Command Dispatch & Generation Validation** (`command_dispatch.rs`):
   - `ComputerOperation` state machine (`Close`, `Capabilities`, `Act`, `Snapshot`).
   - Generation parity guard ensuring `providerGeneration` matches current capability generation with `COMPUTER_CONTRACT_MISMATCH` on desynchronization.
3. **Runtime Manifest & Command Advertising** (`runtime_manifest.rs`):
   - Standard node commands advertised dynamically according to enablement flags (Claude, desktop streaming, device apps, system run).
   - Capability exposure (`system_run.execution_context`, `node.claude_skills`).

## 4. Contract Tests & Verification
- Test harness: `source/host/tests/openclaw_node_host_contract.rs`
  - `test_ocrm_node_host_gateway_auth_and_scope`
  - `test_ocrm_node_host_computer_command`
  - `test_ocrm_node_host_runtime_manifest`
- Evidence: 3/3 passing contract tests with positive and negative assertions.

## 5. Current production gap (revalidated 2026-10-29)

The scoped dossier's pure command-dispatch result is not production-equivalent yet. `source/host/src/extensions/node_host/command_dispatch.rs` currently returns literal fixture payloads for `Act` (`{"result":"executed"}`) and `Snapshot` (`{"image_base64":"fake-png-snapshot"}`); the function has no production executor/permission/lease parameter, and repository search found no shipping caller for it. The existing `openclaw_node_host_contract.rs` therefore proves only the helper's generation branching, not real Computer behavior.

The canonical production Computer owner remains `source/host/src/remote_computer_target.rs` plus `mahayana_computer::execute_with_lease_fenced`, while `UnifiedGatewayApi::call_openclaw_node_invoke` currently owns local `system.run` through `HostLocalExecExtension`. These are distinct valid paths. A future node-host repair must either route through those existing owners or introduce a narrowly specified adapter with real currentness/permission/result fencing; it must not preserve the fake snapshot or create a second Computer state owner. Until that repair is implemented and exact-head Actions-tested, OCRM-M13 and the node-host runtime-manifest responsibility remain pending.
