# Fabushi 统一融合蓝图 11：多模态生成、富媒体编辑器、断点传输与即时预览
# Volume 11: Multimodal Generation, Media Editor, Resumable Transfers, and Instant View

> **规范编号**: `FBCP-BLUEPRINT-11` / `UFX-BLUEPRINT-11`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **责任领域**: `OCRM-M23`（附件与传输）、`OCRM-M24`（多模态生成）、`TCG-N19`（分片传输）、`TCG-N20`（流式缓存）、`TCG-N49`（媒体编辑器）、`TCG-N50`（即时预览 Instant View）  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. 构建全功能非破坏性内置媒体编辑器（`FabushiMediaEditor`），闭环图片/视频裁切、旋转、马赛克、自由画笔涂鸦与文字贴图，补齐 `TCG-N49` 核心短板。
> 2. 闭环多模态生成流水线（`OCRM-M24`，图/视/音/曲），打通生图参数归一化（1K/2K/4K）、参考图条件注入与轮询取消状态机，解除 `BL-CURSOR-BACKEND-IMAGE-RESOLUTION-001` 阻塞。
> 3. 落地 512 KiB 分片并发与断点续传引擎（`TCG-N19` + `OCRM-M23`），支持网络抖动自动退避、SHA-256 块校验与本地磁盘预分配。
> 4. 实现全景媒体多级 LRU 预加载缓存与流式边下边播（`TCG-N20`），并提供零脚本污染的沉浸式即时文章纯净阅读器（`Instant View`，`TCG-N50`）。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | Telegram Desktop 锁定源码锚点 | OpenClaw 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **内置媒体编辑器与涂鸦**<br>(`TCG-N49` / Media Editor) | `Telegram/SourceFiles/editor/editor_scene.cpp:1-420`<br>`Telegram/SourceFiles/boxes/photo_editor_box.cpp:1-380`<br>`ui/image/image.cpp:110-240` | `src/canvas/`<br>`src/interactive/`<br>`packages/media-core/` | `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx:41-94`<br>`frontend/src/recovered/features/conversation/workspace/desktop.ts` | `FabushiMediaEditorModal`<br>`frontend/src/recovered/features/conversation/workspace/media-editor/` |
| **多模态生图/生视/生音**<br>(`OCRM-M24` / Multimodal Gen) | `storage/storage_media_prepare.cpp:120-210`<br>`data/data_photo.cpp:80-190` | `src/image-generation/image-assets.ts:1-275`<br>`src/image-generation/resolution.ts:1-15`<br>`src/media-generation/runtime-shared.ts:1-733`<br>`src/music-generation/provider-assets.ts:1-152` | `source/host/src/runner/tools/sand_generate_image_tool.rs:1-180`<br>`source/host/src/extensions/attachments/media_generation_service.rs`<br>`source/host/src/extensions/attachments/generate_image_service.rs` | `SandGenerateImageService`<br>`MultimodalGenerationCoordinator`<br>`source/host/src/extensions/attachments/` |
| **大文件分片断点续传**<br>(`TCG-N19` / Chunked Transfer) | `Telegram/SourceFiles/storage/file_upload.cpp:1-550`<br>`Telegram/SourceFiles/storage/file_download.cpp:1-620`<br>`storage/storage_media_prepare.cpp:1-310` | `src/media/configured-max-bytes.ts:1-96`<br>`src/media/local-media-access.ts:1-290`<br>`extensions/file-transfer/` | `source/electron-main/attachments/attachment-manager.ts:1-280`<br>`source/host/src/extensions/attachments/attachments_service.rs:1-340`<br>`source/shared/media/attachment-limits.ts:1-110` | `ResumableTransferManager`<br>`source/electron-main/attachments/`<br>`native/mahayana-messaging/src/blob_store.rs` |
| **流式播放与预加载缓存**<br>(`TCG-N20` / Stream & LRU) | `Telegram/SourceFiles/data/data_streaming.cpp:1-450`<br>`Telegram/SourceFiles/media/player/`<br>`media/audio/media_audio.cpp` | `src/media/sniff-mime-from-base64.ts:1-29`<br>`packages/media-core/` | `native/mahayana-messaging/src/media_cache.rs:1-160`<br>`source/electron-main/media/media-protocol.ts:1-190`<br>`frontend/src/recovered/features/conversation/workspace/media-viewer.tsx:1-140` | `MediaCacheService`<br>`native/mahayana-messaging/src/media_cache.rs`<br>`frontend/.../media-viewer.tsx` |
| **即时预览纯净阅读器**<br>(`TCG-N50` / Instant View) | `Telegram/SourceFiles/iv/iv_instance.cpp:1-390`<br>`Telegram/SourceFiles/iv/iv_box.cpp:1-280`<br>`data/data_web_page.cpp:150-320` | `src/link-understanding/`<br>`src/web-fetch/`<br>`packages/markdown-core/` | `source/host/src/runner/tools/sand_web_tools.rs:40-150`<br>`frontend/src/recovered/features/conversation/cards/transcript-card/views/widget.tsx` | `InstantViewOverlay`<br>`frontend/src/recovered/features/conversation/workspace/instant-view/` |
| **附件安全沙箱与嗅探**<br>(`OCRM-M23` / Security Sandbox) | `storage/storage_media_prepare.cpp:210-290`<br>`core/mime_type.cpp` | `src/media/local-media-access.ts:1-290`<br>`src/media-understanding/defaults.ts:1-193` | `source/host/src/attachment_paths.rs:1-160`<br>`source/host/src/media_mime.rs:1-120`<br>`source/shared/media/attachment-open-policy.ts` | `AttachmentSecuritySandbox`<br>`source/host/src/media_mime.rs`<br>`source/electron-main/attachments/` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **Telegram Desktop 侧**：
  - 富媒体编辑由 `editor/editor_scene.cpp` 与 `boxes/photo_editor_box.cpp` 统管，提供多层非破坏性画布（Base Image Layer、Crop/Rotate Transform Matrix、Paint Stroke List、Text Item List），支持 1:1/4:3/16:9/自由裁剪及 90° 步进旋转，并维护完整的 Undo/Redo 历史栈；导出时通过高保真离屏渲染输出压缩文件。
  - 大文件传输由 `storage/file_upload.cpp` 与 `storage/file_download.cpp` 驱动，固定采用 512 KiB 分片（Part Size），基于分片索引序列与滑动窗口并发执行；传输支持按 Part 断点恢复、计算分片 SHA-256 并聚合为整体校验码；取消传输立即释放本地预分配句柄。
  - Instant View 由 `iv/iv_instance.cpp` 持有，上游解析 HTML DOM 后剥离所有 `<script>`/跟踪像素，重构为纯净抽象语法树（IV Block: Title, Author, Cover Media, Paragraph, Blockquote, Collapsible），支持离线缓存与字体/深浅模式动态适配。
