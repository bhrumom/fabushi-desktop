# Fabushi 统一融合蓝图 10：全双工实时语音、Talk Mode、唤醒词与音频引擎
# Volume 10: Realtime Voice, Talk Mode, Audio Capabilities, and Waveforms

> **规范编号**: `FBCP-BLUEPRINT-10` / `UFX-BLUEPRINT-10`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **责任域对齐**:
> - `OCRM-M25`（Talk Mode / Voice Engine: 全双工实时语音、VAD 语音活动检测、Local whisper.cpp / sherpa-onnx STT、ElevenLabs / Piper / Edge TTS、Barge-in 打断）
> - `OCRM-M26`（Audio Capabilities & Wake Words: 本地唤醒词检测、系统声音回放/Loopback 捕获、降噪抑制、会议录制与说话人分离转录机器人）
> - `TCG-N10`（Telegram Desktop 5-bit 紧凑波形编码、语音消息播放器与录制手势条）
> **权威源码与合约参照**:
> - Native 合约测试: `source/host/tests/openclaw_audio_talk_contract.rs:1-827`（12 项不可伪造测试预言机 O01–O12）
> - 消息体媒体结构: `native/mahayana-messaging/src/message.rs:412-416`（`MessageContent::Voice { media, caption, waveform }`）
> - 既有语音前端控制器: `frontend/src/recovered/features/conversation/workspace/voice.tsx:1-487`（`VoiceSessionController`, `VoiceWaveform`）

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照与 Canonical Owner 责任对齐矩阵

| 功能项与能力域 | Telegram Desktop 源码锚点 | OpenClaw 源码锚点 | Fabushi 既有代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **全双工实时语音与 VAD 打断**<br>(Full-Duplex Talk Mode & Barge-in) | `Telegram/SourceFiles/calls/calls_call.cpp:205-320`<br>`calls/calls_controller_webrtc.cpp:110-180` | `src/talk/talk-session-controller.ts:1-222`<br>`src/talk/exact-speech-protocol.ts:1-94` | `source/host/tests/openclaw_audio_talk_contract.rs:608-646` (`MockPlaybackSession`)<br>`frontend/src/recovered/features/conversation/workspace/voice.tsx:1-150` | `RealtimeTalkCoordinator`<br>`frontend/src/production/talk/TalkModeOrbModal.tsx`<br>`native/mahayana-messaging/src/realtime.rs` |
| **混合语音转录与合成流水线**<br>(Local/Cloud STT & TTS Pipeline) | `Telegram/SourceFiles/media/audio/media_audio.cpp:80-240`<br>`history/view/media/history_view_voice.cpp` | `src/tts/tts-manager.ts:1-180`<br>`src/realtime-transcription/websocket-session.ts` | `source/host/tests/openclaw_audio_talk_contract.rs:762-827` (`SandTranscribeEmptyAudioError`, Edge TTS CJK 30%) | `SpeechTranscriptionService`<br>`SpeechSynthesisEngine`<br>`source/host/src/runner/mod.rs` |
| **本地唤醒词与硬件音频采集**<br>(Wake Words & Loopback Audio) | `Telegram/SourceFiles/platform/mac/touchbar/mac_touchbar_audio.mm`<br>`media/audio/media_audio_capture.cpp` | `src/node-host/audio-loopback-capture.ts:1-160`<br>`src/wake-word/openwakeword-runner.ts` | `frontend/src/recovered/features/conversation/workspace/voice.tsx:14-18` (`AUDIO_CONSTRAINTS`)<br>`source/host/src/ports/mod.rs:AudioOutputPort` | `HardwareAudioCaptureManager`<br>`WakeWordDetector`<br>`source/host/src/runner/computer_use.rs` |
| **多方会议录制与说话人分离机器人**<br>(Meeting Bot & Live Diarization) | `Telegram/SourceFiles/calls/calls_group_panel.cpp:150-320`<br>`calls/calls_group_call.cpp:400-610` | `src/meeting-bot/meeting-recorder.ts:1-310`<br>`src/transcripts/diarization-merger.ts:1-190` | `source/node-agent-coordinator/src/tdrp/calls_group_calls.rs:27-44,140-210`<br>`native/mahayana-messaging/src/realtime.rs:34-42` | `MeetingRecordingBotCoordinator`<br>`source/host/src/runner/mod.rs` (`TurnContextTracker`) |
| **5-bit 紧凑波形与流式播放器**<br>(5-Bit Waveform & Player) | `Telegram/SourceFiles/media/audio/media_audio_waveform.cpp:1-120`<br>`history/view/media/history_view_voice.cpp:45-180` | `src/media/waveform-visualizer.ts:1-85` | `native/mahayana-messaging/src/message.rs:412-416` (`waveform: Vec<u8>`)<br>`frontend/src/recovered/features/conversation/workspace/voice.tsx:422-487` | `VoiceWaveformRenderer`<br>`frontend/src/production/chat/VoiceMessagePlayer.tsx` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **OpenClaw 语音与 Talk 架构**：
  OpenClaw 采用基于 WebRTC Audio Track 与实时 WebSocket 的双工架构。在 `talk-session-controller.ts` 中维护了会话世代轮转（Generation Fencing）、320 音频块 / 1 MiB 缓冲区硬上限（`openclaw_audio_talk_contract.rs:440-445`）、SharedArrayBuffer 32 位原子关闭隔离（`AtomicI32` close fence, `openclaw_audio_talk_contract.rs:496-543`），以及基于 VAD 的立即打断（Barge-in，`openclaw_audio_talk_contract.rs:608-646`）。TTS 提供 CJK 占字符 30% 门限的 Edge TTS / Piper 自动语言回退（`openclaw_audio_talk_contract.rs:787-826`）。
