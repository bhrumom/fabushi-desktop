Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@b0d1fe5e08c963402b8b71648c70a2bf68e6fab2 (root tree e279373aec283ebfcc6e3bb7385e51205acd84a2). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,824; first unread=5,825 Telegram/SourceFiles/test/test_style.cpp@9fe31e6e846bb3546ee5d1a8c9a4afeab8c26bdd; unread=10,301; unknown=15,846; omitted=0. Immediate upstream delta 811b83a1→b0d1fe5e modifies deterministic order 5,166 Telegram/SourceFiles/media/view/media_view_overlay_widget.cpp only; its new e35f6711 blob was re-read/reconciled and removes viewer-driven automatic Save(ToCacheOrFile) for forbidsSaving messages. The earlier order 5,951 chat_theme_readability.cpp delta remains unread/unknown. Runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 remain historical-only for 863cf10d+b4d9f7f8; a8a139b8 runs 37967712462/37967712518 are predecessor-only 811b83a1 evidence.

# Telegram 源码 → Fabushi 全量等价重写 — Source of Truth

Status: active  
Project ID: TDRP-001  
Revision: 9  
Date: 2026-10-07  
Parent: FBCP-001 Revision 7

## Role

本项目负责完整 `telegramdesktop/tdesktop` 源码及其递归依赖、资源、工具链的逐文件、逐模块理解、责任对照、等价重写、生产接入和 exact-head 取证。它不是 capability research 子项目，也不能以 survey、ledger 填写、局部 vertical slice 或 UI Demo 作为最终完成。

上游是完整产品行为、状态机、边界条件与工程责任的 source authority，不是最终 runtime/network/product。全部 applicable 产品职责必须吸收入 current canonical Fabushi；C++ 非 UI 产品逻辑默认 Rust，UI 使用 Fabushi React/TypeScript 表现层，平台职责采用最薄 adapter。UI 文件中的业务、权限、生命周期、错误处理、键盘与可访问性行为仍然必须迁移。

## Must read

- root `AGENTS.md`
- `projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md` Revision 7
- `docs/specs/fabushi-bot-communication-platform.md` Revision 7
- `docs/specs/telegram-desktop-rust-equivalence-migration.md` Revision 9
- `module-map.md`
- current upstream lock / inventory / ledger / dossiers / validators / quality plans
- current canonical Fabushi shipping owners and GitHub Actions release evidence

较早 Revision 2/4/5 中 research-only、选择性吸收、where accepted、blocked 可算最终验收、htch-runtime 可执行本项目验证等口径全部废止。历史记录只能保留 provenance，不能继承为 implemented/verified。

## Baseline

Historical predecessor authority: 2026-10-09 live rebaseline 后 tdesktop accepted discovery HEAD 为 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e（tree 5030985204963cbbd362ced7412d231b04ebd0cc）；42f8a36d43b8c805bc821905bea4cfeb3af1d41d 以及更早 baseline 仅作历史证据。本次三提交 delta 修改 15 个 root paths（12 modified / 3 added），root non-directory 6,651，recursive denominator 16,123。当前 source closure 仍 open，baseline_ready/acceptance.accepted 仍必须为 false。

accepted baseline 必须在 GitHub Actions 中重新确认，并递归闭合：
- root tracked files / gitlinks；
- nested submodules；
- build-time downloads / patches；
- LFS / external objects（如有）；
- generated-input contracts；
- functional resources / locales / assets；
- build / packaging / updater / tests / tooling / docs / licenses。

只有当 current lock、inventory、ledger、dossiers 对同一 exact upstream commit 一致，且 unknown=0、unread=0、omitted=0，才能把 `baseline_ready` 设为 true。旧 baseline 的 verified 状态不得自动继承。

## Mandatory chain

`source file -> source symbol -> responsibility -> capability -> existing Fabushi owner -> target path/symbol/language -> production entrypoint -> tests -> current-head evidence -> verified`

允许多源文件汇聚到一个高内聚 owner，也允许一个 mixed 文件拆到多个 owner；禁止机械一文件对一文件复制。任何 responsibility 无去向都不能完成。

## Existing-owner-first

每项 responsibility 都必须先审计 current exact-head Fabushi：
- ProductShell / navigation / ConversationList / ConversationWorkspace / Transcript / Composer；
- Participant/Profile / Shared Room/member / Permissions / Identity/account / Contacts / Search；
- Resource / attachments / artifacts / storage / recovery / notifications / settings；
- Coordinator / Host / Runner / Agent；
- Plugins / MCP / Marketplace / Computer / Automations / Tasks；
- networking / platform / packaging / updater。

有合适 existing owner 时必须扩展它。只有 exact-head audit 证明没有 owner，且记录 owner-absence evidence、rejected candidates、最小 state/lifecycle responsibility、commands/events、persistence/concurrency/security/service dependency 并经 ADR 批准后，才能新增 source-neutral 最小 owner。

禁止新增 TelegramCore、TelegramRuntime、TelegramProvider、TelegramMessaging、CommunicationCore、MessengerRuntime 或第二套 Identity / Conversation / Message / Search / Settings / Marketplace。

## Product invariants

