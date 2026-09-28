# Incident response

Use this process for production/release defects, security-sensitive operational failures, data/state corruption, widespread crashes, update failures, or other incidents requiring coordinated containment and evidence.

## 1. Detect and classify

Record:

- affected version/SHA;
- platforms/population known to be affected;
- first observed time;
- symptoms and user impact;
- whether security/privacy/data integrity may be involved.

Do not speculate beyond evidence.

## 2. Contain

Choose the safest reversible containment available, such as stopping a publication, disabling a risky rollout path, pausing an updater path, or reverting to a known-good source/artifact when the governing system supports it.

Security incidents follow `SECURITY.md` and must not expose sensitive details publicly.

## 3. Preserve evidence

Capture exact source/release IDs, workflow runs, artifacts, logs, traces, crash reports, screenshots/video, and reproduction steps before destructive cleanup where practical.

## 4. Diagnose

Build a timeline and distinguish root cause from contributing factors. Link findings to affected architecture/Spec requirements.

## 5. Repair

Create/update a Spec for the fix. Architecture-affecting repairs require ADR/architecture/migration updates under `AGENTS.md`.

## 6. Verify

Use the verification level appropriate to the incident, including exact-head CI, failure/recovery regression tests, packaged acceptance, and published-artifact checks where applicable.

## 7. Recover

Restore service/product delivery only when the required gates are satisfied. Record the exact recovered version/SHA/artifacts.

## 8. Close and prevent recurrence

Document root cause, corrective action, regression coverage, architecture/process changes, and remaining follow-ups. Do not mark an incident closed while required prevention/rollback work remains unowned.
