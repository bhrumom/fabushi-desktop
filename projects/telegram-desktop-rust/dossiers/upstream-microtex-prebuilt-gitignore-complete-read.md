# desktop-app/MicroTeX prebuilt/.gitignore complete read — strict191

## Authority

- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: 27b963fc198f3b69a8ac545f7b1c12654056b69c
- Pre-receipt tree: 1e1a6386684486e8bcfb85faaa54a2dd6bf907f5
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop pin remains telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf (tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5).
- Direct child pin remains desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f (tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2).
- CMake helper pin remains desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095 (tree f5c386cb34208309aae521187611fac4a2045577).
- Target: prebuilt/.gitignore@777ef934cd0b38e8a1c21cec325f77c53dba7cd3, mode 100644, 5 Git-tree bytes, exact content `*.clm` with no trailing newline.
- Pinned MicroTeX recursive tree is truncated=false with 315 entries / 279 blobs.
- Tree order around the target is meson.build, meson_options.txt, prebuilt/ directory, prebuilt/.gitignore, prebuilt/otf_clm.py.
- Pre-write independent path/SHA and responsibility registration count for this target: 0.

This dossier closes only the generated-prebuilt ignore/provenance responsibility. It does not close the independently next source entry prebuilt/otf_clm.py, formula rendering, resource loading, packaging, or release acceptance.

## Exact source meaning

The target has one rule only:

- `*.clm`

Within the `prebuilt/` directory this makes generated files whose basename ends in `.clm` ordinary ignored untracked worktree outputs unless they are explicitly tracked by Git.

The exact pinned recursive tree contains zero paths ending in `.clm`. Therefore no generated CLM binary is part of the accepted pinned MicroTeX Git object graph at this commit.

The rule does not create a file, choose an input font, establish a build target, install/package anything, or make a runtime load decision.

## Generator context: prebuilt/otf_clm.py

The immediately following source entry was fully read as direct context only and remains independently unregistered for source accounting:

- prebuilt/otf_clm.py@aba6e00be5221f7ee18d4ae71f8acbf8c92cf17b
- mode 100755
- 14,267 bytes
- 507 physical lines
- Python 2 shebang
- imports FontForge and struct
- source comment: parse an OpenType font and convert it to CLM format

Its direct command contract is positional:

1. `sys.argv[1]` — input OpenType font path
2. `sys.argv[2] == 'true'` — whether to serialize math-font data
3. `sys.argv[3]` — output file path

The script opens the source with `fontforge.open(file_path)`, derives Unicode/glyph mapping, kerning classes, ligatures, metrics and optional OpenType MATH data, then opens the caller-supplied output path in binary write mode. The serialized stream begins with the math-font boolean and em/x-height, followed by the generated mapping/kerning/ligature data, optional math constants, and glyph records.

The generator does not enforce a `.clm` output suffix; the ignore rule and generator are therefore coupled only by convention and directory intent, not by a filename assertion in the script.

Negative generator boundaries observed from the exact source:

- no argv-count/help validation before direct positional indexing;
- no exception handling around FontForge input or output creation/write;
- no atomic temp-file + rename transaction;
- no magic/version/checksum envelope is written by this script;
- the output path is supplied by the caller;
- the script itself is tooling context, not a shipping runtime owner.

These observations are provenance/negative oracles, not a request to execute the Python 2 tool.

## Build/package consumer audit

Exact pinned candidate build/package/CI consumers were re-read for direct `.clm`, `prebuilt/` or `otf_clm.py` references:

- MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9 — zero direct references.
- MicroTeX meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 — zero direct references.
- MicroTeX src/meson.build@84ce4f05fceb8419a326bf6a059d2583c793fc9a — zero direct references.
- MicroTeX src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7 — zero direct references.
- ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764 — zero direct references.
- windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa — zero direct references.
- windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838 — zero direct references.

README.md@824af04fd8607532efed3f73752b81e5093577ef has a heading “Prebuilt packages”, but that section documents externally maintained openSUSE Build Service binaries for cLaTeXMath/cairo. It does not reference the tracked `prebuilt/` directory, `.clm`, or `otf_clm.py`, and is not evidence that CLM outputs are packaged.

Accordingly, this target is a VCS/generated-output exclusion record. No pinned build/package producer consumes the ignored CLM output in the checked candidate composition.

## Accepted tdesktop shipping boundary

The accepted tdesktop shipping chain was exact-ref re-read:

- cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 directly builds the pinned MicroTeX C++ sources, defines BUILD_QT, links Qt, and attaches res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa through target_prepare_qrc.
- that external CMake file has no direct `.clm`, `prebuilt/` or `otf_clm.py` reference;
- bundled.qrc has no CLM/prebuilt/tool reference;
- Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex and has no CLM/prebuilt/tool reference;
- Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 packages the external_microtex_bundled RCC on macOS and has no CLM/prebuilt/tool reference.

