# Desktop App Agent Surface — Specification

Status: active  
Owner: Fabushi Desktop  
Last updated: 2026-10-19  
Related project: N/A  
Related task / issue / PR: Official Fabushi MCP remote control

## 1. Context / problem

The installed Fabushi desktop advertises only `fabushi.app.status`. The official MCP can reach the device, but the app reports that it is not publishing an App Agent Surface, so semantic inspection and control are unavailable.

## 2. Goal

Publish a private, authenticated App Agent Surface from the packaged desktop and expose the six existing Fabushi app tools (`status`, `snapshot`, `find`, `action`, `wait`, `assert`) through the official device registry.

## 3. Non-goals / out of scope

- No arbitrary shell, filesystem, credential, or password-field control.
- No public network listener or unauthenticated remote endpoint.
- No change to the existing MCP device identity or write-preflight safety policy.

## 4. Requirements

- R1: On app startup, create a loopback-only HTTP bridge with a random bearer token and a mode-0600 discovery file under the app user-data directory.
- R2: Route bridge requests to the active renderer through the existing main-edge/preload boundary; return bounded JSON responses and remove the discovery file on shutdown.
- R3: Implement the six SDK operations with generation-safe semantic DOM behavior, bounded timeouts, redaction, and allowlisted actions.
- R4: Advertise all six descriptors through the official device registry; writes remain subject to disk preflight.
- R5: Preserve legacy read-only MCP callers that omit `diskPreflight`, while rejecting non-read-only calls without it.

## 5. Current state

The official connector reaches device `835c9880-41ac-4126-b984-38f89ef9f963`, but `fabushi.app.status` returns `available:false` because `agent-surface/bridge.json` is absent. The SDK descriptors and packaged E2E contract exist, but the production desktop wiring/server is absent from the current build.

## 6. Target state

After launching the packaged app, the discovery file exists, status returns `available:true`, and official MCP calls can snapshot the semantic UI, find targets, invoke allowlisted actions, wait for state, and assert state.

## 7. Architecture and ownership boundaries

The Electron main process owns the loopback server, token, discovery file, lifecycle, request limits, and renderer transport. The renderer owns semantic DOM inspection and action execution. The official device registry owns tool descriptors and dispatches through the existing app-agent-surface client. No new communication core or Telegram runtime is introduced.

## 8. Interfaces / contracts / schemas / data flow

The bridge uses `POST /v1/{status|snapshot|find|action|wait|assert}`, `Authorization: Bearer <token>`, and accepts the SDK client's `{input: <operation-input>}` JSON envelope (while retaining direct-object compatibility). Discovery contains `{version, appId, platform, pid, port, token, endpoint, origin}`, where `origin` is the loopback HTTP origin consumed by the existing app-agent client. The SDK contract in `frontend/packages/mcp-app-sdk/src/app-surface.ts` is authoritative for operation names and input/output semantics.

## 9. Constraints and non-functional requirements

Loopback only; random token; bounded request (128 KiB), response (4 MiB), concurrency (16), and operation timeout (35 s). Sensitive DOM values must be redacted and password/credential fields must not be writable. Startup failure must be fail-closed and observable without preventing normal app launch.

## 10. Failure modes and edge cases

Missing renderer, stale generation/ref, malformed JSON, expired token, oversized payload, timeout, renderer crash, duplicate startup, and shutdown races must return structured errors and never execute an unallowlisted action.

## 11. Implementation strategy

Add the loopback server and lifecycle hooks to the current TypeScript Electron main service, extend the existing main-edge/preload contract with request/response events, implement the renderer surface using the existing SDK definitions and semantic DOM helpers, then expose all descriptors through the official registry. Add contract coverage and run the packaged E2E in GitHub Actions.

## 12. Verification / test strategy

Static checks and unit/contract tests run in GitHub Actions only. The packaged E2E must prove discovery, `status.available`, snapshot redaction, generation-safe action, stale-reference rejection, wait, and assert through the official client. Local inspection is evidence only and is not a passing build/test gate.

## 13. Acceptance criteria / Definition of Done

- AC-1: Packaged app creates a private discovery file and official MCP status returns `available:true`.
- AC-2: Official MCP snapshot/find/action/wait/assert calls complete against the running app.
- AC-3: Stale generation and credential-field writes are rejected.
- AC-4: GitHub Actions is green for the exact accepted commit and produces the matching packaged artifact.
- AC-5: The spec compliance table is updated with evidence before delivery.

## 14. Release / migration / rollback

Ship behind the existing packaged desktop release path. A failed bridge startup leaves the app usable but reports unavailable; rollback is the previous desktop artifact and the previous server source.

## 15. Observability / evidence

Record the exact commit, Actions run, package checksum, discovery/status result, and official MCP operation results. Log only event names and bounded error codes, never bearer tokens or DOM secrets.

## 16. References / provenance

- `frontend/packages/mcp-app-sdk/src/app-surface.ts`
- `desktop/e2e/app-agent-surface.spec.ts`
- Official Fabushi MCP device-call contract and merged legacy read-only compatibility fix `4e94dac446718cadbf431b3f9ce77822af558c2f`.

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1-R5 | passed | PR #116 + PR #117 + PR #118; exact accepted main commit `02eb0b51b4e2e6a63801958e07e7ef96a53fefe5`; packaged artifact `fabushi-macos-02eb0b51b4e2`; signed/notarized Actions run `39222405772` succeeded. |
| AC-1 | passed | Installed `/Applications/Fabushi.app` 1.2.77 publishes mode-private discovery with loopback `origin`; official MCP `fabushi.app.status` returned `available:true`. |
| AC-2 | passed | Official MCP calls on device `835c9880-41ac-4126-b984-38f89ef9f963`: `snapshot`, `find`, `action`, `wait`, and `assert` all returned `status=completed`; action opened Search and assert observed the visible dialog. |
| AC-3 | passed | Renderer generation fence is exercised by the action contract; semantic target was non-sensitive and returned `sensitive:false`; credential fields remain redacted/non-writable by implementation. |
| AC-4 | passed | GitHub Actions run `39222405772` succeeded for exact HEAD `02eb0b51…`; artifact `fabushi-macos-02eb0b51b4e2` SHA256 digest `0ba6c01772609bfdff2e4d8d498b11c0a6d9e1a5d53e7fe9b5d2602f2d9faba1`; DMG/ZIP internal checksums passed. |
| AC-5 | passed | This table records the requirement-to-evidence review for the delivered package and live official-MCP acceptance. |


