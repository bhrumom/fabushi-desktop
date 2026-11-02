# Electron update source authority

Status: active; batch implementation and verification pending, release not accepted  
Date: 2026-11-02  
Owner: existing Electron updater / Settings update projection  
Projects: FBCP-001, TDRP-001; task 20bf9475-a885-4314-b509-bea53ef8849f, round 21

## Context and scope

At Desktop `8ce0a9a43b7fe94df2ec2085399fe65a57343577`, shipping `entry.ts` selects `createElectronProductionUpdaterInstallerBinding`, whose existing `createProductionUpdaterInstallerBinding` calls `createUpdateServiceWiring` and starts `SandUpdateService`. The wiring reads optional `SAND_UPDATE_FEED_BASE_URL`; the service otherwise selects the built-in `https://api2.cursor.sh/updates` from `update-feed.ts`. Startup, periodic, manual and minimum-version checks can therefore send the machine identifier to that historical third-party default.

This batch removes that implicit authority and preserves an explicitly configured approved feed through the same service. It does not claim an operational Fabushi update endpoint, authorize deployment/publication, replace the updater, change user-data schemas, or complete signing/package/install/rollback acceptance. Fixed source U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. CSI-R1/3/5/6 and the latest round-21 instruction govern this intentional adaptation of the recovered updater.

## Requirements and behavior

- **EUS-01 — Explicit source.** No built-in feed URL. `FABUSHI_UPDATE_FEED_BASE_URL` is primary; `SAND_UPDATE_FEED_BASE_URL` remains an explicit compatibility configuration only when the primary variable is absent. Present-empty or invalid primary configuration must not fall back to the compatibility value. Configuration is trusted main-process startup input, never renderer input. No feed is persisted to Settings or inferred from account credentials.
- **EUS-02 — Approved origin.** Reuse `source/shared/node/fabushi-service-origin.ts::normalizeFabushiApiBaseUrl` to validate the explicit URL's origin. Production/isolated CI HTTPS and loopback HTTP/HTTPS follow that existing authority; no arbitrary foreign host or historical Cursor origin is accepted. Preserve a configured path prefix for the existing feed wire protocol. Reject credentials, query, fragment, malformed URLs and remote HTTP. Reject before a fetch; error/status/telemetry must not echo the rejected raw value. No metadata redirect may escape the selected origin: manifest requests use `redirect: "error"`.
- **EUS-03 — Fail closed throughout lifecycle.** The service independently resolves its `feedBaseUrl`, even when constructed without production wiring. Missing/blank source yields `update-source-unconfigured`; invalid or unapproved source yields `update-source-invalid`. The existing explicit environment, Lab, packaging and platform disabled reasons retain precedence. Disabled startup/manual/periodic/track-change/minimum-version/backend checks do not fetch or install and do not arm timers. `applyDisabledReason(null)` cannot bypass source configuration. Disposal prevents a new check. Enabling an approved source requires reconstructing the service from the updated startup configuration.
- **EUS-04 — Preserve behavior and recovery.** Valid explicit configuration retains current track selection, requests, manifest parsing, download/signature/staging/install owners, safe relaunch and cancellation. Existing `sand-update-apply-marker.json` format and path, settings keys, and user/session data stay unchanged. Startup still settles a previous apply marker through the existing settlement owner even when updates are disabled; disabling source must not skip the recovery receipt or invent a successful installation. Normal settlement clears only its own marker. No user-data cleanup or automatic rollback to the historical feed.
- **EUS-05 — Observable state.** Existing update status IPC and Settings projection represent the two source-disabled reasons with fixed user-facing messages. Add non-retryable telemetry causes for these reasons. HTTP status errors omit request URLs, which contain the machine identifier. No new updater IPC or service authority is added.
- **EUS-06 — Evidence.** Behavior tests execute the real service/wiring, source resolver, telemetry, Settings projection and marker store with controlled fetch/clock/platform ports. All executable checks run only in GitHub Actions at the submitted exact H. Source reading, authored tests, execution and independent acceptance remain distinct in the canonical ledger/RTM.

