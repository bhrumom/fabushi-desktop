# Persisted Fabushi model identity through the existing inference owner

Contract: `OCRM-MODEL-IDENTITY-v1`  
Status: contract frozen; implementation and executable evidence pending  
Base: `126dea3a7f077677e86f2e0fbfe73b660e11bd60`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`

## User behavior and source boundary

An authenticated caller stores a provider-bound `agentDefaultModel` through
the existing Settings/Gateway surface. The next Fabushi turn uses that exact
model in its real Responses JSON, runtime metadata and accepted checkpoint.
Changing the default affects later turns, not the active turn's tool steps or
accepted-checkpoint retry. A model selection never grants provider access or
changes the authorized route.

Current production forwards an explicitly selected Cursor-shaped model into
Fabushi Responses and otherwise selects `FABUSHI_RESPONSES_MODEL` or
`deepseek-chat`. That interim bridge cannot distinguish a legacy Cursor default
from a provider-bound Fabushi selection. Session state already stores
provider/model overrides together, but the entry point discards the provider.
Commit `5f0d548c325998ce4dcbedd1f1f04f91489df323` added the checkpoint's optional
model identity and known-model mismatch fence. This batch completes provider
binding, turn freezing, and strict Fabushi legacy-checkpoint refusal through the
existing owners; it does not add another runtime or persistence owner.

Read identities and honest full/partial extents are in
`../../dossiers/model-picker-scoped.md`. Fixed-U selection preserves resolved
provider/model identity instead of re-running aliases; provider normalization
does not lowercase model IDs. Ordinary Responses automatic continuation keeps
model in its request comparison and sends a full request on `request_changed`.
That behavior does not mean every upstream model change is rejected. This
batch's stricter accepted-checkpoint refusal is a target recovery contract:
Fabushi has no safe cross-model transformation of already accepted tool and
reasoning context. Full catalog/plugin resolution and continuation branches
remain separate required migration responsibilities.

## Single-owner production chain

Existing authenticated `config.set` / `setHostSettings` -> `SettingsService`
atomic writer and `settings.json` -> `ProductionAgentInferenceOwner` ->
`start_routed_provider_task` after route/session authorization and before turn
side effects -> existing production composition / Runner attempt -> Fabushi
provider adapter -> `CodexDirectOptions` / actual JSON serializer -> existing
checkpoint acceptance and retry owner.

The Settings owner alone reads/writes defaults. Session alone owns its existing
provider/model pair. Inference resolves the model; Runner carries the frozen
value. No new store, model catalog, route authority, Agent loop or scheduler is
introduced. React/Electron stay unchanged for this API-level behavior.

## Frozen public contract

- Existing `agentDefaultModel` retains `modelId`, `maxMode`, `parameters`,
  unknown fields and account clearing. Add optional `provider` using the
  existing canonical names: `fabushi`, `cursor`, `codex`, `claude-code`,
  `openrouter`. Noncanonical/unknown/non-string/null bindings are invalid;
  they cannot be interpreted as legacy unbound values. This target input
  contract requires canonical names; it does not claim all upstream aliases.
- Legacy unbound defaults retain their Cursor meaning. A bound Fabushi default
  is not a Cursor default. A valid default bound to another provider does not
  select a Fabushi model. No name blacklist guesses model provenance.
- Add `SettingsService::get_agent_default_model_checked() ->
  Result<Option<Value>, String>`. Under the existing owner lock it reads one
  document without normalization writes. Missing source/absent/null default
  means None; unreadable, invalid JSON/root/version or malformed present
  default is an error. Existing compatibility getters retain their contract.
- Reject an explicitly malformed provider-bound update before any settings
  commit/notification, through both whole-document and specialized writers.
  Existing handling of unbound legacy shapes and readonly/CAS/account fences
  remains intact. The checked getter must also reject malformed on-disk data.
- Add `ProductionAgentInferenceOwner::resolve_fabushi_turn_model(
  session_model_id: Option<&str>, session_provider: Option<&str>,
  explicit_model_id: Option<&Value>) -> Result<String, ProviderSessionError>`.
  Preserve raw field presence: missing/null means no explicit override for
  compatibility; any present non-string value is invalid. A present blank or
  control-containing string is invalid rather than a request for the default.
  Priority: matching persisted session pair > raw explicit turn model > bound
  Fabushi default > environment > `deepseek-chat`. A session pair with only
  one member, a blank member or a different provider fails before effects.
  When the persistent default is consulted, use the checked Settings view;
  invalid source is not absence. Explicit selections never silently fall back.
  Validate any present raw explicit model before applying precedence, so a
  malformed explicit field cannot be hidden behind a valid persisted pair.
- A selected model is trimmed, nonempty and free of control characters; retain
  case, nested paths, `@` and other model identity characters. No catalog
  availability or authorization is inferred from this syntactic check.
- Add `fabushi_model: Option<String>` to production composition/Runner-owned
  values and `fabushi_model: Option<&str>` to borrowed routed options. Existing
  constructors default to None; shipping Fabushi entry points supply Some.
  Only the Fabushi adapter consumes it. Cursor auth and its web tool model keep
  their existing inputs; Fabushi runtime metadata uses the frozen Fabushi value.
- Add `resolve_fabushi_request_model(requested: Option<&str>,
  resume: Option<&CodexDirectCheckpoint>) -> Result<String,
  ProviderSessionError>` at the existing provider adapter boundary. With no
  checkpoint, it validates an explicit selection or resolves the established
  environment/builtin fallback once. With a checkpoint, require its nonempty,
  already-normalized model; an explicit mismatch fails as Protocol before
  transport/auth/tool activity. Without an explicit value, a known checkpoint
  pins its model instead of consulting a changed environment. Missing identity
  is a refusal, not permission to replay accepted effects under a new model.
- Retain the existing `#[serde(default)] model: Option<String>` on
  `CodexDirectCheckpoint` and its capture from the actual options.model at the
  accepted tool boundary.
  Generic Codex legacy deserialization/behavior remains compatible; the strict
  missing/mismatch fence is in the Fabushi adapter. Do not erase old checkpoints.

