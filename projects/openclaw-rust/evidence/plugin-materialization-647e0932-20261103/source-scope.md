# Plugin materialization attempt custody — source / owner / evidence scope

## 1. 结论

本批应锚定既有 **`OCRM-RESP-SKILLS-RUNTIME-LIBRARY-001` / `OCRM-UNIT-SKILLS-001` / `skills.runtime_library`**，作为 Plugin Skills snapshot admission 的 SDK 文件物化边界补强。产品生命周期、权限、快照提交与清理决策仍归 `SandPluginSkillsService`，运行时投影仍归 `FileWorkflowStore`；SDK 的 `DefaultPluginCacheManager` / `loadFromMarketplaceSource` 负责既有文件 I/O 的 attempt custody。

对主集成拟议的最小记账方案，**没有发现具体问题**：在现有 SKILLS responsibility 的 `evidence_refs` 中追加新 spec / 本批 target evidence，并在同 owner 原有八条 RTM 记录中追加 spec；保持 `status`、global flags、`source_refs`、`read` / canonical source-review 状态不变。不得借此升级未生产接线的 `OCRM-RESP-PLUGINS-INSTALL-MANAGEMENT-001` / `OCRM-UNIT-PLUGINS-002`。

这是来源、所有权和验证权威的只读映射结论。**不是待修实现的独立 APPROVE，不是 CI 通过，不是 OpenClaw 插件 Rust 迁入完成声明。** 同 H 的实际 Actions 证据及最终冻结候选审查仍由主集成完成。

## 2. 本报告的身份与方法

| 项目 | 本次边界 |
| --- | --- |
| Fabushi source base | `0e845ce1679e0cdf95346adc54f5d58bf06464e8` |
| 仓库 | `/workspace/scratch/425c55809bda/fabushi-desktop` |
| 主集成告知的远端提交 | `3438eb5bac755c399c9ec686414bbb4c0816888d`；本报告没有把它当作已逐文件重新审阅的 target H |
| 固定 upstream | `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`，tree `90c8aab39972da11cfad220361781d49e1bc6700` |
| 工作类别 | 只读 source / owner / evidence mapping；模型继承父配置，沿用父级 xhigh 范围分类，未调用另一个模型或执行器调整配置 |
| Target 读取 | 使用上述 base 的 Git 对象；不依赖另一个 worker 正在修改的工作树 |
| Upstream 读取 | 本地对象缺失后，GitHub connector 直接按精确 U 读取文件，核对返回 blob 与 canonical inventory 记录；没有访问最新 upstream |
| 实际操作 | Git/source 文本读取、既有 gzip inventory 文本读取、精确源码比较与人工分析 |
| 未进行 | 本机 / htch / Actions 测试、编译、lint、parser、validator、generator，或任何仓库文件修改 |

已完整读过根 `AGENTS.md` 和 base 下 1050 行 `projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md`。大体量生产文件只读下文列出的函数和调用 / 消费边界，不声称逐行读完 `main.rs`、workflow、ledger 或全部 RTM。既有 `/workspace/scratch/425c55809bda/reviews/plugin-materialization-gap.md` 已读；它以 `071aeb6af1b223cceee9d81b5f7df3bad484d5a4` 为调查基线，故其关键缺陷已在本次 `0e845ce…` 的实际实现重新确认。

## 3. 当前生产链与为何归 SKILLS

以下 target 引用均绑定第 2 节 base，行号为该 Git 对象的行号。

