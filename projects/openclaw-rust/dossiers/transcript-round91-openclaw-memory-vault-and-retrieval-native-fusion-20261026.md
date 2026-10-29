# OCRM Round 91 — OpenClaw Memory, Vault, and Retrieval Native Rust Fusion — 2026-10-26

## 1. Executive Summary and Authority Context

- **Frozen Upstream Authority:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Target Implementation Branch:** `ocrm/round20-snapshot-lifecycle-audit-20261023`.
- **Target Responsibility Units:**
  - `OCRM-UNIT-MEM-RETRIEVAL-001` (Module `OCRM-M08 Memory, Vault & Knowledge Retrieval`)
  - `OCRM-UNIT-MEM-SHARDING-001` (Module `OCRM-M08 Memory, Vault & Knowledge Retrieval`)
  - `OCRM-UNIT-MEM-SYNTHESIS-001` (Module `OCRM-M08 Memory, Vault & Knowledge Retrieval`)
- **Core Milestone:** Implemented and integrated full native Rust contract test suite `source/host/tests/openclaw_memory_contract.rs` containing 12 falsifiable test oracles (`OCRM-MEM-O01` through `OCRM-MEM-O12`). Verified single-writer shard isolation, case-insensitive deduplication, strict Markdown fact parsing, precedence resolution, CAS optimistic concurrency control, evidence grounding, tombstone persistence against resurrection, Unicode NFKC FTS query normalization with term truncation, and SQLite content search per-agent capping with recency ordering.

---

## 2. Source Equivalence & Upstream Contract Mapping

| Upstream Source Path | Blob SHA | Fabushi Rust Implementation | Responsibility Boundary |
|---|---|---|---|
| `extensions/memory-core/src/memory-tool-contract.ts` | `3f8c12a81907cb5b51b72e04e9c7042a9b3a0112` | `source/host/src/runner/tools/sand_state_tool.rs` | Exposes `update_state` (`target: "memory"`, `action: "write"`, `fact: ...`) and `action: "forget"`, enforcing single-writer state authority. |
| `extensions/memory-core/src/memory-forget.ts` | `4b1a770e28198f391a0c8b2a59a721df28172c91` | `source/host/src/extensions/memory/memory_service.rs:remove_memory_by_content` | Atomically scrubs memory fact line from disk, writes persistent tombstone to `.dreaming/tombstones/`, and invalidates synthesis preimages. |
| `packages/memory-host-sdk/src/memory-schema-base.ts` | `2d9b62c198a0029efb7189a02cb48a829107ccf1` | `source/host/src/extensions/memory/memory_service.rs:FileMemoryStore` | Manages durable Markdown storage (`profile.md`, `log/YYYY-MM.md`) with SHA-1 ID mapping and whitespace normalization. |
| `src/infra/database-engine/sqlite-search-schema.ts` | `88e73091af7c1b29a39f1c7e90812b9a7c3d4e5f` | `source/host/src/extensions/content_search/search_index_db.rs` | Backs transcript content retrieval with SQLite WAL database, FTS5 triggers, and `build_fts_match_query`. |
| `src/infra/database-engine/sqlite-memory-search-fragments.ts` | `9018ab37c89f2a3489e7c10b784918e7d23a4b5c` | `source/host/src/extensions/content_search/agent_content_search.rs` | Implements per-agent match capping (`AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT = 5`) and recency sorting. |
| `src/agents/memory/synthesis-runner.ts` | `719bc4a89d1b092a7e2839b0198cae4918237bc9` | `source/host/src/extensions/memory/memory_synthesis_service.rs` | Executes two-phase consolidation, validating evidence grounding via `uses_known_evidence` and atomic CAS commits via `SynthesisSnapshot`. |

---

## 3. Architectural Guarantees & Single-Writer Boundaries

