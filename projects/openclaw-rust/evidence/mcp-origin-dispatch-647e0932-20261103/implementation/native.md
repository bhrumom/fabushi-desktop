Implemented the assigned native producer/Attempt scope and authored its causal tests. **No executable validation was run.** I manually read the integrated `production-provider.ts` binding and found it consistent with the new composition tests.

## Implementation

- Each concrete control server creates a UUID source context and passes it separately to executors. Disposal and protocol breach establish settlement before external callbacks can reenter the server.
- Routed executors forward the exact native source context.
- The existing MCP Attempt now exposes:
  ```ts
  interface McpRuntimeAttempt<TManager> {
    readonly id: string;
    readonly promise: Promise<TManager>;
    readonly assertCurrent: () => undefined;
  }
  ```
  `captureMcpAttempt()` reserves or shares the current Attempt for listing. `requireMcpAttempt(expectedAttemptId)` only checks the existing Attempt; it cannot construct or await a replacement.
- The production MCP adapter captures source and Attempt before listing awaits, rechecks them after manager and catalog completion, and stamps accepted object rows with the original origin. Non-array catalogs and malformed rows remain available to the existing Rust catalog refusal path.
- Execution validates the serialized origin against the independently supplied source and exact Attempt. Its typed-refusal catch ends before facade invocation. The composed synchronous assertion reaches the facade’s second argument, and entered execution outcomes remain unchanged.
- Existing credential `requireCurrent` errors and MR/MC/MB/MO behavior remain conservative. No new runtime, registry, permission owner, or persistence format was added.

`coordinator-runtime.ts` required no change: its dependencies inherit the updated executor type through `LaunchCoordinatorDependencies`, and its existing relaunch path creates a new concrete control server.

## Exact diff report

