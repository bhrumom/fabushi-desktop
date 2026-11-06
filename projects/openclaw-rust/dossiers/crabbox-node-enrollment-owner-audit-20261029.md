# OCRM technical dossier: Crabbox node-enrollment owner audit

Status: scoped audit complete for the current Crabbox source slice; broader provider/process callers and production acceptance remain pending.

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

The directly corresponding oracle files were read at the same tree: `crabbox-worker-warm-image-allocation.test.ts` (`89af0fe6b67238f2605fb960287d56c2e7f3c811`), `crabbox-managed-binary.test.ts` (`384ec9d5792f532401ade5e2ade5d61ceccedcc3`), `crabbox-binary.test.ts` (`09291b712594ad0fbb64e5652718a692b1f77f8f`), and `crabbox-process.test.ts` (`d3c6721a6c3938074e4e08b55b8bb0af49010a77`). These are source oracles; they have not been executed locally.

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

The focused test oracles cover capture lifetime ordering and scrub suppression after recovery, required versus optional capture failure, cold/snapshot status reporting, failed-fork retirement, unsupported capture receipts, pinned-generation retention, borrower-held deletion debt, exact replay/cache/runtime identity, project authority expiry, uncertain capture fencing across restart, allocation capacity, managed binary checksum/archive/path safety, shared acquisition cancellation, damaged-installation recovery, download idle/total budgets, version admission and redacted probe errors, platform binary discovery precedence, and process working-directory normalization.

The provider/lifecycle oracle files were also read at the pinned tree: `crabbox-worker-provider.test.ts` (`de4ae1ab51e3465c51254918df613846025abec9`), `crabbox-worker-provider-platforms.test.ts` (`b660cf474d0192e373f40cf9b7e550c03ac86a0b`), `crabbox-worker-provider-desktop.test.ts` (`f4bd5a507f37d20dde71081ac5bf08df3d4c0ebf`), `crabbox-worker-provider-binary.test.ts` (`85f31de011c97c968e9e011d0ab71039677e336a`), `crabbox-worker-warm-image.test.ts` (`a6932d2bb9f962379f7c8ee0f7c50d5eccb9ee7c`), `crabbox-worker-warm-image-lifecycle.test.ts` (`ce9907a7e82397be46e50e51a2ba9c1d032799c5`), `crabbox-worker-warm-image-recovery.test.ts` (`66ddbed3b5941e17fab48ffa662cdb31a690d296`), `crabbox-worker-warm-image-policy.test.ts` (`1a3b29958c3ddd6b78d923c32061b6ae3bfc42d3`), and `crabbox-worker-warm-image-store.test.ts` (`7018deb0324e0d2cf3ac1eba37fd5b7d06c22f46`).

The remaining warm-image source/oracle closure was re-read and recorded in the
canonical inventory at the locked tree:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `crabbox-worker-warm-image-capture.ts` | `12cc2cbcd40a33502927bc1a284216cc66428912` | 519 | owner-audit |
| `crabbox-worker-warm-image-checkpoint.ts` | `4f2c80c517d8be3c075c72fbf6ee40ab39ae9458` | 226 | owner-audit |
| `crabbox-worker-warm-image-recipe.ts` | `094708190c05b5d088a59d94b2989886c524d74a` | 55 | owner-audit |
| `crabbox-worker-warm-image-records.ts` | `c1e56eefef55704a5c3f9124a67412eb3f23ff31` | 26 | owner-audit |
| `crabbox-worker-warm-image-maintenance.test.ts` | `dc5e22a5f1e7290cbaa6a5bda1ddd65e8f34de73` | 420 | test-oracle-replaced |
| `crabbox-worker-warm-image-recovery.test.ts` | `66ddbed3b5941e17fab48ffa662cdb31a690d296` | 168 | test-oracle-replaced |
| `crabbox-worker-warm-image-retirement.test.ts` | `37e850c507874009ab6a62dbf0900ef08de365f0` | 365 | test-oracle-replaced |
| `crabbox-worker-warm-image-store.test.ts` | `7018deb0324e0d2cf3ac1eba37fd5b7d06c22f46` | 243 | test-oracle-replaced |