## Lifecycle and consumers

Resolve the main turn's Fabushi model after existing provider and session
authorization. Immediately after `require_routed_turn_lease` succeeds, create
the existing `RoutedTurnLeaseGuard`, then resolve the model, before permission
`begin_turn`, attachment staging, registry admission and worker launch. Some
callers already accepted the send: a new refusal must drop that guard and
settle their existing lease, not strand it. Carry the same immutable value
through composition, every attempt,
runtime metadata and turn-memory follow-up. A later Settings commit cannot
change that value. Existing provider checkpoint tags still fence provider
identity. Cancellation remains with the existing owner and precedes dispatch;
configuration/protocol identity errors retain nonretryable classifications.

The summarization producer has no user session/turn model. It resolves the
Fabushi default through the same inference owner and must not forward its
internally generated Cursor summarization model. Other provider routes retain
their current selection and authentication behavior.

Reopening Settings must reproduce the stored default. Existing checkpoint
serde/persistence and in-memory accepted retry must retain model identity and
avoid re-executing accepted tools. The production store currently has no disk
load/resume ingress; this batch cannot claim process-restart turn resumption.
True restart ingress, invalidation/expiry and upgrade recovery remain open.

### Runner tool-effect retry custody

Contract: `OCRM-RUNNER-EFFECT-CUSTODY-v1`. This necessary recovery increment
is based on target `3ce1cd31898d090f38ba3a8c5b895da5b2de2892` and the same
fixed U above. It closes an in-process duplicate-effect path before introducing
disk recovery claims. The current adapter rearms its watchdog after a tool
checkpoint, but tool-only progress does not invalidate that checkpoint. A later
tool can finish after watchdog expiry; its stale checkpoint is ignored and the
old checkpoint, or the original input, is retried. Cancellation between two
tools can skip the next checkpoint entirely. The existing tool timeout also
returns before its detached worker finishes, so error serialization cannot
prove that the operation stopped.

The existing `StreamAttemptRuntime`, production attempt adapter and routed
executor own the control. The actual direct tool wrapper and the existing
Claude MCP server are its producers; checkpoint acceptance and the existing
retry policy are its consumers. Keep one per-run custody state with immutable
attempt bindings, started-effect revision, pending completion receipts,
checkpoint-covered revision and sticky unknown/result-loss state. No durable
schema, scheduler, second tool ledger or provider loop is introduced.

