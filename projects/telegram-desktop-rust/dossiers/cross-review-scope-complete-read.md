# Complete read — .agents/shared/scripts/cross_review_scope.py

Status: independently understood and mapped; exact-head verification pending  
Project: TDRP-001 Revision 9  
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`  
Source: `.agents/shared/scripts/cross_review_scope.py@d9fafdf9fa0857cc83f0ccb49d15d4fad8782768`

## Inventory identity

GitHub Actions run `39335233589`, artifact `12226273177`, digest `sha256:0192d50b480c4c72a5e48debd611fb4a3d72483b28a59298fdbf908195416287`, produced the current exact-head recursive authority artifact for Fabushi PR #139. Its `upstream-recursive-inventory.json` has 16,202 non-directory entries and directly identifies actual 1-based array entry 16 as `.agents/shared/scripts/cross_review_scope.py`, blob `d9fafdf9fa0857cc83f0ccb49d15d4fad8782768`, size 6,081 bytes. Entries 14 and 15 are independently closed by their own ledger rows; older migration-progression ordinals are not artifact-array positions.

## Complete source read

The entire production helper was read. It is a small repository review-scope freezer/snapshotter, not Telegram product runtime code.

Independent source responsibilities:

- `command_init` normalizes the supplied source root and verifies the repository top-level/common Git directory.
- It freezes one base: `HEAD` for uncommitted mode or verified `HEAD~N` for bounded commit-review mode; an N that reaches beyond history fails instead of silently changing scope.
- It creates a collision-resistant default session beneath the repository common Git directory unless an explicit session is supplied, and refuses an existing session identity.
- `session.json` persists repository root, scope mode, commit count, exact base commit and subject, starting HEAD, commit list, dirty-at-start fact, and creation time.
- `command_snapshot` reads that durable session rather than recomputing the review base from the mutable worktree.
- Snapshot staging is isolated with a private `GIT_INDEX_FILE`; the existing real index is copied only as a starting point and is never used as scratch state.
- `git add -A` in the private index deliberately includes tracked modifications, deletions and untracked files, after which `write-tree` creates one immutable tree identity for the round.
- Each round persists `tree.txt`, a complete rename-aware, binary-safe `scope.patch` against the frozen base, `files.txt` name/status projection, file count and short-stat.
- `previous_tree` finds the nearest existing prior round rather than assuming every earlier round number exists.
- From round two onward, the helper writes a rename-aware, binary-safe `fix.patch` from that prior immutable tree to the new tree and records the prior round plus fix stat.
- Git/repository/session acquisition failures fail closed through `fail`; a valid but empty scope is explicitly signaled with exit code 3 rather than treated as review success.

## Real callers and dependencies

Repository search shows the production helper is called by:

- `.agents/shared/cross-review/protocol.md@51dabe142d5c3c7247e4050790c4e74b40f35b27` — normative frozen-scope lifecycle; invokes `init` once and `snapshot` per round, declares `scope.patch`, `files.txt`, `fix.patch`, private-index isolation and empty-scope exit 3.
- `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361` — command runner freezes/snapshots round 1 before reviewers, snapshots subsequent rounds after fixes, and runs comment rules against `SESSION/index` without touching the real index.
- `.agents/shared/cross-review/prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d` — reviewer prompts consume the complete round `scope.patch`/changed paths and later focused reviewers consume `fix.patch` while retaining the complete frozen scope.
- Existing Fabushi cross-review policy already consumes the source-neutral frozen-scope semantics in root `AGENTS.md`, Test Strategy and Test Evidence Contract.

No shipping messaging, UI, Coordinator, Host or Runner caller owns this helper.

## Existing-owner-first resolution

The applicable responsibility is not to copy a Git/Python helper. Current Fabushi already has the source-neutral owners:

1. root `AGENTS.md` — frozen scope and bounded review lifecycle;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` — cross-review scope/arbitration qualification;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` — durable review ledger/evidence.

This closure extends those owners with the independently attributable snapshot-acquisition details: exact base/starting HEAD, private staging, untracked inclusion, immutable tree identity, rename/binary-safe scope and fix deltas, acquisition failure and empty-scope semantics.

Rejected owners:

- shipping Agent/Coordinator/Host/Runner are product execution owners, not repository review-scope evidence owners;
- packaging/release consumes review evidence but does not own scope acquisition;
- a copied `cross_review_scope.py`, Telegram review runtime or second review store would duplicate existing source-neutral governance.

No new owner or ADR is required.

## Fabushi-equivalent contract

`verified repository + declared mode -> exact frozen base/start HEAD/session identity -> isolated complete worktree staging (including untracked) -> immutable round tree -> complete scope diff/status manifest -> blind review -> accepted fixes -> next immutable round + fix delta against nearest prior tree`

Fail-closed branches:

- invalid repository/session/base/history range -> no snapshot;
- staging/tree/diff acquisition failure -> no review evidence;
- real-index mutation is forbidden;
- missing prior round -> no invented delta;
- valid empty scope -> explicit no-review stop, not acceptance;
- reviewer evidence binds the immutable round snapshot, not an ambient mutable worktree.

The upstream Python and Git command implementation are instruments. Fabushi absorbs the source-neutral scope identity, isolation and evidence invariants into its existing review governance.

## Traceability IDs

- Requirement: `TDRP-R9-CROSS-REVIEW-SCOPE-SNAPSHOT-001`
- Oracles: `ORA-TDRP-CROSS-REVIEW-SCOPE-IDENTITY-001`, `ORA-TDRP-CROSS-REVIEW-INDEX-ISOLATION-001`
- Invariants: `INV-TDRP-REVIEW-BASE-FROZEN-001`, `INV-TDRP-REVIEW-SNAPSHOT-COMPLETE-001`, `INV-TDRP-REVIEW-REAL-INDEX-UNTOUCHED-001`, `INV-TDRP-REVIEW-ROUND-IMMUTABLE-001`, `INV-TDRP-REVIEW-FIX-DELTA-LINEAGE-001`

## Verification disposition

This closure changes repository governance/evidence contracts only and adds no shipping runtime. Exact-head executable verification remains TDRP Revision 9 source authority plus Rust desktop runtime on the final PR head.

After this independent source-entry closure the counters become recursive=16,202, unknown=16,185, unread=16,158, omitted=0. Full source closure and baseline acceptance remain fail-closed.
