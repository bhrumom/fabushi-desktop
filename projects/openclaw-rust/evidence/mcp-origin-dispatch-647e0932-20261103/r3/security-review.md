# MCP-ORIGIN-DISPATCH-v1 — fresh blind R3 general/security/lifetime review

Terminal status: COMPLETE.
Recommendation: APPROVE_FOCUSED_SOURCE_ONLY.
Falsifiable findings: none. No blocking or non-blocking defect is raised by this review.
Execution status: NOT RUN. This is a source assessment, not a build, test, runtime, packaged, or external-service acceptance result.

## Exact subject and independence

Repository: bhrumom/fabushi-desktop.
Review commit: 5d58596d2db779d9d1d3a94ac4d369f50c0108c6.
Review tree: 08684cba1d291419254b4a25814108faf01b3091.
Original base and starting HEAD: 4e80e161c6cc539f4c120d64f91421619a57def1.
Original base tree: efb2a28c92d2f24cd4f6bbbd84d86709c0b2f028.
Prior-round tree used only for source comparison: 73d5fac02a8410a2214dc0e4f337314e2a41b92c.
Fixed upstream U: 1440bb1771372334bea23d7642bb52deb7db1845; upstream tree 90c8aab39972da11cfad220361781d49e1bc6700.
Manifest blob: 71df0c5c09829fbf80a0463b33e1154ed4c34738.
Complete original-scope patch blob: 63d865ade78d70c150ab042156220167b97191c6; declared 245054 bytes, SHA256 4d166bfb9e23bec49d24fdd911ea7c67a767d8c70195e5168a9408b2870c3b54.
Focused R3 patch blob: e0f25d3e3ef10f0ac264607aa9ae51fbc0460252; declared 877 bytes, SHA256 6f5c0ca3a92836d34ed7067194dac3582e125b44846c0266b805b55a50be7a00.

I fetched the immutable commit metadata, all three relevant recursive trees, the complete manifest, and both patch blobs through the GitHub connector. The commit names the stated review tree and the original base as its parent. All tree responses were untruncated. Comparing blob identities independently gives exactly 77 changed paths against the original base, equal to the manifest's current members set. Every current member's blob matches the review tree. The R2-to-R3 tree comparison changes exactly one path, coordinator-account-runtime.contract.test.ts, from 3605572af9d62734a6cc79d1f801ddf2a158eeab to 517cd42f6af6d6114790d4144cc7ba9156f4e308. The manifest's historical paths array was not mistaken for its current members array. Patch byte counts and SHA256 values above are declared supplied identities; this reviewer did not run a local hashing or validation command.

The complete 77-path patch remained available throughout the focused review. I read no prior review report, root arbitration/decision, or peer report. The prior tree was used solely as immutable source context. No branch/ref/commit/source mutation or workflow dispatch was performed. The only authorized write is the single UTF-8 report blob containing this report.

Requested review effort is xhigh because the assignment includes account and source lifetime, asynchronous admission, structured-clone/protobuf representation, and Rust/TypeScript effect custody. The reviewer retained the configured parent session/model. No independent effort override was applied or claimed; the task's requested level is not represented as proof of a separately exposed runtime setting.

## Governing instructions and read facts

Read AGENTS.md at blob e4682748087759b2d4bb2b3a47916b6148e8a6f2, including its independent/focused review, immutable-snapshot, evidence, and execution-environment requirements. The complete current tree contains no additional nested AGENTS.md governing any of the 77 paths.

Read the applicable MCP contract file at blob 1a5edb91bc4a943a1b7d73ffadcb52a1c9df0d59, including MCP-ORIGIN-DISPATCH-v1 OD-01 through OD-07 and OD-T1 through OD-T5, and its retained lifecycle/received-catalog boundaries. Read OCRM-001 §17/OCRM-EXEC-02 at docs/specs/openclaw-rust-equivalence-migration.md, blob 69c9667af2328e20a1fba09ebf80c5aab4d5c336. Consulted the authority/owner portions of SOURCE_OF_TRUTH.md, blob 3748d734b10c9ed5b2d948467fbaa9483f1dd306, and module-map.md, blob 15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3. These supporting reads do not promote their historical source or acceptance states.

