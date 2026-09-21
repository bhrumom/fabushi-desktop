export type FabAvatarIdentityAlias = { alias: string; canonical: string };

const aliases = new Map<string, string>();
const listeners = new Set<() => void>();
let revision = 0;

export function registerFabAvatarIdentityAliases(entries: readonly FabAvatarIdentityAlias[]): void {
  let changed = false;
  for (const entry of entries) {
    if (!entry.alias || !entry.canonical || aliases.get(entry.alias) === entry.canonical) continue;
    aliases.set(entry.alias, entry.canonical);
    changed = true;
  }
  if (!changed) return;
  revision += 1;
  listeners.forEach((listener) => listener());
}

export function canonicalFabAvatarIdentity(identity: string): string {
  let current = identity;
  const visited = new Set<string>();
  while (!visited.has(current)) {
    visited.add(current);
    const next = aliases.get(current);
    if (!next) break;
    current = next;
  }
  return current;
}

export function subscribeFabAvatarIdentity(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function fabAvatarIdentityRevision(): number {
  return revision;
}
