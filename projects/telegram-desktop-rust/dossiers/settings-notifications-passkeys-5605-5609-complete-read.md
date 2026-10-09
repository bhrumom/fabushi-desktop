# Notifications + Passkeys — deterministic orders 5605–5609 complete read

Authority: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, root tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

All five blobs are exact-source read-complete. They remain **mapped-open**; reading does not reduce unknown.

## Notifications 5605–5607
The Settings surface combines server-backed per-type mute/sound/exceptions and reaction policies with app preferences for desktop notifications, master sound/volume, content preview privacy, badges, multi-account scope, native/custom manager selection, custom position/count and multi-display targeting. Disabling all-account notifications explicitly clears inactive-session notifications. Contact-join and incoming-call switches mutate account/server state. Native/custom manager recreation is a platform adapter boundary.

Canonical Fabushi owners are Notification Policy, Account/Session notification scope, Privacy preview policy, Call authorization and Platform Notification adapters. Native reply/mark-read/open remains an exact-scope authorization pipeline responsibility; Settings may configure that pipeline but cannot own message truth. Closure requires account-switch/stale-result/retry/reload, exact inactive-session cleanup, manager recreation, native capability absence/enforcement, display topology changes, keyboard/focus/a11y and packaged native action regressions.

## Passkeys 5608–5609
Passkey creation is a strict chain: server `initRegistration` -> platform `WebAuthn::RegisterKey` -> server `registerPasskey`. Unsupported or unsigned builds fail closed; UI callbacks are lifetime-guarded. Deletion uses the exact server passkey id behind explicit confirmation. The list projects name, software emoji and created/last-used metadata from the canonical session Passkeys owner.

Fabushi must bind this to its canonical Account Authentication/Passkeys owner plus a bounded platform WebAuthn adapter. Required evidence covers stale/expired challenge, user cancel, unsupported and unsigned builds, account switch/teardown, platform-success/server-failure reconciliation, duplicate/idempotent registration/delete, exact credential-id scope, reload/restart, and signed packaged Windows/macOS WebAuthn acceptance.

## Accounting
Recursive denominator 16,123; read-through **5,609**; unread **10,514**; unknown **15,844**; omitted **0**. First unread: order **5,610**, `Telegram/SourceFiles/settings/sections/settings_premium.cpp`, blob `60ac3cbbfcd4b74eda0d3cbca46c969a22878b94`.
