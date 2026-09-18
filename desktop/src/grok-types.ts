export type AgentStatus = 'idle' | 'thinking' | 'running' | 'waiting' | 'error';
export type AccountStatus =
  | {kind:'logged-out';available:boolean;reason?:string|null}
  | {kind:'logging-in';available:boolean}
  | {kind:'logged-in';available:true;authId:string|null;email:string|null;displayName:string|null;avatarUrl:string|null};
export type ToolMessageStatus = 'queued' | 'waiting-approval' | 'running' | 'streaming' | 'done' | 'error' | 'cancelled';

export interface AgentSummary {
  id:string;
  name:string;
  status:AgentStatus;
  createdAt:number;
  updatedAt:number;
  unread:boolean;
  pinned?:boolean;
  isGroup?:boolean;
  memberIds?:string[];
  isSharedRoom?:boolean;
  description?:string;
  title?:string;
  notifyOnUpdatesEnabled?:boolean;
  purpose?:string;
  parentAgentId?:string|null;
  hidden?:boolean;
  avatarDataUrl?:string|null;
  avatarShape?:string|null;
  avatarColor?:string|null;
}
export interface AttachmentDescriptor {
  id:string;
  name:string;
  mime:string;
  size:number;
  kind:'image'|'pdf'|'text'|'file';
  createdAt:number;
  remoteUrl?:string;
  alt?:string;
}
export interface AttachmentPreview extends AttachmentDescriptor { dataUrl:string; }
export interface AgentMessage {
  id:string;
  role:'user'|'assistant'|'system'|'tool';
  text:string;
  createdAt:number;
  updatedAt?:number;
  status?:ToolMessageStatus;
  toolName?:string;
  arguments?:Record<string,unknown>;
  approvalId?:string;
  approvalSummary?:string;
  errorCode?:string;
  requestId?:string;
  toolCallId?:string;
  reviewMode?:'off'|'shadow'|'enforce';
  reviewFingerprint?:string;
  reviewDecision?:'allow'|'block';
  reviewReason?:string;
  outputLocation?:{filePath:string;sizeBytes:number;lineCount:number;truncated:boolean;originalSizeBytes:number;toolCallId:string};
  display?:{kind:'image';dataUrl:string};
  attachments?:AttachmentDescriptor[];
  replyToId?:string;
  reactions?:{emoji:string;by:string}[];
  widget?:{prompt:string;helpText?:string;options:{label:string;value?:string;description?:string;style?:string}[];allowCustom?:boolean;dismissOnMoveOn?:boolean};
  respondedValue?:string|null;
  widgetDismissed?:boolean;
  secretRequest?:{label:string;description?:string;connector:string;field:string};
  secretProvided?:boolean;
}
export type ConversationOutlineItem =
  | {kind:'user';id:string;text:string}
  | {kind:'assistant-text';id:string;text:string}
  | {kind:'tool-call';id:string;name:string;status:'pending'|'done'|'failed';summary?:string;outputLocation?:{filePath:string;sizeBytes:number;lineCount:number;truncated:boolean;originalSizeBytes:number;toolCallId:string}};
