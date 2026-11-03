# SG74 media owner/oracle implementation report

2026-11-03 UTC。Agent：`/root/media_74_gate_diagnosis`。本轮按 root 明确指定的 inherited **xhigh** 执行，未改变模型配置。状态：**实现已原位写入、源码/diff 已读；编译、测试、parser 及平台执行均未进行，未宣称 verified/accepted。**

## 1. 授权、先行决策与源身份

Authoring root：`/workspace/scratch/fadc4429f7a8/fusion-shared-gates-74-20261103`。

实施前完整读取本轮 `projects/openclaw-rust/evidence/shared-gate-recovery-d56b3d1d-20261103/decisions.json` 中 SG74-MEDIA-OWNER-01 / SG74-MEDIA-ORACLE-02 及现有 `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` 的完整 MSU-v1 和新增 SG74-MEDIA-v1。Windows 新证据到达后，先暂停 AST 编辑，继续仅处理已经授权的 DB 测试；root 新增 SG74-MEDIA-WINDOWS-03 与 SG74-M03 后，再读取这些具体文本，随后才编辑 AST/换行入口。三项决定均为 FIX / TEST_FLAW，未以失败日志自行改写验收要求。

复用此前完整只读诊断 `/tmp/media-74-gate-diagnosis.md` 的 MSU/MAC/FUA、事务与 guard 合同、真实 owner/helper/consumer 阅读。AGENTS 和主 Spec §17 从 exact H Git 对象读取；authoring root 是限路径副本，未包含本地 AGENTS/主 Spec，首次本地读取失败已明确观察，未自行创建或修改它们。根 AGENTS 的适用范围与之前诊断相同，没有编辑 subtree 的额外 AGENTS。

固定来源 U：`1440bb1771372334bea23d7642bb52deb7db1845`，tree `90c8aab39972da11cfad220361781d49e1bc6700`。本次原始失败 subject H：`74f11d3d31c6245ac5886dc0181460b2d4349744`，tree `aca91e1cfbedeecb039d442fb3528caac89873a3`。root 指定 source base `b15e58957a597997363e1273f2f32a222365c310`，并说明两路径直到 `4f0639321c4ae03cc7dbef1d8e49dea0f042e180` 源码未变。

编辑前逐字节将两本地文件与 bare repo `/tmp/ocrm-source-84120837.git` 中 H 的对应 blob 比较：均完全相同。因此所附 diff 精确以 H 的这两个源文件为 before，不依赖 authoring root 的 Git index 或可变 ref。

| 文件 | Before bytes / SHA256 | After bytes / SHA256 | diff |
| --- | --- | --- | --- |
| `source/host/src/extensions/session/agent_db.rs` | 426548 / `9a8ad4aa952f11a3d7595eebf636f7d20c450381120416c00667b93ef0149c7b` | 427027 / `5cf63b89b1d2c1d9b4c283b60257376cddc087ac4051aa5936ff4cadf0bd2ea3` | +14 / -2 |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | 162305 / `e61629a709e4154dda1297d3f5fa9fd13efc38c00d67c08edbc93fb70a27d32e` | 170216 / `6415e02b5915074acba5e9b3ed5e7d894fb8f02f91d84bb0b4a4e46bebcf0802` | +160 / -20 |

完整 unified diff：`/tmp/shared-gate-74-media-impl.diff`，26701 bytes，SHA256 `63b98ae9f2697328b4209b511ac75c9f3de43a535795d84212f84ed281ac9d93`。两文件已经原位编辑；diff 是完整 review/handoff 产物。

## 2. SG74-M01：owner 完整集合与真实可见集合

只修改 ADB 的 `#[cfg(test)] mod media_settlement_tests` 内两个原失败断言。当前边界：lost-ACK 的 9362–9368；exact-owner 的 9583–9589。其前所有字节与 H 相同，完整 diff 也没有任何生产方法、helper、type、import 或 schema 改动。

每处现在先比较：

```rust
owner.get_transcript_entries().unwrap() == fixture.rows()
```

然后实际调用 `get_transcript_page(TranscriptPageQuery { before_seq: None, since_ms: None, until_ms: i64::MAX, limit: 10 })`，要求 `entries == vec![entry.clone()]`。页面入口实际委派 `read_transcript_page`，SQL 只选择可见 kind，`page()` 使用 `parse_transcript_entry`，不增加 seq 字段，也没有测试内过滤 receipt。

