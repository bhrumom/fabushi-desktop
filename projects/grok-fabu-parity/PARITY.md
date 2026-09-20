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
| Product shell | Agent-first sidebar + transcript + composer; secondary capabilities live behind overlays/menus | Telegram-like sections for Chats, Contacts, Bots, Groups, Channels, Calls, Saved, Payments and Mini Apps | IN PROGRESS: normal Agents now mount `AgentWorkspace -> AgentTranscript -> GrokAgentComposer`; Messenger remains only as compatibility routing for non-Agent/Mini App surfaces |
| New chat | Creates a new Agent and opens it immediately | New button primarily creates groups/channels | IMPLEMENTED: visible New button issues `bot.create`, then opens the created Agent |
| Agent identity | Agent is the runtime/state owner; presentation identity is separate | Bot/conversation IDs were frequently reused as runtime identity | IMPLEMENTED: explicit Bot -> `agentId` mapping and runtime-ID routing |
| Sidebar state | Pinned/unpinned Agents, Working, unread/attention, hidden Agents, collapse, rename/duplicate/delete | Generic peer list and messenger status | IN PROGRESS: pinned drag reorder, Working/unread/collapse/pin/hide/rename/duplicate/delete plus draft/waiting/current-activity projection implemented; custom sections/multi-select batch move remain |
| Parallel Agents | One Agent may work while another Agent is opened or receives work | One renderer-global `agentOperationId` / pending request blocked unrelated Agents | IMPLEMENTED: `AgentOperationRegistry` owns request/operation state per Agent peer |
| Send lifecycle | pending/queued/failed/sent/cancelled is transport state; thinking/running is separate | UI pending state and runtime operation state were partially coupled | IMPLEMENTED: Fabu-style submission queue plus per-Agent runtime ownership |
| Agent files | each Agent owns `profile.json`, `settings.json`, memory, automations, workflow enablement | mixed global/profile JSON and runtime-specific state | IMPLEMENTED for profile/settings, memory/workflows, per-Agent automation folders and clone semantics |
| Automation IDs | namespace is Agent directory + automation ID | global map could collide across Agents | IMPLEMENTED: runtime map is namespaced by Agent and disk layout is per-Agent |
| Agent clone | copies reusable Agent-owned state, not transcript/audit/attachments | clone was primarily Bot profile clone | IMPLEMENTED: memory, automations, workflow enablement copied; transcript/audit/attachments excluded |
| Inference session | independent session per Agent conversation | historically provider-global or active-chat-oriented state | IMPLEMENTED: session ID is conversation scoped; session snapshot path is Agent scoped |
| Account switching | durable Agent state survives account/session switching | reset path could destroy local transcript/session state | IMPLEMENTED: non-destructive history switching + account-scoped persistence |
| Cloud Agent store | immutable CAS blobs + mutable root/reference state | renderer cloud sync was profile-oriented | IN PROGRESS: profile/memory/workflow/automation mirror through account Agent Store; full local CAS graph sync remains |
| Coordinator boundary | narrow lifecycle + command/event protocol, reconnection, pending requests outside view components | `messaging-shell-v2.tsx` owns too many runtime, product and presentation concerns | IN PROGRESS: `AgentWorkspaceController` now owns request/operation adoption and draft projection; `AgentCoordinatorClient` owns send/open/broadcast/interrupt/attachment upload; reconnect/event-family extraction remains |
| Transcript projection | ordered rich entries for text, reasoning, tools, approvals, attachments and timeline state | mixed DisplayMessage / AssistantTurn / legacy message paths | IN PROGRESS: canonical `TranscriptEntry` adapter and `AgentTranscript` are mounted for normal Agents; message/thinking/tool/AssistantTurn now share one timeline boundary; approval/permission/computer-handoff emitters still need full projection |
| Command palette | keyboard-first Agent navigation/actions | no equivalent primary Agent palette | IMPLEMENTED component and `Cmd/Ctrl+K` integration |
| Plugins / Computer | secondary overlays from Agent workspace | top-level messenger/product sections and profile panels | PARTIAL: Plugins/Settings moved to Agent footer; Computer is a primary Agent header action |
| Groups / Agent network | first-class Agent group/org semantics | group/channel semantics are primarily messenger/community based | PARTIAL: Agent groups exist, but org/network UX and Grok-style group coordination are not yet at parity |
| Legacy messenger | not the primary desktop product shell | dominates navigation and renderer architecture | TRANSITION: normal Agent header/transcript/composer no longer render through the Messenger path; non-Agent/Mini App compatibility UI and hidden legacy navigation still need final removal |

