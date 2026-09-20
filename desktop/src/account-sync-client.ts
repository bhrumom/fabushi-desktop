import { invokeNativeDesktop } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type { MarketplacePluginSummary } from '../../frontend/apps/web/src/lib/mahayana-host/transport';

export type AccountSyncEvent = {
  sequence: number;
  cursor: string;
  type: string;
  entityId: string;
  payload: Record<string, unknown>;
  occurredAtMs: number;
};

export type AccountSyncEnvelope = {
  protocol: 'fabushi.account.sync.v1' | string;
  mode: 'snapshot' | 'difference';
  reason?: string;
  cursor: string;
  hasMore: boolean;
  snapshot: null | {
    miniApps: Array<Record<string, unknown>>;
    bots: AccountBotMembership[];
    agents?: AccountAgentSummary[];
    cloudRevisions: Array<Record<string, unknown>>;
    agentStoreRevisions?: Array<Record<string, unknown>>;
  };
  events: AccountSyncEvent[];
};

export type AccountBotMembership = {
  bot: {
    id: string;
    /** Cloud Bot identity is distinct from the runtime Agent identity it owns. */
    agentId?: string;
    username?: string;
    displayName?: string;
    description?: string;
    conversationId?: string;
    managedBy?: string;
    menuButton?: Record<string, unknown>;
  };
  sources: Array<{ source: string; sourceId: string; addedAtMs?: number }>;
  updatedAtMs?: number;
};

export type AccountAgentSummary = {
  agentId: string;
  profile: {
    agentId?: string;
    name?: string;
    description?: string;
    title?: string;
    avatarShape?: string;
    avatarColor?: string;
  };
  metadata: {
    agentId?: string;
    latestRootBlobId?: string;
    name?: string;
    mode?: 'default' | 'plan' | 'debug' | 'search' | string;
    isRunEverything?: boolean;
    approvalMode?: 'allowlist' | 'unrestricted' | 'auto-review' | string;
    createdAt?: number;
    lastUsedModel?: string;
    lastDebugServerPort?: number;
    currentPlanUri?: string;
    subagentInfo?: Record<string, unknown>;
    blobEncryptionKeyId?: string;
  };
  createdAtMs?: number;
  updatedAtMs?: number;
};

export type AccountAgentStoreObject = {
  agentId: string;
  path: string;
  blobId: string;
  sha256?: string;
  etag: string;
  revision: number;
  updatedAtMs: number;
  sizeBytes?: number;
  dataBase64?: string;
};

export type AccountAgentStoreList = {
  agentId: string;
  files: AccountAgentStoreObject[];
};

export type AccountMiniAppList = {
  protocol?: string;
  apps: Array<{
    id: string;
    version: string;
    title?: string;
    description?: string;
    bot?: Record<string, unknown>;
    commands?: unknown[];
    surfaces?: unknown[];
    distribution?: Record<string, unknown>;
  }>;
  accountSynchronized?: boolean;
  cursor?: string | null;
};

export type MiniAppBotStoredMessage = {
  messageId: string;
  role: 'user' | 'assistant' | 'miniApp' | 'miniapp' | 'tool' | 'error';
  text: string;
  payload?: Record<string, unknown>;
  status?: string;
  createdAt: string;
  updatedAt?: string;
};

export type MiniAppBotMessagePage = {
  pluginInstanceId: string;
  messages: MiniAppBotStoredMessage[];
  nextCursor?: string | null;
};

export async function readAccountSync(cursor: string | null, limit = 200): Promise<AccountSyncEnvelope> {
  return invokeNativeDesktop<AccountSyncEnvelope>('getAccountSync', { cursor, limit });
}

export async function readAccountBots(): Promise<AccountBotMembership[]> {
  const response = await invokeNativeDesktop<{ bots?: AccountBotMembership[] }>('getAccountBots', {});
  return Array.isArray(response?.bots) ? response.bots : [];
}

export async function readAccountMiniApps(): Promise<AccountMiniAppList> {
  return invokeNativeDesktop<AccountMiniAppList>('getAccountMiniApps', {});
}

export async function readAccountAgents(): Promise<AccountAgentSummary[]> {
  const response = await invokeNativeDesktop<{ agents?: AccountAgentSummary[] }>('getAccountAgents', {});
  return Array.isArray(response?.agents) ? response.agents : [];
}

export async function readAccountAgent(agentId: string): Promise<AccountAgentSummary> {
  return invokeNativeDesktop<AccountAgentSummary>('getAccountAgent', { agentId });
}

export async function upsertAccountAgent(
  agentId: string,
  profile: AccountAgentSummary['profile'],
  metadata: AccountAgentSummary['metadata'],
): Promise<AccountAgentSummary> {
  return invokeNativeDesktop<AccountAgentSummary>('upsertAccountAgent', { agentId, profile, metadata });
}

