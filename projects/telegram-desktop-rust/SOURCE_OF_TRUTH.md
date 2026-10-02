# Telegram Desktop Rust 迁移 — Source of Truth

Status: active  
Date: 2026-10-02  
Project ID: TDRP-001  
Repository: `bhrumom/fabushi-desktop`

## 最新明确要求

将 `telegramdesktop/tdesktop` 使用 Rust 逐个文件、逐个模块等价迁移到本仓库，覆盖完整功能、UI、协议、数据、平台、工具与发布路径。本次工作先把完整、可验收的 Spec 持久化，不把规范落库当作实现完成。

## 必须先读

1. 根 [`AGENTS.md`](../../AGENTS.md)。
2. 主规范 [`telegram-desktop-rust-equivalence-migration.md`](../../docs/specs/telegram-desktop-rust-equivalence-migration.md)，尤其范围、Rust/FFI 边界、状态所有权、P0–P8 和 AC-01…AC-14。
3. 本文件的执行约束，以及 [`upstream.lock.json`](upstream.lock.json)、[`module-map.md`](module-map.md)、[`contracts/parity-ledger.schema.json`](contracts/parity-ledger.schema.json)、[`STATUS.md`](STATUS.md)。
4. 之后逐工作单元读取其真实上游文件、调用链/条件分支和台账；不得只读本概述就开始写空壳代码。

以上是一个规范体系，不存在互相覆盖的第二套 Telegram Spec。主规范定义产品要求；本文件补充用户明确的执行场所限制和入口。若有冲突，以最新明确用户要求为首，先修改规范，不擅自猜测。

## 执行场所 — 强制约束

**全部构建和测试只能在 GitHub Actions 或 `htch-runtime` 上执行。禁止在开发者本地电脑、Mac/Windows 工作站或助手本地工作容器运行构建、测试或验收。**

该限制包含参考 C++ 客户端构建、Rust/Cargo、生成器执行、lint/fmt 检查、JSON/schema 检查、单元/合同/差分/集成/E2E、安装包验证、性能/功耗/fuzz/soak；不得以“轻量检查”“临时验证”为由本地执行。只读查看、编辑文件和 Git/API 提交不属于运行测试。

跨平台用例优先通过 GitHub Actions 的对应系统 runner 执行。旧 OS/真实设备必须以受控 GitHub Actions self-hosted runner 提供；或在 `htch-runtime` 上以适用且可证明的环境执行。未配置相应运行环境时标记 `not-configured/blocked`，不能改到用户 Mac 上直接测，不能用 Linux 编译或模拟截图代替 Windows/macOS 实测。

`htch-runtime` 证据必须记录设备标识、目标 exact SHA、源 lock 摘要、实际命令、环境、退出码、测试数量和日志/产物摘要，并归档到对应的验收记录。正式安装包的 provenance 和 canonical-main 集成门仍按主规范 §12.4/§13 执行。禁止用重新现编的包冒充已锁定 CI 安装包。

## 固定源与现有代码

- 上游：`telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`。
- 目标调查起点：`main@3b1f2c5deaf47867629c96b08bf4694f973177e2`；每轮实际工作都须重新读取目标 HEAD。
- 目标 Rust workspace：`telegram-rs/`，属于规划路径，初始不存在不代表缺失文件已实现。
- 已有 Grok/Agent、Electron、Mahayana 保留自身范围和验收。本项目是依据最新要求批准的独立 Telegram Rust 项目；其业务 UI 不能永久依赖旧 Electron，亦不把 Grok 模块替代 Telegram 的源码职责。
- 不在规范阶段删除、移动或重写既有运行代码，不自动切换用户数据或启动入口，不向其他仓库实施本项目。

## 下一步

执行 P0：在允许的运行环境完整递归展开冻结源码、外部下载和生成链；建立所有叶文件/符号/平台的台账、许可和资源处置；实现并验证 fail-closed inventory/parity 检查器。所有初始行从 `discovered` 开始。

当前只有规范与基线记录。全量递归文件数、符号数、映射覆盖率及产品通过率尚无可靠证据，必须显示 `unknown/blocked`，不能显示 100%。
