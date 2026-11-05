# UFX 执行与独立验收状态 (`UFX-FLOW-01`)

Snapshot: 2026-11-06（北京时间）  
Branch: `design/unified-fabushi-20261105`  
Stage 1 (Tri-Source Deep Reading): **COMPLETED**  
Stage 2 (High-Cohesion Module Blueprints `00..15`): **COMPLETED (`VERDICT: VICTORY CONFIRMED`)**  
Stage 3 (Open PR Asset Absorption + Subagent Module Implementation + Narrow Root Wiring): **COMPLETED**  
Stage 4 (Remote GitHub Actions CI + Test/Beta Release $\rightarrow$ Formal Production Release): **IN PROGRESS (Dispatched to GitHub Actions)**

---

## 1. Stage 1 & Stage 2 完成交付物（16 份蓝图卷册 + 全量 Open PR 复用矩阵）

在 `docs/specs/fusion-blueprints/` 下完成了 **1 份总表 (`00`) + 15 份高内聚模块实施蓝图 (`01..15`)**，通过独立 `teamwork_victory_auditor` 审计（`OCRM=32/32`、`TCG=52/52`、`VolsReferenced=15/15`、`OpenPRs=12/14`、全卷 7 章节、源码五问 Q1–Q5、TypeScript + Rust 双端契约 100% 完备）：

1. `docs/specs/fusion-blueprints/00-master-capability-and-module-index.md` — 84 能力域双向映射 + Open PR 盘点与复用矩阵（§4.2 & §7）
2. `docs/specs/fusion-blueprints/01-shell-navigation-and-layout.md` — 统一应用壳层、左轨一级导航、会话流、文件夹与三档响应式布局
3. `docs/specs/fusion-blueprints/02-unified-conversation-stage-and-header.md` — 统一会话主舞台、单轨顶栏、话题栏与虚拟化消息时间线
4. `docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md` — 统一输入台、类型化卡片直发、斜杠命令、模型/安全门控与无损转发
5. `docs/specs/fusion-blueprints/04-unified-context-inspector-drawer.md` — 统一右侧上下文详情栏（7 抽屉收敛为单一 `ContextInspector`）
6. `docs/specs/fusion-blueprints/05-global-tasks-automations-and-subagents.md` — 一级全局任务中心、定时自动化与子代理树（含 `GLOBAL_ASYNC_TASKS_SCOPE_ID`）
7. `docs/specs/fusion-blueprints/06-realtime-calls-media-and-stories.md` — 实时音视频通话、群组语音房、多媒体查看器与快拍动态
8. `docs/specs/fusion-blueprints/07-channels-plugins-mcp-and-miniapps.md` — 全渠道网关、插件市场、MCP 工具与 MiniApp 容器（含 Ed25519 信任链）
9. `docs/specs/fusion-blueprints/08-devices-computer-and-backend-parity.md` — 云电脑、配对设备、原生数据库与后端契约对齐
10. `docs/specs/fusion-blueprints/09-memory-vector-search-and-knowledge-graph.md` — 长期记忆、混合向量检索与会话知识图谱
11. `docs/specs/fusion-blueprints/10-realtime-voice-talk-mode-and-audio-engine.md` — 全双工语音对话（Talk Mode）、VAD 打断与 5-bit 音频波形引擎
12. `docs/specs/fusion-blueprints/11-multimodal-generation-media-editor-and-transfers.md` — 多模态生成、媒体编辑器与断点续传引擎
13. `docs/specs/fusion-blueprints/12-e2e-encryption-passkeys-and-security-vault.md` — 端到端加密（Secret Chat）、Passkeys 与云端安全金库（WebCrypto PBKDF2/SHA-256）
14. `docs/specs/fusion-blueprints/13-checkout-stars-ledger-and-economy.md` — 原生收银台、Stars 账本与创作者经济（含真实余额门控与幂等键）
15. `docs/specs/fusion-blueprints/14-storage-wal-diagnostics-and-migration.md` — SQLite WAL 存储引擎、可观测性诊断与跨平台迁移
16. `docs/specs/fusion-blueprints/15-managed-communities-analytics-and-moderation.md` — 超级群组治理、频道统计分析与反垃圾风控审计

