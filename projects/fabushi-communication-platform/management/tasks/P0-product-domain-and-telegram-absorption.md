# P0 — 全量源文件/模块与现有 Fabushi 架构对照

Status: active / not accepted  
Project: FBCP-001 Revision 7 / TDRP-001 Revision 9  
Updated: 2026-10-07  
Execution: all executable verification only GitHub Actions

## Goal

为全部 tdesktop 非 UI 代码职责的等价重写建立精确、无遗漏的输入和实施合同，而不是只做 Telegram 功能研究。最终完整实现原 Bot + 全部源范围通信能力，统一 Fabushi 品牌与微信式左竖栏布局。

## A. Re-read canonical architecture and source

重新读取 main、PR #20 状态和 current canonical exact HEAD。PR #20 在本次规范读取时已合并；不能继续从历史 open/draft 认知开始，也不能回退到旧实现分支。既有 Bot 架构/验收硬门继续有效。

重新读取 tdesktop discovery HEAD。当前 accepted discovery HEAD 为 `72b3b71c3d6e450e5ef94a3112dd750a0168aa0b`（由 `f23c37857220eb84f8559f0901ea26fb304b564b` 前进 7 commits），更早研究基线为 `33261535a0e747f125e0ed25486f01e556330677`。在 GitHub Actions 中建立完整 root tree、递归 gitlinks、外部依赖/补丁/资源/工具链一致 baseline，更新现有 lock/inventory/ledger/dossiers，记录旧→新差异。`baseline_ready` 不得在递归/外部来源未闭合时为 true。

## B. Inventory all files and understand all modules

每个源文件/依赖条目有 exact identity/hash、类型和责任，不能只统计目录。按 TDRP-SRC-02/SRC-03 全量覆盖，特别拆出 mixed UI 文件里的非 UI 逻辑。解析未知或 API 截断必须补读，无法访问保留 blocker。

逐模块 dossier 记录完整源文件/symbol、状态机、所有权、调用图、行为 oracle、错误/安全/生命周期/恢复/性能/平台与许可证。现有 capability graph 是旧基线索引，先审计差异，不是最终范围上限。

## C. Exact-head existing-owner inventory

读取真实 shipping entrypoints，而非根据旧聊天推测：

- product shell/左侧导航、conversation/list/workspace、transcript/cards、composer/drafts；
- account/identity、Shared Room/member、permissions、contacts、search；
- attachments/artifacts/resources、storage/recovery、notifications/settings/platform；
- Coordinator/Host/Runner、Agent、Computer、Plugins/MCP/marketplace、Automations/Task；
- 当前 calls/media/network/service owner（若存在）。

每个 source responsibility 填写 owner 候选、选择原因、target paths/symbols、语言、需修改状态/事件/接口、服务依赖、数据迁移、tests/production evidence。目标 owner 尚未合适时做 rejected-owner analysis 与最小 new-owner ADR，禁止整块另建 CommunicationCore/Telegram subsystem。

同时建立 canonical composition inventory：ProductShell、ConversationWorkspace、ConversationList、Transcript、Composer、Participant/Profile、Resource viewer/editor、Search、Settings、Marketplace、Task/Automation 等每类概念只能有一个 root。每个责任写 `composition_root + composition_slot + variation_axes`；禁止 Bot/Human/Group/Channel/来源型完整 UI/状态栈分叉。

如果某项 Telegram 能力当前完全没有 owner，不能标 N/A：记录 `novel_capability`、所有 rejected owners、owner-absence evidence、最小 state/lifecycle responsibility 和 ADR。批准后只能新增 source-neutral capability owner，并通过 typed contracts 接入统一 shell、identity/resource/permissions/navigation。

## D. Language / production contract

C++ 非 UI 产品逻辑默认 Rust；前端表现与交互投影使用现有 React/TS；系统特有职责采用最薄平台适配。所有非默认语言/通用依赖选择写明理由与安全/许可证边界。

不得以 TDLib、原 C++ core、FFI wrapper、sidecar、外置 Telegram Web 或“暂时 Provider”替代源责任重写。已有实现如违反目标，记录 cutover、数据兼容与移除路径，不能因已存在就自动接受，也不在本次文档任务中盲删生产代码。

## E. Native service completeness

对每项客户端远端依赖列 client/server contract、Fabushi owner、认证/权限/持久化、部署与真实 E2E。自有 messaging/sync/presence/media/call/push 和必要商业能力支持现有模型，不形成第二产品。

缺服务、账号、密钥/签名或产品配置时标 blocked；不得以 mock、仅 UI 或“不用 Telegram 网络”当成功能完成/排除。

## F. UI / branding plan

明确同一 shell 内：左侧窄竖栏 -> 当前入口列表/搜索 -> 主工作区 -> 可选详情。消息、联系人、插件市场必须一级可达；插件市场迁移复用原安装/权限/Plugins/MCP owner，旧路由兼容不复制状态。

Agent、任务、Computer、Automations、账号设置等现有主能力仍可达。消息列表/正文、草稿、未读、运行中任务和 Agent 输出维持同一状态。品牌清单覆盖窗口、头像、Logo、导航、默认文案、通知、托盘、安装和更新；统一 Fabushi，法律声明/provenance 例外精确记录。不得把用户原截图或其私人数据提交公开仓库。

UI 计划必须给出 single-composition matrix：Conversation 只允许一个 workspace、Profile 只允许一个 framework、Resource/Search/Settings/Marketplace 等同理。差异只作为 typed section/entry/action/panel/overlay。Call/Story/media editor/payment 等若没有现成 surface，可设计最小 capability surface，但仍由统一 shell/router 管理并复用 canonical truth。

同时必须交付 **UI capability-placement matrix**。每个用户可见 capability 记录：`ui_entry_class`、primary nav owner、collection/filter、creation flow、object action、detail section、capability surface、route、keyboard/focus/accessibility、responsive behavior、state continuity。没有这些字段的 UI responsibility 不能进入 mapped/implemented。

消息领域必须明确一个 `ConversationCreationSurface`：Human private / Group / Channel / Agent / Hybrid / future Conversation kind 以 typed variant 实现；Group/Channel 不新增独立一级 App。Contacts 的 add/invite 与 Messages 的 conversation creation 分开，联系人发消息只 find-or-create canonical Direct Conversation。

搜索必须明确一个 Search owner 和三层 UX：入口内搜索、当前对象搜索、Universal Search（Cmd/Ctrl+K）。Participant/contact/member/admin picker 复用同一 Participant Search + eligibility policy。P0 必须列 local/remote data source、typed result/provider、权限/隐私、ranking/dedupe/cursor/cancel/debounce/stale-result fencing 与 result navigation。

P0 还必须完成 exact-head UI foundation audit：当前 runtime theme tokens、Sand/shared primitives、overlay/focus behavior、command palette、conversation list/workspace、composer、menus/dialogs、settings/plugins surfaces。基于实际 owner 建立 Fabushi semantic token/component adapter 计划，禁止凭空重做设计系统。

必须输出 design-system migration matrix：现有 token/primitive -> Fabushi semantic token/component；哪些 legacy `sand-*`/`cursor-*` 留在 compatibility/evidence adapter；哪些 feature-local magic values 必须收口；每个核心 screen 对应的 canonical pattern 与 protected slots。

必须建立 canonical screen/state inventory：Messages、Contacts、Conversation(Human/Agent/Group/Channel/Hybrid)、Profile、Creation、Search、Marketplace、Settings、Tasks/Automations、Computer、Call、Story/Media、Mini App，逐一列 loading/empty/error/offline/permission/data-heavy/light/dark/locale/responsive/a11y/motion states。

## G. First shipping vertical slice

existing Fabushi shell + 左竖栏 -> Human identity/contact -> 同一 conversation/composer -> Fabushi native durable send/receive -> canonical transcript -> 显式 Agent action/permission -> Coordinator/Host/Runner -> 同一 transcript/artifact -> restart/reconnect。

同时明确消息/联系人导航与插件市场迁移的可执行验收。第一 slice 通过不等于全部模块完成，后续必须继续全部长尾功能。

## H. Ledger and validator update

按 TDRP-001 §2/§8 扩展现有 schema/validator，建立 file/responsibility/module/owner/target/evidence 双向 gate，不能新造平行账本或批量标 verified。schema/代码/测试修改是下一实现任务，须在 GitHub Actions 执行；本文不声称 gate 已实现。

validator 还必须建立 composition graph gate：检查 canonical roots、routes、state owners、capability registrations 与 production entrypoints，发现按 participant/conversation/source 类型复制完整 workspace/list/profile/composer/resource/settings root 时 fail；对 novel capability 强制 owner-absence evidence + ADR + typed integration。

另建立 UI IA/Search gate：所有可见 capability 必须有合法 entry class/canonical route；Conversation creation 不得分叉为 Group/Channel/Bot 独立 root；Search consumer 必须连接 canonical Search owner/provider registry，禁止页面私建第二索引/权限真相。对应可执行检查与 UI acceptance 仅在 GitHub Actions。

另建立 Design System / Visual gate：检查新迁移 UI 是否只消费 Fabushi semantic tokens/canonical components，是否匹配 canonical screen pattern；固定 viewport、light/dark、zh-CN/en、长文本/RTL、keyboard/focus、reduced-motion、large-data states 生成 visual artifacts 并比对 approved baselines。新增 token/primitive/pattern 必须有 design exception review。

另建立 Quality/Test Governance gate：扩展 ledger/schema 记录 quality_risk、requirement/oracle/invariant/test IDs、negative/property/temporal/fault/UI/visual/a11y/performance/security/regression/exploratory evidence。P0 必须生成 RTM，并确保每个 planned capability 在实现前已有 test basis/oracle。

优先建立 Conversation Turn / Transcript Ordering / Reconciliation 三个 oracle，因为它们覆盖此前最容易被 eventual-state E2E 漏掉的 final disappearing、intermediate replacement、message reorder、late baseline 与 optimistic/authoritative merge 类问题。

packaged acceptance 必须扩展为时间序列：不能只等待 marker 出现；需要记录整个 logical turn 的所有 assistant/tool entries，在 terminal 后经过 quiet window、switch-away/back、reconnect/reload/restart，再比较 canonical semantic result。完整 session video 必须由独立验收者实际审阅。

## Exit criteria

P0 只在以下均完成时通过：

- upstream/target exact baseline、一致 lock/inventory 与递归/外部闭合可核验；
- 全部文件和 non-UI responsibilities 有明确对照，无未知范围；
- 全模块行为合同、current owner resolution 和必要最小 ADR 完整；
- 语言、服务端依赖、接口/数据迁移、UI/品牌与发布风险明晰；
- first slice 和后续全部模块均有具体实现/测试路径；
- 新 ledger/validator gate 取得 exact-head GitHub Actions 证据。

P0 通过只是实施输入闭合，不是产品已完成。某项 blocked 时继续无依赖阻塞的研究、合同、已批准实现或证据收集；不能伪造 P0/既有 Bot gate 已通过来启动依赖它的工作。需要用户支持先检查相同通知及回复，按指定渠道通知一次，再推进其他内容。


## Current Stage-A production wiring — Telegram client ownership handoff (2026-11-01)

The specific client-id injection gap recorded on 2026-10-31 is now **implemented / validation not run** in production commit `e8a8b7726679c06862205ef21fdfbfc0c5aca4ed`. The existing Fabushi account/platform-login owner now derives an account/device-scoped Telegram storage secret name, keeps the 32-byte storage key in the existing OS-protected secret lifecycle, and projects only that key through the already-private `0600` Host credential file. The Rust native credential reader consumes the private field in process memory; the shipping Host auth parser accepts but does not expose it.

Canonical AppHost production composition opens/restores one persistent embedded Telegram client under the canonical account-scoped app-data root, retains a `TelegramClientLease` for process lifetime, and passes only a typed `TelegramClientHandoff` into `HostCreateConfig.telegram_client_id`. Existing `MahayanaHost` then registers the existing `TelegramConversationProvider::from_client_id` non-owningly. No Telegram-specific account owner, provider stack, sidecar, subprocess, Web surface, or second runtime was added. Generic FeatureHost/CLI paths that have no canonical platform-account storage owner still do not manufacture an ephemeral Telegram account.