- **OpenClaw 侧**：
  - 多模态生成由 `src/image-generation/`、`src/video-generation/` 与 `src/music-generation/`（OCRM-M24）支撑。核心函数 `resolve_closest_aspect_ratio` 与 `resolve_closest_resolution`（1K/2K/4K）依据对数长宽比距离与像素面积双重打分；生成状态流通过轮询长任务 Token 驱动，并在任务取消时向外部 API 下发 Abort 信令。

#### Q2（现有链路定位）
- **富媒体查看与编辑**：
  - 点击消息附件 -> `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx:195` 弹出 Lightbox 画廊，仅支持缩放（`MIN_ZOOM=1` 至 `MAX_ZOOM=5`）与键盘翻页，完全缺失任何裁剪、涂鸦或旋转能力。
- **大文件与附件传输**：
  - 输入台选文件 -> `frontend/src/recovered/features/conversation/workspace/desktop.ts` -> Electron IPC `attachment-manager.ts` -> Host `attachments_service.rs`。目前仅支持整文件单次读写，缺少分片切分与断点持久化记录。
- **多模态 AI 生成**：
  - Composer/Tool 触发 -> `sand_generate_image_tool.rs` 调用 `SandGenerateImageService` -> 尝试派发至后端。因 `BL-CURSOR-BACKEND-IMAGE-RESOLUTION-001` 阻断，多厂商模型能力未能完整透传。

#### Q3（断点精确诊断）
1. **富媒体涂鸦裁剪编辑器完全缺失（`TCG-N49` 孤岛断点）**：
   - `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx` 仅实现了纯查看 Lightbox，既未接入 `boxes/photo_editor_box.cpp` 对应的裁剪工具条，也未实现 Canvas 涂鸦图层；用户在发送截屏或图片前无法进行敏感信息打码（Mosaic）与红框批注。
2. **大文件传输无分片状态机，断网即全量作废（`TCG-N19` 割裂断点）**：
   - `source/electron-main/attachments/attachment-manager.ts` 接收文件时采用全量内存 Buffer 写入，一旦传输中途网络中断或应用关闭，已下载的数百兆数据全部被静默丢弃，缺乏基于 Part Index 的断点续传账本。
3. **生图分辨率与参数被下游 Proto 丢弃（`OCRM-M24` 断链）**：
   - `projects/openclaw-rust/dossiers/media-generation-attachments-scoped.md:54` 明确记录 `BL-CURSOR-BACKEND-IMAGE-RESOLUTION-001`：Cursor protobuf 缺少 resolution 字段，导致 `sand_generate_image_tool.rs` 归一化后的 1K/2K/4K 参数在最终调用前被降级吞没。
4. **外部链接无即时预览纯净排版（`TCG-N50` 缺失）**：
   - 消息流中点击含文章链接的消息卡片，仅能调用系统默认浏览器外跳打开，缺少 Telegram `iv/iv_instance.cpp` 级别的内置无干扰阅读器。

#### Q4（状态机与安全防护）
- **分片传输状态转移方程**：
  $$\text{TransferState} \in \{\text{Idle}, \text{Preflight}, \text{Transferring}(p/N), \text{Paused}, \text{Verifying}, \text{Completed}, \text{Failed}, \text{Cancelled}\}$$
  - $\text{Preflight} \xrightarrow{\text{quota\_ok \& fd\_allocated}} \text{Transferring}(0/N)$
  - $\text{Transferring}(p/N) \xrightarrow{\text{chunk\_ack}(p+1)} \text{Transferring}(p+1/N)$
  - $\text{Transferring} \xrightarrow{\text{net\_drop}} \text{Paused (Retry in ExpBackoff)}$
  - $\text{Transferring}(N/N) \xrightarrow{\text{sha256\_match}} \text{Completed}$
  - $\text{Transferring} \xrightarrow{\text{user\_cancel / account\_switch}} \text{Cancelled (Release Fd)}$
- **安全沙箱与目录穿越防御**：
  - 强制遵循 `source/host/src/attachment_paths.rs` 与 `local-media-access.ts`：所有本地暂存文件必须严格限制在当前沙箱目录内，严禁通过 `../` 逃逸；MIME 嗅探必须结合 Magic Bytes 校验，防御伪造扩展名的可执行文件渗透。
- **多账号跨域隔离**：
  - 切换账号触发 `accountScopeGeneration` 递增，在途的大文件分片上传/下载与多模态生成任务立即触发 Abort 围栏，旧账号的分片临时文件不得被新账号读取。

