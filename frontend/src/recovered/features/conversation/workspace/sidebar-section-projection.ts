export const AGENTS_SECTION_ID = "__agents__";
export const AGENTS_SECTION_NAME = "Unassigned";
export const EMPTY_SECTION_BODY_LABEL = "Drag chats here";

export interface SidebarSection {
  id: string;
  name: string;
  agentIds: string[];
  isCollapsed: boolean;
}

export interface SidebarSectionProjection<Agent extends { id: string }> {
  id: string;
  name: string;
  isSynthetic: boolean;
  isCollapsed: boolean;
  agents: Agent[];
}

export interface SidebarSectionHeaderMetadata {
  id: string;
  label: string;
  count: number;
  isFolded: boolean;
  isLocked: boolean;
  ariaExpanded: boolean;
  dataSectionId: string;
}

export function projectSidebarSections<Agent extends { id: string }>(args: {
  readonly agents: readonly Agent[];
  readonly pinnedIds: readonly string[];
  readonly sections: readonly SidebarSection[];
}): SidebarSectionProjection<Agent>[] {
  if (args.sections.length === 0) return [];

  const pinned = new Set(args.pinnedIds);
  const available = args.agents.filter((agent) => !pinned.has(agent.id));
  const assigned = new Map<string, string>();
  for (const section of args.sections) {
    if (section.id === AGENTS_SECTION_ID) continue;
    for (const agentId of section.agentIds) {
      assigned.set(agentId, section.id);
    }
  }

  return args.sections
    .map((section) => {
      const isSynthetic = section.id === AGENTS_SECTION_ID;
      const agents = available.filter((agent) =>
        isSynthetic
          ? !assigned.has(agent.id)
          : assigned.get(agent.id) === section.id
      );
      return {
        id: section.id,
        name: section.name,
        isSynthetic,
        isCollapsed: section.isCollapsed,
        agents,
      };
    })
    .filter((section) => !section.isSynthetic || section.agents.length > 0);
}

export function projectSidebarSectionHeader(
  section: SidebarSectionProjection<{ id: string }>,
): SidebarSectionHeaderMetadata {
  return {
    id: section.id,
    label: section.name,
    count: section.agents.length,
    isFolded: section.isCollapsed,
    isLocked: section.isSynthetic,
    ariaExpanded: !section.isCollapsed,
    dataSectionId: section.id,
  };
}
