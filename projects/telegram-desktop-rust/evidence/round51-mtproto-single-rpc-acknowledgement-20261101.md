# Round 51 — MTProto single-RPC acknowledgement

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 20  
Stage: A — implementation only, zero CI

## Fixed authorities

- Upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- Desktop starting checkpoint: `82a4865217d532cbfd961a5935797d52ae1604b9`

## Source finding

The fixed-U `mtpc_msgs_ack` branch parses `msgs_ack#62d6b459 msg_ids:Vector<long>`, then calls `requestsAcked(ids)`. In `requestsAcked(ids, byResponse=false)`, a sent request with a response callback is deliberately retained rather than erased, so a normal transport ACK does not complete that RPC. Full historical ACK bookkeeping also traverses `haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, `_stateAndResendRequests`, and per-request timing; those multi-request semantics are not representable by Fabushi's current one-outstanding-RPC owner and remain open/fail-closed.

## Implemented exact subset

Canonical platform-core:
- spec `9f72142c7b4d8ab0e605e95301f4194d964ffa73`;
- implementation `3470cd572891222c8846a9e91f98ca621624ec64`;
- protocol api `d38546a0f6954bb6a03aaec80968dbf195945edc`;
- protocol lib `da012dab3b1111f18c318cf0001f2f3d1f32bf6f`;
- network `70645dd06426374ae4e20bd9c9545388948f65bb`.

The protocol owner now parses `msgs_ack` with the existing bounded TL reader and rejects trailing/malformed payloads. The network owner explicitly handles ACKs in both current RPC and ping/service paths, recognizes current-request membership, and always continues waiting rather than manufacturing `RpcResult`, replay, or completion from a normal ACK.

Desktop provider synchronization:
- implementation commit `6804906c0646d75a16b150b4f745476f44e19953`;
- provider protocol api `d38546a0f6954bb6a03aaec80968dbf195945edc`;
- provider protocol lib `da012dab3b1111f18c318cf0001f2f3d1f32bf6f`;
- provider network `70645dd06426374ae4e20bd9c9545388948f65bb`.

## Authored contracts, not run

- `msgs_ack` vector parsing and trailing-byte rejection;
- current-request ACK correlation predicate;
- intended Stage-B RPC ACK-then-result and ping ACK-then-pong integration contracts recorded in the ledger.

No build, typecheck, lint, validator, generator, test, packaging, deployment, installation, Actions/status/check query, or runtime acceptance was run or queried.

## Deferred/fail-closed responsibilities

- arbitrary `haveSent` ACK settlement;
- `_resendingIds` / `toSend` cancellation;
- `_sentContainers` ACK fan-out;
- `_stateAndResendRequests` lifecycle;
- per-request fast-clock correction based on historical send timestamps;
- broader multi-request sent/resend durability and concurrency;
- higher-level new-session update/difference propagation;
- remaining fixed-U source/legal/release closure and Stage-B evidence.

## Stage-A status

`implementation_status=implemented` for `TDRP-R9-MTPROTO-SINGLE-RPC-ACKNOWLEDGEMENT-001`.  
`validation_status=not-run`.  
`accepted=false`.  
Global source-entry counters remain `unknown=16172`, `unread=16145`, `omitted=0`.