Authored regression contracts cover account/device/issuer-scoped secret naming, strict private-key parsing, and stable account-scoped Telegram database paths. Per Stage A policy none of those tests, builds, typechecks, validators, generators, packages, or runtime checks were executed. Broader Telegram authentication/self-user settlement, source-semantic closure, external-source blockers, release legal acceptance, Stage-B execution evidence and independent acceptance remain open; `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.


## Stage-A Telegram authorization / canonical self-user settlement (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Production implementation through `d19fc3742b9a2d5076513ec444383d7683a35ea2` extends the **same** account-bound `TelegramClientLease` created by the canonical Fabushi account/platform-login composition. No second Telegram account, runtime, provider registry, message store, sidecar, subprocess or Web wrapper was introduced.

The embedded runtime now parses `auth.Authorization.user.id` after `auth.sentCodeSuccess`, sign-in, password and registration success and owns it as the canonical Telegram `self_user_id` only while authorization is `Ready`. Authorization/challenge state, phone-code/SRP context and MTProto auth-key/DC/salt restore material are stored through the existing account-scoped encrypted SQLite store. Restore fails closed when `Ready` lacks either self identity or auth-key material; transport reconnect reuses the persisted auth key instead of manufacturing a new Telegram runtime.

`TelegramClientLease::handoff` now reads the live runtime `AuthorizationSnapshot` and passes `client_id + authorization_state + self_user_id` into the existing `HostCreateConfig`. The existing non-owning `TelegramConversationProvider` remains attached to that one client and dynamically resolves `Ready+self_user_id`; before authorization it rejects identity-sensitive operations, while same-process authorization completion does not require a second client or Host recomposition.

Authored-only contracts cover authorization user-id parsing, encrypted private metadata, Ready restore/fail-closed restore, and Host handoff consistency. Per Stage A policy, no build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was executed or queried. This slice is `implemented / validation_status=not-run / accepted=false`; broader fixed-U semantic closure, Boost/libiconv member enumeration, original-host GitLab blockers, release/legal acceptance, Stage-B evidence and independent ACCEPT remain open.


## Stage-A canonical Telegram login entry / private application credential (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact fixed-U source reads anchor the applicable login behavior to `intro_phone.cpp@ee14ffa7c6f31e171cdcbdfaf627ea9868af1246` (`auth.sendCode`), `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489` (`auth.signIn`), `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e` (`auth.checkPassword`) and `intro_signup.cpp@1ba7a38379a6ae2febe615783333fd0822e0bb96` (`auth.signUp`). These are behavioral provenance anchors only; no whole intro file is claimed semantically closed and global unknown/unread counts remain unchanged.

Production implementation through `c8109e3412d8c2cb4e7362a0fe4ec4976c07bd22` keeps existing owners: `FabushiAuthService` reads an application-scoped `api_id/api_hash` pair from the existing protected main-process secret store, validates the exact bounded shape, and copies it only through the existing private mode-0600 Native Host credential. The private Rust credential reader retains the pair in process memory; `TelegramApplicationCredentials` redacts `api_hash` from Debug. Renderer-facing methods never accept or return either application credential.

The existing production renderer/Coordinator carrier now exposes typed `feature.auth.telegramStatus`, `telegramSendCode`, `telegramSignIn`, `telegramPassword` and `telegramRegistration` calls. AppHost routes all of them to the same process-lifetime `TelegramClientLease`. First send-code bootstraps transport only when no live/restorable transport exists; restored auth-key material is reused. Every authorization mutation rereads the same runtime `AuthorizationSnapshot` and returns only renderer-safe authorization state/self-user identity while the existing Host/ConversationProvider continues to resolve that same client.

Actual Telegram application credential values remain external product configuration and are intentionally absent from repository and renderer state; missing/malformed configuration fails closed. Authored contracts cover application-credential bounds, private Native parsing and renderer parameter secrecy. Per Stage A policy, no build, typecheck, lint, schema validator, generator, test, package, deployment, installation, workflow/status query or runtime acceptance was executed. State is `implemented / validation_status=not-run / accepted=false`.


## Stage-A Telegram auth lifecycle typed error/backoff contract (round 44, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual exact-source reads covered `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489`, `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e`, `intro_email.cpp@dfb5ceeddeb48ab00773add72e75e0aef8ceb074`, `intro_phone.cpp@ee14ffa7c6f31e171cdcbdfaf627ea9868af1246` and `intro_signup.cpp@1ba7a38379a6ae2febe615783333fd0822e0bb96`. These sources establish resend/cancel, password recovery, email verification, flood handling, challenge invalidation and bounded retry/error lifecycle responsibilities; they are behavioral anchors only and do not close the whole intro files.

Production implementation through `a9e9683a3bf8cd05fd79b46213c19f56bc7c606d` plus the manual bounded-number guard correction `e7c0da2a654aad9a65e6c7fc545f9cf7d9dd48a8` keeps the existing owner chain. The retained `TelegramClientLease` now classifies bounded Telegram auth errors, turns `FLOOD_WAIT_*` / `FLOOD_PREMIUM_WAIT_*` into a capped same-process backoff, prevents hot-looped same-lease auth mutations while that deadline is active, and always projects the live `AuthorizationSnapshot/self_user_id` on recognized issues. The shared Coordinator contract now has a discriminated Telegram authorization-state union plus bounded error/backoff validation; the production renderer client consumes that shared type rather than `unknown`. AppHost status remains routed to the same lease. No second account/runtime/provider/state store was added.

The current embedded runtime was also inspected at `bhrumom/fabushi@edde714def34e6eb5ae133dba20790c80a355c85`: it implements send-code, code submit, password submit and registration, but has no protocol command/builder path for `auth.resendCode`, `auth.cancelCode`, `auth.requestPasswordRecovery`, `auth.checkRecoveryPassword`, `account.sendVerifyEmailCode` or `account.verifyEmail`. Those remote lifecycle operations therefore remain explicit fail-closed implementation blockers instead of being fabricated in renderer/AppHost code. The source-specific `SRP_ID_INVALID` challenge refresh likewise remains blocked on a safe runtime `account.getPassword` refresh primitive; the typed projection records the required action without retry-spinning.

Authored-only contracts are the inline Rust auth-error/backoff test and `source/shared/rpc/coordinator.contract.test.ts`; neither was run. Stage A remained execution-free: no GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, test, packaging, deployment, installation or runtime acceptance occurred. The ledger adds `TDRP-R9-ACCOUNT-TELEGRAM-AUTH-LIFECYCLE-ERROR-BACKOFF-001`; `responsibilities_total=56`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain in force.


## Stage-A remaining Telegram auth lifecycle primitives (round 45, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact TL authority is `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a`; fixed-U intro behavior remains anchored by `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489`, `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e`, and `intro_email.cpp@dfb5ceeddeb48ab00773add72e75e0aef8ceb074`.

Canonical shared owner work is committed on `bhrumom/fabushi-platform-core` checkpoint `checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: protocol `314e8804ddfc997acdde2c19bff831b5e26b1706`, encrypted storage `bcb8298a4622616f2f8a226899c0e3dc8aa3ba8f`, runtime `ec854e93fb1523d95225203554600858798e4981`, and spec/evidence `a50f33da685506f4e170729a9d11f01f8dda8ec5`. These add auth.resendCode, auth.cancelCode, auth.requestPasswordRecovery/auth.checkRecoveryPassword, account.sendVerifyEmailCode/account.verifyEmail, explicit account.getPassword challenge refresh, and encrypted pending AuthorizationState/AuthContext persistence.

Desktop implementation `169d6cc7798507f0302d6c93fef7deef1a302d28` wires the matching seven commands through the existing `TelegramClientLease`, AppHost `feature.auth`, shared Coordinator contract and production renderer client. No second account/runtime/provider/state store was added, and no api_hash is exposed to renderer code.

The current desktop checkpoint is a sparse overlay and contains no Cargo/dependency manifest, so exact binding of its `fabushi_telegram_runtime` dependency to the platform-core checkpoint cannot be proven from this ref. That remains fail-closed integration evidence, not silently treated as shipping validation.

All protocol/storage/runtime/coordinator tests are authored only. No build, typecheck, lint, schema validator, generator, test, packaging, deployment, installation, workflow/status query or runtime acceptance was executed. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.

## Stage-A Telegram runtime Cargo composition fusion (round 46, 2026-11-01)

The previously recorded sparse-checkpoint composition blocker is now statically closed without executing validation. The actual Mahayana Cargo authority is preserved exactly: `source/mahayana/mahayana-rs/Cargo.toml@144c67c76daaf2229e817f9830e678a3e6d4cf3a` includes `mahayana-telegram` and `providers/telegram-runtime`; `mahayana-telegram/Cargo.toml@f43109356f1f486a82a2d0120f6e78808c431c40` path-depends on `fabushi-telegram-runtime = { path = "../providers/telegram-runtime" }`; and `providers/telegram-runtime/Cargo.toml@0fc8ee357ef5c00074e5fdcd1d7594860e289bd1` retains the sibling protocol/network/storage path dependencies.

Canonical shared-owner fusion is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16@13f5458aadf19a87c52bfa904f9942d81c88979a`. It preserves the prior `AuthorizationSnapshot/self_user_id` and resumable MTProto auth-key baseline while retaining resend/cancel/recovery/email-verification/account.getPassword challenge-refresh primitives. Protocol success parsing now carries canonical `auth.Authorization.user.id`; encrypted storage is schema v4 and converges both historical v3 table shapes (`private_metadata` and `secure_value`) without creating a second persistence owner.

Desktop commit `43d9d398801f2885da982d106c17a74efb1f9472` synchronizes those canonical protocol/network/storage/runtime sources into the existing `providers/telegram-*` path dependency chain; authored-only composition/runtime/storage/protocol contracts are saved through `221a661e17f2ad417e349df6604815d3fbfa946f`. The existing checkpoint `mahayana-telegram/src/lib.rs` lease/AppHost/Coordinator wiring is preserved; no second account, runtime, provider, session or state store is introduced.

This is Stage-A static implementation/composition evidence only. No GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was executed. `responsibilities_total=57`, `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged; `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain in force. Broader fixed-U source/implementation closure, complete Boost/libiconv archive member enumeration, original-host GitLab blockers, legal/release review, Stage-B executable evidence and independent ACCEPT remain open.


## Stage-A MTProto bad-message recovery (round 47, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The next build/shipping-reachable responsibility was manually read at `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, corroborated by `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. Fixed-U `bad_msg_notification` codes 16/17/64 correct trusted server time and resend, while 32/33 reset inconsistent session sequence state; unrelated/fatal codes do not silently recover.

Existing-owner-first implementation is committed in `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16@ac9e20217812aa72b92049d4dfb52f76d6380da6` after durable spec correction `47ed28d35dbb6c7dd19bdcf1cd66fbf88dc68f67`. Canonical blobs are `native/telegram-network/src/lib.rs@ba9935b54e9c178c7d9377cd75572f2c21b62144` and `native/telegram-protocol/src/session.rs@ab8fd351a05543854c3f62de5c8332a13b7ff472`. Recovery remains in the unique `EstablishedSession`: 16/17/64 update the live clock offset, clear only the client message-id floor while preserving server replay history, and retry once; 32/33 rotate the session id/reset `SessionSequence` and retry once; mismatched/fatal/repeated recovery fails closed.

Desktop implementation commit `6c714e664d6cd4d66228b313664a6e776a4fd558` synchronizes those exact blobs into the existing Cargo path-dependency chain at `providers/telegram-network` and `providers/telegram-protocol`; no second account, runtime, provider, transport, auth key, session owner or state store was introduced. Inline protocol/network recovery contracts are authored only and were not run.

The ledger adds `TDRP-R9-MTPROTO-BAD-MESSAGE-RECOVERY-001`; `responsibilities_total=58`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged because this is a responsibility slice rather than whole-file semantic closure. No GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was executed. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain in force. Broader fixed-U closure, archive/legal blockers, Stage-B executable evidence and independent release ACCEPT remain open.


## Stage-A MTProto new-session pending-RPC replay (round 48, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U read of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` lines around `mtpc_new_session_created`, corroborated by `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`, shows that the session adopts the new salt and resends pending requests whose message ids are older than `first_msg_id`.

