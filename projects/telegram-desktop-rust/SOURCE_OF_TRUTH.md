Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (root tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6). Root non-directory=6,653; recursive non-directory=16,125; read-through=6,853; first unread=6,854 Telegram/ThirdParty/MicroTeX::src/core/macro_def.cpp@0ccf9fe8b4028f97c62834555d2a3d344d0f8726; unread=9,272; unknown=15,845; omitted=0. The 28ac5769→6fed91ff rebaseline changes ten existing root blobs with no path-count/order change; all ten changed blobs inside the read prefix are explicitly re-read/responsibility-reconciled. MicroTeX order 6,853 remains unchanged. Fresh descendant exact-head GitHub Actions evidence is required.
<!-- TDRP_CURRENT_SUMMARY read-through=6853 unread=9272 unknown=15845 omitted=0 first-unread=6854 path=Telegram/ThirdParty/MicroTeX::src/core/macro_def.cpp -->

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

### Revision 9 live read-through note: 5825-5830

Orders 5,825–5,830 are exact-blob read-complete/responsibility-decomposed UI evidence harnesses: stable-window style/baseline verification, real InputField text-drop delivery/refusal/ancestor shielding with no clipboard-content access, and narrowly-scoped U+00A0/U+202F text-read normalization with deliberate negative controls. They create no production owner and close no global unknown. First unread is 5,831 `Telegram/SourceFiles/test/test_toast_capture.cpp`; read-through 5,830/16,125, unread 10,295, unknown 15,846, omitted 0.


### Source closure through 5,848

Under live `65e23ba7` authority, orders **5,837–5,848** were completely read and exact-blob dispositioned. Accounting is now **5,848/16,125 read, 10,277 unread, 15,846 unknown, 0 omitted**; these test/evidence-only rows do not reduce global unknown. Fresh exact-head Actions are required.


### Source closure through 5,859

Under live `65e23ba7` authority, orders **5,849–5,859** are now exact-blob read-complete/responsibility-decomposed. Tests 5,849–5,854 are professional evidence-only responsibilities; Tray 5,855–5,859 remains mapped-open production work pending canonical source-neutral owner/composition and exact-head evidence. Accounting: **5,859/16,125 read, 10,266 unread, 15,846 unknown, 0 omitted**. First unread is 5,860 `ui/boxes/about_cocoon_box.cpp@4aab8e79…`. Unknown is not reduced by reading alone.


### Source closure through 5,871

Orders **5,860–5,871** (`ui/boxes`) are exact-blob read-complete/responsibility-decomposed. They remain mapped-open product/UI responsibilities; reading does not reduce unknown. Current accounting: **5,871/16,125 read; 10,254 unread; 15,846 unknown; 0 omitted**. First unread 5,872 `choose_font_box.cpp@56a7386c…`. Canonical owner/applicability evidence remains mandatory.


### Source closure through 5,909

The deterministic live prefix is exact-blob read-complete/responsibility-decomposed through **5,909**, completing `ui/boxes/**`. Accounting is **5,909/16,125 read; 10,216 unread; 15,846 unknown; 0 omitted**. First unread is 5,910 `ui/cached_round_corners.cpp@7da69a22…`. The predecessor d52b8c5 authority failure identified a stale `upstream.lock.json.source_disposition_evidence.read_through`; this descendant fixes that data contract without modifying or weakening the validator. Canonical production owner, UI/UX, backend/platform dependency and same-head evidence remain mandatory.


### Source closure through 5,980

Orders **5,910–5,980** are exact-blob read-complete/responsibility-decomposed. Accounting is **5,980/16,125 read; 10,145 unread; 15,846 unknown; 0 omitted**. First unread is 5,981 `ui/color_contrast.cpp@f981e717…`. Parent `9c969d1` repaired the validator-required narrative tokens; this batch preserves that exact contract. Canonical production owner, UI/UX, backend/platform dependency and same-head evidence remain mandatory.


### Source closure through 5,991

The deterministic prefix is read/decomposed through **5,991**. Color contrast/serialization maps to canonical semantic-theme utilities; button busy/context-menu/two-label behavior maps to canonical Button/IconButton/ContextMenu without source-named components. A visual disabled state does not replace command-level duplicate-execution fencing. Reading closes no unknown. Accounting: **5,991/16,125 read; 10,134 unread; 15,846 unknown; 0 omitted**.


### Source closure through 6,001

The deterministic prefix is exact-blob read/decomposed through **6,001**. Call action/mute UI maps to existing Human call + canonical Button/IconButton owners; service check maps to Checkbox/theme; compose-AI/large-paste maps to canonical Composer/attachment/AI owners; custom emoji Toast maps to Toast + emoji/media resolution. The current production slice canonicalizes Human call buttons and puts duplicate mute refusal plus rollback in the command owner, not presentation state. Unknown is unchanged because the complete responsibilities are not yet verified. Accounting: **6,001/16,125 read; 10,124 unread; 15,846 unknown; 0 omitted**. First unread is 6,002 `ui/controls/delete_message_context_action.cpp@9cecf76e…`.


### Source closure through 6,011

Orders **6,002-6,011** preserve delete TTL countdown/user-vs-expiry callback separation, aggregate download progress/finished navigation projection, dynamic image strip pointer-intent + cyclic keyboard selection, and emoji loading/panel geometry/suggestion lifetime semantics. Canonical owners are ContextMenu/Message lifecycle, Downloads/Resource, source-neutral Picker/Avatar rows, and IconButton/Composer/TextField/Popover. Exact applicability, domain ownership and executable evidence remain open, so global unknown stays **15,846**. Accounting: **6,011/16,125 read; 10,114 unread; 0 omitted**; first unread 6,012 `ui/controls/feature_list.cpp@8c3bf36c…`.

### Source closure through 6,032

Orders **6,012-6,032** are exact-blob read/decomposed for feature/detail rows, filter/share header state, invite-link actions/ContextMenu, jump-down unread projection, labeled emoji Tabs, location-picker service/UI boundaries, participant LoadingState skeletons and Popover lifecycle. All map to source-neutral canonical owners; reading does not reduce unknown. The descendant Human-call shipping surface also serializes camera/screen-share mutations through one `video-media` command fence with canonical pending projection, rollback and capture teardown. ForceMuted/RaisedHand/scheduled/audio-reactive call presentation remains mapped-open. Accounting: **6,032/16,125 read; 10,093 unread; 15,846 unknown; 0 omitted**. First unread 6,033 `ui/controls/round_video_recorder.cpp@b52b73bd…`. No baseline/release credit is granted.

