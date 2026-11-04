import { useState } from "react";
import { SandButton, SandIcon } from "../recovered/ui/sand-kit-primitives";
import type { RuntimeDetailSectionId } from "./agent-runtime/types";
import "./agent-runtime/agent-runtime.css";

export interface FabushiAgentSessionToolbarProps {
  readonly agentId: string;
  readonly agentName: string;
  readonly onOpenRuntimeDetails?: (section: RuntimeDetailSectionId) => void;
}

const AVAILABLE_MODELS = [
  { id: "gemini-1.5-pro", name: "Gemini 1.5 Pro", provider: "Google" },
  { id: "claude-3-5-sonnet", name: "Claude 3.5 Sonnet", provider: "Anthropic" },
  { id: "gpt-4o", name: "GPT-4o", provider: "OpenAI" },
  { id: "local-llama-3", name: "Llama 3 70B", provider: "Local Host" },
];

export function FabushiAgentSessionToolbar({
  agentId: _agentId,
  agentName: _agentName,
  onOpenRuntimeDetails,
}: FabushiAgentSessionToolbarProps) {
  const [selectedModel, setSelectedModel] = useState("gemini-1.5-pro");
  const [usedTokens, setUsedTokens] = useState(38420);
  const maxTokens = 128000;
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (message: string) => {
    setToastMessage(message);
    setTimeout(() => setToastMessage(null), 3000);
  };

  const handleCompact = () => {
    setUsedTokens((prev) => Math.max(9500, Math.round(prev * 0.42)));
    showToast("上下文已压缩 (/compact)，节省 58% 历史 Token");
  };

  const handleFork = () => {
    showToast("已创建会话分支 (/fork)，新分支独立推演");
    onOpenRuntimeDetails?.("runtime");
  };

  const handleReset = () => {
    setUsedTokens(2100);
    showToast("会话上下文已重置 (/reset)");
  };

  const handleSubagents = () => {
    onOpenRuntimeDetails?.("runtime");
    showToast("打开子 Agent 调度队列 (Subagents)");
  };

  const handleMemory = () => {
    showToast("检索 QMD 混合记忆库 (Memory)");
    onOpenRuntimeDetails?.("runtime");
  };

  const usagePercent = Math.min(100, Math.round((usedTokens / maxTokens) * 100));

  return (
    <div className="sand-agent-session-toolbar" data-testid="fabushi-agent-session-toolbar">
      {/* Left: Status, Model Routing, and Token Budget */}
      <div className="sand-agent-session-toolbar__left">
        <span className="sand-session-status-badge">
          <span className="sand-session-status-dot" />
          会话活跃
        </span>

        {/* Model Selector */}
        <div className="sand-session-model-select">
          <SandIcon name="sparkle" size="sm" />
          <select
            value={selectedModel}
            onChange={(e) => {
              setSelectedModel(e.target.value);
              showToast(`已切换推理模型: ${e.target.value}`);
            }}
            style={{
              background: "transparent",
              border: "none",
              color: "inherit",
              fontSize: "inherit",
              outline: "none",
              cursor: "pointer",
            }}
          >
            {AVAILABLE_MODELS.map((m) => (
              <option key={m.id} value={m.id} style={{ background: "#222", color: "#fff" }}>
                {m.name} ({m.provider})
              </option>
            ))}
          </select>
        </div>

        {/* Token Context Meter */}
        <div className="sand-session-token-meter" title={`上下文 Token: ${usedTokens} / ${maxTokens}`}>
          <SandIcon name="archive" size="sm" />
          <span>
            {Math.round(usedTokens / 1000)}k/{Math.round(maxTokens / 1000)}k ({usagePercent}%)
          </span>
          <div className="sand-session-token-bar-track">
            <div className="sand-session-token-bar-fill" style={{ width: `${usagePercent}%` }} />
          </div>
        </div>

        {toastMessage && (
          <div className="sand-agent-action-toast">
            <SandIcon name="sparkle" size="sm" />
            <span>{toastMessage}</span>
          </div>
        )}
      </div>

      {/* Right: Quick Action Buttons (strictly SandButton + SandIcon, NO emoji icons) */}
      <div className="sand-agent-session-toolbar__right">
        <SandButton
          aria-label="压缩上下文"
          data-testid="session-action-compact"
          onClick={handleCompact}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="archive" size="sm" />
          压缩
        </SandButton>

        <SandButton
          aria-label="分支会话"
          data-testid="session-action-fork"
          onClick={handleFork}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="fork" size="sm" />
          分支
        </SandButton>

        <SandButton
          aria-label="重置会话"
          data-testid="session-action-reset"
          onClick={handleReset}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="refresh" size="sm" />
          重置
        </SandButton>

        <SandButton
          aria-label="子 Agent 队列"
          data-testid="session-action-subagents"
          onClick={handleSubagents}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="agents" size="sm" />
          子 Agent
        </SandButton>

        <SandButton
          aria-label="混合记忆检索"
          data-testid="session-action-memory"
          onClick={handleMemory}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="brain" size="sm" />
          记忆
        </SandButton>

        <SandButton
          aria-label="运行时详情面板"
          data-testid="session-action-runtime-details"
          onClick={() => onOpenRuntimeDetails?.("runtime")}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="sliders" size="sm" />
          详情
        </SandButton>
      </div>
    </div>
  );
}
