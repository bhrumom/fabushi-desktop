# desktop-app/MicroTeX readme/tex_logo.svg complete read — strict216

## Authority
- PR #154 branch: tdrp-r9-entry31-includes-r27-20261020.
- Lease / pre-receipt HEAD: 4d53d67c93955cd4d0934a0c21345818406b50b3; tree 0d109b39a4fffc874698c34e2e5a4655bb481f4e.
- Live main re-read before publication: 16aae12ea258f6a872a46aa47824766f64e1bdca; tree 82126ec4a712fca352a49d6a3caae70c7d21ca8c.
- Accepted/live tdesktop dev: a7f453612f4896ac04d0007a36ab0d377e5bcbcf; tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Pinned/live MicroTeX master: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f; tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- Fixed MicroTeX recursive tree is truncated=false with 315 entries.
- Independent pre-write responsibility row count for TDRP-R9-MICROTEX-README-TEX-LOGO-SVG-PROVENANCE-001 is 0. The only pre-write ledger mention of readme/tex_logo.svg is strict215's next pointer.
- Target: readme/tex_logo.svg@33647d95ba36e42839ac5a2469464c8cde2b6c99, mode 100644, size 1,298 bytes.
- Creation commit: 63555c2e902a42bfad457d3d58a4a24c5c51c53e. Creation and accepted target blob are identical.
- Previous independent entry: readme/samples/sample_11.svg@28039e570607bab47db7ff3976b16847c32fd768. Next unread resource marker: res/.clatexmath-res_root@e69de29bb2d1d6434b8b29ae775ad8c2e48c5391.

## Complete SVG read
The complete accepted target was read. It is 1,298 ASCII bytes with 20 split-line slots including the terminal LF, width 61pt, height 22pt and viewBox 0 0 61 22.

Exact structural inventory:
- 3 symbol elements
- 3 path elements
- 3 use elements
- 1 g element
- 3 id values: c, b, a; all unique
- 3 xlink:href references: #c, #b, #a; all local fragments and all resolved
- zero text, script, image, foreignObject, style, @font-face, external href, DOCTYPE or XML comments

The accepted pin resolves the same 33647d95 blob created at 63555c2. Future content or active/external-surface drift requires re-audit.

## Exhaustive exact-path history and co-change boundary
The accepted-pin exact-path query for readme/tex_logo.svg returns exactly one commit on page 1 and an empty page 2:
1. 63555c2e902a42bfad457d3d58a4a24c5c51c53e, "new version README", tree e26c0a1ea4487fead39497509c486c588a27c545, parent c85c84194835439319981a38485dc3fa4e8d4236; creates the target as 33647d95ba36e42839ac5a2469464c8cde2b6c99.

The creation parent recursive tree is truncated=false with 228 entries and has no readme/tex_logo.svg.

The creation commit changes exactly twelve paths:
- README.md
- readme/example_bw_false.svg
- readme/example_bw_true.svg
- readme/example_cjk_trying.svg
- readme/example_custom_symbols.svg
- readme/example_debug.svg
- readme/example_have_log.svg
- readme/example_keep_trying.svg
- readme/example_logos.svg
- readme/example_ubuntu.png
- readme/tex_logo.svg
- res/SAMPLES.tex

This is a README/documentation/source refresh co-change. It does not by itself assign the logo to the sample generator or product shipping.

## README same-commit binding
The creation parent README.md@ef95f41a3412363a90506f962e05d7d2c38927c7 begins with:
- # cLaTeXMath

The 63555c2 creation README.md@7f1a737577cdc31f2f4d4138b530e56441b8511e begins with:
- # ![logo](readme/tex_logo.svg)

The accepted README.md@824af04fd8607532efed3f73752b81e5093577ef retains the same first-line logo reference. Thus the asset is positively bound to README presentation from creation through the accepted pin.

## Generator negative oracle
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 contains zero tex_logo references. Its documented generation path runs LaTeX in headless mode for percent-separated res/SAMPLES.tex bodies, scour-normalizes indexed SVGs into readme/samples, stages readme/samples.md and readme/samples, and optionally commits those generated sample docs.

Therefore the tracked sample/headless/Cairo/scour chain is a negative oracle for readme/tex_logo.svg. No checked generator writes, stages, regenerates or byte-compares the logo.

## Build, CI and shipping negative oracles
Pinned MicroTeX:
- CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9: zero tex_logo refs.
- src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7: zero tex_logo refs.
- res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa: zero tex_logo refs.
- meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5: zero tex_logo/readme refs; install_data is res/RES_README, res/SAMPLES.tex and res/.clatexmath-res_root.
- .github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764: zero tex_logo refs.
- .github/workflows/windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa: zero tex_logo refs.
- .github/workflows/windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838: zero tex_logo refs.

