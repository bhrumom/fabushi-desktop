Live Revision 9 authority (2026-10-11): telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (root tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6). Root non-directory=6,653; recursive non-directory=16,125; deterministic read-through=8,601; first unread=8,602 Telegram/ThirdParty/hunspell::tests/checkcompoundrep2.good@737578025367c6a45d4ab904aa1c9f18d8265fe3; unread=7,524; unknown=15,845; omitted=0. Orders 8,494–8,601 revalidate Dutch IJ capitalization/suggestion and the regression/ABI gate, then exact-read affix/AF-AM morphology aliases, all-caps/homonym/UTF-8/Arabic normalization, baseline replacement/suggestion behavior, recursive/default/disabled break semantics, compound-case/duplicate-component rules, CHECKCOMPOUNDPATTERN boundary/sandhi behavior and CHECKCOMPOUNDREP regressions; source reading alone closes no unknown. The source-neutral native platform-input adapter remains in the existing main-RPC/preload/PromptRichTextEditor/Composer path; observable OS method truth is additive while language/width/scriptVariant/candidate stay typed unknown when unavailable and async samples remain scope/generation/latest-request fenced. Descendant exact-head GitHub Actions evidence is required.
<!-- TDRP_CURRENT_SUMMARY read-through=8601 unread=7524 unknown=15845 omitted=0 first-unread=8602 path=Telegram/ThirdParty/hunspell::tests/checkcompoundrep2.good -->
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


# Fabushi 完整通信能力等价重写 — Source of Truth

Status: active  
Project ID: FBCP-001  
Revision: 7  
Date: 2026-10-07  
Repository: `bhrumom/fabushi-desktop`

## Canonical statement

最终产品是一个完整 Fabushi Bot。必须逐文件、逐模块理解 `telegramdesktop/tdesktop`，把全部非 UI 代码职责用最合适语言重写进 current canonical Fabushi；C++ 产品职责默认 Rust；原 UI 改为 Fabushi UI，但 UI 文件的非 UI 逻辑及所有功能必须保留。最终同时实现现有 Bot 全部能力与完整源范围通信能力。

这不是 Telegram 绑定/Provider/外置 runtime，也不是研究后挑选少量功能。禁止“research complete”代替“production complete”。

## Normative documents

- root `AGENTS.md`
- `docs/specs/fabushi-bot-communication-platform.md` — FBCP-001 Revision 7
- `docs/specs/telegram-desktop-rust-equivalence-migration.md` — TDRP-001 Revision 9
- current canonical Bot architecture/spec and exact-head implementation
- `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md` and `module-map.md`
- `projects/fabushi-communication-platform/ui-information-architecture.md`
- `projects/fabushi-communication-platform/design-system.md`
- `projects/fabushi-communication-platform/ui-component-contract.md`
- `projects/fabushi-communication-platform/canonical-screen-patterns.md`
- `projects/fabushi-communication-platform/visual-acceptance.md`
- `projects/fabushi-communication-platform/quality/README.md` and all normative quality plans/oracles/contracts under `quality/`
- current task, existing architecture-map, dossiers, ledger, ADR and evidence

以上新 Revision 替代旧 project metadata/研究材料中 research-only、选择性吸收、where accepted 或 blocked 可算最终验收的口径。旧数据保留历史身份，不自动改为 verified；现有 schema/validator 尚需按新合同扩展。

## Hard rules

