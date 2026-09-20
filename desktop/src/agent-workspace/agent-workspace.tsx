import React, { type FormEvent, type ReactNode } from 'react';
import type { BotMarkState } from '../../../frontend/apps/web/src/app/host/bot-mark';
import AgentHeader from './agent-header';
import AgentComposer from './agent-composer';
import AgentSearch from './agent-search';
import AgentTranscript, { type AgentTranscriptProps } from './agent-transcript';

export interface AgentWorkspaceProps extends Omit<AgentTranscriptProps, 'title' | 'description' | 'botId'> {
  title: string;
  description: string;
  botId: string;
  botState: BotMarkState;
  status: string;
  pinned: boolean;
  searchActive: boolean;
  searchQuery: string;
  computerActive: boolean;
  infoActive: boolean;
  miniAppTitle?: string;
  onOpenMiniAppHeader?(): void;
  onToggleSearch(): void;
  onSearchQuery(value: string): void;
  onSelectSearchResult(entryId: string): void;
  onToggleComputer(): void;
  onTogglePin(): void;
  onToggleInfo(): void;

  composerValue: string;
  composerRichText?: string;
  composerReady: boolean;
  composerBusy: boolean;
  composerUploading?: boolean;
  composerAttachments?: ReadonlyArray<{ id: string; name: string; sizeBytes?: number }>;
  composerReplyTarget?: { id: string; label: string; text: string };
  composerMentionCandidates?: ReadonlyArray<{ id: string; name: string; description?: string; kind?: 'agent' | 'mcp' }>;
  composerWorkflowCandidates?: ReadonlyArray<{ id: string; name: string; description?: string }>;
  enterToSend: boolean;
  onComposerChange(value: string, richText?: string): void;
  onComposerMention?(candidate: { id: string; name: string; description?: string; kind?: 'agent' | 'mcp' }): void;
  onComposerWorkflowReference?(candidate: { id: string; name: string; description?: string }): void;
  onComposerSubmit(event: FormEvent<HTMLFormElement>): void;
  onComposerFiles(files: readonly File[]): void;
  onRemoveComposerAttachment(id: string): void;
  onClearComposerReply?(): void;
  onTranscribeVoice?(file: File): Promise<string>;
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
    <AgentHeader
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
    {props.searchActive ? <AgentSearch
      entries={props.entries}
      query={props.searchQuery}
      onQuery={props.onSearchQuery}
      onClose={props.onToggleSearch}
      onSelect={props.onSelectSearchResult}
    /> : null}
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
      onResolveApproval={props.onResolveApproval}
      onContextMenu={props.onContextMenu}
    />
    {props.beforeComposer}
    {props.composerAccessory}
    <AgentComposer
      value={props.composerValue}
      richText={props.composerRichText}
      scopeKey={props.botId}
      agentName={props.title}
      ready={props.composerReady}
      busy={props.composerBusy}
      uploading={props.composerUploading}
      attachments={props.composerAttachments}
      replyTarget={props.composerReplyTarget}
      mentionCandidates={props.composerMentionCandidates}
      workflowCandidates={props.composerWorkflowCandidates}
      onClearReplyTarget={props.onClearComposerReply}
      onMention={props.onComposerMention}
      onWorkflowReference={props.onComposerWorkflowReference}
      enterToSend={props.enterToSend}
      onChange={props.onComposerChange}
      onSubmit={props.onComposerSubmit}
      onAttachFiles={props.onComposerFiles}
      onRemoveAttachment={props.onRemoveComposerAttachment}
      onTranscribeVoice={props.onTranscribeVoice}
      onStop={props.onStop}
    />
  </>;
}