`fixture.rows()` 不是唯一 oracle：现有 `assert_committed_storage` 完整保留，仍用独立 literal JSON 固定恰好两行及顺序，逐字段固定 send 与 receipt 的 id/kind/sessionKey/mediaWorkId/lifecycleRevision/mediaPaths/transcriptEntryIds，并要求释放的 admission/fence 和原媒体字节。lost-ACK 测试在 reopen 前及每轮显式 close 后固定该 image；另一个测试在 other 成功提交后及投影检查后固定该 image。测试状态中没有新增写事务，所以 owner/raw 读互等不能替代或绕过 literal 期望。

完整保留的原要求包括：

- true COMMIT 后 fault claimed/applied、typed Unknown，错误不可伪装普通拒绝；
- 调用 owner closed、connection None、准确剩余 live handles、recovery count 为零；
- no unknown publication、write generation 不变、closed owner retry 拒绝；
- 两次 fresh reopen，每次 admission 清除检查、显式 close 和 raw image 再检查；
- session/revision/work 三 mismatch 不消费 fault、不改变 pending image；
- 同文件同身份另一个 owner 的成功不能消费原 fault；Unknown 仅退役调用 owner，另一个 owner 仍开放；
- 最后 events/close 等原先因早期失败未到达的尾部，全部保留。

此修复不修改 owner receipt 解码或隐藏规则；它要求存储和 owner 必须保留 receipt，同时真实呈现 API 只显示 send。

## 3. SG74-M02：两个 inherent selector 修复

当前 ORACLE 路径均指 `source/host/tests/openclaw_sessions_and_config_contract.rs`。

1. `media_commit_inherent_impl` 当前 1703 起：候选 filter_map 在收集前跳过 `item.trait_.is_some()`。原有 self_ty 最后 segment 筛选、候选唯一性、exact path、qself/attrs/trait/defaultness/unsafety/generics/where 检查保留。没有将 duplicate inherent 或 foreign-qualified owner 当合法例外。
2. `media_commit_owner_is_handled` 当前 1748 起，其逐 item 重解析在 1801 左右同时要求 `implementation.trait_.is_none()` 和 exact `SandAgentDb` path，避免用 inherent grammar 解析 `impl Drop for SandAgentDb` 的 `for` token。

这两处仅控制“选择哪个 impl 作为有界 owner”。没有删除或替换 trait source，没有改生产 Drop，也没有删整个 impl。后续 channel token walker 仍遍历完整 source image。原 method-sized exception、真实 helper bodies、public typed signature、prefix、直接调用同步写闭包及三 guard、完整 tail、returned outcome、rollback/autocommit/retirement/Unknown/publication 全部保留。

`channel_commit_call_count(...) != Some(1)` 保持原文。全局 `assert!(!owner_source.contains("db.execute_batch(\"COMMIT\")?;"))` 保持原文。media 仅移除已证明的 exact raw helper method，并替换其一个已证明调用的 oracle-local 图像；rollback 本文和每个其他 raw COMMIT、alias、UFCS、reference、macro token 仍可见。没有提高 caller allowance，也没有改 channel command 的生产实现或既有 bounded proof 规则。

## 4. SG74-M03：Windows CRLF 入口与证据界限

root 的先行 Windows 决策记录：原 owner 仍 11/2，commit 为 20/7，比 Linux 多一个 `media_commit_oracle_rejects_missing_duplicate_or_conditional_owner` 在原 line1945 `source.find(MEDIA_COMMIT_OWNER_SIGNATURE)` 失败。root 对原 Windows 62 输入做字节审计，ADB 工作字节 436762 / SHA256 `276b6c58a5260c7c0c3fd1561d65249eba307827767c894e149ba0e4bcb9fdb9`，ORACLE 工作字节 165691 / SHA256 `119cf6ef914b4342719f63cfc7a032c754152b0c6c04a0f9d031d665ef83d946`，均为 Git LF 的精确 CRLF 投影。这部分依赖已读 root durable decision；本 agent 没有扩大任务再独立读取 Windows artifact 或 API。

新增且仅有一个换行政策函数 `transaction_commit_source_image`（当前 1695 起）：

```rust
if source.contains("\r\n") {
    Cow::Owned(source.replace("\r\n", "\n"))
} else {
    Cow::Borrowed(source)
}
```

