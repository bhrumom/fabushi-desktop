# cmark-gfm 7,308–7,403 complete read — 6fed authority

Status: read-complete / responsibility-decomposed / mapped-open / not verified  
Spec: TDRP-001 Revision 9 / FBCP-001 Revision 7  
Accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`  
Pinned component: `desktop-app/cmark-gfm@d7d4a24a9ebfa7581994ea6df0297662bfeaf413`  
Orders: `7,308–7,403` (96 exact non-directory entries)  
Together with `7,242–7,307`, the pinned cmark-gfm component is now `162/162` exact-read.

## Shipping reachability and owner decision

Telegram IV Markdown is the shipping consumer: it includes cmark-gfm, attaches GFM extensions, converts cmark AST nodes into its own rich document model and applies host parse limits/budget. Fabushi must not retain a cmark/Telegram runtime, provider, wrapper or second rich-content root. Applicable behavior maps to existing `ConversationTranscript` / rich-content, TranscriptCard URL/action, Resource/Artifact, Find/Search and Build/Release owners.

The CLI renderers, man pages, wrappers and legacy build scripts are not themselves Fabushi product owners. They are retained as conformance/provenance/performance evidence only where they constrain the shipping behavior or its validation.

## Core parser lifecycle

- `src/blocks.c` carries feed/reentrant-feed, container matching/finalization, source line/column state, parser reset/dispose and host-owned allocation-abort polling.
- The fork explicitly bounds new list nesting with `MAX_LIST_DEPTH=100` and per-line block-quote creation with `MAX_BLOCK_QUOTE_DEPTH=512`.
- `src/parser.h` caps link labels at 1,000 and preserves the host-owned abort flag across parser reset.
- `src/arena.c` defines aligned slab allocation and requires deterministic arena reset/pop after parse lifecycle completion.
- Parse abort/failure is not allowed to strand the AST, reference maps, delimiter stacks or extension state.

## Ownership / UAF / teardown

- `node.c` rejects cross-allocator containment and ancestor/cycle insertion, and tears trees down iteratively rather than recursively.
- user-data and extension opaque data have explicit destruction hooks.
- `footnotes.c` deliberately unlinks all footnote nodes before freeing the map because unused/cyclic footnotes can otherwise trigger use-after-free.
- `iterator.c` defines enter/exit traversal and safe text-node consolidation/ownership conversion.

These are lifecycle contracts, not implementation details to discard when replacing the C parser.

## Unicode, entities and escaping

- UTF-8 validation follows RFC 3629; invalid sequences settle deterministically instead of leaking malformed bytes through the model.
- invalid numeric entities (zero, surrogate range, or >= U+110000) become U+FFFD.
- reference labels use Unicode case-folding plus trim/whitespace normalization; ctype behavior is locale-independent.
- HTML/HREF escaping and entity unescaping are context-sensitive and bounded; entity names are length-bounded.
- scanner contracts include explicit dangerous-URL recognition and raw-HTML lexical boundaries.

## Raw HTML / URL safety

The reference HTML renderer omits raw HTML unless unsafe behavior is explicitly enabled, and applies extension filters when enabled. Fabushi does not need to reuse this renderer, but equivalent policy must hold before canonical rich-content enters a privileged renderer. Untrusted Markdown cannot gain HTML/script/unsafe-URL authority merely because a parser recognizes it.

## Fuzz / pathological / spec oracles

- dedicated quadratic fuzzers cover repeated links, brackets and extension-enabled parser/render paths;
- the generic fuzz harness exercises parser plus CommonMark/HTML/LaTeX/man/XML renderers under multiple options/extensions;
- the fuzz runner uses sanitizer/leak checks and a one-second per-input timeout;
- pathological tests require bounded completion and include nested emphasis, unmatched link brackets, hard link/emphasis cases, deeply nested brackets, 50k-scale block quotes/lists and reference collisions;
- the accepted fork's block-quote cap is encoded in the pathological expected output;
- the full GFM/CommonMark spec, extension interop, regression corpus, entity checks, smart punctuation, roundtrip and HTML-normalization suites are reference oracles;
- regressions include table buffer-overread, Windows EOL, link/emphasis corner cases, malicious footnote href escaping and extension interactions.

## Extension / registry responsibility

The plugin/registry/syntax-extension files express a typed extension lifecycle: register -> attach -> callbacks/custom node kinds -> postprocess/render/filter -> release private/opaque state. Fabushi should preserve typed extension behavior where applicable, but it must not create a cmark plugin subsystem or duplicate Marketplace/MCP ownership.

## New Revision 9 oracles / invariants

- **INV-TDRP-CMARK-LIFECYCLE-001** — parser/AST/footnote teardown is leak/UAF safe, including abort/failure paths.
- **INV-TDRP-CMARK-DEPTH-001** — adversarial list/block-quote/link/emphasis nesting has explicit bounded-work behavior and cannot cause stack overflow or superlinear denial of service.
- **INV-TDRP-CMARK-UTF8-001** — invalid UTF-8 and illegal entity codepoints settle deterministically with locale-independent classification.
- **INV-TDRP-CMARK-URLSAFE-001** — Markdown projection cannot expose dangerous URLs/raw HTML without explicit product policy; escaping/filtering precedes privileged projection.
- **INV-TDRP-CMARK-FOOTNOTE-001** — footnote labels/backrefs are escaped and stable, while teardown is cycle/UAF safe.
- **INV-TDRP-CMARK-TREE-001** — tree mutation enforces one-owner allocator/acyclic topology and deterministic source-range identity.
- **INV-TDRP-CMARK-REFMAP-001** — references use deterministic Unicode casefold/whitespace normalization and overflow-safe bounded accounting.
- **INV-TDRP-CMARK-EXTENSION-001** — extension behavior is typed and deterministically composed/released without creating a source-shaped product subsystem.
- **ORA-TDRP-CMARK-FUZZ-001** — focused replacement fuzz/pathological tests must cover crash/leak/timeout/complexity behavior in GitHub Actions.
- **ORA-TDRP-CMARK-ROUNDTRIP-001** — adopted source-neutral rich-content parsing has normalization/roundtrip oracles for the applicable CommonMark/GFM subset.

## Evidence status

This completes source reading/decomposition of the pinned cmark-gfm component only. It does **not** mark the parser responsibilities verified and closes **zero** unknown rows. Production replacement, shipping composition, focused lifecycle/security/complexity tests, exact-head GitHub Actions evidence, packaged temporal/a11y checks and independent release acceptance remain open.

Next unread: `7,404 Telegram/ThirdParty/expected::.appveyor.yml@f63c5b6c4517a723f0a99228ad3c057222e71e40`.
