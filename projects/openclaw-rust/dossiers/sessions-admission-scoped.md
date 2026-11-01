# OCRM Technical Dossier: Unit OCRM-UNIT-SESSIONS-001 — Session Admission, Keys & Turn Context Parity

## 1. Unit Overview & Scope

- **Unit ID:** `OCRM-UNIT-SESSIONS-001`
- **Module ID:** `OCRM-M07` (Sessions, History, Branches and Transcript)
- **Status:** **Admitted into Parity Ledger; 35 source files mapped to reviewed; 6 responsibilities registered; 6 falsifiable test oracles registered in openclaw_sessions_and_config_contract.rs (not executed in the current development freeze); acceptance pending**
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

The current implementation/test recheck is stricter: `session_profile_files.rs`
contains agent profile and avatar read/write helpers, not auth-profile
preservation or removal logic. The O05 Rust test only writes a temporary JSON
document and asserts that two strings can be read back; it does not invoke the
selected owner and does not exercise active-session retention, incompatible
provider clearing, stale-profile removal, or a producer/consumer lifecycle.
This is test-flaw/static fixture evidence, not an implementation slice. The
parity row therefore remains `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending`, with `implementation=not-started` until the real
auth-profile owner and lifecycle contract are implemented.

The model-override recheck found the same gap in a neighboring owner. The
existing `model_picker_service.rs` helper and session-state shape are now
owned by `agent_session.rs` through a narrow SQLite KV projection:
`sessionModelSelection` is read and written only for the requested agent id,
malformed or cross-session state is rejected, and only an `Applied` model-picker
result is persisted. This is the implementation slice for the session owner;
it does not mutate parent agent defaults.