- **Telegram Desktop 语音与波形架构**：
  Telegram Desktop 在 `media/audio/media_audio_waveform.cpp` 与 `history_view_voice.cpp` 中将音频能量压缩为每个采样点 5-bit（0..31）的连续字节流，通过 100 个波峰值精准还原音频振幅，支持录制手势上锁、滑动取消、变速播放（1x / 1.5x / 2x）与基于时间戳的消息定位。

#### Q2（现有链路与已有实现定位）
1. **测试与协议基准**：`source/host/tests/openclaw_audio_talk_contract.rs:1-827` 已经原生完整实现了 12 项合约测试，包含 generation 递增重连、信令序号跨度拦截、幂等重放、终态防篡改、320 块溢出截断、原子栅栏、防复读 Sentinel 提取、VAD 打断等。
2. **前端音频录制控制器**：`frontend/src/recovered/features/conversation/workspace/voice.tsx:134-354` 提供了 `VoiceSessionController`，包含状态机监听、录音上限时间限制（300s）、WebM/Opus MIME 协商以及 Canvas 动态频率渲染。
3. **Rust 数据核心**：`native/mahayana-messaging/src/message.rs:412-416` 的 `MessageContent::Voice` 显式持有 `waveform: Vec<u8>` 与 `MediaRef`。

#### Q3（现有断点与割裂根因诊断）
- **断点 1：单工录音与全双工 Talk 割裂**。既有前端仅有静态录音组件 `voice.tsx`，无法与大模型进行像真人对话般的实时双工对话（Talk Mode），缺少打断反馈与交互光球。
- **断点 2：缺少 5-bit 紧凑波形互通转换器**。前端 `voice.tsx` 动态提取的是 256-bin FFT 字节流，发送至后端时未序列化为 Telegram 标准的 5-bit 紧凑波形字节数组，导致桌面原生端与 Web 端波形失真。
- **断点 3：缺乏打断（Barge-in）协同**。当 AI 语音播放过程中用户重新开口时，播放流无法在 100ms 内静音并重置会话上下文，导致大模型自言自语产生回环幻觉。