#### Q5（UI/UX 极致融合落点）
- **媒体编辑器（`FabushiMediaEditorModal`）**：
  - 点击待发送图片右上角“编辑（笔刷/裁剪）”图标唤起全屏模态层；左侧/顶部常驻工具条：自由裁剪、固定比例（1:1、4:3、16:9）、90度顺时针旋转、涂鸦画笔（画笔粗细 3 档、调色盘 8 色）、高斯模糊马赛克、文字气泡；底部包含“撤销（Undo）”、“重做（Redo）”、“取消”与“应用并发送”。
- **大文件断点续传指示器**：
  - 消息气泡与输入台附件托盘中提供真实进度环与速度指示（例：“14.2 MB / 85.0 MB · 2.1 MB/s”）；支持显式暂停与恢复按钮，禁止虚假平滑 CSS 动画。
- **即时预览阅读器（`InstantViewOverlay`）**：
  - 点击消息中的“Instant View / 快速阅读”按钮，在会话主舞台上方平滑滑出全高抽屉/覆盖层，居中展示最大宽度 720px 的出版级纯净排版，支持字号无级缩放与目录锚点跳转。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-11-01** | 富媒体查看器仅支持缩放翻页，完全缺失涂鸦裁剪编辑 | `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx:41-94,195-205` | 历史移植仅吸纳了最简 Lightbox 预览视图，未将 `editor_scene.cpp` 与 `photo_editor_box.cpp` 的离屏 Canvas 状态机接入工程。 | **P0 功能缺失** |
| **BUG-11-02** | 大文件传输无分片续传账本，断网导致下载全量报废 | `source/electron-main/attachments/attachment-manager.ts:50-130`<br>`source/host/src/extensions/attachments/attachments_service.rs:80-160` | IPC 与宿主之间使用单一单次流通信，未设计 512 KiB 分块 Range 协议与本地 SQLite/JSON 分片续传持久化账本。 | **P1 可靠性缺陷** |
| **BUG-11-03** | 多模态生图高级参数（分辨率与几何）在终端调用前被吞没 | `source/host/src/runner/tools/sand_generate_image_tool.rs:48-95`<br>`projects/openclaw-rust/dossiers/media-generation-attachments-scoped.md:54` | 宿主虽然完成了 `normalize_image_generation_resolution`，但底层适配器受限于外部旧 Proto 缺乏 resolution 字段，未建立本地降级分发矩阵。 | **P1 状态断链** |
| **BUG-11-04** | 外部文章无内置即时阅读，强跳外部浏览器打断会话沉浸 | `frontend/src/recovered/features/conversation/cards/transcript-card/views/widget.tsx:120-180`<br>`source/host/src/runner/tools/sand_web_tools.rs:40-75` | 缺乏将 HTML DOM 转换为本地安全纯净 AST 的渲染管道，导致 Telegram 标志性的 Instant View 沦为外部外链打开。 | **P1 体验割裂** |
| **BUG-11-05** | 本地媒体缓存缺乏多级配额保护，易导致磁盘空间耗尽 | `native/mahayana-messaging/src/media_cache.rs:1-120`<br>`source/host/src/media_mime.rs:25` | 缓存仅做简单文件存储，未实现基于最后访问时间（LRU）与最大存储容量（如 2GB/10GB 用户配额）的自动清理剔除策略。 | **P2 资源泄漏** |
| **BUG-11-06** | 换号登录时旧账号附件上传临时文件未被原子清理 | `source/electron-main/attachments/attachments.ts:35-80` | 文件暂存路径以全局临时目录组织，未加入 `accountScopeGeneration` 哈希子目录隔离，存在跨账号读取未完成附件的隐私风险。 | **P1 隔离漏洞** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 架构设计与单一权威归属

为彻底根除上述 6 大割裂缺陷，确立 **多模态与富媒体全生命周期统合架构（Unified Multimodal Media Pipeline）**：

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   ProductionRenderer (Unified Workspace)                               │
├────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 1. 交互与编辑表面 (Interaction Surface)                                                                │
│    - FabushiMediaEditorModal (非破坏性离屏 Canvas: 自由涂鸦、1:1/4:3/16:9 裁剪、马赛克、文字贴图)     │
│    - LightboxMediaViewer (高刷新平移缩放、预加载翻页、画中画 PIP、一键进入编辑)                        │
│    - InstantViewOverlay (安全清洗 DOM AST、排版无干扰纯净文章阅读器)                                   │
├────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 2. 状态机与控制器 (Frontend Store & Controllers)                                                       │
│    - ResumableTransferStore (512 KiB 分片断点跟踪、速度计量、暂停/恢复、SHA-256 完整性)                │
│    - MultimodalGenerationStore (生图/视/音长任务轮询、分辨率归一化、参数历史回填)                      │
│    - InstantViewStore (文章内容缓存、字号/主题排版配置、TOC 目录索引)                                  │
├────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 3. 桌面 IPC 桥接层 (Electron Main Attachments Gateway)                                                 │
│    - ResumableTransferManager (多连接并发 Range 下载、分片文件落盘句柄池、断网自动重试)                │
│    - AttachmentSecuritySandbox (Magic Bytes 嗅探、路径遍历拦截、accountScope 隔离)                     │
├────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 4. 宿主服务与原生存储层 (Host Service & Native Rust Storage)                                          │
│    - SandGenerateImageService & MultimodalDispatch (多模型多模态生成适配器、Fallback 矩阵)            │
│    - MediaCacheEngine (native/mahayana-messaging: 2级 LRU 淘汰、波形预渲染、分片流式代理)              │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 模块拆分与 200–600 行内聚预算清单

严格遵循 `Same Owner != Same File` 原则，所有代码文件控制在 200–600 行以内：

