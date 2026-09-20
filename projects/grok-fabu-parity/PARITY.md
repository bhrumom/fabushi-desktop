# Grok/Fabu -> Fabushi Desktop parity refactor

Reference snapshot: `bhrumom/fabu@939eb8e5e01d0d75ce5df2e46f96db8f835d5891`

Target branch: `refactor/fabu-agent-runtime-20260920`

The goal is behavioral and architectural parity with the recovered Grok Bot/Fabu desktop model while preserving Fabushi's product-owned Mahayana runtime and local-computer execution boundary. This document is a code-level inventory, not a claim of completed parity.

## Reference architecture inspected

The parity source is the accessible Fabu reconstruction:
- `docs/ARCHITECTURE.md`
- `frontend/src/production/ProductionRenderer.tsx`
- `frontend/src/production/model.ts`
- `frontend/src/production/sidebar-model.ts`
- `frontend/src/production/coordinator-client.ts`
- `frontend/src/recovered/features/conversation/workspace/sidebar.tsx`
- `frontend/src/recovered/features/conversation/workspace/submission.ts`
- `source/host/agents/agent-profile.ts`
- `source/host/agents/settings-file.ts`
- `source/host/agents/agent-messaging.ts`
- `source/host/agents/agent-worker-pool.ts`
- `source/host/agent-isolation/agent-store-worker.ts`
- `source/host/agent-isolation/conversation-blob-store.ts`
- `source/host/automations/automation-store.ts`

## Code-level difference inventory

| Area | Fabu/Grok behavior | Fabushi legacy behavior | Refactor state |
| --- | --- | --- | --- |
| Product shell | Agent-first sidebar + transcript + composer; secondary capabilities live behind overlays/menus | Telegram-like sections for Chats, Contacts, Bots, Groups, Channels, Calls, Saved, Payments and Mini Apps | IN PROGRESS: desktop now mounts through `AgentRootShell`; normal Agents mount `AgentWorkspace -> AgentTranscript -> GrokAgentComposer`; Messenger remains only as compatibility routing for non-Agent/Mini App surfaces |
| New chat | Creates a new Agent and opens it immediately | New button primarily creates groups/channels | IMPLEMENTED: visible New button issues `bot.create`, then opens the created Agent |
| Agent identity | Agent is the runtime/state owner; presentation identity is separate | Bot/conversation IDs were frequently reused as runtime identity | IMPLEMENTED: explicit Bot -> `agentId` mapping and runtime-ID routing |
| Sidebar state | Pinned/unpinned Agents, Working, waiting/attention, unread, custom sections, modifier multi-select and bulk organization | Generic peer list and messenger status | IN PROGRESS: pinned drag reorder, draft/waiting/current-activity priority, collapse/pin/hide/rename/duplicate/delete, account-scoped sections, Ctrl/Cmd multi-select, Shift range selection and bulk move/create/delete are implemented; host/cloud-backed section sync remains |
| Parallel Agents | One Agent may work while another Agent is opened or receives work | One renderer-global `agentOperationId` / pending request blocked unrelated Agents | IMPLEMENTED: `AgentOperationRegistry` owns request/operation state per Agent peer |
| Send lifecycle | pending/queued/failed/sent/cancelled is transport state; thinking/running is separate | UI pending state and runtime operation state were partially coupled | IMPLEMENTED: Fabu-style submission queue plus per-Agent runtime ownership; queued payloads preserve attachments and semantic reply context |
| Agent files | each Agent owns `profile.json`, `settings.json`, memory, automations, workflow enablement | mixed global/profile JSON and runtime-specific state | IMPLEMENTED for profile/settings, memory/workflows, per-Agent automation folders and clone semantics |
| Automation IDs | namespace is Agent directory + automation ID | global map could collide across Agents | IMPLEMENTED: runtime map is namespaced by Agent and disk layout is per-Agent |
| Agent clone | copies reusable Agent-owned state, not transcript/audit/attachments | clone was primarily Bot profile clone | IMPLEMENTED: memory, automations, workflow enablement copied; transcript/audit/attachments excluded |
| Inference session | independent session per Agent conversation | historically provider-global or active-chat-oriented state | IMPLEMENTED: session ID is conversation scoped; session snapshot path is Agent scoped |
| Account switching | durable Agent state survives account/session switching | reset path could destroy local transcript/session state | IMPLEMENTED: non-destructive history switching + account-scoped persistence |
| Cloud Agent store | immutable CAS blobs + mutable root/reference state | renderer cloud sync was profile-oriented | IN PROGRESS: profile/memory/workflow/automation mirror through account Agent Store; full local CAS graph sync remains |
| Coordinator boundary | narrow lifecycle + command/event protocol, reconnection, pending requests outside view components | `messaging-shell-v2.tsx` owns too many runtime, product and presentation concerns | IN PROGRESS: `AgentWorkspaceController` owns Agent-scoped request/draft/upload state; `AgentCoordinatorClient` owns commands plus Host subscription/initialize/close lifecycle; `AgentRuntimeCoordinator` exclusively owns normal Agent event attribution, command-bridge dispatch/accept/fail adoption, delta backpressure and terminal settlement. Explicit transport health/reconnect policy remains |
| Agent Composer | Agent-scoped draft/reply/attachments/drop/paste/voice; transport acceptance independent from Agent turn state | textarea plus Messenger attachment/reply behavior | IN PROGRESS: multi-file/drop/paste, attachment-only sends, restart-safe Agent draft recovery, semantic reply context and voice dictation through existing runtime/offline ASR are implemented; a full rich-text provider/mention model remains |
| Transcript projection | ordered rich entries for text, reasoning, tools, approvals, attachments and timeline state | mixed DisplayMessage / AssistantTurn / legacy message paths | IN PROGRESS: canonical `TranscriptEntry` + `AgentTranscript` are mounted for normal Agents; `AgentTranscriptStore` owns user turns, per-Agent source threads, request→operation rekey, AssistantTurn reduction and terminal settling, and normal Agent rendering no longer round-trips through renderer-global `messages`; approval/permission/computer-handoff emitters still need fully attributable Host events |
| Command palette | keyboard-first Agent navigation/actions | no equivalent primary Agent palette | IMPLEMENTED component and `Cmd/Ctrl+K` integration |
| Plugins / Computer | secondary overlays from Agent workspace | top-level messenger/product sections and profile panels | PARTIAL: Plugins/Settings moved to Agent footer; Computer is a primary Agent header action |
| Groups / Agent network | first-class Agent group/org semantics | group/channel semantics are primarily messenger/community based | PARTIAL: Agent groups exist, but org/network UX and Grok-style group coordination are not yet at parity |
| Legacy messenger | not the primary desktop product shell | dominates navigation and renderer architecture | TRANSITION: normal Agent header/transcript/composer no longer render through the Messenger path; non-Agent/Mini App compatibility UI and hidden legacy navigation still need final removal |

