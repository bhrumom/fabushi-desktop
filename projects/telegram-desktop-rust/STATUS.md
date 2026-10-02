# Telegram Source-Informed Rearchitecture — Status

Date: 2026-10-02  
Spec: TDRP-001, revision 2  
Canonical spec: ../../docs/specs/telegram-desktop-rust-equivalence-migration.md  
Execution policy: SOURCE_OF_TRUTH.md

## 当前状态

本次 Revision 2 只改变实施方法和治理合同，没有交付 Telegram 产品实现。

| 项目 | 状态 | 当前说明 |
| --- | --- | --- |
| Source-informed rearchitecture principle | specified | 读取源码理解后重新设计，不做 source-file parity |
| C++ → Rust replacement rule | specified | 最终 Telegram/desktop-app C++ production logic 必须为零 |
| Best-fit non-C++ language rule | specified | 需 ADR，不强制全部 Rust |
| Fixed upstream baseline | recorded | 33261535a0e747f125e0ed25486f01e556330677 |
| Direct gitlinks | recorded | 35 个直接 gitlink；递归闭包仍未完成 |
| Full recursive source inventory | blocked | P0 未执行 |
| C++ production-logic inventory | blocked | P0 未执行 |
| Capability graph | blocked | P0 未执行 |
| Research dossiers | blocked | P0 未执行 |
| Target architecture ADRs | blocked | P1 未执行 |
| Rust C++ replacement | blocked | 未开始 |
| Product capabilities | blocked | 没有本项目生产实现证据 |
| Packaged acceptance | blocked | 没有本项目 package evidence |
| Release | blocked | 所有 AC 尚未闭合 |

complete: false

## 已废止的完成口径

不得再报告：

- 已迁移多少个 C++ 文件；
- source file → target file 覆盖率；
- source module → target crate 覆盖率；
- class/symbol 同名映射覆盖率；
- 因创建空 Rust 文件而增加的完成度。

历史 Revision 1 的这些口径只作为审计历史，不继承到 Revision 2。

## 新完成度维度

后续分别报告：

1. research coverage
2. capability designed
3. C++ Rust replacement
4. production wired
5. behavior verified
6. packaged accepted
7. platform accepted
8. release accepted

这些维度不合并成一个可被小文件数量稀释的百分比。

## 下一实施任务

P0 — Source Understanding, Capability Graph and Architecture Backlog。

产物：

- complete source/dependency/resource inventory；
- C++ production logic inventory；
- source → capability research coverage；
- capability graph；
- initial research dossiers；
- architecture risk register；
- P1 ADR backlog；
- nonempty capability ledger。

P0 不建立 source → target 文件映射。

## 首个实现 vertical slice

authentication → MTProto/session → updates → storage → dialogs/history → composer/send → server update → restart recovery。

必须通过真实 Rust production path 后才能扩大范围。

## 执行证据

所有生成器、构建、lint、schema、测试、benchmark、fuzz、packaging、acceptance 只能在 GitHub Actions 或 htch-runtime 执行。

没有 runner、OS、设备、账号或签名资格时保持 blocked/not-configured。
