# OCRM Technical Dossier: Unit OCRM-UNIT-SESSIONS-001 — Session Admission, Keys & Turn Context Parity

## 1. Unit Overview & Scope

- **Unit ID:** `OCRM-UNIT-SESSIONS-001`
- **Module ID:** `OCRM-M07` (Sessions, History, Branches and Transcript)
- **Status:** **Admitted into Parity Ledger; 35 source files mapped to reviewed; 6 responsibilities registered; 6 falsifiable test oracles implemented in openclaw_sessions_and_config_contract.rs; acceptance pending**
- **Requirements Traced:** `OCRM-R04`, `OCRM-R08`, `OCRM-AC08`
- **Reviewer:** `xuyao@ombhrum.com`
- **Upstream Baseline:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)

---

## 2. Source Files Mapped & Decomposed

The unit covers the complete session key routing, communication policy, turn settlement, model override, and admission domain in `src/sessions/` (part-0045, 35 files, 4,685 lines):

| Upstream Source Path | Blob SHA | Lines | Language | Status | Disposition | Target Fabushi Rust Owner |
|---|---|---|---|---|---|---|
| `src/sessions/agent-harness-session-key.ts` | `374674a13a484c0dc608f1d641f3e438d33fef44` | 261 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_paths.rs` |
| `src/sessions/agent-harness-session-key.test.ts` | `13db1c3dfa6af7403402dbf8e6b196ff731e70b0` | 129 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/auth-profile-preservation.ts` | `5d93e81b6f237a0783e4b3163fbd3a365cf143b7` | 130 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_profile_files.rs` |
| `src/sessions/auth-profile-preservation.test.ts` | `b702853690bbae7b92c5d1b1d0be5eafcb2057c5` | 195 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/auth-profile-removal.ts` | `378579dd13534027cdce7953942e64822db097e9` | 157 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_profile_files.rs` |
| `src/sessions/background-session-result.ts` | `a70b7b04de18bdba5216ece63e50928cd2e6eea2` | 204 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/production.rs` |
| `src/sessions/background-session-result.test.ts` | `0238674617882d7b5eb8f7b7401a4ed54e841a60` | 358 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/classify-session-kind.ts` | `77eb2557be7101ab4ec76216c958f988367c4b97` | 29 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_paths.rs` |
| `src/sessions/communication-admission.ts` | `72832202551cdcf5884005ac700baa2da30043af` | 89 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/agent_session.rs` |
| `src/sessions/communication-policy.ts` | `5f795818204de25bdd6fc3f20c2651d1e4544d86` | 22 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/agent_session.rs` |
| `src/sessions/communication-policy.test.ts` | `2ccac74416ffe9158c6eada57bb448161340e258` | 42 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/controller-core.test-support.ts` | `536522936a93aed8117df0f58efadaf10c817c56` | 39 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/conversation-turns.ts` | `9e3202483196953131f9f95b7e54c236ae4803f4` | 267 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_conversation_state.rs` |
| `src/sessions/conversation-turns.test.ts` | `6a48c9257ffa1dca8159831bd23ec23244a3c55e` | 307 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/input-provenance.ts` | `a113a1dfc2538927c1a35b1fef1ed866f9bd6844` | 291 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_conversation_state.rs` |
| `src/sessions/input-provenance.test.ts` | `572d1e202a3748e9b1672492c541375c0fcc84d3` | 110 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/level-overrides.ts` | `756a1d79812c4ebec31df4b851c3ca02b470d805` | 62 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/agent_session.rs` |
| `src/sessions/model-overrides.ts` | `a8ee87b3130d380fc2123f2687739ac0302a08f9` | 190 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/agent_session.rs` |
| `src/sessions/model-overrides.test.ts` | `9a6e5f80bce59943227cdd106867aa003a6acce4` | 252 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/model-selection-error.ts` | `dd86f7b59cf6766ff0c5cf29b2695b41711b2387` | 9 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/agent_session.rs` |
| `src/sessions/nested-tool-activity.ts` | `a3ebff0889e985ffbb120034d112fe2bd3641217` | 107 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_conversation_state.rs` |
| `src/sessions/provider-review-terminal.ts` | `12dfedf6fe3546cd3039f0e43fd448c7ffe29844` | 90 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/production.rs` |
| `src/sessions/provider-review.ts` | `28a1a20adda8a961b9d17ec2d2139f71906cf134` | 372 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/production.rs` |
| `src/sessions/provider-review.test.ts` | `55e3154ee96307a25abbe427113f7e6b3d6a8590` | 342 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/reply-operation-state.ts` | `399191e1d5fe0b9e101645c16e42314c19dad5a2` | 125 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_conversation_state.rs` |
| `src/sessions/runtime-context.ts` | `e8a38864c303736818b0ca759068d2fca2ed9b81` | 38 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/production.rs` |
| `src/sessions/send-policy.ts` | `347e43321c960a348e4c7b2bf1d23b663f0c93b6` | 129 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/agent_session.rs` |
| `src/sessions/send-policy.test.ts` | `5e338013d771d54a340a13c99f1042aa216d0840` | 202 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/session-admission.test-support.ts` | `d0f3737586be0bce6c38a5dfd593cf796a59da50` | 14 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/session-agent-status.ts` | `fe9de8f871b514d62e822b2ca6404a98e2173351` | 49 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/production.rs` |
| `src/sessions/session-agent-work-admission.ts` | `7da9e10faaec1bf94044830f7d39d9f65ef3d68a` | 142 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/production.rs` |
| `src/sessions/session-agent-work-admission.test.ts` | `9dbbfa06440dabe644bbe771a9c9f78d0f36f6b0` | 207 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| `src/sessions/session-archive-restore.ts` | `45972689946f442e89561da1a26f807b3432b694` | 136 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/production.rs` |
| `src/sessions/session-chat-type-shared.ts` | `8a5bc75709eb6d2aeef405eba91ce759baacfe4a` | 118 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_paths.rs` |
| `src/sessions/session-chat-type.ts` | `074ce765945f30af711aa61f711c14715be4db1a` | 30 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/session/session_paths.rs` |

