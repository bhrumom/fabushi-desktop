# MCP-ORIGIN-DISPATCH-v1 — transport implementation handoff

## Status and exact subject

The assigned transport production changes and complete contract tests are authored and manually reviewed. **No build, parser, lint, generator, test or other executable validation was run.** GitHub Actions evidence is still required. There were no GitHub mutations, commits, dependency changes or changes outside the five allowed repository paths.

- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Authorized base: `14faff5d9605c99df1494303f1aa0ce31bcfd420`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`.
- Observed HEAD at handoff: `f41a6d3c00150ba59453d14887b5b4914801d4f4`; root advanced HEAD during this work. The five files below remain the assigned working-tree changes.
- Authority: the preimplementation `MCP-ORIGIN-DISPATCH-v1` section in `projects/openclaw-rust/docs/specs/mcp-account-listing-retirement.md`, OD-05 through OD-07 and OD-T3 through OD-T5; OCRM main spec §17 and its Actions-only execution rule.
- Existing assessment and read-facts files are preserved unchanged: `reviews/mcp-origin-transport-contract.md` and `reviews/mcp-origin-transport-readfacts.json`.

### Exact changed files

Paths below are relative to the repository root. Blob IDs are Git content hashes, not a claim that the working tree is committed or accepted.

| Path | Base blob | Final working blob | Lines | Diff from authorized base |
| --- | --- | --- | ---: | ---: |
| `source/shared/node/cursor-backend/backend-mcp-exec.ts` | `6e76db2678a4d832f1e13f12117e40b231529a62` | `db92e823e711204e87f54f8a54372df326a82622` | 108 | +37 / −5 |
| `source/shared/node/cursor-backend/cursor-inference.ts` | `296eeb02b7b0bc37256fc0d89b5c215d7cd1d501` | `8cc6b46db1f36c34c81ab3160e1abcb878c66091` | 194 | +2 / −0 |
| `source/shared/node/mcp/tools-discovery.ts` | `6e4b944417ede0d30617bedb889adf71fc7fa284` | `cb6a0dc007d8abddf0589ee56982cb13b9dfe9f0` | 579 | +8 / −4 |
| `source/electron-main/mcp/desktop-mcp-manager.ts` | `17af10771dfaea32dec8626f5a9e3ab4d5be6ad8` | `6b73527351ef9bb7a11e75c75dfc26f27a5002e1` | 286 | +7 / −3 |
| `source/shared/node/cursor-backend/backend-mcp-origin.contract.test.ts` | new file | `79bdbde6c51ce56506755abb559f3d5be6325056` | 525 | +525 / −0 |

Total assigned delta: five files, 579 inserted lines and 12 deleted lines. No existing contract-test file was edited.

## Implemented contract and ownership

1. `DesktopMcpManagerFacade.executeRoutedTool(request, assertDispatchCurrent?)` accepts the canonical `McpDispatchAssertion`. The facade forwards it as discovery's fourth argument. Its private `generatedBackendClient` explicitly forwards `credentials.beforeDispatch` into `createSandCursorBackendClient`. The account-client factory is unchanged. Relevant final spans: facade 47–57, backend factory 86–92, routed execution 263–276.
2. Discovery forwards the optional assertion into the backend helper's second argument. Existing raw `toolName` disabled checks and public `name` to remote `toolName` mapping remain intact. A generated `rejected` result does not consume either first-call diagnostic slot. Relevant final spans: discovery 418–440 and 508–530.
3. The backend retains the original shared client for unguarded execution, listing and OAuth. Each guarded execute creates a separate client with a request-local callback. No callback slot is mutable or global. The callback records a refusal witness only while catching `McpOriginRetiredError` thrown by its own supplied final assertion, then rethrows. The outer catch returns generated `rejected(MCP_ORIGIN_RETIRED)` only if that witness was set. It does not inspect messages, traverse causes, recheck currentness in catch, retry or reacquire a manager. Relevant final spans: backend 28–40 and 58–94.
4. `SandInferenceOptions.beforeDispatch` has type `() => undefined`. The inference interceptor invokes it synchronously after credential and machine-ID acquisition, the privacy await, all header work and the request-ID observer, immediately before `next(request)`. There is no intervening await. Relevant final spans: inference 119–153.
5. The local `McpExecResult` adapter now permits an unset `case` and optional `value`, matching the actual generated `McpResult` oneof and existing `McpResultLike`. This allows the generated rejection return without an unchecked cast and preserves an unset server result for the existing settlement owner. It adds no runtime classification or schema field. Relevant contracts: backend line 17; generated `agent/v1/mcp_exec_pb.ts` 106–132; `mcp-image-assets.ts` 6–11.

Once transport `next` is entered, this callback has completed successfully and cannot subsequently set the witness. Late server results therefore pass through unchanged, and thrown transport failures retain the existing execution-error/timeout treatment. Credential failures before the callback remain ordinary errors even if their exception happens to use the shared retirement type; they do not acquire a false proof of non-effect.

## New causal tests

All nine named tests are in `backend-mcp-origin.contract.test.ts`. These are authored oracles, **not passing-test evidence**.

| Named test suffix after `MCP origin:` | Boundary and decisive oracle |
| --- | --- |
| `current generated transport preserves public/raw names, nested args and disabled checks` | The real discovery rejects the disabled raw tool before any transport call. An enabled sibling on the same provider enters once. The generated request round-trip preserves public name, provider, nested JSON, call ID and agent ID; actual auth/privacy/request-ID headers are checked. |
| `held privacy retires before real Connect next and does not consume first-call diagnostics` | Credentials and machine ID are observed in the held privacy lookup. Dispose the actual control-server owner, release privacy, require zero `next` calls and the generated fixed rejection. Verify real `ConnectError` wrapping, zero execution-error diagnostics, no first-call report, then one successful fresh-source call and its first report. |
| `held token or machine ID cannot pass a retired source to next` | Separate held-token and held-machine schedules. Both credential operations start; retire the actual source before releasing the held result; require rejection and zero `next`. |
| `final assertion follows the request-ID observer and survives Connect deadline replacement` | The synchronous request-ID observer disposes the owner. Zero `next` and rejection are required. A second case uses `runUnaryCall`'s actual already-expired deadline (`timeoutMs: 0`); Connect substitutes its deadline error and removes the typed cause, while the request-local witness still yields the definitive rejection. |
| `concurrent guarded requests retain separate sources and reverse-order completion` | Hold two requests from two real source owners. Retire the first, complete the second first, then release the first. Require one exact current-source request, one rejection and different per-request callbacks. |
| `matching text and typed exceptions outside the final assertion are not known refusals` | A plain `Error(MCP_ORIGIN_RETIRED)` from the final assertion remains an ordinary error. A typed retirement thrown by credential I/O never reaches the assertion and also remains an ordinary error. Both have zero `next` without being falsely classified as the known refusal. |
| `entered late success, server failure and uncertain transport settlement retain custody` | Latch real `next` entry, retire the real source, then settle seven outcomes: success; success with `isError`; generated error arm; unset generated result; `Unavailable`; deadline error; typed retirement thrown by `next`. Server data survives generated serialization unchanged; thrown failures remain ordinary execution errors. Every schedule has exactly one `next` and no replacement client or retry. |
| `guarded refusal does not attach its assertion to shared listing, auth or unguarded execution` | After a guarded refusal, actual generated list, OAuth completion and unguarded execution all use the original shared client and succeed. |
| `default desktop facade forwards the assertion through its real HTTP client factory` | Instantiate the actual default desktop facade and real Connect Node transport against an owned loopback HTTP server. Warm its generated account/tool RPCs, hold the real privacy RPC, retire the actual source, release privacy, and require zero `ExecuteSandMcpTool` HTTP requests plus generated rejection. Then verify raw-name denial and a fresh-source exact successful request. This directly exercises the private `generatedBackendClient`; no custom backend factory stands in for it. |

### Fixture provenance and limits

The source fixture performs an actual control-server hello and request dispatch, captures the executor's second native context argument, and retires it using that server's `dispose()`. It neither creates a fake source UUID nor derives authority from wire fields. The native worker's second-argument implementation is now present and was spot-checked for constructor, dispatch and retirement compatibility.

The focused transport fixture uses the actual generated `DashboardService` client, `runUnaryCall`, and production inference interceptor. It substitutes only the final network effect, counts that entry before a held reply, and serializes generated request and response messages. The separate default-facade test uses an actual loopback HTTP server and therefore covers the private factory path too. Privacy lookup is a separate read RPC; its entry is not counted as execution admission.

The loopback fixture restores the prior backend environment, clears its privacy cache and diagnostic pin, disposes its facade and native source owners, closes only its own server/connections, and removes only its own temporary settings directory. Cleanup is registered as a test hook as well as a `finally` path so a timeout can release held privacy and close owned resources.

No real connector/account compatibility, native account replacement, MCP Attempt reset, Rust AutoReview, canonical projection or lost-control-reply custody is claimed by these tests. The native worker owns the existing `mcp-runtime.contract.test.ts` integration cases; root owns Rust and shared-schema composition.

## Shared-input and dependency spot-checks

At handoff the following root/native-owned working blobs were read but not edited:

| Input | Working blob |
| --- | --- |
| `source/shared/rpc/coordinator-port.ts` | `154b9c0f2e356288fc84bd9226a542fd492d20ee` |
| `source/shared/node/mcp/mcp-result-factory.ts` | `503e2960484f5fd52b6b449de9c95941764be877` |
| `source/electron-main/coordinator/coordinator-control-server.ts` | `64f335a78cbc0ca190f42caf225476eb1d2e30cc` |
| `projects/openclaw-rust/docs/specs/mcp-account-listing-retirement.md` | `1a5edb91bc4a943a1b7d73ffadcb52a1c9df0d59` |

Connect remains the existing locked `@connectrpc/connect` 1.6.1 dependency. Its exact source commit `f154e67629c4553a232c15c45f5f7bf5c7a1b02d` was read for the test instrument:

- [`protocol/run-call.ts`](https://github.com/connectrpc/connect-es/blob/f154e67629c4553a232c15c45f5f7bf5c7a1b02d/packages/connect/src/protocol/run-call.ts), blob `d3aecbdda40091191b48e964d6ad7a9e1b4599b8`, spans 28–120 and 160–187: normalization, actual interceptor invocation, exception wrapping and replacement by the internal deadline signal.
- [`protocol/signals.ts`](https://github.com/connectrpc/connect-es/blob/f154e67629c4553a232c15c45f5f7bf5c7a1b02d/packages/connect/src/protocol/signals.ts), blob `e67b3f73c4b5922e5243e55a7cf7c529d78f4bd0`, spans 15–101: the internal nonpositive deadline expires synchronously. An external abort is not a substitute for this oracle.
- [`interceptor.ts`](https://github.com/connectrpc/connect-es/blob/f154e67629c4553a232c15c45f5f7bf5c7a1b02d/packages/connect/src/interceptor.ts), blob `1e881fb798758bb450b4e16ec27c0b6850505590`, tail 215–223: reverse reduction places the production inference interceptor inside the tracing interceptor, immediately adjacent to transport `next`.

## Integration exit conditions

Root must include the new test path in the existing MCP bundle and existing Actions targets/receipts, freeze the integrated H with the source/account/Attempt and Rust custody work, and run the affected complete native and Rust contracts in Actions. No build/CI, ledger, lock, shared schema, generated file or existing test target was modified by this worker. The authored tests have not been compiled or executed, and the implementation is not yet verified or accepted.
