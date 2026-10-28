# desktop-app/MicroTeX githooks/pre-push complete read — strict187

## Authority
- Parent PR #154 / branch exact HEAD before receipt: `7c599eef9c4609540014fafd976a04afa39be17b`, tree `7f2434ead3247ea8bc05bd176bd13e29f03eb9dc`.
- Live Fabushi main: `16aae12ea258f6a872a46aa47824766f64e1bdca`, tree `82126ec4a712fca352a49d6a3caae70c7d21ca8c`.
- Accepted tdesktop remains `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX remains `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`; current `master` equals the pin; recursive tree truncated=false, 315 entries / 279 blobs.
- Pinned cmake_helpers remains `7ceca3019b4750d7a5acb2de0faffd523ee14095`, tree `f5c386cb34208309aae521187611fac4a2045577`.
- Target: `githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04`, tracked mode `100644`, 104 line slots / 2630 characters.
- Pre-write parity-ledger independent registration count is zero. The exact target appears once only as sibling dependency context in strict186 pre-commit provenance; that context reference is not an independent source registration.

## Complete source behavior
The Bash file labels itself “Update samples before push”. If an external Git-hook installation dispatches it, it prints an UPDATE SAMPLES banner, captures the current directory, changes into `build`, redirects stdin from `/dev/tty`, and repeatedly prompts `Do you wish to update samples (Y/N)?`. A response beginning with `Y` or `y` runs both sample regeneration and change handling; every other response cancels. It then closes stdin, returns with `cd $workingDir`, and prints DONE.

`update_samples` deletes and recreates build-local `samples/`, then runs `./LaTeX -headless -outputdir=./samples -prefix=sample_ -textsize=14 -foreground=black -background=white`. The pinned Unix/GTK sample executable `src/samples/gtkmm_main.cpp@d491af61d05fbcc5d17c48f64e76664f91d5fce8` routes `-headless` to batch mode; `src/samples/samples.h@2e8bcd0d3d8e2461ad12bce4e9e4763124b6b652` resolves an omitted sample-file argument to `LaTeX::getResRootPath() + "/SAMPLES.tex"`. Root `CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9` copies `res` into the build tree, so this invocation reads the pinned `res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b` through the build-local resource tree.

The hook separately rebuilds build-local `samples.md`: lines consisting only of `%` delimit samples; non-empty source lines are buffered in `__tmp`; each sample becomes a fenced `tex` block plus `![sample_N](samples/sample_N.svg)`, and the final sample is explicitly flushed. The current pinned documentation set is `readme/samples.md@4eb0be8a26e15f098156040366683f4e835631f6` plus 12 tracked SVGs `sample_0.svg` through `sample_11.svg`.

`update_change` ensures `../readme/samples` exists, loops over generated SVGs, and invokes external `scour` to write compressed SVGs into the tracked documentation directory with viewbox/id/comment/short-ID/indent options. It copies `samples.md` to `../readme`, stages `../readme/samples.md` and `../readme/samples`, checks staged diffs for the directory and Markdown separately, and when either differs executes `git commit -m "update samples"`; otherwise it reports no change.

## Direct consumer and installation boundary
- The direct semantic dispatcher is Git's pre-push hook mechanism after external installation/configuration. The tracked file itself is mode `100644`, so repository presence alone does not activate it as an executable hook.
- Because MicroTeX `master` currently equals the exact pin, default-branch code-search observations are pin-equivalent. Searches for `githooks`, `pre-push`, `core.hooksPath`, `UPDATE SAMPLES`, and the exact prompt produced no tracked installer/consumer reference. The sibling pre-commit dossier had already observed the same external-installation boundary.
- `README.md@824af04fd8607532efed3f73752b81e5093577ef` documents the standalone `build/LaTeX` build and headless batch options, but not hook installation.
- The direct called batch implementation is dependency demo/tooling code (`gtkmm_main.cpp` + `samples.h`), and its outputs are documentation/demo assets. None of these sources is a Fabushi shipping product owner.

## Failure and negative oracles
- There is no `set -e`, status propagation, trap, or explicit failure exit. Failure of `cd build`, `./LaTeX`, directory/file operations, `scour`, `cp`, `git add`, `git diff`, `git commit`, or the return `cd` can be followed by later successful commands; the final successful banner can therefore mask earlier failure.
- `/dev/tty` is assumed. If no controlling terminal exists, the redirection itself is not fail-closed; the script has no explicit non-interactive failure branch.
- The unquoted `cd $workingDir`, backtick command substitutions, and `for svg in \`ls samples\`` make whitespace/special-character handling fragile.
- `git add` mutates the developer's real index and `git commit -m "update samples"` mutates local history during pre-push. The hook never parses or reconciles the ref/object records supplied to pre-push and never proves that a commit created during the hook is the object transferred by the already-started push; it cannot serve as current-push closure evidence.
- The tracked destination `../readme/samples` is not cleared before newly compressed files are copied. If the generated set shrinks or generation/compression is partial, stale tracked SVGs can remain unless another operation removes them.
- A failed `git add` can leave the subsequent cached-diff checks observing old index state; the source does not distinguish that from a clean result.
- The hook runs no test suite, packaged acceptance, source-authority proof, artifact-provenance check, or release acceptance. Local sample regeneration cannot substitute for exact-head GitHub Actions evidence.
- No `PrePushGate`, `MicroTeXSampleRuntime`, second CI/release owner, second formula renderer, or second application shell is introduced.

