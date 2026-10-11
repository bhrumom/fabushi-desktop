Live Revision 9 authority (2026-10-11): telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (root tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6). Root non-directory=6,653; recursive non-directory=16,125; deterministic read-through=8,824; first unread=8,825 Telegram/ThirdParty/hunspell::tests/hu.aff@8345e113130a8aa47dc8fa8e759443eb9f06bdf4; unread=7,301; unknown=15,845; omitted=0. Orders 8,749–8,824 exact-read malformed-compressed-input safety, command/text framing, morphology/dynamic-dictionary memory safety, malformed-affix no-crash handling, Unicode/casing/circumfix/fullstrip behavior, suggestion self-loop rejection, REP/regex/oversized-table bounds, slash/flag parsing, alias/homonym morphology, Unicode digit tokenization, derivational stemming, bounded n-gram processing, UTF-8 affix conditions and explicit personal-dictionary path semantics. All rows remain mapped-open/unknown_closed=false/omitted=false unless separately backed by shipping production evidence. Source-authority run 38105710749 job 114370739825 artifact 11689521875 at 9f6ea014ffa7a33bc21f547a6ad79a4c802c83bd supplied exact raw bytes for 8,749–8,812; this descendant accounting/production head requires fresh exact-head Actions. gh1058 self-suggestion rejection is implemented source-neutrally in the existing canonical Electron context-menu owner; no Hunspell/Telegram runtime or second dictionary owner is introduced.
<!-- TDRP_CURRENT_SUMMARY read-through=8824 unread=7301 unknown=15845 omitted=0 first-unread=8825 path=Telegram/ThirdParty/hunspell::tests/hu.aff -->

### Revision 9 deterministic read-through — Hunspell robustness/morphology regression window 8,749–8,824

Orders 8,749–8,812 were exact raw-byte materialized by GitHub Actions run `38105710749`, job `114370739825`, artifact `11689521875` at `9f6ea014ffa7a33bc21f547a6ad79a4c802c83bd` (artifact digest `sha256:70664b45fadb9af72b43dc45c02602161ce27ef49d4a7032fddc7731ce6ab22b`). The previously unread binary/non-UTF entries were independently checked: order 8,749 `gh-hunzip-overflow.aff.hz` matches Git blob `aa6cb05899ed69bb7b295d915669f79f14df4700` and SHA-256 `6369b66257fe542ae733984937110c8b9f1f6752876cd67ead2c20b7bb8d1640`; order 8,766 `gh1044.aff` matches Git blob `e49cfdd84a9c566501770f1c816a6d330c3bd407` and SHA-256 `d77119833b6d1d9a7b03a1469ccd0aa6dafe02fa44a22a2d93991b95980b5ea2`. Orders 8,813–8,824 were exact-read directly from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`.

The responsibility groups cover fail-closed compressed/affix parsing, ispell command-vs-text framing, OCONV/suggestion correctness, morphology alias robustness, dynamic dictionary add/remove ownership, UTF-8 case derivation, no-self-suggestion, prefix/circumfix/FULLSTRIP/NEEDAFFIX semantics, bounded REP/regex/table handling, slash-as-WORDCHARS parsing, AF/AM alias round-trip, homonym-aware morphology rewriting, Persian/Arabic digit tokenization, derivational stem non-duplication, bounded n-gram work, UTF-8 affix conditions, and explicit personal-dictionary path independence from HOME/USERPROFILE.

Existing-owner-first remains mandatory: parser/UAF/overflow/fuzz responsibilities map to the platform spellcheck dependency + security/Resource boundary; text behavior maps to the canonical text-input/spellcheck Unicode/morphology/suggestion owner; explicit dictionary path semantics map to canonical Resource/settings. No Hunspell engine, Telegram provider, sidecar, second editor, second spell settings owner, or second dictionary store is introduced. The `gh1058` responsibility is implemented on the current branch by filtering a native suggestion equal to the normalized misspelled word before canonical context-menu projection; the rest remain mapped-open pending source-neutral production/replacement evidence.

Accounting is **8,824/16,125 read; 7,301 unread; 15,845 unknown; 0 omitted**. First unread is order 8,825 `Telegram/ThirdParty/hunspell::tests/hu.aff@8345e113130a8aa47dc8fa8e759443eb9f06bdf4`.


### Revision 9 deterministic read-through — Hunspell forbidden/case/full-strip/German compounding 8,730–8,748

Orders 8,730–8,748 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`, with repository/path/blob identities cross-checked against the source-authority 8,730–8,793 manifest from GitHub Actions run `38104688586`, job `114367624265`. The range completes the FORBIDDENWORD dictionary and good/wrong oracle: forbidden roots propagate to suffixed/compound forms while permitted homonyms remain valid. It also fixes FORCEUCASE behavior for compound members and suggestions, FULLSTRIP whole-root affix removal, and current/legacy German compounding requirements covering compound begin/middle/end roles, Fuge elements, ONLYINCOMPOUND/COMPOUNDPERMITFLAG, case-boundary rejection, dash composition, forbidden exceptions, and legacy circumfix positioning.

These are source responsibilities, not evidence that Fabushi now embeds Hunspell. They map existing-owner-first to the canonical platform-backed text-input/spellcheck, morphology and dictionary boundary and remain `mapped-open`, `unknown_closed=false`, `omitted=false`. The native Electron spelling-suggestion projection implemented on the current PR is a narrower product interaction and does not close these morphology/compound semantics.

Accounting is **8,748/16,125 read; 7,377 unread; 15,845 unknown; 0 omitted**. Order 8,749 `Telegram/ThirdParty/hunspell::tests/gh-hunzip-overflow.aff.hz@aa6cb05899ed69bb7b295d915669f79f14df4700` is intentionally still first-unread: it is a non-UTF-8 compressed regression blob, and the connector's UTF-8 file reader rejected it. The adjacent harness proves the responsibility is malformed compressed-dictionary input must not crash, but that does not justify falsely marking the binary blob itself read; raw-byte evidence is still required.


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

# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-10-07  
Spec: TDRP-001 Revision 9  
Parent: FBCP-001 Revision 7  
Complete: false

| Area | Status | Note |
| --- | --- | --- |
| Product role | active | tdesktop is complete source authority, not a runtime/provider |
| Discovery upstream | observed | `f23c37857220eb84f8559f0901ea26fb304b564b`; not yet accepted baseline |
| Historical baseline | stale | `33261535a0e747f125e0ed25486f01e556330677`; evidence cannot be inherited |
| Recursive source closure | rebaseline required | current lock/inventory must be reconciled to one exact upstream baseline |
| File-level inventory | not accepted | must reach unknown=0, unread=0, omitted=0 |
| Responsibility decomposition | not accepted | every source symbol/responsibility requires traceable destination |
| Exact-head owner inventory | in progress | existing-owner-first remains mandatory |
| UI composition / IA | in progress | one shell, one Conversation/Search/Profile/Marketplace truth |
| Design-system migration | in progress | new migrated UI must use Fabushi semantic tokens/components |
| Native service completeness | in progress / blocked where absent | blocked service is not functional acceptance |
| Production implementation | partial | existing slices do not imply full migration |
| Existing Bot regression | required | Agent/Host/Coordinator/Runner/Plugins/MCP/Computer/Automations/Tasks etc. must remain green |
| Quality/RTM gates | not fully implemented | Revision 9 schema/validator fields and traceability still require executable gates |
| Packaged acceptance | fail-closed required | acceptance failure must block release |
| Independent release acceptance | not accepted | final verdict must be ACCEPT |

## Current live reconciliation

On 2026-10-07, `main` was observed at `e0a7a4221970088beeaf145d0d9be1763e58229f`. That commit weakened the real signed packaged acceptance step with `continue-on-error: true`, which conflicted with FBCP Revision 7 / TDRP Revision 9. The regression was removed on `main` by commit `2115a91eb888aa65702ef8f90f01848042bb65c7`.

This status file records only observed repository state. It is not completion evidence. Any newer `main` supersedes these SHAs and must be re-read before implementation or acceptance.

## Metrics

Report separately:
- accepted upstream baseline completeness;
- source file read coverage;
- responsibility understanding coverage;
- existing-owner resolution coverage;
- production implementation coverage;
- exact-head test/evidence coverage;
- packaged temporal acceptance;
- independent release acceptance.

Do not report Telegram-provider, Telegram-network, survey, document count, or vertical-slice progress as migration completion.


### cf478d37 live-authority rebaseline

Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

Direct delta: channel_earn.style replaces static negative placeholder margins with floating placeholderShiftLeft; lib_ui 91ff446 implements horizontal interpolation in InputField and MaskedInputField. Canonical TextField remains the sole product owner. Orders 5,774-5,793 were identity/order reconciled; reading alone closes no unknown.


### Revision 9 live read-through note: 5794-5801

Orders 5,794-5,797 preserve professional evidence-log one-line/completion-forgery integrity plus an independent raw-byte oracle. Orders 5,798-5,799 preserve complete mapped-target/viewport capture readiness. Orders 5,800-5,801 preserve the bounded reversible not-marking-read evidence lever. These are test/evidence responsibilities only, create no second product owner, and close no unknown. The new 811b83a1 readability delta is order 5,951 and remains unread/unknown.


### Revision 9 live read-through note: 5802-5805

Orders 5,802-5,803 close the exact-blob read/decomposition of popup/context-menu professional evidence semantics: same-turn fresh-menu identity, explicit refusal taxonomy, isolated QAction queued-callback delivery, prepared-frame capture and lock/teardown behavior. Orders 5,804-5,805 close the SentMessageWatcher client→server id reconciliation contract with stable-history/candidate fencing and a five-second diagnostic probe throttle. These are test/evidence responsibilities only and close no unknown. First unread is now 5,806 `test_notify_override.cpp`.

