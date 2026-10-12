Live Revision 9 authority continuation (2026-10-12): telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6). Recursive non-directory=16,125; read-through=12,972; first unread=12,973 Telegram/lib_ui::ui/arc_angles.h@39c1ce22ffecb5ae8f242dea374d01c3c7b6ae3f; unread=3,153; unknown=15,845; omitted=0. The 12,909-12,972 recursive ordering was recovered from predecessor e4e455cd/38192498288/source-authority 114628073754/artifact 11722021776 only as a diagnostic after that run was cancelled by descendant pushes. Every binary 12,909-12,960 was independently refetched from accepted desktop-app/lib_ui@91ff4463899f23f4227bc4cbbdbe7696d479b9bd and matched Git blob identity, declared byte size, PNG signature and decoded dimensions; 12,961-12,972 text/code blobs were refetched and read at that exact commit. No predecessor workflow result is accepted for the current descendant; fresh same-head runtime/chat/source-authority validation remains required. Reading alone closes no unknown.

### Revision 9 deterministic read-through — lib_ui window resources, GPU compatibility, Button/a11y and animated resources 12,909–12,972

12,909–12,960 finish round/slide/title/window shadow and title-control raster families. Their pixels are presentation authority only; close/maximize/minimize/restore/menu command identity, hit-target/accessibility semantics and elevation/fade behavior map to the existing Window Chrome, canonical IconButton/Menu, Design System elevation and visual-acceptance owners. 12,961–12,962 preserve the Windows GPU vendor/device/driver compatibility denylist and deterministic Qt resource packaging as platform behavior authority, mapped to the current Electron/Chromium platform compatibility and Build/Packaging owners rather than a Qt runtime.

12,963–12,964 cover AbstractButton hover/down/release, click-button/modifier identity, Space/Return/Enter non-autorepeat activation, pressed accessibility projection and state clearing; these map into the existing canonical Button/IconButton, native HTML button semantics and command-fence owners. 12,965–12,970 cover accessibility factory projection, stable dynamic-child identity, deferred safe retirement, role/state/text/action projection, bounds-safe navigation/focus, single-selection semantics and orientation attributes; these map to native DOM/ARIA plus the existing accessibility bridge and canonical list/tab/component owners. 12,971–12,972 cover async animated-resource preparation, weak-lifetime cancellation, frame timing, density-aware resize/cache, custom start/end bounds and reduced-motion behavior under existing Resource/Design-System motion owners. No lib_ui widget, Qt accessibility tree, second Button state owner or frame runtime is introduced.

All 64 rows are exact-read and responsibility-decomposed with `unknown_closed=false` and `omitted=false`. Accounting: **12,972/16,125 read; 3,153 unread; 15,845 unknown; 0 omitted**. Next: 12,973 `Telegram/lib_ui::ui/arc_angles.h@39c1ce22ffecb5ae8f242dea374d01c3c7b6ae3f`.

Live Revision 9 authority continuation (2026-10-12): telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6). Recursive non-directory=16,125; read-through=12,908; first unread=12,909 Telegram/lib_ui::icons/round_shadow_box_left@3x.png@f52dac8931eb6ee42e01ba3a736d6de981668ffe; unread=3,217; unknown=15,845; omitted=0. Exact source evidence is d5e700722a392de1f74a338649ce124f73b21f8b / Rust desktop runtime 38189544832 / source-authority job 114619149670 / artifact 11721111016 (sha256:2b82a4517bf5c8849bb2965d0cf0ab9ea2ba6a6e16639f34a11a48a425711248). Orders 12,845-12,908 were materialized and independently checked: 64/64 byte lengths, SHA-256 values and Git blob identities matched; TTF resources were verified as real Open Sans/Vazirmatn fonts and PNG density families as valid image inputs. The production/accounting descendant requires fresh same-head runtime/chat validation; source reading alone closes no unknown.

### Revision 9 deterministic read-through — lib_ui palette/style, emoji autocomplete, fonts and visual resources 12,845–12,908

12,845–12,846 preserve deterministic palette/style generator inputs, dependency closure and generated-output provenance under Fabushi Build/Resource/Design-System ownership; no lib_ui or Qt style runtime is introduced. 12,847–12,853 cover current and historical emoji datasets as codegen/compatibility authority. 12,854–12,857 cover Composer colon-completion normalization, distinct word-prefix segmentation, duplicate collapse and stable exact/word-count/primary-code ranking. The exact-head existing-owner audit found the shipping path already runs Composer → EditorSuggestionController → canonical EmojiCatalog, so this descendant extends that owner with source-neutral `searchEmojiSuggestions` instead of creating a second autocomplete store; picker substring search remains independent. Focused behavior/wiring coverage is added to the already-executed conversation-infra contract.

12,858–12,864 are functional font-resource inputs: six Open Sans/Vazirmatn TTFs plus qrc packaging. They remain mapped-open to canonical typography/Resource/Packaging pending shipping font-fallback and license/provenance closure; upstream fonts are not copied merely because they exist. 12,865–12,908 are box/search/divider, Human Call shadow, checkbox and fade/elevation PNG density families. Their pixel assets are presentation authority only; interaction/state semantics map to canonical Fabushi Dialog/IconButton/SearchField/Checkbox/Human Call/overlay components and semantic tokens. No Telegram visual component or raster state truth is introduced.

All 64 rows are exact-read and responsibility-decomposed with `unknown_closed=false` and `omitted=false`. Accounting: **12,908/16,125 read; 3,217 unread; 15,845 unknown; 0 omitted**. Next: 12,909 `Telegram/lib_ui::icons/round_shadow_box_left@3x.png@f52dac8931eb6ee42e01ba3a736d6de981668ffe`.

### Revision 9 deterministic read-through — spellcheck completion, storage/recovery, TL/schema, local translate, lib_ui entry 12,781–12,844

12,781–12,807 complete the spell/language/highlighting window: macOS and Windows system spell adapters, asynchronous stale-safe checks, script-aware tokenization, incremental/cold highlighting, suggestion and context-menu lifecycles, bounded custom dictionary integrity/sync semantics, Hunspell fallback and CLD3 recognition. 12,808–12,832 cover canonical storage/cache responsibilities: bounded binlog parsing, corruption/truncation handling, tagged accounting, size/TTL eviction, cleaner/compactor lifecycle, wrong-key/lock distinction, legacy cleanup, database reference/close lifecycle, encrypted-file header integrity and random-access/append recovery. Cryptographic behavior maps to audited Fabushi security/storage dependencies; Telegram's encryption implementation is not a target architecture. 12,833–12,838 cover deterministic schema/codegen, bounded reader/writer and flags/vector/boxed ownership semantics without retaining TL/MTProto. 12,839–12,842 cover local translation availability/error semantics and the macOS Translation/NaturalLanguage adapter. 12,843–12,844 enter lib_ui only at build/resource codegen: palette/style/emoji generated inputs and functional emoji assets map to Fabushi Design System/Resource/Build owners, not a second UI framework.

All 64 rows remain mapped-open with unknown_closed=false and omitted=false. Accounting: **12,844/16,125 read; 3,281 unread; 15,845 unknown; 0 omitted**. Next: 12,845 `Telegram/lib_ui::cmake/generate_palette.cmake@e3f70430fc073eb1558340af9568755f150581d7`.

Live Revision 9 authority continuation (2026-10-12): telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6). Recursive non-directory=16,125; read-through=12,780; first unread=12,781 Telegram/lib_spellcheck::spellcheck/platform/mac/spellcheck_mac.h@49d672c09ca5c6cec8d24db69ad6858279a5d6b3; unread=3,345; unknown=15,845; omitted=0. Exact-source predecessor HEAD c1affdc7c11a03bd471cf290b43f7eebd9ac8006 Source authority run 38187773453/job 114614699733 succeeded and materialized artifact 11720595473 (sha256:d53c4936ad4a4f3b3dd483211f67f3ff10a4cc0c2ffcd9ccf28c0c010f7448f8); the complete 12,717-12,780 exact-blob window has now been formally reviewed in descendant shard 12717-12780.json. This accounting descendant requires fresh same-head runtime/chat validation; source reading alone closes no unknown.

### Revision 9 deterministic read-through — lib_lottie playback, QR, lib_rpl lifecycle, spellcheck entry 12,717–12,780

12,717–12,732 finish the current lib_lottie window: validated/bounded animation parsing, color replacement, prepared-frame rendering, weak-lifetime async handoff, single/multi-player monotonic scheduling, pause/unpause and lateness compensation, preload/jump behavior, and canonical toast/custom-emoji integration. 12,733–12,735 cover QR UTF-8 encoding, ECC policy, deterministic module generation, rounded rendering and safe center-logo reservation. 12,736–12,768 cover lib_rpl producer/consumer terminal semantics, nested lifetime cancellation, stale-delivery suppression, latest-inner switching, combine/merge ordering, dynamic subscription mutation, type erasure and variable dedupe/reset behavior plus regression tests. 12,769–12,780 enter spellcheck: platform/build feature selection, functional grammar-resource packaging, Linux no-native-language fallback, optional dynamically linked Enchant with bounded provider selection/script-aware suggestions/session dictionary mutations and locking, plus macOS NaturalLanguage bounded hypotheses with thread-local recognizer reset. All applicable responsibilities remain existing-owner-first in canonical Media/Resource/Scheduler/lifecycle/QR/text-service/platform/Build/Quality owners; no lib_lottie/lib_rpl/lib_spellcheck product runtime, second state truth, or Telegram-specific UI root is introduced.

All 64 rows remain mapped-open with unknown_closed=false and omitted=false. Accounting: **12,780/16,125 read; 3,345 unread; 15,845 unknown; 0 omitted**. Next: 12,781 `Telegram/lib_spellcheck::spellcheck/platform/mac/spellcheck_mac.h@49d672c09ca5c6cec8d24db69ad6858279a5d6b3`.

Live Revision 9 authority (2026-10-12): telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (root tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6). Root non-directory=6,653; recursive non-directory=16,125; deterministic read-through=11,564; first unread=11,565 Telegram/ThirdParty/tgcalls::tgcalls/TurnCustomizerImpl.h@6029f481555de3b0453fa2012da910ce5515a65c; unread=4,561; unknown=15,845; omitted=0. Orders 11,501-11,564 were formally read/decomposed from predecessor exact-head source-authority evidence 03a01ab3c231c2a2b6a53f719340bc04124939bb / Rust desktop runtime 38157144305 / source-authority job 114522352942 / artifact 11708539924 (sha256:6285b1529ca14b916d3721a485902565ea391302db65b9a96805d7809e275528). That evidence is historical-only for current-head acceptance after PR HEAD advanced; current-head authority must be re-attested by fresh descendant Actions. Applicable behavior remains mapped existing-owner-first to Build/Quality/Dependency-Provenance/License and canonical CallSession/Conversation/Permissions/Resource/WebRTC/media/network owners; no tgcalls C++ runtime, Docker chain, Swift package, TelegramCallRuntime, or second call store is introduced.
<!-- TDRP_CURRENT_SUMMARY read-through=12972 unread=3153 unknown=15845 omitted=0 first-unread=12973 path=Telegram/lib_ui::ui/arc_angles.h -->


### Revision 9 deterministic read-through — range-v3 view/package regressions 11,437–11,500

Exact source-window evidence: `2e8c169214a72abb1a3f16d52f2a811e8f991b2e`, Rust desktop runtime `38156139933` (completed/success), source-authority job `114519461684` (completed/success), artifact `11708199027`, digest `sha256:cd9fcd8e65b428f25271dbf5957647e88e58084e3acb8c89847b0051d9f7b77f`. The current-head helper materialized all 64 entries and verified repository/commit/path/Git-blob identity plus declared byte length.

Orders 11,437–11,498 cover lazy view/adaptor regressions across type erasure, caching, cartesian/chunk/common/concat/const/conversion/counted/cycle, drop/take/slice/stride/split/zip, generator/scan/join/group/set views, span/subrange and related iterator/category, borrowed/dangling, finite/infinite, invalidation and move-only ownership behavior. Orders 11,499–11,500 are Conan/CMake package-consumption smoke wiring; they compile/link/run an example and copy runtime libraries, but contain no new mutable acquisition path.

No Search, Conversation, Message, Agent, Resource, Plugin, Marketplace, Call/media or other shipping product-domain owner is justified. Existing-owner-first remains Build / Quality / Dependency-Provenance / License plus native Rust/TypeScript runtime/lifecycle and standard-library boundaries. All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`.

Accounting: **11,500/16,125 read; 4,625 unread; 15,845 unknown; 0 omitted**. Next: 11,501 `Telegram/ThirdParty/range-v3::test_package/example.cpp@bee0b2ab825f5733e9954514c07445b944ca5259`.


### Revision 9 deterministic read-through — range-v3 regression infrastructure 11,373–11,436

Exact source-window evidence: `d95fea5e0d948bec73a043ae31cc2b3584c86775`, Rust desktop runtime `38154729489` (completed/success), source-authority job `114515308597` (completed/success), artifact `11708216622`, digest `sha256:310c3dec866bd7d6589b6a17cb41d7d1118b65968dc0462d48bc538194403315`. The current-head helper materialized all 64 entries and verified repository/commit/path/blob identity plus declared byte length; the same exact-head rust-host executed Host Runner, prompt attachment projection, Human Computer takeover, and ConversationActor/CapabilityBroker ownership successfully.

Orders 11,373–11,379 cover generic prefix/order/mutation algorithms: starts-with, swap-ranges, transform, unique/unique-copy, unstable-remove-if and upper-bound. Orders 11,380–11,388 cover test fixtures, compiler regressions, constexpr iterator operations and debug-view invalidation/lifetime fencing. Orders 11,389–11,396 cover experimental generator/shared-view ownership plus callable bind-back/composition semantics. Orders 11,397–11,404 cover iterator category/proxy/reference/arrow behavior, common/counted/reverse/move/stream iterators, sentinels and difference arithmetic. Orders 11,405–11,412 cover compile integration and numeric accumulate/adjacent-difference/inner-product/iota/partial-sum boundaries. Orders 11,413–11,420 cover range access/conversion/indexing, bounds failure, finite/infinite distance and test-support contracts. Orders 11,421–11,431 cover box/common-type/comparison/concepts/invoke/meta/scope-exit/semiregular/swap/variant type, exception and lifetime semantics. Orders 11,432–11,436 cover the view manifest plus adaptor/addressability/adjacent filtering/all-view category and reference-ownership constraints.

These files preserve mature iterator/sentinel/category/projection rules, borrowed/dangling and prvalue/xvalue lifetime fencing, move-only ownership, bounded output/result positions, invalidation, exception cleanup, callable cv/ref qualification and finite/infinite range behavior. They do not justify Search, Conversation, Message, Agent or another shipping Fabushi product-domain owner. Existing-owner-first remains Build / Quality / Dependency-Provenance / License plus native Rust/TypeScript runtime/lifecycle and standard-library boundaries. All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`.

Accounting: **11,436/16,125 read; 4,689 unread; 15,845 unknown; 0 omitted**. Next: 11,437 `Telegram/ThirdParty/range-v3::test/view/any_view.cpp@c69c43b9957c2c89d05436910845fcb19d931f8f`.

### Revision 9 deterministic read-through — range-v3 algorithm regressions 11,309–11,372

Exact source-window evidence: `2ab515facff678df69eb53834f37ae1901cbe47d`, Rust desktop runtime `38153128885` (completed/success), source-authority job `114510518094` (completed/success), artifact `11707477754`, digest `sha256:2a7a3754612ea3d53070e19f4df3c562ed3341c1dc447d5142d20913b759cb63`. The current-head helper materialized all 64 entries and verified repository/commit/path/blob identity plus declared byte length; the same exact-head rust-host also executed Host Runner, prompt attachment projection, Human Computer takeover, and ConversationActor/CapabilityBroker ownership successfully.

Orders 11,309–11,316 cover mismatch/move ownership and result positions, permutations, predicates and nth/partial sorting. Orders 11,317–11,323 cover partition and heap mutation/search boundaries. Orders 11,324–11,335 cover remove/replace/reverse/rotate mutation and copy semantics. Orders 11,336–11,338 cover generic sampling/search algorithms; these are dependency regression tests and do not establish the Fabushi product Search owner. Orders 11,339–11,366 cover ordered set difference/intersection/symmetric-difference/union contracts and fixture matrices. Orders 11,367–11,372 cover shuffle/sort/sort-heap/buffered sort/stable partition/stable sort.

These sources preserve iterator/sentinel/projection/category constraints, move-only ownership transfer, borrowed/dangling lifetime behavior, result-position correctness, bounded/empty-range handling, heap/partition/set ordering invariants and stable-sort semantics. They do not justify Search, Conversation, Message, Agent or another shipping product-domain owner. Existing-owner-first remains Build / Quality / Dependency-Provenance / License plus native Rust/TypeScript runtime/standard-library boundaries. All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`.

Accounting: **11,372/16,125 read; 4,753 unread; 15,845 unknown; 0 omitted**. Next: 11,373 `Telegram/ThirdParty/range-v3::test/algorithm/starts_with.cpp@b8e9bbe5b0df44fcf4fb1bceac5d6f3845c292b9`.


### Revision 9 deterministic read-through — range-v3 action/algorithm regressions 11,245–11,308

Exact source-window evidence: `3f9004497d0d69a149171e8320f100042d514015`, Rust desktop runtime `38152310164`, source-authority job `114508097974` (completed/success), artifact `11707316697`, digest `sha256:fd1d889abe5e3320cd523ea78fee6769ee3469fd27335164b5e93fe38c49be13`. The current-head helper materialized all 64 entries and verified repository/commit/path/blob identity plus declared byte length.

Orders 11,245–11,254 are range-v3 action regression contracts for slice/sort/split/stable-sort/stride/take/take-while/transform/unique/unstable-remove behavior. Order 11,255 is the development-only algorithm test manifest. Orders 11,256–11,308 cover generic algorithm regression semantics across adjacency, predicates, binary search, copy/count/equality, fill/find/fold/iteration/generation, includes/merge, heap invariants, partition/permutation/sortedness, lexicographic ordering, lower bound and min/max results.

These sources preserve mature iterator/sentinel/projection/lifetime constraints, bounded mutation/output, empty-range behavior, reference/result lifetime and ordering invariants. They do not establish Search, Conversation, Message, Agent or other shipping Fabushi product-domain owners. Existing-owner-first remains Build / Quality / Dependency-Provenance / License plus native Rust/TypeScript runtime/standard-library boundaries. All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`.

Accounting: **11,308/16,125 read; 4,817 unread; 15,845 unknown; 0 omitted**. Next: 11,309 `Telegram/ThirdParty/range-v3::test/algorithm/mismatch.cpp@4a70c8a75afa1dabc6fe6e0628ab288919c1f753`.



### Revision 9 deterministic read-through — range-v3 views/build provenance/actions 11,181–11,244

Exact source-window evidence: `af7183eb431ee41730ba0631f2306c186478890d`, Rust desktop runtime `38151143307` (completed/success), source-authority job `114504686124`, artifact `11706761066`, digest `sha256:09b595119048168b7fb4f857d8ce3e642e25c46163de2ff93a2eea6a778c9d32`. The exact-head helper materialized all 64 entries and verified each exact Git blob identity before upload.

Orders 11,181–11,223 cover generic range-v3 view/adaptor semantics: iterator/category propagation, lazy ownership and dangling/lifetime fencing, finite/infinite cardinality, caching, bounds, predicate/generator lifetime, split/join/slice/stride/transform/zip and related non-owning traversal behavior. Orders 11,224–11,225 are C++ standard iterator/associated-type compiler compatibility shims. Orders 11,227–11,230 are development-only performance benchmarks. Orders 11,231–11,244 are action regression contracts for mutation/reference identity, bounded container operations, remove/reverse/shuffle and related quality behavior.

Order 11,226 `Telegram/ThirdParty/range-v3::install_libcxx.sh@dcbf38c1c3b06c3b6baf9e81b43d29d91376b51b` is a recursive build-time acquisition finding. Lines 51–53 shallow-clone mutable tips of `llvm-mirror/llvm`, `llvm-mirror/libcxx` and `llvm-mirror/libcxxabi` without commit pins. The alternate path constructs release URLs and lines 67–69 use `curl -LO` without an in-script integrity digest. Fabushi records this under the existing Build / Quality / Dependency-Provenance / License owner and keeps shipping fail-closed; the acquisition path is not imported.

No Search, Conversation, Message, Agent or other product-domain owner is justified by this window. All 64 rows remain `mapped-open` / `unknown_closed=false` / `omitted=false`; source reading alone does not close product/release responsibility.

Accounting: **11,244/16,125 read; 4,881 unread; 15,845 unknown; 0 omitted**. Next: 11,245 `Telegram/ThirdParty/range-v3::test/action/slice.cpp@ff01398e6d72622e3c105dcf3bc9a23380f8c7bc`.



### Revision 9 deterministic read-through — range-v3 utility/view infrastructure 11,117–11,180

