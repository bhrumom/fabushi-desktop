# Round 100 — MTProto req_DH_params request wire

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 69  
Stage: A / zero executable validation

## Source readback

- fixed U: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
- mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8 defines req_DH_params#d712e4be nonce:int128 server_nonce:int128 p:string q:string public_key_fingerprint:long encrypted_data:string = Server_DH_Params.
- mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03 shows DcKeyCreator::pqAnswered factors the active resPQ, chooses the offered server RSA key, encrypts the p_q inner object, then sends MTPReq_DH_params with the active nonce/server_nonce/factors/fingerprint/encrypted_data.

## Existing-owner implementation

Canonical owner remains native/telegram-protocol/src/handshake.rs AuthKeyHandshake/build_req_dh_params. Spec-first commit is 005bf696ba89b32a8965e510069f5867ed921b73 / spec blob 759586ea8e7b07cd9818da7620577512031ae553.

Independent review found the shipping path exact but the public pure builder lacked its own factor-consistency rejection. Platform implementation commit 39f9ba32d0ae6bcb03d7f957f708d690ec5fb74e changes handshake.rs to b6572c91a450a240c56c15ca80585afb97c19839 and rejects factors unless response.pq == factors.pq and p*q == pq, using existing HandshakeError::PqFactorsDoNotMatch. Existing offered-fingerprint and non-empty encrypted-data checks remain.

AuthKeyHandshake::prepare_req_dh_params* still derives factors from the active ResPq, selects the trusted RSA key, builds/encrypts the already-owned *_dc p_q inner-data variant, calls build_req_dh_params, then advances to AwaitingServerDh. No binding/persistence or network scheduling ownership was added.

## Authored-only contracts and desktop mirror

Canonical contract blob is 071b72be6772117fb174daf86d5c9a19ce372b73. It covers exact constructor/nonce/server_nonce/p/q/fingerprint/encrypted_data order and EOF, plus PqFactorsDoNotMatch, NoTrustedServerKey, and EmptyEncryptedInnerData failures. It was not executed.

Desktop has no tracked duplicate handshake.rs in this provider surface, so none was created. Existing tracked protocol_contract.rs was mirrored byte-identically at 0e0188f5d1ebf96d2ee21444e1f7b65994c88ee2 / 071b72be6772117fb174daf86d5c9a19ce372b73. Desktop lib remains abab39e42f100972484191f0e52e9edd9b88c6cf.

## Disposition

Responsibility: TDRP-R9-MTPROTO-REQ-DH-PARAMS-REQUEST-WIRE-001

Status: implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false.

Coverage becomes responsibilities_total=101, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

Auth-key binding/persistence/expiry/renewal, connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 100 final source-level compliance

Platform-core final compliance is `8832c2ce26819033ea7d06c113ab97345ceda290` / spec blob
`ac617d4c87a6fe199c1afd5f541f5f12e1ddafc9`, referencing desktop bookkeeping checkpoint
`cb082b48228fbb2ee43021f41592b605d072bfb9`. The unique
`TDRP-R9-MTPROTO-REQ-DH-PARAMS-REQUEST-WIRE-001` responsibility is
`implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Canonical production `handshake.rs` is `b6572c91a450a240c56c15ca80585afb97c19839`; unchanged canonical/desktop lib is
`abab39e42f100972484191f0e52e9edd9b88c6cf`. Canonical and desktop authored contract blob is
`071b72be6772117fb174daf86d5c9a19ce372b73`. The builder now fails closed on inconsistent active resPQ factors while preserving the existing exact req_DH_params layout, fingerprint membership and non-empty encrypted-data guards.

Coverage is `responsibilities_total=101, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.
Auth-key binding/persistence/expiry/renewal, connection scheduling/racing/send retry, arbitrary request histories,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable
evidence and independent release ACCEPT remain open. No executable validation was performed.
