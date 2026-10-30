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

## 16a. Exact runtime-manifest/runtime read closure (2026-10-29)

The two exact source references attached to
`OCRM-RESP-NODE-HOST-RUNTIME-MANIFEST-001` were read in full from the locked
tree: `src/node-host/runtime-manifest.ts` (blob
`9e675256c727ce03b2a9820dc65fe991f0cc148d`, 113 lines) and
`src/node-host/runtime.ts` (blob
`35c5e3aba727291c797534cf7d2e5abcaa05a9e5`, 719 lines). The read confirms
that manifest construction is feature-, platform-, allowlist- and
ephemeral-aware, while runtime composition owns generation fencing,
disconnect cleanup, dynamic availability publication, cancellation/input/
progress admission, worker/MCP lifetimes and re-entrant close.

This closes source reading only. The Rust manifest remains a static builder
with no plugin/MCP inventory, dynamic availability watcher, path-environment
projection, Computer capability projection or live node-host transport, so
the row remains `wiring=not-wired`, `verification=not-run`, and acceptance
pending.

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

## 145. Node connection and candidate oracle closure (2026-10-29)

The locked `src/node-host/connection.test.ts` (blob
`347d72bbd4434eae94803c2395ccce1500f38372`, 776 lines) was read in full and
added to the connection responsibility as a test oracle. It requires cleanup
to settle before replacement publication, generation-fenced process and stats
events, capability-gated permission details and worker inventory, bounded
publication retry with redacted diagnostics, stale-ack rejection, and timer
retirement on disconnect/close/manifest replacement. The locked
`gateway-candidate-connection.test.ts` (blob
`62ea86b2e8c5c2ea4d63404568a6b52d30292a24`, 181 lines) adds pre-hello-only
failover, winner promotion, stale-close isolation, origin-bound credential
non-transfer, and all-candidate cleanup failure retention. The locked
`local-id.test.ts` (blob `98ca5bff8b3c48a1ffa9868c9d0ecd53cb6f0547`, 119
lines) requires stable canonical device identity reuse, no credential minting
on a miss, independent state-directory separation, and retry after a failed
read.

These are now explicit frozen oracle references in the parity ledger. They do
not change the implementation or wiring state: Fabushi still has no accepted
remote Gateway caller, worker inventory publication path, or exact-head
Actions/device receipt for this responsibility.

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

## 91. Locked native runtime transport and native-worker authority oracle sources (2026-10-29)

The locked `native-runtime-transport.test.ts` and `native-worker-authority.test.ts` were read in full and recorded as `test-oracle-replaced`. The transport oracle exercises every supported built-in SDK API through a trusted private CA, keeps the synthetic custodian provider key outside the compiled runtime, rejects non-loopback plaintext and untrusted/hostname-mismatched TLS before auth or request I/O, refuses same-origin and cross-origin redirects without sensitive headers, admits only a directly configured loopback HTTP hop, classifies loopback addresses canonically, and rejects unprovisioned origins.

The authority oracle holds embedded-agent runtime import open while admission changes, proving native turns fail node-lost before agent execution; it preserves the distinction between native and remote inference across connecting/admitting/reconnecting/stopped transitions, and permits exactly one execution only after the admitted authority remains valid. These are provider transport and worker authority oracles, not Rust implementation, real-provider acceptance, or current Actions evidence.

## 92. Locked native inference-stream and native-runtime authority oracle sources (2026-10-29)

The locked `native-inference-stream.test.ts` and `native-runtime.test.ts` were read in full and recorded as `test-oracle-replaced`. The inference-stream oracle withholds credential prefixes and complete credentials across text, tool-call metadata, nested arguments, raw cumulative deltas, Unicode/escaped JSON, terminal objects, provider errors, and intervening tool blocks; it preserves legitimate divergence, avoids replaying already-published arguments, discards undecodable previews, bounds held previews, and continues streaming ordinary divergent text before terminal validation.

The native-runtime oracle verifies startup-header and HTTP-normalized credential containment, frozen pricing/thinking metadata, per-session cleanup, opaque proxy authentication, sealed local markers, single named environment reads, closed startup/config APIs, duplicate/ambient API rejection, workspace directory and symlink identity, concurrent credential isolation, cancellation/close races, and revocation of escaped stream functions after turn settlement. These are direct secret-boundary, runtime-authority, workspace, lifecycle, and resource-ownership oracles for the existing native worker owners; they do not prove Rust implementation, real-provider acceptance, packaged reachability, or current exact-head Actions evidence.

## 93. Locked worker/native runtime support fixtures (2026-10-29)

The locked `native-worker-entrypoints.test-support.ts`, `native-worker.integration.test-support.ts`, `launch-descriptor.test-support.ts`, `worker-import-runtime.test-support.ts`, `native-runtime-transport.test-support.ts`, and `native-runtime.test-support.ts` were read in full and recorded as `test-oracle-replaced`. These fixtures preserve the executable boundaries used by the upstream oracles: real managed-worker process admission, production JSONL descriptor round-tripping, physical lazy-import entrypoints, child-process native transport requests, startup snapshot mutation checks, and concurrent credential/runtime isolation. They are test harness reachability and negative-boundary evidence only; they do not add a second runtime owner or prove Fabushi Rust packaging, provider acceptance, or Actions execution.

## 94. Locked worker prompt-compatibility, native-provider, and fault-session fixtures (2026-10-29)

The locked `launch-descriptor.prompt-context.test.ts`, `native-worker.test-support.ts`, and `worker-fault-session-target.test-support.ts` were read in full and recorded as `test-oracle-replaced`. The prompt-compatibility fixture pins the historical pre-prompt-context parser by blob hash and verifies the negotiated v4 assignment fields remain rejected by that legacy parser. The native-provider fixture supplies a loopback streaming provider with held-response release, request/error capture, exact tool-call framing, and owned server cleanup.

The fault-session fixture binds a claimed worker turn to the durable session leaf, lifecycle/writer identity, delegated run authority, placement tool surface, abort lifetime, and capability settlement; it releases all owned authority on failure or disposal. These fixtures preserve compatibility, provider-boundary, and session-ownership oracles for existing owners without creating a second runtime or proving Rust packaging, provider acceptance, or Actions execution.

## 95. Node-host manifest availability fence (2026-10-29)

`NodeHostManifest::build` now receives an explicit availability set from the
future manifest/transport owner. Built-in commands are retained only when that
set names them; Claude and desktop-stream commands additionally require their
feature gates, and derived capabilities are emitted only when their command is
actually advertised. The contract oracle covers both an allowlisted positive
manifest and an empty availability set, which must produce no commands or
capabilities.

This is a fail-closed manifest-boundary correction, not a live transport
implementation. No remote command is newly enabled, no pairing state is used
as availability evidence, and OCRM-M13 remains pending until a production
transport owner supplies availability and exact-head/device evidence.

## 96. Gateway device-auth and route-binding sources (2026-10-29)

The locked `call-device-auth.ts`, `call-device-auth.worker.test.ts`, and
`connection-details.ts` were read in full and mapped to the existing NodeHost
connection/auth boundary. Gateway calls create a device identity only when
authentication requires local device state; read-only calls may reuse existing
identity but must not create identity or open writable storage. Stored operator
tokens are selected by device scope and role, while local loopback shared-auth
paths intentionally omit device identity only when the supplied auth contract
actually authenticates the call. Remote route binding derives stable SSH- or
TLS-fingerprint-scoped credential keys, preserves the selected SSH remote port,
redacts URLs in diagnostics, and rejects non-loopback plaintext websocket URLs
unless the explicit break-glass environment contract is enabled. The worker
oracle pins the no-write/read-only and lazy-storage invariants. These sources
extend `OCRM-RESP-NODE-HOST-CONNECTION-001`; they do not create a second auth
owner or prove Fabushi Rust transport, package, device, or Actions acceptance.

## 97. Gateway websocket admission and session attachment (2026-10-29)

The locked `connect-admission.ts`, `connect-device-metadata.ts`,
`connect-operator-access.ts`, `connect-policy.ts`, `connect-session.ts`, and
`node-lifecycle-dispatch.ts` were read in full. Admission now has a clear
producer-to-owner boundary: protocol/version, role, origin, remote-ingress,
startup and scope ceilings are checked before registration; device metadata and
bootstrap profiles remain approval-bound; operator authority is attached to one
socket and revoked without destroying retained run authority; session attach
captures authenticated node pairing identity/generation, rechecks it after
awaited work, registers the live NodeHost session, and fences plugin surfaces
and snapshots to the approved node capability set. The lifecycle tracker drains
only admitted `node.invoke.progress/result` work with a bounded timeout and does
not create a queue. These sources extend the existing NodeHost connection and
command-dispatch owners and the Gateway auth/pairing owners; they do not prove
Rust production wiring, exact-head execution, package reachability, or device
acceptance.

## 103. Active-session shutdown drain and tracker (2026-10-29)

The locked `active-sessions-shutdown-drain.ts`,
`active-sessions-shutdown-tracker.ts`, and
`active-sessions-shutdown-tracker.test.ts` were read in full. The tracker is
the process-local set of unmatched session starts: session identity is the
deduplication key, empty identities are ignored, finalized sessions are
forgotten, and callers receive a snapshot rather than the backing map. The
shutdown owner snapshots tracked sessions, starts all `session_end` emissions
before applying one bounded aggregate timeout, forgets each entry before
emission, preserves stable transcript-source leases until drain settlement,
revokes those leases after the bounded wait, and reports timeout without
claiming late hooks settled. This belongs to the existing session production
owner and plugin hook owner; it does not create a second lifecycle registry or
upgrade Rust implementation, runtime wiring, or shutdown acceptance.

## 98. Gateway hello, capability projection, and pairing-plan sources (2026-10-29)

The locked `connect-hello.ts`, `connect-pairing-approval-plan.ts`,
`connect-device-metadata.test.ts`, `connect-device-metadata.watch.test.ts`,
`connect-hello.update-scope.test.ts`, and
`connect-hello.setup-completion.test.ts` were read in full. Hello projection
uses effective post-admission scopes, gates sensitive/update details, exposes
only approved NodeHost capabilities, and binds browser identity routes to their
live claim lifetime. Bootstrap handoff completion is confirmed only after hello
delivery; failed delivery emits an uncertain state or restores an uncorrelated
token, while replaced public keys cannot consume the setup bearer. Pairing-plan
selection has mutually exclusive silent, trusted-CIDR, trusted-proxy,
remote-ingress, setup-code, and manual lanes, with same-key scope upgrades and
device metadata/platform changes remaining approval-bound as appropriate. These
sources refine the existing NodeHost connection, Gateway auth, DevicePairing
approval, and token owners; they do not prove Rust implementation or package,
device, and exact-head acceptance.

