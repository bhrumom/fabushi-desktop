# Round 95 — MTProto http_wait exact wire adjacent revalidation

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 64
Stage: A (implementation only; executable validation deferred)

## Live authority

- Desktop checkpoint before this round: `9459326f0560def1bd827447636667d966d25166`.
- Platform-core checkpoint before this round: `1542bb5e40d2b3c9d4b21ac59f99cb420d7fdbb7`.
- Desktop contract mirror checkpoint: `e556ba8ee47dfad52aeaab37b622b90b92cf87b2`.
- Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- `session_private.cpp`: `13cbc76ec723386027b5b4dcac79999b791c4614`.

## Disposition

The next eligible adjacent service-wire responsibility selected after Pong is the existing `TDRP-R9-MTPROTO-HTTP-WAIT-WIRE-001` slice for `http_wait#9299359f max_delay:int wait_after:int max_wait:int = HttpWait`. It is independently reviewable only as an exact side-effect-free wire object. The fixed-U `_connection->usingHttpWait()` transport-selection/scheduling lifecycle remains explicitly outside this slice.

Current canonical production owner already closes the exact parser, so this round is **no production delta**. Production api/lib/tl blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `89256ab35184b6dc06a095ab34abb9f44e95a264`.

## Authored-only contracts

- Platform-core spec-first revalidation: `63743d4c188402e812e2f9ba9e7a46ea9ba3ff6c` / spec blob `8084780823adc82a02c5f7227bcabcd0706e0cfa`.
- Full-tree recovery fast-forward after an intermediate tree-construction mistake: `50f3ef5f748e0008c70f990e8dbd0e3896a66455`.
- Canonical external contract: `e3c19abe76b58742e80ef97713049b121e5831c1` / blob `71449041d7a7ae60be5b13379b7b8e7f417dfb8f`.
- Desktop provider mirror: `e556ba8ee47dfad52aeaab37b622b90b92cf87b2` / identical blob `71449041d7a7ae60be5b13379b7b8e7f417dfb8f`.
- Platform-core final source-level compliance: `a9cca9a441ada6738bf3418f389c58293b3ec54e` / spec blob `20dee6cba28ad8dd044314f0bdf4d5f4d1e1c54d`.

Contracts pin exact 16-byte object size, constructor and three i32 offsets, signed boundary preservation, every 0..15 truncation cut, wrong constructor, and strict EOF. They were not executed.

## Status

The sole ledger row remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted / 0 unmapped`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, test, package, deployment, installation, or runtime acceptance was performed.
