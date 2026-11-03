# Cron job identity — independent general / correctness / security / lifetime / oracle review

## Verdict

**SOURCE-ONLY APPROVE**

本轮没有发现能够以具体触发场景和源码因果链成立的阻断缺陷。该结论仅批准下述 immutable candidate 的源码变化；它不是编译通过、测试通过、CI 接受、批次 verified 或最终产品 acceptance。候选没有执行结果，本 reviewer 没有运行任何编译、测试、lint、formatter、parser、generator、validator 或 harness。

## Subject and review integrity

| 项目 | 精确身份 / 状态 |
| --- | --- |
| Repository | `/workspace/scratch/425c55809bda/fabushi-desktop`，`bhrumom/fabushi-desktop` |
| Canonical branch | `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Frozen base commit | `972779ecd290b90e407ffe651aa5c456c31e9f7f` |
| Frozen base tree | `a75bb4706831a34b818e459f6be4df920560a2fe` |
| Reviewed candidate tree | **`f5e577cf9e119f417ce0cf071f03c214c5bf6190`** |
| Fixed upstream U | `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Shared contract | `OCRM-CRON-JOB-IDENTITY-v1` |
| Model | 保留配置模型；会话声明为 GPT-6 Astra Pro；controller 确认 launch 未提供 model override，本 worker 未更换模型或启动外部 AI CLI |
| Effort | `xhigh`；controller 已确认实际 spawn 参数为 `fork_turns="none", reasoning_effort="xhigh"`。选择理由为跨 Agent 身份、权限、取消 / 未结算结果、真实存储与测试 oracle 的相互作用。此为 controller-confirmed launch setting；worker 没有独立的运行时 effort 自检接口 |
| Blindness | 未读取其他 reviewer 的报告、综合发现、runner 裁决或前轮裁决 |
| Execution | Source / Git / exact-ref API reads only；没有本机或 htch-runtime 可执行验证 |

审查使用本轮 `manifest.json` 和 `complete-scope.patch`。对候选源码均采用 `GIT_NO_LAZY_FETCH=1 git show f5e577cf9e119f417ce0cf071f03c214c5bf6190:<path>`；未以 mutable HEAD、工作树或 manifest 中列出的可变 local 文件作为候选。只读 Git tree diff 确认 base tree 到 candidate 恰为下列七条路径；`git ls-tree` 取得的候选 blob 与 manifest 一致。base commit object 的本地可用性不是本结论依据，差异依据精确 base tree。未执行 lazy fetch。

| 变化路径 | Candidate blob |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `8f924c42928fa61242a2f0129ee20f9549a5c92f` |
| `projects/openclaw-rust/docs/specs/cron-job-identity.md` | `988f36d855e3efffc5b5563006dddf926393fcbe` |
| `projects/openclaw-rust/evidence/cron-job-identity-647e0932-20261103/source-contract.json` | `79b752dbc6aab3d4e2fd9612aa5aae89f5cbd24f` |
| `projects/openclaw-rust/parity-ledger.json` | `379557210e62d2c380daba068e6e8c774f051187` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `3605dc7310eb4dfd06385b6aae0612a2b560d576` |
| `source/host/app/src/main.rs` | `5e1a392d615bec42153e9936f7dc23db8954cd81` |
| `source/host/app/src/tests/cron_job_identity_tests.rs` | `646d24ab3d27dede7bfcb4f5fe421585694c390b` |

## Findings

**Blocking findings: none.**

没有提出仅凭风格、命名偏好、抽象层数或未证实猜测构成的 finding。下面是支持源码结论的具体核对结果与范围限制，不是执行成功证据。

### Identity and compatibility

`main.rs::list_openclaw_cron_jobs`（2840–2906）保留原 list projection 和分页处理，仅将两项公开身份改为现有 `stable_automation_id(agent.id, r.id)`。`automation_id.rs` 的 mapper 原样使用 durable Agent/local pair，且输出 UUID version 5；同一 mapper 仍被既有运行 telemetry 使用（`main.rs` 16368–16374）。没有另一份 ID 算法、持久 alias map 或 scheduler。

