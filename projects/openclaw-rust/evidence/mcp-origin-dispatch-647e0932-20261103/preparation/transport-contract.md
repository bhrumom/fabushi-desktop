# MCP-ORIGIN-DISPATCH-v1 — transport contract preparation

Assessment only, 2026-11-03. Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`. Exact base: `14faff5d9605c99df1494303f1aa0ce31bcfd420`. Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`; this report adds no upstream OpenClaw coverage. Configured model and effort were inherited; the requested classification is xhigh because the scope includes final-await lifetime, native authority, and effect custody. No product files were edited and no build, test, parser, lint, schema check, or generator was run.

Read authority: root `AGENTS.md`; OCRM `SOURCE_OF_TRUTH.md` authority/current-U/constraint and status sections; main specification §17; the applicable MCP retirement specification. The prior owner-resolution assessment was context only. Actual source reads and exact blobs are recorded in `mcp-origin-transport-readfacts.json`. The listed production sources/tests were unchanged against the exact base when inspected. The workflow had unrelated existing working-tree changes, so the selector cited below was re-read from the exact base with `git show`.

## Root freeze received at handoff

Root has now added `MCP-ORIGIN-DISPATCH-v1` to the existing specification and owns the shared origin schema/result types. Those new root edits were not independently reviewed in this assessment. Root's frozen names supersede the illustrative names below: `McpDispatchAssertion`, `McpOriginRetiredError`, `MCP_ORIGIN_RETIRED`, and `generatedMcpResultFactory.rejected`. The intended facade API is `executeRoutedTool(request, assertDispatchCurrent?: McpDispatchAssertion)`, passing the synchronous assertion directly rather than wrapping it in the illustrative context object.

Root also resolved the earlier-credential choice: ODT3 promises no effect for the existing early ordinary-error refusal without promising definitive rejection; ODT4 requires the typed witness at the final privacy/next boundary. For disjoint implementation ownership, put the direct real-client/interceptor cases recommended below in the new `source/shared/node/cursor-backend/backend-mcp-origin.contract.test.ts`; the native runtime worker owns additions to the existing runtime contract. Root will add the new target to the existing bundle/lane. This is a test-file placement decision; the required production subject, next counter, held awaits, and entered-late oracles remain the same. No implementation has been performed by this worker.

## Decision to freeze

Use the existing transport-option path, an optional **request-local synchronous callback**, and a **request-local typed refusal witness**. The callback runs immediately before the inference interceptor invokes its inner `next(request)`, after token, machine ID, privacy, header preparation, and the existing synchronous `onRequestId` callback. A guarded MCP execution gets its own client/transport closure. Listing and OAuth retain the existing shared client.

The backend helper records the witness only when its own final callback directly catches the native owner's typed retirement refusal. Its outer catch returns the generated `rejected` result only when that witness exists. It must not inspect message text, treat all callback failures as retirement, re-evaluate currentness in a catch, or look for retirement after an entered call settles. Ordinary transport failures keep the existing `error` arm and therefore upstream unknown custody.

This design is independent of Connect retaining the original exception. The pinned Connect version actually wraps interceptor errors and can replace them with a deadline error; walking `.cause` alone is insufficient. No generated protobuf or external request field needs changing.

## Exact callback path and ownership