### Revision 9 live rebaseline/read-through note: b0d1fe5e / 5166 / 5806-5813

The 811b83a1→b0d1fe5e upstream delta changes only order 5,166 `media_view_overlay_widget.cpp`: the old `forbidsSaving()` control-refresh branch hid controls and could call `DocumentSaveClickHandler::Save(...ToCacheOrFile)` when media was not loaded; b0d1fe5e removes that viewer-driven automatic persistence path. The new blob `e35f67119323850a3cea450548566186c8a2231c` was re-read and the mapped-open MediaViewer lifecycle responsibility now explicitly requires protected/forbids-saving media not to be auto-persisted as a rendering/control-refresh side effect. Path order and total counts are unchanged. Separately, orders 5,806–5,813 are read-complete/responsibility-decomposed professional test harnesses for reversible notify overrides/no-server-write locality, document-open handoff inspection without OS launch, SeparatePanel pointer/liveness evidence, and post-paint temporal sampling. They add no product owner, fake fallback, unknown closure or release credit. First unread is 5,814 `test_probe.cpp`; read-through is 5,813/16,125, unread 10,312, unknown 15,846, omitted 0.


### Revision 9 live read-through note: 5814-5821

Orders 5,814–5,821 are exact-blob read-complete/responsibility-decomposed professional evidence harnesses. Probe preserves mark-bounded, keyed issue→answer correlation with explicit ambiguity/outstanding states and positive controls; controlled RPC fixtures reject malformed/truncated/trailing/wrong-constructor buffers before live parser delivery; retry evidence distinguishes 500 retry registration from 400 fail/unregister and unknown-request controls while logging only code/type/constructor; Runner preserves bounded stage/watchdog execution, explicit N/A, QPointer/paint readiness, exactly-once onFinish teardown, disposable-copy markers, completion drain and post-quit fuse behavior. These rows add no product owner or fake fallback and close no global unknown. First unread is 5,822 `Telegram/SourceFiles/test/test_scenario.cpp`; read-through is 5,821/16,125, unread 10,304, unknown 15,846, omitted 0.

### Revision 9 live read-through note: 5822-5824

Orders 5,822–5,824 are exact-blob read-complete/responsibility-decomposed professional evidence code. The checked-in scenario slot is deliberately no-op/test-only. The secrecy oracle is fail-closed over accepted launch identities, controls, bidirectional MTP headers and canaries; distinguishes client-written plain/Send from report-only Recv, withholds unsafe sites, narrowly handles declared computed fields, emits only public evidence and rescans its own rows. Missing/foreign/unreadable/control/canary/input failures are undecided rather than clean. These rows close no global unknown and create no product owner. First unread is 5,825 `Telegram/SourceFiles/test/test_style.cpp`; read-through is 5,824/16,125, unread 10,301, unknown 15,846, omitted 0.

### Revision 9 live read-through note: 5825-5830

Orders 5,825–5,830 are exact-blob read-complete/responsibility-decomposed UI evidence harnesses: stable-window style/baseline verification, real InputField text-drop delivery/refusal/ancestor shielding with no clipboard-content access, and narrowly-scoped U+00A0/U+202F text-read normalization with deliberate negative controls. They create no production owner and close no global unknown. First unread is 5,831 `Telegram/SourceFiles/test/test_toast_capture.cpp`; read-through 5,830/16,125, unread 10,295, unknown 15,846, omitted 0.


### Source closure through 5,848

Under live `65e23ba7` authority, orders **5,837–5,848** were completely read and exact-blob dispositioned. Accounting is now **5,848/16,125 read, 10,277 unread, 15,846 unknown, 0 omitted**; these test/evidence-only rows do not reduce global unknown. Fresh exact-head Actions are required.


### Source closure through 5,859

Under live `65e23ba7` authority, orders **5,849–5,859** are now exact-blob read-complete/responsibility-decomposed. Tests 5,849–5,854 are professional evidence-only responsibilities; Tray 5,855–5,859 remains mapped-open production work pending canonical source-neutral owner/composition and exact-head evidence. Accounting: **5,859/16,125 read, 10,266 unread, 15,846 unknown, 0 omitted**. First unread is 5,860 `ui/boxes/about_cocoon_box.cpp@4aab8e79…`. No baseline-ready or release credit is granted.


### Source closure through 5,871

Orders **5,860–5,871** (`ui/boxes`) are exact-blob read-complete/responsibility-decomposed. They remain mapped-open product/UI responsibilities; reading does not reduce unknown. Current accounting: **5,871/16,125 read; 10,254 unread; 15,846 unknown; 0 omitted**. First unread 5,872 `choose_font_box.cpp@56a7386c…`. No baseline/release credit is granted.


### Source closure through 5,909

The deterministic live prefix is exact-blob read-complete/responsibility-decomposed through **5,909**, completing `ui/boxes/**`. Accounting is **5,909/16,125 read; 10,216 unread; 15,846 unknown; 0 omitted**. First unread is 5,910 `ui/cached_round_corners.cpp@7da69a22…`. The predecessor d52b8c5 authority failure identified a stale `upstream.lock.json.source_disposition_evidence.read_through`; this descendant fixes that data contract without modifying or weakening the validator. No baseline-ready, accepted, implementation, or release credit is granted.


### Source closure through 5,980

Orders **5,910–5,980** are exact-blob read-complete/responsibility-decomposed. Accounting is **5,980/16,125 read; 10,145 unread; 15,846 unknown; 0 omitted**. First unread is 5,981 `ui/color_contrast.cpp@f981e717…`. Parent `9c969d1` repaired the validator-required narrative tokens; this batch preserves that exact contract. No baseline-ready, accepted, implementation or release credit is granted.


### Source closure through 5,991

Orders **5,981–5,991** are exact-blob read-complete/responsibility-decomposed for shared color semantics and canonical button busy/context-menu/two-label behavior. Busy presentation is not credited as duplicate-operation refusal; the command owner still needs an independent in-flight fence. Accounting: **5,991/16,125 read; 10,134 unread; 15,846 unknown; 0 omitted**. First unread 5,992 `call_button.cpp@af74333f…`. No baseline/release credit is granted.


### Source closure through 6,001

Orders **5,992-6,001** are exact-blob read-complete/responsibility-decomposed for call action/mute state projection, service Checkbox presentation, compose-AI/large-paste classification and custom-emoji Toast projection. They map to source-neutral canonical owners; no Telegram-named component is introduced. The shipping Human call owner now reuses canonical `SandButton` and gives mute a command-level in-flight fence with failure restoration and teardown, but broader call/composer/toast responsibilities remain mapped-open pending same-head evidence. Accounting: **6,001/16,125 read; 10,124 unread; 15,846 unknown; 0 omitted**. First unread 6,002 `delete_message_context_action.cpp@9cecf76e…`. Reading/partial implementation grants no baseline or release credit.


### Source closure through 6,011

Orders **6,002-6,011** are exact-blob read/decomposed for TTL-aware delete ContextMenu actions, aggregate download-status presentation, keyboard/pointer dynamic image selection, and emoji IconButton/input-field picker composition. They map to canonical ContextMenu + Message expiry/delete, Downloads/Resource + status surface, Avatar/Picker, and IconButton + Composer/TextField + Popover/emoji-provider owners. No source-named component is introduced. Accounting: **6,011/16,125 read; 10,114 unread; 15,846 unknown; 0 omitted**. First unread 6,012 `feature_list.cpp@8c3bf36c…`. Reading grants no implementation, verification, baseline or release credit.

### Source closure through 6,032

Orders **6,012-6,032** are exact-blob read/decomposed for feature/detail rows, filter/share header state, invite-link actions/ContextMenu, jump-down unread projection, labeled emoji Tabs, location-picker service/UI boundaries, participant LoadingState skeletons and Popover lifecycle. All map to source-neutral canonical owners; reading does not reduce unknown. The descendant Human-call shipping surface also serializes camera/screen-share mutations through one `video-media` command fence with canonical pending projection, rollback and capture teardown. ForceMuted/RaisedHand/scheduled/audio-reactive call presentation remains mapped-open. Accounting: **6,032/16,125 read; 10,093 unread; 15,846 unknown; 0 omitted**. First unread 6,033 `ui/controls/round_video_recorder.cpp@b52b73bd…`. No baseline/release credit is granted.

### Source closure through 6,041

Orders **6,033-6,041** are exact-blob read/decomposed for round-video recording/encoding, sender identity, the primary Composer send-state control and silent-send accessibility. Round-video remains mapped-open as a real device/media/codec lifecycle, not a cosmetic widget. The descendant Composer now also removes its raw HTML send button and reuses canonical `SandIconButton`. Accounting: **6,041/16,125 read; 10,084 unread; 15,846 unknown; 0 omitted**. First unread is 6,042 `ui/controls/stars_rating.cpp@7b7dce6e…`.

### Source closure through 6,052

Orders **6,042-6,052** are exact-blob read/decomposed for Stars rating, source-neutral tabs/subsection reorder and swipe gesture/scroll ownership. The existing canonical `SandTabs` owner is extended with optional overflow visibility, context-menu requests, locked reorder boundaries, pointer cancellation/edge-scroll and Alt+Arrow keyboard reorder; no Telegram-derived Tabs owner was added. Stars account/reputation truth, complete Avatar/Badge subsection composition and the navigation/conversation swipe owner remain mapped-open. Accounting: **6,052/16,125 read; 10,073 unread; 15,846 unknown; 0 omitted**. First unread is 6,053 `ui/controls/tabbed_search.cpp@ca207d0b…`. Reading alone does not reduce unknown.