---

## 2. Stage 3 已吸收的在途 Open PR 资产与 5 大割裂缺陷根治清单

### 2.1 在途 Open PR 资产直接吸收（零重复造轮子）

| Open PR | 分支 | 吸收落盘目标文件 |
|---|---|---|
| **PR #191** | `origin/checkpoint/tdrp-native-call-scope-20bf9475-r1` | `frontend/src/production/human-call-media.tsx`, `human-call-media.behavior.test.mjs`, `human-call-media.fixture.tsx`, `TdrpCallAndAdminDrawer.tsx` |
| **PR #190** | `origin/checkpoint/tdrp-nmr50-recovery-20bf9475-r1` | `frontend/src/production/composer-dispatch-scope.ts`, `composer-dispatch-scope.contract.test.ts`, `ForwardMessageDialog.tsx` |
| **PR #189** | `origin/checkpoint/tdrp-forward-privacy-task20bf-r1` | `frontend/src/production/ForwardMessageDialog.tsx` |
| **PR #176** | `origin/codex/canonical-transcript-search` | `frontend/src/production/command-palette-provider.ts`, `command-palette-search-provider.ts`, `frontend/src/recovered/features/conversation/workspace/pagination.ts` |
| **PR #165** | `origin/feat/agent-browser-workspace-phase1-20261020` | `frontend/src/recovered/features/browser/workspace.tsx`, `workspace.css` |
| **PR #111** | `origin/tdrp/human-silent-scheduled-current-main-20261018` | `frontend/src/recovered/features/conversation/workspace/submission.ts`, `composer.tsx`, `frontend/src/production/human-silent-scheduled-message.contract.test.ts` |
| **PR #105** | `origin/fix/signed-in-onboarding-request-lifecycle-20261017` | `frontend/src/recovered/features/onboarding/signed-in/computer-readiness.ts`, `model.ts`, `view.tsx` |
| **PR #63** | `origin/work/fpm-sync-extension-trust-contract-20261011` | `native/mahayana-messaging/src/extension.rs`, `extension_trust.rs`, `lib.rs`, `Cargo.toml`, `Cargo.lock` |

### 2.2 `df66fb9df` 候选复核 5 项产品割裂缺陷根治验证

1. **会话顶栏统一（已根治）**：`ProductionRenderer.tsx` 与 `OpenClawTopHeader.tsx` 消除 `activeIsHuman ? .sand-chat-header : <OpenClawTopHeader>` 双轨分叉，通过 `peerKind` 与 `trailingExtras` 统一承载真人通话/转交 Agent 控件与智能体会话操作，保留 `#sand-conversation-heading` 与 WindowChrome 52px 标题栏安全边距。
2. **`OpenClawAskCopilotPanel` 假离线与右侧抽屉竞争（已根治）**：新增 `frontend/src/recovered/features/conversation/workspace/context-inspector/`（`model.ts`、`ContextInspector.tsx`、`index.ts`），移除 `isGatewayOnline={false}` 硬编码与 `setTimeout` 假回复，绑定真实 `transport === "connected"` 状态，并移除覆盖草稿的快捷药丸。
3. **左轨“任务（Tasks）”一级全局入口（已根治）**：`async-tasks/provider.ts` 与 `view.tsx` 引入 `GLOBAL_ASYNC_TASKS_SCOPE_ID` 与 `parentTaskId` 子代理谱系分组，`ProductionRenderer.tsx` 支持在真人会话或未选中 Agent 时直接打开全局任务中心。
4. **投票/待办/发票卡片与欢迎页 Prompt 草稿污染（已根治）**：`ProductionRenderer.tsx` 新增 `submitDirectPayload`，直接经由 `scopeComposerSubmission` + `composerSubmissionQueue.submit` 派发，彻底移除 `composerDraftStore.setDraft` + `setTimeout(..., 50)`。
5. **分享按钮 `window.location.href` 假回退与伪造安全状态（已根治）**：移除全部 `window.location.href` 分享回退、移除 `starsBalance = 1250` 硬编码余额、移除 `hash_${btoa(...)}` 假哈希（升级为 WebCrypto PBKDF2 / SHA-256 真实派生）。
