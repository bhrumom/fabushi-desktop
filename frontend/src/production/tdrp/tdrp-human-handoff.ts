import { useCallback } from "react";
import type { ConversationTranscriptEntry, TranscriptMessage } from "../../recovered/features/conversation/workspace/model";
import type { RendererAgent } from "../model";
import type { ProductionCoordinatorClient } from "../coordinator-client";

export interface UseTdrpHumanHandoffOptions {
  readonly client: ProductionCoordinatorClient | null;
  readonly activeAgent: RendererAgent | null | undefined;
  readonly activeIsHuman: boolean;
  readonly humanHandoffAgentId: string;
  readonly agentsRef: React.MutableRefObject<RendererAgent[]>;
  readonly entries: readonly ConversationTranscriptEntry[];
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
  readonly setBusy: React.Dispatch<React.SetStateAction<boolean>>;
  readonly makeClientNonce: () => string;
}

export function useTdrpHumanHandoff({
  client,
  activeAgent,
  activeIsHuman,
  humanHandoffAgentId,
  agentsRef,
  entries,
  setNotice,
  setBusy,
  makeClientNonce,
}: UseTdrpHumanHandoffOptions) {
  const handoffHumanConversationToAgent = useCallback(async () => {
    if (client == null || activeAgent == null || !activeIsHuman || humanHandoffAgentId.length === 0) return;
    const targetAgent = agentsRef.current.find((agent) => agent.id === humanHandoffAgentId);
    if (targetAgent == null) {
      setNotice("Choose an available Agent for the handoff.");
      return;
    }
    const transcriptContext = entries
      .filter((entry): entry is TranscriptMessage => entry.kind === "message" && entry.text.trim().length > 0)
      .slice(-20)
      .map((entry) => `${entry.author}: ${entry.text.trim()}`)
      .join("\n");
    if (transcriptContext.length === 0) {
      setNotice("Send at least one Human message before handing the conversation to an Agent.");
      return;
    }
    const clientNonce = makeClientNonce();
    const now = Date.now();
    setBusy(true);
    try {
      await client.call("sendPrompt", {
        agentId: targetAgent.id,
        prompt: `Continue from this Human conversation and help with the user's explicit handoff request. Preserve the Human/Agent distinction and use tools or artifacts when useful.\n\nHuman conversation: ${activeAgent.name}\n\n${transcriptContext}`,
        directAddressedAcceptance: true,
        requestSource: "human-handoff",
        humanHandoffConversationId: activeAgent.id,
        attachmentPaths: [],
        attachmentNames: [],
        clientNonce,
        enterEpochMs: now,
        composedAtMs: now
      });
      setNotice(null);
    } catch (error) {
      setNotice(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }, [activeAgent, activeIsHuman, agentsRef, client, entries, humanHandoffAgentId, makeClientNonce, setBusy, setNotice]);

  return { handoffHumanConversationToAgent };
}
