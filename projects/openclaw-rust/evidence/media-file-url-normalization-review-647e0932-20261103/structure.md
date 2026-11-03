# FUA-v1 R2 — reuse / structure / performance independent source review

日期：2026-11-03 UTC。结论：**发现 1 项 P2 的既有转换合同缺口；本轮 normalization 修复本身未发现新的复用、结构或复杂度问题。不能把本报告作为 R2 整体收敛或运行验收。**

## 执行设置与边界

- 本 worker 是独立只读的 reuse / structure / performance 三 lens。根 adapter 回告，实际 spawn 字段为 `model="gpt-6.1-sol"`、`reasoning_effort="xhigh"`、`fork_turns="none"`，工具已接受；没有将提示文字冒充设置。选择原因是 URL/文件权限边界、跨平台规范化和前次 sound `IMPL_BUG`。
- 先读根 AGENTS 全文（395 行），Git blob 本地只读核对为 `e4682748087759b2d4bb2b3a47916b6148e8a6f2`；完整读当前 FUA spec（157 行）。另读 OCRM Source of Truth 当前 authority/约束部分、主 migration spec 当前 §17、module map 的有关 owner 导航。
- 没有读取任何其他 review 报告或 prior runner verdict。前次失败事实由当前 spec/recovery record 和原始 Actions 日志核对，未把历史源审查视作运行通过。
- 仅进行了源码/文本/证据读取、Git 只读对象操作、字节 hash 和 tar 成员读取；未运行产品 parser、构建、测试、lint、schema、generator、benchmark 或任何本机/htch 可执行验证；未改源码、账本、schema、锁、workflow 或真实 index。唯一写入是本报告。

## 冻结身份与实际核对

仓库 `bhrumom/fabushi-desktop`；分支 `codex/openclaw-rust-fusion-chat-20261101-writer`；固定 upstream U `1440bb1771372334bea23d7642bb52deb7db1845`，tree `90c8aab39972da11cfad220361781d49e1bc6700`。

- 完整 scope base：`947e980bd3c9dcb06535127f5cedfd193b83f0fc`，tree `686109128a616195e65d1c9018ccd5b0b9d2cc8a`。
- starting HEAD：`073b98b4826c8125d0a42d15f8ccdbfebb5eb488`；本地 `git rev-parse HEAD` 核对相同。
- prior published H：`3647531d1d47d1c690b98a06f0156ae2734ff7ad`；prior snapshot tree `27b6ac5b5e90023dff847e73e8c564743dd8e109`。
- 本轮 candidate tree：`a2b9d3a154145d8071cc63909a5e6a20feeb7731`。该 tree 身份及 GitHub 非截断八 path 回读来自冻结 manifest/runner 提供的 scope；本 worker 未另做远端 tree API 回读，但独立核对了八份快照的全部 Git blob 与 SHA-256，均逐项等于 manifest。
- manifest：`/workspace/scratch/7d27ea239237/file-url-r2-manifest.json`。
- snapshot：`/workspace/scratch/7d27ea239237/file-url-r2`；base snapshot `file-url-r1-base`；prior snapshot `file-url-r2-prior`。
- complete diff SHA-256 独立核对：`f3823c3d6b987a26c51d9e60db154646e3fc8ec8a6577b106f2b476467c304d4`。
- focused fix delta SHA-256 独立核对：`cab8604963c876680fb3f28e39c1e7c9d3b26b93fe550237a7e9bc7f5355da0f`。

八 path 的 Git blob 核对结果：

| Path | Candidate blob | R2 delta |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `588edd544316bea275a86294c88bb75d1c608e54` | 是 |
| `projects/openclaw-rust/evidence/media-file-url-admission-647e0932-20261103.json` | `95b07bf1b78ef149b7dabb5ed2926693247219fd` | 否 |
| `projects/openclaw-rust/parity-ledger.json` | `df9cba558cccc210b35a811e9b5f9f3f8817cd79` | 否 |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `1d74ab6a725b6bab93428ce571d16944a05afa92` | 否 |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `7eed829018250ca297538df614f3929ea5a09058` | 否 |
| `source/host/src/extensions/attachments/web_media.rs` | `1262d6992264fa9b6893146ac12ae659f12d95c3` | 是 |
| `source/host/tests/openclaw_web_media_contract.rs` | `e0f3afb0415b3f586710fc54ba929f5ad9165cb5` | 是 |
| `projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json` | `fff6b93efd3e1b90184c09a2dcfa41c6cb95362f` | 是 |

相邻源码从 starting H073 的只读版本判断。共享 checkout 的 `host_gateway_api.rs`、`gateway_server.rs` 有 scope 外 WIP，发现后改用 `git show 073b98…:<path>` 读取冻结对象，未采用其 mutable 版本作候选结论、未触碰这些 WIP。其他实际引用的相邻文件通过 `git diff --name-only 073b98… -- <paths>` 核对无本地差异。