### Source closure through 6,068

Orders **6,053-6,068** are exact-blob read/decomposed for grouped search/category picking, detail rows and transient tooltip lifetime, title/status chrome, localized fixed-precision amount input/IME, ephemeral-media countdown presentation, and Avatar/profile-media acquisition/upload/privacy/streaming lifecycle. The existing canonical `SandTooltip` gains optional auto-dismiss, outside-press/Escape dismissal and focus-return policy with a focused contract test; no Telegram-derived Tooltip owner was added. Search/Picker shipping composition, full profile-media ownership, monetary-domain applicability and authoritative ephemeral-message lifecycle remain mapped-open. Accounting: **6,068/16,125 read; 10,057 unread; 15,846 unknown; 0 omitted**. First unread is 6,069 `ui/controls/who_reacted_context_action.cpp@6851dee0…`. Reading alone does not reduce unknown.

### Source closure through 6,084

Orders **6,069-6,084** are exact-blob read/decomposed across who-read/reaction context actions, platform-obsolescence and screen-reader-mode Banners, country selection, dynamic media thumbnails, animated text, and credits/commerce graphics/style. Canonical owners remain source-neutral and all rows remain mapped-open unless independently closed; reading does not reduce unknown. Accounting: **6,084/16,125 read; 10,041 unread; 15,846 unknown; 0 omitted**. First unread is 6,085 `ui/effects/drifting_particles.cpp@ed119cf9…`.


### Source closure through 6,104

Orders **6,085-6,104** are exact-blob read/decomposed across decorative particles, reaction/custom-emoji fly overlays, fireworks/glare/LoadingState skeletons, Composer-to-Transcript message-send transitions, Stars particles/Story outlines, Premium commerce styling and an optional GPU 3D commerce cover. The source responsibilities require pause/teardown correctness, reduced-motion/power-policy handling, target-identity fencing, theme/DPR/RTL/responsive behavior, resource completion recovery and GPU capability fallback. They map only to source-neutral canonical animation/LoadingState/Transcript/Resource/Avatar/Status/commerce/design-system owners; no Telegram-derived parallel visual runtime is accepted. Accounting: **6,104/16,125 read; 10,021 unread; 15,846 unknown; 0 omitted**. First unread is 6,105 `ui/effects/premium_3d_mesh.cpp@b8541b31…`. Reading alone does not reduce unknown or grant implementation/release credit.


### Source closure through 6,125

Orders **6,105-6,125** are exact-blob read/decomposed across validated 3D asset loading, GPU support gating, reactive credits/limit/subscription presentation, coin/diamond RHI renderers, promo particle strategies and the interactive Premium Star lifecycle. Domain truth stays in canonical commerce/credits/gifts/subscription owners; graphics reuse canonical controls and the thinnest visual/GPU adapters. Malformed assets, unsupported RHI, shader/buffer/pipeline failures, power-saving/reduced-motion, pause/resume and partial-init teardown remain explicit failure/lifecycle obligations. Accounting: **6,125/16,125 read; 10,000 unread; 15,846 unknown; 0 omitted**. First unread is 6,126 `ui/effects/premium_star_model.cpp@499d751a…`. Reading alone grants no implementation or release credit.


### Platform-obsolescence Banner implementation slice

The accepted orders **6,071-6,072** now have a source-neutral implementation slice: shared status primitives expose one canonical `SandBanner`, and `frontend/src/production/platform-obsolescence-policy.ts` preserves the exact civil-date **7/30/90-day** dismissal rule with fail-open handling for corrupt/future persisted dates and an adapter for canonical client persistence. This deliberately does **not** invent an OS-support cutoff: the current Desktop bridge has no authoritative `WhenSystemBecomesOutdated` equivalent, so platform cutoff/reason sourcing and root shipping composition remain mapped-open. The focused contract is wired into the existing renderer GitHub Actions gate; no local build/test was used. Unknown remains unchanged.


### Source closure through 6,145

Orders **6,126-6,145** are exact-blob read/decomposed across Premium Star assets/particles/GPU renderer, colored collectible decoration, responsive commerce top-bar capability fallbacks, reaction fly presentation, Checkbox/Avatar selection rendering, scroll-edge shadows, and send-action status animations. Send-action types include record/upload/round/speaking/choose-sticker with typing fallback; same-family restart and speaking finish/restart transitions remain visual projections over authoritative conversation state, and animation-disabled mode resolves to static frames. All responsibilities map to source-neutral canonical owners; reading does not reduce unknown. Accounting: **6,145/16,125 read; 9,980 unread; 15,846 unknown; 0 omitted**. First unread is 6,146 `ui/effects/shake_animation.cpp@38fbd6f0…`.


### Source closure through 6,165

Orders **6,146-6,165** are exact-blob read/decomposed across shake/LoadingState skeleton feedback, snowflake/star-burst decoration, the message-removal dissolve capture/collapse/GPU compute lifecycle, disclosure-arrow affordance and TTL timer-icon projection. The dissolve path is explicitly presentation after authoritative deletion: it capability-gates power/RHI/compute, pre-captures exact Transcript items, reconciles collapse gaps and scroll baselines, bounds GPU particles/frame delta, and tears down pending/per-item resources; it never owns deletion truth. Accounting: **6,165/16,125 read; 9,960 unread; 15,846 unknown; 0 omitted**. First unread is 6,166 `ui/effects/unique_gift_message_bubble.cpp@22802623…`.


### Read-through 6166-6171

Accepted Telegram authority remains `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`. Orders **6,166-6,171** are exact-blob read and responsibility-decomposed: unique-gift message bubble geometry maps to canonical TranscriptEntry/Avatar plus Gift/Commerce presentation; upload progress lifecycle maps to canonical Resource/attachment upload + Status/Progress/IconButton; voice-once particles map to canonical voice-message playback presentation with reduced-motion/teardown policy. Reading alone does not reduce unknown. Accounting after orders 6,166-6,171 was **6,171 / 16,125 read**, **9,954 unread**, **15,846 unknown**, **0 omitted**. The next unread order was **6,172** `Telegram/SourceFiles/ui/empty_userpic.cpp`.

### Read-through 6172-6182

Accepted Telegram authority remains `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`. Orders **6,172-6,182** are exact-blob read and responsibility-decomposed: fallback identity presentation maps to canonical Avatar/semantic Icon/Resource owners; filter-icon selection maps to canonical Picker/Popover/Menu/IconButton with the existing conversation-filter domain owner; grouped-media geometry maps to the canonical TranscriptEntry/Resource layout utility and must remain deterministic and single-owned. Reading alone does not reduce unknown. Accounting after orders 6,172-6,182 was **6,182 / 16,125 read**, **9,943 unread**, **15,846 unknown**, **0 omitted**. The next unread order was **6,183** `Telegram/SourceFiles/ui/image/image.cpp`.

### Read-through 6183-6191

Orders **6,183-6,191** are exact-blob read and responsibility-decomposed across the image/resource base layer: prepared image cache keys and DPR transforms; typed download/image locations with versioned serialization, cache-key derivation and file-reference refresh; protocol-to-resource factories including progressive/cached/in-memory/web/video forms; bounded local-image decoding; and sanitized SVG preview rendering with explicit byte/dimension limits. These map to the existing canonical `Resource`/attachment/media owner rather than a second download/cache owner. Reading does not reduce `unknown`. Current accounting is **6,191 / 16,125 read**, **9,934 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,192** `Telegram/SourceFiles/ui/item_text_options.cpp`.

### Read-through 6192-6202

Orders **6,192-6,202** are exact-blob read and responsibility-decomposed across conversation-aware text option projection, single-window layer-stack lifecycle, canonical context-menu icon semantics, new/attention Badge projection, and local passcode-strength/transliteration helpers. These responsibilities map to existing source-neutral TranscriptEntry, Dialog/Popover, ContextMenu/Menu/Icon, Badge/Status and security/passcode owners. Sensitive passcode candidates must remain ephemeral and unlogged. Reading alone does not reduce unknown. Current accounting is **6,202 / 16,125 read**, **9,923 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,203** `Telegram/SourceFiles/ui/peer/color_sample.cpp@d11ac016c6e318786c3a1206efa415c91928ee21`.

### Read-through 6203-6214

Orders **6,203-6,214** are exact-blob read/decomposed across color/profile selection, video Avatar Resource playback, power-saving/reduced-motion policy, resize lifecycle, bounded row-scroll render caching and SearchField query/a11y composition. All map to existing source-neutral canonical owners and remain mapped-open. Reading alone does not reduce unknown. Accounting: **6,214 / 16,125 read**, **9,911 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,215** `Telegram/SourceFiles/ui/text/format_song_document_name.cpp@d5a77025b392b8385bff3268be1f7465d91d48a2`.

### Read-through 6215-6224

Orders **6,215-6,224** are exact-blob read/decomposed across media naming, locale/value/currency/credits formatting, custom-emoji animation lifecycle and canonical rich-text parser presets. All remain mapped-open; reading alone does not reduce unknown. Accounting: **6,224 / 16,125 read**, **9,901 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,225** `Telegram/SourceFiles/ui/top_background_gradient.cpp@7325d188f856d76f08c67a2bdb43ce1f50eb94b7`.


### Read-through 6225-6239

Orders **6,225-6,239** are exact-blob read/decomposed across commerce/profile top-gradient pattern rendering, unread/peer Badge precedence and animation/power-saving behavior, unread counter formatting, Avatar/Resource cache invalidation and fallback shapes, canonical vertical-list composition, and WebView theme/zoom/style serialization plus attribute/script escaping. All remain mapped-open and source-neutral; build-only `ui_pch.h` is recorded without claiming product closure. Reading alone does not reduce unknown. Accounting: **6,239 / 16,125 read**, **9,886 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,240** `Telegram/SourceFiles/ui/widgets/chat_filters_tabs_mode.h@f5d37194063110eb2ba8698f5b714e8a7ebe37ea`.


