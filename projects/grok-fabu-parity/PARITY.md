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
| Product shell | Agent-first sidebar + transcript + composer; secondary capabilities live behind overlays/menus | Telegram-like sections for Chats, Contacts, Bots, Groups, Channels, Calls, Saved, Payments and Mini Apps | IMPLEMENTED for the primary shell: `AgentRootShell` is the sole mounted desktop root and normal Agents mount `AgentWorkspace -> AgentTranscript -> AgentComposer`. Contacts/Telegram/Mini App/legacy group peer construction, messaging-envelope recognition and compatibility surface routing live behind `messenger-compatibility-adapter.tsx`; dead `SectionPanel`/hidden legacy navigation scaffolding is removed. |
| New chat | Creates a new Agent and opens it immediately | New button primarily creates groups/channels | IMPLEMENTED: visible New button issues `bot.create`, then opens the created Agent |
| Agent identity | Agent is the runtime/state owner; presentation identity is separate | Bot/conversation IDs were frequently reused as runtime identity | IMPLEMENTED: explicit Bot -> `agentId` mapping and runtime-ID routing |
| Sidebar state | Pinned/unpinned Agents, Working, waiting/attention, unread, custom sections, modifier multi-select and bulk organization | Generic peer list and messenger status | IMPLEMENTED: pinned drag reorder, draft/waiting/current-activity priority, collapse/pin/hide/rename/duplicate/delete, account-scoped sections, Ctrl/Cmd multi-select, Shift range selection and bulk move/create/delete are Agent-owned. `account-sidebar-layout.ts` persists `{revision,pinnedOrder,sections}` through the account sync object API with etag/expect-absent CAS conflict retry. Pinning is presentation-only: a pinned Agent retains section ownership and projects back into that section after unpin. |
| Parallel Agents | One Agent may work while another Agent is opened or receives work | One renderer-global `agentOperationId` / pending request blocked unrelated Agents | IMPLEMENTED: `AgentOperationRegistry` owns request/operation state per Agent peer |
| Send lifecycle | pending/queued/failed/sent/cancelled is transport state; thinking/running is separate | UI pending state and runtime operation state were partially coupled | IMPLEMENTED: Fabu-style submission queue plus per-Agent runtime ownership; queued payloads preserve attachments/reply context and are projected into the same `AgentTranscriptStore` instead of a renderer-global queued-message map |
| Agent files | each Agent owns `profile.json`, `settings.json`, memory, automations, workflow enablement | mixed global/profile JSON and runtime-specific state | IMPLEMENTED for profile/settings, memory/workflows, per-Agent automation folders and clone semantics |
| Automation IDs | namespace is Agent directory + automation ID | global map could collide across Agents | IMPLEMENTED: runtime map is namespaced by Agent and disk layout is per-Agent |
| Agent clone | copies reusable Agent-owned state, not transcript/audit/attachments | clone was primarily Bot profile clone | IMPLEMENTED: memory, automations, workflow enablement copied; transcript/audit/attachments excluded |
| Inference session | independent session per Agent conversation | historically provider-global or active-chat-oriented state | IMPLEMENTED: session ID and durable snapshot are conversation/Agent scoped. An optional Agent `inferenceProvider` flows through the Rust Host protocol, `RuntimeCommand`, conversation request and Kernel session metadata; the single `ProviderRoutingEngineBackend` routes Fabushi/Codex/OpenRouter/Claude Code independently per Agent without process-global switching, and provider identity is snapshot-persisted so a provider change cannot silently resume the previous engine session. Absence explicitly inherits the account default. |
| Account switching | durable Agent state survives account/session switching | reset path could destroy local transcript/session state | IMPLEMENTED: non-destructive history switching + account-scoped persistence |
| Cloud Agent store | immutable CAS blobs + mutable root/reference state | renderer cloud sync was profile-oriented | IMPLEMENTED for the primary Agent root: `root.json` is validated, every referenced object is materialized against its blob/etag/revision metadata, and transcript + attachment index + runtime checkpoint recover from the same verified snapshot. A checkpoint that was still running on another device becomes an interrupted recovery notice instead of being falsely adopted as a local active operation. |
| Coordinator boundary | narrow lifecycle + command/event protocol, reconnection, pending requests outside view components | `messaging-shell-v2.tsx` owns too many runtime, product and presentation concerns | IMPLEMENTED for the primary Agent path: Electron Host generation lifecycle is projected to the renderer; `AgentCoordinatorClient` owns bounded reconnect/handshake state; `AgentRuntimeCoordinator` interrupts stale operations without clearing Agent drafts/history and recovered generations reopen current settings/list/conversation state |
| Agent Composer | Agent-scoped draft/reply/attachments/drop/paste/voice; transport acceptance independent from Agent turn state | textarea plus Messenger attachment/reply behavior | IMPLEMENTED for the frozen suggestion scope: TipTap 3.14.0, multi-file/drop/paste, attachment-only sends, restart-safe draft recovery, semantic reply context, voice-at-selection, IME/Escape/send gating and six-file enforcement are Agent-owned. `@Agent`, enabled workflow, normalized `@MCP`, semantic links, emoji catalog suggestions and `#` GitHub PR-reference suggestions are provided through Agent-owned controllers/providers and preserve stable ids through queue failure/recovery. |
| Transcript projection | ordered rich entries for text, reasoning, tools, approvals, attachments and timeline state | mixed DisplayMessage / AssistantTurn / legacy message paths | IN PROGRESS: canonical `TranscriptEntry` + `AgentTranscript` are mounted for normal Agents; `AgentTranscriptStore` owns user turns, per-Agent source threads, request→operation rekey, AssistantTurn reduction and terminal settling, and normal Agent rendering no longer round-trips through renderer-global `messages`; approval emitters are operation/Agent-attributed end-to-end and render as interactive timeline cards; scoped Computer snapshot/action results now carry `agentId` and render as `computer-handoff` entries, while unscoped local/remote events remain in compatibility surfaces |
| Command palette | keyboard-first Agent navigation/actions | no equivalent primary Agent palette | IMPLEMENTED: `Cmd/Ctrl+K`, Agent/message/file/link search and Agent actions are mounted through the Agent-owned `agent-command-palette.tsx` boundary; the primary shell no longer imports the recovered Grok palette directly. |
| Plugins / Computer | secondary overlays from Agent workspace | top-level messenger/product sections and profile panels | IN PROGRESS: Plugins/Settings moved to Agent footer; Computer is a first-class Agent overlay explicitly bound to `agentId` and the installed Fabushi machine. `useAgentComputerController` now owns `RemoteComputerDesktopController` lifetime, registration, pairing, remote-session authorization, capability refresh and Agent-switch visibility; `computer.status` still flows through `AgentRuntimeCoordinator`, and Agent-scoped snapshot/action results project into the owning Agent timeline as `computer-handoff`. |
| Groups / Agent network | first-class Agent group/org semantics | group/channel semantics are primarily messenger/community based | IMPLEMENTED for the current reference scope: the primary Network is backed by Mahayana `group.list/create/update/delete/send`, direct `agent.send/agent.peerHistory`, and `agent.broadcast`. It now renders a real coordination graph with Agent/Group relationships, working/needs-input/unread metrics, clickable group-to-Agent edges and recent direct-handoff feed while keeping each Agent's conversation/operation/Computer state isolated. |
| Legacy messenger | not the primary desktop product shell | dominates navigation and renderer architecture | COMPATIBILITY-ONLY: no hidden legacy main navigation remains and `AgentRootShell` is the sole root. Compatibility peer construction, event-envelope recognition and feature-surface routing are isolated behind `messenger-compatibility-adapter.tsx`; the remaining large file name is legacy debt, not product-shell ownership. |

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
- `desktop/src/agent-workspace/agent-network.tsx`: primary Mahayana Agent collaboration surface for runtime Agent groups and broadcast. Group refresh is open/mode-scoped so Host group events cannot create a renderer refresh loop. Runtime Agent ids and persisted Bot surface ids are reconciled through the canonical Agent identity model because the Rust Host accepts either on write and normalizes stored group members to the Bot surface id.
- `desktop/src/agent-workspace/agent-command-palette.tsx`: primary command-palette ownership boundary; the recovered Grok component is presentation-only behind this Agent workspace adapter.
- `desktop/src/agent-workspace/agent-attachments.ts`: six-file Agent attachment validation/upload preparation matching the existing Mahayana Host limits.
- `desktop/src/agent-workspace/agent-draft-store.ts`: restart-safe text, uploaded attachment, reply-context and structured `@Agent` reference recovery keyed by Agent compatibility peer.
- `desktop/src/agent-workspace/agent-sidebar-state.ts`: account-scoped custom sections, assignment and Unassigned projection.
- `desktop/src/agent-workspace/agent-model.ts`: canonical Agent/sidebar projection model. The old `grok-runtime/agent-model.ts` is compatibility aliases only; the primary renderer no longer imports Grok runtime or presentation modules directly.
- `desktop/src/agent-workspace/use-agent-directory-controller.ts`: Agent directory cache plus list/create/update/duplicate/delete/hide semantics; `bot.*` remains only a Mahayana wire-compatibility detail behind `AgentCoordinatorClient`, and `bot.listed/bot.changed` no longer reduce inside Messenger.
- `desktop/src/agent-workspace/use-agent-network-controller.ts`: Agent Network open/broadcast state, Agent-group projection, direct peer history/handoff and group/broadcast command ownership; Messenger only mounts the surface and retains a separate compatibility group projection for legacy community UI.
- `desktop/src/agent-workspace/use-agent-command-palette-controller.ts`: `Cmd/Ctrl+K`, Escape, query and visibility lifecycle for the global Agent command center.
- `desktop/src/agent-workspace/use-agent-workflow-controller.ts`: per-Agent workflow list/cache and `workflow.changed/listed` refresh for Composer references.
- `desktop/src/agent-workspace/use-agent-store-sync-controller.ts`: memory and automation mutation/list events mirror into the existing `FabuAgentStore` CAS graph without renderer-global memory/automation sync state.
- `desktop/src/agent-workspace/use-agent-sidebar-controller.ts`: Agent-owned pinned-order, durable section hydration/persistence and multi/range-selection state. The Messenger compatibility shell supplies UI prompts/confirmations but no longer owns Sidebar persistence or selection state machines.
- `desktop/src/agent-workspace/prompt-context.ts`: semantic reply context and stable Agent references are kept separate from visible user-message text.
- `desktop/src/grok-shell/grok-agent-composer.tsx`: content-editable Agent input with multi-file/drop/paste, attachment pills/removal, attachment-only send, queued-send compatibility, voice dictation and structured `@Agent` selection.
- `desktop/electron/main.cjs`: microphone permission is limited to trusted `app://bundle` audio-only capture; camera remains denied.
- `desktop/src/messaging-shell-v2.tsx`: normal Agent rendering reads directly from the workspace-owned `AgentTranscriptStore`, while controller/coordinator/submission-queue/draft-persistence lifetime is obtained from `useAgentWorkspaceRuntime` instead of being constructed in Messenger. Regenerate lookup is transcript-owned via `userPromptBefore`; global `messages` remains only for compatibility Messenger/group/Mini App surfaces.
- `desktop/e2e/grok-parity.spec.ts`: verifies one canonical Agent workspace, two-Agent operation/draft isolation, coordinator-owned concurrent A/B streams without cross-talk, draft preservation across runtime reset, atomic draft recovery, one canonical AssistantTurn through request→operation adoption, transcript-owned regenerate lookup that skips queued future prompts, Agent attachment upload/timeline/attachment-only send, modifier selection + custom section creation, and that the mounted Network is the Agent-domain group surface rather than the legacy presentation-only Grok Network.
- `desktop/electron/host-process.cjs` + `host-process.test.cjs`: development/CI Host resolution now supports the staged `desktop/resources/bin` generation without changing the packaged signed-resource boundary; this is covered by the Rust desktop runtime Actions renderer job.
- `desktop/scripts/check-agent-workspace-boundary.mjs`: CI-enforced architecture guard that rejects reintroduction of renderer-global Agent operation pointers, direct Coordinator/submission-queue/draft-persistence ownership, direct Sidebar state/persistence ownership, queued Agent transcript maps, Agent delta buffers, Agent transcript copy-back through `messages`, Messenger-global regenerate lookup, direct Host subscription ownership, legacy Grok Network/Command Palette mounting, or renderer-owned Agent group lifecycle commands.

