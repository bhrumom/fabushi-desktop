# Cloud Workers status consumer fidelity — 2026-10-30

The Host `cloudWorkers.status` producer already publishes read-only
persistence facts from the canonical Cloud Worker owner: whether state was
persisted and the bounded counts of leases, warm images, and migration
receipts. The Coordinator status projection previously declared only owner,
status, provider readiness, and production-mutation fields; serde therefore
discarded the additional facts at the Coordinator consumer boundary.

The typed Coordinator projection now preserves those four additive fields with
defaults for older Gateway responses. The existing status enum and the explicit
`providerReady=false` / `productionMutation=false` boundary remain unchanged.
The event encode/decode path consequently carries the same owner snapshot
facts without adding a second store or provider mutation path. The Coordinator
SSE family registry also now maps `cloud-workers.status` to its own consumer
family instead of silently classifying a future status event as generic
`runtime`.

Evidence is limited to source inspection and `git diff --check`; local OCRM
tests/builds, Actions, provider reachability, and live UI/CLI acceptance remain
pending. The Host-side live event publisher and provider mutation path remain
open.

The read projection is now produced by `CloudWorkerProvisioningExtension` via
`status_snapshot`; `UnifiedGatewayApi` serializes that owner result instead of
repeating lease/image/receipt semantics in the Gateway adapter. A focused Host
contract covers the disabled, provider-not-ready projection and its wire field
names. `statePersisted` now reflects the canonical regular state-file
publication, rather than merely successful access to the in-memory owner. This
is covered in both directions by the focused contract (absent before
`persist()`, present after it), including the serialized `statePersisted` wire
field. This is owner-boundary progress only: it does not add mutation
commands, provider readiness, or a live event publisher.

## P1-F08 typed consumer adapter — 2026-10-31

The next P1-F08 seam now has a narrow Rust adapter at
`source/host/src/runner/host_external_machine_dependencies.rs`:
`CloudWorkerScopedExternalMachineExecutor`. Its producer is still an
ephemeral `CloudWorkerLeasedCapability` supplied by the canonical
`CloudWorkerProvisioningExtension`; its owner remains the Cloud Worker lease
owner; its consumer delegate is exactly one existing `ExternalMachineExecutor`.
Before shell/read/status/history/cancel delegation, the adapter checks the
expected account/device scope and re-reads the current owner capability, so a
disabled owner, stale lease/generation/epoch/runtime, capability unavailable
because of outcome-unknown, or scope mismatch fails closed. Shell/read calls
also retain bounded tool-call replay keys and reject reuse before delegation.

This is adapter-boundary wiring only. No provider producer, physical worker,
device enrollment, Computer grant, sensitive-result writer, Runner loop, live
composition registration, or cloud mutation path was added. Therefore
`providerReady=false`, `productionMutation=false`, exact-head Actions,
provider/device/package evidence, and OCRM acceptance remain pending. This
round used source inspection and `git diff --check` only; no local build/test,
OCRM command, Actions, push, or PR update was run.

## P1-F08 Computer consumer adapter — 2026-10-31

The existing Computer boundary now has a narrow typed adapter at
`source/host/src/runner/host_computer_tool_dependencies.rs`:
`CloudWorkerScopedComputerToolExecutor`. It revalidates the ephemeral
`CloudWorkerLeasedCapability` against the canonical owner and expected scope,
claims the existing tool-call identity as a bounded replay key, then delegates
to exactly one existing `ComputerToolExecutor`.

The adapter does not mint a Computer grant, enroll a device, write sensitive
results, own screenshots, or execute Computer actions itself. The provider,
physical worker, device enrollment, production Runner composition, package
reachability, verification and acceptance remain pending; the parity row marks
only this adapter boundary as wired.

## P1-F06 platform descriptor boundary — 2026-10-31

The Rust platform seam at
`source/host/src/extensions/cloud_workers/platform_bridge.rs` is a descriptor
contract, not a platform executor. `CloudWorkerPlatformBridge` validates the
requested platform, desktop session and bounded launcher identity, rejecting
unsupported Linux/VNC and malformed launcher identities before delegation.

