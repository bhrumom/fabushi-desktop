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
