# Platform macOS / shared-platform responsibilities 5411-5475 — complete read (corrected deterministic order)

Accepted upstream: `telegramdesktop/tdesktop@42f8a36d43b8c805bc821905bea4cfeb3af1d41d`
Accepted tree: `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`
Range: deterministic root non-directory orders 5411-5475 (65 exact blobs).

> The filename is retained for provenance because it already existed on the branch. The current exact tree proves that Windows-specific files begin at order 5481. Orders 5464-5475 are shared `platform_*` contracts; no Windows source-read credit is claimed for them.

## Responsibility groups

- 5411-5414: macOS security-scoped bookmark lifetime plus file/URL/Open-With policy. Canonical owner: existing file/dialog/external-launch/security boundary.
- 5415-5435: macOS app menu, window/chrome/theme/session lifecycle, integration, launcher and application lifecycle. Canonical owners: application-menu, window chrome, theme, lifecycle, settings and security.
- 5419-5420: macOS launcher/updater handoff. Protected updater trusted-path, authenticated-package, private-staging, preflight/privilege, post-install identity and failure-recovery remain mapped-open until production + same-head evidence closes them.
- 5426-5429: macOS native notification capability/focus policy/actions/activation. Current generic Fabushi notification behavior does not by itself close native inline reply/action parity.
- 5436-5459: OCR, Touch Bar composer/pinned-chat/media projections, translation provider and tray. Reuse canonical Composer, ConversationRow, MediaViewer, Menu and platform capability owners; no platform-named product roots.
- 5460-5462: LocalAuthentication/Secure Enclave wallet protection and AuthenticationServices WebAuthn/passkey. Map to canonical wallet/security/account/auth owners; cancellation, user-verification and security-key equivalence require production + security evidence.
- 5463: native window-title/frame preview rendering maps to canonical window chrome/theme.
- 5464: exact location + reverse-geocode callback contract maps to canonical location permission/geolocation/reverse-geocode owner.
- 5465: shared file-bookmark capability facade maps to canonical file permission/bookmark owner.
- 5466-5467: shared external URL/email/Open-With/file launch, file-dialog, downloaded-file postprocess and nested-event-loop risk map to canonical file/dialog/external-launch/security owners.
- 5468-5469: process-wide Integration creation/interface maps to canonical desktop integration/composition lifecycle owner; adapter state must not become a second product truth.
- 5470: shared launcher adapter dispatch maps to canonical launcher/update/relaunch owner.
- 5471: shared MainWindow adapter dispatch maps to canonical window lifecycle/chrome/theme owner.
- 5472: notification capability/default/enforcement/volume/input policy maps to canonical notification policy/capability owner.
- 5473-5474: MediaViewer overlay title controls, maximize/minimize, hover animation, hit-testing, opacity, file-dialog/notch hooks and pointer projection map to canonical MediaViewer/window chrome/IconButton/Dialog owners with a11y/responsive evidence still open.
- 5475: platform start/finish, app translocation, microphone/camera permission, system settings, app identity, single-instance, autostart, tray/taskbar, screenshot protection, crash metadata, maps/currency and third-party lifecycle map to canonical platform lifecycle/security/settings/integration owners.

## Closure policy

Every row is source-read complete but remains `unknown_closed=false`. Reading a blob is not implementation credit. No unknown count is reduced. No Telegram/macOS/Windows parallel product runtime, source-named UI, second notification store, second credential store or second updater owner is permitted.

## Evidence status

The corrected exact blobs and consumer labels are recorded in `source-dispositions/5411-5475.json` and `source-attestation-manifests/5411-5475.json`. Same-head GitHub Actions must attest the corrected deterministic order. Windows-specific responsibilities start at order 5481 and require a later exact-read shard before they receive source-read credit.
