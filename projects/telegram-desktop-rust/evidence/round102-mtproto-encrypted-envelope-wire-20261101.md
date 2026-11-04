# Round 102 MTProto encrypted external envelope wire evidence

Stage A only. Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Fixed-U anchors are `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a` (`AbstractConnection::prepareSecurePacket`), `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` (`sendSecureRequest` / `handleReceived`) and `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. They establish the manually framed encrypted external envelope: little-endian 64-bit auth-key id, 128-bit message key, then encrypted data.

Existing canonical production already closes this exact wire responsibility in `native/telegram-protocol/src/crypto.rs@7882db6b9f3f5e9643fb963526ccc21dd0862d73`: `EncryptedEnvelope::to_bytes/from_bytes`, `encrypt_message`, and `decrypt_message`. Shipping network remains `native/telegram-network/src/lib.rs@54f509474391bb5118ba4678ba5c566ab3e74ec8`. Therefore disposition is no-production-delta.

Platform spec: `bb84071c99141f457748825cb51ed6679ebd4c56` / `78964da9df4b91a32d077d6f608d93dc81686bb9`.
Platform authored contract: `89af115ada9770a1e21e920c9e30a5f5587833c1` / `b82069d5b7f42919e087efe053549ebddbe6e278`.
Desktop authored contract mirror: `e940ec506ea2909dc0f550d533c04f3af7b70d40` / `b82069d5b7f42919e087efe053549ebddbe6e278`.
Desktop network provider remains `54f509474391bb5118ba4678ba5c566ab3e74ec8`.

Unique ledger row: `TDRP-R9-MTPROTO-ENCRYPTED-ENVELOPE-WIRE-001`, implemented / no-production-delta / validation not-run / accepted false. Coverage: responsibilities_total 103; unknown 16172; unread 16145; omitted 0; unmapped_responsibilities 0.

No executable validation or CI/status/check query was performed.


### Round 102 final source-level compliance

Platform-core final compliance is `e449b65cbd68a287c98221d2d84da824b53707ca` / spec blob `806c1bf1f850eec6e346d5dd47f90db6f24c6501`, referencing desktop bookkeeping checkpoint `a6541989393bb6d1e44e76c542192af0cef2c3c0`. The unique `TDRP-R9-MTPROTO-ENCRYPTED-ENVELOPE-WIRE-001` responsibility is `implemented / coverage_disposition=no-production-delta / validation_status=not-run / accepted=false`.

Canonical production `crypto.rs` remains `7882db6b9f3f5e9643fb963526ccc21dd0862d73`; canonical/desktop telegram-network blob remains `54f509474391bb5118ba4678ba5c566ab3e74ec8`; canonical/desktop authored contract blob is `b82069d5b7f42919e087efe053549ebddbe6e278`. Existing `EncryptedEnvelope` preserves exact LE64 auth-key id / 16-byte message-key / ciphertext external framing and the existing shipping decrypt path fails closed on active-key mismatch, empty/non-block-aligned ciphertext and authenticated message-key failure before body dispatch.

Coverage is `responsibilities_total=103, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No executable validation was performed.
