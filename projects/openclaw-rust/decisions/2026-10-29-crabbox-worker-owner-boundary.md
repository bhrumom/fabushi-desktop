# Crabbox worker provisioning owner boundary

Status: proposed implementation boundary; does not claim implementation, wiring, verification, or acceptance
Date: 2026-10-29 (Asia/Shanghai)
Project: OCRM-001 / OCRM-M10 / OCRM-M11 / OCRM-M13 / OCRM-M16
Scoped source reference: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16` (historical scoped read, not the current source lock)

Current live-main discovery candidate is `openclaw/openclaw@2c157ed06dff5032da1093cbb6745efeff2b6f33`, tree `3fb582f6a1299daa74446ef001cb7cb4fd3c2270`; it remains discovery-only until the exact-subject Actions inventory and dependency-closure authority is produced.

## Decision question

Where should the Crabbox provider, verified worker bootstrap, lease/runtime
lifecycle, warm-image state, node enrollment, diagnostics, desktop setup,
plugin/doctor lifecycle and sandbox admission responsibilities land in Fabushi
without creating a second OpenClaw runtime or widening an existing owner past
its contract?

## Source-backed rejected-owner analysis

The exact upstream responsibility decomposition and current Fabushi owner audit
are recorded in
`projects/openclaw-rust/dossiers/crabbox-node-enrollment-owner-audit-20261029.md`.
The live Fabushi files checked there show:

- `NetworkEnrollmentOwner` owns Fabushi credential-scoped grants, durable FDN
  enrollment state, generation/outcome-unknown reconciliation and the Fabushi
  network worker. It has no provider catalog, Crabbox artifact provenance,
  heartbeat/lease, plugin activation, node bootstrap or desktop-session owner.
- `remote_computer_target` owns the admitted device's Computer authorization,
  operation claims, sensitive-result vault/chunking and private transport. It
  can consume a typed worker/device capability but must not create or supervise
  the provider worker or store its lease state.
- `SandCloudAgentManager` and `CloudAgentBackend` own the existing Sand/Cursor
  background-composer wire contract, polling, artifacts, PR projection and
  team policy. They are not a generic provider/worker lifecycle owner.
- Electron Sand connectors own broker/local-Docker gateway descriptors,
  platform credentials and local VM lifecycle. They are platform bridges, not
  the Rust Crabbox provider or bootstrap owner.
- `runner/process_tree` provides bounded cleanup for an already-owned child;
  it does not own launch admission, artifact identity, lease claims, receipts
  or warm-image state. `HostRunnerComposition` wires existing Session,
  Memory, Transcript, permission and Computer surfaces and does not own their
  domain state.

Widening any rejected candidate would either mix unrelated identity/state
domains or silently omit required Crabbox behavior. Reusing the existing
process-tree primitive is allowed as an implementation dependency, but does
not transfer lifecycle ownership.

## Chosen boundary

Create one source-neutral Rust `CloudWorkerProvisioningOwner` composition under
the existing Host/Coordinator boundary, with narrow internal modules rather
than a monolithic runtime. The proposed placement is:

```text
source/host/src/extensions/cloud_workers/
  mod.rs                 # composition and typed public boundary
  provider_catalog.rs    # provider/profile capability discovery
  artifact_admission.rs  # binary/runtime/bundle provenance and verification
  lease.rs               # fixed lease identity, claims, lifetime and fencing
  warm_image.rs          # snapshot allocation/capture/retirement state
  node_bootstrap.rs      # setup, node enrollment and bounded diagnostics
  platform_bridge.rs     # macOS/Windows/Linux desktop/session adapters
  sandbox.rs             # sandbox admission and shared remote-shell contract
  doctor.rs              # health/repair projections, no hidden mutation
