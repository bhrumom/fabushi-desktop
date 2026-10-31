# OCRM Technical Dossier — Unit 9: Agent Loop, ACP & Turn Execution (`OCRM-UNIT-AGENT-TURN-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M05/M06 (Agent Harness, Turn Execution, ACP & Model Catalog Subsystems)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core agent execution, loop detection, session compaction, model pricing, and ACP policy files (totaling 1,626 lines) were audited line-by-line and mapped into canonical shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0021` | `src/acp/commands.ts` | `ed35270d5976a1700d7b0adc7367b0b5b4352f44` | 45 | 1..45 (full-file) | `OCRM-RESP-AGENT-ACP-COMMAND-001` |
| `part-0021` | `src/acp/policy.ts` | `5cbf002ee4ad1c9e2529417ec4b3d054a73e69fb` | 48 | 1..48 (full-file) | `OCRM-RESP-AGENT-ACP-POLICY-001` |
| `part-0025` | `src/agents/sessions/compaction/compaction.ts` | `e26a8d7ba3ddf06b53cb12ec1694fef437a3c306` | 722 | 1..722 (full-file) | `OCRM-RESP-AGENT-COMPACTION-001` |
| `part-0025` | `src/agents/tool-loop-admission.ts` | `bf5d08bb075195a0488c0e0747141f2d61f16605` | 299 | 1..299 (full-file) | `OCRM-RESP-AGENT-LOOP-RUNNER-001` |
| `part-0025` | `src/agents/tool-loop-detection.ts` | `3924feee594b238384b655fcb8b5db0c96c4b2b2` | 340 | 1..340 (full-file) | `OCRM-RESP-AGENT-LOOP-DETECTION-001` |
| `part-0042` | `src/model-catalog/pricing.ts` | `f69ca7e8c3395725d2db24c7f123f95b9d316cb1` | 172 | 1..172 (full-file) | `OCRM-RESP-AGENT-PRICING-CATALOG-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-AGENT-LOOP-RUNNER-001`**:
   - **Behavior**: Active agent turn run loop, `AlreadyRunning` conflict rejection, and pending turn deferred supersedes.
   - **Rust Owner**: `source/host/src/runner/mod.rs:TurnRunShell`.
   - **Oracles**: `OCRM-AGENT-O01`, `OCRM-AGENT-O02`.

2. **`OCRM-RESP-AGENT-LOOP-DETECTION-001`**:
   - **Behavior**: Tool loop detection, immediate cancellation supersedes, and monotonic generation increments in `TurnOwnerToken`.
   - **Rust Owner**: `source/host/src/runner/mod.rs:TurnOwnerToken`.
   - **Oracles**: `OCRM-AGENT-O03`, `OCRM-AGENT-O04`.

3. **`OCRM-RESP-AGENT-COMPACTION-001`**:
   - **Behavior**: Context window token overflow compaction threshold triggering, history message pruning, and summary preservation.
   - **Rust Owner**: `source/host/src/runner/mod.rs:TranscriptCompactor`.
   - **Oracles**: `OCRM-AGENT-O07`, `OCRM-AGENT-O08`.

4. **`OCRM-RESP-AGENT-PRICING-CATALOG-001`**:
   - **Behavior**: Model token cost estimation, prompt and completion token multiplier accounting, and unknown model fallback rate.
   - **Rust Owner**: `source/host/src/runner/mod.rs:PricingCatalog`.
   - **Oracles**: `OCRM-AGENT-O09`, `OCRM-AGENT-O10`.

5. **`OCRM-RESP-AGENT-ACP-COMMAND-001`**:
   - **Behavior**: ACP available commands whitelist resolution, viral turn tainting on unreviewed command execution, and auto-review resolution.
   - **Rust Owner**: `source/host/src/runner/mod.rs:AvailableCommand`.
   - **Oracles**: `OCRM-AGENT-O05`, `OCRM-AGENT-O06`.

6. **`OCRM-RESP-AGENT-ACP-POLICY-001`**:
   - **Behavior**: ACP dispatch policy enforcement, runner settlement receipt emission, and zombie lease watchdog timeout.
   - **Rust Owner**: `source/host/src/runner/mod.rs:SandAutoReviewDecision`.
   - **Oracles**: `OCRM-AGENT-O11`, `OCRM-AGENT-O12`.

