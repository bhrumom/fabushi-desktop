# MCP account-list publication retirement

Contract: `MCP-ACCOUNT-LIST-RETIRE-v1`  
Implementation base: `ae8774e38fd04ba07d571dd5c5ea42ddb600f977`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`  
Status: implementation and exact-H verification pending

## Problem and existing owner

The shipping account-transition cleanup resets the existing Electron MCP runtime,
which disposes `SandMcpManager`. Its first account listing can still be waiting
on account A's captured credentials. Current dispose only clears auth watches;
the late listing retains its generation and can call the shared settings store's
`scopeToAccount(A)` after replacement manager B has installed its scope. That
clears B's disabled-tool map, consumed by the real desktop discovery executor.
A late unavailable response has the same risk because scope publication precedes
the unavailable check. This is source-proven, not yet an executed failure.

Retire publication in the existing bridge manager. Rust Settings, auto-review,
Session, tool-effect custody and account authority remain their current owners.
This does not introduce policy, a permission store, or another MCP runtime in
Electron, and does not establish complete Rust permission migration or a remote
authorization bypass.

## Requirements

- MR-01: disposal establishes an irreversible retired state synchronously before
  callbacks or asynchronous cleanup. Repeated disposal is harmless. In-flight
  account responses from that manager cannot publish a scope, config, cached
  display, migrated instructions, or scope-applied callback afterward.
- MR-02: use the manager's existing account-generation/publication boundary.
  Pending callers settle through the declared empty/unavailable result without
  returning a retired account cache as current authority. Clear retired local
  projections as needed. Do not invoke cache invalidation that starts a new
  account warm pass merely as disposal cleanup.
- MR-03: reload, background refresh and account-list resolution on a retired
  manager cannot start a new account fetch or restore publication authority.
  Existing current-manager loading, cache/failure behavior, discovery, toggles,
  enabled-tool execution and auth-watch cleanup retain their current semantics.
- MR-04: the real public runtime reset and replacement-manager path must keep
  B's exact persisted scope and disabled map when a held first A response is
  released, for both successful and unavailable outcomes. B's real discovery
  must still deny its disabled tool with zero execution calls. A fresh B enabled
  tool and stable-account loading provide positive controls.

The chain is captured account I/O → existing manager generation/lifetime →
shared existing settings store → existing discovery denial check. Use weak or
retired-owner refusal at this bridge boundary; do not infer that canceling a
local observer terminates remote work. Account-scoped async construction races,
already-admitted remote work, full post-dispose mutation reachability and durable
cross-process reconciliation remain separate open contracts.

## Implementation, verification and release

Allowed code paths are `source/shared/node/mcp/mcp-manager.ts` and the existing
`source/electron-main/mcp/mcp-runtime.contract.test.ts`. Root integrates the
existing bundle/workflow selection and narrow source/traceability references.
Use real runtime reset, manager, account-response projection, settings persistence
and discovery. Stub only account/network/backend I/O; deterministic held promises
must decide ordering without sleeps. Test first-load success and unavailable
responses, repeated reset/dispose, no new retired-owner fetch, exact persistent
denial preservation and stable/new-owner positives. These are local lifecycle
controls, not live account/provider or packaged acceptance.

Add the existing runtime contract target to the active OCRM plugin bundle/run
lane; retaining it only in the skipped general lane is insufficient. All
executable checks run in GitHub Actions against the final H. No old pass, authored
test or metadata flag proves this batch verified. No persistent migration or
wire change is needed; reverting restores the stale-publication risk without
deleting settings. Existing diagnostics remain secret-free.

Fixed-U lifecycle sources read at exact identity: `src/mcp/tools-stdio-server.ts`
blob `ef5a603bce25ebe2247966c40932e8539ae80632` and
`src/mcp/tools-stdio-server.lifecycle.test.ts` blob
`c3d3ee2748857f4915245ce02c07a28860c89478`. Their closing-before-drain invariant
supports lifetime discipline, not account-policy parity. Required local source
and actual producer/consumer reads are recorded in the bounded investigation;
no recursive coverage, historical reading or global completion flag is promoted.


## Runtime construction custody — `MCP-RUNTIME-CONSTRUCTION-v1`

Implementation base: `7a0254d4133080877cbfb2d9d4831acb231e673f`.
Fixed U is unchanged. This follow-on contract addresses the existing Electron
runtime's resource construction/publication boundary, not a new execution or
permission owner. The shipping adapter awaits `ensureMcpManager()` before
calling the returned facade. Today concurrent ensures can construct different
managers, and reset/dispose sees only a completed manager: a pending result can
publish after retirement and be used by the old operation.

- MC-01: one existing runtime owns one current construction attempt. Concurrent
  ensures share that attempt and its eventual manager. Reserve the pending
  promise before invoking the factory, including a synchronous reentrant
  factory. If reset/dispose invalidates the attempt before invocation, do not
  invoke it. No alternate manager registry or account scope is introduced.
- MC-02: reset detaches the current attempt synchronously before external
  cleanup, then disposes its already-created manager through that exact attempt.
  Reset permits a later ensure. A late old result is disposed and rejected;
  neither an old caller nor cleanup/finally may adopt or erase replacement B.
  Do not transparently retry an old operation against B. A current construction
  failure retires its callback identity and permits a later fresh attempt.
- MC-03: runtime disposal permanently refuses future ensure/factory admission.
  In-flight creation may settle later and must be retired without publication.
  Teardown of an already-created manager is observable through the existing
  async cleanup caller, and each manager is disposed at most once by this owner.
  Do not wait indefinitely for an unresolved factory merely to return reset or
  shutdown; its eventual settlement remains owned by its construction promise.
- MC-04: callbacks supplied by this runtime for scope-applied, connector auth,
  discovery failures and diagnostics check the exact live attempt. Its auth
  completion observer checks before broadcast and again before Host refresh,
  because broadcast can synchronously reset the runtime. Startup cleanup and
  observer installation cannot make a retired manager current; recheck after
  synchronous external setup callbacks. Current callback behavior is retained.
- MC-05: the captured `getAccessToken` port rejects a retired attempt at entry,
  after awaiting the auth service before calling its token method, and after
  awaiting the token before returning it. An old construction must not newly
  obtain or return replacement-account credentials through this runtime port.
  This does not revoke a credential or an I/O operation already handed out.
- MC-06: exercise the actual runtime with controlled factory/auth/cleanup ports:
  concurrent and reentrant ensure; reset before invocation and during pending
  creation; old success and failure after B; current failure then successful
  retry; disposal and repeated cleanup; reentrant observer/broadcast reset;
  stale reports and all token-await boundaries. Include the real shipping
  adapter's routed operation when practical to prove a retired ensure prevents
  the facade call. Keep all MR account persistence/denial and original callback
  assertions. No timers or sleeps decide ordering. Run only in Actions.

Allowed implementation paths are the existing `mcp-runtime.ts` and
`mcp-runtime.contract.test.ts`; root owns this contract and narrow existing
workflow/ledger/RTM references. The active MCP contract bundle already selects
this file, so a new workflow or test-only runtime is unnecessary. Use the
existing factory, facade and cleanup owner; Electron keeps only the resource
bridge lifecycle and Rust retains policy, registration and execution authority.
No persistent schema, token format or wire change is needed. Revert restores
the old construction race without rewriting user data.

The real desktop factory starts discovery warming before returning its facade.
This bounded publication fence cannot retract arbitrary factory-internal
side effects before settlement, work admitted through an already-returned
facade, or the separate official OAuth `changed()` path. Those lifecycle and
full Rust migration contracts remain open. Late cleanup failures must remain
observable without changing B's authority; a successful source review alone
is not final-H verification. Reuse only the unchanged fixed-U closing/lifetime
source identities and read bounds above; no new full-read or parity claim.

## Retired Box bridge requests — `MCP-RUNTIME-BOX-ADMISSION-v1`

Implementation base: `ecd978f7d2d1ae7511613147302f022155981c9f`; fixed U is
unchanged. The construction owner currently passes its four Box request ports
through unchanged. The real desktop factory retains them in discovery, whose
configuration push and execution paths await configuration or prior work.
Reset can retire that factory/manager during such an await, after which its
captured port can still start a new Host request. Clearing account projections
does not revoke the captured request callback. This is a source-established
bridge-lifetime gap, not an executed remote authorization bypass.

- MB-01: the existing runtime's `loadBoxMcpServers`, `listBoxMcpServers`,
  `listBoxMcpToolsRaw` and `executeBoxMcpToolRaw` callbacks must check their
  exact current construction attempt immediately before calling the underlying
  port. A retired attempt rejects without any new port call. Apply this after
  reset, permanent disposal and failed construction; a later manager does not
  authorize an earlier callback. Use the existing attempt identity only.
- MB-02: current callbacks forward the original arguments once and preserve
  the underlying successful value or rejection. There is no asynchronous gap
  between the lifetime check and invocation, no replacement-account retry and
  no policy, registration, credential or execution-owner substitution.
- MB-03: a request that already crossed the port remains owned by that call.
  Its eventual value or error still reaches its original caller after reset;
  do not discard a completed tool result or infer cancellation from local
  retirement. This gate cannot retract an in-flight Host request or establish
  cross-account ordering inside a remote consumer. Those contracts remain open.
- MB-04: use controlled promises and actual runtime callbacks to cover every
  port, current success/failure, old callbacks after all retirement paths,
  fresh replacement success, and already-admitted late outcomes. Exercise the
  real shipping adapter with a facade that pauses before its captured Box
  execution port, then reset/dispose and require zero old request dispatches.
  Preserve all MC/MR assertions. No sleeps or fixture retries decide order.
- MB-05: extend the already-selected complete MCP runtime contract file and
  run its existing bundle/build lane on the resulting exact H in Actions.
  A source review or a passing older H does not verify these new controls.

Scope is this spec, `source/electron-main/mcp/mcp-runtime.ts` and its existing
`mcp-runtime.contract.test.ts`. Their canonical ledger/RTM references already
point to these files; no acceptance state changes. Electron only guards the
lifetime of its captured IPC bridge callbacks. Host/Rust retains permission,
configuration and execution authority; this is not a complete Rust MCP
permission-lifecycle migration. Returned facade methods, in-flight remote
mutations, official OAuth callbacks and restart reconciliation retain their
separate existing limits. No schema, dependency, wire or storage change is
needed; rollback restores late bridge dispatch without rewriting user data.
Reuse only the fixed-U closing/lifetime evidence and read bounds above.

## Resolved MCP dispatch bridge — `MCP-DISPATCH-READY-v1`

Investigated H: `48a9132ed496bd755919cb79555705b7b18f447f`; implementation
base `485c388b082f71d9e6fb87535e87240551d19dde` changes only the independent
Session guard oracle. Fixed U remains unchanged. Classification is a
source-demonstrated implementation defect; no executable failure is claimed.

The real Rust Runner/Host routed tool bridge crosses the existing Coordinator
relay and Electron production binding into the current desktop MCP facade.
For legacy account connectors, `createMcpToolsDiscovery.executeTool` reads a
display row before provider resolution. A cold current manager has no row yet,
so it skips the existing persisted disabled-tool predicate. Provider/account
resolution can then publish the row before the execution RPC is called, without
another check. The tool can have been disabled before invocation; this defect
does not depend on retroactively canceling admitted work.

The existing SAND `settings.json` field `mcpDisabledToolsByServerId` remains
the only durable record. This batch corrects bridge sequencing around its
existing predicate; it adds no grant, registry, policy rule, storage or execution
owner. The actual legacy per-tool predicate is still in this adapter. Rust
Settings exposes the same record but the real routed execution currently has
no Rust per-server/account/tool semantic admission API. This correction does
not complete that migration; its identity/currentness contract remains required.

- MD-01: when identity/provider preparation completes, apply the current existing
  disabled-tool predicate to the current matching display row immediately before
  calling the HTTP execution port. Do not capture a null initial row or old map
  across the awaited preparation and treat it as a grant.
- MD-02: apply the same rule to Box execution after its awaited configuration
  preparation/push. A disable committed while preparation is held must prevent
  the subsequent execution-port call. Preserve an existing known-disabled early
  refusal when useful; no new policy semantics or alternate predicate owner.
- MD-03: preserve original provider/tool names, arguments, audit identity,
  established denial/error shape, custom-instruction behavior and existing
  unavailable/failed-preparation paths. Do not invent a server identity, retry
  against a replacement account, or change built-in/local configuration
  compatibility. Keep MR, MC and MB current-attempt/account retirement checks.
- MD-04: after a call has crossed its execution port, preserve its actual late
  success or rejection and original caller even if disable/retirement follows.
  Do not replay, synthesize cancellation, discard the result, or infer that a
  local denial undid an external effect. Broader Rust custody remains unchanged.
- MD-05: use the real runtime, manager, definition source, persisted Settings and
  discovery owner, with controlled account/backend/Box I/O only. Hold the first
  account response; establish the same-scope disabled record before the call;
  prove the actual initial row is unavailable; release valid identity and
  require the existing disabled refusal with zero execution calls and unchanged
  permission/scope values. Include an enabled sibling and a retained real
  descriptor through a fresh current owner. A synthetic null-row core alone
  cannot establish this production case.
- MD-06: cover current warm denial, Box preparation paused across a committed
  disable, failed/unavailable preparation, and already-entered late success and
  error. Keep all existing MR/MC/MB assertions. Controlled promises, not sleeps,
  establish order; helper predicates may not replace production enforcement.

Allowed source paths are `source/shared/node/mcp/tools-discovery.ts` and the
existing `source/electron-main/mcp/mcp-runtime.contract.test.ts`; root owns
this specification and integration. The existing complete MCP contract
bundle/build lane already selects this target. Freeze and independently review
the full three-path scope; execute only in Actions on the final integrated H.
No new dependency, schema or wire field is required. Rollback restores the
cold-dispatch gap without rewriting user settings.

Reuse only the exact-U lifetime/admission-versus-drain references above and
the bounded MCP investigation's explicit read ranges; no full upstream review
or global flag is promoted. Real remote effects, provider accounts, OAuth,
durable cross-process revocation, authoritative Rust account/registration
handoff and all full OCRM/FPM acceptance gates remain open.



## Received routed catalog integrity — `MCP-RECEIVED-CATALOG-v1`

Discovery base: `4ed7f5184476a39d02856d4d09ba1f8166ea0676`; fixed U is unchanged.
The actual App decoders currently discard malformed rows, the production turn
projection converts all discovery failures to empty vectors, live canonical
projection stringifies failures, and dynamic construction suppresses them.
Claude's HTTP endpoint retains its previous map on every refresh error, while
the actual attempt returns only the external CLI result. Correct this one
received-catalog behavior in the existing Rust App/Runner/inference owners.
This is not a new registration, policy, account, permission, or runtime owner.

### Admission and compatibility

- RC-01: both actual App list decoders reject the entire response only when
  its top level is not an array or any row lacks a string `name`,
  `providerIdentifier`, or `toolName`, or violates the versioned internal
  origin contract added by `MCP-ORIGIN-DISPATCH-v1` below. Introduce internal
  `ProviderSessionError::McpCatalog(McpCatalogRefusal)` in the existing error
  owner. The copyable refusal contains only a `McpCatalogKind` (`Tools` or
  `MetaTools`), optional zero-based `row`, and `McpCatalogReason`
  (`ExpectedArray`, `NameMustBeString`, `ProviderIdentifierMustBeString`,
  `ToolNameMustBeString`, `OriginInvalid`). Never include the invalid value, descriptor,
  account identity or payload in this error.
- RC-02: retain every currently accepted value: empty arrays, blank required
  strings, public/raw name differences, duplicate rows and order, arbitrary
  explicit inputSchema JSON including null, and the existing missing-schema
  default. Optional non-string descriptions/IDs remain absent; non-null plugin
  and marketplace JSON and ignored unknown fields retain their current meaning.
  No uniqueness, normalization, reserved-provider or native-alias rule is added.
- RC-03: the normal production preparation propagates this specific refusal
  before provider execution, tool dispatch or review. Preserve the existing
  ordinary discovery-unavailable -> empty snapshot positive. Live projection
  obtains the typed vector before passing accepted data to the same canonical
  grouper; preserve first-seen-provider grouping and within-provider order.
  Keep the public String-based generic MCP port and Box adapter unchanged.
  Change the existing toolset builder to return Result: only an observed
  McpCatalog refusal escapes dynamic construction; ordinary failures retain
  the prior fallback. Explicit empty-subagent exposure stays empty.

### Attempt lifecycle and effect custody

- RC-04: the existing sequential HTTP worker owns its current map and an
  attempt-local optional catalog refusal. A valid refresh atomically replaces
  the map and clears only that refusal, including valid `[]`. Structural
  refusal clears the map and records the refusal. Ordinary availability
  failure changes neither state: it preserves a previously valid map and
  cannot resurrect a map cleared by structural refusal. Calls to removed
  names must not reach downstream execution or review. Preserve current
  duplicate last-wins behavior and actual tool success/server-error results.
- RC-05: the actual Claude attempt closes its owned endpoint and joins its
  worker before reading the final refusal. Keep the observation independently
  of a worker return value; a failed join cannot be interpreted as proof of a
  clean catalog. Preserve actual user cancellation, explicit unknown outcome,
  and authentication/configuration errors. Preserve true pending/unknown
  custody before any catalog override. Otherwise an unresolved catalog
  refusal becomes the typed terminal error even when the CLI returns success,
  Transport/Protocol, or watchdog cancellation. A later valid refresh permits
  the original CLI outcome. Do not retract text already emitted by the CLI:
  a completed CLI response can already have produced text before the attempt
  observes the catalog terminal. A fresh turn starts a new endpoint/fault state.
- RC-06: this type is permanently ineligible for automatic retry, including a
  simultaneous watchdog. In the existing adapter, when custody has no pending
  or unknown effect, return this refusal without retry even if completed known
  effects lack a checkpoint. Retain their receipts and revisions; never
  manufacture a checkpoint, discard a result or authorize replay. True
  pending/unknown and user cancellation retain the existing owner precedence.
  All other errors retain their current retry-block policy. Reports and the
  public Transcript classifier use a fixed permanent protocol classification,
  before message heuristics, without sensitive values.

### Real-path oracles and scope

- RC-07: actual App CoordinatorToolRelay/GatewayEventHub controls exercise both
  list methods, malformed first/middle/last rows and non-array responses, typed
  secret-free refusal, zero tool/review dispatch and drained waiters. Valid
  controls preserve the full permissive language above. Keep existing
  downstream ToolOutcomeUnknown and review-denial assertions.
- RC-08: actual production projection, canonical grouping, dynamic construction
  and preparation/execute boundaries distinguish structural refusal from
  ordinary unavailability. Preserve valid frozen snapshots, native wrappers,
  empty-subagent behavior and all existing builder positives.
- RC-09: actual HTTP controls exercise valid -> call -> invalid -> old-call
  refusal -> unavailable -> valid restore -> empty clear; separately retain
  valid -> unavailable -> old-call success. An entered held list released as
  invalid during endpoint close proves observation occurs after join.
- RC-10: exercise the public routed runtime through its real private production
  attempt, actual CLI launch and HTTP endpoint, using owned child processes and
  child-scoped CLAUDE_CODE_PATH. Reuse current_exe/--exact/nonce/PID/bounded
  cleanup conventions; do not mutate HOME/PATH or process-global env, add a
  dependency, or fake the attempt executor as proof of this connection.
  Cover CLI success/nonzero, refresh recovery, watchdog, user cancellation,
  next-turn isolation, and successful or server-error tool result followed by
  invalid refresh. Require one launch/no retry and exactly one original tool
  result/call identity. Known completed effects retain revision1/checkpoint0
  with no pending/unknown effects. Retain existing unknown and delivery-failure
  tests; an unreachable synthetic custody tuple is only a policy control.

Product scope is App main, provider_session, production_turn_input_projection,
turn_agent_composition, tools/turn_toolset, routed_provider_runtime,
production_turn_run_shell_adapter and Transcript turn_runtime. Root serializes
the error contract, composition, spec and existing Runtime selectors. Extend
existing App/projection/toolset/routed-provider/adapter tests; adapt the six
existing builder test callers and the credential automatic-attempt observation
match without deleting assertions. Existing native list wrappers propagate
errors and need no policy changes. All executable verification stays in
Actions, after the shared current product gate, at the final integrated H.

The finite reference search used exact 4ed Host blobs, then semantically read
matched callers and 18 native wrapper list branches; it is not a full read of
every fetched file. Fixed-U references retain their precise existing identity:
plugin-tools-handlers.ts `a4c6ad2286313226007d2527ed5929494dd21aa2`
(full129 lines) preserves duplicate last-wins and the scheduler alias;
tools-stdio-server.ts `ef5a603bce25ebe2247966c40932e8539ae80632`
(full203 lines) supplies lifecycle context. The bounded tools.ts438–746 and
plugin-tools-serve.test.ts184–215 readings support compatibility only; do not
promote them to full-file reads. Their inventory links and historical evidence
remain unchanged. This HTTP/Runner repair does not close stdio wiring, plugin
SDK compatibility, native provenance, account-cache identity, Settings writers,
per-tool Rust authority, real services or full OCRM/FPM acceptance. The existing
OCRM ledger/RTM and eight false global flags remain authoritative.

No persistent data or wire field is added. Rollback restores the permissive
partial-catalog/erased-error behavior without rewriting user state; it cannot
undo an already executed tool. Freeze and independently review the complete
delta, then require the selected exact-H compile, real-path, retry/custody and
existing regression evidence before claiming this batch's behavioral checks.

### RC-11: direct observation of the new subject's normal campaign

The completed 4ed Host whole-log request returned `Transport closed`; no raw
output was received. Preserve that unread evidence gap and its successful API
metadata separately. Large prior consoles and the existing helper's full-log
`cat` support reducing replayed volume, but do not establish the transport
failure's cause or a connector byte limit.

Before this batch's normal Actions campaign, retain the existing
`run_recovery_check` command invocation, complete raw log, quoted command,
original exit TSV, exact-subject receipt and always-uploaded seven-day artifact.
Replace only its final console replay with bounded JSON records containing
original Cargo target/harness lines and bounded failure context with file/line
provenance. ANSI removal is for recognition only. Encode both `##[` and `::`
as JSON Unicode escapes; emit a mixed-marker canary through the actual path.
Keep feature-graph output as a bounded original-line exception. Do not aggregate
child summaries into parent counts or treat capture completion as product approval.

