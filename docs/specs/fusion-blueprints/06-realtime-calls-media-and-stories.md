# Fabushi 统一融合蓝图 06：实时音视频通话、富媒体查看器与 Stories 动态
# Volume 06: Realtime Calls, Media, and Stories

> **规范编号**: `FBCP-BLUEPRINT-06` / `UFX-BLUEPRINT-06`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. 统一 1v1 与群组音视频通话、屏幕共享与后台通话横幅（Background Active Call Banner），彻底废除 `TdrpCallAndAdminDrawer.tsx` 中的 `setTimeout(() => setCallState("active"), 1200)` 伪在线假逻辑，切断通话与无关弹窗（Admin/Export）的强耦合。
> 2. 彻底移除 `TdrpStoriesAndStarsBar.tsx` 中的 `INITIAL_STORIES` 伪造假数据与 `starsBalance = 1280` 硬编码，将 Stories 收敛至会话集合入口（折叠条/弹窗），释放会话舞台垂直空间。
> 3. 基于 `Telegram/SourceFiles/media/stories/`、`data/data_stories.cpp` 与 `native/mahayana-messaging/src/story.rs`，实现真实 24 小时过期、隐私可见范围（Everyone/Contacts/CloseFriends/SelectedUsers）、防陈旧推荐围栏（`LookupShownId`）与阅览/反应回执。
> 4. 实现全屏沉浸式富媒体查看器（Immersive Media Viewer）、画中画（PIP）、流式播放、LRU 缓存配额与网络分片缓冲调优（`MediaCache`）。
> 5. 融合 OpenClaw 实时 Talk、TTS 语音合成与实时语音流（OCRM-M24/M25，`TurnContextTracker`、`RealtimeVoiceAudioQueue`、`AudioOutputPort` 320 块/1 MiB 上限与原子关闭围栏）。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | Telegram Desktop 锁定源码锚点 | OpenClaw 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **1v1 音视频通话与信令**<br>(1:1 Voice & Video Call) | `Telegram/SourceFiles/calls/calls_call.cpp:205-510`<br>`calls/calls_instance.cpp:203-474`<br>`history/view/history_view_top_bar_widget.cpp:306` | `src/talk/talk-session-controller.ts:1-222`<br>`src/talk/client-voice-session-lifecycle.ts:1-208` | `frontend/src/production/human-call-media.tsx:1-140`<br>`source/node-agent-coordinator/src/tdrp/calls_group_calls.rs:6-120`<br>`native/mahayana-messaging/src/realtime.rs:1-100`<br>`source/host/src/extensions/session/production.rs` | `CallSessionStore`<br>`native/mahayana-messaging/src/realtime.rs`<br>`HumanCallControls` |
| **群组语音与视频会议**<br>(Group Voice & Video Chats) | `Telegram/SourceFiles/calls/calls_group_panel.cpp`<br>`Telegram/SourceFiles/calls/calls_group_call.cpp` | `src/meeting-bot/`<br>`src/transcripts/` (meeting capture) | `source/node-agent-coordinator/src/tdrp/calls_group_calls.rs:27-44,140-210`<br>`native/mahayana-messaging/src/realtime.rs:34-42` | `GroupCallController`<br>`native/mahayana-messaging/src/realtime.rs`<br>`GroupCallStage` |
| **屏幕共享与设备控制**<br>(Screen Sharing & Device) | `Telegram/SourceFiles/calls/calls_controller_webrtc.cpp`<br>`Telegram/SourceFiles/calls/calls_panel.cpp` | `src/node-host/`<br>`extensions/cua-computer/`<br>`src/talk/audio-output-port.ts` | `frontend/src/production/human-call-media.tsx:91-99`<br>`source/host/src/runner/computer_use.rs`<br>`source/host/src/runner/remote_computer_target.rs` | `DesktopMediaManager`<br>`HumanCallControls` |
| **后台全局通话浮动条**<br>(Active Call Banner) | `Telegram/SourceFiles/calls/calls_top_bar.cpp`<br>`Telegram/SourceFiles/window/window_main_menu.cpp` | `src/talk/turn-context-tracker.ts` | `frontend/src/production/human-call-media.tsx`<br>`frontend/src/production/tdrp/TdrpCallAndAdminDrawer.tsx:27-60` | `GlobalActiveCallBanner`<br>`ProductionRenderer.tsx` 全局层 |
| **Stories 动态发布与隐私**<br>(Story Post & Privacy) | `Telegram/SourceFiles/data/data_stories.cpp:1-450`<br>`Telegram/SourceFiles/media/stories/media_stories_sibling.cpp` | `src/snapshot/`<br>`src/session-cards/` | `source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs:8-30,69-120`<br>`native/mahayana-messaging/src/story.rs:1-80`<br>`native/mahayana-messaging/src/engine_tdrp_media_stories.rs:48-62`<br>`frontend/src/production/tdrp/TdrpStoryPostModal.tsx` | `native/mahayana-messaging/src/story.rs`<br>`TdrpStoryPostModal`<br>`stories_gifts_stars.rs` |
| **Stories 浏览与陈旧围栏**<br>(Story Viewer & Sibling Fencing) | `Telegram/SourceFiles/media/stories/media_stories_sibling.h:20-60`<br>`media/stories/media_stories_view.cpp` | `src/trajectory/`<br>`src/interactive/` | `native/mahayana-messaging/src/story.rs:80-150`<br>`native/mahayana-messaging/src/engine_tdrp_media_stories.rs:80-100`<br>`frontend/src/production/tdrp/TdrpStoriesAndStarsBar.tsx:31-76` | `StoryViewerModal`<br>`native/mahayana-messaging/src/story.rs` |
| **沉浸式媒体查看器**<br>(Immersive Media Viewer) | `Telegram/SourceFiles/media/view/media_view_overlay_widget.cpp`<br>`Telegram/SourceFiles/media/view/media_view_playback_progress.cpp` | `src/media/`<br>`src/canvas/` | `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx`<br>`frontend/src/production/tdrp/TdrpInteractiveMediaCards.tsx` | `ImmersiveMediaViewer`<br>`media-viewer.tsx` |
| **媒体缓存与网络缓冲调优**<br>(Media Cache & Buffer Tuning) | `Telegram/SourceFiles/data/data_streaming.cpp`<br>`Telegram/SourceFiles/storage/file_download.cpp` | `src/media/media-pipeline.ts`<br>`packages/media-core/` | `native/mahayana-messaging/src/media_cache.rs:1-120`<br>`native/mahayana-messaging/src/blob_store.rs`<br>`source/host/src/extensions/attachments/` | `native/mahayana-messaging/src/media_cache.rs`<br>`DesktopBridge:callMedia` |
| **实时 Talk / TTS / Voice**<br>(Voice Mode & TTS Stream) | `Telegram/SourceFiles/media/audio/media_audio.cpp`<br>`calls/calls_box.cpp` | `src/tts/`<br>`src/talk/exact-speech-protocol.ts`<br>`src/realtime-transcription/websocket-session.ts` | `source/host/src/runner/mod.rs` (`TurnContextTracker`, `RealtimeVoiceAudioQueue`)<br>`source/host/src/ports/mod.rs:AudioOutputPort`<br>`source/host/src/media_mime.rs` | `source/host/src/runner/mod.rs`<br>`AudioOutputPort` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **Telegram Desktop 侧**：
  - 音视频通话由 `calls/calls_instance.cpp`（blob `256ac969efa07be95c3248551f79257b9d9744f6`）与 `calls/calls_call.cpp`（blob `a67723f5827e56be3f36033f3ad59bc37d0acf7b`）持有。1v1 通话状态机经历构造、Diffie-Hellman 密钥协商（`refreshDhConfig`）、`MTPphone_RequestCall`、`handleSignalingData`，并在 `confirmAcceptedCall` 校验指纹。群组通话（`calls_group_call.cpp`）维护每个参会者的静音、举手与 SSRC。
  - Stories 由 `data/data_stories.cpp`（blob `7d2042efa947ca0842a173e6a987ef1c0ba3145b`）与 `media/stories/media_stories_sibling.cpp` 持有，具备显式 24 小时生命周期、`LookupShownId` 防陈旧推荐围栏（防止在源切换时错误渲染已删除的 Story），以及严格的受众隐私（Contacts / Close Friends / Selected）。
  - 媒体查看器（`media_view_overlay_widget.cpp`）支持平移缩放、进度拖动、全屏切换与画中画（PIP）；`data_streaming.cpp` 结合分片范围请求与动态套接字缓冲区调优。
