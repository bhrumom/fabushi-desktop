# tools/rules hook.py complete read — strict134

## Authority
- Upstream: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434`, fully read.
- Supporting anchors: `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`, `tools/rules/test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f`, `tools/clangtidy/README.md@2e64a408cb2d76aed0425706e321a24bb58f350f`, `.claude/settings.json@d47520dc0e3b7e8d6067456e33062140c75f7c79`, and `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361`.
- Parent Fabushi exact head: `6b5c69f923277607ce56831ecdbfeb9c65b2ff00`, tree `83fbf92bf600a324ef8aa0922cb6571766a7b438`.
- Parent TDRP `39991260296`: 61/61 success; authority `12518660328` / `sha256:3c332b1d5852e07cfacc1173e2ac0d5fc9dbb423236a2ccc4f7afd5314d7e7f0`; closure `12517688846` / `sha256:c02e879f42ce4680839b619324ce90e3e7f310e95c3ca4e4ffe537e78642f20e`; reachability `12518415977` / `sha256:f48b624620ad018ec1b6a2abaaf84b21e505a5b3f8e77e4c8676bc4cb9b422cc`.
- Parent Rust desktop runtime `39991260326`: 6/6 success; artifact `12517458444` / `sha256:821a30357d7a674acaa73351e2a827518f1b3f228026c30d2ef8052a93c00579`.
- Submission lease basis: live main `7394f6dc7687f9b986ee5589d40aa680ce23b90a`; live upstream dev remains `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Direct behavior and staged/edit semantics
`banned()` derives forbidden C++ helper names and replacement guidance from the locked `.clang-tidy` CustomFunctions list. String/character literals are hollowed before comment/function scanning so names inside literals or comments do not become calls. The edit path reads `new_string`, whole-file `content`, and multi-edit payloads, compares replaced text, scopes itself to `.cpp/.h/.mm/.cxx`, and returns actionable nonzero diagnostics for decidable violations.

Comment policy is source-specific: ordinary comment blocks are one line, a `// WHY:` block may span at most three lines, license text is exempt, test-harness paths are exempt from comment policy but not banned-function checks, and edit-time checks deliberately have no aggregate comment budget. A full-file write may read the prior `HEAD` version to avoid treating moved existing comment text as newly authored.

`--staged` reads `git diff --cached -U0 -M`, reconstructs added source lines and removed comment text, reports banned calls with source line numbers, exempts comment text found among removed lines as moved, permits at most two plain comment lines plus one marked block, and returns 1 when violations exist. The upstream README exposes this as an optional Git pre-commit command, while `.claude/settings.json` separately registers the same script as a Claude PostToolUse command for Edit/Write/MultiEdit. Cross-review can also run the staged mode against an alternate index.

## Limits that must remain visible
The adapter is not a completeness oracle. Malformed JSON explicitly returns 0; a missing/unsupported path is ignored; staged `git diff` is executed without `check=True`, so a failed diff can yield empty output and a clean return; prior-file `git show` failure is also not promoted to an error. Moved-comment credit is textual rather than identity-based. These fail-open/local heuristics are acceptable only as early feedback provenance and can never be cited as clean migration, build, static-analysis, or release evidence.

## Existing-owner-first / applicability
Fabushi already owns the applicable source-neutral responsibility in `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97` under Repository edit-policy feedback adapter governance: optional local/editor hooks derive checks from durable policy, stay limited to the changed file/diff, emit actionable diagnostics, fail closed for rules they can decide, never invent source-specific rules, and never replace exact-head GitHub Actions acceptance.

The upstream C++ suffix set, Qt helper replacement table, comment budgets, `// WHY:` convention, Telegram test-harness exemption, Claude PostToolUse payload shape and Git-hook command are provenance only. Current Fabushi exact-head repository search finds no `PostToolUse`, `hook.py`, `pre-commit`, or `--staged` implementation. Because the adapter is optional and the durable owner already exists, no second hook/runtime/quality owner is introduced; lack of a local adapter is not passing evidence and does not waive exact-head validators.

## Fail-closed
A local clean result, disabled/missing hook, malformed provider payload, failed local diff, unsupported file class, or source-specific exemption cannot prove Fabushi policy compliance. Any adopted fast-feedback adapter must remain subordinate to durable Fabushi rules, and FBCP/TDRP executable validation remains exact-head GitHub-Actions-only. Recursive source closure and independent release acceptance remain open.

## Traceability
Requirement `TDRP-R9-RULE-HOOK-FEEDBACK-001`.

Oracles: `ORA-TDRP-RULE-HOOK-SOURCE-001`, `ORA-TDRP-RULE-HOOK-BANNED-FUNCTION-001`, `ORA-TDRP-RULE-HOOK-EDIT-SCOPE-001`, `ORA-TDRP-RULE-HOOK-STAGED-DIFF-001`, `ORA-TDRP-RULE-HOOK-COMMENT-POLICY-001`, `ORA-TDRP-RULE-HOOK-FAIL-OPEN-LIMIT-001`, `ORA-TDRP-RULE-HOOK-OWNER-001`.

Invariants: `INV-TDRP-RULE-HOOK-DURABLE-POLICY-AUTHORITY-001`, `INV-TDRP-RULE-HOOK-CHANGED-SCOPE-ONLY-001`, `INV-TDRP-RULE-HOOK-NO-SOURCE-SPECIFIC-POLICY-COPY-001`, `INV-TDRP-RULE-HOOK-MALFORMED-INPUT-NOT-PASSING-EVIDENCE-001`, `INV-TDRP-RULE-HOOK-NO-LOCAL-ACCEPTANCE-001`.

Closing only strict134 changes counters to unknown=16,242, unread=16,217, omitted=0, responsibilities=152. `baseline_identity_accepted=false`, `source_closure_ready=false` and independent release acceptance remain mandatory.

Next deterministic entry: strict135 `tools/rules/test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f`.
