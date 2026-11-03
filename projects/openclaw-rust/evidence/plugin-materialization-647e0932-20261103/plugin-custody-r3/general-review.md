# Plugin materialization custody — R3 independent general review

## Verdict

**CHANGES REQUESTED — one actionable source finding (P1).**

The candidate preserves the incoming Host clone-category guard and consolidates attempt-local materialization into the existing cache adapter. Its principal filesystem custody paths are sound within the documented cooperating-publisher boundary. However, a cleanup error with code `ETIMEDOUT` changes the SDK classification of a genuine primary Git authorization denial from `clone` to `timeout`. The preserved Host guard then drops the canonical authorization block. This violates PM-04 and PM-06 and can preserve a previously eligible skill after the new remote denial.

This is a source-only verdict. No tests, builds, lint, generators, schema checks, parsers, harnesses or other executable validation were run locally or on htch-runtime. Exact final-H GitHub Actions evidence remains pending; this report supplies no execution or product-acceptance claim.

## Review identity and applied settings

| Field | Exact value |
| --- | --- |
| Repository root | `/workspace/scratch/425c55809bda/fabushi-desktop` |
| Frozen base commit | `54b32b96ff827f4fee56fed494c866ca8c8b53ec` |
| Base tree | `9110796b7edf4a971f6ffc9b7214a8f7043b4f29` |
| Starting HEAD | `0e845ce1679e0cdf95346adc54f5d58bf06464e8` |
| Reviewed immutable candidate tree | `4f9c45c13151c58a2cfa784032cfdae4c0269578` |
| Prior snapshot used only as the source-delta base | `c217f0f20090d75213c45602d6f333139c6f25ed` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed-U tree recorded by candidate lock | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Review lens | General correctness, lifetime, security/permission and oracle integration |
| Model and effort | Configured parent GPT-6 Astra Pro and `xhigh`, inherited by this fresh isolated child (`fork_turns="none"`); no model/effort override or fallback |
| Effort reason | Concurrent directory publication, exact cleanup custody, permission-sensitive error propagation, Rust admission/pruning and receipt validity require interacting-subsystem review |
| Independence | Fresh blind review. No current/prior review reports or arbitration ledgers were read. |

Inputs were the supplied manifest, complete patch and R3 fix delta under `/workspace/scratch/425c55809bda/reviews/plugin-custody-r3/`, followed by direct `git show`/`git grep` reads from the candidate tree and exact base. `git diff --name-status --find-renames` reports precisely the ten declared paths. `git ls-tree` matches every candidate blob in the manifest. The mutable working tree was not used as product-source authority and was not edited.

## Finding G-R3-01 — Cleanup timeout diagnostics suppress a real authorization denial

**Severity: P1 — permission/admission correctness.**

**Primary changed location:** `source/packages/cursor-plugins/cursor-marketplace.ts`, `materializeToCache`, line 181 and the aggregate message at line 188. `cacheIoCode` at line 27 admits and returns `ETIMEDOUT` verbatim.

**Affected consumer chain:**

- `source/packages/cursor-plugins/loader.ts`, `classifyLoadError`, line 34; per-entry classification at line 135.
- `source/electron-main/mcp/host-plugin-skills-loader.ts`, `authBlocksFromFailures`, line 116.
- `source/host/src/extensions/mcp/plugin_skills.rs`, `run_pass`, especially the successful-ID predicate at line 1042 and prior-record fallback at line 1073.

All source paths in this report are relative to the exact repository root above; line references refer to candidate tree `4f9c45c13151c58a2cfa784032cfdae4c0269578`.

### Concrete trigger and result

1. The raw authoritative listing still contains canonical plugin ID `41`. A prior committed, eligible skill for `41` exists, without a prior denial.
2. The current version is a cache miss. The actual install callback rejects with the primary message `git clone failed: Permission denied (publickey).` This message alone is classified as `clone`, and the Host maps it to the canonical auth block for `41`.
3. Removing this attempt's staging directory also fails with native I/O code `ETIMEDOUT`, such as a filesystem I/O timeout. The new code accepts that code under `/^E[A-Z0-9]+$/` and forms this public aggregate message:

   ```text
   git clone failed: Permission denied (publickey).
   Plugin cache staging cleanup failed; residual state unknown (I/O code: ETIMEDOUT).
   ```

4. `classifyLoadError` searches the entire message and checks `/timed?\s*out/i` before `/clone|fetch|git/i`. The substring `TIMEDOUT` inside `ETIMEDOUT` satisfies the first expression, including its zero-whitespace case. Consequently, the per-entry failure becomes `errorType: "timeout"`.
5. The preserved Host guard requires `failure.errorType === "clone"` before evaluating the auth-message classifier, so `authBlocked` omits `41` despite the unchanged primary denial text. The error entry has no successful install path, while `41` remains listed.
6. Rust therefore has no current denial to import, no fresh successful ID for `41`, and retains the old eligible record through its existing listed-but-unsuccessful fallback. Its real workflow consumer continues to admit that record. The cleanup error has changed permission meaning rather than merely adding diagnostics.

