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
pending. The provider mutation path remains open; the Host event publication
seam is documented below and still lacks mutation-driven state producers.

## Coordinator RPC and packaged Settings consumer — 2026-10-31

The shared Coordinator RPC contract now registers `cloudWorkers.status` and
`cloudWorkers.doctor` with typed response parsers. The doctor request preserves
the Host wire shape (`scope`, `action`, `acknowledged_mutation`, and
`replay_key`) and is rejected in the production client when malformed. The
Host gateway allow-list also includes the doctor route.

The production Settings overlay adds a read-only Cloud Workers section. It
fetches the status projection through `ProductionCoordinatorClient`, subscribes
to the `cloud-workers.status` family, and ignores event payloads that fail the
shared projection parser. It displays owner status, provider/mutation
readiness, persistence, and bounded owner counts; it does not add a mutation
control or a second state store. This closes the packaged status consumer
boundary only. The Host publishes the initial snapshot and refresh snapshots
after status/doctor gateway reads through the existing event bus; mutation-
driven publication, CLI consumer, provider/device/process producer, executable
verification, and acceptance remain open.

The read projection is now produced by `CloudWorkerProvisioningExtension` via
`status_snapshot`; `UnifiedGatewayApi` serializes that owner result instead of
repeating lease/image/receipt semantics in the Gateway adapter. A focused Host
contract covers the disabled, provider-not-ready projection and its wire field
names. `statePersisted` now reflects the canonical regular state-file
publication, rather than merely successful access to the in-memory owner. This
is covered in both directions by the focused contract (absent before
`persist()`, present after it), including the serialized `statePersisted` wire
field. This is owner-boundary progress only: it does not add mutation
commands or provider readiness; at this earlier stage it did not yet add the
event publication seam documented below.

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

The owner also exposes `select_provider_machine`, which requires the exact
cached binary identity and deterministically selects a matching provider,
profile and resource shape. Cache misses and incompatible choices fail closed;
the projection does not authorize or launch a worker. The cache revalidates
loader-supplied catalogs before admission, so a producer cannot bypass the
catalog's duplicate, identity or resource constraints by constructing a typed
value directly.

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

## Provider/process and CLI owner audit — 2026-10-31

A read-only audit of the current Host composition confirms that no existing
Fabushi owner can be promoted to the Crabbox provider/process producer:

- `source/host/src/runner/process_tree.rs` owns bounded cleanup for already
  owned Runner children; it does not own provider launch admission, artifact
  identity, lease claims, runtime/session identity or Cloud Worker receipts.
- `source/host/src/extensions/cloud_agents/cloud_agents_service.rs` owns the
  existing cloud-agent provider lifecycle and is not the Crabbox provider or
  node-worker owner.
- `source/host/src/network_worker_process.rs` owns the Fabushi network worker,
  not the Crabbox provider or Gateway worker lifecycle.
- `source/host/src/host_production_extensions.rs` composes the disabled
  Cloud Worker owner but does not supply configured provider binary, secret
  scope, launch custody or a process receipt producer.

The repository's existing `source/mahayana/codex-rs/cli` is a separate Codex
runtime and is not a Cloud Worker Coordinator consumer. No CLI route was added
from this audit. Adding a direct `Command::new` provider launcher, reusing a
generic process owner as if it owned Crabbox state, or treating the catalog
loader/process receipt shape as production discovery would violate the
Crabbox owner boundary and could falsely promote `providerReady`.

The provider catalog, process receipt and doctor contracts therefore remain
owner-bound but producer/consumer-unwired. `providerReady=false`,
`productionMutation=false`, provider/device/process acceptance, CLI reachability
and exact-head verification remain pending. This audit used source inspection
only; no local OCRM build/test/lint, Actions, push or PR update was performed.

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

## Lease lifecycle persistence façade — 2026-10-31

The Host extension now exposes typed `admit_lease` and `settle_lease`
operations. Both use the canonical owner and atomic store; replay, owner-epoch,
generation, runtime identity and outcome-unknown rules remain in the lease
owner. This is lifecycle/state persistence wiring only and does not discover a
provider, enroll a device, launch a process or authorize a consumer.

## Bootstrap record persistence façade — 2026-10-31

Bootstrap records are now registered and settled through the Host extension and
the same canonical owner snapshot. Lease scope, epoch, generation and runtime
identity are checked before persistence; diagnostics remain bounded and
redacted, and opaque secret references are retained without resolving them.
Provider/process, FDN enrollment and platform consumers remain pending.

## Artifact admission persistence façade — 2026-10-31

Artifact admissions now enter the canonical owner through policy validation and
can be settled as verified, damaged or outcome-unknown with atomic snapshot
persistence. The owner revalidates persisted digest/source structure on load;
download, credential egress, atomic artifact publication and provider
consumers remain pending.

Bootstrap admission now also requires the requested digest and size to match a
canonical artifact record in `Verified` state; a shape-valid serialized digest
alone cannot authorize bootstrap.

## Verified byte admission persistence façade — 2026-10-31

