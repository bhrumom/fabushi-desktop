# Complete read: process-inbox registration, workspace transaction, and video helper

## Authority and exact inventory identity

Source authority is telegramdesktop/tdesktop at commit 0eca52223cceed229932add1cc80792763120844, root tree 1fc479a4b448aed851181bf4a4796d3015368348. The recursive tree was not truncated. The strict zero-based root-tree entry 52 is .agents/skills/process-inbox/agents/openai.yaml, blob b664f98ad4440de7a8074dfe790d80f3d2201546. Neighboring entries are index 51 .agents/skills/process-inbox/SKILL.md, blob 0ce70794ac743d99e6d5c80894cde970b52842db; index 53 scripts/video.py, blob 977d75aca7aea0b6c74471d533a5ee0e2a156ac3; and index 54 scripts/workspace.py, blob e1fdabbac9ccd635962b7b6dc1a62357fc5b931c. The four files contain 4, 432, 293 and 5649 lines respectively.

The root inventory has 6819 non-directory entries, while the full recursive inventory including recursively inspected gitlinks/acquisitions has 16305. Reading dependency files here does not close their independent strict inventory entries: indices 53 and 54 remain separately open after index 52.

## What the OpenAI YAML owns

The four-line YAML declares only three host-facing fields: display_name, short_description and default_prompt. The prompt selects the existing $process-inbox skill. There is no executable code, schema, transaction, persistence, media processing, repository mutation, or product state in this file. The provider-specific spelling is an adapter, not a second semantic owner.

## Canonical call chain

1. External host discovery reads the registration and displays the name/description. Its default prompt routes into the shared process-inbox SKILL.md; it does not implement the workflow itself.
2. SKILL.md section Process Inbox and Workspace (lines 6-52) defines repository-only intake and setup. It reads the applicable repository instructions and shared development context, verifies the current checkout and ai_main checkout, checks the inbox transaction, and invokes workspace.py rather than directly mutating canonical repository state.
3. Route and plan (lines 53-173) requires current source authority and targeted project/task discovery before task assignment. It resolves superseded and split task lineage, rejects duplicate or stale task work, keeps routing/approval in the ordinary task lifecycle, and restricts disposable leaf planning to ignored transaction-local files. The orchestrator remains the owner of tracked task/state/receipt artifacts.
4. Assign task paths (lines 174-192) and Write tracked artifacts (lines 193-358) create the explicit task, state and receipt records required by the workflow. The generated receipt binds the processing transaction and payload digest to the proposed work. The skill distinguishes a proposed route from an approved, durable task.
5. Validate and publish (lines 359-425) requires schema/receipt/digest checks and a guarded publication through workspace.py. Reporting (lines 426-432) reports the actual result rather than claiming success on a failed or incomplete transaction.

## workspace.py: transaction and repository lifecycle

The helper is 5649 lines and is a general repository workflow CLI, not an application runtime. Its parser/main are near the end of the file. The relevant command path is:

- repository_config(args) derives repository/worktree locations and checkout identity from the repository layout, the build/ai-machine-tag marker, and supported environment overrides.
- worktree_config(args) and inbox_worktree_config(args) resolve the isolated inbox worktree and branch. The inbox branch is checkout-specific rather than a global shared state store.
- sync_inbox_worktree(args) and sync_inbox_canonical(config) keep the isolated checkout aligned with the canonical repository. publish_worktree(config, ...) owns guarded publication, including checking the main worktree/branch, fetching origin, requiring a safe clean worktree, rebasing onto current master and pushing the resulting commit without force-push semantics. Failures remain explicit/retryable rather than being silently treated as publication success.
- command_inbox_ensure(args) verifies or creates the canonical inbox/worktree arrangement. command_prepare(args) refuses unsafe or ambiguous state. It resumes only one valid active .processing transaction after verifying repository/checkout identity; multiple active transactions fail closed. For a new transaction it requires non-empty input, synchronizes the canonical branch, snapshots the input into a uniquely named backup/.processing-* directory and records transaction metadata.
- payload_entries(inbox), iter_payload_files(inbox), payload_digest(inbox), copy_payload(inbox, destination) and unique_backup_name(...) provide the payload boundary. The digest is deterministic: sorted relative file paths and file contents are included with separators; symlink inputs are rejected. Processing uses the immutable snapshot, not a mutable live inbox. The transaction records its version, identity, digest, creation time and repository/worktree context.
- command_inbox_publish(args) validates transaction identity, receipt location and digest, explicit publication paths and the allowed changed-file set. It stages only approved paths, commits only when there are staged changes, and delegates to the guarded publication/rebase/push path. It does not treat arbitrary worktree changes as eligible publication.
- load_transaction(value), write_local_receipt(...) and clear_payload(inbox) validate transaction metadata, record local outcome evidence, and only clear the live payload behind the digest fence.
- command_finalize(args) requires that the generated receipt is present in the canonical master HEAD and contains the expected digest. It compares the current live inbox digest with the transaction digest before clearing anything; changed input is preserved, and the processed snapshot/receipt are retained. Finalization without a published receipt fails closed.
- command_abort(args) records a failed snapshot and receipt while preserving live input. The skill's lifecycle rule is that abort is for pre-publication failure; after a generated inbox commit exists, resume/publish must be used rather than discarding a committed transaction.
- The unarchive command is a separate recovery route for a previously archived inbox payload. It is not a substitute for the digest-bound active transaction.

The state machine is therefore inbox-ensure -> prepare or validate/resume one .processing snapshot -> classify and assign -> write tracked task/state/receipt artifacts -> validate -> inbox-publish explicit paths -> verify canonical receipt and unchanged live digest -> finalize. Failure paths preserve the input and transaction for recovery; the helper is not a generic queue and not a product Inbox implementation.

## video.py: media inspection and temporary-resource lifecycle

The helper is 293 lines and provides local probe, extract and sheet commands. It is invoked by the skill when a supplied video is an input; workspace.py does not directly call video.py.

- probe uses ffprobe metadata and frame timestamps, computes SHA-256 and byte size for the input, and reports codec/dimensions/frame-rate/duration/frame timing. Timestamp-based selection is necessary for variable-frame-rate input; nominal FPS alone is not authoritative.
- extract validates requested frame selections and crop geometry, builds an ffmpeg filter chain, uses passthrough frame timing, verifies the decoded frame count, and names extracted frames with source-time provenance. Invalid ranges, unavailable tools and count mismatches are failures, not partial-success claims.
- sheet uses Pillow to assemble a bounded contact sheet from the extracted frames, with explicit frame selection/step and a maximum of 200 displayed frames. Decoded temporary frames are created in a temporary directory and removed when that scope exits; retained evidence is the requested output sheet/frame data, not an uncontrolled accumulation of temp files.
- ffmpeg and ffprobe must be available on PATH; Pillow is required for contact-sheet generation. The helper operates on supplied local media and does not introduce network transfer, upload, a persistent media store, or a new product Resource owner.

## Strict inventory index 53 — video.py responsibility and owner disposition

Exact inventory anchor: `telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844:.agents/skills/process-inbox/scripts/video.py` blob `977d75aca7aea0b6c74471d533a5ee0e2a156ac3` (293 lines, mode `100644`). This is a distinct strict source entry from index 52's four-line OpenAI registration metadata and index 54's executable workspace transaction helper.

- Responsibility symbols: `frame_times`, `sha256`, `parse_frames`, `parse_crop`, `filter_chain`, `decode`, `command_probe`, `command_extract`, `command_sheet`, and `main`.
- Real invocation chain: the shared `SKILL.md` invokes `video.py probe/extract/sheet` only when supplied video input needs inspection; `workspace.py` does not directly call `video.py`. The skill retains the input in the digest-bound transaction and uses the workspace helper for publication/finalization.
- State and lifecycle: `probe` reads local media and reports SHA-256, byte size, codec/dimensions/duration, frame count and per-frame timing; `extract` validates selected frame indices/crop, invokes ffmpeg with passthrough frame timing, verifies the decoded frame count, and names outputs with frame/time provenance; `sheet` uses Pillow for a contact sheet capped at 200 frames. Intermediate decoded frames live in `TemporaryDirectory`; explicit extracted frames/sheets are retained at caller-selected paths. There is no upload, network dependency, or persistent product resource store.
- Failure boundaries: missing ffprobe/ffmpeg/Pillow, no usable frame stream, invalid frame range/crop, and decode-count mismatch are failures. Empty selections are not explicitly rejected in this upstream implementation and can reach an unhandled `IndexError` in extract/sheet; a local equivalent would need explicit negative coverage rather than inheriting this accident silently.
- Existing-owner decision: use the existing source-neutral repository task/evidence workflow, anchored by the unique `Context provenance and handoff evidence` contract in `projects/fabushi-communication-platform/quality/evidence-contract.md`. Product Resource/attachment owners, Agent/Coordinator/Host/Runner/Conversation/Computer, and a new video/media root are rejected because this is repository tooling, not a shipping media capability. Current-main search found no local ffprobe/video helper; no equivalent helper or focused behavior tests are claimed. The ledger row is `TDRP-R9-PROCESS-INBOX-VIDEO-HELPER-001`, with `coverage_disposition=toolchain-equivalent` and `implementation_status=mapped` only.

Index 54 `.agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c` and index 55 `.agents/skills/process-inbox/scripts/workspace_test.py@d045c95ca4415ffc02953ff02ac10e329dc69e7b` remain separate source entries and are not closed by this mapping.

## Fabushi existing-owner mapping

