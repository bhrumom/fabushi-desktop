# Round 54 — MTProto single-RPC message-container traversal

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 23  
Stage: A — implementation only, zero CI

## Fixed authorities

- Upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- Desktop starting checkpoint: `b30c7374e3a926765a2238ed9045cf7647c71aa8`
- Platform-core starting checkpoint: `c8defba26a6ade40ddd97445233809fb366580a7`

## Source finding

The adjacent fixed-U receive-service branches not yet independently mapped were `mtpc_gzip_packed` and `mtpc_msg_container`. `msg_container` is exactly representable by existing protocol/network owners on the current RPC/current ping receive path: bounded parse, inner message-id replay validation, odd-sequence current-RPC ACK accumulation, and recursive dispatch. `gzip_packed` is not folded into this slice because fixed U requires a separate decompression nesting/unpacked-budget lifecycle.

## Implemented exact subset

Canonical platform-core:
- spec `b223407bc9829a2fc14449bb5a7cfe6d7f4b082e`;
- implementation `98a2ff425407725571966a4ed17b1643fba87b4d`;
- protocol api `14c282bfc20a33446c69a47db4fdd6f0b887348a`;
- protocol lib `f21d38495e5a7f5c6e2a35f28a633ac6e3433d4b`;
- network `bab8893bd8a2a0632014d826503a8091cfe64d46`.

Desktop provider synchronization:
- implementation commit `e8091133b85eb7e368839e6c9b09547620be17c2`;
- provider protocol api `14c282bfc20a33446c69a47db4fdd6f0b887348a`;
- provider protocol lib `f21d38495e5a7f5c6e2a35f28a633ac6e3433d4b`;
- provider network `bab8893bd8a2a0632014d826503a8091cfe64d46`.

## Authored contracts, not run

- successful fixed-U multi-entry message-container parsing with opaque inner bodies;
- rejection of zero/unaligned body length, oversized count, and trailing bytes;
- current-RPC ACK accumulation predicate from contained `seq_no` parity.

No build, typecheck, lint, validator, generator, test, packaging, deployment, installation, Actions/status/check query, or runtime acceptance was run or queried.

## Deferred/fail-closed responsibilities

- `gzip_packed` decompression, nesting and unpacked-byte budget;
- arbitrary outbound/historical `wasSent` / `haveSent` request routing;
- `_resendingIds` / `toSend` cancellation;
- outbound `_sentContainers` fan-out;
- multi-request callback/request-id ownership and broader sent/resend durability;
- higher-level new-session update/difference propagation;
- remaining fixed-U source/legal/release closure and Stage-B evidence.

## Stage-A status

`implementation_status=implemented` for `TDRP-R9-MTPROTO-SINGLE-RPC-MESSAGE-CONTAINER-001`.  
`validation_status=not-run`.  
`accepted=false`.  
Global source-entry counters remain `unknown=16172`, `unread=16145`, `omitted=0`.