- 最终只有一个 Fabushi Desktop 产品。
- Human / Agent / Group / Channel / Hybrid / Topic 共用 canonical Conversation 架构、ConversationWorkspace、Transcript、Composer、Draft、routing 与 resource lifecycle。
- Telegram 的 reply/quote/forward/edit/delete/reaction/poll/scheduled/silent/read/pin/expiry/album 等能力优先升级 canonical Message/Transcript model，不建立来源型 message store。
- Participant / Contacts / Profile 使用统一 framework；联系人发消息进入 canonical Direct Conversation。
- Conversation creation 只有一个 typed `ConversationCreationSurface`。
- Search 只有一个 canonical owner；collection/object/universal search 与 participant/member/admin pickers 复用同一 provider/permission contract。
- 新迁移 UI 只消费 Fabushi semantic tokens 与 canonical components；legacy `sand-*` / `cursor-*` 只能作为兼容实现细节。
- Telegram/MTProto 不进入 shipping runtime；但 ordering/replay/gap/dedupe/idempotency/durability/reconnect/retry/backoff/resume/session/rate-limit/cancellation/security 等成熟责任必须在 Fabushi 自有协议与服务中等价实现。
- 远端 service 缺失只能标 blocked；mock、fixture、UI-only 或 static response 不能算完成。
- 现有 Agent / Host / Coordinator / Runner / Plugins / MCP / Marketplace / Computer / Automations / Task / attachments / approvals / recovery / account / packaging/update 不得回退。

## Verification

所有 FBCP/TDRP 可执行 build/test/lint/generator/schema/benchmark/fuzz/E2E/visual/a11y/performance/soak/fault/package/acceptance 只允许 GitHub Actions。

每个 responsibility 的 Definition of Done 同时要求：
- production implementation 已进入唯一 shipping composition；
- requirement/oracle/invariant/test 可追溯；
- exact-head GitHub Actions evidence；
- 无 flaky/skipped/blocked/not-run 冒充 pass；
- packaged temporal acceptance 验证 terminal result 在 quiet window、切换、reconnect、reload、restart 后不消失、不回滚、不被 intermediate 替代。

release candidate 只有在 independent acceptance 为 ACCEPT、0 open P0/P1/blocker、0 omitted/unmapped、0 stub/fake fallback 后才可发布。packaged acceptance 必须 fail closed；不得用 `continue-on-error`、跳过步骤或软化 assertion 绕过。

## Current status

迁移尚未完成。当前 durable specs 已升级到 FBCP Revision 7 / TDRP Revision 9，但历史 lock / inventory / ledger / dossiers / STATUS / validators 中仍可能包含旧 Revision 4/5 语义，必须逐项 rebaseline/reconcile，不能据此宣称 source completeness 或 production completeness。

每个模块继续执行 `unreviewed -> understood -> mapped -> implemented -> verified` 的真实证据路径。遇到 service/account/signing blocker，记录解除条件并继续推进所有不依赖该 blocker 的 responsibility。

Historical predecessor authority: Current live authority (2026-10-09): `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (root tree `5030985204963cbbd362ced7412d231b04ebd0cc`), three commits ahead of historical `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`; 15 root paths changed (12 modified, 3 added), recursive denominator is 16,123, read-through is 5,773, unread is 10,350, unknown is 15,844, omitted is 0, and source closure remains open.

Current source accounting: deterministic read-through `5599/16123`; unread `10524`; unknown `15844`; unknown-closed `279`; omitted `0`. Orders 5001-5599 are exact-blob read-complete. Media-view/menu responsibilities and MTProto-derived transport/session/auth/config/security/proxy/error/schema/reconnect/bootstrap responsibilities remain mapped-open except for explicitly cited existing partial Fabushi slices; MTProto wire/socket/DC mechanics are source-neutral platform/protocol replacements, not a second runtime and not omitted. Unknown stays unchanged until complete responsibility and exact-head verification gates close; no baseline-ready or release credit is granted.


### Read-through 5291-5300

Orders 5291-5300 are exact-blob read-complete at upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`. WebProxy transport/WebView invariants are mapped-open to existing Coordinator/Host/backend/desktop-bridge security, lifecycle, bounded-backpressure and recovery owners; no MTProto/WebProxy runtime is introduced. Overview selection/media layout responsibilities are mapped-open to canonical Message/SearchIndex/MediaCache, Conversation/shared-media collection, MediaViewer/Resource and design-system Checkbox/ResultRow owners; no Telegram/Overview UI root is introduced. Unknown remains 15,841 and omitted remains 0 until production + exact-head evidence closes each responsibility.


### Read-through 5301-5310

