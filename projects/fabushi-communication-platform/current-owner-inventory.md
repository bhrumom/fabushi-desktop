# FBCP P0 — Exact-HEAD Current Owner Inventory

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Status: evidence snapshot; P0 not yet passed
Current bounded refresh: round 22, inspected base `8ce0a9a43b7fe94df2ec2085399fe65a57343577`; FBCP-001 r8 / TDRP-001 r10 / FMEP-001 r2. The older captures below are historical.
Project: FBCP-001 Revision 2
Captured: 2026-10-03
Main input: `d9ae2773f2c517a0cb911e7b7bc996905cf3ada4`
PR #20 input: `a538679392bc09531041030a98fa0880107e7d40`
PR state at capture: open, draft, base `main`, head `refactor/grok-018-architecture-rebuild`

This file records the current production owners that FBCP must absorb into. It is not an implementation-complete claim. Any later PR #20 HEAD invalidates the source conclusions below until this inventory is refreshed.

## Method

### Round 18 — actual CLI Host and packaged runtime consumers

Frozen production input `1b0e674b3e39292ae715a3eb50435d83e2cb8577`. CLI and gateway own
`RuntimeHandle(MahayanaHost)`; their ordinary dependency explicitly selects
`desktop-full` and `local-only`. NativeEngine/NativeAgentBackend and the existing
local MCP owner retain lifecycle and command authority. Status/provider names
alone were compatible with an unavailable fallback; real MCP assertions and
single-root production binary/process checks now guard that composition.
No FFI dependency, new runtime, provider, public schema or external ABI removal
was added. The package workflow remains the final-byte owner and the unchanged
promotion consumer remains the installer-member authority.

The reviewed desktop path is Electron entry -> utilityProcess carrier ->
Coordinator child process -> AppHost child process, using MessagePorts/stdio/HTTP.
The complete selected non-Rust executable-surface scan at this H found no actual
target C ABI caller; its1780 paths/1769 distinct blobs and exclusions are recorded
in the caller index. Retained C exports and Dart/iOS comments do not identify a
tracked caller or prove external absence. Keep historical SQLite, credentials
and compatibility readers pending actual external-client evidence.

Exact-H Electron/UI/Windows and promotion6 passed; Rust jobs were still queued at
the recorded observation. Actual CLI binary/MCP/process and signed installer
acceptance remain open. See the round18 delivery receipt and its exact run/blob
identities. Concurrent Session/Transcript/forward/reply/resource owners are
preserved; no message or account authority moves into the CLI or renderer.

### 2026-11-02 immutable forward picker intent (task20bf round1 resumed)

Current inspected input is `59f5fa24827812f78336c464995d161f403dedfb` on the
existing migration branch; main remains `5528062177e3865eef60a073a066cca144f9e3e8`.
The existing transcript action opens `ForwardMessageDialog`; its callback goes
through the Coordinator Session gateway to `ProductionSessionWorkers::forward_human_message`.
The existing source-conversation `HumanForwardBatch` journal binds account,
issuer, source, normalized privacy and original targets before the first POST;
destination SandAgentDb rows plus the authenticated Fabushi service settle the
messages. Same-nonce subset retries are allowed, expansion/privacy changes are
rejected. No renderer state can authorize or own these sends.

This batch changes the existing picker intent and settlement projection only:
immutable retry parameters, exact requested-target confirmation, the Host's
32-recipient limit, reverse privacy transition and close-with-durable-work
semantics. The already integrated account-generation and roster/unmount fences
remain. Search, resources, approvals, Agent execution and old data retain their
current owners. The historical statement that desktop Telegram/FeatureHost
writers remain reachable is superseded by the current Native guards below and
their later changes; no old registration or database is deleted here.

Oracle: ORA-TDRP-FORWARD-INTENT-UI-001. Exact-H execution is recorded by the
existing native-service-contracts Actions artifacts; source reviews and final
service/package/Bot/independent acceptance remain separate.

### 2026-11-03 Windows updater publisher authority (round 22; narrow owner)

At base `8f089dd8640dacb4ec39105aa6d43e4c547ac5a2`, production main-process update wiring constructed the existing `SandWindowsInstaller` without a publisher override, accepting inherited Anysphere common names by default. Neither product metadata nor historical signer names establishes formal Fabushi publisher authority. The round-22 implementation at `5d271ff0c03d8f01368ca44026d14cd7519ad6bd` retains the existing `entry.ts` → production binding → `update-wiring.ts` → `SandUpdateService` → `win32-installer.ts` → apply-marker/Settings path; it removes the default, requires trusted `FABUSHI_WINDOWS_UPDATE_PUBLISHERS_JSON` exact certificate subject plus DER SHA-256, checks valid embedded Authenticode and candidate bytes, and consumes the launch receipt once. Missing/invalid policy disables updates; persistent data and staged failure recovery remain untouched. No new updater, identity owner, or renderer authorization surface is created.

