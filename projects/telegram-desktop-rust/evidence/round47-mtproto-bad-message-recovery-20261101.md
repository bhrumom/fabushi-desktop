# Round 47 — MTProto bad-message recovery evidence

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 18  
Stage: A — implementation only, executable validation intentionally not run

## Fixed upstream authority

- U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
  - `mtpc_bad_msg_notification`: recoverable codes 16, 17, 32, 33, 64.
  - 16/17 and bad-container 64 follow the source resend path with trusted server-time correction.
  - 32/33 require session reset because message-id/sequence state is inconsistent.
  - unrelated/fatal notifications remain fail-closed.
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
  - `bad_msg_notification#a7eff811`
  - `bad_server_salt#edab447b`
  - `new_session_created#9ec20908`
  - `msgs_ack#62d6b459`

## Canonical shared-owner implementation

Repository: `bhrumom/fabushi-platform-core`  
Checkpoint: `checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`

- durable recovery spec commit: `47ed28d35dbb6c7dd19bdcf1cd66fbf88dc68f67`
- implementation commit: `ac9e20217812aa72b92049d4dfb52f76d6380da6`
- network blob: `native/telegram-network/src/lib.rs@ba9935b54e9c178c7d9377cd75572f2c21b62144`
- message-id/session blob: `native/telegram-protocol/src/session.rs@ab8fd351a05543854c3f62de5c8332a13b7ff472`

The existing `EstablishedSession` remains the only transport/session state owner. 16/17/64 correct the live clock from an authenticated server message id, reset only the client message-id monotonic floor, preserve the server replay set, and permit one fresh-message retry. 32/33 rotate the existing session id and reset the content-related sequence counter without replacing auth key/transport. Nested container messages use the same incoming replay/parity/time validation. Recovery is bounded to one retry.

## Desktop shipping composition

Repository: `bhrumom/fabushi-desktop`  
Checkpoint: `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`

Implementation commit: `6c714e664d6cd4d66228b313664a6e776a4fd558`

- `source/mahayana/mahayana-rs/providers/telegram-network/src/lib.rs@ba9935b54e9c178c7d9377cd75572f2c21b62144`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/session.rs@ab8fd351a05543854c3f62de5c8332a13b7ff472`
- existing protocol export authority remains `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@34bcd749f758cfd3f562b6e701ee10eb51d5d0c2`.

This is the same established Cargo path-dependency chain from the round-46 composition evidence. No second Telegram account/runtime/provider/transport/session/persistence owner was added.

## Authored-only contracts

- `telegram-network/src/lib.rs::recovery_contract_tests`
- `telegram-protocol/src/session.rs::message_id_recovery_contract_tests`

These contracts cover fixed-U classification, code 64 time-recovery parity, and resetting the client message-id floor while retaining server replay history. They were authored but not executed.

## Stage-A status

Responsibility: `TDRP-R9-MTPROTO-BAD-MESSAGE-RECOVERY-001`  
Implementation: `implemented`  
Validation: `not-run`  
Accepted: `false`

No GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, packaging, deployment, installation or runtime acceptance was performed. Global unknown/unread counters are not decremented because this evidence closes a responsibility slice, not the entire upstream source file.
