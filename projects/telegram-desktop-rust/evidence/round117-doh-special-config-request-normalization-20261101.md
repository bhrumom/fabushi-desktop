# Round 117 — DoH / special-config request normalization

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 86  
Stage: A, deferred executable validation

## Fixed source

- `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/details/mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9`
- `Telegram/SourceFiles/mtproto/special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88`

## Source finding

The request-shaping subset is independent of request scheduling: `DnsDomains`, `GenerateDnsRandomPadding`, `DnsUserAgent`, Google/Mozilla scheme/URL-host/path/query/Host/Accept generation, and Firestore direct/camouflage URL-host/path/Host generation can be represented as pure request descriptors. The non-deterministic padding boundary needs entropy but no scheduler; the exact fixed-U rejection/count/alphabet semantics are preserved with an explicit entropy-injection function plus a production wrapper using the existing network crate `getrandom` dependency.

The remaining carrier responsibility starts only when a normalized descriptor is actually dispatched through a direct no-proxy HTTP client, raced on the fixed 800 ms schedule, cancelled/finalized, and later connected to global HTTP-time mutation or temporary-DC/dc_options application.

## Existing-owner-first implementation

- Platform spec-first commit: `997c7b12cdc8b22368b6f1a04490ea7b602294bc`
- Spec blob: `0983f2ae91648e1d8a48b553df6bd98c67dd3553`
- Platform implementation/authored-contract commit: `9751072e735630accbbb921178c737b609dbb039`
- Network lib blob: `3743331b0b1453048a98a21ae4ac2d2b91a2b2a5`
- Request-normalization module blob: `2f3ea627054779296102f13c70be17a3655ddcb9`
- Authored contract blob: `55d55d5de956cad3d809d64efc97993d270dcd97`
- Desktop initial mirror commit: `688ba8dff5fb415736c8cee9d397d97bb0fe1837`
- Platform source-review vector correction: `a65278227c78fac5fd5a6ac41c841b4fc509fcb5`
- Desktop byte-identical correction mirror: `712bde77f6f232efafe775ff9390faa47e9595fe`

No Cargo.toml or Cargo.lock delta was made. No new HTTP/bootstrap runtime, settings owner, DC directory or response parser was introduced.

## Ledger disposition

- `TDRP-R9-MTPROTO-REQUEST-NORMALIZATION-001`: `implemented / rewritten / validation_status=not-run / accepted=false`.
- `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001`: remains `mapped / pending / validation_status=not-run / accepted=false`; remaining scope is actual HTTP dispatch/racing/reply lifecycle, global HTTP-time mutation, temporary-DC help.getConfig lifecycle and dc_options application.
- Coverage: `source_entries_total=16202, responsibilities_total=120, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

## Deferred contracts

Internal and public contracts were authored for constants, padding rejection/range/alphabet, Google/Mozilla A/AAAA and ANY/TXT shapes, and Firestore direct/camouflage Host behavior. They were not executed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Source-review contract correction

Before final compliance, manual source review found that one authored-only padding test fed four rejected count bytes before its character bytes while asserting a prefix for a different sequence. No executable test was run. The vector input was corrected so the injected entropy deterministically consumes rejected count bytes `0, 23`, accepts `25 -> count 13`, then maps character bytes `0,25,61,62,63,26` to `az9abA`. Production padding logic was unchanged.

Platform correction commit: `a65278227c78fac5fd5a6ac41c841b4fc509fcb5`. Desktop correction mirror: `712bde77f6f232efafe775ff9390faa47e9595fe`. Final module/contract blobs: `2f3ea627054779296102f13c70be17a3655ddcb9` / `55d55d5de956cad3d809d64efc97993d270dcd97`. Network lib remains `3743331b0b1453048a98a21ae4ac2d2b91a2b2a5`.

Executable validation remains intentionally not-run.


### Round 117 final source-level compliance

Platform-core final source-level compliance is `445f49beed303bd7dbef2f8b998bfb22d8aaadce` / spec blob `94c6b75c471167009aacec4bdb3fd4d00cb2bb27`. It records R1-R13 plus AC-1 through AC-8 as passed at source/implementation/bookkeeping review level.

Final canonical and desktop telegram-network lib/request-normalization/authored-contract blobs are byte-identical at `3743331b0b1453048a98a21ae4ac2d2b91a2b2a5`, `2f3ea627054779296102f13c70be17a3655ddcb9`, and `55d55d5de956cad3d809d64efc97993d270dcd97`. `TDRP-R9-MTPROTO-REQUEST-NORMALIZATION-001` remains `implemented / rewritten / validation_status=not-run / accepted=false`.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`: remaining scope is actual direct no-proxy HTTP dispatch of normalized descriptors, 800 ms racing/reply cancellation/finalization, global HTTP-derived time mutation, temporary-DC help.getConfig lifecycle and dc_options application.

Coverage remains `source_entries_total=16202, responsibilities_total=120, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
