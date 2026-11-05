/**
 * Canonical Gateway & Context Inspector Bridge (formerly `OpenClawAskCopilotPanel`)
 *
 * Upstream comparative anchors studied before modification:
 * - OpenClaw: `src/gateway/client.ts` & `ui/src/components/gateway-status.ts`
 * - Telegram Desktop: `Telegram/SourceFiles/info/info_controller.cpp`
 * - Blueprint 04 (`04-unified-context-inspector-drawer.md` §1.1, §2.1 BUG-04-01):
 *   - Eradicates hardcoded `isGatewayOnline = false` default parameter and fake connector readiness copy.
 *   - Eradicates destructive `composerDraftStore.setDraft` overwrites from quick-action pills.
 *   - Delegates to the unified `ContextInspector` while keeping `OpenClawAskCopilotPanelProps`
 *     100% backward-compatible for existing callers (`ProductionRenderer.tsx`, `OpenClawChatStage.tsx`).
 */

import React from "react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import {
  ContextInspector,
  type ContextInspectorTabId,
  type InspectorAgentSubject,
} from "../../recovered/features/conversation/workspace/context-inspector";

export interface CopilotMessageItem {
  readonly id: string;
  readonly role: "user" | "assistant" | "system";
  readonly content: string;
  readonly timestamp?: number | string;
}

export interface OpenClawAskCopilotPanelProps {
  readonly onClose: () => void;
  readonly gatewayAddress?: string;
  readonly isGatewayOnline?: boolean;
  readonly gatewayStatusText?: string;
  readonly connectedChannelsCount?: number;
  readonly messages?: readonly CopilotMessageItem[];
  /**
   * @deprecated Quick-action pills must never overwrite `composerDraftStore`.
   * Retained in the type contract solely for caller backward compatibility.
   */
  readonly onExecuteAction?: (actionText: string) => void;
  readonly agentName?: string;
  readonly activeAgent?: InspectorAgentSubject | null;
  readonly accountScopeGeneration?: number;
  readonly client?: ProductionCoordinatorClient | null;
  readonly initialTab?: ContextInspectorTabId;
}

export function OpenClawAskCopilotPanel({
  onClose,
  gatewayAddress = "ws://127.0.0.1:18789",
  isGatewayOnline,
  gatewayStatusText,
  connectedChannelsCount,
  messages = [],
  agentName,
  activeAgent,
  accountScopeGeneration = 0,
  client = null,
  initialTab = "skills_channels",
}: OpenClawAskCopilotPanelProps): React.JSX.Element {
  const resolvedSubject: InspectorAgentSubject = activeAgent ?? {
    id: agentName ? `agent:${agentName}` : "openclaw-gateway-inspector",
    name: agentName || "OpenClaw Runtime",
    title: "网关与连接器诊断",
    isHuman: false,
    isGroup: false,
  };

  return (
    <ContextInspector
      isOpen={true}
      onClose={onClose}
      activeAgent={resolvedSubject}
      accountScopeGeneration={accountScopeGeneration}
      client={client}
      isGatewayOnline={isGatewayOnline}
      gatewayAddress={gatewayAddress}
      gatewayStatusText={gatewayStatusText}
      connectedChannelsCount={connectedChannelsCount}
      initialTab={initialTab}
      customTabRenderers={
        messages.length > 0
          ? {
              skills_channels: () => (
                <div
                  data-testid="openclaw-diagnostic-log"
                  style={{
                    padding: "10px 12px",
                    borderRadius: "8px",
                    background: "var(--fabushi-bg-elevated, rgba(128, 128, 128, 0.05))",
                    border: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.14))",
                    display: "flex",
                    flexDirection: "column",
                    gap: "6px",
                  }}
                >
                  <div style={{ fontWeight: 700, fontSize: "11.5px" }}>网关诊断日志</div>
                  {messages.map((msg) => (
                    <div
                      key={msg.id}
                      style={{
                        fontSize: "11.5px",
                        padding: "6px 8px",
                        borderRadius: "6px",
                        background: "rgba(128, 128, 128, 0.06)",
                      }}
                    >
                      {msg.content}
                    </div>
                  ))}
                </div>
              ),
            }
          : undefined
      }
    />
  );
}