#### Q4（状态机流转与安全围栏）
- 严格遵循 `openclaw_audio_talk_contract.rs` 的 12 项不可伪造预言机：
  1. 世代围栏（Generation Fencing）：重连即自增 `generation` 并置零 `signalSeq`，彻底丢弃旧世代任何延迟抵达的音频数据包。
  2. 队列安全上限：限制 `REALTIME_VOICE_MAX_PENDING_AUDIO_CHUNKS = 320` 与 `1 MiB` 字节上限，杜绝内存爆仓。
  3. 终态不可逆：一旦进入 `Ended` / `Failed` 终态，拒绝任何后续音频帧推送与信令流转。
  4. 文本防复读协议：匹配 `Internal OpenClaw voice playback result.` 专用哨兵指令，杜绝语音合成与工具调用死循环。

#### Q5（UI/UX 极致体验落点）
- 在输入框右侧提供语音录制手势条，支持上滑锁定持续录制、左滑取消、波形实时跃动。
- 在应用顶栏或全局悬浮入口提供独立的“实时对话（Talk Mode）”能量光球（Floating Orb），支持 3 种视觉形态（Idle 呼吸光晕、Listening 弹性波纹、Speaking 多彩能量流动），并可一键切换为背景常驻最小化浮动胶囊。


---

## 2. 现有割裂缺陷与根因分析（Existing Defects & Root Cause Analysis）

### 2.1 缺陷全景清单

| 缺陷编号 | 现象与影响 | 涉及模块与文件定位 | 根因分类与机理分析 |
|---|---|---|---|
| **BUG-10-01** | **单工与全双工割裂**：用户只能先长按录音、等待整段转录、再发起文本提问，无法进行即时全双工打断对话 | `frontend/src/recovered/features/conversation/workspace/voice.tsx:134-220`<br>`frontend/src/production/openclaw/OpenClawCleanComposer.tsx` | **架构模型陈旧**：既有逻辑仅实现了单向的“录音上传转录”单工通道，缺少 WebSocket/WebRTC 实时双工语音流与音频输出管道 |
| **BUG-10-02** | **缺少 Barge-in 打断反馈**：AI 正在播放语音回复时，用户插话无法立即打断播放，产生音频回声串扰 | `source/host/tests/openclaw_audio_talk_contract.rs:608-646`<br>`frontend/src/recovered/features/conversation/workspace/voice.tsx` | **状态机未闭环**：未接线 WebRTC VAD 事件 `input_audio_buffer.speech_started`，前端缺少原子关闭栅栏与音频缓冲区即时刷新机制 |
| **BUG-10-03** | **波形编码不兼容**：发送语音消息后，桌面原生端无法显示 5-bit 波形，波形柱高度全平或闪烁 | `native/mahayana-messaging/src/message.rs:412-416`<br>`Telegram/SourceFiles/media/audio/media_audio_waveform.cpp:1-60` | **数据契约断层**：Web 端直接使用 Canvas 绘制瞬时 FFT 频谱，未在录音结束时按照 Telegram 算法压缩为 100 采样点的 5-bit 紧凑波形二进制 |
| **BUG-10-04** | **系统声音 Loopback 越权与泄漏**：屏幕共享或唤醒词监听未隔离多租户上下文，导致敏感音频串流 | `source/host/src/runner/computer_use.rs`<br>`source/host/tests/openclaw_audio_talk_contract.rs:51-147` | **缺乏 Scope 世代围栏**：音频采集管道未绑定 `accountScopeGeneration`，跨会话或切换账号时未强制注销硬件捕获 Track |
| **BUG-10-05** | **会议录制缺少静音修剪**：长时间多方会议录制产生大量无声数据块，导致转录 token 浪费与内存溢出 | `source/host/tests/openclaw_audio_talk_contract.rs:440-492,648-693`<br>`native/mahayana-messaging/src/realtime.rs` | **生命周期管理缺失**：未落地 `TurnContextTracker::prune_silent_turns` 机制，未在 320 块与 1 MiB 上限前自动清除静音历史回合 |