只处理成对 CRLF；LF 路径借用原字符串，不复制、不改写。没有 trim、空白折叠、注释/trait/token 删减、整源重排；lone CR 保留。源码仍是独立输入哈希所绑定的 source，只是在 oracle 自己的文本图像中匹配 Rust multiline literals。

该函数接入四个必要入口，均沿同一政策：

- `actual_transaction_commit_oracle_fixture`：在任何 actual-source `find/replacen` 之前规范化，并返回完整 String；
- `source_without_owned_media_commit_adapter`：允许直接 LF/CRLF 调用有界 media proof；
- `source_without_owned_channel_commit_adapter`：保护其 literal 边界及 raw command 匹配；
- `agent_db_transaction_owner_source`：支持独立 channel fixture 和 AST method/tail 证明的 LF/CRLF 输入。

所有 12 个受影响 actual-source 使用点均切到共享 fixture：media 正例、六拒绝族、raw 禁令保留测试、global raw 禁令测试，以及三个 channel actual-source setup。不能只在 parser 内规范化，因为旧 Windows `find` 会更早失败；本实现已覆盖这一顺序问题。未修改无关 admission/source 文本测试、workflow、.gitattributes、Git 配置或生产文件换行。

## 5. 有效基线、变异控制与新增 trait 反例

`actual_transaction_commit_oracle_fixture` 当前 1896–1915：

- 首先对未变实际 LF image 取得两 adapter composition Some；
- 再要求实际 CRLF image 同样 Some，且所得完整 remaining image 与 LF 一致；
- 固定两个 inherent header 和 commit/rollback/signature/prefix/tail 七个 mutation 边界在实际 source 中各出现一次。

因此六个 media rejection family 在创建任何 mutation 前已经取得有效实际基线，不会继承 H74 的“baseline None 导致所有 negatives 假通过”。

`assert_rejected_crlf_mutation` 当前 1917–1925：对每一份 mutation 要求 `changed != source`，LF→CRLF 必须确实改变文本；规范化回来的整源必须恰好等于 changed；CRLF mutation 必须拒绝。原每份 LF mutation 的 `syn::parse_file(...).is_ok()`（适用时）和原 rejection assertion 仍保留。原 malformed negative 仍保持 malformed 路径，不强迫它合法解析。

| 原固定测试族 | 保留要求与新增控制 |
| --- | --- |
| `media_commit_oracle_accepts_actual_owner_and_preserves_admission_guards` | 原 guard检查、raw count、rollback文本、channel方法、composition、ordinary RAII positives 全保留；增加实际 LF/CRLF 全 image 等价、lone CR 保留、原 Drop 文本保留，以及给 SandAgentDb/MediaSettlementCommands 同时增加 benign trait 的 LF/CRLF 正例。 |
| `...rejects_missing_duplicate_or_conditional_helper` | 原缺失/foreign/qualified/duplicate/helper-method-comment/cfg/malformed 全保留；增加把唯一 inherent helper 改为 trait impl 的拒绝；每份都具有效基线、changed 检查及 LF/CRLF 结果。 |
| `...rejects_missing_duplicate_or_conditional_owner` | 原改名、重复 impl/方法、cfg、pub(crate)、返回类型、qualified、comment tail 全保留；增加 trait-only owner 拒绝；Windows literal lookup 在 normalized fixture 后进行。 |
| `...rejects_changed_commit_or_rollback_commands` | 原假 COMMIT、丢 error、另开事务、假 ACK 成功、假 rollback、forget authorizer、吞结果全保留，并对同一 mutation 同时要求 LF/CRLF 拒绝。 |
| `...rejects_extra_aliased_or_macro_callers` | 原 15 free-function/macro调用及 helper sibling 全保留；新增两个 owner 的 trait 内各 4 个调用控制：alias.commit、alias.r#commit、UFCS、method reference，共 8 份，全部 LF/CRLF 拒绝。 |
| `...rejects_discarded_or_changed_settlement_handling` | 原 10 tail 变异、async/move closure 两变异全保留，逐份 changed 与 LF/CRLF 拒绝。 |
| `...rejects_changed_unknown_entry_handling` | 原六个 poison/prior/BEGIN/rollback/retirement/条件变异全保留，逐份 changed 与 LF/CRLF 拒绝。 |
| `...preserves_raw_commit_prohibition_outside_proven_method` | 原 owner前/后、helper sibling、write closure 内 raw COMMIT 全保留；增加两个 owner 的 trait 内 raw COMMIT。每份 LF/CRLF image 都须仍可证明两有界 adapter，且 remaining 必须保留 forbidden direct statement。 |