## 具体 findings

### FUA-STRUCT-R2-01 — P2：Windows drive-only URL 在 native conversion 前缺少绝对路径前置条件

**位置：** candidate `source/host/src/extensions/attachments/web_media.rs:313–317`（`file_url_to_path` 对 `Url::to_file_path` 的调用）；相邻继续接纳该结果的位置是 `web_media.rs:355–375`。这项问题不是 R2 normalization delta 新引入的回归；base H947 的转换函数已直接调用同一依赖（base 文件 286–288 行）。它是本次完整 FUA scope 所改写的 converter 仍保留的具体合同缺口。

**触发：** 原生 Windows 上调用 `file_url_to_path("file:///C:")` 或加载同一引用；`file:///C%3A` 是同类 encoded-colon 变体。三条 leading separators 不触发新增 guard；empty host 不触发 remote-host/callback 拒绝；path 不含 `%2f`/`%5c`。前置条件均通过后，函数进入锁定依赖的 native converter。

**源码因果链：** exact `url` 2.5.8 archive 的 `src/parser.rs:1220–1238,1344–1378` 将此类输入保留为单一 drive path segment；空 host 不会产生 `FileWithHostAndWindowsDrive`。`src/lib.rs:1442–1444` 将 `/C:` 拆成唯一的 `C:` segment。Windows converter `lib.rs:3139–3165` 直接向 String 写入 `C:`（encoded-colon 的四字符分支也写入 `C:`），只有后续 segment 才会在 `3168–3175` 添 `\\`。没有后续 segment 时，`3186–3191` 构造 drive-relative `PathBuf("C:")`，随后执行 `debug_assert!(path.is_absolute())`。

因此，启用 debug assertions 的 Windows 构建可能在 dependency 内 panic，`map_err` 没有机会返回 FUA 的 `invalid-file-url`；禁用该断言时则返回 drive-relative path。后续 final guard 的 `Prefix::Disk(_) => false` 只走字符串 network 检查，不要求绝对路径，随后可以进入私有 reader。此时根检查/metadata 所见对象依赖 Windows 的 drive-specific current directory，而非 converter 声明的 native absolute path，且错误不再必然是 URL admission error。这里没有声称已实际运行、读到文件、建立远端连接或证明进程级崩溃；这是直接源码路径判断。

**合同：** FUA spec 33–38 行的相同 native file/error 行为，FUA-02 的 parse/native-conversion 失败保留 `invalid-file-url` 与 legitimate native absolute 路径合同，以及 FUA-O01/O02 的 conversion-invalid/零 reader entry 控制。当前 error 文案自己也声称 conversion 产出 absolute local path。

**建议：** 在调用 `to_file_path()` **之前**，建立 Windows 本地 file URL 的 native absolute drive/root 前置条件，覆盖 parser-normalized drive-only 及 encoded-colon 情形，并返回同一 `invalid-file-url`；仅在转换之后增加 `is_absolute()` 无法挡住依赖内 debug assertion。应由既有、合同明确的转换 owner 实现这一前置条件，保留 `file:///C:/`、native filename/Unicode/localhost 等合法 rooted positives。新增原生 Windows 的纯转换和私有零-reader-entry断言，要求 exact code；在 Actions 验证拒绝不 panic、rooted positives 仍有效。若采用已有共享 converter，需要另外阅读其全部受影响消费者再审；本报告没有审查或批准任何未读的 LFA/其他候选。

本项计为 **1 个 P2**；没有另列仅属偏好的重构意见。

## 已直接阅读的生产链与结构判断

