import { Monitor, Pin, Search, Settings, X } from 'lucide-react';
import React from 'react';
import { BotMark, type BotMarkState } from '../../../frontend/apps/web/src/app/host/bot-mark';
import type { ComputerStatus } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { RemoteComputerDesktopState } from '../../../frontend/apps/web/src/lib/remote-computer/desktop-peer';
import AgentSettingsPanel, { type AgentSettingsProfileUpdate, type AgentSettingsProfileValue } from './agent-settings-panel';
import styles from './agent-overlays.module.css';

export interface AgentOverlayComputerProps {
  agentId: string;
  open: boolean;
  label: string;
  status: string;
  online: boolean;
  aiControlEnabled: boolean;
  remoteControlEnabled: boolean;
  state: RemoteComputerDesktopState | null;
  capabilityStatus?: ComputerStatus | null;
  onToggle(): void;
  onRefreshPairingCode(): void;
  onApproveSession(sessionId: string): void;
  onDenySession(sessionId: string): void;
  onDisconnect(): void;
  onToggleRemoteControl(): void;
  onOpenControlPage(): void;
}

export interface AgentOverlaySettingsProps {
  agentId: string;
  open: boolean;
  value: AgentSettingsProfileValue;
  pending: import('./agent-settings-controller').AgentSettingsPending;
  error: string | null;
  onToggle(): void;
  onUpdateProfile(profile: AgentSettingsProfileUpdate): Promise<void>;
  onSetNotifications(enabled: boolean): Promise<void>;
}

export interface AgentOverlaysProps {
  title: string;
  description: string;
  botId: string;
  botState: BotMarkState;
  pinned: boolean;
  overlay: boolean;
  computer: AgentOverlayComputerProps;
  settings: AgentOverlaySettingsProps;
  onClose(): void;
  onSearch(): void;
  onTogglePin(): void;
}

/**
 * Agent-owned secondary surface for profile and Computer.
 *
 * Messenger-only concepts such as voice/video calls, archive and payment are
 * intentionally absent. Computer is projected as a first-class Agent
 * capability while its executor remains the installed Fabushi machine.
 */
export default function AgentOverlays(props: AgentOverlaysProps) {
  const { computer, settings } = props;
  const pending = computer.state?.pendingAuthorization;
  return <aside className={styles.root} data-testid="agent-overlays" data-overlay={props.overlay || undefined}>
    <header className={styles.header}>
      <strong>Agent</strong>
      <button type="button" onClick={props.onClose} aria-label="Close Agent info"><X size={17} /></button>
    </header>

    <section className={styles.identity}>
      <BotMark botId={props.botId} state={props.botState} size={88} label={props.title} />
      <strong>{props.title}</strong>
      <small>{props.description}</small>
      <div className={styles.quickActions}>
        <button type="button" onClick={props.onSearch}><Search size={17} /><span>Search</span></button>
        <button type="button" data-active={props.pinned || undefined} onClick={props.onTogglePin}><Pin size={17} /><span>{props.pinned ? 'Unpin' : 'Pin'}</span></button>
        <button type="button" data-testid="bot-computer-toggle" data-active={computer.open || undefined} onClick={computer.onToggle}><Monitor size={17} /><span>Computer</span></button>
        <button type="button" data-testid="agent-settings-toggle" data-active={settings.open || undefined} onClick={settings.onToggle}><Settings size={17} /><span>Settings</span></button>
      </div>
    </section>

    {settings.open ? <AgentSettingsPanel
      agentId={settings.agentId}
      value={settings.value}
      pending={settings.pending}
      error={settings.error}
      onUpdateProfile={settings.onUpdateProfile}
      onSetNotifications={settings.onSetNotifications}
    /> : null}

    {computer.open ? <section className={styles.computer} data-testid="bot-computer-panel" data-agent-id={computer.agentId}>
      <header>
        <span className={styles.computerIcon}><Monitor size={18} /></span>
        <span><strong>This computer</strong><small>{computer.label}</small></span>
        <i data-live={computer.state?.channelOpen ? 'active' : computer.online ? 'online' : 'offline'} />
      </header>
      <div className={styles.metrics}>
        <span><small>Device</small><strong>{computer.status}</strong></span>
        <span><small>Authorized clients</small><strong>{computer.state?.clients.length ?? 0}</strong></span>
        <span><small>Agent control</small><strong>{computer.aiControlEnabled ? 'Allowed' : 'Off'}</strong></span>
      </div>
      {computer.capabilityStatus ? <div className={styles.metrics} data-testid="agent-computer-permissions">
        <span><small>Screen Recording</small><strong>{computer.capabilityStatus.screenRecordingGranted ? 'Granted' : 'Required'}</strong></span>
        <span><small>Accessibility</small><strong>{computer.capabilityStatus.accessibilityGranted ? 'Granted' : 'Required'}</strong></span>
        <span><small>Capture</small><strong>{computer.capabilityStatus.captureSupported ? 'Supported' : 'Unavailable'}</strong></span>
        <span><small>Input</small><strong>{computer.capabilityStatus.inputSupported ? 'Supported' : 'Unavailable'}</strong></span>
        <span><small>Local execution</small><strong>{computer.capabilityStatus.localExecutionEnabled ? 'Enabled' : 'Off'}</strong></span>
        <span><small>Platform</small><strong>{computer.capabilityStatus.platform}</strong></span>
      </div> : null}
      <p>The Computer surface belongs to this Agent. Execution is bound to the machine where Fabushi is installed, not a cloud computer.</p>

      {computer.remoteControlEnabled && computer.state?.registration?.pairingCode ? <div className={styles.request}>
        <span><small>Pairing code</small><strong>{computer.state.registration.pairingCode}</strong></span>
        <button type="button" onClick={computer.onRefreshPairingCode}>Refresh</button>
      </div> : null}

      {pending ? <div className={styles.request} data-testid="remote-session-consent">
        <span><small>Remote request</small><strong>{pending.clientLabel || 'Paired device'}</strong></span>
        <button type="button" onClick={() => computer.onApproveSession(pending.sessionId)}>Allow once</button>
        <button type="button" data-danger="true" onClick={() => computer.onDenySession(pending.sessionId)}>Deny</button>
      </div> : null}

      {computer.state?.activeSessionId ? <button type="button" className={styles.disconnect} onClick={computer.onDisconnect}>Disconnect active session</button> : null}
      <div className={styles.computerActions}>
        <button type="button" data-enabled={computer.remoteControlEnabled || undefined} onClick={computer.onToggleRemoteControl}>
          {computer.remoteControlEnabled ? 'Disable remote control' : 'Enable remote control'}
        </button>
        <button type="button" onClick={computer.onOpenControlPage}>Open Computer</button>
      </div>
      {computer.state?.error ? <small className={styles.error}>{computer.state.error}</small> : null}
    </section> : null}
  </aside>;
}
