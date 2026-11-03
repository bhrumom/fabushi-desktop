# Media admission R1 — independent GENERAL / security / correctness review

## 结论

**REQUEST_CHANGES — 1 项 P2 发现。**

冻结 R1 的共享 URL/path 准入、实际 SendMessage 错误传播与回滚、Channel 最终路径检查、web 私有 reader 边界，以及 SGF 共享 gate 的整合总体有对应源码证据。但是，真实 `SendToAgent` 在“第一张合法本地图已经物化，第二张图被新增准入检查拒绝”时，会遗留第一张受管文件；其 owner 没有捕获或回滚该文件。详见 `LFA-GEN-R1-01`。

除该项外，本轮所列读取范围内没有其他已确认的 GENERAL/security/correctness 发现。此结论仅评价冻结源码，**不是 Actions 通过、运行时验收、完整迁移 ACCEPT 或发布批准**。R1 必须保留此发现；任何后续修正和并发整合都属于另一个冻结对象，本报告不评价它们。

## 1. 审查身份、独立性与限制

- Reviewer：`/root/media_admission_general_r1`，fresh blind GENERAL/security/correctness worker；按父任务要求 `fork_turns=none`。
- Model：本会话系统声明 **GPT-6 Astra Pro**；GENERAL lens 保留父级 configured model，没有请求或执行 model override。
- Effort：父任务明确将完整范围分类为复杂，并指定 **xhigh**；父任务说明实际 adapter 已应用此设置。本 worker 未自行启动其他模型、CLI 或 worker；可见工具未提供独立的底层模型/effort introspection，因此不虚构另一份运行参数证明。
- 复杂性依据：URL parser 会丢弃输入信息；Windows 路径别名与 native conversion；实际 App、Session、Transcript 和文件 custody 的副作用；跨 OS oracle；并发 SGF 共享目标与严格计数。
- 没有参与本候选实现，没有读取其他 reviewer 报告或旧 review 报告。候选 spec/evidence 中保留的历史描述仅作本批声明审查，没有据其继承审批。
- 只做 source、字节/hash、archive、Git object 与 exact-blob API 读取。没有运行产品、URL 示例、parser、build、test、lint、schema、generator 或 capture；没有 htch 验证，没有 dispatch，也没有修改代码、锁、workflow、账本、branch 或 index。唯一写入是本报告。
- `/workspace/scratch/29e7135d8bfe/fabushi-desktop` 只用作 exact Git object 容器：相邻源码均通过固定 base 或 blob 读取，没有使用其中其他会话的 mutable WIP。

## 2. 精确冻结身份与核对

