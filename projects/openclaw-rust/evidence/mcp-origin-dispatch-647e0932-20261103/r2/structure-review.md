# MCP origin dispatch — fresh R2 structure, reuse and performance review

**Recommendation: request one test-oracle correction before accepting R2.** The focused R2 carrier expectation correctly adds the generated `isReadonly: false` field and retains the original strict assertion. A separate strict expectation in the unchanged account-runtime test helper still omits that field. It is used by two required OD-T2 cases and will reject the actual generated result after the native MessagePort clone.

No additional actionable production structure, reuse or performance defect was established within the reviewed contract. This is a source-only assessment; it does not establish compilation, passing tests, external compatibility or acceptance.

## Review identity and constraints

| Item | Identity |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Contract | `MCP-ORIGIN-DISPATCH-v1` |
| Frozen base | `4e80e161c6cc539f4c120d64f91421619a57def1` |
| Base tree | `efb2a28c92d2f24cd4f6bbbd84d86709c0b2f028` |
| Previous frozen R1 tree | `474e8c79e8f01dbd1ba0da32fdef08ee51312c49` |
| Reviewed frozen R2 tree | `73d5fac02a8410a2214dc0e4f337314e2a41b92c` |
| Published source fallback | `72b6ed8b6e91a0ac7e0588fd9492a7d30c535079` |
| Scope | Complete original 77-path change, plus focused R2 delta |
| R2 changed test blob | `19318f75ff5454a583801351ddbd6aa1a3ff067d` |
| Original account-test blob | `3605572af9d62734a6cc79d1f801ddf2a158eeab` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |

The following SHA-256 values were read directly from the frozen local evidence before the workspace connection failed:

| Frozen artifact | SHA-256 |
| --- | --- |
| `manifest.json` | `acdf3da3d88c24ddc17fc08ecf95844734952f5f7dca6feaf0d25c83215d92e9` |
| `complete.patch` | `f062218f89c5a9596a5d0ba6531dfcdd7efca67de9ee3a178e9aeb7b7148d1d3` |
| `focused-delta.patch` | `d3abc39bce63dd5688ab1e8ad8da3714c3578ff62b2f128ee583eb73bdac6649` |
| `changed-paths.txt` | `7653497a7f7e88a598b90389791d42150958e7bf36b5e2fcf0bb7800784f1131` |
| `required-cases.json` | `3cbfa80dba9ec2ea7adb0d15ada46a761f22397b29bd0ba6daf746881f0892dd` |

The complete R2 patch is 245,035 bytes and 4,283 lines; the focused delta is 987 bytes. The manifest is the complete path/blob/SHA-256 identity carrier for the 77 members.

I read the current root `AGENTS.md`, the applicable OD contract and related RC compatibility requirements. I did not read prior peer reviews or root arbitration records, including their published evidence copies. No subagent was launched by this reviewer. The active host model and effort were inherited; an in-place `gpt-6.1-sol` override is not exposed or independently observable in this worker session, so this report does not claim that override was applied.

No local build, test, lint, generator, schema check, benchmark or acceptance command was run. Source reads, Git-object reads and evidence hashing were the only local operations. The workspace became unavailable during review. Remaining reads used GitHub at the exact published fallback H, whose original 77-path source blobs are the R1 blobs; the separately read R2 delta supplies the single changed carrier expectation. No network artifact retry or Actions dispatch was performed.

## Finding SR-R2-01 — account-runtime rejection oracle omits a generated own property

**Severity: high / acceptance-blocking test flaw.**  
**Action: correct the strict expected payload in the account-runtime helper; retain all production behavior and other assertions.**

Affected source:

- `source/electron-main/coordinator/coordinator-account-runtime.contract.test.ts:906`, `assertWireOriginRetired`.
- Calls at lines 922 and 932 in `OD-T2 production Coordinator refuses departed account and serialized origin replay before manager B`.
- Call at line 962 in `OD-T2 same-account child restart retires its source while retaining the current MCP Attempt`.

The helper currently asserts:

```ts
assert.deepEqual(value, {
  result: {
    case: "rejected",
    value: { reason: MCP_ORIGIN_RETIRED },
  },
});
```

The actual result on this fixture’s path also contains the own property `isReadonly: false`.

The proof follows the concrete producer and consumer:

1. `source/electron-main/coordinator/production-provider.ts:474–480` catches a typed failure from the captured source/account binding and returns `generatedMcpResultFactory.rejected(MCP_ORIGIN_RETIRED)`. That is the stale-account path exercised by the first required OD-T2 case. It returns before resolving a replacement MCP service.
2. `source/shared/node/mcp/mcp-result-factory.ts:66–68` constructs a real generated `McpResult` containing `new McpRejected({ reason })`.
3. The unchanged generated constructor at `source/packages/proto/generated/agent/v1/mcp_exec_pb.ts:410–417`, blob `e1a0ccf6fdbd8d3d100aa806e45514e577a3b1ff`, assigns `this.reason = ""` and `this.isReadonly = false` before `initPartial`. Supplying only `reason` preserves the false default.
4. `source/electron-main/coordinator/coordinator-control-server.ts:100–116` retains the returned object as `outcome.value` and posts the outcome directly.
5. The account fixture’s `OriginNativePort.postMessage`, at `coordinator-account-runtime.contract.test.ts:684`, directly invokes the real native MessagePort’s `postMessage(value)`. Its receive path forwards the cloned frame without canonical protobuf JSON conversion or field filtering.
6. Thus the generated own property survives into the plain object compared at line 907. Canonical `.toJson()` default elision does not occur on this path.

