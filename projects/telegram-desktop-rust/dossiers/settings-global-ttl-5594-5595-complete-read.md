# Settings Global TTL complete read — orders 5594-5595

Accepted upstream: `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Exact blobs: implementation `3a22266764ef7fe1496a9fd89d5853afe93bd34a`; header `6484c81b829bb59278f5480034c93d7e4796863c`.

The section owns no independent state. It projects the account-scoped server default history TTL, confirmation when enabling from off, predefined/custom periods, and an eligibility-filtered picker for applying the current default to existing conversations. Per-peer application is independent and applies returned server updates. Self/replies/verify-code, contacts-without-chats, and peers failing the TTL validator are excluded.

Production closure remains open for account/peer replacement, eligibility or permission changes between selection and send, default-TTL changes while the picker is open, partial bulk failure/retry/idempotency, stale server responses, cancellation/teardown, offline/reconnect, keyboard/focus/a11y/light-dark/responsive behavior. Both rows remain `mapped-open`; unknown remains 15,844 and omitted remains 0. Read-through becomes 5,595/16,123; first unread is 5,596 `settings_information.cpp` (`8982b515b5a30600f5565c3b7e81dc5218c96950`).