### Read-through 6240-6244

Orders **6,240-6,244** are exact-blob read/decomposed across persisted chat-filter Tabs presentation mode, canonical Tabs/Badge/ContextMenu composition, locked-range handling, custom-emoji pause, and drag-reorder lifecycle with pinned intervals, threshold start, edge auto-scroll, cancel/apply convergence and stable index remap. These remain mapped-open and must reuse existing source-neutral Tabs/ContextMenu/Badge/Status plus the single canonical ordering state owner. Reading alone does not reduce unknown. Accounting: **6,244 / 16,125 read**, **9,881 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,245** `Telegram/SourceFiles/ui/widgets/chat_filters_tabs_strip.cpp@750feab6cbf3b30a3b1e2bc946d85eaed4a4382c`.


### Read-through 6245-6251

Orders **6,245-6,251** are exact-blob read/decomposed across the shipping chat-filter Tabs composition, premium lock/menu/edit/remove/mark-read and saved ordering lifecycle, plus canonical Color Picker/validated field synchronization and Continuous/Media Slider pointer-wheel-keyboard-a11y progress/finished semantics. These remain mapped-open and must reuse source-neutral Tabs/ContextMenu/Badge/Picker/TextField/Slider and existing filter/settings/media domain owners. Reading alone does not reduce unknown. Accounting: **6,251 / 16,125 read**, **9,874 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,252** `Telegram/SourceFiles/ui/widgets/cross_fade_label.cpp@fb094439ceebc0ad1b45db68ebf08d31450464e3`.


### Read-through 6252-6261

Orders **6,252-6,261** are exact-blob read/decomposed across cross-fade text motion, Discrete/Settings Slider selection timing and ripple state, expandable participant Checkbox/Avatar collection semantics, phone/country/username masked-input normalization including Windows IME re-entry fencing, and count-aware time-part placeholders. These remain mapped-open and must reuse source-neutral Status/text motion, Tabs/Slider, ParticipantRow/Checkbox/Avatar, and TextField owners. Reading alone does not reduce unknown. Accounting: **6,261 / 16,125 read**, **9,864 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,262** `Telegram/SourceFiles/ui/widgets/glare_tooltip.cpp@b38e44a0769199bf65b5d584181e3b4f24d716aa`.


### Read-through 6262-6269

Orders **6,262-6,269** are exact-blob read/decomposed across glare Tooltip tracking/timer/teardown and motion, gradient Button glare/ripple/cache behavior, equal-width horizontal Button layout, and LevelMeter value projection. These are presentation responsibilities over existing source-neutral Tooltip/Button/layout/Status owners and must honor reduced-motion and lifecycle teardown. Reading alone does not reduce unknown. Accounting: **6,269 / 16,125 read**, **9,856 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,270** `Telegram/SourceFiles/ui/widgets/marquee_label.cpp@ddb4fac4dc2eb2f4d990b283aad018416ce7d1d2`.


### Read-through 6270

Order **6,270** `Telegram/SourceFiles/ui/widgets/marquee_label.cpp@ddb4fac4dc2eb2f4d990b283aad018416ce7d1d2` is exact-blob read-complete/responsibility-decomposed. It owns overflowing text presentation with delayed wraparound marquee motion; animation is gated by visibility, active window, overflow, reduced-motion policy and selection/menu pause state. It also carries DPR/palette/text/geometry cache invalidation, selectable text with word/paragraph selection and edge-scroll, keyboard/mouse clipboard projection, outside-click/menu teardown, and StaticText accessibility. The responsibility maps only to source-neutral canonical text/Status, ContextMenu/Menu, clipboard, accessibility and reduced-motion owners and remains mapped-open pending shipping composition plus focused lifecycle/a11y evidence. Reading does not reduce unknown. Accounting: **6,270 / 16,125 read**, **9,855 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,271** `Telegram/SourceFiles/ui/widgets/marquee_label.h@b2623a32af81775016bed793ed4c31d66da81a24`.


### Read-through 6271-6292

Orders **6,271-6,292** are exact-blob read/decomposed across the remaining marquee contract, middle-click and selecting autoscroll, responsive MultiSelect, participant expander checks, ephemeral passcode-strength projection, peer identity bubbles, sensitive verification-code normalization/resend-call countdown, Slider width adaptation, SlidingTabs lifecycle/preload fencing and VerticalDrumPicker input/snap semantics. All map to existing source-neutral canonical text/status, ContextMenu, ScrollArea, Picker/SearchField/TextField/ListRow/ParticipantRow/Avatar, Checkbox/Button, Security/Passcode, Authentication/Verification, Slider/Tabs and shared motion/a11y/platform owners. Sensitive candidates remain ephemeral/unlogged; hidden tabs must not preload; timers/animations/cursors/event filters require teardown and stale-callback fencing. Reading alone does not reduce unknown. Accounting: **6,292 / 16,125 read**, **9,833 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,293** `Telegram/SourceFiles/ui/window_palette.cpp@003a1871a2242d25a48f0826938f6d8a1c1d8b29`.


### Read-through 6293-6303

Orders **6,293-6,303** are exact-blob read/decomposed across per-window palette lifetime, Wallet presentation tokens, TON address/transfer-link parsing, animated amount-entry/glyph-diff/IME/selection/reduced-motion semantics, amount painting, and the session-scoped TON-center request adapter. The Wallet API owns validated shifted-DC routing, request IDs, cancellation, 30s timeout settlement, late/duplicate answer fencing, pending accounting and 10s idle-session teardown; it maps only into the canonical Wallet/Payments service owner. The amount field carries the concrete glyph matching/roll/fade/scale and deterministic animation-disabled settlement semantics relevant to the animated-text responsibility, but reading alone does not constitute Fabushi shipping closure. Accounting: **6,303 / 16,125 read**, **9,822 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,304** `Telegram/SourceFiles/wallet/wallet_card_angle.cpp@94800a1532b5ca0baaa878c4d06785f084120ec5`.
Orders **6,304-6,315** are exact-blob read/decomposed across Wallet card pointer-follow motion and DPR-aware gradient caches, a capability-restricted Wallet-to-chat/session Show adapter, collectible metadata/artwork priority lanes with a 10-chain budget, 60-second deadline, generation fencing, Gift-to-web fallback, cancellable Resource loading and preload-window projection, plus the encrypted transfer-comment reveal lifecycle with account/session/app-lock/sleep fencing, bounded identity resolution, key authorization, revision/scope cancellation and plaintext wipe on reset. These remain mapped-open to the existing source-neutral Wallet/Payments/Commerce, Resource/file-transfer/cache, Session/Navigation, Security/Key Protection, Theme, motion and canonical design-system owners; no Telegram-derived parallel Wallet UI/runtime is introduced. Reading alone does not reduce unknown. Accounting: **6,315 / 16,125 read**, **9,810 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,316** `Telegram/SourceFiles/wallet/wallet_content.cpp@6aa42680fee3870ae9969104fdb5872c309e8ef2`.
Orders **6,316-6,319** are exact-blob read/decomposed across the full Wallet content/composition monolith and its public seam plus the durable custody store. The source proves exact-dependency/revision fee quotes, per-press frozen recipient/amount/comment/link expiry, key-ladder and vault-epoch fencing, bounded signing readiness, QuoteExpired retries only before anything leaves the device, SubmissionUnknown operation handoff rather than blind resend, bounded recipient/name/address lookups, secure phrase/restore/import/conflict/backup/rotation lifecycles, polling/teardown and list/scroll-anchor behavior. Custody persistence is v4, separates immutable anchor and current signing keys, retains pending rotation/new-key/awaiting-server reconciliation, validates old/broken data fail-closed and centralizes secret-reference removal. All remain mapped-open to existing source-neutral Wallet/Payments, Security/Key Protection/Custody, storage, Session/Navigation, Resource, Motion and canonical design-system owners; no Telegram-derived parallel owner is introduced. Reading alone does not reduce unknown. Accounting: **6,319 / 16,125 read**, **9,806 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,320** `Telegram/SourceFiles/wallet/wallet_diamond_flight.cpp@9444c0eac18a5c625aad9a2671bbd319974129fb`.
Orders **6,320-6,323** are exact-blob read/decomposed across the send-diamond animation and the Wallet engine/host bridge. Motion semantics include a dynamic target, target-loss cancellation, exact single landing/finish, reduced-motion and power-saving suppression, loop continuity and bounded dirty-region repaint. Engine semantics include strict provider-origin/path enforcement, Accepted/Rejected/Uncertain routed submission, a 40-second ambiguous-broadcast wait that preserves late MTProto settlement, durable send-journal compare-exchange, account/clear-epoch protected-secret fencing, exact failed-call secret cleanup, stale private-result rejection, serial worker ownership and teardown that wakes blocked calls before shutdown while relying on durable journal recovery on restart. These remain mapped-open to existing source-neutral Wallet/Payments, Network, Vault/Key Protection, durable Storage/Journal and Motion owners. Reading alone does not reduce unknown. Accounting: **6,323 / 16,125 read**, **9,802 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,324** `Telegram/SourceFiles/wallet/wallet_fiat.cpp@6e16d23676a8d24ff840c71f88310cb21d540ce4`.
Orders **6,324-6,325** are exact-blob read/decomposed across Wallet fiat/rates formatting: unavailable-rate ellipsis, currency-specific exponent/decimal/grouping, approximate U+2248 output, tiny positive-value significant digit projection, localized currency names, stable top-currency ordering from the active currency plus cloud/system/input-method/phone-country locale signals, and minor-unit nanos. They remain mapped-open to source-neutral canonical Money/Currency formatting and Wallet rates owners.

