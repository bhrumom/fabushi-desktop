# Round 96 — MTProto help.configSimple adjacent wire revalidation

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 65
Stage: A (implementation/contracts only; executable validation deferred)

## Live authority

- Desktop checkpoint before the round: `10bc676c457df1f94f42a657fa10230cf7a9bf0b`.
- Platform-core checkpoint before the round: `a9cca9a441ada6738bf3418f389c58293b3ec54e`.
- Desktop provider contract mirror checkpoint for this round: `231337450087a652b678747b98359914c20c8058`.
- Platform-core final source-level compliance checkpoint: `070ff5136f550a3c79f3839d91667857f71c9f14`.
- Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- `session_private.cpp`: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- `special_config_request.cpp`: `fdb65506b0a886bc2daeccb61090bc44be7e2d88`.
- `special_config_request.h`: `91901489e778089b03187682a9556e929867b94f`.

## Disposition

The re-reviewed responsibility is the existing `TDRP-R9-MTPROTO-CONFIG-SIMPLE-WIRE-001` slice for `ipPort#d433ad73`, `ipPortSecret#37982646`, `accessPointRule#4679b65f`, and `help.configSimple#5a592a6c`.

Fresh fixed-U readback confirms the wire family is separable from Telegram Desktop's `SpecialConfigRequest` acquisition/decryption/HTTP-time/expiry/non-empty/phone-prefix/IP conversion/DC endpoint callback lifecycle. `session_private.cpp` has no help.configSimple application branch.

Current canonical `telegram-protocol` already closes the independent wire responsibility exactly, so this round is **no production delta**. Canonical production api/lib blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf`; desktop provider api/lib are byte-identical.

## Authored-only contracts

- Spec-first revalidation: platform-core `99bb755e64a1fb96889acf0b4b673f855f5e48b6` / blob `a80fb4dd9dd7a6855fb28162ba03716daa000b09`.
- Canonical public API contract: platform-core `2d78cb1aa4ad7e7edb3a31bb596b9eb79e610d32` / blob `bc5c49ee5be958a49f16d7c14617e51e47c3961b`.
- Desktop provider mirror: `231337450087a652b678747b98359914c20c8058` / identical blob `bc5c49ee5be958a49f16d7c14617e51e47c3961b`.
- Platform-core final source-level compliance: `070ff5136f550a3c79f3839d91667857f71c9f14` / spec blob `ac08d9b91e06af12d8d794d9b146d2aaba829d60`.

The external contract preserves signed nested values and binary secrets, explicitly accepts empty wire-level `rules` instead of importing the higher non-empty policy, and rejects every truncation boundary, wrong top-level/nested/vector constructors, and strict-EOF violations across the public IpPort/AccessPointRule/help.ConfigSimple parsers. It was authored only and not executed.

## Status

The single ledger row remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted / 0 unmapped`.

Special-config lifecycle/application, arbitrary request/container history, connection scheduling, auth-key binding/persistence, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain open or outside this wire-only slice.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.
