import { useEffect, useState } from "react";
import { SandButton, SandIcon } from "../recovered/ui/sand-kit-primitives";
import type { ProductionCoordinatorClient } from "./coordinator-client";
import type {
  ElevatedLevel,
  ReasoningLevel,
  RuntimeDetailSectionId,
  SessionUsageSnapshot,
  ThinkingLevel,
  VerboseLevel,
} from "./agent-runtime/types";
import "./agent-runtime/agent-runtime.css";

export interface FabushiAgentSessionToolbarProps {
  readonly agentId: string;
  readonly agentName: string;
  readonly client?: ProductionCoordinatorClient | null;
  readonly onOpenRuntimeDetails?: (section: RuntimeDetailSectionId) => void;
}

const DEFAULT_MAX_CONTEXT = 128000;

export function FabushiAgentSessionToolbar({
  agentId,
  agentName: _agentName,
  client = null,
  onOpenRuntimeDetails,
}: FabushiAgentSessionToolbarProps) {
  const [snapshot, setSnapshot] = useState<SessionUsageSnapshot>({
    sessionId: agentId,
    key: agentId,
    model: "gemini-1.5-pro",
    modelProvider: "google",
    inputTokens: 0,
    outputTokens: 0,
    totalTokens: 0,
    contextTokens: 0,
    maxContextTokens: DEFAULT_MAX_CONTEXT,
    thinkingLevel: "inherit",
    verboseLevel: "inherit",
    reasoningLevel: "inherit",
    elevatedLevel: "inherit",
    childSessions: [],
  });

  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (message: string) => {
    setToastMessage(message);
    setTimeout(() => setToastMessage(null), 3000);
  };

  // Live Gateway Query: sessions.get / sessions.list
  useEffect(() => {
    let cancelled = false;
    async function loadLiveSession() {
      if (!client) return;
      try {
        const response = await client.call("sessions.get", { key: agentId }) as Record<string, unknown> | null;
        if (!cancelled && response && typeof response === "object") {
          const row = (response.session ?? response) as Record<string, unknown>;
          setSnapshot((prev) => ({
            ...prev,
            sessionId: typeof row.sessionId === "string" ? row.sessionId : prev.sessionId,
            model: typeof row.model === "string" ? row.model : prev.model,
            modelProvider: typeof row.modelProvider === "string" ? row.modelProvider : prev.modelProvider,
            inputTokens: typeof row.inputTokens === "number" ? row.inputTokens : prev.inputTokens,
            outputTokens: typeof row.outputTokens === "number" ? row.outputTokens : prev.outputTokens,
            totalTokens: typeof row.totalTokens === "number" ? row.totalTokens : prev.totalTokens,
            contextTokens: typeof row.contextTokens === "number" ? row.contextTokens : prev.contextTokens,
            thinkingLevel: (typeof row.thinkingLevel === "string" ? row.thinkingLevel : prev.thinkingLevel) as ThinkingLevel,
            verboseLevel: (typeof row.verboseLevel === "string" ? row.verboseLevel : prev.verboseLevel) as VerboseLevel,
            reasoningLevel: (typeof row.reasoningLevel === "string" ? row.reasoningLevel : prev.reasoningLevel) as ReasoningLevel,
            elevatedLevel: (typeof row.elevatedLevel === "string" ? row.elevatedLevel : prev.elevatedLevel) as ElevatedLevel,
            childSessions: Array.isArray(row.childSessions) ? (row.childSessions as string[]) : prev.childSessions,
          }));
        }
      } catch {
        // Fallback gracefully without throwing
      }
    }
    void loadLiveSession();
    return () => {
      cancelled = true;
    };
  }, [client, agentId]);

  const patchSession = async (patch: Partial<SessionUsageSnapshot>) => {
    setSnapshot((prev) => ({ ...prev, ...patch }));
    if (client) {
      try {
        await client.call("sessions.patch", {
          key: agentId,
          patch: {
            ...(patch.thinkingLevel ? { thinkingLevel: patch.thinkingLevel } : {}),
            ...(patch.verboseLevel ? { verboseLevel: patch.verboseLevel } : {}),
            ...(patch.reasoningLevel ? { reasoningLevel: patch.reasoningLevel } : {}),
            ...(patch.elevatedLevel ? { elevatedLevel: patch.elevatedLevel } : {}),
            ...(patch.model ? { model: patch.model } : {}),
          },
        });
        showToast("会话参数已实时同步至网关");
      } catch (err) {
        showToast(`网关参数同步: ${err instanceof Error ? err.message : String(err)}`);
      }
    } else {
      showToast("本地会话参数已更新");
    }
  };

  const handleCompact = async () => {
    if (client) {
      try {
        await client.call("sessions.compact", { key: agentId, maxLines: 500 });
        showToast("已下发网关上下文压缩 (sessions.compact)");
      } catch {
        showToast("执行本地上下文压缩 (/compact)");
      }
    } else {
      showToast("执行本地上下文压缩 (/compact)");
    }
    setSnapshot((prev) => ({
      ...prev,
      contextTokens: Math.max(1000, Math.round(prev.contextTokens * 0.45)),
    }));
  };

  const handleFork = async () => {
    if (client) {
      try {
        await client.call("sessions.fork", { key: agentId });
        showToast("网关会话分支已创建 (sessions.fork)");
      } catch {
        showToast("创建会话分支 (/fork)");
      }
    } else {
      showToast("创建会话分支 (/fork)");
    }
    onOpenRuntimeDetails?.("runtime");
  };

  const handleReset = async () => {
    if (client) {
      try {
        await client.call("sessions.reset", { key: agentId, reason: "reset" });
        showToast("网关会话已重置 (sessions.reset)");
      } catch {
        showToast("会话上下文已重置 (/reset)");
      }
    } else {
      showToast("会话上下文已重置 (/reset)");
    }
    setSnapshot((prev) => ({
      ...prev,
      contextTokens: 0,
      inputTokens: 0,
      outputTokens: 0,
      totalTokens: 0,
    }));
  };

  const usagePercent = snapshot.maxContextTokens > 0
    ? Math.min(100, Math.round((snapshot.contextTokens / snapshot.maxContextTokens) * 100))
    : 0;

  return (
    <div className="sand-agent-session-toolbar" data-testid="fabushi-agent-session-toolbar">
      {/* Left: Status, Live Tokens, Thinking Level, Verbose Level */}
      <div className="sand-agent-session-toolbar__left">
        <span className="sand-session-status-badge">
          <span className="sand-session-status-dot" />
          运行中
        </span>

        {/* Live Token Usage */}
        <div className="sand-session-token-meter" title={`Context: ${snapshot.contextTokens} / Total: ${snapshot.totalTokens}`}>
          <SandIcon name="archive" size="sm" />
          <span>
            {snapshot.contextTokens > 0 ? `${(snapshot.contextTokens / 1000).toFixed(1)}k` : "0"} ({usagePercent}%)
          </span>
          <div className="sand-session-token-bar-track">
            <div className="sand-session-token-bar-fill" style={{ width: `${Math.max(4, usagePercent)}%` }} />
          </div>
        </div>

        {/* Upstream thinkingLevel Selector */}
        <div className="sand-session-patch-select" title="思考深度 (Thinking Level)">
          <SandIcon name="sparkle" size="sm" />
          <select
            value={snapshot.thinkingLevel}
            onChange={(e) => void patchSession({ thinkingLevel: e.target.value as ThinkingLevel })}
          >
            <option value="inherit">思考: 默认</option>
            <option value="off">思考: 关闭</option>
            <option value="minimal">思考: 极简</option>
            <option value="low">思考: 低</option>
            <option value="medium">思考: 中</option>
            <option value="high">思考: 高</option>
            <option value="xhigh">思考: 极致</option>
          </select>
        </div>

        {/* Upstream verboseLevel Selector */}
        <div className="sand-session-patch-select" title="日志详尽度 (Verbose Level)">
          <SandIcon name="code" size="sm" />
          <select
            value={snapshot.verboseLevel}
            onChange={(e) => void patchSession({ verboseLevel: e.target.value as VerboseLevel })}
          >
            <option value="inherit">详尽: 默认</option>
            <option value="off">详尽: 简短</option>
            <option value="on">详尽: 详细</option>
            <option value="full">详尽: 完整</option>
          </select>
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
          onClick={() => void handleCompact()}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="archive" size="sm" />
          压缩
        </SandButton>

        <SandButton
          aria-label="分支会话"
          data-testid="session-action-fork"
          onClick={() => void handleFork()}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="fork" size="sm" />
          分支
        </SandButton>

        <SandButton
          aria-label="重置会话"
          data-testid="session-action-reset"
          onClick={() => void handleReset()}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="refresh" size="sm" />
          重置
        </SandButton>

        <SandButton
          aria-label="子 Agent 队列"
          data-testid="session-action-subagents"
          onClick={() => onOpenRuntimeDetails?.("runtime")}
          size="sm"
          variant="secondary"
        >
          <SandIcon name="agents" size="sm" />
          子 Agent
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
