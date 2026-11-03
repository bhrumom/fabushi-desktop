# LXO-v1 frozen R1 — independent structure / reuse / performance review

## Verdict

**No blocking structure, reuse or performance defect was established in the frozen candidate. No repair is requested by this review.** This is a read-only source-review verdict, not a build/test pass or product acceptance. Runtime verification is explicitly pending the exact frozen subject's GitHub Actions receipts.

Findings: none. Preference-only refactors were not promoted into findings. The existing full termination-controller / process-tree / real provider/device acceptance gaps remain gaps; this review neither cuts those product responsibilities nor treats them as newly completed by LXO.

## Review identity and independence

- Repository: `bhrumom/fabushi-desktop`.
- Exact base: `7722759802847348346c538e7eaf3b25058f40f1`; base tree read through GitHub: `930846450247862a249453fc3395b6eb43f0e136`.
- Exact immutable candidate: `e4f1b8d2c4f682d3de7d993ccad5cdd8b3ed23e8`; Git commit read confirms its sole parent is the declared base and its tree is `d3fda339f5b7d1dc480287051602d477bfbd5fe7`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Full scope manifest: `da8da1204c5fb3a42010e1954fb860ef25b39e89`.
- Complete frozen unified diff: `d6c11aa4cfab692eea297de1accdd0ad71a02d23`.
- Applied launch assignment: model `gpt-6.1-sol`, effort `xhigh`. The applicable root AGENTS policy explicitly pins Codex structure/reuse/performance lenses to this model; the assignment recorded a supported override. The selected effort covers shared state ownership, lock ordering, cancellation/lifetime, durable publication and real production consumers. No fallback was used or requested.
- No other reviewer's report, prior arbitration or root disposition was supplied. This reviewer did not spawn any agents.
- No source, ref, tree, commit or workflow was changed. The only authorized optional write is this unreferenced immutable review-report blob; no push is performed.

## Authority actually read

