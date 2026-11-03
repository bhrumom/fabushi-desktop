**R3 structure, reuse and performance review: source-only PASS. No actionable findings.**

The focused change correctly repairs the strict native-wire expectation without weakening its assertions. I found no new structure, reuse or performance defect in the reviewed fix or its affected producer/consumer chain.

The review subject is repository `bhrumom/fabushi-desktop`, commit `5d58596d2db779d9d1d3a94ac4d369f50c0108c6`, tree `08684cba1d291419254b4a25814108faf01b3091`. The frozen base remains commit `4e80e161c6cc539f4c120d64f91421619a57def1`, tree `efb2a28c92d2f24cd4f6bbbd84d86709c0b2f028`; the preceding round tree is `73d5fac02a8410a2214dc0e4f337314e2a41b92c`. Fixed upstream U remains `1440bb1771372334bea23d7642bb52deb7db1845`.

I read the complete 77-member manifest, the full frozen-scope patch, and the focused R3 delta supplied as Git blobs `71df0c5c09829fbf80a0463b33e1154ed4c34738`, `63d865ade78d70c150ab042156220167b97191c6`, and `e0f25d3e3ef10f0ac264607aa9ae51fbc0460252`. The manifest and patch each contain 77 distinct changed paths. Review authority was root `AGENTS.md`, the applicable `MCP-ORIGIN-DISPATCH-v1` specification, and OCRM §17. Lens assignment was `gpt-6.1-sol` / `xhigh`, appropriate to the cross-language identity, construction and lifetime boundaries.

All source access used GitHub reads at the immutable review commit. I performed no builds, tests, generators, Actions dispatches, local environment recovery, or source/branch/schema mutations. I did not read prior review reports, runner decisions or peer reports.

**Focused R3 assessment**

