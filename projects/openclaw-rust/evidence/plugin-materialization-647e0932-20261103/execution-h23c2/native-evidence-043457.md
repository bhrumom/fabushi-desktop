# H23 native evidence addendum — 2026-11-03 04:34:57 UTC

This addendum preserves the later evidence separately from the earlier bounded snapshot. It grants no unit/global/package acceptance and no result to bb5 auth-recovery or later unpublished production changes.

## Exact identity and latest bounded status

- Repository bhrumom/fabushi-desktop; canonical branch codex/openclaw-rust-fusion-chat-20261101-writer.
- H23: 23c2a567d874509f56353b8cae737cdf1437309d.
- Tree: b10485b13c15cd1cf012a5f51acaf6927aeb322a.
- U: 1440bb1771372334bea23d7642bb52deb7db1845; U tree 90c8aab39972da11cfad220361781d49e1bc6700.
- Run https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105 ; attempt 1.
- API refresh at 04:32:06 UTC retained this exact head/tree/attempt; the run summary was still queued / null.
- Final job/artifact snapshot at 04:34:57 UTC:
  - sqlite-native-path, job 125721731498, remained in_progress / null. Portable step 12 completed successfully at 04:30:11Z; palette consumer step 15 succeeded at 04:30:37Z; before-Automation evidence upload step 17 succeeded at 04:30:38Z; typed Automation step 18 succeeded at 04:31:41Z. App step 19, started 04:31:41Z, remained in_progress. Prior-root step 20, native dispatch step 21 and final evidence upload step 23 remained pending.
  - rust-host, job 125721731668, remained queued / null on macos-15 with steps empty and runner_id 0.
- No terminal whole-job log or final native-path artifact was available in this snapshot. There was no additional artifact between the two refreshes. No continuing polling loop was started.

Job URLs:
- https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/job/125721731498
- https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/job/125721731668

## Newly published artifact

- Artifact ID 13203922970.
- Name ocrm-linux-before-automation-23c2a567d874509f56353b8cae737cdf1437309d-41576693105-1.
- URL https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/artifacts/13203922970
- Created 2026-11-03T04:30:38Z; 466873 bytes; expired false.
- API digest and downloaded ZIP SHA256: 96aaebf6d1bd920b997df907b6cf07f88babfcbc961c0ed208f5db4fb241e1a1.
- Archive contains 67 files. Its original bytes and extracted original files are preserved.
- Portable, native SQLite and same-job palette receipts independently bind H23, the tested tree, fixed U, run 41576693105 and attempt 1.
- This is the before-Automation checkpoint artifact. Step 18's later success is only a job API step outcome here; this checkpoint does not contain its completed evidence or the later App/prior-root/dispatch outcomes.

## Portable actual commands and original log readback

channel-pending-recovery/receipt.json contains 52 planned commands and 52 execution records. Every recorded command and required-positive list matches its planned record. Every execution records cargo_exit 0, capture_exit 0, execution_error null and oracle_accepted true.

All 52 original per-command log SHA256 values match the receipt. Literal original-log readback found:
- 98 successful Rust summary records.
- 1244 passed test executions, 0 failed, 1 ignored.
- 1244 original successful test-name lines, matching the recorded names and counts.
- 610 command-scoped required positive names, each appearing exactly once in its corresponding original log.
- No observed-name, summary, selected-command or required-positive difference.
- 14 result-oracle controls are recorded as matching expected/actual outcomes in Actions. Their expected negative cases are controls, not failed product tests; they are separate from the 1244 Rust executions.

These are executions across the selected command set, not a count of globally unique tests or whole-library coverage. Some owners are selected in multiple command groups.

## Plugin Rust commands: completed H23 evidence

| Actual command group | Passed | Failed | Ignored | Required positives |
| --- | ---: | ---: | ---: | ---: |
| plugin-uninstall-service | 101 | 0 | 1 | 9 |
| plugin-uninstall-state | 73 | 0 | 0 | 26 |
| plugin-uninstall-binding | 9 | 0 | 0 | 3 |
| plugin-uninstall-relay-handoff | 1 | 0 | 0 | 1 |
| plugin-cache-publication | 3 | 0 | 0 | 0 |
| plugin-session-consumer | 3 | 0 | 0 | 0 |
| Total for six groups | 190 | 0 | 1 | 39 |

Every one of the 39 required plugin positives appears exactly once in its original command log. All six commands have cargo_exit 0, capture_exit 0, no execution_error, and oracle_accepted true.

