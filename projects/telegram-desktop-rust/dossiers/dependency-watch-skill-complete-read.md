# Dependency Watch skill complete read — strict recursive index 39

Status: strict index 43 exact-head verified at aa3b353daca8b75be04a534f5643465297263962; strict index 44 source-grounded and existing-owner mapped; child exact-head verification pending.

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


## Strict index 40 — OpenAI native skill registration

Source: .agents/skills/dependency-watch/agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d on telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844. It is declarative host-registration metadata only: display_name="Dependency Watch", short_description="Track dependency releases and missing upstream fixes", and default_prompt selects $dependency-watch against freshly fetched origin/dev with pressing updates before the review-someday list.

Caller/owner trace: the descriptor's default prompt names sibling $dependency-watch; workflow semantics remain in SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8 and executable snapshot/version behavior remains in scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38. scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea covers fresh fetch without worktree mutation, fetch-failure rejection, missing required manifests, offline-unverified state, linked-worktree storage, and version classification. Upstream AGENTS.md@b138b4fbfae7a8d1e92d28c154e09eaca2418e51 states the scheduled dependency monitor remains advisory and does not implement fixes.

Fabushi existing owner: quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70 defines Repository quality-health native invocation evidence for host-visible metadata/default prompt/equivalent selector and canonical source-neutral workflow resolution, with no independent scanner/state/task owner. TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb qualifies native skill surfaces and rejects stale/mutable-source publication or provider-specific syntax becoming semantic ownership. AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b keeps exact-source quality-health orchestration canonical. Shipping app updater and MCP installable-package owners are rejected. novel_capability=false.

Parent exact-head gate: TDRP 39506049373 succeeded at 67fd7d1a8259d4990096bc1952b0459bebf7b464 and proved root 7116/6783/36/6819, recursive=16305, cppgir/expected-lite, root/child/next shard artifacts, fixed-point final_scanned_authorities=203 with unresolved_gitlink_authorities=0, and Boost 582/582/open=0. Reachability artifact 12304695149 digest sha256:83223fb59675f6e5320a3e8b6e1a593b52b90f1ceafda1d4e87d9e766bed0c48 still reports 95 authorities, 133 gitlinks, 3865 acquisition candidates, 4674 generated inputs, 4602 resource inputs and 4255 tool inputs open; source_closure_ready=false. Rust 39506049401 succeeded 6/6 jobs with artifact 12303721533 digest sha256:5a3fbcd642b49ead461369ae9815971d481cf1839f76fdd89960b4082ca5614a.

Traceability:
- TDRP-R9-DEPENDENCY-WATCH-NATIVE-INVOCATION-001
- ORA-TDRP-DEPENDENCY-WATCH-NATIVE-ROUTING-001
- INV-TDRP-DEPENDENCY-WATCH-REGISTRATION-NO-SECOND-OWNER-001
- SEC-TDRP-DEPENDENCY-WATCH-NATIVE-INVOCATION-FAIL-CLOSED-001

Coverage after strict index 40: source_entries_total=16305, unknown=16262, unread=16237, omitted=0. baseline accepted=false and source_closure_ready=false. Next strict entry index 41 is .agents/skills/dependency-watch/references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde.


## Strict index 41 — patched-fork and original-upstream provenance

Source: .agents/skills/dependency-watch/references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde on telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844/tree 1fc479a4b448aed851181bf4a4796d3015368348.

Responsibility: patched or copied upstreams are not one moving version. The monitor keeps (1) the exact revision pinned by the current Telegram snapshot, (2) the fork's own current upstream branch, and (3) the original project's stable/development upstream as separate authorities. Fork-head movement can be a pin update while a fresh original-upstream import can carry a large Telegram patch stack. Comparable history must be reconstructed from actual import provenance before calculating lag. A failed merge-base, unrelated commit count, matching subject or merely-applicable patch is not proof. Security candidates require the fixing diff and prerequisites plus pinned source presence, compiled backend/build flags and reachable use before classifying confirmed-missing, already-present/backported, not-applicable, or needs-investigation. The monitor proposes update/import/backport routes; it does not apply them. Mapped baselines, scanned ranges, unresolved old tails and per-fix evidence remain durable across scans.