This closure confirms the warm-image boundary: capture claims a compare-and-
apply operation, scrubs forwarded credentials before native image creation,
retains unresolved native outcomes for explicit recovery, and refuses required
project enrollment while capture remains uncertain. Checkpoint parsing requires
bounded native identities and distinguishes absent, pending and available
states. Recipe diagnostics are bounded and UTF-8 safe; legacy selectors bind
the original row bytes. The focused oracles cover conflict rechecks, capacity,
indeterminate writes, retirement/borrower fencing, restart recovery, stale
generation protection and disposal ordering.

The plugin/API boundary was also re-read and recorded in the same inventory:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `index.ts` | `f7b32bbed65069e3a0fcad1d01592f8c154ef487` | 124 | owner-audit |
| `openclaw.plugin.json` | `4c09ec4e64721e7e9f9de5b4833015a8975745e0` | 110 | owner-audit |
| `package.json` | `129116e692a6a377f25c0a281c7b602c7ce93306` | 27 | owner-audit |
| `cli-runtime-api.ts` | `26e66781b4ec4dd972f79dbb273a3aaefadc9b33` | 13 | owner-audit |
| `doctor-contract-api.ts` | `26bbb1c544e7e1818fdfc17a72491f02d18f13c3` | 306 | owner-audit |
| `doctor-health-api.ts` | `6f25e0bcff9e6c2786174b7d407df021bdee3ac0` | 22 | owner-audit |
| `worker-profile-api.ts` | `475c7b12f129f7153c46e1293ce8e5315a1f1dff` | 20 | owner-audit |

The entrypoint confirms one plugin lifecycle boundary: Crabbox registers its
tool, CLI, admin-scoped image methods, worker provider, optional sandbox
backend, runtime cleanup and provider disposal. It explicitly keeps the Agent
loop on the Gateway host and routes only exec/file work through the leased SSH
backend. The manifest and API exports preserve lazy loading, schema constraints,
doctor-only state migration, and runtime-store profile discovery. These facts
support plugin-lifecycle and worker-provider owner analysis; they do not prove
Fabushi plugin registration, sandbox execution, or release acceptance.

The managed-artifact and desktop-bridge chain was then re-read and added to
the locked source ledger:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `crabbox-managed-binary.ts` | `ac6297fbaaaf821fd1f2f4c9919f36732829c6d8` | 442 | owner-audit |
| `crabbox-managed-binary.test.ts` | `384ec9d5792f532401ade5e2ade5d61ceccedcc3` | 629 | test-oracle-replaced |
| `crabbox-worker-bootstrap-download.test.ts` | `b0b1ebab9cf49992c1221b559b636809c02d5fca` | 622 | test-oracle-replaced |
| `crabbox-worker-desktop-macos.ts` | `1a049d4ec64d0843f0420ade4135cff1525a4fa4` | 160 | owner-audit |
| `crabbox-worker-desktop-macos.test.ts` | `cf42b6d056f1ecc0fc91b370a891eadfa82da18f` | 257 | test-oracle-replaced |
| `crabbox-worker-desktop-windows.ts` | `c0146aad36bbe2af0c7ea22ed7744bacbcade9ff` | 419 | owner-audit |
| `crabbox-worker-desktop-windows.test.ts` | `1f873e3fdb31140177e84a23e8ee0eb02408f128` | 440 | test-oracle-replaced |

The managed-binary contract uses a fixed minimum release, probes configured
and managed candidates, downloads only from an HTTPS GitHub allowlist, enforces
idle/total deadlines and byte limits, verifies the published checksum, rejects
unsafe archive entries, probes the staged executable, publishes under a lock,
and preserves a damaged prior installation. Shared acquisition is reference
counted so only the last waiter cancels the install. The macOS bridge requires
the inspected non-root GUI account, console ownership, launchctl GUI session,
signed Developer ID app identity, lease-scoped state and owned browser/CDP
processes. The Windows bridge binds the active SID/session, launcher service,
ACLs, loopback-only authenticated VNC, browser identity and node process
identity, with bounded cancellation and outcome-unknown handling. These are
source contracts only; no real provider, device, package or platform
acceptance is claimed.

The provider, profile, node-process/enrollment and canonical warm-image store
were then re-read against the same exact tree and registered in inventory. The
provider remains one disposable composition owner for binary resolution,
heartbeat, allocation/capture, project preparation, node enrollment, desktop
setup, snapshot maintenance and teardown. It preserves indeterminate leases
for replay, stops heartbeats when caller ownership ends, and never fabricates a
device identity. The node enrollment boundary forwards only declared
bootstrap inputs and waits for the enrolled device result; the warm-image store
remains the sole compare-and-apply state owner with bounded records, generation
fences and retirement debt. These source facts narrow the future Rust contract
but do not establish a Fabushi implementation or acceptance.

