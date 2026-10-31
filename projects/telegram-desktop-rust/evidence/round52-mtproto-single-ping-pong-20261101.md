# Round 52 — MTProto single-ping pong correlation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 21  
Stage: A — implementation only, zero CI

## Fixed authorities

- Upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- Desktop starting checkpoint: `c875c906215ba70b5ab923b22276c5470214cdef`

## Source finding

The fixed-U `mtpc_pong` branch parses `pong#347773c5 msg_id:long ping_id:long`, requires the pong message id to refer to a recently sent request, and clears the live ping only when the returned `ping_id` matches. Fabushi's existing `EstablishedSession::send_ping` has exactly one outstanding ping, so it can represent exact current-ping correlation using the current request message id plus ping id. Arbitrary historical `wasSent/haveSent` lookup is broader multi-request history and remains open/fail-closed.

## Implemented exact subset

Canonical platform-core:
- spec `47246159887072673aff6609fb57f9cbc81ff2a2`;
- implementation `bf6f063b229be43597bddc4231baab24d06a09bd`;
- protocol api `32a7610bfc74835734c1e5c4385e4eb4cb2ee1f3`;
- protocol lib `ffbabc8f66112c2321be75a34928b086d6adf57f`;
- network `e0bf1b5f286ad0f8bc45a36e86074e19b7dde560`.

The protocol owner now strictly parses the fixed-U pong object and rejects wrong constructor, truncation and trailing bytes. The network owner routes the existing ping branch through that parser and accepts only a pong whose `msg_id` equals the current ping request message id and whose `ping_id` equals the current ping id. Mismatch fails through the existing `PingMismatch` path.

Desktop provider synchronization:
- implementation commit `3e363ccdaf287878f457797442aa97e8b597384b`;
- provider protocol api `32a7610bfc74835734c1e5c4385e4eb4cb2ee1f3`;
- provider protocol lib `ffbabc8f66112c2321be75a34928b086d6adf57f`;
- provider network `e0bf1b5f286ad0f8bc45a36e86074e19b7dde560`.

## Authored contracts, not run

- fixed-U pong field parsing;
- pong trailing-byte rejection;
- exact current request-message-id + ping-id correlation.

No build, typecheck, lint, validator, generator, test, packaging, deployment, installation, Actions/status/check query, or runtime acceptance was run or queried.

## Deferred/fail-closed responsibilities

- arbitrary `haveSent` / historical `wasSent` lookup;
- `_resendingIds` / `toSend` cancellation;
- `_sentContainers` fan-out;
- `_stateAndResendRequests` lifecycle;
- multi-ping or multi-request historical correlation;
- broader sent/resend durability and concurrency;
- higher-level new-session update/difference propagation;
- remaining fixed-U source/legal/release closure and Stage-B evidence.

## Stage-A status

`implementation_status=implemented` for `TDRP-R9-MTPROTO-SINGLE-PING-PONG-001`.  
`validation_status=not-run`.  
`accepted=false`.  
Global source-entry counters remain `unknown=16172`, `unread=16145`, `omitted=0`.