export interface ConversationOutlineTurn {
  rawUserText:string;
  userMessageId:string;
  items:ConversationOutlineItem[];
}
export interface PendingApproval {
  id:string;
  agentId:string;
  messageId:string;
  toolName:string;
  summary:string;
  args:Record<string,unknown>;
  createdAt:number;
}
export interface WorkspaceMessageSearchResult { agentId:string;agentName:string;entryId:string;role:'user'|'assistant';text:string;timestampMs:number; }
export interface WorkspaceMediaSearchResult { agentId:string;agentName:string;entryId:string;attachmentId:string;fileName:string;ext:string;mime:string|null;kind:'image'|'video'|'audio'|'pdf'|'markdown'|'table'|'json'|'text'|'document'|'archive'|'file';timestampMs:number;width:number|null;height:number|null; }
export interface WorkspaceLinkSearchResult { agentId:string;agentName:string;entryId:string;url:string;timestampMs:number; }
export interface AgentThread {
  agent:AgentSummary;
  messages:AgentMessage[];
  outline?:ConversationOutlineTurn[];
  pendingApprovals?:PendingApproval[];
}
export interface PluginDescriptor {
  id:string;
  name:string;
  description:string;
  category:string;
  installed:boolean;
  enabled:boolean;
  builtin:boolean;
  provider?:string;
  removable?:boolean;
  kind?:'local'|'mcp';
  serverId?:string;
  transport?:'stdio'|'http';
  accountKey?:string;
}
export interface McpServerDescriptor {
  id:string;
  name:string;
  transport:'stdio'|'http';
  command:string;
  args:string[];
  url:string;
  enabled:boolean;
  disabledTools:string[];
  customInstructions:string;
  accountKey:string;
  accountKeys:string[];
  oauthClientId:string;
  oauthAuthorizationUrl:string;
  oauthTokenUrl:string;
  oauthRegistrationUrl:string;
  oauthScopes:string[];
}
export interface McpAccountStatus {
  serverId:string;
  accountKey:string;
  connected:boolean;
  expiresAt:number|null;
  scope:string;
  supported:boolean;
  active?:boolean;
}
export interface McpToolDescriptor {
  name:string;
  description:string;
  inputSchema:Record<string,unknown>;
  isDisabled:boolean;
}
export interface MarketplaceVariableField {
  key:string;label:string;placeholder:string;isRequired:boolean;isSecret:boolean;defaultValue?:string;hint?:string;
}
export interface MarketplacePluginDescriptor {
  id:string;name:string;displayName:string;description:string;category:string;homepage:string|null;iconUrl:string|null;
  connectors:{name:string;description:string}[];skills:{name:string;description:string}[];fields:MarketplaceVariableField[];
  publisher:{name:string;displayName:string;isUserOwned:boolean}|null;
  marketplace:{name:string;displayName:string;ownership:'team'|'user'}|null;
  installed:boolean;install?:{serverIds:string[];skillIds:string[];installedAt:number}|null;
}
export interface MarketplaceCatalogDescriptor {
  available:boolean;reason:string|null;includesPrivateMarketplaces:boolean;plugins:MarketplacePluginDescriptor[];
}
export interface WorkflowDescriptor {
  id:string;
  name:string;
  description:string;
  body:string;
  trigger:{schedule:string;isEnabled:boolean}|null;
  source:'workflow';
  sourceRef:string|null;
  isEnabledForAgent:boolean;
  disableModelInvocation:boolean;
  createdAt:number;
  updatedAt:number;
  filePath:string;
}
export interface RoutineRunDescriptor {
  id:string;
  status:'running'|'ok'|'error';
  startedAt:number;
  detail?:string|null;
  event?:string|null;
}
export interface RoutineAutomationDescriptor {
  id:string;
  name:string;
  prompt:string;
  trigger:{type:'cron';schedule:string};
  triggerDescription:string;
  isEnabled:boolean;
  runs:RoutineRunDescriptor[];
  createdAt:number;
  lastRunAt:number|null;
  nextRunAt:number|null;
}
export interface AgentChannelManifest {
  platform:string;displayName:string;blurb:string;credentialLabel:string;availability:'available'|'coming-soon';connectGuide:string;
  setupGuide?:{steps:{text:string;code?:string}[]};
}
export interface AgentChannelConnection {platform:string;label:string;status:string;detail?:string|null}
export interface AgentChannelsView {manifests:AgentChannelManifest[];connections:AgentChannelConnection[]}
export interface AsyncTaskSummary {
  kind:'subagent'|'shell'|'cloud-agent';
  id:string;
  label:string;
  status:'running';
  startedAtMs:number;
  detail?:string;
  subagentType?:string;
}
export interface ExperimentsSnapshot {
  isInitialized:boolean;
  featureGates:Record<string,boolean>;
  experiments:Record<string,Record<string,unknown>>;
  dynamicConfigs:Record<string,Record<string,unknown>>;
  featureFlags?:{items:{name:string;value:boolean;override:boolean|null}[];isLive:boolean};
}
export type FeatureFlagOverrideCommand =
  | {kind:'set';name:string;value:boolean}
  | {kind:'clear';name:string}
  | {kind:'clear-all'};

