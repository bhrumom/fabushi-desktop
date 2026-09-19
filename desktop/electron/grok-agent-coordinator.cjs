'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');
const crypto=require('node:crypto');
const {fileURLToPath}=require('node:url');
const {createHostRuntime}=require('./grok-host-runtime.cjs');
const {listBackgroundProcesses}=require('./local-tool-executor.cjs');
const {createAgentRunner}=require('./grok-agent-runner.cjs');
const {createMcpManager,normalizeServer}=require('./grok-mcp-manager.cjs');
const {createAccountSession}=require('./grok-account-session.cjs');
const {createSecretStore}=require('./grok-secret-store.cjs');
const {createMcpOAuthManager,normalizeAccountKey}=require('./grok-mcp-oauth.cjs');
const {createWorkflowManager}=require('./grok-workflow-manager.cjs');
const {normalizeSchedule,isValidSchedule,computeNextRunAt,describeSchedule}=require('./grok-automation-schedule.cjs');
const {createLocalBrowserRuntime}=require('./grok-local-browser.cjs');
const {createPluginMarketplace}=require('./grok-plugin-marketplace.cjs');
const {createOutputSpiller}=require('./grok-output-spill.cjs');
const {deriveConversationOutline}=require('./grok-conversation-outline.cjs');
const {normalizedReactions,toggleSelfReaction,resolveReplyTarget,searchWorkspaceIndex}=require('./grok-message-interactions.cjs');
const {createMemoryStore}=require('./grok-memory-store.cjs');
const {createActionAuditor}=require('./grok-action-audit.cjs');
const {createExperimentsRuntime}=require('./grok-experiments.cjs');

const capabilityCatalog=[
  {id:'filesystem',name:'Files',description:'Read and modify files on this Mac.',category:'Computer',builtin:true,provider:'local-exec'},
  {id:'shell',name:'Terminal',description:'Run cancellable foreground and background shell processes on this Mac.',category:'Computer',builtin:true,provider:'local-exec'},
  {id:'browser',name:'Browser',description:'Open approved HTTPS pages from the desktop agent.',category:'Web',builtin:true,provider:'local-exec'},
  {id:'computer',name:'Computer',description:'Capture the screen and drive macOS input through Accessibility.',category:'Computer',builtin:true,provider:'local-exec'}
];
const catalogIds=new Set(capabilityCatalog.map(x=>x.id));
const clean=(v,f='Agent')=>String(v||'').trim().replace(/[\r\n\t]/g,' ').slice(0,80)||f;
function defaultCapabilities(){return Object.fromEntries(capabilityCatalog.map(p=>[p.id,{installed:true,enabled:true}]));}
function configuredChannelManifests(env=process.env){
  const raw=String(env.FABUSHI_CHANNELS_JSON||'').trim();if(!raw)return[];
  let parsed;try{parsed=JSON.parse(raw)}catch{throw Error('FABUSHI_CHANNELS_JSON must be valid JSON.')}
  const rows=Array.isArray(parsed)?parsed:Array.isArray(parsed?.manifests)?parsed.manifests:[];
  return rows.flatMap(value=>{
    if(!value||typeof value!=='object')return[];
    const platform=String(value.platform||'').trim().toLowerCase();if(!platform||platform==='telegram')return[];
    const displayName=String(value.displayName||platform).trim().slice(0,120);
    const blurb=String(value.blurb||'Connect this agent to '+displayName+'.').trim().slice(0,1000);
    const credentialLabel=String(value.credentialLabel||'token').trim().slice(0,120);
    const availability=value.availability==='coming-soon'?'coming-soon':'available';
    const connectGuide=String(value.connectGuide||'Follow your provider instructions to create a credential.').trim().slice(0,4000);
    const steps=Array.isArray(value.setupGuide?.steps)?value.setupGuide.steps.flatMap(step=>!step||typeof step!=='object'||!String(step.text||'').trim()?[]:[{text:String(step.text).trim().slice(0,1000),...(step.code==null?{}:{code:String(step.code).slice(0,2000)})}]):undefined;
    let verifyUrl=null;if(value.verifyUrl){try{const url=new URL(String(value.verifyUrl)),loopback=['127.0.0.1','localhost','::1','[::1]'].includes(url.hostname);if(url.protocol==='https:'||(url.protocol==='http:'&&loopback))verifyUrl=url.toString()}catch{}}
    return[{platform,displayName,blurb,credentialLabel,availability,connectGuide,...(steps?{setupGuide:{steps}}:{}),verifyUrl}];
  });
}

