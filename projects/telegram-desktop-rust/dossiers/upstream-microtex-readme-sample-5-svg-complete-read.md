# desktop-app/MicroTeX readme/samples/sample_5.svg complete read — strict209

## Authority
- PR #154 branch: tdrp-r9-entry31-includes-r27-20261020.
- Lease / pre-receipt HEAD: 96c301df56e7cb7b6224a312ee22ff41a0921b60; tree f3b52feab97f491e12a99847dcad2da5c5e7e8da.
- Live main re-read before publication: 16aae12ea258f6a872a46aa47824766f64e1bdca; tree 82126ec4a712fca352a49d6a3caae70c7d21ca8c.
- Accepted/live tdesktop dev: a7f453612f4896ac04d0007a36ab0d377e5bcbcf; tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Pinned/live MicroTeX master: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f; tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- Fixed MicroTeX recursive tree is truncated=false with 315 entries.
- Independent pre-write responsibility row count for sample_5 is 0.
- Target: readme/samples/sample_5.svg@67e94f41adf1098fc2a083bdd13427ca2aae0c61, mode 100644.
- Previous independent entry: sample_4.svg@77d65ed8eabdd51063b6a54104f6d5c96d595a7d. Next: sample_6.svg@b007345c2b8ffa1d60199bff3dceb84f4d17de7e.

## Complete SVG read
The complete accepted target was read. It is 27,657 ASCII bytes with 248 split-line slots including the terminal LF, width 349pt, height 115pt and viewBox 0 0 349 115.

Exact structural inventory:
- 20 symbol elements
- 27 path elements
- 58 use elements
- 1 rect
- 58 g elements
- 20 id values, all unique
- 58 href/xlink:href references, all local fragments and all resolved
- zero text, script, image, foreignObject, style, @font-face, external href, DOCTYPE or XML comments

The accepted blob is passive, self-contained vector documentation content at this pin. Future blob drift requires re-audit.

## Stable index-5 source and Markdown relation
The sixth SAMPLES.tex body, sample index 5, is stable across every checked state. After removing the delimiter parser's trailing LF, it is exactly 208 characters at:
- 95a54b5 creation: res/SAMPLES.tex@7fea73994bec7453fc82f3cb91bab842112cd0ab and samples.md@bed457ee0da253f6ff4a37b0152be6eaaca9c20c.
- 63555c2 source-change state: res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b while samples.md remains bed457ee0da253f6ff4a37b0152be6eaaca9c20c.
- 08d9d6b generated-documentation update: SAMPLES.tex remains e03f0088e3fba1d225b90d06d586f32c5f70310b and samples.md becomes 4eb0be8a26e15f098156040366683f4e835631f6.
- accepted pin: the same e03f0088 SAMPLES.tex and 4eb0be8a samples.md.

At all four points the normalized index-5 SAMPLES.tex body exactly equals the corresponding samples.md fenced TeX body. The source is therefore stable even while other sample corpus state changes.

## Exhaustive exact-path and creation co-add history
The accepted-pin exact-path query for readme/samples/sample_5.svg returns exactly one commit on page 1 and an empty page 2:
1. 95a54b5a493335679f227ed42dbe29660309d9f1, "update samples", tree 31ac1b121c2af8a6f2167dc2cdcfa9b80ea72cdd, parent 105a81c58cac19ba3be2d66cd88ba05bf92ebcc1.

The parent tree is truncated=false with 192 entries and has no target. The creation commit changes exactly thirteen paths, all additions: readme/samples.md plus sample_0.svg through sample_11.svg. sample_5.svg is created as blob 67e94f41adf1098fc2a083bdd13427ca2aae0c61 with 247 added lines.

The accepted tree is truncated=false with 315 entries and still resolves sample_5.svg to mode 100644, size 27,657 and the exact same blob 67e94f41adf1098fc2a083bdd13427ca2aae0c61. No later exact-path mutation exists at the accepted pin. Creation=accepted byte identity is a Git history fact; it is not an execution receipt or deterministic-regeneration proof.