The canonical shared owner remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `8275952bed7bea3a7c275f1b122c2c9c710ba262` precedes implementation commit `6ec2d64da2d30d78c919bd971e8e0d1953aed587`; the network blob is `7df3dd74ed1cc4fea5273a36152d1ff8bd00ec65`. Fabushi's synchronous `invoke_raw` has one outstanding RPC, so the fixed-U pending-request responsibility maps exactly to that request: if its id is less than `first_msg_id`, update `server_salt` and replay once with a fresh id on the same auth key/transport/runtime; equal/newer requests continue waiting; a repeated replay signal fails closed with `NewSessionReplayLimit`. Ping adopts the salt but does not fabricate content-RPC replay.

Desktop commit `01b78f13e4a500bdb401c080a40f5295d8110779` synchronizes the exact network blob into the existing `providers/telegram-network` path dependency. The ledger adds `TDRP-R9-MTPROTO-NEW-SESSION-REPLAY-001`; `responsibilities_total=59`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Higher-level propagation of `new_session_created` into update/difference recovery remains a separate open responsibility and is not claimed by this slice.

Inline predicate/replay contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, test, package, deployment, installation or runtime acceptance was performed. `accepted=false`, `validation_status=not-run`.


## Stage-A MTProto detailed-answer resend (round 49, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source authority remains `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` plus `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`: matching `msg_detailed_info#276d3ec6` acknowledges the sent request and requests missing `answer_msg_id` through `msg_resend_req#7d861a08`.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec `9bea551d1e26a4ff5efa522417999388256486ff`, implementation `0536aa2ad9f93e59f1b1e3cdc19aabaf9a104fce`, protocol api/lib blobs `1feaefd062f4e8635bbf002a7dce30b804c9f6ed` / `3e17847045f3be606d8b821c85f7b8136b89474a`, network blob `581156c32289082ad1f53410ab0637bdeef85c66`. The existing protocol owner adds a fixed-U `build_msg_resend_req`; the existing synchronous `EstablishedSession` sends that non-content service request only when `msg_detailed_info.msg_id` equals the current outstanding RPC, then keeps waiting.

Desktop `3f5e2d3a8b2e0c2726a1b1dcd1ab1172a0c5758d` synchronizes those exact blobs into the existing provider path chain. No second request history, runtime, session, provider or transport owner is introduced. `msg_new_detailed_info` and `msgs_state_req/info/all_info` full reconciliation remain explicitly open.

Ledger adds `TDRP-R9-MTPROTO-DETAILED-ANSWER-RESEND-001`; `responsibilities_total=60`, `unknown=16172`, `unread=16145`, `omitted=0`. Contracts are authored only; no Actions/status query or trigger, build, typecheck, lint, validator, generator, tests, packaging, deployment, installation or runtime acceptance occurred. `accepted=false`, `validation_status=not-run`.


## Stage-A MTProto single-RPC state reconciliation (round 50, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U source review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` plus `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` closed the subset of `msg_new_detailed_info`, `msgs_state_req`, `msgs_state_info`, `msgs_all_info`, and `handleMsgsStates` that Fabushi's existing one-outstanding-RPC owner can represent exactly. The upstream low-three-bit rule is preserved: `(state & 0x07) == 0x04` means the server received the request; other states require replay.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `469e46f72d15138aa4fb343a49e06feeaefb1303` defines the single-RPC boundary; implementation commit `404623cad89e0d77ab326e140784b893d677f31e` adds `msgs_state_req` encoding, bounded `msgs_state_info/msgs_all_info` parsing, one current state-query id, one bounded state-driven replay through the existing outer retry loop, and `msg_new_detailed_info` answer reconciliation through the existing server-message replay window. Canonical blobs are protocol api `ed695ed575c5950c99818acc7086a2cf8b1c712d`, protocol lib `21ec771260dd356d4a8e653be6e2e250a32ab5ce`, and network `c24515c2ee15608f51970a5420769e7713e60327`.

Desktop implementation commit `fe17d4bcdea22d1df8e474961aa12b5f04b291db` synchronizes those exact blobs into the existing provider path chain. No second request history, runtime, session, provider, transport or received-message store is introduced. Telegram Desktop semantics requiring arbitrary `haveSent`, `_resendingIds`, `_stateAndResendRequests`, simultaneous state queries, queued-resend cancellation, container-wide resend bookkeeping, or durable multi-request history remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-STATE-RECONCILIATION-001`; `responsibilities_total=61`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-RPC acknowledgement (round 51, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` selected the next service-message slice that the existing one-outstanding-RPC owner can represent exactly: `mtpc_msgs_ack` plus the `requestsAcked(..., byResponse=false)` rule that a normal ACK does not settle a callback RPC still waiting for a response.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `9f72142c7b4d8ab0e605e95301f4194d964ffa73` defines the single-RPC acknowledgement boundary; implementation commit `3470cd572891222c8846a9e91f98ca621624ec64` adds bounded `msgs_ack` parsing, explicit RPC/ping ACK dispatch, current-request correlation, malformed-payload fail-closed behavior, and callback-RPC non-settlement. Canonical blobs are protocol api `d38546a0f6954bb6a03aaec80968dbf195945edc`, protocol lib `da012dab3b1111f18c318cf0001f2f3d1f32bf6f`, and network `70645dd06426374ae4e20bd9c9545388948f65bb`.