This is not a claim of released parity. Operation-scoped approval request/resolution, generation-aware Host recovery, verified Agent-root CAS, Composer suggestions, account sidebar CAS, Agent-scoped provider routing and the richer Network graph are implemented in code. The remaining release gate is exact-HEAD CI plus packaged macOS acceptance/evidence; no `main` merge is permitted before those gates pass. Fabu profile fields `name/title/description/avatarShape/avatarColor`, notifications and explicit Agent provider overrides are carried through the Agent settings controller; hidden-from-sidebar remains correctly owned by Sidebar state.

## 2026-09-21 Section failure evidence and current release gate

The only failure-evidence entry for the Section regression is Desktop Chat Parity run `35522950977` at source HEAD `e21018fb4bf0ab64cd10564d66c9b753a1ff6efe`: `Focused Electron chat E2E` job `106110502994` failed while `Renderer typecheck and build` job `106110503124` succeeded. Artifact `10608449358` contains the screenshot/error-context/trace showing the selected primary Agent still under Pinned and no `Focused work` section after the Section action. The old path used a browser `window.prompt`, and the Sidebar/controller also incorrectly filtered pinned Agents from section membership.

The branch then replaced the browser prompt with the Agent-owned section dialog and run `35524248478` at HEAD `2b312ba7defd9d2309e1cb85245341f30a18f9de` passed both `Renderer typecheck and build` job `106113547985` and `Focused Electron chat E2E` job `106113548111`; Playwright artifact `10608533873` has digest `sha256:cc9671e7f0b5993b6e39027c50a13a8283973edf8bfc350705fc647d5daf1beb`. This slice further fixes the underlying pin/section ownership semantics and strengthens the E2E to unpin the selected Agent and prove it projects into its persisted section. These new changes require a fresh exact-HEAD CI run before they count as verified.

