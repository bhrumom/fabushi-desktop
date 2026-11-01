# Plugin SDK Session-Memory Adapter — Specification

Status: active
Owner: existing Session/Transcript owner with Plugin SDK read adapter
Last updated: 2026-11-01
Related project: `projects/openclaw-rust`
Related responsibility: `OCRM-RESP-PLUGIN-SDK-SESSION-MEMORY-001`
Related blocker: `BL-PLUGIN-SDK-SESSION-WIRING-001`

## 1. Context / problem

`source/host/src/extensions/plugin_sdk/session_memory.rs` implements the
bounded pagination, generation-aware cursor, reset-window projection, paired
tool-result filtering, and current-turn read-fence policy required by the
fixed Plugin SDK session-memory contract. It is currently pure/read-side code
with no production caller. The canonical transcript/session owner already
lives in `ProductionSessionWorkers -> SandAgentDb` and already provides
transcript snapshots, pages, windows, tails, session admission, and lifecycle
identity.

The missing boundary is a typed adapter that supplies those canonical facts to
the existing Plugin SDK policy without introducing a second session catalog,
transcript store, reset-generation store, or admission authority.

## 2. Goal

Create one production read adapter that consumes a canonical session snapshot
from the existing Session/Transcript owner and delegates visibility, cursor,
pairing, byte/message budgets, and current-turn fencing to
`plugin_sdk::session_memory`. The adapter must expose one traceable
producer → existing Session owner → Plugin SDK projection → existing Gateway
or plugin consumer path.

## 3. Non-goals / out of scope

- Do not create a second session catalog, transcript database, cursor store,
  reset-generation store, or memory-session persistence owner.
- Do not let `session_memory.rs` open SQLite, append/rewrite transcript rows,
  infer reset boundaries, or create admission records.
- Do not make a generic unauthenticated Gateway route or trust caller-supplied
  generation, reset boundary, session identity, or current-turn fence.
- Do not wire incognito sessions to persistent transcript synchronization.
- Do not claim upstream source closure, verification, production acceptance,
  provider/device/package acceptance, or ledger completion from this adapter.

## 4. Requirements

- R1 — Canonical identity: the adapter accepts a validated agent/session
  identity from the existing Session owner and rejects empty, mismatched, or
  successor identities before projection.
- R2 — Owner snapshot: transcript entries, generation, latest reset boundary,
  reset-tail position, and optional current-turn read fence are captured from
  one canonical owner read boundary; the adapter cannot manufacture any of
  them from request JSON.
- R3 — Projection delegation: pagination, opaque cursor validation, reset
  invalidation, paired tool-result filtering, message/byte budgets, and fence
  clipping delegate to `read_visible_transcript_page`.
- R4 — Scope policy: the existing Plugin SDK sync policy rejects incognito
  visibility before persistent transcript publication or disclosure.
- R5 — No mutation: the read path performs no transcript, session, cursor, or
  reset-state mutation and does not acknowledge an external delivery.
- R6 — Stale safety: a generation, session, reset-boundary, or current-turn
  mismatch returns an explicit reset/stale/error result and never discloses a
  successor session's entries.
- R7 — Consumer ownership: the first production consumer is an existing
  authenticated Gateway/plugin adapter selected by the owning Host contract;
  presentation code remains a consumer and does not become a new state owner.

## 5. Current state

- `plugin_sdk::session_memory` exposes
  `read_visible_transcript_page`, `SessionVisibility`, and
  `VisibleTranscriptReadFence` as pure policy/projection APIs.
- `ProductionSessionWorkers` exposes canonical transcript page/window/tail
  reads and session-work admission reads, but no typed Plugin SDK adapter,
  reset-anchor producer, reset-tail projection position, or current-turn
  message-position fence.
- Existing Gateway methods consume the Session owner directly; no production
  caller consumes the Plugin SDK session-memory functions.
- The dossier records implementation as partial/implemented, wiring as
  `not-wired`, verification as `not-run`, and acceptance as `pending`.

The 2026-11-01 static audit found that `lifecycle_revision` is a usable
admission generation for stale-write fencing, but it is not by itself a
transcript reset anchor or a message-position fence. The existing owner APIs
therefore do not yet satisfy R2. Adding an adapter that invents those values,
maps them from request JSON, or treats the old Gateway transcript methods as
the SDK consumer would violate this specification and remains out of scope
until the owner contract is published.

The 2026-11-01 authenticated-consumer sweep confirms the same boundary against
the current checkout: `source/host/src/extensions/session/gateway.rs` reaches
`ProductionSessionWorkers::{read_agent_transcript_entries,
read_agent_transcript_page, read_agent_transcript_window,
read_agent_transcript_tail}`, while no production source calls
`plugin_sdk::session_memory::{paginate_sessions,
read_visible_transcript_page, SessionTranscriptLockRegistry}`. The only
callers of the Plugin SDK projection are the focused contract tests in
`source/host/tests/openclaw_plugin_sdk_core_contract.rs`. Existing Gateway
authentication therefore does not establish AC1, and the direct Gateway
transcript methods cannot be relabeled as the required adapter consumer.

## 6. Target state

An authenticated existing consumer requests a bounded session-memory read.
The adapter resolves the canonical session owner, captures the owner-provided
snapshot and identity/fence facts, enforces visibility policy, delegates the
projection, and returns a typed page/reset result. No adapter-owned durable
state is introduced.

## 7. Architecture and ownership boundaries