| 环节 | 实际入口 / owner / consumer | 对本批的直接意义 |
| --- | --- | --- |
| 生产组合 | `source/host/app/src/main.rs:16564–16588` 调用现有 MCP composition；`source/host/src/host_production_extensions.rs:564–580,1333–1380` 构造 `CoordinatorPluginSkillsLoader`、`SandPluginSkillsService`、`McpHostService` | 复用现存 Skills 生命周期，没有创建插件安装的第二个业务 owner。 |
| Host → Coordinator → Electron | `source/host/src/extensions/mcp/coordinator_relay.rs:85–93,125–190`；`source/node-agent-coordinator/src/main.rs:282–293,1105–1146`；`source/electron-main/coordinator/coordinator-control-server.ts:65–155` | Node Coordinator 的 control waiter 使用 15 秒超时；请求工作在线程 / 独立异步 executor 中运行。仅 waiter 结束不能证明远端文件操作已停止。 |
| 等待取消边界 | `source/node-agent-coordinator/src/control_port_client.rs:46–98`；Host relay 的 timeout 清理在 `coordinator_relay.rs:175–181` | 本地 drop / timeout 退休关联等待；源代码明确不等于取消远端执行。旧 attempt 与后继 materialization 重叠是实际可达边界，不能靠延长 timeout 或新调度器代替文件所有权。 |
| Electron source/materialization bridge | `source/electron-main/mcp/host-plugin-skills-loader.ts:190–331`，尤以 `258–277,303–329` | 每次调用使用相同 `<sandRoot>/plugins/cache` 下的新 `DefaultPluginCacheManager`，调用真实 SDK，明确 `pruneOldVersions:false`；返回真实 `installPath`、canonical plugin identity、listing / auth 信息。 |
| SDK 文件 I/O | `source/packages/cursor-plugins/cursor-marketplace.ts:1–146`；`source/packages/cursor-plugins/loader.ts:105–136` | cache completion 为 `.cache-complete`；当前 loader 第 126 行直接在共享 final dir 安装，错误分支递归删除该 dir。两个独立 cache-manager 实例仍共享相同磁盘路径。 |
| Rust admission / cache / pruning | `source/host/src/extensions/mcp/plugin_skills.rs:323–357,966–1045,1086–1130` | Rust owns complete-listing eligibility 与 commit；成功 `installPath` 的保留独立于 skills 是否为空。pruner 遍历两级 slug/plugin-version 目录，不是任意 SDK 临时空间的 owner。 |
| 实际文件消费者 | `source/host/src/workflows/workflow_store.rs:302–377` | 运行时沿提交记录读取 skill 文件并再次检查拒绝状态。删除已提交 winner 的物理目录会影响真实消费，单纯 cache JSON / admission 正确不能覆盖该 I/O 缺陷。 |

缺陷的具体时序：A 已进入最终目录的安装 I/O；A 的调用方等待结束，但 Electron 操作仍未结束；B 对同 key/version 完成物化并写 completion sentinel，Rust 可接收 B 的返回；A 随后失败，旧 catch 删除共享 final dir。A 的清理权限由“知道路径”推导，而不是由“拥有该 attempt 的对象”推导，所以可能删除 B 的成功文件。上述判断来自 base 的真实 SDK 生产分支与 timeout / executor 生命周期；不是仅凭测试 fixture 推测。

本批足够窄的修复责任是：每个 attempt 只准备、写入和清理自己的临时物化对象；发布发生在完整内容准备之后；失败或落败 attempt 不删除 / 覆写已完成 winner；临时空间不成为 Rust 两级 installed-cache pruner 的误删对象。保持既有 canonical final path、零技能成功路径保留、Host 两处禁用 SDK prune 的语义以及外部调用默认行为。临时目录和原子文件发布是 adapter I/O 机制，不应新建缓存索引或业务状态表。

更广的账号切换、卸载后的迟到成功、权限世代、进程丢失后的未知远端效果属于 Rust admission / lifecycle 交接边界；本批文件 I/O 修复不得声称一并完成这些行为。

## 4. 已有 spec 中的精确责任

引用：`projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md`，base 如上。