This is introduced by the candidate relative to the frozen base. The inline staging implementation at base `54b32b96ff827f4fee56fed494c866ca8c8b53ec`, `loader.ts` line 126, rethrows the original installer error after its cleanup attempt. It does not append the cleanup code to the message subsequently classified by the SDK. The candidate's more explicit cleanup reporting must preserve that primary classification.

The finding is derived directly from the accepted code alphabet, literal output string, ordered regex branches, Host guard and Rust fallback. It is not an executed reproduction, and it does not claim that every local filesystem produces this error. Its precise triggering condition is an owned cleanup rejection whose code is `ETIMEDOUT`.

### Required repair and regression oracle

Keep the primary installer error's category stable when adding cleanup diagnostics. Preserve the Host clone guard, original primary message/cause, explicit cleanup-failure status and full native cleanup error in `AggregateError.errors`/cause. Make the appended diagnostic text incapable of changing the SDK's primary classifier result. A narrow cache-adapter repair can map classifier-sensitive display codes such as `ETIMEDOUT` to the already-supported neutral `unknown` token while retaining the exact native code in the cleanup error; this avoids changing the public Host/Rust protocol or remote-denial policy.

Add a deterministic controlled-cleanup-error case with `code: "ETIMEDOUT"`. Through the actual SDK and Host consumers, require that the primary Git denial still produces SDK `errorType: "clone"` and `authBlocked: [{ pluginId: "41", ... }]`, while cleanup failure remains observable and no successful install path is reported. Retain the existing real-filesystem `ENOTDIR` residue oracle and the paired non-auth error control.

The existing test at `host-plugin-skills-loader.contract.test.ts` line 422 forces `ENOTDIR` only. Its new explicit Git primary is correct, but that code does not intersect the earlier timeout classifier and cannot expose this failure. The same gap applies to the named passing-test receipt: it can prove the current test passed, not that every displayed cleanup code preserves primary error meaning.

## Complete scope disposition

| Declared path | Verified candidate blob | Review disposition |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `4390b005bc02152fb332d97630327b73ded50c36` | Reviewed materialization selectors, exact-subject binding, build/test ordering, TAP checks, input hashes, receipt scope and retained adjacent native selections. |
| `projects/openclaw-rust/docs/specs/plugin-materialization-attempt-custody.md` | `6e9ea946688f1b6efd44251ea5220001d4582a98` | Read complete task contract, six requirements, seven named oracle groups plus retained controls, atomicity limit and R3 preservation clause. |
| `projects/openclaw-rust/evidence/plugin-materialization-647e0932-20261103/source-contract.json` | `c856e08b147e595b91da8b4a4ed4dc9ec2a372b8` | Reviewed identity, owner chain, bounded source observations, requested tests and explicit pending execution/acceptance. No upstream reading states were independently promoted. |
| `projects/openclaw-rust/parity-ledger.json` | `d3e99f9b1c5ef77755d394110700f07de9c35ed8` | Diff adds the task spec and source-contract references to the existing responsibility; no completion-state promotion. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `3722bc58b292c84c7157e09e2f95dd89b59f6765` | Eight scoped spec-reference additions; no requirement deletion, acceptance relaxation or new owner. |
| `source/packages/cursor-plugins/cursor-marketplace.ts` | `6a2433588b5f17f3609e6ca85e4aa0f2a8612e21` | Deep review of inspection, staging, marker, publication, losing-publisher cleanup and existing cache interfaces. G-R3-01 applies. |
| `source/packages/cursor-plugins/loader.ts` | `0d9ead685ecde665d3c5041a5c962e5909c1b443` | Deep review of the replacement materialization call, parser consumption, failure projection and unchanged two prune sites. G-R3-01 reaches this classifier. |
| `source/packages/cursor-plugins/loader.contract.test.ts` | `45f17992d5d5f7f01e75f0a892704e1fadab3a89` | Read all actual SDK controls, including preserved incoming late-failure control, new bounded races, collision/symlink negatives, isolated cleanup and marker/parent failures. |
| `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts` | `7b94d6388549f4459c19b8231e3336a63ab1e995` | Read all changed controls and adjacent listing, identity, denial and prune controls; actual control-server/inline-backend custody oracle retained. G-R3-01 is missing from the cleanup-code matrix. |
| `source/electron-main/mcp/host-plugin-skills-loader.ts` | `d81e1da13c9b39f4e2943ab48e8b9073bb8dc987` | Read complete loader. Typed failure consumer preserves the incoming runtime guard and canonical-ID validation. No separate finding in this change. |

