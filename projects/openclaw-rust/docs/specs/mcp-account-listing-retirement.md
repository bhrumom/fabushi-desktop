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