---

## 3. 统一架构与职责归属（Unified Architecture & Line Budgets）

### 3.1 架构设计原则与 Single Composition 拓扑
1. **单一装配（Single Composition）**：全双工实时 Talk Mode、语音消息录制、唤醒词监听与多方会议录制统一汇聚至 `native/mahayana-messaging/src/audio_engine.rs` 与 `RealtimeTalkCoordinator`。
2. **账号与世代隔离（Generation Fencing）**：任何账号切换、会话注销或重连均递增 `accountScopeGeneration`，原子关闭旧 AudioContext 与 WebRTC MediaStream。
3. **同主不同室（Same Owner != Same File）**：每个核心类与模块保持在 200–600 行严格内聚预算内，杜绝巨石文件。

### 3.2 200–600 行内聚预算与文件拓扑

| 文件相对路径 | 语言 | 行数预算 | 核心职责与边界说明 |
|---|---|---|---|
| `frontend/src/production/talk/TalkModeOrbModal.tsx` | TSX | 260–340 行 | 浮动 Talk Mode 能量光球 UI、三态光晕粒子动画、全屏与最小化切换 |
| `frontend/src/production/talk/RealtimeTalkCoordinator.ts` | TS | 320–420 行 | 全双工信令调度、VAD 打断、世代围栏与 WebRTC AudioTrack 声明周期控制 |
| `frontend/src/production/chat/VoiceMessagePlayer.tsx` | TSX | 240–310 行 | 语音消息气泡播放器、5-bit 紧凑波形渲染、进度拖动、变速与转录文本折叠展开 |
| `frontend/src/production/audio/WaveformCodec5Bit.ts` | TS | 200–260 行 | Telegram 兼容的 5-bit（0..31）音频能量采样、压缩打包与插值解压算法库 |
| `native/mahayana-messaging/src/audio_engine.rs` | Rust | 350–450 行 | 实时音频管道状态机、320 块/1 MiB 队列管理、静音回合修剪与音频多路复用 |
| `source/host/src/runner/talk_vad_bargein.rs` | Rust | 280–380 行 | 桥接 Whisper.cpp / Sherpa-onnx STT、Edge/Piper TTS 与 VAD 打断哨兵校验 |


---

## 4. 极致 UI/UX 与交互设计规范（UI/UX, Talk Mode Orb & Design Tokens）

### 4.1 浮动 Talk Mode 能量光球（The Floating Talk Mode Orb）

1. **三维能量光球视觉形态**：
   - **Idle（待机）**：36px 呼吸光晕，中心微弱透明光环，周期 3.2s，低功耗等待唤醒或触控。
   - **Listening（倾听用户）**：随用户说话音量产生径向脉冲扩展（Scale 1.0 $\to$ 1.35），外圈呈现绿色或青色高频声波涟漪。
   - **Speaking（AI 语音回复）**：全彩流光能量球（紫-蓝-琥珀色梯度流动），基于 TTS 输出频段实时形变，外放音频能量映射为光球表面起伏。
   - **Interrupting（打断瞬间）**：当 VAD 检测到用户在 AI 说话时开口，光球在 60ms 内瞬间聚拢微缩并闪烁一次，随后无缝切换至 Listening 状态。
2. **多模态视图切换**：
   - **全屏沉浸模式**：居中 160px 超大光球，顶部展示实时转录双语字幕，底部展示静音、挂断与文字输入入口。
   - **浮动胶囊模式（PIP）**：缩小为屏幕右下角 48px $\times$ 48px 浮动圆球，在用户浏览其他聊天或执行复杂自动化任务时保持后台语音常驻。

### 4.2 Telegram 风格语音录制手势条与 5-bit 波形交互