```

`mod.rs` is a composition owner only. Each module owns one responsibility and
has an explicit typed contract. The owner is not named `OpenClawRuntime`, does
not embed an Agent loop, and does not own conversations, transcripts,
permissions, automations, marketplace state, model routing or Fabushi device
identity.

## Owned state and contracts

The owner may persist only scoped worker-provisioning state, keyed by an
account/device scope plus a fixed lease/profile identity. Records must include
the exact provider/profile, artifact digests and provenance, lease generation,
runtime identity, owner epoch, preparation/cache keys, warm-image generation,
cleanup obligation and outcome-unknown status. It must not duplicate canonical
Fabushi credential, Conversation, Transcript, Participant, Permission,
Automation or Marketplace stores.

The public Host/Coordinator contract is typed and capability-oriented:

- discovery: provider/profile/catalog and verified artifact capabilities;
- lifecycle: admit, prepare, warmup, bootstrap, enroll, inspect, heartbeat,
  stop, dispose, capture, recover;
- execution: a scoped leased worker capability consumed by existing Runner,
  sandbox and Computer owners;
- observation: bounded status, diagnostics and cleanup obligations with
  redacted errors;
- UI/CLI: projections and commands routed through existing Coordinator and
  Electron surfaces, never a second OpenClaw dashboard or state authority.

Every mutating call carries lease identity, owner epoch, profile/runtime
identity and an idempotency/replay key. Events must distinguish submitted,
completed, failed, cancelled and outcome-unknown. A lost response cannot
silently retry a non-replayable mutation or release a lease whose stop is not
confirmed.

## Security and platform constraints

- Artifact downloads require allowlisted HTTPS/origin, bounded size and time,
  checksum/signature or equivalent provenance, safe archive paths and atomic
  publication.
- Secret refs are resolved by the existing Fabushi secret-egress/credential
  owner. Raw setup/model/device credentials never enter command arguments,
  remote logs, persisted worker records or warm images.
- Process cleanup proves ownership using lease/runtime/state identity and uses
  the existing bounded process-tree primitive; broad process-name cleanup is
  forbidden.
- Node enrollment and Computer use the existing FDN/device contracts only
  after a worker capability is admitted. Sensitive results retain existing
  account/device/session/generation fencing.
- Platform code is a thin adapter for OS session, launcher, browser/CDP/VNC
  and desktop identity. Business policy, state and retries remain in Rust.
- Sandbox execution uses the shared remote-shell/file/workspace contract and
  cannot create a second agent or conversation runtime.

## Integration and migration

1. Add the typed owner boundary and source-backed state schema without changing
   existing FDN, Computer, Sand or Runner ownership.
2. Implement the provider/artifact/lease slice, then warm-image and
   node-bootstrap slices in dependency order.
3. Expose Coordinator commands/events and adapt existing Runner/Computer/
   Electron consumers through typed capability projections.
4. Migrate any legacy Crabbox/OpenClaw records through preview, backup,
   transactional conversion and rollback; unknown or conflicting records stay
   visible as recovery obligations.
5. Keep the owner disabled when provider/account/platform prerequisites are
   absent; never report a simulated worker as production success.

Rollback is a capability/configuration cutover back to the prior Fabushi owner
for unaffected paths. Worker records are retained as explicitly versioned
recovery state until confirmed cleanup; rollback never deletes provider
artifacts or releases an outcome-unknown lease blindly.

## Required approval and evidence gates

This ADR authorizes a design boundary only. Before product implementation,
the owner must be approved against the governing OCRM spec and its affected
responsibility rows must identify source blob/symbol, target symbol, consumer,
oracle, migration and rollback. Before any completion flag changes, required
evidence includes exact-head Actions checks, security/compatibility tests,
packaged UI/CLI paths, real provider/device/platform receipts and independent
OCRM-AC01–AC24 review. Until then all implementation, wiring, verification
and acceptance states remain unchanged.

## References

- `docs/specs/openclaw-rust-equivalence-migration.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/crabbox-node-enrollment-owner-audit-20261029.md`
- `projects/openclaw-rust/decisions/2026-10-29-node-worker-owner-boundary.md`
- locked upstream Crabbox sources recorded by the dossier and inventory
