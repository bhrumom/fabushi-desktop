# Platform macOS / Windows responsibilities 5411-5475 — complete read

Accepted upstream: `telegramdesktop/tdesktop@42f8a36d43b8c805bc821905bea4cfeb3af1d41d`
Accepted tree: `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`
Range: deterministic recursive orders 5411-5475 (65 exact blobs).

## Responsibility groups

- 5411-5414 and 5464-5465: security-scoped bookmark lifetime, file/URL/Open-With launch policy, Windows downloaded-file zone handling. Canonical owner: existing file/dialog/external-launch/security boundary.
- 5415-5435 and 5468-5475: native app menu, window/taskbar/DWM/IME/theme/lock/session lifecycle, platform integration, permissions/autostart/single-instance/screenshot protection. Canonical owners: application-menu, window chrome, theme, lifecycle, settings, security.
- 5419-5420 and 5466-5467: native launcher/updater handoff. Canonical Fabushi updater now additionally requires a 64-hex SHA-256 on Windows/iupdate offers, uses exclusive mode-0600 staged package creation and mode-0700 staging directories, then retains Authenticode verification, safe relaunch gating and Windows parent identity handoff. macOS authenticated apply, privileged/preflight equivalence where applicable, and post-install installed-artifact identity remain open; keep mapped-open.
- 5426-5429 and 5472-5473: macOS legacy/UserNotifications and Windows WinRT notification capability, focus/DND policy, inline reply/actions, activation and exact-scope cleanup. Current generic Fabushi OS notification manager proves focus/click/scope-clear subsets only; native inline reply/action parity remains open.
- 5436-5459: OCR, Touch Bar composer/pinned-chat/media projections, translation provider and tray. Reuse canonical Composer, ConversationRow, MediaViewer, Menu and platform capability owners; no platform-named product roots.
- 5460-5462: LocalAuthentication/Secure Enclave wallet protection and AuthenticationServices WebAuthn/passkey. Map to canonical wallet/security/account/auth owners; native registration/assertion, cancellation/user-verification and security-key equivalence require production + security evidence.

## Closure policy

Every row is source-read complete but remains `unknown_closed=false`. Reading a blob is not implementation credit. No unknown count is reduced. No Telegram/macOS/Windows parallel product runtime, source-named UI, second notification store, second credential store or second updater owner is permitted.

## Evidence status

The exact blobs and consumer labels are recorded in `source-dispositions/5411-5475.json` and `source-attestation-manifests/5411-5475.json`. Same-head GitHub Actions must attest the updated master accounting. Production gaps identified here remain open until the canonical shipping owner plus contract/integration/security/E2E/packaged evidence is present on one exact HEAD.