## 100. Gateway websocket frame orchestration and authenticated request dispatch (2026-10-29)

The locked `message-handler.ts` and `authenticated-request-dispatch.ts` were
read in full and mapped to the existing Gateway dispatch registry. The frame
owner enforces pre-auth size and handshake shape, remote-ingress freshness,
root-work admission, pairing-cleanup release, ordered queued handshake frames,
phase/timing diagnostics, and node-only close-drain continuation. The
authenticated dispatcher validates frames, retains request authority, fences
device revocation/shared-auth generation/operator access/runtime identity,
serializes credential-invalidating mutations, binds expected profile and
mutation authority, and delegates actual methods to the canonical Gateway
method owner. It preserves node lifecycle draining and request cancellation
without making a second agent/session/runtime owner. This closes the
Gateway-frame producer → authority fences → existing method/NodeHost consumer
map; Rust production wiring and exact-head acceptance remain unverified.

## 99. Dedicated worker websocket admission and dispatch (2026-10-29)

The locked `worker-admission-boundary.ts`, `worker-connection-dispatch.ts`,
`worker-connection-frames.ts`, and `worker-connection.ts` were read in full.
Worker ingress is a separate protocol owner: public admission is rate-limited
and serialized, credentials are checked before environment availability is
charged, ownership is claimed once, and the receiver limit is raised only after
the admitted identity is known. The dispatcher never calls the generic Gateway
method registry; every worker operation is feature-gated, schema-validated,
owner-epoch/connection-validated, and projected through typed worker errors.
Tool and computer work are bounded per connection and survive transport loss
only through the existing durable turn authority; transcript commit replay is
distinguished from settled persistence failure. Heartbeat refreshes revalidate
identity before publishing results, and credential expiry closes unattached
workers. This maps to existing NodeHost connection and command-dispatch owners
without introducing a second worker runtime or queue; Rust implementation,
packaging, exact-head execution, and real provider/device acceptance remain
unverified.

## 101. Gateway request lifetime, start budget, diagnostics, and policy fencing (2026-10-29)

The locked `server-request-entry.ts`, `request-diagnostics.ts`,
`request-start.ts`, and `ws-policy-close.ts` were read in full. Preparation
leases are sealed and joined independently from handler execution; only the
exact current node completion invoke may cross a closing boundary. Request
starts use bounded work/control budgets, per-connection caps, FIFO preservation
with explicitly independent reads, bounded wait expiry, and a preparation
concurrency limit. Diagnostics retain only bounded classified facts and keep
response/dispatch/handler outcomes distinct. Policy mutations hold their
response until completion, invalidate source authority immediately, and notify
accepted work without confusing transport closure with business settlement.
These are orchestration/evidence primitives under the existing Gateway dispatch
owner, not a new queue, findings store, or runtime; Rust wiring and exact-head
acceptance remain unverified.

## 102. Gateway request lifecycle conformance oracles (2026-10-29)

The locked `server-request-entry.test.ts`, `request-start.test.ts`, and
`authenticated-request-dispatch.policy-close.test.ts` were read in full.
Together they pin the negative and ordering boundaries for request entry:
preparation failure cannot bypass an already accepted mutation, pending node
completion may drain only through the exact allowed close path, independent
reads may bypass parked preparation while connection mutations preserve order,
start and frame budgets remain bounded, cancellation releases parked waiters,
and policy writers deliver only accepted results before source invalidation
closes transport. These tests are conformance oracles attached to the existing
Gateway dispatch owner; they do not authorize a second Rust queue/runtime or
constitute local test execution, Rust parity, packaging, or provider/device
acceptance.

## 104. Session-end transcript source and hook ownership (2026-10-29)

The locked `session-end-transcript-reader.ts`,
`session-transcript-files.fs.ts`, `session-hooks.ts`,
`session-end-transcript.ts`, and `hook-runner-global.ts` were read in full.
The session owner bounds visible ended-transcript reads by message and JSON
bytes, distinguishes closed/reset/archived sources, verifies reset archive
identity before fallback, canonicalizes and bounds archive discovery, and
keeps archive cleanup path-scoped. Reply lifecycle hooks detach from the
request writer/controller lanes while retaining bounded Gateway work custody.
The plugin owner exposes transcript access only through a private, revocable
capability lease and resolves the live global hook registry on every dispatch;
conversation access remains explicitly opt-in and lifecycle revocation fences
late reads. These are existing Session and Plugins Hook Pipeline boundaries,
not a second transcript store, hook registry, or Rust runtime; implementation,
production wiring, and acceptance remain unverified.

## 105. Agent turn admission, model selection, and failure revalidation (2026-10-29)

The locked `agent-admission-controller.ts`, its focused test,
`agent-run-admission-model.ts`, `agent-run-admission-types.ts`,
`agent-run-admission-revalidation.ts`, and
`agent-run-admission-cleanup.ts` were read in full. Admission binds the
resolved session target before side effects, joins the existing session-effect
lease, rechecks dedupe/session/archive authority after asynchronous preparation,
preserves expected-session and restart-abort semantics, and emits cached
outcomes only after releasing the lease. Model selection applies restored
continuation overrides through the existing runtime resolver. Revalidation
retains owner-specific errors, fences stale model publication and parent
resume, records abort dedupe state, and joins rejected preparation cleanup
before releasing runtime, caller, session, and recovery authorities. These
sources map to the existing Session Turns Controller; they do not authorize a
second admission queue, model authority, or runtime. Rust implementation,
production wiring, and exact-head acceptance remain unverified.

## 106. Agent turn dedupe reservation and replay ownership (2026-10-29)

The locked `agent-dedupe.ts` and `agent-dedupe-lifecycle.ts` were read in
full. Dedupe keys include the request idempotency key and optional execution
approval follow-up identity; replay distinguishes accepted reservations,
queued aborts, durable-input reconciliation, live RPC ownership, incognito
errors, and terminal cache outcomes. Reservation lifecycle preserves an
intentional Stop, binds the committed session target only while the exact
reservation owns every key, refuses stale ownership, and records timeout or
cancellation only after current admission checks. Unaccepted cleanup removes
only the exact owned keys, while reset completion and durable retry reconcile
through the existing completion owner. These sources remain under the Session
Turns Controller and existing Gateway dedupe store; they do not introduce a
second cache, replay authority, or queue. Rust implementation and runtime
acceptance remain unverified.

## 107. Agent execution identity, recovery, and terminal settlement (2026-10-29)

The locked `agent-run-dispatch-execution-identity.ts`,
`agent-run-dispatch-outcome.ts`, `agent-run-execution-lineage.ts`,
`agent-run-execution-types.ts`, `agent-run-recovery-admission.ts`,
`agent-run-terminal-producer.ts`, and `agent-run-undispatched-outcome.ts` were
read in full. Execution identity facts are carried through a private context
carrier and authenticated spawn lineage is consumed once at child admission;
local-user, agent-tool, ACP, represented-subject, grant, and runtime-assurance
facts stay explicit. Terminal classification preserves restart/rpc/stop/timeout
semantics and keeps raw provider failures distinct from owner-recorded
cancellation. Restart recovery is committed durably before execution and
restored or cancelled exactly once; terminal publication is fenced to the
registered run instance, lifecycle generation, session identity, and delegated
authority. Undispatched input settles before replay publication and cleanup.
These boundaries remain under the existing Session Turns Controller and
security/agent authorities; they do not create a second execution identity,
recovery, or terminal store. Rust implementation and acceptance remain
unverified.

## 108. Agent turn source admission and preparation custody (2026-10-29)

The locked `agent-turn-source-admission.ts`,
`agent-turn-source-admission.liveness.test.ts`,
`agent-turn-preparation-source.ts`, and
`agent-turn-admission-cleanup.ts` were read in full. Source admission first
authorizes the canonical physical session target, then registers the exact
abort/source identity with timeout, lifecycle generation, owner connection and
device facts. Preparation retains an opaque controller mailbox until routing
selects the physical target, transfers or retires it exactly once, and
publishes cancellation against the current alias/target. Cleanup discards
prepared media and clears unaccepted dedupe only after owner release, then
releases Gateway admission, Cron continuation, and recovery in the declared
order. The liveness oracle confirms requester custody across default/custom
stores and early/prepared admission without claiming foreign child ownership.
These are existing Session Turns Controller boundaries; Rust wiring and
acceptance remain unverified.

## 109. Agent turn transport, privacy, and context contracts (2026-10-29)

The locked `agent-handler-helpers.ts`, `agent-run-diagnostics.ts`, `io.ts`,
`principal.ts`, and `types.ts` were read in full. Session helpers preserve
deleted/archive/harness admission errors, reusable Cron continuation facts,
inter-session prompt suppression, and bounded continuation-release recovery.
Diagnostics capture incognito privacy once and apply it consistently to runtime
logging, replay entries, and RPC metadata. The I/O adapter preserves
positional response ordering and late final delivery; principal capture
transfers local-user/operator source identity without rebuilding trusted
metadata and gates tool-event observation by client capability. The typed
context contract exposes only the existing Gateway/session services required
by Agent Turn. These sources map across the existing Session Turns and Gateway
dispatch owners; they do not create another transport, principal, or privacy
authority. Rust wiring and acceptance remain unverified.

## 110. Agent turn media understanding and final-media custody (2026-10-29)

The locked `agent-image-understanding.ts`,
`agent-image-understanding.test.ts`, `agent-run-media.ts`,
`agent-run-final-media.ts`, and `agent-run-media.test.ts` were read in full.
Text-only turns reuse the existing media-understanding runtime, preserve image
facts while suppressing pixel hydration after a successful description, and
fail closed on missing capability, empty output, scope denial, or revoked
authority. Final media remains bound to the registered run/source and current
session placement/database identity; visible payload selection excludes
sensitive/internal/incognito/external-delivery cases, persists managed media
and transcript rewrites only under the exact lifecycle revision, and removes
unretained artifacts after abort or authority change. The focused oracle also
confirms source-file deletion does not remove committed managed artifacts.
These sources map to existing media-understanding and media-store read-scope
owners; they do not add a second media store or transcript authority. Rust
wiring, package reachability, and acceptance remain unverified.

## 111. Agent request preflight, routing, and restart recovery context (2026-10-29)

