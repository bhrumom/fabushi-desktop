# Cron job identity through the existing Automations owner

Contract: `OCRM-CRON-JOB-IDENTITY-v1`
Status: implementation authorized; source review and exact-candidate Actions pending
Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1
Implementation base: `972779ecd290b90e407ffe651aa5c456c31e9f7f`
Base tree: `a75bb4706831a34b818e459f6be4df920560a2fe`
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`
U tree: `90c8aab39972da11cfad220361781d49e1bc6700`

## Problem and user behavior

The public Host `cron.list` adapter aggregates routines from every current
Agent but exposes each Agent-local automation folder ID as the global job ID.
`cron.trigger` scans Agents and runs the first matching local ID. Creating
`Daily check` separately in two Agents naturally creates two `daily-check`
folders. Selecting the second list entry can therefore execute the first
Agent's prompt and record history in that wrong Agent.

This is a concrete target production defect. Both methods are registered in
`host_gateway_api.rs` and reached by the authenticated HTTP `/api/<method>`
route through `UnifiedGatewayApi`. The trigger's optional-looking `agentId`
input has never been a selector or authorization grant; current code ignores it.

## Existing ownership and scope

The Rust App owns this thin external protocol projection. Existing
`SandAgentSessionStore` / `ProductionSessionWorkers` and
`FileAutomationStore` own Agent and routine data. The existing
`AutomationRuntime` / `AutomationRunPath` own admission, execution,
cancellation custody and history settlement. The App's existing
`runAgentAutomationNow` dispatch selects the configured routed Runner or
existing Host lane. This change creates no scheduler, permission registry,
session store, durable alias map, runtime or new product module.

Reuse `automations::automation_id::stable_automation_id(agent_id, local_id)`,
already used by cloud definitions, backend fire delivery and run telemetry.
Keep local folder IDs and native `agentId + automationId` APIs unchanged.

The current target registers `cron.list` and `cron.trigger`. Fixed U uses
`cron.run` with additional access, queue and wait contracts. This repair
absorbs only unique list-to-execution identity; it does not claim complete
upstream cron wire compatibility or excuse the remaining methods.

Allowed production changes are the two existing handlers and private
same-file production functions in `source/host/app/src/main.rs`. A dedicated
App test file may be added. The integrator serially owns the existing workflow,
this spec, exact-source/evidence records and narrow references in the unique
OCRM ledger/RTM. No unrelated implementation or shared source rows are changed.

## Requirements

- **CJI-01 — canonical list identity.** Every listed job's `id` and `jobId`
  are the existing stable ID of its exact durable Agent/local-routine pair.
  Preserve `agentId`, schedule, enabled state, prompt, pagination and newest
  history projection. Two same-name routines in different Agents remain
  independently addressable.
- **CJI-02 — one resolved target.** Resolve the input from the existing
  `id` / `jobId` alias precedence. Enumerate the actual visible owners and
  select the exact Agent/local-routine pair before any run dispatch. Match the
  canonical stable ID or a legacy local ID, counting each pair only once.
  A unique legacy ID outside the reserved stable-ID namespace remains
  compatible. UUID version-5 inputs are reserved for canonical identities;
  a match only to a legacy local ID in that namespace is refused and the
  caller must obtain that routine's own canonical ID from a fresh list.
  More than one matching pair is
  `BadRequest`, even if one match is canonical and another is a legacy local
  ID. This conservative cross-namespace refusal prevents a historical alias
  from silently selecting another Agent. No first-match precedence may turn
  ambiguity into an execution. Enumeration errors propagate without dispatch.
- **CJI-03 — local execution and canonical response.** Invoke the existing
  `runAgentAutomationNow` once with the resolved `agentId` and **local**
  `automationId`. Read before/after run history using that same pair. Return
  canonical `run.jobId`; retain the current real observed run ID, status,
  detail, accepted/finished and timestamp behavior. Never pass a stable ID as
  a local folder ID, synthesize completion, retry or choose another Agent.
- **CJI-04 — refusal has no execution effect.** Missing, blank, unknown,
  deleted or ambiguous job IDs cause no run dispatch and no routine/history
  mutation. A supplied `agentId` cannot override the job identity or grant
  access. Existing Gateway authentication/operator/scope checks stay in place.
- **CJI-05 — durable continuity.** Reopening the same actual owners derives
  the same canonical IDs from the durable pair, without writing migration or
  alias state. Deleting one routine makes its old canonical ID fail; it must
  not fall back to a same-name routine elsewhere. Native Agent-local APIs,
  cloud IDs, prior history and unrelated Agents remain unchanged.
- **CJI-06 — existing outcome custody.** A dispatcher error is propagated
  without retry or a success response. Through the real AutomationRuntime,
  completed, execution-error, interrupted and retained-unsettled results keep
  their existing history/terminal semantics. This resolver does not own
  cancellation, uncertain external effects or replay decisions.
- **CJI-07 — actual production seam.** Extract the existing list/trigger
  method bodies into private same-file functions only to make the actual
  consumer boundary testable. `UnifiedGatewayApi` supplies its existing
  session store and its existing `self.call("runAgentAutomationNow", ...)`
  port. Tests execute those production functions with real file-backed owners
  and real AutomationRuntime; the external Runner/Host result is controlled.
  No copied resolver, independent example registry or test-only production
  branch can establish the behavior.

## Implementation and shared test interface

Private functions in the App's `main.rs`:

```rust
fn list_openclaw_cron_jobs(
    store: &SandAgentSessionStore,
    args: &serde_json::Value,
) -> Result<serde_json::Value, GatewayCommandError>;

