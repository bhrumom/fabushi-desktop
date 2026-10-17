# Signed-in Onboarding Request Lifecycle

Status: implementation in progress; acceptance requires exact-HEAD GitHub Actions evidence.
Owner: `frontend/src/recovered/features/onboarding/signed-in/`.

## Authority and ownership

- FBCP-001 Revision 7: `docs/specs/fabushi-bot-communication-platform.md` and `projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md`.
- FBCP-QA-08 requires applicable timeout, cancel, restart, and stale-data paths; FBCP-QA-14 binds evidence to exact source.
- FDN-001 FDN-A07 requires account-switch and old asynchronous results to be fenced. This task applies that lifecycle rule without adding another account, computer, coordinator, or readiness owner.
- Existing owners remain the signed-in onboarding model/view, shared ComputerStatusStore projection, and ProductionCoordinatorClient.

## Requirements

- ONB-LIFE-01: every submit has a monotonic generation; account slot, client, readiness instance/status, bridge/probe/callbacks, and component lifecycle must match after every await. A→B→A cannot revive the original A request.
- ONB-LIFE-02: readiness returns ready/timeout/cancelled. The existing 60-second timeout remains fail-open; disposal cancels.
- ONB-LIFE-03: hand-off dwell returns completed/cancelled. The 1500 ms timer is not equivalent to cancellation; only a current request calls onComplete.
- ONB-LIFE-04: stale createAgent replies cannot be surfaced or cached; existing canonical account/client ownership remains authoritative.
- ONB-LIFE-05: deterministic tests cover readiness success/timeout/dispose, dwell completion/cancel/dispose, stale generation, and A→B→A.

## Acceptance

Run `node --test ../scripts/onboarding-lifecycle.test.mjs` from `desktop/`. The PR exact HEAD must pass renderer typecheck, renderer build, focused lifecycle tests, and relevant Electron E2E jobs. Skipped, cancelled, blocked, or non-terminal jobs are not passing evidence. All executable verification runs only in GitHub Actions.