Orders 5301-5310 are exact-blob read-complete at upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`. Passport-derived secure authorization responsibilities are mapped-open to current source-neutral account/identity, provider authorization/OAuth, credential/security, recovery, resource upload and canonical form owners. Telegram Passport protocol/crypto/UI is not introduced as a second runtime. Unknown remains 15,841 and omitted remains 0.


### Read-through 5311-5320

Orders 5311-5320 are exact-blob read-complete. Secure form/panel/scan responsibilities remain mapped-open to canonical account/identity, provider authorization, attachment/resource, recovery, form-validation and design-system owners. No Telegram Passport UI/runtime is introduced. Unknown remains 15,841; omitted remains 0.


### Read-through 5321-5330

Orders 5321-5330 are exact-blob read-complete. Passport detail/form/password responsibilities plus checkout lifecycle are mapped-open to canonical account/identity/provider authorization, attachment/resource, form validation/design-system and existing payment/payment-provider owners. No Passport or Telegram payment runtime is introduced. Unknown remains 15,841; omitted remains 0.

### Read-through 5331-5340

Orders 5331-5340 are exact-blob read-complete. Payment orchestration, balance/reaction fencing and provider tokenization responsibilities are mapped-open to native Mahayana payment/payment_provider/wallet/service, canonical auth/external-URL/security, and production reaction owners. Telegram Stars/MTProto/SmartGlocal runtime is not introduced. Unknown remains 15,841; omitted remains 0.

### Read-through 5341-5350

Orders 5341-5350 are exact-blob read-complete. Provider card/token/error decoding and Stripe request lifecycle responsibilities are mapped-open to canonical payment-provider/security/error owners; provider wire implementations are source-specific and are not copied. Unknown remains 15,841; omitted remains 0.


### Read-through 5351-5360

Orders 5351-5360 are exact-blob read-complete. Stripe card metadata/input validation/decode/error responsibilities are mapped-open to canonical PaymentProvider/payment, form-validation, security and error-projection owners. Stripe SDK/wire behavior is not copied as a second runtime. Unknown remains 15,841; omitted remains 0.


### Read-through 5361-5370

Orders 5361-5370 are exact-blob read-complete. Provider form encoding/configuration/token decode and checkout card-editor behavior map to canonical PaymentProvider/payment, security, form-validation and canonical payment UI owners. No Stripe runtime or Stripe-specific UI root is introduced. Unknown remains 15,841; omitted remains 0.

### Read-through 5371-5380

Orders 5371-5380 are exact-blob read-complete. Requested customer information, payment-field normalization, checkout summary and panel orchestration map to canonical payment, form, webview/security and command owners. Commit `39da06252e33ef7f8d53061514f8234108ee609e` adds fail-closed source-neutral invoice-requested customer validation before charging plus a focused Rust contract; exact-head Actions and remaining UI/provider responsibilities stay open. Unknown remains 15,841; omitted remains 0.

### Read-through 5381-5390

Orders 5381-5390 are exact-blob read-complete. Paid-reaction amount/identity/anonymity/balance maps to canonical Reaction/Wallet/payment/Dialog; Linux location, XDG Open-With, activation/sleep/lock/theme and protected update/relaunch map to existing platform and update/packaging owners. Unknown remains 15,841; omitted remains 0.

### Read-through 5391-5400

Orders 5391-5400 are exact-blob read-complete. Global menu/unread badge/focus state, native notification capability negotiation/actions/inline reply/activation-token/exact-scope cleanup, DBus/Flatpak schemas, autostart/single-instance/scheme launch and platform capability deltas map to existing application-menu, notification, deep-link, lifecycle, settings and packaging owners. Native Linux services remain thin adapters. Unknown remains 15,841; omitted remains 0.


### Read-through 5401-5410

Orders 5401-5410 are exact-blob read-complete. Linux text-recognition availability, external translation-provider process lifecycle, tray icon/menu/cache behavior, protected updater security, WebAuthn passkey routing and macOS authorization-aware location/reverse-geocode behavior map to existing source-neutral capability/platform owners. The protected updater remains high-risk mapped-open until trusted-path/package/staging/privilege/post-install verification obtains exact-head evidence. Unknown remains 15,841; omitted remains 0.

### Read-through 5411-5475

Orders 5411-5475 are exact-blob read-complete at accepted upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` after correcting deterministic order 5464-5475 against the live root tree. The range contains macOS platform responsibilities through order 5463, followed by shared `platform_*` contracts for current location/reverse geocode, file bookmarks/utilities/dialogs/external launch, Integration composition, launcher/MainWindow dispatch, notification policy, MediaViewer overlay/window controls and platform lifecycle/permission/security settings. Windows-specific files begin at order 5481 and are not credited by this range. All responsibilities remain mapped-open to existing source-neutral canonical owners until production implementation and same-head evidence close them; reading alone does not reduce unknown. No Telegram-derived parallel runtime or platform-named product UI was introduced. Unknown remains 15,841; omitted remains 0.


### Read-through 5476-5500

Orders 5476-5500 are exact-blob read-complete at accepted upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`. Shared platform text-recognition/translation/tray/WebAuthn/window-title contracts and Windows exact location, file/Open-With/dialog security, lifecycle/power fencing, updater/relaunch parent identity, locked-window privacy, exact-scope native notification actions, screenshot/settings/autostart integration, native OCR/translation capability absence, tray/taskbar behavior and Windows Hello wallet protection have been decomposed and mapped to existing source-neutral canonical owners. Windows native OCR/translation absence is recorded as a capability delta rather than an excuse to invent parallel providers. Existing Fabushi updater and scoped-notification slices are credited only as partial production evidence; inline notification reply/mark-read, locked-content/capture privacy, Windows Hello wallet equivalence and other mapped responsibilities remain open. Unknown remains 15,841; omitted remains 0; reading alone grants no production/release credit.


### Read-through 5501-5529

Orders 5501-5529 are exact-blob read-complete at accepted upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`. Windows Hello registration timing, native WebAuthn/fallback routing, AppUserModelID/shortcut/toast identity, autostart policy, safe dynamic symbols, Quiet Hours, taskbar media controls and toast activation map to canonical security/packaging/settings/media/notification owners. Poll option-link and token-fenced media upload responsibilities expose a current production gap: no canonical poll/survey owner was found, so these rows remain mapped-open rather than being credited to generic Composer attachments. Profile header/block/drop-area presentation maps to canonical ProfileSection/design-system; settings globals map to canonical settings/persistence/security/lifecycle, including passcode retry backoff and working-directory permission semantics. Unknown remains 15,841; omitted remains 0.


### Read-through 5530-5540

Orders 5530-5540 are exact-blob read-complete. Away Message, Chat Intro, Business Chat Links, delegated Chatbot permissions, Greeting and Business Location are applicable product responsibilities rather than Telegram-only UI mechanics. Current Fabushi repository search found no equivalent production owners, so these rows are explicit mapped-open product gaps under source-neutral business automation, conversation onboarding/link/deep-link, Agent/Bot CapabilityBroker and profile/location owners. Elevated delegated-bot permissions require explicit warnings; scheduling/recipient/shortcut limits and map capability/fallback semantics remain fail-closed. Unknown stays 15,841; omitted stays 0.


### Read-through 5541-5571

Orders 5541-5549 are exact-read business quick-reply, recipient-scope, shortcut-message and working-hours responsibilities. They enforce premium/count/name/message limits, include/exclude invariants, per-day interval normalization including next-day ranges, and timezone fallback/loading; no equivalent complete canonical owner has yet been proven. Orders 5550-5571 are the cloud-password/account-security flow: transient StepData, password create/check/change, recovery email, email/login-email codes, password hint constraints, reset/pending-reset/cancel-reset, manage/disable, 10-minute idle expiry, and cross-device password-change invalidation. Repository search did not prove an equivalent complete 2SV/recovery state machine. These remain critical mapped-open security responsibilities; unknown stays 15,841 and omitted stays 0.


