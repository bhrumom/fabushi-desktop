# Hermes-informed Bot runtime completeness

Status: active implementation, 2026-10-21. This is a continuation of the user's
request to compare the complete Fabushi Bot design with Hermes, absorb suitable
designs, and implement missing responsibilities. It does not replace the current
Rust Runner, RuntimeStore, capability/authorization, conversation or message owner.

## Authority and scope

Fabushi starting source: `26ec2ebb7159b21b0382a8c6d46d3534a5cdfab4`, containing
the MiniApp authoring PR #166. This follow-up uses a separate dependent branch.
Hermes reference: NousResearch/hermes-agent
`40cc180823d182826536416de2fc27be44b4ac57`.
Root AGENTS.md and the current canonical Bot architecture remain authoritative;
the old PR/version snapshots in post-1.2.74-agent-architecture-cutover.md are
historical, while its Rust-owned execution/state and single-owner boundaries apply.
Phase model/effort: inherited session configuration; no override applied.

Compare responsibilities, not file counts or Python syntax. Existing implementations
must be inspected before calling something missing. UI channels, provider brands
and Hermes-specific deployment services are compared as capabilities and adapted
to existing Fabushi owners; they are not new parallel products. Mac remains the
first delivery target. Never enable paid-model tests as a side effect of this work.

## Requirements and acceptance

- HRA-01: retain the truthful resource diagnostics and observational (not turn-
  cancelling) repeat advice already added in #166. Normal model completion remains
  authoritative subject to existing user cancellation, approvals and real budgets.
- HRA-02: persist a tool-call execution boundary through the existing provider
  checkpoint owner before dispatching effects. A failed checkpoint prevents
  dispatch. A recovered unresolved boundary must not silently reissue effects or
  request another model response from stale history; report uncertainty and require
  reconciliation. Cover Fabushi/Codex Responses, Cursor and OpenRouter adapters,
  batches, persistence failure and backward-compatible completed checkpoints.
  Pending execution disarms the model first-output watchdog; only a completed
  tool boundary starts the next model-response deadline. Tool duration is not
  evidence that a model stream stalled.
- HRA-03: oversized output must have explicit byte bounds, valid UTF-8 and truthful
  truncation evidence. Never present a capped artifact as a complete result. A
  failed materialization must not replay unbounded output into model context or
  cause an uncertain mutation to be retried automatically.
- HRA-04: background completions preserve originating conversation/task identity,
  routing and acknowledgment. Repeated observations do not create duplicate user
  replies. Completion promises require an actual delivery path. Existing bounded
  Host waits/AwaitShell own waits, not repeated model turns. Worker spawn failure
  must release the claimed watch and publish no durable pending marker. A start
  gate orders pending publication before terminal completion. Shell results expose
  whether automatic completion delivery was registered; notification failure does
  not imply that the backend process failed and must not rerun it. Inject worker
  spawn failure through the existing watch owner, not a replacement scheduler.
- HRA-05: verify completion/cancellation/empty-output and partial-stream semantics
  across the production provider adapters. Automatic transport/recovery attempts
  are separate from model-directed business retries and remain bounded.
- HRA-06: assess context compression, usage accounting, budgets, duplicate results,
  memory/skills, plugin discovery, authorization and subagent lineage/lifecycle.
  Fix concrete defects in current owners and add regression oracles. Do not adopt
  a fixed interactive failure count, a new credential store or another scheduler.
- HRA-08: malformed or non-object provider tool arguments must never become an
  empty object and reach an effect-capable tool. Return an explicit tool error to
  the model, keep the call/result pair balanced, allow a corrected call, and do
  not perform an automatic model retry. Preserve per-tool semantic validation.
- HRA-09: report confirmed Responses usage once per received request usage event,
  including a later tool/checkpoint/stream failure. A resumed aggregate is history,
  not new spend. Absent provider usage remains unknown; do not invent estimates.
  Both Fabushi first-party and Codex paths use the same reporting implementation.
- HRA-10: extend observational repeat feedback to provider tool results, including
  MCP success envelopes carrying `isError: true`. Preserve original result data;
  null/false/zero/empty error fields and policy refusals are not execution failures.
  Advise once for the second identical tool/arguments/failure; changed arguments,
  diagnostics and a successful matching call are separate observations. Bound
  hashed per-turn state; no extra requests, forced stop, or approval bypass.
- HRA-07: durable evidence maps each audited responsibility to exact Hermes source,
  current owner, observed behavior, gaps, changes and Actions evidence. Unknown,
  blocked and not-run are not complete. All executable checks run in Actions;
  source edits, read-only source inspection and Git/API operations run locally.

## Responsibility audit (live task ledger)