## Generator lineage
githooks/pre-push is b426a77d194ac023a93acb57235635608a0d9e04 at creation, 08d9d6b and the accepted pin. It runs LaTeX in headless mode with outputdir ./samples, prefix sample_, textsize 14, black foreground and white background; reconstructs samples.md from percent-separated res/SAMPLES.tex bodies; then scour-normalizes generated SVGs into readme/samples before staging and optional commit.

At creation, samples.h is 633cdcc8163f34914d80eb172e1a049f924b81a7, gtkmm_main.cpp is 3b7b1c8ac7e40c43e948afe9a332d3f9780a975e and graphic_cairo.cpp is 5d8c3b465e37a2fba9177088eff20278da214f69. At 08d9d6b, gtkmm_main.cpp is e0dc57832ea7d722e5d089ab8aa87b4f079962fa and graphic_cairo.cpp is ae1535053ade8b3919e32fa69ee1eef34a3af681. At the accepted pin, samples.h is 2e8bcd0d3d8e2461ad12bce4e9e4763124b6b652, gtkmm_main.cpp is d491af61d05fbcc5d17c48f64e76664f91d5fce8 and graphic_cairo.cpp is 16331bfdb1d69926b7e14c7276acb3df052f40a8.

The source-backed chain is SAMPLES index 5 -> LaTeX::parse -> Cairo::SvgSurface/Cairo::Context/Graphics2D_cairo -> indexed sample_5.svg -> scour -> tracked readme/samples asset. Generator implementation drift while the committed target remains unchanged reinforces the need to separate source lineage from an unobserved execution environment.

## Build, CI and shipping negative oracles
Pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7 and res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa have zero sample_5/readme-samples/SAMPLES target references.

meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 installs res/SAMPLES.tex as source/demo data but does not install readme/samples/sample_5.svg. Source corpus installation is distinct from generated documentation shipping.

The fixed child workflows ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838 contain zero sample_5/readme-samples/SAMPLES/scour target regeneration or byte-comparison path.

Accepted shipping is separate. desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 builds external_microtex and external_microtex_bundled with BUILD_QT and bundled resources; Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex; iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2 initializes bundled MicroTeX resources; Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 and build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 carry external_microtex_bundled.rcc. All checked shipping files have zero sample_5/readme-samples/SAMPLES references.

Thus accepted shipping is positive for the Qt/bundled MicroTeX runtime and negative for this generated documentation SVG.

## Reproducibility, font, resource and license boundary
Proven:
- exact accepted and creation target identity and byte equality;
- full passive/local SVG structure;
- stable normalized 208-character same-index source/Markdown body;
- single exact-path creation commit, parent absence and complete thirteen-path co-add;
- creation/update/current generator source lineage;
- build/CI/shipping negative oracles.

Not proven:
- that the Git pre-push hook was installed or dispatched for 95a54b5;
- exact historical LaTeX executable bytes, compiler, GTK/Cairo/Pango/Freetype/Fontconfig versions, OS image, locale or machine;
- exact external scour executable/version;
- raw pre-scour sample_5.svg bytes;
- exact font file/version used for each finalized outline;
- bit-for-bit regeneration in an unspecified environment;
- any GitHub Actions execution of the interactive developer hook.

Creation=accepted same-blob identity does not fill any of those gaps.

MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README.md@824af04fd8607532efed3f73752b81e5093577ef explicitly excludes fonts and XML resources under res from blanket MIT. res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee preserves the resource inventory; Knuth@5b03bf8a533e66c79d113b82ba95586f8c7af3b8, dsrom@35e3ff2160fe16cb26f7830f936557ebc44c3e1c, OFL@6836a0be000498319aebcc92a172159b6f80bb52 and Greek/Cyrillic@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7 remain layered license inputs. Final SVG paths do not prove per-outline font derivation.

## Existing-owner-first disposition
Selected owner remains the existing TDRP recursive source/dependency documentation/tooling/license provenance governance. Strict203 provides generated manifest context; strict187 provides tracked pre-push context; GTK/Cairo/formula/font and accepted external_microtex/td_iv owners remain independent and open.

