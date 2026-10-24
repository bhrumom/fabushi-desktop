# desktop-app/MicroTeX Windows Qt workflow complete read — strict181

## Authority

- Parent PR #154 / branch exact HEAD before this write: `38f3195f364ac7924c0368f0d89dea1418255e90`, tree `8909648ecb189084b843ebd9dfff28df50c10bcf`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Accepted Telegram Desktop: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned child: `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`.
- Child recursive tree: `truncated=false`, 315 entries / 279 blobs.
- Target blob: `.github/workflows/windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838` (1,627 Git-tree bytes; 59 line slots).
- The pinned child has exactly three workflow blobs: Ubuntu GTK, Windows GDI+ and Windows Qt. The first two were already registered by strict179/strict180; Windows Qt was the remaining workflow target before this write.

## Complete workflow read

The workflow is named `Windows Qt`, triggers on push and pull_request, and declares a Windows build matrix:

1. `matrix.os` contains `windows-2019`.
2. The matrix `include` data contains Qt `5.15.2` and `6.0.0` records, both using `win64_msvc2019_64`, `x64` MSVC and `msvc2019_64` install metadata. This is recorded as source data; no claim is made here that both variants were successfully executed by this historical child workflow.
3. Source checkout uses mutable `actions/checkout@v2`.
4. CMake/Ninja provisioning uses mutable branch `lukka/get-cmake@latest`.
5. The workflow clones `https://github.com/Microsoft/vcpkg.git` without a ref and bootstraps that live checkout.
6. MSVC environment setup uses mutable major tag `ilammy/msvc-dev-cmd@v1` with `arch: x64`.
7. Qt provisioning uses the versioned tag `jurplel/install-qt-action@v2.13.0`, passing the matrix Qt version/architecture and disabling its cache shortcut.
8. Configure is `cmake .. -DCMAKE_TOOLCHAIN_FILE=.../vcpkg.cmake -DQT=ON -DCMAKE_BUILD_TYPE=Release -G Ninja`.
9. Compile is plain `ninja`.

There is no CTest, unit-test command, sample launch, executable smoke test, runtime assertion or packaged acceptance step. A successful child workflow is compile evidence only.

## Direct action, toolchain and acquisition provenance

- `actions/checkout@v2` currently resolves to commit `0717577d45739eb3c851188b29f50ed6c0b2194e`; `action.yml@9f346aa7726e2cb9df0d98417c0bcf77a6f215c7`. The workflow retains the mutable major tag.
- `lukka/get-cmake@latest` currently resolves to commit `da37b03df250c343a90268c30948f7814f31d88e`; `action.yml@f3fa62850305cbb701f746f6d6a14fc8cf2a8739`. The workflow retains the mutable branch name.
- `ilammy/msvc-dev-cmd@v1` currently resolves to commit `0b201ec74fa43914dc39ae48a89fd1d8cb592756`; `action.yml@b20fad3c4c1f9c98913251f57311e941b69a57cc`. The workflow retains the mutable major tag.
- `jurplel/install-qt-action@v2.13.0` resolves to commit `88f3374475d094ce56ca6a7cc41cfda5a6ec093c`; `action.yml@7dd0a0a66e25fd7e721fe3ca4f616be8e81e5a34`; its license blob is `b3cc338d8a60bedbb0cb20df0db9670c5a677203` (MIT). The action metadata uses Node 12 and declares an `aqtinstall` version input/default; this remains third-party CI provisioning provenance, not Fabushi runtime code.
- `Microsoft/vcpkg` `master` currently resolves to `0c2a9ace1964b818b15f7ae50ba1c42404918a7b`. The child workflow has no ref on its clone, so this resolution is not frozen by the pinned MicroTeX commit.
- Pinned MicroTeX `vcpkg.json@c6c84611e66339b7a599902cd19009719532efef` declares `tinyxml2`.

Mutable/current resolutions above are provenance observations only; they are not immutable supply-chain pins or Fabushi shipping trust evidence.

## CMake Qt build and sample path

Pinned `CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9` handles `QT=ON` before the platform-specific WIN32 branch:

