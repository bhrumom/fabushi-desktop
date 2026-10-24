# Code-health review.py complete read

## Source authority
- Repository: `telegramdesktop/tdesktop`
- Commit: `78ceaa548ae0e055aa56b53fde22064d13f250ef`
- Path: `.agents/skills/code-health/scripts/review.py`
- Blob: `070b55553052267e8fbc12d70985b447b24bbb02`
- Root tree: `76b967833b38eb7aa2cb4047008c07331390e61e`.
- Parent exact-head run `39467249963` at `a9953cffaa81da2b672b41529bc8b598b7340334` proved the current 7,012-entry root tree, 6,679 blobs, 36 direct gitlinks, 16,201 reconciled recursive non-directory entries, recursive gitlink fixed point unresolved=0, current Qt-aware acquisition discovery, and Boost 582/582/open=0. That parent-head evidence becomes historical-only for this mapping commit.
- Recursive authority index 33 is `review.py`, immediately after `report.py` at index 32. The next strict entry is `.agents/skills/code-health/scripts/sources.py@688aa05c4223d0d45b8572cff89fbc5148254844`.

## Complete source responsibility
`review.py` is the diff-time repository-quality ratchet plus move-only proof. It does not own product behavior, canonical findings, task routing, or release acceptance.

- `code_path`, `tracked_path`, `untracked`, `changed`, `diff_lines`, `base_text`, and `added_hunks` bind the review subject to a declared Git base and current/staged tracked scope, including relevant untracked additions.
- `check(tree, base, staged, strict)` builds the source-derived CMake model and include graph, then checks only newly introduced change pressure: oversized new files; over-limit file growth beyond `GROWTH_TOLERANCE=20`; growing large classes; new target-crossing includes; materially heavier header reach using `BUILD_SHARE=0.002`; copied substantive hunks; removed definitions that remain mentioned; and removed last uses that leave only a definition.
- New cross-target includes are errors when the target pair has no existing crossing; another crossing on an already-violated pair is a warning. Over-limit growth and copied code are warnings normally and become errors under strict mode.
- `definitions`, `mention_lines`, `defines`, `orphaned`, `platform_only`, and `stranded` implement bounded lexical evidence for C++ definitions/uses plus style and language keys. Platform references remain visible even when the host does not compile them.
- `grown_classes` and `heavier` project caller-owned structural/build-work facts into review warnings.
- `moves(repo, base, head, staged)`, together with `out_of_order`, `comparable`, and `declaration_like`, proves relocation only by accounting moved substantive lines, allowed declaration/scaffolding additions, lost lines, substantive novel lines, and statement-order changes. Changed values, changed logic, lost/new substantive lines, or reordered statements make the verdict non-move-only.

## Real callers and regression coverage
- `.agents/skills/code-health/scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` exposes `check --base [--staged] [--strict]` and `moves --base [--head] [--staged]`, preserving warning/error and MOVE_ONLY/NOT_MOVE_ONLY result semantics.
- `.agents/skills/code-health/scripts/health_test.py@a79addaaceb35848dbd17e94e01ea428aede7788` directly exercises the ratchet in `CheckTests`: clean changes, new upward include errors, oversized-file growth and strict promotion, copy detection, stranded definitions, untracked new files, known-crossing warnings, platform mentions, class-qualified removed methods, orphaned helpers/styles/language keys, complete deletion and moved-definition negatives.
- The same test file exercises move proof in `MoveTests`: staged and untracked relocation positives, changed initializer values, reordered statements, reordered complete functions, and changed-logic negatives.
- `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` defines the human-facing check/moves semantics and explicitly keeps ordinary task proof stronger than the scanner.

## Existing-owner decision
Applicable responsibility is source-neutral exact-base change-ratchet evidence and move-only proof. Fabushi already owns it:
- `AGENTS.md` states that every source change is subject to a proportional quality ratchet against its exact base and enumerates new crossings, oversized growth, heavier dependency/build reach, duplicated logic, stranded definitions and orphan leftovers.
- Repository quality-health command orchestration delegates change warnings/errors to review policy instead of creating a second scanner/reviewer owner.
- `TEST_STRATEGY.md` qualifies change-ratchet warnings/errors, strict/non-strict review and move-only versus substantive-change negative controls.
- `evidence-contract.md` requires exact base/head or staged identity, changed-path scope, errors/warnings, move novel/lost-line accounting and stronger task/build/test evidence.

Therefore no Telegram Python review runtime, second review owner, second findings ledger or second task queue is introduced.

## Failure and lifecycle boundaries
Review evidence is valid only for the declared exact base and current/staged subject plus qualified source/build/include/clone facts. A stale/moving base, failed Git operation, incomplete source, unresolved recognizer case or failed dependency instrument is non-passing/unknown evidence; it cannot be converted to a clean ratchet.

Warnings/errors are quality-review disposition only. A clean result never proves behavior preservation, compilation, runtime correctness, package identity, security, or release acceptance. Move-only proof is deliberately narrower than semantic equivalence: it may establish that substantive lines were relocated without novel/lost/reordered logic, but the ordinary task still owns placement, consumer/build correctness and applicable product-risk verification.

## Traceability
- Requirement: `TDRP-R9-CODE-HEALTH-CHANGE-RATCHET-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-REVIEW-EXACT-BASE-001`, `ORA-TDRP-CODE-HEALTH-REVIEW-RATCHET-001`, `ORA-TDRP-CODE-HEALTH-MOVE-ONLY-001`
- Invariants: `INV-TDRP-CODE-HEALTH-REVIEW-NO-CLEAN-ON-UNKNOWN-001`, `INV-TDRP-CODE-HEALTH-REVIEW-STRICTNESS-001`, `INV-TDRP-CODE-HEALTH-MOVE-NO-SEMANTIC-CHANGE-001`
- Direct test families: `CheckTests` and `MoveTests` in `health_test.py`.

This closes only `review.py`. Coverage becomes unknown=16,165, unread=16,140, omitted=0. Recursive generated/resource/tool disposition remains open and `source_closure_ready` stays false.
