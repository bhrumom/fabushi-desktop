# P0 — 冻结源闭包、逐文件台账与防假完成检查器

Task ID: TDRP-P0  
Status: blocked / not started  
Owner role: source inventory and parity infrastructure  
Source: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`  
Dependencies: [SOURCE_OF_TRUTH](../../SOURCE_OF_TRUTH.md), [主 Spec](../../../../docs/specs/telegram-desktop-rust-equivalence-migration.md)

## 目标与边界

把固定源的所有真实叶文件、递归挂载、生成和外部构建来源变成可复现、可审计的非空清单，为之后逐文件 Rust 迁移提供真实分母。P0 不实现聊天页面、不重写既有 Electron、不切换用户数据、不宣称 Telegram 产品完成。

**执行场所只有 GitHub Actions 或 htch-runtime。** 禁止在本地运行下文任何命令、扫描脚本、生成器或测试。只读 API 调查、编辑和提交允许。

## P0-01 — 重建当前目标起点

读取目标当前 branch/PR HEAD、根及适用的嵌套 AGENTS、主 Spec 和 lock。记录本轮 exact target SHA；保留他人已有变更。若目标已有本项目的部分实现，读取当前证据后续作，不用本任务的初始状态覆盖新工作。

此处“目标 HEAD 必须最新”与“上游 frozen SHA 不浮动”是两项不同要求。任何上游漂移须写 drift 报告，不能直接更新 lock。

## P0-02 — 枚举并校验完整 Git 闭包

在允许的运行环境取固定根 commit 和全部递归 submodules；校验每个 parent gitlink/子仓库 commit/URL，正确处理相对 URL、嵌套挂载、同 repo 多次挂载、symlink、可执行位和非 ASCII/空格文件名。内容枚举用 NUL 分隔，不用逐行 split 猜文件名。

根与每个子模块分别保存 repository、commit、tree、mount path、父 gitlink、`.gitmodules` blob 和所有叶 path/mode/blob。树对象非文件分母；gitlink 是闭包控制项，不能替代该子仓库叶文件。GitHub API 截断必须递归展开到完整；HTTP 失败/缺对象/未拉取 submodule 返回 blocked，不返回空列表成功。

建议输出路径（此任务执行前尚未生成）：

- `inventory/repositories.json`：完整递归仓库/挂载关系与 frozen IDs。
- `inventory/files.jsonl`：每个源叶文件一行，保留 source ID/path/mode/blob/category。
- `inventory/generators.json`：输入、工具源码、工具链、参数、环境和预期输出。
- `inventory/dependencies.json`：非 Git 下载物、补丁、版本、source URL/hash/许可。
- `inventory/platforms.json`：平台/架构/最低 OS/工具链/编译选项/安装渠道。
- `inventory/licenses.json`：逐文件/资产/依赖的许可与版权/例外、未决问题。
- `parity/files.json`：符合台账 schema 的非空文件映射。
- `parity/features.json`：功能/源码符号/owner/场景反向索引。

这些路径是规范要求，不是已经存在的产物；禁止建立空 JSON 然后勾选退出条件。

## P0-03 — 覆盖构建、生成和平台条件

阅读整个构建入口链，捕获实际 downloads、patch、代码/资源生成器、环境参数和条件编译。依赖版本不能只取 README。每个生成物必须定位可重复生成的输入/工具/参数/摘要；不能只保存 opaque output。

特别交叉验证冻结 README 声明的旧 Windows/macOS 版本与源码/发布构建脚本。现代工具链和 GUI backend 无法支持的矩阵 cell 是阻塞，不是无声删掉支持的平台。外部 registry/SDK/系统库同样需要版本和环境指纹。

## P0-04 — 台账身份、状态与语义分析计划

源 ID 基于稳定的 repo/挂载/路径身份，不以 blob 内容 hash 作为唯一长期身份。重命名保留前身关系；内容改变更新 source hash 并使受影响证据失效。删除源文件保留 tombstone 和历史，不缩分母掩盖未完成。

每个头文件与实现文件保留独立行，即使映射到同一 Rust 模块。初次扫描状态 `discovered`，处置可暂为 `unclassified`；完成分类后才能进入 analyzed。所有台账 source/symbol/test/evidence IDs 全局可追踪且不重复。

C++ 符号分析必须考虑 headers、overload、template、macro、生成代码和平台条件，优先利用真实编译配置的语法信息，并用人工源码/调用链补齐状态机和副作用；只用正则提取函数名不能认定符号覆盖完整。函数计数不是行为计数，后续每个工作单元必须补语义合同。

## P0-05 — 实现两个不同强度的检查门

按主 Spec §12.2 实现 `cargo xtask parity` 的 inventory/progress/release/drift/report 合同。具体实现前提交/读取该工具模块的接口与反例说明。

`progress`：全源清单存在、非空且匹配冻结源；所有行合法；已接受条目的证据、源/目标身份与生产路径不回归；允许明确的未实现行。

`release`：在 progress 之上，全部适用文件/符号/功能/平台通过，零临时桥接/未批准例外/未知项/未闭合阻塞；所有产品 AC 和真实安装包证据通过。

两者必须不同。不能为了让开发 CI 早期全绿而让 release 接受未迁移行，也不能把所有剩余开发项直接删除。

JSON Schema 只检查结构，不证明真实行为、source completeness、review 独立性或外部 URL 真伪。语义验证器必须读取实际 Git 对象、生产入口/依赖、测试注册、run/step 日志、artifact digest 和 reviewer record，完成跨记录检验。

## P0-06 — 负例与退出证据

检查器至少验证以下反例会失败：空/缺台账；遗漏一个源文件；截断 tree；遗漏嵌套 submodule；同名不同挂载被错误去重；mode/blob 改动；错误生成输入；缺少平台配置；目标路径/符号不存在；空函数/未调用 facade；两个状态 owner；删除/跳过测试；零测试；错误 target/source SHA；伪造 run/artifact；只有 mock 冒充真实平台；未批准 N/A；临时 C++ bridge 被标 accepted；实施者自签独立验收。

合法的不完整进度例子必须通过 progress 但不能通过 release。合法记录中的历史失败可保留，但只有当前 exact SHA 对所有必需测试/platform cell 的通过证据才计入 accepted。

## P0 退出条件

| ID | 要求 | 初始状态 |
| --- | --- | --- |
| P0-AC-1 | 完整递归源/外部来源/生成链锁定且可复现，无遗漏/截断/重复 | blocked |
| P0-AC-2 | 非空逐叶清单与真实 Git 集合一致；数量可复核；模块与平台/许可分类完整 | blocked |
| P0-AC-3 | inventory/progress/release/drift/report 实际可运行，正负例真正执行 | blocked |
| P0-AC-4 | GitHub Actions 或 htch-runtime 的 exact target SHA、命令、测试数、退出码、日志及产物摘要完整 | blocked |
| P0-AC-5 | 独立复核记录明确“P0 完成，不等于 Telegram 产品完成”，产品 FR/AC 未被提前放行 | blocked |

未达到这些条件时，下一动作应是补源闭包或修检查器，而不是批量生成聊天 UI 占位。