Exact source-window evidence: `e2f2ca67fad222cf5631b858a13719a7cbad29b3`, Rust desktop runtime `38143537190`, source-authority job `114482083927`, artifact `11704176427`, digest `sha256:aacf80e5efad0eda199b16cd5aa8d3112f69aca2d36d4ed77dc3324b134390d1`. The exact-head helper materialized all 64 entries and verified each exact Git blob SHA before artifact upload.

This window covers utility concept/copy/iterator/dangling/invoke/memory/move/optional/random/scope-exit/semiregular/swap/tagged tuple/variant boundaries plus lazy view/adaptor lifecycle and category constraints for all/any_view/cache/cartesian/chunk/common/concat/counted/cycle/drop/filter/generate/group_by/indices/indirect. These are generic C++ dependency/runtime semantics, not Fabushi Search/Conversation/Agent product ownership.

Existing-owner-first keeps applicability at Build / Quality / Dependency-Provenance / License and native Rust/TypeScript runtime/standard-library owners. No range-v3 shipping runtime, second container abstraction or source-named product root is introduced. All 64 rows remain `mapped-open` pending applicable production/evidence closure.

Accounting: **11,180/16,125 read; 4,945 unread; 15,845 unknown; 0 omitted**. Next: 11,181 `Telegram/ThirdParty/range-v3::include/range/v3/view/interface.hpp@aee727d5ee7d9133bf6e269c0af2143cd09a9c41`.




### Revision 9 deterministic read-through — range-v3 functional/iterator/range 11,053–11,116

Exact source-window evidence: `d59eea1e646d01d53bc59aa41b422020239e9faa`, Rust desktop runtime `38142444139`, source-authority job `114478888136`, artifact `11703104785`, digest `sha256:f9eeaba39ea570cd44f22b38c635c0991983eea09a8e4f9e1fb298a91f612a5f`. The exact-head helper materialized the 64-entry window and verifies each exact Git blob SHA.

This window covers callable bind/invoke/pipeable/reference lifetime; iterator/sentinel/category/read-write/move/advance/distance; numeric fold/difference/product/iota/partial-sum; range begin/end/size/data, borrowed/dangling lifetime fencing and conversion; plus utility type-erasure/storage/common tuple/type helpers and deprecated forwarding headers. Mature responsibilities include cv/ref-qualified invocation, fail-fast preconditions, bounded output, borrowed iterator result types, dangling prevention and compatibility deprecation boundaries.

Existing-owner-first keeps these responsibilities in Build / Quality / Dependency-Provenance / License and language runtime owners. No shipping range-v3 consumer or product-domain owner is introduced. All rows remain `mapped-open`.

Accounting: **11,116/16,125 read; 5,009 unread; 15,845 unknown; 0 omitted**. Next: 11,117 `Telegram/ThirdParty/range-v3::include/range/v3/utility/concepts.hpp@b8a1c91fc37b8dffa08b465722f8ec7baab46656`.
### Revision 9 deterministic read-through — range-v3 algorithms/access infrastructure 10,989–11,052

Exact source-window evidence: `a790336594b1eea34816c20d051c695514484929`, Rust desktop runtime `38141647544`, source-authority job `114476566497`, artifact `11702534129`, digest `sha256:f5c8b708bc668829d02ad166e73c27913ae880acf342fbd57e5cd75712274edf`. The exact-head Actions helper materialized the 64-entry window from pinned `ericniebler/range-v3@a81477931a8aa2ad025c6bda0609f38e09e4d7ec`, verifying Git blob identity before upload.

Orders 10,989–11,030 cover generic algorithm preconditions/category/callable contracts, empty/non-empty result semantics, bounded mutation/copy, ordering/search behavior, shuffle length safety and stable allocation/fallback behavior. Orders 11,031–11,052 cover public accessors/facades, compiler/config/range-access plumbing, fail-fast assertions, variant/lifetime compatibility, distance/empty/front, generator/shared-view and functional arithmetic.

No shipping Fabushi `range-v3` consumer was found. Existing-owner-first keeps these responsibilities in Build / Quality / Dependency-Provenance / License and language runtime owners. All 64 rows remain `mapped-open`; source reading does not close product or release verification.

Accounting: **11,052/16,125 read; 5,073 unread; 15,845 unknown; 0 omitted**. Next: 11,053 `Telegram/ThirdParty/range-v3::include/range/v3/functional/bind.hpp@b5b20a88347a5546965c9edc17dee397bbac8d00`.
### Revision 9 deterministic read-through — range-v3 actions/algorithms 10,925–10,988

Exact source-window evidence: `8b7f722d3df5f301f816ef51344d6eb1fe1fb841`, Rust desktop runtime `38137121929`, source-authority job `114463165742`, artifact `11701397304`, digest `sha256:b5d5766cfa0495a6d2e42ab2978af2cc92c71049b2c2770ce7e80711814e9a8b`. The staged shard records all 64 exact repository/commit/path/blob identities and the independently verified window has 64/64 matches.

Orders 10,925–10,938 complete generic mutating actions including remove/remove_if, reverse, shuffle, slice, sort, split, stable_sort, stride, take, transform and unique. Orders 10,939–10,988 continue range-v3 algorithm contracts including adjacency, binary search, copy/count, ends_with/equal, find/fold, generate, heap operations, inplace_merge, sorted/lower-bound/min/max/merge families and related generic helpers. Important preconditions remain source-neutral quality contracts: bounded slice indices, positive stride, non-negative take/count, valid random generators, binary search on finite ranges, length rejection before ends_with comparison, non-empty range requirements for min/max value extraction, and deterministic temporary-buffer fallback for adaptive algorithms.

Current Fabushi production search has no shipping `range-v3`, `range/v3`, `ranges::` or `ericniebler` consumer. Existing-owner-first therefore keeps these responsibilities with Build / Quality / Dependency Provenance / License and the language standard-library owners used by actual product code. No Telegram/range-v3 runtime, second container abstraction or product-domain owner is justified. All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`: reading does not close product verification.

Accounting: **10,988/16,125 read; 5,137 unread; 15,845 unknown; 0 omitted**. Next: 10,989 `Telegram/ThirdParty/range-v3::include/range/v3/algorithm/minmax.hpp@3c5e6ab2f9789e3861a000efa269d42f688fae4c`.

### Revision 9 deterministic read-through — range-v3 build/docs/examples/concepts/actions 10,861–10,924

Exact identities/bytes: `5b103b8b96c312e0be16c66cfaa5c0fda8e44463`, run `38135519400`, source-authority job `114458511752`, artifact `11700608450`, digest `sha256:11cb9a9adbfa53c7d367e2db4af0d443f5dd01a94eec628f376b89ad69e23215`. Independent verification matched repository/commit/path/object identity for all 64 entries; 63 blob entries independently matched Git blob SHA, SHA-256 and byte length, while order 10,873 `doc/gh-pages` is an exact gitlink to `2dae74bb693e42d850fb0adcc9045c5b71fbdeae` and therefore correctly has no blob-byte claim.

Order 10,861 is the GoogleTest configure-time bootstrap: it configures and builds an external GoogleTest project and adds it to the build. Combined with the already-read `GoogleTest.cmake.in` `GIT_TAG master`, this is an explicit floating build-time acquisition risk. Fabushi's dependency/provenance gates must fail closed on mutable refs; this dependency is not imported.

Orders 10,862-10,868 cover imported CMake targets, compiler/platform detection, diagnostics, C++ standard handling, sanitizer/modules/native/Polly options, aligned-new/coroutine/concepts probes and Conan packaging. Orders 10,869-10,883 are documentation tooling and standards/release material: Doxygen preprocessing, generated gh-pages clean/copy/update, Pandoc helpers, release notes and the exact docs-only gh-pages gitlink. The docs error-filter helper is evidence about upstream documentation generation only and does not weaken Fabushi shipping gates.

Orders 10,884-10,906 are Buck/CMake examples and test/golden-output fixtures for generic range algorithms, views and actions; the calendar example conditionally finds Boost rather than defining a Fabushi service/runtime dependency. Orders 10,907-10,913 implement generic C++ concepts/meta/compiler-compatibility and Clang module-map behavior. Orders 10,914-10,924 implement range-v3's generic mutating action layer: action closures/piping, container/reservable constraints, bounded reserve/max-size checks and adjacent-remove/drop/drop-while/erase/insert/join/push-back/push-front operations.

Current canonical production search found no `range-v3`, `range/v3`, `ranges::` or `ericniebler` consumer. These files therefore map to the existing Build/Quality/Dependency-Provenance/License owners and do not justify a new runtime, container abstraction, or product owner. All 64 rows remain `mapped-open` / `unknown_closed=false` / `omitted=false`; source reading alone does not close product responsibility.

Accounting: **10,924/16,125 read; 5,201 unread; 15,845 unknown; 0 omitted**. Next: 10,925 `Telegram/ThirdParty/range-v3::include/range/v3/action/remove.hpp@096a94250d112afae97d733d73e9049a9aa917a1`.


### Revision 9 deterministic read-through — Nimf preedit/Wayland/XIM, distro patches and range-v3 build provenance 10,797–10,860

Exact identities/bytes: `ab7fc412ff80f5858378e781c5b7039d22bb976e`, run `38134640557`, source-authority job `114455966489`, artifact `11700566936`, digest `sha256:c474a4a2ea18b54068f030e52a3b40ebf765f942bc70cb98d384d627ef17ef45`. Independent artifact verification recomputed Git blob SHA, SHA-256 and byte length for all 64 entries with 64/64 matches.

Orders 10,797-10,798 carry preedit popup lifecycle/text/cursor/monitor-bounded placement. Orders 10,799-10,803 carry Wayland display prepare/read/flush/dispatch fail-closed behavior, input-method activation/deactivation/context serials, XKB modifiers, unconsumed key forwarding and surrounding/reset/preedit/commit. Orders 10,804-10,824 carry XIM/IMdkit framing and lifecycle: byte order, length/padding, open/close, create/destroy IC, attributes, focus, forward/sync, commit/reset, preedit/status callbacks and disconnect cleanup. These map to existing PromptRichTextEditor/input-composition-state/platform-input, canonical IPC and Electron lifecycle/recovery/security; Chromium/OS stays the concrete IME owner.

Orders 10,825-10,830 are packaging, historical distro patches and a historical Qt5 input-context binary. The RPM spec itself names a mutable `master.tar.gz` source; the two patches alter broad libnimf/client/engine/indicator/config behavior and are treated as compatibility evidence rather than copied implementation. The binary `libqt5im-nimf.so` was verified from the Actions artifact as a 39,848-byte x86-64 ELF, Git blob `250bb0e74acfb123a4f2f441b6bcdc1f579765cf`, SHA-256 `e4b22ebc4af772f91f5385ddaf0ef9b00ff6f2898810c94e8f3bcc9202284c1f`; it remains evidence-only, with no source-level claims and no production dependency.

Orders 10,831-10,835 are localization/catalogue inputs. Orders 10,836-10,860 begin pinned `ericniebler/range-v3@a81477931a8aa2ad025c6bda0609f38e09e4d7ec` and cover CI/build/package/license/configuration plus compiler/header probes. `.gitmodules` carries a mutable `gh-pages` branch hint, while accepted TDesktop CI documents that nested range-v3 gh-pages is documentation-only. More importantly, `GoogleBenchmark.cmake.in` and `GoogleTest.cmake.in` use `GIT_TAG master`, and `gbenchmark.cmake` executes the configure-time acquisition. Those refs are recorded as fail-closed provenance/build-time-acquisition responsibilities and must never be introduced into Fabushi as floating dependencies.

All 64 rows remain `mapped-open` / `unknown_closed=false` / `omitted=false`; reading source does not close product responsibility. Accounting: **10,860/16,125 read; 5,265 unread; 15,845 unknown; 0 omitted**. Next: 10,861 `Telegram/ThirdParty/range-v3::cmake/gtest.cmake@a989aec90627c8db83d71012a749e26e64f8e5e0`.

### Revision 9 deterministic read-through — LZ4 manuals / examples / core / first OSS-Fuzz 10,349–10,412

Exact identities and bytes come from exact-head `99e027714e9ee707e544073c6e9240fa0c77c289` run `38127755807`, source-authority job `114435611847`, artifact `11697459714` (digest `sha256:c53bc4284dd3e358aa891df11d89083625fa56c2092d4f4b8ab1a509513a4953`). The range covers generated API-manual tooling, Meson and Snap packaging, block/frame format contracts, reference streaming/dictionary/file examples, library build/DLL/pkg-config/license inputs, the block/HC/frame/file/xxHash implementation surfaces, and the first OSS-Fuzz frame target.

Applicable source-neutral behavior is explicit: decoders require exact compressed/output bounds and reject malformed input; frame state carries version/block mode/content size/dictionary/checksum semantics; streaming callers own retained history and context cleanup; allocation, overflow and block-size limits are fail-closed; file wrappers have open/read-write/close error lifecycles; xxHash is corruption detection rather than cryptographic authentication; fuzz coverage deliberately exercises undersized output buffers and exact round trips. Existing-owner-first maps these to canonical Resource/Storage, restore/archive, Build/Quality, Packaging/Updater and License owners.

Current shipping Host already declares `flate2` + `tar`, and current Box/resource paths enforce path containment, byte/hash/size checks, disk-space/resource limits and deterministic cleanup. No current LZ4/Zstd dependency or LZ4-specific shipping consumer was proven by this range. Therefore this tranche adds no compression runtime or second archive owner. All 64 rows remain `mapped-open` / `unknown_closed=false` until later LZ4 consumer reachability and product/release evidence determine applicability.

Accounting is **10,412/16,125 read; 5,713 unread; 15,845 unknown; 0 omitted**. Next is 10,413 `Telegram/ThirdParty/lz4::ossfuzz/compress_fuzzer.c@edc8aad9a929dc7d384de191a812e83b632f22d8`.


### Revision 9 deterministic read-through — LZ4 build / quality / packaging 10,285–10,348

Exact bytes come from `c95e04afee5e8fa712c5329f6331c9d945668345` run `38127286071`, source-authority job `114434237346`, artifact `11697319116` (digest `sha256:45e8495d4f0f015a97191be79bc98247423d00797a42aa6b05a532b7cc0c5314`). The range covers LZ4 CI/toolchain matrices, Windows VS2010/2017/2022 build/test/package definitions, CMake export/install contracts, Debian packaging and legacy DJGPP portability/license inputs.

Existing-owner-first keeps Fabushi's current Build/Quality/Platform/Packaging/Updater/License owners. Current Fabushi has no LZ4/Zstd production dependency; accepted tdesktop reachability seen so far is the direct gitlink plus Snap `liblz4-dev` build package, with no direct Telegram source/CMake consumer found in this tranche. Therefore no LZ4/Telegram runtime or second compression owner is introduced. Historical upstream allow-failure cases are retained only as risk signals and do not weaken Fabushi gates. All 64 rows stay `mapped-open` / `unknown_closed=false` until later LZ4 core source and consumer reachability establish applicability.

Accounting is **10,348/16,125 read; 5,777 unread; 15,845 unknown; 0 omitted**. Next is 10,349 `Telegram/ThirdParty/lz4::contrib/gen_manual/.gitignore@6ea967f7c148b0a5e27b39ee35afaa509c588b33`.


### Revision 9 deterministic read-through — libfido2 tail / libprisma / LZ4 entry 10,221–10,284

Exact bytes come from `f810ac1eb2edba61afd6d2f89141d439b3f6e407` run `38124618782`, source-authority job `114426354062`, artifact `11696915688` (digest `sha256:306a7639eb06e2851ca640b6d6bc48ef5d552d7f194541c1263a42c72e041b9e`). The range finishes libfido2's reset/U2F/TPM/WinHello/security-key-management/platform packaging responsibilities, reads all of libprisma's Prism-derived grammar/tokenization implementation and generated grammar database, and begins LZ4's cross-platform quality matrix.

Existing-owner-first found one concrete desktop product gap: Rust/TUI already has bounded `syntect/two_face`, but the shipping React Conversation Transcript rendered fenced code as raw text. The descendant keeps Transcript as the only desktop owner and uses pinned PrismJS 1.30.0 (MIT, zero dependencies) as an offline tokenizer dependency. Token output is rendered as React text/span nodes rather than HTML, streaming content and unknown/over-budget inputs fall back to plain text, and the existing 512 KiB/10,000-line bound is preserved. Existing Focused Electron acceptance is extended to prove TypeScript keyword/number tokens on the shipping assistant transcript. The libprisma rows nevertheless remain `mapped-open`: its upstream language catalogue exceeds the initial common grammar set and exact-head descendant evidence is still required.

Accounting is **10,284/16,125 read; 5,841 unread; 15,845 unknown; 0 omitted**. Next is 10,285 `Telegram/ThirdParty/lz4::.circleci/images/primary/Dockerfile@77670148ccfef16d6490070911b62e2a8ff4ae8a`.

### Revision 9 deterministic read-through — libfido2 core/transports 10,157–10,220

Exact identities and materialized blobs come from exact-head `8cd23e6ec07d69b186be674947ac45ce38f3453a` Rust runtime run `38123961987`, source-authority job `114424363470`, artifact `11695949458` (digest `sha256:fa16db976f759301292bf23dd454bed9f6e70ad386c772333bfbb93c0b329291`). Orders 10,157–10,220 cover ES384/RS256 regressions, mock authenticator transport, build/export composition, assertion and credential ceremonies, PIN/UV and device timeout/cancel state, CBOR, biometric/configuration/credential management, HID across supported desktop platforms, getInfo capability discovery, ISO7816/NFC/PCSC, largeBlob, Linux device discovery, diagnostics and OS CSPRNG behavior.

Existing-owner-first keeps the current Coordinator WebAuthn/security runtime plus Host bridge, canonical Settings/platform-device, Secrets/Security/Crypto, Platform/Resource, Telemetry/Diagnostics and Build/Packaging/Release owners. No Telegram/libfido2 runtime, second credential store, second Settings owner or custom cryptography is introduced. The source review exposed one concrete lifecycle gap in the existing spawned signer: explicit cancellation was supported, but a helper could remain alive indefinitely when no remote cancel arrived. This descendant derives the RP timeout hint from the frozen ceremony options, clamps it to the WebAuthn client 300,000–600,000 ms range with a 300,000 ms default, and kills/reaps a stuck helper at the deadline. Focused Rust tests cover timeout extraction/clamping and a synthetic stuck Unix child.

All 64 source rows remain `mapped-open` / `unknown_closed=false`: source reading and one production hardening do not close packaged helper provenance, real HID/NFC/PCSC hardware, resident-credential/biometric/largeBlob product policy or release evidence. Accounting is **10,220/16,125 read; 5,905 unread; 15,845 unknown; 0 omitted**. Next is 10,221 `Telegram/ThirdParty/libfido2::src/reset.c@4e09dbbca5cf968ddb9d728dcc13da88cdfd9264`.

### Revision 9 deterministic read-through — libfido2 API/compat/regression 10,093–10,156

Exact identities and materialized blobs come from predecessor exact-head `b8a52c954b9ea63e9f64b31d68fc537e71e1f2bf` Rust runtime run `38120045447`, source-authority job `114412753321`, artifact `11695420570` (digest `sha256:f08d296699a1041550c1fd1421f78b47c04f53f5063eab901369157f37275ab5`). Orders 10,093–10,156 cover libfido2 assertion/credential/key APIs, RP/client-data/authdata/signature verification, UP/UV/resident credential/HMAC/largeBlob behavior, PUAT/device timeout/cancel/touch/PIN state, biometric/credential management, OpenBSD portability/security primitives and assertion/credential/device/EdDSA/ES256 regression suites.

Existing-owner-first keeps the current Coordinator WebAuthn/security runtime plus Host bridge, canonical Settings/platform-device/secure-storage/Secrets and Build/Quality/Release. No Telegram/libfido2 side runtime, second identity store or custom cryptography is introduced. The source review exposed an applicable secret-lifetime gap in the existing helper path: PIN strings and their serialized helper request bytes were ordinary buffers. This descendant moves the PIN out of the control reply and uses pinned `zeroize 1.9.0` with `Zeroizing<String>` and `Zeroizing<Vec<u8>>` until the helper write completes. Packaged `sand-webauthn-signer` producer/co-staging/provenance and wider hardware-management evidence remain open; all rows in this tranche stay `mapped-open` / `unknown_closed=false` pending full production/release closure.

Accounting is **10,156/16,125 read; 5,969 unread; 15,845 unknown; 0 omitted**. Next is 10,157 `Telegram/ThirdParty/libfido2::regress/es384.c@b55ce015e17a2d58728fef19dd945017f05359ab`.


### Revision 9 deterministic read-through — libfido2 ceremony/device/fuzz/man 10,029–10,092

Exact identities and exact blob materialization come from `53166126816c059a99298b28d4b3d8c2ca204910` Rust runtime run `38119139308`, source-authority job `114410128913`, artifact `11693708963` (digest `sha256:29a6feb40f3cbaa28f932d822bab4242821378bba701bdfbba1fb18f26a35542`). Orders 10,029–10,092 cover macOS/OpenSSL3/Windows/ShellCheck build matrices; libfido2 feature/dependency hardening; assertion/credential verification; PIN/UV, resident credentials, credProtect, enterprise attestation, largeBlob, biometric enrollment, device enumeration/touch selection/reset; secure serialization; fuzz/sanitizer/fault-injection and FIDO2/U2F corpora; coverage evidence; and API/man/key-lifetime conformance.

Existing-owner-first keeps the current Coordinator WebAuthn/security owner plus canonical Settings/platform-device/secure-storage and Build/Quality/Release owners. No Telegram/libfido2 runtime, second Identity/Credential store, or custom cryptography is introduced. Comparing the upstream fail-closed semantics exposed a production defect in the existing spawned helper path: a non-zero helper exit could still be accepted when stdout said `ok:true`, and a success result could omit a usable credential object. Staging commit `6495e0ad637075c499d49ffd8d05c18f59987195` now rejects those cases and adds focused unit tests. The broader device-management, helper packaging/co-staging, adversarial security and release responsibilities remain `mapped-open` until exact-head production evidence exists.

Accounting is **10,092/16,125 read; 6,033 unread; 15,845 unknown; 0 omitted**. Next is 10,093 `Telegram/ThirdParty/libfido2::man/es384_pk_new.3@e865913b780700f95fe8f16c406e5b83065cbdd1`.


### Revision 9 deterministic read-through — libcbor tail + libfido2 start 9,965–10,028

Exact-head `f936a5e…` source range resolution and attestation succeeded in run `38118212864` / job `114407396444`, artifact `11693652953` (digest `sha256:e9307971683ac2f1920de0aed9d9cd9f9be0a4dbfd0f2d31910459cd80878970`). Revision 9 validation then failed solely because `inventory/index.json.note` did not use the strict validator-required `unread_minimum is … / unknown remains … / omitted remains …` narrative. This descendant restores that contract; the validator is not weakened.

Orders 9,965–10,005 close the libcbor component with adversarial tests for malformed/incomplete/memory/syntax separation, exact streaming required bytes, SIZE_MAX arithmetic/reallocation overflow, nesting limits, UTF-8 rejection, allocation-failure cleanup and reference-count stability. Orders 10,006–10,028 begin `Yubico/libfido2@b974e7c…` and establish cross-platform Debug/Release regression, scan-build, ShellCheck, CodeQL, ASan/MSan/OSS-Fuzz/CIFuzz and pinned build-dependency provenance. All map existing-owner-first to canonical WebAuthn/Account Auth security and Build/Quality/Release; no second FIDO/CBOR runtime is introduced.

At `f936a5e…`, Focused Electron chat E2E, renderer/platform-worker, all three updater jobs and all three pressure-profiler jobs succeeded; the independent Coordinator contract also passed with bounded WebAuthn helper-output hardening present. The same-head source-authority job is not final-green because of the narrative contract failure, so fresh descendant exact-head evidence remains required.

Accounting is **10,028/16,125 read; 6,097 unread; 15,845 unknown; 0 omitted**. Next is 10,029 `Telegram/ThirdParty/libfido2::.github/workflows/macos_builds.yml@5d4bc4def3bbea615f089b3a153d117563e65be4`.

### Revision 9 deterministic read-through — libcbor core 9,901–9,964

Exact-head source-authority run `38117917553`, job `114406468676`, at `2bc832466e640f7a8bb11c569450bd1d5cb89045` succeeded through validation, source-range attestation, review-window regression, exact materialization and accepted-mode fail-closed checks. Artifact `11694536942` (digest `sha256:e1d914906cc18e4a1debb277afba53553e9e3f6c755f69fdef7b8988bd586187`) materializes orders 9,901–9,964.

The range includes CBOR conformance fixtures, streaming/load examples, OSS-Fuzz with an explicit allocator budget, build/release tooling, and the core codec through `src/cbor/tags.c`. Applicable behavior includes malformed/incomplete/memory/syntax failure separation, size_t range checks, overflow-safe collection growth and serialized-size accounting, bounded parser nesting, strict UTF-8 validation, deterministic reference cleanup, and exact streaming progress/required-byte semantics.

Existing-owner-first audit maps these behaviors into the current Coordinator WebAuthn provider/signing lifecycle plus canonical Account/Auth security and Build/Quality/Release. The staged production hardening caps external signer stdout at 1 MiB and signer event output at 256 KiB, fails closed on oversize/IO failure, and makes non-interactive signing reuse the same bounded path; focused Rust unit tests cover exact-boundary, over-boundary and IO-failure reads. This implementation is not verified until fresh descendant exact-head GitHub Actions pass.

The external `sand-webauthn-signer` packaged-helper provenance/staging gap remains open. Accounting is **9,964/16,125 read; 6,161 unread; 15,845 unknown; 0 omitted**. Next is 9,965 `Telegram/ThirdParty/libcbor::src/cbor/tags.h@a7365df10208c949b303c7fb8afc9710a7ade78f`.

### Revision 9 deterministic read-through — libcbor build/docs/examples 9,837–9,900

Exact-head source-authority run `38116111367`, job `114401252125`, at `9e9d83a7944a02d269fbb0820cbc2e67a465bedc` succeeded and artifact `11693503533` (digest `sha256:30e45c473073318b0f159d391d6b27f5a5bb206a32e5fe9b481e4cab377a7e7c`) materialized exact identities for orders 9,837–9,900. The six Windows CMake screenshots were visually inspected as build documentation only.

The range establishes RFC 8949 parsing/serialization, reference ownership, streaming/incremental decode, bounded parser-stack and allocation/overflow/fuzz hardening responsibilities. Current exact-head audit corrects the earlier owner-absence note: Fabushi already ships a source-neutral Coordinator WebAuthn provider/signing lifecycle; Account/Auth passkeys remain a distinct canonical product lifecycle. These rows therefore stay mapped to existing authentication/security plus Build/Quality/Release owners. No libcbor, FIDO or Telegram runtime is introduced.

A shipping gap remains open: the Coordinator resolves an external `sand-webauthn-signer`, while the inspected desktop build/staging path does not yet prove that helper is produced and co-staged at the packaged lookup path. No row is marked verified from source reading alone.

Accounting is **9,900/16,125 read; 6,225 unread; 15,845 unknown; 0 omitted**. Next is 9,901 `Telegram/ThirdParty/libcbor::examples/create_items.c@a6e67a9870ef366248263bffff467a002e38bfa7`.

### Revision 9 deterministic read-through — KImageFormats tail and libcbor/WebAuthn entry 9,773–9,836

Exact-head source-authority run `38115822404`, job `114400440327`, at `17af29979829565d5d7cc0d7ec81b6db78d03b82` completed Revision 9 validation, review-window regression, materialization and expected fail-closed accepted-mode checks. Artifact `11692899297` (digest `sha256:95b4ca47e543b78d48f45a7bd3298655e187b8f9227e890e3265fdd3794abb9f`) re-attested exact Git blob identities for orders 9,773–9,836.

Orders 9,773–9,828 close the remaining KDE KImageFormats source range: JPEG XL, KRA/ORA package projection, PCX/PIC/PSD/QOI/RAS/RAW/RGB/TGA/XCF, shared RLE/scanline/allocation helpers and codec test utilities. The component is therefore fully read, but its responsibility rows remain `mapped-open` because direct KDE KImageFormats product composition is absent from pinned tdesktop CMake and applicability must be closed responsibility-by-responsibility rather than inferred from recursive checkout.

Order 9,829 begins `PJK/libcbor@170bee2b82cdb7b2ed25af301f62cb6efdd40ec1`. Unlike KImageFormats, libcbor has confirmed shipping reachability: pinned `Telegram/cmake/lib_fido2.cmake` directly compiles libcbor C sources into tdesktop's USB-HID CTAP2/WebAuthn security-key backend and pins libcbor 0.11.0 with `CBOR_BUFFER_GROWTH=2` and `CBOR_MAX_STACK_SIZE=2048`. Current Fabushi search found no dedicated WebAuthn/passkey/FIDO owner, so this responsibility cannot be declared non-applicable merely because there is no Telegram network dependency. Existing Fabushi Identity/account/security owners must be audited first; only if owner absence is proven may a minimal source-neutral WebAuthn capability owner be introduced.

The first libcbor rows cover cross-platform CI, sanitizer/Valgrind testing, fuzz workflows and repository quality contracts. Ongoing source review has already identified protocol-neutral requirements for overflow-proof allocation, parser stack limits, allocator-failure cleanup, malformed-input fail-closed behavior, incremental decoder required-byte semantics, UTF-8/codepoint handling and fuzzing of network-like CBOR. These remain mapped to the future canonical authentication/protocol/resource boundary pending full lib_fido2/WebAuthn context.

Persisted accounting is **9,836/16,125 read; 6,289 unread; 15,845 unknown; 0 omitted**. Next is 9,837 `Telegram/ThirdParty/libcbor::.gitmodules@b1f6f9c259a83355bd23576a06702fd1d8541d44`. Source reading is not product verification; exact-head CI and responsibility-level closure remain mandatory.

### Revision 9 deterministic read-through — XCF/write harness and image-plugin implementations 9,709–9,772

GitHub Actions source-authority run `38114691755` at exact head `9ee0141a340df211836d139db8d7fe0a5e9ac657` materialized orders 9,709–9,772 in artifact `11693336517` (digest `sha256:372545e780f881f81a5043b5e5aa47b62ed40ab7c67e920633884cc8f72b75aa`) after strict source-range attestation. The exact window closes the remaining XCF ICC/reference fixtures, shared random/sequential read and write/readback harnesses, LibRaw/build/plugin metadata, and the first ANI/AVIF/EPS/EXR/HDR/HEIF implementation sources.

Pinned tdesktop reachability was checked separately. The direct KImageFormats gitlink is recursively checked out, but the exact root/Telegram product CMake does not add or include KDE KImageFormats; accepted preparation and packaged dependency paths select Qt `qtimageformats`. Recursive checkout therefore proves source availability, not direct KDE codec shipping composition. No KDE/Telegram codec runtime or second media registry is introduced.

Existing-owner-first exposed one applicable production delta in a capability Fabushi already ships: AVIF/HEIF attachment preview dimensions. The canonical Host ISO-BMFF parser already resolves the primary item `ispe` and `irot`; it now also applies a valid shrinking AVIF `clap` clean aperture before rotation. Malformed or expanding `clap` data is ignored rather than expanding dimensions or failing preview ingestion. Commit `e5f8efc750d2c00db98760311da78f5a5ef7a2b0` implements this in `link_preview_image_bounds.rs` with a focused synthetic primary-item `ispe + clap + irot` contract test. Descendant exact-head GitHub Actions remain required before marking the production delta verified.

All 64 source rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`: exact reading plus one implemented sub-responsibility does not close the full codec/product responsibility set. Accounting is **9,772/16,125 read; 6,353 unread; 15,845 unknown; 0 omitted**. Next is 9,773 `Telegram/ThirdParty/kimageformats::src/imageformats/jxl.cpp@bb3f09bf96a794c455b1f002309e623eed88d1db`.