No merge to `main` is allowed until the fresh code gates and same-HEAD packaged macOS acceptance are both green with retained evidence.

### 2026-09-21 cross-device Sidebar convergence hardening

The account Sidebar document is no longer merely CAS-protected last-write-wins data. `account-sidebar-layout.ts` now performs a three-way merge from the renderer's last authoritative account snapshot, the local mutation, and the newest remote snapshot observed after an etag conflict. Unpin/removal cannot be resurrected by a concurrent reorder; remote-only pinned Agents and independently-created sections survive local retries; concurrent rename/collapse edits merge by field. `use-agent-sidebar-controller.ts` retains the authoritative snapshot, polls the account object revision/etag while the account is active, projects remote changes back into the Agent-owned Sidebar state, and writes the merged committed result through the existing serialized cloud write chain.

The regression is executable rather than documentary: `grok-parity.spec.ts` covers a local unpin + remote pin addition + independent local/remote section edits, while the visible Section journey continues through the Agent-owned `Create section` dialog and proves that a pinned Agent retains section ownership and appears there after Unpin. `check-agent-workspace-boundary.mjs` rejects removal of the three-way merge, authoritative cloud snapshot, remote refresh, or base-aware write path, and Desktop Chat Parity's renderer job now runs that architecture gate before typecheck/build.