- **OpenClaw 侧**：
  - 实时 Talk 与语音流（OCRM-M24/M25）由 `src/talk/talk-session-controller.ts` 与 `src/talk/client-voice-session-lifecycle.ts` 驱动，具备严格的信令单调序列号（`signal_seq`）、VAD 打断机制（Barge-in）、320 块/1 MiB 的音频队列上限（`AudioOutputPort`），以及 30% CJK 语音阈值（`resolve_edge_tts_voice`）。

#### Q2（现有链路定位）
- **通话控制**：
  `frontend/src/production/human-call-media.tsx`（`HumanCallControls`）-> `DesktopBridge.callMedia` -> Coordinator `source/node-agent-coordinator/src/tdrp/calls_group_calls.rs` -> Host `source/host/src/extensions/session/gateway.rs` / `production.rs` -> Native Rust `native/mahayana-messaging/src/realtime.rs`（`CallSession`, `CallSignal`）。
- **Stories 生产链路**：
  `frontend/src/production/tdrp/TdrpStoryPostModal.tsx` -> Coordinator `stories_gifts_stars.rs:post_story` -> Native Rust `native/mahayana-messaging/src/story.rs`（`Story::record_view`, `Story::react`）与 `engine_tdrp_media_stories.rs:decide_publish_story`。