- REC-01: Before any actual tool delegate can execute, admit it against its
  captured attempt and cancellation authority and register its effect receipt.
  All direct provider tool routes and Claude's real MCP `tools/call` route use
  the same handle. An old attempt's MCP endpoint cannot admit work into a newer
  attempt; keep the existing server lifecycle bound to one immutable attempt.
- REC-02: A definitive tool return ends execution custody, but cannot make an
  earlier checkpoint safe. Started work remains uncheckpointed until its result
  is covered by a durably accepted checkpoint. New attempts do not reset this
  state. A missing checkpoint callback cannot bypass the fence.
- REC-03: Timeout, worker loss and discarded results retain an explicit unknown
  disposition independently of error text. Keep the worker's completion receipt
  alive until actual return; a late return cannot clear a recorded result loss.
  Do not claim that a timeout terminated the tool or automatically replay it.
  The real MCP producer retains that same effect's result tracker through its
  HTTP response write; a known delivery failure marks the result discarded
  before another call can be admitted. Failed non-effect requests do not create
  tool custody, and successful delivery retains ordinary endpoint behavior.
- REC-04: Capture the effect revision, persist the checkpoint, then accept it
  only if the same attempt remains current, cancellation has not won, the
  revision is unchanged and no pending or unknown effect remains. Failure,
  stale callbacks and expiry during persistence leave prior evidence intact but
  ineligible for replay of later work. Do not hold the custody lock across
  external calls, persistence, callbacks or thread joins.
- REC-05: Check custody before generic timeout/transport retry classification.
  Pending, unknown or uncheckpointed effects prohibit automatic new attempts.
  Preserve effect-free transient retries and accepted-checkpoint continuation
  when no later effect or output invalidates it. Keep the frozen provider/model
  and existing cancellation precedence. Watchdog deadlines remain bounded.
- REC-06: Diagnostic retry reports distinguish ineligible effect custody from
  retry exhaustion. A terminal response or cancellation acknowledgment is not
  proof that a detached tool ended. This increment does not claim durable
  custody after Host loss, original-turn disk recovery or external effect
  reconciliation; those responsibilities remain open.

Actions-only oracles must reach the real Responses tool loop and actual
production checkpoint/attempt owners with controlled transport and a counted
non-idempotent tool receipt. Cover C0 then tool B then watchdog expiry, no-C0
expiry, cancellation between calls in one batch, expiry/failure during
checkpoint persistence, timeout with an observed still-running worker, and a
late worker result. Assert no duplicate tool or new provider attempt and no
false checkpoint acceptance. Retain positive pre-effect retry and safe C0
continuation with the same model. Exercise the actual local Claude MCP server
and stale attempt admission; a mock executor alone cannot prove the producer
wiring. Real provider/MCP compatibility remains separately blocked on service
acceptance. Selected checks must pass on the final exact H before this bounded
contract is called verified.

Fixed-U support is bounded source review, not a new full-read claim:
`packages/agent-core/src/agent-loop.ts`, blob
`a3c3a0dc20d3a84a0d8c7fdc72b1a4cb23fae7fb`, lines 833–846 drains started
tool work after persistence failure; `packages/agent-core/src/tool-batch-completion.ts`,
blob `dab3cfd9ab2c61b8b42849557cb4905d5130335e`, lines 13–30 and 47–79
awaits started outcomes, settles them and then disposes execution entries.
Target source, control wiring and these bounded ranges are necessary authority
for this repair, not full Runner/source closure. Rollback requires reverting
the whole shared custody contract and its wiring together; retaining old raw
checkpoint files never grants them recovery eligibility.

### Downstream tool outcome custody — REC-07/08/09

Continuation base `4eb0d25ade0d06b8a467e240440a5691ec36efc8`; fixed U is
unchanged. A Host relay waiter can return while its Coordinator/Electron tool
still runs. Coordinator's control command can itself time out after writing
the request and report an ordinary remote failure. The official and legacy
MCP adapters also encode transport/authorization-change failures as the
existing `McpResult.error` value. None proves that an external effect stopped.

