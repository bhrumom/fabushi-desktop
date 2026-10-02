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
    try {
      const parsed = JSON.parse(readFileSync(path, "utf8")) as Partial<HumanIdentityFile>;
      if (parsed.schemaVersion === 1 && parsed.identities != null && typeof parsed.identities === "object") {
        return { schemaVersion: 1, identities: { ...parsed.identities } };
      }
    } catch {
      // Missing or corrupt metadata is recovered by minting a scoped identity.
    }
    return { schemaVersion: 1, identities: {} };
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
