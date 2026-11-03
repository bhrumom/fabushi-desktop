# MCP-ORIGIN-DISPATCH-v1 R1 — independent general/security/lifetime review

## Verdict

**Source-only approval of the complete frozen 77-path R1 scope. No actionable P0, P1, P2 or P3 finding established.**

The reviewed change consistently carries the original native control-server source, exact account-runtime lifetime and original MCP Attempt through the existing routed execution path. The Rust descriptor also retains its original local relay binding through review and projection. The final legacy HTTP assertion occurs synchronously immediately before the actual inner Connect `next(request)`. The request-local witness distinguishes that specific pre-dispatch refusal from unrelated failures and from work already admitted to transport.

This is an independent source verdict, not a compilation, executable-contract, external-provider, migration-acceptance or release verdict. No executable MCP result exists in the evidence supplied to this review. `unitVerified` remains false; global acceptance remains false; the eight global flags remain false. No change to permission authority or any additional runtime owner is approved by this report.

## Immutable subject and accepted launch

| Field | Reviewed identity |
| --- | --- |
| Repository | `/workspace/scratch/29e7135d8bfe/fabushi-desktop` |
| Contract / round | `MCP-ORIGIN-DISPATCH-v1` / R1 |
| Frozen base and starting HEAD | `4e80e161c6cc539f4c120d64f91421619a57def1` |
| Base tree | `efb2a28c92d2f24cd4f6bbbd84d86709c0b2f028` |
| Candidate tree | `474e8c79e8f01dbd1ba0da32fdef08ee51312c49` |
| Implementation contract base | `14faff5d9605c99df1494303f1aa0ce31bcfd420` |
| Pre-implementation specification blob | `1a5edb91bc4a943a1b7d73ffadcb52a1c9df0d59` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Complete patch | 245,016 bytes; 4,283 lines |
| Complete patch SHA-256 | `6c8e25f105e0f73ec98873568f216a95414148c86213e5309bdca89b6c5d7ce7` |
| Scope carrier | `/workspace/scratch/29e7135d8bfe/reviews/mcp-origin-r1/manifest.json` |
| Exact path list | `/workspace/scratch/29e7135d8bfe/reviews/mcp-origin-r1/changed-paths.txt` |
| Authored exact-name requirements | `/workspace/scratch/29e7135d8bfe/reviews/mcp-origin-r1/required-cases.json` |

The base/tree/spec identities and patch size/hash were checked with read-only Git/source identity operations. The complete patch was read, including all constructor/test adaptations, CI/capture additions and ledger/RTM reference changes. Repository source reads used `git show 474e8c79e8f01dbd1ba0da32fdef08ee51312c49:<path>` rather than mutable working-tree contents. The later root checkpoint reported during review does not replace this review subject or provide execution evidence.

The root reports that `collaboration.spawn_agent` was issued with explicit `model="gpt-6-astra"`, `reasoning_effort="xhigh"`, `fork_turns="none"`, and `task_name="mcp_origin_r1_security_review"`. The tool accepted that configuration and returned canonical agent `/root/mcp_origin_r1_security_review` without a fallback/error. No separate launch ID or independent applied-runtime metadata is exposed. These are accepted launch settings, not independently attested hidden runtime settings. The xhigh selection is proportionate to the cross-language account/Attempt/relay lifetimes, asynchronous credentials/privacy preparation and before/after-effect settlement distinction.

## Independence, authority and read facts

This review was fresh and blind. I did not read prior engineering handoffs, prior review reports, root decisions or peer reports. The applicable authority read was the frozen root `AGENTS.md`, the complete MCP account/listing/retirement specification including OD-01–OD-07 and OD-T1–OD-T5, the relevant authority sections of `projects/openclaw-rust/SOURCE_OF_TRUTH.md`, OCRM §17 including OCRM-EXEC-02, and the pertinent existing owner entries in `module-map.md`. This is not a claim to have read every historical paragraph of the entire source-of-truth or module-map documents.

All 77 changed paths were covered by the complete patch. Depth then followed the actual integration boundaries:

- Native source and account ownership: complete control-server/launcher flow; production-provider capture and executor composition; adjacent account-runtime binding, departure, pending launch and retirement flow; adjacent coordinator-runtime child restart/disposal flow.
- MCP Attempt and native bridge: complete MCP runtime, changed adapter listing/execute/dispose paths, default facade/backend factories, production services signatures and actual executor forwarding.
- Final HTTP and results: backend execution helper, inference interceptor and privacy preparation, adjacent RPC tracing, discovery HTTP selection and disabled-tool checks, generated rejection construction, all new backend-origin contract tests and their controlled resources.
- Rust custody and consumers: origin/binding definitions, relay binding/validation/publication/settlement, actual shipping App decoders and post-AutoReview call site, canonical in-process MCP state and protobuf encoder/decoder, live/frozen turn projections, dynamic toolset/meta-tool reconstruction, and adjacent provider-session/routed-provider consumers that select or serialize descriptors.
- Verification and scope support: actual new/changed contract-test bodies, all 49 built-in/test constructor-file adaptations, selected bundle declarations, affected existing Actions receipt/case selection, portable capture precheck/oracles, and all ledger/RTM diff hunks. The ledger/RTM delta adds evidence references rather than promoting acceptance flags.

Adjacent source reads included `coordinator-account-runtime.ts`, `coordinator-runtime.ts`, `rpc-tracing.ts`, and `routed_provider_runtime.rs`, as well as unchanged portions of the modified files. The review read changed hunks and relevant complete functions/consumers; it does not claim a full read of every unchanged line of the million-byte App file or the entire repository.

No build, test, lint, formatter, parser, generator, simulation, dependency installation or executable validation was run locally or through htch. No repository file or index was mutated. The only authored file is this report. External source retrieval described below was read-only.

## Security and lifetime assessment

### Native source, account and Attempt remain distinct existing lifetimes

`coordinator-control-server.ts` creates an opaque source UUID and an assertion over that server's own settlement state. Executor dispatch supplies this native context independently from the parsed request arguments. Disposal, protocol breach, shutdown and port closure reserve settlement before outward cleanup/report callbacks. Consequently a caller's JSON `origin` cannot create a live native context or choose which context executes the request. `coordinator-launcher.ts` creates a new server for each concrete child; same-account relaunch does not revive the departed child's source.

The production-provider capture checks the independently supplied source, captures the existing account runtime's current binding, and verifies exact `runtimeIdentity === ownedRuntime` before `context.requireMcp()`. Its closure retains that exact binding and never reacquires the replacement account's binding. The adjacent account runtime withdraws currentness synchronously before asynchronous transition/cleanup. The capture-only rejection catch ends before the awaited service execution, so an entered service failure cannot be reclassified merely because the account later retired. Source anchor: `source/electron-main/coordinator/production-provider.ts:432–486` and adjacent account-runtime binding/retirement methods.

The MCP runtime reserves an Attempt UUID together with its promise/currentness assertion before factory work can run. `captureMcpAttempt()` is the listing/construction path; `requireMcpAttempt(expectedId)` only selects the matching existing live Attempt. Its missing/mismatch/retired cases do not invoke construction, `ensureMcpManager()` or replacement selection. Reset detaches the current Attempt before disposal. The existing once-only disposal promise is reserved before calling manager cleanup.

Listing captures the source ID and Attempt before the manager await, rechecks after both manager acquisition and catalog acquisition, stamps origin onto object rows without sanitizing away the existing malformed-catalog contract, and checks again before publication. Execute validates the serialized origin against the independently supplied source, requires the existing matching Attempt, and rechecks the same captured objects after the manager await. The manager call is outside that preparation catch. Source anchor: `source/electron-main/adapters/mcp-oauth.ts:353–408` and `source/electron-main/mcp/mcp-runtime.ts` capture/require/reset/disposal functions.

These checks reject old-A execution after replacement and old-A origin replay on source B without first acquiring manager B. They are platform-resource admission checks under existing owners; they do not grant a tool permission or introduce a new account authority.

### Serialized origin does not manufacture a local relay binding

Both parsers require version 1 and exactly bounded, canonical lower-case hyphenated UUID strings. Extra fields are ignored rather than copied into authority. Native stamping overwrites any row-supplied origin. The Rust whole-catalog decoder rejects a missing/malformed origin through the existing fixed `OriginInvalid` catalog refusal, preserving the earlier RC public-field, duplicate, order and schema rules. The refusal contains row/kind/reason, not account credentials or the rejected descriptor contents. Source anchors: `source/shared/rpc/coordinator-port.ts:5–35`; `source/host/src/extensions/inference/provider_session.rs:186–245`; `source/host/app/src/main.rs:8491` onward.

