# desktop-app/MicroTeX meson_options.txt complete read — strict190

## Authority

- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: 59d84e726b285a40020e473a72599710994a75b2
- Pre-receipt tree: d97ad6ff38f186704d7e7cd33ed0a81cc8dbf129
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca (tree 82126ec4a712fca352a49d6a3caae70c7d21ca8c).
- Accepted tdesktop pin remains telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf (tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5).
- Direct child pin remains desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f (tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2).
- CMake helper pin remains desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095.
- Target: meson_options.txt@998b92ad0052398df0f7843422c79f674c20dd58, mode 100644, fully read at 5 physical lines / 257 Git-tree bytes.
- Pinned MicroTeX recursive tree: truncated=false, 315 entries / 279 blobs.
- Tree order: latex.qrc, meson.build, meson_options.txt, then prebuilt/.
- Pre-write independent responsibility registration count: 0. The exact target is present in exactly two existing dependency-context rows: strict179 Ubuntu GTK workflow and strict189 meson.build.

This dossier closes source/build-option schema provenance only. It does not close the separately owned tdesktop/Fabushi formula renderer, resource initialization, packaging, runtime behavior, or release acceptance.

## Exact option schema

meson_options.txt defines exactly two options.

1. TARGET_DEVEL
   - type: boolean
   - default: true
   - purpose comment: generate pkgconfig files and install headers

2. TARGET_DEMO
   - type: combo
   - legal choices: NONE and GTK
   - default: NONE
   - source comment says QT and Win32 are TODO only; they are not legal values in this exact schema.

There is no third option in the file.

## Complete direct Meson consumer inventory

All 17 tracked meson.build manifests in the fixed child tree were exact-blob re-read.

Only src/meson.build@84ce4f05fceb8419a326bf6a059d2583c793fc9a directly reads TARGET_DEVEL:
- install_headerfiles = get_option(TARGET_DEVEL)
- a second get_option(TARGET_DEVEL) gates pkgconfig.generate.

Only two manifests directly read TARGET_DEMO:
- src/meson.build reads get_option(TARGET_DEMO) != NONE before entering src/samples and constructing the clatexmath executable.
- src/samples/meson.build@68e634d18291a5113c800725782bcc4d5e7b3327 reads get_option(TARGET_DEMO) == GTK and then defines gtkmm_main.cpp plus fontconfig, gtkmm-3.0 and gtksourceviewmm-3.0 dependencies.

No other tracked Meson manifest directly reads either option.

TARGET_DEVEL also fans out indirectly through the install_headerfiles variable. The conditional header-install manifests are atom, box, core, fonts, graphic, root src, Cairo platform, tracked Qt platform, res, res/reg, and utils. Strict189 established 45 tracked header declarations with 44 currently reachable because src/platform/meson.build enters only cairo and not qt.

## TARGET_DEVEL behavior

Default true:
- pkgconfig.generate is configured for clatexmath;
- install_headerfiles is true, so reachable conditional install_headers rules are active.

Explicit false:
- pkg-config generation is suppressed;
- conditional development-header installation is suppressed.

Negative boundary:
- TARGET_DEVEL does not control construction or installation of the clatexmath library.
- TARGET_DEVEL does not control root meson.build resource installs. res/fonts, res/greek, res/cyrillic and the three root resource data files are independent of this option.

## TARGET_DEMO NONE / GTK behavior

Default NONE:
- src/meson.build does not enter subdir(samples);
- no clatexmath sample executable is constructed by this branch.

GTK:
- src/meson.build enters samples;
- src/samples/meson.build selects samples/gtkmm_main.cpp and the three GTK-related dependencies;
- src/meson.build then constructs an installable clatexmath executable linked to clatexmath_lib.

Invalid or future-looking values:
- because TARGET_DEMO is a Meson combo with choices NONE and GTK, values such as QT or Win32 are rejected by option validation in this exact schema;
- the comment mentioning QT and Win32 is not executable authority;
- a future schema expansion to a new non-NONE value must also add a matching sample branch. Otherwise src/meson.build would enter samples while samples_src/samples_dep remain undefined. The current schema prevents that composition failure.

## External Meson configure consumer and documentation

.github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764 is the direct pinned CI configure consumer:
- meson _build -DTARGET_DEMO=GTK
- ninja -C _build
- archive/upload build output

The workflow does not set TARGET_DEVEL, so the default true applies. It does not run meson install, a sample executable, unit tests, CTest, assertions, or packaged-product acceptance.

README.md@824af04fd8607532efed3f73752b81e5093577ef documents the Cairo Meson path and demonstrates TARGET_DEMO=NONE or GTK. Documentation does not broaden the exact combo choices.

## Accepted tdesktop shipping bypass

Accepted tdesktop keeps the pins unchanged:
- Telegram/ThirdParty/MicroTeX resolves to 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f.
- cmake resolves to desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095.

The shipping producer remains external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 in cmake_helpers:
- directly lists MicroTeX sources;
- includes src/platform/qt/graphic_qt.cpp;
- sets BUILD_QT PUBLIC;
- links desktop-app::external_qt;
- attaches res/bundled.qrc and target_prepare_qrc;
- exports desktop-app::external_microtex.