- **媒体缓存与富文本**：
  `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx` -> `native/mahayana-messaging/src/media_cache.rs`（`MediaCache`）。

#### Q3（断点精确诊断）
1. **抽屉虚假定时器断点**（`frontend/src/production/tdrp/TdrpCallAndAdminDrawer.tsx:69-71`）：
   ```tsx
   setTimeout(() => {
     setCallState("active");
   }, 1200);
   ```
   **诊断**：`TdrpCallAndAdminDrawer` 在发起呼叫后，不论后端真实 WebRTC 信令是否接通，硬编码 1200ms 后强制将状态变更为 `"active"`，这是严重的虚假在线违规；且该抽屉将通话控制、禁言管理（Admin Modal）与消息导出（Export Modal）杂糅在一起，破坏了单一职责。
2. **Stories 硬编码假数据断点**（`frontend/src/production/tdrp/TdrpStoriesAndStarsBar.tsx:24-29,37`）：
   ```tsx
   const INITIAL_STORIES: readonly StoryItem[] = [
     { id: "self", authorName: "我的动态", avatarText: "我", content: "分享生活与思考...", timestamp: "刚刚", isSelf: true },
     ...
   ];
   const [starsBalance, setStarsBalance] = useState(1280);
   ```
   **诊断**：写死了假数据 mock 数组和 1280 假积分余额；且作为横幅常驻在聊天舞台第二行，严重侵占聊天屏幕纵向高度。
3. **通话对象范围安全校验缺失修复点**（`projects/telegram-desktop-rust/dossiers/native-call-scope-admission.md:49-53`）：
   `ProductionSessionWorkers::create_call_session` 曾仅验证 A↔B 具有可读性，未严格校准请求参数 `[A,C]` 与数据库中存储的规范对话双方是否精确匹配，存在跨会话越权发起呼叫的风险。

#### Q4（状态机与安全防护）
- **WebRTC 通话 7 态状态转移方程**：
  $$\text{CallState} \in \{\text{Invited}, \text{Ringing}, \text{Negotiating}, \text{Connected}, \text{Reconnecting}, \text{Ended}, \text{Failed}\}$$
  - $\text{Invited} \xrightarrow{\text{remote\_alerting}} \text{Ringing}$
  - $\text{Ringing} \xrightarrow{\text{remote\_accept}} \text{Negotiating}$
  - $\text{Negotiating} \xrightarrow{\text{ice\_connected}} \text{Connected}$
  - $\text{Connected} \xrightarrow{\text{network\_disconnect}} \text{Reconnecting}$
  - $\text{Reconnecting} \xrightarrow{\text{ice\_reconnected}} \text{Connected}$
  - $\text{Reconnecting} \xrightarrow{\text{timeout (30s)}} \text{Failed}$
  - $\text{Ringing / Connected} \xrightarrow{\text{user\_hangup / remote\_reject}} \text{Ended}$
- **通话 Lease 设备独占与防串号**：
  通过 `CallLease { userId, deviceId, role, isOwner, claimAvailable, generation, eventSeq }` 确保同一账号下的通话在同一时刻仅能在一个物理设备上处于 Active 状态；切换账号或注销时，递增 `generation` 并立即断开 WebRTC MediaStream。
