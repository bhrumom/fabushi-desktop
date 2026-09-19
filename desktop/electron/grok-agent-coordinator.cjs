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
const {createLocalXuserSharingRuntime}=require('./grok-reference-xuser-sharing.cjs');

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
    sharing:{isEnabled:false,selfAuthId:null,pendingJoinRequests:[],rooms:[],typingUsers:[]}
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
  base.sharing={isEnabled:false,selfAuthId:null,pendingJoinRequests:[],rooms:[],typingUsers:[]};
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
  let sharingRuntime=null;

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
    await sharingRuntime?.noteAgentDeleted(String(agentId||'')).catch(()=>{});
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
  sharingRuntime=createLocalXuserSharingRuntime({
    backendUrl:process.env.FABUSHI_SHARING_BACKEND_URL||null,
    getAccessToken:({backendUrl})=>accountSession.getValidAccessToken({backendUrl}),
    getSelfAuthId:async()=>{const status=await accountSession.status();return status.kind==='logged-in'?status.authId:null},
    manager:{
      getSharedRoomIdForAgent,listRoomAgentIds,markMirrorRoomRevoked,getAgentDisplayProfile:getSharingAgentDisplayProfile,getAgentAvatar:getSharingAgentAvatar,
      restampRoomEntry,appendSharedRoomActivityNotice,runRemoteRequestedMemberTurn,appendMirrorRoomEntry,postSharedRoomGuestMessage,ensureMirrorRoom,ensureHostedSharedRoom,findRoomAgentId,isAgentCapReached
    },
    emitSharing:state=>emit('sharing',state),
    resolveAttachment:async rawUrl=>{
      try{const url=new URL(String(rawUrl||''));if(url.protocol!=='file:')return null;const filePath=fileURLToPath(url),data=await fs.readFile(filePath),ext=path.extname(filePath).toLowerCase(),mime=ext==='.png'?'image/png':ext==='.jpg'||ext==='.jpeg'?'image/jpeg':ext==='.webp'?'image/webp':ext==='.gif'?'image/gif':null;return mime?{data:new Uint8Array(data),mimeType:mime}:null}catch{return null}
    }
  });
  void sharingRuntime.start().catch(()=>{});
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
    shell,getInferenceAccessToken:()=>accountSession.getValidAccessToken(),getLocalToolPermission,getAutoReviewMode:async()=>(await load()).settings.autoReviewMode,getAutoReviewInstructions:async()=>{const settings=(await load()).settings;return{allowInstructions:[...(settings.autoReviewAllowInstructions||[])],blockInstructions:[...(settings.autoReviewBlockInstructions||[])]}},resolveAttachments:async ids=>attachmentGateway?attachmentGateway.resolve(ids):[],requestApproval,onToolState,onAgentStatus,onAssistantDelta,sendVisibleMessage:appendVisibleAssistantMessage,reactToConversationMessage:reactFromAgent,updateState:applyAgentStateUpdate,getMemoryContext:agentId=>memoryStore.context(agentId),auditAction:record=>actionAuditor.record(record),onTurnUsage:payload=>{if(payload.usage)actionAuditor.record({agentId:payload.agentId,turnId:payload.turnId,occurredAtMs:payload.endedAt,action:{kind:'turnUsage',...payload.usage}})},onTurnObservation:event=>{
      if(event.kind==='turn-ended')actionAuditor.record({agentId:event.agentId,turnId:event.turnId,occurredAtMs:event.endedAt,action:{kind:'turnSummary',outcome:event.outcome,durationMs:event.durationMs,toolCallCount:event.toolCallCount,retryCount:event.retryCount,lastTool:event.lastTool||null}});
      if(event.kind==='tool-started'){
        const args=event.args&&typeof event.args==='object'?event.args:{};
        let action=null;
        if(event.toolName==='computer_click'&&Number.isFinite(Number(args.x))&&Number.isFinite(Number(args.y)))action={agentId:event.agentId,type:'click',x:Number(args.x),y:Number(args.y)};
        else if(event.toolName==='computer_mouse_move'&&Number.isFinite(Number(args.x))&&Number.isFinite(Number(args.y)))action={agentId:event.agentId,type:'move',x:Number(args.x),y:Number(args.y)};
        else if(event.toolName==='computer_drag'&&Number.isFinite(Number(args.toX))&&Number.isFinite(Number(args.toY)))action={agentId:event.agentId,type:'drag',x:Number(args.toX),y:Number(args.toY)};
        if(action)emit('computer-action',{...action,toolCallId:event.toolCallId,at:event.at});
      }
    },
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
  async function sendMessage({agentId,text,attachmentIds=[],replyToId=null,internal=false,publishSharedReply=false}){
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
    if(!internal&&agent.sharedRoomId)void sharingRuntime?.publishRoomEntry(agent.sharedRoomId,{kind:'message',id:user.id,role:'user',content:user.text,clientNonce:user.clientNonce,timestampMs:user.createdAt}).catch(()=>{});
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
        if(agent.sharedRoomId&&assistant.status==='done'&&assistant.text&&(!internal||publishSharedReply))void sharingRuntime?.publishRoomEntry(agent.sharedRoomId,{kind:'send-message',id:assistant.id,message:{type:'text',content:assistant.text},author:{id:agent.id,name:agent.name},streaming:false,timestampMs:assistant.updatedAt||Date.now()}).catch(()=>{});
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

  const disabledSharingState=()=>({isEnabled:false,selfAuthId:null,pendingJoinRequests:[],rooms:[],typingUsers:[]});
  async function sharingRoomAgent(roomId){
    const current=await load();
    return current.agents.find(row=>row.sharedRoomId===String(roomId||''))||null;
  }
  async function getSharedRoomIdForAgent(agentId){
    const agent=await findAgent(agentId);
    return agent?.sharedRoomId||agent?.remoteRoom?.roomId||null;
  }
  async function listRoomAgentIds(roomId){
    const current=await load(),roomAgent=current.agents.find(row=>row.sharedRoomId===String(roomId||''));
    if(roomAgent?.isGroup)return[...(roomAgent.memberIds||[])];
    return current.agents.filter(row=>row.id!==roomAgent?.id&&row.sharedRoomId===String(roomId||'')&&!row.isGroup).map(row=>row.id);
  }
  async function markMirrorRoomRevoked(roomId){
    const current=await load(),agent=current.agents.find(row=>row.sharedRoomId===String(roomId||''));if(!agent)return;
    agent.remoteRoom={...(agent.remoteRoom||{}),roomId:String(roomId),isRevoked:true};agent.updatedAt=Date.now();
    await save();emit('agents.changed',{agentId:agent.id,sharedRoom:true});
  }
  async function getSharingAgentDisplayProfile(agentId){
    const agent=await findAgent(agentId);return agent?{name:agent.name,description:agent.description||''}:null;
  }
  async function getSharingAgentAvatar(agentId){
    const agent=await findAgent(agentId);return agent?.avatarDataUrl?{dataUrl:agent.avatarDataUrl}:null;
  }
  async function restampRoomEntry({roomId,entryId,timestampMs}){
    const current=await load(),agent=current.agents.find(row=>row.sharedRoomId===String(roomId||''));if(!agent)return;
    const transcript=current.messages[agent.id]||[],entry=transcript.find(row=>row.id===entryId||row.id==='remote:'+entryId);if(!entry)return;
    entry.createdAt=Number(timestampMs)||entry.createdAt;entry.updatedAt=Number(timestampMs)||entry.updatedAt;await save();emit('message.changed',{agentId:agent.id,messageId:entry.id});
  }
  async function appendSharedRoomActivityNotice({roomId,text}){
    const current=await load(),agent=current.agents.find(row=>row.sharedRoomId===String(roomId||''));if(!agent)return;
    const entry={id:'notice-'+crypto.randomUUID(),role:'assistant',text:String(text||''),createdAt:Date.now(),updatedAt:Date.now(),status:'done',sharedRoomNotice:true};
    (current.messages[agent.id]||(current.messages[agent.id]=[])).push(entry);await save();emit('message.done',{agentId:agent.id,messageId:entry.id,text:entry.text,status:'done'});
  }
  async function runRemoteRequestedMemberTurn({agentId,systemPrompt,prompt}){
    const result=await sendMessage({agentId,text:String(systemPrompt||'')+'\n\n'+String(prompt||''),internal:true,publishSharedReply:true});
    const message=await waitForMessage(agentId,result.messageId);
    const current=await load(),stored=(current.messages[agentId]||[]).find(row=>row.id===result.messageId);if(stored){stored.internal=true;await save()}
    return message?.text?[String(message.text)]:[];
  }
  async function appendMirrorRoomEntry(args){
    const current=await load(),agent=current.agents.find(row=>row.sharedRoomId===String(args?.roomId||''));if(!agent)return false;
    const wire=args?.entry&&typeof args.entry==='object'?args.entry:{},id='remote:'+String(wire.entryId||crypto.randomUUID()),transcript=current.messages[agent.id]||(current.messages[agent.id]=[]);
    if(transcript.some(row=>row.id===id||(wire.clientNonce&&row.clientNonce===wire.clientNonce)))return true;
    const human=wire.kind==='human-message',entry={
      id,role:human?'user':'assistant',text:String(wire.text||''),createdAt:Number(wire.timestampMs)||Date.now(),updatedAt:Number(wire.timestampMs)||Date.now(),status:'done',
      ...(wire.clientNonce?{clientNonce:String(wire.clientNonce)}:{}),
      ...(human?{fromUser:{name:String(wire.authorName||'Guest'),authId:String(wire.authorAuthId||''),...(wire.authorAvatarUrl?{avatarUrl:String(wire.authorAvatarUrl)}:{})}}:{fromAgent:{name:String(wire.authorName||'Agent'),ownerAuthId:String(wire.agentOwnerAuthId||''),agentId:String(wire.agentId||'')}})
    };
    transcript.push(entry);agent.updatedAt=Date.now();await save();emit('message.done',{agentId:agent.id,messageId:entry.id,text:entry.text,status:'done'});return true;
  }
  async function postSharedRoomGuestMessage(args){
    const current=await load(),agent=current.agents.find(row=>row.sharedRoomId===String(args?.roomId||''));if(!agent)return;
    const transcript=current.messages[agent.id]||(current.messages[agent.id]=[]),entry={
      id:'remote-user:'+String(args?.clientNonce||crypto.randomUUID()),role:'user',text:String(args?.text||''),createdAt:Number(args?.timestampMs)||Date.now(),status:'done',
      ...(args?.clientNonce?{clientNonce:String(args.clientNonce)}:{}),
      fromUser:{name:String(args?.authorName||'Guest'),authId:String(args?.authorAuthId||''),...(args?.authorAvatarUrl?{avatarUrl:String(args.authorAvatarUrl)}:{})}
    };
    if(!transcript.some(row=>row.id===entry.id)){transcript.push(entry);await save();emit('message.changed',{agentId:agent.id,messageId:entry.id,status:'done'})}
    await sendMessage({agentId:agent.id,text:'[Shared room message from '+entry.fromUser.name+']\n'+entry.text,internal:true,publishSharedReply:true});
  }
  async function ensureMirrorRoom(room,selfAuthId){
    const current=await load();let agent=current.agents.find(row=>row.sharedRoomId===String(room?.roomId||''));
    if(!agent){agent=await createAgent({name:String(room?.name||'Shared room'),purpose:'shared-room'});agent=await findAgent(agent.id)}
    agent.isSharedRoom=true;agent.sharedRoomId=String(room.roomId);agent.remoteRoom={roomId:String(room.roomId),hostAuthId:String(room.hostAuthId||''),members:Array.isArray(room.members)?room.members.map(member=>({...member})):[],selfAuthId:String(selfAuthId||''),isRevoked:false};agent.name=clean(room.name,agent.name);agent.updatedAt=Date.now();
    await save();emit('agents.changed',{agentId:agent.id,sharedRoom:true});return agent.id;
  }
  async function ensureHostedSharedRoom(args){
    const current=await load(),roomId=String(args?.roomId||'');let group=current.agents.find(row=>row.sharedRoomId===roomId);
    const localMemberIds=(Array.isArray(args?.localMemberIds)?args.localMemberIds:[]).filter(id=>current.agents.some(row=>row.id===id&&!row.isGroup));
    if(!group&&!args?.isCreationAllowed)return null;
    if(!group){
      if(localMemberIds.length===0)return null;
      group=await createAgent({name:String(args?.name||'Shared room'),purpose:'group',isGroup:true,memberAgentIds:localMemberIds});
      group=await findAgent(group.id);
    }
    group.isGroup=true;group.isSharedRoom=true;group.sharedRoomId=roomId;group.memberIds=localMemberIds;group.remoteMembers=Array.isArray(args?.remoteMembers)?args.remoteMembers.map(member=>({...member})):[];group.name=clean(args?.name,group.name);group.updatedAt=Date.now();
    await save();emit('agents.changed',{agentId:group.id,sharedRoom:true});return group.id;
  }
  async function findRoomAgentId(roomId){return(await sharingRoomAgent(roomId))?.id||null}
  async function isAgentCapReached(){return (await listAgents()).length>=100}
  async function getSharingState(){return sharingRuntime?await sharingRuntime.getState():disabledSharingState()}
  async function createRoomFromAgent({agentId}){
    if(!sharingRuntime)return{status:'error',message:'Sharing backend is not configured.'};
    const result=await sharingRuntime.createRoomFromAgent(String(agentId||''));
    if(result?.status==='ok'&&result.roomId){const agent=await findAgent(agentId);if(agent){agent.isSharedRoom=true;agent.sharedRoomId=String(result.roomId);agent.updatedAt=Date.now();await save();emit('agents.changed',{agentId:agent.id,sharedRoom:true})}}
    return result;
  }
  async function createSharedRoom(args={}){return sharingRuntime?sharingRuntime.createSharedRoom(args):{status:'error',message:'Sharing backend is not configured.'}}
  async function createRoomInvite({roomId}){return sharingRuntime?sharingRuntime.createRoomInvite(String(roomId||'')):{status:'error',message:'Sharing backend is not configured.'}}
  async function joinSharedRoom({link}){return sharingRuntime?sharingRuntime.joinRoom(String(link||'')):{status:'error',message:'Sharing backend is not configured.'}}
  async function respondToRoomJoinRequest(args){return sharingRuntime?sharingRuntime.respondToJoinRequest(args):disabledSharingState()}
  async function addOwnAgentToSharedRoom(args){
    const state=sharingRuntime?await sharingRuntime.addOwnAgent(args):disabledSharingState();
    const agent=await findAgent(args?.agentId);if(agent&&args?.roomId){agent.isSharedRoom=true;agent.sharedRoomId=String(args.roomId);agent.updatedAt=Date.now();await save();emit('agents.changed',{agentId:agent.id,sharedRoom:true})}
    return state;
  }
  async function removeOwnAgentFromSharedRoom({roomId,agentId}){
    const state=sharingRuntime?await sharingRuntime.removeOwnAgent(String(roomId||''),String(agentId||'')):disabledSharingState();
    const agent=await findAgent(agentId);if(agent&&agent.sharedRoomId===String(roomId||'')){agent.isSharedRoom=false;delete agent.sharedRoomId;agent.updatedAt=Date.now();await save();emit('agents.changed',{agentId:agent.id,sharedRoom:true})}
    return state;
  }
  async function setSharedRoomTyping({roomId,isTyping}){if(sharingRuntime)await sharingRuntime.setRoomTyping(String(roomId||''),isTyping===true)}
  async function leaveSharedRoom({roomId,targetAuthId}){
    const state=sharingRuntime?await sharingRuntime.leaveSharedRoom(String(roomId||''),targetAuthId==null?undefined:String(targetAuthId)):disabledSharingState();
    if(targetAuthId==null){const current=await load();for(const agent of current.agents.filter(row=>row.sharedRoomId===String(roomId||''))){agent.isSharedRoom=false;delete agent.sharedRoomId;agent.updatedAt=Date.now()}await save();emit('agents.changed',{sharedRoom:true})}
    return state;
  }

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
    sharingRuntime?.dispose();sharingRuntime=null;
    await experiments.dispose().catch(()=>{});await accountSession.cancelLogin().catch(()=>{});mcp.dispose();localBrowser.dispose();await attachmentGateway?.dispose?.();
    await actionAuditor.flush();await writing.catch(()=>{});
    return{ok:true};
  }

  return{
    listAgents,createAgent,renameAgent,updateAgent,setAgentAvatarBytes,generateAgentAvatarImage,setAgentNotifyOnUpdates,setAgentPinned,setAgentUnread,duplicateAgent,setGroupMembers,setAgentHidden,deleteAgent,getThread,registerAttachment,readAttachment,respondToWidget,dismissWidget,submitSecret,getAgentChannels,connectChannel,disconnectChannel,refreshChannel,reactToMessage,searchMessages,searchMedia,searchLinks,sendMessage,stopAgent,
    listPlugins,setPluginInstalled,setPluginEnabled,getAccountStatus,loginAccount,cancelAccountLogin,logoutAccount,updateAccountName,getAccountAvatar,listMcpServers,addMcpServer,updateMcpServer,removeMcpServer,setMcpServerEnabled,getMcpAccountStatus,listMcpAccounts,connectMcpAccount,disconnectMcpAccount,renameMcpAccount,removeMcpAccount,setMcpActiveAccount,listMcpServerTools,setMcpToolEnabled,listRoutedMcpTools,executeRoutedMcpTool,listMarketplacePlugins,installMarketplacePlugin,uninstallMarketplacePlugin,listWorkflows,saveWorkflow,deleteWorkflow,setWorkflowEnabled,getSkillPublishTargets,publishSkill,resyncPublishedSkill,unpublishSkill,getListenerIntegrations,getListenerConnectUrl,getAgentAutomations,createAgentAutomation,setAgentAutomationEnabled,updateAgentAutomation,deleteAgentAutomation,runAgentAutomationNow,getSharingState,createRoomFromAgent,createRoomInvite,joinSharedRoom,respondToRoomJoinRequest,createSharedRoom,addOwnAgentToSharedRoom,removeOwnAgentFromSharedRoom,setSharedRoomTyping,leaveSharedRoom,getAsyncTasks,getExperimentsSnapshot,refreshExperiments,applyFeatureFlagOverride,getRuntimeSettings,setLocalToolPermission,setAutoReviewMode,setAutoReviewInstructions,resolveApproval,dispose
  };
}
module.exports={createCoordinatorRuntime,capabilityCatalog};
