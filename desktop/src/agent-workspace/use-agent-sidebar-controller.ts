import { useCallback, useEffect, useRef, useState } from 'react';
import type { RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { invokeNativeDesktop, subscribeNativeDesktopEvents } from '../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type { AgentCoordinatorClient } from './coordinator-client';
import {
  assignAgentsToSidebarSection,
  createAgentSidebarSection,
  normalizeAgentSidebarSections,
  readAgentSidebarSections,
  readAgentSidebarSectionsDurable,
  removeAgentSidebarSection,
  renameAgentSidebarSection,
  toggleAgentSidebarSection,
  type AgentSidebarSection,
} from './agent-sidebar-state';
import {
  mergeAccountSidebarLayoutState,
  readAccountSidebarLayout,
  writeAccountSidebarLayout,
  type AccountSidebarLayoutSnapshot,
} from './account-sidebar-layout';

const legacyPinnedOrderKey = 'fabushi.desktop.grok-pinned-order.v1';

function pinnedOrderKey(accountScope: string): string {
  return `fabushi.desktop.agent-pinned-order.v2.${encodeURIComponent(accountScope)}`;
}

function pinStateManagedKey(accountScope: string): string {
  return 'fabushi.desktop.agent-pin-state-managed.v1.' + encodeURIComponent(accountScope);
}

function readLegacyPinStateManaged(accountScope: string): boolean {
  if (typeof window === 'undefined') return false;
  try {
    return window.localStorage.getItem(pinStateManagedKey(accountScope)) === '1';
  } catch {
    return false;
  }
}

export interface AgentSidebarStateItem {
  readonly key: string;
  readonly pinned: boolean;
}

export interface AgentSidebarController {
  readonly ready: boolean;
  readonly pinnedOrder: readonly string[];
  readonly sections: readonly AgentSidebarSection[];
  readonly selectedKeys: readonly string[];
  handle(event: RuntimeEvent): boolean;
  adoptLegacyPinnedState(pinnedKeys: readonly string[]): void;
  togglePin(key: string): void;
  reorderPinned(
    movedKey: string,
    targetKey: string,
    position: 'before' | 'after',
    pinnedKeys: readonly string[],
  ): void;
  toggleSelection(key: string): void;
  rangeSelect(key: string, orderedKeys: readonly string[]): void;
  clearSelection(): void;
  createSection(name: string, items: readonly AgentSidebarStateItem[]): void;
  moveToSection(items: readonly AgentSidebarStateItem[], sectionId: string): void;
  renameSection(sectionId: string, name: string): void;
  removeSection(sectionId: string): void;
  toggleSection(sectionId: string): void;
}

function normalizePinnedOrder(value: unknown): string[] {
  return Array.isArray(value)
    ? [...new Set(value.filter((entry): entry is string =>
        typeof entry === 'string' && entry.trim().length > 0,
      ).map((entry) => entry.trim()))]
    : [];
}

interface SidebarWorkspaceSnapshot {
  readonly schemaVersion: 1;
  readonly accountScope: string;
  readonly pinStateManaged: boolean;
  readonly pinnedOrder: string[];
  readonly sections: AgentSidebarSection[];
}

function sidebarWorkspaceStateKey(accountScope: string): string {
  return `agent-sidebar:layout:v1.${encodeURIComponent(accountScope)}`;
}

function sidebarRequestId(prefix: string): string {
  return `${prefix}:${Date.now().toString(36)}:${crypto.randomUUID()}`;
}

function parseSidebarWorkspaceSnapshot(
  value: unknown,
  accountScope: string,
): SidebarWorkspaceSnapshot | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const parsed = value as Partial<SidebarWorkspaceSnapshot>;
  if (parsed.schemaVersion !== 1 || parsed.accountScope !== accountScope) return null;
  return {
    schemaVersion: 1,
    accountScope,
    pinStateManaged: parsed.pinStateManaged === true,
    pinnedOrder: normalizePinnedOrder(parsed.pinnedOrder),
    sections: Array.isArray(parsed.sections)
      ? normalizeAgentSidebarSections(parsed.sections.filter((entry): entry is AgentSidebarSection =>
          Boolean(entry) && typeof entry === 'object' && !Array.isArray(entry),
        ))
      : [],
  };
}