`main.rs::trigger_openclaw_cron_job`（2908–2993）保留原 `id` 优先于 `jobId` 的输入规则，先从真实 Session owner 枚举，再以 canonical/local 两个名称空间的并集选一个 pair。单条 record 即使同时匹配两个名称空间也只处理一次；遇到第二个匹配 pair 时返回 `BadRequest`，在此之前没有调用 `run_now`。没有 first-match dispatch，也没有使用调用方的 `agentId` 重定向或缩小歧义集合（2922–2946）。枚举返回的错误沿 `?` 传播。

UUID v5 保留规则（2948–2956）位于调度之前。原 canonical owner 删除后，即使该字符串仍是另一条 routine 的唯一 local alias，也会拒绝；另一条 routine 仍能通过自身 canonical list ID 执行。该行为符合规格的预先决策 `CJI-ID-NAMESPACE-01`。它有意收紧该名称空间的旧 alias，并非声称保留全部旧 UUID local addressing。

最终 invocation 明确传 resolved `agentId` 与 **local** `automationId`，before/after history 观察也使用这个 pair；公开响应的 `run.jobId` 才使用 canonical ID（2958–2984）。`FnOnce` port 和单个调用点没有重试分支。

### Production ownership and permission chain

已核对实际链路：`host_gateway_api.rs` 的现有注册与 Automations owner 映射 → `gateway_server.rs` HTTP command admission → `UnifiedGatewayApi::call_with_context` / `call` → `call_openclaw_gateway_command` → 两个薄方法 → 被测试的同文件生产函数。

新方法的唯一执行 port 仍是 `self.call("runAgentAutomationNow", run_args)`（`main.rs` 5296–5303）。既有 native dispatcher 在 13957–14011 根据收到的 Agent/local pair 读取定义，并选择 configured routed Runner 或既有 Host lane；没有新 execution owner。测试所调用的 `run_agent_automation_now_with` 正是 routed production 分支使用的 Runtime 接口。

Gateway 原有鉴权、node/operator 限制和 mutation scope 检查保持在这条调用链之前：`gateway_server.rs` 1018–1062 将 `cron.trigger` 标记为 mutation、将两个 cron 方法标记为 operator methods；1752–1806、1893–1938、1998–2041 保留现有 bearer/role/scope admission 与 dispatch。新增解析器本身既不读取 token，也不创建授权或把 `agentId` 当作 grant。该核对证明本批没有修改既有权限合同；不声称已经实现 fixed-U 的全部 per-job grant / visibility 合同。

### Outcomes, cancellation and lifetime

调度错误在 `run_now(...)?` 原样返回，不继续构造成功响应、不重试、不改选其他 Agent。调用成功后仍按实际新增历史行报告 run ID、status、detail 与 finished，保留旧实现的 before/after projection，而没有根据 port 的 `Ok` 直接伪造 terminal history。

已沿 `AutomationRuntime::run_agent_automation_now_with`（773–819）、`AdmittedAutomationRunStore`（319–385）及 `AutomationRunPath::fire_automation_inner`（484–741）核对实际结果消费：历史在执行前由现有 store 准入；completed/error/interrupted 按现有规则结算；`RetainedUnsettled` 保留 Running 字节、不发表 terminal observation。新 adapter 没有接管取消或未知外部效果。既有 local Runner consumer 的 `attach_automation_run_fences` 也直接保留 `RetainedUnsettled`（`main.rs` 6246–6254）。

Fixture 通过 `create_dir` 独占随机根，native creation dispatcher 建立实际 routine；没有手工创建另一份 job registry。`Owners::drop` 先 dispose Runtime，再 shutdown ProductionSessionWorkers；`Fixture::drop` 在 owners 结束后仅删除其独占 root。重开用 `owners.take()` 结束原 owners 后再建同根 owners。已核对既有 `ProductionSessionWorkers::shutdown_inner` flush / close DB owners / close worker pool 的顺序（6360–6379）。测试执行叶子受控且同步，没有新增脱离 fixture 所有权的 worker、全局环境覆盖或进程名清理。

## Requirement and oracle assessment

下表的“源码符合 / oracle 已编写”不等于运行通过。

