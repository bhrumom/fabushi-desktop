import { useState } from "react";
import { SandButton, SandIcon } from "../../recovered/ui/sand-kit-primitives";
import type { DevicePairingEntry, MultimodalVoiceState, NodeHostCapability } from "./types";

interface NodesMultimodalSectionProps {
  readonly agentId: string;
}

const INITIAL_CAPABILITIES: NodeHostCapability[] = [
  { id: "system.run", name: "系统命令执行 (system.run)", description: "受限沙箱本地命令执行环境", enabled: true, activeExecutions: 0 },
  { id: "browser.proxy", name: "浏览器代理 (browser.proxy)", description: "Playwright/ChromeDevTools 自动化控制", enabled: true, activeExecutions: 0 },
  { id: "canvas", name: "动态画布渲染 (canvas)", description: "前端多维交互卡片与 Mini App 宿主", enabled: true, activeExecutions: 1 },
];

const INITIAL_DEVICES: DevicePairingEntry[] = [
  { id: "dev-mac-01", deviceName: "Fabushi Companion (Mac)", platform: "macos", requestedAt: "1d ago", status: "approved" },
  { id: "dev-ios-02", deviceName: "Mobile Client (iOS)", platform: "ios", requestedAt: "3h ago", status: "approved" },
];

const INITIAL_VOICE: MultimodalVoiceState = {
  audioTranscriptionEngine: "Local Whisper Turbo",
  imageDocInspectionEngine: "Vision Multimodal (Built-in)",
  videoAnalysisEngine: "Frame Sampling OCR",
  talkVoiceEngine: "Piper Neural TTS (zh-CN-Xiaoxiao)",
  talkSpeedRate: 1.0,
  talkPitch: 1.0,
};

export function NodesMultimodalSection({ agentId: _agentId }: NodesMultimodalSectionProps) {
  const [capabilities, setCapabilities] = useState<NodeHostCapability[]>(INITIAL_CAPABILITIES);
  const [devices, setDevices] = useState<DevicePairingEntry[]>(INITIAL_DEVICES);
  const [voice, setVoice] = useState<MultimodalVoiceState>(INITIAL_VOICE);
  const [pairCode, setPairCode] = useState("");
  const [toast, setToast] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  const toggleCapability = (id: string) => {
    setCapabilities((prev) =>
      prev.map((c) => (c.id === id ? { ...c, enabled: !c.enabled } : c))
    );
    showToast("节点能力配置已更新");
  };

  const handlePairSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!pairCode.trim()) return;
    const newDev: DevicePairingEntry = {
      id: `dev-${Date.now().toString(36)}`,
      deviceName: `新配对设备 (${pairCode.slice(0, 4)})`,
      platform: "web",
      requestedAt: "刚刚",
      status: "approved",
    };
    setDevices((prev) => [newDev, ...prev]);
    setPairCode("");
    showToast("设备配对成功，已授予节点互操作权限。");
  };

  const handleRevokeDevice = (id: string) => {
    setDevices((prev) => prev.filter((d) => d.id !== id));
    showToast("已撤销该设备配对。");
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      {toast && (
        <div className="sand-agent-action-toast">
          <SandIcon name="server" size="sm" />
          <span>{toast}</span>
        </div>
      )}

      {/* Node Capabilities Card */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>节点宿主能力 (Node-Host Capabilities)</span>
        </div>
        <p style={{ margin: "0 0 10px", fontSize: 11, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
          向 Agent 暴露本地执行与代理能力，严格受租约时效与审计门禁保护。
        </p>

        {capabilities.map((cap) => (
          <div key={cap.id} className="sand-runtime-switch-row">
            <div>
              <div style={{ fontWeight: 500, fontSize: 11.5 }}>{cap.name}</div>
              <div style={{ fontSize: 10.5, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
                {cap.description} {cap.activeExecutions > 0 && `• 活动中 (${cap.activeExecutions})`}
              </div>
            </div>
            <input
              type="checkbox"
              checked={cap.enabled}
              onChange={() => toggleCapability(cap.id)}
              style={{ width: 18, height: 18, cursor: "pointer", accentColor: "#3b82f6" }}
            />
          </div>
        ))}
      </div>

      {/* Device Pairing Card */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>设备配对与受信任终端 (Device Pairing)</span>
        </div>
        <p style={{ margin: "0 0 10px", fontSize: 11, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
          通过单次配对令牌批准跨设备会话同步与远程执行请求。
        </p>

        <form onSubmit={handlePairSubmit} style={{ display: "flex", gap: 6, marginBottom: 10 }}>
          <input
            type="text"
            placeholder="输入 6 位配对令牌 (Pairing Token)..."
            value={pairCode}
            onChange={(e) => setPairCode(e.target.value)}
            style={{
              flex: 1,
              padding: "4px 8px",
              fontSize: 11,
              background: "rgba(0,0,0,0.3)",
              border: "1px solid var(--cursor-border, rgba(255,255,255,0.12))",
              borderRadius: 4,
              color: "#fff",
              outline: "none",
            }}
          />
          <SandButton size="sm" type="submit" variant="secondary">
            验证配对
          </SandButton>
        </form>

        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {devices.map((d) => (
            <div key={d.id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "6px 8px", background: "rgba(255,255,255,0.02)", borderRadius: 4, fontSize: 11 }}>
              <div>
                <span style={{ fontWeight: 500 }}>{d.deviceName}</span>
                <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)", marginLeft: 6 }}>平台: {d.platform}</span>
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <span className="sand-runtime-pill sand-runtime-pill--green">已信任</span>
                <button
                  onClick={() => handleRevokeDevice(d.id)}
                  style={{ background: "transparent", border: "none", color: "#f87171", cursor: "pointer", fontSize: 10 }}
                >
                  撤销
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Multimodal & Talk Voice Card */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>多模态理解与语音交互 (Multimodal & Talk)</span>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr", gap: 8, fontSize: 11 }}>
          <div>
            <label style={{ display: "block", color: "var(--cursor-text-secondary, #a1a1aa)", marginBottom: 2 }}>音频转录引擎:</label>
            <input
              type="text"
              value={voice.audioTranscriptionEngine}
              onChange={(e) => setVoice({ ...voice, audioTranscriptionEngine: e.target.value })}
              style={{ width: "100%", padding: "4px 8px", fontSize: 11, background: "rgba(0,0,0,0.3)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: 4, color: "#fff" }}
            />
          </div>
          <div>
            <label style={{ display: "block", color: "var(--cursor-text-secondary, #a1a1aa)", marginBottom: 2 }}>视觉/图文理解服务:</label>
            <input
              type="text"
              value={voice.imageDocInspectionEngine}
              onChange={(e) => setVoice({ ...voice, imageDocInspectionEngine: e.target.value })}
              style={{ width: "100%", padding: "4px 8px", fontSize: 11, background: "rgba(0,0,0,0.3)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: 4, color: "#fff" }}
            />
          </div>
          <div>
            <label style={{ display: "block", color: "var(--cursor-text-secondary, #a1a1aa)", marginBottom: 2 }}>Talk 语音合成 (TTS):</label>
            <input
              type="text"
              value={voice.talkVoiceEngine}
              onChange={(e) => setVoice({ ...voice, talkVoiceEngine: e.target.value })}
              style={{ width: "100%", padding: "4px 8px", fontSize: 11, background: "rgba(0,0,0,0.3)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: 4, color: "#fff" }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
