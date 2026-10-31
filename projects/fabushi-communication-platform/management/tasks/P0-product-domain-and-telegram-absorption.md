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

