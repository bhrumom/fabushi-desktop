# Round 48 — MTProto new-session pending-RPC replay evidence

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 18  
Stage: A — authored and committed, not executed

## Source authority

- U `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- fixed-U `new_session_created#9ec20908` supplies `first_msg_id`, `unique_id`, and `server_salt`; pending requests older than `first_msg_id` are resent.

## Existing-owner-first implementation

Platform-core spec: `8275952bed7bea3a7c275f1b122c2c9c710ba262`  
Platform-core implementation: `6ec2d64da2d30d78c919bd971e8e0d1953aed587`  
Canonical network blob: `7df3dd74ed1cc4fea5273a36152d1ff8bd00ec65`

Desktop synchronization: `01b78f13e4a500bdb401c080a40f5295d8110779`  
Desktop network blob: `source/mahayana/mahayana-rs/providers/telegram-network/src/lib.rs@7df3dd74ed1cc4fea5273a36152d1ff8bd00ec65`

The unique synchronous `EstablishedSession::invoke_raw` now updates the salt and replays its one outstanding logical RPC exactly once only when `request_message_id < first_msg_id`. A second replay signal returns `NewSessionReplayLimit`. No source-specific multi-request queue or second runtime/session store was introduced.

## Scope boundary

Higher-level delivery of the new-session update into Telegram/Fabushi update-difference recovery remains open and is not claimed implemented here.

## Authored-only contract

`recovery_contract_tests::new_session_replays_only_requests_older_than_first_message` exists in the shared and synchronized network source. It was not executed.

Status: implemented / validation_status=not-run / accepted=false. No CI/status query, build, lint, typecheck, schema generation/validation, test, packaging, deployment or installation was performed.