`RoutedMcpOriginBinding` has private fields, no Serialize/Deserialize implementation, and an opaque Debug implementation. Its native DTO and private local relay incarnation have separate roles. The reviewed production construction site binds a received catalog to the actual relay; `validate_tool_origin()` refuses absent, foreign or closed-relay custody. Copying the wire DTO alone cannot deserialize a binding into a routed descriptor or import another relay's local incarnation.

The actual App bridge retains `&RoutedToolDefinition` across AutoReview and validates its original binding after review, before execution publication. It sends the native origin separately from tool arguments. It does not re-list tools, substitute a fresh origin or restore metadata from an ambiguous name match. The subsequent relay request retains its existing closure/registration checks. Source anchors: `source/host/app/src/main.rs:8434–8475`; `source/host/src/runner/coordinator_tool_relay.rs` binding/validation functions and `request_with_execution_outcome()`.

### Projection preserves custody without exporting it

The canonical in-process MCP grouping moves each row's binding with that row. The live projection preserves the existing first-seen server grouping and within-group order; the frozen projection retains the exact vector. Dynamic toolset/meta-tool reconstruction carries the whole original descriptor. Existing tool-name selection semantics remain unchanged; the change does not introduce a second lookup that tries to recover origin after discarding it.

The canonical protobuf encoder explicitly lists only its public schema fields; its decoder constructs `origin: None`. Built-in descriptors likewise receive `None`. That prevents external protobuf round trips from becoming a substitute for the internal Coordinator catalog. The actual Coordinator bridge rejects unbound descriptors before execution. Source anchors: `source/host/src/ports/mcp_state_executor.rs:132–165,195–280`; the modified projection functions in `turn_agent_composition.rs` and `runner/tools/turn_toolset.rs`.

The inspected provider schema/model serializers and meta-tool list response build explicit public-field objects. The App's transcript activity records provider/server identifiers and existing descriptive data, without adding origin/binding. The local binding has no general serialization implementation. These facts support absence of newly serialized custody metadata in the reviewed model, protobuf and transcript paths, rather than a claim that arbitrary caller arguments cannot themselves contain an unrelated field named `origin`.

### Final HTTP admission and request-local evidence

The assertion is forwarded through the default desktop facade, actual HTTP discovery branch and backend helper. Existing raw-tool disabled checks remain before backend execution, including the check after asynchronous provider selection. Public backend alias, raw name used for local disabled checks, nested arguments, call ID and Agent ID retain their existing roles.

The backend helper allocates a guarded execution client/transport only for a supplied assertion. Its shared listing/auth/unguarded client remains unmodified. The callback type is `() => undefined`, excluding a Promise-returning check from the typed contract. The actual inference interceptor finishes token/machine acquisition and privacy resolution, prepares request headers, invokes the request-ID observer, then invokes the assertion and immediately calls `next(request)` in that continuation. There is no intervening await or callback between assertion and `next`. Source anchors: `source/shared/node/cursor-backend/backend-mcp-exec.ts:58–91`; `source/shared/node/cursor-backend/cursor-inference.ts:120–160`.

This is final execution admission after all preparation awaits. It does not claim that previously entered credential/privacy operations are cancelled, that the separately shared auth/listing clients are guarded, or that the native credential/session epoch work explicitly left open by the specification is complete.

Only the request's own wrapper around that final assertion can set `preDispatchRefusal`, and only when that assertion throws `McpOriginRetiredError`. Connect may wrap or replace the surfaced exception; the local witness therefore records the fact needed for the fixed generated rejected result without examining exception text. A same-message exception, or a typed exception originating elsewhere, does not set that witness. Parallel requests capture separate closures/variables. No global current-request callback is added.

### Entered results keep their existing ownership

After transport entry, no source-currentness catch check or retry is introduced. Successful and server-declared failure results are returned unchanged; transport errors keep the existing error/uncertain classification. The native adapter and production-provider catches are deliberately bounded to pre-dispatch preparation rather than surrounding the awaited entered execution.

The Rust relay still treats lost replies, remote errors, timeout and closure after publication as unknown. Its definitive-result classifier interprets only recognized result envelopes, not a matching message or nested user data; the existing error arm remains uncertain. A native operation that independently refused before HTTP does not allow Rust to infer a known refusal when the reply was lost. Source anchors: `source/host/src/runner/coordinator_tool_relay.rs:138–152,215–234,336–397`.

