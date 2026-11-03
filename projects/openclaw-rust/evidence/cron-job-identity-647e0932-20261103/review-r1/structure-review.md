# Cron job identity — independent structure / reuse / performance review, round 1

**Verdict: SOURCE-ONLY APPROVE.**

No actionable blocking finding was established in the complete seven-path change scope. This verdict covers source structure, reuse, production reachability and proportionate performance only. It is not compilation, test execution, batch verification, whole-Automations acceptance or final OCRM acceptance.

## Review identity and method

- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`; canonical remote `bhrumom/fabushi-desktop`; declared branch `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Exact base commit supplied by the controller: `972779ecd290b90e407ffe651aa5c456c31e9f7f`; exact base tree used for comparisons: `a75bb4706831a34b818e459f6be4df920560a2fe`.
- Immutable candidate tree reviewed: `f5e577cf9e119f417ce0cf071f03c214c5bf6190`.
- Fixed upstream U: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Model / effort: controller-confirmed launch parameters `fork_turns="none"`, `model="gpt-6.1-sol"`, `reasoning_effort="xhigh"`. This worker has no independent runtime introspection interface. The scope requires xhigh because it crosses identity namespaces, durable state, execution settlement and a production/test extraction seam.
- Blind review: no other review report or runner decision was read. No production/shared source was edited.
- Candidate text was read using `GIT_NO_LAZY_FETCH=1 git show <candidate-tree>:<path>`. Exact base/candidate Git diff and tree entries were inspected; mutable HEAD/worktree was not treated as the candidate. The base reconstruction/API procedure recorded in the supplied manifest was not independently repeated.
- No compiler, build, test, lint, formatter, parser, generator, validator or harness was run locally or on htch-runtime. No candidate execution results were supplied or inferred.

The complete changed-path/status set was independently read from the exact base/candidate trees and is exactly the seven manifest paths. Their candidate blob identities match the manifest:

| Path | Candidate blob | Reading scope |
| --- | --- | --- |
| `source/host/app/src/main.rs` | `5e1a392d615bec42153e9936f7dc23db8954cd81` | Every changed hunk; complete new functions; removed old bodies; adjacent App/native/telemetry consumers below |
| `.github/workflows/rust-desktop-runtime.yml` | `8f924c42928fa61242a2f0129ee20f9549a5c92f` | Every changed hunk; relevant trigger, job, cache, App build/test selector, receipt and artifact context |
| `source/host/app/src/tests/cron_job_identity_tests.rs` | `646d24ab3d27dede7bfcb4f5fe421585694c390b` | Complete new file |
| `projects/openclaw-rust/docs/specs/cron-job-identity.md` | `988f36d855e3efffc5b5563006dddf926393fcbe` | Complete new file |
| `projects/openclaw-rust/evidence/cron-job-identity-647e0932-20261103/source-contract.json` | `79b752dbc6aab3d4e2fd9612aa5aae89f5cbd24f` | Complete new file |
| `projects/openclaw-rust/parity-ledger.json` | `379557210e62d2c380daba068e6e8c774f051187` | Complete diff and affected dispatch-registry row context |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `3605dc7310eb4dfd06385b6aae0612a2b560d576` | Complete diff and affected R04/AC11 context |

## Findings

None. There is no speculative or preference-only request for change.

## Source assessment

### Existing-owner reuse and production reachability

`main.rs::list_openclaw_cron_jobs` (around lines 2839–2906) and `trigger_openclaw_cron_job` (around lines 2908–2994) remain private same-file production functions, outside `cfg(test)`. They receive the existing `SandAgentSessionStore`, hold no durable/global state, and add no owner or product module. The change removes the entire original list/trigger bodies from `UnifiedGatewayApi`, including the old first local-ID match and `break`; the old implementation does not remain as a parallel branch.