Current exact recipe evidence: Telegram/build/prepare/prepare.py@4063cc766ffb60c018b7a6ad456453ec585d5566 pins desktop-app/tg_owt@07fb35bc33b3e79125a1beab7690e830f183f5d8 with recursive submodules, desktop-app/tg_angle@b34c4dc879d115d34794a657c7fa5f6bb34bc10a on Windows, and desktop-app/patches@fe957833b84525e634a8d776a7f834ed536d9a21. Telegram/build/docker/centos_env/Dockerfile@f72162089985263493c6e3f7e23b566f5df8e454 and snap/snapcraft.yaml@38f0eb362832c3623b868c9aa90c03bc4f426d01 independently pin the same tg_owt/patches revisions where used. The reference maps tg_owt to the WebRTC original upstream and tg_angle to the ANGLE original upstream, but explicitly treats those mappings as discovery seeds rather than fixed baseline claims. Current tg_angle exposure must therefore remain configured-build/reachability evidence, not be inferred from an old milestone note.

Caller/test trace: SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8 is the direct caller: it requires references/forks.md for patched/copied upstreams and always for tg_owt/tg_angle when present. Repository search finds no other direct references/forks.md caller. scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38 owns generic snapshot/version mechanics but does not directly parse this reference or encode tg_owt/tg_angle import archaeology. scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea provides the freshness/fail-closed primitives used by the workflow: stable version ordering, fresh remote fetch without worktree mutation, fetch-failure rejection, required-manifest failure, offline-unverified state and linked-worktree report storage. No dedicated fork-provenance test is falsely claimed.

Fabushi existing owner: quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70 already requires repository quality-health observations to bind exact source authority/revision, fetch outcome and skipped/unavailable pinned inputs, preserving stable finding identities and provenance. TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb keeps exact-source qualification fail-closed. security-privacy-test-plan.md@f6d6672988d46c06152f4e2f5ff585fecc424141 owns security/release evidence gates. build-time-acquisitions.json@e714a52f7033d288ef6e13e02721dc3c3896f772 already records the exact tg_owt/patches pins and recursive build-reachability evidence. Shipping app updater and MCP installable-package owners are rejected; no Telegram fork scanner/backport daemon or second findings store is required. novel_capability=false.

Parent exact-head gate: TDRP run 39509311259 succeeded at 37504a29151be2d65a92f1b60b34efc5913a7e0e; source-authority artifact 12304418404 digest sha256:0d1ac43c798569ee82a75c43af60e64b0bcde11fcd33590c73ef65a0dae11079. Its recursive fixed-point remains 203 scanned authorities / 0 unresolved gitlink authorities, while recursive reachability remains partial-audited-open with authorities=95, tree_entries=167824, gitlinks=133, acquisition_candidates=3865, patches=235, lfs_pointers=21, generated_inputs=4674, resource_inputs=4602 and tool_inputs=4255; source_closure_ready=false. Rust run 39509311149 succeeded 6/6 jobs at the same exact head; rust-host steps 13-17 plus post steps completed successfully and artifact 12304913206 has digest sha256:a8dcd88399978b0f59d6d0a6bb8aec328b958d585a364ae94cf4c3f63fb35a65.

Traceability:
- TDRP-R9-DEPENDENCY-WATCH-FORK-PROVENANCE-001
- ORA-TDRP-DEPENDENCY-WATCH-FORK-THREE-STATE-001
- ORA-TDRP-DEPENDENCY-WATCH-IMPORT-PROVENANCE-001
- ORA-TDRP-DEPENDENCY-WATCH-FIX-APPLICABILITY-001
- INV-TDRP-DEPENDENCY-WATCH-NO-INCOMPARABLE-LAG-001
- INV-TDRP-DEPENDENCY-WATCH-RECIPE-PROVENANCE-001
- INV-TDRP-DEPENDENCY-WATCH-ADVISORY-ONLY-001
- SEC-TDRP-DEPENDENCY-WATCH-PATCHED-FORK-FIX-CLASSIFICATION-001