The locked `agent-request-preflight.ts`,
`agent-request-preflight.source-lineage.test.ts`,
`agent-request-routing.ts`, `agent-request-routing.session-id.test.ts`,
`agent-restart-recovery-context.ts`, and its focused test were read in full.
Preflight fences backend-only controls, model overrides, raw-model/session
mutation, swarm collector registration, database admission refusal, source
provenance, restart-recovery fields, and structured-output schema before
reservation. Routing resolves agent/session identity without opening a
database during a read-only lookup, reserves before async store/recipient
resolution, validates expected incarnation, and drops stale exec-approval
followups after session rebound. Recovery context restores only an exact
durable claim, preserves channel/control-UI presentation facts, withholds
delivery authority for blank targets, and rehydrates execution identity only
for the admitted recovery cycle. These boundaries map to the existing Session
Turns Controller and Gateway session owners; Rust wiring and acceptance remain
unverified.

## 112. Agent turn wait/facade contracts and recovery typing (2026-10-29)

The locked `agent-turn-start-request.types.ts`, `agent-wait.ts`,
`internal-facade.runtime.ts`, `internal-facade.types.ts`,
`rpc-source.test-support.ts`, `recovery-typing.ts`, and
`recovery-typing.test.ts` were read in full. The internal facade admits only
trusted controller input and source assertions, exposes start-owner observation
and abort hooks, and keeps final delivery/settle wake replay out of wire
requests. Agent wait preserves exact run IDs when required, captures the
initial session and lifecycle generation, distinguishes queued ownership from
terminal snapshots, and returns bounded timeout results. RPC test support
creates exact scoped sources and marks only the matching admitted operation as
started. Recovery typing is cosmetic only: it shares one run loop, revalidates
gateway/config/channel authority before each guarded send, honors live typing
policy and timeout, and always clears typing on retirement or failure. These
remain existing Gateway dispatch, Session Turns, and channel lifecycle owners;
no second wait/facade or delivery authority is introduced.

## 113. Agent session persistence and lifecycle mutation ownership (2026-10-29)

The locked `agent-session-persist.ts` and its focused test were read in full.
Persistence revalidates lifecycle generation, expected session identity, archive
and restart-recovery quarantine, Cron continuation identity, reusable runtime
facts, scheduled-tool grants, creator authorization, sandbox provenance, and
send-policy admission before replacing the canonical session entry. It records
foreground recovery ownership only for the exact admitted run, preserves
session-id rotation lineage, emits session-created and end/start plugin hooks
with bounded transcript custody, and returns the post-admission session target
without creating another session store. The focused oracle covers synthetic-run
sandbox provenance, interrupted recovery ownership, queued-peer fencing, and
authorization revoked during preparation. These sources map to the existing
Session Turns Controller and session/plugin owners; Rust persistence, wiring,
and exact-head acceptance remain unverified.

## 114. Agent execution phase ownership and settlement fencing (2026-10-29)

The locked `agent-run-execution-phase.ts`, its owner test support, owner test,
and execution-lineage oracle were read in full. The execution phase retains
Gateway work admission and session-controller source custody, binds workspace
readiness and model-runtime/plugin-generation leases, revalidates operator,
requester, abort, lifecycle, RPC-source, session, and delegated-authority
identity before dispatch, and separates undispatched abort/failure settlement
from terminal publication. It transfers the admitted run through the existing
agent dispatch owner, preserves media and transcript cleanup ordering, releases
Cron/restart-recovery ownership exactly once, drains runtime disposal before
acknowledging cleanup, and keeps final delivery deferred until settlement. The
owner oracle covers real session drain, preparation-to-dispatch lending,
runtime-generation freezing, liveness/access fencing, follow-up ownership, and
pre-dispatch diagnostics; the lineage oracle rejects unsupported or incomplete
parent evidence. These sources remain one Session Turns Controller execution
boundary and do not authorize a second run loop, runtime lease, or settlement
store. Rust implementation, production wiring, and acceptance remain
unverified.

## 115. Agent content preparation and media admission ownership (2026-10-29)

The locked `agent-content-phase.ts` and focused test were read in full.
Content preparation normalizes and validates channel hints, applies voice-wake
routing only for eligible unbound requests, resolves the canonical session and
agent owner before attachment reads, selects the persisted or explicit model
for image capability, and parses/offloads media under bounded bytes and live
admission assertions. It strips image markers only when the selected model
requires understanding, preserves input provenance, and returns the prepared
content without creating a media or model authority. The oracle covers global
session ownership, replacement during read, target refusal, text-only model
offload, and explicit vision override. This remains an existing Session Turns
Controller boundary delegating to media/model owners; Rust wiring and
acceptance remain unverified.

## 116. Agent delivery planning and scoped outbound observation (2026-10-29)

The locked `agent-delivery-phase.ts` and focused test were read in full.
Delivery planning resolves the active session owner, observes only live RPC
sources matching the selected session/store/Gateway scope, preserves WebChat
session ownership, resolves channel/account/thread and target fallbacks through
the existing outbound delivery owners, and distinguishes strict failure from
best-effort session-only degradation. Incognito diagnostics suppress private
provider errors while retaining the caller-visible failure contract. The
oracle covers local versus foreign Gateway/store source scope and strict or
best-effort target/channel failures across ordinary and incognito sessions.
This is a Session Turns Controller orchestration boundary over existing
outbound channel owners, not a second delivery transport or receipt store;
Rust implementation, wiring, and acceptance remain unverified.

## 117. Agent job cache, terminal settlement, and wait fallback ownership (2026-10-29)

The locked `agent-job.ts`, `agent-job.execution-settlement.test.ts`, and
`agent-job.timeout-fallback.test.ts` were read in full. The job owner listens
to lifecycle start/end/error events, retains bounded per-source snapshots with
TTL and capacity eviction, merges sticky terminal outcomes without erasing
reply/delivery/receipt evidence, and binds session facts to the exact lifecycle
generation. Gateway dedupe publication is only a replay projection: it
preserves request identity, separates active/terminal/untracked states, and
cannot overwrite an authoritative lifecycle outcome or borrow timing across a
new attempt, session, or generation. `agent.wait` uses scoped waiters, close
and lifecycle-reset cancellation, bounded outer timeouts, and retry-grace
fallbacks that distinguish hard timeout, retryable provider failure, yielded
execution, and definitive liveness failure. The two oracles cover lifecycle
versus dedupe ordering, terminal reply/receipt custody, session replacement,
retry-grace recovery, multiple waiters, cancellation precedence, and cache
expiry. This remains the existing Session Turns Controller observation owner;
it does not create a second dedupe cache, wait authority, or execution store.
Rust implementation, wiring, and acceptance remain unverified.

## 118. Agent user-turn input custody and subagent follow-up ownership (2026-10-29)

The locked `agent-run-user-turn.ts`, its focused test,
`agent-run-subagent.ts`, and its focused test were read in full. User-turn
preparation claims only trusted internal approval handoffs, validates private
completion and frozen-cohort replay provenance, persists inbound media and
pending input under the admitted session identity, records sender/owner and
requester facts, and transfers transcript/media cleanup exactly once across
abort, timeout, persistence failure, and settlement failure. It rechecks live
requester authority before dispatch and keeps retained media when durable input
custody has already transferred. Subagent preparation prioritizes exact
follow-up successors and parent-resume custody, rejects requester mismatches,
persists plugin ownership before acceptance, reactivates only eligible child
sessions, and settles an unstarted parent/follow-up only when the exact RPC
source, operational run instance, lifecycle generation, and session identity
still match. Timeout and one-use completion-tool grants inherit only from the
same admitted child identity. The oracles cover authority revocation during
input/media staging, cleanup ownership outcomes, plugin registration fencing,
parent resume, requester matching, and successor adoption without sibling
ownership. These are existing Session Turns, transcript, approval, and
subagent-registry boundaries; no second input store, subagent registry, or
runtime is introduced. Rust implementation, wiring, and acceptance remain
unverified.

## 119. Gateway agent dispatch, terminal producer, and replay ownership (2026-10-29)

The locked `agent-run-dispatch.ts`, its owner support fixture, owner test, and
incognito test were read in full. Dispatch binds the admitted RPC source,
operational run instance, lifecycle generation, session identity, abort signal,
and delegated continuation before invoking the existing agent command owner.
It keeps media finalization and terminal-producer completion in one settlement
boundary, completes pending user-turn processing before replay publication,
projects canonical terminal outcomes separately from wire status, preserves
request identity while fencing stale replay, and cleans the exact owner before
emitting the final response. Cron creator authority and skill workspace scopes
are carried only through their existing capabilities; follow-up settlement
requires the exact retained owner and successor reply. The owner oracle covers
startup/producer failures, withdrawn-input confirmation, registration/session/
instance replacement, settlement-before-replay ordering, follow-up custody,
yield successor ownership, and canonical terminal classification. The
incognito oracle proves live caller payloads remain available while runtime,
delivery, and cached replay diagnostics omit private output. These sources map
to the existing Session Turns Controller, transcript, terminal-producer, and
agent command owners; no second dispatch loop, replay store, or transport is
introduced. Rust implementation, wiring, and acceptance remain unverified.

## 120. Agent request preflight and session reuse oracle closure (2026-10-29)

The locked `agent-request-preflight.test.ts`,
`agent-session-preparation.worker.test.ts`, and `agent-session-reuse.test.ts`
were read in full. The preflight oracle verifies database-admission refusal
retry contracts, Swarm collector schema/registration/launch ownership and
exact cached replay, requester-agent overrides, restart-recovery Code Mode
authority, explicit bare-session ownership, and reserved exec-completion
provenance. The preparation oracle verifies active/failed session candidates
and archived-session rejection without host-row reads. The reuse oracle pins
failed/killed transcript-window behavior, session-header backfill, lifecycle
revision preservation, exact UUID allocation, concurrent replacement
preservation, and expected-session fencing after a worker read yields. These
tests are conformance evidence for the existing Session Turns Controller,
session preparation/reuse, database admission, and subagent scheduler owners;
they do not justify a second preflight queue or session store. Rust wiring,
runtime execution, and acceptance remain unverified.

## 121. Internal agent-turn facade and Gateway dispatch custody (2026-10-29)