- REC-07: retain the existing `CoordinatorToolRelay::request` result contract,
  correlation owner, close/registration fence and callback-outside-lock rule.
  Add an effect-aware execution entry using that same implementation. An
  unsupported/closed/poisoned-registration refusal before publication is
  known not dispatched. Every execution error after publication, including
  an untyped remote failure, has an explicit unknown outcome. Discovery keeps
  its existing behavior. Do not classify localized error strings or trust a
  new caller-supplied completion flag.
- REC-08: at the routed MCP execution boundary, recognize the existing plain
  oneof `result: {case, value}` produced by the shipping structured-clone
  carrier, and the explicitly supported protobuf JSON oneof representation.
  `success` with a valid content array (or omitted protobuf default empty
  content) is a settled result, including its server-returned `isError`;
  existing explicit rejection/permission-denial/
  tool-not-found/server-not-found/approval variants keep their known-result
  behavior. A wrapper `error`, absent/malformed/ambiguous oneof or malformed
  success is unknown. A valid native MCP result with top-level content or
  structuredContent remains a result; nested user payload fields cannot be
  interpreted as wrapper authority. Preserve all known result bytes.
- REC-09: a late reply from a replaced or simultaneous tool-relay instance
  cannot settle a new instance's first request. Reuse the existing lifecycle
  relay's per-instance UUID plus non-recycling checked sequence pattern in
  this tool relay. Keep the request ID opaque to Coordinator; old opaque IDs
  remain parseable. Refuse sequence exhaustion before publication, preserve
  pending requests, and do not claim that UUID isolation terminates old work.

Carry unknown through `ProviderSessionError::ToolOutcomeUnknown` into the
existing guarded worker. Mark the same effect's sticky
`ToolEffectUnknownResult::DownstreamOutcomeUnknown` before completing its
local receipt. Provider error serialization cannot erase custody; accepted
checkpoint and automatic retry remain ineligible even when the local waiter
has returned. Map the new error to nonretryable tool/protocol failure through
existing Responses/Transcript/retry conversions, retaining cancellation
precedence. The shipping `CoordinatorRoutedToolBridge` must call this entry.
Keep its existing review gate, activity updates and list discovery. No new
runtime, wire flag, permission truth, store or durable schema is introduced.

Actions oracles must drive real relay publication/resolution, the actual
guard, Responses loop and production checkpoint owner with a counted effect.
Cover timeout/close, Coordinator-style remote failure, both typed synthetic
error encodings and malformed results; require sticky unknown, no accepted
checkpoint, no new attempt and no second effect even after a late response.
Pair them with pre-publication refusal, list compatibility, ordinary success,
server `isError`, explicit rejection and nested payload controls. Exercise the
actual shipping bridge in an app test; existing assertions remain intact.
Also cover replacement/simultaneous relay isolation, sequence exhaustion and
Coordinator's exact opaque-ID success/error echo through its existing owner.
Independent full-scope review and exact-H Actions are required. These controls
do not prove physical termination, remote idempotency, reconciliation after
Host loss, original-turn disk recovery, or Automation settlement of unknown
effects; those obligations remain open.

Necessary target authority is the existing Host relay/bridge/guard plus
Coordinator `control_command`, `runner_tool_relay`, Electron's control server
and MessageChannel carrier, generated `McpResult` oneof and its official/
backend producers. This is contract review of generated representation and
bounded deep review of failure/call paths, not a new whole-file reading claim.
The fixed-U started-tool settlement ranges above remain scoped authority.
Rollback must revert the error variant, its producers and all consumers
together; retaining an error-bearing raw checkpoint never grants eligibility.

The internal entry is `CoordinatorToolRelay::request_tool_execution(args)`;
its `CoordinatorToolExecutionError` retains the original relay error and an
`outcome_unknown()` fact determined by the owner, not request JSON. Existing
Runner `coordinator_tool_execution_error` maps that fact to provider errors.
No other caller changes the original generic `request(method, args)` contract.

### Coordinator control waiter retirement — REC-10

