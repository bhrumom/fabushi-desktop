# tools/clangtidy compile_db.py complete read — strict133

## Authority
- Upstream: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `tools/clangtidy/compile_db.py@16a99ac4ad860cc37ee7c84bfb82cf899fb1cced`, fully read.
- Parent Fabushi exact head: `fdf9715e8a08c808b66523a9f68a70b0a4485a4a`, tree `92e0a6fb2dff89a1b08781ec07c039978f6df048`. This parent is the traceability-only repair descendant of strict132; no production code changed in that repair.
- TDRP `39988917272`: 61/61 success; authority `12517051705` / `sha256:f9ade467c5a1226702daab6d9616f0bf23ba5be17408c01054ff9084b7969c17`; closure `12516254999` / `sha256:12d62689fd596f24365acfc3c151572667586b79d336d43bcfd51aed65483bd1`; reachability `12516364938` / `sha256:fb02f4c3696bd4befc29218328b763da2a9322f322262f1a8632ea5deb20790a`.
- Rust desktop runtime `39988917401`: 6/6 success; artifact `12516288454` / `sha256:c68b9e6aef92233641fba5a5f49574478c2d9d74c9e05036553c622edc1c1a47`.
- Live main at submission lease check: `7394f6dc7687f9b986ee5589d40aa680ce23b90a`. Live upstream dev remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Configured-build provenance and command adaptation
`main()` requires a real `build.ninja` and obtains entries from `ninja -C <build> -t compdb` with `check=True`; it does not synthesize compiler commands from source names. It keeps only existing C++ translation units with non-empty commands, then emits analyzer-only `<build>/clang-tidy/compile_commands.json`.

`split_command()` preserves platform parsing semantics: POSIX uses `shlex.split`, while Windows delegates to `CommandLineToArgvW` and frees the returned allocation. `compiler_for()` substitutes clang-tidy's sibling `clang-cl` for MSVC and `clang++` otherwise. `rewrite()` removes ccache/sccache wrappers, clang PCH injection, `-fhardened`, and declared MSVC PCH/output/noise flags while intentionally preserving forced include semantics such as `/FI`.

`platform_arguments()` adds the macOS SDK/libc++ view, the Windows narrowing compatibility switch, or a GCC toolchain prefix when the configured driver needs one. Missing macOS SDK extras fall back to no extra arguments; an unresolved GCC toolchain prefix warns and falls back. Empty commands and missing source files are skipped. Those behaviors are analyzer-input limitations, not proof that the configured build or source closure is complete.

The script chooses the most common configured driver only to derive shared platform analyzer arguments and writes `arguments` arrays per retained entry. Its output is a transformed static-analysis database and must not be promoted to target membership, artifact, runtime, or release truth.

## Existing-owner-first / applicability
The applicable responsibility is source-neutral: static analysis must be grounded in evaluated configured-build inputs, command adaptation must preserve declared compiler semantics, platform/parser limitations must remain visible, and analyzer evidence must stay weaker than configured-build/release evidence.

Fabushi already owns this boundary in `TEST_STRATEGY.md@4d2c88e118ab489d3d0339d2a61371a7ea74e72e` and `evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`: configured-build qualification explicitly covers compilation-database parsing, command-line definition ordering and PCH coherence; the evidence contract states configured-graph acquisition is read-only/fail-closed and that generic compilation databases do not establish configured target membership.

C++ suffixes, Ninja command syntax, clang-cl/clang++, MSVC PCH/output flags, `-fhardened`, Homebrew LLVM lookup and macOS SDK spelling are upstream toolchain provenance. They do not justify a Telegram/clang-tidy runtime, second build graph, or second quality owner in Fabushi.

Exact-head `.github/workflows/rust-desktop-runtime.yml@905fc0e17e5b4e54cb18708c55d1eb8725d18f83`, `.github/workflows/desktop-chat-parity-ci.yml@2689fd2ed01c6818586f251fb8a879a8b162c61b` and `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84` still provide Rust build/test and TypeScript typecheck/build but no dedicated cargo clippy/fmt or ESLint/lint script. The target-language static-analysis execution gap therefore remains mapped-open.

## Fail-closed
Upstream explicitly fails when `build.ninja` or clang-tidy is unavailable and propagates `ninja -t compdb` failure. Its skip/fallback paths for absent files, empty commands or platform extras are not complete-coverage evidence. In Fabushi, stale/synthetic command input, generic compilation databases without configured-graph identity, unsupported command semantics, missing dedicated target-language analyzer gates, or local analyzer success cannot satisfy migration/release acceptance. Recursive source closure and independent release acceptance remain open.

## Traceability
Requirement `TDRP-R9-CLANG-TIDY-COMPDB-001`.

Oracles: `ORA-TDRP-CLANG-TIDY-COMPDB-SOURCE-001`, `ORA-TDRP-CLANG-TIDY-NINJA-COMPDB-001`, `ORA-TDRP-CLANG-TIDY-COMMAND-REWRITE-001`, `ORA-TDRP-CLANG-TIDY-PLATFORM-ADAPTATION-001`, `ORA-TDRP-CLANG-TIDY-COMPDB-OWNER-001`, `ORA-TDRP-CLANG-TIDY-COMPDB-GATE-GAP-001`.

Invariants: `INV-TDRP-CLANG-TIDY-COMPDB-CONFIGURED-SOURCE-001`, `INV-TDRP-CLANG-TIDY-COMPDB-NO-BUILD-AUTHORITY-001`, `INV-TDRP-CLANG-TIDY-COMPDB-COMMAND-SEMANTICS-001`, `INV-TDRP-CLANG-TIDY-COMPDB-PLATFORM-LIMITS-001`.

Closing only strict133 changes counters to unknown=16,243, unread=16,218, omitted=0, responsibilities=151. `baseline_identity_accepted=false`, `source_closure_ready=false` and independent release acceptance remain mandatory.

Next deterministic entry: strict134 `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434`.