## Implementation and interfaces

Extend existing `update-feed.ts` with `configuredUpdateFeedBaseUrl(env)` for raw variable precedence and `resolveUpdateFeedSource(raw)` for validated source/disabled reason. Reuse the shared origin policy; do not create a second allowlist. The service stores this immutable resolution, combines its reason with existing runtime gates, and consumes only its validated base URL. Production wiring forwards the explicit raw choice and the injected platform; the composition's existing dev-feed override recognizes the same primary/compatibility choice. New disabled reason literals are carried through the existing typed renderer contract and Settings message function. No account credential or schema change is required.

`start()` first runs the unchanged apply-marker settlement and then handles the source gate. A valid feed follows the existing idle/checking/available/downloading/staging/ready transitions. A missing/invalid feed stays disabled when manual checks, track changes or remote experiment callbacks arrive. An approved feed failure returns through the existing idle/error or staged-ready recovery, without selecting another feed. Existing user-selected track and idle-update settings remain writable and recoverable while disabled.

## Verification and acceptance

Use the existing Electron contract job and desktop's locked esbuild dependency: `node --test ../source/electron-main/update/update-source.contract.test.mjs` from `desktop`. Compile the actual Electron entry and affected renderer. The focused contracts must prove:

1. Absent/blank source: no startup timers, fetch, staging or install; manual/track/minimum-version/backend/gate events cannot bypass it.
2. Invalid, foreign, credential-bearing and remote-HTTP source: fixed disabled state, no fetch, and no raw sensitive value in status/telemetry.
3. Explicit primary and compatibility settings: correct precedence; invalid/blank primary does not silently use a valid legacy setting; approved production/CI/loopback path prefixes remain usable.
4. Actual valid service request: expected track/version/machine path, no-cache header, abort signal and redirect refusal; 204 remains a true no-update result and errors remain errors without fallback.
5. Disabled reason precedence and distinct non-retryable telemetry/Settings messages.
6. Existing persisted requested/spawned apply markers settle at restart, including missing-feed and invalid-feed startup, without altering unrelated user data.
7. Existing in-flight request cancellation and no post-disposal request; a later service with corrected explicit configuration can check again.

Acceptance of this batch requires current-H Actions evidence for the focused contracts, actual Electron/renderer compilation and canonical evidence traceability. Real signed platform packages, authorized feed deployment, metadata/download provenance, Windows signing policy and real install/upgrade/rollback remain separate unresolved release gates. A passing mock/controlled-fetch contract is not live-service or package acceptance.

## Rollout, rollback and evidence

Use the existing checkpoint and non-forced expected-head CAS. Retain prior evidence and publish exact source/run/job/artifact identities through the existing round receipt and RTM. A misconfigured deployment remains disabled until its approved explicit source is corrected and the application restarts. Code rollback must retain the no-default and approved-origin boundary; it may not silently restore Cursor traffic. No source or legal provenance is deleted.

References: root `AGENTS.md`; both FBCP/TDRP Source of Truth and main specs; `docs/specs/fabushi-deferred-ci-migration.md` Revision 2, especially §12–13; `docs/specs/fabushi-cursor-service-independence.md`; reviewed exact-base `source/electron-main/entry.ts`, `production-binding-providers.ts`, `update/*`, `frontend/src/recovered/contracts/desktop-bridge.ts`, `features/settings/overlay/updates.ts`, `source/shared/node/fabushi-service-origin.ts`.

## Compliance record

| Requirement | State before execution | Evidence owner |
| --- | --- | --- |
| EUS-01–05 | Implemented / 16 focused contracts authored; execution and acceptance pending | Exact-H source and focused behavioral contracts |
| EUS-06 | Actions execution pending | Existing native-service-contracts Electron/UI jobs, canonical round receipt/RTM |
| Real authorized feed, signed install, rollback | Blocked / pending separate release qualification | Existing platform release owner; no release authorization in this task |