The live main branch at ba9ecce728ef021fee609796c8704d8af6fe4b5a contains no .agents skill-registration tree or local parser for this OpenAI YAML. The exact existing target anchors were checked on both the current PR tree and live main:

- PR branch AGENTS.md blob 57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b and live main AGENTS.md blob 9ebe5fabd9d37075620b3712d79edb59fe0cb3f3: Host-native skill registration metadata, command aliases, and default prompts are invocation surfaces only.
- PR branch TEST_STRATEGY.md blob 0d3a4c993784625f9b70279602c655d4788fe9fb and live main blob b00d4b5bd911d8a6b6202253527e830efbfa5f86: Native skill-surface qualification is intentionally smaller than scanner qualification.
- PR branch evidence-contract.md blob 2d1d709a6ac3204bd05bedf5b3e98f37856e8c70 and live main blob b1c3b67fef2ddb0d0a017700600c2395aeeac7a7: Native skill-registration evidence records the host-visible metadata, default prompt, existing source-neutral workflow owner and routing contract.

The selected target is existing source-neutral repository task/review/evidence governance. Shipping Task, Agent/Coordinator, Host/Runner, Conversation, Automations, Computer and Marketplace are rejected as owners because the source YAML is repository-tooling metadata, not a product capability. No new owner or product route is introduced. The source helper's worktree/digest/media behavior is documented as source authority only; this dossier does not claim an equivalent local Fabushi helper or direct behavior tests.

The PR remains based on 2bfc3fd3638596b37a01c54e17a333c1798414f3 while live main is ba9ecce728ef021fee609796c8704d8af6fe4b5a. The target anchors are present in the PR tree and were rechecked against live main, but merge-base reconciliation remains a release-acceptance blocker.

## Evidence and open gates

Parent exact-head TDRP run 39555252098 at 90eee7251784f938ef3a7ef87713c1c2ee4fe0d1 completed with 61/61 jobs and 508/508 successful steps. Rust desktop runtime run 39555252118 completed with 6/6 jobs and 81/81 successful steps. The root source archive ZIP artifact 12324764539 was downloaded and SHA-256 checked against its GitHub digest. The archive report had correct top-level target/upstream/lock provenance and correct source identity for the Boost and libiconv archives, but each archive record lacked lock_commit and lock_tree. The next commit adds those per-record fields and fail-closed assertions; only its own exact-head Actions can validate the fix.

After mapping strict indices 52-53, counters are unknown=16249, unread=16224, omitted=0, with 71 responsibility rows and zero unmapped responsibility rows. The video helper mapping is source/toolchain disposition only; it does not claim a local helper, behavior tests, or shipping media implementation. The child exact-head TDRP run 39557717865 at 3ce96c7055646c5600f93a24bc17f6394bfa1c8c failed at validate-tdrp-revision9-authority.mjs:95 because lock minima 16251/16226 exceeded ledger counters 16250/16225; root source archive accounting job 119081364795 and Boost source archive build reachability job 119081364793 were skipped, so the archive script's per-record lock provenance fix remains unverified. Rust desktop runtime run 39557717817 was still in progress at the last poll. The previous-head archive artifact 12324632136 ZIP SHA-256 5fdfd5d7ee3a42d5133c90f853fffa08a0266c9531e05c3d51259fe9837d5286 matched its GitHub digest, but the two archive records had null lock_commit/lock_tree. The old job 119035504866 in run 39543230323 remains API-inconsistent (status=in_progress, conclusion=success, completed_at set) and is not terminal evidence. The current upstream identity remains unaccepted and recursive disposition remains open; baseline_identity_accepted=false and source_closure_ready=false. The PR base still reports 2bfc3fd3638596b37a01c54e17a333c1798414f3 while live main is ba9ecce728ef021fee609796c8704d8af6fe4b5a; reconcile live main before independent release acceptance. Next strict inventory entry is index 54 .agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c; index 55 .agents/skills/process-inbox/scripts/workspace_test.py@d045c95ca4415ffc02953ff02ac10e329dc69e7b remains separately open.


## Traceability anchors for the process-inbox skill entry

These IDs bind the strict `.agents/skills/process-inbox/SKILL.md` source entry to the repository-only transaction/evidence behavior already described above. They do not convert process-inbox into a shipping Fabushi Inbox/Task runtime, do not close the independently inventoried `workspace.py` or `workspace_test.py` entries, and do not claim current-head verification.

- `TDRP-R9-PROCESS-INBOX-001`: the source responsibility is the fail-closed repository intake/publication lifecycle: immutable payload snapshot and digest, one resumable active transaction, explicit approved publication paths, canonical receipt verification, and digest-fenced finalize/abort behavior. The selected Fabushi owner remains the existing source-neutral repository task/review/evidence governance documented in this dossier and `projects/fabushi-communication-platform/quality/evidence-contract.md`.
- `ORA-TDRP-PROCESS-INBOX-DIGEST-TRANSACTION-001`: for a fixed payload, the transaction snapshot/digest is deterministic over sorted relative paths and bytes; symlink or ambiguous input is rejected, processing reads the immutable snapshot, and later live-inbox mutation cannot silently redefine the accepted transaction.
- `ORA-TDRP-PROCESS-INBOX-RECEIPT-PUBLICATION-001`: publication succeeds only for the transaction-bound receipt and explicitly approved changed paths, and finalization requires that receipt to exist in canonical history with the expected digest; missing/mismatched receipt or changed live input remains recoverable failure rather than success.
- `INV-TDRP-PROCESS-INBOX-SINGLE-ACTIVE-TRANSACTION-001`: at most one valid active `.processing` transaction may be resumed for the repository/checkout identity; multiple active candidates fail closed instead of choosing one implicitly.
- `INV-TDRP-PROCESS-INBOX-RECEIPT-DIGEST-FENCE-001`: no finalize/clear operation may remove live inbox input unless the published canonical receipt and current live payload both match the accepted transaction digest; changed input is preserved.

The validator may use these exact anchors for bidirectional traceability. Execution evidence remains separate: the current PR must pass its own exact-head TDRP source-authority workflow before any of these IDs can contribute to a verified disposition.


## Exact-head owner uniqueness correction — 1d0439cd7122824dc9ccd84ff9b5f883fe3a6050

At live upstream `5c674f1f6a0f6ace55b67f8bc295138680fe588b`, strict index 52 remains byte-identical at `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`. PR #154 is based on live main `ba9ecce728ef021fee609796c8704d8af6fe4b5a`.

The validator exposed a target-owner collision with earlier native-registration responsibilities. For `TDRP-R9-PROCESS-INBOX-OPENAI-INTERFACE-001`, AGENTS.md and TEST_STRATEGY.md remain supporting policy candidates, while the unique target symbol is narrowed to the evidence-contract boundary `Provider-specific $skill, slash-command, or adapter spelling is provenance only.` This exactly captures the four-line OpenAI YAML's applicable responsibility without assigning a shared target symbol to multiple rows.

No production runtime was changed. Recursive accounting remains 16,306 total, unknown=16,250, unread=16,225, omitted=0, with `baseline_identity_accepted=false` and `source_closure_ready=false`.


## Strict index 54 — workspace.py complete read at live dev 5c674f1f6a0f6ace55b67f8bc295138680fe588b