The cumulative projection budget is 2 MiB / 12,000 records, reserving a terminal
receipt or refusal. Keep all required harness rows; overflow/missing raw data/
capture failure is non-passing with full raw artifacts retained. Optional
failure context is bounded to 240 lines and reports omitted context. A nonzero
product command retains its original return status even if capture also fails;
a zero product command requires successful capture. Check these wrapper branches
with owned normal-campaign instrument fixtures, including nonzero command,
capture failure and the combined case. Controls also cover colored target
recognition, compiler-noise rejection, marker round-trip, zero-test refusal,
non-test cargo-tree/no-run distinction and budget refusal. No product rerun,
new workflow/job or weaker assertion is introduced. Post-command original-exit
and capture-status TSV write failures follow the same precedence and have owned
zero/nonzero controls. Terminal refusal uses the same cumulative accounting and
persists a refusal latch before console emission; subsequent calls remain
non-passing without more projection output. If accounting or refusal-latch
persistence is unavailable, return capture failure without unaccounted output.
An owned actual-projector control reaches the byte/record boundary and invokes
it again, checking bounded output, retained raw logs and outcome precedence.

Freeze the new MCP named-case set from the authored App, projection, toolset,
actual HTTP/CLI and retry-policy tests before the run. Acceptance requires each
actual passing named row under its correct command/target and positive executed
counts; absent, ignored, wrongly targeted or zero-run cases remain unmet even
when another test in the command passes. The projector exposes original
observations; root applies this batch's named-case oracle before claiming the
behavior verified. Early Host/App/Native/Coordinator console producers remain
unchanged, so whole-log retrieval is not guaranteed by this scoped repair.

