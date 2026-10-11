Live TDRP Revision 9 upstream authority (2026-10-11): telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (root tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6). Root non-directory=6,653; recursive non-directory=16,125; deterministic read-through=9,517; first unread=9,518 Telegram/ThirdParty/kimageformats::autotests/read/eps/bw.eps@3af28d5d28351a749cbcaf0e30e3652a159d830a; unread=6,608; unknown=15,845; omitted=0. Kcoreaddons direct-gitlink source is now exact-read through its component tail; next component is KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec.

### Revision 9 deterministic read-through — PIC/AVIF media acceptance 9,474–9,517

Orders 9,474–9,517 exact-read the next pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec` range. PIC fixtures and `pictest.cpp` establish raw/RLE compression, alpha/non-alpha output, deterministic encoded bytes, short/full-length Description metadata, long runs, input-format conversion, and pre-read text/size/image-format semantics. AVIF fixtures cover grayscale, grayscale-alpha, RGB/RGBA and the full auto-transform orientation matrix: unrotated/90/180/270 combined with horizontal or vertical mirroring all converge to the same canonical 480×160 reference orientation.

Binary fixtures were read by exact base64/raw bytes; PNG dimensions and selected reference visuals were inspected. Existing-owner-first keeps the canonical attachment/media, Resource and Media/Quality owners. The current production branch also bounds the existing local-image preview fast path before decode using `ATTACHMENT_PREVIEW_BYTE_CAP`; the strict Grok semantic-adaptation evidence lock is rebound to those intentional target blobs rather than weakened.

All 44 source rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; the bounded preview fix is narrower than full codec parity. Accounting is **9,517/16,125 read; 6,608 unread; 15,845 unknown; 0 omitted**. Next is 9,518 `Telegram/ThirdParty/kimageformats::autotests/read/eps/bw.eps@3af28d5d28351a749cbcaf0e30e3652a159d830a`. Fresh descendant exact-head GitHub Actions remain mandatory.

### Revision 9 deterministic read-through — kimageformats opening range 9,454–9,473

Orders 9,454–9,473 exact-read the opening range of pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`: repository/build policy, optional codec dependency gates, exact license corpus, declared read-only/read-write image capabilities, format acceptance topology, the ANI binary fixture plus all three frame PNGs, its decoder contract, and the bounded fuzzy image comparator. Binary fixtures were read by exact base64/raw bytes and the PNG frames were visually inspected rather than inferred from filenames.

The ANI contract preserves 32×32 dimensions, Title/Author metadata, four-frame custom sequencing, 166/333 ms delays, end-of-stream behavior and jump/reset semantics. Existing-owner-first keeps attachment/media, Resource, Build/Packaging and Release/NOTICE owners canonical. The current branch separately fixes the shipping local-image fast path so the canonical preview-byte cap is enforced before file decode; that narrow production fix does not mark this broader codec range verified.

All 20 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **9,473/16,125 read; 6,652 unread; 15,845 unknown; 0 omitted**. Next is 9,474 `Telegram/ThirdParty/kimageformats::autotests/long-runs.pic@620a3bb8eb322d06e0468eb641c56bd72b6c6545`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — kcoreaddons test/demo tail 9,445–9,453

Orders 9,445–9,453 exact-read the nine final non-directory blobs of pinned `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15`. They cover the optional Qt Widgets manual-test build, system-user face-icon discovery with missing-icon fallback and bounded scaling, directory-watch dirty/created/deleted events with explicit scan start/stop/restart plus multi-watcher file/subdirectory observation, deterministic seeded random-sequence/shuffle behavior, and a debounced plaintext-to-HTML demo surface.

Existing-owner-first keeps Build/Quality, platform user/profile Resource, Resource/filesystem-watch, security randomness/test utility and Transcript safe-rendering owners canonical. The Qt demo UI is replacement-only; system-user data is not Fabushi account identity; deterministic `KRandomSequence` behavior is never security authority for tokens/nonces/identity. This range corrects the exact-head CI defect that proved `src/mimetypes/kde5.xml` was not the component tail. The validator remains strict; no source entry is skipped.

All nine rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **9,453/16,125 read; 6,672 unread; 15,845 unknown; 0 omitted**. Kcoreaddons is now exact-read through all 377 non-directory entries. Next is 9,454 `Telegram/ThirdParty/kimageformats::.git-blame-ignore-revs@b34bba3cd8fb4c6785690422f5c46762d60bb7dc` in pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — kcoreaddons component tail 9,401–9,444

Orders 9,401–9,444 exact-read the remainder of pinned `KDE/kcoreaddons`: safe text/emoticon interface, legacy configuration migration, locale-aware byte/duration/date formatting, executable/open-file discovery, memory telemetry, `os-release`, cross-platform process enumeration, runtime-platform hints, sandbox host bridging, Unix/Windows shell parsing and quoting, signal integration, Unix/Windows user/group/SID semantics, and the final MIME corpus (57 types, 73 globs, 26 subclass relations, 9 magic rules, 3 aliases).

Existing-owner-first maps these into canonical Localization, Settings/Persistence, Resource/platform, Host/Runner/Computer, Permissions, Transcript and attachment/MIME owners. Critical boundaries stay explicit: migrations never overwrite existing target state; open-files reports typed unsupported rather than fake success; memory distinguishes free from available; OS user/group data is platform permission context, not Fabushi account identity; sandbox/shell helpers cannot bypass approval; MIME and rendered-link inputs stay validation-bound. No KDE runtime or second platform/settings/identity truth is introduced.

This also fixes an authority-accounting defect exposed by exact-head CI: `upstream.lock.source_disposition_evidence` had remained at 9,201 while master/index/ledger had advanced. The lock summary is now advanced from the same source prefix instead of weakening the validator.