Coverage after strict index 41: source_entries_total=16305, unknown=16261, unread=16236, omitted=0. baseline accepted=false and source_closure_ready=false. Next strict entry index 42 is .agents/skills/dependency-watch/references/release-trust.md@81a68395f9ad54ed651a84464a933ed7e57c9d32.


## Strict index 42 — dependency release trust and adoption readiness

Source: .agents/skills/dependency-watch/references/release-trust.md@81a68395f9ad54ed651a84464a933ed7e57c9d32 on telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844/tree 1fc479a4b448aed851181bf4a4796d3015368348.

Responsibility: release urgency and adoption readiness are separate. Each actionable candidate binds publication date, canonical/immutable source identity and the source form actually consumed, then records relevant upstream plus independent advisory/regression evidence, source/package/build deltas, changed publisher/signer/repository/release process, generated archive-only inputs and signature/attestation evidence where available. No checksum, signature, elapsed observation window or passing build is a safety proof. The durable decision is Update, Backport, Hold, Skip or Track with decision_reason, trust_checks, regression_checks, unknown_checks and review_after/review_condition. Missing evidence remains unknown/hold rather than clearance. Known compromised candidates such as XZ 5.6.0/5.6.1 are rejected specifically; later versions are reassessed from current evidence instead of permanently blacklisting the project. The monitor remains advisory-only and cannot install, bump pins or approve deployment.

Caller/test trace: SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8 is the direct caller and requires release-trust.md before any candidate recommendation. Repository search found no second direct caller. scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38 owns exact fresh snapshot and stable-version comparison mechanics; scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea covers version classification, fresh fetch without worktree mutation, fetch-failure fail-closed behavior, required-manifest failure, offline-unverified state and linked-worktree storage. It does not encode a dedicated release-trust decision matrix, and no such dedicated test is falsely claimed.

Fabushi existing owner: quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70 owns Repository quality-health evidence and exact-revision provenance; TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb requires fresh exact-source authority before durable publication and keeps repository-quality evidence separate from release acceptance; security-privacy-test-plan.md@f6d6672988d46c06152f4e2f5ff585fecc424141 makes Security/privacy P0/P1 release-blocking. Shipping app updater and MCP installable-package owners are rejected because they execute product/package updates rather than own repository dependency advisory truth. No Telegram-specific release scanner/updater or second findings store is required; novel_capability=false.

Parent exact-head gate: TDRP run 39511909703 succeeded at c380f8bdae6c1f6b909874e92b6d551a58262e32. Source-authority artifact 12306591548 digest sha256:555388802314c7031a814637cf8746e3294e6ddb5f0806806fc184461d4a4b74; Boost artifact 12306521471 digest sha256:34d19cebfb85dc49fa2798b475f78b1b9aacd02cf8e33b0066cc947d0a2bd86a proves 582/582/open=0; recursive closure artifact 12306387143 digest sha256:b5cfe63821e62d70415468df742dfe01b2cb446b46905e2e99dd37a78bb3a777 has unresolved_gitlink_authorities=0 but source_closure_ready=false; reachability artifact 12305924061 digest sha256:caf811ba6d94dc896adc1958f2a8cd84731c63e21627607c62f710a9b0e8c32c remains partial-audited-open with 95 authorities, 133 gitlinks, 3865 acquisition candidates, 4674 generated inputs, 4602 resource inputs and 4255 tool inputs open. Rust run 39511909737 succeeded 6/6 exact-c380 jobs; artifact 12305069290 digest sha256:b327bfe8a430eef8653190c7f146c04d76af3c89d69a0be570e9a8bf4d9f6304.

Traceability:
- TDRP-R9-DEPENDENCY-WATCH-RELEASE-TRUST-001
- ORA-TDRP-DEPENDENCY-WATCH-RELEASE-IDENTITY-001
- ORA-TDRP-DEPENDENCY-WATCH-ADOPTION-DECISION-001
- ORA-TDRP-DEPENDENCY-WATCH-TRUST-EVIDENCE-001
- INV-TDRP-DEPENDENCY-WATCH-NO-AUTO-ADOPT-001
- INV-TDRP-DEPENDENCY-WATCH-NO-TRUST-OVERCLAIM-001
- INV-TDRP-DEPENDENCY-WATCH-UNKNOWN-FAIL-CLOSED-001
- INV-TDRP-DEPENDENCY-WATCH-CANDIDATE-SPECIFIC-COMPROMISE-001
- SEC-TDRP-DEPENDENCY-WATCH-RELEASE-TRUST-001

