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

const pinnedOrderKey = 'fabushi.desktop.grok-pinned-order.v1';

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

function readPinnedOrder(): string[] {
  if (typeof window === 'undefined') return [];
  try {
    const value = JSON.parse(window.localStorage.getItem(pinnedOrderKey) || '[]');
    return Array.isArray(value)
      ? value.filter((entry): entry is string => typeof entry === 'string' && entry.length > 0)
      : [];
  } catch {
    return [];
  }
}

function persistPinnedOrder(order: readonly string[]): void {
  if (typeof window === 'undefined') return;
  const value = [...order];
  try {
    window.localStorage.setItem(pinnedOrderKey, JSON.stringify(value));
  } catch {
    // Native persistence below remains the durable mirror.
  }
  void invokeNativeDesktop<boolean>('writeClientPersistence', {
    key: pinnedOrderKey,
    value,
  }).catch(() => {});
}

/**
 * Agent-owned sidebar state controller.
 *
 * Pinned ordering, custom sections and multi-selection are durable Agent
 * workspace concerns, not Messenger navigation state. The shell may supply
 * prompts/confirmations, but it no longer owns these state machines.
 */
export function useAgentSidebarController(
  accountScope: string | null | undefined,
): AgentSidebarController {
  const [pinnedOrder, setPinnedOrderState] = useState<string[]>(readPinnedOrder);
  const [sections, setSections] = useState<AgentSidebarSection[]>([]);
  const [sectionsScope, setSectionsScope] = useState<string | null>(null);
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const selectionAnchorRef = useRef<string | null>(null);

  const updatePinnedOrder = useCallback((build: (current: readonly string[]) => string[]) => {
    setPinnedOrderState((current) => {
      const next = build(current);
      if (next.length === current.length && next.every((key, index) => key === current[index])) return current;
      persistPinnedOrder(next);
      return next;
    });
  }, []);

  useEffect(() => {
    let cancelled = false;
    void invokeNativeDesktop<unknown>('readClientPersistence', { key: pinnedOrderKey }).then((value) => {
      if (cancelled || !Array.isArray(value)) return;
      const nativeOrder = value.filter((entry): entry is string =>
        typeof entry === 'string' && entry.length > 0,
      );
      if (!nativeOrder.length) return;
      try {
        window.localStorage.setItem(pinnedOrderKey, JSON.stringify(nativeOrder));
      } catch {
        // Native persistence remains authoritative.
      }
      setPinnedOrderState(nativeOrder);
    }).catch(() => {});
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    setSelectedKeys([]);
    selectionAnchorRef.current = null;
    if (!accountScope) {
      setSections([]);
      setSectionsScope(null);
      return;
    }

    let cancelled = false;
    setSections(readAgentSidebarSections(accountScope));
    setSectionsScope(null);
    void readAgentSidebarSectionsDurable(accountScope).then((nextSections) => {
      if (cancelled) return;
      setSections(nextSections);
      setSectionsScope(accountScope);
    });
    return () => { cancelled = true; };
  }, [accountScope]);

  useEffect(() => {
    if (!accountScope || sectionsScope !== accountScope) return;
    persistAgentSidebarSections(accountScope, sections);
  }, [accountScope, sections, sectionsScope]);

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
    const sectionable = items.filter((item) => !item.pinned).map((item) => item.key);
    setSections((current) => createAgentSidebarSection(current, name, sectionable).sections);
    clearSelection();
  }, [clearSelection]);

  const moveToSection = useCallback((items: readonly AgentSidebarStateItem[], sectionId: string) => {
    const keys = items.filter((item) => !item.pinned).map((item) => item.key);
    if (!keys.length) return;
    setSections((current) => assignAgentsToSidebarSection(current, keys, sectionId));
    clearSelection();
  }, [clearSelection]);

  const renameSection = useCallback((sectionId: string, name: string) => {
    setSections((current) => renameAgentSidebarSection(current, sectionId, name));
  }, []);

  const removeSection = useCallback((sectionId: string) => {
    setSections((current) => removeAgentSidebarSection(current, sectionId));
  }, []);

  const toggleSection = useCallback((sectionId: string) => {
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