The mutation-producer boundary remains open. The current checkout's later
reachability audit confirms that the existing Runner/Inference consumer reads
the persisted state and projects it into `SandSessionOptions`; the O04 test
still only round-trips `modelOverride` and `thinkingLevel` in a JSON value and
does not prove that runtime edge. The model override parity row is therefore
`implementation=implemented` for the owner/state slice while retaining
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending` until a
real authenticated session mutation route is connected and verified.

### Session model-override implementation contract (2026-10-31)

`SandAgentSessionStore::apply_session_model_selection` is the sole persistence
entrypoint for this slice. It opens the existing `SandAgentDb` owner, loads the
agent-scoped `sessionModelSelection` record, delegates policy/generation/runtime
checks to `model_picker_service::apply_session_model_selection`, and writes the
new state only for an `Applied` result. Rejected and conflict results leave the
previous state untouched. A persisted record whose `sessionId` differs from the
requested agent is a hard error. This contract is static implementation
evidence only; no local OCRM test/build was run under the development freeze.

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

The current caller recheck found no non-test call to
`plan_session_communication` or `resolve_session_communication_policy` in the
Host, Host app, Coordinator, Electron, or frontend sources. The planner is
therefore a pure owner-level contract slice; no inbound inter-session message
producer, approval-card consumer, or outbound transcript/channel consumer is
connected to it. The parity integration records both sides as absent while
retaining the existing pending statuses.

The locked `conversation-turns.ts` source was also re-read in full. The Rust
session conversation owner now contains a bounded pending-turn registry with
agent/id uniqueness, outbound-message reverse indexing, session/thread and
conversation matching, ready gating, claim/release/complete transitions, and
expiry pruning. O03 now exercises that registry rather than a JSON-only
simulation. The registry is still process-local and has no shipping inbound
producer or outbound transcript consumer, so its ledger wiring and verification
remain pending.

The registry is now retained by the existing `ProductionSessionWorkers` owner
and exposed through `SandAgentSessionStore`, so separately constructed session
facades share one process-lifetime pending-turn state instead of creating
parallel registries. This is owner-lifetime integration only; no inbound
channel dispatcher, outbound provider, or transcript publication path has been
invented, so production wiring remains pending.

The missing producer/consumer boundary is now explicit in the ledger as
`BL-SESSION-COMMUNICATION-PRODUCTION-WIRING-001` for O02 and
`BL-SESSION-CONVERSATION-TURN-PRODUCTION-WIRING-001` for O03. These blockers
describe absent production paths; they do not convert the existing contract
oracles into runtime acceptance.

## 8. Poll-intent oracle mapping correction (2026-10-29)

The locked source reference attached to
`OCRM-RESP-SESSION-ROUTING-DISPATCH-001` was read directly from the pinned
commit/tree: `src/poll-params.test.ts`, blob
`bf7918df646d0e185f97fc3551a35c06c6cdeafc`, 33 lines. It tests only
`hasPollCreationParams`: schema-padded duration/multi defaults must not create
poll intent, while real question/options fields must do so; snake-case duration
and public metadata alone remain non-intent.

This oracle does not test session routing keys, conversation cards, project
scope, or realtime transcription dispatch. The current Rust
`source/host/src/extensions/session/session_conversation_state.rs` owns
conversation/transcript state and pending-turn correlation, but the current
repository search found no Rust poll-intent owner or consumer for this source
contract. The ledger row is therefore a mapping mismatch that remains pending;
the 33-line read is retained as source evidence but must not be counted as
Session Routing equivalence or acceptance. No status flag was closed.

## 9. Session-state production entrypoint correction (2026-10-29)

The ledger entrypoint `get_conversation_state` does not exist in the current
tree. The real production boundary is
`source/host/src/extensions/session/production.rs::SandAgentSessionStore::read_agent_conversation_state`,
which delegates to `SessionConversationState::read_agent_conversation_state`
and resolves the persisted latest root blob through the existing agent-store
worker/blob owner. The contract fixture is
`source/host/tests/session_conversation_state_contract.rs`; it is not evidence
that the upstream poll-intent oracle is implemented. The ledger now records
this actual call chain while preserving `wiring=not-wired`, `verification=not-run`,
and the poll-intent mapping mismatch above.

## 10. Work-admission/archive production reachability recheck (2026-10-31)

The current `ProductionSessionWorkers` API was re-read and searched for the
work-admission, archive/restore, background-result, and review-terminal
operations represented by this responsibility. The worker is a real session
storage/materialization owner, but no corresponding admission or archive
transition producer/consumer was found in the Host, gateway, runner, Electron,
or frontend paths.

The O06 Rust test only constructs a JSON value and asserts `status == "active"`
and `archived == false`; it does not call `ProductionSessionWorkers` or prove
rejection of archived work, restore transitions, background completion capture,
or review settlement. This is a fixture/test-flaw, not implementation or
integration evidence. The parity row is therefore `implementation=not-started`,
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending` until the
real lifecycle contract and call chain exist.

## 11. Session-key classification production reachability recheck (2026-10-31)

`call_openclaw_sessions_list` now routes each canonical `agent:<id>:main`
session row through `resolve_gateway_session_route` before projecting its
kind. `sessions.get`, `sessions.delete`, `chat.send`, and `chat.abort` also
return the classified kind using their optional `chatType` and `spawnedBy`
metadata. The public Host app session and chat commands now reach
`resolve_gateway_session_route`, which delegates to the canonical
`resolve_public_session_route` owner and its `is_agent_harness_session_key`
guard, so the reserved `harness:` namespace policy is production-reachable.
This closes a real projection consumer but does not
prove complete cron/direct/group/global/spawn-child routing/admission across
every mutation path; the parity row therefore remains `wiring=not-wired`,
`verification=not-run`, and `acceptance=pending` until the classifier governs
the full public routing boundary.

## 11.1 Canonical public selector resolver (2026-11-01)

The existing `session_paths.rs` owner now exposes the pure
`resolve_public_session_route` boundary. It trims selectors, rejects the
reserved harness namespace, rejects malformed `agent:<empty>:...` selectors,
extracts the durable agent identity through `parse_agent_session_key`, and
classifies the route through the existing `classify_session_kind` function.

