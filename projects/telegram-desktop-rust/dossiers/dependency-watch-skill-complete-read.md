# Dependency Watch skill complete read — strict recursive index 39

Status: source-grounded; existing-owner mapped; exact-head verification pending.

Authority: telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844 tree 1fc479a4b448aed851181bf4a4796d3015368348. Strict index 39 remains .agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8.

Live dev advanced linearly by 108 commits from 78ceaa548ae0e055aa56b53fde22064d13f250ef: 107 paths added, 182 modified, 3 removed. Current root non-directory entries are 6819. Telegram/lib_ui is the only changed direct gitlink; old/new lib_ui each contain 432 non-directory entries and no nested gitlinks. Exact recursive total therefore rebaselines 16201 -> 16305.

## Responsibility

Dependency Watch is advisory-only repository dependency/security monitoring. It must bind durable results to a freshly fetched exact revision, expand candidate matches into the complete manifest/recipe/gitlink input set, distinguish shipped/system/build dependencies, inspect releases and security advisories independently, preserve patched-fork provenance and release-trust evidence, retain stable pending/resolved finding history, mark partial/unavailable coverage explicitly, and atomically publish latest evidence without erasing prior runs. It must not edit pins/product source, build product, create tasks/PRs, apply backports, or infer an all-clear from stale/incomplete evidence.

## Source graph read

Read at the same live commit:
- agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d
- references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde
- references/release-trust.md@81a68395f9ad54ed651a84464a933ed7e57c9d32
- scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38
- scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea
- upstream AGENTS.md@b138b4fbfae7a8d1e92d28c154e09eaca2418e51 keeps scheduled dependency monitoring advisory.

Focused tests cover stable-version ordering, exact-ref fetch without working-tree mutation, fetch-failure fail-closed behavior, missing required manifest failure, offline-unverified state, and linked-worktree common report storage.

## Fabushi owner

Applicable behavior is already owned by:
- AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b repository quality-health exact-revision acquisition, durable findings, atomic publication, and fail-visible incomplete authority;
- quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb exact-head evidence/invalidation;
- quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70 durable provenance/publication evidence;
- quality/plans/security-privacy-test-plan.md@f6d6672988d46c06152f4e2f5ff585fecc424141 fail-closed security release gating.

Rejected owners: source/electron-main/update/* is the shipping app updater; source/host/src/extensions/mcp/installable_package/* is the shipping plugin-package installer. Neither owns repository dependency/advisory truth. No new owner is required.

## Parent evidence

TDRP 39497897314 at e709693ec625dc26cc027ac641f20fca348a5ed0 succeeded 61/61 jobs and 506/506 steps against the old upstream. It proved recursive=16201, cppgir/expected-lite, all root/child/next shards, fixed-point authorities=203 with unresolved_gitlink_authorities=0, and Boost 582/582/open=0. Reachability remained partial-audited-open with 95 authorities, 133 gitlinks, 3865 acquisitions, 4674 generated inputs, 4602 resource inputs and 4255 tool inputs open; source_closure_ready=false.

Rust 39497897342 succeeded 6/6 jobs; artifact 12299250602 digest sha256:56407823b6ac8397f653dfb75305effb84731c96e63206b8b01d8f324503b1e6.

Both runs are historical for the new upstream baseline.

## Validator delta

The Revision 9 validator is minimally generalized for positive root additions: added_path_count must match; each added path must be absent in the old tree and present as the recorded current blob; previous-total reconciliation subtracts explicit additions, adds explicit removals, and reverses direct-component delta. Exact fail-closed reconciliation is preserved.

Coverage after rebaseline and this mapping: source_entries_total=16305, unknown=16263, unread=16238, omitted=0, source_closure_ready=false, baseline accepted=false. Next strict entry is index 40 .agents/skills/dependency-watch/agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d.

## Traceability

- TDRP-R9-DEPENDENCY-WATCH-ADVISORY-001
- ORA-TDRP-DEPENDENCY-WATCH-FRESH-SNAPSHOT-001
- ORA-TDRP-DEPENDENCY-WATCH-ADVISORY-PROVENANCE-001
- ORA-TDRP-DEPENDENCY-WATCH-FAIL-CLOSED-001
- INV-TDRP-DEPENDENCY-WATCH-NO-WORKTREE-MUTATION-001
- INV-TDRP-DEPENDENCY-WATCH-NO-STALE-ALL-CLEAR-001
- INV-TDRP-DEPENDENCY-WATCH-STABLE-FINDING-HISTORY-001
- SEC-TDRP-DEPENDENCY-WATCH-ADVISORY-ONLY-001

Focused upstream cases at watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea: VersionTests.test_stable_ordering_and_prefixes; SnapshotTests.test_fetches_remote_without_touching_working_tree; test_fetch_failure_never_uses_stale_ref; test_missing_required_manifest_fails; test_offline_snapshot_is_marked_unverified; test_linked_worktree_uses_common_report_storage.
