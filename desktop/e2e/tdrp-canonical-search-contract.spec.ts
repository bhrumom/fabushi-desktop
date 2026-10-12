import { expect, test } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  commandPaletteEntries,
  dedupeCommandPaletteAgents,
  type CommandPaletteAgent,
} from '../../frontend/src/production/command-palette-model';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

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


test('canonical fuzzy search preserves Unicode, RTL and separator semantics from mature source authority', () => {
  const rows: readonly CommandPaletteAgent[] = [
    { id: 'human:house', name: 'Häuser', isGroup: false, isHidden: false },
    { id: 'human:urdu', name: 'اردو', isGroup: false, isHidden: false },
    { id: 'human:file', name: 'the_file', isGroup: false, isHidden: false },
    { id: 'human:short', name: 'Nam', isGroup: false, isHidden: false },
  ];

  const search = (query: string) => commandPaletteEntries({
    agents: rows,
    commands: [],
    query,
    tab: 'agents',
  }).flatMap((entry) => entry.kind === 'agent' ? [entry.agent.id] : []);

  expect(search('hau')).toEqual(['human:house']);
  expect(search('ارو')).toEqual(['human:urdu']);
  expect(search('tf')).toEqual(['human:file']);
  expect(search('Name')).not.toContain('human:short');
});

test('shipping canonical selection crosses renderer gateway into the one durable Recent Peers owner', async () => {
  const [renderer, bridgeContract, sessionGateway, productionSession, messagingService, coordinatorRpc] =
    await Promise.all([
      readFile(path.join(repoRoot, 'frontend/src/production/ProductionRenderer.tsx'), 'utf8'),
      readFile(path.join(repoRoot, 'frontend/src/recovered/contracts/desktop-bridge.ts'), 'utf8'),
      readFile(path.join(repoRoot, 'source/host/src/extensions/session/gateway.rs'), 'utf8'),
      readFile(path.join(repoRoot, 'source/host/src/extensions/session/production.rs'), 'utf8'),
      readFile(path.join(repoRoot, 'native/mahayana-messaging/src/service.rs'), 'utf8'),
      readFile(path.join(repoRoot, 'source/shared/rpc/coordinator.ts'), 'utf8'),
    ]);

  expect(bridgeContract).toContain('createCanonicalRecentParticipantSelectionRequest');
  expect(renderer).toContain('"noteRecentParticipantSuggestion"');
  expect(renderer).toContain('createCanonicalRecentParticipantSelectionRequest(agentId)');
  expect(renderer).toContain('accountScopeGenerationRef.current === accountScopeGeneration');
  expect(renderer).toContain('openAgentRequestGenerationRef.current === requestGeneration');
  expect(sessionGateway).toContain('"noteRecentParticipantSuggestion"');
  expect(productionSession).toContain('service.note_recent_participant_selection(');
  expect(productionSession).toContain('self.agents_root.join("_messaging")');
  expect(messagingService).toContain('pub fn note_recent_participant_selection(');
  expect(messagingService).toContain('pub fn note_recent_participant_suggestion(');
  expect(messagingService.match(/self\.engine = previous_engine/g)?.length).toBeGreaterThanOrEqual(2);
  expect(coordinatorRpc).toContain('noteRecentParticipantSuggestion: { args: "object", reply: "record" }');

  const openAgentStart = renderer.indexOf('const openAgent = useCallback');
  const openAgentEnd = renderer.indexOf('const openGroupMemberChat');
  expect(openAgentStart).toBeGreaterThanOrEqual(0);
  expect(openAgentEnd).toBeGreaterThan(openAgentStart);
  const selectionBranch = renderer.slice(openAgentStart, openAgentEnd);
  expect(selectionBranch).toContain('} catch {');
  expect(selectionBranch.split('noteRecentParticipantSelection(')).toHaveLength(3);
});