export interface RuntimeSettings {
  localToolPermission:'always'|'ask'|'never';
  autoReviewMode:'off'|'shadow'|'enforce';
  autoReviewAllowInstructions:string[];
  autoReviewBlockInstructions:string[];
  computerTarget:'local-mac';
}
export type DesktopUpdateTrack='stable'|'nightly'|'dogfood';
export type DesktopUpdateState=
  |{type:'disabled';reason:'not-packaged'|'unsupported-platform'|'disabled-by-env'}
  |{type:'idle';lastCheck?:{at:number;result:'up-to-date'|'error';errorMessage?:string}}
  |{type:'checking'}
  |{type:'available';version:string}
  |{type:'downloading';version:string;progress?:number}
  |{type:'ready';version:string};
export interface DesktopUpdateStatus {
  state:DesktopUpdateState;
  currentVersion:string;
  currentTrack:DesktopUpdateTrack;
  trackOverride:DesktopUpdateTrack|null;
  buildDefaultTrack:DesktopUpdateTrack;
  availableTracks:DesktopUpdateTrack[];
  isTrackManagedByPolicy:boolean;
  isBelowMinimumVersion:boolean;
  autoUpdateWhenIdleOptIn:boolean;
  autoUpdateWhenIdleGateEnabled:boolean;
}
export interface DesktopInfo {version:string;platform:string;isPackaged:boolean}
export interface DeepLinkInfo {version:1;source:'protocol';route:'info';topic:'deep-links';url:string}
export type FeedbackResult={ok:true}|{ok:false;code:'access-denied'|'invalid-feedback'|'not-signed-in'|'rate-limited'|'subscription-required'|'unavailable'};
export interface AgentEvent {
  type:
    |'agents.changed'|'agent.changed'|'message.delta'|'message.changed'|'message.done'
    |'plugins.changed'|'workflows.changed'|'automations.changed'|'settings.changed'|'account.changed'|'experiments.changed'
    |'approval.requested'|'approval.resolved'|'approval.cancelled';
  agentId?:string;
  messageId?:string;
  text?:string;
  status?:string;
  approvalId?:string;
  approved?:boolean;
}
export interface GrokAgentBridge {
  listAgents():Promise<AgentSummary[]>;
  createAgent(input:{name:string}):Promise<AgentSummary>;
  renameAgent(input:{agentId:string;name:string}):Promise<AgentSummary>;
  updateAgent(input:{id:string;profile?:{name:string;title?:string;description:string};avatarShape?:string;avatarColor?:string}):Promise<AgentSummary>;
  setAgentNotifyOnUpdates(input:{id:string;isEnabled:boolean}):Promise<AgentSummary>;
  setAgentPinned(input:{agentId:string;pinned:boolean}):Promise<AgentSummary>;
  setAgentUnread(input:{agentId:string;unread:boolean}):Promise<AgentSummary>;
  duplicateAgent(input:{agentId:string}):Promise<AgentSummary>;
  setGroupMembers(input:{id:string;memberAgentIds:string[]}):Promise<AgentSummary>;
  setAgentHidden(input:{agentId:string;hidden:boolean}):Promise<AgentSummary>;
  deleteAgent(input:{agentId:string}):Promise<{ok:true}>;
  getThread(input:{agentId:string}):Promise<AgentThread>;
  sendMessage(input:{agentId:string;text:string;attachmentIds?:string[];replyToId?:string|null}):Promise<{messageId:string}>;
  respondToWidget(input:{agentId:string;entryId:string;value:string}):Promise<{accepted:boolean}>;
  dismissWidget(input:{agentId:string;entryId:string}):Promise<{accepted:boolean}>;
  submitSecret(input:{agentId:string;entryId:string;value:string}):Promise<{accepted:boolean;reason?:string}>;
  reactToMessage(input:{agentId:string;entryId:string;emoji:string}):Promise<{reactions:{emoji:string;by:string}[]}>;
  searchMessages(input:{query:string;limit?:number}):Promise<WorkspaceMessageSearchResult[]>;
  searchMedia(input:{query:string;limit?:number}):Promise<WorkspaceMediaSearchResult[]>;
  searchLinks(input:{query:string;limit?:number}):Promise<WorkspaceLinkSearchResult[]>;
  stopAgent(input:{agentId:string}):Promise<{ok:true}>;
  listPlugins():Promise<PluginDescriptor[]>;
  setPluginInstalled(input:{pluginId:string;installed:boolean}):Promise<PluginDescriptor[]>;
  setPluginEnabled(input:{pluginId:string;enabled:boolean}):Promise<PluginDescriptor[]>;
  getAccountStatus():Promise<AccountStatus>;
  loginAccount():Promise<AccountStatus>;
  cancelAccountLogin():Promise<AccountStatus>;
  logoutAccount():Promise<AccountStatus>;
  updateAccountName(input:{name:string}):Promise<AccountStatus>;
  getAccountAvatar():Promise<string|null>;
  getAgentChannels(input:{id:string}):Promise<AgentChannelsView>;
  connectChannel(input:{id:string;platform:string;token:string}):Promise<AgentChannelsView>;
  disconnectChannel(input:{id:string;platform:string}):Promise<AgentChannelsView>;
  refreshChannel(input:{id:string;platform:string}):Promise<AgentChannelsView>;
  getAsyncTasks(input:{id:string}):Promise<AsyncTaskSummary[]>;
  getRuntimeSettings():Promise<RuntimeSettings>;
  getExperimentsSnapshot():Promise<ExperimentsSnapshot>;
  refreshExperiments():Promise<ExperimentsSnapshot>;
  applyFeatureFlagOverride(input:FeatureFlagOverrideCommand|{command:FeatureFlagOverrideCommand}):Promise<ExperimentsSnapshot>;
  setLocalToolPermission(input:{permission:RuntimeSettings['localToolPermission']}):Promise<RuntimeSettings>;
  setAutoReviewMode(input:{mode:RuntimeSettings['autoReviewMode']}):Promise<RuntimeSettings>;
  setAutoReviewInstructions(input:{allowInstructions:string[];blockInstructions:string[]}):Promise<RuntimeSettings>;
  resolveApproval(input:{approvalId:string;approved:boolean}):Promise<{ok:true}>;
  listMcpServers():Promise<McpServerDescriptor[]>;
  addMcpServer(input:{name:string;transport?:'stdio'|'http';command?:string;args?:string[];url?:string;customInstructions?:string;accountKey?:string;oauthClientId?:string;oauthAuthorizationUrl?:string;oauthTokenUrl?:string;oauthRegistrationUrl?:string;oauthScopes?:string[]}):Promise<McpServerDescriptor>;
  updateMcpServer(input:{serverId:string;name?:string;transport?:'stdio'|'http';command?:string;args?:string[];url?:string;customInstructions?:string;accountKey?:string;oauthClientId?:string;oauthAuthorizationUrl?:string;oauthTokenUrl?:string;oauthRegistrationUrl?:string;oauthScopes?:string[]}):Promise<McpServerDescriptor>;
  removeMcpServer(input:{serverId:string}):Promise<{ok:true}>;
  setMcpServerEnabled(input:{serverId:string;enabled:boolean}):Promise<McpServerDescriptor>;
  getMcpAccountStatus(input:{serverId:string;accountKey?:string}):Promise<McpAccountStatus>;
  listMcpAccounts(input:{serverId:string}):Promise<McpAccountStatus[]>;
  connectMcpAccount(input:{serverId:string;accountKey?:string}):Promise<McpAccountStatus>;
  disconnectMcpAccount(input:{serverId:string;accountKey?:string}):Promise<McpAccountStatus>;
  renameMcpAccount(input:{serverId:string;accountKey:string;newAccountKey:string}):Promise<McpAccountStatus>;
  removeMcpAccount(input:{serverId:string;accountKey:string}):Promise<McpAccountStatus[]>;
  setMcpActiveAccount(input:{serverId:string;accountKey:string}):Promise<McpAccountStatus[]>;
  listMcpServerTools(input:{serverId:string}):Promise<McpToolDescriptor[]>;
  setMcpToolEnabled(input:{serverId:string;toolName:string;enabled:boolean}):Promise<McpToolDescriptor[]>;
  listMarketplacePlugins():Promise<MarketplaceCatalogDescriptor>;
  installMarketplacePlugin(input:{entryId:string;values?:Record<string,string>}):Promise<MarketplaceCatalogDescriptor>;
  uninstallMarketplacePlugin(input:{entryId:string}):Promise<MarketplaceCatalogDescriptor>;
  listWorkflows():Promise<WorkflowDescriptor[]>;
  saveWorkflow(input:{id?:string;name:string;description?:string;body:string;trigger?:{schedule:string;isEnabled:boolean}|null;isEnabledForAgent?:boolean;disableModelInvocation?:boolean}):Promise<WorkflowDescriptor>;
  deleteWorkflow(input:{id:string}):Promise<{ok:true}>;
  setWorkflowEnabled(input:{id:string;enabled:boolean}):Promise<WorkflowDescriptor>;
  getAgentAutomations(input:{id:string}):Promise<RoutineAutomationDescriptor[]>;
  createAgentAutomation(input:{id:string;spec:{name:string;prompt:string;trigger:{type:'cron';schedule:string};isEnabled:boolean}}):Promise<RoutineAutomationDescriptor[]>;
  setAgentAutomationEnabled(input:{id:string;automationId:string;isEnabled:boolean}):Promise<RoutineAutomationDescriptor[]>;
  updateAgentAutomation(input:{id:string;automationId:string;spec:{name:string;prompt:string;trigger:{type:'cron';schedule:string};isEnabled:boolean}}):Promise<RoutineAutomationDescriptor[]>;
  deleteAgentAutomation(input:{id:string;automationId:string}):Promise<RoutineAutomationDescriptor[]>;
  runAgentAutomationNow(input:{id:string;automationId:string}):Promise<void>;
  pickFile():Promise<AttachmentDescriptor|null>;
  pickAvatarFile():Promise<{dataUrl:string;fileName:string}|null>;
  generateAgentAvatarImage(description:string):Promise<string>;
  setAgentAvatarBytes(input:{id:string;pngBase64:string|null}):Promise<AgentSummary>;
  readAttachment(input:{id:string}):Promise<AttachmentPreview>;
  getDesktopInfo():Promise<DesktopInfo>;
  getUpdateStatus():Promise<DesktopUpdateStatus>;
  checkUpdate():Promise<DesktopUpdateStatus>;
  setUpdateTrack(input:{track:DesktopUpdateTrack}):Promise<DesktopUpdateStatus>;
  setAutoUpdate(input:{enabled:boolean}):Promise<DesktopUpdateStatus>;
  quitAndInstall():Promise<void>;
  submitFeedback(input:{message:string;conversationId?:string}):Promise<FeedbackResult>;
  getOnboardingSeen():Promise<boolean>;
  setOnboardingSeen(input:{seen:boolean}):Promise<boolean>;
  openExternal(input:{url:string}):Promise<{ok:true}>;
  onUpdateStatus(listener:(status:DesktopUpdateStatus)=>void):()=>void;
  onDeepLink(listener:(link:DeepLinkInfo)=>void):()=>void;
  subscribe(listener:(event:AgentEvent)=>void):()=>void;
}