The `windows-publisher.contract.test.mjs` 28-case and `windows-publisher.native.test.mjs` 11-case suites plus valid-format wrong AppHost hash negative are **authored but not yet executed at the containing H**. The native suite uses an ephemeral CI-only signer and inert fixture; it is never proof of authorized Fabushi publication. Scope and exact base runs: `projects/telegram-desktop-rust/evidence/round22-windows-publisher-authority.json`. Formal publisher approval, actual signed release, installation/rollback, asset provenance and independent ACCEPT remain unqualified.

The generic social provider may still read/write outside Session/Transcript while recovered records participate in Host startup. Its specific producer/consumer and retained-data contracts must be fully traced before any change: no deletion of provider registration or data under this updater batch.

### 2026-11-02 Explicit Electron update source (round 21)

The shipping chain remains `entry.ts` →
`createElectronProductionUpdaterInstallerBinding` →
`createProductionUpdaterInstallerBinding` → `createUpdateServiceWiring` →
`SandUpdateService`; the existing status IPC feeds the Settings update projection.
EUS-01–06 removes the inherited Cursor default from this actual owner. Main-process
`FABUSHI_UPDATE_FEED_BASE_URL` takes precedence over an explicitly set compatibility
variable, including a present-empty primary value. The existing
`normalizeFabushiApiBaseUrl` authority validates the origin; no new origin registry,
feed deployment, updater, account store or renderer configuration writer is added.
Missing/invalid sources stay disabled and metadata redirects are refused.

The existing apply-marker owner still settles a prior installation attempt before
the disabled-startup return; settings, marker schema and all user data retain their
existing owners. Sixteen focused contracts cover these paths plus valid-feed,
cancellation and staging behavior through controlled ports. They are authored,
not locally executed; the containing-H Electron/UI Actions must qualify the change.
Formal feed availability, publisher trust (including the historical Windows signer
allowlist), download provenance and actual signed upgrades/rollback remain open.
The partial upstream audit in `evidence/round21-update-source-authority.json`
does not turn the Windows workflow mapping into full updater product acceptance.

### 2026-11-02 Native startup and message authority refresh

Shipping launch is `source/electron-main/entry.ts` → production coordinator
ports/executors → `rust-coordinator-carrier.ts` → Node Coordinator `spawn_host`
→ `source/host/app/src/main.rs::main` → `UnifiedAppHost::new` → `AppHost::new`.
Electron's account adapter sets Native product mode and the private credential
file. At the inspected base, round 14 has removed automatic Telegram secret
provisioning, SQLite open and provider handoff. Old secure-store
entries, SQLite and auxiliary files are retained without conversion or deletion.

| Responsibility | Observed Native path / state authority | Remaining boundary |
| --- | --- | --- |
| Identity | Fabushi service identity → Electron `FabushiAuthService` → atomic Native v1 credential; Rust `native_desktop_session` is a fresh read-only projection | Old descriptor optional Telegram fields stay readable for upgrade compatibility; they cannot activate a client. |
| Conversation | `ProductionRenderer` / Session gateway → `ProductionSessionWorkers::create_human_conversation`; canonical participant-pair identity and existing conversation SQLite | Group/channel and the separate FeatureHost messaging path are not unified by this batch. |
| Message | Session `append_human_message_with_context` / `sync_human_conversation` → `FabushiNativeMessagingClient` → `/api/social/messages`; stable local transcript/outbox plus service remote IDs | NMR-34 refuses standalone FeatureHost execution in Production+Electron before parsing/locks/I/O; retained and non-desktop state is not imported or accepted. |
| Resource | Session attachment materialization/upload → `/api/social/message-resources` and existing local resource paths/provenance | Production+Electron FeatureHost blob reads refuse before I/O; existing blob/upload bytes remain available for reviewed rollback. |
| Permission | Native credential/issuer/user binding, participant-pair admission and service enforcement before local settlement | Standalone FeatureHost access issuance is refused in Production+Electron; its unscoped access registry is retained, not canonical Native authorization. |
| Search | Renderer `searchHumanMessages` → Session `search_human_messages` → the same authorized transcript | The standalone engine/SearchIndex cannot be constructed through desktop FeatureHost messaging dispatch; non-desktop/canonical cross-surface search acceptance remains open. |

