'use strict';
const {contextBridge,ipcRenderer,webFrame}=require('electron');

const events=[
  'agents.changed','agent.changed','message.delta','message.changed','message.done',
  'plugins.changed','workflows.changed','automations.changed','settings.changed','account.changed','experiments.changed','sharing','computer-action','approval.requested','approval.resolved','approval.cancelled'
];
const invoke=(method,args={})=>ipcRenderer.invoke('grok-agent:'+method,args);
const refCoordinator=(method,args={})=>ipcRenderer.invoke('grok-reference:coordinator',{method,args});
const refDesktop=(method,args={})=>ipcRenderer.invoke('grok-reference:desktop',{method,args});
const noopUnsubscribe=()=>{};
const refListen=(name,listener)=>listen('grok-reference:'+name,listener);
function reconnectCoordinatorPort(){
  activePort?.close?.();activePort=null;
  if(coordinatorClaimed&&coordinatorConsumer&&typeof coordinatorConsumer.onPort==='function'){
    activePort=makeCoordinatorPort();coordinatorConsumer.onPort(activePort);
  }
}
const listen=(channel,listener)=>{
  if(typeof listener!=='function')return noopUnsubscribe;
  const fn=(_event,value)=>listener(value);ipcRenderer.on(channel,fn);return()=>ipcRenderer.off(channel,fn);
};

