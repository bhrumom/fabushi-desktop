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

The scoped dossier's pure command-dispatch result is not production-equivalent yet. Before the dispatch-owner slice recorded in `docs/specs/node-host-computer-dispatch-owner.md`, `source/host/src/extensions/node_host/command_dispatch.rs` returned literal fixture payloads for `Act` and `Snapshot`; the current branch removes those false successes, fails the legacy no-executor entry point closed, and exposes an executor-driven boundary for protocol tests. It still has no shipping caller or production executor/permission/lease wiring, so the contract test proves only the boundary and fixture delegation, not real Computer behavior.

The canonical production Computer owner remains `source/host/src/remote_computer_target.rs` plus `mahayana_computer::execute_with_lease_fenced`, while `UnifiedGatewayApi::call_openclaw_node_invoke` currently owns local `system.run` through `HostLocalExecExtension`. These are distinct valid paths. A future node-host repair must either route through those existing owners or introduce a narrowly specified adapter with real currentness/permission/result fencing; it must not preserve the fake snapshot or create a second Computer state owner. Until that repair is implemented and exact-head Actions-tested, OCRM-M13 and the node-host runtime-manifest responsibility remain pending.

## 6. Upstream protocol correction (revalidated 2026-10-29)

The pinned upstream `src/worker/node-computer-protocol.ts` and `src/plugins/computer-use-contract.ts` define `provider.generation` as an opaque non-empty string, and require execution-owner fields on `act`/`snapshot` payloads. The dispatch-owner slice therefore uses string generation fencing and records execution-owner/schema validation as a separate follow-up; it does not claim the current fixture contract is a complete Computer protocol implementation.

## 7. Computer owner decision boundary (revalidated 2026-10-29)

The pinned upstream `src/node-host/computer-command.ts` does not implement desktop side effects itself: it reads the registered provider capabilities, fences the opaque generation, then invokes `computer.act` or `screen.snapshot` and validates the returned contract. Its node-host `system.run` and desktop-stream paths are separate command owners with their own policy, cancellation, and loopback transport rules.

Fabushi's existing `NativeDesktopRemoteExecutor` is a valid native Computer owner for the `Computer` tool, with remote-control policy, `ComputerControlLeaseRequest`, and `execute_with_lease_fenced`; its shipping private target service is already attached through `NetworkEnrollmentSupervisor`. It intentionally rejects other tool names, so it cannot be silently reused as a generic node-host `computer.act`/`screen.snapshot` executor. The remaining production decision is a contract-preserving adapter to that owner, or a separately approved node-host owner, including execution-owner validation, result schema projection, currentness, and real node/device transport. No such adapter is claimed by the current slice.

## 8. Node-host runtime ownership frontier (revalidated 2026-10-29)

The pinned upstream `src/node-host/runtime.ts` prepares the node-local command/plugin manifest, starts the Gateway client, carries `commands`, capabilities and `computerUse` in the node connection, and updates the manifest on availability changes. `src/node-host/client.ts` binds each invoke result to the original invoke id/node id and does not replay a lost mutation. The Gateway-side node registry therefore owns live connection/activity truth; pairing approval alone is not an equivalent substitute.

Fabushi has no shipping Rust node-host client/runtime or live node registry in the inspected owner set. The current `DevicePairingOwner` projection is consequently limited to durable identity and approval. Implementing the missing runtime requires a separate spec covering Gateway protocol, reconnect/cancellation, manifest updates, invoke settlement and real device acceptance; it must not be smuggled into the pairing or node RPC handlers.

## 9. Pinned Rust node-host crate audit (2026-10-29)

The pinned upstream `crates/openclaw-node-host` is contract evidence, not yet an accepted Fabushi implementation. The exact source read so far establishes these owner boundaries:

