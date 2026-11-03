# FUA-v1 R2 独立 general / security / correctness 源码评审

## 结论

**REQUEST_CHANGES — 1 项具体 finding：FUA-R2-GEN-01（P2 / Medium）。**

本轮四分隔符归一化失败的修复与原始日志、锁定依赖实现一致；candidate 也正确保存了被 Windows drive normalization 丢弃的非空 host 信息。但完整 FUA-v1 范围内仍有一个未受保护的 Windows native-conversion 边界：file:///C: 会进入锁定 url 依赖的 drive-relative PathBuf 分支，开启 debug assertions 时 panic，关闭时返回 C: 并可能进入本地 reader，而不是 typed invalid-file-url。具体触发、源码因果链和修复要求见文末 finding。

没有发现本轮 Cell callback lifetime、新增前缀扫描或正常 rooted native / single-slash / localhost fixture 的另一个可证实缺陷。尚存 finding 未修复之前，本报告不表示 FUA 源码收敛。

这只是独立源码评审结论，不是编译通过、测试通过、三平台运行验收、完整 Node/Rust 等价、独立 ACCEPT 或发布批准。修复后的冻结 H 仍须取得适用 GitHub Actions 原生证据；不能将 H364 的部分成功或本报告冒充新 H 的验收。

## 评审身份、设置和限制

- Reviewer：/root/fua_general_r2；fresh independent general/security/correctness lens。
- 时间：2026-11-03；主要源码读取时钟为 02:56:51 UTC，其后完成 Windows drive-root 边界补充复核。
- Effort 选择：xhigh。原因：URL 与 Windows 路径归一化、安全准入、callback lifetime、消费者集成，以及前轮存在真实 IMPL_BUG。
- 实际 adapter 设置：root 告知其成功调用 spawn_agent，传入 fork_turns="none"、reasoning_effort="xhigh"，未传 model override。因此记录为 **root host adapter accepted explicit xhigh; configured model inherited, no model override**。本 worker 没有更深层 runtime setting 的自省接口，不把提示词或模型自述当作额外运行时证明。
- 模型：保持配置模型；本会话标识为 GPT-6 Astra Pro，未切换到其他模型。
- 只读：没有编译、测试、lint、schema check、仓库验证器或任何本地/htch 产品代码、URL parser 执行；没有 dispatch、push、发布、Git index 写入或产品/账本/schema/锁/工作流修改。
- 独立性：没有读取其他 reviewer 的报告、R1 review 文本或其他 lens 的结论。读取的 recovery JSON 和失败原始日志属于指定修复的源码/执行事实，不是其他评审结论。
- 补充事实来源：初稿后 root 提供了 published url 对 bare Windows drive 的源码线索；我重新读取冻结 parser、path_segments、Windows converter 与本轮 guard，独立确认下述 finding。没有读取 root 所提另一个 LFA candidate 的实现或评审，也没有将本报告范围扩展到该候选。
- 唯一写入产物：本报告。

## 冻结身份与完整范围

| 字段 | 身份 |
| --- | --- |
| Repository | bhrumom/fabushi-desktop |
| Branch | codex/openclaw-rust-fusion-chat-20261101-writer |
| Task | 647e0932-bea4-4113-9a61-65ae3db58955 |
| Fixed OpenClaw U | 1440bb1771372334bea23d7642bb52deb7db1845 |
| Fixed U tree | 90c8aab39972da11cfad220361781d49e1bc6700 |
| Complete-scope base | 947e980bd3c9dcb06535127f5cedfd193b83f0fc |
| Base tree | 686109128a616195e65d1c9018ccd5b0b9d2cc8a |
| Starting HEAD | 073b98b4826c8125d0a42d15f8ccdbfebb5eb488 |
| Prior published subject | 3647531d1d47d1c690b98a06f0156ae2734ff7ad |
| Prior review candidate tree | 27b6ac5b5e90023dff847e73e8c564743dd8e109 |
| R2 candidate tree | a2b9d3a154145d8071cc63909a5e6a20feeb7731 |
| Manifest | /workspace/scratch/7d27ea239237/file-url-r2-manifest.json |
| Snapshot | /workspace/scratch/7d27ea239237/file-url-r2 |
| Base snapshot | /workspace/scratch/7d27ea239237/file-url-r1-base |
| Prior snapshot | /workspace/scratch/7d27ea239237/file-url-r2-prior |
| Complete diff SHA-256 | f3823c3d6b987a26c51d9e60db154646e3fc8ec8a6577b106f2b476467c304d4 |
| R2 fix delta SHA-256 | cab8604963c876680fb3f28e39c1e7c9d3b26b93fe550237a7e9bc7f5355da0f |

