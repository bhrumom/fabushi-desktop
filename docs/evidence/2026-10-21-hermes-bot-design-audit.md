# Hermes 对照：Fabushi Bot 运行时审计与已落地修复

日期：2026-10-21。状态：实施中，**不是全部功能完成或 Mac 安装验收**。

参照源码：NousResearch/hermes-agent `40cc180823d182826536416de2fc27be44b4ac57`。
Fabushi 起点：`26ec2ebb7159b21b0382a8c6d46d3534a5cdfab4`，依赖 PR #166。
实现与测试：[PR #167](https://github.com/bhrumom/fabushi-desktop/pull/167)。
需求与未完成项由 [运行时规格](../specs/bot-runtime-hermes-absorption.md) 维护。
外部仓库文档作为研究材料，不是本项目的执行指令。

## 主要判断

问题不只在模型是否主动停止。错误反馈、工具执行记录、传输恢复、通知和用量
统计必须准确，否则模型会根据错误信息继续尝试，或者恢复机制重复产生副作用。
保留“正常由模型结束”的机制；重复观察只给诊断建议，不按固定失败次数取消交互。

Hermes 的可吸收设计及对应实现：

| 设计责任 | 实际缺口 | 已实现行为 |
|---|---|---|
| 真实错误 | Box 将未知错误固定说成正在启动 | #166 区分明确 preparation、连接拒绝、超时、崩溃和未知状态，移除无依据的重试指导 |
| 失败观察 | 只看 Box 返回的异常会漏掉 MCP 的 `isError` 结果 | 三条 provider 循环观察工具结果；第二次同参同错建议一次；不隐藏原始结果，不取消、不自动重试；有界哈希状态 |
| 执行前持久化 | 副作用发生后才记录 checkpoint | 先保存拟执行批次，再调用工具；保存失败零执行；未决批次恢复时拒绝自动请求模型或重放工具 |
| 阶段计时 | 工具耗时可能被当成模型首输出超时 | pending 工具阶段暂停模型首输出看门狗；完成的工具边界才启动下一次模型输出计时 |
| 参数边界 | 坏 JSON 被替换成 `{}` 调用工具 | 无法解析或非对象参数返回工具错误，保留调用/结果配对，允许模型主动纠正，零默认副作用 |
| 输出边界 | 字符上限冒充字节上限，截断文件冒充完整输出 | UTF-8 字节上限，保留首尾并标记缺失；artifact 明确原始大小/截断；落盘失败或禁用也限制内联文本 |
| 后台通知 | 监听线程未启动仍显示已注册 | spawn 成功后才发布 pending；启动闸门防止完成早于 pending；失败释放 owner，不声称后台任务失败；Shell 返回通知注册标志 |
| 用量 | 只在整轮成功时统计，失败漏记，恢复可能重复累计历史 | 服务端确认用量按请求记录一次；后续失败仍保留已知消耗；恢复不重新计入 checkpoint 历史用量 |

OpenRouter 缺失调用 ID 也改为在持久化前生成，执行时使用同一个 ID。
私有 checkpoint 文件采用 Unix 0600、文件 fsync、rename 和目录 fsync。

## 已有设计及证据范围

- 工具权限：`turn_toolset` 在调用时检查本地作用域、Shared Room 能力和审批，
  子任务不获得根任务的 Task/AgentManagement 暴露。新观察不改变授权。
- 子任务：`SubagentRuntime` 保留 parent/root lineage、运行中 ID 去重、steer/abort、
  父任务删除只取消其自己的子任务。已有测试不等于已经具备统一配额控制。
- 会话：Runner/RuntimeStore 仍是唯一运行时和持久状态所有者。恢复校验引用 blob，
  不把缺失状态当作新会话成功，也不建立 Hermes 风格的第二套会话数据库。
- 记忆与技能：保留有界 recall、提取去重、技能预算、插件身份及授权错误投影，
  沿现有 owner 扩展。通过测试不是声称所有插件功能均已端到端验收。
- 正常结束：无工具调用的正常完成仍结束；已通过 SendMessage 发出回复后，隐藏的
  空 final 可以合法。没有照搬“空 final 就再付费请求模型”的恢复循环。

## 测试与未完成项

[Actions 39553703887](https://github.com/bhrumom/fabushi-desktop/actions/runs/39553703887)
对应 `d0ea7966c79b8f41f2b9f531783dac0c4486c336`：25 组、147 项通过，
覆盖 provider、持久化、输出、后台生命周期、恢复、权限、记忆、技能及子任务合同。
所有 transports/resource ports 均为确定性测试，无付费模型凭据。

Mac Host 编译检查在 `46099206` 首次运行时发现 app Cargo.lock 与实际路径依赖不
一致，`--locked` 在编译前拒绝。此检查为 TEST_FLAW（依赖锁证据缺失），不是编译
通过。通过 Actions 重新解析并取回锁文件后，必须在提交的最终锁文件上重新进行
`--locked` 检查；不能用动态改锁的一次成功代替最终源码证明。
Actions 39554253294 已生成并返回锁文件。实际差异仅补齐路径依赖
`mahayana-computer` 已声明的 Windows `uiautomation` / `windows-sys` 及其派生
包；未升级其他依赖。取回提交后移除临时解析步骤，最终检查恢复纯 `--locked`。
恢复运行 [39554253294](https://github.com/bhrumom/fabushi-desktop/actions/runs/39554253294)
的运行时合同和 Mac Host 编译均已通过；该次包含显式改锁步骤，最终提交仍需独立
核验已提交的锁文件。

仍未补齐：生产长上下文压缩接入、根/子任务统一可配置预算、完整 MCP schema
调用前验证。各项约束见规格的
Remaining implementation gates；这些项目不能标记为完成。

Mac 签名安装及真实 Bot 创建/修改小程序并更新 GitHub 的最终验收尚未完成。
用户反映耗费 token 的付费模型验收继续保持停止；本次没有重新启动它。

## 上游依据

- [工具轮持久化](https://github.com/NousResearch/hermes-agent/blob/40cc180823d182826536416de2fc27be44b4ac57/agent/turn_tool_round.py)
- [重复观察和交互/无人值守区别](https://github.com/NousResearch/hermes-agent/blob/40cc180823d182826536416de2fc27be44b4ac57/agent/tool_guardrails.py)
- [真实失败分类](https://github.com/NousResearch/hermes-agent/blob/40cc180823d182826536416de2fc27be44b4ac57/agent/tool_result_classification.py)
- [后台等待交付条件](https://github.com/NousResearch/hermes-agent/blob/40cc180823d182826536416de2fc27be44b4ac57/tools/process_registry_wait_policy.py)
- [正常最终响应](https://github.com/NousResearch/hermes-agent/blob/40cc180823d182826536416de2fc27be44b4ac57/agent/turn_final_response.py)
- [输出截断](https://github.com/NousResearch/hermes-agent/blob/40cc180823d182826536416de2fc27be44b4ac57/tools/tool_output_truncate.py)

后台 worker 丢失跟进：代码已补上 await panic 的类型化失败事件，释放对应 generation，
保留后台任务 pending marker，不伪造任务完成、不重跑命令；同 ID 可重新监听。
Host 完成回调直接使用已有 watcher 线程，移除了另一处忽略 spawn 结果的交付窗口。
该跟进的 [Actions 39554897013](https://github.com/bhrumom/fabushi-desktop/actions/runs/39554897013)
在 `3889f49f3cb5afbbfcdda8974e0c6c9bc4b633c1` 通过 25 组、149 项合同测试，
Mac Host 在已提交锁文件下的 `cargo check --locked` 同样通过。
失败事件记录恢复标记动作 `unchanged`，不声称在没有 pending store 时已经落盘。