Decision `REC-CONTROL-RETIRE-01 — FIX / IMPL_BUG`, specified before code on
`3761bde297377bca80ad7497d1664f6f3d32bc58`; fixed U is unchanged. The actual
`control_command` registers a request and waiter in `ControlPortClient`, writes
the frame, and waits 15 seconds. Its write-error and timeout exits currently
leave both entries behind until a late remote reply or whole-port shutdown.
Repeated lost replies retain unbounded local wait state. REC-07 correctly
preserves the remote outcome as unknown, but does not retire this Node state.

Keep the existing control client as sole correlation/waiter owner. Factor the
current synchronous call into a production-used helper in that module, accepting
the existing mutex, method/args, wait duration and frame-write callback. The
shipping wrapper supplies the existing 15-second duration and carrier writer.
Keep `ControlCallResult`, ordinary `call_waiting`, all legacy control consumers,
Runner relay payloads, and `Failure`/frame schemas unchanged. No new execution
disposition, permission owner, retry policy or second waiter map is needed.

Before returning after write failure, receive timeout/disconnection or an
unexpected post-registration action, retire exactly that request from both
existing maps. Retirement is idempotent against a reply or port settlement
that already won. A late reply for the retired ID produces no resolution; an
unrelated request retains its waiter and can still finish. Never hold the client
mutex across the writer callback or blocking receive. Registration refusal does
not invoke the writer. Preserve original error code/message projection, and
keep cleanup independent of diagnostics. Poisoned cleanup must not strand the
owned entries when the mutex can be recovered for that exact removal.

This is local wait-state retirement, never remote cancellation or termination.
A write can partially publish before failing; a timeout or port close can leave
Electron work running. Continue returning the existing failure so REC-07–09
keeps its unknown outcome; do not synthesize a success or permit replay.

Actions oracles use the real client and production helper, with a short injected
deadline and counted remote work held behind entry/release/completion channels.
Cover timeout while work still runs, invoked write failure, closure/protocol
settlement, exact two-map cleanup, a surviving unrelated waiter, ignored late
reply, synchronous reply/callback reentry, ordinary remote failure and refusal
before the callback. Preserve existing control/relay assertions. Exercise the
helper through `execute_runner_tool_request` to retain the real Host resolution
shape. The actual main wrapper must call that helper; compile the shipping
Coordinator and run its existing `coordinator_contract` target with the retained
relay targets. No local executable validation. Shared gates and independent
frozen-scope review precede verification claims. There is no persisted-format
change; rollback restores the prior local leak, and external reconciliation,
original-turn restart and full acceptance remain open.

## Actions oracles and limits

Use the existing scoped GitHub Actions workflow, on one frozen H. No local or
htch-runtime verification. Check shipping Host compilation and the existing
Settings/Inference/provider/Runner targets affected by these shared fields.
Positive and negative evidence must exercise the actual owners and serializer:

1. Persist a bound default through the existing writer, reopen the owners, and
   observe its exact model in a captured shipping Fabushi Responses request.
2. Verify legacy unbound Cursor behavior, a bound default's route isolation,
   session-over-explicit precedence, model case/path preservation, and invalid
   binding/source/pair/raw-field refusal. The refused turn releases its lease
   without advancing permission direction; a later valid turn can be admitted.
   No name-based fallback or route escalation.
3. Commit default B during turn A; observe A through its next tool step and
   accepted-checkpoint retry, B on a later turn. Observe actual checkpoint model
   and serde/persistence retention, without calling that restart execution.
4. Reject absent/mismatched checkpoint model before HTTP/tools, retain accepted
   tool result without duplicate execution, and retain cancellation precedence.
5. Preserve Cursor/Codex/Claude/OpenRouter contracts, internal summarization and
   Cursor web inputs. A fake transport/loopback capture proves protocol wiring,
   not real provider account or service compatibility.

This batch does not close catalog discovery, aliases/hooks, supported model
availability, `parameters`/thinking mapping, fallback ladders, trusted session
model mutation ingress, real service testing or the final G01–G12/AC01–AC24
matrix. OCRM inventory/ledger/RTM remain authoritative; no verified/accepted
state or global flag changes follow from authored code or a local green result.