- **Stories 隐私过滤与防陈旧推荐围栏**：
  严格执行 `native/mahayana-messaging/src/engine_tdrp_media_stories.rs` 中的 `story_visibility_flags`，仅在满足 `is_contact` 或 `is_close_friend` 时返回有效 Story；通过 `LookupShownId` 拦截已过期（`now > expires_at_ms`）或已被作者删除的 Story ID。

#### Q5（UI/UX 极致融合落点）
- **通话交互落点**：
  - 会话顶栏右上角放置通话按钮（电话/视频图标）。
  - 通话发起后，主舞台顶部出现轻量级通话状态条；若用户在通话过程中切换到其他会话或页面，页面顶部常驻全局绿色通话横幅（`GlobalActiveCallBanner`），显示对方姓名、通话时长、静音控制与一键挂断。
  - 群组通话提供网格视频会议舞台（`GroupCallStage`），支持全屏与浮动画中画（PIP）。
- **Stories 交互落点**：
  - 彻底移出会话内部顶栏；将其收敛至会话列表（`Sidebar`）顶部的联系人动态头像托盘（Stories Tray），或者整合到“联系人”一级导航中；支持横向平滑滚动与未读彩色光环渐变指示。
- **富媒体查看器落点**：
  - 点击消息流中的图片/视频/音频，全屏弹起暗色高透 `ImmersiveMediaViewer`，支持高刷新率手势缩放、键盘上下左右键翻页，以及一键下载/转存。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与根因定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-06-01** | `TdrpCallAndAdminDrawer` 使用 `setTimeout(..., 1200)` 伪造呼叫接通状态 | `frontend/src/production/tdrp/TdrpCallAndAdminDrawer.tsx:69-71` | 界面开发者为了在无信令联调时展示效果，硬编码了 1.2 秒定时器强行置 `setCallState("active")`，完全违背了真实状态单调性铁律。 | **P0 真实性违规** |
| **BUG-06-02** | `TdrpStoriesAndStarsBar` 硬编码假数据列表与假积分余额 | `frontend/src/production/tdrp/TdrpStoriesAndStarsBar.tsx:24-29,37` | 前端组件内置了写死的 `INITIAL_STORIES` 与 `starsBalance = 1280`，且未与 Native Rust `story.rs` 真实数据表闭环。 | **P0 真实性违规** |
| **BUG-06-03** | Stories 托盘常驻在聊天主舞台第二行，严重侵占纵向空间 | `frontend/src/production/tdrp/TdrpStoriesAndStarsBar.tsx:20-30`<br>`ProductionRenderer.tsx` | 信息架构放置错误。Stories 属于跨会话的好友社交动态，强行作为会话顶部的常驻子工具条，导致聊天正文高度被严重压缩。 | **P1 布局缺陷** |
| **BUG-06-04** | 通话控制与群管理禁言、聊天记录导出强行混在一个右侧抽屉 | `frontend/src/production/tdrp/TdrpCallAndAdminDrawer.tsx:37-46` | 将完全不同频次、不同业务域的逻辑（高频的音视频控制与低频的群管理、导出）杂糅在一个组件内，造成代码职责混乱和状态竞争。 | **P1 职责混乱** |
| **BUG-06-05** | 切换到其他会话后，通话界面完全消失或被阻断，缺少全局通话常驻横幅 | `frontend/src/production/human-call-media.tsx:131-150` | `HumanCallControls` 仅挂载在单会话视图内，未将活动中的通话会话提升至应用壳（Production Shell）全局层面。 | **P2 体验断层** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 架构设计与单一权威归属

为消除通话、媒体与 Stories 的割裂，确立以下架构拓扑：

