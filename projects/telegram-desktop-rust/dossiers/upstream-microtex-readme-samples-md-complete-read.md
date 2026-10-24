# desktop-app/MicroTeX readme/samples.md complete read — strict203

## Authority
- PR #154 branch: tdrp-r9-entry31-includes-r27-20261020.
- Lease / pre-receipt HEAD: 257b121e839c339562afce90a698b551a07999e5; tree de9d7f11d4a296b6084d233781fc40b71ff12771.
- Live main re-read: 16aae12ea258f6a872a46aa47824766f64e1bdca; tree 82126ec4a712fca352a49d6a3caae70c7d21ca8c.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin/master: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- Target: readme/samples.md@4eb0be8a26e15f098156040366683f4e835631f6, mode 100644, 9,390 bytes, 302 line slots.
- Fixed MicroTeX tree is truncated=false: 315 entries / 279 blobs.
- Independent pre-write responsibility count for the target is 0. Its sole previous ledger occurrence is a dependency-context edge in strict187 githooks/pre-push.
- Previous non-directory entry: readme/example_windows.png@1d5682ae38d994aa0b4ebb1ac31d7a38b8f8581c.
- Next non-directory entry: readme/samples/sample_0.svg@9b71b0170ea06428a3f05f06766840c3bb66f866.

## Complete line read and exact document structure
The complete target was read. It contains # SAMPLES, exactly twelve fenced TeX bodies, and exactly twelve local image references. Reference N points to samples/sample_N.svg for N=0..11.

README.md@824af04fd8607532efed3f73752b81e5093577ef links readme/samples.md as the additional demo collection and separately documents Linux headless operation, -samples, -outputdir and the default res/SAMPLES.tex source.

## Exact source-to-Markdown relation
res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b was fully read. githooks/pre-push treats a line consisting only of one or more percent signs as a delimiter, ignores blank lines while accumulating a sample, and emits each accumulated body through write_sample.

Applying that exact rule to the pinned SAMPLES.tex yields twelve nonempty segments. All twelve are text-equal to the corresponding fenced bodies in samples.md. Their decoded character lengths are 360, 929, 1562, 2259, 433, 208, 227, 595, 283, 1080, 250 and 217. This is an exact pinned source-to-generated-document relation, not a filename inference.

## Real generation chain
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 was fully read. On Y/y it enters build/, recreates build-local samples/, invokes:

./LaTeX -headless -outputdir=./samples -prefix=sample_ -textsize=14 -foreground=black -background=white

It then recreates build-local samples.md from res/SAMPLES.tex, runs scour over each generated SVG with viewboxing, ID/comment stripping, ID shortening and no indentation, writes compressed SVGs into ../readme/samples, copies samples.md into ../readme, stages those docs and may create an "update samples" commit.

CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9 copies res into the build directory and BUILD_GTK builds src/samples/gtkmm_main.cpp as output LaTeX. samples.h@2e8bcd0d3d8e2461ad12bce4e9e4763124b6b652 defaults an empty sample path to LaTeX::getResRootPath()/SAMPLES.tex. gtkmm_main.cpp@d491af61d05fbcc5d17c48f64e76664f91d5fce8 Headless::runBatch iterates Samples and writes prefix+index+.svg; generateSingle parses LaTeX, creates Cairo::SvgSurface and draws via Graphics2D_cairo.

The repository therefore contains a real source/generator chain. It does not prove that a particular historical commit was produced by an installed hook invocation, nor the exact historical LaTeX binary, compiler, GTK/Cairo/scour versions, OS image, locale, font versions or machine.

## Current sample SVG siblings — context only
The pinned readme/samples directory contains exactly:
- sample_0.svg@9b71b0170ea06428a3f05f06766840c3bb66f866
- sample_1.svg@52dd4b61d61cfc5e96ec118a422de7fa20fe2092
- sample_2.svg@64ce25f438a4d4d88ff7b9d8947e0b3dd74257b0
- sample_3.svg@3937fe165e4c90414c4b9e18ae5b32c27784c7e6
- sample_4.svg@77d65ed8eabdd51063b6a54104f6d5c96d595a7d
- sample_5.svg@67e94f41adf1098fc2a083bdd13427ca2aae0c61
- sample_6.svg@b007345c2b8ffa1d60199bff3dceb84f4d17de7e
- sample_7.svg@6fe53b06b0df9d4433650c9deff13804c74a57d7
- sample_8.svg@2a3aceabd646957e4f47900551b60204b19b4794
- sample_9.svg@fc19f5fa1c500f2b1659eaaee05352ae0024990f
- sample_10.svg@31a02661b1fccb964dc9c0b8a727b521a85f42ea
- sample_11.svg@28039e570607bab47db7ff3976b16847c32fd768

