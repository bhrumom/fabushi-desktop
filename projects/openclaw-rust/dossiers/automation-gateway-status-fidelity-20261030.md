# Automation Gateway status fidelity — 2026-10-30

## Finding

The Host `cron.list` projection previously changed an event-triggered
automation with no cron expression into a synthetic `0 * * * *` schedule and
reported `lastRunStatus: "ok"` even when no run existed. `cron.trigger` also
returned `finished: true` and `status: "ok"` without observing a terminal run.

## Owner-preserving repair

`source/host/app/src/main.rs` now projects an empty schedule as an explicit
event schedule, derives update/status/error fields from the existing
`AutomationRecord` and its bounded run history, and returns `accepted` rather
than a fabricated terminal result when a post-trigger terminal run is not
observable. It also omits `runId` until a newly observed canonical run exists;
there is no UUID fallback for an unobserved trigger.

No second automation store or scheduler was added. The existing automation
runtime remains the producer and lifecycle owner; this change only repairs the
Gateway consumer projection.

## Evidence limits

Static source review and `git diff --check` are the only development evidence.
Local OCRM tests/builds, Actions receipts, and live scheduler acceptance remain
pending under the project freeze.