### Read-through 5572-5580

Orders 5572-5580 are exact-blob read-complete. Detailed settings rows explicitly carry Switch/checkable accessibility semantics. Active Sessions manages current/incomplete/other authorizations, 60-second refresh, device naming, terminate-one/all and auto-terminate TTL. Advanced settings span source-neutral network/proxy, download/storage, auto-download, title/frame, tray/taskbar/close behavior, autostart, updater, power/archive and platform capability surfaces. Blocked Peers is a canonical privacy list with reactive server-backed state. These rows remain mapped-open until same-head product/security/a11y evidence proves equivalence; unknown remains 15,841.

### Rebaseline 42f8a36d -> 863cf10d

Live root tree `5030985204963cbbd362ced7412d231b04ebd0cc` has 6,651 non-directory entries; recursive total is 16,123. Deterministic read-through is 5,599. First unread is order 5,600 `Telegram/SourceFiles/settings/sections/settings_local_storage.cpp` blob `b5371a40e0dce0d21583ca6a026bcd7df43ab15f`. Unknown is 15,844 and remains fail-closed.

Changed prior-read blobs were re-read at their 863cf10d exact blob identities and re-decomposed. New `bot_menu.tgs` and `history_view_bot_menu_button.cpp/.h` are read-complete but mapped-open to canonical Composer + Button/IconButton + animation/resource owners. No source-named BotMenuButton product component is allowed. `chat.style` (order 5,941) and `wallet_content.cpp` (order 6,314) remain outside deterministic read-through; delta semantics are mapped-open without granting prefix credit.

### Read-through 5590-5593

Orders 5590-5591 are exact `settings_credits.cpp/.h` blobs `dbe0b4cf48e52c30e2066c6ad4181893ed604646` / `56c88e68b16b4c2c49bcbcc846b9a5969dfb4867`. They expose account-scoped Stars/currency balance projection, top-up option loading, subscriptions and transaction history, receipt routing, statistics/gift/affiliate actions, TON/USD presentation and withdrawal availability/action. These remain mapped-open to canonical Wallet/Payments/Credits + billing/history/earnings owners; no Telegram Stars runtime is introduced.

Orders 5592-5593 are exact `settings_folders.cpp/.h` blobs `8c5b9b4d6b7a5b3166c12ac8e20c50775a84e9dd` / `8a2984febbf9753dd4aeb698eb7924d4960ca19a`. They expose canonical conversation-filter create/edit/remove/restore/reorder behavior, shared-chat-list leave suggestions and peer removal, recommended filters and limits, premium-gated color tags with 500 ms debounce plus teardown flush, and vertical/horizontal/tab display preferences. Server mutations are intentionally ordered and locally reconciled; these remain mapped-open to the existing Conversation collection/filter + Settings + account messaging-service owners.

Unknown stays 15,844 and omitted stays 0 because source reading alone does not prove production closure. Deterministic read-through is 5,593/16,123; first unread is order 5,594 `Telegram/SourceFiles/settings/sections/settings_global_ttl.cpp` (`3a22266764ef7fe1496a9fd89d5853afe93bd34a`).

### Read-through 5594-5595

Orders 5594-5595 are exact `settings_global_ttl.cpp/.h` blobs `3a22266764ef7fe1496a9fd89d5853afe93bd34a` / `6484c81b829bb59278f5480034c93d7e4796863c`. This is the global conversation-retention/auto-delete settings contract: server-authoritative default history TTL, first-enable confirmation, predefined/custom periods, eligibility-filtered existing-conversation selection, and per-peer application through independent history-TTL mutations. Self/replies/verify-code/ineligible peers are excluded and current per-peer TTL is projected in the picker.

The responsibility remains mapped-open to canonical Privacy/Retention + Conversation policy + account messaging owners. Production evidence must cover account switching, stale peer eligibility, permission changes, server rejection/retry, bulk partial success/failure, duplicate application, default-vs-per-conversation reconciliation, teardown/cancellation, and keyboard/focus/a11y/light-dark/responsive behavior. Unknown stays 15,844; omitted stays 0. Read-through is 5,595/16,123; first unread is 5,596 `settings_information.cpp`.

### Read-through 5596-5597

Orders 5596-5597 are exact `settings_information.cpp/.h` blobs `8982b515b5a30600f5565c3b7e81dc5218c96950` / `54435faa2f1a8d156d5da3501a9e9b0c13b96bae`. Responsibilities are canonical Identity/Profile and Account projections: profile photo optimistic local preview followed by upload, name/username/phone and copy/edit routing, birthday plus privacy state, personal channel/color, Chat Automation, Bio editing with premium/default limits and one-second debounce plus teardown flush, multi-account add/switch/new-window/logout/reorder/limit handling, and account/premium/unread badge projections.

These remain mapped-open to existing source-neutral Identity/Profile/Account/Business/Privacy owners. Exact-head production evidence must cover failed/stale avatar upload rollback, bio save races and remote replacement, account switch guard/new-window continuity, reorder persistence, locked-account limits, logout fencing, birthday privacy, chatbot/personal-channel account scope, unread/badge state, and keyboard/focus/a11y/light-dark/responsive behavior. Unknown remains 15,844; omitted remains 0. Read-through is 5,597/16,123; first unread is 5,598 `settings_local_passcode.cpp`.

### Read-through 5598-5599