### Source closure through 6,041

Orders **6,033-6,041** are exact-blob read/decomposed for round-video recording/encoding, sender identity, the primary Composer send-state control and silent-send accessibility. Round-video remains mapped-open as a real device/media/codec lifecycle, not a cosmetic widget. The descendant Composer now also removes its raw HTML send button and reuses canonical `SandIconButton`. Accounting: **6,041/16,125 read; 10,084 unread; 15,846 unknown; 0 omitted**. First unread is 6,042 `ui/controls/stars_rating.cpp@7b7dce6e…`.

### Source closure through 6,052

Orders **6,042-6,052** are exact-blob read/decomposed for Stars rating, source-neutral tabs/subsection reorder and swipe gesture/scroll ownership. The existing canonical `SandTabs` owner is extended with optional overflow visibility, context-menu requests, locked reorder boundaries, pointer cancellation/edge-scroll and Alt+Arrow keyboard reorder; no Telegram-derived Tabs owner was added. Stars account/reputation truth, complete Avatar/Badge subsection composition and the navigation/conversation swipe owner remain mapped-open. Accounting: **6,052/16,125 read; 10,073 unread; 15,846 unknown; 0 omitted**. First unread is 6,053 `ui/controls/tabbed_search.cpp@ca207d0b…`. Reading alone does not reduce unknown.

### Source closure through 6,068

Orders **6,053-6,068** are exact-blob read/decomposed for grouped search/category picking, detail rows and transient tooltip lifetime, title/status chrome, localized fixed-precision amount input/IME, ephemeral-media countdown presentation, and Avatar/profile-media acquisition/upload/privacy/streaming lifecycle. The existing canonical `SandTooltip` gains optional auto-dismiss, outside-press/Escape dismissal and focus-return policy with a focused contract test; no Telegram-derived Tooltip owner was added. Search/Picker shipping composition, full profile-media ownership, monetary-domain applicability and authoritative ephemeral-message lifecycle remain mapped-open. Accounting: **6,068/16,125 read; 10,057 unread; 15,846 unknown; 0 omitted**. First unread is 6,069 `ui/controls/who_reacted_context_action.cpp@6851dee0…`. Reading alone does not reduce unknown.

### Source closure through 6,084

Orders **6,069-6,084** are exact-blob read/decomposed across who-read/reaction context actions, platform-obsolescence and screen-reader-mode Banners, country selection, dynamic media thumbnails, animated text, and credits/commerce graphics/style. Canonical owners remain source-neutral and all rows remain mapped-open unless independently closed; reading does not reduce unknown. Accounting: **6,084/16,125 read; 10,041 unread; 15,846 unknown; 0 omitted**. First unread is 6,085 `ui/effects/drifting_particles.cpp@ed119cf9…`.


### Source closure through 6,104

Orders **6,085-6,104** are exact-blob read/decomposed across decorative particles, reaction/custom-emoji fly overlays, fireworks/glare/LoadingState skeletons, Composer-to-Transcript message-send transitions, Stars particles/Story outlines, Premium commerce styling and an optional GPU 3D commerce cover. The source responsibilities require pause/teardown correctness, reduced-motion/power-policy handling, target-identity fencing, theme/DPR/RTL/responsive behavior, resource completion recovery and GPU capability fallback. They map only to source-neutral canonical animation/LoadingState/Transcript/Resource/Avatar/Status/commerce/design-system owners; no Telegram-derived parallel visual runtime is accepted. Accounting: **6,104/16,125 read; 10,021 unread; 15,846 unknown; 0 omitted**. First unread is 6,105 `ui/effects/premium_3d_mesh.cpp@b8541b31…`. Reading alone does not reduce unknown or grant implementation/release credit.


### Source closure through 6,125

Orders **6,105-6,125** are exact-blob read/decomposed across validated 3D asset loading, GPU support gating, reactive credits/limit/subscription presentation, coin/diamond RHI renderers, promo particle strategies and the interactive Premium Star lifecycle. Domain truth stays in canonical commerce/credits/gifts/subscription owners; graphics reuse canonical controls and the thinnest visual/GPU adapters. Malformed assets, unsupported RHI, shader/buffer/pipeline failures, power-saving/reduced-motion, pause/resume and partial-init teardown remain explicit failure/lifecycle obligations. Accounting: **6,125/16,125 read; 10,000 unread; 15,846 unknown; 0 omitted**. First unread is 6,126 `ui/effects/premium_star_model.cpp@499d751a…`. Reading alone grants no implementation or release credit.


### Source closure through 6,145

Orders **6,126-6,145** are exact-blob read/decomposed across Premium Star assets/particles/GPU renderer, colored collectible decoration, responsive commerce top-bar capability fallbacks, reaction fly presentation, Checkbox/Avatar selection rendering, scroll-edge shadows, and send-action status animations. Send-action types include record/upload/round/speaking/choose-sticker with typing fallback; same-family restart and speaking finish/restart transitions remain visual projections over authoritative conversation state, and animation-disabled mode resolves to static frames. All responsibilities map to source-neutral canonical owners; reading does not reduce unknown. Accounting: **6,145/16,125 read; 9,980 unread; 15,846 unknown; 0 omitted**. First unread is 6,146 `ui/effects/shake_animation.cpp@38fbd6f0…`.


### Source closure through 6,165

Orders **6,146-6,165** are exact-blob read/decomposed across shake/LoadingState skeleton feedback, snowflake/star-burst decoration, the message-removal dissolve capture/collapse/GPU compute lifecycle, disclosure-arrow affordance and TTL timer-icon projection. The dissolve path is explicitly presentation after authoritative deletion: it capability-gates power/RHI/compute, pre-captures exact Transcript items, reconciles collapse gaps and scroll baselines, bounds GPU particles/frame delta, and tears down pending/per-item resources; it never owns deletion truth. Accounting: **6,165/16,125 read; 9,960 unread; 15,846 unknown; 0 omitted**. First unread is 6,166 `ui/effects/unique_gift_message_bubble.cpp@22802623…`.


