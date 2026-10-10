# 全量源文件/模块 → Fabushi Owner 对照规则

Status: active  
Project: TDRP-001 Revision 9  
Parent: FBCP-001 Revision 7  
Updated: 2026-10-07

## Purpose

本文件定义完整迁移对照要求，不是已完成的全量对照表。旧的“只映射能力、不做 source-equivalent target mapping”口径废止。

必须同时提供逐源文件映射与逐模块行为合同。等价目标是功能/责任/状态机/产品效果，不是同名文件/类/目录。多文件可合并到一个 owner，一个文件可拆到多个 owner，但每个 source responsibility 都必须能反查其实现与证据。

## Required record

每个源文件记录 exact repository/commit/path/blob、分类、module/responsibility IDs、完整理解记录、UI 与非 UI 责任划分、capabilities/dependencies、existing owner 候选与选择、Fabushi target paths/symbols、语言与理由、状态/lifecycle/persistence/security/concurrency、服务端依赖、生产入口、测试/证据、target SHA、blockers 与 licenses/provenance。

对所有用户可见责任还必须记录：`ui_entry_class`、canonical route/root/slot、collection/filter、creation flow、object action/detail section、capability surface、search scope/provider/result/filter/permission、keyboard/accessibility、responsive/state-continuity。Group/Channel/Topic 等不能因上游页面独立而映射成第二应用；Search consumer 不得自建 search truth。

还必须记录 `design_system_version`、semantic tokens、canonical component IDs、screen pattern/slot、visual state matrix、icon/avatar/copy/motion contract、visual baseline/evidence 与任何 design exception ADR。能用现有 canonical component 表达却自建 feature-local primitive/pattern 的映射不得通过。

测试字段同样是 required record：`quality_risk`、`requirement_ids`、`oracle_ids`、`invariant_ids`、unit/property-state/contract/integration/functional-E2E/temporal/fault/UI/visual/a11y/performance/soak/security case IDs、regression IDs、exploratory charter、fixture/environment、flake status、execution evidence、acceptance reviewer 与 release-gate status。没有 oracle/traceability 的 mapped/implemented 不能升级 verified。

字段、状态和完整 gate 以 TDRP-001 §2/§8 为准。旧目录级表仅能作为导航，不能证明逐文件覆盖。proposed path 不等于已存在 production code。

## Owner-first hypotheses

以下是研究/映射起点，必须读当前源码替换为 exact-head 路径/符号；不是完整能力清单，也不是实现完成声明。

| 源职责区域 | 必须承接的责任示例 | 优先检查的 Fabushi owner |
| --- | --- | --- |
| dialogs/history/data | 列表、顺序、历史、gap、未读、编辑删除、relations、草稿 | navigation/list、conversation、transcript、composer |
| contacts/info/profile/groups | 身份、成员、角色、群频道话题、privacy | identity、Shared Room/member、permissions |
| api/mtproto/session | 消息确认、重试、幂等、同步、重连、协议状态 | durable state、既有 command/event owner、最小 native network infra |
| storage/media/export | 持久化、迁移、传输、缓存、播放、编辑、导出 | attachments/artifacts/resources、storage/data lifecycle |
| calls/tgcalls/lib_webrtc | 会话、信令、设备、流、共享、取消/恢复 | Computer/媒体/平台 owner；缺失 call-session 以最小 ADR 补齐 |
| bots/inline/webview | 命令/callback、交互、安全会话、Mini Apps | typed Agent interactions、Composer、Plugins/MCP/Web surface |
| business/payments/credits/gifts | 自动化、权益、价值流、结算、审计 | Automations/Task/settings，缺失服务以最小 ADR 补齐 |
| settings/platform/core | 本地锁、通知、窗口/后台、凭据、更新 | account/secrets/settings/platform/lifecycle |
| ui/boxes/window/history views | 表现层替代及混合文件中的全部非 UI 业务 | React/TS UI；非 UI 责任回到相应 Rust/domain owner |
| build/generator/resources/dependencies | 工具链、schema、资源、许可证、可重现发布 | Fabushi 自有 CI/packaging、最薄平台适配、审查过的通用依赖 |

## Decision procedure

1. 全文件读取及调用关系研究，列出所有独立 responsibility。
2. 检查 current exact-head Fabushi 合理 owner，选择并解释 existing_owner。
3. 不存在合理 owner 时，写 rejected owners、最小责任和批准 ADR；禁止整体 TelegramRuntime/CommunicationCore。
3a. 若为当前 Fabushi 完全没有的 novel capability，新增最小 source-neutral owner，而不是删功能；记录 owner-absence evidence，并通过 typed capability contract 接入统一 shell/canonical truth。
3b. 为每项 UI/product responsibility 记录唯一 canonical `composition_root` 与 typed `composition_slot`；不得按 Human/Agent/Group/Channel/来源复制完整 workspace/list/profile/composer/resource/settings root。
4. 确定 target paths/symbols 与 best-fit language；C++ 非 UI 产品逻辑默认 Rust。
5. 实现真实状态机和依赖，接入 shipping composition，再记录测试证据。
6. 每个文件/责任取得证据后单独升级状态，不批量冒进。

## UI and network distinctions

不迁 Qt 表现实现，不等于不迁 UI 文件中的业务与用户功能。微信式左竖栏/消息/联系人/插件市场和 Fabushi 品牌由 FBCP-001 规定。

不接官方 Telegram 网络，不等于跳过 mtproto/api/session/storage 责任；其对应行为必须进入 Fabushi 自有服务/协议。上游服务端缺失时补充 Fabushi server contract/implementation，并以真实 E2E 闭合。

## Completion

0 omitted/unmapped files or responsibilities；所有 non-UI 产品责任 rewritten + wired + verified；所有 UI/platform/toolchain 替代有证明；法律来源记录保留；0 open in-scope blocker。数量相等、目录级 mapping、编译绿或旧 dossier 都不是完成证据。

所有可执行验证只在 GitHub Actions。现有旧 baseline graph/ledger 先做差异审计，不得继承未验证的新基线状态。

### TooManyCooks runtime authority — orders 7,057–7,154

All 126 pinned TooManyCooks entries are exact-read. Runtime behavior maps to existing Coordinator/Host/Runner/canonical messaging/Build-Release owners; no TMC subsystem is created. Executor/priority, wait/wake/UAF, foreign callback lifetime, queue close/reclamation, CPU capacity/topology and build-quality responsibilities remain mapped-open pending focused GitHub Actions evidence.
