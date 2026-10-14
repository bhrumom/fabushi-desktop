# TartanLlama/expected 7,404–7,427 complete read

Status: exact-source read complete; responsibility decomposition complete; production verification open  
Project: TDRP-001 Revision 9  
Accepted root: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`  
Component: `TartanLlama/expected@292eff8bd8ee230a7df1d6a1c00c4ea0eb2f0362`  
Mount: `Telegram/ThirdParty/expected`  
Orders: 7,404–7,427 (24 non-directory entries)

## Source responsibilities

The component is a header-only typed success/error value implementation plus its build/provenance and regression suite. Orders 7,404–7,412 are build, CI, package, license and provenance inputs. Order 7,413 `include/tl/expected.hpp` carries the runtime-neutral semantic contract: success/error discrimination; constructors and assignment; move/copy/triviality; observers; emplacement and swap; void specialization; constexpr/noexcept behavior; and `map`, `map_error`, `and_then`, `or_else` composition. Orders 7,414–7,427 exercise those semantics, including move-only ownership, conversion/assignment, observer access, exception/noexcept boundaries, relational operations and swap.

Accepted tdesktop consumers include typed settlement in MTProto key creation/session paths, Story lookup, attach-bot WebView work and reaction-selector creation. Those are source evidence for the responsibility, not authorization for a Telegram/Expected runtime.

## Existing-owner-first disposition

Fabushi already has the correct source-neutral language-native result mechanisms. Rust owners use `Result`/`Option`; TypeScript owners use discriminated result/settlement unions and explicit parsing. The applicable behavior therefore belongs to existing Coordinator, Host, Runner and domain owners that already own the underlying operation, with Build/Release retaining dependency/license/provenance checks.

No `ExpectedRuntime`, Telegram runtime/provider, second error bus, second Conversation/Message store, or source-named product owner is permitted.

## Required production closure

Reading alone does not close any unknown. Before these responsibilities can be verified, affected canonical owners must demonstrate: exact success/error discrimination; no loss of original error category during forwarding; move/ownership-safe cancellation and teardown; no accidental default-success fallback; explicit handling of absent/void results; and focused regression tests for typed propagation at actual shipping entrypoints. Any remote operation must continue to satisfy its own authentication/authorization/persistence/retry/recovery contract.

## Validator defect found during acceptance

The predecessor exact HEAD `de430cbdff6fd55b7a527de7c5f7cdca93951a64` failed Revision 9 validation at order 7,288 because `Telegram/ThirdParty/cmark-gfm::bench/stats.py` was recorded with a malformed 38-character SHA in three disposition identity fields. The accepted blob is `c244b419f66624bd2d2d3cb028d1f65edabc22`. This descendant corrects the durable record; the validator remains fail-closed and unchanged.

## Accounting

- recursive total: 16,125
- read-through: 7,427
- unread: 8,698
- unknown: 15,845
- omitted: 0
- first unread: 7,428 `Telegram/ThirdParty/fcitx5-qt::.clang-format@04b3745d495e42f358142112a044b42c7600794b`

Next work starts from the pinned `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340` component. Existing-owner-first and exact-head GitHub Actions evidence remain mandatory.