### Read-through 6166-6171

Accepted Telegram authority remains `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`. Orders **6,166-6,171** are exact-blob read and responsibility-decomposed: unique-gift message bubble geometry maps to canonical TranscriptEntry/Avatar plus Gift/Commerce presentation; upload progress lifecycle maps to canonical Resource/attachment upload + Status/Progress/IconButton; voice-once particles map to canonical voice-message playback presentation with reduced-motion/teardown policy. Reading alone does not reduce unknown. Accounting after orders 6,166-6,171 was **6,171 / 16,125 read**, **9,954 unread**, **15,846 unknown**, **0 omitted**. The next unread order was **6,172** `Telegram/SourceFiles/ui/empty_userpic.cpp`.

### Read-through 6172-6182

Accepted Telegram authority remains `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`. Orders **6,172-6,182** are exact-blob read and responsibility-decomposed: fallback identity presentation maps to canonical Avatar/semantic Icon/Resource owners; filter-icon selection maps to canonical Picker/Popover/Menu/IconButton with the existing conversation-filter domain owner; grouped-media geometry maps to the canonical TranscriptEntry/Resource layout utility and must remain deterministic and single-owned. Reading alone does not reduce unknown. Accounting after orders 6,172-6,182 was **6,182 / 16,125 read**, **9,943 unread**, **15,846 unknown**, **0 omitted**. The next unread order was **6,183** `Telegram/SourceFiles/ui/image/image.cpp`.

### Current read-through 6183-6191

Orders **6,183-6,191** are exact-blob read and responsibility-decomposed across the image/resource base layer: prepared image cache keys and DPR transforms; typed download/image locations with versioned serialization, cache-key derivation and file-reference refresh; protocol-to-resource factories including progressive/cached/in-memory/web/video forms; bounded local-image decoding; and sanitized SVG preview rendering with explicit byte/dimension limits. These map to the existing canonical `Resource`/attachment/media owner rather than a second download/cache owner. Reading does not reduce `unknown`. Current accounting is **6,191 / 16,125 read**, **9,934 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,192** `Telegram/SourceFiles/ui/item_text_options.cpp`.

### Read-through 6192-6202

Orders **6,192-6,202** are exact-blob read and responsibility-decomposed across conversation-aware text option projection, single-window layer-stack lifecycle, canonical context-menu icon semantics, new/attention Badge projection, and local passcode-strength/transliteration helpers. These responsibilities map to existing source-neutral TranscriptEntry, Dialog/Popover, ContextMenu/Menu/Icon, Badge/Status and security/passcode owners. Sensitive passcode candidates must remain ephemeral and unlogged. Reading alone does not reduce unknown. Current accounting is **6,202 / 16,125 read**, **9,923 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,203** `Telegram/SourceFiles/ui/peer/color_sample.cpp@d11ac016c6e318786c3a1206efa415c91928ee21`.

### Read-through 6203-6214

Orders **6,203-6,214** are exact-blob read/decomposed across color/profile selection, video Avatar Resource playback, power-saving/reduced-motion policy, resize lifecycle, bounded row-scroll render caching and SearchField query/a11y composition. All map to existing source-neutral canonical owners and remain mapped-open. Reading alone does not reduce unknown. Accounting: **6,214 / 16,125 read**, **9,911 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,215** `Telegram/SourceFiles/ui/text/format_song_document_name.cpp@d5a77025b392b8385bff3268be1f7465d91d48a2`.

### Read-through 6215-6224

Orders **6,215-6,224** are exact-blob read/decomposed across media naming, locale/value/currency/credits formatting, custom-emoji animation lifecycle and canonical rich-text parser presets. All remain mapped-open; reading alone does not reduce unknown. Accounting: **6,224 / 16,125 read**, **9,901 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,225** `Telegram/SourceFiles/ui/top_background_gradient.cpp@7325d188f856d76f08c67a2bdb43ce1f50eb94b7`.


### Read-through 6225-6239

Orders **6,225-6,239** are exact-blob read/decomposed across commerce/profile top-gradient pattern rendering, unread/peer Badge precedence and animation/power-saving behavior, unread counter formatting, Avatar/Resource cache invalidation and fallback shapes, canonical vertical-list composition, and WebView theme/zoom/style serialization plus attribute/script escaping. All remain mapped-open and source-neutral; build-only `ui_pch.h` is recorded without claiming product closure. Reading alone does not reduce unknown. Accounting: **6,239 / 16,125 read**, **9,886 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,240** `Telegram/SourceFiles/ui/widgets/chat_filters_tabs_mode.h@f5d37194063110eb2ba8698f5b714e8a7ebe37ea`.


### Read-through 6240-6244

Orders **6,240-6,244** are exact-blob read/decomposed across persisted chat-filter Tabs presentation mode, canonical Tabs/Badge/ContextMenu composition, locked-range handling, custom-emoji pause, and drag-reorder lifecycle with pinned intervals, threshold start, edge auto-scroll, cancel/apply convergence and stable index remap. These remain mapped-open and must reuse existing source-neutral Tabs/ContextMenu/Badge/Status plus the single canonical ordering state owner. Reading alone does not reduce unknown. Accounting: **6,244 / 16,125 read**, **9,881 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,245** `Telegram/SourceFiles/ui/widgets/chat_filters_tabs_strip.cpp@750feab6cbf3b30a3b1e2bc946d85eaed4a4382c`.


### Read-through 6245-6251

Orders **6,245-6,251** are exact-blob read/decomposed across the shipping chat-filter Tabs composition, premium lock/menu/edit/remove/mark-read and saved ordering lifecycle, plus canonical Color Picker/validated field synchronization and Continuous/Media Slider pointer-wheel-keyboard-a11y progress/finished semantics. These remain mapped-open and must reuse source-neutral Tabs/ContextMenu/Badge/Picker/TextField/Slider and existing filter/settings/media domain owners. Reading alone does not reduce unknown. Accounting: **6,251 / 16,125 read**, **9,874 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,252** `Telegram/SourceFiles/ui/widgets/cross_fade_label.cpp@fb094439ceebc0ad1b45db68ebf08d31450464e3`.


### Read-through 6252-6261