Exact source identities for this closure are:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `crabbox-worker-provider.ts` | `a0ef59bbd42ef07b88d3bfb5a5a73fdc507b1b11` | 692 | owner-audit |
| `crabbox-worker-provider.test.ts` | `de4ae1ab51e3465c51254918df613846025abec9` | 1269 | test-oracle-replaced |
| `crabbox-worker-profile.ts` | `e182292b6f4fe4a16e20fa6679e680b0250dbbd1` | 550 | owner-audit |
| `crabbox-worker-node-enrollment.ts` | `7860f5b649f05beae18fd848c01dbdda34bb1236` | 508 | owner-audit |
| `crabbox-worker-node-enrollment.test.ts` | `b6f534ec508c97ee0cf3624b05a668570eb20dab` | 931 | test-oracle-replaced |
| `crabbox-worker-node-process.ts` | `414d84ab591091511ee6abd66bd12d8b064e4763` | 245 | owner-audit |
| `crabbox-worker-warm-image-store.ts` | `1031bf93f52dc15f65eaacd4e7a0a6197ace60bd` | 764 | owner-audit |
| `crabbox-worker-warm-image.test.ts` | `a6932d2bb9f962379f7c8ee0f7c50d5eccb9ee7c` | 418 | test-oracle-replaced |
| `crabbox-worker-warm-image-lifecycle.test.ts` | `ce9907a7e82397be46e50e51a2ba9c1d032799c5` | 334 | test-oracle-replaced |

The direct command/provision/project helper closure was also read at the
locked tree and recorded as follows:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `crabbox-worker-command.ts` | `36a37ad3f476e4baa7ace04e72202942a6c4481d` | 317 | owner-audit |
| `crabbox-worker-command.test.ts` | `ad0aaae43abda4cd3bfe215719c1f5617e5517ac` | 326 | test-oracle-replaced |
| `crabbox-worker-provision-commands.ts` | `f13913eee2587a64b23c85a2213825f2c7111a26` | 370 | owner-audit |
| `crabbox-worker-setup-egress.ts` | `03b9250850bd2f9210698212156ac210beb7e3b6` | 182 | owner-audit |
| `crabbox-worker-desktop-setup.ts` | `5fbf9a02dac8c2d16bd6459a6c590465294fcec8` | 229 | owner-audit |
| `crabbox-worker-desktop-setup.test.ts` | `d0b902ea3c7931f81165092ff32b682577676912` | 332 | test-oracle-replaced |
| `crabbox-worker-project.ts` | `35a5d4329a405437f076c0955043b411f36199dc` | 146 | owner-audit |
| `crabbox-worker-project.test.ts` | `c9501e8785abfa83ff9904e29d4a730cfb13526` | 908 | test-oracle-replaced |
| `crabbox-worker-preflight.ts` | `450d6985c8cfbabbcdfe71164a10a2748b9f35e0` | 53 | owner-audit |
| `crabbox-worker-node-enrollment-replay.test.ts` | `429f844edd2d5c07ae6d7364cd3358dd651c778b` | 422 | test-oracle-replaced |

These helpers establish the remaining control boundaries: command output is
redacted and bounded, fixed-lease coordinator retries are capped and only
replay-safe phases retry, malformed or ambiguous inspect output preserves
cleanup uncertainty, and every failed setup routes through one cleanup owner.
Setup egress keeps upstream secret grants in the host child and passes only
sentinels/proxy material to the lease. Project preparation delegates Git and
workspace authority to Core while Crabbox owns lease transport and bounded
upload. Desktop setup binds the exact XFCE session or native Windows/macOS
interactive identity, and preflight rejects unsafe provider configuration.

These tests prove, as source oracles, catalog projection and bounded recovery, setup-secret separation and revocation, preflight security rejection, readiness/timeout/cleanup classification, diagnostics redaction, fixed-lease replay after lost replies, heartbeat scheduling/fencing, platform/desktop limitations, managed binary sharing, warm-image lifecycle/recovery, compare-and-apply conflict behavior, and config-schema rejection. They do not prove Fabushi packaged behavior or real provider/device acceptance.