const grokAgent=Object.freeze({
  listAgents:()=>invoke('list-agents'),createAgent:x=>invoke('create-agent',x),renameAgent:x=>invoke('rename-agent',x),updateAgent:x=>invoke('update-agent',x),
  setAgentNotifyOnUpdates:x=>invoke('set-agent-notify',x),setAgentPinned:x=>invoke('set-agent-pinned',x),setAgentUnread:x=>invoke('set-agent-unread',x),
  duplicateAgent:x=>invoke('duplicate-agent',x),setGroupMembers:x=>invoke('set-group-members',x),setAgentHidden:x=>invoke('set-agent-hidden',x),deleteAgent:x=>invoke('delete-agent',x),
  getThread:x=>invoke('get-thread',x),sendMessage:x=>invoke('send-message',x),respondToWidget:x=>invoke('respond-to-widget',x),dismissWidget:x=>invoke('dismiss-widget',x),
  submitSecret:x=>invoke('submit-secret',x),reactToMessage:x=>invoke('react-to-message',x),searchMessages:x=>invoke('search-messages',x),searchMedia:x=>invoke('search-media',x),
  searchLinks:x=>invoke('search-links',x),stopAgent:x=>invoke('stop-agent',x),listPlugins:()=>invoke('list-plugins'),setPluginInstalled:x=>invoke('set-plugin-installed',x),
  setPluginEnabled:x=>invoke('set-plugin-enabled',x),getAccountStatus:()=>invoke('get-account-status'),loginAccount:()=>invoke('login-account'),cancelAccountLogin:()=>invoke('cancel-account-login'),
  logoutAccount:()=>invoke('logout-account'),updateAccountName:x=>invoke('update-account-name',x),getAccountAvatar:()=>invoke('get-account-avatar'),
  getAgentChannels:x=>invoke('get-agent-channels',x),connectChannel:x=>invoke('connect-channel',x),disconnectChannel:x=>invoke('disconnect-channel',x),refreshChannel:x=>invoke('refresh-channel',x),
  getAsyncTasks:x=>invoke('get-async-tasks',x),getRuntimeSettings:()=>invoke('get-runtime-settings'),getExperimentsSnapshot:()=>invoke('get-experiments-snapshot'),
  refreshExperiments:()=>invoke('refresh-experiments'),applyFeatureFlagOverride:x=>invoke('apply-feature-flag-override',x),setLocalToolPermission:x=>invoke('set-local-tool-permission',x),
  setAutoReviewMode:x=>invoke('set-auto-review-mode',x),setAutoReviewInstructions:x=>invoke('set-auto-review-instructions',x),resolveApproval:x=>invoke('resolve-approval',x),
  listMcpServers:()=>invoke('list-mcp-servers'),addMcpServer:x=>invoke('add-mcp-server',x),updateMcpServer:x=>invoke('update-mcp-server',x),removeMcpServer:x=>invoke('remove-mcp-server',x),
  setMcpServerEnabled:x=>invoke('set-mcp-server-enabled',x),getMcpAccountStatus:x=>invoke('get-mcp-account-status',x),listMcpAccounts:x=>invoke('list-mcp-accounts',x),
  connectMcpAccount:x=>invoke('connect-mcp-account',x),disconnectMcpAccount:x=>invoke('disconnect-mcp-account',x),renameMcpAccount:x=>invoke('rename-mcp-account',x),
  removeMcpAccount:x=>invoke('remove-mcp-account',x),setMcpActiveAccount:x=>invoke('set-mcp-active-account',x),listMcpServerTools:x=>invoke('list-mcp-server-tools',x),
  setMcpToolEnabled:x=>invoke('set-mcp-tool-enabled',x),listMarketplacePlugins:()=>invoke('list-marketplace-plugins'),installMarketplacePlugin:x=>invoke('install-marketplace-plugin',x),
  uninstallMarketplacePlugin:x=>invoke('uninstall-marketplace-plugin',x),listWorkflows:()=>invoke('list-workflows'),saveWorkflow:x=>invoke('save-workflow',x),deleteWorkflow:x=>invoke('delete-workflow',x),
  setWorkflowEnabled:x=>invoke('set-workflow-enabled',x),getAgentAutomations:x=>invoke('get-agent-automations',x),createAgentAutomation:x=>invoke('create-agent-automation',x),
  setAgentAutomationEnabled:x=>invoke('set-agent-automation-enabled',x),updateAgentAutomation:x=>invoke('update-agent-automation',x),deleteAgentAutomation:x=>invoke('delete-agent-automation',x),
  runAgentAutomationNow:x=>invoke('run-agent-automation-now',x),pickFile:()=>ipcRenderer.invoke('grok-agent:pick-file'),pickAvatarFile:()=>ipcRenderer.invoke('grok-agent:pick-avatar-file'),
  generateAgentAvatarImage:description=>invoke('generate-agent-avatar-image',{description}),setAgentAvatarBytes:x=>invoke('set-agent-avatar-bytes',x),readAttachment:x=>invoke('read-attachment',x),
  getDesktopInfo:()=>invoke('get-desktop-info'),getUpdateStatus:()=>invoke('get-update-status'),checkUpdate:()=>invoke('check-update'),setUpdateTrack:x=>invoke('set-update-track',x),
  setAutoUpdate:x=>invoke('set-auto-update',x),quitAndInstall:()=>invoke('quit-and-install'),submitFeedback:x=>invoke('submit-feedback',x),getOnboardingSeen:()=>invoke('get-onboarding-seen'),
  setOnboardingSeen:x=>invoke('set-onboarding-seen',x),openExternal:x=>invoke('open-external',x),onUpdateStatus:listener=>listen('grok-agent:update-status',listener),
  onDeepLink:listener=>listen('grok-agent:deep-link',listener),
  subscribe(listener){
    if(typeof listener!=='function')return noopUnsubscribe;
    const handlers=events.map(name=>{const fn=(_e,payload)=>listener({type:name,...(payload||{})});ipcRenderer.on('grok-agent:event:'+name,fn);return[name,fn]});
    return()=>handlers.forEach(([name,fn])=>ipcRenderer.off('grok-agent:event:'+name,fn));
  }
});

