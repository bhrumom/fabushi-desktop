'use strict';
const {contextBridge,ipcRenderer}=require('electron');

const events=[
  'agents.changed','agent.changed','message.delta','message.changed','message.done',
  'plugins.changed','workflows.changed','automations.changed','settings.changed','approval.requested','approval.resolved','approval.cancelled'
];
const invoke=(method,args={})=>ipcRenderer.invoke('grok-agent:'+method,args);

contextBridge.exposeInMainWorld('grokAgent',Object.freeze({
  listAgents:()=>invoke('list-agents'),
  createAgent:x=>invoke('create-agent',x),
  renameAgent:x=>invoke('rename-agent',x),
  deleteAgent:x=>invoke('delete-agent',x),
  getThread:x=>invoke('get-thread',x),
  sendMessage:x=>invoke('send-message',x),
  stopAgent:x=>invoke('stop-agent',x),
  listPlugins:()=>invoke('list-plugins'),
  setPluginInstalled:x=>invoke('set-plugin-installed',x),
  setPluginEnabled:x=>invoke('set-plugin-enabled',x),
  getRuntimeSettings:()=>invoke('get-runtime-settings'),
  setLocalToolPermission:x=>invoke('set-local-tool-permission',x),
  setAutoReviewMode:x=>invoke('set-auto-review-mode',x),
  resolveApproval:x=>invoke('resolve-approval',x),
  listMcpServers:()=>invoke('list-mcp-servers'),
  addMcpServer:x=>invoke('add-mcp-server',x),
  updateMcpServer:x=>invoke('update-mcp-server',x),
  removeMcpServer:x=>invoke('remove-mcp-server',x),
  setMcpServerEnabled:x=>invoke('set-mcp-server-enabled',x),
  getMcpAccountStatus:x=>invoke('get-mcp-account-status',x),
  listMcpAccounts:x=>invoke('list-mcp-accounts',x),
  connectMcpAccount:x=>invoke('connect-mcp-account',x),
  disconnectMcpAccount:x=>invoke('disconnect-mcp-account',x),
  renameMcpAccount:x=>invoke('rename-mcp-account',x),
  removeMcpAccount:x=>invoke('remove-mcp-account',x),
  setMcpActiveAccount:x=>invoke('set-mcp-active-account',x),
  listMcpServerTools:x=>invoke('list-mcp-server-tools',x),
  setMcpToolEnabled:x=>invoke('set-mcp-tool-enabled',x),
  listMarketplacePlugins:()=>invoke('list-marketplace-plugins'),
  installMarketplacePlugin:x=>invoke('install-marketplace-plugin',x),
  uninstallMarketplacePlugin:x=>invoke('uninstall-marketplace-plugin',x),
  listWorkflows:()=>invoke('list-workflows'),
  saveWorkflow:x=>invoke('save-workflow',x),
  deleteWorkflow:x=>invoke('delete-workflow',x),
  setWorkflowEnabled:x=>invoke('set-workflow-enabled',x),
  getAgentAutomations:x=>invoke('get-agent-automations',x),
  createAgentAutomation:x=>invoke('create-agent-automation',x),
  setAgentAutomationEnabled:x=>invoke('set-agent-automation-enabled',x),
  updateAgentAutomation:x=>invoke('update-agent-automation',x),
  deleteAgentAutomation:x=>invoke('delete-agent-automation',x),
  runAgentAutomationNow:x=>invoke('run-agent-automation-now',x),
  pickFile:()=>ipcRenderer.invoke('grok-agent:pick-file'),
  subscribe(listener){
    if(typeof listener!=='function')return()=>{};
    const handlers=events.map(name=>{
      const fn=(_e,payload)=>listener({type:name,...(payload||{})});
      ipcRenderer.on('grok-agent:event:'+name,fn);return[name,fn];
    });
    return()=>handlers.forEach(([name,fn])=>ipcRenderer.off('grok-agent:event:'+name,fn));
  }
}));