The existing Linux `sqlite-native-path` job also exercises the same Rust owners
using its current Host cache and portable case runner. Add separate preparation,
projection, toolset and routed-runtime targets, the exact retry-policy library
case, the five other adapted builder targets and the credential-context target.
Keep all inherited Memory/Session/Automation/media cases. Bind the two App cases
to the existing uniquely identified shipping App test executable, preserving
exact listing and one-pass/zero-failed/zero-ignored checks. The fixed 18 new
named positives must pass under their own command/target; child summaries do
not replace the parent harness. Retain all raw logs, commands, digests and
always-uploaded artifacts. This is controlled Linux HTTP/CLI/relay evidence,
not real provider or packaged platform acceptance. RC-11's projection budget
still applies only to the existing macOS recovery helper.

### RC-11 recovery: production Todo fixture (2026-11-02)

Decision **RC-11-TODO-01 — FIX / TEST_FLAW**. At subject
`aa5a448e5758bd2b46aed92e699f56861ad0acdc`, Runtime run `41543476338`, Linux
job `125613140173`, the `received-catalog-builder-regressions` command exited
101 with E0277 at `sand_multitask_todo_tool_contract.rs:177`: `SandAgentDb`
does not implement `MultitaskTodoState`. The production adapter is already
`ProductionMultitaskTodoState`, which reads/writes through admitted Session
operations. The old fixture predates this owner boundary and cannot decide the
durable Todo behavior. Repair only its construction to match production; retain
the actual materialized Session, raw persisted-KV assertion, merge/replace and
rendered-result assertions. Rebuild the real adapter as well as the bridge so
the persistence assertion does not depend on retaining an adapter instance.

