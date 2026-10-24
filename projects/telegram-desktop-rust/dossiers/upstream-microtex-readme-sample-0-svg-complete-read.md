# desktop-app/MicroTeX readme/samples/sample_0.svg complete read — strict204

## Authority
- PR #154 branch: tdrp-r9-entry31-includes-r27-20261020.
- Lease / pre-receipt HEAD: fdfbbc012945a292c3cdea07d302539affd160dc; tree 8ff6ca51f96aa84ddb992eda53f189f755d90827.
- Live main re-read immediately before publication: 16aae12ea258f6a872a46aa47824766f64e1bdca; tree 82126ec4a712fca352a49d6a3caae70c7d21ca8c.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Pinned MicroTeX: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2; live master re-read in strict204 remained exactly the pin.
- Pinned cmake_helpers: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Fixed MicroTeX recursive tree is truncated=false: 315 entries; target is mode 100644, 125,265 bytes.
- Target: readme/samples/sample_0.svg@9b71b0170ea06428a3f05f06766840c3bb66f866.
- Independent pre-write responsibility count is 0. Earlier rows mention the target only as sibling/dependency/reference context; strict203 explicitly leaves every sample_N.svg independent.
- Previous non-directory entry: readme/samples.md@4eb0be8a26e15f098156040366683f4e835631f6.
- Next non-directory entry: readme/samples/sample_1.svg@52dd4b61d61cfc5e96ec118a422de7fa20fe2092.

## Complete SVG read
The complete target was read. It is XML/SVG 1.1 with width 156pt, height 221pt and viewBox 0 0 156 221. It has 666 line slots and 125,265 bytes.

Exact structural inventory:
- 1 svg
- 108 symbol elements
- 109 path elements
- 110 use elements
- 1 rect
- 111 g elements
- 109 unique id values, all at most two characters
- 110 href/xlink:href references, all local # fragments and all resolving to in-document ids
- 0 text, script, image, foreignObject, style, @font-face, external href or DOCTYPE
- 0 XML comments
- the only explicit fill value is #fff

The current blob is therefore passive, self-contained vector documentation content. This observation is exact for the pinned blob and is not a permanent policy for future drift.

## Exact source block and Markdown relation
Current res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b was fully read under the tracked pre-push delimiter rule. It yields twelve nonempty samples. Current readme/samples.md@4eb0be8a26e15f098156040366683f4e835631f6 contains twelve fenced TeX bodies and twelve same-index image references.

The first current SAMPLES.tex segment is exactly equal to the first current samples.md fenced body, has 360 decoded characters, and maps to samples/sample_0.svg. The first segment begins with the sideset test and contains the complete first formula corpus; there is no filename-only inference.

Creation-time source identity is stronger: at commit 95a54b5a493335679f227ed42dbe29660309d9f1, res/SAMPLES.tex@7fea73994bec7453fc82f3cb91bab842112cd0ab and newly added samples.md@bed457ee0da253f6ff4a37b0152be6eaaca9c20c also yield twelve exact-equal source/Markdown bodies and twelve sequential image references. Their first 360-character body is byte-for-byte text-equal to the current first source/Markdown body. Thus sample_0's source text did not drift across the later corpus update.

## Exhaustive exact-path and co-change history
The accepted-pin exact-path query for readme/samples/sample_0.svg has exactly one result on page 1 and zero on page 2:
1. 95a54b5a493335679f227ed42dbe29660309d9f1, 2019-02-16, "update samples", tree 31ac1b121c2af8a6f2167dc2cdcfa9b80ea72cdd, parent 105a81c58cac19ba3be2d66cd88ba05bf92ebcc1.

The target is absent from that parent. The creation commit changes exactly thirteen paths, all added: readme/samples.md plus sample_0.svg through sample_11.svg. sample_0's creation blob is already the current blob 9b71b0170ea06428a3f05f06766840c3bb66f866.

The later 08d9d6b3ca060f4923ccf0117565430c865059e9 "update samples" commit changes samples.md and sample_2, sample_3, sample_4, sample_6, sample_7, sample_8 and sample_11; it does not change sample_0. Therefore the target has a single exact-path creation event and no later target mutation at the accepted pin.

Co-change and commit-message evidence is consistent with the repository's tracked sample generator. It is not a process log and cannot by itself prove that an installed hook produced that commit.