Authority: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`. Strict index 54 is `.agents/skills/process-inbox/scripts/workspace.py@5d74a11083d66f2ebe9bbf0fcf2e991fa2c6c1c5` (5,696 lines). The full file was read and contains exactly 212 unique top-level functions. Adjacent strict index 55 `workspace_test.py@6acfeadb6f9128a8b5c164d7c0265b03db1fee44` contains 127 unique test methods; those tests were read as dependency evidence only and index 55 remains independently open.

The applicable responsibility is repository-development workflow control, not a shipping Fabushi Inbox, Task, Automation, Conversation, Agent, Host or Runner runtime. Existing-owner-first maps it to Fabushi's source-neutral repository task/review/evidence governance; no source-named product runtime, second queue owner, second task store or product route is introduced.

### Live-dev delta: checkout-specific source mirroring

The prior workspace blob had 211 functions. Live dev adds `mirror_source_head`: after `command_source_commit` validates ownership, stages and commits only allowed paths, validates the task commit and optionally marks source green, it mirrors HEAD to `private:refs/heads/ai-<checkout_tag>` using a bounded 120-second non-interactive force push. Missing tag/remote is an explicit skip. Timeout or remote rejection is mirror-failure evidence and does not roll back the already-created source commit. New focused tests `test_source_commit_force_mirrors_head_to_the_checkout_branch` and `test_source_commit_keeps_the_commit_when_the_mirror_is_unavailable` prove branch replacement and commit survival/worktree cleanliness on mirror failure.

### 212-function responsibility decomposition

Each function appears exactly once in one mutually exclusive domain.

### A repository/worktree configuration, task state, queue, batch, split and consolidation preparation

- `run_git` — line 109
- `git_root` — line 121
- `absolute_git_dir` — line 126
- `source_root` — line 134
- `read_machine_tag` — line 142
- `repository_config` — line 156
- `linked_worktree` — line 208
- `worktree_config` — line 237
- `inbox_worktree_config` — line 250
- `ensure_clean` — line 263
- `sync_local_slot` — line 269
- `sync_inbox_worktree` — line 278
- `sync_branch_worktree` — line 287
- `routing_worktree_config` — line 309
- `sync_routing_worktree` — line 322
- `sync_routing_canonical` — line 331
- `routing_unpublished_count` — line 336
- `health_worktree_config` — line 351
- `health_unpublished_count` — line 365
- `publish_routing` — line 376
- `has_origin` — line 386
- `origin_master_exists` — line 390
- `retryable_push_failure` — line 401
- `sync_canonical` — line 406
- `state_paths` — line 418
- `task_id_for_state` — line 425
- `state_path` — line 436
- `parse_scalar` — line 445
- `parse_dependencies` — line 454
- `load_state` — line 467
- `load_states` — line 535
- `format_scalar` — line 542
- `update_state` — line 548
- `task_ready` — line 580
- `queue_sort_key` — line 587
- `changed_paths` — line 596
- `is_linked_worktree` — line 609
- `indexed_gitlinks` — line 642
- `source_changed_paths` — line 652
- `source_local_changes` — line 674
- `mismatched_submodules` — line 692
- `ensure_source_clean` — line 701
- `registered_nested_worktrees` — line 711
- `protect_worktrees_from_submodule_target` — line 732
- `update_source_submodules` — line 753
- `prepare_source` — line 767
- `command_source_prepare` — line 782
- `path_is_stageable` — line 792
- `unpublished_counts` — line 799
- `task_relative_dir` — line 810
- `source_task_ref` — line 814
- `resolved_ref` — line 820
- `ensure_changes_scoped` — line 831
- `update_main_from_origin` — line 844
- `sync_inbox_canonical` — line 854
- `publish_worktree` — line 859
- `publish_slot` — line 902
- `publish_inbox` — line 912
- `commit_paths` — line 921
- `task_summary` — line 931
- `normalized_task_name` — line 942
- `superseded_paths` — line 946
- `split_paths` — line 953
- `retained_task_digest` — line 960
- `load_superseded` — line 1003
- `load_splits` — line 1055
- `resolve_split_targets` — line 1132
- `split_task_summary` — line 1165
- `resolve_task_reference` — line 1192
- `resolve_task_id` — line 1215
- `resolve_task` — line 1234
- `pending_consolidations` — line 1287
- `batch_relative_path` — line 1306
- `task_record_status` — line 1312
- `batch_task_order` — line 1328
- `write_batch` — line 1348
- `load_batch` — line 1363
- `file_newline` — line 1388
- `command_queue` — line 1392
- `command_resolve` — line 1496

### B task/source lineage validation, source refs, retry/checkpoint and source lifecycle

- `command_start` — line 1526
- `command_source_lineage` — line 1644
- `command_retry` — line 1658
- `task_action_config` — line 1722
- `command_checkpoint` — line 1738
- `task_commit_matches` — line 1750
- `validate_task_commit` — line 1761
- `task_series_refs` — line 1769
- `task_tip_commits` — line 1795
- `task_has_retained_source_change` — line 1816
- `source_lineage_report` — line 1835
- `is_ancestor` — line 1903
- `task_type` — line 1914
- `validate_source_state` — line 1918
- `ensure_no_persisted_commit_hashes` — line 1958
- `delete_source_refs` — line 1974
- `transfer_source_refs` — line 1983
- `command_source_begin` — line 2002
- `mark_source_green` — line 2064
- `command_source_mark_green` — line 2077

### C test-account/process ownership, build-lock recovery, console-idle gating and test execution/cleanup

- `run_git_binary` — line 2086
- `literal_paths` — line 2100
- `resolved_exe` — line 2108
- `app_bundle_of` — line 2115
- `portable_root_for` — line 2122
- `background_launch_bundle` — line 2132
- `unique_destination` — line 2150
- `move_stale_leftover` — line 2161
- `clear_stale_crash_state` — line 2177
- `golden_test_account` — line 2205
- `setup_test_account` — line 2212
- `reset_broken_test_account` — line 2232
- `processes_with_executable` — line 2242
- `kill_processes_with_executable` — line 2286
- `executable_killed_on_abort` — line 2305
- `windows_process_records` — line 2325
- `locking_process_ids` — line 2364
- `normalized_path_text` — line 2468
- `recoverable_build_processes` — line 2472
- `terminate_process_ids` — line 2516
- `path_inside` — line 2549
- `build_root_matches_source` — line 2557
- `command_build_lock_recover` — line 2571
- `parse_test_log` — line 2671
- `log_marks_complete` — line 2697
- `tail_of_file` — line 2712
- `parse_env_values` — line 2719
- `console_unsupported_reason` — line 2731
- `console_command_output` — line 2737
- `plist_dicts` — line 2756
- `console_idle_seconds` — line 2762
- `console_screen_locked` — line 2778
- `console_frontmost_app` — line 2798
- `console_value` — line 2818
- `console_input_reading` — line 2826
- `wait_for_console_idle` — line 2844
- `command_test_run` — line 2882
- `command_test_cleanup` — line 3100
- `command_test_account_reset` — line 3114

### D test overlay, source commit/mirror, commit verification, fences, preflight and result validation

- `overlay_work_dir` — line 3125
- `read_overlay_paths` — line 3132
- `initialized_submodule_paths` — line 3146
- `overlay_inventory_groups` — line 3164
- `overlay_coverage` — line 3202
- `overlay_outside_inventory` — line 3213
- `clear_overlay_submodule_bundle` — line 3234
- `read_overlay_submodule_bundle` — line 3243
- `command_overlay_save` — line 3280
- `command_overlay_apply` — line 3403
- `gitlink_paths` — line 3465
- `mirror_source_head` — line 3474
- `command_source_commit` — line 3516
- `command_source_verify_commit` — line 3582
- `file_sha256` — line 3596
- `command_fence_create` — line 3607
- `command_fence_check` — line 3628
- `command_source_preflight` — line 3660
- `required_result_value` — line 3698
- `validate_outcome_result` — line 3711
- `validate_blocked_result` — line 3736
- `owned_source_paths` — line 3790
- `source_worktree_snapshot` — line 3801
- `validate_carried_worktree` — line 3883
- `reconcile_split_source_refs` — line 3901
- `validate_split_required_result` — line 3928

### E finish/publish/approval, split/consolidation, archive, routing and health publication

- `write_carried_work` — line 3968
- `command_finish` — line 3979
- `recover_approval_source_refs` — line 4079
- `command_publish` — line 4116
- `normalized_publish_path` — line 4142
- `path_is_covered` — line 4165
- `parse_consolidation_mapping` — line 4169
- `validate_dependency_graph` — line 4183
- `validate_superseded_graph` — line 4218
- `validate_split_graph` — line 4249
- `validate_split_tree` — line 4267
- `approval_task_for_head` — line 4348
- `split_validation_for_head` — line 4357
- `write_split_record` — line 4377
- `command_split_publish` — line 4393
- `validate_consolidation_tree` — line 4538
- `consolidation_validation_for_head` — line 4632
- `command_task_content_digest` — line 4681
- `command_consolidate_publish` — line 4691
- `validate_receipt_text` — line 4851
- `command_inbox_publish` — line 4858
- `rewrite_project_links` — line 4941
- `project_last_activity` — line 4952
- `command_archive_stale` — line 4972
- `unarchive_project` — line 5014
- `command_unarchive` — line 5042
- `command_inbox_unarchive` — line 5047
- `payload_entries` — line 5052
- `iter_payload_files` — line 5059
- `payload_digest` — line 5073
- `copy_payload` — line 5089
- `unique_backup_name` — line 5100
- `command_ensure` — line 5109
- `command_route_ensure` — line 5114
- `command_route_publish` — line 5128
- `command_health_ensure` — line 5198
- `command_health_publish` — line 5218
- `command_inbox_ensure` — line 5265

### F inbox transaction preparation, digest/receipt/finalize/abort and CLI dispatch

- `command_prepare` — line 5270
- `load_transaction` — line 5331
- `write_local_receipt` — line 5343
- `clear_payload` — line 5354
- `command_finalize` — line 5363
- `command_abort` — line 5407
- `add_common_arguments` — line 5417
- `parse_args` — line 5423
- `main` — line 5684

### workspace_test.py dependency evidence — 127 test methods; strict index 55 remains open

- `test_rejects_explicit_commit_references` — line 25
- `test_preserves_runtime_values_and_content_digests` — line 48
- `test_inbox_publication_paths_must_be_specific` — line 367
- `test_only_non_fast_forward_push_failures_are_retryable` — line 390
- `test_inbox_worktree_config_does_not_create_task_slot` — line 401
- `test_inbox_lifecycle_ignores_dirty_task_slot` — line 431
- `test_inbox_publish_handles_restored_project_deletion` — line 517
- `test_inbox_publish_stages_a_removed_non_ascii_publication_path` — line 583
- `test_finalize_rejects_receipt_outside_master` — line 642
- `test_finalize_requires_receipt_from_master_head` — line 675
- `test_prepare_rejects_another_checkouts_transaction` — line 714
- `test_resolve_prefers_blocked_over_approved_history` — line 750
- `test_resolve_follows_durable_superseded_task_alias` — line 767
- `test_dependency_validation_rejects_missing_and_cyclic_edges` — line 795
- `test_queue_inventory_finds_pending_consolidation_markers` — line 810
- `test_generic_publish_recognizes_consolidation_validation` — line 822
- `test_consolidation_validation_rejects_a_non_ascii_superseded_path` — line 850
- `test_no_merge_consolidation_publishes_durable_completion` — line 904
- `test_merged_consolidation_publishes_a_removed_directory` — line 953
- `test_merged_consolidation_validates_aliases_and_dependencies` — line 1075
- `test_retry_reopens_owned_blocked_task_and_resets_routing_marker` — line 1186
- `test_start_atomically_assigns_unclaimed_todo` — line 1221
- `test_start_refuses_source_dependency_absent_from_branch` — line 1260
- `test_source_lineage_finds_compatible_local_branch` — line 1303
- `test_checkpoint_updates_only_local_task_state` — line 1393
- `test_normal_lifecycle_publishes_only_start_and_approve` — line 1415
- `test_retry_refuses_to_compete_with_active_task` — line 1502
- `test_retained_task_commit_can_precede_later_work` — line 1539
- `test_source_begin_initializes_new_task_without_a_green_ref` — line 1587
- `test_archive_stale_moves_only_old_fully_approved_projects` — line 1616
- `test_archive_stale_without_candidates_reports_nothing` — line 1663
- `test_unarchive_restores_links_and_stages_the_project` — line 1682
- `test_linked_worktrees_are_ignored_only_by_source_checks` — line 2063
- `test_unregistered_repositories_and_moved_worktrees_still_block` — line 2089
- `test_symlink_to_linked_worktree_is_not_ignored` — line 2105
- `test_nested_module_worktree_does_not_make_parent_dirty` — line 2114
- `test_preparation_updates_recursive_recorded_pins_without_following_remote` — line 2134
- `test_preparation_preserves_local_and_staged_changes_before_any_update` — line 2171
- `test_registered_worktree_at_gitlink_path_is_never_updated` — line 2202
- `test_submodule_target_cannot_enter_an_existing_linked_worktree` — line 2224
- `test_preflight_is_read_only_and_fresh_baseline_updates_modules` — line 2259
- `test_source_lifecycle_overlay_and_carry_checks_ignore_linked_worktree` — line 2278
- `test_build_lock_recovery_selects_only_exact_owned_processes` — line 2310
- `test_build_lock_recovery_deletes_only_named_build_artifacts` — line 2375
- `test_build_lock_recovery_rejects_artifact_outside_build_tree` — line 2450
- `test_parse_env_values_requires_name_value_pairs` — line 2478
- `test_setup_test_account_lifecycle` — line 2488
- `test_test_run_reports_complete_markers` — line 2531
- `test_test_run_redirects_the_launch_for_a_portable_root` — line 2546
- `test_test_run_launches_without_activation_by_default` — line 2590
- `test_test_run_activate_lets_the_launch_activate_the_client` — line 2620
- `test_test_run_refuses_an_env_that_sets_the_activation_variable` — line 2651
- `test_background_launch_bundle_needs_macos_and_the_bundle_executable` — line 2670
- `test_test_run_launches_a_macos_bundle_in_the_background` — line 2704
- `test_test_run_activate_keeps_the_direct_launch_for_a_macos_bundle` — line 2766
- `test_test_run_reports_a_failed_background_launch` — line 2800
- `test_test_run_ends_a_background_client_by_path` — line 2818
- `test_test_run_ends_a_background_client_when_interrupted` — line 2849
- `test_parse_test_log_lists_skipped_rows_beside_pass_and_fail` — line 2928
- `test_log_marks_complete_reads_the_marker_as_a_whole_line` — line 2942
- `test_test_run_reports_a_death_after_complete` — line 2964
- `test_test_run_reports_both_death_signals_after_complete` — line 2989
- `test_test_run_counts_a_crashpad_dump_written_during_the_run` — line 3009
- `test_test_run_counts_a_breakpad_dump_written_during_the_run` — line 3029
- `test_test_run_ignores_a_crashpad_dump_from_before_the_run` — line 3049
- `test_test_run_reports_crash_diagnostics` — line 3073
- `test_test_run_still_diagnoses_a_crash_after_clearing_stale_state` — line 3099
- `test_test_run_kills_on_deadline` — line 3150
- `test_test_run_keeps_a_grace_kill_complete` — line 3164
- `test_test_run_refuses_a_line_that_only_contains_the_marker` — line 3184
- `test_test_run_reports_a_grace_kill_with_a_dump` — line 3216
- `test_test_run_clears_and_preserves_stale_crash_state` — line 3235
- `test_test_run_without_leftovers_reports_nothing_cleared` — line 3294
- `test_test_run_leaves_an_unmarked_live_folder_alone` — line 3307
- `test_test_run_ignores_a_stale_crash_report_copied_from_golden` — line 3352
- `test_test_run_refuses_to_launch_when_the_stale_report_cannot_move` — line 3379
- `test_test_run_reports_a_dump_that_could_not_be_moved` — line 3398
- `test_test_run_discards_a_partial_report_copy_from_a_failed_move` — line 3443
- `test_test_run_reports_input_during_the_run` — line 3469
- `test_test_run_reports_no_input_when_idle_covers_the_run` — line 3502
- `test_test_run_leaves_input_unknown_when_a_reading_fails` — line 3518
- `test_test_run_gate_launches_once_the_console_is_idle` — line 3603
- `test_test_run_gate_launches_after_its_bound_when_never_idle` — line 3645
- `test_test_run_skips_readings_and_gate_off_macos` — line 3671
- `test_test_run_parses_the_screen_lock_state` — line 3706
- `test_test_run_rejects_a_bad_idle_gate` — line 3722
- `test_test_run_gate_fails_before_waiting_without_a_golden` — line 3745
- `test_portable_root_for_prefers_app_bundle_parent` — line 3761
- `test_unique_destination_avoids_an_existing_name` — line 3781
- `test_overlay_save_and_apply_roundtrip` — line 3799
- `test_overlay_apply_reports_a_non_ascii_conflict_literally` — line 3842
- `test_overlay_save_rejects_uninventoried_and_untracked_paths` — line 3892
- `test_overlay_save_preserves_bundle_when_repeat_save_is_empty` — line 3929
- `test_overlay_save_preserves_bundle_when_candidate_collection_fails` — line 3987
- `test_overlay_save_preserves_bundle_when_patch_does_not_verify` — line 4041
- `test_overlay_save_replacement_removes_obsolete_nested_entries` — line 4105
- `test_overlay_save_nested_only_replacement_unlinks_root_patch` — line 4167
- `test_overlay_save_inventory_refusal_preserves_existing_bundle` — line 4223
- `test_source_commit_stages_owned_paths_and_marks_green` — line 4266
- `test_source_commit_force_mirrors_head_to_the_checkout_branch` — line 4316
- `test_source_commit_keeps_the_commit_when_the_mirror_is_unavailable` — line 4364
- `test_source_commit_stages_a_removed_tracked_file` — line 4400
- `test_source_commit_stages_a_non_ascii_tracked_file` — line 4430
- `test_source_commit_rejects_paths_outside_owned_set` — line 4481
- `test_already_satisfied_source_state_requires_baseline` — line 4515
- `test_adaptive_outcome_result_contract` — line 4531
- `test_test_block_requires_real_recovery_exhaustion` — line 4573
- `test_task_type_defaults_to_implementation` — line 4628
- `test_fence_create_and_check_detects_changes` — line 4652
- `test_source_preflight_reports_dirty_and_account_state` — line 4686
- `test_finish_publishes_split_required_with_carried_work` — line 4713
- `test_carried_work_snapshot_seals_owned_submodule_changes` — line 4770
- `test_split_publish_routes_and_starts_implementation_carrier` — line 4801
- `test_publish_recovers_approval_refs_after_fetch_failure` — line 4951
- `test_publish_preserves_approval_refs_for_unsafe_source` — line 5016
- `test_publish_cleans_refs_when_approval_landed_before_interruption` — line 5058
- `test_publish_never_cleans_refs_without_owned_approval` — line 5114
- `test_start_writes_the_batch_into_its_start_commit` — line 5300
- `test_start_rejects_unknown_batch_ids_before_changing_anything` — line 5346
- `test_start_without_a_batch_marks_only_the_started_task` — line 5365
- `test_a_batch_marks_tasks_without_claiming_them` — line 5377
- `test_a_lost_start_leaves_no_batch_published` — line 5432
- `test_queue_lists_an_absent_batch_as_empty` — line 5462
- `test_batch_option_collects_every_id_in_order` — line 5467
- `test_reading_a_batch_ignores_what_it_cannot_use` — line 5485
- `test_second_run_of_the_day_is_dropped` — line 5567
- `test_only_code_health_paths_are_published` — line 5592

### Traceability anchors for strict index 54

- `TDRP-R9-PROCESS-INBOX-WORKSPACE-001`: repository-only worktree/task/source/test/inbox lifecycle and publication mechanics map to existing source-neutral development/evidence governance, not a shipping product owner.
- `ORA-TDRP-WORKSPACE-OWNED-PUBLICATION-001`: publication stages/commits/pushes only the explicit owned write set and preserves dirty-path, gitlink, lineage and ambiguous-state failures.
- `ORA-TDRP-WORKSPACE-SOURCE-COMMIT-MIRROR-001`: source mirroring is a separate bounded outcome after a validated commit; skip/timeout/failure cannot erase or reinterpret that commit.
- `ORA-TDRP-WORKSPACE-TEST-RECOVERY-001`: build-lock/process/test recovery acts only on positively identified owned state; timeout/cancellation/incomplete evidence remains non-passing.
- `INV-TDRP-WORKSPACE-NO-SHIPPING-OWNER-COLLISION-001`: repository workflow state never becomes a second shipping Task/Automation/Conversation/Agent/Host/Runner owner.
- `INV-TDRP-WORKSPACE-MIRROR-FAILURE-NO-COMMIT-ROLLBACK-001`: mirror failure must not roll back the source commit, dirty the worktree, or fabricate publication success.

Closing only index 54 changes counters to unknown=16,249, unread=16,224, omitted=0, responsibilities=72. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. The next strict entry is index 55 `.agents/skills/process-inbox/scripts/workspace_test.py@6acfeadb6f9128a8b5c164d7c0265b03db1fee44`.


## Strict index 55 — workspace_test.py complete read at live dev 5c674f1f6a0f6ace55b67f8bc295138680fe588b

Authority: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`. Strict index 55 is `.agents/skills/process-inbox/scripts/workspace_test.py@6acfeadb6f9128a8b5c164d7c0265b03db1fee44` (5602 lines). The full file was read. It contains exactly 127 unique `test_*` methods across six `unittest.TestCase` classes: CommitHashArtifactTests=2, WorkspaceTest=30, SourcePreparationTest=10, MechanicsTest=75, BatchMarkTest=8, HealthWorktreeTest=2. The method accounting below is one-to-one: every test method appears exactly once, with no duplicate or omitted method identity.

