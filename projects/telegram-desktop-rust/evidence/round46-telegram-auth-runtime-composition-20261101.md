# Round 46 — Telegram auth runtime Cargo composition fusion

Date: 2026-11-01
Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Desktop implementation base: `221a661e17f2ad417e349df6604815d3fbfa946f`
Platform-core canonical checkpoint: `13f5458aadf19a87c52bfa904f9942d81c88979a`

## Composition authority

- Workspace: `source/mahayana/mahayana-rs/Cargo.toml@144c67c76daaf2229e817f9830e678a3e6d4cf3a`.
- Consumer: `mahayana-telegram/Cargo.toml@f43109356f1f486a82a2d0120f6e78808c431c40` -> `../providers/telegram-runtime`.
- Runtime manifest: `providers/telegram-runtime/Cargo.toml@0fc8ee357ef5c00074e5fdcd1d7594860e289bd1` -> sibling core/network/protocol/storage.
- Existing `TelegramClientLease` remains the sole process-lifetime client owner.

## Fused canonical sources

- protocol API `751219099926a809d455d31b005c48a841d879f2`
- protocol exports `34bcd749f758cfd3f562b6e701ee10eb51d5d0c2`
- SRP persistence contract `06ff7c4ed54f445c9df8bf96deb1deb7defe4939`
- network resume primitive `24e236df503e273490e44645251d757e30712ea3`
- encrypted storage schema-v4 fusion `e6cc314ec749f8a258bfe0125a25beb3613eb2ca`
- runtime identity/lifecycle fusion `10031c5261bb0810ad2df2fca3de2e2fe4f65301`
- runtime public projection `0c79707ba1e0932de15fad3c57ad00fcb9682aff`

The fused runtime preserves canonical `authorization_snapshot/self_user_id`, resumable MTProto transport and Ready fail-closed semantics while implementing the seven lifecycle commands on the same client/session. Storage schema v4 contains both historical encrypted metadata shapes and authored migration contracts cover both v3 layouts.

## Desktop synchronization

- production composition commit: `43d9d398801f2885da982d106c17a74efb1f9472`
- authored integration contracts: `9110c9a5bce67f917a9bcba4af72de31199a290a`
- v3 migration contract correction: `221a661e17f2ad417e349df6604815d3fbfa946f`
- cross-crate composition contract: `mahayana-telegram/tests/telegram_runtime_composition.rs@a0453c88d6d8874e7bd0b32f8a9e357eab6799a4`
- storage contracts: `providers/telegram-storage/tests/encrypted_store.rs@ecde6882fc7265eee0f1c2714a8eb4fa1543d63f`
- runtime contracts: `providers/telegram-runtime/tests/runtime_ffi.rs@97eb341731e6122f615aeac2aba5eaa12d3a9fbd`

## Validation state

Stage A remained zero-CI. No Actions/status read or trigger, build, typecheck, lint, validator, generator, test, packaging, deploy, install or runtime acceptance was performed. All executable evidence fields remain empty/not-run; no verified/accepted claim is made.
