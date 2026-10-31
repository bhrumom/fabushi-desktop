# Round 70 — MTProto destroy_auth_key request wire evidence

Date: 2026-11-01
Stage: A (implementation; executable validation deferred)
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Source readback

- Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8: `destroy_auth_key#d1435160 = DestroyAuthKeyRes`.
- Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614 was reread for adjacent lifecycle context; no lifecycle behavior is imported into this wire-only slice.

## Spec-first implementation

- Platform-core spec commit: `529f5c9147f0144b67eb077733b52d61b469f55d`.
- Platform-core implementation: `8eafd0cc7a84ab9af6b402665670b38f3ad72406`.
- Canonical api/lib blobs: `fbca9b0ebb85d0edd5c01ceea9896d0977294b53` / `ed581d8356375e264fabe85a2e5cccbd9b772f21`.
- Desktop synchronization: `fe0093d767ad2a12e049b4d0a474c0145d3468ab`, with identical api/lib blobs.

The builder emits exactly the little-endian constructor `0xd1435160` and no payload. The exact-layout contract is authored but not executed.

## Fail-closed boundary

This slice does not decide auth-key destruction eligibility, parse/apply a remote DestroyAuthKeyRes, delete or rotate a local auth key, mutate persistence, tear down/reconnect transports, or touch arbitrary sent/resend/container history, gzip, or complete updates difference decoding/application.

Implementation status: implemented.
Validation status: not-run.
Accepted: false.