Accounting: **9,444/16,125 read; 6,681 unread; 15,845 unknown; 0 omitted**. Kcoreaddons source reading is complete; next 9,445 is `Telegram/ThirdParty/kimageformats::.git-blame-ignore-revs@b34bba3cd8fb4c6785690422f5c46762d60bb7dc` in pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`.


### Revision 9 deterministic read-through — plugin/text/security utility source 9,381–9,400

Orders 9,381–9,400 exact-read static-plugin helpers, random/shuffle APIs, fuzzy matching/ranking, localized JSON lookup, macro expansion and platform shell quoting, string utilities, and bounded plain-text → HTML/linkification. The security disposition is explicit: upstream KRandom itself says it is not suitable for serious randomness, so it is never authority for Fabushi tokens/nonces/identity; macro expansion remains behind canonical Host/Runner argument, quoting, approval and sandbox boundaries; text rendering must validate/bound links and escape entities under Transcript/Resource safe-link policy.

Existing-owner-first maps fuzzy scoring to canonical Search, localized JSON to Localization, shell-aware expansion to Host/Runner/Computer, and linkification/text projection to Transcript/Resource. Static Qt plugin registration remains non-shipping implementation detail. Rows remain mapped-open / unknown_closed=false until production and same-head evidence close them.

Accounting: **9,400/16,125 read; 6,725 unread; 15,845 unknown; 0 omitted**. Next 9,401 `Telegram/ThirdParty/kcoreaddons::src/lib/text/ktexttohtml_p.h@93bbb72f1eeb0adc8c2ec57ebc6ab5c3309d197d`.


### Revision 9 deterministic read-through — licenses and plugin lifecycle 9,361–9,380

Orders 9,361–9,380 exact-read the KCoreAddons runtime-version contract, installed license corpus, legacy desktop metadata parser, plugin factory/loader and typed plugin metadata. Applicable semantics include runtime-vs-build provenance, exact third-party license availability, strict typed metadata conversion, service-type/path resolution with malformed-input failure, concurrency-safe parser caching, stable plugin/source identity, load/error/lifecycle separation, MIME inheritance, default-enable state, localized metadata fallback and author/license/version provenance.

Existing-owner-first audit found stronger shipping equivalents in `source/packages/cursor-plugins/manifest-parser.ts` and Mahayana `core-plugins`: manifest size is bounded, plugin/schema/path/id data is typed, invalid/duplicate entries are fenced, variables are recursively validated, and loader/store/cache/version lifecycle stays under the canonical Marketplace owner. Therefore no KDE/Qt plugin runtime or second schema truth is introduced. These rows remain `mapped-open` / `unknown_closed=false` until full applicability and same-head production evidence close them.

Accounting: **9,380/16,125 read; 6,745 unread; 15,845 unknown; 0 omitted**. Next 9,381 `Telegram/ThirdParty/kcoreaddons::src/lib/plugin/kstaticpluginhelpers.cpp@1bf196d1b85d1969f0fd70a10e5023b4a4a5522c`.


### Revision 9 deterministic read-through — portal/job/provenance 9,345–9,360

Orders 9,345–9,360 exact-read the XDG file-transfer/KIO-fuse interfaces, KJob/KCompositeJob/tracker/UI-delegate lifecycle, and KAboutData/core version source. Applicable product semantics include explicit portal Start/Add/Retrieve/Stop/closed lifecycle; bounded resource ownership; exactly-once task terminal signals; kill/suspend/resume capabilities; first-error propagation; explicit subtask ownership; progress/speed observability; and typed SPDX/component provenance distinct from Fabushi product identity.

Existing-owner-first maps these to canonical Resource/attachment + platform portal, Task/Coordinator + Task UI, and product identity/NOTICE owners. No KDE Job framework, KDE UI delegate, KDE product identity or DBus runtime is introduced as a second owner. Rows remain mapped-open until implementation/evidence obligations close.

Accounting: **9,360/16,125 read; 6,765 unread; 15,845 unknown; 0 omitted**. Next 9,361 `Telegram/ThirdParty/kcoreaddons::src/lib/kcoreaddons.h@77b482d1b75330c1e38908a7d9c974fced309dd7`.


### Revision 9 deterministic read-through — kcoreaddons production source 9,309–9,344

Orders 9,309–9,344 exact-read the first 36 blobs of pinned `KDE/kcoreaddons/src`. The window covers build and localization extraction, strict desktop-metadata → typed JSON conversion, shared derived-cache versioning/eviction/corruption recovery and process-safe locking, stale autosave/backup ownership, create/delete/recreate/move-aware filesystem watching, filesystem/network-mount classification, collision-safe file naming, typed diagnostics, child-process argv/shell/environment/timeout semantics, and URL/attachment transfer through local/portal boundaries.

Existing-owner-first mapping keeps Fabushi build/packaging, Plugins/MCP/Marketplace, Persistence/Recovery, Resource/platform, Settings, Host/Runner/Computer and attachment owners canonical. The recently added Marketplace cache-hit recovery closes only a narrow corrupted-derived-cache behavior and is not used to overclaim the broader shared-cache rows. No KDE/Qt runtime, second cache truth, second process owner, second Settings owner or second attachment owner is introduced. All rows remain `mapped-open` / `unknown_closed=false` until their production/evidence obligations close.

Accounting is **9,344/16,125 read; 6,781 unread; 15,845 unknown; 0 omitted**. Next is 9,345 `Telegram/ThirdParty/kcoreaddons::src/lib/io/kurlmimedata.h@15bd05e4bc36d6c5b377029c695bb54b876f2332`. Fresh descendant exact-head GitHub Actions are mandatory.


### Revision 9 deterministic read-through — kcoreaddons localization resources 9,202–9,308

All 107 `poqm/*/kcoreaddons5_qt.po` blobs from pinned `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15` were read by exact blob identity. The common live catalog shape is 133 msgids / 132 contexts / 6 plural entries, but English is intentionally a six-entry plural shim and the four Serbian variants are sparse 117-msgid catalogs. Canonical localization therefore must preserve locale/context/plural/UTF-8/placeholder semantics while supporting missing-category and missing-entry fallback; fuzzy/obsolete translation history is evidence only and must not become shipping copy.

Existing-owner audit finds the shipping owner in `frontend/src/recovered/features/settings/overlay/localization.ts`, consumed by the real Settings panels. This descendant keeps that single owner and strengthens the live accessibility-count projection with locale-safe `Intl.PluralRules` category selection, category-to-`other` fallback and invalid-locale/non-finite-count fail-safe behavior. No KDE/gettext runtime, second localization store or KDE product wording is introduced. The broader rows remain `mapped-open` / `unknown_closed=false` pending complete production evidence.

Accounting is **9,308/16,125 read; 6,817 unread; 15,845 unknown; 0 omitted**. Next is 9,309 `Telegram/ThirdParty/kcoreaddons::src/CMakeLists.txt@1c15425347a9ffc4e12bbd63797616337c78fe33`. Fresh descendant exact-head GitHub Actions are mandatory.


Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@65e23ba7137ea4129b6bc1b2616104a1f59495ef (root tree 6b616494f3465324e749a04dcd1c9d508657a998). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,836; first unread=5,837 Telegram/SourceFiles/test/test_wallet_intercept.cpp@7a0885063d4f393de43cd05f368e6671dfd40b7d; unread=10,289; unknown=15,846; omitted=0. The b0d1fe5e→65e23ba7 delta is ten commits / 25 modified paths; all 18 changed blobs inside the prior prefix were explicitly re-read and reconciled before retaining prefix credit. Orders 5,831-5,836 were then read/decomposed on the same live authority. Fresh descendant exact-head GitHub Actions evidence is required; b0400d+b0d1fe5 runs are predecessor-only.

# P0 — 全量源文件/模块与现有 Fabushi 架构对照

Status: active / not accepted  
Project: FBCP-001 Revision 7 / TDRP-001 Revision 9  
Updated: 2026-10-11  
Execution: all executable verification only GitHub Actions

## Goal

为全部 tdesktop 非 UI 代码职责的等价重写建立精确、无遗漏的输入和实施合同，而不是只做 Telegram 功能研究。最终完整实现原 Bot + 全部源范围通信能力，统一 Fabushi 品牌与微信式左竖栏布局。

## A. Re-read canonical architecture and source

重新读取 main、PR #20 状态和 current canonical exact HEAD。PR #20 在本次规范读取时已合并；不能继续从历史 open/draft 认知开始，也不能回退到旧实现分支。既有 Bot 架构/验收硬门继续有效。

Historical predecessor authority: 重新读取 tdesktop discovery HEAD。当前 accepted discovery HEAD 为 `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`（root tree `5030985204963cbbd362ced7412d231b04ebd0cc`）；`42f8a36d43b8c805bc821905bea4cfeb3af1d41d` 与更早 authority 仅作历史证据。当前 recursive denominator 16,123；read-through 5,773；unread 10,350；unknown 15,844；source closure 仍 open，predecessor Actions 不得证明新 baseline。

## B. Inventory all files and understand all modules

每个源文件/依赖条目有 exact identity/hash、类型和责任，不能只统计目录。按 TDRP-SRC-02/SRC-03 全量覆盖，特别拆出 mixed UI 文件里的非 UI 逻辑。解析未知或 API 截断必须补读，无法访问保留 blocker。

逐模块 dossier 记录完整源文件/symbol、状态机、所有权、调用图、行为 oracle、错误/安全/生命周期/恢复/性能/平台与许可证。现有 capability graph 是旧基线索引，先审计差异，不是最终范围上限。

## C. Exact-head existing-owner inventory

读取真实 shipping entrypoints，而非根据旧聊天推测：

- product shell/左侧导航、conversation/list/workspace、transcript/cards、composer/drafts；
- account/identity、Shared Room/member、permissions、contacts、search；
- attachments/artifacts/resources、storage/recovery、notifications/settings/platform；
- Coordinator/Host/Runner、Agent、Computer、Plugins/MCP/marketplace、Automations/Task；
- 当前 calls/media/network/service owner（若存在）。

每个 source responsibility 填写 owner 候选、选择原因、target paths/symbols、语言、需修改状态/事件/接口、服务依赖、数据迁移、tests/production evidence。目标 owner 尚未合适时做 rejected-owner analysis 与最小 new-owner ADR，禁止整块另建 CommunicationCore/Telegram subsystem。

同时建立 canonical composition inventory：ProductShell、ConversationWorkspace、ConversationList、Transcript、Composer、Participant/Profile、Resource viewer/editor、Search、Settings、Marketplace、Task/Automation 等每类概念只能有一个 root。每个责任写 `composition_root + composition_slot + variation_axes`；禁止 Bot/Human/Group/Channel/来源型完整 UI/状态栈分叉。

如果某项 Telegram 能力当前完全没有 owner，不能标 N/A：记录 `novel_capability`、所有 rejected owners、owner-absence evidence、最小 state/lifecycle responsibility 和 ADR。批准后只能新增 source-neutral capability owner，并通过 typed contracts 接入统一 shell、identity/resource/permissions/navigation。

## D. Language / production contract

C++ 非 UI 产品逻辑默认 Rust；前端表现与交互投影使用现有 React/TS；系统特有职责采用最薄平台适配。所有非默认语言/通用依赖选择写明理由与安全/许可证边界。

不得以 TDLib、原 C++ core、FFI wrapper、sidecar、外置 Telegram Web 或“暂时 Provider”替代源责任重写。已有实现如违反目标，记录 cutover、数据兼容与移除路径，不能因已存在就自动接受，也不在本次文档任务中盲删生产代码。

## E. Native service completeness

对每项客户端远端依赖列 client/server contract、Fabushi owner、认证/权限/持久化、部署与真实 E2E。自有 messaging/sync/presence/media/call/push 和必要商业能力支持现有模型，不形成第二产品。

缺服务、账号、密钥/签名或产品配置时标 blocked；不得以 mock、仅 UI 或“不用 Telegram 网络”当成功能完成/排除。

## F. UI / branding plan

明确同一 shell 内：左侧窄竖栏 -> 当前入口列表/搜索 -> 主工作区 -> 可选详情。消息、联系人、插件市场必须一级可达；插件市场迁移复用原安装/权限/Plugins/MCP owner，旧路由兼容不复制状态。

Agent、任务、Computer、Automations、账号设置等现有主能力仍可达。消息列表/正文、草稿、未读、运行中任务和 Agent 输出维持同一状态。品牌清单覆盖窗口、头像、Logo、导航、默认文案、通知、托盘、安装和更新；统一 Fabushi，法律声明/provenance 例外精确记录。不得把用户原截图或其私人数据提交公开仓库。

UI 计划必须给出 single-composition matrix：Conversation 只允许一个 workspace、Profile 只允许一个 framework、Resource/Search/Settings/Marketplace 等同理。差异只作为 typed section/entry/action/panel/overlay。Call/Story/media editor/payment 等若没有现成 surface，可设计最小 capability surface，但仍由统一 shell/router 管理并复用 canonical truth。

同时必须交付 **UI capability-placement matrix**。每个用户可见 capability 记录：`ui_entry_class`、primary nav owner、collection/filter、creation flow、object action、detail section、capability surface、route、keyboard/focus/accessibility、responsive behavior、state continuity。没有这些字段的 UI responsibility 不能进入 mapped/implemented。

消息领域必须明确一个 `ConversationCreationSurface`：Human private / Group / Channel / Agent / Hybrid / future Conversation kind 以 typed variant 实现；Group/Channel 不新增独立一级 App。Contacts 的 add/invite 与 Messages 的 conversation creation 分开，联系人发消息只 find-or-create canonical Direct Conversation。

搜索必须明确一个 Search owner 和三层 UX：入口内搜索、当前对象搜索、Universal Search（Cmd/Ctrl+K）。Participant/contact/member/admin picker 复用同一 Participant Search + eligibility policy。P0 必须列 local/remote data source、typed result/provider、权限/隐私、ranking/dedupe/cursor/cancel/debounce/stale-result fencing 与 result navigation。

P0 还必须完成 exact-head UI foundation audit：当前 runtime theme tokens、Sand/shared primitives、overlay/focus behavior、command palette、conversation list/workspace、composer、menus/dialogs、settings/plugins surfaces。基于实际 owner 建立 Fabushi semantic token/component adapter 计划，禁止凭空重做设计系统。

必须输出 design-system migration matrix：现有 token/primitive -> Fabushi semantic token/component；哪些 legacy `sand-*`/`cursor-*` 留在 compatibility/evidence adapter；哪些 feature-local magic values 必须收口；每个核心 screen 对应的 canonical pattern 与 protected slots。

必须建立 canonical screen/state inventory：Messages、Contacts、Conversation(Human/Agent/Group/Channel/Hybrid)、Profile、Creation、Search、Marketplace、Settings、Tasks/Automations、Computer、Call、Story/Media、Mini App，逐一列 loading/empty/error/offline/permission/data-heavy/light/dark/locale/responsive/a11y/motion states。

## G. First shipping vertical slice

existing Fabushi shell + 左竖栏 -> Human identity/contact -> 同一 conversation/composer -> Fabushi native durable send/receive -> canonical transcript -> 显式 Agent action/permission -> Coordinator/Host/Runner -> 同一 transcript/artifact -> restart/reconnect。

同时明确消息/联系人导航与插件市场迁移的可执行验收。第一 slice 通过不等于全部模块完成，后续必须继续全部长尾功能。

## H. Ledger and validator update

按 TDRP-001 §2/§8 扩展现有 schema/validator，建立 file/responsibility/module/owner/target/evidence 双向 gate，不能新造平行账本或批量标 verified。schema/代码/测试修改是下一实现任务，须在 GitHub Actions 执行；本文不声称 gate 已实现。

validator 还必须建立 composition graph gate：检查 canonical roots、routes、state owners、capability registrations 与 production entrypoints，发现按 participant/conversation/source 类型复制完整 workspace/list/profile/composer/resource/settings root 时 fail；对 novel capability 强制 owner-absence evidence + ADR + typed integration。

另建立 UI IA/Search gate：所有可见 capability 必须有合法 entry class/canonical route；Conversation creation 不得分叉为 Group/Channel/Bot 独立 root；Search consumer 必须连接 canonical Search owner/provider registry，禁止页面私建第二索引/权限真相。对应可执行检查与 UI acceptance 仅在 GitHub Actions。

另建立 Design System / Visual gate：检查新迁移 UI 是否只消费 Fabushi semantic tokens/canonical components，是否匹配 canonical screen pattern；固定 viewport、light/dark、zh-CN/en、长文本/RTL、keyboard/focus、reduced-motion、large-data states 生成 visual artifacts 并比对 approved baselines。新增 token/primitive/pattern 必须有 design exception review。

另建立 Quality/Test Governance gate：扩展 ledger/schema 记录 quality_risk、requirement/oracle/invariant/test IDs、negative/property/temporal/fault/UI/visual/a11y/performance/security/regression/exploratory evidence。P0 必须生成 RTM，并确保每个 planned capability 在实现前已有 test basis/oracle。

优先建立 Conversation Turn / Transcript Ordering / Reconciliation 三个 oracle，因为它们覆盖此前最容易被 eventual-state E2E 漏掉的 final disappearing、intermediate replacement、message reorder、late baseline 与 optimistic/authoritative merge 类问题。

packaged acceptance 必须扩展为时间序列：不能只等待 marker 出现；需要记录整个 logical turn 的所有 assistant/tool entries，在 terminal 后经过 quiet window、switch-away/back、reconnect/reload/restart，再比较 canonical semantic result。完整 session video 必须由独立验收者实际审阅。

## Exit criteria

P0 只在以下均完成时通过：

- upstream/target exact baseline、一致 lock/inventory 与递归/外部闭合可核验；
- 全部文件和 non-UI responsibilities 有明确对照，无未知范围；
- 全模块行为合同、current owner resolution 和必要最小 ADR 完整；
- 语言、服务端依赖、接口/数据迁移、UI/品牌与发布风险明晰；
- first slice 和后续全部模块均有具体实现/测试路径；
- 新 ledger/validator gate 取得 exact-head GitHub Actions 证据。

P0 通过只是实施输入闭合，不是产品已完成。某项 blocked 时继续无依赖阻塞的研究、合同、已批准实现或证据收集；不能伪造 P0/既有 Bot gate 已通过来启动依赖它的工作。需要用户支持先检查相同通知及回复，按指定渠道通知一次，再推进其他内容。

Historical predecessor authority: Current live authority (2026-10-09): `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (root tree `5030985204963cbbd362ced7412d231b04ebd0cc`), three commits ahead of historical `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`; 15 root paths changed (12 modified, 3 added), recursive denominator is 16,123, read-through is 5,773, unread is 10,350, unknown is 15,844, omitted is 0, and source closure remains open.


## Live deterministic source batch 5600-5604

Local Storage and Settings Main are exact-source read-complete and mapped-open. Required implementation owners are the existing canonical Settings/Storage/Cache, Identity/Account, Privacy/Security, Wallet/Payments, Business and platform adapters. The next deterministic source is 5605 `settings_notifications.cpp`. Reading these five files does not close unknown; production, service/platform, UI/UX and exact-head evidence remain required.


## Live deterministic source batch 5605-5613

Fail-closed Revision 9 validation rejected the transient incorrect 5608 path and forced exact-order repair. Corrected source responsibilities are Notifications 5605–5607, Reactions 5608–5609, per-type notification policy 5610–5611 and Account Passkeys 5612–5613. Priority production closure remains native notification reply/mark-read/open exact-scope authorization/cleanup plus canonical Account Passkeys list/create/delete and signed packaged WebAuthn acceptance. Existing remote WebAuthn proxy is infrastructure, not Account Passkeys product completion. Next source: 5614 `settings_premium.cpp`.

## Live deterministic source batch 5614–5616

Premium entitlement/subscription/commerce is exact-source read-complete and mapped-open. Existing canonical Entitlement/Subscription, Wallet/Payments/Commerce, Settings/Identity and design-system owners must absorb it. Purchase/ref routing requires invalid-route fail-closed and idempotent cancel/failure/settlement/account-switch/reload/restart reconciliation. Accounting: 5,616/16,123 read, 10,507 unread, 15,844 unknown, 0 omitted. Next: 5617 `settings_privacy_security.cpp`.

### Read-through 5617–5622

Privacy/Security composition, editable keyboard shortcuts, and server-backed Website/Bot authorization sessions are exact-blob read-complete and mapped-open to existing canonical Account Security/Privacy, Authorization, Command Registry, Bot/Blocked-Peer, Payments/Retention and Settings owners. Exact authorization hash, account/session fencing, destructive confirmation/retry/idempotency and keyboard/a11y behavior remain production gates. Accounting: read-through 5,622; unread 10,501; unknown 15,844; omitted 0. First unread: 5623 `settings/settings_builder.cpp`.

### Read-through 5623–5632

Settings Builder/Common/Search/section-factory infrastructure, hidden diagnostic codes, and Credits/Gift commerce graphics are exact-blob read-complete. UI infrastructure maps to canonical Settings/UniversalSearch/design-system owners; diagnostic codes require controlled development/support policy rather than product copying; Credits/Gift actions remain mapped-open under canonical Wallet/Payments/Credits + Gift/Commerce with exact-account/id, cancel/retry/idempotency/rollback/settlement/restart gates. Accounting: read-through 5,632; unread 10,491; unknown 15,844; omitted 0. First unread 5633 `settings_experimental.cpp`.

### Read-through 5633–5642

Experimental flags/support, FAQ suggestion loading, Settings layer/focus/keyboard navigation, notification-common presentation and Power Saving are exact-blob read-complete. They map to canonical feature-flag policy, Help/Support, Settings/design-system navigation, Notification Settings and Performance/Power owners. No source-named product UI or parallel state owner is introduced. Accounting: read-through 5,642; unread 10,481; unknown 15,844; omitted 0. First unread 5643 `settings_power_saving.h`.

## Deterministic source-order correction at 5,623 and live read-through 5,653

Accepted recursive tree `5030985204963cbbd362ced7412d231b04ebd0cc` proves `Telegram/SourceFiles/settings/settings.style@8b9e56dd…` is blob order **5,623**, between `sections/settings_websites.h` (5,622) and `settings_builder.cpp` (5,624). The earlier 5,623–5,642 labels were therefore off by one and are superseded. Their shard/attestation artifacts are replaced rather than treated as parallel authority.

The corrected contiguous batches are 5,623–5,633 (settings.style through Credits/Gift graphics), 5,634–5,643 (Experimental/FAQ/Settings shell/navigation/Power Saving implementation), and 5,644–5,653 (Power Saving interface, privacy controllers, recent Settings search, scale preview, Settings search/type). Current accounting is **5,653 / 16,123 read**, **10,470 unread**, **15,844 unknown**, **0 omitted**. Reading alone closes no unknown. First unread is 5,654 `Telegram/SourceFiles/statistics/chart_lines_filter_controller.cpp@40c3b612…`.

Production closure remains mapped-open; no responsibility receives verified status merely because its source blob was read.

## Deterministic statistics batch 5,654–5,665

Exact blobs for line-filter animation, ruler generation, interactive chart widget, range min/max segment tree, statistics style/common types and chart JSON deserialization are read/decomposed. They map to canonical Analytics/Insights data projection + reusable Chart/design-system owners; business/account truth must not live in the chart. JSON parse errors, empty columns and column-length mismatches fail closed. Hover/zoom/filter/footer state is derived and lifetime-bounded. Accounting: **5,665/16,123 read; 10,458 unread; 15,844 unknown; 0 omitted**. First unread: 5,666 `statistics_format_values.cpp`.

Production closure remains mapped-open; chart rendering alone does not prove analytics data/service parity.


## Deterministic statistics formatting/export batch 5,666–5,674

Exact blobs for locale/timezone analytics labels, TON/Credits chart graphics, analytics-to-sheet projection, 64-bit chart values, and XLSX/OpenXML serialization are read/decomposed. They map to canonical Analytics/localization, Wallet/Credits visual projection and Export owners. Export must preserve inline-string formula-injection safety, XML/control escaping, legal unique sheet names, empty/short-series behavior, numeric/date boundaries, ZIP failure, cancellation/account fencing and cross-platform save behavior. No Telegram-derived statistics/export runtime or UI is introduced. Accounting: **5,674/16,123 read; 10,449 unread; 15,844 unknown; 0 omitted**. First unread: 5,675 `statistics/view/abstract_chart_view.cpp@4b5f2929…`. Reading alone closes no unknown.

Production closure remains mapped-open; export serialization evidence does not prove analytics acquisition/service parity.


## Deterministic statistics chart-view batch 5,675–5,684

Exact blobs for chart-view abstraction, double-line ratios, bar/stack bars, animated dual-axis/currency rulers, closed chart-type dispatch and linear/DPR cache + pointer hit-testing are read/decomposed. They remain mapped-open to one canonical Analytics/Chart design-system owner. The Fabushi port must strengthen empty/one-point, zero-range/max, all-disabled, mismatched-series, invalid-type and pointer-endpoint cases fail closed; rendering state owns no analytics truth. Accounting: **5,684/16,123 read; 10,439 unread; 15,844 unknown; 0 omitted**. First unread: 5,685 `statistics/view/stack_chart_common.cpp@82a1143f…`.

Production closure remains mapped-open; painter parity alone is insufficient without data/service and interaction evidence.


## Deterministic statistics stack/pie batch 5,685–5,690

The remaining statistics/view blobs are read/decomposed: stack geometry/index mapping, filtered pie-percentage rounding, and stack-linear local zoom/pie transition/selection. They remain mapped-open to the canonical Analytics/Chart owner. Canonical code must fail closed for <2 points, zero-width ranges/denominators, invalid indices/line lengths, zero totals and stale/racing zoom/filter/hover state. Accounting: **5,690/16,123 read; 10,433 unread; 15,844 unknown; 0 omitted**. First unread: 5,691 `statistics/widgets/chart_header_widget.cpp@b8754ceb…`.


## Deterministic statistics widgets batch 5,691–5,696

Statistics widgets are exact-blob read/decomposed: responsive chart header, line-filter controls with last-visible-line guard, and localized/currency point details with zoom/cache/ripple state. They map only to canonical source-neutral Analytics/Chart + existing design-system controls. Empty/mismatched data, invalid indices, RTL/long text, currency precision, a11y/focus/theme/DPR and teardown remain production gates. Accounting: **5,696/16,123 read; 10,427 unread; 15,844 unknown; 0 omitted**. Next order 5,697 must be rebound from the accepted recursive tree before recording its path.


## Deterministic PCH + storage security/migration batch 5,697–5,701

Order 5,697 maps C++ PCH/platform composition to canonical cross-platform build evidence. Orders 5,698–5,701 map critical durable local-file protocol, Argon2id/scrypt/legacy passcode KDFs, encryption/signature/version checks and versioned settings/session migrations to canonical Persistence + Local Security/Key Protection + Settings Migration owners. Invalid/over-cost KDFs, corrupt/truncated/unknown blocks, interrupted writes, wrong passcodes and legacy/account/restart recovery must fail closed. This does not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. Accounting: **5,701/16,123 read; 10,422 unread; 15,844 unknown; 0 omitted**. First unread: 5,702 `storage/download_manager_mtproto.cpp@d0007eb1…`.


### TDRP Revision 9 source read 5702-5711 — storage transfer lifecycle

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,702-5,711 cover adaptive remote download scheduling, generic cache/file loader lifecycle, part-based download/resume, HTTP(S)-only web transfer and multipart media upload. Canonical mapping is the existing source-neutral messaging attachment/file-transfer owner plus platform download/save adapters; MTProto/DC/CDN details are protocol-specific and must not create a Telegram runtime. Required parity includes cancel/teardown/account-switch stale-result fencing, partial resume, integrity/reference refresh, redirect downgrade/scheme protection, TLS/auth/disk failures, adaptive concurrency, removed-session resend, transcode/archive cancellation, upload progress/finalization and restart/recovery. Accounting: 5,711/16,123 read, 10,412 unread, 15,844 unknown, 0 omitted; reading alone closes no unknown. First unread: 5,712 `Telegram/SourceFiles/storage/localimageloader.cpp`.

### TDRP Revision 9 source read 5712-5721 — media preparation and durable serialization

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,712–5,713 cover source-neutral media-send preparation and worker/result lifecycle; 5,714–5,717 cover local settings/theme/language/update persistence, legacy encrypted migration and bounded serialization primitives; 5,718–5,721 cover versioned document/media and user/chat/channel projection persistence with legacy location migration and fail-closed corruption handling. Canonical mapping stays with existing messaging Attachment/media-preparation, Persistence/Settings/Local Security/Updater/Theme/Localization, messaging data persistence, and Identity/Profile/Conversation owners. Telegram/MTP file formats and flags are migration inputs only and must not become a parallel runtime or product UI. Required closure includes cancel/account-switch/teardown stale-result fencing, malformed media/encode/archive failures, wrong-key/tamper/truncation/future-version handling, interrupted atomic writes, legacy migration, cache-vs-authority reconciliation, reload/restart and canonical UI/a11y evidence. Accounting: **5,721/16,123 read, 10,402 unread, 15,844 unknown, 0 omitted**; reading alone closes no unknown. First unread: **5,722 `Telegram/SourceFiles/storage/storage_account.cpp@5888ffa1164327b7170a2a3d8df7793a3fa467da`**.

### TDRP Revision 9 source read 5722-5729 — account/domain storage and local security

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,722–5,723 cover encrypted account-scoped drafts/cache/settings/trust/payment/webview/bot/wallet persistence; 5,724–5,725 cover bounded cloud-blob download/extraction; 5,726–5,727 define passcode wraps, bounded memory-hard derivation, secret cleansing, fresh verification tokens, App Lock mutation, crash-safe committed generations, legacy migration and wallet keyring protection; 5,728–5,729 route shared-media/profile-photo cache queries and invalidation. The local-security rows strengthen but do not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. All remain mapped-open to canonical source-neutral owners; no Telegram storage/security runtime or UI family is permitted. Production evidence must cover wrong/unsupported KDF, stale verification, passcode/App Lock mutation, crash/restart migration, wallet corruption/absence, account fencing, delayed-write teardown, cloud/archive failure, cache invalidation and canonical UI/a11y. Accounting: **5,729/16,123 read, 10,394 unread, 15,844 unknown, 0 omitted**. First unread: **5,730 `Telegram/SourceFiles/storage/storage_file_lock.h@181b5bc063827458c4e59a0daa167355f835ae0b`**.

### TDRP Revision 9 source read 5730-5740 — file locking, archive/media preparation and sparse indexes

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,730–5,732 define exclusive storage locking on POSIX/Windows, including conflict-process handling; 5,733–5,734 define folder/multi-file ZIP preparation with symlink exclusion, progress/cancel, size/disk failure and stale temporary cleanup; 5,735–5,736 cover Composer/editor media drag/drop, MIME/limit/decode and asynchronous preview preparation; 5,737–5,740 define shared-media categories and sparse message-id range merge/query/count/invalidation. All remain mapped-open to canonical Persistence/platform adapters, Attachment/Media Editor and Shared Media/index owners. Required evidence includes cross-platform lock conflict/retry, archive cancellation/limit/disk failure/cleanup, malformed media and stale async results, and sparse pagination/count consistency. Accounting: **5,740/16,123 read, 10,383 unread, 15,844 unknown, 0 omitted**. First unread: **5,741 `Telegram/SourceFiles/storage/storage_user_photos.cpp@232befde40c5edea34286105374058fcc66ab658`**.

### TDRP Revision 9 source read 5741-5754 — user-photo cache, streamed download and operator support

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,741–5,742 cover profile-photo history cache list/count/paging; 5,743–5,744 cover bounded aligned streaming-part download with duplicate suppression, cancellation and completion; 5,745–5,754 cover operator/support template autocomplete/search, contact confirmation, shortcut navigation, occupation leases, support-info editing, history preload, fast-button preferences and template update/reload. Support-specific code is conservatively mapped-open pending explicit applicability evidence rather than silently omitted. Required evidence includes profile count/paging consistency, part failure/cancel/staleness, support account/history fencing, edit request replacement, occupation expiry/race, preload retry and malformed/update-failed template recovery plus canonical keyboard/focus/a11y. Accounting: **5,754/16,123 read, 10,369 unread, 15,844 unknown, 0 omitted**. First unread: **5,755 `Telegram/SourceFiles/tde2e/tde2e_api.cpp@efe4edc5eef735c954b39d7a927722a7e8f9214c`**.

### TDRP Revision 9 source read 5755-5758 — call end-to-end encryption

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,755–5,758 own call E2EE temporary/private key lifecycle, one-shot ECDH envelopes, encrypt/decrypt callbacks, participant/state blocks, inbound/outbound queues, subchain reconciliation, failure terminality and emoji verification hash, with protocol conversion as an adapter. They remain mapped-open to canonical Calls E2EE/Crypto + Call Session owners; transport TLS is not a substitute. This implementation batch also hardens the canonical Wallet ledger so reuse of an idempotency request id must match operation kind, accounts, normalized currency, amount and reference; conflicting credit/transfer/refund reuse fails closed with `RequestConflict` and unit coverage. Broader wallet rows remain open pending full production/evidence closure. Accounting: **5,758/16,123 read, 10,365 unread, 15,844 unknown, 0 omitted**. First unread: **5,759 `Telegram/SourceFiles/test/README.md@429b4f3d83ec6fa6aa71203624a894d1d7973d35`**.

### TDRP Revision 9 source read 5759-5771 — professional task-test harness

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,759–5,771 define the upstream task-test authority: pure bounded readiness; exact object/action identity; assertions outside waits; diagnostic timeouts; precondition-only N/A; packed-scenario teardown; protected fixture secrets; deterministic activation/focus and animation timing; exact dialog-shell interaction; visual capture of real paint owners with DPR/scale boundaries; and privacy-safe clipboard classification/retry without logging foreign clipboard contents. These map to Fabushi's professional cross-platform GitHub Actions test/evidence harness, not to production Telegram UI. The same descendant repairs current-head authority failures by correcting order 5,739 to accepted blob `cc0e3ed6b2c763326c2c16635a7671a747ef506b` and refreshing the strict Grok semantic-adaptation target for `source/electron-main/attachments/attachments.ts` to production blob `9fb1ed46b2ae623e7fa781d6d3ab47f465f4a332`; no gate is weakened. Orders 5,772-5,773 cover the Windows console-lock professional-test contract: the implementation reads process-session/WTS metadata and classifies locked/unlocked/unknown, while the header fixes the public NotApplicable/Unlocked/Locked/Unknown state model, makes `locked()` true only for Locked, and requires `ConsoleLockGate` to remain empty for Unknown and NotApplicable. Only a positive locked reading gates; documented legacy-Windows reversed flags remain unknown; diagnostics never read or print user/domain/window-station fields; non-Windows is explicit N/A. This maps to the canonical professional cross-platform test/evidence harness and creates no production Telegram UI. Accounting: **5,773/16,123 read, 10,350 unread, 15,844 unknown, 0 omitted**. Reading alone closes no unknown. First unread: **5,774 `Telegram/SourceFiles/test/test_corner_patch.cpp@a34785a050cdeea405c3948345704d2e19395843`**.



### cf478d37 live-authority rebaseline

Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

Direct delta: channel_earn.style replaces static negative placeholder margins with floating placeholderShiftLeft; lib_ui 91ff446 implements horizontal interpolation in InputField and MaskedInputField. Canonical TextField remains the sole product owner. Orders 5,774-5,793 were identity/order reconciled; reading alone closes no unknown.


### Revision 9 live read-through note: 5794-5801

Orders 5,794-5,797 preserve professional evidence-log one-line/completion-forgery integrity plus an independent raw-byte oracle. Orders 5,798-5,799 preserve complete mapped-target/viewport capture readiness. Orders 5,800-5,801 preserve the bounded reversible not-marking-read evidence lever. These are test/evidence responsibilities only, create no second product owner, and close no unknown. The new 811b83a1 readability delta is order 5,951 and remains unread/unknown.


### Revision 9 live read-through note: 5802-5805

Orders 5,802-5,803 close the exact-blob read/decomposition of popup/context-menu professional evidence semantics: same-turn fresh-menu identity, explicit refusal taxonomy, isolated QAction queued-callback delivery, prepared-frame capture and lock/teardown behavior. Orders 5,804-5,805 close the SentMessageWatcher client→server id reconciliation contract with stable-history/candidate fencing and a five-second diagnostic probe throttle. These are test/evidence responsibilities only and close no unknown. First unread is now 5,806 `test_notify_override.cpp`.

### Revision 9 live rebaseline/read-through note: b0d1fe5e / 5166 / 5806-5813

The 811b83a1→b0d1fe5e upstream delta changes only order 5,166 `media_view_overlay_widget.cpp`: the old `forbidsSaving()` control-refresh branch hid controls and could call `DocumentSaveClickHandler::Save(...ToCacheOrFile)` when media was not loaded; b0d1fe5e removes that viewer-driven automatic persistence path. The new blob `e35f67119323850a3cea450548566186c8a2231c` was re-read and the mapped-open MediaViewer lifecycle responsibility now explicitly requires protected/forbids-saving media not to be auto-persisted as a rendering/control-refresh side effect. Path order and total counts are unchanged. Separately, orders 5,806–5,813 are read-complete/responsibility-decomposed professional test harnesses for reversible notify overrides/no-server-write locality, document-open handoff inspection without OS launch, SeparatePanel pointer/liveness evidence, and post-paint temporal sampling. They add no product owner, fake fallback, unknown closure or release credit. First unread is 5,814 `test_probe.cpp`; read-through is 5,813/16,125, unread 10,312, unknown 15,846, omitted 0.

### Recursive component read-through 6,654-6,697 — Microsoft/GSL

Orders **6,654-6,697** exact-read the pinned `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37` component mounted at `Telegram/ThirdParty/GSL`. The 44 entries cover repository/build/CI metadata, license/security/provenance, header-only safety contracts (bounds and extent checks, non-null ownership, checked narrowing, byte/string boundaries, scope cleanup), and focused fail-closed tests. Applicable semantics map to existing Fabushi Build/Quality/Provenance and Rust/TypeScript/platform-boundary owners; no Telegram/GSL-specific runtime is introduced. Reading does **not** reduce `unknown`; production/shipping evidence remains required. Accounting: **6,697/16,125 read; 9,428 unread; 15,845 unknown; 0 omitted**. Next: **6,698 `Telegram/ThirdParty/MicroTeX::.github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764`**.

### Recursive component read-through 6,698-6,714 — MicroTeX entry/build contracts

Orders **6,698-6,714** exact-read the pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f` build/entry contract under `Telegram/ThirdParty/MicroTeX`. The applicable product behavior maps into the existing ConversationWorkspace/Transcript math owner and existing Build/Resource/Release provenance owners. Current production work adds a 32 KiB pre-parse formula bound and bounded 32 MiB LRU markup cache without retaining the C++/Qt/GDI runtime. Reading does **not** reduce `unknown`; current exact-head GitHub Actions evidence is still required. Accounting: **6,714/16,125 read; 9,411 unread; 15,845 unknown; 0 omitted**. Next: **6,715 `Telegram/ThirdParty/MicroTeX::readme/example_bw_false.svg@9e812c823d3cbe06ab43ee77f10b6122bd638dd`**.

### Revision 9 deterministic read-through — TooManyCooks 7,057–7,154

Pinned authority `tzcnt/TooManyCooks@b86af81982860c96295a7e95e4c60cb335615cef` is now exact-read through all **126** non-directory entries. Orders **7,057–7,154** revalidate the prior temporary 7,057–7,064 read and cover all remaining runtime headers, implementations and package inputs.

Existing-owner-first audit found real shipping candidates: `GatewayHostSupervisor`; `HostRunnerComposition`; the frozen Host→Runner bridge; `RoutedProviderCancellation/RoutedProviderTaskRegistry`; `TurnSettlement`; canonical Mahayana messaging; `PressureCpuProfiler`; and existing GitHub Actions Build/Release owners. Applicable/open behavior includes executor/priority restoration, awaitable/fork/join lifetime, UAF-safe settlement, atomic wait/wake and teardown, foreign callback lifetime/cancellation, queue close/reclamation, CPU quota/topology/work-stealing capacity behavior, and sanitizer/fuzz/coverage/build-option consistency. Exact hwloc pinning is not presumed complete without a dedicated shipping worker-pool owner. vcpkg/CMake/version descriptors are build/provenance inputs, not a new product owner.

No TMC/Telegram runtime/provider or duplicate Identity/Conversation/Message/Search/Settings/Marketplace is introduced. Accounting is **7,154/16,125 read; 8,971 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown. Next deterministic entry is **7,155** `Telegram/ThirdParty/cld3::.github/workflows/main.yml@b26ff5ec0300683c819c0c7f17edfc21bfdd0fc3` in `google/cld3`.

### Revision 9 deterministic read-through — CLD3 7,155–7,241

Pinned `google/cld3@b48dc46512566f5a2d41118c8c1116c4f96dc661` is exact-read through all **87** non-directory entries. The component implements local language identification: bounded input selection; UTF-8/interchange repair and script spans; character n-gram + relevant-script features; embedded feed-forward neural inference; `language / probability / is_reliable` results; multilingual, empty, invalid-UTF8 and script tests; generated model parameters; and build/binding/license inputs.

Consumer tracing in accepted Telegram source finds `Platform::Language::Recognize` used by translation offer/skip policy, chat translation tracking, AI compose default source language, URL translation and Linux translation-provider flows. Current Fabushi shipping tree contains no `language/translate/spellcheck/i18n` path. Prior dossiers only define a **Canonical Translation service gateway** as mapped-open. Therefore no existing production owner can honestly be claimed.

Owner-absence decision: reject Search, Settings/Localization and Message as detector owners because they consume/filter/project language state but should not own classification. If implementation proceeds, create only an approved minimal source-neutral `TextLanguageClassifier` contract behind the canonical Translation gateway; it must be local/private by default, typed unknown/unreliable on bad/short/mixed input, bounded for long text, model/version/license pinned, and covered by multilingual + Unicode/property tests. This paragraph is an ADR requirement/proposal, not approval to add a CLD3 runtime/provider.

Accounting: **7,241 read; 8,884 unread; 15,845 unknown; 0 omitted**. Next is **7,242** `Telegram/ThirdParty/cmark-gfm::.editorconfig@12032e647502de7a91f241f75fcd7666be375875`.
