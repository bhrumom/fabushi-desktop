# AR-CI round 1 — independent general/security/lifetime source review

Verdict: **APPROVE — source review only.** No blocking or non-blocking defect finding is raised for this immutable four-path recovery change. This is not a runtime, Actions, platform, ecosystem, or full OCRM acceptance verdict.

## Review identity and execution limits

- Reviewer setting: `gpt-6-astra`, `xhigh`, as applied in the parent launch/handoff. The complete assignment warrants xhigh because it changes a poisoned authority boundary after a one-shot retirement claim and must preserve refresh ownership, retained readers, callback boundaries, and causal oracles. No model/effort override or additional worker was introduced by this reviewer.
- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Exact base: `871b033c5b9aabbe322cc7322af92648939db9b5`.
- Immutable candidate tree: `ba707272d259b7da5f0def5f31eb3380ce6fd3e8`.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`; unchanged and not rebaselined or re-reviewed as complete upstream source.
- Read the manifest and complete patch. Read-only Git confirmed that the base-to-tree path set is exactly the four paths below. The retained patch SHA-256 is `c403ee62e75d7d4da5d1ea61ee72f0908f5c12aea3348f70d442c1a0a6130c3d` (7,191 bytes per manifest).
- The report is independent: no root decision ledger, peer report, or mutable DFA implementation was consulted. Candidate implementation and adjacent source reads used `git show`/`git grep` against the immutable tree. Current repository instructions and scoped project governance were read separately.
- No build, test, lint, formatter, generator, schema check, harness, workflow dispatch, local validation, htch-runtime validation, source edit, or Git mutation was performed. The original exact-H `d70f86f33b5e3c67545954b270373e1548809f4b` Actions run `41578381556`, Linux job `125727154700`, is supplied failure provenance; this reviewer did not independently adjudicate its full logs. No new execution is claimed.

## Complete changed scope

| Path | Candidate blob | Disposition |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md` | `c99e744188d90d7b31448e45e9887148c835dda6` | Read the entire changed AR-CI amendment, the complete PS-LIVE and AR contracts, original snapshot requirements, and the applicable durable-uninstall requirements. The amendment preserves the existing authority and acceptance boundaries. |
| `source/host/src/extensions/mcp/plugin_skills.rs` | `a0054eebb18e38b234689d72ca998dac3fd77901` | Read the full file, all changed hunks, the admission/query state machine, construction, refresh/persistence, retirement, disposal, publication/pruning, status and sync/coalescing paths. |
| `source/host/src/extensions/mcp/plugin_skills_sync_tests.rs` | `cb8abb868bef2469bded2dda87e0680e5c54b329` | Read the complete changed poisoned-reader test and its real fixture/helpers, same-root/different-root handoff, stale import/staged publication, old-prune, disposal, panic/recursion/reporter and pending-uninstall controls necessary to judge this change. Unchanged unrelated test bodies were not represented as a complete-file read. |
| `source/host/tests/mcp_coordinator_relay_contract.rs` | `71144e4e578a51fd2281dcb6b743e5c7a11d569e` | Read the complete changed desktop refresh/status test and actual relay/service fixture and adjacent lifecycle controls. Unchanged unrelated test bodies were not represented as a complete-file read. |

Current `AGENTS.md`, including its independent frozen-review and execution rules, and OCRM-EXEC-02 §17 were read. The project source-of-truth amendment and M16/M17 module ownership were checked for relevant authority; historical project documents were not promoted to current implementation or acceptance evidence. The complete frozen diff was reviewed; adjacent deep reading was selected by the concrete risks rather than sampling changed hunks.

## Security and lifetime assessment

The altered helper at `plugin_skills.rs:741`, `fence_owned_refresh`, handles only admission-lock poison differently. A poisoned `PluginSkillsLiveAdmission::is_denied` still returns true (`:426–439`). The helper neither obtains the poisoned inner value nor clears poison, resets facts, substitutes a fresh reader, writes persistence, invokes a clock/loader/reporter, nor performs cleanup. Its healthy branch still checks the refresh owner's Arc identity before marking the exact current custody fenced. A late repeated disposal of the predecessor therefore cannot fence its successor's token.

`retire_for_replacement` (`:883–904`) still acquires publication then service state; publication/state poison remains an error before the claim. A duplicate claim remains an error. It records `disposed` and `replacement_claimed`, clears pending work, wakes waiters, releases service state, and then fences admission while holding publication. Returning success after admission poison now accurately reports that this one-shot retirement completed. The previous behavior could consume the claim and return an error, leaving Host unable to construct its required successor. The recovery does not turn unavailable admission into authority.

`successor_with_loader` (`:871–881`) retains the same admission Arc through `with_admission`, which creates only the successor's local service/publication/refresh-owner state and performs no recovery I/O. `PluginSkillsBinding::prepare_service` retains the new owner before later fallible Session binding. Same-root binding is idempotent by reader identity; different-root replacement is guarded by the existing registered-Session root checks. Fresh disk recovery remains outside the Host binding lock. The changed helper introduces no callback or I/O under that lock.

