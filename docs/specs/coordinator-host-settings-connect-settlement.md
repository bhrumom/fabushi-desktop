# Coordinator host-settings connection ownership — Specification

Status: active
Owner: Fabushi Desktop Electron Coordinator / host-settings owner
Last updated: 2026-11-01
Related project: `projects/openclaw-rust`
Related task / issue / PR: N/A

## 1. Context / problem

The production Coordinator `transport-connected` event currently invokes the
same host-settings hydration entrypoint through both the resync port and the
direct service context. The resync port is already bound to
`context.hostSettingsFields.onTransportConnected()`, so one transport event
starts two `absorbFromBox()` operations for the same onboarding mirror. The
corresponding `transport-down` event and main-data-port replacement path also
mark the same owner down twice.

## 2. Goal

Each Coordinator transport lifecycle event invokes the existing host-settings
owner exactly once while preserving the existing hydration, down-state, and
account-departure behavior.

## 3. Non-goals / out of scope

- No new settings store, hydration state owner, generation counter, or retry
  policy.
- No change to Host settings wire schemas or Coordinator event schemas.
- No claim that successful packaged Host hydration, runtime acceptance, or
  cross-transport race behavior is verified by this change.
- No change to the separate resync chain or unrelated Coordinator event
  consumers.

## 4. Requirements

- R1: The production `transport-connected` handler must have exactly one call
  path to `hostSettingsFields.onTransportConnected()`.
- R2: The production `transport-down` and main-data-port replacement paths
  must each have exactly one call path to
  `hostSettingsFields.setBoxStreamLive(false)`.
- R3: The existing resync port remains the adapter used by the Coordinator
  production composition; no duplicate settings owner is introduced.
- R4: Telemetry flush, transport telemetry, MCP/resync behavior, and account
  transition behavior remain unchanged.
- R5: Delivery failures or unproven runtime behavior remain explicitly
  pending; static source review must not be represented as runtime acceptance.

## 5. Current state

`production-provider.ts` handles `transport-connected` by calling
`ports.resync.onHostSettingsTransportConnected()` and then directly calling
`context.hostSettingsFields.onTransportConnected()`. In the production
auxiliary provider, the former is exactly the latter. The down handler and
main-data-port replacement path have the same duplicated adapter/direct
pattern for the live flag.

## 6. Target state

The Coordinator event handler calls the existing resync-port lifecycle adapter
once for each direction. That adapter remains bound to the host-settings owner,
so hydration and live-state transitions retain their current implementation and
ownership without duplicate starts.

## 7. Architecture and ownership boundaries

- `host-settings-fields.ts` owns the desktop mirror and Box stream lifecycle.
- `production-root-auxiliary-provider.ts` owns the production adapter binding.
- `production-provider.ts` owns Coordinator event composition and delegates
  host-settings lifecycle changes through the adapter.
- No Rust, renderer, Host, or second persistence owner is added.

## 8. Interfaces / contracts / schemas / data flow

Existing flow remains:

`Coordinator transport event -> production provider -> resync host-settings
adapter -> hostSettingsFields owner`.

No interface or persisted schema changes are required.

## 9. Constraints and non-functional requirements

- Preserve synchronous event ordering around telemetry and resync calls.
- Do not retry or replay hydration as part of this change.
- Keep account-departure clearing and existing live-state semantics intact.
- Follow the development freeze: local commit and static checks only; no
  tests, builds, packaging, CI, or runtime acceptance.

## 10. Failure modes and edge cases

- A reconnect must not start two concurrent onboarding reads solely because
  of duplicate composition.
- A transport-down event must not depend on duplicate idempotent calls.
- Account departure remains the existing in-flight fence and local mirror
  clear; this change does not replace that fence.
- Host read failure and mirror persistence failure remain governed by their
  existing owners and are not reclassified as successful here.

## 11. Implementation strategy

Remove the direct host-settings lifecycle calls from the Coordinator event and
main-data-port handlers while retaining the existing `ports.resync` adapter
calls. Confirm the adapter binding and inspect the final diff for accidental
changes to adjacent event behavior.

## 12. Verification / test strategy

- Static source review: confirm each production lifecycle direction has one
  adapter call and the adapter resolves to the existing host-settings owner.
- `git diff --check` and `git show --check` after the local commit.
- Do not run tests, builds, package/install flows, CI, or runtime probes in
  this development-freeze task.

## 13. Acceptance criteria / Definition of Done

- AC-1: R1–R3 are satisfied in the production composition source.
- AC-2: R4 is preserved by a focused diff review.
- AC-3: Static checks pass and the work is committed locally.
- AC-4: Runtime, test, packaged, exact-head, and OCRM acceptance remain
  pending/not-run and are reported as such.

## 14. Release / migration / rollback

No migration or release action. Rollback is reverting the local commit.

## 15. Observability / evidence

The source-level call graph, focused diff, and static whitespace check are
required evidence. Existing host-settings persistence telemetry remains the
runtime observability surface; no new event is introduced.

## 16. References / provenance

- `AGENTS.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/transcript-round67-renderer-malformed-reply-and-onboarding-connect-20261024.md`
- `source/electron-main/coordinator/production-provider.ts`
- `source/electron-main/coordinator/production-root-auxiliary-provider.ts`
- `source/electron-main/prefs/host-settings-fields.ts`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1–R4 | passed | `production-provider.ts` now delegates connect, transport-down, and main-data-port replacement lifecycle changes only through the existing resync host-settings adapter; telemetry/resync/account behavior is unchanged in the focused diff. |
| R5 | passed | Development-freeze constraints are recorded; runtime and acceptance are not authorized. |
| AC-1–AC-3 | passed | Focused source review and `git diff --check` passed; local commit follows. |
| AC-4 | passed | Runtime, test, packaged, exact-head, and OCRM acceptance remain explicitly pending/not-run. |