Orders **6,326-6,339** are exact-blob read/decomposed across Wallet key protection, funding/on-ramp, palette replacement, singleton panel lifecycle, recovery phrase shares, live fiat-rate refresh, and sending effects. Existing-owner-first production work at parent exact HEAD `882728682058ea6235cf625c7e8a33f851fad1a3` extends `native/mahayana-messaging/src/wallet.rs`, `engine.rs`, and `service.rs`: provider-neutral authenticated key wrapping with epoch fencing and zeroized secret bytes; fixed-field recovery seed/XOR shares plus authenticated holder encryption; 5-minute rate refresh / 30-second retry state; funding-request generation fencing and optional base-currency fallback; singleton panel state; deterministic clock/glare math; native/server-authoritative runtime projection. Exact-head evidence: Desktop Chat Parity run `38016864742` completed success; Rust desktop runtime run `38016864748` bound the same SHA, canonical messaging tests succeeded and artifact `11656926928` has digest `sha256:cb27c66d4893776f5105741254f86df84d25a707ff7cd9f256e708c76676b0fa`. These rows intentionally remain mapped-open: no concrete platform key-protection adapter, real funding/rates service adapter, wallet shipping renderer/panel consumer, wallet-specific palette consumer, recovery-share product entrypoint, or sending-effect paint consumer has exact-head production evidence yet. Reading therefore does not reduce global unknown. Accounting: **6,339 / 16,125 read**, **9,786 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,340** `Telegram/SourceFiles/wallet/wallet_session.cpp@7000a36395819deea9ca8ba0cce227ff0e535f32`.


### Read-through 6340-6343

Orders **6,340-6,343** are exact-blob read/responsibility-decomposed across the Wallet Session monolith/public state contract and the real-time Wallet stream. Existing-owner-first production work extends only canonical `native/mahayana-messaging` Wallet/ConnectedApp owners: parked-wallet discovery, panel-scoped legacy-balance lookup, durable connected-app recovery fallback separation, and a source-neutral live-stream epoch/backoff/keepalive/renew/coalesce/history-recheck state machine. The stream still lacks a real Fabushi Host/network service adapter and 6,340-6,341 retain substantial mapped-open custody/history/collectibles/preview/send-recovery/rotation/TonConnect/UI responsibilities, so none of these rows is marked verified and global `unknown` is unchanged. Accounting: **6,343 / 16,125 read**, **9,782 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,344** `Telegram/SourceFiles/wallet/wallet_ton_connect.cpp@60bf68a9…`. Fresh exact-head GitHub Actions are required for the accounting descendant.

### Read-through 6344-6353

Orders **6,344-6,353** are exact-blob read/responsibility-decomposed across the connected-app session/key/connect/disconnect lifecycle, canonical connect/settings UI projection contract, durable claim journal, transaction emulation, and deep-link/start-param boundary. Existing-owner-first production descendants already implement transport message identity, staged claim recovery, source-neutral connect/key/disconnect state, emulation parsing and start-param security/encoding inside canonical native/mahayana-messaging owners. The connect/settings UI projection, real service/navigation composition and complete request-family closure remain mapped-open; no row is promoted to verified and reading does not reduce global unknown. Accounting: **6,353 / 16,125 read**, **9,772 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,354** Telegram/SourceFiles/wallet/wallet_ton_connect_request.cpp@496881cf….

### Read-through 6354-6357

Orders **6,354-6,357** are exact-blob read/responsibility-decomposed across the full connected-app request scheduler/Flow contract and its request-confirmation projection. Existing-owner-first production descendants now cover transport message identity, active/silent/waiting arbitration, durable claim recovery rebuild, recovery polling/submission settlement, flow-completion ownership fencing and Recovery::Offer fallback through canonical native/mahayana-messaging ConnectedApp owners. Remote pending/service composition, closed-session late context, wallet resolve/polling, cryptographic decrypt/sign/encrypt, fee/emulation send integration, Recovery::Answer re-encryption and shipping request UI/a11y/visual evidence remain mapped-open. No row is promoted to verified and reading does not reduce global unknown. Accounting: **6,357 / 16,125 read**, **9,768 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,358** Telegram/SourceFiles/wallet/wallet_transfer_messages.cpp@69cfdacd….



### Revision 9 source closure through 6,361

Orders **6,358-6,361** are exact-blob read/responsibility-decomposed for optimistic outbound transfer-message reconciliation and the durable submitted-transfer store. The canonical Wallet journal already owns handoff/lookup/terminal recovery; this descendant additionally fail-closes noncanonical source, destination and collectible addresses through the existing wallet-address validator. The Message owner still lacks the complete per-draft server-id floor, transport-identity and identical-twin adoption/refusal contract, so 6,358-6,359 remain explicitly mapped-open rather than receiving false implementation credit. Accounting is **6,361/16,125 read; 9,764 unread; 15,846 unknown; 0 omitted**. First unread is 6,362 `Telegram/SourceFiles/wallet/wallet_unlock.cpp@5fbaea9d950772164094e7477e5f786f9a78279f`.


### Revision 9 read-through 6362-6367

Wallet unlock, user-address and vault files are exact-blob read/responsibility-decomposed. Descendant `cb6d56174fb3e3f34c99e7bed3156c3b1e8d523d` closes one security gap: unlocked Wallet projection is runtime-only across serde/restart. Full grant/retention/platform-factor/address-service/multi-account vault lifecycle remains mapped-open. Accounting: 6,367/16,125 read; 9,758 unread; 15,846 unknown; 0 omitted. First unread: 6,368 `Telegram/SourceFiles/webauthn/cable.h@6909eb913dbb41765ebbaeecb2072f130f868655`.


### Revision 9 read-through 6368-6377

The first CaBLE/WebAuthn tranche is exact-blob read/responsibility-decomposed. Existing Fabushi WebAuthn provider/signer/proxy is the canonical owner; full BLE/QR/tunnel/Noise/CTAP and ceremony UI parity stays mapped-open. `94aaee4` already closes one fail-open boundary by rejecting unknown ceremony kinds instead of treating them as login/get. Accounting: 6,377/16,125 read; 9,748 unread; 15,846 unknown; 0 omitted. First unread: 6,378 `Telegram/SourceFiles/webauthn/org.bluez.xml@245e0b9bc27e1cfbb31bd78d750ae189adf5f0cc`.


### Revision 9 read-through 6378-6380

BlueZ interface and common WebAuthn security-key/CaBLE orchestration are exact-blob read/responsibility-decomposed. Existing Fabushi WebAuthn provider/signer/proxy remains the canonical owner; device/PIN/cancellation/fallback/platform parity remains mapped-open. Accounting: 6,380/16,125 read; 9,745 unread; 15,846 unknown; 0 omitted. First unread: 6,381 `Telegram/SourceFiles/window/main_window.cpp@ae2b4949734e6c8826310c96b65f51b630ab600d`.


### Revision 9 read-through 6381-6388

Main window and first notification-manager tranche are exact-blob read/responsibility-decomposed against existing Electron main/window-state/window-chrome/dock-badge/OS-notification owners. Existing geometry/focus/maximized/badge and scoped notification action behavior is retained; close-to-tray/background, privacy-aware OS title, full canonical conversation notification mute/privacy/grouping/device-delay and fallback/resource lifecycle remain explicitly mapped-open. Accounting: 6,388/16,125 read; 9,737 unread; 15,846 unknown; 0 omitted. First unread: 6,389 `Telegram/SourceFiles/window/section_memento.h@d139928a9996a6333e541a5db9c77e64056b41fd`.


### Revision 9 read-through 6389-6392

Section navigation/workspace contracts and draw-to-reply media flow are exact-blob read/responsibility-decomposed. Qt widget/slide/painting is not copied; routing, memento identity, focus/search/permission/theme behavior and async draw-to-reply target/payment/ephemeral safeguards map into existing ProductShell/ConversationWorkspace/Search/Permissions/Composer/Resource/Message owners. Accounting: 6,392/16,125 read; 9,733 unread; 15,846 unknown; 0 omitted. First unread: 6,393 `Telegram/SourceFiles/window/themes/window_theme.cpp@2ead4b28873908361728df3f622318ed073cdee2`.


### Revision 9 read-through 6393-6412

Theme runtime/editor/preview/chat/cloud/embedded/accent/name-generation files are exact-blob read/responsibility-decomposed. Existing SandThemeController and the Fabushi Design System remain the only app/theme visual authority; Telegram Qt palette/theme-format rendering is not ported. Applicable bounded parsing/resources, transactional test→keep/revert, async generation fencing, per-conversation/cloud theme state, accessibility/contrast and editor lifecycle remain mapped-open against existing Theme/Settings/Conversation/Resource owners. Accounting: 6,412/16,125 read; 9,713 unread; 15,846 unknown; 0 omitted. First unread: 6,413 `Telegram/SourceFiles/window/window.style@a91951fc42761db61f7f7718f89e226fc9d3b5c2`.


### Revision 9 read-through 6413-6423

Window style/adaptive/chat preview/chat switch/connection/controller files are exact-blob read/responsibility-decomposed. Current Fabushi already has canonical responsive shell, ConversationWorkspace/List/navigation, CoordinatorConnectionController and Electron window owners, so no Telegram window runtime is introduced. Full adaptive columns, accessible chat preview/switch, proxy/retry status, multi-account/separate-window and security/layer sequencing remain mapped-open where current behavior is partial. Accounting: 6,423/16,125 read; 9,702 unread; 15,846 unknown; 0 omitted. First unread: 6,424 `Telegram/SourceFiles/window/window_filters_favorite.cpp@c51e19763a7939fe908fb8b9fc3eb5f73735674f`.