The actual route remains connected: `GatewayApi for UnifiedGatewayApi::call` at approximately 13451–13461 dispatches registered OpenClaw commands; `call_openclaw_gateway_command` at 4307–4347 routes `cron.list` and `cron.trigger` to the two App methods; those methods at 5288–5304 construct the store from `self.session_workers` and invoke the extracted functions. The production trigger port calls `self.call("runAgentAutomationNow", run_args)`. The existing native handler at 13957–14013 consumes the resolved Agent and **local** automation ID, selects the configured routed Runner or existing Host lane, and invokes the existing `AutomationRuntime` path. The supplementary test source anchor is therefore corroborated by direct source reading, rather than serving as the only wiring evidence.

`host_gateway_api.rs` still registers the two methods and assigns both to `HostGatewayOwner::Automations`. Relevant `gateway_server.rs` operator/mutating-scope and authenticated command-call boundaries were read; this patch changes none of them and introduces no alternate grant or authorization truth. The input `agentId` is not a selector in the extracted resolver.

The existing ID function was read in full in `source/host/src/automations/automation_id.rs`. Its hash input is the Agent ID, a separator and the local ID; its established UUID-shaped output has version nibble 5. The patch calls that function directly. Related existing usages were read in `automation_runtime.rs::lifecycle_event`, `automation_store.rs::find_for_backend_delivery` and the App's run reporter at about 16370. There is no second ID algorithm, migration/alias map, scheduler, session store or run-history owner. `source/host/app/Cargo.toml` already declares `uuid`; UUID recognition does not require a dependency or feature addition.

### Resolution and state boundaries

The resolver scans the same visible Agent/routine projection used by the list, tests the canonical ID and legacy local ID together, and records each returned pair once. It resolves before dispatch. A second matching pair produces `BadRequest`; canonical-vs-legacy overlap cannot obtain first-match priority. A unique legacy match outside the reserved namespace remains compatible. An input parsed as UUID v5 cannot execute solely through a legacy match after its former canonical owner disappears. The explicit namespace decision is documented consistently in the complete task spec and source contract.

The resolved Agent/local pair also governs before/after history observation, while `run.jobId` returns its canonical ID. `FnOnce` and the single invocation preserve the one-dispatch boundary; callback errors propagate with `?`, without retry or fabricated completion. List fields, pagination and newest-history projection are the extracted prior implementation apart from the intended identity change.

The adjacent owners were read at their actual boundaries: `SandAgentSessionStore::create_session`, `list_agents`, `automation_store_for`, `list_agent_automations` and `surface_automations`; `ProductionSessionWorkers` fixture constructors, `open_automation_store`, `list_agent_summaries` and shutdown; `FileAutomationStore` definition creation, slug/unique-ID producer, list/read-history and stable backend lookup; the runtime module's free `dispatch_automation_command` function and `AutomationRuntime` create/delete/get/run-now methods; and `AutomationRunPath` admission, duplicate suppression, execution result settlement and reporting. Those owners continue to own persistence and terminal custody. No cancellation or uncertain-effects state was copied into the compatibility adapter.

The task spec explicitly retains the current fail-soft visible listing and existing lookup-to-execution race/result-correlation limits. This review does not turn that existing projection into a checked global filesystem inventory or a transaction. No unrelated global source-closure requirement was used to block this bounded identity repair.

### Test relevance and build selector

The complete test file uses the actual extracted production functions. Fixtures exclusively create a UUID-named root with `create_dir`; Agents come from real `SandAgentSessionStore::create_session`; routines come from `dispatch_automation_command -> AutomationRuntime::create_agent_automation -> FileAutomationStore::upsert`. Expected canonical IDs use the existing stable mapper. The run callback passes the **adapter-supplied** pair to `run_agent_automation_now_with`, rather than substituting the expected target. The controlled element is the final execution result.

