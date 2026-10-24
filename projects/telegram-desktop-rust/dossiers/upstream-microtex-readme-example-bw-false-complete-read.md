# desktop-app/MicroTeX readme/example_bw_false.svg complete read — strict193

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: 63f3d8f40603b2b066e710399a921e384767520a
- Pre-receipt tree: 930eee6b9370af7a0b5fcaa6bee003785f913c84
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Target: readme/example_bw_false.svg@9e812c823d3c3be06ab43ee77f10b6122bd638dd, mode 100644, 57,285 bytes, 681 physical lines.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write ledger target mention count: 0.
- Next non-directory entry: readme/example_bw_true.svg@c7f73236f4569d8b25b9ebcc481f2372864adb45.

## Exact SVG provenance
The accepted SVG is 499pt x 71pt with viewBox 0 0 499 71. It contains 44 symbol elements, 44 path elements and 180 use instances. All 180 href/xlink:href values are local fragment references. There are no script, image, style, text, font-face/font-family markers or external hrefs.

This exact structure establishes a passive inline vector/documentation boundary for this blob only. Any later active or external SVG content requires a new audit.

## README consumer
README.md@824af04fd8607532efed3f73752b81e5093577ef documents \\breakEverywhere. It directly embeds readme/example_bw_false.svg for \\breakEverywhere{false} and uses the separate readme/example_bw_true.svg@c7f73236f4569d8b25b9ebcc481f2372864adb45 for true.

## Context-only product semantics
The image does not own runtime behavior. macro_def.cpp@0ccf9fe8b4028f97c62834555d2a3d344d0f8726 registers breakEverywhere. macro_impl.h@0e6f6712dc079d4f0283ba71d659e22819dc8b6f assigns RowAtom::_breakEveywhere = args[1] == L"true"; every non-exact-lowercase-true value becomes false. atom_row.cpp@fab326690dc46e2200b52be6c4b78b02643981ee initializes false. When a row is breakable, true adds a break position after every emitted child; false still permits positions after BreakMark or ASCII-digit cases. _breakable=false suppresses both branches.

These semantics are recorded only to define the documentation context. Formula/layout production closure remains independently open.

## Generation/update provenance
Exact-path history through the accepted pin has one change commit: 63555c2e902a42bfad457d3d58a4a24c5c51c53e, parent c85c84194835439319981a38485dc3fa4e8d4236, authored 2019-02-25, message "new version README". That commit added README.md plus example_bw_false.svg and example_bw_true.svg together.

README documents a generic -headless SVG conversion capability, but no exact command, fixture, script, CI step, commit message or other repository evidence binds that facility to this file. Concrete generator provenance is therefore unknown and is not inferred.

## strict187 de-dup
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 regenerates samples from res/SAMPLES.tex, compresses them with scour and writes only readme/samples/* plus readme/samples.md. It stages only those paths and has zero example_bw_false references. This top-level README asset is therefore outside the strict187 sample-regeneration responsibility.

## Build/package/shipping bypass
Direct target references are zero in pinned MicroTeX CMakeLists.txt, meson.build, src/QtLatex.pri, ubuntu-gtk.yml, windows-gdi.yml and windows-qt.yml. Direct target references are also zero in cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334, tdesktop Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 and Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277. Accepted shipping continues through external_microtex and bundled.qrc rather than this README asset.

## License/font-derived boundary
README states that project code/docs are MIT excluding fonts and XML resources under res, which retain separate licenses. The SVG stores path outlines but no font identity metadata. Exact source inspection cannot prove which font/resource generated each outline or relicense resource-derived shapes. Root MIT therefore does not close font/resource-derived redistribution provenance.

## Negative oracles
- Static SVG is runtime line-break evidence: false.
- false and true share one asset: false.
- non-true argument preserves prior flag: false; it assigns false.
- false means no break positions: false; BreakMark/ASCII digit can still add positions.
- true bypasses _breakable=false: false.
- strict187 pre-push generates this top-level asset: false.
- README headless docs prove this file's exact generator: false.
- checked build/package/shipping composes this asset: false.
- root MIT settles all outline/font derivation: false.
- this row closes formula/layout production, baseline identity, source closure or release acceptance: false.

## Coverage
Before strict193: total 16,379; unknown 16,184; unread 16,159; omitted 0; responsibilities 210.
After this one independent registration: total 16,379; unknown 16,183; unread 16,158; omitted 0; responsibilities 211.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent 63f3d8f40603b2b066e710399a921e384767520a was revalidated before the receipt:
- TDRP Revision 9 run 40121419493: completed/success, 61/61.
  - authority 12571731446 sha256:5b8f864dc0552a86fb1cd7b931cf013c5b3681f8dd8da43d2e1b97b168c4f8a2
  - closure 12571852289 sha256:3f77052a23a5ed1c3ccf7d63b15c97d2833cf462527dc9026e75c887164e8c1a
  - reachability 12571726630 sha256:a34be0951fc19a889f6717ddb8d996755d3ab3ed6e6b0054816bf3d094815601
  - Boost 12571213732 sha256:f50e0e7691a12f8954ff359795e56c81374ebb3a61c6bffe6a4ffa779d0383bc
- Rust desktop runtime run 40121419512: completed/success, 6/6.
  - artifact 12571288117 sha256:bf0fc830a6a15f824c57a7be3b8bfb3cd8f66d2e965f151a442eee59d8df8792
- Official MCP marketplace run 40121419510: completed/success, 1/1.

These become historical parent evidence after the receipt. The receipt exact-head must be revalidated independently.

## Traceability
Requirement: TDRP-R9-MICROTEX-README-BREAK-EVERYWHERE-FALSE-SVG-PROVENANCE-001
Target symbol: TDRP_MICROTEX_README_BREAK_EVERYWHERE_FALSE_SVG_PROVENANCE_001
Oracles: exact source; local SVG references; README consumer; runtime-context boundary; history/generator boundary; strict187 de-dup; build/package consumer zero; shipping bypass; license/font-derived boundary.
Invariants: documentation is not runtime authority; target is not strict187 sample output; target is not a checked shipping resource; production responsibilities remain open.
