# Independent general / permission / lifetime / security / oracle review

**Verdict: FINDINGS — changes requested.** One P2 correctness finding. The attempt-local publication repair otherwise has no established blocking defect from this source review.

## Review identity and method

- Reviewer: `/root/plugin_custody_general_r1`, fresh and blind to other review reports and arbitration decisions.
- Model/effort: configured parent **GPT-6 Astra Pro**, inherited **xhigh**, no model override. The launch handoff selected xhigh for concurrent filesystem publication and authorization-denial propagation; no independent runtime model/effort introspection is exposed to this reviewer.
- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Frozen base: `3438eb5bac755c399c9ec686414bbb4c0816888d`.
- Starting HEAD recorded by the manifest: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`.
- Frozen snapshot tree: `e9569789cc65760dc41403fefe42261b6b7a5c0f`.
- Fixed upstream recorded by the contract: `1440bb1771372334bea23d7642bb52deb7db1845`.
- Read root `AGENTS.md`, the complete new custody specification, applicable OCRM authority/ownership sections, and the existing snapshot/admission requirements. Read the complete nine-path change and relevant frozen source with `git show TREE:path`; checked all nine blob identities against `manifest.json` with `git ls-tree`. No mutable worktree source was used.
- **No builds, tests, linters, generators, parsers, validators, or local/htch executable validation ran. No CI pass is claimed.** No reviewed repository file was edited.

Complete declared scope:

1. `.github/workflows/rust-desktop-runtime.yml`
2. `projects/openclaw-rust/docs/specs/plugin-materialization-attempt-custody.md`
3. `projects/openclaw-rust/evidence/plugin-materialization-647e0932-20261103/source-contract.json`
4. `projects/openclaw-rust/parity-ledger.json`
5. `projects/openclaw-rust/requirements-traceability-matrix.json`
6. `source/packages/cursor-plugins/cursor-marketplace.ts`
7. `source/packages/cursor-plugins/loader.ts`
8. `source/packages/cursor-plugins/loader.contract.test.ts`
9. `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts`

## G1 — P2: local cache-I/O paths can manufacture a canonical authorization denial

**Affected changed symbol:** `DefaultPluginCacheManager.materializeToCache`, `source/packages/cursor-plugins/cursor-marketplace.ts:107–133`, especially the explicit conflict messages at `:109` and `:125` and the exclusive marker write at `:121`.

Both new collision errors interpolate the absolute `cacheDir` into their message. That message is an authorization-classification input, not merely a diagnostic:

- `source/packages/cursor-plugins/loader.ts:133–135` preserves it as `failure.errorMessage` and the failed plugin's `loadError`.
- `source/electron-main/mcp/host-plugin-skills-loader.ts:116–128` calls `classifyCloneError(failure.errorMessage)` and returns a canonical `authBlocked` entry for `user_git_access`.
- `source/packages/cursor-plugins/marketplace-cache.ts:37,49–54` classifies any occurrence of `permission denied` (among other strings) as `user_git_access`.
- The real production data-root resolver accepts an arbitrary absolute `SAND_DATA_ROOT` unchanged (`source/shared/node/sand-data-root.ts:41–58`). The new Host cleanup fixture already exercises a root component named `permission denied`.

**Concrete triggering scenario:** use a supported data root such as `/owned/permission denied`; retain an already committed eligible old version of canonical plugin `41`; place an incomplete directory at that plugin's newly listed version path. The SDK observes the incomplete target, does not call the backend installer, and throws `Plugin cache target is incomplete: /owned/permission denied/...`. The Host then returns `authBlocked: [{ pluginId: "41", ... }]` even though only a local cache collision occurred. The same problem occurs when the conflicting target appears during installation and reaches line 125.

**Consequence:** Rust imports this false block into live admission (`plugin_skills.rs:757–789`) and excludes the prior eligible fallback from its committed snapshot (`plugin_skills.rs:1018–1019,1047–1078`). The real `FileWorkflowStore` consequently refuses the old skill. A directory-name string has acquired authorization meaning. The neutral cleanup suffix at lines 146–158 does not fix this path because it preserves the already contaminated primary conflict message; cleanup need not fail for the bug to occur.

**Same-cause newly authored filesystem branches:** removing only the two explicit `cacheDir` strings is insufficient. The new helper passes its own raw filesystem errors through the same `normalized.message` route:

| Operation | Frozen lines | Change-specific consequence |
| --- | --- | --- |
| Strict initial, prepublication and failed-rename target inspection | 107, 123, 132; helper 85–99 | New materialization admission/publication checks propagate raw `lstat` errors; `isCached` alone remains fail-soft. |
| Attempt-parent creation and unique allocation | 114–115 | New staging filesystem operations propagate their raw errors. |
| Exclusive staged sentinel creation | 121 | A reserved marker now correctly fails with `EEXIST`, whose raw path can supply the auth phrase; the former non-exclusive final marker write would overwrite a regular marker. This is a second concrete new trigger under the same auth-like data-root scenario. |
| Final-parent creation and directory publication | 126, 130–133 | Parent creation moved after installation and directory rename is new; publication errors are rethrown raw when a complete winner is not observed. |

Analogous raw errors were possible in the former direct mkdir/marker block. This finding concerns the new helper's generated conflicts and newly owned phases; it does not require a general MCP error-protocol redesign or reclassification of backend-install exceptions.

**Specific fix:** make errors from these cache-manager-owned I/O phases neutral and independent of filesystem path/native diagnostic text, exposing controlled operation/state/code and retaining the original error as cause or structured diagnostics. Preserve the original backend callback error at line 119, so real remote authorization denials still reach the existing classifiers, including when cleanup also fails. Preserve collision refusal and foreign bytes. Add actual Host-loader controls under an auth-like data-root path for an initial/during-install incomplete target and a reserved staged marker; assert per-entry failure and canonical ID, unchanged protected bytes, empty install path, and `authBlocked: []`. Keep the real-install-denial plus cleanup-failure positive unchanged. This requires no new permission owner or cache schema.

## Other source conclusions

- **Custody and publication:** unique `mkdtemp` staging is a sibling of the configured cache root. The real backend inline, marketplace-copy, direct-Git-copy, and release-copy routes receive this staging path. The marker uses exclusive creation, completed publication uses rename, and successful rename clears staging ownership. Failure/loser cleanup targets only the owned staging name. Existing final targets and marker symlinks are inspected with `lstat`; the documented arbitrary-writer/empty-directory rename limitation is not represented as solved.
- **Lifecycle boundary:** the production Electron adapter calls the real Host loader; Rust composition still binds `CoordinatorPluginSkillsLoader` to the existing `SandPluginSkillsService`, with Session and `FileWorkflowStore` using its admission reader. The Rust two-level installed-cache pruner cannot traverse the staging sibling. Complete-pass, denial, actual-install-path/zero-skill retention, pending-uninstall, and successor publication gates remain unchanged. No second lifecycle/admission owner was added.
- **Primary failure plus cleanup failure:** for a genuine install denial, `AggregateError` retains the primary message/cause and both errors; the cleanup message exposes only a controlled code and unknown residue. The paired real Host fixture verifies denial preservation and prevents its cleanup path from adding a denial to an ordinary install error. G1 concerns the separate new primary collision messages.
- **Regression oracles:** the SDK and Host tests hold A, complete B, then fail A and inspect winner file/marker bytes plus a fresh cache hit. The Host test uses the actual control server and backend inline synthesizer and covers a zero-skill command/manifest plugin. Completed-winner, target/sentinel symlink, incomplete-target, isolated cleanup, marker, and publication-failure controls operate on real filesystem paths. Existing default/false/true pruning, listing/currentness, canonical-ID and zero-skill assertions are retained; fixture roots were adjusted to own their staging siblings.
- **Execution/receipts:** the existing shipping esbuild selector includes both changed test files with Node 24 options. The workflow runs those bundles under `pipefail`, requires each of the eight new named positives exactly once, and rejects failed/cancelled/skipped/TODO TAP totals before writing its receipt. It binds H/tree/U and the changed source/spec/test inputs. Existing adjacent Rust/Coordinator selections are retained. Ledger/RTM changes add evidence pointers without advancing completion states.

## Limits

These are source conclusions, not execution evidence or final acceptance. Exact-candidate GitHub Actions results remain required. This review does not independently reperform the historical fixed-U source reads recorded in `source-contract.json`, prove real marketplace/account behavior, arbitrary concurrent filesystem writers, process-crash residue recovery, cross-device publication, cross-process behavior, or macOS/Windows packaged acceptance. The new spec and receipt explicitly retain those limits.
