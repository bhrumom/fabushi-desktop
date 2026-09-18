'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');
const crypto=require('node:crypto');
const {createHostRuntime}=require('./grok-host-runtime.cjs');
const {createMcpManager,normalizeServer}=require('./grok-mcp-manager.cjs');
const {createSecretStore}=require('./grok-secret-store.cjs');
const {createMcpOAuthManager,normalizeAccountKey}=require('./grok-mcp-oauth.cjs');
const {createWorkflowManager}=require('./grok-workflow-manager.cjs');
const {normalizeSchedule,isValidSchedule,computeNextRunAt,describeSchedule}=require('./grok-automation-schedule.cjs');
const {createLocalBrowserRuntime}=require('./grok-local-browser.cjs');
const {createPluginMarketplace}=require('./grok-plugin-marketplace.cjs');
const {createOutputSpiller}=require('./grok-output-spill.cjs');
const {deriveConversationOutline}=require('./grok-conversation-outline.cjs');

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
    version:6,
    agents:[{id,name:'Chief',status:'idle',createdAt:now,updatedAt:now,unread:false}],
    messages:{[id]:[]},
    plugins:defaultCapabilities(),
    settings:{localToolPermission:'ask',autoReviewMode:'enforce'},
    pendingApprovals:{},
    mcpServers:[],
    marketplaceInstalls:{},
    automations:{[id]:[]}
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
  const autoReviewMode=parsed.settings?.autoReviewMode;
  if(['off','shadow','enforce'].includes(autoReviewMode))base.settings.autoReviewMode=autoReviewMode;
  base.pendingApprovals={};
  base.mcpServers=Array.isArray(parsed.mcpServers)?parsed.mcpServers.flatMap(server=>{try{return[normalizeServer(server)]}catch{return[]}}):[];
  base.marketplaceInstalls=parsed.marketplaceInstalls&&typeof parsed.marketplaceInstalls==='object'&&!Array.isArray(parsed.marketplaceInstalls)?parsed.marketplaceInstalls:{};
  base.automations={};
  for(const agent of base.agents){
    const rows=Array.isArray(parsed.automations?.[agent.id])?parsed.automations[agent.id]:[];
    base.automations[agent.id]=rows.flatMap(value=>{
      try{
        const schedule=normalizeSchedule(value?.trigger?.schedule||'');
        if(!value||typeof value!=='object'||!String(value.id||'')||!String(value.name||'')||!String(value.prompt||'')||!isValidSchedule(schedule))return[];
        const createdAt=Number(value.createdAt)||Date.now(),lastRunAt=Number.isFinite(Number(value.lastRunAt))?Number(value.lastRunAt):null;
        return[{
          id:String(value.id),name:clean(value.name,'Routine'),prompt:String(value.prompt).slice(0,100000),
          trigger:{type:'cron',schedule},triggerDescription:describeSchedule(schedule),isEnabled:value.isEnabled!==false,
          runs:Array.isArray(value.runs)?value.runs.slice(0,100).filter(run=>run&&['running','ok','error'].includes(run.status)&&Number.isFinite(run.startedAt)):[],
          createdAt,lastRunAt,nextRunAt:Number.isFinite(Number(value.nextRunAt))?Number(value.nextRunAt):computeNextRunAt(schedule,lastRunAt??createdAt)
        }];
      }catch{return[]}
    });
  }
  return base;
}
function createCoordinatorRuntime({app,BrowserWindow,shell,safeStorage=null,pluginMarketplace=null}){
  const file=path.join(app.getPath('userData'),'grok-agent-runtime.json');
  let state=null,loading=null,writing=Promise.resolve();
  const aborts=new Map();
  const approvals=new Map();
  let automationTimer=null,automationSweep=null;
  let automationTickRunning=false,disposed=false;
  const activeTurns=new Set();

  async function load(){
    if(state)return state;
    if(!loading){
      loading=(async()=>{
        try{return normalizeState(JSON.parse(await fs.readFile(file,'utf8')))}
        catch{return initialState()}
      })().then(value=>{state=value;return value}).finally(()=>{loading=null});
    }
    return await loading;
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
  async function createAgent({name,parentAgentId=null,purpose='user',description=''}){
    const s=await load(),now=Date.now(),agent={id:crypto.randomUUID(),name:clean(name),description:String(description||'').slice(0,500),purpose:String(purpose||'user').slice(0,40),parentAgentId:parentAgentId||null,status:'idle',createdAt:now,updatedAt:now,unread:false};
    s.agents.unshift(agent);s.messages[agent.id]=[];s.automations[agent.id]=[];await save();emit('agents.changed');return agent;
  }
  async function renameAgent({agentId,name}){
    const agent=await findAgent(agentId);if(!agent)throw Error('Agent not found');
    agent.name=clean(name,agent.name);agent.updatedAt=Date.now();await save();emit('agent.changed',{agentId});return agent;
  }
  async function deleteAgent({agentId}){
    const s=await load();aborts.get(agentId)?.abort();cancelApprovals(agentId,'Agent deleted.');
    s.agents=s.agents.filter(a=>a.id!==agentId);delete s.messages[agentId];delete s.automations[agentId];localBrowser.disposeAgent(agentId);await save();emit('agents.changed');return{ok:true};
  }
  async function getThread({agentId}){
    const s=await load(),agent=s.agents.find(x=>x.id===agentId);if(!agent)throw Error('Agent not found');
    const messages=s.messages[agentId]||[];
    return{agent,messages,outline:deriveConversationOutline(messages),pendingApprovals:Object.values(s.pendingApprovals||{}).filter(x=>x.agentId===agentId)};
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
  async function onAssistantDelta({agentId,entry,delta}){
    emit('message.delta',{agentId,messageId:entry.id,text:delta,status:entry.status});
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

  const secretStore=createSecretStore({app,safeStorage});
  let mcp=null;
  const oauth=createMcpOAuthManager({
    getServer:async serverId=>(await load()).mcpServers.find(server=>server.id===serverId)||null,
    tokenStore:secretStore,
    openExternal:url=>shell.openExternal(url),
    onChanged:serverId=>{mcp?.disposeServer(serverId);emit('plugins.changed',{serverId})}
  });
  mcp=createMcpManager({
    getServers:async()=>[...((await load()).mcpServers||[])],
    getAuthorizationHeader:(server,signal)=>oauth.authorizationHeader(server,signal)
  });
  const workflowManager=createWorkflowManager({app});
  const marketplace=pluginMarketplace||createPluginMarketplace();
  const outputSpiller=createOutputSpiller({app});
  const localBrowser=createLocalBrowserRuntime({BrowserWindow});
  const host=createHostRuntime({
    shell,getLocalToolPermission,getAutoReviewMode:async()=>(await load()).settings.autoReviewMode,requestApproval,onToolState,onAgentStatus,onAssistantDelta,
    getExternalTools:()=>mcp.collectToolDefinitions(),
    executeExternalTool:(name,args)=>mcp.executeRoutedTool(name,args),
    getWorkflowContext:prompt=>workflowManager.buildAgentContext(prompt),
    spillToolOutput:(text,meta)=>outputSpiller.spillText(text,meta),
    browser:localBrowser,
    subagents:{
      async create({parentAgentId,name,prompt,background,signal}){
        const agent=await createAgent({name,parentAgentId,purpose:'subagent',description:'Delegated agent'});
        const run=await sendMessage({agentId:agent.id,text:String(prompt||'')});
        if(background)return{agentId:agent.id,status:'background'};
        const stop=()=>{void stopAgent({agentId:agent.id})};
        if(signal?.aborted)stop();else signal?.addEventListener('abort',stop,{once:true});
        try{
          const message=await waitForMessage(agent.id,run.messageId);
          return{agentId:agent.id,status:message.status==='done'?'success':'error',finalMessage:message.text||'',toolCallCount:(state?.messages?.[agent.id]||[]).filter(row=>row.role==='tool').length};
        }finally{signal?.removeEventListener('abort',stop)}
      },
      async check({agentId}){
        const thread=await getThread({agentId});
        return{agentId,status:thread.agent.status,parentAgentId:thread.agent.parentAgentId||null,recentMessages:thread.messages.slice(-8).map(row=>({role:row.role,text:row.text,status:row.status,toolName:row.toolName||null}))};
      },
      async message({parentAgentId,agentId,prompt,interrupt,signal}){
        const agent=await findAgent(agentId);if(!agent)throw Error('Sub-agent not found.');
        if(agent.parentAgentId!==parentAgentId)throw Error('Sub-agent does not belong to this parent agent.');
        if(['thinking','running','waiting'].includes(agent.status)){
          if(!interrupt)throw Error('Sub-agent is currently running. You may send the follow-up message when it has completed. If you intended to interrupt this agent, you may retry with interrupt set to true.');
          await stopAgent({agentId});
        }
        const run=await sendMessage({agentId,text:String(prompt||'')});
        const stop=()=>{void stopAgent({agentId})};if(signal?.aborted)stop();else signal?.addEventListener('abort',stop,{once:true});
        try{const message=await waitForMessage(agentId,run.messageId);return{agentId,status:message.status==='done'?'success':'error',finalMessage:message.text||''}}finally{signal?.removeEventListener('abort',stop)}
      },
      async stop({agentId}){await stopAgent({agentId});return{agentId,status:'aborted'}}
    }
  });

  async function sendMessage({agentId,text}){
    if(disposed)throw Error('Agent runtime is shutting down.');
    const s=await load(),agent=s.agents.find(x=>x.id===agentId);if(!agent)throw Error('Agent not found');
    const body=String(text||'').trim();if(!body)throw Error('Message required');
    if(['thinking','running','waiting'].includes(agent.status))throw Error('Agent already running');
    const now=Date.now(),user={id:crypto.randomUUID(),role:'user',text:body,createdAt:now,status:'done'};
    const assistant={id:crypto.randomUUID(),role:'assistant',text:'',createdAt:now+1,status:'streaming'};
    const transcript=s.messages[agentId]||(s.messages[agentId]=[]);
    transcript.push(user);agent.status='thinking';agent.updatedAt=now;await save();
    emit('message.changed',{agentId,messageId:user.id,status:'done'});emit('agent.changed',{agentId,status:'thinking'});
    const controller=new AbortController();aborts.set(agentId,controller);

    const turnPromise=(async()=>{
      try{
        await onAgentStatus(agentId,'running');
        const finalText=await host.runTurn({
          agent,history:[...transcript],transcript,enabled:enabledCapabilityIds(s),signal:controller.signal,assistantEntry:assistant
        });
        if(!transcript.includes(assistant))transcript.push(assistant);
        assistant.text=finalText;assistant.status='done';assistant.updatedAt=Date.now();await onAgentStatus(agentId,'idle');
      }catch(error){
        const cancelled=controller.signal.aborted||error?.name==='AbortError';
        if(!transcript.includes(assistant))transcript.push(assistant);
        assistant.text=cancelled?'Stopped.':'Agent error: '+(error instanceof Error?error.message:String(error));
        assistant.status=cancelled?'cancelled':'error';assistant.updatedAt=Date.now();
        await onAgentStatus(agentId,cancelled?'idle':'error');
      }finally{
        agent.updatedAt=Date.now();await save();emit('message.done',{agentId,messageId:assistant.id,text:assistant.text,status:assistant.status});
        aborts.delete(agentId);cancelApprovals(agentId,'Turn finished.');
      }
    })();
    activeTurns.add(turnPromise);void turnPromise.finally(()=>activeTurns.delete(turnPromise));
    return{messageId:assistant.id};
  }
  async function stopAgent({agentId}){
    aborts.get(agentId)?.abort();cancelApprovals(agentId,'Stopped by user.');
    const agent=await findAgent(agentId);
    if(agent){agent.status='idle';agent.updatedAt=Date.now();await save();emit('agent.changed',{agentId,status:'idle'})}
    return{ok:true};
  }

  function automationRows(s,agentId){return s.automations[agentId]||(s.automations[agentId]=[])}
  function automationSpec(input){
    const name=clean(input?.name,'Routine'),prompt=String(input?.prompt||'').trim().slice(0,100000);
    const schedule=normalizeSchedule(input?.trigger?.schedule||'');
    if(!prompt)throw Error('Automation prompt is required.');
    if(!isValidSchedule(schedule))throw Error('Automation schedule is invalid.');
    return{name,prompt,trigger:{type:'cron',schedule},triggerDescription:describeSchedule(schedule),isEnabled:input?.isEnabled!==false};
  }
  function automationNext(record,after=Date.now()){return record.isEnabled?computeNextRunAt(record.trigger.schedule,after):null}
  async function getAgentAutomations({id}){
    const s=await load();if(!s.agents.some(agent=>agent.id===id))throw Error('Agent not found');
    return automationRows(s,id).map(row=>({...row,runs:[...(row.runs||[])]}));
  }
  async function createAgentAutomation({id,spec}){
    const s=await load();if(!s.agents.some(agent=>agent.id===id))throw Error('Agent not found');
    const now=Date.now(),normalized=automationSpec(spec);
    const record={id:crypto.randomUUID(),...normalized,runs:[],createdAt:now,lastRunAt:null,nextRunAt:normalized.isEnabled?computeNextRunAt(normalized.trigger.schedule,now):null};
    automationRows(s,id).unshift(record);await save();emit('automations.changed',{agentId:id,automations:await getAgentAutomations({id})});void armAutomationTimer();return getAgentAutomations({id});
  }
  async function setAgentAutomationEnabled({id,automationId,isEnabled}){
    const s=await load(),record=automationRows(s,id).find(row=>row.id===automationId);if(!record)throw Error('Automation not found');
    record.isEnabled=isEnabled===true;record.nextRunAt=record.isEnabled?automationNext(record,Date.now()):null;
    await save();emit('automations.changed',{agentId:id,automations:await getAgentAutomations({id})});void armAutomationTimer();return getAgentAutomations({id});
  }
  async function updateAgentAutomation({id,automationId,spec}){
    const s=await load(),record=automationRows(s,id).find(row=>row.id===automationId);if(!record)throw Error('Automation not found');
    const normalized=automationSpec(spec);Object.assign(record,normalized);record.nextRunAt=normalized.isEnabled?computeNextRunAt(normalized.trigger.schedule,Date.now()):null;
    await save();emit('automations.changed',{agentId:id,automations:await getAgentAutomations({id})});void armAutomationTimer();return getAgentAutomations({id});
  }
  async function deleteAgentAutomation({id,automationId}){
    const s=await load(),rows=automationRows(s,id),before=rows.length;s.automations[id]=rows.filter(row=>row.id!==automationId);
    if(s.automations[id].length===before)throw Error('Automation not found');
    await save();emit('automations.changed',{agentId:id,automations:await getAgentAutomations({id})});void armAutomationTimer();return getAgentAutomations({id});
  }
  async function waitForMessage(agentId,messageId,timeoutMs=600000){
    const deadline=Date.now()+timeoutMs;
    while(Date.now()<deadline){
      const s=await load(),message=(s.messages[agentId]||[]).find(row=>row.id===messageId);
      if(message&&['done','error','cancelled'].includes(message.status))return message;
      await new Promise(resolve=>setTimeout(resolve,200));
    }
    throw Error('Automation run timed out.');
  }
  async function executeAutomation(agentId,automationId,event='manual'){
    const s=await load(),record=automationRows(s,agentId).find(row=>row.id===automationId);if(!record)throw Error('Automation not found');
    const run={id:crypto.randomUUID(),status:'running',startedAt:Date.now(),detail:null,event};
    record.runs=[run,...(record.runs||[])].slice(0,100);record.lastRunAt=run.startedAt;record.nextRunAt=null;
    await save();emit('automations.changed',{agentId,automations:await getAgentAutomations({id:agentId})});
    try{
      const result=await sendMessage({agentId,text:record.prompt});
      const message=await waitForMessage(agentId,result.messageId);
      run.status=message.status==='done'?'ok':'error';run.detail=String(message.text||'').slice(0,4000);
    }catch(error){
      run.status='error';run.detail=error instanceof Error?error.message:String(error);
    }finally{
      const current=await load(),fresh=automationRows(current,agentId).find(row=>row.id===automationId);
      if(fresh){
        const persisted=(fresh.runs||[]).find(item=>item.id===run.id);if(persisted)Object.assign(persisted,run);
        fresh.lastRunAt=run.startedAt;fresh.nextRunAt=fresh.isEnabled?computeNextRunAt(fresh.trigger.schedule,run.startedAt):null;
        await save();emit('automations.changed',{agentId,automations:await getAgentAutomations({id:agentId})});
      }
      void armAutomationTimer();
    }
  }
  async function runAgentAutomationNow({id,automationId}){await executeAutomation(id,automationId,'manual')}
  async function processDueAutomations(){
    if(automationTickRunning)return;automationTickRunning=true;
    try{
      const s=await load(),now=Date.now(),due=[];
      for(const agent of s.agents){
        for(const record of automationRows(s,agent.id)){
          if(record.isEnabled&&Number.isFinite(record.nextRunAt)&&record.nextRunAt<=now)due.push([agent.id,record.id]);
        }
      }
      for(const [agentId,automationId] of due)await executeAutomation(agentId,automationId,'schedule');
    }finally{automationTickRunning=false;if(!disposed)void armAutomationTimer()}
  }
  async function armAutomationTimer(){
    if(disposed)return;
    if(automationTimer){clearTimeout(automationTimer);automationTimer=null}
    const s=await load(),now=Date.now();let nearest=null,changed=false;
    for(const agent of s.agents){
      for(const record of automationRows(s,agent.id)){
        if(!record.isEnabled){record.nextRunAt=null;continue}
        if(!Number.isFinite(record.nextRunAt)){record.nextRunAt=computeNextRunAt(record.trigger.schedule,record.lastRunAt??record.createdAt??now);changed=true}
        if(Number.isFinite(record.nextRunAt)&&(nearest==null||record.nextRunAt<nearest))nearest=record.nextRunAt;
      }
    }
    if(changed)await save();
    if(nearest!=null){
      const delay=Math.max(25,Math.min(2147483647,nearest-now));
      automationTimer=setTimeout(()=>{automationTimer=null;if(disposed)return;automationSweep=processDueAutomations().finally(()=>{automationSweep=null})},delay);
      automationTimer.unref?.();
    }
  }
  void armAutomationTimer();

  async function listMcpServers(){
    const s=await load();
    return (s.mcpServers||[]).map(server=>({
      id:server.id,name:server.name,transport:server.transport||'stdio',command:server.command||'',args:[...(server.args||[])],url:server.url||'',
      enabled:server.enabled!==false,disabledTools:[...(server.disabledTools||[])],customInstructions:server.customInstructions||'',accountKey:server.accountKey||'default',accountKeys:[...(server.accountKeys||[server.accountKey||'default'])],
      oauthClientId:server.oauthClientId||'',oauthAuthorizationUrl:server.oauthAuthorizationUrl||'',oauthTokenUrl:server.oauthTokenUrl||'',
      oauthRegistrationUrl:server.oauthRegistrationUrl||'',oauthScopes:[...(server.oauthScopes||[])]
    }));
  }
  async function addMcpServer(input){
    const s=await load(),server=normalizeServer(input);
    if((s.mcpServers||[]).some(x=>x.id===server.id))throw Error('MCP server id already exists.');
    s.mcpServers.push(server);await save();emit('plugins.changed');return server;
  }
  async function updateMcpServer({serverId,...patch}){
    const s=await load(),index=s.mcpServers.findIndex(x=>x.id===serverId);if(index<0)throw Error('MCP server not found.');
    const previous=s.mcpServers[index],next=normalizeServer({...previous,...patch,id:previous.id});
    const oldAccount=previous.accountKey||'default',newAccount=next.accountKey||'default';
    const authIdentityChanged=previous.transport==='http'&&(next.transport!=='http'||previous.url!==next.url||previous.oauthClientId!==next.oauthClientId||previous.oauthAuthorizationUrl!==next.oauthAuthorizationUrl||previous.oauthTokenUrl!==next.oauthTokenUrl);
    if(previous.transport==='http'&&next.transport==='http'&&oldAccount!==newAccount&&!authIdentityChanged){
      await oauth.rename(previous.id,oldAccount,newAccount).catch(()=>{});
    }else if(previous.transport==='http'&&authIdentityChanged){
      await oauth.disconnect(previous.id,oldAccount).catch(()=>{});
    }
    s.mcpServers[index]=next;mcp.disposeServer(previous.id);await save();emit('plugins.changed',{serverId:previous.id});return next;
  }
  async function removeMcpServer({serverId}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId),before=s.mcpServers.length;
    if(!server)throw Error('MCP server not found.');
    if(server.transport==='http')await oauth.disconnect(serverId,server.accountKey||'default').catch(()=>{});
    s.mcpServers=s.mcpServers.filter(x=>x.id!==serverId);
    if(s.mcpServers.length===before)throw Error('MCP server not found.');
    mcp.disposeServer(serverId);await save();emit('plugins.changed');return{ok:true};
  }
  async function setMcpServerEnabled({serverId,enabled}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    server.enabled=enabled===true;if(!server.enabled)mcp.disposeServer(serverId);await save();emit('plugins.changed');return server;
  }
  async function getMcpAccountStatus({serverId,accountKey}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    if(server.transport!=='http')return{serverId,accountKey:'default',connected:false,expiresAt:null,scope:'',supported:false};
    const key=normalizeAccountKey(accountKey||server.accountKey||'default');
    return{...(await oauth.status(serverId,key)),supported:true};
  }
  async function listMcpAccounts({serverId}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    if(server.transport!=='http')return[];
    const keys=[...new Set(server.accountKeys||[server.accountKey||'default'])];
    return await Promise.all(keys.map(async key=>({...await oauth.status(serverId,key),supported:true,active:key===(server.accountKey||'default')})));
  }
  async function connectMcpAccount({serverId,accountKey}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    if(server.transport!=='http')throw Error('Only HTTP MCP servers support OAuth accounts.');
    const key=normalizeAccountKey(accountKey||server.accountKey||'default');
    const status=await oauth.connect(serverId,key);
    server.accountKeys=[...new Set([...(server.accountKeys||[]),key])];server.accountKey=key;await save();
    mcp.disposeServer(serverId);emit('plugins.changed',{serverId});return{...status,supported:true,active:true};
  }
  async function disconnectMcpAccount({serverId,accountKey}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    const key=normalizeAccountKey(accountKey||server.accountKey||'default');
    const status=await oauth.disconnect(serverId,key);mcp.disposeServer(serverId);emit('plugins.changed',{serverId});return{...status,supported:server.transport==='http',active:key===(server.accountKey||'default')};
  }
  async function renameMcpAccount({serverId,accountKey,newAccountKey}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    if(server.transport!=='http')throw Error('Only HTTP MCP servers support OAuth accounts.');
    const from=normalizeAccountKey(accountKey||server.accountKey||'default'),to=normalizeAccountKey(newAccountKey);
    const status=await oauth.rename(serverId,from,to);
    server.accountKeys=[...new Set((server.accountKeys||[from]).map(key=>key===from?to:key))];
    if(server.accountKey===from)server.accountKey=to;await save();mcp.disposeServer(serverId);emit('plugins.changed',{serverId});
    return{...status,supported:true,active:server.accountKey===to};
  }
  async function removeMcpAccount({serverId,accountKey}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    if(server.transport!=='http')throw Error('Only HTTP MCP servers support OAuth accounts.');
    const key=normalizeAccountKey(accountKey||server.accountKey||'default');
    await oauth.disconnect(serverId,key);
    const remaining=(server.accountKeys||[server.accountKey||'default']).filter(value=>value!==key);
    server.accountKeys=remaining.length?remaining:['default'];
    if(server.accountKey===key)server.accountKey=server.accountKeys[0];
    await save();mcp.disposeServer(serverId);emit('plugins.changed',{serverId});return await listMcpAccounts({serverId});
  }
  async function setMcpActiveAccount({serverId,accountKey}){
    const s=await load(),server=s.mcpServers.find(x=>x.id===serverId);if(!server)throw Error('MCP server not found.');
    if(server.transport!=='http')throw Error('Only HTTP MCP servers support OAuth accounts.');
    const key=normalizeAccountKey(accountKey);
    if(!(server.accountKeys||[]).includes(key))throw Error('MCP account slot not found.');
    server.accountKey=key;await save();mcp.disposeServer(serverId);emit('plugins.changed',{serverId});
    return await listMcpAccounts({serverId});
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
      id:'mcp:'+server.id,name:server.name,description:server.transport==='http'?'Remote MCP: '+server.url:'MCP server: '+server.command,category:'MCP',builtin:false,
      provider:server.transport==='http'?'http-mcp':'stdio-mcp',installed:true,enabled:server.enabled!==false,removable:true,kind:'mcp',serverId:server.id,accountKey:server.accountKey||'default',transport:server.transport||'stdio'
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
  async function listMarketplacePlugins(){
    const [catalog,s]=await Promise.all([marketplace.list(),load()]);
    return{...catalog,plugins:(catalog.plugins||[]).map(plugin=>({...plugin,installed:Boolean(s.marketplaceInstalls?.[plugin.id]),install:s.marketplaceInstalls?.[plugin.id]||null}))};
  }
  async function installMarketplacePlugin({entryId,values={}}){
    const catalog=await marketplace.list();
    if(!catalog.available)throw Error(catalog.reason||'Plugin marketplace provider is unavailable.');
    const plugin=(catalog.plugins||[]).find(row=>row.id===String(entryId||''));if(!plugin)throw Error('Marketplace plugin not found.');
    const missing=(plugin.fields||[]).filter(field=>field.isRequired&&!String(values?.[field.key]??field.defaultValue??'').trim());
    if(missing.length)throw Error(plugin.displayName+' requires '+missing.map(field=>field.label).join(', ')+'.');
    const s=await load();s.marketplaceInstalls??={};
    if(s.marketplaceInstalls[plugin.id])throw Error('Marketplace plugin is already installed.');
    const payload=await marketplace.install(plugin.id,values);
    const preparedServers=payload.servers.map(raw=>normalizeServer({...raw,id:raw.id||crypto.randomUUID()}));
    const existing=new Set(s.mcpServers.map(server=>server.id));
    for(const server of preparedServers)if(existing.has(server.id))throw Error('Marketplace returned duplicate MCP server id: '+server.id);
    const savedSkills=[];
    try{
      for(const skill of payload.skills){
        savedSkills.push(await workflowManager.saveWorkflow({
          name:skill.name,description:skill.description,body:skill.body,isEnabledForAgent:skill.isEnabledForAgent,disableModelInvocation:skill.disableModelInvocation,trigger:null
        }));
      }
      s.mcpServers.push(...preparedServers);
      s.marketplaceInstalls[plugin.id]={
        pluginId:plugin.id,displayName:plugin.displayName,providerUrl:marketplace.providerUrl,
        serverIds:preparedServers.map(server=>server.id),skillIds:savedSkills.map(skill=>skill.id),installedAt:Date.now()
      };
      await save();emit('plugins.changed',{marketplacePluginId:plugin.id});emit('workflows.changed',{marketplacePluginId:plugin.id});
      return await listMarketplacePlugins();
    }catch(error){
      for(const skill of savedSkills)await workflowManager.deleteWorkflow({id:skill.id}).catch(()=>{});
      throw error;
    }
  }
  async function uninstallMarketplacePlugin({entryId}){
    const s=await load(),record=s.marketplaceInstalls?.[String(entryId||'')];if(!record)throw Error('Marketplace plugin is not installed.');
    for(const serverId of record.serverIds||[]){
      const server=s.mcpServers.find(row=>row.id===serverId);
      if(server?.transport==='http')await oauth.disconnect(serverId,server.accountKey||'default').catch(()=>{});
      mcp.disposeServer(serverId);
    }
    s.mcpServers=s.mcpServers.filter(server=>!(record.serverIds||[]).includes(server.id));
    for(const skillId of record.skillIds||[])await workflowManager.deleteWorkflow({id:skillId}).catch(()=>{});
    delete s.marketplaceInstalls[String(entryId||'')];await save();
    emit('plugins.changed',{marketplacePluginId:String(entryId||'')});emit('workflows.changed',{marketplacePluginId:String(entryId||'')});
    return await listMarketplacePlugins();
  }
  async function listWorkflows(){return await workflowManager.list()}
  async function saveWorkflow(input){const record=await workflowManager.saveWorkflow(input);emit('workflows.changed',{workflowId:record.id});return record}
  async function deleteWorkflow(input){const result=await workflowManager.deleteWorkflow(input);emit('workflows.changed',{workflowId:input.id});return result}
  async function setWorkflowEnabled(input){const record=await workflowManager.setWorkflowEnabled(input);emit('workflows.changed',{workflowId:record.id});return record}

  async function getRuntimeSettings(){
    const s=await load();return{localToolPermission:s.settings.localToolPermission,autoReviewMode:s.settings.autoReviewMode,computerTarget:'local-mac'};
  }
  async function setLocalToolPermission({permission}){
    if(!['always','ask','never'].includes(permission))throw Error('Permission must be always, ask, or never.');
    const s=await load();s.settings.localToolPermission=permission;await save();emit('settings.changed',{localToolPermission:permission});
    return getRuntimeSettings();
  }
  async function setAutoReviewMode({mode}){
    if(!['off','shadow','enforce'].includes(mode))throw Error('Auto-review mode must be off, shadow, or enforce.');
    const s=await load();s.settings.autoReviewMode=mode;await save();emit('settings.changed',{autoReviewMode:mode});
    return getRuntimeSettings();
  }
  async function dispose(){
    if(disposed)return{ok:true};
    disposed=true;
    if(automationTimer){clearTimeout(automationTimer);automationTimer=null}
    for(const controller of aborts.values())controller.abort();
    for(const approval of [...approvals.values()])approval.finish(false);
    if(automationSweep)await Promise.resolve(automationSweep).catch(()=>{});
    await Promise.allSettled([...activeTurns]);
    mcp.dispose();localBrowser.dispose();
    await writing.catch(()=>{});
    return{ok:true};
  }

  return{
    listAgents,createAgent,renameAgent,deleteAgent,getThread,sendMessage,stopAgent,
    listPlugins,setPluginInstalled,setPluginEnabled,listMcpServers,addMcpServer,updateMcpServer,removeMcpServer,setMcpServerEnabled,getMcpAccountStatus,listMcpAccounts,connectMcpAccount,disconnectMcpAccount,renameMcpAccount,removeMcpAccount,setMcpActiveAccount,listMcpServerTools,setMcpToolEnabled,listMarketplacePlugins,installMarketplacePlugin,uninstallMarketplacePlugin,listWorkflows,saveWorkflow,deleteWorkflow,setWorkflowEnabled,getAgentAutomations,createAgentAutomation,setAgentAutomationEnabled,updateAgentAutomation,deleteAgentAutomation,runAgentAutomationNow,getRuntimeSettings,setLocalToolPermission,setAutoReviewMode,resolveApproval,dispose
  };
}
module.exports={createCoordinatorRuntime,capabilityCatalog};
