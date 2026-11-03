# Media admission R1 — 独立 reuse / structure / performance 源码审查

结论：**发现 1 项 P2 correctness / resource-custody 缺陷，R1 不应据此宣布源码收敛。** 除下述 AgentManagement 多图片失败路径外，在本次完整冻结改动及必要邻接范围内，未发现另一个可证实的复用、所有权、结构或性能缺陷。此结论是源码审查，未执行产品、依赖 parser、build、test、lint、schema、generator 或 capture，也不代表 scoped runtime 或全量迁移接受。

## 审查身份、设置与独立性

- Worker：`/root/media_admission_structure_r1`；专门 lens 为 reuse / structure / performance，可报告具体 correctness。
- 实际调用设置：父层编排器明确传入 `gpt-6.1-sol`、`xhigh`、`fork_turns=none`；本报告按该 launch metadata 记录。选择 xhigh 的原因是跨平台 URL 规范化、native read 边界、实际 App custody / Session 生命周期，以及并发进入的 SGF shared gate。
- 先读取根 AGENTS blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2`，尤其独立/冻结审查、模型与 effort、Actions-only 和 OCRM 条款；读取 exact base 的 OCRM `SOURCE_OF_TRUTH.md` 当前权威/约束段、主规范 Revision 2 §17 全段、`management/prompts/full-fusion-batch.md` 全文，以及 module map 的媒体/渠道所有权段。
- 完整阅读候选两份 admission spec，基于它们审查当前生产实现。未打开其他或先前 reviewer 的报告、review decision ledger 或旧 review 输出；没有 dispatch 或启动其他 reviewer。历史失败身份仅来自本次 scope 内的 spec / evidence 源文件，不冒充本轮运行。
- 缺陷在向父层发送初步发现前独立形成。父层随后通知将在另一轮修复；本报告仍只评冻结 R1，未读取或评价未冻结 R2。
- 仅写本报告。未修改产品、台账、锁、workflow、任何真实 index、checkout 或 branch。邻接代码通过 `git show 497c2c7...:<path>` / exact blob 读取，未把有 WIP 的 checkout 当作候选源码。

| 身份 | 值 |
| --- | --- |
| Repository / branch | `bhrumom/fabushi-desktop` / `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed U tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Base / starting H | `497c2c7a7bc5201aead83d3d36e44f52ba164d04` |
| Base tree | `97181e47831735417e4aeb8197d9a62a5d66b12d` |
| Candidate tree | `3129fb0c3897bdf78d50e1cd067cc3dd65310297`，父层冻结 manifest 指定的远端 tree |
| Snapshot | `/workspace/scratch/7d27ea239237/media-admission-r1` |
| Base snapshot | `/workspace/scratch/7d27ea239237/media-admission-r1-base` |
| Manifest SHA-256 | `b99472353191573521cad3b98237a7adc7070d0c2fed3e71ba3fe0b4a47faa86` |
| Complete patch SHA-256 | `d37db7a1f0d16fc766dcf158de958739e45f1776d9d5535023025971dd4be4f0` |

独立做了只读字节核对：20 个候选路径的字节数、SHA-256 与 Git blob 计算值全部符合 manifest；15 个已存在路径的 base snapshot Git blob 亦全部相符；完整 patch 为 161,435 字节并符合上述 SHA-256。候选远端 tree 的 10,493 entries / non-truncated / 20 blobs readback 是父层提供的 freeze 信息；本 worker 没有再次远端枚举，不把该项写成独立 API 验证。

## Finding R1-STRUCT-01 — P2：后续图片 URL 拒绝会遗留 SendToAgent 已物化的前序媒体

**主要位置：** `source/host/app/src/main.rs:1180–1186`，`ProductionAgentManagementSink::resolve_image_source`。新增 `.map_err(...)?` 现在能够在后续图片上返回拒绝，而该 sink 仍以 `(resolved, _)` 丢弃 resolver 返回的 owned path。