## 2.1 Current live-candidate drift — 2026-10-31

The historical ACP command source above is bound to the then-locked subject
`ad8d12f126d29b0db204187adf5092db80294e70` and remains historical evidence.
Read-only GitHub contents checks against the current discovery candidate
`openclaw/openclaw@b924498923dff81a02fb8a89d57828717c6e160d` found no
`src/acp/commands.ts`; the path returns 404. The command surface is now
represented by at least:

- `src/auto-reply/commands-registry-list.ts::listChatCommands`, which adds
  skill command projections and filters operator-disabled command keys;
- `src/auto-reply/commands-registry.data.ts::getChatCommands`, which caches
  the built-in registry and supplies runtime thinking-level choices; and
- `src/auto-reply/commands-registry.runtime.ts`, which exports the lazy-boundary
  runtime facade.

This is a source-contract drift observation, not a current-source closure or
implementation approval. The existing parity row for
`OCRM-RESP-AGENT-ACP-COMMANDS-001` must retain its historical source identity
until a fresh exact-subject Actions inventory and dependency closure accepts
the live candidate. A future rebaseline must remap the responsibility from the
old ACP-only `AvailableCommand` shape to the current chat-command registry,
including feature-flag filtering, skill command injection, aliases, argument
parsing, scope/category metadata, and the boundary between chat commands and
ACP presentation. No Rust target or completion flag is upgraded by this note.

## 2.2 ACP Rust owner/consumer audit — 2026-10-31

The current Fabushi checkout contains
`source/host/src/extensions/plugin_sdk/acp_runtime.rs`, but its
`AcpSessionManager`, `verify_session_access`, and `generate_doctor_report`
symbols have no production caller. The only references are
`source/host/tests/openclaw_plugin_sdk_contract.rs` and the module export in
`plugin_sdk/mod.rs`. The manager owns an in-memory session map and its doctor
method returns `Ready` from that local map; this is a contract slice, not the
canonical Session/Runner ACP lifecycle or a live ACP transport.

The existing Runner search likewise found no `AvailableCommand` or
`get_available_commands` implementation, and no production route exposing the
historical ACP command list. The current live OpenClaw candidate's command
registry drift described above means the old parity integration
(`runner/mod.rs` as both producer and consumer) is not evidence of real
production wiring. A valid implementation must first identify the canonical
ACP handshake/transport owner, then adapt the current Session/Runner state and
command registry without creating a second session store or treating a test
manager as a production doctor.

Disposition: keep the ACP command responsibility
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending`; retain
the historical source and implementation contract until exact-subject source
closure permits a rebaseline. The standalone `AcpSessionManager` contract
does not satisfy production consumer, security-context, or packaged ACP
acceptance evidence.

The ACP policy row requires the same separation. A repository-wide search
found no production ACP dispatch-policy producer or consumer for the historical
`src/acp/policy.ts` contract. `SandAutoReviewDecision` is a policy-shaped
Runner helper, while `source/host/src/extensions/transcript/production_runtime.rs`
owns a separate Host turn lease and watchdog lifecycle. The O11 settlement and
O12 zombie checks in `openclaw_agent_turn_contract.rs` construct local
`TurnSettlement`/`TurnWatchdogLease` fixtures and do not establish an ACP
transport or dispatch-policy call chain. Therefore the parity row
`OCRM-RESP-AGENT-ACP-POLICY-001` retains `implementation=not-started`,
`wiring=not-wired`,
`verification=not-run`, and `acceptance=pending`; the independent Transcript
runtime must not be counted as ACP compatibility.

The context-compaction mapping was also rechecked. A non-test search found no
production caller for `ContextWindowTracker::check_compaction`; production
prompt code currently reads a persisted compaction epoch through
`HostRunnerComposition`. The O09 contract calls the tracker directly, while
O10 defines a local `validate_tool_schema` fixture inside the test file rather
than a production tool-admission owner. The epoch projection and those tests
therefore do not prove the mapped threshold-compaction plus tool-schema
responsibility is implemented or wired. The parity row
`OCRM-RESP-AGENT-CONTEXT-COMPACTION-001` is corrected to
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending` until the real Runner producer/consumer path exists.