- **候选 owner：** 完整读 `web_media.rs:1–647`，重点为新增 guard/callback `261–318`、纯 admission `321–375`、existing local body `378–481` 和 owner tests `483–647`。完整 scope 中旧手工 slash-stripping/lossy fallback 已删除；R2 没有恢复 fallback，也没有新增依赖/runtime/parser owner。
- **唯一生产 reader 边界：** `load_web_media` 仍固定传入 `load_local_web_media`（326 行）；private `FnOnce(&Path,&WebMediaOptions)` 接口没有出现在 public options、serde 或 Gateway args 中。admission 的 trim、URL parse、Path join/components/is_absolute 都是纯值操作；源代码中 canonicalize、bounded read 及其 metadata/open/read 均位于 callback 后。同步回调至多调用一次，原 options 按引用传递。这里确认的是这条 loader 的边界，不是全产品 syscall 或 safe-reader capability 完成。
- **既有 tail：** roots/sandbox 判定仍在 `378–409`，cap 与 bounded reader 在 `411–421`，原 filename/HTML byte-policy/return 在 `423–455`，image probe/optimization/result 在 `458–479`。完整 diff 表明提取边界后的这些业务段无 semantic hunk；R2 fix delta 完全未动 tail。直接读了 H073 `media_reference.rs:291–324`：metadata 是 file/cap 检查，随后 open 和 bounded read，增长超过 cap 仍返回错误。没有把这些旧实现的身份/race或 sandbox 缺口记作本轮已关闭。
- **实际 direct service：** 读 H073 `attachments_service.rs:1990–2030`，真实 `loadWebMedia` arm 在 `2006–2022`，只构造已有 `WebMediaOptions`、调用此 loader、传播 typed error 的字符串；没有 public injected reader、roots/capability 参数扩大。该文件 blob 为 `7cfeff678c334320515de0bc01200bdb6af39379`。
- **实际 composition：** 读 `extensions/attachments/extension.rs:1–85` 和 `host_production_extensions.rs:1600–1646`；production 仍构造原 AttachmentsService、通过 HostAttachmentsExtension 转发。没有由 FUA 产生第二个 service、worker、持久状态或权限真相。
- **有限公开 registry：** 读冻结 H073 `host_gateway_api.rs` 的 registry 宏/所有方法成员及 `gateway_protocol.rs:1–28`，其中没有 `loadWebMedia`。前者冻结 blob `d4d4c4f5ca100138be864bf7ca0898b10793ac45`，后者 `9eba2275f97ebaa9b30ac057f76dc610e862d5e7`。又读 H073 `gateway_server.rs:1–125,1706–1711`，HTTP command 的 unknown-method 拒绝仍在实际入口。未将 direct service 测试升级成 HTTP/UI/Runner 接线验收。
- **另一个实际 consumer：** 读 H073 `connectors/channel_attachment.rs:1–91` 与其 `media_file_context.rs:167–229` resolver；channel upload 仍用它自己的返回 `Option` 的路径策略、reanchor 和 bounded reader。FUA 没有把它改接为新 loader，也没有复制它进 FUA。该 consumer 的既有差异、SendMessage 或其他 App sinks 没有因本轮 source review 得到通过。

## Normalization 修复、依赖合同与性能判断

`has_excess_file_url_separators` 只保留 parser 原本会丢失的 prohibited prefix 证据：逐字节跳过 leading C0/space，忽略 ASCII tab/CR/LF，大小写无关匹配 `file:`，计满四个 `/` 或 `\\` 就拒绝。它不建立新的 URL AST、不推导权限、不复制完整 parser。之后仍经过唯一的 `Url` scheme/host/encoded-separator/native-conversion/final-path chain。

`Cell<bool>` 与 `observe_violation` 是一次函数调用的 stack-local 值，没有缓存、global、持久化或共享线程状态。exact-crate `lib.rs:255–315` 中 callback 是借用的 `Fn`，Parser 同步执行且 `Url` 不保留 callback；`parser.rs:1360–1363` 在非空 host 被 drive normalization 丢弃前发该 violation。`parser.rs:1057–1079` 将 empty/localhost 设为无 host，所以这个 callback 不会仅因合法 localhost drive path 误拒。新判断只拒绝指定的丢 host violation，未把其他 URL syntax warnings 全部改为错误。

本轮新增前缀 guard 不分配；最多扫描被忽略的前导/间隔字符和固定数量有效前缀字节。启用 violation callback 会让 `Input::new_trim_c0_control_and_space` 对输入做额外的 ignored-character 检查，但整体仍为输入长度的线性扫描，额外状态常数级；没有平方循环、重试、重复读、文件 buffering 或额外大对象 clone。原 parsed-path lowercase allocation 与原 native converter 分配仍存在，不是 R2 新增的规模放大。private generic `FnOnce` 没有 Box/dyn allocation。未做性能执行或绝对延迟/CPU/内存数值声称。

依赖证据分清冻结文本与新获得事实：

- Host/App 两份 Cargo.lock 均锁定 `url` 2.5.8、checksum `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`；host Cargo.toml 仍为既有 `url = "2"`。
- 本 worker 在 runner 提供的 `/tmp/fabushi-647e-pairing/local-file-url-dependency/url-2.5.8.crate` 上独立核对 archive SHA-256 为该同值；只读提取 `src/parser.rs`、`src/lib.rs` 到 stdout 后 hash，分别为 `2dd4210e1773fc5d3fdc15ce506ecff30dbf8eb358a79bbbc2835cc30d513c3a`、`51f47a93654528f3b8cb92c94fc26ad25775860d84197fff0c3124a687f8c5e3`，与读取的 selected 成员完全相同。没有运行 crate。
- 因此，冻结 evidence 中“exact crate archive 未取得”是该文档当时的事实；此次后续只读获取把本报告使用的 API/parser/native-conversion 分支精确绑定到 lock archive。它不证明 dependency 全量闭包或全部法律/平台/行为已审查。
- 直接读 exact-crate parser 的 `210–240,327–342,360–400,512–580,1040–1112,1200–1379,1763–1814` 及 lib 的 `245–345,1430–1445,2694–2757,3050–3191`。除上述明确范围外未声称全 crate 深读。
- 另直接读固定 U `web-media.ts:530–562,638–688` 和完整 windows test 1–48，Git blob 分别独立核对为 `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`、`bdaedcde111ea2880b96d9afa547a9c1d2a36b42`；完整读实际 packaged `local-file-access.js`，SHA-256 为 `7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8`。原 fs-safe 依赖还有 Windows alias 等更广合同；未将本轮两类 normalization 修复说成完整 Node/Rust alias parity。