function createSidebarWorkspaceSnapshot(
  accountScope: string,
  pinStateManaged: boolean,
  pinnedOrder: readonly string[],
  sections: readonly AgentSidebarSection[],
): SidebarWorkspaceSnapshot {
  return {
    schemaVersion: 1,
    accountScope,
    pinStateManaged,
    pinnedOrder: normalizePinnedOrder(pinnedOrder),
    sections: normalizeAgentSidebarSections(sections),
  };
}

function readPinnedOrder(accountScope: string): string[] {
  if (typeof window === 'undefined') return [];
  for (const key of [pinnedOrderKey(accountScope), legacyPinnedOrderKey]) {
    try {
      const raw = window.localStorage.getItem(key);
      if (!raw) continue;
      const value = normalizePinnedOrder(JSON.parse(raw));
      if (value.length) return value;
    } catch {
      // Try the next durable mirror.
    }
  }
  return [];
}

async function readPinnedOrderDurable(accountScope: string): Promise<string[]> {
  const fallback = readPinnedOrder(accountScope);
  for (const key of [pinnedOrderKey(accountScope), legacyPinnedOrderKey]) {
    try {
      const value = normalizePinnedOrder(
        await invokeNativeDesktop<unknown>('readClientPersistence', { key }),
      );
      if (value.length) return value;
    } catch {
      // Native persistence is an offline mirror; cloud hydration below remains authoritative.
    }
  }
  return fallback;
}

function sameStringOrder(left: readonly string[], right: readonly string[]): boolean {
  return left.length === right.length && left.every((entry, index) => entry === right[index]);
}

function sameSections(left: readonly AgentSidebarSection[], right: readonly AgentSidebarSection[]): boolean {
  return left.length === right.length && left.every((section, index) => {
    const candidate = right[index];
    return Boolean(candidate)
      && section.id === candidate.id
      && section.name === candidate.name
      && section.isCollapsed === candidate.isCollapsed
      && sameStringOrder(section.agentKeys, candidate.agentKeys);
  });
}

function layoutMatchesSnapshot(
  pinnedOrder: readonly string[],
  sections: readonly AgentSidebarSection[],
  snapshot: AccountSidebarLayoutSnapshot | null,
): boolean {
  return Boolean(snapshot)
    && sameStringOrder(pinnedOrder, snapshot!.layout.pinnedOrder)
    && sameSections(sections, snapshot!.layout.sections);
}

function cloneSections(sections: readonly AgentSidebarSection[]): AgentSidebarSection[] {
  return sections.map((section) => ({ ...section, agentKeys: [...section.agentKeys] }));
}

/**
 * Agent-owned sidebar state controller.
 *
 * Pinned ordering and custom sections are one account workspace document.
 * Mahayana RuntimeStore is the local durable source; the account Agent-store
 * CAS object provides cross-device convergence. Legacy local/native values are
 * read only once as migration input and are never written by the renderer.
 */
