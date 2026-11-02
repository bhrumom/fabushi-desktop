# OpenClaw → Fabushi 全量融合执行入口

先读 root/nested AGENTS、OCRM SOT、主 Spec Revision 2 §17、当前 task，以及当前批次相关生产源码；按需读领域合同，不反复加载全部历史。

1. 核对 live main、既有迁移线/未提交变更、锁定 U 与机器台账；沿既有实现线继续，旧 discovery 快照不覆盖新锁，不追逐浮动 upstream。不得覆盖其他 worker。
2. 完整登记来源；产品/混合 UI/高风险深读，generated/dependency 审查输入/API/补丁/消费者，文档资源工具作证据处置。不机械迁移 .agents 或重复治理。未知升级审查，缺功能不能 N/A。
3. 如工作分支仍使用全局 unread=0，先按 §17.2 同批更新现有 schema/validator/正反用例；保留历史阅读事实与 false flags。主干规范不代表机器门已迁移，不另造账本。
4. 从真实 producer→owner→state/storage/service→consumer 查当前缺口，优先模型推理、channel/session/投递、cron/Automations、插件权限/生命周期等关键链。选择用户行为批次，定义依赖、oracle、失败/恢复与出口；复用现有 owner，业务 Rust，UI/桥接保持薄。
5. 当前批次必要来源与合同已清楚即可实现；全局 closure 未通过不阻止独立批次。真正未知依赖先解决。shared contract 稳定后按互不重叠路径并行，共享文件单一集成人处理。
6. 实现、生产接线和用例一起推进。及时云端提交；批次 H 在 Actions 验受影响编译/合同/状态机/集成及高风险负例。允许 push/PR/dispatch，不在本机/htch-runtime 跑验证。复用已有工作流，避免每文件全量 CI。
7. 等待期间推进独立任务；共享合同失败先修。相同 H/输入有效结果保留，仅重验失效或失败项。不重跑挑绿、不虚构真实设备/provider 通过。缺外部条件记录 owner、解除条件与可推进任务。
8. 保留全部 OpenClaw 功能与授权的 channel/Gateway/SDK/node 适配，禁止第二 Agent loop/session/scheduler/marketplace/OpenClaw runtime；Fabushi 原有 Bot 能力不回退，TDRP/OCRM 证据独立。
9. 全范围闭合后冻结最终候选，完成全部 G01–G12/AC01–AC24、真实生态/设备/平台、UI/时序/安全/性能、包体升级回滚与独立 ACCEPT。遵守 subject target/carrier 区分，局部绿色不提升全项目 flags。
10. 每轮只报告 U/H、真实能力与接线、必要审查缺口、验证选择/结果、保存位置、阻塞和下一批。继续产品实现，不陷入反复报告与规则扩写。
