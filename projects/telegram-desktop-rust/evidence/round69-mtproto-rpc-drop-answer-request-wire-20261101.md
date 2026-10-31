# Round 69 — MTProto rpc_drop_answer request wire evidence

Date: 2026-11-01
Stage: A (implementation; executable validation deferred)
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Source readback

- Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8: `rpc_drop_answer#58e4a740 req_msg_id:long = RpcDropAnswer`.
- Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614: no direct `rpc_drop_answer` branch found in the fixed-U source; no historical RPC lifecycle is inferred.
- Existing response responsibility remains `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001`, implemented by `parse_rpc_drop_answer`.

## Spec-first implementation

- Platform-core spec amendment: `fde4b1eadc3d03c1dd943bcb5170e3434ff2771a`.
- Platform-core implementation: `37fff2cb24658b6e0f738bbbd23cecdf075f4bb1`.
- Canonical api blob: `3ec13cd2248e68f55296ed9bbaffa5c10ffce969`.
- Canonical lib blob: `e4705eb4563a087e0bba811948f1244af8f9f5cb`.
- Desktop provider synchronization: `7c17952b90cc89d575345e192733c2e6c23deb03`.
- Desktop api/lib blobs are the same canonical blobs.

The request builder emits exactly constructor `0x58e4a740` followed by one little-endian signed `i64` `req_msg_id`. Authored contracts cover representative layout, `i64::MIN`/`i64::MAX`, and rejection by the separate response parser. They were not executed.

## Fail-closed boundary

This slice does not select a historical outstanding request, schedule a drop, correlate a response, mutate callbacks, ACK/resend/container state, or implement `wasSent/haveSent/_resendingIds/toSend/_sentContainers`. It does not touch gzip or complete difference/differenceSlice decoding/application.

Implementation status: implemented.
Validation status: not-run.
Accepted: false.


## Source-level compliance readback

Platform-core compliance commit: `510b75727303147a06b6276fa94f7dcd3a7cb31b`. Spec blob: `f8408584b8ac997cad97d213fe90eb18155544f1`. Canonical/desktop protocol blobs remain api `3ec13cd2248e68f55296ed9bbaffa5c10ffce969`, lib `e4705eb4563a087e0bba811948f1244af8f9f5cb`. Request-side R9-R16 and AC7-AC9 are satisfied by source/ownership/blob inspection. AC10 executable validation remains blocked/not-run with `accepted=false`; no historical RPC lifecycle or runtime acceptance is claimed.
