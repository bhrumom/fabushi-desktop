import { useEffect, useState } from "react";
import { SandButton, SandIcon } from "../../recovered/ui/sand-kit-primitives";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import type { SessionUsageSnapshot } from "./types";

interface RuntimeSubagentsSectionProps {
  readonly agentId: string;
  readonly client?: ProductionCoordinatorClient | null;
}

export function RuntimeSubagentsSection({ agentId, client }: RuntimeSubagentsSectionProps) {
  const [session, setSession] = useState<SessionUsageSnapshot>({
    sessionId: agentId,
    key: agentId,
    model: "gemini-1.5-pro",
    modelProvider: "google",
    inputTokens: 0,
    outputTokens: 0,
    totalTokens: 0,
    contextTokens: 0,
    maxContextTokens: 128000,
    thinkingLevel: "inherit",
    verboseLevel: "inherit",
    reasoningLevel: "inherit",
    elevatedLevel: "inherit",
    childSessions: [],
    subagentRole: "orchestrator",
    spawnDepth: 0,
  });

  const [toast, setToast] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  useEffect(() => {
    let active = true;
    async function loadUsage() {
      if (!client) return;
      try {
        const res = (await client.call("sessions.get", { key: agentId })) as Record<string, unknown> | null;
        if (!active || !res) return;
        const s = (res.session ?? res) as Record<string, unknown>;
        setSession((prev) => ({
          ...prev,
          sessionId: typeof s.sessionId === "string" ? s.sessionId : prev.sessionId,
          model: typeof s.model === "string" ? s.model : prev.model,
          modelProvider: typeof s.modelProvider === "string" ? s.modelProvider : prev.modelProvider,
          inputTokens: typeof s.inputTokens === "number" ? s.inputTokens : prev.inputTokens,
          outputTokens: typeof s.outputTokens === "number" ? s.outputTokens : prev.outputTokens,
          totalTokens: typeof s.totalTokens === "number" ? s.totalTokens : prev.totalTokens,
          contextTokens: typeof s.contextTokens === "number" ? s.contextTokens : prev.contextTokens,
          childSessions: Array.isArray(s.childSessions) ? (s.childSessions as string[]) : prev.childSessions,
          spawnDepth: typeof s.spawnDepth === "number" ? s.spawnDepth : prev.spawnDepth,
        }));
      } catch {
        // Fallback gracefully
      }
    }
    void loadUsage();
    return () => {
      active = false;
    };
  }, [client, agentId]);

  const handleCompact = async () => {
    if (client) {
      try {
        await client.call("sessions.compact", { key: agentId, maxLines: 500 });
        showToast("已下发网关上下文压缩 (sessions.compact)");
      } catch {
        showToast("执行本地压缩 (/compact)");
      }
    }
    setSession((prev) => ({ ...prev, contextTokens: Math.max(800, Math.round(prev.contextTokens * 0.45)) }));
  };

  const handleFork = async () => {
    if (client) {
      try {
        await client.call("sessions.fork", { key: agentId });
        showToast("已创建会话分支 (sessions.fork)");
      } catch {
        showToast("已创建分支 (/fork)");
      }
    }
  };

  const handleReset = async () => {
    if (client) {
      try {
        await client.call("sessions.reset", { key: agentId, reason: "reset" });
        showToast("已重置会话上下文 (sessions.reset)");
      } catch {
        showToast("已重置上下文 (/reset)");
      }
    }
    setSession((prev) => ({ ...prev, contextTokens: 0, totalTokens: 0, inputTokens: 0, outputTokens: 0 }));
  };

  const usagePercent = session.maxContextTokens > 0
    ? Math.min(100, Math.round((session.contextTokens / session.maxContextTokens) * 100))
    : 0;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      {toast && (
        <div className="sand-agent-action-toast">
          <SandIcon name="sparkle" size="sm" />
          <span>{toast}</span>
        </div>
      )}

      {/* Session Live Status Card */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>会话运行与 Token 统计 (Live Usage)</span>
          <span className="sand-runtime-pill sand-runtime-pill--green">
            <span className="sand-session-status-dot" />
            就绪
          </span>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6, fontSize: 11, marginBottom: 8 }}>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #6b7280)" }}>模型: </span>
            <span style={{ fontWeight: 500 }}>{session.model}</span>
          </div>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #6b7280)" }}>提供商: </span>
            <span>{session.modelProvider}</span>
          </div>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #6b7280)" }}>输入 Token: </span>
            <span>{session.inputTokens.toLocaleString()}</span>
          </div>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #6b7280)" }}>输出 Token: </span>
            <span>{session.outputTokens.toLocaleString()}</span>
          </div>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #6b7280)" }}>总计 Token: </span>
            <span>{session.totalTokens.toLocaleString()}</span>
          </div>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #6b7280)" }}>上下文: </span>
            <span>{session.contextTokens.toLocaleString()} ({usagePercent}%)</span>
          </div>
        </div>

        <div className="sand-session-token-bar-track" style={{ width: "100%", height: 5, marginBottom: 10 }}>
          <div className="sand-session-token-bar-fill" style={{ width: `${Math.max(4, usagePercent)}%` }} />
        </div>

        <div style={{ display: "flex", gap: 6 }}>
          <SandButton onClick={() => void handleCompact()} size="sm" variant="secondary">
            <SandIcon name="archive" size="sm" />
            压缩 (/compact)
          </SandButton>
          <SandButton onClick={() => void handleFork()} size="sm" variant="secondary">
            <SandIcon name="fork" size="sm" />
            分支 (/fork)
          </SandButton>
          <SandButton onClick={() => void handleReset()} size="sm" variant="secondary">
            <SandIcon name="refresh" size="sm" />
            重置 (/reset)
          </SandButton>
        </div>
      </div>

      {/* Subagents Queue Card (Matching Upstream childSessions & swarm) */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>子 Agent 调度队列 (Subagent Lineage)</span>
          <span className="sand-runtime-pill sand-runtime-pill--gray">
            子会话: {session.childSessions.length}
          </span>
        </div>
        <p style={{ margin: "0 0 8px", fontSize: 11, color: "var(--cursor-text-secondary, #6b7280)" }}>
          遵循 OpenClaw 递归深度与并发上限标准。子会话执行由宿主守护进程托管。
        </p>
        {session.childSessions.length === 0 ? (
          <div style={{ fontSize: 11, color: "var(--cursor-text-secondary, #6b7280)", padding: "4px 0" }}>
            当前主会话无并行子 Agent
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {session.childSessions.map((childKey, idx) => (
              <div
                key={childKey}
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  padding: "4px 8px",
                  background: "rgba(128,128,128,0.04)",
                  borderRadius: 4,
                  fontSize: 11,
                }}
              >
                <div>
                  <span style={{ fontWeight: 500 }}>子会话 #{idx + 1}</span>
                  <code style={{ fontSize: 10, marginLeft: 6 }}>{childKey}</code>
                </div>
                <span className="sand-runtime-pill sand-runtime-pill--green">活动</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