| 既有记录 | 位置 | 本批应沿用的约束 |
| --- | --- | --- |
| 既有 owner 分工 | 19–25 | Electron SDK 物化 source identity；Rust reconcile、eligibility、publication、lifecycle 与清理；同一个 service/cache/workflow owner。 |
| `PS-06` | 75–92 | staging 在业务锁之外；publication / dispose 有明确次序；失败清理仅针对 owned staged file；后继提交不能被旧清理误伤。 |
| `PS-07` | 93–98 | Host SDK 的两处 pruning 均禁用；Rust 仅对完整提交 listing 清理；部分失败保留旧文件；不增加另一个版本 GC。 |
| `PS-CACHE-RETENTION-01` | 151–165 | 保留真实成功 `installPath`，包括零 skill；不从名字反推目录，不增加 cache index，不把目录名当权限身份。 |
| 验证边界 | 200–246 | Actions only，真实 SDK/Host owner，shipping Electron esbuild/dependency options；不能把受控 Node contract 描述为整图 strict TypeScript 或真实服务验收。 |
| `PS-LIVE-07` / `PS-LIVE-10` | 374–388 / 413–430 | 同 root 后继与旧清理的寿命边界、锁外 callback；不推导 Electron 远端工作已 quiesce。 |
| `PS-DESKTOP-06` | 779–783 | 调用方 timeout 不取消已 admitted 远端工作，也不授权重放。 |
| recovery `PR-04` / `PR-05` / `PR-06` | 898–928 | owned temp 清理、发布后未知效果、exact ticket / pending 与禁止 replay；保留零 skill reinstall。 |
| source provenance | 545–554,989–998 | 复用精确 U 的既有已读证据；明确 bounded target/source 读取，不把大型文件、递归依赖或 broad parity 全文已读作为推论。 |

此前调查提出的 `PS-MATERIALIZE-CUSTODY-01` 是拟议局部标签，不是本报告宣布新增的第二套账本。主集成可由本批新 spec 承接该 I/O 行为并链接上述现有责任。

## 5. 现有账本与 source-review 权威不可混同

### 5.1 现有 responsibility 与 RTM

`projects/openclaw-rust/parity-ledger.json:32914–33093` 是本批责任记录。其 `owner_decision` 在 33003，`unit_id` 在 33092；已包含 snapshot-admission spec、`snapshot-skills-scoped.md`、两个 SDK/Host Node tests、Rust plugin/workflow/sync 等证据。状态在 33061–33067 明确为：

- `implementation=implementing`、`mapping=mapped`、`read=read-complete`；
- `wiring=not-wired`、`verification=not-run`、`acceptance=pending`。

这是 broad responsibility 状态；不能因为本批修复一个真实生产 I/O 缺陷而提升整条责任。其已有 requirement IDs 是 `OCRM-R04`、`OCRM-R08`、`OCRM-R09`、`OCRM-R21`、`OCRM-R24`、`OCRM-AC08`、`OCRM-AC21`、`OCRM-AC24`。在 `requirements-traceability-matrix.json` 中，这八条记录的 ID 分别位于 base 的 171、474、645、1272、1502、1957、2722、2978 行；它们已引用同一 PS spec/dossier。沿这八条原记录补充新 spec，不新建一套 requirement，是适当的最小更新。

`parity-ledger.json:31222–31423` 的 `OCRM-RESP-PLUGINS-INSTALL-MANAGEMENT-001` 指向 `InstalledPluginIndexStore`。31284–31287 明确当前没有生产 installed-index producer/consumer，只有 contract fixture 和 bundled 常量引用；blocker 是 `BL-PLUGINS-INSTALL-WIRING-001`，状态仍 `not-wired/not-run/pending`。`dossiers/plugins-runtime-scoped.md:30–40` 同样记录此边界。SDK materialization 不应挂到该表来声称完成安装管理接线。

### 5.2 历史 dossier 声明与 canonical rows 的真实差异

`dossiers/snapshot-skills-scoped.md:138–166` 记录前一批作者从固定 U 直接取得并完整读取六个文件：