Orders **6,252-6,261** are exact-blob read/decomposed across cross-fade text motion, Discrete/Settings Slider selection timing and ripple state, expandable participant Checkbox/Avatar collection semantics, phone/country/username masked-input normalization including Windows IME re-entry fencing, and count-aware time-part placeholders. These remain mapped-open and must reuse source-neutral Status/text motion, Tabs/Slider, ParticipantRow/Checkbox/Avatar, and TextField owners. Reading alone does not reduce unknown. Accounting: **6,261 / 16,125 read**, **9,864 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,262** `Telegram/SourceFiles/ui/widgets/glare_tooltip.cpp@b38e44a0769199bf65b5d584181e3b4f24d716aa`.


### Read-through 6262-6269

Orders **6,262-6,269** are exact-blob read/decomposed across glare Tooltip tracking/timer/teardown and motion, gradient Button glare/ripple/cache behavior, equal-width horizontal Button layout, and LevelMeter value projection. These are presentation responsibilities over existing source-neutral Tooltip/Button/layout/Status owners and must honor reduced-motion and lifecycle teardown. Reading alone does not reduce unknown. Accounting: **6,269 / 16,125 read**, **9,856 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,270** `Telegram/SourceFiles/ui/widgets/marquee_label.cpp@ddb4fac4dc2eb2f4d990b283aad018416ce7d1d2`.
Orders **6,304-6,315** are exact-blob read/decomposed across Wallet card pointer-follow motion and DPR-aware gradient caches, a capability-restricted Wallet-to-chat/session Show adapter, collectible metadata/artwork priority lanes with a 10-chain budget, 60-second deadline, generation fencing, Gift-to-web fallback, cancellable Resource loading and preload-window projection, plus the encrypted transfer-comment reveal lifecycle with account/session/app-lock/sleep fencing, bounded identity resolution, key authorization, revision/scope cancellation and plaintext wipe on reset. These remain mapped-open to the existing source-neutral Wallet/Payments/Commerce, Resource/file-transfer/cache, Session/Navigation, Security/Key Protection, Theme, motion and canonical design-system owners; no Telegram-derived parallel Wallet UI/runtime is introduced. Reading alone does not reduce unknown. Accounting: **6,315 / 16,125 read**, **9,810 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,316** `Telegram/SourceFiles/wallet/wallet_content.cpp@6aa42680fee3870ae9969104fdb5872c309e8ef2`.
Orders **6,316-6,319** are exact-blob read/decomposed across the full Wallet content/composition monolith and its public seam plus the durable custody store. The source proves exact-dependency/revision fee quotes, per-press frozen recipient/amount/comment/link expiry, key-ladder and vault-epoch fencing, bounded signing readiness, QuoteExpired retries only before anything leaves the device, SubmissionUnknown operation handoff rather than blind resend, bounded recipient/name/address lookups, secure phrase/restore/import/conflict/backup/rotation lifecycles, polling/teardown and list/scroll-anchor behavior. Custody persistence is v4, separates immutable anchor and current signing keys, retains pending rotation/new-key/awaiting-server reconciliation, validates old/broken data fail-closed and centralizes secret-reference removal. All remain mapped-open to existing source-neutral Wallet/Payments, Security/Key Protection/Custody, storage, Session/Navigation, Resource, Motion and canonical design-system owners; no Telegram-derived parallel owner is introduced. Reading alone does not reduce unknown. Accounting: **6,319 / 16,125 read**, **9,806 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,320** `Telegram/SourceFiles/wallet/wallet_diamond_flight.cpp@9444c0eac18a5c625aad9a2671bbd319974129fb`.
Orders **6,320-6,323** are exact-blob read/decomposed across the send-diamond animation and the Wallet engine/host bridge. Motion semantics include a dynamic target, target-loss cancellation, exact single landing/finish, reduced-motion and power-saving suppression, loop continuity and bounded dirty-region repaint. Engine semantics include strict provider-origin/path enforcement, Accepted/Rejected/Uncertain routed submission, a 40-second ambiguous-broadcast wait that preserves late MTProto settlement, durable send-journal compare-exchange, account/clear-epoch protected-secret fencing, exact failed-call secret cleanup, stale private-result rejection, serial worker ownership and teardown that wakes blocked calls before shutdown while relying on durable journal recovery on restart. These remain mapped-open to existing source-neutral Wallet/Payments, Network, Vault/Key Protection, durable Storage/Journal and Motion owners. Reading alone does not reduce unknown. Accounting: **6,323 / 16,125 read**, **9,802 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,324** `Telegram/SourceFiles/wallet/wallet_fiat.cpp@6e16d23676a8d24ff840c71f88310cb21d540ce4`.
Orders **6,324-6,325** are exact-blob read/decomposed across Wallet fiat/rates formatting: unavailable-rate ellipsis, currency-specific exponent/decimal/grouping, approximate U+2248 output, tiny positive-value significant digit projection, localized currency names, stable top-currency ordering from the active currency plus cloud/system/input-method/phone-country locale signals, and minor-unit nanos. They remain mapped-open to source-neutral canonical Money/Currency formatting and Wallet rates owners.

Orders **6,326-6,339** are exact-blob read/decomposed across Wallet key protection, funding/on-ramp, palette replacement, singleton panel lifecycle, recovery phrase shares, live fiat-rate refresh, and sending effects. Existing-owner-first production work at parent exact HEAD `882728682058ea6235cf625c7e8a33f851fad1a3` extends `native/mahayana-messaging/src/wallet.rs`, `engine.rs`, and `service.rs`: provider-neutral authenticated key wrapping with epoch fencing and zeroized secret bytes; fixed-field recovery seed/XOR shares plus authenticated holder encryption; 5-minute rate refresh / 30-second retry state; funding-request generation fencing and optional base-currency fallback; singleton panel state; deterministic clock/glare math; native/server-authoritative runtime projection. Exact-head evidence: Desktop Chat Parity run `38016864742` completed success; Rust desktop runtime run `38016864748` bound the same SHA, canonical messaging tests succeeded and artifact `11656926928` has digest `sha256:cb27c66d4893776f5105741254f86df84d25a707ff7cd9f256e708c76676b0fa`. These rows intentionally remain mapped-open: no concrete platform key-protection adapter, real funding/rates service adapter, wallet shipping renderer/panel consumer, wallet-specific palette consumer, recovery-share product entrypoint, or sending-effect paint consumer has exact-head production evidence yet. Reading therefore does not reduce global unknown. Accounting: **6,339 / 16,125 read**, **9,786 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,340** `Telegram/SourceFiles/wallet/wallet_session.cpp@7000a36395819deea9ca8ba0cce227ff0e535f32`.