Coverage after strict index 42: source_entries_total=16305, unknown=16260, unread=16235, omitted=0. baseline accepted=false and source_closure_ready=false. Next strict entry index 43 is .agents/skills/dependency-watch/scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38.

## Strict index 43 — exact-source dependency snapshot and stable-version classifier

Source: .agents/skills/dependency-watch/scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38 on telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844/tree 1fc479a4b448aed851181bf4a4796d3015368348. The file was read completely (191 lines) together with direct workflow caller SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8 (150 lines) and watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea (100 lines).

Responsibility: snapshot() resolves the Git top-level, optionally fetches only refs/heads/dev into refs/remotes/origin/dev without tags or recursive submodules, resolves one immutable origin/dev commit, enumerates that exact root tree, preserves gitlinks, selects required build/dependency files plus manifest-name matches, and fails when required manifests are missing/non-regular or unreadable. It uses cat-file on exact blob ids rather than mutable working-tree files, writes copied UTF-8 sources plus snapshot.json/candidates.json only under the Git common directory dependency-watch/runs, records skipped inputs explicitly, and marks --no-fetch runs fresh_fetch=false/audit_status=snapshot-only. The candidate regex is navigation evidence only, matching the sibling SKILL.md warning that it is not complete dependency inventory. compare() classifies only supported three-component stable versions as same/older/major/minor/patch and returns unknown for prereleases, hashes, milestones, legacy-letter or four-component schemes.

Caller/test trace: SKILL.md invokes python3 .agents/skills/dependency-watch/scripts/watch.py snapshot and uses watch.py compare CURRENT CANDIDATE only for ordinary three-component stable versions. watch_test.py imports watch directly. VersionTests.test_stable_ordering_and_prefixes covers prefixes, stable ordering and unsupported forms. SnapshotTests.test_fetches_remote_without_touching_working_tree proves fresh remote acquisition, exact fetched source content, gitlink capture and unchanged dirty checkout/HEAD; test_fetch_failure_never_uses_stale_ref rejects stale fallback; test_missing_required_manifest_fails rejects missing required scope; test_offline_snapshot_is_marked_unverified preserves fresh_fetch=false; test_linked_worktree_uses_common_report_storage proves common Git storage. These are source-read test contracts; this mapping does not falsely claim a fresh execution of upstream Python tests.

Fabushi existing owner: AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b already defines Repository quality-health command orchestration as the source-neutral owner for exact-revision acquisition, dependency refresh, transient measurement, durable findings publication and fail-visible execution. Durable scan state must derive from one freshly resolved exact source revision; failed fetch, missing commit, mismatched detached HEAD, incomplete dependency or process failure is non-passing. Detached inspection worktrees/caches may be used only as exact-source subjects without mutating the canonical checkout. quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70 and TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb retain exact-revision provenance and fresh exact-source qualification; security-privacy-test-plan.md@f6d6672988d46c06152f4e2f5ff585fecc424141 keeps security/privacy release evidence fail-closed. Shipping app updater and MCP installable-package owners are rejected. Copying source-specific watch.py or adding a second scanner/findings store would violate the canonical source-neutral owner; novel_capability=false.