Orders 5598-5599 are exact `settings_local_passcode.cpp/.h` blobs `0f1b6ace690851441a588074cbff816fb3e5a8a1` / `b0292c43e503bc8b7ad2fa66f9681be59c1c272c`. This is a local-device security domain, distinct from Cloud Password/2SV. It retains verified passcode bytes only inside the passcode settings area with a 60-second interaction-reset inactivity expiry, invalidates retained bytes on external passcode changes, performs derivation/verification on worker paths, tracks wrong attempts, and cleanses transient UTF-8 buffers. Create/check/change/manage navigation hands secure bytes only across the immediate settings hop and removes stale passcode sections from navigation history.

Manage semantics include wallet key-protection revalidation/migration before passcode removal, fresh verification for writes that weaken app lock, refusal/stale recovery back to Check, app-lock launch protection, auto-lock settings, and platform system-unlock availability/toggles including Windows Hello, Touch ID, Apple Watch, or system password. These remain mapped-open to canonical Local Security/App Lock + Wallet Key Protection + platform-unlock owners; they do not close Cloud Password/2SV. Unknown remains 15,844; omitted remains 0. Read-through is 5,599/16,123; first unread is 5,600 `settings_local_storage.cpp`.


### Read-through 5600-5604

Orders 5600-5604 are exact `settings_local_storage.cpp/.h/.style` and `settings_main.cpp/.h` blobs. Local Storage owns account-local dual-cache statistics, category/all clearing, device-capacity projection, coupled total/media cache quotas and retention policy; clearing is duplicate-fenced and settles only after both databases plus the minimum presentation interval. Settings Main composes active-account profile plus canonical Notifications/Privacy/Chat/Folders/Advanced/Devices/Power/Language/Wallet/Business/Help routes, add-account/logout, scale confirm/restart, validation/support actions and account-scoped refreshes.

All responsibilities remain mapped-open to source-neutral owners. Production closure must cover failure/cancel/retry/idempotency/account-switch/stale-result/teardown/reload/restart/recovery plus keyboard/focus/a11y/light-dark/responsive/reduced-motion evidence. Unknown remains 15,844; omitted remains 0. Read-through is 5,604/16,123; first unread is 5,605 `settings_notifications.cpp` (`a16c3ee9690de934d795a2803ada0e82c3f9db94`).


### Read-through 5605-5613

Revision 9 validation correctly rejected the transient branch mapping that placed Passkeys at order 5608. Exact recursive order is now restored: 5605–5607 Notifications, 5608–5609 Reactions, 5610–5611 Notification Type, 5612–5613 Passkeys.

Notifications covers global/app policy projection, multi-account cleanup, privacy preview, badge/event/call settings and native/custom platform-manager configuration. Reactions adds server-backed None/Contacts/All scopes for message reactions and poll votes plus preview privacy. Notification Type owns private/group/broadcast defaults for mute, sound/tone/volume and exact peer exceptions. Native notification reply/mark-read/open still requires canonical exact account/conversation/topic-or-sublist/message authorization and exact cleanup; existing scoped click-only OS manager is partial, not closure.

Passkeys is an Account Authentication product flow: server registration challenge → bounded platform authenticator/WebAuthn → server finalize, with unsupported/unsigned/cancel failure closed and exact server credential-id deletion. Fabushi already has remote Agent WebAuthn proxy/signer infrastructure, but that is not evidence of an Account Passkeys UI/list/create/delete owner, so this remains mapped-open.

Accounting: read-through 5,613/16,123; unread 10,510; unknown 15,844; omitted 0. First unread is 5,614 `settings_premium.cpp` (`60ac3cbbfcd4b74eda0d3cbca46c969a22878b94`).

### Current live source accounting after 5614–5616