It does not own or connect a launcher, Docker, VNC, CDP, Electron lifecycle,
physical worker process, or platform credentials. No existing platform adapter,
session owner, provider producer, package path or live platform receipt is
wired to this descriptor. The parity row therefore records
`implementation=implemented`, `wiring=not-wired`, `verification=not-run` and
`acceptance=pending`; platform/device/package acceptance remains open.

The descriptor is now carried by `CloudWorkerBootstrapRequest` and revalidated
before bootstrap request admission. This is a type-level bootstrap contract
connection only; it still does not launch a platform, resolve credentials or
provide a production producer/consumer path.

## Provider catalog owner connection — 2026-10-31

`CloudWorkerProvisioningOwner` now owns the ephemeral
`CloudWorkerCatalogCache` and exposes `resolve_catalog`. Catalog results remain
bounded, normalized and keyed by the exact opaque provider-binary identity;
failed refreshes cannot leave stale capability data available. The loader is an
explicit typed boundary and still does not discover or launch a provider,
authorize a worker, or set `providerReady=true`. The parity row therefore marks
the Rust catalog contract implemented while producer wiring and acceptance stay
pending.

The Host extension now exposes `resolve_provider_catalog` as the sole typed
façade to that owner cache. A poisoned owner maps to an explicit unavailable
catalog error; no provider discovery, authorization or launch fallback is
introduced.

## Doctor read-only owner projection — 2026-10-31

`CloudWorkerProvisioningExtension::inspect_doctor` now validates and accepts
the typed `Inspect` action from the canonical extension boundary. Repair,
secret-reference migration and lease/warm-image reconciliation actions return
an explicit `mutation-not-wired` error even when acknowledged, so the
projection cannot accidentally become a second recovery owner. Plugin/UI/CLI
registration, provider mutation and recovery receipts remain pending. The
read-only result is now reachable through the `cloudWorkers.doctor` Host
Gateway registry/route; this does not make the broader doctor responsibility
production-wired because mutating and packaged consumer paths remain absent.

## Sandbox capability fencing — 2026-10-31

`CloudWorkerSandboxCapability` now must be derived from an enrolled
`CloudWorkerLeasedCapability` and carries lease ID, scope, owner epoch,
generation and runtime identity. `validate_against` rejects a stale or
deserialized capability before any future sandbox consumer can use it. The
workspace/path/argv contract remains side-effect-free and no sandbox executor
or remote provider is connected; the parity row therefore remains
`wiring=not-wired`, `verification=not-run`, `acceptance=pending`.

The canonical extension now exposes
`validate_current_sandbox_capability`, which re-reads the owner lease and
rejects disabled, stale, invalid or outcome-unknown capabilities before a
future consumer can delegate. This is still an owner-validation seam, not a
remote sandbox execution path.

The owner boundary now also validates each `CloudWorkerSandboxRequest` against
the current capability, including workspace identity and the requested
filesystem/remote-shell grant, before any future delegate call.

## Verified artifact to bootstrap binding — 2026-10-31

`CloudWorkerBootstrapRequest` now carries `CloudWorkerVerifiedArtifact` rather
than a bare digest. The artifact proof is serializable for typed boundaries
and revalidates its digest shape after deserialization before bootstrap
admission. This preserves the distinction between artifact admission and
bootstrap execution: no download, extraction, process launch or provider
readiness is introduced by this binding.

The bootstrap request now also carries lease ID, owner epoch and generation.
`CloudWorkerProvisioningExtension::validate_current_bootstrap_request` checks
those fields, scope and runtime identity against the canonical owner before a
future producer can act, preserving stale/outcome-unknown fencing without
adding enrollment or process side effects.

## Warm-image owner persistence façade — 2026-10-31

The Host extension now exposes typed `register_warm_image` and
`apply_warm_image` methods. Both route through the single provisioning owner
and its atomic persistence adapter; generation fencing and rollback/outcome
unknown handling remain centralized in that owner. No provider snapshot,
credential scrub, image process or recovery consumer is connected by these
methods.
