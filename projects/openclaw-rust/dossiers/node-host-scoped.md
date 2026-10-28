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

## 19. Node-host closure surface count (2026-10-29)

The locked upstream tree contains 226 blob files in the node-host scope: 209 under `src/node-host/` and 17 under `crates/openclaw-node-host/`. The three existing node-host parity rows cover only the connection, command-dispatch, and manifest responsibilities; they do not establish file-by-file closure for the remaining worker, stream, update, plugin, protocol, sidecar, and test sources. Accordingly, the node-host source-closure and responsibility-mapping gates remain open even though the three rows now point at production source files rather than test-only anchors.

## 20. Host live-session registry seam (2026-10-29)

The first production wiring slice adds `source/host/src/extensions/node_host/session_registry.rs` as the single Host-side live-session fact owner. It provides generation-bearing `connect`, `set_active`, `disconnect`, `get`, and `list` operations; a predecessor lease cannot retire or mark active a successor session. Disconnect clears the advertised command and capability surface.

`UnifiedGatewayApi` now admits the local Host session through this registry at process startup and projects `node.list`, `node.status`, and local `node.register` from the registry. Pairing-backed remote rows remain explicitly paired-but-disconnected until a real transport owner calls `connect`; remote `node.invoke` remains unavailable. This removes the prior RPC-side synthetic local snapshot, but it does not yet implement the remote Gateway transport, invocation runtime, reconnect loop, or exact-head/real-device acceptance.

## 21. Gateway token scope correction (2026-10-29)

The locked `gateway-auth.ts` delegates to `gatewayOriginScope`, whose normalized scope includes the URL pathname (with `/` and trailing slashes normalized), not only scheme/host/port. The former Rust helper therefore admitted token reuse across different Gateway context paths. `gateway_connection.rs` now parses absolute URLs, preserves the normalized pathname, removes default ports, handles IPv6 host formatting, and fails back to the trimmed input for unparseable values. The contract fixture now covers matching `/ws` candidates and rejects a different pathname, so this security-sensitive oracle remains explicit.
## 22. Gateway candidate failover seam (2026-10-29)

The locked gateway candidate connection requires a non-empty candidate list, forwards events only from the current candidate, promotes the first successful hello as the winner, and advances only when a candidate closes pre-hello without sending the connect request. After a winner or stop, late candidate callbacks are inert; stopping drains every candidate client and preserves cleanup failures.

Rust now contains the transport-neutral NodeHostGatewayCandidateRouter and websocket URL formatter. It captures the failover admission and IPv6/context-path formatting rules while leaving sockets, TLS, Cloudflare Access headers, request dispatch, and stop/drain with the future Gateway client owner. It is therefore a parity seam, not a claim that remote transport is wired.

## 23. Gateway endpoint configuration boundary (2026-10-29)

The locked node-host configuration persists optional TLS fingerprint and WebSocket context-path metadata, while Cloudflare Access remains a pair of SecretInput values resolved by the existing secrets/config owner and redacted before use. Rust NodeHostGatewayConfig now carries the non-sensitive fingerprint/path fields and the candidate formatter consumes the configured context path. No Cloudflare credential, token, or materialized secret was added to the session registry or parity projection; the transport/config owner still needs to wire secret resolution and TLS pin enforcement.
## 24. Strict invocation frame admission (2026-10-29)

The locked Rust node client decodes node.invoke.request with canonical paramsJSON, rejects a direct params field, preserves the original serialized parameter text and byte count, validates optional timeout/idempotency/session metadata, and bounds node.invoke.input payloadJSON at 16 KiB. It also rejects empty correlation identity fields and parses node.invoke.cancel by invoke id plus node id.

Rust now exposes the equivalent transport-neutral invoke_protocol seam with request, input, and cancel parsers plus falsifiable unit oracles for canonical preservation, direct-params rejection, malformed JSON, input bounds, and cancel identity. The parser is not yet called by a remote Gateway transport; the public Host node.invoke compatibility path remains local-only and remote invocation remains fail-closed unavailable.
## 25. Ordered invoke-input buffer seam (2026-10-29)

The locked node-invoke-input owner advances the expected sequence on any non-stale frame, records gaps without rejecting them, buffers frames received before handler registration, and fails closed when the pending buffer exceeds 64 KiB. Duplicate/late sequence values are ignored and registration drains buffered input in order.

Rust now contains NodeInvokeInputBuffer with the same bounded state transitions and focused oracles. It remains a transport/runtime primitive until the live node invocation owner supplies the handler and couples buffer failure to invocation cancellation.
## 26. Invocation generation-fence seam (2026-10-29)

The pinned Rust node client correlates invocation id and node id, rejects duplicate admission, drains/cancels work on connection retirement, and prevents a predecessor result from settling after its generation is retired. The side-effect owner remains responsible for actually stopping physical work and reporting outcome-unknown transport loss.

Rust now contains NodeHostInvocationRegistry with generation-bound admission, duplicate rejection, one-way cancel/settle transitions, and generation drain. The seam is not yet coupled to a live Gateway connection or side-effect executor, so production wiring and exact-head verification remain pending.

## 27. Current live-main identity refresh (2026-10-29)

