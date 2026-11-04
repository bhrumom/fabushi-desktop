# Round 55 — MTProto new-detailed-info exact wire validation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 24  
Stage: A — implementation only, zero CI

## Fixed authorities

- Upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- Desktop starting checkpoint: `b3ed649d1fc8d0ed1af6d0229489df82ecd42a78`
- Platform-core starting checkpoint: `b477aac2fe82fadea6d513872fa368eb2a8c9e48`

## Gzip blocker

Fixed U sets `kMaxUnpackedMessageLength=32 MiB`, `kMaxGzipNesting=64`, shares one `unpackedBudget` through recursive receive handling, and uses zlib gzip inflate. Canonical protocol dependency inputs `Cargo.toml@d8a5261034b1eb395d12149a6bcc34cd7e371b73` and `Cargo.lock@97ad9046ce8926764258c978664cd8cd92fce503` have no `flate2` or `miniz_oxide` owner. The gzip responsibility therefore remains fail-closed/open; no compression dependency was added.

## Implemented next independent responsibility

Fixed-U `msg_new_detailed_info#809db6df` carries `answer_msg_id:long bytes:int status:int`. The pre-existing Fabushi branch read only `answer_msg_id` by byte offset and could therefore accept a truncated or trailing object before invoking reconciliation.

Canonical platform-core:
- spec `40d53cfb44f2a495e4bf1a84002f653f555532e5`;
- implementation `5bdf993dc1048ed22194d7f1c0439049e820ee5f`;
- protocol api `eff20dbc601ca78fa7e6a4662bfeda088706d4f4`;
- protocol lib `b6ab0397cbad0776b269d965f89427030824ba4c`;
- network `7f3b140a9becda2305a2f5e252fcea61108597e0`.

Desktop provider synchronization:
- implementation commit `3b37ff49e7c3b100af12206ab88662f4d83959c2`;
- provider protocol api `eff20dbc601ca78fa7e6a4662bfeda088706d4f4`;
- provider protocol lib `b6ab0397cbad0776b269d965f89427030824ba4c`;
- provider network `7f3b140a9becda2305a2f5e252fcea61108597e0`.

The typed parser consumes all fixed-U fields and requires exact full-buffer consumption. Network reconciliation now occurs only after successful parsing. Existing received-message history still decides ACK-versus-resend; no historical sent-request/container owner was added.

## Authored contracts, not run

- exact `answer_msg_id/bytes/status` parsing;
- truncation rejection;
- trailing-byte rejection;
- existing current-session detailed-answer ACK/resend reconciliation remains the network contract.

No build, typecheck, lint, validator, generator, test, packaging, deployment, installation, Actions/status/check query, or runtime acceptance was run or queried.

## Deferred/fail-closed responsibilities

- `gzip_packed` decompression, 64-level nesting guard, and 32 MiB shared unpacked-byte budget;
- arbitrary `wasSent` / `haveSent` request history;
- `_resendingIds` / `toSend` cancellation;
- outbound `_sentContainers` fan-out;
- multi-request callback/request-id ownership and broader sent/resend durability;
- higher-level new-session update/difference propagation;
- remaining fixed-U source/legal/release closure and Stage-B evidence.

## Stage-A status

`implementation_status=implemented` for `TDRP-R9-MTPROTO-NEW-DETAILED-INFO-WIRE-001`.  
`validation_status=not-run`.  
`accepted=false`.  
Global source-entry counters remain `unknown=16172`, `unread=16145`, `omitted=0`.