The depth was proportional to R3 under §17. The focused helper, its consumers, generated producer, actual carrier, and account/source/Attempt ownership chain were inspected semantically. The full original diff supplied context for Rust origin validation, relay binding, projections, final HTTP admission, result custody, all constructor adaptations, tests, build selection and metadata references. Mechanical origin: None additions were inspected as constructor/interface adaptations, not misreported as full semantic rereads of unrelated tools. Documentation, build and evidence changes were dispositioned by their actual consumers. The appendix enumerates all 77 current identities and the review category used.

Load-bearing source reads, all at the review commit/tree:

- coordinator-account-runtime.contract.test.ts: imports/test bounds at lines 1–44 and the complete origin fixture and three cases at 678–1002, especially strict helper 906–908 and calls 922, 932 and 962.
- coordinator-control-server.ts: full source, including independently created source UUID, settlement-before-callback ordering, native dispatch and reply suppression after retirement.
- coordinator-control-server.contract.test.ts: source-context controls at 47–109 and complete native-port/shipping-carrier fixture at 112–356; retained strict rejected payload at 315–317.
- production-provider.ts: changed native/account binding composition and adjacent runtime construction at 428–601.
- coordinator-account-runtime.ts, unchanged adjacent owner, blob 84e0e0ee6f83c0932a66ec4f89167b32e3f0e4ef: account observation/capture/restart/disposal at 484–575, with lifetime locations traced in the file.
- mcp-runtime.ts: full source, including exact Attempt reservation, identity checks, post-construction publication, failure cleanup, reset and disposal.
- mcp-oauth.ts, coordinator-executors.ts, coordinator-launcher.ts, main-production-services.ts and desktop-mcp-manager.ts: changed source/context signatures and their producer/forwarding/consumer hunks.
- mcp-runtime.contract.test.ts and backend-mcp-origin.contract.test.ts: complete origin-related changed hunks, controlled held-await cases, refusal classification controls, concurrent isolation, real generated client/interceptor and owned loopback HTTP fixture.
- mcp-result-factory.ts: full source. coordinator-port.ts: internal source/origin contract, parser and surrounding request carrier definitions.
- source/packages/proto/generated/agent/v1/mcp_exec_pb.ts, unchanged blob e1a0ccf6fdbd8d3d100aa806e45514e577a3b1ff: McpRejected runtime constructor and field schema at 410–451.
- rust-coordinator-carrier.ts, unchanged blob 375a706177e9bfa7aa7c9bf591d6688887af12fb: full source, especially writeEnvelope at 64–70 and transferred-port handling at 131–168.
- backend-mcp-exec.ts and cursor-inference.ts: changed request-local client/witness logic and interceptor admission, including cursor-inference.ts 133–160. tools-discovery.ts: actual raw-name disable check, post-preparation HTTP forwarding and result postprocessing at 401–458 and 508–530.
- source/host/app/src/main.rs: changed real bridge/AutoReview/decoder functions and the added or adapted origin controls. provider_session.rs: origin DTO, canonical UUID validation, private relay binding and descriptor extension.
- coordinator_tool_relay.rs: origin binding/validation, real request publication/settlement, closure and definitive result classification; mcp_state_executor.rs: in-process grouping, canonical encoding and external decode. The two turn composition projections and dynamic toolset reconstruction were inspected in their changed hunks.
- .github/workflows/rust-desktop-runtime.yml: all original-scope changed hunks plus build/test setup at 4869–4888, account target invocation at 5096–5100, named-positive requirements at 5233–5245 and exact-subject outcome gating at 5276–5295. desktop/scripts/build-electron-runtime.mjs: full source. capture_portable_contracts.py: changed input/case/constructor-compile/receipt hunks.

## Assessment

1. The R3 assertion matches the generated object that crosses the actual test boundary.

The generated McpRejected constructor creates own reason and isReadonly fields, initializing isReadonly to false before initPartial. generatedMcpResultFactory.rejected supplies only reason and nests that generated instance in the rejected arm of McpResult. The production control server returns the result unchanged. OriginNativePort forwards it through node:worker_threads MessagePort.postMessage; it does not call protobuf toJson or reconstruct the rejection. The child therefore receives the structured-cloned plain-object shape with isReadonly: false. The exact expected value at line 907 now reflects that producer and carrier.

