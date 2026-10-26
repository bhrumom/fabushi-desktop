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

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_agent_turn_contract.rs`)

1. **`OCRM-AGENT-O01`**: `test_ocrm_agent_o01_active_inference_stream_conflict_rejection`
2. **`OCRM-AGENT-O02`**: `test_ocrm_agent_o02_deferred_supersedes_on_pending_turn_start`
3. **`OCRM-AGENT-O03`**: `test_ocrm_agent_o03_immediate_cancellation_supersedes_active_turn`
4. **`OCRM-AGENT-O04`**: `test_ocrm_agent_o04_turn_owner_token_monotonic_generation_fencing`
5. **`OCRM-AGENT-O05`**: `test_ocrm_agent_o05_viral_turn_tainting_upon_unreviewed_execution`
6. **`OCRM-AGENT-O06`**: `test_ocrm_agent_o06_auto_review_decision_resolution`
7. **`OCRM-AGENT-O07`**: `test_ocrm_agent_o07_context_token_compaction_threshold`
8. **`OCRM-AGENT-O08`**: `test_ocrm_agent_o08_system_prompt_pinned_across_compaction`
9. **`OCRM-AGENT-O09`**: `test_ocrm_agent_o09_model_pricing_catalog_cost_estimation`
10. **`OCRM-AGENT-O10`**: `test_ocrm_agent_o10_unknown_model_fallback_rate`
11. **`OCRM-AGENT-O11`**: `test_ocrm_agent_o11_runner_settlement_receipt_emission`
12. **`OCRM-AGENT-O12`**: `test_ocrm_agent_o12_zombie_turn_attempt_lease_expiration`

---

## 4. Invariant Verification & Global Conservation

- **Exact Mapped Paths**: 112 (cumulative across Units 1–9)
- **Omitted Known**: 0
- **Total Tracked Paths**: 58,750
- **Unread / Unclassified Paths**: 58,638
- **Strict Conservation**: $112 + 58638 = 58750$.
- **All Global Completion Flags**: `false` strictly preserved.
