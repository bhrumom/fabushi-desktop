# Code-health harness_test.py complete read

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Fixed commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries / 6,859 blobs)
- Path: `.agents/skills/code-health/scripts/harness_test.py`
- Blob: `5d6b2537c9289cb3e1db0904b9f8e1b99dec4b74`

## Exact responsibility

This file is the executable regression-contract suite for the helper-inventory instrument. It owns no shipping product state and no independent repository-quality state. Its fixtures create disposable Git repositories for source and archived task evidence, then exercise `harness.inspect`, `harness.patch_code`, `harness.identifiers`, exact-ref/worktree source adapters, and the existing report finding merge lifecycle.

The 13 fixed-U tests cover:

1. `test_ninety_day_boundary_and_low_lifetime_use`: exactly 90 days enables review reasons; 89 days does not, and distinct product-task counts change the low-lifetime threshold without inventing recent use.
2. `test_young_helper_and_missing_archive_are_not_zero_use_proof`: young helpers and missing archives cannot become absence-of-use proof.
3. `test_no_saved_patches_is_unavailable_not_zero_use`: an archive with no usable saved patches is unavailable, not observed zero.
4. `test_creation_harness_and_self_test_calls_are_separate`: creator, harness-only and self-test-only tasks stay out of independent product-consumer evidence.
5. `test_final_patch_context_counts_but_deletions_and_helper_edits_do_not`: surviving final C++ context can count; deleted lines and edits inside the helper implementation are excluded.
6. `test_distinct_tasks_not_duplicate_patches_or_calls`: repeated calls or duplicate patches within one task remain one consumer.
7. `test_comments_strings_and_commented_includes_are_not_consumers`: comments, strings/raw strings and commented includes cannot manufacture use; a real include can.
8. `test_self_tests_internal_consumers_and_open_work`: self-test/internal classification, helper dependency edges, indirect product use, open-work propagation and routing holds are preserved.
9. `test_unclassified_consumers_do_not_become_known_zero`: unknown task kind stays uncertain and suppresses false review reasons.
10. `test_docs_are_read_from_the_scanned_ref`: exact-ref scans read committed documentation while worktree scans read the mutable worktree.
11. `test_dirty_archive_cannot_supply_retirement_evidence`: dirty canonical task state invalidates archive coverage and product-task claims.
12. `test_other_namespaces_are_not_harness_dependencies`: symbol ownership is namespace-sensitive and unrelated names do not become harness callers.
13. `test_stable_identity_and_unavailable_evidence_do_not_resolve_findings`: stable finding identity and `first_seen` survive rescans; unavailable retention evidence yields `unverified`, not false resolution.

## Direct callers/consumers and dependencies

- The file directly imports `harness.py@652a9ee7ed6a690705cad45aeff06568210db6b1`, `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12`, `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b`, and the disposable `Repo` fixture from `health_test.py@6058fa64a5121c966021d8589b2010480aec8794`.
- It is directly executable through its own `if __name__ == '__main__': unittest.main()` and is eligible for ordinary Python unittest discovery by its `*_test.py` name.
- Fixed-U repository search found no separate product/runtime caller by file name; `report.py` and `health.py` consume `harness.py`, not this test module. The suite therefore remains qualification evidence, not a production entrypoint.

## Existing-owner-first

The applicable owner is the existing Fabushi repository quality/test-evidence governance in `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` and `quality/evidence-contract.md`. This follows the already-established `health_test.py` mapping: preserve source-neutral regression oracles and negative controls, not upstream Python unittest plumbing.

No platform-core production delta, product UI/runtime change, new quality scanner, second findings ledger, second task queue, or new ADR is required.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-HARNESS-INVENTORY-REGRESSION-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-HARNESS-REGRESSION-BOUNDARIES-001`, `ORA-TDRP-CODE-HEALTH-HARNESS-REGRESSION-COVERAGE-001`, `ORA-TDRP-CODE-HEALTH-HARNESS-REGRESSION-CONSUMERS-001`, `ORA-TDRP-CODE-HEALTH-HARNESS-REGRESSION-IDENTITY-001`
- Contracts authored, not run: `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-BOUNDARIES-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-COVERAGE-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-CONSUMERS-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-PATCH-FILTERING-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-LEXICAL-OWNERSHIP-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-EXACT-REF-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-FINDING-LIFECYCLE-001`, `INT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-001`
- Fault controls authored, not run: `FAULT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-MISSING-ARCHIVE-001`, `FAULT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-DIRTY-ARCHIVE-001`, `FAULT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-LEXICAL-FALSE-POSITIVE-001`

Stage A disposition: `implemented / validation_status=not-run / accepted=false`.
