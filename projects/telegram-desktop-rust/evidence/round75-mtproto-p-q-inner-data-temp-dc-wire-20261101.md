# Round 75 — MTProto p_q_inner_data_temp_dc wire + temporary Req_DH_params preparation

Date: 2026-11-01
Stage: A (implementation; executable validation deferred)
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Source readback

- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `p_q_inner_data_temp_dc#56fddf88 ... dc:int expires_in:int`.
- `Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` stores `expiresIn` when `pqSend` starts and selects `MTP_p_q_inner_data_temp_dc(..., protocolDcId, expiresIn)` after `resPQ` when the attempt is temporary. The value has no later consumer in the DH stages.
- Canonical Fabushi `AuthKeyHandshake` already owns `ResPqReceived -> Req_DH_params -> AwaitingServerDh`; the existing persistent path previously always serialized `p_q_inner_data_dc`.

## Existing-owner decision

The temporary wire/request-preparation responsibility fits the existing `native/telegram-protocol/src/handshake.rs` owner. No second handshake, provider, network stack, request history, scheduler or auth-key store is introduced.

Broader temporary-key establishment does not yet fit the existing runtime persistence owner: `PersistedTransportState` stores auth-key material without expiry/binding metadata. Therefore telegram-network main establishment and runtime bootstrap remain persistent and temporary persistence/binding stays fail-closed.

## Spec-first + implementation

- Initial spec: `93283c35d075a16498b672e548071e16bdafd2c3`.
- Boundary correction after persistence-owner readback: `8f09112d7af1875e19eddb7096ef1a0a05347d8d`.
- Canonical handshake implementation: `af7b37f57be70a88cd10728d79d352c7aade88f5`, blob `3e740e0714dfca71bb908ec46f182e98605582e5`.
- Canonical facade: `68c20daadfc680e24393c63d0505feb5fee4783e`, blob `a2dc3f48a15c7a14657f598d7a5ce4b1fed9ed7f`.
- Authored canonical contracts: `bcd516892fa49407a4de757e05235ed585ad4338`, blob `748ca5f7962ff0704a630144c7368c067d212de3`.
- Desktop facade/contracts: `aafca841fd745f3aee2deb9b89921f6d34021361` / `bd7c3a1c648107c92a265c76c46bae1da745f7e9`, with blobs byte-identical to canonical.

Implementation adds `P_Q_INNER_DATA_TEMP_DC_CONSTRUCTOR=0x56fddf88`, pure `build_p_q_inner_data_temp_dc`, positive lifetime validation, and `AuthKeyHandshake::prepare_temporary_req_dh_params*`. The temporary path reuses factorization, trusted RSA fingerprint selection, new_nonce generation, RSA_PAD, `build_req_dh_params`, and the existing state transition. Existing `prepare_req_dh_params*` remains the persistent path.

## Authored, not executed

Contracts cover exact constructor/field order, pq/p/q TL bytes, raw nonce/server_nonce/new_nonce, dc_id, expires_in=86400, no trailing bytes, zero lifetime rejection, no state mutation on invalid lifetime, and successful temporary preparation reaching `AwaitingServerDh`. No contract was executed.

## Fail-closed boundary

telegram-network/runtime temporary-key establishment, local expiry enforcement, permanent-key binding, renewal/persistence, connection scheduling/racing, plaintext framing, send/retry/timeout, arbitrary sent/resend/container histories, gzip, complete difference decoding/application, broader source/legal closure and independent release ACCEPT remain open.

Implementation status: implemented.
Validation status: not-run.
Accepted: false.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


## Final source-level readback

Platform-core compliance commit `d04917b2ab583a6b143ab7d0925a38bda4e7ac14` updates the durable spec to blob `f7f1b220b79c8ab92f49d6a07054ffc2821b8a3c` and records requirement-by-requirement source compliance. Canonical handshake/facade/contracts remain `3e740e0714dfca71bb908ec46f182e98605582e5` / `a2dc3f48a15c7a14657f598d7a5ce4b1fed9ed7f` / `748ca5f7962ff0704a630144c7368c067d212de3`. Desktop tracked facade/contracts remain byte-identical.

Remote ledger readback confirms `TDRP-R9-MTPROTO-P-Q-INNER-DATA-TEMP-DC-WIRE-001` is `implemented / validation_status=not-run / accepted=false`; ledger blob `971489ca6ff27838b1ef884a24ced884f60da4ea` reports `responsibilities_total=85`, `unknown=16172`, `unread=16145`, `omitted=0`.

Source readback confirms both selection branches in the single canonical helper: `Some(expires_in)` selects `build_p_q_inner_data_temp_dc`, while `None` keeps `build_p_q_inner_data_dc`. Non-positive temporary lifetime is rejected before the shared helper/state mutation. telegram-network/runtime temporary establishment remains deliberately unchanged/fail-closed because current persisted transport material lacks expiry/binding metadata.

No executable validation was performed. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


## Final wording correction readback

Platform-core commit `6e6aad98d390163bef77b1e21b8481c6e7c046d0` corrects the durable spec wording so it no longer implies a runtime `temporaryExpiresIn` input is implemented. Final spec blob is `08b7734d8c42c90cf10fb62eb1d21426f1de9254`. The implemented boundary remains: canonical `AuthKeyHandshake::prepare_temporary_req_dh_params*` owns temporary Req_DH_params preparation; telegram-network/runtime temporary establishment and expiry/binding persistence remain unavailable/fail-closed. No production code changed in this wording correction and no executable validation was performed.
