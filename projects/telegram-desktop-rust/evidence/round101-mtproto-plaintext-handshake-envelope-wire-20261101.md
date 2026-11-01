# Round 101 — MTProto plaintext auth-key handshake envelope wire

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 70
Stage: A / zero executable validation

Fixed U remains telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4. connection_abstract.h@217b684fa222c42e968dfddeaa62f16a0054017f defines prepareNotSecurePacket as auth_key_id=0, message_id, message_data_length and request plus 0..63 random 32-bit padding words; the declared length includes padding. connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a validates zero auth key id and bounded aligned declared message data. mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03 routes the auth-key handshake through this path.

Existing-owner-first selected PlaintextEnvelope plus TcpTransport::exchange_plaintext. Spec-first platform-core checkpoint is acfc281740f3a31a17d7a50ec26327f5ddab0f8b / d999d6349b64729783695704464b89ae706698b6. Production/authored-contract checkpoint is 7b25a2cf1d690c1f441f3f45fcc92b7d0a86f057 with handshake.rs@301554c60fe318a621726cacdd37f0d5d1de2a3e, network lib.rs@54f509474391bb5118ba4678ba5c566ab3e74ec8 and contract@a7c10e88172ccd664bad0370acea8fac7a3e2f3d.

The production delta adds fixed-U-equivalent secure random 0..63 32-bit padding words and fail-closed body/padding/alignment/length validation. MessageIdGuard remains the server message-id semantic owner. Desktop mirrors existing network/contracts byte-identically at d2aaaf1a4979ad4f825532ec14e6648fd5bbc8eb; no duplicate handshake.rs was created.

Responsibility TDRP-R9-MTPROTO-PLAINTEXT-HANDSHAKE-ENVELOPE-WIRE-001 is implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false. Coverage becomes responsibilities_total=102, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 101 final source-level compliance

Platform-core final compliance is `4a3f7c2c6954d1ce69e9373b9ce058b73549e31d` / spec blob
`28ec5dea119e20220481f672e3c5bc00d1a0ddf7`, referencing desktop bookkeeping checkpoint
`0d8b5b19d9bab4b509db550f8b6ce57441b80ed5`. The unique
`TDRP-R9-MTPROTO-PLAINTEXT-HANDSHAKE-ENVELOPE-WIRE-001` responsibility is
`implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Canonical production `handshake.rs` is `301554c60fe318a621726cacdd37f0d5d1de2a3e`; canonical/desktop
telegram-network blob is `54f509474391bb5118ba4678ba5c566ab3e74ec8`; canonical/desktop authored contract blob is
`a7c10e88172ccd664bad0370acea8fac7a3e2f3d`. The existing shipping plaintext auth-key exchange now emits
fixed-U-equivalent 0..63 random 32-bit padding words with inclusive message-data length and fails closed on invalid
body/padding alignment, non-zero auth_key_id, invalid declared length and declared/actual mismatch. MessageIdGuard
remains the server message-id semantic owner.

Coverage is `responsibilities_total=102, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.
Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable
evidence and independent release ACCEPT remain open. No executable validation was performed.