The AppHost methods remain stable rejection surfaces: explicit
`feature.messaging.execute/blob.read/access.issue` and wrapped/unwrapped typed
`feature.execute` / `messaging.execute` reach the same FeatureHost guard.
The actual Human Composer does not select them. Retained account-scoped
`runtime/accounts/<account-hash>/agents/_messaging/{snapshot.json,blobs}` and
unscoped `runtime/agents/_messaging/access.json` remain byte-preserved. Automation
discovery/override/write/cleanup excludes `_messaging`; it is not an Agent owner.
NMR-36 removes Host's explicit legacy provider injection: direct Rust and
`mahayana-ffi::mahayana_runtime_create` reject either legacy client or self-id
before product configuration, storage or runtime construction. Compatibility
fields remain decodable; the C ABI returns its existing zero/error result.
The Host no longer directly depends on the Telegram adapter (NMR-37).
NMR-38 removes the absent AppHost lease and unreachable forwards; old proxy
projection types are only test inputs. Production Normal/Build graph and actual
executable symbols must be qualified at the containing commit. The twelve
legacy auth routes retain explicit errors and do not become another writer.

`mahayana-ffi` still force-links direct Telegram C APIs, and `mahayana-cli`
actually consumes its generic RuntimeHandle ABI. Electron's positive launch
path selects the `source/host/app` executable through process IPC. NMR-39
limits its resources/bin copy to the four staged executable names/Windows
forms and excludes nested files. It retains original staging contents and
all data/readers; no external ABI/mobile or signed-package absence is inferred.

Base `adf15caadac5775fdd8582227cad3bbf9c93cf0f` passed run `41447812811`,
including Host6/C ABI7 and the direct dependency negative. NMR-38/39 tests and
binary/copy gates require their own exact-H Actions. Original storage open is
not a forensic read-only operation; rollback checks use synthetic/authorized
copies. See `evidence/round16-apphost-package-boundary.json` for source identities
and the unqualified backend/whole-product boundaries.

The inventory was built from the exact PR #20 HEAD, starting at shipping entrypoints and following the production composition paths. Directory/name similarity by itself is not evidence. The paths below are the current owners or production seams that are actually composed by the renderer, Electron main, Coordinator, Host, and Runner.

Refresh through `a538679392bc09531041030a98fa0880107e7d40` revalidated the live PR #20 production ownership. The exact `dcb19a94..a5386793` delta is 13 commits and 19 production/test/manifest files, confined to Host Transcript (`production_runtime`, `roster_emit`, `runner_registry`, `send_pipeline`, `send_turn_dispatch`, `session_runtime`, `workflow_commands`), Host Runner (`mod.rs`, `turn_run_shell.rs`), shipping Host composition, and Coordinator inference routing plus focused contracts. It does not move the renderer shell/sidebar/workspace/composer, Electron account/settings/media/notification roots, Shared Room, attachments, search, permissions, Computer, MCP, Automations, or storage owner roots. The delta strengthens the existing Transcript/Runner/Coordinator owners rather than creating a parallel communication owner.

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
| Calls / call signaling | No dedicated canonical production call/session owner exists at this exact head; Computer remains the existing screen/control integration surface | inspected `frontend/src/recovered/features`, `source/host/src`, `source/host/src/extensions`, `source/electron-main`; owner resolution `CALL-REALTIME`; `docs/adr/FBCP-ADR-003-call-session-signaling.md` | `existing_owner = none` only for call-session/signaling state. ADR-003 specifies the minimal new owner; Computer/platform remain reused integration owners. This is an approved P0 owner resolution, not an implemented call product. |

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

Before using this inventory for implementation, compare the live PR #20 head to `a538679392bc09531041030a98fa0880107e7d40`. If it changed, re-run the source inspection and replace stale owner evidence before coding.

PR #20 rebaseline 2026-10-03: the exact `dcb19a94..a5386793` delta was inspected file-by-file at the ownership boundary. It advances send-pipeline/send-turn dispatch, workflow commands, runner registry/turn shell, Coordinator inference routing, and Session runtime behavior inside the same canonical Host Transcript/Runner/Coordinator architecture. `session-runtime.ts` now has a production Rust owner with focus/contact freshness, live-session ownership, deferred bounded-open activation/catch-up, snapshot/roster activation wiring, boot fallback ordering evidence, and focused contracts; `a5386793` is the first head that marks that row implemented, and only its own Actions may accept it. No FBCP product owner root moved in this delta.

### Resumed round 6: existing Human history owner extension

NMR-PAGING-01 uses `SandAgentDb` and `agent_db_transcript_pages` for both canonical full reads and durable bounded traversal indices; `ProductionSessionWorkers` and the existing gateway admit the scoped cursor. The existing renderer pagination/root-cache/projection owners carry it and preserve committed updates across failed older-page retries. NMR-ACTIVITY-01 keeps activity and Agent handoff in the existing per-conversation mutation lane and repairs interrupted row/metadata persistence on replay. No additional message owner or user-data migration is introduced. Later credential, reply/forward, legacy-retention and source work remains intact. These authored changes require containing-H Actions; this inventory is not an acceptance claim.



