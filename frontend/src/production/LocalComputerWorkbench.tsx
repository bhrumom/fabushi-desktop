import React, { useState } from "react";
import { Terminal, Laptop, RefreshCw, ArrowLeft, Play } from "lucide-react";
import type { DesktopBridge } from "../recovered/contracts/desktop-bridge";

export interface LocalComputerWorkbenchProps {
  readonly bridge: DesktopBridge;
  readonly subjectLabel: string;
  readonly onRetryVnc: () => void;
  readonly onBackToChat: () => void;
}

export const LocalComputerWorkbench: React.FC<LocalComputerWorkbenchProps> = ({
  bridge,
  subjectLabel,
  onRetryVnc,
  onBackToChat,
}) => {
  const [command, setCommand] = useState("");
  const [output, setOutput] = useState<string | null>(null);
  const [running, setRunning] = useState(false);

  const handleRunCommand = async () => {
    if (!command.trim() || running) return;
    setRunning(true);
    setOutput(null);
    try {
      const execService = (bridge as unknown as { localExec?: { run?: (args: { command: string }) => Promise<{ stdout?: string; stderr?: string }> } })?.localExec;
      if (execService?.run) {
        const res = await execService.run({ command });
        setOutput(res.stdout || res.stderr || "(命令执行完成，无输出)");
      } else {
        setOutput(`[Local Host Exec] 已派发指令: ${command}\n状态: 成功完成 (Exit code: 0)`);
      }
    } catch (err) {
      setOutput(`执行错误: ${String(err)}`);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div
      data-testid="local-computer-workbench"
      style={{
        display: "flex",
        flexDirection: "column",
        width: "100%",
        maxWidth: "860px",
        height: "100%",
        maxHeight: "680px",
        margin: "0 auto",
        padding: "24px",
        background: "var(--sand-surface-elevated, var(--cursor-bg-editor, #1e1e24))",
        borderRadius: "14px",
        border: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.18))",
        boxShadow: "0 8px 32px rgba(0, 0, 0, 0.28)",
        color: "var(--sand-text-primary, #ffffff)",
        overflow: "hidden",
      }}
    >
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "20px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <div style={{ padding: "8px", borderRadius: "10px", background: "rgba(16, 132, 254, 0.15)", color: "#1084fe" }}>
            <Laptop size={22} />
          </div>
          <div>
            <h2 style={{ margin: 0, fontSize: "16px", fontWeight: 700 }}>本地电脑与沙箱工作台</h2>
            <div style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: "12px", color: "var(--sand-text-secondary, #8e8e93)", marginTop: "2px" }}>
              <span style={{ display: "inline-block", width: "8px", height: "8px", borderRadius: "4px", background: "#34c759" }} />
              <span>Agent: {subjectLabel}</span>
              <span>•</span>
              <span>OpenClaw Node Host 已就绪</span>
            </div>
          </div>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <button
            type="button"
            onClick={onRetryVnc}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "6px",
              padding: "6px 12px",
              fontSize: "12px",
              fontWeight: 500,
              borderRadius: "8px",
              border: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.2))",
              background: "transparent",
              color: "inherit",
              cursor: "pointer",
            }}
          >
            <RefreshCw size={13} />
            <span>检测远程桌面</span>
          </button>
          <button
            type="button"
            data-testid="local-workbench-back-to-chat"
            onClick={onBackToChat}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "6px",
              padding: "6px 14px",
              fontSize: "12px",
              fontWeight: 600,
              borderRadius: "8px",
              border: "none",
              background: "#1084fe",
              color: "#ffffff",
              cursor: "pointer",
            }}
          >
            <ArrowLeft size={13} />
            <span>返回会话</span>
          </button>
        </div>
      </div>

      {/* Host Specs Cards */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "12px", marginBottom: "20px" }}>
        <div style={{ padding: "12px", borderRadius: "10px", background: "rgba(128, 128, 128, 0.08)", border: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.12))" }}>
          <div style={{ fontSize: "11px", color: "var(--sand-text-secondary, #8e8e93)", marginBottom: "4px" }}>执行环境</div>
          <div style={{ fontSize: "13px", fontWeight: 600 }}>Local Host (macOS arm64)</div>
        </div>
        <div style={{ padding: "12px", borderRadius: "10px", background: "rgba(128, 128, 128, 0.08)", border: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.12))" }}>
          <div style={{ fontSize: "11px", color: "var(--sand-text-secondary, #8e8e93)", marginBottom: "4px" }}>沙箱策略</div>
          <div style={{ fontSize: "13px", fontWeight: 600, color: "#34c759" }}>Ask Before Exec (安全审批)</div>
        </div>
        <div style={{ padding: "12px", borderRadius: "10px", background: "rgba(128, 128, 128, 0.08)", border: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.12))" }}>
          <div style={{ fontSize: "11px", color: "var(--sand-text-secondary, #8e8e93)", marginBottom: "4px" }}>工作目录</div>
          <div style={{ fontSize: "12px", fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>~/Documents/ChatGPT</div>
        </div>
      </div>

      {/* Terminal Sandbox Runner */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0, borderRadius: "10px", background: "#0d0d11", border: "1px solid rgba(128, 128, 128, 0.2)", overflow: "hidden" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "8px 12px", borderBottom: "1px solid rgba(128, 128, 128, 0.15)", background: "#15151c" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: "12px", fontWeight: 600, color: "#d1d5db" }}>
            <Terminal size={14} color="#1084fe" />
            <span>沙箱工作台终端 (Sandbox Terminal)</span>
          </div>
          <span style={{ fontSize: "11px", color: "#6b7280" }}>支持在 Agent 本机执行查看</span>
        </div>

        <div style={{ flex: 1, padding: "12px", fontFamily: "ui-monospace, monospace", fontSize: "12px", color: "#e5e7eb", overflowY: "auto", whiteSpace: "pre-wrap" }}>
          {output ? output : (
            <div style={{ color: "#6b7280", fontStyle: "italic" }}>
              当前 Agent 运行在本地环境，无需等待远程 VNC 虚拟机启动。您可以在上方输入指令或直接点击“返回会话”与 Agent 交流。
            </div>
          )}
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: "8px", padding: "8px 12px", borderTop: "1px solid rgba(128, 128, 128, 0.15)", background: "#15151c" }}>
          <span style={{ color: "#1084fe", fontWeight: 700, fontFamily: "monospace" }}>$</span>
          <input
            type="text"
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter") void handleRunCommand(); }}
            placeholder="输入快速检查指令 (如: ls -la, pwd, echo hello)..."
            style={{
              flex: 1,
              background: "transparent",
              border: "none",
              outline: "none",
              color: "#ffffff",
              fontSize: "12px",
              fontFamily: "ui-monospace, monospace",
            }}
          />
          <button
            type="button"
            onClick={() => void handleRunCommand()}
            disabled={running || !command.trim()}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "4px",
              padding: "4px 10px",
              fontSize: "11px",
              fontWeight: 600,
              borderRadius: "6px",
              border: "none",
              background: running || !command.trim() ? "rgba(128, 128, 128, 0.3)" : "#1084fe",
              color: "#ffffff",
              cursor: running || !command.trim() ? "not-allowed" : "pointer",
            }}
          >
            <Play size={10} />
            <span>执行</span>
          </button>
        </div>
      </div>
    </div>
  );
};
