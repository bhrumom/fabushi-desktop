# Node-host runtime embedding and Gateway session owner

Status: active implementation specification, not full OCRM-M13 acceptance  
Owner: Fabushi host-runtime owner  
Last updated: 2026-10-29  
Related project: `projects/openclaw-rust` / OCRM-001  
Related task: P0 source-baseline-and-owner-audit; node-host runtime frontier

## 1. Context / problem

The pinned OpenClaw source contains a complete node-host runtime boundary: connection challenge/signing, negotiated command manifest, invocation/input/cancel/result correlation, reconnect classification, shutdown drain, and live connection state. Fabushi currently has pairing approval and local node RPC projection, but no shipping Rust node-host client/runtime or live remote-node registry. Treating pairing approval as a live connection, or adding a second transport/state owner inside the RPC handlers, would produce false equivalence.

The upstream Rust crate `crates/openclaw-node-host` is useful protocol evidence, but importing or copying it wholesale would create a second runtime boundary unless every responsibility is explicitly assigned to existing Fabushi owners.

## 2. Goal

Define and implement a Fabushi-native node-host runtime embedding that preserves the upstream observable node session contract while using existing host, credential, policy, worker, Computer, and transport owners. The result must expose live connection/activity facts only from the runtime transport owner and must be accepted only with exact-head GitHub Actions and real device/transport evidence.

## 3. Non-goals / out of scope

- Do not create a second Gateway client, node registry, pairing store, Computer state owner, or generic Agent loop.
- Do not infer `connected`, `active`, or executable commands from pairing approval.
- Do not import the upstream crate as an unreviewed standalone runtime or claim its tests prove Fabushi integration.
- Do not route remote `system.run` through the local-only execution owner.
- Do not implement desktop Computer side effects in the protocol dispatcher; use an approved existing owner or a separately approved adapter.
- Do not run local builds, tests, generators, schema tools, formatters, or linters for OCRM verification.

## 4. Requirements

- R1: Each connection attempt produces a challenge-bound node connect envelope with protocol, identity, command manifest, capability surface, and authentication owned by the existing credential/policy boundaries.
- R2: Device identity keys and issued device tokens remain owned by the existing secure credential/persistence owner; logs and diagnostics never expose secrets.
- R3: The session parser accepts the canonical `paramsJSON` wire field, preserves original JSON when required for result/provenance handling, rejects direct or ambiguous `params`, and bounds input sizes.
- R4: Invocation, ordered input, cancellation, and final result remain correlated by the original invocation id and node id; lost result transport is not replayed on a replacement connection.
- R5: Command manifest and Computer capability advertising are regenerated from current availability and never advertise unavailable commands.
- R6: Reconnect classifies retryable transport/runtime failures separately from terminal configuration, identity, protocol, pairing, or authentication pauses; backoff and retry-after are bounded.
- R7: Shutdown and replacement close the old session, cancel/drain owned invocation work, and prevent predecessor callbacks from mutating successor state.
- R8: A live node registry/transport owner is the sole authority for remote `connected`, `active`, manifest, and invoke routing facts; pairing remains approval/identity only.
- R9: Node-host Computer dispatch validates opaque generation and delegates side effects to an approved Computer executor with permission, currentness, and result-schema fencing.
- R10: Any sidecar or worker bridge is introduced only when an existing worker/supervisor owner requires it, with bounded framed protocol, authenticated channel binding, and no new durable product state.

## 5. Current state