The change keeps assert.deepEqual over the entire result. It does not replace it with a partial shape check, delete fields, normalize away an unexpected value, or change the producer. A wrong arm, wrong reason, missing/non-false isReadonly, or additional property still fails the assertion. No permission or result-classification behavior changes in R3.

The adjacent carrier test already expects the same field at lines 315–317. Its real shipping carrier receives an already-cloned frame before JSON.stringify at rust-coordinator-carrier.ts:69; this matters because JSON.stringify applied directly to a generated protobuf instance can use its canonical protobuf JSON representation. The test preserves that distinction. Rust has_definitive_mcp_result accepts the rejected arm only with correctly typed optional reason/isReadonly fields; explicit false remains a definitive pre-effect result. No Rust consumer change is needed for this expectation correction.

2. All three strict-helper uses retain causal safety controls.

The departed-account case obtains A's actual stamped catalog, observes B, and proves A's captured source is already retired before transition preparation completes. Its stale call still must return the exact generated rejection. requireMcp lookup stays at its prior count, factories remain one, and executed remains empty. After transition and child replacement, replaying A's serialized origin on B's native source still rejects before constructing manager B. The fresh B list/call requires different source and Attempt IDs, one exact manager-2 request, two total factories and no recorded problems.

The same-account restart case keeps the account binding/runtime identity and current MCP Attempt while requiring a new concrete source. The cloned old-origin request rejects with no execution, no new factory and no manager disposal. The fresh row retains the Attempt ID, changes the source ID, and executes exactly once against manager 1.

The held-catalog case remains separate: a catalog released after account departure returns the fixed failure rather than publishing stale origin, leaves the original factory/execute counts unchanged, and permits a fresh B catalog. R3 alters none of these controls, barriers, cleanup paths or named cases.

3. Source, account, Attempt and Host relay authority remain distinct.

The concrete native server generates the source UUID and supplies its synchronous assertion separately from parsed args. It retires settlement before external shutdown/problem callbacks. The production provider binds the request to the exact account-runtime object and captured account lifetime; it does not reacquire a replacement binding later. Account observe retires the old lifetime synchronously.

The MCP adapter checks that serialized source ID against the independently supplied native source and requires the exact current Attempt. requireMcpAttempt cannot reserve or await a replacement. The adapter checks currentness around its manager await and carries the same synchronous assertion into the facade. After facade admission its outer catch does not reclassify a late outcome.

Rust validates the bounded versioned origin and stores it with a private relay incarnation. The real bridge retains the descriptor across AutoReview and validates its binding before execute publication. In-process grouping and reconstruction move the per-row binding, including duplicate public names. Canonical protobuf encoding omits it and external decode sets origin to None. The original-scope constructor changes explicitly give built-ins/ordinary fixtures no Coordinator binding.

4. Final admission and late-result custody remain unchanged by R3.

The legacy HTTP path forwards a request-local assertion through discovery and the generated backend client. The actual inference interceptor invokes it after credential/machine/privacy preparation and the request-ID callback, immediately before next(request), without an intervening await. The backend witness is set only when that callback observes McpOriginRetiredError. It does not infer non-dispatch from message text or from a currentness check in catch. Separate request clients keep concurrent sources isolated.

The retained tests distinguish held pre-next retirement from already-entered success, server-declared failure, transport error and even typed failure originating from next. They require zero next calls for the former and one original call/no replacement for the latter. Disabled raw-tool checks and the fresh enabled sibling remain present. This is source evidence about the authored causal controls; none of those cases was executed by this reviewer.

5. The corrected target remains in active workflow selection.

The existing --plugin-skills-contracts build bundles coordinator-account-runtime.contract.test.ts with the shipping build configuration. The native attachment-binding/MAB command runs its entire bundle. Its receipt lists all three exact OD account case names, requires each successful row exactly once, verifies positive aggregate counts and zero failed/cancelled/skipped/todo, checks node/capture exits, and binds the source to FABUSHI_TARGET_SHA and fixed U.

The separate plugin-edge command still runs control-server, executor, complete runtime and backend-origin targets and supplies the actual built carrier artifact. Rust/App relay, canonical projection and constructor adaptations remain represented by the existing App/portable targets and constructor compile gate. Authored workflow selection is not evidence that any run has completed.

## Findings, recommendation and residual boundaries