Accepted tdesktop shipping is separate:
- Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex but has zero tex_logo refs.
- Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 carries external_microtex_bundled.rcc but has zero tex_logo refs.
- Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2 initializes bundled MicroTeX resources and renders formulas through the Qt backend but has zero tex_logo refs.
- Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 packages external_microtex_bundled.rcc but has zero tex_logo refs.

Thus accepted shipping is positive for the external_microtex Qt/bundled runtime and negative for this README logo.

## License and font-derivation boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README.md@824af04fd8607532efed3f73752b81e5093577ef says the project is MIT except fonts and XML resources under res, which have separate licenses.

The checked layered inputs remain:
- res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee
- Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8
- License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c
- OFL.txt@6836a0be000498319aebcc92a172159b6f80bb52
- res/greek/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7
- res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7

readme/tex_logo.svg itself contains only finalized vector paths and local uses. The repository license placement does not establish which font, if any, generated each outline. Therefore root-project licensing and potential font derivation are distinct questions; the layered res licenses cannot be flattened into an unsupported per-outline provenance claim.

## Existing-owner-first disposition
Selected owner remains the existing TDRP recursive source/dependency documentation/tooling/license provenance governance. Strict187 pre-push/sample generation is negative context only; formula/render/font and accepted external_microtex/td_iv owners remain independent and open.

No DocumentationLogoRuntime, MicroTeXDocsRuntime, second renderer/font owner, second build/CI/release gate, second shipping root, second design system, Telegram-specific runtime or Fabushi UI root is created.

## Coverage
Before strict216: total 16,379; unknown 16,161; unread 16,136; omitted 0; responsibilities 233.
After strict216: total 16,379; unknown 16,160; unread 16,135; omitted 0; responsibilities 234.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed. Independent release acceptance remains not accepted.

## Parent exact-head evidence
Lease 4d53d67c93955cd4d0934a0c21345818406b50b3:
- TDRP Revision 9 source authority run 40161934482 completed/success, 61/61 jobs. Authority artifact 12586104944 sha256:e7fe8c758e89397c319e65b28aefe9b2bf63f2711fa24d064122b6e80597a786; recursive closure 12587255417 sha256:31ce37604b3aecd87834820851562c1e82ff01db1e7a1bf04098fd258ee4bcfa; recursive reachability 12587110597 sha256:265e6962093a720def03dafeb6a0b4868ea28854f9e3c6b93da3b03a1126d17f; Boost reachability 12587230189 sha256:eda89d1883371c9caee37a69b1e84664f033607dd508eed8bb6574e0a5813723.
- Rust desktop runtime run 40161934455 completed/success, 6/6 jobs; artifact 12586970548 sha256:ef8e800552148c007c0fa295378b2c340e824bbc56ec1d13509feaa6e05a3d95.
- Official MCP marketplace run 40161934498 completed/success, 1/1 job.

Receipt-head evidence must be fresh.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-TEX-LOGO-SVG-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_TEX_LOGO_SVG_PROVENANCE_001
- Gates: G-FILE, G-TRACEABILITY, G-EVIDENCE.

## Machine-readable traceability anchors
- ORA-TDRP-MICROTEX-TEX-LOGO-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-TEX-LOGO-SVG-PASSIVE-LOCAL-CONTENT-001
- ORA-TDRP-MICROTEX-TEX-LOGO-EXACT-PATH-HISTORY-001
- ORA-TDRP-MICROTEX-TEX-LOGO-README-BINDING-001
- ORA-TDRP-MICROTEX-TEX-LOGO-GENERATOR-BYPASS-001
- ORA-TDRP-MICROTEX-TEX-LOGO-BUILD-CI-SHIPPING-BYPASS-001
- ORA-TDRP-MICROTEX-TEX-LOGO-LICENSE-FONT-BOUNDARY-001
- INV-TDRP-MICROTEX-TEX-LOGO-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-TEX-LOGO-NOT-SAMPLE-GENERATOR-OUTPUT-001
- INV-TDRP-MICROTEX-TEX-LOGO-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-TEX-LOGO-FONT-DERIVATION-NOT-INFERRED-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-TEX-LOGO-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-TEX-LOGO-README-BINDING-001
- CONTRACT-TDRP-MICROTEX-TEX-LOGO-GENERATOR-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-TEX-LOGO-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-TEX-LOGO-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-TEX-LOGO-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-TEX-LOGO-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-TEX-LOGO-README-REFERENCE-LOSS-001
- FAULT-TDRP-MICROTEX-TEX-LOGO-GENERATOR-CONFLATION-001
- FAULT-TDRP-MICROTEX-TEX-LOGO-SHIPPING-PROMOTION-001
- FAULT-TDRP-MICROTEX-TEX-LOGO-LICENSE-FONT-CONFLATION-001
