import React, { useState } from "react";
import { AgentAvatar } from "../../recovered/features/conversation/workspace/agent-avatar";

export interface OpenClawAskCopilotPanelProps {
  onClose: () => void;
  gatewayAddress?: string;
  isGatewayOnline?: boolean;
  onExecuteAction?: (actionText: string) => void;
}

interface CopilotMessageItem {
  id: string;
  role: "user" | "assistant";
  content: string;
}

export function OpenClawAskCopilotPanel({
  onClose,
  gatewayAddress = "ws://127.0.0.1:18789",
  isGatewayOnline = true,
  onExecuteAction,
}: OpenClawAskCopilotPanelProps) {
  const [activeTab, setActiveTab] = useState<"home" | "ask">("ask");
  const [messages, setMessages] = useState<CopilotMessageItem[]>([
    {
      id: "init-1",
      role: "user",
      content: "talk to agent",
    },
    {
      id: "init-2",
      role: "assistant",
      content: "Opening a chat with your agent. You can return through Settings -> Ask OpenClaw.",
    },
  ]);
  const [inputVal, setInputVal] = useState("");

  const handleSend = () => {
    const text = inputVal.trim();
    if (!text) return;
    const userMsg: CopilotMessageItem = { id: `user-${Date.now()}`, role: "user", content: text };
    setMessages((prev) => [...prev, userMsg]);
    setInputVal("");

    setTimeout(() => {
      let reply = `已收到指令: "${text}"。正在与网关 (${gatewayAddress}) 通信...`;
      if (text.includes("whatsapp") || text.includes("telegram") || text.includes("slack") || text.includes("discord")) {
        reply = `正在启动频道向导: ${text}。请检查控制台或在设置中配置相应凭据。`;
      } else if (text === "channels") {
        reply = "当前已注册 18 个多通道网关连接器 (Telegram, Slack, Discord, WhatsApp 等)，已全量就绪。";
      } else if (text === "help") {
        reply = "OpenClaw 助手支持自然语言配置网关、调度会话、挂载节点以及管理模型凭据。";
      }
      setMessages((prev) => [...prev, { id: `asst-${Date.now()}`, role: "assistant", content: reply }]);
    }, 400);
  };

  const handlePillClick = (pill: string) => {
    if (onExecuteAction) {
      onExecuteAction(pill);
    }
    const userMsg: CopilotMessageItem = { id: `user-${Date.now()}`, role: "user", content: pill };
    setMessages((prev) => [...prev, userMsg]);
    setTimeout(() => {
      setMessages((prev) => [
        ...prev,
        {
          id: `asst-${Date.now()}`,
          role: "assistant",
          content: `执行操作: "${pill}"。网关正在调度对应子任务...`,
        },
      ]);
    }, 300);
  };

  return (
    <aside className="oc-copilot-drawer" aria-label="询问 OpenClaw">
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
            className={`oc-copilot-drawer__tab ${activeTab === "home" ? "oc-copilot-drawer__tab--active" : ""}`}
            onClick={() => setActiveTab("home")}
          >
            主页
          </button>
          <button
            type="button"
            className={`oc-copilot-drawer__tab ${activeTab === "ask" ? "oc-copilot-drawer__tab--active" : ""}`}
            onClick={() => setActiveTab("ask")}
          >
            询问 OpenClaw
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

      {/* Transcript Body */}
      <div className="oc-copilot-drawer__body">
        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`oc-copilot-msg ${msg.role === "user" ? "oc-copilot-msg--user" : "oc-copilot-msg--assistant"}`}
          >
            {msg.content}
          </div>
        ))}

        {/* Divider: 更早 */}
        <div className="oc-copilot-divider">更早</div>

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
              }}
            />
            <span>推理已就绪。</span>
          </div>
          <div className="oc-copilot-status-card__sub">
            Gateway: 正在 {gatewayAddress} 运行。
          </div>

          {/* Quick Action Pills */}
          <div className="oc-copilot-pills">
            {["connect whatsapp", "connect telegram", "connect slack", "connect discord", "channels", "help"].map(
              (pill) => (
                <button
                  key={pill}
                  type="button"
                  className="oc-action-pill"
                  onClick={() => handlePillClick(pill)}
                >
                  {pill}
                </button>
              )
            )}
          </div>
        </div>
      </div>

      {/* Bottom Composer */}
      <div className="oc-copilot-drawer__footer">
        <input
          type="text"
          className="oc-copilot-drawer__input"
          value={inputVal}
          onChange={(e) => setInputVal(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              handleSend();
            }
          }}
          placeholder="给 OpenClaw 发消息..."
        />
        <button
          type="button"
          className="oc-copilot-drawer__send"
          onClick={handleSend}
          title="发送"
          aria-label="Send"
        >
          ➔
        </button>
      </div>
    </aside>
  );
}