我通过 GitHub GET 独立回读 candidate tree：10,477 entries、truncated=false。八个声明路径的 mode/blob/size 与 manifest 相符；本地八个文件又分别核对 Git blob 和 SHA-256。两份 patch 的 SHA-256 也独立核对相符。读取范围采用相同冻结字节，没有用可变分支头替代 candidate。

| 冻结路径 | Git blob | 本次读取深度 |
| --- | --- | --- |
| projects/openclaw-rust/docs/specs/media-file-url-admission.md | 588edd544316bea275a86294c88bb75d1c608e54 | 全文；FUA-01—06、O01—05、R2 recovery 与非目标 |
| projects/openclaw-rust/evidence/media-file-url-admission-647e0932-20261103.json | 95b07bf1b78ef149b7dabb5ed2926693247219fd | 全文，历史 scope/evidence/acceptance 限制 |
| projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json | fff6b93efd3e1b90184c09a2dcfa41c6cb95362f | 全文，失败身份、原因、保留/失效证据及修复方向 |
| projects/openclaw-rust/parity-ledger.json | df9cba558cccc210b35a811e9b5f9f3f8817cd79 | 全部变更 hunks；OCRM-RESP-MEDIA-WEB-PIPELINE-001 完整相关 row，15892—16025 |
| projects/openclaw-rust/requirements-traceability-matrix.json | 1d74ab6a725b6bab93428ce571d16944a05afa92 | 全部变更 hunks及 R24/AC24 相关 evidence refs 上下文 |
| projects/openclaw-rust/scripts/capture_media_cleanup.py | 7eed829018250ca297538df614f3929ea5a09058 | 全文 1—336；只读源码，未执行 |
| source/host/src/extensions/attachments/web_media.rs | 1262d6992264fa9b6893146ac12ae659f12d95c3 | 全文 1—647；准入、local body、全部 owner tests 深读 |
| source/host/tests/openclaw_web_media_contract.rs | e0f3afb0415b3f586710fc54ba929f5ad9165cb5 | 全文 1—836；实际 service、native fixture、原有 HTML/image/root 控制 |

以上对应 SHA-256（顺序与表一致）：

1. 224bc6bfc8a48d33cb6800eed4c5d29645b89434e851853e15e0c61e4027b2c9
2. 22f0bde2c4d2425cb41e27914f9c53b8be06d1375fc89c0d74780faffa8d6a3f
3. 1ec2716fd29f894d845be58bd1fb1157ce3ebb52a25eea5c584b0c3e991c646c
4. 22e7c7b03060d8701591ceea70d51d04a8462d7f719b785af0ffe9f7f61a198e
5. c11ee831f28be316acf7f9fed23d66edb9f2f4e501920f9029de49fe29210b36
6. 6b35213e3e236ce133642c66b7e3d03334acc23cef3a5285fe924a608c4f8d9c
7. 586611c5dc3c55f2edb5b6303bf95871d71f328646d3e94b01628f9e40c0e5be
8. c3f70c5254de3b31dd91bf6388838fdfb06fe865165ba8c2ad12ec99b8d51ac2

## 实际读取的规则、调用点与依赖

### 仓库规则和相关合同

完整读取 root AGENTS.md，本地 Git hash 与 candidate tree 都为 e4682748087759b2d4bb2b3a47916b6148e8a6f2。candidate tree 中另一个 nested AGENTS 仅位于 source/mahayana/codex-rs/tui/src/bottom_pane，不支配本范围。

读取了 OCRM SOURCE_OF_TRUTH 的权威/固定来源入口、主规范现行 §17 全节、full-fusion-batch.md 全文、module-map 的状态合同与 M23 owner 映射，以及 upstream.lock.json 全文。上述辅助 checkout 文件的 Git blob 均与 candidate tree 对齐。现行局部行为合同使用本轮 FUA-v1 全文；没有将历史 discovery SHA 或文档中的旧快照当成当前源身份。

### 相邻生产与测试消费者