Parent exact-head gate: TDRP run 39514005648 is terminal success at a1dfd724295a985c286ad25e8d5628b794fcf056. Source-authority artifact 12307031769 digest sha256:f90afa1a92a4888260906d2b1a46722ffa3279bb73e645b5b97362acb1f801bf; Boost artifact 12307546480 digest sha256:e2aa1736144a124fe0ad36da78c511aab513e1f797e3bc20731fa717eaca7e76 proves 582/582/open=0; recursive closure artifact 12307181868 digest sha256:4c63d10b02a7138530ea5e3f3292742ec32707d456b9f952feb5250e0d97b11f reaches final_scanned_authorities=203 and unresolved_gitlink_authorities=0; recursive reachability artifact 12308110096 digest sha256:30c70e0acb688c8407537075b62a16439845a80315a9756631a03a8e7a09e053 remains partial-audited-open with 95 authorities, 133 gitlinks, 3865 acquisition candidates, 4674 generated inputs, 4602 resource inputs and 4255 tool inputs open, so source_closure_ready=false. Root 0-11, child 0-23 and next 0-15 shard artifacts are complete; next aggregate artifact 12306921952 digest sha256:91f40466f218e8fd5aaee46656d389294f29b4b5122fdbf5c12cc79b05c47fd6. Rust run 39514005658 is terminal success with renderer, platform-worker, rust-host and three pressure-profiler jobs 6/6 successful. rust-host steps 1-17, both post steps and Complete job succeeded; artifact 12307581107 digest sha256:4b07048bf3637d3f6f02e8841edd4dbd4ae6a07adb203f54313d30eb4ae86776.

Traceability:
- TDRP-R9-DEPENDENCY-WATCH-SNAPSHOT-001
- ORA-TDRP-DEPENDENCY-WATCH-FRESH-SNAPSHOT-001
- ORA-TDRP-DEPENDENCY-WATCH-EXACT-BLOB-001
- ORA-TDRP-DEPENDENCY-WATCH-VERSION-CLASSIFICATION-001
- INV-TDRP-DEPENDENCY-WATCH-NO-WORKTREE-MUTATION-001
- INV-TDRP-DEPENDENCY-WATCH-NO-STALE-ALL-CLEAR-001
- INV-TDRP-DEPENDENCY-WATCH-COMMON-STORAGE-001
- INV-TDRP-DEPENDENCY-WATCH-CANDIDATES-NOT-COMPLETE-001
- SEC-TDRP-DEPENDENCY-WATCH-SNAPSHOT-FAIL-CLOSED-001
- REG-TDRP-DEPENDENCY-WATCH-MUTABLE-WORKTREE-READ-001

Coverage after strict index 43: source_entries_total=16305, unknown=16259, unread=16234, omitted=0, responsibilities_total=62, unmapped=0. baseline accepted=false and source_closure_ready=false. Next strict entry index 44 is .agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea.

## Strict index 44 — dependency snapshot regression and fail-closed qualification

Source: .agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea on telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844/tree 1fc479a4b448aed851181bf4a4796d3015368348. The file was read completely (100 lines) together with the exact implementation under test watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38 (191 lines) and workflow contract SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8 (150 lines).

Responsibility: the unittest suite qualifies the dependency-watch snapshot/version helper with an isolated temporary Git remote and checkout. VersionTests.test_stable_ordering_and_prefixes proves supported prefixed three-component stable versions classify as patch/minor/major/older/same while prereleases, hashes, milestones, four-component versions and OpenSSL legacy-letter versions stay unknown. SnapshotTests.test_fetches_remote_without_touching_working_tree proves a fresh remote update is read from the fetched exact commit rather than a dirty checkout, a gitlink is retained exactly, and local HEAD/status remain unchanged. test_fetch_failure_never_uses_stale_ref requires acquisition failure instead of stale-ref fallback and requires that no dependency-watch storage be published. test_missing_required_manifest_fails rejects incomplete required scope. test_offline_snapshot_is_marked_unverified keeps fetch=False explicitly non-fresh with audit_status=snapshot-only. test_linked_worktree_uses_common_report_storage proves detached linked worktrees share the main repository Git common dependency-watch storage root.

Caller/test trace: watch_test.py imports watch directly and is a standalone unittest module ending in unittest.main(); repository search found no separate production caller for the test module. The real implementation symbols exercised are watch.compare, watch.snapshot, watch.git and watch.REQUIRED, with snapshot transitively exercising exact ls-tree/cat-file source selection, required-manifest checks, gitlink capture and Git-common storage. SKILL.md is the production workflow caller of watch.py, not a caller of watch_test.py. These are source-read upstream test contracts; this mapping does not claim that the upstream Python suite was freshly executed. All Fabushi executable TDRP verification remains GitHub-Actions-only.

