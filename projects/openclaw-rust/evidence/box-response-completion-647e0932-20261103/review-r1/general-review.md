# BXR-v1.1 独立盲审报告 — general / security / lifetime — round 1

## 审阅结论

**阻塞：1 项 P2 finding，BXR-GEN-001。** 已知 ShellResult 元数据的早先畸形值可被稍后的合法重复字段覆盖，随后完整的 close/end 被接受为成功。这违反本批次 BW07 的完整消息与畸形字段拒绝要求。建议先修复并补充受控线缆反例，再对新的不可变候选重新审阅和执行最终 exact-H Actions。

这是源码审阅结论，不是执行结果。本审阅没有编译、运行测试、运行 parser / validator / generator，也没有执行任何本机或 htch 命令；候选尚未提供 Actions 执行证据。未发现的风险不等于已验证不存在。本报告不批准合并、平台接受、外部 Box 接受或上游迁移全量接受。

## 不可变输入与独立性

| 项目 | 固定值 |
|---|---|
| Repository | bhrumom/fabushi-desktop |
| Branch | codex/openclaw-rust-fusion-chat-20261101-writer |
| Base / starting HEAD | b35408891546694606ed0750c60af0302669aac9 |
| Base tree | 158826d8a7483ad0c7182052419a48c5fb4cc210 |
| Candidate commit | 19baca121c79f99a4955d5a0307afc4eb175601b |
| Candidate tree | 6037d903de1abe954db973d0da92ba1aed4e8502 |
| Scope manifest blob | 3e31be5f2e5cddc333f80f1b49071c8eaaf2ec21 |
| Complete 18-path diff blob | 37869b1b111db59a70a4dfc0ecd18aaaf651f5c0 |
| BXR-v1.1 specification blob | 49acb2114574739690b0e3eeef48f6ae98c9b271 |
| Fixed OpenClaw U | 1440bb1771372334bea23d7642bb52deb7db1845 |
| Reviewer | /root/bxr_r1_general |
| Model / effort | 继承当前 GPT-6 Astra Pro 配置；xhigh（启动任务明确记录实际已选择）；本审阅未切换模型或降低 effort |

已先读 exact-base 根 AGENTS.md，再读 scope、完整 BXR specification、完整 249,511 字符差异。候选 commit/tree/parent 通过 GitHub Git API 读取；后续源码以候选 tree 对应的 immutable blob 为准。未读取另一 reviewer 的报告，未读取既有 runner 裁决，未用其他候选或移动 U。未修改候选源码、spec、共享文件、tree、commit、ref。唯一允许写入的是本报告的不可变 Git blob。

项目 SOURCE_OF_TRUTH、module-map 及 OCRM 主 spec 用作定向规则/所有者背景；其中与本任务相关的执行约束和 M10/M11/M23 边界已读，不声称完整复审其中全部历史内容。

## 阻塞 finding

### BXR-GEN-001 — P2 — 在逐字段验证前覆盖重复值，使早先畸形的已知元数据逃逸

**位置：**

