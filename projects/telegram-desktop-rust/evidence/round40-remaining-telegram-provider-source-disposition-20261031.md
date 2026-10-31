# Round 40 — remaining Telegram provider source disposition

Date: 2026-10-31
Checkpoint base: `1caf8ed26becbdad129b41ed70cd353efd58c8d6`
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

No Stage A CI/Actions query or trigger and no executable validation was performed.

## Reviewed residual provider files

1. `providers/telegram-protocol/src/request.rs@d59aae57f411c0f0c346c729a1c9bb528af144fe`
   - Implements local monotonic RequestId sequencing and JSON `@type/@extra` envelope projection.
   - No exact upstream byte-copy/generated/transcribed relationship was established by the reviewed source.
   - Disposition: Fabushi-owned helper; no new source-license claim.

2. `providers/telegram-storage/src/lib.rs@13378d686b9ea45ac57f165a95052ef559545fc5`
   - Defines the encrypted-storage module boundary and explicitly assigns master-key lifecycle to Android Keystore, Apple Keychain, desktop credential vaults, or the web adapter.
   - No exact Telegram/TDLib copied/generated/transcribed relationship was established.
   - Disposition: Fabushi-owned storage boundary; no new source-license claim.

3. `providers/telegram-network/src/bin/telegram-auth-key-probe.rs@2854784c2db2ebdfc005ceb85c7bad9b4a7682d5`
   - This is a diagnostic/tooling binary, not a production state owner.
   - Literal request constructor `0xb921bd04` is exactly `get_future_salts#b921bd04 num:int = FutureSalts`.
   - Expected result constructor `0xae500895` is exactly `future_salts#ae500895 ... = FutureSalts`.
   - Both declarations are in `tdlib/td@a17f87c4cff7b90b278d12b91ba0614383aaee82/td/generate/scheme/mtproto_api.tl@b2508b478b9e5dd51af1dde33adcbf87949b9a5c`, under the exact TDLib Boost-1.0 license already retained downstream.
   - Transformation: two schema IDs embedded in Fabushi-owned manual auth-key/ping diagnostics; no whole-file copy claim.

## Disposition rule

Telegram naming, common domain models, SQLite use, HMAC/SHA-256, or local JSON conventions are not treated as copying evidence without an exact source-object or behavior mapping.

Rounds 36–39 remain authoritative for the other Telegram file-level mappings and the canonical provider-activation blocker. Boost/libiconv complete member manifests and the no-spin original-host blockers remain open. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.
