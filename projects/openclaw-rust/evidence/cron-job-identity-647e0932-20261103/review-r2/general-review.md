# Cron job identity — Round 2 independent general review

## Verdict

**SOURCE-ONLY APPROVE.** No concrete correctness, security, lifetime, oracle, or composition defect was established in the frozen seven-path cron delta against the exact integration base.

This verdict is source review only. It does not establish compilation, test success, CI admission, product verification, release readiness, or acceptance. The declared exact-published-H Actions work remains required.

## Review identity and independence

- Review lens: fresh general/correctness/security/lifetime/oracle integration review.
- Actual launch effort: **xhigh**, as configured by the controller. The scope combines cross-Agent identity, durable storage, actual execution dispatch, retained unknown outcomes, and concurrent integration.
- Model: unchanged configured model, inherited without an override.
- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Reviewed immutable tree: `215733e43f8016e159ef1d676e433138272f6dc6`.
- Exact integration base: commit `71b039ae860c45e982dfbc8b1f401143d49d2454`, tree `3c30ec8ac78d0b8b549267a4b794a7cb177b8b23`.
- Original cron base: commit `972779ecd290b90e407ffe651aa5c456c31e9f7f`, tree `a75bb4706831a34b818e459f6be4df920560a2fe`.
- Prior immutable cron snapshot: `f5e577cf9e119f417ce0cf071f03c214c5bf6190`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Contract: `OCRM-CRON-JOB-IDENTITY-v1`.

I read no other reviewer report, prior runner decision, or integration-decision document. I did not use a mutable checkout or branch as the review subject, and did not substitute the later independent PRID/evidence commit for this tree. The supplied manifest, complete-scope patch and integration-delta patch were the review inputs; the original complete cron patch was retained as context and compared textually with the Round 2 complete patch. That comparison changes base identities, offsets and surrounding incoming context, while retaining the original cron production algorithm, tests, spec and source contract.

## Declared scope read

The immutable Git comparison from the integration base lists exactly these seven changed paths:

| Path | Reviewed change |
| --- | --- |
| `source/host/app/src/main.rs` | Actual list/trigger functions, App wrappers, test-module registration; adjacent Gateway and native run dispatch; preservation of the incoming SendMessage path |
| `.github/workflows/rust-desktop-runtime.yml` | Source inputs, both receipt field blocks, shipping/test feature capture, exact App executable and case selection, surrounding failure/receipt handling |
| `source/host/app/src/tests/cron_job_identity_tests.rs` | Complete 679-line real-owner fixture and all five required tests |
| `projects/openclaw-rust/docs/specs/cron-job-identity.md` | Complete 246-line candidate spec, including CJI-01 through CJI-07 and CJI-ID-NAMESPACE-01 |
| `projects/openclaw-rust/evidence/cron-job-identity-647e0932-20261103/source-contract.json` | Complete original-base source/owner/oracle contract and its explicit pending acceptance limits |
| `projects/openclaw-rust/parity-ledger.json` | Narrow cron evidence references, affected Gateway responsibility context, preserved incoming media additions and completion flags |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Narrow R04/AC11 cron references, preserved incoming media references/oracles/blockers and completion flags |

The complete root `AGENTS.md` was read before judging. Related authority/context read included the OCRM Source of Truth, applicable module-map ownership, and the governing scope, ownership and Revision 3 §17 execution requirements in the main OCRM spec. There is no nested `AGENTS.md` governing these seven paths.

Adjacent production reads covered `stable_automation_id`; `SandAgentSessionStore` enumeration and automation projection; `ProductionSessionWorkers` fixture construction, enumeration, automation-store access and shutdown; `FileAutomationStore` native creation, local IDs, history ordering and storage; `dispatch_automation_command`; `AutomationRuntime` manual execution and disposal; `AutomationRunPath` admission, execution, terminal history and retained-unsettled handling; and the Gateway registry and HTTP authentication/operator/scope dispatch. The exact App Cargo manifest was read for its existing UUID dependency and distinct normal/dev Host feature declarations.

For upstream provenance, I independently obtained the five bounded fixed-U slices identified by the source contract: `jobs.ts` 171–230, `ops-read.ts` 1–100, `cron-job-access.ts` 126–235, `cron.ts` 501–625, and `register.cron-simple.ts` 251–280. Returned blobs match the source-contract identities. These establish the narrow selected-job identity/consumer contract; they do not establish full upstream cron compatibility.

All repository source reads used `GIT_NO_LAZY_FETCH=1` with immutable objects. Missing local objects were read through exact GitHub blob/commit APIs, including App Cargo blob `a2c58167a82657194b79aaf3e1e0d55577b2221c` and Session production blob `c5912cab4f95d3a75fc13cb264a21b1deef9a733`. No implicit fetch or source mutation was used.

## Assessment

### Target identity and dispatch

`list_openclaw_cron_jobs` derives both wire ID fields through the existing stable mapper using the exact Agent/local-routine pair. The surrounding schedule, prompt, enabled state, pagination and newest-history projection are preserved from the original method. No local storage identity or cloud mapping is rewritten.

`trigger_openclaw_cron_job` preserves the existing `id`-before-`jobId` input precedence, enumerates through the same current owners, and records one selected pair. A record matching both its canonical and local ID is counted once because the match is one branch per record. Any second matching record refuses before invocation. The canonical/local namespace collision therefore cannot choose an Agent by enumeration order.

