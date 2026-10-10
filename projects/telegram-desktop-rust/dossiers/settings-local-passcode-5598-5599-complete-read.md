# Settings Local Passcode complete read — orders 5598-5599

Accepted upstream: `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Exact blobs: implementation `0f1b6ace690851441a588074cbff816fb3e5a8a1`; header `b0292c43e503bc8b7ad2fa66f9681be59c1c272c`.

This is local-device security, not Cloud Password/2SV. Verified passcode bytes are retained only by the current security section and expire after 60 seconds without interaction; mouse/key activity inside the owner restarts the countdown, busy crypto flows pause it, external passcode changes make retained bytes stale, and navigation handover is one-hop with cleanup. Worker derivation/check paths enforce create/check/change; wrong checks increment retry state. Transient UTF-8 passcode copies are explicitly cleansed.

Manage revalidates live wallet key-protection policy before disabling the passcode, uses fresh verification for weakening writes, fails back to Check on stale/refused proof, separates launch app-lock from the existence of a passcode, and exposes auto-lock plus Windows Hello/Touch ID/Apple Watch/system-password unlock only when available. Passcode removal disables system unlock.

Production closure remains open for crypto/storage atomicity, stale/external-change fencing, 60-second expiry, retry throttling, vault migration partial failure/cancel, app-lock proof, system-unlock availability/toggle, navigation/close cleanup, and security/a11y E2E. Both rows remain `mapped-open`; unknown remains 15,844 and omitted remains 0. Read-through becomes 5,599/16,123; first unread is 5,600 `settings_local_storage.cpp`.