```
                               ┌──────────────────────────────────────────────┐
                               │     Production Shell (全局壳与顶层容器)      │
                               └──────────────────────┬───────────────────────┘
                                                      │ (活动通话全局感知)
                                                      ▼
                      ┌────────────────────────────────────────────────────────┐
                      │    GlobalActiveCallBanner (全局浮动活动通话横幅)       │
                      │    - 通话对象、耗时计时器、静音/摄像头开关、一键挂断   │
                      └────────────────────────────────────────────────────────┘
                                    │                                  │
                  (处于通话所属会话时)                               (处于其他会话或全屏时)
                                    ▼                                  ▼
      ┌─────────────────────────────────────────┐  ┌─────────────────────────────────────────┐
      │  ConversationHeader 通话动作区          │  │  GroupCallStage / VideoFloatingPIP      │
      │  - 会话内内嵌通话控制与屏幕共享触发    │  │  - 独立全屏网格会议 / 画中画小窗        │
      └─────────────────────────────────────────┘  └─────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                            Stories 与媒体查看器统一落点                                     │
│  1. StoriesAvatarTray: 位于会话列表顶部 (Sidebar Header 下方，可按需折叠)                   │
│  2. StoryViewerModal: 点击头像触发独立沉浸式 Stories 模态流                                 │
│  3. ImmersiveMediaViewer: 点击消息内媒体触发全屏暗色预览 (Zoom/Pan/Download/PIP)             │
└─────────────────────────────────────────────┬───────────────────────────────────────────────┘
                                              │ (底层状态机与原生驱动)
                      ┌───────────────────────┴───────────────────────┐
                      ▼                                               ▼
┌───────────────────────────────────────────┐   ┌───────────────────────────────────────────┐
│     CallSession & Signaling Service       │   │     Story & MediaCache Engine             │
│ - native/mahayana-messaging/src/realtime  │   │ - native/mahayana-messaging/src/story.rs  │
│ - source/host/src/extensions/session/     │   │ - native/mahayana-messaging/src/media_cache│
│ - source/node-agent-coordinator/src/tdrp/ │   │ - Coordinator stories_gifts_stars.rs      │
└───────────────────────────────────────────┘   └───────────────────────────────────────────┘
```

### 3.2 模块拆解与 200–600 行内聚预算

1. **`frontend/src/production/calls/GlobalActiveCallBanner.tsx`** (~260 行)
   - **职责**: 跨路由/跨会话常驻的全局活动通话条。展示通话对方信息、状态（通话中/重连中）、动态计时器及快速控制按钮。
   - **预算**: 220–300 行。
2. **`frontend/src/production/calls/GroupCallStageModal.tsx`** (~380 行)
   - **职责**: 群组音视频多方会议网格舞台。管理参会者视频流网格、发言者高亮、举手队列、屏幕共享与画中画切换。
   - **预算**: 340–420 行。
3. **`frontend/src/production/stories/StoriesAvatarTray.tsx`** (~240 行)
   - **职责**: 会话列表顶部的联系人 Stories 动态头像托盘。支持未读渐变彩环、自发动态入口与横向平滑滚动。
   - **预算**: 200–280 行。
4. **`frontend/src/production/stories/StoryViewerModal.tsx`** (~350 行)
   - **职责**: 沉浸式 24 小时动态播放器。支持左右翻页、进度条逐秒推进、防陈旧围栏、消息快捷回复与点赞。
   - **预算**: 300–400 行。
5. **`frontend/src/production/media/ImmersiveMediaViewer.tsx`** (~420 行)
   - **职责**: 高性能富媒体查看器。支持图片平滑缩放、双击还原、视频自定义播放控件、画中画（PIP）与文件元数据展示。
   - **预算**: 380–480 行。


---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 渐进披露与交互层级设计

音视频、富媒体与 Stories 均属于多媒体高感官交互，必须遵循清晰的视觉与控制渐进披露设计：

