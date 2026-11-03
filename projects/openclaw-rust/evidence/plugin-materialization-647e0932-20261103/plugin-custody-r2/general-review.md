# Plugin materialization custody — independent GENERAL / lifetime / permission / oracle review, R2

## Verdict

**APPROVE — source-only. No actionable findings.**

The frozen change satisfies the reviewed PM-01 through PM-06 source contracts. I found no falsifiable correctness, lifetime, permission, integration, or test-oracle defect requiring a source change. This verdict does not assert compilation, test execution, GitHub Actions success, or release acceptance.

## Review identity and method

- Reviewer: `/root/plugin_custody_general_r2`, fresh independent blind reviewer.
- Actual launch configuration: inherited configured parent model, GPT-6 Astra Pro, at inherited `xhigh`; no child model or effort override. The assigned scope warrants `xhigh` because it combines concurrent filesystem publication, cleanup ownership, authorization-sensitive error projection, and downstream admission.
- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Frozen base: `113c32276b99ed668be095e0be6ed164dffdf46d`.
- Frozen review tree: `c217f0f20090d75213c45602d6f333139c6f25ed`.
- Prior snapshot used only as the source-delta reference: `e9569789cc65760dc41403fefe42261b6b7a5c0f`.
- Scope: the exact nine paths in `manifest.json`, with `complete.patch` against the frozen base and `fix-delta.patch` as focused R2 context. `git diff --name-status` against these immutable identities shows exactly those nine paths.
- Authority read: frozen root `AGENTS.md`; relevant OCRM source of truth and main-spec section 17; module-map plugin/Skills boundaries; the complete materialization specification; and the applicable snapshot-admission requirements. Frozen-tree instruction discovery found no nested `AGENTS.md` governing these paths.
- Source inspection used `git show <tree>:<path>`, immutable Git diffs/searches, and reads of the supplied review artifacts. No mutable product source was used for the verdict. No prior review reports, arbitration records, or runner decisions were consulted.
- No local or htch-runtime test, build, lint, parser, validator, generator, or executable acceptance was run. The only write is this requested review artifact outside the repository.

All source references below refer to the frozen review tree, not to a later working tree.

## Scope disposition

| Path | Review disposition |
| --- | --- |
| `source/packages/cursor-plugins/cursor-marketplace.ts` | Deep review of the complete changed cache state inspection, materialization/publication, and all newly authored error/cleanup paths; compatibility review of the unchanged extraction, key calculation, enumeration and explicit removal interfaces. |
| `source/packages/cursor-plugins/loader.ts` | Deep review of the production call site, per-entry failure projection, cache-hit path, final-path content parsing, and both prune conditions. |
| `source/packages/cursor-plugins/loader.contract.test.ts` | Read all six new named test bodies and the retained listing, identity, default/false/true pruning and failure controls. |
| `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts` | Read all three new named test bodies, fixture changes, and retained Host listing, identity, authorization, zero-skill path and currentness controls. |
| `projects/openclaw-rust/docs/specs/plugin-materialization-attempt-custody.md` | Complete contract review, including PM-06, PM-O07, the atomicity limit, and explicit remaining scope. |
| `projects/openclaw-rust/evidence/plugin-materialization-647e0932-20261103/source-contract.json` | Reviewed identity, ownership, requirements, named positives, execution status and limits. It records authored work and pending exact-candidate execution; it does not claim this review ran tests. |
| `.github/workflows/rust-desktop-runtime.yml` | Reviewed exact-base additions plus surrounding build, TAP command, named-positive checks, receipt construction and retained native selections. |
| `projects/openclaw-rust/parity-ledger.json` | Reviewed this batch's added evidence references in the existing Skills responsibility and preservation of incoming shared context. No verification/completion promotion is introduced by the batch-only diff. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Reviewed this batch's added materialization-spec references and preservation of incoming shared context. |

The incoming DTR and media-related shared-workflow/ledger/RTM context in the fix delta is preserved in the current frozen base/tree relationship. It is not a new plugin-custody edit and this review does not re-adjudicate device business semantics.

## Production and lifetime assessment

`DefaultPluginCacheManager.materializeToCache` allocates a unique `mkdtemp` directory under the lexical absolute parent of `cacheRoot` (lines 133–135). The real Rust pruner enumerates only inside `plugins/cache` and then its marketplace/plugin children (`plugin_skills.rs:323–356`). Thus an in-progress stage is outside that traversal for the supported topology. The operation has no recursive removal of a final cache directory, sibling attempt, or prior version.