### Read-through 6340-6343

Orders **6,340-6,343** are exact-blob read/responsibility-decomposed across the Wallet Session monolith/public state contract and the real-time Wallet stream. Existing-owner-first production work extends only canonical `native/mahayana-messaging` Wallet/ConnectedApp owners: parked-wallet discovery, panel-scoped legacy-balance lookup, durable connected-app recovery fallback separation, and a source-neutral live-stream epoch/backoff/keepalive/renew/coalesce/history-recheck state machine. The stream still lacks a real Fabushi Host/network service adapter and 6,340-6,341 retain substantial mapped-open custody/history/collectibles/preview/send-recovery/rotation/TonConnect/UI responsibilities, so none of these rows is marked verified and global `unknown` is unchanged. Accounting: **6,343 / 16,125 read**, **9,782 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,344** `Telegram/SourceFiles/wallet/wallet_ton_connect.cpp@60bf68a9…`. Fresh exact-head GitHub Actions are required for the accounting descendant.

### Read-through 6344-6353

Orders **6,344-6,353** are exact-blob read/responsibility-decomposed across the connected-app session/key/connect/disconnect lifecycle, canonical connect/settings UI projection contract, durable claim journal, transaction emulation, and deep-link/start-param boundary. Existing-owner-first production descendants already implement transport message identity, staged claim recovery, source-neutral connect/key/disconnect state, emulation parsing and start-param security/encoding inside canonical native/mahayana-messaging owners. The connect/settings UI projection, real service/navigation composition and complete request-family closure remain mapped-open; no row is promoted to verified and reading does not reduce global unknown. Accounting: **6,353 / 16,125 read**, **9,772 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,354** Telegram/SourceFiles/wallet/wallet_ton_connect_request.cpp@496881cf….

### Read-through 6354-6357

Orders **6,354-6,357** are exact-blob read/responsibility-decomposed across the full connected-app request scheduler/Flow contract and its request-confirmation projection. Existing-owner-first production descendants now cover transport message identity, active/silent/waiting arbitration, durable claim recovery rebuild, recovery polling/submission settlement, flow-completion ownership fencing and Recovery::Offer fallback through canonical native/mahayana-messaging ConnectedApp owners. Remote pending/service composition, closed-session late context, wallet resolve/polling, cryptographic decrypt/sign/encrypt, fee/emulation send integration, Recovery::Answer re-encryption and shipping request UI/a11y/visual evidence remain mapped-open. No row is promoted to verified and reading does not reduce global unknown. Accounting: **6,357 / 16,125 read**, **9,768 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,358** Telegram/SourceFiles/wallet/wallet_transfer_messages.cpp@69cfdacd….



### Revision 9 source closure through 6,361

Orders **6,358-6,361** are exact-blob read/responsibility-decomposed for optimistic outbound transfer-message reconciliation and the durable submitted-transfer store. The canonical Wallet journal already owns handoff/lookup/terminal recovery; this descendant additionally fail-closes noncanonical source, destination and collectible addresses through the existing wallet-address validator. The Message owner still lacks the complete per-draft server-id floor, transport-identity and identical-twin adoption/refusal contract, so 6,358-6,359 remain explicitly mapped-open rather than receiving false implementation credit. Accounting is **6,361/16,125 read; 9,764 unread; 15,846 unknown; 0 omitted**. First unread is 6,362 `Telegram/SourceFiles/wallet/wallet_unlock.cpp@5fbaea9d950772164094e7477e5f786f9a78279f`.


### Revision 9 read-through 6362-6367

Wallet unlock, user-address and vault files are exact-blob read/responsibility-decomposed. Descendant `cb6d56174fb3e3f34c99e7bed3156c3b1e8d523d` closes one security gap: unlocked Wallet projection is runtime-only across serde/restart. Full grant/retention/platform-factor/address-service/multi-account vault lifecycle remains mapped-open. Accounting: 6,367/16,125 read; 9,758 unread; 15,846 unknown; 0 omitted. First unread: 6,368 `Telegram/SourceFiles/webauthn/cable.h@6909eb913dbb41765ebbaeecb2072f130f868655`.


### Revision 9 read-through 6368-6377

The first CaBLE/WebAuthn tranche is exact-blob read/responsibility-decomposed. Existing Fabushi WebAuthn provider/signer/proxy is the canonical owner; full BLE/QR/tunnel/Noise/CTAP and ceremony UI parity stays mapped-open. `94aaee4` already closes one fail-open boundary by rejecting unknown ceremony kinds instead of treating them as login/get. Accounting: 6,377/16,125 read; 9,748 unread; 15,846 unknown; 0 omitted. First unread: 6,378 `Telegram/SourceFiles/webauthn/org.bluez.xml@245e0b9bc27e1cfbb31bd78d750ae189adf5f0cc`.


### Revision 9 read-through 6378-6380

BlueZ interface and common WebAuthn security-key/CaBLE orchestration are exact-blob read/responsibility-decomposed. Existing Fabushi WebAuthn provider/signer/proxy remains the canonical owner; device/PIN/cancellation/fallback/platform parity remains mapped-open. Accounting: 6,380/16,125 read; 9,745 unread; 15,846 unknown; 0 omitted. First unread: 6,381 `Telegram/SourceFiles/window/main_window.cpp@ae2b4949734e6c8826310c96b65f51b630ab600d`.


### Revision 9 read-through 6381-6388

Main window and first notification-manager tranche are exact-blob read/responsibility-decomposed against existing Electron main/window-state/window-chrome/dock-badge/OS-notification owners. Existing geometry/focus/maximized/badge and scoped notification action behavior is retained; close-to-tray/background, privacy-aware OS title, full canonical conversation notification mute/privacy/grouping/device-delay and fallback/resource lifecycle remain explicitly mapped-open. Accounting: 6,388/16,125 read; 9,737 unread; 15,846 unknown; 0 omitted. First unread: 6,389 `Telegram/SourceFiles/window/section_memento.h@d139928a9996a6333e541a5db9c77e64056b41fd`.


### Revision 9 read-through 6389-6392