Desktop Chat Parity run `35536089732` on exact code HEAD `a5d686de8671e64ddcb732eb5e6ffc7477bdb054` passed both `Focused Electron chat E2E` job `106145271909` and `Renderer typecheck and build` job `106145272030`. Playwright artifact `10612852168` has digest `sha256:907d42f5212af0dc5587b31b9593fa8f2d8de98bdf6c78ea9745dc55cd613d35`; its successful trace contains the cross-device CAS regression and the Section → `Focused work` → Unpin journey. This inventory update itself is documentation-only, so the release/package gate still requires a fresh run on the resulting exact HEAD before packaging.


## 2026-09-20 Composer ownership cutover

The primary Agent Composer no longer copies Agent text into the renderer-global `composer` state. Normal Agent input now reads and writes `AgentWorkspaceController` directly, and send atomically takes the complete Agent-owned draft before entering `AgentSubmissionQueue`. Failed/queued submissions continue to restore through `useAgentWorkspaceRuntime` without clobbering a newer draft.

- `desktop/src/agent-workspace/agent-composer.tsx` now owns the actual Agent Composer implementation; `grok-shell/grok-agent-composer.tsx` is compatibility-only.
- Agent switching no longer hydrates the Messenger `composer` string from an Agent draft.
- Edit-message for a normal Agent writes directly to the Agent draft.
- Workflow suggestions now persist a stable `workflow:id` prompt reference, matching the existing stable `agent:id` reference behavior.
- `check-agent-workspace-boundary.mjs` fails CI if the primary Agent Composer reimports the Grok compatibility shell or is rebound to renderer-global Messenger composer state.