## Other source conclusions

### Custody, publication and filesystem boundaries

The materialization implementation has one production caller in `loadFromMarketplaceSource`. The existing cache class remains the filesystem adapter; there is no new permission registry, installation service or lifecycle generation. Existing standalone extraction/removal/version-enumeration purposes remain intact. An exact source search found no production `extractToCache` caller and no second `materializeToCache` implementation.

`cacheDirectoryState` uses `lstat` for both final target and marker. Initial and prepublication conflicts are preserved; a completed target is reused. Staging is uniquely allocated below `dirname(resolve(cacheRoot))`, which is outside the two-level traversal of Rust `prune_uninstalled_plugin_dirs`. The marker is created exclusively before publication. A completed cooperating winner contains its marker and cannot be merged into by the attempted directory rename. After a successful rename, `ownsStaging` becomes false without another asynchronous boundary, so cleanup cannot target a later occupant of the old temporary name. Losing and failed attempts remove only their own staging path.

The documented arbitrary-writer/parent-alias/no-replace limit and cross-device fail-closed behavior are explicit. This review does not treat those excluded guarantees as provided. No other actionable custody or collision defect was established within that boundary.

### R3 integration, permission and existing lifecycle owners

Compared with the frozen base, the Host runtime clone guard is preserved. R3's type change uses the real `MarketplaceLoadFailure` contract instead of an ad hoc type omitting `errorType`. The bare-permission negative remains, the canonical-ID positive supplies an explicit Git producer error, and the cleanup double-failure positive uses that same category. The incoming older-failed-install successor-byte test also remains. These are appropriate integration choices; G-R3-01 concerns the newly appended cleanup code rather than a reason to relax the guard.

The shipping adapter still constructs the actual Host loader, supplies a captured per-pass account credential, rechecks current account context after materialization and requests `pruneOldVersions: false`. Rust `CoordinatorPluginSkillsLoader` deserializes the resulting aggregate directly. `SandPluginSkillsService` continues to own canonical listed-ID validation, denials, prior-record fallback, snapshot publication, pending-uninstall preservation, successor fences and committed pruning. Successful zero-skill install paths remain included in retention. `FileWorkflowStore` still consumes the shared admitted snapshot and live denial reader before and after file reads. Those adjacent paths explain both why the custody repair is correctly placed and why a missing auth block is materially significant.

### Oracles and evidence contract

The new SDK late-failure and successful-competitor tests distinguish staged bytes from winner bytes and inspect the final marker, losing-stage disappearance and fresh cache reuse. The Host race uses the actual control server and real inline synthesis, with zero-skill command/manifest bytes and an independent request B while A remains held. Collision tests inspect real directory/file/symlink state and foreign bytes. Marker and final-parent faults report per-entry failure without a successful install path. These controls call production owners rather than copying the reconciliation or publication algorithm.

The shipping Electron build script compiles the two contract files using its existing Node 24 common options and dependency resolution. The workflow runs the resulting bundles under `set -euo pipefail`, then requires each of the nine materialization names exactly once and zero failure/cancellation/skip/TODO totals before writing the receipt. Its receipt binds HEAD/tree, U, source/spec/test hashes and TAP digest and expressly limits the claim to the Linux Node boundary. Rust plugin lifecycle/cache/workflow and Coordinator waiter-retirement selections remain present. No Node result is represented as Rust, remote cancellation, real marketplace/account or platform-package acceptance.

The prior-root workflow additions appearing in the R3 delta are preserved incoming integration content: they are absent from the complete candidate-versus-frozen-base patch. They are not newly attributed to this plugin repair.

## Authority, method and remaining gate

Read repository root `AGENTS.md`, the relevant current source-of-truth authority/ownership sections, OCRM main specification §17, the plugin/skills/MCP module-map responsibilities, the full task specification, the adjacent snapshot-admission contract, and the candidate upstream lock. Product-source review included the actual backend materializer and error producers, marketplace auth classifier, shipping Host adapter/control server/Coordinator relay, Rust service/cache/pruner and workflow consumer, shipping Electron build selection and workflow receipt paths.

Only source/identity reads and creation of this report were performed. No repository source was modified. No current or previous review report or arbitration state was consulted. The fixed-U source-record claims were assessed as bounded provenance and were not represented as new independent upstream deep reads or acceptance.

Resolve G-R3-01, perform the required focused independent re-review against a new immutable snapshot, then obtain the applicable exact final-H Actions evidence. The source-contract and specification correctly leave execution and acceptance pending; those states remain pending here.