Section navigation/workspace contracts and draw-to-reply media flow are exact-blob read/responsibility-decomposed. Qt widget/slide/painting is not copied; routing, memento identity, focus/search/permission/theme behavior and async draw-to-reply target/payment/ephemeral safeguards map into existing ProductShell/ConversationWorkspace/Search/Permissions/Composer/Resource/Message owners. Accounting: 6,392/16,125 read; 9,733 unread; 15,846 unknown; 0 omitted. First unread: 6,393 `Telegram/SourceFiles/window/themes/window_theme.cpp@2ead4b28873908361728df3f622318ed073cdee2`.


### Revision 9 read-through 6393-6412

Theme runtime/editor/preview/chat/cloud/embedded/accent/name-generation files are exact-blob read/responsibility-decomposed. Existing SandThemeController and the Fabushi Design System remain the only app/theme visual authority; Telegram Qt palette/theme-format rendering is not ported. Applicable bounded parsing/resources, transactional test→keep/revert, async generation fencing, per-conversation/cloud theme state, accessibility/contrast and editor lifecycle remain mapped-open against existing Theme/Settings/Conversation/Resource owners. Accounting: 6,412/16,125 read; 9,713 unread; 15,846 unknown; 0 omitted. First unread: 6,413 `Telegram/SourceFiles/window/window.style@a91951fc42761db61f7f7718f89e226fc9d3b5c2`.


### Revision 9 read-through 6413-6423

Window style/adaptive/chat preview/chat switch/connection/controller files are exact-blob read/responsibility-decomposed. Current Fabushi already has canonical responsive shell, ConversationWorkspace/List/navigation, CoordinatorConnectionController and Electron window owners, so no Telegram window runtime is introduced. Full adaptive columns, accessible chat preview/switch, proxy/retry status, multi-account/separate-window and security/layer sequencing remain mapped-open where current behavior is partial. Accounting: 6,423/16,125 read; 9,702 unread; 15,846 unknown; 0 omitted. First unread: 6,424 `Telegram/SourceFiles/window/window_filters_favorite.cpp@c51e19763a7939fe908fb8b9fc3eb5f73735674f`.


### Revision 9 live read-through note: 6424-6437

Orders **6,424-6,437** are exact-blob read-complete and responsibility-decomposed on accepted `28ac576967a1026ecc89a927360fa0e138d7c88c`. Favorite/filter files preserve validated source-neutral shortcut/deep-link cancellation, collection selection/reorder/premium-lock, unread/mark-read, drag-hover and keyboard/screen-reader semantics. History-hider files preserve exactly-once overlay dismissal. Lock files preserve passcode flood/busy/async derivation, OS-unlock cooldown/lifetime and terms-consent security semantics. Main-menu/helpers preserve multi-account unread, archive lifecycle, profile/status, Contacts/Calls/Wallet/Settings navigation, theme/scale, owned-room enumeration and lifetime-bound dynamic Mini App/Plugin resource projection. All remain **mapped-open** to existing canonical ProductShell/Account/Conversation/ReadState/Security/Settings/Marketplace/MiniApp/Resource owners; no Telegram runtime/sidebar/menu/security registry is introduced. Reading alone closes no unknown. Accounting: **6,437/16,125 read; 9,688 unread; 15,846 unknown; 0 omitted**. First unread: **6,438 `Telegram/SourceFiles/window/window_media_preview.cpp@4fe0e7f11298b0b56c99c47e43c27c6fd47d0282`**. Fresh descendant exact-head Actions are required.

### Revision 9 live read-through note: 6438-6448

Orders **6,438-6,448** are exact-blob read-complete and responsibility-decomposed on accepted `28ac5769`. They cover media-preview resource/playback lifetime, context-sensitive canonical conversation commands, temporary restore shells, durable multi-window/account/conversation restore, section fallback semantics, and detachable-window route/security identity. All remain **mapped-open**. Current `source/electron-main/window-state-persistence.ts` is explicit partial parity: it persists one main-window placement but not durable account/conversation multi-window restore. Reading alone closes no unknown. Accounting: **6,448/16,125 read; 9,677 unread; 15,846 unknown; 0 omitted**. First unread: **6,449 `Telegram/SourceFiles/window/window_session_controller.cpp@df216237e47d22c200a51b2c7fda31175682d3c5`**.

### Revision 9 live read-through note: 6449-6458

Orders **6,449-6,458** are exact-blob read-complete and responsibility-decomposed. The large SessionController is decomposed into canonical Navigation/Conversation/Search/Window/Resource/Permissions/Calls/Settings/Theme/Story owners rather than ported as a new monolith. The range also captures typed deep-link intent, setup-email enrollment/verification with flood/expiry handling, canonical motion/adaptive-top-bar semantics, and passcode unlock exactly-once convergence across windows. All remain **mapped-open**. Accounting: **6,458/16,125 read; 9,667 unread; 15,846 unknown; 0 omitted**. First unread: **6,459 `Telegram/Telegram.plist@d156eda29c86bbd11b3c7757b9d419e2c863ea8b`**.

### Revision 9 live read-through note: 6459-6483

Orders **6,459-6,483** close the upstream macOS bundle plist, helper/app entitlements, icon-catalog manifest and every referenced 16/32/128/256/512 1x/2x icon blob by exact identity. The applicable responsibility is bundle/privacy/protocol/signing/sandbox/icon coverage under **Fabushi** identity; Telegram/TON schemes and Telegram artwork are explicitly not migrated. Current `desktop/package.json` already owns `com.ombhrum.fabushi`, the `fabushi` scheme, Fabushi icon, forced signing, notarization, and microphone/camera usage; current MAS entitlements own sandbox/network/user-selected/mic/camera. Downloads/bookmarks/location remain applicability-open and are not added without a shipping need. Accounting: **6,483/16,125 read; 9,642 unread; 15,846 unknown; 0 omitted**. First unread: **6,484 `Telegram/ThirdParty/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`**.

### Revision 9 live read-through note: 6484-6505