- `identity.rs` owns Ed25519 device identity and canonical v3 challenge-bound signing metadata, while secure key persistence remains with the embedding host.
- `reconnect.rs` classifies pairing/auth/protocol/configuration failures into typed pauses, bounds transport backoff, permits one explicitly bounded stored-token retry, sanitizes pairing metadata, and keeps request-scoped errors on a healthy session.
- `lifecycle.rs` calls the connection factory once per attempt, re-advertises the current manifest, emits secret-free typed events, hands issued device tokens to the embedding persistence callback, closes/drains the old session on shutdown, and retries runtime failures only after the old session is retired.
- `duplex.rs` bounds and sequences input/output, emits heartbeats without fabricated output, and makes cancellation/stop state observable to the invocation owner.
- `node.rs` preserves the negotiated V3/V4 dialect, withholds command/capability activation until the connect envelope is finalized, strictly parses `paramsJSON`, rejects direct/ambiguous `params`, preserves received JSON and byte size, and correlates invocation/input/cancel/result by node and invocation identity.

The current Fabushi owner decision is therefore: credential persistence remains in the existing credential owner; live session/connection-generation and remote presence must be a single transport owner; invocation drain and cancellation must be a single runtime owner; and RPC/pairing layers may only project those facts. No upstream crate import, second registry, or production completion is claimed. The durable implementation contract is [node-host-runtime-embedding.md](../../../docs/specs/node-host-runtime-embedding.md).

The remaining exact-source closure includes the rest of `crates/openclaw-node-host` (`host.rs`, `runtime.rs`, sidecar protocol/handshake/runtime, and tests) plus the individually unread `src/node-host` files. Inventory and parity flags remain incomplete until those reads and the corresponding owner mapping are recorded.

## 10. Host/runtime boundary audit (2026-10-29)

The exact pinned reads of `host.rs` and `runtime.rs` add two important constraints to the owner map:

- `host.rs` is an embedding adapter, not a product policy owner. It loads non-secret configuration, reads identity/auth through indirections, binds a loopback health surface, creates the bounded command runtime, adopts a returned device token for later attempts, emits redacted lifecycle events, and delegates connection/retry behavior to `NodeLifecycle`/`NodeClient`.
- `runtime.rs` owns bounded invocation admission and settlement: deterministic command/capability registration, reserved `system.*` namespace protection, embedding admission callbacks, concurrency permits, input/output/deadline limits, duplicate-id detection, ordered duplex control, cancellation, and disconnect cleanup. It does not own credential persistence, pairing approval, or platform side effects.
- The upstream host sample currently stores an issued device token in process memory. Fabushi must not copy that storage choice where an existing secure credential owner is available; this is an integration seam, not an acceptance receipt.
- The runtime explicitly refuses duplex commands through its one-shot dispatch path, closes the session on critical delivery saturation/result-task failure, and rejects commands absent from the current connection manifest. These are required lifecycle invariants for any Fabushi adapter.

This narrows the next implementation boundary: an existing Fabushi transport/credential owner must supply the connection factory and token persistence; a node-host runtime owner may compose bounded invocation execution and manifest projection; existing local exec, Computer, PTY, stream, and worker owners remain the only side-effect owners. No product code was changed in this audit.

## 11. Sidecar protocol audit (2026-10-29)

The pinned sidecar sources define an optional out-of-process boundary, not a required node/Gateway path:

- `sidecar_protocol.rs` negotiates additive feature bits and the minimum of both peers' frame/in-flight/bootstrap limits, binds role/direction/session/generation/sequence to one channel instance, authenticates frames with an out-of-band zeroized key, and permanently retires a channel on invalid state or liveness loss.
- `sidecar_handshake.rs` requires supervisor initiation, binds the handshake to the exact channel instance, verifies the runtime's claimed selection against independently recomputed negotiation, and only lowers the frame ceiling after the final acceptance frame.
- `sidecar_runtime.rs` exchanges one bounded configuration/manifest, validates portable JSON and reserved command namespaces, adapts admission/invoke/result/cancel to the bounded command runtime, and projects secret-free lifecycle status. It does not own process creation, artifact verification, or key delivery; those belong to the product supervisor.

Fabushi already has process/supervisor and bounded framing owners, but the inspected owner set does not establish that this OpenClaw sidecar contract is the canonical transport for OCRM node hosting. Therefore it is recorded as an optional adapter contract only. It must not be introduced until a concrete existing worker/supervisor caller, artifact/key handoff, and real packaged acceptance path are identified; a plain line framer or existing local bridge is not evidence of authenticated sidecar equivalence.

