# Round 167 — fixed-U root clang-tidy static-analysis policy

- Task round: 136
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `52591d7976c5ec113ff9bcb8089a739e767c23a7`
- Spec-first desktop commit: `43415c040400f2c36daf33cf43b34ab7b45b0a3b`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs
- Closed source: `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`
- Source identity: recursive-tree index 84 zero-based; 1,264 bytes / characters; 38 newline characters with trailing newline; 39 split lines

The exact root static-analysis policy was read completely. It enables only `bugprone-unsafe-functions`, promotes it to error, defines the Qt-helper-to-standard/safe `CustomFunctions` mapping, and deliberately leaves `HeaderFilterRegex` empty. The mapping includes the `qBound` argument-order caveat and safe `qRound` replacement.

Direct fixed-U consumer `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434` parses this exact file to reject the same helper calls at edit/staged time. `tools/clangtidy/README.md@2e64a408cb2d76aed0425706e321a24bb58f350f` plus `compile_db.py@16a99ac4ad860cc37ee7c84bfb82cf899fb1cced` define the clang-tidy runner/compile-database path and clang-tidy 20+ prerequisite. Claude settings/cross-review are secondary hook callers; `test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f` is an unexecuted regression consumer.

Existing-owner-first adds `TDRP-R9-REPOSITORY-STATIC-ANALYSIS-POLICY-001` under the existing repository quality/test-evidence owner. No second scanner/findings/task owner, shipping runtime or platform-core delta is introduced.

Authored/not-run contracts: `CONTRACT-TDRP-STATIC-ANALYSIS-CONFIG-SCOPE-001`, `CONTRACT-TDRP-STATIC-ANALYSIS-CUSTOM-FUNCTIONS-001`, `CONTRACT-TDRP-STATIC-ANALYSIS-HOOK-PARITY-001`, `CONTRACT-TDRP-STATIC-ANALYSIS-TOOLCHAIN-VERSION-001`, `INT-TDRP-STATIC-ANALYSIS-QUALITY-COMPOSITION-001`, and `FAULT-TDRP-STATIC-ANALYSIS-CONFIG-DRIFT-001`.

Unique-ledger closure: `source_entries_total=16202 / unknown=16141 / unread=16114 / omitted=0 / responsibilities_total=137 / unmapped_responsibilities=0`. Implementation is `implemented`, validation remains `not-run`, accepted remains false.

Next unrepresented fixed-U blob: `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger and no clang-tidy, hook/test execution, configure/build, compile, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deploy, install, runtime evidence or acceptance was performed.