1. **录制手势条交互体系**：
   - **长按录音**：按住输入框右侧麦克风图标，震动反馈后即刻开始采集，左侧滑出手势取消提示。
   - **上滑锁定（Lock Hands-free）**：手指向上滑动 48px，麦克风锁定为免提录制模式，释放双手。
   - **左滑取消（Slide to Cancel）**：手指向左滑动 80px，立即取消录音并以碎纸机动画销毁音频缓存。
2. **5-bit 紧凑波形与播放器控件**：
   - 气泡内固定渲染 100 根波峰柱（柱宽 2px，间距 1.5px），高度范围 4px 至 24px，精准映射 5-bit（0..31）振幅。
   - 播放进度双色覆盖：已播放柱体使用 `--fabushi-accent-primary`，未播放柱体使用 `--fabushi-border-subtle`。
   - 支持实时进度拖拽（Scrubbing）、变速切换（1x $\to$ 1.5x $\to$ 2x）与一键折叠展开转录文字。

### 4.3 设计变量规范（Design Tokens）与无障碍标准

```css
/* Fabushi Realtime Voice & Waveform Design Tokens */
:root {
  --fabushi-talk-orb-idle: rgba(99, 102, 241, 0.45);
  --fabushi-talk-orb-listening: rgba(16, 185, 129, 0.85);
  --fabushi-talk-orb-speaking-start: #6366f1;
  --fabushi-talk-orb-speaking-end: #ec4899;
  --fabushi-waveform-played: var(--fabushi-accent-primary, #2563eb);
  --fabushi-waveform-unplayed: var(--fabushi-border-subtle, rgba(148, 163, 184, 0.35));
  --fabushi-waveform-hover: var(--fabushi-accent-hover, #1d4ed8);
  --fabushi-voice-recording-red: #ef4444;
}
```

- **响应式断点**：
  - $\ge 1200\text{px}$（宽屏桌面）：Talk Mode 支持作为右侧第三栏停靠，或以 480px 浮窗居中渲染。
  - $900\text{px} \sim 1199\text{px}$（平板/分屏）：自动折叠至右下角浮动光球胶囊。
  - $< 900\text{px}$（移动窄屏）：全屏独占沉浸式模式，底部固定单手手势安全区。
- **无障碍（a11y）合规**：
  - 光球与播放器具备标准 `aria-live="polite"` 与 `aria-label="实时语音对话，当前正在倾听"`。
  - 键盘快捷键：`Space` 触发/暂停播放或切换临时静音，`Escape` 最小化 Talk Mode 光球。


---

## 5. 精确接口契约与状态机流转（Exact Contracts, FSM & Fencing）

### 5.1 数学状态转移方程（Talk Mode & Audio Pipeline FSM）

$$\text{TalkState} \in \{\text{Idle}, \text{Listening}, \text{Thinking}, \text{Speaking}, \text{Interrupted}, \text{Reconnecting}, \text{TerminalClosed}\}$$

- $\text{Idle} \xrightarrow{\text{evt: user\_start} \lor \text{evt: wake\_word\_detected}} \text{Listening}$
- $\text{Listening} \xrightarrow{\text{evt: vad\_speech\_stop}} \text{Thinking}$（提交最后音频分片至 STT）
- $\text{Thinking} \xrightarrow{\text{evt: tts\_first\_chunk}} \text{Speaking}$（启动流式音频播放通道）
- $\text{Speaking} \xrightarrow{\text{evt: vad\_barge\_in}} \text{Interrupted} \xrightarrow{\tau \le 50\text{ms}} \text{Listening}$（原子取消播放、清空音频队列并重置当前回合）
- $\text{Speaking} \xrightarrow{\text{evt: tts\_playback\_ended}} \text{Listening}$（维持长连继续倾听）
- $\forall S \setminus \{\text{TerminalClosed}\} \xrightarrow{\text{evt: disconnect}} \text{Reconnecting} \xrightarrow{\text{evt: resume\_ok}} \text{Listening}$
- $\text{Reconnecting} \xrightarrow{\Delta t > 30\text{s}} \text{TerminalClosed}$
- $\forall S \xrightarrow{\text{evt: account\_switch} \lor \text{evt: user\_exit}} \text{TerminalClosed}$（递增 `accountScopeGeneration`，终态单调锁死）

