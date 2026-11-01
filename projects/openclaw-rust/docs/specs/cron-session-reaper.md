# Cron session reaper

Status: implementation in progress; bounded registration slice approved
Project: OCRM-001
Responsibility: OCRM-RESP-CRON-SESSION-REAPER-001

## Context

OpenClaw's cron reaper removes expired per-run cron sessions and archives their
unreferenced transcripts. It preserves reusable base sessions, skips active
controller work and pending generated-media/subagent work, and uses the
existing guarded lifecycle owner when a session row has a session identity,
lifecycle revision, and update timestamp.

The current Fabushi Rust automation path records automation runs in the
agent-owned automation store but does not currently materialize or index
per-run cron session rows. `source/host/src/extensions/transcript/mod.rs` also
does not contain the previously recorded `TranscriptCleanup` owner. The old
ledger target and O05 oracle therefore do not establish an implementation
path.

The first implementation slice is now approved for D01--D03 only: scheduled
and event-triggered runs register and settle a durable per-run session row in
the existing agent database through `ProductionSessionWorkers`. This makes the
producer-to-owner-to-state edge explicit without claiming reaper deletion,
transcript archival, media/subagent preservation, or acceptance. D04--D05
remain an architecture and verification gate for the later cleanup slice.

## Goal

Define and implement one existing-owner Rust path for cron run-session
retention only after the per-run session representation, lifecycle authority,
transcript archive semantics, and scheduler/reaper trigger are explicitly
selected and approved.

## Non-goals

- Do not delete automation definitions or historical automation run records.
- Do not delete a session based only on filesystem age, automation-run age, or
  a stale projection.
- Do not add a second session database, cleanup owner, or unauthenticated
  deletion path.
- Do not treat the existing subagent cleanup wiring as cron reaper evidence.
- Do not claim production wiring, verification, or acceptance from this spec.

## Required design decisions

- CRON-REAPER-D01: identify the durable per-run session row/key and its owner;
  if Rust has no equivalent, specify the minimal migration before cleanup.
- CRON-REAPER-D02: define retention parsing, including `false`, invalid values,
  zero duration, default duration, and bounded sweep throttling.
- CRON-REAPER-D03: define the producer trigger and ownership fence for a sweep,
  including agent availability and current Gateway/lifecycle authority.
- CRON-REAPER-D04: define active media/subagent preservation and exact
  transcript artifact cleanup ownership.
- CRON-REAPER-D05: define guarded deletion fields and the changed/failed
  outcomes without retrying unknown deletion outcomes.

The bounded registration slice implements only the durable identity and
terminal-state portions of D01 and the producer/owner handoff in D03. It does
not select a cleanup trigger, perform retention parsing, archive transcript
artifacts, or delete rows. A read-only owner projection of these rows is part
of the same D01 slice so a future sweep cannot read `runs.json` as a substitute
for session state.

The next approved slice is the pure D02 policy seam: retention parsing and a
per-owner five-minute throttle primitive. It has no scheduler caller and does
not decide whether a row is deletable; D03 trigger wiring and D04--D05 cleanup
remain pending.

The bounded D03 bridge may expose a read-only candidate-sweep request from the
existing `AutomationRuntime`. It must parse policy, apply the owner-scope
throttle, and delegate row selection to `ProductionSessionWorkers`; it must not
delete, archive, or create a second scheduler/cleanup owner.

The following bounded read slice is also approved: the existing Session owner
may project expired, terminal, non-base rows while excluding active
controller/media/subagent fences. This is candidate selection only; it must not
archive transcript artifacts or delete a row until D04--D05 are approved.
Fence columns are fail-closed: until a real producer publishes a complete
fence snapshot for the row, `fence_state_known` is false and the row is not a
cleanup candidate.
Schema initialization must migrate an already-created registration table by
adding this marker with a default of false; migration errors remain explicit
and non-passing.

The next D04 slice may publish a typed fence snapshot at the existing Host
automation terminal boundary. It may read only the current admitted media
state, runner stream registry, and generated-subagent runtime through their
existing owners; any unavailable or mismatched fact keeps the row unknown.
Snapshot publication is not transcript archival or deletion authorization.

## Verification and Definition of Done

- A source-backed architecture decision names the existing owners and all
  producer → owner → durable row → consumer edges.
- Focused oracles cover disabled/zero retention, invalid fallback, expired
  deletion, base-session preservation, active media/subagent preservation,
  lifecycle change during sweep, and transcript cleanup failure.
- The exact upstream source identity and current Rust source identity are
  recorded; no old O05 auto-review test is reused as reaper evidence.
- Exact-head Actions, runtime, package, provider, and device evidence remain
  required before the responsibility is considered accepted.
- The bounded registration slice is complete only when the run path calls the
  owner before execution and settles the same guarded row after execution;
  verification remains pending during the development freeze.
- The D02 policy slice is complete only when disabled, zero, invalid, finite,
  and throttle outcomes are represented without reading filesystem or run age;
  verification remains pending during the development freeze.

## References

- `projects/openclaw-rust/dossiers/cron-automations-heartbeat-scoped.md`
- `projects/openclaw-rust/dossiers/upstream-current-session-lifecycle-delta-20261031.md`
- OpenClaw `src/cron/session-reaper.ts` at the current discovery subject
- `source/host/src/extensions/transcript/automation_run_path.rs`
- `source/host/src/extensions/transcript/automation_runtime.rs`
- `source/host/src/extensions/session/production.rs`
- `projects/openclaw-rust/decisions/2026-11-01-cron-session-reaper-owner-boundary.md`
