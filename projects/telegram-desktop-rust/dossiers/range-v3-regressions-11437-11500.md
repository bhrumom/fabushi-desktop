# Range-v3 view and package regressions — recursive orders 11,437–11,500

## Authority

- Telegram Desktop accepted baseline: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`, tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6`.
- Recursive source component: `ericniebler/range-v3@a81477931a8aa2ad025c6bda0609f38e09e4d7ec`.
- Exact-head Fabushi evidence: `2e8c169214a72abb1a3f16d52f2a811e8f991b2e`.
- GitHub Actions: Rust desktop runtime `38156139933`, source-authority job `114519461684`.
- Artifact: `11708199027`, digest `sha256:cd9fcd8e65b428f25271dbf5957647e88e58084e3acb8c89847b0051d9f7b77f`.
- Window: recursive orders 11,437–11,500, 64/64 materialized with repository/commit/path/Git-blob and byte-length verification.

## Responsibility decomposition

- 11,437–11,446: type-erased/cached/cartesian/chunk/common/concat/const/conversion/counted view contracts.
- 11,447–11,460: cyclic/delimited/drop/enumerate/scan/facade/generate/getlines/group/indirect view lifetime and iterator behavior.
- 11,461–11,476: intersperse/iota/stream/iterator-range/join/keys-values/distribution/move/partial-sum/remove/repeat/replace/reverse contracts.
- 11,477–11,498: sample/set views/slice/sliding/span/split/stride/subrange/take/tokenize/transform/trim/unique/all/zip, including bounded/infinite and borrowed-range behavior.
- 11,499–11,500: Conan/CMake package-consumption smoke wiring. The scripts compile/link/run an example and copy dynamic libraries; they do not fetch an unpinned dependency or create a shipping package-manager responsibility.

## Existing-owner-first decision

This window is development-time dependency/runtime regression infrastructure. It preserves iterator/category propagation, borrowed/dangling and prvalue/xvalue lifetime fences, cache invalidation, finite/infinite cardinality, view composition, move-only ownership, result/reference identity and package-consumption smoke coverage.

No source establishes a Fabushi Search, Conversation, Message, Agent, Resource, Plugin, Marketplace, Call, media or other product-domain owner. Existing-owner-first remains Build / Quality / Dependency-Provenance / License plus native Rust/TypeScript runtime/lifecycle boundaries. No `range-v3` shipping runtime, C++ compatibility sidecar, second container abstraction, Telegram runtime/provider or duplicate state owner is introduced.

All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; source reading does not itself close implementation or release acceptance.

## Traceability and next boundary

The shard `projects/telegram-desktop-rust/inventory/source-dispositions/11437-11500.json` preserves per-row exact source identity and owner mapping.

Accounting after this window: **11,500/16,125 read; 4,625 unread; 15,845 unknown; 0 omitted**.

First unread: **11,501** `Telegram/ThirdParty/range-v3::test_package/example.cpp@bee0b2ab825f5733e9954514c07445b944ca5259`.
