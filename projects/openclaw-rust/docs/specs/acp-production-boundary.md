# ACP production boundary

Status: active architecture/specification; implementation not started
Owner: existing Fabushi Gateway/Host/Runner owners
Related responsibilities: `OCRM-RESP-AGENT-ACP-COMMANDS-001`, `OCRM-RESP-AGENT-ACP-POLICY-001`
Fixed upstream: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`
Fixed tree: `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`

## 1. Context and problem

The fixed upstream subject contains an ACP command projection, ACP policy
guards, and a Gateway-backed NDJSON server. The relevant source identities are
recorded in `dossiers/agent-turn-execution-scoped.md`:

- `src/acp/commands.ts` — blob `ed35270d5976a1700d7b0adc7367b0b5b4352f44`;
- `src/acp/policy.ts` — blob `5cbf002ee4ad1c9e2529417ec4b3d054a73e69fb`;
- `src/acp/server.ts` — blob `4d63e0c08b9a18eac1825b3ead16cc669e84c8ff`;
- `packages/acp-core/src/index.ts` — blob `91e9a6b6c010ded1356e28e9740a0b876705ea25`.

Fabushi currently has useful adjacent owners: Gateway authentication and
dispatch, session routing/storage, Runner auto-review and settlement, and the
plugin-SDK `AcpSessionManager`. They are not yet one ACP production chain.
The plugin-SDK manager is an in-memory/test-consumed helper and must not become
a second session store, ACP transport, or doctor authority.

## 2. Goal

Provide ACP compatibility through the existing Fabushi Gateway, Session, and
Runner owners with one traceable production path:

`ACP transport input -> existing Gateway/auth admission -> canonical session
route/state -> existing command projection and policy owner -> existing Runner
turn/settlement consumer -> ordered ACP response/output -> owned shutdown`.

## 3. Non-goals

- Do not copy the OpenClaw Gateway, session database, or plugin-SDK ACP store.
- Do not expose policy inputs, allowed models, or security context from
  untrusted ACP request JSON.
- Do not count `AcpSessionManager`, contract fixtures, `SandAutoReviewDecision`,
  or transcript watchdog helpers as ACP wiring without a production caller.
- Do not rebaseline the fixed historical ACP command contract to the moving
  OpenClaw command registry during this task.
- Do not claim package/archive/native/license closure from TypeScript import
  presence or a package manifest alone.

## 4. Functional requirements

- `ACP-01`: The command projection exposes the fixed command names and input
  hints through the canonical ACP connection, while preserving the existing
  command registry/policy owner.
- `ACP-02`: ACP enablement, dispatch enablement, explicit-turn enablement, and
  normalized allowed-agent checks reject before any Runner effect.
- `ACP-03`: Every ACP session is bound to the canonical Fabushi session route,
  identity, lifecycle generation, and authorization context.
- `ACP-04`: Approved ACP turns use the existing Runner turn owner and existing
  settlement/watchdog lifecycle; no parallel turn queue or lease owner is
  introduced.
- `ACP-05`: Requests and responses are ordered, bounded, and fail closed on
  malformed frames, protocol mismatch, EOF, Gateway disconnect, cancellation,
  and shutdown failure.
- `ACP-06`: Shutdown retires ACP transport state before releasing the existing
  Gateway/session/database owners, and retries do not duplicate settlement or
  cleanup effects.

## 5. Current and target state

| Concern | Current state | Target state |
| --- | --- | --- |
| ACP command producer | No production `AvailableCommand` producer | Existing command-registry projection adapted at the ACP boundary |
| ACP policy producer | No production ACP dispatch-policy caller | Existing Gateway/auth/session policy context reaches Runner admission |
| Session state | Canonical Fabushi session owners exist; plugin ACP manager is separate | ACP session identity is a projection of the canonical session owner |
| Turn consumer | Runner settlement/watchdog exists independently | ACP-approved turn reaches the same Runner and settlement path |
| Transport | No Fabushi ACP production transport chain | One bounded, ordered ACP adapter owned by existing Gateway/Host lifecycle |
| Acceptance | Contract/static evidence only | Runtime, packaged, security-negative, and exact-head evidence on one H |

## 6. Ownership and architecture

- The ACP adapter owns protocol framing, ordering, and transport-local
  cancellation only.
- Existing Gateway/auth owners own connection admission, credential scope, and
  shutdown coordination.
- Existing Session routing/storage owners own session identity, lifecycle
  generation, and transcript/session state.
- Existing command-registry or command-presentation owner owns command
  projection; the adapter must not maintain a second registry.
- Existing Runner auto-review/policy owner owns admission decisions; existing
  Runner settlement/watchdog owns turn completion and lease release.
- Existing event/telemetry owners own redacted diagnostics and receipts.

## 7. Interfaces and data flow

The implementation must document concrete Rust types and call sites for:

1. bounded ACP frame decode and protocol-version negotiation;
2. authenticated connection/session admission;
3. canonical session-route resolution and lifecycle-generation capture;
4. command projection and policy evaluation;
5. Runner turn admission, cancellation, settlement receipt, and watchdog
   release;
6. ordered response/event emission and shutdown cleanup.

Each mutation must carry the captured session identity and generation to the
canonical owner. A stale generation, missing authorization context, unknown
command, disabled policy, or ambiguous route is a rejected request, not a
best-effort fallback.

## 8. Failure modes and security requirements

The design must cover malformed/oversized frames, unsupported protocol
versions, missing or invalid credentials, disallowed agents, disabled ACP or
dispatch, unknown commands, stale session generations, duplicate request IDs,
Gateway disconnect, provider failure, cancellation, EOF, partial output, and
failed shutdown. Error output must not expose tokens, passwords, transcript
contents, or unredacted provider credentials.

## 9. Implementation strategy

1. Complete the fixed ACP transitive source/package closure and record exact
   blobs, package identities, generated/native inputs, and license/notice
   limits.
2. Select the existing Fabushi transport/auth/session/Runner owners and record
   the call graph before writing runtime code.
3. Implement the smallest adapter in the selected owner boundary, reusing
   canonical state and lifecycle APIs.
4. Add mutation-sensitive positive and security-negative contract oracles for
   `ACP-01` through `ACP-06`.
5. Perform a source/spec compliance review, then defer exact-head Actions,
   packaged, and external-client acceptance until the development freeze is
   lifted.

## 10. Verification and acceptance

- Static review must prove every producer → owner → state → consumer edge.
- Contract oracles must prove policy rejection, route/generation fencing,
  command projection, ordered responses, duplicate suppression, cancellation,
  and shutdown ownership.
- A packaged/runtime ACP client must exercise initialize, session creation,
  command projection, approved turn, rejected turn, disconnect, and restart.
- Exact-head Actions evidence and independent acceptance are required before
  either parity row becomes verified or accepted.
- During the current development freeze, no OCRM CI/build/test is run and no
  ledger verification/acceptance flag is promoted.

## 11. Definition of Done

- `ACP-AC01`: fixed source/package closure is complete and exact identities are
  recorded;
- `ACP-AC02`: one canonical production transport/auth/session/Runner call chain
  is reachable;
- `ACP-AC03`: positive and negative lifecycle/security oracles pass on the same
  source revision;
- `ACP-AC04`: packaged/runtime client evidence proves the ACP behavior;
- `ACP-AC05`: no helper, fixture, static source presence, or independent Host
  lease is counted as ACP acceptance.

## 12. References

- `AGENTS.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/parity-ledger.json`
- `projects/openclaw-rust/dossiers/agent-turn-execution-scoped.md`
- `projects/openclaw-rust/decisions/2026-10-31-session-model-selection-ingress.md`
- Fixed upstream commit and tree listed above