export function useAgentSidebarController(
  client: AgentCoordinatorClient,
  accountScope: string | null | undefined,
): AgentSidebarController {
  const [pinnedOrder, setPinnedOrderState] = useState<string[]>([]);
  const [sections, setSections] = useState<AgentSidebarSection[]>([]);
  const [layoutScope, setLayoutScope] = useState<string | null>(null);
  const [pinStateManaged, setPinStateManaged] = useState(false);
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const selectionAnchorRef = useRef<string | null>(null);
  const pinStateManagedRef = useRef(false);
  const layoutMutationRevisionRef = useRef(0);
  const cloudWriteChainRef = useRef<Promise<unknown>>(Promise.resolve());
  const cloudSnapshotRef = useRef<AccountSidebarLayoutSnapshot | null>(null);
  const pinnedOrderRef = useRef<string[]>([]);
  const sectionsRef = useRef<AgentSidebarSection[]>([]);
  const activeScopeRef = useRef<string | null>(null);
  const runtimeHydrationRef = useRef<{
    scope: string;
    key: string;
    resolve: (snapshot: SidebarWorkspaceSnapshot | null) => void;
  } | null>(null);
  const runtimeWriteChainRef = useRef<Promise<unknown>>(Promise.resolve());

  useEffect(() => {
    pinnedOrderRef.current = [...pinnedOrder];
  }, [pinnedOrder]);

  useEffect(() => {
    sectionsRef.current = cloneSections(sections);
  }, [sections]);

  const handle = useCallback((event: RuntimeEvent): boolean => {
    const pending = runtimeHydrationRef.current;
    if (event.type !== 'agent.workspaceState' || !pending || event.key !== pending.key) return false;
    runtimeHydrationRef.current = null;
    pending.resolve(parseSidebarWorkspaceSnapshot(event.value, pending.scope));
    return true;
  }, []);

  const updatePinnedOrder = useCallback((build: (current: readonly string[]) => string[]) => {
    setPinnedOrderState((current) => {
      const next = build(current);
      if (next.length === current.length && next.every((key, index) => key === current[index])) {
        return current;
      }
      layoutMutationRevisionRef.current += 1;
      return next;
    });
  }, []);

  useEffect(() => {
    setSelectedKeys([]);
    selectionAnchorRef.current = null;
    setLayoutScope(null);
    activeScopeRef.current = null;
    cloudSnapshotRef.current = null;
    pinStateManagedRef.current = false;
    setPinStateManaged(false);

    const previousHydration = runtimeHydrationRef.current;
    if (previousHydration) {
      runtimeHydrationRef.current = null;
      previousHydration.resolve(null);
    }

    if (!accountScope) {
      setPinnedOrderState([]);
      setSections([]);
      return;
    }

    const scope = accountScope;
    const workspaceKey = sidebarWorkspaceStateKey(scope);
    activeScopeRef.current = scope;
    let cancelled = false;
    const hydrationMutationRevision = layoutMutationRevisionRef.current;
    const legacyPinStateManaged = readLegacyPinStateManaged(scope);
    pinStateManagedRef.current = legacyPinStateManaged;
    setPinStateManaged(legacyPinStateManaged);
    setPinnedOrderState(readPinnedOrder(scope));
    setSections(readAgentSidebarSections(scope));

    const runtimeState = new Promise<SidebarWorkspaceSnapshot | null>((resolve) => {
      runtimeHydrationRef.current = { scope, key: workspaceKey, resolve };
    });
    void client.readWorkspaceState(
      sidebarRequestId('agent-sidebar-state-read'),
      workspaceKey,
    ).catch(() => {
      const pending = runtimeHydrationRef.current;
      if (pending?.key !== workspaceKey) return;
      runtimeHydrationRef.current = null;
      pending.resolve(null);
    });

    void Promise.all([
      readPinnedOrderDurable(scope),
      readAgentSidebarSectionsDurable(scope),
      readAccountSidebarLayout(scope).catch(() => null),
      runtimeState,
    ]).then(([nativePinnedOrder, nativeSections, cloud, runtimeSnapshot]) => {
      if (cancelled || activeScopeRef.current !== scope) return;
      cloudSnapshotRef.current = cloud;
      if (layoutMutationRevisionRef.current === hydrationMutationRevision) {
        const runtimeManaged = runtimeSnapshot?.pinStateManaged ?? legacyPinStateManaged;
        const runtimePinnedOrder = runtimeSnapshot?.pinnedOrder ?? nativePinnedOrder;
        const runtimeSections = runtimeSnapshot?.sections ?? nativeSections;
        if (cloud) {
          const managed = cloud.layout.pinStateManaged || runtimeManaged;
          setPinnedOrderState(
            cloud.layout.pinStateManaged || !runtimeManaged
              ? cloud.layout.pinnedOrder
              : runtimePinnedOrder,
          );
          setSections(cloud.layout.sections);
          pinStateManagedRef.current = managed;
          setPinStateManaged(managed);
        } else {
          setPinnedOrderState(runtimePinnedOrder);
          setSections(runtimeSections);
          pinStateManagedRef.current = runtimeManaged;
          setPinStateManaged(runtimeManaged);
        }
      }
      // Any user mutation that raced hydration stays local and is written to
      // RuntimeStore + account CAS after scope activation instead of being
      // clobbered by a stale device snapshot.
      setLayoutScope(scope);
    });

    return () => {
      cancelled = true;
      if (activeScopeRef.current === scope) activeScopeRef.current = null;
      const pending = runtimeHydrationRef.current;
      if (pending?.key === workspaceKey) {
        runtimeHydrationRef.current = null;
        pending.resolve(null);
      }
    };
  }, [accountScope, client]);

  useEffect(() => {
    if (!accountScope || layoutScope !== accountScope) return;
    const scope = accountScope;
    let cancelled = false;

    const refreshRemoteLayout = async () => {
      try {
        const remote = await readAccountSidebarLayout(scope);
        if (cancelled || activeScopeRef.current !== scope || !remote) return;
        const previous = cloudSnapshotRef.current;
        if (previous && remote.etag === previous.etag && remote.serverRevision === previous.serverRevision) return;

        const merged = mergeAccountSidebarLayoutState(
          previous?.layout ?? null,
          {
            pinnedOrder: pinnedOrderRef.current,
            sections: sectionsRef.current,
          },
          remote.layout,
        );
        cloudSnapshotRef.current = remote;

        const managed = remote.layout.pinStateManaged || pinStateManagedRef.current;
        pinStateManagedRef.current = managed;
        setPinStateManaged(managed);

        if (!sameStringOrder(pinnedOrderRef.current, merged.pinnedOrder)) {
          setPinnedOrderState(merged.pinnedOrder);
        }
        if (!sameSections(sectionsRef.current, merged.sections)) {
          setSections(cloneSections(merged.sections));
        }
      } catch {
        // Cross-device convergence is best-effort. Rust RuntimeStore remains usable offline.
      }
    };

    const refreshWhenForegrounded = () => {
      if (document.visibilityState === 'visible') void refreshRemoteLayout();
    };
    const unsubscribeNative = subscribeNativeDesktopEvents({
      'account-state-changed': () => { void refreshRemoteLayout(); },
      'window-state': (payload) => {
        if ((payload as { focused?: boolean } | null)?.focused) void refreshRemoteLayout();
      },
    });
    window.addEventListener('focus', refreshWhenForegrounded);
    document.addEventListener('visibilitychange', refreshWhenForegrounded);
    void refreshRemoteLayout();
    return () => {
      cancelled = true;
      unsubscribeNative();
      window.removeEventListener('focus', refreshWhenForegrounded);
      document.removeEventListener('visibilitychange', refreshWhenForegrounded);
    };
  }, [accountScope, layoutScope]);

  useEffect(() => {
    if (!accountScope || layoutScope !== accountScope || !pinStateManaged) return;

    const scope = accountScope;
    const pinnedSnapshot = [...pinnedOrder];
    const sectionSnapshot = cloneSections(sections);
    const runtimeSnapshot = createSidebarWorkspaceSnapshot(
      scope,
      pinStateManaged,
      pinnedSnapshot,
      sectionSnapshot,
    );
    runtimeWriteChainRef.current = runtimeWriteChainRef.current
      .catch(() => undefined)
      .then(() => client.writeWorkspaceState(
        sidebarRequestId('agent-sidebar-state-write'),
        sidebarWorkspaceStateKey(scope),
        runtimeSnapshot,
      ))
      .catch(() => undefined);
    const baseSnapshot = cloudSnapshotRef.current;
    if (baseSnapshot?.layout.pinStateManaged && layoutMatchesSnapshot(pinnedSnapshot, sectionSnapshot, baseSnapshot)) {
      return;
    }

    // Serialize renderer writes and carry the last authoritative snapshot into
    // the account-level three-way merge. The committed merged result is then
    // projected back locally so concurrent remote-only edits appear here too.
    cloudWriteChainRef.current = cloudWriteChainRef.current
      .catch(() => undefined)
      .then(async () => {
        const currentBase = cloudSnapshotRef.current ?? baseSnapshot;
        const committed = await writeAccountSidebarLayout(
          scope,
          pinnedSnapshot,
          sectionSnapshot,
          { base: currentBase },
        );
        if (activeScopeRef.current !== scope) return;
        cloudSnapshotRef.current = committed;
        pinStateManagedRef.current = true;
        setPinStateManaged(true);
        if (!sameStringOrder(pinnedOrderRef.current, committed.layout.pinnedOrder)) {
          setPinnedOrderState(committed.layout.pinnedOrder);
        }
        if (!sameSections(sectionsRef.current, committed.layout.sections)) {
          setSections(cloneSections(committed.layout.sections));
        }
      })
      .catch(() => undefined);
  }, [accountScope, client, layoutScope, pinStateManaged, pinnedOrder, sections]);

  const adoptLegacyPinnedState = useCallback((pinnedKeys: readonly string[]) => {
    if (!accountScope || layoutScope !== accountScope || pinStateManagedRef.current) return;
    const legacyPinned = [...new Set(pinnedKeys.filter((key) => key.trim().length > 0))];
    updatePinnedOrder((current) => current.length ? [...current] : legacyPinned);
    pinStateManagedRef.current = true;
    setPinStateManaged(true);
  }, [accountScope, layoutScope, updatePinnedOrder]);

  const togglePin = useCallback((key: string) => {
    const normalized = key.trim();
    if (!normalized || !accountScope || layoutScope !== accountScope) return;
    if (!pinStateManagedRef.current) {
      pinStateManagedRef.current = true;
      setPinStateManaged(true);
      }
    updatePinnedOrder((current) => current.includes(normalized)
      ? current.filter((candidate) => candidate !== normalized)
      : [...current, normalized]);
  }, [accountScope, layoutScope, updatePinnedOrder]);

  const reorderPinned = useCallback((
    movedKey: string,
    targetKey: string,
    position: 'before' | 'after',
    pinnedKeys: readonly string[],
  ) => {
    if (movedKey === targetKey) return;
    updatePinnedOrder((current) => {
      const pinned = new Set(pinnedKeys);
      if (!pinned.has(movedKey) || !pinned.has(targetKey)) return [...current];
      const base = [
        ...current.filter((key) => pinned.has(key)),
        ...pinnedKeys.filter((key) => !current.includes(key)),
      ].filter((key) => key !== movedKey);
      const targetIndex = base.indexOf(targetKey);
      const insertionIndex = targetIndex < 0
        ? base.length
        : position === 'after'
          ? targetIndex + 1
          : targetIndex;
      base.splice(insertionIndex, 0, movedKey);
      return base;
    });
  }, [updatePinnedOrder]);

  const toggleSelection = useCallback((key: string) => {
    selectionAnchorRef.current = key;
    setSelectedKeys((current) => current.includes(key)
      ? current.filter((candidate) => candidate !== key)
      : [...current, key]);
  }, []);

  const rangeSelect = useCallback((key: string, orderedKeys: readonly string[]) => {
    const anchorKey = selectionAnchorRef.current;
    const anchorIndex = anchorKey ? orderedKeys.indexOf(anchorKey) : -1;
    const targetIndex = orderedKeys.indexOf(key);
    if (anchorIndex < 0 || targetIndex < 0) {
      selectionAnchorRef.current = key;
      setSelectedKeys([key]);
      return;
    }
    const start = Math.min(anchorIndex, targetIndex);
    const end = Math.max(anchorIndex, targetIndex);
    const range = orderedKeys.slice(start, end + 1);
    setSelectedKeys((current) => [...new Set([...current, ...range])]);
  }, []);

  const clearSelection = useCallback(() => {
    setSelectedKeys([]);
    selectionAnchorRef.current = null;
  }, []);

  const createSection = useCallback((name: string, items: readonly AgentSidebarStateItem[]) => {
    const sectionable = items.map((item) => item.key);
    layoutMutationRevisionRef.current += 1;
    setSections((current) => createAgentSidebarSection(current, name, sectionable).sections);
    clearSelection();
  }, [clearSelection]);

  const moveToSection = useCallback((items: readonly AgentSidebarStateItem[], sectionId: string) => {
    const keys = items.map((item) => item.key);
    if (!keys.length) return;
    layoutMutationRevisionRef.current += 1;
    setSections((current) => assignAgentsToSidebarSection(current, keys, sectionId));
    clearSelection();
  }, [clearSelection]);

  const renameSection = useCallback((sectionId: string, name: string) => {
    layoutMutationRevisionRef.current += 1;
    setSections((current) => renameAgentSidebarSection(current, sectionId, name));
  }, []);

  const removeSection = useCallback((sectionId: string) => {
    layoutMutationRevisionRef.current += 1;
    setSections((current) => removeAgentSidebarSection(current, sectionId));
  }, []);

  const toggleSection = useCallback((sectionId: string) => {
    layoutMutationRevisionRef.current += 1;
    setSections((current) => toggleAgentSidebarSection(current, sectionId));
  }, []);

  return {
    ready: Boolean(accountScope && layoutScope === accountScope),
    pinnedOrder,
    sections,
    selectedKeys,
    handle,
    adoptLegacyPinnedState,
    togglePin,
    reorderPinned,
    toggleSelection,
    rangeSelect,
    clearSelection,
    createSection,
    moveToSection,
    renameSection,
    removeSection,
    toggleSection,
  };
}
