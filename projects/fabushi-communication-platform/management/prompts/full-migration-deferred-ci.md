# Fabushi 全量迁移执行提示词：按产品闭环推进并及时验证

你负责完整 Telegram 来源适用职责融入当前 Fabushi，保留已有 Bot 全部能力。先读 root/nested AGENTS、FBCP/TDRP SOT/spec 和 `docs/specs/fabushi-deferred-ci-migration.md` Revision 2；它撤销旧全量完成前零 CI 限制。当前文件名仅为链接兼容。

1. 读 live main 与既有迁移线，固定 upstream U；沿用同一 inventory/ledger/RTM，完整核算来源。复用未变化 blob 的理解，不反复重读全部历史或追逐浮动上游。
2. 从当前消息生产链及状态权威收口开始，按 owner、依赖、用户价值和风险选择功能批次。每批明确用户入口、状态/权限、存储/服务、事件消费者和错误/重启恢复；不按源文件字典序持续迁移开发工具。
3. 公共 Identity/Conversation/Message/Resource/Permission 合同先稳定。复用现有 production owner；最小新 owner 须有 ADR。查明 Session/Transcript、MessagingService、旧 Telegram Provider/FFI 的实际调用关系。旧路径退出必须保护调用者和存量数据，准备版本迁移与回滚，不能直接按名字删库。
4. 支持 worker 时按独立 owner/path 并行，交付 exact base、接口与 diff；公共 schema/composition/账本由单一集成人处理。没有 worker 就沿同一依赖队列推进，不虚构后台执行。
5. 每批实现、接线和用例一起完成。冻结批次 H，在 GitHub Actions 跑最小必要的受影响编译、合同/状态机和接线验证；权限、同步、迁移和恢复关键负例及时验证。允许 push/PR/dispatch，不允许本机或 htch-runtime 执行验证。
6. 检查现有 workflow 触发范围，复用 job/harness/缓存；避免每文件、每提交全量跑。同 H/输入有效结果保留，只重验失败或失效项。CI 运行时推进独立任务；共享合同失败先修复再推进依赖批次，不重跑挑绿，不关闭保护。
7. 及时云端保存，沿用已有实现线，不 force push 覆盖他人修改。工具治理、阅读、产品实现、生产接线、测试编写、执行和独立验收分别报告；未运行保持 not-run，局部绿色不能算完整迁移。
8. Human/Agent/Group/Channel 复用统一 shell/transcript/composer；使用既有设计系统。完整消息、群频道、搜索、媒体通话、Stories、商业、平台等适用责任都要闭合，现有 Agent/Host/Coordinator/Runner/Plugins/MCP/Computer/Automations 不得回退。源工具可有证据地复用/适配/判定不适用，不机械导入 root 治理。
9. 全范围实现与接线闭合、关键失败修复后冻结 Hfinal，在 Actions 完成全部适用功能、时序/恢复、UI/visual/a11y、多语言、真实服务设备、多平台包体、性能/soak、安全及升级回滚门。允许复用同 H 同输入且覆盖对应门的有效批次证据；其他项必须补全。最终 shipping SHA 与包体 provenance 必须对应。
10. 独立 ACCEPT、0 open P0/P1/blocker、全部 required evidence 后才交付发布。缺配置明确 blocked，继续无依赖任务，不改假结果。每轮更新既有 owner matrix/STATUS/ledger 相关项，报告实际用户行为、H/U、检查选择与结果、缺口、云端位置和下一批。

现在按 FMEP-001 §12 继续现有生产批次；不要再次只扩充治理报告。政策文档合并不代表功能完成。

### Source-review amendment — 2026-11-02

FMEP-SOURCE-01 (`docs/specs/fabushi-deferred-ci-migration.md` §13) supersedes blanket per-file deep-reading/unread=0 requirements: enumerate and disposition every exact source identity; review product/mixed/high-risk logic deeply, generated and dependency contracts at their owning inputs/interfaces, and resources/docs/tooling by evidenced disposition. Preserve full product scope. Unknown or ambiguous responsibilities escalate to deep review. Historical unread counts are not rewritten as completed reads. Final acceptance requires complete identity coverage, required review depth, responsibility mapping and all original product/evidence gates.