let activePort=null;
let trayStore=[];
function publishTrayEvent(value){void emitFamily('tray',value)}
function pushCoordinatorTray(error,requestId,method){
  const detail=String(error?.message||error||'Coordinator request failed').slice(0,1200);
  const title=method==='sendPrompt'?'Agent request failed':'Action failed';
  const existing=trayStore.find(row=>row.title===title&&row.detail===detail);
  if(existing){existing.count=(existing.count||1)+1;publishTrayEvent({type:'pushed',tray:{...existing}});return}
  const tray={kind:'error',id:'local-'+Date.now().toString(36)+'-'+Math.random().toString(36).slice(2,8),title,detail,requestId:String(requestId||''),errorKind:String(error?.code||'local_runtime_error')};
  trayStore=[...trayStore.slice(-19),tray];publishTrayEvent({type:'pushed',tray});
}
async function coordinatorCallWithLocalState(method,args,requestId){
  if(method==='getTrays')return trayStore.map(row=>({...row}));
  if(method==='dismissTray'){
    const id=String(args?.id||'');trayStore=trayStore.filter(row=>row.id!==id);publishTrayEvent({type:'dismissed',id});return;
  }
  if(method==='clearTrays'){trayStore=[];publishTrayEvent({type:'cleared'});return}
  const value=await refCoordinator(method,args);
  if((method==='getForeverBoxStatus'||method==='ensureForeverBox')&&value&&typeof value==='object'){
    void emitFamily('forever-box',value);
    if(value.diskPressure!=null)void emitFamily('box-disk-pressure',value.diskPressure);
  }
  if(method==='handBackForeverBox'){
    const id=args?.id||args?.agentId;
    if(id)void refCoordinator('getForeverBoxStatus',{id}).then(status=>{if(status){void emitFamily('forever-box',status);if(status.diskPressure!=null)void emitFamily('box-disk-pressure',status.diskPressure)}}).catch(()=>{});
  }
  return value;
}
function makeCoordinatorPort(){
  let closed=false;const messageListeners=new Set(),closeListeners=new Set();
  const emit=data=>{if(!closed)for(const fn of [...messageListeners])queueMicrotask(()=>fn({data}))};
  const close=()=>{if(closed)return;closed=true;for(const fn of [...closeListeners])queueMicrotask(()=>fn({}));messageListeners.clear();closeListeners.clear();if(activePort===port)activePort=null};
  const port={
    postMessage(frame){
      if(closed||!frame||typeof frame!=='object')return;
      if(frame.kind==='lifecycle'&&frame.phase==='hello'){queueMicrotask(()=>emit({kind:'lifecycle',phase:'ready',protocolVersion:1}));return}
      if(frame.kind==='lifecycle'&&frame.phase==='shutdown'){close();return}
      if(frame.kind==='cancel')return;
      if(frame.kind==='request'&&typeof frame.requestId==='string'&&typeof frame.method==='string'){
        coordinatorCallWithLocalState(frame.method,frame.args,frame.requestId).then(
          value=>emit({kind:'reply',requestId:frame.requestId,outcome:{status:'ok',value}}),
          error=>{pushCoordinatorTray(error,frame.requestId,frame.method);emit({kind:'reply',requestId:frame.requestId,outcome:{status:'failed',failure:{code:String(error?.code||'failed'),message:String(error?.message||error||'Coordinator request failed'),transportKind:'local-mac'}}})}
        );
      }
    },
    close,start(){},
    addEventListener(type,listener){if(type==='message')messageListeners.add(listener);else if(type==='close')closeListeners.add(listener)}
  };
  port.close=close;port.__emit=emit;return port;
}
let coordinatorClaimed=false,coordinatorConsumer=null;
const coordinatorPort=Object.freeze({
  claim(consumer){
    if(coordinatorClaimed||!consumer||typeof consumer.onPort!=='function')return null;
    coordinatorClaimed=true;coordinatorConsumer=consumer;
    return{
      request(){if(!coordinatorClaimed||!coordinatorConsumer||activePort)return;activePort=makeCoordinatorPort();coordinatorConsumer.onPort(activePort)},
      release(){coordinatorClaimed=false;coordinatorConsumer=null;activePort?.close();activePort=null}
    };
  }
});
async function emitFamily(family,payload){activePort?.__emit?.({kind:'event',family,payload})}
for(const name of events){
  ipcRenderer.on('grok-agent:event:'+name,(_event,payload)=>{
    if(name==='agents.changed'||name==='agent.changed'){
      void refCoordinator('listAgents').then(async rows=>{
        await emitFamily('agents',rows);
        if(name==='agent.changed'&&payload?.agentId){const row=rows.find(x=>x&&x.id===payload.agentId);if(row)await emitFamily('agent-upserted',row)}
        for(const parent of rows){
          const subagents=rows.filter(row=>row?.parentAgentId===parent.id);
          await emitFamily('subagents',{parentAgentId:parent.id,subagents});
          const tasks=await refCoordinator('getAsyncTasks',{id:parent.id}).catch(()=>[]);
          await emitFamily('async-tasks',{parentAgentId:parent.id,tasks:Array.isArray(tasks)?tasks:[]});
        }
        const statusIds=new Set(rows.map(row=>row?.id).filter(Boolean));
        for(const id of statusIds){
          const status=await refCoordinator('getForeverBoxStatus',{id}).catch(()=>null);
          if(status){await emitFamily('forever-box',status);if(status.diskPressure!=null)await emitFamily('box-disk-pressure',status.diskPressure)}
        }
      }).catch(()=>{});
    }
    if(name==='automations.changed')void emitFamily('automations',payload||{});
    if(name==='plugins.changed')void emitFamily('plugins',payload||{});
    if(name==='settings.changed')void emitFamily('host-settings',payload||{});
    if(name==='sharing')void emitFamily('sharing',payload||{});
    if(name==='computer-action')void emitFamily('computer-action',payload||{});
    if(name==='message.changed'||name==='message.done'||name==='message.delta'){
      void refCoordinator('listAgents').then(async rows=>{
        for(const parent of rows){
          const tasks=await refCoordinator('getAsyncTasks',{id:parent.id}).catch(()=>[]);
          await emitFamily('async-tasks',{parentAgentId:parent.id,tasks:Array.isArray(tasks)?tasks:[]});
        }
      }).catch(()=>{});
    }
  });
}