```
┌────────────────────────────────────────────────────────────────────────────────┐
│ 1. 活跃通话渐进披露 (Call Controls Progressive Disclosure)                     │
│ Level 1: 顶栏轻量指示条 (Top Header Mini Bar: 头像 · 绿点 · 通话中 01:24)       │
│ Level 2: 浮动操作条 (Active Floating Pill: [麦克风] [摄像头] [屏幕共享] [挂断]) │
│ Level 3: 全屏网格会议 / 沉浸视频通话 (Full Screen Video Grid / WebRTC Stage)   │
├────────────────────────────────────────────────────────────────────────────────┤
│ 2. Stories 动态渐进披露 (Stories Progressive Disclosure)                       │
│ Level 1: 会话列表顶部动态头像环 (未读彩色光环渐变 · 头像右下角小徽标)          │
│ Level 2: 悬停/点击微预览卡片 (作者昵称 · 最近一条动态预览 · 发布时间)          │
│ Level 3: 全屏沉浸式 Stories 播放器 (逐秒进度条 · 视频/图片 · 底部快捷点赞/回复)│
├────────────────────────────────────────────────────────────────────────────────┤
│ 3. 沉浸式富媒体查看器 (Media Viewer Progressive Disclosure)                    │
│ Level 1: 消息气泡内自适应缩略图 (保持比例 · 优雅圆角 · 渐进加载骨架屏)          │
│ Level 2: 浮动快捷工具条 (缩放指示 100% · 旋转 · 原始尺寸 · 画中画 PIP · 下载) │
│ Level 3: 暗色遮罩全屏查看 (按 Esc 退出 · 左右方向键切图 · 拖拽平移)            │
└────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 响应式断点与设备适配

1. **≥1200px（宽屏模式）**：
   - 通话支持侧边停靠或全屏网格展开；视频网格支持最多 16 人并排同屏，超过 16 人自动启用“当前发言者高亮大图 + 底部参会者缩略托盘”。
   - 媒体查看器在大屏上限制最大预览尺寸为视口的 90%，背景采用 80% 高斯模糊暗色遮罩。
2. **900px–1199px（中屏平板模式）**：
   - 通话控制收敛为吸顶横幅；群组会议网格采用 2x2 或 3x3 布局。
   - Stories 播放器居中显示，固定比例 9:16（移动端标准）。
3. **<900px（移动/窄屏模式）**：
   - 正在进行的通话全屏沉浸接管，点击最小化后收缩为右下角可拖拽的画中画（PIP）小圆球。
   - 媒体查看器双指捏合缩放（Pinch-to-zoom），上下滑动手势触发关闭返回。

### 4.3 语义设计变量绑定（`--fabushi-*` Design Tokens）

严格绑定统一设计系统的语义颜色，禁止写死十六进制：

| UI 语义元素 | CSS Variable 绑定 | 浅色模式回退值 (Light) | 深色模式回退值 (Dark) | 规范说明 |
|---|---|---|---|---|
| **通话连接中呼吸灯** | `var(--fabushi-call-connecting)` | `#0284c7` | `#38bdf8` | 脉冲呼吸动画 |
| **通话中常驻绿条** | `var(--fabushi-call-active)` | `#16a34a` | `#22c55e` | 全局通话中指示 |
| **通话挂断强警示** | `var(--fabushi-call-danger)` | `#dc2626` | `#ef4444` | 挂断与静音红色 |
| **未读 Story 彩环起始** | `var(--fabushi-story-ring-start)`| `#ec4899` | `#f472b6` | 动态未读粉色光环 |
| **未读 Story 彩环终止** | `var(--fabushi-story-ring-end)`  | `#8b5cf6` | `#a78bfa` | 渐变紫色光环 |
| **媒体查看器背景遮罩** | `var(--fabushi-media-overlay-bg)`| `rgba(0,0,0,0.85)`| `rgba(0,0,0,0.92)`| 暗色高透明度遮罩 |
| **媒体查看器控制按钮** | `var(--fabushi-media-control-bg)`| `rgba(255,255,255,0.2)`| `rgba(255,255,255,0.15)`| 磨砂玻璃质感 |

### 4.4 无障碍（A11y）与快捷键控制

1. **通话交互无障碍**：
   - 呼叫状态变化通过 `aria-live="assertive"` 立即播报：“正在呼叫张三”、“通话已接通”、“对方已挂断”。
   - 静音按钮通过 `aria-pressed="true|false"` 声明当前麦克风状态。
2. **快捷键规范**：
   - `Space`（长按）：临时开麦对讲（Push-to-Talk）。
   - `M`：全局切换麦克风静音/开麦。
   - `V`：全局切换摄像头开/关。
   - `Esc`：关闭媒体查看器 / 退出全屏 / 关闭 Stories。
   - `Left / Right`：Stories 上下条切换 / 媒体查看器翻页。

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 核心 TypeScript 数据契约