| 模块路径 | 职责与内聚边界 | 预算行数 | Canonical Owner |
|---|---|---|---|
| `frontend/src/recovered/features/conversation/workspace/media-editor/FabushiMediaEditorModal.tsx` | 媒体编辑器主模态层，挂载 Canvas 舞台、工具栏与操作按钮 | 320–480 行 | `MediaEditorUI` |
| `frontend/src/recovered/features/conversation/workspace/media-editor/editor-canvas-engine.ts` | 离屏 Canvas 渲染引擎，负责画笔笔触、马赛克矩阵计算与几何变换矩阵 | 380–520 行 | `EditorCanvasEngine` |
| `frontend/src/recovered/features/conversation/workspace/media-editor/editor-history-store.ts` | 非破坏性编辑图层数据模型、撤销/重做（Undo/Redo）历史栈管理 | 220–320 行 | `EditorHistoryStore` |
| `frontend/src/recovered/features/conversation/workspace/instant-view/InstantViewOverlay.tsx` | 即时预览纯净文章覆盖层，支持字号调整、主题自适应与目录导航 | 300–450 行 | `InstantViewUI` |
| `frontend/src/recovered/features/conversation/workspace/instant-view/instant-view-parser.ts` | HTML DOM 安全清洗器，过滤危险标签，输出纯净 IV AST 节点 | 260–380 行 | `InstantViewParser` |
| `source/electron-main/attachments/resumable-transfer-manager.ts` | 512 KiB 分片并发下载器、断点状态持久化、文件句柄生命周期管理 | 350–500 行 | `ResumableTransferManager` |
| `source/host/src/extensions/attachments/media_generation_service.rs` | 多模态生成调度中心，处理任务创建、模型路由、取消信号分发与轮询 | 360–540 行 | `SandGenerateImageService` |
| `native/mahayana-messaging/src/media_cache.rs` | 本地媒体多级 LRU 缓存引擎，管理磁盘配额剔除与音视频流式 Range 请求 | 280–420 行 | `MediaCacheEngine` |

---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 媒体编辑器交互规范（`FabushiMediaEditorModal`）

媒体编辑器采用 Telegram 桌面端 `boxes/photo_editor_box.cpp` 经典的分层交互范式，兼顾桌面端鼠标精确操作与触控板手势：

1. **渐进披露工具层级（Progressive Disclosure）**：
   - **顶栏（Top Bar）**：居左显示当前图片尺寸与缩放比例（例：`1920 × 1080 · 100%`）；居中显示当前激活模式指示（`裁剪` / `画笔` / `马赛克` / `滤镜` / `文字`）；居右常驻 `撤销（Cmd/Ctrl+Z）`、`重做（Cmd/Ctrl+Shift+Z）` 与 `重置全部修改` 按钮。
   - **主画布舞台（Main Viewport Canvas）**：居中自适应缩放原图，周围保留 24px 安全边距。图片支持两指捏合手势或滚轮平滑缩放（1.0x 至 4.0x），按住空格键（Space）+ 拖拽支持自由平移视口。
   - **底栏控制托盘（Bottom Tool Dock）**：采用悬浮胶囊设计（高度 56px，深色毛玻璃背景 `rgba(18, 24, 38, 0.88)`），由左至右横向排列 5 个核心工具按钮：
     - **裁剪/旋转（Crop & Rotate）**：展开二级比例预设胶囊（`自由`、`1:1 原形/正方形`、`4:3`、`16:9`、`9:16 竖屏动态`）与 90° 顺时针旋转按钮、水平镜像翻转按钮。
     - **自由画笔（Brush）**：展开二级粗细选择（3px 细线、8px 标注、20px 记号笔）与 8 色快捷调色盘（红色 `#f04438`、黄色 `#f79009`、绿色 `#12b76a`、蓝色 `#0ba5ec`、紫色 `#7a5af8`、白色 `#ffffff`、黑色 `#101828`、荧光绿 `#32d583`）。
     - **安全马赛克（Mosaic / Blur Shield）**：提供像素马赛克与高斯模糊 2 种笔刷模式，涂抹区域自动模糊脱敏，用于遮蔽手机号、密码、银行卡号等隐私。
     - **富文本贴字（Text Overlay）**：点击画布任意位置插入可编辑文本框，支持描边字体、背景底色与字体大小调整。
     - **动作收口区**：右侧包含“取消（Escape）”与高亮蓝底“完成并应用（Enter）”。
2. **非破坏性（Non-destructive）撤销栈设计**：
   - 所有的笔触操作、马赛克蒙版与文字均作为矢量图层保存在内存历史栈（`EditorHistoryStore`）中，用户在编辑过程中可任意回退，原始图片像素不发生降质或不可逆覆盖。

### 4.2 Lightbox 画廊手势与双击缩放（`LightboxMediaViewer`）

画廊查看器深度承接 `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx` 的手势物理学：

- **平移缩放物理阻尼**：双击图片在当前光标位置触发 `1.0x <-> 2.5x` 快速切换；在放大状态下，单指拖拽具备粘滞阻尼感知，当平移到图片边缘时触发回弹动效。
- **键盘全功能导航**：
  - `Escape`：优雅退出查看器并恢复触发前焦点元素；
  - `ArrowLeft` / `ArrowRight`：按环形队列切换上一张/下一张媒体，带有微幅水平滑动过渡（180ms ease-out）；
  - `Space`：在查看视频时切换播放/暂停；
  - `E` 键：快捷将当前图片推入 `FabushiMediaEditorModal` 编辑模式；
  - `D` 键：触发当前媒体下载/另存为。

### 4.3 即时预览纯净阅读器（`InstantViewOverlay`）

