import React, { type FormEvent, type ReactNode } from 'react';
import type { BotMarkState } from '../../../frontend/apps/web/src/app/host/bot-mark';
import GrokAgentHeader from '../grok-shell/grok-agent-header';
import GrokAgentComposer from '../grok-shell/grok-agent-composer';
import AgentTranscript, { type AgentTranscriptProps } from './agent-transcript';

export interface AgentWorkspaceProps extends Omit<AgentTranscriptProps, 'title' | 'description' | 'botId'> {
  title: string;
  description: string;
  botId: string;
  botState: BotMarkState;
  status: string;
  pinned: boolean;
  searchActive: boolean;
  computerActive: boolean;
  infoActive: boolean;
  miniAppTitle?: string;
  onOpenMiniAppHeader?(): void;
  onToggleSearch(): void;
  onToggleComputer(): void;
  onTogglePin(): void;
  onToggleInfo(): void;

  composerValue: string;
  composerReady: boolean;
  composerBusy: boolean;
  enterToSend: boolean;
  onComposerChange(value: string): void;
  onComposerSubmit(event: FormEvent<HTMLFormElement>): void;
  onAttach(): void;
  onStop(): void;
  notice?: ReactNode;
  beforeComposer?: ReactNode;
  composerAccessory?: ReactNode;
}

/**
 * Primary Agent workspace composition.
 *
 * The Messenger shell supplies compatibility data/adapters only. Header,
 * transcript and composer now share one Agent-owned surface so later Computer,
 * settings and coordinator overlays can move here without re-coupling to
 * Messenger navigation.
 */
export default function AgentWorkspace(props: AgentWorkspaceProps) {
  return <>
    <GrokAgentHeader
      title={props.title}
      description={props.description}
      botId={props.botId}
      botState={props.botState}
      status={props.status}
      pinned={props.pinned}
      searchActive={props.searchActive}
      computerActive={props.computerActive}
      infoActive={props.infoActive}
      miniAppTitle={props.miniAppTitle}
      onOpenMiniApp={props.onOpenMiniAppHeader}
      onToggleSearch={props.onToggleSearch}
      onToggleComputer={props.onToggleComputer}
      onTogglePin={props.onTogglePin}
      onToggleInfo={props.onToggleInfo}
    />
    {props.notice}
    <AgentTranscript
      title={props.title}
      description={props.description}
      botId={props.botId}
      entries={props.entries}
      activeOperationId={props.activeOperationId}
      hasEarlierMessages={props.hasEarlierMessages}
      messageAreaRef={props.messageAreaRef}
      showScrollToLatest={props.showScrollToLatest}
      onLoadEarlier={props.onLoadEarlier}
      onOpenMiniApp={props.onOpenMiniApp}
      onScroll={props.onScroll}
      onScrollToLatest={props.onScrollToLatest}
      onCopyMessage={props.onCopyMessage}
      onRegenerate={props.onRegenerate}
      onEdit={props.onEdit}
      onContextMenu={props.onContextMenu}
    />
    {props.beforeComposer}
    {props.composerAccessory}
    <GrokAgentComposer
      value={props.composerValue}
      agentName={props.title}
      ready={props.composerReady}
      busy={props.composerBusy}
      enterToSend={props.enterToSend}
      onChange={props.onComposerChange}
      onSubmit={props.onComposerSubmit}
      onAttach={props.onAttach}
      onStop={props.onStop}
    />
  </>;
}
