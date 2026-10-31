# Round 57 — MTProto detailed-info exact wire evidence

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 26
Stage A: implementation/contracts authored; executable validation not run.

## Fixed-U source authority

- `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- fixed-U type: `msg_detailed_info#276d3ec6 msg_id:long answer_msg_id:long bytes:int status:int = MsgDetailedInfo`

Manual source reread confirmed Telegram Desktop fully decodes the generated TL object before using `msg_id`, `answer_msg_id`, `bytes`, or `status`.

## Pre-change gap

At desktop/platform checkpoint start, `EstablishedSession::scan_rpc_body` matched constructor `0x276d3ec6` but read only `msg_id` at offset 4 and `answer_msg_id` at offset 12. It did not validate `bytes`, `status`, or the exact end of the object. The adjacent `msg_new_detailed_info#809db6df` path already used a complete typed parser.

## Canonical implementation

- spec commit: `8fffe0ca4e7d48d097e9f15f0cf332610e28f4fc`
- implementation commit: `4c5bea7b847a3e04ee3d0c9b05fab228101acb64`
- source-level spec compliance: `46cf745e2c228f0638d2d0afa2856579ddba3d70`
- protocol api blob: `5da871fc557a49a7e57f1dd0677f307a5ed1bac8`
- protocol lib blob: `3f7c363f9fd2eb86e60c8294144d7f1dced9ccca`
- network blob: `2a50068d148b25e710961973315d78fc1b1ee5a8`

The existing protocol owner now exposes `MSG_DETAILED_INFO_CONSTRUCTOR`, `MsgDetailedInfo`, and `parse_msg_detailed_info`. The parser consumes all four fixed-U fields and rejects wrong constructor, truncation, and trailing bytes. The existing network branch calls this parser before current-request matching or `reconcile_detailed_answer`.

## Desktop production composition

Desktop implementation commit `45c24752b9d4764768a9eec26ead9fb6b36dc426` synchronizes the same three blobs into:

- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs`
- `source/mahayana/mahayana-rs/providers/telegram-network/src/lib.rs`

The production path remains the existing account-bound `TelegramClientLease -> RuntimeClient -> EstablishedSession` composition. No second runtime/provider/history owner was created.

## Authored-only contracts

The canonical protocol source contains authored tests for exact four-field parsing plus wrong-constructor, truncation, and trailing-byte rejection. Existing network contracts still cover current-request-only matching and the bounded current-session reconciliation predicate. None were run in Stage A.

## Ledger disposition

The existing `TDRP-R9-MTPROTO-DETAILED-ANSWER-RESEND-001` row is hardened in place rather than duplicated. Counters remain `responsibilities_total=67`, `unknown=16172`, `unread=16145`, `omitted=0`, `unmapped_responsibilities=0`.

Open/fail-closed: `gzip_packed`; complete `updates.differenceSlice/full` decoding/application and new-session propagation; arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` multi-request history; broader fixed-U source/legal/release closure; all Stage-B executable evidence.

Status: implemented / validation_status=not-run / accepted=false.
