# cmark-gfm 7,242–7,307 complete read — 6fed authority

Status: read-complete / responsibility-decomposed / mapped-open / not verified  
Spec: TDRP-001 Revision 9 / FBCP-001 Revision 7  
Accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`  
Pinned component: `desktop-app/cmark-gfm@d7d4a24a9ebfa7581994ea6df0297662bfeaf413`  
Orders: `7,242–7,307` (66 exact non-directory entries)

## Shipping reachability

The component is not dead ThirdParty/build noise. Accepted Telegram shipping code directly includes and links it:

- `Telegram/SourceFiles/iv/markdown/iv_markdown_parse_convert.h` includes `<cmark-gfm.h>`.
- `iv_markdown_parse_finalize.cpp` drives cmark parser options/extensions, including source positions, footnotes and strikethrough behavior.
- `iv_markdown_parse_convert.cpp` consumes core and GFM node identities, task-list checked state, table columns/alignment/header rows and source ranges.
- `Telegram/cmake/td_iv.cmake` links `desktop-app::external_cmark_gfm` into the IV Markdown shipping composition.
- `ParseLimitsForIv()` bounds source bytes to 4 MiB, cmark nodes to 100,000, nesting to 128, formula bytes to 64 KiB and formula count to 10,000; `RunBudgetedCmarkParse` adds a parser heap budget and fail-cleanup path.

## Source groups read

- **7,242–7,254 — build/license/documentation:** cross-platform Linux/macOS/Windows CI, shared/static builds, CMake/NMake, large-file offsets, ASan/fuzzer build modes, exact license/provenance and parser capability documentation.
- **7,255–7,261 — API contract tests:** AST construction/mutation, iterators, hierarchy, custom nodes, parser feeds, renderers, UTF-8/replacement behavior, line endings, entities, pathological regressions and source positions.
- **7,262–7,291 — benchmark/security/Unicode inputs:** flat/nested block and inline cases, explicit worst-case emphasis/link/reference inputs, benchmark statistics, security changelog and Unicode 9.0 case-fold data.
- **7,292–7,307 — shipping GFM extensions:** extension registration/scanners, autolink, strikethrough, tables, tag filtering and task lists.

Every row carries exact component repository, commit, path, object/blob SHA, mode, type and size in `inventory/source-dispositions/7242-7307.json`.

## Applicable behavior / security contracts

### Autolink
Preserve safe protocol/domain recognition, delimiter trimming, balanced-parenthesis handling, host/domain underscore rules, email/mailto/xmpp distinctions, no nested autolinking inside an existing link, source positions and the accepted anti-quadratic guard for very long multi-segment hosts.

### Tables
Preserve table/header/cell identity, column count and alignment, escaped-pipe parsing, inline cell content, source positions and ownership cleanup. The accepted source explicitly caps autocompleted cells (`MAX_AUTOCOMPLETED_CELLS = 0x80000`) to prevent malicious input from causing denial-of-service work amplification.

### Strikethrough / task lists / tag filter
Preserve typed strikethrough delimiter semantics; task-list checked state must come only from the matched task prefix (body text containing `[x]` cannot flip state); dangerous raw HTML tag families are filtered before privileged rendering. The tag filter covers `title`, `textarea`, `style`, `xmp`, `iframe`, `noembed`, `noframes`, `script` and `plaintext`.

### Complexity and memory safety
The accepted changelog records fixes for multiple polynomial-time complexity/DoS advisories, crafted Markdown crashes, integer overflow/heap corruption, autolink and table denial-of-service paths, plus quadratic fuzzing. Equivalent replacement work therefore needs adversarial complexity, memory-budget, overflow and malformed-input oracles; choosing a different parser dependency is not by itself evidence of equivalence.

## Existing-owner-first audit
No cmark/Markdown/Telegram runtime root is authorized. Existing canonical targets remain:
- `frontend/src/recovered/features/conversation/workspace/transcript.tsx` and the existing rich-content projection;
- existing TranscriptCard URL/action owners for prepared links;
- existing Resource/Artifact/media owners for derived content;
- existing Find-in-chat/Search owners for document/transcript search behavior;
- GitHub Actions Build/Release quality gates for sanitizer, fuzz/performance/security and cross-platform evidence.

The current repository has IV Markdown dossiers mapping these behaviors into the same owners. This batch does not create a second Conversation, Message, Search, Resource, Settings or Marketplace truth.

## Oracles added by this read
- **ORA-TDRP-CMARK-BOUNDED-001:** hostile Markdown must settle as bounded success/failure without unbounded CPU/heap amplification.
- **INV-TDRP-CMARK-AUTOLINK-001:** link recognition cannot broaden unsafe schemes, create nested links or reintroduce accepted quadratic domain scanning.
- **INV-TDRP-CMARK-TABLE-001:** malformed/sparse tables cannot overflow column accounting or exceed the accepted bounded autocompletion budget.
- **INV-TDRP-CMARK-TASK-001:** task checked state is derived only from the task-list prefix.
- **INV-TDRP-CMARK-TAGFILTER-001:** dangerous raw HTML tag families cannot enter a privileged rendering path through Markdown parsing.
- **INV-TDRP-CMARK-SOURCEPOS-001:** canonical typed nodes that participate in search/selection/navigation preserve deterministic source-range identity where that behavior is exposed.

## Evidence status
This is source-reading/mapping evidence only. Exact-head production implementation, focused parser/security/performance tests, required GitHub Actions, packaged temporal/a11y evidence and independent release acceptance remain open. Reading these 66 entries closes **zero** unknown rows.

Next unread: `7,308 Telegram/ThirdParty/cmark-gfm::fuzz/CMakeLists.txt@a9ed57aa57bacc79907698bdeb13bc90e73b8f6d`.
