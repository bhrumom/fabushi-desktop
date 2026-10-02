# FBCP P0 — Exact-HEAD Current Owner Inventory

Status: evidence snapshot; P0 not yet passed
Project: FBCP-001 Revision 2
Captured: 2026-10-02
Main input: `d9ae2773f2c517a0cb911e7b7bc996905cf3ada4`
PR #20 input: `dcb19a94383833fc1ec5074f10c4bbbd28c09036`
PR state at capture: open, draft, base `main`, head `refactor/grok-018-architecture-rebuild`

This file records the current production owners that FBCP must absorb into. It is not an implementation-complete claim. Any later PR #20 HEAD invalidates the source conclusions below until this inventory is refreshed.

## Method

The inventory was built from the exact PR #20 HEAD, starting at shipping entrypoints and following the production composition paths. Directory/name similarity by itself is not evidence. The paths below are the current owners or production seams that are actually composed by the renderer, Electron main, Coordinator, Host, and Runner.

Refresh through `556f6308...` revalidated the live PR #20 production ownership. The delta from the prior `b541e2e1...` snapshot changes only `source/host/src/extensions/transcript/profile_watch.rs` and its focused contract; it stays inside the same Host Transcript owner and introduces no competing FBCP product owner.

## Exact-head owners

| Responsibility | Current owner / production seam | Exact-head evidence | FBCP consequence |
| --- | --- | --- | --- |
| Product shell | React production renderer | `frontend/src/main.tsx`; `frontend/src/production/bootstrap.tsx`; `frontend/src/production/ProductionRenderer.tsx` | Extend this shell; do not add a Telegram shell. |
| Sidebar / navigation | Existing conversation sidebar, currently Agent-shaped | `frontend/src/recovered/features/conversation/workspace/sidebar.tsx`; `frontend/src/production/sidebar-model.ts`; composition in `ProductionRenderer.tsx` | Evolve row model from Agent-only semantics to Human/Agent/Group/Channel/Topic without a second sidebar. |
| Agent roster / outline projection | Host Transcript `RosterProjection` consumed by shipping `ProductionRosterEmit` | `source/host/src/extensions/transcript/roster_projection.rs`; `source/host/src/extensions/transcript/roster_emit.rs`; focused evidence in `source/host/tests/roster_emit_contract.rs` | This remains the Agent roster/outline projection owner. It is reusable evidence for projection/coalescing behavior but is not a canonical Human contacts/presence owner. |
| Conversation workspace | Existing conversation workspace mounted by `ProductionRenderer` | `frontend/src/production/ProductionRenderer.tsx`; `frontend/src/recovered/features/conversation/workspace/*` | Human conversations must enter this workspace. |
| Transcript / cards | Existing typed transcript entry union + card resolver + strengthened Host transcript extension/production runtime at current PR #20 | `frontend/src/recovered/features/conversation/workspace/model.ts`; `frontend/src/recovered/features/conversation/workspace/transcript.tsx`; `frontend/src/recovered/features/conversation/cards/transcript-card/*`; `source/host/src/extensions/transcript/extension.rs` | Add Human-message semantics to this transcript; preserve tool/thinking/permission/task typed entries. |
| Composer / drafts | Existing composer, client-persisted drafts, submission queue | `frontend/src/recovered/features/conversation/workspace/composer.tsx`; `draft-state.ts`; `submission.ts`; `ProductionRenderer.tsx` | Generalize send intent; current shipping submit path is still Agent `sendPrompt`. |
| Reply / thread relation | Existing reply/thread projection | `frontend/src/recovered/features/conversation/workspace/reply-thread-controller.ts`; `source/shared/transcript.ts`; `source/shared/transcript-threads.ts` | Extend existing relation/provenance types; no Telegram-specific message model. |
| Shared Room / groups / members | Existing Shared Room UI/provider plus Host GroupChatGlue/orchestrator and group domain/store | `frontend/src/recovered/features/agent-info/shared-room/*`; `frontend/src/recovered/features/agent-info/group-members/*`; `source/host/src/extensions/transcript/group_chat_glue.rs`; `source/host/src/extensions/transcript/group_chat_orchestrator.rs`; `source/host/src/extensions/transcript/send_group_fanout.rs`; `source/host/src/groups/group_chat.rs`; `group_store.rs`; `remote_room_store.rs` | Extend participant/member model to Human + Agent through this shipping owner; do not create TelegramGroup/TelegramUser truth. |
| Reactions | Existing transcript reaction root/actions/picker | `frontend/src/production/reaction-root.ts`; `frontend/src/recovered/features/conversation/cards/transcript-card/reaction-actions.ts`; `reaction-picker.tsx` | Reuse reaction owner and expand actor identity semantics. |
| Attachments / resource ingress | Existing composer attachment lifecycle + Electron attachment manager + Host attachment extension | `frontend/src/recovered/features/conversation/workspace/desktop.ts`; `source/electron-main/attachments/attachment-manager.ts`; `source/host/src/extensions/attachments/*`; `source/host/src/attachment_paths.rs` | Extend shared resource lifecycle for Human messages/media; do not duplicate blob truth. |
| Agent artifacts / rich results | Existing transcript-card/tool/artifact surfaces | `frontend/src/recovered/features/conversation/cards/*`; `frontend/src/recovered/features/conversation/workspace/spreadsheet-viewer*`; Runner tool-result projections | Human messaging must coexist with these typed Agent-native results. |
| Permissions / approvals | Existing local-tool permission UI + Host permission extension + transcript permission entries | `frontend/src/recovered/features/permissions/local-tool/*`; `source/host/src/extensions/local_tool_permission/*`; transcript permission types in workspace `model.ts` | Room/member permissions should integrate here where semantically appropriate; execution approvals remain typed. |
| Search | Existing command palette/find-in-chat plus Host content-search extension | `frontend/src/production/CommandPalette.tsx`; `frontend/src/recovered/features/conversation/workspace/find-in-chat*`; `source/host/src/extensions/content_search/*` | Extend indexes/query scope for Human messages/resources rather than add Telegram search. |
| Settings | Existing settings overlay + Electron prefs + Host settings extension | `frontend/src/recovered/features/settings/overlay/*`; `source/electron-main/prefs/*`; `source/host/src/extensions/settings/*` | Add communication/privacy sections to existing settings. |
| Notifications / tray / badge | Existing root notification host + Electron notification managers + Host notification/tray extensions | `frontend/src/recovered/features/window-chrome/notification-host.tsx`; `source/electron-main/notifications/*`; `source/host/src/extensions/notifications/*`; `source/host/src/extensions/trays/*` | Native push/notification infrastructure must feed these owners. |
| Coordinator | Independent Rust Node Agent Coordinator role | `source/node-agent-coordinator/src/main.rs`; `lib.rs`; `renderer_port_server.rs`; `supervisor.rs` | Explicit Human→Agent escalation reuses this boundary; native messaging must not collapse it. |
| Electron ↔ Coordinator bridge | Electron launcher/port/runtime boundary | `source/electron-main/coordinator/*`; `source/electron-preload/coordinator-port-bridge.ts`; frontend `coordinator-client.ts` | Preserve process boundary and protocol ownership. |
| Host | Independent Rust Host process and extension registry | `source/host/app/src/main.rs`; `source/host/src/lib.rs`; `source/host/src/extensions/registry.rs`; `host_production_extensions.rs` | Native communication runtime support may be hosted here only where lifecycle/cohesion fits; Host must not become a broad parallel CommunicationCore. |
| Runner | Host Runner / turn execution owner | `source/host/src/runner/*`; `source/host/src/extensions/turn_execution/*`; `host_runner_composition.rs` | Agent execution remains Runner-owned. Human message transport is not a Runner responsibility. |
| Computer | Existing Computer UX + Runner Computer dependencies + VNC bridge | `frontend/src/recovered/features/computer/*`; `source/host/src/runner/computer_use.rs`; `source/electron-main/vnc/*` | Screen-share/call work must inspect and reuse this realtime surface where applicable. |
| Plugins / MCP | Existing plugin UI plus Electron/Coordinator/Host MCP lifecycle | `frontend/src/recovered/features/plugins/*`; `source/electron-main/mcp/*`; `source/node-agent-coordinator/src/mcp*`; `source/host/src/extensions/mcp/*` | Human-triggered Agent flows retain the existing MCP boundary. |
| Automations | Existing automation UI, Host domain/store/trigger, Host extension | `frontend/src/recovered/features/automations/*`; `source/host/src/automations/*`; `source/host/src/extensions/automations/*`; `source/shared/automations.ts` | Scheduled send should become an automation action, not a separate scheduler. |
| Durable local state / recovery | Existing client persistence + Host storage/SQLite recovery + state-backstop/session recovery | `source/shared/persistence.ts`; `frontend/.../draft-state.ts`; `source/host/src/storage/store_db.rs`; `sqlite_recovery.rs`; `source/host/src/extensions/state_backstop/*`; `source/host/src/extensions/session/*` | Extend existing durable/recovery mechanisms; Human messaging still needs its own durable queue contract. |
| Account/auth precursor | Existing Cursor/Fabushi account authorization and session wiring | `source/electron-main/account/*`; `source/electron-main/auth/auth-callback-registration.ts`; account/session frontend surfaces | This is an auth/session precursor, not yet a canonical Fabushi Human identity model. |
| External channels/connectors | Existing external connector channel address/delivery and Host connector runtime | `source/shared/channels.ts`; `source/shared/channel-messaging.ts`; `source/host/src/connectors/*` | Useful behavior reference/integration edge only. It must not be mislabeled as Fabushi native Human messaging. |
| Calls / call signaling | No dedicated canonical product call/session owner was found in the exact-head top-level renderer, Host extensions, Host domain roots, or Electron-main roots inspected for this snapshot | inspected `frontend/src/recovered/features`, `source/host/src`, `source/host/src/extensions`, `source/electron-main` | Keep unresolved in P0. Reuse Computer/platform where appropriate; propose a minimal call-session/signaling owner only after call research and ADR. |

