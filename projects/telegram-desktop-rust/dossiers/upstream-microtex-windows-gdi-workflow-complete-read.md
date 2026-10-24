# desktop-app/MicroTeX Windows GDI+ workflow complete read — strict180

## Authority

- Parent PR #154 / branch exact HEAD before this write: `993a5bf9c8c6b19d0b970cf6035474fd6f316b7f`, tree `2bebba64cac996f1e879fed1c9c48febd47514a7`.
- Accepted Telegram Desktop: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned child: `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`.
- Child recursive tree: `truncated=false`, 315 entries / 279 blobs.
- Target blob: `.github/workflows/windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa` (1,030 Git-tree bytes; 41 line slots).

## Complete workflow read

The workflow is named `Windows GDI+`, triggers on push and pull_request, and has one Windows build job:

1. Matrix contains only `windows-2019`.
2. Source checkout uses mutable `actions/checkout@v2`.
3. CMake/Ninja provisioning uses mutable branch `lukka/get-cmake@latest`.
4. The workflow clones `https://github.com/Microsoft/vcpkg.git` without a ref, bootstraps that checkout, then configures CMake with its toolchain file.
5. MSVC environment setup uses mutable major tag `ilammy/msvc-dev-cmd@v1` with `arch: x64`.
6. Configure is `cmake .. -DCMAKE_TOOLCHAIN_FILE=.../vcpkg.cmake -DCMAKE_BUILD_TYPE=Release -G Ninja`.
7. Compile is plain `ninja`.

There is no CTest, unit-test command, sample launch, executable smoke test, runtime assertion or packaged acceptance step. Success proves only that the child Windows/GDI+ graph compiled in that workflow environment.

## Direct action/toolchain and acquisition provenance

- `actions/checkout@v2` currently resolves to commit `0717577d45739eb3c851188b29f50ed6c0b2194e`; `action.yml@9f346aa7726e2cb9df0d98417c0bcf77a6f215c7`. The workflow retains the mutable major tag.
- `lukka/get-cmake@latest` currently resolves to branch commit `da37b03df250c343a90268c30948f7814f31d88e`; `action.yml@f3fa62850305cbb701f746f6d6a14fc8cf2a8739` uses Node 24 and installs CMake/Ninja. The workflow retains the mutable branch name.
- `ilammy/msvc-dev-cmd@v1` currently resolves to commit `0b201ec74fa43914dc39ae48a89fd1d8cb592756`; `action.yml@b20fad3c4c1f9c98913251f57311e941b69a57cc` uses Node 20. The workflow retains the mutable major tag.
- `Microsoft/vcpkg` default branch is `master`; its live branch head at this read is `0c2a9ace1964b818b15f7ae50ba1c42404918a7b`. Because the workflow clones with no ref, this resolution is not frozen by the pinned MicroTeX commit.
- Pinned MicroTeX `vcpkg.json@c6c84611e66339b7a599902cd19009719532efef` declares `tinyxml2`.

These mutable/unpinned resolutions are provenance observations, not durable supply-chain pins or Fabushi shipping trust evidence.

## CMake Windows/GDI+ source and sample path

- `CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9` selects the `elseif(WIN32)` branch, defines `BUILD_WIN32`, builds `LaTeXWin32Sample` from `src/platform/gdi_win/graphic_win32.cpp` plus `src/samples/win32_main.cpp`, links `gdiplus` and `LaTeX`, and names the sample executable `LaTeX`.
- `src/platform/gdi_win/graphic_win32.cpp@054860effc94bd42c7004e3bd6523f71aee059b5` implements Windows font loading/style selection, text layout measurement/drawing and the GDI+ graphics adapter.
- `src/platform/gdi_win/graphic_win32.h@f2ee130adfd6e61c5871d972197466bd163cf88b` declares those Windows graphics/font/text-layout adapters.
- `src/samples/win32_main.cpp@a7c2a150c96ec2568394fa017c9bf67204006713` starts/shuts down GDI+, initializes/releases LaTeX, parses formulas and draws through `Graphics2D_win32`.
- `example/CMakeLists.txt@cd997be0b043913d17c49c1e1e2c50df2c5d6942` is a separate optional Qt `latex2png` example and is not the GDI+ workflow sample path.

Strict180 records only the child Windows CI/build provenance. It does not migrate or close the GDI+ adapter or sample runtime.

## Accepted tdesktop production boundary

Accepted tdesktop `.github` tree `795f933c8f1be4b46e0c952bde8b20d436351a24` was recursively read with `truncated=false` and 32 entries; there are zero paths matching `windows-gdi`, `MicroTeX` or `gdi`. Thus the child workflow itself is not a Telegram Desktop shipping workflow.

The pinned MicroTeX dependency remains genuinely production-reachable in Telegram Desktop:

- `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` links `td_iv` with `desktop-app::external_microtex`.
- Previously verified exact-ref production consumers remain `iv_markdown_microtex.cpp/.h`, `iv_markdown_math_renderer.cpp`, editor math integration and bundled RCC packaging.

Therefore Windows child-CI build success cannot close formula parsing, measurement/rendering, cache/fallback, resource packaging, shipping platform integration or Fabushi production ownership.

## README and layered license boundary

- `README.md@824af04fd8607532efed3f73752b81e5093577ef` describes Windows support, requires GDI+ for the Windows demo, and explicitly excludes fonts/XML resources under `res` from the blanket project MIT statement.
- Root `LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2` is MIT.
- `res/greek/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` and `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` contain GPL-3.0 text.
- `res/fonts/licences/Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8` and `License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c` carry font-specific terms.

