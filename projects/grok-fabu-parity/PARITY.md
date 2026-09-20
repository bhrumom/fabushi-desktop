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
| Sidebar state | Pinned/unpinned Agents, Working, waiting/attention, unread, custom sections, modifier multi-select and bulk organization | Generic peer list and messenger status | IN PROGRESS: pinned drag reorder, draft/waiting/current-activity priority, collapse/pin/hide/rename/duplicate/delete, account-scoped sections, Ctrl/Cmd multi-select, Shift range selection and bulk move/create/delete are implemented. Section state and pinned order now hydrate/persist through the Native Host with localStorage as fallback; true cross-device sync still requires an account-scoped preference/CAS API that does not exist in the current control plane. |
| Parallel Agents | One Agent may work while another Agent is opened or receives work | One renderer-global `agentOperationId` / pending request blocked unrelated Agents | IMPLEMENTED: `AgentOperationRegistry` owns request/operation state per Agent peer |
| Send lifecycle | pending/queued/failed/sent/cancelled is transport state; thinking/running is separate | UI pending state and runtime operation state were partially coupled | IMPLEMENTED: Fabu-style submission queue plus per-Agent runtime ownership; queued payloads preserve attachments/reply context and are projected into the same `AgentTranscriptStore` instead of a renderer-global queued-message map |
| Agent files | each Agent owns `profile.json`, `settings.json`, memory, automations, workflow enablement | mixed global/profile JSON and runtime-specific state | IMPLEMENTED for profile/settings, memory/workflows, per-Agent automation folders and clone semantics |
| Automation IDs | namespace is Agent directory + automation ID | global map could collide across Agents | IMPLEMENTED: runtime map is namespaced by Agent and disk layout is per-Agent |
| Agent clone | copies reusable Agent-owned state, not transcript/audit/attachments | clone was primarily Bot profile clone | IMPLEMENTED: memory, automations, workflow enablement copied; transcript/audit/attachments excluded |
| Inference session | independent session per Agent conversation | historically provider-global or active-chat-oriented state | IMPLEMENTED: session ID is conversation scoped; session snapshot path is Agent scoped |
| Account switching | durable Agent state survives account/session switching | reset path could destroy local transcript/session state | IMPLEMENTED: non-destructive history switching + account-scoped persistence |
| Cloud Agent store | immutable CAS blobs + mutable root/reference state | renderer cloud sync was profile-oriented | IMPLEMENTED for the primary Agent root: `root.json` is validated, every referenced object is materialized against its blob/etag/revision metadata, and transcript + attachment index + runtime checkpoint recover from the same verified snapshot. A checkpoint that was still running on another device becomes an interrupted recovery notice instead of being falsely adopted as a local active operation. |
| Coordinator boundary | narrow lifecycle + command/event protocol, reconnection, pending requests outside view components | `messaging-shell-v2.tsx` owns too many runtime, product and presentation concerns | IMPLEMENTED for the primary Agent path: Electron Host generation lifecycle is projected to the renderer; `AgentCoordinatorClient` owns bounded reconnect/handshake state; `AgentRuntimeCoordinator` interrupts stale operations without clearing Agent drafts/history and recovered generations reopen current settings/list/conversation state |
| Agent Composer | Agent-scoped draft/reply/attachments/drop/paste/voice; transport acceptance independent from Agent turn state | textarea plus Messenger attachment/reply behavior | IN PROGRESS: multi-file/drop/paste, attachment-only sends, restart-safe Agent draft recovery, semantic reply context and voice dictation through existing runtime/offline ASR are implemented. `@Agent` selections now persist a stable Agent id in the Agent-owned draft, survive queued-send failure/recovery, and are supplied to the Coordinator as structured prompt references. Full Fabu TipTap/provider/workflow rich-document parity remains. |
| Transcript projection | ordered rich entries for text, reasoning, tools, approvals, attachments and timeline state | mixed DisplayMessage / AssistantTurn / legacy message paths | IN PROGRESS: canonical `TranscriptEntry` + `AgentTranscript` are mounted for normal Agents; `AgentTranscriptStore` owns user turns, per-Agent source threads, request→operation rekey, AssistantTurn reduction and terminal settling, and normal Agent rendering no longer round-trips through renderer-global `messages`; approval emitters are operation/Agent-attributed end-to-end and render as interactive timeline cards; scoped Computer snapshot/action results now carry `agentId` and render as `computer-handoff` entries, while unscoped local/remote events remain in compatibility surfaces |
| Command palette | keyboard-first Agent navigation/actions | no equivalent primary Agent palette | IMPLEMENTED: `Cmd/Ctrl+K`, Agent/message/file/link search and Agent actions are mounted through the Agent-owned `agent-command-palette.tsx` boundary; the primary shell no longer imports the recovered Grok palette directly. |
| Plugins / Computer | secondary overlays from Agent workspace | top-level messenger/product sections and profile panels | IN PROGRESS: Plugins/Settings moved to Agent footer; Computer is a first-class Agent overlay explicitly bound to `agentId` and the installed Fabushi machine. `computer.status` now flows through `AgentRuntimeCoordinator`, and the Agent overlay presents authoritative Accessibility, Screen Recording, capture, input and local-execution capability state; Agent-scoped snapshot/action results project into the owning Agent timeline as `computer-handoff`. |
| Groups / Agent network | first-class Agent group/org semantics | group/channel semantics are primarily messenger/community based | IN PROGRESS: the primary Network surface is now Agent-owned and backed by Mahayana `group.list/create/update/delete/send` plus `agent.broadcast` through `AgentCoordinatorClient`; Agent group lifecycle is no longer routed through Messenger peers. Full org graph and richer Grok-style coordination UX remain. |
| Legacy messenger | not the primary desktop product shell | dominates navigation and renderer architecture | TRANSITION: normal Agent header/transcript/composer no longer render through the Messenger path; non-Agent/Mini App compatibility UI and hidden legacy navigation still need final removal |