- 全仓库与递归依赖逐文件对照；文件、责任、capability、target symbol、production entrypoint 与测试双向追溯。
- 除 UI 表现实现外全部产品职责等价重写；mixed UI 文件必须拆出业务，不允许按目录整块忽略。
- current existing owner first；新 owner 只允许批准 ADR 下的最小新增。
- 无第二 Identity/Conversation/Message/Resource 真相、无平行 CommunicationCore/Telegram runtime。
- Fabushi 自有身份/通信/同步/媒体/通话/推送服务；不用 Telegram 网络不是省略服务端功能的理由。
- Human、Agent、群、频道、混合房间使用同一 shell/transcript/composer。
- 所有同类产品表面遵守 single-composition：统一 ConversationWorkspace/Profile/列表/资源/设置等 canonical root；Human/Agent/Group/Channel/新能力差异只能是 typed capability/section/renderer/action/panel/overlay。
- Telegram 中当前 Fabushi 完全没有的能力不得丢弃：先证明 existing owner 不存在，再以 ADR 新增最小 source-neutral owner，并接入统一 shell/canonical identity/resource/permission；专用 surface 不得成为第二应用。
- 微信式结构：左竖栏 + 列表/搜索 + 主工作区；消息、联系人、插件市场均为一级入口，市场复用现有 Plugins/MCP owner。
- UI 入口按作用域而非源码来源分层：一级领域 / collection / creation / object action / detail / capability surface / global command / system surface；每个可见能力只有一个主要入口与 canonical route。
- Group/Channel/Topic 等是统一 Conversation 体系的 kind/filter；新建私聊/群组/频道/Agent/混合房间复用一个 typed ConversationCreation flow，不产生独立 Group/Channel App。
- Search 只有一个 canonical owner：入口内、对象内、Universal Search 与所有 Participant/member/admin picker 复用 typed query/result/provider/permission 合同；本地/远端索引位于同一 owner 后。
- UI 必须使用 Fabushi semantic design tokens、canonical components 与 canonical screen patterns；AI/开发者不得按模块自由发明新的 visual grammar。
- 当前 recovered `sand-*` / `cursor-*` 可作为兼容实现细节，但新迁移 UI 不直接把来源命名当公共 API；目标层为 Fabushi-owned semantic aliases/wrappers。
- 核心 UI 变更必须有 current-head GitHub Actions visual regression、light/dark、locale、responsive、keyboard/a11y、reduced-motion 和状态矩阵证据。
- 功能/QA 使用 requirement→oracle/invariant→test case→execution evidence→independent verdict 的双向 RTM；没有 traceability 不得 verified。
- 动态 Agent/消息/UI 行为必须做 temporal acceptance；final/terminal 在 settlement、切换、reconnect、reload、restart 后不可消失/回滚/被 intermediate 替代。
- 人工/后阶段发现的 defect 必须形成永久 regression case 和 gap analysis；flaky/skipped/blocked/not-run 不算 pass。
- release candidate 必须由独立验收者复核 packaged timeline/screenshots/video/evidence，0 open P0/P1/blocker 才可 ACCEPT。
- 产品名/图标/窗口/文案/安装更新全部 Fabushi；无 Telegram/Grok Bot/Gok Bot 产品品牌；必要法律声明和 provenance 保留。
- 所有可执行验证仅 GitHub Actions，不沿用本项目旧 htch-runtime allowance。
- 完整迁移需要 0 in-scope open blockers、0 omitted/unmapped、0 stub/fake fallback 和 exact-head 生产/发布证据。

## Snapshot, not live truth

Revision 3 读取：main `860f03a8553c779fe006c7826f190c3014a571dc`；PR #20 已合并（merge `ee66bacdf47f36af2ae96c0a8a8ec401460426e7`）。每次执行重新读取 live refs，不能继续把 PR #20 当作未合并工作分支。

Historical predecessor authority: 上游 discovery HEAD 已 rebaseline 到 `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`（tree `5030985204963cbbd362ced7412d231b04ebd0cc`）；`42f8a36d43b8c805bc821905bea4cfeb3af1d41d` 与更早 authority 仅作历史证据。当前 recursive denominator 16,123；read-through 5,773；unread 10,350；unknown 15,844；source closure 仍 open，predecessor Actions 不得证明新 baseline。

## Next task / completion

`management/tasks/P0-product-domain-and-telegram-absorption.md`

先做新基线全量差异与 current owner 对照，再按已明确责任推进真实实现。遇到阻塞记录解除条件并推进不依赖该阻塞的下一项，不能放宽 gate。此次只修订文档，未完成模块迁移或 UI 改造；全产品完成以 FBCP-001 AC-01 至 AC-50 全部通过为准。

Historical predecessor authority: Current live authority (2026-10-09): `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (root tree `5030985204963cbbd362ced7412d231b04ebd0cc`), three commits ahead of historical `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`; 15 root paths changed (12 modified, 3 added), recursive denominator is 16,123, read-through is 5,773, unread is 10,350, unknown is 15,844, omitted is 0, and source closure remains open.


### TDRP read-through 5600-5604

Local cache/storage policy and Settings landing composition are exact-source read-complete but mapped-open. They reuse existing canonical Settings, Storage/Cache, Identity/Account, Privacy/Security, Wallet/Payments, Business and platform-adapter owners; no second Settings or storage runtime is introduced. Current accounting: read-through 5,604; unread 10,519; unknown 15,844; omitted 0.


### TDRP read-through 5605-5613

The Revision 9 validator caught and rejected a transient order drift; corrected exact order is Notifications 5605–5607, Reactions 5608–5609, Notification Type 5610–5611 and Account Passkeys 5612–5613. All remain production-open under existing source-neutral Notification/Account/Privacy/Call/platform and Account Auth/WebAuthn owners. Remote Agent WebAuthn proxy infrastructure is not credited as Account Passkeys lifecycle parity. Current accounting: read-through 5,613; unread 10,510; unknown 15,844; omitted 0.

### Current live source accounting after 5614–5616

Accepted upstream remains `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (tree `5030985204963cbbd362ced7412d231b04ebd0cc`). Deterministic read-through is **5,616 / 16,123**; unread **10,507**; unknown **15,844**; omitted **0**. Orders 5614–5616 remain mapped-open to canonical Entitlement/Subscription, Wallet/Payments/Commerce, Settings and design-system owners. First unread is 5617 `settings_privacy_security.cpp`.

