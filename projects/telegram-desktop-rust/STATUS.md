# Telegram Rust 迁移 — 状态与首次执行队列

Date: 2026-10-02  
Spec: `TDRP-001`, revision 1  
Canonical spec: [完整规范](../../docs/specs/telegram-desktop-rust-equivalence-migration.md)  
Execution policy: [SOURCE_OF_TRUTH.md](SOURCE_OF_TRUTH.md)

## 当前状态

本次变更交付规范、基线和台账格式合同，没有交付 Telegram Rust 客户端实现。不得将本文的规范条目数、模块规划数或直接 gitlink 数当作迁移完成数量。

| 项目 | 状态 | 当前证据及限制 |
| --- | --- | --- |
| 产品范围、架构/owner、接口、失败语义、分阶段任务和验收标准 | specified | 主 Spec 的 17 节、FR-01…FR-18、AC-01…AC-14 |
| 上游首轮基线 | recorded | `33261535a0e747f125e0ed25486f01e556330677`；非 release tag，不是永久的“最新版本” |
| 目标调查基线 | recorded | `3b1f2c5deaf47867629c96b08bf4694f973177e2`；执行时重新读实时 HEAD |
| 35 个直接 gitlink、重要 tree/blob、根许可说明 | recorded | `upstream.lock.json`；只表示这些直接记录被读取，不等于递归闭包完整 |
| 模块/顶层文件/共享库/资源/构建对应关系 | planned | `module-map.md`；基于真实目录的职责规划，尚未逐函数核验 |
| 逐文件台账数据格式 | specified | `contracts/parity-ledger.schema.json`；非运行中的检查器，不证明 evidence 真实 |
| 递归源文件/符号/生成物/外部下载/平台和许可证清点 | blocked | P0 尚未执行；总量及覆盖率 unknown，不得填写 100% |
| Telegram Rust 产品功能 | blocked | 没有本项目实现和验收证据；不继承 Electron/Mahayana/Grok 状态 |
| 产品 CI / 安装包 / 真实互通 / 兼容性 | blocked | 没有本项目产品运行证据；文档或 JSON 检查即使通过也不改变本行 |
| 正式 release | blocked | 必须先逐项通过所有 AC 和适用平台/合规门 |

FR-01…FR-18 与 AC-01…AC-14 的初始合规状态全部是 **blocked**。后续允许逐项改为 passed 或有充分理由的 not-applicable，但不得批量继承其他项目的绿灯。全项目 complete 保持 false，直至 release 门真实通过。

## 第一次实施：P0

入口任务：[P0 — 冻结闭包与逐文件台账](management/tasks/P0-source-closure-and-ledger.md)。

顺序：确认目标 exact HEAD → 读取本规范 → 在 GitHub Actions 或 htch-runtime 展开固定上游与所有递归依赖 → 平台/构建/生成/许可清点 → 非空 file/symbol ledger → 实现和验证 fail-closed parity checker → 记录检查的 target SHA/run/command/count/hash → 独立审核后进入 P1。

不要先创建空 UI/crate 占位，再按路径数量宣布完成；不要从当前 tdesktop dev 直接替换冻结版本；不要在用户 Mac 或助手本地容器跑任何构建或测试。

## 后续阶段

P1 基础设施和 Rust UI/platform ADR；P2 协议/账号/存储；P3 真实聊天纵向闭环；P4 群/频道/消息/设置/通知/语言；P5 媒体/通话；P6 gated/敏感/长尾功能；P7 全平台和发布硬化；P8 完整独立验收及切换。各阶段的精确范围、依赖和退出条件以主 Spec §11.3 为准。

## 证据记录原则

只记录真实可读取的 run/artifact/device/log，不能为了填满表格制造链接、hash 或 reviewer。检查器执行结果由运行环境生成，并作为不可变证据归档；文档提交引用被检查产品 SHA，不能用后续文档 SHA 冒充产品测试 SHA。

失败证据保留历史，不覆盖原记录。最终 accepted 必须覆盖当前产品 exact SHA 的每一项适用测试和平台。没有真实账号/签名/设备/旧 OS 配置时保持 blocked/not-configured，而不是跳过后成功。
