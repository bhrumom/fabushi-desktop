# Plugin authentication CI recovery — independent structure, reuse and performance review

Verdict: **APPROVE, source-only**, for the complete four-path frozen delta identified below. I found no actionable defect in the assigned reuse, structure or performance lenses. This verdict does not establish compilation, test execution, runtime behavior, packaged-platform behavior, upstream parity, or global OCRM acceptance.

## Review identity and execution limits

- Reviewer: `/root/plugin_auth_ci_structure_r1`, fresh independent worker. No current-round peer report or root arbitration decision was read. The required frozen specification was read completely, including its historical contract amendments; those embedded historical records were not treated as current-round review results.
- Actual launch settings: `model="gpt-6.1-sol"`, `reasoning_effort="xhigh"`, `fork_turns="none"`, confirmed by the parent from the successful adapter call. These are root-verified launch metadata, not worker self-introspection. The parent reported that an earlier launch attempt created no worker because of a thread limit.
- Effort rationale: authority poisoning interacts with one-shot lifecycle claims, shared state ownership, old refresh guards, error boundaries and retained test oracles. A small diff does not make those interactions routine.
- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Exact base and starting HEAD: `871b033c5b9aabbe322cc7322af92648939db9b5`.
- Immutable review tree: `ba707272d259b7da5f0def5f31eb3380ce6fd3e8`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`; unchanged by this delta. No new upstream source-reading or closure claim is made here.
- Manifest: `/workspace/scratch/29e7135d8bfe/reviews/plugin-auth-ci-r1/manifest.json`.
- Complete patch: 7,191 bytes; SHA-256 `c403ee62e75d7d4da5d1ea61ee72f0908f5c12aea3348f70d442c1a0a6130c3d`. A read-only hash read matched the supplied identity. Git's base-to-tree diff showed exactly four changed paths, 53 insertions and six deletions.
- Source was inspected with `git show` against the immutable tree and exact base, rather than relying on mutable working-tree source. Commands were read-only Git, source reads/searches, and patch identity inspection. No source edit, Git mutation, dispatch, build, test, lint, rustfmt, generator, schema check, benchmark, harness, local validation or htch-runtime validation was run. The only authored file is this report.

The assignment and frozen AR-CI amendment identify original exact-H `d70f86f33b5e3c67545954b270373e1548809f4b`, Actions run `41578381556`, Linux job `125727154700`, as having two existing-contract failures. I did not independently inspect those original logs and do not use that supplied history as candidate execution evidence. The reviewed candidate is unexecuted for the purposes of this review.

## Complete frozen scope and read depth

| Changed path | Frozen blob | Disposition |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md` | `c99e744188d90d7b31448e45e9887148c835dda6` | Complete file read, including PS-LIVE, uninstall/private-image prerequisites, AR-01–09 and the complete AR-CI amendment; all added lines reviewed. |
| `source/host/src/extensions/mcp/plugin_skills.rs` | `a0054eebb18e38b234689d72ca998dac3fd77901` | Complete production file read; changed helper, retirement and dispose compared with exact base; all callers of the changed helper searched. |
| `source/host/src/extensions/mcp/plugin_skills_sync_tests.rs` | `cb8abb868bef2469bded2dda87e0680e5c54b329` | Complete changed test and all added assertions read, together with actual harness/loaders and adjacent handoff, coalescing, cleanup and pending-uninstall controls. Test-name enumeration covered the file; unrelated test bodies were not all reread. |
| `source/host/tests/mcp_coordinator_relay_contract.rs` | `71144e4e578a51fd2281dcb6b743e5c7a11d569e` | Complete file read in sections, including real service/relay fixtures, the entire changed desktop status test and retained close, unknown-outcome, request-correlation and byte-retention controls. |

