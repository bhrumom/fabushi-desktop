'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');
const crypto=require('node:crypto');
const {createHostRuntime}=require('./grok-host-runtime.cjs');
const {createMcpManager,normalizeServer}=require('./grok-mcp-manager.cjs');
const {createWorkflowManager}=require('./grok-workflow-manager.cjs');

const capabilityCatalog=[
  {id:'filesystem',name:'Files',description:'Read and modify files on this Mac.',category:'Computer',builtin:true,provider:'local-exec'},
  {id:'shell',name:'Terminal',description:'Run cancellable foreground and background shell processes on this Mac.',category:'Computer',builtin:true,provider:'local-exec'},
  {id:'browser',name:'Browser',description:'Open approved HTTPS pages from the desktop agent.',category:'Web',builtin:true,provider:'local-exec'},
  {id:'computer',name:'Computer',description:'Capture the screen and drive macOS input through Accessibility.',category:'Computer',builtin:true,provider:'local-exec'}
];
const catalogIds=new Set(capabilityCatalog.map(x=>x.id));
const clean=(v,f='Agent')=>String(v||'').trim().replace(/[\r\n\t]/g,' ').slice(0,80)||f;
function defaultCapabilities(){return Object.fromEntries(capabilityCatalog.map(p=>[p.id,{installed:true,enabled:true}]));}
function initialState(){
  const now=Date.now(),id=crypto.randomUUID();
  return{
    version:3,
    agents:[{id,name:'Chief',status:'idle',createdAt:now,updatedAt:now,unread:false}],
    messages:{[id]:[]},
    plugins:defaultCapabilities(),
    settings:{localToolPermission:'ask'},
    pendingApprovals:{},
    mcpServers:[]
  };
}
function normalizeState(parsed){
  if(!parsed||typeof parsed!=='object')return initialState();
  const base=initialState();
  if(Array.isArray(parsed.agents)&&parsed.agents.length){
    base.agents=parsed.agents.map(a=>({...a,status:['idle','thinking','running','waiting','error'].includes(a.status)?a.status:'idle'}));
    base.messages={};
    for(const agent of base.agents)base.messages[agent.id]=Array.isArray(parsed.messages?.[agent.id])?parsed.messages[agent.id]:[];
  }
  for(const id of catalogIds){
    const previous=parsed.plugins?.[id];
    if(previous)base.plugins[id]={installed:true,enabled:previous.enabled!==false};
  }
  const permission=parsed.settings?.localToolPermission;
  if(['always','ask','never'].includes(permission))base.settings.localToolPermission=permission;
  base.pendingApprovals={};
  base.mcpServers=Array.isArray(parsed.mcpServers)?parsed.mcpServers.flatMap(server=>{try{return[normalizeServer(server)]}catch{return[]}}):[];
  return base;
}
function createCoordinatorRuntime({app,BrowserWindow,shell}){
  const file=path.join(app.getPath('userData'),'grok-agent-runtime.json');
  let state=null,writing=Promise.resolve();
  const aborts=new Map();
  const approvals=new Map();

  async function load(){
    if(state)return state;
    try{state=normalizeState(JSON.parse(await fs.readFile(file,'utf8')))}catch{state=initialState()}
    return state;
  }
  async function save(){
    const text=JSON.stringify(await load(),null,2)+'\n';
    writing=writing.then(async()=>{
      await fs.mkdir(path.dirname(file),{recursive:true});
      const tmp=file+'.tmp';await fs.writeFile(tmp,text,{mode:0o600});await fs.rename(tmp,file);
    });
    return writing;
  }
  function emit(type,payload={}){
    for(const win of BrowserWindow.getAllWindows()){
      if(!win.isDestroyed()&&!win.webContents.isDestroyed())win.webContents.send('grok-agent:event:'+type,payload);
    }
  }
  async function findAgent(id){const s=await load();return s.agents.find(a=>a.id===id)||null}
  async function listAgents(){const s=await load();return[...s.agents].sort((a,b)=>b.updatedAt-a.updatedAt)}
  async function createAgent({name}){
    const s=await load(),now=Date.now(),agent={id:crypto.randomUUID(),name:clean(name),status:'idle',createdAt:now,updatedAt:now,unread:false};
    s.agents.unshift(agent);s.messages[agent.id]=[];await save();emit('agents.changed');return agent;
  }
  async function renameAgent({agentId,name}){
    const agent=await findAgent(agentId);if(!agent)throw Error('Agent not found');
    agent.name=clean(name,agent.name);agent.updatedAt=Date.now();await save();emit('agent.changed',{agentId});return agent;
  }
  async function deleteAgent({agentId}){
    const s=await load();aborts.get(agentId)?.abort();cancelApprovals(agentId,'Agent deleted.');
    s.agents=s.agents.filter(a=>a.id!==agentId);delete s.messages[agentId];await save();emit('agents.changed');return{ok:true};
  }
  async function getThread({agentId}){
    const s=await load(),agent=s.agents.find(x=>x.id===agentId);if(!agent)throw Error('Agent not found');
    return{agent,messages:s.messages[agentId]||[],pendingApprovals:Object.values(s.pendingApprovals||{}).filter(x=>x.agentId===agentId)};
  }
  function enabledCapabilityIds(s){
    return new Set(capabilityCatalog.filter(p=>s.plugins?.[p.id]?.installed&&s.plugins?.[p.id]?.enabled).map(p=>p.id));
  }
  async function onAgentStatus(agentId,status){
    const agent=await findAgent(agentId);if(!agent)return;
    agent.status=status;agent.updatedAt=Date.now();await save();emit('agent.changed',{agentId,status});
  }
  async function onToolState({agentId,entry}){
    await save();emit('message.changed',{agentId,messageId:entry.id,status:entry.status});
  }
  async function getLocalToolPermission(){return(await load()).settings.localToolPermission}
  async function requestApproval({agentId,messageId,toolName,summary,args,signal}){
    if(signal?.aborted)return false;
    const s=await load(),approvalId=crypto.randomUUID();
    const message=(s.messages[agentId]||[]).find(x=>x.id===messageId);
    if(message){message.approvalId=approvalId;message.approvalSummary=summary}
    s.pendingApprovals[approvalId]={id:approvalId,agentId,messageId,toolName,summary,args,createdAt:Date.now()};
    await save();emit('approval.requested',{approvalId,agentId,messageId,toolName,summary});
    return await new Promise(resolve=>{
      let settled=false;
      const finish=value=>{
        if(settled)return;settled=true;
        signal?.removeEventListener('abort',abort);
        approvals.delete(approvalId);
        void (async()=>{const current=await load();delete current.pendingApprovals[approvalId];await save();emit('approval.resolved',{approvalId,agentId,approved:value})})().finally(()=>resolve(value));
      };
      const abort=()=>finish(false);
      approvals.set(approvalId,{agentId,finish});
      signal?.addEventListener('abort',abort,{once:true});
    });
  }
  function cancelApprovals(agentId,reason='Cancelled.'){
    for(const [id,approval] of approvals){if(approval.agentId===agentId)approval.finish(false)}
    void load().then(async s=>{
      let changed=false;
      for(const [id,value] of Object.entries(s.pendingApprovals||{})){if(value.agentId===agentId){delete s.pendingApprovals[id];changed=true}}
      if(changed){await save();emit('approval.cancelled',{agentId,reason})}
    });
  }
  async function resolveApproval({approvalId,approved}){
    const pending=approvals.get(String(approvalId||''));if(!pending)throw Error('Approval is no longer pending.');
    pending.finish(approved===true);return{ok:true};
  }

  const mcp=createMcpManager({getServers:async()=>[...((await load()).mcpServers||[])]});
  const workflowManager=createWorkflowManager({app});
  const host=createHostRuntime({
    shell,getLocalToolPermission,requestApproval,onToolState,onAgentStatus,
    getExternalTools:()=>mcp.collectToolDefinitions(),
    executeExternalTool:(name,args)=>mcp.executeRoutedTool(name,args),
    getWorkflowContext:prompt=>workflowManager.buildAgentContext(prompt)
  });

  async function sendMessage({agentId,text}){
    const s=await load(),agent=s.agents.find(x=>x.id===agentId);if(!agent)throw Error('Agent not found');
    const body=String(text||'').trim();if(!body)throw Error('Message required');
    if(['thinking','running','waiting'].includes(agent.status))throw Error('Agent already running');
    const now=Date.now(),user={id:crypto.randomUUID(),role:'user',text:body,createdAt:now,status:'done'};
    const assistant={id:crypto.randomUUID(),role:'assistant',text:'',createdAt:now+1,status:'streaming'};
    const transcript=s.messages[agentId]||(s.messages[agentId]=[]);
    transcript.push(user,assistant);agent.status='thinking';agent.updatedAt=now;await save();
    emit('message.changed',{agentId,messageId:user.id,status:'done'});emit('agent.changed',{agentId,status:'thinking'});
    const controller=new AbortController();aborts.set(agentId,controller);

    void(async()=>{
      try{
        await onAgentStatus(agentId,'running');
        assistant.text=await host.runTurn({
          agent,history:transcript.slice(0,-1),transcript,enabled:enabledCapabilityIds(s),signal:controller.signal
        });
        assistant.status='done';await onAgentStatus(agentId,'idle');
      }catch(error){
        const cancelled=controller.signal.aborted||error?.name==='AbortError';
        assistant.text=cancelled?'Stopped.':'Agent error: '+(error instanceof Error?error.message:String(error));
        assistant.status=cancelled?'cancelled':'error';
        await onAgentStatus(agentId,cancelled?'idle':'error');
      }finally{
        agent.updatedAt=Date.now();await save();emit('message.done',{agentId,messageId:assistant.id,text:assistant.text,status:assistant.status});
        aborts.delete(agentId);cancelApprovals(agentId,'Turn finished.');
      }
    })();
    return{messageId:assistant.id};
  }
  async function stopAgent({agentId}){
    aborts.get(agentId)?.abort();cancelApprovals(agentId,'Stopped by user.');
    const agent=await findAgent(agentId);
    if(agent){agent.status='idle';agent.updatedAt=Date.now();await save();emit('agent.changed',{agentId,status:'idle'})}
    return{ok:true};
  }

  async function listMcpServers(){
    const s=await load();
    return (s.mcpServers||[]).map(server=>({id:server.id,name:server.name,command:server.command,args:[...(server.args||[])],enabled:server.enabled!==false,disabledTools:[...(server.disabledTools||[])]}));
  }
  async function addMcpServer(input){
    const s=await load(),server=normalizeServer(input);
    if((s.mcpServers||[]).some(x=>x.id===server.id))throw Error('MCP server id already exists.');
    s.mcpServers.push(server);await save();emit('plugins.changed');return server;
  }
  async function removeMcpServer({serverId}){
    const s=await load(),before=s.mcpServers.length;
    s.mcpServers=s.mcpServers.filter(x=>x.id!==serverId);
    if(s.mcpServers.length===before)throw Error('MCP server not found.');
    mcp.disposeServer(serverId);await save();emit('plugins.changed');return{ok:true};
  }
  async function setMcpServerEnabled({serverId,enabled}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    server.enabled=enabled===true;if(!server.enabled)mcp.disposeServer(serverId);await save();emit('plugins.changed');return server;
  }
  async function listMcpServerTools({serverId}){return await mcp.listServerTools(serverId)}
  async function setMcpToolEnabled({serverId,toolName,enabled}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    const disabled=new Set(server.disabledTools||[]);if(enabled)disabled.delete(toolName);else disabled.add(toolName);
    server.disabledTools=[...disabled];await save();emit('plugins.changed');return await mcp.listServerTools(serverId);
  }
  async function listPlugins(){
    const s=await load();
    const local=capabilityCatalog.map(p=>({...p,installed:true,enabled:s.plugins?.[p.id]?.enabled!==false,removable:false,kind:'local'}));
    const servers=(s.mcpServers||[]).map(server=>({
      id:'mcp:'+server.id,name:server.name,description:'MCP server: '+server.command,category:'MCP',builtin:false,
      provider:'stdio-mcp',installed:true,enabled:server.enabled!==false,removable:true,kind:'mcp',serverId:server.id
    }));
    return[...local,...servers];
  }
  async function setPluginInstalled({pluginId,installed}){
    if(String(pluginId).startsWith('mcp:')){
      if(installed!==false)throw Error('MCP servers are installed through Add MCP Server.');
      return removeMcpServer({serverId:String(pluginId).slice(4)}).then(()=>listPlugins());
    }
    if(!catalogIds.has(pluginId))throw Error('Plugin not found');
    if(installed===false)throw Error('Built-in local capabilities cannot be removed; disable them instead.');
    return listPlugins();
  }
  async function setPluginEnabled({pluginId,enabled}){
    if(String(pluginId).startsWith('mcp:')){
      await setMcpServerEnabled({serverId:String(pluginId).slice(4),enabled});return listPlugins();
    }
    const s=await load();if(!catalogIds.has(pluginId))throw Error('Plugin not found');
    s.plugins[pluginId]={installed:true,enabled:enabled===true};await save();emit('plugins.changed');return listPlugins();
  }
  async function listWorkflows(){return await workflowManager.list()}
  async function saveWorkflow(input){const record=await workflowManager.saveWorkflow(input);emit('workflows.changed',{workflowId:record.id});return record}
  async function deleteWorkflow(input){const result=await workflowManager.deleteWorkflow(input);emit('workflows.changed',{workflowId:input.id});return result}
  async function setWorkflowEnabled(input){const record=await workflowManager.setWorkflowEnabled(input);emit('workflows.changed',{workflowId:record.id});return record}

  async function getRuntimeSettings(){
    const s=await load();return{localToolPermission:s.settings.localToolPermission,computerTarget:'local-mac'};
  }
  async function setLocalToolPermission({permission}){
    if(!['always','ask','never'].includes(permission))throw Error('Permission must be always, ask, or never.');
    const s=await load();s.settings.localToolPermission=permission;await save();emit('settings.changed',{localToolPermission:permission});
    return getRuntimeSettings();
  }

  return{
    listAgents,createAgent,renameAgent,deleteAgent,getThread,sendMessage,stopAgent,
    listPlugins,setPluginInstalled,setPluginEnabled,listMcpServers,addMcpServer,removeMcpServer,setMcpServerEnabled,listMcpServerTools,setMcpToolEnabled,listWorkflows,saveWorkflow,deleteWorkflow,setWorkflowEnabled,getRuntimeSettings,setLocalToolPermission,resolveApproval
  };
}
module.exports={createCoordinatorRuntime,capabilityCatalog};
