# Range-v3 actions/algorithms — deterministic review 10,925–10,988

## Exact source authority
- Accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`, tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6`.
- Nested source authority: `ericniebler/range-v3@a81477931a8aa2ad025c6bda0609f38e09e4d7ec`.
- Evidence HEAD: `8b7f722d3df5f301f816ef51344d6eb1fe1fb841`.
- GitHub Actions: Rust desktop runtime `38137121929`, source-authority job `114463165742`, artifact `11701397304`.
- Artifact digest: `sha256:b5d5766cfa0495a6d2e42ab2978af2cc92c71049b2c2770ce7e80711814e9a8b`.
- Orders: `10,925–10,988`, 64 recursive blob entries.
- Independent artifact check: 64/64 files matched declared Git blob SHA, SHA-256 and byte length; mismatches: 0.

## Responsibility decomposition
### 10,925–10,939 — generic actions
Remove/reverse/shuffle/slice/sort/split/stride/take/transform/unique actions preserve generic range/container preconditions. Slice bounds are validated, stride is positive, count-like quantities are non-negative, shuffle requires a qualifying random source, and destructive actions stay bounded by range/container concepts.

### 10,940–10,951 — aggregate/detail algorithms
The aggregate header and detail helpers cover adjacent search/removal, all/any predicates, n-limited equal/lower/upper bound, merge with explicit n/buffer contracts, partition-point and sort-with-buffer helpers.

### 10,952–10,988 — generic algorithms
Binary search/contains, copy/count/fill/find, folds, for-each/generate, heap/inplace-merge, partition/sorted/lexicographical ordering, lower-bound, min/max and merge are generic algorithm responsibilities. Binary search assumes finite input; ends_with/equal can reject by size first; range min/max require non-empty input; copy/count/generate respect explicit bounds; inplace_merge supports temporary-buffer allocation with fallback.

Current canonical production has no shipping `range-v3`, `range/v3`, `ranges::` or `ericniebler` consumer. Existing-owner-first maps all 64 entries to Build/Quality/Dependency-Provenance/License. No range-v3 runtime, second container abstraction, Search/Conversation/message owner is introduced.

## Accounting and closure
All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Reading/decomposition does not close product responsibility.

After this window: `10,988/16,125` read, `5,137` unread, `15,845` unknown, `0` omitted. First unread: `10,989 Telegram/ThirdParty/range-v3::include/range/v3/algorithm/minmax.hpp@3c5e6ab2f9789e3861a000efa269d42f688fae4c`.
