# OCRM Technical Dossier: Unit OCRM-UNIT-MEM-001 — Memory, Knowledge Retrieval & Synthesis Parity

## 1. Unit Overview & Scope

- **Unit ID:** `OCRM-UNIT-MEM-001`
- **Module ID:** `OCRM-M08` (Memory, Vault & Knowledge Retrieval)
- **Status:** **Admitted into Parity Ledger; 6 source files mapped to reviewed; 6 responsibilities registered; 12 falsifiable test oracles verified in openclaw_memory_contract.rs; acceptance pending**
- **Requirements Traced:** `OCRM-R08`, `OCRM-AC08`
- **Reviewer:** `xuyao@ombhrum.com`
- **Upstream Baseline:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)

---

## 2. Source Files Mapped & Decomposed

The unit covers the core memory tool contracts, forget mechanics, citations formatting, candidate promotion, and standing intent boundaries of upstream OpenClaw (6 files, 1,754 total lines in shard `part-0011`):

| Upstream Source Path | Blob SHA | Lines | Language | Status | Disposition | Target Fabushi Rust Owner |
|---|---|---|---|---|---|---|
| `extensions/memory-core/src/memory-tool-contract.ts` | `fbe0a808c397455dac1d5f14e34907226b6b322b` | 181 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/runner/tools/sand_state_tool.rs` |
| `extensions/memory-core/src/memory-forget.ts` | `9b8a87c09dd561436c6977a9987451ec0bfc3deb` | 508 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/memory/memory_service.rs` |
| `extensions/memory-core/src/tools.citations.ts` | `adf027012af64e8d80768e0ae729d821f03a228e` | 61 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/content_search/agent_content_search.rs` |
| `extensions/memory-core/src/short-term-promotion.ts` | `144ada5c23cd886d101eef8511e3a0717f4de220` | 229 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/memory/memory_synthesis_service.rs` |
| `extensions/memory-core/src/standing-intents-tool.ts` | `9a1bd3f74d02020014a0d7578299f52693788e33` | 188 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/automations/` |
| `extensions/memory-core/src/tools.ts` | `a88465d7930f4bebb1a05f65429798ab0d857132` | 587 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/content_search/search_index_db.rs` |

---

## 3. Registered Responsibilities in Parity Ledger

1. **`OCRM-RESP-MEM-TOOL-CONTRACT-001`**: Memory tool contract, argument normalization, parameter aliasing (`maxResults` > `max_results` > `limit`), and safe project-scope comment formatting (`<!-- project: ${projectKey} -->`).
2. **`OCRM-RESP-MEM-FORGET-ATOMIC-001`**: Atomic memory fact scrubbing, session-referenced corpus line removal, and durable tombstone persistence against resurrection.
3. **`OCRM-RESP-MEM-CITATIONS-FORMAT-001`**: Memory search result presentation, citation formatting (`#L{start}` or `#L{start}-L{end}`), and auto-mode suppression on channel/group sessions.
4. **`OCRM-RESP-MEM-SHORT-TERM-PROMOTION-001`**: Multi-dimensional candidate ranking (frequency, relevance, diversity, recency, consolidation, conceptual) and dreaming phase signal boosting.
5. **`OCRM-RESP-MEM-STANDING-INTENTS-001`**: Standing intent creation, channel and sender scoping, keyword normalization, and expiration enforcement.
6. **`OCRM-RESP-MEM-SEARCH-CORPUS-ROUTING-001`**: Memory search corpus validation, closed enum enforcement (`memory`, `wiki`, `all`, `sessions`), cooldown tracking, and failure presentation.

---

## 4. Test Oracles & Conformance Evidence

Implemented and verified in `source/host/tests/openclaw_memory_contract.rs`:
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

CI Verification: Verified passing in GitHub Actions run 40276785979 on exact commit `03f548f22`.