### 5.2 TypeScript 强类型契约定义

```typescript
// frontend/src/production/talk/RealtimeTalkContracts.ts

export type TalkModeState = "idle" | "listening" | "thinking" | "speaking" | "interrupted" | "reconnecting" | "terminal_closed";

export interface TalkModeOrbProps {
  readonly state: TalkModeState;
  readonly rmsVolume: number; // 0.0 .. 1.0 用于驱动声波振幅
  readonly isMuted: boolean;
  readonly isCompact: boolean;
  readonly onToggleMute: () => void;
  readonly onToggleCompact: () => void;
  readonly onClose: () => void;
}

export interface Waveform5BitData {
  readonly samples: Uint8Array; // 100 个 5-bit (0..31) 采样点
  readonly durationMs: number;
}

export interface VoiceMessagePlayerProps {
  readonly voiceMediaId: string;
  readonly waveform: Uint8Array; // 来自 MessageContent::Voice.waveform
  readonly durationMs: number;
  readonly localPath?: string;
  readonly remoteUrl?: string;
  readonly transcriptText?: string;
}
```

### 5.3 Rust 契约与 RPC 结构体定义

```rust
// native/mahayana-messaging/src/audio_engine.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TalkSessionState {
    Idle,
    Listening,
    Thinking,
    Speaking,
    Interrupted,
    Reconnecting,
    TerminalClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioEngineSignal {
    pub call_id: String,
    pub generation: u32,
    pub seq: u64,
    pub action: AudioEngineAction,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioEngineAction {
    StartTalk,
    VadBargeIn,
    PushAudioChunk,
    CancelTts,
    CloseSession,
}

pub const MAX_PENDING_AUDIO_CHUNKS: usize = 320; // 对应 OCRM-AUDIO-O05
pub const MAX_PENDING_AUDIO_BYTES: usize = 1_048_576; // 1 MiB 缓冲区硬上限
```

### 5.4 世代隔离与防复读 Fencing 规则
1. **世代隔离（Generation Fencing）**：Rust 网关在接收到任何 `AudioEngineSignal` 时，比对当前会话的 `generation`；若 `sig.generation < current_generation`，严格抛出 `SessionGatewayError::Internal("stale call generation")` 丢弃（参考 `openclaw_audio_talk_contract.rs:140-145`）。
2. **防自言自语复读（Exact Speech Loop Prevention）**：通过提取并校验 `Internal OpenClaw voice playback result.` 与 `Do not call openclaw_agent_consult...` 哨兵，若返回内容命中历史 exact speech，则判定为 `ExactSpeechEcho` 并直接短路，不重走大模型推理（参考 `openclaw_audio_talk_contract.rs:562-583`）。


---

## 6. 子智能体实施切片与文件写入边界（Implementation Slices & Whitelists）

### 6.1 五阶段实施切片规划（SLICE-10-A 至 SLICE-10-E）