```typescript
// 1. 通话会话状态与信令契约
export type CallState =
  | "invited"
  | "ringing"
  | "negotiating"
  | "connected"
  | "reconnecting"
  | "ended"
  | "failed";

export interface ActiveCallSession {
  readonly id: string;
  readonly conversationId: string;
  readonly peerId: string;
  readonly peerName: string;
  readonly isGroup: boolean;
  readonly isVideo: boolean;
  readonly state: CallState;
  readonly generation: number;
  readonly startedAtMs?: number;
  readonly isMuted: boolean;
  readonly isCameraEnabled: boolean;
  readonly isScreenSharing: boolean;
  readonly participants: readonly {
    readonly actorId: string;
    readonly name: string;
    readonly muted: boolean;
    readonly videoEnabled: boolean;
    readonly screenSharing: boolean;
    readonly isSpeaking: boolean;
  }[];
}

// 2. Stories 数据契约
export type StoryPrivacyKind = "everyone" | "contacts" | "closeFriends" | "selected";

export interface StoryItemContract {
  readonly id: string;
  readonly ownerId: string;
  readonly ownerName: string;
  readonly ownerAvatarUrl?: string;
  readonly mediaUrl: string;
  readonly mediaType: "photo" | "video";
  readonly caption?: string;
  readonly privacy: {
    readonly kind: StoryPrivacyKind;
    readonly allowedUserIds?: readonly string[];
  };
  readonly createdAtMs: number;
  readonly expiresAtMs: number;
  readonly hasViewed: boolean;
  readonly viewsCount: number;
  readonly reactions: readonly {
    readonly actorId: string;
    readonly emoji: string;
  }[];
}

// 3. 沉浸式富媒体契约
export interface MediaViewerTarget {
  readonly mediaId: string;
  readonly url: string;
  readonly type: "image" | "video" | "audio" | "document";
  readonly name: string;
  readonly sizeBytes: number;
  readonly width?: number;
  readonly height?: number;
  readonly durationSeconds?: number;
  readonly thumbnailUrl?: string;
}
```

### 5.2 Rust 原生消息信令与缓存契约（Native Messaging Rust API）

```rust
// native/mahayana-messaging/src/realtime.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallSignalPayload {
    pub call_id: String,
    pub generation: u64,
    pub seq: u64,
    pub sender_device_id: String,
    pub signal_type: String, // "offer" | "answer" | "candidate" | "leave" | "hangup"
    pub sdp_or_candidate: serde_json::Value,
}

// native/mahayana-messaging/src/media_cache.rs
impl MediaCache {
    pub fn insert_scoped(
        &mut self,
        media: MediaRef,
        local_path: impl Into<String>,
        size_bytes: u64,
        now_ms: i64,
        pinned: bool,
        conversation_id: Option<String>,
        topic_id: Option<String>,
    ) -> Result<Vec<MediaCacheEntry>, MediaCacheError>;
}
```

### 5.3 隔离围栏（Fencing Rules）与防竞态规则

1. **通话设备独占与迟到信令丢弃（Lease & Signal Seq Fencing）**：
   - 客户端维护 `currentSignalSeq: number` 与 `callLeaseGeneration: number`。
   - 收到对端信令时，若其 `generation !== currentGeneration`，或者其 `seq <= lastHandledSeq`，一律作为过期/重放数据丢弃，防止已被拒绝的旧连接迟到干扰新连接。
2. **Stories 陈旧推荐围栏（`LookupShownId` Stale Suggestion Fencing）**：
   - 在轮播 Stories 时，如果当前展示的 Story ID 已从后端的 `StoriesSource` 列表中删除，或者本地系统时钟判定 `now > expiresAtMs`（24 小时过期），立即回退到该作者当前的有效 Story，绝不向用户展示空白或报错弹窗。
3. **媒体缓存 LRU 磁盘配额围栏（Media Cache Quota Fencing）**：
   - 严格遵循 `MediaCache::insert_scoped` 中的 `max_bytes` 约束（默认桌面端 10GB 配额）。
   - 当新增大文件导致总容量超标时，优先淘汰非置顶（`pinned === false`）且最久未访问（`last_accessed_ms` 最早）的分片数据，保证离线核心数据不丢失，同时防止磁盘写满。
4. **实时语音音频队列与原子关闭围栏（Voice Queue & Atomic Close Fencing）**：
   - `AudioOutputPort` 维持严格的 320 块与 1 MiB 缓冲区上限；使用 `AtomicI32` 实施原子关闭保护，当语音会话打断（VAD Barge-in）发生时，立即置关闭标位并清空后续缓冲分块，防止打断后仍播出发音残留。

---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 任务切片划分

