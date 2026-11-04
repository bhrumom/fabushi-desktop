# Round 134 — SOCKS5 / HTTP selectable proxy carriers

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 103
Stage: A / zero CI
Fixed U: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Live refs at start

- desktop: b8820f5ff82193d339cf08ded937379e8527d24e
- platform-core: 9a8e1c552e3ca9a611e5e0e7b845b845b22854ac

## Fixed-U source reread

- Telegram/SourceFiles/mtproto/connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381
- Telegram/SourceFiles/mtproto/mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef
- Telegram/SourceFiles/mtproto/connection_resolving.cpp@d319e632655bbfe28956c8ced981a5cdf0299d1d
- Telegram/SourceFiles/mtproto/details/mtproto_abstract_socket.cpp@653033365b543e554104c99a963ee194ddb830d9
- Telegram/SourceFiles/core/core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26

For SOCKS5/HTTP, fixed-U connection_tcp keeps the Telegram DC address/port as the logical destination and hands the selected proxy to the same native socket path. ToNetworkProxy maps Socks5/Http to their native proxy carriers with host/port/user/password. Custom proxy-host resolution is source-defined for SOCKS5 (and MTProto) but not HTTP. After the proxy connection is established, the existing endpoint secret/TLS/MTProto path remains authoritative.

## Existing-owner-first / spec-first implementation

Platform-core spec/implementation head:
- commit: 0a8d8f9b1830e989793da934210f699ea5d39d1f
- network: b2ce7b4f018494cc806520f0fd241f5264ba2bbd
- spec: 17e42e6671a2bd553bf6a04610348815124b7e26

Desktop production mirror:
- commit: e2fa2dc75f52763e126b5f3f7db8492ae26b92b5
- network: b2ce7b4f018494cc806520f0fd241f5264ba2bbd
- spec: 17e42e6671a2bd553bf6a04610348815124b7e26

No new runtime/provider/settings/session owner exists. ProductSettingsOwner v2 remains durable authority; TelegramClientLease/RuntimeClient carry only the transient selected-proxy projection; RuntimeClient's existing projection-change fencing invalidates live transport and preserves reusable auth material; telegram-network remains the one socket lifecycle owner.

telegram-network now:
- establishes SOCKS5 TCP to selected proxy host/port, negotiates RFC1928 CONNECT to the chosen Telegram DC target, and supports no-auth or RFC1929 username/password;
- reuses the existing transient direct-NoProxy proxy-host resolver and good-IP promotion for SOCKS5 hostnames;
- establishes HTTP TCP to selected proxy host/port, negotiates CONNECT to the chosen Telegram DC target, and sends optional Basic proxy authorization;
- uses the ordinary socket resolver for HTTP proxy hostnames;
- after either tunnel succeeds, reuses the existing endpoint secret, protocol DC identity, obfuscation/TLS camouflage, framing, auth-key and session lifecycle;
- propagates handshake/auth/resolution/cancellation/deadline failures without direct-transport fallback.

## Authored contracts — not executed

The existing telegram-network source contracts now cover SOCKS5 CONNECT wire shape and HTTP CONNECT + Basic authorization request shape. Existing RuntimeClient contracts cover secret-free transient projection and projection-change session fencing. No contract or other executable validation was run.

## Ledger disposition

`TDRP-R9-PROXY-MTPROTO-CONSUMER-001` remains implemented with `validation_status=not-run` and `accepted=false`. The prior SOCKS5/HTTP selectable-carrier blocker is closed; Stage B executable validation remains. The Web carrier row no longer lists SOCKS5/HTTP as an open adjacent blocker. Global source counters are unchanged because this round reread already-mapped fixed-U blobs rather than fabricating new source-entry completion.

## Stage-A execution statement

No GitHub Actions/check/status/workflow query or trigger was performed. No build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deploy, install or runtime acceptance was executed.


Round103 ledger hygiene: historical missing-canonical-Settings blockers were removed from `TDRP-R9-MTPROTO-WEB-PROXY-CANONICAL-ADDRESS-SETTINGS-001` and `TDRP-R9-PROXY-EXTERNAL-MUTATION-CONSUMER-001` because ProductSettingsOwner v2 is now authoritative. Neither row was advanced to implemented: exact reviewed IDNA/QUrl::toAce support and a verified shipping source-neutral external local-automation command surface remain open respectively. No executable validation was run.
