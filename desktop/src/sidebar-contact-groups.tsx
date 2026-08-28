import React, { useEffect, useMemo, useState } from 'react';
import { Folder, Plus, Trash2, X } from 'lucide-react';
import { invokeNativeDesktop } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import { MAHAYANA_ACCOUNT_SESSION_RESET_EVENT } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';

export const SIDEBAR_CONTACT_GROUPS_KEY = 'fabushi.desktop.sidebar-contact-groups.v1';
export const SIDEBAR_UNASSIGNED_GROUP_ID = '__unassigned__';
export const SIDEBAR_PINNED_GROUP_ID = '__pinned__';

export type SidebarContactGroup = {
  id: string;
  name: string;
  peerKeys: string[];
  isCollapsed: boolean;
};

export type SidebarGroupPeer = {
  key: string;
  title: string;
  subtitle?: string;
  pinned?: boolean;
};

export type SidebarPeerGroup<T extends SidebarGroupPeer> = {
  id: string;
  name: string;
  isSynthetic: boolean;
  isCollapsed: boolean;
  peers: T[];
};

type StoredSidebarContactGroups = {
  version: 1;
  groups: SidebarContactGroup[];
};

type ManagerDraft = { id: string | null; name: string; peerKeys: Set<string> } | null;

function copyGroups(groups: readonly SidebarContactGroup[]): SidebarContactGroup[] {
  return groups.map((group) => ({ ...group, peerKeys: [...group.peerKeys] }));
}

export function normalizeSidebarContactGroups(value: unknown): SidebarContactGroup[] {
  const source = value && typeof value === 'object' && !Array.isArray(value)
    ? value as Partial<StoredSidebarContactGroups>
    : null;
  if (!source || source.version !== 1 || !Array.isArray(source.groups)) return [];
  const seenIds = new Set<string>();
  const claimedPeers = new Set<string>();
  const normalized: SidebarContactGroup[] = [];
  for (const candidate of source.groups) {
    if (!candidate || typeof candidate !== 'object') continue;
    const id = typeof candidate.id === 'string' ? candidate.id.trim() : '';
    const name = typeof candidate.name === 'string' ? candidate.name.trim() : '';
    if (!id || !name || id.startsWith('__') || seenIds.has(id) || !Array.isArray(candidate.peerKeys)) continue;
    seenIds.add(id);
    const peerKeys: string[] = [];
    for (const key of candidate.peerKeys) {
      if (typeof key !== 'string' || !key || claimedPeers.has(key)) continue;
      claimedPeers.add(key);
      peerKeys.push(key);
    }
    normalized.push({ id, name, peerKeys, isCollapsed: candidate.isCollapsed === true });
  }
  return normalized;
}

export function assignPeersToSidebarContactGroup(
  groups: readonly SidebarContactGroup[],
  groupId: string,
  peerKeys: readonly string[],
): SidebarContactGroup[] {
  const uniquePeerKeys = peerKeys.filter((key, index) => Boolean(key) && peerKeys.indexOf(key) === index);
  const moved = new Set(uniquePeerKeys);
  return groups.map((group) => ({
    ...group,
    peerKeys: group.id === groupId
      ? uniquePeerKeys
      : group.peerKeys.filter((key) => !moved.has(key)),
  }));
}

export function projectSidebarContactGroups<T extends SidebarGroupPeer>(
  peers: readonly T[],
  groups: readonly SidebarContactGroup[],
  searchActive = false,
): SidebarPeerGroup<T>[] {
  if (!groups.length) return [];
  const pinned = peers.filter((peer) => peer.pinned);
  const unpinned = peers.filter((peer) => !peer.pinned);
  const byKey = new Map(unpinned.map((peer) => [peer.key, peer]));
  const claimed = new Set<string>();
  const projected: SidebarPeerGroup<T>[] = [];

  if (pinned.length) {
    projected.push({ id: SIDEBAR_PINNED_GROUP_ID, name: '置顶', isSynthetic: true, isCollapsed: false, peers: pinned });
  }

  for (const group of groups) {
    const groupPeers: T[] = [];
    for (const key of group.peerKeys) {
      const peer = byKey.get(key);
      if (!peer || claimed.has(key)) continue;
      claimed.add(key);
      groupPeers.push(peer);
    }
    if (!searchActive || groupPeers.length) {
      projected.push({
        id: group.id,
        name: group.name,
        isSynthetic: false,
        isCollapsed: searchActive ? false : group.isCollapsed,
        peers: groupPeers,
      });
    }
  }

  const unassigned = unpinned.filter((peer) => !claimed.has(peer.key));
  if (unassigned.length) {
    projected.push({ id: SIDEBAR_UNASSIGNED_GROUP_ID, name: '未分组', isSynthetic: true, isCollapsed: false, peers: unassigned });
  }
  return projected;
}