| 实际读取路径 | Candidate blob | 已读范围/用途 |
| --- | --- | --- |
| source/host/src/extensions/attachments/attachments_service.rs | 7cfeff678c334320515de0bc01200bdb6af39379 | 1915—2050，尤其 2006—2021：loadWebMedia 参数、默认 options、真实 loader 调用、错误传播和序列化 |
| source/host/src/extensions/attachments/extension.rs | 1662b752a4a0366d11079c25d424ba121a5a3eee | 全文；extension 向既有 service 的委派 |
| source/host/src/extensions/attachments/media_reference.rs | acbed906d06b496b481fd4d4b5c604d9fc3eda57 | 98—178、275—340；分类上下文、metadata/open/有界读取实际副作用 |
| source/host/src/gateway_protocol.rs | 9eba2275f97ebaa9b30ac057f76dc610e862d5e7 | 全文；finite registry admission |
| source/host/src/host_gateway_api.rs | d4d4c4f5ca100138be864bf7ca0898b10793ac45 | 全文；宏生成的注册表及 owner 函数，没有 loadWebMedia |
| source/host/tests/openclaw_image_ops_contract.rs | f7230c34879341a9126e52b38bc1e2621db3b083 | 396—491；实际 loader 的 PNG 前缀、bytes/root/cap/retry 相邻控制 |
| source/host/src/connectors/channel_attachment.rs | 356e901feb2328729f882afe487fbe0199f4a40a | 全文 1—92；确认它是另一条本地路径解析/读取消费者 |
| source/host/src/extensions/attachments/media_file_context.rs | 2372f03102c106fdb93d3c5a50e71a74c80e55b3 | 140—215；channel 调用的独立 resolve_local_media_path |
| source/host/src/runner/tools/send_message_tool.rs | 08bfbe0bedc8138bb00804b5fbbd3b5ce3b046f2 | 1—90；独立 file_path_from_file_url/identity_attachment_source 边界，非完整 SendMessage 审计 |
| source/host/Cargo.toml | 9b477d438dd0a9325cbe52b670c2ada774f352e8 | 全文，既有 url 依赖 |
| source/host/Cargo.lock | 9895ce7752768b1c8d3fe107ef95e513457ca518 | url package/version/checksum/dependencies |
| source/host/app/Cargo.lock | 5f929af729ac7b537e054283c5977cc269be38ad | url package/version/checksum/dependencies |
| .github/workflows/rust-desktop-runtime.yml | c653192ddc9107a213a2f72e73d9548f99dffbd4 | macOS/Linux/Windows job 入口、exact-head bind 与 capture/upload 接缝；75—96、151—165、3067—3088、3135—3149、4961—5005 的相关片段 |

辅助 checkout 的 host_gateway_api.rs 和 workflow 与 candidate blob 不一致，因此这两项使用 GitHub exact blob 读取，不使用本地副本作结论。其余表列本地源读取逐项核对 candidate blob。另做了 Rust 引用文本定位；定位本身不代替上表实际调用链阅读，不宣称完成全仓库消费者穷举审计。

### Fixed U 与实际依赖

通过 GitHub exact-U 读取：

- src/media/web-media.ts，blob f48bca504f6fb77c3d094864eb656cb2b1bf25e2：1—70、530—562、638—680；确认实际导入 @openclaw/fs-safe/advanced、file-scheme 选择、转换失败类型化以及 resolved-path/read ordering。
- src/media/web-media.file-url.windows.test.ts，blob bdaedcde111ea2880b96d9afa547a9c1d2a36b42：全文 1—48；native uppercase single-slash 与原有拒绝控制。
- package.json，blob 6989c0618742d3c2b93cc336cbe1d9c3c4f0a657：2404—2416。
- pnpm-lock.yaml，blob 83b4975eb96942ec007a2eb9847ccfe98d380bda：288—298、5154—5157。

本轮审阅期间取得了额外的、可独立核对的发布包原文，记录如下；这不改写冻结 evidence 文本记载的此前获取状态：

- url-2.5.8.crate：SHA-256 ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed，与两份目标 lock checksum 一致；archive .cargo_vcs_info.json 指向 d6ea13c5f8e7e6e627f6390161b3e185bda5e5ce。
- 发布包 src/parser.rs：archive member 与 selected 原文 SHA-256 都为 2dd4210e1773fc5d3fdc15ce506ecff30dbf8eb358a79bbbc2835cc30d513c3a，Git blob dbdf9b906238c20ef0896d114d5cb8ad69197e34。实际深读 Input trimming/iteration、scheme/file/host/path parsing、drive-host normalization、empty-segment stripping 和字符谓词：76—81、196—241、327—336、368—474、512—610、1046—1114、1148—1385、1745—1753、1768—1818。
- 发布包 src/lib.rs：archive member 与 selected 原文 SHA-256 都为 51f47a93654528f3b8cb92c94fc26ad25775860d84197fff0c3124a687f8c5e3，Git blob f1558682b6287d0cba1cd93d84cee6cf6cbeaf44。实际读取 ParseOptions/callback/parse、Url::options、path_segments、Unix/Windows native conversion：252—335、616—638、1420—1465、2694—2757、3060—3193。
- fs-safe-0.27.0.tgz：SHA-256 02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c；独立计算的 SHA-512 base64 与 fixed-U pnpm-lock 的 SRI 完全一致。
- 读取该包 package/dist/local-file-access.js 全文，member SHA-256 7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8。读取的是实际导出的 file URL admission 依赖合同，不声称完整 package、生成器、所有 alias helper 或递归依赖闭包均已审阅。