| 项目 | 身份 |
| --- | --- |
| Repository / branch | `bhrumom/fabushi-desktop` / `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| U tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Base / starting H | `497c2c7a7bc5201aead83d3d36e44f52ba164d04` |
| Base tree | `97181e47831735417e4aeb8197d9a62a5d66b12d` |
| Frozen candidate tree | `3129fb0c3897bdf78d50e1cd067cc3dd65310297` |
| Manifest | `/workspace/scratch/7d27ea239237/media-admission-r1-manifest.json` |
| Manifest SHA-256 | `b99472353191573521cad3b98237a7adc7070d0c2fed3e71ba3fe0b4a47faa86` |
| Complete patch | `/workspace/scratch/7d27ea239237/media-admission-r1-complete.patch` |
| Patch SHA-256 | `d37db7a1f0d16fc766dcf158de958739e45f1776d9d5535023025971dd4be4f0` |
| Candidate / base snapshots | `/workspace/scratch/7d27ea239237/media-admission-r1` / `/workspace/scratch/7d27ea239237/media-admission-r1-base` |

独立逐项重算了全部 **20 个 candidate 文件的 bytes、SHA-256、Git blob SHA-1**，并核对每个现有 base snapshot 的 Git blob 和新增路径的 base 缺失，全部与 manifest 一致。收尾时再次读取并重算，20 项仍一致。完整 patch SHA-256 也独立匹配。

Manifest 记载的远端 candidate tree 非截断 10,493 entries 回读和 20 blobs 匹配，是主集成人提供的冻结记录；本 reviewer **没有另行完整下载该远端 tree**。本轮独立审查对象是上述已逐文件核对的完整 20-path snapshot，加 exact base 的相邻生产源码。未把 tree identity 冒充已存在的 candidate commit H 或已执行 receipt。

### 适用 authority 的读取事实

| Authority | Exact blob | 本轮读取范围 |
| --- | --- | --- |
| Root `AGENTS.md` | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` | 全文 1–395；独立审查/冻结/模型 effort 237–270 另行核对 |
| `projects/openclaw-rust/SOURCE_OF_TRUTH.md` | `3748d734b10c9ed5b2d948467fbaa9483f1dd306` | authority、当前 U、约束 1–85；终态/执行 453–508；不声明历史 discovery 全文深读 |
| 主 Spec `docs/specs/openclaw-rust-equivalence-migration.md` | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` | 637–720，重点完整读取当前 OCRM-EXEC-02 §17 的 671–720 |
| `management/prompts/full-fusion-batch.md` | `5ff7894d245588ed539ab075b09215a2f1a82716` | 全文 1–14 |
| `module-map.md` | `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3` | 导航 1–64、84–101、243–279；不声明全文深读 |
| LFA spec | 见下表 | **全文 1–86** |
| FUA spec | 见下表 | **全文 1–165** |

Exact base 的 AGENTS 路径核对没有发现适用于这些 Host/App/项目路径的额外 nested AGENTS。上述 authority 来自 exact base/tree/blob，不来自可变工作文件。

## 3. Finding

### LFA-GEN-R1-01 — [P2 / Medium] SendToAgent 后续图片被拒绝时遗留已物化的前序图片

**主位置：** `source/host/app/src/main.rs:1180–1186`，`ProductionAgentManagementSink::resolve_image_source`。本次新增 `.map_err(ProviderSessionError::Tool)?` 在这里把准入失败向上返回，而返回值中的 `owned_path` 仍在 1180 被 `_` 丢弃。

**相邻真实调用链：** exact base `source/host/src/runner/tools/sand_agent_management_tools.rs:55–65,93–101`（blob `e8cf8008e9c4eab2dbd909289ab6fa9a1345726c`）；候选 `main.rs:618–705`；exact base `source/host/src/attachment_paths.rs:236–254`（blob `c66a2d1ce8dfb70532a16857a0fe02d31788ca62`）。

**具体触发：** 正常已准入的 source Session 向一个有效、不同于自己的 target 调用真实 `SendToAgent`，提交两张图片：

1. 第一张是 fixture 自有、可读取 PNG 的合法 native file URL，例如 `file:///owned/first.png`（Windows 使用 native drive URL）。
2. 第二张是同一 owned fixture 路径的 encoded-separator 变体，例如 `file:///owned%2Ffirst.png`。它有合法 file scheme，`Url::parse` 可接受，因此通过现有 `parse_images` 的 scheme-only 验证；新的共享准入在 pathname 检查时拒绝 `%2F`。

**源码证明的顺序：**

- `parse_images` 完成整批结构/scheme 检查，但没有调用 shared local-file admission（93–101）。
- 随后的 iterator 顺序调用每张图片的真实 `resolve_image_source`（61–64）。第一张经过 `main.rs:659–675` 读取并成功调用 `persist_runner_media_bytes`。`persist_agent_media_bytes` 使用新的 UUID 文件名，写临时文件后 rename 为 source agent 的受管 assets 文件（`attachment_paths.rs:242–254`）。
- `ProductionAgentManagementSink` 不持有这一 owned path；它不像 SendMessage sink 那样调用 `capture_session_media_admission`，也没有为这次调用建立 `OutgoingMediaCustody`。
- 第二张到达新增 `main.rs:1186` 的 `?` 后，iterator 的 `collect::<Result<Vec<_>,_>>()?` 直接退出。`send_to_agent_with_reply` 尚未发生；这个 bridge/trait 没有 abort hook，第一张已经写好的文件也没有被删除。
- Session 的既有恢复只回收 durable media fence 中记录的 paths；`production.rs:1757–1772` 在没有 work id、没有 paths 时直接返回。这里丢弃 owned path 且未建立 fence，因此这一正常的 URL 拒绝路径也不能由既有 reopen recovery 自动补救。