Actual selected command scope:
- plugin-uninstall-service: cargo test --locked --manifest-path source/host/Cargo.toml --no-fail-fast with the ten integration targets mcp_service_contract, plugin_skills_contract, host_bundle_and_plugin_cache_contract, workflow_store_contract, workflow_commands_contract, legacy_live_references_contract, mcp_coordinator_relay_contract, mcp_extension_contract, mcp_production_contract and host_production_extensions_contract; --test-threads=1.
- plugin-uninstall-state: --lib extensions::mcp::plugin_skills::; --test-threads=1.
- plugin-uninstall-binding: --lib host_production_extensions::plugin_skills_binding_tests::; --test-threads=1.
- plugin-uninstall-relay-handoff: --lib extensions::mcp::coordinator_relay::lifecycle_tests::desktop_confirmation_keeps_the_actual_reader_across_same_root_handoff; --exact --test-threads=1.
- plugin-cache-publication: --lib extensions::mcp::plugin_skills_cache::tests::; --test-threads=1.
- plugin-session-consumer: --lib extensions::session::production::plugin_skills_admission_tests::; --test-threads=1.

### Actual owner / cancellation-close / process-recovery positives

The original logs contain explicit success for, among others:
- uninstall_reservation_failure_prevents_backend_mutation.
- uninstall_close_after_reservation_settles_only_not_dispatched.
- desktop_service_contracts::closed_desktop_service_never_replays_an_admitted_uninstall_or_dispatches_new_work.
- desktop_service_contracts::service_close_during_catalog_lookup_prevents_the_uninstall_mutation.
- desktop_service_contracts::late_desktop_uninstall_reply_cannot_settle_the_successor_service_request.
- host_mcp_lifecycle_close_settles_an_admitted_waiter_and_rejects_late_resolution.
- extensions::mcp::plugin_skills::sync_tests::active_failure_does_not_cancel_a_pending_success_or_borrow_its_result.
- extensions::mcp::plugin_skills::sync_tests::real_poll_shutdown_retires_a_queued_sync_before_joining_its_thread.
- confirmed_uninstall_consumers::uninstall_custody_survives_owned_process_exit_and_fresh_process_consumers.
- extensions::mcp::plugin_skills::sync_tests::uninstall_recovery_loads_exact_pending_and_confirmed_facts_without_invoking_the_loader.
- extensions::mcp::plugin_skills::sync_tests::uninstall_recovery_fault_requires_a_fresh_owner_after_repair.
- host_production_extensions::plugin_skills_binding_tests::fresh_host_binding_recovers_uninstall_custody_before_either_startup_order.
- host_production_extensions::plugin_skills_binding_tests::initialized_uninstall_state_loss_blocks_the_real_host_binding.
- host_production_extensions::plugin_skills_binding_tests::same_root_handoff_retains_failed_refusal_across_a_delayed_old_runtime_stop.

The sole ignored plugin case is confirmed_uninstall_consumers::uninstall_recovery_child, described in the original log as a private owned process-loss entrypoint that must be invoked only with --exact and a child-scoped request. The enclosing owned-process-exit/fresh-consumers test explicitly passed. The ignored helper is preserved as ignored in the top-level counts; it is not relabeled as a separately observed top-level success.

The recorded plugin scope is durable pre-dispatch tickets and exact settlements in the existing MCP/Skills owner; pending and confirmed refusal in actual Session/prompt consumers; strict image/marker recovery; cache-before-release barriers; and same-root Host handoff. Owned producer exits and fresh consumers use controlled backend I/O. Pending outcomes are not replayed or cleared by ordinary sync.

This supports the named H23 owner, close/retirement and owned-process-recovery controls. It does not accept account/backend incarnation changes, remote receipt/quiescence reconciliation, real ecosystem behavior, hardware power-loss or packaged-platform behavior. It does not prove later bb5 auth-recovery changes, despite some older H23 test names also mentioning auth/refusal.

## Native SQLite checkpoint and nested child count

The checkpoint's top-level receipt records:

cargo test --locked --no-fail-fast --manifest-path source/host/Cargo.toml --test host_storage_contract --test agent_db_recovery_contract --test conversation_blob_db_recovery_contract --test automation_store_contract -- --test-threads=1

cargo_exit and capture_exit are both 0. All 18 required positives (7 native-path, 7 checked-delivery, 4 fresh-custody) appear once.

Original outer-suite summaries:
- agent_db_recovery_contract: 18 passed, 0 failed, 1 ignored.
- automation_store_contract: 45 passed, 0 failed, 0 ignored.
- conversation_blob_db_recovery_contract: 4 passed, 0 failed, 0 ignored.
- host_storage_contract: 11 passed, 0 failed, 0 ignored.
- Total outer-suite passes: 78, failures: 0, ignored helpers: 1.

The agent DB parent test child_held_public_guard_blocks_recovery_until_exact_terminal_exit prints a nested invocation. automation_custody_public_guard_child is ignored in the outer listing but explicitly runs in the child, which reports 1 passed / 0 failed / 0 ignored. The parent then prints its own successful terminal on a separate line. Therefore summing all five raw summaries yields 79 passed including that nested child; it must not be presented as 79 different outer selected tests. The receipt's 78 observed inline success names include the child and omit the parent whose terminal is split by the child output. This formatting is visible in the original log; the four outer summaries and the nested summary are preserved separately.

