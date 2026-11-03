# R4 independent GENERAL/security/lifetime/oracle source review

## Verdict

**SOURCE-ONLY APPROVE for the declared ten-path plugin materialization scope at tree `77fa185818b171a5b3829fc54104a0beec9ad8b4`.** I found no actionable blocking source defect in that scope. No product change is requested by this review.

This verdict is a source review result. It is **not** an execution result, a `verified` promotion, or final **ACCEPT**. No exact-subject execution receipt was evaluated here. The reviewed identity is an immutable tree; a subsequently selected subject commit H and its Actions evidence must be bound separately before execution or acceptance claims are made.

## Identity, independence, and method

| Item | Reviewed identity or constraint |
| --- | --- |
| Repository | `/workspace/scratch/425c55809bda/fabushi-desktop` |
| Frozen comparison base | `fdb0f6acb06165b6f2796961186e06a2de98c229` |
| Immutable R4 tree | `77fa185818b171a5b3829fc54104a0beec9ad8b4` |
| Prior tree for the supplied focused delta | `4f9c45c13151c58a2cfa784032cfdae4c0269578` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Review identity | `/root/plugin_custody_general_r4`; fresh blind GENERAL/security/lifetime/oracle review |
| Actually applied model/effort | Inherited configured **GPT-6 Astra Pro / xhigh**, confirmed by the parent; neither model nor reasoning-effort override was supplied |
| Actual fork | `fork_turns="none"`, confirmed by the parent |
| Risk classification | xhigh for stateful permission, concurrency, and recovery scope |

I read current `AGENTS.md`, the current authority portions of `projects/openclaw-rust/SOURCE_OF_TRUTH.md`, migration specification §17 including operative OCRM-EXEC-02 §§17.1–17.5, the M16/M17/M18 module ownership entries, fixed-U lock identity, the frozen materialization specification, and the relevant adjacent snapshot/admission specification. I did not read prior review reports, review decisions, or another reviewer's output. I used the supplied snapshot artifacts and scoped source diffs, and read product source with `git show` against the immutable R4 tree rather than mutable working files or an old HEAD.

Every Git operation used `GIT_NO_LAZY_FETCH=1`. No network fetch was performed. An initial unscoped prior-tree diff could not read an unrelated unhydrated blob; the parent identified it as unrelated incoming evidence/LFA scope. Subsequent diff reads were restricted to the declared paths. No necessary scoped or adjacent source read remained blocked. No tests, builds, parsers, linters, validators, generators, or harnesses were executed, locally or through htch-runtime. The only file written by this review is this report.

## Read scope

The complete declared change scope was reviewed against the frozen base, together with the focused prior-tree delta. Whole product implementations, both contract suites, the frozen spec, and source-contract metadata were read. For the large shared ledger, matrix, and workflow, I read the complete scoped changes and the relevant surrounding ownership/status/selection/receipt sections; this report does not claim a review of every unrelated responsibility or workflow job.

| Declared path | Review coverage |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | Complete scoped changes; exact-subject checkout assertions, shipping Electron build, five Node bundles, TAP controls, receipt inputs and upload, and relevant retained native/Coordinator selectors |
| `projects/openclaw-rust/docs/specs/plugin-materialization-attempt-custody.md` | Full frozen spec, PM-01 through PM-07, PM-O01 through PM-O08, boundaries, evidence and acceptance requirements |
| `projects/openclaw-rust/evidence/plugin-materialization-647e0932-20261103/source-contract.json` | Full source-contract metadata, fixed-U attribution, unresolved gaps, validation contract and pending claims |
| `projects/openclaw-rust/parity-ledger.json` | Complete declared change and affected Plugin Skills responsibility/status/evidence context |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Complete declared change and affected requirement/spec references |
| `source/packages/cursor-plugins/cursor-marketplace.ts` | Full cache adapter, target inspection, staging, exclusive marker, publication, cleanup, and unchanged cache operations |
| `source/packages/cursor-plugins/loader.ts` | Full loader, primary-failure classification, actual-path projection, listing behavior and both prune gates |
| `source/packages/cursor-plugins/loader.contract.test.ts` | Full suite, including new causal race/fault cases and retained incoming controls |
| `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts` | Full suite, including actual control-server concurrency, canonical identity, cleanup faults, diagnostics, account and prune controls |
| `source/electron-main/mcp/host-plugin-skills-loader.ts` | Full production Host loader, canonical identity, auth-block gate, current-account checks and real SDK configuration |

