# desktop-app/MicroTeX res/.clatexmath-res_root complete read — strict217

## Authority

- PR #154 branch: `tdrp-r9-entry31-includes-r27-20261020`.
- Lease / pre-receipt HEAD: `b745d625d41eeccdfc3404bf07947b9fdefd838f`; tree `ed4e78c46e56047ae1c02285de4d0cebe9d47b40`.
- Live Fabushi main: `16aae12ea258f6a872a46aa47824766f64e1bdca`; tree `82126ec4a712fca352a49d6a3caae70c7d21ca8c`.
- Accepted/live tdesktop dev: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`; tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned/live MicroTeX master: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`; tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`.
- Fixed MicroTeX tree: `truncated=false`, 315 entries / 279 blobs.
- Pre-write independent responsibility row count for `TDRP-R9-MICROTEX-RES-CLATEXMATH-RES-ROOT-PROVENANCE-001`: **0**. The only pre-write ledger mention is strict216's next pointer.
- Target: `res/.clatexmath-res_root@e69de29bb2d1d6434b8b29ae775ad8c2e48c5391`, mode `100644`, **0 bytes**.
- Previous fixed-tree blob: `readme/tex_logo.svg@33647d95ba36e42839ac5a2469464c8cde2b6c99`.
- Next fixed-tree blob: `res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee`.

## Complete target read

The target is the canonical Git empty-file blob: zero bytes, no text, metadata payload, executable content, or external surface. Its responsibility is presence at the resource root, not embedded configuration.

## Exact-path history and creation boundary

The accepted-pin exact-path query returns exactly one commit on page 1 and an empty page 2:

- `3bdd94b9b7263e1a30325d75aafe275a2f00fc45`, “initial support of resource detection”, tree `b76eaa9bb00c5d418ca85c71b5b90693e3cce9e4`, parent `cd291c031d5b7c3b20fa739501dc36ff53665e65`.

The creation parent recursive tree is `truncated=false` with 257 entries and has no marker. Creation changes exactly five paths:

- `meson.build`
- `meson_options.txt`
- `res/.clatexmath-res_root`
- `src/latex.cpp`
- `src/latex.h`

Thus marker creation is bounded to the same resource-detection change that introduced `LaTeX::queryResourceLocation`.

## Accepted discovery and fallback contract

Pinned `src/latex.cpp@9ed2d29525b057943da537e8d2dd9e3afedc45b2` defines `CHECK_FILE=".clatexmath-res_root"` and keeps `RES_BASE="res"` as the default.

`LaTeX::queryResourceLocation` probes, in order:

1. caller custom path;
2. `CLM_DEVEL`;
3. `XDG_DATA_HOME + "/clatexmath/"`;
4. each `XDG_DATA_DIRS` entry plus `/clatexmath/` (`;` on Windows, `:` elsewhere);
5. non-Windows `$HOME/.local/share/clatexmath/`;
6. non-Windows `/usr/share/clatexmath/`;
7. non-Windows `/usr/local/share/clatexmath/`.

A candidate is accepted only when the marker exists. No hit returns an empty path. `LaTeX::init` catches discovery failures and replaces `RES_BASE` only for a non-empty result; otherwise the default `res` base remains and initialization continues. This is fail-soft discovery, not a mandatory startup gate.

## Repair history

- `d6995921e57b13a4a5f8d145450736619e1de6df`: removes invalid deletion/free of an environment pointer.
- `4ba3bc6ef33fc0d67305211abe0affd49067e398`: extends XDG sensing so MinGW can discover resources.
- `6fbdd5c4a8ca730d30067645cc3fc6a337e0699d`: adds `CLM_DEVEL`, platform-specific XDG separators, and `/clatexmath/` suffix construction.

These are part of the resource-location responsibility: environment ownership, platform path semantics, and resilient discovery.

## Meson install-path repair

- `b56c22413413970c27308cbb8b558d8319bc25d8` first adds the marker to `install_data` using the wrong source path.
- `0e3db008b7cc9a88fa1b14a67649777c025d1699` corrects it to `res/.clatexmath-res_root`.
- Accepted `meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5` installs `RES_README`, `SAMPLES.tex`, and the corrected marker alongside the standalone resource data.

This is standalone Meson install/discovery provenance, not accepted tdesktop package membership.

## Accepted tdesktop bundled-shipping bypass

Accepted tdesktop does not use marker discovery:

- `desktop-app/cmake_helpers:external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334` builds `external_microtex_bundled`, selects `BUILD_QT`, compiles the Qt backend, attaches `res/bundled.qrc`, and runs `target_prepare_qrc`.
- `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` links `desktop-app::external_microtex`.
- `Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` registers bundled resources and calls `tex::LaTeX::initBundled()`.
- `initBundled()` bypasses `queryResourceLocation`; bundled fonts resolve through Qt `:/res/...` resources.
- `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` and `Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` carry `external_microtex_bundled.rcc` in macOS Resources.

Therefore this marker is not a current tdesktop runtime requirement and must not be promoted into a Fabushi resource locator.

## Existing-owner-first audit

Live Fabushi authority was re-read before disposition:

- `current-owner-inventory.md@d252cac8d8d6f8cb1a5b485788fef47a5866f9be`: attachments/resource ingress remains the existing Composer + Electron attachment manager + Host attachment extension; artifacts remain existing transcript/tool/artifact surfaces; durable state/recovery remains existing persistence/Host storage.
- `owner-resolution-matrix.md@7f822ccc57000501c47f1771499a68f93aa35aa8`: app/platform lifecycle remains Electron main/startup/window/update.
- `architecture-map.md@77a314c1c982da3aa85bb65165d80468b2663db3`: files/media remain attachments/artifacts/resource lifecycle and platform concerns remain existing lifecycle owners.
- Concrete exact-head owners include `source/electron-main/attachments/attachment-manager.ts@0d055b7d65c49911deb18ff08af2caa1639d93f7`, `source/host/src/attachment_paths.rs@55a816100d698bbccf69a38bd226cf2417cff4c9`, `frontend/src/production/runtime-assets.ts@470eb11f3d7f22b959008755e639a96a7f71aba0`, `source/electron-main/main.ts@b8eeed322ec07e5c8f67fa367cd2f58ac212b4f0`, `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84`, and `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643`.

Selected owner: **existing TDRP recursive source/build/resource/packaging provenance governance**. The product-runtime owners above remain reuse context. No `MicroTeXResourceLocator`, Telegram resource runtime, second Resource store, second platform lifecycle, second packaging root, or second CI/release gate is created.

## Negative oracles

- Zero-byte marker presence is not runtime configuration content.
- Marker absence is not fatal initialization: standalone init keeps the default base on no hit.
- Standalone Meson install is not accepted tdesktop shipping.
- Accepted tdesktop does not run this custom/XDG/HOME/system probe; it calls `initBundled()`.
- Fabushi does not need a new source-specific resource runtime.
- This source read does not close formula rendering, font loading, bundled packaging, or release acceptance.

## Coverage

Before strict217: total 16,379; unknown 16,160; unread 16,135; omitted 0; responsibilities 234.

After strict217: total 16,379; unknown **16,159**; unread **16,134**; omitted **0**; responsibilities **235**.

`baseline_identity_accepted=false` and `source_closure_ready=false` remain fail-closed. Independent release acceptance remains not accepted.

## Parent exact-head evidence

Lease `b745d625d41eeccdfc3404bf07947b9fdefd838f` was revalidated:

- TDRP Revision 9 source authority run `40163570117`: **61/61 success**.
  - authority `12586794955` — `sha256:11c8a9a50ed096c87c66a67e6f73ce97537ce82e0a7f9f1817131f7f1b41e9c8`
  - closure `12587770458` — `sha256:9e2a7d72cd079eb9a3d863631d0370b7fc42fe218f0827f249163bd1b7965098`
  - reachability `12587293320` — `sha256:e3d5070795dfe8bc0c99546d3c879845feaf4382816394cfd928a8bdbfff6b93`
  - Boost `12587810346` — `sha256:202bc01fb14468fa5efd3a14dbf97fc0bc0790801feadccd3d7c7ef79c9f9056`
- Rust desktop runtime run `40163570116`: **6/6 success**; artifact `12586694904` — `sha256:7d7e4193a598830252f21bbe98d3a7e96b56891832fb21ed87f2e1c1c42e1bc9`.
- Official MCP marketplace run `40163570243`: **1/1 success**.

These are historical-only parent evidence after the receipt commit; receipt-head Actions must be fresh.

## Traceability

- Requirement: `TDRP-R9-MICROTEX-RES-CLATEXMATH-RES-ROOT-PROVENANCE-001`
- Target symbol: `TDRP_MICROTEX_RES_CLATEXMATH_RES_ROOT_PROVENANCE_001`
- Gates: `G-FILE`, `G-TRACEABILITY`, `G-EVIDENCE`

Anchors:
- `ORA-TDRP-MICROTEX-RES-ROOT-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-RES-ROOT-HISTORY-001`
- `ORA-TDRP-MICROTEX-RES-ROOT-DISCOVERY-ORDER-001`
- `ORA-TDRP-MICROTEX-RES-ROOT-EMPTY-FALLBACK-001`
- `ORA-TDRP-MICROTEX-RES-ROOT-PLATFORM-REPAIRS-001`
- `ORA-TDRP-MICROTEX-RES-ROOT-MESON-INSTALL-REPAIR-001`
- `ORA-TDRP-MICROTEX-RES-ROOT-TDESKTOP-BUNDLED-BYPASS-001`
- `INV-TDRP-MICROTEX-RES-ROOT-ZERO-BYTE-SENTINEL-001`
- `INV-TDRP-MICROTEX-RES-ROOT-NOT-TDESKTOP-RUNTIME-001`
- `INV-TDRP-MICROTEX-RES-ROOT-NO-PARALLEL-RESOURCE-OWNER-001`
- `INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001`
- `CONTRACT-TDRP-MICROTEX-RES-ROOT-PINNED-SOURCE-001`
- `CONTRACT-TDRP-MICROTEX-RES-ROOT-DISCOVERY-FALLBACK-001`
- `CONTRACT-TDRP-MICROTEX-RES-ROOT-SHIPPING-BOUNDARY-001`
- `CONTRACT-TDRP-MICROTEX-RES-ROOT-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-MICROTEX-RES-ROOT-SOURCE-DRIFT-001`
- `FAULT-TDRP-MICROTEX-RES-ROOT-HISTORY-MISATTRIBUTION-001`
- `FAULT-TDRP-MICROTEX-RES-ROOT-PATH-SENSING-CONFLATION-001`
- `FAULT-TDRP-MICROTEX-RES-ROOT-MESON-PATH-REGRESSION-001`
- `FAULT-TDRP-MICROTEX-RES-ROOT-SHIPPING-PROMOTION-001`