`assertWireOriginRetired` now expects `isReadonly: false` alongside the fixed rejection reason. This matches the generated `McpRejected` constructor, which assigns `reason` and `isReadonly` as instance properties and defaults `isReadonly` to `false`. The production result factory constructs that generated message. The account fixture crosses native `MessageChannel` ports, so its strict comparison observes the structured-cloned instance shape. The revised literal correctly describes that boundary. See [the corrected helper and its callers](https://github.com/bhrumom/fabushi-desktop/blob/5d58596d2db779d9d1d3a94ac4d369f50c0108c6/source/electron-main/coordinator/coordinator-account-runtime.contract.test.ts#L897) and [the generated constructor](https://github.com/bhrumom/fabushi-desktop/blob/5d58596d2db779d9d1d3a94ac4d369f50c0108c6/source/packages/proto/generated/agent/v1/mcp_exec_pb.ts#L410).

The three helper calls remain connected to meaningful production scenarios:

- Account A’s request during departure is refused before another MCP service lookup; factory count remains one and the execution list remains empty.
- Serialized A-origin replay on child B is refused before constructing manager B. A subsequent fresh B catalog and execution succeeds, with both source and Attempt identities changed.
- A same-account child restart refuses the old serialized source while retaining the current MCP Attempt. Factory count remains one, manager disposal count remains zero, and a fresh source executes through the retained manager.

The held production catalog case also remains intact: departure prevents A’s awaited catalog from publishing its origin, and B subsequently lists successfully. None of these controls became an existence check, self-derived expected value, loose comparison, or synthetic replacement for production enforcement. The separate control-server/carrier expectation includes the same generated default, keeping the two wire assertions consistent.

**Read coverage and integration assessment**

Every frozen changed-path delta was inspected. Depth followed OCRM §17: product and high-risk behavior received direct caller/consumer review; constructor-only adaptations received contract review; workflow, specification and ledger changes received evidenced disposition. This does not claim full-file deep reading of unrelated portions of the million-byte App source, large ledgers, or every adapted fixture.

The deeply reviewed chain includes concrete control-server source creation and retirement, launcher/runtime restart, account-runtime binding, production Coordinator capture, production MCP adapter, MCP Attempt capture/lookup, desktop facade, discovery, backend execution helper and the actual Connect interceptor. Adjacent unchanged account-runtime, Coordinator runtime and shipping Rust carrier source were read to assess the edited integration.

The existing owners remain coherent. The concrete control server creates its source context outside serialized arguments. Production capture associates that context with the exact account-runtime object. The MCP adapter captures the original Attempt for listing and requires that same Attempt for execution; absent or stale execution does not construct or await a successor. The request-local assertion passes through the existing facade, discovery and backend client factory. There is no added current-request global, second permission registry, parallel runtime or replacement-origin lookup. See [production source capture](https://github.com/bhrumom/fabushi-desktop/blob/5d58596d2db779d9d1d3a94ac4d369f50c0108c6/source/electron-main/coordinator/production-provider.ts#L428) and [Attempt ownership](https://github.com/bhrumom/fabushi-desktop/blob/5d58596d2db779d9d1d3a94ac4d369f50c0108c6/source/electron-main/mcp/mcp-runtime.ts#L171).

On the Rust side, review covered the shared descriptor/binding contract, relay binding and validation, both App list decoders, the post-AutoReview dispatch check, canonical in-process grouping, both composition snapshot paths and dynamic toolset reconstruction. Per-row custody moves with each descriptor, including duplicate public names. Canonical protobuf construction omits custody, and external protobuf decoding produces `origin: None`. The binding has no serialization implementation and its debug representation is redacted. Built-in constructors explicitly remain unbound. See [the relay validation boundary](https://github.com/bhrumom/fabushi-desktop/blob/5d58596d2db779d9d1d3a94ac4d369f50c0108c6/source/host/src/runner/coordinator_tool_relay.rs#L84) and [canonical projection and encoding](https://github.com/bhrumom/fabushi-desktop/blob/5d58596d2db779d9d1d3a94ac4d369f50c0108c6/source/host/src/ports/mcp_state_executor.rs#L98).

The remaining Rust constructor adaptations consistently add `origin: None`; they introduce no competing authority or behavioral rewrite. The canonical-boundary and projection controls retain public fields, order, duplicates, schemas and foreign/unbound refusal assertions.

The backend’s refusal witness remains request-local and is set only by the final supplied assertion observing the typed retirement error. Entered execution outcomes remain outside that pre-dispatch classification. Auth/listing clients retain their existing shared path, while guarded execution obtains its own assertion-bearing client. The authored controls retain concurrent request isolation, ordinary same-message exception negatives, disabled-tool/sibling positives, late success/server-error/transport outcomes and the default desktop facade’s actual loopback HTTP boundary.

**Reuse and performance disposition**

The R3 change itself has no production runtime cost. Within the frozen context, identity checks are constant-time comparisons using existing ownership state. Catalog stamping adds one pass over rows and per-row copies; custody propagation extends existing projections without a new lookup registry or catalog reconstruction. UUID generation uses existing platform/dependency facilities. I found no new unbounded cache, retained request registry, background polling loop, retry loop or material algorithmic regression attributable to these changes.

The guarded client allocation is deliberately request-scoped so concurrent requests cannot overwrite one another’s assertion. No measured latency, memory or throughput claim follows from this source assessment.

**Verification wiring and limits**

The existing Electron build script selects the complete runtime, control-server, account-runtime, executor and new backend-origin targets. The active workflow invokes the new backend/executor entries, supplies the shipping carrier artifact, and requires unique successful named rows for the origin controls. The account receipt separately requires all three production account cases and binds the relevant source/bundle identities. Rust/App selection includes the AutoReview/relay case; the portable runner adds the canonical custody boundary and distinguishes the full constructor compile gate from executed test evidence. Ledger/RTM edits add references without promoting acceptance flags. See [the active workflow selection and named-positive oracle](https://github.com/bhrumom/fabushi-desktop/blob/5d58596d2db779d9d1d3a94ac4d369f50c0108c6/.github/workflows/rust-desktop-runtime.yml#L4871).

**Findings: 0 blocking, 0 non-blocking actionable defects.** No fix is requested by this lens.

This verdict approves the reviewed source shape and focused oracle correction. It does not establish compilation or test passage at this commit, real provider/account compatibility, packaged acceptance, or global OCRM acceptance. Exact-H Actions evidence remains required. Official/Box final resource boundaries and native credential/session epoch responsibilities remain open under the specification.