### Revision 9 deterministic read-through — SGI RGB / TGA / XCF window 9,645–9,708

`cbf48857db75d2c5557385723ae449acaa6de54d` exact-head Rust runtime run `38114391045` passed source-range attestation, strict Revision 9 validation and the review-window helper before materializing orders 9,645–9,708. Artifact `11692612830` (`sha256:c6c1d7244ee4495e3671e4003189e526e7c83ede7b914871038cb5fc05d5d4ab`) contains all 64 exact blobs and a window manifest with recomputed Git blob SHA for every entry.

SGI fixtures constrain 32×32 three-channel one-byte/channel decode across aggressive RLE, ordinary RLE and verbatim storage. TGA fixtures constrain 24/32-bit RGB/RGBA, RLE/raw storage and upper/lower-left origin handling. XCF fixtures constrain v0/v11/v12, RGB/grayscale/alpha output, precision codes for U8/U16/U32 integer plus half/float samples, a historical bug case and ICC-profile variants. Exact headers/reference dimensions were inspected from the artifact, not inferred from filenames.

Existing-owner audit finds one canonical Fabushi Media/Resource path: `source/host/src/media_mime.rs` plus Electron attachment/media owners. That shipping path currently declares AVIF/BMP/GIF/ICO/JPEG/PNG/SVG/WebP and servable HEIC/HEIF, not SGI RGB/TGA/XCF. Since pinned tdesktop build/shipping reachability for kimageformats remains unproven, all 64 rows stay `mapped-open`, `unknown_closed=false`, `omitted=false`; no KDE/Telegram codec runtime or second media registry is added. Accounting is **9,708/16,125 read; 6,417 unread; 15,845 unknown; 0 omitted**. Next is 9,709 `Telegram/ThirdParty/kimageformats::autotests/read/xcf/fruktpilot_icc.xcf@c57c6548d7ddb8a3b0e9556a2df8323cde65be06`. Fresh descendant exact-head Actions remain mandatory.

### Revision 9 deterministic read-through — PSD/QOI/RAS/RAW/SGI window 9,581–9,644

GitHub Actions run `38113926190` at exact head `05ac0fb3c69f4f5b8c5d2266ec9eefaec1f5feb6` materialized the persisted first-unread window into artifact `11692219924` (`sha256:f4bc0854618bc82f39638d59d728538f3e5be77488ccc71c9d779a24bf1942b4`) and recomputed every Git blob SHA before upload. The prior >1 MiB connector gap is therefore closed by exact Actions evidence rather than inference.

Orders 9,581–9,625 cover PSD/PSB/PDD/TIFF acceptance inputs and references: PSD version 1 / PSB version 2, 1/8/16/32-bit depths, 1–57 channels, Bitmap/Grayscale/Indexed/RGB/CMYK/Multichannel/Duotone/Lab modes, Photoshop/Affinity/GIMP compatibility fixtures and paired output dimensions. Orders 9,626–9,637 cover QOI RGB/RGBA from 1×1 through 256×256; 9,638–9,639 cover 24-bit Sun Raster; 9,640–9,641 cover Kodak RAW plus its 1216×912 reference; 9,642–9,644 begin SGI RGB coverage. Exact binary headers/dimensions/depth/channel fields were inspected from the Actions artifact.

All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`: source reading establishes codec/resource acceptance responsibilities but does not prove pinned tdesktop shipping reachability or Fabushi codec parity. Existing-owner-first keeps the source-neutral canonical Media/Resource owner; no KDE/Telegram media runtime or second registry is introduced. Accounting is **9,644/16,125 read; 6,481 unread; 15,845 unknown; 0 omitted**. Next is 9,645 `Telegram/ThirdParty/kimageformats::autotests/read/rgb/rgb-gimp-2.8.10-aggressive-rle.rgb@c5f7f03f816516189825acad40721c83558d8ea9`. Fresh descendant exact-head Actions remain mandatory.

### Revision 9 deterministic read-through — kimageformats decode corpus 9,518–9,580

Orders 9,518–9,580 exact-read the next pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec` acceptance corpus: EPS, OpenEXR, Radiance HDR, HEIF/HEJ2, JPEG XL, KRA/ORA, PCX and the opening 16-bit PSD cases. Binary content was read directly and inspected by magic/header/reference dimensions rather than inferred from file names. JXL orientation 1–8 converges through the shared `readtest.cpp` auto-transform contract; KRA/ORA flatten to one 200×200 reference; PCX spans mono, indexed and RGB planes; PSD begins 16-bit RGB/RLE and grayscale depth coverage.

Pinned tdesktop reachability was also audited. Repository search at `telegramdesktop/tdesktop@6fed91ff…` currently finds `kimageformats` only in `.gitmodules`, with no direct `kimg_*`, `QImageIOPlugin` or `ThirdParty/kimageformats` shipping-build reference. Therefore codec fixtures remain source authority but their product applicability cannot be upgraded merely because the submodule contains a decoder. Fabushi's existing Media/Resource owner remains canonical.

All 63 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **9,580/16,125 read; 6,545 unread; 15,845 unknown; 0 omitted**. The next exact source is 9,581 `Telegram/ThirdParty/kimageformats::autotests/read/psd/16bit_photoshop.psb@c09d6bb5bc492441d4684531aaa1a6fbe67b4c60`. The GitHub Contents connector currently does not return that >1 MiB binary payload, so it remains unread rather than inferred; work continues on other non-blocked responsibilities. Fresh descendant exact-head Actions remain mandatory.

### Revision 9 deterministic read-through — PIC/AVIF media acceptance 9,474–9,517

Orders 9,474–9,517 exact-read the next pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec` range. PIC fixtures and `pictest.cpp` establish raw/RLE compression, alpha/non-alpha output, deterministic encoded bytes, short/full-length Description metadata, long runs, input-format conversion, and pre-read text/size/image-format semantics. AVIF fixtures cover grayscale, grayscale-alpha, RGB/RGBA and the full auto-transform orientation matrix: unrotated/90/180/270 combined with horizontal or vertical mirroring all converge to the same canonical 480×160 reference orientation.

Binary fixtures were read by exact base64/raw bytes; PNG dimensions and selected reference visuals were inspected. Existing-owner-first keeps the canonical attachment/media, Resource and Media/Quality owners. The current production branch also bounds the existing local-image preview fast path before decode using `ATTACHMENT_PREVIEW_BYTE_CAP`; the strict Grok semantic-adaptation evidence lock is rebound to those intentional target blobs rather than weakened.

All 44 source rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; the bounded preview fix is narrower than full codec parity. Accounting is **9,517/16,125 read; 6,608 unread; 15,845 unknown; 0 omitted**. Next is 9,518 `Telegram/ThirdParty/kimageformats::autotests/read/eps/bw.eps@3af28d5d28351a749cbcaf0e30e3652a159d830a`. Fresh descendant exact-head GitHub Actions remain mandatory.

### Revision 9 deterministic read-through — kimageformats opening range 9,454–9,473

Orders 9,454–9,473 exact-read the opening range of pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`: repository/build policy, optional codec dependency gates, exact license corpus, declared read-only/read-write image capabilities, format acceptance topology, the ANI binary fixture plus all three frame PNGs, its decoder contract, and the bounded fuzzy image comparator. Binary fixtures were read by exact base64/raw bytes and the PNG frames were visually inspected rather than inferred from filenames.

The ANI contract preserves 32×32 dimensions, Title/Author metadata, four-frame custom sequencing, 166/333 ms delays, end-of-stream behavior and jump/reset semantics. Existing-owner-first keeps attachment/media, Resource, Build/Packaging and Release/NOTICE owners canonical. The current branch separately fixes the shipping local-image fast path so the canonical preview-byte cap is enforced before file decode; that narrow production fix does not mark this broader codec range verified.

All 20 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **9,473/16,125 read; 6,652 unread; 15,845 unknown; 0 omitted**. Next is 9,474 `Telegram/ThirdParty/kimageformats::autotests/long-runs.pic@620a3bb8eb322d06e0468eb641c56bd72b6c6545`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — kcoreaddons test/demo tail 9,445–9,453

Orders 9,445–9,453 exact-read the nine final non-directory blobs of pinned `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15`. They cover the optional Qt Widgets manual-test build, system-user face-icon discovery with missing-icon fallback and bounded scaling, directory-watch dirty/created/deleted events with explicit scan start/stop/restart plus multi-watcher file/subdirectory observation, deterministic seeded random-sequence/shuffle behavior, and a debounced plaintext-to-HTML demo surface.

Existing-owner-first keeps Build/Quality, platform user/profile Resource, Resource/filesystem-watch, security randomness/test utility and Transcript safe-rendering owners canonical. The Qt demo UI is replacement-only; system-user data is not Fabushi account identity; deterministic `KRandomSequence` behavior is never security authority for tokens/nonces/identity. This range corrects the exact-head CI defect that proved `src/mimetypes/kde5.xml` was not the component tail. The validator remains strict; no source entry is skipped.

All nine rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **9,453/16,125 read; 6,672 unread; 15,845 unknown; 0 omitted**. Kcoreaddons is now exact-read through all 377 non-directory entries. Next is 9,454 `Telegram/ThirdParty/kimageformats::.git-blame-ignore-revs@b34bba3cd8fb4c6785690422f5c46762d60bb7dc` in pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — kcoreaddons component tail 9,401–9,444

Orders 9,401–9,444 exact-read the remainder of pinned `KDE/kcoreaddons`: safe text/emoticon interface, legacy configuration migration, locale-aware byte/duration/date formatting, executable/open-file discovery, memory telemetry, `os-release`, cross-platform process enumeration, runtime-platform hints, sandbox host bridging, Unix/Windows shell parsing and quoting, signal integration, Unix/Windows user/group/SID semantics, and the final MIME corpus (57 types, 73 globs, 26 subclass relations, 9 magic rules, 3 aliases).

Existing-owner-first maps these into canonical Localization, Settings/Persistence, Resource/platform, Host/Runner/Computer, Permissions, Transcript and attachment/MIME owners. Critical boundaries stay explicit: migrations never overwrite existing target state; open-files reports typed unsupported rather than fake success; memory distinguishes free from available; OS user/group data is platform permission context, not Fabushi account identity; sandbox/shell helpers cannot bypass approval; MIME and rendered-link inputs stay validation-bound. No KDE runtime or second platform/settings/identity truth is introduced.

This also fixes an authority-accounting defect exposed by exact-head CI: `upstream.lock.source_disposition_evidence` had remained at 9,201 while master/index/ledger had advanced. The lock summary is now advanced from the same source prefix instead of weakening the validator.