No falsifiable defect was identified in the focused R3 change or its affected source, account, Attempt, generated-result and carrier integration. There is therefore no finding with severity, failing trigger or requested source fix to report. The recommendation is to accept this focused source fix and proceed through the existing exact-published-H Actions verification process.

Do not interpret this recommendation as a green test, unitVerified=true, globalAcceptance=true, a merge/release approval, or an OCRM completion flag. No build, linter, generator, schema check, test, local recovery, workflow trigger or validation execution occurred. No external MCP account/service, actual Electron utilityProcess/Rust-binary end-to-end run, platform package, device or signing gate was exercised. The account test controls child/facade effects; the carrier test controls its echo peer; the HTTP test controls its loopback service. Their scope cannot be combined into real-provider acceptance without the required evidence.

Official and Box final resource boundaries remain explicitly open. So do same-manager registration changes, native credential/session epoch capture, canonical Rust permission migration, durable process-loss reconciliation, and the original OCRM G01–G12/AC01–AC24 acceptance obligations. No cancellation or rollback of an already-entered external effect is claimed.

## Complete current 77-path context and disposition

Codes describe this round's source-review use, not execution/acceptance:
F = focused helper plus its complete origin fixture/consumers read deeply.
L = relevant lifecycle/security/admission/product hunks and adjacent owner boundaries read semantically.
T = origin-related causal test/interface hunks inspected.
C = mechanical origin: None constructor/fixture adaptations inspected; unrelated file bodies not re-read.
D = specification/evidence/build metadata disposition through its consumers.
P = per-row in-process projection/canonical boundary hunks inspected semantically.

