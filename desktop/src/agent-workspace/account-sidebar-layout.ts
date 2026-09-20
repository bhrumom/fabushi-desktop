import {
  listAccountAgentStore,
  readAccountAgentStoreObject,
  writeAccountAgentStoreObject,
  type AccountAgentStoreObject,
} from '../account-sync-client';
import {
  normalizeAgentSidebarSections,
  type AgentSidebarSection,
} from './agent-sidebar-state';

export const ACCOUNT_SIDEBAR_LAYOUT_AGENT_ID = 'mahayana-assistant';
export const ACCOUNT_SIDEBAR_LAYOUT_PATH = 'account/sidebar-layout.v1.json';

export interface AccountSidebarLayout {
  schemaVersion: 1;
  accountScope: string;
  revision: number;
  /**
   * False only for pre-Agent-layout documents that still need a one-time
   * migration from legacy Messenger pin state.
   */
  pinStateManaged: boolean;
  pinnedOrder: string[];
  sections: AgentSidebarSection[];
  updatedAtMs: number;
}

export interface AccountSidebarLayoutSnapshot {
  layout: AccountSidebarLayout;
  etag: string;
  serverRevision: number;
}

export interface AccountSidebarLayoutState {
  pinnedOrder: string[];
  sections: AgentSidebarSection[];
}

export interface AccountSidebarLayoutWriteOptions {
  /**
   * Last authoritative cloud snapshot observed by this renderer. It is used as
   * the three-way merge base when another device advances the server etag.
   */
  base?: AccountSidebarLayoutSnapshot | null;
  maxAttempts?: number;
}

function sameOrder(left: readonly string[], right: readonly string[]): boolean {
  return left.length === right.length && left.every((entry, index) => entry === right[index]);
}

function sameSection(left: AgentSidebarSection, right: AgentSidebarSection): boolean {
  return left.id === right.id
    && left.name === right.name
    && left.isCollapsed === right.isCollapsed
    && sameOrder(left.agentKeys, right.agentKeys);
}

function sameSections(left: readonly AgentSidebarSection[], right: readonly AgentSidebarSection[]): boolean {
  return left.length === right.length && left.every((section, index) => sameSection(section, right[index]));
}

function mergeOrderedKeys(
  base: readonly string[],
  local: readonly string[],
  remote: readonly string[],
): string[] {
  if (sameOrder(local, base)) return [...remote];
  if (sameOrder(remote, base)) return [...local];

  const baseSet = new Set(base);
  const localSet = new Set(local);
  const remoteSet = new Set(remote);
  const localRemoved = new Set(base.filter((key) => !localSet.has(key)));
  const remoteRemoved = new Set(base.filter((key) => !remoteSet.has(key)));
  const merged: string[] = [];
  const seen = new Set<string>();

  const append = (key: string) => {
    if (!key || seen.has(key)) return;
    if (baseSet.has(key) && (localRemoved.has(key) || remoteRemoved.has(key))) return;
    seen.add(key);
    merged.push(key);
  };

  // Local ordering is authoritative for keys this renderer still owns, while
  // remote-only additions are retained below. A deletion on either device wins
  // over a concurrent reorder so unpin/move operations cannot be resurrected.
  local.forEach(append);
  remote.forEach(append);
  return merged;
}

function mergeSectionValue(
  base: AgentSidebarSection,
  local: AgentSidebarSection,
  remote: AgentSidebarSection,
): AgentSidebarSection {
  if (sameSection(local, base)) return { ...remote, agentKeys: [...remote.agentKeys] };
  if (sameSection(remote, base)) return { ...local, agentKeys: [...local.agentKeys] };
  return {
    id: local.id,
    name: local.name !== base.name ? local.name : remote.name,
    isCollapsed: local.isCollapsed !== base.isCollapsed ? local.isCollapsed : remote.isCollapsed,
    agentKeys: mergeOrderedKeys(base.agentKeys, local.agentKeys, remote.agentKeys),
  };
}