## Tests、恢复证据与声明检查

直接读候选 integration tests 的 fixture/service helpers、`162–320` 新旧 FUA controls，以及 owner tests 全段。原四 slash 断言仍保留在 integration 189 行、owner 527 行，R2 是往现有表增加 five-slash、mixed separators、ignored tab/CR/LF、leading C0/scheme-tab 与 remote-host drive 变体。owner 仍跨 roots None/Some、sandbox false/true 检查 exact typed code 和零 entry；positive 仍检查同一 path、options 指针和一次 entry，error sentinel 仍不重试。native bytes/service/cap-recovery controls 用原生 temp path 和 `Url::from_file_path`；Windows rooted verbatim workspace positive 与 UNC/device negatives 仍在。没有通过删除失败断言或改期待来恢复绿色。

但这些表目前没有 FUA-STRUCT-R2-01 的 drive-only/pre-conversion assertion，不能据已有正常 native file positives 推出该前置条件成立。

完整读 capture 脚本 `1–336`：新增 FUA group/平台名字进入现有 exact-name 唯一性、零 skipped、summary/command-exit/capture-exit oracle；原 13 个 synthetic controls 仍在；input before/after identity 和 exact HEAD/dispatch guard 保留。R2 没有改 capture 或其 oracle。没有执行这个脚本。

完整读两份 FUA evidence；ledger/RTM 阅读完整 diff 所有 hunks和 FUA row/status/refs相邻内容，未声称读完 1.4 MB ledger 的全部职责。`OCRM-RESP-MEDIA-WEB-PIPELINE-001` 状态仍为 verification `not-run`、wiring `not-wired`、acceptance `pending`；historical read-complete 没有被本轮改写。RTM 两处只是 evidence refs 增加。没有本轮 status/global flag 提升 hunk。

独立核对原 H364 Linux 日志字节：`file-url-admission.log` SHA-256 `f01a32539a38becff9443ee28d4c55049bfd3d6f7e6a52d4eb06ca39682478da`，明确记录四 slash reader entries `left:1/right:0` 和 `4 passed/1 failed`；`explicit-html-consumers.log` SHA-256 `4e22d9aa0a716ab94a54449b98cf0096056339af222eac1156b66002ef23706a`，明确保留原四 slash assertion、`/files.example/share/image.png` 错误结果及 `19 passed/2 failed`。与 recovery record 的 `IMPL_BUG` 因果分类相符。历史 Windows 的 passing 摘要没有在本 worker 中被重新验证，也不能迁移为 repaired candidate 结果。

## 缺口、未验证项与出口

1. FUA-STRUCT-R2-01 的 Windows native-absolute precondition 仍需修复、focused source re-review 和 Actions 因果用例；本报告不能给当前原八 path 候选总体无缺陷结论。
2. 原 H364 失败已保存；修复后的 candidate 没有本 worker 查看过的 exact-H Linux/macOS/Windows执行证据。源代码推理不等于 compile 或运行通过，缺/queued OS 不能计 pass。
3. 本轮只审原冻结八 path FUA scope和列明相邻链，不审后续组合批次、其他 LFA/应用 sinks、真实 SendMessage、设备/渠道或完整 App 接线，也不批准其发布。后续更大 scope 必须另冻完整 tree、diff/manifest，审实际 producer/owner/consumer。
4. 默认/session roots、safe-reader capability、sandbox boolean bypass、file identity/concurrent replacement、generated HTML provenance、MIME/codec全量等仍是 spec 的独立 open scope；当前私有无I/O seam不授予任何新读权限。
5. 精确 crate archive/所读成员绑定现已取得；full Node/Rust path/filesystem alias parity、完整 dependency/legal closure、G01–G12/AC01–AC24、独立 ACCEPT 与 shipping SHA acceptance 均未由本轮关闭。

本轮只读结构/复用/性能结论限于所列证据：**1 项具体 P2 遗留合同缺口；R2 前缀与 discarded-host 修复继续复用原 owner 和 parser，未发现其新增的结构或性能回归。**
