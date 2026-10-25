# desktop-app/MicroTeX res/SAMPLES.tex complete read — strict219

## Authority

- PR #154 lease / pre-receipt HEAD: `52e148efdbe7217f1d40982f6c6ff00ae3f204cc`.
- Live Fabushi main: `16aae12ea258f6a872a46aa47824766f64e1bdca`.
- Accepted/live tdesktop dev: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`.
- Pinned/live MicroTeX master: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`; recursive tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`, `truncated=false`, 315 entries.
- Target: `res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b`, mode `100644`, 9,933 bytes, 9,549 decoded UTF-8 characters, 262 line slots.
- Pre-write independent responsibility row count for `TDRP-R9-MICROTEX-RES-SAMPLES-TEX-PROVENANCE-001`: **0**.

## Complete target read

The pinned file is a source/demo sample corpus with exactly 11 pure-percent separator lines and 12 ordered non-empty sample blocks. Decoded block lengths are `360, 929, 1562, 2259, 433, 208, 228, 595, 283, 1080, 250, 217`.

## Exact-path history and creation boundary

Exact-path history contains 13 commits through the accepted pin; page 2 is empty. Creation is `dfa166a26fa3d16aabab1db078e873d959d65893`; parent `c389273d3e88b9d9577abcb0ecd02e817fb075cf` has a `truncated=false` 187-entry recursive tree without the target. Creation changes exactly six paths: `res/SAMPLES.tex`, `src/common.h`, `src/latex.cpp`, `src/latex.h`, `src/samples/gtkmm_main.cpp`, `src/samples/samples.h`.

## Parser and headless contract

`src/samples/samples.h@2e8bcd0d3d8e2461ad12bce4e9e4763124b6b652` defaults to `LaTeX::getResRootPath() + "/SAMPLES.tex"`, splits only non-space lines made entirely of `%`, discards blank/space-only lines, and UTF-8 converts non-empty accumulated blocks.

`src/samples/gtkmm_main.cpp@d491af61d05fbcc5d17c48f64e76664f91d5fce8` accepts `-samples=[FILE]`; headless batch requires `-outputdir`, fails on zero samples, and emits one indexed SVG per sample.

`githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04` calls `./LaTeX -headless` without `-samples`, regenerates `samples.md`, and publishes `sample_0.svg` through `sample_11.svg` with `scour`.

## Build/install and accepted shipping boundary

`meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5` installs `res/SAMPLES.tex` via `install_data`; `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` has no SAMPLES entry.

Accepted `desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334`, `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30`, `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277`, `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2`, and `iv_markdown_math_renderer.cpp@46864b7ce87207c6e57b6bcc7610337120580bea` each have zero `SAMPLES.tex` / `sample_` shipping references. Production uses compiled MicroTeX plus bundled qrc/font resources.

## Existing-owner-first audit

Live Fabushi owner inventory, architecture, owner-resolution matrix, quality contracts and desktop package composition were re-read. Selected owner: **existing TDRP recursive source/test-fixture/resource/build/provenance governance**. No MicroTeX sample runtime, second Resource/test-fixture store, second packaging root, or Telegram runtime is introduced.

## Negative oracles

- Standalone Meson installation does not imply accepted tdesktop shipping.
- Generated `samples.md` and `sample_0..11.svg` are documentation artifacts, not production runtime resources.
- Pure-percent delimiter semantics must not be widened.
- CLI help relative-path wording must not replace actual resource-root default behavior.
- A sample corpus does not prove formula-rendering parity or release readiness.
- `res/bundled.qrc` remains independently auditable.

## Coverage

Before strict219: total 16,379; unknown 16,158; unread 16,133; omitted 0; responsibilities 236.

After strict219: total 16,379; unknown **16,157**; unread **16,132**; omitted **0**; responsibilities **237**.

`baseline_identity_accepted=false`, `source_closure_ready=false`, independent release acceptance remains not accepted.

## Parent exact-head evidence

Lease `52e148efdbe7217f1d40982f6c6ff00ae3f204cc` is terminal-success: TDRP Revision 9 run `40166899536` **61/61 success**; authority `12588204365` `sha256:d09a1be68cd40bca26f18a6a518cf25fd7f822427bd503863e80965ca1361db1`; closure `12588049206` `sha256:258e1f226dac55af0fc663c7ba105998daeb3df5ecc46f314fe694d3b7bec814`; reachability `12588254549` `sha256:e43d8bbad8a0c695d7585ed7f3e7a607420bbbe22246300b63b0ae65354668a4`; Boost `12587994328` `sha256:eb1aedb81e48254bf7b2f85e9a6acb4be2ac294830702283d519f58f7ac571e1`. Rust desktop runtime run `40166899589` is **6/6 success**, artifact `12588293982` `sha256:edd3dddbb242af1ce500ef6e59a55f220536763bb2b65732a943763c96838a79`. Official MCP marketplace run `40166899582` is **1/1 success**.

These are historical parent evidence after this receipt commit; fresh receipt-head Actions are required.

## Traceability

Requirement: `TDRP-R9-MICROTEX-RES-SAMPLES-TEX-PROVENANCE-001`. Target symbol: `TDRP_MICROTEX_RES_SAMPLES_TEX_PROVENANCE_001`. Gates: `G-FILE`, `G-TRACEABILITY`, `G-EVIDENCE`.

Anchors: `ORA-TDRP-MICROTEX-RES-SAMPLES-TEX-EXACT-SOURCE-001`, `ORA-TDRP-MICROTEX-RES-SAMPLES-TEX-HISTORY-001`, `ORA-TDRP-MICROTEX-RES-SAMPLES-TEX-PARSER-001`, `ORA-TDRP-MICROTEX-RES-SAMPLES-TEX-REGENERATION-001`, `ORA-TDRP-MICROTEX-RES-SAMPLES-TEX-SHIPPING-NEGATIVE-001`, `INV-TDRP-MICROTEX-RES-SAMPLES-TEX-12-BLOCKS-001`, `INV-TDRP-MICROTEX-RES-SAMPLES-TEX-PURE-PERCENT-SPLIT-001`, `INV-TDRP-MICROTEX-RES-SAMPLES-TEX-NOT-SHIPPING-001`, `INV-TDRP-MICROTEX-RES-SAMPLES-TEX-NO-PARALLEL-OWNER-001`, `CONTRACT-TDRP-MICROTEX-RES-SAMPLES-TEX-PINNED-SOURCE-001`, `CONTRACT-TDRP-MICROTEX-RES-SAMPLES-TEX-PARSER-BOUNDARY-001`, `CONTRACT-TDRP-MICROTEX-RES-SAMPLES-TEX-SHIPPING-BOUNDARY-001`, `CONTRACT-TDRP-MICROTEX-RES-SAMPLES-TEX-EXACT-HEAD-EVIDENCE-001`, `FAULT-TDRP-MICROTEX-RES-SAMPLES-TEX-SOURCE-DRIFT-001`, `FAULT-TDRP-MICROTEX-RES-SAMPLES-TEX-HISTORY-MISATTRIBUTION-001`, `FAULT-TDRP-MICROTEX-RES-SAMPLES-TEX-SPLIT-SEMANTICS-DRIFT-001`, `FAULT-TDRP-MICROTEX-RES-SAMPLES-TEX-SHIPPING-CONFLATION-001`, `FAULT-TDRP-MICROTEX-RES-SAMPLES-TEX-RUNTIME-PROMOTION-001`.