Desktop implementation commit `6804906c0646d75a16b150b4f745476f44e19953` synchronizes those exact blobs into the existing provider path chain. No second request history, sent/resend queue, runtime, provider, transport, session or message store is introduced. Telegram Desktop semantics requiring arbitrary `haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, container ACK fan-out, queued-resend cancellation, or per-request fast-clock history remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-ACKNOWLEDGEMENT-001`; `responsibilities_total=62`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-ping pong correlation (round 52, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` selected the next adjacent service-message slice that the existing single-outstanding-ping owner can represent exactly: `mtpc_pong` / `pong#347773c5 msg_id:long ping_id:long`. Telegram Desktop additionally consults historical `wasSent`; Fabushi intentionally maps only the current-ping correlation that does not require a multi-request sent history.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `47246159887072673aff6609fb57f9cbc81ff2a2` defines the single-ping boundary; implementation commit `bf6f063b229be43597bddc4231baab24d06a09bd` adds bounded `parse_pong`, strict trailing-byte rejection, explicit `Pong` shape, and `pong_matches_request` correlation requiring both the current request message id and ping id. Canonical blobs are protocol api `32a7610bfc74835734c1e5c4385e4eb4cb2ee1f3`, protocol lib `ffbabc8f66112c2321be75a34928b086d6adf57f`, and network `e0bf1b5f286ad0f8bc45a36e86074e19b7dde560`.

Desktop implementation commit `3e363ccdaf287878f457797442aa97e8b597384b` synchronizes those exact blobs into the existing provider path chain. A valid exact pong settles only the current ping; either id mismatch uses the existing `PingMismatch` failure path, and malformed/trailing payload fails closed as `InvalidServiceMessage`. No second request history, ping history, runtime, provider, transport, session or message store is introduced. Telegram Desktop semantics requiring arbitrary `haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, historical sent-message lookup, container fan-out or queued-resend cancellation remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-PING-PONG-001`; `responsibilities_total=63`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-RPC result correlation (round 53, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` selected the next adjacent service-message slice that the existing one-outstanding-RPC owner can represent exactly: `mtpc_rpc_result` / `rpc_result#f35c6d01 req_msg_id:long result:Object`. Telegram Desktop additionally consults historical `wasSent` and multi-request acknowledgement state; Fabushi intentionally maps only current-request correlation and leaves that history open/fail-closed.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `6511c5f887c16caa0494918c987fb400774c4349` defines the single-RPC-result boundary; implementation commit `144512ce2bb7366aef57ba76924b2d02852cbce3` adds bounded `parse_rpc_result`, explicit `RpcResultEnvelope`, validation of non-empty word-aligned opaque result bytes, and `rpc_result_matches_request` correlation against only the current request. Canonical blobs are protocol api `b8b9872ec8ce7548817ebbe92615d47aed470e89`, protocol lib `4b6695cac59dc507f76c85f93ff3678d0d5bcbd0`, and network `8ec039d4a12040fcea87af883dba9cd02a2b3efb`.

Desktop implementation commit `7dc33be41568162c2cab398da67b3e6bc9c12dea` synchronizes those exact blobs into the existing provider path chain. A well-formed result settles only when `req_msg_id` equals the current `request_message_id`; unrelated result ids continue waiting, and malformed/empty/non-word-aligned envelopes fail closed as `InvalidServiceMessage`. No second request history, callback registry, runtime, provider, transport, session or message store is introduced. Telegram Desktop semantics requiring arbitrary `wasSent/haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, container fan-out, queued-resend cancellation or multi-request callback routing remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-RESULT-001`; `responsibilities_total=64`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-RPC message-container traversal (round 54, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` selected the adjacent `mtpc_msg_container` receive-side branch. Fixed U requires a bounded contained-message count, per-entry `msg_id/seq_no/bytes`, body length of at least one TL word and word alignment, ACK collection for odd `seq_no`, replay registration, and recursive service handling. This round maps only the receive-side subset that Fabushi's existing current RPC/current ping owners can represent; Telegram Desktop's outbound `_sentContainers` and arbitrary historical request fan-out remain open/fail-closed.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `b223407bc9829a2fc14449bb5a7cfe6d7f4b082e` defines the bounded receive-container boundary; implementation commit `98a2ff425407725571966a4ed17b1643fba87b4d` adds `parse_message_container`, `MessageContainer/MessageContainerEntry`, 1024-entry bounding, minimum/aligned body checks, exact full-buffer consumption, and reuses the typed parser in both current-RPC and current-ping network paths. Canonical blobs are protocol api `14c282bfc20a33446c69a47db4fdd6f0b887348a`, protocol lib `f21d38495e5a7f5c6e2a35f28a633ac6e3433d4b`, and network `bab8893bd8a2a0632014d826503a8091cfe64d46`.

Desktop implementation commit `e8091133b85eb7e368839e6c9b09547620be17c2` synchronizes those exact blobs into the existing provider path chain. The existing `MessageIdGuard` still validates every contained server message id before recursion; the RPC path adds only odd-sequence contained ids to its existing acknowledgement accumulator; nested results/pongs/recovery continue through existing exact current-request/current-ping correlation. No second request history, sent-container store, callback registry, runtime, provider, transport, session or message store is introduced.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-MESSAGE-CONTAINER-001`; `responsibilities_total=65`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Parser/sequence-parity contracts are authored only. `gzip_packed` decompression nesting/budget semantics and outbound/multi-request `wasSent/haveSent/_resendingIds/toSend/_sentContainers` remain open and fail-closed. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto new-detailed-info exact wire validation (round 55, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` confirmed the adjacent `mtpc_gzip_packed` path requires `kMaxGzipNesting=64`, a shared `unpackedBudget` initialized to 32 MiB, and zlib/gzip decompression. The canonical `native/telegram-protocol/Cargo.toml@d8a5261034b1eb395d12149a6bcc34cd7e371b73` and `Cargo.lock@97ad9046ce8926764258c978664cd8cd92fce503` contain no approved `flate2`/\`miniz_oxide` decompression owner, so gzip remains explicitly fail-closed rather than introducing an unreviewed dependency.

The next independently implementable responsibility is the complete fixed-U wire contract for `msg_new_detailed_info#809db6df answer_msg_id:long bytes:int status:int` from `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. Canonical spec commit `40d53cfb44f2a495e4bf1a84002f653f555532e5` precedes implementation commit `5bdf993dc1048ed22194d7f1c0439049e820ee5f`. The existing protocol owner now exports `MsgNewDetailedInfo`, `MSG_NEW_DETAILED_INFO_CONSTRUCTOR`, and `parse_msg_new_detailed_info`; the existing `EstablishedSession::scan_rpc_body` uses that parser before the existing detailed-answer ACK/resend reconciliation. Canonical blobs are protocol api `eff20dbc601ca78fa7e6a4662bfeda088706d4f4`, protocol lib `b6ab0397cbad0776b269d965f89427030824ba4c`, and network `7f3b140a9becda2305a2f5e252fcea61108597e0`.

Desktop implementation commit `3b37ff49e7c3b100af12206ab88662f4d83959c2` synchronizes those exact blobs into the existing provider path chain. Truncated, wrong-constructor, or trailing `msg_new_detailed_info` payloads now fail as `InvalidServiceMessage` before an ACK or `msg_resend_req` side effect; a valid message still reuses the current session replay window and existing resend primitive. No second request history, decompression runtime, provider, transport, session or message store was introduced.

Ledger adds `TDRP-R9-MTPROTO-NEW-DETAILED-INFO-WIRE-001`; `responsibilities_total=66`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact-field and malformed-payload contracts are authored only. `gzip_packed`, `wasSent/haveSent/_resendingIds/toSend/_sentContainers`, higher-level new-session update/difference propagation, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto new-session wire validation and high-level difference blocker (round 56, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review bound the low-level service object to `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `new_session_created#9ec20908 first_msg_id:long unique_id:long server_salt:long` in `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. The adjacent high-level chain is `Api::Updates::mtpNewSessionCreated -> _updatesSeq=0 -> getDifference`; full/slice differences are applied before intermediate/final state advancement in `api_updates.cpp@71c929cfe3c779c94719efef32a47a4d029e2c67`.

Existing-owner-first review found the high-level recovery is not yet precisely implementable: canonical `telegram-protocol/src/updates.rs` can build `updates.getDifference` and parse only update state plus terminal empty/too-long differences, while no current protocol/runtime bridge decodes and applies arbitrary `Message/EncryptedMessage/Update/Chat/User` vectors from `updates.difference#f49ca0` or `updates.differenceSlice#a8fb1981` into the existing TelegramEngine owners. A terminal-only recovery would drop missed updates or advance state without applying them, so this responsibility is recorded fail-closed in platform-core spec `telegram-new-session-update-difference-recovery.md` from spec commit `ba9143b660d58728de8b93a49e7c2f97fcd39ed5`.

The next independent responsibility, exact new-session wire validation, is implemented instead. Platform-core implementation `dc66cc7aae7644530fa03160438df5174809beb4` adds `NEW_SESSION_CREATED_CONSTRUCTOR`, `NewSessionCreated`, and `parse_new_session_created` to the existing protocol updates owner, consuming all three fixed-U fields and rejecting wrong constructors, truncation, and trailing bytes. Both `EstablishedSession::scan_rpc_body` and `process_service_body` now parse the complete typed object before server-salt mutation or the existing bounded current-RPC replay decision. Canonical blobs are protocol updates `23a09bb6e4e78ef526823cf01d0a207565fb2a3e`, protocol lib `7b9c641eabccb7dba8c5a425904b6a676b12d866`, and network `234c6c0070a1a92218fe5c974658d16d37fee46a`.

Desktop implementation `23b24ab2db29697071d21eb747f475b2196d891f` synchronizes those exact blobs into the existing provider composition. The provider protocol tree did not previously contain `src/updates.rs` even though its synchronized `lib.rs` declares `pub mod updates`; this round adds the canonical file at that declared existing module slot rather than introducing a second owner.

Ledger adds `TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001`; `responsibilities_total=67`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact-field/malformed contracts are authored only and were not executed. High-level full/slice difference recovery, `gzip_packed`, historical `wasSent/haveSent/_resendingIds/toSend/_sentContainers`, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed. No Actions/status/check query or trigger, build, typecheck, lint, validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto detailed-info exact wire hardening (round 57, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirmed that `msg_detailed_info#276d3ec6` is a four-field object (`msg_id:long answer_msg_id:long bytes:int status:int`) and Telegram Desktop performs generated TL decoding before using it. The existing Fabushi network branch still read only the first two fields by raw offsets, so malformed/trailing objects could reach the current-session ACK/resend path.

Spec-first canonical work remains on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `8fffe0ca4e7d48d097e9f15f0cf332610e28f4fc`, implementation commit `4c5bea7b847a3e04ee3d0c9b05fab228101acb64`, and source-level compliance commit `46cf745e2c228f0638d2d0afa2856579ddba3d70`. Canonical blobs are protocol api `5da871fc557a49a7e57f1dd0677f307a5ed1bac8`, protocol lib `3f7c363f9fd2eb86e60c8294144d7f1dced9ccca`, and network `2a50068d148b25e710961973315d78fc1b1ee5a8`. The protocol owner now exports `MSG_DETAILED_INFO_CONSTRUCTOR`, `MsgDetailedInfo`, and `parse_msg_detailed_info`; wrong constructor, truncation, and trailing bytes fail before network side effects. The network owner parses the complete object first, then keeps the existing one-outstanding-RPC current-id gate and `reconcile_detailed_answer` behavior.

Desktop implementation commit `45c24752b9d4764768a9eec26ead9fb6b36dc426` synchronizes those three blobs byte-for-byte into the existing provider chain; no second Telegram protocol/runtime/provider, request history, callback registry, or received-message store was introduced. The existing ledger responsibility `TDRP-R9-MTPROTO-DETAILED-ANSWER-RESEND-001` is updated rather than creating a duplicate responsibility; `responsibilities_total=67`, `unknown=16172`, `unread=16145`, `omitted=0` remain unchanged.

Exact-field, wrong-constructor, truncation, trailing-byte, and current-request contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference decoding/propagation, and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain explicitly open and fail-closed.


## Stage-A MTProto future-salts exact wire validation (round 58, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Reread of `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms `future_salts#ae500895 req_msg_id:long now:int salts:vector<future_salt>` and nested `future_salt#0949d9dc valid_since:int valid_until:int salt:long`. The existing Fabushi receive branch still recognized `ae500895` locally, read only `req_msg_id` from raw offset 4, and returned the entire body for a matching current request without validating `now`, the TL vector, nested constructors/fields, or trailing bytes.

Existing-owner-first review found this slice is independently representable by the canonical one-outstanding-RPC owner: complete response validation and current-id correlation require no historical `wasSent/haveSent/_resendingIds/toSend/_sentContainers` state. Platform-core spec commit `69167c52582d67df73c2284bb1e98a47c3846343` defines that boundary. Implementation commit `f631d7cc0c1a21db93ab647d3b6dcb29b42cb74e` adds `FUTURE_SALT_CONSTRUCTOR`, `FUTURE_SALTS_CONSTRUCTOR`, typed `FutureSalt/FutureSalts`, and `parse_future_salts` in the existing protocol owner. It validates top-level/vector/nested constructors and all fields, checks the declared count's fixed-size byte requirement against remaining input before vector allocation, and rejects truncation or trailing bytes. The network branch parses first, then settles only when the typed `request_message_id` equals the one outstanding RPC, preserving the exact validated raw object as the existing `RpcResult.body`. Canonical blobs are api `bdac614fab9ca16ffeb36c7e98a012fe20231597`, lib `6b6bde59a9f5e86ec33b0258e1e6a34f2b1622f9`, network `b2eb7022c6a8d38d7ac9f4a9fd13fe22913621de`; source-level compliance commit is `f604e40e175d6a05ef10d6c4811ec1e363c85529`.

Desktop implementation commit `a3329a8c948cbffab1d6843c78abc8024afa87b4` synchronizes those three blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-FUTURE-SALTS-WIRE-001`; `responsibilities_total=68`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No second Telegram runtime/provider/request history was added.

Exact-field, wrong top-level/vector/nested constructor, truncation, impossible-count, trailing-byte, and current-request contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain explicitly open/fail-closed.


## Stage-A MTProto bad-server-salt exact wire validation (round 59, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms that `bad_server_salt#edab447b` is a four-field object: `bad_msg_id:long bad_msg_seqno:int error_code:int new_server_salt:long`. Telegram Desktop first decodes the generated TL object and then consumes all four fields. The existing Fabushi RPC and ping/service branches still used raw offsets for only `bad_msg_id`, `error_code`, and `new_server_salt`, leaving `bad_msg_seqno` untyped and allowing trailing bytes before live salt mutation.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `12bc8a0ce1e453af1b3f2af0ffbd9ce868c92447`, implementation commit `3a850171c59a5f27809e176ba54971c7c0c003dc`, and final source-level compliance commit `515857196fd5fd8688326c7971a60079b67a0b71`. Canonical blobs are protocol api `75959eb82460cb36bf5f43c6e4470ef8814ea2a9`, protocol lib `3540f1a60df0d07f77b6e38349e0c500ec2d12d8`, and network `42be9e9fa78c3acc77367799c1ecb506cdd25ab0`. The protocol owner now exports `BAD_SERVER_SALT_CONSTRUCTOR`, typed `BadServerSalt`, and `parse_bad_server_salt`; it consumes `bad_msg_id`, `bad_msg_seqno`, `error_code`, and `new_server_salt`, and rejects wrong constructors, truncation, and trailing bytes. Both network receive branches parse first, then correlate only the one outstanding request before assigning the exact parsed server salt and preserving the existing bounded `NewSalt` retry outcome.

Desktop implementation commit `131980d0bc3c5b6e8fad3ad2e394715426cd1529` synchronizes those three canonical blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-BAD-SERVER-SALT-WIRE-001`; `responsibilities_total=69`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No second Telegram runtime/provider/request history was introduced.

Exact-field, wrong-constructor, truncation, trailing-byte, both-receive-branch, and current-request contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain explicitly open/fail-closed.


## Stage-A MTProto bad-msg-notification exact wire validation (round 60, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms `bad_msg_notification#a7eff811 bad_msg_id:long bad_msg_seqno:int error_code:int`; Telegram Desktop first decodes the generated TL object and then reads all three fields. The existing Fabushi RPC and ping/service branches still read only `bad_msg_id` at raw offset 4 and `error_code` at raw offset 16, while `direct_bad_message_time_recovery` also granted its clock-window exception from raw constructor/error-code offsets. This skipped typed `bad_msg_seqno` validation and allowed trailing bytes to reach recovery behavior.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `42d90d6351a31f6d0815c47f2d647ae8162eccf8`, implementation commit `0e569130a80a53b48b74f8e27a96004363b30e7e`. Canonical blobs are protocol api `15e34bbb0e9c5a5d4fdd8b7a9cdedf3d6b123080`, protocol lib `2c4897cfdba4502d8ca880d7a58cfd0e88877bf7`, and network `1fc1b1f69e3e37377400d078f916599e0c7b20bc`. The protocol owner now exports `BAD_MSG_NOTIFICATION_CONSTRUCTOR`, typed `BadMsgNotification`, and `parse_bad_msg_notification`; it consumes `bad_msg_id`, `bad_msg_seqno`, and `error_code`, rejecting wrong constructors, truncation, and trailing bytes. Both network receive branches parse first before current-request correlation or recovery action, and the direct clock-window exception now requires the same fully valid typed object. The decoded sequence number is retained without introducing historical sent/container reconciliation.

Desktop implementation commit `1205f8709995aa4f5fd5dafdc9fb5121fbfce06c` synchronizes those three canonical blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-BAD-MSG-NOTIFICATION-WIRE-001`; `responsibilities_total=70`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No second Telegram runtime/provider/request history was introduced.

Exact-field, wrong-constructor, truncation, trailing-byte, current-request correlation, recovery-code mapping, and malformed-clock-window contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain explicitly open/fail-closed.


## Stage-A MTProto RpcDropAnswer exact wire validation (round 61, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `rpc_answer_unknown#5e2ad36e`, `rpc_answer_dropped_running#cd78e586`, and `rpc_answer_dropped#a43ad8b7 msg_id:long seq_no:int bytes:int`. The fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct `rpc_answer_*` branch; these are schema-defined `rpc_drop_answer` result bodies.

Spec-first canonical work: platform-core spec `0c85884ee491d985e4fbf0d6ec1d1acca1a7d8a4`, implementation `5251ab3eacd8035cb8bb86609d76641233dd22eb`; canonical blobs api `77b9cc0a7d191ef9f197fb8fd718e29631197ddc`, lib `9229c83fa6b484a5af657ee37f0bd827fecdc28d`. The existing protocol owner now has typed `RpcDropAnswer` variants and `parse_rpc_drop_answer`, with exact zero-field/trailing validation for Unknown/DroppedRunning and exact `msg_id/seq_no/bytes` parsing plus truncation/trailing rejection for Dropped. Desktop implementation `4257fc9c875526c4b49a1f3cfbbeec7cd5a3eb0f` synchronizes both protocol blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001`; `responsibilities_total=71`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No outbound `rpc_drop_answer` production call path or historical request registry exists, so interpreting these results as drop/cancel state mutations remains explicitly fail-closed; no `wasSent/haveSent/_resendingIds/toSend/_sentContainers` semantics were invented.

Parser contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.


## Stage-A MTProto DestroySessionRes exact wire validation (round 62, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `destroy_session_ok#e22045fc session_id:long`, `destroy_session_none#62d350c9 session_id:long`, and `destroy_session#e7512126 session_id:long`. The fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct `destroy_session_*` receive branch, so wire decoding is separated from any live teardown lifecycle.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `dee7cff150070d7891478510386674970a6217af`, implementation `cfe6a0564c40d2c9b9ea62b686470139a1eeb2ab`; canonical blobs api `d82cca134274b8d939c1140ec44ee00230e2cfac`, lib `655498ce9db2faf8a76f093250d4a1f02f77ff67`. The existing protocol owner now exports typed `DestroySessionResult::{Ok,None}` and `parse_destroy_session_result`; both variants preserve exactly one `session_id:long` and reject unknown constructors, truncation, and trailing bytes. Desktop implementation `b179796344fba3e22bccb1310b74f229d613b531` synchronizes both protocol blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-SESSION-WIRE-001`; `responsibilities_total=72`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No canonical outbound `destroy_session` production path or live teardown owner exists, so interpreting Ok/None as session mutation remains explicitly fail-closed; no `wasSent/haveSent/_resendingIds/toSend/_sentContainers` semantics were invented.

Parser contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.


## Stage-A MTProto http_wait exact wire validation (round 63, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `http_wait#9299359f max_delay:int wait_after:int max_wait:int = HttpWait`. Fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` constructs `MTP_http_wait(100, 30, 25000)` only when `_connection->usingHttpWait()`; no independent receive-side `mtpc_http_wait` service-message branch was found. The call-site values are therefore recorded as lifecycle evidence, not treated as schema validity constraints.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `03b40ed111c8e64d159ea837848bfae0fa85102c`, implementation commit `96bb0e2cd31cdfc82aaa44e0f8e25acf8c30a204`; canonical blobs api `7b731b6d6bbd2009453c5107a24ebea58cc2ece3`, lib `4d8b7198fbe7aa8cbb71180587970760f6ebd1d9`. The existing protocol owner now exports `HTTP_WAIT_CONSTRUCTOR`, typed `HttpWait { max_delay, wait_after, max_wait }`, and `parse_http_wait`; it consumes exactly constructor + three TL ints and rejects wrong constructors, every-byte truncation, and trailing bytes. The parser is side-effect free and introduces no connection/session/request mutation.

Desktop implementation commit `1495975bbdd2e5d56826a95854def198862e2c5b` synchronizes both protocol blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-HTTP-WAIT-WIRE-001`; `responsibilities_total=73`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Telegram Desktop-equivalent `usingHttpWait()` connection scheduling is not represented by a proven canonical owner, so that lifecycle remains explicitly fail-closed; no second HTTP scheduler/runtime/provider or `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history was invented.

Exact-field, wrong-constructor, every-byte truncation, and trailing-byte contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary historical sent/resend/container state, broader fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain open/fail-closed.


## Stage-A MTProto help.configSimple exact wire validation (round 64, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares adjacent `ipPort#d433ad73`, `ipPortSecret#37982646`, `accessPointRule#4679b65f`, and `help.configSimple#5a592a6c`. `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no help.configSimple receive/application branch. The actual fixed-U higher-level consumer was reread at `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` / `special_config_request.h@91901489e778089b03187682a9556e929867b94f`: after decrypting a simple config it checks expiry and non-empty rules, applies phone-prefix filtering, converts raw IPv4 values, and emits DC endpoint callbacks. Those lifecycle/policy steps are intentionally not inferred by the wire parser.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `db4e6db92d6ebf6f03442cb3eb2134b9bf0a144e`; implementation commit `cde1297cacd3861764d74a111629d002e74c9d71`; constructor-first fail-closed correction `46db4422aa7221b1c8048f96f1c7c47cd0c01458`. Canonical blobs are api `0c852a5e3bc41a4ac571d713ae3d4edea19b3619` and lib `04e5020e7b1e7ed46b403c151f84a095a1a17e4c`. The existing protocol owner now exports all four fixed-U constructors, typed `IpPort`, `AccessPointRule`, `HelpConfigSimple`, and exact parsers for each object family. Nested vectors use the existing bounded TL reader; raw signed ints, strings, bytes and ordering are preserved; wrong constructors, malformed nested framing, truncation, and trailing bytes fail closed.

Desktop implementation commit `870ec15e7760718ad7ef1eb5e578dfcef8f0e46b` synchronizes both protocol blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-CONFIG-SIMPLE-WIRE-001`; `responsibilities_total=74`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Telegram Desktop-equivalent special-config web acquisition/decryption/HTTP-date/expiry/phone-prefix/DC endpoint application is not represented by a proven canonical owner, so that lifecycle remains explicitly fail-closed; no second bootstrap runtime/provider/DC owner or `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history was invented.

Both IpPort variants, nested rule/config preservation, wrong-constructor, every-byte truncation, malformed nested constructor and trailing-byte contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary historical sent/resend/container state, broader fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain open/fail-closed.


### Round 64 final authored-contract and compliance readback

After the initial parser synchronization, canonical authored-only negative contracts were expanded at platform-core commit `022b578b8f7823a9af737a2f88fdf4c684fa4e56` to explicitly cover wrong TL vector constructors and invalid UTF-8 `phone_prefix_rules`; no contract was executed. Final canonical blobs are api `0c852a5e3bc41a4ac571d713ae3d4edea19b3619` and lib `04e5020e7b1e7ed46b403c151f84a095a1a17e4c`. Desktop final contract-sync commit `878329a3f54895da0c44ab33e14cf8856444e128` has byte-identical provider api/lib blobs.

Platform-core spec compliance review commit `d1c85e53a4cb162ef02f5344fcadf3c4a3476f6a` updates `docs/specs/telegram-mtproto-config-simple-wire.md@f7dd300a5a5f49fe6958617817f95629dcf95e8f`: AC1-AC7 are satisfied at source/implementation/bookkeeping level; AC8 remains blocked/not-run for Stage B executable validation. Special-config acquisition/decryption/expiry/phone-prefix/DC endpoint application remains fail-closed.


## Stage-A MTProto ping_delay_disconnect exact wire serialization and TLS-template blocker (round 65, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Reread of `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms that the declarations immediately after `help.configSimple` are `tlsClientHello/tlsBlock*`, followed by `ping#7abe77ec` and `ping_delay_disconnect#f3427b8c ping_id:long disconnect_delay:int`. Fixed-U `mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16` shows that `tlsClientHello/tlsBlock*` are not inbound MTProto response objects: `PrepareClientHelloRules` constructs a local typed template AST and `Generator::Part::writeBlock` consumes it to generate TLS camouflage ClientHello bytes, including random blocks, X25519 public-key material, HMAC digest/timestamp injection, scoped lengths and permutations. The current canonical `native/telegram-network` owner exposes direct `TcpTransport` only and has no equivalent TLS camouflage socket/generator lifecycle, so no artificial TL parser was introduced; that lifecycle remains fail-closed/open.

The next independently representable responsibility is exact `ping_delay_disconnect` request serialization. Fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` serializes it on the main connected session using the current ping id and `kPingDelayDisconnect=60`, while session timers decide when to send it. Platform-core spec commit `c98a668a64a25c516065f0dda938200290712018` separates exact wire layout from those timer/session semantics. Implementation commit `cb816c7540ea93717238b3aec96372cdab1c1706` adds `PING_DELAY_DISCONNECT_CONSTRUCTOR=0xf3427b8c` and `build_ping_delay_disconnect` to the existing telegram-protocol owner. Canonical blobs are api `405b04166bea039051833a38ad0a75c62107f70c` and lib `ef7fe73335551e303fa71b02f5cf066d778ae8a4`; the builder emits exactly constructor + little-endian `i64 ping_id` + little-endian `i32 disconnect_delay`, preserves signed boundary values, and performs no timer/socket/session/request-history mutation.

Desktop implementation commit `d38f6bef11a9a4334f8387c73c7b74e7994c0cdb` synchronizes both protocol blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-PING-DELAY-DISCONNECT-WIRE-001`; `responsibilities_total=75`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. The Telegram Desktop main-session/timer policy for deciding when to use `ping_delay_disconnect` remains open/fail-closed rather than being invented by a wire-only builder.

Exact-layout and signed-boundary contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. TLS camouflage ClientHello generation/socket lifecycle, special-config acquisition/decryption/expiry/phone-prefix/DC endpoint application, `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.


### Round 65 source-level compliance readback

Platform-core compliance commit `6cfc81d79901493c6b34ec4e174d4fbdae81800e` updates `docs/specs/telegram-mtproto-ping-delay-disconnect-wire.md@aaa7c8b0290264b162405f35285a777fb37df91f`. Source inspection records R1-R10 and AC1-AC6 satisfied at source/implementation/bookkeeping level; AC7 remains Stage-B blocked/not-run. Remote blob readback confirms canonical and desktop provider api/lib remain byte-identical at `405b04166bea039051833a38ad0a75c62107f70c` / `ef7fe73335551e303fa71b02f5cf066d778ae8a4`. No runtime claim is made for the TLS camouflage ClientHello generator/socket lifecycle or the main-session ping timer/scheduling policy.


## Stage-A MTProto destroy_session exact request serialization (round 66, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `destroy_session#e7512126 session_id:long = DestroySessionRes`. The existing `DestroySessionRes` parser remains a separate response responsibility. Platform-core spec amendment `6f1ecac8f9284405e49dfc69af20270f914d2c83` and implementation `5f259ca296432384f5388ffef9539dbbf1f4033a` add only `DESTROY_SESSION_CONSTRUCTOR` plus exact `build_destroy_session(session_id:i64)`; canonical blobs are api `2f0c2893e256064f822082dd2fd7deae394b5763`, lib `05c690a72f5f9b5a547f10486a5289229d7bb0ff`. Desktop implementation `28d36535b4d4d342b72c6a0e35752ce56dbcfd26` synchronizes those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-SESSION-REQUEST-WIRE-001`; `responsibilities_total=76`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 12-byte layout, signed-boundary preservation, and request/result-parser separation contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.

Live outbound destroy-session/teardown lifecycle remains fail-closed because no precise existing owner currently represents it. TLS camouflage lifecycle, gzip, complete difference/differenceSlice decoding/application, and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain open/fail-closed.


## Stage-A MTProto get_future_salts exact request serialization (round 67, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `get_future_salts#b921bd04 num:int = FutureSalts`; the existing `FutureSalts` parser/current-request correlation remains a separate response responsibility. Platform-core spec amendment `5a52c2a1c53efbe9ce396919598a934dc732a8d7` and implementation `58a9635f0286283218f6d8b7b2b14dd466bd4b98` add only `GET_FUTURE_SALTS_CONSTRUCTOR` plus exact `build_get_future_salts(num:i32)`; canonical blobs are api `e8026c79b8b14261ef30d39bf6de8b0bc92306a6`, lib `809eeb7aa3256e1aa259e694f413b9c0377f08d6`. Desktop implementation `dd232c52e1b39cdccb409c6d6307f1523e4d538f` synchronizes those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-GET-FUTURE-SALTS-REQUEST-WIRE-001`; `responsibilities_total=77`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 8-byte layout, signed-`i32` boundary preservation, and request/FutureSalts-parser separation contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.

Live `get_future_salts` scheduling and returned future-salt installation/rotation remain fail-closed because no precise existing owner currently represents those timer/session/history semantics. TLS camouflage lifecycle, gzip, complete difference/differenceSlice decoding/application, and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain open/fail-closed.


### Round 67 source-level compliance readback

Platform-core compliance commit `eaf1eeefdde1401d820d9fecd15da3b4aa9cfde5` updates `docs/specs/telegram-mtproto-future-salts-wire.md@4b690dcafce939c13540d1ff0a55a66bed1138bb`. Source inspection records R11-R17 and AC9-AC12 satisfied at source/implementation/bookkeeping level; AC13 keeps executable validation blocked/not-run with `accepted=false`. Remote source readback confirms canonical and desktop provider api/lib remain byte-identical at `e8026c79b8b14261ef30d39bf6de8b0bc92306a6` / `809eeb7aa3256e1aa259e694f413b9c0377f08d6`. Live `get_future_salts` scheduling and future-salt installation/rotation remain explicit fail-closed scope.


## Stage-A MTProto ping exact request serialization (round 68, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `ping#7abe77ec ping_id:long = Pong`; the existing strict `Pong` parser/current-ping correlation remains a separate receive responsibility. Fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` selects ordinary `MTPPing` only from connection/main-session/timer state, so that scheduling/selection lifecycle is not synthesized by this wire-only slice. Platform-core spec amendment `cd40329f3fbc39c06599e3b5d4dda63f09ae6b17` and implementation `d6ef74975d3541ddb4d187bfb5641773dc01a32d` add only `PING_CONSTRUCTOR=0x7abe77ec` plus exact `build_ping(ping_id:i64)`; canonical blobs are api `714c24a5f1727e13693008ea42676831b3e8d3fd`, lib `ef44b1bb7fcac0183f1b83cca8c9d994ecc8ce9c`. Desktop implementation `36b425e547509f334453ac5ebf09033e366fe6ae` synchronizes those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-PING-REQUEST-WIRE-001`; `responsibilities_total=78`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 12-byte layout, signed-`i64` boundary preservation, and request/Pong-parser separation contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.

Live ordinary-ping scheduling/selection, message-id assignment, send/ACK settlement, timeout/reconnect lifecycle and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain fail-closed. TLS camouflage lifecycle, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.


### Round 68 source-level compliance readback

Platform-core compliance commit `bf654c6501776fb46a8e40bad5f9aacea4612514` updates `docs/specs/telegram-mtproto-single-ping-pong.md@4faf7eb9c9a6c7b9e5988c850c9068c9a0a21d79`. Source inspection records request-side R9-R16 and AC-6 through AC-9 satisfied at source/implementation/bookkeeping level; AC-10 keeps executable validation blocked/not-run with `accepted=false`. Remote source identities are canonical/desktop api `714c24a5f1727e13693008ea42676831b3e8d3fd` and lib `ef44b1bb7fcac0183f1b83cca8c9d994ecc8ce9c`. Live ordinary-ping scheduling/selection, message-id assignment, send/ACK settlement, timeout/reconnect, arbitrary historical sent/resend/container state, TLS camouflage, gzip and complete difference decoding remain explicit fail-closed scope.


### Round 69 — rpc_drop_answer request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `rpc_drop_answer#58e4a740 req_msg_id:long = RpcDropAnswer`; reread `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct `rpc_drop_answer` branch, so no historical scheduling/correlation lifecycle is inferred.

Platform-core spec-first amendment `fde4b1eadc3d03c1dd943bcb5170e3434ff2771a` and implementation `37fff2cb24658b6e0f738bbbd23cecdf075f4bb1` add only `RPC_DROP_ANSWER_CONSTRUCTOR=0x58e4a740` plus exact `build_rpc_drop_answer(req_msg_id:i64)`. Canonical blobs are api `3ec13cd2248e68f55296ed9bbaffa5c10ffce969`, lib `e4705eb4563a087e0bba811948f1244af8f9f5cb`; desktop synchronization `7c17952b90cc89d575345e192733c2e6c23deb03` copies them byte-for-byte.

Ledger adds independent request responsibility `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-REQUEST-WIRE-001` while retaining `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001` as the response-parser responsibility; `responsibilities_total=79`, `unknown=16172`, `unread=16145`, `omitted=0`. Exact 12-byte layout, signed-`i64` boundary preservation, and request/response-parser separation contracts are authored only. `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Historical outstanding-RPC selection/correlation/drop mutation and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` state remain fail-closed; gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


### Round 69 source-level compliance readback

Platform-core compliance commit `510b75727303147a06b6276fa94f7dcd3a7cb31b` updates `docs/specs/telegram-mtproto-rpc-drop-answer-wire.md@f8408584b8ac997cad97d213fe90eb18155544f1`. Remote readback confirms canonical and desktop provider api/lib blobs are byte-identical at `3ec13cd2248e68f55296ed9bbaffa5c10ffce969` / `e4705eb4563a087e0bba811948f1244af8f9f5cb`; `RPC_DROP_ANSWER_CONSTRUCTOR=0x58e4a740`, public `build_rpc_drop_answer(req_msg_id:i64)`, and the pre-existing `parse_rpc_drop_answer` response parser are all present. Request-side R9-R16 and AC7-AC9 are satisfied at source/implementation/bookkeeping level; AC10 executable validation remains blocked/not-run with `accepted=false`. Historical outstanding-RPC selection/correlation/drop mutation, arbitrary sent/resend/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


### Round 70 — destroy_auth_key request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `destroy_auth_key#d1435160 = DestroyAuthKeyRes`. The existing shared `telegram-protocol` api owner can represent this zero-field request independently; auth-key eligibility, result handling, local key deletion, persistence and teardown/reconnect lifecycle remain fail-closed.

Platform-core spec-first commit `529f5c9147f0144b67eb077733b52d61b469f55d` and implementation `8eafd0cc7a84ab9af6b402665670b38f3ad72406` add only `DESTROY_AUTH_KEY_CONSTRUCTOR=0xd1435160` plus exact `build_destroy_auth_key()`. Canonical blobs are api `fbca9b0ebb85d0edd5c01ceea9896d0977294b53`, lib `ed581d8356375e264fabe85a2e5cccbd9b772f21`; desktop synchronization `fe0093d767ad2a12e049b4d0a474c0145d3468ab` copies them byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-REQUEST-WIRE-001`; `responsibilities_total=80`, `unknown=16172`, `unread=16145`, `omitted=0`. Exact four-byte/no-payload contract is authored only. `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Auth-key destruction lifecycle, arbitrary sent/resend/container history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


### Round 70 source-level compliance readback

Platform-core compliance commit `7643f936cd2b40aa628041880565c88539a1aace` updates `docs/specs/telegram-mtproto-destroy-auth-key-request-wire.md@ca43a6b328e68d68dfbfd7c9bae90c614918a046`. Remote readback confirms canonical and desktop provider api/lib blobs are byte-identical at `fbca9b0ebb85d0edd5c01ceea9896d0977294b53` / `ed581d8356375e264fabe85a2e5cccbd9b772f21`; `DESTROY_AUTH_KEY_CONSTRUCTOR=0xd1435160` and public `build_destroy_auth_key()` are present. R1-R5 are satisfied at source/ownership/bookkeeping level; R6 executable validation remains blocked/not-run with `accepted=false`. Auth-key destruction eligibility/result handling/local deletion/persistence/teardown, arbitrary sent/resend/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open.

### Round 71 — req_pq request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `req_pq#60469778 nonce:int128 = ResPQ`. Fixed-U `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a` confirms `AbstractConnection::preparePQFake` constructs `MTPReq_pq(nonce)` before not-secure packet/message-id wrapping; reread `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no `req_pq` branch. The existing shared `telegram-protocol` owner can independently carry the exact request body; probe scheduling/racing, nonce generation, not-secure framing, response correlation and live auth-key lifecycle remain fail-closed.

Platform-core spec-first commit `c0b1b72f501822023e73f0e971a41bb0bfe38baf` and implementation `48b8dcb9fc7d6f7a79c1ccdce59041493a7a3b7a` add only `REQ_PQ_CONSTRUCTOR=0x60469778` plus exact `build_req_pq(nonce:&[u8;16])`. Canonical blobs are api `54ab072495a4f8f1eca322a7f8f46c5dcd4396ee`, lib `8df4760c6bf6dbeeba3e8d8b72232bac6a410ff0`; desktop synchronization `9e3065eddd2d0497dfb4ea2ed7a38d4ee7acb13f` copies them byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-REQ-PQ-REQUEST-WIRE-001`; `responsibilities_total=81`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 20-byte constructor-plus-raw-int128-nonce contracts, including zero/0xff nonce fixtures, are authored only. `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `req_pq_multi`, live preparePQFake scheduling/transport racing, not-secure packet/message-id construction, send/retry/timeout, response correlation, existing ResPQ/auth-key handshake integration, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.

### Round 71 source-level compliance readback

Platform-core compliance commit `893b83679268c9a2e3bd8d867419adb4b5145d26` updates `docs/specs/telegram-mtproto-req-pq-request-wire.md@b90b86e526f24665832326cc1b3ff7e95d0d32c2`. Remote readback confirms canonical and desktop provider api/lib blobs are byte-identical at `54ab072495a4f8f1eca322a7f8f46c5dcd4396ee` / `8df4760c6bf6dbeeba3e8d8b72232bac6a410ff0`; `REQ_PQ_CONSTRUCTOR=0x60469778`, public `build_req_pq(nonce:&[u8;16])`, raw int128 serialization and both authored contracts are present. R1-R5 are satisfied at source/ownership/bookkeeping level; R6 executable validation remains blocked/not-run with `accepted=false`. Live connection-probe scheduling/transport racing, nonce generation, not-secure packet/message-id construction, send/retry/timeout, response correlation, `req_pq_multi`, live ResPQ/auth-key handshake integration, arbitrary sent/resend/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open.



### Round 72 — req_pq_multi request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `req_pq_multi#be7e8ef1 nonce:int128 = ResPQ`. Fixed-U `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` confirms shipping `DcKeyCreator::pqSend` sets `Stage::WaitingPQ`, generates the attempt nonce, and passes `MTPReq_pq_multi(attempt->data.nonce)` to `sendNotSecureRequest`; `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no adjacent `req_pq_multi` branch.

Platform-core spec-first commit `c5b3021a760892eaf05fcd9b40dc35dff4c2c783` and canonical implementation commits `60e4b9165c0f41804bf68e7270aa52f39e824f14` / `591176a5ba091bb2a00f925d38ed87a1fdae0b11` add public `REQ_PQ_MULTI_CONSTRUCTOR=0xbe7e8ef1` plus pure `build_req_pq_multi(nonce:&[u8;16])` in the existing request-wire owner. Existing `AuthKeyHandshake::begin` delegates only body serialization to that builder while retaining its invalid-state guard, caller message id, `AwaitingResPq` transition, and plaintext-envelope lifecycle. Canonical blobs are api `3e3b250b709ce2f307f896dda6d1c2da55559f7b`, handshake `b1b22af0e83b577d6152beb11a75e44eb76ffdbe`, lib `2d48c2fe1c677b4b4672e7e76d8bd0a29a7ac003`, contracts `bf5aeb85f10baeeb7e645a660fd3bf26259e5989`. Desktop synchronization `59e08389bae4ab2cdcbb3f67808da44dff452ca9` makes its tracked api/lib/contracts byte-identical without introducing a second handshake lifecycle owner.

Ledger adds `TDRP-R9-MTPROTO-REQ-PQ-MULTI-REQUEST-WIRE-001`; `responsibilities_total=82`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 20-byte constructor-plus-raw-int128-nonce contracts, including patterned/zero/0xff nonce fixtures and begin-path body equality, are authored only. `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Connection-probe scheduling/transport racing, nonce/message-id generation, plaintext/not-secure framing, send/retry/timeout, ResPQ correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.


### Round 72 source-level compliance readback

Platform-core compliance commit `492c822360c7dacedef656226386126a41ba5d91` updates `docs/specs/telegram-mtproto-req-pq-multi-request-wire.md@e9cddb463c32ef9c4483a1a7bd57c130fe260f19`. Remote readback confirms public `REQ_PQ_MULTI_CONSTRUCTOR=0xbe7e8ef1`, pure `build_req_pq_multi(nonce:&[u8;16])`, and the existing `AuthKeyHandshake::begin` body-only delegation are present; canonical api/handshake/lib/contracts blobs are `3e3b250b709ce2f307f896dda6d1c2da55559f7b` / `b1b22af0e83b577d6152beb11a75e44eb76ffdbe` / `2d48c2fe1c677b4b4672e7e76d8bd0a29a7ac003` / `bf5aeb85f10baeeb7e645a660fd3bf26259e5989`. Desktop tracked api/lib/contracts remain byte-identical and no second desktop handshake lifecycle owner was introduced.

Spec R1-R7 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-REQ-PQ-MULTI-REQUEST-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; Stage-B executable evidence is intentionally absent. Connection probing/racing, nonce/message-id generation, plaintext/not-secure framing, network send/retry/timeout, ResPQ correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 73 — set_client_DH_params exact request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `req_DH_params#d712e4be ...` and `set_client_DH_params#f5045f1f nonce:int128 server_nonce:int128 encrypted_data:string = Set_client_DH_params_answer`. Fixed-U `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` confirms shipping `pqAnswered` sends `MTPReq_DH_params` and `dhClientParamsSend` sends `MTPSet_client_DH_params` after moving to `Stage::WaitingDone`; `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` contains no direct constructor call.

Source review confirms existing canonical `build_req_dh_params` already serializes constructor, nonce, server_nonce, p, q, fingerprint and encrypted_data in fixed-U order, with an authored contract that consumes every field and requires no trailing bytes; that responsibility was left unchanged. `set_client_DH_params` was still inline in `prepare_client_dh`, so platform-core spec-first commit `f3d85c368ace17c864eebf2b4d97a44c9d3d964f` and implementation `a353cdf718a93f8e51f48e58c1be4a9889c3bc32` add pure `build_set_client_dh_params` and delegate only final request-body framing from the existing DH preparation path. Canonical blobs are handshake `2e9acc80d10a87a8633173c88568aef706d4e123`, lib `3a05251d749f0df868e4ebd94fb7e28e31fad8f1`, contracts `3cef7f7532cf98c46f9f9eb06f9b0ff2026dfcb4`. Desktop synchronization `85db8d62f8014614234ab2340560367d087ac4e0` mirrors lib/contracts byte-for-byte without introducing a second handshake lifecycle owner.

Ledger adds `TDRP-R9-MTPROTO-SET-CLIENT-DH-PARAMS-REQUEST-WIRE-001`; `responsibilities_total=83`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact constructor/raw-nonce/TL-string-padding and prepare-path body-equality contracts are authored only. `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Connection probing/racing, message-id generation, plaintext/not-secure framing, send/retry/timeout, server response correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.


### Round 73 source-level compliance readback

Platform-core compliance commit `dd584145d268084545e082c399e9f5fe2e52cba9` updates `docs/specs/telegram-mtproto-set-client-dh-params-request-wire.md@7ec57e2cfa47f62f0ae372e776ec8a1dcb39c04d`. Remote readback confirms canonical `build_set_client_dh_params`, its public facade export, and `prepare_client_dh` body-only delegation are present; canonical handshake/lib/contracts blobs are `2e9acc80d10a87a8633173c88568aef706d4e123` / `3a05251d749f0df868e4ebd94fb7e28e31fad8f1` / `3cef7f7532cf98c46f9f9eb06f9b0ff2026dfcb4`. Desktop tracked lib/contracts are byte-identical. Existing `build_req_dh_params` and its complete field-order/trailing contract remain present and unchanged.

Spec R1-R8 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-SET-CLIENT-DH-PARAMS-REQUEST-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; `responsibilities_total=83`, `unknown=16172`, `unread=16145`, `omitted=0`. Stage-B executable evidence is intentionally absent.

Connection probing/racing, message-id generation, plaintext/not-secure framing, network send/retry/timeout, server response correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 74 — client_DH_inner_data exact wire serializer; temp_dc remains fail-closed

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `client_DH_inner_data#6643b654 nonce:int128 server_nonce:int128 retry_id:long g_b:string = Client_DH_Inner_Data` and adjacent `p_q_inner_data_temp_dc#56fddf88 ... dc:int expires_in:int`. Fixed-U `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` constructs `MTP_client_DH_inner_data` inside shipping `dhClientParamsSend`; its temporary-key branch chooses `MTP_p_q_inner_data_temp_dc` only when `attempt->expiresIn != 0`. `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct construction of either inner object.

Existing canonical `prepare_client_dh` already owned the full DH/auth-key preparation lifecycle but framed `client_DH_inner_data` inline. Platform-core spec-first commit `20049ba38728a50239a900a1c0a684527d427b30` and implementation `b5e5f9cd85ed663b0c2ab35998bc1fbf116df0e9` add pure `build_client_dh_inner_data`, preserving exact constructor/raw nonce/server_nonce/little-endian retry_id/canonical TL bytes framing, and delegate only that inner-object serialization from `prepare_client_dh`. Canonical blobs are handshake `0e5e6c0fa306f75aa1fcb5c26440b1c39f87c9f9`, lib `7273e030bf02c248d9b592b2578cb5dc9716b485`, contracts `7236df6bec3c832363901034d6a7d1c7cd3df3fc`. Desktop synchronization commit `1de1da4bf045c405d26ff44c373ad2dd63bf9ce1` mirrors lib/contracts byte-for-byte and introduces no second handshake owner.

`p_q_inner_data_temp_dc` is deliberately not marked implemented: the canonical `AuthKeyHandshake::prepare_req_dh_params*` currently accepts `dc_id` but has no temporary-key `expires_in` input/state. Adding a dead standalone serializer would not close the shipping temporary-key lifecycle. Ledger adds `TDRP-R9-MTPROTO-CLIENT-DH-INNER-DATA-WIRE-001`; `responsibilities_total=84`, `unknown=16172`, `unread=16145`, `omitted=0`. New responsibility is `implemented / validation_status=not-run / accepted=false`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Connection probing/racing, message-id generation, plaintext/not-secure framing, send/retry/timeout, server response correlation/application, temporary-key p_q_inner_data_temp_dc lifecycle, broader auth-key persistence/session lifecycle, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.


## Round 74 source-level compliance readback

Platform-core compliance commit `e1b2e5fc59e46b198c07c64845810a73a62b04b4` updates `docs/specs/telegram-mtproto-client-dh-inner-data-wire.md@dc09433a720daa1af5fa7eca4dc641cc542e52cc`. Remote readback confirms canonical `build_client_dh_inner_data`, its public facade export, and `prepare_client_dh` inner-object-only delegation are present; canonical handshake/lib/contracts blobs are `0e5e6c0fa306f75aa1fcb5c26440b1c39f87c9f9` / `7273e030bf02c248d9b592b2578cb5dc9716b485` / `7236df6bec3c832363901034d6a7d1c7cd3df3fc`. Desktop tracked lib/contracts are byte-identical.

Spec R1-R8 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-CLIENT-DH-INNER-DATA-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; `responsibilities_total=84`, `unknown=16172`, `unread=16145`, `omitted=0`. Adjacent `p_q_inner_data_temp_dc#56fddf88` remains explicitly fail-closed because current canonical handshake lifecycle does not represent temporary-key `expires_in`; no dead serializer is counted as production closure. Stage-B executable evidence is intentionally absent.

Temporary-key p_q_inner_data_temp_dc lifecycle, connection probing/racing, message-id generation, plaintext/not-secure framing, network send/retry/timeout, response correlation/application, broader auth-key persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 75 — p_q_inner_data_temp_dc exact wire + temporary request preparation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. fixed-U `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `p_q_inner_data_temp_dc#56fddf88 ... dc:int expires_in:int`; `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` records `Attempt::expiresIn` and consumes it only when selecting/serializing the temporary p_q inner object before Req_DH_params. Existing canonical `AuthKeyHandshake` can therefore precisely own the wire/request-preparation slice without a second state machine.

Spec-first commits `93283c35d075a16498b672e548071e16bdafd2c3` and boundary correction `8f09112d7af1875e19eddb7096ef1a0a05347d8d` precede implementation. Canonical commits `af7b37f57be70a88cd10728d79d352c7aade88f5` / `68c20daadfc680e24393c63d0505feb5fee4783e` add `P_Q_INNER_DATA_TEMP_DC_CONSTRUCTOR`, pure `build_p_q_inner_data_temp_dc`, positive lifetime validation, and `prepare_temporary_req_dh_params*` reusing the existing ResPqReceived -> AwaitingServerDh preparation path. Contract commit `bcd516892fa49407a4de757e05235ed585ad4338` is authored only. Desktop commits `aafca841fd745f3aee2deb9b89921f6d34021361` / `bd7c3a1c648107c92a265c76c46bae1da745f7e9` mirror facade/contracts byte-for-byte.

The main telegram-network/runtime establishment remains persistent: current persisted transport state has no temporary-key expiry/binding metadata, so routing a temporary key there would misrepresent lifetime semantics. That broader lifecycle is explicitly fail-closed rather than silently persisted. Ledger adds `TDRP-R9-MTPROTO-P-Q-INNER-DATA-TEMP-DC-WIRE-001`; `responsibilities_total=85`, `unknown=16172`, `unread=16145`, `omitted=0`; new responsibility is `implemented / validation_status=not-run / accepted=false`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. Broader temporary auth-key expiry/binding/persistence, connection scheduling/racing, plaintext/send lifecycle, multi-request history, gzip, complete difference decoding, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.

### Round 76 — DestroyAuthKeyRes exact typed parser + adjacent p_q disposition

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Source readback confirms `DcKeyCreator` requires `protocolDcId` and its shipping pq-answer path selects only `p_q_inner_data_dc#a9f55f95` (persistent) or `p_q_inner_data_temp_dc#56fddf88` (temporary). The adjacent no-dc `p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` therefore have no independent fixed-U shipping responsibility in this path and no duplicate builders were added. `bind_auth_key_inner#75a3f765` remains fail-closed because temporary/permanent binding, expiry and session lifecycle require broader state ownership.

The next independent shipping auth-key wire responsibility is `DestroyAuthKeyRes`: fixed-U `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087` sends `destroy_auth_key#d1435160`, parses the response union, and distinguishes `destroy_auth_key_ok#f660e1d4`, `destroy_auth_key_none#0a9f2259`, and `destroy_auth_key_fail#ea109b13`. Spec-first platform commit `791308b5c885b15b4f69b395ae0df0a75f47f634` defines the wire-only boundary. Canonical implementation commit `ebb1e5f859e224c45f9d34beb1aa32b4c18ca4b2` adds the three constructors, `DestroyAuthKeyResult`, strict `parse_destroy_auth_key_result`, facade exports and authored-not-run contracts. Canonical api/lib/contracts blobs are `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `a53c90f09ec18f8e99d8cfb36eb8c8842f27635e`; desktop sync commit `4b152da131b17558fcad7c1aee45fd0cec3269e6` mirrors those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-RESULT-WIRE-001`; `responsibilities_total=86`, `unknown=16172`, `unread=16145`, `omitted=0`. The new responsibility is `implemented / validation_status=not-run / accepted=false`. Destruction eligibility, local key removal, persistence mutation, transport/session teardown/reconnect, temporary/permanent binding and renewal remain fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.

## Round 76 final source-level readback

Platform-core compliance commit `42c7ac4d193f7f1872a9bfd87ce70b2af973c2df` updates `docs/specs/telegram-mtproto-destroy-auth-key-result-wire.md` to blob `da653d871e78a8a97770449034d0ac4db2f71cfe` and records R1-R8 source-level compliance. Canonical api/lib/contracts remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `a53c90f09ec18f8e99d8cfb36eb8c8842f27635e`; desktop tracked provider files are byte-identical.

The ledger responsibility `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-RESULT-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=86`, `unknown=16172`, `unread=16145`, `omitted=0`. Adjacent `p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` remain dispositioned as non-shipping in fixed-U `DcKeyCreator`; `bind_auth_key_inner#75a3f765` and broader auth-key destruction/binding/persistence/teardown lifecycle remain fail-closed.

No executable validation was performed. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.



### Round 77 — Server_DH_Params exact constructor-first union parsing

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping `DcKeyCreator::dhParamsAnswered` consumes fixed-U `server_DH_params_ok#d0e8075c` / `server_DH_params_fail#79cb045d` through the existing canonical `AuthKeyHandshake::receive_server_dh`. The existing parser already owned the wire responsibility, but read nonce/server_nonce before constructor discrimination, so a complete four-byte unknown constructor was misclassified as TL truncation.

Spec-first platform commit `9a6b687e043bae9f7f40e776ce1d5ec98220a168` defines the strict wire-only boundary. Canonical implementation commit `274bdb92bc7e4d5755bb87fd4996398ee6cf768b` hardens `parse_server_dh_params` to reject unknown constructors before field reads while preserving exact nonce binding, OK TL-bytes / FAIL int128 payload decoding and no-trailing validation. Canonical handshake/contracts blobs are `ed9eda7732f4753b442de98ebf510b8a7558b823` / `a19c4a14d113e6c82fe3cc801f1764c555f26f76`. Desktop sync commit `000d38bd2db19052766ccf8062eea4a6fb93c7eb` mirrors the contracts byte-for-byte; the tracked desktop facade remains `abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate desktop `handshake.rs` is introduced.

Ledger adds `TDRP-R9-MTPROTO-SERVER-DH-PARAMS-WIRE-001`; coverage becomes `responsibilities_total=87`, `unknown=16172`, `unread=16145`, `omitted=0`; the new responsibility is `implemented / validation_status=not-run / accepted=false`. Broader auth-key destruction/binding/persistence/teardown, connection scheduling/racing/send lifecycle, multi-request history, gzip, complete difference decoding/application, broader source/legal closure and release acceptance remain fail-closed/open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 77 final source-level compliance

Platform-core `8654e737a04900b6cac8c92660fa8a8b84b449bc` / spec blob `b9b0f486aa47071ad46ba4ff5c238cee4b2bbfce` records source-level R1-R9 and AC-1-AC-6 compliance for the constructor-first fixed-U `Server_DH_Params` parser. Canonical handshake/contracts are `ed9eda7732f4753b442de98ebf510b8a7558b823` / `a19c4a14d113e6c82fe3cc801f1764c555f26f76`; desktop contracts remain byte-identical and facade `abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged. Ledger stays `implemented / not-run / accepted=false`, coverage `87 / 16172 unknown / 16145 unread / 0 omitted`. No executable validation was performed.


### Round 78 — dh_gen_ok/retry/fail exact result/hash validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`DcKeyCreator::dhClientParamsAnswered` consumes fixed-U `dh_gen_ok#3bcbf734`,
`dh_gen_retry#46dc1fb9` and `dh_gen_fail#a69dae02`. Existing
`AuthKeyHandshake::receive_dh_gen / parse_dh_gen_result` already owned nonce binding,
tag-specific `new_nonce_hash{1,2,3}` authentication, strict trailing validation, OK completion,
handshake-local RETRY and FAIL rejection. The remaining exact-wire gap was discriminator ordering:
nonce/server_nonce/hash were read before constructor matching, so a complete four-byte unknown constructor
was misclassified as TL truncation.

Spec-first platform commit `0c16a7c83a7e511484f5099399acb4c308aecb6c`
(`docs/specs/telegram-mtproto-dh-gen-result-wire.md@a43f4db8123abfcd2334fe49079b413e40c58ea6`)
defines the wire-only boundary. Canonical implementation commit
`5a5c6095d3e049de5d187b89c4942d7f3377ae00` moves three-way constructor discrimination before
all common fields in `handshake.rs@87616f4b96a1775455da42b0bbed77711fb3c9f6`.
Canonical authored-contract commit `39fce63bce58718727541ac9f18dbc258766d159` expands the existing harness to all
OK/RETRY/FAIL variants, every truncation cut, four-byte unknown constructor, nonce/server_nonce mismatch,
wrong variant-tag hash and trailing bytes; contract blob is
`6710d6ef3fd915689792c68d69b3922b446c9bfa`. Desktop sync
`965db14afa7104deae0fc63a76d2cc0df35d81fd` mirrors that blob byte-for-byte; the tracked desktop
facade remains `abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate `handshake.rs` is introduced.

The existing RETRY branch is precisely representable by the current `AwaitingDhGen` owner: it uses the
previous auth-key auxiliary hash as `retry_id`, replaces only prepared client-DH material, and remains in
`AwaitingDhGen`. It does not require or claim temporary/permanent key binding, expiry, persistence,
teardown, connection scheduling, multi-request history, gzip, or complete difference decoding/application.

Ledger adds `TDRP-R9-MTPROTO-DH-GEN-RESULT-WIRE-001`; coverage becomes
`responsibilities_total=88`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`.

No executable validation was performed. No GitHub Actions/status/check query or trigger, build, typecheck,
lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or
runtime acceptance was performed.


### Round 78 final source-level compliance

Platform-core `3a08f54f8a49c8a8af7a23b252938d1ea4607954` / spec blob
`d846673483e2b165d7fad7463be0d7720ee6a49b` records source-level R1-R9 and AC-1-AC-6 compliance
for the constructor-first fixed-U `dh_gen_ok/retry/fail` parser/hash responsibility.
Canonical handshake/contracts are `87616f4b96a1775455da42b0bbed77711fb3c9f6` /
`6710d6ef3fd915689792c68d69b3922b446c9bfa`; desktop contracts remain byte-identical and facade
`abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged. The existing RETRY path stays inside
`AwaitingDhGen` and does not claim binding/persistence or connection scheduling. Ledger remains
`implemented / not-run / accepted=false`, coverage `88 / 16172 unknown / 16145 unread / 0 omitted`.
No executable validation was performed.


### Round 79 — resPQ exact constructor/nonce/server_nonce/pq/fingerprint-vector validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`DcKeyCreator::pqAnswered` consumes `resPQ#05162463 nonce:int128 server_nonce:int128 pq:string server_public_key_fingerprints:Vector<long>`.
Remote source review confirms the existing canonical `AuthKeyHandshake::receive_res_pq / parse_res_pq` already owns and exactly implements
this fixed-U wire responsibility: constructor-first discrimination, exact nonce/server_nonce/pq/vector field order, bounded TL Vector
constructor/count validation, strict EOF, active request nonce binding, empty-pq rejection and empty-fingerprint rejection before state mutation.
The server nonce is fresh server-provided handshake material at this phase and is captured only after the response passes validation.

Because production behavior was already exact, no production `handshake.rs` delta was manufactured. Spec-first platform commit
`03dc0f804a8c80d8d60500e6667cc9b091ccc3cf`
(`docs/specs/telegram-mtproto-res-pq-wire.md@ab044dcb2b34d7def042e4dd2eb798f3b658c52a`) records that disposition.
Canonical authored-contract commit `5e1ba90b32ac7c58b7be0d9630f1630a0a764061` extends the existing protocol harness with exact
positive resPQ, every truncation cut, four-byte unknown constructor, wrong Vector constructor, negative/over-limit/short declared vector,
trailing byte, active nonce mismatch, empty pq, empty fingerprints and invalid receiver-state cases; canonical contract blob is
`44d2a5d141e7d7854529379aaaad660853c8cbe3`. Desktop provider sync commit
`c72c7f85c926d05b21842c41a3b11c1313537793` mirrors that contract blob byte-for-byte; the tracked desktop facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate handshake core is introduced.

Ledger adds `TDRP-R9-MTPROTO-RES-PQ-WIRE-001`; coverage becomes
`responsibilities_total=89`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`. Temporary/permanent key binding/expiry/persistence/teardown,
connection scheduling/racing/send lifecycle, arbitrary multi-request history, gzip, complete difference decoding/application,
broader source/legal closure and release acceptance remain fail-closed/open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 79 final source-level compliance

Platform-core `54f1a26231827ac11a1ea0544dc4d1f7c3762c6c` / spec blob
`404bbe591fb514aba732e5b4e8f6481cc2904618` records source-level R1-R9 and AC-1-AC-5 compliance for the
fixed-U `resPQ#05162463` wire responsibility. Canonical production handshake remains
`87616f4b96a1775455da42b0bbed77711fb3c9f6` with no behavior delta; canonical/desktop authored contract blob is
`44d2a5d141e7d7854529379aaaad660853c8cbe3`, and desktop facade
`abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged. Ledger remains
`implemented / validation_status=not-run / accepted=false`, coverage
`89 / 16172 unknown / 16145 unread / 0 omitted`. No executable validation was performed.


### Round 80 — server_DH_inner_data exact authenticated wire validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`DcKeyCreator::dhParamsAnswered` consumes fixed-U
`server_DH_inner_data#b5890dba nonce:int128 server_nonce:int128 g:int dh_prime:string g_a:string server_time:int`.
Remote source review confirms the existing canonical
`AuthKeyHandshake::receive_server_dh -> decrypt_server_dh_inner_data` already owns and exactly implements this
wire/decryption responsibility: non-empty AES-IGE block framing, temporary key/IV derivation from active
new_nonce/server_nonce, constructor-first exact field decode, active nonce/server_nonce binding, SHA-1 authentication
over exactly the consumed TL object, residual AES alignment padding bounded to 0..15 bytes, signed server_time
preservation, and existing safe-prime/generator/public-value checks before client-DH preparation.

Because production behavior is already exact, no `handshake.rs` behavior delta was manufactured. Spec-first
platform commit `3bdde5a158373e3392cac987c8846d16a6fb3162`
(`docs/specs/telegram-mtproto-server-dh-inner-data-wire.md@5174248bd16e0d125abca66b389d0e49bc2a3f05`)
records the no-production-delta disposition. Canonical authored-contract commit
`d724e29265e53a7ee7efd821c8bb37f6e4994cb2` expands the existing harness with exact signed server_time,
empty/non-block ciphertext, unknown constructor, every serialized-inner truncation cut, nonce/server_nonce mismatch,
SHA-1 mismatch, >15 residual padding, invalid generator, dh_prime length/trust, empty/oversized/unsafe g_a and
rejected-inner state-continuity cases; canonical contract blob is
`256f421f54c41256646388833b634cf1191afd91`. Desktop sync
`4323b97d3b6041731d5b547118fb746d2eba37a6` mirrors that blob byte-for-byte; desktop facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate handshake core is introduced.

Ledger adds `TDRP-R9-MTPROTO-SERVER-DH-INNER-DATA-WIRE-001`; coverage becomes
`responsibilities_total=90`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`. Auth-key binding/expiry/persistence/teardown,
connection scheduling/racing/send lifecycle, arbitrary multi-request history, gzip, complete difference
decoding/application, broader source/legal closure and release acceptance remain fail-closed/open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 80 final source-level compliance

Platform-core `26cc521c1e57944a9f27fa16f469fff870f34517` / spec blob
`1290cb76c59c01343ac7903a83113b28a90b4147` records source-level R1-R13 and AC-1-AC-6 compliance for the
fixed-U `server_DH_inner_data#b5890dba` responsibility. Canonical production handshake remains
`87616f4b96a1775455da42b0bbed77711fb3c9f6` with no behavior delta; canonical/desktop authored contract blob is
`256f421f54c41256646388833b634cf1191afd91`, and desktop facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf`. Ledger remains
`implemented / validation_status=not-run / accepted=false`, coverage
`90 / 16172 unknown / 16145 unread / 0 omitted`. No executable validation was performed.
