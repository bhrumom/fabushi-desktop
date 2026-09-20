import type { FabuAgentProfile, FabuAgentSettings } from './agent-domain';

export const FABU_AGENT_ROOT_PATH = 'root.json';
export const FABU_AGENT_PROFILE_PATH = 'profile.json';
export const FABU_AGENT_SETTINGS_PATH = 'settings.json';
export const FABU_AGENT_MEMORY_INDEX_PATH = 'memory/index.json';
export const FABU_AGENT_WORKFLOW_INDEX_PATH = 'workflows/index.json';
export const FABU_AGENT_ATTACHMENT_INDEX_PATH = 'attachments/index.json';
export const FABU_AGENT_RUNTIME_CHECKPOINT_PATH = 'runtime/checkpoint.json';

export function fabuAgentConversationTranscriptPath(conversationId: string): string {
  return `conversations/${encodeURIComponent(conversationId.trim()).replace(/%2F/gi, '_')}/transcript.json`;
}

export function fabuAgentAutomationPath(automationId: string): string {
  return `automations/${encodeURIComponent(automationId.trim()).replace(/%2F/gi, '_')}/automation.json`;
}

export interface FabuAgentStoreObject {
  path: string;
  blobId?: string;
  etag?: string;
  revision?: number;
  dataBase64?: string;
}

export interface FabuAgentRootEntry {
  path: string;
  blobId?: string;
  etag?: string;
  revision?: number;
}

export interface FabuAgentRootManifest {
  schemaVersion: 1;
  agentId: string;
  updatedAtMs: number;
  files: FabuAgentRootEntry[];
}

export interface FabuAgentStoreTransport {
  list(agentId: string, prefix?: string): Promise<{ files?: FabuAgentStoreObject[] }>;
  read(agentId: string, path: string): Promise<FabuAgentStoreObject>;
  write(
    agentId: string,
    path: string,
    dataBase64: string,
    options?: { baseEtag?: string; expectAbsent?: boolean },
  ): Promise<Record<string, unknown>>;
  delete(agentId: string, path: string, baseEtag?: string): Promise<Record<string, unknown>>;
}

export class FabuAgentStoreConflictError extends Error {
  override name = 'FabuAgentStoreConflictError';
  constructor(readonly agentId: string, readonly path: string) {
    super('Agent Store write conflicted for ' + agentId + ':' + path);
  }
}

function encodeUtf8Base64(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let binary = '';
  for (let offset = 0; offset < bytes.length; offset += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000));
  }
  return btoa(binary);
}

function decodeUtf8Base64(value: string): string {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
  return new TextDecoder().decode(bytes);
}

async function sha256Etag(value: string): Promise<string | undefined> {
  const subtle = globalThis.crypto?.subtle;
  if (!subtle) return undefined;
  const digest = await subtle.digest('SHA-256', new TextEncoder().encode(value));
  const hex = [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
  return `sha256:${hex}`;
}

function stringField(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

function numberField(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
}

/**
 * Renderer-side facade over the account Agent Store.
 *
 * Fabu uses immutable content-addressed blobs and a mutable latest-root/ref
 * layer. The server owns those semantics; this class keeps renderer writes
 * etag-aware so concurrent devices cannot silently overwrite Agent state.
 */
export class FabuAgentStore {
  private readonly refs = new Map<string, FabuAgentStoreObject>();
  private refsLoaded = false;

  constructor(
    readonly agentId: string,
    private readonly transport: FabuAgentStoreTransport,
  ) {
    if (!agentId.trim()) throw new Error('Agent id is required.');
  }

  async refresh(prefix = ''): Promise<readonly FabuAgentStoreObject[]> {
    const response = await this.transport.list(this.agentId, prefix);
    const files = Array.isArray(response.files) ? response.files : [];
    for (const file of files) {
      if (!file || typeof file.path !== 'string' || !file.path.trim()) continue;
      this.refs.set(file.path, file);
    }
    if (!prefix) this.refsLoaded = true;
    return files;
  }

  private async ensureRefs(): Promise<void> {
    if (!this.refsLoaded) await this.refresh();
  }

  async readText(path: string): Promise<string> {
    const object = await this.transport.read(this.agentId, path);
    if (typeof object.dataBase64 !== 'string') {
      throw new Error('Agent Store object has no body: ' + path);
    }
    this.refs.set(path, object);
    return decodeUtf8Base64(object.dataBase64);
  }

  async readJson<T>(path: string): Promise<T> {
    return JSON.parse(await this.readText(path)) as T;
  }

  async writeText(path: string, value: string): Promise<FabuAgentStoreObject> {
    await this.ensureRefs();
    const current = this.refs.get(path);
    const nextEtag = await sha256Etag(value);
    if (current?.etag && nextEtag && current.etag === nextEtag) {
      return current;
    }
    const response = await this.transport.write(
      this.agentId,
      path,
      encodeUtf8Base64(value),
      current?.etag ? { baseEtag: current.etag } : { expectAbsent: true },
    );
    if (response.outcome === 'conflict') {
      await this.refresh();
      throw new FabuAgentStoreConflictError(this.agentId, path);
    }
    const storedPath = stringField(response.storedPath) ?? stringField(response.path) ?? path;
    const next: FabuAgentStoreObject = {
      path: storedPath,
      blobId: stringField(response.blobId),
      etag: stringField(response.etag),
      revision: numberField(response.revision),
    };
    this.refs.set(storedPath, next);
    if (storedPath !== path) throw new FabuAgentStoreConflictError(this.agentId, path);
    return next;
  }

  async writeJson(path: string, value: unknown): Promise<FabuAgentStoreObject> {
    return this.writeText(path, JSON.stringify(value, null, 2) + '\n');
  }

  async delete(path: string): Promise<void> {
    await this.ensureRefs();
    const current = this.refs.get(path);
    await this.transport.delete(this.agentId, path, current?.etag);
    this.refs.delete(path);
  }

  rootManifest(): FabuAgentRootManifest {
    const files = [...this.refs.values()]
      .filter((entry) => entry.path !== FABU_AGENT_ROOT_PATH)
      .map((entry) => ({
        path: entry.path,
        ...(entry.blobId ? { blobId: entry.blobId } : {}),
        ...(entry.etag ? { etag: entry.etag } : {}),
        ...(entry.revision != null ? { revision: entry.revision } : {}),
      }))
      .sort((left, right) => left.path.localeCompare(right.path));
    return {
      schemaVersion: 1,
      agentId: this.agentId,
      updatedAtMs: Date.now(),
      files,
    };
  }

  async checkpointRoot(): Promise<FabuAgentStoreObject> {
    await this.ensureRefs();
    return this.writeJson(FABU_AGENT_ROOT_PATH, this.rootManifest());
  }

  async readRoot(): Promise<FabuAgentRootManifest> {
    return this.readJson<FabuAgentRootManifest>(FABU_AGENT_ROOT_PATH);
  }

  writeProfile(profile: FabuAgentProfile): Promise<FabuAgentStoreObject> {
    return this.writeJson(FABU_AGENT_PROFILE_PATH, profile);
  }

  writeSettings(settings: FabuAgentSettings): Promise<FabuAgentStoreObject> {
    return this.writeJson(FABU_AGENT_SETTINGS_PATH, settings);
  }
}