### Read-through 5614–5616

Premium feature order, entitlement, subscription options/ref attribution and purchase routing map to canonical Entitlement/Subscription + Wallet/Payments/Commerce + Settings owners; no Telegram Premium runtime/UI is introduced.

### Read-through 5617–5622

Privacy/Security composition, editable keyboard shortcuts, and server-backed Website/Bot authorization sessions are exact-blob read-complete and mapped-open to existing canonical Account Security/Privacy, Authorization, Command Registry, Bot/Blocked-Peer, Payments/Retention and Settings owners. Exact authorization hash, account/session fencing, destructive confirmation/retry/idempotency and keyboard/a11y behavior remain production gates. Accounting: read-through 5,622; unread 10,501; unknown 15,844; omitted 0. First unread: 5623 `settings/settings_builder.cpp`.

### Read-through 5623–5632

Settings Builder/Common/Search/section-factory infrastructure, hidden diagnostic codes, and Credits/Gift commerce graphics are exact-blob read-complete. UI infrastructure maps to canonical Settings/UniversalSearch/design-system owners; diagnostic codes require controlled development/support policy rather than product copying; Credits/Gift actions remain mapped-open under canonical Wallet/Payments/Credits + Gift/Commerce with exact-account/id, cancel/retry/idempotency/rollback/settlement/restart gates. Accounting: read-through 5,632; unread 10,491; unknown 15,844; omitted 0. First unread 5633 `settings_experimental.cpp`.

### Read-through 5633–5642

Experimental flags/support, FAQ suggestion loading, Settings layer/focus/keyboard navigation, notification-common presentation and Power Saving are exact-blob read-complete. They map to canonical feature-flag policy, Help/Support, Settings/design-system navigation, Notification Settings and Performance/Power owners. No source-named product UI or parallel state owner is introduced. Accounting: read-through 5,642; unread 10,481; unknown 15,844; omitted 0. First unread 5643 `settings_power_saving.h`.

## Deterministic source-order correction at 5,623 and live read-through 5,653

Accepted recursive tree `5030985204963cbbd362ced7412d231b04ebd0cc` proves `Telegram/SourceFiles/settings/settings.style@8b9e56dd…` is blob order **5,623**, between `sections/settings_websites.h` (5,622) and `settings_builder.cpp` (5,624). The earlier 5,623–5,642 labels were therefore off by one and are superseded. Their shard/attestation artifacts are replaced rather than treated as parallel authority.

The corrected contiguous batches are 5,623–5,633 (settings.style through Credits/Gift graphics), 5,634–5,643 (Experimental/FAQ/Settings shell/navigation/Power Saving implementation), and 5,644–5,653 (Power Saving interface, privacy controllers, recent Settings search, scale preview, Settings search/type). Current accounting is **5,653 / 16,123 read**, **10,470 unread**, **15,844 unknown**, **0 omitted**. Reading alone closes no unknown. First unread is 5,654 `Telegram/SourceFiles/statistics/chart_lines_filter_controller.cpp@40c3b612…`.

## Deterministic statistics batch 5,654–5,665

Exact blobs for line-filter animation, ruler generation, interactive chart widget, range min/max segment tree, statistics style/common types and chart JSON deserialization are read/decomposed. They map to canonical Analytics/Insights data projection + reusable Chart/design-system owners; business/account truth must not live in the chart. JSON parse errors, empty columns and column-length mismatches fail closed. Hover/zoom/filter/footer state is derived and lifetime-bounded. Accounting: **5,665/16,123 read; 10,458 unread; 15,844 unknown; 0 omitted**. First unread: 5,666 `statistics_format_values.cpp`.


## Deterministic statistics formatting/export batch 5,666–5,674

Exact blobs for locale/timezone analytics labels, TON/Credits chart graphics, analytics-to-sheet projection, 64-bit chart values, and XLSX/OpenXML serialization are read/decomposed. They map to canonical Analytics/localization, Wallet/Credits visual projection and Export owners. Export must preserve inline-string formula-injection safety, XML/control escaping, legal unique sheet names, empty/short-series behavior, numeric/date boundaries, ZIP failure, cancellation/account fencing and cross-platform save behavior. No Telegram-derived statistics/export runtime or UI is introduced. Accounting: **5,674/16,123 read; 10,449 unread; 15,844 unknown; 0 omitted**. First unread: 5,675 `statistics/view/abstract_chart_view.cpp@4b5f2929…`. Reading alone closes no unknown.


## Deterministic statistics chart-view batch 5,675–5,684

