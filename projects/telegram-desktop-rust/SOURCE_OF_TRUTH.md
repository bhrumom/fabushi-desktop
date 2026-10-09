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

2026-10-09 live rebaseline 后 tdesktop accepted discovery HEAD 为 42f8a36d43b8c805bc821905bea4cfeb3af1d41d（tree 6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa）；3a15bf1f 以及更早 baseline 仅作历史证据。本次四提交 delta 仅修改 9 个既有路径且无 gitlink/path 集变化。当前 source closure 仍 open，baseline_ready/acceptance.accepted 仍必须为 false。

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

Current live authority (2026-10-09): `telegramdesktop/tdesktop@42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (root tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`), four commits ahead of historical `3a15bf1fe34b6950916215a11b50eadfce4bbb41`; exactly nine existing paths changed, no add/delete/gitlink change, recursive denominator remains 16,120, and source closure remains open.

Current source accounting: deterministic read-through `5300/16120`; unread `10820`; unknown `15841`; unknown-closed `279`; omitted `0`. Orders 5001-5290 are exact-blob read-complete. Media-view/menu responsibilities and MTProto-derived transport/session/auth/config/security/proxy/error/schema/reconnect/bootstrap responsibilities remain mapped-open except for explicitly cited existing partial Fabushi slices; MTProto wire/socket/DC mechanics are source-neutral platform/protocol replacements, not a second runtime and not omitted. Unknown stays unchanged until complete responsibility and exact-head verification gates close; no baseline-ready or release credit is granted.


### Read-through 5291-5300

Orders 5291-5300 are exact-blob read-complete at upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`. WebProxy transport/WebView invariants are mapped-open to existing Coordinator/Host/backend/desktop-bridge security, lifecycle, bounded-backpressure and recovery owners; no MTProto/WebProxy runtime is introduced. Overview selection/media layout responsibilities are mapped-open to canonical Message/SearchIndex/MediaCache, Conversation/shared-media collection, MediaViewer/Resource and design-system Checkbox/ResultRow owners; no Telegram/Overview UI root is introduced. Unknown remains 15,841 and omitted remains 0 until production + exact-head evidence closes each responsibility.
