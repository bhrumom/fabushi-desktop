# Cron session reaper

Status: architecture/specification required before implementation
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

## References

- `projects/openclaw-rust/dossiers/cron-automations-heartbeat-scoped.md`
- `projects/openclaw-rust/dossiers/upstream-current-session-lifecycle-delta-20261031.md`
- OpenClaw `src/cron/session-reaper.ts` at the current discovery subject
- `source/host/src/extensions/transcript/automation_run_path.rs`
- `source/host/src/extensions/transcript/automation_runtime.rs`
- `source/host/src/extensions/session/production.rs`
- `projects/openclaw-rust/decisions/2026-11-01-cron-session-reaper-owner-boundary.md`
