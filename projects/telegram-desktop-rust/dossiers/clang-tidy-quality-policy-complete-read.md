# Fixed-U complete read — root .clang-tidy repository static-analysis policy

- Task round: 136
- Date: 2026-11-01
- Desktop checkpoint parent: `52591d7976c5ec113ff9bcb8089a739e767c23a7`
- Desktop main observed before work: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core authority: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries / 6,859 blobs
- Source: `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`
- Recursive-tree index: 84 zero-based
- Source size: 1,264 bytes; ASCII/UTF-8; trailing newline

## Complete policy semantics

The root configuration is intentionally narrow. `Checks` disables every check and enables only `bugprone-unsafe-functions`; `WarningsAsErrors` promotes that same check to an error. The comment says the list grows one check at a time, so breadth is deliberate policy rather than an incomplete wildcard configuration.

The single `CustomFunctions` table bans Qt helpers only where the source declares a standard/safe replacement: `qMin -> std::min`, `qMax -> std::max`, `qAbs -> std::abs`, `qBound -> std::clamp` with the explicit low/value/high order warning, `qSwap -> std::swap`, `qMakePair -> std::make_pair`, `qExchange -> std::exchange`, `qAsConst -> std::as_const`, `qFabs -> std::fabs`, `qSin -> std::sin`, `qCos -> std::cos`, `qTan -> std::tan`, `qAcos -> std::acos`, `qAsin -> std::asin`, `qAtan -> std::atan`, `qAtan2 -> std::atan2`, `qSqrt -> std::sqrt`, `qLn -> std::log`, `qExp -> std::exp`, `qPow -> std::pow`, `qHypot -> std::hypot`, `qIsNaN -> std::isnan`, `qIsInf -> std::isinf`, `qIsFinite -> std::isfinite`, `qPopulationCount -> std::popcount`, `qCountLeadingZeroBits -> std::countl_zero`, `qCountTrailingZeroBits -> std::countr_zero`, `qCeil -> int(std::ceil(...))`, `qFloor -> int(std::floor(...))`, and `qRound -> int(base::SafeRound(...))`.

`HeaderFilterRegex` is explicitly empty. The source comment records the reason: header findings repeat per translation unit, while submodules are not repository-owned code for this rule.

## Direct consumers and callers

- `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434` is the direct parser. Its `banned()` function opens `ROOT/.clang-tidy` and extracts each `^::name$,replacement,` pair. Both edit-payload and staged-diff paths reject new calls to those names in C/C++/ObjC++ source and print the configured replacement. This makes the hook a consumer/mirror of the canonical list, not an independent policy store.
- `tools/clangtidy/README.md@2e64a408cb2d76aed0425706e321a24bb58f350f` documents the real clang-tidy consumption path: generate the compile DB after configure, then use LLVM's `clang-tidy-diff.py` or `run-clang-tidy`. It requires clang-tidy 20+ because older versions silently ignore `CustomFunctions`.
- `tools/clangtidy/compile_db.py@16a99ac4ad860cc37ee7c84bfb82cf899fb1cced` prepares `out/clang-tidy/compile_commands.json`, substitutes clang/clang-cl for the configured compiler, strips incompatible GCC/MSVC/PCH arguments, and adds platform-specific arguments. It does not copy the rule list; the runner discovers the root config.
- `.claude/settings.json@d47520dc0e3b7e8d6067456e33062140c75f7c79` invokes `tools/rules/hook.py` after Edit/Write/MultiEdit. `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361` invokes the staged hook during its review loop. These are secondary hook callers, not direct `.clang-tidy` authorities.
- `tools/rules/test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f` contains regression coverage for banned-function replacement messages, identifier/comment/string false positives, source-path filtering, staged line reporting, and comment-rule behavior. It is test evidence only and is not executed in Stage A.

## Existing-owner-first disposition

Responsibility `TDRP-R9-REPOSITORY-STATIC-ANALYSIS-POLICY-001` belongs to the existing Fabushi repository quality/test-evidence governance. The existing repository quality-health owner already governs exact-source observations, stable evidence identity, change-time ratchets, task routing boundaries, and Stage-B qualification. This source adds a narrow static-analysis policy contract; it does not justify another scanner, findings ledger, task queue, Agent/Coordinator/Host/Runner owner, product runtime, Telegram-specific quality subsystem, or platform-core production change.

## Authored/not-run contracts

- `CONTRACT-TDRP-STATIC-ANALYSIS-CONFIG-SCOPE-001`
- `CONTRACT-TDRP-STATIC-ANALYSIS-CUSTOM-FUNCTIONS-001`
- `CONTRACT-TDRP-STATIC-ANALYSIS-HOOK-PARITY-001`
- `CONTRACT-TDRP-STATIC-ANALYSIS-TOOLCHAIN-VERSION-001`
- `INT-TDRP-STATIC-ANALYSIS-QUALITY-COMPOSITION-001`
- `FAULT-TDRP-STATIC-ANALYSIS-CONFIG-DRIFT-001`

No clang-tidy, hook test, lint, configure, build, validator, reviewer, CI, runtime evidence, or acceptance was executed.
