# tools/clangtidy README complete read — strict132

## Authority
- Upstream: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `tools/clangtidy/README.md@2e64a408cb2d76aed0425706e321a24bb58f350f`, fully read.
- Supporting anchors: `tools/clangtidy/compile_db.py@16a99ac4ad860cc37ee7c84bfb82cf899fb1cced`, `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`, `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434`, `tools/rules/test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f`.
- Parent Fabushi exact head: `22a4c56f2dd4ae8f5e3884cd5bb220275abe41a8`, tree `f3d840c1e74ce7d0529deb4e5befafaf0f739837`.
- TDRP `39984901447`: 61/61 success; authority `12514373943` / `sha256:4a5b71d5e6c8965797eef2bf63a0766da69aebc72f25238a872d2943fb32636c`; closure `12514514383` / `sha256:d26f54c0f5656cba152906a9be28b280ee1887eacca98337da4e03f92ddc3a16`; reachability `12516030885` / `sha256:f4017ca396bd83a4c9b6cdf6553f865ef11f54c418b87d5740a1c27db243df02`.
- Rust desktop runtime `39984901437`: 6/6 success; artifact `12515950645` / `sha256:1715b509fbad46f9bb3d96634ad3898b6f3a13d8caa9f5953922a6d0cc56fc53`.

## Responsibility decomposition
The README is developer static-analysis workflow provenance, not a shipping runtime. It requires deriving analyzer input from a real configured build via Ninja compdb, using clang-tidy 20+ so the locked CustomFunctions rules are not silently ignored, and optionally running diff/full-file/local pre-commit feedback.

The supporting `compile_db.py` adapts evaluated compiler commands for clang-tidy without inventing build truth: it parses command lines, substitutes clang-cl/clang++, removes only declared incompatible output/PCH/cache details, adds platform toolchain arguments, filters existing C++ translation units, and writes analyzer-only compile commands.

The documented pre-commit installation delegates to `tools/rules/hook.py --staged`; `test_hook.py` proves Telegram's local edit/staged policy. These local hooks are replaceable feedback adapters, never Fabushi acceptance authority.

## Existing-owner-first / applicability
Fabushi already has source-neutral owners in `TEST_STRATEGY.md@4d2c88e...` for source static-analysis migration and configured-build graph qualification, and `AGENTS.md@da3e3725...` for local/editor feedback adapters. Existing Rust/TypeScript GitHub Actions remain the executable authority.

C++/Qt/Ninja/clang-tidy runner mechanics, Telegram comment budgets and Qt helper spellings are provenance; no C++ runtime or second quality owner is introduced.

Exact-head inspection of `.github/workflows/rust-desktop-runtime.yml@905fc0e...`, `.github/workflows/desktop-chat-parity-ci.yml@2689fd2...` and `desktop/package.json@3d50062...` shows Cargo metadata/tests and TypeScript typecheck/build, but no dedicated `cargo clippy`, `cargo fmt`, ESLint or lint script. Thus target-language static-analysis coverage remains mapped-open.

## Fail-closed
Stale/synthetic configured-build input, unsupported analyzer capability, missing/disabled local hooks, and local analyzer success are never acceptance. Missing target-language dedicated static-analysis gates remain open. Recursive source closure and independent release acceptance remain open.

## Traceability
Requirement `TDRP-R9-CLANG-TIDY-WORKFLOW-001`; oracles `ORA-TDRP-CLANG-TIDY-README-SOURCE-001`, `ORA-TDRP-CLANG-TIDY-CONFIGURED-COMPDB-001`, `ORA-TDRP-CLANG-TIDY-MIN-VERSION-001`, `ORA-TDRP-CLANG-TIDY-LOCAL-HOOK-ADAPTER-001`, `ORA-TDRP-CLANG-TIDY-FABUSHI-GATE-GAP-001`.

Closing only strict132 changes counters to unknown=16,244, unread=16,219, omitted=0, responsibilities=150. `baseline_identity_accepted=false`, `source_closure_ready=false` and independent release acceptance remain mandatory.

Next deterministic entry: strict133 `tools/clangtidy/compile_db.py@16a99ac4ad860cc37ee7c84bfb82cf899fb1cced`.
