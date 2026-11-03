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

### Fabushi Responses credential context on refresh — OCRM-FABUSHI-CREDENTIAL-CONTEXT-v1

Decision `FABUSHI-CREDENTIAL-CONTEXT-01 — FIX / IMPL_BUG`, specified before
implementation on `ae8774e38fd04ba07d571dd5c5ea42ddb600f977`; fixed U is unchanged.
The existing Responses transport captures a token and endpoint, but its 401
path rereads the credential file without retaining account/session identity.
Replacing that file after login can send the same request with another user's
token. Authentication/configuration failures also lose their type in the
Responses conversion and can be retried, including after watchdog expiry.

The existing Electron Native account owner writes the version-1 credential;
the existing Host Auth owner validates it as `FabushiCoreCredential`.
Inference owns Responses transport and Runner owns attempt classification.
Reuse those owners without a new account store, credential cache, auth route,
permission authority, provider loop, dependency or persisted schema.

- FC-01: share the existing Host Auth typed document decoder instead of adding
  another account parser. Host Auth retains its current file/clock behavior.
  Responses retains its private-regular-file and issuer/endpoint checks, reads
  one document, and obtains token and context from that same typed image. Keep
  version, expiry, required identity and supported-origin validation. Invalid,
  expired or unavailable credentials cannot become an anonymous/token-only
  context. Preserve the existing producer's string/numeric user-ID handling.
- FC-02: within one `CodexHttpTransport`, capture canonical issuer, user ID,
  device ID and session ID plus the already validated complete Responses URL.
  On the existing single 401 reread, accept a replacement token/expiry only if
  all four identity fields match. Compare typed fields, not Debug output, token
  hashes, localized messages or an unverified JWT claim. Retain the captured
  endpoint; do not consult a changed environment to redirect the retry. A
  mismatch or invalid reread fails before the second HTTP request and leaves
  the original transport binding intact. A newly constructed transport may
  capture the newly signed-in context. Do not introduce active account refresh.
- FC-03: preserve explicit Authentication and Configuration variants through
  `CodexDirectError` in both conversion directions and through real Runner
  failure/retry reporting. A simultaneous watchdog flag cannot convert either
  permanent failure into a transient timeout or a new attempt. Retain explicit
  user cancellation precedence, sticky unknown-effect custody, ordinary
  effect-free transient retries and safe accepted-checkpoint continuation.
  Do not classify an identity mismatch by its message text.
- FC-04: errors and Debug output must not expose access/refresh tokens or raw
  credential documents. Do not serialize the credential into checkpoints or
  transfer Fabushi credentials to Cursor/Sand. Model, tool, Session admission,
  permission and existing provider selection behavior remain owned by their
  current paths; this contract grants no additional provider authorization.

Actions must exercise the actual HTTP transport with a local server which
remains observable after returning 401, and an actual credential-file swap:
same-context token rotation sends exactly one permitted retry and succeeds;
user/device/session/issuer changes, missing fields, malformed or expired images
produce the typed refusal and no second request. A later new transport accepts
the valid new context. Retain request/model bytes and demonstrate redaction.
Use the real attempt adapter for Authentication/Configuration plus watchdog
controls and existing transient/custody positives; a direct classifier-only
assertion is supplementary. Existing model and tool-custody HTTP fixtures must
use the complete valid Native producer image, retaining every behavioral
assertion; token-only/missing-identity input becomes an explicit negative.

Implementation stays in existing `auth_service`, `provider_session`,
`codex_direct_responses`, and `production_turn_run_shell_adapter`, plus their
existing tests/fixtures and scoped Actions selector. Freeze the complete diff,
independently review it and obtain final exact-H shipping/shared gates. No local
or htch executable validation. No data migration is introduced; rollback must
revert the typed transport and retry contract together.

This is a target-owner identity correction. Necessary source authority is the
existing Native account credential producer, Host Auth typed reader, Responses
401/serialization path and Runner retry consumers. Backend token renewal keeps
the same account-session row and its user/device/session IDs. This bounded
review does not add an upstream whole-file reading claim. It does not close
credential pinning across unrelated new Runner attempts, original-turn durable
claim/restore/settle, login revocation of already in-flight HTTP, real provider
compatibility or full OCRM acceptance; those responsibilities remain open.

### Native HTTP error projection — FC-04-HTTP-01

Decision **FC-04-HTTP-01 — FIX / IMPL_BUG**, before the focused correction.
Independent frozen source review identified that the existing non-401 Fabushi
HTTP error path copies arbitrary response text into Transport diagnostics and
copies policy-refusal strings into Debug-derived error diagnostics. A response
can therefore reflect credentials despite FC-04. This is an existing adjacent
defect exposed during this batch, not a newly introduced regression.

The existing Fabushi Responses boundary must project unsuccessful HTTP
responses into safe diagnostics. Generic errors retain the canonical HTTP
status and existing Transport classification, without copying the raw body.
A recognized policy refusal retains its typed ProviderRefusal category and
canonical HTTP status through a fixed diagnostic, without forwarding arbitrary
explanation, continuation or error-type strings from that body. A normal
credential-free Native refusal must remain a policy refusal; it must not become
an authentication error or a retryable generic failure. This deliberate
diagnostic restriction applies to the Native HTTP-error boundary. Preserve the
existing parser and ChatGPT route's structured refusal semantics, the successful
SSE protocol, model/request bytes, the sole same-context 401 retry, explicit
cancellation and all previous valid assertions. No token-pattern guesses,
second sanitizer framework, persisted credential state or dependency is needed.

Extend the existing real HTTP/owned-child receipt fixture with generic 403/500
bodies and recognized refusal bodies containing distinct access/refresh-token
sentinels in both generic and structured fields; cover an error after a valid
401 rotation as well as an initial error. Require the actual request count,
unchanged status-based retry disposition where applicable, retained refusal
category, zero output/tool/checkpoint/usage publication, and no body or secret
in either Display or Debug. A credential-free refusal is the positive
classification control; existing ordinary parser/ChatGPT tests retain their
structured values. This is not global arbitrary-response or all-provider
redaction acceptance. Fresh focused independent review and final exact-H
Actions are required before verification.

### Credential HTTP fixture I/O — FC-04-HTTP-02

Decision **FC-04-HTTP-02 — FIX / TEST_FLAW**, before implementation. At
`2c32127539bba5e797eaef89a02096bb2cb24f5d`, Runtime `41440794963`, Host job
`125263299411`, the real credential child returned Transport while its valid
oracle required Authentication. The fixture makes its listener nonblocking
but uses blocking-style request reads without resetting accepted sockets;
macOS/BSD can inherit that mode. The existing model/custody fixtures already
clear it. The parent also panics before joining the server, and Drop discards
its error, so the old log cannot establish the particular server error or case.
This is a source-proven fixture portability/capture defect, not proof that the
production identity check failed or that this specific failure was WouldBlock.

Set each accepted fixture connection to blocking I/O before the existing
bounded read/write timeouts. Preserve nonblocking listener polling, bounded
child lifetime, complete responses, every case, request count, model/body,
typed refusal, redaction, cancellation, custody and execution-receipt assertion.
Report the failing fixture case/phase and server I/O outcome even when the
child fails; diagnostics must not print authorization, request bodies or
credential documents. A socket/fixture error remains failure, never a typed
authentication result or a successful negative control. No production retry,
credential policy, dependency, source lock or acceptance threshold changes.
Fresh focused review and the whole existing credential target plus the affected
shared gates on the resulting exact H in GitHub Actions are required. Preserve
the original failed run; do not rerun the unchanged instrument to select green.


### Private transport fixture I/O — FC-04-HTTP-03

Decision **FC-04-HTTP-03 — FIX / TEST_FLAW**, before the source-review correction.
The same accepted-stream assumption remains in the existing `#[cfg(test)]`
`provider_session::fabushi_credential_context_tests` reader. Both of its real
HTTP listeners are nonblocking, so this shared reader must restore blocking
mode before its existing five-second read/write timeouts. Retain the actual
transport, all five request/endpoint/context/token/body assertions, typed
callback refusal, child nonce and bounded lifecycle. Its normal path already
joins the server before adjudicating child success; no production or retry
change is required. The existing selected private lib filter did not execute
at failed H `2c32127539bba5e797eaef89a02096bb2cb24f5d`; this is a source-proven
adjacent fixture defect, not another observed CI failure. Include this test-only
source delta in fresh focused review and final exact-H Actions with the whole
credential target and shared gates. Fixed U and full acceptance remain intact.


