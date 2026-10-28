# OCRM technical dossier: Crabbox node-enrollment owner audit

Status: scoped audit complete for the seven source files listed below; broader provider/process callers and production acceptance remain pending.

Project: OCRM-001  
Upstream repository: `openclaw/openclaw`  
Upstream commit/tree: `ad8d12f126d29b0db204187adf5092db80294e70` / `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`  
Audit date: 2026-10-29 (Asia/Shanghai)

## Source slice

The following exact pinned files were read completely in this audit:

- `extensions/crabbox/src/crabbox-worker-machine-options.ts` (`7c66b6ffab48d9dfca6131ceaef064fc461e3499`)
- `extensions/crabbox/src/crabbox-worker-node-enrollment-diagnostics.ts` (`1d1b10d90ad31543ab70d27065767daf12050a8d`)
- `extensions/crabbox/src/crabbox-worker-node-enrollment.ts` (`7860f5b649f05beae18fd848c01dbdda34bb1236`)
- `extensions/crabbox/src/crabbox-worker-profile.ts` (`e182292b6f4fe4a16e20fa6679e680b0250dbbd1`)
- `extensions/crabbox/src/crabbox-worker-node-process.ts` (`414d84ab591091511ee6abd66bd12d8b064e4763`)
- `extensions/crabbox/src/crabbox-worker-desktop-macos.ts` (`1a049d4ec64d0843f0420ade4135cff1525a4fa4`)
- `extensions/crabbox/src/crabbox-worker-desktop-windows.ts` (`c0146aad36bbe2af0c7ea22ed7744bacbcade9ff`)
- `extensions/crabbox/src/crabbox-worker-node-enrollment.test.ts` (`b6f534ec508c97c968e9e011d0ab710eb20dab`)
- `extensions/crabbox/src/crabbox-worker-node-enrollment-diagnostics.test.ts` (`8ed15639eaf6189ad2bd5a4e686316591fb77d88`)

The enrollment file imports and inlines behavior from `crabbox-worker-script.ts`, `crabbox-worker-command.ts`, `crabbox-worker-desktop-setup.ts`, and timeout/process helpers. The profile and process contracts plus the macOS/Windows platform bridges are included in this slice; the remaining helper files and reverse callers are not marked complete by this dossier.

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

### Profile, lease, command and process contracts

The dependent profile contract adds several non-obvious responsibilities that must not be lost in a Rust translation:

- profile keys are closed; unknown keys, inline setup secrets, retired `setupEnv`, unsafe environment names, non-store SecretRefs, invalid absolute binary paths, unsupported desktop providers, invalid targets, and non-positive/overflowing Go durations are rejected;
- lease IDs are domain-separated and derived from the operation ID; legacy random provision IDs are rejected rather than replayed;
- warm-image keys include provider, target, setup, protected-secret mode, sorted secret references, desktop mode, and exact machine class; Linux warm preparation and setup-secret capability are independently resolved;
- setup shell compatibility is tied to the configured target interpreter; WSL2 desktop is rejected, native Windows is required for desktop, and the machine picker preserves a configured class as a default even when it is absent from the current catalog;
- allocation arguments explicitly select public networking, disable Tailscale, preserve provider/target/windows-mode/market/TTL/idle-timeout/lease identity, and add desktop/browser/XFCE flags only for compatible targets.

The command layer owns a separate bounded process/tree custody contract. It restricts lease scripts to `--no-sync`, forwards only explicitly named environment variables, caps output at 64 KiB, redacts and suffix-truncates diagnostics, distinguishes ordinary process failures from coordinator transport failures, retries only fixed-lease resolution cases within the caller deadline, and treats ambiguous mutation results as non-replayable. Lease stop has a provider-specific Azure recovery path and recognizes an absent lease only from tightly matched inspect/release evidence.

The process runtime verifies an existing node by PID, executable/CLI, runtime directory, state directory, start time, environment, and platform-specific process receipt. It refuses replay when the runtime pointer or launch receipt is incomplete, refuses a mismatched live process, and cleans only the owned process tree/desktop host on launch failure. Linux, Windows, Darwin, signed macOS desktop host, Windows desktop launcher, and XFCE/browser/terminal setup are separate platform responsibilities; none can be collapsed into the native FDN worker process.

### Platform bridges and falsifiable tests

The macOS bridge requires the inspected non-root GUI account, console ownership and `launchctl` GUI session, a separately signed `ai.openclaw.cloud-worker` app with the expected host version and bundled CUA driver, lease-scoped private desktop state, and process/browser ownership checks on the loopback CDP port. The Windows bridge requires an active interactive session/SID, a running privileged Crabbox launcher service, lease-scoped ACLs, loopback-only authenticated TightVNC, and process/session/SID/executable identity checks for browser, terminal, and Node. Both bridges preserve cancellation/outcome-unknown handling and never treat a port or PID alone as ownership proof.

The pinned enrollment tests provide direct positive and negative oracles for: runtime reuse and warm-image upgrade, occupied runtime pointers, plugin activation failure, malformed credential non-disclosure, FIFO archive rejection, concurrent artifact preparation, process identity replay, transient HTTP retry budget, digest/length/redirect/reset/TLS failures before launch, pinned TLS before bearer transmission, desktop setup after launch and replay, exact XFCE environment binding, and invalid desktop-session rejection. The diagnostics test separately proves UTF-8/UTF-16 tail handling, redaction, runtime-path truncation, dead-process markers, and the 2,048-byte cap. These are source oracles only; no local execution is claimed.

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
- Read the platform-specific `crabbox-worker-desktop-macos.ts` and `crabbox-worker-desktop-windows.ts` contracts and their tests before any owner/ADR decision.
- Read the remaining helper source (`crabbox-worker-script.ts`, `crabbox-worker-command.ts`, `crabbox-worker-desktop-setup.ts`, timeout/process helpers), provider entrypoint, warm-image callers, and all reverse tests before any owner/ADR decision.
- Inspect the current Fabushi Host/Coordinator production composition and determine the narrow typed contract for provider catalog, artifact provenance, worker lease, process lifecycle, and diagnostic evidence.
- Complete rejected-owner analysis and an ADR before creating a new owner. The current `NetworkEnrollmentOwner`, `remote_computer_target`, Sand connectors, and Runner process owners are candidates but cannot be widened by assumption.
- Add positive and negative oracle rows for malformed catalogs, incompatible targets, credential non-propagation, archive hash/range/TLS failures, unsafe paths, cancellation cleanup, plugin activation failure, stale runtime pointers, diagnostic truncation/redaction, and node launch outcome-unknown.
- Run all executable checks only in GitHub Actions. Local work here is read-only investigation and dossier/spec editing; this dossier is not implementation, wiring, verification, or acceptance evidence.

## Status

This audit reduces the next frontier by establishing the exact responsibility boundary and prevents an incorrect reuse of FDN or Computer owners. It does not change any global OCRM flag, inventory read state, production wiring, or acceptance state.