## Shipping flow facts relevant to the first vertical slice

### Current Agent conversation send

The mounted composer does not emit a generic message command. In the current renderer, `sendComposerPrompt` commits attachments and calls Coordinator method `sendPrompt` with an `agentId`, prompt, nonce, reply target and attachment paths. Therefore a Human messaging slice cannot be implemented by relabeling an Agent row; it needs a new typed send intent below the same composer while retaining the existing Agent path.

### Current transcript reconciliation

The renderer already reconciles optimistic user entries with authoritative entries using `clientNonce`, and the transcript model already carries `delivery`, `replyToId`, reactions, attachments and typed non-message entries. These are strong reuse points for message identity/retry/reconciliation, but the role model is currently `user | assistant`, not a durable Human/Agent participant identity model.

### Current sidebar shape

The rendered sidebar is still explicitly Agent-oriented (`aria-label="Agents"`, `ConversationAgentSummary`, Agent actions). FBCP must evolve the existing row/domain shape; adding a second Human sidebar would violate the target architecture.

### Shared Room is not yet Human identity

Shared Room/group code has Agent-member IDs and remote Agent ownership fields. It is the correct group/member owner to extend, but it is not evidence that a native Fabushi Human identity or Human member already exists.

### External channels are not the native network

The existing channel connector path models outside platforms and delivery addresses. It is not the Fabushi-owned durable messaging/sync network required by FBCP.