The structured draft contract from this slice is now consumed by the TipTap editor parity slice below; Mahayana continues to execute the normalized plain-text projection.

## 2026-09-20 Structured draft + canonical transcript cutover

Agent drafts now persist both the normalized plain prompt used by Mahayana and a TipTap-compatible serialized `richText` document. The document preserves stable Agent/workflow nodes, is carried through queued submissions and failure recovery, and is attached to optimistic user transcript entries. This gives the later exact Fabu rich editor a stable storage contract without changing the Rust execution protocol.

The Agent transcript source store now canonicalizes by `operationId`: only one `assistant-turn` survives for an operation, and a legacy peer message with the same operation is folded into an otherwise-empty assistant turn instead of rendering as a second answer. This normalization runs on hydration and every store update, so restart/cloud history cannot reintroduce the duplicate-reply UI that the live reducer already avoided.

Failure recovery also no longer copies an Agent draft back into the renderer-global Messenger composer. The controller remains the sole owner through send, queue, failure and retry.

## 2026-09-20 Runtime-owned submission/adoption cutover

The primary renderer no longer creates Mahayana chat request ids, begins optimistic Agent turns, calls `AgentCoordinatorClient.send`, or adopts operation ids. `useAgentWorkspaceRuntime` now owns that complete lifecycle and exposes a narrow `submit(...)` boundary. The shell supplies stable Agent/conversation identity plus the Agent-owned draft; queueing, request creation, optimistic transcript projection, transport send, operation adoption, rollback and failed-draft restoration stay inside the Agent runtime boundary.

`check-agent-workspace-boundary.mjs` now rejects any reintroduction of `agentSubmissionQueue`, raw submission-queue wiring, or `dispatchAgentPromptNow` in the primary shell.

## 2026-09-20 TipTap Composer parity cutover

The primary Agent input surface now uses TipTap 3.14.0, matching the frozen Fabu reference dependency line. `AgentRichTextEditor` owns the ProseMirror document, serializes the same `doc/paragraph/text` shape, preserves Agent mention and workflow-reference nodes, handles external `setContent(..., { emitUpdate: false })` synchronization, and routes file paste through the existing Agent attachment path.

The existing Composer shell remains responsible for attachment chips, drag/drop, voice dictation, reply state and suggestion list presentation, but it no longer mutates `contentEditable.innerText` directly. Agent switching remounts the editor by Agent scope key, while `AgentWorkspaceController` remains the durable owner of `prompt + richText + references`.

`desktop/package.json` and `desktop/package-lock.json` are updated together from the pinned Fabu 3.14.0 dependency graph so CI can continue using a locked install.

## 2026-09-20 Agent Settings controller cutover

The Agent settings UI now follows the frozen Fabu controller boundary rather than owning request generation/pending/error state inside the view. `AgentSettingsController` fences profile/notification mutations by selected Agent generation, consumes the authoritative `AgentDirectoryController` projection, and prevents late replies from a previous Agent selection from mutating the newly opened Agent settings surface. The Messenger compatibility shell no longer defines direct profile/notification mutation helpers.