**必要邻接证据：** exact-base `source/host/src/runner/tools/sand_agent_management_tools.rs` blob `e8cf8008e9c4eab2dbd909289ab6fa9a1345726c`，61–65、93–101 行；`source/host/src/runner/tools/send_message_schema.rs` blob `0e11c27ace3ae51888fdef4089cd11b09271ea7d`，151–155 行；`source/host/src/attachment_paths.rs` blob `c66a2d1ce8dfb70532a16857a0fe02d31788ca62`，236–254 行；候选 `main.rs:618–704`。

可证伪触发为：对真实有效 target 调用 `SendToAgent`，传两张图片，第一张是可读的正常 native `file:` URL，第二张是同一已存在 fixture 的 URL、但把一个路径分隔符写成 `%2F`。两张都能通过当前 `parse_images` 的“Url 可解析且 scheme 为 file/https”检查；该检查没有做共享 local-file admission。随后 bridge 逐个调用 sink：

1. 第一张进入实际 `resolve_production_attachment_source`，`fs::read` 成功，`persist_runner_media_bytes` 经既有 Session admitted-filesystem-mutation 写出 UUID 命名的 assets 文件，返回 `(resolved_url, Some(owned_path))`。
2. AgentManagement sink 丢弃 owned path，只把新 URL 放入临时结果 Vec。
3. 第二张通过新的共享转换得到 `Err`。`collect::<Result<Vec<_>, _>>()?` 提前结束，既没有调用 `send_to_agent_with_reply`，也没有将前序文件的引用写入实际 recipient transcript。
4. 这个 bridge 没有 SendMessage 的 `abort_attachment_custody` 分支；本路径也没有创建 `OutgoingMediaCustody`。临时 Vec 析构只丢弃字符串，不会删除 `persist_agent_media_bytes` 已 rename 的文件。请求返回错误，前序 managed 文件仍留在实际 owner 目录中；重复此失败可不断产生新的无引用副本。

这是新增拒绝传播带出的具体失败分支。它不依赖 Box 失败、未知写出结果、网络 share、并发 race 或物理 crash；不能用 spec 已另列的 persistence fallback / unknown-outcome 开口解释。R1 对 SendMessage 的相同“先有效、后拒绝”场景使用现有 custody/abort 正确处理，但 AgentManagement 的生产 consumer 没有对称的整批拒绝或回滚边界。

**建议的窄修复：** 在 AgentManagement 现有图片整批解析阶段，先复用 `resolve_local_file_url` 对所有 file URL 做纯准入，整批通过后才调用任何 materializing sink；或复用已有 outgoing-media ownership 机制，确保这个早退路径回滚本次实际拥有的文件。前一种方式可直接解决本 finding 的确定性 admission 场景，不需要新 registry、pending store、runtime 或公共 trait。仍应由实际 sink 保留最后的 `Result` 拒绝，避免 facade 的 `None` 再度变为原 URL 成功。

**要求的回归 oracle：** 扩展现有实际 App AgentManagement 用例，以“有效第一张 + 非法第二张”调用真实 `AgentManagementToolBridge` 和 shipping sink；比较前后 source/recipient transcript、managed media 文件集合、Session admission、wake/outgoing tracking，要求整体无变化并保留原 source bytes，再做合法调用恢复。必要时加纯 contract，确认第二张拒绝发生在第一张 sink 被调用之前。R1 的 `local_media_file_url_app_agent_management_refuses_image_before_outward_send`（`local_media_file_url_tests.rs:595–654`）只传一张非法图片，不能捕获这个问题。

## 复用、结构与性能判断

`media_reference.rs:20–180` 是已有 Attachments owner 内的一组纯准入函数。shared converter 明确区分 non-file `Ok(None)`、admitted file `Ok(Some(path))` 与 rejected file `Err`；`Cell` 是单次同步解析的局部状态，用 parser 的公开 violation callback 保留被规范化丢弃的 remote-host 证据。它没有共享可变状态、线程、锁、任务、数据库、permission owner 或 I/O。

读取实际依赖后，Windows drive-root 前置检查有因果依据：锁定 `url` 的 Windows converter 从第一个 drive segment 开始拼接，bare drive 可构造非绝对结果并到达 debug assertion；本候选在 `to_file_path` 前检查 decoded drive-root，随后检查实际 native path 的绝对性和 spelling。Windows guard 的网络/namespace/colon 条件对应实际 fs-safe 的 spelling 合同；Unix 分支只增加 NUL 早拒绝，保留 colon 和普通本地路径。