### Revision 9 live read-through note: 6424-6437

Orders **6,424-6,437** are exact-blob read-complete and responsibility-decomposed on accepted `28ac576967a1026ecc89a927360fa0e138d7c88c`. Favorite/filter files preserve validated source-neutral shortcut/deep-link cancellation, collection selection/reorder/premium-lock, unread/mark-read, drag-hover and keyboard/screen-reader semantics. History-hider files preserve exactly-once overlay dismissal. Lock files preserve passcode flood/busy/async derivation, OS-unlock cooldown/lifetime and terms-consent security semantics. Main-menu/helpers preserve multi-account unread, archive lifecycle, profile/status, Contacts/Calls/Wallet/Settings navigation, theme/scale, owned-room enumeration and lifetime-bound dynamic Mini App/Plugin resource projection. All remain **mapped-open** to existing canonical ProductShell/Account/Conversation/ReadState/Security/Settings/Marketplace/MiniApp/Resource owners; no Telegram runtime/sidebar/menu/security registry is introduced. Reading alone closes no unknown. Accounting: **6,437/16,125 read; 9,688 unread; 15,846 unknown; 0 omitted**. First unread: **6,438 `Telegram/SourceFiles/window/window_media_preview.cpp@4fe0e7f11298b0b56c99c47e43c27c6fd47d0282`**. Fresh descendant exact-head Actions are required.

### Revision 9 live read-through note: 6438-6448

Orders **6,438-6,448** are exact-blob read-complete and responsibility-decomposed on accepted `28ac5769`. They cover media-preview resource/playback lifetime, context-sensitive canonical conversation commands, temporary restore shells, durable multi-window/account/conversation restore, section fallback semantics, and detachable-window route/security identity. All remain **mapped-open**. Current `source/electron-main/window-state-persistence.ts` is explicit partial parity: it persists one main-window placement but not durable account/conversation multi-window restore. Reading alone closes no unknown. Accounting: **6,448/16,125 read; 9,677 unread; 15,846 unknown; 0 omitted**. First unread: **6,449 `Telegram/SourceFiles/window/window_session_controller.cpp@df216237e47d22c200a51b2c7fda31175682d3c5`**.

### Revision 9 live read-through note: 6449-6458

Orders **6,449-6,458** are exact-blob read-complete and responsibility-decomposed. The large SessionController is decomposed into canonical Navigation/Conversation/Search/Window/Resource/Permissions/Calls/Settings/Theme/Story owners rather than ported as a new monolith. The range also captures typed deep-link intent, setup-email enrollment/verification with flood/expiry handling, canonical motion/adaptive-top-bar semantics, and passcode unlock exactly-once convergence across windows. All remain **mapped-open**. Accounting: **6,458/16,125 read; 9,667 unread; 15,846 unknown; 0 omitted**. First unread: **6,459 `Telegram/Telegram.plist@d156eda29c86bbd11b3c7757b9d419e2c863ea8b`**.

### Revision 9 live read-through note: 6459-6483

Orders **6,459-6,483** close the upstream macOS bundle plist, helper/app entitlements, icon-catalog manifest and every referenced 16/32/128/256/512 1x/2x icon blob by exact identity. The applicable responsibility is bundle/privacy/protocol/signing/sandbox/icon coverage under **Fabushi** identity; Telegram/TON schemes and Telegram artwork are explicitly not migrated. Current `desktop/package.json` already owns `com.ombhrum.fabushi`, the `fabushi` scheme, Fabushi icon, forced signing, notarization, and microphone/camera usage; current MAS entitlements own sandbox/network/user-selected/mic/camera. Downloads/bookmarks/location remain applicability-open and are not added without a shipping need. Accounting: **6,483/16,125 read; 9,642 unread; 15,846 unknown; 0 omitted**. First unread: **6,484 `Telegram/ThirdParty/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`**.

### Revision 9 live read-through note: 6484-6505

Orders **6,484-6,505** close the root ThirdParty gitlink identity layer: exact pinned commits and `.gitmodules` repositories are mapped to source-neutral capability/implementation roles. This includes generic C++ support (GSL/expected/range-v3/TooManyCooks), math/Markdown/QR/syntax/image/language/spell utilities, Linux IME/portal adapters, CBOR/FIDO2/passkey dependencies, compression/hash/password-strength primitives, and `tgcalls`. The root pin does **not** close any nested recursive source; all nested entries remain independently unread/unknown until their later deterministic orders. No Telegram runtime/library is automatically retained. Accounting: **6,505/16,125 read; 9,620 unread; 15,846 unknown; 0 omitted**. First unread: **6,506 `Telegram/build/build.bat@3e5f698f99c5944e237c514d60124bc25489ea76`**.

### Revision 9 live read-through note: 6506-6541

Orders **6,506-6,541** close the root build/release tooling source read. Product-relevant responsibilities include fail-fast release prerequisites, version/channel/artifact collision fencing, macOS preserved-toolchain and universal-resource checks, Windows/Linux packaging, App Store/GitHub publication, source tarballs including submodules, symbols, installer/upgrade semantics, reproducible dependency preparation, canary positive+negative signature fixtures, ES256 Key Vault signing with manifest-key-coordinate verification, and digest/dedupe/retry semantics for spellcheck dictionary distribution. Telegram product/private paths are not targets; equivalent Fabushi guarantees must live in existing GitHub Actions/electron-builder/updater/platform/resource owners. Accounting: **6,541/16,125 read; 9,584 unread; 15,846 unknown; 0 omitted**. First unread: **6,542 `Telegram/cmake/binobj2obj.py@3683a6dd3a72706ee382ea415d6e9cfaae84d8ce`**.

### Source closure through 6,574

Orders **6,542-6,574** are exact-blob read-complete/responsibility-decomposed for CMake/build-runtime composition. They preserve source-neutral responsibilities for deterministic code/resource generation, updater trust-root and signed-manifest embedding, FIDO2/WebAuthn, Calls/media composition, Wallet/Payments, password-strength dictionaries, Localization, data export, protocol/session maturity, Apple Swift runtime packaging and production-code updater tests. No Telegram-specific runtime or second product owner was introduced. Accounting: **6,574/16,125 read; 9,551 unread; 15,846 unknown; 0 omitted**. First unread is 6,575 gitlink `Telegram/codegen@dbd1d53137cd581cfdbeebe93bcaf54abf55ec74`. Reading/decomposition grants no implementation or release credit; same-head Actions and production/release evidence remain mandatory.


### Source closure through 6,591

Orders **6,575-6,591** are exact root-entry read-complete/responsibility-decomposed. The pinned `Telegram/codegen` child tree was inspected for tokenizer/UTF-8, emoji compatibility, localization and style-generation responsibilities; configure wrappers preserve fail-closed target/toolchain/credential propagation; direct library gitlinks map to existing runtime/concurrency/media/QR/reactive/spellcheck/storage/protocol/translation/design-system/call/webview owners. `Telegram/create.bat` is explicitly classified development-only non-applicable after reading, so it closes exactly one unknown without counting as omitted. Accounting: **6,591/16,125 read; 9,534 unread; 15,845 unknown; 0 omitted**. First unread is 6,592 `Telegram/shaders/argb32.frag@4b4deb01e408b5f441808dc104488e3b8eeabfe2`. No mapped-open row receives implementation or release credit.


### Source closure through 6,626

Orders **6,592-6,626** are exact-blob read-complete/responsibility-decomposed for GPU shader contracts. The tranche preserves ARGB/NV12/YUV420 conversion, OpenGL/Vulkan coordinate-origin handling, blur/dither, premultiplied alpha, rounded/fade/shadow composition, PiP nine-slice shadow behavior, premium time/night/alpha material inputs, and seeded/timestep-bounded particle lifecycle. These are mapped to source-neutral Fabushi rendering/Resource/Call/visual-effect owners; Telegram branding and pixel design are not carried over. Accounting: **6,626/16,125 read; 9,499 unread; 15,845 unknown; 0 omitted**. First unread is 6,627 `changelog.txt@bf82383b02717265acac50035c2ce3000afdac7a`. All shader rows remain mapped-open until shipping consumers, focused tests and cross-platform visual evidence exist.


### Root source closure complete at 6,653

Orders **6,627-6,653** close the remaining root entries: release changelog/build helpers, credential/build documentation, AppStream screenshot assets, WEB proxy design and acceptance contracts, Linux XDG/AppStream/DBus/Snap shipping integration, Wallet/thread/shared-media/navigation task evidence, and static-analysis/edit-rule tooling. Root accounting is now **6,653/6,653 exact-read**, while the recursive census remains **6,653/16,125 read; 9,472 unread; 15,845 unknown; 0 omitted**. The next recursive identity is order 6,654: `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`, mount `Telegram/ThirdParty/GSL`, path `.clang-format`, object `c12d3bf2994fd5a083c04025d355a5ab4b3f6802`. The existing validator intentionally forbids crossing the root prefix without an explicit component identity schema, so schema/validator extension is the next required gate rather than a bookkeeping bypass.

### Recursive component read-through 6,654-6,697 — Microsoft/GSL

Orders **6,654-6,697** exact-read the pinned `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37` component mounted at `Telegram/ThirdParty/GSL`. The 44 entries cover repository/build/CI metadata, license/security/provenance, header-only safety contracts (bounds and extent checks, non-null ownership, checked narrowing, byte/string boundaries, scope cleanup), and focused fail-closed tests. Applicable semantics map to existing Fabushi Build/Quality/Provenance and Rust/TypeScript/platform-boundary owners; no Telegram/GSL-specific runtime is introduced. Reading does **not** reduce `unknown`; production/shipping evidence remains required. Accounting: **6,697/16,125 read; 9,428 unread; 15,845 unknown; 0 omitted**. Next: **6,698 `Telegram/ThirdParty/MicroTeX::.github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764`**.

