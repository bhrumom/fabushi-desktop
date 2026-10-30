# OCRM P1 — Crabbox worker provisioning owner

Status: in-progress / foundation-only
Task ID: OCRM-P1-CRABBOX-WORKER
Spec: [OCRM-001](../../../../docs/specs/openclaw-rust-equivalence-migration.md)
ADR: [Crabbox worker owner boundary](../../decisions/2026-10-29-crabbox-worker-owner-boundary.md)
Source dossier: [Crabbox node-enrollment owner audit](../../dossiers/crabbox-node-enrollment-owner-audit-20261029.md)
Upstream lock: `ad8d12f126d29b0db204187adf5092db80294e70` / `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
Last updated: 2026-10-29 (Asia/Shanghai)

## 1. Context and problem

The locked Crabbox extension carries provider catalog discovery, managed
artifact admission, fixed lease and heartbeat fencing, warm-image allocation
and cleanup debt, node bootstrap/enrollment, desktop/session setup, sandbox
execution, plugin/doctor lifecycle and recovery CLI. The current Fabushi
owners were inspected and are related but not equivalent: FDN owns Fabushi
network identity, Remote Computer owns admitted-device authorization and
sensitive results, Sand owns its own box/cloud-agent contracts, Runner owns
turn/process composition, and Electron connectors own Sand platform bridges.

No current owner can absorb the full Crabbox contract without duplicating state
or silently dropping behavior. This task defines the first scoped Rust
implementation unit under the approved source-neutral boundary. It does not
claim that the OpenClaw extension, a provider, an account, a device, or a
packaged Fabushi path is currently available.

The locked repository operations sources add a non-optional execution policy:
`.crabbox.yaml` resolves the repository's provider/profile/workflow/job,
`.agents/skills/crabbox/SKILL.md` requires a resolved provider and an owned
bounded lease, and `references/operations.md` requires exact command,
observability and cleanup handling. They prohibit provider fallback, queue
waiting, invented consumer commands, secret-bearing output, and treating a
missing live-list row as proof of cleanup. Therefore the native owner must
model provider selection, lease lifetime, retained recovery state and
observability as explicit typed inputs/results; it must not embed these
operations as an unreviewed local shell wrapper or silently substitute a
different provider.

## 2. Goal

Implement and wire the Crabbox worker provisioning capability as a native Rust
Host/Coordinator owner while preserving existing Fabushi owners and exposing a
typed leased-worker capability to existing Runner, sandbox and Computer
consumers. The result must support the applicable locked-source behaviors with
explicit compatibility differences and real migration/recovery evidence.

## 3. Non-goals and hard boundaries

- Do not create `OpenClawRuntime`, an Agent loop, conversation/transcript
  store, scheduler, permission truth, marketplace, or model-routing owner.
- Do not move Fabushi account/device identity, FDN grants, Computer
  authorization, sensitive-result storage, Session/Memory/Transcript state or
  Sand cloud-agent state into the new owner.
- Do not use the OpenClaw Node runtime, Gateway daemon, TypeScript provider,
  npm install, wrapper process or remote OpenClaw service as the production
  implementation.
- Do not mark any global flag or responsibility accepted from source reading,
  a fixture, a mock, a generated schema, or a successful local command.
- Do not broaden scope to unrelated OpenClaw modules; downstream tasks must
  create their own source-backed implementation units while the full project
  remains in scope.

## 4. Functional requirements

### P1-F01 — Provider/profile catalog

Discover only explicitly configured providers and targets; parse bounded
catalog output deterministically; preserve architecture/resource constraints;
cache by exact binary identity; invalidate failed loads; and project compatible
machine/OS choices without granting permission merely because a capability is
advertised.

### P1-F02 — Artifact admission

Resolve explicit/sibling/PATH candidates; acquire managed binaries and worker
artifacts through the existing approved egress/credential boundaries; enforce
origin allowlists, HTTPS, size/time limits, archive path safety, checksum or
signature provenance, atomic publication and version admission; preserve
damaged installs and outcome-unknown state for recovery.

### P1-F03 — Lease and lifecycle fencing

Mint/validate fixed lease identities; persist owner epoch, generation, profile,
runtime identity and idempotency key; implement admit/prepare/warmup/bootstrap/
inspect/heartbeat/stop/dispose with explicit submitted/completed/failed/
cancelled/outcome-unknown states. Non-replayable mutations must not be retried
after an ambiguous response. Stop releases only after confirmed stop or a
strictly proven absence result.

### P1-F04 — Warm-image state

Implement bounded compare-and-apply records for allocation holds, checkpoint
generations, pending/available/missing/retiring/capture-uncertain states,
retention and capacity policy, generation-fenced fork/capture/delete/rollback,
cleanup debt and exact recovery selectors. Warm-image state must not enter
FDN, Computer, Session or conversation stores.

### P1-F05 — Node bootstrap and diagnostics

Prepare verified runtime/bundle inputs, keep setup/model/device credentials out
of argv/logs/remote persistent state/images, enroll through the typed existing
device/network contract, and return bounded redacted diagnostics. Preserve
process/runtime identity, cancellation custody, retry classes and late-result
outcome-unknown handling.

### P1-F06 — Platform and sandbox bridges

Keep macOS/Windows/Linux session, launcher, browser/CDP/VNC and filesystem
operations as thin platform adapters. Expose the shared remote-shell,
workspace/file/workdir contract to existing sandbox consumers. Prove process
ownership before cleanup; never use broad process-name termination.

### P1-F07 — Plugin, doctor and recovery surfaces

Register typed Coordinator/CLI/UI projections for health, managed-binary
repair, protected-secret migration, warm-image inspection and exact recovery.
Read/recovery surfaces must not run provider mutations without explicit
acknowledgement and must preserve unrelated state.

### P1-F08 — Existing-owner integration

Provide typed consumers for existing Runner/Computer/Sand/Electron paths.
FDN remains the sole Fabushi identity/network owner; Remote Computer remains
the sole admitted-device action/result owner; Host/Runner remains the sole turn
and process-composition owner. Every runtime command has a producer, owner and
consumer recorded in the RTM.

#### P1-F08 implementation seam (source-backed, not yet wired)

The current exact-head consumers are deliberately narrower than the Crabbox
capability and must remain the integration targets:

- `source/host/src/runner/tools/sand_computer_tool.rs` owns the
  `ComputerToolExecutor` contract and its `SandComputerToolBridge`; the
  production implementation in
  `source/host/src/runner/host_computer_tool_dependencies.rs` owns Box-backed
  Computer actions and screenshot persistence.
- `source/host/src/remote_computer_target.rs` owns the
  `RemoteComputerNativeExecutor`, tool authorization, admitted-device
  identity, private transport and sensitive-result path.
- `source/host/src/runner/tools/sand_external_machine_tools.rs` owns the
  `ExternalMachineExecutor` contract for Sand external shell/read operations,
  including operation status/history/cancellation and background polling.
- `source/host/src/runner_production_bridge.rs` and
  `source/host/src/host_runner_composition.rs` own the Runner turn/process
  composition boundary; they do not own worker lifecycle or device grants.

The next implementation unit is a typed adapter at these existing trait
boundaries. It must accept an ephemeral
`CloudWorkerLeasedCapability`/sandbox request, validate account/device scope,
lease generation, runtime identity and replay key, then delegate the actual
action to exactly one existing executor. The adapter may not mint a Computer
grant, enroll a device, write a sensitive-result store, create a second Runner
loop, or bypass Sand/FDN/Remote Computer authorization. A capability without a
completed enrolled lease, a stale generation, a mismatched scope, or an
outcome-unknown mutation must fail closed before delegation.

The adapter task remains **not started** until the ADR is approved and the
existing trait owners expose the minimum typed input needed for this validation
without widening their state ownership. Adding a standalone cloud-worker
executor, registering a second tool bridge, or changing the disabled-by-default
production extension into a live provider is explicitly not an acceptable
substitute. Acceptance requires named producer/owner/consumer rows in the RTM,
negative tests for scope/generation/replay/outcome-unknown rejection, and an
exact-head Actions proof of the real production composition path.

## 5. Target architecture and contracts

Proposed Rust placement:

```text
source/host/src/extensions/cloud_workers/
  mod.rs
  provider_catalog.rs
  artifact_admission.rs
  lease.rs
  warm_image.rs
  node_bootstrap.rs
  platform_bridge.rs
  sandbox.rs
  doctor.rs