/**
 * Three-way merge for the account-scoped Sidebar document.
 *
 * This is deliberately identity/revision based rather than last-write-wins:
 * non-conflicting edits from another device survive a local CAS retry; removal
 * wins over a concurrent reorder; local section ordering is retained while
 * remote-only sections/Agent additions are appended.
 */
export function mergeAccountSidebarLayoutState(
  baseLayout: Pick<AccountSidebarLayout, 'pinnedOrder' | 'sections'> | null,
  localState: AccountSidebarLayoutState,
  remoteLayout: Pick<AccountSidebarLayout, 'pinnedOrder' | 'sections'> | null,
): AccountSidebarLayoutState {
  const basePinned = cleanOrder(baseLayout?.pinnedOrder ?? []);
  const localPinned = cleanOrder(localState.pinnedOrder);
  const remotePinned = cleanOrder(remoteLayout?.pinnedOrder ?? []);
  const baseSections = normalizeAgentSidebarSections(baseLayout?.sections ?? []);
  const localSections = normalizeAgentSidebarSections(localState.sections);
  const remoteSections = normalizeAgentSidebarSections(remoteLayout?.sections ?? []);

  const baseById = new Map(baseSections.map((section) => [section.id, section] as const));
  const localById = new Map(localSections.map((section) => [section.id, section] as const));
  const remoteById = new Map(remoteSections.map((section) => [section.id, section] as const));
  const mergedById = new Map<string, AgentSidebarSection>();

  for (const id of new Set([...baseById.keys(), ...localById.keys(), ...remoteById.keys()])) {
    const base = baseById.get(id);
    const local = localById.get(id);
    const remote = remoteById.get(id);

    if (!base) {
      if (local && remote) {
        mergedById.set(id, {
          id,
          name: local.name || remote.name,
          isCollapsed: local.isCollapsed,
          agentKeys: mergeOrderedKeys([], local.agentKeys, remote.agentKeys),
        });
      } else if (local) {
        mergedById.set(id, { ...local, agentKeys: [...local.agentKeys] });
      } else if (remote) {
        mergedById.set(id, { ...remote, agentKeys: [...remote.agentKeys] });
      }
      continue;
    }

    // A deletion on either side wins. This prevents a CAS retry from
    // resurrecting a section that another device intentionally removed.
    if (!local || !remote) continue;
    mergedById.set(id, mergeSectionValue(base, local, remote));
  }

  const orderedSectionIds = [
    ...localSections.map((section) => section.id),
    ...remoteSections.map((section) => section.id).filter((id) => !localById.has(id)),
  ];
  const sections = normalizeAgentSidebarSections(
    orderedSectionIds
      .map((id) => mergedById.get(id))
      .filter((section): section is AgentSidebarSection => Boolean(section)),
  );

  return {
    pinnedOrder: mergeOrderedKeys(basePinned, localPinned, remotePinned),
    sections,
  };
}

function cleanOrder(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return [...new Set(value.filter((entry): entry is string =>
    typeof entry === 'string' && entry.trim().length > 0,
  ).map((entry) => entry.trim()))];
}

function decodeBase64Json(dataBase64: string): unknown {
  const binary = atob(dataBase64);
  const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
  return JSON.parse(new TextDecoder().decode(bytes));
}

function encodeBase64Json(value: unknown): string {
  const bytes = new TextEncoder().encode(JSON.stringify(value));
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function parseLayout(value: unknown, accountScope: string): AccountSidebarLayout | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const candidate = value as Partial<AccountSidebarLayout>;
  if (candidate.schemaVersion !== 1 || candidate.accountScope !== accountScope) return null;
  const revision = Number(candidate.revision);
  return {
    schemaVersion: 1,
    accountScope,
    revision: Number.isSafeInteger(revision) && revision >= 0 ? revision : 0,
    pinStateManaged: candidate.pinStateManaged === true,
    pinnedOrder: cleanOrder(candidate.pinnedOrder),
    sections: Array.isArray(candidate.sections)
      ? normalizeAgentSidebarSections(candidate.sections.filter((section): section is AgentSidebarSection => {
          if (!section || typeof section !== 'object') return false;
          const value = section as Partial<AgentSidebarSection>;
          return typeof value.id === 'string'
            && typeof value.name === 'string'
            && Array.isArray(value.agentKeys)
            && value.agentKeys.every((key) => typeof key === 'string')
            && typeof value.isCollapsed === 'boolean';
        }))
      : [],
    updatedAtMs: Number.isFinite(Number(candidate.updatedAtMs))
      ? Number(candidate.updatedAtMs)
      : 0,
  };
}