## Accepted tdesktop shipping separation
Accepted tdesktop does not install or invoke this developer hook in shipping composition. `desktop-app/cmake_helpers:external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334` directly compiles the pinned fork, Qt backend and bundled resources into `external_microtex_bundled`. `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` links `desktop-app::external_microtex`; `Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` owns bundled-resource initialization, input/image limits, exception containment and QImage rendering; `iv_markdown_math_renderer.cpp@46864b7ce87207c6e57b6bcc7610337120580bea` owns normalized render/cache/fallback behavior. `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` includes `td_iv`, links it into Telegram, and copies `external_microtex_bundled.rcc` into the macOS application resources. Those shipping production responsibilities remain independently open.

## License
`LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2` is MIT; the README preserves separate font/resource license notices. The hook and generated sample documents are retained only as dependency source/developer-tooling/documentation provenance.

## Coverage
Before: source_entries=16379 unknown=16190 unread=16165 omitted=0 responsibilities=204.
After: source_entries=16379 unknown=16189 unread=16164 omitted=0 responsibilities=205.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted; shipping MicroTeX responsibilities remain open.

## Parent exact-head evidence
Parent `7c599eef9c4609540014fafd976a04afa39be17b`: TDRP Revision 9 run 40110594679 terminal success 61/61; authority artifact 12565928211 sha256:2479f5a5b73a0af2ec31c8af7052ae5e7bf9dbbe337d031897eef63d2f27d159; recursive closure artifact 12566875621 sha256:1d951bb5192d6a6e5adb44d8a99333d71312f578486e73fb7e3be6740388a1d2; recursive reachability artifact 12566821622 sha256:b74b9a8600e564741cf630c263ec6abc93d7ad46cd45d12f0a31e7afb21a93dc; Boost reachability artifact 12566318293 sha256:c9e9663af4287287d7a3c787469203d439a67c9ab37b68ed1f4b7f9785819fe6. Rust desktop runtime run 40110594755 terminal success 6/6; artifact 12566775545 sha256:880b055579223452358adcc8f04a86e8a30c7e4dc339193046650c9e6c3c470d. Official MCP marketplace run 40110594600 terminal success 1/1. Receipt-head Actions are required.

## Traceability IDs
- CAP-DEPENDENCY-SOURCE-PROVENANCE
- CAP-DEPENDENCY-DEVELOPER-HOOK-PROVENANCE
- CAP-DEPENDENCY-SAMPLE-REGENERATION-PROVENANCE
- CAP-DEPENDENCY-DOCUMENTATION-ASSET-PROVENANCE
- TDRP-R9-MICROTEX-GITHOOKS-PREPUSH-PROVENANCE-001
- TDRP_MICROTEX_GITHOOKS_PREPUSH_PROVENANCE_001
- ORA-TDRP-MICROTEX-PREPUSH-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-PREPUSH-INTERACTIVE-BRANCH-001
- ORA-TDRP-MICROTEX-PREPUSH-SAMPLE-REGENERATION-001
- ORA-TDRP-MICROTEX-PREPUSH-INSTALLATION-BOUNDARY-001
- ORA-TDRP-MICROTEX-PREPUSH-FAILURE-PROPAGATION-001
- ORA-TDRP-MICROTEX-PREPUSH-SHIPPING-SEPARATION-001
- INV-TDRP-MICROTEX-PREPUSH-NO-PARALLEL-RELEASE-GATE-001
- INV-TDRP-MICROTEX-PREPUSH-NOT-FAIL-CLOSED-001
- INV-TDRP-MICROTEX-PREPUSH-DOC-ASSETS-NOT-SHIPPING-RUNTIME-001
- INV-TDRP-MICROTEX-PREPUSH-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-PREPUSH-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-PREPUSH-DEVELOPER-HOOK-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-PREPUSH-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-PREPUSH-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-PREPUSH-NONINTERACTIVE-001
- FAULT-TDRP-MICROTEX-PREPUSH-PARTIAL-SAMPLE-UPDATE-001
- FAULT-TDRP-MICROTEX-PREPUSH-HIDDEN-PUSH-DRIFT-001
- FAULT-TDRP-MICROTEX-PREPUSH-PRODUCT-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