Exact blobs for chart-view abstraction, double-line ratios, bar/stack bars, animated dual-axis/currency rulers, closed chart-type dispatch and linear/DPR cache + pointer hit-testing are read/decomposed. They remain mapped-open to one canonical Analytics/Chart design-system owner. The Fabushi port must strengthen empty/one-point, zero-range/max, all-disabled, mismatched-series, invalid-type and pointer-endpoint cases fail closed; rendering state owns no analytics truth. Accounting: **5,684/16,123 read; 10,439 unread; 15,844 unknown; 0 omitted**. First unread: 5,685 `statistics/view/stack_chart_common.cpp@82a1143f…`.


## Deterministic statistics stack/pie batch 5,685–5,690

The remaining statistics/view blobs are read/decomposed: stack geometry/index mapping, filtered pie-percentage rounding, and stack-linear local zoom/pie transition/selection. They remain mapped-open to the canonical Analytics/Chart owner. Canonical code must fail closed for <2 points, zero-width ranges/denominators, invalid indices/line lengths, zero totals and stale/racing zoom/filter/hover state. Accounting: **5,690/16,123 read; 10,433 unread; 15,844 unknown; 0 omitted**. First unread: 5,691 `statistics/widgets/chart_header_widget.cpp@b8754ceb…`.


## Deterministic statistics widgets batch 5,691–5,696

Statistics widgets are exact-blob read/decomposed: responsive chart header, line-filter controls with last-visible-line guard, and localized/currency point details with zoom/cache/ripple state. They map only to canonical source-neutral Analytics/Chart + existing design-system controls. Empty/mismatched data, invalid indices, RTL/long text, currency precision, a11y/focus/theme/DPR and teardown remain production gates. Accounting: **5,696/16,123 read; 10,427 unread; 15,844 unknown; 0 omitted**. Next order 5,697 must be rebound from the accepted recursive tree before recording its path.


## Deterministic PCH + storage security/migration batch 5,697–5,701

Order 5,697 maps C++ PCH/platform composition to canonical cross-platform build evidence. Orders 5,698–5,701 map critical durable local-file protocol, Argon2id/scrypt/legacy passcode KDFs, encryption/signature/version checks and versioned settings/session migrations to canonical Persistence + Local Security/Key Protection + Settings Migration owners. Invalid/over-cost KDFs, corrupt/truncated/unknown blocks, interrupted writes, wrong passcodes and legacy/account/restart recovery must fail closed. This does not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. Accounting: **5,701/16,123 read; 10,422 unread; 15,844 unknown; 0 omitted**. First unread: 5,702 `storage/download_manager_mtproto.cpp@d0007eb1…`.


### TDRP Revision 9 source read 5702-5711 — storage transfer lifecycle

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,702-5,711 cover adaptive remote download scheduling, generic cache/file loader lifecycle, part-based download/resume, HTTP(S)-only web transfer and multipart media upload. Canonical mapping is the existing source-neutral messaging attachment/file-transfer owner plus platform download/save adapters; MTProto/DC/CDN details are protocol-specific and must not create a Telegram runtime. Required parity includes cancel/teardown/account-switch stale-result fencing, partial resume, integrity/reference refresh, redirect downgrade/scheme protection, TLS/auth/disk failures, adaptive concurrency, removed-session resend, transcode/archive cancellation, upload progress/finalization and restart/recovery. Accounting: 5,711/16,123 read, 10,412 unread, 15,844 unknown, 0 omitted; reading alone closes no unknown. First unread: 5,712 `Telegram/SourceFiles/storage/localimageloader.cpp`.

### TDRP Revision 9 source read 5712-5721 — media preparation and durable serialization

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,712–5,713 cover source-neutral media-send preparation and worker/result lifecycle; 5,714–5,717 cover local settings/theme/language/update persistence, legacy encrypted migration and bounded serialization primitives; 5,718–5,721 cover versioned document/media and user/chat/channel projection persistence with legacy location migration and fail-closed corruption handling. Canonical mapping stays with existing messaging Attachment/media-preparation, Persistence/Settings/Local Security/Updater/Theme/Localization, messaging data persistence, and Identity/Profile/Conversation owners. Telegram/MTP file formats and flags are migration inputs only and must not become a parallel runtime or product UI. Required closure includes cancel/account-switch/teardown stale-result fencing, malformed media/encode/archive failures, wrong-key/tamper/truncation/future-version handling, interrupted atomic writes, legacy migration, cache-vs-authority reconciliation, reload/restart and canonical UI/a11y evidence. Accounting: **5,721/16,123 read, 10,402 unread, 15,844 unknown, 0 omitted**; reading alone closes no unknown. First unread: **5,722 `Telegram/SourceFiles/storage/storage_account.cpp@5888ffa1164327b7170a2a3d8df7793a3fa467da`**.