## 2026-09-21 Computer + runtime-control ownership cutover

The primary Messenger compatibility shell no longer owns the installed-machine Computer runtime. `desktop/src/agent-workspace/use-agent-computer-controller.ts` now owns `RemoteComputerDesktopController` creation/disposal, account-scoped registration, pairing-code refresh, remote-session approval/denial/disconnect, remote-control enablement, capability refresh and automatic close on Agent switch. The shell only projects the controller state into Agent surfaces.

Approval resolution and interrupt are now exposed by `useAgentWorkspaceRuntime`; primary Agent UI no longer calls `AgentCoordinatorClient.resolveApproval` or `AgentCoordinatorClient.interrupt` directly. This keeps request/operation control inside the same runtime facade that already owns submission/adoption/transcript state.

`desktop/scripts/check-agent-workspace-boundary.mjs` now fails CI if the primary shell reintroduces `RemoteComputerDesktopController`, Computer capability state setters, direct `computer.status`, `reportOpenComputer`, approval resolution or interrupt calls.

## 2026-09-21 Agent conversation + attachment IO cutover

Normal Agent `conversation.open` and attachment/voice uploads now enter `useAgentWorkspaceRuntime` rather than calling `AgentCoordinatorClient` from `messaging-shell-v2.tsx`. Compatibility Messenger conversations keep their direct adapter path, but Agent conversation activation and Agent-owned file persistence are now fenced by the workspace runtime and report failures to the owning Agent.

## 2026-09-21 Fabu Composer semantic parity slice

The frozen `bhrumom/fabu@939eb8e5e01d0d75ce5df2e46f96db8f835d5891` Composer/rich-text editor was re-read before changing behavior. The resulting changes deliberately match semantic behavior rather than adding unsupported formatting UI:

- Voice transcription inserts through TipTap at the current selection, including Fabu's spacing rule, instead of appending to the end of the draft.
- IME composition, Escape blur/cancel behavior and every submit path now respect the same send fence; recording/transcription cannot accidentally submit a partial draft.
- File input, drag/drop and paste are all capped at `AGENT_ATTACHMENT_LIMIT` before upload and the attach action disables at the limit.
- TipTap Link is an explicit 3.14.0 dependency. Link marks remain non-inclusive and normalized prompt text preserves a non-URL label as `label (href)`.
- `useAgentMcpController` normalizes Mahayana's deliberately-untyped `mcp.listed.servers` payload and exposes stable MCP references to the Agent Composer. MCP discovery is deferred until core workspace hydration so it cannot compete with first-frame Agent/conversation bootstrap.
- MCP references reuse `AgentPromptReference(kind='mcp')`, `AgentWorkspaceController`, the submission queue, persisted rich-text mentions and failure recovery; no second draft or provider runtime was introduced.

A per-Agent inference-provider picker is still intentionally absent because the current Mahayana contract exposes `ProductHostSettings.inferenceProvider` globally. Adding a visual per-Agent selector before the runtime contract exists would recreate the global-state bug this refactor is removing.

## 2026-09-21 Agent Network and profile parity slice

Agent collaboration is no longer only a Broadcast facade. Mahayana already exposes `agent.send`, `agent.peerHistory` and `agent.peerMessage`; `AgentCoordinatorClient` and `useAgentNetworkController` now own those commands/events. The Network surface uses a single selected target as a direct Agent-to-Agent handoff from the active Agent, supports the runtime priority flag, and shows recent peer history. Explicit Broadcast mode and multi-target sends continue to use `agent.broadcast`.

The Network controller also owns its own `GroupSummary[]`. `group.listed/group.changed` update that Agent-domain projection while still falling through to the old Messenger group reducer during compatibility migration. The visible Agent Network no longer reads the renderer-global Messenger `groups` state.

