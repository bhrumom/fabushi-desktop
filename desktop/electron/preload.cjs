'use strict';
const {contextBridge,ipcRenderer}=require('electron');

const events=[
  'agents.changed','agent.changed','message.delta','message.changed','message.done',
  'plugins.changed','settings.changed','approval.requested','approval.resolved','approval.cancelled'
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
  resolveApproval:x=>invoke('resolve-approval',x),\n  listMcpServers:()=>invoke('list-mcp-servers'),\n  addMcpServer:x=>invoke('add-mcp-server',x),\n  removeMcpServer:x=>invoke('remove-mcp-server',x),\n  setMcpServerEnabled:x=>invoke('set-mcp-server-enabled',x),\n  listMcpServerTools:x=>invoke('list-mcp-server-tools',x),\n  setMcpToolEnabled:x=>invoke('set-mcp-tool-enabled',x),
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
