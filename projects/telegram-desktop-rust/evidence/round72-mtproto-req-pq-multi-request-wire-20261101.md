# Round 72 — MTProto req_pq_multi request wire evidence

Date: 2026-11-01
Stage: A (implementation; executable validation deferred)
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Source readback

- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`: `req_pq_multi#be7e8ef1 nonce:int128 = ResPQ`.
- `Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03`: shipping `DcKeyCreator::pqSend` sets `Stage::WaitingPQ`, assigns a random attempt nonce, and calls `sendNotSecureRequest(MTPReq_pq_multi(attempt->data.nonce))`.
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`: adjacent session context reread; no `req_pq_multi` branch.

## Spec-first implementation

- Platform-core spec commit: `c5b3021a760892eaf05fcd9b40dc35dff4c2c783`.
- Platform-core initial implementation: `60e4b9165c0f41804bf68e7270aa52f39e824f14`.
- Platform-core canonicalization: `591176a5ba091bb2a00f925d38ed87a1fdae0b11`.
- Canonical blobs: api `3e3b250b709ce2f307f896dda6d1c2da55559f7b`, handshake `b1b22af0e83b577d6152beb11a75e44eb76ffdbe`, lib `2d48c2fe1c677b4b4672e7e76d8bd0a29a7ac003`, contracts `bf5aeb85f10baeeb7e645a660fd3bf26259e5989`.
- Desktop synchronization: `59e08389bae4ab2cdcbb3f67808da44dff452ca9`; tracked api/lib/contracts match the canonical blobs byte-for-byte.

The pure builder emits exactly 20 bytes: little-endian constructor `0xbe7e8ef1` followed by the caller's 16 nonce bytes unchanged. Existing `AuthKeyHandshake::begin` reuses only that body serializer; its state guard, message-id input, transition to `AwaitingResPq`, and `PlaintextEnvelope` behavior remain in the pre-existing handshake owner.

## Authored, not executed

- exact constructor + patterned raw int128 nonce;
- zero and 0xff nonce identity without TL bytes framing;
- existing begin-path body equals `build_req_pq_multi(&nonce)`;
- public facade export and desktop provider synchronization.

## Fail-closed boundary

This slice does not own or expand endpoint probing, transport racing, nonce/message-id generation, plaintext/not-secure envelope framing, network send/retry/timeout, response correlation, ResPQ application, RSA/DH progression, auth-key persistence, session teardown/reconnect, arbitrary sent/resend/container histories, gzip, or complete updates difference decoding/application.

Implementation status: implemented.
Validation status: not-run.
Accepted: false.

No GitHub Actions/status/check query or trigger and no executable validation was performed.