- `DevicePairingOwner` provides durable identity and approval, not live transport.
- `source/host/app/src/main.rs` owns local node registration/invocation and currently reports approved remote nodes as paired but disconnected with no remote commands.
- `source/host/src/extensions/node_host/` contains protocol slices for connection, Computer dispatch, and runtime manifest, but the runtime has no shipping caller or live remote transport.
- Existing `NativeDesktopRemoteExecutor` and `execute_with_lease_fenced` own native Computer behavior; they are not generic node-host invoke executors.
- Pinned upstream revision is `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

## 6. Target state

One Fabushi-owned node-host runtime composes the canonical Gateway/transport owner, credential owner, command manifest owner, invocation runtime, and existing side-effect owners. Pairing and RPC handlers project facts from this runtime; they do not maintain parallel connection state. On disconnect or replacement, old invocation tails and callbacks are fenced before the successor is admitted. Remote commands remain empty/unavailable until the live transport owner positively supplies them.

## 7. Architecture and ownership boundaries

- **Credential/identity owner:** secure key/token persistence, device id, challenge signing, redaction.
- **Gateway transport/session owner:** websocket lifecycle, protocol negotiation, event delivery, request deadlines, live session registry, connection generation.
- **Node invocation runtime owner:** admission, bounded concurrency, cancellation, ordered duplex input/output, result settlement, and disconnect drain.
- **Manifest owner:** current command/capability registration and availability changes; it does not own transport truth.
- **Policy/side-effect owners:** local exec, Computer, PTY, desktop stream, portal stream, and worker/Claude handlers retain their existing authorization and process lifecycles.
- **Gateway RPC projection:** reads live registry facts and pairing facts; it owns no node session, heartbeat, or replay state.

The Rust `NodeHostSessionRegistry` is the Host-side projection boundary for the
transport/session owner. Its `connect`, `set_active`, and generation-fenced
`disconnect` operations are the only path that can create or retire a live
session snapshot. `DevicePairingOwner`, `node.register`, and RPC projection
may not call those operations or infer `connected` from approval. Until a real
transport caller exists, approved remote devices therefore remain
`paired=true, connected=false, active=false, commands=[]`.

Candidate connection policy is a separate, transport-neutral seam. A
candidate router may advance only after a pre-hello close where no connect
request was sent and no winner has been selected. A successful hello promotes
the current candidate and permanently disables fallback for that connection;
stopping the router makes all later candidate callbacks inert. The eventual
Gateway client remains the owner of sockets, TLS, edge-auth headers, and
drain/cleanup.

The upstream crate may be used as a contract reference or narrowly embedded primitive only after source-to-owner mapping proves it does not introduce a second owner.

## 8. Interfaces / contracts / data flow

The implementation must preserve these externally relevant contracts:

1. challenge → signed connect envelope → negotiated protocol/session;
2. session manifest/capabilities → live registry projection;
3. `node.invoke.request` (`id`, `nodeId`, `command`, `paramsJSON`) → bounded parsed invocation;
4. `node.invoke.input` sequence and `node.invoke.cancel` → owning invocation;
5. one terminal `node.invoke.result` success/error envelope bound to original ids;
6. connection generation → cancellation/drain fence for predecessor callbacks;
7. typed lifecycle events → observability and fail-closed RPC presence.

Direct `params` and ambiguous `params` + `paramsJSON` are invalid. A disconnected or retired session cannot advertise commands or settle a result on a successor session.

## 9. Constraints and non-functional requirements

- Security: TLS/endpoint scope, identity/token secrecy, command authorization, and Computer lease/currentness are fail-closed.
- Reliability: bounded retry/backoff, explicit terminal pause, no mutation replay after outcome-unknown transport loss.
- Compatibility: support the pinned OpenClaw node protocol range only where the existing Gateway client proves the negotiated version and schema.
- Resource safety: bounded messages, input/output buffers, in-flight invocations, worker lifetime, and shutdown grace.
- Evidence: all executable OCRM checks run in GitHub Actions against the exact accepted head; local inspection is read/edit/Git/API only.

## 10. Failure modes and edge cases

- stale/rejected device token, pairing-required, auth rate limit, protocol mismatch, TLS pin/configuration error;
- transport loss during invoke result, reconnect, manifest refresh, or shutdown;
- event lag, malformed frame, oversized `paramsJSON`, invalid sequence, duplicate invocation, and unknown command;
- stale Computer generation, missing executor, permission denial, lease loss, and invalid result schema;
- old session callback arriving after successor activation;
- worker/sidecar crash, held process cleanup, and cancellation that signals before physical work drains.

Each case must retain typed failure evidence and must not be converted into a clean success or inferred live presence.

## 11. Implementation strategy

1. Complete per-file source closure for `src/node-host/` and `crates/openclaw-node-host` against the pinned upstream revision, recording exact refs and rejected duplicate-owner candidates.
2. Reconcile the mapping with current Fabushi Host, Coordinator, credential, worker, Computer, and transport owners; repair this spec before source changes if ownership changes.
3. Implement the narrowest session/lifecycle seam behind existing owners, starting with live transport and generation fencing, then invocation/manifest composition, then side-effect adapters.
4. Add contract tests and Actions workflow coverage without fabricating provider/device success.
5. Verify exact-head Actions, packaged runtime, and real device/transport cells; update the parity ledger only from those receipts.

## 12. Verification / test strategy

- V1: source closure ledger records each in-scope upstream file as read, mapped, omitted with reason, or unresolved; no bulk-read claim substitutes for file evidence.
- V2: Rust/host contract tests cover connect envelope, strict invocation parsing, input/cancel/result correlation, lifecycle pause/backoff/drain, generation fencing, manifest gating, and redaction.
- V3: Actions run the focused tests and relevant regression suites on the exact changed head; local execution is not used as OCRM acceptance evidence.
- V4: packaged host connects to the real Gateway/device cell, advertises only available commands, completes a real invoke, and demonstrates reconnect/cancellation behavior.
- V5: negative real-device/provider cells prove stale token, unavailable command, stale generation, transport loss, and predecessor-result rejection fail closed.

## 13. Acceptance criteria / Definition of Done

- AC1: every applicable OCRM node-host source file is individually read and mapped to one existing owner or explicitly unresolved with evidence.
- AC2: the shipping runtime has one live node transport/registry owner and pairing/RPC code contains no inferred connection state.
- AC3: exact-head Actions pass all selected contract/regression checks with retained run ids and logs.
- AC4: packaged and real transport/device acceptance proves connect, manifest, invoke, cancel/reconnect, and result settlement.
- AC5: no fake Computer/system-run/stream success remains; side effects route through approved owners with currentness and permission fencing.
- AC6: parity ledger, dossier, and this spec agree on implementation, wiring, verification, and acceptance state; OCRM-M13 is not marked complete while any required evidence is absent.

## 14. Release / migration / rollback

Ship behind the existing host/runtime feature and deployment gates. Do not enable remote commands before live transport acceptance. Roll back by disabling the new runtime composition and retaining fail-closed paired-but-disconnected projections; never restore synthetic success payloads.

## 15. Observability / evidence

Retain redacted lifecycle events, connection generation, negotiated protocol, manifest digest/contents where safe, invocation ids/node ids, result transport outcome, exact source revision, Actions run ids, packaged artifact identity, and real device/provider receipts. Secrets, tokens, raw auth headers, and private payloads are excluded.

## 16. References / provenance

- User requirement: full OpenClaw→Fabushi Rust-native equivalence with exact source closure, existing-owner integration, production wiring, and OCRM acceptance.
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `docs/specs/openclaw-rust-equivalence-migration.md`
- `projects/openclaw-rust/module-map.md` OCRM-M13
- `docs/specs/node-rpc-remote-presence.md`
- `docs/specs/node-host-computer-dispatch-owner.md`
- pinned upstream `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`, especially `src/node-host/{runtime,client,invoke}.ts` and `crates/openclaw-node-host/src/{node,lifecycle,reconnect,identity,duplex}.rs`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1–R10 | pending | Runtime embedding not yet implemented and wired. |
| AC1 | in progress | Node-host Rust crate closure is being read in exact pinned chunks; inventory remains incomplete. |
| AC2–AC6 | pending | No live remote transport/device acceptance or exact-head acceptance receipt yet. |