No new rollback, automatic replay, scheduler, persistence format, session store or permission registry appears in the diff. Existing official/Box execution branches are preserved; their complete final-resource admission guarantees are explicitly outside this bounded HTTP contract.

## Requirement disposition and authored causal evidence

“Passed” in this table means source-review conformity only. All executable acceptance criteria remain **blocked pending exact-H GitHub Actions evidence**.

| Requirement | Source disposition and evidence |
| --- | --- |
| OD-01 native producer | Passed: independently supplied concrete-server context; exact account-runtime capture; retirement before callbacks; child relaunch creates a new source. |
| OD-02 original Attempt | Passed: reserved Attempt ID; nonconstructing require path; checks before/after manager await; reset withdraws currentness before cleanup. |
| OD-03 internal wire | Passed: bounded canonical origin validation and whole-catalog refusal; DTO outside tool args; RC compatibility retained. |
| OD-04 Rust custody | Passed: private relay binding, complete-descriptor propagation, built-in/external protobuf unbound, public serializers omit custody. |
| OD-05 review/dispatch | Passed: actual post-AutoReview validation plus original native account/source/Attempt checks and request-local forwarding. |
| OD-06 final HTTP | Passed: synchronous assertion after preparation and request-ID observer immediately before actual inner Connect next; guarded client is request-scoped. |
| OD-07 result custody | Passed: typed final-callback witness; no arbitrary-error classification, post-effect currentness reclassification or retry; lost replies remain unknown. |
| OD-T1 Rust/App/projection controls | Authored source is causally relevant: actual relay/EventHub and App review seam; malformed whole catalogs; replacement-relay refusal and fresh positive; actual canonical/live/frozen row identity. Execution blocked. |
| OD-T2 production account/source controls | Authored source exercises actual production Coordinator/account composition, serialized replay, zero manager-B work for stale requests, fresh-B positive and same-account child relaunch. Execution blocked. |
| OD-T3 held construction/list/credentials | Authored held factory/catalog and post-manager-await cases use the actual Attempt/native adapter; preserve existing official/Box/runtime target coverage. Execution blocked. |
| OD-T4 final admission | Authored tests use generated messages/client and real Connect call machinery with controlled final effect, plus an owned loopback test through the default desktop facade and real HTTP factory. They count final entry and inspect exact fields. Execution blocked. |
| OD-T5 entered settlement | Authored entry latches hold late success, server failure and uncertain failure; assert one entry, preserved result, no retry and no replacement access. Disabled/enabled and permissive catalog positives remain. Execution blocked. |

The review read the exact names in `required-cases.json` against the authored test and CI additions. New controls cover dispose/port-close/shutdown/protocol-breach retirement; held factory and catalog releases; source versus Attempt retirement after the manager await; arbitrary/typed errors outside the final assertion; request-ID observer retirement plus Connect deadline error replacement; concurrent guarded requests completing in reverse order; shared auth/listing isolation; and late native outcomes.

The new HTTP test uses owned loopback resources, an ephemeral port, controlled promises, explicit control-server disposal, privacy-cache/reporter/environment restoration, connection closure and temporary-store cleanup. The reviewed ordering controls use latches/observed entry rather than sleeps to decide which effect happened. That is a source assessment of the fixtures and cleanup, not a successful run or a proof that all failure paths have executed.

## Existing CI and receipt assessment

The affected existing workflow/bundle/capture owners were extended; no new campaign or workflow was launched by this reviewer.

- The plugin/Skills path builds the added backend-origin target and runs the selected complete bundled Node targets, including the existing control/executor/runtime targets. Its receipt checks exact subject HEAD, required named positive results exactly once, positive test/pass counts, and zero failed/cancelled/skipped/todo tests. The complete existing result-discriminant target remains selected; no claim is made that every pre-existing case has its own newly added receipt-name entry.
- The existing account/main-port bundle path requires the three new production account/source names, exact candidate identity, source content digests, fixed-U identity and successful build/test outcomes. Existing complete targets and shipping bundle checks remain selected.
- The actual App selection includes the four required received-catalog/review/origin cases. The Linux harness first proves an exact single name selection, then requires the exact case's one-passed/zero-failed/zero-ignored result; macOS selection retains the authored exact case names.
- Portable capture adds both real projection cases and the canonical serialization boundary case. It retains actual command status and raw logs, rejects missing/nonunique required positive names and nonpositive/inconsistent harness summaries, and makes the new canonical boundary require a single successful nonignored harness. The Host `cargo check --locked --manifest-path source/host/Cargo.toml --tests` is a compile precheck, not a substitute for those tests.

