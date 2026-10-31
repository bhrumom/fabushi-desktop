# Round 56 — MTProto new-session wire validation and difference blocker

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 25  
Stage: A — implementation only, zero CI

## Fixed authorities

- Upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- `Telegram/SourceFiles/api/api_updates.cpp@71c929cfe3c779c94719efef32a47a4d029e2c67`
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a`
- Desktop starting checkpoint: `f50194055cf2db53b7f8be3c4d9e4793465511b4`
- Platform-core starting checkpoint: `a72754045c81a193e8bdd36049577e01dc0a3097`

## Existing-owner-first high-level recovery review

Fixed U forwards `new_session_created` to `Api::Updates::mtpNewSessionCreated`, resets sequence continuity, and calls `updates.getDifference`. Empty responses advance date/seq, slice responses apply data then advance to intermediate state and continue, and final responses apply data before final state.

Current canonical owners do not yet provide a complete decoder/application bridge for the arbitrary `Message`, `EncryptedMessage`, `Update`, `Chat`, and `User` objects carried by full/slice differences. `telegram-protocol/src/updates.rs` only owns get-state/get-difference builders, update-state parsing, and terminal empty/too-long parsing. `telegram-core::TelegramEngine` has source-neutral chat/message commands, but no TL difference adapter currently connects these nested wire objects to those commands.

Therefore the complete high-level recovery responsibility is fail-closed. The missing condition is a complete full/slice TL decoder plus canonical application bridge with state advancement only after successful application. A terminal-only implementation was rejected because it could discard missed updates.

Durable blocker/spec: platform-core `ba9143b660d58728de8b93a49e7c2f97fcd39ed5:docs/specs/telegram-new-session-update-difference-recovery.md`.

## Implemented independent responsibility

The current network path previously recognized `new_session_created#9ec20908` by raw offsets and did not validate `unique_id` or exact full-buffer consumption. Platform-core spec `ba9143b660d58728de8b93a49e7c2f97fcd39ed5:docs/specs/telegram-mtproto-new-session-wire.md` defines the bounded exact-wire slice.

Canonical implementation:
- commit `dc66cc7aae7644530fa03160438df5174809beb4`;
- protocol updates `23a09bb6e4e78ef526823cf01d0a207565fb2a3e`;
- protocol lib `7b9c641eabccb7dba8c5a425904b6a676b12d866`;
- network `234c6c0070a1a92218fe5c974658d16d37fee46a`.

The shared protocol owner now parses all three longs and requires exact full-buffer consumption. Both the current-RPC receive path and ping/service path invoke that parser before salt mutation. The current-RPC path retains the existing one-retry `request_message_id < first_message_id` contract; no historical request queue is added.

Desktop provider synchronization:
- implementation commit `23b24ab2db29697071d21eb747f475b2196d891f`;
- provider protocol updates `23a09bb6e4e78ef526823cf01d0a207565fb2a3e`;
- provider protocol lib `7b9c641eabccb7dba8c5a425904b6a676b12d866`;
- provider network `234c6c0070a1a92218fe5c974658d16d37fee46a`.

The existing desktop provider `lib.rs` declares `pub mod updates` but its tree lacked `src/updates.rs`; this synchronization fills that declared canonical module slot. No second protocol/runtime/provider was created.

## Authored contracts, not run

- exact first_msg_id / unique_id / server_salt parsing;
- wrong-constructor rejection;
- truncation rejection;
- trailing-byte rejection;
- existing bounded new-session current-RPC replay predicate remains authored.

No build, typecheck, lint, validator, generator, test, packaging, deployment, installation, Actions/status/check query, or runtime acceptance was run or queried.

## Deferred / fail-closed responsibilities

- complete new-session -> updates.getDifference -> full/slice decode/apply/continue propagation;
- `gzip_packed` bounded decompression;
- arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history;
- remaining fixed-U source/legal/release closure;
- Stage-B executable evidence and independent release ACCEPT.

## Stage-A status

`implementation_status=implemented` for `TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001`.  
`validation_status=not-run`.  
`accepted=false`.  
Global source-entry counters remain `unknown=16172`, `unread=16145`, `omitted=0`.
