import { useState } from "react";
import { SandButton, SandIcon } from "../../recovered/ui/sand-kit-primitives";
import type { SecurityPolicyState } from "./types";

interface SecurityPolicySectionProps {
  readonly agentId: string;
}

const INITIAL_SECURITY_STATE: SecurityPolicyState = {
  shellAllowlist: true,
  ssrfProtection: true,
  commandObfuscationDetector: true,
  skillSourceScanner: true,
  viralTaintStatus: "clean",
  taintTurnsCount: 0,
  quarantinedPayloads: 0,
};

export function SecurityPolicySection({ agentId: _agentId }: SecurityPolicySectionProps) {
  const [securityState, setSecurityState] = useState<SecurityPolicyState>(() => {
    try {
      const saved = localStorage.getItem("sand_agent_security_policy");
      if (saved) return JSON.parse(saved);
    } catch {
      // fallback
    }
    return INITIAL_SECURITY_STATE;
  });

  const [toast, setToast] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  const toggleGate = (key: keyof Pick<SecurityPolicyState, "shellAllowlist" | "ssrfProtection" | "commandObfuscationDetector" | "skillSourceScanner">) => {
    setSecurityState((prev) => {
      const updated = { ...prev, [key]: !prev[key] };
      try {
        localStorage.setItem("sand_agent_security_policy", JSON.stringify(updated));
      } catch {
        // ignore
      }
      return updated;
    });
    showToast(`安全门禁设置已更新`);
  };

  const handleResetTaint = () => {
    setSecurityState((prev) => {
      const updated: SecurityPolicyState = {
        ...prev,
        viralTaintStatus: "clean",
        taintTurnsCount: 0,
        quarantinedPayloads: 0,
      };
      try {
        localStorage.setItem("sand_agent_security_policy", JSON.stringify(updated));
      } catch {
        // ignore
      }
      return updated;
    });
    showToast("已重置对话污染隔离标记 (Taint Cleared)");
  };

  const taintColor =
    securityState.viralTaintStatus === "clean"
      ? "#4ade80"
      : securityState.viralTaintStatus === "monitored"
      ? "#fbbf24"
      : "#f87171";

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      {toast && (
        <div className="sand-agent-action-toast">
          <SandIcon name="shield" size="sm" />
          <span>{toast}</span>
        </div>
      )}

      {/* Taint Monitor Card */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>对话污染与安全审计 (Turn Taint Status)</span>
          <span
            className="sand-runtime-pill"
            style={{ background: `${taintColor}20`, color: taintColor, border: `1px solid ${taintColor}40` }}
          >
            <span className="sand-session-status-dot" style={{ background: taintColor }} />
            {securityState.viralTaintStatus === "clean" ? "清洁状态 (Clean)" : "受监控 (Monitored)"}
          </span>
        </div>
        <p style={{ margin: "0 0 10px", fontSize: 11, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
          基于 ACP 病毒式提示词注入防护规范。外部渠道不可信输入将被附加污染标识，限制高危工具调用。
        </p>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8, fontSize: 11, marginBottom: 10 }}>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)" }}>污染轮次: </span>
            <span>{securityState.taintTurnsCount}</span>
          </div>
          <div>
            <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)" }}>已拦截载荷: </span>
            <span>{securityState.quarantinedPayloads}</span>
          </div>
        </div>
        <SandButton onClick={handleResetTaint} size="sm" variant="secondary">
          <SandIcon name="refresh" size="sm" />
          重置污染隔离
        </SandButton>
      </div>

      {/* Pre-Execution Security Gates */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>工具执行前置安全门禁 (Pre-Execution Gate)</span>
        </div>
        <p style={{ margin: "0 0 10px", fontSize: 11, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
          由 Rust Host 审计层拦截，严禁未授权命令越权或网络探测。
        </p>

        <div className="sand-runtime-switch-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 11.5 }}>Shell 命令白名单 (Shell Allowlist)</div>
            <div style={{ fontSize: 10.5, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
              仅允许预定义指令，拦截 rm -rf, curl | bash, ssh 遍历等高危行为
            </div>
          </div>
          <input
            type="checkbox"
            checked={securityState.shellAllowlist}
            onChange={() => toggleGate("shellAllowlist")}
            style={{ width: 18, height: 18, cursor: "pointer", accentColor: "#3b82f6" }}
          />
        </div>

        <div className="sand-runtime-switch-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 11.5 }}>SSRF 局域网防护 (SSRF Guard)</div>
            <div style={{ fontSize: 10.5, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
              阻止向 127.0.0.1、10.0.0.0/8、192.168.0.0/16 及元数据服务发起请求
            </div>
          </div>
          <input
            type="checkbox"
            checked={securityState.ssrfProtection}
            onChange={() => toggleGate("ssrfProtection")}
            style={{ width: 18, height: 18, cursor: "pointer", accentColor: "#3b82f6" }}
          />
        </div>

        <div className="sand-runtime-switch-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 11.5 }}>指令混淆检测 (Obfuscation Detector)</div>
            <div style={{ fontSize: 10.5, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
              识别 Base64、Hex 编码载荷、多语言 Polyglot 绕过尝试
            </div>
          </div>
          <input
            type="checkbox"
            checked={securityState.commandObfuscationDetector}
            onChange={() => toggleGate("commandObfuscationDetector")}
            style={{ width: 18, height: 18, cursor: "pointer", accentColor: "#3b82f6" }}
          />
        </div>

        <div className="sand-runtime-switch-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 11.5 }}>技能源码指纹校验 (Skill Source Scanner)</div>
            <div style={{ fontSize: 10.5, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
              验证安装技能的密码学 SHA-256 签名，杜绝动态篡改
            </div>
          </div>
          <input
            type="checkbox"
            checked={securityState.skillSourceScanner}
            onChange={() => toggleGate("skillSourceScanner")}
            style={{ width: 18, height: 18, cursor: "pointer", accentColor: "#3b82f6" }}
          />
        </div>
      </div>
    </div>
  );
}
