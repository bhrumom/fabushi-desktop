# Telegram 源码 → Fabushi 全量等价重写 — Source of Truth

Status: active  
Project ID: TDRP-001  
Revision: 10
Date: 2026-10-31
Parent: FBCP-001 Revision 8
Execution addendum: FMEP-001 Revision 1 (2026-10-31)

## Role

本项目负责完整 `telegramdesktop/tdesktop` 源码及其递归依赖、资源、工具链的逐文件、逐模块理解、责任对照、等价重写、生产接入和 exact-head 取证。它不是 capability research 子项目，也不能以 survey、ledger 填写、局部 vertical slice 或 UI Demo 作为最终完成。

上游是完整产品行为、状态机、边界条件与工程责任的 source authority，不是最终 runtime/network/product。全部 applicable 产品职责必须吸收入 current canonical Fabushi；C++ 非 UI 产品逻辑默认 Rust，UI 使用 Fabushi React/TypeScript 表现层，平台职责采用最薄 adapter。UI 文件中的业务、权限、生命周期、错误处理、键盘与可访问性行为仍然必须迁移。

## Must read

- root `AGENTS.md`
- `docs/specs/fabushi-deferred-ci-migration.md` — FMEP-001 Revision 1；执行时机、批次协作、保存与集中验收的权威补充
- `projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md` Revision 8 with execution addendum
- `docs/specs/fabushi-bot-communication-platform.md` Revision 8
- `docs/specs/telegram-desktop-rust-equivalence-migration.md` Revision 10
- `module-map.md`
- current upstream lock / inventory / ledger / dossiers / validators / quality plans
- current canonical Fabushi shipping owners

较早 Revision 2/4/5 中 research-only、选择性吸收、where accepted、blocked 可算最终验收、htch-runtime 可执行本项目验证等口径全部废止。历史记录只能保留 provenance，不能继承为 implemented/verified。

## Execution timing

阶段 A 全量实现与融合期间不进行任何迁移 CI，也不跑本机验证。只做源码理解、责任映射、生产实现/接线、测试/oracle/fixture/workflow 编写、人工审查和安全保存。按目标 owner 成批工作、共享接口先定型、可分离路径并行、公共文件由唯一集成人合入，避免一文件一轮 CI 或反复重读同一内容。

全范围实现与接线完成后冻结 H，阶段 B 才统一执行 GitHub Actions 验证/修复；最终独立接受属于阶段 C。“统一”不限制失败修复次数，禁止无理由重复执行同一失败手段。任何阶段都不能将 `not-run` 伪装成 pass。

FMEP-001 优先解释旧任务/P0/ledger 对测试时机的表述：`implemented` 与 `verified` 分离；未运行测试不是继续实现下一个模块的障碍。正式基线校验、编译产物、部署和包体验证延期至阶段 B，不作为阶段 A 开工前置。

## Baseline

本文件曾记录 2026-10-19 candidate `1e18da644e37856c9b1d87434bfcd6a2e3d11c57`（tree `30c1d5cce4c1bde962882c1f2ca4b2e51bb03ae6`），前一 `0a11537c220df35fe59a96aebe413b1e52bc6e47`。这些是历史快照，不是当前 U 的覆盖性声明；与较新的主 Spec/SOT 冲突时不得直接沿用。开工以同一 live revision 下的 lock、inventory、ledger、dossiers 核对实际 U，本次政策更新不修改这些机器身份。

阶段 A 用只读 Git/API 固定 U，递归读取并理解：
- root tracked files / gitlinks；
- nested submodules；
- build-time downloads / patches；
- LFS / external objects（如有）；
- generated-input contracts；
- functional resources / locales / assets；
- build / packaging / updater / tests / tooling / docs / licenses。

不因上游浮动 HEAD 自动重开已经固定的迁移范围。阶段 B 再由 GitHub Actions 对冻结 U 的完整性与身份作正式验证。只有 lock/inventory/ledger/dossiers 同一 exact U、一致且 unknown=0、unread=0、omitted=0 并满足正式门槛，才能将 baseline_ready 设 true。旧 verified 状态不得继承。

## Mandatory chain

阶段 A：`source file -> symbol -> responsibility -> capability -> existing Fabushi owner -> target path/symbol/language -> shipping wiring -> tests authored -> implemented / validation not-run`。

阶段 B/C：`full implementation closure -> freeze H -> GitHub Actions -> evidence -> independent review -> verified / ACCEPT`。

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

## Verification and preservation

所有 FBCP/TDRP 可执行 build/test/lint/generator/schema/benchmark/fuzz/E2E/visual/a11y/performance/soak/fault/package/acceptance 只允许阶段 B 的 GitHub Actions。

代码检查点按 FMEP-001 §5 安全保存在云端：未证明不会触发 CI 时，不更新实现 ref 或 PR；确认已隔离后可以沿既有分支保存。只靠 skip commit 文本或 draft PR 不足以证明所有触发已排除；不得在未获准时全库关闭 workflow 或移除最终保护。不把重要成果长期仅留用户本机，未安全备份不清理。

每个 responsibility 最终 Definition of Done 仍要求：
- production implementation 已进入唯一 shipping composition；
- requirement/oracle/invariant/test 可追溯；
- 最终冻结候选的 GitHub Actions evidence；
- 无 flaky/skipped/blocked/not-run 冒充 pass；
- packaged temporal acceptance 验证 terminal result 在 quiet window、切换、reconnect、reload、restart 后不消失、不回滚、不被 intermediate 替代。

失败先分类，修复后冻结新 H，集中重验受影响项；同 H 有效通过不无意义重跑，最终 Hfinal 仍须全部适用 required gates。release candidate 只有在 independent acceptance 为 ACCEPT、0 open P0/P1/blocker、0 omitted/unmapped、0 stub/fake fallback 后才可发布。不得用 continue-on-error、跳过步骤或软化 assertion 绕过。

## Current status

本次为执行规范与提示词更新，不代表迁移、基线或验收完成。历史 lock / inventory / ledger / dossiers / STATUS / validators 仍须按本轮实际来源和职责逐项核对。

继续沿用 `unreviewed -> understood -> mapped -> implemented -> verified`：阶段 A 到 implemented/未验证，阶段 B/C 才可 verified/ACCEPT。遇到 service/account/signing blocker，记录解除条件并继续无依赖任务。

完整提示词：`projects/fabushi-communication-platform/management/prompts/full-migration-deferred-ci.md`。
