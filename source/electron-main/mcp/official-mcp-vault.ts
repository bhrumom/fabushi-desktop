import { join } from "node:path";
import { SandUserSecretsStore } from "../secrets/user-secrets-store.js";
import type { OfficialMcpPorts } from "./official-mcp-service.js";

/** Dedicated vault: never included in user/Box secrets export or renderer IPC. */
export function createOfficialMcpVaultPorts(
  root: string,
  getScope: () => Promise<string>,
): OfficialMcpPorts {
  const stores = new Map<string, SandUserSecretsStore>();
  const store = (scope: string) => {
    if (!/^[a-f0-9]{64}$/.test(scope)) throw new Error("Invalid account scope.");
    let value = stores.get(scope);
    if (!value) {
      value = new SandUserSecretsStore(join(root, "official-mcp", `${scope}.json`), () => scope);
      stores.set(scope, value);
    }
    return value;
  };
  return {
    getScope,
    read: scope => store(scope).reveal("OFFICIAL_MCP_INSTALLS"),
    write: (scope, value) => store(scope).upsert({ OFFICIAL_MCP_INSTALLS: value }),
    fetch: (input, init) => fetch(input, init),
  };
}