- **版心与排版规范**：正文区域锁定在最大宽度 `720px` 黄金阅读宽度，行高 `1.65`，段间距 `16px`。首行大图封面支持全宽展开，文中图片支持点击调起全屏查看。
- **排版控制悬浮条（Typography Bar）**：
  - 顶栏右侧提供字体调节菜单：支持 5 档字号缩放（14px, 16px 标准, 18px, 20px, 22px 特大）；
  - 字体族切换：现代无衬线（Inter / System-UI）与经典衬线体（New York / Georgia）；
  - 阅读模式：浅色纸张（`#fdfcf9`）、羊皮纸护眼（`#f5eedc`）、纯黑暗夜（`#0e1117`）。
- **左侧大纲抽屉（Table of Contents）**：宽度 240px，自动提取文档中的 H1/H2/H3 锚点，点击平滑滚动并高亮当前阅读章节。

### 4.4 响应式断点与无障碍（A11y）

| 视口断点 | 媒体编辑器交互布局 | 画廊查看器与视频播放 | 即时预览（Instant View）阅读器 |
|---|---|---|---|
| **宽屏（≥1200px）** | 居中最大 `1080px` 画布，工具托盘悬浮于底部，快捷键全局生效 | 居中全屏 Lightbox，两侧保留 48px 翻页箭头，底部保留缩略图条 | 居中 720px 阅读容器，左侧常驻 240px TOC 目录树 |
| **中屏（900–1199px）** | 居中最大 `840px` 画布，二级选项收纳为浮出气泡面板 | 居中全屏，翻页箭头收缩为悬浮半透明小按钮 | 居中 680px 阅读容器，TOC 目录转为顶栏下拉抽屉 |
| **窄屏（<900px）** | 满屏适配，底栏工具栏支持横向滑动，优先保障绘图面积 | 满屏滑动，支持左右边缘滑动手势切换上一张/下一张 | 全屏覆盖主舞台，顶栏收缩为紧凑标题与关闭叉号 |

- **无障碍（A11y）与 Focus Trap 规范**：
  - 打开编辑器或阅读器时，焦点自动捕获在内部首个可操作元素，禁止 Tab 键逃逸至背景会话；
  - 所有工具图标必须具有显式 `aria-label` 与 `role="button"`；
  - 颜色选择器必须包含颜色名称的读屏文本（如 `aria-label="红色，画笔"`）；
  - 严格保持与 WCAG AA 标准一致的文本与背景对比度（`--fabushi-text-primary` 对 `--fabushi-bg-layer-1` ≥ 4.5:1）。

### 4.5 `--fabushi-*` 语义设计变量绑定

```css
/* 媒体编辑器与即时预览专用语义设计系统映射 */
.fabushi-media-editor {
  --fabushi-editor-bg: var(--fabushi-surface-scrim, rgba(10, 14, 22, 0.94));
  --fabushi-editor-dock-bg: var(--fabushi-surface-card, rgba(22, 30, 46, 0.88));
  --fabushi-editor-dock-border: var(--fabushi-stroke-glass, rgba(255, 255, 255, 0.08));
  --fabushi-editor-dock-shadow: var(--fabushi-shadow-elevation-3, 0 16px 36px rgba(0, 0, 0, 0.35));
  --fabushi-editor-accent: var(--fabushi-accent-primary, #1084fe);
  --fabushi-editor-accent-hover: var(--fabushi-accent-primary-hover, #0073ea);
  --fabushi-editor-active-bg: var(--fabushi-selection-bg, rgba(16, 132, 254, 0.16));
  --fabushi-editor-text-title: var(--fabushi-text-primary, #f5f7fa);
  --fabushi-editor-text-secondary: var(--fabushi-text-secondary, #98a2b3);
}

.fabushi-instant-view {
  --fabushi-iv-bg-light: #fdfcf9;
  --fabushi-iv-bg-sepia: #f5eedc;
  --fabushi-iv-bg-dark: #0e1117;
  --fabushi-iv-text-light: #1d2939;
  --fabushi-iv-text-sepia: #3d332a;
  --fabushi-iv-text-dark: #e4e7ec;
  --fabushi-iv-border: var(--fabushi-stroke-subtle, rgba(0, 0, 0, 0.08));
  --fabushi-iv-accent: var(--fabushi-accent-primary, #1084fe);
}
```

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 前端 TypeScript 接口定义

```typescript
// frontend/src/recovered/features/conversation/workspace/media-editor/types.ts

export type EditorToolKind = "crop" | "brush" | "mosaic" | "text";
export type CropAspectRatio = "free" | "1:1" | "4:3" | "16:9" | "9:16";

export interface Point2D {
  readonly x: number;
  readonly y: number;
}

export interface BrushStrokeLayer {
  readonly kind: "brush";
  readonly id: string;
  readonly color: string;
  readonly strokeWidth: number;
  readonly points: readonly Point2D[];
}

export interface MosaicRegionLayer {
  readonly kind: "mosaic";
  readonly id: string;
  readonly mode: "pixel" | "blur";
  readonly intensity: number;
  readonly points: readonly Point2D[];
}

export interface TextOverlayLayer {
  readonly kind: "text";
  readonly id: string;
  readonly text: string;
  readonly position: Point2D;
  readonly fontSize: number;
  readonly color: string;
  readonly backgroundColor?: string;
}

export type EditorLayer = BrushStrokeLayer | MosaicRegionLayer | TextOverlayLayer;

export interface EditorTransformMatrix {
  readonly cropRect: { x: number; y: number; width: number; height: number };
  readonly rotationDegrees: 0 | 90 | 180 | 270;
  readonly flipHorizontal: boolean;
}

export interface EditorSessionSnapshot {
  readonly transform: EditorTransformMatrix;
  readonly layers: readonly EditorLayer[];
  readonly timestamp: number;
}

export interface FabushiMediaEditorProps {
  readonly isOpen: boolean;
  readonly originalMediaSrc: string;
  readonly mimeType: string;
  readonly initialFileName: string;
  readonly onApply: (editedBlob: Blob, previewDataUrl: string) => void;
  readonly onCancel: () => void;
  readonly accountScopeGeneration: number;
}
```

