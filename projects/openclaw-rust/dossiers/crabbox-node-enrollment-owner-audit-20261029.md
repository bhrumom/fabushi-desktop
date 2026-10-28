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

The following reverse/provider callers were subsequently read at the same pinned tree: `extensions/crabbox/index.ts`, `extensions/crabbox/src/crabbox-worker-provider.ts` (complete 691-line source), `extensions/crabbox/src/crabbox-worker-warm-image-allocation.ts`, and `extensions/crabbox/src/crabbox-gateway-methods.ts`. Their tests and the remaining provider helper graph are still outside this closure claim.

The directly invoked helper graph was then read at the same pinned tree: `crabbox-worker-preflight.ts` (`450d6985c8cfbabbcdfe71164a10a2748b9f35e0`), `crabbox-worker-project.ts` (`35a5d4329a405437f076c0955043b411f36199dc`), `crabbox-worker-provision-commands.ts` (`f13913eee2587a64b23c85a2213825f2c7111a26`), `crabbox-worker-setup-egress.ts` (`03b9250850bd2f9210698212156ac210beb7e3b6`), `crabbox-worker-provider-binary.ts` (`a9361ad684143856c5f57336940bdebebb57016d`), `crabbox-worker-snapshot-actions.ts` (`78a412b39bfd4b54b7935049d4a8c3f08e8b4f57`), `crabbox-worker-warm-image-policy.ts` (`6559c442a8b956a6704f52085ff5c0259d901728`), `crabbox-worker-heartbeat.ts` (`f9f39d8ac6df3248532ef6dede7b728aeba6c0f9`), `crabbox-worker-inspect.ts` (`ec9e52f9a146c117bd4ae23a11c5615218137b87`), `crabbox-worker-warm-image.ts` (`af4b96a5fab3adf751864b784981e01c61fb8be5`), and `crabbox-worker-warm-image-store.ts` (`1031bf93f52dc15f65eaacd4e7a0a6197ace60bd`). The warm-image capture/checkpoint/recipe/records helpers and their tests remain open closure work.

The remaining directly invoked implementation helpers were read as well: `crabbox-worker-warm-image-capture.ts` (`12cc2cbcd40a33502927bc1a284216cc66428912`), `crabbox-worker-warm-image-checkpoint.ts` (`4f2c80c517d8be3c075c72fbf6ee40ab39ae9458`), `crabbox-worker-warm-image-recipe.ts` (`094708190c05b5d088a59d94b2989886c524d74a`), `crabbox-worker-warm-image-records.ts` (`c1e56eefef55704a5c3f9124a67412eb3f23ff31`), `crabbox-worker-warm-image-allocation.ts` (`27427bcb41602ad97b157f7f6e736b20b44b1221`), `crabbox-process.ts` (`0ccfe03b9db815de9ea466891396eacc76597203`), `crabbox-managed-binary.ts` (`ac6297fbaaaf821fd1f2f4c9919f36732829c6d8`), and `crabbox-binary.ts` (`27f5952d24b8fd5133acd86e2fc4aeb13a7e22cc`).

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

### Provider and reverse-caller composition

The plugin entrypoint registers the Crabbox tool, CLI, warm-image gateway methods, worker provider, optional sandbox backend, and cleanup service. It therefore establishes a provider/plugin lifecycle owner rather than a node-RPC implementation detail.

The provider composes binary resolution, machine options, heartbeat renewal, warm-image allocation/capture/maintenance, profile and timeout parsing, project preparation, provider inspection/cleanup, node enrollment setup, desktop preparation, and execution-mode declarations. Provisioning is explicitly node-enrollment-dependent and returns the enrolled `deviceId`; failed or indeterminate phases preserve lease cleanup/replay semantics. Warm-image allocation separately owns checkpoint fork/cold fallback and records operating-system/machine-class invariants. Gateway image methods own snapshot list/recovery/mutation validation and do not provision or execute a node.

This reverse-caller read strengthens the rejected-owner result: the Crabbox provider and plugin lifecycle cannot be recreated inside `call_openclaw_node_register` or `call_openclaw_node_invoke`. Those Fabushi handlers may consume an already-established native device/pairing or local-exec capability, while provider/artifact/bootstrap/process/desktop responsibilities require a separate future owner and ADR.

The helper responsibilities are now explicit. Preflight rejects unsafe AWS instance profiles and unmanaged Hetzner desktop coordinators. Project preparation keeps Git/project authority with the existing Core/project contract, while only lease transport, setup egress, runtime preparation, and bounded upload are delegated to Crabbox. Provision commands own fixed-lease inspection, security attestation, readiness polling, retry class, setup idempotence, desktop setup, and cleanup-indeterminate errors. Setup egress keeps upstream secret grants in the host child, passes only sentinels/proxy material to the lease, enforces Linux/native-egress capability, and requires process-tree extinction. Binary resolution is provider-owned and cancellation of one waiter cannot cancel shared acquisition. Snapshot actions track operation settlement and resolve all configured binaries without inventing a replacement. Heartbeat renewal observes lease lifetime and stops on provider capability refusal; inspect parsing redacts and bounds failure detail.

