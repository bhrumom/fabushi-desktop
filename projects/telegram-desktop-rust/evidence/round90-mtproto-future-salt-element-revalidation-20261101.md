# Round 90 — MTProto future_salt nested element wire revalidation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 task round 59  
Stage: A (source implementation/bookkeeping only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before this bookkeeping write: `b5891b2907d6ca72fb1a5385f27f29ad4b0e215e`.
- Platform-core checkpoint after spec-first contract authoring: `8b9d113599b12d0d0a7fdd250b75aab87f881821`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U `mtproto.tl` blob: `76225965bfc794388db241f3f760db8ab79e84f8`.
- Fixed-U `session_private.cpp` blob: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- Ledger before this bookkeeping write: `69d895236bf50b93bd788935132c2d4062e8d710`; coverage `responsibilities_total=97 unknown=16172 unread=16145 omitted=0`.

## Source-level finding

The nested `future_salt#0949d9dc valid_since:int valid_until:int salt:long` responsibility is already closed inside the existing `TDRP-R9-MTPROTO-FUTURE-SALTS-WIRE-001` owner; no production behavior change is required.

- Canonical protocol api `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` stores `valid_since`/ `valid_until` as signed `i32` and `salt` as signed `i64`.
- The parser uses `FUTURE_SALT_WIRE_BYTES = 20`: nested constructor 4 bytes plus exact fields 4/4/8 bytes.
- Declared vector count is multiplied by 20 with checked arithmetic and compared with remaining bytes before `Vec::with_capacity(count)`.
- Each element validates `FUTURE_SALT_CONSTRUCTOR` before fixed-size field reads; shared `TlReader` blob `89256ab35184b6dc06a095ab34abb9f44e95a264` uses fixed-size little-endian arrays and fails closed on truncation.
- Strict EOF rejects bytes after the last declared element.
- Canonical network blob `1fc1b1f69e3e37377400d078f916599e0c7b20bc` calls `parse_future_salts` before current-request matching/settlement.
- Fixed-U `session_private.cpp` contains adjacent decoded service-message handling but no dedicated `future_salts` application branch; adoption/rotation/scheduling/persistence is therefore not part of this wire-only claim.

## Spec-first and authored-only contracts

- Platform spec commit: `9797db85df1aa286511296887463ef44451c133b`, spec blob `16dd5b50297449b467108767cf70bc4c8243310a`.
- Canonical external-contract commit: `8b9d113599b12d0d0a7fdd250b75aab87f881821`.
- Desktop mirror commit: `b5891b2907d6ca72fb1a5385f27f29ad4b0e215e`.
- Canonical/desktop external contract blob: `0f99a2c99b377920792316de433c2229bc781e60`.

The new external contracts pin two exact 20-byte elements, signed i32/i32/i64 boundary values, byte offsets for both vector elements, wrong nested constructor rejection, truncation at constructor and each field boundary, impossible declared count, and trailing-byte rejection. They were **not run**.

## Scope boundary

Future-salt installation/rotation, request scheduling, persistence, retry, arbitrary request history, gzip, complete updates difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open or outside this nested wire slice.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.
