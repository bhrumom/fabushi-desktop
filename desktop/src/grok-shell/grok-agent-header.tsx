import { AppWindow, Monitor, MoreVertical, Pin, Search } from 'lucide-react';
import React from 'react';
import FabAvatar, { type FabAvatarInputState } from '../ui/avatar/fab-avatar';
import styles from './grok-agent-header.module.css';

export default function GrokAgentHeader({
  title,
  description,
  botId,
  botState,
  status,
  pinned,
  searchActive,
  computerActive,
  infoActive,
  miniAppTitle,
  onOpenMiniApp,
  onToggleSearch,
  onToggleComputer,
  onTogglePin,
  onToggleInfo,
}: {
  title: string;
  description: string;
  botId: string;
  botState: FabAvatarInputState;
  status: string;
  pinned: boolean;
  searchActive: boolean;
  computerActive: boolean;
  infoActive: boolean;
  miniAppTitle?: string;
  onOpenMiniApp?(): void;
  onToggleSearch(): void;
  onToggleComputer(): void;
  onTogglePin(): void;
  onToggleInfo(): void;
}) {
  return <header className={styles.root} data-testid="grok-agent-header">
    <div className={styles.identity}>
      <FabAvatar identity={botId} state={botState} size={34} label={title} active />
      <div>
        <strong>{title}</strong>
        <small data-testid="conversation-status">{status || description}</small>
      </div>
    </div>
    <div className={styles.actions}>
      {onOpenMiniApp ? <button type="button" data-testid="miniapp-bot-open" title={miniAppTitle ?? 'Open app'} onClick={onOpenMiniApp}><AppWindow size={17} /></button> : null}
      <button type="button" title="Search conversation" data-active={searchActive || undefined} onClick={onToggleSearch}><Search size={17} /></button>
      <button type="button" title="Computer" data-active={computerActive || undefined} onClick={onToggleComputer}><Monitor size={17} /></button>
      <button type="button" title={pinned ? 'Unpin' : 'Pin'} data-active={pinned || undefined} onClick={onTogglePin}><Pin size={17} /></button>
      <button type="button" title="Agent info" data-testid="conversation-info-toggle" data-active={infoActive || undefined} onClick={onToggleInfo}><MoreVertical size={17} /></button>
    </div>
  </header>;
}