- [generated_production.rs, decode_string_field / decode_bytes_field, L1070–1095](https://github.com/bhrumom/fabushi-desktop/blob/19baca121c79f99a4955d5a0307afc4eb175601b/source/host/src/box/generated_production.rs#L1070-L1095)
- [同文件 decode_message_oneof, L1127–1153](https://github.com/bhrumom/fabushi-desktop/blob/19baca121c79f99a4955d5a0307afc4eb175601b/source/host/src/box/generated_production.rs#L1127-L1153)
- [同文件 decode_shell_result, L1229–1242](https://github.com/bhrumom/fabushi-desktop/blob/19baca121c79f99a4955d5a0307afc4eb175601b/source/host/src/box/generated_production.rs#L1229-L1242)
- 源码 blob：7b1f50c27b30870c7a59d41774decc660e454f71。
- 冻结 schema 将 ShellResult 103 声明为 STRING terminals_folder、101 声明为 message sandbox_policy：[shell_exec_pb.ts, L703–713](https://github.com/bhrumom/fabushi-desktop/blob/19baca121c79f99a4955d5a0307afc4eb175601b/source/packages/proto/generated/agent/v1/shell_exec_pb.ts#L703-L713)。
- 约束：[BXR BW07, L114–123](https://github.com/bhrumom/fabushi-desktop/blob/19baca121c79f99a4955d5a0307afc4eb175601b/docs/specs/g11-box-browser-wire-completion.md#L114-L123)。

**具体失败场景（静态构造，未执行）：**

请求 id=0、exec_id 为空，服务端返回一个 ShellResult，之后给出正确匹配的 StreamClose 和唯一合法 Connect end，HTTP framing 也完整。ShellResult 的完整内层 bytes 为：

```text
0a 00  ba 06 01 ff  ba 06 00
```

含义依次为：

1. field 1：长度 0 的 ShellSuccess（合法 proto3 默认 success，exit_code=0）；
2. 已知 field 103 terminals_folder：长度 1、内容 ff，是非法 UTF-8；
3. 同一个 field 103：长度 0 的合法空字符串。

decode_message_oneof 只校验 metadata 的 wire type 和外层长度，并把字符串内容作为 opaque bytes 跳过。decode_bytes_field 遍历至末尾，每次遇到 field 103 都覆盖 value，最后只返回第三段空值。decode_string_field 的 String::from_utf8 因此仅验证最后一个值。早先 ff 从未做已知字符串验证。decode_shell_result 随后返回 ShellSuccess；manager 的身份/close 和 Connect end 检查均可正常完成；execute_shell_result 得到普通成功，新的 unknown/custody 路径不会被触发。

同一因果还影响已知嵌套消息 metadata 101：内层 bytes

```text
0a 00  aa 06 01 00  aa 06 00
```

先给 sandbox_policy 一个含非法 field-key 0 的消息，再给空 policy。当前 decode_bytes_field 只保留最后一个 policy，L1237 的 validate_protobuf_message 只验证空消息，早先畸形嵌套消息同样被遮蔽。这个例子不依赖对 UTF-8 重复字段合并语义的争议：先前嵌套消息本身含非法 protobuf tag。

**影响与严重度：**

该批次明确要求完整扫描并拒绝畸形字段。这里完整 close/end 并不能证明响应的已知字段合法，调用方会把畸形响应作为确定的成功结果。问题需要畸形 peer 响应才触发，因此标 P2；不是声称当前合法 daemon 会主动产生上述 bytes，也不是声称绕过认证或权限。它直接缺失 BW07 的拒绝条件，因此是本批次接受的阻塞项。

**最小修复：**

在现有解码 owner 内，对每次出现的已知字符串立即进行 UTF-8 验证，对每次出现的已知嵌套 metadata 立即进行消息验证，再保留该类型允许的最终值。不要把“最终值合法”当作“所有已消费字段合法”。继续保持现有 identity 的矛盾重复拒绝和 oneof 的多 payload 拒绝；无需新建 codec/runtime，也无需一律禁止本来允许的合法重复 scalar metadata。

在现有受控 TCP fixture 增加上述两种“坏值在前、合法同字段在后”的完整响应，分别配完整匹配 close/end。断言实际 production accessor 返回 outcome_unknown，保留一次发送、零重试；可附合法重复 metadata 正向控制，避免修复退化成无差别拒绝。现有 negative cases 覆盖单个畸形 metadata、错误 wire、identity、oneof 和 framing，但未见这个被后值遮蔽的顺序反例。修复前后预期仅为源码推导，必须由获准的 Actions 实测。

## 实读范围

以下 18 个路径的完整变更均已通读。核心实现还读了候选完整文件/相邻上下文；大体量 workflow/capture 与既有测试，完整改动已读，未变化内容只按相关 owner/caller 定向补读。

| 路径 | 本轮覆盖 |
|---|---|
| .github/workflows/rust-desktop-runtime.yml | 完整 diff；Linux job、固定 HEAD、cache、capture 条件/超时、前置与最终 artifact 上传 |
| docs/specs/g11-box-browser-wire-completion.md | 完整文档和完整 diff，重点 BW07–BW12、执行与不接受范围 |
| projects/openclaw-rust/evidence/box-response-completion-647e0932-20261103/source-contract.json | 完整新增文件；owner、source binding、命名测试与 acceptance 边界 |
| projects/openclaw-rust/parity-ledger.json | 完整 diff，唯一相关条目窄改及未接受状态 |
| projects/openclaw-rust/requirements-traceability-matrix.json | 完整 diff，相关 R23/AC23 证据与 oracle 窄改 |
| projects/openclaw-rust/scripts/capture_portable_contracts.py | 完整 diff；before/after 源码绑定、full Host check、daemon build/lock/binary、所有新增 case、命名计数 oracle、receipt acceptance |
| source/host/src/box/box_remote_accessor.rs | 完整候选 owner；泛型 envelope/error、ID 分配、所有 data/control 分支、private exhaustion 测试 |
| source/host/src/box/generated_production.rs | 完整改动及核心全链路：类型/转换、所有 accessor、protobuf 编解码、HTTP/Connect、第一写入边界 |
| source/host/src/extensions/forever_box/runner_tools.rs | production 路径完整读取；所有 delegate、连接/cache/lease/navigation、终端 watch；private Computer fixture 完整改动 |
| source/host/src/runner/box_tool_access.rs | 完整候选文件；真实 Runner bridge、入参/权限先验、upstream 一次调用和 spill 返回 |
| source/host/src/runner/large_output_spill.rs | 完整候选文件；一次 Write、已知拒绝 inline、unknown 原样传播 |
| source/host/src/runner/runner_prompt_glue.rs | 完整候选文件；Result API 与 enabled/disabled 转发 |
| source/host/tests/box_exec_daemon_process_contract.rs | 完整新增正向 fixture 及进程 owner 上下文 |
| source/host/tests/box_generated_production_contract.rs | 完整 diff 与既有 fixture/主要正向上下文；全部新增 identity/framing/result/unknown cases |
| source/host/tests/box_remote_accessor_contract.rs | 完整 diff 与旧正向泛型 fixture |
| source/host/tests/large_output_spill_contract.rs | 完整候选文件 |
| source/host/tests/runner_box_tool_composition_contract.rs | 完整 diff 与既有 bridge/admission fixture |
| source/host/tests/runner_prompt_glue_contract.rs | 完整 diff 与旧 spill 调用上下文 |

额外按候选 blob 读取、用于判断实际接线和未改动能力的相邻来源：

- source/host/src/runner/tools/mcp_meta_tools.rs：execute_with_timeout_ms_tracked 的实际 admission、unknown mark、receipt completion、timeout/discard 分支。
- source/host/src/runner/production_turn_run_shell_adapter.rs：RoutedProviderAttemptControl 的 current/check/admit/custody 部分。
- source/host/tests/runner_effect_custody_contract.rs：真实 guard/checkpoint/replay/late completion controls，尤其 downstream_unknown_outcomes_block_responses_checkpoint_and_replay_after_local_return 与 production_generated_runner_preserves_terminal_tool_custody；仅源码，不是 pass 记录。
- source/host/src/runner/remote_box_resources.rs、remote_box_resources_contract.rs、sand_action_audit.rs：既有生命周期 finally-probe 与错误保留。
- source/host/src/box/exec_daemon_process.rs、production.rs、protected_path_guard.rs、box_windows.rs 的相邻 owner；box_file_transfer.rs 完整读取及 loopback_sand_box.rs 上传路径、其他 transfer/ForeverBox owner 定向上下文。
- source/host/src/box/box_mcp.rs、source/host/src/extensions/mcp/box_mcp_exec.rs、box_mcp_exec_contract.rs：相关泛型和错误接口边界。
- source/packages/proto/generated/agent/v1/exec_pb.ts、exec_service_pb.ts、shell_exec_pb.ts：本批次涉及的冻结 envelope/metadata/oneof schema。
- source/box-exec-daemon 的 Cargo.toml、main.rs，以及 daemon.rs 的真实消息定义、execute_with_emitter、HTTP auth/route、framed response writer；daemon.rs blob a40dc3b5f78e414c3ac302137668f1d5be8dc73f。
- source/host/Cargo.toml 和 build.rs 的相邻 generated-binding 构建部分；没有执行 build.rs。

没有做全仓所有未变更模块的穷尽复审或全符号索引；静态兼容判断限于上述实际读取来源。此限制不由“full diff 已读”消除；最终 Host check --tests gate 才能提供编译证据。

## 其他审阅判断（非新的阻塞 findings）

### 身份、终结、错误分层与已有能力

除 BXR-GEN-001 外，本轮源码证据没有形成第二个独立阻塞项：

- manager 仍是每 accessor 的单一 ID owner；uint32 范围检查在 serialize/transport 之前；默认 0/空 identity 与非零 identity 都有显式路径；data、heartbeat、close、Throw 关联检查以及 close 后消息拒绝位于同一个 owner。没有发现新 nonce、隐式 ID 重用、失败重发或另一套 allocator。
- control Throw 的 error/stack/code 保留。manager 无 data 但有 close 可以返回空 Vec，但单次操作 accessor 随后拒绝 missing result；未将这一层分工误报为成功空结果。
- 全消息 oneof 扫描、raw response tag 白名单、ShellResult metadata 区分、single-result 类型与数量、stream 有序多数据路径在源码中接线。没有要求该批次猜测或扩展被冻结的 raw-resource tag 映射。
- HTTP chunk/CL/Connect end 的检查包括截断、重复终结、结束后数据、非法 flags、冲突 framing 与 checked length arithmetic；保留有效 chunk extensions/trailers。所有第一 write attempt 之后的 I/O/status/HTTP/Connect/decode failure 统一包一次 ExecutionOutcomeUnknown，并保留 cause。URL/header/connect/setup 失败仍在该边界之前。unary 15 秒/Exec 120 秒的已有区分保留。
- ProductionBoxExecError 将 post-write transport、protocol、Throw、InvalidResult、Missing 分成 unknown；ID exhaustion、protected path、NoMonitor 与 pre-write transport 保持非 unknown。没有新重试 owner。

### ForeverBox、实际 Runner 和 MCP spill custody

- direct Shell、timed/background Shell、Read、Write、ComputerUse 使用同一 project_box_exec_error 保留 ToolOutcomeUnknown。缺少 timed terminal、无效 background id 在安装 watch 之前返回 unknown。已有 finally-navigation probe 仍是独立观测，生命周期 owner 保留原 delegate 错误，不以该观测替换结果。
- 实际 RunnerBoxToolBridge 对 upstream 只调用一次；large_output_spill 与 prompt glue 的返回类型变为 Result；unknown Write 不返回 inline success、不添加 outputLocation、不做第二次 spill 或清理。已知 refusal 的原始 inline 分支保留。
- 既有 execute_with_timeout_ms_tracked 在 receipt.complete 之前标记 DownstreamOutcomeUnknown。RoutedProviderAttemptControl::check/admit 和既有 custody tests 的源码覆盖了 unknown 后禁止 checkpoint/replay/new admission 的 owner。没有将 receipt 的本地完成误读成远端 effect 已确定完成。
- 上述结论是源码因果分析。新增 Box malformed-response fixtures、spill/bridge tests 与既有 custody controls 仍需在同一最终输入上执行；不能从“测试存在”推导运行通过。

### 实际 daemon producer 和 CI/oracle

- actual daemon 的 execute_with_emitter 回传请求 id/exec_id，并发出匹配 close；writer 在未 abort 的成功 framing 中追加唯一 Connect end 和 HTTP zero chunk。实际 producer 的合法默认 0 与后续非零 id 与本批次新 reader 的正向路径吻合。
- 新 ignored Unix 正向 fixture 使用 shipping supervisor 启动当前构建的 daemon；通过同一 accessor 连续 Shell、Write、Read，并覆盖 binary bytes 与 text result。它不是同一个手写 fake codec 自证。读取到 owned process RAII/close 与工作目录的清理，不声称 socket cancellation 或所有 OS 行为已接受。
- capture 增加 full Host cargo check --locked --tests、当前 daemon build、6 个 integration target 与 2 个私有 lib filter，保留既有 Runner custody 回归组。source 与 producer before/after binding、命名结果/计数/exit/log receipt 以及失败 build 时拒绝复用旧 daemon 的分支已读。
- 当前 daemon 没有 tracked Cargo.lock。capture 保存实际生成 lock 的完整输出，并记录 lock/binary before/after 哈希；这不能冒称构建输入已经锁定。后续 source-freeze/依赖闭包接受仍然未完成，此点是已声明的后续门槛，未重复列为新源码 finding。
- workflow 的对应 Linux capture 条件可覆盖指定 branch push / ocrm-config dispatch，artifact 步骤保存证据目录；这只是可执行接线的静态检查。本候选没有据此得到任何“已通过 Actions”的结论。

## 明确未接受与下一步

仍未建立：最终 exact-H Actions 的编译与实际测试结果、固定生成 daemon lock 后的依赖输入接受、真实外部 Box/device/platform、raw stdout fidelity/完整 reader EOF/输出 cap、socket cancellation、generated origin 的 durable provenance、迁移持久化或全项目接受。坏的历史 cancellation oracle 未作为本轮证据，也没有用它支持正向结论。

修复 BXR-GEN-001 后，应生成新的完整不可变候选和全范围 diff，以相同固定 U 做下一轮独立审阅，并把所有新增/受影响测试及真实 producer gate 绑定到最终候选 H。当前报告只适用于上表 commit/tree，不自动转移到后续修改。

报告结束。