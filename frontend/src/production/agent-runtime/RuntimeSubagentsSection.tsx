import { useState } from "react";
import { SandButton, SandIcon } from "../../recovered/ui/sand-kit-primitives";
import type { NodeHostLease, SessionRuntimeStats } from "./types";

interface RuntimeSubagentsSectionProps {
  readonly agentId: string;
}

const INITIAL_STATS: SessionRuntimeStats = {
  sessionId: "session-can-prod-01",
  modelName: "gemini-1.5-pro",
  provider: "Google DeepMind",
  usedTokens: 42180,
  maxTokens: 128000,
  subagentQueueCount: 2,
  branchesCount: 1,
  memorySnippetsCount: 14,
};

const INITIAL_LEASES: NodeHostLease[] = [
  { id: "lease-7a8f", clientId: "agent-sub-01", capability: "system.run", grantedAt: "2m ago", ttlRemainingSeconds: 240 },
  { id: "lease-9b12", clientId: "agent-sub-02", capability: "browser.proxy", grantedAt: "30s ago", ttlRemainingSeconds: 570 },
];

export function RuntimeSubagentsSection({ agentId: _agentId }: RuntimeSubagentsSectionProps) {
  const [stats, setStats] = useState<SessionRuntimeStats>(INITIAL_STATS);
  const [leases, setLeases] = useState<NodeHostLease[]>(INITIAL_LEASES);
  const [feedback, setFeedback] = useState<string | null>(null);

  const showFeedback = (msg: string) => {
    setFeedback(msg);
    setTimeout(() => setFeedback(null), 3000);
  };

  const handleCompact = () => {
    setStats((prev) => ({
      ...prev,
      usedTokens: Math.max(12000, Math.round(prev.usedTokens * 0.45)),
    }));
    showFeedback("已执行上下文压缩 (/compact)，清理历史冗余 Token。");
  };

  const handleFork = () => {
    setStats((prev) => ({
      ...prev,
      branchesCount: prev.branchesCount + 1,
    }));
    showFeedback("已创建会话分支 (/fork)，可在分支中独立推演。");
  };

  const handleReset = () => {
    setStats((prev) => ({
      ...prev,
      usedTokens: 2500,
    }));
    showFeedback("会话上下文已重置 (/reset)。");
  };

  const handleRevokeLease = (id: string) => {
    setLeases((prev) => prev.filter((l) => l.id !== id));
    showFeedback(`已撤销租约 ${id}`);
  };

  const usagePercent = Math.min(100, Math.round((stats.usedTokens / stats.maxTokens) * 100));

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      {feedback && (
        <div className="sand-agent-action-toast">
          <SandIcon name="sparkle" size="sm" />
          <span>{feedback}</span>
        </div>
      )}

      {/* Session & Model Info Card */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>会话状态与模型路由</span>
          <span className="sand-runtime-pill sand-runtime-pill--green">
            <span className="sand-session-status-dot" />
            运行就绪
          </span>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8, fontSize: 11, marginBottom: 10 }}>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)" }}>模型: </span>
            <span style={{ fontWeight: 500 }}>{stats.modelName}</span>
          </div>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)" }}>提供商: </span>
            <span>{stats.provider}</span>
          </div>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)" }}>会话 ID: </span>
            <code style={{ fontSize: 10 }}>{stats.sessionId}</code>
          </div>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)" }}>分支数: </span>
            <span>{stats.branchesCount}</span>
          </div>
        </div>

        {/* Token Budget Gauge */}
        <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11 }}>
            <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)" }}>Token 预算使用</span>
            <span>
              {stats.usedTokens.toLocaleString()} / {stats.maxTokens.toLocaleString()} ({usagePercent}%)
            </span>
          </div>
          <div className="sand-session-token-bar-track" style={{ width: "100%", height: 6 }}>
            <div className="sand-session-token-bar-fill" style={{ width: `${usagePercent}%` }} />
          </div>
        </div>

        <div style={{ display: "flex", gap: 6, marginTop: 12 }}>
          <SandButton onClick={handleCompact} size="sm" variant="secondary">
            <SandIcon name="archive" size="sm" />
            压缩 (/compact)
          </SandButton>
          <SandButton onClick={handleFork} size="sm" variant="secondary">
            <SandIcon name="fork" size="sm" />
            分支 (/fork)
          </SandButton>
          <SandButton onClick={handleReset} size="sm" variant="secondary">
            <SandIcon name="refresh" size="sm" />
            重置 (/reset)
          </SandButton>
        </div>
      </div>

      {/* Subagents Queue Card */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>子 Agent 调度队列 (Subagents)</span>
          <span className="sand-runtime-pill sand-runtime-pill--gray">
            队列中: {stats.subagentQueueCount}
          </span>
        </div>
        <p style={{ margin: "0 0 8px", fontSize: 11, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
          任务编排由 Rust ProductionSessionWorkers 状态机托管，严格受深度与并发上限约束。
        </p>
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "6px 8px", background: "rgba(255,255,255,0.02)", borderRadius: 4, fontSize: 11 }}>
            <div>
              <span style={{ fontWeight: 500 }}>subagent-code-eval</span>
              <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)", marginLeft: 6 }}>递归深度: 1</span>
            </div>
            <span className="sand-runtime-pill sand-runtime-pill--green">执行中</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "6px 8px", background: "rgba(255,255,255,0.02)", borderRadius: 4, fontSize: 11 }}>
            <div>
              <span style={{ fontWeight: 500 }}>subagent-data-miner</span>
              <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)", marginLeft: 6 }}>等待前置结果</span>
            </div>
            <span className="sand-runtime-pill sand-runtime-pill--gray">排队中</span>
          </div>
        </div>
      </div>

      {/* Execution Leases Card */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>ACP 执行租约 (Execution Leases)</span>
          <span style={{ fontSize: 10, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
            活动: {leases.length}
          </span>
        </div>
        {leases.length === 0 ? (
          <div style={{ fontSize: 11, color: "var(--cursor-text-secondary, #a1a1aa)", padding: "4px 0" }}>无活动执行租约</div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            {leases.map((lease) => (
              <div key={lease.id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "6px 8px", background: "rgba(255,255,255,0.02)", borderRadius: 4, fontSize: 11 }}>
                <div>
                  <code style={{ fontSize: 10 }}>{lease.id}</code>
                  <span style={{ marginLeft: 6, fontWeight: 500 }}>{lease.capability}</span>
                  <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)", marginLeft: 6 }}>剩余 {lease.ttlRemainingSeconds}s</span>
                </div>
                <button
                  onClick={() => handleRevokeLease(lease.id)}
                  style={{ background: "transparent", border: "none", color: "#f87171", cursor: "pointer", fontSize: 10 }}
                >
                  撤销
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
