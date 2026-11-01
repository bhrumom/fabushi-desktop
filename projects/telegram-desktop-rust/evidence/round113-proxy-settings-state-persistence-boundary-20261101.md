# Round 113 — Proxy settings state/persistence boundary — source evidence

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 82
Stage: A / zero CI

## Live refs at start

- desktop: `757a1dc07e9c79a98f96703bb1cbd921c197b81d`
- platform-core before this round: `2682d35b46d5521d71ea82896557466c3e07a58f`
- fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Fixed-U source read

- `Telegram/SourceFiles/core/core_settings_proxy.h@1f2ec5c516e03ae49e80c15bb6083bffa7ec6d99`
- `Telegram/SourceFiles/core/core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26`
- `Telegram/SourceFiles/core/core_settings.h@7192f0db82ddc214489f4c8bd5c247b8911b8a07`
- `Telegram/SourceFiles/core/core_settings.cpp@6ef88cd5f0b946b0b92512d1a94e42a0f5c1fabb`
- `Telegram/SourceFiles/boxes/connection_box.h@02717ab7b646089aa055c441b86e18e35d156e09`
- `Telegram/SourceFiles/boxes/connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2`
- `Telegram/SourceFiles/core/application.cpp@7172206f5c1e540e19c90798e9a9e5721276cc53`
- `Telegram/SourceFiles/core/proxy_rotation_manager.h@c1fee276961c4f98e21ab2a8e495a1d687672804`
- `Telegram/SourceFiles/core/proxy_rotation_manager.cpp@72723d4db408be654dbf68e1ac40bba21c59d4e6`
- `Telegram/SourceFiles/storage/localstorage.h@ca847ab41eb2614b2d56ea248e932ab8ebead9ec`
- `Telegram/SourceFiles/storage/localstorage.cpp@d71385083c0e940580398612fb65e522afc768b1`
- `Telegram/SourceFiles/mtproto/mtproto_proxy_data.h@d8deebac2359936f72901ba0eec0cbfdfb186e90`

The unique durable source owner is `Core::SettingsProxy`, embedded once in `Core::Settings`. `connection_box` is an edit/view projection over that owner. `Application::setCurrentProxy` mutates the same selected+mode state and publishes projections. `ProxyRotationManager` owns transient reachability/timer state only and persists preference promotion/selection by writing back through the Settings owner.

Persistence is one chain: `SettingsProxy::serialize` -> `Core::Settings::serialize` -> encrypted `dbiApplicationSettings` / `Local::writeSettings`; restore returns through `SettingsProxy::setFromSerialized`.

## Current Fabushi owner inventory

- desktop AppHost: `mahayana-app-host/src/lib.rs@ed69b2b7aa0d87ebd34210e9a13d9c8a8b3e3087`; `feature.persistence.*` writes `feature-host/runtime/runtime.sqlite3` table `ui_state(key,value_json,updated_at_ms)`. This is generic UI/client feature persistence, not canonical product Settings truth.
- desktop/platform Telegram storage: `sqlite.rs@e6cc314ec749f8a258bfe0125a25beb3613eb2ca`; encrypted Telegram runtime/auth/transport private persistence, not global Settings.
- desktop account/session owner: `fabushi-account-service.ts@8e26fd5eab70d27ee553791ca30a8d6147358bf5`; account/session and protected Telegram storage-key lifecycle, not proxy preferences.
- telegram-network and rotation/checker runtime are downstream consumers/schedulers, not persisted settings owners.
- no matching first-party canonical Fabushi product Settings/application state owner exists in the two starting checkpoint trees.

## Disposition

Spec-first platform commit: `5724aee519adcd3990bbf98f88a97581602bc313` / initial spec blob `e31a0d4f4d9f9286f7af35562c7d9edac6687487`.

No production code was changed. `TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` is recorded as `mapped/pending`: the exact state responsibility is carrier-independent, but shipping it now would require either the wrong existing persistence owner or a forbidden parallel Telegram settings subsystem.

Authored contract vectors in the durable spec cover mode/type mapping, list/preferred-index mutation invariants, timeout normalization, append-compatible legacy restore, malformed/negative-count rejection, and single-owner persistence. They were not executed. No fake executable contract against a nonexistent canonical Settings API was added.

Coverage becomes `source_entries_total=16202, responsibilities_total=113, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

## Stage-A execution statement

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Final platform source-compliance readback

- platform-core final commit: `e987637031705b3c3ef4612c1fd9524d9ee6756c`
- final spec blob: `33c419f1ac5a833e4b41f7f16602c9fc076d12a0`
- desktop initial bookkeeping commit: `c6c4dc28262154d2af11c467cd0ae4b731c52916`

The final spec records R13-R14 as blocked, all other applicable source-review requirements and AC-1..AC-8 as passed, and executable validation as not-run. The responsibility remains mapped/pending with no production delta.