| 历史 fixed-U 已读声明中的路径 | 精确 blob |
| --- | --- |
| `src/skills/loading/plugin-skills.ts` | `19fcb97b484bbe4b199324c69cc86eef4d3235a4` |
| `src/plugins/plugin-root-contributions.ts` | `614b703b81889b048f43efd67d260e398cf86131` |
| `src/plugins/config-activation-shared.ts` | `f1d6c806adab30da1f194b90482b056953803c38` |
| `src/plugins/uninstall.ts` | `8a9015d2de7bbc0dda157a7113696614a2fb796f` |
| `src/plugins/uninstall-package-plan.ts` | `661aebe2a3846f2d2500f6f4ce87a1eb69c4c3b4` |
| `src/plugins/uninstall-package-config.ts` | `e99f0490eb8f5a8b554138854fe3537e09bef807` |

本 worker 读取了这份历史记录，**没有在本次把六个 upstream 实现逐个重新全文阅读**。其适用结论是 activation 与 filesystem publication 分离、canonical plugin identity、stale entry retirement、package owner 与 runtime child 清理分离；该 dossier 自己也明确没有闭合所有 slot/package/npm/hardlink/uninstall 或递归 helper。

对应 canonical inventory 的 `part-0046.jsonl.gz`、`part-0047.jsonl.gz`、`part-0049.jsonl.gz` 行仍为 `read_status=unread`、`required_review_depth=deep`、`review_status=pending`，且 `read_ranges=[]`、`evidence_refs=[]`。其中 `plugin-skills.ts` 已映射 SKILLS，其他若干仍是 unclassified。这是**历史 narrative 已读证据未物化到机器 source-review 的差异**，不是可自动修正为 deep/read 的事实。本报告原样保留，不回填、不升 authority、不借本批新增 evidence_refs 消除它。

另一个不能混淆的身份：dossier 历史 §7 / ledger `historical_source_refs` 的 `workspace-skill-loader.ts` blob 是 `6368d346df09b8ff3b31d5b745f89b9b7ab17fe9`；本次固定 U canonical row 是 `f1855ce5eb0d9c558757bacd0eefc773ed5ea969`。历史 508 行已读不能转记为固定 U 的已读。

`inventory/index.json` 仍保留 `latest_exact_head_evidence=null`、source closure / semantic read / dependency closure 等未闭合状态；canonical manifest identity 是 `64b4e062c0f1cbc34c9383f2bf957eaa4bf1c55bf12ceaa9d88c58a0398859e4`。本次只读工作不改变任何这些状态。

## 6. 本次直接读取的最相关固定 U 实现

以下七个完整文件均通过 GitHub connector 以 `ref=1440bb1771372334bea23d7642bb52deb7db1845` 取得，返回 SHA 与 base 所携 canonical inventory 对应行一致。表中“读取范围”只说明本 worker 实际阅读的源文本，**不代表已运行、已完成递归依赖审查或 canonical deep review 已物化**。