No SampleSvgRuntime, MicroTeXDocsRuntime, second renderer/font owner, second build/CI/release gate, second shipping root, Telegram-specific runtime or Fabushi UI root is created.

## Coverage
Before strict209: total 16,379; unknown 16,168; unread 16,143; omitted 0; responsibilities 226.
After strict209: total 16,379; unknown 16,167; unread 16,142; omitted 0; responsibilities 227.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed. Independent release acceptance remains not accepted.

## Parent exact-head evidence
Lease 96c301df56e7cb7b6224a312ee22ff41a0921b60:
- TDRP Revision 9 source authority run 40149981806 completed/success, 61/61 jobs. Authority artifact 12581499218 sha256:85c84437672e77841eb19e9f9510fdd97868965a0406e49f42b99685b938cd8b; recursive closure 12581739279 sha256:9da9abdc93155c77ceff27a9eaf04d064d2fd6654487ca898608c9e267da5ca5; recursive reachability 12581719368 sha256:c43d5296563cbe2a23b220fd233f6f4328c503db0f48079232ce893c4453d834; Boost reachability 12582048961 sha256:d4915bf3ea5cf5588613b9d88540a799cca5bda46aa37e9e82e91cd4e611e0fb.
- Rust desktop runtime run 40149981855 completed/success, 6/6 jobs; artifact 12581967454 sha256:baa650b77464edc32563c776eca646e4ff53d1d5b1a399a1cba325e8184a767c.
- Official MCP marketplace run 40149981863 completed/success, 1/1 job.

Receipt-head evidence must be fresh.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-SAMPLE-5-SVG-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_SAMPLE_5_SVG_PROVENANCE_001
- Gates: G-FILE, G-TRACEABILITY, G-EVIDENCE.

## Machine-readable traceability anchors
- ORA-TDRP-MICROTEX-SAMPLE-5-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-SAMPLE-5-SVG-PASSIVE-LOCAL-CONTENT-001
- ORA-TDRP-MICROTEX-SAMPLE-5-STABLE-SOURCE-IDENTITY-001
- ORA-TDRP-MICROTEX-SAMPLE-5-EXACT-PATH-HISTORY-001
- ORA-TDRP-MICROTEX-SAMPLE-5-CREATION-ACCEPTED-IDENTITY-001
- ORA-TDRP-MICROTEX-SAMPLE-5-HISTORICAL-PREPUSH-001
- ORA-TDRP-MICROTEX-SAMPLE-5-HISTORICAL-HEADLESS-CAIRO-001
- ORA-TDRP-MICROTEX-SAMPLE-5-BUILD-CI-SHIPPING-BYPASS-001
- ORA-TDRP-MICROTEX-SAMPLE-5-REPRODUCIBILITY-BOUNDARY-001
- ORA-TDRP-MICROTEX-SAMPLE-5-LICENSE-FONT-BOUNDARY-001
- INV-TDRP-MICROTEX-SAMPLE-5-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-SAMPLE-5-GENERATOR-SOURCE-NOT-EXECUTION-PROOF-001
- INV-TDRP-MICROTEX-SAMPLE-5-SAME-BLOB-NOT-REPRODUCIBILITY-PROOF-001
- INV-TDRP-MICROTEX-SAMPLE-5-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-SAMPLE-SIBLINGS-REMAIN-INDEPENDENT-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-5-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-5-STABLE-SOURCE-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-5-GENERATOR-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-5-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-5-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-SAMPLE-5-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-SAMPLE-5-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-5-SAME-BLOB-REPRODUCIBILITY-CONFLATION-001
- FAULT-TDRP-MICROTEX-SAMPLE-5-ENVIRONMENT-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-5-SIBLING-PREMATURE-CLOSURE-001
- FAULT-TDRP-MICROTEX-SAMPLE-5-SHIPPING-PROMOTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-5-LICENSE-CONFLATION-001