The existing stable mapper emits version-5 UUID-shaped IDs. The added UUID recognition reserves that namespace when the selected match is only a local alias, so deleting the canonical owner cannot reinterpret its former ID as a different routine's unique local alias. Supplied `agentId` data never participates in resolution. Errors returned by the enumeration APIs propagate before the execution port is invoked.

The App's registered `cron.list` and `cron.trigger` routes still enter the extracted production functions. The trigger wrapper invokes `self.call("runAgentAutomationNow", run_args)` with only the resolved Agent ID and local automation ID. The existing native run dispatch accepts those fields and retains its routed Runner/Host selection. Observation uses the same Agent/local pair and returns the selected canonical `run.jobId`. The closure is `FnOnce`, and its error is propagated without retry or replacement-target selection.

### Security, lifetime and outcome custody

The changed resolver adds no permission grant, arbitrary-path access, global registry, durable alias map, scheduler, background worker or new state owner. Local execution IDs come from existing surfaced records. Gateway authentication, operator-only method classification and mutation scope checks remain upstream of App dispatch and are unchanged by this delta.

The new resolver owns only call-local values. The pre-existing runtime remains responsible for admission, duplicate suppression, history settlement and uncertain effects. `RetainedUnsettled` continues to preserve the running history and suppress terminal observations; the adapter reports the actual newly observed row rather than manufacturing completion. Existing concurrent before/after result-correlation limits and the nontransactional lookup-to-invocation boundary are explicitly retained by the candidate spec, and this review does not claim they were repaired.

The test fixture exclusively creates its root, uses actual Session/AutomationRuntime owners, and disposes/shuts down those owners before deleting only that root. Reopen drops the same owners before creating fresh owners against the retained files. No global environment mutation or broad process/path cleanup is added by the cron tests.

### Oracle and required cases

The five cases invoke the actual extracted production functions. Native automation commands create and delete the routines, and the trigger test port executes the pair supplied by the adapter through the real `run_agent_automation_now_with` method. It does not replace a dispatched pair with the test's expected pair. Only the final external execution result is controlled.

The cases cover reverse-order selection of same-name routines through both aliases, canonical response and real run identity, native/local ID preservation, unique legacy compatibility, ambiguity with misleading Agent fields, canonical/local collision before and after deletion, reopening, deletion continuity, and completed/error/interrupted/retained-unsettled outcomes. Refusal controls assert zero execution-port entries and unchanged config/history bytes. Non-target routines and prior history are explicitly preserved. A direct dispatch-error control verifies one attempted dispatch with the original error and no invented run history. The shipping-source assertions supplement these behavioral controls and correctly disclaim full HTTP/App construction.

The workflow includes all five exact `tests::cron_job_identity_tests::...` names. Its existing selection path lists the exact test name, invokes that same exact case, requires one-passed/zero-failed/zero-ignored summaries, and records the case only after success. The receipt binds the actual executable, source input hashes, command/log hashes, H/tree/U and Actions run/attempt. This is sound source-level preparation for the required execution evidence, not evidence that the cases have run.

### Concurrent integration preservation

Against the exact `71b039...` integration base, the production `main.rs` delta is confined to the cron functions/wrappers and cron test registration. The incoming `ProductionSendMessageSink` media state machine, `AgentDbProjectionError` import, per-identity custody entries, unknown-outcome propagation and publication boundary remain present and unchanged. I reviewed their preservation during composition; I did not perform or claim a complete independent review of the media implementation batch.

The composed workflow retains the incoming Host feature observation function, shipping `cargo check --message-format=json`, App test-build artifact capture and the separate expected `test-support` modes. The exact App manifest declares normal Host dependencies without `test-support` and enables it through the existing dev dependency. Cron introduces no dependency or feature change.

Both source-input sets remain in the App receipt. Both `media_settlement_*` and `cron_job_identity_*` receipt fields remain distinct and present, as does the recorded SQLite environment. The five cron cases are appended alongside the eight exact media settlement cases. The two incoming SendMessage bridge controls also remain in their existing integration-contract requirement list. No prior requirement list or feature observation is replaced by the new cron block.

The cron ledger/RTM changes append only the three narrow evidence references. The incoming media responsibility, its unresolved-equivalence and exact-H blockers, its MSU oracle references and its existing status remain preserved. No source review row, unit verification state or acceptance state is promoted by this integration. All eight global completion flags remain false. The spec/source-contract original-base and round-1 fields remain historical design provenance, not purported execution evidence for the Round 2 tree.

## Unknowns and execution boundary

- No build, test, lint, formatter, generator, parser, validator, benchmark or harness was executed locally, on htch-runtime, or in Actions by this reviewer. No execution acceptance is inferred from source reading.
- Compilation, all required positive cases and regressions, actual feature observations, and receipt generation must still be established against the exact published candidate H in the allowed Actions environment.
- The new fixture controls external execution results and does not exercise a live provider/backend, the complete desktop/HTTP construction, or packaged platforms.
- Complete upstream `cron.run` access/grant/queue/wait compatibility, broader scheduler/timezone/reaper behavior, global dependency/source closure and final G01–G12/AC01–AC24 acceptance remain outside this narrow repair and remain open as declared.
- Existing fail-soft display enumeration and concurrent run-result correlation remain the explicit pre-existing limits of this seam; the review does not upgrade them to transactional or complete filesystem authority.

## Findings

None established. There is no requested source fix from this review.

Only this report was written. Reviewed repository source and review inputs were left unchanged.
