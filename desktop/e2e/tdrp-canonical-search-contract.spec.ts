import { expect, test } from '@playwright/test';
import {
  commandPaletteEntries,
  dedupeCommandPaletteAgents,
  type CommandPaletteAgent,
} from '../../frontend/src/production/command-palette-model';

const replacedRoster: readonly CommandPaletteAgent[] = [
  { id: 'human:ada', name: 'Ada (stale)', title: 'Old row', isGroup: false, isHidden: false },
  { id: 'human:grace', name: 'Grace', isGroup: false, isHidden: false },
  { id: 'human:ada', name: 'Ada', title: 'Current row', isGroup: false, isHidden: false },
];

test('canonical participant identity replaces stale rows without duplicate search results', () => {
  const deduped = dedupeCommandPaletteAgents(replacedRoster);
  expect(deduped).toHaveLength(2);
  expect(deduped.map((agent) => agent.id)).toEqual(['human:ada', 'human:grace']);
  expect(deduped[0]?.name).toBe('Ada');
  expect(deduped[0]?.title).toBe('Current row');

  const all = commandPaletteEntries({
    agents: replacedRoster,
    commands: [],
    query: 'ada',
    tab: 'all',
  });
  expect(all.filter((entry) => entry.kind === 'agent')).toHaveLength(1);
  expect(all[0]?.kind).toBe('agent');
  if (all[0]?.kind === 'agent') expect(all[0].agent.name).toBe('Ada');

  const participants = commandPaletteEntries({
    agents: replacedRoster,
    commands: [],
    query: 'ada',
    tab: 'agents',
  });
  expect(participants).toHaveLength(1);
  expect(participants[0]?.kind).toBe('agent');
});

test('latest replacement controls hidden/visible projection without parallel participant roots', () => {
  const rows: readonly CommandPaletteAgent[] = [
    { id: 'human:lin', name: 'Lin', isGroup: false, isHidden: true },
    { id: 'human:lin', name: 'Lin', isGroup: false, isHidden: false },
  ];
  const entries = commandPaletteEntries({
    agents: rows,
    commands: [],
    query: '',
    tab: 'all',
  });
  expect(entries).toHaveLength(1);
  expect(entries[0]?.kind).toBe('agent');
  if (entries[0]?.kind === 'agent') expect(entries[0].isHidden).not.toBe(true);
});
