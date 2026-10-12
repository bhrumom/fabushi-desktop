# Settings Local Storage + Main — deterministic orders 5600–5604 complete read

Authority: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, root tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

All five exact blobs are read in full and decomposed. Source reading grants no production closure: every row remains **mapped-open**, unknown stays 15,844, omitted stays 0.

## 5600–5602 Local Storage
Two account-local cache databases (normal + big media) feed one storage surface. The user can select image/sticker/voice/video/animation/media categories and clear exact tags or all cache; all-clear also clears irrelevant emoji cache. Duplicate clear requests are fenced. Completion waits for both cache databases to settle plus a 1.2-second minimum presentation interval, then freed bytes are recomputed from live stats. Device capacity is refreshed asynchronously. Total cache, large-media cache and retention are one policy; total minus media cache must remain at least 100 MB.

Canonical target is the existing Fabushi Settings/Storage + Cache/MediaCache policy owner and filesystem-capacity adapter. Required closure includes partial failure, restart/account-switch, stale result, clear idempotency, quota/retention persistence and full a11y/visual states.

## 5603–5604 Settings Main
Settings Main is composition, not a second owner. It projects active-account profile state and routes canonical Notifications, Privacy/Security, Chat, Folders, Advanced, Devices/Calls, Power, Language, Premium, Credits/Currency, Business/Gifts and Help. It also provides add-account/logout, interface-scale preview/confirm/restart, validation suggestions and support routing, and refreshes cloud-password/privacy/premium/theme/FAQ state at entry.

Fabushi must reuse the one canonical Settings shell plus Identity/Profile, Account/Session, Privacy/Security, Wallet/Payments, Business, preferences and platform adapters. Async operations require exact session fencing, cancellation/retry/idempotency, rollback/recovery and teardown evidence.

## Accounting
Recursive denominator 16,123; read-through **5,604**; unread **10,519**; unknown **15,844**; omitted **0**. First unread is order **5,605** `Telegram/SourceFiles/settings/sections/settings_notifications.cpp`, blob `a16c3ee9690de934d795a2803ada0e82c3f9db94`.