**影响：** 请求返回失败、没有对应 target 消息，但 source agent 的受管目录新增无消息引用、无 custody 记录的文件。反复提交该序列会反复产生 UUID 副本并占用磁盘；正常 reopen 不会凭空恢复被丢弃的文件身份。第二个不合法 URL 仍然在它自己的 read 前被正确拒绝；本发现针对的是整个失败调用的前序副作用，不声称发生了远程 share 访问或对外泄露。

这是本次新增 admission error 的可达清理缺口，**与 spec 明确保留的“media persistence 失败后 Box/original-URL fallback”不同**：此触发中第一张的持久化已成功，第二张因新纯准入规则失败。先前 AgentManagement resolver 没有在这个位置返回该错误；本批给顺序 materialization 增加了新的中途退出。

**现有用例为何不足：** `source/host/app/src/tests/local_media_file_url_tests.rs:596–654` 确实调用真实 AgentManagement sink 和真实 bridge，但 631–634 只传入一张非法图片。它能证明第一张即拒绝时无 effects，不能覆盖先成功物化、后准入失败的序列。SendMessage 的 421–456 用例已做因果更强的前序物化观察和回滚，但这不会覆盖另一条 AgentManagement 调用链。

**具体修复建议：** 在现有 `SendToAgent` 的整批图片解析/准入阶段，复用同一个纯 `resolve_local_file_url` owner，对所有图片完成准入后再进入带副作用的逐图 resolution；继续保留生产 sink 的防御性检查。也可复用现有 custody 机制，在该 bridge 的 resolution 失败时回滚已物化的路径，但不要另造第二个状态/恢复 owner。前一种方式能在本批错误类别上保持没有任何前序 materialization。

**所需验证：** 在真实 App AgentManagement sink + bridge 中增加 valid-first/invalid-later 用例；断言整个调用失败、source/target transcript 与 admission 不变、源文件保留、受管文件集合不变、没有 wake/message tracking，并在重复调用和真实 reopen 后再次检查不累积文件。保留合法本地图片能真正物化/发送的正控制，防止“从未进入真实 sink”造成假阳性。危险变体仅使用 owned local fixture；不要访问实际 share。

**证据等级：** source-confirmed；未在本地执行用例，不能写成已复现的 Actions failure。修复必须在新的冻结 scope 中独立复审，再取得 corrected exact-H Actions。

## 4. 其余关键审查结果

### 4.1 共同准入 owner 与规范化信息保留

`media_reference.rs:20–69,72–181` 是现有 owner 内的无 I/O 扩展，没有新增 module、store、permission authority 或 public reader capability。三态结果清楚区分非 file input、合法本地 file URL 与非法 file URL。parse-error 分类匹配 parser 的 C0/space trim 与 TAB/CR/LF 忽略规则；成功 parse 的 scheme 仍由 URL parser 决定。

远端 authority、userinfo/port、pathname 非法 percent、编码分隔符、无效 UTF-8 和 NUL 都在 native filesystem 转换/访问前拒绝。检查作用于 pathname，query/fragment 不被误当路径。先严格检查、再一次 percent decoding，没有 manual/lossy fallback。`SyntaxViolation::FileWithHostAndWindowsDrive` 回调保留了 parser 在 drive normalization 时删除 remote host 的事实，防止 post-parse host 检查被归一化绕过。

Windows 在 `to_file_path` 前要求 decoded `/drive:/`，因此 bare `file:///C:`/encoded-colon 不到达依赖的绝对路径 debug assertion；转换后再次要求 native absolute，并检查网络/device、drive-relative/ADS colon 别名。普通 rooted drive 与允许的 rooted verbatim drive 仍有正控制。Unix colon filename 和该共同 owner 的 Unix 路径行为保留；不把这一拼写 guard 解释成 symlink、TOCTOU、opened-handle 或 root confinement。

### 4.2 实际 SendMessage 路径、状态 owner 与恢复

App 的 `resolve_production_attachment_source` 在 `main.rs:645–647` 消费 Result，早于 `fs::read`（659）、ForeverBox status/download（678–684）和 materialization。非 file/HTTPS 保持现有 URL passthrough。Runner 的 Option facade 仅保留 metadata/default-trait 用途；实际 App 不再用 `None` 混同非法 file URL。