The locked `internal-facade.ts` and `internal-facade.test.ts` were read in
full. The facade is the frame-free Gateway boundary for agent dispatch and
`agent.wait`: it performs pre-dispatch authorization and parameter validation,
enters and releases the request lifetime exactly once, joins the existing
execution scope, preserves acceptance-before-final ordering, and delegates
turn execution to the existing agent-turn service. It fences context,
admission, lifecycle generation, session, agent, and RPC-source identity;
late cancellation targets the captured source registration rather than a
later run-id lookup. `agent.wait` uses the same authorization/envelope and
observation scope, retains private session facts, and converts deadline or
abort handling through the existing Gateway dispatch wait boundary. The
facade oracle covers cold-import joining, closure/abort/admission/context
rechecks, acceptance and post-acceptance errors, cached execution reporting,
startup-owner fencing, watchdog non-reentrancy, and deadline cancellation
without aborting an unrelated run. These sources map to the existing Gateway
dispatch registry and agent-turn service owners; they do not authorize a
second agent runtime, transport, cancellation registry, or session store.
Rust implementation, wiring, runtime execution, and acceptance remain
unverified.

## 122. Agent wait/dedupe observation and replay custody (2026-10-29)

The locked `agent-wait-dedupe.test.ts` was read in full. The oracle binds
queue observations to the exact Gateway, session, lifecycle generation, and
RPC source; keeps whitespace-distinct run IDs separate; distinguishes active,
queued, terminal, retired, and sessionless registrations; and preserves
terminal reply, delivery, receipt, timeout, cancellation, and retry evidence
across lifecycle-versus-dedupe publication order. It verifies sticky terminal
retirement only after an admitted new attempt, expiry from the latest write,
visible queued waits after cache expiry, session replacement fencing,
agent/chat abort-source selection, later queue admission without completing
another Gateway's waiter, concurrent waiter settlement, scope-local observer
retirement, gateway-draining timeout projection, and timeout/cancellation
precedence. These are conformance oracles for the existing Session Turns
Controller, agent-job, session mailbox, and Gateway context owners; they do
not create a second wait authority, dedupe cache, replay store, or execution
runtime. Rust implementation, wiring, runtime execution, and acceptance
remain unverified.

## 123. Agent run admission and ownership-transfer boundary (2026-10-29)

The locked `agent-run-admission-phase.ts` was read in full. The admission
phase restores cached pre-registration outcomes, acquires and releases
Gateway/session work admission, binds the operational run instance and
provider facts to the exact RPC source, and creates the run abort owner only
after the session target is authorized. It fences lifecycle generation,
requester/cron authority, parent-resume and follow-up ownership, model-runtime
leases, plugin-subagent registration, pending-input custody, operator
authority, restart recovery, and session identity before acceptance. After
the synchronous acceptance transfer it publishes the dedupe projection,
arms cancellation, records the user-turn participant, and transfers cleanup
to execution; every pre-acceptance failure releases the exact leases,
operator, subagent, recovery, transcript/media, and Gateway admission owners.
This is the existing Session Turns admission and execution-custody boundary,
not a second admission queue, abort registry, model runtime, or subagent
runtime. Rust implementation, wiring, runtime execution, and acceptance
remain unverified.

## 124. Agent turn service orchestration and execution handoff (2026-10-29)

The locked `agent-turn-service.ts` was read in full. The service owns the
single Gateway turn preparation pipeline: cached replay and source identity
checks, dedupe reservation, request routing, session actor-storage custody,
content preparation, reset/session preparation and authorization, session
persistence, delivery resolution, run admission, and asynchronous execution
handoff. It composes the existing phase owners, keeps source/request/session
assertions active through every awaited boundary, transfers storage,
preparation-source, media, cron-continuation, recovery, and Gateway admission
ownership exactly once, and releases them on non-transfer paths. `agent.wait`
is a projection through the existing wait owner. The service therefore maps
to the existing Session Turns Controller, session storage/admission,
transcript, delivery, and execution owners; it does not create a second
pipeline, scheduler, transcript store, or runtime. Rust implementation,
wiring, runtime execution, and acceptance remain unverified.

## 125. Native requester-settle continuation and pending-input oracle (2026-10-29)

The locked `agent-run-settle-continuation.native.test.ts` was read in full.
The native oracle proves that requester-settle processing survives later user
turns, replays completed private input without reopening it, preserves
unconsumed interrupted input, refuses legacy committed input outside the
current turn, and cannot publish cancellation after completion authority is
revoked. It keeps timeout input until the producer settles, distinguishes
hard provider timeout from nonfinal timeout, preserves peer-delivery success
and duplicate reconciliation, retains the committed child target and session
metadata, and preserves accepted sender/profile identity through rename and
replay. The fixture also validates exact RPC-source, lifecycle-generation,
session-target, pending-input, transcript, and cleanup fencing. This is a
conformance oracle for the existing Session Turns, pending-input,
transcript-processing, subagent-settle, and terminal execution owners; it
does not authorize a second continuation engine or input store. Rust
implementation, wiring, runtime execution, and acceptance remain
unverified.

## 126. Approval channel custody, audience lineage, and runtime authority (2026-10-29)

The locked `approval-channel-custody.ts`, its focused oracle,
`approval-session-audience.ts`, its focused oracle, and
`agent-runtime-approval-authority.ts` were read in full. Channel custody
binds an approval reviewer to the normalized channel/account/sender,
configured plugin capability, session delivery, eligible account set, and
current session row; it retains the exact read owner and invalidates approval
when configuration, plugin capability, delivery, or session identity changes.
Plugin approval is refused unless the channel explicitly supports scoped
reviewer policy, while system-agent approval on channels without approver
settings remains restricted to the configured command owner. Audience
resolution canonicalizes agent-scoped aliases, follows current registry
controller/requester lineage before stale session metadata, bounds breadth,
rechecks incognito/source rows and registry readiness, and falls back to a
scoped stream key only when lookup is unavailable. Runtime approval further
requires current worker/run delegated authority and a matching message-action
turn capability. These sources map to the existing Plugin SDK approval
security, session lineage, and run-authority owners; they do not create a
second approval registry, audience graph, or runtime authority store. Rust
implementation, wiring, runtime execution, and acceptance remain
unverified.

## 127. Agent runtime identity, lineage handoff, and command-policy custody (2026-10-29)

The locked `agent-command-policy.ts`, `agent-runtime-execution-lineage.ts`,
`agent-runtime-session-spawn-context.ts`, `agent-runtime-identity-token.ts`,
and `agent-runtime-identity-token.test.ts` were read in full. Session reset
commands remain a narrow existing agent-turn policy surface. Runtime
identity is a signed, state-directory-bound bearer whose payload binds the
normalized agent/session, operational run instance, delegated local/worker
authority, execution identity, UI/source routing, message-action capability,
cron grants, and bounded session-spawn policy. Minting requires active run
authority and validates worker claim, original approval claim, grant
provenance, terminal reply correlation, source exclusivity, and short-lived
capability TTLs; verification rechecks the local socket secret, schema,
authority instance, lifecycle, expiry, and execution identity. Spawn lineage
facts are stripped from the bearer and transferred through a bounded,
one-shot, expiring handoff whose redemption is tied to the exact parent
authority and operational run instance. The oracle covers terminal and
replacement invalidation, worker delayed-use fencing, secret rotation and
state-directory binding, payload tampering, spawn policy round-trips,
short-lived cron/message capabilities, approval-write serialization, and
one-shot lineage redemption. These sources map to the existing Session Turns
command-policy owner and Plugin SDK approval/run-authority security owner;
they do not create a second bearer store, authority registry, or spawn
runtime. Rust implementation, wiring, runtime execution, and acceptance
remain unverified.

## 128. Session work admission and agent-deletion fencing (2026-10-29)

The locked `session-agent-work-admission.ts` and its oracle were read in
full. The work-admission owner canonicalizes agent/state database identity,
keeps deletion cleanup closures scoped to the exact agent and state path,
retains terminal writes while deletion cleanup is pending, and replays those
writes before removing the journal fence. Session writes are rejected by the
existing deletion ingress cleanup or the durable pending-deletion journal;
agent/session interruption drains only the matching admitted work, preserves
other state paths, and refuses self-deletion from the active turn. The oracle
covers FIFO refusal before a queued write's turn, restart-persisted deletion,
overlapping session Stop precedence, unseen sessions, retained terminal
writes, and revalidation while cancellation drains. This maps to the existing
Session Work Admission/agent deletion owner and does not create a second
database lock, deletion journal, or session write gate. Rust implementation,
wiring, runtime execution, and acceptance remain unverified.

## 129. Session turn admission, mailbox claims, and sequence invariants (2026-10-29)

The locked `session-controller.admission.ts`,
`session-controller.admission.test.ts`, `session-controller.admission-rule.ts`,
`session-controller.admission-rule.test.ts`, and
`session-controller-sequence.test.ts` were read in full. Turn admission
borrows only the exact ambient owner/claim and matching physical session
target, otherwise claims the mailbox source or submits a new task. It binds
incarnation and store scope, composes caller/owner cancellation, keeps source
cleanup receipts ahead of successor admission, retires failed operations,
and releases claims exactly once. The rule owner distinguishes retired
entries, clearing mailboxes, stale claims, waiting inputs, follow-up and
successor barriers, active turns, lifecycle blocks, and mutation-owned turns;
only a mutation's own fenced turn may bypass its waiting inputs. The model
sequence oracle independently checks arrival order, one newest interrupt,
native retry/late detach, raw owner and source settlement, mutation fences,
backend Stop identity, borrowed native work, and absence of orphaned mailbox
entries across generated interleavings. These sources map to the existing
Session Turns Controller and mailbox/operation owners; they do not create a
second scheduler, claim registry, or turn reducer. Rust implementation,
wiring, runtime execution, and acceptance remain unverified.

## 130. Session target identity, mailbox claim release, and operation admission (2026-10-29)

The locked `session-controller.target.ts`,
`session-controller.mailbox-claim.ts`, and
`session-controller.operation-admission.ts` were read in full. A captured
session target binds canonical store scope, aliases, agent owner, optional
physical store path, and incarnation; matching deliberately treats
incarnation as evidence rather than live authority and requires the same
physical store and compatible agent owner. Mailbox claim adoption accepts
only the current operation and owner, records exact inputs/sources, binds the
Gateway context, and defers release until operation, adoption, settlement,
injection, and source-cleanup receipts finish. Retry-before-execution is a
specific producer receipt and reopens only a still-current, non-aborted source;
otherwise inputs retire and claim settlement reports aggregate cleanup
failure. Operation admission validates the canonical key/session ID, applies
the turn rule, installs the exact owner/target/context, retains lifecycle
custody, and supports owner adoption without releasing the previous source
until its producer settles. These sources map to the existing Session Turns
Controller, mailbox, operation, and target owners; they do not introduce a
second identity authority, claim registry, or settlement path. Rust
implementation, wiring, runtime execution, and acceptance remain
unverified.

