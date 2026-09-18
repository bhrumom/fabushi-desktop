export type AgentStatus = 'idle' | 'thinking' | 'running' | 'waiting' | 'error';
export type ToolMessageStatus = 'queued' | 'waiting-approval' | 'running' | 'streaming' | 'done' | 'error' | 'cancelled';

export interface AgentSummary {
  id:string;
  name:string;
  status:AgentStatus;
  createdAt:number;
  updatedAt:number;
  unread:boolean;
  description?:string;
  purpose?:string;
  parentAgentId?:string|null;
  hidden?:boolean;
}
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
export interface RuntimeSettings {
  localToolPermission:'always'|'ask'|'never';
  autoReviewMode:'off'|'shadow'|'enforce';
  computerTarget:'local-mac';
}
export interface AgentEvent {
  type:
    |'agents.changed'|'agent.changed'|'message.delta'|'message.changed'|'message.done'
    |'plugins.changed'|'workflows.changed'|'automations.changed'|'settings.changed'
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
  setAgentHidden(input:{agentId:string;hidden:boolean}):Promise<AgentSummary>;
  deleteAgent(input:{agentId:string}):Promise<{ok:true}>;
  getThread(input:{agentId:string}):Promise<AgentThread>;
  sendMessage(input:{agentId:string;text:string}):Promise<{messageId:string}>;
  stopAgent(input:{agentId:string}):Promise<{ok:true}>;
  listPlugins():Promise<PluginDescriptor[]>;
  setPluginInstalled(input:{pluginId:string;installed:boolean}):Promise<PluginDescriptor[]>;
  setPluginEnabled(input:{pluginId:string;enabled:boolean}):Promise<PluginDescriptor[]>;
  getRuntimeSettings():Promise<RuntimeSettings>;
  setLocalToolPermission(input:{permission:RuntimeSettings['localToolPermission']}):Promise<RuntimeSettings>;
  setAutoReviewMode(input:{mode:RuntimeSettings['autoReviewMode']}):Promise<RuntimeSettings>;
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
  pickFile():Promise<{path:string;name:string}|null>;
  subscribe(listener:(event:AgentEvent)=>void):()=>void;
}