- defines `BUILD_QT`;
- resolves Qt 6 or Qt 5 `Gui`, `Widgets` and `PrintSupport`;
- adds `src/platform/qt/graphic_qt.cpp@016b07c72ad92fab12af5f2daae435976fc82de5` to the LaTeX library and links Qt Gui;
- builds `LaTeXQtSample` from `src/samples/qt_texwidget.cpp@1e185d2b9effa15f2bea6f0fc01c25f9ed7c6505`, `qt_mainwindow.cpp@ba837ed5450c5e40e07ef983e95c2243995cd500` and `qt_main.cpp@2b089368c88d415ded50725987ecfbdc40053f49`;
- associated headers are `qt_texwidget.h@6bac6b5213a6f665bce830a2e83856ee5b8c4e19` and `qt_mainwindow.h@e22be299c9441c41a92852727596dff27d747db7`;
- links Widgets, PrintSupport and LaTeX, enables AUTOMOC, and names the sample executable `LaTeX`.

The Qt adapter header is `src/platform/qt/graphic_qt.h@777faa8d7dd18695fb4b42f4a2992c4f913c79d4`. The implementation supplies `Font_qt`, `TextLayout_qt` and `Graphics2D_qt`, including font loading, bounds/layout, painter transforms and text/path/primitive drawing.

Strict181 records the child workflow/build provenance only. It does not migrate or close the Qt adapter, sample runtime, formula engine or resource responsibilities.

## Accepted tdesktop production boundary

The accepted tdesktop `.github` subtree is `795f933c8f1be4b46e0c952bde8b20d436351a24`, recursively read with `truncated=false` and 32 entries. It contains no MicroTeX/Windows-Qt/LaTeXQtSample workflow path.

Exact accepted-source searches show:

- `windows-qt.yml`: zero hits;
- `LaTeXQtSample`: zero hits;
- `BUILD_QT`: zero hits;
- `graphic_qt`: one direct production hit, `Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2`.

That production source directly includes `platform/qt/graphic_qt.h`, initializes/parses MicroTeX and constructs `tex::Graphics2D_qt` for painter-backed formula rendering. The production build path remains:

- `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` includes the MicroTeX integration sources and links `desktop-app::external_microtex`;
- `iv_markdown_math_renderer.cpp@46864b7ce87207c6e57b6bcc7610337120580bea` owns formula cache/measure/render integration;
- `iv_editor_math_box.cpp@5e873da9b408fb1e58bf422e4fbce8923e0f6fb9` consumes `RenderWithMicrotex`;
- `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` and `Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` package `external_microtex_bundled.rcc`.

Therefore the child Windows Qt workflow is not a tdesktop shipping workflow, while its Qt adapter is genuinely production-reachable through the pinned dependency. Compile-only workflow provenance cannot close formula parsing, measurement/rendering, cache/fallback, painter semantics, resources, packaging or Fabushi production ownership.

## README and layered license boundary

- `README.md@824af04fd8607532efed3f73752b81e5093577ef` describes Qt as a supported build mode and documents `-DQT=ON`; it separately excludes fonts/XML resources under `res` from the blanket project MIT statement.
- Root `LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2` is MIT.
- `res/greek/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` and `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` contain GPL-3.0 text.
- `res/fonts/licences/Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8` and `License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c` carry font-specific terms.

No all-MIT claim is made for the shipping resource set.

## Negative oracles

- Accepted tdesktop has no `windows-qt.yml`, `LaTeXQtSample` or `BUILD_QT` hit.
- The child workflow executes no tests and does not launch `LaTeXQtSample`.
- `checkout@v2`, `get-cmake@latest`, `msvc-dev-cmd@v1` and the no-ref vcpkg clone are not immutable provenance.
- The versioned install-qt action tag does not make the workflow a Fabushi shipping trust or release gate.
- A successful child build is not evidence for formula semantics, Qt painter behavior, resource/license closure, packaged desktop acceptance or Fabushi shipping ownership.
- Real tdesktop MicroTeX production responsibilities remain open.

## Traceability anchor

- Target symbol: `TDRP_MICROTEX_WINDOWS_QT_WORKFLOW_PROVENANCE_001`
- Responsibility: `TDRP-R9-MICROTEX-WINDOWS-QT-WORKFLOW-PROVENANCE-001`

## Traceability IDs

