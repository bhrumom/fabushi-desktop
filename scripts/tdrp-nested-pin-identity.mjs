// Validate exact, bidirectional nested source identity before recursive accounting.
// A nested pin is identified by its parent and mount path; its repository and
// commit must match the locked source, including GitLab children.
export function canonicalNestedRepository(url) {
  if (typeof url !== 'string') return null;
  const match = /^(?:https:\/\/|ssh:\/\/git@|git@)(github\.com|gitlab\.com)[/:]([A-Za-z0-9_.-]+)\/([A-Za-z0-9_.-]+?)(?:\.git)?\/?$/.exec(url.trim());
  if (!match) return null;
  const [, host, owner, name] = match;
  return host === 'github.com' ? `${owner}/${name}` : `gitlab.com/${owner}/${name}`;
}

export function assertNestedGitlinkPins(known, observed) {
  if (!Array.isArray(known) || !Array.isArray(observed)) {
    throw new Error('nested gitlink pin arrays are required');
  }
  const byMount = pins => {
    const result = new Map();
    for (const pin of pins) {
      if (!pin || !['parent', 'path', 'repository', 'commit'].every(field =>
        typeof pin[field] === 'string' && pin[field].trim() === pin[field] && pin[field].length > 0
      ) || !/^[0-9a-f]{40}$/.test(pin.commit)) {
        throw new Error('invalid nested gitlink identity');
      }
      const mount = JSON.stringify([pin.parent, pin.path]);
      if (result.has(mount)) throw new Error('duplicate nested gitlink mount: ' + mount);
      result.set(mount, pin);
    }
    return result;
  };
  const expected = byMount(known);
  const actual = byMount(observed);
  for (const [mount, pin] of expected) {
    const found = actual.get(mount);
    if (!found) throw new Error('missing nested gitlink: ' + mount);
    if (found.parent !== pin.parent || found.path !== pin.path ||
        found.repository !== pin.repository || found.commit !== pin.commit) {
      throw new Error('nested gitlink source identity mismatch: ' + mount);
    }
  }
  for (const mount of actual.keys()) {
    if (!expected.has(mount)) throw new Error('unexpected nested gitlink: ' + mount);
  }
  return { locked: expected.size, observed: actual.size };
}