| Existing source boundary | Required narrow contract |
| --- | --- |
| `adapters/mcp-oauth.ts:346–353` | Root's origin-aware routed adapter selects the exact existing Attempt and creates the native request context. It must not use a stale request to call `ensureMcpManager()` and adopt B. This table consumes root's source/Attempt schema; it does not define another one. |
| `mcp/desktop-mcp-manager.ts:48–55,260–272` | Add an optional second native argument to `executeRoutedTool(request, context?)`. Keep it separate from the serialized request and pass it to discovery as a fourth argument. The adapter's production routed path must supply it; omission remains compatibility for unrelated existing callers, not a fallback for unbound routed descriptors. |
| `shared/node/mcp/tools-discovery.ts:505–526,418–434` | Carry that same context through `executeTool(_ctx,args,auditIdentity,context?)` and `executeToolRaw(args,auditIdentity,context?)` to `core.backendMcpExec.executeTool(wireArgs, context)`. Do not repurpose `_ctx`: `SandMcpExecutor.execute` already forwards another execution context there at549–563. |
| `shared/node/cursor-backend/backend-mcp-exec.ts:22–29,37–60` | Widen the existing client-construction argument with the optional callback. Keep the current shared client for unguarded operations. For each guarded execute, allocate one local witness and create one client with that request's wrapped callback. Serialize the same Struct and request fields once. |
| `mcp/desktop-mcp-manager.ts:84–89,115–119` | Change the backend factory parameter from the account-client credential `Pick` to the backend helper's own widened client options. Explicitly forward the callback to `createSandCursorBackendClient`. The current two-property reconstruction would otherwise silently erase it. Do not change the separate account factory at77–82. |
| `shared/node/cursor-backend/cursor-inference.ts:120–158` | Add the optional callback to `SandInferenceOptions`; the existing `Omit<SandInferenceOptions,"backendUrl">` propagation already carries it through client and transport creation. Invoke it at the end of the interceptor, after149 and immediately before150. Nothing asynchronous or externally callable belongs between assertion and `next`. |
| `shared/node/mcp/mcp-result-factory.ts:1–54` | Reuse this existing shared result owner for the fixed native error and generated rejection constructor, subject to root's naming decision. Retain generated result construction and ordinary error behavior. |

The legacy disabled predicate still checks raw `args.toolName` at `tools-discovery.ts:401–415,422–425`. The remote method still receives public `args.name` as `toolName`, the original `providerIdentifier`, JSON arguments, tool-call ID, and Agent ID at426–432. Origin/callback metadata must not enter provider arguments or headers. The existing custom-instruction and image transformations return non-success results unchanged (`tools-discovery.ts:52–65`; `mcp-image-assets.ts:26–35`).

One small diagnostic consequence: `tools-discovery.ts:433` currently considers every non-`error` result a successful first call. A locally rejected origin must not produce `first_call_ok` telemetry. Suppress that notification for the known new local refusal; do not broaden this batch into a telemetry reclassification project. Production currently does not supply this optional discovery callback, but alternate consumers can.

## Concrete signatures and the witness

Names are recommendations for root to freeze, not implemented API. Put the shared native-only types with the existing MCP result owner rather than making cursor inference depend on Coordinator or MCP runtime internals:

```ts
export interface McpOriginDispatchContext {
  readonly assertCurrent: () => undefined;
}

export class McpOriginRetiredError extends Error {
  // No caller-supplied string, descriptor, account, token, or payload.
  constructor() { super("MCP_ORIGIN_RETIRED"); }
}

// Existing result factory gains this exact, bounded constructor:
originRetired(): McpResult;

// Existing backend helper interfaces:
export interface DashboardMcpClientOptions {
  readonly getAccessToken:
    DashboardMcpExecDependencies["getAccessToken"];
  readonly getMachineId:
    DashboardMcpExecDependencies["getMachineId"];
  readonly beforeDispatch?: () => undefined;
}
// DashboardMcpExecDependencies.createClient:
(options: DashboardMcpClientOptions) => DashboardMcpExecClient;
// Returned executeTool:
(args: ExistingBackendExecuteArgs,
 context?: McpOriginDispatchContext) => Promise<McpExecResult>;
// SandInferenceOptions adds:
readonly beforeDispatch?: () => undefined;
```

Using `() => undefined` rather than `() => void` makes an accidentally async callback incompatible with the type. TypeScript permits async functions where a void-returning callback is expected; ignoring that promise would break the boundary. The owner assertion itself is a synchronous comparison of the captured native source and exact Attempt, not a fresh manager/account lookup.

Required backend control flow, shown as design pseudocode:

