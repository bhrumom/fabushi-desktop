# Host-settings hydration failure settlement — Specification

Status: active
Owner: Fabushi Desktop Electron host-settings owner
Last updated: 2026-11-01
Related project: `projects/openclaw-rust`
Related task / issue / PR: N/A

## 1. Context / problem

The Electron host-settings owner starts `BoxSettingsField.absorbFromBox()`
from `onTransportConnected()` as a detached Promise. Box read failures are
contained inside the shared field, but a synchronous local mirror write or
clear can still reject the Promise. The detached rejection currently has no
consumer, so a persistence failure has no attributable settlement path.

## 2. Goal

Every onboarding hydration attempt started by the Electron host-settings owner
must settle locally: successful repaint retains the existing persistence
telemetry, while mirror persistence failures are reported through the existing
Electron failure owner and never escape as an unhandled rejection.

## 3. Non-goals / out of scope

- No retry, rollback, or second settings store.
- No change to the shared `BoxSettingsField` read/write semantics.
- No change to account-departure cleanup or its fail-closed propagation.
- No change to the valid `client_persistence` writeback telemetry schema.
- No claim of runtime, packaged, CI, or OCRM acceptance during the
  development freeze.

## 4. Requirements

- H1: `onTransportConnected()` must attach a rejection consumer to the
  `absorbFromBox()` Promise it starts.
- H2: A hydration rejection must be routed to the existing Electron failure
  reporter with area `host-settings` and a stable onboarding hydration leg.
- H3: Successful `repainted` hydration must retain the existing
  `client_persistence` writeback telemetry exactly as before.
- H4: Read failures that the shared field already converts to no-op remain
  no-op and must not be misreported as local persistence failures.
- H5: The existing host-settings owner and production composition remain the
  only state and failure owners.

## 5. Current state

`createDesktopHostSettingsFields()` calls
`void onboardingSeen.absorbFromBox().then(...)` without a rejection handler.
Production construction supplies persistence telemetry but no hydration
failure callback. The root `ProductionServiceContext` already exposes
`reportFailure(area, leg, error)` and the production binding routes it to the
existing diagnostics owner.

## 6. Target state

The host-settings owner accepts an optional failure callback. Production binds
that callback to `reportFailure("host-settings", "onboarding-hydrate", error)`.
The detached operation uses a rejection handler; test and lightweight callers
that do not provide a callback still receive contained settlement without a
new owner.

## 7. Architecture and ownership boundaries

- `host-settings-fields.ts` owns the hydration Promise settlement boundary.
- `main-production-services.ts` binds the production failure reporter.
- `ProductionServiceContext.reportFailure` remains the diagnostics owner.
- `client-failure-telemetry` remains the success/writeback telemetry owner.
- No renderer, Rust, Coordinator, or alternate persistence owner is added.

## 8. Interfaces / contracts / schemas / data flow

The factory options gain one optional callback:

`reportHydrationFailure?: (error: unknown) => void`

Flow remains:

`transport-connected -> host-settings owner -> absorbFromBox -> mirror`

Only the rejection edge is added:

`mirror failure -> ProductionServiceContext.reportFailure`.

No wire or persisted schema changes are required.

## 9. Constraints and non-functional requirements

- Do not swallow a local mirror persistence failure without attribution in
  production.
- Do not report a successful writeback for a rejected hydration.
- Do not include raw error text in the `client_persistence` success event.
- Preserve synchronous connect ordering and the existing live-state flag.
- Follow the development freeze: local commit and static checks only.

## 10. Failure modes and edge cases

- Host read rejection remains handled by `BoxSettingsField` and resolves as a
  no-op; it is not escalated as a mirror persistence failure.
- A mirror write/clear throw reaches the production failure reporter once for
  that hydration Promise.
- A failure reporter that itself throws must not recreate an unhandled
  rejection; the callback boundary must be contained.
- Account departure still calls `abandonInFlight()` and then retains its
  existing synchronous clear behavior.
- Duplicate transport callbacks are governed separately by the connection
  ownership specification and are not reintroduced here.

## 11. Implementation strategy

Add the optional callback to the host-settings factory, attach a rejection
handler to the detached hydration Promise, and invoke the callback inside a
contained failure boundary. Bind the production callback in
`main-production-services.ts` to the existing `bindings.reportFailure` owner.
Leave the success telemetry branch and shared field untouched.

## 12. Verification / test strategy

- Static review of the Promise chain and production binding.
- Confirm the success telemetry call remains unchanged and no error-shaped
  `client_persistence` event is introduced.
- Run `git diff --check` and `git show --check` after commit.
- Do not run tests, builds, packaging, CI, or runtime probes in this task.

## 13. Acceptance criteria / Definition of Done

- AC-1: H1–H5 are satisfied in the existing host-settings owner/composition.
- AC-2: Static checks pass and the change is committed locally.
- AC-3: Runtime, tests, packaged, exact-head, and OCRM acceptance remain
  explicitly pending/not-run.

## 14. Release / migration / rollback

No migration. Rollback is reverting the local commit.

## 15. Observability / evidence

The existing failure reporter receives the stable
`host-settings/onboarding-hydrate` classification. Existing successful
`sand.client_persistence` writeback telemetry remains the positive path.

## 16. References / provenance

- `AGENTS.md`
- `docs/specs/openclaw-rust-equivalence-migration.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/transcript-round69-onboarding-io-failure-account-transition-20261024.md`
- `source/electron-main/prefs/host-settings-fields.ts`
- `source/shared/host-settings-field.ts`
- `source/electron-main/main-production-services.ts`
- `source/electron-main/telemetry/client-failure-telemetry.ts`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| H1–H5 | passed | `onTransportConnected()` now consumes hydration rejection, production binds the existing failure owner, success telemetry is unchanged, shared read failures remain no-op, and no new state owner was added. |
| AC-1 | passed | Focused source review confirms the existing host-settings owner/composition satisfies the contract. |
| AC-2 | passed | `git diff --check` passed; local commit follows. |
| AC-3 | passed | Runtime, tests, packaged, exact-head, and OCRM acceptance remain explicitly pending/not-run. |
