# OCRM technical dossier: Crabbox node-enrollment owner audit

Status: scoped audit complete for the three source files listed below; dependent profile/process/script contracts and production acceptance remain pending.

Project: OCRM-001  
Upstream repository: `openclaw/openclaw`  
Upstream commit/tree: `ad8d12f126d29b0db204187adf5092db80294e70` / `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`  
Audit date: 2026-10-29 (Asia/Shanghai)

## Source slice

The following exact pinned files were read completely in this audit:

- `extensions/crabbox/src/crabbox-worker-machine-options.ts` (`7c66b6ffab48d9dfca6131ceaef064fc461e3499`)
- `extensions/crabbox/src/crabbox-worker-node-enrollment-diagnostics.ts` (`1d1b10d90ad31543ab70d27065767daf12050a8d`)
- `extensions/crabbox/src/crabbox-worker-node-enrollment.ts` (`7860f5b649f05beae18fd848c01dbdda34bb1236`)

The enrollment file imports and inlines behavior from `crabbox-worker-profile.ts`, `crabbox-worker-node-process.ts`, `crabbox-worker-script.ts`, `crabbox-worker-command.ts`, `crabbox-worker-desktop-setup.ts`, and timeout/process helpers. Those dependencies are not marked complete by this dossier; their exact-source reading and reverse callers are the next closure step.

## Observed responsibilities

### Machine catalog and setup choices

`createCrabboxMachineCatalogResolver` invokes the provider binary's `providers --json` command with a bounded one-megabyte output allowance, process-tree cleanup, and a timeout. It parses only enrolled targets, accepts mapped class profiles, prefers `amd64` when present, preserves positive CPU/memory values, and caches successful or empty catalog promises per resolved binary. A failed load removes the cache entry and emits a warning rather than returning an invented catalog. `createCrabboxMachineOptionsResolver` projects that catalog through the parsed worker profile and returns only setup-compatible operating systems and machine classes.

The observable contract is therefore provider capability discovery plus deterministic UI/setup projection, not machine provisioning itself. A missing binary, malformed JSON, unmapped class catalog, incompatible target, or failed command must remain an explicit empty/failed discovery result.

### Enrollment diagnostics

`collectCrabboxNodeEnrollmentEvidence` runs a lease-scoped diagnostic command after the enrollment deadline. The generated script reads only the lease's runtime symlink, PID liveness, and a bounded tail of `node.log`; it handles UTF-8 and UTF-16LE logs and keeps an absent/dead marker when evidence is unavailable. Output is redacted, whitespace-normalized, independently bounded for the runtime path, and capped at 2,048 bytes for the evidence tail. Diagnostic failure changes the label to `box evidence unavailable`; it does not replace the original enrollment failure or claim that the node is healthy.

### Bootstrap and activation lifecycle

`createCrabboxNodeEnrollmentSetup` and `createCrabboxNodeRuntimeSetup` share one setup generator. The generator:

1. removes bootstrap and worker-bundle tokens from the serialized bootstrap object;
2. forwards credentials only through private environment variables, then deletes them from the child environment before launching descendants;
3. creates a lease-scoped state directory and a content-addressed runtime directory;
4. validates runtime identity, package name/version, archive size/hash, regular-file/link safety, and immutable worker archive naming;
5. downloads artifacts over HTTP(S) with range-resume, response/range/length validation, optional TLS fingerprint validation, bounded retries and explicit no-progress termination;
6. installs the verified Node artifact with a bounded owned child process and exact process-group cleanup on cancellation or timeout;
7. optionally overlaps worker-bundle download with runtime installation, publishes only verified bytes, and removes stale content-addressed archives;
8. enables the declared plugin set only after runtime verification, writes setup code only for connect mode, activates the runtime pointer, launches the node process, and performs platform-specific desktop setup;
9. emits phase markers and fails with redacted, bounded error detail while preserving cancellation and cleanup ownership.

The lifecycle has distinct preparation, runtime verification, artifact verification/publication, activation, and completion phases. A successful command construction or local fixture does not prove a real provider, artifact origin, node launch, or packaged desktop path.

## Fabushi owner resolution

| Responsibility | Existing owner | Decision | Reason / boundary |
| --- | --- | --- | --- |
| Fabushi account and device identity, device secret, enrollment grant, FDN control URL, tsnet worker lifecycle | `source/host/src/network_enrollment_owner.rs` | Reuse and extend only for Fabushi-native network enrollment | It owns the canonical Fabushi credential/device scope, persisted enrollment state, generation fencing, outcome-unknown handling, and native network worker. It is not a Crabbox provider catalog or OpenClaw Node bootstrap owner. |
| Native remote Computer operations, sensitive-result transport, grants, snapshot fencing | `source/host/src/remote_computer_target.rs` | Reuse for the resulting device capability consumer | It already owns Computer authorization and operation/result fencing. It must consume a typed native worker/device contract; it must not launch a second OpenClaw runtime or store Crabbox enrollment state. |
| Sand gateway/box connection and local Docker VM lifecycle | `source/electron-main/box/box-host-connector.ts`, `source/electron-main/box/local-docker-host-connector.ts` | Keep as platform/box bridges; do not assign Crabbox bootstrap ownership | These connectors resolve Sand gateway descriptors, local VM readiness, and box credentials. They do not own Crabbox provider catalogs, remote artifact verification, or node enrollment. |
| Crabbox provider catalog, verified worker artifact bootstrap, plugin activation, node process lifecycle and enrollment diagnostics | No existing complete Fabushi owner identified | `new_owner_proposal` pending rejected-owner analysis and ADR | The responsibility is materially broader than FDN enrollment, Computer execution, or Sand box connection. It requires a minimal source-neutral Rust owner under the existing Host/Runner/Coordinator boundaries, with platform launch remaining a thin bridge. |

The proposed owner is not permission to create an `OpenClawRuntime`, second Agent loop, second session store, or second scheduler. Its state must be a scoped worker/bootstrap lease consumed by existing Host/Runner/Computer owners; canonical Fabushi identity, conversation, transcript, permission and network state remain with their current owners.

## Required next evidence

- Read and map the imported profile, process, command, script, desktop-setup, timeout, provider, and reverse caller/test contracts at the same pinned upstream tree.
- Inspect the current Fabushi Host/Coordinator production composition and determine the narrow typed contract for provider catalog, artifact provenance, worker lease, process lifecycle, and diagnostic evidence.
- Complete rejected-owner analysis and an ADR before creating a new owner. The current `NetworkEnrollmentOwner`, `remote_computer_target`, Sand connectors, and Runner process owners are candidates but cannot be widened by assumption.
- Add positive and negative oracle rows for malformed catalogs, incompatible targets, credential non-propagation, archive hash/range/TLS failures, unsafe paths, cancellation cleanup, plugin activation failure, stale runtime pointers, diagnostic truncation/redaction, and node launch outcome-unknown.
- Run all executable checks only in GitHub Actions. Local work here is read-only investigation and dossier/spec editing; this dossier is not implementation, wiring, verification, or acceptance evidence.

## Status

This audit reduces the next frontier by establishing the exact responsibility boundary and prevents an incorrect reuse of FDN or Computer owners. It does not change any global OCRM flag, inventory read state, production wiring, or acceptance state.