Root `AGENTS.md` blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2` (including independent/frozen review, scope, effort/model and verification rules); project `SOURCE_OF_TRUTH.md` current authority/execution entries; OCRM master `docs/specs/openclaw-rust-equivalence-migration.md` blob `69c9667af2328e20a1fba09ebf80c5aab4d5c336`, complete §17; complete LXO spec `projects/openclaw-rust/docs/specs/local-exec-outcome-ownership.md` blob `bd22147504fe06d357b5e5b16a81f426196059f0`; current process-exec-shell dossier blob `c124391ce66034e4b4d8200e92133d80c7b99c96`, with its historical identities kept historical.

Upstream `src/process/exec-termination.ts` blob `8b799aea4cccb13dd5b76b4b25c49b4c025cf5d1` was read in full from the upstream repository. Its signal-delivery/observed-exit distinction supports the LXO contract; it is not evidence that the complete upstream controller has a Fabushi production caller.

## Source assessment

### One operation owner, permission facades, identity and publication

`HostLocalExecExtension` owns one `Arc<GatewayLocalExecOperationOwner>`. Cloning the Host extension keeps that Arc, and every `sandbox(gate)` passes it into its facade. The facade's bridge and process-capacity owner remain the existing Host-owned objects. Directly constructed isolated sandboxes still allocate their own owner. The new wrapper contains the existing active map, bounded history, journal path/health and transaction lock; it is not a second execution runtime, journal database, permission authority or replay fence.

`GatewayLocalExecOperationOwner::with_state` captures the configured root on first owner use and loads the existing version-2 journal once. Merely constructing a Host extension, sandbox, manager or remote accessor performs no journal load/publication. Operation/status access can initialize the owner, which matches the spec's first-owner-initialization rule.

`register_operation`, `mark_operation_started` and `finish_operation` now perform their state mutation and journal publication under the same owner mutex. The terminal record enters history without an unlock between active removal and insertion. Duplicate admission checks both collections in that same transaction. A later admission/publication cannot interleave with an earlier snapshot's replacement/fsync, so the source no longer permits torn active/history views or older writer overwrite within this Host owner.

The registration ticket is the bridge request ID returned only after successful registration. A permission/computer/duplicate refusal leaves `registered_request_id` empty; its outer error branch therefore cannot finish an older operation with the same public ID. Started/terminal mutations require matching public operation ID plus bridge request ID. Failed provider-channel admission after successful registration retires only that registration.

`cancel_operation_for_agent` checks the owning agent and sends through the matching active record without releasing the owner lock in between. Deadline cancellation also carries the registration's request identity. No lookup/replacement gap remains in these changed paths.

### Lock ordering and actual I/O/cancellation effects

I traced the complete bridge request/send/close/cancel path, not only the manager's mutex declaration. In `SandLocalExecBridge::request_with_before_send`, the bridge-state mutex is released after provider selection/pending insertion and before `before_send` invokes owner registration. On callback refusal, bridge pending cleanup occurs after the owner transaction has returned. `LocalExecCancellationHandle::cancel` uses its captured unbounded `std::sync::mpsc::Sender` and one-shot atomic; it does not take bridge.state. The owner-held cancellation path therefore introduces no reverse bridge-state/operation-owner lock order in this chain.

Permission authorization happens before the registration callback, outside the operation lock. The existing permission owner and per-turn direction epoch remain responsible for policy; shared operation custody does not make distinct permission facades interchangeable. Telemetry callbacks on Throw are likewise outside the operation transaction.

Publication deliberately includes directory creation, JSON serialization, temporary create-new/write/file fsync, replacement and Unix parent fsync while the transaction is held. This serializes status/admission/cancel against a publication; it is the explicit coherence tradeoff in LXO-01, rather than an accidental lock around provider execution. The owner lock does not span response waiting or the shell's remote execution. Ordinary output chunks after the first observed activity do not trigger another journal write: `mark_operation_started` returns once already started.

`LocalExecRequest::close` continues to retire its exact pending request and request cancellation. Neither the cancel send nor closing the Host waiter claims the provider has exited. The implementation does not add broad process cleanup, locally signal a remote PID, replay an uncertain operation, or start another scheduler.

### Known/unknown outcomes and authoritative terminal facts

The shell loop recognizes only actual ShellStream Exit as Completed/Cancelled; the aborted bit selects Cancelled, regardless of an earlier requested cancel. A normal Exit after a cancel request remains Completed. StreamClose, receive failure/silence/deadline, malformed or missing client/control, opaque Throw, backend/start-timeout codes and unexpected file errors fall through to OutcomeUnknown after dispatch.

Start, output and Backgrounded set observed activity. Rejected/PermissionDenied and the dedicated pre-entry tag are known failures only before that evidence. Contradictory late refusals cannot clear it. The existing error message/Display surface is preserved while the added typed flag carries uncertainty to consumers.

Both success-shaped unknown results and error-shaped unknown outcomes finish through the same operation owner. Partial output is appended before a possible started-journal failure, then retained in the terminal snapshot. If terminal publication fails after Exit, the record keeps the observed Exit/code/aborted facts and the returned error becomes typed unknown for durability/custody. Known no-entry refusal stays known even if its journal write fails, while journal health remains poisoned for subsequent admission. Startup recovery of durable Accepted/Running to OutcomeUnknown and version/corruption fail-closed behavior are retained.

### Real consumers and reuse of existing Runner custody

The actual App constructs `ProductionExternalMachineExecutor::for_turn` with the Host extension, existing permission owner and current direction epoch. The production bridge passes it through `TurnAgentComposition` and `TurnToolsetDependencies`; the existing `ExternalMachineToolBridge` invokes it for ExternalShell and delegates status/history/cancel to that same owner. This is a reachable consumer path, not an unused adapter.

`ProductionExternalMachineExecutor::execute_shell` maps both explicit unknown manager results and typed unknown errors to `ProviderSessionError::ToolOutcomeUnknown`, retaining public operation/request identity, stable resultRef, cause and bounded observed output where the journal-owner query is available. The existing common routed-tool guard marks DownstreamOutcomeUnknown before completing the local worker receipt. Its existing per-run custody prevents further tool admission and accepted checkpoints, and existing retry/terminal reporting consumes that custody. The change creates no second Runner retry/retention mechanism.

The actual `UnifiedGatewayApi::call_openclaw_node_invoke` still performs request validation and canonical `NodeHostInvocationRuntime::start` before dispatch. It now calls the shared `execute_local_node_shell_invocation` helper. That helper projects unknown to `EXECUTION_OUTCOME_UNKNOWN`, includes stable identity plus available output/observed Exit facts, projects known errors to `EXECUTION_ERROR`, keeps confirmed Exit success, and completes the existing invocation response lifecycle. Completing that node response does not purport to terminate the remote process or create a second node-result store.

### Protocol and build boundaries

The TypeScript change is a narrow optional-field adapter. Provider permission refusal and generated decode refusal occur before `executor.execute`; the controlled-manager no-handler branch emits the tag after no handler accepted the message. Entered handler exceptions retain their existing opaque/backend/start-timeout classifications. The default production executor forwards the optional code to the actual generated ExecClientThrow type.

The Rust canonical schema already has optional `error_code = 4`, and the frozen generated TypeScript type exposes the same optional field. The production Rust codec now retains it through the existing generated binding. No new hand-written wire codec, generated source replacement, dependency/version change or runtime policy owner is introduced. I inspected the owning schema, generation entrypoint and actual consumer functions.

### Performance, output and storage bounds

The existing process-capacity custody remains shared across Host facades; its default local-exec limit is 16 simultaneous slots. Retained shell stdout and stderr are individually capped at 256 KiB on UTF-8 boundaries. Durable stdout/stderr are individually capped at 64 KiB, with a terminal history limit of 128. Thus the nominal maximum retained terminal output payload is 16 MiB before record metadata and serialization copies; this is arithmetic from source constants, not a measured allocation or throughput result.

Snapshot publication clones/serializes the bounded terminal history and current active records, as the existing journal mechanism already did. This candidate centralizes its publication order and suppresses repeated started writes; it does not introduce an unbounded replay set, second history, extra per-chunk disk publication or remote polling loop. Existing raw transport queues and arbitrary wire-frame sizes are not proven globally memory bounded by these string caps. No new regression in those unchanged mechanisms was established, and no whole-system memory/performance acceptance is claimed.

## Oracle and capture integration assessment

All 92 declared hunks across all 24 paths were read, with full relevant production functions and adjacent consumers/build/precedents. Each new blob was fetched by its immutable identity; candidate commit/tree identity was independently read back. The manifest's complete-diff accounting is source accounting, not a runtime pass.

The new manager cases use distinct bridge request IDs, actual before-send registration, real journal writes/failure obstacles, two permission facades and the owner query. The publication test pauses at the real snapshot boundary while holding the owner transaction and contends a second facade. It is relevant evidence design for serialization; I did not execute it.

The three Runner additions instantiate the actual `ProductionExternalMachineExecutor`, real Host/permission owner and `create_production_runner`, receive the real bridge request before injecting frames, and count a separate filesystem effect. No mock injects ToolOutcomeUnknown as the behavioral oracle. The source checks unknown returns with zero local pending receipts, sticky DownstreamOutcomeUnknown, no checkpoint/retry/second dispatch and retained terminal custody while a no-Exit external worker remains alive. Known permission/pre-entry/Exit controls are paired. Controlled downstream transport does not prove a real provider/device contract.

The eight App cases call the same helper used by the live route with real LocalExec/permission and canonical invocation runtime. They exercise unknown close/malformed/deadline, known normal/aborted Exit, permission/pre-entry refusal and terminal journal failure. They are scoped helper/owner integration, not complete UnifiedGatewayApi construction or remote-node acceptance.

The six TypeScript tests use the actual generated codec/default executor/provider and actual controlled manager. They distinguish provider no-entry permission/decode, no handler, entered exceptions and cancel-without-Exit. The build script adds the test to the existing esbuild contract bundle.

Existing Actions captures are extended rather than replaced: native cleanup capture selects owner/gateway/codec/telemetry targets on the existing platform jobs; portable capture adds the real Runner names to its existing regression command; actual App capture selects all eight exact names; renderer executes the six protocol cases after the existing bundle build. Source/input bindings, positive name requirements, exit/capture outcomes and receipt/log pointers remain in those owners. The stricter native and protocol oracles have missing/duplicate/zero/ignored/nonzero/capture-failure controls. I inspected the portable existing per-label oracle rather than assuming every label has identical strictness. The new required Runner names must each occur once; none is silently supplied by source-authoring counts.

The ledger/RTM diffs link the new source/tests without promoting global completion or acceptance. The termination-controller verification downgrade to not-run is explicit. The historical raw-log archive-status artifact names the older subject and records its blocked archival status; it does not substitute for LXO execution evidence.

## Complete changed-scope inventory

The complete diff declares 24 regular UTF-8 files, 92 hunks, 2,773 additions and 401 deletions; no renames, binary or mode changes. The table records every inspected path and hunk count. Unrelated unchanged bodies of very large App/workflow/accounting files were not represented as a complete semantic audit of the repository.

| Inspected changed path | Hunks | Delta | Candidate blob |
| --- | ---: | ---: | --- |
| `.github/workflows/rust-desktop-runtime.yml` | 4 | +138/−0 | `8814c337d65c6d76a3cdf4fa7eee0d054b2194c8` |
| `desktop/scripts/build-electron-runtime.mjs` | 1 | +1/−0 | `213397b2172b3173c3d8c7ed31c1696b11425bd5` |
| `projects/openclaw-rust/evidence/local-exec-outcome-647e0932-20261103/source-contract.json` | 1 | +240/−0 | `6ec575e5293e947a4c69ee083a29bb40b9b30809` |
| `projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/execution-r6/actual-app/terminal/raw-log-archive-status.json` | 1 | +24/−0 | `ee58f8f8dae3b465c27f10b03df864f9aeb8bfdb` |
| `projects/openclaw-rust/parity-ledger.json` | 13 | +41/−15 | `dd549878bea5608bb70da23baf2ef23ed7798b9a` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | 5 | +15/−5 | `7f9dbdd539f474ec67a7720898710ccb71f9758e` |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | 4 | +105/−1 | `b9a0020746feb58d677a841d3f37b7f7de0e4da9` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | 6 | +62/−6 | `d59ff684e62deab01daa584f3c8b6252f9ab9ea4` |
| `source/host/app/src/main.rs` | 2 | +102/−53 | `d261aa4428899dc8932a4e3e219f9e2d889428ce` |
| `source/host/app/src/tests/local_exec_outcome_tests.rs` | 1 | +244/−0 | `bfc6d3925cbbb41c4ec9866e4779bda1f0bdbe3b` |
| `source/host/local-exec/local-exec-provider.ts` | 3 | +4/−4 | `10d6a23519ab15b1c00545239e61d19f580779bc` |
| `source/host/src/extensions/local_exec/extension.rs` | 4 | +4/−1 | `508e2463e5f50acec325b42a8b2c47e15b90ed56` |
| `source/host/src/extensions/local_exec/gateway_local_exec_sand_box.rs` | 18 | +871/−291 | `cdfda2d2ca8c25a5ae1d5b15656a53176bc8d5e9` |
| `source/host/src/extensions/local_exec/local_exec_error.rs` | 1 | +12/−1 | `f8508ff904b873f93ad3d873dcc190b270e384d7` |
| `source/host/src/extensions/local_exec/production.rs` | 2 | +2/−1 | `9daa65ce980eaae31d3ea0f4045e752200d3c58c` |
| `source/host/src/runner/host_external_machine_dependencies.rs` | 1 | +36/−11 | `1e2338bd9d490e7c681ee06759735998471ae65c` |
| `source/host/tests/extensions_local_exec_contract.rs` | 1 | +4/−0 | `259f3d8426a77e4acf960b3c59406d297bbe6407` |
| `source/host/tests/gateway_local_exec_sand_box_contract.rs` | 8 | +22/−6 | `9ffcc5994757b16c75963bfc9efdeb37699d1cd9` |
| `source/host/tests/local_exec_production_codec_contract.rs` | 2 | +18/−0 | `af3d3c407820f258b52b9122a937355d3bb5d521` |
| `source/host/tests/runner_effect_custody_contract.rs` | 7 | +641/−1 | `08751e9c99d854391db862641b35ec2f6d43e969` |
| `source/local-exec-daemon/production-executor.ts` | 3 | +4/−4 | `f5b8b8276a7fa1bf31c2b7c7a8a16d1b29a639e7` |
| `source/local-exec-daemon/production-outcome.contract.test.ts` | 1 | +179/−0 | `051c9458b71666e4d73f1ccda7c8430dd07c4a9d` |
| `source/packages/agent-exec/controlled.ts` | 2 | +2/−1 | `4260b6ece63ca2b3c35482ff68762799703c8acd` |
| `source/shared/local-exec-gateway.ts` | 1 | +2/−0 | `845a1116fee2b51051a604d22d258c78a3633761` |

## Adjacent source inspection inventory

The following exact candidate-tree sources were additionally fetched and their relevant complete production functions/contracts read. This is an adjacency inventory, not a claim that all unrelated portions of large modules were audited:

- `source/host/src/extensions/local_exec/local_exec_bridge.rs` — `f19d738cec812920eb28b377015ddc8e240310ca`; provider registration/response routing, request_with_before_send, finish_request, cancel, close and Drop.
- `source/host/src/extensions/process/command_custody.rs` — `369beb8769b1b5735df15f9a67f68a8d2d7782ef`; complete capacity and guard owner.
- `source/host/src/extensions/local_tool_permission/extension.rs` — `829889281cf49162aed3b3cad8086473ec153011`; permission facade.
- `source/host/src/extensions/local_tool_permission/local_tool_permission_controller.rs` — `2befb28cc54f7b5d7bce5b2e6fb7298e46bec867`; scope/decision data, authorize/authorize_with_cancel and direction/standing-grant integration.
- `source/host/src/runner/tools/sand_external_machine_tools.rs` — `7f70954155982bc208fda2ded5019001914a7cb6`; complete ExternalMachineExecutor/ExternalMachineToolBridge contract.
- `source/host/src/runner/tools/mcp_meta_tools.rs` — `aed2d181ba495ad529f4eb083a42a76b6c460439`; common tool timeout/delivery tracker and mark-before-receipt-complete path.
- `source/host/src/runner/stream_attempt.rs` — `6eb8a3eae5556eb6d6a6c03d00059d7c1180190f`; custody/receipt/tracker, tool admission and checkpoint fences.
- `source/host/src/runner/routed_provider_runtime.rs` — `a351081d6e8e9e9c1fb51439f4d67b48e4ace886`; actual tool guard invocation and typed unknown propagation.
- `source/host/src/runner/production_turn_run_shell_adapter.rs` — `d0fe2714923338dd717284f08f05eef438db69d9`; current lifetime/custody/retry consumer adjacency.
- `source/host/src/runner/turn_agent_composition.rs` — `f4d8138ca1ff8063903ed206160eeb6702e983f6`; external executor binding into turn tool dependencies.
- `source/host/src/runner/tools/turn_toolset.rs` — `605b17103d3f46a48538f9df4312425ae125dc37`; actual ExternalMachineToolBridge composition.
- `source/host/src/runner_production_bridge.rs` — `270b711727c4a9e5ecf06e2a0254898df1e70b2f`; create_production_runner and immutable composition projection.
- `source/host/src/host_runner_composition.rs` — `b75c24202ef15600a21e25180947a1432f44061c`; production composition adjacency.
- `source/host/src/extensions/node_host/invocation_runtime.rs` — `5fe97531210e9bbf1661e24f51873fe792650249`; invocation start/complete and generation ownership.
- `source/host/src/extensions/node_host/runtime_state.rs` — `77729af350ea94510460e3466fa3b37b3e0505cf`; request admission/session generation and settlement.
- `source/host/src/extensions/node_host/invoke_protocol.rs` — `06a2f2b156dc82a499619e88f936f832df617248`; request/result/error wire contract.
- `source/host/src/extensions/local_exec/agent_v1_exec_wire.rs` — `6120c6735ffcd28eea942ed3bff8d99c6f2024f5`; generated binding consumer.
- `source/host/proto/agent/v1/local_exec.proto` — `6e1bf9e41ab7573ffeaba28c1685d81f8b857a6c`; owning Throw optional field and shell/control oneofs.
- `source/host/build.rs` — `64e0666f1fa9773181344abd8ca817996cd652a8`; complete local-exec generation/provenance function.
- `source/packages/proto/generated/agent/v1/exec_pb.ts` — `b3d569d1ad5f923444d472a08efb8bad58c5941f`; generated ExecClientThrow optional errorCode contract.
- `source/host/local-exec/local-exec-machine.ts` — `863c8ab5e02744cfd380e478c8be5db6a3ce7f55`; complete production manager builder/root guards.
- `source/host/Cargo.toml` — `e42cc9f56cc98337a49622ae9117ebd9823ef820`; complete Host declarations.
- `source/host/app/Cargo.toml` — `a2c58167a82657194b79aaf3e1e0d55577b2221c`; complete App/test-support declarations.
- `source/host/src/host_paths.rs` — `844be87e4a278bca99bc4968bbf85b6d751983b9`; configured root ownership.
- `source/host/src/durable_file_policy.rs` — `65a1f432372f2d67d6fe7fcac76be5fc6b492cab`; existing journal filename policy.

Changed manager production was read through its complete owner/state, journal load/persist/query/cancel/register/start/finish/shell/read functions and transfer adjacency. Changed App review covers the complete live node.invoke method/helper plus actual per-turn executor construction and production dependency projection. Changed TS provider/default executor/controlled-manager production functions were read completely.

## Limitations and completion boundary

No local or htch executable validation, tests, build, lint, generator, parser execution or test enumeration was performed. GitHub tools supplied immutable source reads; no local executor was used. Static source extraction/metadata accounting is not runtime validation. No Actions run or historical receipt was reclassified as the frozen candidate's result. No raw runtime log was uploaded.

This review cannot prove compile success, exercised timing/fault behavior, filesystem durability on actual hardware, process-tree termination, real external provider/device compatibility, packaged behavior, global OCRM closure or final acceptance. The authored tests and capture wiring are reviewable instruments awaiting exact-H Actions evidence. The absence of a blocking source finding does not satisfy those remaining verification/acceptance requirements.