## 131. Session mailbox source custody, selection, operation lifecycle, and settlement (2026-10-29)

The locked `session-controller.mailbox-source.ts`,
`session-controller.mailbox-dispatch.ts`, `session-controller.operation.ts`,
and `session-controller.settlement.ts` were read in full. Source custody
keeps injection order ahead of later reservations, requires live authority
and exact mailbox ownership before native injection, distinguishes ACK from
the final native outcome, and retains cleanup/abandonment receipts through
cancellation, withdrawal, source adoption, and disposal. The mailbox selector
is the sole successor selector: it applies mutation ownership, priority
interrupts, follow-up barriers, gateway scope matching, delivery grouping,
debounce, restart-drain refusal, and capability preparation before creating a
claim. Reply operations bind the exact owner/target, maintain lifecycle
state, backend readiness, watchdog phases, tool authority, terminal-producer
fences, session-id/key adoption, rekey ownership, abort result codes, yield,
recovery, and cleanup barriers; settlement waits on producer completion or
owner fencing and provides bounded barrier/end-by-session waits. These sources
map to the existing Session Turns Controller mailbox, operation, watchdog,
and settlement owners; they do not create a second scheduler, operation
state machine, source registry, or wait authority. Rust implementation,
wiring, runtime execution, and acceptance remain unverified.

## 132. Session controller lifecycle protection and physical identity resolution (2026-10-29)

The locked `session-controller.lifecycle-state.ts`,
`session-controller.identity.ts`, and `session-controller.identity.test.ts`
were read in full. Lifecycle state retains captured targets and raw operation
settlement independently of the current physical key, computes admission
blocks from mutations/closures/foreign operations, wakes mailbox successors,
and emits settled-input lifecycle events only when backend and RPC-source
custody are no longer active. Identity resolution reads the exact reserved or
adopted source, rejects stale/aborted/non-current operations, preserves
session-id aliases across incarnation rotation, and returns an explicit
ambiguous result when the same session ID is active in multiple store scopes
instead of guessing. The oracle covers equal IDs in different stores,
selected-owner mutation, alias sharing, cross-agent global sessions,
reservation dedupe/conflict, exact source adoption and cancellation,
retargeting without receipt replacement, nested native maintenance borrowing,
reentrant successor cleanup, and old/new incarnation lookup. These sources
map to the existing Session Turns Controller lifecycle and identity owners;
they do not create a second owner registry, identity index, or lifecycle event
authority. Rust implementation, wiring, runtime execution, and acceptance
remain unverified.

## 133. Session controller lifecycle admission, registry state, and custody contracts (2026-10-29)

The locked `session-controller.lifecycle.ts`,
`session-controller.state.ts`, `session-controller.contracts.ts`,
`session-controller.lifecycle-runtime.ts`,
`session-controller.lifecycle-agent.ts`, and
`session-controller.lifecycle-work.ts` were read in full. Lifecycle mutation
and subordinate-effect admission preserve one ordered Session Turns owner:
mutations close ingress, wait or preempt competing effects, retain writer and
producer receipts, and release only after cleanup; effects never create a
second turn queue and are fenced through validation, exclusive store writes,
gateway context, agent-deletion admission, handoff, interruption, and release.
The state registry binds physical store and agent ownership, indexes logical
aliases, rejects ambiguous cross-store lookup, preserves operation identity
through rekeying and backend detach, tracks execution/cleanup/follow-up
barriers, and prunes only entries with no remaining custody. The contracts
make these boundaries explicit for reply operations, mailbox claims and
inputs, source custody, producer settlement, tool authority, native attempts,
embedded attachments, lifecycle state, and interruption receipts. Runtime
rotation evicts stale reply generations and records global-lane wait progress;
agent deletion closes the same admission boundary and drains retained
operations; interruption captures exact owners before aborting and exposes
bounded release receipts. These sources map to the existing Session Turns
Controller lifecycle, state, admission, and custody owners; they do not create
a second registry, scheduler, lifecycle authority, or cancellation truth.
Rust implementation, wiring, runtime execution, and acceptance remain
unverified.

## 134. Session controller lifecycle observation, projections, and bounded diagnostics (2026-10-29)

The locked `session-controller.lifecycle-diagnostics.ts`,
`session-controller.lifecycle-observation.ts`, and
`session-controller.lifecycle-projections.ts` were read in full. Observation
waits preserve caller abort reasons, delay activation until retiring sources
settle, aggregate exact effect/operation/source custody, and treat handoff
tokens as one-shot references on the existing effect rather than a new
admission. Projections resolve source-backed identities from captured
producers, allow only mutation borrowers or finishing captured turns through a
closure, select claims/effects/operations by physical store and aliases,
collect active targets for restart and agent-deletion drains, classify
restart-recoverable owners by exact abort reason and lifecycle generation, and
capture interruption owners once so a successor cannot be rediscovered during
abort. Diagnostics remain metadata on the same owner: hashed identities,
bounded holder/watcher and rate budgets, trace context, queue phases, slow
waiter/operation records, and phase logs cannot admit, cancel, release, or
alter cleanup. These sources map to the existing Session Turns Controller
observation, projection, and diagnostics owners; they do not create a second
selection, settlement, or observability authority. Rust implementation,
wiring, runtime execution, and acceptance remain unverified.

## 135. Session controller async ownership, cleanup escape, and terminal producer fencing (2026-10-29)

The locked `session-controller.context.ts`,
`session-controller.cleanup-context.ts`,
`session-controller.execution-custody.ts`,
`session-controller.native-attempt.ts`,
`session-controller.owner-context.ts`,
`session-controller.source-turn.ts`,
`session-controller.terminal-producer-fences.ts`, and
`session-controller.test-support.ts` were read in full. One AsyncLocalStorage
owner carries the exact operation, claim, source, effects, and mutations;
claim entry validates the live mailbox owner, source cleanup can leave turn
custody without inheriting it, and queued-source settlement excludes only the
current turn input. Borrowed execution retains completion until every borrower
releases; native attachments require the exact operation and preserve only
bounded backend/presentation facts after detach; source-turn binding refuses
stale, aborted, or replaced operations. Terminal producer fences capture
exact persisted writers, revoke them only while the owner is current, retain
failure details, and mark cleanup blocked when fencing cannot be proven. Test
support resets the same canonical storage rather than introducing a parallel
registry. These sources map to the existing Session Turns Controller async
ownership, native-attempt, cleanup, and producer-fence owners; they do not
create a second execution authority or settlement truth. Rust implementation,
wiring, runtime execution, and acceptance remain unverified.

## 136. Session mailbox cleanup, predecessor restoration, and native task claims (2026-10-29)

The locked `session-controller.mailbox-cleanup.ts`,
`session-controller.mailbox-predecessor.ts`, and
`session-controller.mailbox-task.ts` were read in full. Mailbox clearing
captures only authorized, non-current inputs, detaches matching summary
sources, preserves reentrant successor generations, aborts and retires each
source in order, and keeps committed cancellation custody intact; restart
drain binds to the existing gateway signal and cleans waiting inputs without
stealing active producer ownership. A pre-execution claim can reserve one
predecessor, reorder selector and cancellation sequence monotonically, and
restore the original claim only after the predecessor settles and all claim
custody invariants still hold; abort or loss of custody fails closed. Native
task claims reuse the physical mailbox selector, reject consumed/injected or
already-requested sources, and start only the existing claim without creating
another runnable input. These sources map to the existing Session Turns
Controller mailbox cleanup, predecessor, and native-task owners; they do not
create a second queue, cancellation authority, or task scheduler. Rust
implementation, wiring, runtime execution, and acceptance remain unverified.

## 137. Session mailbox identity, reservation, adoption, and physical retargeting (2026-10-29)

The locked `session-controller.mailbox.ts` was read in full. The mailbox is
created only on the existing Session Controller entry and exposes one ordered
sequence for follow-ups and native producers; claim, drain, in-flight, queue
items, summary state, and recent-source views all derive from that mailbox.
Reservations capture store scope, agent/session incarnation, protocol/source
identity, cancellation custody, and immutable queue policy before prompt or
attachment preparation. Repeated reservation IDs join only an equivalent live
source; conflicting protocol, policy, physical owner, or target state fails
closed. Gateway adoption is allowed only for an unbound preparing source with
matching owner and live authority, while cancellation signals are forwarded
into the same source controller. Source lookup is physical-store scoped and
does not create a mailbox for rejected redelivery. Native task submission,
source binding, and retargeting all reuse the same selector and custody;
retargeting is limited to an unbound preparing source and preserves store
boundaries, sequence order, and cancellation identity. These paths map to the
existing Session Turns Controller mailbox/reservation owner and do not create a
second queue, identity registry, or native-task scheduler. Rust
implementation, wiring, runtime execution, and acceptance remain unverified.

## 138. Session controller storage, queries, waits, and exact session-reader projection (2026-10-29)

The locked `session-controller.storage.ts`,
`session-controller.queries.ts`, `session-controller.wait.ts`, and
`session-controller.session-reader.ts` were read in full. One global storage
owner holds controller entries, alias/store indexes, RPC source references,
operation admissions, backend generations, message-injection attempts, and
after-clear/successor barriers; test cleanup closes watchdogs and waiters
through that same storage instead of replacing it. Queries distinguish logical
key busy state from one physical owner, reject ambiguous session IDs, retain
backend run IDs after detach until settlement, and resolve protocol run IDs
only when exactly one live operation owns them. Idle and admission waits are
bounded, abort-aware, prune their waiters, merge database-scoped lineage
sources across repeated barriers, and preserve the distinction between
follow-up and successor admission. Session readers borrow only the exact
admitted operation's reader, target, actor, and transition callback. These
sources map to the existing Session Turns Controller storage, query, wait, and
session-admission owners; they do not create a second registry, wait authority,
or session reader. Rust implementation, wiring, runtime execution, and
acceptance remain unverified.

## 139. Session controller message injection, join authority, and delivery custody (2026-10-29)

