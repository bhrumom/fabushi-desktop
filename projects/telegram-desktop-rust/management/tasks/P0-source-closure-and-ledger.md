# P0 — Source Understanding, Capability Graph and Architecture Backlog

Status: active  
Project: TDRP-001  
Depends on: Spec Revision 2 and fixed upstream lock  
Execution: GitHub Actions or htch-runtime only for all executable checks

## 1. 目标

建立完整源码理解基线，但不建立逐文件 Rust port 计划。

P0 要回答：

- 上游到底有哪些产品能力？
- 每个能力由哪些源码/子模块/协议共同实现？
- 所有 C++ production logic 在哪里？
- 哪些行为和失败语义必须兼容？
- 哪些原架构问题应该在目标设计中消除？
- P1 必须做哪些 ADR？

## 2. 必须产物

### A. Complete source inventory

递归冻结：

- root blobs/trees/gitlinks；
- nested submodules；
- build downloads；
- patches；
- generators；
- resources；
- shaders；
- platform build definitions；
- dependency licenses。

这个 inventory 只用于 research coverage、license 和 provenance。

### B. C++ production logic inventory

每个 C++ 区域分类：

- product logic
- protocol
- state machine
- UI
- platform policy
- build tool
- third party
- test/reference
- unreachable with evidence

最终产品相关的前六类必须进入 Rust replacement backlog。

### C. Capability graph

建立 capability → source evidence → official docs → scenarios → risks 图。

不建立 source file → target file 表。

### D. Research dossiers

优先至少完成：

1. authentication/accounts
2. MTProto transport/session
3. update consistency
4. storage/recovery
5. dialogs/history
6. message send lifecycle
7. UI architecture requirements
8. platform lifecycle

### E. ADR backlog

根据研究输出架构问题和至少两个候选方案，不在 P0 预先把上游目录结构写成目标。

### F. Capability ledger

使用 contracts/parity-ledger.schema.json 的新 capability schema，记录：

research → designed → implemented → production-wired → verified → accepted。

## 3. 禁止事项

- 不创建同名 Rust 文件来“占位完成”。
- 不按 C++ 文件数计算实现百分比。
- 不把 header/source pair 强制映射到同一个或两个 Rust 文件。
- 不把“读完某目录”标记为 product implemented。
- 不在本地运行 inventory 工具或测试。
- 不开始大规模页面实现，直到 ADR-UI-BACKEND 通过。
- 不把 source-informed 路线称为 clean-room。

## 4. 退出条件

P0 只有在以下条件同时成立时通过：

- 固定 upstream 闭包完整可重建；
- 每个 source leaf 已分类到 research capability 或有不相关理由；
- 所有 C++ production 区域均已进入 Rust replacement backlog；
- 首批 research dossiers 完整；
- capability graph 无 unknown top-level product area；
- architecture risk register 和 P1 ADR backlog 已创建；
- capability ledger 非空且 fail-closed；
- 允许环境中的检查通过并绑定 exact target SHA。

P0 通过不表示任何 Telegram 产品 capability 已实现。

## 5. 下一阶段

P1 完成 target architecture ADR，然后以“认证 → protocol → sync → storage → dialogs/history → send/recovery”作为第一个 Rust vertical slice。
