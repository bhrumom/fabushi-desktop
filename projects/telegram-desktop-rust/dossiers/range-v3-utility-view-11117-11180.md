# Range-v3 utility/view responsibilities — recursive orders 11,117–11,180

## Authority

- Root authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`, accepted tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6`.
- Mounted dependency: `Telegram/ThirdParty/range-v3` -> `ericniebler/range-v3@a81477931a8aa2ad025c6bda0609f38e09e4d7ec`.
- Exact-head evidence: Fabushi `e2f2ca67fad222cf5631b858a13719a7cbad29b3`; Rust desktop runtime `38143537190`; source-authority job `114482083927`; artifact `11704176427`; digest `sha256:aacf80e5efad0eda199b16cd5aa8d3112f69aca2d36d4ed77dc3324b134390d1`.
- The current-source helper materialized all 64 entries and verified each Git blob identity. No source entry was omitted.

## Responsibility decomposition

Orders 11,117–11,145 are generic utility/value/lifetime infrastructure: concepts and copy/move forwarding; counted/dangling iterator boundaries; functional/get/in-place/infinity/invoke helpers; iterator traits; memory construction/destruction utilities; optional/value presence; checked polymorphic cast; random engines/distributions; scope-exit cleanup; semiregular storage; swap/tagged tuple/pair algorithms; unreachable/variant/version compatibility.

Orders 11,146–11,180 enter lazy view/adaptor infrastructure: the aggregate view surface and adaptor/facade mechanics; address/adjacent/all/any-view/c-string/cache boundaries; cartesian/chunk/chunk-by/common/concat/const/counted/cycle/delimit/drop variants; empty/enumerate/exclusive-scan/filter/for-each/generate/getlines/group-by/indices/indirect semantics. The applicable behavior is iterator/category propagation, borrowed/dangling lifetime safety, lazy evaluation, predicate/generator ownership, caching, bounds, composition and cleanup.

## Existing-owner-first decision

These files do not establish a Fabushi Search, Conversation, Agent, Message, Marketplace or product-domain owner. Their applicable responsibilities stay with existing Build / Quality / Dependency-Provenance / License authority and the native Rust/TypeScript standard-library/runtime boundaries used by shipping Fabushi code.

No `range-v3` runtime, C++ compatibility sidecar, second container abstraction, Telegram-derived product root or source-named owner is introduced. Each source row is therefore `mapped-open`: reading/decomposition is complete, while global production/evidence closure is not.

## Accounting

- deterministic read-through: **11,180 / 16,125**
- unread: **4,945**
- unknown: **15,845**
- omitted: **0**
- first unread: **11,181** `Telegram/ThirdParty/range-v3::include/range/v3/view/interface.hpp@aee727d5ee7d9133bf6e269c0af2143cd09a9c41`

The source-disposition shard is `projects/telegram-desktop-rust/inventory/source-dispositions/11117-11180.json`.