The successor's `begin_refresh` checks poisoned admission before intent construction/persistence and before `run_pass` can call the loader. Uninstall reservation likewise fails at the unchanged admission read. Retired active results cannot import new refusals or rename/release a snapshot because the publication-guarded active-owner checks remain. Old refresh guard cleanup cannot acquire poisoned admission; the poison itself continues to enforce denial. In the healthy path its owner/token checks and the existing fenced disposition prevent it from changing successor custody. Publication/prune ordering and callback placement are unchanged.

Retained readers remain effective in service `current` and all successful sync returns, including coalesced/disposed returns. The Session forwarding reader clones its delegate and releases its slot before querying it; FileWorkflowStore still checks that shared reader on plugin list/get/update admissions. Poison consequently cannot be converted into an empty refusal list by these consumers. Raw cache/status diagnostics remain separate from execution admission.

The private-image and cache publication code was read to verify that this recovery does not route around durable refusal/unknown handling. Exact expected/candidate-byte readback, strict image decoding, persistence errors, negative-before-content ordering and positive release after cache publication/readback remain unchanged. The new branch is confined to no-I/O lifecycle retirement; it is not an error suppression path for persistence, refresh, uninstall or publication.

## Oracle assessment

The poisoned-reader test (`plugin_skills_sync_tests.rs:1730–1768`) poisons the actual private mutex while holding its guard. Its existing same-Arc successor construction remains decisive: it would fail at the formerly erroneous retirement return. Existing retained/current denials, raw retained cache, refused sync/uninstall and unchanged cache-byte assertions remain. New assertions pin continued denial for both IDs, the fixed `authRecovery` poison projection and a rejected second retirement claim. No retained assertion was removed or weakened.

The added `h.assert_calls(1)` observes the predecessor Harness loader, not the successor's distinct StaticLoader. It should not be cited as a direct measurement of successor loader calls. The successor pre-loader refusal is established here by source tracing plus the exact returned poison error; the narrow required AR-CI oracle remains satisfied. This is an evidence-description limit, not a defect in the changed implementation or a reason to weaken the gate.

The desktop test (`mcp_coordinator_relay_contract.rs:852–917`) still uses actual McpHostService dispatch, CoordinatorPluginSkillsLoader and relay request resolution. It verifies waiting before actual loader completion, real record projection, error propagation, cache-byte retention, and closed-service refusal. The changed expectation remains an exact whole-object equality, with unchanged `authBlocked` and the actual known-refusal projection `authRecovery.refusals` plus `refreshOutcomeUnknown: false`. This follows AR-08 and the production `PluginSkillsPort::status` implementation; it does not mask an implementation error with a partial-field assertion.

Healthy same-root success, distinct owner/token protection, stale loader and staged publication refusal, old-prune ordering, ordinary disposal cleanup, queued wakeup, panic/recursive sync and reporter reentry controls were read as retained source controls. Their existence and unchanged assertions are source facts; their current-candidate runtime outcomes remain unverified.

## Adjacent source identities and read limits

All identities below are from the candidate tree. Reads were scoped to the named responsibility except the full production private-image implementation.

- `plugin_skills_uninstall_state.rs`: `0c2a4f7fd11c6b47dc9941b659ad8eac89200c9a`; production schema/decoding/encoding/load/exact-image validation/staging/publication/cleanup, lines 1–500. Test instrumentation and other tests were not treated as fully read.
- `plugin_skills_cache.rs`: `f395533056fd0203a68a51c51a412ff835925001`; strict snapshot reader and staged publication/cleanup, lines 180–330.
- `plugin_skills_auth_recovery_tests.rs`: `504eba599ff6684c8f0688698331a2637d7b4853`; integrity-fault and old-loader/guard/repeated-dispose successor-token controls, lines 520–680.
- `mcp/production.rs`: `fb672e8c3b9473607438a0db4fc733e205d5c59b`; service/port delegation and actual status projection, lines 265–323.
- `mcp/mcp_service.rs`: `e903112e400783e70e9735b41745223a0f29f06c`; lifecycle active check and desktop refresh/sync/status dispatch, lines 247–311.
- `host_production_extensions.rs`: `36b5dbac27d346f883150df179372034e0bea223`; PluginSkillsBinding and preparation, lines 470–581; production MCP startup/stop, lines 1333–1444. Other displayed surrounding code was not promoted to a complete-file review.
- `session/production.rs`: `c5912cab4f95d3a75fc13cb264a21b1deef9a733`; forwarding query and workflow-store construction/idempotent binding, lines 235–251 and 1494–1517.
- `workflows/workflow_store.rs`: `048f7948db3d31d1a057902f956b3f1db791dad7`; reader injection/query, plugin projection/list/get and publisher/update admission, lines 149–161, 339–398 and 577–600.

No newly identified finding requires a code fix. Exact-candidate Actions verification and all broader outstanding acceptance obligations remain required and are not advanced by this approval.