### Recursive component read-through 6,698-6,714 — MicroTeX entry/build contracts

Orders **6,698-6,714** exact-read the pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f` build/entry contract under `Telegram/ThirdParty/MicroTeX`. The applicable product behavior maps into the existing ConversationWorkspace/Transcript math owner and existing Build/Resource/Release provenance owners. Current production work adds a 32 KiB pre-parse formula bound and bounded 32 MiB LRU markup cache without retaining the C++/Qt/GDI runtime. Reading does **not** reduce `unknown`; current exact-head GitHub Actions evidence is still required. Accounting: **6,714/16,125 read; 9,411 unread; 15,845 unknown; 0 omitted**. Next: **6,715 `Telegram/ThirdParty/MicroTeX::readme/example_bw_false.svg@9e812c823d3cbe06ab43ee77f10b6122bd638dd`**.


### Recursive component read-through 6,715-6,740 — MicroTeX reference/golden/resource manifest

Orders **6,715-6,740** exact-read the pinned MicroTeX reference SVG/PNG outputs, executable sample corpus, golden rendered samples, TeX logo, resource-root sentinel and `RES_README` font/license/language resource manifest. Applicable semantics map to the existing canonical ConversationWorkspace/Transcript math visual-regression owner plus Resource/Release provenance owners. These assets are regression/provenance authority only; they do not justify retaining or shipping a MicroTeX/Telegram renderer. Reading does **not** reduce `unknown`; exact-head production/resource/visual evidence remains required. Accounting: **6,740/16,125 read; 9,385 unread; 15,845 unknown; 0 omitted**. Next: **6,741 `Telegram/ThirdParty/MicroTeX::res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b`**.

### Recursive component read-through 6,741-6,800 — MicroTeX multilingual formula resources

Orders **6,741-6,800** exact-read the executable formula sample corpus, compiled font-resource manifest, Cyrillic license/mappings/symbols/font metrics/fonts, Latin/math font resources and licenses, and the first Greek font/metric resources. Applicable responsibilities stay in the existing ConversationWorkspace/Transcript math owner plus Resource/Packaging/Release provenance owners: multilingual Unicode-to-symbol/formula mapping, deterministic glyph metrics/kerning, explicit resource fallback, packaged resource completeness, license/notice provenance, and rendering regression. Reading does **not** reduce `unknown` and provides no verified credit. Accounting: **6,800/16,125 read; 9,325 unread; 15,845 unknown; 0 omitted**. Next: **6,801 `Telegram/ThirdParty/MicroTeX::res/greek/fcmbpg.xml@dbedd0385cdb004569e0d69b2567a1fa6c6cad5f`**.

### Recursive component read-through 6,801-6,830 — MicroTeX Greek resources and atom safety

Orders **6,801-6,830** exact-read the rest of the Greek glyph resources plus the first atom/rendering implementation slice. Applicable atom responsibilities include wrapper-chain depth bounding, copy-before-mutation for shared cached atoms, exceptional-path state cleanup, bounded matrix/column-spec expansion, finite user length handling, TeX operator/glue/kerning isolation, delimiter/matrix/layout semantics, and deterministic multilingual glyph resources. These map to the existing canonical KaTeX math owner and Resource/Release provenance owners. The shipping owner now sets explicit `maxExpand=1000`, `maxSize=1000em`, and `trust=false` for strict and recovery renders, with a focused Desktop Chat Parity contract; exact-head verification is still required. Reading does **not** reduce `unknown`. Accounting: **6,830/16,125 read; 9,295 unread; 15,845 unknown; 0 omitted**. Next: **6,831 `Telegram/ThirdParty/MicroTeX::src/atom/atom_space.h@3ba203a52a1fcff831a1137dfa6c3e35927fb0d7`**.

### Recursive component read-through 6,831-6,852 — MicroTeX bounded layout/parser safety

Orders **6,831-6,852** exact-read the pinned MicroTeX unit/color, box/layout, formula/environment and glue implementation contracts. Applicable responsibilities are source-neutral: invalid unit/atom/style/table indices fail closed; NaN/Infinity color/scale/rotation/size values cannot poison renderer transforms; delimiter/arrow/box/matrix growth is bounded; partial parse keeps a non-null fallback; exceptional paths restore graphics/font/parser state; predefined formula/glue semantics remain deterministic. Build/config files preserve only dependency/platform provenance. These map to the existing canonical ConversationWorkspace/Transcript KaTeX owner plus Build/Release provenance; no MicroTeX C++/Qt runtime is retained. The canonical owner now also fail-closes non-finite custom budgets and has focused contracts for rendered-markup bounds, cache sharing/LRU eviction, loader isolation, invalidation and retry-after-rejection. Reading does **not** reduce `unknown`. Accounting: **6,852/16,125 read; 9,273 unread; 15,845 unknown; 0 omitted**. Next: **6,853 `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83`**.

### Recursive component read-through 6,853 — MicroTeX localized decimal input

Order **6,853** exact-reads `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83` from pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`. Its applicable responsibility is deterministic numeric input normalization: U+066B becomes the ASCII decimal point and the twenty accepted Unicode decimal blocks map digit-for-digit to ASCII while every unrelated code point remains unchanged. The existing canonical ConversationWorkspace/Transcript KaTeX owner performs that normalization before both strict and recovery renders, and the focused Desktop Chat Parity contract covers all twenty blocks, unchanged non-target text, and identical strict/recovery input. No MicroTeX parser/runtime is retained. Reading does **not** reduce `unknown`; descendant exact-head GitHub Actions remain required. Accounting: **6,853/16,125 read; 9,272 unread; 15,845 unknown; 0 omitted**. Next: **6,854 `Telegram/ThirdParty/MicroTeX::src/core/macro_def.cpp@0ccf9fe8b4028f97c62834555d2a3d344d0f8726`**.

### Live rebaseline 28ac5769 → 6fed91ff

