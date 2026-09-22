import {
  deleteAccountAgentStoreObject,
  listAccountAgentStore,
  readAccountAgentStoreObject,
  writeAccountAgentStoreObject,
} from '../account-sync-client';
import { FabuAgentStore } from '../fabu-runtime/agent-store';

const stores = new Map<string, FabuAgentStore>();

function storeFor(agentId: string): FabuAgentStore {
  const existing = stores.get(agentId);
  if (existing) return existing;
  const store = new FabuAgentStore(agentId, {
    list: (id, prefix) => listAccountAgentStore(id, prefix),
    read: (id, path) => readAccountAgentStoreObject(id, path),
    write: (id, path, dataBase64, options) => writeAccountAgentStoreObject(id, path, dataBase64, options),
    delete: (id, path, baseEtag) => deleteAccountAgentStoreObject(id, path, baseEtag),
  });
  stores.set(agentId, store);
  return store;
}

export async function mirrorAgentStoreObject(agentId: string, path: string, value: unknown): Promise<void> {
  if (!agentId.trim()) return;
  const store = storeFor(agentId);
  await store.writeJson(path, value);
  await store.checkpointRoot();
}

export async function removeAgentStoreObject(agentId: string, path: string): Promise<void> {
  if (!agentId.trim()) return;
  const store = storeFor(agentId);
  await store.delete(path);
  await store.checkpointRoot();
}