| 切片编号 | 切片目标 | 允许修改文件白名单 | 只读参考文件 | 验收准则 |
|---|---|---|---|---|
| **SLICE-06-A** | 消除呼叫假定时器与抽屉业务解耦 | `frontend/src/production/tdrp/TdrpCallAndAdminDrawer.tsx`<br>`frontend/src/production/ProductionRenderer.tsx` | `frontend/src/production/human-call-media.tsx`<br>`projects/telegram-desktop-rust/dossiers/calls-group-calls-complete-read.md` | 彻底移除 `setTimeout(..., 1200)`；将 Admin Modal 与 Export Modal 拆分为独立动作菜单。 |
| **SLICE-06-B** | 全局活动通话浮动条与多方会议舞台 | `frontend/src/production/calls/GlobalActiveCallBanner.tsx`<br>`frontend/src/production/calls/GroupCallStageModal.tsx` | `native/mahayana-messaging/src/realtime.rs`<br>`source/node-agent-coordinator/src/tdrp/calls_group_calls.rs` | 通话中离开会话，全局顶部常驻浮动条并能一键静音/挂断；群组会议提供视频网格。 |
| **SLICE-06-C** | 消除 Stories 假数据并收敛至会话列表 | `frontend/src/production/stories/StoriesAvatarTray.tsx`<br>`frontend/src/production/stories/StoryViewerModal.tsx`<br>`frontend/src/production/tdrp/TdrpStoriesAndStarsBar.tsx` | `native/mahayana-messaging/src/story.rs`<br>`source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs` | 彻底移除 `INITIAL_STORIES` 与假积分；Stories 头像托盘放置在列表顶部，点击调起全屏查看器。 |
| **SLICE-06-D** | 沉浸式富媒体查看器与缓存联动 | `frontend/src/production/media/ImmersiveMediaViewer.tsx`<br>`frontend/src/recovered/features/conversation/workspace/media-viewer.tsx` | `native/mahayana-messaging/src/media_cache.rs`<br>`Telegram/SourceFiles/media/view/media_view_overlay_widget.cpp` | 支持图片/视频全屏缩放、画中画（PIP）、拖拽与下载，接入底层 `MediaCache`。 |

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 正反例检查表（Anti-Cheating Checklist）

| 检查项 | 正确实现标准（Acceptance Passing） | 严禁作弊/违规行为（Forbidden Cheating） | 审计判定方式 |
|---|---|---|---|
| **呼叫状态真实性** | 状态完全由 WebRTC/Coordinator 信令事件驱动 | 使用 `setTimeout` 延时 1.2 秒自动变成 `active` | 静态检查代码是否存在 `setTimeout` 修改 `callState`；无网络呼叫验证是否停在 Dialing。 |
| **Stories 数据来源** | 列表完全由后端 `stories.list` / Rust `story.rs` 提供 | 本地预制 `INITIAL_STORIES` 假数组并写死积分 `1280` | 检查源码中是否仍存在硬编码故事数据与假用户昵称。 |
| **Stories 视口占用** | 收敛为会话列表顶部的可折叠托盘，不侵占会话高度 | 继续作为固定横幅霸占在每个聊天会话的顶部第二排 | 打开会话，检查聊天顶栏下方是否干净，无第二排固定的 Stories 按钮。 |
| **全屏媒体查看器** | 覆盖全屏暗色遮罩，支持平滑缩放、PIP 与键盘 Esc 关闭 | 简单的 `window.open(url)` 或弹出未定制的原生新窗口 | 点击聊天中的图片与视频，验证是否在统一 UI 壳内弹出高质感查看器。 |
| **后台通话保活** | 处于通话中时切换到其他会话，顶层仍显示全局控制条 | 切换到其他会话时通话静默断开或丢失控制按钮 | 发起通话并切到联系人页面，检查顶部是否出现绿色活动通话条且能控制麦克风。 |
| **作用域范围校验** | 呼叫发起前严格校验请求成员集合与存储的直接对谈者一致 | 仅校验双方具有可读性，未阻止 `[A,C]` 跨越权调用 | 检查 `ProductionSessionWorkers::create_call_session` 是否包含精确双人集合断言。 |
| **代码内聚预算** | 新增文件严格控制在 200–600 行以内，结构清晰 | 产生单个超过 1000 行的巨石文件，或直接全塞入 `ProductionRenderer.tsx` | 统计新增文件行数：`wc -l frontend/src/production/calls/* frontend/src/production/stories/*`。 |