This file is an executable regression oracle for repository-development workflow mechanics, not a shipping Fabushi product runtime. Existing-owner-first maps the applicable responsibility to Fabushi's source-neutral repository development/test/evidence governance. The suite does not justify a second Task, Automation, Conversation, Agent, Coordinator, Host, Runner, Resource, Search, account, or message owner, and the upstream Python harness is not copied into shipping composition.

### Support-fixture accounting

Top-level support functions: `task_state` (lines 75–90), `write_task` (lines 91–120), `write_project` (lines 121–131), `write_project_task` (lines 132–156), `git` (lines 157–166), `git_bytes` (lines 167–175), `git_repo` (lines 176–182), `inbox_worktrees` (lines 183–253), `set_origin_url` (lines 254–257), `task_refs` (lines 258–266), `approval_recovery_fixture` (lines 267–339), `finish_approval` (lines 340–351), `publish_approval` (lines 352–361), `head_subject` (lines 362–365), `write_fake_exe` (lines 1712–1722), `write_complete_markers_exe` (lines 1723–1740), `write_dump_after_complete_exe` (lines 1741–1757), `write_argv_recording_exe` (lines 1758–1802), `read_argv` (lines 1803–1807), `read_recorded_env` (lines 1808–1820), `make_portable_root` (lines 1821–1828), `plant_leftover_crash_state` (lines 1829–1838), `source_repo_with_task` (lines 1839–1872), `overlay_bundle_bytes` (lines 1873–1889), `overlay_repo_with_nested_module` (lines 1890–1912), `run_command` (lines 1913–1919), `run_test_run` (lines 1920–1937), `patched_background_launch` (lines 1938–1950), `lsappinfo_name_output` (lines 1951–1961), `hid_system_plist` (lines 1962–1969), `console_root_plist` (lines 1970–1989).

