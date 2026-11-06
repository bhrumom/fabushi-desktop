# Coordinator control-server settlement — Specification

Status: active
Owner: Fabushi Desktop Electron Coordinator control owner
Last updated: 2026-11-01
Related project: `projects/openclaw-rust`
Related task / issue / PR: N/A

## 1. Context / problem

`coordinator-control-server.ts` receives Coordinator control frames and posts
ready/reply/shutdown frames through one owned control-port callback. Some post
failures currently escape a synchronous handler or an async executor callback,
while undeclared event families only report a problem and leave the control
server live.

## 2. Goal

Every control transport/protocol failure enters the existing breach/settlement
owner exactly once and never escapes as an unhandled callback exception or
rejection.

## 3. Non-goals / out of scope

- No retry or replay of a ready/reply frame with unknown delivery outcome.
- No new control-port, process, or durable result owner.
- No change to executor business semantics or frame schema.

## 4. Requirements

- CCTRL-1: Ready-frame post failure enters protocol-breach settlement.
- CCTRL-2: Asynchronous executor-reply post failure enters the same breach
  settlement and does not create an unhandled rejection.
- CCTRL-3: Undeclared control event families are protocol breaches, not merely
  diagnostics followed by continued service.
- CCTRL-4: Existing breach shutdown-post failure remains contained and still
  settles the server.
- CCTRL-5: Valid executor replies, valid events, clean shutdown, and disposal
  retain existing behavior.
- CCTRL-6: An allowed event family with no consumer, or a consumer that
  throws, enters breach settlement rather than escaping the message callback.

## 5. Current state

The control server owns `settlement` and `settled`, and `breach()` already
contains its shutdown post. Ready and async reply posts do not use that owner;
unknown event families call `onProblem` and return.

## 6. Target state

The control server has one terminal settlement path for malformed/protocol and
transport failures. A frame whose delivery cannot be established is not
retried or reported as successfully delivered.

## 7. Architecture and ownership boundaries

The control server owns the control-port protocol state and terminal settlement.
`coordinator-launcher` owns the child process lifetime; executors own command
business behavior. No second pending or process state store is introduced.

## 8. Interfaces / contracts / schemas / data flow

Existing flow is Coordinator frame → `handleMessage`/`handleFrame` → executor
or control-port frame post. Failure uses the existing `breach(detail)` and
`settled` promise; no new wire frame is required.

## 9. Constraints and non-functional requirements

- No callback may leak a synchronous post exception.
- Async executor completion must always settle or explicitly breach.
- A failed post must not cause frame replay.

## 10. Failure modes and edge cases

Cover ready post throw, reply post throw after executor completion, unknown
event family, shutdown-post throw during breach, clean shutdown, disposal,
missing/throwing event consumer, and executor rejection converted to a valid
failed reply.

## 11. Implementation strategy

Wrap ready and async reply posts in the existing breach path. Replace the
undeclared-event diagnostic-only branch with breach. Keep valid replies/events
and existing shutdown-post containment unchanged. Resolve the event consumer
before invoking it and catch consumer failures into breach.

## 12. Verification / test strategy

Static review must show every control-port post failure reaches `breach` or
existing disposal handling. Focused contracts should inject ready/reply post
throws and unknown event families, asserting one terminal settlement and no
unhandled callback failure. Local tests/builds and OCRM CI remain not run
during the development freeze.

## 13. Acceptance criteria / Definition of Done

- AC-1: CCTRL-1 through CCTRL-5 are implemented in the existing owner.
- AC-2: No covered control-server post/protocol failure escapes its callback.
- AC-3: `git diff --check` and source review pass; runtime, tests, packaged,
  and exact-head acceptance remain pending.

## 14. Release / migration / rollback

No migration. Rollback is reverting the control-server/spec commits.

## 15. Observability / evidence

`onProblem`, optional `reportFailure`, source diff, and focused contract
results are the evidence surfaces. No runtime acceptance is claimed here.

## 16. References / provenance

- `AGENTS.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `source/electron-main/coordinator/coordinator-control-server.ts`
- `source/electron-main/coordinator/coordinator-launcher.ts`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| CCTRL-1–CCTRL-6 | passed | Ready/reply post failures, event-family validation, and missing/throwing event consumers now enter the existing breach settlement. |
| AC-1 | passed | The existing control server remains the sole protocol settlement owner. |
| AC-2 | passed | Ready and async reply posts are caught, and unknown event families use `breach`. |
| AC-3 | passed | `git diff --check` and source review passed; runtime, tests, packaged, and exact-head acceptance remain pending. |