## 2026-09-20 Agent workspace extraction

This refactor slice is implemented on PR #7 without merging `main`.

- `desktop/src/agent-workspace/agent-root-shell.tsx`: primary desktop product boundary; compatibility Messenger surfaces no longer define root product identity.
- `desktop/src/agent-workspace/agent-workspace-controller.ts`: per-Agent request/operation ownership plus Agent-scoped text, attachments, reply context, upload state and atomic draft take/restore.
- `desktop/src/agent-workspace/agent-transcript-store.ts`: per-Agent transcript source ownership for queued and accepted user turns, provisional-request adoption, one-turn AssistantTurn reduction and terminal settlement outside the Messenger renderer.
- `desktop/src/agent-workspace/coordinator-client.ts`: narrow Mahayana command boundary for send/open/broadcast/interrupt/approval, Agent group list/create/update/delete/send and `attachment.upload -> attachment.stored`; it now owns retry/backoff and generation-aware recovery handshake instead of delegating reconnect or Agent collaboration commands to React.
- `desktop/src/agent-workspace/agent-runtime-coordinator.ts`: single Agent event reducer for request→operation adoption, background Agent attribution, operation-scoped approvals, 16 ms delta coalescing, ordered finalization and per-Agent terminal settlement; it never infers ownership from the currently visible Agent.
- `desktop/src/agent-workspace/use-agent-workspace-runtime.ts`: renderer lifecycle facade that owns the long-lived workspace controller, transcript store, runtime coordinator, submission queue and restart-safe draft persistence. Messenger compatibility code supplies peer/transport adapters but no longer constructs or persists Agent runtime state.
- `desktop/src/agent-workspace/transcript-model.ts`: canonical `TranscriptEntry` domain replacing view-level dependence on three overlapping transcript shapes.
- `desktop/src/agent-workspace/agent-transcript.tsx`: primary ordered Agent timeline for messages, thinking, tools and AssistantTurn.
- `desktop/src/agent-workspace/agent-workspace.tsx`: Agent-owned composition of header, transcript, notices and composer.
- `desktop/src/agent-workspace/agent-network.tsx`: primary Mahayana Agent collaboration surface for runtime Agent groups and broadcast. Group refresh is open/mode-scoped so Host group events cannot create a renderer refresh loop.
- `desktop/src/agent-workspace/agent-command-palette.tsx`: primary command-palette ownership boundary; the recovered Grok component is presentation-only behind this Agent workspace adapter.
- `desktop/src/agent-workspace/agent-attachments.ts`: six-file Agent attachment validation/upload preparation matching the existing Mahayana Host limits.
- `desktop/src/agent-workspace/agent-draft-store.ts`: restart-safe text, uploaded attachment, reply-context and structured `@Agent` reference recovery keyed by Agent compatibility peer.
- `desktop/src/agent-workspace/agent-sidebar-state.ts`: account-scoped custom sections, assignment and Unassigned projection.
- `desktop/src/agent-workspace/use-agent-sidebar-controller.ts`: Agent-owned pinned-order, durable section hydration/persistence and multi/range-selection state. The Messenger compatibility shell supplies UI prompts/confirmations but no longer owns Sidebar persistence or selection state machines.
- `desktop/src/agent-workspace/prompt-context.ts`: semantic reply context and stable Agent references are kept separate from visible user-message text.
- `desktop/src/grok-shell/grok-agent-composer.tsx`: content-editable Agent input with multi-file/drop/paste, attachment pills/removal, attachment-only send, queued-send compatibility, voice dictation and structured `@Agent` selection.
- `desktop/electron/main.cjs`: microphone permission is limited to trusted `app://bundle` audio-only capture; camera remains denied.
- `desktop/src/messaging-shell-v2.tsx`: normal Agent rendering reads directly from the workspace-owned `AgentTranscriptStore`, while controller/coordinator/submission-queue/draft-persistence lifetime is obtained from `useAgentWorkspaceRuntime` instead of being constructed in Messenger. Regenerate lookup is transcript-owned via `userPromptBefore`; global `messages` remains only for compatibility Messenger/group/Mini App surfaces.
- `desktop/e2e/grok-parity.spec.ts`: verifies one canonical Agent workspace, two-Agent operation/draft isolation, coordinator-owned concurrent A/B streams without cross-talk, draft preservation across runtime reset, atomic draft recovery, one canonical AssistantTurn through request→operation adoption, transcript-owned regenerate lookup that skips queued future prompts, Agent attachment upload/timeline/attachment-only send, modifier selection + custom section creation, and that the mounted Network is the Agent-domain group surface rather than the legacy presentation-only Grok Network.
- `desktop/electron/host-process.cjs` + `host-process.test.cjs`: development/CI Host resolution now supports the staged `desktop/resources/bin` generation without changing the packaged signed-resource boundary; this is covered by the Rust desktop runtime Actions renderer job.
- `desktop/scripts/check-agent-workspace-boundary.mjs`: CI-enforced architecture guard that rejects reintroduction of renderer-global Agent operation pointers, direct Coordinator/submission-queue/draft-persistence ownership, direct Sidebar state/persistence ownership, queued Agent transcript maps, Agent delta buffers, Agent transcript copy-back through `messages`, Messenger-global regenerate lookup, direct Host subscription ownership, legacy Grok Network/Command Palette mounting, or renderer-owned Agent group lifecycle commands.

