export type AgentStatus = 'idle' | 'thinking' | 'running' | 'waiting' | 'error';
export interface AgentSummary { id:string; name:string; status:AgentStatus; createdAt:number; updatedAt:number; unread:boolean; }
export interface AgentMessage { id:string; role:'user'|'assistant'|'system'|'tool'; text:string; createdAt:number; status?:'streaming'|'done'|'error'; toolName?:string; }
export interface AgentThread { agent:AgentSummary; messages:AgentMessage[]; }
export interface PluginDescriptor { id:string; name:string; description:string; category:string; installed:boolean; enabled:boolean; builtin:boolean; }
export interface AgentEvent { type:'agents.changed'|'agent.changed'|'message.delta'|'message.done'|'plugins.changed'; agentId?:string; messageId?:string; text?:string; }
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
  pickFile():Promise<{path:string;name:string}|null>;
  subscribe(listener:(event:AgentEvent)=>void):()=>void;
}