# Coordinator account-transition cleanup owner wiring — Specification

Status: active
Owner: Fabushi Desktop Electron Coordinator account-transition owner
Last updated: 2026-11-01
Related project: `projects/openclaw-rust`
Related task / issue / PR: N/A

## 1. Context / problem

The repository already contains `createProductionAccountTransitionCleanup`,
which owns the account-departure latch, connector fence, host-settings clear,
and local scope/descriptor/MCP cleanup. The production Coordinator composition
does not use this owner. Instead, `production-provider.ts` performs
`onAccountDeparted()` before calling the account port, while
`production-root-provider.ts` duplicates only part of the cleanup. If the
onboarding mirror clear throws, the account port is never entered and its
`finally` cleanup is skipped.

## 2. Goal

Wire the existing account-transition cleanup owner into the production account
port so every account departure reaches the same cleanup settlement boundary,
including when local onboarding mirror persistence fails.

## 3. Non-goals / out of scope

- No new account, settings, descriptor, MCP, or connector state owner.
- No change to account authorization, descriptor binding, or account-slot
  derivation semantics.
- No change to the Coordinator account-runtime stop/start ordering beyond
  ensuring the existing preparation owner is always entered.
- No runtime, packaged, CI, or OCRM acceptance claim during the development
  freeze.

## 4. Requirements

- A1: Production account transition preparation must use the existing
  `createProductionAccountTransitionCleanup` owner.
- A2: The owner must set the account-departing latch and fence connector
  observations before host/local cleanup, exactly once per departure.
- A3: Host-settings clear failure must be reported and must not skip local
  account-scope, descriptor, or MCP cleanup in the owner's `finally` path.
- A4: `production-provider.ts` must not perform a second account-departure
  latch or host-settings mirror-clear before delegating to the account owner.
- A5: Startup/no-previous-account transitions retain their no-op preparation
  behavior.
- A6: Existing authorization, reset, status delivery, and failure ownership
  remain unchanged.

## 5. Current state

`source/electron-main/coordinator/account-transition-cleanup.ts` exposes the
required owner but has no production caller. `production-root-provider.ts`
implements a partial duplicate that clears Host settings and then runs local
cleanup in `finally`. `production-provider.ts` calls
`accountLifecycle.beginTransition()` and `hostSettingsFields.onAccountDeparted()`
before invoking the account port, so a synchronous mirror-clear error can
prevent the partial owner from running.

## 6. Target state

`production-root-provider.ts` constructs the existing cleanup owner per account
port and delegates `prepareAccountTransition` to it with the already-owned
context callbacks. The root account-lifecycle adapter exposes the transition
latch and connector fence as separate callbacks so the owner can invoke each
exactly once. `production-provider.ts` delegates preparation without
performing the duplicate latch/mirror-clear calls. The cleanup owner remains
the sole transition-preparation owner.

## 7. Architecture and ownership boundaries

- `account-transition-cleanup.ts` owns account-departure preparation and its
  failure/`finally` settlement.
- `production-root-provider.ts` binds that owner to existing context services.
- `production-provider.ts` owns Coordinator composition but only delegates
  the transition request.
- `coordinator-account-runtime.ts` remains the caller and existing error
  consumer; it is not made a second cleanup owner.

## 8. Interfaces / contracts / schemas / data flow

Existing flow becomes:

`Coordinator account runtime -> production provider delegation -> account port
-> createProductionAccountTransitionCleanup -> existing context owners`.

The existing transition payload and persisted settings schema are unchanged.

## 9. Constraints and non-functional requirements

- Preserve cleanup ordering: latch, egress fence, onboarding mirror fence/clear,
  Host clear attempt, then local scope/descriptor/MCP cleanup.
- Preserve best-effort Host-clear reporting and `finally` cleanup semantics.
- Do not introduce a second call to `noteAccountDeparted`, mirror clear, or
  account latch.
- Local commit and static checks only; no tests, builds, CI, packaging, or
  runtime probes.

## 10. Failure modes and edge cases

- Onboarding mirror clear throws: Host/local cleanup owner is still entered
  and its `finally` work remains reachable; the thrown error is reported at the
  established failure boundary.
- Host settings clear rejects: existing owner reports it and continues its
  `finally` cleanup.
- Descriptor or MCP cleanup rejects: existing account-runtime error handling
  remains responsible; this change does not invent a new recovery policy.
- `previousSlot == null`: cleanup remains a no-op.
- Account authorization's foreign-scope mirror cleanup remains a separate
  authorization path and is not conflated with transition preparation.

## 11. Implementation strategy

Import and instantiate `createProductionAccountTransitionCleanup` in the
production account-port provider, binding its callbacks to the existing
`ProductionServiceContext`. Replace the duplicated inline preparation body
with the owner call. Remove the Coordinator provider's pre-delegation
`beginTransition()` and `onAccountDeparted()` calls so the owner performs them
once.

## 12. Verification / test strategy

- Static call-graph review confirms the cleanup factory has one production
  caller and the provider no longer pre-clears the mirror.
- Review callback order and the existing `finally` boundary.
- Run `git diff --check` and `git show --check` after commit.
- Do not run tests, builds, packaging, CI, or runtime probes in this task.

## 13. Acceptance criteria / Definition of Done

- AC-1: A1–A6 are satisfied in production source.
- AC-2: Static checks pass and the change is committed locally.
- AC-3: Runtime, tests, packaged, exact-head, and OCRM acceptance remain
  explicitly pending/not-run.

## 14. Release / migration / rollback

No migration. Rollback is reverting the local commit.

## 15. Observability / evidence

Existing `reportFailure("host-settings", "clear", error)` and account runtime
problem reporting remain the evidence surfaces. No new telemetry schema is
introduced.

## 16. References / provenance

- `AGENTS.md`
- `docs/specs/openclaw-rust-equivalence-migration.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/transcript-round69-onboarding-io-failure-account-transition-20261024.md`
- `source/electron-main/coordinator/account-transition-cleanup.ts`
- `source/electron-main/coordinator/production-root-provider.ts`
- `source/electron-main/coordinator/production-provider.ts`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| A1–A6 | passed | The production account port now delegates to `createProductionAccountTransitionCleanup`; latch and egress fence are separate explicit callbacks, the provider no longer pre-clears, and existing authorization/reset/status paths remain unchanged. |
| AC-1 | passed | Static call-graph review confirms one production cleanup owner and preserved `finally` ordering. |
| AC-2 | passed | `git diff --check` passed; local commit follows. |
| AC-3 | passed | Runtime, tests, packaged, exact-head, and OCRM acceptance remain explicitly pending/not-run. |
