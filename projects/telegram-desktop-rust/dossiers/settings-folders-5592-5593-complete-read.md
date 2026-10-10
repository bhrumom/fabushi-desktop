# Settings Folders complete read — orders 5592-5593

Accepted upstream: `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Exact blobs: `settings_folders.cpp` `8c5b9b4d6b7a5b3166c12ac8e20c50775a84e9dd`; header `8a2984febbf9753dd4aeb698eb7924d4960ca19a`.

This section projects canonical Conversation collection/filter truth. It stages create/edit/remove/restore, assigns collision-free IDs, preserves order, applies local updates, serializes remove/shared-leave/add server requests, and reconciles results. Shared list deletion obtains leave suggestions and selected peer removal. Premium tags use a 500 ms debounce with restart and teardown flush. Vertical/horizontal layout and tab text/icon modes are local presentation preferences.

Production closure remains open: durable filter/order/tag server contract, idempotency, ordered mutations, optimistic rollback/reconnect/account-switch fencing, shared peer-removal safety, premium downgrade, teardown flush/cancellation, count reconciliation, keyboard/focus/a11y/light-dark/responsive evidence. Both rows remain `mapped-open`; unknown remains 15,844 and omitted remains 0. Deterministic read-through becomes 5,593/16,123; first unread is 5,594 `settings_global_ttl.cpp`.