const readJson=async(key,fallback)=>{try{const raw=await refDesktop('persistence-read',{key});return raw==null?fallback:JSON.parse(raw)}catch{return fallback}};
const writeJson=(key,value)=>refDesktop('persistence-write',{key,value:JSON.stringify(value)});
const mcpState=async()=>({servers:(await invoke('list-mcp-servers')).map(server=>({...server,isEnabled:server.enabled!==false,accounts:server.accountKeys||[server.accountKey||'default']}))});
const catalog=async()=>{
  const result=await invoke('list-marketplace-plugins');
  return (result?.plugins||[]).map(p=>({id:p.id,name:p.name||p.displayName,displayName:p.displayName||p.name,description:p.description||'',category:p.category||'Plugins',homepage:p.homepage,iconUrl:p.iconUrl,...(Number.isFinite(Number(p.teamPopularity??p.popularity))?{popularity:Number(p.teamPopularity??p.popularity)}:{}),connectors:p.connectors||[],skills:p.skills||[],fields:p.fields||[],marketplace:{name:'fabushi',displayName:'Fabushi Marketplace',ownership:'user'},publisher:p.publisher||{name:p.provider||'Fabushi',displayName:p.provider||'Fabushi',isUserOwned:false}}));
};
const mcp=Object.freeze({
  list:mcpState,
  async effectivePlugins(){return (await invoke('list-plugins')).map(p=>({pluginId:p.id,name:p.name,displayName:p.name,installMode:'user',isEnabled:p.enabled!==false}))},
  catalog,teamPopularity:async()=>Object.fromEntries((await catalog()).flatMap(plugin=>Number.isFinite(Number(plugin.popularity))?[[plugin.id,Number(plugin.popularity)]]:[])),pluginLogo:url=>refDesktop('plugin-logo',{url}),
  async install(request){await invoke('install-marketplace-plugin',{entryId:request.entryId,values:request.values||{}});return mcpState()},
  async updatePluginInstall(request){await invoke('uninstall-marketplace-plugin',{entryId:request.pluginId}).catch(()=>{});await invoke('install-marketplace-plugin',{entryId:request.pluginId,values:request.values||{}});return mcpState()},
  async remove(serverId){await invoke('remove-mcp-server',{serverId});return{state:await mcpState(),removed:true}},
  async uninstallPlugin(pluginId){await invoke('uninstall-marketplace-plugin',{entryId:pluginId});return{state:await mcpState(),removed:true}},
  async authenticate(serverId,accountKey){try{const result=await invoke('connect-mcp-account',{serverId,accountKey});return result?.authorizationUrl?{status:'started',authorizationUrl:result.authorizationUrl,serverName:serverId}:{status:'already-authenticated',serverName:serverId}}catch(error){return{status:'unreachable',serverName:serverId,message:String(error?.message||error)}}},
  async renameAccount(args){await invoke('rename-mcp-account',args);return mcpState()},
  async removeAccount(args){await invoke('remove-mcp-account',args);return mcpState()},
  async setCustomInstructions(args){await invoke('update-mcp-server',{serverId:args.serverId,customInstructions:args.instructions});return mcpState()},
  async listServerTools(serverId){return (await invoke('list-mcp-server-tools',{serverId})).map(t=>({...t,isDisabled:t.enabled===false}))},
  async toggleToolDisabled(args){const tools=await invoke('list-mcp-server-tools',{serverId:args.serverId}),tool=tools.find(t=>t.name===args.toolName);await invoke('set-mcp-tool-enabled',{serverId:args.serverId,toolName:args.toolName,enabled:tool?.enabled===false});return (await invoke('list-mcp-server-tools',{serverId:args.serverId})).map(t=>({...t,isDisabled:t.enabled===false}))},
  onAuthCompleted:listener=>{if(typeof listener!=='function')return noopUnsubscribe;const fn=(_e,p)=>listener({serverId:p?.serverId||'',status:'completed'});ipcRenderer.on('grok-agent:event:plugins.changed',fn);return()=>ipcRenderer.off('grok-agent:event:plugins.changed',fn)}
});
const account=Object.freeze({
  getStatus:()=>invoke('get-account-status'),login:()=>invoke('login-account'),cancelLogin:()=>invoke('cancel-account-login'),logout:()=>invoke('logout-account'),
  updateName:name=>invoke('update-account-name',{name}),getAvatar:()=>invoke('get-account-avatar'),getWeeklyUsage:async()=>null,getUsageSummary:async()=>null,
  getPrReviewPreferences:async()=>null,getPrivacyModeEnabled:async()=>false,getSandAccess:async()=>({state:'granted',reason:'none'}),getSandAccessFresh:async()=>({state:'granted',reason:'none'}),
  invokeDashboardAction:async()=>({ok:true}),cancelTrial:async()=>({ok:false,reason:'not-applicable'}),
  onStatusChanged:listener=>{if(typeof listener!=='function')return noopUnsubscribe;const fn=()=>void invoke('get-account-status').then(listener).catch(()=>{});ipcRenderer.on('grok-agent:event:account.changed',fn);return()=>ipcRenderer.off('grok-agent:event:account.changed',fn)}
});
const experiments=Object.freeze({
  initialSnapshot:{},getSnapshot:()=>invoke('get-experiments-snapshot'),applyFeatureFlagOverride:command=>invoke('apply-feature-flag-override',{command}),refresh:()=>invoke('refresh-experiments'),
  startRpcTraceWindow:async()=>false,onChanged:listener=>{if(typeof listener!=='function')return noopUnsubscribe;const fn=()=>void invoke('get-experiments-snapshot').then(listener).catch(()=>{});ipcRenderer.on('grok-agent:event:experiments.changed',fn);return()=>ipcRenderer.off('grok-agent:event:experiments.changed',fn)}
});
const telemetry=Object.freeze(Object.fromEntries(['reportAgentLoad','reportBoxVisibility','reportSendLatency','reportHeapMetrics','reportSendAck','reportReactionAck','reportRenderTtfr','reportRenderStream','reportAgentsUnreachable','reportAccessBlocked','reportRecoveryAction','reportRebuildLifecycle','reportReconciliation','reportVncSession','reportVncLiveness','reportOpenComputer','reportUpdatePrompt','reportSigninGate','reportOnboardingStep','reportClientFailure','noteSentryConversation'].map(k=>[k,()=>{}])));
const clientPersistence=Object.freeze({
  read:key=>refDesktop('persistence-read',{key}),write:(key,value)=>refDesktop('persistence-write',{key,value}),remove:key=>refDesktop('persistence-remove',{key}),
  listKeys:prefix=>refDesktop('persistence-list',{prefix}),migrateFromLocalStorage:entries=>refDesktop('persistence-migrate',{entries})
});
const agentBridge=Object.freeze({
  async getPinnedAgents(){return (await invoke('list-agents')).filter(a=>a.pinned===true).map(a=>a.id)},
  async setPinnedAgents(ids){const wanted=new Set(ids||[]),rows=await invoke('list-agents');await Promise.all(rows.map(a=>invoke('set-agent-pinned',{agentId:a.id,pinned:wanted.has(a.id)})));return[...wanted]},
  getSidebarSections:()=>readJson('agent.sidebarSections',[]),async setSidebarSections(sections){await writeJson('agent.sidebarSections',sections);return sections},
  getDefaultModel:()=>readJson('agent.defaultModel',null),async setDefaultModel(model){await writeJson('agent.defaultModel',model);return model},
  getComputerUseModel:()=>readJson('agent.computerUseModel',null),async setComputerUseModel(model){await writeJson('agent.computerUseModel',model);return model},
  getAvailableModels:async()=>({models:[{id:'gpt-5.6-sol',name:'GPT-5.6 Sol'}]}),clientPersistence
});
const desktop=Object.freeze({
  resolveAttachmentMedia:path=>refDesktop('resolve-media',{path}),readAttachmentText:path=>refDesktop('read-text',{path}),readAttachmentBytes:(path,maxBytes)=>refDesktop('read-bytes',{path,maxBytes}),
  downloadAttachment:(path,suggestedName)=>refDesktop('download',{path,suggestedName}),getLinkMetadata:url=>refDesktop('link-metadata',{url}),openExternal:url=>invoke('open-external',{url}),
  openCloudAgent:async()=>{},stageAttachmentBytes:(filename,bytes)=>refDesktop('stage-attachment',{filename,bytes}),commitStagedAttachments:(paths,filenames)=>refDesktop('commit-staged',{paths,filenames}),
  discardStagedAttachment:path=>refDesktop('discard-staged',{path}),mcp,forceGatewayReconnect:async()=>reconnectCoordinatorPort(),pickAvatarSource:async()=>{const value=await ipcRenderer.invoke('grok-agent:pick-avatar-file');return value?.dataUrl||null;},
  async pickAvatarFile(){const value=await ipcRenderer.invoke('grok-agent:pick-avatar-file');return value?.dataUrl?{dataUrl:value.dataUrl,fileName:value.fileName}:null},
  generateAgentAvatarImage:description=>invoke('generate-agent-avatar-image',{description}),
  onFocusAgent:listener=>refListen('focus-agent',listener),onDeepLink:listener=>listen('grok-agent:deep-link',listener),deepLinksReady:async()=>{},getBoxMigrationStatus:async()=>({status:'ready',computerTarget:'local-mac'}),
  onBoxMigration:()=>noopUnsubscribe,onDevBoxRebuild:()=>noopUnsubscribe,onOpenFeedback:listener=>refListen('open-feedback',()=>listener()),onOpenAbout:listener=>refListen('open-about',()=>listener()),submitFeedback:payload=>invoke('submit-feedback',payload),
  onWidgetGallery:()=>noopUnsubscribe,onForceOnboarding:listener=>refListen('force-onboarding',()=>listener()),transcribeAudio:(audio,mimeType,language)=>refDesktop('transcribe-audio',{audio,mimeType,language}),cursorAccount:account,experiments,platform:process.platform,isDev:!process.env.NODE_ENV||process.env.NODE_ENV!=='production',
  getWindowState:()=>refDesktop('get-window-state'),onWindowStateEvent:listener=>refListen('window-state',listener),getZoomFactor:()=>webFrame.getZoomFactor(),onZoomFactorEvent:listener=>refListen('zoom-factor-changed',payload=>listener(Number(payload?.factor??payload))),
  windowControls:{minimize:()=>refDesktop('window-control',{action:'minimize'}),toggleMaximize:()=>refDesktop('window-control',{action:'toggle-maximize'}),close:()=>refDesktop('window-control',{action:'close'}),setTitleBarOverlayTone:isOverlayTone=>refDesktop('window-control',{action:'overlay-tone',isOverlayTone}),resizeWidth:deltaWidth=>refDesktop('window-control',{action:'resize-width',deltaWidth})},
  foreverBox:{forceRecreate:async()=>({state:'running',computerTarget:'local-mac'}),update:async id=>({id,state:'running',computerTarget:'local-mac',vncUrl:null}),onVncUserPresence:()=>noopUnsubscribe,onDevBoxPullProgress:()=>noopUnsubscribe,egressTunnel:{initial:false,initialStatus:{enabled:false},get:async()=>false,set:async()=>false,onChanged:()=>noopUnsubscribe,getStatus:async()=>({enabled:false}),onStatusChanged:()=>noopUnsubscribe},webauthnProxy:{initial:false,get:async()=>false,set:async()=>false,onChanged:()=>noopUnsubscribe}},
  onboarding:{getSeen:()=>invoke('get-onboarding-seen'),setSeen:seen=>invoke('set-onboarding-seen',{seen}),onSkip:listener=>refListen('skip-onboarding',()=>listener())},telemetry,timeZone:{get:()=>refDesktop('time-zone-get'),setOverride:timeZone=>refDesktop('time-zone-set',{timeZone})},
  autoReviewInstructions:{async get(){const s=await invoke('get-runtime-settings');return{isEnabled:s.autoReviewMode!=='off',allowInstructions:s.autoReviewAllowInstructions||[],blockInstructions:s.autoReviewBlockInstructions||[]}},async set(v){await invoke('set-auto-review-instructions',{allowInstructions:v.allowInstructions||[],blockInstructions:v.blockInstructions||[]});await invoke('set-auto-review-mode',{mode:v.isEnabled===false?'off':'enforce'});return v}},
  localToolPermission:{async get(){return(await invoke('get-runtime-settings')).localToolPermission},async set(permission){return(await invoke('set-local-tool-permission',{permission})).localToolPermission},ceiling:()=>refDesktop('local-tool-permission-ceiling'),recordApproval:(approvalId,action,target)=>refDesktop('local-tool-approval-record',{approvalId,action,target}),clearApprovals:()=>refDesktop('local-tool-approval-clear')},
  theme:{initial:{preference:'system',resolved:'dark'},get:()=>refDesktop('theme-get'),set:preference=>refDesktop('theme-set',{preference}),onChanged:listener=>refListen('theme-changed',listener)},
  secrets:{list:()=>refDesktop('secrets-list'),reveal:key=>refDesktop('secrets-reveal',{key}),upsert:entries=>refDesktop('secrets-upsert',{entries}),remove:keys=>refDesktop('secrets-remove',{keys})},
  agent:agentBridge,
  update:{getStatus:()=>invoke('get-update-status'),check:()=>invoke('check-update'),setTrack:track=>invoke('set-update-track',{track}),quitAndInstall:()=>invoke('quit-and-install'),setAutoUpdateWhenIdleOptIn:enabled=>invoke('set-auto-update',{enabled}),onStatusEvent:listener=>listen('grok-agent:update-status',listener)},
  attachProdBox:{getStatus:async()=>({enabled:false,computerTarget:'local-mac'}),setEnabled:async enabled=>({enabled,computerTarget:'local-mac'})}
});
contextBridge.exposeInMainWorld('grokAgent',grokAgent);
contextBridge.exposeInMainWorld('desktop',desktop);
contextBridge.exposeInMainWorld('coordinatorPort',coordinatorPort);
