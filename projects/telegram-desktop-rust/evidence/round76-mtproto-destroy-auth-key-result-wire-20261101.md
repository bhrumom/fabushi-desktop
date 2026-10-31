# Round 76 — MTProto DestroyAuthKeyRes exact wire parser

Date: 2026-11-01
Stage: A (implementation; executable validation deferred)
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Source readback and adjacent disposition

- `scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines the four p_q inner variants and the three zero-field `DestroyAuthKeyRes` variants.
- `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` requires `protocolDcId` and selects only `p_q_inner_data_dc` or `p_q_inner_data_temp_dc`; the no-dc `p_q_inner_data` and `p_q_inner_data_temp` constructors are not selected by the fixed-U shipping auth-key path, so no duplicate canonical builders were added.
- `bind_auth_key_inner` remains blocked on broader temporary/permanent key binding, expiry and session lifecycle ownership.
- `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087` has a shipping key-destroyer path that reads `MTPDestroyAuthKeyRes` and distinguishes ok/fail/none.

## Existing-owner implementation

The exact response union belongs to the existing canonical `native/telegram-protocol/src/api.rs` owner. Spec-first commit: `791308b5c885b15b4f69b395ae0df0a75f47f634`. Canonical implementation: `ebb1e5f859e224c45f9d34beb1aa32b4c18ca4b2`.

Implementation adds:
- `DESTROY_AUTH_KEY_OK_CONSTRUCTOR=0xf660e1d4`;
- `DESTROY_AUTH_KEY_NONE_CONSTRUCTOR=0x0a9f2259`;
- `DESTROY_AUTH_KEY_FAIL_CONSTRUCTOR=0xea109b13`;
- typed `DestroyAuthKeyResult::{Ok,None,Fail}`;
- strict `parse_destroy_auth_key_result` with no-trailing enforcement;
- public facade exports;
- authored contracts for all three variants, unknown constructor, truncation and trailing bytes.

Canonical api/lib/contracts blobs: `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`, `abab39e42f100972484191f0e52e9edd9b88c6cf`, `a53c90f09ec18f8e99d8cfb36eb8c8842f27635e`. Desktop sync commit `4b152da131b17558fcad7c1aee45fd0cec3269e6` mirrors all three byte-for-byte.

## Fail-closed boundary

This parser does not decide destruction eligibility and does not delete local keys, mutate persistence, tear down/reconnect sessions, retry requests, or own request history. Those broader destructive lifecycle semantics remain open. Temporary/permanent binding, expiry enforcement and renewal/persistence also remain open.

Implementation status: implemented.
Validation status: not-run.
Accepted: false.

No executable validation was performed. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.

## Round 76 final source-level readback

Platform-core compliance commit `42c7ac4d193f7f1872a9bfd87ce70b2af973c2df` updates `docs/specs/telegram-mtproto-destroy-auth-key-result-wire.md` to blob `da653d871e78a8a97770449034d0ac4db2f71cfe` and records R1-R8 source-level compliance. Canonical api/lib/contracts remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `a53c90f09ec18f8e99d8cfb36eb8c8842f27635e`; desktop tracked provider files are byte-identical.

The ledger responsibility `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-RESULT-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=86`, `unknown=16172`, `unread=16145`, `omitted=0`. Adjacent `p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` remain dispositioned as non-shipping in fixed-U `DcKeyCreator`; `bind_auth_key_inner#75a3f765` and broader auth-key destruction/binding/persistence/teardown lifecycle remain fail-closed.

No executable validation was performed. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.

