# Premium settings / entitlement commerce — deterministic orders 5614–5616 complete read

Authority: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

5614 composes server-configured feature order, exact-account entitlement/subscription options, amount/currency projection, source/ref attribution, gift/status contexts, promo telemetry, dynamic subscribe visibility and trusted bot/invoice purchase routing. Invalid/unavailable routing fails closed; animation/media/top-bar state is derived. 5615 is the public route/payment interface. 5616 is presentation-only and maps to canonical design-system primitives.

Fabushi mapping stays source-neutral: canonical Entitlement/Subscription + Wallet/Payments/Commerce owns truth; Settings and reusable components project it. Required evidence includes duplicate/cancel/failure/settlement/reload/restart/account-switch reconciliation, stale option/currency refresh, invalid-route fail-closed and packaged accessibility/theme/responsive coverage.

Accounting: total 16,123; read-through **5,616**; unread **10,507**; unknown **15,844**; omitted **0**. Reading alone does not reduce unknown. First unread: 5617 `settings_privacy_security.cpp` blob `b5c5ec4a2322ab43322778a62ebb1102cbf6f913`.
