# Plugin materialization custody — R3 reuse, structure and performance review

**Verdict: APPROVED for the source-only reuse, structure and performance review.** No actionable finding was established in the complete frozen scope or the R3 integration delta. Exact final-H GitHub Actions evidence and independent acceptance remain outstanding; this verdict is not an execution or release approval.

## Review identity and method

- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Review base: `54b32b96ff827f4fee56fed494c866ca8c8b53ec`.
- Candidate tree: `4f9c45c13151c58a2cfa784032cfdae4c0269578`.
- Starting implementation HEAD recorded by the scope: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`.
- Prior review tree used by the supplied fix delta: `c217f0f20090d75213c45602d6f333139c6f25ed`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`.
- Applied worker model/effort: **`gpt-6.1-sol` / `xhigh`**, as configured by the native review-runner launch. No fallback was reported. This follows the root `AGENTS.md` policy pin for these three lenses. The complete assignment warrants xhigh because it includes concurrent publication, ownership transfer, cleanup failure, error classification and SDK/Host/Rust consumer boundaries.
- Independence: fresh source review; no earlier or current reviewer report or arbitration decision was read.
- Execution restriction: no tests, builds, lint, generator, schema checker, parser, validator, benchmark or harness was executed locally or on htch-runtime. Only read-only source/Git inspection was performed. The only written artifact is this report.

I read the applicable root instructions and relevant OpenClaw source-of-truth authority, main specification §17, module-map boundaries M16/M17/M18, the candidate custody specification and the adjacent committed-snapshot specification. The exact tree contains no nested `AGENTS.md` governing these changed paths. I inspected the complete change set through the supplied patch/hunk scope and direct Git comparisons, read the production changes and tests from the immutable tree, and inspected adjacent real producers and consumers. `git diff --name-status` yields exactly the ten manifest paths, and `git ls-tree` agrees with every candidate blob ID in the manifest.

## Complete-scope disposition

| Path | Review result |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | Existing renderer build/test/receipt path is extended; no second workflow or runner is introduced. The nine custody positives must each occur exactly once, total TAP completion must have no failed/cancelled/skipped/TODO tests, and the receipt records the exact commit/tree, fixed-U inputs and relevant source/spec/test hashes. |
| `projects/openclaw-rust/docs/specs/plugin-materialization-attempt-custody.md` | The materialization I/O boundary, complete-winner reuse, target preservation, neutral errors, cleanup ownership and atomicity limits are explicit. Incoming staging and clone-guard integration are described without expanding Electron into a lifecycle owner. |
| `projects/openclaw-rust/evidence/plugin-materialization-647e0932-20261103/source-contract.json` | Records the existing producer/owner/consumer chain, bounded upstream observations, required positives and pending exact-H execution. It does not promote full-unit verification or global completion. |
| `projects/openclaw-rust/parity-ledger.json` | Adds references to this bounded contract/evidence on the existing responsibility; no new ledger or completion truth. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Adds the bounded specification to existing requirement rows; no acceptance/status promotion in the diff. |
| `source/packages/cursor-plugins/cursor-marketplace.ts` | Existing cache adapter owns the single new materialization implementation and shares its directory/marker inspection with `isCached`. |
| `source/packages/cursor-plugins/loader.ts` | Delegates installation to the cache adapter and retains listing, parser, canonical identity, per-entry failure and pruning behavior. Superseded inline staging code/imports are removed. |
| `source/packages/cursor-plugins/loader.contract.test.ts` | New tests use the actual loader/cache adapter and real filesystem bytes. Incoming older-failure/successor coverage is retained. |
| `source/electron-main/mcp/host-plugin-skills-loader.ts` | Reuses the actual SDK `MarketplaceLoadFailure` type, retaining the incoming clone-category guard and existing canonical-ID projection. |
| `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts` | Real Host/SDK/control-server tests add zero-skill winner and cache-hit coverage, cleanup double-failure coverage, neutral cache-error controls and explicit clone-denial/canonical-ID coverage. The local-permission negative is preserved. |

## Reuse and ownership assessment

The placement is appropriate. `DefaultPluginCacheManager.materializeToCache` (`cursor-marketplace.ts:122`) extends the existing filesystem/cache-key adapter. The loader now delegates through it (`loader.ts:125–130`) instead of retaining the incoming separate mkdir/mkdtemp/install/marker/rename/cleanup sequence. No new dependency, global in-flight registry, installation index, generation store, lock service or scheduler is added.

The shared `cacheDirectoryState` helper (`cursor-marketplace.ts:106`) supplies both the fail-soft `isCached` surface and the fail-closed materialization path. It uses `lstat` for the target and reserved marker, so the changed cache-hit and publisher paths agree on complete-directory evidence and reject target/marker symlinks. The separate public `extractToCache`, explicit removal, completion marking and version enumeration interfaces retain their existing purposes. Exact-source caller searches found no production extraction caller that requires changing that separate interface for this repair.

I checked the actual backend entrypoints, rather than assuming the installation callback tolerates staging. `BackendMarketplaceClient.installPlugin` (`backend-marketplace-client.ts:293`) forwards its target to inline synthesis, marketplace-cache copying, direct Git installation and release installation. Those paths populate an existing directory. `synthesizeInlinePluginDir` creates files beneath the supplied target, and `MarketplaceCacheManager.copyPluginToDir` (`marketplace-cache.ts:280`) copies beneath it. The cache adapter therefore changes the destination custody without duplicating backend download or synthesis behavior.