```ts
let preDispatchRefusal: McpOriginRetiredError | undefined;
const beforeDispatch = context === undefined ? undefined : (): undefined => {
  try {
    context.assertCurrent();
  } catch (error) {
    if (error instanceof McpOriginRetiredError) preDispatchRefusal = error;
    throw error;
  }
  return undefined;
};

try {
  // Existing Struct normalization remains here.
  const executeClient = context === undefined ? sharedClient : deps.createClient({
    getAccessToken: deps.getAccessToken,
    getMachineId: deps.getMachineId,
    beforeDispatch,
  });
  const response = await executeClient.executeSandMcpTool(existingRequest, existingTimeout);
  return response.result ?? existingMissingResultError;
} catch (error) {
  if (preDispatchRefusal !== undefined) return generatedMcpResultFactory.originRetired();
  // Existing recordExecError / deadline / ordinary error handling.
}
```

Check the witness before optional error-report callbacks, so an unrelated diagnostic failure cannot erase the known refusal. There is one generated execute call per request and no retry. The callback is not installed on a mutable shared client, the privacy subrequest, account listing, OAuth, or the manager as a mutable “current request.” No post-result assertion is added.

The final interceptor tail is exactly `options.onRequestId?.(requestId); options.beforeDispatch?.(); return await next(request);`. A synchronous retirement induced by `onRequestId` is therefore caught. The declaration and call should not accept Promise/thenable return values. The current interceptor ordering places tracing outside inference (`cursor-inference.ts:156`; Connect `interceptor.ts:209–223`); tracing does not add an await between the final assertion and the inner network continuation.

### Earlier credential failure must be explicit in the spec

`mcp-runtime.ts:70–73,106–112` currently throws an ordinary Error on retirement at credential entry, after auth-service lookup, or after token lookup. A rejection there can prevent the final callback from running. The witness design above honestly leaves that result in the existing error/unknown classification; it cannot prove the typed final-callback refusal occurred.

If root's new contract promises definitive rejection for those earlier retirement exits too, the native owner must emit the same typed refusal there, and a request-local preparation wrapper must record that exact refusal while still demonstrably before `next`. Specify and test that extra provenance path. Do not achieve it by matching the old message, traversing arbitrary errors, or testing currentness in the general backend catch. The held-privacy and held-machine-after-token tests isolate the new final boundary regardless of this choice.

## Why Connect wrapping changes the design

The repository pins `@connectrpc/connect` and `connect-node` to1.6.1 and protobuf to1.10.1 (`desktop/package-lock.json:343–371`). Primary source was fetched read-only at Connect tag1.6.1, resolved to commit `f154e67629c4553a232c15c45f5f7bf5c7a1b02d`:

- `packages/connect/src/protocol/run-call.ts:48–68` invokes the interceptor stack and sends rejection to `abort`. At166–182, `abort` calls `ConnectError.from`; if its deadline signal is already aborted it substitutes the deadline reason.
- `packages/connect/src/connect-error.ts:111–130` wraps ordinary Error instances in ConnectError; preserving `cause` on the ordinary path does not help when the deadline branch substituted another reason.
- `packages/connect/src/promise-client.ts:95–113` awaits transport unary and returns its response message. It does not return an interceptor's original exception as a separate typed result.

Thus a plain outer `instanceof McpOriginRetiredError`, a string comparison, and recursive `.cause` inspection are each insufficient as the sole proof. The per-request witness records the owner's decision before the wrapper sees it. Ordinary same-text exceptions and server-supplied fields cannot write that witness.

