# MCP-ORIGIN-DISPATCH-v1 — R1 independent structure, reuse and performance review

Verdict: **APPROVE, SOURCE ONLY** for this review lens and the immutable 77-path scope below. I found no actionable P1/P2 structure, reuse, API completeness, lifetime-forwarding or performance defect in that delta. This verdict does not establish compilation, executed behavior, scoped G01/G02, real-provider compatibility, or OCRM acceptance. No product fixes are requested by this report.

## Review identity and independence

- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Round: R1; contract: `MCP-ORIGIN-DISPATCH-v1`.
- Accepted launch identity supplied by the assigning runner: **model `gpt-6.1-sol`, effort `xhigh`, `fork_turns=none`**. The assignment requires xhigh because it crosses native account/source/Attempt lifetimes, a shared Rust descriptor, AutoReview, Connect error wrapping, and existing CI receipt contracts. I did not infer an applied model or effort setting from prompt prose or start a separate worker to tune it; the accepted launch declaration is the available launch provenance.
- Frozen base and starting HEAD: `4e80e161c6cc539f4c120d64f91421619a57def1`; base tree independently read as `efb2a28c92d2f24cd4f6bbbd84d86709c0b2f028`.
- Reviewed tree: **`474e8c79e8f01dbd1ba0da32fdef08ee51312c49`**.
- Contract-specified implementation source base: `14faff5d9605c99df1494303f1aa0ce31bcfd420`; preimplementation/current spec blob: `1a5edb91bc4a943a1b7d73ffadcb52a1c9df0d59`, 47,065 bytes.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`. This is the declared fixed source identity, not a new upstream coverage or full-read claim by this review.
- I read the complete frozen delta, including all 77 changed paths, and used `git show`/`git grep` against the reviewed tree for production adjacency. I did not use mutable working files as product-source authority. Root `AGENTS.md` was read completely; its reviewed-tree blob is `e4682748087759b2d4bb2b3a47916b6148e8a6f2`. The tree's only other `AGENTS.md` is under `source/mahayana/codex-rs/tui/src/bottom_pane`, outside this scope.
- I read the complete current MCP specification, including inherited MR/MC/MB/MD/RC/MO contracts and OD-01–07/OD-T1–5; the applicable latest OCRM §17 execution/review amendment; and relevant source-of-truth/module-map ownership and completion distinctions. Historical source-of-truth material unrelated to this change was not promoted into current evidence.
- I did not read root arbitration decisions, engineering reports, previous review reports or peer reports before forming this verdict. The later runner notification about an outside-scope cloud checkpoint did not change the reviewed source subject. It supplied no MCP executable result.
- No repository source, test, configuration, ledger or manifest was edited. This report is the only file written by this review.

The supplied `complete.patch` is 245,016 bytes and 4,283 lines. Its SHA-256 is `6c8e25f105e0f73ec98873568f216a95414148c86213e5309bdca89b6c5d7ce7`. An independent read-only `git diff --binary --full-index` of the frozen base against the reviewed tree produced the same SHA-256 and exactly 77 paths. Supporting review-input hashes read here are:

| Input | SHA-256 |
| --- | --- |
| `reviews/mcp-origin-r1/manifest.json` | `5a51e4a9c5dd5750e6177ee8923368ae521bf071af090c4e3c39e1aa320ed3fd` |
| `reviews/mcp-origin-r1/changed-paths.txt` | `7653497a7f7e88a598b90389791d42150958e7bf36b5e2fcf0bb7800784f1131` |
| `reviews/mcp-origin-r1/required-cases.json` | `3cbfa80dba9ec2ea7adb0d15ada46a761f22397b29bd0ba6daf746881f0892dd` |

## Findings and architecture assessment

**No actionable priority finding.** The following source facts support approval; they are not claims of an executed pass.

1. **Existing owners remain the owners.** `coordinator-control-server.ts` creates one fresh source UUID per concrete server and derives retirement from its existing settlement latch. `breach()` and `dispose()` settle before diagnostics/post callbacks can reenter. The existing launcher creates a new server for every child launch, and process exit closes that same server. In `production-provider.ts:435–487`, the new per-runtime executors relate the native source to the existing account runtime's captured binding and exact `ownedRuntime`; a later account binding cannot authorize the earlier runtime. The source wrapper captures the original binding rather than reacquiring one during final assertion. This is native resource admission, not another permission store, registration owner, account policy engine or MCP runtime.

2. **Attempt reservation and lookup preserve the old construction owner.** `mcp-runtime.ts:171–248` extends the existing Attempt with UUID/assertion, retains reservation before deferred factory invocation, and keeps reset's synchronous detachment before cleanup. `requireMcpAttempt()` never calls capture/ensure and never constructs a replacement. `mcp-oauth.ts:353–409` captures the original Attempt before listing awaits, checks after manager and catalog awaits, stamps only object rows, and rechecks before publishing them. Execution checks source equality and the required original Attempt before waiting for its manager and again afterward. The facade call lies outside that pre-admission catch, so an entered late typed exception cannot be rewritten as a known refusal. The unchanged construction and teardown code still owns late manager disposal and propagation of cleanup failure.

3. **The internal DTO has a single shared owner and does not reinterpret tool arguments.** `coordinator-port.ts:5–36` owns the versioned origin and native context separately. Validation bounds each UUID to exactly 36 canonical lower-case hyphenated characters, requires version 1, rejects arrays/nonobjects, and returns only the three known origin members. Native arguments cannot mint the server's callback. The Coordinator executor and production-service interfaces consistently carry the native context as a separate parameter. Adjacent Rust Coordinator request parsing preserves the entire JSON `args` object; native structured cloning and the shipping carrier's JSON envelope preserve the origin field without converting it into native authority. No protobuf schema, dependency or persisted format is added.

4. **Rust custody is local, sealed at the public API, and carried with the exact descriptor.** `provider_session.rs:186–245` separates serializable wire origin from `RoutedMcpOriginBinding`, whose private fields include the existing relay incarnation. The binding has no serde implementation and redacts its Debug representation. Its crate-private constructor is used by the existing relay; external callers obtain a binding through `CoordinatorToolRelay::bind_tool_origin()`. `validate_tool_origin()` rejects a missing/foreign binding or closed relay before publication. The actual App's two decoders produce complete vectors or the existing typed/redacted `OriginInvalid` refusal; the actual `call_tool()` checks after the AutoReview callback returns and before activity/execution publication (`main.rs:8431–8479`). The existing request owner still independently checks closure and retains before-/after-publication uncertainty. Nothing reconstructs a local binding from a model payload or a serialized relay UUID.

5. **Projection reuse is complete at the changed reconstruction sites.** The canonical `execute_mcp_state()` retains first-seen provider grouping and within-provider order while moving `tool.origin` with each row. Both `TurnAgentComposition` snapshots and `TurnToolsetMcpProjectionBridge` move that same field when reconstructing descriptors; frozen paths clone their exact original vectors. The existing production turn-input projection consumes those vectors without a second discovery owner. The meta bridge retains its original source descriptors and clones the selected original descriptor for dispatch; it does not repair lost origin by a fresh catalog/name lookup. Existing public selection semantics remain unchanged. All changed built-in constructors explicitly use `None`; all changed integration fixtures are adapted. A repository-wide exact-tree text search for both shared struct constructors found the relevant constructors under Host, including the App and nested tests, with no additional constructor locations outside Host. This is source enumeration, not a substitute for Cargo compilation.

6. **Serialization boundaries deliberately drop execution custody.** `mcp_state_executor.rs:198–282` builds canonical protobuf tools from their existing public fields only and assigns `origin: None` on external decode. The generated MCP result already has the rejected arm; `mcp-result-factory.ts` constructs it without editing generated code. Adjacent Codex, OpenRouter, Cursor and MCP meta schema builders explicitly project public names/descriptions/schemas and omit the binding. The actual App's activity update includes provider/server identity, not the binding. The new canonical boundary oracle compares bound and metadata-free protobuf bytes and checks that decoding cannot regain local authority. The review did not find a newly added binding serialization path.

7. **The final assertion and refusal witness have request lifetime.** The callback type is `() => undefined` at the source context, facade/backend and interceptor seams, excluding a Promise-returning function in the declared TypeScript API. The actual default generated backend client forwards the option. `cursor-inference.ts:135–160` invokes it after credential/machine/privacy awaits, header preparation and `onRequestId`, immediately before `next(request)` with no intervening await. The existing tracing interceptor remains outside that inference interceptor. Guarded calls create their own client/transport; shared listing, OAuth and unguarded execution retain the pre-existing client. The backend helper's `preDispatchRefusal` closure belongs to one execute call and is set only if that call's final assertion throws `McpOriginRetiredError`. Catch does not inspect arbitrary message text, reacquire a source, or recheck currentness. Thus Connect wrapping/deadline replacement can preserve a proven pre-dispatch refusal while a typed failure from credential acquisition or from entered `next` remains in the existing ordinary/uncertain error path.

8. **Entered result custody and compatibility are preserved.** The helper returns actual server results unchanged, including an unset generated result, and applies its old transport-error handling when no final-assertion witness exists. There is no new retry or relist. HTTP discovery still checks the raw tool name against the persisted disabled map before backend dispatch, sends the public tool name to the existing backend API, retains nested arguments/call/Agent IDs, and suppresses first-call connector diagnostics for rejected calls. Custom-instruction application already returns nonsuccess arms unchanged. Official and Box final boundaries are explicitly deferred by the contract; their existing behavior is preserved rather than silently claimed as covered by the HTTP assertion.

9. **The added cost is bounded by existing resource lifetimes.** Source IDs are one per control server, Attempt IDs one per reserved Attempt, native row stamping is one pass over the returned catalog, and Rust storage/clone cost is per retained descriptor. Canonical grouping keeps its pre-existing search/order algorithm. A guarded execution allocates one client/transport and one closure/witness, as OD-06 expressly requires; it does not add a timer, loop, global request callback, registry, cache keyed by origins, or a second network attempt. Reusing the existing shared client for unguarded calls avoids charging listing/auth paths for this isolation. No benchmark, latency or socket-pooling claim is made: Connect dependency sources under `desktop/node_modules` were unavailable for inspection, and no runtime performance evidence was obtained. There is no source-demonstrated unbounded growth or extra execution request requiring a performance fix in this scope.

## Oracles, build integration and evidence limits

The complete new native transport file and all new account/control/Attempt cases were read, including their controlled-promise ordering, current positives, exact request/result assertions, source replacement, refusal-before-factory counters, reverse-order concurrent completion, Connect deadline wrapping and entered late settlements. The actual default-facade case uses its real generated HTTP factory against an owned loopback endpoint. The new Rust App case holds the actual bridge callback path across relay retirement, refuses old/unbound descriptors on the successor, and verifies fresh dispatch and drained pending counts. The projection cases include duplicate public rows carrying distinct original bindings. The canonical case distinguishes in-process custody from protobuf data. These are appropriate causal source-level oracles for this contract; they have not been executed here.

The existing `--plugin-skills-contracts` build list now includes `coordinator-executors.test.ts` and `backend-mcp-origin.contract.test.ts` alongside the already bundled control, runtime and account targets. The existing renderer job executes the complete control/executor/runtime/transport targets in its Plugin edge command and the complete account target in the MAB command. The Plugin receipt requires each of its 27 origin/executor names exactly once plus positive aggregate counts and zero failure/cancellation/skip/todo; MAB adds the three account cases to its strict exact-name set. The existing generated-result carrier case remains selected through the complete control-server target. It is a required existing regression in the review manifest; the workflow currently establishes it through full-target execution/aggregate TAP, rather than adding that particular old name to the new origin-name dictionary. Final evidence review should retain its actual named passing row as required by `required-cases.json`; this review does not represent aggregate TAP as an independently enforced exact-name receipt for that old case.

The existing OCRM Linux portable runner adds the complete `mcp_state_executor_contract` target and requires its new canonical name, one positive harness, no ignored cases, and count/name consistency. Existing preparation/projection/toolset/runtime cases remain selected. The new preflight `cargo check --locked --manifest-path source/host/Cargo.toml --tests` binds the full Host constructor/test graph and records original exit, raw log and digest, aborting before a passing receipt if compilation/capture fails. It is explicitly compilation evidence, not test execution. Host Cargo uses automatic integration-test discovery; the App is a separate package. The existing App step retains shipping feature-off `cargo check`, feature-observed test-harness compilation, unique actual App executable discovery, exact test listing and one-pass/zero-failed/zero-ignored execution. The four required App cases include the new review/relay case. These OCRM Rust selections are active for `workflow_dispatch` with `ocrm-config` and the configured migration-writer push branch; they are not a claim that every workflow/event selects the same batch.

The workflow still binds tested HEAD, lock identity, commands/logs and existing receipt/artifact owners. Source, build/test and canonical traceability changes reuse the existing workflow, bundle, portable runner, ledger and RTM. The ledger/RTM delta only adds narrow evidence references; both still show all eight global completion flags false. No unit verification or global acceptance is inferred from test authorship, bundle membership, an eventual compile, a historical pass, or this review. Known outside-scope App media fixture and background-cleanup source-oracle recovery are not reviewed or accepted here; the next campaign still requires their integration and its own frozen published H.

Only source text, Git object/diff/identity reads and hashes were used. **No parser, linter, formatter, generator, build, test, benchmark, harness, local/htch executable validation, or Actions campaign was run.** No MCP executable result exists for this review subject. Compilation and all required positive rows remain unmet until the authorized exact-H Actions evidence is read and adjudicated. Real accounts/providers, actual remote compatibility, package/platform acceptance, full Rust permission migration, official/Box final admission, same-manager registration and credential/session-epoch follow-ups remain outside this scoped approval. G01/G02 and the wider OCRM gates remain open; `unitVerified=false` and `globalAcceptance=false` remain correct.

## Complete frozen-path read record

Every path below had its complete change read. Behavioral changes and new tests received deep changed-function/call-chain/oracle review. Constructor-only changes received complete hunk/API contract review, with the global constructor search and the existing owning builders/dispatchers checked. The large workflow, ledger/RTM and existing test files were reviewed at their changed sections and necessary adjacent contracts; this is not a claim that their unrelated entire contents were deeply read. The complete MCP specification was read. The exact per-path blob/byte/hash identities remain in the hashed manifest above.

```text
.github/workflows/rust-desktop-runtime.yml
desktop/scripts/build-electron-runtime.mjs
projects/openclaw-rust/docs/specs/mcp-account-listing-retirement.md
projects/openclaw-rust/parity-ledger.json
projects/openclaw-rust/requirements-traceability-matrix.json
projects/openclaw-rust/scripts/capture_portable_contracts.py
source/electron-main/adapters/mcp-oauth.ts
source/electron-main/coordinator/coordinator-account-runtime.contract.test.ts
source/electron-main/coordinator/coordinator-control-server.contract.test.ts
source/electron-main/coordinator/coordinator-control-server.ts
source/electron-main/coordinator/coordinator-executors.test.ts
source/electron-main/coordinator/coordinator-executors.ts
source/electron-main/coordinator/coordinator-launcher.ts
source/electron-main/coordinator/production-provider.ts
source/electron-main/main-production-services.ts
source/electron-main/mcp/desktop-mcp-manager.ts
source/electron-main/mcp/mcp-runtime.contract.test.ts
source/electron-main/mcp/mcp-runtime.ts
source/host/app/src/main.rs
source/host/src/cloud_agents/cloud_agent_tool.rs
source/host/src/extensions/inference/provider_session.rs
source/host/src/extensions/mcp/production_box_state.rs
source/host/src/ports/mcp_state_executor.rs
source/host/src/runner/box_tool_access.rs
source/host/src/runner/coordinator_tool_relay.rs
source/host/src/runner/tools/box_help_tool.rs
source/host/src/runner/tools/mcp_meta_tools.rs
source/host/src/runner/tools/sand_agent_management_tools.rs
source/host/src/runner/tools/sand_await_shell_tool.rs
source/host/src/runner/tools/sand_browser_tools.rs
source/host/src/runner/tools/sand_computer_tool.rs
source/host/src/runner/tools/sand_external_machine_tools.rs
source/host/src/runner/tools/sand_file_transfer_tools.rs
source/host/src/runner/tools/sand_generate_image_tool.rs
source/host/src/runner/tools/sand_mcp_management_tools.rs
source/host/src/runner/tools/sand_multitask_todo_tool.rs
source/host/src/runner/tools/sand_reaction_tool.rs
source/host/src/runner/tools/sand_state_tool.rs
source/host/src/runner/tools/sand_subagent_management_tools.rs
source/host/src/runner/tools/sand_task_subagent_tool.rs
source/host/src/runner/tools/sand_web_tools.rs
source/host/src/runner/tools/send_message_tool.rs
source/host/src/runner/tools/turn_toolset.rs
source/host/src/runner/turn_agent_composition.rs
source/host/src/runner/turn_agent_composition/tool_call_recorder_tests.rs
source/host/tests/box_mcp_exec_contract.rs
source/host/tests/client_side_tool_v2_projection_contract.rs
source/host/tests/cursor_inference_transport_contract.rs
source/host/tests/external_machine_tool_contract.rs
source/host/tests/fabushi_responses_credential_context_contract/automatic_attempt_binding.rs
source/host/tests/inference_provider_session_contract.rs
source/host/tests/mcp_meta_tools_contract.rs
source/host/tests/mcp_state_executor_contract.rs
source/host/tests/openclaw_memory_contract.rs
source/host/tests/production_checkpoint_restart_contract.rs
source/host/tests/production_turn_input_projection_contract.rs
source/host/tests/runner_box_tool_composition_contract.rs
source/host/tests/runner_communicate_listener_contract.rs
source/host/tests/runner_effect_custody_contract.rs
source/host/tests/runner_production_bridge_contract.rs
source/host/tests/runner_routed_provider_contract.rs
source/host/tests/sand_action_audit_contract.rs
source/host/tests/sand_browser_tools_contract.rs
source/host/tests/sand_mcp_management_tools_contract.rs
source/host/tests/sand_reaction_tool_contract.rs
source/host/tests/sand_spotlight_tools_contract.rs
source/host/tests/sand_state_tool_contract.rs
source/host/tests/send_message_tool_contract.rs
source/host/tests/turn_observation_contract.rs
source/host/tests/turn_toolset_await_mcp_contract.rs
source/host/tests/turn_toolset_contract.rs
source/shared/node/cursor-backend/backend-mcp-exec.ts
source/shared/node/cursor-backend/backend-mcp-origin.contract.test.ts
source/shared/node/cursor-backend/cursor-inference.ts
source/shared/node/mcp/mcp-result-factory.ts
source/shared/node/mcp/tools-discovery.ts
source/shared/rpc/coordinator-port.ts
```

Additional relevant exact-tree adjacency read: the complete native account-runtime state machine and launcher/control/Attempt implementations; production-provider account setup/capture/disposal; native runtime launch/restart/exit paths; full default desktop facade construction/dispatch; HTTP/Box discovery and disabled/custom-instruction paths; the shared frame DTO and parser; shipping carrier envelope serialization/forwarding; the Rust Coordinator routed-request parser/forwarder; the complete Host relay; canonical state encoding/decoding; App decoders/review/execute/error boundaries; both composition snapshots and dynamic/frozen toolset projection; production turn-input projection; MCP meta descriptor selection/rendering; explicit Codex/OpenRouter/Cursor provider schema projections; generated MCP result/rejected definitions; Host/App Cargo manifests; desktop package/build declarations; OCRM workflow event/job/selection/receipt paths and portable runner command/result handling. These reads are bounded integration evidence, not a whole-repository or full-upstream review.
