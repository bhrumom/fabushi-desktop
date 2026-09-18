'use strict';

const path=require('node:path');

const SELF_REACTION='me';
const QUICK_REACTION_EMOJIS=Object.freeze(['👍','👎','❤️','😂','🎉','😮']);

function normalizeReactionEmoji(value){
  const emoji=String(value||'').trim();
  if(!emoji||emoji.length>32)throw Error('Reaction emoji is invalid.');
  return emoji;
}
function normalizedReactions(value){
  if(!Array.isArray(value))return[];
  return value.flatMap(row=>{
    const emoji=String(row?.emoji||'').trim(),by=String(row?.by||'').trim();
    return emoji&&by?[{emoji:emoji.slice(0,32),by:by.slice(0,120)}]:[];
  }).slice(0,200);
}
function toggleSelfReaction(entry,value){
  if(!entry||typeof entry!=='object')throw Error('Message is required.');
  const emoji=normalizeReactionEmoji(value),rows=normalizedReactions(entry.reactions);
  const index=rows.findIndex(row=>row.emoji===emoji&&row.by===SELF_REACTION);
  if(index>=0)rows.splice(index,1);else rows.push({emoji,by:SELF_REACTION});
  entry.reactions=rows;
  return rows;
}
function resolveReplyTarget(messages,targetId){
  const id=String(targetId||'').trim();
  if(!id)return null;
  const row=(Array.isArray(messages)?messages:[]).find(message=>message?.id===id);
  return row&&(row.role==='user'||row.role==='assistant')?row:null;
}
function normalizeUrl(value){
  let raw=String(value||'').trim().replace(/[),.;!?]+$/u,'');
  if(!/^https?:\/\//iu.test(raw))return null;
  try{
    const url=new URL(raw);
    return url.protocol==='http:'||url.protocol==='https:'?url.toString():null;
  }catch{return null}
}
function extractLinks(text){
  const matches=String(text||'').match(/https?:\/\/[^\s<>"'`]+/giu)||[];
  const seen=new Set(),rows=[];
  for(const match of matches){
    const url=normalizeUrl(match);
    if(url&&!seen.has(url)){seen.add(url);rows.push(url)}
  }
  return rows;
}
function attachmentKind(attachment){
  const mime=String(attachment?.mime||'').toLowerCase(),ext=path.extname(String(attachment?.name||'')).toLowerCase();
  if(mime.startsWith('image/'))return'image';
  if(mime.startsWith('video/'))return'video';
  if(mime.startsWith('audio/'))return'audio';
  if(mime==='application/pdf'||ext==='.pdf')return'pdf';
  if(['.md','.mdx'].includes(ext))return'markdown';
  if(['.csv','.tsv','.xls','.xlsx'].includes(ext))return'table';
  if(mime.includes('json')||ext==='.json')return'json';
  if(mime.startsWith('text/'))return'text';
  if(['.doc','.docx','.rtf','.odt'].includes(ext))return'document';
  if(['.zip','.tar','.gz','.tgz','.7z','.rar'].includes(ext))return'archive';
  return'file';
}
function searchWorkspaceIndex({agents=[],messages={},query='',limit=50}={}){
  const needle=String(query||'').trim().toLowerCase(),max=Math.max(1,Math.min(50,Number(limit)||50));
  const names=new Map((Array.isArray(agents)?agents:[]).map(agent=>[String(agent.id),String(agent.name||'Agent')]));
  const messageResults=[],mediaResults=[],linkResults=[],seenLinks=new Set();
  const ordered=[];
  for(const [agentId,rows] of Object.entries(messages&&typeof messages==='object'?messages:{})){
    for(const entry of Array.isArray(rows)?rows:[]){
      if(!entry||typeof entry!=='object')continue;
      ordered.push({agentId,entry});
    }
  }
  ordered.sort((a,b)=>Number(b.entry.createdAt||0)-Number(a.entry.createdAt||0));
  for(const {agentId,entry} of ordered){
    const text=String(entry.text||''),haystack=(names.get(agentId)+' '+text).toLowerCase();
    if((entry.role==='user'||entry.role==='assistant')&&(!needle||haystack.includes(needle))&&messageResults.length<max){
      messageResults.push({agentId,agentName:names.get(agentId)||'Agent',entryId:String(entry.id||''),role:entry.role,text:text.slice(0,2000),timestampMs:Number(entry.createdAt)||0});
    }
    for(const attachment of Array.isArray(entry.attachments)?entry.attachments:[]){
      const fileName=String(attachment?.name||''),fileHaystack=(names.get(agentId)+' '+fileName+' '+String(attachment?.mime||'')).toLowerCase();
      if(mediaResults.length<max&&(!needle||fileHaystack.includes(needle))){
        mediaResults.push({
          agentId,agentName:names.get(agentId)||'Agent',entryId:String(entry.id||''),attachmentId:String(attachment?.id||''),
          fileName,ext:path.extname(fileName).toLowerCase(),mime:attachment?.mime?String(attachment.mime):null,kind:attachmentKind(attachment),
          timestampMs:Number(entry.createdAt)||Number(attachment?.createdAt)||0,width:null,height:null
        });
      }
    }
    for(const url of extractLinks(text)){
      if(seenLinks.has(url))continue;
      const linkHaystack=(names.get(agentId)+' '+url+' '+text).toLowerCase();
      if(needle&&!linkHaystack.includes(needle))continue;
      seenLinks.add(url);
      if(linkResults.length<max)linkResults.push({agentId,agentName:names.get(agentId)||'Agent',entryId:String(entry.id||''),url,timestampMs:Number(entry.createdAt)||0});
    }
  }
  return{messages:messageResults,media:mediaResults,links:linkResults};
}

module.exports={SELF_REACTION,QUICK_REACTION_EMOJIS,normalizeReactionEmoji,normalizedReactions,toggleSelfReaction,resolveReplyTarget,extractLinks,searchWorkspaceIndex};