## P0 gaps that remain after this inventory

1. Complete recursive Telegram source/dependency/resource closure at the frozen upstream commit.
2. Complete the full Telegram capability graph and source-informed behavior dossiers.
3. Resolve every capability to one of the owners above or a minimal ADR-backed new owner.
4. Define native identity/auth, durable messaging, sync, presence, media, call-signaling and push contracts without creating a second product architecture.
5. Define the incremental Conversation/Participant/Message/Resource model changes for the first Human vertical slice.
6. Prove the first vertical slice with focused contracts, integration/E2E, and exact-head packaged acceptance on allowed infrastructure.

## Refresh rule

Before using this inventory for implementation, compare the live PR #20 head to `dcb19a94383833fc1ec5074f10c4bbbd28c09036`. If it changed, re-run the source inspection and replace stale owner evidence before coding.
PR #20 rebaseline 2026-10-02: exact delta c2767eac..95995bdf was inspected. It changes only the Grok parity manifest, Host Transcript runner_registry/transcript_manager production ownership, and their focused contracts. It closes the former runner-registry blocker inside the same Host Transcript owner; it does not move FBCP sidebar/account/settings/media/notification/security owner roots. The later exact delta 95995bdf..de0f1749 was also inspected and changes only `source/host/src/selected_image_inputs.rs`, restoring native HEIC/HEIF-style ISO-BMFF image dimension/rotation parsing. The subsequent de0f1749..bbc7b34a delta adds only the focused `source/host/tests/transcript_send_echo_contract.rs` parity coverage for HEIC/HEIF/AVIF primary-item rotation. The exact delta bbc7b34a..dcb19a94 changes only `projects/grok-fabu-parity/architecture-manifest.json`, accepting the send-message-shaping parity row; it does not move any FBCP product owner. The dcb19a94 exact-head renderer run `37017583630` fails the strict gate with 41 remaining `existing-needs-parity` rows, first `source/host/extensions/transcript/send-pipeline.ts`. This strict-gate evidence is independent from FBCP owner resolution and does not turn manifest status into production implementation evidence.