fn trigger_openclaw_cron_job(
    store: &SandAgentSessionStore,
    args: &serde_json::Value,
    run_now: impl FnOnce(serde_json::Value)
        -> Result<serde_json::Value, GatewayCommandError>,
) -> Result<serde_json::Value, GatewayCommandError>;
```

The functions keep no durable or global state. Resolution retains the selected
Agent, local ID, canonical ID and previous run ID in the one call. Invocation
and observation use those exact resolved values. No new parser or ID algorithm
is added.

The ordinary existing list projection remains fail-soft where its existing
owners are fail-soft; this patch does not claim stronger global filesystem
enumeration or a transactional lookup-to-execute snapshot. Definition changes
after resolution continue through the existing runtime's admission checks.
Concurrent run-result correlation beyond the existing before/after projection
is not repaired by this identity change.

## Verification and acceptance

All executable checks run only in GitHub Actions on a frozen exact H, reusing
the current affected App/Host workflow and caches. No local or htch-runtime
compiler, test, lint, generator, validator or harness execution is allowed.

New tests live in `source/host/app/src/tests/cron_job_identity_tests.rs`.
Use exclusively created fixture roots, real Session/AutomationRuntime creation,
actual files and bounded synchronous calls. Shut down owned workers before
removing only the fixture's own root. Test the existing native creation producer
when building same-name routines; do not seed a second job registry.

Required named controls and oracles:

1. `cron_job_identity_list_and_trigger_keep_same_name_agents_distinct`:
   create two actual Agents/routines with the same local ID and different
   prompts. Assert both canonical IDs against the existing stable algorithm;
   trigger both in reverse listing order via both input aliases. Only each
   selected Agent/prompt reaches the real runtime executor, its real history
   settles, and response job/run identities match that history. Preserve
   non-target config/history bytes and existing pagination/history fields.
2. `cron_job_identity_legacy_aliases_refuse_ambiguity_without_effects`:
   a unique old local ID works and returns the canonical job ID; creating the
   same name in another Agent then makes that alias fail before dispatch.
   Missing/blank/unknown inputs and misleading `agentId` inputs do not
   redirect work. Assert actual unchanged per-Agent files and zero dispatch.
3. `cron_job_identity_canonical_legacy_collision_refuses_without_effects`:
   use the native creation producer to create a routine whose local slug
   equals another routine's canonical ID. The shared input is ambiguous;
   neither pair may execute and both stores retain their exact bytes. Delete
   the original canonical target through the existing runtime: the old ID
   must remain refused rather than redirecting to the now-unique legacy
   alias. The remaining routine must still work through its own listed
   canonical ID.
4. `cron_job_identity_reopen_and_delete_preserve_target_continuity`:
   close/reopen real owners, show stable list IDs and selected execution;
   delete one routine through the existing runtime and reject its former
   canonical ID while another same-name routine remains unchanged and usable.
5. `cron_job_identity_existing_execution_outcomes_remain_owned`:
   invoke the real Runtime through the production trigger function and
   observe its successful/error/interrupted/retained-unsettled history
   behavior on separate routines. A direct dispatcher refusal remains an
   error and causes no invented history or retry.

A bounded source wiring assertion supplements those actual behavior controls:
the registered App dispatch still enters the two production functions, and
the trigger port still invokes the native command. It does not claim to
construct or run the full desktop/HTTP stack. Retain existing native
AutomationRuntime and Gateway permission/registry regression targets.
The receipt must require each new positive exactly once and bind input source,
raw logs, command, H/tree/U and run/attempt. Source review approval alone is
not execution acceptance.

## Rollout, failure, recovery and remaining limits

This is an adapter identity change with no data migration. Existing native
files and cloud IDs remain readable. Old globally unique local IDs outside
the reserved stable-ID namespace continue to work; ambiguous old aliases
now explicitly fail rather than running the
wrong Agent. Consumers can refresh `cron.list` for canonical IDs. A deliberate
canonical/local alias collision remains refused until the conflicting local
definition is removed; no guessed precedence is safe for both namespaces.

**CJI-ID-NAMESPACE-01 — implementation-before-code decision.** An unrestricted
unique legacy fallback could redirect a deleted canonical job ID to a routine
whose local ID happens to equal that old UUID. The current stable mapper
always emits UUID version 5. Reserve that existing namespace at this adapter:
without a canonical match, never execute it as a legacy local alias. A native
routine with such a local name is retained and remains usable through its own
canonical listing ID; only the ambiguous old addressing form is refused.
Use the App's existing UUID dependency for recognition, not another ID
algorithm or durable alias registry. Keep joint-match ambiguity refusal even
when a canonical match exists, so an old local-ID caller cannot be silently
redirected either.

A crash between resolution and invocation adds no new durable state or work.
The runtime's existing admission, cancellation, unknown outcome and history
recovery rules remain authoritative after invocation. Rolling back restores
the cross-Agent first-match defect and is not an accepted recovery strategy.
Real backend/provider execution, full cron/run/hooks grants and queue/wait
parity, scheduler/timezone/reaper coverage, packaged platform UI and final
G01–G12/AC01–AC24 acceptance remain open. All eight global flags remain false.

## Source and owner provenance

Target discovery at the exact base:

- `main.rs` blob `4831ae8cf002ce64e6409f2d695b48aaa1914c45`:
  registered dispatch, list/trigger, native run/create dispatch and telemetry.
- `host_gateway_api.rs` `127b340f120f931af778783aabcf338869fe0bc8`;
  `gateway_server.rs` `2cc83cf387ae37c3b1c28663f72d9d0492e84ccd`:
  existing public routes, Automations owner and permission checks.
- `automation_id.rs` `7565275096e3351ffd18a6e55b7591e89292c534`;
  `automation_store.rs` `57f39e545e525a36de3c7e9830970623862ee1fd`:
  existing global mapping and Agent-local identity/storage.
- `automation_runtime.rs` `044f53c8eb94924168cc7dd953f3732cd70fa668`;
  `automation_run_path.rs` `a80ca919c1d05c30be29ad60e109f3a54f324e19`:
  native execution and outcome custody.

Fixed-U identity contract slices (not claims of full-file/global review):

- `src/cron/service/jobs.ts`, blob
  `780d8542ad4460b63ab39a4b2096671192853ec4`, createJob 188–224.
- `src/cron/service/ops-read.ts`, blob
  `3fd0daf829edc3c94ed052dd78478c3ce9f1bea0`, readJob 75–92.
- `src/gateway/server-methods/cron-job-access.ts`, blob
  `28a75469df97ceaef4cc137510ee8477ee07038a`, ID/access 135–229.
- `src/gateway/server-methods/cron.ts`, blob
  `35cd3e5e36e26ccefba265c4955377fff0bc5ca0`, cron.run 518–615.
- `src/cli/cron-cli/register.cron-simple.ts`, blob
  `ed18eba20a47260fba17a9b7b1b75fba372fb524`, CLI consumer 265 onward.

The fixed-U job ID selects one exact stored job; Agent/name are job attributes.
Fabushi absorbs that contract through its existing stable identity and owners.
These bounded reads do not upgrade canonical source-review rows, replace
historical read facts, close recursive dependency/LFS review, or mark the
Automations unit verified.
