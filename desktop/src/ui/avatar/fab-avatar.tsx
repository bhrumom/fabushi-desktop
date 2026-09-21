import React from 'react';
import styles from './fab-avatar.module.css';

export type FabAvatarState =
  | 'idle'
  | 'thinking'
  | 'working'
  | 'waiting'
  | 'speaking'
  | 'success'
  | 'error'
  | 'offline';

export type FabAvatarInputState =
  | FabAvatarState
  | 'sending'
  | 'waking'
  | 'alerting'
  | 'result'
  | 'sleeping'
  | 'listening'
  | 'notifying';

export function normalizeFabAvatarState(state: FabAvatarInputState): FabAvatarState {
  switch (state) {
    case 'sending':
    case 'thinking':
      return 'thinking';
    case 'working':
      return 'working';
    case 'alerting':
    case 'waiting':
    case 'notifying':
      return 'waiting';
    case 'speaking':
    case 'listening':
      return 'speaking';
    case 'result':
    case 'success':
      return 'success';
    case 'error':
      return 'error';
    case 'sleeping':
    case 'offline':
    case 'waking':
      return 'offline';
    default:
      return 'idle';
  }
}

function identityHue(identity: string): number {
  let hash = 2166136261;
  for (let index = 0; index < identity.length; index += 1) {
    hash ^= identity.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return Math.abs(hash) % 360;
}

export interface FabAvatarProps {
  readonly identity: string;
  readonly label: string;
  readonly state?: FabAvatarInputState;
  readonly size?: number;
  readonly active?: boolean;
  readonly className?: string;
}

/**
 * Low-power Agent identity.
 *
 * The component owns no timers, observers or requestAnimationFrame loop.
 * Roster instances are static. Only an explicitly active identity may opt into
 * a small CSS animation, which is disabled by prefers-reduced-motion.
 */
export default function FabAvatar({
  identity,
  label,
  state = 'idle',
  size = 36,
  active = false,
  className,
}: FabAvatarProps) {
  const normalized = normalizeFabAvatarState(state);
  const style = {
    '--fab-avatar-size': `${Math.max(16, Math.round(size))}px`,
    '--fab-avatar-hue': String(identityHue(identity)),
  } as React.CSSProperties;
  return <span
    className={[styles.root, className].filter(Boolean).join(' ')}
    style={style}
    role="img"
    aria-label={`${label} · ${normalized}`}
    data-engine="fab-avatar"
    data-shape={`hue-${identityHue(identity)}`}
    data-state={normalized}
    data-active={active || undefined}
    title={label}
  >
    <span className={styles.face} aria-hidden="true">
      <i className={styles.eye} />
      <i className={styles.eye} />
    </span>
    <span className={styles.status} aria-hidden="true" />
  </span>;
}