That file does not consume TARGET_DEVEL, TARGET_DEMO, meson_options.txt, or the standalone Meson composition.

Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex. Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 links td_iv and on macOS copies external_microtex_bundled.rcc into the app bundle. The pinned runtime formula owners remain iv_markdown_microtex.cpp and iv_markdown_math_renderer.cpp.

Therefore these Meson options are dependency/build provenance, not accepted tdesktop shipping controls.

## License boundary

The target build-option file is outside res and is covered by the project root MIT license, LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2.

README.md explicitly excludes fonts and XML under res from the blanket MIT statement. The fixed tree retains:
- res/greek/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7
- res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7
- font-specific Knuth, dsrom and SIL OFL notices.

Changing TARGET_DEVEL or TARGET_DEMO does not change those licenses. Standalone root Meson resource installation remains a separate responsibility, and accepted tdesktop shipping separately uses the reduced bundled.qrc path.

This is provenance, not legal advice.

## Negative and composition oracles

- TARGET_DEVEL=false disables the library. False: it suppresses development metadata/header install surfaces only.
- TARGET_DEVEL controls resource installation. False: root resource installs are unconditional.
- TARGET_DEMO=NONE builds a sample. False: it skips the sample subdir/executable.
- QT and Win32 are legal TARGET_DEMO values. False: they appear only in a TODO comment; legal choices are NONE and GTK.
- Any non-NONE value is safely handled. False as a future schema claim: the current schema only permits GTK, and a new non-NONE value would need a matching src/samples branch.
- Ubuntu TARGET_DEMO=GTK proves tests pass. False: the workflow compiles/archives and runs no tests.
- meson_options.txt controls accepted tdesktop shipping. False: accepted shipping bypasses Meson through pinned CMake external_microtex.
- Root MIT applies to all resources. False: README preserves resource-specific license boundaries.
- This source read closes renderer/resource/package/release responsibilities. False: those owners and gates remain separate and open.

## Coverage accounting

Before strict190:
- recursive source entries: 16,379
- unknown: 16,187
- unread: 16,162
- omitted: 0
- responsibilities: 207

After registering this independently unrecorded source:
- recursive source entries: 16,379
- unknown: 16,186
- unread: 16,161
- omitted: 0
- responsibilities: 208

baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence

The pre-receipt parent 59d84e726b285a40020e473a72599710994a75b2 is terminal-success:
- TDRP Revision 9 source authority run 40116367956: 61/61 jobs success
  - authority 12570285067 — sha256:c87d85698a5fc45a63c77ac3f49f06333020862031bd4830a967c12b69af08f6
  - closure 12570345510 — sha256:d4286aa925d4b232e178abe66cc1f16afdbca83d1f60af6ed60c3e06e8a0d23f
  - reachability 12570005694 — sha256:af66cbf31b72d8717cb2071a89664e369f4e3559e958623fd86c365f76fdfb33
  - Boost 12569795463 — sha256:031d4329a87e8afc969ab30a2e6c94652f8c6477c1fc6bcc3aa0310e2446db4d
- Rust desktop runtime run 40116367954: 6/6 jobs success
  - artifact 12567684712 — sha256:aa4fc8e2eaca4bd46bbf19552a7977b3983589b72dfe35aad14f730d899f39a8
- Official MCP marketplace run 40116368142: 1/1 job success

These are parent evidence after publication. The receipt exact-head must be independently revalidated before this round reports current-head Actions success.

## Traceability

- Requirement: TDRP-R9-MICROTEX-MESON-OPTIONS-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_MESON_OPTIONS_PROVENANCE_001
- Exact-source oracle: ORA-TDRP-MICROTEX-MESON-OPTIONS-EXACT-SOURCE-001
- TARGET_DEVEL oracle: ORA-TDRP-MICROTEX-TARGET-DEVEL-SCHEMA-CONSUMER-001
- TARGET_DEMO composition oracle: ORA-TDRP-MICROTEX-TARGET-DEMO-NONE-GTK-COMPOSITION-001
- Invalid-choice oracle: ORA-TDRP-MICROTEX-TARGET-DEMO-INVALID-CHOICE-001
- CI consumer oracle: ORA-TDRP-MICROTEX-MESON-OPTIONS-UBUNTU-GTK-CONSUMER-001
- Shipping-bypass oracle: ORA-TDRP-MICROTEX-MESON-OPTIONS-TDESKTOP-SHIPPING-BYPASS-001
- License-boundary oracle: ORA-TDRP-MICROTEX-MESON-OPTIONS-LICENSE-BOUNDARY-001
- Invariant: INV-TDRP-MICROTEX-MESON-OPTIONS-NOT-TDESKTOP-SHIPPING-ROOT-001
- Invariant: INV-TDRP-MICROTEX-TARGET-DEMO-CHOICES-COUPLED-TO-SAMPLE-CONSUMER-001
- Invariant: INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