The tool-loop mapping was rechecked after the detector implementation landed.
`TurnRunShell::ToolLoopGate` now canonicalizes tool arguments, bounds per-turn
history, classifies the third consecutive identical call, and requests
cancellation through the existing `RoutedProviderCancellation`. The live
chain is `ProductionTurnAgentOwner` -> `TurnAgentComposition` ->
`CheckpointRecordingRoutedToolBridge::call_tool` -> the existing routed
provider executor. The parity row
`OCRM-RESP-AGENT-LOOP-DETECTION-001` is therefore
`implementation=implemented`, `wiring=wired`, `verification=not-run`, and
`acceptance=pending`; O03/O04 remain lifecycle oracles and do not replace the
dedicated frozen tool-loop verification.

The model-pricing mapping was rechecked as well. `TurnUsage` is a real
production-carried usage shape, but the current non-test source has no
`PricingCatalog`/model-cost calculation caller. The remote catalog projection
and `RateLimitAdmissionController` are defined under the Inference extension
without a production consumer in this path; the Runner adapter's HTTP status
handling provides retry/timeout classification, not the mapped provider
fallback chain. The O07/O08 tests therefore do not establish the combined
pricing and fallback responsibility. The parity row
`OCRM-RESP-AGENT-MODEL-PRICING-001` is corrected to
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending` until model pricing and provider fallback have a real
producer/consumer path.

The loop-runner mapping has a narrower partial positive. The production
`ProductionTurnRunShellAdapter` calls `TurnRunShell::begin_run`, marks the
owner dispatched, and settles the active run, so the active-run lifecycle is
reachable. However, the deferred supersede behavior in O02 is implemented by
the local `DeferredTurnQueue` declared inside
`openclaw_agent_turn_contract.rs`; no production queue producer/consumer was
found. Because the mapped responsibility requires both `AlreadyRunning`
handling and deferred supersede, the parity row
`OCRM-RESP-AGENT-LOOP-RUNNER-001` is corrected to
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending`. The active-run adapter remains recorded as partial
evidence, not full responsibility acceptance.

The parity row's blocker attribution was rechecked at the same time. The prior
`BL-MEDIA-DOC-TRUNCATION-UPSTREAM-CALLER-001` reference did not describe this
ACP transport/consumer gap; the row now points to the existing
`BL-PLUGIN-SDK-ACP-WIRING-001` blocker. This is a blocker-classification
correction only and does not promote implementation, wiring, verification, or
acceptance.

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_agent_turn_contract.rs`)

1. **`OCRM-AGENT-O01`**: `test_ocrm_agent_o01_active_stream_conflict_rejection`
2. **`OCRM-AGENT-O02`**: `test_ocrm_agent_o02_deferred_supersedes_queue`
3. **`OCRM-AGENT-O03`**: `test_ocrm_agent_o03_immediate_cancellation`
4. **`OCRM-AGENT-O04`**: `test_ocrm_agent_o04_owner_token_monotonic_generation`
5. **`OCRM-AGENT-O05`**: `test_ocrm_agent_o05_viral_turn_tainting`
6. **`OCRM-AGENT-O06`**: `test_ocrm_agent_o06_auto_review_approval_gate`
7. **`OCRM-AGENT-O07`**: `test_ocrm_agent_o07_model_catalog_pricing_calculation`
8. **`OCRM-AGENT-O08`**: `test_ocrm_agent_o08_provider_fallback_chain`
9. **`OCRM-AGENT-O09`**: `test_ocrm_agent_o09_context_window_compaction_trigger`
10. **`OCRM-AGENT-O10`**: `test_ocrm_agent_o10_tool_argument_validation`
11. **`OCRM-AGENT-O11`**: `test_ocrm_agent_o11_runner_settlement_receipt`
12. **`OCRM-AGENT-O12`**: `test_ocrm_agent_o12_zombie_turn_lease_expiration`

---

## 4. Invariant Verification & Global Conservation

- **Exact Mapped Paths**: 112 (cumulative across Units 1–9)
- **Omitted Known**: 0
- **Total Tracked Paths**: 58,750
- **Unread / Unclassified Paths**: 58,638
- **Strict Conservation**: $112 + 58638 = 58750$.
- **All Global Completion Flags**: `false` strictly preserved.
