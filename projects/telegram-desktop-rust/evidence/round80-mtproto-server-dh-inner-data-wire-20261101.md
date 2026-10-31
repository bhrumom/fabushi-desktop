# Round 80 — MTProto server_DH_inner_data wire responsibility

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536
Stage: A — implementation/bookkeeping only; executable validation intentionally not run

## Fixed source

- upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
- mtproto.tl: 76225965bfc794388db241f3f760db8ab79e84f8
- mtproto_dc_key_creator.cpp: 13c57454f14b814b5923ea253048fb881ad8cf03
- constructor: server_DH_inner_data#b5890dba

## Source-level disposition

Existing canonical production owner requires no behavior delta:
- platform-core native/telegram-protocol/src/handshake.rs@87616f4b96a1775455da42b0bbed77711fb3c9f6
- platform-core native/telegram-protocol/src/tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264
- shipping path: AuthKeyHandshake::receive_server_dh -> parse_server_dh_params -> decrypt_server_dh_inner_data -> prepare_client_dh

The existing code validates exact constructor/nonce/server_nonce/g/dh_prime/g_a/server_time fields, AES-IGE
ciphertext framing, SHA-1 of exactly the parsed inner TL object, at most 15 residual alignment-padding bytes,
and DH safety before advancing the handshake.

## Spec and authored-only contracts

- spec commit: 3bdde5a158373e3392cac987c8846d16a6fb3162
- spec blob: 5174248bd16e0d125abca66b389d0e49bc2a3f05
- canonical authored-contract commit: d724e29265e53a7ee7efd821c8bb37f6e4994cb2
- canonical contract blob: 256f421f54c41256646388833b634cf1191afd91
- desktop mirror commit: 4323b97d3b6041731d5b547118fb746d2eba37a6
- desktop contract blob: 256f421f54c41256646388833b634cf1191afd91
- desktop facade blob: abab39e42f100972484191f0e52e9edd9b88c6cf

Authored cases cover exact signed server_time preservation, empty/non-block ciphertext, unknown constructor,
all serialized-inner truncation cuts, active nonce/server_nonce mismatch, SHA-1 mismatch, residual padding above
15 bytes, invalid generator, dh_prime length/trust, empty/oversized/unsafe g_a and failure-state continuity.
These contracts were written and manually reviewed only; they were not executed.

## Ownership boundary

No second Telegram runtime/provider/handshake core was added. Temporary/permanent auth-key binding, expiry,
renewal, persistence, destruction/teardown, connection scheduling/racing, multi-request history, gzip and
complete updates difference decoding/application remain outside this responsibility.

## Ledger state

- responsibility: TDRP-R9-MTPROTO-SERVER-DH-INNER-DATA-WIRE-001
- implementation_status: implemented
- validation_status: not-run
- accepted: false
- coverage after this row: responsibilities_total=90, unknown=16172, unread=16145, omitted=0

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.