The private inspection helper uses `lstat` on both the final directory and marker (`cursor-marketplace.ts:105–119`), so a final target symlink is not a directory and a marker symlink is not a regular file. A missing marker does not make an existing directory eligible for replacement. Before publication, exclusive marker creation with `wx` rejects plugin-provided marker content without following a marker symlink (line 142). The successful staging directory is nonempty before the rename.

Both the prepublication completed-target path and the failed-rename completed-winner path return the existing final path while retaining ownership of the loser's stage until its `finally` cleanup succeeds (lines 144–177). A successful rename instead clears `ownsStaging` before return, so the old temporary name is never subsequently removed by this invocation (lines 168–169). Rename failure has no copy or merge fallback. The specified non-cooperating empty-directory insertion/parent-replacement and crash-residue limits remain explicit; the implementation does not claim a portable no-replace primitive.

The SDK caller passes the stage directly to the actual `client.installPlugin` and parses content only from the returned final path (`loader.ts:124–135`). The real backend branches were read: inline synthesis, marketplace-cache copy, Git installation, and release installation (`backend-marketplace-client.ts:179–217,293–317`, `inline-plugin-synthesizer.ts`, and `marketplace-cache.ts:279–280`). Their target-directory writes remain compatible with an already allocated empty stage; inline synthesis does not persist the temporary root in emitted metadata, and copy paths retain the existing verbatim-symlink behavior. No production `extractToCache` caller was found in the exact source search; its separate public operation is unchanged as specified.

Canonical identities, source-unavailable versus authoritative-empty behavior and per-entry error records remain in the existing loader/Host path. Both SDK pruning conditions are unchanged, and Host still sets `pruneOldVersions: false` (`host-plugin-skills-loader.ts:258–276`). Rust still admits successful materialization only when there is no `load_error` and there is a real install path; retains eligible unresolved old records; imports canonical denials before publication; and prunes only after a complete, non-denied committed pass (`plugin_skills.rs:1018–1129`). Its successful zero-skill install-path retention and pending-uninstall/successor guards remain in place (`plugin_skills.rs:797–900,1110–1129`). `FileWorkflowStore` still reads eligible snapshot records and consults live admission on consumption/mutation (`workflow_store.rs:339–398,593–599`).

## PM-06: complete cache-owned error-path audit

The neutral wrapper preserves the native error as `cause` and admits only an uppercase `E[A-Z0-9]+` code or `unknown` into message text (`cursor-marketplace.ts:27–41`). The Host classifier consumes the projected failure message, not the native cause (`host-plugin-skills-loader.ts:102–130`; `marketplace-cache.ts:36–54`). The standalone Rust classifier and the actual Rust consumer were also read; no classifier or public error protocol changes are introduced.

| Path / phase | Source behavior and authorization consequence |
| --- | --- |
| Initial inspection | `cacheIo("initial inspection", ...)` wraps inspection failure; existing incomplete/non-directory states use fixed state/phase text and retain the target only in structured cause details (lines 127–129). |
| Staging parent creation and allocation | Both filesystem operations use the neutral wrapper before any owned stage is available (lines 134–135). Native path text cannot become the failure message. |
| Actual installation callback | Deliberately outside `cacheIo` (line 140). The original callback error is rethrown when cleanup succeeds. Genuine remote authorization-denial text is preserved. |
| Completion marker creation | Exclusive write is wrapped (line 142). A native `EEXIST` path, including an auth-like data root, is not exposed in the classified message. |
| Prepublication inspection/conflict | Inspection failure is wrapped and conflict text is fixed, with the cache path confined to cause details (lines 144–146). |
| Publication parent creation | Wrapped separately (line 147). |
| Directory publication with readable destination | Failed rename rechecks destination completeness. A completed winner may be reused; otherwise the rename becomes a neutral `directory publication` error (lines 151–166). |
| Directory publication plus failed winner inspection | The aggregate contains two neutral phase errors, each retaining its raw cause. Its message concatenates only those controlled messages, and does not lose the publication failure (lines 153–163). |
| Cleanup after a primary failure | The aggregate preserves the primary error object as cause/first member and its original message, while appending only fixed cleanup text and a controlled code. Raw cleanup path/diagnostic text remains in the aggregate member (lines 178–190). |
| Cleanup after successful winner reuse | Cleanup failure rejects the operation with neutral text and its raw cause; it does not claim the residue vanished or return a successful materialization (line 192). |