`send_message_tool.rs:41–45` 保持 metadata-only Option facade，通过共享 owner 委托；它不再自带第二个 converter。App 两个实际 sink 直接消费 Result。`media_file_context.rs:170–207` 的旧手工/宽松 fallback 被移除，转换失败保留 None；普通 absolute / home 路径也经过同一个 spelling guard。`channel_attachment.rs:60–74` 先解析，再 reanchor，再纯 guard，之后才 `metadata` / bounded read；Xuser 实际 consumer 经 `AttachmentsService::resolve_channel_attachment` 使用同一路径。

`web_media.rs:267–310` 留在原 adapter 的 prefix guard 只保存该 adapter 比 Unix fs-safe 更严格的历史 spelling 规则：忽略 parser 同样忽略的 tab/CR/LF、跳过 leading C0/space，然后检查四个 slash/backslash。该函数没有重建 authority 或 native path；结构解析仍在 shared owner。这与 shared helper 的 parse-error scheme classification 责任不同，不构成应合并的重复 parser。typed error 映射保留 `invalid-file-url`，raw/workspace network 分类保留既有 `network-path-not-allowed`。

Web 现有 private synchronous reader seam 未扩大公共注入面。production 总是传 `load_local_web_media`，所有 canonicalize、bounded read 与格式处理仍在 `web_media.rs:375–477`；准入在 `local_reader(...)` 前结束。roots、sandbox、HTML、cap、image body 没有被这次 converter 合并重写。direct service arm 只是调用现有 loader；finite public Gateway registry 没有新增 `loadWebMedia`。

SendMessage 的实际链已经深读：shared admission → read/materialize → Session media admission → pending outgoing custody → bridge 构建 message → canonical Transcript append → commit/optional Host delivery。后续图片拒绝会走 `send_message_tool.rs:405–410` 的原 abort，App `main.rs:969–995` 移除 owned 文件并释放 media admission。Session work admission 仍先于 source resolution（`main.rs:885–898`）；没有为准入新增 owner 或持久状态。AgentManagement 的例外就是上述 finding。

成本为对 reference/path 的固定次数线性扫描，加既有 Url/native PathBuf 分配；严格 percent / UTF-8 检查最多产生线性大小的解码 buffer，没有 nested retry、递归、I/O probe、全局缓存、background task 或 quadratic 组合。web 的 prefix guard 到四个 separator 即停止。没有发现足以报告的性能回退；这只是静态复杂度与分配判断，未做 benchmark，也不证明完整 migration 的功耗/性能 gates。

## 完整 20-path scope 的实际阅读处置

所有路径均按 manifest bytes/blob 身份读取或对比。下表中的“完整改动”指完整 frozen base-to-candidate delta，不把未改大文件整篇阅读冒充已完成。