## Historical generator chain at the creation commit
At 95a54b5a493335679f227ed42dbe29660309d9f1:
- githooks/pre-push is b426a77d194ac023a93acb57235635608a0d9e04, exactly the same blob as the accepted pin.
- res/SAMPLES.tex is 7fea73994bec7453fc82f3cb91bab842112cd0ab.
- CMakeLists.txt is 6cb7a829aa7ef18bb33b3ffbbc87254c6350e05f and copies res into the build tree.
- src/samples/samples.h is 633cdcc8163f34914d80eb172e1a049f924b81a7 and defaults an omitted samples file to LaTeX::getResRootPath()/SAMPLES.tex.
- src/samples/gtkmm_main.cpp is 3b7b1c8ac7e40c43e948afe9a332d3f9780a975e.
- src/platform/cairo/graphic_cairo.cpp is 5d8c3b465e37a2fba9177088eff20278da214f69 and graphic_cairo.h is 5fe735b6014225adb47253935f64b0951a06fdc0.

The historical Headless path was read directly. It constructs Samples, iterates by index, parses each source body with LaTeX::parse, computes padded dimensions, creates Cairo::SvgSurface at dir/prefix/index.svg, creates Cairo::Context and Graphics2D_cairo, fills the requested background when non-transparent and draws the parsed result. runHeadless parses outputdir/samples/prefix/textsize/foreground/background and requires an output directory.

The unchanged pre-push blob invokes:
./LaTeX -headless -outputdir=./samples -prefix=sample_ -textsize=14 -foreground=black -background=white

It then applies external scour to every generated SVG with viewboxing, ID stripping, comment stripping, ID shortening and no indentation, copies the generated samples.md, stages the docs and may create an "update samples" commit.

This establishes a source-level creation-era SAMPLES.tex -> headless GTK/Cairo SVG -> scour -> tracked sample_N.svg chain. It does not establish an execution receipt.

## Current generator chain
At the accepted pin, CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9 still copies res; samples.h@2e8bcd0d3d8e2461ad12bce4e9e4763124b6b652 still defaults to SAMPLES.tex; gtkmm_main.cpp@d491af61d05fbcc5d17c48f64e76664f91d5fce8 still implements indexed headless generation through LaTeX::parse -> Cairo::SvgSurface -> Graphics2D_cairo; githooks/pre-push remains b426a77d194ac023a93acb57235635608a0d9e04 and still runs the same headless command plus scour.

The chain remains real and source-backed. Strict204 does not run it locally and does not treat source equivalence as bit-for-bit reproducibility proof.

## Build, CI and shipping negative oracles
Pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7 and res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa have zero sample_0.svg, samples.md or readme/samples packaging references.

meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 installs res/SAMPLES.tex and resource directories, but does not install readme/samples/sample_0.svg or generated samples.md. Source corpus installation is distinct from generated documentation assets.

The complete fixed MicroTeX workflow set is ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838. All three have zero sample_0/samples.md/SAMPLES/scour/headless-generation comparison path. Thus there is no pinned CI oracle that regenerates this committed SVG and compares bytes.

Accepted shipping uses desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 with BUILD_QT and res/bundled.qrc; Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex; iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2 initializes bundled resources and registers external_microtex_bundled.rcc; Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 and build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 carry that runtime resource path. None contains sample_0.svg, generated samples.md or SAMPLES.tex shipping references.

Accepted shipping is therefore positive for the Qt/bundled MicroTeX runtime and negative for this generated documentation SVG.

## Reproducibility and provability boundary
Proven:
- exact target identity, full content and passive/local SVG structure;
- a single exact-path creation commit, parent absence and complete thirteen-path co-change set;
- exact creation-time and current first SAMPLES.tex -> samples.md body identity;
- creation-time pre-push is the same exact blob as current;
- creation-time headless GTK/Cairo source writes indexed SVGs;
- current headless/scour chain remains source-backed;
- build, workflow and accepted shipping negative oracles.

Not proven:
- that the Git pre-push hook was installed or dispatched for commit 95a54b5;
- the exact historical LaTeX executable bytes, compiler, GTK/Cairo versions, OS image, locale or machine;
- the exact external scour executable/version used for the committed file;
- the raw pre-scour sample_0.svg bytes;
- exact font files/versions selected for every glyph outline in that historical run;
- bit-for-bit regeneration in an unspecified environment;
- any GitHub Actions execution of this interactive developer hook.

