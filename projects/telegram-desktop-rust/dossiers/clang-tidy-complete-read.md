# clang-tidy complete read — strict index 57

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Strict zero-based root index: `57`.
- Source: `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`, 39 lines, fully read.
- Parent exact-head source-authority evidence: Fabushi `1be25d01d4c235ec38461db748cb600e155e58eb`, TDRP run `39699476395`, 61/61 jobs success.
- Authority artifact: `12392176925`, digest `sha256:f95456a2bb611fa17f32f748567aefd069565a6b904c53e0881b1cac92c34307`.
- Recursive reachability artifact: `12391778924`, digest `sha256:7ac645a3f3dce25489ccbd09e4644f0f64f1a44c4d8f855fe8cdfe0e54c843a6`.
- Root source archive artifact: `12392866281`, digest `sha256:a28439ff218472bbc03f9578ec4b829f5c6c1fc347efb03c0355137bc1657c92`.
- Fresh child exact-head Actions remain mandatory after this mapping commit.

## Complete responsibility decomposition

The file disables the default clang-tidy set and enables only `bugprone-unsafe-functions`; that check is also promoted to warning-as-error. The configuration therefore owns a narrow source-quality migration rule rather than a general C++ analyzer policy.

`bugprone-unsafe-functions.CustomFunctions` records exact replacements for Qt helper APIs: min/max/abs/bound/swap/pair/exchange/as-const, trigonometric/log/exp/pow/hypot functions, finite/NaN/infinity predicates, population/leading/trailing-zero bit counts, and ceil/floor/round conversions. `qBound` explicitly records the standard `std::clamp` argument-order difference; qCeil/qFloor require integer conversion and qRound maps to `base::SafeRound`, so these are semantic replacement contracts rather than blind spelling substitutions.

`HeaderFilterRegex: ''` suppresses repeated header findings and avoids treating submodule-owned source as this repository's defect surface. That scope boundary is part of the responsibility and cannot be interpreted as proof that headers or third-party source are safe.

## Existing-owner-first

Selected owner: Fabushi repository quality/test-evidence governance in `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`.

Rejected:
- a new C++ or Telegram static-analysis runtime, because Fabushi's target architecture does not retain upstream C++ as a product owner;
- shipping Resource/Conversation/Task/Agent/Host/Runner owners, because this is developer source-quality configuration;
- copying the Qt helper list into production code as a dependency contract, because the list is migration guidance and must be re-expressed in the target language/toolchain where applicable.

## Platform/language adaptation

The upstream rules are C++/Qt-specific. Fabushi preserves the applicable intent—unsafe helper replacement, exact semantic substitution, typed edge cases, fail-on-warning behavior, and owned-source filtering—inside the existing Rust/TypeScript/platform quality lifecycle. No C++ runtime or Qt dependency is introduced merely to preserve the checker. Executable verification is GitHub-Actions-only.

## Traceability anchors
- `TDRP-R9-CLANG-TIDY-SAFETY-MIGRATION-001`
- `ORA-TDRP-CLANG-TIDY-EXACT-CHECK-SET-001`
- `ORA-TDRP-CLANG-TIDY-SEMANTIC-REPLACEMENT-001`
- `ORA-TDRP-CLANG-TIDY-SCOPE-BOUNDARY-001`
- `INV-TDRP-CLANG-TIDY-NO-CXX-RUNTIME-001`
- `INV-TDRP-CLANG-TIDY-EXACT-SEMANTICS-001`

Closing only strict index 57 changes counters to unknown=16,246, unread=16,221, omitted=0, responsibilities=75. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root entry: index 58 `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`.
