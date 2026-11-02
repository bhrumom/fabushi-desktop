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
