# Round 77 — MTProto Server_DH_Params exact union parsing

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536
Stage: A — implementation only; executable validation deferred

## Fixed-U source readback

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
`Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
defines the exact two-variant `Server_DH_Params` union:
`server_DH_params_ok#d0e8075c nonce:int128 server_nonce:int128 encrypted_answer:string` and
`server_DH_params_fail#79cb045d nonce:int128 server_nonce:int128 new_nonce_hash:int128`.

Shipping `DcKeyCreator::dhParamsAnswered` in
`Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03`
consumes this union in the auth-key handshake. Requested adjacent
`mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087` was reread and does not own this handshake union.
Broader auth-key destruction and temporary/permanent binding/persistence remain separate fail-closed lifecycle work.

## Existing-owner decision

The current canonical `native/telegram-protocol/src/handshake.rs` already owns
`AuthKeyHandshake::receive_server_dh` and `parse_server_dh_params`; no new owner is required.
The concrete gap was discriminator ordering: the parser read nonce/server_nonce before matching the constructor,
so a complete four-byte unknown constructor was reported as TL truncation instead of an unknown
`Server_DH_Params` variant.

Platform spec-first commit:
`9a6b687e043bae9f7f40e776ce1d5ec98220a168`
(`docs/specs/telegram-mtproto-server-dh-params-wire.md@8444e20eb848b61bb9ae28d99281b9fa93f5f8a3`).

Canonical implementation commit:
`274bdb92bc7e4d5755bb87fd4996398ee6cf768b`.
It moves constructor validation ahead of common-field reads while preserving nonce/server_nonce validation,
exact OK TL-bytes parsing, exact FAIL int128 parsing, and strict trailing-byte rejection.
Canonical handshake blob is `ed9eda7732f4753b442de98ebf510b8a7558b823`.
Canonical authored-contract blob is `a19c4a14d113e6c82fe3cc801f1764c555f26f76`.

Desktop sync commit `000d38bd2db19052766ccf8062eea4a6fb93c7eb` mirrors that contract blob byte-for-byte.
The desktop tracked overlay intentionally has no copied `handshake.rs`; its public facade remains
`src/lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf`, preserving the single canonical handshake core.

## Authored-only contracts

The existing protocol contract harness now covers:
- exact OK and FAIL variants;
- a four-byte unknown constructor rejected specifically as `UnexpectedServerDhConstructor`;
- every truncation cut of both representative variant bodies;
- nonce mismatch;
- server_nonce mismatch;
- trailing-byte rejection for both variants.

These contracts were authored and manually source-reviewed only. They were not executed.

## Bookkeeping state

Ledger adds `TDRP-R9-MTPROTO-SERVER-DH-PARAMS-WIRE-001`.
Coverage becomes `responsibilities_total=87`, `unknown=16172`, `unread=16145`, `omitted=0`.
The responsibility is `implemented / validation_status=not-run / accepted=false`.
Unknown/unread remain unchanged because this is a responsibility slice within already-open fixed-U source coverage,
not a whole-source-entry closure.

## Explicit fail-closed remainder

Auth-key destruction eligibility/local deletion/persistence/teardown, temporary/permanent binding/expiry/renewal,
connection scheduling/racing, plaintext send/retry/timeout, arbitrary multi-request sent/resend/container history,
gzip bounded decompression, complete updates difference/differenceSlice decoding/application, broader source/legal
closure, Stage-B executable evidence, and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


## Round 77 final source-level readback

Platform-core compliance commit `8654e737a04900b6cac8c92660fa8a8b84b449bc` updates `docs/specs/telegram-mtproto-server-dh-params-wire.md` to blob `b9b0f486aa47071ad46ba4ff5c238cee4b2bbfce` and records R1-R9 / AC-1-AC-6 source-level compliance. Canonical handshake/contracts remain `ed9eda7732f4753b442de98ebf510b8a7558b823` / `a19c4a14d113e6c82fe3cc801f1764c555f26f76`; desktop tracked contracts are byte-identical and the desktop facade remains `abab39e42f100972484191f0e52e9edd9b88c6cf` with no copied handshake core.

Ledger responsibility `TDRP-R9-MTPROTO-SERVER-DH-PARAMS-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=87`, `unknown=16172`, `unread=16145`, `omitted=0`. This source-level compliance readback is not executable verification or release acceptance.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. Broader auth-key destruction/binding/persistence/teardown, connection scheduling/racing/send lifecycle, multi-request history, gzip, complete difference decoding/application, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.
