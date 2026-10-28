# desktop-app/MicroTeX readme/example_cjk_trying.svg complete read — strict195

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: f53f8fdb2ea352c7800469a843de6edf3c8adabc
- Pre-receipt tree: 2438ae0ee2470e8f9efa105253bef6e9863e4b82
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Target: readme/example_cjk_trying.svg@3434f8ef08e6cb63a0fa379cf46ef07b03ca3454, mode 100644, 17,746 bytes, 55 line slots.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write exact target ledger mention count: 0; independent responsibility count: 0.
- Previous sequential source: readme/example_bw_true.svg@c7f73236f4569d8b25b9ebcc481f2372864adb45 (strict194).
- Next non-directory entry: readme/example_custom_symbols.svg@45888ac3f5cf19100b9c4096eb8d38bb29d0419f.

## Exact SVG provenance
The accepted CJK example is an SVG 1.1 document sized 86pt x 55pt with viewBox 0 0 86 55. It contains 8 symbol elements, 8 inline path elements and 8 use elements. The eight href/xlink:href references are #h, #g, #f, #e, #d, #c, #b and #a; all are local fragments. The exact blob contains zero script, image, style, text, font-face/font-family markers or external hrefs.

This establishes a passive inline vector/documentation boundary for this exact blob only. Any later active/external SVG content or blob drift requires a fresh audit.

## README direct consumer and CJK/TextLayout documentation context
README.md@824af04fd8607532efed3f73752b81e5093577ef directly embeds this target in the tex::TextLayout section. It documents the example `\\int_{now}^{\\infty} \\text{努力}`, identifies 努/力 as CJK Unified Ideographs in an unregistered CJK alphabet, states that such unregistered-alphabet characters use tex::TextLayout for layout/bounds, points to TextLayout_cairo as the Cairo demonstration, embeds readme/example_cjk_trying.svg, and then points to src/fonts/alphabet.cpp for predefined Unicode blocks.

README lists BASIC_LATIN, LATIN1_SUPPLEMENT, CYRILLIC, GREEK, GREEK_EXTENDED and UNKNOWN. It also documents how a new UnicodeBlock and AlphabetRegistration may be registered. The documentation visual therefore belongs to CJK/TextLayout/Unicode-block provenance, not to a new Fabushi CJK renderer or a second text stack.

## Runtime context-only boundary
The runtime sources are context only and remain independently open.

- src/platform/cairo/graphic_cairo.h@99ee55dd019d95e994ce201be31405510d57b035 declares TextLayout_cairo.
- src/platform/cairo/graphic_cairo.cpp@16331bfdb1d69926b7e14c7276acb3df052f40a8 implements TextLayout_cairo with a lazily created 1x1 Cairo ImageSurface/Context, Pango layout, wide2utf8 text conversion, font description/style/weight, baseline capture, pixel-size bounds, Pango/Cairo drawing, and TextLayout::create -> TextLayout_cairo for Font_cairo.
- src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891 defines the five named predefined blocks above, records them in UnicodeBlock::_defined, defines UNKNOWN as the complement of registered blocks through contains(), supports runtime define(start,end), and resolves UnicodeBlock::of() by the registered list before UNKNOWN.

These reads corroborate the README's documented context but do not make this SVG runtime implementation evidence. This row does not close the complete parser/atom caller chain, cross-platform text layout, Unicode fallback behavior, formula rendering, resource loading, or production equivalence.

## Exact-path history and generation provenance
Exact-path history through the accepted MicroTeX pin contains one change commit: 63555c2e902a42bfad457d3d58a4a24c5c51c53e, parent c85c84194835439319981a38485dc3fa4e8d4236, authored 2019-02-25, message "new version README". That commit added README.md and readme/example_cjk_trying.svg together with the breakEverywhere and other README examples.

README documents generic headless SVG conversion, including single-input/output operation, but no exact command, fixture, font selection, script, CI step, commit message or other repository evidence binds that facility to this exact target. Concrete generation provenance therefore remains unknown and is not inferred from the static paths or generic headless support.