`sessions.list`, `sessions.get`, `sessions.delete`, `chat.send`, and `chat.abort`
now reach that resolver through the small Gateway adapter
`resolve_gateway_session_route`. Their existing session-worker, lifecycle,
transcript, and runner owners remain the only state/execution consumers.
Focused contract assertions cover canonical group routing, harness rejection,
malformed identity rejection, and static use by all five public projections.

The `sessions.list` projection now also routes each generated canonical
`agent:<id>:main` row through `resolve_gateway_session_route` before projecting
its kind, removing the remaining classifier-only public projection path. This
does not claim that cron/direct/group/global/spawn-child semantics are complete
across every mutation or channel ingress path. The parity row therefore remains
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending` until the
full routing/admission contract is implemented and the frozen exact-head
verification lane runs. The remaining full-boundary gap is recorded as
`BL-SESSION-KEY-ROUTING-FULL-WIRING-001`.

The same boundary now lowercases only the structural agent identity extracted
from canonical `agent:<id>:<rest>` selectors; opaque legacy selectors remain
unchanged. The undeletable root-main guard is owned by
`is_root_main_session_key`, which applies case-insensitive matching to
`global` and `agent:global:main` (including their structural wrapper) and is
reached by the production `sessions.delete` path. Contract coverage includes
`GLOBAL` and `AGENT:GLOBAL:MAIN`; this closes a case-normalization guard gap,
but does not change the parity disposition above.

## 11.3 Current adapter-to-owner reduction recheck — 2026-11-01

The exact current Host app path confirms that the five public projections do
invoke `resolve_gateway_session_route`, but it also shows why this is not yet
full session-key fusion. `sessions.get` resolves a route and then reads the
agent-wide transcript through `session_workers`; `chat.send` resolves a route
and passes only the resulting `agent_id` plus the raw key into the existing
`call_send_prompt` path; and `chat.abort` resolves a route but cancels runner
streams selected by agent-wide registry state. `sessions.delete` resolves a
route and then dispatches the existing `deleteAgent` lifecycle owner, whose
operation removes the whole agent directory. `sessions.patch` rejects every
key other than `agent:<id>:main` before invoking the existing archive owner.

This is positive evidence for selector validation and negative evidence for
per-kind/per-run state ownership. A cron, group, spawn-child, or other
non-main key is not yet a durable transcript/session target merely because
`classify_session_kind` labels it. The remaining implementation boundary is
therefore a source/spec task for a session catalog and per-key transcript,
archive, abort, and admission owners; it cannot be closed by changing the
adapter's return type or by routing non-main keys to whole-agent deletion.
The parity row remains `implementation=implemented`, `wiring=not-wired`,
`verification=not-run`, and `acceptance=pending`.

## 11.2 Internal sendPrompt boundary recheck — 2026-11-01

The remaining full-wiring gap was checked against the actual dispatch graph.
`call_openclaw_chat_send` resolves the public selector and then passes the
resolved `agent_id` to the existing `call_send_prompt` owner. The latter is
also invoked directly by trusted internal producers such as automation,
upgrade-resume, acknowledgement redrive, and group-member dispatch. Those
internal calls intentionally carry a canonical agent identity rather than a
public session selector; they are not additional public routing authorities.

This confirms the current boundary but does not prove the missing
cron/direct/group/global/spawn-child mutation semantics: the internal
`sendPrompt` path does not independently carry or re-resolve `SessionKind`,
and channel/automation producers do not all enter through the public
`resolve_gateway_session_route` adapter. Promoting the ledger row would
therefore conflate a resolved public projection with full mutation admission.
The next implementation instrument is a trusted internal session-admission
API that accepts the canonical route plus producer context and fences the
existing send/runner owner consistently across those producers; no second
session store or public parser is authorized.

## 11.3 Trusted internal session-admission implementation slice — 2026-11-01

The existing session-path owner now exposes `resolve_internal_send_route`.
`call_send_prompt` and `ProductionTranscriptRuntime::accept_routed_send` use
that same resolver before durable acceptance or Runner dispatch whenever a
trusted producer supplies a selector or requires one. The selector is checked
against the supplied `agentId`; an absent selector is rejected for automation,
group-member, upgrade-resume, acknowledgement redrive, and subagent sources.
An explicitly declared `directMainFallback` is the only route that may
synthesize `agent:<id>:main`.

The production producer edges now carry selectors: automation uses
`agent:<id>:cron:<automationId>`, group-member uses the existing room context
under `:group:`, upgrade-resume preserves a cron selector when its marker has
an automation id and otherwise uses the direct main selector, acknowledgement
redrive uses direct main, and subagent sends carry `spawnedBy` so the canonical
route is classified as `SpawnChild`. `chat.send` forwards its resolved public
selector, `chatType`, and `spawnedBy` into the existing send owner.

This is source-level implementation and contract evidence only. The broader
producer graph still requires exact-head verification of every channel and
background path; no local OCRM build/test or CI was run, and the parity row
remains `verification=not-run` and `acceptance=pending`.

## 12. Session model-override production consumer (2026-10-31)

The inference consumer edge is now explicit. The existing
`SandAgentSessionStore::apply_session_model_selection` remains the only writer
of the agent-scoped `sessionModelSelection` KV. `runner.startRoutedProvider`
reads that same record before turn composition, rejects malformed or
cross-agent state through the existing reader, and projects model/thinking
values into the one `SandSessionOptions` passed to
`ProductionInferenceExtension::cursor_auth_for_session`. The resulting
`RequestedModel` is consumed by the existing Cursor inference transport and
`HostRunnerComposition` turn; no second session or inference owner is added.

This does not yet close the full producer→owner→state→consumer contract: no
production gateway/UI mutation caller for `apply_session_model_selection` was
found in the current source. The parity row therefore remains
`wiring=not-wired` until that mutation route is connected with its existing
visibility/operator/runtime fences.

The same reachability audit is now recorded explicitly in the ledger as
`BL-SESSION-MODEL-MUTATION-WIRING-001`; it is a missing production mutation
producer, not a failed model-picker policy implementation.

## 12.1 Fixed-source classification correction — 2026-11-01

The fixed upstream `classifySessionKind` checks `entry?.spawnedBy` by
truthiness. The Rust classifier previously used `spawned_by.is_some()`, which
misclassified an explicitly supplied empty string as `SpawnChild`. The active
session routing spec now records this as `SKR-07`; the existing session-path
owner uses a non-empty check, and the contract oracle includes empty-string
spawn ancestry with both direct and group fallbacks. This is a local semantic
correction only; it does not promote the broader routing responsibility to
production acceptance or runtime verification.

The inference contract oracle now covers the positive precedence case and the
negative parent-default invariant: a persisted session override selects its
own model and thinking parameter while the stored agent default remains
unchanged. This is implementation and wiring evidence only. No local OCRM
build/test or packaged acceptance was run in the development phase, so the
ledger remains `verification=not-run` and `acceptance=pending` until the
frozen exact-head Actions lane executes the required unit, integration and
negative checks.

## 12.2 Fixed-source Cron classification correction — 2026-11-01

The fixed upstream `isCronSessionKey` first parses the canonical
`agent:<agent-id>:<rest>` wrapper and then checks only whether the parsed
`rest` starts with `cron:`. The Rust classifier previously accepted bare
`cron:*` keys and arbitrary embedded `:cron:` tokens, which was broader than
the fixed source. The session-path owner now uses the canonical parser before
the Cron check; the contract oracle covers a valid agent-scoped Cron key and
negative bare/embedded-token cases. This remains a local semantic correction;
no production wiring, build/test execution, or acceptance status is claimed.

## 12.3 Fixed-source agent identity normalization correction — 2026-11-01

The fixed upstream `parseAgentSessionKey` first applies
`normalizeSessionKeyPreservingOpaquePeerIds`; ordinary structural portions of
`AGENT:Agent-123:Main` therefore resolve to the canonical agent identity
`agent-123`, while provider-owned opaque peer spans retain their bytes. The
Rust public resolver previously preserved the wrapped agent identity's case.
It now lowercases only the structured agent identity after the existing
parser, leaving legacy raw selectors opaque. Contract oracles cover the mixed
case canonical route. This is a local boundary correction only; production
wiring, verification, and acceptance remain pending.

## 12.5 Channel inbound provenance correction — 2026-11-01

`append_channel_inbound_entries` now resolves the session identity from the
existing `ProductionSessionWorkers::read_session_work_admission` record and
fails closed when no non-empty canonical key is available. It no longer
synthesizes `agent:<id>:main`. Each materialized channel inbound transcript
entry receives that same `sessionKey` before the existing admitted append
transaction.

This corrects the producer/owner contract for the wired channel inbound slice:
the previous producer supplied an agent-derived key to the admission check but
omitted entry provenance, so the owner rejected the append. It does not prove
that every channel type has a session catalog or that provider delivery,
thread correlation, reconnect, and runtime acceptance are complete.

## 12.6 Background revival identity correction — 2026-11-01

`run_local_background_revival_turn_with_context` now obtains its session key
from the existing owner admission record before capturing lifecycle/work
admission. If no non-empty record is available, the revival fails closed; it
does not synthesize `agent:<id>:main`. The captured key is then carried into
the routed send and Runner arguments as before.

This removes an agent-only identity guess at the background producer boundary.
It does not close the broader detached-result lifecycle, per-kind session
catalog, or runtime acceptance requirements.

## 12.4 Owner-bound remote Human transcript append — 2026-11-01

`ProductionSessionWorkers::materialize_remote_human_message` now reads the
existing `SandAgentDb` session-work admission record before materializing a new
remote Human message. When that record contains a non-empty canonical
`session_key`, the projected entry carries the same `sessionKey` and is
appended through `append_transcript_entries_if_session_admitted`, preserving
the owner transaction's archive/rebind fence. If no admission record exists,
the migration-compatible generic append remains available.

This is a narrow producer-to-owner provenance slice. It does not infer a
missing channel conversation identity, rewrite legacy Human rows that lack
provenance, bind outbound pending Human sends or identity settlement, or prove
channel delivery/receipt behavior. Full channel-to-session correlation,
verification, and acceptance remain pending.

## 12.7 Background producer selector correction — 2026-11-01

Upgrade-resume (without an automation id), kickstart, and
`runner.startRoutedProvider` now resolve a missing session selector from the
existing owner admission record. They fail closed when no non-empty canonical
record is available. Upgrade-resume entries with an automation id retain the
producer-derived canonical `agent:<id>:cron:<automationId>` selector and still
pass the existing owner admission check.

This removes four remaining agent-only main-key guesses from background and
Runner producer paths. Acknowledgement redrive and newly materialized child
session/public direct-main projection paths remain separately governed and are
not promoted by this slice.

## 12.8 Acknowledgement redrive identity correction — 2026-11-01

The production acknowledgement-redrive worker now reads the existing owner
admission record before constructing its hidden `sendPrompt` payload and passes
that canonical key into `build_ack_redrive_send_args`. Missing or unreadable
admission identity skips the redrive rather than synthesizing a main-session
key. The builder no longer owns or invents session identity; its contract
requires the producer-supplied key.

The focused contract fixture was updated to pass an explicit canonical test
key. This is static producer wiring evidence only; redrive runtime,
conversation-turn settlement, and exact-head acceptance remain pending.

## 12.9 Human handoff result session provenance — 2026-11-01

The authenticated `runner.startRoutedProvider` producer already carries the
validated `worker_session_key` into the terminal Human handoff result path.
That key is now passed to `ProductionSessionWorkers::append_human_agent_message`
and persisted as `sessionKey` in the existing Human conversation owner. A
same-stream replay preserves the bound identity, upgrades a legacy matching
row with the producer-owned provenance, and rejects a replay that supplies a
different session key.

This closes the producer-owned provenance edge for Agent results written into
the separate Human conversation store without deriving a key from
`conversation_id` or making that store a session-admission owner. Human
pending-send settlement, remote delivery correlation, conversation-turn
registry production wiring, and runtime/exact-head acceptance remain pending.
