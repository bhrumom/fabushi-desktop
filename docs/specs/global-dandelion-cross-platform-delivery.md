# Global Dandelion cross-platform delivery and evidence plan

Status: active; discovery baseline 2026-10-19; implementation must remain
fail-closed until each platform has a real Host owner and exact-head evidence.

This document is the durable cross-repository plan for Global Dandelion. The
existing offline package specification and the native Dandelion architecture
spec remain authoritative for package format, protocol, security and Rust
ownership. This document only binds the cross-platform delivery work to stable
requirement and acceptance IDs.

## Current baseline

| Repository | Observed revision | Current fact |
| --- | --- | --- |
| Desktop | `6fc7291143ae2bc988105c9361cf5d44db9263c3` | Typed `fabushi.miniapp.tools.v1` router and CLI-backed Host owner exist. |
| Android | `266f90f394e5a12219bc5d9c53cca5aec243bca3` | WebView bridge and marketplace projection exist; Global Dandelion native adapter is not evidenced. |
| iOS | `a99a181f58aa544b2d3fcad6eece9dc3e96c316a` | Generic signed Mini App/WebMCP surface exists; Global Dandelion native adapter is not evidenced. |
| Backend | `56de6f0d27705f95e1b98ec99515074eb8f3f728` plus uncommitted release worktree changes | Package delivery/review code is being repaired; no current public approval is inferred. |
| Global Dandelion | remote `main` `16e5eeb0ea939b62e228c03fa6c06762435781af` (parity source `aa7f3720c42856f0e1d18742f78fba156227ee79`) | Native Rust CLI exposes the 37-row parity inventory; the local snapshot is not the provenance source. The current spec calls for corrective release `0.2.1`. |

The baseline is evidence for planning only. It is not release or device
acceptance evidence.

## Stable requirements

- GD-X01: one versioned tool contract is the only UI-to-Host boundary.
- GD-X02: every call validates app identity, package digest, account
  generation, declared capability, arguments, approval and platform state.
- GD-X03: native code owns file handles, credentials, sockets, processes,
  mounts and update keys; no such value crosses the WebView bridge.
- GD-X04: long operations have operation ID, bounded progress, cancellation,
  timeout, cleanup and late-result fencing.
- GD-X05: unsupported or unconfigured platform behavior returns a stable
  `CAPABILITY_UNAVAILABLE`/`NOT_CONFIGURED` result and is hidden in the UI.
- GD-X06: package signature, review, catalog approval, installation and live
  device behavior are independent evidence layers.
- GD-X07: all executable verification follows the owning repository rule:
  GitHub Actions or an explicitly authorized remote runtime only.

## Capability matrix and current disposition

| Acceptance ID | Contract methods | Desktop | Android | iOS | Evidence required |
| --- | --- | --- | --- | --- | --- |
| GD-XA-01 | `offline.capabilities`, `offline.state.read` | implemented in Host | bridge projection only | bridge projection only | exact-head Host call and offline launch |
| GD-XA-02 | `selection.pick`, `selection.release` | implemented with picker/root checks | WebView bridge only | generic picker surface only | real picker and scope revalidation |
| GD-XA-03 | `transfer.send/receive/status/cancel` | CLI owner, config-gated | not configured | not configured | authenticated fixture transfer, cancel/resume |
| GD-XA-04 | `scan.metadata/read` | CLI owner, root-gated | not configured | not configured | bounded scan and negative path tests |
| GD-XA-05 | `state.inspect/import` | CLI owner; import is native-picker gated | not configured | not configured | corruption, import idempotence, restart |
| GD-XA-06 | `config.defaults/check` | CLI owner | not configured | not configured | schema and effective-config evidence |
| GD-XA-07 | `platform.inspect/mounts/time/watch` | CLI owner | not configured | not configured | supported OS API or hidden panel |
| GD-XA-08 | `hotspot.plan/apply/status/stop` | explicit opt-in; Host-configured journal/settings | not configured | not configured | real reversible OS API and rollback |
| GD-XA-09 | `update.check/stage` | updater owner, opt-in | not configured | not configured | signed staging and rollback |
| GD-XA-10 | `diagnostics.selfTest/doctor/status` | CLI owner | not configured | not configured | exact binary/version and failure mapping |
| GD-XA-11 | `compat.packetInspect/countryLookup/linkExercise/linkRecover` | CLI owner; compat-root bounded | not configured | not configured | complete adapter or capability hidden |
| GD-XA-12 | package/review/install | backend + Desktop Host path | catalog/install path only | catalog/install path only | signature, review, immutable digest, clean install |

“Implemented” above means source ownership only. A row becomes passed only
after the evidence listed in its final column is attached to an exact commit.

## Delivery order

1. Keep this matrix and the existing package specs synchronized with the exact
   contract and platform status.
2. Finish Desktop contract tests and exact-head Actions evidence.
3. Add Android and iOS native adapters around their existing Host/Coordinator
   owners. Until then, their bridges may discover only genuinely granted tools
   and must not advertise the desktop CLI capabilities.
4. Update the descriptor only after platform adapters exist; re-run package
   signing and review rather than widening an existing signed artifact.
5. Run platform install and device workflows, then perform the compliance review
   against GD-X01..GD-X07 and GD-XA-01..GD-XA-12.

## Explicit blockers

- Android/iOS native Rust-core integration and device workflows are not present
  in the observed revisions.
- The original local Global Dandelion snapshot lacks Git provenance; remote main
  provenance is now recorded above and must be used for release work.
- Hardware hotspot, removable-device hotplug, production signer, catalog
  approval, physical-device installation and store distribution require external
  accounts/devices or protected Actions configuration.

These are blocked, not passed, until their exact-source evidence exists.

## Current delivery evidence (2026-10-19)

The following entries are the current review heads and must be refreshed before
any promotion decision. A green contract or source check does not promote a
platform capability whose runtime evidence is still missing.

| Repository / PR | Exact review head | Current evidence | Disposition |
| --- | --- | --- | --- |
| Desktop #123 | `1e664cd315ccf09775d45842467e7775e8fabbd7` | Focused Electron E2E, renderer, native-intents, contracts, publisher, Rust Host, and macOS/Windows/Linux pressure jobs passed in Actions run `39273775698` and related checks. | Contract and desktop evidence passed; install/live release evidence remains separately required. |
| Android #4 | `19a692dcabf1f75c1a6e7ad2de89ee7f34a86371` | Prior contract checks passed in runs `39274346480` and `39274354667`; the lifecycle-revocation follow-up run `39281899784` is queued. | Contract projection and lifecycle source change are present; the follow-up Actions result, native Global Dandelion adapter, real permissions, background/recovery and device transfer evidence remain incomplete. |
| iOS #10 | `3431e2a7548517b1f3365c631f874e424b529aa0` | Architecture parity `39282312583` and protected session `39282312586` are queued; no terminal result exists yet. | Not passed. The queued jobs must finish against this exact head before promotion. |
| Backend #117 | `969f6f7749f808235a4675b90740ba3a8f08f7ae` | Candidate contract and pure review gates passed in runs `39274252506`, `39274457773`, and `39274457838`. | Registry/review contract evidence passed; external signer, catalog promotion, clean install and upgrade/revoke evidence remain separately required. |

The evidence above is deliberately split by layer: source ownership,
contract tests, package approval, installation, permission grant and live
device behavior are not interchangeable. Any row in the capability matrix
without its final-column evidence remains `blocked` or `not passed`.