The locked `session-controller.backend-contracts.ts`,
`session-controller.message-injection-backend.ts`,
`session-controller.message-injection-failure.ts`,
`session-controller.message-injection.ts`, and
`session-controller.message-join.ts` were read in full. Backend contracts
separate captured execution identity, requester/tool authority, transcript
commit, source delivery mode, image/media support, pending-question custody,
and V1/V2 compatibility. Injection captures the exact physical backend
attachment, rejects stale/replaced/non-running owners, composes fresh policy
preparation at the final sink, distinguishes acceptance from transcript
commitment, shares only idempotent receipts, records participants once, and
retains indeterminate outcomes without authorizing cancellation or replay.
Failure classification preserves authority refusals, unsupported sinks,
policy mismatches, target loss, and unconfirmed commitment as distinct
outcomes. Join preparation verifies requester provenance and the incumbent
route, supports only explicitly compatible collaborative steering, preserves
fallback-route identity, and returns valid input to follow-up custody when the
captured owner ends. These sources map to the existing Session Turns
Controller backend, message-injection, join, tool-authority, and delivery
owners; they do not create a second execution sink, permission authority, or
transcript commitment path. Rust implementation, wiring, runtime execution,
and acceptance remain unverified.

## 140. Session controller barriers, mutation preemption, and operation recovery (2026-10-29)

The locked `session-controller.barriers.ts`,
`session-controller.execution-selection.ts`,
`session-controller.mutation-preemption.ts`,
`session-controller.native-runtime.ts`,
`session-controller.operation-recovery.ts`, and
`session-controller.producer-custody.ts` were read in full. Follow-up and
successor barriers retain one source per physical store, publish only after
the exact owner clear/handoff boundary, keep failed handoffs fenced, and merge
after-clear delivery callbacks without hiding same-store maintenance. Execution
selection captures admitted session/config/runtime/auth facts immutably before
steering. Mutation preemption captures exact claims, inputs, operations, and
effects, applies the configured wait/abort policy through the existing Stop
owner, and bounds settlement without releasing the mutation body early. Native
waits join the captured attempt and controller owner rather than a same-ID
successor. Operation recovery revalidates owner, key, session, agent, backend,
lease, and effect custody before watchdog recovery, cleanup, terminal producer
fencing, or admission retirement; independent producers retain cancellation
cleanup on exact sources and record failures without borrowing the parent
turn's cancellation authority. These sources map to the existing Session
Turns Controller barriers, execution selection, mutation, native runtime,
watchdog recovery, and producer-custody owners; they do not create a second
barrier, stop, recovery, or cancellation authority. Rust implementation,
wiring, runtime execution, and acceptance remain unverified.

## 141. Stop/watchdog owner read-only audit retained pending controlled inventory writer (2026-10-29)

The locked upstream files `session-controller.stop-capture.ts`
(`75a3fa0ba37d16d2abe84de8b83ddb537ec249fb`, 293 lines),
`session-controller.stop-runtime.ts`
(`18db2d67e36691df0e22edf993ceb3ace5a83120`, 156 lines),
`session-controller.stop.ts`
(`aa041cd8132b21bcfd82780cc3fedf16c35e1d6c`, 714 lines),
`session-controller.watchdog-state.ts`
(`a51606630267f949e75dd3b03b9ba81971037437`, 92 lines), and
`session-controller.watchdog.ts`
(`26071e9aaaff0d8653dddfb7a6ce60031af8cf9c`, 631 lines) were read in full.
The read confirms one captured Stop sequencer for queued inputs, active
operations, external parents, child cancellation, local-model waits, cutoff
publication, hooks, and settlement; watchdog state and timers remain attached
to one exact operation, with semantic/transport progress, owned waits/tools/
requests, execution and cleanup deadlines, recovery blocking, model recovery,
and current-owner revalidation at each asynchronous side effect. This section
is intentionally read-only evidence: because the former local inventory writer
is frozen, these rows are not promoted here to canonical `read-complete` or
ledger `source_refs`; no generated shard/index state was altered and no
completion flag was advanced. Rust implementation, wiring, runtime execution,
and acceptance remain unverified.

## 142. Session controller diagnostics, RPC source contracts, and public facade read-only audit (2026-10-29)

The locked upstream files `session-controller.diagnostics.ts`
(`aa0f5f0af3e918c617a15f77ccf0c949110047f7`, 43 lines),
`session-controller.rpc-sources.types.ts`
(`c3c114eb48bcaf43d00d861c0b0cb8191b6bf2b8`, 57 lines),
`session-controller.ts`
(`ce4dc474334bc4ba2fd94669009d8046c2bea7bf`, 44 lines), and
`session-controller.stop-authority.test.ts`
(`d1094d14b3a170e4f3ac76bed29a5eda36bdeb64`, 42 lines) were read in full.
Diagnostics report source-claim blockers through the existing bounded phase
logger. RPC source contracts keep byte-exact input correlation while carrying
source-local queue deadlines, lifecycle generation, operational instance,
approval lease, provider/auth identity, terminal persistence state, and abort
diagnostics; running timeout and cancellation remain owned by the execution
and input owners. The public facade only re-exports the existing operation,
injection, steer, wait, barrier, source-turn, and Stop owners. The authority
oracle proves Stop revalidates requester authority at each captured effect and
does not interrupt later effects after authority loss. This is read-only
evidence pending a controlled inventory writer: no generated shard/index state,
ledger ref, or completion flag was changed. Rust implementation, wiring,
runtime execution, and acceptance remain unverified.

## 143. Session controller RPC source lifecycle, steer, and tool-authority read-only audit (2026-10-29)

The locked upstream files `session-controller.rpc-sources.ts`
(`2ed8a2e48efac2363f5c84b3c193aaba2e353311`, 542 lines),
`session-controller.steer.ts`
(`dccd72cce94a84b5c9987b12409f50218e9fdb25`, 208 lines), and
`session-controller.tool-authority.ts`
(`b6f8cff3f27d7b93369805a9b413b32d71b66d29`, 546 lines) were read in full.
RPC source ownership is selected by exact run, captured Gateway scope, signal,
physical controller target, lifecycle generation, and session identity; source
registration rejects same-Gateway duplicates, retirement remains coupled to
input settlement, and queue/execution state, deadlines, renewal, cancellation,
and active-session projection remain owned by the exact source/operation.
Steer admission reserves behind older inputs, revalidates source and tool
authority before and during preparation, combines abort custody, and classifies
accepted, rejected, and indeterminate native outcomes so an unconfirmed commit
is never replayed. Tool authority retains admitted participants, freezes the
operation snapshot and route, binds async preparation to the captured owner,
rechecks lifecycle/backend/route/fingerprint at each await, preserves fallback
policy, and closes retained authority on operation end. This is read-only
evidence pending a controlled inventory writer: no generated shard/index state,
ledger ref, or completion flag was changed. Rust implementation, wiring,
runtime execution, and acceptance remain unverified.

## 144. Session controller reference-model test support read-only audit (2026-10-29)

The locked upstream `session-controller-model.test-support.ts`
(`905982150067f05f3dacdef42c41435faee2d0d1`) was read in full. It is a
test-only reference model rather than a runtime controller: the pilot state
models one operation generation, phase, slot/writer custody, cancellation,
live authority, compaction, injection availability, and per-input offered /
accepted / indeterminate / rejected / failed custody; its transition oracle
keeps stale generations and revoked authority from becoming replayable. The
mailbox model covers duplicate suppression, bounded overflow retirement, and
durable completion deduplication, while deterministic sequence generation
exercises receipt races and terminal cleanup. The canonical ledger currently
records this source range as ending at line 238, whereas the locked blob reads
249 lines; the SHA matches, but the range metadata remains a stale item for a
controlled writer reconciliation. No generated shard/index state, ledger ref,
or completion flag was changed. Rust implementation, wiring, runtime
execution, and acceptance remain unverified.

## 145. Worker deploy entrypoints and sealed-runtime registry read-only audit (2026-10-29)

The locked upstream worker deploy files were read in full:
`worker-deploy-browser-runtime.ts`
(`963014da81018e74d57f58bb96ef106ffe5331db`, 9 lines),
`worker-deploy-client-credentials.ts`
(`8fae136772f103bd016dc5397afd3e9a32688acb`, 2 lines),
`worker-deploy-entry.ts`
(`930ab46ba9ce13d3ed58cd801101f172455e0cc6`, 54 lines),
`worker-deploy-file-tool-planning.ts`
(`9eaaff72594868ca2d763f4421fa2438ac11c524`, 2 lines),
`worker-deploy-file-tool-read.ts`
(`d068828c308c292d69ad30c8df378190ac6c6101`, 2 lines),
`worker-deploy-image-processor.ts`
(`d7de2a0f1ffb37addf9567918589692290543497`, 2 lines),
`worker-deploy-runtime.ts`
(`7f5bf2454fe8b42139c12f4aa8164e66926b3dac`, 29 lines),
`worker-deploy-sqlite-source-revision.ts`
(`cca4e0d487d63828d3b4e0abbe7659a88e7ac6ed`, 2 lines),
`worker-deploy-sqlite-store.ts`
(`24ae90cdc8e431d96ab6cb81b65c44b79701fda3`, 8 lines), and
`worker-deploy-state-read.ts`
(`130eeb3833390e51d1dca3c983e5b09669b9fce4`, 2 lines).
The deploy entry accepts only the four declared internal flags, rejects
duplicates and unknown arguments, requires managed mode for native inference,
keeps prewarm separate from execution, formats operator-safe failures, and
destroys/disconnects startup resources on every exit path. The sealed-runtime
registry maps bundle-hash-controlled entrypoints for credentials, file tools,
GitHub execution, image processing, SQLite/source revision, state read and
shared state; the thin deploy modules only bootstrap their one worker owner.
Browser deployment is explicitly fail-closed when the build has not composed
its runtime. This is read-only evidence for the existing Node Host/Runner
bundle and process-composition owners: no generated shard/index state, ledger
ref, or completion flag was changed. Rust implementation, production wiring,
packaged execution, and acceptance remain unverified.

## 146. Node pairing admission and worker-bundle provenance read-only audit (2026-10-29)

The locked upstream `src/gateway/node-registry-pairing.ts`
(`6ee2e1669f2315ad414383b9153d8ff73f0bffbf`, 59 lines),
`src/gateway/server/ws-connection/connect-node-session.ts`
(`51aad1c06c2b8caa84753dfd53c5755767ff10bf`, 221 lines), and
`src/shared/worker-bundle-hash.ts`
(`359539d729e3100c34f2faba10f3db77dccc0a44`, 68 lines) were read in full.
Pairing leases bind the published node to pairing identity and optional
generation, reject missing/current-state mismatches, and fail closed when the
pairing-state oracle throws. Node admission begins one cleanup claim before
reconciliation, re-reads device approval provenance, rate-limits new pairing
requests, distinguishes same-host silent / SSH-verified / bootstrap consent
from network-only trust, and never lets stored local provenance authorize a
remote or browser connection. Only the initial capability surface may inherit
eligible approval; later manifest changes remain subject to the declared and
legacy-protocol-filtered capability/command ceiling. The bundle contract
defines immutable artifact names and modes, a SHA-256-addressed archive path,
safe chunk naming, and deterministic manifest hashing over version/path/mode/
size/digest. These are source oracles for existing Fabushi FDN, Gateway,
Node Host and bundle owners; no generated shard/index state, ledger ref, or
completion flag was changed. Rust implementation, production wiring,
packaged execution, and acceptance remain unverified.

