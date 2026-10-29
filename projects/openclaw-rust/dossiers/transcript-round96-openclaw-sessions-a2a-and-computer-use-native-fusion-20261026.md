# OCRM Technical Dossier: Round 96 — OpenClaw Sessions, A2A & Computer Use Native Fusion

## 1. Executive Summary

This dossier records the full-source native Rust equivalence and fusion for:
1. **`OCRM-M07`**: Sessions, History, Branches, Transcript & Agent-to-Agent Messaging (`src/sessions/`, `src/agents/sessions/`, `packages/agent-core/`)
2. **`OCRM-M17`**: Computer Use, Screen Automation, Input Devices & Accessibility (`src/computer/`, `packages/computer-core/`)

All responsibilities from upstream `openclaw/openclaw` in `src/sessions/`, `src/agents/sessions/`, and `src/computer/` are mapped natively into canonical Fabushi Rust architecture under `source/host/src/extensions/session/`, `source/host/src/extensions/transcript/`, and `source/host/src/runner/tools/sand_computer_tool.rs`.

---

## 2. Upstream Source to Canonical Fabushi Rust Mapping

### 2.1 Sessions, History & Agent-to-Agent Messaging (`OCRM-M07`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/sessions/store.ts` | Session state, KV storage & optimistic concurrency | `AgentDbProjection`, `read_kv`, `write_kv` | Host Session Store |
| `src/sessions/metadata.ts` | Session metadata, unread tracking & dividers | `initialize_metadata_if_missing`, `get_unread_state` | Host Session Store |
| `src/sessions/profile.ts` | Session profile & system prompt persistence | `SandProfile`, `get_sand_profile`, `set_sand_profile` | Host Session Store |
| `packages/agent-core/a2a.ts` | Agent-to-agent message dispatch & priority lanes | `AgentToAgentMessaging::send_to_agent`, `priority` | Host Transcript |
| `src/agents/sessions/interrupt.ts` | Priority peer interruption of active turns | `should_interrupt_priority_peer`, `RunLane` | Host Transcript |
| `src/agents/sessions/queue.ts` | Inbound message queue merging & FIFO preservation | `merge_agent_inbound_queue`, `prioritize_agent_inbound` | Host Transcript |

### 2.2 Computer Use, Screen Automation & Accessibility (`OCRM-M17`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/computer/actions.ts` | Computer action names & action serialization | `ComputerActionName` (`Screenshot`, `Click`, `Drag`, etc.) | Host Runner / Sand Computer |
| `src/computer/safety.ts` | Read-only bypass action classification | `is_sand_computer_auto_review_bypass_action` | Host Auto-Review |
| `src/computer/gate.ts` | Pointer mutating action declared description guard | `requires_sand_computer_declared_description` | Host Auto-Review |
| `src/computer/validation.ts` | Action parameter object normalization | `normalize_sand_computer_exact_action_args` | Host Auto-Review |
| `src/computer/page-state.ts` | Page target state deterministic SHA-256 fingerprint | `compute_sand_computer_page_state_identity` | Host Auto-Review |
| `src/computer/bounds.ts` | Execution timeout & character length bounds | `SAND_COMPUTER_MAX_WAIT_MS`, `MAX_TEXT_CHARS` | Host Runner / Sand Computer |

---

## 3. Falsifiable Test Oracles

1. **`OCRM-SESSION-O01` (A2A Message Partitioning)**: Validates partitioning inbound peer batches into priority and normal lanes.
2. **`OCRM-SESSION-O02` (Priority Interruption)**: Validates that urgent peer messages can interrupt standard turns while respecting critical non-interruptible states.
3. **`OCRM-SESSION-O03` (Queue Merging)**: Validates deterministic ordering when merging queued and deferred agent messages.
4. **`OCRM-SESSION-O04` (Session KV Storage)**: Validates transactional read, write, and delete operations on session KV entries.
5. **`OCRM-SESSION-O05` (Unread State Tracking)**: Validates unread message counters and viewed timestamp resets.
6. **`OCRM-SESSION-O06` (Session Profile Models)**: Validates JSON serialization and deserialization of session configurations.
7. **`OCRM-COMPUTER-O01` (Bypass vs Mutating Actions)**: Validates that `screenshot`, `move`, `wait`, and `scroll` are classified as bypass actions, while `click`, `doubleClick`, `drag`, `type`, and `key` are classified as mutating.
8. **`OCRM-COMPUTER-O02` (Declared Description Guard)**: Validates that pointer-altering actions (`click`, `doubleClick`, `drag`) require a declared description string.
9. **`OCRM-COMPUTER-O03` (Argument Object Normalization)**: Validates rejection of non-object action payloads.
10. **`OCRM-COMPUTER-O04` (Page State SHA-256 Fingerprint)**: Validates order-independent deterministic hashing of Chrome CDP page targets.
11. **`OCRM-COMPUTER-O05` (Timeout and Follow-Up Bounds)**: Validates `SAND_COMPUTER_MAX_WAIT_MS` (30,000 ms) and `SAND_COMPUTER_MAX_FOLLOW_UP_ACTIONS` (9).
12. **`OCRM-COMPUTER-O06` (Text Input Bounds)**: Validates `SAND_COMPUTER_AUTO_REVIEW_MAX_TEXT_CHARS` (2,000) and `SAND_COMPUTER_AUTO_REVIEW_MAX_KEY_CHARS` (256).
