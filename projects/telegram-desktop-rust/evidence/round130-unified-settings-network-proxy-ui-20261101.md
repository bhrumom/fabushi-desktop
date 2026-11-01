# Round 130 — unified Settings network-proxy projection/edit surface

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 99
Stage: A / zero CI
Fixed U: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

Authority re-read: desktop checkpoint `695758b14d101e850651e1389861088e16c55f92`; shipping main `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core checkpoint `c90e9150914f741b2ec3fbf81011da4005688d61`. Canonical UI owner is `SettingsDesktopSurface -> GeneralSettingsPanel`; renderer/Main allowlist is `source/shared/rpc/main.ts`; MainData allowlist is `source/shared/rpc/coordinator-main.ts`; authoritative state owner remains AppHost `ProductSettingsOwner v2`.

Implemented chain: DesktopBridge.networkProxy.get -> getNetworkProxySettings -> settings.snapshot, and typed mutations -> mutateNetworkProxySettings -> settings.mutateNetworkProxy. The existing General Settings surface edits mode, selected/list endpoints, tryIpv6, useProxyForCalls, checkIpWarningShown and rotation enabled/timeout/preferred indices with canonical Sand components. Renderer local endpoint drafts are edit buffers only; authoritative mutation settlement replaces the projected snapshot. No Telegram-specific Settings screen/store/runtime was added, and no selected/list endpoint was connected to a carrier.

Authored but not run: preload Main RPC contract additions, source-neutral Main/MainData allowlist contract, DesktopBridge ownership contract, and preferred-index UI contract. No Actions/check/status query or trigger, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, package, deploy, install or runtime acceptance was performed.

Status: `TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` implemented / validation_status=not-run / accepted=false; `TDRP-R9-PROXY-MTPROTO-CONSUMER-001` remains mapped/pending because selected-proxy/custom-resolution carrier consumption is intentionally unwired.
