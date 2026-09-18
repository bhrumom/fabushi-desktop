export type AgentStatus = 'idle' | 'thinking' | 'running' | 'waiting' | 'error';
export type ToolMessageStatus = 'queued' | 'waiting-approval' | 'running' | 'streaming' | 'done' | 'error' | 'cancelled';

export interface AgentSummary {
  id:string;
  name:string;
  status:AgentStatus;
  createdAt:number;
  updatedAt:number;
  unread:boolean;
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
  display?:{kind:'image';dataUrl:string};
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
}
export interface McpServerDescriptor {
  id:string;
  name:string;
  command:string;
  args:string[];
  enabled:boolean;
  disabledTools:string[];
}
export interface McpToolDescriptor {
  name:string;
  description:string;
  inputSchema:Record<string,unknown>;
  isDisabled:boolean;
}
export interface RuntimeSettings {
  localToolPermission:'always'|'ask'|'never';
  computerTarget:'local-mac';
}
export interface AgentEvent {
  type:
    |'agents.changed'|'agent.changed'|'message.delta'|'message.changed'|'message.done'
    |'plugins.changed'|'settings.changed'
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
  deleteAgent(input:{agentId:string}):Promise<{ok:true}>;
  getThread(input:{agentId:string}):Promise<AgentThread>;
  sendMessage(input:{agentId:string;text:string}):Promise<{messageId:string}>;
  stopAgent(input:{agentId:string}):Promise<{ok:true}>;
  listPlugins():Promise<PluginDescriptor[]>;
  setPluginInstalled(input:{pluginId:string;installed:boolean}):Promise<PluginDescriptor[]>;
  setPluginEnabled(input:{pluginId:string;enabled:boolean}):Promise<PluginDescriptor[]>;
  getRuntimeSettings():Promise<RuntimeSettings>;
  setLocalToolPermission(input:{permission:RuntimeSettings['localToolPermission']}):Promise<RuntimeSettings>;
  resolveApproval(input:{approvalId:string;approved:boolean}):Promise<{ok:true}>;
  listMcpServers():Promise<McpServerDescriptor[]>;
  addMcpServer(input:{name:string;command:string;args?:string[]}):Promise<McpServerDescriptor>;
  removeMcpServer(input:{serverId:string}):Promise<{ok:true}>;
  setMcpServerEnabled(input:{serverId:string;enabled:boolean}):Promise<McpServerDescriptor>;
  listMcpServerTools(input:{serverId:string}):Promise<McpToolDescriptor[]>;
  setMcpToolEnabled(input:{serverId:string;toolName:string;enabled:boolean}):Promise<McpToolDescriptor[]>;
  pickFile():Promise<{path:string;name:string}|null>;
  subscribe(listener:(event:AgentEvent)=>void):()=>void;
}
