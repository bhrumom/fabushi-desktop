# Complete read — .agents/shared/build-lock-recovery.md

Status: understood, mapped, toolchain-equivalent; independent verification pending  
Upstream: `telegramdesktop/tdesktop@0a11537c220df35fe59a96aebe413b1e52bc6e47`  
Path: `.agents/shared/build-lock-recovery.md`  
Blob: `ec75573e7efe74e9c9a344f3e66f2fc43ac7680f`  
Inventory order: first previously unread recursive entry

## Complete responsibility decomposition

The file is autonomous developer/build tooling documentation. It does not define Telegram product runtime, UI, protocol, storage, or end-user behavior. Its sections `Safety boundary`, `Before every build`, `Recover a lock`, and `Exhaustion` define one engineering responsibility: keep a task-owned Windows Debug build recoverable from transient output-file locks without killing unrelated processes or deleting unrelated files.

The material guarantees:
- exact source/build/executable identity before process or file action;
- serialization for builds sharing one build root;
- exact locked-artifact extraction from compiler/linker diagnostics;
- bounded recovery and retry;
- no image-name process killing, IDE/debugger termination, directory deletion, cross-checkout mutation, or guessed artifact path;
- fail-closed exhaustion when ownership cannot be proven.

## Fabushi applicability and owner audit

TDRP executable verification is forbidden on user workstations and local assistant containers. `AGENTS.md` makes GitHub Actions the sole FBCP/TDRP execution environment. Current TDRP/runtime workflows execute on job-scoped hosted runners and bind the exact target SHA before work. Therefore the upstream local Windows process-control/file-deletion mechanism is not copied. Its applicable engineering guarantee is preserved by the existing source-neutral CI execution boundary:

- `.github/workflows/tdrp-revision9-authority.yml` — `TDRP_TARGET_SHA`, exact checkout assertion, isolated hosted runner, exact-head artifacts;
- `.github/workflows/rust-desktop-runtime.yml` — `FABUSHI_TARGET_SHA`, hosted runner matrix/concurrency, exact-head runtime evidence;
- `AGENTS.md` — all TDRP executable verification only in GitHub Actions; no local Mac/Windows/container build/test.

Rejected owners:
- Host/Coordinator/Runner product runtime: the source file governs development build recovery, not Fabushi runtime lifecycle.
- A new Windows build-lock daemon/helper: it would reintroduce workstation mutation that the canonical GitHub-Actions-only policy intentionally excludes.
- A Telegram-named tooling owner: source-specific duplication is unnecessary.

No novel product owner is required. No UI, search, persistence, network, account, permission, or service contract is introduced.

## Equivalent behavior

Canonical state machine:

`exact target SHA -> fresh hosted runner/workspace -> exact checkout assertion -> execute build/test -> success artifact OR fail-closed job -> clean hosted runner on rerun`

The upstream condition “a transient lock must not authorize unrelated process/file mutation” is preserved more strongly: Fabushi does not perform local process termination or artifact deletion on a user machine at all. A failed job remains failed evidence; a rerun executes in a fresh Actions environment rather than weakening assertions or deleting guessed local state. Other independent jobs continue and are not converted to global acceptance.

## Traceability

Requirement: `TDRP-R9-BUILD-LOCK-RECOVERY-001`  
Oracle: `ORA-TDRP-BUILD-LOCK-RECOVERY-001`  
Invariant: `INV-TDRP-BUILD-EXECUTION-ISOLATION-001`

Oracle:
- TDRP verification runs only in GitHub Actions;
- the workflow binds and asserts the exact target SHA;
- evidence artifacts name/bind that target SHA;
- no Fabushi production/local helper can kill arbitrary Windows processes or delete build-tree artifacts;
- a failed executable check remains failed rather than being reclassified as pass.

Evidence already observed before this mapping commit:
- PR #122 exact-head authority run `39269513082@fdf901e4310579b3620a5a7a11638bee57b0290b`: success, 61 jobs, exact-head authority artifact `12193884897`, recursive inventory 16,186.
- PR #122 exact-head Rust desktop runtime run `39269513136@fdf901e4310579b3620a5a7a11638bee57b0290b`: success; renderer/rust-host/platform-worker/three pressure jobs passed; messaging artifact `12195032657` has `head_sha=fdf901e4310579b3620a5a7a11638bee57b0290b`.
- Merge commit `f6b916bbac6e578add9e33acb140d9864946880d` makes that rebaseline the current main authority.

This row is not marked independently verified until this mapping commit itself receives exact-head GitHub Actions evidence.