`CloudWorkerProvisioningExtension::verify_and_register_artifact` now verifies
caller-supplied bytes against the typed source digest and policy, constructs a
`Verified` artifact admission, and persists it through the same canonical owner
and atomic store used by other Cloud Worker mutations. Digest, origin and size
failures remain explicit errors. This binds proof to persistence without
introducing download, credential egress, extraction, publication, provider
launch or packaged consumer behavior; those producer and consumer paths remain
pending.

## Managed artifact publication foundation — 2026-10-31

`CloudWorkerArtifactPublisher` now provides the local managed-publication
boundary for verified bytes. It is digest-addressed by artifact kind, refuses
symlink/non-regular destinations, publishes through a private create-new temp
file plus file sync, atomic rename and directory barrier, and permits only
byte-identical idempotent re-publication. Network acquisition, extraction,
credential egress, provider launch and packaged consumers remain outside this
seam. The separate extension façade now composes publication with admission
state, but the production producer/bootstrap consumer path remains absent, so
the artifact responsibility stays `wiring=not-wired`, `verification=not-run`,
`acceptance=pending`.

The extension contract now also records the positive persistence and digest
mismatch negative oracles in
`source/host/src/extensions/cloud_workers/extension.rs`; they are authored for
Actions execution and have not been run locally under this task's verification
constraints.

## Managed artifact publication owner composition — 2026-10-31

`CloudWorkerProvisioningExtension::publish_verified_artifact` now composes
managed publication with the canonical admission owner. It persists `Pending`
before publishing, settles known write/admission failures as `Damaged`, maps a
directory-barrier failure to `OutcomeUnknown`, and settles `Verified` only
after the digest-addressed file is published. If the owner settlement itself
fails, the façade returns an explicit non-success result and does not claim
that file and owner state are durably reconciled. This is local publication
and recovery-state composition only; acquisition, extraction, provider launch,
bootstrap/process consumers and packaged acceptance remain pending.

The extension also exposes read-only
`resolve_verified_artifact`, which rechecks the canonical `Verified` admission
and the digest-addressed regular file before a future consumer uses it. A
missing, replaced or mismatched file returns an explicit failure and is never
silently repaired or accepted from owner state alone.

Bootstrap now has a matching
`resolve_current_bootstrap_artifact` façade that first revalidates lease,
scope and runtime fencing and then returns the revalidated published handle.
It remains a producer handoff contract; enrollment, secret resolution, process
launch and provider consumption are not wired.

Warm-image capture ambiguity now has a typed extension façade that persists
`CaptureUncertain` and cleanup debt under generation fencing. Provider snapshot
reconciliation and cleanup consumers remain pending.

The extension now also exposes `inspect_warm_image`, which reads through the
canonical owner and revalidates the persisted warm-image record before a future
snapshot/recovery consumer can use it. It does not query, capture, delete or
scrub a provider image.

The current implementation boundary therefore contains persistence and
fencing contracts only; it has no runtime-control producer, process launcher,
device enrollment caller or packaged consumer acceptance.

Bootstrap diagnostics now pass through the existing Host error sanitizer before
they are marked redacted and persisted. Bearer/basic credentials, known secret
prefixes, sensitive query parameters and PEM blocks therefore use the canonical
redaction path, with the source-backed 2,048-character bound retained. This is
diagnostic admission hardening only; it does not establish provider, process,
device or packaged acceptance.

The read-only doctor Inspect route now projects live canonical snapshot counts
and whether the state path exists. Repair, secret migration and reconciliation
actions remain acknowledgement-gated and explicitly unwired.

The status projection now also carries additive `bootstrapCount` and
`artifactCount` fields, with Coordinator decoding defaults for older Host
payloads; provider readiness and production-mutation remain fail-closed.

The Coordinator gateway client now has a typed `cloudWorkers.doctor`
projection/dispatch. Doctor request arguments remain Host-owned JSON contract
data; Coordinator stores no recovery state and malformed responses fail closed.

## Runtime-control admission foundation — 2026-10-31

`source/host/src/extensions/cloud_workers/runtime_control.rs` now defines the
side-effect-free F03 admission contract for `Prepare`, `Warmup`, `Bootstrap`,
`Inspect`, `Heartbeat`, `Stop` and `Dispose`. It maps only valid current lease
lifecycles to completed target states (inspect/heartbeat return to `Enrolled`),
preserves the lease ID/owner epoch/generation/runtime identity and validates
the replay key before returning a `CloudWorkerLeaseMutation`.
`CloudWorkerProvisioningOwner::admit_runtime_control` and the Host extension
provide the single owner admission/settlement façade, with action-specific
next-state selection and explicit outcome-unknown handling. No provider RPC,
process cleanup, device enrollment or persistence is performed by this slice;
the future producer remains responsible for the external side effect. The
runtime-control row therefore remains `wiring=not-wired`, `verification=not-run`,
`acceptance=pending`.

The admission request now also carries validated timeout and output bounds;
zero or excessive values fail closed before the fenced mutation is returned.
These are typed producer inputs only, with no timer, process transport,
provider RPC or output capture wired.

`CloudWorkerRuntimeControlResult` and
`settle_bounded_runtime_control` now validate action/mutation identity,
elapsed-time bounds (with timeout overruns requiring explicit
`OutcomeUnknown`) and diagnostic redaction before lifecycle settlement. This
classifies a producer observation without persisting raw output or creating a
provider/process executor.