The worker-lifetime, sandbox and gateway/tool closure was then read at the same
locked tree and recorded in the canonical inventory:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `crabbox-worker-heartbeat.ts` | `f9f39d8ac6df3248532ef6dede7b728aeba6c0f9` | 200 | owner-audit |
| `crabbox-worker-heartbeat.test.ts` | `4dba70b8a4a4443a6ea207fbcff9c1c641f830b3` | 95 | test-oracle-replaced |
| `crabbox-worker-inspect.ts` | `ec9e52f9a146c117bd4ae23a11c5615218137b87` | 122 | owner-audit |
| `crabbox-worker-inspect.test.ts` | `ebb0703d532aca7e8c8d3442d087007267f3f02f` | 61 | test-oracle-replaced |
| `crabbox-worker-lifetime.ts` | `c548faa75d776e6d5b7c3b6ed88a2ba4eb03f808` | 233 | owner-audit |
| `crabbox-worker-lifetime.test.ts` | `6030e6b830969b27023e947010cddebb9bffe64b` | 193 | test-oracle-replaced |
| `crabbox-worker-provider-binary.ts` | `a9361ad684143856c5f57336940bdebebb57016d` | 59 | owner-audit |
| `crabbox-sandbox-config.ts` | `7f904e7871a27749b853443df8570ce82afc09c1` | 37 | owner-audit |
| `crabbox-sandbox-lease.ts` | `1408b75390c07ef16ec26bdea0a953cafe5c522e` | 9 | owner-audit |
| `crabbox-sandbox-backend.ts` | `38be0ffb5960095357ca696accdff79e59ff8317` | 293 | owner-audit |
| `crabbox-sandbox-backend.test.ts` | `9555bbdb51a6e8fe95e7609ee02297a0d23f6889` | 300 | test-oracle-replaced |
| `crabbox-gateway-methods.ts` | `b55bdd1f79ee96963561a3806c19dc8366251cba` | 178 | owner-audit |
| `crabbox-gateway-methods.test.ts` | `4b65a6064431dfc96316c99fa2b8589f6e7d2921` | 413 | test-oracle-replaced |
| `crabbox-tool.ts` | `8879fee7eee95ba233cfb615d03e7f908531b053` | 354 | owner-audit |
| `crabbox-tool.test.ts` | `3b29199a02f697a93c1ffff36c580f3f79ec0791` | 382 | test-oracle-replaced |
| `doctor.ts` | `aa26ac1aab70a3f199ab0a8698ad866b4ce9341e` | 350 | owner-audit |
| `doctor.test.ts` | `79dbcd0f1dd7069c4a9d676d4d005723ead947b8` | 363 | test-oracle-replaced |

The new source facts preserve several distinct owners. Heartbeat is a
currentness- and generation-fenced singleton per lease; it retries through the
coordinator and joins child shutdown before deleting its entry. Lifetime is
admission-only source capture and does not extend or replace a lease. Inspect
parses strict lease identity/lifetime fields and returns redacted bounded
failures, while the provider-owned binary resolver shares acquisition without
letting one caller abort other waiters. Sandbox configuration and fixed `cbx_`
lease IDs are narrow admission contracts. The shared remote-shell backend owns
workspace seeding, file operations, workdir and repository-owned cleanup after
`crabbox exec --check`; it does not become a second agent runtime. Gateway
methods validate image mutations and recovery acknowledgement, and expose
secret references without opening the secret store. The tool keeps the Agent
loop and conversation attachment on the host while routing only leased
execution/process actions. Doctor registers provider-binary, setup-secret and
warm-image health checks, with repair explicitly bounded to managed binary or
protected-secret migration. These are source contracts and test oracles only;
no Fabushi runtime, provider, device, package or acceptance evidence is claimed.