Necessary adjacent reads included the SDK backend client and marketplace clone classifier; the shipping Electron bundler; Electron production adapter/provider/executor/control-server routing; Rust `CoordinatorPluginSkillsLoader`, `SandPluginSkillsService`, snapshot cache, production extension and MCP ports, and workflow-file consumers; Coordinator control-call retirement and production timeout paths; and the retained `coordinator_contract.rs` timeout retirement oracle. These were read to establish actual ownership, callers, consumers, lifetime behavior and build selection, not to claim a general review of all adjacent modules.

## Findings and supporting source analysis

### 1. Attempt ownership and publication remain in the existing cache adapter

`DefaultPluginCacheManager.materializeToCache` in `cursor-marketplace.ts` owns only the filesystem materialization attempt. Its `mkdtemp` directory is a unique sibling under the parent of the absolute cache root, outside the two-level cache traversal used by Rust prune. The installer receives that attempt's stage. The adapter creates `.cache-complete` exclusively, then publishes with same-filesystem `rename`; it does not add a copy fallback, persistent lock, cache index, or TypeScript permission/lifecycle owner.

The relevant falsifiable adverse ordering is: A writes partial stage bytes and remains pending; B independently completes and publishes the shared final target; A subsequently rejects. A's `finally` removes only its captured staging path. There is no final-target removal in this path. Successful rename clears `ownsStaging`, so cleanup cannot later remove a name for which the attempt has transferred custody. When A instead succeeds after B, the complete winner is reused and A's private stage is retired. A failed rename also re-inspects the destination and accepts only a complete winner.

Both SDK race cases use separate cache-manager instances and explicit barriers, inspect B's bytes before releasing A, then inspect final bytes and a subsequent cache-hit consumer. The Host race uses the actual Coordinator control server, actual Host loader, and real cache with a zero-skill plugin. This matters because a zero-skill result still carries the actual installation path needed by Rust retention. The assertions are stronger than checking only rejection text or a mocked cache call. These are authored, selected source oracles; this review did not execute them.

### 2. Marker and collision handling fails closed within the stated filesystem model

`cacheDirectoryState` uses `lstat` for the target and its reserved marker. A reusable target must be a directory with a regular-file `.cache-complete`. An incomplete directory, ordinary file, target symlink, directory marker, or marker symlink is preserved and rejected. The exclusive staged marker creation rejects an installer-supplied reserved marker, including a symlink, without overwriting its referent. Initial and post-install checks precede publication; a publication error has a separate winner inspection.

The SDK controls assert preserved sentinel contents or link targets, whether installation ran, missing final success, disappearance of only the owned stage, and preservation of an unrelated stage and old version. The publication control now creates its conflicting target during installation, so it can reach the post-install publication boundary rather than stopping at the initial cache check. The Host cache-diagnostic controls put `git permission denied` in fixture paths and preserve external bytes behind a reserved-marker symlink. That makes accidental promotion of filesystem path prose into a Git authorization denial observable.

The spec expressly limits the atomicity claim to cooperating publishers. Arbitrary external replacement of target/parent paths or insertion of an empty directory in the final check-to-rename interval is not established by this design. Lossy cache-key collisions, artifact trust, crash-residue collection and platform-wide filesystem guarantees also remain outside this bounded change. I did not treat those explicit, unchanged boundaries as newly introduced defects.

### 3. Cleanup failures preserve primary meaning and raw evidence

The stage owner retains a holder for the primary thrown value, rather than using `undefined` itself as the no-error sentinel. Cleanup failure with a primary error produces an `AggregateError` whose `cause` is that primary value and whose `errors` retain both original values. Successful work followed by cleanup failure is reported as a cleanup error rather than a successful load. The diagnostic states that residual state is unknown; it does not claim deletion completed.

`classifyLoadError` in `loader.ts` follows own `AggregateError.cause` chains with cycle protection, including an explicitly present `undefined` cause. It leaves ordinary Error wrappers opaque. Thus a neutral cache wrapper's raw native cause remains available for diagnosis without feeding path/native-message prose back into permission classification. Cache-owned visible messages contain only controlled phase/state and a constrained I/O code. Real installer callback errors retain their original semantics. Native `EACCES`, `EPERM` and `EROFS` are recognized as installation failures even when the ordinary message does not name the code.

The SDK fault control substitutes only `fsPromises.rm` for the exact current stage, delegates other calls, restores the binding, and checks actual remaining stage bytes, absent completion, absent fresh final result, full raw cause identity and the original category. It covers cleanup `ETIMEDOUT` and `EACCES` against genuine clone denial, ordinary failure, a nested aggregate primary and explicit `undefined`. The Host fault control checks canonical denial versus ordinary failure and a subsequent successful load. A separate native `ENOTDIR` scenario remains: the fixture changes the parent shape so real cleanup fails while the stage's bytes remain observable. These controls exercise failure, ownership, diagnosis and recovery rather than merely recognizing a desired string.