export async function listAccountAgentStore(agentId: string, prefix = ''): Promise<AccountAgentStoreList> {
  return invokeNativeDesktop<AccountAgentStoreList>('listAgentStore', { agentId, prefix });
}

export async function readAccountAgentStoreObject(agentId: string, path: string): Promise<AccountAgentStoreObject> {
  return invokeNativeDesktop<AccountAgentStoreObject>('getAgentStoreObject', { agentId, path });
}

export async function writeAccountAgentStoreObject(
  agentId: string,
  path: string,
  dataBase64: string,
  options: { baseEtag?: string; expectAbsent?: boolean } = {},
): Promise<Record<string, unknown>> {
  return invokeNativeDesktop<Record<string, unknown>>('putAgentStoreObject', {
    agentId,
    path,
    dataBase64,
    ...options,
  });
}

export async function deleteAccountAgentStoreObject(
  agentId: string,
  path: string,
  baseEtag?: string,
): Promise<Record<string, unknown>> {
  return invokeNativeDesktop<Record<string, unknown>>('deleteAgentStoreObject', { agentId, path, baseEtag });
}

export function accountMiniAppsAsMarketplaceSummaries(account: AccountMiniAppList): MarketplacePluginSummary[] {
  return (Array.isArray(account?.apps) ? account.apps : []).flatMap((app) => {
    const pluginId = typeof app?.id === 'string' ? app.id.trim() : '';
    if (!pluginId) return [];
    const distribution = app.distribution && typeof app.distribution === 'object' ? app.distribution : {};
    return [{
      pluginId,
      displayName: typeof app.title === 'string' && app.title.trim() ? app.title.trim() : pluginId,
      description: typeof app.description === 'string' ? app.description : '',
      latestVersion: typeof app.version === 'string' && app.version.trim() ? app.version.trim() : '0',
      bot: app.bot,
      commands: Array.isArray(app.commands) ? app.commands : [],
      surfaces: Array.isArray(app.surfaces) ? app.surfaces : [],
      installMode: typeof distribution.installMode === 'string' ? distribution.installMode : undefined,
    } satisfies MarketplacePluginSummary];
  });
}

export async function reconcileAccountMiniApps(): Promise<Record<string, unknown>> {
  return invokeNativeDesktop<Record<string, unknown>>('reconcileAccountMiniApps', {});
}

export async function readMiniAppBotMessages(miniAppId: string, after = '', limit = 500): Promise<MiniAppBotMessagePage> {
  const response = await invokeNativeDesktop<MiniAppBotMessagePage | { data?: MiniAppBotMessagePage }>('getMiniAppBotMessages', {
    pluginId: miniAppId,
    after,
    limit,
  });
  return 'data' in response && response.data ? response.data : response as MiniAppBotMessagePage;
}

export async function appendMiniAppBotMessages(
  miniAppId: string,
  messages: Array<{
    messageId: string;
    role: MiniAppBotStoredMessage['role'];
    text: string;
    createdAt: string;
    updatedAt?: string;
    payload?: Record<string, unknown>;
  }>,
): Promise<void> {
  await invokeNativeDesktop('appendMiniAppBotMessages', { pluginId: miniAppId, messages });
}

export async function readMiniAppCloudStorage(miniAppId: string, key?: string): Promise<Record<string, unknown>> {
  const response = await invokeNativeDesktop<Record<string, unknown>>('getMiniAppCloudStorage', { pluginId: miniAppId, key });
  if (!key || response.item) return response;

  // Native test accounts and the production account service historically used
  // different keyed-read envelopes. Normalize every supported wire shape into
  // the single `item` contract consumed by the Mini App CloudStorage bridge.
  if (Object.prototype.hasOwnProperty.call(response, 'value')) {
    return { ...response, item: { key, value: response.value } };
  }
  const values = response.values;
  if (values && typeof values === 'object' && !Array.isArray(values) && Object.prototype.hasOwnProperty.call(values, key)) {
    return { ...response, item: { key, value: (values as Record<string, unknown>)[key] } };
  }
  const items = Array.isArray(response.items) ? response.items : [];
  const item = items.find((candidate) => {
    if (!candidate || typeof candidate !== 'object' || Array.isArray(candidate)) return false;
    return (candidate as { key?: unknown }).key === key;
  });
  return { ...response, item: item ?? null };
}

export async function writeMiniAppCloudStorage(miniAppId: string, values: Record<string, string>): Promise<Record<string, unknown>> {
  return invokeNativeDesktop<Record<string, unknown>>('setMiniAppCloudStorage', { pluginId: miniAppId, values });
}

export async function deleteMiniAppCloudStorage(miniAppId: string, key: string): Promise<Record<string, unknown>> {
  return invokeNativeDesktop<Record<string, unknown>>('deleteMiniAppCloudStorage', { pluginId: miniAppId, key });
}