Non-test class methods/fixtures: `FrozenDate.today` (lines 71–365), `WorkspaceTest.record_commit` (lines 1430–1989), `FakeConsole.__init__` (lines 1991–1998), `FakeConsole.clock` (lines 1999–2001), `FakeConsole.sleep` (lines 2002–2009), `FakeConsole.output` (lines 2010–2024), `FakeConsole.installed` (lines 2025–2046), `FakeConsole.wait` (lines 2028–2046), `SourcePreparationTest.make_repo` (lines 2048–2054), `SourcePreparationTest.add_module` (lines 2055–2062), `MechanicsTest.interrupt` (lines 2850–2852), `MechanicsTest.terminate` (lines 2853–5114), `MechanicsTest.popen` (lines 2879–2882), `MechanicsTest.kill` (lines 2883–2886), `MechanicsTest.sleep` (lines 2887–5198), `MechanicsTest.move_unless_dump` (lines 3410–3453), `MechanicsTest.write_then_fail` (lines 3454–3612), `MechanicsTest.pause` (lines 3613–4014), `MechanicsTest.fail_binary_diff` (lines 4015–4068), `MechanicsTest.fail_reverse_check` (lines 4069–4445), `MechanicsTest.record_covered` (lines 4446–4579), `MechanicsTest.check` (lines 4580–5198), `MechanicsTest.replace_state` (lines 5115–5198), `BatchMarkTest.setUp` (lines 5200–5237), `BatchMarkTest.identity` (lines 5238–5241), `BatchMarkTest.task` (lines 5242–5267), `BatchMarkTest.machine` (lines 5268–5281), `BatchMarkTest.run_json` (lines 5282–5287), `BatchMarkTest.slot` (lines 5288–5291), `BatchMarkTest.published` (lines 5292–5294), `BatchMarkTest.head_paths` (lines 5295–5299), `HealthWorktreeTest.setUp` (lines 5517–5537), `HealthWorktreeTest.identity` (lines 5538–5541), `HealthWorktreeTest.machine` (lines 5542–5555), `HealthWorktreeTest.run_json` (lines 5556–5561), `HealthWorktreeTest.write` (lines 5562–5566).

These fixtures support the 127 regression oracles and are covered by the file-level responsibility. They do not create independent product capabilities.

### 127 independent test-method records

#### CommitHashArtifactTests (2 test methods)

- `001` `CommitHashArtifactTests.test_rejects_explicit_commit_references` — lines 25–47; direct subjects: `WorkspaceError`, `ensure_no_persisted_commit_hashes`. Contract: rejects explicit commit references. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `002` `CommitHashArtifactTests.test_preserves_runtime_values_and_content_digests` — lines 48–68; direct subjects: `ensure_no_persisted_commit_hashes`. Contract: preserves runtime values and content digests.

#### WorkspaceTest (30 test methods)

- `003` `WorkspaceTest.test_inbox_publication_paths_must_be_specific` — lines 367–389; direct subjects: `WorkspaceError`, `normalized_publish_path`. Contract: inbox publication paths must be specific. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `004` `WorkspaceTest.test_only_non_fast_forward_push_failures_are_retryable` — lines 390–400; direct subjects: `retryable_push_failure`. Contract: only non fast forward push failures are retryable.
- `005` `WorkspaceTest.test_inbox_worktree_config_does_not_create_task_slot` — lines 401–430; direct subjects: `inbox_worktree_config`. Contract: inbox worktree config does not create task slot.
- `006` `WorkspaceTest.test_inbox_lifecycle_ignores_dirty_task_slot` — lines 431–516; direct subjects: `command_prepare`, `command_inbox_publish`, `changed_paths`, `command_finalize`. Contract: inbox lifecycle ignores dirty task slot.
- `007` `WorkspaceTest.test_inbox_publish_handles_restored_project_deletion` — lines 517–582; direct subjects: `command_prepare`, `command_inbox_unarchive`, `command_inbox_publish`. Contract: inbox publish handles restored project deletion.
- `008` `WorkspaceTest.test_inbox_publish_stages_a_removed_non_ascii_publication_path` — lines 583–641; direct subjects: `command_prepare`, `command_inbox_publish`. Contract: inbox publish stages a removed non ascii publication path.
- `009` `WorkspaceTest.test_finalize_rejects_receipt_outside_master` — lines 642–674; direct subjects: `command_prepare`, `WorkspaceError`, `command_finalize`. Contract: finalize rejects receipt outside master. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `010` `WorkspaceTest.test_finalize_requires_receipt_from_master_head` — lines 675–713; direct subjects: `command_prepare`, `WorkspaceError`, `command_finalize`. Contract: finalize requires receipt from master head. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `011` `WorkspaceTest.test_prepare_rejects_another_checkouts_transaction` — lines 714–749; direct subjects: `WorkspaceError`, `command_prepare`. Contract: prepare rejects another checkouts transaction. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `012` `WorkspaceTest.test_resolve_prefers_blocked_over_approved_history` — lines 750–766; direct subjects: `resolve_task`. Contract: resolve prefers blocked over approved history.
- `013` `WorkspaceTest.test_resolve_follows_durable_superseded_task_alias` — lines 767–794; direct subjects: `retained_task_digest`, `load_states`, `resolve_task`. Contract: resolve follows durable superseded task alias.
- `014` `WorkspaceTest.test_dependency_validation_rejects_missing_and_cyclic_edges` — lines 795–809; direct subjects: `WorkspaceError`, `validate_dependency_graph`. Contract: dependency validation rejects missing and cyclic edges. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `015` `WorkspaceTest.test_queue_inventory_finds_pending_consolidation_markers` — lines 810–821; direct subjects: `CONSOLIDATION_PENDING`, `pending_consolidations`. Contract: queue inventory finds pending consolidation markers.
- `016` `WorkspaceTest.test_generic_publish_recognizes_consolidation_validation` — lines 822–849; direct subjects: `CONSOLIDATION_PENDING`, `CONSOLIDATION_COMPLETE`, `consolidation_validation_for_head`. Contract: generic publish recognizes consolidation validation.
- `017` `WorkspaceTest.test_consolidation_validation_rejects_a_non_ascii_superseded_path` — lines 850–903; direct subjects: `CONSOLIDATION_PENDING`, `CONSOLIDATION_COMPLETE`, `superseded_paths`, `WorkspaceError`, `consolidation_validation_for_head`. Contract: consolidation validation rejects a non ascii superseded path. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `018` `WorkspaceTest.test_no_merge_consolidation_publishes_durable_completion` — lines 904–952; direct subjects: `CONSOLIDATION_PENDING`, `CONSOLIDATION_COMPLETE`, `command_consolidate_publish`. Contract: no merge consolidation publishes durable completion.
- `019` `WorkspaceTest.test_merged_consolidation_publishes_a_removed_directory` — lines 953–1074; direct subjects: `CONSOLIDATION_PENDING`, `CONSOLIDATION_COMPLETE`, `retained_task_digest`, `command_consolidate_publish`. Contract: merged consolidation publishes a removed directory.
- `020` `WorkspaceTest.test_merged_consolidation_validates_aliases_and_dependencies` — lines 1075–1185; direct subjects: `CONSOLIDATION_COMPLETE`, `retained_task_digest`, `validate_consolidation_tree`, `WorkspaceError`. Contract: merged consolidation validates aliases and dependencies. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `021` `WorkspaceTest.test_retry_reopens_owned_blocked_task_and_resets_routing_marker` — lines 1186–1220; direct subjects: `CONSOLIDATION_PENDING`, `CONSOLIDATION_COMPLETE`, `command_retry`, `load_state`. Contract: retry reopens owned blocked task and resets routing marker.
- `022` `WorkspaceTest.test_start_atomically_assigns_unclaimed_todo` — lines 1221–1259; direct subjects: `command_start`, `load_state`. Contract: start atomically assigns unclaimed todo.
- `023` `WorkspaceTest.test_start_refuses_source_dependency_absent_from_branch` — lines 1260–1302; direct subjects: `WorkspaceError`, `command_start`, `load_state`. Contract: start refuses source dependency absent from branch. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `024` `WorkspaceTest.test_source_lineage_finds_compatible_local_branch` — lines 1303–1392; direct subjects: `source_lineage_report`. Contract: source lineage finds compatible local branch.
- `025` `WorkspaceTest.test_checkpoint_updates_only_local_task_state` — lines 1393–1414; direct subjects: `command_checkpoint`, `load_state`. Contract: checkpoint updates only local task state.
- `026` `WorkspaceTest.test_normal_lifecycle_publishes_only_start_and_approve` — lines 1415–1501; direct subjects: `command_start`, `command_checkpoint`, `command_finish`, `load_state`. Contract: normal lifecycle publishes only start and approve.
- `027` `WorkspaceTest.test_retry_refuses_to_compete_with_active_task` — lines 1502–1538; direct subjects: `WorkspaceError`, `command_retry`. Contract: retry refuses to compete with active task. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `028` `WorkspaceTest.test_retained_task_commit_can_precede_later_work` — lines 1539–1586; direct subjects: `task_series_refs`, `command_source_begin`, `resolved_ref`, `source_task_ref`, `validate_source_state`, `delete_source_refs`. Contract: retained task commit can precede later work.
- `029` `WorkspaceTest.test_source_begin_initializes_new_task_without_a_green_ref` — lines 1587–1615; direct subjects: `command_source_begin`, `resolved_ref`, `source_task_ref`. Contract: source begin initializes new task without a green ref.
- `030` `WorkspaceTest.test_archive_stale_moves_only_old_fully_approved_projects` — lines 1616–1662; direct subjects: `datetime`, `command_archive_stale`. Contract: archive stale moves only old fully approved projects.
- `031` `WorkspaceTest.test_archive_stale_without_candidates_reports_nothing` — lines 1663–1681; direct subjects: `command_archive_stale`. Contract: archive stale without candidates reports nothing.
- `032` `WorkspaceTest.test_unarchive_restores_links_and_stages_the_project` — lines 1682–1989; direct subjects: `command_unarchive`, `PORTABLE_GOLDEN`, `OVERLAY_PATCH_FILE`, `OVERLAY_SUBMODULES_FILE`, `OVERLAY_SUBMODULES_DIR`, `source_task_ref`, `OVERLAY_PATHS_FILE`, `WAIT_IDLE_MAX_DEFAULT`, `command_test_run`. Contract: unarchive restores links and stages the project.