Accepted upstream advanced by four commits to `6fed91ffab9861771a75f29df65031b3c80b6941` / root tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6` without adding or removing a root path. Ten root blobs changed at deterministic orders **3,155, 3,201, 3,202, 3,766, 4,521, 4,522, 5,166, 5,943, 6,541, 6,627**. Each changed blob was re-read against its predecessor object and reconciled through the explicit source-disposition rebaseline record. Version/resource/build changes preserve source-neutral release-version coherence. The history-photo delta adds owned enlarge eligibility/hit-testing plus press/release ripple lifecycle and keeps sponsored/editor/small-media exclusions; it maps to the existing Transcript/media interaction owner rather than a new media root. The media-viewer delta explicitly claims Save/SaveAs at the application event-filter boundary so competing shortcuts cannot consume Ctrl+S; it maps to the existing canonical media-viewer shortcut owner. The new style opacity is presentation-only under the Fabushi design system. The 7.3.1 changelog adds wallet Windows Hello/passcode, copy-restricted download, custom-theme date-color and photo-editor Link regression cues; those cues remain open until their existing Fabushi owners have shipping evidence. Recursive totals/order are unchanged, so read-through remains **6,853/16,125**, first unread remains **6,854**, unread **9,272**, unknown **15,845**, omitted **0**.


### Revision 9 exact read-through 6862-6872 — MicroTeX font lifecycle

Orders **6,862-6,872** are exact-blob read-complete/responsibility-decomposed for Unicode/alphabet registration, glyph metrics/extensions, safe missing lookup, font/symbol registration, math-size/magnification reset boundaries and font build closure. They map to the existing canonical math wrapper plus Build/Release provenance; no second font runtime is introduced. Reading alone does not reduce unknown. Accounting: **6,872/16,125 read; 9,253 unread; 15,845 unknown; 0 omitted**. First unread is **6,873 `Telegram/ThirdParty/MicroTeX::src/fonts/tex_font.h`**.


### Revision 9 exact read-through 6873-6892 — MicroTeX rendering boundary

Orders **6,873-6,892** are exact-blob read/decomposed for TeXFont, graphics primitives, resource init/release, platform adapters and render builder/draw lifecycle. Applicable target responsibilities are bounded numeric/color/raster behavior, resource provenance, platform-consistent failure semantics and guaranteed transform/color restoration; Cairo/GDI/Qt/Skia implementations themselves are not ported. Accounting: **6,892/16,125 read; 9,233 unread; 15,845 unknown; 0 omitted**. First unread: **6,893 `Telegram/ThirdParty/MicroTeX::src/res/builtin/formula_mappings.res.cpp`**.


### Revision 9 exact read-through 6893-6900 — MicroTeX builtin resources and font provenance

Orders **6,893-6,900** are exact-blob read/decomposed for Unicode/codepoint→formula normalization, symbol/text mappings, TeX spacing/layout constants, builtin symbol taxonomy, Meson resource closure, CTAN font provenance, and the first `bi10`/`bx10` font metric/style definitions. Existing-owner-first keeps one canonical Transcript math/KaTeX owner plus Build/Release provenance; no MicroTeX parser/font/resource runtime is introduced. These responsibilities remain **mapped-open**: source reading does not establish production equivalence, and `unknown` therefore stays **15,845**. Accounting: **6,900/16,125 read; 9,225 unread; 15,845 unknown; 0 omitted**. First unread: **6,901 `Telegram/ThirdParty/MicroTeX::src/res/font/cmbsy10.def.cpp@6e72926a0156f2b06033d9995c7653b5d76b2a10`**.


### Revision 9 exact read-through 6901-6910 — MicroTeX math font metrics and fallback graph

Orders **6,901-6,910** exact-read `cmbsy10`, `cmbx10`, `cmbxti10`, `cmex10`, `cmmi10` (+ unchanged compatibility variant), `cmmib10` (+ unchanged compatibility variant), `cmr10` and `cmss10`. The files carry glyph metrics plus family/style fallback edges, larger-glyph and delimiter-extension chains, ligatures, kerns and skew metadata. Existing-owner-first keeps the sole AssistantMath/KaTeX shipping owner and Build/Release provenance; no MicroTeX font runtime or second font truth is added. All remain **mapped-open**, and source reading does not reduce `unknown`. Accounting: **6,910/16,125 read; 9,215 unread; 15,845 unknown; 0 omitted**. First unread: **6,911 `Telegram/ThirdParty/MicroTeX::src/res/font/cmssbx10.def.cpp@8b1c030f0123c71809eda15a48314298475300e7`**.


### Source closure through 6,921

Orders **6,911-6,921** are exact-blob read-complete/responsibility-decomposed for MicroTeX sans-bold/sans-italic, math-symbol, text-italic/mono, double-struck, Euler Fraktur, base italic metrics and the exact font Meson build closure. Metrics, family/style fallback, larger-glyph, ligature, kern/skew and math-alphabet semantics remain mapped-open to the existing canonical AssistantMath/KaTeX dependency interface plus Build/Release provenance owner; no second MicroTeX/font runtime is introduced. Accounting: **6,921/16,125 read; 9,204 unread; 15,845 unknown; 0 omitted**. First unread is 6,922 `Telegram/ThirdParty/MicroTeX::src/res/font/moustache.def.cpp@629e014e…`. Reading grants no implementation, verification, baseline or release credit.


### Source closure through 6,932

Orders **6,922-6,932** are exact-blob read-complete/responsibility-decomposed across moustache extensible delimiter pieces, AMS symbol/blackboard fonts, roman/sans compatibility variants, RSFS script kern data and special/sans glyph metrics. They stay mapped-open to the existing AssistantMath/KaTeX dependency interface plus Build/Release provenance owner; no second font runtime is introduced. Accounting: **6,932/16,125 read; 9,193 unread; 15,845 unknown; 0 omitted**. First unread is 6,933 `Telegram/ThirdParty/MicroTeX::src/res/font/stmary10.def.cpp@c687e395…`. Reading grants no implementation or release credit.


### Source closure through 6,948

Orders **6,933-6,948** are exact-blob read-complete/responsibility-decomposed across StMary/mono typography, font/symbol definition macros, dependency registries, resource Meson closure and AMS/base/StMary/special symbol mappings. They remain mapped-open to the existing AssistantMath/KaTeX dependency interface plus Build/Release provenance owner; no second registry or MicroTeX runtime is introduced. Accounting: **6,948/16,125 read; 9,177 unread; 15,845 unknown; 0 omitted**. First unread is 6,949 `Telegram/ThirdParty/MicroTeX::src/samples/gtkmm_main.cpp@d491af61…`. Reading grants no implementation, verification, baseline or release credit.


### Source closure through 6,962

Orders **6,949-6,962** are exact-blob read-complete/responsibility-decomposed for MicroTeX GTK/memory-check/Qt/Skia/QML/Win32 samples. Platform demo UI is not copied; applicable single-owner init/parse/render/export/failure/teardown semantics map to existing AssistantMath/KaTeX, test and Build/Release owners. Accounting: **6,962/16,125 read; 9,163 unread; 15,845 unknown; 0 omitted**. First unread is 6,963 `Telegram/ThirdParty/MicroTeX::src/utils/dict_tree.h@314b8eca…`. Reading grants no implementation or release credit.


### Source closure through 6,976

Orders **6,963-6,976** are exact-blob read-complete/responsibility-decomposed for MicroTeX utility and dependency boundaries. Binary lookup must remain one-past-end safe; byte ctype classification must use unsigned-byte domain; malformed UTF/numeric/resource input must fail locally; typed error distinctions and dependency provenance remain explicit. C++ utilities are not copied. Accounting: **6,976/16,125 read; 9,149 unread; 15,845 unknown; 0 omitted**. First unread is 6,977 `Telegram/ThirdParty/QR::Readme.markdown@66292b4b…`.

### Source closure through 7,028

Orders **6,977-7,028** are exact-blob read-complete/responsibility-decomposed for the pinned Nayuki QR component across C/C++, Java/fast-Java, Python, Rust/no-heap, Rust and TypeScript/JavaScript implementations, tests, demos and package/build inputs. Applicable invariants include versions 1-40, four ECC levels, numeric/alphanumeric/byte/Kanji/ECI segmentation, bounded capacity/version selection, optional ECC boost, mask selection/penalty scoring, Reed-Solomon block/interleave behavior, bounded module access, typed too-long/invalid-input failure, no-heap caller-buffer contracts and output border/scale bounds.

Existing-owner-first audit rejects a new QR product/state subsystem: RemoteControl already owns pairing/manual code, auth, expiry, status and persistence; SharedRoom owns share URL/expiry/room lifecycle; DeepLink owns canonical URL parsing. QR is mapped-open as a stateless representation/codec boundary behind those owners plus canonical Resource presentation. Reading grants no implementation, verification or unknown-closure credit.

Accounting: **7,028/16,125 read; 9,097 unread; 15,845 unknown; 0 omitted**. First unread is order 7,029 `Telegram/ThirdParty/TooManyCooks::.clang-format@b484c8323870e494c8031a35ddac49aa114052b3`.

### Source closure through 7,056

Orders **7,029-7,056** exact-read TooManyCooks formatting, CI, project authority and package/build contracts. The source authority requires sanitizer/platform/topology coverage, cold task semantics, executor/priority inheritance and resume affinity, mandatory await/detach linear lifetime, fork/join lifetime, coroutine-lambda capture safety, work-stealing/topology behavior and build-option consistency. These remain mapped-open to existing Fabushi Coordinator/Host/Runner/ConversationActor/CapabilityBroker plus Build/Release owners; no TMC provider/runtime is introduced. Reading closes no unknown.

Accounting: **7,056/16,125 read; 9,069 unread; 15,845 unknown; 0 omitted**. First unread is order 7,057 `Telegram/ThirdParty/TooManyCooks::include/tmc/all_headers.hpp@9f14633b57c5d4b319d1309b4bc6db643bc1822d`.

### Revision 9 deterministic read-through — TooManyCooks 7,057–7,154

Pinned authority `tzcnt/TooManyCooks@b86af81982860c96295a7e95e4c60cb335615cef` is now exact-read through all **126** non-directory entries. Orders **7,057–7,154** revalidate the prior temporary 7,057–7,064 read and cover all remaining runtime headers, implementations and package inputs.

Existing-owner-first audit found real shipping candidates: `GatewayHostSupervisor`; `HostRunnerComposition`; the frozen Host→Runner bridge; `RoutedProviderCancellation/RoutedProviderTaskRegistry`; `TurnSettlement`; canonical Mahayana messaging; `PressureCpuProfiler`; and existing GitHub Actions Build/Release owners. Applicable/open behavior includes executor/priority restoration, awaitable/fork/join lifetime, UAF-safe settlement, atomic wait/wake and teardown, foreign callback lifetime/cancellation, queue close/reclamation, CPU quota/topology/work-stealing capacity behavior, and sanitizer/fuzz/coverage/build-option consistency. Exact hwloc pinning is not presumed complete without a dedicated shipping worker-pool owner. vcpkg/CMake/version descriptors are build/provenance inputs, not a new product owner.

No TMC/Telegram runtime/provider or duplicate Identity/Conversation/Message/Search/Settings/Marketplace is introduced. Accounting is **7,154/16,125 read; 8,971 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown. Next deterministic entry is **7,155** `Telegram/ThirdParty/cld3::.github/workflows/main.yml@b26ff5ec0300683c819c0c7f17edfc21bfdd0fc3` in `google/cld3`.

### Revision 9 deterministic read-through — CLD3 7,155–7,241

Pinned `google/cld3@b48dc46512566f5a2d41118c8c1116c4f96dc661` is exact-read through all **87** non-directory entries. The component implements local language identification: bounded input selection; UTF-8/interchange repair and script spans; character n-gram + relevant-script features; embedded feed-forward neural inference; `language / probability / is_reliable` results; multilingual, empty, invalid-UTF8 and script tests; generated model parameters; and build/binding/license inputs.

Consumer tracing in accepted Telegram source finds `Platform::Language::Recognize` used by translation offer/skip policy, chat translation tracking, AI compose default source language, URL translation and Linux translation-provider flows. Current Fabushi shipping tree contains no `language/translate/spellcheck/i18n` path. Prior dossiers only define a **Canonical Translation service gateway** as mapped-open. Therefore no existing production owner can honestly be claimed.

Owner-absence decision: reject Search, Settings/Localization and Message as detector owners because they consume/filter/project language state but should not own classification. If implementation proceeds, create only an approved minimal source-neutral `TextLanguageClassifier` contract behind the canonical Translation gateway; it must be local/private by default, typed unknown/unreliable on bad/short/mixed input, bounded for long text, model/version/license pinned, and covered by multilingual + Unicode/property tests. This paragraph is an ADR requirement/proposal, not approval to add a CLD3 runtime/provider.

Accounting: **7,241 read; 8,884 unread; 15,845 unknown; 0 omitted**. Next is **7,242** `Telegram/ThirdParty/cmark-gfm::.editorconfig@12032e647502de7a91f241f75fcd7666be375875`.