Integration on `800f166b8e30aa79e55435804dd514f915728332` preserves the incoming
FC-04-HTTP-02/03 repair and REC-11/12/13 producer contract. The following prior
fixture decisions add direct mode-transition and failure-observation controls;
they do not replace the incoming assertions or upgrade historical evidence.

### Credential HTTP fixture I/O — FC-HTTP-FIXTURE-01

Decision **FC-HTTP-FIXTURE-01 — FIX / TEST_FLAW**, before implementation on
`2c32127539bba5e797eaef89a02096bb2cb24f5d`. Runtime `41440794963`, attempt 1,
job `125263299411` compiled shipping Host and ran the Settings group. Its new
credential-context child returned Transport where Authentication was required;
the parent propagated the same failed case. The original log has no case label
or retained server-worker outcome, so its exact hidden server error is unknown.
Do not reinterpret the observed Transport as an Authentication result or claim
that this run proved the complete new credential contract.

The actual fixture accepts streams from a nonblocking listener, then uses
`BufRead`/`read_exact`/`write_all` without restoring blocking mode. Apple accept
inherits file flags; Rust 1.97.1's Apple accept path does not clear this flag.
An accepted socket can therefore return WouldBlock before a complete request,
terminate this fixture worker and close the listener. Its destructor discards
the worker error when a child assertion fails first. This is a source-proven
fixture defect; the missing original worker error is not retrospectively filled
in from a later green result.

- HTTP-FIXTURE-01: the common accepted-stream reader explicitly sets blocking
  mode before retaining its existing five-second read/write deadlines. Keep
  the listener nonblocking, its owned polling/stop path, the child deadline,
  real private-file replacement and actual production provider invocation.
  No production transport, credential validation or error classification changes.
- HTTP-FIXTURE-02: a deterministic actual TCP control starts with an explicitly
  nonblocking accepted stream, exercises this same configuration/reader and
  receives a request in separate header/body parts. Prove the mode transition
  directly where the platform exposes it and retain complete parsing/body
  assertions; do not rely on a larger sleep, timeout, transient retry or a
  platform-specific lucky arrival of all bytes before the first read.
- HTTP-FIXTURE-03: identify the credential case/control in parent and child
  evidence. Preserve accepted/parsed/responded counts and the server-worker
  outcome when a child fails, including worker errors otherwise hidden by
  unwinding. Do not print authorization, credential documents, request bodies
  or raw token-bearing server responses. An HTTP fixture failure remains a
  failure, not a substitute successful receipt.
- HTTP-FIXTURE-04: retain every current context mismatch, initial refusal,
  single-401 retry, later-new-transport, watchdog/cancellation, typed error,
  redaction, model/request-byte and zero-publication assertion. The entire
  existing credential target plus the direct stream control must pass on the
  next exact H in Actions. No unmodified failed-subject rerun or assertion
  relaxation is authorized.

