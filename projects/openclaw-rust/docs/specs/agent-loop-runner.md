# Agent loop runner and deferred steering — Specification

Status: active
Owner: OCRM Runner owner
Last updated: 2026-10-31
Related project: projects/openclaw-rust
Related task / issue / PR: OCRM-RESP-AGENT-LOOP-RUNNER-001

## 1. Context / problem

The parity row currently combines two behaviors: exclusive active-turn
ownership (`AlreadyRunning`) and deferred input that is injected at a safe
agent-loop checkpoint. Fabushi already owns the active lifecycle in
`TurnRunShell` and the production adapter reaches begin/dispatch/settlement,
but the only deferred-turn queue is a local test fixture. The production
worker constructs one `ProductionTurnAgentOwner` per worker request, so that
fixture cannot be treated as a production queue or session owner.

The fixed OpenClaw subject is
`openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`. Its relevant
production source is `packages/agent-core/src/agent-loop.ts` together with
`packages/agent-core/src/agent-loop-steering.ts`; the source uses checkpoint
steering/follow-up injection and does not justify a separate Fabushi queue.

## 2. Goal

Preserve one canonical Runner/turn owner while providing equivalent production
behavior for active-run exclusion and safe deferred steering/follow-up
delivery. The implementation must be enabled only after a real production
request ingress and consumer are identified in the existing Host/Runner
boundary.

## 3. Non-goals / out of scope

- Do not create a second Agent loop, session store, transcript store, or
  scheduler.
- Do not promote `source/host/tests/openclaw_agent_turn_contract.rs` fixtures
  to production behavior.
- Do not reinterpret a new worker request as an in-turn steering message
  without a real ingress contract.
- Do not claim verification, wiring, or acceptance from static symbols or
  contract tests.

## 4. Requirements

- RLR-01: The canonical Runner owner rejects a second active run with
  `TurnRunShellError::AlreadyRunning`.
- RLR-02: Deferred input is owned by the existing agent-loop/Runner boundary,
  is bounded, and is injected only at a documented checkpoint or final
  follow-up boundary.
- RLR-03: A deferred item that is cancelled, skipped, or superseded cannot
  enter the transcript or execute a tool; accepted items retain ordering and
  identity.
- RLR-04: Production ingress, queue state, checkpoint consumer, and terminal
  settlement must form one traceable call chain. A test-only queue does not
  satisfy this requirement.
- RLR-05: Stale owners, duplicate settlement, cancellation, and worker
  recreation fail closed without leaking deferred state across workers.

## 5. Current state

- `TurnRunShell::begin_run` and the production lifecycle adapter implement
  active-run ownership and settlement.
- `ProductionTurnAgentOwner` is used by `SandAgentRunner`; the shipping host
  path creates the runner for a worker request and calls the owner directly.
- `DeferredTurnQueue` exists only inside the OCRM contract test and has no
  production producer or consumer.
- The fixed OpenClaw source provides agent-loop steering checkpoints and
  follow-up drains, not evidence for adding an independent Fabushi queue.

## 6. Target state

The existing Runner owner remains the sole owner of active turn state. A real
production ingress is identified and connected to the same owner; its
deferred steering/follow-up state is bounded, checkpoint-consumed, ordered,
cancel-safe, and observable. Until that ingress and consumer are proven, the
responsibility remains `implementation=not-started`, `wiring=not-wired`,
`verification=not-run`, and `acceptance=pending`.

## 7. Architecture and ownership boundaries

- `TurnRunShell` owns active owner tokens, cancellation, and settlement.
- The existing Host/Runner ingress owns request classification and must be the
  only producer of deferred turn input.
- The existing generated agent loop owns checkpoint injection and tool
  execution ordering; no parallel loop may be introduced.
- Session/transcript persistence and Gateway transport remain with their
  existing owners.

## 8. Interfaces / contracts / schemas / data flow

Required flow: `production ingress -> canonical Runner owner -> bounded
deferred steering state -> agent-loop checkpoint -> existing tool/provider
execution -> one terminal settlement`.