```typescript
// source/electron-main/attachments/resumable-transfer-types.ts

export type TransferDirection = "upload" | "download";
export type TransferPhase = "idle" | "preflight" | "transferring" | "paused" | "verifying" | "completed" | "failed" | "cancelled";

export interface TransferChunkRecord {
  readonly partIndex: number;
  readonly offsetBytes: number;
  readonly lengthBytes: number;
  readonly sha256Hex: string;
  readonly state: "pending" | "transferring" | "acked" | "failed";
  readonly retryCount: number;
}

export interface ResumableTransferSession {
  readonly transferId: string;
  readonly direction: TransferDirection;
  readonly localFilePath: string;
  readonly remoteResourceUrl: string;
  readonly totalBytes: number;
  readonly partSizeBytes: number; // 默认 524288 (512 KiB)
  readonly totalParts: number;
  readonly phase: TransferPhase;
  readonly transferredBytes: number;
  readonly speedBytesPerSecond: number;
  readonly parts: readonly TransferChunkRecord[];
  readonly accountScopeGeneration: number;
  readonly sessionKey: string;
  readonly lastError?: string;
}
```

### 5.2 宿主与原生 Rust 数据结构与 RPC 契约

```rust
// source/host/src/extensions/attachments/media_generation_service.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageResolutionGrade {
    Grade1K, // 1024x1024
    Grade2K, // 2048x2048
    Grade4K, // 4096x4096
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalGenerationRequest {
    pub request_id: String,
    pub session_key: String,
    pub account_scope_generation: u64,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub aspect_ratio: String, // "1:1", "16:9", "4:3"
    pub resolution_grade: ImageResolutionGrade,
    pub reference_image_path: Option<String>,
    pub seed: Option<u64>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationPhase {
    Queued { queue_position: u32 },
    Generating { progress_percent: u8 },
    PostProcessing,
    Completed { asset_id: String, local_path: String, sha256: String },
    Failed { error_code: String, error_message: String },
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationProgressEvent {
    pub request_id: String,
    pub session_key: String,
    pub phase: GenerationPhase,
    pub elapsed_ms: u64,
}
```

```rust
// native/mahayana-messaging/src/media_cache.rs

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCacheConfig {
    pub max_cache_bytes: u64,       // 默认 5 GiB (5 * 1024 * 1024 * 1024)
    pub high_watermark_ratio: f32,   // 0.90 触发剔除
    pub low_watermark_ratio: f32,    // 0.75 剔除目标
    pub cache_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCacheEntry {
    pub sha256_hash: String,
    pub mime_type: String,
    pub file_size_bytes: u64,
    pub local_relative_path: String,
    pub last_accessed_timestamp_ms: u64,
    pub hit_count: u64,
}
```

### 5.3 分片传输状态机与数学方程

大文件分片传输基于分块确认与指数退避模型：

1. **分块数量与偏移量计算方程**：
   $$P_{\text{size}} = 512 \times 1024 = 524,288 \text{ bytes}$$
   $$N = \left\lceil \frac{\text{TotalBytes}}{P_{\text{size}}} \right\rceil$$
   $$\text{Offset}(p) = p \times P_{\text{size}}, \quad \text{Length}(p) = \min(P_{\text{size}}, \text{TotalBytes} - \text{Offset}(p))$$
2. **瞬时传输速率加权移动平均（EMA）方程**：
   $$S_t = \alpha \cdot \frac{\Delta \text{Bytes}}{\Delta t} + (1 - \alpha) \cdot S_{t-1}, \quad \text{where } \alpha = 0.3$$
3. **网络故障指数退避重试方程**：
   $$T_{\text{retry}}(k) = \min(T_{\text{max}}, T_{\text{base}} \times 2^k) + \text{Uniform}(-0.2, 0.2) \times T_{\text{base}}$$
   其中 $T_{\text{base}} = 1000\text{ms}, T_{\text{max}} = 30000\text{ms}, k \le 5$。重试超过 5 次则跃迁至 `Failed` 状态并提示用户重试。

### 5.4 跨账号与跨会话 Fencing 围栏规则

1. **`accountScopeGeneration` 单调递增隔离**：
   - 当用户注销或切换账号时，系统触发：
     $$\text{accountScopeGeneration} \leftarrow \text{accountScopeGeneration} + 1$$
   - 所有的活跃上传/下载句柄、多模态生成轮询长连接、以及未完成的分片记录必须在收到事件的单个宏任务周期内全部 `Abort` 熔断；
   - 本地暂存文件与分片账本必须存放在带有账号隔离前缀的路径中：
     $$\text{Path} = \text{AppData/Transfers/} + \text{SHA256}(\text{accountId}) + \text{/} + \text{transferId}$$
2. **`session_key` 范围保护与迟到数据丢弃**：
   - 正在执行的多模态生图任务在其返回值返回时，先比对返回结果中的 `session_key` 与 UI 当前活跃会话；
   - 若 `response.session_key != currentActiveSessionKey`，则仅将生成的资产沉淀进后台沙箱资源数据库，禁止在当前舞台意外弹出或将结果插入错误的对话草稿。

---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 专项施工子智能体与文件写入白名单

