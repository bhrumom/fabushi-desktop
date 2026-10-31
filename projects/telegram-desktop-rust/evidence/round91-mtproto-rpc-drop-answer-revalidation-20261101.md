# Round 91 — MTProto RpcDropAnswer result wire revalidation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 task round 60  
Stage: A (source implementation/bookkeeping only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before bookkeeping: `314b0b90fa96efe492fd3785e141a0d1390646d3`.
- Platform-core checkpoint after spec-first external contract authoring: `cf51ff32eb217bc2084b8f2cbf9c886eb2a588ec`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U `mtproto.tl` blob: `76225965bfc794388db241f3f760db8ab79e84f8`.
- Fixed-U `session_private.cpp` blob: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- Existing ledger responsibility reused: `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001`; coverage remains `responsibilities_total=97 unknown=16172 unread=16145 omitted=0`.

## Source-level finding

The fixed-U response variants `rpc_answer_unknown#5e2ad36e`, `rpc_answer_dropped_running#cd78e586`, and `rpc_answer_dropped#a43ad8b7 msg_id:long seq_no:int bytes:int` are already exactly carried by the existing canonical `telegram-protocol` owner. No production behavior change is required.

- Canonical protocol api remains `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`: `parse_rpc_drop_answer` dispatches only the three fixed-U constructors.
- Unknown and DroppedRunning are exact zero-field objects: constructor only (4 bytes), followed by strict EOF.
- Dropped is exactly 20 bytes: constructor 4 + signed `msg_id:i64` 8 + signed `seq_no:i32` 4 + signed `bytes:i32` 4.
- Canonical `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` reads signed little-endian fixed widths through bounded arrays; truncation fails closed.
- Canonical `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser/type/constructors.
- Desktop provider api/lib remain byte-identical to canonical.
- Fixed-U `session_private.cpp` contains no direct `rpc_answer_*` branch. Therefore this revalidation does not infer or add historical request drop/cancel/application semantics.

Disposition: **no production delta**.

## Spec-first and authored-only contracts

- Platform spec commit: `930a6839e8f64fafab836e43080c485507d4a4a6`, spec blob `8e8c3376dd39b93ae50d8490aedceed8f947ba12`.
- Canonical external contract commit: `cf51ff32eb217bc2084b8f2cbf9c886eb2a588ec`.
- Desktop contract mirror commit: `314b0b90fa96efe492fd3785e141a0d1390646d3`.
- Canonical/desktop external contract blob: `3c69527d941fad9eddb8c264e695666ad3e404f9`.

The new external contracts pin both exact 4-byte zero-field variants, exact 20-byte Dropped field offsets, signed `i64/i32/i32` extremes, unknown constructor rejection, every constructor/field truncation cut, and trailing-byte rejection on all variants. They were **authored only and not run**.

## Scope boundary

Outbound/history-aware `rpc_drop_answer` selection, drop/cancel scheduling, arbitrary historical request mutation, `wasSent/haveSent/_resendingIds/toSend/_sentContainers`, connection scheduling, auth-key binding/persistence, gzip, complete updates difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain open or outside this wire-only claim.

The existing ledger row remains `implemented / validation_status=not-run / accepted=false`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


## Final source-level compliance

Platform-core final compliance commit `df3feac4618636cdbc863b19e05106e19ba10446` / spec blob `2a6a5adae6b2f8bb1ebf06faa29dba88f35c958e` confirms the same no-production-delta disposition. Canonical production api/tl/lib blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `89256ab35184b6dc06a095ab34abb9f44e95a264` / `abab39e42f100972484191f0e52e9edd9b88c6cf`. Canonical and desktop authored-only external contract blob remains `3c69527d941fad9eddb8c264e695666ad3e404f9` and remains not run. The existing ledger responsibility remains `implemented / validation_status=not-run / accepted=false`; no historical request drop/cancel/application semantics are claimed.
