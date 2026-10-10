# Notification Type + Passkeys — deterministic orders 5610–5613 complete read

Authority: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, root tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

## Notification type 5610–5611
Private/group/broadcast defaults own enable/mute, sound/tone/volume and exact peer exceptions. Exception lists react to authoritative peer notification changes; invalid peer classes are excluded; add/reset/clear operations mutate canonical notification settings and require retry/idempotency/account-switch evidence.

## Passkeys 5612–5613
Account Passkeys are a product-auth lifecycle, distinct from Fabushi's existing remote Agent WebAuthn proxy infrastructure. Source flow is server challenge -> platform authenticator -> server finalize; unsigned/unsupported/cancel failures fail closed, and delete targets exact server credential id. Canonical Account Auth/Passkeys UI/state ownership and signed packaged WebAuthn evidence therefore remain open.

Accounting after this batch: total 16,123; read-through **5,613**; unread **10,510**; unknown **15,844**; omitted **0**. First unread is 5614 `settings_premium.cpp` blob `60ac3cbbfcd4b74eda0d3cbca46c969a22878b94`.