| 切片编号 | 阶段主题 | 核心交付目标与接线动作 | 依赖前置 |
|---|---|---|---|
| **SLICE-10-A** | **5-bit 波形与气泡播放器** | 实现 `WaveformCodec5Bit.ts` 与 `VoiceMessagePlayer.tsx`，将 Telegram 5-bit 紧凑波形渲染到聊天气泡，支持变速与洗带定位 | 无 |
| **SLICE-10-B** | **Talk Mode 能量光球** | 交付 `TalkModeOrbModal.tsx` 与 CSS 粒子光晕动画，实现 Idle / Listening / Speaking / Interrupted 视觉流转与 PIP 最小化 | SLICE-10-A |
| **SLICE-10-C** | **前端全双工 Talk 协调器** | 编写 `RealtimeTalkCoordinator.ts`，接线 VAD 打断、MediaStream 采集、音频上下文清理与世代隔离 | SLICE-10-B |
| **SLICE-10-D** | **Rust 音频引擎与世代围栏** | 在 `native/mahayana-messaging/src/audio_engine.rs` 中实现 320 块/1 MiB 缓冲区保护与原子关闭栅栏 | SLICE-10-C |
| **SLICE-10-E** | **STT/TTS 桥接与会议录制机器人** | 交付 `source/host/src/runner/talk_vad_bargein.rs`，对齐 `openclaw_audio_talk_contract.rs` 12 项不可伪造预言机 | SLICE-10-D |

### 6.2 严格文件修改白名单（Write Whitelist）
任何实施 Worker 仅允许写入或修改以下白名单路径，严禁跨界修改其他业务文件：
- `frontend/src/production/talk/TalkModeOrbModal.tsx`
- `frontend/src/production/talk/RealtimeTalkCoordinator.ts`
- `frontend/src/production/talk/RealtimeTalkContracts.ts`
- `frontend/src/production/chat/VoiceMessagePlayer.tsx`
- `frontend/src/production/audio/WaveformCodec5Bit.ts`
- `native/mahayana-messaging/src/audio_engine.rs`
- `source/host/src/runner/talk_vad_bargein.rs`

---

## 7. 防作弊审计与验收标准（Anti-Cheating Checklist & Acceptance Gates）

### 7.1 正反例防作弊检查表

| 检查维度 | ❌ 严禁的作弊违规实现（Counter Example） | ✅ 合规的真实生产实现（Positive Example） |
|---|---|---|
| **全双工 Talk 状态** | 使用 `setTimeout(() => setStatus("speaking"), 1000)` 伪造正在对话 | 基于真实 WebRTC 音频流与 WebSocket 实时帧事件驱动状态流转 |
| **打断机制（Barge-in）** | 前端仅将本地静音，后台 TTS 仍持续消耗 token 并在网络层传输 | 触发 `vad_barge_in` 即刻向 Rust 发送 `CancelTts`，后端清空 320 块队列并重置回合 |
| **波形编码** | 使用 `Math.random() * 31` 随机生成波峰高度充当视觉波形 | 从实际 PCM 音频采样的 RMS 振幅计算，精确映射至 5-bit（0..31）整数并编码进 `waveform: Vec<u8>` |
| **世代与重连隔离** | 忽视信令世代，接受陈旧连接推过来的延迟音频数据 | 校验 `sig.generation == current_generation`，陈旧世代严格抛出 `stale call generation` 错误 |
| **音频内存保护** | 无限追加 `VecDeque<Vec<u8>>`，无上限缓存网络推流 | 硬性限制 320 块与 1 MiB 缓冲区（`REALTIME_VOICE_MAX_PENDING_AUDIO_CHUNKS`），超限拒绝并报错 |

### 7.2 交付验收门禁（Acceptance Gates）
- **Gate 1（编译与契约通过）**：所有新增 TypeScript 与 Rust 类型定义零 `any`、零 `TODO` 占位，所有字段具有明确的 optionality 与注释说明。
- **Gate 2（预言机全绿挂接）**：`source/host/tests/openclaw_audio_talk_contract.rs` 中 12 项预言机（O01 至 O12）全部与架构模块建立显式引用与断言映射。
- **Gate 3（架构预算达标）**：新建文件均严格落在 200–600 行内聚预算区间，无任何新增巨石文件。
- **Gate 4（UI/UX 体验闭环）**：Talk Mode 能量光球具备三种动态形态，Telegram 风格 5-bit 波形支持拖拽定位与变速播放，实现真实无缝交互体验。