- `ProductionSessionWorkers` / `SandAgentDb`: sole durable transcript,
  session identity, lifecycle/generation, reset-boundary, and admission owner.
- A new thin Session-side adapter: composes the existing owner facts with the
  pure Plugin SDK projection; owns no durable state.
- `plugin_sdk::session_memory`: owns pure cursor, visibility, pairing, budget,
  and fence projection semantics.
- Existing authenticated Gateway/plugin transport: owns caller authority and
  selects the consumer contract; it must not persist transcript state.
- Electron/renderer/UI: consumes the typed page and reset result; it does not
  become a transcript or cursor owner.

The adapter may depend on the existing Session owner and Plugin SDK projection.
The Plugin SDK projection must not depend on a second storage implementation,
provider transport, or a new session registry.

## 8. Interfaces / contracts / data flow

The implementation must introduce a typed request/result boundary equivalent
to:

```text
SessionMemoryReadRequest {
  agent_id,
  session_id,
  visibility,
  start: transcript | reset_window,
  cursor: optional opaque cursor,
  max_messages,
  max_bytes,
  authenticated caller scope,
}

CanonicalSessionMemorySnapshot {
  agent_id,
  session_id,
  generation,
  entries,
  latest_reset,
  reset_tail_start,
  current_turn_read_fence: optional,
}
```

The caller may request projection parameters, but `generation`, reset facts,
entry contents, and fence identity must be owner-derived. The adapter calls
`read_visible_transcript_page` and returns its typed `Page` or `Reset` result
plus explicit owner/admission errors. Cursor bytes remain opaque and are not
stored by the adapter.

Required flow:

`authenticated existing consumer → Session owner snapshot/fence → Plugin SDK
projection → typed page/reset consumer`

## 9. Constraints and non-functional requirements

- Preserve transcript ordering and tool-call/tool-result pairing.
- Bound serialized work using the existing Plugin SDK message and byte caps.
- Never disclose a successor generation after a reset or owner replacement.
- Keep incognito reads explicitly non-persistent and policy-denied where sync
  would be implied.
- Do not log transcript contents, cursor contents, or credentials.
- Keep verification and acceptance pending during the repository development
  freeze.

## 10. Failure modes and edge cases

- Missing/empty agent or session identity: reject before projection.
- Caller identity differs from canonical owner identity: reject closed.
- Missing owner snapshot or admission: return an explicit unavailable result.
- Invalid, foreign, or generation-stale cursor: return the projection's reset
  reason without successor entries.
- Reset boundary moved or disappeared: return `SessionReset`, `AnchorMoved`,
  or `AnchorMissing` as applicable.
- Current-turn fence is stale or belongs to another scope: reject/clamp only
  according to the existing projection contract; never widen visibility.
- Orphan tool result or unsupported row: filter/reject according to the
  existing projection contract; never stringify it into an unrelated message.
- Incognito sync request: reject with the existing sync-denial policy.

## 11. Implementation strategy

1. Add the typed Session-side snapshot/request/result adapter contract without
   changing the durable Session owner.
2. Bind the adapter to one existing authenticated consumer selected by the
   Host/Gateway contract; pass only owner-derived identity, generation, reset,
   and fence facts.
3. Delegate projection policy to `plugin_sdk::session_memory` and preserve
   explicit reset/error results.
4. Add focused contract coverage for identity mismatch, cursor generation
   reset, reset-window pairing, fence clipping, byte/message limits, and
   incognito denial.
5. Re-read the exact changed source and run only the repository-authorized
   verification in a future unfrozen phase; do not promote ledger fields in
   this development freeze.

## 12. Verification / test strategy

- Static: prove one consumer, one Session owner, no adapter storage, and no
  direct SQLite access from Plugin SDK projection.
- Focused contracts: exercise the adapter with owner-shaped snapshots and
  negative identity/generation/fence cases.
- Exact-head Actions: required for verification; no local cargo/test/OCRM run
  is authorized in this phase.
- Runtime/package/device/provider acceptance: required separately and remains
  pending.

## 13. Acceptance criteria / Definition of Done

- AC1: A real authenticated existing consumer reaches the typed adapter.
- AC2: The adapter consumes one canonical Session owner snapshot and delegates
  all projection rules to `plugin_sdk::session_memory`.
- AC3: Cursor/reset/generation/fence mismatch cases fail closed with typed
  reset/error results.
- AC4: No second session/transcript/cursor/reset owner or incognito persistence
  path is introduced.
- AC5: Focused contracts and required exact-head Actions evidence pass before
  wiring/verification/acceptance are reconsidered.

## 14. Release / migration / rollback

No durable schema migration is expected. Rollback is removal of the adapter
consumer binding; the existing Session/Gateway transcript paths remain
authoritative and unchanged.

## 15. Observability / evidence

Record adapter rejection reason, owner generation, and consumer outcome without
recording transcript contents, cursor bytes, or credentials. Preserve exact
source revision and focused verification run identity when the frozen
verification phase is authorized.

## 16. References / provenance

- `projects/openclaw-rust/dossiers/plugin-sdk-core-scoped.md`
- `projects/openclaw-rust/docs/specs/session-work-admission.md`
- `source/host/src/extensions/plugin_sdk/session_memory.rs`
- `source/host/src/extensions/session/production.rs`
- `source/host/src/extensions/session/agent_session.rs`
- `projects/openclaw-rust/parity-ledger.json`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1–R7 | pending | Adapter implementation and production consumer are not yet present. |
| AC1–AC5 | pending | Wiring, focused contracts, exact-head verification, and acceptance remain open. |