Fabushi existing owner: AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b owns the source-neutral repository quality-health command orchestration and makes failed fetch, stale/mismatched source, incomplete dependency and process failure explicitly non-passing. quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70 already owns integrated repository-quality regression evidence: exact test-subject revision, fixture inputs, exercised existing owners, positive/negative oracle observations, exact-ref durable-publication behavior, and the rule that temporary Git repositories/subprocess wrappers are test mechanics rather than a second semantic owner. TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb keeps repository-quality qualification fail-closed and requires FBCP/TDRP executable verification in GitHub Actions. No Telegram-specific Python test owner, second scanner, second findings store or product runtime is required; novel_capability=false.

Parent exact-head gate: TDRP Revision 9 run 39515594730 is terminal success at aa3b353daca8b75be04a534f5643465297263962. Its first Source authority job 118946870617 had already passed the Revision 9 validator before an external chromium.googlesource.com mini_chromium HTTP 503; the failed acquisition job was retried without a code change and latest-attempt Source authority job 118949358056 succeeded. Successful source-authority artifact 12308247434 digest sha256:cf9a57537be204f97241ee2a684f447fb4c954a957fe2dc363bded601078237a binds aa3b353daca8b75be04a534f5643465297263962/0eca52223cceed229932add1cc80792763120844/1fc479a4b448aed851181bf4a4796d3015368348. Boost artifact 12307911850 digest sha256:d4bd045fc2ac550310bc94bb2b640cef693236f3534ba2336b411c863ea36a67 proves 582/582/open=0. Root aggregate artifact 12307043808 digest sha256:9db6bde956e510091ff31c4aac2710bf3e355174acd39f6673d7eb38e6e6d995 contains root shards 0-11; child aggregate 12307643558 digest sha256:e153244fa21ba1d5ca1aee1df88f415ce4d00e589e9ee89a04db8f715eb95728 contains child shards 0-23; next aggregate 12307404473 digest sha256:573937916232daffe55f1251215d303baf1064ec77fc459e196040094e3c22eb contains next shards 0-15. Recursive closure artifact 12306893764 digest sha256:c8e62637612e6dae6d759d771ce3b00b1344b097647129518db11a901174ed3c reaches final_scanned_authorities=203 and unresolved_gitlink_authorities=0. Recursive reachability artifact 12307503655 digest sha256:53da0ab81dcca94939a330c7351fd5de440cf9366176c0119f57290316fee94d remains partial-audited-open with 95 authorities, 133 gitlinks, 3865 acquisition candidates, 4674 generated inputs, 4602 resource inputs and 4255 tool inputs open, so source_closure_ready=false. Rust desktop runtime run 39515594749 is terminal success with rust-host, renderer, platform-worker and macOS/Windows/Ubuntu pressure profiler 6/6 successful; rust-host steps 1-17, both post steps and Complete job succeeded. Artifact 12306824163 digest sha256:8172d2ce4fabc25fe9a36475eb9be948101f239e7e9cd2f27e14ddbaca07307b is bound to aa3b353daca8b75be04a534f5643465297263962.

Traceability:
- TDRP-R9-DEPENDENCY-WATCH-SNAPSHOT-TESTS-001
- ORA-TDRP-DEPENDENCY-WATCH-TEST-FRESH-FETCH-001
- ORA-TDRP-DEPENDENCY-WATCH-TEST-FAIL-CLOSED-001
- ORA-TDRP-DEPENDENCY-WATCH-TEST-COMMON-STORAGE-001
- ORA-TDRP-DEPENDENCY-WATCH-TEST-VERSION-CLASSIFICATION-001
- INV-TDRP-DEPENDENCY-WATCH-TEST-NO-WORKTREE-MUTATION-001
- INV-TDRP-DEPENDENCY-WATCH-TEST-NO-STALE-FALLBACK-001
- INV-TDRP-DEPENDENCY-WATCH-TEST-OFFLINE-UNVERIFIED-001
- SEC-TDRP-DEPENDENCY-WATCH-TEST-FAIL-CLOSED-001
- REG-TDRP-DEPENDENCY-WATCH-TEST-FIXTURE-ISOLATION-001

Coverage after strict index 44: source_entries_total=16305, unknown=16258, unread=16233, omitted=0, responsibilities_total=63, unmapped=0. baseline accepted=false and source_closure_ready=false. Next strict entry index 45 is .agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c.

