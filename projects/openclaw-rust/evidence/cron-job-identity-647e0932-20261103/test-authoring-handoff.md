# Cron job identity tests handoff

## Scope and authority

- Base H: `972779ecd290b90e407ffe651aa5c456c31e9f7f`.
- Base tree: `a75bb4706831a34b818e459f6be4df920560a2fe`.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`.
- U tree: `90c8aab39972da11cfad220361781d49e1bc6700`.
- Contract read completely before authoring: `projects/openclaw-rust/docs/specs/cron-job-identity.md`, `OCRM-CRON-JOB-IDENTITY-v1`, including the implementation-before-code `CJI-ID-NAMESPACE-01` amendment.
- Stage: xhigh production-oracle authoring as requested; actual host model/effort inheritance retained. No prompt-only effort override is claimed.
- Only repository file written: `source/host/app/src/tests/cron_job_identity_tests.rs`.
- No main.rs, workflow, ledger, RTM, spec or other source was changed by this worker.

## Actual tested production seam

The five tests call the private App production functions imported through `super::super`:

- `list_openclaw_cron_jobs(&SandAgentSessionStore, &Value)`.
- `trigger_openclaw_cron_job(&SandAgentSessionStore, &Value, run_now)`.

Fixtures exclusively create their own UUID/PID-qualified temporary root. They create actual Agents through `SandAgentSessionStore::create_session`, and routines through the existing `dispatch_automation_command("createAgentAutomation", ...)` producer. Every positive trigger asserts the dispatch's actual `agentId` and local `automationId`, then executes that supplied pair through `AutomationRuntime::run_agent_automation_now_with`. Only its terminal executor result is controlled. The test reads real Running history at executor entry, actual terminal or retained history afterward, and actual `AutomationRunObservation` reports.

Non-target and refusal controls preserve the exact configuration/history bytes, including missing-history/deleted-file states. Prior history is established through real native execution, not a copied job registry. `Owners::drop` disposes the Runtime and shuts down owned Session workers before `Fixture::drop` removes only its exclusively claimed root. Reopen drops the old owners first and reconstructs them over the same files.

## Named cases and acceptance mapping

| Test | Contracts and causal evidence authored |
| --- | --- |
| `cron_job_identity_list_and_trigger_keep_same_name_agents_distinct` | CJI-01/02/03/07. Two native same-name routines have the same local slug and distinct existing-algorithm stable IDs. List fields, pagination and newest-history projection are checked. Both jobs execute in reverse listing order using both wire aliases despite misleading agentId values. Each response run ID matches the real newly admitted history; previous rows and the other Agent's config/history bytes remain unchanged. Native IDs remain local. |
| `cron_job_identity_legacy_aliases_refuse_ambiguity_without_effects` | CJI-02/03/04. A unique ordinary legacy slug succeeds with canonical response; the same slug in a second Agent makes it ambiguous. Missing, blank, non-string, unknown and id-precedence inputs produce BadRequest, zero dispatch, zero observations and byte-identical files. A canonical id retains precedence over the jobId alias and ignores a misleading agentId. |
| `cron_job_identity_canonical_legacy_collision_refuses_without_effects` | CJI-02/04/05 and CJI-ID-NAMESPACE-01. Native creation produces a real local slug equal to another job's canonical UUID5. Joint matches refuse both aliases without effects. Deleting the original canonical owner through the native runtime still refuses its old UUID, even when only the legacy collision remains. The survivor executes through its own freshly listed canonical ID and cannot recreate the deleted files. |
| `cron_job_identity_reopen_and_delete_preserve_target_continuity` | CJI-03/04/05. Real owners close/reopen with identical public IDs and unchanged file bytes. The selected Agent still executes. Native deletion makes its old canonical ID fail without touching the other same-name routine, which remains usable through its now-unique ordinary legacy slug. |
| `cron_job_identity_existing_execution_outcomes_remain_owned` | CJI-03/06/07. Separate routines exercise Completed, executor Err, reported Failed, Interrupted and RetainedUnsettled. Terminal statuses/details and exactly one existing-owner observation match real history. Uncertainty preserves the exact Running admission bytes and publishes no terminal observation. A direct native dispatch error propagates once with no retry, new history or observations. Unrelated and previously exercised routine files remain unchanged. |

The first test also checks the live compiled method registry/owner and bounded source anchors for the existing `UnifiedGatewayApi` handlers, their calls to the two tested production functions, and the `self.call("runAgentAutomationNow", ...)` port. This is supplementary wiring evidence, not a full HTTP/desktop execution claim.

## Source reading and validation status

Exact-H API reads supplied the Runtime, RunPath, FileAutomationStore, stable ID and existing native-runtime test contracts. Local H/U Git objects were unavailable and the mutable repository source was not treated as the authority. The integrator's isolated candidate main.rs was read only to reconcile the new function signatures and source anchors.

No compiler, formatter, linter, generator, parser, validator, test or harness was run locally or on htch-runtime. Read-only source inspection and Git diff/identity operations are not execution acceptance. All five tests are authored, **not run**. The integrator owns module registration, independent blind review, the frozen candidate, API publication and GitHub Actions evidence. The change does not establish full cron.run/hooks interoperability, live provider/backend behavior or any global completion flag.

The diff beside this handoff is an exact addition against `/dev/null`; its Git diff exit code 1 means a new-file difference, not a test failure.