## 12. Upstream integration-test oracle audit (2026-10-29)

The pinned crate tests make the required evidence concrete:

- `tests/node_lifecycle.rs` crosses a real WebSocket Gateway fixture, forces a first transport failure, verifies a fresh second connect/challenge/signature, captures the issued device token with redacted Debug output, and checks the complete typed event sequence through clean shutdown.
- `tests/node_session.rs` verifies real Gateway invoke/result exchange, manifest-gated command routing, cancellation during admission before handler construction, retirement before buffered work starts, and suppression of native effects after cancellation.
- `tests/sidecar_process.rs` crosses a real child-process boundary and verifies authenticated offer/accept, configuration/manifest acknowledgement, admission, invoke, and result frames with bounded timeouts.

These are direct oracles for future Actions jobs, not current Fabushi acceptance. The current repository has no evidence yet that its live Gateway/device lane can execute the same node lifecycle or that its existing supervisor can provide the exact sidecar artifact/key handshake. Until those exact-head and real-environment receipts exist, the parity ledger remains pending.

## 13. Pinned `src/node-host/runtime.ts` closure (2026-10-29)

The complete 719-line runtime read confirms that the central OpenClaw node-host responsibility is a connection-generation and cleanup owner, not merely a command switch:

- preparation resolves local plugin/MCP/skill/worker availability into a manifest and inventory; availability changes can publish a new manifest without widening an operator command allowlist;
- each `start()` owns `connectionGeneration`, exact `activeInvokes`, pending invocation tails, a shared disconnect-cleanup barrier, and worker/MCP/manifest watcher lifetimes;
- every invoke waits for the prior cleanup barrier, checks generation and current manifest, installs cancellation/input/progress state before the first awaited handler work, and prevents a redelivered id from orphaning the predecessor;
- `cancelAll()` advances generation before aborting predecessor work, refreshes connection-scoped caches, joins idle-worker and registered-command disconnect cleanup, and latches cleanup failure so later invokes fail closed;
- `close()` is re-entrant-safe and joins watcher, supervisor, MCP, disconnect, and pending invocation tails; aborting a signal alone is explicitly not treated as physical process cleanup.

Fabushi already has related worker supervisor, process-observation, MCP, and cleanup owners, but the inspected Rust node-host slice does not yet compose them behind a single live remote node connection-generation owner. This is the concrete production wiring gap behind the earlier fail-closed remote presence decision. A future implementation must preserve these fences and delegate physical worker/process cleanup to the existing owners rather than reimplementing them in RPC handlers.

## 14. Fabushi owner recheck after runtime closure (2026-10-29)

The current branch recheck finds:

- `source/host/app/src/main.rs` still owns only the RPC projection and local `system.run` path; remote invoke remains explicitly unavailable and remote list/status remains paired-but-disconnected.
- `source/host/src/device_pairing_owner.rs` remains the durable approval/identity owner, not a live Gateway session registry.
- `source/host/src/network_enrollment_owner.rs`, `source/host/src/network_worker_process.rs`, and the existing worker supervisors own private transport/process lifecycle for their declared products, not OpenClaw node-host connection generation.
- `source/host/src/remote_computer_target.rs` remains the Computer/lease owner and must not be widened into a generic node-host runtime.
- Existing MCP and local-exec owners are present, but no current owner exposes the required remote node `connect → manifest → invoke → cancel/result → reconnect` lifecycle.

This is a confirmed owner gap, not permission to add a second registry inside `main.rs`. The next implementation slice must introduce or designate one canonical transport/runtime owner at the host boundary, then adapt the existing owners through it.

## 15. Pinned `src/node-host/invoke.ts` command-owner audit (2026-10-29)

The exact read of `invoke.ts` confirms command dispatch is a policy router over existing owners, with several important fail-closed rules:

