# desktop-app/MicroTeX Ubuntu GTK workflow complete read — strict179

## Authority

- Parent PR #154 / branch exact HEAD before this write: `c71b1a215b44d64cbe70f1c969304760393f0783`, tree `300fef80203dcf4ceab7af492cf3707e918c9471`.
- Accepted Telegram Desktop: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned child: `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`.
- Child recursive tree: `truncated=false`, 315 entries / 279 blobs.
- Target blob: `.github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764` (1,419 Git-tree bytes; 55 line slots).

## Complete workflow read

The workflow is named `Ubuntu GTK`, runs on push and pull_request, and has two Ubuntu 20.04 jobs:

1. `build_by_meson`: `actions/checkout@v2`; apt installs `build-essential meson libgtksourceviewmm-3.0-dev libgtkmm-3.0-dev libtinyxml2-dev`; configures `meson _build -DTARGET_DEMO=GTK`; compiles with `ninja -C _build`; archives the build tree with `tar | zstd`; uploads `artifacts.tar.zst` using `actions/upload-artifact@v1`.
2. `build_by_cmake`: `actions/checkout@v2`; installs the same apt dependencies; creates `build/`; configures `cmake .. -DCMAKE_BUILD_TYPE=Release -G Ninja`; compiles with `ninja`.

There is no test, CTest, executable-run, smoke-test or assertion step in either job. Success proves these child build paths compile on that hosted environment; it does not prove runtime behavior, formula correctness, Fabushi product behavior, packaging, release acceptance or current compatibility with a modern hosted runner.

## Direct action/toolchain provenance

- `actions/checkout@v2` currently resolves to commit `0717577d45739eb3c851188b29f50ed6c0b2194e`; its `action.yml@9f346aa7726e2cb9df0d98417c0bcf77a6f215c7` uses Node 12. The child workflow retains the mutable major tag.
- `actions/upload-artifact@v1` resolves through annotated tag `34622df80861c3ed63eb2bff892de2f1fbf4c9da` to commit `3446296876d12d4e3a0f3145a3c87e67bf0a16b5`; `action.yml@2876ff84ed8cf637a6b5368a6a93ebe396b3d86e` uses the legacy publish plugin. The child workflow retains the mutable major tag.
- `meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5`, `src/meson.build@84ce4f05fceb8419a326bf6a059d2583c793fc9a` and `meson_options.txt@998b92ad0052398df0f7843422c79f674c20dd58` show `TARGET_DEMO=GTK` builds the clatexmath library plus GTK sample executable; Meson also installs resource directories.
- `CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9` defines the LaTeX library and, on UNIX, GTK/Cairo/Fontconfig dependencies plus `LaTeXGtkSample`; the workflow compiles this Release Ninja graph.

## Accepted tdesktop production boundary

Exact search at accepted tdesktop found zero references to `ubuntu-gtk`, `build_by_meson` or `build_by_cmake`: the child workflow itself is not a Telegram Desktop shipping entrypoint.

The MicroTeX dependency is nevertheless production-reachable and must not be dismissed as test-only:

- `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` links `td_iv` against `desktop-app::external_microtex` and registers `iv_markdown_microtex.cpp/.h` and math-renderer sources.
- `Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` owns MicroTeX initialization plus measure/render adaptation and registers `external_microtex_bundled.rcc` on macOS.
- `Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.h@297d4a124d75834e4d876f1eddb7fe5fe7cea965` exposes `EnsureMicrotexInitialized`, `MeasureWithMicrotex` and `RenderWithMicrotex`.
- `Telegram/SourceFiles/iv/markdown/iv_markdown_math_renderer.cpp@46864b7ce87207c6e57b6bcc7610337120580bea` calls both measure and render paths and layers bounded caching/fallback behavior around them.
- `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` and `Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` carry `external_microtex_bundled.rcc` into macOS shipping resources.

Therefore strict179 records only child Ubuntu GTK CI/build provenance. Formula parsing/measurement/rendering, cache/fallback behavior, bundled-resource packaging and related UX/product responsibilities remain separate open migration work.

## README and license boundary

- `README.md@824af04fd8607532efed3f73752b81e5093577ef` describes MicroTeX as a cross-platform embeddable LaTeX renderer and documents GTK/Qt builds, headless rendering, compile-time options and platform adaptation.
- Root `LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2` is MIT.
- README explicitly excludes fonts and XML resources from that blanket MIT statement. The pinned child tree contains `res/greek/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` and `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` with GPL-3.0 text, plus font-specific license files such as `res/fonts/licences/Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8` and `License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c`.
- This row preserves that split provenance and does not claim shipping-license closure for MicroTeX resources.

## Negative oracles