Agent settings now carry the complete frozen Fabu profile-file fields that the current Mahayana `BotSummary/bot.update` contract already supports: `name`, `title`, `description`, `avatarShape` and `avatarColor`. Notifications remain settings-controller-owned; hidden-from-sidebar remains Sidebar-owned via `bot.setHidden`, matching the existing product boundary instead of duplicating hidden state in two controllers.

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

P1: DONE for shell ownership: AgentRootShell, AgentWorkspaceController, AgentTranscriptStore, AgentRuntimeCoordinator, Agent directory/navigation projection, Sidebar/Network/Command Palette/Workflow/Store-Sync controllers, AgentHeader, Transcript, Composer and overlays are extracted. Contacts/Telegram/Mini App peer construction, messaging-envelope recognition and secondary-surface routing now sit behind `messenger-compatibility-adapter.tsx`; the remaining `messaging-shell-v2.tsx` filename is compatibility debt and does not own the product root.

P2: DONE for the frozen core parity inventory: emoji and GitHub PR-reference suggestion providers are Agent-owned; per-Agent provider selection is backed by a Rust/Mahayana Agent-scoped contract, one Kernel `ProviderRoutingEngineBackend` and durable session identity before the UI selector; sidebar sections/pinned order use account object CAS/revision; and Agent Network now exposes org relationships, coordination metrics and recent handoff history. Richer cosmetic avatar preset presentation remains optional polish, not a core architecture gate.

P3: DONE for the primary Agent path: HostProcess lifecycle/generation is replayed through the Electron edge, `AgentCoordinatorClient` performs bounded recovery handshakes, stale Agent turns settle interrupted, drafts/history survive, and recovered generations refresh settings/list/current conversation. Non-Agent compatibility event families remain in the legacy adapter.

P4: DONE for the primary Agent root: cross-device `root.json` validation, full referenced-object materialization, transcript + attachment-index + runtime-checkpoint recovery and etag/conflict-safe writes are implemented. A remote `running` checkpoint is never promoted to a local active operation without a fresh runtime event.

P5: CODE COMPLETE for the current Network inventory; the remaining gate is packaged macOS acceptance against the exact successful HEAD, including direct handoff+broadcast and two-Agent isolation with retained video/screenshots/trace/logs.

## 2026-09-21 Rust-owned workspace state and capability gateway closure

This slice closes the two remaining architecture leaks identified during the OpenBot + Grok Bot design review before any new CI/release run:

- Agent Composer drafts, rich-text documents, attachments, reply context and stable references are no longer persisted by renderer `localStorage` or the generic Electron client-persistence bridge. `useAgentWorkspaceRuntime` hydrates `agent-workspace:drafts:v2` from Mahayana `RuntimeStore`, merges any one-time legacy draft without overwriting edits made while the Host connects, and writes the authoritative snapshot back to Rust with a short trailing debounce.
- `RuntimeStore` now has an explicit SQLite `workspace_state` table. FeatureHost scopes the durable key by the authenticated account fingerprint before it reaches the Runtime, so drafts cannot cross account boundaries. The old localStorage key remains read-only migration input and is deleted only after a successful Rust write.
- `CapabilityBroker` now accepts generic `CapabilityRequest` authorization, not only registry descriptors. FeatureHost applies that gateway before privileged Computer screen/input, Remote Computer sessions, MCP tool calls, Agent-to-Agent handoff/broadcast, Agent workspace file read/write, Mini App open and Connector management paths. Existing domain-specific permission checks and the physical `ComputerControlLease` remain a second enforcement layer.
- `desktop/scripts/check-agent-workspace-boundary.mjs` now fails CI if Agent draft persistence is moved back to renderer writes or if the privileged FeatureHost paths bypass the Rust CapabilityBroker.
- Desktop version is `1.2.74`. No code/test/version mutation is permitted after the exact-HEAD CI gate; the same merged source commit must be passed to the signed/notarized macOS packaging workflow.

The remaining gate is verification, not additional architecture work: create a fresh PR from `refactor/fabu-agent-runtime-20260920` to `main`, require the Desktop Chat Parity and Rust desktop runtime workflows on its exact HEAD, merge only after they are green, then package and publish macOS from the merged exact source.