Source anchors are the changed workflow blocks around the actual App harness selection, plugin receipts and account receipts; `desktop/scripts/build-electron-runtime.mjs`; and `projects/openclaw-rust/scripts/capture_portable_contracts.py` target definitions and capture/result processing. The native build surface is esbuild plus selected bundled Node tests, not whole-graph strict TypeScript checking. No build/test success is inferred from inspecting these declarations.

## Primary-source dependency audit

The frozen desktop lock pins Connect and Connect Node to **1.6.1**. Read-only retrieval of the primary `connectrpc/connect-es` v1.6.1 source was used to inspect the final boundary and error behavior; the initial web retrieval route was unavailable, and the read-only GitHub connector supplied the source. No dependency code was executed or installed.

| Primary source at v1.6.1 | Retrieved blob | Relevant source fact |
| --- | --- | --- |
| [protocol/run-call.ts](https://github.com/connectrpc/connect-es/blob/v1.6.1/packages/connect/src/protocol/run-call.ts) | `d3aecbdda40091191b48e964d6ad7a9e1b4599b8` | Unary call error handling converts errors and may prefer the deadline reason; it does not preserve arbitrary exception identity as a custody guarantee. |
| [interceptor.ts](https://github.com/connectrpc/connect-es/blob/v1.6.1/packages/connect/src/interceptor.ts) | `1e881fb798758bb450b4e16ec27c0b6850505590` | Reverse reduction makes the configured tracing interceptor outer and the inference interceptor inner. |
| [protocol-connect/transport.ts](https://github.com/connectrpc/connect-es/blob/v1.6.1/packages/connect/src/protocol-connect/transport.ts) | `efefb53121c859de0b455e469964d497e3ade6a4` | The inner unary next owns serialization and HTTP dispatch; no automatic retry was found in the inspected call path. |
| [connect-node/connect-transport.ts](https://github.com/connectrpc/connect-es/blob/v1.6.1/packages/connect-node/src/connect-transport.ts) | `d2442a61d0389ad12f64a22b9da66c88651b649a` | The Node factory delegates to the Connect transport using normalized Node options. |
| [connect-node/node-transport-options.ts](https://github.com/connectrpc/connect-es/blob/v1.6.1/packages/connect-node/src/node-transport-options.ts) | `38aa16ea5d5c8d522b35afed9c9bf450f8b38a35` | The reviewed shipping configuration is HTTP/1.1, with no default send compression or new HTTP/2 session manager requirement. |

The adjacent repository tracing interceptor calls next once and rethrows failure; it adds no intervening retry below the final inference assertion. These source facts support the final-check and local-witness design. They do not attest installed npm bytes, compiled output or external network behavior.

For the fixed-U precedent, read-only GitHub retrieval inspected `src/mcp/tools-stdio-server.ts` at U (blob `ef5a603bce25ebe2247966c40932e8539ae80632`) and `src/mcp/plugin-tools-handlers.ts` at U (blob `a4c6ad2286313226007d2527ed5929494dd21aa2`). The relevant precedent is closing before drain and retaining already accepted work, alongside existing tool-map/alias behavior. This is a bounded source comparison, not new full-upstream module coverage or execution equivalence.

## Outstanding evidence and limits

There is no source fix requested by this review. The next acceptance step is the already required exact-published-H Actions campaign after the independently owned App fixture and background-wake oracle recovery is integrated. The reported unrelated fc App media fixture E0609 and background-wake oracle failure are outside this 77-path scope; this report neither diagnoses nor resolves them, and does not treat their recovery as an MCP passing result.

Compilation, selected full native targets, the named actual App/Rust cases and exact receipt identities must all be evaluated on the final integrated candidate. Any subsequent source change affecting this loop requires the appropriate focused review against this frozen scope. Successful controlled-transport tests would still be distinct from real external account/provider acceptance.

The specification's explicit follow-ups remain open: official and Box final resource boundaries, same-manager registration changes, native credential/session epoch capture, and canonical Rust permission migration. OCRM G01–G12 and AC01–AC24 are not waived. No persistence migration is introduced; rollback must preserve matching producer/consumer deployment and cannot retract an effect already admitted externally.