I read current root `AGENTS.md` and both governance sections numbered 17 in `docs/specs/openclaw-rust-equivalence-migration.md`, including OCRM-EXEC-02's review-depth and evidence distinctions. The tree contains no nested AGENTS instructions governing these four paths. I consulted the current Source of Truth's authoritative entry/constraints, the module-map M16–M18 ownership rows, and the snapshot-skills dossier's relevant source-provenance navigation. The broader Source of Truth/module map/dossier and their historical linked records were not represented as completely read.

Adjacent source inspected at the frozen tree included:

- `host_production_extensions.rs`: `PluginSkillsBinding`, registration/root checking, preparation outside the binding lock, repeated installation checks, retention of the newest owner before Session binding, actual `start_session`, `start_mcp` and `stop_mcp`, plus relevant same-root, different-root and failed-binding tests.
- `extensions/mcp/extension.rs`: runtime stop/dispose and poll disposal/join boundary; `production.rs`: actual Skills service/port adapters and status projection; `mcp_service.rs`: desktop refresh/sync/status dispatch and closed-service check.
- `extensions/session/production.rs`: shared forwarding reader and actual workflow-store construction/binding; `workflows/workflow_store.rs`: committed snapshot/parse-cache reads and live list/get/update admission checks; `host_runner_composition.rs`: direct consumption of the Session-opened workflow store.
- `plugin_skills_uninstall_state.rs`: strict v1/v2 data model, byte-bound expected-image checks, load, persistence, failed-auth-publication inspection, regular-file checks, staging/rename/directory-sync and owned cleanup. Its unrelated test instrumentation was not exhaustively reread.
- `plugin_skills_cache.rs`: committed eligibility and actual staged cache publication/barriers; `source/host/Cargo.toml`, the MCP module declaration and the existing library test-module declarations as the relevant build/test inclusion surfaces.
- `plugin_skills_auth_recovery_tests.rs`: adjacent old-loader/guard/repeated-dispose versus successor-token control and panic/coalesced unknown-recovery control. This unchanged test file was not read in full.

Read limits are deliberate: this is a complete review of the frozen change scope with the needed adjacent lifecycle/consumer contracts, not a fresh audit of every unchanged MCP/Session/Runner function, every upstream U source, the whole dependency graph, all workflow bodies, or the complete OCRM project.

## Findings

**None.** There is no severity/path/scenario/fix entry because no concrete defect was established. No preference-only cleanup is being promoted into a blocking finding.

## Source reasoning supporting the verdict

### The nonfatal boundary is exactly the poisoned admission fence

At frozen `plugin_skills.rs:741`, `fence_owned_refresh` remains private and is called only from retirement and disposal while the service holds its publication lock. Its new `let Ok(...) ... else { return; }` handles only failure to acquire the admission mutex. It does not call `into_inner`, inspect unavailable admission, clear poison, replace an Arc, initialize state, perform a disk operation, invoke the loader, sample the clock, or call an external callback.

That distinction matters because `PluginSkillsLiveAdmission::is_denied` already maps an admission-lock error to `true` (`plugin_skills.rs:426` onward). Keeping the same poisoned Arc therefore retains plugin-wide denial. The new early return cannot become an empty denial set or a grant.

The successful retirement still acquires publication first, acquires service state next, rejects a previously claimed replacement, sets `disposed` and `replacement_claimed`, clears the pending slot and wakes waiters before dropping state (`plugin_skills.rs:883–903`). Publication and service-state lock failures remain explicit errors before the claim. A duplicate claim still returns `plugin skills replacement already claimed`. Only the post-claim admission fence stops manufacturing a new error that would strand Host at an already-consumed predecessor. Plain dispose already discarded that helper's error at the exact base, so its changed call preserves that existing outward behavior.

Healthy admission still checks the exact `refresh_owner` Arc before setting `Fenced`. The helper does not fence a successor's different owner token. `successor_with_loader` still calls retirement before constructing a same-root owner with the identical admission Arc (`plugin_skills.rs:871–881`). An old guard checks both owner and operation ID, while old aggregate import/publication/settlement paths check the disposed predecessor under its own publication interval. The new poison handling does not bypass those checks.