两类 trait 负例有意区分：无合法 inherent owner/helper 必须直接拒绝；合法 owner 旁边的无关 trait 可以保留，但 trait 中的 extra adapter caller 仍使完整 caller 证明失败，trait 中直接 COMMIT 则必须留给全域禁令拒绝。没有把“跳过 trait 候选”变成“跳过 trait 安全检查”。

channel 原测试全部保留。由于 channel literal proof 入口也使用同一 EOL 政策，额外在既有 channel positive、error-arm mutation、missing/ambiguous/malformed helper 测试中加了 LF/CRLF baseline 或相同 mutation 拒绝控制；原 ordinary RAII、隐藏 owner、alias/reference/macro、tail/cleanup/unknown、唯一调用规则不变。

所有新增检查放在原测试函数或非测试 helper 中，**没有新增、删除或重命名 `#[test]`**。原 native owner 13 名、commit selector 27 名的固定 capture 集合不变；没有改 capture 文件或准入计数。这里的“集合不变”来自完整 diff 的源码审查，不是运行 `--list` 或 test harness 得出的新执行结果。

## 6. 源码审查的具体覆盖

本轮在先前完整诊断基础上额外读取：最新全部 MSU及三决策；H AGENTS/§17；当前两授权文件编辑前字节身份；两失败完整测试、fixture literal storage/retirement 和 import/page入口；media constants 对照真实 signature/prefix/tail；两个 selector、完整 media proof/六 mutation族、shared-source composition、channel token计数/AST proof/actual setups；页面类型/实际SQL/`page()`结果构造；全部最终 unified diff。

人工审查确认的源级事实：

- ADB 生产前缀字节完全未变，只有两处 cfg(test) hunk；使用已有 `TranscriptPageQuery`，无需新增依赖或 public接口。
- 原 receipt/fault/connection/readers/custody/typed outcome 实现均未编辑，guard/table/constants均未改。
- Rust 字符串变异在完整 normalized fixture 上完成；新 traits 的 source fixtures 保留完整 trait定义/实现或 trait-only counterfactual，原 syntax validity assertions仍在。
- 所有原有效 mutation 仍存在，旧失败尾部未删；无 assert ignore/should_panic/降低数量/吞异常改动。
- raw global prohibition 与 `Some(1)` 按源码保留，normalizer不删除任何安全相关 token。
- 本轮最终源码两路径及 diff 的 hash 见上表；生成 diff 使用 Python 标准库 difflib 与只读 Git blob，未调用项目 helper/parser。

这属于 author 自审，不冒充 independent ACCEPT。root 可将原位文件和 exact diff 提供给新的独立 focused reviewer；本 agent 未读取其他 reviewer 的 AP 或 peer reports。

## 7. 未执行状态和交付边界

**未运行** cargo、rustc、test、test --list、syn parser、project Python capture/helper、schema、lint、format、build、htch 或其他本地项目执行。没有调用 Git add/commit/update-ref/reset/checkout/push；没有修改 index/refs；没有 dispatch；没有改工作流、spec、ledger、inventory、decisions、公用 schema、composition 或其他 agent 路径。除两授权源码文件外，本轮只写要求的 `/tmp/shared-gate-74-media-impl.diff` 与本报告。

尚未冻结/物化新 H；任何当前源码都尚无新 H 的 Linux/Windows 执行证据。原 Linux 8 失败及 Windows 多出的 fixture 失败不被本报告覆盖或重标。没有称修复已经通过，也没有推定所有被早退遮挡的问题均已排除。

待 root 按既有门在 GitHub Actions 验证：固定 13 owner + 27 commit 名称完整执行；有效 LF/CRLF baselines、原全部 negatives 和新增 trait/EOL controls；既定 admission guard、独立 actual App/Session/Runner、ordinary feature-off shipping、平台/FUA 等受影响门。source声明/materialization及exact-child证据、独立review和最终accepted状态由root管理。此次仅修复测试与oracle，不提升任何 OCRM 全局完成标志。
