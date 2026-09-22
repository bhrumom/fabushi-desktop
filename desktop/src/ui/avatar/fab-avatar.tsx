import React, { useSyncExternalStore } from 'react';
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

export interface FabAvatarIdentityAlias {
  readonly alias: string;
  readonly canonical: string;
}

const identityAliases = new Map<string, string>();
const identityAliasListeners = new Set<() => void>();
let identityAliasRevision = 0;

function aliasSnapshot(): number {
  return identityAliasRevision;
}

function subscribeAliasRegistry(listener: () => void): () => void {
  identityAliasListeners.add(listener);
  return () => identityAliasListeners.delete(listener);
}

export function registerFabAvatarIdentityAliases(aliases: readonly FabAvatarIdentityAlias[]): void {
  let changed = false;
  for (const entry of aliases) {
    const alias = entry.alias.trim();
    const canonical = entry.canonical.trim();
    if (!alias || !canonical || identityAliases.get(alias) === canonical) continue;
    identityAliases.set(alias, canonical);
    changed = true;
  }
  if (!changed) return;
  identityAliasRevision += 1;
  identityAliasListeners.forEach((listener) => listener());
}

export function resolveFabAvatarIdentity(identity: string): string {
  let current = identity.trim() || 'fabushi:unknown';
  const seen = new Set<string>();
  while (!seen.has(current)) {
    seen.add(current);
    const next = identityAliases.get(current);
    if (!next) break;
    current = next;
  }
  return current;
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
  useSyncExternalStore(subscribeAliasRegistry, aliasSnapshot, aliasSnapshot);
  const canonicalIdentity = resolveFabAvatarIdentity(identity);
  const normalized = normalizeFabAvatarState(state);
  const hue = identityHue(canonicalIdentity);
  const style = {
    '--fab-avatar-size': `${Math.max(16, Math.round(size))}px`,
    '--fab-avatar-hue': String(hue),
  } as React.CSSProperties;
  return <span
    className={[styles.root, className].filter(Boolean).join(' ')}
    style={style}
    role="img"
    aria-label={`${label} · ${normalized}`}
    data-fab-avatar="true"
    data-avatar-input={identity}
    data-avatar-identity={canonicalIdentity}
    data-shape={`fab-geometric-${hue % 4}`}
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