为实现本蓝图的工程落地，同时保障多子智能体并行编码时零冲突、零死锁，将实施任务严格切分为 3 个高内聚子智能体（Work Slices）：

#### 切片 A：前端富媒体编辑器与即时预览阅读器（`Worker-11A-UI`）
- **职责**：实现 `FabushiMediaEditorModal` 离屏 Canvas 画布、非破坏性图层历史栈、裁剪旋转手势、马赛克高斯模糊计算、以及 `InstantViewOverlay` 纯净阅读器与 DOM 清洗器。
- **允许修改与新增的文件白名单**：
  - `frontend/src/recovered/features/conversation/workspace/media-editor/FabushiMediaEditorModal.tsx`（新增）
  - `frontend/src/recovered/features/conversation/workspace/media-editor/editor-canvas-engine.ts`（新增）
  - `frontend/src/recovered/features/conversation/workspace/media-editor/editor-history-store.ts`（新增）
  - `frontend/src/recovered/features/conversation/workspace/media-editor/types.ts`（新增）
  - `frontend/src/recovered/features/conversation/workspace/instant-view/InstantViewOverlay.tsx`（新增）
  - `frontend/src/recovered/features/conversation/workspace/instant-view/instant-view-parser.ts`（新增）
  - `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx`（仅限接线：添加“编辑”按钮、快捷键 `E` 唤起编辑器、以及 `media-viewer` 焦点恢复）

#### 切片 B：Electron 桌面断点分片传输管理器（`Worker-11B-Transfer`）
- **职责**：实现 512 KiB 分片并发上传与下载、本地断点续传 SQLite/JSON 账本、网络故障指数退避重试、SHA-256 块校验与本地磁盘预分配。
- **允许修改与新增的文件白名单**：
  - `source/electron-main/attachments/resumable-transfer-manager.ts`（新增）
  - `source/electron-main/attachments/resumable-transfer-types.ts`（新增）
  - `source/electron-main/attachments/attachment-manager.ts`（仅限接线：将大文件 Range 请求转发至 `ResumableTransferManager`）
  - `source/shared/media/attachment-limits.ts`（更新分片常量与字节上限）

#### 切片 C：宿主多模态生成调度与原生媒体缓存（`Worker-11C-HostNative`）
- **职责**：闭环多模态生成调度服务，解决分辨率降级阻塞；实现 Native Rust 多级 LRU 媒体缓存剔除机制与音视频流式 Range 代理。
- **允许修改与新增的文件白名单**：
  - `source/host/src/extensions/attachments/media_generation_service.rs`（核心状态机闭环）
  - `source/host/src/runner/tools/sand_generate_image_tool.rs`（参数透传与 Fallback 矩阵）
  - `native/mahayana-messaging/src/media_cache.rs`（2 级 LRU 淘汰与配额水位线）
  - `source/host/src/media_mime.rs`（Magic Bytes 补充与嗅探规则）

---

### 6.2 只读参考与不可触碰边界（Read-Only Guardrails）

严禁任何子智能体越权写入以下根文件与上游基线：
- **`frontend/src/production/ProductionRenderer.tsx`**：全局根渲染器，仅由主整合者（Integrator）统一写入窄挂载点，子智能体严禁大段插入逻辑；
- **`contracts/desktop-bridge.ts`**：桌面 IPC 核心基准契约，禁止私自篡改未冻结的全局接口；
- **`docs/specs/fusion-blueprints/README.md`** 与 `00-master-capability-and-module-index.md`：全局架构宪章，仅受审查委员会授权维护；
- **Upstream Scratch 仓库**：`/Users/bhrum/.gemini/antigravity/scratch/**` 均为只读参考基准，严禁任何写操作；
- **单文件预算红线**：所有新建或修改的代码文件必须严格维持在 **200–600 行** 内，违反将被自动审计拒绝。

---

### 6.3 与生产装配入口的窄接线契约（Narrow Wiring Contract）

各切片对既有系统的影响必须通过极简的窄接口进行装配挂载：

```typescript
// ProductionRenderer.tsx 或 ConversationWorkspace 中的窄挂载接线点

// 1. 媒体编辑器挂载窄契约
import { FabushiMediaEditorModal } from "../recovered/features/conversation/workspace/media-editor/FabushiMediaEditorModal";

// 仅在用户点击编辑或发送前修图时以 Portal 形式条件渲染：
{activeEditingMedia != null && (
  <FabushiMediaEditorModal
    accountScopeGeneration={accountScopeGeneration}
    initialFileName={activeEditingMedia.name}
    isOpen={true}
    mimeType={activeEditingMedia.mimeType}
    onApply={(editedBlob, previewUrl) => {
      handleCommitEditedMedia(editedBlob, previewUrl);
      setActiveEditingMedia(null);
    }}
    onCancel={() => setActiveEditingMedia(null)}
    originalMediaSrc={activeEditingMedia.src}
  />
)}

// 2. 即时文章阅读器挂载窄契约
import { InstantViewOverlay } from "../recovered/features/conversation/workspace/instant-view/InstantViewOverlay";

{activeInstantViewUrl != null && (
  <InstantViewOverlay
    accountScopeGeneration={accountScopeGeneration}
    articleUrl={activeInstantViewUrl}
    isOpen={true}
    onClose={() => setActiveInstantViewUrl(null)}
  />
)}
```

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 正反例审计对照检查表

独立审计子智能体（Auditor Subagent）将对照以下正反例标准逐条核验，任何违例将被无条件判定为审计不合格：

