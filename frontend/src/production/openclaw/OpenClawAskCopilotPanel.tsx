import React, { useState } from "react";
import { AgentAvatar } from "../../recovered/features/conversation/workspace/agent-avatar";

export interface CopilotMessageItem {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp?: number | string;
}

export interface OpenClawAskCopilotPanelProps {
  onClose: () => void;
  gatewayAddress?: string;
  isGatewayOnline?: boolean;
  gatewayStatusText?: string;
  connectedChannelsCount?: number;
  messages?: CopilotMessageItem[];
  onExecuteAction?: (actionText: string) => void;
  agentName?: string;
}

export function OpenClawAskCopilotPanel({
  onClose,
  gatewayAddress = "ws://127.0.0.1:18789",
  isGatewayOnline = false,
  gatewayStatusText,
  connectedChannelsCount,
  messages = [],
  onExecuteAction,
  agentName,
}: OpenClawAskCopilotPanelProps) {
  const [activeTab, setActiveTab] = useState<"home" | "ask">("ask");

  const handlePillClick = (pill: string) => {
    if (onExecuteAction) {
      onExecuteAction(pill);
    }
  };

  const statusLabel = gatewayStatusText
    ? gatewayStatusText
    : isGatewayOnline
    ? "网关已连接 (在线)"
    : "网关未连接 (未知)";

  return (
    <aside className="oc-copilot-drawer" aria-label="OpenClaw 网关检查器">
      {/* Top Tab Header */}
      <div className="oc-copilot-drawer__header">
        <div className="oc-copilot-drawer__tabs">
          <AgentAvatar
            agentId="copilot-assistant"
            color="cyan"
            shape="wedge"
            size="xs"
          />
          <button
            type="button"
            className={`oc-copilot-drawer__tab ${activeTab === "ask" ? "oc-copilot-drawer__tab--active" : ""}`}
            onClick={() => setActiveTab("ask")}
          >
            网关检查
          </button>
          <button
            type="button"
            className={`oc-copilot-drawer__tab ${activeTab === "home" ? "oc-copilot-drawer__tab--active" : ""}`}
            onClick={() => setActiveTab("home")}
          >
            连接参数
          </button>
        </div>

        <button
          type="button"
          className="oc-copilot-drawer__close"
          onClick={onClose}
          title="关闭面板"
          aria-label="Close"
        >
          ✕
        </button>
      </div>

      {/* Body */}
      {activeTab === "home" ? (
        <div className="oc-copilot-drawer__body">
          <div className="oc-copilot-status-card">
            <div className="oc-copilot-status-card__headline">
              <span
                style={{
                  display: "inline-block",
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  backgroundColor: isGatewayOnline ? "#27AE60" : "#E74C3C",
                  flexShrink: 0,
                }}
                aria-label={isGatewayOnline ? "在线" : "离线"}
              />
              <span>连接参数概览</span>
            </div>
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 8,
                fontSize: 12,
                marginTop: 6,
                color: "var(--oc-text-primary)",
              }}
            >
              <div>
                <span style={{ color: "var(--oc-text-muted, #888)" }}>端点地址: </span>
                <span style={{ fontFamily: "monospace" }}>{gatewayAddress}</span>
              </div>
              <div>
                <span style={{ color: "var(--oc-text-muted, #888)" }}>在线状态: </span>
                <span style={{ color: isGatewayOnline ? "#27AE60" : "#E74C3C", fontWeight: 500 }}>
                  {statusLabel}
                </span>
              </div>
              <div>
                <span style={{ color: "var(--oc-text-muted, #888)" }}>通信协议: </span>
                <span>WebSocket (OpenClaw RPC 2.0)</span>
              </div>
              <div>
                <span style={{ color: "var(--oc-text-muted, #888)" }}>当前智能体: </span>
                <span>{agentName || "当前活动会话"}</span>
              </div>
              <div>
                <span style={{ color: "var(--oc-text-muted, #888)" }}>外部渠道: </span>
                <span>
                  {connectedChannelsCount !== undefined
                    ? `已连接 ${connectedChannelsCount} 个渠道`
                    : isGatewayOnline
                    ? "待同步通道配置"
                    : "未连接 (需启动网关并配置渠道凭据)"}
                </span>
              </div>
            </div>
          </div>

          <div
            style={{
              fontSize: 12,
              color: "var(--oc-text-muted, #888)",
              lineHeight: 1.6,
              padding: "4px 2px",
            }}
          >
            OpenClaw 网关为外部渠道与设备提供接入支持。当前状态反映实际后端探测结果，严禁伪造在线状态。
          </div>
        </div>
      ) : (
        <div className="oc-copilot-drawer__body">
          {/* Status Card */}
          <div className="oc-copilot-status-card">
            <div className="oc-copilot-status-card__headline">
              <span
                style={{
                  display: "inline-block",
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  backgroundColor: isGatewayOnline ? "#27AE60" : "#E74C3C",
                  flexShrink: 0,
                }}
                aria-label={isGatewayOnline ? "在线" : "未连接"}
              />
              <span>{statusLabel}</span>
            </div>
            <div className="oc-copilot-status-card__sub">
              {gatewayAddress
                ? `Gateway: ${gatewayAddress} (${isGatewayOnline ? "已连接" : "未连接"})`
                : "Gateway 地址: 未配置"}
            </div>
            <div
              style={{
                fontSize: 11.5,
                color: "var(--oc-text-muted, #888)",
                marginTop: 2,
              }}
            >
              {connectedChannelsCount !== undefined
                ? `通道连接器: 已连接 ${connectedChannelsCount} 个`
                : isGatewayOnline
                ? "通道连接器: 待同步"
                : "通道连接器: 未就绪 (需在设置或本地启动网关)"}
            </div>

            {/* Quick Action Pills */}
            <div className="oc-copilot-pills">
              {[
                "connect whatsapp",
                "connect telegram",
                "connect slack",
                "connect discord",
                "channels",
                "help",
              ].map((pill) => (
                <button
                  key={pill}
                  type="button"
                  className="oc-action-pill"
                  onClick={() => handlePillClick(pill)}
                  title={`将 "${pill}" 调度至主输入框`}
                >
                  {pill}
                </button>
              ))}
            </div>
          </div>

          {/* Divider: 诊断与历史 */}
          <div className="oc-copilot-divider">诊断与消息记录</div>

          {/* Transcript / Messages */}
          {messages.length > 0 ? (
            messages.map((msg) => (
              <div
                key={msg.id}
                className={`oc-copilot-msg ${
                  msg.role === "user"
                    ? "oc-copilot-msg--user"
                    : msg.role === "system"
                    ? "oc-copilot-msg--system"
                    : "oc-copilot-msg--assistant"
                }`}
              >
                {msg.content}
              </div>
            ))
          ) : (
            <div
              style={{
                padding: "16px 8px",
                color: "var(--oc-text-muted, #888)",
                fontSize: 12.5,
                textAlign: "center",
                lineHeight: 1.5,
              }}
            >
              暂无网关诊断记录。点击上方快捷操作可调度对应指令至主会话。
            </div>
          )}
        </div>
      )}

      {/* Footer: Unified composer reminder (eliminates dual-composer collision) */}
      <div
        className="oc-copilot-drawer__footer"
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: "10px 14px",
          borderTop: "1px solid var(--oc-border, #e0e0e0)",
          background: "var(--oc-bg-surface, #f8f9fa)",
          fontSize: 12,
          color: "var(--oc-text-muted, #888)",
          textAlign: "center",
        }}
      >
        <span>统一输入：请使用底部主输入框与智能体交互</span>
      </div>
    </aside>
  );
}