## strict187 de-dup
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 regenerates default res/SAMPLES.tex through the headless demo, writes/compresses readme/samples/* plus readme/samples.md, stages only those paths, and contains zero example_cjk_trying/cjk_trying reference. This top-level README example is outside strict187's sample-regeneration responsibility.

## Build/package/shipping bypass
Direct target references are zero in pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7, res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa, ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838.

Direct target references are also zero in cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334, tdesktop Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 and Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277. Accepted Telegram Desktop composition continues through external_microtex / external_microtex_bundled.rcc, not this README asset.

## License/font/resource-derived boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README explicitly excludes fonts and XML resources under res from the blanket project MIT statement and points to their separate licenses. This SVG contains vector path outlines but no font identity metadata. Exact-source inspection cannot prove which font/resource produced every outline or collapse any resource-derived redistribution obligation into project MIT. Font/resource-derived provenance therefore remains independently open.

## Existing-owner-first disposition
This asset is source-only documentation visual provenance. Existing TDRP recursive documentation/source/license provenance governance remains the selected owner. TextLayout_cairo/alphabet/runtime formula owners and pinned external_microtex/td_iv shipping owners are context only. No CJKRuntime, DocumentationSvgRuntime, TextLayoutRuntime, second renderer, second resource root or source-specific product shell is introduced.

## Negative oracles
- Static CJK SVG is TextLayout runtime implementation evidence: false.
- README's example proves the complete parser/atom -> TextLayout caller chain: false.
- The predefined alphabet table registers CJK: false; CJK is the documented unregistered example.
- UNKNOWN means a fixed CJK block: false; it is the complement of currently defined blocks in alphabet.cpp.
- Generic headless SVG support proves this file's exact generator/font: false.
- strict187 pre-push generates this top-level asset: false.
- Checked MicroTeX/tdesktop build or shipping composition includes this asset: false.
- Project MIT settles all path-outline/font/resource derivation: false.
- This row closes TextLayout, formula rendering, baseline identity, source closure or release acceptance: false.

## Coverage
Before strict195: total 16,379; unknown 16,182; unread 16,157; omitted 0; responsibilities 212.
After this one independent registration: total 16,379; unknown 16,181; unread 16,156; omitted 0; responsibilities 213.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent f53f8fdb2ea352c7800469a843de6edf3c8adabc was revalidated before this receipt:
- TDRP Revision 9 source authority run 40124933718: completed/success, 61/61 jobs success.
  - authority 12572169942 sha256:e039fc9c89ea0832877369bcb28df3ee9703c34f70c7815086895afaf8311648
  - recursive closure 12573365317 sha256:47a4558058956870710705f2820bd026153f33a290d0d3ec6e649f8f0b58af5c
  - recursive reachability 12572906145 sha256:2913d6398cd5c6d6fbbb2248fa8e7b89b82fc65d331d75e3db619025bb1c50bc
  - Boost 12572776108 sha256:38aa6c11355786a08f0fdbc9b22294563022a425132fbdcf6c98037730fba16c
- Rust desktop runtime run 40124933702: completed/success, 6/6 jobs success.
  - artifact 12573050572 sha256:f338f1db20f8e5fbd5ddac6c9e882915e761ca459e5192336f7b1eab7cd3f14f
- Official MCP marketplace run 40124933662: completed/success, 1/1 job success.

These become historical parent evidence after the receipt. The receipt exact-head must be validated independently.

## Traceability
Requirement: TDRP-R9-MICROTEX-README-CJK-TRYING-SVG-PROVENANCE-001
Target symbol: TDRP_MICROTEX_README_CJK_TRYING_SVG_PROVENANCE_001
Oracles: exact source; local-only SVG references; README direct CJK/TextLayout consumer; TextLayout_cairo runtime-context boundary; Unicode-block/alphabet context; exact-path history/generator boundary; strict187 de-dup; build/package zero-consumer; shipping bypass; license/font/resource-derived boundary.
Invariants: documentation is not runtime authority; CJK example does not create a CJK runtime owner; target is not strict187 sample output; target is not a checked shipping resource; production responsibilities remain open.

## Machine-readable traceability anchors
- TDRP-R9-MICROTEX-README-CJK-TRYING-SVG-PROVENANCE-001
- ORA-TDRP-MICROTEX-CJK-TRYING-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-CJK-TRYING-SVG-INLINE-LOCAL-REFERENCE-001
- ORA-TDRP-MICROTEX-CJK-TRYING-README-CONSUMER-001
- ORA-TDRP-MICROTEX-CJK-TEXTLAYOUT-CAIRO-CONTEXT-001
- ORA-TDRP-MICROTEX-CJK-UNICODE-BLOCK-ALPHABET-CONTEXT-001
- ORA-TDRP-MICROTEX-CJK-TRYING-HISTORY-GENERATOR-BOUNDARY-001
- ORA-TDRP-MICROTEX-CJK-TRYING-STRICT187-DEDUP-001
- ORA-TDRP-MICROTEX-CJK-TRYING-BUILD-PACKAGE-CONSUMER-001
- ORA-TDRP-MICROTEX-CJK-TRYING-TDESKTOP-SHIPPING-BYPASS-001
- ORA-TDRP-MICROTEX-CJK-TRYING-LICENSE-FONT-RESOURCE-BOUNDARY-001
- INV-TDRP-MICROTEX-CJK-TRYING-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-CJK-TRYING-NO-CJK-RUNTIME-OWNER-001
- INV-TDRP-MICROTEX-CJK-TRYING-NOT-STRICT187-SAMPLE-OUTPUT-001
- INV-TDRP-MICROTEX-CJK-TRYING-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-CJK-TRYING-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-CJK-TRYING-README-CONTEXT-001
- CONTRACT-TDRP-MICROTEX-CJK-TRYING-RUNTIME-CONTEXT-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-CJK-TRYING-CONSUMER-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-CJK-TRYING-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-CJK-TRYING-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-CJK-TRYING-EXTERNAL-SVG-CONTENT-DRIFT-001
- FAULT-TDRP-MICROTEX-CJK-TRYING-GENERATOR-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-CJK-TRYING-RUNTIME-CONFLATION-001
- FAULT-TDRP-MICROTEX-CJK-TRYING-LICENSE-CONFLATION-001
- FAULT-TDRP-MICROTEX-CJK-TRYING-SHIPPING-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
