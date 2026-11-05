# Antigravity /teamwork 执行委托

用户授权（2026-11-05）：在 Fabushi 内完整实现 Telegram 与 OpenClaw 能力，重新设计自然统一的 UI/UX；由 Antigravity CLI /teamwork 团队推进，Codex 独立验收。

本委托与 `docs/specs/unified-fabushi-experience-20261105.md` 是已经完成的本轮设计/范围决定。无需重新采访用户，也不要只写方案后结束。正式执行分阶段实现；若 teamwork 必须产出 approval artifact，先返回该 artifact 与 conversation ID，由委托方按已授权范围核对批准。

工作根：`/Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105`；基线 7f6548d42，独立分支 design/unified-fabushi-20261105。不要用旧 origin/main 覆盖本分支。不得修改其他已有 worktree；团队可从当前分支创建自己的隔离子 worktree，整合者唯一写共享接口/ProductionRenderer/ledger。保持现有未提交内容。

优先阅读 AGENTS.md 与 UFX 设计，再完整读取适用 FBCP/TDRP/OCRM 规范、项目 SOURCE_OF_TRUTH、当前 task/ADR、canonical-screen-patterns/ui-component-contract/visual-acceptance 和执行政策。设计增补不豁免既有全范围验收。所有复杂阶段 xhigh；模型继承实际配置，记录实际设置；不能为调参改用户全局设置。

上游只读：
- `/Users/bhrum/.gemini/antigravity/scratch/openclaw` @1440bb1771372334bea23d7642bb52deb7db1845
- `/Users/bhrum/.gemini/antigravity/scratch/tdesktop` @f985723b552b30fe1c17d377d57f44e61a15a8c4
先验证 source identity 与 current owner。必须学习业务源码与协议/状态机，不只读 README/UI 截图。沿现有 inventory/ledger/RTM 补齐，不能用重做文档冒充实现。不要改锁以追最新版。

完整目标按 UFX M0–M5 推进，分团队但按依赖集成。第一可交付 M1 必须移除假的 Ask OpenClaw 第二聊天、恢复统一生产 header/composer 与真实状态，保留所有已有 typed action/批准/上传/输入法/草稿功能；不能靠隐藏所有入口让截图好看。现面板 setTimeout 伪造回复/默认 gateway 在线是已确认缺陷。M2 闭合任务真实状态与子代理 parent 元数据，不能靠名称过滤；之后持续闭合完整 TDRP/OCRM 产品能力。

验证默认仅 GitHub Actions，禁止本机 build/lint/test/generator/benchmark/packaged acceptance（遵守根 AGENTS；发现专项条款冲突记录并采用严格约束）。可只读/编辑/Git/API；允许任务分支 push、PR、dispatch。服务/账号/runner 缺失明确 blocked 并记录解除条件，继续无依赖工作；mock/stub/静态文本不是功能验证。不要安装应用、发布 release、改权限策略或泄露秘密。

实施者与 Critic/Challenger/Auditor 独立。每次交付记录 exact head、diff、source references、实际生产调用链、测试命令/Actions run/数量/退出码、截图/录像链接、spec compliance、剩余缺口；Codex 最终核验。不把 M1 交付说成完整融合。项目 complete=false 直到所有原项目 gates 与 UFX AC 通过。

请建立任务级 durable request/plan/progress/evidence，报告实际 teamwork 团队/会话 ID、工作目录、角色与首批实施状态。保留当前规范之外的内部 task 状态与恢复标识，能够继续同一工作，不能只说已安排。

---

## 强制开发流程增补（`UFX-FLOW-01`：全源码精读 $\rightarrow$ 分模块蓝图 $\rightarrow$ 子智能体落地）

所有团队与子智能体必须严格执行根 `AGENTS.md` 中的 **`UFX-FLOW-01` 四阶段开发流程**（标准执行提示词位于 `projects/fabushi-communication-platform/management/prompts/full-unified-fabushi-fusion.md`）：

1. **阶段一（全量三方源码精读与四步精读法）**：在修改任何生产代码前，严格按“四步精读法”与“源码五问”深度阅读三方源码（Telegram Desktop 50+ 能力域 + OpenClaw 32 责任域 + Fabushi 现有生产源码），通过真实调用链定位每个功能点，确保 **0 遗漏**。
2. **阶段二（编写全景能力索引与动态分模块蓝图）**：根据源码理解，在 `docs/specs/fusion-blueprints/` 目录下首先编写 `00-master-capability-and-module-index.md` 全景能力索引，再按高内聚原则产出 `01..N-*.md` 动态分模块融合蓝图（严格遵循 7 章节模板：功能矩阵、割裂根因、职责与 Canonical Owner、极致 UI/UX、精确 TS/Rust 契约与状态机、子智能体写入白名单、防作弊验收标准）。
3. **阶段三（子智能体按蓝图严格执行）**：仅当对应模块的蓝图文件落盘后，主控才可将具体 `docs/specs/fusion-blueprints/0X-*.md` 派发给 Worker 子智能体按契约实现，根文件 `ProductionRenderer.tsx` 由唯一整合者串行窄接线。
4. **阶段四（独立取证审计）**：由独立 Reviewer/Challenger/Auditor 对照蓝图与 `docs/specs/unified-fabushi-experience-20261105.md` 逐行核验，确保彻底消除 `activeIsHuman` 双顶栏割裂、移除 `OpenClawAskCopilotPanel` 及 `isGatewayOnline={false}` 假离线、合入唯一右侧 `ContextInspector`、打通无门控全局 Tasks 中心、修复 Poll/Todo 50ms 定时器覆盖草稿 Bug 与 `window.location.href` 分享回退。
