# Rust Coordinator carrier lifecycle — Specification

Status: active
Owner: Fabushi Desktop Electron Coordinator carrier owner
Last updated: 2026-11-01
Related project: `projects/openclaw-rust`
Related task / issue / PR: N/A

## 1. Context / problem

`source/electron-main/coordinator/rust-coordinator-carrier.ts` bridges
transferred MessagePorts to the Rust Coordinator process over line-framed
stdin/stdout. A synchronous write failure on either side of that bridge can
escape an event callback without entering the carrier's existing `close()`
owner, leaving ports/process state partially live.

## 2. Goal

Every synchronous carrier I/O failure becomes an explicit carrier failure and
enters the existing close path, which closes owned transferred ports and
terminates the owned Coordinator child.

## 3. Non-goals / out of scope

- No retry or replay of a frame after an ambiguous write.
- No new durable receipt or business-operation owner.
- No change to channel/frame schemas or Rust Coordinator protocol semantics.
- No claim that a remote side effect did not occur before an I/O failure.

## 4. Requirements

- C1: A synchronous child-stdin frame write failure is caught at the carrier
  boundary and invokes `close(1)`.
- C2: A child stdin `error` event invokes the same failure close path.
- C3: A synchronous transferred-port frame write failure is caught and invokes
  `close(1)`.
- C4: Close remains idempotent and only owns the carrier's transferred ports
  and Coordinator child.
- C5: Failure handling does not retry or replay the frame whose outcome is
  unknown.

## 5. Current state

The carrier owns `ports`, `child`, and an idempotent `close()` function, but
the port-message listener and stdout-line listener call their respective write
operations without local exception handling. Child process `error`/`exit`
events cover process lifecycle but not every synchronous stream write failure.

## 6. Target state

All carrier I/O edges route failures to the existing `close()` owner. The
carrier stops forwarding after `closed` becomes true and never replays a frame
whose write outcome is uncertain.

## 7. Architecture and ownership boundaries

The Electron utility-process carrier owns only transport bridging and the
Coordinator child/ports it spawned or received. The Rust Coordinator owns
protocol/business dispatch; Host owns business state; neither is replaced by a
new JS state store.

## 8. Interfaces / contracts / schemas / data flow

Existing flow: transferred port frame → JSON line on child stdin → Rust
Coordinator stdout envelope → selected transferred port frame. Error outcomes
are local carrier shutdown (`close(1)`); no wire-level retry is emitted.

## 9. Constraints and non-functional requirements

- Failure handling must be bounded and non-throwing from event callbacks.
- Close must remain idempotent under simultaneous port, stream, and child exit
  failures.
- Logs may identify the carrier edge and error, but must not include secrets.

## 10. Failure modes and edge cases

Cover stdin write throw, stdin error event, stdout-to-port post throw, child
exit racing an I/O failure, transferred-port close throw, and duplicate close
signals. The uncertain frame must not be sent again.

## 11. Implementation strategy

Add a small carrier-owned failure helper that logs the edge and invokes the
existing `close(1)`. Wrap both synchronous forwarding calls and subscribe to
the owned child's stdin error event. Preserve existing child error/exit and
port-close behavior.

## 12. Verification / test strategy

Static review must show all synchronous bridge writes and stdin error events
reach the existing close owner. Focused carrier tests should inject each
failure and assert ports/child cleanup without a second frame attempt. Under
the current development freeze, local tests/builds and OCRM CI remain not run.

## 13. Acceptance criteria / Definition of Done

- AC-1: C1–C5 are implemented in the existing carrier owner.
- AC-2: No carrier I/O failure escapes an event callback as an unhandled throw.
- AC-3: `git diff --check` and source review pass; runtime, test, packaged,
  and exact-head acceptance remain pending.

## 14. Release / migration / rollback

No migration. Rollback is reverting the carrier/spec commits.

## 15. Observability / evidence

Carrier-edge failure logs and source-level diff are required. No runtime or
packaged evidence is claimed during the development freeze.

## 16. References / provenance

- `AGENTS.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `source/electron-main/coordinator/rust-coordinator-carrier.ts`
- `source/electron-main/coordinator/coordinator-main-legs.ts`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| C1–C5 | passed | `rust-coordinator-carrier.ts` routes stdin/stdout forwarding throws and Coordinator stdin errors through the existing idempotent `close(1)` owner without retrying frames. |
| AC-1 | passed | The existing carrier remains the sole transport/process owner; no new state owner was introduced. |
| AC-2 | passed | Forwarding callbacks catch synchronous I/O errors and the stdin error event has an explicit failure handler. |
| AC-3 | passed | `git diff --check` and source review passed; runtime, focused test, packaged, and exact-head acceptance remain pending. |