The script-transport, timeout-budget, snapshot-action and process-dispatch
chain was then read at the same locked tree:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `crabbox-worker-script.ts` | `3fa6163aa004fa168b3fcef4abb3ff79658f971c` | 54 | owner-audit |
| `crabbox-worker-timeouts.ts` | `2d13d497ed213dc8c24b09a86b2c1e7b97368daf` | 148 | owner-audit |
| `crabbox-worker-snapshot-actions.ts` | `78a412b39bfd4b54b7935049d4a8c3f08e8b4f57` | 83 | owner-audit |
| `crabbox-process.ts` | `0ccfe03b9db815de9ea466891396eacc76597203` | 13 | owner-audit |
| `crabbox-binary.ts` | `27f5952d24b8fd5133acd86e2fc4aeb13a7e22cc` | 58 | owner-audit |
| `crabbox-worker-script.test.ts` | `b7c3437ab1fee4d24193156c4eee9bbedee75026` | 67 | test-oracle-replaced |
| `crabbox-worker-read-budget.test.ts` | `08443061d24ec0bb3df09d3ecf08752665c5be4a` | 94 | test-oracle-replaced |
| `crabbox-worker-stop-lifetime.test.ts` | `fed558101bbf741fd293947cd60456cd0dda9e94` | 223 | test-oracle-replaced |
| `crabbox-worker-stop.test.ts` | `0fc72458a1a29593e38e8e2b8c83504c370dd4b6` | 90 | test-oracle-replaced |
| `crabbox-worker-provision-cancellation.test.ts` | `1d960cd7fb9f891d8bffb586a1f8259298c5078d` | 195 | test-oracle-replaced |
| `crabbox-binary.test.ts` | `09291b712594ad0fbb64e5652718a692b1f77f8f` | 86 | test-oracle-replaced |
| `crabbox-process.test.ts` | `d3c6721a6c3938074e4e08b55b8bb0af49010a77` | 31 | test-oracle-replaced |

The source contract fixes the guest transport and deadline semantics. Linux
and macOS use a strict POSIX heredoc; bootstrap additionally binds the selected
Node executable into a private runtime path, while Windows writes a temporary
UTF-8 PowerShell script and removes it in `finally`. Timeout composition
includes acquisition, bootstrap, lifecycle, enrollment diagnostics, child/tree
settlement, stop/release, and optional warm-image phases; Machine0 and desktop
profiles receive distinct budgets and polling cadences. Snapshot pin/rollback/
delete operations are signal-fenced and tracked until settlement, and deletion
resolves all configured provider binaries while tolerating partial discovery
but failing when none are usable. Binary discovery separates explicit/sibling/
PATH lookup from the bare dispatch fallback and preserves Windows suffix and
PATH ordering. Process dispatch resolves path-like executables before leaving
the caller directory and defaults to the host home directory.

The corresponding oracles pin Unicode transport, bootstrap Node replacement,
owner-shutdown command custody, provider read budgets, composed provision
deadlines, Azure recovery within one monotonic deadline, heartbeat custody
through destroy/dispose/inspection loss, late stop settlement and cleanup debt,
phase-by-phase cancellation without later work, binary precedence and Windows
suffix ordering, and process working-directory selection. They remain source
oracles only and have not been executed locally.

The profile/env and model-run boundary was then read at the same pinned tree:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `crabbox-worker-env-profile.ts` | `f5e5dda10371f2331d9efd7138f15634a23d7384` | 41 | owner-audit |
| `crabbox-worker-profile-suggestion.ts` | `f3adbe0f0d3ef8d118317ac0ef28ab188e6fd862` | 85 | owner-audit |
| `crabbox-model-run-cli.ts` | `5914b6490562b567ed515f4adf580897ad39d127` | 175 | owner-audit |
| `crabbox-worker-profile.test.ts` | `fe92e36ea3102b0f023229c06330194f4b05015a` | 121 | test-oracle-replaced |
| `crabbox-worker-profile-suggestion.test.ts` | `897085a0c0209c40473fba43294c1ff5276cdd16` | 116 | test-oracle-replaced |
| `crabbox-model-run.test.ts` | `24a071808a0bfe9e91ec74d3faa9b5e5bf7e6085` | 157 | test-oracle-replaced |

The environment-profile helper writes setup values only to a mode-600,
lease-scoped temporary file, rejects shell-ambiguous bytes, removes inherited
names from the child environment, passes only the explicit allow marker, and
removes the directory in `finally`. Profile suggestion is discovery-only: it
requires an installed binary, explicitly selected/authenticated provider
configuration, an advertised Linux class, and caches failures until explicit
refresh; it never installs or executes a missing CLI. The profile oracle also
binds warm-image identity to resolved disk inputs and protected secret refs,
rejects retired plaintext setupEnv and unsafe SecretRefs, and preserves the
allocation target/network/TTL contract.

