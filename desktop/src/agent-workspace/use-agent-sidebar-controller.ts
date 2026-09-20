import { useCallback, useEffect, useRef, useState } from 'react';
import { invokeNativeDesktop } from '../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
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
  readAccountSidebarLayout,
  writeAccountSidebarLayout,
} from './account-sidebar-layout';

const legacyPinnedOrderKey = 'fabushi.desktop.grok-pinned-order.v1';

function pinnedOrderKey(accountScope: string): string {
  return `fabushi.desktop.agent-pinned-order.v2.${encodeURIComponent(accountScope)}`;
}

export interface AgentSidebarStateItem {
  readonly key: string;
  readonly pinned: boolean;
}

export interface AgentSidebarController {
  readonly pinnedOrder: readonly string[];
  readonly sections: readonly AgentSidebarSection[];
  readonly selectedKeys: readonly string[];
  reconcilePinnedOrder(pinnedKeys: readonly string[]): void;
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
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const selectionAnchorRef = useRef<string | null>(null);
  const layoutMutationRevisionRef = useRef(0);
  const cloudWriteChainRef = useRef<Promise<unknown>>(Promise.resolve());

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
    if (!accountScope) {
      setPinnedOrderState([]);
      setSections([]);
      return;
    }

    const scope = accountScope;
    let cancelled = false;
    const hydrationMutationRevision = layoutMutationRevisionRef.current;
    setPinnedOrderState(readPinnedOrder(scope));
    setSections(readAgentSidebarSections(scope));

    void Promise.all([
      readPinnedOrderDurable(scope),
      readAgentSidebarSectionsDurable(scope),
      readAccountSidebarLayout(scope).catch(() => null),
    ]).then(([nativePinnedOrder, nativeSections, cloud]) => {
      if (cancelled) return;
      if (layoutMutationRevisionRef.current === hydrationMutationRevision) {
        if (cloud) {
          setPinnedOrderState(cloud.layout.pinnedOrder);
          setSections(cloud.layout.sections);
        } else {
          setPinnedOrderState(nativePinnedOrder);
          setSections(nativeSections);
        }
      }
      // Any user mutation that raced hydration stays local and is written via
      // CAS after scope activation instead of being clobbered by an old device.
      setLayoutScope(scope);
    });

    return () => { cancelled = true; };
  }, [accountScope]);

  useEffect(() => {
    if (!accountScope || layoutScope !== accountScope) return;
    persistPinnedOrder(accountScope, pinnedOrder);
    persistAgentSidebarSections(accountScope, sections);

    const scope = accountScope;
    const pinnedSnapshot = [...pinnedOrder];
    const sectionSnapshot = sections.map((section) => ({
      ...section,
      agentKeys: [...section.agentKeys],
    }));
    // Serialize writes from this renderer. Each cloud write still performs its
    // own server-side CAS read/write cycle so a second device cannot be blindly
    // overwritten with a stale etag.
    cloudWriteChainRef.current = cloudWriteChainRef.current
      .catch(() => undefined)
      .then(() => writeAccountSidebarLayout(scope, pinnedSnapshot, sectionSnapshot))
      .catch(() => undefined);
  }, [accountScope, layoutScope, pinnedOrder, sections]);

  const reconcilePinnedOrder = useCallback((pinnedKeys: readonly string[]) => {
    updatePinnedOrder((current) => {
      const pinned = new Set(pinnedKeys);
      return [
        ...current.filter((key) => pinned.has(key)),
        ...pinnedKeys.filter((key) => !current.includes(key)),
      ];
    });
  }, [updatePinnedOrder]);

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
    pinnedOrder,
    sections,
    selectedKeys,
    reconcilePinnedOrder,
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
