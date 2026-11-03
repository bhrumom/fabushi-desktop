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