```

The composition boundary exposes typed commands/events and a leased-worker
capability. Mutating requests include account/device scope, profile/provider,
lease ID, owner epoch, generation/runtime identity and replay key. Persistent
records are versioned, private, bounded and atomically updated. Secret values
are referenced through existing secret-egress/credential owners, never copied
into the new store.

The Coordinator projection must be explicit about authorization, account and
device scope, cancellation, timeout, reconnect, stale generation, duplicate
request and outcome-unknown behavior. UI and CLI are projections of this
contract and do not own lifecycle state.

## 6. Failure modes and edge cases

- missing/invalid/multiple provider configuration;
- malformed or oversized catalog/artifact/archive output;
- unsupported architecture/OS/provider/desktop mode;
- checksum, signature, TLS, redirect, range, timeout or no-progress failure;
- secret-store unavailable, unsafe secret name/value or credential leakage;
- duplicate/replayed lease, stale epoch/generation/runtime, lost response;
- cancellation while a child/process tree is still settling;
- provider stop/release accepted but cleanup uncertain;
- pending/uncertain warm-image capture, failed fork, pinned/held checkpoint,
  capacity exhaustion and exact-row recovery conflict;
- process PID reuse, wrong cwd/runtime/state identity or desktop session;
- FDN/device enrollment rejection, stale network binding or Computer grant
  mismatch;
- Coordinator disconnect/reconnect and UI projection receiving stale status;
- rollback with retained provider artifacts or unknown cleanup.

Each case needs a deterministic typed result, redacted diagnostic, state
transition and recovery/rollback action. Unknown is not success.

## 7. Implementation strategy

1. Re-read the latest exact source/target blobs and reconcile this task with
   the ADR, dossier, module map and RTM; obtain owner approval before creating
   the new Rust module.
2. Implement schema/types and state invariants with no provider side effects.
3. Implement catalog/artifact/lease foundations, then warm-image and bootstrap
   in dependency order; reuse existing FDN/Computer/Sand/process primitives
   only through typed contracts.
4. Add Coordinator/CLI/UI projections and real production composition.
5. Add migration preview/backup/transaction/rollback and observability.
6. Run only the required executable checks in GitHub Actions against one exact
   target head; preserve full run identity and failure evidence.
7. Perform independent spec-compliance and owner-boundary review before any
   status promotion.

## 8. Verification and evidence plan

The implementation must have source-backed unit/contract tests for every F01–
F08 invariant, including negative and cancellation cases. Required evidence
dimensions include:

- Rust type/schema and persistence compatibility;
- configured provider/artifact and secret-egress security;
- process, network, device and platform lifecycle;
- Runner/Computer/Sand/Electron consumer reachability;
- migration, rollback and recovery obligations;
- performance budgets for catalog, warmup, bootstrap, inspect/heartbeat and
  stop/cleanup;
- packaged UI/CLI and install/update behavior;
- real provider/device/platform receipts where the responsibility requires it.

All executable checks are Actions-only. Local inspection and document checks do
not upgrade `verified` or `accepted`.

## 8.1 Current implementation slice

The first implementation slice is intentionally limited to the typed lease
boundary at `source/host/src/extensions/cloud_workers/lease.rs`, exported by
`cloud_workers/mod.rs` and registered in the Host extension module. It defines
scope, provider/profile/runtime identity, lease generation, owner epoch,
replay key, lifecycle and mutation outcome types, plus pre-side-effect
identity validation. It performs no provider discovery, artifact download,
process launch, device enrollment, persistence, Coordinator wiring or UI
projection. The slice is therefore an implementation foundation, not P1
completion or production reachability.

The next slice adds the side-effect-free catalog contract at
`source/host/src/extensions/cloud_workers/provider_catalog.rs`: bounded JSON
input, explicit provider/profile/machine records, deterministic ordering,
duplicate rejection and resource/architecture validation. It still does not
discover a provider, grant permission, select a model or launch a worker.

Catalog profiles now expose deterministic OS/architecture/min-resource
matching. A match is only a compatibility projection; it does not authorize
the provider, grant a lease, select a model or launch a worker.

The catalog contract now also exposes an exact-binary-identity cache. Successful
catalog loads are reused only for the same opaque binary identity, while every
failed refresh invalidates that identity before returning the error. This is a
side-effect-free cache contract; provider discovery, binary probing,
authorization and launch remain open.

The artifact-admission foundation at
`source/host/src/extensions/cloud_workers/artifact_admission.rs` is likewise
pure validation: it enforces HTTPS allowlisted origins, bounded size, SHA-256
shape, safe archive member paths and explicit pending/damaged/outcome-unknown
states. It does not download, verify bytes, publish files or mutate a managed
artifact store. It now also exposes bounded byte-level SHA-256 verification;
digest mismatch remains a hard admission failure, while download, extraction,
publication and managed artifact mutation remain outside this slice.

The lease foundation now also includes an explicit transition guard: only a
completed mutation may advance the lifecycle, submitted/failed/cancelled
mutations cannot silently change state, and an ambiguous result enters
`OutcomeUnknown` until a permitted recovery transition is applied.

The warm-image contract at
`source/host/src/extensions/cloud_workers/warm_image.rs` adds generation-fenced
compare-and-apply transitions, fork holds, retirement protection, capture
uncertainty and cleanup debt. It is an in-memory state contract only; no
snapshot allocation, capture, deletion or persistence is performed.

The bootstrap foundation at
`source/host/src/extensions/cloud_workers/node_bootstrap.rs` keeps credentials
as opaque secret references, binds bootstrap records to the admitted scope,
runtime and artifact digest, bounds redacted diagnostics, and distinguishes
safe replay from reconcile-first outcome-unknown handling. It does not resolve
secrets, enroll a device, launch a process or claim readiness.

Bootstrap settlement is now transition-fenced: readiness requires both
preparation and enrollment, while failed/cancelled/outcome-unknown results
cannot be overwritten as ready without the declared recovery path.

The platform and sandbox boundary now has thin descriptors for OS/session
bridges and lease-scoped workspace operations. Sandbox requests use explicit
argv vectors and workspace-relative paths; they do not interpolate shell
strings, own platform policy, or create a second agent/conversation runtime.

The doctor/recovery projection at
`source/host/src/extensions/cloud_workers/doctor.rs` distinguishes inspection
from mutating repair/reconciliation and requires an explicit acknowledgement
for every mutating action. It is a request/result contract only; no repair,
migration, lease release or warm-image mutation is executed.

The Host registry now has a formal `CloudWorkers` slot in
`extension_ids.generated.rs` and `extensions/registry.rs`; production Host
composition constructs it through
`start_cloud_worker_provisioning_extension()` and exposes it on
`ProductionHostExtensions`. The slot starts disabled by default and therefore
does not claim provider readiness, Coordinator producer/consumer reachability,
packaged UI/CLI reachability, or exact-head acceptance.

The pure contracts are now composed by
`source/host/src/extensions/cloud_workers/owner.rs` into a single
`CloudWorkerProvisioningOwner` with disabled-by-default configuration,
lease/warm-image maps, independent lease and warm-image capacity fencing, and
typed mutation delegation. The
production extension boundary exists, but the owner still has no
persistence or provider side-effect adapter. The Coordinator→Host Gateway now
exposes only the read-only `cloudWorkers.status` projection; mutating commands
remain unregistered until their real producer, authorization, persistence and
consumer paths are implemented. Packaged UI/CLI reachability remains open.

Versioned persistence is now defined by
`CloudWorkerPersistentSnapshot` and `CloudWorkerProvisioningStore`: lease and
warm-image metadata are serialized deterministically and written through an
exact-path temporary file, file sync, rename and (on Unix) parent-directory
sync. A directory-barrier failure is returned as a write failure after the
rename, preserving the possibility of an outcome-unknown publication rather
than claiming durable success. Corrupt/unsupported snapshots fail closed;
secret payloads, credentials and Fabushi identity stores are not part of the
snapshot. The production extension now injects
`app_data_dir/cloud-workers/state.json`, loads existing state fail-closed and
exposes an atomic `persist()` boundary. Migration/rollback receipts and
`backup_existing` now syncs the copied backup and its directory entry before
returning success; a barrier failure leaves the backup visible but reports a
non-success outcome for recovery handling. Migration/rollback receipts and
mutation-triggered persistence remain open. Both load and save reject state
JSON larger than the explicit 8 MiB private-state bound before decoding or
publication. The snapshot now carries
append-only `CloudWorkerMigrationReceipt` records, and the store exposes an
explicit non-overwriting `backup_existing` primitive; no legacy conversion is
claimed until a real source format and migration oracle are supplied.

The Coordinator crate now owns the serializable projection contract in
`source/node-agent-coordinator/src/cloud_worker_projection.rs`, with the exact
`cloudWorkers.status` method and `cloud-workers.status` event family. It is a
schema/command boundary only. `CoordinatorGatewayClient` now has a dedicated
status projection call that reuses the existing gateway transport and error
classification; a live event publisher and renderer/CLI consumer are still
open.

The Coordinator event projection now has a family-scoped decode/encode
boundary for `cloud-workers.status`: unknown families and malformed payloads
are rejected, while valid status events round-trip through the typed event.
This remains a protocol contract only; it does not claim a live publisher,
renderer, CLI, or provider mutation path.

The owner now exposes an ephemeral `CloudWorkerLeasedCapability` only for an
enrolled lease whose last mutation completed successfully. The capability is
fenced by account/device scope, lease ID, owner epoch, generation and runtime
identity, and carries no lifecycle or authorization state of its own. Runner,
Computer and sandbox adapters remain responsible for consuming this contract;
provider launch and real consumer wiring remain open.

## 9. Definition of Done / acceptance criteria

- P1-AC01: ADR is approved and the target symbols/consumers are recorded in
  the RTM; no rejected existing owner is widened by assumption.
- P1-AC02: F01–F08 have Rust implementation contracts and no duplicate
  canonical state owner.
- P1-AC03: all affected source/oracle rows are read-complete, mapped and
  linked to target symbols and negative oracles.
- P1-AC04: exact-head Actions prove required unit, contract, security,
  migration, performance and package checks; failures remain visible.
- P1-AC05: packaged Coordinator/CLI/UI paths reach the owner and existing
  Runner/Computer/Sand consumers with account/device isolation.
- P1-AC06: real provider/device/platform acceptance and cleanup/recovery
  receipts exist for every applicable lane; unavailable dependencies remain
  explicitly blocked.
- P1-AC07: independent review confirms no OpenClaw runtime/agent/session/
  scheduler/marketplace duplication and no owner-boundary regression.

Completion of P1 does not set project-wide global flags unless the main OCRM
spec's exact-target acceptance predicates are independently satisfied.

## 10. Release, migration, rollback and observability

Ship behind an explicit capability/configuration gate with safe disabled and
unsupported states. Persist versioned worker records and migration receipts;
backup before conversion; retain unknown cleanup obligations; and make
rollback idempotent. Emit redacted lifecycle phase, provider, lease generation,
runtime identity hash, duration, retry class, cleanup state and outcome-unknown
telemetry. Never emit credentials, raw secret refs, full command lines or
unbounded provider output.

## 11. References and provenance

- OCRM-001 main spec and Source of Truth.
- `projects/openclaw-rust/decisions/2026-10-29-crabbox-worker-owner-boundary.md`.
- `projects/openclaw-rust/dossiers/crabbox-node-enrollment-owner-audit-20261029.md`.
- Exact upstream Crabbox source identities recorded in the dossier/inventory.
- Live Fabushi owner source identities recorded in the dossier.

Current state: typed lease foundation implemented locally; provider/artifact,
warm-image, bootstrap, platform, sandbox, doctor/recovery, integration,
executable verification and acceptance remain open. The ADR remains a proposed
boundary until owner approval and the required RTM review are recorded.
