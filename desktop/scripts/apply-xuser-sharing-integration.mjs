import fs from "node:fs";

const file = "desktop/electron/grok-agent-coordinator.cjs";
let s = fs.readFileSync(file, "utf8");
const before = s;

if (!s.includes("grok-reference-xuser-sharing.cjs")) {
  s = s.replace(
    "const {createExperimentsRuntime}=require('./grok-experiments.cjs');",
    "const {createExperimentsRuntime}=require('./grok-experiments.cjs');\nconst {createLocalXuserSharingRuntime}=require('./grok-reference-xuser-sharing.cjs');"
  );
}

s = s.replace(
  "sharing:{isEnabled:true,selfAuthId:'local-user',pendingJoinRequests:[],rooms:[],typingUsers:[]}",
  "sharing:{isEnabled:false,selfAuthId:null,pendingJoinRequests:[],rooms:[],typingUsers:[]}"
);

const normalizeStart = s.indexOf("  const sharing=parsed.sharing&&typeof parsed.sharing==='object'&&!Array.isArray(parsed.sharing)?parsed.sharing:{};");
const normalizeEnd = normalizeStart >= 0 ? s.indexOf("  base.channels={};", normalizeStart) : -1;
if (normalizeStart >= 0 && normalizeEnd > normalizeStart) {
  s = s.slice(0, normalizeStart) +
    "  base.sharing={isEnabled:false,selfAuthId:null,pendingJoinRequests:[],rooms:[],typingUsers:[]};\n" +
    s.slice(normalizeEnd);
}

if (!s.includes("  let sharingRuntime=null;")) {
  s = s.replace("  const runners=new Map();", "  const runners=new Map();\n  let sharingRuntime=null;");
}

const blockStart = s.indexOf("  function cloneSharingState(s){");
const blockEnd = blockStart >= 0 ? s.indexOf("  async function getAsyncTasks", blockStart) : -1;
if (blockStart >= 0 && blockEnd > blockStart) {
  const realBlock = `  const disabledSharingState=()=>({isEnabled:false,selfAuthId:null,pendingJoinRequests:[],rooms:[],typingUsers:[]});
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
    const result=await sendMessage({agentId,text:String(systemPrompt||'')+'\\n\\n'+String(prompt||''),internal:true,publishSharedReply:true});
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
    await sendMessage({agentId:agent.id,text:'[Shared room message from '+entry.fromUser.name+']\\n'+entry.text,internal:true,publishSharedReply:true});
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

`;
  s = s.slice(0, blockStart) + realBlock + s.slice(blockEnd);
}

const accountLine = "  const accountSession=createAccountSession({secretStore,openExternal:url=>shell.openExternal(url),onChanged:status=>emit('account.changed',{status})});";
if (!s.includes("sharingRuntime=createLocalXuserSharingRuntime({")) {
  if (!s.includes(accountLine)) throw new Error("account session anchor missing");
  s = s.replace(accountLine, accountLine + `
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
  void sharingRuntime.start().catch(()=>{});`);
}

if (!s.includes("await sharingRuntime?.noteAgentDeleted")) {
  s = s.replace(
    "  async function deleteAgent({agentId}){\n    const s=await load();aborts.get(agentId)?.abort();cancelApprovals(agentId,'Agent deleted.');",
    "  async function deleteAgent({agentId}){\n    await sharingRuntime?.noteAgentDeleted(String(agentId||'')).catch(()=>{});\n    const s=await load();aborts.get(agentId)?.abort();cancelApprovals(agentId,'Agent deleted.');"
  );
}

s = s.replace(
  "async function sendMessage({agentId,text,attachmentIds=[],replyToId=null,internal=false})",
  "async function sendMessage({agentId,text,attachmentIds=[],replyToId=null,internal=false,publishSharedReply=false})"
);

if (!s.includes("publishRoomEntry(agent.sharedRoomId,{kind:'message'")) {
  const anchor = "    transcript.push(user);agent.status='thinking';agent.updatedAt=now;await save();";
  if (!s.includes(anchor)) throw new Error("send user anchor missing");
  s = s.replace(anchor, anchor + "\n    if(!internal&&agent.sharedRoomId)void sharingRuntime?.publishRoomEntry(agent.sharedRoomId,{kind:'message',id:user.id,role:'user',content:user.text,clientNonce:user.clientNonce,timestampMs:user.createdAt}).catch(()=>{});");
}

if (!s.includes("publishRoomEntry(agent.sharedRoomId,{kind:'send-message'")) {
  const anchor = "        agent.updatedAt=Date.now();await save();if(transcript.includes(assistant)&&assistant.internal!==true)emit('message.done',{agentId,messageId:assistant.id,text:assistant.text,status:assistant.status});";
  if (!s.includes(anchor)) throw new Error("assistant publish anchor missing");
  s = s.replace(anchor, anchor + "\n        if(agent.sharedRoomId&&assistant.status==='done'&&assistant.text&&(!internal||publishSharedReply))void sharingRuntime?.publishRoomEntry(agent.sharedRoomId,{kind:'send-message',id:assistant.id,message:{type:'text',content:assistant.text},author:{id:agent.id,name:agent.name},streaming:false,timestampMs:assistant.updatedAt||Date.now()}).catch(()=>{});");
}

if (!s.includes("sharingRuntime?.dispose();sharingRuntime=null;")) {
  const anchor = "    await experiments.dispose().catch(()=>{});await accountSession.cancelLogin().catch(()=>{});mcp.dispose();localBrowser.dispose();await attachmentGateway?.dispose?.();";
  if (!s.includes(anchor)) throw new Error("dispose anchor missing");
  s = s.replace(anchor, "    sharingRuntime?.dispose();sharingRuntime=null;\n" + anchor);
}

if (s === before) {
  console.log("xuser sharing integration already applied");
  process.exit(0);
}
fs.writeFileSync(file, s);
console.log("applied xuser sharing integration to", file);