async function listedObject(): Promise<AccountAgentStoreObject | null> {
  const listing = await listAccountAgentStore(
    ACCOUNT_SIDEBAR_LAYOUT_AGENT_ID,
    ACCOUNT_SIDEBAR_LAYOUT_PATH,
  );
  return listing.files.find((file) => file.path === ACCOUNT_SIDEBAR_LAYOUT_PATH) ?? null;
}

export async function readAccountSidebarLayout(
  accountScope: string,
): Promise<AccountSidebarLayoutSnapshot | null> {
  const listed = await listedObject();
  if (!listed) return null;
  const object = listed.dataBase64
    ? listed
    : await readAccountAgentStoreObject(
        ACCOUNT_SIDEBAR_LAYOUT_AGENT_ID,
        ACCOUNT_SIDEBAR_LAYOUT_PATH,
      );
  if (!object.dataBase64) return null;
  const layout = parseLayout(decodeBase64Json(object.dataBase64), accountScope);
  if (!layout) return null;
  return {
    layout,
    etag: object.etag,
    serverRevision: object.revision,
  };
}

/**
 * Writes one account-wide sidebar document using the existing Agent-store CAS
 * service. Every retry first observes the newest server etag/revision; there is
 * no unconditional cloud overwrite. The canonical built-in Mahayana Agent is
 * only the authenticated account namespace anchor -- no renderer Agent state is
 * stored in this document.
 */
export async function writeAccountSidebarLayout(
  accountScope: string,
  pinnedOrder: readonly string[],
  sections: readonly AgentSidebarSection[],
  options: AccountSidebarLayoutWriteOptions = {},
): Promise<AccountSidebarLayoutSnapshot> {
  const maxAttempts = Math.max(1, options.maxAttempts ?? 3);
  const base = options.base?.layout?.accountScope === accountScope
    ? options.base.layout
    : null;
  const localState: AccountSidebarLayoutState = {
    pinnedOrder: cleanOrder(pinnedOrder),
    sections: normalizeAgentSidebarSections(sections),
  };

  let lastError: unknown = null;
  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    const current = await readAccountSidebarLayout(accountScope);
    const merged = mergeAccountSidebarLayoutState(base, localState, current?.layout ?? null);

    if (current
      && sameOrder(current.layout.pinnedOrder, merged.pinnedOrder)
      && sameSections(current.layout.sections, merged.sections)
      && current.layout.pinStateManaged) {
      return current;
    }

    const layout: AccountSidebarLayout = {
      schemaVersion: 1,
      accountScope,
      revision: Math.max(current?.layout.revision ?? 0, current?.serverRevision ?? 0) + 1,
      pinStateManaged: true,
      pinnedOrder: merged.pinnedOrder,
      sections: merged.sections,
      updatedAtMs: Date.now(),
    };
    try {
      await writeAccountAgentStoreObject(
        ACCOUNT_SIDEBAR_LAYOUT_AGENT_ID,
        ACCOUNT_SIDEBAR_LAYOUT_PATH,
        encodeBase64Json(layout),
        current ? { baseEtag: current.etag } : { expectAbsent: true },
      );
      const committed = await readAccountSidebarLayout(accountScope);
      if (!committed) throw new Error('Account sidebar CAS write completed without a readable snapshot.');
      return committed;
    } catch (cause) {
      lastError = cause;
      // A concurrent device advanced the etag. Retry against the latest
      // server snapshot while preserving both sides with the original base.
    }
  }
  throw lastError instanceof Error
    ? lastError
    : new Error(String(lastError ?? 'Account sidebar CAS write failed.'));
}