Product ownership stays in Rust. The Host still passes `pruneOldVersions: false` and projects the actual returned final `installPath` (`host-plugin-skills-loader.ts:257–267`). Rust still validates the loaded snapshot, derives successful canonical IDs, reconciles denials/fallback, publishes through the existing snapshot store and admits workflow records through the existing consumer. The TS additions do not decide listed eligibility, release uninstall custody or publish a second lifecycle snapshot.

## Publication, cleanup and integration assessment

The new temporary directory is allocated with `mkdtemp` under `dirname(resolve(cacheRoot))` (`cursor-marketplace.ts:133–136`). For the shipping Host cache root, this is a sibling under the existing plugins parent. I checked `prune_uninstalled_plugin_dirs` (`plugin_skills.rs:323–357`): it traverses only `cache_root/<slug>/<plugin>` and removes selected plugin directories. It cannot traverse this sibling attempt. This resolves the structural interaction that would remain if staging stayed beneath a plugin directory that the Rust pruner owns.

Installation and exclusive reserved-marker creation precede final publication. Both the initial and prepublication inspections preserve incomplete, non-directory and nonregular-marker targets; the publication code neither merges into nor recursively removes a final directory. After a rename failure it checks for a complete winner and reuses only that state. The operation uses rename directly and does not add a cross-device copying fallback. The standard rename/no-replace limitation for a noncooperating writer inserting an empty directory in the final inspection window is explicitly bounded by the specification.

The `ownsStaging` transition occurs immediately after a successful rename (`cursor-marketplace.ts:167–168`). The `finally` branch consequently does not remove the old temporary name after transfer. A failed attempt or a successful loser cleans only the exact allocated staging path. Cleanup failure remains a failure, including when the main operation had selected an existing complete winner. When a primary failure exists, the aggregate retains that failure and cause while adding neutral cleanup text. The full raw cleanup error remains structured data; it is not appended to the classifier-facing message.

Cache-owned diagnostics use fixed phase/state text and a controlled code (`cursor-marketplace.ts:26–40`), while installation callback failures retain their existing message. The Host consumes the actual SDK failure type and preserves the base guard requiring `errorType === "clone"` before interpreting a remote Git denial (`host-plugin-skills-loader.ts:103–116`). The R3 denial tests now supply an explicit Git clone error, so they exercise that real producer contract; the incoming bare local-permission negative remains separate coverage. The cleanup pair checks both a genuine clone denial and an ordinary install error under an auth-like filesystem path.

I also inspected the real surrounding lifetime/store boundaries: `call_control_command` retires its local waiter on exit without cancelling remote work (`control_port_client.rs:45–98`), and the Electron control server dispatches each executor asynchronously (`coordinator-control-server.ts:77–109`). Rust `publish_snapshot_if_active`, `prune_committed_if_current` and the complete-pass gate remain unchanged. The service retains actual successful install paths, including zero-skill plugins (`plugin_skills.rs:1108–1132`); partial materialization does not authorize pruning. `PluginSkillsSnapshot.eligible_skills` and `FileWorkflowStore.plugin_skill_to_workflow` still own committed and live admission. Nothing in this patch bypasses those consumers.

## Performance and evidence assessment

There is no introduced polling loop, timer, persistent worker, full-cache scan, global serialization or second file-tree copy. A normal cache hit adds a target `lstat` to the former marker-only inspection so directory and marker evidence can be checked without following symlinks. A miss adds bounded target inspections and parent-directory operations around the existing installation work. Each concurrent attempt may still perform its own installation; the change does not claim or introduce coalescing. Cleanup traverses only its own staged output. These source changes do not establish a material performance regression requiring a new benchmark gate for this bounded repair.

The authored tests pin actual observable behavior: distinguishable winning bytes, the completion sentinel, absence of losing-only files, disappearance of owned staging, preservation of old/unrelated bytes and a fresh load that consumes the winner without reinstalling. The Host zero-skill test also traverses the actual control server and actual inline backend, rather than inferring success from skill counts. Existing omitted/true/false pruning cases remain present. The older incoming late-failure test remains in the complete candidate, alongside the stronger new custody oracle.

The workflow reuses the shipping Electron build options in `desktop/scripts/build-electron-runtime.mjs:91–115`; the selected Node tests are compiled through the existing entries. The receipt additions at `.github/workflows/rust-desktop-runtime.yml:4774–4841` require the named new positives and bind the changed inputs. Existing Rust plugin/cache/workflow and Coordinator selections are retained. Incoming prior-root workflow changes seen in the R3 fix delta are already present in the integrated review base and are not additional plugin changes in the complete base-to-tree scope.

No passing execution is inferred from this source review. The final candidate must still obtain its own Actions build/test logs and exact-subject receipt, including the applicable adjacent Rust controls. Real marketplace/accounts, arbitrary external writers, crash-residue collection, post-response revocation and supported-platform package acceptance remain outside the bounded source approval and explicitly open.

## Findings

**None.** No severity-bearing defect or nonblocking cleanup request is raised by these lenses. Source-only approval applies to the exact base/tree and ten-path scope above.
