# Round 71 — MTProto req_pq request wire evidence

Date: 2026-11-01
Stage: A (implementation; executable validation deferred)
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Source readback

- Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8: `req_pq#60469778 nonce:int128 = ResPQ`.
- Telegram/SourceFiles/mtproto/connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a: `AbstractConnection::preparePQFake` constructs `MTPReq_pq(nonce)` and then delegates not-secure packet/message-id wrapping.
- Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614 was reread as adjacent session context and contains no `req_pq` branch.

## Spec-first implementation

- Platform-core spec commit: `c0b1b72f501822023e73f0e971a41bb0bfe38baf`.
- Platform-core implementation: `48b8dcb9fc7d6f7a79c1ccdce59041493a7a3b7a`.
- Canonical api/lib blobs: `54ab072495a4f8f1eca322a7f8f46c5dcd4396ee` / `8df4760c6bf6dbeeba3e8d8b72232bac6a410ff0`.
- Desktop synchronization: `9e3065eddd2d0497dfb4ea2ed7a38d4ee7acb13f`, with identical api/lib blobs.

The builder emits exactly 20 bytes: little-endian constructor `0x60469778` followed by the caller's 16 nonce bytes unchanged. The exact-layout/raw-int128 contracts are authored but not executed.

## Fail-closed boundary

This slice does not generate the nonce, schedule or race connection probes, build not-secure packets, generate MTProto message ids, send/retry/timeout requests, correlate or apply ResPQ, select req_pq_multi, or mutate auth-key/session/persistence state. Arbitrary sent/resend/container history, gzip, and complete updates difference decoding/application remain outside this slice.

Implementation status: implemented.
Validation status: not-run.
Accepted: false.