The other fourteen portable commands have successful command/capture outcomes;
the sixteen portable new MCP cases and eight channel cases have actual unique
positive rows. Retain these exact-aa5 observations without rebranding them as
new-H evidence. The App cases were skipped. The later LPG receipt failed only
because the skipped Automation step left its required MCP-composition receipt
absent; its own Host, Coordinator and App commands passed. Keep that dependency
gate and all original assertions. Recover the failed builder command and the
unreached App/Automation/dependent receipt checks on the new frozen H through
the existing scoped workflow; no runtime trait implementation, fallback, oracle
deletion or local execution is authorized by this fixture correction.

## Official MCP bridge retirement — `MCP-OFFICIAL-LIFETIME-v1`

Implementation source base: `0c7da50fcf457184f38d6a61b73f8e32810cff49`.
The independently assessed four source paths are unchanged from the assessment
base `78aa967cc9400bb5c207441b2a78dbf7021d596c`; incoming plugin/media changes
through `8e5cee76ebbe6e748dc6a8dc87d73142bf04109b` do not overlap them.
Fixed U and all earlier MC/MR/MB/MD contracts remain unchanged.

The actual default production factory supplies official OAuth with a raw
account-service/token getter and a direct broadcast/Host-refresh callback.
Both bypass the runtime's existing current-attempt fence. An entered poll,
acknowledgment or disconnect cleanup can complete after reset and notify the
replacement. Separately, `OfficialMcpService.scope()` checks disposal only
before its awaited scope lookup; a final held lookup can resume an old
mutation and newly invoke the shared same-account vault write after disposal.
The source supports stale same-account image overwrite, not a demonstrated
remote cross-account authorization bypass. These are three admission edges
of one existing official-service resource lifetime, not new OAuth policy.

