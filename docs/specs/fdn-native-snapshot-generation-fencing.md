# FDN Native Snapshot Generation and Stale Fencing — Specification

Status: active
Owner: Fabushi Desktop Mahayana Computer executor + Host Remote Computer target
Last updated: 2026-10-15
Related project: FDN-001
Related task / issue / PR: follow-up to Desktop PR #82; G10 / FDN-R12 / FDN-A07

## 1. Context / problem

The current Computer executor serializes desktop I/O and captures a final observation after each action batch, but observations have no stable generation that a remote caller can bind to. A remote action can therefore be prepared from an older screenshot without the target having a first-class way to reject that stale observation before side effects.

## 2. Goal

Give every successful Computer observation a monotonic in-process generation and provide an atomic stale fence inside the existing Mahayana Computer execution lock. The Remote Computer target must expose that generation with observations and reject a caller-supplied stale generation before performing any side effect.

## 3. Non-goals / out of scope

This slice does not implement window control, accessibility tree/semantic actions, persistent generation across Host restart, browser tab generations, or mandatory generation for legacy callers. It does not create a second Computer executor or Remote Computer state store.

## 4. Requirements

- SGF-R1: successful Computer observations receive a monotonically increasing non-zero generation owned by `mahayana-computer`.
- SGF-R2: stale comparison and action execution occur while holding the existing desktop execution mutex; no check-then-act race is allowed.
- SGF-R3: a supplied expected generation that differs from the current generation fails before input, click, typing, scrolling, drag, or other action execution.
- SGF-R4: Remote Computer exposes `snapshotGeneration` in normal action results and authenticated sensitive screenshot metadata.
- SGF-R5: the Remote Computer schema accepts optional `snapshotGeneration`; nested `then` actions cannot override the primary fence.
- SGF-R6: compatibility is additive in this slice. Callers that omit the field retain existing behavior until protocol-version negotiation makes stale fencing mandatory. The compliance record must not claim FDN-A07 passed until that mandatory rollout is separately verified.
- SGF-R7: stale failures use explicit `snapshot_stale` semantics and never trigger automatic action replay.
- SGF-R8: all executable verification runs only in GitHub Actions.

## 5. Current state

At Desktop main `8df146269da324693183b37c259b7258deaba45c`, `mahayana-computer` owns the shared execution mutex/control lease/final screenshot. `source/host/src/remote_computer_target.rs` maps Official MCP Computer actions into that executor. Results expose capture time/size but no observation generation.

## 6. Target state

The Computer owner maintains one atomic observation generation. Capture-only and act-then-observe paths allocate a generation only after a successful capture. The fenced execution entry point compares the caller's expected generation under the same I/O mutex immediately before actions. The exact generation associated with the final observation is returned to Host and projected to Remote Computer result metadata.

## 7. Architecture and ownership boundaries

Generation state remains in `mahayana-computer`; Host only transports the caller expectation and returned generation. Core remains operation/auth/result authority and stores only ordinary result JSON or sensitive-result metadata, never screenshot plaintext. Existing lease, authorization, network-generation, request-digest, and sensitive-result fences remain unchanged.

## 8. Interfaces / contracts / schemas / data flow

Remote Computer `Computer` input adds optional unsigned `snapshotGeneration`. It is valid only on the primary action. Output adds required `snapshotGeneration` for new exact-head results. Sensitive screenshot ready metadata carries the same field. Stale error payload contains `code=snapshot_stale`, expected and current generation.

Internal executor adds an observed/fenced execution API that returns `(ComputerActionResult, generation)` while preserving existing unfenced entry points for compatibility.

## 9. Constraints and non-functional requirements

No plaintext screenshot is added to Core. Generation is process-local, monotonic until integer exhaustion, and resets on Host restart; restart fencing continues to rely on existing session/network/grant generations. The counter must not advance on failed capture. Existing desktop mutex remains the sole interaction arbitration primitive for this change.

## 10. Failure modes and edge cases

Generation 0 means no successful observation exists yet. Overflow fails closed rather than wrapping. A stale expected generation is rejected before side effects. Capture failure leaves the previous generation current. Nested action generation is rejected. Missing generation is accepted only for compatibility and must remain visibly pending in FDN compliance.

## 11. Implementation strategy

First add the generation owner and refactor the existing capture/execute paths so generation allocation happens under the existing execution lock. Then add a fenced remote execution entry point. Extend Host argument parsing/result projection/sensitive-result metadata and focused contracts. Do not weaken existing authorization or screenshot sensitivity rules.

## 12. Verification / test strategy

GitHub Actions Rust desktop runtime must compile macOS/Windows/Linux pressure jobs and rust-host. Focused tests verify monotonic successful observations, stale rejection before action execution using non-side-effect actions where possible, nested generation rejection, schema/result projection, sensitive metadata propagation, and explicit error mapping. No local build/test is allowed.

## 13. Acceptance criteria / Definition of Done

- SGF-AC1: exact-head Actions pass for all Rust desktop runtime jobs.
- SGF-AC2: focused contracts prove an expected stale generation is rejected atomically before execution.
- SGF-AC3: normal and sensitive Remote Computer observations expose the exact generation returned by the shared executor.
- SGF-AC4: existing callers without the new field still parse/execute; mandatory enforcement remains pending until version negotiation and production caller rollout are evidenced.

## 14. Release / migration / rollback

No storage migration. Rollback removes additive generation metadata/fencing API and returns to prior compatibility behavior. This PR must remain behind the existing Remote Computer authorization/feature enablement; it does not independently open a new capability.

## 15. Observability / evidence

Record Desktop exact SHA, PR, Rust desktop runtime run/job IDs, focused test names, and any failure log. Stale errors may log only generations and operation identifiers, not screenshots or input payload contents.

## 16. References / provenance

- `docs/specs/fabushi-same-account-device-network.md` FDN-R12, G10, FDN-A07.
- Desktop main `8df146269da324693183b37c259b7258deaba45c`.
- `source/mahayana/mahayana-rs/mahayana-computer/src/lib.rs`.
- `source/host/src/remote_computer_target.rs`.
- Explicit user requirement in the 2026-10-15 FDN execution handoff: snapshot generation/stale fencing and act-then-observe must be completed in the existing Host/Computer owner.

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| SGF-R1–R8 | blocked | implementation / exact-head Actions pending |
| SGF-AC1 | blocked | exact-head Actions pending |
| SGF-AC2 | blocked | focused stale-fence evidence pending |
| SGF-AC3 | blocked | result projection evidence pending |
| SGF-AC4 | blocked | compatibility contract pending |