#### SourcePreparationTest (10 test methods)

- `033` `SourcePreparationTest.test_linked_worktrees_are_ignored_only_by_source_checks` — lines 2063–2088; direct subjects: `source_changed_paths`, `prepare_source`, `WorkspaceError`, `ensure_clean`. Contract: linked worktrees are ignored only by source checks. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `034` `SourcePreparationTest.test_unregistered_repositories_and_moved_worktrees_still_block` — lines 2089–2104; direct subjects: `source_changed_paths`, `WorkspaceError`, `prepare_source`. Contract: unregistered repositories and moved worktrees still block. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `035` `SourcePreparationTest.test_symlink_to_linked_worktree_is_not_ignored` — lines 2105–2113; direct subjects: `source_changed_paths`. Contract: symlink to linked worktree is not ignored.
- `036` `SourcePreparationTest.test_nested_module_worktree_does_not_make_parent_dirty` — lines 2114–2133; direct subjects: `literal_paths`, `source_changed_paths`, `initialized_submodule_paths`, `prepare_source`, `WorkspaceError`. Contract: nested module worktree does not make parent dirty. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `037` `SourcePreparationTest.test_preparation_updates_recursive_recorded_pins_without_following_remote` — lines 2134–2170; direct subjects: `command_source_prepare`, `prepare_source`, `source_changed_paths`, `mismatched_submodules`. Contract: preparation updates recursive recorded pins without following remote.
- `038` `SourcePreparationTest.test_preparation_preserves_local_and_staged_changes_before_any_update` — lines 2171–2201; direct subjects: `WorkspaceError`, `prepare_source`. Contract: preparation preserves local and staged changes before any update. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `039` `SourcePreparationTest.test_registered_worktree_at_gitlink_path_is_never_updated` — lines 2202–2223; direct subjects: `is_linked_worktree`, `WorkspaceError`, `prepare_source`, `initialized_submodule_paths`. Contract: registered worktree at gitlink path is never updated. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `040` `SourcePreparationTest.test_submodule_target_cannot_enter_an_existing_linked_worktree` — lines 2224–2258; direct subjects: `WorkspaceError`, `prepare_source`. Contract: submodule target cannot enter an existing linked worktree. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `041` `SourcePreparationTest.test_preflight_is_read_only_and_fresh_baseline_updates_modules` — lines 2259–2277; direct subjects: `command_source_preflight`, `command_source_begin`. Contract: preflight is read only and fresh baseline updates modules.
- `042` `SourcePreparationTest.test_source_lifecycle_overlay_and_carry_checks_ignore_linked_worktree` — lines 2278–2308; direct subjects: `command_source_begin`, `command_source_preflight`, `command_source_commit`, `source_worktree_snapshot`, `command_overlay_save`, `command_overlay_apply`, `ensure_source_clean`. Contract: source lifecycle overlay and carry checks ignore linked worktree.

#### MechanicsTest (75 test methods)