- MO-01: route the default official OAuth credential getter through the existing
  factory option `getAccessToken({ backendUrl })`. Preserve its entry,
  post-auth-service and post-token current-attempt checks, original backend,
  current success/error and official unauthenticated message. An old getter
  cannot newly obtain or return a replacement account's credential. Disposal's
  best-effort OAuth cancel uses that same retired getter; do not retain a token
  or retry through the replacement merely to make cleanup succeed.
- MO-02: add one generic factory callback `onAuthChanged(completion)` to the
  existing runtime options, using the same private forwarding helper as the
  existing auth observer. Check the exact attempt before broadcast and again
  afterward before refresh, including synchronous broadcast-triggered reset.
  Preserve the official `{ status: "changed", provider: "fabushi-official" }`
  event and its refresh with no completion argument. The legacy observer still
  forwards its completion to refresh. Both paths use the existing attempt;
  introduce no second epoch, callback registry or account authority.
- MO-03: reuse the official service's existing irreversible disposed latch.
  Check it after the awaited scope lookup and immediately before invoking the
  vault write, with no await between that final check and invocation. A scope
  continuation or queued old mutation that resumes after disposal must reject
  without a new write, even when a fresh service uses the same account hash.
  Preserve the existing per-service mutation queue, scope equality, expected
  revision, state schema, native vault and current operation behavior.
