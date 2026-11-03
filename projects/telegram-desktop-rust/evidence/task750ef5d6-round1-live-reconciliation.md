# Task 750ef5d6 round 1 — live GitHub reconciliation

Date: 2026-11-03
Repository: bhrumom/fabushi-desktop
Default branch observed: main
Live main commit observed: 5528062177e3865eef60a073a066cca144f9e3e8
Task scope: Telegram Desktop to Fabushi full migration; this note is a recovery pointer, not implementation or acceptance evidence.

## Live authorities

- AGENTS.md blob e4682748087759b2d4bb2b3a47916b6148e8a6f2.
- docs/specs/fabushi-deferred-ci-migration.md blob 359659cc3125d29ecb626cc74c5de40a9a1081a9; FMEP-001 Revision 2, including §12 owner-first convergence and §13 FMEP-SOURCE-01.
- projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md blob 9c19b65b62ba42f7f22f530040f0bf4a555a8f15; TDRP-001 Revision 10.
- projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md blob 3f1408a1460a589eeffceba3c3404c0c151d1bfa; FBCP-001 Revision 8.
- docs/specs/telegram-desktop-rust-equivalence-migration.md blob 2b3873eb34a5e31ed2460c1a9f6b72df51583694.
- docs/specs/fabushi-bot-communication-platform.md blob 93c543fafda43a0f84ecc13808dba9dd2ef71b42.
- projects/telegram-desktop-rust/upstream.lock.json blob 4538c49b47ad655d2bf0a254a545de12a410d355.
- projects/telegram-desktop-rust/parity-ledger.json blob 526f0fce19df7348e0f6f71478c8590070a3b6aa.
- projects/telegram-desktop-rust/STATUS.md blob fe87d89540009e8c316c57570e34bb3b2ad20cfb.
- projects/telegram-desktop-rust/module-map.md blob 69f739b17a241386e3f22a5e8ed2d842e72baa43.
- projects/fabushi-communication-platform/current-owner-inventory.md blob 749a9432b2dcb2d8db1037fe051ef68d2144f8d1.
- projects/fabushi-communication-platform/owner-resolution-matrix.md blob 089bcf8b9e21b5c697ed13042be05f5649e134cc.

## Current source and ledger facts

- The lock pins telegramdesktop/tdesktop commit 351204f3695b4a39bc3f4d2f230c061b9a01fe2a, tree 5151db79560ed40bf3802482fcfb74b950be6c40, and says accepted=false.
- The ledger records 16,202 source entries, unknown 16,172, unread 16,145, omitted 0, 52 responsibilities, and 0 unmapped responsibilities. These are not acceptance evidence; source-review contract reconciliation remains required.
- The TDRP STATUS is dated 2026-10-19 and contains historical/stale baseline claims; do not reuse those counts as live proof.
- The current-owner inventory explicitly labels itself a 2026-10-03 evidence snapshot. Its warning says a native Human messaging production path already exists and stale not-implemented rows must not create parallel owners.
- FMEP-001 §12 requires tracing Session/Transcript, native messaging, FeatureHost MessagingService, and legacy Telegram Provider/FFI from shipping entrypoints before choosing a messaging owner. §13 requires exact source identities and risk-tiered review; it does not authorize changing historical unread counts.

## Existing migration line and PR reconciliation

- Live checkpoint branch checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1 was read at a5bc9f52eab64547a3a35d74448e1be167e28bf9.
- Open drafts #189, #190 and #191 are distinct sibling work. Do not force-push, overwrite, or infer their changes are integrated.
- PR #189 head: 64bce6748353eaead0d273d55ee9e2ab0d08e30a. Its patch includes Forward UI, canonical transcript projection/pagination, Session forward batch journal/native messaging, coordinator admission, settings proxy behavior, and the source review workflow.
- PR #190 head: 831b1f5a2cb5c968d8b4fea6280018cf388614e0. Its description reports integrated candidate H 2a78098a74be74b0a73a335cd751a6d1f2397e4a and changes to Human message recovery, original-account dispatch fences, queue origin and persisted draft attachment retention. Its source closure, deployed service, signed packages, security, temporal, performance, rollback and independent ACCEPT gates remain open.
- PR #191 head: a1ec5b644f9f70b40aefc6f4a18370626b1f8efd. Its patch adds Human call identity/credential scope and media capture lifetime work under existing Session and call owners; the described exact 8f4 Actions runs were pending, and physical device/OS, deployed WebRTC/TURN, account-epoch dispatch, restart rejoin, package and final acceptance remain open.
- A direct workflow-run query for PR #190 branch head returned no runs; combined status reads for H 2a78098a74be74b0a73a335cd751a6d1f2397e4a and 831b1f5a2cb5c968d8b4fea6280018cf388614e0 returned no status entries. Do not infer passing Actions.

## Round-1 state

No production code or tests were changed in this round. No source-review status, responsibility status, coverage count, or acceptance value was promoted. This file records verified live references and a continuation pointer only.

## Immediate next action

On the exact migration branch, complete the required read of FBCP/TDRP applicable requirements and the source_reviews schema/validator. Trace the shipping message entrypoint through Session/Transcript, native_messaging, FeatureHost MessagingService and all legacy Telegram Provider/FFI callers. Compare #189/#190/#191 ancestry and full changed-file content against the branch before integrating any non-overlapping production batch. Implement owner code and tests together, freeze exact H, run only required GitHub Actions, and preserve exact evidence. Do not claim source closure, full implementation, Actions approval, or independent ACCEPT from this note.
## Actions and validator follow-up (2026-11-03)