The exact request identity, cancellation token, ordering rule, capacity, and
recovery behavior must be specified when the production ingress is located.
No new durable schema is authorized by this spec.

## 9. Constraints and non-functional requirements

- Preserve request/session identity and cancellation authority.
- Bound memory and prevent deferred work from surviving owner disposal.
- Preserve existing tool ordering and avoid prompt interleaving.
- Keep all completion flags false until exact frozen verification and
  production evidence exist.

## 10. Failure modes and edge cases

Handle active-run conflict, queue capacity, cancellation while drained,
worker recreation, stale owner tokens, duplicate settlement, checkpoint
failure, final follow-up arriving during an awaited drain, and tool execution
already started before steering becomes available. Unknown ingress semantics
are an evidence gap, not permission to guess.

## 11. Implementation strategy

1. Identify and document the real production request ingress and its owner.
2. Reconcile that ingress with the existing Runner/agent-loop checkpoint
   contract and this spec.
3. Implement only inside the existing owner boundary, if the call chain is
   proven; otherwise retain the explicit not-started state and record the
   blocker.
4. Add focused tests only after the production contract exists; do not use a
   fixture-only queue as evidence.

## 12. Verification / test strategy

- Static review must prove the source identity, owner boundary, and complete
  producer/consumer call chain.
- Contract tests must cover RLR-01 through RLR-05, including cancellation and
  stale-owner negatives.
- Frozen OpenClaw oracle and exact Fabushi source revision must be verified in
  GitHub Actions under the repository's normal gates.
- Local OCRM build/test/formatter/lint and OCRM CI are intentionally not run
  during the current development freeze.

## 13. Acceptance criteria / Definition of Done

- AC-RLR-01: One production owner rejects concurrent active runs.
- AC-RLR-02: A real production deferred-input producer and checkpoint
  consumer are linked without a second Agent loop or state owner.
- AC-RLR-03: Ordering, cancellation, capacity, stale-owner, and settlement
  behavior have direct frozen-oracle evidence.
- AC-RLR-04: Required Actions evidence is bound to the exact accepted source
  and target revision; no fixture or static symbol is counted as acceptance.

## 14. Release / migration / rollback

N/A until a production ingress is identified. Any later implementation must
ship behind the existing Runner rollout boundary and be removable without
changing session or transcript persistence.

## 15. Observability / evidence

Retain exact source blob identities, production call-chain references,
capacity/cancellation outcomes, Actions run/job/artifact IDs, and the parity
ledger state transition. Evidence must distinguish `implemented`, `wired`,
`verified`, and `accepted`.

## 16. References / provenance

- Root instructions: `AGENTS.md`
- Project authority: `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- Parity row: `projects/openclaw-rust/parity-ledger.json`
- Existing evidence: `projects/openclaw-rust/dossiers/agent-turn-execution-scoped.md`
- Fixed upstream source:
  `packages/agent-core/src/agent-loop.ts` blob
  `c6d5ce6c43fe0897f47ee71f2e74ac4e93034634`
- Fixed upstream steering source:
  `packages/agent-core/src/agent-loop-steering.ts` blob
  `8c6f1bade7526eb67e6e7813c34542cf0b454ba9`
- Existing Rust owner:
  `source/host/src/runner/turn_run_shell.rs`
- Existing production adapter:
  `source/host/src/runner/production_turn_run_shell_adapter.rs`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| RLR-01 | passed | Existing `TurnRunShell::begin_run` and O01 contract; production active lifecycle path identified. |
| RLR-02 | blocked | No production deferred-input producer/consumer has been identified; test queue is not evidence. |
| RLR-03 | blocked | No production deferred state contract exists to verify. |
| RLR-04 | blocked | Required production call chain is absent. |
| RLR-05 | pending | Must be verified with the real ingress once identified. |
| AC-RLR-01 | pending | Existing direct production acceptance remains pending under frozen Actions gate. |
| AC-RLR-02 | blocked | Missing production deferred-input ingress and checkpoint consumer. |
| AC-RLR-03 | blocked | No production deferred-input contract to exercise. |
| AC-RLR-04 | pending | Frozen verification and acceptance have not run. |