| Path | Current Git blob | Disposition |
| --- | --- | --- |
| .github/workflows/rust-desktop-runtime.yml | 21027a8dac6fc0b23959cf84c946f88fe24fad26 | D |
| desktop/scripts/build-electron-runtime.mjs | effeffd58a684e803d64715494097c8b741f43c7 | D |
| projects/openclaw-rust/docs/specs/mcp-account-listing-retirement.md | 1a5edb91bc4a943a1b7d73ffadcb52a1c9df0d59 | D |
| projects/openclaw-rust/parity-ledger.json | a5b005be86ceee0230448d16f24a0d7cadbbcb25 | D |
| projects/openclaw-rust/requirements-traceability-matrix.json | 9a59949b150ef7f9b31ecbc45b2a65283074ec68 | D |
| projects/openclaw-rust/scripts/capture_portable_contracts.py | 18bf24178b066f063391d395e3b30d888205b442 | D |
| source/electron-main/adapters/mcp-oauth.ts | 1684a9cfafd02f6cdd96091b7b85d9efb3f3dbcc | L |
| source/electron-main/coordinator/coordinator-account-runtime.contract.test.ts | 517cd42f6af6d6114790d4144cc7ba9156f4e308 | F |
| source/electron-main/coordinator/coordinator-control-server.contract.test.ts | 19318f75ff5454a583801351ddbd6aa1a3ff067d | T |
| source/electron-main/coordinator/coordinator-control-server.ts | 64f335a78cbc0ca190f42caf225476eb1d2e30cc | L |
| source/electron-main/coordinator/coordinator-executors.test.ts | ce64821253c942771e08694b55edf273fb34fa8d | T |
| source/electron-main/coordinator/coordinator-executors.ts | 206624baf04310d6b77e10ea28ae3c4b156a7e1f | L |
| source/electron-main/coordinator/coordinator-launcher.ts | 50c1c7ff634839fc02266efdf742c5db39252b8d | L |
| source/electron-main/coordinator/production-provider.ts | c21a21b5803775e74716994afe44a68b4b1ece1e | L |
| source/electron-main/main-production-services.ts | 5729119ba67f1ffadc21abac2d5b87f062943ebf | L |
| source/electron-main/mcp/desktop-mcp-manager.ts | 6b73527351ef9bb7a11e75c75dfc26f27a5002e1 | L |
| source/electron-main/mcp/mcp-runtime.contract.test.ts | 7e523aef17d0d47f4291b28cc38984d11c52c066 | T |
| source/electron-main/mcp/mcp-runtime.ts | b1d651721db46b8752fc9c1ef95b689082e6c94a | L |
| source/host/app/src/main.rs | eafdf6a0cbae3651bb70d55c5cae719e31a89fdb | L |
| source/host/src/cloud_agents/cloud_agent_tool.rs | 382715a992433b4900749cd65ac12e289fe33e5a | C |
| source/host/src/extensions/inference/provider_session.rs | 5cba4171ca0ea2806c1f5b715ca366581196c841 | L |
| source/host/src/extensions/mcp/production_box_state.rs | 7ed580dabb0edfd522c2ef5c6415b7d2fdf76e50 | C |
| source/host/src/ports/mcp_state_executor.rs | 82aae3d6897b758c6c74eee206015409318c5be6 | P |
| source/host/src/runner/box_tool_access.rs | 7d856c9f39bf50642d3d757451f0cc954bc47b1b | C |
| source/host/src/runner/coordinator_tool_relay.rs | 64f0548dd1c93c6813039aaa5dce6b2ab3b8f0d1 | L |
| source/host/src/runner/tools/box_help_tool.rs | dc4c282b6b70f4f7f79ac15bb383f06838e46b47 | C |
| source/host/src/runner/tools/mcp_meta_tools.rs | aed2d181ba495ad529f4eb083a42a76b6c460439 | C |
| source/host/src/runner/tools/sand_agent_management_tools.rs | b846e5238562896cdfd35c20aa11b28f6e12b2d0 | C |
| source/host/src/runner/tools/sand_await_shell_tool.rs | ef4ab7f5835396bac1ec4d9b28d3e2e52e9d25b5 | C |
| source/host/src/runner/tools/sand_browser_tools.rs | 5144b22f69433e86e665c9b6324bc181b8a766ba | C |
| source/host/src/runner/tools/sand_computer_tool.rs | 1b3f4d6608876535607a57b422ed9aeee49f9655 | C |
| source/host/src/runner/tools/sand_external_machine_tools.rs | 7f70954155982bc208fda2ded5019001914a7cb6 | C |
| source/host/src/runner/tools/sand_file_transfer_tools.rs | bcff5d7743cb92423754db13e847f94b53b85617 | C |
| source/host/src/runner/tools/sand_generate_image_tool.rs | ea30731cb35d34943b8225ddebd8d9829eea7317 | C |
| source/host/src/runner/tools/sand_mcp_management_tools.rs | 61d592676b2f6bcd5c469f091bb1975f750585d2 | C |
| source/host/src/runner/tools/sand_multitask_todo_tool.rs | cdd1654f19bdb4f1eb1ca009346e2824e8e40ce5 | C |
| source/host/src/runner/tools/sand_reaction_tool.rs | 96bd37f8d1703d337da92d101a4186a66b850993 | C |
| source/host/src/runner/tools/sand_state_tool.rs | fe0bb0832669723d0df93b18f562d1a9e9970634 | C |
| source/host/src/runner/tools/sand_subagent_management_tools.rs | 1573fbe0b18203fe79d596675e1f0c8396fe50ad | C |
| source/host/src/runner/tools/sand_task_subagent_tool.rs | c12134aff0d106a5931b59c8d3bea48a3e992f18 | C |
| source/host/src/runner/tools/sand_web_tools.rs | b01ebb034d23337bc345bef750c911cc8447b9f2 | C |
| source/host/src/runner/tools/send_message_tool.rs | 75c7ef377d9fd5482d1082138064cccdacbe48b5 | C |
| source/host/src/runner/tools/turn_toolset.rs | 605b17103d3f46a48538f9df4312425ae125dc37 | P |
| source/host/src/runner/turn_agent_composition.rs | f4d8138ca1ff8063903ed206160eeb6702e983f6 | P |
| source/host/src/runner/turn_agent_composition/tool_call_recorder_tests.rs | a3a6618ced8384d4edc93704a6bfff580d959bbf | C |
| source/host/tests/box_mcp_exec_contract.rs | 46420600c3db19f6d6f358bb1e28fc558220fdb4 | C |
| source/host/tests/client_side_tool_v2_projection_contract.rs | 6ff566e2dd2ae05d936f231de4c4ae4c3f0ff91f | C |
| source/host/tests/cursor_inference_transport_contract.rs | 23118694b64ac543d545108640adb4ccd4a76d55 | C |
| source/host/tests/external_machine_tool_contract.rs | 92ec3c41244ef00b1552d621151658b24d13cea3 | C |
| source/host/tests/fabushi_responses_credential_context_contract/automatic_attempt_binding.rs | 9b0e5fa162dfe224c90bba1a725a23dc0404ac2f | C |
| source/host/tests/inference_provider_session_contract.rs | f5192b9237c041c663508823b2cbad2fa92450ca | C |
| source/host/tests/mcp_meta_tools_contract.rs | a16364eacc694c89e37012021775668bb5cd0844 | C |
| source/host/tests/mcp_state_executor_contract.rs | c7aa74419cd77abb20c1da925e0a37c564063de9 | T |
| source/host/tests/openclaw_memory_contract.rs | a50d3c7390a755b82734cf6313148227958d88d8 | C |
| source/host/tests/production_checkpoint_restart_contract.rs | dc46aef65c9f5dc65903196e519aac6bcf9564ad | C |
| source/host/tests/production_turn_input_projection_contract.rs | 88d4a82e933cf63ab12e2dc7d92c598cc5642a89 | T |
| source/host/tests/runner_box_tool_composition_contract.rs | 65d6142be02840f89bfcf7408287e00663272ade | C |
| source/host/tests/runner_communicate_listener_contract.rs | e1ba4f0240d9680ff0a3691232d6a4922615918b | C |
| source/host/tests/runner_effect_custody_contract.rs | f0d9e2b2d6e1b94d0338bcf3ad7a8ab071c325db | C |
| source/host/tests/runner_production_bridge_contract.rs | 696e43f1dbcd88f4f98358463d3384ac5c2713d5 | C |
| source/host/tests/runner_routed_provider_contract.rs | 39a19db6cc41ab3e5bb42168f7cbfd6eb7e17816 | C |
| source/host/tests/sand_action_audit_contract.rs | 6999681247230bbca2c98196fb2c82abdc52cba0 | C |
| source/host/tests/sand_browser_tools_contract.rs | 5caf3e0c7b7175b6808109d8ed459641c4b0e44f | C |
| source/host/tests/sand_mcp_management_tools_contract.rs | 39df7eb8be5a9c6d74ecced2900067346f155d15 | C |
| source/host/tests/sand_reaction_tool_contract.rs | 9a0f3020cc5b4902159fd98cafb6c2fd767db711 | C |
| source/host/tests/sand_spotlight_tools_contract.rs | 9698db000d4000b514c351699baf1c5d92689b94 | C |
| source/host/tests/sand_state_tool_contract.rs | 5398cb621d013e608a4a8021c3fe71f8a680bb21 | C |
| source/host/tests/send_message_tool_contract.rs | 396c27b41d8b223dcde3d550f45b61939a5c4295 | C |
| source/host/tests/turn_observation_contract.rs | 7271a1b61bf25edc2e7cbc3f4cbb87c0b3bcfe02 | C |
| source/host/tests/turn_toolset_await_mcp_contract.rs | 2615f4126b66a2e22211eccea5b92e6afb24b020 | C |
| source/host/tests/turn_toolset_contract.rs | 0dd1d4fa1be3165892acd4f1345c8816634ff467 | C |
| source/shared/node/cursor-backend/backend-mcp-exec.ts | db92e823e711204e87f54f8a54372df326a82622 | L |
| source/shared/node/cursor-backend/backend-mcp-origin.contract.test.ts | 79bdbde6c51ce56506755abb559f3d5be6325056 | T |
| source/shared/node/cursor-backend/cursor-inference.ts | 8cc6b46db1f36c34c81ab3160e1abcb878c66091 | L |
| source/shared/node/mcp/mcp-result-factory.ts | 503e2960484f5fd52b6b449de9c95941764be877 | L |
| source/shared/node/mcp/tools-discovery.ts | cb6a0dc007d8abddf0589ee56982cb13b9dfe9f0 | L |
| source/shared/rpc/coordinator-port.ts | 154b9c0f2e356288fc84bd9226a542fd492d20ee | L |

Publication destination verified through GitHub repository metadata: bhrumom/fabushi-desktop, repository ID 1375312893, owner bhrumom (Organization), private=false, push/admin permissions true. This report contains same-repository source facts and review conclusions; it contains no credential, secret or private user data. Publication is one report-only Git blob, with no branch/ref/commit change.

End of terminal report.