### TDRP Revision 9 source read 5722-5729 — account/domain storage and local security

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,722–5,723 cover encrypted account-scoped drafts/cache/settings/trust/payment/webview/bot/wallet persistence; 5,724–5,725 cover bounded cloud-blob download/extraction; 5,726–5,727 define passcode wraps, bounded memory-hard derivation, secret cleansing, fresh verification tokens, App Lock mutation, crash-safe committed generations, legacy migration and wallet keyring protection; 5,728–5,729 route shared-media/profile-photo cache queries and invalidation. The local-security rows strengthen but do not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. All remain mapped-open to canonical source-neutral owners; no Telegram storage/security runtime or UI family is permitted. Production evidence must cover wrong/unsupported KDF, stale verification, passcode/App Lock mutation, crash/restart migration, wallet corruption/absence, account fencing, delayed-write teardown, cloud/archive failure, cache invalidation and canonical UI/a11y. Accounting: **5,729/16,123 read, 10,394 unread, 15,844 unknown, 0 omitted**. First unread: **5,730 `Telegram/SourceFiles/storage/storage_file_lock.h@181b5bc063827458c4e59a0daa167355f835ae0b`**.

### TDRP Revision 9 source read 5730-5740 — file locking, archive/media preparation and sparse indexes

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,730–5,732 define exclusive storage locking on POSIX/Windows, including conflict-process handling; 5,733–5,734 define folder/multi-file ZIP preparation with symlink exclusion, progress/cancel, size/disk failure and stale temporary cleanup; 5,735–5,736 cover Composer/editor media drag/drop, MIME/limit/decode and asynchronous preview preparation; 5,737–5,740 define shared-media categories and sparse message-id range merge/query/count/invalidation. All remain mapped-open to canonical Persistence/platform adapters, Attachment/Media Editor and Shared Media/index owners. Required evidence includes cross-platform lock conflict/retry, archive cancellation/limit/disk failure/cleanup, malformed media and stale async results, and sparse pagination/count consistency. Accounting: **5,740/16,123 read, 10,383 unread, 15,844 unknown, 0 omitted**. First unread: **5,741 `Telegram/SourceFiles/storage/storage_user_photos.cpp@232befde40c5edea34286105374058fcc66ab658`**.

### TDRP Revision 9 source read 5741-5754 — user-photo cache, streamed download and operator support

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,741–5,742 cover profile-photo history cache list/count/paging; 5,743–5,744 cover bounded aligned streaming-part download with duplicate suppression, cancellation and completion; 5,745–5,754 cover operator/support template autocomplete/search, contact confirmation, shortcut navigation, occupation leases, support-info editing, history preload, fast-button preferences and template update/reload. Support-specific code is conservatively mapped-open pending explicit applicability evidence rather than silently omitted. Required evidence includes profile count/paging consistency, part failure/cancel/staleness, support account/history fencing, edit request replacement, occupation expiry/race, preload retry and malformed/update-failed template recovery plus canonical keyboard/focus/a11y. Accounting: **5,754/16,123 read, 10,369 unread, 15,844 unknown, 0 omitted**. First unread: **5,755 `Telegram/SourceFiles/tde2e/tde2e_api.cpp@efe4edc5eef735c954b39d7a927722a7e8f9214c`**.

### TDRP Revision 9 source read 5755-5758 — call end-to-end encryption

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,755–5,758 own call E2EE temporary/private key lifecycle, one-shot ECDH envelopes, encrypt/decrypt callbacks, participant/state blocks, inbound/outbound queues, subchain reconciliation, failure terminality and emoji verification hash, with protocol conversion as an adapter. They remain mapped-open to canonical Calls E2EE/Crypto + Call Session owners; transport TLS is not a substitute. This implementation batch also hardens the canonical Wallet ledger so reuse of an idempotency request id must match operation kind, accounts, normalized currency, amount and reference; conflicting credit/transfer/refund reuse fails closed with `RequestConflict` and unit coverage. Broader wallet rows remain open pending full production/evidence closure. Accounting: **5,758/16,123 read, 10,365 unread, 15,844 unknown, 0 omitted**. First unread: **5,759 `Telegram/SourceFiles/test/README.md@429b4f3d83ec6fa6aa71203624a894d1d7973d35`**.

