# Round 130 — unified Settings network-proxy projection/edit surface

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 99
Stage: A / zero CI
Fixed U: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

Authority re-read: desktop checkpoint `695758b14d101e850651e1389861088e16c55f92`; shipping main `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core checkpoint `c90e9150914f741b2ec3fbf81011da4005688d61`. Canonical UI owner is `SettingsDesktopSurface -> GeneralSettingsPanel`; renderer/Main allowlist is `source/shared/rpc/main.ts`; MainData allowlist is `source/shared/rpc/coordinator-main.ts`; authoritative state owner remains AppHost `ProductSettingsOwner v2`.

Implemented chain: DesktopBridge.networkProxy.get -> getNetworkProxySettings -> settings.snapshot, and typed mutations -> mutateNetworkProxySettings -> settings.mutateNetworkProxy. The existing General Settings surface edits mode, selected/list endpoints, tryIpv6, useProxyForCalls, checkIpWarningShown and rotation enabled/timeout/preferred indices with canonical Sand components. Renderer local endpoint drafts are edit buffers only; authoritative mutation settlement replaces the projected snapshot. No Telegram-specific Settings screen/store/runtime was added, and no selected/list endpoint was connected to a carrier.

Authored but not run: preload Main RPC contract additions, source-neutral Main/MainData allowlist contract, DesktopBridge ownership contract, and preferred-index UI contract. No Actions/check/status query or trigger, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, package, deploy, install or runtime acceptance was performed.

Status: `TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` implemented / validation_status=not-run / accepted=false; `TDRP-R9-PROXY-MTPROTO-CONSUMER-001` remains mapped/pending because selected-proxy/custom-resolution carrier consumption is intentionally unwired.


## Exact production composition

The first source-only patch commit was not pushed directly because manual tree review showed that the checkpoint history had become sparse and lacked the complete shipping renderer/Electron dependency graph. The production integration commit is `a9cece6754710ab4f91dd9089e3d4eee9ffb4e57`, tree `b56bde12522d062968ce328d6550b11866c659a5`. It fuses the checkpoint migration history with current shipping main `9993ca8e107f7e019a8201e5ab74fa54e82159f2` and overlays all checkpoint/round-99 blobs, preserving migration work while restoring the complete current shipping tree. The resulting tree contains 9,618 entries / 8,456 blobs and includes `AGENTS.md`, `source/electron-main/entry.ts`, generated Main RPC runtime, `ProductionRenderer.tsx`, the canonical Settings components, and AppHost ProductSettingsOwner v2.

Key production blobs are bound in the parity ledger. This was established by Git tree/blob/commit re-read only; no generator, typecheck, build, test, visual/a11y runner, package, install or runtime acceptance was executed.
