import { useCallback, useEffect, useRef, useState } from 'react';
import { invokeNativeDesktop, subscribeNativeDesktopEvents } from '../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import {
  assignAgentsToSidebarSection,
  createAgentSidebarSection,
  persistAgentSidebarSections,
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

function readPinStateManaged(accountScope: string): boolean {
  if (typeof window === 'undefined') return false;
  try {
    return window.localStorage.getItem(pinStateManagedKey(accountScope)) === '1';
  } catch {
    return false;
  }
}

function persistPinStateManaged(accountScope: string): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(pinStateManagedKey(accountScope), '1');
  } catch {
    // The account CAS object remains authoritative when localStorage is unavailable.
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

function persistPinnedOrder(accountScope: string, order: readonly string[]): void {
  const key = pinnedOrderKey(accountScope);
  const value = normalizePinnedOrder(order);
  if (typeof window !== 'undefined') {
    try {
      window.localStorage.setItem(key, JSON.stringify(value));
    } catch {
      // Native persistence remains the local durable mirror.
    }
  }
  void invokeNativeDesktop<boolean>('writeClientPersistence', {
    key,
    value,
  }).catch(() => {});
}

/**
 * Agent-owned sidebar state controller.
 *
 * Pinned ordering and custom sections are one account workspace document.
 * LocalStorage/Native Host are offline mirrors only; authenticated devices
 * converge through the account Agent-store CAS object (etag + revision).
 */
export function useAgentSidebarController(
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

  useEffect(() => {
    pinnedOrderRef.current = [...pinnedOrder];
  }, [pinnedOrder]);

  useEffect(() => {
    sectionsRef.current = cloneSections(sections);
  }, [sections]);

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
    if (!accountScope) {
      setPinnedOrderState([]);
      setSections([]);
      return;
    }

    const scope = accountScope;
    activeScopeRef.current = scope;
    let cancelled = false;
    const hydrationMutationRevision = layoutMutationRevisionRef.current;
    const localPinStateManaged = readPinStateManaged(scope);
    pinStateManagedRef.current = localPinStateManaged;
    setPinStateManaged(localPinStateManaged);
    setPinnedOrderState(readPinnedOrder(scope));
    setSections(readAgentSidebarSections(scope));

    void Promise.all([
      readPinnedOrderDurable(scope),
      readAgentSidebarSectionsDurable(scope),
      readAccountSidebarLayout(scope).catch(() => null),
    ]).then(([nativePinnedOrder, nativeSections, cloud]) => {
      if (cancelled || activeScopeRef.current !== scope) return;
      cloudSnapshotRef.current = cloud;
      if (layoutMutationRevisionRef.current === hydrationMutationRevision) {
        if (cloud) {
          // A managed cloud document is authoritative. For a pre-migration
          // cloud document, preserve a locally managed pin state so a failed
          // CAS retry cannot resurrect a legacy Messenger pin on next launch.
          const managed = cloud.layout.pinStateManaged || localPinStateManaged;
          setPinnedOrderState(
            cloud.layout.pinStateManaged || !localPinStateManaged
              ? cloud.layout.pinnedOrder
              : nativePinnedOrder,
          );
          setSections(cloud.layout.sections);
          pinStateManagedRef.current = managed;
          setPinStateManaged(managed);
          if (managed) persistPinStateManaged(scope);
        } else {
          setPinnedOrderState(nativePinnedOrder);
          setSections(nativeSections);
          pinStateManagedRef.current = localPinStateManaged;
          setPinStateManaged(localPinStateManaged);
        }
      }
      // Any user mutation that raced hydration stays local and is written via
      // CAS after scope activation instead of being clobbered by an old device.
      setLayoutScope(scope);
    });

    return () => {
      cancelled = true;
      if (activeScopeRef.current === scope) activeScopeRef.current = null;
    };
  }, [accountScope]);

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
        if (managed) persistPinStateManaged(scope);

        if (!sameStringOrder(pinnedOrderRef.current, merged.pinnedOrder)) {
          setPinnedOrderState(merged.pinnedOrder);
        }
        if (!sameSections(sectionsRef.current, merged.sections)) {
          setSections(cloneSections(merged.sections));
        }
      } catch {
        // Cloud polling is best-effort. Local/native mirrors remain usable offline.
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
    persistPinStateManaged(accountScope);
    persistPinnedOrder(accountScope, pinnedOrder);
    persistAgentSidebarSections(accountScope, sections);

    const scope = accountScope;
    const pinnedSnapshot = [...pinnedOrder];
    const sectionSnapshot = cloneSections(sections);
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
  }, [accountScope, layoutScope, pinStateManaged, pinnedOrder, sections]);

  const adoptLegacyPinnedState = useCallback((pinnedKeys: readonly string[]) => {
    if (!accountScope || layoutScope !== accountScope || pinStateManagedRef.current) return;
    const legacyPinned = [...new Set(pinnedKeys.filter((key) => key.trim().length > 0))];
    updatePinnedOrder((current) => current.length ? [...current] : legacyPinned);
    pinStateManagedRef.current = true;
    setPinStateManaged(true);
    persistPinStateManaged(accountScope);
  }, [accountScope, layoutScope, updatePinnedOrder]);

  const togglePin = useCallback((key: string) => {
    const normalized = key.trim();
    if (!normalized || !accountScope || layoutScope !== accountScope) return;
    if (!pinStateManagedRef.current) {
      pinStateManagedRef.current = true;
      setPinStateManaged(true);
      persistPinStateManaged(accountScope);
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