### TDRP Revision 9 source read 5759-5771 — professional task-test harness

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,759–5,771 define the upstream task-test authority: pure bounded readiness; exact object/action identity; assertions outside waits; diagnostic timeouts; precondition-only N/A; packed-scenario teardown; protected fixture secrets; deterministic activation/focus and animation timing; exact dialog-shell interaction; visual capture of real paint owners with DPR/scale boundaries; and privacy-safe clipboard classification/retry without logging foreign clipboard contents. These map to Fabushi's professional cross-platform GitHub Actions test/evidence harness, not to production Telegram UI. The same descendant repairs current-head authority failures by correcting order 5,739 to accepted blob `cc0e3ed6b2c763326c2c16635a7671a747ef506b` and refreshing the strict Grok semantic-adaptation target for `source/electron-main/attachments/attachments.ts` to production blob `9fb1ed46b2ae623e7fa781d6d3ab47f465f4a332`; no gate is weakened. Orders 5,772-5,773 cover the Windows console-lock professional-test contract: the implementation reads process-session/WTS metadata and classifies locked/unlocked/unknown, while the header fixes the public NotApplicable/Unlocked/Locked/Unknown state model, makes `locked()` true only for Locked, and requires `ConsoleLockGate` to remain empty for Unknown and NotApplicable. Only a positive locked reading gates; documented legacy-Windows reversed flags remain unknown; diagnostics never read or print user/domain/window-station fields; non-Windows is explicit N/A. This maps to the canonical professional cross-platform test/evidence harness and creates no production Telegram UI. Accounting: **5,773/16,123 read, 10,350 unread, 15,844 unknown, 0 omitted**. Reading alone closes no unknown. First unread: **5,774 `Telegram/SourceFiles/test/test_corner_patch.cpp@a34785a050cdeea405c3948345704d2e19395843`**.



### cf478d37 live-authority rebaseline

Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

Direct delta: channel_earn.style replaces static negative placeholder margins with floating placeholderShiftLeft; lib_ui 91ff446 implements horizontal interpolation in InputField and MaskedInputField. Canonical TextField remains the sole product owner. Orders 5,774-5,793 were identity/order reconciled; reading alone closes no unknown.


### Revision 9 live read-through note: 5794-5801

Orders 5,794-5,797 preserve professional evidence-log one-line/completion-forgery integrity plus an independent raw-byte oracle. Orders 5,798-5,799 preserve complete mapped-target/viewport capture readiness. Orders 5,800-5,801 preserve the bounded reversible not-marking-read evidence lever. These are test/evidence responsibilities only, create no second product owner, and close no unknown. The new 811b83a1 readability delta is order 5,951 and remains unread/unknown.


### Revision 9 live read-through note: 5802-5805

Orders 5,802-5,803 close the exact-blob read/decomposition of popup/context-menu professional evidence semantics: same-turn fresh-menu identity, explicit refusal taxonomy, isolated QAction queued-callback delivery, prepared-frame capture and lock/teardown behavior. Orders 5,804-5,805 close the SentMessageWatcher client→server id reconciliation contract with stable-history/candidate fencing and a five-second diagnostic probe throttle. These are test/evidence responsibilities only and close no unknown. First unread is now 5,806 `test_notify_override.cpp`.

### Revision 9 live rebaseline/read-through note: b0d1fe5e / 5166 / 5806-5813

The 811b83a1→b0d1fe5e upstream delta changes only order 5,166 `media_view_overlay_widget.cpp`: the old `forbidsSaving()` control-refresh branch hid controls and could call `DocumentSaveClickHandler::Save(...ToCacheOrFile)` when media was not loaded; b0d1fe5e removes that viewer-driven automatic persistence path. The new blob `e35f67119323850a3cea450548566186c8a2231c` was re-read and the mapped-open MediaViewer lifecycle responsibility now explicitly requires protected/forbids-saving media not to be auto-persisted as a rendering/control-refresh side effect. Path order and total counts are unchanged. Separately, orders 5,806–5,813 are read-complete/responsibility-decomposed professional test harnesses for reversible notify overrides/no-server-write locality, document-open handoff inspection without OS launch, SeparatePanel pointer/liveness evidence, and post-paint temporal sampling. They add no product owner, fake fallback, unknown closure or release credit. First unread is 5,814 `test_probe.cpp`; read-through is 5,813/16,125, unread 10,312, unknown 15,846, omitted 0.


### TDRP read-through 6424-6437

Exact accepted blobs for Window favorite/filter navigation, history overlay dismissal, lock/security, main-menu composition and helpers are read/decomposed through order 6,437. Responsibilities remain mapped-open to existing source-neutral ProductShell, Account/Security, Conversation/ReadState, Settings, Marketplace/MiniApp/Plugin and Resource owners; reading alone does not reduce global unknown. Current accounting: **6,437/16,125 read; 9,688 unread; 15,846 unknown; 0 omitted**. First unread is 6,438 `window/window_media_preview.cpp@4fe0e7f11298b0b56c99c47e43c27c6fd47d0282`.

### TDRP read-through 6438-6448

