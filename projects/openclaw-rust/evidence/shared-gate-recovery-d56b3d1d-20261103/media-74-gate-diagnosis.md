# H74 Linux media gate 只读因果诊断

诊断日期：2026-11-03 UTC。诊断者：`/root/media_74_gate_diagnosis`。范围：run `41580539491` / attempt `1` / job `125734079585` 的 `capture_media_cleanup` 两组失败；不对其他 gate 或整体迁移作验收结论。

## 1. 结论与证据性质

**逐项分类：本次观测到的 8 个失败均为 TEST_FLAW；没有从这 8 个失败得到 IMPL_BUG 或 ENV 的证据。** 两个数据库测试把 owner 的完整持久行集合误当成呈现集合；六个事务 oracle 测试被同一 actual-source 基线选择错误阻断。第二类还有一个被首个错误遮挡的同类重新解析错误，最小修复必须覆盖两处。

这不是宣告实现没有 bug，也不是预言修复后的 Actions 一定通过。运行证据只属于 H74。失败断言之后尚未执行的要求、修复后的 oracle 基线及 mutation 敏感性，都必须在新 H 的 Actions 重新取得证据。本报告未运行 Rust、Cargo、syn/parser、项目 Python helper、lint、format、build、schema 或 htch 验证；只读 Git/source/原始日志，以及使用 Python 标准库读 ZIP/JSON、计算原始字节哈希。未修改仓库文件、索引、refs，未 push/dispatch，未读 peer review 报告。唯一写入物是本报告。

分类依据是“合同 → 实际 owner/consumer → fixture/oracle → 原始失败”，不是从 `left/right` 反向改断言。适用执行约束先读根 AGENTS 和主 Spec §17：本任务属于高风险事务/失败控制诊断，继承 agent 的 reasoning 配置；未声称手工更改模型 effort。主 Spec §17 的风险分层允许针对相关责任深读，但不允许把未读范围标成已读、把 TEST_FLAW 修复冒称产品验收，或在本机执行项目验证。

## 2. 精确身份与原始输入核对

| 项目 | 身份/核对结果 |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| 被诊断 H | `74f11d3d31c6245ac5886dc0181460b2d4349744` |
| H tree | `aca91e1cfbedeecb039d442fb3528caac89873a3`；独立 `git rev-parse` 核对 |
| 固定 U | `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`，tree `90c8aab39972da11cfad220361781d49e1bc6700` |
| Git 读取来源 | `/tmp/ocrm-source-84120837.git`，所有源码读取显式指定 H，不依赖可变 checkout |
| 原完整 job log | `/tmp/ar-74-linux-evidence/linux-original-decoded.log`；1,220,138 bytes，17,762 行；SHA256 `4630d79b0ecbdbd01cf5f0ce4fd2f34a4696fed8e20dbff0501d529f125acf43` |
| 原 artifact | ID `13205686727`；`/tmp/ar-74-linux-evidence/media-original.zip`；142,379 bytes；SHA256 `284a316258ebf9b60dd29d781c6aceb44ef3d1ca351a3337e0945efd7b2699b5` |
| ZIP receipt | `receipt.json`，120,067 bytes；SHA256 `5fc8dfd24cf2635e253056c12d06c1f67f29e66f5cabe9e3003a1efc0bdfc6e2` |
| owner 原日志 | `media-settlement-owner.log`，30,567 bytes；SHA256 `26ab3027d0bd03ff450927df814349347f9fb104f0b8ec2677b4126aa838fad8` |
| commit 原日志 | `transaction-commit-owners.log`，32,185 bytes；SHA256 `921b7317adf72cfad5ddd22b541f1e366a4d8beddc48214b913f642fbaaf1bc5` |

ZIP 内为 receipt 加 19 个原 case 日志。以上 ZIP、receipt、两 case 日志已独立按原始字节核对。artifact ID/API 绑定由 root 提供；本 agent 未重新联网获取 artifact。原 job log 的相关失败段与 ZIP 中两 case 日志的内容、source panic 位置、计数一致。

