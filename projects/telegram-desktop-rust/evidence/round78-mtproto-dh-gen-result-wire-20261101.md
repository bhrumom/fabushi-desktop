# Round 78 — MTProto dh_gen result/hash exact validation

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536
Stage: A — implementation only; executable validation deferred

## Fixed-U source readback

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
`Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
defines the exact `Set_client_DH_params_answer` union:
- `dh_gen_ok#3bcbf734 nonce:int128 server_nonce:int128 new_nonce_hash1:int128`
- `dh_gen_retry#46dc1fb9 nonce:int128 server_nonce:int128 new_nonce_hash2:int128`
- `dh_gen_fail#a69dae02 nonce:int128 server_nonce:int128 new_nonce_hash3:int128`

Shipping `DcKeyCreator::dhClientParamsAnswered` in
`Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03`
validates the active nonces, authenticates the tag-specific new-nonce hash and performs success/retry/fail handling.

## Existing-owner decision

The canonical `native/telegram-protocol/src/handshake.rs` already owns
`AuthKeyHandshake::receive_dh_gen` and `parse_dh_gen_result`.
The existing implementation already:
- binds nonce and server_nonce to the active handshake;
- authenticates `SHA1(new_nonce || tag || auth_key_aux_hash_le)[4..20]` in constant time;
- uses tag 1/2/3 for OK/RETRY/FAIL;
- rejects trailing bytes;
- completes on OK;
- keeps RETRY entirely inside `AwaitingDhGen` by regenerating prepared client-DH with prior aux hash as retry_id;
- rejects FAIL without establishing a key.

Therefore broader temporary/permanent key lifetime state is not needed for this responsibility.
The concrete gap was only constructor ordering: fields were consumed before the three-way constructor match.

## Spec-first and canonical implementation

Spec-first platform commit:
`0c16a7c83a7e511484f5099399acb4c308aecb6c`
(`docs/specs/telegram-mtproto-dh-gen-result-wire.md@a43f4db8123abfcd2334fe49079b413e40c58ea6`).

Canonical implementation commit:
`5a5c6095d3e049de5d187b89c4942d7f3377ae00`.
It moves constructor discrimination ahead of nonce/server_nonce/hash reads in
`native/telegram-protocol/src/handshake.rs@87616f4b96a1775455da42b0bbed77711fb3c9f6`
while preserving all existing hash and state-transition semantics.

Canonical authored-contract commit:
`39fce63bce58718727541ac9f18dbc258766d159`.
Contract blob:
`native/telegram-protocol/tests/protocol_contract.rs@6710d6ef3fd915689792c68d69b3922b446c9bfa`.

Desktop provider contract sync:
`bhrumom/fabushi-desktop@965db14afa7104deae0fc63a76d2cc0df35d81fd`,
with byte-identical contract blob
`source/mahayana/mahayana-rs/providers/telegram-protocol/tests/protocol_contract.rs@6710d6ef3fd915689792c68d69b3922b446c9bfa`.
Desktop public facade remains
`source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf`;
no second handshake core is introduced.

## Authored-only contracts

The existing contract harness now contains source-authored cases for:
- valid OK, RETRY and FAIL with the exact tag-specific hash;
- all truncation cuts for each exact fixed-U body;
- a complete four-byte unknown constructor, which must fail as `UnexpectedDhGenConstructor`;
- active nonce mismatch;
- active server_nonce mismatch;
- OK/RETRY/FAIL bodies carrying a hash generated with the wrong variant tag;
- trailing byte rejection.

These contracts were written and manually reviewed only. They were not executed.

## Scope disposition

Closed by this slice:
- exact constructor-first three-way result discrimination;
- active nonce/server_nonce binding;
- exact tag-specific hash validation;
- strict body length/trailing validation;
- existing handshake-local retry ownership.

Still fail-closed/open:
- temporary/permanent auth-key binding, expiry, renewal, persistence and teardown;
- connection scheduling/racing, plaintext send/retry/timeout;
- arbitrary multi-request sent/resend/container history;
- gzip_packed bounded decompression;
- complete updates difference/differenceSlice decoding/application;
- broader fixed-U source/legal/release closure;
- all Stage-B executable evidence and independent release ACCEPT.

Ledger state:
`TDRP-R9-MTPROTO-DH-GEN-RESULT-WIRE-001 = implemented / validation_status=not-run / accepted=false`.
Coverage:
`responsibilities_total=88`, `unknown=16172`, `unread=16145`, `omitted=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.