Orders **6,484-6,505** close the root ThirdParty gitlink identity layer: exact pinned commits and `.gitmodules` repositories are mapped to source-neutral capability/implementation roles. This includes generic C++ support (GSL/expected/range-v3/TooManyCooks), math/Markdown/QR/syntax/image/language/spell utilities, Linux IME/portal adapters, CBOR/FIDO2/passkey dependencies, compression/hash/password-strength primitives, and `tgcalls`. The root pin does **not** close any nested recursive source; all nested entries remain independently unread/unknown until their later deterministic orders. No Telegram runtime/library is automatically retained. Accounting: **6,505/16,125 read; 9,620 unread; 15,846 unknown; 0 omitted**. First unread: **6,506 `Telegram/build/build.bat@3e5f698f99c5944e237c514d60124bc25489ea76`**.

### Revision 9 live read-through note: 6506-6541

Orders **6,506-6,541** close the root build/release tooling source read. Product-relevant responsibilities include fail-fast release prerequisites, version/channel/artifact collision fencing, macOS preserved-toolchain and universal-resource checks, Windows/Linux packaging, App Store/GitHub publication, source tarballs including submodules, symbols, installer/upgrade semantics, reproducible dependency preparation, canary positive+negative signature fixtures, ES256 Key Vault signing with manifest-key-coordinate verification, and digest/dedupe/retry semantics for spellcheck dictionary distribution. Telegram product/private paths are not targets; equivalent Fabushi guarantees must live in existing GitHub Actions/electron-builder/updater/platform/resource owners. Accounting: **6,541/16,125 read; 9,584 unread; 15,846 unknown; 0 omitted**. First unread: **6,542 `Telegram/cmake/binobj2obj.py@3683a6dd3a72706ee382ea415d6e9cfaae84d8ce`**.

### Source closure through 6,574

Orders **6,542-6,574** are exact-blob read-complete/responsibility-decomposed for CMake/build-runtime composition. They preserve source-neutral responsibilities for deterministic code/resource generation, updater trust-root and signed-manifest embedding, FIDO2/WebAuthn, Calls/media composition, Wallet/Payments, password-strength dictionaries, Localization, data export, protocol/session maturity, Apple Swift runtime packaging and production-code updater tests. No Telegram-specific runtime or second product owner was introduced. Accounting: **6,574/16,125 read; 9,551 unread; 15,846 unknown; 0 omitted**. First unread is 6,575 gitlink `Telegram/codegen@dbd1d53137cd581cfdbeebe93bcaf54abf55ec74`. Reading/decomposition grants no implementation or release credit; same-head Actions and production/release evidence remain mandatory.


### Source closure through 6,591

Orders **6,575-6,591** are exact root-entry read-complete/responsibility-decomposed. The pinned `Telegram/codegen` child tree was inspected for tokenizer/UTF-8, emoji compatibility, localization and style-generation responsibilities; configure wrappers preserve fail-closed target/toolchain/credential propagation; direct library gitlinks map to existing runtime/concurrency/media/QR/reactive/spellcheck/storage/protocol/translation/design-system/call/webview owners. `Telegram/create.bat` is explicitly classified development-only non-applicable after reading, so it closes exactly one unknown without counting as omitted. Accounting: **6,591/16,125 read; 9,534 unread; 15,845 unknown; 0 omitted**. First unread is 6,592 `Telegram/shaders/argb32.frag@4b4deb01e408b5f441808dc104488e3b8eeabfe2`. No mapped-open row receives implementation or release credit.


### Source closure through 6,626

Orders **6,592-6,626** are exact-blob read-complete/responsibility-decomposed for GPU shader contracts. The tranche preserves ARGB/NV12/YUV420 conversion, OpenGL/Vulkan coordinate-origin handling, blur/dither, premultiplied alpha, rounded/fade/shadow composition, PiP nine-slice shadow behavior, premium time/night/alpha material inputs, and seeded/timestep-bounded particle lifecycle. These are mapped to source-neutral Fabushi rendering/Resource/Call/visual-effect owners; Telegram branding and pixel design are not carried over. Accounting: **6,626/16,125 read; 9,499 unread; 15,845 unknown; 0 omitted**. First unread is 6,627 `changelog.txt@bf82383b02717265acac50035c2ce3000afdac7a`. All shader rows remain mapped-open until shipping consumers, focused tests and cross-platform visual evidence exist.


### Root source closure complete at 6,653

Orders **6,627-6,653** close the remaining root entries: release changelog/build helpers, credential/build documentation, AppStream screenshot assets, WEB proxy design and acceptance contracts, Linux XDG/AppStream/DBus/Snap shipping integration, Wallet/thread/shared-media/navigation task evidence, and static-analysis/edit-rule tooling. Root accounting is now **6,653/6,653 exact-read**, while the recursive census remains **6,653/16,125 read; 9,472 unread; 15,845 unknown; 0 omitted**. The next recursive identity is order 6,654: `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`, mount `Telegram/ThirdParty/GSL`, path `.clang-format`, object `c12d3bf2994fd5a083c04025d355a5ab4b3f6802`. The existing validator intentionally forbids crossing the root prefix without an explicit component identity schema, so schema/validator extension is the next required gate rather than a bookkeeping bypass.

### Recursive component read-through 6,654-6,697 — Microsoft/GSL

Orders **6,654-6,697** exact-read the pinned `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37` component mounted at `Telegram/ThirdParty/GSL`. The 44 entries cover repository/build/CI metadata, license/security/provenance, header-only safety contracts (bounds and extent checks, non-null ownership, checked narrowing, byte/string boundaries, scope cleanup), and focused fail-closed tests. Applicable semantics map to existing Fabushi Build/Quality/Provenance and Rust/TypeScript/platform-boundary owners; no Telegram/GSL-specific runtime is introduced. Reading does **not** reduce `unknown`; production/shipping evidence remains required. Accounting: **6,697/16,125 read; 9,428 unread; 15,845 unknown; 0 omitted**. Next: **6,698 `Telegram/ThirdParty/MicroTeX::.github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764`**.

### Recursive component read-through 6,698-6,714 — MicroTeX entry/build contracts

