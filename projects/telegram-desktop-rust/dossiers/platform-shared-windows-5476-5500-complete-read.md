# Shared platform / Windows responsibilities 5476-5500 — complete read

Accepted upstream: `telegramdesktop/tdesktop@42f8a36d43b8c805bc821905bea4cfeb3af1d41d`
Accepted tree: `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`
Range: deterministic orders 5476-5500 (25 exact blobs).

## Responsibility groups

- **5476-5480 — shared platform contracts.** Text-recognition geometry/glyph results, translation-provider construction, tray capability, WebAuthn registration/login result/error semantics, and native window-title/theme preview dispatch. These are source-neutral capability interfaces; platform implementations remain adapters.
- **5481-5482 — Windows exact location.** WinRT requests high-accuracy location and returns an empty result on unavailable/failed async operations. Windows reverse geocoding is explicitly unavailable in this adapter. Canonical owner: location permission/geolocation/reverse-geocode.
- **5483-5484 — Windows file/dialog security.** Open-With handler discovery/lifetime, external mail/file launch, NTFS `Zone.Identifier` Internet-zone postprocess, file-dialog read/write/folder modes, persisted last directory, and large-directory startup fencing. Canonical owners: file/dialog/external-launch/download-security.
- **5485-5486 — Windows integration lifecycle.** Native events are fenced through the sandbox; Jump List/taskbar are projections; shutdown, time-change, screen lock/unlock, settings changes, and a six-state suspend/resume machine with 1s debounce feed canonical lifecycle/security/window state.
- **5487-5488 — Windows launcher/updater.** Unicode command-line parsing, non-modal headless-test crash handling, relaunch/update argument preservation, write-protected `runas`, and parent PID + process-creation-time handoff. Existing Fabushi Windows updater already has Authenticode/signer and parent-creation-time validation plus private staged download hardening, but full protected-updater acceptance remains open.
- **5489-5490 — Windows main-window privacy/lifecycle.** Passcode lock forces DWM thumbnail/live-preview redaction rather than exposing window content. The same adapter handles geometry/workmode, tablet mode, IME start, native theme/high contrast interaction, tray activation, unread taskbar overlays, display identity and native icon lifetime.
- **5491-5492 — Windows native notifications.** Toast identity is scoped by process + session + peer + topic/sublist + message. Inline reply, mark-read and open activations are rejected unless the exact notification remains active. Exact item/topic/sublist/history/session cleanup and Focus Assist/Quiet Hours/user-notification-state policy are part of the responsibility. Current `SandOsNotificationManager` proves scoped source-neutral notification/click cleanup but does not yet expose native inline reply/mark-read actions; keep this production gap open.
- **5493 — Windows overlay.** The adapter uses the shared default MediaViewer/window-chrome overlay helper; no Windows-specific product-state owner.
- **5494-5495 — Windows platform security/settings.** App-data/shortcut cleanup, OpenSSL no-config startup, autostart/send-to integration, crash memory metadata, capture protection, cross-process activation, process wait, microphone/settings routing, dark/high-contrast detection, version migration, maps/currency and platform capability declarations map to existing source-neutral lifecycle/security/settings owners. Screenshot/capture privacy remains open until canonical shipping behavior and exact-head evidence prove equivalence.
- **5496-5497 — native capability absence.** Windows-native OCR and Windows-native translation provider explicitly report unavailable. Fabushi must preserve truthful capability/fallback semantics; these files do not justify inventing a Windows-native OCR/translation subsystem.
- **5498-5499 — Windows tray/taskbar UX.** Tray creation/destruction, deferred menu callback after hide, first-use tooltip persistence, unread/muted/support-mode counters, monochrome icon rendering and taskbar light/dark cache invalidation are native projections of canonical Menu/Badge/theme state.
- **5500 — Windows Hello wallet protection.** Availability requires an appropriate installed/canary/debug-portable environment, TPM presence and Windows Hello support. Enrollment requires hardware attestation, signs a random challenge, derives a wrap key with HKDF-SHA256 and random salt, zeros the signature copy under app control, and distinguishes Absent/Cancelled/Unavailable/Corrupt. Credential retirement is bounded and asynchronous. This remains a critical mapped-open security responsibility.

## Invariants

- Native adapters cannot become a second Conversation, notification, wallet, update, file, window, tray, permission or settings truth owner.
- Notification reply/mark/open must retain exact session/peer/topic-or-sublist/message scope and must fail closed on stale or unknown native activation.
- Locked or protected content must not leak through DWM previews or capture APIs where the platform supports protection.
- Update/relaunch privilege or parent-process handoff never substitutes for authenticated package, trusted staging, preflight/post-install identity and recoverable failure semantics.
- WebAuthn/Windows Hello cancellation, absence, unsupported environment and corrupt cryptographic state are distinct fail-closed outcomes.
- Platform capability absence is an explicit product fact; unsupported native OCR/translation must not be advertised as available.

## Closure policy

All 25 exact blobs are read/decomposed, but every row remains `unknown_closed=false` and `omitted=false`. Read completion grants source-accounting credit only. Production/UI/service/security/testing/release/independent-acceptance evidence must close each applicable responsibility on one exact Fabushi HEAD before unknown may decrease.

## Evidence status

The exact blobs are recorded in `inventory/source-dispositions/5476-5500.json` and `inventory/source-attestation-manifests/5476-5500.json`. The commit carrying this dossier requires fresh exact-head GitHub Actions source-authority and production evidence; the successful `f348ad762b6124bcee3599579141dc556a1b4e7b` runs are predecessor evidence after this accounting commit.