The five named controls cover both canonical aliases in reverse list order, unique then ambiguous legacy addressing, wrong `agentId`, canonical/legacy namespace collision and post-delete refusal, owner reopen, completed/error/failed/interrupted/retained-unsettled outcomes, and callback refusal without retry. Assertions read real running and terminal history, actual observations, prior-run preservation, and selected/nonselected config/history bytes. Shutdown calls `runtime.dispose()` and `workers.shutdown()` before deleting the exclusively owned fixture root. These are source-reachable behavior controls, not a copied resolver or a standalone test registry. They still do not construct the complete HTTP/App stack or execute a real external provider.

The workflow additions are in the pre-existing `sqlite-native-path` / `automation_app` lane. The canonical push branch and `workflow_dispatch` with `validation_scope=ocrm-config` select that App step. The unchanged surrounding workflow checks `cfg(not(test))` via the App bin, builds the actual App test harness through Cargo compiler artifacts, restricts the selected executable to the owned target directory, and executes every required name with `--exact`. The added module path in `main.rs` and all five fully qualified selector names agree. The selector requires one exact listed test and one-passed/zero-failed/zero-ignored completion; exceptions leave the receipt failed. The existing upload includes the receipt and raw logs. H/tree/U, run/attempt, input hashes, exact commands, executable hash and stream hashes remain recorded. This is an authored execution path, not evidence that this candidate has passed it.

### Complexity, dependencies and evidence references

The list retains its prior aggregate allocation and per-owner read behavior and adds one existing ID computation per record. Trigger now completes enough of the owner scan to prove uniqueness, instead of stopping at the first local match. Its additional work is linear in visible records plus their existing owner I/O; it retains one selected tuple, not a new global index. That cost follows directly from cross-Agent/cross-namespace ambiguity detection. There is no introduced nested global scan, polling loop, background worker, network round trip, dependency change or persistent cache. The substantial fixture code is under the existing test module and does not enlarge the shipping runtime surface.

The ledger change only appends the three real spec/source-contract/test paths to the existing `OCRM-RESP-GW-DISPATCH-REGISTRY-001` evidence list. RTM changes append the same real paths to existing R04 and AC11 rows. The new source contract expressly limits that anchor and does not pretend the registry row closes every cron responsibility. No source-review row, historical read fact, implementation/verification status or global flag is upgraded by these diffs.

## Authority actually read and limits

Root `AGENTS.md` was read completely. The candidate task spec and source contract were read completely. Main OCRM Spec §17, including the older §17 record and the overriding OCRM-EXEC-02 revision, was read. Relevant Source of Truth authority/fixed-U and existing-owner restrictions, module-map M19/ownership context and the exact lock upstream identity were inspected. This was selective contextual reading, not a claim to have deeply reviewed the complete project navigation/history documents.

Five fixed-U source slices were fetched through the exact-ref GitHub file API and their returned blob SHAs matched the source contract:

| Upstream path | Lines actually read | Returned blob |
| --- | --- | --- |
| `src/cron/service/jobs.ts` | 171–230 | `780d8542ad4460b63ab39a4b2096671192853ec4` |
| `src/cron/service/ops-read.ts` | 1–100 | `3fd0daf829edc3c94ed052dd78478c3ce9f1bea0` |
| `src/gateway/server-methods/cron-job-access.ts` | 126–235 | `28a75469df97ceaef4cc137510ee8477ee07038a` |
| `src/gateway/server-methods/cron.ts` | 501–625 | `35cd3e5e36e26ccefba265c4955377fff0bc5ca0` |
| `src/cli/cron-cli/register.cron-simple.ts` | 251–280 | `ed18eba20a47260fba17a9b7b1b75fba372fb524` |

These reads corroborate exact stored-job selection and the actual CLI ID consumer only. They are not full-file deep reviews, complete upstream cron access/grant/queue/wait compatibility, recursive dependency/LFS/source closure, real service/provider behavior, packaged platform compatibility or unit/global acceptance. Target adjacent files were read in the specific owner/caller/consumer slices described above, not globally in full. Candidate compilation, all executable test outcomes and performance measurements remain unknown until exact-candidate GitHub Actions evidence is assessed.

**Final source verdict: SOURCE-ONLY APPROVE; zero blocking findings.**