Exact accepted blobs are read/decomposed through order 6,448. Media preview, canonical peer commands, window restore and detachable-window identity remain mapped-open to existing source-neutral owners. Main-window state persistence is partial parity, not multi-window restore evidence. Accounting: **6,448/16,125 read; 9,677 unread; 15,846 unknown; 0 omitted**. First unread is 6,449 `window/window_session_controller.cpp@df216237e47d22c200a51b2c7fda31175682d3c5`.

### TDRP read-through 6449-6458

Exact accepted blobs are read/decomposed through order 6,458. Session-scoped navigation/composition, deep-link intents, email enrollment, motion/adaptive top bar and unlock-passcode semantics remain mapped-open to canonical owners. Accounting: **6,458/16,125 read; 9,667 unread; 15,846 unknown; 0 omitted**. First unread is 6,459 `Telegram/Telegram.plist`.

### TDRP read-through 6459-6483

macOS bundle, entitlement and icon-source responsibility is read/decomposed through order 6,483. Only source-neutral packaging/security/icon coverage migrates; Telegram/TON identity and artwork do not. Current Fabushi packaging is the existing owner and permission expansion remains least-privilege/applicability-gated. Accounting: **6,483/16,125 read; 9,642 unread; 15,846 unknown; 0 omitted**.

### TDRP read-through 6484-6505

Root ThirdParty gitlinks are provenance/role-read through order 6,505. Nested third-party source remains independently open and no Telegram/Qt runtime is adopted by default. Accounting: **6,505/16,125 read; 9,620 unread; 15,846 unknown; 0 omitted**.

### TDRP read-through 6506-6541

Build/release tooling is read/decomposed through 6,541 as release-security, supply-chain, install/update and provenance responsibility, not excluded as mere tooling. Telegram-specific private infrastructure is replaced by current Fabushi GitHub Actions/electron-builder/updater/platform owners. Accounting: **6,541/16,125 read; 9,584 unread; 15,846 unknown; 0 omitted**.

### Recursive component read-through 6,654-6,697 — Microsoft/GSL

Orders **6,654-6,697** exact-read the pinned `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37` component mounted at `Telegram/ThirdParty/GSL`. The 44 entries cover repository/build/CI metadata, license/security/provenance, header-only safety contracts (bounds and extent checks, non-null ownership, checked narrowing, byte/string boundaries, scope cleanup), and focused fail-closed tests. Applicable semantics map to existing Fabushi Build/Quality/Provenance and Rust/TypeScript/platform-boundary owners; no Telegram/GSL-specific runtime is introduced. Reading does **not** reduce `unknown`; production/shipping evidence remains required. Accounting: **6,697/16,125 read; 9,428 unread; 15,845 unknown; 0 omitted**. Next: **6,698 `Telegram/ThirdParty/MicroTeX::.github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764`**.

### Recursive component read-through 6,698-6,714 — MicroTeX entry/build contracts

Orders **6,698-6,714** exact-read the pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f` build/entry contract under `Telegram/ThirdParty/MicroTeX`. The applicable product behavior maps into the existing ConversationWorkspace/Transcript math owner and existing Build/Resource/Release provenance owners. Current production work adds a 32 KiB pre-parse formula bound and bounded 32 MiB LRU markup cache without retaining the C++/Qt/GDI runtime. Reading does **not** reduce `unknown`; current exact-head GitHub Actions evidence is still required. Accounting: **6,714/16,125 read; 9,411 unread; 15,845 unknown; 0 omitted**. Next: **6,715 `Telegram/ThirdParty/MicroTeX::readme/example_bw_false.svg@9e812c823d3cbe06ab43ee77f10b6122bd638dd`**.


### TDRP read-through 6831-6852

MicroTeX unit/color, box/layout, formula/environment and glue contracts are exact-read through order 6,852. Applicable finite-value, bounded-work, failure-isolation and deterministic layout semantics stay in the existing ConversationWorkspace/Transcript KaTeX owner; build/config responsibilities stay with existing Build/Release provenance. No MicroTeX runtime is introduced. Accounting: **6,852/16,125 read; 9,273 unread; 15,845 unknown; 0 omitted**. First unread: **6,853 `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83`**.

### Recursive component read-through 6,853 — MicroTeX localized decimal input

Order **6,853** exact-reads `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83` from pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`. Its applicable responsibility is deterministic numeric input normalization: U+066B becomes the ASCII decimal point and the twenty accepted Unicode decimal blocks map digit-for-digit to ASCII while every unrelated code point remains unchanged. The existing canonical ConversationWorkspace/Transcript KaTeX owner performs that normalization before both strict and recovery renders, and the focused Desktop Chat Parity contract covers all twenty blocks, unchanged non-target text, and identical strict/recovery input. No MicroTeX parser/runtime is retained. Reading does **not** reduce `unknown`; descendant exact-head GitHub Actions remain required. Accounting: **6,853/16,125 read; 9,272 unread; 15,845 unknown; 0 omitted**. Next: **6,854 `Telegram/ThirdParty/MicroTeX::src/core/macro_def.cpp@0ccf9fe8b4028f97c62834555d2a3d344d0f8726`**.