The model-run CLI is a bounded foreground command path, not an agent runtime.
It validates an explicit provider/model reference and timeout, upgrades or
selects the managed Crabbox binary before credential use, checks native egress
capability, obtains host-only configured model egress, and sends only a
sentinel plus loopback proxy material into the remote command. The generated
remote script uses a temporary CA file, restricted permissions, explicit
allowlisted hosts and no-sync/no-hydrate flags; process-tree extinction and
uncertain cleanup are fail-closed. Its oracle proves managed-binary upgrade
ordering, proxy-secret non-disclosure, sentinel-only remote environment,
lookalike-flag rejection before credential access, and cleanup uncertainty
failure. These are source contracts only; no Fabushi CLI, model provider,
credential store or packaged execution acceptance is claimed.

The warm-image allocation and credential-scrub chain was then read at the same
locked tree:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `crabbox-worker-warm-image.ts` | `af4b96a5fab3adf751864b784981e01c61fb8be5` | 706 | owner-audit |
| `crabbox-worker-warm-image-scrub.ts` | `69b437fddcb2540f7c8e87562d6308bfb56a90c7` | 64 | owner-audit |
| `crabbox-worker-warm-image-policy.ts` | `6559c442a8b956a6704f52085ff5c0259d901728` | 41 | owner-audit |
| `crabbox-worker-warm-image-allocation.ts` | `27427bcb41602ad97b157f7f6e736b20b44b1221` | 187 | owner-audit |
| `crabbox-worker-warm-image-policy.test.ts` | `1a3b29958c3ddd6b78d923c32061b6ae3bfc42d3` | 74 | test-oracle-replaced |
| `crabbox-worker-warm-image-allocation.test.ts` | `89af0fe6b67238f2605fb960287d56c2e7f3c811` | 721 | test-oracle-replaced |
| `crabbox-worker-warm-image.test-support.ts` | `16bda6158d6bbb1e6a25039f4b4bb179ffd829fb` | 238 | test-oracle-replaced |

Warm-image management is a stateful provider owner, not a node-RPC or
conversation store. It uses compare-and-apply rows, generation fencing,
allocation holds and bounded capacity; freezes checkpoint-versus-cold choice
before the first provider call; verifies pending checkpoints; retains
indeterminate retirement/capture obligations; and releases a lease only after
confirmed stop or explicit absence. Pin, rollback and delete reject active,
held or pinned snapshots and remain owner-current/signal fenced. Allocation
keeps runtime identity, preparation/cache keys, project facts and cold reasons
in the same record, while the provider layer owns actual fork/warmup calls.

The scrub script is a security boundary: it verifies PID, process group,
runtime path, cwd and `OPENCLAW_STATE_DIR` ownership before terminating a node;
removes only lease/session state and SSH workspaces; retains machine-level
runtime/bundle caches; and replaces its shell before deleting uploaded scripts.
Warm-image policy accepts only bounded whole-unit durations and an explicit
binary `keepPrevious` flag. The focused oracles cover default/schema policy,
capacity and eviction, generation/replay fencing, checkpoint availability,
provider-fork failure, retention/retirement debt and cancellation ordering.
These facts constrain a future Rust owner but do not prove Fabushi state-store,
provider, filesystem-scrub or packaged acceptance.

The remaining direct Crabbox provider/UI helpers were then read at the same
locked tree:

| Path | Blob | Lines | Disposition |
| --- | --- | ---: | --- |
| `crabbox-worker-machine-options.ts` | `7c66b6ffab48d9dfca6131ceaef064fc461e3499` | 150 | owner-audit |
| `crabbox-worker-node-enrollment-diagnostics.ts` | `1d1b10d90ad31543ab70d27065767daf12050a8d` | 90 | owner-audit |
| `crabbox-worker-wallpaper.ts` | `6216fc833d6fe5969664e08c26a3fa505af49962` | 31 | owner-audit |
| `crabbox-worker-warm-image-cli.ts` | `28f1b4d2cca8bf199145de123290f680b9fd54dc` | 105 | owner-audit |
| `crabbox-worker-warm-image-cli.test.ts` | `6dacb5f3d978984ade76c2aa69cae8a6efa07bdf` | 146 | test-oracle-replaced |