Accounting: **9,444/16,125 read; 6,681 unread; 15,845 unknown; 0 omitted**. Kcoreaddons source reading is complete; next 9,445 is `Telegram/ThirdParty/kimageformats::.git-blame-ignore-revs@b34bba3cd8fb4c6785690422f5c46762d60bb7dc` in pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`.


### Revision 9 deterministic read-through — plugin/text/security utility source 9,381–9,400

Orders 9,381–9,400 exact-read static-plugin helpers, random/shuffle APIs, fuzzy matching/ranking, localized JSON lookup, macro expansion and platform shell quoting, string utilities, and bounded plain-text → HTML/linkification. The security disposition is explicit: upstream KRandom itself says it is not suitable for serious randomness, so it is never authority for Fabushi tokens/nonces/identity; macro expansion remains behind canonical Host/Runner argument, quoting, approval and sandbox boundaries; text rendering must validate/bound links and escape entities under Transcript/Resource safe-link policy.

Existing-owner-first maps fuzzy scoring to canonical Search, localized JSON to Localization, shell-aware expansion to Host/Runner/Computer, and linkification/text projection to Transcript/Resource. Static Qt plugin registration remains non-shipping implementation detail. Rows remain mapped-open / unknown_closed=false until production and same-head evidence close them.

Accounting: **9,400/16,125 read; 6,725 unread; 15,845 unknown; 0 omitted**. Next 9,401 `Telegram/ThirdParty/kcoreaddons::src/lib/text/ktexttohtml_p.h@93bbb72f1eeb0adc8c2ec57ebc6ab5c3309d197d`.


### Revision 9 deterministic read-through — licenses and plugin lifecycle 9,361–9,380

Orders 9,361–9,380 exact-read the KCoreAddons runtime-version contract, installed license corpus, legacy desktop metadata parser, plugin factory/loader and typed plugin metadata. Applicable semantics include runtime-vs-build provenance, exact third-party license availability, strict typed metadata conversion, service-type/path resolution with malformed-input failure, concurrency-safe parser caching, stable plugin/source identity, load/error/lifecycle separation, MIME inheritance, default-enable state, localized metadata fallback and author/license/version provenance.

Existing-owner-first audit found stronger shipping equivalents in `source/packages/cursor-plugins/manifest-parser.ts` and Mahayana `core-plugins`: manifest size is bounded, plugin/schema/path/id data is typed, invalid/duplicate entries are fenced, variables are recursively validated, and loader/store/cache/version lifecycle stays under the canonical Marketplace owner. Therefore no KDE/Qt plugin runtime or second schema truth is introduced. These rows remain `mapped-open` / `unknown_closed=false` until full applicability and same-head production evidence close them.

Accounting: **9,380/16,125 read; 6,745 unread; 15,845 unknown; 0 omitted**. Next 9,381 `Telegram/ThirdParty/kcoreaddons::src/lib/plugin/kstaticpluginhelpers.cpp@1bf196d1b85d1969f0fd70a10e5023b4a4a5522c`.


### Revision 9 deterministic read-through — portal/job/provenance 9,345–9,360

Orders 9,345–9,360 exact-read the XDG file-transfer/KIO-fuse interfaces, KJob/KCompositeJob/tracker/UI-delegate lifecycle, and KAboutData/core version source. Applicable product semantics include explicit portal Start/Add/Retrieve/Stop/closed lifecycle; bounded resource ownership; exactly-once task terminal signals; kill/suspend/resume capabilities; first-error propagation; explicit subtask ownership; progress/speed observability; and typed SPDX/component provenance distinct from Fabushi product identity.

Existing-owner-first maps these to canonical Resource/attachment + platform portal, Task/Coordinator + Task UI, and product identity/NOTICE owners. No KDE Job framework, KDE UI delegate, KDE product identity or DBus runtime is introduced as a second owner. Rows remain mapped-open until implementation/evidence obligations close.

Accounting: **9,360/16,125 read; 6,765 unread; 15,845 unknown; 0 omitted**. Next 9,361 `Telegram/ThirdParty/kcoreaddons::src/lib/kcoreaddons.h@77b482d1b75330c1e38908a7d9c974fced309dd7`.


### Revision 9 deterministic read-through — kcoreaddons production source 9,309–9,344

Orders 9,309–9,344 exact-read the first 36 blobs of pinned `KDE/kcoreaddons/src`. The window covers build and localization extraction, strict desktop-metadata → typed JSON conversion, shared derived-cache versioning/eviction/corruption recovery and process-safe locking, stale autosave/backup ownership, create/delete/recreate/move-aware filesystem watching, filesystem/network-mount classification, collision-safe file naming, typed diagnostics, child-process argv/shell/environment/timeout semantics, and URL/attachment transfer through local/portal boundaries.

Existing-owner-first mapping keeps Fabushi build/packaging, Plugins/MCP/Marketplace, Persistence/Recovery, Resource/platform, Settings, Host/Runner/Computer and attachment owners canonical. The recently added Marketplace cache-hit recovery closes only a narrow corrupted-derived-cache behavior and is not used to overclaim the broader shared-cache rows. No KDE/Qt runtime, second cache truth, second process owner, second Settings owner or second attachment owner is introduced. All rows remain `mapped-open` / `unknown_closed=false` until their production/evidence obligations close.

Accounting is **9,344/16,125 read; 6,781 unread; 15,845 unknown; 0 omitted**. Next is 9,345 `Telegram/ThirdParty/kcoreaddons::src/lib/io/kurlmimedata.h@15bd05e4bc36d6c5b377029c695bb54b876f2332`. Fresh descendant exact-head GitHub Actions are mandatory.


### Revision 9 deterministic read-through — kcoreaddons localization resources 9,202–9,308

All 107 `poqm/*/kcoreaddons5_qt.po` blobs from pinned `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15` were read by exact blob identity. The common live catalog shape is 133 msgids / 132 contexts / 6 plural entries, but English is intentionally a six-entry plural shim and the four Serbian variants are sparse 117-msgid catalogs. Canonical localization therefore must preserve locale/context/plural/UTF-8/placeholder semantics while supporting missing-category and missing-entry fallback; fuzzy/obsolete translation history is evidence only and must not become shipping copy.

Existing-owner audit finds the shipping owner in `frontend/src/recovered/features/settings/overlay/localization.ts`, consumed by the real Settings panels. This descendant keeps that single owner and strengthens the live accessibility-count projection with locale-safe `Intl.PluralRules` category selection, category-to-`other` fallback and invalid-locale/non-finite-count fail-safe behavior. No KDE/gettext runtime, second localization store or KDE product wording is introduced. The broader rows remain `mapped-open` / `unknown_closed=false` pending complete production evidence.

Accounting is **9,308/16,125 read; 6,817 unread; 15,845 unknown; 0 omitted**. Next is 9,309 `Telegram/ThirdParty/kcoreaddons::src/CMakeLists.txt@1c15425347a9ffc4e12bbd63797616337c78fe33`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — Hunspell compound flags, conditions, casing and continuation grammar 8,666–8,729

Orders 8,666–8,729 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The range completes the next COMPOUNDRULE sequence/repetition/optionality corpus; proves equivalent ordinal grammar under base, long and numeric flag encodings; covers digit, decimal, percent and per-mille composition with stable analyze/stem metadata; exercises legacy ISO-8859-2 and UTF-8 affix-condition character classes and negation; and preserves continuation-class gating for derived prefixes plus homonym morphology.

The range also fixes product requirements for digit-and-hyphen word compounds, Turkish dotted/dotless-I casing, ISO-8859-15 oe-ligature case conversion, semantic parity across single/long/numeric/UTF-8 flag formats, and Swedish linking-morpheme composition through COMPOUNDBEGIN/ONLYINCOMPOUND/COMPOUNDPERMITFLAG. Order 8,729 begins FORBIDDENWORD configuration semantics; its dictionary and good/wrong oracles remain the next unread files rather than being guessed across the deterministic boundary. No Hunspell/Telegram side runtime, second input truth or second dictionary truth was introduced.

All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; source reading therefore reduces only unread. Accounting is **8,729/16,125 read; 7,396 unread; 15,845 unknown; 0 omitted**. Next is 8,730 `Telegram/ThirdParty/hunspell::tests/forbiddenword.dic@a6b69bea0b713dfc5ff5e7d7e6d2d33ef8363ebb`. The exact-head `b1ab6c8e…` workflows were fully green and bound the source window, but become predecessor evidence after this descendant commit; fresh descendant exact-head GitHub Actions are mandatory before acceptance.

### Revision 9 deterministic read-through — Hunspell compound/casing/morphology regressions 8,602–8,665

Orders 8,602–8,665 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. They close the remaining CHECKCOMPOUNDREP acceptance oracle, then preserve CHECKCOMPOUNDTRIPLE boundary rejection; German `ß`/SS special capitalization and KEEPCASE behavior in legacy/UTF paths; CIRCUMFIX pairing with comparative/superlative analyze/stem metadata; punctuation WORDCHARS tokenization; two-level COMPLEXPREFIXES including Coptic Unicode; a concrete compound recursion `wnum` overflow/no-crash regression; compound-affix base, permit and forbid rules; COMPOUNDMIN/COMPOUNDFLAG eligibility; COMPOUNDFORBIDFLAG precedence; and the beginning of COMPOUNDRULE exact sequence, repetition and optionality grammar.

The predecessor 8,494–8,601 shard had valid range/accounting content but serialized the row collection under `entries`; the Revision 9 authority validator requires `rows`. This descendant fixes that authority serialization contract rather than weakening the validator. All 64 new rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`.

Accounting is **8,665/16,125 read; 7,460 unread; 15,845 unknown; 0 omitted**. Next is 8,666 `Telegram/ThirdParty/hunspell::tests/compoundrule3.dic@7d07bbc89a80ce1992b566104fa91dad014b54a8`. Fresh descendant exact-head GitHub Actions are mandatory; upstream regression fixtures do not by themselves prove Fabushi production closure.

### Revision 9 deterministic read-through — Hunspell IJ, ABI, aliases, casing, breaks and compound regressions 8,494–8,601

Orders 8,494–8,601 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The 8,494–8,501 tranche was deliberately re-read rather than inheriting the prior unaccounted observation: every repository/path/blob identity matched, including Dutch IJ case correction, the Automake regression graph, the public C/C++ ABI baseline, and the fail-closed ABI checker whose baseline may be refreshed only after a deliberate SONAME break and which explicitly handles LP64/ILP32 and non-libstdc++ differences.

The remaining rows establish prefix/suffix cross-product behavior; AF/AM morphology aliases and complex prefixes; all-caps, mixed-case, forbidden-case and homonym behavior; UTF-8 and Arabic combining/IGNORE handling; legacy/UTF replacement suggestions including Turkish dotted-I; recursive/default/disabled word-break semantics; compound-case and duplicate-component constraints; CHECKCOMPOUNDPATTERN boundary transformations including language-specific sandhi; and CHECKCOMPOUNDREP rejection, including the regression where a replacement applied at the wrong byte position. These map to existing canonical text-input/spellcheck/tokenization and Build/Release owners; no Hunspell/Telegram parallel runtime or second input/dictionary truth is introduced.

All 108 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,601/16,125 read; 7,524 unread; 15,845 unknown; 0 omitted**. Next is 8,602 `Telegram/ThirdParty/hunspell::tests/checkcompoundrep2.good@737578025367c6a45d4ab904aa1c9f18d8265fe3`. Fresh descendant exact-head GitHub Actions are mandatory; upstream fixtures and ABI scripts do not by themselves prove Fabushi production closure.

### Revision 9 deterministic read-through — Hunspell tools, fuzzers and regression fixtures 8,430–8,493

Orders 8,430–8,493 were exact-read from `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The tools tail adds explicit bounded compressed-dictionary, morphology, parser and personal-dictionary fuzz contracts; a targeted >65,535-byte `add_with_affix` UAF regression; safe temporary-file and hzip framing/error behavior; affix/morph alias conversion; munch/unmunch FULLSTRIP/cross-product/capacity semantics; SET-aware Unicode word-form transforms; and deterministic wordlist-to-dictionary generation. These map to existing canonical spellcheck/Resource/dictionary/Unicode/security/Build owners; Hunspell CLI/script identity is replacement-only.

The regression fixtures are product requirements, not ignorable test data. They lock capitalized German n-gram suggestion behavior in legacy and UTF-8 forms; German homonym, NEEDAFFIX and COMPOUNDRULE semantics; large AF/AM alias-count crash handling including malformed directives; Arabic UTF-8 IGNORE combining-mark/tatweel normalization with affix conditions; three-part compound pattern ordering; flag-qualified compound patterns; and interaction between COMPOUNDRULE and COMPOUNDBEGIN/COMPOUNDEND. Empty `.good` files in the alias-crash cases represent no-crash expectations rather than absence of responsibility.

All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,493/16,125 read; 7,632 unread; 15,845 unknown; 0 omitted**. Next is 8,494 `Telegram/ThirdParty/hunspell::tests/IJ.aff@c817c2e9132f389983b698b6f13555669ca0f317`. Fresh descendant exact-head GitHub Actions remain mandatory; upstream tests/fuzzers do not by themselves prove Fabushi production closure.

### Revision 9 deterministic read-through — Hunspell core, parsers and tooling 8,366–8,429

Orders 8,366–8,429 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. Predecessor source-authority artifact 11686195087 (sha256:5d777a472ca1c07d3ea5b0f31250925802bb1fc3ec508518e22d7c6dc1120aaa) materialized all 64 accepted blobs and fixed path/mode/size/object identity; after this descendant it remains source-identity evidence only, not descendant acceptance evidence.

The core range carries production responsibilities for affix/prefix/suffix and compound state machines, hard suggestion/time bounds, Unicode and legacy-encoding conversion, language-sensitive casing, dictionary/custom-word mutation and persistence, compressed-dictionary failure handling, phonetic/replacement/suggestion ranking, public API lifecycle and ABI/version/visibility contracts. Parser sources add text/HTML/XML/ODF/LaTeX/man/first-field segmentation with bounded state, URL/entity/markup shielding and escaping-safe replacement. Tooling adds deterministic Unicode/resource generators, dictionary discovery and personal-dictionary persistence, secure filename/temp behavior, parser/encoding selection, and fuzz coverage for aff/dic, public spell APIs and compressed dictionaries. Standalone Hunspell CLI/C++ identity remains replacement-only; no second input, dictionary or messaging runtime is introduced.

All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,429/16,125 read; 7,696 unread; 15,845 unknown; 0 omitted**. The next source is 8,430 `Telegram/ThirdParty/hunspell::src/tools/hzfuzzer.options@e39560b87eed0e2afe3ed217893d757ec8efb942`. Fresh descendant exact-head GitHub Actions remain mandatory; source reading does not grant production or release credit.

### Revision 9 deterministic read-through — remaining Hunspell localization 8,335–8,365

Orders 8,335–8,365 were exact-read from `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The remaining locale catalogs preserve the same 65-message spellcheck error/help surface and 12 c-format entries with materially different translation completeness, so they map to canonical localization/Resource fallback, Unicode, formatting-safety and text-input/spellcheck owners rather than locale-specific runtimes. `quot.sed` preserves deterministic Unicode quotation substitution, `remove-potcdate.sin` removes only the first non-semantic POT creation timestamp for reproducible catalogs, and `stamp-po` is a generator freshness sentinel rather than runtime state.

All 31 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,365/16,125 read; 7,760 unread; 15,845 unknown; 0 omitted**. The next source is 8,366 `Telegram/ThirdParty/hunspell::src/Makefile.am@45b7703d662c0a1465ebaa17d6f3741972f6184a`, which begins the actual Hunspell production source ownership range. Descendant exact-head GitHub Actions remain required.

### Revision 9 deterministic read-through — Hunspell docs, Windows, fuzzing, gettext 8,258–8,334

Orders 8,258–8,334 were exact-read from `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b` and decomposed existing-owner-first. The manpage range contributes mature spellcheck responsibilities rather than presentation-only documentation: dictionary discovery, personal dictionary persistence, interactive correction and pipe semantics, suggestion/morphology/compound behavior, Unicode and encoding rules, API allocation/free lifecycle, locale behavior, parser modes, and compressed dictionary portability. Source-specific Hunspell CLI/C++ APIs and hzip formats are not retained as product owners.

The MSVC range records Windows Win32/x64/ARM64, Debug/Release, static/DLL runtime, warnings/linking/security settings and binary version metadata; these map to canonical Windows Build/packaging/provenance owners. `oss-fuzz-build.sh` maps core, aff/dic, personal-dictionary, parser, morphology and compressed-dictionary fuzz coverage plus representative corpora into canonical security/fuzz responsibilities.

The gettext range records explicit supported locales, POT/PO/GMO generation and deterministic update/install lifecycle, UTF-8 extraction policy, localized diagnostic/help messages with c-format placeholders, quote/Unicode terminal transforms, partial/fuzzy catalog coverage and fallback expectations. These map to canonical localization/Resource + text-input/spellcheck owners; no source-specific gettext runtime is introduced.

All 77 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,334/16,125 read; 7,791 unread; 15,845 unknown; 0 omitted**. Next source is 8,335 `Telegram/ThirdParty/hunspell::po/nb_NO.po@8bb372e8c9f2728a81365d13bdfbbf3b892c07e0`. Descendant exact-head GitHub Actions remain required.

### Revision 9 deterministic read-through — Hunspell licenses and portable build macros 8,243–8,257

Current-head authority artifact 11685032724 fixed immutable recursive identity for this window and exposed one predecessor ledger drift: order 8,232 `ChangeLog.O` is blob `a2c712d73bc4f5d8cbfbe5eea5f0696ef9d921dd` with Git-tree size 20,359 bytes, not 21,588. The correction changes no source object, responsibility, disposition or gate; it restores exact tree identity so the existing validator can continue to fail closed.

Orders 8,243–8,257 were then exact-read from `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The two license files preserve Hunspell tri-license and MySpell redistribution/attribution provenance. The m4 range carries mature source-neutral build responsibilities: C++ standard capability detection; gettext/NLS and PO catalog generation; iconv/Unicode portability; linker and transitive-library/rpath/multilib discovery; deterministic executable lookup; and compiler symbol-visibility capability detection. These map existing-owner-first into canonical NOTICE/provenance, text-input/spellcheck, Unicode/Resource/localization, Build/toolchain/linker/packaging/security owners. No Hunspell or Autoconf product runtime is introduced. Every row remains `mapped-open`, `unknown_closed=false`, `omitted=false`.

The same descendant adds the minimal platform-input bridge demanded by the earlier Hime/fcitx source analysis. macOS reads the OS-owned current input-source identifier; Linux probes installed Fcitx5/IBus providers; Windows and unavailable fields fail closed to typed unknown. Chromium composition events remain the authoritative preedit/focus/lifecycle owner, and native async snapshots are accepted only when scope, composition generation and latest-request fencing still match. No browser locale inference and no second input runtime are introduced.

Accounting is **8,257/16,125 read; 7,868 unread; 15,845 unknown; 0 omitted**. Next source is 8,258 `Telegram/ThirdParty/hunspell::man/Makefile.am@ee37fb3aec4520edab3b32be072d8f9d22f02c64`. Descendant exact-head GitHub Actions remain required.


### Revision 9 deterministic correction — Hime GTAB / GTK native input / runtime 8,096–8,162

Fresh exact-head validation on `77c411df86d5f4314be10e09a5a67478fdbf4262` failed closed at order 8,101: the accepted recursive path was `src/gtk-im/Makefile`, while the recorded row had drifted to `src/gtk_bug_fix.c`. The repair does not relax `tdrp-recursive-inventory-contract.mjs` or `validate-tdrp-revision9-authority.mjs`. Instead it reconstructs the range from the pinned Hime Git tree.

Orders 8,101–8,105 were newly read as the GTK2 input-context branch. Its applicable product semantics are client-window/input-context lifecycle, key press/release routing with printable fallback, preedit start/change/end and typed attributes/cursor, commit, focus in/out, reset, cursor geometry, teardown, exactly-one provider registration and locale-as-default-hint-only. Orders 8,106–8,110 are GTK3 build composition plus four exact mode-`120000` symlinks whose literal targets point back to the GTK2 source; those aliases do not duplicate product responsibility. Orders 8,111–8,162 are re-bound by exact source path/blob identity to their true recursive positions, preserving the already-read responsibility decompositions for resource/config/security/module/phrase/IPC/locale/Anthy behavior.

Hime/GTK/XIM runtime identity remains replacement-only. The canonical `PromptRichTextEditor` DOM composition path truthfully reports focus/composition/preedit lifecycle, but current OS input method, language, width, script variant and candidate state still require a source-neutral native platform adapter and cannot be inferred from locale or browser state.

All rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Correct accounting is **8,162/16,125 read; 7,963 unread; 15,845 unknown; 0 omitted**. The true next source is 8,163 `Telegram/ThirdParty/hime::src/modules/chewing-conf.c@93cc1fd31b57e7b45c1fe272ebd945ed7f6632c5`.

### Revision 9 deterministic read-through — Hime IMdkit/runtime/input catalog 8,077–8,095

Orders 8,077–8,095 were read directly from the pinned `hime-ime/hime@9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5` source; each GitHub-returned blob SHA matched accepted recursive inventory. The range closes source reading for typed/nested input-context mutation, input-server capabilities, per-client protocol dispatch/sync/pending queues, client identity/message framing, bounded transport and deterministic disconnect cleanup; then maps Hime's central focus/composition/output/candidate state machine, parse cache, full-width mapping, rendering compatibility, candidate buffer editing/learning, table loading/validation and method-catalog/default-selection semantics into existing source-neutral Fabushi owners.

No XIM, Hime or GTK product runtime is introduced. Existing `PromptRichTextEditor` remains the canonical DOM composition/scope owner; native platform-input method/language/width/script/candidate signals remain open and must be truthful rather than inferred. All rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; source reading alone closes no unknown. Accounting is **8,095/16,125 read; 8,030 unread; 15,845 unknown; 0 omitted**. Next source is 8,096 `Telegram/ThirdParty/hime::src/gtab-tsin-fname.c@59ed9cbeb62f6a0accb99dd2ec097b56a10576d4`. Fresh exact-head GitHub Actions are still required.

### Revision 9 deterministic read-through — Hime pink tail, settings/data lifecycle and IMdkit 8,013–8,076

Exact-head source-authority artifact 11681780325 (sha256:92d496e33e3f5fdd523a0033ce41bcac995dd829643116fda6581650fb38ced8) from `46dfaf8102187cc46824362b143e9ac4f3d4b4b9` materialized the complete 64-entry review window with accepted Git identity, mode, byte length and SHA-256. Orders 8,013–8,023 are the remaining pink method/language/symbol status glyphs and were visually inspected; their artwork/palette is replacement-only. 8,024 is LGPL v2.1 provenance. 8,025–8,047 expose manpage responsibilities for deterministic table transforms, platform diagnostics/lifecycle, script/full-width/input settings, virtual keyboard, notifications, phrase learning/edit/import/export and the input-service contract. Symlink manpages preserve exact alias provenance only.

Orders 8,048–8,060 cover symmetric Settings launcher packaging, localization catalog lifecycle and localized input-setting semantics, a fail-fast static-analysis gate, helper packaging, first-run non-clobbering data initialization, diagnostics, deterministic phrase-data merge and external-editor policy. Fabushi maps these into its single canonical Settings/Resource/platform-input/localization/Build owners rather than copying Hime surfaces.

Orders 8,061–8,076 begin the IMdkit implementation: per-editable input-context identity/focus/preedit/status lifecycle, typed capability/attribute negotiation, connection/auth/open/close and focus/event/sync/commit/reset/preedit/status state machines, endian-aware length/count/padding framing, exactly-one transport selection with fail-closed teardown, and async/sync callback settlement. Hime/XIM/X11 protocol is not retained; applicable mature semantics map to the existing source-neutral platform-input, `PromptRichTextEditor`, networking/serialization, Settings and recovery owners. Every row remains `mapped-open`, `unknown_closed=false`, `omitted=false`.

The predecessor `46dfaf8102187cc46824362b143e9ac4f3d4b4b9` required workflows are fully green (Rust desktop runtime 38084031087; Desktop Chat Parity CI 38084031095), with source authority artifact 11681780325, rust messaging artifact 11681353908 and Playwright artifact 11682075159. After main reconciliation and this accounting commit those are predecessor evidence only; fresh descendant exact-head evidence is mandatory. Accounting is **8,076/16,125 read; 8,049 unread; 15,845 unknown; 0 omitted**. Next source is 8,077 `Telegram/ThirdParty/hime::src/IMdkit/lib/i18nIc.c@61b576f996efb6d4964a776e5a6ea5ca78074ddb`.

### Revision 9 deterministic read-through — Hime gray tail, brand art and pink theme 7,949–8,012

Exact-head source-authority artifact 11681552029 (sha256:83a3108678dafcdda74afec1bcffffb9606f48315623fe24d4b5bf9a1e5d8fd4) from `7d155a0461bfe318da513ada5db7dfb65d71b71f` materialized the full 64-entry review window. All 63 PNGs were visually inspected and order 7,975 `icons/pink/COPYING` was text-read. Orders 7,949–7,972 finish the gray input-method/status set; 7,973 `icons/hime.png` is 64×64 Hime mascot/brand artwork and is replacement-only; 7,974–8,012 begin the pink theme copy of the same method/language/width/script semantics. Gray/pink palette choice is presentation, not a second input state owner. The pink full/half × simplified/traditional resources repeat the orthogonal state axes already proven from pinned Hime production source. The exact LGPL text remains provenance-only.

Existing-owner audit still finds the canonical composition lifecycle in `PromptRichTextEditor`. This descendant adds a source-neutral typed input-status snapshot and an accessible Composer status projection driven only by actual DOM composition lifecycle. It deliberately leaves method, language, width and script variant as `unknown` unless a future native platform adapter supplies truthful evidence; browser locale and keyboard guesses are prohibited. This is production progress, not full closure: native OS input-source reporting remains mapped-open and requires focused descendant exact-head evidence.

Accounting is **8,012/16,125 read; 8,113 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 8,013 `Telegram/ThirdParty/hime::icons/pink/scj.png@0d50a486136715f720a4bbc813098b2cce9d83ca`.

### Revision 9 deterministic read-through — Hime gray language/input-mode axes 7,933–7,948

Exact-head source-authority artifact 11680793020 (sha256:b767c2137f6f7c3e0d451197583a45868c7da1c2c33f7eb7e917950ec8ff22ee) from `2b3711e8f0f35d664af5dafe567e2089201f869d` materialized all 16 accepted PNG blobs with exact Git identity, mode, byte length and SHA-256; every PNG was visually inspected. The window spans en-kana-nippon, en-tsin, erbi, Esperanto, ez, fcitx-qxm, fcitx-wanfeng, gdayi3, Greek, Hakka and Hangul identities plus `full-simp`, `full-trad`, `half-simp`, `half-trad` and `halfwidth-kana`. The paired full/half × simplified/traditional resources demonstrate that width mode and script variant are independent typed axes rather than one opaque IME state, while the language/method glyphs demonstrate a separate input-method/language identity axis. Browser locale or a generic composition boolean is therefore not accepted as a production substitute. Hime artwork, palette, labels and branding remain replacement-only; full shipping closure requires a real platform-input signal feeding canonical Resource/design-system/Composer projection with accessible text, without a second input runtime. All rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; source reading closes no unknown. Accounting is **7,948/16,125 read; 8,177 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,949 `Telegram/ThirdParty/hime::icons/gray/hime-tray.png@040374994c12a7f448018f3dba80ee768d2351e7`. Descendant exact-head GitHub Actions remain required after the accounting commit.

### Revision 9 deterministic read-through — Hime dark-tail and gray input-status icons 7,917–7,932

Exact-head source-authority artifact 11680279359 (sha256:8a11aad8a061722e2f1a7024cb5fd3bdbfb3bfdaec01e2de86c9ab04f1d6ca82) from `11caed45b8b0a4e8215a6d93cc814b9f048c95a0` materialized all 16 accepted entries with a window manifest carrying Git blob identity, mode, byte length and SHA-256. The 15 PNGs were visually inspected: orders 7,917–7,922 finish dark Telecode/Tsin/Tsou/Vims/wm2/Wubi, and 7,923 plus 7,925–7,932 begin the gray 4corner/NewCJ3/Amis/array/Chewing/Cangjie/Dayi status set. Order 7,924 `icons/gray/COPYING` is the exact 151-byte text, Copyright (C) 2011 Lu, Chao-Ming (Tetralet), GNU LGPL v2.1 or later. These rows preserve typed source-neutral input-method/language/mode identity, contrast/status meaning and deterministic resource provenance; Hime artwork, palette, names and branding remain replacement-only, while the license row maps only to canonical NOTICE/license accounting. All rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; reading establishes no shipping verification and closes no unknown. Accounting is **7,932/16,125 read; 8,193 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,933 `Telegram/ThirdParty/hime::icons/gray/en-kana-nippon.png@915bba6d50df2c8bbd2fb1cbf17124220cc043b1`. The predecessor `11caed45b8b0a4e8215a6d93cc814b9f048c95a0` required workflows are completed/success; this descendant accounting commit requires fresh exact-head GitHub Actions.

### Revision 9 deterministic read-through — Hime blue-tail and dark input-status icons 7,869–7,916

The accepted `hime-ime/hime@9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5` blobs for 7,869–7,916 were fetched directly and matched against the pinned Git-tree identity/mode/size. PNGs were visually inspected; 7,874 `icons/dark/COPYING` was read as text and preserves Copyright (C) 2011 Lu, Chao-Ming (Tetralet), GNU LGPL v2.1 or later. The window finishes blue Tsou/Vims/wm2/Wubi and continues the dark input-method/language/status set through Taiwan. Repeated full/half × simplified/traditional variants confirm independent width-mode and script-variant axes across themes. Theme artwork, palette, branding and filenames remain replacement-only; typed state stays with canonical Resource/design-system/platform-input owners with accessible non-glyph alternatives, while COPYING maps to canonical NOTICE/license accounting. Reading closes no unknown. Accounting is **7,916/16,125 read; 8,209 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,917 `Telegram/ThirdParty/hime::icons/dark/telecode.png`. Descendant exact-head GitHub Actions remain required.

### Revision 9 deterministic read-through — Hime black language/method/symbol icons 7,805–7,820

Current-head artifact 11680830198 (sha256:e05eecc728d2b69fdbf944ef0d600354c698197730832fb9ef7120629f7d1b2a) materialized the whole window; archive digest, Git blob SHA and byte lengths matched accepted Hime authority. Visual review confirmed Latin letters, NewCJ, noseeing, Paiwan, Pinyin, Russian, Sakura, SCJ, Shuangpin, Simplex, Symbols, Taiwan, Telecode, Tsin, Tsou and Vims glyphs. These encode typed language/input-method/symbol-mode identity and high-contrast status projection; Hime artwork/names remain replacement-only. Applicable semantics stay with canonical Resource/design-system/platform-input owners, with accessible alternatives rather than glyph-only meaning. Reading closes no unknown. Accounting is **7,820/16,125 read; 8,305 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,821 `Telegram/ThirdParty/hime::icons/black/wm2.png`.

### Revision 9 deterministic read-through — Hime black input-mode/status icons 7,789–7,804

Current-head artifact 11680630074 (sha256:472fbfbab38fa8b5dbadf9a3aafa2778b24fa0ac0ce8bbfb11bc04398e383581) materialized the whole window; archive digest, Git blob SHA and byte lengths matched accepted Hime authority. Visual inspection confirmed fcitx Wanfeng, full/half simplified/traditional, gDayi, Greek, Hakka, half-width Kana, Hangul, tray/service presence, internal-code, JTCJ GB, Zhuyin, Jyutping and Kana Nippon glyphs. The full/half × simplified/traditional quartet is an explicit state-model signal: width and script variant are independent typed axes, not one opaque icon enum. Hime artwork/names remain replacement-only; applicable semantics belong to canonical Resource/design-system/platform-input owners with accessible non-glyph-only projection. Reading closes no unknown. Accounting is **7,804/16,125 read; 8,321 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,805 `Telegram/ThirdParty/hime::icons/black/latin-letters.png`.

### Revision 9 deterministic read-through — Hime black input-mode icons 7,773–7,788

Exact-head artifact 11680335933 (sha256:e425c4b675c000fe5effe8170f9b294272be66606acab7c380b7d7ca7eed17bf) materialized the whole window; all Git blob SHA/size values were recomputed and matched. Visual inspection confirmed 30×30 high-contrast black glyphs for distinct input methods/modes (four-corner, NewCJ3, Amis, array/row-column, Chewing, Cangjie variants, Dayi, Kana/Tsin, Erbi, Esperanto, EZ and fcitx-qxm). Those glyphs carry typed mode/status identity and contrast semantics but Hime artwork/names are replacement-only. Order 7,774 establishes copyright Lu, Chao-Ming (Tetralet) and LGPL v2.1-or-later provenance for the black icon set. Reading closes no unknown. Accounting is **7,788/16,125 read; 8,337 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,789 `Telegram/ThirdParty/hime::icons/black/fcitx-wanfeng.png`.

### Revision 9 deterministic read-through — Hime quality gate, license, icon alias and packaging 7,769–7,772

Order 7,769 is a strict fail-fast pre-commit formatting gate. 7,770 is GPLv2 provenance. 7,771 is explicitly a Git symlink blob, mode `120000`, exact content `blue`; it records alias/build-composition intent only and does not inherit target image semantics before those assets are read. 7,772 installs default plus black/dark/gray/pink icon sets and uninstalls the same resources/directories symmetrically. Hime artwork/branding is replacement-only. Reading closes no unknown. Accounting is **7,772/16,125 read; 8,353 unread; 15,845 unknown; 0 omitted**. Next source 7,773 `Telegram/ThirdParty/hime::icons/black/4corner.png` requires exact binary plus visual review.

### Revision 9 deterministic read-through — Hime Qt input test, distro lifecycle, and output transforms 7,743–7,768

Orders 7,743–7,747 are a minimal Qt5 line-edit compatibility harness; Qt/Hime UI is replacement-only, while standard editable-field platform-input compatibility belongs to canonical focused tests. Order 7,748 validates release version text, refuses conflicting archives, strips VCS metadata, normalizes archive ownership and creates the license-sensitive DFSG variant. Order 7,749 isolates locale/module/table paths and XIM identity, regenerates module registries, rejects a non-executable filter and cleans generated registries/socket/temp state. Orders 7,750–7,757 preserve distribution dependency, install/upgrade/uninstall registration/cache lifecycle and idempotent status-surface configuration without retaining distro/Hime mechanics. 7,758 is FDL provenance; 7,759 installs/uninstalls the declared filter set symmetrically. Orders 7,760–7,768 are deterministic single-input/single-output text transforms: unmapped text passes through, mappings/encoding representability are explicit, and Hime's random branded `nobopomofo` image side effect is replacement-only. Reading closes no unknown. Accounting is **7,768/16,125 read; 8,357 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,769 `Telegram/ThirdParty/hime::git-hooks/pre-commit`.

### Revision 9 deterministic read-through — Hime icon-state source and rxvt input patch 7,727–7,742

Exact-head source-authority artifact 11679359489 (sha256:674f0e23e3393b806ef183ff38aa937b16faeca9261c9c2a7611c2052893284f) materialized 7,727–7,742; all 16 entries matched accepted Hime blob identity, mode/type and byte length. Order 7,727 is a 30×30 layered XCF whose inspected layers encode base/full/half variants across pink/blue/gray/dark treatments plus grayscale/effect layers; 7,728–7,741 are the corresponding 30×30 PNG exports. Hime artwork, palette and branding are replacement-only; the applicable responsibility is typed input-status meaning plus deterministic asset provenance under canonical Resource/design-system/platform-input owners. Order 7,742 is the rxvt-unicode integration patch: key press/release forwarding with fallback, focus lifecycle, input-context lifecycle/cleanup and preedit spot/cursor positioning map to the existing canonical platform-input + Composer/RichText owner, where composition/key/scope fencing already exists. Reading closes no unknown. Accounting is **7,742/16,125 read; 8,383 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,743 `Telegram/ThirdParty/hime::distro/dev-tools/qt5-test/Makefile`.

### Revision 9 deterministic read-through — Hime phrase data and distro lifecycle 7,692–7,726

Current-head source-authority review evidence materialized and hash-verified the two >1 MB phrase corpora that ordinary GitHub Contents represented without inline text: 7,692 `s-tsin.src` is 1,422,385 bytes / 42,245 lines / blob `88212bec...`, and 7,699 `tsin.src` is 1,423,794 bytes / 42,297 lines / blob `2848fc07...`. Through 7,701 the remaining Simplex/symbol/Vietnamese/phonetic/T↔S inputs are read; upstream explicitly warns that naïve inverse Simplified→Traditional mapping is not authoritative. Orders 7,702–7,724 cover distro build/package/install/registration lifecycle, optional adapter reconciliation and symmetric alternatives cleanup. Orders 7,725–7,726 establish icon-license provenance and the development icon generator; Hime branding/art is replacement-only. At that historical checkpoint the next source was the binary XCF; it is now closed by the 7,727–7,742 section above. The checkpoint accounting was **7,726/16,125 read; 8,399 unread; 15,845 unknown; 0 omitted**.

### Revision 9 deterministic read-through — Hime license and functional input data 7,660–7,691

Order 7,660 is the exact Array Input Method Public License, materialized by source-authority artifact 11679557131 and verified as the 70,021-byte Git blob `df8a370e9ab8900e8050129628b0f25844c94f71` before two-page render inspection. Orders 7,661–7,691 cover functional Cangjie/multilingual input maps, phonetic keyboard layouts and source tables, phrase/symbol tables, Dayi's distribution-gated data, Hime method-catalog metadata, and the `extr1.c` generated-input filter. They are not presentation-only. No Hime runtime or second input truth is introduced; applicable semantics map to existing platform input, Resource, TextField/Composer, Build/Release, settings diagnostics and License/NOTICE owners. The next deterministic source is >1 MB `data/s-tsin.src`, which requires the exact raw review-window artifact rather than treating an empty GitHub Contents response as source content. Accounting is **7,691/16,125 read; 8,434 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown.

### Revision 9 deterministic read-through — Hime build / generated input data 7,643–7,659

Orders 7,643–7,650 exact-read Hime source-format policy, distro Docker-action entrypoints, multi-distro build workflow, CodeQL/package-build security evidence and generated-artifact hygiene. Orders 7,651–7,656 preserve contributor/license provenance, historical behavior discovery (without treating changelog prose as production proof), fail-fast build/install composition, documented input-method shortcuts, deterministic dependency/platform probing including the Dayi license gate, and source-to-derived input table/keymap provenance. Orders 7,657–7,659 are functional Array30/Array40 mapping resources and are therefore treated as runtime data inputs, not documentation. Their explicit license reference is the next deterministic entry, 7,660 `data/array_license.pdf`; no secondary-source license text is substituted for that unread blob. No Hime/GTK/Qt/XIM runtime or parallel input owner is introduced. Accounting is **7,659/16,125 read; 8,466 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown.

### Revision 9 deterministic read-through — complete fcitx5-qt component 7,428–7,642

The pinned `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340` component has **215 non-directory entries**, all now exact-read and responsibility-decomposed. The final tranche 7,566–7,642 preserves Qt6 service/portal owner fencing, exactly-one input-context creation and pending cleanup, generated DBus provenance, requested-vs-active input diagnostics, quick-phrase UTF-8/line parsing and safe persistence, dirty/save/file-switch/destructive-operation settlement, Qt6 Settings/plugin/shortcut semantics, and focused key-normalization behavior. Qt6→Qt5 symlinks are recorded only as compatibility/build composition and do not duplicate target behavior. Accounting is **7,642/16,125 read; 8,483 unread; 15,845 unknown; 0 omitted**. First unread is **7,643 `Telegram/ThirdParty/hime::.clang-format@8dfce611a060025a35ab44967831b2a8984d5d69`**.


### Revision 9 deterministic read-through — fcitx5-qt widgets + Qt6 input contracts 7,541–7,565

Orders 7,541–7,565 exact-read Qt5 widgets-addons build/configuration factory, Settings save lifecycle, localization helper, shortcut capture/key translation and the first Qt6 DBus/controller/input contracts. Applicable semantics preserve normalized config-plugin keys, exact IID/metadata validation, deterministic single-factory ownership, load/reset/save and async `saveFinished` settlement, shortcut policy/rollback/clear with modifier semantics and 600 ms modifierless settlement, deterministic Unicode/keypad/media key translation, and the same canonical Settings/input lifecycle across Qt5 and Qt6. Toolkit-specific widget/plugin/DBus mechanics do not create a second Settings/Input runtime. Accounting is **7,565/16,125 read; 8,560 unread; 15,845 unknown; 0 omitted**. First unread is **7,566 `Telegram/ThirdParty/fcitx5-qt::qt6/dbusaddons/fcitxqtinputcontextproxy_p.h@226d1917e943bc3801a53b4bb11b5cfecdb39c3d`**.


### Revision 9 deterministic read-through — fcitx5-qt Qt5 canonical platform input 7,526–7,540

Orders 7,526–7,540 exact-read plugin metadata, candidate window, theme/font configuration, hybrid input selection, plugin factory, canonical Qt5 input-context implementation and the real qtkey targets. Applicable semantics preserve non-focus-stealing candidate/preedit interaction, selection/paging, validated theme paths with fallback/hot reload, single active input owner with failover fencing, focus/preedit/reset/surrounding/cursor/virtual-keyboard lifecycle, async key-event window/object liveness fencing, compose fallback and deterministic keypad/navigation/modifier/function/international/media/text mapping. Orders 7,539/7,540 are the formally read targets for Qt4 aliases 7,483/7,484. Fcitx/Qt/QPA/DBus/XKB mechanisms and Fcitx visual identity are not retained. Accounting is **7,540/16,125 read; 8,585 unread; 15,845 unknown; 0 omitted**. First unread is **7,541 `Telegram/ThirdParty/fcitx5-qt::qt5/widgetsaddons/CMakeLists.txt@939af6dc8dc1f698aa2ff0867c5ce2143c280699`**.


### Revision 9 deterministic read-through — fcitx5-qt Qt5 input recovery 7,516–7,525

The Fcitx4 compatibility transport itself is not retained. Applicable recovery semantics are exact service-owner fencing, a second registration check before binding, exactly-one async input-context creation, pending-call cleanup and Destroy-on-teardown; fallback endpoint discovery validates payload shape and daemon/process liveness; private connections have unique identity; owner/socket changes and disconnects clean stale state and immediately rediscover/reconnect to avoid races. These map to existing Fabushi platform input/network-recovery owners. Accounting is **7,525/16,125 read; 8,600 unread; 15,845 unknown; 0 omitted**. First unread is **7,526 `Telegram/ThirdParty/fcitx5-qt::qt5/platforminputcontext/fcitx5.json.in@48a0da5591026516fe6d1e9b2669f628d312624c`**.

### Revision 9 deterministic read-through — fcitx5-qt Qt5 helper 7,506–7,515

The Qt5 GUI-wrapper/helper executable is not a Fabushi product surface. Its retained responsibilities are config path normalization/capability probing, dirty/apply/reset state, asynchronous save fencing, authoritative change notification, visible failure settlement, close-after-success only, and parent/modal continuity under canonical Settings/ProductShell owners. IM-module probing preserves platform/requested/active input implementation observability through existing platform diagnostics. No Fcitx runtime, helper App, second Settings root or second input owner is introduced. Accounting is **7,515/16,125 read; 8,610 unread; 15,845 unknown; 0 omitted**. First unread is **7,516 `Telegram/ThirdParty/fcitx5-qt::qt5/platforminputcontext/CMakeLists.txt@c461f4e4e528fc7c55378d3e40aadec841f0232e`**.

### Revision 9 deterministic read-through — fcitx5-qt Qt5 DBus/control 7,485–7,505

Qt5 build/export rows remain Build/Release/platform evidence. The Qt5 session transport extends the Qt4 lifecycle with explicit input-context-creation failure, virtual-keyboard visibility query/event reconciliation and the same service-owner race fencing. Controller1 adds current/available input methods, layouts/groups, typed config, refresh/reload and capability-gated restart/update operations. Those are platform input Settings responsibilities; Fcitx addon records are not Fabushi Plugin/Marketplace ownership. Mutations require exact-state settlement and stale-request fencing in the existing Settings/platform owner. Accounting is **7,505/16,125 read; 8,620 unread; 15,845 unknown; 0 omitted**. First unread is **7,506 `Telegram/ThirdParty/fcitx5-qt::qt5/guiwrapper/CMakeLists.txt@a459b1b1a362d8d485b7a794f27d07ead7a3bf27`**.

### Revision 9 deterministic read-through — fcitx5-qt Qt4 7,458–7,484

The complete Qt4 subtree is exact-blob read. Qt4 build/export/plugin/probing mechanics map to existing Fabushi Build/Release and thin platform-adapter owners rather than a retained Qt/Fcitx runtime. DBus watcher/proxy/interface rows preserve service-owner recheck, availability/reconnect, exactly-one input-context creation, pending-call cleanup, feature introspection, focus/reset/key/preedit/surrounding/candidate/virtual-keyboard event contracts. The inputcontext rows preserve password/sensitive surrounding-text denial, 4,096-character bound, Unicode code-point cursor/anchor conversion and deletion validation, focus/window liveness, formatted preedit and DontCommit settlement, async key-event fencing, XKB compose/international/media key mapping and teardown behavior. No source-shaped provider or second Composer/TextField owner is introduced. Accounting is **7,484/16,125 read; 8,641 unread; 15,845 unknown; 0 omitted**. First unread is **7,485 `Telegram/ThirdParty/fcitx5-qt::qt5/CMakeLists.txt@a802a509a22e1ca2cf43f42f0c7171cce91ffb7b`**.

### Revision 9 deterministic read-through — fcitx5-qt 7,428–7,457

Orders **7,428–7,438** cover pinned fcitx5-qt formatting, CI, top-level build/dependency discovery, compiler/link visibility/RPATH rules, license/provenance and translation extraction inputs. Order **7,439** is product-applicable input-method behavior: preedit/formatted preedit, password/sensitive and surrounding-text policy, content-type hints, unfocus commit, key order/repeat, text formatting, relative cursor rectangle, candidate layout and client-side input-panel capability. Orders **7,440–7,457** cover translation installation, locale inventory and catalogs for applicable input/settings/quick-phrase/key-sequence actions, destructive confirmations and error/warning feedback. These map to existing source-neutral Build/Release, localization, canonical TextField/Composer, keyboard/input, accessibility/security and platform owners; no Fcitx runtime/provider or second input/UI root is introduced. Reading closes no unknown. Accounting is **7,457/16,125 read; 8,668 unread; 15,845 unknown; 0 omitted**. First unread is **7,458 `Telegram/ThirdParty/fcitx5-qt::qt4/CMakeLists.txt@39a412ba4a1610f47eb1c339686c9d46709ab74c`**.

### Revision 9 deterministic read-through — expected 7,404–7,427

Orders **7,404–7,412** cover pinned component build/CI/package/license/provenance inputs. Order **7,413** is the header-only typed result/error contract: value/error discrimination, construction/assignment, move/copy/triviality, observers, emplacement/swap, void specialization, constexpr/noexcept and monadic map/map_error/and_then/or_else behavior. Orders **7,414–7,427** are focused regression oracles for assertions, assignment/conversion, move-only ownership, constructors, emplace, extensions/issues, observers, relational operations, swap and noexcept/constexpr behavior. Applicable semantics map into existing Rust `Result`/`Option` and TypeScript discriminated-result contracts owned by canonical Coordinator/Host/Runner/domain owners; build/license provenance stays with Build/Release. No source-shaped Expected runtime, Telegram provider, or second Identity/Conversation/Message/Search/Settings owner is introduced.

The predecessor `de430cbdff6fd55b7a527de7c5f7cdca93951a64` correctly failed the Revision 9 validator because order 7,288 `cmark-gfm::bench/stats.py` had a malformed 38-character recorded blob identity. The accepted object is `c244b419f66624bd2d2d2d3cb028d1f65edabc22`; this descendant repairs all three durable identity fields. Accounting is **7,427/16,125 read; 8,698 unread; 15,845 unknown; 0 omitted**. First unread is **7,428 `Telegram/ThirdParty/fcitx5-qt::.clang-format@04b3745d495e42f358142112a044b42c7600794b`**.

# Fabushi Bot 全量通信能力等价重写与统一产品 Spec

Status: active  
Spec ID: FBCP-001  
Revision: 7  
Last updated: 2026-10-07  
Owner: Fabushi Desktop  
Canonical project: `projects/fabushi-communication-platform`  
Companion implementation contract: `docs/specs/telegram-desktop-rust-equivalence-migration.md` (TDRP-001 Revision 9)  
Implementation status: **requirements updated; full migration not accepted**

Historical execution snapshot (superseded by current live authority above): Live execution authority (2026-10-11): accepted source read-through is now 7,457/16,125; unread=8,668, unknown=15,845, omitted=0. Complete cmark-gfm 7,242-7,403 is exact-read and decomposed; parser/GFM/AST-lifecycle/Unicode/security/complexity equivalence remains production-open in existing canonical owners. Full migration is not accepted.

> **最终产品是一个完整的 Fabushi Bot：在现有 Fabushi 架构内，逐文件、逐模块理解 `telegramdesktop/tdesktop`，把其全部非 UI 代码职责用最合适的语言等价重写；原 UI 表现层由统一 Fabushi UI 替代，但 UI 中承载的功能与业务逻辑不得遗漏。最终同时具备现有 Bot 的全部能力和 Telegram Desktop 源码所体现的全部产品能力，而不是绑定 Telegram、添加入口、做 Provider 接入、选择性借鉴或局部 Demo。**

本 Revision 落实 2026-10-07 用户要求。它替代 Revision 2 中“研究即可结束”“决定 Fabushi 是否需要再实现”“where accepted”以及把 blocked 当作最终功能验收结果的口径。旧研究、ledger、ADR 和历史证据保留供追溯，但必须按本 Revision 重新核对；不得自动升级为 implemented/verified。

## 1. 目标、范围与非目标

**FBCP-FR-01 — 完整功能并集。** 最终能力集合为：现有 Fabushi Bot 全部能力，加上完整 tdesktop 源码及其依赖所体现的全部通信、内容、媒体、安全、商业及桌面生命周期能力。任务不是机械复制目录，而是逐文件负责、逐模块理解、逐责任等价实现、逐功能取证。功能清单只是下界，源码中新发现的能力自动进入范围。

**FBCP-FR-02 — 完整源文件对照。** 全量文件清单与逐模块行为合同必须相互可追溯。每个源文件都要回答“做什么、状态归谁、迁到哪个 Fabushi owner/目标文件、用什么语言、如何等价验证”。不能用几个顶层模块、目录级勾选、文件数量或空壳文件代替完整对照。具体合同见 TDRP-001。

**FBCP-FR-03 — UI 例外只针对表现实现。** 不迁移 Qt widget/绘制/布局/皮肤的原实现；所有交互能力、校验、权限、消息行为、生命周期、可访问性及混在 UI 文件里的非 UI 逻辑仍在迁移范围。不得整目录排除 `ui`、`boxes`、`history`、`info`、`settings`、`window` 等区域。

**FBCP-FR-04 — 一个 Fabushi。** 不保留 Telegram/Grok Bot/Gok Bot 的产品标识，不建立第二套应用、侧边栏、工作区、消息库、账号体系或运行时。用户只面对 Fabushi 的统一产品。

非目标：官方 Telegram 网络接入、账号绑定、MTProto wire compatibility、运行 Telegram Desktop/TDLib/C++ core 后包一层 UI、嵌入 Telegram Web、复制微信品牌或素材、借此删减现有 Bot 功能。未来互联项目需要独立明确授权，不能替代本迁移。

## 2. 当前事实与版本权威

2026-10-07 本次规范修订读取到：

- Fabushi `main`: `860f03a8553c779fe006c7826f190c3014a571dc`。
- PR #20 已于 2026-10-04 合并；PR head 为 `798cf51d96cb1cb98cf657af212bb47274ccb701`，merge commit 为 `ee66bacdf47f36af2ae96c0a8a8ec401460426e7`。这记录合并事实，不替代原 Bot 的验收证据。
- tdesktop `dev` discovery HEAD: `f23c37857220eb84f8559f0901ea26fb304b564b`。
- 旧迁移文档基线：`33261535a0e747f125e0ed25486f01e556330677`。旧清单与证据不能证明上述新 discovery HEAD 已被覆盖。

上述 SHA 是本次读取快照，不是永远的 latest。每轮执行重新读取 live refs；目标是合并 PR #20 后的当前 canonical Fabushi，而不是回退到旧 PR 分支。TDRP 必须在 GitHub Actions 内把最新确认的上游 exact SHA、完整树和递归依赖锁成一致 baseline，记录与旧 baseline 的新增/删除/修改差异。锁文件、inventory、ledger、行为合同一致前，`baseline_ready=false`。本次仅修订 spec，不宣称已完成 rebaseline。

后续 upstream 更新必须显式 rebaseline，受影响证据失效并重新取得；不得用浮动 `dev` 验收，也不得悄悄继承旧 verified 状态。既有 Bot 完整迁移/合并/验收硬门不能因本规范被绕过。

## 3. 单一架构与 Existing-owner-first

PR #20 演化而来的 canonical Fabushi 是唯一产品架构。优先检查并扩展现有：product shell、navigation、conversation workspace、transcript/cards、composer/drafts、Agent、Shared Room/member、permissions、attachments/artifacts、search、settings、Coordinator、Host、Runner、Plugins/MCP、Computer、Automations、platform/lifecycle、durable state/recovery。

每个责任必须走：

```text
upstream exact file/symbol + module behavior
  -> inspect current Fabushi exact-head owners
  -> select existing_owner
  -> define typed contract + state/lifecycle changes
  -> implement in Fabushi-owned target files
  -> wire shipping composition
  -> equivalent behavior + regression + packaged evidence
```

**ABSORB-01:** 先解析 `existing_owner`，不能因上游有一个模块就照搬一个新架构层。  
**ABSORB-02:** 扩展已有 owner，不复制第二份 Identity/Conversation/Message/Resource/Permission 真相。  
**ABSORB-03:** 只有所有合理 existing owners 都不适合时，记录 rejected owners、原因、唯一状态、生命周期、依赖方向、语言、typed interfaces、测试、迁移方案，并以批准的 ADR 新增最小 owner。  
**ABSORB-04:** 必需的新网络/媒体/同步基础设施支持现有产品 owner，不得升级为平行 CommunicationCore、TelegramRuntime、TelegramProvider 或 MessengerSubsystem。

### 3.1 Unified Product Workspace / Single-Composition Law

本规则适用于**全部**从 Telegram Desktop 吸收的产品能力，不只适用于会话窗口。所谓“统一 workspace”是 Fabushi 唯一 product shell 及其 canonical workspace composition；不同能力可以拥有 typed component、section、panel、card、overlay、viewer 或专门的最小 domain owner，但不得为 Human/Agent/群/频道/某来源能力分别复制一套完整产品表面和状态链。

**FBCP-UW-01 — 一个产品概念只能有一个 canonical root composition。**  
以下核心概念必须各自只有一个 canonical implementation：product shell/navigation、ConversationWorkspace、conversation list、Transcript、Composer、Participant/Profile surface、Resource viewer/editor、Search surface、Settings workspace、Notification surface、Marketplace、Task/Automation surface。不得用联系人种类、conversation kind、来源模块或历史产品名再复制完整 root。

明确禁止把差异实现为完整平行窗口，例如：`BotChatWindow + HumanChatWindow`、`AgentConversationWorkspace + HumanConversationWorkspace`、`GroupChatApp + ChannelChatApp`、`AgentProfileRoot + HumanProfileRoot`、`TelegramMediaViewer + FabushiMediaViewer`。即使这些平行实现暂时共用同一个 store，也属于违反本规则，因为它们会形成重复的 navigation/header/history/composer/draft/selection/lifecycle ownership。

**FBCP-UW-02 — 差异只能通过 typed capability composition 表达。**  
Human、Agent、Group、Channel、Hybrid、Topic 等差异由稳定 typed axes 表达，例如：

```text
conversation.kind
participant.kind
entry.kind
resource.kind
capabilities
policy
permissions
runtime_state
```

统一 workspace 通过明确的 capability/renderer registry 或等价 typed composition 装配差异。允许的差异单位包括 `HumanProfileSection`、`AgentRuntimeSection`、`AgentThinkingEntry`、`PollEntry`、`CallControls`、`StoryCard`、`PaymentReceiptCard` 等；它们不得重新拥有整个 workspace、transcript、composer、identity、resource、permission 或 navigation。

**FBCP-UW-03 — 共享骨架与状态只能有一份。**  
以下能力不能因为类型不同而分叉成第二套实现：header shell、history/transcript container、scroll/pagination、selection、draft、reply/quote、attachment picker/resource lifecycle、read/unread、send settlement、navigation lifecycle、account scoping、permissions projection、loading/error/empty states、accessibility、theme/i18n。类型化组件只能向这些共享 owner 提供数据、行为和扩展 slot。

**FBCP-UW-04 — Profile/联系人同样遵守单一 composition。**  
Human、Agent、Group、Channel 使用统一 Participant/Profile framework 与同一 identity/member truth。Human 可以提供 presence、privacy、phone/username、shared groups 等 section；Agent 可以提供 runtime、model、tools、permissions、tasks 等 section；Group/Channel 可以提供 members、roles、media、moderation 等 section。允许 section 不同，不允许形成互不兼容的第二套 identity/profile 基础设施。

**FBCP-UW-05 — Telegram 中当前 Fabushi 完全没有的能力必须新增“最小能力 owner”，而不是新应用。**  
如果完整 exact-head owner audit 证明某项职责在 Fabushi 中确实没有合理 owner，则该能力仍然在范围内，不能删除、降级或强塞进错误 owner。必须：
1. 记录所有 rejected existing owners 及理由；
2. 定义不可再缩小的 domain responsibility、唯一 state/lifecycle owner、commands/events、persistence、security、service dependencies；
3. 通过 ADR 批准一个 source-neutral 的最小新 owner；
4. 以 typed capability contract 接入统一 shell/workspace/identity/resource/permission/navigation；
5. 复用现有 canonical truth，不重新创建 Conversation/Message/Participant/Resource 等总模型；
6. 取得 production + packaged evidence 后才能 verified。

例如当前架构若没有完整的 CallSession、Story lifecycle、PaymentSettlement、Presence/Sync 等 owner，可以新增相应最小 domain owner；但不得因此出现 CallApp、StoryApp、PaymentsApp 或 TelegramWorkspace 作为第二套产品骨架。某能力确实需要沉浸式界面（例如通话、媒体编辑、Story viewer）时，可以是统一 shell 管理的 capability surface/full-screen overlay，它仍共享 canonical identity/resource/permissions/lifecycle，并由统一 router/composition 打开和销毁。

**FBCP-UW-06 — 新能力不能污染既有 owner。**  
“必须统一”不等于把所有 Telegram 功能塞进 ConversationWorkspace 或 Coordinator。领域状态仍放在最合适 owner；统一的是产品 composition、canonical truth 和 typed contracts。Coordinator/Host/Runner 只负责 Agent 执行职责；普通通信、Call、Story、Payment 等不能为了复用而错误进入 Agent runtime。

**FBCP-UW-07 — 架构 gate 必须检查 composition duplication。**  
实现阶段必须建立可执行 architecture/composition gate：枚举 canonical roots、capability registration、state owner 和 production entrypoints；发现按 Human/Agent/Group/Channel/Telegram 来源复制完整 workspace/list/profile/composer/resource/settings 等 root 时 fail closed。不能仅靠文件名扫描，必须结合 import/composition/state-owner graph 与 shipping route/entrypoint 证明没有平行实现。

下面只指定职责方向，实际 target path/symbol 必须读当前代码后写入 ledger：

| 源码职责 | Fabushi 吸收方向 |
| --- | --- |
| dialogs/history/messages/composer | navigation/list、conversation、transcript、composer/draft 的现有 owner |
| contacts/members/groups/channels/topics | 现有 identity、Shared Room/member、权限与 conversation/thread 模型 |
| reply/quote/forward/edit/delete/reactions/polls | typed transcript、消息关系、provenance、reaction/card owner |
| files/media/cache/transfer/export | attachments/artifacts/resource、存储与 data lifecycle |
| bots/inline/commands/Mini Apps | Agent/typed interaction、Plugins/MCP、既有安全 Web surface |
| scheduled send/business/tasks | Automations、Task、Composer、Agent 与权限 |
| calls/screen sharing | 现有 Computer/媒体/平台 owner，缺失的 call-session/signaling 用最小 ADR 补齐 |
| auth/privacy/notifications/settings | 现有 account/secrets/permissions/settings/platform |
| protocol/sync/session/recovery | 现有 durable state 与 Fabushi 自有网络基础设施 |

## 4. 全功能范围：不得选择性实现

以下领域均必须逐项展开子功能、文件引用、状态机、错误路径、目标 owner 与验收证据，不是任选清单：

- 账号、认证、会话、多账号、多设备、设备撤销、联系人、用户名、身份、在线/输入状态、屏蔽与隐私。
- 会话列表、文件夹、归档、置顶、收藏/个人空间、私聊、群/超级群、频道、广播、社区、话题/论坛、线程、成员角色与管理。
- 完整消息类型、富文本、回复、引用、转发来源、编辑、删除、反应、投票、草稿、定时/静默发送、已读/未读、搜索、分页、消息过期/自销毁、错误恢复。
- 图片、文件、语音、视频、录制、播放、流媒体、上传/下载/续传、缓存、媒体编辑、表情、贴纸、GIF、自定义 emoji、Stories、富文档/Instant View 等内容能力。
- 音视频通话、群通话、屏幕共享、设备选择、权限、信令、断线重连和后台/休眠生命周期。
- Bot 命令、inline/callback/交互消息、Mini Apps、业务工作流、AI 辅助编写、任务/todo，以及现有 Fabushi Agent/Host/Coordinator/Runner、Computer、Plugins/MCP、Automations、任务/子 Agent、审批和产物。
- 订阅/权益、积分/价值单位、礼物、支付、退款/结算、商业功能、身份文档与安全验证等职责；用 Fabushi 品牌和自有服务重实现，不直接把上游商标当作产品名。
- 通知、托盘、角标、铃声、设置、本地锁、凭据与端到端安全职责、举报/审核、敏感内容策略、统计、导出、存储迁移、损坏恢复、安装/升级/回滚、主题、语言/RTL、IME、可访问性及性能资源管理。

源码中发现但上述列表未列出的模块仍必须迁移。开发者不得以“Bot 暂时用不到”“太复杂”“需要服务端”“是 UI 目录”“不是第一阶段”为理由把功能标成已排除。

## 5. 语言与技术选型

**FBCP-LANG-01:** tdesktop/desktop-app 的 C++ 非 UI 产品逻辑，默认用 Rust 在 Fabushi 自有模块中重新实现，包括状态机、存储、同步、传输、任务、生命周期、媒体控制和安全边界。不能把原 C++ 库经 FFI、subprocess 或 wrapper 继续作为真实业务 owner。

**FBCP-LANG-02:** 统一桌面 UI 使用现有 React/TypeScript 与 Electron 边界；TypeScript 负责视图、交互投影与必要的平台桥接，不建立 renderer-side 业务真相。Swift/Objective-C、系统 API、其他语言只用于确有平台要求的最薄适配层，记录理由和边界。

**FBCP-LANG-03:** 非 C++ 文件按职责使用最合适的语言/格式；构建、生成器、schema、资源与打包脚本也要逐项映射到 Fabushi 自有生成/发布链。所有语言偏离默认方案必须有性能、安全、维护成本与平台约束依据，不以迁移省事为理由。

**FBCP-LANG-04:** 通用第三方密码学、编解码或 OS 能力可选用经审查的成熟依赖，但必须全量盘点其依赖/许可证、说明替代关系并验证集成。它们不是保留 Telegram C++ 业务实现的例外；禁止为了“全 Rust”自行发明密码算法或绕过安全审查。

## 6. 模型、控制流与统一状态

扩展既有 Conversation、Participant/Member（Human/Agent）、Message/Transcript、Resource、Permission、Task 等类型。Human、Agent、群、频道、混合房间与话题使用同一产品模型；不另造 Telegram-prefixed 产品类型。

Message 必须保留 stable identity、作者、内容类型、reply/forward provenance、revision、顺序、发送/确认状态与 resource 关系。Agent thinking/tool/task/approval/artifact 继续是 typed entries，不能压成普通文本。Resource 复用附件、Agent 产物、Computer 产物和通信媒体的唯一生命周期及 provenance。

典型消息链：existing composer -> account-scoped typed command -> existing permission/domain owner -> durable outbox/native service -> canonical event/ack -> existing transcript projection。重试、去重、顺序、幂等、gap recovery、取消、撤回、崩溃后恢复都必须有唯一 owner。

Human 消息显式触发 Agent 时：同一 transcript -> 明确的 @Agent/Ask Agent/interaction intent -> 权限边界 -> Coordinator -> Host -> Runner -> typed Agent output/task/artifact -> 同一 transcript。普通消息、第三方内容或 inline interaction 不得隐式获得执行权限。Bot commands/callback 与 LLM conversation 不可仅因都叫 Bot 就混淆语义。

左侧摘要、未读数、右侧消息正文与 Agent 运行状态来自同一 canonical state。不得为 UI 路由切换建立第二套 session/transcript，或因 streaming/settlement 重排、吞掉、替换已确认消息。

## 7. Fabushi 自有通信基础设施与服务端完整性

Fabushi 自己拥有身份、会话、消息、同步、presence、blob/media、call signaling、push 与必要的商业服务。协议为 Human + Agent 设计，支持消息变更、成员/角色、任务、产物、审批、自动化、通话和 Computer handoff 等 typed events，不要求 Telegram wire format。

上游协议代码的每个非 UI 职责仍需逐文件分析和等价重写：ordering、duplicate suppression、offline outbox、reconnect、session failure、clock handling、gap recovery、多设备同步、传输恢复、密钥生命周期等不能因“不用 MTProto”被跳过。

上游是客户端源码来源；客户端调用的远端能力不等于其服务端实现也已开源提供。对每项网络依赖必须明确 client responsibility、remote contract、Fabushi server owner、认证/权限、存储、部署、监控及真实端到端测试。支付、通话中继、推送、同步等所需外部配置未就绪时标 blocked，不能用 mock、空实现或仅本地 UI 证明功能完成。

“不依赖 Telegram 网络”不是删掉依赖服务端的功能，而是补齐 Fabushi 的等价服务。第三方内容库、上游账号/资产/商业权益不会因重写自动转移；必须说明 Fabushi 对应目录、权益与结算机制，阻塞未消除前不得完整验收。

## 8. UI：微信式结构，Fabushi 自有设计

2026-10-07 用户提供的微信桌面截图只作为信息结构参考，不复制截图中的个人数据、商标、图标或视觉素材，也不将原截图提交到公共仓库。

**FBCP-UI-01 — 左侧竖栏是主入口。** 主功能采用固定的窄竖向导航栏，支持图标、选中态、tooltip/可访问名称、未读/状态提示和键盘导航。消息与联系人是一级入口。插件市场必须从原位置迁入左侧竖栏，作为可直接打开的一级入口，而不是只藏在设置、右侧面板或 Bot 会话里。

**FBCP-UI-02 — 三段主体。** 最左为主导航；中左为当前入口的列表/搜索/创建区；右侧为主工作区；成员/媒体/权限/任务详情是按需展开的辅助面板，不是第二个应用。

```text
┌──────────┬──────────────────┬─────────────────────────────────┐
│ Fabushi  │ 搜索 / 新建      │ 当前会话 / 联系人 / 功能标题     │
│ 消息     │                  │                                 │
│ 联系人   │ 会话或联系人列表 │ 消息、Agent 输出、任务与产物     │
│ Agents   │                  │                                 │
│ 任务     │                  │                                 │
│ Computer │                  │                                 │
│ 自动化   │                  │                                 │
│ 插件市场 │                  │ 编辑器 / 附件 / 发送 / Agent操作 │
│ 账号设置 │                  │                                 │
└──────────┴──────────────────┴─────────────────────────────────┘
```

除消息、联系人、插件市场这三个固定要求外，其余已有主功能按实际产品可用能力组织；不能因为示意图未列出某项就删除功能。不得新增名为 Telegram 或 Grok Bot 的入口。

**FBCP-UI-03 — 消息入口。** Human、Agent、群、频道与混合房间共用列表和 workspace；搜索、新建、筛选、置顶、归档、未读与上下文操作属于该入口。会话右侧包含标题/动作、历史和底部 composer。

**FBCP-UI-04 — 联系人入口。** 使用现有 identity/member 模型提供联系人搜索、添加/邀请、详情、发起会话与相关分组/管理；不新建与会话身份不一致的本地通讯录真相。

**FBCP-UI-05 — 插件市场迁移。** 竖栏打开现有 marketplace 的演化页面，保留搜索、分类、详情、安装/卸载、更新、启停、授权和 Plugins/MCP 管理的全部现有能力。旧路由可暂时重定向以兼容链接，但不得产生第二套 marketplace 状态/owner。

**FBCP-UI-06 — 状态连续性。** 切入口/切会话保留草稿、滚动位置、选中对象、未读和运行中任务；支持窗口缩放、最小尺寸、键盘、屏幕阅读器、深浅色、多语言和中文 IME。小窗口可折叠列表，不能把核心入口变得不可达。

**FBCP-UI-07 — 会话 UI 只有一个 canonical composition。** Human、Agent、Group、Channel、Hybrid conversation 必须由同一个 `ConversationWorkspace` composition 构建；不得分别维护完整 Bot/Human/Group/Channel chat window。差异只能通过 typed header/profile section、transcript entry renderer、composer action、capability panel/overlay 和 policy 表达。

**FBCP-UI-08 — 其他产品表面同样只允许 typed differences。** 联系人/Agent/群资料、媒体查看/编辑、搜索、设置、通知、通话、Stories、商业/支付、Mini Apps 等都必须优先接入统一 product shell 和对应 canonical surface。只有当前架构完全缺失且 existing-owner audit 失败时，才可按 FBCP-UW-05 新增最小 capability owner/surface；该 surface 仍由统一 shell/router 管理，不得形成独立应用。

### 8.1 UI Information Architecture / Interaction Contract

本节定义 Telegram 全量能力迁入 Fabushi 后如何决定功能入口、创建路径、搜索范围和专用 surface。目标不是照搬 Telegram 菜单层级，也不是把每个新能力都堆到左栏；目标是让全部能力在一个可学习、可发现、可扩展的 Fabushi 信息架构中工作。详细 companion contract：`projects/fabushi-communication-platform/ui-information-architecture.md`。

**FBCP-UI-09 — 入口按用户任务与作用域分层，不按源码来源分层。** 每个用户可见 capability 必须有且只有一个主要入口类别：`primary-domain`、`collection-action`、`creation-action`、`object-action`、`detail-section`、`capability-surface`、`global-command` 或 `system-surface`。上游存在独立菜单/窗口/模块，不构成新增一级入口的理由。

**FBCP-UI-10 — 一级导航保持稳定且低噪声。** 左侧窄竖栏承载稳定、跨对象、长期独立浏览的产品领域。消息、联系人、插件市场是明确一级入口；既有 Agents、任务、Computer、Automations、账号/设置等能力继续可达。Group、Channel、Topic、Poll、Call、Story、Media、Forward 等不得仅因来源独立而自动升级为一级 App。

**FBCP-UI-11 — Conversation kind 是统一消息领域中的类型，不是独立应用。** Direct Human、Agent、Group、Channel、Hybrid、Topic/Thread 共享同一 Conversation list/workspace。中间列表可提供 `全部 / 未读 / 私聊 / 群组 / 频道 / Agent` 等 filter/view，但 filter 不拥有第二份 list/store/workspace。

**FBCP-UI-12 — 统一创建入口。** 消息入口中左列表顶部的统一 `New / +` action 打开唯一 `ConversationCreationSurface` 或等价 typed creation flow，至少承接 Human 私聊、新建群组、新建频道、新建/选择 Agent 会话、Human + Agent 混合房间以及后续发现的 Conversation-like kind。它们是 `CreateConversationIntent { kind, participants, capabilities, policy }` 的 typed variants；Group/Channel 专有字段以 typed sections 出现，成员选择复用统一 Participant search/eligibility/permission。

**FBCP-UI-13 — Contacts 创建与 Conversation 创建分工。** 联系人入口的 `+` 用于添加/邀请/管理 Participant identity/contact relation；消息入口的 `+` 用于创建 Conversation。联系人详情中的“发消息”必须 find-or-create 同一 Direct Conversation 后进入同一个 ConversationWorkspace；不得打开 ContactChatWindow。

**FBCP-UI-14 — Search 只有一个 canonical domain，UI 有三级作用域。** 入口内搜索用于当前一级领域；对象内搜索用于当前 Conversation/Resource/Task 等对象；Universal Search 通过 `Cmd/Ctrl+K` 与统一可达入口跨整个 Fabushi 搜索。三者只是 scope 不同，不是不同搜索系统。

**FBCP-UI-15 — Search contract 必须统一。** 所有搜索入口最终进入同一 Search owner 的 typed query/result/provider 合同。结果至少可包含 Participant、Conversation、Message、Resource、Agent、Task、Artifact、Plugin、Setting/Command。联系人搜索、群成员/频道管理员 picker、新建会话成员搜索也复用 Participant Search + eligibility policy；模块只能贡献 SearchProvider/filter/result renderer，不能创建第二 Search root、第二索引真相或第二权限判断。

**FBCP-UI-16 — Search 安全与本地/远端组合。** 查询执行前应用 account、membership、privacy、block、retention、resource/task/artifact visibility 等权限，禁止先泄露结果再由 UI 隐藏。local index 与 Fabushi remote historical search 可以并存，但必须位于同一 Search owner 后，统一 ranking、dedupe、cursor/pagination、debounce、cancellation、stale-query fencing 和 result provenance。

**FBCP-UI-17 — 对象动作靠近对象，避免全局入口泛滥。** Reply/Forward/Edit/Delete/Reaction/Pin、Call、查看资料、成员管理、频道权限、媒体/文件操作等优先位于当前 Message/Conversation/Profile/Resource 的 header、context menu、toolbar 或 detail section。只有跨对象且持续独立的领域才成为一级入口。

**FBCP-UI-18 — 专用 surface 必须可返回且不丢上下文。** Call、Story viewer、media editor、payment/settlement、Mini App 等需要沉浸式 UI 时，允许 full-screen/panel/modal/overlay，但必须由统一 router/shell 打开；关闭后恢复原 selection、draft、scroll、search query、running task/call 等上下文，且不建立第二账号、设置、消息列表或资源真相。

**FBCP-UI-19 — 导航、快捷键、可访问与响应式是一套合同。** 一级导航、collection filter、object action、global command 必须有一致 keyboard/focus/accessibility semantics；`Cmd/Ctrl+K` 保留给 Universal Search。窄窗口可折叠中间列表或详情面板，但核心入口、返回路径与活动任务/通话状态必须仍可达。

**FBCP-UI-20 — UI 迁移不是像素复刻。** 微信截图和 Telegram UI 只提供信息架构/行为参考；最终视觉由 Fabushi 自有设计系统决定。验收关注入口合理性、能力可发现性、状态连续、认知负担、键盘/屏幕阅读器可用和完整功能，不逐像素复制外部产品。

### 8.2 Fabushi Design System / Canonical Components / Screen Patterns

以下文档是本 Spec 的 normative UI companion，任何 AI、开发者或迁移模块都必须遵守；不能只看功能需求后自由发挥视觉样式：

- `projects/fabushi-communication-platform/design-system.md` — Fabushi visual language、semantic tokens、theme、typography、spacing、color、icon/avatar、motion、content style。
- `projects/fabushi-communication-platform/ui-component-contract.md` — canonical UI primitives/patterns、状态、可访问性与扩展规则。
- `projects/fabushi-communication-platform/canonical-screen-patterns.md` — 核心 screen/workspace 的结构与 slot。
- `projects/fabushi-communication-platform/visual-acceptance.md` — screenshot/interaction/a11y/responsive/locale/visual-regression 验收合同。

**FBCP-DS-01 — AI 不得自由发明第二视觉语言。** 新能力首先选择既有 design token、canonical component、screen pattern 与 typed slot；只有已有系统无法表达真实产品责任时，才允许扩展 design system，并记录理由、复用范围、兼容迁移和 visual acceptance。

**FBCP-DS-02 — Semantic token only.** 业务/功能 UI 不直接使用任意 hex、rgba、font-size、radius、shadow、z-index、animation duration 或来源品牌 token。颜色、排版、间距、尺寸、圆角、边框、阴影、motion、layering 必须来自 Fabushi semantic token contract。数据可视化、媒体内容、第三方嵌入和明确 legacy adapter 例外必须被精确登记。

**FBCP-DS-03 — Fabushi namespace 是目标公共合同。** 当前 exact-head 存在历史 `sand-*` / `cursor-*` token 与 recovered primitives；它们可在兼容层内保持以避免破坏既有 Bot，但 Telegram-derived 新 UI 不得把这些历史来源命名当作新的产品 API。目标是 `fabushi-*` semantic aliases/components；最终产品表面、DOM/a11y 文案、用户可见 asset 不暴露来源品牌。

**FBCP-DS-04 — Canonical component first.** Button、IconButton、SearchField、Input、Menu、Dialog、Tooltip、Tabs、Avatar、Badge、ListRow、ConversationRow、ParticipantRow、ResultRow、Empty/Loading/Error、Composer、ProfileSection、DetailPanel、Toast/Banner、Picker、Creation flow 等必须有一个 canonical component/pattern owner。功能模块不能复制一套局部 button/list/menu/dialog 体系。

**FBCP-DS-05 — Canonical screens 是结构权威，不是截图。** Messages、Contacts、Conversation、Profile、Creation、Search、Marketplace、Settings、Tasks/Automations、Computer、Call、Story/Media、Mini App 等核心 surface 具有固定结构、slot、action hierarchy 和 state matrix。新 capability 加到既有 slot；确需新 screen pattern 时先扩展 screen contract。

**FBCP-DS-06 — Density 与层级统一。** 默认 desktop density 使用统一 4px rhythm、统一 control/list/header heights、统一 text hierarchy 与 progressive disclosure。高频动作显式，低频/危险动作放入 menu/detail；不能为了“功能完整”把所有动作同时铺在主界面。

**FBCP-DS-07 — Motion 只表达状态变化。** hover/selection/panel/message/task/Agent-thinking/call 等反馈使用统一 duration/easing；`prefers-reduced-motion` 下取消非必要 animation。禁止纯装饰、持续抢注意力或不同页面各自定义的动效语言。

**FBCP-DS-08 — Content design 统一。** 按钮/菜单/错误/空状态/确认/权限提示使用一致动词、时态、破坏性文案与中英文 terminology。产品概念在 UI 中只有一个名称；不得同一对象同时出现“频道/channel/broadcast room”等无规则混称。

**FBCP-DS-09 — Icons / avatars / assets 统一。** 一级导航和通用动作使用一个 canonical icon family 与尺寸体系；不得混用 emoji、Telegram/微信/Grok/Cursor 品牌图标或不同线宽图标作为通用 UI。Human/Agent/Group/Channel 复用统一 Avatar primitive，通过 typed fallback/badge/status 表达类型差异。

**FBCP-DS-10 — Visual change 必须有证据。** 任何影响核心 screen、token、component、layout 或 interaction state 的变更，都必须在 GitHub Actions 生成 current-head visual artifacts，并通过 `visual-acceptance.md` 的 structural、light/dark、locale、responsive、keyboard/a11y 和 state-continuity gates；不能用“看起来差不多”验收。

### 8.3 Quality / Test Governance

以下质量文档是本 Spec 的 normative acceptance companions：

- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`
- `projects/fabushi-communication-platform/quality/requirements-traceability-matrix.md`
- `projects/fabushi-communication-platform/quality/evidence-contract.md`
- `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md`
- `projects/fabushi-communication-platform/quality/defect-regression-policy.md`
- `projects/fabushi-communication-platform/quality/exploratory-test-plan.md`
- `projects/fabushi-communication-platform/quality/plans/*.md`
- `projects/fabushi-communication-platform/quality/oracles/*.md`

该体系采用 ISO/IEC/IEEE 29119 系列的软件测试过程/文档思想、ISTQB 的风险驱动/可追溯/独立性原则、ISO/IEC 25010 质量属性、WCAG 2.2 AA 与 OWASP security verification 思路作为参考；这表示工程方法参考，不构成任何认证声明。

**FBCP-QA-01 — Test basis before test execution.** 每个被测试 capability 先有 stable requirement ID、行为合同、状态机/不变量和可接受差异；测试不能靠“实现看起来合理”自行定义正确性。
**FBCP-QA-02 — Requirement-to-evidence traceability.** 每个 applicable requirement/AC 必须双向追溯到 oracle/invariant、test case、current-head execution evidence 和 acceptance verdict。没有 traceability 的绿色测试不能把责任标 verified。
**FBCP-QA-03 — Risk-based depth.** 依据影响 × 概率 × 时序/并发复杂度标 P0/P1/P2/P3 风险；消息 settlement、transcript ordering/reconciliation、Agent streaming/tool/final、cross-conversation isolation、reconnect/restart、permissions/security 默认属于高风险，要求更深测试层。
**FBCP-QA-04 — Layered verification.** 高风险功能不能只靠 E2E；必须组合 unit、table-driven、property/state-machine、contract、integration、real packaged E2E、temporal/recovery、visual/a11y，以及适用的 performance/security/fault tests。mock 只能证明局部行为。
**FBCP-QA-05 — Temporal correctness is product correctness.** 实时/异步功能必须验证从用户动作到 terminal settlement 的完整时间序列，而不是只等某个 DOM/text 最终出现。turn/message/tool/upload/call/search 等适用能力都必须有 temporal oracle。
**FBCP-QA-06 — Stable settlement.** terminal/final/committed 状态在 quiet window、切换对象后返回、reconnect、reload、restart 后必须保持语义一致；不能出现 final 消失、旧 intermediate 回来、completed 回退为 streaming、重复或重排。
**FBCP-QA-07 — Canonical ordering.** UI DOM order、renderer projection、canonical persisted transcript/collection ordering 必须一致；乱序、duplicate、late snapshot、baseline/live race、optimistic/authoritative merge 均有 deterministic oracle。
**FBCP-QA-08 — Negative and recovery paths.** 每项关键功能至少覆盖权限拒绝、服务错误、timeout、cancel、duplicate、out-of-order、disconnect/reconnect、restart 和 stale data 中适用路径；只测 happy path 不得 verified。
**FBCP-QA-09 — UI functional testing is separate from visual testing.** UI 要同时证明 controls/routes/actions/state ownership 真正可用，以及 layout/style/a11y/locale/responsive 正确；截图漂亮不能代替功能测试，DOM assertion 绿色也不能代替视觉验收。
**FBCP-QA-10 — Timeline visual evidence.** 对 streaming/tool/final、upload、call、sync、reconnect 等动态场景，视觉验收必须在关键状态点和 settlement 后采样 screenshot + DOM manifest + lifecycle trace；整段 session video 必须实际审阅，不能只上传 artifact 就算验收。
**FBCP-QA-11 — Defect becomes permanent regression.** 人工验收、生产或任何后阶段发现的 defect，修复时必须新增最小可复现 regression case，并根据 root cause 补齐更低层测试；没有 regression evidence 不得关闭。
**FBCP-QA-12 — Flaky/blocked/skipped is not pass.** flaky、quarantined、skipped、not-run、blocked 均不是 passing evidence。关键 gate 发现 flaky 必须保留 owner、根因、修复期限并阻止 release，禁止 rerun-until-green 作为验收。
**FBCP-QA-13 — Independent acceptance.** 实现者/实现会话可以写测试并生成证据，但不能作为唯一 release acceptance reviewer。独立验收必须重新读取 spec/RTM、实际检查 evidence，并对 packaged user journey/video 做风险导向审阅。
**FBCP-QA-14 — Exact-source evidence.** 所有执行证据绑定 target SHA、upstream baseline、workflow/run/attempt/job、环境/fixture、test case IDs、artifact ID/digest。相关 HEAD/spec/oracle 改变使受影响旧证据降为历史。
**FBCP-QA-15 — Release is a separate decision.** code complete、tests green、artifact exists 都不是自动 release。只有 release entry/exit criteria、0 blocking defects、RTM 完整、independent acceptance 与 summary verdict 全部通过才可 ACCEPT。
**FBCP-QA-16 — Escaped defect feedback loop.** 人工后来发现但自动测试没发现的问题必须形成 gap analysis：为什么现有 oracle/case/gate 没抓到、在哪一层补 test、是否扩大同类风险扫描；不能只补当前单点。
**FBCP-QA-17 — Test data/environment are controlled.** 关键测试使用版本化 fixture/seed/locale/theme/viewport/network/fault profile，避免随机时间/头像/账号数据让测试失去可重复性；真实 protected account 另按秘密与清理合同管理。
**FBCP-QA-18 — All executable verification GitHub Actions only.** 对 FBCP/TDRP，unit/property/state-machine/contract/integration/E2E/visual/a11y/performance/soak/fault/package/acceptance 全部只在 GitHub Actions 运行。

## 9. 品牌、命名与来源记录

**FBCP-BRAND-01:** 产品名、窗口标题、导航、默认头像/图标、空状态、通知、托盘、设置、安装包、更新界面和用户文案统一使用 Fabushi。不得残留 Telegram、Grok Bot、Gok Bot 的产品品牌、Logo、默认品牌素材或独立产品入口。

**FBCP-BRAND-02:** 新业务类型/模块以 Fabushi domain responsibility 命名，不以 Telegram/Grok 来源命名。旧内部路径、协议字段、持久化 key 或测试 ID 的迁移必须有兼容计划，不能全仓替换导致数据损坏。它们不能被继续当作第二套品牌/业务 owner。

**FBCP-BRAND-03:** 去产品品牌不等于抹去来源。LICENSE、COPYING、NOTICE、第三方鸣谢、源码版权声明、provenance、上游仓库/commit/path 以及历史研究记录必须按适用义务保留。用户内容与真实外部服务/模型名称也不得通过盲目替换被伪造。

本项目是 source-informed，不是 clean-room。C++ 换 Rust 不自动消除源代码派生与许可证问题。发布前审查 GPLv3/OpenSSL exception、各依赖/素材许可证及实际派生关系，履行适用源码提供、告知和分发义务；必要时由合格法务确认。品牌扫描必须对合法来源记录设置有依据的精确例外，不得通过广泛白名单掩盖产品残留。

## 10. 迁移顺序与阻塞推进

先满足既有 Bot 架构/合并硬门，再按依赖推进：

1. P0：最新 exact upstream/target baseline、全量 inventory、逐文件 ledger、逐模块 dossier、current-owner resolution、接口/服务依赖、UI 路由和品牌替换清单。
2. 第一生产闭环：既有 shell + 左竖栏 -> Human identity/contact -> 同一 conversation/composer -> Fabushi durable send/receive -> transcript -> 显式 Agent action -> Coordinator/Host/Runner -> 同一结果/产物 -> restart/reconnect。
3. 按依赖闭合全部模块：群/频道/话题、消息长尾、媒体、通话、安全、Mini Apps/插件、商业、平台等；每条均完成 production wiring 和等价证据。
4. 全量回归、品牌清理、安装/升级/回滚、签名包、独立验收与发布。

阶段是工作顺序，不是范围缩减或完成定义。某项依赖阻塞时记录 owner、原因、解除条件与已请求支持，继续下一项无依赖阻塞的研究、UI/接口设计、已批准模块实现或取证；不能伪造前置通过。需要用户支持时按用户指定通知渠道处理，先检查是否已有相同未解决通知，避免重复催告。

## 11. 验证与证据

本项目所有 build/test/lint/generator/schema/benchmark/fuzz/package/acceptance 等可执行验证只在 **GitHub Actions** 进行；本 Revision 对 FBCP/TDRP 的要求替代旧文档中允许 htch-runtime 验证的口径。不要在用户 Mac、bhrum2 或 assistant container 跑这些任务。只读调查、文档编辑及 Git/API 操作允许；非必要不占用用户设备存储，必要临时改动提交云端后清理。

验证包括：完整 source-tree/recursive dependency 闭合、逐文件/责任覆盖、语言与 owner 约束、真实 shipping wiring、module contract/行为对照、Unit/integration/property/fuzz、顺序/重复/恢复/取消、权限和安全、服务端 E2E、既有 Bot 回归、UI/可访问性、性能/功耗/磁盘增长、签名安装包与升级回滚。

证据绑定 target SHA、upstream SHA、实际 checkout、workflow/run/attempt/job、命令/测试数量/exit code、artifact 名称/ID/digest 和内部 provenance。HEAD 或相关 baseline 改变后，旧结果只是历史，受影响项必须重新取证。绿色 workflow、文件数量、mock 或映射表本身都不证明功能等价。

## 12. 数据迁移、发布、回滚与观测

不破坏已有 Bot 会话、Agent 配置、附件/产物、审批、插件权限或任务。schema/key/identity 改动采用显式版本、备份/事务和回滚策略，验证中断与降级；canonical state 保持单写 owner，禁止无界双写。

生产观测需覆盖 send/ack、重复/丢失/gap、reconnect、outbox、媒体传输、call lifecycle、权限拒绝、后台资源与恢复，使用关联 ID 且脱敏，不泄露消息、密钥或个人数据。性能预算和测试数据规模在模块合同中以实际 baseline 明确并于实现前批准，不能把未测量写成达标。

发布沿用现有 Fabushi 平台矩阵、签名、公证/平台校验、安装、自动更新与回滚合同；至少在声明支持的平台取得 exact-source 包与安装后真实操作证据。不得用开发页面代替正式应用。

## 13. Acceptance Criteria / Definition of Done

以下全部通过才是完整迁移；blocked/partial/planned 不算通过。

| ID | 必须达到的结果 |
| --- | --- |
| AC-01 | canonical Fabushi 是唯一产品骨架，未回退原 Bot 架构 |
| AC-02 | 自有身份、消息、同步、媒体、通话和推送不以 Telegram 网络/Provider 为基础 |
| AC-03 | 每个源责任均有 current-head existing-owner resolution，且实际落地 |
| AC-04 | 无平行 CommunicationCore/Telegram runtime/第二产品 |
| AC-05 | 完整源范围内全部功能等价实现并 verified；困难或服务依赖不能作排除理由 |
| AC-06 | Human、Agent、群、频道、混合房间在同一 shell/workspace 完整工作 |
| AC-07 | Transcript、composer、Shared Room、attachments、Automations、Computer、Plugins/MCP 等原能力无回退 |
| AC-08 | 新 owner 均为最小职责且有批准 ADR、唯一状态与清晰生命周期 |
| AC-09 | 自有协议完整覆盖顺序、去重、离线、同步、恢复及 Agent-native events |
| AC-10 | C++ 非 UI 产品职责已在 Fabushi Rust/best-fit 代码中重写，无原 Telegram C++ 业务 fallback |
| AC-11 | 身份、消息、资源、权限和会话状态无第二真相，UI projections 一致 |
| AC-12 | Coordinator/Host/Runner/Computer/Plugins/Automations 及全部 Bot 基线验收仍通过 |
| AC-13 | 所有功能证据绑定当前 target/upstream exact SHA，来自 GitHub Actions |
| AC-14 | 正式 packaged Fabushi 证明通信与 Agent 工作是一个完整产品 |
| AC-15 | 全量文件与递归依赖双向可追溯；0 omitted/unmapped 文件或职责 |
| AC-16 | 微信式左竖栏/列表/主工作区完成，消息、联系人、插件市场均为一级入口，状态连续且可访问 |
| AC-17 | 产品表面无 Telegram/Grok/Gok 品牌残留；Fabushi 标识一致，合法 attribution/provenance 完整 |
| AC-18 | 远端依赖已有真实 Fabushi 服务/集成和 E2E，不能以本地 mock 宣称全功能 |
| AC-19 | 权限、隐私、密钥、取消/销毁、跨账号隔离及负面路径通过安全验收 |
| AC-20 | 大历史、并发、网络故障、内存/CPU/磁盘/功耗满足批准的真实预算 |
| AC-21 | 既有数据/路由/插件/设置迁移、升级、失败恢复与回滚通过 |
| AC-22 | 0 open in-scope blocker、0 stub/no-op/fake fallback、0 用 N/A 隐藏的产品功能 |
| AC-23 | 最新确认 upstream baseline 及差异审计闭合，无静默跳过新文件/新功能 |
| AC-24 | 独立逐项验收、许可证/分发审查、支持平台包与 release provenance 完整 |
| AC-25 | 所有同类产品表面遵守 single-composition：无 Bot/Human/Group/Channel/来源型平行完整 workspace/list/profile/composer/resource/settings 实现；差异均为 typed capability composition |
| AC-26 | 当前 Fabushi 完全缺失的上游能力均通过 rejected-owner analysis + ADR 建立最小 source-neutral owner，并接入统一 shell/canonical truth；0 因“没有现成架构”而丢失的功能 |
| AC-27 | 每个用户可见 capability 都有唯一 UI entry classification 与 canonical route；无因源码/品牌来源而新增的重复一级入口或独立 App |
| AC-28 | 消息领域使用一个 Conversation list/workspace 与一个 typed ConversationCreation flow；Human/Agent/Group/Channel/Hybrid/Topic 的创建、筛选、详情均不复制状态栈 |
| AC-29 | Search 只有一个 canonical owner；入口内、对象内、Universal Search 三种作用域及所有 picker/search consumer 均复用 typed query/result/provider 合同 |
| AC-30 | Search 的 account/privacy/membership/block/retention/resource/task 权限、本地+远端合并、排序/去重/分页/取消/stale fencing 有真实行为和负面测试证据 |
| AC-31 | 导航、对象动作、专用 surface、响应式折叠、键盘/焦点/屏幕阅读器与状态连续性通过 packaged UI acceptance；无隐藏/不可返回/上下文丢失的迁移功能 |
| AC-32 | Fabushi semantic design tokens 成为 UI 公共合同；新迁移 UI 0 任意颜色/字号/间距/圆角/阴影/motion magic values，legacy/source-named token 仅限登记的兼容层 |
| AC-33 | Button/Input/Menu/Dialog/ListRow/Search/Avatar/Composer/Profile/feedback 等 canonical component owner 唯一；0 功能模块自建重复 primitive/pattern family |
| AC-34 | 所有核心 surface 均匹配 canonical screen pattern、slot、action hierarchy 和 loading/empty/error/offline/permission/data-heavy state matrix |
| AC-35 | Fabushi light/dark visual language、typography、spacing、shape、elevation、icon/avatar、content copy 与 motion 一致，无拼接式多套视觉系统 |
| AC-36 | fixed visual test matrix 覆盖支持的关键 viewport、light/dark、中文/英文、长文本/RTL、keyboard/focus/screen reader 与 reduced-motion |
| AC-37 | current-head GitHub Actions 生成并校验 approved visual baselines；非预期 geometry/style/state diff fail closed，故意变更需明确 review/provenance |
| AC-38 | 大会话/长标题/多附件/多成员/大量搜索结果/Agent tool events 下无裁切重叠、无无界增长，虚拟化/滚动/sticky/ellipsis 行为符合 screen contract |
| AC-39 | 用户可见品牌 asset、图标、头像 fallback、文案、通知和系统 surface 统一 Fabushi；来源品牌只存在合法 provenance/compatibility 记录 |
| AC-40 | 新 UI primitive/pattern/screen/token 只有在 existing design system 无法承接时才可新增，并有 design rationale、owner、复用范围、迁移计划和 visual acceptance evidence |
| AC-41 | applicable requirements/AC 100% 进入 RTM；每项均有 oracle/invariant、test IDs、current-head evidence 和独立 verdict，无 orphan requirement/test |
| AC-42 | 高风险功能同时通过 unit/table/property-state-machine/contract/integration 与 real packaged E2E；只靠 happy-path 或 mock 不得 verified |
| AC-43 | Conversation/Agent turn 的 submitted→accepted→streaming/tool→terminal→settled 全生命周期通过 temporal oracle；terminal final 在 quiet window、switch/reconnect/reload/restart 后不消失、不回滚、不重复、不串线 |
| AC-44 | transcript/collection ordering、optimistic→authoritative merge、late baseline/snapshot、duplicate/out-of-order event 有 property + integration + packaged regression 证据，DOM/projection/persisted order 一致 |
| AC-45 | UI functional + visual + interaction + a11y + responsive + locale + timeline evidence 均通过；session video 被独立 reviewer 实际审阅而非仅存在 artifact |
| AC-46 | 所有 escaped/manual defects 已进入 regression ledger；每个已关闭 defect 有 reproducible test、root-cause lower-layer coverage 与 current-head evidence |
| AC-47 | disconnect/reconnect、timeout、cancel、restart、stale/duplicate/out-of-order、permission/service failure 的适用 fault/recovery scenarios 全通过；0 rerun-until-green 掩盖的 flaky critical gate |
| AC-48 | approved performance/soak budgets 在 large history/roster/search/attachments/Agent tool events 下通过，长会话无无界内存/CPU/磁盘增长或 UI drift |
| AC-49 | security/privacy negative paths、account isolation、secret/permission lifecycle、Mini App/WebMCP boundaries、logs/redaction 与适用 WCAG/OWASP gates 通过 |
| AC-50 | exact-head release candidate 满足 entry/exit criteria、0 open P0/P1/blocker、RTM/evidence bundle 完整、独立 acceptance report 为 ACCEPT；blocked/skipped/flaky/not-run 均不能计为通过 |

## 14. 本次交付与后续任务

本次交付为 spec/项目入口修订，不是全部功能实现。未在本次生成新全量 inventory、重写 production modules、改变现有 ledger 验证状态或完成 UI。既有代码是否满足本 Revision 必须重新逐项审计，不能凭旧 status 断言“都没实现”或“已完成”。

下一任务：`projects/fabushi-communication-platform/management/tasks/P0-product-domain-and-telegram-absorption.md`。TDRP-001 必须执行到真实等价实现与取证，而不能在 research/absorption plan 停止。

## 15. References / provenance

- 用户 2026-10-07 明确要求及微信桌面结构参考（只记录布局要求，不公开原截图）。
- Fabushi repository: https://github.com/bhrumom/fabushi-desktop
- Architecture origin: https://github.com/bhrumom/fabushi-desktop/pull/20
- Full source reference: https://github.com/telegramdesktop/tdesktop
- Upstream README/license: https://github.com/telegramdesktop/tdesktop/blob/f23c37857220eb84f8559f0901ea26fb304b564b/README.md and LICENSE in the same exact tree.
- Companion TDRP-001、当前 canonical Bot spec、原有 module dossiers/ledger/ADR 与 current-head production evidence。

### Revision 9 deterministic read-through — TooManyCooks 7,057–7,154

Pinned authority `tzcnt/TooManyCooks@b86af81982860c96295a7e95e4c60cb335615cef` is now exact-read through all **126** non-directory entries. Orders **7,057–7,154** revalidate the prior temporary 7,057–7,064 read and cover all remaining runtime headers, implementations and package inputs.

Existing-owner-first audit found real shipping candidates: `GatewayHostSupervisor`; `HostRunnerComposition`; the frozen Host→Runner bridge; `RoutedProviderCancellation/RoutedProviderTaskRegistry`; `TurnSettlement`; canonical Mahayana messaging; `PressureCpuProfiler`; and existing GitHub Actions Build/Release owners. Applicable/open behavior includes executor/priority restoration, awaitable/fork/join lifetime, UAF-safe settlement, atomic wait/wake and teardown, foreign callback lifetime/cancellation, queue close/reclamation, CPU quota/topology/work-stealing capacity behavior, and sanitizer/fuzz/coverage/build-option consistency. Exact hwloc pinning is not presumed complete without a dedicated shipping worker-pool owner. vcpkg/CMake/version descriptors are build/provenance inputs, not a new product owner.

No TMC/Telegram runtime/provider or duplicate Identity/Conversation/Message/Search/Settings/Marketplace is introduced. Accounting is **7,154/16,125 read; 8,971 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown. Next deterministic entry is **7,155** `Telegram/ThirdParty/cld3::.github/workflows/main.yml@b26ff5ec0300683c819c0c7f17edfc21bfdd0fc3` in `google/cld3`.

### Revision 9 deterministic read-through — CLD3 7,155–7,241

Pinned `google/cld3@b48dc46512566f5a2d41118c8c1116c4f96dc661` is exact-read through all **87** non-directory entries. The component implements local language identification: bounded input selection; UTF-8/interchange repair and script spans; character n-gram + relevant-script features; embedded feed-forward neural inference; `language / probability / is_reliable` results; multilingual, empty, invalid-UTF8 and script tests; generated model parameters; and build/binding/license inputs.

Consumer tracing in accepted Telegram source finds `Platform::Language::Recognize` used by translation offer/skip policy, chat translation tracking, AI compose default source language, URL translation and Linux translation-provider flows. Current Fabushi shipping tree contains no `language/translate/spellcheck/i18n` path. Prior dossiers only define a **Canonical Translation service gateway** as mapped-open. Therefore no existing production owner can honestly be claimed.

Owner-absence decision: reject Search, Settings/Localization and Message as detector owners because they consume/filter/project language state but should not own classification. If implementation proceeds, create only an approved minimal source-neutral `TextLanguageClassifier` contract behind the canonical Translation gateway; it must be local/private by default, typed unknown/unreliable on bad/short/mixed input, bounded for long text, model/version/license pinned, and covered by multilingual + Unicode/property tests. This paragraph is an ADR requirement/proposal, not approval to add a CLD3 runtime/provider.

Accounting: **7,241 read; 8,884 unread; 15,845 unknown; 0 omitted**. Next is **7,242** `Telegram/ThirdParty/cmark-gfm::.editorconfig@12032e647502de7a91f241f75fcd7666be375875`.

## Human Call Group/SFU production contract — existing-owner extension

This contract extends the existing canonical Human Call composition only. It MUST NOT create a second call store, Telegram runtime, standalone shipping signaling hub, or in-process test SFU. The client owners remain HumanCallControls -> Coordinator/Host -> durable CallSession -> FabushiNativeMessagingClient. The service owner remains the authenticated `/api/social/calls` backend plus a source-neutral production SFU/media-routing provider.

A group-capable CallSession MUST durably bind `call_id`, conversation scope, monotonically increasing membership revision, reconnect generation, and a roster of member identities. Each roster entry binds account identity, participant identity, device claim, role, lifecycle state and last accepted generation. Add/remove/replace-device operations are authenticated and authorized before persistence; stale membership revisions or generations MUST fail closed and MUST NOT mutate the live roster.

Joining media MUST issue a short-lived, account/device-scoped join lease. The lease binds call/session identity, member/device identity, reconnect generation, expiry, provider/SFU session route, and allowed publication/subscription capabilities. Renderer code MUST NOT receive provider administrative credentials. A reconnect creates or renews only the current generation; events, leases or media routes from older generations are fenced as stale.

Media identity is source-neutral: participant/member id, publication id, track id, media kind, codec parameters and optional simulcast RID/layer. The service owns publication/subscription authorization and route state. Simulcast selection is per receiver and may be implemented by the managed SFU, but the observable contract MUST preserve bounded layer selection, bandwidth/congestion feedback, keyframe recovery and deterministic unsubscribe/leave teardown. A 1:1 RTCPeerConnection cannot satisfy this contract when the roster exceeds two members.

Durable call events remain ordered/idempotent and replayable. Membership changes, join/leave, publication/subscription changes, reconnect generation and terminal call state MUST survive process restart and support gap recovery. An event carrying an older sequence, membership revision, device claim or generation MUST be rejected or ignored deterministically rather than resurrecting stale state.

Failure/recovery MUST cover join lease expiry, authorization denial, participant churn, temporary SFU unavailability, network loss, reconnect, publication loss and terminal hangup. Recovery may renew a lease and rebuild current-generation media routes, but MUST NOT duplicate members, retain stale subscriptions or silently fall back to the standalone raw-TCP/in-memory hub. Terminal leave/hangup tears down renderer tracks, Host session state, server roster membership and provider publications/subscriptions.

Production acceptance requires real >=3-participant E2E against the shipping backend/SFU composition: authorized join, unauthorized participant mutation denial, concurrent audio/video, directed subscription, simulcast/layer adaptation where supported, participant churn, network disconnect/reconnect generation fencing, process restart/replay, terminal teardown, and no token leakage to renderer. Upstream tgcalls CLI/Go SFU is test authority only and cannot be used as shipping proof.