| 路径 | 审查深度与范围 |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | 完整改动；触发/branch/subject binding，三平台 capture 调用与 artifact，两个 SGF required lists、13/14/17 汇总，Linux App 输入、实际非 test check / harness 构建、exact selection 和 exit 判定 |
| `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md` | 全文；LFA-01–06、ownership、state/error/reopen、范围与待验条件 |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | 全文；FUA retained prefix/private reader/typed-error 合同及历史失败边界 |
| `projects/openclaw-rust/evidence/local-media-file-url-647e0932-20261103.json` | 全文；身份、生产/测试接线与 not-run 状态声明，未将声明当运行证据 |
| `projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json` | 全文；历史 failure / repair provenance 与当前 not-run 的分离，未读旧 review 报告 |
| `projects/openclaw-rust/parity-ledger.json` | 完整改动；只涉及四个既有 responsibility 的 behavior / owner / risk / evidence / test refs，status、completion 未被 delta 提升 |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | 完整改动；六处 existing requirement / AC evidence refs 增补，没有改验收或完成状态 |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | 全文；11 groups、OS 分支、exact positive names、oracle controls、原始 log hash、command/capture exits、H/U/platform/input binding |
| `source/host/app/src/main.rs` | 完整改动；618–1245 相关实际 resolver、两个 sinks、custody rollback、Session fence、Transcript/Host delivery 邻接链和新增 test module 接线 |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | 全文 693 行；actual owners、观察 wrapper、owned source 与 materialized bytes、partial rollback、commit、reopen、retry/repair、AgentManagement 与 HTTPS |
| `source/host/src/connectors/channel_attachment.rs` | 全文；scheme passthrough、local resolution、reanchor、metadata、bounded read、filename/MIME outcome |
| `source/host/src/extensions/attachments/media_file_context.rs` | 完整改动及 local-path 函数/邻接 API；移除 fallback 后的 None / home / Windows behavior |
| `source/host/src/extensions/attachments/media_reference.rs` | 全文；新增 shared admission 与既有 media classification / inbound / bounded reader 所有权 |
| `source/host/src/extensions/attachments/web_media.rs` | 全文；admission/private read 与全部新增/保留 boundary tests |
| `source/host/src/runner/tools/send_message_tool.rs` | 全文；Option facade、真实 bridge 的 sequential resolution、abort、encoding/send、观察/计数接口 |
| `source/host/tests/channel_attachment_contract.rs` | 全文；existing positives / bounds，native bytes、metadata、invalid encoding retry，Windows pure refusals |
| `source/host/tests/local_media_file_url_contract.rs` | 全文；三态、normalization、all authority classes、percent/UTF-8/NUL、decode-once、OS positive/negative |
| `source/host/tests/openclaw_media_file_context_contract.rs` | 完整改动和实际 service dispatch/helper 邻接；native exact path 替代不成立的 Windows POSIX positive，Unix control 保留，OS guards |
| `source/host/tests/openclaw_web_media_contract.rs` | 完整改动；actual loader/service helpers、native bytes/cap/retry/alias controls、新增 local URL / typed-error cases |
| `source/host/tests/send_message_tool_contract.rs` | 完整新增改动与 test-name inventory；两个 metadata facade cases、完整 17-case target 对 shared gates 的影响 |

额外 exact-base 邻接读取包括：`sand_agent_management_tools.rs` 的 trait / dispatch / image parser、`send_message_schema.rs::is_valid_attachment_url`、`attachment_paths.rs` 的 persistence / custody / file URL producer、`host_paths.rs::reanchor_sand_path`、`attachments_service.rs` 的 local-path / Channel / loadWebMedia arms、`cross_user_sharing/production.rs:385–398` 的真实消费者，以及 `channel_host_lane_tests.rs` 的 `IdleRunnerOwners` 构建/处置代码。不存在依赖 mutable workingtree 邻接源码的结论。

## 依赖合同阅读与身份

固定 U 的四个实际 source 文件在既有只读 acquired source 中核对 Git blob：`local-media-path.ts` 为 `f105ca514b5754849284c429b032af30d1487f1a`（全文），Windows local-path oracle 为 `1aef7d957f8e243203ccec8dd92742ca3e795761`（全文），web-media 为 `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`（读取 516–575、622–725 的实际 converter → reader 边界），web-media Windows oracle 为 `bdaedcde111ea2880b96d9afa547a9c1d2a36b42`（全文）。这些是边界合同 review，不是 upstream 全媒体实现阅读。

`url-2.5.8.crate` 为 86,512 字节，SHA-256 `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`；Host 与 App exact-base locks 都固定相同版本/checksum。只读 archive 的 parser/lib 成员 Git blob 分别为 `dbdf9b906238c20ef0896d114d5cb8ad69197e34` / `f1558682b6287d0cba1cd93d84cee6cf6cbeaf44`；`.cargo_vcs_info.json` 对应 `d6ea13c5f8e7e6e627f6390161b3e185bda5e5ce`。深读 parser 1250–1415 的 path / drive / host erase / leading-empty removal，以及 lib 2726–2757、3064–3191 的实际 Unix/Windows native conversion。

`fs-safe-0.27.0.tgz` 为 1,101,331 字节，SHA-256 `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`。读取实际 packaged `safeFileURLToPath` / network check 段、`windows-path-alias.js` 与 `windows-path-syntax.js` 全文。未执行原例子或任何 package code；未宣称完整包、native binaries、生成输入、licenses 或递归依赖闭包审查。