Machine catalog discovery runs the provider's bounded `providers --json`
command, preserves complete catalog JSON, prefers amd64 and valid positive
resource summaries, caches successful metadata by binary, and removes failed
loads from the cache. Setup/UI projection filters incompatible targets while
preserving the configured target as the default. Enrollment diagnostics are
best-effort and separately timed: they report runtime-link, PID liveness and a
bounded UTF-8/UTF-16 log tail, redact sensitive output and preserve the
original enrollment failure when diagnostics fail. The wallpaper helper only
accepts a PNG with the exact 1024x576 IHDR dimensions.

The warm-image CLI is a read/recovery surface, not a provider execution path.
It lists local image/legacy allocation state, requires an exact selector plus
explicit provider-cleanup acknowledgement before recovery, preserves unrelated
images and retirement debt, and prints bounded next steps without running
provider commands. Its oracle covers unsupported-capture cold guidance,
legacy-row migration fencing, exact unchanged-row recovery, stale/uncertain
capture guidance and pending deletion display. These source facts do not prove
Fabushi provider catalog UI, enrollment diagnostics, desktop assets or CLI
acceptance.

## Fabushi owner resolution

The live Fabushi composition was checked before selecting a target owner. The
following exact working-tree files were read for the relevant contracts (their
blob identities are the current local target, not upstream identities):

| Fabushi path | Blob | Lines | Evidence-bearing responsibility |
| --- | --- | ---: | --- |
| `source/host/src/network_enrollment_owner.rs` | `df5492db2b5c5e42565f63595283f4ea0e96b7a5` | 716 | Fabushi credential-scoped grant issuance, durable enrollment state, generation/outcome-unknown reconciliation and owned tsnet worker lifecycle |
| `source/host/src/remote_computer_target.rs` | `b4c042cef82dd97411324daf7d3d4903fd200275` | 2713 | Remote Computer tool catalog/authorization, device secret validation, operation claims, sensitive-result vault/chunking and private transport; also the network-service adapter consumed by FDN |
| `source/host/src/host_runner_composition.rs` | `f1729d4af9a9e3b18431a06e28510252b262f7cf` | 594 | Host composition wiring; resolves turn-scoped Session/Memory/Transcript/permission/computer surfaces without owning their domain state |
| `source/host/src/runner/process_tree.rs` | `4ed09f9d43e9114f14e612d04622bb4c7baad9aa` | 207 | Generic owned-child descendant discovery and bounded TERM/KILL cleanup |
| `source/host/src/extensions/cloud_agents/cloud_agents_service.rs` | `8e5c51151abd4dc0aab35febeed30836acc7e9b4` | 1440 | Existing Sand/Cursor cloud-agent backend, background composer launch/poll/reply/cancel/archive/delete/artifact/PR projection and team policy |
| `source/electron-main/box/box-host-connector.ts` | `1c2a0c61d295dc835fc0100f8a1f9dbbb7868dd2` | 154 | Sand broker/env gateway descriptor connection, blocked/update-required handling, VNC/network-token projection and Sand credential issuance |
| `source/electron-main/box/local-docker-host-connector.ts` | `a42872f97843a68940fb3dfdef3486f6f4a38c40` | 269 | Owned local Docker Sand VM lifecycle, content-addressed host/daemon bundle staging, readiness, credential mount and runtime routing |

The ownership comparison is now concrete rather than name-based. FDN's
`NetworkEnrollmentOwner` validates Fabushi-origin credentials, persists only
its own enrollment state, reconciles grants and completion, and starts the
Fabushi network worker. It has no provider catalog, remote worker artifact
provenance, Crabbox lease/heartbeat, plugin activation, node bootstrap or
desktop-session identity. `remote_computer_target` can consume a resulting
device/network binding and already owns operation/sensitive-result fencing;
expanding it to create or supervise a Crabbox worker would violate that
boundary. `CloudAgentBackend`/`SandCloudAgentManager` already owns a distinct
background-composer service contract and uses Sand/Cursor wire types; it is not
a generic cloud-worker or OpenClaw plugin owner. The Electron box connectors
own Sand gateway/local Docker descriptors and must not become a Rust Crabbox
provider. `runner/process_tree` is a cleanup primitive, not a launch claim,
bundle admission, lease store or receipt owner. `HostRunnerComposition` is
wiring-only and explicitly delegates state to Session/Memory/Transcript.

Resulting bidirectional mapping for the audited Crabbox slice:

| Crabbox responsibility | Candidate Fabushi consumer/owner | Decision |
| --- | --- | --- |
| Fabushi account/device/network identity after a worker is admitted | `NetworkEnrollmentOwner` + `remote_computer_target` | Reuse only the resulting typed device/network contract; do not store Crabbox provider or node state there |
| Remote Computer action/result projection for an admitted device | `remote_computer_target` and Runner Computer tools | Reuse/extend at the consumer seam; Crabbox does not own Computer authorization or sensitive-result state |
| Sand gateway/local Docker box | Electron box connectors | Keep platform bridge; Crabbox sandbox backend is a separate provider-admission contract |
| Agent turn/session/transcript/permission/automation state | `HostRunnerComposition` and its existing Session/Memory/Transcript/permission owners | Consume typed worker capability only; no Crabbox shadow state |
| Crabbox catalog, binary/artifact verification, lease/heartbeat/lifetime, warm-image state, node bootstrap/process/desktop setup, plugin/doctor lifecycle | No complete existing owner | Requires a minimal source-neutral Rust owner/ADR before implementation; process cleanup may call the existing primitive but cannot transfer ownership |

| Responsibility | Existing owner | Decision | Reason / boundary |
| --- | --- | --- | --- |
| Fabushi account and device identity, device secret, enrollment grant, FDN control URL, tsnet worker lifecycle | `source/host/src/network_enrollment_owner.rs` | Reuse and extend only for Fabushi-native network enrollment | It owns the canonical Fabushi credential/device scope, persisted enrollment state, generation fencing, outcome-unknown handling, and native network worker. It is not a Crabbox provider catalog or OpenClaw Node bootstrap owner. |
| Native remote Computer operations, sensitive-result transport, grants, snapshot fencing | `source/host/src/remote_computer_target.rs` | Reuse for the resulting device capability consumer | It already owns Computer authorization and operation/result fencing. It must consume a typed native worker/device contract; it must not launch a second OpenClaw runtime or store Crabbox enrollment state. |
| Sand gateway/box connection and local Docker VM lifecycle | `source/electron-main/box/box-host-connector.ts`, `source/electron-main/box/local-docker-host-connector.ts` | Keep as platform/box bridges; do not assign Crabbox bootstrap ownership | These connectors resolve Sand gateway descriptors, local VM readiness, and box credentials. They do not own Crabbox provider catalogs, remote artifact verification, or node enrollment. |
| Crabbox provider catalog, verified worker artifact bootstrap, plugin activation, node process lifecycle and enrollment diagnostics | No existing complete Fabushi owner identified | `new_owner_proposal` pending rejected-owner analysis and ADR | The responsibility is materially broader than FDN enrollment, Computer execution, or Sand box connection. It requires a minimal source-neutral Rust owner under the existing Host/Runner/Coordinator boundaries, with platform launch remaining a thin bridge. |

The proposed owner is not permission to create an `OpenClawRuntime`, second Agent loop, second session store, or second scheduler. Its state must be a scoped worker/bootstrap lease consumed by existing Host/Runner/Computer owners; canonical Fabushi identity, conversation, transcript, permission and network state remain with their current owners.

## Required next evidence

- Read and map the remaining imported script, timeout, provider helper, reverse caller and fixture contracts at the same pinned upstream tree.
- Complete the remaining provider reverse-caller/test closure before any owner/ADR decision.
- Inspect the current Fabushi Host/Coordinator production composition and determine the narrow typed contract for provider catalog, artifact provenance, worker lease, process lifecycle, and diagnostic evidence.
- Complete rejected-owner analysis and an ADR before creating a new owner. The current `NetworkEnrollmentOwner`, `remote_computer_target`, Sand connectors, and Runner process owners are candidates but cannot be widened by assumption.
- Add positive and negative oracle rows for malformed catalogs, incompatible targets, credential non-propagation, archive hash/range/TLS failures, unsafe paths, cancellation cleanup, plugin activation failure, stale runtime pointers, diagnostic truncation/redaction, and node launch outcome-unknown.
- Run all executable checks only in GitHub Actions. Local work here is read-only investigation and dossier/spec editing; this dossier is not implementation, wiring, verification, or acceptance evidence.

## Status

This audit reduces the next frontier by establishing the exact responsibility boundary and prevents an incorrect reuse of FDN or Computer owners. It does not change any global OCRM flag, inventory read state, production wiring, or acceptance state.