The same helper also checks the replay refusal from the MCP adapter on the successor source. That adapter uses the same generated factory. The native account fixture’s success expectation already includes the corresponding generated defaults `content: []` and `isError: false`, which is consistent with its direct structured-clone transport.

The existing workflow makes this material to acceptance. `.github/workflows/rust-desktop-runtime.yml:5239–5244` adds all three account-origin case names to the MAB required positives, including both cases using this helper. The account-runtime contract target is part of that actual Node command. A missing default in a strict expectation prevents the required positive row even when the product correctly refuses stale execution.

The narrow correction is:

```ts
assert.deepEqual(value, {
  result: {
    case: "rejected",
    value: { reason: MCP_ORIGIN_RETIRED, isReadonly: false },
  },
});
```

Keep `assert.deepEqual` and the exact discriminant/reason. Retain the zero replacement-factory/service-resolution/execution assertions, the serialized-origin replay, the same-account restart check and the fresh successor positives. Do not alter generated constructors, normalize away defaults or weaken required positive-name checks to accommodate this mismatch.

This conclusion is directly supported by source and the platform API boundary visible in the fixture. It is **not** a claim that this reviewer executed the test or independently observed its failure in Actions.

## Focused R2 correction

The focused delta changes only the strict expected `origin-retired` payload in `coordinator-control-server.contract.test.ts`, adding `isReadonly: false`.

That correction agrees with the generated `McpRejected` constructor. The carrier test’s actual path deliberately removes generated prototypes through a native MessagePort structured clone before the shipping carrier performs JSON I/O. Consequently, the enumerable instance defaults remain in the carrier’s plain object representation. The test comments and neighboring generated-success expectations make that representation explicit.

Adding the field retains and sharpens the whole-object oracle: the test still requires the exact `rejected` discriminant, fixed retirement reason and complete expected value shape. It does not replace the strict comparison with a partial match and does not change production serialization.

The focused correction therefore has no separate actionable finding. Finding SR-R2-01 concerns the distinct, unchanged account-runtime helper in the complete frozen scope.

## Production structure, custody and reuse assessment

The implementation extends the existing owners and does not introduce another permission registry, account owner, MCP runtime or task loop.

The control server creates its own UUID source and provides its synchronous assertion as a second executor argument. Parsed request fields cannot supply that context. Its terminal paths latch settlement before reentrant diagnostics or shutdown posting. Launcher disposal, port closure and child exit lead back to that server’s retirement.

Production composition creates routed executors inside the owned account runtime’s construction. `captureMcpSource` compares the current binding’s `runtimeIdentity` to that exact runtime and retains the captured binding for subsequent assertions. The adjacent account owner synchronously retires its existing lifetime on an observed account departure. The captured assertion does not reacquire a replacement binding. Same-account child restart is distinguished by the fresh control-server source while the account identity and, where appropriate, MCP Attempt can remain current.

The MCP runtime adds an ID and assertion to its existing reserved Attempt. `captureMcpAttempt` reserves the object before deferred construction, preserving the existing single-construction/reentrancy mechanism. `requireMcpAttempt` checks the existing current object and expected ID; it does not call `capture` or `ensure`, so a stale execute cannot create a successor. Reset clears currentness before cleanup. The production adapter retains the same Attempt across manager and catalog awaits and checks source/Attempt currentness before publishing stamped rows. Malformed top-level and row values remain available to the Rust catalog decoder instead of being converted into an incidental native mapping failure.

On the Rust side, the wire origin and relay binding are distinct types. The relay incarnation stays private to the binding, and the binding is not serializable. Both App decoders validate the whole received catalog through the existing permanent catalog-refusal mechanism and bind each accepted descriptor to the current relay. The actual App bridge retains that descriptor through AutoReview and validates its binding after review. It emits the wire origin outside tool arguments.

Canonical in-process MCP grouping, both turn snapshots and dynamic toolset reconstruction move the existing per-row binding through the descriptor. They do not restore it through a new public-name lookup. The adjacent meta-tool bridge retains and clones its original source descriptor when selecting a target. The canonical protobuf encoder continues to enumerate only public fields; external decoding explicitly creates unbound rows. The provider request projections inspected also enumerate public tool fields and schemas rather than serializing the binding.

The shared result factory is reused for the generated refusal. The facade and discovery layer forward the optional assertion into the legacy HTTP execution path. The actual generated client factory forwards it to the Connect interceptor. Guarded execution creates a request-scoped execution client; shared listing, auth and unguarded execution retain their existing client.

