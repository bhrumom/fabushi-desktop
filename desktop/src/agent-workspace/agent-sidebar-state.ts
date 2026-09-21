import { invokeNativeDesktop } from '../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';

export const AGENT_SIDEBAR_UNASSIGNED_ID = '__agents__';

export interface AgentSidebarSection {
  id: string;
  name: string;
  agentKeys: string[];
  isCollapsed: boolean;
}

export interface AgentSidebarSectionProjection<T> {
  id: string;
  name: string;
  isSynthetic: boolean;
  isCollapsed: boolean;
  agents: T[];
}

const schemaVersion = 1;

function storageKey(accountScope: string): string {
  return `fabushi.agent-sidebar.sections.v1.${encodeURIComponent(accountScope)}`;
}

function parseSectionSnapshot(value: unknown): AgentSidebarSection[] | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const parsed = value as { schemaVersion?: unknown; sections?: unknown };
  if (parsed.schemaVersion !== schemaVersion || !Array.isArray(parsed.sections)) return null;
  return normalizeAgentSidebarSections(parsed.sections.filter(isSection));
}

function isSection(value: unknown): value is AgentSidebarSection {
  if (!value || typeof value !== 'object') return false;
  const section = value as Partial<AgentSidebarSection>;
  return typeof section.id === 'string'
    && typeof section.name === 'string'
    && Array.isArray(section.agentKeys)
    && section.agentKeys.every((key) => typeof key === 'string')
    && typeof section.isCollapsed === 'boolean';
}

export function normalizeAgentSidebarSections(sections: readonly AgentSidebarSection[]): AgentSidebarSection[] {
  const claimed = new Set<string>();
  const seen = new Set<string>();
  const result: AgentSidebarSection[] = [];
  for (const input of sections) {
    const id = input.id.trim();
    if (!id || id === AGENT_SIDEBAR_UNASSIGNED_ID || seen.has(id)) continue;
    seen.add(id);
    const agentKeys: string[] = [];
    for (const key of input.agentKeys) {
      if (!key || claimed.has(key)) continue;
      claimed.add(key);
      agentKeys.push(key);
    }
    result.push({
      id,
      name: input.name.trim() || 'Section',
      agentKeys,
      isCollapsed: Boolean(input.isCollapsed),
    });
  }
  return result;
}

export function readAgentSidebarSections(accountScope: string | null | undefined): AgentSidebarSection[] {
  if (!accountScope || typeof window === 'undefined') return [];
  try {
    const raw = window.localStorage.getItem(storageKey(accountScope));
    if (!raw) return [];
    return parseSectionSnapshot(JSON.parse(raw)) ?? [];
  } catch {
    return [];
  }
}

export async function readAgentSidebarSectionsDurable(
  accountScope: string | null | undefined,
): Promise<AgentSidebarSection[]> {
  const fallback = readAgentSidebarSections(accountScope);
  if (!accountScope) return fallback;
  try {
    const stored = await invokeNativeDesktop<unknown>('readClientPersistence', {
      key: storageKey(accountScope),
    });
    const native = parseSectionSnapshot(stored);
    return native ?? fallback;
  } catch {
    return fallback;
  }
}

export function createAgentSidebarSection(
  sections: readonly AgentSidebarSection[],
  name: string,
  agentKeys: readonly string[] = [],
): { id: string; sections: AgentSidebarSection[] } {
  const id = `section-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
  return {
    id,
    sections: normalizeAgentSidebarSections([
      { id, name: name.trim() || 'New section', agentKeys: [...agentKeys], isCollapsed: false },
      ...sections,
    ]),
  };
}

export function assignAgentsToSidebarSection(
  sections: readonly AgentSidebarSection[],
  agentKeys: readonly string[],
  sectionId: string,
): AgentSidebarSection[] {
  const moved = new Set(agentKeys);
  const unassigned = sections.map((section) => ({
    ...section,
    agentKeys: section.agentKeys.filter((key) => !moved.has(key)),
  }));
  if (sectionId === AGENT_SIDEBAR_UNASSIGNED_ID) return normalizeAgentSidebarSections(unassigned);
  return normalizeAgentSidebarSections(unassigned.map((section) => section.id === sectionId
    ? { ...section, agentKeys: [...section.agentKeys, ...agentKeys] }
    : section));
}

export function removeAgentSidebarSection(
  sections: readonly AgentSidebarSection[],
  sectionId: string,
): AgentSidebarSection[] {
  if (sectionId === AGENT_SIDEBAR_UNASSIGNED_ID) return normalizeAgentSidebarSections(sections);
  return normalizeAgentSidebarSections(sections.filter((section) => section.id !== sectionId));
}

export function renameAgentSidebarSection(
  sections: readonly AgentSidebarSection[],
  sectionId: string,
  name: string,
): AgentSidebarSection[] {
  if (sectionId === AGENT_SIDEBAR_UNASSIGNED_ID) return normalizeAgentSidebarSections(sections);
  return normalizeAgentSidebarSections(sections.map((section) => section.id === sectionId
    ? { ...section, name: name.trim() || section.name }
    : section));
}

export function toggleAgentSidebarSection(
  sections: readonly AgentSidebarSection[],
  sectionId: string,
): AgentSidebarSection[] {
  if (sectionId === AGENT_SIDEBAR_UNASSIGNED_ID) return normalizeAgentSidebarSections(sections);
  return normalizeAgentSidebarSections(sections.map((section) => section.id === sectionId
    ? { ...section, isCollapsed: !section.isCollapsed }
    : section));
}

export function projectAgentSidebarSections<T extends { key: string; pinned: boolean }>(
  agents: readonly T[],
  sections: readonly AgentSidebarSection[],
): AgentSidebarSectionProjection<T>[] {
  const unpinned = agents.filter((agent) => !agent.pinned);
  const claimed = new Map<string, string>();
  for (const section of sections) {
    for (const key of section.agentKeys) claimed.set(key, section.id);
  }
  if (!sections.length) {
    return [{
      id: AGENT_SIDEBAR_UNASSIGNED_ID,
      name: 'Agents',
      isSynthetic: true,
      isCollapsed: false,
      agents: unpinned,
    }];
  }
  const projected = sections.map((section) => ({
    id: section.id,
    name: section.name,
    isSynthetic: false,
    isCollapsed: section.isCollapsed,
    agents: unpinned.filter((agent) => claimed.get(agent.key) === section.id),
  }));
  projected.push({
    id: AGENT_SIDEBAR_UNASSIGNED_ID,
    name: 'Unassigned',
    isSynthetic: true,
    isCollapsed: false,
    agents: unpinned.filter((agent) => !claimed.has(agent.key)),
  });
  return projected;
}