Scope is native-byte SQLite paths and checked Automation FileStore delivery/fresh-history custody, with invalid UTF-8 identities, raw custody and proposed fresh identities. It does not establish all-filesystem behavior, metadata/open races, power loss, whole-platform acceptance or any later independent cron job-identity repair.

## Same-job palette checkpoint

The same SQLite job's palette receipt links the actual persisted Rust Session search export to the actual desktop provider/model:
- consumer: 12 tests, 12 pass, 0 fail/cancelled/skipped/todo; exact required names all in original TAP.
- production typecheck command npm --prefix desktop run typecheck: command_exit 0, capture_exit 0; original log is preserved.
- Fixture digest db2e2deebd5488524f7ef0fa10d75b0980e47abb873dc14fe15bb828a5800655 matches the portable export and the extracted bytes.
- Producer receipt and producer log digests match; palette_message_gateway_exports_persisted_rust_search_results appears exactly once in the original portable integration log.
- Input hashes and fixture digest before/after the consumer agree.

This is separate from the renderer job already owned by the primary integrator; that renderer artifact was not downloaded again. It does not assert rendered-browser, accessibility, real remote/indexed backend or package acceptance.

## Inputs and evidence byte identities

- Portable receipt: 372 input paths.
- Native fresh-custody receipt: 3 input paths, overlapping the portable set.
- Palette receipt: 18 input paths.
- Combined distinct paths: 375, with no conflicting hashes between receipts.
- All 375 exact H23 source bytes match the receipt SHA256 values.
- Exact H23 git ls-tree records bind all 375 paths to regular-file Git blob identities.
- The sole locally unavailable selected blob was the previously recovered exact-H capture_portable_contracts.py; its Git blob identity ee404c54ce24a740f6808a8c7bba9650a0f0e2d6 matches the H23 tree. Its raw source was copied as evidence, not executed.
- 57 archive-evidence SHA256 comparisons succeeded: 52 portable logs, native tests.log, 2 palette logs, palette fixture and portable producer receipt.
- Original raw receipts are retained; derived readback tables do not replace them.

A read-only Git source export emitted an invalid-object warning for projects/openclaw-rust/evidence/device-token-rotation-wire-review-647e0932-20261103/actions/0c7-source-audit.json (object 73ef1a488ae4ffe88ceb9a2145a0ed962ce5185e). Export status was not treated as proof. The final 375 selected-file byte comparisons and exact-H path/blob inventory establish the stated input readback, including the independently recovered capture source. This acquisition warning is not a product execution failure.

## Failure classification and remaining boundary

No selected completed portable/plugin/native/palette execution in this artifact reports a product test failure. Ignored private helpers and intentional negative oracle controls are identified above instead of being silently counted as passed.

The overall SQLite job remains in_progress and macOS remains queued at the last snapshot. Missing final evidence remains pending; no scheduling cause or implementation defect is inferred from that absence. Full job completion, original-turn/global restart, all plugin permissions/accounts, later bb5 auth-recovery, later unpublished cron production work and unit/global/packaged acceptance remain unassigned.

Only GitHub/API reads, exact-H source reads, raw evidence download/extraction, literal log readback and byte/identity comparisons were performed. No local/htch build, test, lint, product parser/generator/validator/harness or repository script execution occurred. No production/workflow/ledger edit or push occurred. Actual host model/effort were inherited unchanged.

## Evidence entry points

All files live under /workspace/scratch/425c55809bda/reviews/plugin-h23-native-evidence/ .
- h23-linux-before-automation-13203922970.zip — original downloaded artifact.
- linux-before-automation-artifact/channel-pending-recovery/receipt.json — original portable receipt.
- linux-before-automation-artifact/channel-pending-recovery/plugin-*.log — six original plugin command logs.
- linux-before-automation-artifact/receipt.json and tests.log — original native SQLite checkpoint.
- linux-before-automation-artifact/palette-message/41576693105-1/ — original same-job palette evidence.
- portable-literal-readback.json — derived per-command raw-name/count/command identity comparisons.
- plugin-command-readback.json — exact plugin command/receipt records.
- native-sqlite-literal-readback.json — native literal name readback, with nested-count qualification in this report.
- cross-receipt-readback.json — H/tree/U and palette producer linkage.
- source-input-identities.txt and source-input-sha256-comparison.txt — all selected exact-H Git identities and byte comparisons.
- before-automation-evidence-sha256-comparison.txt — all 57 evidence-byte comparisons.
- run-41576693105-attempt1-jobs-043457-api.json and run-41576693105-artifacts-043457-api.json — final bounded snapshots.
- FILE-INVENTORY.txt — complete absolute-path inventory, including all raw extracted files and exact-H source copies.
