# Global Dandelion Mini App full-capability contract

Status: active; implementation started 2026-10-19  
Owner: existing Plugins/MCP Mini App owner plus the existing native Host/Coordinator owners

Review note 2026-10-20: the exact source and platform disposition are recorded
in `docs/evidence/2026-10-20-global-dandelion-delivery.md`. The current source
CLI is Linux-specific. Desktop Host discovery therefore exposes the native CLI
capabilities only on Linux; macOS and Windows retain offline/selection
compatibility until a real native adapter exists. Android and iOS remain
bridge-compatible but not Dandelion-native-configured.

## Context

The approved Global Dandelion package currently exposes an offline Web/Wasm
workbench. The upstream native Rust application has a larger, bounded feature
set: transfer/receive, scan, state/resume, configuration and diagnostics,
platform inventory/watch/time, hotspot lifecycle, compatibility exercises,
verified update staging, self-test and status. A descriptor that merely lists
these capabilities without a native owner would be an unsafe false claim.

## Goal

Make the same signed Mini App usable on Desktop, iOS and Android through one
versioned tool contract. Every advertised capability must have an implementation
owned by the platform Host, explicit permission/approval, bounded arguments,
stable errors, cancellation where applicable, and exact-source Actions evidence.
The Web bundle remains unprivileged; it never receives a path, credential,
private key, shell, socket or arbitrary native method.

## Non-goals

- Giving Web/Wasm direct filesystem, socket, process, hotspot or key access.
- Replacing Fabushi Host/Coordinator ownership with a second runtime.
- Claiming Linux-only operations on iOS/Android when the OS API cannot provide
  an equivalent operation.
- Treating a capability probe, mock, or CLI presence as device acceptance.

## Capability matrix

| ID | Mini App methods | Native owner | Desktop | iOS | Android |
| --- | --- | --- | --- | --- | --- |
| offline | `offline.capabilities`, `offline.state.read` | Mini App Host | required | required | required |
| selection | `selection.pick`, `selection.release` | OS document picker | required | required | required |
| transfer | `transfer.send/status/cancel`, `transfer.receive/status/cancel` | Dandelion transport adapter | required | required | required |
| scan | `scan.metadata/read` | Dandelion scanner adapter | required | required | required |
| state | `state.inspect/import` | Dandelion state adapter | required | required | required |
| config | `config.defaults/check` | Dandelion config adapter | required | required | required |
| platform | `platform.inspect/mounts/time/watch` | platform adapter | required | capability-gated | capability-gated |
| hotspot | `hotspot.plan/apply/status/stop` | OS network owner | Linux/desktop where supported | OS API gated | OS API gated |
| update | `update.check/stage` | verified updater owner | required | required | required |
| diagnostics | `diagnostics.selfTest/doctor/status` | Dandelion diagnostics adapter | required | required | required |
| compat | `compat.packetInspect/countryLookup/linkExercise/linkRecover` | compatibility adapter | required | required | required |

`required` means the platform ships a real implementation before the
capability is advertised. `capability-gated` means the bridge may advertise it
only after probing an approved OS API and user permission. Unsupported means
the whole affected panel is hidden and calls return `CAPABILITY_UNAVAILABLE`.

## Security and lifecycle requirements

- The descriptor declares the capability IDs and network policy; the Host
  intersects them with platform policy, account grants and runtime probes.
- Mutating or network-affecting methods require a current user approval record.
- Receive binds only configured authenticated targets. WAN remains disabled by
  default; mTLS keys stay in Keychain/Keystore/Host storage.
- All file roots are selected by the native picker and revalidated before every
  operation. Limits are enforced before reading: 128 files, 1 GiB per
  selection, bounded packet/read budgets and bounded operation count.
- Long operations return an operation ID, support status and cancellation, and
  fence late results by account generation and app digest.
- Hotspot apply/stop requires an explicit confirmation and a journaled rollback
  record. No platform without a reversible OS API may advertise apply/stop.
- A platform-specific CLI or native adapter must be checked before capability
  discovery and again at invocation. A configured binary path alone is not
  evidence that the current OS can execute the source implementation.
- Native errors are sanitized stable codes; no credential, private key, host
  path or raw file content crosses the bridge.

## Implementation strategy

1. Extend `fabushi.miniapp.tools.v1` with the matrix above and strict argument
   validation.
2. Extend the existing Desktop Dandelion CLI owner to route every supported
   command through argv arrays, never a shell.
3. Add the same typed request/response surface to the existing mobile Host
   bridge. iOS and Android adapters call the shared Rust Dandelion core and use
   platform services only for picker, network, hotspot and secure storage.
4. Update the Global Dandelion descriptor only after all three adapters exist;
   the registry release must be re-reviewed and re-signed.
5. Add cross-platform contract tests plus device workflows. A platform build or
   probe without an end-to-end device operation is not acceptance evidence.

## Verification and Definition of Done

- Each row has a Host implementation, contract tests, negative permission tests,
  cancellation/late-result tests and bounded-resource tests.
- Desktop, iOS and Android Actions workflows build the exact source revision.
- Each platform runs: cold launch, capability discovery, picker, metadata scan,
  packet transfer/receive on authenticated fixtures, state resume, update
  verification, diagnostics, and safe teardown. Linux-only hotspot/platform
  operations are tested on the supported OS and are hidden elsewhere.
- The signed descriptor, package digest, review record and catalog entry match
  the exact artifact. No release claims full parity while a matrix row is
  unimplemented or lacks device evidence.
- Failure remains fail-closed: missing SDK, OS permission, signer, device or
  configured target is `not-configured`/blocked, never a pass.

## References

- `docs/specs/offline-first-installable-mini-app-platform.md`
- `docs/specs/desktop-miniapp-host-install-service.md`
- `source/electron-main/mcp/miniapp-tools.ts`
- Global Dandelion `docs/specs/fabushi-miniapp.md`
- Global Dandelion `docs/specs/global-dandelion-rust-architecture.md`
