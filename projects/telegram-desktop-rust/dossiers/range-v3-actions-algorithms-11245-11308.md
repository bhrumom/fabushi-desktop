# Range-v3 action and generic algorithm regression dossier — orders 11,245–11,308

## Exact authority

- Accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`, recursive mount `ericniebler/range-v3@a81477931a8aa2ad025c6bda0609f38e09e4d7ec`.
- Evidence HEAD: `3f9004497d0d69a149171e8320f100042d514015`.
- Rust desktop runtime: `38152310164`.
- Source-authority job: `114508097974` — completed/success.
- Artifact: `11707316697`.
- Digest: `sha256:fd1d889abe5e3320cd523ea78fee6769ee3469fd27335164b5e93fe38c49be13`.
- 64/64 rows matched repository/commit/path/blob identity and declared byte length in the current-head source artifact.

## Responsibility decomposition

Orders 11,245–11,254 cover action regressions for slice/sort/split/stable-sort/stride/take/take-while/transform/unique/unstable-remove behavior. Order 11,255 is the development-only algorithm test manifest. Orders 11,256–11,308 cover generic algorithm regression contracts across adjacent search/removal, all/any predicates, binary search, copy/count, equality/range equality, fill/find/fold/for-each/generate, includes/in-place merge, heap invariants, partition/permutation/sortedness, lexicographic comparison, lower-bound and min/max result behavior.

The mature semantics are iterator/sentinel/category and projection correctness, bounded mutation/output, empty-range handling, result/reference lifetime, ordering, heap/permutation invariants and deterministic quality coverage. They are generic dependency/runtime semantics rather than Fabushi Search, Conversation, Message, Agent, Marketplace or other product-domain ownership.

## Existing-owner-first result

All 64 rows map to existing Build / Quality / Dependency-Provenance / License and native Rust/TypeScript standard-library/runtime boundaries. No range-v3 runtime, second container abstraction or source-named product owner is introduced. Rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`.

## Accounting after commit

**11,308 / 16,125 read; 4,817 unread; 15,845 unknown; 0 omitted**.

Next unread: **11,309** `Telegram/ThirdParty/range-v3::test/algorithm/mismatch.cpp@4a70c8a75afa1dabc6fe6e0628ab288919c1f753`.
