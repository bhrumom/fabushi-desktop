import { expect, test } from '@playwright/test';
import {
  SIDEBAR_PINNED_GROUP_ID,
  SIDEBAR_UNASSIGNED_GROUP_ID,
  assignPeersToSidebarContactGroup,
  normalizeSidebarContactGroups,
  projectSidebarContactGroups,
  type SidebarContactGroup,
} from '../src/sidebar-contact-groups';

const peers = [
  { key: 'peer:a', title: 'A', pinned: true },
  { key: 'peer:b', title: 'B', pinned: false },
  { key: 'peer:c', title: 'C', pinned: false },
  { key: 'peer:d', title: 'D', pinned: false },
];

const groups: SidebarContactGroup[] = [
  { id: 'ops', name: '运营部', peerKeys: ['peer:b'], isCollapsed: false },
  { id: 'qa', name: '测试部', peerKeys: ['peer:c'], isCollapsed: true },
];

test('projects pinned, named and unassigned sections in stable order', () => {
  const projected = projectSidebarContactGroups(peers, groups);
  expect(projected.map((group) => group.id)).toEqual([
    SIDEBAR_PINNED_GROUP_ID,
    'ops',
    'qa',
    SIDEBAR_UNASSIGNED_GROUP_ID,
  ]);
  expect(projected[0].peers.map((peer) => peer.key)).toEqual(['peer:a']);
  expect(projected[1].peers.map((peer) => peer.key)).toEqual(['peer:b']);
  expect(projected[2].isCollapsed).toBe(true);
  expect(projected[3].peers.map((peer) => peer.key)).toEqual(['peer:d']);
});

test('assignment is exclusive and editing can remove an existing member', () => {
  const moved = assignPeersToSidebarContactGroup(groups, 'qa', ['peer:b']);
  expect(moved.find((group) => group.id === 'ops')?.peerKeys).toEqual([]);
  expect(moved.find((group) => group.id === 'qa')?.peerKeys).toEqual(['peer:b']);

  const removed = assignPeersToSidebarContactGroup(moved, 'qa', []);
  expect(removed.find((group) => group.id === 'qa')?.peerKeys).toEqual([]);
});

test('normalization rejects duplicate ownership and reserved section ids', () => {
  const normalized = normalizeSidebarContactGroups({
    version: 1,
    groups: [
      { id: 'ops', name: '运营部', peerKeys: ['peer:b', 'peer:b'], isCollapsed: false },
      { id: 'qa', name: '测试部', peerKeys: ['peer:b', 'peer:c'], isCollapsed: true },
      { id: '__pinned__', name: '伪造置顶', peerKeys: ['peer:d'], isCollapsed: false },
    ],
  });
  expect(normalized).toEqual([
    { id: 'ops', name: '运营部', peerKeys: ['peer:b'], isCollapsed: false },
    { id: 'qa', name: '测试部', peerKeys: ['peer:c'], isCollapsed: true },
  ]);
});

test('search expands matching collapsed groups and omits empty named groups', () => {
  const projected = projectSidebarContactGroups(
    peers.filter((peer) => peer.key === 'peer:c'),
    groups,
    true,
  );
  expect(projected.map((group) => group.id)).toEqual(['qa']);
  expect(projected[0].isCollapsed).toBe(false);
});