以上操作仅为源码、archive member 与字节哈希读取，没有执行 Rust/Node parser 或产品代码。

## R2 修复与完整 FUA 范围的具体判断

### 1. 失败属于实现错误，修复作用在丢失信息之前

独立读取 H364 artifact 13200058555 的两个失败原日志，并核对它们的 SHA-256 与 recovery JSON 一致：

- file-url-admission.log：f01a32539a38becff9443ee28d4c55049bfd3d6f7e6a52d4eb06ca39682478da；705—715 明确显示 FILE:////files.example/share/image.png 的 reader entry 为 1，预期为 0，owner 4 passed / 1 failed。
- explicit-html-consumers.log：4e22d9aa0a716ab94a54449b98cf0096056339af222eac1156b66002ef23706a；791—805 显示原有四斜线 refusal 失败、转换结果为 /files.example/share/image.png，consumer 19 passed / 2 failed。

url parser.rs:1371—1378 会无 callback 地移除 file path 的开头空 segment；因此 R1 的 post-parse host 和 converted-path guard 不能重建原始四分隔符。web_media.rs:264—287 在 parse 之前检查这一信息，确实直接覆盖上述失败输入，并不会为 URL 失败建立新 fallback。

该 prefix 检查与所读依赖的相关词法规则相符：开头 U+0000—U+0020、忽略 ASCII TAB/CR/LF、ASCII scheme 大小写，以及 file separator 的 slash/backslash 处理。它只回答所声明的前缀拒绝条件；剩余语法仍交给结构化 parser。

### 2. host 丢弃回调与 lifetime 没有发现问题

web_media.rs:289—305 使用每次调用独立的 Cell<bool>，遇到 FileWithHostAndWindowsDrive 单调置 true。parser.rs:1344—1363 在丢弃非空 host 的同一位置同步报告该 violation。即使归一化后的 Url 已无 host，这次调用仍拒绝。

ParseOptions 在 lib.rs:255—315 借用 Fn callback，在 parse 返回前同步消费它；返回的是不携带 callback 借用的 Url。closure/Cell 均活过该次 parse，没有异步留存、跨线程共享、重试状态或下次请求污染。

localhost 在 parser.rs:1063—1075 已归一化为 HostInternal::None，因此正常 localhost + rooted Windows drive 不触发这个专用 violation。single-slash 所产生的 ExpectedFileDoubleSlash 等其他 violation 没有被笼统拒绝。这与保留正常 native / uppercase / single-slash / localhost 兼容的要求一致；bare drive 的 native conversion 是下述独立 finding。

### 3. URL 拒绝和路径拒绝均在本轮 reader 边界之前

web_media.rs:281—318 对显式返回的错误使用固定 invalid-file-url code 和固定文本：prefix、parse、scheme、discarded/nonlocal host、encoded separators、native conversion Err、converted network path。没有再执行原来的 slash stripping / lossy decoding fallback；没有把输入中的 username/password 拼入这些错误。需要注意，map_err 只能转换 native converter 返回的 Err，不能覆盖下述依赖内部 panic 或意外成功返回的 drive-relative path。

load_web_media:322—326 固定传入既有 local body；private helper 的 reader 参数没有进入 public options 或 Gateway JSON。helper:336—375 仅做字符串、Url 和 Path 操作，最后才调用 FnOnce reader。local_roots / sandbox_validated 的分支位于 local body:382—409，不能跳过此前的 URL admission。

实际 filesystem 副作用在该边界之后：root canonicalize 位于 web_media.rs:385、394；bounded reader 位于 412，并进一步进入 media_reference.rs:295 metadata、313 open 和 315 read。零 callback entry 控制与这段源码可以支持“所覆盖的拒绝输入没有进入本地读取阶段”，不能被描述为所有 OS syscall 的通用拦截证明。