receipt 的 `subject_commit`、`subject_tree`、`upstream` 与上述 H/tree/U 一致；`run_id=41580539491`、`run_attempt=1`、`job=sqlite-native-path`、`runner_os=Linux`。记录的 Rust 为 `rustc 1.97.1 (8bab26f4f 2026-07-14)`、host `x86_64-unknown-linux-gnu`；Python 3.12.3。环境记录无外部 SQLite include/lib/static/pkg-config 注入。此处复述原记录，不据此推定其他平台行为。

**62 项 capture 输入身份已做完整字节审计。** `inputs_before == inputs_after`，receipt 的 `inputs_unchanged=true`。逐项读取 H 的 Git blob，独立计算 bytes、SHA256、Git blob SHA1，全部吻合每项的 `bytes`、`working_tree_sha256`、`git_blob_sha`、`normalized_worktree_blob_sha`，无 mismatch。这是输入身份核验，不等同于对这 62 文件全部语义深读。关键 blob：

| H 路径 | Git blob |
| --- | --- |
| `AGENTS.md` | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` |
| `source/host/src/extensions/session/agent_db.rs` | `6c27e9bb9f665b1160457b9acc29dd5d64fba289` |
| `source/host/src/extensions/session/agent_db_serde.rs` | `d1885e2d31313c1f525fffd85bafc5031cf8d127` |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `ba7dde9c12eb14cb0a0615a49a013a2e126b860d` |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `e42ea96e12c4647c23d98154fabd4a87749ea5c7` |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | `bab905e889a472027a906b551a874081131cc85e` |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `8efd93cc604fa814b4660b5153b6b38033faa2ca` |

两个失败 case 都是 `cargo_exit=101`、`capture_exit=0`、`execution_error=null`、`oracle_accepted=false`。其余 17 case 在 receipt 中各为 cargo/capture 0、accepted true；本报告不把这些 receipt 状态扩展成相关产品全部通过。整 receipt 正确保持 `result=failed-or-incomplete`。`oracle_qualification.accepted=true` 只指 capture 的 13 项合成日志准入控制，不能覆盖 Rust AST oracle 的基线失败。

后继 b15/4f 等 evidence-only 提交不是这次执行 subject；本报告不把 H74 结果重标为其运行证据。

## 3. 适用合同及实际路径

以下行号均指 H74，而非后继工作树。简称：ADB=`source/host/src/extensions/session/agent_db.rs`；ORACLE=`source/host/tests/openclaw_sessions_and_config_contract.rs`；SERDE=`source/host/src/extensions/session/agent_db_serde.rs`；SESSION=`source/host/src/extensions/session/production.rs`。

### 3.1 MSU/MAC 的不可删减要求

`media-settlement-unknown.md` MSU-01～05（51–115）要求：真实 COMMIT 成功才允许成功；任何已尝试 COMMIT 的错误保持 typed Unknown，不能以随后 rollback/autocommit 洗成普通拒绝；事务前失败只有确认清理并回到 autocommit 才能 ordinary refusal；Unknown 退役恰好当前 connection，不发布成功，不重开后自动重试/重播；durable send、receipt、fence、原媒体字节与同 ID retry 由各状态真实保留。

MSU-06（119–141）允许第二个**逐方法证明**的 media command exception，但 channel 例外的完整 owner/tail/唯一调用、macro/alias 负例及原始 raw COMMIT 直接传播禁令必须保留。MSU-07（142–154）要求 test-only、身份绑定、one-shot、逐 owner 隔离的 fault lease，旧 lease Drop 不能清新 arm。MSU-08～10（155–182）要求真实 DB 命令、真实 App/Session/Runner 消费链；原生 owner gate 不能替代独立 App gate。

`final-assistant-media-custody.md` 的 durable receipt（26–33、88–97）与 MSU 更正（134–150）共同限定：receipt 是持久事务凭据，Unknown 不能销毁其媒体，也不能把控制行当普通对话显示。`media-file-url-admission.md` FUA 合同及 FUA-PIPE、`local-media-file-url-admission.md` 保留原入口、路径/字节/权限与平台否定控制；它们没有要求 owner 丢弃 settlement receipt，也没有给 raw COMMIT 放宽例外。本次两处 receipt 比较/六处 oracle 失败均不发生于 URL 转换、路径权限、cap 或平台 native opener。

`channel-inbound-worker-handoff.md` CA-01～06 与 oracle 追加（69–245）要求 ordinary/unknown 区分及独立 command 的狭小例外。`session-work-admission.md` 的事务 fence（202–224）和 guard owner 追加（1701–1763），连同 ORACLE 716–1155，要求实际 owner 及直接同步 write closure 的 guard；不能以注释、字符串、宏文本、未调用 closure、条件编译复制来骗取证明。

### 3.2 实际 owner 与 consumer，不是 fixture 假装状态

ADB `append_transcript_entries_and_release_media`（4799–5104）在真实 connection 锁内执行 BEGIN、同步 write closure、三次 identity admission check、send/receipt 插入和 fence CAS；receipt 已存在时逐字段核对并验证原 send，精确 replay 返回空 inserted 集合。`MediaSettlementCommands::commit`（6253–6263）先实际 `db.execute_batch("COMMIT")?`，只有真成功后才可模拟丢 ACK。real COMMIT 错误不被 fault 取代。rollback（6265–6278）实际执行 SQL；测试 authorizer（6280–6341）只拒绝 ROLLBACK，guard Drop 清理 authorizer。

commit-error tail（5064–5103）即便 rollback 清理成功仍退役当前 owner、返回 `TranscriptCommitUnknown`；成功分支只有确认 autocommit 后才 drop 锁并发布。`retire_channel_inbound_connection`（4576–4585）取走并关闭该 connection、注销该 handle；并不关闭另一 `SandAgentDb`。fault slot/lease（882–932、4723–4794）与三身份及具体 owner 绑定。

完整读取 API 的链为 ADB `get_transcript_entries`（5843–5857）→ `read_transcript_entries_from_db`（7777–7789）→ SERDE `parse_owner_transcript_entry`（174–177）。SERDE 169–188 明确把 `outgoing-media-settlement` 等控制行作为 owner 有效行保留；schema 的 LIST ALL（`agent_db_schema.rs` 97–98）不筛呈现 kind。与此相对，SERDE `parse_transcript_entry`（161–167）及 page SQL（schema 128–148）服务呈现；SESSION `hide_session_work_receipts`（41–48）与 `read_agent_transcript`（1290–1300）隐藏内部 receipt。这不是临时为失败提出的区分，而是已存在的两类 API 合同。

真实消费链已经读取：App PendingOutgoingMedia/AgentManagement sink（`app/src/main.rs` 708–1275）→ Transcript runtime（`production_runtime.rs` 1797–1837）→ SESSION（4063–4084）→ ADB。typed Unknown 保留经过两层 Result 边界；App 先 seal custody 再分配/提交，Unknown 不走 ordinary build-error cleanup，也不在该失败路径发送成功 ACK。`attachment_paths.rs` 24–105 的 custody 在 pending/unknown Drop 下保留媒体。SESSION 1640–1891 的首次打开恢复读取 receipt 和 admission，按身份释放/拒绝恢复；这些路径是源码审核，**不是本次原生两组失败执行的 App 证据**。

## 4. 八个失败逐项分类

日志锚点用完整 job log 行号；括号为 ZIP 中相应 case 原日志行号。测试完整名称可由下表直接定位。下表前两项共同前缀是 `extensions::session::agent_db::media_settlement_tests::`。

| 测试 | 原失败锚点 / H74 source | 分类及独立因果 |
| --- | --- | --- |
| `media_settlement_fault_matches_three_identities_and_only_its_exact_owner` | job 1560–1563；owner log 709–712；ADB 9577 | **TEST_FLAW**。另外的 owner 保持开放，实际完整读取为 send+精确 receipt；断言错误期待仅 send。owner 隔离本身已在 9575–9576 成立。 |
| `media_settlement_lost_commit_acknowledgement_retires_without_publication` | job 1568–1571；owner log 717–720；ADB 9362 | **TEST_FLAW**。real COMMIT 已落盘，失 ACK 使调用 owner 退役；第一次 fresh reopen 的完整读取正确包含 receipt，断言误用呈现单行预期。 |
| `channel_commit_oracle_rejects_hidden_owner_and_aliased_calls_in_actual_source` | job 3063–3064；commit log 788–789；ORACLE 2619 | **TEST_FLAW**。对 actual source 先证明 media exception 的 setup 得到 None，尚未执行该组 channel hidden/alias mutations。 |
| `channel_commit_oracle_retains_unsafe_owners_before_after_and_inside_helper_impl` | job 3069–3070；commit log 794–795；ORACLE 2471 | **TEST_FLAW**。独立小 fixture 的 adapter/ordinary RAII 正例已越过，actual source 的 media setup 失败；后续 actual-source 留存检查未到达。 |
| `media_commit_oracle_accepts_actual_owner_and_preserves_admission_guards` | job 3074–3075；commit log 799–800；ORACLE 1889 | **TEST_FLAW**。1886 的 admission guard owner 检查先完成；actual media owner 正例因把 Drop impl 当第二 owner 返回 None。 |
| `media_commit_oracle_preserves_raw_commit_prohibition_outside_proven_method` | job 3079–3080；commit log 804–805；ORACLE 2097 | **TEST_FLAW**。第一份外加 raw COMMIT 反例的 adapter 基线先失败，未到 remaining.contains(direct) 的禁令保留断言。不能解释成 raw COMMIT 已漏过或可放宽。 |
| `test_agent_db_has_no_direct_commit_error_propagation` | job 3084–3085；commit log 809–810；ORACLE 2360 | **TEST_FLAW**。两独立 adapter 的 composition 在 media 证明处停止，原全域 raw COMMIT 禁令检查 2361 根本未执行。 |
| `test_commit_error_oracle_rejects_unguarded_propagation_and_missing_cleanup` | job 3089–3090；commit log 814–815；ORACLE 2411 | **TEST_FLAW**。actual media adapter setup None；下游 channel rollback/autocommit/unknown 等 mutation 尚未运行。 |

运行小结：job 1541–1554 / owner log 690 起报告 13 tests；job 1578 / owner log 727 为 11 passed、2 failed、0 ignored。job 3030–3057 / commit log 755 起报告 27 tests；job 3101 / commit log 826 为 21 passed、6 failed、0 ignored。两个程序均已编译并执行至具体断言，原 capture 没有 execution_error，加上输入完全吻合，**不支持 ENV 分类**。没有利用后续重跑覆盖此原失败。

## 5. 两个 DB 失败的完整因果与最小修复

### 5.1 丢 COMMIT ACK

fixture（ADB 9094–9240）建立真实磁盘 SQLite、真实原文件、合法 admission；`assert_committed_storage`（9189–9204）用 literal JSON 固定恰好两行及顺序：send，随后 receipt；receipt 的 id、kind、sessionKey、mediaWorkId、lifecycleRevision、mediaPaths、transcriptEntryIds 全部逐字段期待。另固定已释放 admission 与原字节。

测试 9340–9369 在失败前依次完成：fault 未 claim/apply；append 返回 typed Unknown；fault 确实 claim 且在 real COMMIT 后 applied；当前 owner closed/connection None/live handles=0/recovery count=0；真实 raw DB 已是精确两行且 fence 清除、媒体原字节保留；publication 为零，write generation 不变；closed owner retry 拒绝且 closed read 为空。然后 fresh reopen 成功，9362 用 owner 全量读取，却期待 `[entry]`，与同一测试刚通过的 durable receipt 合同直接冲突。

**未执行边界：**第一次 reopen 的 admission 检查 9363–9364、显式 close 9365、再检查 storage 9366，以及第二轮 fresh reopen 和最终 events 9368 均未到达。不能把通过了初始 durable image 说成“两次 reopen 已通过”。

### 5.2 三身份与逐 owner fault 隔离

测试 9545–9581 在 session/revision/work 三个 mismatch 上均 ordinary refuse，fault 未 claim/apply，pending storage 不变。另一具体 owner 在同文件、同三身份上提交成功且原 owner fault 仍未 claim；9569 再次固定精确 send+receipt image。随后原 owner 对既存 receipt 做精确 replay，此调用的真实 COMMIT 丢 ACK，返回 typed Unknown；9575 `assert_retired(1)` 证明只退役调用 owner，9576 证明 `other` 仍活着。9577 又把 `other.get_transcript_entries()` 错期待为单 send。

**未执行边界：**9578 的再次 durable image、9579 的 zero publication 以及 9580 的显式 other.close 尚未到达。之前成功 other 提交本来允许成功发布，events 在 9570 才订阅，因此这里要证明的是随后 Unknown 不发成功，不能把两次调用的发布责任混成零。

### 5.3 修复选择及必须保留的要求

最小修改限于测试：把两处 owner 读取和完整 durable rows 比较，同时独立验证实际呈现 API 只返回 send。可使用 `get_transcript_entries() == fixture.rows()`，**条件是同状态仍保留 `fixture.assert_committed_storage(&entry)` 的 literal 两行精确断言**，使 raw/read 两者即使共同出错也不能互相“证明”。也可把已有 literal 期望抽成纯 fixture expected rows 再复用；不得以生产 receipt builder 生成所有 expected 字段。

呈现验证使用真实 `get_transcript_page` 与明确 query（before_seq/since_ms=None、until_ms=i64::MAX、足够 limit），比较 entries 恰好 `[entry]`，或用既有 Session facade；不能只在测试内过滤 owner 返回值然后假装呈现合同被执行。`agent_db_transcript_pages.rs` 45–98 与 schema 128–148 已给出真实筛选路径。App 既有 `media_settlement_unknown_tests.rs` 224–287 已分别比较完整 raw image 与 Session 可见 entries，是可参照的正确分层。

保留：全部 receipt 字段/精确行数顺序、admission/fence、媒体字节、fault claim/apply、两次 fresh reopen、connection/live-handle/recovery 计数、same-ID retry、zero unknown publication/write generation，以及 other owner 仍可用。禁止改生产全量 owner API 来隐藏 receipt、删除 durable receipt、把 `==` 降成 contains/非空/仅长度、删除失败用例或其尾部。

## 6. AST oracle 的两层确定性缺陷

### 6.1 首个实际阻断点：候选集合错误

ORACLE `media_commit_inherent_impl`（1693–1709）的 filter_map 先只按 `self_ty` 最后 path segment 收集候选，再在 1703 检查 `candidates.next().is_none()`；`item.trait_.is_none()` 到 1705 才检查。H74 ADB 同时存在：

- 951：`impl SandAgentDb {`，目标 inherent owner；
- 6237：`impl Drop for SandAgentDb {`，正常 destructor，6239 调 `self.close(false)`；
- 6252：`impl MediaSettlementCommands {`，另一个正常 helper。

前两者的 AST self_ty 都是 `SandAgentDb`。因此 1744 对 SandAgentDb 调 selector 时得到两个候选，返回 None；尚未进入 signature/prefix/write-closure/tail 证明。trait 检查落在集合唯一性之后，不能排除第二候选。该原因不依赖 Linux I/O、SQLite、编译时 cfg 或 runtime fault；这是对 H74 原源码和 selector 的静态逐步推导，并与六个 `.expect` 的运行锚点一致。没有运行本地 syn 来“复现”。

### 6.2 只改第一层仍会失败

ORACLE 1782–1858 手工重读每个 top-level item；1788 匹配所有 Item::Impl，1790 又只看 `self_ty` 是 SandAgentDb。合法 `impl Drop for SandAgentDb` 仍会进入，以 inherent grammar 重读：1792 消费 impl，1793 把 Drop 当 Type，1795 立刻要求 `{`，但实际下一个 token 为 `for`。解析 error 经 1858 的 `.ok()?` 再返回 None。它目前被 1744 的早退遮挡；修复首层后才会暴露。

最小修复是**两处都仅选择 inherent impl**：1694–1700 的候选收集先跳过 `item.trait_.is_some()`；1790 的逐项重解析也在处理前跳过 `implementation.trait_.is_some()`。现有 guard oracle 的 ORACLE 937 已有排除 trait impl 的先例。不要移动/删除/重命名生产 Drop 来迁就测试，不要给所有 SandAgentDb impl 或全部 source 一个例外。

### 6.3 必须区分“选择 owner”与“保留检查范围”

跳过 trait 的唯一用途是选出受证明的方法 owner。**所有 trait impl 原 token 必须留在 source 中，继续参与后续完整 caller/alias/macro/raw COMMIT 检查。** source_without_owned_media_commit_adapter（1862–1875）只移除已证明 exact raw COMMIT 方法，保留 rollback 全文，且仅把那一个已证明的 `commands.commit(db)` 替成 oracle-local 占位；不能删整个 impl、trait 或写入闭包。其余 raw `db.execute_batch("COMMIT")?;` 必须仍可被 2361 禁令拒绝。

保留以下原限制：唯一 unconditional inherent owner/helper；typed public signature；没有重复方法/条件 attrs/foreign-qualified owner；完整 entry prefix；直接调用同步 write closure 与三 guard；完整返回 tail（COMMIT/rollback/autocommit/retirement/Unknown/publication）；真实 command exact bodies；channel 全部旧 owner/tail/error cleanup；`channel_commit_call_count` 的 `Some(1)`。不能把“一共两个适配器”变成全文件 Some(2)，也不能因承认 media adapter 例外而忽略 extra/alias/UFCS/raw-identifier/macro caller。

生产 command 的真实事务规则在本次故障测试中另有运行覆盖；AST oracle 的正确接受本身仍需新 H Actions。上述两点是确定的必要修复，不排除新运行揭示其他被早退遮挡的问题。

## 7. mutation 负例有效性与重新资格化

H74 的六个 `media_commit_oracle_rejects_*` 测试虽记录 `ok`，都从 `include_str!(actual ADB)` 建立 mutation，然后期待 composition 为 None。因为 **未变的 actual baseline 已为 None**，这些 `ok` 不能证明是 mutation 导致拒绝。它们中的 syntax assert 有价值，fixture 内容也有价值，但当前记录不构成拒绝能力的因果证据。不能把 21 passed 简单解读成“所有负例有效”。

| 现有测试/族 | 已读范围 | 必须保留的反例与重新验证条件 |
| --- | --- | --- |
| `media_commit_oracle_rejects_missing_duplicate_or_conditional_helper` | ORACLE 1911–1940 | 缺/foreign/qualified/重复 helper、重复方法、注释假方法、impl/method/file cfg、malformed；先证明原 baseline Some。 |
| `...missing_duplicate_or_conditional_owner` | 1943–1970 | owner 改名、重复 impl/方法、cfg、pub(crate)、返回类型改变、qualified owner、tail 注释；固定目标唯一、changed 非原文。 |
| `...changed_commit_or_rollback_commands` | 1973–1996 | 假 COMMIT、丢 error、另开事务、ACK 假成功、假 ROLLBACK、forget authorizer guard、吞结果；原方法 expected 字面匹配唯一。 |
| `...extra_aliased_or_macro_callers` | 1999–2028 | direct、alias、r#commit、UFCS、method reference、macro definition/invocation、helper sibling；全部源 token 保留参与 caller oracle。 |
| `...discarded_or_changed_settlement_handling` | 2031–2062 | 丢弃尾结果、cfg、颠倒条件、Unknown 降级、缺 rollback/retirement/drop/autocommit、Unknown 变成功、async/move closure。 |
| `...changed_unknown_entry_handling` | 2065–2080 | poison/prior/BEGIN Unknown 改 ordinary、去 retirement/rollback、autocommit 条件恒真/恒假、条件 guard。 |
| `...preserves_raw_commit_prohibition_outside_proven_method` | 2083–2101 | owner 前/后、helper sibling、write closure 内 raw COMMIT 必须被保留；adapter 仍能独立证明，然后全局禁令拒绝。H74 未到此断言。 |
| channel actual-source 负例 | 2408–2441、2451–2501、2615–2699 | 清理/unknown变更、helper前后/内部/隐藏owner、alias/macro 等旧要求全保留；三个 actual setup 当前失败。 |
| channel 独立 fixture 负例 | 2503–2613（完整读取包含于 1390–2701） | extra callers、method refs、changed/discarded handling、missing/ambiguous/malformed helper 可独立运行；不受 media baseline None 自动冒充通过，仍应完整回归。 |

建议每个实际 media mutation 族共用一个 fixture helper：首先断言 actual source 的 media proof 和两-adapter composition 都 Some；再对每次替换断言目标唯一/`changed != source`，对 intended-valid-syntax 的 mutation 保留 `syn::parse_file(...).is_ok()`，最后判断该被破坏的合同确实拒绝。malformed 反例另保留 malformed 预期。不要只把六个拒绝断言包进一个永远返回 None 的分支。

针对此次 selector 修复，应补互补控制：

1. 正例：实际 source 连同既存 `Drop for SandAgentDb` 可证明，附加一个无关合法 trait impl 也不影响 inherent owner 选择。
2. 负例：只有 trait 方法而无合法 inherent owner/helper、duplicate inherent impl、conditional/foreign owner 仍拒绝；不能因跳过 trait 让“无 owner”变成功。
3. trait 范围的未受保护 raw COMMIT：例如在保留正常 Drop 的源里，插入独立 trait 方法或修改 trait body 加入 direct 字面式。adapter proof 应保持其自身有效且 remaining 仍包含 direct；global 禁令因此拒绝。
4. trait 范围 extra adapter call/alias/UFCS/method reference：必须使 caller 证明拒绝，不能被仅选 inherent 的实现误从全文件扫描范围抹除。

这些是修复后必须执行的对照要求，不是本 agent 已运行/已通过的测试。现有 guard oracle 的 8 owners/11 calls、单个 media closure 三 guard、comment/string/macro/dormant/conditional/async 的负例也不应删减。

## 8. capture 的责任与后续最小验证范围

`capture_media_cleanup.py` 对 13 media owner names 与 27 commit names 作固定唯一结果核对（204–256）；assess（358–388）要求非空、准确 names、无 missing/duplicate/unexpected/failed/ignored、零退出及有效成功 summary。run_case（418–442）记录 cargo/capture/error/log SHA 后继续采集，main（449–560）最终所有 case accepted 才成功。原 capture 因真实失败标红是正确行为；**不应更改 required names、扩大 selector 限制、删除 failing tests、将 failed summary 当 passed，或把 `cargo_exit=101` 容忍为成功。**

建议由 root 先落 durable TEST_FLAW/FIX 决策，再仅改相关 test/oracle；任何 spec/index/ledger 更新由 root 独占。source 改变后只在 Actions 上取得新 H 的完整受影响 capture：两个原 case 的固定集合、全部有效 mutation、独立 session admission guards，以及 native 平台/FUA/既定 App gate 要求。增加 top-level test 时同步固定 name 集合；也可在既有测试内加补充控制而不改测试名。精确新 H/U/input/artifact 证据必须重新绑定；原 H74 failure 不覆盖、不重标。

原生 receipt 已明确不证明 actual App、Session first-open recovery、generated Runner、完整路径/权限闭包、真实服务、crash matrix、打包或全迁移完成。它们的既有 required gate/blocked 条件仍须保留，不能因为本次诊断是 TEST_FLAW 就提升全局完成标志。

## 9. 实际读取范围与未验证边界

下表是实际语义读取的主要连续区间。检索、文件名枚举、原始字节哈希不计深读。读取的大输出曾发生截断，相关事务/helper/test/oracle缺口已用小区间补读；对非相关长表只记录看到的行或检索，不冒称全文。

| 路径/范围（均 H74） | 实际读取说明 |
| --- | --- |
| `AGENTS.md` 1–395 | 全文；tree 中另一个 AGENTS 在 mahayana/codex-rs/tui/bottom_pane，不适用本范围。 |
| `docs/specs/openclaw-rust-equivalence-migration.md` 637–720 | 当前执行部分，含 §17/OCRM-EXEC-02；其余只索引检索，没有全文深读。 |
| `projects/openclaw-rust/SOURCE_OF_TRUTH.md` 1–95 | authority、固定 U、约束与历史身份区分；其余未深读。 |
| `projects/openclaw-rust/module-map.md` | relevant M07/M23/M24/M29/M32 等行检索；58–95 部分表格可见，存在截断，不记完整逐行深读。 |
| specs `media-settlement-unknown.md` 1–207；`final-assistant-media-custody.md` 1–207；`media-file-url-admission.md` 1–327；`local-media-file-url-admission.md` 1–171 | 四份全文，含追加修订、保留负例与执行限制。 |
| `channel-inbound-worker-handoff.md` 1–245 | 含 CA 事务语义及 oracle 追加；后续部分未完整深读。 |
| `session-work-admission.md` 145–230、475–565、686–708、1701–1763 | 此次所需 admission/transaction/guard 合同；其余只命中检索。 |
| ADB 1–130、870–1085、1375–1685、4560–4605、4710–5108、5815–5915、5920–5995、6080–6128、6235–6470、6502–6644、6660–6705、7735–7910、8056–8081、8124–8182、9080–9633 | 完整相关 media owner/commands/lease/fixture/13 tests、channel command/tail/error helpers；含 CAS、open/close、retirement、owner readers。并非整个 ADB 文件全文。 |
| SERDE 120–225；`agent_db_schema.rs` 95–180；`agent_db_transcript_pages.rs` 1–156 | owner/presentation decode、相关 SQL、page模块全文。 |
| ORACLE 716–1155、1390–2701 | admission guard owner/parser/有效负例，以及完整 media+channel source oracle/constants/组合/全部相关反例。其余测试文件未全文读。 |
| SESSION 1–64、1272–1330、1620–1905、2283–2340、2682–2820、4005–4115、4818–4962 | 真实 receipt投影、owner缓存/first-open恢复、FS mutex、media admission/release、typed append facade；最后一区间属额外读取，不提升无关责任状态。 |
| `extensions/transcript/production_runtime.rs` 1740–1850 | 完整 media append 接线及邻接处理。 |
| `source/host/app/src/main.rs` 708–1350；`source/host/src/attachment_paths.rs` 1–120 | Pending/sink真实状态、settlement结果处理、ACK/events边界、custody/Drop。 |
| `app/src/tests/media_settlement_unknown_tests.rs` 1–432 | fixture/raw durable image/quiet/两fresh helper/known-refusal/directUnknown与fault入口；433–1035未深读，仅查函数名。 |
| `app/src/tests/channel_host_lane_tests.rs` 400–580、852–910 | 实际 Session fixture/open/read/stop/drop；其余未全文读。 |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` 204–260、330–444、449–560 | fixed names、capture准入/合成qualification、实际执行记录/summary；仅读，未执行。 |
| 原始证据 | 原完整 job log 字节身份核对及相关失败/capture段；ZIP目录、完整receipt结构与62 inputs身份、两失败case相关运行段及整文件字节哈希；其他17组日志正文未逐项语义分析。 |

未验证边界包括：未执行修复或 parser；没有审查尚未物化的 root patch；没有将源码推导当运行结果；未重新取得 API job状态或GitHub签名/下载链；未读 upstream U 全源码，也未作全来源闭包审计；未全文读所有相关模块的其他业务、build/workflow全文、所有 FUA/BR/HTML/image实现；未从17个其他case的receipt accepted断言其全语义正确；未执行 App/Windows/macOS/live服务/设备/打包；未完成独立实现验收。上述边界不影响定位这8个具体失败，但阻止扩展结论为 MSU 全面通过或迁移已完成。