### Round23 resumed: desktop Social ownership boundary

At inspected106bb95ba97ef7722e9523285783824393039bc0, Production Electron
AppHost selects the existing Session host policy; ordinary/Codex registries
omit generic Social. FeatureHost rejects contact ConversationOpen/ChatSend
before auth/runtime/state/I/O/events; Runtime retains nonempty Social records
before recovery writes, and the existing desktop platform client blocks generic
message/resource HTTP before credentials. Human message writes remain in the
existing Session/native_messaging path. No owner or user-data migration is added.
The named NMR-SOCIAL-OWNER spec adds three missing real FeatureHost dispatch
controls and keeps existing Agent, automation, non-desktop and Test boundaries.
Exact-H execution remains required; no whole-owner acceptance is promoted.

### Recovery round15: retained forwarding source discovery

The actual Session forward/sync entry continues through
`ProductionSessionWorkers`, source `SandAgentDb` journals and the existing Native
HTTP client. Initial request lookup and restart recovery inspect retained source
journals before dispatch and report missing/unreadable identity, so unavailable
data cannot release a requestId for another source. Own revoked-source denial,
other-account isolation and explicit Human retry keep their existing owners.

One added process/HTTP/SQLite case raises the current Human gate from67 to68.
Existing CREATE01..04 creation14/storage3 retain all current positive/negative
oracles. Their immutable negative main-format check uses a private pager and
propagates all schema_version errors; the peer's real retained-WAL and
cross-process lock cases remain.
Missing/zero main with any retained -wal/-shm/-journal refuses before any open;
zero/no-sidecar creation remains compatible. No independent main-file
descriptor, readonly_shm path, new FFI or alternate initializer is added.
Arbitrary corrupt-WAL/hot-journal/crash preservation remains unqualified.

This update is recorded in the root inventory. The two dated round26/28 sections
previously present only in the same-named nested copy are retained below; that
nested path now links here. Separately named P0 captures remain dated evidence,
not an additional current state authority.


### 2026-11-03 Forward journal recovery and storage admission (round 28)

At the inspected base, renderer `forwardHumanMessage` calls the existing
Session gateway and `ProductionSessionWorkers`, constructed by the actual
AppHost entry. Source `SandAgentDb` metadata owns the whole forwarding intent;
destination transcript/outbox owns local delivery, and the existing Native
HTTP service owns remote message IDs. Ordinary sync discovers the same journal.
This round adds expected-key-value CAS to that metadata writer, preventing a
different batch from being erased by an earlier read/merge snapshot. Source
policy denial persists on unfinished targets and does not stop unrelated
history; storage failure still fails closed. No owner, public schema or
service API is added. Five process/HTTP/SQLite integration cases and one CAS
regression are authored, not-run at preparation. Remote source permission and
provenance, attachment retention and simultaneous multi-AppHost cross-source
uniqueness remain unqualified. See the round28 evidence receipt and exact-H
Actions artifacts; prior round27 claims of five existing tests are withdrawn.

### 2026-11-02 Forward account and dialog lifetime (round 26)

The actual forwarding entry is ProductionRenderer Transcript action → existing
ForwardMessageDialog → coordinator searchHumanRecipients/forwardHumanMessage
→ authenticated Gateway allowlist → Session gateway/ProductionSessionWorkers
→ the same native client and SandAgentDb transcript/outbox. Electron account
observation remains identity authority; renderer accountScopeGenerationRef is
an invalidation projection, not a second identity or persistence owner.
NMR-46 binds source selection, asynchronous search/send and subsequent roster
projection to that generation. Dialog close/source change/unmount invalidates
its own lifetime; renderer unmount invalidates the same generation in layout
cleanup. No durable send is cancelled or undone by suppressing stale UI.

At inspected H, run41496846317 passed Social9 and HTTP-owner6; the retained
Social provider's queued/running/waiting recovery must remain intact. Gateway
HTTP1 proves Bearer authentication and allowlisted method reachability with a
TestApi echo; Human33 exercises the actual Session service boundary separately.
It is not an end-to-end real production forwarding proof. AppHost copy4 is
three fixtures plus one real copied binary; Windows native11 uses an ephemeral
CI-only signer. The historical authored-only statements below are superseded
for those bounded scopes. Full signed product/real service/source/legal/ACCEPT
remain blocked. Whole-destination-plan durability and remote forward provenance
remain separate gaps in the existing owners. See `projects/telegram-desktop-rust/evidence/round26-forward-account-lifetime.json`.

