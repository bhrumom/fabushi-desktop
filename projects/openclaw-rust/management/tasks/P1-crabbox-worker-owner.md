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

The artifact-admission foundation at
`source/host/src/extensions/cloud_workers/artifact_admission.rs` is likewise
pure validation: it enforces HTTPS allowlisted origins, bounded size, SHA-256
shape, safe archive member paths and explicit pending/damaged/outcome-unknown
states. It does not download, verify bytes, publish files or mutate a managed
artifact store.

The lease foundation now also includes an explicit transition guard: only a
completed mutation may advance the lifecycle, submitted/failed/cancelled
mutations cannot silently change state, and an ambiguous result enters
`OutcomeUnknown` until a permitted recovery transition is applied.

The warm-image contract at
`source/host/src/extensions/cloud_workers/warm_image.rs` adds generation-fenced
compare-and-apply transitions, fork holds, retirement protection, capture
uncertainty and cleanup debt. It is an in-memory state contract only; no
snapshot allocation, capture, deletion or persistence is performed.

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
