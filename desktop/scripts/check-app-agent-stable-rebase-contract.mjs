import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const bridgeSource = await readFile(new URL('../src/app-agent-surface.ts', import.meta.url), 'utf8');
const domSource = await readFile(new URL('../../frontend/apps/web/src/lib/app-agent-surface/dom-agent-surface.ts', import.meta.url), 'utf8');

assert.match(
  domSource,
  /if \(requestedGeneration !== this\.generation\) \{\s*throw new Error\(`stale_app_surface_generation:/u,
  'base DOM App Surface must keep exact generation fail-closed semantics',
);
assert.match(
  bridgeSource,
  /operation !== 'action' \|\| !isStaleGeneration\(initialError\)/u,
  'rebasing must apply only to stale actions',
);
assert.match(
  bridgeSource,
  /if \(operation === 'snapshot'\) rememberStableActionLease\(leases, result\)/u,
  'a stale action may only rebase from a previously observed snapshot lease',
);
assert.match(
  bridgeSource,
  /requestedLease = requestedGeneration == null \? null : leases\.get\(requestedGeneration\)/u,
  'requested generation must map to a recorded lease',
);
assert.match(
  bridgeSource,
  /\(ref && ref !== `agent:\$\{agentId\}`\)/u,
  'volatile positional refs must never be rebound across generations',
);
assert.match(
  bridgeSource,
  /String\(current\.route \?\? ''\) !== requestedLease\.route[\s\S]*String\(current\.screen \?\? ''\) !== requestedLease\.screen/u,
  'route and screen changes must remain fail-closed',
);
assert.match(
  bridgeSource,
  /currentMatches\.length !== 1[\s\S]*stableTargetFingerprint\(currentMatches\[0\]\) !== requestedFingerprint/u,
  'stable target identity and semantics must remain unchanged before rebasing',
);
assert.match(
  bridgeSource,
  /const MAX_STABLE_ACTION_LEASES = 32/u,
  'snapshot leases must remain bounded',
);
assert.match(
  bridgeSource,
  /const MAX_STABLE_REBASE_ATTEMPTS = 3/u,
  'stale-generation retries must remain bounded',
);

console.log('macOS stable App target rebase contract: PASS');