- `043` `MechanicsTest.test_build_lock_recovery_selects_only_exact_owned_processes` — lines 2310–2374; direct subjects: `recoverable_build_processes`. Contract: build lock recovery selects only exact owned processes.
- `044` `MechanicsTest.test_build_lock_recovery_deletes_only_named_build_artifacts` — lines 2375–2449; direct subjects: `command_build_lock_recover`. Contract: build lock recovery deletes only named build artifacts.
- `045` `MechanicsTest.test_build_lock_recovery_rejects_artifact_outside_build_tree` — lines 2450–2477; direct subjects: `WorkspaceError`, `command_build_lock_recover`. Contract: build lock recovery rejects artifact outside build tree. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `046` `MechanicsTest.test_parse_env_values_requires_name_value_pairs` — lines 2478–2487; direct subjects: `parse_env_values`, `WorkspaceError`. Contract: parse env values requires name value pairs. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `047` `MechanicsTest.test_setup_test_account_lifecycle` — lines 2488–2530; direct subjects: `PORTABLE_LIVE`, `PORTABLE_REAL`, `PORTABLE_GOLDEN`, `WorkspaceError`, `setup_test_account`, `PORTABLE_MARKER`, `reset_broken_test_account`. Contract: setup test account lifecycle. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `048` `MechanicsTest.test_test_run_reports_complete_markers` — lines 2531–2545; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run reports complete markers.
- `049` `MechanicsTest.test_test_run_redirects_the_launch_for_a_portable_root` — lines 2546–2589; direct subjects: `PORTABLE_LIVE`, `PORTABLE_GOLDEN`, `PORTABLE_MARKER`. Contract: test run redirects the launch for a portable root.
- `050` `MechanicsTest.test_test_run_launches_without_activation_by_default` — lines 2590–2619; direct subjects: `LAUNCH_ACTIVATION_VARIABLE`. Contract: test run launches without activation by default.
- `051` `MechanicsTest.test_test_run_activate_lets_the_launch_activate_the_client` — lines 2620–2650; direct subjects: `LAUNCH_ACTIVATION_VARIABLE`. Contract: test run activate lets the launch activate the client.
- `052` `MechanicsTest.test_test_run_refuses_an_env_that_sets_the_activation_variable` — lines 2651–2669; direct subjects: `LAUNCH_ACTIVATION_VARIABLE`, `WorkspaceError`, `PORTABLE_LIVE`. Contract: test run refuses an env that sets the activation variable. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `053` `MechanicsTest.test_background_launch_bundle_needs_macos_and_the_bundle_executable` — lines 2670–2703; direct subjects: `background_launch_bundle`. Contract: background launch bundle needs macos and the bundle executable.
- `054` `MechanicsTest.test_test_run_launches_a_macos_bundle_in_the_background` — lines 2704–2765; direct subjects: `LAUNCH_ACTIVATION_VARIABLE`, `PORTABLE_LIVE`. Contract: test run launches a macos bundle in the background.
- `055` `MechanicsTest.test_test_run_activate_keeps_the_direct_launch_for_a_macos_bundle` — lines 2766–2799; direct subjects: `LAUNCH_ACTIVATION_VARIABLE`. Contract: test run activate keeps the direct launch for a macos bundle.
- `056` `MechanicsTest.test_test_run_reports_a_failed_background_launch` — lines 2800–2817; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run reports a failed background launch.
- `057` `MechanicsTest.test_test_run_ends_a_background_client_by_path` — lines 2818–2848; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run ends a background client by path.
- `058` `MechanicsTest.test_test_run_ends_a_background_client_when_interrupted` — lines 2849–2927; direct subjects: `subprocess`, `time`. Contract: test run ends a background client when interrupted. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `059` `MechanicsTest.test_parse_test_log_lists_skipped_rows_beside_pass_and_fail` — lines 2928–2941; direct subjects: `parse_test_log`. Contract: parse test log lists skipped rows beside pass and fail.
- `060` `MechanicsTest.test_log_marks_complete_reads_the_marker_as_a_whole_line` — lines 2942–2963; direct subjects: `log_marks_complete`. Contract: log marks complete reads the marker as a whole line.
- `061` `MechanicsTest.test_test_run_reports_a_death_after_complete` — lines 2964–2988; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run reports a death after complete.
- `062` `MechanicsTest.test_test_run_reports_both_death_signals_after_complete` — lines 2989–3008; direct subjects: `PORTABLE_LIVE`, `CRASHPAD_COMPLETED_DIR`. Contract: test run reports both death signals after complete.
- `063` `MechanicsTest.test_test_run_counts_a_crashpad_dump_written_during_the_run` — lines 3009–3028; direct subjects: `PORTABLE_LIVE`, `CRASHPAD_COMPLETED_DIR`. Contract: test run counts a crashpad dump written during the run.
- `064` `MechanicsTest.test_test_run_counts_a_breakpad_dump_written_during_the_run` — lines 3029–3048; direct subjects: `PORTABLE_LIVE`. Contract: test run counts a breakpad dump written during the run.
- `065` `MechanicsTest.test_test_run_ignores_a_crashpad_dump_from_before_the_run` — lines 3049–3072; direct subjects: `setup_test_account`, `PORTABLE_LIVE`, `CRASHPAD_COMPLETED_DIR`, `STALE_CRASH_DIR`. Contract: test run ignores a crashpad dump from before the run.
- `066` `MechanicsTest.test_test_run_reports_crash_diagnostics` — lines 3073–3098; direct subjects: `PORTABLE_LIVE`. Contract: test run reports crash diagnostics.
- `067` `MechanicsTest.test_test_run_still_diagnoses_a_crash_after_clearing_stale_state` — lines 3099–3149; direct subjects: `setup_test_account`, `PORTABLE_LIVE`, `STALE_CRASH_DIR`. Contract: test run still diagnoses a crash after clearing stale state.
- `068` `MechanicsTest.test_test_run_kills_on_deadline` — lines 3150–3163; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run kills on deadline.
- `069` `MechanicsTest.test_test_run_keeps_a_grace_kill_complete` — lines 3164–3183; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run keeps a grace kill complete.
- `070` `MechanicsTest.test_test_run_refuses_a_line_that_only_contains_the_marker` — lines 3184–3215; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run refuses a line that only contains the marker.
- `071` `MechanicsTest.test_test_run_reports_a_grace_kill_with_a_dump` — lines 3216–3234; direct subjects: `PORTABLE_LIVE`, `CRASHPAD_COMPLETED_DIR`. Contract: test run reports a grace kill with a dump.
- `072` `MechanicsTest.test_test_run_clears_and_preserves_stale_crash_state` — lines 3235–3293; direct subjects: `PORTABLE_GOLDEN`, `PORTABLE_REAL`, `setup_test_account`, `PORTABLE_LIVE`, `STALE_CRASH_DIR`, `PORTABLE_MARKER`. Contract: test run clears and preserves stale crash state.
- `073` `MechanicsTest.test_test_run_without_leftovers_reports_nothing_cleared` — lines 3294–3306; direct subjects: `setup_test_account`, `STALE_CRASH_DIR`. Contract: test run without leftovers reports nothing cleared.
- `074` `MechanicsTest.test_test_run_leaves_an_unmarked_live_folder_alone` — lines 3307–3351; direct subjects: `PORTABLE_GOLDEN`, `PORTABLE_LIVE`, `PORTABLE_REAL`, `STALE_CRASH_DIR`, `PORTABLE_MARKER`. Contract: test run leaves an unmarked live folder alone.
- `075` `MechanicsTest.test_test_run_ignores_a_stale_crash_report_copied_from_golden` — lines 3352–3378; direct subjects: `PORTABLE_GOLDEN`, `PORTABLE_LIVE`. Contract: test run ignores a stale crash report copied from golden.
- `076` `MechanicsTest.test_test_run_refuses_to_launch_when_the_stale_report_cannot_move` — lines 3379–3397; direct subjects: `setup_test_account`, `PORTABLE_LIVE`, `shutil`, `WorkspaceError`. Contract: test run refuses to launch when the stale report cannot move. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `077` `MechanicsTest.test_test_run_reports_a_dump_that_could_not_be_moved` — lines 3398–3442; direct subjects: `setup_test_account`, `PORTABLE_LIVE`, `shutil`, `STALE_CRASH_DIR`. Contract: test run reports a dump that could not be moved.
- `078` `MechanicsTest.test_test_run_discards_a_partial_report_copy_from_a_failed_move` — lines 3443–3468; direct subjects: `setup_test_account`, `PORTABLE_LIVE`, `shutil`, `WorkspaceError`, `STALE_CRASH_DIR`. Contract: test run discards a partial report copy from a failed move. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `079` `MechanicsTest.test_test_run_reports_input_during_the_run` — lines 3469–3501; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run reports input during the run.
- `080` `MechanicsTest.test_test_run_reports_no_input_when_idle_covers_the_run` — lines 3502–3517; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run reports no input when idle covers the run.
- `081` `MechanicsTest.test_test_run_leaves_input_unknown_when_a_reading_fails` — lines 3518–3602; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run leaves input unknown when a reading fails.
- `082` `MechanicsTest.test_test_run_gate_launches_once_the_console_is_idle` — lines 3603–3644; direct subjects: `PORTABLE_LIVE`, `CRASHPAD_COMPLETED_DIR`, `setup_test_account`. Contract: test run gate launches once the console is idle.
- `083` `MechanicsTest.test_test_run_gate_launches_after_its_bound_when_never_idle` — lines 3645–3670; direct subjects: `PORTABLE_LIVE`. Contract: test run gate launches after its bound when never idle.
- `084` `MechanicsTest.test_test_run_skips_readings_and_gate_off_macos` — lines 3671–3705; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run skips readings and gate off macos.
- `085` `MechanicsTest.test_test_run_parses_the_screen_lock_state` — lines 3706–3721; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: test run parses the screen lock state.
- `086` `MechanicsTest.test_test_run_rejects_a_bad_idle_gate` — lines 3722–3744; direct subjects: `WorkspaceError`, `PORTABLE_LIVE`. Contract: test run rejects a bad idle gate. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `087` `MechanicsTest.test_test_run_gate_fails_before_waiting_without_a_golden` — lines 3745–3760; direct subjects: `WorkspaceError`, `PORTABLE_LIVE`. Contract: test run gate fails before waiting without a golden. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `088` `MechanicsTest.test_portable_root_for_prefers_app_bundle_parent` — lines 3761–3780; direct subjects: `portable_root_for`. Contract: portable root for prefers app bundle parent.
- `089` `MechanicsTest.test_unique_destination_avoids_an_existing_name` — lines 3781–3798; direct subjects: `unique_destination`. Contract: unique destination avoids an existing name.
- `090` `MechanicsTest.test_overlay_save_and_apply_roundtrip` — lines 3799–3841; direct subjects: `source_task_ref`, `OVERLAY_PATHS_FILE`, `command_overlay_save`, `changed_paths`, `command_overlay_apply`. Contract: overlay save and apply roundtrip.
- `091` `MechanicsTest.test_overlay_apply_reports_a_non_ascii_conflict_literally` — lines 3842–3891; direct subjects: `source_task_ref`, `OVERLAY_PATHS_FILE`, `command_overlay_save`, `command_overlay_apply`. Contract: overlay apply reports a non ascii conflict literally.
- `092` `MechanicsTest.test_overlay_save_rejects_uninventoried_and_untracked_paths` — lines 3892–3928; direct subjects: `OVERLAY_PATHS_FILE`, `WorkspaceError`, `command_overlay_save`. Contract: overlay save rejects uninventoried and untracked paths. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `093` `MechanicsTest.test_overlay_save_preserves_bundle_when_repeat_save_is_empty` — lines 3929–3986; direct subjects: `command_overlay_save`, `OVERLAY_PATCH_FILE`, `OVERLAY_SUBMODULES_FILE`, `OVERLAY_SUBMODULES_DIR`, `WorkspaceError`. Contract: overlay save preserves bundle when repeat save is empty. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `094` `MechanicsTest.test_overlay_save_preserves_bundle_when_candidate_collection_fails` — lines 3987–4040; direct subjects: `command_overlay_save`, `run_git_binary`, `WorkspaceError`. Contract: overlay save preserves bundle when candidate collection fails. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `095` `MechanicsTest.test_overlay_save_preserves_bundle_when_patch_does_not_verify` — lines 4041–4104; direct subjects: `command_overlay_save`, `subprocess`, `WorkspaceError`. Contract: overlay save preserves bundle when patch does not verify. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `096` `MechanicsTest.test_overlay_save_replacement_removes_obsolete_nested_entries` — lines 4105–4166; direct subjects: `command_overlay_save`, `OVERLAY_SUBMODULES_FILE`, `OVERLAY_SUBMODULES_DIR`, `OVERLAY_PATCH_FILE`, `command_overlay_apply`. Contract: overlay save replacement removes obsolete nested entries.
- `097` `MechanicsTest.test_overlay_save_nested_only_replacement_unlinks_root_patch` — lines 4167–4222; direct subjects: `command_overlay_save`, `OVERLAY_PATCH_FILE`, `OVERLAY_SUBMODULES_FILE`, `command_overlay_apply`. Contract: overlay save nested only replacement unlinks root patch.
- `098` `MechanicsTest.test_overlay_save_inventory_refusal_preserves_existing_bundle` — lines 4223–4265; direct subjects: `command_overlay_save`, `WorkspaceError`. Contract: overlay save inventory refusal preserves existing bundle. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `099` `MechanicsTest.test_source_commit_stages_owned_paths_and_marks_green` — lines 4266–4315; direct subjects: `source_task_ref`, `command_source_commit`, `resolved_ref`, `command_source_verify_commit`. Contract: source commit stages owned paths and marks green.
- `100` `MechanicsTest.test_source_commit_force_mirrors_head_to_the_checkout_branch` — lines 4316–4363; direct subjects: `command_source_commit`. Contract: source commit force mirrors head to the checkout branch.
- `101` `MechanicsTest.test_source_commit_keeps_the_commit_when_the_mirror_is_unavailable` — lines 4364–4399; direct subjects: `command_source_commit`. Contract: source commit keeps the commit when the mirror is unavailable.
- `102` `MechanicsTest.test_source_commit_stages_a_removed_tracked_file` — lines 4400–4429; direct subjects: `command_source_commit`. Contract: source commit stages a removed tracked file.
- `103` `MechanicsTest.test_source_commit_stages_a_non_ascii_tracked_file` — lines 4430–4480; direct subjects: `path_is_covered`, `command_source_commit`. Contract: source commit stages a non ascii tracked file.
- `104` `MechanicsTest.test_source_commit_rejects_paths_outside_owned_set` — lines 4481–4514; direct subjects: `WorkspaceError`, `command_source_commit`. Contract: source commit rejects paths outside owned set. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `105` `MechanicsTest.test_already_satisfied_source_state_requires_baseline` — lines 4515–4530; direct subjects: `source_task_ref`, `validate_source_state`, `WorkspaceError`. Contract: already satisfied source state requires baseline. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `106` `MechanicsTest.test_adaptive_outcome_result_contract` — lines 4531–4572; direct subjects: `validate_outcome_result`, `WorkspaceError`. Contract: adaptive outcome result contract. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `107` `MechanicsTest.test_test_block_requires_real_recovery_exhaustion` — lines 4573–4627; direct subjects: `validate_blocked_result`, `WorkspaceError`. Contract: test block requires real recovery exhaustion. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `108` `MechanicsTest.test_task_type_defaults_to_implementation` — lines 4628–4651; direct subjects: `load_state`, `update_state`, `WorkspaceError`. Contract: task type defaults to implementation. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `109` `MechanicsTest.test_fence_create_and_check_detects_changes` — lines 4652–4685; direct subjects: `command_fence_create`, `command_fence_check`. Contract: fence create and check detects changes. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `110` `MechanicsTest.test_source_preflight_reports_dirty_and_account_state` — lines 4686–4712; direct subjects: `command_source_preflight`. Contract: source preflight reports dirty and account state.
- `111` `MechanicsTest.test_finish_publishes_split_required_with_carried_work` — lines 4713–4769; direct subjects: `source_task_ref`, `command_finish`, `load_state`. Contract: finish publishes split required with carried work.
- `112` `MechanicsTest.test_carried_work_snapshot_seals_owned_submodule_changes` — lines 4770–4800; direct subjects: `source_worktree_snapshot`. Contract: carried work snapshot seals owned submodule changes.
- `113` `MechanicsTest.test_split_publish_routes_and_starts_implementation_carrier` — lines 4801–4950; direct subjects: `source_task_ref`, `source_worktree_snapshot`, `command_split_publish`, `resolve_task`, `load_states`, `command_start`, `load_batch`, `resolved_ref`. Contract: split publish routes and starts implementation carrier.
- `114` `MechanicsTest.test_publish_recovers_approval_refs_after_fetch_failure` — lines 4951–5015; direct subjects: `WorkspaceError`, `unpublished_counts`, `load_state`, `state_path`. Contract: publish recovers approval refs after fetch failure. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `115` `MechanicsTest.test_publish_preserves_approval_refs_for_unsafe_source` — lines 5016–5057; direct subjects: `WorkspaceError`, `unpublished_counts`. Contract: publish preserves approval refs for unsafe source. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `116` `MechanicsTest.test_publish_cleans_refs_when_approval_landed_before_interruption` — lines 5058–5113; direct subjects: `WorkspaceError`, `unpublished_counts`. Contract: publish cleans refs when approval landed before interruption. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `117` `MechanicsTest.test_publish_never_cleans_refs_without_owned_approval` — lines 5114–5198; direct subjects: fixture/wrapper behavior without a direct `workspace.*` symbol reference. Contract: publish never cleans refs without owned approval.