---

## 3. Registered Responsibilities in Parity Ledger

### Locked-source file audit (2026-10-28)

`src/sessions/auth-profile-preservation.ts` (blob
`5d93e81b6f237a0783e4b3163fbd3a365cf143b7`, 130 lines) and its complete
`auth-profile-preservation.test.ts` oracle (blob
`b702853690bbae7b92c5d1b1d0be5eafcb2057c5`, 195 lines) were read from the
locked authority checkout and recorded as full-range rows in the canonical
inventory. The implementation resolves the effective provider profile from
runtime/persisted/config sources, preserves a compatible pinned profile, and
clears an incompatible cross-provider override. The test source covers
preservation and pruning decisions. Fabushi's existing session profile owner
does not yet have a production caller proving this lifecycle, so
`OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001` remains `not-wired`/`not-run`;
the read is source-closure evidence only, not a production acceptance claim.

The same pass read `src/sessions/auth-profile-removal.ts` in full (blob
`378579dd13534027cdce7953942e64822db097e9`, 157 lines). It is mapped to the
same lifecycle responsibility as the preservation source, while its removal
behavior remains source evidence only; no separate test file exists at the
locked tree and no production caller was inferred.

The pass also completed the five primary session sources already named by the
ledger: `agent-harness-session-key.ts` (blob
`374674a13a484c0dc608f1d641f3e438d33fef44`, 261 lines),
`background-session-result.ts` (`a70b7b04de18bdba5216ece63e50928cd2e6eea2`,
204 lines), `communication-admission.ts`
(`72832202551cdcf5884005ac700baa2da30043af`, 89 lines),
`conversation-turns.ts` (`9e3202483196953131f9f95b7e54c236ae4803f4`, 267
lines), and `model-overrides.ts`
(`a8ee87b3130d380fc2123f2687739ac0302a08f9`, 190 lines). Each full-range
row is mapped to its existing responsibility and no implementation or
verification status was upgraded by this source read.

1. **`OCRM-RESP-SESSION-KEY-ROUTING-001`**: Classify and route session keys (cron, direct, group, global, spawn-child), reserving `harness:` namespace.
2. **`OCRM-RESP-SESSION-COMMUNICATION-POLICY-001`**: Inter-session communication admission policy (modes: always, ask, never) with approval requirements.
3. **`OCRM-RESP-SESSION-CONVERSATION-TURNS-001`**: Conversation turn correlation, pending turn registry, outbound message matching, and turn settlement state machine.
4. **`OCRM-RESP-SESSION-MODEL-OVERRIDES-001`**: Scoped per-session model overrides and thinking levels without mutating global agent configuration.
5. **`OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001`**: Preserving and pruning auth profiles associated with active and terminated sessions.
6. **`OCRM-RESP-SESSION-WORK-ADMISSION-001`**: Session work admission control, background session completion capture, archive/restore lifecycle, and review terminal gating.

---

## 4. Test Oracles & Conformance Evidence

Implemented in `source/host/tests/openclaw_sessions_and_config_contract.rs`:
- **`OCRM-SESSION-O01`**: `test_ocrm_session_o01_session_key_routing`
- **`OCRM-SESSION-O02`**: `test_ocrm_session_o02_communication_policy`
- **`OCRM-SESSION-O03`**: `test_ocrm_session_o03_conversation_turn_correlation`
- **`OCRM-SESSION-O04`**: `test_ocrm_session_o04_model_overrides`
- **`OCRM-SESSION-O05`**: `test_ocrm_session_o05_auth_profile_preservation`
- **`OCRM-SESSION-O06`**: `test_ocrm_session_o06_work_admission_and_archive`

## 5. Communication policy owner boundary (2026-10-29)

The locked `communication-policy.ts` and `communication-admission.ts` sources
were re-read at the pinned commit. The existing Rust session owner now exposes
the corresponding independent send/receive resolution and admission planner:
owned-task and same-session bypass, `never` denial, and ordered `ask`
approvals for source-send then target-receive. The O02 contract calls these
owner functions directly instead of an inline policy lambda. This remains
`not-wired` until a real session communication producer and consumer use the
planner; no production wiring or acceptance status is upgraded here.