| 需求 | 源码与 oracle 核对 | 本轮结论 |
| --- | --- | --- |
| CJI-01 | list 两个 ID 使用既有 stable mapper；same-name test 从 native producer 建两条相同 local ID、不同 Agent / prompt，检查 canonical IDs、分页、history projection | 源码符合；执行 pending |
| CJI-02 | 全部 visible pair 解析后才调度；joint-namespace ambiguity 拒绝；保留安全 unique legacy alias；reserved UUID fallback 拒绝 | 源码符合；对应 legacy / collision oracle 已编写 |
| CJI-03 | exact resolved pair 进入 native command；真实 history 回读；响应 canonical job ID / actual run ID | 源码符合；`trigger_real` 使用实际 run_args 调用真实 Runtime |
| CJI-04 | 缺失 / 空白 / 非字符串 / unknown / ambiguity 在 port 前失败；误导 `agentId` 不改变选择 | 源码符合；测试核对 dispatch count、实际配置和历史字节、observation count |
| CJI-05 | 无 migration / alias state；reopen 同一 pair 派生同 ID；native delete 后旧 canonical 失败；其他 same-name routine 可用 | 源码符合；reopen / delete / survivor controls 已编写 |
| CJI-06 | direct dispatch error 返回；success / executor error / reported failure / interrupted / retained-unsettled 仍由真实 RunPath 产生并观察 | 源码符合；outcome test 有各自历史 / observation oracle |
| CJI-07 | private 同文件函数位于生产编译区；实际 Gateway 方法调用这些函数；port 仍调用原 native command | 源码符合；真实 owner 行为测试与有限 source wiring assertion 边界准确 |

关键 oracle 是有因果性的：`trigger_real`（test 225–287）先断言 adapter 实际给出的 pair，然后从 **该实际参数** 提取 Agent/local ID 进入 Runtime；没有用 expected pair 替换被测输出。执行叶子检查目标 prompt，并从真实文件读 Running history。返回后检查响应 run ID 就是执行前观察到的真实历史身份，随后核对 Runtime history 与 reporter。若恢复旧 first-local-match 算法或把 stable ID 误传成 local ID，反向 listing-order 的同名测试不能保持这些断言。

collision test（515–562）通过 native slug producer 实际建立 canonical/local collision，先检查双匹配拒绝，再删除原 canonical target，确认不能降级成剩余 legacy alias，最后验证 surviving routine 的 canonical ID 可用。reopen test（565–606）实际关闭 / 重开所有者，并比较原文件字节和稳定公开身份。outcomes test（609–679）检查 unsettled history 保留原 Running 行、无 finished timestamp、无 terminal observation；direct refusal 控制则检查一次 port 调用、原 error 与原文件字节。

测试中的 source-string wiring assertion 明确只作补充，没有冒称实际构造完整 `UnifiedGatewayApi`、HTTP server 或真实 provider。编译、链接、运行时结果仍需 exact-candidate Actions。

## Workflow, ledger and evidence

完整 changed hunks 仅对既有 Linux App check / exact-case receipt 追加本批 contract/scope、三个 source inputs 与五个 required names。没有新增 CI owner，也没有删除或弱化旧 controls。

已读取该 harness 的 source-input binding、`cfg(not(test))` App check、Cargo compiler-artifact executable 选择、executable path/digest、每个 name 的 `--exact --list`、实际运行、非零 exit 拒绝和一通过 / 零失败 / 零忽略 summary 条件（workflow 3950–4320）。receipt 保留 H/tree/U、run/attempt、命令、stdout/stderr 与其 hash；整个目录由既有 artifact upload 保留（4486–4493）。本批五个名称与 test declarations 一致且各追加一次。

既有 AutomationRuntime / RunPath regression targets（1707–1708、3831）和 Gateway registry / server regression targets（1736–1737）仍存在，diff 没有删除它们。实际允许执行入口是现有 canonical-branch push 或 `workflow_dispatch` 的 `ocrm-config` lane；此报告不把一个未运行该 lane 的其他 workflow 成功当作本批证据。

ledger 仅在既有 `OCRM-RESP-GW-DISPATCH-REGISTRY-001` evidence refs 追加三个本批引用；RTM 仅在既有 `OCRM-R04` / `OCRM-AC11` evidence refs 追加同样引用。diff 没有改 source-review rows、历史 read facts、verification / acceptance 状态或全局 flags。`source-contract.json` 明确说明该 registry anchor 不闭合全部 cron responsibilities，且 execution / independent acceptance pending，声明边界准确。