| Area | Hermes source | Existing Fabushi owner | Finding / state |
|---|---|---|---|
| Turn/final messages | agent/conversation_loop.py, turn_final_response.py | extensions/inference provider loops; Runner production_turn_agent_owner | No-tool final ends normally, including a legitimate empty final after tool delivery; no automatic empty-response loop imported |
| Error truth/replay advice | tool_guardrails.py, tool_result_classification.py | ports/box; Runner box_tool_access | Truthful resource errors in #166; all provider tool-result repeat feedback added here; Actions 39553703887 passed |
| Persist before effects | turn_tool_round.py | provider checkpoints; turn_agent_composition | Implemented pre-dispatch pending checkpoints, uncertain-resume refusal and watchdog phase boundary; Actions 39553703887 passed |
| Artifact/context bounds | tools/tool_output_truncate.py, tool_result_storage.py | Runner large_output_spill, context projection | Implemented UTF-8 byte bounds, explicit partial artifacts and bounded fallback; Actions 39553703887 passed |
| Background wait/notify | tools/process_registry_wait_policy.py | background_work, shell_terminal_watch, AwaitShell, pending wake owners | Generation/parent fencing, dedupe and settlement ordering retained; worker spawn failure now rolls back without ghost pending; Actions 39553703887 passed; Mac rearm integration verification pending |
| API retry/cancellation | turn_retry_state.py, turn_api_error.py | stream_attempt; routed provider runtime | Bounded transport retry and one auth refresh retained; uncertain effects no longer eligible for model/tool replay; Actions 39553703887 passed |
| Context and compression | context_compressor*, turn_preflight* | conversation_state; generated history projection | OPEN: summary facade/config exists, but no production preflight or mid-loop summarization invocation was found in current routed path |
| Usage/budgets | iteration_budget.py, turn_iteration_prep.py | turn_usage, usage sink, provider step limit, subagent runtime | Per-request Responses accounting repaired; OPEN: explicit root/child shared quota and admission policy not established |
| Multi-agent execution | tools/delegate_tool*, agent/subagent_lifecycle.py | subagent_runtime, task tools, durable handoffs | Parent/root lineage, live-ID dedupe, steer/abort and depth fencing present; Actions 39553703887 passed; shared admission remains open |
| Authorization/tools | agent/tool_executor.py, managed_tool_gateway.py | capability broker, auto_review, toolset fences | Role/capability and review fences retained at invocation; malformed arguments no longer default to {}; Actions 39553703887 passed; universal JSON-Schema validation remains open |
| Memory/skills/plugins | memory_manager_tools.py, skill_preprocessing.py, tools/mcp_tool* | sand_memory, skill/plugin/MCP owners | Bounded recall, extraction/dedupe, skill budgets and installed-plugin identity/auth projection retained; focused contracts passed 39553703887 |
| Session ingress/delivery | gateway/run_turn*, session_* | Rust conversation actors, RuntimeStore, common transcript | Blob reference validation and per-conversation recovery owners retained; contracts passed 39553703887; installed recovery acceptance not run |

## Verification and delivery

Use deterministic transports/resource ports to inject failures at boundaries;
assert actual dispatch counts, exact pending identities, recovery refusal, valid
UTF-8/byte counts and original ownership. Fixtures do not claim real-model or
installed-device acceptance. Carry forward applicable existing production-contract
tests rather than introduce a second validation runtime.

Review every requirement before completion. Integrate only accepted source;
signed Mac delivery and installed acceptance remain separate evidence layers.
Rollback restores the prior behavior without deleting user work or uncertain-call
evidence. Source completeness cannot be claimed while this ledger has open gaps.

## Remaining implementation gates (not completion)

The current comparison is not a claim of whole-product parity. The following
concrete gaps remain open after the immediate safety/cost fixes:

1. Context management: bind the existing summarization service to the production
   preflight and completed-tool boundary. Persist summaries/archives through the
   canonical Agent checkpoint owner, retain system/authority and the newest user
   request, preserve balanced tool pairs and pending effect identities, and never
   repeatedly pay for a summary after the same failed compaction. Test summary
   failure, cancellation, recovery and subsequent request size deterministically.
2. Root/child resource policy: retain model-directed stopping. A configurable real
   quota/admission policy, when supplied by the existing settings/budget owner,
   must account for descendants atomically and release admission on every terminal
   path. No new fixed interactive failure threshold or parallel quota ledger.
3. Invocation validation: the new parser rejects malformed/non-object JSON. It
   does not claim full JSON-Schema validation for every MCP schema. Reuse the
   canonical validation/authorization boundary; cover schema errors before effects
   and preserve truthful feedback without acquiring new grants.
4. Background worker loss: spawn failure is repaired. An await callback panic or
   disappearance after registration still needs typed watcher-failure handling,
   preserving the backend job and recovery marker rather than inventing a job
   completion or rerunning it.

Mac compilation, signing, installation and real-model MiniApp authoring/GitHub
acceptance are separate delivery gates. Real-model acceptance remains stopped
following the user's token-consumption complaint. All current Actions checks use
fake transports/resource ports and do not call paid models.