- MO-04: work already admitted at an underlying network/write port retains its
  actual outcome and original ownership. Do not replay, roll back, synthesize
  cancellation or claim local disposal undid a durable write/remote effect.
  A facade may still refuse its later read after the admitted write completes;
  do not promise that every old facade call succeeds. The credential getter's
  fence is not a new final HTTP-admission gate after it has returned a token,
  nor proof that the remote broker processed cancellation or expires an attempt.
- MO-05: extend the already-selected `mcp-runtime.contract.test.ts` using the
  actual default `createProductionMcpOAuthPorts` factory, runtime, desktop
  facade and official service. Control auth/network/vault I/O only. Hold both
  auth awaits across actual reset/replacement; prove no new old token call or
  broker dispatch at the respective boundary, preserve current backend/token
  positives and cover cancellation initiated by real disposal. Test-authored
  guarded callbacks alone cannot prove the default production wiring.
- MO-06: exercise actual official poll failure and ready/acknowledgment,
  disconnect cleanup and current callback producers with controlled promises
  and an owned timer control. Old completions after reset must produce zero
  new broadcasts/refreshes; current completions retain exact payload/argument
  shape. A broadcast that synchronously resets permits that one broadcast and
  suppresses its later refresh; a fresh replacement remains usable. No sleep
  or fixture retry decides ordering, and all global/prototype I/O controls are
  restored in finally without affecting unrelated tests.
- MO-07: hold the final scope lookup before a real old-service mutation writes,
  dispose/reset, let replacement B commit its same-account image, then release
  A and require zero A writes and exact unchanged B bytes. Include a current
  successful write and a separately already-invoked held write to distinguish
  refusal from admitted outcome. Additionally qualify the last pre-write check
  independently: a real service with controlled scope I/O must retire after its
  scope check has completed but before the mutation continuation invokes write.
  Register the I/O completion notification before the awaited scope consumer,
  then use that notification to queue retirement between the two continuations;
  do not replace the production guard or guess elapsed delays. Require zero old
  writes, unchanged stored bytes and a current replacement write positive.
  Retain the default-composition cases and all prior MC/MR/MB/MD and official
  controls; no credential or payload is added to diagnostics.

Allowed product paths are `source/electron-main/mcp/mcp-runtime.ts`,
`source/electron-main/adapters/mcp-oauth.ts`,
`source/electron-main/mcp/official-mcp-service.ts` (lifetime checks only), and
the existing runtime contract test. Root serializes this spec and any narrow
existing OCRM evidence references. The complete runtime contract target is
already in the active plugin/Skills bundle and renderer Actions lane; no new
workflow, dependency, persistence migration or wire schema is required.

This preserves Fabushi's existing official connector behavior through its thin
resource/credential bridge. Rust policy, registration, permissions, execution
and state ownership are not replaced or duplicated. The existing official
service still contains policy awaiting the full Rust migration; this bounded
repair does not accept that remaining work or real provider/account behavior.
Reuse only the precise fixed-U closing-before-drain source references above.
The assessment separately reads the actual account transition, production
composition, OAuth, facade, vault and native store consumers; it is not new
upstream coverage or a full-read claim on unrelated modules.

Freeze the complete five-path candidate, obtain both independent source lenses
and run the existing affected build/contract selection only in Actions at its
exact published H. Scoped G01/G02 authority, external services and full OCRM
acceptance remain open. Rollback changes no persisted format and restores these
late-admission risks; it cannot retract already admitted work.

## Original routed execution source — `MCP-ORIGIN-DISPATCH-v1`