The backend’s refusal witness is local to a single `executeTool` invocation. Only that request’s final assertion can set it, and only for `McpOriginRetiredError`. The catch path neither classifies by error text nor samples currentness after a possible effect. The interceptor invokes the synchronous assertion after credential/machine/privacy completion and the request-ID observer, immediately before entering `next(request)`. Its `() => undefined` callback type excludes ordinary Promise-returning callbacks.

The late-result boundary remains intact: native and production-composition catches cover their pre-admission capture phase, and do not reinterpret a later facade result. The backend preserves returned server results and existing uncertain transport-error handling after `next` entry. The new tests contain held-boundary, concurrent request, deadline-replacement, arbitrary same-message exception and late typed-exception controls that target those distinctions.

## Performance assessment

No actionable performance regression was established from source.

The new catalog work is bounded descriptor mapping and validation. Origin checks compare fixed-size UUID identities and current owner objects. The Attempt owner still retains one current attempt rather than an accumulating origin registry. The final witness and callback are request-local. No retry loop, periodic polling, extra discovery pass or name-based metadata reconstruction was added.

Request-scoped generated-client/transport construction adds per-call allocation for guarded HTTP execution. That construction is required by the declared isolation contract and prevents an assertion from contaminating concurrent or shared clients. The reviewed source supplies no evidence that its cost is material, and this review ran no benchmark. The pre-existing first-seen grouping implementation and its complexity were not changed by this patch.

## Complete scope coverage and limits

All 77 manifest members were identity-enumerated and dispositioned. All 4,283 lines of the complete original-scope patch and the complete focused R2 delta were read using bounded output. Truncated early output was covered again with bounded reads. Reading the complete change does **not** mean every line of every large enclosing file was deeply read.

| Scope group | Paths | Review depth |
| --- | ---: | --- |
| Workflow, build script, OD spec, parity ledger, RTM and portable capture script | 6 | Every changed hunk; actual affected Node commands, required-name/count oracles, source input binding and compile-gate additions |
| Native adapter, account/control tests, control server/executors/launcher, production composition, service interface, facade and MCP runtime | 12 | Every changed hunk; deep source read of lifecycle/admission owners, default client composition and concrete native test transport |
| Rust App, inference contract, canonical port, relay, turn toolset and turn composition | 6 | Every changed hunk; actual decode/review/dispatch, relay request/settlement, canonical encoding and live/frozen reconstruction |
| Rust built-in constructors, fixtures and contract tests | 47 | Every changed constructor/test hunk; deeper canonical-origin and projection oracles plus the adjacent meta-tool target-selection consumer |
| Shared RPC DTO, result factory, discovery, backend helper, interceptor and backend-origin contracts | 6 | Every changed hunk; deep source read of final assertion, witness, generated transport and controlled HTTP/late-outcome tests |
| **Total** | **77** | Complete declared change coverage |

Adjacent reads included the actual account-runtime lifetime implementation, generated MCP result constructors, native launcher/control reply path, custom-instruction handling for non-success results, provider schema projections and meta-tool descriptor selection.

The 47 Rust constructor/fixture paths are the exact manifest members outside the six semantic Rust owners listed above. The mechanical `origin: None` additions preserve the intended absence of Coordinator authority on built-ins and external fixtures. The richer new canonical/projection cases retain distinct origins on duplicate public rows and compare origin-free canonical bytes. The compile declarations cover the complete Host test graph and existing actual App harness selection, but their presence is not successful compilation evidence.

The ledger and RTM hunks add evidence/source references; they do not promote the global acceptance flags. The 37 independently incoming R6/evidence paths outside the original 77 were not included in this review.

The permitted original Actions logs were not consumed before the environment outage. This report therefore makes no independent claim about run `41587096196`, renderer job `125755041471`, actual test counts or the precise observed failure excerpt. No R2 executable evidence was produced or inspected. Required tests remain pending until their exact-subject Actions evidence is available and assessed against the retained named-case oracles.

## Terminal disposition

- **Focused R2 carrier fix:** source-approved.
- **Complete frozen scope:** one acceptance-blocking `TEST_FLAW`, SR-R2-01.
- **Additional production structure/reuse/performance findings:** none established.
- **Executable verification:** not performed by this reviewer; no passing claim.
- **Source-only recommendation:** correct the account-runtime helper, obtain focused re-review of that change and its generated/native consumers, and preserve the required Actions checks.

Official and Box final resource boundaries, same-manager registration changes, native credential/session epoch capture, canonical Rust permission migration, real accounts/services, packaged platform acceptance and OCRM G01–G12 / AC01–AC24 remain open. This origin contract does not close them.

**Persistence note:** the workspace was explicitly unavailable before the report could be written. This terminal text is returned for the root agent to persist as `mcp-origin-r2-structure-review.md`. Its final file-byte SHA-256 must be computed after that serialization; no report-file hash is fabricated here.