# Testing and verification strategy

Fabushi Desktop treats testing as evidence that a specific source revision satisfies a specific requirement. A green test from another SHA is not transferable evidence.

## Verification layers

Use the layers affected by the change:

1. **Static / type / format** — compile/type safety and basic repository consistency.
2. **Architecture checks** — dependency direction, ownership boundaries, forbidden legacy imports/patterns, generated bundle boundaries.
3. **Unit tests** — deterministic local behavior.
4. **Contract/protocol tests** — request/reply/event/schema/state-machine compatibility.
5. **Integration tests** — multi-component behavior across real boundaries.
6. **E2E tests** — user-visible renderer/Electron behavior.
7. **Failure/recovery tests** — cancellation, crash, timeout, reconnect/resync, stale generation, partial failure.
8. **Migration/rollback tests** — old/new state compatibility and rollback safety.
9. **Non-functional tests** — latency, performance, power, security/privacy, accessibility where required.
10. **Packaged acceptance** — real built/signed/notarized/installed behavior when packaging/runtime integration is in scope.
11. **Release verification** — updater metadata, checksums, publication assets, installed version and post-update behavior when release is in scope.

## Requirement-to-test mapping

Non-trivial Specs should map requirement/acceptance IDs to verification.

| Requirement | Verification example |
| --- | --- |
| R3 | architecture checker + contract test |
| R7 | Electron E2E + failure/recovery case |
| AC-8 | exact-head GitHub Actions run |
| AC-12 | published artifact + install/update evidence |

Do not remove narrow failing assertions merely to obtain green CI.

## Exact-source evidence

When GitHub Actions is the acceptance authority:

- checkout must bind to the exact PR head or release source SHA;
- the accepted SHA must be recorded;
- a new commit requires new evidence;
- reruns are acceptable only when still bound to the same source and the reason is understood;
- artifacts/logs/screenshots/traces should identify the run/SHA.

Path-filtered workflows that do not run for a documentation-only change are not runtime validation; report them accurately as not triggered/not applicable.

## Packaged acceptance

Use real packaged acceptance when risk exists only after packaging, signing, entitlements, process spawning, updater configuration, or production runtime wiring. Source-level success must not be used to claim packaged behavior is healthy.

## Failure paths

For Agent/runtime work, test relevant abnormal paths, including provider/tool timeout, first-output stall, cancellation, Host/Runner crash, renderer reload, transport disconnect, reconnect/resync, duplicate/out-of-order events, stale operation/generation, concurrent Agent isolation, permission denial, and restart recovery.

## Evidence retention

Specs/projects should identify required workflow URLs, run IDs, exact SHA, artifacts, logs, traces, screenshots, or video. Evidence must support the claim being made, not merely show that some test ran.
