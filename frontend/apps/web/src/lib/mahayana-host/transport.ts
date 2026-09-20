import type {
  ApprovalResolution,
  AuthState,
  AuthProvider,
  BrowserLoginAttempt,
  BrowserLoginPollResult,
  BrowserLoginReopenResult,
  AuthProviderId,
  CommandAccepted,
  HostConfig,
  HostInfo,
  RuntimeCommand,
  RuntimeEvent,
  OAuthAttempt,
  OAuthPollResult,
} from "./contracts";

export type RuntimeEventListener = (event: RuntimeEvent) => void;

export type MarketplacePluginSummary = {
  pluginId: string;
  displayName: string;
  description: string;
  latestVersion: string;
  platforms?: string[];
  releaseStatus?: string;
  releaseManifest?: Record<string, unknown>;
  install?: Record<string, unknown>;
  source?: Record<string, unknown>;
  bot?: Record<string, unknown>;
  commands?: unknown[];
  surfaces?: unknown[];
  installMode?: string;
};

export type MarketplaceBrowseResult = {
  plugins: MarketplacePluginSummary[];
};

export type MarketplaceReleaseMetadata = {
  pluginId: string;
  version: string;
  releaseStatus?: string;
  releaseManifest: Record<string, unknown>;
  install?: Record<string, unknown>;
};

export type InstalledPluginPointer = {
  pluginId: string;
  version: string;
  artifactId: string;
  artifactSha256: string;
  runtime: string;
  entry?: string;
  requestedPermissions?: string[];
  installedPath: string;
};

export type InstalledPluginList = {
  plugins: InstalledPluginPointer[];
};

export type PluginUninstallResult = {
  pluginId: string;
  removed: boolean;
  permissionsRemoved?: boolean;
};

export type PluginUiDocument = {
  pluginId: string;
  html: string;
};

export interface MahayanaHostTransport {
  initialize(config: HostConfig): Promise<HostInfo>;
  execute(command: RuntimeCommand): Promise<CommandAccepted>;
  marketplaceBrowse(query?: string): Promise<MarketplaceBrowseResult>;
  marketplaceRelease(pluginId: string, version: string): Promise<MarketplaceReleaseMetadata>;
  pluginInstall(release: Record<string, unknown>, platform?: string): Promise<InstalledPluginPointer>;
  pluginUninstall(pluginId: string): Promise<PluginUninstallResult>;
  pluginRollback(pluginId: string): Promise<InstalledPluginPointer | null>;
  pluginActive(pluginId: string): Promise<InstalledPluginPointer | null>;
  pluginListInstalled(): Promise<InstalledPluginList>;
  pluginUiDocument(pluginId: string): Promise<PluginUiDocument>;
  authStatus(): Promise<AuthState>;
  authProviders(): Promise<AuthProvider[]>;
  browserLoginStart(): Promise<BrowserLoginAttempt>;
  browserLoginPoll(attemptId: string): Promise<BrowserLoginPollResult>;
  browserLoginCancel(attemptId: string): Promise<BrowserLoginPollResult>;
  browserLoginReopen(attemptId: string): Promise<BrowserLoginReopenResult>;
  oauthStart(provider: AuthProviderId): Promise<OAuthAttempt>;
  oauthPoll(attemptId: string): Promise<OAuthPollResult>;
  openExternal(url: string): Promise<void>;
  openSystemSettings(pane: "screen-recording" | "accessibility"): Promise<void>;
  windowFocused(): Promise<boolean>;
  showNotification(title: string, body: string): Promise<void>;
  passwordLogin(username: string, password: string): Promise<AuthState>;
  logout(): Promise<AuthState>;
  subscribe(listener: RuntimeEventListener): () => void;
  interrupt(operationId: string): Promise<void>;
  resolveApproval(resolution: ApprovalResolution): Promise<void>;
  close(): Promise<void>;
}