## Read scope and explicit unknowns

### 完整变化阅读

七条路径的全部变化均已审查。完整读取新增 feature spec（246 行）、source-contract（129 行）和测试文件（679 行）；读取 `main.rs` 全部新增生产函数、替换的两方法、test module 接线及必要调用者；读取 workflow 全部 changed hunks 与复用的执行 / receipt / artifact 边界；读取 ledger / RTM 的全部追加 hunks及其 existing-owner 上下文。没有声称逐行深读未变化的整份巨大 `main.rs`、workflow 或全量账本。

### 必要 authority / 相邻源码

- 根 `AGENTS.md`；feature spec 全文；主 Spec 当前 §17（并确认旧 §17 / historical amendment 与新 OCRM-EXEC-02 的优先关系）。
- `SOURCE_OF_TRUTH.md` 的当前身份 / authority / 约束段；`module-map.md` 的 M04 / M19 owner 映射及其非验收含义；`upstream.lock.json` 的 fixed U 与未接受状态。
- `main.rs` 的真实 Gateway dispatch、`call_with_context`、native run/create dispatch、local Automation Runner admission / terminal consumer、stable telemetry consumer。
- `host_gateway_api.rs` cron registry / owner mapping；`gateway_server.rs` HTTP route、authentication、role/scope checks、context dispatch。
- `automation_id.rs` 全部 mapper；`automation_store.rs` 的 local ID / native creation、list / history order、fresh run / delete；`agent_session.rs` 创建、列 Agent / routine、store access、admitted mutation。
- `ProductionSessionWorkers` fixture construction、store resolution、Agent summary enumeration、shutdown；`AutomationRuntime` native CRUD、manual run、admitted store、dispose；`AutomationRunPath` execution result 类型、duplicate custody、admission/history/terminal branches。
- App `Cargo.toml` 的真实 bin、existing UUID dependency 与 test build 边界。

### Fixed-U bounded identity reads

本地 U commit object 不存在时，使用 GitHub exact-ref file API 读取以下 source-contract 指定切片；返回 blob SHA 全部与 candidate source-contract 一致。未使用浮动 main，也未将切片阅读提升为整文件或全局 source closure。

| Fixed-U path | 已读行 | Exact blob |
| --- | --- | --- |
| `src/cron/service/jobs.ts` | 171–230 | `780d8542ad4460b63ab39a4b2096671192853ec4` |
| `src/cron/service/ops-read.ts` | 1–100 | `3fd0daf829edc3c94ed052dd78478c3ce9f1bea0` |
| `src/gateway/server-methods/cron-job-access.ts` | 126–235 | `28a75469df97ceaef4cc137510ee8477ee07038a` |
| `src/gateway/server-methods/cron.ts` | 501–625 | `35cd3e5e36e26ccefba265c4955377fff0bc5ca0` |
| `src/cli/cron-cli/register.cron-simple.ts` | 251–280 | `ed18eba20a47260fba17a9b7b1b75fba372fb524` |

这些切片支持“stored job ID 独立于显示名称、exact read、authorized selected ID 进入 enqueue / history、实际 CLI 传 chosen ID”的限定合同。完整 `cron.run` grants / visibility / queue / wait、scheduler/timezone/reaper、递归依赖与 LFS、所有真实 provider/backend/channel、supported packages/platforms 及最终 G01–G12 / AC01–AC24 没有在本批闭合。按主 Spec §17，这些无关全局未闭合项不构成拒绝本次独立 identity repair 的理由。

既有 fail-soft filesystem list、非事务性的 lookup-to-execute 窗口、同一 pair 的并发 run-result correlation、真实 cancellation / unknown external-effect recovery 的全链路行为不由本批改变或证明；规格已明确这些限制。当前源码和测试没有冒领这些结论。

## Required next evidence boundary

允许进入冻结候选后的既有 GitHub Actions focused verification。只有在候选 H/tree、实际编译链接结果、五个 exact positives、既有相关 regression、完整 raw logs / receipt 与独立证据评估齐备后，才可另行宣称该批 verified。本报告本身只提供 **SOURCE-ONLY APPROVE**。