- Workflow-name/job-name searches in accepted tdesktop: `ubuntu-gtk=0`, `build_by_meson=0`, `build_by_cmake=0`.
- Neither workflow job executes tests.
- A successful child build is not evidence for tdesktop/Fabushi formula semantics, bundled-resource correctness, production packaging, platform release acceptance or Fabushi shipping ownership.
- The dependency's real tdesktop production consumers remain open and are explicitly not closed by this provenance row.

## Traceability anchor

- Target symbol: `TDRP_MICROTEX_UBUNTU_GTK_WORKFLOW_PROVENANCE_001`
- Responsibility: `TDRP-R9-MICROTEX-UBUNTU-GTK-WORKFLOW-PROVENANCE-001`

## Traceability IDs

- `CAP-DEPENDENCY-CI-PROVENANCE`
- `CAP-DEPENDENCY-BUILD-INPUT-PROVENANCE`
- `CAP-DEPENDENCY-LINUX-GTK-BUILD-PROVENANCE`
- `ORA-TDRP-MICROTEX-UBUNTU-GTK-WORKFLOW-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-UBUNTU-GTK-DUAL-BUILD-CHAIN-001`
- `ORA-TDRP-MICROTEX-UBUNTU-GTK-NO-TEST-STEP-001`
- `ORA-TDRP-MICROTEX-UBUNTU-GTK-NO-SHIPPING-OWNER-001`
- `ORA-TDRP-MICROTEX-UBUNTU-GTK-MUTABLE-ACTION-REF-001`
- `ORA-TDRP-MICROTEX-PRODUCTION-CONSUMERS-REMAIN-OPEN-001`
- `INV-TDRP-MICROTEX-UBUNTU-GTK-DEPENDENCY-CI-ONLY-001`
- `INV-TDRP-MICROTEX-UBUNTU-GTK-NO-PARALLEL-RUNTIME-001`
- `INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001`
- `CONTRACT-TDRP-MICROTEX-UBUNTU-GTK-PINNED-SOURCE-001`
- `CONTRACT-TDRP-MICROTEX-UBUNTU-GTK-MESON-CMAKE-BUILD-001`
- `CONTRACT-TDRP-MICROTEX-UBUNTU-GTK-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-MICROTEX-UBUNTU-GTK-SOURCE-DRIFT-001`
- `FAULT-TDRP-MICROTEX-UBUNTU-GTK-TOOLCHAIN-DRIFT-001`
- `FAULT-TDRP-MICROTEX-UBUNTU-GTK-MUTABLE-ACTION-DRIFT-001`
- `FAULT-TDRP-MICROTEX-UBUNTU-GTK-PRODUCT-PROMOTION-001`
- `FAULT-TDRP-MICROTEX-RESOURCE-LICENSE-DRIFT-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`

## Owner disposition

Selected owner: existing TDRP recursive source/dependency and CI/build-provenance governance.

Rejected for this workflow row: Fabushi shipping renderer/resource/platform/package owners, because this YAML only validates the third-party child build; a MicroTeXRuntime, TelegramRenderer or second rich-document root would incorrectly preserve dependency/upstream architecture. The actual formula responsibilities must be mapped existing-owner-first when their production source files are read.

Production entrypoints for this workflow row are empty. Separate upstream production consumers are retained only as negative-boundary evidence.

## Coverage accounting

Before this write: source entries 16,379; unknown 16,198; unread 16,173; omitted 0; responsibilities 196.

After recording this blob: source entries 16,379; unknown 16,197; unread 16,172; omitted 0; responsibilities 197. MicroTeX child recursive tree still has 278 blobs not individually registered by this strict179 row; accepted tdesktop recursive source closure remains open.

## Parent exact-head evidence

Parent HEAD `c71b1a215b44d64cbe70f1c969304760393f0783` is terminal-success:
- TDRP Revision 9 run `40094886294`: 61/61 jobs success; source-authority artifact `12560169433` sha256:f9be5aa8e4da987fbb13ac5d3fdf2671484bab4e649e2f943cc31aa606d96313; recursive closure `12561190021` sha256:da3d149b7ab113f3c1a6a45ba15a2b21432beb31f472a847812888100fd58719; recursive reachability `12561170179` sha256:bffcc93deda2ba144e9e957dc91920abd786f04cbb55b1bf808eb69ea6f5527e; Boost reachability `12560656756` sha256:db6faf5b4b3852b0c8fbb6751aefe645bc4b7f6e52cbc247d7fb92ce9df77594.
- Rust desktop runtime run `40094886220`: 6/6 success; artifact `12560736371` sha256:b0f11e52918df55cb365f42e480ff831d0383cb54a083140c6e1ec5bc4480920.
- Official MCP marketplace run `40094886217`: contracts success.

These are parent-head evidence. The strict179 receipt commit must obtain fresh exact-head workflow terminal results.

## Closure

- `baseline_identity_accepted=false`
- `source_closure_ready=false`
- independent release acceptance remains not accepted
- no production implementation or shipping-owner closure is claimed by this dependency CI/build workflow row.