Therefore ignored `prebuilt/*.clm` outputs are not part of the accepted tdesktop shipping composition evidenced by these exact refs. The existing external_microtex/resource/formula owners remain separately open.

## License and provenance boundary

- prebuilt/.gitignore and prebuilt/otf_clm.py are outside `res/` and fall under the MicroTeX project root MIT license, LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2.
- README explicitly excludes fonts and XML resources under `res/` from the blanket project MIT statement.
- the pinned tree preserves font-specific notices under res/fonts/licences and separate Greek/Cyrillic license texts.
- an ignored generated CLM binary is derived from the caller-supplied font. The ignore rule does not change the input font's license/provenance obligations.
- there is no tracked `.clm` blob at the accepted pin, so this row records no binary redistribution/license conclusion for a generated CLM artifact.

This is provenance, not legal advice.

## Negative oracles

- `*.clm` means a CLM file is part of the accepted source tree. False: the pinned tree has zero tracked `.clm` paths.
- The ignore rule generates CLM output. False: it only affects untracked Git discovery.
- otf_clm.py automatically writes into `prebuilt/` or enforces a `.clm` suffix. False: the output path is positional caller input.
- MicroTeX CMake/Meson/Qt build composition invokes otf_clm.py. Not established; the exact candidate manifests checked above contain zero direct reference.
- README “Prebuilt packages” proves CLM packaging. False: it describes external distro packages of cLaTeXMath/cairo.
- Accepted tdesktop shipping consumes `prebuilt/*.clm`. False for the exact external_microtex/bundled.qrc/td_iv composition checked here.
- Ignoring a generated file makes its license irrelevant. False: generated data remains subject to its input provenance/license context.
- This source read closes formula rendering, resource loading, packaging or release acceptance. False: those remain independent and open.

## Coverage accounting

Before strict191:
- recursive source entries: 16,379
- unknown: 16,186
- unread: 16,161
- omitted: 0
- responsibilities: 208

After independently registering this target only:
- recursive source entries: 16,379
- unknown: 16,185
- unread: 16,160
- omitted: 0
- responsibilities: 209

prebuilt/otf_clm.py remains the next independently unregistered source entry even though it was fully read here as direct context. baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence

The pre-receipt parent 27b963fc198f3b69a8ac545f7b1c12654056b69c was revalidated before this receipt:

- TDRP Revision 9 source authority run 40117495406: completed/success; recorded 61/61 jobs success.
  - authority 12569357086 — sha256:e47841af0d905a3806b3275db0c7d0351e134780546d472081fcfd0c44c37f65
  - closure 12570490852 — sha256:4c6d2821017ea96992e4e143d5d104f4fe110c6a0056706c8fb4610317602a25
  - reachability 12569907445 — sha256:99f5fddecb25d9bc04de48ab940f6ffe235e2bf320ab23c8951264816381a9f4
  - Boost 12570297346 — sha256:5820ba8ad9d29268854939d9b11c53470862ab73ca9fd1e845f1119a840c8702
- Rust desktop runtime run 40117495408: completed/success; 6/6 jobs success.
  - artifact 12569561425 — sha256:335955b85da2693dc1287be21c6f04bd5114ec5c82046e3f56f32a00505c84d7
- Official MCP marketplace run 40117495475: completed/success; 1/1 job success.

These become historical parent evidence after this receipt commit. The new receipt exact-head must be independently revalidated before current-head success is reported.

## Traceability

- Requirement: TDRP-R9-MICROTEX-PREBUILT-GITIGNORE-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_PREBUILT_GITIGNORE_PROVENANCE_001
- Exact-source oracle: ORA-TDRP-MICROTEX-PREBUILT-GITIGNORE-EXACT-SOURCE-001
- Tree-zero oracle: ORA-TDRP-MICROTEX-PREBUILT-CLM-TRACKED-ZERO-001
- Generator-context oracle: ORA-TDRP-MICROTEX-PREBUILT-OTF-CLM-CONTEXT-001
- Build/package-consumer oracle: ORA-TDRP-MICROTEX-PREBUILT-CLM-BUILD-PACKAGE-CONSUMER-001
- Shipping-bypass oracle: ORA-TDRP-MICROTEX-PREBUILT-CLM-TDESKTOP-SHIPPING-BYPASS-001
- License-boundary oracle: ORA-TDRP-MICROTEX-PREBUILT-CLM-LICENSE-BOUNDARY-001
- Invariant: INV-TDRP-MICROTEX-PREBUILT-IGNORE-VCS-ONLY-001
- Invariant: INV-TDRP-MICROTEX-PREBUILT-CLM-NOT-SHIPPING-ROOT-001
- Invariant: INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001

Next pinned recursive-tree non-directory entry: prebuilt/otf_clm.py@aba6e00be5221f7ee18d4ae71f8acbf8c92cf17b.
