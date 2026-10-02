# Telegram Desktop Source Research and Target Architecture Map

Status: active  
Purpose: research navigation, capability discovery and architecture planning  
Not a file-to-file migration map.

## 1. 使用规则

这个文件不再回答“上游目录要迁移到哪个同名 Rust crate”。

它回答三件事：

1. 哪些上游区域需要一起研究才能理解一个 capability；
2. capability 的关键行为、风险和平台差异在哪里；
3. 目标架构需要哪些边界和 ADR。

同一个 source area 可以属于多个 capability；同一个 capability 可以横跨多个 source area。

完成一个 source area 的阅读，不代表实现完成。

## 2. Capability map

| Capability | 主要研究区域 | 目标责任方向 |
| --- | --- | --- |
| App lifecycle | main, core, window, platform, settings | Rust app supervisor + platform ports |
| Authentication | intro, api, mtproto, data | Rust application/auth + protocol session |
| MTProto | mtproto, api, lib_tl, codegen | Rust protocol/wire/session |
| Update consistency | api, data, history, mtproto | Rust sync actor + domain transaction |
| Accounts | main, data, settings, tray | Rust account/session ownership |
| Dialogs | dialogs, data, history, main | Rust domain + application + Rust UI |
| History/messages | history, data, api, chat_helpers | Rust message domain + send lifecycle + UI |
| Search | dialogs, history, api, data | Rust query/application + index/cache strategy |
| Groups/channels/topics | info, profile, api, data, boxes | Rust domain/application + UI |
| Media transfer | media, storage, api, ffmpeg | Rust transfer/media orchestration |
| Playback/recording | media, platform, ffmpeg | Rust media state + best-fit codec/platform deps |
| Calls | calls, lib_webrtc, tgcalls, platform | Rust call state/signaling/device ownership |
| Stickers/emoji/GIF | chat_helpers, ui, media, data, lib_lottie | Rust product state/UI + best-fit decoding/rendering |
| Stories | data, api, history, ui | Rust capability owner + Rust UI |
| Bots/Mini Apps | inline_bots, iv, ui, api, lib_webview | Rust business/permissions + secure WebView boundary |
| Payments/Stars | payments, api, data, boxes | Rust payment state/reconciliation/UI |
| Passport/WebAuthn | passport, webauthn, platform | Rust security/application + platform authenticator |
| Settings/privacy | settings, data, platform, ui | Rust settings domain + platform adapters |
| Notifications/tray | platform, window, tray, data | Rust notification policy + platform adapter |
| Export | export, storage, data | Rust export pipeline |
| Local storage | storage, lib_storage, data | Rust storage owner + compatibility reader/migrator |
| Language/i18n | lang, countries, lib_translate | best-fit generated resources + Rust runtime |
| UI framework | ui, lib_ui, layout, window, mainwidget | Rust UI architecture, independently designed |
| Accessibility/IME | ui, platform, ThirdParty IME libs | Rust UI semantics + platform integration |
| Update/install | core, platform, build, cmake, tools | Rust updater/release orchestration + platform packaging |
| Build/codegen | CMake, cmake, codegen, tools, tasks | best-fit reproducible build/generation tooling |
| Resources/shaders | Resources, shaders | native resource formats / shader language, not forced Rust |

这张表只是起点。P0 发现新的可达 capability 时必须新增。

## 3. C++ research classification

每个 C++/header/source 区域在 P0 只做以下分类：

- product-logic
- protocol
- state-machine
- ui
- platform-policy
- build-tool
- test-or-reference
- third-party
- unreachable-with-evidence

product-logic/protocol/state-machine/ui/platform-policy/build-tool 若进入最终产品职责，最终 owner 必须为 Rust。

third-party C++ 也不能自动保留。必须评估 Rust 替代、系统 API 或其他 best-fit dependency；没有批准时保持 blocked。

## 4. Target architecture questions

P1 至少形成以下 ADR：

- ADR-CORE-BOUNDARIES
- ADR-CONCURRENCY-AND-ACTORS
- ADR-PROTOCOL-RUNTIME
- ADR-STORAGE
- ADR-UI-BACKEND
- ADR-MEDIA
- ADR-CALLS
- ADR-WEBVIEW
- ADR-PLATFORM
- ADR-BUILD-AND-CODEGEN
- ADR-DEPENDENCY-AND-LANGUAGE-POLICY
- ADR-UPDATE-AND-PACKAGING
- ADR-LICENSE-PROVENANCE

每个 ADR 必须引用相关 research dossier，但不能用“上游就是这么做的”作为唯一理由。

## 5. 首个 vertical slice

认证 → MTProto/session → updates → durable storage → dialogs/history → Rust composer → send → server update → restart recovery。

该 slice 的目标是验证新的 ownership、protocol、storage 和 Rust UI 架构是可行的，而不是先铺满所有目录。

## 6. Research dossier 模板

每个 capability 至少记录：

- Capability ID / name
- 用户场景
- 上游 commit
- 主要 source references
- 官方协议/API references
- 输入/输出
- 状态机
- ordering/idempotency/retry/cancellation
- persistence/restart semantics
- thread/async constraints
- platform variants
- failure/negative cases
- security/privacy
- performance/resource constraints
- observed upstream technical debt
- behavior oracle
- target architecture questions
- license/provenance notes

研究 dossier 完成后，capability 才能进入 ADR 阶段。
