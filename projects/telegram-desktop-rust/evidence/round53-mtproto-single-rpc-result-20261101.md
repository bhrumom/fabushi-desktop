# Round 53 — MTProto single-RPC result correlation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 22  
Stage: A — implementation only, zero CI

## Fixed authorities

- Upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- Desktop starting checkpoint: `770510d554552d5d4aa2f2b69f3d62612e3e33ad`
- Platform-core starting checkpoint: `8ba9e9b27c152cb3739d29374ec41a863b7459d2`

## Source finding

The fixed-U `mtpc_rpc_result` branch manually parses `req_msg_id`, acknowledges/routes it through Telegram Desktop's historical sent-request state, and forwards the opaque result object for method-specific handling. Fabushi's existing `EstablishedSession::invoke_raw` has exactly one outstanding content RPC, so it can represent exact current-request correlation without introducing a historical request registry.

## Implemented exact subset

Canonical platform-core:
- spec `6511c5f887c16caa0494918c987fb400774c4349`;
- implementation `144512ce2bb7366aef57ba76924b2d02852cbce3`;
- protocol api `b8b9872ec8ce7548817ebbe92615d47aed470e89`;
- protocol lib `4b6695cac59dc507f76c85f93ff3678d0d5bcbd0`;
- network `8ec039d4a12040fcea87af883dba9cd02a2b3efb`.

The protocol owner now validates the fixed-U `rpc_result` envelope, preserving a non-empty word-aligned result object unchanged. The network owner routes the current RPC branch through that parser and settles only when the parsed request id equals the current request message id. Unrelated well-formed results continue waiting.

Desktop provider synchronization:
- implementation commit `7dc33be41568162c2cab398da67b3e6bc9c12dea`;
- provider protocol api `b8b9872ec8ce7548817ebbe92615d47aed470e89`;
- provider protocol lib `4b6695cac59dc507f76c85f93ff3678d0d5bcbd0`;
- provider network `8ec039d4a12040fcea87af883dba9cd02a2b3efb`.

## Authored contracts, not run

- fixed-U `rpc_result` request-id + opaque-body parsing;
- missing/non-word-aligned result-body rejection;
- exact current-request result correlation.

No build, typecheck, lint, validator, generator, test, packaging, deployment, installation, Actions/status/check query, or runtime acceptance was run or queried.

## Deferred/fail-closed responsibilities

- arbitrary historical `wasSent` / `haveSent` request routing;
- `_resendingIds` / `toSend` cancellation;
- `_sentContainers` fan-out;
- multi-request callback/request-id ownership;
- broader sent/resend durability and concurrency;
- higher-level new-session update/difference propagation;
- remaining fixed-U source/legal/release closure and Stage-B evidence.

## Stage-A status

`implementation_status=implemented` for `TDRP-R9-MTPROTO-SINGLE-RPC-RESULT-001`.  
`validation_status=not-run`.  
`accepted=false`.  
Global source-entry counters remain `unknown=16172`, `unread=16145`, `omitted=0`.
