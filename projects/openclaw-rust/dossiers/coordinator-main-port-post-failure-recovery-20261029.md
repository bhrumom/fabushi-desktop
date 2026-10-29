# Coordinator main-data post-failure recovery — 2026-10-29

Status: production-source fix committed on the current branch; executable verification and packaged account-transition acceptance remain pending.

## Scope and authority

This is a focused follow-up to the read-only liveness finding in
`transcript-round65-renderer-js-first-permission-and-main-port-post-20261024.md`.
The applicable product authority remains OCRM-001 and the existing Coordinator
main-data owner. No second RPC/session owner is introduced.

Current implementation branch: `codex/openclaw-rust-full-fusion-20261029`.

## Failure and boundary

`createCoordinatorMainPortClient` stores a pending call before posting a
request frame. The production `adoptPort` wrapper previously caught
`MessagePort.postMessage` failures, reported `leg-post`, and returned without
letting the client settle the pending call. A port that neither replied nor
closed could therefore leave `setHostSettings` (and the serialized
account-transition chain that awaits it) pending indefinitely.

The fix stays inside the existing Coordinator owner:

- the port wrapper rethrows a post failure after reporting it;
- the client converts a request post exception into a `post-failed` settlement,
  rejects all pending calls through the existing settle path, and closes the
  endpoint;
- hello-stage post failure is caught by `adoptPort`, reported, and closed
  without replacing a still-live previous generation.
- protocol-breach shutdown-post failure is contained so malformed input still
  reaches the local breach settlement instead of throwing before pending calls
  are released.
- explicit `dispose()` uses the same contained shutdown-post path, so generation
  revocation cannot leave pending callers behind when the endpoint is already
  broken.
- endpoint close failure is contained after the local phase is marked settled,
  so cleanup failure cannot prevent callers and the settlement Promise from
  being released.

This does not claim that a remote side effect did not occur. It only settles
the local transport call as failed and preserves the existing generation and
shutdown boundaries.

## Oracle and remaining evidence

`source/electron-main/coordinator/coordinator-main-port-client.contract.test.ts`
asserts the direct negative cases: request post throws, the pending promise
rejects with `post-failed`, hello post failure does not replace the current
generation, a protocol-breach shutdown post failure still settles and closes
the endpoint, explicit disposal settles pending calls even when its shutdown
post throws, and endpoint close failure does not prevent settlement.

The oracle has not been executed locally. Required remaining evidence is the
exact-target GitHub Actions run plus a packaged-equivalent account-transition
exercise with a real main-data port, including close, normal reply, and
disposal controls. This slice does not prove Host settings support, cross-
account authorization, or full OCRM acceptance.