Orders **6,698-6,714** exact-read the pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f` build/entry contract under `Telegram/ThirdParty/MicroTeX`. The applicable product behavior maps into the existing ConversationWorkspace/Transcript math owner and existing Build/Resource/Release provenance owners. Current production work adds a 32 KiB pre-parse formula bound and bounded 32 MiB LRU markup cache without retaining the C++/Qt/GDI runtime. Reading does **not** reduce `unknown`; current exact-head GitHub Actions evidence is still required. Accounting: **6,714/16,125 read; 9,411 unread; 15,845 unknown; 0 omitted**. Next: **6,715 `Telegram/ThirdParty/MicroTeX::readme/example_bw_false.svg@9e812c823d3cbe06ab43ee77f10b6122bd638dd`**.


### Recursive component read-through 6,715-6,740 — MicroTeX reference/golden/resource manifest

Orders **6,715-6,740** exact-read the pinned MicroTeX reference SVG/PNG outputs, executable sample corpus, golden rendered samples, TeX logo, resource-root sentinel and `RES_README` font/license/language resource manifest. Applicable semantics map to the existing canonical ConversationWorkspace/Transcript math visual-regression owner plus Resource/Release provenance owners. These assets are regression/provenance authority only; they do not justify retaining or shipping a MicroTeX/Telegram renderer. Reading does **not** reduce `unknown`; exact-head production/resource/visual evidence remains required. Accounting: **6,740/16,125 read; 9,385 unread; 15,845 unknown; 0 omitted**. Next: **6,741 `Telegram/ThirdParty/MicroTeX::res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b`**.

### Recursive component read-through 6,741-6,800 — MicroTeX multilingual formula resources

Orders **6,741-6,800** exact-read the executable formula sample corpus, compiled font-resource manifest, Cyrillic license/mappings/symbols/font metrics/fonts, Latin/math font resources and licenses, and the first Greek font/metric resources. Applicable responsibilities stay in the existing ConversationWorkspace/Transcript math owner plus Resource/Packaging/Release provenance owners: multilingual Unicode-to-symbol/formula mapping, deterministic glyph metrics/kerning, explicit resource fallback, packaged resource completeness, license/notice provenance, and rendering regression. Reading does **not** reduce `unknown` and provides no verified credit. Accounting: **6,800/16,125 read; 9,325 unread; 15,845 unknown; 0 omitted**. Next: **6,801 `Telegram/ThirdParty/MicroTeX::res/greek/fcmbpg.xml@dbedd0385cdb004569e0d69b2567a1fa6c6cad5f`**.

### Recursive component read-through 6,801-6,830 — MicroTeX Greek resources and atom safety

Orders **6,801-6,830** exact-read the rest of the Greek glyph resources plus the first atom/rendering implementation slice. Applicable atom responsibilities include wrapper-chain depth bounding, copy-before-mutation for shared cached atoms, exceptional-path state cleanup, bounded matrix/column-spec expansion, finite user length handling, TeX operator/glue/kerning isolation, delimiter/matrix/layout semantics, and deterministic multilingual glyph resources. These map to the existing canonical KaTeX math owner and Resource/Release provenance owners. The shipping owner now sets explicit `maxExpand=1000`, `maxSize=1000em`, and `trust=false` for strict and recovery renders, with a focused Desktop Chat Parity contract; exact-head verification is still required. Reading does **not** reduce `unknown`. Accounting: **6,830/16,125 read; 9,295 unread; 15,845 unknown; 0 omitted**. Next: **6,831 `Telegram/ThirdParty/MicroTeX::src/atom/atom_space.h@3ba203a52a1fcff831a1137dfa6c3e35927fb0d7`**.

### Recursive component read-through 6,831-6,852 — MicroTeX bounded layout/parser safety

Orders **6,831-6,852** exact-read the pinned MicroTeX unit/color, box/layout, formula/environment and glue implementation contracts. Invalid fixed-table indices and non-finite numeric input must fail closed; layout/repeat/matrix growth is bounded; partial-parse and exceptional paths preserve deterministic later renders. These responsibilities stay in the existing canonical ConversationWorkspace/Transcript KaTeX owner plus Build/Release provenance and do not justify a MicroTeX runtime. The canonical owner adds fail-closed non-finite custom budgets and focused output/cache lifecycle tests. Reading does **not** reduce `unknown`. Accounting: **6,852/16,125 read; 9,273 unread; 15,845 unknown; 0 omitted**. Next: **6,853 `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83`**.

### Recursive component read-through 6,853 — MicroTeX localized decimal input

Order **6,853** exact-reads `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83` from pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`. Its applicable responsibility is deterministic numeric input normalization: U+066B becomes the ASCII decimal point and the twenty accepted Unicode decimal blocks map digit-for-digit to ASCII while every unrelated code point remains unchanged. The existing canonical ConversationWorkspace/Transcript KaTeX owner performs that normalization before both strict and recovery renders, and the focused Desktop Chat Parity contract covers all twenty blocks, unchanged non-target text, and identical strict/recovery input. No MicroTeX parser/runtime is retained. Reading does **not** reduce `unknown`; descendant exact-head GitHub Actions remain required. Accounting: **6,853/16,125 read; 9,272 unread; 15,845 unknown; 0 omitted**. Next: **6,854 `Telegram/ThirdParty/MicroTeX::src/core/macro_def.cpp@0ccf9fe8b4028f97c62834555d2a3d344d0f8726`**.

### Live rebaseline 28ac5769 → 6fed91ff

Accepted upstream advanced by four commits to `6fed91ffab9861771a75f29df65031b3c80b6941` / root tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6` without adding or removing a root path. Ten root blobs changed at deterministic orders **3,155, 3,201, 3,202, 3,766, 4,521, 4,522, 5,166, 5,943, 6,541, 6,627**. Each changed blob was re-read against its predecessor object and reconciled through the explicit source-disposition rebaseline record. Version/resource/build changes preserve source-neutral release-version coherence. The history-photo delta adds owned enlarge eligibility/hit-testing plus press/release ripple lifecycle and keeps sponsored/editor/small-media exclusions; it maps to the existing Transcript/media interaction owner rather than a new media root. The media-viewer delta explicitly claims Save/SaveAs at the application event-filter boundary so competing shortcuts cannot consume Ctrl+S; it maps to the existing canonical media-viewer shortcut owner. The new style opacity is presentation-only under the Fabushi design system. The 7.3.1 changelog adds wallet Windows Hello/passcode, copy-restricted download, custom-theme date-color and photo-editor Link regression cues; those cues remain open until their existing Fabushi owners have shipping evidence. Recursive totals/order are unchanged, so read-through remains **6,853/16,125**, first unread remains **6,854**, unread **9,272**, unknown **15,845**, omitted **0**.