### 4. True Git refusal remains distinct from local failure

`authBlocksFromFailures` in `host-plugin-skills-loader.ts` still requires both `failure.errorType === "clone"` and `classifyCloneError(failure.errorMessage) === "user_git_access"`. It uses the canonical plugin database ID, rejects a missing canonical identity, and deduplicates by that identity. The SDK's primary-error classification therefore remains a meaningful boundary before the Host's textual Git refusal recognizer.

The new controlled cleanup tests show, in their source assertions, that cleanup timeout/access errors cannot change an ordinary primary into authorization denial and cannot erase a genuine primary denial. The canonical identity test deliberately separates the denied plugin's database ID from its slug and another plugin's database ID. Incoming listing/version-resolution failures preserve their typed category and canonical ID; unavailable and empty listing results remain distinct. No new permission decision was moved into the cache adapter.

### 5. Production Rust ownership and consumers remain intact

The shipping Electron adapter constructs this Host loader and the actual control executor calls it. The Host passes `pruneOldVersions: false`; both SDK prune sites respect that configuration. The Host returns the actual published installation path from the SDK, preserves empty paths on per-entry failure, validates listing identity before filtering, and performs its existing current-account check before returning the aggregate.

In Rust, `CoordinatorPluginSkillsLoader` requests `loadHostPluginSkills`; `SandPluginSkillsService` validates the aggregate, imports current refusal facts, chooses eligible prior fallback, publishes/readbacks the snapshot and performs committed prune only under its existing active/current and admission checks. `prune_uninstalled_plugin_dirs` traverses cache-root plugin directories, preserving retained actual installation paths, including zero-skill paths. It cannot reach the new sibling attempt directories through that traversal. `plugin_skills_cache.rs` and workflow-file consumers remain responsible for canonical snapshot eligibility and actual retained-file consumption. The patch does not create a second admission, snapshot, uninstall or prune authority.

Coordinator timeout retirement remains local waiter retirement. The production control-call path and retained `timeout_retires_only_owned_waiter_while_remote_effect_continues` test do not promise cancellation of a running remote installer. The new cache design tolerates that installer continuing and later settling. This is the lifetime condition the materialization change needs to handle; no cancellation guarantee is inferred from a timed-out reply.

### 6. Evidence selection is concrete and does not promote source claims

The workflow builds the shipping Electron bundle and contract entries with the existing Node 24/esbuild options, then runs the five actual Node test bundles under shell failure propagation. The plugin evidence step checks the checked-out commit against the event's intended subject, records its tree and the fixed-U lock identity, hashes the source/spec/build inputs and TAP log, and requires exactly one successful top-level TAP occurrence for each of eleven named materialization controls. It also requires a completed all-pass TAP summary with no failed, cancelled, skipped or todo tests. This addresses the risk of a green wrapper selecting none of the new scenarios.

The materialization spec and evidence source-contract explicitly retain the need for exact-H Actions results and separate Rust evidence. The scoped ledger delta adds references without promoting verification, wiring or acceptance; the RTM delta adds spec references without changing status. The prospective renderer receipt explicitly states its Linux/controlled-I/O scope and keeps independent acceptance pending. These are appropriate limits for the evidence this workflow could produce. I did not evaluate an actual run or infer that future receipt generation has succeeded.

## Remaining gaps and acceptance boundary

1. **Execution remains pending.** Nothing was run in this review. The selected subject commit H needs its own successful Actions execution and inspected receipts; this immutable-tree source approval does not substitute for those results or authorize borrowing an older subject's receipt.
2. **Native evidence remains separate.** The renderer receipt cannot establish Rust admission, snapshot/prune recovery, or Coordinator execution outcomes. Relevant native selectors remain in source, but their actual execution was not reviewed here.
3. **Product acceptance remains pending.** Live marketplace/accounts, complete cross-account and persistence-failure revocation behavior, remote cancellation, process-crash residue handling, and macOS/Windows package acceptance were not established. The source documents disclose these limits.
4. **Fixed-U provenance is bounded.** I checked the fixed identity, authority and in-scope source-contract attribution and its admitted upstream gaps. I did not independently reopen the original upstream implementation in this review and do not claim complete upstream parity closure.

There are **zero actionable source findings** to return for correction in the declared R4 scope. The appropriate next gate is exact-subject executable evidence and independent acceptance under the existing authority, with this source approval kept distinct from both.
