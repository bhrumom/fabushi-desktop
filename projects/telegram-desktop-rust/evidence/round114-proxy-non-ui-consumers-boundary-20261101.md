# Round 114 — Proxy non-UI consumers boundary — source evidence

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 83
Stage: A / zero CI

## Live refs at start

- desktop: `9c92009815d2a82f6a3bd5f3b07888c4233e299f`
- platform-core: `1cff6f7176de9d879576e0630503626636240622`
- fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`

Both checkpoint comparisons were identical at source-review start.

## Fixed-U source read

- `core/external_control.cpp@43327bc03c528b65b41ce72add5e7cca584ea371`
- `calls/calls_call.cpp@a67723f5827e56be3f36033f3ad59bc37d0acf7b`
- `mtproto/mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`
- `mtproto/config_loader.h@13a80b1b049f28866c3db110df5c662e8e74dede`
- `mtproto/config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194`
- `core/core_settings_proxy.h@1f2ec5c516e03ae49e80c15bb6083bffa7ec6d99`
- `core/core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26`
- `mtproto/mtproto_proxy_data.h@d8deebac2359936f72901ba0eec0cbfdfb186e90`
- `mtproto/mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef`

## Exact consumer split

External automation is a command adapter over the one SettingsProxy: list/add/remove/use/next/toggle/undo all read or mutate that owner and persist through Local::writeSettings. It does not own a second list/selection.

The fixed-U call consumer is intentionally inert: `ProxyData::supportsCalls()` returns false unconditionally, so the tgcalls SOCKS5 descriptor branch is unreachable even when `useProxyForCalls` is true. Adding call-proxy behavior would be non-equivalent.

MTProto consumes proxy state without owning it: it projects isEnabled into ConfigLoader, suppresses special-config acquisition while proxy is enabled, and attaches transient resolved-IP/expiry observations to proxy values. Resolver completion/promoted good-IP writes selected back only through SettingsProxy under the source-defined enabled/change gates. `SerializeProxyData` excludes resolved IPs and expiry.

## Current Fabushi owner inventory

- desktop telegram-network `241cd8ec792070a8de8af9ec6ca5b317e34376d1`: no proxy/config-loader/custom-resolution symbols; sockets/retry owner only.
- desktop telegram-runtime `10031c5261bb0810ad2df2fca3de2e2fe4f65301`: no proxy/config-loader/custom-resolution symbols.
- tracked desktop production tree has no shipping call/proxy consumer file beyond protocol WebProxy primitives.
- prior owner rejects remain: AppHost ui_state is client/UI persistence; telegram-storage is provider-private runtime/auth/transport persistence; account-service owns identity/session, not product Settings.
- no canonical Fabushi product Settings/application state owner exists at this checkpoint.

## Disposition

Platform spec-first commit: `2e155bb7be49b9e745f94cdc8b40fd831e052af7`
Initial spec blob: `b4220a6ac80b210cc8a969925ea0b180405486d7`

Three unique responsibilities are added:
- `TDRP-R9-PROXY-EXTERNAL-MUTATION-CONSUMER-001`
- `TDRP-R9-PROXY-CALL-CONSUMER-001`
- `TDRP-R9-PROXY-MTPROTO-CONSUMER-001`

All remain `mapped/pending`, `validation_status=not-run`, `accepted=false`, with no production delta. Coverage becomes `source_entries_total=16202, responsibilities_total=116, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Authored contracts are in the platform spec and ledger metadata only. No executable test is run or fabricated against absent shipping owners.

## Stage-A execution statement

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Final platform source-compliance readback

- platform-core final commit: `de2460e31dfa5d47df0a1993580ef11a1cfa1378`
- final spec blob: `438b64863eab0cc6fc746954cff8513de2fdd247`
- desktop initial bookkeeping commit: `a23452a00e9c4d395f3f97bf74ec27fd82747fb2`
- initial ledger/P0/source-closure/evidence blobs: `ca7fb45076804a0905d48ad1e3605e922310f33a` / `85825d4c00857ad76abc36998e087c99afacd98f` / `c3a83028db52a1ffe25206b72512699000f2e57c` / `a7e318f8aa33e81fb953fb3d143cd4db43ef9588`

The final platform spec records R1-R8 source-review passed, R9 production implementation blocked, R10 executable validation not-run, and AC-1 through AC-8 source/bookkeeping satisfied. All three new responsibilities remain mapped/pending with no production delta.