### Live rebaseline 28ac5769 → 6fed91ff

Accepted upstream advanced by four commits to `6fed91ffab9861771a75f29df65031b3c80b6941` / root tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6` without adding or removing a root path. Ten root blobs changed at deterministic orders **3,155, 3,201, 3,202, 3,766, 4,521, 4,522, 5,166, 5,943, 6,541, 6,627**. Each changed blob was re-read against its predecessor object and reconciled through the explicit source-disposition rebaseline record. Version/resource/build changes preserve source-neutral release-version coherence. The history-photo delta adds owned enlarge eligibility/hit-testing plus press/release ripple lifecycle and keeps sponsored/editor/small-media exclusions; it maps to the existing Transcript/media interaction owner rather than a new media root. The media-viewer delta explicitly claims Save/SaveAs at the application event-filter boundary so competing shortcuts cannot consume Ctrl+S; it maps to the existing canonical media-viewer shortcut owner. The new style opacity is presentation-only under the Fabushi design system. The 7.3.1 changelog adds wallet Windows Hello/passcode, copy-restricted download, custom-theme date-color and photo-editor Link regression cues; those cues remain open until their existing Fabushi owners have shipping evidence. Recursive totals/order are unchanged, so read-through remains **6,853/16,125**, first unread remains **6,854**, unread **9,272**, unknown **15,845**, omitted **0**.

### Revision 9 deterministic read-through — TooManyCooks 7,057–7,154

Pinned authority `tzcnt/TooManyCooks@b86af81982860c96295a7e95e4c60cb335615cef` is now exact-read through all **126** non-directory entries. Orders **7,057–7,154** revalidate the prior temporary 7,057–7,064 read and cover all remaining runtime headers, implementations and package inputs.

Existing-owner-first audit found real shipping candidates: `GatewayHostSupervisor`; `HostRunnerComposition`; the frozen Host→Runner bridge; `RoutedProviderCancellation/RoutedProviderTaskRegistry`; `TurnSettlement`; canonical Mahayana messaging; `PressureCpuProfiler`; and existing GitHub Actions Build/Release owners. Applicable/open behavior includes executor/priority restoration, awaitable/fork/join lifetime, UAF-safe settlement, atomic wait/wake and teardown, foreign callback lifetime/cancellation, queue close/reclamation, CPU quota/topology/work-stealing capacity behavior, and sanitizer/fuzz/coverage/build-option consistency. Exact hwloc pinning is not presumed complete without a dedicated shipping worker-pool owner. vcpkg/CMake/version descriptors are build/provenance inputs, not a new product owner.

No TMC/Telegram runtime/provider or duplicate Identity/Conversation/Message/Search/Settings/Marketplace is introduced. Accounting is **7,154/16,125 read; 8,971 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown. Next deterministic entry is **7,155** `Telegram/ThirdParty/cld3::.github/workflows/main.yml@b26ff5ec0300683c819c0c7f17edfc21bfdd0fc3` in `google/cld3`.

### Revision 9 deterministic read-through — CLD3 7,155–7,241

Pinned `google/cld3@b48dc46512566f5a2d41118c8c1116c4f96dc661` is exact-read through all **87** non-directory entries. The component implements local language identification: bounded input selection; UTF-8/interchange repair and script spans; character n-gram + relevant-script features; embedded feed-forward neural inference; `language / probability / is_reliable` results; multilingual, empty, invalid-UTF8 and script tests; generated model parameters; and build/binding/license inputs.

Consumer tracing in accepted Telegram source finds `Platform::Language::Recognize` used by translation offer/skip policy, chat translation tracking, AI compose default source language, URL translation and Linux translation-provider flows. Current Fabushi shipping tree contains no `language/translate/spellcheck/i18n` path. Prior dossiers only define a **Canonical Translation service gateway** as mapped-open. Therefore no existing production owner can honestly be claimed.

Owner-absence decision: reject Search, Settings/Localization and Message as detector owners because they consume/filter/project language state but should not own classification. If implementation proceeds, create only an approved minimal source-neutral `TextLanguageClassifier` contract behind the canonical Translation gateway; it must be local/private by default, typed unknown/unreliable on bad/short/mixed input, bounded for long text, model/version/license pinned, and covered by multilingual + Unicode/property tests. This paragraph is an ADR requirement/proposal, not approval to add a CLD3 runtime/provider.

Accounting: **7,241 read; 8,884 unread; 15,845 unknown; 0 omitted**. Next is **7,242** `Telegram/ThirdParty/cmark-gfm::.editorconfig@12032e647502de7a91f241f75fcd7666be375875`.