Accepted upstream remains `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (tree `5030985204963cbbd362ced7412d231b04ebd0cc`). Deterministic read-through is **5,616 / 16,123**; unread **10,507**; unknown **15,844**; omitted **0**. Orders 5614–5616 remain mapped-open to canonical Entitlement/Subscription, Wallet/Payments/Commerce, Settings and design-system owners. First unread is 5617 `settings_privacy_security.cpp`.

### Read-through 5614–5616

Premium entitlement/subscription/commerce and presentation are exact-blob read-complete and mapped-open; no production closure is credited.

### Read-through 5617–5622

Privacy/Security composition, editable keyboard shortcuts, and server-backed Website/Bot authorization sessions are exact-blob read-complete and mapped-open to existing canonical Account Security/Privacy, Authorization, Command Registry, Bot/Blocked-Peer, Payments/Retention and Settings owners. Exact authorization hash, account/session fencing, destructive confirmation/retry/idempotency and keyboard/a11y behavior remain production gates. Accounting: read-through 5,622; unread 10,501; unknown 15,844; omitted 0. First unread: 5623 `settings/settings_builder.cpp`.

### Read-through 5623–5632

Settings Builder/Common/Search/section-factory infrastructure, hidden diagnostic codes, and Credits/Gift commerce graphics are exact-blob read-complete. UI infrastructure maps to canonical Settings/UniversalSearch/design-system owners; diagnostic codes require controlled development/support policy rather than product copying; Credits/Gift actions remain mapped-open under canonical Wallet/Payments/Credits + Gift/Commerce with exact-account/id, cancel/retry/idempotency/rollback/settlement/restart gates. Accounting: read-through 5,632; unread 10,491; unknown 15,844; omitted 0. First unread 5633 `settings_experimental.cpp`.

### Read-through 5633–5642

Experimental flags/support, FAQ suggestion loading, Settings layer/focus/keyboard navigation, notification-common presentation and Power Saving are exact-blob read-complete. They map to canonical feature-flag policy, Help/Support, Settings/design-system navigation, Notification Settings and Performance/Power owners. No source-named product UI or parallel state owner is introduced. Accounting: read-through 5,642; unread 10,481; unknown 15,844; omitted 0. First unread 5643 `settings_power_saving.h`.

## Deterministic source-order correction at 5,623 and live read-through 5,653

Accepted recursive tree `5030985204963cbbd362ced7412d231b04ebd0cc` proves `Telegram/SourceFiles/settings/settings.style@8b9e56dd…` is blob order **5,623**, between `sections/settings_websites.h` (5,622) and `settings_builder.cpp` (5,624). The earlier 5,623–5,642 labels were therefore off by one and are superseded. Their shard/attestation artifacts are replaced rather than treated as parallel authority.

The corrected contiguous batches are 5,623–5,633 (settings.style through Credits/Gift graphics), 5,634–5,643 (Experimental/FAQ/Settings shell/navigation/Power Saving implementation), and 5,644–5,653 (Power Saving interface, privacy controllers, recent Settings search, scale preview, Settings search/type). Current accounting is **5,653 / 16,123 read**, **10,470 unread**, **15,844 unknown**, **0 omitted**. Reading alone closes no unknown. First unread is 5,654 `Telegram/SourceFiles/statistics/chart_lines_filter_controller.cpp@40c3b612…`.

## Deterministic statistics batch 5,654–5,665

Exact blobs for line-filter animation, ruler generation, interactive chart widget, range min/max segment tree, statistics style/common types and chart JSON deserialization are read/decomposed. They map to canonical Analytics/Insights data projection + reusable Chart/design-system owners; business/account truth must not live in the chart. JSON parse errors, empty columns and column-length mismatches fail closed. Hover/zoom/filter/footer state is derived and lifetime-bounded. Accounting: **5,665/16,123 read; 10,458 unread; 15,844 unknown; 0 omitted**. First unread: 5,666 `statistics_format_values.cpp`.


## Deterministic statistics formatting/export batch 5,666–5,674

Exact blobs for locale/timezone analytics labels, TON/Credits chart graphics, analytics-to-sheet projection, 64-bit chart values, and XLSX/OpenXML serialization are read/decomposed. They map to canonical Analytics/localization, Wallet/Credits visual projection and Export owners. Export must preserve inline-string formula-injection safety, XML/control escaping, legal unique sheet names, empty/short-series behavior, numeric/date boundaries, ZIP failure, cancellation/account fencing and cross-platform save behavior. No Telegram-derived statistics/export runtime or UI is introduced. Accounting: **5,674/16,123 read; 10,449 unread; 15,844 unknown; 0 omitted**. First unread: 5,675 `statistics/view/abstract_chart_view.cpp@4b5f2929…`. Reading alone closes no unknown.


## Deterministic statistics chart-view batch 5,675–5,684

Exact blobs for chart-view abstraction, double-line ratios, bar/stack bars, animated dual-axis/currency rulers, closed chart-type dispatch and linear/DPR cache + pointer hit-testing are read/decomposed. They remain mapped-open to one canonical Analytics/Chart design-system owner. The Fabushi port must strengthen empty/one-point, zero-range/max, all-disabled, mismatched-series, invalid-type and pointer-endpoint cases fail closed; rendering state owns no analytics truth. Accounting: **5,684/16,123 read; 10,439 unread; 15,844 unknown; 0 omitted**. First unread: 5,685 `statistics/view/stack_chart_common.cpp@82a1143f…`.


## Deterministic statistics stack/pie batch 5,685–5,690

The remaining statistics/view blobs are read/decomposed: stack geometry/index mapping, filtered pie-percentage rounding, and stack-linear local zoom/pie transition/selection. They remain mapped-open to the canonical Analytics/Chart owner. Canonical code must fail closed for <2 points, zero-width ranges/denominators, invalid indices/line lengths, zero totals and stale/racing zoom/filter/hover state. Accounting: **5,690/16,123 read; 10,433 unread; 15,844 unknown; 0 omitted**. First unread: 5,691 `statistics/widgets/chart_header_widget.cpp@b8754ceb…`.


## Deterministic statistics widgets batch 5,691–5,696

Statistics widgets are exact-blob read/decomposed: responsive chart header, line-filter controls with last-visible-line guard, and localized/currency point details with zoom/cache/ripple state. They map only to canonical source-neutral Analytics/Chart + existing design-system controls. Empty/mismatched data, invalid indices, RTL/long text, currency precision, a11y/focus/theme/DPR and teardown remain production gates. Accounting: **5,696/16,123 read; 10,427 unread; 15,844 unknown; 0 omitted**. Next order 5,697 must be rebound from the accepted recursive tree before recording its path.


## Deterministic PCH + storage security/migration batch 5,697–5,701

Order 5,697 maps C++ PCH/platform composition to canonical cross-platform build evidence. Orders 5,698–5,701 map critical durable local-file protocol, Argon2id/scrypt/legacy passcode KDFs, encryption/signature/version checks and versioned settings/session migrations to canonical Persistence + Local Security/Key Protection + Settings Migration owners. Invalid/over-cost KDFs, corrupt/truncated/unknown blocks, interrupted writes, wrong passcodes and legacy/account/restart recovery must fail closed. This does not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. Accounting: **5,701/16,123 read; 10,422 unread; 15,844 unknown; 0 omitted**. First unread: 5,702 `storage/download_manager_mtproto.cpp@d0007eb1…`.


### TDRP Revision 9 source read 5702-5711 — storage transfer lifecycle

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,702-5,711 cover adaptive remote download scheduling, generic cache/file loader lifecycle, part-based download/resume, HTTP(S)-only web transfer and multipart media upload. Canonical mapping is the existing source-neutral messaging attachment/file-transfer owner plus platform download/save adapters; MTProto/DC/CDN details are protocol-specific and must not create a Telegram runtime. Required parity includes cancel/teardown/account-switch stale-result fencing, partial resume, integrity/reference refresh, redirect downgrade/scheme protection, TLS/auth/disk failures, adaptive concurrency, removed-session resend, transcode/archive cancellation, upload progress/finalization and restart/recovery. Accounting: 5,711/16,123 read, 10,412 unread, 15,844 unknown, 0 omitted; reading alone closes no unknown. First unread: 5,712 `Telegram/SourceFiles/storage/localimageloader.cpp`.

### TDRP Revision 9 source read 5712-5721 — media preparation and durable serialization

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,712–5,713 cover source-neutral media-send preparation and worker/result lifecycle; 5,714–5,717 cover local settings/theme/language/update persistence, legacy encrypted migration and bounded serialization primitives; 5,718–5,721 cover versioned document/media and user/chat/channel projection persistence with legacy location migration and fail-closed corruption handling. Canonical mapping stays with existing messaging Attachment/media-preparation, Persistence/Settings/Local Security/Updater/Theme/Localization, messaging data persistence, and Identity/Profile/Conversation owners. Telegram/MTP file formats and flags are migration inputs only and must not become a parallel runtime or product UI. Required closure includes cancel/account-switch/teardown stale-result fencing, malformed media/encode/archive failures, wrong-key/tamper/truncation/future-version handling, interrupted atomic writes, legacy migration, cache-vs-authority reconciliation, reload/restart and canonical UI/a11y evidence. Accounting: **5,721/16,123 read, 10,402 unread, 15,844 unknown, 0 omitted**; reading alone closes no unknown. First unread: **5,722 `Telegram/SourceFiles/storage/storage_account.cpp@5888ffa1164327b7170a2a3d8df7793a3fa467da`**.

### TDRP Revision 9 source read 5722-5729 — account/domain storage and local security

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,722–5,723 cover encrypted account-scoped drafts/cache/settings/trust/payment/webview/bot/wallet persistence; 5,724–5,725 cover bounded cloud-blob download/extraction; 5,726–5,727 define passcode wraps, bounded memory-hard derivation, secret cleansing, fresh verification tokens, App Lock mutation, crash-safe committed generations, legacy migration and wallet keyring protection; 5,728–5,729 route shared-media/profile-photo cache queries and invalidation. The local-security rows strengthen but do not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. All remain mapped-open to canonical source-neutral owners; no Telegram storage/security runtime or UI family is permitted. Production evidence must cover wrong/unsupported KDF, stale verification, passcode/App Lock mutation, crash/restart migration, wallet corruption/absence, account fencing, delayed-write teardown, cloud/archive failure, cache invalidation and canonical UI/a11y. Accounting: **5,729/16,123 read, 10,394 unread, 15,844 unknown, 0 omitted**. First unread: **5,730 `Telegram/SourceFiles/storage/storage_file_lock.h@181b5bc063827458c4e59a0daa167355f835ae0b`**.

### TDRP Revision 9 source read 5730-5740 — file locking, archive/media preparation and sparse indexes

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,730–5,732 define exclusive storage locking on POSIX/Windows, including conflict-process handling; 5,733–5,734 define folder/multi-file ZIP preparation with symlink exclusion, progress/cancel, size/disk failure and stale temporary cleanup; 5,735–5,736 cover Composer/editor media drag/drop, MIME/limit/decode and asynchronous preview preparation; 5,737–5,740 define shared-media categories and sparse message-id range merge/query/count/invalidation. All remain mapped-open to canonical Persistence/platform adapters, Attachment/Media Editor and Shared Media/index owners. Required evidence includes cross-platform lock conflict/retry, archive cancellation/limit/disk failure/cleanup, malformed media and stale async results, and sparse pagination/count consistency. Accounting: **5,740/16,123 read, 10,383 unread, 15,844 unknown, 0 omitted**. First unread: **5,741 `Telegram/SourceFiles/storage/storage_user_photos.cpp@232befde40c5edea34286105374058fcc66ab658`**.

### TDRP Revision 9 source read 5741-5754 — user-photo cache, streamed download and operator support

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,741–5,742 cover profile-photo history cache list/count/paging; 5,743–5,744 cover bounded aligned streaming-part download with duplicate suppression, cancellation and completion; 5,745–5,754 cover operator/support template autocomplete/search, contact confirmation, shortcut navigation, occupation leases, support-info editing, history preload, fast-button preferences and template update/reload. Support-specific code is conservatively mapped-open pending explicit applicability evidence rather than silently omitted. Required evidence includes profile count/paging consistency, part failure/cancel/staleness, support account/history fencing, edit request replacement, occupation expiry/race, preload retry and malformed/update-failed template recovery plus canonical keyboard/focus/a11y. Accounting: **5,754/16,123 read, 10,369 unread, 15,844 unknown, 0 omitted**. First unread: **5,755 `Telegram/SourceFiles/tde2e/tde2e_api.cpp@efe4edc5eef735c954b39d7a927722a7e8f9214c`**.

### TDRP Revision 9 source read 5755-5758 — call end-to-end encryption

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,755–5,758 own call E2EE temporary/private key lifecycle, one-shot ECDH envelopes, encrypt/decrypt callbacks, participant/state blocks, inbound/outbound queues, subchain reconciliation, failure terminality and emoji verification hash, with protocol conversion as an adapter. They remain mapped-open to canonical Calls E2EE/Crypto + Call Session owners; transport TLS is not a substitute. This implementation batch also hardens the canonical Wallet ledger so reuse of an idempotency request id must match operation kind, accounts, normalized currency, amount and reference; conflicting credit/transfer/refund reuse fails closed with `RequestConflict` and unit coverage. Broader wallet rows remain open pending full production/evidence closure. Accounting: **5,758/16,123 read, 10,365 unread, 15,844 unknown, 0 omitted**. First unread: **5,759 `Telegram/SourceFiles/test/README.md@429b4f3d83ec6fa6aa71203624a894d1d7973d35`**.

### TDRP Revision 9 source read 5759-5771 — professional task-test harness

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,759–5,771 define the upstream task-test authority: pure bounded readiness; exact object/action identity; assertions outside waits; diagnostic timeouts; precondition-only N/A; packed-scenario teardown; protected fixture secrets; deterministic activation/focus and animation timing; exact dialog-shell interaction; visual capture of real paint owners with DPR/scale boundaries; and privacy-safe clipboard classification/retry without logging foreign clipboard contents. These map to Fabushi's professional cross-platform GitHub Actions test/evidence harness, not to production Telegram UI. The same descendant repairs current-head authority failures by correcting order 5,739 to accepted blob `cc0e3ed6b2c763326c2c16635a7671a747ef506b` and refreshing the strict Grok semantic-adaptation target for `source/electron-main/attachments/attachments.ts` to production blob `9fb1ed46b2ae623e7fa781d6d3ab47f465f4a332`; no gate is weakened. Orders 5,772-5,773 cover the Windows console-lock professional-test contract: the implementation reads process-session/WTS metadata and classifies locked/unlocked/unknown, while the header fixes the public NotApplicable/Unlocked/Locked/Unknown state model, makes `locked()` true only for Locked, and requires `ConsoleLockGate` to remain empty for Unknown and NotApplicable. Only a positive locked reading gates; documented legacy-Windows reversed flags remain unknown; diagnostics never read or print user/domain/window-station fields; non-Windows is explicit N/A. This maps to the canonical professional cross-platform test/evidence harness and creates no production Telegram UI. Accounting: **5,773/16,123 read, 10,350 unread, 15,844 unknown, 0 omitted**. Reading alone closes no unknown. First unread: **5,774 `Telegram/SourceFiles/test/test_corner_patch.cpp@a34785a050cdeea405c3948345704d2e19395843`**.



### cf478d37 live-authority rebaseline

Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

Direct delta: channel_earn.style replaces static negative placeholder margins with floating placeholderShiftLeft; lib_ui 91ff446 implements horizontal interpolation in InputField and MaskedInputField. Canonical TextField remains the sole product owner. Orders 5,774-5,793 were identity/order reconciled; reading alone closes no unknown.


### Revision 9 live read-through note: 5794-5801

Orders 5,794-5,797 preserve professional evidence-log one-line/completion-forgery integrity plus an independent raw-byte oracle. Orders 5,798-5,799 preserve complete mapped-target/viewport capture readiness. Orders 5,800-5,801 preserve the bounded reversible not-marking-read evidence lever. These are test/evidence responsibilities only, create no second product owner, and close no unknown. The new 811b83a1 readability delta is order 5,951 and remains unread/unknown.


### Revision 9 live read-through note: 5802-5805

Orders 5,802-5,803 close the exact-blob read/decomposition of popup/context-menu professional evidence semantics: same-turn fresh-menu identity, explicit refusal taxonomy, isolated QAction queued-callback delivery, prepared-frame capture and lock/teardown behavior. Orders 5,804-5,805 close the SentMessageWatcher client→server id reconciliation contract with stable-history/candidate fencing and a five-second diagnostic probe throttle. These are test/evidence responsibilities only and close no unknown. First unread is now 5,806 `test_notify_override.cpp`.

### Revision 9 live rebaseline/read-through note: b0d1fe5e / 5166 / 5806-5813

The 811b83a1→b0d1fe5e upstream delta changes only order 5,166 `media_view_overlay_widget.cpp`: the old `forbidsSaving()` control-refresh branch hid controls and could call `DocumentSaveClickHandler::Save(...ToCacheOrFile)` when media was not loaded; b0d1fe5e removes that viewer-driven automatic persistence path. The new blob `e35f67119323850a3cea450548566186c8a2231c` was re-read and the mapped-open MediaViewer lifecycle responsibility now explicitly requires protected/forbids-saving media not to be auto-persisted as a rendering/control-refresh side effect. Path order and total counts are unchanged. Separately, orders 5,806–5,813 are read-complete/responsibility-decomposed professional test harnesses for reversible notify overrides/no-server-write locality, document-open handoff inspection without OS launch, SeparatePanel pointer/liveness evidence, and post-paint temporal sampling. They add no product owner, fake fallback, unknown closure or release credit. First unread is 5,814 `test_probe.cpp`; read-through is 5,813/16,125, unread 10,312, unknown 15,846, omitted 0.


### Revision 9 live read-through note: 5814-5821

Orders 5,814–5,821 are exact-blob read-complete/responsibility-decomposed professional evidence harnesses. Probe preserves mark-bounded, keyed issue→answer correlation with explicit ambiguity/outstanding states and positive controls; controlled RPC fixtures reject malformed/truncated/trailing/wrong-constructor buffers before live parser delivery; retry evidence distinguishes 500 retry registration from 400 fail/unregister and unknown-request controls while logging only code/type/constructor; Runner preserves bounded stage/watchdog execution, explicit N/A, QPointer/paint readiness, exactly-once onFinish teardown, disposable-copy markers, completion drain and post-quit fuse behavior. These rows add no product owner or fake fallback and close no global unknown. First unread is 5,822 `Telegram/SourceFiles/test/test_scenario.cpp`; read-through is 5,821/16,125, unread 10,304, unknown 15,846, omitted 0.

### Revision 9 live read-through note: 5822-5824

Orders 5,822–5,824 are exact-blob read-complete/responsibility-decomposed professional evidence code. The checked-in scenario slot is deliberately no-op/test-only. The secrecy oracle is fail-closed over accepted launch identities, controls, bidirectional MTP headers and canaries; distinguishes client-written plain/Send from report-only Recv, withholds unsafe sites, narrowly handles declared computed fields, emits only public evidence and rescans its own rows. Missing/foreign/unreadable/control/canary/input failures are undecided rather than clean. These rows close no global unknown and create no product owner. First unread is 5,825 `Telegram/SourceFiles/test/test_style.cpp`; read-through is 5,824/16,125, unread 10,301, unknown 15,846, omitted 0.