Warm-image state is a compare-and-apply keyed store with bounded profile/allocation capacity, generation identity, pinned/previous image ownership, replay identity checks, cold fallback reasons, capture uncertainty recovery, and explicit retirement obligations. Allocation freezes the selected checkpoint or cold choice before the first provider call; publication and demand renewal are generation-fenced. These invariants are source behavior to preserve in any future Rust owner, not justification for adding warm-image state to node RPC or existing FDN identity state.

The capture/checkpoint layer adds further non-negotiable boundaries: checkpoint creation distinguishes not-submitted/native-unsupported from ambiguous provider outcomes; capture claims a compare-and-apply operation before scrubbing credentials, refuses enrollment while capture is unresolved, publishes only matching lease/profile/runtime generations, retains deletion debt, and treats optional teardown capture differently from required project preparation. Managed Crabbox binary acquisition is version-gated, HTTPS/allowlist/size/checksum/archive bounded, published under a file lock with damaged installations preserved, and shared across waiters with last-waiter cancellation. Binary discovery resolves explicit/sibling/PATH candidates without treating a missing candidate as a successful provider.

## Fabushi owner resolution

| Responsibility | Existing owner | Decision | Reason / boundary |
| --- | --- | --- | --- |
| Fabushi account and device identity, device secret, enrollment grant, FDN control URL, tsnet worker lifecycle | `source/host/src/network_enrollment_owner.rs` | Reuse and extend only for Fabushi-native network enrollment | It owns the canonical Fabushi credential/device scope, persisted enrollment state, generation fencing, outcome-unknown handling, and native network worker. It is not a Crabbox provider catalog or OpenClaw Node bootstrap owner. |
| Native remote Computer operations, sensitive-result transport, grants, snapshot fencing | `source/host/src/remote_computer_target.rs` | Reuse for the resulting device capability consumer | It already owns Computer authorization and operation/result fencing. It must consume a typed native worker/device contract; it must not launch a second OpenClaw runtime or store Crabbox enrollment state. |
| Sand gateway/box connection and local Docker VM lifecycle | `source/electron-main/box/box-host-connector.ts`, `source/electron-main/box/local-docker-host-connector.ts` | Keep as platform/box bridges; do not assign Crabbox bootstrap ownership | These connectors resolve Sand gateway descriptors, local VM readiness, and box credentials. They do not own Crabbox provider catalogs, remote artifact verification, or node enrollment. |
| Crabbox provider catalog, verified worker artifact bootstrap, plugin activation, node process lifecycle and enrollment diagnostics | No existing complete Fabushi owner identified | `new_owner_proposal` pending rejected-owner analysis and ADR | The responsibility is materially broader than FDN enrollment, Computer execution, or Sand box connection. It requires a minimal source-neutral Rust owner under the existing Host/Runner/Coordinator boundaries, with platform launch remaining a thin bridge. |

The proposed owner is not permission to create an `OpenClawRuntime`, second Agent loop, second session store, or second scheduler. Its state must be a scoped worker/bootstrap lease consumed by existing Host/Runner/Computer owners; canonical Fabushi identity, conversation, transcript, permission and network state remain with their current owners.

## Required next evidence

- Read and map the imported profile, process, command, script, desktop-setup, timeout, provider helper, and reverse caller/test contracts at the same pinned upstream tree.
- Read the platform-specific `crabbox-worker-desktop-macos.ts` and `crabbox-worker-desktop-windows.ts` contracts and their tests before any owner/ADR decision.
- Read and map all remaining warm-image and managed-binary tests, then read the provider's reverse callers and fixtures before any owner/ADR decision.
- Inspect the current Fabushi Host/Coordinator production composition and determine the narrow typed contract for provider catalog, artifact provenance, worker lease, process lifecycle, and diagnostic evidence.
- Complete rejected-owner analysis and an ADR before creating a new owner. The current `NetworkEnrollmentOwner`, `remote_computer_target`, Sand connectors, and Runner process owners are candidates but cannot be widened by assumption.
- Add positive and negative oracle rows for malformed catalogs, incompatible targets, credential non-propagation, archive hash/range/TLS failures, unsafe paths, cancellation cleanup, plugin activation failure, stale runtime pointers, diagnostic truncation/redaction, and node launch outcome-unknown.
- Run all executable checks only in GitHub Actions. Local work here is read-only investigation and dossier/spec editing; this dossier is not implementation, wiring, verification, or acceptance evidence.

## Status

This audit reduces the next frontier by establishing the exact responsibility boundary and prevents an incorrect reuse of FDN or Computer owners. It does not change any global OCRM flag, inventory read state, production wiring, or acceptance state.