`main.rs:880–966` 保留 Session work admission 在 resolver 之前，以及真实媒体路径 capture、pending map 和错误清理；969–996 的 abort 回滚既有 custody 并释放对应 lifecycle/work id。`SendMessageToolBridge` 的构建错误分支调用该 abort；CountingSendMessageSink 仍转发错误、dimensions 和 abort。成功路径仍由 Transcript 的既有 append-and-release-media 和 AgentDB transaction 共同结算，没有新增 durable owner。

真实 production composition 从 App 11553–11590、12318/12330，经 `turn_agent_composition.rs:1011–1032` 到 `turn_toolset.rs:751–754,824–831`，分别装入真实 AgentManagement/SendMessage bridges。实际 source-resolution caller 搜索没有发现遗漏的第三个 App caller；当前 P2 位于已识别的第二条链的批量错误路径。

新增 App tests 的 observation port 委派真实私有 `ProductionSendMessageSink`，在 resolver 成功后立即读取实际文件字节、pending work id 和 durable Session media fence（tests 116–138）。因此 421–456 的 later-image refusal 不只断言最终目录为空，还先证明第一张真实物化，再证明实际 bridge abort 删除受管副本、保留原文件、保持 Transcript 和 fence。正控制与重复失败/修复/reopen 测试 459–593 使用真实 Session/Transcript owners，不是用同名 mock resolver 自证。

既有 persistence Option/Box/original-URL fallback 仍在 `main.rs:618–636,659–704`，与 spec 的显式 open item 一致；没有把它写成已解决。物理进程 crash、未知 commit outcome 和取消时序没有因这些 source checks 获得批准。

### 4.3 Channel/Xuser 与 web adapter

Channel 经 file-context 共享 owner，在 `channel_attachment.rs:60–63` 对 **reanchor 后** 的最终 path 调用 guard，早于 metadata 和 bounded reader。真实 `AttachmentsService::resolve_channel_attachment` 与 Xuser publication 消费该路径；现有 invalid→None/omission、HTTP(S) URL 和本地 bytes/MIME/size 路径保留。

`web_media.rs:263–311` 在 parser 之前保留该 adapter 特有的四个及更多 slash/backslash/mixed-separator 拒绝，并覆盖 leading C0 与 parser 忽略 TAB/CR/LF 的拼写。它委派公共结构化转换而保留私有 synchronous reader。URL error 仍映射 `invalid-file-url`；raw/workspace network path 的旧 `network-path-not-allowed` 顺序保留；新 alias/NUL guard 使用 `path-not-allowed`。最终 path admission 位于 canonicalization、metadata、open/read 和原有 HTML/image body 之前。

Windows verbatim-drive workspace 的特殊允许仍保持，UNC/device 不因此放行。web adapter 的 Unix raw/resolved network 限制与共同 owner 的平台合同不同，spec 33、62 已明确，不能据此宣称完整 Node/Rust Unix URL 等价。

`AttachmentsService` 的 `loadWebMedia` arm 是现有 direct service dispatch。实际 `gateway_protocol::is_grok_gateway_command` 与 `host_gateway_api` 的有限注册没有增加该方法；未发现 public Gateway 或文件读取权限扩大。roots、sandbox flags 和 reader 注入没有新增外部入口。

### 4.4 危险输入、兼容性与 oracle

远端 host/UNC/device 负例走纯 converter 或私有 no-I/O boundary；实际 App negative fixtures 是 owned local file 的 percent 编码变体。没有要求负例真正接触 share。私有边界 tests 观察 reader 次数，这证明准入先于该 reader；它们不是任意 OS syscall 计数器，报告不作这种夸大。

Native tests 保留 uppercase/single-slash/localhost、空间/Unicode/percent filename、精确 path 和 byte 控制。原有跨平台 drive-less POSIX 正断言改为实际 native fixture，Unix 原断言保留，Windows 增加明确拒绝，没有将 Windows conversion 失败改判成功。

Box root/desktop/image helper 和其现有 contract assertions 保留；实际 App resolver 在通过 admission 后仍调用同一 helper。HTTPS 在真实 App SendMessage 和 AgentManagement source resolution 中保持 passthrough。此为 source compatibility 审查，未声称 live Box、provider 或 channel 已验证。

### 4.5 Capture、共享 SGF gate、账本和证据