| 固定 U 路径 | 返回 blob | 实际读取范围 | 与本批直接相关的实现 |
| --- | --- | --- | --- |
| [`src/plugins/install-shared.ts`](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/plugins/install-shared.ts) | `6549ffb326548bbc23c723ddba925bafb14a54ed` | 1–498，全文件 | `installPluginDirectoryIntoExtensions` 在 339 起处理真实安装调用；约 406–470 构造 package-dir 参数、在 staged bytes 上执行 after-install / consent binding，把 deferred owner / settlement request 传给 runtime，并回传 package transaction。 |
| [`src/plugins/install.runtime.ts`](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/plugins/install.runtime.ts) | `75f233a9245fed7d7a73a21c0e709252edf2e6c0` | 1–29，全文件 | 第 9 行直接 re-export `../infra/install-package-dir.js::installPackageDir`，确认实际 caller → implementation，不以同名路径猜测。 |
| [`src/infra/install-package-dir.ts`](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/install-package-dir.ts) | `8b66768a0171af8779ae3181503549505c0d3bd9` | 1–758，全文件 | 137–203 deferred transaction；235–261 sticky ownership / settlement refusal；293–342 captured directory identity 与受检查的 removal；343–511 publication receipts、回滚隔离与恢复；513–610 独有 stage / validation；612–688 backup 与发布；690–758 commit / rollback settlement。 |
| [`src/infra/package-update-filesystem.ts`](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/package-update-filesystem.ts) | `84d0b7476c15e9b7d2febb829ca2dd6e761c2903` | 1–760，全文件 | `movePackageRoot` 从 124 行起：把 owner checks 与 publication receipt 转给 fs-safe atomic；Windows allow-hardlinks 分支使用受 authority / path identity 检查的 rename retries；调用方不能把未知发布当作没有效果。 |
| [`src/plugins/install-transaction.ts`](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/plugins/install-transaction.ts) | `fd4c96164d7713d9f8b50c3bc58854de13182a1a` | 1–283，全文件 | 107–217 的 transaction wrapper 区分 publication authority、settlement 与外部 deferred custody；244–283 共用 in-flight settlement、反序 rollback、失败重试保留进度。发布索引后的清理失败不能反向删除已提交 package。 |
| [`src/plugins/plugin-cache.ts`](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/plugins/plugin-cache.ts) | `ed175464e52c7725bf97eb02e4b472bd900349e1` | 1–550，全文件 | 257–298 pending fact 只在仍是同一 promise 时替换 / 删除；356–455 retirement 关闭 admission、abort、等待 retained work / cleanup，未完成资源不被提前释放。它是运行期 metadata/resource cache owner，不是目录安装发布器。 |
| [`src/plugins/plugin-cache-files.ts`](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/plugins/plugin-cache-files.ts) | `afe7990dc87e3c5e661eca2cf3698f13bf14d020` | 1–470，全文件 | metadata / stat / path identity 缓存、safe-open 与 descriptor 收尾；可帮助理解缓存事实身份，但不能用来证明文件安装发布或 winner-preserving cleanup。 |

最有用的 U 事实是 `install-package-dir.ts` 在 519 行以 `mkdtemp` 创建独立 stage；其发布成功保留 path/dev/ino receipt（676–685）。回滚在递归删除前，重新检查 owner、base 与 installed object identity，把 exact object 原子移到私有 quarantine（355–388）；后续清理只对该已捕获对象负责。失败准备的清理针对 `stageDir`（416–418），而不是盲删一个共享 final path。这个来源支持本批“准备 / 发布 / 清理必须具有各自 custody”的设计约束。

这并不要求 Fabushi SDK 引入 upstream 的 transaction registry、cohort recovery 或另一套安装状态。修复现存薄桥 I/O 的范围内可吸收该约束，而 Rust 权限 / admission owner、现有 cache 数据格式及公开 API 不应转移。

上述七条 canonical inventory 行仍全部为 `unread/deep/pending`，`responsibility_status=unclassified`、`read_ranges=[]`、`evidence_refs=[]`。infra 两项在 `part-0042.jsonl.gz`，plugin 五项在 `part-0046.jsonl.gz`。本报告增加本次真实读取的证据，不执行 source-review materialization，也不将整个 install / cache 家族批量映射到 SKILLS。

## 7. 明确保留的读取与接受缺口

本次最相关 U 实现通过 connector 已可直接取得，因此不能再把这些七个文件称为“本地缺失而未知”。仍未闭合的是：

- `@openclaw/fs-safe/atomic::movePathWithCopyFallback`、`MovePathPublicationReceipt` 的底层原子移动实现及平台特性：只读到本次七个文件中的 import、参数与调用，未读其实现。不能从 caller 的命名推导跨文件系统 copy fallback、symlink/hardlink、所有平台 no-clobber 的完整保证。
- `src/infra/install-safe-path.ts`（canonical blob `ffc77064f450028ea0dd94be80dd494092dea692`）、`src/infra/mutation-authority.ts`（`282e5fc6bb7360c007d3f7534e2c31c54abec7ae`）的实际实现未在本次读取；仅确认 exact identities 和 import / use sites。fs-safe remove、cohort lease/recovery、dependency install execution 等递归实现同样未闭合。
- Git/npm/source-plan / batch-install 生产入口及其全部协议没有在本次重新端到端审查；已读 `installPluginDirectoryIntoExtensions` 到 package-dir 的具体调用边界，不能扩展为所有安装入口审查完成。
- 新 target 生产修复及两个新测试尚未作为最终 frozen candidate 交给本 worker 评审；本报告没有检查它们的新分支或新 named-test 逐条实现。
- 未执行任何平台、真实 marketplace、同进程 / 跨进程并发、包装运行或 fault-injection 验证。源码分析不代替这些证据；也不以本批需要为理由无限扩张全局依赖闭包。