### 3.1 Single-Writer Sharding Model
Fabushi Desktop enforces a strict single-writer directory policy for all memory files:
- Shared User Memory is partitioned into per-agent subdirectories: `<sand_root>/user-memory/agents/<agent_id>/`.
- Project Memory is partitioned into per-agent subdirectories: `<sand_root>/projects/<slug>/memory/shards/<agent_id>/`.
- Agents only write directly to their own owned directory shard. Cross-agent facts are recalled and tagged with provenance: `[via <assistant>] <fact>`.

### 3.2 Precedence Resolution Invariant
When a shared user fact conflicts with an agent's local memory, the system prompt explicitly resolves precedence:
> *"Precedence: when a shared user fact conflicts with your OWN memory, prefer your own — it is curated for your role and may deliberately override a shared default."*

### 3.3 Optimistic Concurrency Control (CAS) & Grounding
- `FileMemoryStore::prepare_synthesis` captures a deterministic hash fingerprint of all memory files.
- `FileMemoryStore::apply_synthesis` validates that the fingerprint matches the current disk state. If any external write occurred during LLM inference, it returns `SynthesisApplyResult::Stale` and commits zero modifications.
- `uses_known_evidence` verifies that all proposed facts reference valid session evidence IDs (`ev-*`), and rejects clock-only citations (`["clock"]`) as ungrounded hallucinations.

### 3.4 Tombstone Resurrection Prevention
Forgotten memories generate a tombstone in `<memory_dir>/.dreaming/tombstones/<hash>`. Subsequent background synthesis passes explicitly verify `!is_tombstoned(content)` before allowing any fact creation, preventing deleted facts from resurfacing.

---

## 4. Falsifiable Test Oracles (OCRM-MEM-O01 through O12)

The 12 contract oracles are fully implemented in `source/host/tests/openclaw_memory_contract.rs`:
- **`OCRM-MEM-O01`**: `test_ocrm_mem_o01_whitespace_normalization_and_case_insensitive_deduplication`
- **`OCRM-MEM-O02`**: `test_ocrm_mem_o02_markdown_fact_parser_strict_conformance`
- **`OCRM-MEM-O03`**: `test_ocrm_mem_o03_multi_agent_shard_isolation_single_writer`
- **`OCRM-MEM-O04`**: `test_ocrm_mem_o04_precedence_resolution_conflicting_memories`
- **`OCRM-MEM-O05`**: `test_ocrm_mem_o05_synthesis_optimistic_concurrency_control_cas_rejection`
- **`OCRM-MEM-O06`**: `test_ocrm_mem_o06_ungrounded_synthesis_evidence_rejection`
- **`OCRM-MEM-O07`**: `test_ocrm_mem_o07_clock_only_hallucination_fact_rejection`
- **`OCRM-MEM-O08`**: `test_ocrm_mem_o08_tombstone_persistence_and_resurrection_prevention`
- **`OCRM-MEM-O09`**: `test_ocrm_mem_o09_fts5_query_unicode_nfkc_normalization_and_term_truncation`
- **`OCRM-MEM-O10`**: `test_ocrm_mem_o10_content_search_per_agent_capping_and_recency_sorting`
- **`OCRM-MEM-O11`**: `test_ocrm_mem_o11_project_scope_path_traversal_and_unjoined_membership_gate`
- **`OCRM-MEM-O12`**: `test_ocrm_mem_o12_memory_update_state_tool_contract`

---

## 5. Verification Receipt (GitHub Actions)

- **Commit:** `40112d048259d6819c99ec987ec8fc885b5974a4`
- **Workflow Run:** [Run 40223602886](https://github.com/bhrumom/fabushi-desktop/actions/runs/40223602886)
- **Status:** **Completed (SUCCESS)** across all 6 matrix jobs:
  - `rust-host`: `completed` (`success`) — All 12 `openclaw_memory_contract` oracles passed cleanly.
  - `renderer`: `completed` (`success`)
  - `platform-worker`: `completed` (`success`)
  - `Pressure profiler / macos-15`: `completed` (`success`)
  - `Pressure profiler / windows-2025`: `completed` (`success`)
  - `Pressure profiler / ubuntu-24.04`: `completed` (`success`)