`capture_media_cleanup.py` 全文已读。现有三平台调用保留，11 groups 的 source-selected 数是 Linux/macOS **126**、Windows **125**；这些是源码选取数，**不是通过数**。新增四个完整目标和现有 web 目标覆盖新调用，Windows-specific cases 在原生 Windows 选取。capture 的 exact-H/U/platform、前后 input blobs、command/capture exits、exact positive-name uniqueness、无 extra、单一精确 summary、zero failed/ignored，以及 13 个 synthetic oracle qualification controls 没有放宽。

Workflow 两处 required-name 表都新增两个 LFA facade names（1813–1814、4020–4021）。SGF `generated-send-message-contracts` 仍跑完整 SendMessage target，4419–4421 只把其 summary 从 15 提升到 17，另两个保持 13、14；required positive 每个恰一次、总数一致、三份 zero-failed/zero-ignored summary 的检查保留。没有漏掉 inherited SGF App cases。

Linux actual App capture 继续编译 non-test App 和实际 linked test executable，绑定源输入并通过精确 `--list`/`--exact` 运行新增 6 cases（4990–5013）。这些目前还不能捕获 finding 中的 AgentManagement valid-first/invalid-later 序列。源码 gate 的存在不是该 candidate 的执行结果。

字节 diff 和只读 JSON 数据比较确认 ledger 仅修改四个既有责任：CHANNEL-ATTACHMENT-RESOLVER、MEDIA-LOCAL-PATH、MEDIA-REFERENCE-RESOLUTION、MEDIA-WEB-PIPELINE。改动是行为/风险/owner 集成描述、合同和证据/测试引用，没有改变 status。RTM 仅在 R15/R23/R24/AC15/AC23/AC24 追加四项 evidence_refs。两个文件的八个 completion flags 全部仍为 false，没有新 ledger、没有 unit verified/accepted 提升。

两份新增 evidence 全文按声明审查；它们明确区分 historical H364 失败、未发布历史 slice 与本 candidate 未运行状态。没有把 H364 Linux/Windows 或旧 source 结果解释为本冻结对象的批准。历史 log/hash 数字在本轮没有重新下载所有原始 CI artifacts；它们不是本 finding 或本 source verdict 的必要证明。

## 5. 完整 changed-path 处置与实际读取范围

表内路径相对 repository root。所有候选 blob 均独立匹配 manifest。范围是实际阅读事实；“全部 hunks”不等同把巨大文件全文读完。

| Path | Candidate blob | 读取/处置 |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `878c50de122b1b5dd8c83bbfeaeec87d9039c58c` | 全部差异 hunks；相关触发/OS/绑定、target、oracle、App executable/selection：1–160、1590–1632、1790–1842、3172–3260、3995–4062、4392–4438、4698–4886、4985–5046、5190–5260 |
| `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md` | `0c923b5fc38eac3b7d9d80a9de906af1d4bf7e03` | 全文 1–86 |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `12dafdc789de410302aff4562cc3432945184bea` | 全文 1–165 |
| `projects/openclaw-rust/evidence/local-media-file-url-647e0932-20261103.json` | `3694addf9afb7470a1398c43afe81f8a06fa21dc` | 全文 1–201；候选声明/身份/未验收限制 |
| `projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json` | `40844282d947bf6f7c769bb35cc05b80dac8a59e` | 全文 1–126；历史身份与当前未执行状态分离 |
| `projects/openclaw-rust/parity-ledger.json` | `e9304c81b3fe856f95a5566db230d93355f19365` | 全部差异 hunks、只读 JSON 字段比较、八 flags；未声称全文业务审查 |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `2fdb3b6a522fa55854ab0399dcf76268e2a9eb23` | 全部六个引用差异、状态/flags 比较 |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `61ec9ca1827cc175b877f4fc046c2f06fa47d356` | 全文 1–416，仅 source reading |
| `source/host/app/src/main.rs` | `6a3c7035fa0c43fdc4b513baa93fa0c224b072c0` | 全部 product hunks、540–1225 相关实现、11510–11625 composition、12300–12355 sink 注入、17922 附近 module registration；不声称约 18k 行全文 |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `b00e0e4b367ab5abc41c42ca0d453824fd1ff317` | 全文 1–693 |
| `source/host/src/connectors/channel_attachment.rs` | `723006c2a5b054fc9ddc47f58fdb4ebd8639351e` | 全文 1–93 |
| `source/host/src/extensions/attachments/media_file_context.rs` | `0777b2fff28f09e5a6f276ed71c6ec776d6601e6` | 全文 1–309；旧 fallback 删除差异另核对 |
| `source/host/src/extensions/attachments/media_reference.rs` | `8d547f434ddb15e8119b724554a346e5eeb3bce0` | 全文 1–490 |
| `source/host/src/extensions/attachments/web_media.rs` | `041078abfae537ab8a9a9a0044c8fec9970d9494` | 全文 1–662，含 unchanged read/HTML/image body 与全部私有边界 tests |
| `source/host/src/runner/tools/send_message_tool.rs` | `9e12d7d2b457715f0fac168e5e4f262e6ef15985` | 全文 1–481 |
| `source/host/tests/channel_attachment_contract.rs` | `887f2cb3ea78060be617ad1c039f5b28306c6ac0` | 全文 1–208 |
| `source/host/tests/local_media_file_url_contract.rs` | `5170951a59fb94d8c1a3673586690a9dc8315a88` | 全文 1–199 |
| `source/host/tests/openclaw_media_file_context_contract.rs` | `2c2365d9f2ab380bd2fc4374dba4096a1911fd0f` | 全部差异/相关控制 120–215、350–466 |
| `source/host/tests/openclaw_web_media_contract.rs` | `a965e74b2e4097da9b3bf5fda4d8742b240f07e9` | 全部差异/相关控制 160–345、835–923 |
| `source/host/tests/send_message_tool_contract.rs` | `8c334d36e4b507842a38509b945956fe5085beb7` | 全文 1–685；保留 Box/SGF/assertion 与新增 facade cases |