The live `openclaw/openclaw` default branch advanced again after the earlier node-host delta read. The current direct GitHub identity is commit `70bf5121704c7a7173722c2bd7e360362e5da165`, tree `3732f45aa4d3a924ac88b4b684f21fbab13b3c79`; the locked implementation source remains `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The locked-to-live compare reports 4,851 commits ahead. Because the very large compare response did not provide a usable file list, file absence or equality is not inferred from it. Direct live-tree observations for the node-host surface are recorded for `crates/openclaw-node-host/src/node.rs`, `crates/openclaw-node-host/src/runtime.rs`, `src/node-host/config-shared.ts`, `src/node-host/connection.ts`, `src/node-host/invoke.ts`, and `src/node-host/runtime-manifest.ts` using their current live blob identities. This refresh supersedes the earlier `bf6a97a1`/`3fa25384` observation for live identity only; it does not accept the new tree as the implementation baseline, alter inventory/parity counts, or close source mapping.

## 31. Live-main refresh after progress/stream read (2026-10-29)

The live default branch advanced again to `29ebbe869cc4b4d085be6f7ef79f0edc3c204ec9`, tree `d2f7823097e89fb26dd27ad3ef6fe42d1451c8a2`; the locked implementation source remains unchanged. The locked-to-live compare now reports 4,853 commits ahead and returns 300 changed files. The current live progress and stream blobs are `2a42164cf8ee933250e63ef7017362c47c99b9bb` and `080cdf9110eb90c1163490475cfe52b03f526e5a`, respectively, so the detailed behavior read and Rust seam just recorded remain explicitly bound to the locked blobs, not to live main. No inventory, baseline, or completion flag is changed by this refresh.

## 28. Local invocation admission wiring (2026-10-29)

The local `node.invoke system.run` path now admits its idempotency key against the startup local-session lease in `NodeHostInvocationRegistry` before calling the existing `HostLocalExecExtension`, then settles the admission after the executor returns. Duplicate invocation ids therefore fail before a second local side effect. This is a narrow producer-to-owner wiring improvement and does not claim transport parity: remote Gateway admission, cancellation propagation, disconnect generation drain, progress/result publication, and physical side-effect cancellation remain open.

## 29. Locked progress and auxiliary stream transport read (2026-10-29)

The exact locked blobs `src/node-host/node-invoke-progress.ts` (`26535b827d8bebd549491badea55a7f7cfccd6f6`) and `src/node-host/node-stream-transport.ts` (`aaaf11f3114eedc0eb71c0f2f34ef245e1e41ead`) add two distinct transport responsibilities:

- Invocation progress serializes writes through one promise queue, splits text into 16 KiB UTF-8-safe chunks, increments sequence only after a successful Gateway request, reports the first error, and emits empty chunks as bounded heartbeats. Heartbeats clamp idle timeout to 250–5000 ms, coalesce concurrent requests, stop on abort/close, and never turn a failed progress publication into a successful invocation result.
- Auxiliary streams bind an enrolled Gateway WebSocket attach URL to the expected path and original gateway origin/mount, apply a 1 MiB WebSocket payload limit, optional TLS fingerprint pin and Cloudflare Access headers, and send binary metadata before splicing bytes. Target-to-Gateway flow pauses on WebSocket backpressure; Gateway-to-target rejects non-binary frames; target close waits for close/flush acknowledgement within bounded timers before terminating; abort, socket, WebSocket, malformed-frame, send, and startup failures retain typed diagnostic triggers and destroy owned resources.

The stream implementation is an adapter around the existing Gateway WebSocket/TLS/SecretInput owners, not a second durable node state owner. Fabushi currently has no shipping node-host progress publisher or auxiliary Gateway stream caller in `source/host`; these reads therefore refine the open node-host transport contract and do not change `wired`, `verified`, or `accepted` status. A Rust implementation must preserve the bounded queue/backpressure and close-flush semantics while delegating credential materialization and WebSocket ownership to existing boundaries.

## 30. Rust progress framing seam (2026-10-29)

`source/host/src/extensions/node_host/invoke_progress.rs` now implements the transport-neutral part of the locked progress contract: invocation/node correlation, monotonically increasing sequence numbers, UTF-8-safe 16 KiB chunking, 250–5000 ms heartbeat interval clamping, heartbeat scheduling state, and terminal stop behavior. It emits frames for a future Gateway publisher but owns no socket, promise queue, credential, cancellation source, or result settlement. Its focused tests are source-level oracles only; no local test execution or production transport wiring is claimed.

## 32. Auxiliary stream URL admission seam (2026-10-29)

`gateway_connection::attach_gateway_stream_url` now captures the pure URL-admission part of the locked auxiliary stream owner: only `ws`/`wss` transport is accepted, the resolved attach path must match the expected path and remain on the connected Gateway origin, and the Gateway context mount is prepended before delivery. It deliberately does not open sockets, apply TLS pins, materialize Cloudflare credentials, splice bytes, or own close/drain state. Those responsibilities remain pending in the existing transport/config owners.

## 34. Auxiliary stream splice state seam (2026-10-29)

`source/host/src/extensions/node_host/stream_transport.rs` now models the non-I/O stream contract: binary-only Gateway frames, a 1 MiB payload bound, target-to-Gateway pause/resume at 4 MiB buffered bytes, first-trigger preservation, and bounded target-close decisions for close acknowledgement versus buffered flush. It is deliberately not a socket or WebSocket implementation; actual writes, timer ownership, TLS/edge credentials, and resource destruction remain with the future Gateway transport owner.

## 33. Live-main refresh after stream URL seam (2026-10-29)

Live main advanced to `c0f1ba6ba4439a79d2cb91dde9666038fff7ad69`, tree `f80fc84244c576760b0d36555d9b6849978b6baf`. The locked-to-live compare reports 4,854 commits ahead and 300 returned changed files. The auxiliary stream URL seam remains explicitly based on the locked source read; this live refresh changes no lock, inventory, mapping count, or completion flag.

## 35. Live-main refresh after stream splice seam (2026-10-29)

Live main currently resolves to `bf9709250b87bec4a26637046096092ee68d3366`, tree `9daef696c7c912ef65b44f7005c86309f26fd646`; the locked-to-live compare is 4,855 commits ahead with 300 returned changed files. The stream splice seam remains bound to the locked source read and does not change the lock, inventory, mapping totals, or completion flags.

## 36. Desktop and portal stream command admission (2026-10-29)

The locked `node-desktop-protocol.ts` and `portal-stream-command.ts` require a 16 KiB bounded JSON request, exact lower-case 48-hex ticket, fixed attach path, integer port in `1..=65535`, and no unknown fields. Desktop username requires an absolute password-file path; the parser does not read or expose that secret. Rust `desktop_stream_protocol.rs` now preserves these admission rules for desktop and portal requests while leaving RFB probing, password materialization/redaction, WebSocket transport, and ticket ownership to their existing/future owners.

## 37. Worker supervisor identity and receipt seam (2026-10-29)

The locked `node-supervisor-protocol.ts` binds every launch/receipt to `launchId`, 64-hex `planHash`, environment/session identity, `ownerEpoch`, `placementGeneration`, and `runId`. Receipt states are constrained to pending/running/completed/failed/interrupted/cancelled; completed result JSON and terminal error text are bounded, with no newline-bearing error diagnostics. Rust `worker_supervisor_protocol.rs` now provides exact-key identity/receipt admission and full-field equality matching. It does not own process/container launch, durable receipts, cleanup, or worker side effects, which remain unresolved owner integration work.

The same seam now validates the launch cross-fields that prevent bundle or placement confusion: `environmentSession=1`, safe `gatewayNamespace`, optional bounded `sessionKey`/`idleRetention`, `launchId == descriptor.assignment.turnId`, and `expectedBundleHash == descriptor.admission.handshake.bundleHash`. Full descriptor/schema validation remains outside this seam.

`worker_bundle.rs` now resolves an installed `worker.mjs` only after canonicalizing the configured root, namespace bundle, and entry, then requiring strict containment and a regular file. It rejects unsafe namespace/hash components and symlink escapes. It does not spawn the worker or own process-group/container cleanup.

`worker_launch_policy.rs` now captures the locked launch-mode admission: local inference cannot run in a container and requires a trusted inference snapshot; container mode requires a non-loopback WebSocket endpoint, forbids host browser assignments, and requires a lifecycle owner. The policy returns a mode only; it performs no spawn, container creation, credential materialization, or cleanup.

## 38. Host runtime composition and local projection wiring (2026-10-29)

`NodeHostRuntimeState` now composes the existing session-generation registry and invocation-generation registry behind one Host owner. Session replacement drains the predecessor generation before the successor is admitted, while invocation admission and settlement remain generation-fenced. `UnifiedGatewayApi` now reads node status/list projections and admits local `node.invoke system.run` through this composed owner; startup creates the local lease through the same runtime rather than constructing parallel session and invocation owners.

This is a real local producer-to-owner wiring improvement, but it is not remote transport acceptance. The runtime still has no Gateway WebSocket caller, remote manifest refresh, progress publisher, physical disconnect cancellation, worker launch owner, or exact-head Actions receipt. The parity row therefore remains `wiring=not-wired` for the full node-host responsibility and `verification=not-run`/`acceptance=pending`.

## 39. Live-main identity refresh after runtime composition (2026-10-29)

The current direct GitHub identity for `openclaw/openclaw` main is commit `56eba2fae267818e540d9826899c2288cf5aea08`, tree `19327a453b4374bf9fa34855d2c1e5406372ed76`. The locked-to-live compare is `4856` commits ahead with `300` returned files. Current live blob observations include `src/node-host/invoke.ts` at `dd97dd4eed05eb6878bed0024f1a6c9a8b1c56dc` and `src/node-host/node-stream-transport.ts` at `080cdf9110eb90c1163490475cfe52b03f526e5a`.

This refresh does not change `upstream.lock.json`, inventory totals, mapping counts, or any completion flag. The locked revision remains the implementation baseline, and the live-main delta remains evidence for future re-baselining rather than proof of current Fabushi parity.

## 40. Locked Rust crate host/runtime/sidecar closure read (2026-10-29)

The locked crate files `host.rs`, `runtime.rs`, `sidecar_handshake.rs`, `sidecar_protocol.rs`, and `sidecar_runtime.rs` were read at their exact locked blobs and added to the node-host source ledger. Their responsibilities are distinct:

- `host.rs` owns headless host configuration/secret indirection, health listener bounds, Gateway lifecycle invocation, issued device-token retry/adoption, redacted lifecycle events, and the product command runtime factory.
- `runtime.rs` owns bounded command registration/admission, cancellation, duplex input/output, timeout/grace handling, concurrency, result delivery, and disconnect cleanup; the embedding host supplies the policy and handlers.
- `sidecar_handshake.rs` and `sidecar_protocol.rs` own authenticated, generation-bound, sequence-checked framed IPC negotiation and channel retirement, while the product supervisor owns process creation, artifact verification, IPC transport, and session-key delivery.
- `sidecar_runtime.rs` owns the bounded portable-JSON sidecar manifest/configuration exchange and adapter bridge; its capability adapter is an embedding boundary, not a product process owner or durable state store.

No new Fabushi sidecar or generic command runtime was added. The existing owner decision remains fail-closed: only an identified Fabushi supervisor/worker owner may supply process lifecycle, artifact verification, authenticated IPC transport, and adapter permissions. Until that owner and its production caller are identified, these crate reads are mapping evidence; remote transport, worker launch, sidecar acceptance, and exact-head Actions remain pending.

## 41. Existing worker-owner revalidation (2026-10-29)

The current Fabushi candidates were re-read before any worker bridge was attempted. `network_worker_process.rs` owns the enrolled Tailscale/network worker protocol and process tree for network enrollment; `network_enrollment_owner.rs` owns its grant/state lifecycle and secret material. `agent_isolation/agent_worker_pool.rs` owns conversation blob-store workers, not Gateway node workers. `runner/process_tree.rs` only provides generic descendant termination for an already-owned child. CloudAgent and runner watch owners likewise do not expose the combined node-worker launch identity, bundle admission, environment/session, owner epoch, placement generation, and durable receipt lifecycle required by the locked supervisor contract.

Therefore none of these is an approved substitute owner. Adding a sidecar launch or adapting a network/blob worker here would create an ownership collision and cannot be counted as production wiring. The owner boundary decision remains active and the node-worker launch/receipt path stays explicitly pending.

## 42. Canonical inventory reconciliation for audited source refs (2026-10-29)

The canonical row shards were reconciled against the exact path/blob/range references already present in `parity-ledger.json`. This removed stale one-way edges for the audited source set without changing the locked upstream commit/tree or global completion flags. The resulting structural counts are 58,750 tracked, 894 read-complete, 886 responsibility-mapped, 57,856 unread, and 57,864 unclassified; all 886 ledger source paths now have reverse shard responsibility edges and shard/manifest hashes agree.

This is a bookkeeping and traceability correction, not evidence of full source closure or semantic completion. The remaining unread/unclassified denominator, dependency closure, implementation, production wiring, Actions verification, and product acceptance remain open.

## 43. Locked node client lifecycle source read (2026-10-29)

The remaining core Rust crate files `duplex.rs`, `identity.rs`, `lifecycle.rs`, `node.rs`, `reconnect.rs`, `lib.rs`, and `main.rs` were read at their locked blobs and mapped to the NodeHost connection responsibility. The read confirms the upstream contract boundary: Ed25519 device identity and challenge signing; V3/V4 negotiation with bounded connect/challenge/request/write/event buffers; ordered duplex input, progress and cancellation; typed lifecycle events with shutdown drain and runtime restart; retry classification for pairing/auth/protocol/configuration versus transient transport; and a thin binary entrypoint that delegates to the host runner.

These sources strengthen the Rust contract mapping but do not authorize importing the crate as a second runtime. Fabushi still lacks a shipping remote Gateway client, credential persistence adapter, manifest refresh caller, device/transport lane, and current exact-head Actions receipt; node-host connection wiring and acceptance remain pending.

## 44. Node-host manifest, sidecar documentation, and oracle read (2026-10-29)

The locked `crates/openclaw-node-host/Cargo.toml` and `SIDECAR_PROTOCOL.md` were read and mapped with the connection contract. The manifest confirms a private Rust 1.93 crate with forbidden unsafe code, bounded Tokio/network/crypto dependencies, and a separate `openclaw-node` binary. The protocol document explicitly keeps artifact verification, process creation, protected credential/config delivery, IPC transport selection, packaging, restart/rollback, and production credential storage outside the crate.

The locked `node_lifecycle.rs`, `node_session.rs`, and `sidecar_process.rs` tests were also read as oracle evidence. They cover fresh connection acquisition and token delivery, cancellation/admission fencing, manifest enforcement, duplex ordering/overflow, V3/V4 fallback, authenticated sidecar process crossing, and stable failure/result envelopes. They are upstream contract evidence only; they have not been run locally or accepted as Fabushi Actions evidence.

## 45. Node connection publication and host identity sources (2026-10-29)

The locked `src/node-host/connection.ts`, `gateway-platform-identity.ts`, `host-stats.ts`, `local-id.ts`, and `node-event-params.ts` were read and mapped to the connection responsibility. `connection.ts` is a generation-fenced publication owner: it retires timers and optional publication state on disconnect, publishes host stats and worker/plugin/skill inventory only after Gateway hello, distinguishes legacy unsupported/rejected/transient failures, retries uncertain transport outcomes, and cancels the prepared runtime before successor publication. The smaller sources define platform/model identity projection, bounded host resource sampling, process-stable device-id lookup without minting credentials, and canonical `node.event` payloadJSON framing.

The Rust side currently has only transport-neutral pieces and local projection wiring; it does not yet provide this real Gateway publication caller, worker inventory publication, or remote reconnect path. Status remains implementation evidence with production wiring and acceptance pending.

## 46. Locked node-worker capacity, child, container, environment, and journal sources (2026-10-29)

The locked worker sources `node-worker-capacity.ts`, `node-worker-child-lifecycle.ts`, `node-worker-child-secrets.ts`, `node-worker-container-engine.ts`, `node-worker-container-lifecycle.ts`, `node-worker-environment.ts`, and `node-worker-journal-worker.ts` were read in full and mapped to the existing node-host command-dispatch responsibility as source evidence. Their contracts are separate but jointly required for a production worker owner:

- `node-worker-capacity.ts` gates launches through a durable journal slot, publishes capacity and reclaimable-idle counts, fences expired or aborted claims, recovers pending/running receipts, and prunes terminal history only after the configured retention boundary.
- `node-worker-child-lifecycle.ts` owns physical child custody, process identity, launch/turn admission, descriptor delivery, cancellation, idle retention, process-change publication, observed/deferred terminal outcomes, and bounded SIGTERM/SIGKILL cleanup. It refuses stale owners and does not release a durable launch until child/container cleanup and terminal reconciliation are sufficiently known.
- `node-worker-child-secrets.ts` defines the exact credential values exposed to a physical child for redaction, including provider credentials, Cloudflare access values, GitHub tokens, and native-inference model credentials/headers.
- `node-worker-container-engine.ts` resolves and pins a Docker/Podman daemon identity, creates labeled containers with bundle/workspace mounts and constrained environment, verifies host/gateway/launch ownership labels, treats daemon/context drift as an ownership failure, and lists only containers belonging to the node-host root. `node-worker-container-lifecycle.ts` adds startup cleanup and preserves only matching pending/running receipts; foreign, orphaned, or engine-mismatched containers are not adopted.
- `node-worker-environment.ts` constructs a small platform-specific non-secret environment, normalizes home/path and compile-cache fencing, and always disables child respawn with `OPENCLAW_NO_RESPAWN=1`.
- `node-worker-journal-worker.ts` wraps worker state operations with admission/currentness/settlement tracking, publishes prepared workspace state, and retains an explicit outcome-unknown fence when cleanup or persistence cannot prove a terminal result.

These sources materially narrow the owner contract, but they do not identify a shipping Fabushi worker owner. The existing network enrollment worker, blob-store worker pool, and generic process-tree controller still lack the combined bundle admission, launch plan/hash, environment/session binding, owner epoch, placement generation, durable receipt, container identity, credential scrubbing, and outcome-unknown lifecycle above. No second runtime or sidecar owner was introduced, and no production worker wiring or acceptance flag is claimed. The node-host implementation and exact-head Actions verification therefore remain pending.

## 47. Locked node-worker receipt and recovery sources (2026-10-29)

The locked `node-worker-launch-receipt.ts`, `node-worker-launch-store.ts`, `node-worker-supervisor-ownership.ts`, `node-worker-supervisor-recovery.ts`, and `node-worker-tree-control.ts` were read in full and mapped to the same node-host command-dispatch responsibility. The receipt parser is a strict persistence boundary: launch state, plan hash, supervisor/worker PID plus start time, boot identity, cleanup mode, subreaper lineage facts, and Docker/Podman container identity must round-trip from the canonical state database with bounded formats and cross-field validation. The launch store executes claim observation and claim through the journal worker, rereads the supervisor identity outside SQLite, and exposes only journal-owned operations for nonterminal enumeration, matching, cleanup binding, cancellation, and terminal settlement.

The supervisor ownership model separates environment/session facts that survive a turn from disposable descriptor authority, binds active children to gateway namespace, environment/session, owner epoch, placement generation, bundle hash, workspace and permission mode, and compares both process identity and exact container identity before accepting a receipt as its own. Recovery is idempotently coalesced per launch identity, retains durable capacity when shutdown or cleanup observation is incomplete, detects boot changes, revalidates container ownership, refuses numeric-PID signaling after restart or reuse, and only settles a stale launch after the applicable process lineage/container extinction and journal facts are proven. Tree control uses detached process groups on POSIX, native process-tree handling on Windows, an anchor-only signal path when a recognized owner must preserve lineage evidence, and bounded polling/escalation.

These sources make the missing Fabushi owner boundary stricter: a generic process terminator or existing network/blob worker cannot safely implement this receipt/recovery contract. They remain upstream oracle and mapping evidence only; no process or container recovery path is claimed wired into Fabushi production, and all node-worker verification/acceptance remains pending.

## 48. Locked node-worker supervisor and journal-kernel sources (2026-10-29)

The locked `node-worker-supervisor.ts`, `node-worker-launch-store.kernel.ts`, `node-worker-turn-store.kernel.ts`, `node-worker-launch-observation.ts`, `node-worker-lineage-completion.ts`, `node-worker-journal.worker.ts`, and `node-worker-journal.types.ts` were read in full and mapped to the node-host command-dispatch responsibility.

`NodeWorkerSupervisor` is the top-level admission owner: it validates and hashes a launch, binds it to the current workspace/session/owner epoch, serializes same-environment admissions, initializes container/capacity/journal state, reuses or recovers physical owners, attaches logical turns, propagates cancellation, stops environments, and drains workspace, child, recovery, retention, turn, and journal state before close. `NodeWorkerLaunchKernel` is the durable physical-owner state machine: it enforces schema admission, bounded identifiers/timestamps, capacity, replay plan identity, supervisor boot/process fencing, pending→running transition, cleanup/container metadata, terminal settlement, and bounded terminal pruning in the canonical state database. `NodeWorkerTurnKernel` keeps turn receipts separate from the physical launch, rejects owner/session/generation/process mismatches, settles active turns when the launch terminates, and retains terminal outcomes only for a bounded period.

`observeNodeWorkerChild` keeps stdout framing and stderr diagnostics bounded, waits for journal readiness before consuming result frames, sanitizes credentials, distinguishes incomplete output from a valid result, and returns deferred rather than terminal success whenever process/container extinction is uncertain. `node-worker-lineage-completion.ts` records positive anchor-lineage or Linux-subreaper descendant evidence only from the admitted live owner with matching database binding. `node-worker-journal.worker.ts` exposes these kernels through the existing SQLite worker-operation admission and state write transaction, alongside prepared-workspace operations; it is not a parallel state store.

This source cluster confirms the required producer→owner→canonical state→consumer chain for a real node worker, but the corresponding Fabushi producer, Rust owner, physical process/container executor, and exact-head Actions oracle are still absent. The mapping reduces the audited unread/unclassified frontier only; it does not change any global completion flag or claim worker production acceptance.

## 49. Locked node-worker workspace custody sources (2026-10-29)

The locked `node-worker-workspace.ts`, `node-worker-workspace-admission.ts`, `node-worker-workspace-identity.ts`, `node-worker-workspace-quiescence.ts`, `node-worker-workspace-processes.ts`, `node-worker-workspace-replacement.ts`, `node-worker-workspace-retention.ts`, and `node-worker-workspace-seeds.ts` were read in full and mapped to the node-host command-dispatch responsibility.

`NodeWorkerWorkspaceRuntime` is the custody boundary for managed and device-folder workspaces. It creates private roots, validates device-folder permissions, binds prepared workspaces to the durable environment/session/ownerEpoch/session-key identity, serializes operations by session root, rejects deleting generations, and dispatches drain, reset, seed, transfer, inspection, manifest capture, quiescence, and process operations only after path and generation checks. Retention snapshots combine controller sequence/signature, retained generations, live nonterminal launches, active operations, transfer manifests, and deletion fencing before removing generations or manifests. Seed import/store is keyed and bounded, excludes unsafe Git alternates, and refreshes/prunes machine-local Git object seeds under private roots.

`node-worker-workspace-identity.ts` requires canonical realpaths, strict gateway/hash/generation layout, private directory modes, and argv path containment; it never treats an arbitrary cwd as node placement authority. `NodeWorkerWorkspaceProcesses` keeps preview processes alive across tool calls, fingerprints process IDs to commands, caps long-lived processes, bounds output, binds cleanup to the process supervisor, and refuses environment retirement until process-tree extinction is confirmed. `NodeWorkerWorkspaceQuiescence` owns a separate watchdog lease for preview/environment quiescence, serializes acquire/renew/release controls by nonce, reuses only matching idle helpers, and retains workspace custody when control or cleanup is uncertain. Atomic workspace/bundle replacement uses a rollbackable staging/backup commit point; retention enumerates only validated owned session paths.

These sources identify a workspace-custody contract that is broader than Fabushi’s generic workspace or process helpers. No existing Fabushi owner has yet been proven to provide the combined placement identity, prepared-workspace binding, retain/delete fencing, preview-process lifetime, quiescence watchdog, seed isolation, and transfer rollback semantics. The Rust implementation, production producer/consumer path, and exact-head Actions acceptance remain pending.

## 50. Locked prepared-workspace and transfer sources (2026-10-29)

The locked `node-worker-prepared-workspace.ts`, `node-worker-prepared-workspace-store.ts`, `node-worker-prepared-workspace-store.kernel.ts`, `node-worker-prepared-workspace-transfer.ts`, `node-worker-transfer-client.ts`, `node-worker-transfer-http.ts`, `node-worker-device-folder.ts`, and `node-worker-workspace-commands.ts` were read in full and mapped to the node-host command-dispatch responsibility.

Prepared workspaces are a durable one-node registration and one-session binding: registration verifies source/prepared manifests against the immutable base commit before publication, binding consumes the available row exactly once, mutation creates a retiring tombstone, and completion is the only path that reopens it. The kernel uses the existing node journal and commit receipts; it rejects a second environment or changed row rather than silently adopting a path. Collection fences the generation before filesystem removal and rechecks protection against live launches/operations.

Workspace transfer is staged and manifest-driven. Downloads validate response status, gzip expansion, manifest/path eligibility, blob byte counts and SHA-256, Git base/seed integrity, attachment roots, and the final captured manifest before the atomic workspace replacement; prepared overlays read only immutable baseline blobs and complete a journaled mutation. Uploads capture a base/current delta, snapshot only changed files, stream a bounded framed reconcile request with backpressure, and accept only an acknowledgement naming the captured manifest. The HTTP adapter converts the Gateway WebSocket endpoint to HTTP(S), requires TLS for Cloudflare Access, optionally pins the peer certificate, sends bearer/access headers, bounds response bodies, and does not return while a request body remains incomplete.

Device-folder access is fresh consent plus symlink, root-overlap, canonical-path, device/inode, and bound-session validation. Manifest command helpers pin HOME and disable interactive Git/credential prompts, preserve bounded hash memo state, and expose only the exact built-in capture argv. These sources strengthen the required producer→owner→storage→consumer contract, but no Fabushi owner or Rust production path currently proves equivalent prepared-workspace/transfer behavior; verification and acceptance remain pending.

## 51. Locked node-worker protocol and admission schemas (2026-10-29)

The locked worker contract sources `node-workspace-protocol.ts`, `node-workspace-prepared-protocol.ts`, `node-workspace-retain-protocol.ts`, `node-workspace-transfer-protocol.ts`, `node-workspace-transfer-body.ts`, `node-bundle-install-protocol.ts`, `protocol-record.ts`, `worker-process-protocol.ts`, `worker-process-observation.ts`, and `node-supervisor-protocol.ts` were read in full and mapped to the node-host command-dispatch responsibility.

The shared protocol primitive rejects malformed JSON, oversized payloads, unknown own keys, unbounded identifiers, and unsafe gateway namespace components before domain parsing. Workspace requests bound argv/input/timeout/output, make device-folder, prepared, transfer, seed, process, reset, drain, inspection, and quiescence operations mutually explicit, and require exact operation-specific shapes. Retain requests normalize and de-duplicate generation/manifest/bundle sets while binding acknowledged bundle status to the retained set. Transfer contracts distinguish download/upload stages, attachment/checkpoint/seed restrictions, bounded manifest refs and tokens, and typed invalid reasons; bundle installation binds a short-lived archive token, exact SHA-256/size, namespace, prewarm version, and admission handshake.

The process protocol is a negotiated JSONL boundary: turn descriptors must match turn IDs and expected bundle hashes, optional idle/canonical-workspace/process-change markers require the advertised feature, results are restricted to bounded completed/failed/fenced/deadline envelopes, and observation replies must contain exactly one result or error. Supervisor identities include launch/plan/environment/session/ownerEpoch/placement/run IDs; plan hashing is stable over the descriptor and all admission-affecting fields, while result/error diagnostics have independent byte and newline limits.

These schemas are direct upstream oracle contracts for Rust parsing and negative tests. They do not imply that Fabushi has an equivalent production transport or worker owner; current Rust wiring and exact-head Actions verification remain pending.

## 52. Locked worker bundle and repository transfer sources (2026-10-29)

The locked `node-worker-bundle-installer.ts`, `node-worker-upload-snapshot.ts`, `node-worker-repository-transfers.ts`, and `node-worker-workspace-git.ts` were read in full and mapped to the node-host command-dispatch responsibility.

`NodeWorkerBundleInstaller` serializes every namespace install, validates local or Gateway archive bytes with bounded Range-resume and transient retry handling, verifies exact size/SHA-256, extracts under the archive limits, writes a durable admission receipt, atomically publishes the bundle, revalidates its manifest/entry path, optionally prewarms the exact build, and retains/deletes only validated namespace-owned bundle generations. It treats malformed receipts, unsafe files, unexpected ranges, no-progress retries, TLS/Access failures, and staging residue as missing or failed rather than trusted.

Upload snapshots copy and hash each changed source under a private temporary root before request streaming, so later workspace writes cannot alter the bytes being reconciled. Git initialization fixes SHA-1/SHA-256 object format from the pinned base, imports only the admitted pack, verifies the exact commit, isolates the worker branch/HEAD, skips gitlinks, and checks out only manifest-eligible paths. Repository checkpoints require the same immutable base and conflict-free manifest; publication starts from the explicit empty manifest and a dedicated snapshot directory.

These sources define the artifact and repository compatibility boundary for worker hosting, but Fabushi still lacks a proven Rust bundle installer, Gateway transfer caller, durable bundle-retention consumer, and exact-head Actions/package acceptance. The mapping is not a production-equivalence claim.

## 53. Locked node-host runner and app-worker entry sources (2026-10-29)

The locked `src/node-host/runner.ts`, `worker.ts`, `worker-support.ts`, `runtime-skill-bins.ts`, and `startup-state-readiness.ts` were read in full and mapped to the node-host connection/lifecycle responsibility. The headless runner loads configuration and device identity, distinguishes explicit local/remote credentials from paired-device fallback, prepares the runtime, advertises the platform/capability/path environment, dispatches Gateway invoke/input/cancel events, refreshes inventory after pairing, persists the winning endpoint, applies reconnect pause and proxy-rejection policy, starts launcher readiness and auto-update only after connection, and aggregates client/runtime shutdown errors.

The macOS app worker is a private JSONL bridge around the same runtime. It binds request/response state to a connection generation, ignores stale frames, routes `node.invoke.result` and `node.event` through their dedicated paths, closes pending requests on route changes, and stops the runtime/client together on EOF or signal. `SkillBinsCache` is a bounded TTL trust-inventory snapshot that preserves the previous value on refresh failure; startup readiness delegates to the credential-preparation owner.

These are real entrypoint and lifecycle sources, not a substitute for the command executor. Fabushi has no accepted Rust equivalent for the complete Gateway runner/worker lifecycle, and no exact-head Actions, package, or real-device acceptance is claimed. The mapping reduces the audited unread/unclassified frontier only; all global completion flags remain pending.

## 54. Locked node-host update, launcher, macOS entry, and skill sources (2026-10-29)

The locked `auto-update-compatibility.ts`, `auto-update.ts`, `auto-update-install.ts`, `launcher-bootstrap.ts`, `launcher-client.ts`, `mac-worker-entry.ts`, and `skills.ts` were read in full and mapped to the node-host connection/lifecycle responsibility. Update compatibility validates the candidate package layout and exact state/agent schema versions, rechecks runtime and copied database preflight after idle admission, and fails closed on migration, malformed reports, indeterminate state, or cleanup failure. Installation stages an exact semver into a private immutable generation, verifies registry integrity and package contents, fences the update executor, never overwrites a retained generation, and retries only in a fresh generation after invalid residue.

The launcher bootstrap resolves profile-scoped state without loading runtime state; launcher IPC is connected-child-only, acknowledgement-bounded, and retires the node on parent stdin EOF. The sealed macOS worker accepts only `node worker`, applies the CLI/runtime/plugin cleanup scope, routes machine output safely, and owns the worker lifetime. Node skill discovery enforces canonical in-root paths, matching directory/frontmatter names, bounded descriptor size/count/total bytes, and deterministic ordering.

These sources clarify update and entrypoint ownership but do not establish a Fabushi launcher, private-generation updater, skill producer, or exact-head Actions/package/device acceptance. They remain upstream mapping evidence; global completion flags stay false.

## 55. Read-ledger promotion guard (2026-10-29)

The source-read bookkeeping tool now requires every new `read-complete` or responsibility update to originate in its explicit bounded `READS` table. Existing parity-ledger `source_refs` may reconcile an already explicit file's responsibility and evidence metadata, but a reverse edge alone cannot authorize semantic reading, a full-file range, native-equivalent disposition, or a reviewer assertion. This guard is evidence hygiene only; the existing historical rows outside the current explicit table remain a separate audit backlog and are not treated as newly revalidated by this change.

## 56. Locked node-host invoke execution and result adapters (2026-10-29)

The locked `invoke-system-run.ts`, `invoke-run-command.ts`, `invoke-device-apps.ts`, `invoke-mcp-result.ts`, and `invoke-plugin-context.ts` were read in full and mapped to the node-host command-dispatch responsibility. `handleSystemRunInvoke` is a three-phase admission/policy/execute pipeline: it parses and binds command/cwd/env/session/run identity, evaluates approvals/allowlists/auto-review and shell-wrapper policy, binds executable/cwd/mutable-file snapshots, revalidates them after durable authorization, routes macOS execution through the authenticated companion socket when required, and otherwise executes with revocation checks and bounded result events. It fails closed on policy drift, persistence errors, approval mismatch, permission absence, or plan mismatch.

The local runner preserves cwd failure diagnostics, Windows `cmd.exe` argument semantics, login-shell PATH restoration, process-tree termination, bounded head output, timeout and signal results. Installed-app discovery requires explicit sharing consent and bounded strict parameters. MCP results remove mirrored text, cap text at 1 MiB and the entire invoke payload at 20 MiB with UTF-8-safe markers. Plugin invocation context binds workspace capability acquisition to the session and abort lifetime and releases an acquisition if authority closes.

These are direct command-dispatch producer/consumer contracts, not proof of an equivalent Rust executor or production companion socket. The Rust owner currently has only the previously mapped validation/runtime seam; exact-head Actions, real permission/service/device, and release acceptance remain pending.

## 57. Locked node-host manifest, connection configuration, and progress sources (2026-10-29)

The locked `runtime-manifest.ts`, `gateway-candidate-connection.ts`, `config.ts`, `config-shared.ts`, `gateway-cloudflare-access.ts`, `exec-policy.ts`, and `node-invoke-progress.ts` were read in full and explicitly bound to their existing ledger responsibilities. The manifest derives deterministic capability/command/inventory projections from plugin registration and feature gates; candidate connection owns pre-hello failover, winner promotion, manifest propagation, and aggregated cleanup. Configuration reads only canonical SQLite state, rejects retired file state, normalizes gateway/command data, and writes through the existing journal owner.

Cloudflare Access credentials remain SecretInput references, require paired values, are materialized only at connection time, and are registered for redaction; origin matching includes protocol and host/port. Exec policy layers global and agent config and fails closed for denied, allowlist-miss, Windows shell-wrapper, and approval-required cases. Progress writing serializes UTF-8-safe 16 KiB chunks, bounded heartbeats, sequence numbers, aborts, and error propagation.

This closes the source-reading evidence for these seven production seams only. It does not prove Rust parity, remote Gateway wiring, permission/device acceptance, or exact-head Actions verification; those states remain pending.

## 58. Locked node-host runtime composition source (2026-10-29)

The locked `src/node-host/runtime.ts` was read in full and explicitly bound to the connection/runtime owner. `prepareNodeHostRuntime` composes config, plugin registration, path and feature gates, skills, MCP, desktop host policy, worker workspace/container supervision, bundle installation, native inference, manifest and inventory. Its attached runtime admits invocations only after generation and disconnect-cleanup barriers, fences duplicate IDs, binds abort/input/progress/duplex state, forwards the canonical Gateway/permission/worker context into `handleInvoke`, and joins watcher, MCP, supervisor, worker, disconnect, and invocation tails before close.

The source also makes the ownership boundary explicit: update pause is admission control across local/MCP/registered-command/worker work, while worker state, workspace custody, bundle lifecycle, plugin availability, and Gateway publication remain separate owners. The Rust runtime currently covers only the previously implemented local session/invocation projection, not this complete production composition or packaged consumer path; verification and acceptance remain pending.

## 59. Locked node-host plugin registry and MCP manager sources (2026-10-29)

The locked `plugin-node-host.ts` and `mcp.ts` were read in full and explicitly bound to the node-host command-dispatch responsibility. The plugin registry loads a fresh runtime-scoped registry, runs filtered preparation before manifest publication, deterministically projects capabilities/commands/tools, watches availability, drains owner callbacks on disconnect, and refuses invocation after registry/plugin/session/approval authority changes. Duplex commands receive the bounded IO/context seam; local exec policy is re-read and revalidated rather than overridden by Gateway approval.

The MCP manager owns process-lifetime sessions for enabled non-OAuth servers, bounded concurrent connection admission, stable descriptor naming/catalog limits, schema validation, tool filters, list-change refresh, exponential reconnect, session-expiry invalidation, typed unavailable/timeout/tool errors, and close-time work/catalog draining. These sources clarify the producer→registry/MCP owner→manifest/inventory→invoke consumer chain, but do not prove a Rust plugin/MCP equivalent, real service acceptance, or exact-head Actions evidence.

## 60. Locked node-host system-run plan, allowlist, and denial sources (2026-10-29)

The locked `invoke-system-run-allowlist.ts`, `invoke-system-run-plan.ts`, and `invoke-system-run-denied.ts` were read in full and bound to command-dispatch security. Allowlist evaluation distinguishes shell and argv analysis, safe-bin/skill trust, durable authorization, Windows wrapper semantics, and planned argv replacement; POSIX wrapper rewrites are reconstructed only from a validated authorization plan. Approval-plan preparation rejects unsafe wrappers/env overrides, captures canonical cwd/executable/mutable-file bindings, and preserves the exact command/session/agent identity. Denial mapping emits `exec.denied` plus bounded transport/permission error envelopes without conflating companion loss or timeout with policy denial.

These are security-contract evidence for the existing Rust command-dispatch owner, not a claim that Rust has equivalent execution, approval persistence, or packaged macOS companion wiring. Exact-head Actions and real permission/device acceptance remain pending.

## 61. Locked node-host invoke payload, input, portal, and launch transport sources (2026-10-29)

The locked `invoke-payload.ts`, `invoke-types.ts`, `node-invoke-input.ts`, `portal-stream-command.ts`, and `node-worker-launch-transport.ts` were read in full and explicitly bound to the command-dispatch/worker transport responsibility. Invoke coercion validates identity and bounded UTF-8 input; input dispatch preserves sequence fencing and a fail-closed 64 KiB pre-spawn buffer. Portal streams require exact ticket/attach-path/port shapes and route only through the enrolled Gateway transport.

Worker launch transport resolves a namespace- and bundle-hash-pinned entry, restricts local inference secrets to the trusted snapshot, selects one process-group/relay or container transport, rejects incompatible loopback/browser/container combinations, binds cleanup to the durable launch store, and serializes stdin through the negotiated worker protocol. This is direct upstream transport evidence; it does not prove the corresponding Fabushi process/container executor or exact-head packaged acceptance.

## 62. Locked node-host desktop, PTY, and Claude command sources (2026-10-29)

The locked `desktop-stream-command.ts`, `pty-command.ts`, `claude-skill-session.ts`, `invoke-agent-cli-claude-handler.ts`, and `invoke-agent-cli-claude-params.ts` were read in full and explicitly bound to command dispatch. Desktop streaming gates platform/ephemeral policy, probes authenticated loopback RFB, bounds and redacts password material, and splices only the ticket-bound Gateway stream. PTY handling validates cwd/dimensions/parameters, merges the node environment, preserves abort/currentness fencing, serializes output backpressure, and handles input/resize races.

Claude node execution keeps the approval boundary around all caller-supplied executable arguments, allows only a narrow bounded argument/env surface, rejects bypass permissions and conflicting credentials, passes secrets through descriptor input with scrubbing, and exposes only verified skill resources plus an invocation-scoped MCP Workshop route. Session teardown revokes pending calls, closes MCP servers, removes temporary artifacts, and refuses stale duplex frames. These are upstream security and capability contracts; Fabushi Rust execution, native PTY/desktop/Claude consumers, and exact-head acceptance remain pending.

## 63. Locked node-host Claude execution and worker observation sources (2026-10-29)

The locked `invoke-agent-cli-claude.ts`, `node-worker-native-inference.ts`, `node-worker-output.ts`, `node-worker-process-identity.ts`, and `node-worker-process-observation.ts` were read in full and bound to command-dispatch/worker execution. Claude execution streams bounded UTF-8 output and terminal result lines through the invocation progress owner, uses the process supervisor for hard/idle timeout and cancellation, injects only descriptor-backed secrets, and waits for descendant extinction before removing prompt/skill artifacts. Native inference snapshots only configured, usable, schema-valid models into a node-owned workspace-bound startup payload; credentials remain outside child argv.

Worker output is bounded and credential-redacted, process identity is PID/start-time/boot conservative, and observation requests are correlated to the physical owner with journal readiness, placement/bundle/environment fencing, a bounded pending set, timeout, abort, and stale-owner rejection. These sources narrow the required Rust owner contract but do not prove a native Fabushi Claude/worker executor or packaged acceptance.

## 64. Locked node-host worker control, turn journal, schema, workspace publication, and timeout sources (2026-10-29)

The locked `node-worker-supervisor-commands.ts`, `node-worker-turn-settlement.worker.ts`, `node-worker-turn-store.ts`, `node-worker-launch-schema.ts`, `node-worker-prepared-workspace-publication.ts`, and `with-timeout.ts` were read in full and explicitly bound to the command-dispatch/worker owner. The supervisor command surface dispatches workspace preparation/execution/retention, bundle installation, desktop and portal streams, desktop launch, process observation, environment stop, and launch/status/cancel controls; it forms the worker ingress endpoint, preserves gateway TLS/Cloudflare context, returns bounded generic errors, and never replays a failed mutation.

The turn settlement worker closes only unfinished turns for the exact physical launch and converts a completed physical launch to `interrupted` rather than fabricating a result. The turn store delegates claim/get/finish to the journal worker and rechecks the full supervisor identity before exposing a receipt. Launch schema selection is engine-aware and uses the admitted state-schema owner; prepared-workspace publication validates the durable row shape before projecting it. The timeout helper bounds each re-armed window by a total deadline and propagates abort reasons with dynamic stage labels.

These sources establish the remaining upstream producer→journal/schema/publication→consumer contracts for this cluster. They do not prove a Rust equivalent, remote worker executor, durable production wiring, or exact-head Actions/device/package acceptance; all such states remain pending.

## 65. Locked node-host supervisor receipt contract source (2026-10-29)

The locked `node-worker-supervisor-contract.ts` was read in full and explicitly bound to command dispatch. It narrows the public supervisor control surface to launch/status/workspace-retention/cancel/environment-stop/process-observation operations and projects only identity-fenced durable launch receipts. Completed receipts expose the recorded result, failed/interrupted/cancelled receipts expose bounded error state, and non-terminal receipts expose neither fabricated result nor terminal error; the projected shape is parsed again and rejects inconsistent durable state.

This closes the upstream receipt projection contract for the cluster. It does not prove the Fabushi Rust supervisor implementation, durable remote worker execution, or exact-head Actions and packaged acceptance.

## 66. Locked worker Gateway connection admission and frame transport sources (2026-10-29)

The locked `src/worker/worker-connection-admission.ts`, `worker-connection-contract.ts`, `worker-connection-endpoint.ts`, `worker-connection-frames.ts`, and `worker-node-loss.ts` were read in full and explicitly bound to the node-host connection/command-dispatch responsibilities. Endpoint parsing accepts only bounded Unix or worker-ingress WebSocket targets, rejects credentials in URLs and unsafe path/query forms, requires secure transport for TLS pins or Cloudflare Access, and delegates transport pin/header construction to the existing Gateway owner. Admission sends one version-fenced connect request, validates the matching response identity and protocol-feature set, classifies retryable versus fenced/terminal closes, bounds the handshake, and suppresses stale-generation frames.

The connection state contract distinguishes idle, connecting, admitting, ready, reconnecting, fenced, failed, and stopped; errors redact endpoint and credential material before bounded diagnostics. The frame dispatcher owns typed request/response correlation, payload budgets, pending timeout interruption, transcript/gateway-tool/inference validation, session/epoch identity fences, and invalid-frame closure. `WorkerNodeLostError` deliberately marks non-idempotent tool effects as outcome-unknown and records the model fallback stop rather than authorizing an automatic replay.

This establishes the upstream connection producer→admission owner→typed frame consumer and node-loss safety boundary. It does not prove the corresponding Fabushi Rust socket, Gateway service, inference/tool consumer, or exact-head Actions/device/package acceptance.

## 67. Locked worker connection state machine and RPC consumer sources (2026-10-29)

The locked `src/worker/worker-connection.ts`, `worker-rpc-client-shared.ts`, `worker-rpc-inference-client.ts`, `worker-rpc-live-event-client.ts`, and `worker-rpc-transcript-client.ts` were read in full and explicitly bound to the node-host connection/command-dispatch responsibilities. `WorkerConnection` owns one generation-fenced lifecycle: bounded admission and reconnect backoff, readiness/terminal promises, heartbeat and owner-epoch fencing, replay only for Gateway-tool operations whose owner retains settlement, and no automatic replay for desktop input. Shutdown fences ownership before callbacks and socket cleanup, rejects pending work, and preserves the first terminal exit.

The RPC clients are distinct consumers over that single connection. Inference keys operations by session/run/turn, resumes only with identity and sequence checks, reports stream gaps, and settles exactly once. Live events maintain bounded preview buffering, cumulative ACKs, resync cursors and terminal delivery; preview failure degrades without losing the authoritative terminal path. Transcript commits fit batches to the protocol budget, advance the base leaf and sequence only on accepted results, and make stale-base or non-retryable rejection terminal for the turn. Ownership errors fence the connection rather than replaying against a replacement owner.

This closes the worker connection state-machine and RPC producer→owner→consumer contract for the read cluster. It does not prove the Fabushi Rust implementation, Gateway/inference/transcript production wiring, or exact-head Actions and real platform/package acceptance.

## 68. Locked worker computer, message projection, and device-folder boundary sources (2026-10-29)

The locked `assistant-message-projection.ts`, `computer-runtime.ts`, `device-folder.ts`, and `node-computer-protocol.ts` were read in full and explicitly bound to command dispatch. Assistant projection removes provider scratch fields and emits the closed worker text/image/thinking/tool-call and usage shapes, preserving only approved signatures and execution metadata. The computer runtime binds every request to the launch descriptor's node and run identity, validates the typed protocol budget, serializes close exactly once, aborts by fencing pending input and issuing a native close, and delegates actual side effects to the existing registered Computer Use provider through the established transport.

The private computer protocol permits only capability, snapshot, act, and execution-close operations with bounded JSON, provider-generation fencing, and execution ownership; it cannot carry arbitrary node commands. A device folder is explicitly live user data rather than a Gateway synchronization snapshot, so its manifest sentinel must not be treated as remote state. This establishes the worker capability producer→existing Computer owner→typed consumer boundary, but does not prove Fabushi Rust Computer wiring, UI/device acceptance, or exact-head Actions/package evidence.

## 69. Locked worker native-inference runtime and stream-safety sources (2026-10-29)

The locked `native-inference-startup.ts`, `native-inference-stream.ts`, `native-runtime-config.ts`, `native-runtime-model-support.ts`, `native-runtime-transport.ts`, and `native-runtime.ts` were read in full and explicitly bound to command-dispatch/worker execution. Startup drains a bounded private descriptor pipe before the worker start gate, zeroes and closes the FD, and checks the admitted model and canonical workspace. Configuration admits only strict, unique model records with bounded identifiers, explicit endpoint/header/cost/input metadata, and no turn-wire configuration surface; model support rejects ambient Azure/Vertex credential modes.

The runtime owns provider I/O in a dedicated process, uses the existing AI registry without importing the Gateway stream facade, binds one exact model/workspace/session per turn, rejects workspace inode/path drift, propagates cancellation, clears credentials and headers on close, and detects credential material or prefixes before protocol publication. Transport permits HTTPS or literal loopback HTTP only, strips credential-bearing URL forms, rejects query/hash/placeholders, pins one origin, and disables redirects. The stream guard bounds tool arguments and aggregate payloads, holds credential-prefix fragments until safe, rejects post-terminal output, and emits fixed credential-free failure events. These contracts are upstream evidence for a native inference owner; they do not prove Fabushi Rust provider wiring, real provider acceptance, or exact-head Actions/package evidence.

## 70. Locked worker launch, process, placement, and workspace inspection sources (2026-10-29)

The locked `launch-descriptor.ts`, `worker-process.ts`, `worker-placement-tools.ts`, `workspace-inspection-protocol.ts`, and `workspace-inspection.ts` were read in full and explicitly bound to command dispatch/worker execution. The launch descriptor is a versioned, closed admission contract: it validates exact connect identity, model/inference mode, transcript/live-event cursors, permission and containment roots, tool authority, browser/Computer/GitHub bindings, and endpoint; legacy absence remains restrictive and host-specific approvals are not reconstructed in the worker.

The worker process accepts only a strict internal start message, binds inherited lineage/native-owner descriptors, keeps diagnostics off the result stream, reports connection failure through the owned IPC channel, and terminates only its owned process tree. Placement constructs reuse the canonical coding-tool owner while disabling plugins and non-portable host execution/approval state. Workspace inspection accepts bounded list/get/download/set/diff operations, preserves session-root ownership, uses optimistic expected hashes, bounds encoded results, and truncates oversized diff patches without dropping changed-file statistics. These sources define the worker producer→existing tool/workspace owners→consumer boundaries; they do not prove Rust production wiring, packaged process reachability, or exact-head Actions/device/package acceptance.

## 71. Locked worker transcript projection and inference stream adapter sources (2026-10-29)

The locked `transcript-message.ts` and `inference-stream.runtime.ts` were read in full and explicitly bound to command-dispatch/worker execution. Transcript projection converts user, assistant, tool-result, and permitted runtime-context messages into the closed worker schema, preserves only approved provider replay fields within both replay and commit-frame budgets, redacts diagnostics/errors, and returns an explicit local-retry outcome when authoritative replay cannot be carried safely. Frame safety is measured using the exact protocol envelope rather than content length alone.

The inference adapter fits replay images to the request budget, invalidates stale Computer frames, fences every stream by session/run/turn epoch, reconstructs text/thinking/tool-call state, tolerates only declared stream gaps, and makes cancellation/remote cancellation terminal. It rejects malformed tool state, oversized transcript results, and late events; terminal errors are projected into bounded safe assistant messages. This closes the worker inference producer→stream adapter→existing agent-loop consumer boundary, but does not prove Fabushi Rust inference wiring, provider acceptance, or exact-head Actions/package evidence.

## 72. Locked embedded-agent transcript and live-event runtime sources (2026-10-29)

The locked `embedded-agent-transcript.runtime.ts` and `embedded-agent-live.runtime.ts` were read in full and explicitly bound to command-dispatch/worker execution. Transcript runtime projects the canonical session context into bounded worker inference messages, rejects media-bearing runtime-context markers it cannot preserve, windows replay history, serializes assistant/tool/user messages with diagnostic redaction, and flushes transcript commits behind the existing session-write settlement queue. A failed submitted commit remains outcome-known and is not silently retried; aborted unsubmitted output may stop without fabricating durability.

Live runtime emits bounded lifecycle, assistant, thinking, and tool previews with UTF-8-safe truncation, 75ms pacing, semantic phase handling and assistant occurrence identity. Preview failure degrades only the preview path; terminal lifecycle is merged through the canonical attempt outcome, held until transcript durability, marked replay-invalid on model fallback cleanup failures, and sent as the authoritative terminal event. These sources close the agent-session producer→transcript/live owners→Gateway consumer ordering contract, but do not prove Fabushi Rust agent-loop integration, production Gateway wiring, or exact-head Actions/package acceptance.

## 73. Locked embedded worker agent runtime source (2026-10-29)

The locked `embedded-agent.runtime.ts` was read in full and explicitly bound to command-dispatch/worker execution. `runWorkerEmbeddedTurn` is the worker-side composition boundary: it materializes skill resources, checks browser/Computer/tool authority and run identity, selects the admitted model/inference mode, builds in-memory session/auth/settings/resource owners, restores initial messages, and routes all persistence through the canonical transcript settlement queue. Placement tools, browser/Computer transports, Gateway tool proxies, abort wrappers, caller identity, plugin metadata, and tool-surface compaction are assembled into one existing agent session rather than a second agent loop.

The runtime chooses native inference only through the guarded node-local stream; remote inference first projects bounded replay/context and uses the worker RPC adapter. It orders prompt, runtime-context, subscription/live events, cleanup, transcript flush, terminal live ACK, and session disposal; cleanup failures become model-fallback failures, submitted transcript failures remain visible, and promoted background processes remain owned by the enclosing environment. This source closes the worker producer→canonical agent/session/tool/transcript/live consumers path at the upstream level, but does not prove Fabushi Rust agent-loop implementation, real production reachability, or exact-head Actions/service/device/package acceptance.

## 74. Locked worker command shell and runtime reachability sources (2026-10-29)

The locked `worker-command.runtime.ts` and `worker.runtime.ts` were read in full and explicitly bound to command dispatch/worker execution. The command shell accepts either one bounded launch descriptor or a managed line-framed process protocol, binds native-inference startup only through the private FD carrier, owns signals/IPC lifetime, preserves retained background-process observation and stop operations, rejects duplicate turn execution, fences canonical workspace identity, and reports results only after retention/profile cleanup rules settle.

The runtime creates a 0700 isolated state directory and scoped process-supervisor cleanup, restores ambient state only after database/finalizer completion, validates containment and native inference assignment, registers/redacts Cloudflare credentials, connects through the single worker connection owner, and constructs the transcript/live/inference clients. It distinguishes admission deadline, fencing, node loss, caller cancellation, turn failure, and completed terminal ACK; GitHub refresh is retired after the terminal fence, native runtime credentials are closed, clients are disposed, and connection/environment cleanup is awaited. This closes the upstream process→runtime→agent/session/transport consumer path, but does not prove the Fabushi Rust packaged entrypoint, real Gateway/provider/device wiring, or exact-head Actions/package acceptance.

## 75. Locked worker GitHub binding, sidecar entries, build identity, and replay-window sources (2026-10-29)

The locked `github-binding.runtime.ts`, `memory-worker-entry.ts`, `skills-worker-entry.ts`, `worker-build-identity.ts`, and `replay-message-window.ts` were read in full and explicitly bound to command-dispatch/connection responsibilities. GitHub binding creates turn-scoped owner-only credential profiles, clears inherited helpers, redacts tokens, reconciles only the admitted checkout origin/branch with fast-forward-only behavior, refreshes expiring credentials behind heartbeat/currentness fences, and removes the profile on turn disposal; checkout metadata failures do not discard the coding turn.

Memory and skills worker entries keep stdout reserved for their typed IPC result, load only the approved bundled artifact/worker owner, validate bounded arguments, and drain singleton/watch lifecycle state on exit. Build identity requires the execution-context and authority protocol features and compares bundle/version/features as an unordered protocol set. Replay-window logic refuses to cut through provider replay ownership, preserves complete user turns, fits only model-projection images, protects the active Computer tool result, and returns explicit local-retry/unavailable outcomes when the opaque replay unit cannot safely be changed. These sources do not prove Fabushi Rust GitHub/memory/skills production consumers, provider/device acceptance, or exact-head Actions/package evidence.
## 76. Locked worker Gateway-tool proxy and workspace receiver/retention sources (2026-10-29)

The locked `worker-gateway-tools.ts`, `workspace-rsync-receiver.ts`, and `workspace-seed-retention.ts` were read in full and explicitly bound to command-dispatch/worker execution. Gateway-tool proxies expose only entries admitted for Gateway execution, validate object arguments, derive bounded timeout budgets, bind cancellation to the retained Gateway connection, preserve replay and update callbacks, reject unsuccessful responses, and retain plugin metadata and before/after ownership hooks. The proxy is an adapter over the existing Gateway tool owner, not a second tool executor.

The workspace rsync receiver accepts only three closed modes, canonical base64url context, a fixed destination marker, and a nonce; it restores an attested destination before invoking the shared locked remote receiver and uses a separate accepted-next staging root. Seed retention uses a single host-independent artifact source for pruning, keeps the just-prepared seed, distinguishes temporary seeds from durable hashes, and bounds both age and entry count. These sources establish upstream Gateway-tool and workspace custody contracts, but do not prove Fabushi Rust proxy/transfer wiring, storage ownership, package reachability, or exact-head Actions/device/provider acceptance.

## 77. Locked worker deployment entrypoints and sealed runtime registry sources (2026-10-29)

The locked worker deployment entrypoints were read in full and explicitly bound to command-dispatch/worker execution. The deploy entry validates the closed internal argument set, requires native inference to be managed, supports prewarm through the admitted turn runtime, and otherwise starts the existing worker process with the composed Browser runtime; startup errors stay on stderr and input/IPC cleanup runs on every exit. The Browser deploy placeholder fails closed when composition is absent rather than inventing a provider.

The deployment modules register sealed runtime entrypoints for client credentials, code mode, state reads, native lifecycle, file tools, Gateway heartbeat, GitHub execution, image processing, service relay, SQLite source revision/store, and shared state. Thin entries load the sealed bootstrap before the existing worker owner and preserve SQLite backend exports through the canonical state implementation. These sources define package/runtime entrypoint reachability and owner boundaries upstream, but do not prove Fabushi Rust packaging, installed bundle execution, provider/device acceptance, or exact-head Actions evidence.

## 78. Locked worker Gateway-proxy, deploy-entry, and reconnect-race oracle sources (2026-10-29)

The locked `worker-gateway-tools.test.ts`, `worker-deploy-entry.test.ts`, and `repro-worker-connection-closing-window.ts` were read in full and recorded as test-oracle evidence rather than product runtime implementation. The Gateway proxy oracle checks canonical tool-definition preservation, replay/update forwarding, timeout derivation, exact invocation cancellation, late-result rejection, object-argument validation, remote failure projection, and output-contract rejection through Tool Search. The deploy-entry oracle checks runtime/prewarm/argument/worker failures remain sanitized on stderr, process transport cleanup occurs exactly once, prewarm flush ordering is preserved, and live execution retains transport until settlement.

The reconnect-race reproducer creates the locked worker admission handshake, observes the first socket close, issues the same replayable Gateway-tool invocation during the close window, accepts only the admitted response, then stops the owned connection/server. It is an executable upstream oracle for close/reconnect behavior and does not constitute Rust implementation or local acceptance. All three rows use `test-oracle-replaced` disposition with their source refs retained under the existing connection/command-dispatch responsibilities; no test result or mock is promoted to production wiring.

## 79. Locked worker process admission, protocol, and retained-process oracle sources (2026-10-29)

The locked `worker-process.test.ts`, `worker-process-protocol.test.ts`, and `worker-process-observation.runtime.test.ts` were read in full and recorded as `test-oracle-replaced`. The process admission oracle proves SQLite capability initialization settles before the restricted worker accepts commands. The protocol oracle preserves the closed result/retention matrix, exact idle readiness identity, own-key/inherited-discriminator rejection, and bounded process list/stop frames; it rejects mixed owner operations and oversized errors.

The retained-process oracle runs the managed worker command path with a canonical descriptor, observes only negotiated process-change events, lists owned background processes with bounded output, rejects a stale owner epoch without cancelling, accepts an owned stop request, and confirms post-turn output/exit transitions while excluding another session's process. These are upstream falsifiable lifecycle controls for the existing worker/process owners; they do not prove Fabushi Rust implementation, packaged reachability, or Actions execution.

## 80. Locked worker native-inference, GitHub-failure, and replay-window oracle sources (2026-10-29)

The locked `worker-runtime-native-inference.suite.ts`, `worker-runtime-github-failures.suite.ts`, and `worker-runtime-replay.suite.ts` were read in full and recorded as `test-oracle-replaced`. The native-inference oracle binds a fixture model and credential to a local provider, disconnects Gateway admission while the request is active, and requires a node-lost terminal result, provider connection closure, exactly one request, and an aborted finishing lifecycle event. The GitHub oracle proves inherited credential helpers are cleared without a turn identity, no profile is created, write failures prevent inference, and sensitive/profile state is disposed.

The replay oracle retains an opaque provider replay anchor through a bounded tool loop, preserves the user/assistant/tool-result ordering and transcript commits, and fails before a second inference when the replay unit cannot fit the context window with the explicit local-retry error. These suites preserve negative and positive worker lifecycle invariants for existing inference, credential, transcript, and connection owners; they do not prove Rust parity, provider access, or current exact-head Actions results.

## 81. Locked worker permission, filesystem containment, and patch-policy oracle source (2026-10-29)

The locked `worker-runtime-permissions.suite.ts` was read in full and recorded as `test-oracle-replaced`. Its cases exercise the worker entry point across workspace-only and unrestricted filesystem policy, patch enable/model restrictions, memory-flush write ownership, read-only/guarded/workspace/full permission clamps, safe-bin approval behavior, and apply-patch containment. The oracle checks both positive effects and negative evidence: outside paths remain unchanged when denied, omitted tools and approval errors are explicit, the worker never reaches Gateway approval methods, memory writes stay on the configured path, and operator diagnostics identify the containment owner without leaking that hint into the tool result.

This is a cross-boundary security oracle for the existing worker, tool-permission, filesystem, and audit owners; it does not prove Rust implementation, sandbox/package reachability, or exact-head Actions execution.

## 82. Locked worker prompt, replay, skill, and transcript-ordering oracle source (2026-10-29)

The locked `worker-runtime-prompt.suite.ts` was read in full and recorded as `test-oracle-replaced`. Its cases prove empty-model tool finalization and bounded failure after one empty continuation; remote inference receives only the admitted system prompt, runtime context, skill bundle, and canonical tool surface; lifecycle finishing follows transcript durability; replay retains timestamp ownership and transcript sequence; and malformed legacy runtime-context carriers are rejected.

The suite also creates ambient workspace files and verifies they are not opened or injected when the launch descriptor supplies prepared prompt inputs, while an explicit literal prompt remains explicit data rather than an implicit file-read instruction. This oracle binds prompt construction, skill materialization, session replay, transcript/live ordering, and boundary-file-read ownership, but does not prove Fabushi Rust implementation, packaged reachability, or current Actions acceptance.

## 83. Locked worker background-exec lifecycle and environment-finalization oracle source (2026-10-29)

The locked `worker-runtime-background-exec.suite.ts` was read in full and recorded as `test-oracle-replaced`. Its lifecycle cases cover registered background execution across worker, anchor, node-host, and environment-stop loss; owned supervisor capacity; process identity and descendant cleanup; stalled process census; SDK/module-loader source witnesses; nonterminal cleanup fencing when a Linux subreaper anchor is lost; and managed EOF joining retained processes before the owner closes.

Its finalization cases hold both exec finalizers and settlement callbacks open while closing the worker environment, proving state/config paths remain available until both complete for foreground and hidden-background execution, then are removed only after settlement. This is a high-risk process, resource, plugin-loader, and cleanup oracle for existing Node-worker/Host/Runner owners; it does not prove Rust implementation, packaged execution, or current Actions acceptance.

## 84. Locked worker Gateway-tool transport queue, replay, and cancellation oracle source (2026-10-29)

The locked `worker-connection-gateway-tools.suite.ts` was read in full and recorded as `test-oracle-replaced`. Its transport fixture proves excess Gateway-tool calls queue in order while control traffic and queued aborts remain independent; cancellation of an issued call is distinct from cancellation of a queued call; heartbeat and cancellation frames retain the admitted connection; and the connection returns to ready only after the bounded in-flight set settles.

Across reconnect, the oracle requires replayable invocation resend exactly once, update sequence/generation filtering, stale-surface update rejection, cancellation during reconnect without replay, and successful completion after reconnect with the retained result. These checks bind the existing worker connection/transport owner and its Gateway-tool adapter, but do not prove Rust implementation, provider reachability, or current exact-head Actions acceptance.

## 85. Locked worker Browser, Computer observation, native-startup, bundle-install, and workspace-retention oracle sources (2026-10-29)

The locked `browser-runtime.test.ts`, `computer-observation.runtime.test.ts`, `native-inference-startup.test.ts`, `node-bundle-install-protocol.test.ts`, and `node-workspace-retain-protocol.test.ts` were read in full and recorded as `test-oracle-replaced`. The Browser oracle requires build-composed runtime use, fixed non-shell launcher arguments, bounded execution, disposal, and sanitized launcher failure. The Computer oracle ensures large pixels persist only as typed image content, never in details or JSON metadata, while retaining the provider result unchanged.

The native-startup oracle consumes fragmented private descriptor input exactly once, closes the FD before returning credentials, removes the launch marker, and rejects/redacts missing, malformed, invalid-shape, or oversized input. Bundle-install and workspace-retention oracles enforce closed exact-build identities, bounded archive/token/hash fields, prewarm compatibility, canonical sorted snapshots, generation uniqueness, retained status hashes, and exact bounded results. These are protocol/security/ownership oracles only; they do not prove Rust implementation, packaged reachability, or Actions acceptance.

## 86. Locked node-supervisor and worker-connection endpoint/close-window oracle sources (2026-10-29)

The locked `node-supervisor-protocol.test.ts`, `worker-connection-closing-window.test.ts`, and `worker-connection-endpoint.test.ts` were read in full and recorded as `test-oracle-replaced`. The supervisor protocol oracle preserves bounded lookup waits, authoring and idle-retention identity hashes, complete environment ownership, bounded connection diagnostics, and closed launch receipts with safe single-line failure output. The close-window oracle drives the real worker-thread reproducer and requires the close event to precede replay, exactly two requests, and a non-open socket state.

The endpoint oracle rejects unsafe schemes, paths, credentials, inherited properties, oversized URL/Access fields, plaintext public endpoints, and Access credentials on plaintext; it retains the private-network break-glass only behind the explicit environment gate and projects the closed Cloudflare Access header pair. These are connection/supervisor security and lifecycle oracles, not Rust implementation, packaged reachability, or current Actions acceptance.

## 87. Locked worker connection lifecycle, admission failure, backoff, and callback-isolation oracle source (2026-10-29)

The locked `worker-connection.test.ts` was read in full and recorded as `test-oracle-replaced`. Its cases keep long presence work pending through cold geolocation windows, settle heartbeat callbacks before stop cleanup, reject TLS pin mismatches before WebSocket upgrade, retain redacted admission-deadline diagnosis across clock shifts, exclude local cancellation from connection-failure reporting, and keep stopped/fenced/failed connections terminal across late hello, observers, completed startup, and invalid frames.

The suite also staggers twenty workers after transport loss with bounded jitter, isolates observer failures while preserving terminal errors, and rejects inference frames with wrong payload/session/epoch before callback notification. Together with the Gateway-tool suite it covers the connection owner's queue, admission, retry, fencing, cleanup, and callback boundaries; it does not prove Rust implementation or current exact-head Actions acceptance.

## 88. Locked worker inference/live/transcript RPC client oracle source (2026-10-29)

The locked `worker-rpc-clients.test.ts` was read in full and recorded as `test-oracle-replaced`. Its transcript cases reject frames over the byte ceiling and split semantic commits with chained leaf cursors. Live-event cases bound the preview queue, terminal barrier, cumulative ACKs, concurrent/out-of-order responses, replay resync retries, inconsistent cursors, stopped/fenced terminal errors, and disposal without retired rescheduling. Inference cases preserve stream-gap reporting, terminal delivery, active-turn reattachment, and replayed terminal consumption.

These tests bind the existing worker connection frame dispatcher plus inference, live-event, and transcript RPC owners; they distinguish transport acknowledgement from durable transcript/live terminal state and retain negative cursor/ownership cases. They do not prove Rust implementation, production packaging, provider acceptance, or current exact-head Actions evidence.

## 89. Locked worker environment cleanup, admission imports, and transcript-failure oracle sources (2026-10-29)

The locked `worker-runtime-environment.test.ts`, `worker.runtime-imports.test.ts`, and `worker.transcript-failure.test.ts` were read in full and recorded as `test-oracle-replaced`. The environment oracle closes only the worker-owned SQLite database and state directory while preserving unrelated state. The import oracle exercises rejected, cancelled, import-error, and accepted admission paths, proving runtime/workspace preparation is joined before the turn only after admission and that child-process environment isolation remains explicit.

The transcript-failure oracle distinguishes deterministic storage/schema failures, which fail once without reconnect or provider invocation, from retryable schema/SQLite/admission/broker/cancellation failures, which reconnect and replay the exact commit request without duplicate messages. It also preserves outcome-unknown semantics after a lost commit receipt and verifies the canonical session leaf/entry count. These are cleanup, admission, persistence, and recovery oracles; they do not prove Rust implementation, packaged reachability, or current Actions acceptance.

## 90. Locked managed worker command lifetime, retention, fencing, and frame-limit oracle source (2026-10-29)

The locked `worker-command.runtime.test.ts` was read in full and recorded as `test-oracle-replaced`. Its cases prove diagnostics remain JSON-safe on stdout while logs route to stderr; internal IPC start frames reject inherited/duplicate fields; build-composed Browser runtime reaches the worker boundary; pre-start lifetime gates and disconnect abort only owned execution trees; retained turns reuse one environment and exact turn cancellation; finalizers and GitHub profile disposal settle before idle readiness; background retention waits for process-scope settlement without stale readiness; and state closes on owner/output termination.

The suite also rejects duplicate or retargeted workspace identities, canonical device-folder swaps during setup, concurrent turns, oversized/unterminated managed input, same-chunk turn/cancel races, and cap-exceeding standalone/managed Unicode/image inputs while preserving ordinary large images. These are direct command-shell, ownership, cleanup, framing, and security oracles for the existing worker owners; they do not prove Rust implementation, packaged reachability, or current Actions acceptance.
