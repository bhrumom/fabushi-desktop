# Plugin materialization custody — reuse, structure, performance and contract review

Verdict: **APPROVE — source-only**. No concrete blocking finding was identified in the declared frozen scope under these lenses. This is a review of source and authored verification wiring, not a build, test, CI or product-acceptance result.

## Review identity and method

- Reviewer: fresh independent worker `/root/plugin_custody_structure_r1`.
- Applied model/effort: **`gpt-6.1-sol` / `xhigh`**, as selected for this worker. The effort is appropriate to filesystem publication, concurrent attempts, cleanup ownership, auth-classifier coupling and Rust/SDK consumer contracts. No model fallback was used or requested.
- Repository root: `/workspace/scratch/425c55809bda/fabushi-desktop`; verified by the repository's Git root read.
- Base commit: `3438eb5bac755c399c9ec686414bbb4c0816888d`.
- Starting HEAD recorded by the manifest: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`.
- Frozen tree reviewed: `e9569789cc65760dc41403fefe42261b6b7a5c0f`.
- Scope source: `manifest.json`, `changed-paths.txt`, and the complete base-to-tree Git diff. Git reports exactly the nine declared changed paths. The base exists as a commit and the review subject as a tree.
- Product/source observations were made with `git show TREE:path`, `git grep TREE`, and base-to-tree diffs. The mutable working tree was not the source subject. No other review report or review decision was read.
- Authority read: root `AGENTS.md`; the full new `plugin-materialization-attempt-custody.md` spec; the applicable OCRM source-of-truth/owner mapping and main-spec architecture, plugin, reliability and execution sections; and the adjacent Plugin Skills snapshot-admission requirements.

## Complete declared scope disposition

| Path | Review disposition |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | Reviewed the complete changed hunks and surrounding renderer build, Node test command, exact-subject binding, TAP acceptance, receipt fields and relevant existing Rust selections. The change adds positive selection and evidence; it removes no existing selector or control. |
| `projects/openclaw-rust/docs/specs/plugin-materialization-attempt-custody.md` | Read completely. PM-01 through PM-05, PM-O01 through PM-O06, atomicity limits and remaining work match the bounded I/O repair. |
| `projects/openclaw-rust/evidence/plugin-materialization-647e0932-20261103/source-contract.json` | Read completely as authored provenance/verification planning. It explicitly leaves exact-H execution and acceptance pending and distinguishes bounded upstream observations from canonical source-review completion. |
| `projects/openclaw-rust/parity-ledger.json` | Reviewed all changed content: two evidence/spec references in the existing responsibility record. No status or global completion flag changes appear in the diff. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Reviewed all changed content: eight references to the new spec alongside the existing snapshot-admission references. No requirement, acceptance criterion or result was removed or promoted. |
| `source/packages/cursor-plugins/cursor-marketplace.ts` | Read the full module and reviewed the new helper, inspection, publication, ownership and diagnostics paths against existing cache interfaces. |
| `source/packages/cursor-plugins/loader.ts` | Read the module and reviewed the actual listing, cache-hit, materialization, parser, per-entry failure and two pruning call sites. |
| `source/packages/cursor-plugins/loader.contract.test.ts` | Read the existing controls and new tests. Existing default/false/true prune, unavailable/empty, identity and partial-pass controls are retained; fixture output is now under an exclusive parent that also contains staging. |
| `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts` | Read the full test source and new concurrency/cleanup tests. Existing listing, canonical identity, zero-skill, account-currentness and auth controls remain. |

## Substantive review conclusions

### Placement, reuse and ownership

`DefaultPluginCacheManager.materializeToCache` (`cursor-marketplace.ts:102–165`) belongs in the existing filesystem cache adapter. It allocates attempt-local storage, calls the supplied installer, marks complete output, publishes it, and cleans only still-owned staging. It does not retain an installed-plugin list, authorization decision, lifecycle generation, promise registry, timer or scheduler. The only new imports are Node standard-library filesystem/path operations; no dependency manifest changes occur.

The existing `MarketplaceCacheManager` is not a suitable shared publication owner: its private `moveToCanonicalDir` removes the current clone directory before rename (`marketplace-cache.ts:157–165`), and its staging discovery/GC operates in the marketplace-clone cache. Reusing it would introduce destructive replacement or a second retention mechanism into the installed-plugin path. The new operation does not copy that algorithm, its global in-flight registry or its cleanup sweep. The separate public extraction, version enumeration, explicit removal, pruning and completion methods retain their previous purposes. Exact-tree caller search found no production `extractToCache` call.

Rust remains the lifecycle owner. `SandPluginSkillsService::run_pass` retains canonical listed/successful/denied/fallback reconciliation, snapshot publication, complete-pass/no-denial pruning gates and actual install-path retention (`plugin_skills.rs:993–1130`). `prune_committed_if_current` still checks successor state, uninstall observation and pending uninstall custody before filesystem pruning (`862–900`). The new staging path is outside the two-level subtree enumerated by `prune_uninstalled_plugin_dirs` (`323–357`); it does not become a Rust cache entry.

### Actual producer and final-path compatibility

The actual production chain remains `createHostPluginSkillsLoader` → `loadFromMarketplaceSource` → `BackendMarketplaceClient.installPlugin`. Host still supplies `pruneOldVersions: false` and returns the SDK's `installPath` unchanged (`host-plugin-skills-loader.ts:258–277,303–328`). Both SDK prune guards remain intact (`loader.ts:132,138`).

The backend installer consumes the directory argument rather than deriving its own final cache path: inline synthesis writes relative component paths and manifest entries, marketplace installs call `copyPluginToDir`, direct Git installs copy materialized source to the argument, and release installs extract/copy into it (`backend-marketplace-client.ts:179–218,293–317`; `inline-plugin-synthesizer.ts:28–73`; `marketplace-cache.ts:279–280`). Content parsing and path expansion occur after `materializeToCache` returns the original deterministic final path (`loader.ts:129–132`). Moving that argument to staging therefore does not expose staging paths in the loader result or the normal manifest/component projection.

The real downstream file consumer remains compatible: skill records join the returned final `install_path` with the relative skill path (`plugin_skills.rs:223–281`), and `FileWorkflowStore` reads the indexed path and checks live denial before and after its parse-cache lookup (`workflow_store.rs:293–397`). Zero-skill plugins continue to retain their actual final install path independently of skill records (`plugin_skills.rs:1110–1129`).

### Publication and cleanup cost

The miss path adds a unique directory allocation, bounded path inspections, a marker creation and a same-filesystem directory rename. It does not add a full-tree copy, directory scan, hashing pass, refresh loop or retry campaign. Backend clone/extract/copy costs remain in their existing owners. Cache hits add a directory `lstat` to the existing marker check; there is no cache-wide work. Concurrent same-key misses can each perform the already-admitted installation, with losing staging removed after observing a completed winner. This bounded repair does not add admission coalescing or another concurrency owner.

Successful rename ends staging ownership before return. Failed and losing invocations delete only their own `mkdtemp` path. A completed target is reused only after directory and regular-marker observations; existing incomplete, non-directory and symlink collisions are preserved. The final key derivation remains unchanged. These choices directly address the late-failed-attempt deletion scenario without adding version GC or changing Rust retirement.

Crash/interruption can leave staging siblings that Rust does not collect. This is an explicit retention limit in the new spec, not an undocumented cleanup sweep or a claim of crash recovery. No numerical performance, disk-growth or cross-platform result is inferred from this source review.

### Error-classifier and verification coupling

The cleanup aggregate keeps the primary error message first and retains the original primary as `cause` plus both errors in `AggregateError.errors` (`cursor-marketplace.ts:145–161`). Only a controlled `E...` code enters the neutral cleanup suffix. Arbitrary cleanup error messages and paths do not enter the string used by `authBlocksFromFailures` and `classifyCloneError` (`host-plugin-skills-loader.ts:102–131`; `marketplace-cache.ts:37–54`). The paired authored Host cleanup fixture exercises both preservation of a canonical denial and the absence of a new denial when the cleanup path itself contains auth-like text.

The authored oracles operate through the real SDK/cache manager and real Host/backend inline installer. They check final bytes, marker bytes, final paths, losing-stage disappearance and subsequent fresh cache hits, including a command/manifest-only plugin. Held barriers use bounded Node test timeouts rather than sleeps. The Host concurrent test uses the actual `createCoordinatorControlServer` asynchronous dispatch; the existing Rust timeout-retirement contract independently demonstrates that local waiter retirement does not cancel remote work (`coordinator_contract.rs:253–289`; `control_port_client.rs:46–98`).

The existing build helper already bundles both changed test entries with the shipping `common` options and `node24` target (`desktop/scripts/build-electron-runtime.mjs:19–36,89–110`). The workflow executes those bundles with `set -euo pipefail`, then requires exactly one successful occurrence of each new named materialization test and zero failed/cancelled/skipped/TODO tests (`rust-desktop-runtime.yml:4641–4697`). Existing Rust plugin/cache/workflow, sync/uninstall and Session admission selections remain present. The source contract and generated receipt wording preserve the distinction between Node edge evidence, Rust evidence and final ecosystem/platform acceptance.

## Findings and execution limits

**Findings: none.** No preference-only change is requested, and this review does not require a fix round.

No local or htch-runtime build, test, lint, parser/schema validation, generator, benchmark or acceptance execution was performed. Git/source reads were the only investigation instruments. No repository source file was edited. This report is the only authored output of the worker.

Exact candidate-H GitHub Actions execution is still required for the new named tests and applicable retained controls. Real marketplace/account behavior, process-crash residue recovery, arbitrary non-cooperating filesystem writers, parent/symlink replacement, cache-key identity collisions, late authorization/uninstall success, and supported macOS/Windows packaging remain outside the evidence established here, as declared by the spec. This source-only approval does not mark those limits passed and does not claim CI success.
