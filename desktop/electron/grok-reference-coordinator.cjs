'use strict';

const fs=require('node:fs/promises');
const os=require('node:os');

function asObject(value){return value&&typeof value==='object'&&!Array.isArray(value)?value:{}}
function agentRow(agent){
  if(!agent||typeof agent!=='object')return agent;
  const running=['thinking','running','waiting'].includes(agent.status);
  return {
    ...agent,
    isPinned:agent.pinned===true,
    isHiddenFromSidebar:agent.hidden===true,
    hiddenFromSidebar:agent.hidden===true,
    hasUnread:agent.unread===true,
    isRunning:running,
    currentActivity:running?String(agent.status||'running'):null,
    awaitingUserResponse:agent.status==='waiting'?{reason:'waiting'}:null
  };
}
function transcriptEntry(row,index){
  if(!row||typeof row!=='object')return null;
  const id=String(row.id||('entry-'+index));
  const timestampMs=Number(row.timestampMs||row.createdAt||row.updatedAt)||Date.now();
  if(row.role==='tool'){
    return {kind:'tool-call',id,name:String(row.toolName||row.name||'Tool'),summary:String(row.text||row.summary||''),status:row.status==='error'?'failed':row.status==='cancelled'?'aborted':row.status==='done'?'done':'running',timestampMs};
  }
  return {
    ...row,id,timestampMs,
    role:row.role==='assistant'?'assistant':'user',
    text:String(row.text||''),
    ...(row.status==='streaming'?{isStreaming:true}:{}),
    ...(typeof row.replyToId==='string'?{replyToId:row.replyToId}:{}),
    ...(typeof row.clientNonce==='string'?{clientNonce:row.clientNonce}:{})
  };
}
function pageFromThread(thread,limit=200,beforeSeq){
  const all=(thread?.messages||[]).map(transcriptEntry).filter(Boolean);
  const end=Number.isFinite(Number(beforeSeq))?Math.max(0,Math.min(all.length,Number(beforeSeq))):all.length;
  const start=Math.max(0,end-Math.max(1,Math.min(500,Number(limit)||200)));
  return {entries:all.slice(start,end),nextBeforeSeq:start>0?start:undefined};
}
async function localComputerStatus(id){
  const agentId=String(id||'local-mac');
  let diskPressure=null;
  try{
    const stats=await fs.statfs(os.homedir()),blockSize=Number(stats.bsize)||0;
    const totalBytes=(Number(stats.blocks)||0)*blockSize;
    const freeBlocks=Number(stats.bavail??stats.bfree)||0,freeBytes=freeBlocks*blockSize;
    const usedRatio=totalBytes>0?Math.max(0,Math.min(1,1-freeBytes/totalBytes)):0;
    diskPressure={
      source:'local-mac',usedRatio,totalBytes,freeBytes,
      level:usedRatio>=0.95?'critical':usedRatio>=0.90?'high':usedRatio>=0.80?'elevated':'normal'
    };
  }catch{}
  return{agentId,state:'running',kind:'local-computer',computerTarget:'local-mac',vncUrl:null,diskPressure};
}
function workflowSpec(spec={}){
  const value=asObject(spec);
  return {
    ...(value.id?{id:String(value.id)}:{}),
    name:String(value.name||value.title||'Skill'),
    description:String(value.description||''),
    body:String(value.body||value.instructions||value.content||'# Skill\n'),
    isEnabledForAgent:value.isEnabled!==false&&value.isEnabledForAgent!==false,
    disableModelInvocation:value.disableModelInvocation===true,
    ...(value.sourceRef?{sourceRef:String(value.sourceRef)}:{}),
    ...(value.trigger?{trigger:value.trigger}:{})
  };
}
function createReferenceCoordinator(runtime){
  const teach=new Map();
  async function agents(){return (await runtime.listAgents()).map(agentRow)}
  async function call(method,args={}){
    const input=asObject(args);
    switch(method){
      case'listAgents':return agents();
      case'countAgents':return (await runtime.listAgents()).length;
      case'searchAgents':{
        const q=String(input.query||input.search||'').trim().toLowerCase();
        const rows=await agents();
        return q?rows.filter(a=>String(a.name||'').toLowerCase().includes(q)||String(a.description||'').toLowerCase().includes(q)):rows;
      }
      case'createAgent':return agentRow(await runtime.createAgent(input));
      case'createGroup':return agentRow(await runtime.createAgent({...input,isGroup:true,memberAgentIds:input.memberAgentIds||input.memberIds||[]}));
      case'setGroupMembers':return agentRow(await runtime.setGroupMembers({id:input.id,memberAgentIds:input.memberAgentIds||input.memberIds||[]}));
      case'updateAgent':return agentRow(await runtime.updateAgent(input));
      case'deleteAgents':for(const id of Array.isArray(input.ids)?input.ids:[])await runtime.deleteAgent({agentId:id});return{ok:true};
      case'duplicateAgent':return agentRow(await runtime.duplicateAgent({agentId:input.id||input.agentId}));
      case'kickstartAgent':{
        const id=input.id||input.agentId;
        const row=(await runtime.listAgents()).find(agent=>agent.id===id);if(!row)return null;
        if(!['thinking','running','waiting'].includes(row.status))await runtime.sendMessage({agentId:id,text:String(input.prompt||'Continue the current task.'),internal:true});
        return agentRow((await runtime.listAgents()).find(agent=>agent.id===id)||row);
      }
      case'setAgentUnread':await runtime.setAgentUnread({agentId:input.id||input.agentId,unread:input.isUnread===true||input.unread===true});return;
      case'setAgentHiddenFromSidebar':await runtime.setAgentHidden({agentId:input.id||input.agentId,hidden:input.isHidden===true||input.hidden===true||input.isHiddenFromSidebar===true});return;
      case'setAgentNotificationsEnabled':
      case'setAgentNotifyOnUpdates':await runtime.setAgentNotifyOnUpdates({id:input.id||input.agentId,isEnabled:input.isEnabled===true||input.enabled===true});return;
      case'setAgentAvatarBytes':return agentRow(await runtime.setAgentAvatarBytes(input));
      case'getAgentAvatar':{
        const row=(await runtime.listAgents()).find(a=>a.id===(input.id||input.agentId));
        return row?{id:row.id,dataUrl:row.avatarDataUrl||null,avatarShape:row.avatarShape||null,avatarColor:row.avatarColor||null}:null;
      }
      case'getAgentThread':
      case'getAgentTranscriptWindow':
      case'getAgentTranscriptTail':
      case'openAgentTail':{
        const id=input.id||input.agentId;
        const thread=await runtime.getThread({agentId:id});
        const page=pageFromThread(thread,input.limit,input.beforeSeq);
        if(method==='getAgentTranscriptWindow')return{...page,threadCounts:{}};
        return method==='getAgentThread'?{agent:agentRow(thread.agent),...page}:page;
      }
      case'sendPrompt':{
        const attachmentIds=[];
        const paths=Array.isArray(input.attachmentPaths)?input.attachmentPaths:[];
        for(const filePath of paths){try{const rec=await runtime.registerAttachment({path:filePath});attachmentIds.push(rec.id)}catch{}}
        const result=await runtime.sendMessage({agentId:input.agentId||input.id,text:String(input.prompt??input.text??''),attachmentIds,replyToId:input.replyToId||input.replyTo||null});
        return{accepted:true,messageId:result.messageId,clientNonce:input.clientNonce||null};
      }
      case'promptAcceptanceStatus':return{status:'accepted'};
      case'respondToWidget':return runtime.respondToWidget(input);
      case'dismissWidget':return runtime.dismissWidget(input);
      case'submitSecret':return runtime.submitSecret(input);
      case'reactToMessage':return runtime.reactToMessage({agentId:input.agentId||input.id,entryId:input.entryId||input.messageId,emoji:input.emoji,userOnly:input.userOnly===true});
      case'resolveAutoReviewApproval':
      case'resolveLocalToolPermission':return runtime.resolveApproval({approvalId:input.approvalId||input.requestId,approved:input.approved===true||input.status==='always'||input.status==='allow-once'||input.permission==='always'});
      case'getAgentChannels':return runtime.getAgentChannels(input);
      case'connectChannel':return runtime.connectChannel(input);
      case'disconnectChannel':return runtime.disconnectChannel(input);
      case'refreshChannel':return runtime.refreshChannel(input);
      case'getAsyncTasks':return runtime.getAsyncTasks(input);
      case'getSubagents':{
        const rows=await agents(),id=input.id||input.agentId;
        return rows.filter(a=>a.parentAgentId===id);
      }
      case'getConversationOutline':{
        const thread=await runtime.getThread({agentId:input.id||input.agentId});
        return Array.isArray(thread.outline)?thread.outline:[];
      }
      case'searchMedia':return runtime.searchMedia(input);
      case'isGlobalSearchEnabled':return true;
      case'isAgentNetworkEnabled':return true;
      case'isEgressTunnelAvailable':return false;
      case'getAgentWorkflows':return runtime.listWorkflows();
      case'createAgentWorkflow':{
        const record=await runtime.saveWorkflow(workflowSpec(input.spec||input));
        return await runtime.listWorkflows();
      }
      case'updateAgentWorkflow':{
        await runtime.saveWorkflow({...workflowSpec(input.spec||input),id:input.workflowId||input.id});
        return await runtime.listWorkflows();
      }
      case'setAgentWorkflowEnabled':{
        await runtime.setWorkflowEnabled({id:input.workflowId,enabled:input.isEnabled===true});
        return await runtime.listWorkflows();
      }
      case'deleteAgentWorkflow':{
        await runtime.deleteWorkflow({id:input.workflowId});
        return await runtime.listWorkflows();
      }
      case'runAgentWorkflowNow':{
        const list=await runtime.listWorkflows(),wf=list.find(x=>x.id===input.workflowId);
        if(!wf)throw Error('Workflow not found.');
        await runtime.sendMessage({agentId:input.id||input.agentId,text:'@'+wf.name});
        return;
      }
      case'importAgentWorkflowText':{
        const spec=workflowSpec({name:input.name||'Imported skill',body:input.text||input.body||'',description:input.description||''});
        const record=await runtime.saveWorkflow(spec);return{imported:[record],failed:[]};
      }
      case'importAgentWorkflowUrl':{
        const raw=String(input.url||'').trim();let url;try{url=new URL(raw)}catch{return{imported:[],failed:[{url:raw,reason:'Invalid skill URL.'}]}}
        const loopback=['localhost','127.0.0.1','::1','[::1]'].includes(url.hostname);if(url.protocol!=='https:'&&!(url.protocol==='http:'&&loopback))return{imported:[],failed:[{url:raw,reason:'Skill URL must use HTTPS.'}]};
        try{const response=await fetch(url,{headers:{accept:'text/markdown, text/plain;q=0.9, */*;q=0.1'},signal:AbortSignal.timeout(12000)});if(!response.ok)throw Error('HTTP '+response.status);const body=await response.text();if(!body.trim())throw Error('Skill URL returned an empty body.');if(body.length>200000)throw Error('Skill document exceeds 200 KB.');const base=url.pathname.split('/').filter(Boolean).at(-1)||'Imported skill',name=String(input.name||base.replace(/\.(md|txt)$/i,'').replace(/[-_]+/g,' ')).trim()||'Imported skill';const record=await runtime.saveWorkflow(workflowSpec({name,description:input.description||'Imported from '+url.hostname,body,sourceRef:url.toString()}));return{imported:[record],failed:[]}}catch(error){return{imported:[],failed:[{url:url.toString(),reason:error instanceof Error?error.message:String(error)}]}}
      }
      case'portAgentLocalSkills':{const imported=await runtime.listWorkflows();return{imported,failed:[]}};
      case'skillsCatalog':return runtime.listWorkflows();
      case'syncPluginSkills':return runtime.listWorkflows();
      case'getPluginSyncStatus':return{status:'ready',isSyncing:false,lastError:null};
      case'listRoutedMcpTools':return runtime.listRoutedMcpTools();
      case'executeRoutedMcpTool':return runtime.executeRoutedMcpTool({name:input.name||input.toolName,args:input.args||input.arguments||{}});
      case'getSkillPublishTargets':return runtime.getSkillPublishTargets();
      case'publishSkill':return runtime.publishSkill({workflowId:input.workflowId,teamId:input.teamId});
      case'resyncPublishedSkill':return runtime.resyncPublishedSkill({workflowId:input.workflowId});
      case'unpublishSkill':return runtime.unpublishSkill({workflowId:input.workflowId});
      case'listAllAutomations':{
        const rows=await runtime.listAgents(),out=[];
        for(const a of rows)for(const item of await runtime.getAgentAutomations({id:a.id}))out.push({...item,agentId:a.id,agentName:a.name});
        return out;
      }
      case'getAgentAutomations':return runtime.getAgentAutomations(input);
      case'createAgentAutomation':return runtime.createAgentAutomation(input);
      case'setAgentAutomationEnabled':return runtime.setAgentAutomationEnabled(input);
      case'updateAgentAutomation':return runtime.updateAgentAutomation(input);
      case'deleteAgentAutomation':return runtime.deleteAgentAutomation(input);
      case'runAgentAutomationNow':return runtime.runAgentAutomationNow(input);
      case'getListenerIntegrations':return runtime.getListenerIntegrations();
      case'getListenerConnectUrl':return runtime.getListenerConnectUrl(input);
      case'getTeachRecordingStatus':return teach.get(input.id||input.agentId)||{status:'idle',agentId:input.id||input.agentId||null};
      case'startTeachRecording':{
        const value={status:'recording',agentId:input.id||input.agentId||null,startedAtMs:Date.now()};teach.set(value.agentId,value);return value;
      }
      case'stopTeachRecording':{
        const id=input.id||input.agentId||null,value={status:'idle',agentId:id,stoppedAtMs:Date.now()};teach.set(id,value);return value;
      }
      case'getForeverBoxStatus':
      case'ensureForeverBox':return await localComputerStatus(input.id||input.agentId);
      case'handBackForeverBox':return;
      case'getTrays':return[];
      case'dismissTray':
      case'clearTrays':return;
      case'getBoxSecretsStatus':return{keys:[],isPersistent:true};
      case'getSharingState':return runtime.getSharingState();
      case'createRoomFromAgent':return runtime.createRoomFromAgent(input);
      case'createRoomInvite':return runtime.createRoomInvite(input);
      case'joinSharedRoom':return runtime.joinSharedRoom(input);
      case'respondToRoomJoinRequest':return runtime.respondToRoomJoinRequest(input);
      case'createSharedRoom':return runtime.createSharedRoom(input);
      case'addOwnAgentToSharedRoom':return runtime.addOwnAgentToSharedRoom(input);
      case'removeOwnAgentFromSharedRoom':return runtime.removeOwnAgentFromSharedRoom(input);
      case'setSharedRoomTyping':return runtime.setSharedRoomTyping(input);
      case'leaveSharedRoom':return runtime.leaveSharedRoom(input);
      case'getCloudAgentInfo':return null;
      case'requestDiskSaverAudit':return{status:'not-needed',computerTarget:'local-mac'};
      case'broadcastToAgents':{
        const ids=Array.isArray(input.ids)?input.ids:(await runtime.listAgents()).filter(a=>!a.isGroup).map(a=>a.id);
        const results=[];for(const id of ids)results.push(await runtime.sendMessage({agentId:id,text:String(input.prompt||input.text||'')}));return{results};
      }
      default:throw Object.assign(Error('Unsupported Grok coordinator method: '+method),{code:'method-unavailable'});
    }
  }
  return{call,agentRow,transcriptEntry};
}
module.exports={createReferenceCoordinator,agentRow,transcriptEntry};