## 2026-09-20 Agent workspace extraction

This refactor slice is implemented on PR #7 without merging `main`.

- `desktop/src/agent-workspace/agent-workspace-controller.ts`: per-Agent request/operation ownership plus Agent-scoped draft projection.
- `desktop/src/agent-workspace/coordinator-client.ts`: narrow Mahayana boundary for send/open/broadcast/interrupt and `attachment.upload -> attachment.stored`.
- `desktop/src/agent-workspace/transcript-model.ts`: canonical `TranscriptEntry` domain replacing view-level dependence on three overlapping transcript shapes.
- `desktop/src/agent-workspace/agent-transcript.tsx`: primary ordered Agent timeline for messages, thinking, tools and AssistantTurn.
- `desktop/src/agent-workspace/agent-workspace.tsx`: Agent-owned composition of header, transcript, notices and composer.
- `desktop/src/agent-workspace/agent-attachments.ts`: six-file Agent attachment validation/upload preparation matching the existing Mahayana Host limits.
- `desktop/src/agent-workspace/agent-draft-store.ts`: restart-safe text and uploaded attachment draft recovery keyed by Agent compatibility peer.
- `desktop/src/grok-shell/grok-agent-composer.tsx`: multi-file input, drag/drop, attachment pills/removal, queued-send compatible behavior.
- `desktop/src/messaging-shell-v2.tsx`: normal Agent rendering and lifecycle calls route through the new workspace/controller/coordinator; Messenger attachment sending is no longer used by normal Agents.
- `desktop/e2e/grok-parity.spec.ts`: verifies exactly one canonical Agent composer/message list and an Agent-owned attachment upload/send flow.

This is not a claim of full parity. Custom sidebar sections/multi-select, full approval/permission/computer transcript projection, final Computer workspace extraction, complete CAS graph restoration and removal of the remaining compatibility Messenger DOM remain open gates.

## Refactor rule

Do not repeat the abandoned wholesale-source replacement. Keep Mahayana as the authoritative Rust Agent/runtime layer, but port the reference's domain boundaries and visible interaction model:
1. Agent owns runtime state, memory, automation, session and computer context.
2. Renderer owns only projection, local composer state and view interactions.
3. Independent Agents never block one another.
4. Transport state never doubles as Agent reasoning state.
5. Telegram/social/payment/Mini App capabilities may remain in the product, but they must not distort the primary Grok-style Agent workspace.
6. Cloud synchronization follows Agent-owned roots and conflict-safe CAS semantics.
7. Legacy messenger code is removed only after equivalent backend capability has a non-messenger entry point or explicit compatibility adapter.

## Remaining parity gates

P0: finish primary shell migration, current CI, command palette, visible Agent selectors, two-Agent parallel-operation regression, hidden/rename/delete persistence.

P1: extract the monolithic messenger renderer into AgentWorkspaceController + AgentSidebar + AgentHeader + Transcript + Composer + overlays; retire hidden legacy sidebar DOM.

P2: port the richer renderer Agent model (draft, waiting reason, last entry/message, running state, group metadata), ordered transcript projection, tool/approval timeline, Agent settings/profile editor and sidebar drag/reorder/sections.

P3: move remaining renderer/Host lifecycle work behind a Fabu-style coordinator client, including reconnect, pending request adoption, lifecycle handshake and event families.

P4: complete local content-addressed Agent Store graph synchronization and cross-device root restoration; preserve conflict objects instead of last-write-wins.

P5: complete Agent-to-Agent/group/org workflow UX and packaged macOS acceptance against a pinned reference timeline.