- `CAP-DEPENDENCY-CI-PROVENANCE`
- `CAP-DEPENDENCY-BUILD-INPUT-PROVENANCE`
- `CAP-DEPENDENCY-WINDOWS-QT-BUILD-PROVENANCE`
- `ORA-TDRP-MICROTEX-WINDOWS-QT-WORKFLOW-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-WINDOWS-QT-CMAKE-BUILD-001`
- `ORA-TDRP-MICROTEX-WINDOWS-QT-NO-TEST-STEP-001`
- `ORA-TDRP-MICROTEX-WINDOWS-QT-NO-SHIPPING-WORKFLOW-001`
- `ORA-TDRP-MICROTEX-WINDOWS-QT-ACTION-REFS-001`
- `ORA-TDRP-MICROTEX-WINDOWS-QT-VCPKG-UNPINNED-001`
- `ORA-TDRP-MICROTEX-QT-PRODUCTION-CONSUMER-OPEN-001`
- `INV-TDRP-MICROTEX-WINDOWS-QT-DEPENDENCY-CI-ONLY-001`
- `INV-TDRP-MICROTEX-WINDOWS-QT-NO-PARALLEL-RUNTIME-001`
- `INV-TDRP-MICROTEX-WINDOWS-QT-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001`
- `CONTRACT-TDRP-MICROTEX-WINDOWS-QT-PINNED-SOURCE-001`
- `CONTRACT-TDRP-MICROTEX-WINDOWS-QT-CMAKE-BUILD-001`
- `CONTRACT-TDRP-MICROTEX-WINDOWS-QT-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-QT-SOURCE-DRIFT-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-QT-TOOLCHAIN-DRIFT-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-QT-MUTABLE-REF-DRIFT-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-QT-VCPKG-DRIFT-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-QT-PRODUCT-PROMOTION-001`
- `FAULT-TDRP-MICROTEX-QT-RESOURCE-LICENSE-DRIFT-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`

## Owner disposition

Selected owner: existing TDRP recursive source/dependency and CI/build-provenance governance.

Rejected for this workflow row: Fabushi rich-document/formula renderer, resource, Windows platform and package owners because this YAML only describes third-party child compilation. A `MicroTeXRuntime`, `TelegramRenderer`, `QtRuntime`, second rich-document root or second Windows platform owner would preserve dependency/upstream architecture and violate existing-owner-first.

Production entrypoints for the workflow row are empty. The exact tdesktop Qt adapter and formula consumers above are negative-boundary production evidence only and remain separately open.

## Coverage accounting

Before this write: source entries 16,379; unknown 16,196; unread 16,171; omitted 0; responsibilities 198.

After recording this target blob: source entries 16,379; unknown 16,195; unread 16,170; omitted 0; responsibilities 199. All three pinned MicroTeX workflow blobs are now registered as distinct workflow-provenance targets, while the wider child source/resource tree and real tdesktop production responsibilities remain independently open. Accepted tdesktop recursive source closure remains open.

## Parent exact-head evidence

Parent HEAD `38f3195f364ac7924c0368f0d89dea1418255e90` is terminal-success:
- TDRP Revision 9 run `40098148898`: 61/61 jobs success; source-authority artifact `12561402585` sha256:b752e680bb624929f6fd0b8b2562239a26b222cceef6587f66da7e38c0cba9cc; recursive closure `12562330584` sha256:8efa0d6941ccc3733e7e6a1756cf5f3bd56b7525b58aff383830b86ba66368a6; recursive reachability `12561472711` sha256:88a02a0045e706a7278f046b566477eabe94b84e1577cbce409949c1e0dfe376; Boost reachability `12561856328` sha256:846bc03ca2d9ebf52974bc67444bfa02884a5f8b8b6c92bdf3512c5fa73fb520.
- Rust desktop runtime run `40098148889`: 6/6 jobs success; artifact `12562170509` sha256:73774dd3feae7306ab33e674a2fd72714d25d94e5340f3bb5da5b0cdf0281b86.
- Official MCP marketplace run `40098148944`: 1/1 contracts success.

These are parent-head evidence. The strict181 receipt commit requires fresh exact-head terminal results.

## Closure

- `baseline_identity_accepted=false`
- `source_closure_ready=false`
- independent release acceptance remains not accepted
- no production formula/Qt renderer/resource/package implementation or shipping acceptance is claimed by this dependency CI/build row.