### Existing composition and reader ownership remain intact

The actual Host binding retains one latest Skills service and one registered Session reference. Same-root preparation uses the one-shot successor operation; the prepared disk reader is used only when appropriate for fresh/different-root construction. Recovery I/O remains outside the binding lock, root checks repeat after preparation, and the newest successor Arc is retained before a fallible Session bind (`host_production_extensions.rs:466–581`). None of that is replaced by the patch.

The Session forwarding slot clones its delegate before querying and accepts rebinding only to the same reader Arc. Workflow admission therefore continues through the original poisoned reader even for stores opened earlier; no copied policy or second permission truth is introduced. The workflow store checks live admission around skill fact reads, including cache hits, and before its plugin write. The Runner composition uses that Session-opened store directly. These existing consumers make the same-Arc requirement material rather than a cosmetic pointer assertion.

A same-root poisoned successor is intentionally unable to sync: `begin_refresh` still fails on that admission mutex before a reservation can reach the loader. Old retained readers also remain denied. A separate fresh process's recovery behavior still belongs to the unchanged private image protocol; this patch neither repairs nor resets that durable state.

### Lock, I/O, callback and performance costs do not expand

The changed healthy path acquires the same single admission mutex under the same publication interval and makes the same bounded owner-pointer check/update. The poisoned path exits without inspecting state. There is no new allocation-backed store, traversal, polling loop, disk read/write, network call, scheduler, lock-order edge or callback. Service state is still explicitly dropped before the admission fence. The actual Host path may hold its binding lock around retirement, but this helper continues to perform no I/O or callback there, as AR-07/AR-CI-02 require.

The private-image schema, strict parser, expected-byte authority, write/readback and barrier ordering are unchanged. The positive cache-then-journal release and old cleanup ordering are unchanged. No additional performance execution is justified by this source delta alone; that is a source assessment, not a measured runtime claim.

### Both test changes preserve their decisive oracles

The entire poisoned-reader test (`plugin_skills_sync_tests.rs:1730–1767`) still poisons the actual private admission mutex, retains a real committed allowed snapshot, proves raw diagnostic records survive, checks live denial for both plugins, refuses sync/uninstall, disposes, and successfully constructs a same-Arc successor. It still requires the successor to return no current records, reject sync with the exact poisoned-admission error and preserve cache bytes. The additions pin both denied retained reads after handoff, the fixed fault/unknown status, the consumed one-shot claim and the original loader count. No prior assertion was deleted or relaxed.

The adjacent healthy handoff, staged-old-success, prune-before/after-handoff, ordinary-dispose, pending-uninstall and auth refresh-token controls remain in place. Their production call paths were inspected; this report does not claim they passed execution.

The desktop test (`mcp_coordinator_relay_contract.rs:853–927`) continues to construct the real service with the concrete lifecycle relay and actual Skills loader. It still holds refresh/sync results pending until the correlated loader response, verifies returned records and path, commits a real refusal, asserts full status-object equality, preserves cache bytes on loader errors, and checks closed-service refusal and absence of new requests. The sole status expectation change adds the exact AR-08 projection: refusal 41/Calendar with `refreshOutcomeUnknown: false`. This agrees with the actual `PluginSkillsPort::status` adapter in `production.rs:295–305`, while retaining `authBlocked` exactly and forbidding unexpected extra fields through equality. It does not replace the oracle with a partial-field check or fabricate a second status owner.

## Remaining gate

The source delta is suitable to proceed to the required exact-revision GitHub Actions checks. The original failures remain historical evidence; no result from them is relabeled as candidate success. Candidate compilation and the selected actual owner contracts remain pending. Real providers, account/backend incarnation beyond response, captured prompts/effects, platform durability limits, full source/dependency closure and G01–G12/AC01–AC24 remain outside this source approval and unaccepted.