Primary platform contract review is limited to Rust tag `1.97.1`,
`library/std/src/sys/net/connection/socket/unix.rs` blob
`936dd4c93233c4a11d65b32404a04f6a497d22f4` (accept, lines 241–270), and Apple
`bsd/kern/uipc_syscalls.c` blob `8e7e708bfd6d94cd366b4b46d62e94d853187256`
(accepted file flags/nonblocking state, lines 618–650). Sources are
[Rust](https://github.com/rust-lang/rust/blob/1.97.1/library/std/src/sys/net/connection/socket/unix.rs)
and [Apple XNU blob](https://api.github.com/repos/apple-oss-distributions/xnu/git/blobs/8e7e708bfd6d94cd366b4b46d62e94d853187256).
The XNU source establishes platform behavior, not the exact runner kernel build
or a syscall trace of this failure. This is a dependency/API contract review,
not a new fixed-U full-file reading claim. Scope is this spec and the existing
credential-context test file; retain the existing workflow selection. Freeze
and independently review the repair before delivery. Per-transport identity
and broader original-turn/model/permission acceptance limits remain unchanged.

Decision **FC-HTTP-FIXTURE-02 — FIX / TEST_FLAW**, before the additional edit:
the final selector review identified the same accepted-stream assumption in
the already-selected `provider_session::fabushi_credential_context_tests` reader
at this same base. Its two nonblocking listeners feed that separate `cfg(test)`
reader. Extend the scope only to normalizing that reader before its existing
timeouts; production transport code and every existing endpoint/context/body
assertion remain unchanged. The existing real HTTP lib case stays selected.
This is an adjacent source-proven fixture defect, not a second observed failure
at H2c, whose Recovery/lib step did not execute. Re-freeze the complete six-file
scope and independently review the fix delta plus its callers before delivery;
the earlier five-file snapshot cannot approve the added path.

### Credential fixture causal controls and capture — FC-HTTP-ORACLE-02

Continuation of FC-04-HTTP-02/03, retaining the published blocking-I/O repair.
Decision `FC-HTTP-ORACLE-02 — FIX / TEST_FLAW` preceded this implementation;
the original failed run's exact transport cause remains unproven. Exercise
both actual request readers with explicitly nonblocking accepted sockets and
inspect the OS flag afterward, alongside request/body projection and bounded
I/O settings. Removing normalization must fail independently of packet timing.

Collect both owned terminal outcomes before a child failure assertion. A
timeout or wait error must remain failure: terminate/reap only that owned child,
retain its cleanup result, stop/join the server, and collect a credential-free
case/phase/count, redirected output and available receipt before root cleanup.
Do not retry a child or reinterpret Transport as Authentication. Preserve every
typed refusal, request count/body/model, endpoint, nonce, cancellation, retry and
redaction assertion. This changes test instruments only. Use fresh independent
review and the existing exact-H Actions selectors; source review is not a pass.
The separate 007c Automation directory error remains unresolved and is not
explained by this socket repair.

`F5-PRIVATE-CREDENTIAL-DIAGNOSTIC — FIX / TEST_FLAW`: the private transport
fixture must also project redirected output to byte counts, parsed panic
coordinates or I/O error classes, never raw child output. Preserve every token, body and nonce equality predicate
without value-printing failure messages; a sensitive-output negative control
must exercise the same diagnostic projection. Production behavior is unchanged.


### Credential fixture observation union — FC-FIXTURE-OBSERVATION-01

Decision **FC-FIXTURE-OBSERVATION-01 — FIX / TEST_FLAW**, before integration
on `a514b70ab7901e9c8b4d87a8d29bf8e7d52be507`. Preserve FC-04-HTTP-02/03,
FC-HTTP-FIXTURE-01/02 and every incoming assertion. The independently reviewed
parallel fixture candidate adds a second, distinct empty-socket control and
pre-assertion evidence; merge those through the same reader and observation
owners rather than replacing the existing split-header/body control.

- Keep both actual TCP controls: the split request directly checks socket mode,
  five-second deadlines, header/body phase ordering and complete 204 response;
  the delayed empty connection inspects the shared descriptor before sending
  bytes and uses the actual fixture's 200/SSE response path. Use one reader.
- Publish nonce/case/control-bound running and observed child receipts atomically
  before semantic assertions, and completion only after all checks. Sanitizing
  a private sentinel is sticky failure, never permission to accept the receipt.
- Collect the exact child's bounded exit and join its owned server before parent
  adjudication. Retain safe phase, numeric panic location, accepted/parsed/
  responded/captured counts and server outcome even on failure. Planned status
  and completed response write remain different evidence. Do not print private
  request, credential or response bodies. Keep all existing timeout and retry
  bounds and every typed/error/model/body/redaction/custody predicate.
- Apply the same pre-assertion phase and explicit finalization discipline to
  the already selected private transport fixture. Preserve its five actual
  requests, captured endpoint/context, callback refusal and new-transport proof.

Reconciliation on `b60a2e406071827031ee18e81b35d8e71d4f1aee` retains the
FC-HTTP-ORACLE-02 and F5-PRIVATE-CREDENTIAL-DIAGNOSTIC additions above. Keep
the private forced-nonblocking reader control and its exact request, OS-flag
and read/write-timeout predicates, plus the sensitive-output negative control
using the actual diagnostic projection. Both parents require successful reads
of redirected stdout/stderr and expose only byte counts, numeric panic
coordinates or safe I/O error classes. Receipt diagnostics retain read/parse
outcomes and binding booleans without printing raw output or nonce values.
Reader/replacement/write phases and owned child cleanup observations join the
same existing report, not another phase store, receipt file or provider loop.

`FC-CHILD-CUSTODY-01 — FIX / TEST_FLAW`: a kill request alone is not proof
that the owned exact-test child exited. Replace the fixtures' unbounded
post-kill wait with bounded status observation; keep timeout, wait and cleanup
errors nonpassing. Only an actual exit/reap observation releases that child's
fixture-root deletion permission. An unresolved child retains its exclusive
root and failed cleanup evidence. A real nonce/PID-bound child control must
prove live-child root retention and observed retirement before cleanup. This
repair adds no product retry, process-name cleanup or production authority.
The incoming FTB private transport control uses this same existing fixture
finalizer; its additional initializer and wait path cannot bypass the bounded
retirement and root-retention contract. Preserve all FTB request predicates.

This scope is the two existing credential test surfaces and this contract;
production behavior, dependencies, workflow selection and full acceptance stay
unchanged. Historical failures and the unavailable original errno remain as
recorded. Fresh frozen-scope review and exact-H Actions are required.


### Frozen terminal tool-custody evidence — REC-11/12/13

Contract: `OCRM-RUNNER-TERMINAL-CUSTODY-v1`, specified before code on
`3dea753726c6e2751252ce4ed2a68b17cfdc97cf`; fixed U is unchanged. The existing
generated stream creates the per-run `StreamAttemptRuntime`, but returns only
text/error. Its real owner/facade and the Host finalizer therefore lose the
typed pending/unknown facts which REC-01–09 already preserve. Cancellation or
an ordinary serialized provider error cannot recover those facts.

Keep the existing generated stream -> `ProductionTurnAgentOwner` ->
`SandAgentRunner` -> shipping `RoutedProviderTerminal` -> inference event chain.
The companion is a frozen value, not a new execution owner, registry, store or
terminal event. This is a necessary producer contract for later Automation
retention; it does not yet alter Automation history, completion POST/ACK,
background receipt settlement, upgrade replay or original-run durability.

- REC-11: preserve the public generated-stream Result entry and factor its
  existing implementation into an internal structured outcome carrying that
  result plus a snapshot from the exact runtime supplied to composition. Read
  custody after the real generated lifecycle/persistence returns on both
  success and error. No copied provider loop or test-only custody setter.
  The turn owner exposes its last invocation as `Known` with actual shell run
  generation and snapshot, `NotStarted` only when the generated executor never
  entered, or explicit `Unavailable`. Reset this observation before validation
  and admission can return; a failed new invocation cannot inherit an old one.
  A legacy `run_with` callback is uninstrumented once entered and must report
  Unavailable rather than a clear snapshot. The existing facade forwards it.
- REC-12: a fresh stream-bound report accumulates one observation after every
  actual primary, reply-nudge-loop and closing-nudge invocation. Retain each
  invocation's own revisions; do not sum them into an invented common clock,
  replace earlier uncertainty with a later clear result, or retain live Arc
  handles whose late workers can rewrite the frozen report. Missing/empty
  evidence or a mismatched/blank stream binding is Unavailable. Known pending
  work or sticky unknown requires retention; a started/checkpoint revision
  difference alone is only the existing retry restriction, not unknown outcome.
- REC-13: consume the report beside the local terminal after the final
  result-changing Human handoff write. Add the typed `toolEffectCustody`
  companion to the existing event payload. Keep waiting/cancellation/success/
  failure tags, provider-review precedence, raw result content, registry/lease
  retirement, telemetry and once-only publication unchanged. Legacy/missing
  observations cannot be silently projected as clear. The report is evidence
  of the instrumented tool boundary, not physical termination of remote work.

Scope is the existing Runner stream/owner/facade and exports, the Host finalizer
and its three invocation sites, existing owner/effect-custody/composition contract targets,
and the existing scoped workflow. Reuse its locked commands and retain all
channel/Session/Automation/terminal assertions. Correct the prior non-blocking
channel lib-filter receipt-index omission while this workflow is already in
scope; the actual channel test selection remains unchanged.

Actions-only oracles must exercise a real bound production Runner/generated
stream with child-scoped loopback Responses transport, actual guarded tool and
relay, counted non-idempotent effect, strict checkpoint owner, and late external
completion. Require the owner/facade snapshot, one effect, no replay/checkpoint,
and immutable unknown after a late return. Preserve the lower-level genuine
pending-worker controls; add actual finalizer/report tests for pending and
downstream-unknown facts, late completion, multiple invocation observations,
stream mismatch, missing evidence, clear success/failure, pre-admission refusal,
and uncheckpointed-but-known distinction. Do not infer a specific provider
error variant from cancellation; the existing token controls the Host terminal.
No global environment mutation, live account or service acceptance is claimed.

Necessary authority is the actual target call/state chain and the fixed-U
started-tool settlement ranges already recorded above; this adds no whole-file
reading or source-closure claim. Freeze and independently review the complete
scope before exact-H Actions. Rollback reverts producer, companion and consumer
together without deleting persisted checkpoints. Durable original-run claims,
Automation retention and backend dedupe/ACK, original-turn restart, unknown
effect reconciliation, real ecosystems and all full-acceptance gates stay open.

### Native active-turn request identity — OCRM-FABUSHI-TURN-BINDING-v1

Decision **FTB-01 — FIX / IMPL_BUG**, specified before implementation on
`a514b70ab7901e9c8b4d87a8d29bf8e7d52be507`; fixed U is unchanged. FC-02
correctly fences one transport's 401 retry. A later Runner attempt currently
constructs a fresh transport from current environment and credentials, so a
transient retry can silently continue an admitted turn as a different account.
The same gap affects separately invoked reply/closing nudges and turn-memory.
Keep the existing Native account producer, Auth decoder, Inference transport,
Host admission and Runner composition as their sole owners.

- FTB-01: add cloneable `FabushiResponsesRequestBinding` in `provider_session`,
  holding only an `Arc<OnceLock<...>>` of the validated Auth context (canonical
  issuer/user/device/session), complete endpoint and credential path. It has
  no Debug/serialization, credential/token/expiry cache, public reset/default,
  or public unbound constructor. Public
  `capture_fabushi_responses_request_binding()` eagerly validates one actual
  credential image, captures its nonsecret identity and discards that image.
  Reuse the existing guarded file reader and typed decoder without another
  auth authority or dependency.
- FTB-02: before each new bound transport, reread the captured path with the
  captured endpoint and require the same context. Same-context token/expiry
  rotation is allowed; unavailable, malformed, expired or changed context
  returns the existing typed permanent error before HTTP. A mismatch never
  mutates the binding. Do not consult changed endpoint/path environment after
  capture. Preserve FC-02's existing single 401 refresh and all current file,
  issuer, endpoint, redaction and cancellation constraints. This does not
  revoke an already in-flight request or poll credentials after each tool.
- FTB-03: shipping admission captures for Fabushi only, after accepted-lease
  guard, checked Session state and model resolution, before permission
  `begin_turn` and subsequent effects. Map Authentication to Unauthorized,
  Configuration to BadRequest and other errors to Internal. A refusal drops
  the existing lease without advancing direction. Non-Native routes perform
  no Native credential read. Carry the owned optional binding alongside the
  frozen model through `ProductionRunnerCompositionInput`,
  `TurnAgentComposition`, `RoutedProviderRun` and its actual attempt executor;
  borrowed `RoutedProviderOptions` carries `Option<&...>`. Composition defaults
  to None and exposes `with_fabushi_request_binding(binding)`. Host supplies
  Some, shared by main execution and every same-worker reply/closing nudge.
- FTB-04: a direct Fabushi Runner invocation with None allocates one empty
  binding at executor construction, without I/O; all attempts reuse it. The
  first valid Native transport binds once. Race losers compare the complete
  identity and refuse disagreement. Use stable OnceLock get/set with all I/O
  outside synchronization. Preserve cancellation, checkpoint-provider and
  model/checkpoint checks before credential acquisition. Another independent
  invocation with None gets a fresh binding. Direct provider calls with None
  retain fresh-operation behavior, including the FC-02 new-account control.
- FTB-05: move Host's existing no-tools turn-memory prompt body into
  `ProductionInferenceExtension::run_turn_memory_prompt(data_dir, provider,
  fabushi_model, fabushi_request_binding, system_prompt, user_prompt,
  should_cancel)`. The real Host memory closure passes its admitted values;
  extraction and episode synthesis use that same binding. Preserve two-message
  construction, existing Cursor auth, cancellation, no-tools and RecordEvidence
  behavior; do not reselect current route/model or capture a new identity.
  Native turn-memory without its admitted binding refuses as Configuration
  before transport; explicit cancellation retains precedence.
  Standalone and post-turn compaction summarization keep current route/default
  and None. A separately admitted later turn may capture the new account.

Scope is the six existing provider/runtime/composition/bridge/Host/Inference
production paths and their current constructor callers and selected contract
tests. Session/checkpoint schemas, terminal custody, channel and Automation
owners, Electron account producer and source locks are unchanged. Required
review is the actual producer/decoder, request and retry paths, Host admission,
generated composition cloning, memory consumer and failure/cancellation order;
this target-owner correction adds no upstream whole-file reading claim.

Actions-only oracles use actual private credential files, loopback HTTP,
generated production Runner and real accepted checkpoint/effect owners. Cover
pre-effect 503 retry and post-tool accepted-checkpoint retry with account swap
(one dispatch/no replay), same-context rotation (successful continuation),
main-to-nudge and actual turn-memory identity reuse, and a separately admitted
new-account positive. Preserve durable checkpoint bytes, model/history, effect
counts, no output on refusal and all existing unknown/cancellation controls.
Also cover captured endpoint/path despite changed configuration, checkpoint
error precedence, initial malformed/unavailable credentials, and non-Native
isolation. Extend the existing shipping admission test with a nonce-bound owned
child, real private credentials and an exact completion receipt, retaining the
valid-after-refusal case and lease/direction assertions. No process-global
environment mutation or mock substitute for the real consumer is accepted.

Reuse existing scoped Settings/Inference, shipping Host, model/credential,
Runner/composition/custody and memory selections. Freeze the complete scope,
independently review it, then obtain resulting exact-H Actions evidence before
verification. Rollback reverts this ephemeral binding and all callers together;
no data migration or credential persistence is introduced. Original-turn disk
load/claim/settle, identity after process loss, summary identity policy, remote
revocation, real service compatibility and all full OCRM acceptance stay open.

## Existing desktop default-model bridge — OCRM-MODEL-BRIDGE-v1

Integration base: `0533ca72870a2794c2db32959b9d01427abf325a`, fixed U unchanged.
The Rust identity chain above is already implemented at this base. The existing
`agents.setDefaultModel` preload/appWindow RPC still loses `provider` while
`main-edge.ts::parseAgentModel` reconstructs the selection. The existing local
settings mirror setter and fast-parameter migration also drop the field. That
turns a bound default into a legacy unbound selection before Rust can decide it.

- MB-01: Preserve the presence and exact value of an explicitly supplied
  `provider` through the existing default-model RPC into Host Settings. Missing
  remains missing. Null, non-string, blank and unknown bindings must reach the
  existing Rust validator unchanged and be refused there; the bridge cannot
  delete them or silently turn them into legacy defaults. Retain the existing
  base-shape parsing and parameter copying. An own `provider: undefined`
  cannot be represented in JSON and must fail explicitly at the bridge before
  Host dispatch, rather than being omitted during serialization. Model authorization, canonical
  provider validation, normalization and inference remain Rust responsibilities.
- MB-02: Update the local display mirror only from an acknowledged Host result,
  preserving its provider and model metadata through setter/getter, reopening
  and the existing fast-parameter migration. Host refusal or unavailability
  leaves the previous mirror intact. An acknowledged absent/null whole model
  clears the mirror; this does not erase a null provider inside a model object.
  Retain account-clear and legacy unbound
  behavior. The mirror never becomes inference or permission authority.
- MB-03: The shared and renderer DTOs may carry an optional opaque provider
  value so raw IPC presence is not lost; they do not add a provider allowlist,
  route selection, catalog or policy evaluator in TypeScript. Preserve the
  existing appWindow top-frame trust boundary and Computer Use behavior.
- MB-04: Actions exercise the actual served handler and real file-backed mirror,
  with a controlled Host acknowledgment/refusal port, malformed-binding forwarding,
  legacy/default migration and account-clearing controls. Reuse the shipping
  Electron bundler and the existing Rust Settings/Inference/model contracts.
  These are bridge and owner-contract proofs, not real provider or UI acceptance.

No new session-model mutation API is admitted. The trusted catalog/operator/
visibility/placement inputs required by the existing session mutation ADR remain
unresolved; the full picker responsibility remains not-wired. Completed-turn
upgrade-marker acknowledgment now exists, but does not prove interrupted
provider continuation after restart. Final service, UI and platform gates remain.

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

### Automatic-attempt verification — OCRM-FABUSHI-AUTOMATIC-ATTEMPT-BINDING-v1

Decision **FABUSHI-ATTEMPT-BINDING-01 — FIX / IMPL_BUG** was specified and
implemented against `73cf0269c5ccbbd1dff946e0949824a1bbbeffd4`. Before
publication, `1501be999f195e7204da8b095a0ab64d5bc3a5aa` supplied the broader
FTB-01–05 implementation above. Integration adopts that existing request
binding and all its callers; the unpublished separate `FabushiAttemptBinding`
implementation is superseded, with its immutable source/review history kept.
There is one production binding. This increment adds the distinct actual
Runner oracles and the fixture repairs above; it does not replace FTB controls.

- AB-01/02: exercise FTB-04 through one actual public
  `run_routed_provider_in_runner` invocation with no pre-supplied binding. Its
  real executor captures the first validated typed context, complete URL and
  source once, after existing cancellation and checkpoint/model checks. Each
  later transport rereads that source; same-context token/expiry rotation may
  proceed, while a changed or invalid identity refuses before HTTP. A separate
  invocation may capture the later account. The shipping admission, nudge and
  memory lifetime remains FTB's responsibility and retains its own oracles.
- AB-03: retain existing typed permanent failures, retry budget, model/provider
  fences, output and checkpoint eligibility, cancellation and sticky unknown
  custody. Binding cannot authorize replay or clear custody. The integration
  uses the incoming public options without adding another forwarding argument,
  parser, credential cache, persisted binding, permission truth or runtime.
- AB-04: the already selected credential target includes private module
  `tests/fabushi_responses_credential_context_contract/automatic_attempt_binding.rs`.
  Its parent owns observable HTTP, fixed child environment and real private
  credential replacement before the 503. Cover changed user/device/session/
  issuer, invalid/expired/missing identity and file-shape/private-file failures;
  require initial Retried, attempt-2 GaveUpIneligible/AuthenticationError, exact
  request/report counts, no extra attempt or output/effect/checkpoint/usage,
  and a separate new-account positive. Same-context rotation must complete
  with the original request bytes and model.
- AB-05: use the actual guarded tool, synced effect receipt and durable
  checkpoint store. After a continuation's 503, changed identity refuses the
  third request and retains exact checkpoint bytes and one effect. Rotation
  resumes exact history/model without repeating the effect and preserves
  cumulative usage. Cancellation at the real retry callback retains the
  current outer Transport reason and existing unknown-effect controls.

The exact-child receipt binds module-qualified test, nonce, PID, scenario,
observed phase and completion. Keep the server observable through completion,
reuse the existing parser/writer and bounded cleanup, and do not count child
summaries as extra parent tests. Source review and exact-H Actions cover this
integrated scope; authored tests do not establish runtime success. Fixed U,
full product scope and all global flags remain unchanged. Original-turn disk
claim/restore/settle, unknown-result reconciliation, real services, platforms
and final G01–G12/AC01–AC24 acceptance remain open. No local/htch validation.

## Accepted-turn cancellation before worker registration — PRC-v1

Implementation base `7eb81e20b7dc2eac5a71516a9a0c78795d252e95`, fixed U
`1440bb1771372334bea23d7642bb52deb7db1845`. Continue the existing Transcript
lease, Runner cancellation token, App admission/finalizer and Coordinator
consumer. No new runtime, scheduler, permission owner or durable store.

Decision **PRC-01 — FIX / IMPL_BUG**: an accepted routed input already holds
its Host lease and emits lifecycle started, but `runner.cancelRoutedProvider`,
`interruptAgent` and `chat.abort` consult only the later worker registry.
Before registration they return false and leave the input executable. The
later registration creates an unrelated clean token. Decision **PRC-02 —
FIX / IMPL_BUG**: the App guard settles by agent/stream without its captured
ticket/generation; a stale guard must not retire a replacement lease.
Decision **PRC-03 — FIX / IMPL_BUG**: a pre-worker cancellation must reach
the existing cancelled consumer outcome. An ordinary start refusal would be
recorded as inference failure; an accepted-false reply ignored by an internal
waiter would wait for a worker that never started.

Necessary source understanding reuses the exact-U full reads of
`attempt-session-prepare.ts` (blob `b97bcf04649ea5787a341182123c62b20450bd6e`),
`attempt-session-runtime-prepare.ts` (`da120eecf356c6b0e27d0add0b1b7e843bf747ec`),
`attempt-session-settle.ts` (`e5735df007a1a469257c8797e6b2a2ce12e3ff0f`),
`attempt-session-recovery.test.ts` (`de442b4724c1fdeebca5e820b71a8ff5842953f7`),
and their checked original-input, permission, transcript-lifecycle and cleanup
dependencies. These preserve cancellation/current-owner checks across
preparation and real cleanup ownership; cleanup return is not proof of a
stopped external effect. This is not a claim of complete upstream coverage.

Literal source reads are appended to the existing `inventory/read-observations.json`
with their exact U blobs and full line ranges; existing observations remain.
Reader `/root/runner_restart_next_batch` fully read the four files above plus
the following direct constraints. Root inspected the reported call-chain and
matched the target consumers. Known observed edges do not assert recursive
dependency closure or semantic coverage of all responsibilities in these files.

| Exact-U path (under `src/agents/embedded-agent-runner/` unless stated) | Blob | Fully read lines |
| --- | --- | --- |
| `run/pre-persisted-user-turn.ts` | `346f5d38c39b9805a9f0f9c8768f0cb3c632cb82` | 1–623 |
| `run/attempt-permission-prompt.ts` | `4d02a1608048625e944a6aa202096d727725c1ea` | 1–218 |
| `run/attempt-transcript-lifecycle.ts` | `99daab9dec252a1826c09813e357704c66b3cea1` | 1–199 |
| `run/attempt-subscription-cleanup.ts` | `c90c99156ecf10812708c221b014df9900a8ffe0` | 1–89 |
| `wait-for-idle-before-flush.ts` | `9fa2d2954c1d3c9421a94ef5cdc22ad75b3cc3cb` | 1–40 |
| `src/gateway/server-methods/chat-abort-handler.ts` | `5c24cf496ed11cea105887d1176f23468e461ec8` | 1–721 |
| `src/gateway/server-methods/chat.ts` (navigation only) | `990a1bcba90efe1a66eed638dc3fac16fff4df83` | 1–80 |

The four preparation/runtime/settlement/recovery sources above were read
through lines 759/530/224/280 respectively. The abort handler captures the
reserved preparation before awaiting, never reselects a successor by run ID,
and distinguishes cancellation from physical cleanup and uncertain RPC return.
Unexamined abort helper dependencies remain outside this bounded existing-owner
repair and are not declared reviewed. The source materializer, not hand-edited
shard counts, incorporates these literal observations in Actions.

### Owner and entry contract

- PRC-01: each admitted `RoutedTurnLease` creates one existing
  `RoutedProviderCancellation`. A private-field start control captures its
  agent, stream, ticket and generation, and the same token. Only one start
  claimant may take its retirement responsibility. Capture rejects missing,
  foreign or already-claimed leases; cancelled or scheduler-stale matching
  leases still reach their guard so rejection cannot strand them.
- PRC-02: capture the guard after parsing agent/stream and before fallible
  Session/provider preparation. Recheck current identity and cancellation
  before model/account reads and again at the owner admission boundary before
  `permission.begin_turn`. Cancellation and preparation admission are ordered
  under the existing Transcript mutex. Cancellation winning that boundary
  causes no permission direction or subsequent preparation. Admission winning
  first permits already-entered preparation; later cancellation is cooperative
  and does not promise rollback of work that already happened.
- PRC-03: model binding cannot swallow start-control rejection as optional
  original-turn proof. Before provider-run/ACK/auto-review resources begin,
  check the same control again. Later registration accepts its existing token
  through `register_for_agent_with_cancellation`,
  `register_routed_provider_with_cancellation` (including recovery shape), and
  `register_group_member_with_cancellation`. Existing default-token wrappers
  remain for direct users without a Transcript lease. A late attach must not
  reset a prior cancellation, its first reason or first-change semantics.
- PRC-04: the three actual cancellation commands include matching admitted
  leases and registered workers, deduplicate stream identities, and preserve
  agent/session route restrictions. A first cancellation returns true even
  before registration; repeat/unknown/foreign requests return false. Unknown
  streams create no future cancellation record. Cancellation alone never
  removes the lease/registry entry, ends a run, opens the serial lane or emits
  a provider completion. An owner admission/control, not an arbitrary reason
  string or public request flag, governs execution and recovery rights.
  A present live lease's captured token is authoritative even on a repeated
  cancellation; do not reselect a later registry incarnation after cancelling
  it. Only direct registry users with no live lease use the registry fallback.
- PRC-05: guard retirement checks agent/stream/ticket/generation under the
  same mutex before removal. The actual guard or worker finalizer retires
  exactly its own lease; stale/foreign controls cannot release a successor.
  The existing lifecycle ended and queue notifications follow actual retirement.
- PRC-06: an internal typed pre-worker-cancelled start outcome is consumed by
  every App caller. Gateway projection is `accepted:false,cancelled:true` for
  the exact stream after guard retirement. The Coordinator maps only that
  well-formed response to its existing `INFERENCE_PROVIDER_CANCELLED` outcome,
  without writing an inference error or waiting for a nonexistent worker.
  Group, Automation and revival callers use their existing aborted/no-execution
  result contracts. Ordinary start errors and accepted workers keep existing
  paths. No successful provider result or external effect receipt is invented.

### Required Actions evidence and limits

Decision **PRC-ORACLE-01 — FIX / TEST_FLAW** (direct source review, before
repair): the existing chat-abort, turn-telemetry and App terminal-wiring checks
assume inline cancellation or a direct worker-only registry call. Follow the
actual Gateway delegation and lease-aware helper instead. Preserve the same
session/run filtering, telemetry ownership, cancellation-only semantics and
every finalizer-order assertion; execute both affected existing host targets.
These source checks remain wiring evidence, separate from the actual-owner
App cases below, and do not establish runtime success by themselves.

Decision **PRC-ORACLE-02 — FIX / TEST_FLAW** (independent source integration
review and root confirmation, before repair): the adjacent group-member
deletion wiring oracle still requires the old default-token registration call.
Require the actual group branch to pass the accepted lease's cancellation token
to `register_group_member_with_cancellation`. Preserve every existing group
isolation, preemption, deletion and serial-drain assertion. Select the complete
three-test existing target in both scoped Host commands, require all three
named positives, and bind its source hash in the existing receipts. This does
not change the group owner or substitute source wiring for runtime evidence.

Use the actual accepted lease, model-admission/late-attach helpers, real
file-backed Session/Settings and current permission owner. Controlled barriers
may pause production boundaries but may not supply their decisions. Require:
all three public cancellation commands before registration; cancellation before
model reading and after account/model capture but before permission; unchanged
permission, retained original input, no downstream preparation/provider/tool
effect on that refusal; late registration retaining the cancelled token;
duplicate, foreign and stale-control negatives; no lane/lifecycle release on a
cancel request; exact finalizer retirement and a later successful turn. Retain
the existing registered-worker finalizer and model-refusal tests. Check the
Coordinator's exact cancellation response, malformed/foreign response refusal,
normal acceptance and ordinary rejection. Internal waiters must not wait after
typed pre-worker cancellation. Run affected shipping compilation and contracts
on one frozen H in the existing Actions workflow, preserving incoming gates.

This closes a live accepted-before-registration boundary only. Durable
original-turn claim/revoke/seal, controlled process restart, unknown external
effects, all provider/channel compatibility and final OCRM gates remain open.
The protected existing `routed_send_acceptance_contract.rs` draft is not part of
this implementation. No global completion or scoped verified flag is promoted
by authored code, source review or an unrelated passing H.

Decision **CMI-PRC-CHILD-01 — FIX / TEST_FLAW** (historical CMI draft
identifier `PRC-ORACLE-03`; root integration source review
at `6273f9252f308355526c68e833da8dea194bb6cf`, before repair): the App fixture's
45-second parent deadline can unwind into `OwnedChild::drop`, which calls an
unbounded `wait()` after kill. Bound that single Drop retirement observation to
three seconds using `try_wait`. A kill request, observation error or exhausted
deadline is not terminal completion; retain the root unless real terminal exit
and the existing successful exact nonce/PID/completion receipt were observed.
Preserve all four selected tests and every behavioral assertion. This repairs
the test lifecycle only; it does not establish provider or restart acceptance.

### Provider checkpoint publication — OCRM-PROVIDER-CHECKPOINT-WRITE-v1

Decision **CPW-DEFECT-01 — FIX / IMPL_BUG**. At source base
`60631925e24b6d9ee7114d97b9974e51a913684e`, the existing
`ProductionRoutedProviderCheckpointStore::persist` creates its payload with
ambient file permissions, deletes the temporary path even if `create_new`
failed before ownership, replaces a colliding final path through `rename`,
and returns before synchronizing the directory entry. Provider input and tool
results can be private; a successful file write alone cannot establish the
publication boundary on which REC-04 relies. This is a target owner defect,
not a new upstream runtime or an extension of restart eligibility.

The producer remains the existing provider checkpoint callback; the sole
writer remains `runner/routed_provider_runtime.rs`; the consumer remains
`ProductionTurnRunShellAdapter` and its existing effect-revision acceptance.
The JSON provider/model/input payload, hashed agent/stream directory, unique
UUID filename and public `RoutedProviderCheckpointStore` contract stay intact.
The private installation-key publisher in `host_secret_store.rs` is the local
precedent for exclusive private staging and no-clobber hard-link publication;
its key-specific API is not a reusable checkpoint owner.

- **CPW-01 — private, owned staging.** Create each new temporary file with
  `create_new`; on Unix request mode `0600` at creation, never after exposing a
  public file. Acquire cleanup ownership only after successful creation, close
  its file before publication/cleanup, and clean only that owned temporary
  path. Existing temporary files, final files and unrelated entries must keep
  their exact bytes on collision or refusal. Do not retry a collision by
  deleting or taking over its entry.
- **CPW-02 — complete no-clobber publication.** Serialize the same typed
  checkpoint, write all bytes and synchronize the file before making its final
  name visible. Use the existing same-directory hard-link publication
  precedent, which fails if the final entry exists; never replace an existing
  final entry. The returned identifier names that complete JSON payload.
- **CPW-03 — directory barriers.** Newly created checkpoint directory segments
  are private on Unix (`0700`) and their parent entries are synchronized in
  creation order. After final publication, synchronize the containing
  directory before returning success. Preserve existing directory modes.
  If a creation succeeds before its parent barrier fails, a later attempt
  must reestablish that barrier; the leftover directory's existence is not
  evidence that the failed publication succeeded.
  Non-Unix keeps the established file-sync/no-clobber behavior without claiming
  a portable directory-sync guarantee; filesystem rejection is an error, not
  a fallback to replacement. This contract is not hardware power-loss proof.
- **CPW-04 — uncertain publication.** A post-publication synchronization error
  returns failure and preserves the visible final payload. Best-effort cleanup
  of this operation's owned temporary entry cannot remove that final name or
  alter another operation. A raw or leftover file never proves REC-04
  acceptance, an original-turn claim, or permission to replay an effect.
- **CPW-05 — unchanged custody consumer.** The actual shell adapter must not
  accept or resume from a failed production-store write. Preserve its existing
  effect revision, cancellation, unknown-result and later-output fences, all
  valid assertions, typed provider/model checks and same-context retry rules.
  No credentials, durable execution grants, new Session state or second
  scheduler are introduced by this publication repair.
- **CPW-06 — decisive oracles and execution.** On the frozen published H,
  GitHub Actions must exercise the actual writer's JSON round trip and Unix
  privacy, occupied temporary and final entries with exact byte preservation,
  new-directory creation, controlled directory-barrier failure before and
  after publication, and ordinary successful writes after an unrelated
  refusal. Use deterministic private helper inputs only to select a collision
  or a barrier failure; do not replace the public store with a fake for its
  file-publication claim. Exercise a real production-store error through the
  existing adapter, asserting no accepted checkpoint, retry or resume and
  preserved original filesystem bytes. Retain the existing real Fabushi
  checkpoint-before-follow-up and model/account retry controls. Named positives
  and command/capture success are required; no local/htch validation.

The implementation batch is limited to this specification, the existing
writer and its private tests, the existing adapter contract target, and the
shared workflow. Existing payloads need no migration and are neither rewritten
nor reclassified. Rollback preserves the on-disk format; it does not erase
failed or unaccepted files. Exact-H compilation and the scoped publication,
adapter and existing model/custody checks decide this increment. Full durable
original-turn eligibility, nonsecret Native identity revalidation, generated
tool-history retention, atomic recovery claims and actual startup continuation
remain separate unfinished production work under the same owners.


### PRC source-observation bounds recovery — PRC-SOURCE-01

Decision **PRC-SOURCE-01 — FIX / IMPL_BUG (source metadata)**, recorded before
repair: source run `41550052051`, attempt 1, job `125635116113`, for H
`6273f9252f308355526c68e833da8dea194bb6cf` failed during inventory expansion
because the new `chat-abort-handler.ts` observation ended at line 722 while
its exact-U blob `5c24cf496ed11cea105887d1176f23468e461ec8` has 721 lines.
The adjacent `chat.ts` observation ended at 81 while exact-U blob
`990a1bcba90efe1a66eed638dc3fac16fff4df83` has 80 lines. Both files have a
terminal LF; its empty split element is not another source line.

Repair only those two literal range endpoints and their table above. Preserve
the existing reader's literal history, exact blobs, read status, dependencies,
responsibility mappings and all completion flags. This correction does not
add a new reading claim or declare either file's semantic responsibilities
verified. The original invalid values and failed subject remain recorded here
and in their original commit. Keep `source_review.py`'s acquired-content bound
and complete-range checks, including the existing out-of-content negative;
no validator, generator or gate is relaxed.

The failed job had already passed the 33 source-review tests, 21 inventory
materialization tests and pre-expansion integrity check. They remain evidence
for that historical H only. The complete decoded job log SHA-256 is
`9e46c3986db11a93673f06298bc58330af376d3239401ac6b4b67167c920e005`.
The repaired subject requires its own Actions materialization and structural
source receipt; any resulting generated-evidence commit requires its own
exact-H affected runtime evidence. Source/dependency closure, scoped semantic
acceptance and final OCRM acceptance remain open. No local/htch validation.

### PRC-ORACLE-03: exact-H portable contract recovery

H `6273f9252f308355526c68e833da8dea194bb6cf`, runtime run `41550052024`,
Linux job `125635116924`, artifact `13192085740` has four assertion failures
across three portable commands (Cargo 101, capture 0; no compile failure).
Decision **FIX / TEST_FLAW**, recorded before repair: CGWA's reopen test compares
sealed proofs from different owner allocations, contrary to DPA-v1; two Session
source oracles still require the old inline abort/Runner success shape; the
node oracle requires `execute_shell` instead of the existing timeout-aware call.

Keep the production code unchanged. For CGWA, retain actual HTTP mutation,
file/metadata isolation and reopen controls; compare persisted device, role and
scopes and require old/new issuer proofs to differ. Adopt the existing owner's
same repair from `31cac7c95ec0297f7dee72d4bc40dad56aee4f01` unchanged; retain
the DPA owner/proof controls. For Session, require the exact abort delegation
and helper route/rejection, and require both provider-review release operations
to exist in order in send failure, Runner error and exact-stream cancellation.
For node invocation, require the same local execution owner, timeout argument
and existing request/result/refusal constraints. No valid assertion is removed.
Re-execute the affected existing targets and unmet actual-owner gates in Actions
at the repair H; prior positives retain their original subject identities.

Decision **PRC-ORACLE-04 — FIX / TEST_FLAW** (independent source review): the
old send-failure slice selected the earlier acknowledgment-claim refusal, where
no claimed continuation is owned. Optional index ordering could falsely pass
without finding a release call. Bind the slice to the actual `accept_routed_send`
failure and require both release operations and their order. The earlier claim
refusal remains a distinct path. This finding is source evidence, not a new
executed Actions failure.


### Frozen Actions dispatch admission — OCRM-FROZEN-DISPATCH-01

Decision **OCRM-FROZEN-DISPATCH-01 — FIX / TEST_FLAW (verification
orchestration)**, recorded before workflow repair. At H
`d95efb8a690750ad8dc78ec375ec5ff2a7ad2660`, source run `41551759333`
passed tracked-tree and structural gates, but Rust dispatch `41551761864`,
attempt 1, remained `pending` with zero jobs. The older admitted dispatch
`41541130401` for H `79920666e475920109690520cd913df40b28c0f7` still held
the same branch-only concurrency group: its renderer and restore jobs had
finished successfully, its native job had failed, and macOS job
`125624966769` remained queued. Both immutable workflow versions map
`workflow_dispatch` to
`rust-desktop-runtime-refs/heads/codex/openclaw-rust-fusion-chat-20261101-writer`.
The queued older platform job therefore retains the group while the new
frozen subject cannot enter any job. This is an evidenced workflow admission
problem; the cause of macOS runner unavailability is not established.

Keep the existing workflow and evidence owners. For the migration writer
branch, both push and workflow-dispatch events use the actual immutable
`github.sha` in the existing concurrency group. Preserve the PR group and
PR-only cancellation policy, all other branch behavior, job selection,
permissions, commands, named positives, input hashes and artifact contracts.
Continue to bind each checkout to `github.sha`; when `expected_subject` is
provided, retain its full-SHA and equality refusal before checks. The supplied
expected value is never a replacement for the actual checked-out identity.

Verification requires the repaired published subject to receive actual job
admission and its own affected compile/contract evidence through this workflow.
A queued job, successful dispatch request or source-only approval is not a
runtime pass. Existing historical runs/results remain bound to their subjects;
no cancellation, replay, weakened gate or promotion of platform acceptance is
part of this repair. The macOS queue and all unexecuted checks retain their
actual non-passing state. No local/htch executable validation.

### Interrupted generated tool history — OCRM-GENERATED-HISTORY-v1

Decision **GTH-DEFECT-01 — FIX / IMPL_BUG**, recorded before implementation.
At base `f64479b089184d5e48c12b2d0792a8c0bd52c695`, the actual generated
stream returns a provider error before draining completed tool observations or
staging its canonical checkpoint. A completed tool followed by a failed or
cancelled continuation therefore has no generated tool history in AgentStore
or the Journal, although its effect already occurred. The inactive stream's
error cleanup does not persist final state. Preserve that failed terminal;
repair history custody through the existing owners without replaying the tool.

The entry remains `create_production_runner` and the generated Agent stream;
the producer remains `CheckpointRecordingRoutedToolBridge`; the unique state
owners remain `ProductionAgentStateCheckpointSink`, the checked AgentStore
root transaction and the existing Transcript mirror. Use the supplied outer
checkpoint sink before returning the original provider failure. No alternate
log, scheduler, loop, restore grant or persisted schema is introduced.

- **GTH-01:** When generation fails, invalidate this sink's original-turn
  completion observations, including earlier invocations. A persisted failed
  invocation must never create a completed-original-turn capability or allow
  an upgrade marker to be acknowledged. Keep actual Failed/Cancelled terminal
  projection and the existing model, account and cancellation identities.
- **GTH-02:** The bounded history path requires a nonempty, checked drain of
  actual ordered tool observations, each with an observed successful result,
  and the same composition runtime's custody with zero pending workers, no
  unknown result and a started revision equal to the observation count.
  Poisoned observations refuse persistence. Unknown, outstanding, incomplete
  or absent observations do not become invented successful tool results.
  Accepted provider-checkpoint revision is not advanced by history persistence.
- **GTH-03:** Stage those exact generated steps with the original accepted user
  identity and no final assistant text. Persist through the existing outer
  checkpoint and checked-root/Journal transaction; then return the original
  failure. Do not call successful stream hooks or final-state persistence.
  Root drift or write/mirror refusal remains an error, preserves unrelated
  committed state, and cannot create a completion receipt or automatic retry.
  Report a secondary history failure without replacing the original typed
  provider failure or its cancellation identity.
- **GTH-04:** No-effect errors leave canonical bytes unchanged. Successful
  generation keeps its existing final checkpoint and completion observations.
  Transient turns without a checkpoint sink retain their existing error path.
  A fresh consumer reads the committed tool/user history without executing a
  provider/tool, duplicating entries, sealing an original turn or releasing its
  marker. Existing stored formats require no migration; rollback does not
  delete committed history or reinterpret it as permission to resume.
- **GTH-05:** Extend the existing real generated-Runner/process-reopen harness.
  Actions must cover permanent provider failure after one real file effect,
  cancellation after that effect, real provider-checkpoint-store refusal,
  unknown outcome and zero-effect refusal, concurrent canonical-root drift,
  repeated fresh consumers, and a successful control. Assert actual terminal,
  tool/HTTP counts, exact user/tool/result order in canonical state and Journal,
  missing completion authority and retained marker where applicable. Keep
  existing root, model/account, retry, cancellation and ORT controls. Reuse the
  existing Linux/macOS workflow with exact named positives and command/capture
  evidence; no local/htch execution.

Decision **GTH-SOURCE-01 — FIX / IMPL_BUG** (independent source assessment,
before correction): propagating a history encoding/staging/persistence error
with `?` would replace the original `ProviderRefusal` with `Protocol`. The
shipping Host's existing Codex review/continuation consumer matches the original
typed refusal. Keep that error authoritative, emit a content-free secondary
history diagnostic, and never retry the write. A Journal failure may follow a
durable canonical commit; refusal does not imply the prior root is unchanged.

This reuses the prior deep review at fixed U `1440bb1771372334bea23d7642bb52deb7db1845`:
`attempt-subscription-cleanup.ts` blob `c90c99156ecf10812708c221b014df9900a8ffe0`,
`attempt-session-settle.ts` blob `e5735df007a1a469257c8797e6b2a2ce12e3ff0f`,
`wait-for-idle-before-flush.ts` blob `9fa2d2954c1d3c9421a94ef5cdc22ad75b3cc3cb`
under `src/agents/embedded-agent-runner/`, and
`packages/agent-core/src/agent-loop-tool-recovery.ts` blob
`bc558648d2dbd88b153630edbea1b3772c58d993`. The cleanup contract retains required
transcript persistence after cancellation; recovery must preserve actual result
provenance. This is no new literal-read credit or source-authority promotion.
Unknown/pending-effect history, complete typed replay/UI consumption, durable
original-turn eligibility/claim/startup continuation and real-provider/platform
acceptance remain unfinished; this bounded increment does not exempt them.

The retained representation is the existing generated checkpoint format, not a
claim that every provider JSON payload round-trips losslessly. Source inspection
of the ordinary-history consumer also found pre-existing direct SendMessage
argument/result schema mismatches and lossy rich MCP normalization. Their
canonical encoding and ordinary replay consumer must be repaired before
claiming full provider-history equivalence; SQL tool cards or empty generated
objects cannot substitute for the original paired provider data.

Decision **GTH-REVIEW-R1-F1 — FIX / IMPL_BUG (P2)**, before correction.
The independent GENERAL/security/lifetime review of candidate
`4dcdf02a6d6c6cce528de6a28ec35c344a6487c9` / tree
`571817ed6697757df9ff34a30f04b212b835f542` found that successful generation
and staging can be followed by a failed final outer persistence. The source's
local error guard does not see that later error; precommit blob, Journal-prepare
or guarded-root refusals can leave an earlier successful observation readable
and appendable. Existing Host finalization separately refuses the returned
error, so this is not evidence of a false-completion or marker-release exploit.
Retain the same checkpoint sink through the actual lifecycle result and
invalidate it on every returned error, while preserving invalidation before
failed-provider history publication. Extend the existing process oracle with
prior success, successful provider generation plus a real final-write refusal,
and a later successful write after repair. The original error must remain,
completion observations must stay unavailable, and no completion receipt or
marker acknowledgement may be produced. GENERAL uses the configured model at
xhigh; the independent structure/reuse/performance lens uses gpt-6.1-sol at
xhigh. Authored tests and static review do not establish execution acceptance.

## Canonical root admission — OCRM-CANONICAL-ROOT-ADMISSION-v1

Implementation base is `a2f7ac9f6ede2e4c4561f9b82f85de7e074c7323`.
Fixed U and all existing completion/acceptance gates remain unchanged. This
batch completes the existing Runner/AgentStore admission binding; it adds no
store, lease, permission authority, replay engine or automatic recovery.

**CRA-DEFECT-01 — FIX / IMPL_BUG**, before implementation: the production
sink retains only cached checkpoint bytes. Projection reads that cache and the
generated preflight reads it again, so neither read establishes current durable
authority before model execution. Byte equality also collapses an absent root
and a present empty root, or two legacy root IDs with identical bytes. Staging
and persistence read a later live root but compare only its bytes; a concurrent
successful user of the same sink can advance its cache after another invocation
has passed preflight. Bind the original invocation's complete optional snapshot
at every boundary instead of silently adopting that later root.

**CRA-DEFECT-02 — FIX / IMPL_BUG**, before implementation: early build,
prompt and projection refusals return outside the generated lifecycle guard.
After a prior successful invocation they can leave that same sink's completion
observations available. Every returned shipping-owner error must permanently
invalidate those observations while preserving the original typed error and
the existing terminal/custody/cancellation behavior.

### Shared internal contract

Reuse `Option<ProductionAgentCheckpointSnapshot>` exactly, including the actual
`root_blob_id` and `checkpoint_bytes`. `None` is established absence;
`Some` with empty bytes is a distinct valid checkpoint. Never invent a root ID
by hashing bytes and never default an unsupported readiness reader to absence.

The existing `AgentStateCheckpointSink` gains `checked_base_snapshot()` with an
unsupported-capability Protocol error by default. Its byte convenience reader
derives from this checked result. `stage_text_turn`, `stage_generated_turn` and
`persist_staged_text_turn` receive `expected_base: &Option<Snapshot>` as their
first argument after `self`; their remaining arguments are unchanged.
`checkpoint_text_turn` keeps its signature and captures the checked snapshot
once for both stage and persist. `TextTurnCheckpointArtifacts` stays unchanged.
Existing test sinks explicitly implement the checked capability where required;
their controlled values do not become production authority.

`HostRunnerComposition` retains its checked optional snapshot, borrows its bytes
for the existing transcript route and passes the complete value to the sink
constructor. The constructor checks equality with a fresh checked store read
before mirror recovery. The sink retains one mutex-protected optional snapshot.
Its checked getter compares that complete value with the existing AgentStore
checked read and returns only on equality; mismatch/read failure never reloads
the store, reopens an owner or changes the retained expectation.

`ProductionTurnAgentInputProjection.expected_base` owns that value and
`GeneratedAgentTurnContext.expected_base` borrows it. The generated preflight
checks equality again before `TurnAgentComposition::run`; staging and outer
persistence pass this same captured expectation. Under the existing sink lock,
stage/persist require captured == retained == checked-live. Persist also checks
the artifact's prior bytes and passes the captured expectation to the existing
`handle_checkpoint_bytes_if_current_blocking` writer. After the existing
successful canonical and mirror boundary, advance the retained snapshot using
only that writer's returned root and this invocation's committed bytes.

The complete returned-error boundary of
`ProductionTurnAgentOwner::run_routed_provider_with_projected_turn_input`,
including build validation, invalidates its same sink on Err. Keep GTH's earlier
pre-publication and outer persistence guards. No invalidation can restore a
previously unavailable observation log, replace the original error, retry the
provider/tool, mint a completion receipt or acknowledge an upgrade marker.

### Required behavior and Actions oracle

| ID | Existing-owner requirement |
| --- | --- |
| CRA-01 | A valid captured absent, present-empty or legacy root admits the actual Runner; successful own publication advances the same sink for the next explicit invocation. |
| CRA-02 | Competing roots before projection or between projection and generated preflight refuse before model HTTP/tool effects; no provider checkpoint, retry or extra canonical/confirmation/Journal mutation follows. |
| CRA-03 | Absent/present-empty, same bytes under another valid root ID, and changed bytes under the same fixed ID remain distinct after an explicit checked store reload. The old sink does not rebind. |
| CRA-04 | Existing strict metadata/root/direct-reference read failures and completed owner retirement refuse through the actual checked store. Mere registry eviction is not full retirement. |
| CRA-05 | When another invocation advances the same sink after preflight, the first invocation cannot stage or persist on the successor root. Its captured expectation remains binding through the guarded write. |
| CRA-06 | Build, projection and preflight errors revoke prior successful observations permanently. An explicit repair may allow later valid work only against the original still-current expectation; it cannot revive completion authority. |
| CRA-07 | A post-durable Journal error retains the old expectation, so that same sink refuses later work before provider execution. Preserve the GTH pre-root blob-write refusal/repair positive, whose expected root never changed. |
| CRA-08 | Preserve typed MCP catalog refusal precedence, model/account identity, custody accounting, cancellation, all GTH9 controls and original-turn successful/rejection controls. |

Use the existing real Session/AgentStore/Journal, generated Runner and controlled
Responses process harness, with actual durable mutations and fresh consumers.
Require zero observed model requests/effects for admission refusals, exact
durable byte/confirmation preservation, and actual successful controls. Extend
the current projection/owner/bridge contracts for propagation and precedence.
Run affected compile, named contracts and process controls only in GitHub
Actions at the frozen exact H; inspect command/capture, counts and raw receipts.

The guarantee is checked admission immediately before model/tool execution and
a bound guarded commit afterward, not a lock held across provider execution.
Existing MCP catalog preparation precedes the snapshot read. Keep CKR's bounded
root/direct-reference policy; this does not add recursive graph validation or
new hash restrictions on legacy IDs. The generated UserMessage field 10 still
uses the hash of nonempty prior bytes and does not preserve a present-empty or
legacy fixed prior-root reference. That wire-provenance limitation, complete
ordinary-history replay, real providers/platforms and final acceptance remain
open; this internal admission binding does not waive them or upgrade a flag.

No persisted schema or checkpoint byte format changes in this batch. A strict
refusal exposes a stale or unreadable base without repairing, reopening or
rebasing it. Existing explicit Session recovery and a fresh composition remain
the way to acquire a current owner; preserve pending WAL and unknown-effect
custody through that transition. Rollback must retain those existing durable
formats and cannot treat a rejected invocation as completed or replay its tools.


Review-runner arbitration and source-review provenance are retained in
`projects/openclaw-rust/evidence/canonical-root-admission-d56b3d1d-20261103/review-decisions.json`.
They are separate from the product contract and runtime execution evidence.
