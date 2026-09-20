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
  maxAttempts = 3,
): Promise<AccountSidebarLayoutSnapshot> {
  let lastError: unknown = null;
  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    const current = await readAccountSidebarLayout(accountScope);
    const layout: AccountSidebarLayout = {
      schemaVersion: 1,
      accountScope,
      revision: Math.max(current?.layout.revision ?? 0, current?.serverRevision ?? 0) + 1,
      pinStateManaged: true,
      pinnedOrder: cleanOrder(pinnedOrder),
      sections: normalizeAgentSidebarSections(sections),
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
      // A concurrent device may have advanced the etag between read and write.
      // The next attempt always re-reads the authoritative server revision.
    }
  }
  throw lastError instanceof Error
    ? lastError
    : new Error(String(lastError ?? 'Account sidebar CAS write failed.'));
}