## 2026-09-20 Agent workspace extraction

This refactor slice is implemented on PR #7 without merging `main`.

- `desktop/src/agent-workspace/agent-root-shell.tsx`: primary desktop product boundary; compatibility Messenger surfaces no longer define root product identity.
- `desktop/src/agent-workspace/agent-workspace-controller.ts`: per-Agent request/operation ownership plus Agent-scoped text, attachments, reply context, upload state and atomic draft take/restore.
- `desktop/src/agent-workspace/agent-transcript-store.ts`: per-Agent transcript source ownership, provisional-request adoption, one-turn AssistantTurn reduction and terminal settlement outside the Messenger renderer.
- `desktop/src/agent-workspace/coordinator-client.ts`: narrow Mahayana command boundary for send/open/broadcast/interrupt and `attachment.upload -> attachment.stored`.
- `desktop/src/agent-workspace/agent-runtime-coordinator.ts`: single Agent event reducer for request→operation adoption, background Agent attribution, 16 ms delta coalescing, ordered finalization and per-Agent terminal settlement; it never infers ownership from the currently visible Agent.
- `desktop/src/agent-workspace/transcript-model.ts`: canonical `TranscriptEntry` domain replacing view-level dependence on three overlapping transcript shapes.
- `desktop/src/agent-workspace/agent-transcript.tsx`: primary ordered Agent timeline for messages, thinking, tools and AssistantTurn.
- `desktop/src/agent-workspace/agent-workspace.tsx`: Agent-owned composition of header, transcript, notices and composer.
- `desktop/src/agent-workspace/agent-attachments.ts`: six-file Agent attachment validation/upload preparation matching the existing Mahayana Host limits.
- `desktop/src/agent-workspace/agent-draft-store.ts`: restart-safe text and uploaded attachment draft recovery keyed by Agent compatibility peer.
- `desktop/src/agent-workspace/agent-sidebar-state.ts`: account-scoped custom sections, assignment and Unassigned projection.
- `desktop/src/agent-workspace/prompt-context.ts`: semantic reply context kept separate from visible user-message text.
- `desktop/src/grok-shell/grok-agent-composer.tsx`: multi-file/drop/paste, attachment pills/removal, attachment-only send, queued-send compatibility and voice dictation.
- `desktop/electron/main.cjs`: microphone permission is limited to trusted `app://bundle` audio-only capture; camera remains denied.
- `desktop/src/messaging-shell-v2.tsx`: normal Agent rendering reads directly from `AgentTranscriptStore` and Agent chat/operation events route through `AgentRuntimeCoordinator`; the previous renderer-owned delta buffer, request adoption helpers, operation clearing and Agent `setMessages` copy-back path were removed. Global `messages` now remains only for compatibility Messenger/group/Mini App surfaces.
- `desktop/e2e/grok-parity.spec.ts`: verifies one canonical Agent workspace, two-Agent operation/draft isolation, coordinator-owned concurrent A/B streams without cross-talk, draft preservation across runtime reset, atomic draft recovery, one canonical AssistantTurn through request→operation adoption, Agent attachment upload/timeline/attachment-only send and modifier selection + custom section creation.
- `desktop/electron/host-process.cjs` + `host-process.test.cjs`: development/CI Host resolution now supports the staged `desktop/resources/bin` generation without changing the packaged signed-resource boundary; this is covered by the Rust desktop runtime Actions renderer job.
- `desktop/scripts/check-agent-workspace-boundary.mjs`: CI-enforced architecture guard that rejects reintroduction of renderer-global Agent operation pointers, Agent delta buffers, Agent transcript copy-back through `messages`, or direct Host subscription ownership.