- Confirmed the Revision-9 schema contains the optional historical-candidate `source_reviews` array with exact repository/commit/path/object identity fields, category/risk/depth/status/rationale/evidence/responsibility IDs. The executable `scripts/tdrp-source-review.mjs` validates complete exact identity set coverage, rejects duplicate/stale identities, requires deep review for product/mixed-ui/high-risk, contract review for generated/dependency, and requires evidence plus valid responsibility mapping. The authority entrypoint invokes this validator in `--require-accepted` mode. Historical candidate mode remains optional by design; this does not itself prove that the fixed source inventory is closed.
- PR #189 exact head 64bce6748353eaead0d273d55ee9e2ab0d08e30a Actions: Native service contracts run 41533601458 failed in `native-message-reconciliation` at concurrent Human creation/non-destructive initialization recovery and `native-message-ui` at actual ProductionRenderer history/account/restart projection; shipping compile, Electron, Windows, Rust desktop runtime and source review passed. These two failing checks remain unqualified.
- PR #190 integrated candidate H 2a78098a74be74b0a73a335cd751a6d1f2397e4a Actions: run 41523432089 overall failure because shipping compile was cancelled and `native-auth-and-journal` was skipped; Electron, UI, reconciliation and Windows passed. Its exact source review run 41523432117 passed. Do not mark the batch accepted while required jobs are cancelled/skipped.
- PR #191 bounded call batch: Native call credential/scope run 41525143801 at H 523c8ca4883e6eaa10d1d19a0e237f9d3964b3c6 passed; source-review run 41525143819 passed. Human call media run 41529946311 at H 8f4b1943b4043e6a9da7051d30ff1cefc04b39b2 passed, and its source candidate run 41529946243 passed. Later exact PR head a1ec5b644f9f70b40aefc6f4a18370626b1f8efd source review run 41532506991 passed. These are bounded batch results, not full call acceptance or overall migration ACCEPT.
- The current migration checkpoint branch continued advancing independently after the round-1 documentation commit; at the latest observed read it was 39d0e7555676c8987f6686a55bc6ae0be31c8c8d. This confirms active concurrent work; do not reset or force-update it. Additional branch workflows at prior exact H a5bc9f52eab64547a3a35d74448e1be167e28bf9 include source-review contract and three ledger-object preparation runs that succeeded (run IDs 41567429857, 41567429770, 41567429699, 41567429889). Those are not product behavior acceptance.
- Round-1 remains documentation/evidence-only in this session: no product implementation or test authored here, no source-review status promoted, no Actions pass attributed to the evidence-note commit. All implementation, source closure, legacy-path retirement, deployed-service/platform/package and final independent acceptance gates remain open.
## Follow-up production repair (2026-11-03)

- Exact failing Actions evidence from PR #189 run 41533601458: `human_creation_tests.rs::corrupt_or_notadb_creation_preserves_original_database_sidecars_and_siblings` failed at line 777. The assertion showed the retained main file and unrelated resource still present, but the original `-wal`, `-shm`, and `-journal` sidecars had disappeared after attempted creation.
- Root cause in `agent_db_recovery.rs::probe_human_conversation_store`: any non-empty WAL/journal sidecar set `has_snapshot_sidecar`, which bypassed `PRAGMA quick_check`; `PRAGMA schema_version` alone did not reject a full-length non-SQLite main file. The later writable opener could discard those orphaned sidecars.
- Applied production fix in `source/host/src/extensions/session/agent_db_recovery.rs`: before the writable opener, an existing main file of at least 100 bytes must have the exact SQLite format-3 16-byte header. A WAL/rollback journal can restore pages of a SQLite database, but cannot make a non-SQLite main file a SQLite database. Existing `corrupt_or_notadb_creation_preserves_original_database_sidecars_and_siblings` is the regression oracle; no data or sidecar deletion was introduced.
- Production commit: `b2888e9c16c79ccbfc0b72a9a659f81e276f24aa`, parent `cc16c51eb94c9926cb3560b6973ef41ee241f399`. GitHub API readback verified the commit, changed blob `13af08212519f6ba2210ef6817900c699992c151`, and the migration branch ref at that commit when first published.
- Exact-H Native service contracts run `41568455392` was triggered for H `b2888e9c16c79ccbfc0b72a9a659f81e276f24aa`. At the latest job read, `native-message-reconciliation`, `native-message-ui`, `native-message-electron`, `windows-update-publisher`, `gateway-peer-durability (windows-2025)`, `desktop-social-owner-behaviors`, and `retained-direct-ffi-compatibility` were still running; shipping compile was queued; Native auth/journal was skipped. Only `gateway-peer-durability (ubuntu-24.04)` had completed successfully. No full batch pass is claimed.
- The same PR #189 UI job's 9 failures all occurred in common `openAlice` setup at `desktop/e2e/native-message-projection.spec.ts:34`, expecting one pending older-page request but observing two after the test deliberately dispatches a scroll event while the automatic page request is pending. The UI uses the canonical pagination controller and the fixture counts unresolved older requests. This is retained as an unresolved behavior/fixture-oracle investigation; no expectation was weakened and no UI behavior was marked fixed.
- Remaining batch work: read exact-H Native Actions logs, classify the Native sidecar regression as fixed only if the existing sidecar preservation test passes, investigate the duplicate pending older requests from actual request identity/account scope and component lifetime, then repair the owning production path and keep the E2E oracle strict. Source closure, other product capabilities, legacy Telegram path migration, services/platform packages, final regression and independent ACCEPT remain open.