## 147. Agent admission, session persistence, and execution-owner oracle detail (2026-10-29)

The locked upstream test/support sources were read in full:
`src/gateway/agent-turn/agent-admission-controller.test.ts`
(`0a0db7d9d1a15362587254f9d9a406731acfe8ff`, 133 lines),
`src/gateway/agent-turn/agent-session-persist.test.ts`
(`324f34295a4d0c82f8e748ffc9853d6facc3d10e`, 190 lines), and
`src/gateway/agent-turn/agent-run-execution-phase.owner.test-support.ts`
(`79a7407e202c94bd87e3c6ecd5d1650f89115cf2`, 232 lines). The admission
oracle serializes marker admission against same-session replay, refreshes
after the lifecycle lease while preserving the committed postimage, and
rejects an archived or changed expected session. The persistence oracle
ensures sandbox provenance and creator authorization are committed without
unrelated session-store SQL, retains interrupted recovery ownership only for
the exact queued/non-queued owner, and reports authority revoked during async
preparation. The execution support fixture binds one lifecycle generation,
physical target, RPC source, abort controller, Gateway work admission and
follow-up completion custody, including cleanup when the source settles. These
are focused source oracles for existing Session Turns/Gateway owners; no
generated shard/index state, ledger ref, or completion flag was changed.
Rust implementation, production wiring, executable verification, and
acceptance remain unverified.

## 148. Remaining Agent turn, transcript-fence, and routing oracle closure (2026-10-29)

The remaining locked oracle sources were read in full:
`agent-run-execution-phase.owner.test.ts`
(`244bae9f1bd5f18290fbd32cf9f27b513e8f7509`, 789 lines),
`agent-run-execution-lineage.test.ts`
(`9ac4626e09e4d647ce571ea9e9ce4aa92663e61e`, 109 lines),
`agent-content-phase.test.ts`
(`8ac27c8ce6bf1e39a31a3764c6e56244a942caf7`, 223 lines),
`agent-delivery-phase.test.ts`
(`a1117880331487b783d3a8e434d371b894e70346`, 167 lines),
`agent-run-user-turn.test.ts`
(`2aaba7e64dc915e33d6c6754cb7f51aedf42e25b`, 263 lines),
`agent-run-subagent.test.ts`
(`c96d4da8bac5e266771356b60685367ffb0516d7`, 238 lines),
`agent-run-dispatch.owner.test.ts`
(`e60380357cf976ed5ad7921c615572f6eb4f8be7`, 642 lines),
`agent-run-dispatch.incognito.test.ts`
(`ecdbdd42f6bd65fb7c2fec6028075be19130643a`, 154 lines),
`agent-run-dispatch.test-support.ts`
(`50e2c2a810d653c64e04583e0f2062e1c66804a2`, 54 lines),
`plugin-sdk/context-engine-transcript-runtime.test.ts`
(`53843b0ec5c75b554aeacaefa2cf6c7feeb960a7`, 192 fetched lines), and
`poll-params.test.ts`
(`bf7918df646d0e185f97fc3551a35c06c6cdeafc`, 33 fetched lines).
The execution owner oracle keeps captured source/operation ownership through
terminal publication, disposal, cleanup failures, self-drain, workspace
resume, provider context, runtime-generation borrowing, follow-up successor
replacement, and private replay diagnostics. Lineage facts preserve parent
context/execution/run, grants, policy and runtime-assurance evidence without
inferring unsupported ACP callbacks. Content admission fences session/model
selection around asynchronous reads; delivery observes only the selected
Gateway/store scope and suppresses private diagnostics; user-turn preparation
refuses stale admission and cleans committed/unknown/unretained media exactly
once. Subagent and dispatch oracles preserve requester-bound completion,
successor custody, terminal-save ordering, registration replacement fencing,
canonical result classification, yield ownership, incognito redaction, and
primitive-throw preservation. The context-engine oracle verifies reset-window
tail selection, paired tool results, generation/reset cursor invalidation and
read fences; the poll oracle rejects schema-padded duration/multi defaults as
poll intent while accepting real poll question/options. These are focused
oracles for existing Session Turns, Gateway, Plugin SDK and message-routing
owners; no generated shard/index state, ledger ref, or completion flag was
changed. Rust implementation, production wiring, executable verification, and
acceptance remain unverified.

## 149. Runtime manifest production-caller audit (2026-10-29)

The exact current Rust owner was re-read at
`source/host/src/extensions/node_host/runtime_manifest.rs`. `NodeHostManifest::build`
constructs a deterministic command/capability projection from available-command,
Claude, desktop-stream, and allowlist inputs, and always sets `is_ephemeral` to
`false`. The module is exported by `extensions/node_host/mod.rs`.

The production integration search was then performed across the Host extension
registry, Host production extension assembly, and
`source/node-agent-coordinator/src`. It found no caller of `NodeHostManifest::build`,
no `NodeHostManifest` consumer, and no manifest/runtime slot in
`CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS`, `HOST_EXTENSION_ORDER`, or the
coordinator. The existing registry therefore cannot be treated as a manifest
consumer, and the pure builder cannot be promoted from `wiring=not-wired` to a
shipping capability advertisement.

This confirms the active `docs/specs/node-host-runtime-embedding.md` boundary:
manifest construction must be composed by one future live node-host transport /
invocation owner, while pairing and RPC remain projections and existing local
exec, Computer, stream, MCP, and worker owners remain the side-effect owners.
No product code, generated shard/index state, parity-ledger status, completion
flag, or verification result was changed by this audit.

## 150. Runtime cancellation owner seam (2026-10-30)

`NodeHostInvocationRegistry` already owned generation-fenced cancellation, but
the canonical `NodeHostRuntimeState` composition exposed only admission and
settlement. The runtime owner now exposes `cancel_invocation` and parsed
`NodeInvokeCancel`/`NodeInvokeRequest` adapters so a future Gateway
transport/session owner can bind both admission and cancellation through the
same lease-bound composition before sending or settling node frames; a stale
predecessor, wrong node, or duplicate callback is rejected by the existing
generation/state fence. Focused contract cases record exactly-once
cancellation, reject later settlement, and reject cross-node request/cancel
frames.

This is an implementation seam only. It does not open a socket, publish a
manifest, execute a remote command, or establish production transport wiring;
the node-host responsibilities remain `wiring=not-wired`, verification
`not-run`, and acceptance `pending` until the live transport owner and exact
Actions/device evidence exist.

The shipping local `node.invoke(system.run)` producer now uses the typed
request adapter before entering the existing local execution owner. This is a
real local producer-to-owner generation/identity fence, but it does not widen
the supported command surface or change the remote-node fail-closed result;
remote transport, manifest publication, and device acceptance remain pending.

## 151. Invocation parameter-size admission (2026-10-30)

The strict node invocation parser now rejects `paramsJSON` above 256 KiB
before JSON allocation/parsing, while retaining the existing 16 KiB bound for
ordered input payloads. This closes the local R3 parser-size negative case;
transport frame limits, live invocation execution, and exact-head verification
remain separate pending responsibilities.

## 152. Latest live node-host protocol discovery (2026-10-30)

Read-only inspection of the latest OpenClaw live candidate
`c49e07385f8d17601efbed7746381213d5326ba9` found the following current
protocol facts: `src/node-host/client.ts` (`fa998d7a2c98c9db4ebc1cc1e891898ca83d15f5`)
publishes `node.invoke.result` with the original `id`/`nodeId`, required
`ok`, optional `payload` or `payloadJSON`, and an optional error object whose
details are capability-gated; failed result delivery is best-effort. The
request shape remains `paramsJSON`, `timeoutMs`, `idempotencyKey`, and
`sessionKey` in `src/node-host/invoke-types.ts`
(`720a2d16da2c2e653123b89422fa7e501fa92319`).

The same candidate's `runtime-manifest.ts`
(`856e0f728d07421741a8d3f16aade54b12e284cb`) derives sorted commands and
capabilities from plugin registration, command allowlists, Claude/apps/desktop
availability, ephemeral restrictions, and path environment; the simpler Rust
manifest builder is therefore not yet a complete live manifest owner.
`connection.ts` (`da661abb04c1541fcc58b53a9d25d2aa5d565981`) additionally
fences optional publications and timers by connection generation. These are
latest-candidate discovery facts only: the source lock/inventory still require
Actions rebaseline, and no Rust result transport, dynamic manifest caller, or
exact-head acceptance is claimed.

## 153. Dynamic manifest composition seam (2026-10-30)

`NodeHostManifest::build_with_plugin_manifest` now composes available builtin
commands with current plugin commands/capabilities, applies the command
allowlist, sorts/deduplicates the result deterministically, and marks an
ephemeral surface so restricted capabilities are not advertised. The legacy
builder remains compatible for existing callers. This improves the manifest
owner contract against the latest live source, but no production caller yet
feeds the real plugin registry or publishes the snapshot through a live node
transport; wiring, verification, and acceptance remain pending.

Manifest plugin commands and capabilities now also pass a bounded text
admission check: surrounding whitespace is normalized, empty/control-bearing
entries and entries over 256 characters are omitted before sorting and
publication. This keeps the future hello/manifest boundary fail-closed without
claiming a real plugin registry or remote publication path.

## 154. Typed invocation-result envelope (2026-10-30)

`invoke_protocol.rs` now models the live result shape with original `id` and
`nodeId`, `ok`, mutually selected `payload`/`payloadJSON`, and structured
errors. `without_permission_details` applies the live capability gate so
permission details are removed when the peer lacks the dedicated capability;
stable error code/message remain available. Serialization-focused contract
coverage preserves IDs and both detailed/redacted outcomes.

This is still a transport-neutral protocol owner: no Gateway result sender,
remote invocation executor, best-effort delivery path, or exact-head runtime
acceptance has been added.

## 155. Bounded invocation lifecycle composition (2026-10-30)

