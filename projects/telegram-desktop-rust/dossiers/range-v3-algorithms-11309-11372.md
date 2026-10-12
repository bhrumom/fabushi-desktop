# range-v3 generic algorithm regressions — recursive orders 11,309–11,372

## Authority

- Upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`, root tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6`.
- Nested source: `ericniebler/range-v3@a81477931a8aa2ad025c6bda0609f38e09e4d7ec`.
- Exact-head validation: `2ab515facff678df69eb53834f37ae1901cbe47d`, Rust desktop runtime `38153128885` completed/success.
- Source-authority: job `114510518094`, artifact `11707477754`, digest `sha256:2a7a3754612ea3d53070e19f4df3c562ed3341c1dc447d5142d20913b759cb63`.
- Window: 64/64 exact blobs, 11,309 `test/algorithm/mismatch.cpp@4a70c8a75afa1dabc6fe6e0628ab288919c1f753` through 11,372 `test/algorithm/stable_sort.cpp@ddc5a674dd038bd7aaa8d2b52bf19781148cdd7c`.

## Responsibility decomposition

- 11,309–11,316: mismatch result/lifetime/projection behavior, move/move-backward ownership and iterator result positions, next-permutation/none-of/nth-element/partial-sort contracts.
- 11,317–11,323: partition/partition-copy/partition-move/partition-point, pop/push heap and previous-permutation behavior.
- 11,324–11,335: remove/remove-copy/remove-if, replace variants, reverse/reverse-copy and rotate/rotate-copy mutation/copy semantics.
- 11,336–11,338: sample/search/search_n generic algorithm behavior. These names do **not** imply the Fabushi product Search capability; they are dependency regression contracts.
- 11,339–11,366: ordered set difference/intersection/symmetric-difference/union behavior plus test-matrix fixtures.
- 11,367–11,372: shuffle, sort, sort_heap, sort_n_with_buffer, stable_partition and stable_sort.

Cross-cutting mature constraints include iterator/sentinel/category compatibility, projections, borrowed/dangling lifetime fencing, move-only ownership transfer, exact result positions, bounded and empty-range behavior, ordering/heap/partition/set invariants, stable ordering and allocation/buffer behavior.

## Fabushi disposition

No new shipping product owner is justified. Existing-owner-first maps this window to **Build / Quality / Dependency-Provenance / License** plus the native Rust/TypeScript runtime and standard-library boundaries already used by canonical Fabushi. No range-v3 runtime, C++ compatibility sidecar, second container abstraction, Search root, Conversation root, Message store or Agent runtime is introduced.

All 64 rows are `mapped-open`, `unknown_closed=false`, `omitted=false`. Source reading is not product/release closure.

Accounting after this window: **11,372/16,125 read; 4,753 unread; 15,845 unknown; 0 omitted**. First unread: 11,373 `Telegram/ThirdParty/range-v3::test/algorithm/starts_with.cpp@b8e9bbe5b0df44fcf4fb1bceac5d6f3845c292b9`.