The lack of an execution receipt is retained as an explicit boundary rather than filled by inference.

## Font, resource and license boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README.md@824af04fd8607532efed3f73752b81e5093577ef explicitly excludes fonts and XML under res from its blanket MIT statement and points to separate licenses. res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee enumerates those font/resource inputs and notices.

sample_0.svg contains path outlines and no font identity metadata. Its final bytes therefore cannot establish which specific resource font produced each outline or flatten the layered resource licenses into a blanket source-code MIT conclusion. Strict204 preserves the generated-documentation provenance plus the unresolved per-outline resource derivation boundary. No target SVG is copied into Fabushi shipping.

## Existing-owner-first disposition
Existing TDRP recursive source/dependency documentation/tooling/license provenance governance remains selected. Strict203 owns the generated Markdown manifest context; strict187 owns the tracked pre-push context; GTK/Cairo/formula rendering and accepted external_microtex/td_iv runtime owners remain independently open.

No SampleSvgRuntime, MicroTeXDocsRuntime, second formula renderer, second CI/release gate, second shipping root, Telegram-specific runtime or Fabushi UI root is created.

## Coverage
Before strict204: total 16,379; unknown 16,173; unread 16,148; omitted 0; responsibilities 221.
After strict204: total 16,379; unknown 16,172; unread 16,147; omitted 0; responsibilities 222.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed. Independent release acceptance remains not accepted.

## Parent exact-head evidence
Parent fdfbbc012945a292c3cdea07d302539affd160dc:
- TDRP Revision 9 source authority run 40141860757 completed/success, 61/61 jobs. Authority artifact 12578474553 sha256:29707c93d9f74b06b6ae90a7f343a0132514ef9d15b3f2d86ae0ad7b69d5c019; recursive closure 12579595177 sha256:00febf0ca7d72ad7fe3f7259ec02b3c9f6e08a44f792b50ce35af4d061ab8c7b; recursive reachability 12579510373 sha256:005320800e8af5eeb3b587f3f2104fa13d324983f0c04ea7e8c7516bb8a7a983; Boost reachability 12579386977 sha256:ebee5c066499961a3fa9b3d868b15cce0e501c46da0c5fdee1b5e5be75221ae4.
- Rust desktop runtime run 40141860692 completed/success, 6/6 jobs; artifact 12578858892 sha256:34896155168a28be5a927aaf3882296bade60a2e9ddaea63b0a93bf535829ee5.
- Official MCP marketplace run 40141860655 completed/success, 1/1 job.

Receipt-head evidence must be fresh.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-SAMPLE-0-SVG-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_SAMPLE_0_SVG_PROVENANCE_001
- Gates: G-FILE, G-TRACEABILITY, G-EVIDENCE.

## Machine-readable traceability anchors
- ORA-TDRP-MICROTEX-SAMPLE-0-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-SAMPLE-0-SVG-PASSIVE-LOCAL-CONTENT-001
- ORA-TDRP-MICROTEX-SAMPLE-0-SOURCE-BLOCK-IDENTITY-001
- ORA-TDRP-MICROTEX-SAMPLE-0-CREATION-COCHANGE-001
- ORA-TDRP-MICROTEX-SAMPLE-0-HISTORICAL-PREPUSH-001
- ORA-TDRP-MICROTEX-SAMPLE-0-HISTORICAL-HEADLESS-CAIRO-001
- ORA-TDRP-MICROTEX-SAMPLE-0-BUILD-CI-SHIPPING-BYPASS-001
- ORA-TDRP-MICROTEX-SAMPLE-0-REPRODUCIBILITY-BOUNDARY-001
- ORA-TDRP-MICROTEX-SAMPLE-0-LICENSE-FONT-BOUNDARY-001
- INV-TDRP-MICROTEX-SAMPLE-0-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-SAMPLE-0-GENERATOR-SOURCE-NOT-EXECUTION-PROOF-001
- INV-TDRP-MICROTEX-SAMPLE-0-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-SAMPLE-SIBLINGS-REMAIN-INDEPENDENT-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-0-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-0-GENERATOR-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-0-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-0-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-SAMPLE-0-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-SAMPLE-0-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-0-ENVIRONMENT-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-0-SIBLING-PREMATURE-CLOSURE-001
- FAULT-TDRP-MICROTEX-SAMPLE-0-SHIPPING-PROMOTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-0-LICENSE-CONFLATION-001
