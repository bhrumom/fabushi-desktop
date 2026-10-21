# Bot 创建、修改和发布小程序：深入对照与实现证据

状态：实施中。不是已上架、已安装或真实模型已验收的声明。
实现 [PR #168](https://github.com/bhrumom/fabushi-desktop/pull/168)，依赖 #166/#167。
规范：[bot-miniapp-publication.md](../specs/bot-miniapp-publication.md)。
当前 CI 结论由 PR 的 exact-head checks 维护；下述 run 是不可变历史证据。

## 对照结论及已落地改动

参照 Hermes `40cc180823d182826536416de2fc27be44b4ac57` 的工具执行、失败反馈、
产物交付、后台完成和权限边界，按职责接入已有 Fabushi owner，不引入第二套 Agent。

| 责任 | 原缺口 | 实现及边界 |
|---|---|---|
| 创建/修改 | 模型只有 digest，无法通过 typed tool 读取旧代码 | read 使用 expectedRevision + SHA256 + UTF-8 字节分页；不改项目状态 |
| 已有项目 | 只能从计数器脚手架开始 | import 验证完整 src/tests/config 后原子创建，拒绝覆盖、重复路径、越界和冲突 |
| Mac 可用性 | 创建源码必须先有可用远端 Box；此前 daemon 拒绝造成反复尝试 | 显式 local 目标沿现有 ExternalShell 审核/本机权限；私有 per-agent 目录和同一 SDK；不在失败后自动改用另一目标 |
| 能力声明 | 不能把名字列出来就算已可用 | 只列当前具备 executor + trusted binding 的目标，每次调用重新检查；缺少 Python/权限/连接仍返回真实失败 |
| 发布含义 | export 和 CI 准备包尚未产生 Release | 新生成项目含显式 workflow_dispatch 发布流程；源码、GitHub 发布、市场审核、安装状态分开记录 |
| 发布前验证 | 上传包可能和通过测试的源码不一致 | 写权限 job 从同一 SHA 独立重建只含静态资源的包，逐字节核对准备产物；不解压不可信准备包，也不执行应用测试代码 |
| 版本与并发 | 标签和资产被移动/覆盖会破坏来源 | 版本匹配 tag，expectedSourceSha 匹配 checkout；每次写前后检查实际 tag，冲突拒绝；不移动 tag，不覆盖/删除资产 |
| 不确定写入 | 响应丢失会诱发重复 create/upload | draft -> 逐个确认资产 SHA256/size -> publish；下一次调用只读核对已存在资产后继续，不盲重试 |
| 凭据 | 不能把发布 token 交给应用测试、Bot 或小程序 | 应用测试在 read-only job；write token 只在独立发布步骤，由 GitHub 提供；固定 GitHub host/repo 范围，拒绝重定向 |
| 正常停止 | 不能用固定错误计数强制结束 | 延续 #167：错误回传、重复观察一次提示、执行前 checkpoint、未决写不重放；没有额外付费模型循环 |
| 市场信任 | 旧 publisher 只接受蒲公英固定签名 digest | 保留当前安全门；没有把任意 unsigned ZIP 冒充签名 .fabapp，也没有向生成项目分发审核 token |

local 目标只执行 SDK 的数据读写/导出，不执行 src/ 中的小程序业务代码。
Box 与 local 是不同 store，旧 Box 项目必须明确选择 Box，或校验源码后显式导入。
用户要求的小程序业务功能仍随安装包交付，Host 只提供通用开发/权限机制。

## 可重现证明

源 `beb622f5e20629ecdbaee4bd11b51e598acd5f8a`：
[Actions 39559157549](https://github.com/bhrumom/fabushi-desktop/actions/runs/39559157549)
通过 SDK 7 项、发布状态机 8 项测试以及真实 Rust 工具链测试。
实际 MiniApp -> reviewed Shell 创建、读源码、修改、导出后，通过 job-scoped token
向 GitHub 临时分支写入并逐文件验证 10 个导出文件，运行其真实行为测试和打包。
远端 commit `02a1d86bf2577e11246995b53a693059e8d7ad06`，原样准备包 SHA256
`809e025f07d92850f6e9c901728b704b24a9c6b5212f347cbf33cc6ff809a731`，2330 bytes。
临时分支已按精确 SHA 校验后移除。该证明不声称真实模型决定了这些操作，也没有
公开一个测试 Release。真实 GitHub Release 写入由状态化注入 transport 验证。

同源 [Actions 39559157228](https://github.com/bhrumom/fabushi-desktop/actions/runs/39559157228)
通过现有运行时合同和已提交锁文件上的 Mac Host 编译。local SDK 测试实际使用
隔离测试 home 执行同一 SDK，无远端 Box；拒绝/取消负控保留原有审核。
后续更改增加“不解析不可信 ZIP、从干净源码重建比对”的负控，仍需自身提交的检查。

全部执行在 Actions；没有在用户电脑运行构建/测试，没有恢复付费模型验收。

## 市场发布的真实剩余断点

实际 backend 的 publisher draft/upload/submit 入口是账号授权操作；
`POST /v1/extensions/review/fabapp/sign` 是单独的 reviewer 操作，返回公有签名，
绝不返回私钥。release 的 bundleDigest 和 artifact binding 都不可变。
因此不能简单去掉 native publisher 的固定 digest 检查，再把 unsigned 准备包
当成最终 .fabapp：加签后字节和 SHA256 会改变，与先创建的 release descriptor 冲突。

通用市场链路还需在现有 Registry/review owner 中完成准备提交 -> 扫描/托管签名 ->
最终签名包 digest -> 当前账号 namespace draft/upload/submit -> 实际审核状态 ->
catalog 安装证明。发布权限、审核权限和运行 native code 的权限是不同授权。
生成项目不得持有生产 reviewer token；不另建 Host signer、凭据库或审核绕过入口。
Bot 必须观察真实状态，pending_review 只能报告等待审核，不能报告已上架或已安装。

签名 Mac 安装包、当前安装客户端以及真实 Bot 模型的验收仍是独立未完成的交付层。