`NodeHostInvocationRegistry` now enforces a 64-active-invocation bound and
reopens capacity after terminal settlement. `NodeHostInvocationRuntime`
composes lease-bound request admission, node/id-checked ordered input,
generation-fenced cancellation, and exactly-once result settlement with
payload/payloadJSON exclusivity. Cross-node input/result cases and duplicate
terminal settlement are rejected by focused contracts.

This composition remains transport-neutral and does not provide socket
ownership, command execution, result delivery, or remote device acceptance;
those production responsibilities remain pending.

## 156. Transport-owner boundary audit (2026-10-30)

The current Host crate has no direct WebSocket client dependency or shipping
Node Host transport caller. `node_host::gateway_connection` owns only URL,
origin/mount, token-scope, and candidate-admission policy; it deliberately does
not open a socket, attach TLS or edge credentials, negotiate the Gateway
session, or publish frames. Existing WebSocket implementations under the
Codex app-server and messaging owners are separate transport boundaries and
cannot be silently reused as a Node Host Gateway owner.

Therefore the new invocation lifecycle remains correctly transport-neutral.
Adding a dependency or caller now would require a repaired owner/contract
decision covering credential injection, negotiated protocol, request/event
dispatch, reconnect, and shutdown ownership. No transport wiring or remote
acceptance is claimed from this audit; the missing owner boundary is an
explicit next implementation requirement rather than inferred connectivity.

## 157. Local producer uses typed invocation lifecycle (2026-10-30)

The shipping local `node.invoke system.run` path now starts
`NodeHostInvocationRuntime` with the existing local session lease, executes
through the existing `HostLocalExecExtension`, builds a typed success or
`EXECUTION_ERROR` `NodeInvokeResult`, and completes the lifecycle exactly once.
The public Gateway response remains the existing local compatibility shape;
the typed envelope is now the canonical invocation terminal record rather than
an independent response owner. Remote transport/result delivery and real
device acceptance remain pending.

## 158. Computer side-effect generation admission repair (2026-10-30)

The Computer dispatch owner now rejects `Act` and `Snapshot` requests that
omit or blank `provider_generation`, using the same opaque-generation mismatch
error as stale values. `Capabilities` remains able to query the current
executor generation without an input generation. This closes the pre-side
effect currentness gate and adds a negative contract case; it does not choose
or wire the production Computer executor, remote transport, or device lane.

## 159. Input-buffer failure cancels the invocation (2026-10-30)

`NodeHostInvocationRuntime::dispatch_input` now couples a bounded pre-handler
input-buffer overflow to cancellation of the same lease and generation-bound
invocation. Stale or duplicate sequence frames remain non-terminal rejections;
only the buffer's explicit failure state triggers cancellation. A focused
contract proves that a capacity overflow is followed by failed terminal result
settlement, so a failed invocation cannot later publish a normal completion.

This is lifecycle ownership only. It does not yet stop an already-running
side-effect provider, deliver a cancellation frame over a live transport, or
establish remote device acceptance; those owners remain pending.

## 160. Canonical node.event payload envelope (2026-10-30)

The live candidate's `node-event-params.ts` builds the event envelope with the
event name and `payloadJSON`, using a JSON string for a defined payload and a
JSON null field when no payload is supplied. Rust now models that exact
transport-neutral envelope in `event_protocol.rs`, with focused serialization
contracts for nested Unicode payloads and the absent-payload case.

This does not select event producers, publish frames, own connection
generation, or establish remote event delivery; those remain with the future
Gateway transport/session owner.

## 161. Generation-fenced node.event publication gate (2026-10-30)

`NodeEventPublication` now admits an event only against the current
`NodeHostSessionLease` and requires a final `is_current` check immediately
before a future transport sends it. Disconnect or successor-session admission
retires the publication, matching the live connection owner's generation fence
for host-stat and worker-process event callbacks.

The gate stores no second session registry and performs no socket I/O. A live
Gateway caller, event producer selection, retry policy, and remote delivery
receipt remain unimplemented and are not claimed by this slice.

## 162. Live-source supersession note (2026-10-30)

The historical live-main observations in earlier sections of this dossier
remain preserved as dated discovery evidence. They are superseded for current
source navigation by the later read-only candidate
`c49e07385f8d17601efbed7746381213d5326ba9` / tree
`fcd6e0b9b42da1e3d39ca89c69ac779294cc3bfb`, recorded in the project
`SOURCE_OF_TRUTH.md` and main equivalence Spec. This note does not promote the
candidate to the implementation lock, rewrite prior blob claims, or close
inventory, dependency, Actions, or production-transport acceptance.

## 163. Local Host event bus is not the node.event transport (2026-10-30)

The current Rust `SandHostEventBus`/`GatewayEventHub` publishes Fabushi
product events using the existing `channel` plus `payload` envelope and feeds
the local Gateway/SSE projection. The Node Host `event_protocol.rs` contract
instead produces the remote node `event` plus optional `payloadJSON` request
parameters and applies a session-generation fence immediately before a future
Gateway send. These envelopes have different owners and consumers; routing a
`NodeEventPublication` through the local event bus would create a second
protocol or mislabel a local timeline event as remote node delivery.

No producer or live Gateway transport was added from this review. The next
valid wiring unit remains a real Node Host connection/session owner that takes
the fenced publication at the send edge, with reconnect, cancellation,
backpressure and delivery receipts. The current event protocol and generation
gate therefore remain transport-neutral implementation evidence, not remote
event acceptance.

## 164. Candidate endpoint admission hardening (2026-10-30)

`NodeHostGatewayConfig::validate` now rejects empty/control-bearing hosts,
URL credential/separator characters, unsafe or oversized context paths, and
invalid TLS-fingerprint text before candidate selection. The candidate router
requires every configured endpoint to pass that validation, and the URL
formatter and base URL helper return explicit errors for unsafe override paths
instead of constructing a malformed candidate. This is a transport-neutral
security boundary; plaintext is accepted only for loopback hosts, while
non-loopback candidates require TLS. It does not open sockets, resolve
SecretInput material, pin a peer, or claim Gateway/device acceptance.

## 165. V3/V4 protocol negotiation seam (2026-10-30)

`protocol_negotiation.rs` now captures the source-backed hello rule for the
currently audited V3/V4 dialects: an offered range must be ordered and wholly
supported, the server version must lie inside that range, and the selected
dialect is bounded by the server and offered maximum. Inverted, future and
out-of-range values fail closed. The seam does not create the challenge-bound
connect envelope, sign device metadata, own version persistence, or open a
Gateway socket; live negotiation and device acceptance remain pending.

## 166. New live-main candidate discovery (2026-10-30)

A read-only `git ls-remote`/object acquisition observed OpenClaw `main` at
commit `d67aa608643f6583a6c6102c522043794aafe064`; its root tree is
`1999145af25baef58520f40daf89e0be3b39988a0`, and the fetched recursive listing
contains 227 paths under `src/node-host/` and `crates/openclaw-node-host/`.
This differs from the current implementation lock
`ad8d12f126d29b0db204187adf5092db80294e70` / tree
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16` and supersedes neither the lock nor
its inventory. No source rows, dependency closure, parity disposition,
Actions receipt, or completion flag was changed; the new commit requires the
formal rebaseline workflow before it can become implementation authority.

## 167. Candidate manifest native-auth delta (2026-10-30)

The newly acquired candidate's `src/node-host/runtime-manifest.ts`
(`856e0f728d07421741a8d3f16aade54b12e284cb`) imports and derives
`NODE_CLAUDE_NATIVE_AUTH_CAPABILITY` in addition to the Claude skills
capability. The current Rust manifest seam only derives
`NODE_CLAUDE_SKILLS_CAPABILITY`, so blindly treating the current seam as
complete would omit a candidate capability. This is recorded as an
implementation-impacting source delta only: the candidate is not the
implementation lock, and no Rust manifest behavior, inventory row, or flag was
changed until formal rebaseline and dependency closure are complete.

The candidate dependency read resolves the capability to
`src/infra/node-commands.ts@cda6b5ce41134002012e421d21ae4f5b37a10faa`, where
`NODE_CLAUDE_NATIVE_AUTH_CAPABILITY` is the exact wire value
`claude-cli-native-auth-v1`. The manifest's plugin availability path also
resolves through `src/node-host/plugin-node-host.ts@332de300280a11173f9eff5db59e1b5bd919ebb1`,
which loads the active plugin registry and retains command work across
disconnects. This proves the delta is not merely a display label: the final
implementation must map native-auth capability admission and plugin lifecycle
to their existing owners. No Rust constant or plugin consumer was changed
while the candidate remains unpromoted.

## 168. Latest live-main candidate drift (2026-10-30)

The read-only upstream check observed OpenClaw `main` at
`c32556b0cea7f63b33db0ac393affd71ecb0015b` with root tree
`e08d361d83ca9cae55aa93d49462d991d0a39efc`; the scoped
`src/node-host/` plus `crates/openclaw-node-host/` listing remains 227 paths.
This supersedes d67 only as a newer discovery pointer, not as implementation
authority. No lock, inventory, dependency closure, parity row, Actions
receipt, or completion flag was changed.

The exact path diff between the acquired d67 and c325 objects reports zero
changes under `src/node-host/` and `crates/openclaw-node-host/`. This narrows
the latest upstream drift: the Node Host source identities and the recorded
native-auth delta remain applicable to both candidates, while the overall
source lock and dependency closure still require a project-wide rebaseline.

## 169. Local session manifest boundary audit (2026-10-30)

The shipping Host startup path creates exactly one local session lease for
`sand-local-node` through `NodeHostRuntimeState`. Its public session surface
contains only `system.run`; `node.list` and `node.status` project that same
generation-fenced snapshot, while disconnected retained rows are excluded from
the live list and do not make an approved remote device appear connected.
The local invoke producer admits the request against that lease and delegates
the side effect to the existing `HostLocalExecExtension`; non-local invoke
continues to fail closed because no canonical remote transport owner is
connected.

The `orderedReplicasV1` and `sendAcceptanceV1` values on this local snapshot
are the existing Host gateway capability constants, not evidence that the
transport-neutral `NodeHostManifest` builder is a live manifest owner. A
source audit found no production caller for `NodeHostManifest::build` or
`build_with_plugin_manifest`; the builder remains a future transport/manifest
composition seam. The `node.system_run` builder constant and the public
`system.run` session command therefore must not be treated as a completed
manifest publication path without an explicit protocol adapter and live
connection owner.

This audit records a truthful local boundary and does not widen commands,
capabilities, pairing, remote transport, or acceptance claims. Dynamic
manifest refresh, challenge-bound connection, remote invoke/cancel/result
transport, and device acceptance remain open.