### 4. Windows 本地路径与现有 local body 保持相应边界

web_media.rs:353—373 使用 platform prefix 区分 UNC/VerbatimUNC/device 与 rooted VerbatimDisk workspace；只有绝对 VerbatimDisk 获得这处 workspace local-form 例外。raw 双分隔符检查仍在 resolution 前，原始 raw-path 合同没有因此放宽。普通 native drive/Unix path 继续使用同一 reader。

本轮 complete diff 将原有 local body 抽到私有函数，没有改变 cap、roots、HTML byte policy、image handling 或结果构造；R2 delta 更没有改动该 body。原有 native path/bytes/fileName、explicit HTML、size-cap、retry 和 image consumer 断言仍保留。Unix authority-alias 控制仍在真实 loader/service 下验证旧 fallback 不能恢复一个被拒 URL。

这些判断不宣称 junction/symlink、opened-handle identity、Windows filesystem aliases 或默认读权限已经解决。

### 5. 消费者与证据没有借本轮修复扩权

真实 AttachmentsService:2006—2021 继续构造既有 options、调用 load_web_media，并把 typed error 转为字符串返回；没有新增 public reader 注入或权限字段。当前 exact candidate 的 host_gateway_api/gateway_protocol 仍不注册 loadWebMedia。service success/refusal 控制不能冒充 HTTP/UI/Runner/channel 的公开接线。

consumer tests:178—319 保留原 refusal，并使用 native fixture 更强地断言完整路径、原 bytes、Unicode/space/percent fileName、上限前后一字节和错误后的有效调用。R2 只在已有表内补充五斜线、混合分隔符、忽略控制字符、远端 host + drive/pipe/IP/dot-drive 变体，没有删除原失败断言。

capture:166—195 保留 exact required names、唯一性、summary、command/capture exits 等 fail-closed oracle；289—313 在既有三平台载体中选择 owner/consumer 控制。79/79/76 是选定测试名字数量，不是修复后运行结果。

ledger/RTM 的本轮 recovery 字节未变；完整 FUA diff 仅增加局部行为、invariants、refs/tests 等说明，相关 row 仍为 verification=not-run、wiring=not-wired、acceptance=pending，历史 read 字面事实未被重标。新增 recovery evidence 保留了 H364 失败，未把先前 source convergence 当运行验收。

## 必须保留的范围界限与未验证项

1. **Loader selector 与 converter 的词法范围不同。** web_media.rs:340 的原有 selector 是清理后 ASCII case-insensitive literal file: 前缀；converter 的 parser/prefix guard 可以处理更宽的 C0/embedded-TAB scheme spelling。consumer table 中包含 leading-C0/scheme-TAB 的纯 converter 负例，不等于 service/loader 已对每种这类别名做零 entry 证明。owner 新增 TAB/CR/LF separator 变体仍有 literal file: scheme，因此确实经过本次 admission。固定 U 的实际 loader 也使用 /^file:/ 选择。现行 spec 明确保留完整 Node/Rust URL alias parity 为 open scope；我没有据此虚构本轮承诺，也没有宣称 loader 全面识别 parser 的全部 URL 别名。
2. **其他本地 URL 消费者仍独立。** channel_attachment → media_file_context、SendMessage 的 file_path_from_file_url 不是本次 load_web_media 的调用链。已读取足以确认分离的实际源码，但没有完成其完整权限、来源与读取链审计，不将本轮 guard 的结论外推到这些消费者。
3. **权限与文件身份未闭合。** 默认/session roots、safe-reader capability、sandbox boolean 的既有含义、canonicalize-to-open race、symlink/junction/namespace aliases、HTML generated provenance、完整 codec/MIME 和全体媒体路径仍属于明确开放范围。
4. **依赖获取与等价性分开。** 本次已独立核对 exact url crate 和所读 members，故可以新增“已取得并核对”的来源事实；这不证明完整 Rust/Node alias parity、依赖生成/法律/递归闭包或全部 package source 已读。
5. **运行验证未执行。** 没有运行修复候选，也没有审计整个 Actions artifact ZIP、全部 CI 日志或最终 acceptance。H364 的失败原日志只能支持故障诊断；旧 Windows 成功和 queued/缺失平台不能证明 corrected H。既有 native positives、四分隔符原断言、新增 host-discard 变体、各平台 error/cap/retry 控制仍须在修复后的 exact H 上取得正面证据。
6. **全文阅读声明有界。** 八个范围文件按上表完成对应深度；大 ledger/RTM 只读全部变更和相关 row/context，没有宣称其余全量内容都已读。workflow 只读相关 native capture/identity/upload 接缝；其他 unmodified 仓库实现、并发 evidence archive 内容、其他 review 报告和最终 migration gates 不在本评审的全文阅读声明内。