Specified before implementation on `14faff5d9605c99df1494303f1aa0ce31bcfd420`;
fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`. The actual
Coordinator catalog currently loses its producer incarnation in the Rust
descriptor and canonical in-process projection. After AutoReview, native
execution calls `ensureMcpManager()` and can borrow a replacement. The legacy
HTTP interceptor then awaits credentials, machine ID and privacy headers before
its transport handoff. Close this one execution loop through existing owners.

### Producer, custody and contract

- **OD-01 / native producer:** each concrete Coordinator control server owns a
  fresh opaque UUID source ID and a synchronous assertion on its own settlement
  state. Pass that native context separately from parsed request arguments;
  serialized fields cannot create it. Bind the routed executors to the exact
  account-runtime object that created them, and require its existing current
  binding and lifetime. A same-account child relaunch gets a new source ID.
- **OD-02 / manager attempt:** add a fresh opaque UUID ID to each existing MCP
  runtime Attempt when it is reserved. Listing captures that Attempt before an
  await and checks the original source/account/Attempt before publishing rows.
  Execute requires the expected still-current Attempt before waiting for its
  manager and again after the wait. A missing or retired Attempt must not
  construct, await or select a replacement. Reset retires before cleanup.
- **OD-03 / internal wire:** production routed rows and execution requests
  carry `origin: { version: 1, coordinatorSourceId, mcpAttemptId }` outside tool
  arguments. Both IDs are canonical lower-case hyphenated UUID strings,
  validated as a bounded whole object;
  unknown extra fields do not become authority. Missing/malformed/wrong-version
  origin rejects the entire received catalog with fixed `OriginInvalid` through
  the existing permanent `McpCatalogRefusal`. Preserve all RC-02 public-field,
  duplicate, ordering and schema compatibility. This internal contract is
  deployed with its producer/consumer in one candidate; no origin fallback is
  permitted for an old production catalog.
- **OD-04 / Rust custody:** retain origin in `RoutedToolDefinition` and bind it
  to the existing Host relay's private incarnation. The relay alone creates
  that local binding; a wire UUID is not a Host lease. Carry the same binding
  through canonical in-process MCP grouping, both turn snapshots and dynamic
  toolset reconstruction. Provider schemas, model payloads, canonical protobuf
  and persistent transcripts must not expose or serialize the local binding.
  Built-in tool definitions and decoded external protobuf have no Coordinator
  binding; the actual Coordinator bridge refuses such a definition before
  execution. Do not recover metadata by an ambiguous tool-name lookup.
- **OD-05 / review and dispatch:** the actual Rust bridge retains the original
  descriptor across AutoReview and checks its relay binding after review,
  before execution publication. Native intake compares its serialized source
  ID to the independently supplied source context, then validates the exact
  originating account and Attempt. It forwards one request-local synchronous
  assertion through the facade, legacy discovery and backend execution helper.
  No global current-request callback, re-listing, fresh-origin substitution,
  second permission registry or additional runtime owner.
- **OD-06 / final HTTP admission:** in the actual Connect interceptor, execute
  the captured assertion after every credential/machine/privacy await and after
  the request-ID observer, immediately before `next(request)` in the same
  continuation. The callback type must exclude Promise-returning functions
  (for example `() => undefined`); do not accept an ignored asynchronous check.
  Use a request-scoped execution client/transport when a callback is supplied;
  existing unguarded listing/auth clients retain their behavior. Preserve exact
  provider identifier, public and raw tool names, arguments, call ID and Agent
  ID. Existing legacy disabled-tool checks remain in force.
- **OD-07 / result custody:** only an owner-created typed pre-dispatch
  retirement may return the generated `rejected` arm with fixed
  `MCP_ORIGIN_RETIRED`. The backend helper may keep a request-local witness set
  only when its own final callback observes that typed refusal; this preserves
  classification if Connect wraps the error. Never inspect an arbitrary error
  message or recheck currentness in catch to infer no effect. Once `next` has
  been entered, preserve the actual late success, server-declared failure or
  uncertain transport error; no post-effect rejection, rollback fiction or
  retry. Lost reply or closed control port retains the existing Rust unknown
  custody even when native work independently stopped before transport.

### Implementation and required causal checks

The existing shared Coordinator RPC definitions own the internal DTO; the
existing shared MCP result factory owns the typed bounded refusal and generated
result constructor. Rust inference/relay/App/port/turn owners carry and validate
custody. Native control-server/account-runtime/MCP Attempt owners guard their
existing objects; the facade/discovery/HTTP adapter only passes that context to
its final resource handoff. No persisted format, dependency, Agent loop,
session store, scheduler, marketplace or permission truth is added.

- **OD-T1:** actual Rust relay/EventHub and both App decoders retain origin;
  malformed catalogs fail with redacted typed reasons. Hold actual AutoReview,
  retire/replace the source, then prove zero execute publication for a foreign
  local binding and success for a fresh binding. Actual canonical grouping and
  both turn projection paths preserve per-row identity, including duplicates.
- **OD-T2:** actual production Coordinator adapter, control server/executors
  and MCP adapter refuse an old source's request after account replacement and
  refuse replay of its serialized origin on the new source. Manager-B factory
  and HTTP counters stay zero for stale execution; a fresh-B catalog/call works.
  Also relaunch a child within one account lifetime and prove source retirement.
- **OD-T3:** held factory/list/credential completions followed by reset cannot
  publish an origin or enter transport. A stale execute never creates a new
  Attempt. Preserve all existing runtime construction, official lifetime and
  Box-boundary checks.
- **OD-T4:** exercise the real generated backend client and interceptor with a
  held privacy await after credential acquisition. Retire the source or Attempt,
  release the await, and require zero actual transport `next` calls plus the
  definitive rejected result. Current control reaches `next` once with exact
  request fields. An arbitrary same-message exception is never a known refusal.
- **OD-T5:** latch actual `next` entry, retire/reset, and deliver late success,
  server error and transport failure. Require one call, unchanged settlement
  classification, no automatic retry and no fresh-source access. Retain disabled
  raw-name denial/enabled sibling positives and all previously accepted RC data.

Freeze the complete source, composition, constructor/projection adaptations and
oracles; obtain fresh independent security/lifetime and structure/performance
reviews. Extend the existing MCP bundle's exact input members and named-positive
receipts, and run affected Rust/App and complete native contract targets only
in GitHub Actions on published H. Test compilation, source binding, actual
resource-boundary execution and real external compatibility remain distinct.
No source-only approval or synthetic transport proves real account acceptance.

This loop covers original-source custody for legacy HTTP execution. Official
and Box behavior remains preserved; their full final resource boundaries,
same-manager registration changes, Native credential/session epoch capture and
canonical Rust permission migration remain required follow-up responsibilities.
There is no waiver of OCRM G01–G12 or AC01–AC24. No database upgrade is needed;
rollback restores the stale-source risk and requires matching producer and
consumer versions. It cannot retract a previously entered external effect.


## Official routed provider admission — `MCP-OFFICIAL-DISPATCH-v1`

### Current production gap and existing ownership

This batch extends OD-01–07 after the native source/account/Attempt contract.
At source H `72b6ed8b6e91a0ac7e0588fd9492a7d30c535079`, the production
adapter passes its captured assertion to `DesktopMcpManagerFacade`, but the
Official branch drops it. `OfficialMcpService.execute` then awaits native
credentials, inventory and protocol preparation before provider `fetch`.
Retiring only the original control source during those awaits can leave the
account, manager and install current and permit an obsolete routed tool call.

The existing Coordinator source/account owner, MCP Attempt, native Official
service and encrypted vault remain the owners. The facade only forwards the
existing synchronous assertion. This is native resource admission at the
existing platform edge; it does not migrate or duplicate the canonical Rust
policy owner. No persisted schema, wire-origin field, dependency, Agent loop,
marketplace or permission registry is added. Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`.