function initialState(){
  const now=Date.now(),id=crypto.randomUUID();
  return{
    version:6,
    agents:[{id,name:'Chief',status:'idle',createdAt:now,updatedAt:now,unread:false,pinned:false,isGroup:false,memberIds:[],avatarDataUrl:null,avatarShape:null,avatarColor:null}],
    messages:{[id]:[]},
    plugins:defaultCapabilities(),
    settings:{localToolPermission:'ask',autoReviewMode:'enforce',autoReviewAllowInstructions:[],autoReviewBlockInstructions:[],featureFlagOverrides:{}},
    pendingApprovals:{},
    mcpServers:[],
    marketplaceInstalls:{},
    automations:{[id]:[]},
    channels:{[id]:[]},
    sharing:{isEnabled:true,selfAuthId:'local-user',pendingJoinRequests:[],rooms:[],typingUsers:[]}
  };
}
function normalizeState(parsed){
  if(!parsed||typeof parsed!=='object')return initialState();
  const base=initialState();
  if(Array.isArray(parsed.agents)&&parsed.agents.length){
    base.agents=parsed.agents.map(a=>({...a,hidden:a.hidden===true,pinned:a.pinned===true,unread:a.unread===true,isGroup:a.isGroup===true,memberIds:Array.isArray(a.memberIds)?[...new Set(a.memberIds.map(id=>String(id||'')).filter(Boolean))].slice(0,6):[],isSharedRoom:a.isSharedRoom===true,avatarDataUrl:typeof a.avatarDataUrl==='string'&&a.avatarDataUrl.startsWith('data:image/')?a.avatarDataUrl:null,avatarShape:typeof a.avatarShape==='string'&&a.avatarShape?a.avatarShape:null,avatarColor:typeof a.avatarColor==='string'&&a.avatarColor?a.avatarColor:null,description:String(a.description||'').slice(0,2000),title:a.title==null?undefined:String(a.title).slice(0,240),notifyOnUpdatesEnabled:a.notifyOnUpdatesEnabled===true,status:['idle','thinking','running','waiting','error'].includes(a.status)?a.status:'idle'}));
    base.messages={};
    for(const agent of base.agents)base.messages[agent.id]=Array.isArray(parsed.messages?.[agent.id])?parsed.messages[agent.id].map(row=>({...row,...(typeof row?.replyToId==='string'&&row.replyToId?{replyToId:row.replyToId}:{}),reactions:normalizedReactions(row?.reactions)})):[];
  }
  for(const id of catalogIds){
    const previous=parsed.plugins?.[id];
    if(previous)base.plugins[id]={installed:true,enabled:previous.enabled!==false};
  }
  const permission=parsed.settings?.localToolPermission;
  if(['always','ask','never'].includes(permission))base.settings.localToolPermission=permission;
  const autoReviewMode=parsed.settings?.autoReviewMode;
  if(['off','shadow','enforce'].includes(autoReviewMode))base.settings.autoReviewMode=autoReviewMode;
  const normalizeInstructions=value=>Array.isArray(value)?value.map(item=>String(item||'').trim().slice(0,1000)).filter(Boolean).slice(0,50):[];
  base.settings.autoReviewAllowInstructions=normalizeInstructions(parsed.settings?.autoReviewAllowInstructions);
  base.settings.autoReviewBlockInstructions=normalizeInstructions(parsed.settings?.autoReviewBlockInstructions);
  base.settings.featureFlagOverrides=parsed.settings?.featureFlagOverrides&&typeof parsed.settings.featureFlagOverrides==='object'&&!Array.isArray(parsed.settings.featureFlagOverrides)?Object.fromEntries(Object.entries(parsed.settings.featureFlagOverrides).filter(([,value])=>typeof value==='boolean')):{};
  base.pendingApprovals={};
  base.mcpServers=Array.isArray(parsed.mcpServers)?parsed.mcpServers.flatMap(server=>{try{return[normalizeServer(server)]}catch{return[]}}):[];
  base.marketplaceInstalls=parsed.marketplaceInstalls&&typeof parsed.marketplaceInstalls==='object'&&!Array.isArray(parsed.marketplaceInstalls)?parsed.marketplaceInstalls:{};
  const sharing=parsed.sharing&&typeof parsed.sharing==='object'&&!Array.isArray(parsed.sharing)?parsed.sharing:{};
  base.sharing={
    isEnabled:sharing.isEnabled!==false,
    selfAuthId:String(sharing.selfAuthId||'local-user'),
    pendingJoinRequests:Array.isArray(sharing.pendingJoinRequests)?sharing.pendingJoinRequests.filter(x=>x&&typeof x==='object'&&String(x.requestId||'')&&String(x.roomId||'')).map(x=>({...x,requestId:String(x.requestId),roomId:String(x.roomId),requesterAuthId:String(x.requesterAuthId||''),requesterName:String(x.requesterName||'Guest')})):[],
    rooms:Array.isArray(sharing.rooms)?sharing.rooms.filter(x=>x&&typeof x==='object'&&String(x.roomId||'')).map(x=>({...x,roomId:String(x.roomId),name:String(x.name||'Shared room'),hostAuthId:String(x.hostAuthId||sharing.selfAuthId||'local-user'),members:Array.isArray(x.members)?x.members.filter(Boolean).map(m=>({...m,kind:m.kind==='human'?'human':'agent',authId:String(m.authId||sharing.selfAuthId||'local-user'),displayName:String(m.displayName||''),...(m.agentId?{agentId:String(m.agentId)}:{})})):[]})):[],
    typingUsers:Array.isArray(sharing.typingUsers)?sharing.typingUsers.filter(x=>x&&typeof x==='object'&&String(x.roomId||'')).map(x=>({...x,roomId:String(x.roomId),authId:String(x.authId||sharing.selfAuthId||'local-user'),name:String(x.name||'You'),expiresAtMs:Number(x.expiresAtMs)||Date.now()+5000})):[]};
  base.channels={};
  base.automations={};
  for(const agent of base.agents){
    const channelRows=Array.isArray(parsed.channels?.[agent.id])?parsed.channels[agent.id]:[];
    base.channels[agent.id]=channelRows.flatMap(row=>{
      if(!row||typeof row!=='object')return[];
      const platform=String(row.platform||'').trim().toLowerCase();if(!platform||platform==='telegram')return[];
      return[{platform,label:String(row.label||platform).trim().slice(0,200),status:String(row.status||'error').slice(0,80),detail:row.detail==null?null:String(row.detail).slice(0,1000)}];
    });
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
function createCoordinatorRuntime({app,BrowserWindow,shell,safeStorage=null,pluginMarketplace=null,attachmentGateway=null,notify=async()=>{}}){
  const file=path.join(app.getPath('userData'),'grok-agent-runtime.json');
  let state=null,loading=null,writing=Promise.resolve();
  const aborts=new Map();
  const approvals=new Map();
  let automationTimer=null,automationSweep=null;
  let automationTickRunning=false,disposed=false;
  const activeTurns=new Set();
  const runners=new Map();

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
  async function listAgents(){const s=await load();return[...s.agents].sort((a,b)=>(Number(b.pinned)-Number(a.pinned))||(b.updatedAt-a.updatedAt))}
  async function createAgent({name,parentAgentId=null,purpose='user',description='',isGroup=false,memberAgentIds=[]}){
    const s=await load(),now=Date.now(),group=isGroup===true;
    const requested=Array.isArray(memberAgentIds)?[...new Set(memberAgentIds.map(id=>String(id||'')).filter(Boolean))]:[];
    if(group){
      if(requested.length<1||requested.length>6)throw Error('Groups must contain between 1 and 6 agents.');
      for(const id of requested){const member=s.agents.find(row=>row.id===id);if(!member||member.isGroup)throw Error('Group members must be existing non-group agents.')}
    }
    const agent={id:crypto.randomUUID(),name:clean(name),description:String(description||'').slice(0,2000),title:undefined,notifyOnUpdatesEnabled:false,purpose:group?'group':String(purpose||'user').slice(0,40),parentAgentId:parentAgentId||null,hidden:false,pinned:false,isGroup:group,memberIds:group?requested:[],isSharedRoom:false,status:'idle',createdAt:now,updatedAt:now,unread:false,avatarDataUrl:null,avatarShape:null,avatarColor:null};
    s.agents.unshift(agent);s.messages[agent.id]=[];s.automations[agent.id]=[];s.channels[agent.id]=[];await save();emit('agents.changed');return agent;
  }
  async function renameAgent({agentId,name}){
    const agent=await findAgent(agentId);if(!agent)throw Error('Agent not found');
    agent.name=clean(name,agent.name);agent.updatedAt=Date.now();await save();emit('agent.changed',{agentId});return agent;
  }
  async function updateAgent({id,profile,avatarShape,avatarColor}){
    const agent=await findAgent(id);if(!agent)throw Error('Agent not found');
    if(profile&&typeof profile==='object'){
      const nextName=clean(profile.name,agent.name);if(!nextName)throw Error('Agent name is required.');
      agent.name=nextName;agent.description=String(profile.description??agent.description??'').trim().slice(0,2000);
      const title=String(profile.title??'').trim().slice(0,240);agent.title=title||undefined;
    }
    if(avatarShape!==undefined)agent.avatarShape=String(avatarShape||'').trim().slice(0,40)||null;
    if(avatarColor!==undefined)agent.avatarColor=String(avatarColor||'').trim().slice(0,40)||null;
    agent.updatedAt=Date.now();await save();emit('agent.changed',{agentId:id});return agent;
  }
  async function setAgentAvatarBytes({id,pngBase64}){
    const agent=await findAgent(id);if(!agent)throw Error('Agent not found');
    if(pngBase64==null||pngBase64==='')agent.avatarDataUrl=null;
    else{
      const value=String(pngBase64).replace(/\s+/g,'');
      if(!/^[A-Za-z0-9+/]+={0,2}$/.test(value))throw Error('Avatar PNG is not valid base64.');
      const bytes=Buffer.from(value,'base64');if(bytes.length===0||bytes.length>5*1024*1024)throw Error('Avatar PNG is too large.');
      const signature=bytes.subarray(0,8).toString('hex');if(signature!=='89504e470d0a1a0a')throw Error('Avatar bytes must be PNG.');
      agent.avatarDataUrl='data:image/png;base64,'+value;
    }
    agent.updatedAt=Date.now();await save();emit('agent.changed',{agentId:id,avatar:true});return agent;
  }
  async function generateAgentAvatarImage({description}){
    const prompt=String(description||'').trim();if(!prompt)throw Error('Avatar description is required.');
    const raw=String(process.env.FABUSHI_AVATAR_GENERATE_URL||'').trim();
    if(!raw)throw Error('Avatar generation is not configured. Upload an image or use a Bot character.');
    const url=new URL(raw),loopback=['127.0.0.1','localhost','::1','[::1]'].includes(url.hostname);
    if(url.protocol!=='https:'&&!(url.protocol==='http:'&&loopback))throw Error('Avatar generation endpoint must use HTTPS.');
    const token=await accountSession.getValidAccessToken().catch(()=>null);
    const headers={'content-type':'application/json',accept:'application/json'};if(token)headers.authorization='Bearer '+token;
    const response=await fetch(url,{method:'POST',headers,body:JSON.stringify({prompt})});
    if(!response.ok)throw Error('Avatar generation HTTP '+response.status);
    const body=await response.json(),dataUrl=typeof body?.dataUrl==='string'?body.dataUrl:typeof body?.pngBase64==='string'?'data:image/png;base64,'+body.pngBase64:'';
    if(!dataUrl.startsWith('data:image/'))throw Error('Avatar generation returned no image.');
    return dataUrl;
  }
  async function setAgentNotifyOnUpdates({id,isEnabled}){
    const agent=await findAgent(id);if(!agent)throw Error('Agent not found');
    agent.notifyOnUpdatesEnabled=isEnabled===true;agent.updatedAt=Date.now();await save();emit('agent.changed',{agentId:id});return agent;
  }
  async function setAgentPinned({agentId,pinned}){
    const agent=await findAgent(agentId);if(!agent)throw Error('Agent not found');
    agent.pinned=pinned===true;agent.updatedAt=Date.now();await save();emit('agents.changed',{agentId,pinned:agent.pinned});return agent;
  }
  async function setAgentUnread({agentId,unread}){
    const agent=await findAgent(agentId);if(!agent)throw Error('Agent not found');
    agent.unread=unread===true;agent.updatedAt=Date.now();await save();emit('agents.changed',{agentId,unread:agent.unread});return agent;
  }
  async function duplicateAgent({agentId}){
    const source=await findAgent(agentId);if(!source)throw Error('Agent not found');
    const duplicate=await createAgent({name:source.name+' copy',parentAgentId:source.parentAgentId||null,purpose:source.purpose||'user',description:source.description||''});
    duplicate.title=source.title;duplicate.notifyOnUpdatesEnabled=source.notifyOnUpdatesEnabled===true;duplicate.isGroup=source.isGroup===true;duplicate.memberIds=[...(source.memberIds||[])];duplicate.avatarDataUrl=source.avatarDataUrl||null;duplicate.avatarShape=source.avatarShape||null;duplicate.avatarColor=source.avatarColor||null;await save();emit('agents.changed',{agentId:duplicate.id,duplicatedFrom:agentId});return duplicate;
  }
  async function setGroupMembers({id,memberAgentIds}){
    const s=await load(),group=s.agents.find(row=>row.id===String(id||''));if(!group||group.isGroup!==true||group.isSharedRoom===true)throw Error('Local group not found.');
    const memberIds=Array.isArray(memberAgentIds)?[...new Set(memberAgentIds.map(value=>String(value||'')).filter(Boolean))]:[];
    if(memberIds.length<1||memberIds.length>6)throw Error('Groups must contain between 1 and 6 agents.');
    for(const memberId of memberIds){const member=s.agents.find(row=>row.id===memberId);if(!member||member.isGroup||member.id===group.id)throw Error('Group members must be existing non-group agents.')}
    group.memberIds=memberIds;group.updatedAt=Date.now();await save();emit('agents.changed',{agentId:group.id,groupMembers:true});return group;
  }
  async function setAgentHidden({agentId,hidden}){
    const agent=await findAgent(agentId);if(!agent)throw Error('Agent not found');
    agent.hidden=hidden===true;agent.updatedAt=Date.now();await save();emit('agents.changed',{agentId,hidden:agent.hidden});return agent;
  }
  async function deleteAgent({agentId}){
    const s=await load();aborts.get(agentId)?.abort();cancelApprovals(agentId,'Agent deleted.');
    s.agents=s.agents.filter(a=>a.id!==agentId);for(const row of s.agents)if(row.isGroup)row.memberIds=(row.memberIds||[]).filter(id=>id!==agentId);delete s.messages[agentId];delete s.automations[agentId];delete s.channels[agentId];localBrowser.disposeAgent(agentId);await runners.get(agentId)?.dispose?.();runners.delete(agentId);await save();emit('agents.changed');return{ok:true};
  }
  async function getThread({agentId}){
    const s=await load(),agent=s.agents.find(x=>x.id===agentId);if(!agent)throw Error('Agent not found');
    const messages=(s.messages[agentId]||[]).filter(row=>row.internal!==true);
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
    const notificationAgent=s.agents.find(row=>row.id===agentId);if(notificationAgent?.notifyOnUpdatesEnabled)void Promise.resolve(notify({title:notificationAgent.name,body:'Needs input: '+String(summary||toolName||'Approval required').slice(0,180)})).catch(()=>{});
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
  const accountSession=createAccountSession({secretStore,openExternal:url=>shell.openExternal(url),onChanged:status=>emit('account.changed',{status})});
  const experiments=createExperimentsRuntime({
    app,
    getAccessToken:()=>accountSession.getValidAccessToken(),
    readOverrides:async()=>({...((await load()).settings.featureFlagOverrides||{})}),
    writeOverrides:async overrides=>{const current=await load();current.settings.featureFlagOverrides={...overrides};await save();emit('settings.changed',{featureFlagOverrides:true})},
    onChanged:snapshot=>emit('experiments.changed',{snapshot})
  });
  void experiments.start().catch(()=>{});
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
  const memoryStore=createMemoryStore({app});
  const actionAuditor=createActionAuditor({app});
  const marketplace=pluginMarketplace||createPluginMarketplace();
  const outputSpiller=createOutputSpiller({app});
  const localBrowser=createLocalBrowserRuntime({BrowserWindow});
  function remoteAttachmentDescriptor(rawUrl,alt='',forcedKind=null){
    const url=new URL(String(rawUrl||'')),decoded=decodeURIComponent(url.pathname||''),name=path.basename(decoded)||'Attachment';
    const ext=path.extname(name).toLowerCase(),imageExts=new Set(['.png','.jpg','.jpeg','.gif','.webp','.avif','.svg']);
    const kind=forcedKind|| (imageExts.has(ext)?'image':ext==='.pdf'?'pdf':['.txt','.md','.json','.csv','.tsv','.xml','.html','.js','.ts','.tsx','.py','.rs','.swift','.kt'].includes(ext)?'text':'file');
    return{id:'remote:'+crypto.randomUUID(),name,mime:kind==='image'?'image/*':kind==='pdf'?'application/pdf':'application/octet-stream',size:0,kind,createdAt:Date.now(),remoteUrl:url.toString(),alt:String(alt||'').trim().slice(0,500)};
  }
  async function attachmentFromUrl(rawUrl,alt='',forcedKind=null){
    const url=new URL(String(rawUrl||''));
    if(url.protocol==='file:'){
      if(!attachmentGateway)throw Error('Attachment gateway is unavailable.');
      return await attachmentGateway.register(fileURLToPath(url));
    }
    if(url.protocol==='https:')return remoteAttachmentDescriptor(url.toString(),alt,forcedKind);
    throw Error('Attachment URL must use file:// or https://.');
  }
  async function appendVisibleAssistantMessage({agentId,type='text',content='',url=null,alt='',images=[],widget=null,secret=null,replyToId=null}){
    const s=await load(),agent=s.agents.find(row=>row.id===agentId);if(!agent)throw Error('Agent not found');
    const transcript=s.messages[agentId]||(s.messages[agentId]=[]),replyTarget=replyToId?resolveReplyTarget(transcript,replyToId):null;if(replyToId&&!replyTarget)throw Error('Reply target not found.');
    const now=Date.now(),base={id:crypto.randomUUID(),role:'assistant',createdAt:now,status:'done',...(replyTarget?{replyToId:replyTarget.id}:{})};
    let entry;
    if(type==='text'){
      const text=String(content||'').trim();if(!text)throw Error('Visible message content is required.');
      const attachments=[];for(const image of Array.isArray(images)?images:[])attachments.push(await attachmentFromUrl(image.url,image.alt,'image'));
      entry={...base,text,...(attachments.length?{attachments}:{})};
    }else if(type==='attachment'){
      const attachment=await attachmentFromUrl(url,alt);entry={...base,text:String(alt||'').trim(),attachments:[attachment]};
    }else if(type==='widget'){
      if(!widget||typeof widget!=='object')throw Error('Widget payload is required.');
      entry={...base,text:String(widget.prompt||'').trim(),widget:{...widget,options:(widget.options||[]).map(option=>({...option}))},respondedValue:null,widgetDismissed:false};
    }else if(type==='secret-request'){
      if(!secret||typeof secret!=='object')throw Error('Secret request payload is required.');
      entry={...base,text:'Requested a secret from the user securely: '+String(secret.label||''),secretRequest:{label:String(secret.label||''),description:String(secret.description||''),connector:String(secret.connector||''),field:String(secret.field||'')},secretProvided:false};
    }else throw Error('Unsupported visible message type: '+type);
    transcript.push(entry);agent.updatedAt=now;await save();emit('message.done',{agentId,messageId:entry.id,text:entry.text,status:'done'});return entry.id;
  }
  async function reactFromAgent({agentId,messageAddress,emoji}){return reactToMessage({agentId,entryId:messageAddress,emoji,userOnly:true})}
  async function applyAgentStateUpdate(input){
    const {agentId,target,action}=input,agent=await findAgent(agentId);if(!agent)throw Error('Agent not found');
    if(target==='memory')return action==='write'?memoryStore.write({agentId,content:input.fact,tier:input.tier||'log',scope:input.scope||'agent',project:input.project||null}):memoryStore.forget({agentId,content:input.fact,scope:input.scope||'agent',project:input.project||null});
    if(target==='routine'){
      if(action==='create'){const rows=await createAgentAutomation({id:agentId,spec:{name:input.name,prompt:input.prompt,trigger:{type:'cron',schedule:input.schedule},isEnabled:input.enabled!==false}});return{ok:true,detail:'Routine created: '+rows[0]?.name};}
      if(action==='update'){const automationId=String(input.id||''),existing=(await getAgentAutomations({id:agentId})).find(row=>row.id===automationId);if(!existing)throw Error('Automation not found');const rows=await updateAgentAutomation({id:agentId,automationId,spec:{name:input.name??existing.name,prompt:input.prompt??existing.prompt,trigger:{type:'cron',schedule:input.schedule??existing.trigger.schedule},isEnabled:input.enabled??existing.isEnabled}});return{ok:true,detail:'Routine updated: '+(rows.find(row=>row.id===automationId)?.name||automationId)};}
      if(action==='pause'||action==='resume'){await setAgentAutomationEnabled({id:agentId,automationId:String(input.id||''),isEnabled:action==='resume'});return{ok:true,detail:action==='resume'?'Routine resumed.':'Routine paused.'};}
      if(action==='delete'){await deleteAgentAutomation({id:agentId,automationId:String(input.id||'')});return{ok:true,detail:'Routine deleted.'};}
    }
    if(target==='workflow'){
      if(action==='write'){const record=await saveWorkflow({id:input.id,name:input.name,description:input.description||'',body:input.body,isEnabledForAgent:true});return{ok:true,detail:'Workflow saved: '+record.name+' ('+record.id+')'};}
      if(action==='delete'){await deleteWorkflow({id:String(input.id||'')});return{ok:true,detail:'Workflow deleted.'};}
    }
    if(target==='profile'&&action==='set'){await updateAgent({id:agentId,profile:{name:input.name??agent.name,title:agent.title,description:input.description??agent.description??''}});return{ok:true,detail:'Agent profile updated.'};}
    if(target==='settings'&&action==='set'){
      if(input.hidden_from_sidebar!=null)await setAgentHidden({agentId,hidden:input.hidden_from_sidebar===true});
      if(input.notify_on_updates!=null)await setAgentNotifyOnUpdates({id:agentId,isEnabled:input.notify_on_updates===true});
      return{ok:true,detail:'Agent settings updated.'};
    }
    return{ok:false,reason:'Unsupported state update.'};
  }
  const host=createHostRuntime({
    shell,getLocalToolPermission,getAutoReviewMode:async()=>(await load()).settings.autoReviewMode,getAutoReviewInstructions:async()=>{const settings=(await load()).settings;return{allowInstructions:[...(settings.autoReviewAllowInstructions||[])],blockInstructions:[...(settings.autoReviewBlockInstructions||[])]}},resolveAttachments:async ids=>attachmentGateway?attachmentGateway.resolve(ids):[],requestApproval,onToolState,onAgentStatus,onAssistantDelta,sendVisibleMessage:appendVisibleAssistantMessage,reactToConversationMessage:reactFromAgent,updateState:applyAgentStateUpdate,getMemoryContext:agentId=>memoryStore.context(agentId),auditAction:record=>actionAuditor.record(record),onTurnUsage:payload=>{if(payload.usage)actionAuditor.record({agentId:payload.agentId,turnId:payload.turnId,occurredAtMs:payload.endedAt,action:{kind:'turnUsage',...payload.usage}})},onTurnObservation:event=>{if(event.kind==='turn-ended')actionAuditor.record({agentId:event.agentId,turnId:event.turnId,occurredAtMs:event.endedAt,action:{kind:'turnSummary',outcome:event.outcome,durationMs:event.durationMs,toolCallCount:event.toolCallCount,retryCount:event.retryCount,lastTool:event.lastTool||null}})},
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

  function runnerFor(agentId){
    let runner=runners.get(agentId);
    if(runner)return runner;
    runner=createAgentRunner({
      agentId,
      runTurn:input=>host.runTurn(input),
      onLifecycle:event=>{void Promise.resolve(actionAuditor.record({agentId:event.agentId,turnId:event.requestId,occurredAtMs:event.endedAt||event.startedAt||Date.now(),action:{kind:'runnerLifecycle',type:event.type,generation:event.generation,durationMs:event.durationMs??null,interrupted:event.interrupted===true}})).catch(()=>{})},
      onStateChanged:value=>emit('agent.changed',{agentId,runner:value})
    });
    runners.set(agentId,runner);return runner;
  }
  async function registerAttachment({path}){if(!attachmentGateway)throw Error('Attachment gateway is unavailable.');return attachmentGateway.register(path)}
  async function readAttachment({id}){if(!attachmentGateway)throw Error('Attachment gateway is unavailable.');return attachmentGateway.read(id)}
  async function enqueueInternalTurn({agentId,text,replyToId=null}){
    const task=(async()=>{
      const deadline=Date.now()+60000;
      while(Date.now()<deadline){
        if(disposed)return;
        const agent=await findAgent(agentId);if(!agent)return;
        if(!['thinking','running','waiting'].includes(agent.status)){await sendMessage({agentId,text,replyToId,internal:true});return}
        await new Promise(resolve=>setTimeout(resolve,100));
      }
      throw Error('Agent did not become idle for the interaction follow-up.');
    })();
    activeTurns.add(task);void task.catch(()=>{}).finally(()=>activeTurns.delete(task));
  }
  async function respondToWidget({agentId,entryId,value}){
    const s=await load(),entry=(s.messages[agentId]||[]).find(row=>row.id===entryId&&row.widget);if(!entry||entry.widgetDismissed||entry.respondedValue!=null)return{accepted:false};
    const answer=String(value||'').trim();if(!answer)return{accepted:false};
    const allowed=(entry.widget.options||[]).some(option=>String(option.value??option.label)===answer);
    if(!allowed&&entry.widget.allowCustom!==true)return{accepted:false};
    entry.respondedValue=answer;await save();emit('message.changed',{agentId,messageId:entryId,respondedValue:answer});
    enqueueInternalTurn({agentId,text:'[The user answered your interactive question "'+String(entry.widget.prompt||'')+'" with: '+answer+']',replyToId:entryId});
    return{accepted:true};
  }
  async function dismissWidget({agentId,entryId}){
    const s=await load(),entry=(s.messages[agentId]||[]).find(row=>row.id===entryId&&row.widget);if(!entry||entry.widgetDismissed||entry.respondedValue!=null)return{accepted:false};
    entry.widgetDismissed=true;await save();emit('message.changed',{agentId,messageId:entryId,widgetDismissed:true});
    enqueueInternalTurn({agentId,text:'[The user dismissed your interactive question "'+String(entry.widget.prompt||'')+'" without answering. Treat this as a decline and continue without re-asking.]',replyToId:entryId});
    return{accepted:true};
  }
  async function submitSecret({agentId,entryId,value}){
    const secret=String(value||'').trim();if(!secret)return{accepted:false,reason:'empty'};
    if(!secretStore.encryptedAvailable())return{accepted:false,reason:'secure-storage-unavailable'};
    const s=await load(),entry=(s.messages[agentId]||[]).find(row=>row.id===entryId&&row.secretRequest);if(!entry||entry.secretProvided===true)return{accepted:false,reason:'stale'};
    const request=entry.secretRequest,key='connector-secret:'+agentId+':'+request.connector,existing=await secretStore.get(key);
    await secretStore.set(key,{...(existing&&typeof existing==='object'?existing:{}),[request.field]:secret});
    entry.secretProvided=true;await save();emit('message.changed',{agentId,messageId:entryId,secretProvided:true});
    enqueueInternalTurn({agentId,text:'[The user securely provided the requested secret: "'+request.label+'". It was written straight to secure connector credential storage; you never see the value and it is not in this conversation.]\nConfirm to the user that it is set, then continue.',replyToId:entryId});
    return{accepted:true};
  }
  function publicChannelManifest(manifest){const {verifyUrl,...value}=manifest;return value}
  async function verifyChannelCredential({agentId,manifest,token}){
    if(!manifest.verifyUrl)return{status:'error',label:manifest.displayName,detail:'This connector has no verification endpoint configured.'};
    const controller=new AbortController(),timer=setTimeout(()=>controller.abort(),12000);
    try{
      const response=await fetch(manifest.verifyUrl,{method:'POST',headers:{accept:'application/json','content-type':'application/json',authorization:'Bearer '+token},body:JSON.stringify({agentId,platform:manifest.platform}),signal:controller.signal});
      const raw=await response.text();let body={};try{body=raw?JSON.parse(raw):{}}catch{}
      if(!response.ok)return{status:'error',label:manifest.displayName,detail:'Connector verification HTTP '+response.status};
      if(body?.connected!==true&&body?.status!=='connected'&&body?.ok!==true)return{status:'error',label:manifest.displayName,detail:String(body?.detail||body?.message||'Connector did not confirm the credential.').slice(0,1000)};
      return{status:'connected',label:String(body?.label||body?.accountName||manifest.displayName).slice(0,200),detail:body?.detail==null?null:String(body.detail).slice(0,1000)};
    }catch(error){return{status:'error',label:manifest.displayName,detail:error instanceof Error?error.message:String(error)}}
    finally{clearTimeout(timer)}
  }
  async function getAgentChannels({id}){
    const agentId=String(id||'');if(!await findAgent(agentId))throw Error('Agent not found.');
    const manifests=configuredChannelManifests(),configured=new Set(manifests.map(row=>row.platform)),state=await load();
    return{manifests:manifests.map(publicChannelManifest),connections:(state.channels[agentId]||[]).filter(row=>configured.has(row.platform))};
  }
  async function connectChannel({id,platform,token}){
    const agentId=String(id||''),key=String(platform||'').trim().toLowerCase(),secret=String(token||'').trim();
    if(!await findAgent(agentId))throw Error('Agent not found.');if(key==='telegram')throw Error('Telegram channels are disabled for this build.');
    const manifest=configuredChannelManifests().find(row=>row.platform===key);if(!manifest||manifest.availability!=='available')throw Error('Channel is unavailable.');
    if(!secret)throw Error('Channel credential is required.');if(!secretStore.encryptedAvailable())throw Error('Secure credential storage is unavailable on this Mac.');
    await secretStore.set('agent-channel:'+agentId+':'+key,{token:secret});
    const result=await verifyChannelCredential({agentId,manifest,token:secret}),state=await load(),rows=state.channels[agentId]||(state.channels[agentId]=[]);
    const next={platform:key,label:result.label,status:result.status,detail:result.detail??null},index=rows.findIndex(row=>row.platform===key);if(index>=0)rows[index]=next;else rows.push(next);
    await save();emit('agent.changed',{agentId,channels:true});return await getAgentChannels({id:agentId});
  }
  async function refreshChannel({id,platform}){
    const agentId=String(id||''),key=String(platform||'').trim().toLowerCase();
    if(!await findAgent(agentId))throw Error('Agent not found.');if(key==='telegram')throw Error('Telegram channels are disabled for this build.');
    const manifest=configuredChannelManifests().find(row=>row.platform===key);if(!manifest)throw Error('Channel is unavailable.');
    const stored=await secretStore.get('agent-channel:'+agentId+':'+key),token=String(stored?.token||'');if(!token)throw Error('Channel credential is missing.');
    const result=await verifyChannelCredential({agentId,manifest,token}),state=await load(),rows=state.channels[agentId]||(state.channels[agentId]=[]);
    const next={platform:key,label:result.label,status:result.status,detail:result.detail??null},index=rows.findIndex(row=>row.platform===key);if(index>=0)rows[index]=next;else rows.push(next);
    await save();emit('agent.changed',{agentId,channels:true});return await getAgentChannels({id:agentId});
  }
  async function disconnectChannel({id,platform}){
    const agentId=String(id||''),key=String(platform||'').trim().toLowerCase();if(!await findAgent(agentId))throw Error('Agent not found.');
    await secretStore.remove('agent-channel:'+agentId+':'+key);
    const state=await load();state.channels[agentId]=(state.channels[agentId]||[]).filter(row=>row.platform!==key);await save();emit('agent.changed',{agentId,channels:true});
    return await getAgentChannels({id:agentId});
  }

  async function reactToMessage({agentId,entryId,emoji,userOnly=false}){
    const s=await load(),agent=s.agents.find(row=>row.id===agentId);if(!agent)throw Error('Agent not found');
    const message=(s.messages[agentId]||[]).find(row=>row.id===entryId);if(!message||(message.role!=='user'&&message.role!=='assistant')||(userOnly&&message.role!=='user'))throw Error('Message not found.');
    const reactions=toggleSelfReaction(message,emoji);await save();emit('message.changed',{agentId,messageId:entryId,reactions});return{reactions:[...reactions]};
  }
  async function searchMessages({query='',limit=50}={}){const s=await load();return searchWorkspaceIndex({agents:s.agents,messages:s.messages,query,limit}).messages}
  async function searchMedia({query='',limit=50}={}){const s=await load();return searchWorkspaceIndex({agents:s.agents,messages:s.messages,query,limit}).media}
  async function searchLinks({query='',limit=50}={}){const s=await load();return searchWorkspaceIndex({agents:s.agents,messages:s.messages,query,limit}).links}
  async function sendMessage({agentId,text,attachmentIds=[],replyToId=null,internal=false}){
    if(disposed)throw Error('Agent runtime is shutting down.');
    const s=await load(),agent=s.agents.find(x=>x.id===agentId);if(!agent)throw Error('Agent not found');
    const body=String(text||'').trim();
    const attachments=attachmentGateway?await attachmentGateway.resolve(Array.isArray(attachmentIds)?attachmentIds:[]):[];
    if(!body&&!attachments.length)throw Error('Message or attachment required');
    if(['thinking','running','waiting'].includes(agent.status))throw Error('Agent already running');
    const transcript=s.messages[agentId]||(s.messages[agentId]=[]);
    if(!internal){
      for(const row of transcript){if(row.widget&&row.respondedValue==null&&row.widgetDismissed!==true&&row.widget.dismissOnMoveOn===true)row.widgetDismissed=true}
    }
    const replyTarget=replyToId?resolveReplyTarget(transcript,replyToId):null;if(replyToId&&!replyTarget)throw Error('Reply target not found.');
    const now=Date.now(),turnStartIndex=transcript.length,user={id:crypto.randomUUID(),role:'user',text:body,createdAt:now,status:'done',attachments:attachments.map(row=>({id:row.id,name:row.name,mime:row.mime,size:row.size,kind:row.kind,createdAt:row.createdAt})),...(replyTarget?{replyToId:replyTarget.id}:{}),...(internal?{internal:true}:{})};
    const assistant={id:crypto.randomUUID(),role:'assistant',text:'',createdAt:now+1,status:'streaming'};
    transcript.push(user);agent.status='thinking';agent.updatedAt=now;await save();
    if(!internal)emit('message.changed',{agentId,messageId:user.id,status:'done'});emit('agent.changed',{agentId,status:'thinking'});
    const controller=new AbortController();aborts.set(agentId,controller);

    const turnPromise=(async()=>{
      try{
        await onAgentStatus(agentId,'running');
        const runnerResult=await runnerFor(agentId).run({
          agent,history:[...transcript],transcript,enabled:enabledCapabilityIds(s),signal:controller.signal,assistantEntry:assistant
        });
        const finalText=runnerResult.value;
        if(finalText!=null){if(!transcript.includes(assistant))transcript.push(assistant);assistant.text=finalText;assistant.status='done';assistant.updatedAt=Date.now();}
        else{const visible=transcript.slice(turnStartIndex+1).filter(row=>row.role==='assistant'&&row.internal!==true&&row.id!==assistant.id);const lastVisible=visible.at(-1);assistant.text=String(lastVisible?.text||'');assistant.status='done';assistant.internal=true;assistant.updatedAt=Date.now();if(!transcript.includes(assistant))transcript.push(assistant)}
        await onAgentStatus(agentId,'idle');
      }catch(error){
        const cancelled=controller.signal.aborted||error?.name==='AbortError';
        if(!transcript.includes(assistant))transcript.push(assistant);
        assistant.text=cancelled?'Stopped.':'Agent error: '+(error instanceof Error?error.message:String(error));
        assistant.status=cancelled?'cancelled':'error';assistant.updatedAt=Date.now();
        await onAgentStatus(agentId,cancelled?'idle':'error');
      }finally{
        agent.updatedAt=Date.now();await save();if(transcript.includes(assistant)&&assistant.internal!==true)emit('message.done',{agentId,messageId:assistant.id,text:assistant.text,status:assistant.status});
        if(agent.notifyOnUpdatesEnabled)void Promise.resolve(notify({title:agent.name,body:assistant.status==='done'?'Finished':assistant.text.slice(0,180)})).catch(()=>{});
        aborts.delete(agentId);cancelApprovals(agentId,'Turn finished.');
      }
    })();
    activeTurns.add(turnPromise);void turnPromise.finally(()=>activeTurns.delete(turnPromise));
    return{messageId:assistant.id};
  }
  async function stopAgent({agentId}){
    runnerFor(agentId).interrupt('Stopped by user.');aborts.get(agentId)?.abort();cancelApprovals(agentId,'Stopped by user.');
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

  function cloneSharingState(s){
    return{
      isEnabled:s.sharing.isEnabled!==false,
      selfAuthId:String(s.sharing.selfAuthId||'local-user'),
      pendingJoinRequests:(s.sharing.pendingJoinRequests||[]).map(x=>({...x})),
      rooms:(s.sharing.rooms||[]).map(room=>({...room,members:(room.members||[]).map(member=>({...member}))})),
      typingUsers:(s.sharing.typingUsers||[]).filter(row=>Number(row.expiresAtMs)>Date.now()).map(x=>({...x}))
    };
  }
  async function emitSharing(){
    const s=await load();s.sharing.typingUsers=(s.sharing.typingUsers||[]).filter(row=>Number(row.expiresAtMs)>Date.now());await save();const snapshot=cloneSharingState(s);emit('sharing',snapshot);return snapshot;
  }
  async function getSharingState(){const s=await load();return cloneSharingState(s)}
  async function ensureSharedRoomAgent(agentId,roomId){
    const s=await load(),agent=s.agents.find(row=>row.id===agentId);if(!agent)throw Error('Agent not found.');
    agent.isSharedRoom=true;agent.sharedRoomId=roomId;agent.updatedAt=Date.now();return agent;
  }
  async function createRoomFromAgent({agentId}){
    const s=await load(),agent=s.agents.find(row=>row.id===String(agentId||''));if(!agent)throw Error('Agent not found.');
    const existing=(s.sharing.rooms||[]).find(room=>(room.members||[]).some(member=>member.kind==='agent'&&member.agentId===agent.id&&member.authId===s.sharing.selfAuthId));
    if(existing){await ensureSharedRoomAgent(agent.id,existing.roomId);await save();emit('agents.changed',{agentId:agent.id});await emitSharing();return{status:'ok',roomId:existing.roomId,agentId:agent.id}}
    const roomId=crypto.randomUUID(),room={roomId,name:agent.name,hostAuthId:s.sharing.selfAuthId,members:[
      {kind:'human',authId:s.sharing.selfAuthId,displayName:'You'},
      {kind:'agent',authId:s.sharing.selfAuthId,agentId:agent.id,displayName:agent.name}
    ]};
    s.sharing.rooms.push(room);await ensureSharedRoomAgent(agent.id,roomId);await save();emit('agents.changed',{agentId:agent.id});await emitSharing();return{status:'ok',roomId,agentId:agent.id}
  }
  async function createSharedRoom({agents:requested=[],name='Shared room'}={}){
    const s=await load(),ids=[...new Set((Array.isArray(requested)?requested:[]).map(item=>String(item?.id||item?.agentId||item||'')).filter(Boolean))];
    const selected=ids.map(id=>s.agents.find(row=>row.id===id)).filter(Boolean).filter(row=>!row.isGroup);
    if(selected.length===0)throw Error('createSharedRoom requires at least one local agent.');
    const group=await createAgent({name:clean(name,'Shared room'),purpose:'group',isGroup:true,memberAgentIds:selected.map(row=>row.id)});
    const roomId=crypto.randomUUID();group.isSharedRoom=true;group.sharedRoomId=roomId;
    s.sharing.rooms.push({roomId,name:group.name,hostAuthId:s.sharing.selfAuthId,members:[
      {kind:'human',authId:s.sharing.selfAuthId,displayName:'You'},
      ...selected.map(row=>({kind:'agent',authId:s.sharing.selfAuthId,agentId:row.id,displayName:row.name}))
    ]});
    await save();emit('agents.changed',{agentId:group.id});await emitSharing();return{status:'ok',roomId,agentId:group.id}
  }
  async function createRoomInvite({roomId}){
    const s=await load(),room=s.sharing.rooms.find(row=>row.roomId===String(roomId||''));if(!room)return{status:'error',message:'Shared room not found.'};
    const token=crypto.randomBytes(18).toString('base64url'),expiresAtMs=Date.now()+24*60*60*1000;
    room.inviteToken=token;room.inviteExpiresAtMs=expiresAtMs;await save();
    return{status:'ok',shareUrl:'fabushi://shared-room/join?roomId='+encodeURIComponent(room.roomId)+'&token='+encodeURIComponent(token),expiresAtMs,roomId:room.roomId}
  }
  async function joinSharedRoom({link}){
    let url;try{url=new URL(String(link||''))}catch{return{status:'error',message:'Invalid shared-room link.'}}
    const roomId=String(url.searchParams.get('roomId')||''),token=String(url.searchParams.get('token')||'');
    const s=await load(),room=s.sharing.rooms.find(row=>row.roomId===roomId);
    if(!room||!token||room.inviteToken!==token||Number(room.inviteExpiresAtMs)<Date.now())return{status:'error',message:'Shared-room invite is invalid or expired.'};
    const requestId=crypto.randomUUID();
    s.sharing.pendingJoinRequests.push({requestId,roomId,requesterAuthId:s.sharing.selfAuthId,requesterName:'You'});
    await save();await emitSharing();return{status:'ok',roomId,requestId}
  }
  async function respondToRoomJoinRequest({requestId,isApproved}){
    const s=await load(),request=s.sharing.pendingJoinRequests.find(row=>row.requestId===String(requestId||''));if(!request)return cloneSharingState(s);
    s.sharing.pendingJoinRequests=s.sharing.pendingJoinRequests.filter(row=>row.requestId!==request.requestId);
    if(isApproved===true){
      const room=s.sharing.rooms.find(row=>row.roomId===request.roomId);
      if(room&&!room.members.some(member=>member.kind==='human'&&member.authId===request.requesterAuthId))room.members.push({kind:'human',authId:request.requesterAuthId,displayName:request.requesterName});
    }
    await save();return await emitSharing()
  }
  async function addOwnAgentToSharedRoom({roomId,agentId,agentName}){
    const s=await load(),room=s.sharing.rooms.find(row=>row.roomId===String(roomId||'')),agent=s.agents.find(row=>row.id===String(agentId||''));if(!room||!agent)throw Error('Shared room or agent not found.');
    if(!room.members.some(member=>member.kind==='agent'&&member.agentId===agent.id&&member.authId===s.sharing.selfAuthId))room.members.push({kind:'agent',authId:s.sharing.selfAuthId,agentId:agent.id,displayName:String(agentName||agent.name)});
    await save();return await emitSharing()
  }
  async function removeOwnAgentFromSharedRoom({roomId,agentId}){
    const s=await load(),room=s.sharing.rooms.find(row=>row.roomId===String(roomId||''));if(!room)return cloneSharingState(s);
    room.members=room.members.filter(member=>!(member.kind==='agent'&&member.authId===s.sharing.selfAuthId&&member.agentId===String(agentId||'')));
    await save();return await emitSharing()
  }
  async function setSharedRoomTyping({roomId,isTyping}){
    const s=await load(),authId=s.sharing.selfAuthId;s.sharing.typingUsers=(s.sharing.typingUsers||[]).filter(row=>!(row.roomId===String(roomId||'')&&row.authId===authId));
    if(isTyping===true)s.sharing.typingUsers.push({roomId:String(roomId||''),authId,name:'You',expiresAtMs:Date.now()+5000});
    await save();await emitSharing()
  }
  async function leaveSharedRoom({roomId,targetAuthId}){
    const s=await load(),room=s.sharing.rooms.find(row=>row.roomId===String(roomId||''));if(!room)return cloneSharingState(s);
    if(targetAuthId){room.members=room.members.filter(member=>!(member.kind==='human'&&member.authId===String(targetAuthId)))}
    else{
      room.members=room.members.filter(member=>member.authId!==s.sharing.selfAuthId);
      for(const agent of s.agents.filter(row=>row.sharedRoomId===room.roomId)){agent.isSharedRoom=false;delete agent.sharedRoomId}
    }
    await save();emit('agents.changed');return await emitSharing()
  }
  async function listRoutedMcpTools(){return (await mcp.collectToolDefinitions()).map(({_mcp,...definition})=>definition)}
  async function executeRoutedMcpTool({name,args}){const result=await mcp.executeRoutedTool(String(name||''),args&&typeof args==='object'?args:{});if(result==null)throw Error('Routed MCP tool is unavailable.');return result}

  async function getAccountStatus(){return accountSession.status()}
  async function loginAccount(){return accountSession.login()}
  async function cancelAccountLogin(){return accountSession.cancelLogin()}
  async function logoutAccount(){return accountSession.logout()}
  async function updateAccountName({name}){return accountSession.updateName(name)}
  async function getAccountAvatar(){return accountSession.getAvatar()}

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

  async function getExperimentsSnapshot(){await experiments.start();return experiments.getSnapshot()}
  async function refreshExperiments(){await experiments.start();return await experiments.refreshNow()}
  async function applyFeatureFlagOverride(input={}){await experiments.start();return await experiments.applyFeatureFlagOverrideCommand(input.command??input)}

  async function getAsyncTasks({id}){
    const parentId=String(id||'').trim();if(!parentId)throw Error('Agent id is required.');
    const current=await load();if(!current.agents.some(agent=>agent.id===parentId))throw Error('Agent not found.');
    const tasks=[];
    for(const child of current.agents){
      if(child.parentAgentId!==parentId||!['thinking','running','waiting'].includes(child.status))continue;
      const active=runners.get(child.id)?.snapshot?.().active;
      tasks.push({kind:'subagent',id:child.id,label:child.name||'Subagent',status:'running',startedAtMs:Number(active?.startedAt)||Number(child.updatedAt)||Date.now(),detail:child.status,subagentType:child.purpose||'subagent'});
    }
    for(const process of listBackgroundProcesses({ownerAgentId:parentId,runningOnly:true})){
      tasks.push({kind:'shell',id:process.id,label:String(process.command||'Background shell').slice(0,120),status:'running',startedAtMs:Number(process.startedAt)||Date.now(),detail:process.cwd||undefined});
    }
    tasks.sort((a,b)=>a.startedAtMs-b.startedAtMs);
    return tasks;
  }

  async function getRuntimeSettings(){
    const s=await load();return{localToolPermission:s.settings.localToolPermission,autoReviewMode:s.settings.autoReviewMode,autoReviewAllowInstructions:[...(s.settings.autoReviewAllowInstructions||[])],autoReviewBlockInstructions:[...(s.settings.autoReviewBlockInstructions||[])],computerTarget:'local-mac'};
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
  async function setAutoReviewInstructions({allowInstructions,blockInstructions}){
    const normalize=value=>{
      if(!Array.isArray(value))throw Error('Auto-review instructions must be arrays.');
      const rows=value.map(item=>String(item||'').trim().slice(0,1000)).filter(Boolean);
      if(rows.length>50)throw Error('Auto-review supports at most 50 rules per behavior.');
      return rows;
    };
    const s=await load();
    s.settings.autoReviewAllowInstructions=normalize(allowInstructions);
    s.settings.autoReviewBlockInstructions=normalize(blockInstructions);
    await save();emit('settings.changed',{autoReviewInstructions:true});return getRuntimeSettings();
  }
  async function dispose(){
    if(disposed)return{ok:true};
    disposed=true;
    if(automationTimer){clearTimeout(automationTimer);automationTimer=null}
    for(const runner of runners.values())await runner.dispose().catch(()=>{});runners.clear();
    for(const controller of aborts.values())controller.abort();
    for(const approval of [...approvals.values()])approval.finish(false);
    if(automationSweep)await Promise.resolve(automationSweep).catch(()=>{});
    await Promise.allSettled([...activeTurns]);
    await experiments.dispose().catch(()=>{});await accountSession.cancelLogin().catch(()=>{});mcp.dispose();localBrowser.dispose();await attachmentGateway?.dispose?.();
    await actionAuditor.flush();await writing.catch(()=>{});
    return{ok:true};
  }

  return{
    listAgents,createAgent,renameAgent,updateAgent,setAgentAvatarBytes,generateAgentAvatarImage,setAgentNotifyOnUpdates,setAgentPinned,setAgentUnread,duplicateAgent,setGroupMembers,setAgentHidden,deleteAgent,getThread,registerAttachment,readAttachment,respondToWidget,dismissWidget,submitSecret,getAgentChannels,connectChannel,disconnectChannel,refreshChannel,reactToMessage,searchMessages,searchMedia,searchLinks,sendMessage,stopAgent,
    listPlugins,setPluginInstalled,setPluginEnabled,getAccountStatus,loginAccount,cancelAccountLogin,logoutAccount,updateAccountName,getAccountAvatar,listMcpServers,addMcpServer,updateMcpServer,removeMcpServer,setMcpServerEnabled,getMcpAccountStatus,listMcpAccounts,connectMcpAccount,disconnectMcpAccount,renameMcpAccount,removeMcpAccount,setMcpActiveAccount,listMcpServerTools,setMcpToolEnabled,listRoutedMcpTools,executeRoutedMcpTool,listMarketplacePlugins,installMarketplacePlugin,uninstallMarketplacePlugin,listWorkflows,saveWorkflow,deleteWorkflow,setWorkflowEnabled,getAgentAutomations,createAgentAutomation,setAgentAutomationEnabled,updateAgentAutomation,deleteAgentAutomation,runAgentAutomationNow,getSharingState,createRoomFromAgent,createRoomInvite,joinSharedRoom,respondToRoomJoinRequest,createSharedRoom,addOwnAgentToSharedRoom,removeOwnAgentFromSharedRoom,setSharedRoomTyping,leaveSharedRoom,getAsyncTasks,getExperimentsSnapshot,refreshExperiments,applyFeatureFlagOverride,getRuntimeSettings,setLocalToolPermission,setAutoReviewMode,setAutoReviewInstructions,resolveApproval,dispose
  };
}
module.exports={createCoordinatorRuntime,capabilityCatalog};