Assigned base: `14faff5d9605c99df1494303f1aa0ce31bcfd420`  
Current checkout HEAD at report: `9059d0c4493505851339ab228ec0d58a6023c904`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`

Nine files changed, **880 additions / 54 deletions** against the assigned base. The executor test file was added to my allowed scope by your explicit follow-up.

| File | + / − | Final Git blob |
|---|---:|---|
| `source/electron-main/adapters/mcp-oauth.ts` | 60 / 5 | `1684a9cfafd02f6cdd96091b7b85d9efb3f3dbcc` |
| `source/electron-main/coordinator/coordinator-account-runtime.contract.test.ts` | 347 / 1 | `3605572af9d62734a6cc79d1f801ddf2a158eeab` |
| `source/electron-main/coordinator/coordinator-control-server.contract.test.ts` | 75 / 1 | `b2c987bd59817ce00cacf3555ee64c8a804f2a42` |
| `source/electron-main/coordinator/coordinator-control-server.ts` | 18 / 8 | `64f335a78cbc0ca190f42caf225476eb1d2e30cc` |
| `source/electron-main/coordinator/coordinator-executors.test.ts` | 34 / 11 | `ce64821253c942771e08694b55edf273fb34fa8d` |
| `source/electron-main/coordinator/coordinator-executors.ts` | 7 / 6 | `206624baf04310d6b77e10ea28ae3c4b156a7e1f` |
| `source/electron-main/coordinator/coordinator-launcher.ts` | 2 / 1 | `50c1c7ff634839fc02266efdf742c5db39252b8d` |
| `source/electron-main/mcp/mcp-runtime.contract.test.ts` | 295 / 15 | `7e523aef17d0d47f4291b28cc38984d11c52c066` |
| `source/electron-main/mcp/mcp-runtime.ts` | 42 / 6 | `b1d651721db46b8752fc9c1ef95b689082e6c94a` |

I did not edit shared contracts, facade/discovery/backend files, production composition, specs, workflows, ledgers, locks, or Git state.

## Exact new causal oracle names

**Twenty new named cases** are authored.

### `mcp-runtime.contract.test.ts` — 13 cases

```text
OD-02 captured Attempt is reserved once and stale lookup never constructs a replacement
OD-02/03 held factory cannot publish origin after source retirement
OD-02/03 held factory cannot publish origin after attempt retirement
OD-02/03 held catalog cannot publish origin after source retirement
OD-02/03 held catalog cannot publish origin after attempt retirement
OD-02/05 stale and malformed origins cannot construct or borrow manager B
OD-02 execute rechecks original source after the manager await
OD-02 execute rechecks original attempt after the manager await
OD-03 native origin stamping preserves malformed received catalogs and permissive row values
OD-05/07 native adapter preserves admitted late success after source and Attempt retirement
OD-05/07 native adapter preserves admitted late server-error after source and Attempt retirement
OD-05/07 native adapter preserves admitted late failure after source and Attempt retirement
OD-05/07 native adapter preserves admitted late typed-failure after source and Attempt retirement
```

The stale-origin case covers both zero B construction and refusal while B’s factory is held unresolved. The two manager-await cases retire synchronously after execution reaches its await of the already-resolved original manager promise, independently qualifying the post-await check.

### `coordinator-control-server.contract.test.ts` — 4 cases

```text
OD-01 concrete control source retires before callbacks on dispose
OD-01 concrete control source retires before callbacks on port-close
OD-01 concrete control source retires before callbacks on shutdown
OD-01 concrete control source retires before callbacks on protocol-breach
```

Callback observations are recorded and asserted outside the server’s diagnostic catches, so those catches cannot hide a failing oracle.

The existing test **`MCP results retain their discriminants through native ports and the shipping Rust carrier`** also now includes the `origin-retired` producer row and exact generated `rejected` envelope assertion.

### `coordinator-account-runtime.contract.test.ts` — 3 cases

```text
OD-T2 production Coordinator refuses departed account and serialized origin replay before manager B
OD-T2 same-account child restart retires its source while retaining the current MCP Attempt
OD-T2/3 held production account catalog cannot publish its origin after departure
```

These use the real production Coordinator adapter, account runtime, launcher, control server, executors, and MCP adapter. Native `MessageChannel`s retain actual structured cloning. They test refusal before `requireMcp()`, serialized replay on B with zero B construction, fresh B execution, and child restart within the same account lifetime.

Existing executor tests now assert exact source-context pass-through. Existing MC-06 test names remain unchanged; their formerly unbound execution now expects the generated origin refusal, while the pending-list construction and zero old-facade I/O assertions remain.

## Verification boundaries and remaining work

- **Authored and manually inspected, not executed.** No local or `htch-runtime` parser, generator, lint, build, test, or acceptance command ran.
- The production composition tests control the child ABI and manager resource ports. Their execution counters are **facade/resource-port counters**, not actual HTTP `next` counters. The transport worker’s generated-client/interceptor tests must supply OD-T4/T5 transport evidence.
- The carrier test still requires the shipping carrier artifact built for the final exact H.
- The complete native targets, the 20 new named positives, existing regression cases, and generated rejected carrier row require final-H Actions evidence.
- Fresh independent security/lifetime and structure/performance reviews remain with root integration.
- No upstream read coverage, real-account/provider acceptance, or OCRM completion flag is promoted.

## Launch metadata

- Canonical worker task: `/root/mcp_origin_native`; parent: `/root`.
- Assigned phase: native implementation and causal-test authoring for `MCP-ORIGIN-DISPATCH-v1`, including lifetime, asynchronous-boundary and authority safety.
- Requested reasoning effort: `xhigh`, stated in the original assignment and retained for this persistence follow-up.
- Applied reasoning effort: `xhigh` according to the accepted root launch context, confirmed by root in the persistence follow-up. Launch used a fresh fork with `fork_turns="none"`.
- Model application: inherited configured model, with no worker model override. This worker did not start another AI process or change session configuration.
- Worker-visible metadata limitation: the available agent-list result exposes task names and statuses only. It does not expose an independently verifiable applied model identifier or reasoning-effort label. Prompt wording is not represented as proof of a host-side override.
- Execution scope: repository source reading/editing and read-only Git identity/diff inspection only; no executable verification or GitHub mutation.
- Persistence follow-up: this report is outside the repository; no repository file was changed to save it.
