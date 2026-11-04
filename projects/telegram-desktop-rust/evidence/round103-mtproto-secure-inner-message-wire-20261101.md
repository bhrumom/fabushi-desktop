# Round 103 MTProto secure inner message wire evidence

Stage A only. Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Fixed-U source anchor `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` shows authenticated secure plaintext as 8-byte salt, 8-byte session, 8-byte message id, 4-byte sequence number, 4-byte message length, body and 12..1024 bytes of padding, with `messageLength > 16 * 1024 * 1024` rejected before dispatch. Adjacent `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a` is the already-closed external encrypted envelope; `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` does not define a TL constructor for this inner frame.

Durable platform spec: `2f93a5c5dbfce33f3ea5ee85352f6f9b967ed042` / `00843c11318f7f3f4d5dd025d78ba0527c68abf2`.
Platform production commit: `7ffc264ab3a2f7a2997c82a03e6f8f45ce4b5745`.
Canonical crypto blob: `32d69573acc6ef4ce7bcc80073f5cdd7e2bea7ad`.
Canonical shipping network blob: `54f509474391bb5118ba4678ba5c566ab3e74ec8`.
Canonical focused contract blob: `f8e109c74a82ebe860b15c22766f403998e85b81`.
Desktop contract mirror commit: `125891339237613d9f9813c32c55684f3b5b2ecd` / blob `f8e109c74a82ebe860b15c22766f403998e85b81`.
Desktop shipping network blob: `54f509474391bb5118ba4678ba5c566ab3e74ec8`.

Production disposition is minimal existing-owner rewrite: add only the fixed-U 16 MiB body ceiling to canonical `validate_plain_message` and early `decode_plain_message`; all other secure-inner wire rules and the shipping EstablishedSession call chain remain in their existing owners.

Unique ledger row: `TDRP-R9-MTPROTO-SECURE-INNER-MESSAGE-WIRE-001`, implemented / rewritten / validation not-run / accepted false. Coverage after this row: responsibilities_total 104; unknown 16172; unread 16145; omitted 0; unmapped_responsibilities 0.

Focused contracts are authored only and were not run. No CI/status/check query or executable validation was performed.


### Round 103 final source-level compliance

Platform-core final compliance is `d6ef0db52ef2cabb480959d358386bedd2bb080e` / spec blob `b0fd5be4e048a0b6ddf737c683c21c76dbf73372`, referencing desktop bookkeeping checkpoint `fa7a222978ce3760ddb78cf5bd4c9a381434fca8`. The unique `TDRP-R9-MTPROTO-SECURE-INNER-MESSAGE-WIRE-001` responsibility is `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Canonical production `crypto.rs` is `32d69573acc6ef4ce7bcc80073f5cdd7e2bea7ad`; canonical/desktop telegram-network remains `54f509474391bb5118ba4678ba5c566ab3e74ec8`; canonical/desktop authored contract blob is `f8e109c74a82ebe860b15c22766f403998e85b81`. The canonical owner now rejects secure-inner bodies above 16 MiB both on outbound validation and early authenticated inbound decode while retaining the exact 8/8/8/4/4 header, padding, session and inbound message-id checks in the existing owner.

Coverage remains `responsibilities_total=104, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No executable validation was performed.
