# Device pairing QR presentation

Status: implementation complete and wired; verification not run
Project: OCRM-001
Owner boundary: existing Host `DevicePairingOwner` → existing Coordinator transport → Fabushi Settings renderer

## Context and goal

The production Host already owns device pairing setup-code issuance and returns
an expiring setup payload with `qrDataUrl` and `qrTerminal`. The desktop
renderer had no production consumer, so a user could not complete pairing from
Fabushi's existing UI. Add one discoverable Settings surface that consumes the
canonical `device.pair.setupCode` method and presents the result safely.

## Non-goals

- No second pairing owner, token store, QR encoder, or Gateway protocol.
- No direct renderer access to pairing SQLite or Host internals.
- No persistence, analytics, logging, clipboard history, or automatic approval
  of setup secrets.
- No local build/test/package/device acceptance during the development freeze.

## Requirements

- `DPR-01`: The Settings surface invokes only `coordinatorClient.call("device.pair.setupCode")`.
- `DPR-02`: The response is treated as untrusted transport data; the UI renders
  only a string `qrDataUrl`, `qrTerminal`, `setupCode`, and numeric
  `expiresAtMs` when present.
- `DPR-03`: Loading, transport/error, missing-QR, and expired states are
  visible and retryable. Expiry is computed from the response and refreshed by
  a bounded timer; an expired secret is not presented as usable.
- `DPR-04`: The QR image is rendered with an accessible alternative and the
  setup code is available for deliberate user copy without being written to
  durable state or logs. Terminal output is opt-in in a disclosure.
- `DPR-05`: The feature is mounted in the existing Settings shell and uses the
  existing design-system primitives. It does not add a new navigation or
  pairing authority.

## Current and target state

| Concern | Current | Target |
| --- | --- | --- |
| Setup producer | Host `DevicePairingOwner` | unchanged |
| Transport | Coordinator arbitrary Gateway call | unchanged |
| Desktop consumer | Settings `Pairing` panel is mounted in the existing shell | Settings `Pairing` section |
| Secret lifetime | Host expiry | renderer memory only, visibly expired |

## Failure modes and boundaries

Malformed or incomplete replies are shown as an error and never interpolated
into DOM markup. A missing QR result remains a failed presentation, not a
successful pairing state. Clipboard failure is non-fatal and leaves the secret
visible only in the current in-memory result. Closing Settings drops the result
with the component; no persistence is introduced.

## Implementation and verification strategy

Add a small renderer-only panel, register it in the existing Settings section
registry, and call the already registered Host Gateway method through the
existing Coordinator client. Add focused source oracles for the method name,
response fields, expiry/error branches, and Settings mounting. Verification is
not claimed until exact-head Actions and packaged/runtime evidence are
available.

The focused oracle is
`frontend/src/recovered/features/settings/overlay/device-pairing.contract.test.ts`.
It is intentionally a source contract and does not substitute for runtime or
packaged acceptance.

## Acceptance criteria / Definition of Done

- `DPR-AC01`: Source review proves the chain
  `Settings Pairing panel → Coordinator call → Host device.pair.setupCode →
  DevicePairingOwner → qrDataUrl/qrTerminal consumer`.
- `DPR-AC02`: Source review proves no second pairing or storage owner and no
  secret persistence/logging.
- `DPR-AC03`: Source review proves loading, error, missing-data, expiry and
  retry behavior.
- `DPR-AC04`: Exact-head Actions and packaged/runtime UI evidence are still
  required before `verification=verified` or `acceptance=accepted`.

## Release, rollback, and observability

The change is additive and rollback-safe by removing the Settings panel and
section. Existing Host pairing behavior is unchanged. Renderer errors use the
existing Settings notice path; setup values are not emitted to diagnostics.

## References

- `projects/openclaw-rust/management/tasks/P0-device-pairing-authority.md`
- `projects/openclaw-rust/dossiers/media-qr-png-store-shared-scoped.md`
- `source/host/src/device_pairing_owner.rs`
- `frontend/src/production/coordinator-client.ts`
- `frontend/src/recovered/features/settings/overlay/desktop-surface.tsx`
