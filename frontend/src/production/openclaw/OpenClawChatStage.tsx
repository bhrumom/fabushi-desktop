import React, { useState } from "react";
import { OpenClawTopHeader, type OpenClawTopHeaderProps } from "./OpenClawTopHeader";
import { OpenClawWelcomeHero } from "./OpenClawWelcomeHero";
import { OpenClawCleanComposer } from "./OpenClawCleanComposer";
import { OpenClawAskCopilotPanel } from "./OpenClawAskCopilotPanel";
import "./openclaw-theme.css";

export interface OpenClawChatStageProps {
  title?: string;
  agent?: OpenClawTopHeaderProps["agent"];
  hasEntries: boolean;
  children: React.ReactNode;
  draft: string;
  onChangeDraft: (value: string) => void;
  onSubmit: () => void;
  disabled?: boolean;
  placeholder?: string;
  onStageFiles?: () => void;
  onTranscribeAudio?: () => void;
  isTranscribing?: boolean;
  onSelectStarterPrompt: (prompt: string) => void;
  onShare?: () => void;
  onRename?: () => void;
  onClearHistory?: () => void;
  onExportTranscript?: () => void;
  onOpenDetails?: () => void;
  isDetailsOpen?: boolean;
}

export function OpenClawChatStage({
  title = "Main Session",
  agent,
  hasEntries,
  children,
  draft,
  onChangeDraft,
  onSubmit,
  disabled = false,
  placeholder,
  onStageFiles,
  onTranscribeAudio,
  isTranscribing = false,
  onSelectStarterPrompt,
  onShare,
  onRename,
  onClearHistory,
  onExportTranscript,
  onOpenDetails,
  isDetailsOpen = false,
}: OpenClawChatStageProps) {
  const [isCopilotOpen, setIsCopilotOpen] = useState(false);
  const [securityGate, setSecurityGate] = useState<"deny" | "allowlist" | "full">("allowlist");
  const [selectedModel, setSelectedModel] = useState("GPT-6 Astra");
  const [selectedThinking, setSelectedThinking] = useState("Medium");

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        width: "100%",
        height: "100%",
        minHeight: 0,
        minWidth: 0,
        overflow: "hidden",
      }}
    >
      {/* Center Main Stage */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          flex: 1,
          height: "100%",
          minHeight: 0,
          minWidth: 0,
          overflow: "hidden",
          background: "var(--oc-bg-canvas)",
        }}
      >
        {/* Top Header */}
        <OpenClawTopHeader
          title={title}
          agent={agent}
          isDetailsOpen={isDetailsOpen}
          onToggleDetails={onOpenDetails}
          isLayoutSplit={isCopilotOpen}
          onToggleLayout={() => setIsCopilotOpen((open) => !open)}
          onShare={onShare}
          onRename={onRename}
          onClearHistory={onClearHistory}
          onExportTranscript={onExportTranscript}
          onOpenSettings={onOpenDetails}
        />

        {/* Content Stage: Welcome Hero or Transcript */}
        <div
          style={{
            flex: 1,
            display: "flex",
            flexDirection: "column",
            minHeight: 0,
            overflowY: "auto",
            position: "relative",
          }}
        >
          {!hasEntries ? (
            <OpenClawWelcomeHero
              assistantName={title}
              agent={agent}
              onSelectPrompt={onSelectStarterPrompt}
            />
          ) : (
            children
          )}
        </div>

        {/* Bottom Clean Composer */}
        <div style={{ padding: "0 20px" }}>
          <OpenClawCleanComposer
            draft={draft}
            onChange={onChangeDraft}
            onSubmit={onSubmit}
            disabled={disabled}
            placeholder={placeholder}
            onStageFiles={onStageFiles}
            securityGate={securityGate}
            onChangeSecurityGate={setSecurityGate}
            model={selectedModel}
            onChangeModel={setSelectedModel}
            thinkingLevel={selectedThinking}
            onChangeThinkingLevel={setSelectedThinking}
            onTranscribeAudio={onTranscribeAudio}
            isTranscribing={isTranscribing}
          />
        </div>
      </div>

      {/* Right Copilot / Ask OpenClaw Drawer (Split Layout) */}
      {isCopilotOpen && (
        <OpenClawAskCopilotPanel
          onClose={() => setIsCopilotOpen(false)}
        />
      )}
    </div>
  );
}
