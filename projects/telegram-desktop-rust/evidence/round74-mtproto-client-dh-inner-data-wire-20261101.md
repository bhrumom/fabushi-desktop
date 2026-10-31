# Round 74 — MTProto client_DH_inner_data wire evidence

Date: 2026-11-01
Stage: A (implementation; executable validation deferred)
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Source readback

- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`: `client_DH_inner_data#6643b654 nonce:int128 server_nonce:int128 retry_id:long g_b:string = Client_DH_Inner_Data`; adjacent `p_q_inner_data_temp_dc#56fddf88 ... dc:int expires_in:int = P_Q_inner_data`.
- `Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03`: shipping `dhClientParamsSend` constructs `MTP_client_DH_inner_data(nonce, server_nonce, retry_id, MTP_bytes(g_b))`, then hashes/pads/encrypts it before `MTPSet_client_DH_params`. In `pqAnswered`, temporary attempts use `MTP_p_q_inner_data_temp_dc(..., protocolDcId, expiresIn)`.
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`: adjacent session context reread; no direct construction of either inner object.

## Existing-owner decision

Canonical `native/telegram-protocol/src/handshake.rs` already owns `prepare_client_dh`, including server-DH validation, random exponent generation, g_b/auth-key derivation, retry input, SHA-1 prefixing, randomized AES-block padding, AES-IGE encryption, server-salt derivation and final set_client_DH_params request preparation. No second owner was introduced.

The adjacent temporary `p_q_inner_data_temp_dc` responsibility remains fail-closed. Current `AuthKeyHandshake::prepare_req_dh_params*` exposes dc_id but not temporary-key expires_in, so an unused pure serializer would not close the fixed-U shipping lifecycle.

## Spec-first implementation

- Platform-core spec commit: `20049ba38728a50239a900a1c0a684527d427b30`.
- Platform-core implementation commit: `b5e5f9cd85ed663b0c2ab35998bc1fbf116df0e9`.
- Canonical blobs: handshake `0e5e6c0fa306f75aa1fcb5c26440b1c39f87c9f9`, lib `7273e030bf02c248d9b592b2578cb5dc9716b485`, contracts `7236df6bec3c832363901034d6a7d1c7cd3df3fc`.
- Desktop synchronization: `1de1da4bf045c405d26ff44c373ad2dd63bf9ce1`; tracked lib/contracts match canonical byte-for-byte.

The pure builder writes constructor `0x6643b654`, raw nonce, raw server_nonce, little-endian TL long retry_id, and canonical TL string/bytes framing for caller-owned g_b. Existing `prepare_client_dh` delegates only this framing and retains all cryptographic/lifecycle ownership.

## Authored, not executed

- exact little-endian constructor;
- patterned nonce/server_nonce bytes;
- nonzero retry_id exact little-endian encoding;
- five-byte g_b fixture with canonical one-byte TL length plus two padding bytes;
- reader consumes every field and finishes with no trailing bytes;
- desktop provider public facade and contract mirror synchronization.

## Fail-closed boundary

Temporary-key p_q_inner_data_temp_dc/expires_in lifecycle, connection probing/racing, message-id generation, plaintext/not-secure envelope framing, network send/retry/timeout, server response correlation/application, broader auth-key persistence/session lifecycle, arbitrary sent/resend/container histories, gzip, complete updates difference decoding/application, broader source/legal closure and independent release ACCEPT remain outside this slice.

Implementation status: implemented.
Validation status: not-run.
Accepted: false.

No GitHub Actions/status/check query or trigger and no executable validation was performed.