This is not a claim of full parity. A full rich-text provider, complete approval/permission/computer-handoff transcript emitters, final Computer workspace extraction, coordinator-owned reconnect/event families, complete CAS graph restoration and removal of the remaining compatibility Messenger DOM remain open gates.

## Refactor rule

Do not repeat the abandoned wholesale-source replacement. Keep Mahayana as the authoritative Rust Agent/runtime layer, but port the reference's domain boundaries and visible interaction model:
1. Agent owns runtime state, memory, automation, session and computer context.
2. Renderer owns projection and transient view interactions; Agent draft/attachment/reply state belongs to the Agent workspace controller.
3. Independent Agents never block one another.
4. Transport state never doubles as Agent reasoning state.
5. Telegram/social/payment/Mini App capabilities may remain in the product, but they must not distort the primary Grok-style Agent workspace.
6. Cloud synchronization follows Agent-owned roots and conflict-safe CAS semantics.
7. Legacy messenger code is removed only after equivalent backend capability has a non-messenger entry point or explicit compatibility adapter.

## Remaining parity gates

P0: finish primary shell migration, current CI, command palette, visible Agent selectors, two-Agent parallel-operation regression, hidden/rename/delete persistence.

P1: continue shrinking the monolithic compatibility renderer now that AgentRootShell, AgentWorkspaceController, AgentTranscriptStore, AgentSidebar, AgentHeader, Transcript, Composer and overlays are extracted; remove the remaining legacy-only rendering/event branches after their adapters are isolated.

P2: finish the remaining richer renderer parity: approval/permission/computer-handoff timeline emitters, full rich-text/provider mention editor, Agent settings/profile editor, and host-backed cross-device section state. Draft/waiting/activity, attachment timeline, pinned drag, custom sections and multi-select are now implemented.

P3: add explicit transport health/reconnect policy and recoverable handshake semantics behind `AgentCoordinatorClient`. Host subscription/initialize/close and normal Agent command/event adoption are already outside the React shell; non-Agent compatibility event families remain in the legacy adapter.

P4: complete local content-addressed Agent Store graph synchronization and cross-device root restoration; preserve conflict objects instead of last-write-wins.

P5: complete Agent-to-Agent/group/org workflow UX and packaged macOS acceptance against a pinned reference timeline.