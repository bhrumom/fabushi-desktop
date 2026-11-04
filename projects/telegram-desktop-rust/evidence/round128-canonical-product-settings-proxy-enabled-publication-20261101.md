# Round 128 — canonical product Settings owner and proxyEnabled publication — source evidence

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 97
Stage: A / zero CI
Fixed U: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Live refs at start

- desktop: 36bbe14d8f0fd7cb6008149e5fe9931f9e06891a
- platform-core: ef4edd0df2a9b74e3ac2cc3e088493e08ef75432

## Fixed-U mutation / publication chain reread

- core/core_settings_proxy.h@1f2ec5c516e03ae49e80c15bb6083bffa7ec6d99
- core/core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26
- core/application.cpp@7172206f5c1e540e19c90798e9a9e5721276cc53
- boxes/connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2

`SettingsProxy::setSettings` assigns the canonical mode only. `SettingsProxy::connectionTypeChangesNotify` fires the connection-type event only. `Application::setCurrentProxy` performs selected+mode mutation, global proxy projection refresh, proxyChanges publication, connectionTypeChanges publication and rotation notification. `connection_box` delegates mode/selected mutation to that Application method and persists the same application Settings object; its local list/view state is not a second durable owner.

## Existing-owner-first review

At the starting checkpoints there is still no typed canonical product Settings state owner:
- desktop AppHost `feature.persistence.*` stores arbitrary values in a table literally named `ui_state`;
- Telegram storage is provider-private authorization/transport persistence;
- fabushi-account-service owns account/session state and protected Telegram storage-key material;
- telegram-network/runtime are downstream runtime consumers;
- Agent/Codex config is not product Settings;
- the tracked frontend checkpoint contains no shipping Settings implementation.

Therefore the explicit fallback rule applies: establish one minimal source-neutral owner under the existing AppHost composition, not a Telegram-specific store.

## Spec-first / production implementation

Platform-core:
- spec-first owner decision: 206fa124fec1512bba4fca7f19602a1b691732b2
- final source-compliance: 8064784fcbc22b54d5aeb47f186e9e89f7fbaef2
- owner spec blob: dd97a47106dda5d5a82d5af691fd102dc7fa4034
- proxy settings state spec blob: 4099833bd7899e3e68ab0848bf7087658bab149f

Desktop:
- production implementation: b1b5dd027497c39a355e9e64e64de0b6c190fd66
- AppHost blob: 5b74723e12884ea218e1f9de634b88d4357b2099
- Mahayana Telegram adapter blob: e4410652fe500144d93ff227b10d149d2c0fc2c7

`ProductSettingsOwner` is source-neutral and composed by the existing shipping AppHost. It owns versioned `network.proxyEnabled`, persists at `feature-host/product-settings.json` via temporary-file replacement, and fails closed on malformed/unsupported persisted state. It does not use `ui_state` or Telegram storage.

`settings.setNetworkProxyEnabled` requires a boolean, persists authoritative state before publication, and republishes even when the stored value is unchanged so a failed downstream publication can be retried. The existing Telegram client lease forwards the authoritative boolean as `telegram.refreshSpecialConfig`; no second client/runtime/provider is constructed and RuntimeClient still stores no durable proxy settings.

## Authored contracts / disposition

Authored and not executed:
- ProductSettingsOwner default/persist/reopen contract;
- malformed persisted Settings fail-closed contract;
- source-neutral Settings host method contract;
- exact `telegram.refreshSpecialConfig` + boolean `proxyEnabled` request contract.

`TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` remains mapped/pending: the missing owner blocker is closed, but full proxy mode/list/selected/rotation/migration/UI semantics are not yet implemented.

`TDRP-R9-PROXY-MTPROTO-CONSUMER-001` remains mapped/pending: canonical `proxyEnabled` publication is now wired, while selected-proxy/custom-resolution consumers and Stage-B evidence remain open.

Coverage remains source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
