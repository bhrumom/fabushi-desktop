# Settings Credits complete read — orders 5590-5591

Accepted upstream: `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Exact blobs: `settings_credits.cpp` `dbe0b4cf48e52c30e2066c6ad4181893ed604646`; header `56c88e68b16b4c2c49bcbcc846b9a5969dfb4867`.

This account-scoped Wallet/Payments projection covers Stars/currency balances, top-up option loading and settlement UI, subscriptions and full/in/out transaction history with receipts, statistics/gift/affiliate routing, TON/USD presentation, earn-statistics availability and withdrawal action. Async loading/rebuilders are lifetime-bound; stale-account and cancellation fencing are part of the production responsibility.

Production closure remains open: provider/server authorization, account fencing, top-up idempotency/cancellation, failed/pending/success settlement, balance/receipt reconciliation, history pagination/rebuild dedupe/order, withdrawal availability/error/retry, affiliate policy and keyboard/focus/a11y/light-dark/responsive evidence are required. Both rows remain `mapped-open`; unknown remains 15,844 and omitted remains 0.
