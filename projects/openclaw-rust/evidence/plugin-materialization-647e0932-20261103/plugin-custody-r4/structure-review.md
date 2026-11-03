# R4 independent structure, reuse and performance review

Verdict: **APPROVE — source-only, for the declared plugin materialization scope.**

No falsifiable blocking or non-blocking structure/reuse/performance defect was found in the complete ten-path change against the frozen base. This is not executable verification or final ACCEPT. Exact-candidate GitHub Actions execution and independent final acceptance remain pending.

## Reviewer and immutable identity

- Reviewer: `/root/plugin_custody_structure_r4`.
- Applied model: `gpt-6.1-sol`; applied effort: `xhigh`; `fork_turns="none"`. The parent confirmed the actual accepted spawn fields and that no fallback was reported. The scope includes stateful multi-owner I/O, failure classification, concurrent publication and evidence integration, so xhigh is appropriate.
- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Frozen base commit: `fdb0f6acb06165b6f2796961186e06a2de98c229`.
- Reviewed R4 tree: `77fa185818b171a5b3829fc54104a0beec9ad8b4`.
- Prior source snapshot for focused delta: `4f9c45c13151c58a2cfa784032cfdae4c0269578`.
- Manifest starting HEAD: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`; it was not substituted for the frozen review base or tree.
- Fixed upstream: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Method: direct immutable `git show TREE:path`, source-only `git grep`, and Git diffs from the frozen base/prior snapshot. `GIT_NO_LAZY_FETCH=1` was set on Git source operations. No mutable product source, real index or refs were changed. No prior review reports, review decisions or another reviewer's output were consulted. No local or htch executable validation was run.

## Authority and complete declared scope

Read the root AGENTS independent-review, immutable-snapshot, model/effort and OCRM execution rules; there is no nested AGENTS governing these ten paths. Read the applicable OCRM SOT authority/ownership rules, main specification's OCRM-EXEC-02 section 17, relevant module-map plugin/skills ownership entries, the full batch spec `plugin-materialization-attempt-custody.md`, the relevant committed-snapshot admission contract, and the full-fusion execution entry. The batch PM-01 through PM-07 and PM-O01 through PM-O08 govern this assessment. Historical discovery/status material was not treated as current acceptance.

The complete declared change was reviewed against base, including all changed hunks of each path below. Final production files, both complete final test files, the batch spec and source-contract were read directly; workflow/ledger/RTM enclosing sections and adjacent producer/consumer/build declarations were inspected where they establish the integration claims.

| Frozen path | R4 blob identity | Disposition |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `b97a3a2622620078b7654862c0b33ac47230cca6` | Reviewed all plugin changes plus actual build/test/receipt/trigger selectors; focused prior-tree delta also inspected. |
| `projects/openclaw-rust/docs/specs/plugin-materialization-attempt-custody.md` | `6252635f8a7d1abeae26bc4913e3e4fb134d9e89` | Full batch contract and limits read. |
| `projects/openclaw-rust/evidence/plugin-materialization-647e0932-20261103/source-contract.json` | `3f2bff7e4a56142690f3d5b30dc41c6472c23bce` | Full declaration read; pending execution/acceptance preserved. |
| `projects/openclaw-rust/parity-ledger.json` | `1e56a25479e1390cb98c36d130b68f09946dc0bc` | Complete base delta and enclosing existing Skills responsibility reviewed; only evidence references added. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `7475cee0643fc0e52ddc9bd2cf303aee884c0956` | Complete base delta reviewed; existing mappings extended with batch-spec references. |
| `source/packages/cursor-plugins/cursor-marketplace.ts` | `6a2433588b5f17f3609e6ca85e4aa0f2a8612e21` | Full final cache adapter and base delta read. |
| `source/packages/cursor-plugins/loader.ts` | `f0028c8fbecc602828fb6ed48dce9fd77855102c` | Full final SDK loader, base delta and R4 classifier delta read. |
| `source/packages/cursor-plugins/loader.contract.test.ts` | `fa41d87a2e9f8b03ca7d8125309a96673d57482e` | Full final test file and retained/new controls inspected. |
| `source/electron-main/mcp/host-plugin-skills-loader.ts` | `d81e1da13c9b39f4e2943ab48e8b9073bb8dc987` | Full final Host bridge and base delta read. |
| `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts` | `b8ff7fbdf1c8ff540fa4eaf1b9e798d12077d722` | Full final test file and actual control-server/backend fixture chain inspected. |

Adjacent exact-tree reads covered `backend-marketplace-client.ts` installation/list-failure producers, `marketplace-cache.ts` clone classification and file-copy boundary, `desktop/scripts/build-electron-runtime.mjs`, Electron `coordinator-control-server.ts`, Rust `extensions/mcp/plugin_skills.rs`, `extensions/mcp/coordinator_relay.rs`, `workflows/workflow_store.rs`, and Coordinator `control_port_client.rs`. The source search across all TypeScript under `source/` found no further cache-manager/materialization callers outside the cursor-plugin and Electron paths; the search excluded the separately vendored Mahayana subtree. The cache adapter's separate public extraction and explicit completion/removal/version interfaces were inspected in place.

## Findings

**None.** There is no path/scenario/fix entry because no falsifiable defect was established. Naming preferences, speculative refactoring and broader filesystem/lifecycle problems explicitly outside this batch are not promoted to findings.

## Assessment and source evidence

### Consolidation and ownership

The former inline installation block is removed from `loader.ts`. Its only production cache-miss path calls `cacheManager.materializeToCache(cacheArgs, installCallback)` at lines 143–149. The new operation is implemented once in the existing `DefaultPluginCacheManager` at `cursor-marketplace.ts:122–196`; it adds no service, installed-state index, permission store, scheduler or product lifecycle generation.

The public `PluginCacheKey` retains its existing purpose. New phase/state types and I/O helpers are private to the cache adapter. Host uses the SDK's existing `MarketplaceLoadFailure` type at `host-plugin-skills-loader.ts:102–116`, avoiding a second partial failure shape while retaining the clone-category guard. `extractToCache`, `markCacheComplete`, explicit removal and version enumeration keep their separate public roles; this repair does not create an unnecessary extraction abstraction or change an unrelated public operation to force reuse.

Host still disables both SDK prune sites with `pruneOldVersions: false` at lines 257–265. Rust still selects successful canonical materializations from actual nonempty `install_path` and absent `load_error`, owns fallback/denial reconciliation, and only prunes after its complete-pass/no-denial publication gate (`plugin_skills.rs:1028–1125`). Its two-level installed-cache traversal starts inside the cache root (`plugin_skills.rs:318–352`). Staging at `dirname(resolve(cacheRoot))` is outside that traversal and does not require Electron to duplicate Rust installed-state or pruning truth.

### Publication and failure structure

The cache inspection helper rejects target-directory and marker symlinks through `lstat`. Cache-owned operations use neutral phase/state/code messages, retaining native data in causes. The installation callback remains outside those wrappers. Marker creation uses exclusive `wx`; no final-directory recursive removal or content merge is introduced. A complete competing directory is reused before publication or after a failed rename only after the same complete-directory inspection.

The operation retains staging ownership until a successful rename and cleans only the allocated staging path. Successful publication clears that ownership before returning. A losing successful attempt still cleans its own stage; a cleanup failure changes that call into an explicit error. A primary failure is represented separately from its value, so `throw undefined` is distinguishable from no failure. Cleanup aggregates retain the primary value as `cause` and first member and the raw cleanup object as the second member (`cursor-marketplace.ts:170–193`). The implementation has no alternate inline cleanup path left in the SDK loader.

The R4 classifier follows only aggregate primary causes, uses a visited set to terminate cyclic aggregate chains, and leaves ordinary neutral cache-I/O wrappers opaque (`loader.ts:34–53`). Thus cleanup codes in the aggregate diagnostic do not become the primary SDK category, while native local permission error codes are still recognized. The full diagnostic message reaches existing observation/Host projection, and the existing Host clone-category plus canonical-ID checks continue to decide whether a failure is an auth block. There is no new Host/Rust failure protocol or second authorization classifier.

### Actual-consumer tests and receipt selection

The SDK tests call the actual loader and cache manager and assert final bytes, parsed skill content, completion markers, distinct owned stages, collisions, old-version retention and a fresh cache hit. The Host race test uses the real backend inline installer and actual Coordinator control-server dispatch with independent held requests, then checks command/manifest bytes for a zero-skill plugin and a later cache hit. These are behavior oracles over real consumers, rather than a copied publication algorithm or only call counts.

The R4 cleanup tests substitute only removal of the exact real stage via the Node filesystem binding, restore that binding in `finally`, and perform a later successful load. The SDK cases inspect original primary/cleanup object identity, nested aggregate classification, explicit undefined primary, retained staged bytes, missing completion/final success, unchanged previous bytes and absent SDK prunes. Host cases check canonical denial versus ordinary non-auth failure, residue and recovery. The native ENOTDIR residue control remains. Incoming successor-byte, filesystem EACCES/EPERM/EROFS, local-permission-negative and remote-clone-denial controls are retained once under their named wrappers. The two similar failed-publication tests are intentional retained/new named oracles, not a second runtime implementation.

The shipping build declaration bundles both contract files with the same existing Electron common configuration and dependency resolution, including Node 24. The renderer job runs those actual bundle paths with TAP and shell `pipefail`. The receipt requires exactly one successful occurrence of each of the eleven materialization controls, nonzero completed test counts, every test passing and zero fail/cancelled/skipped/TODO counts (`rust-desktop-runtime.yml:4761–4820`). The receipt binds exact commit/tree, fixed upstream, run identity, TAP digest, and changed code/spec/test hashes (`4821–4888`). Trigger path filters include all changed source/project/workflow paths. The receipt scope explicitly reserves Rust admission, real marketplace/accounts, remote cancellation, crash residues and macOS/Windows package acceptance.

### Performance and maintenance

The change adds bounded per-entry filesystem inspection and staging/publication I/O, with no dependency addition, background timer, global cache/index, unbounded retry, polling loop or long-lived lock. Error-cause traversal is linear in the aggregate chain and detects cycles. The ordinary cache-hit path remains a direct cached-path parse; it now checks the directory and sentinel using the shared strict helper. Source inspection provides no basis for a material performance regression finding; runtime measurements are not claimed.

## Limits and remaining gates

- No test, build, parser, linter, validator, generator, benchmark or harness was executed by this reviewer. Test credibility and workflow correctness here are source assessments only.
- Exact-H Actions results are not present in this review. The batch source-contract correctly keeps execution and acceptance pending; authored test cases and future workflow receipt fields are not treated as execution evidence.
- The known portable rename limit, non-cooperating filesystem writer/parent replacement, cross-device topology, stale positive results after account/uninstall retirement, lossy cache-key collisions, crashed-attempt residue discovery/GC and packaged-platform compatibility remain outside this repair's claims. They were not silently upgraded to solved.
- A broad source grep initially encountered unavailable unrelated Rust media/App blobs with lazy fetch disabled. All required plugin production/test/spec/build/consumer blobs were available; a narrowed complete TypeScript caller search succeeded. No global repository scan, full unrelated media/App review or upstream source-closure claim is made, and no additional blob was required for this verdict.

**Final state: source-only APPROVE for R4 structure/reuse/performance; execution and final ACCEPT pending.**
