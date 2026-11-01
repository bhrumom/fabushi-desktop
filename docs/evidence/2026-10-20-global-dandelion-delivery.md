# Global Dandelion delivery baseline and acceptance ledger

Date: 2026-10-20 (Asia/Shanghai)

This ledger is a review record, not a release claim. A row is `implemented` only
when the Host has a real owner; it is `passed` only when exact-head build,
package, install, and live evidence exists for the named platform.

## Exact provenance

| Item | Exact fact |
|---|---|
| Source repository | `https://github.com/bhrumom/global-dandelion-rs` |
| Source branch | `main` |
| Source commit | `16e5eeb0ea939b62e228c03fa6c06762435781af` |
| Source parity baseline | `aa7f3720c42856f0e1d18742f78fba156227ee79`, blob `54bf6226eb80d1ef6c4f6087fb02579141ab134f` |
| Source CLI package | `dandelion` workspace version `0.1.0` |
| Desktop repository | `https://github.com/bhrumom/fabushi-desktop` |
| Desktop branch / commit | `codex/miniapp-e2e-completion` / `033196e27f7ee4631f89b4f531958482d186487f` |
| Desktop PR | [#123](https://github.com/bhrumom/fabushi-desktop/pull/123) |
| Current CI observation | PR 123 checks were green at review time; this is CI evidence only, not install/live evidence. |

The local source checkout is a shallow/grafted copy. The remote Git branch and
the exact commit above are therefore the provenance authority; the local path
is only a working copy.

## Source responsibility to Mini App/Host mapping

The web page uses only `fabushi.miniapp.tools.v1`. The Host owns the native
picker, opaque handles, CLI argv, credentials, state, sockets, mounts, hotspot
journal and updater. `implemented` below means a real desktop owner exists;
`blocked` means the requested evidence or platform owner is still absent.

| Source parity responsibility | Mini App entry / Host owner | Evidence status |
|---|---|---|
| time-metadata | `platform.time` → Dandelion `platform time` | implemented; live acceptance blocked |
| host-clock-mutation | no entry; intentionally removed by source policy | not applicable (safe removal) |
| disk-partitions, disk-space | `platform.inspect`, `platform.mounts` → Linux inventory | Linux owner; macOS/Windows unavailable |
| network-interfaces, connections, neighbour-discovery | `platform.inspect` → passive Linux inventory | Linux owner; live evidence blocked |
| structured-logging, progress-ui, live-speed-stats | `transfer.status` and bounded operation result | implemented in engine; UI/live evidence blocked |
| day-night-pacing, constant-rate, wire-budget | `transfer.send` → source config/pacing | implemented in engine; target configuration evidence blocked |
| surface-scan, source-scan | `scan.read` → `dandelion scan read` | implemented; live acceptance blocked |
| metadata-scan | `scan.metadata` → `dandelion scan metadata` | implemented; live acceptance blocked |
| hotspot | `hotspot.plan/apply/status/stop` → journaled Linux adapter | opt-in owner; hardware/rollback evidence blocked |
| release-version, updater | `update.check/stage` → signed verifier/staging | implemented; signed staging evidence blocked |
| rename-effect | `compat.linkExercise/linkRecover` → legacy-link adapter | implemented; live acceptance blocked |
| loopback-send, private-vps, lan, wan-workers, batch-send | `transfer.send` → configured authenticated transport | real CLI owner; authorized fixture/live evidence blocked |
| loopback-receive | `transfer.receive` → authenticated receiver | real CLI owner; authorized fixture/live evidence blocked |
| startup-drive-scan, external-drive-monitor | source `run --watch` and `platform.watch` | Linux owner; hotplug evidence blocked |
| chunk-framing | transfer engine and offline Wasm core | contract/source evidence present; end-to-end transfer blocked |
| state | `state.inspect/import`, `offline.state.read` → durable store | implemented; restart/import live evidence blocked |
| country-metadata | `compat.countryLookup` → bounded local dataset | implemented; dataset/live evidence blocked |
| random-public-targets, target-health, target-policy | configured transport policy behind `transfer.send` | source owner present; Mini App target-pool selection needs per-pool runtime configuration evidence |
| worker-explosion | no entry; bounded Supervisor replaces unsafe legacy default | not applicable (safe removal) |
| shutdown | `transfer.cancel` and Host account/window teardown | implemented; installed lifecycle evidence blocked |
| orchestration | Host operation IDs/status/cancel/timeout cleanup | implemented; installed lifecycle evidence blocked |

The full machine inventory remains the 37-row `docs/migration/parity.json` at
the source commit. The grouped rows above preserve every inventory ID and its
corresponding UI/Host surface; removed rows are explicitly identified rather
than silently advertised as supported.

## Platform disposition

| Platform | Build/contract | Installation/live operation | Disposition |
|---|---|---|---|
| Linux | PR 123 CI Host/renderer/native contract checks observed green | No exact-head installed Fabushi + authenticated fixture + package digest evidence attached in this run | implemented, acceptance blocked |
| macOS | Desktop app packaging paths exist; Dandelion source CLI is Linux-specific | No exact-head installed live evidence; Linux-only CLI capabilities are now hidden | offline/selection compatibility only; native parity blocked |
| Windows | Desktop app packaging paths exist; Dandelion source CLI is Linux-specific | No exact-head installed live evidence; Linux-only CLI capabilities are now hidden | offline/selection compatibility only; native parity blocked |
| Android | Generic WebMCP and inline HTML compatibility bridge | No device/emulator acceptance in this run; no Rust-backed Dandelion adapter | install/start compatibility path only; native parity not configured |
| iOS | Generic signed Mini App/WebMCP compatibility bridge | No device/simulator acceptance in this run; no Rust-backed Dandelion adapter | install/start compatibility path only; native parity not configured |

## Known delivery gaps

1. The source Rust platform owner is Linux-only. Advertising transfer, scan,
   state, platform, hotspot, update, diagnostics or compat on macOS/Windows
   would be false; the Host now fails closed by platform and reports only
   offline/selection there.
2. Android and iOS have bridge projection and lifecycle revocation, but not a
   shared-Rust/native Dandelion adapter, picker integration, secure credential
   owner, cancellation/restart implementation, or device evidence.
3. The existing signed `0.2.0` artifact in the separate historical checkout
   has a digest mismatch against its recorded provenance and is not a valid
   acceptance artifact for this source commit. It must not be installed or
   called a release of `16e5eeb`.
4. Production package approval, signer/catalog binding, installed Fabushi
   launch, authenticated send/receive fixture, hotspot hardware, and update
   staging remain external gates. No production credentials are used here.

## Required next evidence

Use GitHub Actions or the repository-authorized remote runtime only: exact-head
desktop package, digest/signature verification, clean install, Linux live
fixture transfer/receive, state import/restart/cancel/timeout, negative path
and stale-account tests, then separately record macOS/Windows package results.
Mobile device acceptance is intentionally not part of this round and must be
reported as “移动端安装运行兼容已实现，设备验收未执行” until a later run.