This assessment covers every newly authored materialization I/O and conflict diagnostic, including combinations not individually triggered by the new Host example cases. It does not expand PM-06 into reclassification of pre-existing backend callback errors, which the contract expressly preserves.

## Test oracles and receipt

The nine required names in the source receipt and workflow match the actual top-level test declarations. Their bodies call the shipping SDK loader/cache manager or shipping Host loader; they do not copy the publisher or authorization classifier.

| New named positive | Causal source oracle |
| --- | --- |
| `SDK late failed materialization preserves the completed winner and a fresh cache hit` | Holds A inside actual install I/O, completes B through a fresh manager, checks returned parser content and file/marker bytes before and after A fails, and proves a third load performs no installation. |
| `SDK concurrent successful materializations reuse the first completed directory` | Uses distinguishable A/B bytes and an A-only file, then verifies both results consume B, the A-only file is absent, staging is gone, and the next load hits the cache. |
| `SDK incomplete and non-directory cache targets are preserved without installation` | Covers incomplete/file/directory-marker/target-symlink/marker-symlink targets, installation count zero, protected bytes/link identity, per-entry failure and no pruning. |
| `SDK a target appearing during materialization is preserved without publication` | Creates the collision during real loader-controlled install I/O, then checks collision bytes, absence of new skill/marker, owned-stage removal and failed cache status. |
| `SDK a failed materialization removes only its owned staging directory` | Checks stage disappearance alongside a separate staging-prefix sibling's bytes, a seeded prior version, no final entry, canonical identity and no prune calls. |
| `SDK marker and publication failures do not report a successful materialization` | Uses a reserved marker and an unusable final parent; checks per-entry failure, empty install path, no success/complete cache, stage cleanup and preserved foreign-parent bytes. |
| `Host late failed materialization preserves a zero-skill winner and a fresh cache hit` | Uses the actual control server, Host loader and backend inline synthesis; B completes while A is held, command/manifest/marker bytes survive A's failure, and a third actual Host request uses the cache. |
| `Host staging cleanup failure preserves the canonical authorization denial` | Forces an actual cleanup `ENOTDIR` within the fixture's exclusive parent; pairs genuine denial and ordinary callback error, checks the canonical block only for the former, explicit unknown residue and the surviving stage bytes. The ordinary case uses a `permission denied` parent name. |
| `Host cache-owned materialization failures cannot create authorization denials` | Runs initial conflict, during-install conflict and staged-marker symlink collision under an auth-like data root; checks exact identity, empty install path, `authBlocked: []`, protected collision/old-version bytes and no fresh completion. |

The barriers order the reported races without sleep-based scheduling. The Host test's actual control server uses independent asynchronous dispatch (`coordinator-control-server.ts:77–109,142–143`). The existing Coordinator timeout-retirement source control separately demonstrates that timeout retires the local waiter while the remote effect remains active (`coordinator_contract.rs:253–289`); the new test does not claim to execute that Rust timeout itself.

The workflow retains the shipping Electron build selector and invokes the actual bundles with `set -euo pipefail`. Receipt creation follows the test step, requires each of the nine names to occur exactly once as a successful TAP result, requires all tests to pass with zero failure/cancel/skip/TODO counts, and binds HEAD/tree, fixed upstream, TAP digest and changed production/spec/test inputs (`rust-desktop-runtime.yml:4730–4855`). Existing Rust Skills/cache/workflow, Session admission and Coordinator selections remain present. The build script uses the existing shipping bundling options; its receipt accurately excludes whole-graph TypeScript typechecking.

## Evidence limits

This is an independent source review of the stated immutable scope. The authored tests and workflow have not been executed by this reviewer, and their presence is not a successful runtime result. Exact-candidate GitHub Actions evidence is still required under the specification. I did not re-perform the receipt's historical fixed-U upstream reads or claim full upstream installation-family parity. The documented limits for arbitrary filesystem writers, cross-device topology, process-crash residue recovery, post-response account/uninstall retirement, live marketplace/accounts and macOS/Windows packaged acceptance remain limits of this batch rather than newly asserted guarantees.

No preference-only changes, speculative cleanup, or additional acceptance claims are proposed.