## Finding 清单

### FUA-R2-GEN-01 — Windows bare-drive file URL 在 native conversion 内 panic，或作为相对路径进入 reader

| 字段 | 内容 |
| --- | --- |
| Severity | P2 / Medium；应在声称 FUA-02/FUA-O01 conversion refusal 与对应 reader admission 完成前修复 |
| Affected path / lines | source/host/src/extensions/attachments/web_media.rs:313—317；后续 loader guard 353—375 |
| Symbol | file_url_to_path → Url::to_file_path → load_web_media_with_local_reader |
| Trigger | Windows 上调用 file_url_to_path("file:///C:")，或通过现有 load_web_media / AttachmentsService::dispatch_gateway 传入同一引用 |
| Evidence type | exact frozen source + checksum-matched published dependency member 的直接控制流推导；没有执行该输入 |

**具体因果链：**

1. 该输入只有三个前导 file separator，所以通过 web_media.rs:264—287；它的 host 为空，也不包含 encoded slash/backslash，因此通过 299—310。空 host 不会触发新加的 FileWithHostAndWindowsDrive callback。
2. published parser.rs:1344—1356 在末尾 drive segment C: 上仅在 ends_with_slash 为 true 时补 slash；本输入没有该 slash。1371—1378 只保留开头根 slash，得到 URL path /C:，不会生成 /C:/。
3. published lib.rs:1442—1444 把 /C: 变为只有一个 C: 元素的 path segment iterator。Windows converter:3139—3148 接受第一段 C:；因为没有后续段，3168—3175 不会加反斜线。
4. converter:3186—3191 因此构造 Windows drive-relative PathBuf("C:")。启用 debug assertions 时，3187 的 path.is_absolute() 断言失败；没有 Err 返回给 web_media.rs:315 的 map_err。关闭 debug assertions 时则返回 Ok(C:)。
5. 发布 build 的这一 Ok 路径同样不被本轮 guard 拦截：file_url_to_path 的 network-prefix check 对 C: 通过；loader 把 Prefix::Disk(_) 标为 false，随后字符串 network check 也通过，最终进入 375 的 local_reader。带 roots 时可以先执行 canonicalize；不带 roots 时可以进入 bounded reader 的 metadata。它不是要求的 absolute native local path refusal。

**影响：** Windows 调用方不能可靠取得声明的 typed invalid-file-url；启用 debug assertions 的执行会 panic，关闭时仍可能对依赖进程/驱动器当前目录语义的相对路径发起文件系统访问。这里不主张能够读取任意文件、发生网络访问或泄漏内容；实际影响证明限于 panic、错误合同及不应进入的 reader/filesystem 边界。

**范围归属：** 这不是由 R2 callback 新引入的回归，旧 native conversion 也有这类依赖边界。它位于完整 FUA-v1 已修改并明确负责的 file URL/native-conversion 函数内，对应 FUA-02、FUA-O01 及“转换为绝对本地路径”的准入合同，不属于需要完成所有 Windows alias 才能成立的广泛等价性诉求。最小触发仅需标准 file:///C:。

**具体修复建议：** 在 Windows native conversion 调用之前，用所解析 path 的结构拒绝 bare drive / 缺 root-separator 的形式并返回现有 invalid-file-url；不能只在 to_file_path 返回后检查 is_absolute，因为 debug assertion 已先发生。转换成功后也可保留绝对路径不变量作为防御。保留 file:///C:/ 和现有 native temp、single-slash、localhost rooted positives 的行为；不要通过 panic catch、修改依赖断言或静默补 slash 改变一个不完整路径的目标。

在已有 Windows converter 和 zero-reader-entry 表中加入 bare-drive negative（以及同一已归一化形态的 localhost 变体），断言 typed code、零 entry 和失败后仍可处理有效 native URL。维持已有原断言、byte/root/cap controls；由后续 exact-H Windows Actions 给出执行证据。

**当前状态：OPEN，未修改代码。** 本报告不批准 root 所提的其他候选或扩大后的生产 batch；若修复涉及共享 owner / 实际消费者，必须按新的完整冻结范围再评审。

执行和接受状态仍由修复后 exact-H GitHub Actions 及其适用验收合同决定。
