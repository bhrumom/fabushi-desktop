# Round 116 — DoH / special-config response normalization boundary (2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Stage A zero-CI constraints remained in force: no Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.

## Fixed-U source read

`mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9` and header `cb1979bbbf0454135c8668ed1e5013da85d1dedc` define `ParseDnsResponse`: empty/malformed/non-object responses, missing/non-array `Answer`, and wholly invalid answers normalize to empty. Individual non-object entries are skipped. When a type restriction is supplied, only numeric source-rounded matches survive. `data` must be a string; missing/non-number TTL becomes zero. Accepted answer order is preserved.

`DomainResolver::requestFinished` consumes the parsed entries and bounds effective TTL to a minimum of 10 seconds per entry and a maximum of 300 seconds overall. Cache mutation, expiry timestamps and queued callbacks remain outside the pure codec.

`special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` reuses the same DoH parser with type 16 for both Google and Mozilla special-config responses. `ConcatenateDnsTxtFields` orders via `QMultiMap(INT_MAX - QString::size())`: descending UTF-16 length, with QMultiMap's beginning-of-equal-range insert yielding later equal-length entries first, then concatenates and converts using `QString::toLatin1` semantics. `ParseFireStoreResponse` projects `fields.data.stringValue` and applies the same Latin-1 projection. The fixed-U constructor does not enqueue `RemoteConfig` or `Realtime` attempts, so those helper branches are not claimed in this responsibility.

## Existing-owner-first implementation

The broader carrier lifecycle still belongs to existing `telegram-network`, but its canonical crate currently has no JSON dependency and the desktop mirror has no separate network manifest. Existing `telegram-protocol` already owns deterministic codecs and already has `serde_json`, so the pure response codec is placed there without a dependency or lock delta. No new provider/runtime/settings/DC owner was created.

Platform spec-first commit: `64bdbc8521a33b0c16d1ca51680c7ea22e482f2d`, initial spec blob `f39d0e7ec80b46944e3c50fd7dec78193ac6549d`.

Platform implementation/authored-contract commit: `3d27d1aa1272bb15d39c1e2031872a1f2266c62e`.

- `native/telegram-protocol/src/response_normalization.rs@9371f73669b4e6b211608f043119ca4eca56e185`
- `native/telegram-protocol/src/lib.rs@27543d45f3825321e26c8a0b664e37c8488169e2`
- `native/telegram-protocol/tests/protocol_contract.rs@259f457eeaacce0ed09f1acf1f0fd821a0269733`

Desktop mirror commit `4070ca2be5db0d22bd54c27857ccf84748ce8763` has byte-identical module/lib/contract blobs.

Authored-only internal contracts cover structural DoH failures, typed filtering/order/TTL bounds, QMultiMap-equivalent TXT ordering including equal-length entries, UTF-16-to-Latin-1 replacement, and Firestore nested extraction. Public contracts cover the pure DNS normalizer and compact special-config response adapters. None were executed.

Platform source-level compliance commit `39db402654e3157d3e9b8694d9d91d89ea2c7f82` / spec blob `8379890d8ee17db235037c42cd9fde0cac00c014` records R1-R11 and AC-1..AC-5/AC-7..AC-8 as source-review passed, with AC-6 waiting for this unique desktop bookkeeping synchronization.

## Ledger disposition

`TDRP-R9-MTPROTO-RESPONSE-NORMALIZATION-001 = implemented / rewritten / validation_status=not-run / accepted=false`.

The existing `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`, but is narrowed to direct request construction/racing, HTTP Date clock mutation, temporary-DC help.getConfig lifecycle and dc-options application. Pure response parsing/normalization is no longer represented as pending carrier work.

Coverage: `source_entries_total=16202, responsibilities_total=119, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.


### Round 116 final source-level compliance

Platform-core final source-level compliance is `2585e75e6586480378f07d9daab39889e4111339` / spec blob `63f2b715e733cc1fe851b151fdf83355e3f2582b`. It references desktop initial bookkeeping commit `4f91cd629df720e79743885807c5da7db97042c8` and records R1-R11 plus AC-1 through AC-8 as passed at source/implementation/bookkeeping review level.

`TDRP-R9-MTPROTO-RESPONSE-NORMALIZATION-001` remains `implemented / rewritten / validation_status=not-run / accepted=false`. The canonical and desktop response module/lib/contract blobs remain byte-identical at `9371f73669b4e6b211608f043119ca4eca56e185`, `27543d45f3825321e26c8a0b664e37c8488169e2`, and `259f457eeaacce0ed09f1acf1f0fd821a0269733`.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false` because request construction/racing, HTTP Date mutation, temporary-DC help.getConfig scheduling/cancel/timeout/kill and dc-options application still require a shipping existing network/runtime lifecycle. Pure response parsing/normalization is no longer part of that blocker.

Coverage remains `source_entries_total=16202, responsibilities_total=119, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