This is not a claim of full parity. Operation-scoped approval request/resolution and generation-aware Host recovery are owned by the Agent runtime boundary. `root.json` now materializes and validates its full CAS graph before transcript/attachment/checkpoint hydration; stale running checkpoints fail closed into an interrupted recovery notice. Computer permission/status projection and Native Host sidebar durability are implemented. Remaining gates are the full Fabu rich-document/provider/workflow editor, account-level cross-device sidebar sync, Agent settings/profile parity, extraction of the remaining compatibility Messenger renderer branches, and packaged macOS parity acceptance.

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

P0: finish primary shell migration and current CI; the command palette, visible Agent selectors, two-Agent parallel-operation regression and hidden/rename/delete persistence are already present on this branch.

P1: continue shrinking the monolithic compatibility renderer now that AgentRootShell, AgentWorkspaceController, AgentTranscriptStore, AgentRuntimeCoordinator, the workspace runtime facade, the Agent Sidebar controller, AgentSidebar, AgentHeader, Transcript, Composer and overlays are extracted. Messenger no longer owns the primary Agent runtime or Sidebar state lifecycle; the remaining work is to move Agent action/peer projection and compatibility-only rendering/event branches behind dedicated adapters, then delete those branches from the primary renderer.

P2: finish the remaining richer renderer parity: full Fabu rich-document/provider/workflow editor and Agent settings/profile editor. OS-level Computer permission/status presentation is implemented. Sidebar sections/pinned order are Native-Host durable, but account-level cross-device section state is blocked on a missing control-plane preference/CAS endpoint. Approval and Agent-scoped Computer handoff timeline projection, draft/waiting/activity, attachment timeline, pinned drag, custom sections, multi-select and stable `@Agent` references are implemented.

P3: DONE for the primary Agent path: HostProcess lifecycle/generation is replayed through the Electron edge, `AgentCoordinatorClient` performs bounded recovery handshakes, stale Agent turns settle interrupted, drafts/history survive, and recovered generations refresh settings/list/current conversation. Non-Agent compatibility event families remain in the legacy adapter.

P4: DONE for the primary Agent root: cross-device `root.json` validation, full referenced-object materialization, transcript + attachment-index + runtime-checkpoint recovery and etag/conflict-safe writes are implemented. A remote `running` checkpoint is never promoted to a local active operation without a fresh runtime event.

P5: Mahayana Agent group lifecycle and broadcast are wired through the Agent-owned Network; complete the remaining Agent-to-Agent/org graph/workflow coordination UX and packaged macOS acceptance against a pinned reference timeline.