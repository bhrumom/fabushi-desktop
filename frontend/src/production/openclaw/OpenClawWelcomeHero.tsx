import React from "react";
import { AgentAvatar } from "../../recovered/features/conversation/workspace/agent-avatar";

export interface StarterPromptItem {
  id: string;
  title: string;
  description: string;
  prompt: string;
}

export const OPENCLAW_STARTER_PROMPTS: StarterPromptItem[] = [
  {
    id: "what-can-you-do",
    title: "你能做什么？",
    description: "探索此 Mac 上可用的工具、智能体与网关能力",
    prompt: "Show me what you can help with on this Mac right now.",
  },
  {
    id: "summarize-recent",
    title: "总结我最近的会话",
    description: "快速归纳自昨天以来的重要线程与未决事项",
    prompt: "Summarize what happened in my threads since yesterday.",
  },
  {
    id: "configure-channel",
    title: "帮我配置一个频道",
    description: "连接 WhatsApp、Telegram、Slack、Discord 等平台",
    prompt: "Help me connect a messaging channel.",
  },
  {
    id: "check-health",
    title: "检查系统健康状况",
    description: "诊断网关状态、推理连通性与节点资源负载",
    prompt: "Summarize the current OpenClaw status and tell me what needs attention.",
  },
];

export interface OpenClawWelcomeHeroProps {
  assistantName?: string;
  agent?: {
    id: string;
    name?: string;
    avatarDataUrl?: string | null;
    avatarShape?: string | null;
    avatarColor?: string | null;
    currentActivity?: unknown | null;
    isRunning?: boolean;
    isComposingMessage?: boolean;
    awaitingUserResponse?: unknown | null;
    isGroup?: boolean;
    isSharedRoom?: boolean;
    memberIds?: readonly string[];
  } | null;
  subtitle?: string;
  prompts?: StarterPromptItem[];
  onSelectPrompt: (promptText: string) => void;
}

export function OpenClawWelcomeHero({
  assistantName = "Assistant",
  agent,
  subtitle = "在下方输入消息 · / 使用命令",
  prompts = OPENCLAW_STARTER_PROMPTS,
  onSelectPrompt,
}: OpenClawWelcomeHeroProps) {
  const avatarKind = agent?.isSharedRoom ? "shared-room" : agent?.isGroup ? "group" : "agent";

  return (
    <div className="oc-welcome-hero" role="region" aria-label="Welcome stage">
      {/* Authentic Fabushi Agent Avatar */}
      <div className="oc-welcome-hero__avatar" style={{ display: "flex", justifyContent: "center" }}>
        <AgentAvatar
          agentId={agent?.id ?? "welcome-agent"}
          kind={avatarKind}
          memberIds={agent?.memberIds}
          dataUrl={agent?.avatarDataUrl}
          color={agent?.avatarColor}
          shape={agent?.avatarShape}
          currentActivity={agent?.currentActivity}
          isComposingMessage={agent?.isComposingMessage}
          isRunning={agent?.isRunning}
          awaitingUserResponse={agent?.awaitingUserResponse}
          size="xl"
        />
      </div>

      {/* Assistant Title */}
      <h1 className="oc-welcome-hero__title">{assistantName}</h1>

      {/* Subtitle */}
      <p className="oc-welcome-hero__subtitle">{subtitle}</p>

      {/* 2x2 Grid Starter Prompt Cards */}
      <div className="oc-welcome-hero__grid">
        {prompts.map((card) => (
          <button
            key={card.id}
            type="button"
            className="oc-prompt-card"
            onClick={() => onSelectPrompt(card.prompt)}
          >
            <span className="oc-prompt-card__title">{card.title}</span>
            <span className="oc-prompt-card__desc">{card.description}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
