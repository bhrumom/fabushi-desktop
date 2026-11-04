# Round 73 — MTProto set_client_DH_params request wire evidence

Date: 2026-11-01
Stage: A (implementation; executable validation deferred)
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Source readback

- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`: `req_DH_params#d712e4be ...` and `set_client_DH_params#f5045f1f nonce:int128 server_nonce:int128 encrypted_data:string = Set_client_DH_params_answer`.
- `Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03`: shipping `pqAnswered` sends `MTPReq_DH_params`; shipping `dhClientParamsSend` sets `Stage::WaitingDone` and sends `MTPSet_client_DH_params(attempt->data.nonce, attempt->data.server_nonce, MTP_string(...))`.
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`: adjacent session context reread; no direct `MTPReq_DH_params` or `MTPSet_client_DH_params` construction.

## Existing req_DH_params closure

Canonical `build_req_dh_params` already writes the fixed-U constructor and all six request fields in schema order. The existing authored contract parses constructor, nonce, server_nonce, p, q, fingerprint and encrypted_data and asserts the reader is finished. No req_DH_params production change was needed this round.

## Spec-first implementation

- Platform-core spec commit: `f3d85c368ace17c864eebf2b4d97a44c9d3d964f`.
- Platform-core implementation: `a353cdf718a93f8e51f48e58c1be4a9889c3bc32`.
- Canonical blobs: handshake `2e9acc80d10a87a8633173c88568aef706d4e123`, lib `3a05251d749f0df868e4ebd94fb7e28e31fad8f1`, contracts `3cef7f7532cf98c46f9f9eb06f9b0ff2026dfcb4`.
- Desktop synchronization: `85db8d62f8014614234ab2340560367d087ac4e0`; tracked lib/contracts match canonical byte-for-byte.

The pure builder writes constructor `0xf5045f1f`, the two caller-owned raw int128 nonce values, and canonical TL string/bytes framing for caller-owned encrypted_data. Existing `prepare_client_dh` still owns DH validation, randomness, auth-key derivation, retry id, temporary AES encryption, server-salt derivation and `PreparedClientDh`; it delegates only final request-body framing.

## Authored, not executed

- exact little-endian constructor plus patterned nonce/server_nonce;
- five-byte encrypted_data fixture with canonical one-byte TL length and two padding bytes;
- TL reader consumes encrypted_data and finishes with no trailing body;
- existing prepare_client_dh request body equals `build_set_client_dh_params` over its already-produced encrypted_data;
- existing req_DH_params exact field/trailing contract remains unchanged;
- public facade export and desktop provider synchronization.

## Fail-closed boundary

This slice does not own or expand endpoint probing, transport racing, message-id generation, plaintext/not-secure envelope framing, network send/retry/timeout, server response correlation, broader RSA/DH/auth-key lifecycle, auth-key persistence, session teardown/reconnect, arbitrary sent/resend/container histories, gzip, or complete updates difference decoding/application.

Implementation status: implemented.
Validation status: not-run.
Accepted: false.

No GitHub Actions/status/check query or trigger and no executable validation was performed.


### Round 73 source-level compliance readback

Platform-core compliance commit `dd584145d268084545e082c399e9f5fe2e52cba9` updates `docs/specs/telegram-mtproto-set-client-dh-params-request-wire.md@7ec57e2cfa47f62f0ae372e776ec8a1dcb39c04d`. Remote readback confirms canonical `build_set_client_dh_params`, its public facade export, and `prepare_client_dh` body-only delegation are present; canonical handshake/lib/contracts blobs are `2e9acc80d10a87a8633173c88568aef706d4e123` / `3a05251d749f0df868e4ebd94fb7e28e31fad8f1` / `3cef7f7532cf98c46f9f9eb06f9b0ff2026dfcb4`. Desktop tracked lib/contracts are byte-identical. Existing `build_req_dh_params` and its complete field-order/trailing contract remain present and unchanged.

Spec R1-R8 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-SET-CLIENT-DH-PARAMS-REQUEST-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; `responsibilities_total=83`, `unknown=16172`, `unread=16145`, `omitted=0`. Stage-B executable evidence is intentionally absent.

Connection probing/racing, message-id generation, plaintext/not-secure framing, network send/retry/timeout, server response correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.