#### BatchMarkTest (8 test methods)

- `118` `BatchMarkTest.test_start_writes_the_batch_into_its_start_commit` — lines 5300–5345; direct subjects: `command_start`, `changed_paths`. Contract: start writes the batch into its start commit.
- `119` `BatchMarkTest.test_start_rejects_unknown_batch_ids_before_changing_anything` — lines 5346–5364; direct subjects: `WorkspaceError`, `command_start`, `changed_paths`, `load_state`, `state_path`. Contract: start rejects unknown batch ids before changing anything. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `120` `BatchMarkTest.test_start_without_a_batch_marks_only_the_started_task` — lines 5365–5376; direct subjects: `command_start`, `load_batch`. Contract: start without a batch marks only the started task.
- `121` `BatchMarkTest.test_a_batch_marks_tasks_without_claiming_them` — lines 5377–5431; direct subjects: `command_start`, `command_queue`. Contract: a batch marks tasks without claiming them.
- `122` `BatchMarkTest.test_a_lost_start_leaves_no_batch_published` — lines 5432–5461; direct subjects: `command_start`, `WorkspaceError`, `changed_paths`. Contract: a lost start leaves no batch published. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `123` `BatchMarkTest.test_queue_lists_an_absent_batch_as_empty` — lines 5462–5466; direct subjects: `command_queue`. Contract: queue lists an absent batch as empty.
- `124` `BatchMarkTest.test_batch_option_collects_every_id_in_order` — lines 5467–5484; direct subjects: `py`, `parse_args`. Contract: batch option collects every id in order.
- `125` `BatchMarkTest.test_reading_a_batch_ignores_what_it_cannot_use` — lines 5485–5515; direct subjects: `load_batch`. Contract: reading a batch ignores what it cannot use.

#### HealthWorktreeTest (2 test methods)

- `126` `HealthWorktreeTest.test_second_run_of_the_day_is_dropped` — lines 5567–5591; direct subjects: `command_health_ensure`, `command_health_publish`, `RebaseConflict`. Contract: second run of the day is dropped. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.
- `127` `HealthWorktreeTest.test_only_code_health_paths_are_published` — lines 5592–5602; direct subjects: `command_health_ensure`, `WorkspaceError`, `command_health_publish`. Contract: only code health paths are published. Contains an explicit expected-failure assertion, so the rejected/ambiguous path is part of the oracle.


### Strict index 55 traceability anchors

- `TDRP-R9-PROCESS-INBOX-WORKSPACE-TESTS-001`: the exact `workspace_test.py` source entry is a repository workflow regression-contract suite. Its applicable responsibility is retained by the existing source-neutral development/test/evidence owner; no shipping runtime is introduced.
- `ORA-TDRP-WORKSPACE-TEST-METHOD-IDENTITY-001`: the locked blob contains exactly 127 unique `test_*` methods, and the dossier records each method once with class and source line range. A missing, duplicate, or silently merged method record is non-passing.
- `ORA-TDRP-WORKSPACE-TEST-FAIL-CLOSED-BOUNDARIES-001`: explicit negative assertions preserve rejection semantics for invalid ownership, path, lineage, receipt, dependency, build-lock/process, overlay, batching, publication and result/evidence states rather than converting ambiguity into success.
- `ORA-TDRP-WORKSPACE-TEST-TRANSACTION-OWNERSHIP-001`: inbox/task/source/health publication tests preserve exact worktree/ref/path/digest/receipt ownership and conflict semantics; source commit mirroring remains a separate bounded outcome and mirror failure cannot erase an already-valid source commit.
- `INV-TDRP-WORKSPACE-TEST-NO-SHIPPING-OWNER-COLLISION-001`: repository regression tests remain quality/development evidence and cannot become a second shipping Task/Automation/Conversation/Agent/Coordinator/Host/Runner/Resource owner.
- `INV-TDRP-WORKSPACE-TEST-COVERAGE-ACCOUNTING-001`: closing index 55 requires 127/127 independently recorded test methods plus support-fixture accounting; dependency-read evidence from index 54 alone is insufficient.

Closing only strict index 55 changes counters to unknown=16,248, unread=16,223, omitted=0 and responsibilities=73. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. The next strict root inventory entry is index 56 `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430`. Fresh exact-head GitHub Actions evidence is required before this new mapping can be treated as validated.