All twelve were inspected only as strict203 context. Each is a passive SVG using path/symbol/use content plus a background rect. None contains text, script, image, foreignObject, external hrefs, font-family or @font-face. They remain twelve independent source entries and strict203 does not consume their coverage.

## Complete exact-path history
The exact-path commit query at the accepted pin has two results on page 1 and zero on page 2:
1. 95a54b5a493335679f227ed42dbe29660309d9f1, 2019-02-16, "update samples", parent 105a81c58cac19ba3be2d66cd88ba05bf92ebcc1: adds readme/samples.md@bed457ee0da253f6ff4a37b0152be6eaaca9c20c and all twelve sample_*.svg files. The target is absent in its parent.
2. 08d9d6b3ca060f4923ccf0117565430c865059e9, 2019-02-25, "update samples", parent 63555c2e902a42bfad457d3d58a4a24c5c51c53e: changes samples.md to current blob 4eb0be8a26e15f098156040366683f4e835631f6 by adding fatalIfCmdConflict{false}; it also updates sample_2, sample_3, sample_4, sample_6, sample_7, sample_8 and sample_11.

The shared "update samples" message and co-change set are consistent with the tracked generator workflow, but are not execution logs and do not prove exact historical tooling.

## Build, CI and shipping negative oracles
Pinned CMakeLists.txt, src/QtLatex.pri and res/bundled.qrc have zero readme/samples.md, readme/samples/sample_N or generated-document packaging references. Root CMake can build sample executables but does not package these generated docs.

meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 installs res/SAMPLES.tex. It does not install samples.md or readme/samples/sample_N.svg. Source corpus installation and generated documentation are distinct.

ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838 have no target/generated-doc generation or comparison path.

desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 compiles accepted BUILD_QT MicroTeX and packages res/bundled.qrc, but has zero samples.md/sample_N/SAMPLES.tex references. Accepted tdesktop td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30, Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277, iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2 and build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 likewise have zero generated-doc/SAMPLES.tex shipping references.

Thus accepted shipping is positive for external_microtex Qt runtime resources and negative for samples.md, the twelve sample_N SVG docs and SAMPLES.tex.

## Provability boundary
Proven: target identity and complete content; twelve source bodies; twelve same-index SVG links; exact pinned SAMPLES.tex body equality; tracked hook/headless-Cairo/scour chain; complete two-commit exact-path history; current twelve sibling identities/passive structure; build/CI/shipping bypass.

Unproven: universal hook installation/dispatch; exact historical hook process; exact historical binary/toolchain/OS/font environment; bit-for-bit deterministic SVG regeneration under an unspecified environment; GitHub Actions execution of this local interactive hook; accepted shipping inclusion of these docs.

## License boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README explicitly excludes fonts and XML under res from its blanket MIT statement and points to separate resource licenses; res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee preserves that inventory.

samples.md is generated documentation outside res. The current SVGs contain path outlines rather than font references, so their exact glyph/font derivation is not self-describing and must not be used to flatten layered font/resource provenance. No generated documentation asset is copied into Fabushi shipping.

## Existing-owner-first disposition
Existing TDRP recursive source/dependency documentation/tooling/license provenance governance remains selected. Strict187 is generator-hook context; GTK/Cairo sample/render owners and external_microtex/td_iv shipping owners remain separate. No SampleManifestRuntime, MicroTeXDocsRuntime, second renderer, second CI/release gate, second shipping root or Fabushi UI root is created.

## Coverage
Before strict203: total 16,379; unknown 16,174; unread 16,149; omitted 0; responsibilities 220.
After strict203: total 16,379; unknown 16,173; unread 16,148; omitted 0; responsibilities 221.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent 257b121e839c339562afce90a698b551a07999e5:
- TDRP Revision 9 source authority 40139128045 completed/success, 61/61; authority 12578235230 sha256:820f0621164de43c18ab53e2b8e3080cf75d4155f57a478a10c23ffcab61ee26; recursive closure 12578700146 sha256:5d66b96dcc4f97484d0996cc8ba1777b885bbaa2d404dccaa0163138c5d5df40; recursive reachability 12578655273 sha256:1d275bc9a08ff77596839efca862a7d5eb44385f294057be3eedd6f280eb26cf; Boost 12578295423 sha256:b7e9c80931b9ebd19c9955f7f4ed9085d7fa6f04577bb421cf565d1f07ac7141.
- Rust desktop runtime 40139128078 final completed/success, 6/6; artifact 12578735781 sha256:3cd2101b1a7361133b663474f66c037791b70ec12ac0e6f1cf4776c81da89c55.
- Official MCP marketplace 40139128083 completed/success, 1/1.

Receipt-head evidence must be fresh.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-SAMPLES-MD-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_SAMPLES_MD_PROVENANCE_001
- Gates: G-FILE, G-TRACEABILITY, G-EVIDENCE.
