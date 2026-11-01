# Round 131 — selected proxy / MTProto carrier consumer

Date: 2026-11-01
Task: `7457eb1c-11a3-4626-8563-a3e495872536` round 100
Stage: A — implementation, zero executable validation

## Authority

- fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- desktop start: `3a704eb84b45f76dfa4c019e609493aad56f1f45`
- platform-core start: `c90e9150914f741b2ec3fbf81011da4005688d61`
- platform spec-first: `828fbc38ced6a78bdcc5f1cf4f224dc598a918fa`
- platform implementation: `2a4e9542f8b102eeaac8cc28f123e1a62648235d`
- platform final source-compliance: `f3075538f761ca0bf8a008c33ae7e3f25a1c66a3`
- desktop production: `5d816f224c21019f75725b56b6f8c759871f5295`

## Fixed-U source reread

The actual consumer chain was manually reread at fixed U: `core/application.cpp@7172206f5c1e540e19c90798e9a9e5721276cc53`, `main/main_account.cpp@2721c7ac12effe454d398610d26d638fadaa1f68`, `mtproto/mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`, `mtproto/session.cpp@40d7048c3057c7c5523b786048670524c8bebe4c`, `mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, `mtproto/connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`, `mtproto/connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a`, `mtproto/connection_resolving.cpp@d319e632655bbfe28956c8ced981a5cdf0299d1d`, `mtproto/details/mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9`, and `mtproto/mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef`.

Source facts: Settings owns durable mode/selected/list; live sessions consume only enabled selected + IPv6. MTProto uses selected host/port/secret while retaining protocol DC id. Custom resolution applies only to SOCKS5/MTProto hostnames, uses direct NoProxy Google/Mozilla DoH with 800ms stagger and TTL bounded to 10–300s, and resolved IP/expiry are transient.

## Existing-owner implementation

No new Telegram-specific settings/store/runtime root was created.

Platform production `2a4e9542f8b102eeaac8cc28f123e1a62648235d`:
- protocol `98e65d426242f9ff1ca0a3a2213dae1e41eca824`: fixed-U MTProto secret decoding while preserving WebProxy 0xee rejection;
- resolver `ef6b36c26257ee375df0a690e63dae814656e545`: transient cache, direct normalized DoH, TTL and successful-IP promotion;
- network `cf1247b3ec2486ce5e05f5a08ceb30b1e460e1db`: source-neutral NetworkProxyProjection, MTProto carrier and fail-closed unsupported enabled carriers;
- runtime `b605b13c110b545d0c703b7fae1cc27d3d10afab`: transient `telegram.applyNetworkProxy`, request/session invalidation and preserved reusable auth material.

Desktop production `5d816f224c21019f75725b56b6f8c759871f5295` mirrors those provider blobs. `TelegramClientLease@8df8e7286f161452336727d7e5750e45b67c413f` carries the typed transient projection; `AppHost@aa10fcfde5f47fc4b56223ff466cd9f128300b33` derives it from ProductSettingsOwner v2, publishes restored state before normal host use, and republishes mode/selected/IPv6 changes after persistence. Existing proxyEnabled special-config suppression remains.

Enabled SOCKS5/HTTP/Web selections deliberately fail closed and remain separate carrier responsibilities.

## Authored contracts — not executed

Contracts were written next to protocol/network/runtime/AppHost/TelegramClientLease for MTProto secret decoding, resolver request shape and transient promotion, missing-selected/unsupported-carrier fail-closed behavior, secret-free runtime results, route-change session fencing, source-neutral carrier DTOs, and post-persistence mutation publication. None were executed.

## Stage-A execution statement

No GitHub Actions/check/status/workflow query or trigger was performed. No build, typecheck, lint, schema validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deploy, install or runtime acceptance was executed.

## Disposition

`TDRP-R9-PROXY-MTPROTO-CONSUMER-001` is `implemented / validation_status=not-run / accepted=false`. Stage B executable validation remains open. The broader fixed-U migration remains incomplete.