This row preserves the license split and makes no all-MIT shipping-resource claim.

## Negative oracles

- Accepted tdesktop exact `.github` subtree has zero `windows-gdi`/MicroTeX/GDI workflow paths.
- The child workflow executes no tests and does not launch `LaTeXWin32Sample`.
- `get-cmake@latest`, `msvc-dev-cmd@v1`, `checkout@v2` and the no-ref vcpkg clone are not immutable provenance.
- A successful child build is not evidence for current Windows release packaging, formula semantics, resource/license closure, runtime acceptance or Fabushi shipping ownership.
- Real tdesktop MicroTeX production responsibilities remain open.

## Traceability anchor

- Target symbol: `TDRP_MICROTEX_WINDOWS_GDI_WORKFLOW_PROVENANCE_001`
- Responsibility: `TDRP-R9-MICROTEX-WINDOWS-GDI-WORKFLOW-PROVENANCE-001`

## Traceability IDs

- `CAP-DEPENDENCY-CI-PROVENANCE`
- `CAP-DEPENDENCY-BUILD-INPUT-PROVENANCE`
- `CAP-DEPENDENCY-WINDOWS-GDI-BUILD-PROVENANCE`
- `ORA-TDRP-MICROTEX-WINDOWS-GDI-WORKFLOW-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-WINDOWS-GDI-NO-TEST-STEP-001`
- `ORA-TDRP-MICROTEX-WINDOWS-GDI-NO-SHIPPING-WORKFLOW-001`
- `ORA-TDRP-MICROTEX-WINDOWS-GDI-MUTABLE-REFS-001`
- `ORA-TDRP-MICROTEX-WINDOWS-GDI-VCPKG-UNPINNED-001`
- `ORA-TDRP-MICROTEX-PRODUCTION-CONSUMERS-REMAIN-OPEN-002`
- `INV-TDRP-MICROTEX-WINDOWS-GDI-DEPENDENCY-CI-ONLY-001`
- `INV-TDRP-MICROTEX-WINDOWS-GDI-NO-PARALLEL-RUNTIME-001`
- `INV-TDRP-MICROTEX-WINDOWS-GDI-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001`
- `CONTRACT-TDRP-MICROTEX-WINDOWS-GDI-PINNED-SOURCE-001`
- `CONTRACT-TDRP-MICROTEX-WINDOWS-GDI-CMAKE-BUILD-001`
- `CONTRACT-TDRP-MICROTEX-WINDOWS-GDI-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-GDI-SOURCE-DRIFT-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-GDI-TOOLCHAIN-DRIFT-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-GDI-MUTABLE-REF-DRIFT-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-GDI-VCPKG-DRIFT-001`
- `FAULT-TDRP-MICROTEX-WINDOWS-GDI-PRODUCT-PROMOTION-001`
- `FAULT-TDRP-MICROTEX-RESOURCE-LICENSE-DRIFT-002`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`

## Owner disposition

Selected owner: existing TDRP recursive source/dependency and CI/build-provenance governance.

Rejected for this workflow row: Fabushi shipping renderer/resource/platform/package owners, because this YAML validates a third-party child compile path rather than owning shipping product behavior. A `MicroTeXRuntime`, `TelegramRenderer`, second rich-document root or second Windows platform owner would preserve upstream/dependency architecture and violate existing-owner-first.

Production entrypoints for this workflow row are empty. Separate tdesktop production consumers remain negative-boundary evidence only.

## Coverage accounting

Before this write: source entries 16,379; unknown 16,197; unread 16,172; omitted 0; responsibilities 197.

After recording this target blob: source entries 16,379; unknown 16,196; unread 16,171; omitted 0; responsibilities 198. The pinned MicroTeX tree has 279 blobs; Windows GDI is the second workflow target registered after Ubuntu GTK, while many child blobs remain independently unread/unregistered. Accepted tdesktop recursive source closure remains open.

## Parent exact-head evidence

Parent HEAD `993a5bf9c8c6b19d0b970cf6035474fd6f316b7f` is terminal-success:
- TDRP Revision 9 run `40096598641`: 61/61 jobs success; source-authority artifact `12561370914` sha256:6525d788f6043c6a0fea213f7544008a7819b2de17efc74ccfb7312d15946946; recursive closure `12560759747` sha256:4cfe9ea7176bc29ab57630cc07f0542c4c8305575768b98d4f69a6c611dd674d; recursive reachability `12560912667` sha256:e50fddc6e9f9fee8ea7cd35f4bf309c3a300216e7e9ae1a56fa994ac9fda4538; Boost reachability `12561446264` sha256:87b566999e89faf67df7dee42eb875021f8d6a888059e9885d94f20e039fbd99.
- Rust desktop runtime run `40096598643`: 6/6 success; artifact `12560779194` sha256:d7f009b88bae83c893f7f747eebea802f156abd62fdf1697b4988dc688a5849f.
- Official MCP marketplace run `40096598672`: 1/1 contracts success.

These are parent-head evidence. The strict180 receipt commit requires fresh exact-head terminal results.

## Closure

- `baseline_identity_accepted=false`
- `source_closure_ready=false`
- independent release acceptance remains not accepted
- no production implementation, Windows shipping-owner closure or resource-license closure is claimed by this dependency CI/build row.