| 校验维度 | 正例标准（Pass Criterion） | 反例硬编码与作弊判定（Fail Violation） |
|---|---|---|
| **1. 涂鸦编辑真实验证** | Canvas 上的每次笔刷拖动与区域马赛克均生成实际的矢量层数据，并正确推入 `undoStack`；点击 Undo 画面真实撤销对应图层；应用后导出真实包含修改的 PNG/JPEG Blob | 只有静态工具栏 UI，点击画笔在画布上拖拽无实际像素变化；或者 Undo 按钮仅做 `setCanUndo(false)` 视觉置灰而无数据回滚 |
| **2. 分片续传真实状态** | 大文件传输按 512 KiB 分块切割并按顺序发送；断开网络时暂停传输并在本地保留 `.part` 分片文件与账本；网络恢复后从断点 Part 继续，无重复网络流量 | 使用 `setTimeout` 每 100ms 将进度自增 1% 模拟平滑上传；断网时没有持久化账本，重新传输仍从第 0 字节开始 |
| **3. 多模态生成状态单调性** | 真实监听宿主 `GenerationProgressEvent`，严格依据服务端返回的 `phase` 更新状态卡片；点击取消立即向 Host 发送 Abort 信号并终止外部 API 轮询 | 前端写死 3000ms 后自动将状态从 `Generating` 变为 `Completed`；取消按钮仅关闭弹窗，后台仍持续消耗 Token 轮询 |
| **4. 生图参数完整透传** | `1K`、`2K`、`4K` 经 `normalize_image_generation_resolution` 规范化后，完整注入到 `MultimodalGenerationRequest` 中；遇到下游 Proto 限制时，执行本地透明降级并如实记录日志 | 在 TS 代码中直接硬编码 `resolution = "1024x1024"` 绕过归一化校验；或者在接收到非法输入时静默忽略错误 |
| **5. Instant View 安全清洗** | 外部文章解析器使用白名单安全过滤，彻底剔除 `<script>`、`<iframe onerror>`、`<link rel="stylesheet">` 等一切潜在 XSS 载荷，纯净渲染标准文本与受控图片 | 使用 `dangerouslySetInnerHTML` 原样注入外部网页抓取的原始 HTML 字符串；或者未对第三方外链进行协议安全过滤 |
| **6. 缓存配额与 LRU 淘汰** | 当本地缓存目录占用超过 `high_watermark_ratio`（90%）时，自动按照最后访问时间升序（最久未访问优先）物理删除旧缓存文件，直到降至 `low_watermark_ratio`（75%） | 仅在内存中维护 Map 计数，从未计算物理磁盘占用；或者无限追加新缓存直至操作系统报 `ENOSPC` 磁盘耗尽 |
| **7. 账号隔离原子性** | 切换账号或注销时，递增 `accountScopeGeneration`，在途的所有分片下载任务、Canvas 待编辑草稿与生成任务立即原子 Abort 并清空内存引用 | 账号 A 切换为账号 B 后，账号 B 仍能看到账号 A 正在下载的文件进度条或未完成的媒体编辑画布 |
| **8. 契约类型安全** | 全量 TypeScript 接口字段均有严密类型声明，严禁出现 `any` 逃逸；Rust 结构体具备严格的 `Serialize`/`Deserialize` 派生与单元测试契约 | 使用 `// @ts-ignore` 压制类型错误；或者接口中存在 `params: Record<string, any>` / `TODO` 逃逸字段 |
| **9. 无障碍与键盘焦点** | 模态框打开时自动聚焦首个操作元素，`Escape` 键平滑退出并调用 `restoreFocus()` 将焦点安全返还触发按钮；所有颜色与工具均有清晰读屏标注 | 模态框关闭后焦点丢失至 `document.body`；键盘 Tab 键可穿透模态框聚焦到底层聊天界面的隐藏输入框 |
| **10. 单文件行数内聚预算** | 所有实施文件均严格控制在 **200–600 行** 规范预算范围内，高内聚低耦合 | 单一文件超过 600 行成为新的巨石组件；或者过度拆分出不足 50 行的碎片空壳文件 |

---

### 7.2 静态审查与验收命令（Verification Methods）

由于本分支严格执行无本地运行命令纪律（No Local Command Execution），独立审计子智能体必须通过以下静态核验规程实施完备验收：

1. **结构完整性核验**：
   - 检查 `docs/specs/fusion-blueprints/11-multimodal-generation-media-editor-and-transfers.md` 是否完整包含 7 大强制章节；
   - 检查章节 1 的源码五问（Q1..Q5）是否均提供确凿的真实文件路径与精确行号证据；
   - 检查章节 3 中的单文件预算是否严格落入 200–600 行区间。
2. **三方契约对齐核验**：
   - 对比 Telegram Desktop `storage/file_upload.cpp` 与 `source/electron-main/attachments/resumable-transfer-types.ts`，确认 512 KiB 分块大小与 Part 索引逻辑 100% 对齐；
   - 对比 OpenClaw `src/image-generation/resolution.ts` 与 `source/host/src/extensions/attachments/media_generation_service.rs`，确认 1K/2K/4K 分辨率阶梯严格镜像映射。
3. **安全围栏与 Fencing 核验**：
   - 检查 `accountScopeGeneration` 与 `session_key` 在前端 Props、IPC 请求、以及 Host 结构体中是否全链路显式透传，不存在断链遗漏。

---

### 7.3 签署与权威封签（Sign-off & Sealed Baseline）

- **蓝图编号**: `FBCP-BLUEPRINT-11` / `UFX-BLUEPRINT-11`
- **编写负责人**: `Worker-Blueprint-11`（Teamwork Authoritative Design Specialist）
- **复核标准**: `docs/specs/fusion-blueprints/README.md`
- **发布状态**: **APPROVED & SEALED FOR M2/M3 STAGE 3 IMPLEMENTATION**
- **时间戳**: `2026-11-05T14:30:00Z`
