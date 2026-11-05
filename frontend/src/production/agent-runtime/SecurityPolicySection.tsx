import { useEffect, useState } from "react";
import { SandButton, SandIcon } from "../../recovered/ui/sand-kit-primitives";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import type { ExecAllowlistEntry, ExecApprovalsFile, ExecAsk, ExecSecurity } from "./types";

interface SecurityPolicySectionProps {
  readonly agentId: string;
  readonly client?: ProductionCoordinatorClient | null;
}

const DEFAULT_APPROVALS_FILE: ExecApprovalsFile = {
  version: 1,
  defaults: {
    security: "allowlist",
    ask: "on-miss",
    askFallback: "deny",
    autoAllowSkills: true,
  },
  allowlist: [
    { id: "al-1", pattern: "git status", description: "版本控制状态只读检查" },
    { id: "al-2", pattern: "cargo check", description: "代码库语法与类型编译" },
    { id: "al-3", pattern: "npm test", description: "本地自动化测试验证" },
  ],
};

export function SecurityPolicySection({ agentId: _agentId, client }: SecurityPolicySectionProps) {
  const [approvals, setApprovals] = useState<ExecApprovalsFile>(DEFAULT_APPROVALS_FILE);
  const [baseHash, setBaseHash] = useState<string | undefined>(undefined);
  const [taintStatus, setTaintStatus] = useState<"clean" | "monitored" | "tainted">("clean");
  const [toast, setToast] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  useEffect(() => {
    let active = true;
    async function loadExecApprovals() {
      if (!client) return;
      try {
        const res = (await client.call("exec.approvals.get", {})) as { file?: ExecApprovalsFile; hash?: string } | null;
        if (!active || !res || !res.file) return;
        setApprovals(res.file);
        setBaseHash(res.hash);
      } catch {
        // Fallback to defaults
      }
    }
    void loadExecApprovals();
    return () => {
      active = false;
    };
  }, [client]);

  const updateDefaults = async (patch: Partial<NonNullable<ExecApprovalsFile["defaults"]>>) => {
    const updated: ExecApprovalsFile = {
      ...approvals,
      defaults: {
        ...approvals.defaults,
        ...patch,
      },
    };
    setApprovals(updated);
    if (client) {
      try {
        await client.call("exec.approvals.set", { file: updated, baseHash });
        showToast("网关执行安全策略已更新 (exec.approvals.set)");
      } catch (err) {
        showToast(`策略同步: ${err instanceof Error ? err.message : String(err)}`);
      }
    } else {
      showToast("本地安全策略已更新");
    }
  };

  const removeAllowlistEntry = async (id: string) => {
    const nextList = (approvals.allowlist ?? []).filter((e) => e.id !== id);
    const updated: ExecApprovalsFile = { ...approvals, allowlist: nextList };
    setApprovals(updated);
    if (client) {
      try {
        await client.call("exec.approvals.set", { file: updated, baseHash });
        showToast("已从白名单移除规则");
      } catch {
        // ignore
      }
    }
  };

  const resetTaint = () => {
    setTaintStatus("clean");
    showToast("对话污染审计标记已重置 (Taint Cleared)");
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      {toast && (
        <div className="sand-agent-action-toast">
          <SandIcon name="shield" size="sm" />
          <span>{toast}</span>
        </div>
      )}

      {/* Upstream Exec Security Gate (exec.approvals.get / set) */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>命令执行前置安全策略 (Exec Approvals)</span>
          <span className="sand-runtime-pill sand-runtime-pill--green">
            模式: {approvals.defaults?.security ?? "allowlist"}
          </span>
        </div>
        <p style={{ margin: "0 0 8px", fontSize: 11, color: "var(--cursor-text-secondary, #6b7280)" }}>
          直接对齐 OpenClaw exec.approvals 协议。控制本地沙箱与命令提示门禁。
        </p>

        <div className="sand-runtime-switch-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 11.5 }}>安全等级 (security)</div>
            <div style={{ fontSize: 10.5, color: "var(--cursor-text-secondary, #6b7280)" }}>
              deny: 完全拦截 / allowlist: 白名单拦截 / full: 全部允许
            </div>
          </div>
          <select
            value={approvals.defaults?.security ?? "allowlist"}
            onChange={(e) => void updateDefaults({ security: e.target.value as ExecSecurity })}
            style={{ fontSize: 11, padding: "2px 4px", borderRadius: 4 }}
          >
            <option value="deny">deny (禁止执行)</option>
            <option value="allowlist">allowlist (白名单)</option>
            <option value="full">full (无限制)</option>
          </select>
        </div>

        <div className="sand-runtime-switch-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 11.5 }}>审批提示策略 (ask)</div>
            <div style={{ fontSize: 10.5, color: "var(--cursor-text-secondary, #6b7280)" }}>
              off: 不弹窗 / on-miss: 未匹配白名单时询问 / always: 每次询问
            </div>
          </div>
          <select
            value={approvals.defaults?.ask ?? "on-miss"}
            onChange={(e) => void updateDefaults({ ask: e.target.value as ExecAsk })}
            style={{ fontSize: 11, padding: "2px 4px", borderRadius: 4 }}
          >
            <option value="off">off (不提示)</option>
            <option value="on-miss">on-miss (未命中提示)</option>
            <option value="always">always (总是提示)</option>
          </select>
        </div>

        <div className="sand-runtime-switch-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 11.5 }}>技能脚本自动放行 (autoAllowSkills)</div>
            <div style={{ fontSize: 10.5, color: "var(--cursor-text-secondary, #6b7280)" }}>
              允许通过完整性校验的已安装技能命令免审批运行
            </div>
          </div>
          <input
            type="checkbox"
            checked={approvals.defaults?.autoAllowSkills ?? true}
            onChange={(e) => void updateDefaults({ autoAllowSkills: e.target.checked })}
            style={{ width: 16, height: 16, cursor: "pointer", accentColor: "#3b82f6" }}
          />
        </div>
      </div>

      {/* Allowlist Patterns List */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>命令白名单规则库 (Allowlist Patterns)</span>
          <span style={{ fontSize: 10, color: "var(--cursor-text-secondary, #6b7280)" }}>
            规则数: {(approvals.allowlist ?? []).length}
          </span>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
          {(approvals.allowlist ?? []).map((entry: ExecAllowlistEntry) => (
            <div
              key={entry.id}
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
                <code style={{ fontWeight: 600 }}>{entry.pattern}</code>
                {entry.description && (
                  <span style={{ color: "var(--cursor-text-secondary, #6b7280)", marginLeft: 6, fontSize: 10 }}>
                    ({entry.description})
                  </span>
                )}
              </div>
              <button
                onClick={() => void removeAllowlistEntry(entry.id)}
                style={{ background: "transparent", border: "none", color: "#f87171", cursor: "pointer", fontSize: 10 }}
              >
                移除
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* Turn Taint Audit Status */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>对话污染隔离 (Viral Taint Audit)</span>
          <span className="sand-runtime-pill sand-runtime-pill--green">
            状态: {taintStatus}
          </span>
        </div>
        <p style={{ margin: "0 0 8px", fontSize: 11, color: "var(--cursor-text-secondary, #6b7280)" }}>
          外部渠道不可信提示词隔离防护。一旦标记为受污染，高危操作将自动熔断。
        </p>
        <SandButton onClick={resetTaint} size="sm" variant="secondary">
          <SandIcon name="refresh" size="sm" />
          重置污染标记
        </SandButton>
      </div>
    </div>
  );
}