function readLocalGroups(): SidebarContactGroup[] {
  if (typeof window === 'undefined') return [];
  try {
    return normalizeSidebarContactGroups(JSON.parse(window.localStorage.getItem(SIDEBAR_CONTACT_GROUPS_KEY) || 'null'));
  } catch {
    return [];
  }
}

function hasLocalGroupSnapshot(): boolean {
  if (typeof window === 'undefined') return false;
  try { return window.localStorage.getItem(SIDEBAR_CONTACT_GROUPS_KEY) !== null; } catch { return false; }
}

function persistGroups(groups: readonly SidebarContactGroup[]): void {
  if (typeof window === 'undefined') return;
  const value: StoredSidebarContactGroups = { version: 1, groups: copyGroups(groups) };
  try { window.localStorage.setItem(SIDEBAR_CONTACT_GROUPS_KEY, JSON.stringify(value)); } catch {}
  void invokeNativeDesktop<boolean>('writeClientPersistence', { key: SIDEBAR_CONTACT_GROUPS_KEY, value }).catch(() => {});
}

export function useSidebarContactGroups() {
  const [groups, setGroups] = useState<SidebarContactGroup[]>(() => readLocalGroups());
  const [persistenceHydrated, setPersistenceHydrated] = useState(false);
  const [managerOpen, setManagerOpen] = useState(false);

  useEffect(() => {
    if (typeof window === 'undefined') return;
    if (hasLocalGroupSnapshot()) {
      setPersistenceHydrated(true);
      return;
    }
    let disposed = false;
    void invokeNativeDesktop<unknown>('readClientPersistence', { key: SIDEBAR_CONTACT_GROUPS_KEY })
      .then((value) => {
        if (disposed) return;
        const restored = normalizeSidebarContactGroups(value);
        if (restored.length) setGroups(restored);
      })
      .catch(() => {})
      .finally(() => { if (!disposed) setPersistenceHydrated(true); });
    return () => { disposed = true; };
  }, []);

  useEffect(() => {
    if (persistenceHydrated) persistGroups(groups);
  }, [groups, persistenceHydrated]);

  useEffect(() => {
    if (typeof window === 'undefined') return;
    const reset = () => {
      setGroups([]);
      try { window.localStorage.removeItem(SIDEBAR_CONTACT_GROUPS_KEY); } catch {}
      void invokeNativeDesktop<boolean>('removeClientPersistence', { key: SIDEBAR_CONTACT_GROUPS_KEY }).catch(() => {});
    };
    window.addEventListener(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, reset);
    return () => window.removeEventListener(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, reset);
  }, []);

  return useMemo(() => ({
    groups,
    managerOpen,
    openManager: () => setManagerOpen(true),
    closeManager: () => setManagerOpen(false),
    create(name: string, peerKeys: readonly string[]) {
      const trimmed = name.trim();
      if (!trimmed) return;
      const id = `contact-group-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
      setGroups((current) => assignPeersToSidebarContactGroup(
        [{ id, name: trimmed, peerKeys: [], isCollapsed: false }, ...current],
        id,
        peerKeys,
      ));
    },
    update(id: string, name: string, peerKeys: readonly string[]) {
      const trimmed = name.trim();
      if (!trimmed) return;
      setGroups((current) => assignPeersToSidebarContactGroup(
        current.map((group) => group.id === id ? { ...group, name: trimmed } : group),
        id,
        peerKeys,
      ));
    },
    remove(id: string) {
      setGroups((current) => current.filter((group) => group.id !== id));
    },
    toggleCollapsed(id: string) {
      setGroups((current) => current.map((group) => group.id === id ? { ...group, isCollapsed: !group.isCollapsed } : group));
    },
    move(id: string, direction: -1 | 1) {
      setGroups((current) => {
        const index = current.findIndex((group) => group.id === id);
        const target = index + direction;
        if (index < 0 || target < 0 || target >= current.length) return current;
        const next = [...current];
        [next[index], next[target]] = [next[target], next[index]];
        return next;
      });
    },
  }), [groups, managerOpen]);
}

export function SidebarContactGroupManager({
  peers,
  groups,
  onCreate,
  onUpdate,
  onRemove,
  onMove,
  onClose,
}: {
  peers: SidebarGroupPeer[];
  groups: SidebarContactGroup[];
  onCreate: (name: string, peerKeys: readonly string[]) => void;
  onUpdate: (id: string, name: string, peerKeys: readonly string[]) => void;
  onRemove: (id: string) => void;
  onMove: (id: string, direction: -1 | 1) => void;
  onClose: () => void;
}) {
  const [draft, setDraft] = useState<ManagerDraft>(null);
  const assignedElsewhere = (key: string, currentId: string | null) => groups.some((group) => group.id !== currentId && group.peerKeys.includes(key));
  const startCreate = () => setDraft({ id: null, name: '', peerKeys: new Set() });
  const startEdit = (group: SidebarContactGroup) => setDraft({ id: group.id, name: group.name, peerKeys: new Set(group.peerKeys) });
  const save = () => {
    if (!draft?.name.trim()) return;
    if (draft.id) onUpdate(draft.id, draft.name, [...draft.peerKeys]);
    else onCreate(draft.name, [...draft.peerKeys]);
    setDraft(null);
  };

  return <div className="fabushi-contact-groups-backdrop" data-testid="contact-groups-manager" onMouseDown={onClose}>
    <section className="fabushi-contact-groups-dialog" role="dialog" aria-modal="true" aria-label="联系人分组" onMouseDown={(event) => event.stopPropagation()}>
      <header><div><Folder size={20} /><span><strong>联系人分组</strong><small>像 Grok Bot 一样把联系人整理到多个侧栏分组</small></span></div><button type="button" aria-label="关闭联系人分组" onClick={onClose}><X size={17} /></button></header>
      {!draft ? <>
        <div className="fabushi-contact-groups-toolbar"><button type="button" data-testid="contact-group-create" onClick={startCreate}><Plus size={15} />新建分组</button></div>
        <div className="fabushi-contact-groups-list">
          {groups.map((group, index) => <article key={group.id} data-testid={`contact-group-settings-${group.id}`}><div><strong>{group.name}</strong><small>{group.peerKeys.length} 个联系人</small></div><aside><button type="button" disabled={index === 0} onClick={() => onMove(group.id, -1)}>↑</button><button type="button" disabled={index === groups.length - 1} onClick={() => onMove(group.id, 1)}>↓</button><button type="button" onClick={() => startEdit(group)}>编辑</button><button type="button" aria-label={`删除 ${group.name}`} onClick={() => onRemove(group.id)}><Trash2 size={14} /></button></aside></article>)}
          {!groups.length ? <div className="fabushi-contact-groups-empty"><Folder size={34} /><strong>还没有联系人分组</strong><small>新建后，侧栏会按分组标题显示联系人。</small></div> : null}
        </div>
      </> : <div className="fabushi-contact-groups-editor">
        <label><span>分组名称</span><input autoFocus data-testid="contact-group-name" value={draft.name} onChange={(event) => setDraft((current) => current ? { ...current, name: event.target.value } : current)} placeholder="例如：运营部" /></label>
        <div className="fabushi-contact-groups-peer-picker"><strong>选择联系人</strong>{peers.map((peer) => {
          const checked = draft.peerKeys.has(peer.key);
          const elsewhere = assignedElsewhere(peer.key, draft.id);
          return <label key={peer.key} data-assigned-elsewhere={elsewhere || undefined}><input type="checkbox" checked={checked} onChange={(event) => setDraft((current) => {
            if (!current) return current;
            const peerKeys = new Set(current.peerKeys);
            if (event.target.checked) peerKeys.add(peer.key); else peerKeys.delete(peer.key);
            return { ...current, peerKeys };
          })} /><span><strong>{peer.title}</strong><small>{elsewhere && !checked ? '保存后会从其他分组移动到这里' : peer.subtitle || '联系人'}</small></span></label>;
        })}</div>
        <footer><button type="button" onClick={() => setDraft(null)}>取消</button><button type="button" data-testid="contact-group-save" disabled={!draft.name.trim()} onClick={save}>保存</button></footer>
      </div>}
    </section>
  </div>;
}