## 捕获与 shared gate 的源码核查

`capture_media_cleanup.py` 保留原 7 groups 并新增现有 owner/consumer targets，继续拒绝缺名、重复、额外 positives、失败、ignored、zero-test、partial/duplicate/failure summary、nonzero cargo / capture exit；13 个 synthetic qualification controls 仍在。H 必须同时匹配 `FABUSHI_TARGET_SHA` / optional frozen dispatch SHA；执行平台必须匹配 runner OS；tracked inputs 在前后核对 HEAD blob / normalized-worktree blob 与工作字节 SHA-256。新消费者源码、测试与 spec 已进入 INPUTS。

以下是依据源码列表和 cfg 分支复核的 **计划选名数，不是已通过数量**：

| Group | Linux | macOS | Windows |
| --- | ---: | ---: | ---: |
| cleanup-owner | 7 | 7 | 8 |
| cleanup-consumers | 17 | 17 | 16 |
| html-policy | 12 | 12 | 9 |
| host-read-format | 1 | 1 | 1 |
| file-url-admission | 5 | 5 | 6 |
| explicit-html-consumers | 24 | 24 | 23 |
| image-probe-consumers | 16 | 16 | 16 |
| local-file-url-owner | 10 | 10 | 11 |
| file-context-consumers | 10 | 10 | 10 |
| channel-attachment-consumers | 7 | 7 | 8 |
| send-message-consumers | 17 | 17 | 17 |
| **合计** | **126** | **126** | **125** |

Workflow 两处旧 SGF required-name lists 均增加两个 LFA facade cases。完整 `generated-send-message-contracts` command 仍选择 `send_message_tool_contract`、`session_conversation_state_contract`、`generated_occurrence_codec_contract`；summary 只有 SendMessage 从 15 改 17，另两个 13/14、zero failed/ignored、name uniqueness 与总 cardinality 保留。未发现为合并 LFA 弱化 SGF gate 的改动。

Linux App 仍先 `cargo check` actual cfg(not(test)) bin，再由 Cargo JSON 确认唯一的 actual main test executable、owned target path 和 executable hash。新增六项完整 exact names 在 `--exact --list` 中要求唯一，再逐个 exact run，保留非零 exit、one-passed/zero-failed/zero-ignored 和缺项拒绝。实际 App tests 的观察 wrapper 委托 production sink，并在第一张成功后记录真实 owned bytes / pending map / durable admission；reopen 会处置并重建实际 Session/Transcript/ForeverBox owners。它没有替换 source resolver 或 persistence owner。AgentManagement 负例的单图片覆盖不足已包含在 finding，而非将这六项源码当作已通过证据。

## 明确未验范围与出口

本轮不执行任何本机或 htch 验证，不运行 source 中的 parser、tests、schema、capture/oracle qualification，也不做性能 measurement。允许的操作只有 source/byte/hash/archive/Git/API 读取与报告写入。未独立远端重取完整 candidate tree；未深读 unchanged App main 其余约万行、所有 Session/Transcript implementation、全部旧媒体测试、全部 workflow jobs 或依赖包实现；仅在上述明确边界使用必要 exact-base 接口/行为。

修复本 finding 后，应冻结新的完整候选并做 focused independent re-review，保留 R1 身份。本轮尚无 corrected exact-H Actions 证据。仍需该新 H 的三平台 11-group media receipts、Linux actual App check/harness/六项及新增整批拒绝回归、受影响 SGF complete-target gates；要求对应 H/tree/U/input blobs、原始 log hashes、command/capture exits、required name uniqueness 和全部 selected assertions。Windows native cases 不能由 Linux/macOS 代替，排队/缺失结果继续未验证。

权限/root confinement、opened-handle/TOCTOU、generated HTML provenance、persistence fallback/unknown outcomes、真实 provider/channel/device、packaging、soak、完整性能/功耗及 G01–G12 / AC01–AC24 / independent ACCEPT 不在本轮被宣布通过。两份 evidence 维持当前 candidate not-run 与全局 flags 未提升的方向正确；本报告不评价最终全量迁移是否完成。
