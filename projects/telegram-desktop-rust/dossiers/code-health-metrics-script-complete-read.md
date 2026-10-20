# Code-health metrics.py complete read

## Source authority
- Repository: `telegramdesktop/tdesktop`
- Commit: `78ceaa548ae0e055aa56b53fde22064d13f250ef`
- Path: `.agents/skills/code-health/scripts/metrics.py`
- Blob: `ebab29607cb52782899d5424636f0bd5e0c126cd`
- Root tree: `76b967833b38eb7aa2cb4047008c07331390e61e`.
- PR #158 exact-head run 39441511131 proved 7,012 root entries, 6,679 blobs, 36 direct gitlinks and 16,201 reconciled recursive non-directory entries. metrics.py is strict recursive index 31, immediately after includes.py index 30.

## Symbols and behavior
`area_of(path)` is a fallback quality-report classifier, not ownership authority. `report.py` prefers an approved atlas and falls back to it only when no atlas classification exists.

`file_metrics(tree)` inspects recognized C/C++ translation units and headers, recording physical line count, translation-unit function count/max length, functions at least `LONG_FUNCTION=150` lines, and classes whose method+field count is at least 40. These are source measurements and candidates; caller-owned thresholds decide findings.

`unused_candidates(tree)` counts identifiers across the exact scanned source while excluding `sources.LANG_FILE`, then examines production `Telegram/SourceFiles` C/C++ translation units excluding `/test/`. It uses a paired header when present and conservative exemptions for destructors, operators, entry points, framework callbacks and tests. `only_declared`/`declares` reject candidates when occurrences cannot be accounted for as declarations or when a local mention is an initializer-list/use context. The result remains a candidate list, not semantic dead-code proof.

## Real callers and tests
- `.agents/skills/code-health/scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` is the production aggregation caller for `area_of`, `file_metrics` and `unused_candidates`.
- `.agents/skills/code-health/scripts/health_test.py@a79addaaceb35848dbd17e94e01ea428aede7788` directly covers `UnusedTests.test_declared_but_never_called` and `UnusedTests.test_initializer_list_call_is_a_use`, while scan/report fixtures exercise aggregation.
- `.agents/skills/code-health/scripts/sources.py@688aa05c4223d0d45b8572cff89fbc5148254844` owns `LANG_FILE = Telegram/Resources/langs/lang.strings`.
- `cpp_text.py` supplies bounded lexical function/class/identifier recognizers; this metric layer is not a compiler or semantic reference resolver.

## Existing-owner decision
Applicable responsibility is exact-source structural quality measurement plus conservative dead-code candidate evidence. Fabushi already has source-neutral repository quality-health governance and source-text normalization evidence, including exact-source, heuristic-limit and metrics-not-acceptance boundaries. Reuse those owners; do not introduce a Telegram Python scanner, duplicate findings ledger, deletion engine or task owner.

## Failure and lifecycle boundaries
Canonical evidence must be bound to one exact scanned source snapshot. A stale, mutable or partial tree is non-passing. Unsupported C++ syntax/macros/generated relationships remain heuristic limits.

An unused candidate is never safe-deletion proof. Upstream explicitly states deletion is proven by building without the candidate; Fabushi keeps that stronger ordinary build/test/change-review boundary. Structural counts select investigation work only and never establish product correctness, ownership, configured-build truth, security, packaging or release acceptance.

## Traceability
- Requirement: `TDRP-R9-CODE-HEALTH-STRUCTURAL-METRICS-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-METRICS-EXACT-SOURCE-001`, `ORA-TDRP-CODE-HEALTH-UNUSED-CANDIDATE-001`, `ORA-TDRP-CODE-HEALTH-METRIC-BOUNDARY-001`
- Invariants: `INV-TDRP-CODE-HEALTH-METRICS-NO-DELETE-PROOF-001`, `INV-TDRP-CODE-HEALTH-METRICS-PROVENANCE-001`, `INV-TDRP-CODE-HEALTH-METRICS-NO-PARALLEL-OWNER-001`
- Direct unit cases: `UnusedTests.test_declared_but_never_called`, `UnusedTests.test_initializer_list_call_is_a_use`.

This closes only metrics.py. Generated/resource/tool disposition and the rest of semantic migration remain open; source_closure_ready stays false.
