# Round 127 — MTProto proxy-enabled special-config suppression — source evidence

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 96
Stage: A / zero CI
Fixed U: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Live refs at start

- desktop: d603f9c06f961907c51e6e5b4e9473dd1ae4f302
- platform-core: 0b24613beea74301bf974dad4e10c56988096c40

## Fixed-U source read

- mtproto/config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194
- mtproto/config_loader.h@13a80b1b049f28866c3db110df5c662e8e74dede
- mtproto/mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087
- core/core_settings_proxy.h@1f2ec5c516e03ae49e80c15bb6083bffa7ec6d99
- core/core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26

SettingsProxy::isEnabled is exactly Enabled mode. Instance::Private projects the initial value into ConfigLoader construction and later connectionTypeChanges into ConfigLoader::setProxyEnabled. ConfigLoader::refreshSpecialLoader resets the loader while enabled; sendSpecialRequest terminates/refuses special endpoint work while enabled.

## Existing-owner resolution

Prior round113/114 evidence remains authoritative that the checkpoint has no canonical product Settings/application persistence owner. AppHost ui_state, telegram-storage and account-service are rejected as durable proxy truth. This round therefore implements only the typed boolean projection consumer inside the existing RuntimeClient and does not create a Telegram settings store.

## Production implementation

- platform spec-first: cb9b53d2574c012fe90e03df81b8b1336859ee2b
- platform implementation: 1c315c155eb0ba0c49871c08547bb7e67977aa87
- platform final source compliance: ef4edd0df2a9b74e3ac2cc3e088493e08ef75432
- platform runtime blob: 871ebe40ecbd284828b0c16ab64547eefeb3c3a3
- platform composition spec blob: eeacc5daa397a487a4f9c6ffcb4f18005fbdeda9
- desktop production mirror: c7fa1f1b373d34d07eedc1aa4b407bd032b0070d
- desktop runtime/spec blobs are byte-identical to platform.

telegram.refreshSpecialConfig requires boolean proxyEnabled. true returns before phone lookup, attempt seeding, request normalization, DirectHttpsRace creation, endpoint decode/application or temporary MTProto work, with suppressedByProxy=true and zero work counters. false preserves the existing acquisition/application path.

## Authored contracts / disposition

Three RuntimeClient contracts are authored and not run: direct suppression, strict boolean command parsing, and suppressed command response without phone/network work.

TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001 remains implemented / validation_status=not-run / accepted=false.
TDRP-R9-PROXY-MTPROTO-CONSUMER-001 remains mapped/pending / validation_status=not-run / accepted=false because the authoritative canonical Settings publisher and remaining selected-proxy/custom-resolution consumers are still open.

Coverage remains source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
