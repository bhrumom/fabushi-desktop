import { createHash, randomUUID } from "node:crypto";
import { mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

interface HumanIdentityFile {
  readonly schemaVersion: 1;
  readonly identities: Readonly<Record<string, string>>;
}

function accountScope(slot: string): string {
  return createHash("sha256").update(slot).digest("hex");
}

export interface ProductionHumanIdentityStore {
  resolve(slot: string): string;
}

export function createProductionHumanIdentityStore(userDataDir: string): ProductionHumanIdentityStore {
  const path = join(userDataDir, "fabushi-human-identities.json");
  const read = (): HumanIdentityFile => {
    let raw: string;
    try {
      raw = readFileSync(path, "utf8");
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") {
        return { schemaVersion: 1, identities: {} };
      }
      throw error;
    }

    let parsed: Partial<HumanIdentityFile>;
    try {
      parsed = JSON.parse(raw) as Partial<HumanIdentityFile>;
    } catch {
      throw new Error("Fabushi Human identity store is corrupt.");
    }
    if (parsed.schemaVersion !== 1 || parsed.identities == null || typeof parsed.identities !== "object" || Array.isArray(parsed.identities)) {
      throw new Error("Fabushi Human identity store is corrupt.");
    }
    for (const [scope, identity] of Object.entries(parsed.identities)) {
      if (scope.length === 0 || typeof identity !== "string" || identity.trim().length === 0) {
        throw new Error("Fabushi Human identity store is corrupt.");
      }
    }
    return { schemaVersion: 1, identities: { ...parsed.identities } };
  };

  return {
    resolve(slot) {
      const normalized = slot.trim();
      if (normalized.length === 0) {
        throw new Error("Fabushi Human identity requires an account slot.");
      }
      const scope = accountScope(normalized);
      const current = read();
      const existing = current.identities[scope]?.trim();
      if (existing) return existing;

      const id = randomUUID();
      const next: HumanIdentityFile = {
        schemaVersion: 1,
        identities: { ...current.identities, [scope]: id },
      };
      mkdirSync(dirname(path), { recursive: true });
      const temporary = `${path}.${randomUUID()}.tmp`;
      writeFileSync(temporary, JSON.stringify(next), { encoding: "utf8", mode: 0o600 });
      renameSync(temporary, path);
      return id;
    },
  };
}
