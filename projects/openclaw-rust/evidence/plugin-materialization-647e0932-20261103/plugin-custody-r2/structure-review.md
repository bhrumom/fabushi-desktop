# Plugin materialization custody — R2 reuse, structure and performance review

**Verdict: APPROVE — source-only.** No concrete blocking or non-blocking defect was found in the declared change scope for these lenses. This approval is not a build, test, benchmark, Actions, platform or product-acceptance result.

## Review identity and constraints

- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Frozen base: `113c32276b99ed668be095e0be6ed164dffdf46d`.
- Frozen candidate tree: `c217f0f20090d75213c45602d6f333139c6f25ed`.
- Starting HEAD recorded by the manifest: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`.
- Prior tree used only to read the R2 source delta: `e9569789cc65760dc41403fefe42261b6b7a5c0f`.
- Accepted launch setting supplied by the review runner: **`gpt-6.1-sol` / `xhigh`**. This is the root `AGENTS.md` pin for the reuse, structure and performance lenses. The assignment warrants `xhigh` because filesystem publication, concurrent remote work, cleanup custody and authorization diagnostics interact. No model/effort fallback was requested or observed in this worker.
- Fresh blind review: no current-batch earlier review report or runner decision ledger was consulted. The earlier tree was source context only. Historical material embedded in an adjacent specification was not used as a current-batch verdict or arbitration input.
- Reviewed source came from immutable `git show`, `git diff` and `git grep` against the candidate tree and exact bases. No reviewed repository file was edited. This report is the only authored output.
- No build, linter, generator, schema check, test, benchmark or other executable validation was run locally, on `htch-runtime`, or in Actions by this reviewer.

The exact-tree AGENTS inventory contains the root instructions and a nested TUI instruction file outside this scope. The root independent-review, immutable-scope, owner-preservation and Actions-only rules apply. Relevant authority read includes the current custody specification, the applicable OCRM source-of-truth/owner-map and §17 execution contract, and the adjacent snapshot-admission requirements.

## Complete frozen scope

All nine declared path diffs were read against the frozen base; the change totals 1,146 insertions and 14 deletions. Production and test R2 source deltas were also read against the prior tree. The complete current-batch base diff, rather than incoming shared-file history in the prior-tree delta, determines this verdict.

| Path | Reviewed change |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | Existing renderer receipt gains nine named materialization positives, exact-one occurrence checks, changed-input hashes and a bounded scope statement. |
| `projects/openclaw-rust/docs/specs/plugin-materialization-attempt-custody.md` | Requirements PM-01–PM-06, atomicity limits, real SDK/Host oracles and explicit remaining acceptance work. |
| `projects/openclaw-rust/evidence/plugin-materialization-647e0932-20261103/source-contract.json` | Owner chain, scoped source provenance, pending exact-H execution and the nine-name receipt contract. |
| `projects/openclaw-rust/parity-ledger.json` | Two evidence references added to the existing responsibility record. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Existing evidence lists gain the custody specification reference; no second state ledger or completion upgrade. |
| `source/packages/cursor-plugins/cursor-marketplace.ts` | Attempt-local staging, complete-directory inspection/publication, owned cleanup and private neutral cache-I/O diagnostics. |
| `source/packages/cursor-plugins/loader.ts` | Existing cache-miss branch delegates to the cache adapter and consumes its final returned path. |
| `source/packages/cursor-plugins/loader.contract.test.ts` | Six real SDK/filesystem controls; prior controls retained. |
| `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts` | Real control-server/Host zero-skill custody test, cleanup-denial control and three-case cache-owned denial negative; prior controls retained. |

## Source assessment

### Placement and reuse

`DefaultPluginCacheManager.materializeToCache` belongs in the existing cache filesystem adapter. It owns only an invocation's filesystem output and its transfer to the deterministic cache path. The SDK loader continues to own its listing loop, parser invocation, per-entry failure projection and public pruning option. No persistent installation registry, account state, lifecycle generation, admission cache, scheduler or cleanup daemon is introduced.

The R2 `CacheIoPhase`, `cacheIoCode`, `cacheIoFailure` and `cacheIo` definitions are private to this adapter. They are a small boundary around the adapter's own native failures, with the same code extraction reused for cleanup. They do not add a public error protocol, dependency, generic framework or parallel authorization classifier. The actual `install(stagingDir)` call is deliberately outside that wrapper. Native cache error details stay in causes, while the existing Host classifier receives fixed phase/state text plus a constrained code. The cleanup aggregate retains the primary installer message and cause, which is necessary for the existing canonical denial behavior.

The existing `markCacheComplete` cannot simply replace the new exclusive staged marker write: its public behavior addresses the deterministic final path and uses an ordinary overwrite. `extractToCache` remains unchanged with its separate public purpose; the exact-tree caller search found no production caller requiring it to be redirected through this repair. The existing version enumeration and explicit removal/pruning APIs retain their behavior.

### Installer, path and consumer compatibility

The real backend's inline synthesizer, marketplace-cache copy, Git-source copy and release extraction/copy accept a supplied destination directory. Inspection found no requirement that the destination already have the final cache basename or hierarchy. Their existing copy operations preserve relative contents; the new cache operation adds a directory rename rather than another payload-copy pass.

The loader still parses the deterministic final path after `materializeToCache` resolves. `createHostPluginSkillsLoader` still configures `<sandRoot>/plugins/cache`, supplies `pruneOldVersions: false`, projects the actual install path, and derives `authBlocked` from failures with canonical database identities. Its account-currentness and publisher-fact handling is unchanged. The pre-existing SDK false/default/true prune controls and unavailable-versus-empty and canonical-ID assertions remain present.

The adjacent Rust sources confirm the intended boundary. `SandPluginSkillsService` remains the sole lifecycle/admission/publication owner; successful plugins with zero skills contribute their actual install path to committed retention. Its complete-pass, no-denial, replacement and pending-uninstall pruning gates are unchanged. `FileWorkflowStore` still resolves records' final file paths and checks the existing live admission reader before and after file-fact retrieval. This patch does not publish an admission decision in Electron or create an alternate consumer truth.

### Custody and concurrency

The attempt directory is created with `mkdtemp` under the absolute lexical parent of `cacheRoot`, outside the Rust pruner's two-level traversal of `cacheRoot`. The completion marker is created exclusively in staging before publication. Target and marker inspections use `lstat`, so existing target symlinks and marker symlinks cannot qualify as complete entries.

On a failed installer, marker error, target conflict, publication error or completed competing winner, cleanup addresses only this invocation's staging name. On successful rename, `ownsStaging` becomes false before return, so the transferred final directory is not removed through the stale temporary name. A losing successful publisher reuses a complete winner after inspection and removes its distinct staging. Failure of that removal becomes an explicit failure with unknown residue rather than an invented successful cleanup.

The rename-failure branch preserves the publication error and, if winner inspection itself fails, both neutral wrapped failures. The R2 fix covers native diagnostics from initial/prepublication/winner inspection, staging allocation, marker creation, final-parent creation and directory publication. The unchanged installer callback remains the source of genuine remote authorization messages. The new actual Host negative uses an auth-like data-root component and exercises initial collision, collision during install and reserved marker conflict; its paired cleanup fixture continues to preserve the real installer denial and to exclude an auth-like cleanup path from `authBlocked`.

The standard rename interface's lack of portable no-replace compare-and-swap, external parent replacement, arbitrary writers and cross-device topology are explicitly bounded in the specification. I found no implementation claim that exceeds that cooperating-publisher contract. No existing final target is recursively removed by the new materialization operation.

### Performance and residue

The warm cache path changes from one marker `stat` to bounded directory-plus-marker `lstat` calls. A cache miss performs additional bounded inspections, one exclusive temporary-directory allocation and an atomic directory rename. It does not introduce recursive scans, extra payload hashing, extra payload copying, a new dependency or a background task. The private async wrappers add a constant number of calls per filesystem phase.

Concurrent attempts can each retain a separate complete or partial plugin until they settle, so peak temporary disk usage scales with active attempts and their plugin bytes. That cost follows from the required isolation; the patch does not pretend to deduplicate active remote installations. Settled failed/losing attempts clean their own staging; process interruption can leave sibling residues. The specification and receipt explicitly leave crash-residue discovery/GC open. Adding a second persistent cleanup owner here would expand this repair's lifecycle scope. No measured latency, memory, disk bound or supported-platform performance result is claimed by this review.

### Build and evidence integration

The existing `desktop/scripts/build-electron-runtime.mjs` selector already bundles both changed contract files with shipping module resolution and Node 24 options. The new Host test invokes the shipping `createCoordinatorControlServer` and actual Host/SDK/backend inline installer under controlled remote/installation I/O; it does not copy the materialization algorithm. The existing control-server dispatch is asynchronous per request, and the Rust `call_control_command` continues to retire only its local waiter.

The workflow keeps its existing pipefail test step and full bundles. Receipt creation follows the successful step and requires each of the nine new named positives exactly once, positive TAP completion, and zero failed/cancelled/skipped/TODO counts. It binds the current exact subject/tree, upstream identity and relevant changed input bytes. The existing Rust plugin/cache/workflow selections and Coordinator timeout-retirement test remain in source. Shared workflow/ledger/RTM edits in the complete base diff only add this batch's entries; incoming media/DTR controls are not removed by these hunks.

## Limits and disposition

There are no findings requiring a repair from these lenses. The approval covers the complete frozen nine-path batch and the R2 neutral cache-I/O diagnostic correction with the adjacent callers and consumers described above.

It does not establish successful execution of the authored tests, exact candidate H Actions results, real remote marketplace/account behavior, Rust execution, crash recovery, arbitrary external writers, complete upstream installation-family closure, cross-account revocation, filesystem aliases, or macOS/Windows packaged behavior. Those remain the integrator's applicable evidence/acceptance work. No CI pass is inferred from test source, workflow selectors, the source-contract file or this approval.