- worker supervisor commands, device apps, desktop stream, MCP, Claude CLI, plugin commands, Computer, and `system.run` are separate branches with separate policy/side-effect owners;
- `system.run.prepare` binds execution context, approval policy, cwd, environment, and allow-always coverage before execution; `system.run` then delegates to the existing exec authorization/runner path;
- Computer dispatch requires a supplied worker Computer provider and delegates `computer.act`/`screen.snapshot` through the provider/plugin owner; absence of the provider returns unavailable rather than a synthetic success;
- an abort-aware client suppresses late result/progress/event writes for superseded invocations, while the exact current framed failure may be reported through the current owner only;
- MCP and desktop-stream failures are typed and bounded, and unknown commands return unavailable.

Therefore the future Rust node runtime should model this as a command-owner registry/admission layer, not merge all commands into one generic executor. The existing Fabushi local exec, Computer lease, MCP, worker supervisor, and stream owners remain the only valid side-effect destinations; the missing work is the live remote node transport and generation fence that invokes them safely.

## 16. Pinned manifest and response transport audit (2026-10-29)

`runtime-manifest.ts` and `client.ts` close two more contracts:

- the manifest is the node-local authority for both commands and capabilities; built-ins are filtered by platform/feature/allowlist/ephemeral policy, plugin capabilities are retained only from the plugin registry owner, and `computerUse`/`pathEnv` are withheld on restricted surfaces;
- manifest equality includes path environment, ordered capability/command lists, and serialized Computer capability data, so availability changes must publish a new surface only when the effective declaration changes;
- every invoke response includes the original `id` and `nodeId`, uses the Gateway capability to decide whether permission details may be exposed, and treats result delivery as best-effort without replaying a failed mutation.

The current Fabushi `runtime_manifest.rs` is only a static pure builder and does not yet carry the upstream dynamic plugin/ephemeral/path/Computer surface or a live response transport. The existing `command_dispatch.rs` is likewise only a protocol boundary. These facts remain implementation gaps, not reasons to widen the pairing/RPC owner.

## 17. Live-main delta read for node-host (2026-10-29)

After the live-main drift check, the changed node-host sources were reread at `bf6a97a1761ed7f752544f4934aa708b9d896bd3`:

- `runtime-manifest.ts` now advertises `NODE_CLAUDE_NATIVE_AUTH_CAPABILITY` alongside the existing Claude skills capability when Claude is enabled; this is a live-main capability delta absent from the locked blob.
- `invoke.ts` now wraps the complete dispatch path inside `handleInvoke`, logs formatted unexpected failures, inlines the MCP branch, and sends `system.run` events through the abort-aware invocation client. The command-owner and late-result fences remain, but the error/observability surface differs from the locked source.
- `runtime.ts` and `client.ts` retain the locked blob identities in the current live tree, so their prior exact reads remain directly applicable to those two files only.

The live delta is recorded as new source evidence, not silently merged into the locked inventory. A Rust implementation must not omit the native-auth capability or copy the older invoke error path when the final source baseline is re-frozen.

## 18. Live-main Rust node protocol delta (2026-10-29)

The live `crates/openclaw-node-host/src/node.rs` at `bf6a97a1761ed7f752544f4934aa708b9d896bd3` differs from the locked `ad8d12f126d29b0db204187adf5092db80294e70` source in behaviorally relevant ways:

- protocol constants are imported from `openclaw_gateway_client::protocol` rather than maintained as local node-host constants;
- the V3 comments and framing describe serialized `paramsJSON`, preserving the explicit JSON wire contract;
- hello negotiation now treats the gateway hello value as the server protocol, requires it to be within the offered range, selects the lower of the server and offered maximum, and validates the final dialect against the selected-to-current range rather than requiring one exact value;
- external-signing coverage now exercises both V3 and V4 and uses protocol-specific Windows metadata (`win32|` for V3 and `windows|windows` for V4).

This is not a documentation-only drift: protocol selection, dependency ownership, and signed device metadata can affect interoperability and identity acceptance. The current Rust implementation and parity evidence remain bound to the locked source until the Actions authority deliberately selects a new exact baseline and regenerates inventory, mapping, and verification receipts.