### Exact-base 相邻生产调用链

以下都来自 `497c2c7a7bc5201aead83d3d36e44f52ba164d04`。只记实际读取，不把搜索命中或整文件取 hash 当成全文阅读。

| Path | 已读范围/目的 |
| --- | --- |
| `source/host/src/runner/tools/sand_agent_management_tools.rs` | 全文 1–102；parse_images、顺序 resolution、send/error 与 trait 无 abort 的事实 |
| `source/host/src/runner/tools/send_message_schema.rs` | 140–182、312–360；实际 file/HTTPS scheme 验证 |
| `source/host/src/runner/tools/send_message_encoding.rs` | 206–275；Box roots、desktop gate、download/persist 分支 |
| `source/host/src/runner/turn_agent_composition.rs` | 988–1042；production sink 传入 toolset |
| `source/host/src/runner/tools/turn_toolset.rs` | 722–765、806–850；实际 AgentManagement/SendMessage bridge 装配 |
| `source/host/src/attachment_paths.rs` | 1–100、205–275；既有 custody、agent roots、写临时文件/rename、URL 生成 |
| `source/host/src/host_paths.rs` | 140–215；最终 reanchor 行为 |
| `source/host/src/extensions/attachments/attachments_service.rs` | 1512–1535、1768–1800、1820–1895、1990–2065；Channel、file-context、web direct service 消费 |
| `source/host/src/extensions/cross_user_sharing/production.rs` | 360–410；实际 Xuser attachment consumer |
| `source/host/src/extensions/session/production.rs` | 1688–1733、1752–1835、2290–2355、2610–2815、4060–4082；open/recovery、admitted mutation、generation/capture/release、Transcript facade |
| `source/host/src/extensions/session/agent_db.rs` | 4680–4919；真实 BEGIN/identity/receipt/append/fence-clear/CAS/COMMIT/ROLLBACK |
| `source/host/src/extensions/transcript/production_runtime.rs` | 1665–1855；实际 generated SendMessage entry 及 custody 结算 |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | 140–335；新 App fixture 复用的真实 owners/Box plumbing 与 dispose |
| `source/host/src/gateway_protocol.rs` | 1–35；有限 public command 判定 |
| `source/host/src/host_gateway_api.rs` | 37–286；actual registry/owner macro 与所有方法字面量 |

实际 caller 搜索使用 exact-base Git object；修改过的函数体再以冻结 candidate 为准。没有把 mutable checkout 搜索结果当成审查依据。

## 6. Pinned U 与 dependency contract 的实际证据

### Fixed U 实际消费者

