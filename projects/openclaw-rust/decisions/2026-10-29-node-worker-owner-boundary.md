# Node-worker launch owner boundary

Status: active decision; does not accept worker parity or production wiring
Date: 2026-10-29 (Asia/Shanghai)
Project: OCRM-001 / OCRM-M13

## Observed source and target facts

The locked OpenClaw worker launch path spans `src/worker/node-supervisor-protocol.ts`, `src/worker/launch-descriptor.ts`, `src/node-host/node-worker-launch-transport.ts`, `src/shared/worker-bundle-hash.ts`, and the process-supervisor/container lifecycle helpers. It binds launch identity and bundle hash before selecting local IPC versus container isolation, then delegates child ownership, cleanup, native-inference custody, and durable launch receipts to separate owners.

Fabushi currently has related but non-equivalent owners:

- `source/host/src/network_worker_process.rs` owns a network-worker child and control protocol for the network product, not OpenClaw worker bundle admission or Gateway worker sessions.
- `source/host/src/runner/process_tree.rs` owns generic process-tree termination for existing Runner supervisors, not worker launch claims, bundle namespace, or receipt identity.
- `source/host/src/extensions/cloud_agents/cloud_agents_service.rs` owns cloud-agent provider launches and artifacts, not node-local worker bundles or Gateway-side worker receipts.
- No current Host/Runner owner exposes the combined `launchId + planHash + environment/session + ownerEpoch + placementGeneration + runId` lifecycle required by the locked node-worker protocol.

## Decision

Do not widen any of those owners or create a second generic worker runtime inside `node_host`. The node-host seam may validate launch identity, namespace/hash/path containment, and receipt correlation, while the eventual production adapter must compose an explicitly designated existing Runner/process/container owner. Until that owner and its exact lifecycle contract are approved, node-worker commands remain unavailable rather than spawning an untracked child.

The Rust seams added so far are therefore limited to:

- `worker_supervisor_protocol.rs`: cross-field identity and bounded receipt admission;
- `worker_bundle.rs`: canonicalized bundle entry resolution and containment;
- existing Host/Runner owners: only future consumers, not implicitly selected by this record.

## Required follow-up before wiring

1. Select or approve one existing process/container lifecycle owner through a source-backed owner review.
2. Define the durable claim/receipt store and cleanup binding without adding a node-host shadow store.
3. Adapt the chosen owner to the validated launch binding, preserving outcome-unknown and predecessor-generation fencing.
4. Add exact-head Actions and real packaged/provider evidence before changing the parity row from `not-wired`.

## References

- `docs/specs/node-host-runtime-embedding.md` R10, §7, §10, §11
- `projects/openclaw-rust/dossiers/node-host-scoped.md`
- `source/host/src/network_worker_process.rs`
- `source/host/src/runner/process_tree.rs`
- `source/host/src/extensions/cloud_agents/cloud_agents_service.rs`
- locked `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`