## 8. 对本批最小证据更新的建议

| 现有位置 / 执行 owner | 最小更新 | 保留的边界 |
| --- | --- | --- |
| 本批新 spec | 说明上述可达 A/B 时序、owned staging / winner / cleanup 要求、返回路径与 pruning 兼容性；记录 U 精确源路径与 blob 的适用 slice，区分文件 I/O 修复和 Rust admission / 产品接受。 | 不宣称所有 OpenClaw 安装 / 缓存 / 权限行为完成，不新建第二账本。 |
| `parity-ledger.json` 的 SKILLS 记录 | 只追加新 spec 与本批 target evidence refs；如 test refs 已存在，避免重复。 | 不改 source_refs/read/status/global flags，不动未接线 INSTALL-MANAGEMENT 行，不把 helper 目录整体记成已映射。 |
| 原有八条 RTM 记录 | 将新 spec 链接到既有同 owner 责任路径。 | 不新增重复 requirement，不将该单批 pass 推升 broad requirement completion。 |
| 现有 renderer / Electron Node contract gate | 主集成提出的 8 个新 named positives 进入现有 gate；使用实际 shipping build 产物和真实 SDK / Host loader。 | 无须新建 workflow；每个预定 positive 必须实际执行并有 pass 证据，不能仅凭 TAP 总计。 |
| 同 H receipt | 绑定最终 H / U、workflow 与命令身份、实际 selector、完整 TAP / exit、两个生产 SDK 文件、两个 test、新 spec 的输入身份。保留现有 receipt 对其它输入的绑定。 | 旧 H green、父分支 green、仅 job conclusion 或测试命令文本不证明新 H 行为。 |
| 既有 Rust 相邻 controls | 保留 snapshot eligibility、complete-listing vs failure、零 skill installPath retention、dispose / successor cleanup、uninstall / denied admission 的既有选择器。 | 不为这一 JS I/O 改动泛化重审所有产品域；不删除原正负 oracle。 |

base workflow 的相关入口在 `.github/workflows/rust-desktop-runtime.yml:5637–5737`：使用 `npm --prefix desktop run build:electron -- --plugin-skills-contracts` 和现有五个 Node contract bundles，输出 `.evidence/ocrm-plugin-edge.tap` / `.json`。base 的 named-required positives 只显式列出路由相关项（5669–5673），其 receipt inputs（5692–5708）未列 SDK 两处生产文件及 SDK/Host 两个测试。因此本批新增 named positives 与输入绑定是具体需要补充的证据点；本报告不声称已经核对主集成的新 workflow。

这批时序的直接 oracle 应能区分：A 被 barrier 留在真实 install I/O、B 同 key/version 已完成、A 随后失败；B 的 bytes、completion sentinel 与下一次 cache-hit 加载均保持。另需覆盖重叠成功不混写 / 不覆盖 winner、失败只清理本 attempt、普通首装与 cache hit、零 skill 成功保留，以及 Host `pruneOldVersions:false` 与既有默认行为。可以由主集成的 8 个具体测试承载这些事实；本报告不另造测试名或声称这些测试已经通过。

## 9. 收束

本批最小记账链接方案可按现有 SKILLS 责任推进。来源上的关键进展是实际读到了固定 U 的安装 caller、独有 staging、发布 receipt、受 ownership 保护的回滚 / 清理和 runtime cache lifetime 边界；不是新增全局闭包。历史 dossier 与 canonical source-review 未物化的差异、其余 dependency / platform / 产品接受缺口全部保留。

本 worker 只写入本报告。仓库源码、spec、ledger、RTM、inventory、workflow 均未修改，未执行任何可执行验证。
