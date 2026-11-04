# Round 129 — typed product proxy Settings state and migration — source evidence

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 98
Stage: A / zero CI
Fixed U: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Live refs at start

- desktop: 0b1b2b9a189e2db70ae2b1f6da36af3aaddfb3e3
- platform-core: 8064784fcbc22b54d5aeb47f186e9e89f7fbaef2

## Fixed-U reread

- core/core_settings_proxy.h@1f2ec5c516e03ae49e80c15bb6083bffa7ec6d99
- core/core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26
- core/core_settings.cpp@6ef88cd5f0b946b0b92512d1a94e42a0f5c1fabb
- boxes/connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2
- mtproto/mtproto_proxy_data.h@d8deebac2359936f72901ba0eec0cbfdfb186e90

SettingsProxy persists tryIPv6, useProxyForCalls, mode, selected, ordered list, IP-warning flag, rotation enabled/timeout and preferred indices. Proxy record type mapping is None/Socks5/Http/Mtproto/Web -> 0/1/2/3/4 with host/port/user/password. Empty legacy payload succeeds; later fields are optional; negative counts fail; None/unknown list entries are skipped; preferred indices are remapped around skipped entries; selected None becomes empty; malformed stream fails.

Core::Settings embeds _proxy.serialize() as one byte array in the application settings stream and restores via _proxy.setFromSerialized(proxy) only after the outer stream is valid. ProxiesBoxController holds a reference to Core::App().settings().proxy(); its save timer calls Local::writeSettings and saveDelayed also notifies proxy rotation state. List edits, mode changes, rotation changes, IPv6/call flags and IP-warning state all persist through that one owner.

## Spec-first

Platform-core commit: c90e9150914f741b2ec3fbf81011da4005688d61
- owner spec blob: 1a582fc7adc6682364c6857219dee79c6204d50f
- proxy settings spec blob: ee9bc5e4666ee48b9aaa74162823a89f418e4306

The spec requires one ProductSettingsOwner v2, a typed source-neutral proxy model, in-place migration of existing Fabushi v1 product-settings.json, exact list/preferred/timeout invariants, and mode-derived proxyEnabled publication only. It explicitly excludes runtime resolved IPs, expiry, sockets, WebView/checker/timer state and forbids a Telegram-specific store.

## Production implementation

Desktop production commit: b6f491d2364684111ebfc8ddbdae67ca3adaff6b
AppHost blob: 3078e50bbfa0583383b3449818a79ba8465b064a

The existing ProductSettingsOwner now persists version 2:
- mode: system | enabled | disabled
- selected: optional typed endpoint
- list: ordered typed endpoints
- tryIpv6
- useProxyForCalls
- checkIpWarningShown
- rotation.enabled / timeoutSeconds / preferredIndices

Typed endpoint persistence is type/host/port/user/password only. The mutation reducer implements fixed-U list replacement, insert/remove preferred-index remapping, timeout normalization, preferred normalization and promotion. setSelectedAndMode is one atomic owner mutation.

Existing v1 {version:1, network:{proxyEnabled:boolean}} migrates in place. true maps to Enabled; false maps to System because v1 did not distinguish System from Disabled and System is the fixed-U neutral default. All other fields receive fixed-U defaults. Malformed, missing-version and unsupported-version state fail closed.

proxyEnabled is derived exactly as mode == Enabled. settings.setNetworkProxyEnabled remains a compatibility projection and republishes on same-value retry; settings.mutateNetworkProxy republishes only for mode-affecting mutations. No selected endpoint/list is sent to a carrier in this round.

No raw TDesktop Core::Settings importer is fabricated. A future explicit import surface must supply a trustworthy outer settings/version boundary and feed the canonical owner.

## Authored contracts / disposition

Authored but not executed:
- fixed-U default state
- v1 -> v2 in-place migration
- typed list/selected/mode persistence and reopen
- preferred-index normalization plus insert/remove remapping
- whole-list replacement clears preferred order
- timeout <= 0 -> 10
- preferred promotion front/idempotent behavior
- malformed persistence fail-closed
- source-neutral host method naming

TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001: implemented / validation_status=not-run / accepted=false. Unified Settings UI projection and Stage-B executable evidence remain open.
TDRP-R9-PROXY-MTPROTO-CONSUMER-001: mapped/pending / validation_status=not-run / accepted=false. Selected-proxy/custom-resolution carrier consumers remain intentionally unwired.

Coverage remains source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
