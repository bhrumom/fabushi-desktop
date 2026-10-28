# desktop-app/MicroTeX meson.build complete read — strict189

## Authority

- Fabushi PR: `bhrumom/fabushi-desktop#154`
- Branch: `tdrp-r9-entry31-includes-r27-20261020`
- Pre-receipt live HEAD: `db0f4b96eaa7b2ee0fce83e29a3f5278a1c66ef8`
- Pre-receipt tree: `9837b0136badab4a18f27bd014e2800cef432258`
- Live main observed before write: `16aae12ea258f6a872a46aa47824766f64e1bdca` (tree `82126ec4a712fca352a49d6a3caae70c7d21ca8c`)
- Accepted source pin remains `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf` (tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`).
- Direct child pin remains `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.
- CMake helper pin remains `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095`.
- Target: `meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5`, mode `100644`, fully read at 17 physical lines / 606 bytes.
- Pinned MicroTeX recursive tree: `truncated=false`, 315 entries / 279 blobs.
- In current fixed-tree blob order, `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270` is immediately followed by this target and then `meson_options.txt@998b92ad0052398df0f7843422c79f674c20dd58`.
- Pre-write independent responsibility registration count: **0**. The exact target is already present in **3 dependency_ids rows** (Ubuntu GTK workflow, .gitignore, latex.qrc); a .gitignore evidence string also mentions it narratively, but no independent `meson.build` responsibility exists.

This dossier closes source/build/install provenance only. It does **not** close the separately owned tdesktop/Fabushi formula renderer, Qt resource packaging, runtime initialization, packaging, or release acceptance.

## Root Meson project and development environment

The exact root manifest defines project `clatexmath`, language `cpp`, version `0.0.5`, default `buildtype=release`, default `warning_level=3`, and `subdir('src')` as its source-composition entry.

For Meson `>=0.58.0`, it creates a Meson developer environment and sets `CLM_DEVEL=<current source directory>/res` through `meson.add_devenv`. This is development/build-run state for locating the source resource tree, not evidence that a shipped Fabushi or tdesktop process requires or persists `CLM_DEVEL`.

## Resource installation responsibility

The root manifest unconditionally installs three complete subtrees into `${datadir}/clatexmath`:

- `res/fonts`: 33 pinned blobs = 30 TTF resources + 3 license documents;
- `res/greek`: 21 pinned blobs = 20 resource files + `LICENSE`;
- `res/cyrillic`: 21 pinned blobs = 20 resource files + `LICENSE`.

It additionally installs:
- `res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee`;
- `res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b`;
- `res/.clatexmath-res_root@e69de29bb2d1d6434b8b29ae775ad8c2e48c5391`.

These installs are independent of `TARGET_DEVEL`. They describe the standalone Meson install surface, not accepted tdesktop package membership.

## TARGET_DEVEL: pkg-config and header install surface

`meson_options.txt@998b92ad0052398df0f7843422c79f674c20dd58` declares `TARGET_DEVEL` as boolean, default **true**. `src/meson.build@84ce4f05fceb8419a326bf6a059d2583c793fc9a` maps it to `install_headerfiles`.

With it enabled:
- `pkgconfig.generate` publishes `clatexmath` metadata using project version `0.0.5` and include subdir `clatexmath`;
- conditional `install_headers` blocks publish the development headers.

Across the 17 tracked Meson manifests, there are **45** conditional header-install declarations. The current root composition reaches **44**. The sole extra declaration is `graphic_qt.h` in the tracked Qt Meson manifest, which the current platform parent does not enter.

Reachable header families are root (4), atom (7), box (4), core (6), fonts (7), graphic (2), Cairo platform (1), res (2), reg (2), and utils (9): exactly **44**.

## TARGET_DEMO=GTK

`meson_options.txt` declares `TARGET_DEMO` as a combo with choices `NONE` and `GTK`, default **NONE**. When non-NONE, `src/meson.build` enters `src/samples` and builds an installable `clatexmath` executable.

`src/samples/meson.build@68e634d18291a5113c800725782bcc4d5e7b3327` defines the only non-NONE branch:
- source `samples/gtkmm_main.cpp`;
- dependencies `fontconfig`, `gtkmm-3.0`, `gtksourceviewmm-3.0`.

This is an optional GTK sample/demo surface, not the accepted tdesktop formula runtime.

## Platform composition: Cairo selected, Qt tracked but unselected

`src/platform/meson.build@d2995274e8aea630f6f45cc1761353d18fd1854e` only initializes `platform_src` / `platform_deps` and enters `subdir('cairo')`.

The reached `src/platform/cairo/meson.build@73b505759b982a669d62dcac0b7662b0e9384dda` adds `BUILD_GTK`, `graphic_cairo.cpp`, `fontconfig`, `gdkmm-3.0`, and the Cairo header install.

A separate tracked `src/platform/qt/meson.build@b73017e71c4362fc5c99df52b6792e1a13763161` defines `BUILD_QT`, Qt5 Core/Gui/Widgets, moc preprocessing and `graphic_qt.h`, but there is no parent `subdir('qt')` in the pinned current Meson composition. Tracked presence must not be promoted to active composition.

This matches `README.md@824af04fd8607532efed3f73752b81e5093577ef`, whose Meson section explicitly describes building the **Cairo version** and shows `TARGET_DEMO=NONE` or `GTK`.

## Direct Meson CI consumer

`.github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764` is the direct pinned CI consumer:
1. Ubuntu 20.04;
2. checkout;
3. apt installs Meson plus GTK/GSVMM dependencies;
4. `meson _build -DTARGET_DEMO=GTK`;
5. `ninja -C _build`;
6. compresses `_build`;
7. uploads the build archive.

It runs no unit test, sample executable, assertion phase, packaged-product test, or runtime acceptance. Its strict179 row remains the independent CI responsibility; here it is direct-consumer context.

## Accepted tdesktop shipping bypass

The accepted tdesktop graph keeps the pins unchanged:
- `.gitmodules@2a71e861fab57dfd74d66e787867f2ede203a099` maps `Telegram/ThirdParty/MicroTeX` to desktop-app/MicroTeX and `cmake` to desktop-app/cmake_helpers;
- `Telegram/ThirdParty/MicroTeX` resolves to `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`;
- root `cmake` resolves to `7ceca3019b4750d7a5acb2de0faffd523ee14095`.

The actual shipping producer is `desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334`, not the root Meson manifest. It directly lists pinned MicroTeX sources, explicitly includes `src/platform/qt/graphic_qt.cpp`, sets `BUILD_QT` PUBLIC, links `desktop-app::external_qt`, attaches `res/bundled.qrc`, runs `target_prepare_qrc`, and exports `desktop-app::external_microtex`.

`Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` links that shipping target. `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` owns one-time bundled-resource/MicroTeX initialization and parse/render calls; `iv_markdown_math_renderer.cpp@46864b7ce87207c6e57b6bcc7610337120580bea` owns formula rendering/cache behavior. `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` links `td_iv` and on macOS packages `external_microtex_bundled.rcc`.

No accepted shipping step invokes `meson.build`. Standalone Meson build/install behavior and accepted tdesktop Qt CMake shipping are distinct responsibilities.

## License boundary

The root `LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2` is MIT for MicroTeX code. `README.md@824af04fd8607532efed3f73752b81e5093577ef` explicitly excludes fonts and XML under `res` from that blanket claim.

The Meson resource subtree install carries:
- `res/fonts/licences/Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8`;
- `res/fonts/licences/License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c`;
- `res/fonts/licences/OFL.txt@6836a0be000498319aebcc92a172159b6f80bb52`;
- `res/greek/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7`;
- `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7`.

This is provenance, not legal advice. Standalone Meson installing Greek/Cyrillic trees does not imply the accepted tdesktop `bundled.qrc` ships them; strict188 established that accepted shipping bundle is the smaller 30-TTF set.

## Negative oracles

- “The current MicroTeX Meson graph builds Qt.” False: the parent platform manifest enters only Cairo.
- “A tracked Qt Meson manifest is active.” False: there is no current parent `subdir('qt')`.
- “CLM_DEVEL is a shipped runtime requirement.” False: it is only a Meson developer environment on Meson >=0.58.
- “TARGET_DEVEL controls resource installation.” False: root resource installs are unconditional.
- “TARGET_DEMO=GTK proves tests pass.” False: the direct Ubuntu workflow compiles/archives and runs no test phase.
- “Standalone Meson install is accepted tdesktop shipping composition.” False: tdesktop bypasses it through pinned cmake_helpers external_microtex.
- “MIT covers every installed font/XML resource.” False: README and resource-specific licenses preserve a layered boundary.
- “This source read closes formula rendering, resource initialization, packaging or release acceptance.” False: those owners/gates remain separate and open.

## Coverage accounting

Before strict189:
- recursive source entries: 16,379
- unknown: 16,188
- unread: 16,163
- omitted: 0
- responsibilities: 206

After registering this one independently unregistered source:
- recursive source entries: 16,379
- unknown: **16,187**
- unread: **16,162**
- omitted: **0**
- responsibilities: **207**

`baseline_identity_accepted=false` and `source_closure_ready=false` remain fail-closed.

## Parent exact-head evidence

The pre-receipt parent `db0f4b96eaa7b2ee0fce83e29a3f5278a1c66ef8` is terminal-success:
- TDRP Revision 9 source authority run `40114563092`: **61/61 jobs success**
  - authority `12567688901` — `sha256:70a3a7bd73a700727fc51bca590a00dea4df0f5cb65f2eb1a77ee355954b4056`
  - closure `12567318919` — `sha256:b0a932f8bd0ea73739ee62b8f234d833658a91e613a3ff50ac0a1ff64bf6a995`
  - reachability `12568001760` — `sha256:ac1ee537e5cfeaafa008f9a240e8303c874cc7f2abf7bec3a96c7b3d3d08d4e4`
  - Boost `12568415551` — `sha256:4d24df45483f4b0a0059001b79837acb523ff6afbc6c45518a29a4bde218ddbd`
- Rust desktop runtime run `40114563093`: **6/6 jobs success**
  - artifact `12567697503` — `sha256:0d48d12dcb36880f9f020c29f1d513723e1b37d6b5ce62ae0be9ca4f77fb354b`
- Official MCP marketplace run `40114563111`: **1/1 job success**

These become parent evidence after the receipt commit. The receipt exact-head must be revalidated before this round reports current-head Actions success.

## Traceability

- Requirement: `TDRP-R9-MICROTEX-MESON-BUILD-PROVENANCE-001`
- Target symbol: `TDRP_MICROTEX_MESON_BUILD_PROVENANCE_001`
- Exact-source oracle: `ORA-TDRP-MICROTEX-MESON-BUILD-EXACT-SOURCE-001`
- Devel/install oracle: `ORA-TDRP-MICROTEX-MESON-DEVEL-INSTALL-001`
- Platform-composition oracle: `ORA-TDRP-MICROTEX-MESON-CAIRO-ONLY-COMPOSITION-001`
- Direct-CI-consumer oracle: `ORA-TDRP-MICROTEX-MESON-UBUNTU-GTK-CONSUMER-001`
- Shipping-bypass oracle: `ORA-TDRP-MICROTEX-MESON-TDESKTOP-SHIPPING-BYPASS-001`
- License-boundary oracle: `ORA-TDRP-MICROTEX-MESON-LICENSE-BOUNDARY-001`
- Invariant: `INV-TDRP-MICROTEX-MESON-NOT-TDESKTOP-SHIPPING-ROOT-001`
- Invariant: `INV-TDRP-MICROTEX-MESON-QT-MANIFEST-UNSELECTED-001`
- Invariant: `INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001`