Primary source link: [Connect1.6.1 run-call](https://github.com/connectrpc/connect-es/blob/f154e67629c4553a232c15c45f5f7bf5c7a1b02d/packages/connect/src/protocol/run-call.ts). Exact dependency blobs and read bounds are in the JSON. These are source/API contract reads; no local npm installation or integrity execution was performed.

## Result and custody contract

The existing generated `McpResult` already has the `rejected` arm (field3), and `McpRejected` has `reason` and `isReadonly` (`mcp_exec_pb.ts:106–137,410–451`). Construct:

```ts
new McpResult({ result: {
  case: "rejected",
  value: new McpRejected({ reason: "MCP_ORIGIN_RETIRED", isReadonly: false }),
} });
```

`isReadonly:false` makes no claim that the requested tool was read-only. No generated file changes are required. These files are recovered generated artifact contracts, with their provenance recorded in their headers; do not invent a new proto input or hand-edit them.

| Observed path | Required result |
| --- | --- |
| Exact owner refusal in final callback; `next` count0 | Generated rejected arm, fixed reason, no credential/origin/provider payload in diagnostics. |
| Arbitrary callback/preparation/serialization error without the typed witness | Existing error classification; do not invent no-effect authority from similar wording. |
| `next` entered; later source retirement; success | Original success value and original waiter; one call. |
| `next` entered; later retirement; server-declared `McpSuccess.isError:true` | Original success arm and server-error content, one call. This is a definitive server result. |
| `next` entered; later retirement; transport/deadline failure or missing result | Existing error arm, preserving unknown custody. No stale-origin override or replay. |
| Native reply/control port lost after publication | Existing upstream unknown disposition. Local proof cannot create a reply that was never received. |

Rust's actual `CoordinatorToolRelay::has_definitive_mcp_result` inspects only the result envelope and recognizes `rejected` at `coordinator_tool_relay.rs:308–369`. It intentionally does not treat `error` as definitive. Native control dispatch turns returned values into `status:"ok"` but thrown errors into `status:"failed"`, and drops replies after settlement (`coordinator-control-server.ts:77–109`). Return the generated refusal through that existing result path; do not throw it past the native executor as the final wire outcome.

## Smallest meaningful test additions

Use the existing complete `mcp-runtime.contract.test.ts` and `coordinator-control-server.contract.test.ts`, plus the already-selected Rust relay regression target. No new workflow or dependency is necessary.

### Runtime file: replace only the network edge of the new cases

Reuse `deferred` at122–130 and the real runtime/manager/settings/discovery fixture at944–1073. Its current `backend.executeTool` at983–989 is a stub, and MD entered-late tests1485–1538 settle that stub directly. Those cases retain value, but they are not the new interceptor oracle. Add an optional *backend construction* fixture port invoked inside the real `createManager(options)` so the new backend receives the actual Attempt-captured token getter. Keep real `SandMcpManager` and `createMcpToolsDiscovery`; do not replace either admission function.

The new backend uses `createDashboardSandBackendMcpExec` with its existing `createClient` dependency. That dependency constructs `createClient(DashboardService, transport)`, where `transport.unary` uses the real exported `runUnaryCall` from `@connectrpc/connect/protocol` and the real `createSandInferenceInterceptor`. Its controlled `next` is the network port: increment the counter on entry, snapshot the real generated message/headers, then return or await a deferred `ExecuteSandMcpToolResponse`. A stream call is an unexpected-fixture failure. Retain the passed signal/timeout and real generated input normalization. Do not emulate `ConnectError.from` or call a helper assertion in place of the interceptor.

This is the smallest direct test of the exact `next` and wrapping behavior. It does not by itself exercise the private `generatedBackendClient` in the default desktop factory. Root must also retain a composition test through the actual routed facade that proves this field is forwarded. The existing default-factory fixture at1756–1934 shows the established controlled-I/O/cleanup pattern; its auth deliberately rejects legacy traffic at1805–1806, so it cannot be reused unchanged as legacy HTTP evidence. A narrow legacy counterpart should permit the synthetic legacy credentials, hold privacy at Node HTTP I/O, and count execute HTTP requests while using the real default facade/factory. Do not claim a custom `createClient` fixture proves a private forwarding adapter it bypassed. Do not rely on monkeypatching bundled private functions: the current esbuild contract bundle inlines dependencies and only leaves Electron external (`build-electron-runtime.mjs:19–35`).

Freeze these new named oracles before execution:

| Case | Deciding observations |
| --- | --- |
| Current positive | Generated Dashboard execute request reaches actual interceptor `next` exactly once; public/raw names differ; nested Struct JSON, provider ID, tool-call ID and Agent ID match; no origin/callback data appears in message. |
| Held privacy → reset | Token and machine ID have completed; real `resolveGhostModeHeader` I/O is held; reset the actual runtime, optionally reserve B, then release. Require next0, generated fixed rejection, no B execute or adoption. |
| Held machine after token → source retirement | Hold the machine promise after the original token is returned; retire the native source and release. Require next0 and the same typed rejection. Include current same-input control. |
| Synchronous final-tail retirement | Use existing `onRequestId` hook to retire the source; require next0. This distinguishes assertion after149 from an earlier guard. |
| Two simultaneous requests | Hold A and B independently, retire A only, release in reverse order. A next0, B next1, separate witnesses, unchanged request identities. This detects a shared mutable callback/client mistake. |
| Ordinary same-text errors | Throw an ordinary Error with the fixed retirement text from final callback and, separately, from entered network next. Neither may become rejected without the directly caught typed witness. |
| Entered-late outcomes | Let next enter once and hold its response; reset/retire; release success, server `isError:true`, and uncertain transport failure as three cases. Preserve original value/classification and next1; no recheck or retry. |
| Connect wrapping | Observe that the actual runUnaryCall rejection is a ConnectError before it reaches the helper; the helper still produces rejected only for its direct witness. Add a controlled expired-deadline substitution case if the spec promises that collision behavior; use owned timer controls, not sleeps. |
| Compatibility | Keep the MD persisted raw-name denial and custom-instruction positives1275–1369, existing MC/MR/MB/MO tests, and shared-client listing/auth behavior. |

For privacy determinism, prefer the existing `resolveGhostModeHeader` injection over mutating the process-wide privacy cache in the direct next tests (`cursor-inference.ts:127,136–137`). The default-composition test must control actual privacy HTTP or explicitly clear/restore the cache and use a unique synthetic account scope. Avoid a warmed cache silently removing the intended await.

### Carrier and Rust custody

Extend the existing native-port/shipping-carrier producer map at `coordinator-control-server.contract.test.ts:161–173` with the **actual backend-produced** refusal from the final-callback test seam, then assert `status:"ok"` and the full `{result:{case:"rejected",value:{reason:"MCP_ORIGIN_RETIRED",isReadonly:false}}}` envelope alongside existing typed success/server-error/error controls at238–270. The existing fixture proves structured clone precedes JSON I/O; direct `JSON.stringify(McpResult)` alone uses canonical proto JSON and is not the native carrier oracle.

Existing Rust target `runner_coordinator_tool_relay_contract.rs:228–261` already checks rejected and server-error definitive values;264–301 checks malformed/error unknown values;322–367 checks timeout/control-close/remote failure and refusal of late replies. Preserve those full cases. A fixed-origin reason sample can be added to the definitive matrix, but a new parallel classifier is unnecessary.

### Execution selection, not execution evidence

The exact-base workflow already builds with `npm --prefix desktop run build:electron -- --plugin-skills-contracts` and selects both complete contract files, with the shipping carrier artifact, at `.github/workflows/rust-desktop-runtime.yml:4835–4853`. The build script includes them at91–110. Run the existing lane only in Actions on root's final integrated H, preserve complete TAP and command exits, and require every new named case with positive executed counts. The Root Rust suite should retain `cargo test --manifest-path source/host/Cargo.toml --test runner_coordinator_tool_relay_contract` in its existing selection. This report authorizes no local executable check and claims no pass.

## Handoff boundary

Root owns the durable spec, origin schema, Rust catalog/grouping/reconstruction propagation, native source and Attempt capture, composition, and integration. This assessment closes only the concrete transport callback/refusal design and its test seams. The earlier-credential classification is resolved by root's ODT3/ODT4 split above. Default-facade I/O proof remains a required composition test before claiming that concrete forwarding path covered. Official/Box final policy, Rust Settings/permission migration, remote revocation, and live provider acceptance remain outside this bounded legacy HTTP contract.
