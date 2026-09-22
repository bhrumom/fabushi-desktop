const MAX_MEMORY_LENGTH = 500;

export function normalizeMemoryContent(raw: string): string {
  return raw.replace(/\s+/gu, " ").trim().slice(0, MAX_MEMORY_LENGTH);
}

export function memoryDedupeKey(raw: string): string {
  return normalizeMemoryContent(raw).normalize("NFKC").toLocaleLowerCase();
}

function mix32(value: number): number {
  value = Math.imul(value ^ (value >>> 16), 0x21f0aaad);
  value = Math.imul(value ^ (value >>> 15), 0x735a2d97);
  return (value ^ (value >>> 15)) >>> 0;
}

function hash32(text: string, seed: number): number {
  let hash = seed >>> 0;
  for (const char of text) {
    hash ^= char.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 0x01000193);
    hash = mix32(hash);
  }
  return hash >>> 0;
}

export function makeMemoryId(raw: string): string {
  const key = memoryDedupeKey(raw);
  const left = hash32(key, 0x811c9dc5).toString(16).padStart(8, "0");
  const right = hash32(key, 0x9e3779b9).toString(16).padStart(8, "0");
  return `${left}${right}`;
}