### Required behavior and effect boundary

- **OF-01 / producer and wiring:** Official routed execution receives the exact
  `McpDispatchAssertion` already supplied by the production adapter. Never
  reacquire a current source/Attempt or attach authority to tool arguments.
  Preserve current unguarded UI listing/auth behavior.
- **OF-02 / final resource admission:** each provider send belonging to that
  execution, including initialize, notification, pagination and tools/call,
  prepares its URL, headers and serialized body before a synchronous check
  immediately adjacent to provider fetch entry. Reuse the service disposal
  check and the captured source/Attempt assertion, with no intervening await
  or arbitrary observer. Payload serialization must not run after that check.
  Assertion types continue to exclude Promise-returning callbacks.
- **OF-03 / request-local proof:** only that invocation's final assertion may
  record a known refusal witness, only for the existing
  `McpOriginRetiredError`, and only before any tools/call fetch has entered.
  Mark tools/call entry synchronously before invoking fetch. Return the existing
  generated rejected arm with fixed `MCP_ORIGIN_RETIRED` for this witnessed
  pre-call refusal. Arbitrary matching error text, typed errors from credential
  I/O, payload preparation or entered fetch are not known refusal.
- **OF-04 / precise effect claim:** rejected means zero tools/call fetch entries.
  Prior initialize, initialized notification and tools/list traffic can already
  exist; tests and evidence count these separately. Zero total provider I/O is
  claimed only if rejection precedes the first provider send. Broker refresh,
  vault writes and account/registration mutation are separate boundaries.
- **OF-05 / entered result custody:** after tools/call entry, source-only
  retirement preserves actual success or provider `isError: true` results
  while account/install/service remains valid. Transport, HTTP and JSON-RPC
  failure remains the existing redacted generic error/unknown outcome. Never
  retry, borrow a replacement source or classify a late failure as no effect.
- **OF-06 / FMCP-007 compatibility:** account change, uninstall, credential
  revision change or service disposal continues to suppress late private
  output through the existing account/install fences. Do not apply legacy
  HTTP late-visibility rules across those fences. Keep raw-name disable checks,
  fixed provider endpoint, provider-only token, exact arguments, protocol and
  session headers, output redaction, time/body/pagination limits and no retry.
  Closing a control port before reply delivery still leaves Host custody
  unknown; a direct native refusal is not proof of a delivered Host rejection.
- **OF-07 / isolation and recovery:** guards and witnesses belong to an
  invocation, never service-global current request state. Concurrent calls on
  one service cannot share refusal state. Reset retires the original Attempt;
  a stale call does not construct or borrow its replacement. A fresh source
  or Attempt may use the existing owner normally. Existing abort and cleanup
  own their original resources; no retry/replay or persistent migration.

### Implementation and causal acceptance

Use the existing `official-mcp-service.ts`, facade and actual default-production
fixture in `mcp-runtime.contract.test.ts`. The fixture constructs the real
adapter, runtime, facade, service, OAuth adapter and vault mapping; it may hold
or observe the native I/O ports and provider fetch without replacing those
owners. Extend its one-shot read hook only as needed for an awaitable gate.
Keep all existing MO, OD, MB and standalone Official contracts.

Required positive and negative oracles:

1. **OF-T1:** obtain a descriptor from the actual native source through the
   default production chain. Hold a vault/protocol completion, retire only
   that source and release it; zero tools/call and a witnessed native refusal.
   A fresh source on the same Attempt succeeds. Also reset an Attempt during
   preparation and prove no old provider handoff or replacement borrowing,
   with a fresh-Attempt positive. Earlier plain disposal errors may remain
   conservative generic failures.
2. **OF-T2:** retire in the last continuation after the final awaited fence
   fulfills but before fetch. Use deterministic promise-reaction ordering,
   not sleeps. A guard placed only before the await must fail this oracle.
   Include body-serialization retirement before the final handoff.
3. **OF-T3:** concurrent requests on one service retain separate assertions
   and witnesses; A retirement cannot reject current B. Distinguish the owned
   final typed refusal from same-message Error and typed errors thrown after
   actual fetch entry.
4. **OF-T4:** latch tools/call fetch entry, retire only source, then complete
   success, server error and transport failure. Require one call, unchanged
   result classification and no replay or replacement access.
5. **OF-T5:** after actual tools/call entry, account switch, uninstall and
   credential revision replacement suppress private late output. Keep
   disabled raw-tool refusal and enabled sibling, pinned URL, provider
   credential isolation, raw nested arguments, negotiated protocol/session
   headers and redaction controls.

Root owns facade/composition, the existing bundle/workflow, ledger and evidence
integration. Frozen source review must include these consumer connections.
Run the complete existing native plugin/MCP and account targets, the existing
standalone Official target, and new exact named positives in GitHub Actions on
the frozen published H. Native-only scoped recovery may reuse the existing
workflow and caches, provided its scope is explicit and it never credits
unexecuted Host/platform gates; existing unchanged-H or unchanged-input
positives keep their actual subject identity. No local executable validation.

This batch needs no database upgrade. Rollback restores the former admission
gap and cannot retract a provider call already entered. Preserve sanitized
fixed errors; do not log credentials or raw private results.

Real provider/account compatibility remains blocked by its actual credentials,
registration and entitlement prerequisites until tested. This batch does not
close Box's final resource boundary, same-manager install/registration atomic
revocation, broker final HTTP admission, native credential/session epochs,
canonical Rust policy migration, or any full OCRM G01–G12 / AC01–AC24 gate.