- `src/media/local-media-path.ts`，blob `f105ca514b5754849284c429b032af30d1487f1a`：全文 1–29。
- `src/media/local-media-path.windows.test.ts`，blob `1aef7d957f8e243203ccec8dd92742ca3e795761`：exact U tree 路径核对，GitHub exact-blob API 全文 98 行；本地 partial clone 没有该 blob，未执行 git fetch 或 URL 示例。
- `src/media/web-media.file-url.windows.test.ts`，blob `bdaedcde111ea2880b96d9afa547a9c1d2a36b42`：全文 1–48。
- `src/media/web-media.ts`，blob `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`：实际 URL admission、local path/read ordering 530–700。没有声明该大文件或整个 U 已全文审查。

### `url 2.5.8`

实际 archive `/tmp/fabushi-647e-pairing/local-file-url-dependency/url-2.5.8.crate`，独立核算 **86,512 bytes**，SHA-256 `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`。exact-base Host lock 3717–3726、App lock 5351–5360 同时绑定该版本/checksum；没有 lock 修改。

Archive 的 `.cargo_vcs_info.json` 指向 `d6ea13c5f8e7e6e627f6390161b3e185bda5e5ce`。直接从 archive 成员重算 Git blob：`src/parser.rs` = `dbdf9b906238c20ef0896d114d5cb8ad69197e34`；`src/lib.rs` = `f1558682b6287d0cba1cd93d84cee6cf6cbeaf44`，与给定 version-source identity 一致。

实际 contract/deep-risk 阅读范围：parser 76–82、180–245、512–625、1030–1120、1200–1410；lib 2675–2765、3064–3199。重点核对 file host/localhost、ignored controls、path empty-segment removal、FileWithHostAndWindowsDrive 事件以及 Windows bare-drive debug assertion 的前提。没有运行 parser 或 crate，没有声明整个依赖已 full-read、所有依赖递归 closure 或许可证总闭包完成。

### `@openclaw/fs-safe 0.27.0`

实际 archive `/tmp/fabushi-647e-pairing/media-fixed-dependency/fs-safe-0.27.0.tgz`，独立核算 **1,101,331 bytes**，SHA-256 `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`。

完整阅读实际发布成员 `dist/local-file-access.js` 1–81、`dist/windows-path-alias.js` 1–140、`dist/windows-path-syntax.js` 1–72、`dist/string-coerce.js` 1–16；读取 package 的 name/version/license/exports 和完整 MIT LICENSE。`local-file-access.js` SHA-256 独立匹配 `7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8`。核对空/localhost authority、编码分隔符、Node conversion 后 alias/network 拒绝及平台区别。

这些是所用发布 API/风险边界的 contract 审查，不是整个 fs-safe 包、Node URL runtime、生成器或全部传递来源的全文理解；不修改历史 full-read 事实，不据此提升 source closure。

## 7. 需要保留的验证与未核验项

1. 先处理 `LFA-GEN-R1-01`，以新的不可变完整 scope 和修正 delta 做 focused re-review；同时保留本 R1 原始 finding。不得用未冻结 WIP 或后续并发变更解释本报告。
2. 发布 corrected exact H 后，仅在 GitHub Actions 执行相关编译/合同/App 检查。保留三平台 native capture、11 groups、名称唯一/完整性和 zero failed/ignored；Linux 不能代替 Windows native conversion/alias 分支，macOS 未取得结果也保持未验证。
3. 保留真实 App 六类控制，并扩展 AgentManagement 混合图片序列。必须检查真实 managed bytes/Session/Transcript/reopen，而不是只看 helper 返回或空目录最终状态。
4. 重新取得同一 H/U/输入 blob 绑定的 App/native/SGF receipts、完整 raw logs/hash、command/capture exits。历史 H364 的失败和旧正结果都不能重标给新 H；source-selected 126/125、App 6 和 SGF 17/14/13 都不是本轮执行成绩。
5. 本轮没有核验实际编译成功、native Windows/macOS 运行、Live Box/provider/channel/device、HTTP/renderer 外部调用、真实取消/进程崩溃/未知 commit、TOCTOU/root confinement、完整 codec、打包/签名/安装升级回滚、性能/soak 或全部 G01–G12/AC01–AC24。它们仍按原合同开放，不因 source review 降级为 N/A。

**R1 terminal verdict：REQUEST_CHANGES，1 个 P2，0 个本地可执行验证，0 个可继承为本 candidate 的 Actions/最终 ACCEPT。**
